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
use sqlx::{Pool, Sqlite, Row};

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

    /// 获取某队在赛事中的选手信息
    /// 优先从赛事统计表获取（避免赛后转会导致记录错误），回退到当前阵容
    async fn get_team_players_info(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        team_id: u64,
    ) -> Vec<(u64, String, String)> {
        let stats = PlayerTournamentStatsRepository::get_by_team_tournament(
            pool, save_id, tournament_id, team_id
        ).await.unwrap_or_default();

        if !stats.is_empty() {
            return stats.iter()
                .map(|s| (s.player_id, s.player_name.clone(), s.position.clone()))
                .collect();
        }

        // 回退到当前队伍成员
        let current_players = PlayerRepository::get_by_team(pool, team_id)
            .await.unwrap_or_default();
        current_players.iter()
            .filter(|p| p.is_starter)
            .map(|p| {
                let pos_str = p.position.as_ref()
                    .map(|pos| format!("{:?}", pos))
                    .unwrap_or_else(|| "Unknown".to_string());
                (p.id, p.game_id.clone(), pos_str)
            })
            .collect()
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
        let _is_international = tournament.tournament_type.is_international();
        let has_third_place = tournament.tournament_type.has_third_fourth(); // 国际赛事和季后赛有季军殿军

        // 检查是否是需要从冠军队伍选择MVP的赛事类型
        // MVP应该颁发给冠军队伍中表现最好的选手
        let use_champion_mvp = matches!(
            tournament.tournament_type,
            crate::models::TournamentType::SpringPlayoffs
            | crate::models::TournamentType::SummerPlayoffs
            | crate::models::TournamentType::Msi
            | crate::models::TournamentType::MadridMasters
            | crate::models::TournamentType::WorldChampionship
            | crate::models::TournamentType::ClaudeIntercontinental
            | crate::models::TournamentType::ShanghaiMasters
            | crate::models::TournamentType::SuperIntercontinental
            | crate::models::TournamentType::IcpIntercontinental
        );

        // ICP洲际对抗赛需要特殊处理 - 按赛区颁发荣誉
        if tournament.tournament_type == crate::models::TournamentType::IcpIntercontinental {
            return self.process_icp_tournament_completion(pool, save_id, tournament_id, &tournament).await;
        }

        // 2. 检查该赛事的荣誉是否已存在（防止重复颁发）
        let existing_honors = HonorRepository::get_by_tournament(pool, save_id, tournament_id)
            .await
            .map_err(|e| format!("Failed to check existing honors: {}", e))?;

        if !existing_honors.is_empty() {
            log::debug!("荣誉已存在，跳过颁发: tournament_id={}, honor_count={}",
                tournament_id, existing_honors.len());
            // 从已存在的荣誉中构建 TournamentHonors 返回
            return Ok(Self::build_tournament_honors_from_existing(&existing_honors));
        }

        // 3. 检查赛事结果是否已存在
        let existing_result = TournamentResultRepository::get_by_tournament(pool, save_id, tournament_id)
            .await
            .map_err(|e| format!("Failed to check existing result: {}", e))?;

        // 4. 如果不存在，推断并保存赛事结果
        let result = if let Some(existing) = existing_result {
            log::debug!("赛事结果已存在，跳过创建: tournament_id={}", tournament_id);
            existing
        } else {
            let inferred_result = self
                .infer_tournament_result(pool, save_id, &tournament)
                .await?;

            TournamentResultRepository::create(pool, &inferred_result)
                .await
                .map_err(|e| format!("Failed to save tournament result: {}", e))?;

            log::debug!("赛事结果已创建: tournament_id={}", tournament_id);
            inferred_result
        };

        // 4. 获取冠军队选手 - 从赛事统计表获取实际参与过该赛事的选手
        // 这样即使选手在赛后转会，也能正确记录冠军成员
        let champion_players_info = Self::get_team_players_info(
            pool, save_id, tournament_id, result.champion_team_id
        ).await;

        log::debug!("冠军队选手数量: {}", champion_players_info.len());

        // 5. 获取赛事MVP（从 player_tournament_stats）
        // 对于MSI和马德里大师赛，从冠军队伍中选择MVP
        let mvp_candidates = if use_champion_mvp {
            log::debug!("国际赛事MVP规则：从冠军队伍中选择MVP (champion_team_id={})", result.champion_team_id);
            PlayerTournamentStatsRepository::get_mvp_candidates_by_team(
                pool, save_id, tournament_id, result.champion_team_id, 1
            )
            .await
            .map_err(|e| format!("Failed to get MVP candidates from champion team: {}", e))?
        } else {
            PlayerTournamentStatsRepository::get_mvp_candidates(pool, save_id, tournament_id, 1)
                .await
                .map_err(|e| format!("Failed to get MVP candidates: {}", e))?
        };

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

        // 7.5 冠军队选手荣誉 - 使用从赛事统计获取的选手信息
        for (player_id, player_name, position) in &champion_players_info {
            tournament_honors.player_champions.push(self.honor_engine.create_player_champion(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                result.champion_team_id,
                &result.champion_team_name,
                *player_id,
                player_name,
                position,
            ));
        }

        log::debug!("创建了 {} 个选手冠军荣誉", tournament_honors.player_champions.len());

        // 7.6 亚军队选手荣誉
        let runner_up_players_info = Self::get_team_players_info(
            pool, save_id, tournament_id, result.runner_up_team_id
        ).await;

        for (player_id, player_name, position) in &runner_up_players_info {
            tournament_honors.player_runner_ups.push(self.honor_engine.create_player_runner_up(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                result.runner_up_team_id,
                &result.runner_up_team_name,
                *player_id,
                player_name,
                position,
            ));
        }

        log::debug!("创建了 {} 个选手亚军荣誉", tournament_honors.player_runner_ups.len());

        // 7.7 季军队选手荣誉（如果有季军）
        if let (Some(third_id), Some(third_name)) = (result.third_team_id, &result.third_team_name) {
            let third_players_info = Self::get_team_players_info(
                pool, save_id, tournament_id, third_id
            ).await;

            for (player_id, player_name, position) in &third_players_info {
                tournament_honors.player_thirds.push(self.honor_engine.create_player_third(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    third_id,
                    third_name,
                    *player_id,
                    player_name,
                    position,
                ));
            }

            log::debug!("创建了 {} 个选手季军荣誉", tournament_honors.player_thirds.len());
        }

        // 7.8 殿军队选手荣誉（如果有殿军）
        if let (Some(fourth_id), Some(fourth_name)) = (result.fourth_team_id, &result.fourth_team_name) {
            let fourth_players_info = Self::get_team_players_info(
                pool, save_id, tournament_id, fourth_id
            ).await;

            for (player_id, player_name, position) in &fourth_players_info {
                tournament_honors.player_fourths.push(self.honor_engine.create_player_fourth(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    fourth_id,
                    fourth_name,
                    *player_id,
                    player_name,
                    position,
                ));
            }

            log::debug!("创建了 {} 个选手殿军荣誉", tournament_honors.player_fourths.len());
        }

        // 7.9 赛事MVP
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

        // 7.10 决赛MVP（暂时跳过，需要 player_match_stats 数据）
        // 可以在后续版本中实现

        // 8. 保存荣誉到数据库
        let all_honors = tournament_honors.to_vec();

        log::debug!("总共保存 {} 个荣誉记录", all_honors.len());

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

        // 查找决赛 - 优先匹配 GRAND_FINAL，避免错误匹配 LOSERS_FINAL
        let final_match = completed_matches
            .iter()
            .find(|m| {
                let stage_upper = m.stage.to_uppercase();
                stage_upper.contains("GRAND_FINAL") || stage_upper.contains("总决赛")
            })
            .or_else(|| {
                // 如果没有 GRAND_FINAL，尝试查找其他决赛（排除 LOSERS_FINAL 和 WINNERS_FINAL）
                completed_matches.iter().find(|m| {
                    let stage_lower = m.stage.to_lowercase();
                    let stage_upper = m.stage.to_uppercase();
                    (stage_lower.contains("final") || stage_lower.contains("决赛"))
                        && !stage_upper.contains("LOSERS_FINAL")
                        && !stage_upper.contains("WINNERS_FINAL")
                })
            })
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

        // 推断季军殿军（国际赛事和季后赛都有）
        let (third_team_id, third_team_name, fourth_team_id, fourth_team_name) =
            if tournament.tournament_type.has_third_fourth() {
                // 方法1: 查找季军赛
                log::debug!("查找季军赛，已完成比赛数量: {}", completed_matches.len());
                for m in completed_matches.iter() {
                    log::debug!("比赛: stage={}, winner_id={:?}", m.stage, m.winner_id);
                }

                let third_place_match = completed_matches.iter().find(|m| {
                    let stage_lower = m.stage.to_lowercase();
                    let stage_upper = m.stage.to_uppercase();
                    stage_lower.contains("third")
                        || stage_lower.contains("季军")
                        || stage_lower.contains("3rd")
                        || stage_upper.contains("THIRD_PLACE")
                });

                log::debug!("找到季军赛: {:?}", third_place_match.map(|m| &m.stage));

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
                    // 方法2: 从双败淘汰赛推断 (LOSERS_FINAL 败者为第三, LOSERS_R4/LOSERS_R3 败者为第四)
                    let losers_final = completed_matches.iter().find(|m| {
                        m.stage.to_uppercase().contains("LOSERS_FINAL")
                            || m.stage.contains("败者决赛")
                    });

                    let losers_r3 = completed_matches.iter().find(|m| {
                        m.stage.to_uppercase().contains("LOSERS_R4")
                            || m.stage.to_uppercase().contains("LOSERS_R3")
                    });

                    log::debug!("双败淘汰赛推断季军殿军:");
                    log::debug!("  - LOSERS_FINAL 比赛: {:?}", losers_final.map(|m| format!("stage={}, winner_id={:?}", m.stage, m.winner_id)));
                    log::debug!("  - LOSERS_R3/R4 比赛: {:?}", losers_r3.map(|m| format!("stage={}, winner_id={:?}", m.stage, m.winner_id)));

                    if let Some(lf) = losers_final {
                        // 败者决赛的败者是第三名
                        let third_id = lf.winner_id.and_then(|winner| {
                            Some(if lf.home_team_id == winner {
                                lf.away_team_id
                            } else {
                                lf.home_team_id
                            })
                        });

                        // 第四名来自 LOSERS_R4/R3 的败者
                        let fourth_id = losers_r3.and_then(|lr| {
                            lr.winner_id.map(|winner| {
                                if lr.home_team_id == winner {
                                    lr.away_team_id
                                } else {
                                    lr.home_team_id
                                }
                            })
                        });

                        log::debug!("  - 推断季军 team_id: {:?}", third_id);
                        log::debug!("  - 推断殿军 team_id: {:?}", fourth_id);

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
                        // 方法3: 从半决赛推断
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
        // 先检查该赛事的荣誉是否已存在（防止重复颁发）
        let existing_honors = HonorRepository::get_by_tournament(pool, save_id, tournament_id)
            .await
            .map_err(|e| format!("Failed to check existing honors: {}", e))?;

        if !existing_honors.is_empty() {
            log::debug!("荣誉已存在，跳过颁发: tournament_id={}, honor_count={}",
                tournament_id, existing_honors.len());
            return Ok(existing_honors);
        }

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

    /// 从已存在的荣誉记录构建 TournamentHonors 结构
    fn build_tournament_honors_from_existing(existing_honors: &[Honor]) -> TournamentHonors {
        let mut tournament_honors = TournamentHonors::default();

        for honor in existing_honors {
            match honor.honor_type {
                HonorType::TeamChampion => {
                    tournament_honors.team_champion = Some(honor.clone());
                }
                HonorType::TeamRunnerUp => {
                    tournament_honors.team_runner_up = Some(honor.clone());
                }
                HonorType::TeamThird => {
                    tournament_honors.team_third = Some(honor.clone());
                }
                HonorType::TeamFourth => {
                    tournament_honors.team_fourth = Some(honor.clone());
                }
                HonorType::PlayerChampion => {
                    tournament_honors.player_champions.push(honor.clone());
                }
                HonorType::PlayerRunnerUp => {
                    tournament_honors.player_runner_ups.push(honor.clone());
                }
                HonorType::PlayerThird => {
                    tournament_honors.player_thirds.push(honor.clone());
                }
                HonorType::PlayerFourth => {
                    tournament_honors.player_fourths.push(honor.clone());
                }
                HonorType::TournamentMvp => {
                    tournament_honors.tournament_mvp = Some(honor.clone());
                }
                HonorType::FinalsMvp => {
                    tournament_honors.finals_mvp = Some(honor.clone());
                }
                _ => {}
            }
        }

        tournament_honors
    }

    /// 处理ICP洲际对抗赛完成 - 按赛区颁发荣誉
    /// ICP的特殊之处：冠军/亚军是赛区，该赛区的所有参赛队伍都获得对应荣誉
    async fn process_icp_tournament_completion(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        tournament: &crate::models::Tournament,
    ) -> Result<TournamentHonors, String> {
        let tournament_type = format!("{:?}", tournament.tournament_type);

        // 检查荣誉是否已存在
        let existing_honors = HonorRepository::get_by_tournament(pool, save_id, tournament_id)
            .await
            .map_err(|e| format!("Failed to check existing honors: {}", e))?;

        if !existing_honors.is_empty() {
            log::debug!("荣誉已存在，跳过颁发: tournament_id={}, honor_count={}",
                tournament_id, existing_honors.len());
            return Ok(Self::build_tournament_honors_from_existing(&existing_honors));
        }

        // 1. 获取所有参赛队伍及其赛区
        let team_rows = sqlx::query(
            r#"
            SELECT DISTINCT t.id as team_id, t.name as team_name, t.region_id, r.name as region_name, r.short_name as region_code
            FROM matches m
            JOIN teams t ON t.id = m.home_team_id OR t.id = m.away_team_id
            JOIN regions r ON t.region_id = r.id
            WHERE m.tournament_id = ?
            ORDER BY t.region_id, t.id
            "#
        )
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get ICP teams: {}", e))?;

        // 按赛区分组
        let mut region_teams: std::collections::HashMap<u64, Vec<(u64, String, String)>> = std::collections::HashMap::new();
        let mut region_names: std::collections::HashMap<u64, String> = std::collections::HashMap::new();

        for row in &team_rows {
            let team_id: i64 = row.get("team_id");
            let team_name: String = row.get("team_name");
            let region_id: i64 = row.get("region_id");
            let region_name: String = row.get("region_name");

            region_names.insert(region_id as u64, region_name);
            region_teams.entry(region_id as u64)
                .or_default()
                .push((team_id as u64, team_name, "".to_string()));
        }

        log::debug!("找到 {} 个赛区，共 {} 支队伍",
            region_teams.len(), team_rows.len());

        // 2. 从ICP_FINAL比赛确定赛区排名
        // ICP决赛是4场BO5，赢3场的赛区获胜
        let final_matches = sqlx::query(
            r#"
            SELECT m.stage, m.home_team_id, m.away_team_id, m.winner_id, m.status,
                   ht.region_id as home_region_id, at.region_id as away_region_id
            FROM matches m
            LEFT JOIN teams ht ON m.home_team_id = ht.id
            LEFT JOIN teams at ON m.away_team_id = at.id
            WHERE m.tournament_id = ? AND (m.stage LIKE 'ICP_FINAL%' OR m.stage LIKE 'ICP_SEMI%')
            AND UPPER(m.status) = 'COMPLETED'
            ORDER BY m.stage
            "#
        )
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get ICP final matches: {}", e))?;

        // 统计决赛和半决赛的胜负
        let mut final_wins: std::collections::HashMap<u64, i32> = std::collections::HashMap::new();
        let mut semi_wins: std::collections::HashMap<u64, i32> = std::collections::HashMap::new();
        let mut final_regions: (Option<u64>, Option<u64>) = (None, None);
        let mut semi_regions: (Option<u64>, Option<u64>) = (None, None);

        for row in &final_matches {
            let stage: String = row.get("stage");
            let winner_id: Option<i64> = row.get("winner_id");
            let home_region_id: Option<i64> = row.get("home_region_id");
            let away_region_id: Option<i64> = row.get("away_region_id");

            if let (Some(winner), Some(home_region), Some(away_region)) = (winner_id, home_region_id, away_region_id) {
                let home_team_id: Option<i64> = row.get("home_team_id");
                let winner_region = if home_team_id == Some(winner) {
                    home_region as u64
                } else {
                    away_region as u64
                };

                if stage.starts_with("ICP_FINAL") {
                    *final_wins.entry(winner_region).or_insert(0) += 1;
                    final_regions = (Some(home_region as u64), Some(away_region as u64));
                } else if stage.starts_with("ICP_SEMI") {
                    *semi_wins.entry(winner_region).or_insert(0) += 1;
                    semi_regions = (Some(home_region as u64), Some(away_region as u64));
                }
            }
        }

        // 确定赛区排名
        // 决赛胜者为冠军，败者为亚军
        let (champion_region_id, runner_up_region_id) = {
            let mut sorted: Vec<_> = final_wins.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));

            if sorted.len() >= 2 {
                (*sorted[0].0, *sorted[1].0)
            } else if sorted.len() == 1 {
                // 只有一个赛区有胜场，取决赛双方
                if let (Some(r1), Some(r2)) = final_regions {
                    if *sorted[0].0 == r1 { (r1, r2) } else { (r2, r1) }
                } else {
                    return Err("Cannot determine ICP champion/runner-up".to_string());
                }
            } else {
                return Err("No ICP final results found".to_string());
            }
        };

        // 半决赛的赛区为第3/4名（如果有半决赛）
        let (third_region_id, fourth_region_id) = {
            if !semi_wins.is_empty() {
                let mut sorted: Vec<_> = semi_wins.iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(a.1));

                if sorted.len() >= 2 {
                    (Some(*sorted[0].0), Some(*sorted[1].0))
                } else if sorted.len() == 1 {
                    if let (Some(r1), Some(r2)) = semi_regions {
                        if *sorted[0].0 == r1 { (Some(r1), Some(r2)) } else { (Some(r2), Some(r1)) }
                    } else {
                        (Some(*sorted[0].0), None)
                    }
                } else {
                    (None, None)
                }
            } else {
                // 没有半决赛，第3/4名是未进入决赛的赛区
                let other_regions: Vec<_> = region_teams.keys()
                    .filter(|&&r| r != champion_region_id && r != runner_up_region_id)
                    .copied()
                    .collect();

                if other_regions.len() >= 2 {
                    (Some(other_regions[0]), Some(other_regions[1]))
                } else if other_regions.len() == 1 {
                    (Some(other_regions[0]), None)
                } else {
                    (None, None)
                }
            }
        };

        log::debug!("赛区排名: 冠军={}, 亚军={}, 季军={:?}, 殿军={:?}",
            champion_region_id, runner_up_region_id, third_region_id, fourth_region_id);

        // 3. 获取各赛区名称并准备结果
        let champion_region_name = region_names.get(&champion_region_id).cloned().unwrap_or_default();
        let runner_up_region_name = region_names.get(&runner_up_region_id).cloned().unwrap_or_default();

        // 4. 验证数据
        let champion_teams = region_teams.get(&champion_region_id).cloned().unwrap_or_default();
        let runner_up_teams = region_teams.get(&runner_up_region_id).cloned().unwrap_or_default();

        if champion_teams.is_empty() || runner_up_teams.is_empty() {
            return Err("Cannot find champion/runner-up teams for ICP".to_string());
        }

        // 5. 创建荣誉
        let mut tournament_honors = TournamentHonors::default();
        let mut all_team_honors: Vec<Honor> = Vec::new(); // 存储所有队伍的荣誉

        // 5.1 战队荣誉 - 为每个赛区的所有队伍创建
        // 冠军赛区的所有队伍
        for (team_id, team_name, _) in &champion_teams {
            let honor = self.honor_engine.create_team_champion(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                *team_id,
                team_name,
            );
            if tournament_honors.team_champion.is_none() {
                tournament_honors.team_champion = Some(honor.clone());
            }
            all_team_honors.push(honor);
        }

        // 亚军赛区的所有队伍
        for (team_id, team_name, _) in &runner_up_teams {
            let honor = self.honor_engine.create_team_runner_up(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                *team_id,
                team_name,
            );
            if tournament_honors.team_runner_up.is_none() {
                tournament_honors.team_runner_up = Some(honor.clone());
            }
            all_team_honors.push(honor);
        }

        // 季军赛区（如果有）
        if let Some(third_region) = third_region_id {
            let third_teams = region_teams.get(&third_region).cloned().unwrap_or_default();
            for (team_id, team_name, _) in &third_teams {
                let honor = self.honor_engine.create_team_third(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    *team_id,
                    team_name,
                );
                if tournament_honors.team_third.is_none() {
                    tournament_honors.team_third = Some(honor.clone());
                }
                all_team_honors.push(honor);
            }
        }

        // 殿军赛区（如果有）
        if let Some(fourth_region) = fourth_region_id {
            let fourth_teams = region_teams.get(&fourth_region).cloned().unwrap_or_default();
            for (team_id, team_name, _) in &fourth_teams {
                let honor = self.honor_engine.create_team_fourth(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    *team_id,
                    team_name,
                );
                if tournament_honors.team_fourth.is_none() {
                    tournament_honors.team_fourth = Some(honor.clone());
                }
                all_team_honors.push(honor);
            }
        }

        // 5.2 选手荣誉 - 为每个赛区的参赛选手创建
        // 冠军赛区选手
        for (team_id, _, _) in &champion_teams {
            let player_stats = PlayerTournamentStatsRepository::get_by_team_tournament(
                pool, save_id, tournament_id, *team_id
            ).await.unwrap_or_default();

            for stats in &player_stats {
                let honor = self.honor_engine.create_player_champion(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    *team_id,
                    &champion_region_name,
                    stats.player_id,
                    &stats.player_name,
                    &stats.position,
                );
                tournament_honors.player_champions.push(honor);
            }
        }

        // 亚军赛区选手
        for (team_id, _, _) in &runner_up_teams {
            let player_stats = PlayerTournamentStatsRepository::get_by_team_tournament(
                pool, save_id, tournament_id, *team_id
            ).await.unwrap_or_default();

            for stats in &player_stats {
                let honor = self.honor_engine.create_player_runner_up(
                    save_id,
                    tournament.season_id,
                    tournament_id,
                    &tournament.name,
                    &tournament_type,
                    *team_id,
                    &runner_up_region_name,
                    stats.player_id,
                    &stats.player_name,
                    &stats.position,
                );
                tournament_honors.player_runner_ups.push(honor);
            }
        }

        // 5.3 赛事MVP - 从冠军赛区选择
        let mvp_candidates = PlayerTournamentStatsRepository::get_mvp_candidates(pool, save_id, tournament_id, 1)
            .await
            .unwrap_or_default();

        if let Some(mvp_stats) = mvp_candidates.first() {
            tournament_honors.tournament_mvp = Some(self.honor_engine.create_tournament_mvp(
                save_id,
                tournament.season_id,
                tournament_id,
                &tournament.name,
                &tournament_type,
                mvp_stats,
            ));
        }

        // 6. 收集所有荣誉并保存
        let mut all_honors = Vec::new();

        // 战队荣誉（所有参赛队伍）
        all_honors.extend(all_team_honors);

        // 选手荣誉
        all_honors.extend(tournament_honors.player_champions.clone());
        all_honors.extend(tournament_honors.player_runner_ups.clone());

        // MVP
        if let Some(ref honor) = tournament_honors.tournament_mvp {
            all_honors.push(honor.clone());
        }

        log::debug!("总共保存 {} 个荣誉记录", all_honors.len());

        HonorRepository::create_batch(pool, save_id, &all_honors)
            .await
            .map_err(|e| format!("Failed to save ICP honors: {}", e))?;

        Ok(tournament_honors)
    }
}
