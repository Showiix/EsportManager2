use crate::db::*;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::helpers::*;
use super::{GameFlowService, TournamentCreated};

impl GameFlowService {
    /// 初始化赛区常规赛（春季 / 夏季）
    pub(crate) async fn init_regional_regular_season(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
        season_label: &str,
    ) -> Result<Vec<TournamentCreated>, String> {
        let type_str = format!("{:?}", tournament_type);
        let mut tournaments_created = Vec::new();

        // 获取已存在的该类型赛事
        let existing_tournaments = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, &type_str
        ).await.map_err(|e| e.to_string())?;
        log::debug!("已存在的{}数量: {}", season_label, existing_tournaments.len());

        // 获取所有赛区ID
        let region_ids = self.get_all_region_ids(pool, save_id).await?;
        log::debug!("实际赛区ID: {:?}", region_ids);

        // 预加载赛区名称映射
        let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
        let region_name_map: std::collections::HashMap<u64, String> = region_rows.iter()
            .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
            .collect();

        for region_id in region_ids {
            let region_name = region_name_map.get(&region_id).map(|s| s.as_str()).unwrap_or(get_region_name(region_id));
            log::debug!("处理赛区: {} (region_id={})", region_name, region_id);

            // 检查该赛区的赛事是否已存在
            let existing = existing_tournaments.iter()
                .find(|t| t.region_id == Some(region_id));

            let id = if let Some(t) = existing {
                log::debug!("{} {}已存在, id={}", region_name, season_label, t.id);
                t.id
            } else {
                log::debug!("{} {}不存在，创建新赛事", region_name, season_label);
                let tournament = Tournament {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    tournament_type,
                    name: format!("S{} {} {}", season_id, region_name, season_label),
                    region_id: Some(region_id),
                    status: TournamentStatus::Upcoming,
                    current_stage: None,
                    current_round: None,
                };

                TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?
            };

            // 检查是否已有比赛
            let match_count = self.count_tournament_matches(pool, id).await?;
            log::debug!("{} {} id={} 已有比赛数: {}", region_name, season_label, id, match_count);

            if match_count == 0 {
                let teams = TeamRepository::get_by_region(pool, save_id, region_id)
                    .await
                    .map_err(|e| e.to_string())?;

                if teams.len() >= 8 {
                    let matches = self.league_service.generate_regular_schedule(id, &teams);
                    log::debug!("{} 生成比赛数: {}", region_name, matches.len());
                    MatchRepository::create_batch(pool, save_id, &matches)
                        .await
                        .map_err(|e| e.to_string())?;

                    let standings: Vec<LeagueStanding> = teams
                        .iter()
                        .map(|team| LeagueStanding {
                            id: 0,
                            tournament_id: id,
                            team_id: team.id,
                            rank: None,
                            matches_played: 0,
                            wins: 0,
                            losses: 0,
                            points: 0,
                            games_won: 0,
                            games_lost: 0,
                            game_diff: 0,
                        })
                        .collect();

                    StandingRepository::upsert_batch(pool, save_id, &standings)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 生成比赛后将赛事状态更新为 InProgress
                    sqlx::query("UPDATE tournaments SET status = 'InProgress' WHERE id = ?")
                        .bind(id as i64)
                        .execute(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("{} {} id={} 状态更新为 InProgress", region_name, season_label, id);
                }
            }

            tournaments_created.push(TournamentCreated {
                id,
                name: format!("S{} {} {}", season_id, region_name, season_label),
                tournament_type: type_str.clone(),
                region: Some(region_name.to_string()),
            });
        }

        Ok(tournaments_created)
    }

    /// 初始化赛区季后赛（春季 / 夏季）
    pub(crate) async fn init_regional_playoffs(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        playoff_type: TournamentType,
        regular_type_str: &str,
        season_label: &str,
    ) -> Result<Vec<TournamentCreated>, String> {
        let type_str = format!("{:?}", playoff_type);
        let mut tournaments_created = Vec::new();

        // 获取已存在的季后赛赛事
        let existing_playoffs = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, &type_str
        ).await.map_err(|e| e.to_string())?;
        log::debug!("已存在的{}数量: {}", season_label, existing_playoffs.len());

