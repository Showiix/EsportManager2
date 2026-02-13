use crate::db::*;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::helpers::*;
use super::{GameFlowService, PhaseInitResult, TournamentCreated};

impl GameFlowService {
    /// 初始化阶段 - 为当前阶段创建对应的赛事
    pub async fn initialize_phase(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        phase: SeasonPhase,
    ) -> Result<PhaseInitResult, String> {
        // 检查该阶段的赛事是否已经存在
        if let Some(tournament_type) = phase.to_tournament_type() {
            let existing = self.get_phase_tournaments(pool, save_id, season_id, tournament_type).await?;
            log::debug!("阶段 {:?}, 已存在赛事数: {}", phase, existing.len());
            if !existing.is_empty() {
                // 检查是否【所有】赛事都有比赛
                // 只有所有赛事都有比赛时才跳过，否则继续初始化（为没有比赛的赛事生成比赛）
                let mut all_have_matches = true;
                let mut total_matches = 0i64;
                for tournament in &existing {
                    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE tournament_id = ?")
                        .bind(tournament.id as i64)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("赛事 {} (id={}) 比赛数: {}", tournament.name, tournament.id, count.0);
                    total_matches += count.0;
                    if count.0 == 0 {
                        all_have_matches = false;
                        // 不要 break，继续统计总数
                    }
                }
                log::debug!("all_have_matches={}, total_matches={}", all_have_matches, total_matches);

                if all_have_matches && total_matches > 0 {
                    log::debug!("所有赛事都有比赛，跳过初始化");
                    // 预加载赛区名称，避免硬编码ID映射
                    let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
                        .bind(save_id)
                        .fetch_all(pool)
                        .await
                        .unwrap_or_default();
                    let region_map: std::collections::HashMap<u64, String> = region_rows.iter()
                        .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
                        .collect();
                    return Ok(PhaseInitResult {
                        phase: format!("{:?}", phase),
                        tournaments_created: existing.iter().map(|t| TournamentCreated {
                            id: t.id,
                            name: t.name.clone(),
                            tournament_type: format!("{:?}", t.tournament_type),
                            region: t.region_id.map(|r| region_map.get(&r).cloned().unwrap_or_else(|| get_region_name(r).to_string())),
                        }).collect(),
                        message: format!("阶段 {:?} 的赛事已存在，跳过初始化", phase),
                    });
                }
                log::debug!("有赛事没有比赛，继续初始化");
                // 有赛事没有比赛，继续生成比赛
            }
        }

        let mut tournaments_created = Vec::new();

        match phase {
            // 春季常规赛 - 为4个赛区各创建一个常规赛赛事
            SeasonPhase::SpringRegular => {
                tournaments_created = self.init_regional_regular_season(
                    pool, save_id, season_id, TournamentType::SpringRegular, "春季赛"
                ).await?;
            }

            // 春季季后赛 - 为4个赛区各创建季后赛
            SeasonPhase::SpringPlayoffs => {
                tournaments_created = self.init_regional_playoffs(
                    pool, save_id, season_id, TournamentType::SpringPlayoffs, "SpringRegular", "春季季后赛"
                ).await?;
            }

            // MSI - 创建全球性赛事并自动生成对阵
            SeasonPhase::Msi => {
                // 检查是否已存在MSI赛事（避免重复创建）
                let existing_msi = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "Msi"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(existing) = existing_msi.into_iter().find(|t| t.status == TournamentStatus::Upcoming) {
                    log::debug!("[MSI Init] 使用已存在的MSI赛事: id={}, name={}", existing.id, existing.name);
                    existing.id
                } else {
                    // 创建新的MSI赛事
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::Msi,
                        name: format!("S{} MSI季中赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[MSI Init] 创建新MSI赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[MSI Init] MSI赛事已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} MSI季中赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::Msi),
                        region: None,
                    });
                } else {
                    // 从春季季后赛获取各赛区的冠亚季军
                    let mut legendary_teams = Vec::new();  // 冠军队伍
                    let mut challenger_teams = Vec::new(); // 亚军队伍
                    let mut qualifier_teams = Vec::new();  // 季军队伍

                    // 直接获取所有春季季后赛，不依赖固定的 region_id
                    let all_playoffs = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SpringPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("[MSI Init] 找到 {} 个春季季后赛赛事", all_playoffs.len());
                    for playoff in &all_playoffs {
                        log::debug!("[MSI Init] 季后赛: id={}, name={}, status={:?}",
                            playoff.id, playoff.name, playoff.status);
                    }

                    for playoff_tournament in all_playoffs {
                        // 获取该季后赛的最终排名
                        let results = self.get_playoffs_top3(pool, playoff_tournament.id).await?;

                        if results.len() >= 3 {
                            legendary_teams.push(results[0].clone());  // 冠军
                            challenger_teams.push(results[1].clone()); // 亚军
                            qualifier_teams.push(results[2].clone());  // 季军
                            log::debug!("从赛事 {} 获取到前3名队伍", playoff_tournament.name);
                        } else {
                            log::debug!("赛事 {} 结果不足3支队伍: {}", playoff_tournament.name, results.len());
                        }
                    }

                    // 如果所有队伍都已就位，生成MSI对阵
                    if legendary_teams.len() == 4 && challenger_teams.len() == 4 && qualifier_teams.len() == 4 {
                        let matches = self.tournament_service.generate_msi_bracket(
                            id,
                            &legendary_teams,
                            &challenger_teams,
                            &qualifier_teams,
                        );

                        MatchRepository::create_batch(pool, save_id, &matches)
                            .await
                            .map_err(|e| e.to_string())?;

                        log::debug!("MSI 生成了 {} 场比赛", matches.len());
                    } else {
                        log::debug!("MSI 队伍不足: legendary={}, challenger={}, qualifier={}",
                            legendary_teams.len(), challenger_teams.len(), qualifier_teams.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} MSI季中赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::Msi),
                        region: None,
                    });
                }
            }

            // 马德里大师赛
            SeasonPhase::MadridMasters => {
                tournaments_created = self.init_32team_masters(
                    pool, save_id, season_id, TournamentType::MadridMasters, "SpringRegular", "马德里大师赛"
                ).await?;
            }

            // 夏季常规赛
            SeasonPhase::SummerRegular => {
                tournaments_created = self.init_regional_regular_season(
                    pool, save_id, season_id, TournamentType::SummerRegular, "夏季赛"
                ).await?;
            }

            // 夏季季后赛
            SeasonPhase::SummerPlayoffs => {
                tournaments_created = self.init_regional_playoffs(
                    pool, save_id, season_id, TournamentType::SummerPlayoffs, "SummerRegular", "夏季季后赛"
                ).await?;
            }

            // Claude洲际赛 (类似马德里大师赛：32队分组+东西半区淘汰)
            SeasonPhase::ClaudeIntercontinental => {
                tournaments_created = self.init_32team_masters(
                    pool, save_id, season_id, TournamentType::ClaudeIntercontinental, "SummerRegular", "Claude洲际赛"
                ).await?;
            }

            // 世界赛
            SeasonPhase::WorldChampionship => {
                // 先检查是否已存在
                let existing = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "WorldChampionship"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(existing_tournament) = existing.first() {
                    log::debug!("赛事已存在, id={}", existing_tournament.id);
                    existing_tournament.id
                } else {
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::WorldChampionship,
                        name: format!("S{} 世界赛", season_id),
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
                let match_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE tournament_id = ?")
                    .bind(id as i64)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

                if match_count.0 == 0 {
                    // 获取参赛队伍：各赛区夏季季后赛冠军(4队)直接进淘汰赛，亚军+季军(8队)进瑞士轮
                    let mut direct_teams = Vec::new();  // 冠军队伍 - 直接进淘汰赛
                    let mut swiss_teams = Vec::new();   // 亚军+季军队伍 - 进瑞士轮

                    // 从各赛区夏季季后赛获取排名
                    let playoffs_tournaments = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SummerPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("找到 {} 个夏季季后赛", playoffs_tournaments.len());

                    for playoffs in playoffs_tournaments {
                        // 获取季后赛最终排名（从荣誉记录获取）
                        // 冠军
                        let champion_row = sqlx::query(
                            "SELECT team_id FROM honors WHERE tournament_id = ? AND honor_type = 'TEAM_CHAMPION'"
                        )
                        .bind(playoffs.id as i64)
                        .fetch_optional(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                        if let Some(row) = champion_row {
                            let team_id: i64 = row.get("team_id");
                            if let Ok(team) = TeamRepository::get_by_id(pool, team_id as u64).await {
                                if !direct_teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                                    direct_teams.push(team);
                                    log::debug!("赛区 {:?} 冠军: team_id={}", playoffs.region_id, team_id);
                                }
                            }
                        }

                        // 亚军
                        let runner_up_row = sqlx::query(
                            "SELECT team_id FROM honors WHERE tournament_id = ? AND honor_type = 'TEAM_RUNNER_UP'"
                        )
                        .bind(playoffs.id as i64)
                        .fetch_optional(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                        if let Some(row) = runner_up_row {
                            let team_id: i64 = row.get("team_id");
                            if let Ok(team) = TeamRepository::get_by_id(pool, team_id as u64).await {
                                if !swiss_teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                                    swiss_teams.push(team);
                                    log::debug!("赛区 {:?} 亚军: team_id={}", playoffs.region_id, team_id);
                                }
                            }
                        }

                        // 季军
                        let third_row = sqlx::query(
                            "SELECT team_id FROM honors WHERE tournament_id = ? AND honor_type = 'TEAM_THIRD'"
                        )
                        .bind(playoffs.id as i64)
                        .fetch_optional(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                        if let Some(row) = third_row {
                            let team_id: i64 = row.get("team_id");
                            if let Ok(team) = TeamRepository::get_by_id(pool, team_id as u64).await {
                                if !swiss_teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                                    swiss_teams.push(team);
                                    log::debug!("赛区 {:?} 季军: team_id={}", playoffs.region_id, team_id);
                                }
                            }
                        }
                    }

                    log::debug!("冠军队伍: {} 支, 瑞士轮队伍: {} 支", direct_teams.len(), swiss_teams.len());

                    // 如果队伍不足，从夏季常规赛积分榜补充
                    if direct_teams.len() < 4 || swiss_teams.len() < 8 {
                        log::debug!("队伍不足，从常规赛积分榜补充");
                        let regular_tournaments = TournamentRepository::get_by_season_and_type(
                            pool, save_id, season_id, "SummerRegular"
                        ).await.map_err(|e| e.to_string())?;

                        for regular in regular_tournaments {
                            let standings = StandingRepository::get_by_tournament(pool, regular.id)
                                .await
                                .map_err(|e| e.to_string())?;

                            for (idx, standing) in standings.iter().enumerate() {
                                if let Ok(team) = TeamRepository::get_by_id(pool, standing.team_id).await {
                                    // 前1名补充到直接晋级
                                    if idx == 0 && direct_teams.len() < 4 {
                                        if !direct_teams.iter().any(|t| t.id == team.id) && !swiss_teams.iter().any(|t| t.id == team.id) {
                                            direct_teams.push(team);
                                            continue;
                                        }
                                    }
                                    // 2-3名补充到瑞士轮
                                    if idx < 3 && swiss_teams.len() < 8 {
                                        if !direct_teams.iter().any(|t| t.id == team.id) && !swiss_teams.iter().any(|t| t.id == team.id) {
                                            swiss_teams.push(team.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    log::debug!("最终: 冠军队伍 {} 支, 瑞士轮队伍 {} 支", direct_teams.len(), swiss_teams.len());

                    if direct_teams.len() >= 4 && swiss_teams.len() >= 8 {
                        let matches = self.tournament_service.generate_worlds_bracket(
                            id,
                            &direct_teams[..4],
                            &swiss_teams[..8]
                        );
                        log::debug!("生成 {} 场比赛", matches.len());
                        if !matches.is_empty() {
                            MatchRepository::create_batch(pool, save_id, &matches)
                                .await
                                .map_err(|e| e.to_string())?;

                            // 初始化积分榜（瑞士轮阶段需要）
                            let mut all_teams = swiss_teams.clone();
                            all_teams.extend(direct_teams.clone());
                            let standings: Vec<LeagueStanding> = all_teams
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
                        log::debug!("队伍不足，无法生成比赛: direct={}, swiss={}", direct_teams.len(), swiss_teams.len());
                    }
                }

                tournaments_created.push(TournamentCreated {
                    id,
                    name: format!("S{} 世界赛", season_id),
                    tournament_type: format!("{:?}", TournamentType::WorldChampionship),
                    region: None,
                });
            }

            // 上海大师赛 - 与MSI相同的双败淘汰赛制，参赛队伍来自夏季季后赛
            SeasonPhase::ShanghaiMasters => {
                // 检查是否已存在上海大师赛赛事（避免重复创建）
                let existing_shanghai = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "ShanghaiMasters"
                ).await.map_err(|e| e.to_string())?;

                // 清理重复的上海大师赛赛事 (保留第一个有效的，删除其他的)
                let valid_tournaments: Vec<_> = existing_shanghai.into_iter()
                    .filter(|t| t.status == TournamentStatus::Upcoming || t.status == TournamentStatus::InProgress)
                    .collect();

                // 获取第一个有效赛事的ID（如果存在）
                let first_valid_id = valid_tournaments.first().map(|t| t.id);

                // 如果有多个，删除多余的
                if valid_tournaments.len() > 1 {
                    log::debug!("[ShanghaiMasters Init] 发现 {} 个重复的上海大师赛，清理中...", valid_tournaments.len());
                    // 保留第一个，删除其他
                    for extra_tournament in valid_tournaments.iter().skip(1) {
                        log::debug!("[ShanghaiMasters Init] 删除重复赛事: id={}", extra_tournament.id);
                        // 先删除比赛
                        sqlx::query("DELETE FROM matches WHERE tournament_id = ?")
                            .bind(extra_tournament.id as i64)
                            .execute(pool)
                            .await
                            .ok();
                        // 删除赛事
                        sqlx::query("DELETE FROM tournaments WHERE id = ?")
                            .bind(extra_tournament.id as i64)
                            .execute(pool)
                            .await
                            .ok();
                    }
                }

                let id = if let Some(existing_id) = first_valid_id {
                    log::debug!("[ShanghaiMasters Init] 使用已存在的上海大师赛赛事: id={}", existing_id);
                    existing_id
                } else {
                    // 创建新的上海大师赛赛事
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::ShanghaiMasters,
                        name: format!("S{} 上海大师赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[ShanghaiMasters Init] 创建新上海大师赛赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[ShanghaiMasters Init] 上海大师赛已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} 上海大师赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::ShanghaiMasters),
                        region: None,
                    });
                } else {
                    // 从夏季季后赛获取各赛区的冠亚季军
                    let mut legendary_teams = Vec::new();  // 冠军队伍
                    let mut challenger_teams = Vec::new(); // 亚军队伍
                    let mut qualifier_teams = Vec::new();  // 季军队伍

                    // 获取所有夏季季后赛
                    let all_playoffs = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SummerPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("[ShanghaiMasters Init] 找到 {} 个夏季季后赛赛事", all_playoffs.len());
                    for playoff in &all_playoffs {
                        log::debug!("[ShanghaiMasters Init] 季后赛: id={}, name={}, status={:?}",
                            playoff.id, playoff.name, playoff.status);
                    }

                    for playoff_tournament in all_playoffs {
                        // 获取该季后赛的最终排名
                        let results = self.get_playoffs_top3(pool, playoff_tournament.id).await?;

                        if results.len() >= 3 {
                            legendary_teams.push(results[0].clone());  // 冠军
                            challenger_teams.push(results[1].clone()); // 亚军
                            qualifier_teams.push(results[2].clone());  // 季军
                            log::debug!("从赛事 {} 获取到前3名队伍", playoff_tournament.name);
                        } else {
                            log::debug!("赛事 {} 结果不足3支队伍: {}", playoff_tournament.name, results.len());
                        }
                    }

                    // 如果所有队伍都已就位，生成上海大师赛对阵（与MSI相同格式）
                    if legendary_teams.len() == 4 && challenger_teams.len() == 4 && qualifier_teams.len() == 4 {
                        let matches = self.tournament_service.generate_msi_bracket(
                            id,
                            &legendary_teams,
                            &challenger_teams,
                            &qualifier_teams,
                        );

                        if !matches.is_empty() {
                            MatchRepository::create_batch(pool, save_id, &matches)
                                .await
                                .map_err(|e| e.to_string())?;

                            // 更新赛事状态为进行中
                            TournamentRepository::update_status(pool, id, TournamentStatus::InProgress)
                                .await
                                .map_err(|e| e.to_string())?;

                            log::debug!("成功生成 {} 场比赛并更新状态为进行中", matches.len());
                        }
                    } else {
                        log::debug!("队伍不足: legendary={}, challenger={}, qualifier={}",
                            legendary_teams.len(), challenger_teams.len(), qualifier_teams.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} 上海大师赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::ShanghaiMasters),
                        region: None,
                    });
                }
            }

            // ICP洲际对抗赛
            SeasonPhase::IcpIntercontinental => {
                // 检查是否已存在ICP赛事
                let existing = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "IcpIntercontinental"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(t) = existing.first() {
                    log::debug!("[ICP Init] 找到已存在的ICP赛事: id={}", t.id);
                    t.id
                } else {
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::IcpIntercontinental,
                        name: format!("S{} ICP洲际对抗赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[ICP Init] 创建新ICP赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[ICP Init] ICP赛事已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} ICP洲际对抗赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::IcpIntercontinental),
                        region: None,
                    });
                } else {
                    // 从夏季季后赛获取各赛区的前4名
                    let mut region_teams: Vec<Vec<crate::models::Team>> = Vec::new();

                    // 获取所有夏季季后赛
                    let all_playoffs = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SummerPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("[ICP Init] 找到 {} 个夏季季后赛赛事", all_playoffs.len());

                    for playoff_tournament in all_playoffs {
                        // 获取该季后赛的前4名
                        let results = self.get_playoffs_top4(pool, playoff_tournament.id).await?;

                        if results.len() >= 4 {
                            region_teams.push(results);
                            log::debug!("[ICP Init] 从赛事 {} 获取到前4名队伍", playoff_tournament.name);
                        } else {
                            log::debug!("[ICP Init] 赛事 {} 结果不足4支队伍: {}", playoff_tournament.name, results.len());
                        }
                    }

                    // 如果所有4个赛区的前4名都已就位，生成ICP对阵
                    if region_teams.len() == 4 {
                        let matches = self.tournament_service.generate_icp_bracket(
                            id,
                            region_teams,
                        );

                        if !matches.is_empty() {
                            MatchRepository::create_batch(pool, save_id, &matches)
                                .await
                                .map_err(|e| e.to_string())?;

                            // 更新赛事状态为进行中
                            TournamentRepository::update_status(pool, id, TournamentStatus::InProgress)
                                .await
                                .map_err(|e| e.to_string())?;

                            log::debug!("[ICP Init] 成功生成 {} 场比赛并更新状态为进行中", matches.len());
                        }
                    } else {
                        log::debug!("[ICP Init] 赛区队伍不足: 需要4个赛区各4队，当前有 {} 个赛区数据", region_teams.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} ICP洲际对抗赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::IcpIntercontinental),
                        region: None,
                    });
                }
            }

            // Super洲际邀请赛
            SeasonPhase::SuperIntercontinental => {
                // 检查是否已存在Super赛事
                let existing = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "SuperIntercontinental"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(t) = existing.first() {
                    log::debug!("[Super Init] 找到已存在的Super赛事: id={}", t.id);
                    t.id
                } else {
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::SuperIntercontinental,
                        name: format!("S{} Super洲际邀请赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[Super Init] 创建新Super赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[Super Init] Super赛事已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} Super洲际邀请赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::SuperIntercontinental),
                        region: None,
                    });
                } else {
                    // 从年度积分获取Top16队伍
                    let rankings = PointsRepository::get_season_rankings(pool, save_id, season_id)
                        .await
                        .map_err(|e| e.to_string())?;

                    log::debug!("[Super Init] 年度积分排名队伍数: {}", rankings.len());

                    if rankings.len() >= 16 {
                        // 获取队伍信息
                        let mut legendary_teams = Vec::new();
                        let mut challenger_teams = Vec::new();
                        let mut fighter_teams = Vec::new();

                        for (idx, ranking) in rankings.iter().take(16).enumerate() {
                            if let Ok(team) = TeamRepository::get_by_id(pool, ranking.team_id).await {
                                match idx {
                                    0..=3 => legendary_teams.push(team),    // 1-4名: 传奇组
                                    4..=7 => challenger_teams.push(team),   // 5-8名: 挑战者组
                                    8..=15 => fighter_teams.push(team),     // 9-16名: Fighter组
                                    _ => {}
                                }
                            }
                        }

                        log::debug!("[Super Init] 传奇组: {} 队, 挑战者组: {} 队, Fighter组: {} 队",
                            legendary_teams.len(), challenger_teams.len(), fighter_teams.len());

                        if legendary_teams.len() == 4 && challenger_teams.len() == 4 && fighter_teams.len() == 8 {
                            let matches = self.tournament_service.generate_super_bracket(
                                id,
                                &legendary_teams,
                                &challenger_teams,
                                &fighter_teams,
                            );

                            if !matches.is_empty() {
                                MatchRepository::create_batch(pool, save_id, &matches)
                                    .await
                                    .map_err(|e| e.to_string())?;

                                // 更新赛事状态为进行中
                                TournamentRepository::update_status(pool, id, TournamentStatus::InProgress)
                                    .await
                                    .map_err(|e| e.to_string())?;

                                log::debug!("[Super Init] 成功生成 {} 场比赛并更新状态为进行中", matches.len());
                            }
                        } else {
                            log::debug!("[Super Init] 队伍数量不足: 传奇组需4队，挑战者组需4队，Fighter组需8队");
                        }
                    } else {
                        log::debug!("[Super Init] 年度积分排名不足16队: 当前 {} 队", rankings.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} Super洲际邀请赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::SuperIntercontinental),
                        region: None,
                    });
                }
            }

            // 年度颁奖典礼 - 不创建赛事，仅显示页面
            SeasonPhase::AnnualAwards => {}

            // 转会期 - 不创建赛事
            SeasonPhase::TransferWindow => {}

            // 选秀 - 不创建赛事
            SeasonPhase::Draft => {}

            // 赛季结束 - 不创建赛事
            SeasonPhase::SeasonEnd => {}
        }

        let message = if tournaments_created.is_empty() {
            format!("{:?} 阶段无需创建赛事", phase)
        } else {
            format!(
                "成功创建 {} 个赛事",
                tournaments_created.len()
            )
        };

        Ok(PhaseInitResult {
            phase: format!("{:?}", phase),
            tournaments_created,
            message,
        })
    }
}
