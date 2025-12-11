//! 荣誉服务
//!
//! 负责从赛事结果推断并颁发荣誉

use crate::db::{
    HonorRepository, MatchRepository, PlayerRepository, PlayerTournamentStatsRepository,
    StandingRepository, TeamRepository, TournamentRepository, TournamentResultRepository,
};
use crate::engines::HonorEngine;
use crate::models::{
    Honor, HonorType, MatchStatus, TournamentResult,
};
use crate::models::tournament_result::TournamentHonors;
use sqlx::{Pool, Sqlite};

/// 荣誉服务
pub struct HonorService {
    honor_engine: HonorEngine,
}

impl Default for HonorService {
    fn default() -> Self {
        Self {
            honor_engine: HonorEngine::new(),
        }
    }
}

impl HonorService {
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理赛事结束 - 推断结果并颁发荣誉
    ///
    /// # Arguments
    /// * `pool` - 数据库连接池
    /// * `save_id` - 存档ID
    /// * `tournament_id` - 赛事ID
    ///
    /// # Returns
    /// * `TournamentHonors` - 颁发的荣誉列表
    pub async fn process_tournament_completion(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<TournamentHonors, String> {
        // 1. 获取赛事信息
        let tournament = TournamentRepository::get_by_id(pool, tournament_id)
            .await
            .map_err(|e| format!("Failed to get tournament: {}", e))?;

        let tournament_type = format!("{:?}", tournament.tournament_type);
        let is_international = tournament.tournament_type.is_international();
        let has_third_place = is_international; // 国际赛事有季军殿军

        // 2. 从比赛记录推断赛事结果
        let result = self
            .infer_tournament_result(pool, save_id, &tournament)
            .await?;

        // 3. 保存赛事结果
        TournamentResultRepository::create(pool, &result)
            .await
            .map_err(|e| format!("Failed to save tournament result: {}", e))?;

        // 4. 获取冠军队选手
        let champion_players = PlayerRepository::get_by_team(pool, result.champion_team_id)
            .await
            .map_err(|e| format!("Failed to get champion players: {}", e))?;

        // 5. 获取赛事MVP（从 player_tournament_stats）
        let mvp_candidates = PlayerTournamentStatsRepository::get_mvp_candidates(pool, save_id, tournament_id, 1)
            .await
            .map_err(|e| format!("Failed to get MVP candidates: {}", e))?;

        let tournament_mvp = mvp_candidates.first();

        // 6. 计算决赛MVP（暂时跳过，因为需要 player_match_stats 表）
        // let finals_mvp = self
        //     .calculate_finals_mvp(pool, save_id, tournament_id)
        //     .await?;

        // 7. 直接生成荣誉（避免生命周期问题）
        let mut tournament_honors = TournamentHonors::default();

        // 7.1 战队冠军
        tournament_honors.team_champion = Some(self.honor_engine.create_team_champion(
            save_id,
            tournament.season_id,
            tournament_id,
            &tournament.name,
            &tournament_type,
            result.champion_team_id,
            &result.champion_team_name,
        ));

        // 7.2 战队亚军
        tournament_honors.team_runner_up = Some(self.honor_engine.create_team_runner_up(
            save_id,
            tournament.season_id,
            tournament_id,
            &tournament.name,
            &tournament_type,
            result.runner_up_team_id,
            &result.runner_up_team_name,
        ));

        // 7.3 战队季军（仅国际赛事）
        if has_third_place {
            if let (Some(third_id), Some(third_name)) = (result.third_team_id, &result.third_team_name) {
                tournament_honors.team_third = Some(self.honor_engine.create_team_third(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    third_id,
                    third_name,
                ));
            }

            // 7.4 战队殿军
            if let (Some(fourth_id), Some(fourth_name)) = (result.fourth_team_id, &result.fourth_team_name) {
                tournament_honors.team_fourth = Some(self.honor_engine.create_team_fourth(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    fourth_id,
                    fourth_name,
                ));
            }
        }

        // 7.5 冠军队选手荣誉
        for player in champion_players.iter().filter(|p| p.is_starter) {
            let pos_str = player.position
                .as_ref()
                .map(|pos| format!("{:?}", pos))
                .unwrap_or_else(|| "Unknown".to_string());

            tournament_honors.player_champions.push(self.honor_engine.create_player_champion(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                result.champion_team_id,
                &result.champion_team_name,
                player.id,
                &player.game_id,
                &pos_str,
            ));
        }

        // 7.6 赛事MVP
        if let Some(mvp_stats) = tournament_mvp {
            tournament_honors.tournament_mvp = Some(self.honor_engine.create_tournament_mvp(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                mvp_stats,
            ));
        }

        // 7.7 决赛MVP（暂时跳过，需要 player_match_stats 数据）
        // 可以在后续版本中实现

        // 8. 保存荣誉到数据库
        let mut all_honors = Vec::new();

        if let Some(ref honor) = tournament_honors.team_champion {
            all_honors.push(honor.clone());
        }
        if let Some(ref honor) = tournament_honors.team_runner_up {
            all_honors.push(honor.clone());
        }
        if let Some(ref honor) = tournament_honors.team_third {
            all_honors.push(honor.clone());
        }
        if let Some(ref honor) = tournament_honors.team_fourth {
            all_honors.push(honor.clone());
        }
        all_honors.extend(tournament_honors.player_champions.clone());
        if let Some(ref honor) = tournament_honors.tournament_mvp {
            all_honors.push(honor.clone());
        }
        if let Some(ref honor) = tournament_honors.finals_mvp {
            all_honors.push(honor.clone());
        }

        HonorRepository::create_batch(pool, save_id, &all_honors)
            .await
            .map_err(|e| format!("Failed to save honors: {}", e))?;

        Ok(tournament_honors)
    }

    /// 从比赛记录推断赛事结果
    async fn infer_tournament_result(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament: &crate::models::Tournament,
    ) -> Result<TournamentResult, String> {
        let tournament_id = tournament.id;
        let tournament_type = format!("{:?}", tournament.tournament_type);

        // 获取所有比赛
        let matches = MatchRepository::get_by_tournament(pool, tournament_id)
            .await
            .map_err(|e| format!("Failed to get matches: {}", e))?;

        let completed_matches: Vec<_> = matches
            .iter()
            .filter(|m| m.status == MatchStatus::Completed)
            .collect();

        if completed_matches.is_empty() {
            return Err("No completed matches found".to_string());
        }

        // 计算统计
        let total_matches = completed_matches.len() as u32;
        let total_games: u32 = completed_matches
            .iter()
            .map(|m| m.home_score as u32 + m.away_score as u32)
            .sum();

        // 查找决赛
        let final_match = completed_matches
            .iter()
            .find(|m| m.stage.to_lowercase().contains("final") || m.stage.to_lowercase().contains("决赛"))
            .or_else(|| {
                // 如果没有标记为决赛，找最后一场比赛
                completed_matches.iter().max_by_key(|m| m.id)
            });

        let (champion_team_id, runner_up_team_id, final_match_id, final_score) = if let Some(final_m) = final_match {
            let winner_id = final_m.winner_id.ok_or("Final match has no winner")?;
            let loser_id = if final_m.home_team_id == winner_id {
                final_m.away_team_id
            } else {
                final_m.home_team_id
            };
            (
                winner_id,
                loser_id,
                Some(final_m.id),
                Some(format!("{}:{}", final_m.home_score, final_m.away_score)),
            )
        } else {
            return Err("Could not determine champion".to_string());
        };

        // 获取队伍名称
        let champion_team = TeamRepository::get_by_id(pool, champion_team_id)
            .await
            .map_err(|e| format!("Failed to get champion team: {}", e))?;

        let runner_up_team = TeamRepository::get_by_id(pool, runner_up_team_id)
            .await
            .map_err(|e| format!("Failed to get runner-up team: {}", e))?;

        // 推断季军殿军（仅国际赛事）
        let (third_team_id, third_team_name, fourth_team_id, fourth_team_name) =
            if tournament.tournament_type.is_international() {
                // 查找半决赛失败者或季军赛
                let third_place_match = completed_matches.iter().find(|m| {
                    m.stage.to_lowercase().contains("third")
                        || m.stage.to_lowercase().contains("季军")
                        || m.stage.to_lowercase().contains("3rd")
                });

                if let Some(third_m) = third_place_match {
                    let third_id = third_m.winner_id;
                    let fourth_id = if third_m.home_team_id == third_id.unwrap_or(0) {
                        Some(third_m.away_team_id)
                    } else {
                        Some(third_m.home_team_id)
                    };

                    let third_team = if let Some(id) = third_id {
                        TeamRepository::get_by_id(pool, id).await.ok()
                    } else {
                        None
                    };

                    let fourth_team = if let Some(id) = fourth_id {
                        TeamRepository::get_by_id(pool, id).await.ok()
                    } else {
                        None
                    };

                    (
                        third_id,
                        third_team.map(|t| t.name),
                        fourth_id,
                        fourth_team.map(|t| t.name),
                    )
                } else {
                    // 从半决赛推断
                    let semi_losers: Vec<u64> = completed_matches
                        .iter()
                        .filter(|m| {
                            m.stage.to_lowercase().contains("semi")
                                || m.stage.to_lowercase().contains("半决赛")
                        })
                        .filter_map(|m| {
                            m.winner_id.map(|winner| {
                                if m.home_team_id == winner {
                                    m.away_team_id
                                } else {
                                    m.home_team_id
                                }
                            })
                        })
                        .collect();

                    if semi_losers.len() >= 2 {
                        let third_team = TeamRepository::get_by_id(pool, semi_losers[0]).await.ok();
                        let fourth_team = TeamRepository::get_by_id(pool, semi_losers[1]).await.ok();

                        (
                            Some(semi_losers[0]),
                            third_team.map(|t| t.name),
                            Some(semi_losers[1]),
                            fourth_team.map(|t| t.name),
                        )
                    } else {
                        (None, None, None, None)
                    }
                }
            } else {
                (None, None, None, None)
            };

        let result = TournamentResult::new(
            save_id.to_string(),
            tournament.season_id,
            tournament_id,
            tournament_type,
            tournament.name.clone(),
            champion_team_id,
            champion_team.name,
            runner_up_team_id,
            runner_up_team.name,
        )
        .with_final_info(final_match_id.unwrap_or(0), final_score.unwrap_or_default())
        .with_stats(total_matches, total_games);

        // 添加季军殿军
        let result = if let (Some(third_id), Some(third_name)) = (third_team_id, third_team_name) {
            result.with_third(third_id, third_name)
        } else {
            result
        };

        let result = if let (Some(fourth_id), Some(fourth_name)) = (fourth_team_id, fourth_team_name)
        {
            result.with_fourth(fourth_id, fourth_name)
        } else {
            result
        };

        Ok(result)
    }

    /// 处理常规赛MVP（常规赛第一名+常规赛MVP）
    pub async fn process_regular_season_honors(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Vec<Honor>, String> {
        let tournament = TournamentRepository::get_by_id(pool, tournament_id)
            .await
            .map_err(|e| format!("Failed to get tournament: {}", e))?;

        let tournament_type = format!("{:?}", tournament.tournament_type);
        let mut honors = Vec::new();

        // 获取积分榜第一名
        let standings = StandingRepository::get_by_tournament(pool, tournament_id)
            .await
            .map_err(|e| format!("Failed to get standings: {}", e))?;

        if let Some(first_place) = standings.first() {
            let team = TeamRepository::get_by_id(pool, first_place.team_id)
                .await
                .map_err(|e| format!("Failed to get team: {}", e))?;

            // 常规赛第一名荣誉
            let honor = Honor::new_team_honor(
                save_id,
                HonorType::RegularSeasonFirst,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                team.id,
                &team.name,
            );
            honors.push(honor);
        }

        // 常规赛MVP
        let mvp_candidates = PlayerTournamentStatsRepository::get_mvp_candidates(pool, save_id, tournament_id, 1)
            .await
            .map_err(|e| format!("Failed to get MVP candidates: {}", e))?;

        if let Some(mvp) = mvp_candidates.first() {
            let honor = self.honor_engine.create_regular_season_mvp(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                mvp,
            );
            honors.push(honor);
        }

        // 保存荣誉
        HonorRepository::create_batch(pool, save_id, &honors)
            .await
            .map_err(|e| format!("Failed to save honors: {}", e))?;

        Ok(honors)
    }
}