        // 从常规赛获取实际的 region_id
        let regular_tournaments = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, regular_type_str
        ).await.map_err(|e| e.to_string())?;
        log::debug!("常规赛数量: {}", regular_tournaments.len());

        // 预加载赛区名称映射
        let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
        let region_name_map: std::collections::HashMap<u64, String> = region_rows.iter()
            .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
            .collect();

        for regular_tournament in &regular_tournaments {
            let region_id = match regular_tournament.region_id {
                Some(id) => id,
                None => continue,
            };
            let region_name = region_name_map.get(&region_id).map(|s| s.as_str()).unwrap_or(get_region_name(region_id));
            log::debug!("处理赛区: {} (region_id={})", region_name, region_id);

            // 检查该赛区的季后赛是否已存在
            let existing = existing_playoffs.iter()
                .find(|t| t.region_id == Some(region_id));

            let id = if let Some(t) = existing {
                log::debug!("{} {}已存在, id={}", region_name, season_label, t.id);
                t.id
            } else {
                log::debug!("{} {}不存在，创建新赛事", region_name, season_label);
                let tournament = Tournament {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    tournament_type: playoff_type,
                    name: format!("S{} {} {}", season_id, region_name, season_label),
                    region_id: Some(region_id),
                    status: TournamentStatus::Upcoming,
                    current_stage: None,
                    current_round: None,
                };
                TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?
            };

            // 检查是否已有比赛
            let match_count = self.count_tournament_matches(pool, id).await?;
            log::debug!("{} {} id={} 已有比赛数: {}", region_name, season_label, id, match_count);

            if match_count == 0 {
                let standings = StandingRepository::get_by_tournament(pool, regular_tournament.id)
                    .await
                    .map_err(|e| e.to_string())?;
                log::debug!("{} 常规赛积分榜队伍数: {}", region_name, standings.len());

                if standings.len() >= 8 {
                    let matches = self.league_service.generate_playoff_bracket(id, &standings);
                    log::debug!("{} 生成比赛数: {}", region_name, matches.len());
                    if !matches.is_empty() {
                        MatchRepository::create_batch(pool, save_id, &matches)
                            .await
                            .map_err(|e| e.to_string())?;
                        log::debug!("{} 比赛已创建", region_name);
                    }
                } else {
                    log::debug!("{} 积分榜队伍不足8支，跳过", region_name);
                }
            } else {
                log::debug!("{} 已有比赛，跳过生成", region_name);
            }

            tournaments_created.push(TournamentCreated {
                id,
                name: format!("S{} {} {}", season_id, region_name, season_label),
                tournament_type: type_str.clone(),
                region: Some(region_name.to_string()),
            });
        }

        Ok(tournaments_created)
    }

    /// 初始化 32 队大师赛（马德里大师赛 / Claude洲际赛）
    pub(crate) async fn init_32team_masters(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
        source_regular: &str,
        tournament_name: &str,
    ) -> Result<Vec<TournamentCreated>, String> {
        let type_str = format!("{:?}", tournament_type);
        let mut tournaments_created = Vec::new();

        // 先检查是否已存在
        let existing = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, &type_str
        ).await.map_err(|e| e.to_string())?;

        let id = if let Some(t) = existing.first() {
            log::debug!("赛事已存在, id={}", t.id);
            t.id
        } else {
            let tournament = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                season_id,
                tournament_type,
                name: format!("S{} {}", season_id, tournament_name),
                region_id: None,
                status: TournamentStatus::Upcoming,
                current_stage: None,
                current_round: None,
            };

            TournamentRepository::create(pool, save_id, &tournament)
                .await
                .map_err(|e| e.to_string())?
        };

        // 检查是否已有比赛
        let match_count = self.count_tournament_matches(pool, id).await?;

        if match_count == 0 {
            // 获取参赛队伍：各赛区常规赛前8名 (共32队)
            let mut teams = Vec::new();

            let regular_tournaments = TournamentRepository::get_by_season_and_type(
                pool, save_id, season_id, source_regular
            ).await.map_err(|e| e.to_string())?;

            log::debug!("找到 {} 个常规赛", regular_tournaments.len());

            for regular in regular_tournaments {
                let standings = StandingRepository::get_by_tournament(pool, regular.id)
                    .await
                    .map_err(|e| e.to_string())?;

                log::debug!("赛区 {:?} 积分榜有 {} 支队伍", regular.region_id, standings.len());

                for standing in standings.iter().take(8) {
                    if let Ok(team) = TeamRepository::get_by_id(pool, standing.team_id).await {
                        if !teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                            teams.push(team);
                        }
                    }
                }
            }

            log::debug!("找到 {} 支参赛队伍", teams.len());

            // 如果队伍不足32支，用各赛区其他队伍填充
            if teams.len() < 32 {
                let fill_region_ids = self.get_all_region_ids(pool, save_id).await?;
                for region_id in fill_region_ids {
                    let region_teams = TeamRepository::get_by_region(pool, save_id, region_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    for team in region_teams {
                        if teams.len() >= 32 {
                            break;
                        }
                        if !teams.iter().any(|t| t.id == team.id) {
                            teams.push(team);
                        }
                    }
                }
            }

            if teams.len() >= 32 {
                let matches = self.tournament_service.generate_masters_bracket(id, &teams[..32]);
                log::debug!("生成 {} 场比赛", matches.len());
                if !matches.is_empty() {
                    MatchRepository::create_batch(pool, save_id, &matches)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 初始化积分榜（小组赛阶段需要）
                    let standings: Vec<LeagueStanding> = teams[..32]
                        .iter()
                        .map(|team| LeagueStanding {
                            id: 0,
                            tournament_id: id,
                            team_id: team.id,
                            rank: None,
                            matches_played: 0,
                            wins: 0,
                            losses: 0,
                            points: 0,
                            games_won: 0,
                            games_lost: 0,
                            game_diff: 0,
                        })
                        .collect();

                    StandingRepository::upsert_batch(pool, save_id, &standings)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("初始化 {} 支队伍的积分榜", standings.len());
                }
            } else {
                log::debug!("队伍不足32支 (只有{}支)，跳过比赛生成", teams.len());
            }
        }

        tournaments_created.push(TournamentCreated {
            id,
            name: format!("S{} {}", season_id, tournament_name),
            tournament_type: type_str,
            region: None,
        });

        Ok(tournaments_created)
    }

}
