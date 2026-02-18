use crate::db::*;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::helpers::*;
use super::{GameFlowService, HonorAwarded, PhaseCompleteResult};

impl GameFlowService {
    /// 检查阶段是否完成
    pub async fn check_phase_completion(
        &self,
        _pool: &Pool<Sqlite>,
        _save_id: &str,
        _season_id: u64,
        phase: SeasonPhase,
    ) -> Result<bool, String> {
        match phase {
            SeasonPhase::AnnualAwards | SeasonPhase::TransferWindow | SeasonPhase::Draft | SeasonPhase::SeasonEnd => {
                // 这些阶段需要手动确认完成
                Ok(false)
            }
            _ => {
                // 检查该阶段所有赛事是否完成
                let tournament_type = phase.to_tournament_type();
                if let Some(_t_type) = tournament_type {
                    // 查询该类型的所有赛事
                    // 检查是否有待进行的比赛
                    // 简化实现：返回false让用户手动推进
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
    }

    /// 完成阶段 - 颁发荣誉并准备下一阶段
    pub async fn complete_phase(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        phase: SeasonPhase,
    ) -> Result<PhaseCompleteResult, String> {
        log::debug!("开始处理阶段: {:?}, season_id={}", phase, season_id);

        // 检查该阶段是否已经完成过（详情页可能已提前处理荣誉和积分）
        // 即使已完成，仍然继续执行：读取已有荣誉、补发奖金和统计等
        let _phase_already_completed = if let Some(t_type) = phase.to_tournament_type() {
            let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
            if !tournaments.is_empty() {
                let all_completed = tournaments.iter().all(|t| t.status == TournamentStatus::Completed);
                if all_completed {
                    log::info!("阶段 {:?} 赛事已标记为完成，将读取已有荣誉并补发奖金/统计", phase);
                }
                all_completed
            } else {
                false
            }
        } else {
            false
        };

        let mut honors_awarded = Vec::new();

        // 根据阶段颁发荣誉
        match phase {
            SeasonPhase::DouyuLadder | SeasonPhase::DouyinLadder | SeasonPhase::HuyaLadder => {
            }

            // 常规赛结束 - 颁发常规赛第一名和常规赛MVP
            SeasonPhase::SpringRegular | SeasonPhase::SummerRegular => {
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    // 获取该阶段的所有赛事
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    log::debug!("找到 {} 个常规赛赛事", tournaments.len());

                    for tournament in tournaments {
                        log::debug!("处理常规赛荣誉: tournament_id={}", tournament.id);
                        match self.honor_service.process_regular_season_honors(pool, save_id, tournament.id).await {
                            Ok(honors) => {
                                log::debug!("常规赛荣誉处理成功，获得 {} 个荣誉", honors.len());
                                for honor in honors {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: format!("{:?}", honor.honor_type),
                                        recipient_name: honor.team_name.clone()
                                            .or(honor.player_name.clone())
                                            .unwrap_or_else(|| "Unknown".to_string()),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to process regular season honors for tournament {}: {}", tournament.id, e);
                            }
                        }
                    }
                }
            }

            // 季后赛/国际赛事结束 - 颁发冠亚季军、MVP等荣誉
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                let tournament_type = phase.to_tournament_type();
                log::debug!("处理季后赛/国际赛事: {:?}, tournament_type={:?}", phase, tournament_type);
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    log::debug!("找到 {} 个赛事", tournaments.len());

                    for tournament in &tournaments {
                        log::debug!("处理赛事荣誉: id={}, name={}", tournament.id, tournament.name);
                        match self.honor_service.process_tournament_completion(pool, save_id, tournament.id).await {
                            Ok(_) => {
                                // 从数据库查询该赛事的所有荣誉（支持ICP等多队伍同名次的赛事）
                                match HonorRepository::get_by_tournament(pool, save_id, tournament.id).await {
                                    Ok(all_honors) => {
                                        log::debug!("从数据库获取到 {} 个荣誉", all_honors.len());
                                        // 只收集战队荣誉和MVP用于显示（选手荣誉太多）
                                        for honor in all_honors {
                                            match honor.honor_type {
                                                // 战队荣誉
                                                HonorType::TeamChampion => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "冠军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::TeamRunnerUp => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "亚军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::TeamThird => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "季军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::TeamFourth => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "殿军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                // MVP荣誉
                                                HonorType::TournamentMvp => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "赛事MVP".to_string(),
                                                        recipient_name: honor.player_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::FinalsMvp => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "决赛MVP".to_string(),
                                                        recipient_name: honor.player_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                // 其他荣誉类型（选手荣誉等）不显示
                                                _ => {}
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::debug!("获取荣誉列表失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::debug!("荣誉处理失败: tournament_id={}, error={}", tournament.id, e);
                            }
                        }
                    }
                }
            }

            // 年度颁奖典礼 - 颁发年度荣誉
            SeasonPhase::AnnualAwards => {
                log::debug!("处理年度颁奖典礼");

                self.recalculate_yearly_scores_with_big_stage(pool, save_id, season_id).await?;

                let top20 = self.get_annual_top20(pool, save_id, season_id).await?;
                log::debug!("获取到 {} 位Top20选手", top20.len());

                for (idx, player) in top20.iter().enumerate() {
                    // 第一名同时获得年度MVP
                    if idx == 0 {
                        let mvp_honor = crate::models::Honor::new_player_honor(
                            save_id,
                            HonorType::AnnualMvp,
                            season_id,
                            None,
                            "年度颁奖典礼",
                            "ANNUAL",
                            player.team_id,
                            &player.team_name,
                            player.player_id,
                            &player.player_name,
                            &player.position,
                            None,
                        );
                        if let Err(e) = HonorRepository::create(pool, save_id, &mvp_honor).await {
                            log::error!("Failed to create annual MVP honor: {}", e);
                        } else {
                            honors_awarded.push(HonorAwarded {
                                honor_type: "年度MVP".to_string(),
                                recipient_name: player.player_name.clone(),
                                tournament_name: "年度颁奖典礼".to_string(),
                            });
                        }
                    }

                    // Top20荣誉
                    let rank = idx + 1;
                    let top20_honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualTop20,
                        season_id,
                        None,
                        &format!("年度Top{}", rank),
                        "ANNUAL",
                        player.team_id,
                        &player.team_name,
                        player.player_id,
                        &player.player_name,
                        &player.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &top20_honor).await {
                        log::error!("Failed to create annual top20 honor for player {}: {}", player.player_id, e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: format!("年度Top{}", idx + 1),
                            recipient_name: player.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取三阵选手并颁发荣誉
                let all_pro_3teams = self.get_annual_all_pro_3teams(pool, save_id, season_id).await?;
                log::debug!("获取到 {} 位最佳阵容选手", all_pro_3teams.len());

                for (tier, player) in &all_pro_3teams {
                    let honor_type = match tier {
                        1 => HonorType::AnnualAllPro1st,
                        2 => HonorType::AnnualAllPro2nd,
                        3 => HonorType::AnnualAllPro3rd,
                        _ => continue,
                    };

                    let tier_name = match tier {
                        1 => "最佳阵容一阵",
                        2 => "最佳阵容二阵",
                        3 => "最佳阵容三阵",
                        _ => continue,
                    };

                    let position_honor = crate::models::Honor::new_player_honor(
                        save_id,
                        honor_type,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        player.team_id,
                        &player.team_name,
                        player.player_id,
                        &player.player_name,
                        &player.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &position_honor).await {
                        log::error!("Failed to create annual all-pro honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: tier_name.to_string(),
                            recipient_name: player.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取最稳定选手
                if let Ok(Some(consistent)) = self.get_annual_most_consistent(pool, save_id, season_id).await {
                    log::debug!("年度最稳定选手: {}", consistent.player_name);

                    let honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualMostConsistent,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        consistent.team_id,
                        &consistent.team_name,
                        consistent.player_id,
                        &consistent.player_name,
                        &consistent.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &honor).await {
                        log::error!("Failed to create annual most consistent honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: "年度最稳定选手".to_string(),
                            recipient_name: consistent.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取最具统治力选手
                if let Ok(Some(dominant)) = self.get_annual_most_dominant(pool, save_id, season_id).await {
                    log::debug!("年度最具统治力选手: {}", dominant.player_name);

                    let honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualMostDominant,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        dominant.team_id,
                        &dominant.team_name,
                        dominant.player_id,
                        &dominant.player_name,
                        &dominant.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &honor).await {
                        log::error!("Failed to create annual most dominant honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: "年度最具统治力".to_string(),
                            recipient_name: dominant.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取年度最佳新秀
                if let Ok(Some(rookie)) = self.get_annual_rookie(pool, save_id, season_id).await {
                    log::debug!("年度最佳新秀: {}", rookie.player_name);

                    let rookie_honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualRookie,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        rookie.team_id,
                        &rookie.team_name,
                        rookie.player_id,
                        &rookie.player_name,
                        &rookie.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &rookie_honor).await {
                        log::error!("Failed to create annual rookie honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: "年度最佳新秀".to_string(),
                            recipient_name: rookie.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }
            }

            _ => {}
        }

        log::debug!("荣誉处理完成，共 {} 个荣誉", honors_awarded.len());

        // 年度颁奖典礼后重算所有选手身价（考虑累积荣誉）
        if phase == SeasonPhase::AnnualAwards {
            log::debug!("开始年度身价重算...");
            match self.recalculate_all_market_values(pool, save_id, season_id).await {
                Ok(count) => log::debug!("年度身价重算完成，共更新 {} 名选手", count),
                Err(e) => log::error!("[complete_phase] 年度身价重算失败: {}", e),
            }

            // 更新所有队伍的品牌价值
            log::debug!("开始更新品牌价值...");
            match self.update_all_brand_values(pool, save_id, season_id).await {
                Ok(count) => log::debug!("品牌价值更新完成，共更新 {} 支队伍", count),
                Err(e) => log::error!("[complete_phase] 品牌价值更新失败: {}", e),
            }
        }

        // 颁发年度积分（季后赛和国际赛事，Super赛除外）
        // Super赛是年度积分的奖励，不颁发积分
        match phase {
            SeasonPhase::DouyuLadder | SeasonPhase::DouyinLadder | SeasonPhase::HuyaLadder => {
            }
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental => {
                log::debug!("颁发年度积分: {:?}", phase);
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    for tournament in &tournaments {
                        log::debug!("为赛事 {} 颁发年度积分", tournament.id);
                        match self.award_tournament_points(pool, save_id, season_id, tournament.id, t_type).await {
                            Ok(awarded) => log::debug!("积分颁发成功: {:?}", awarded),
                            Err(e) => log::debug!("积分颁发失败: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }

        // 发放赛事奖金（季后赛和国际赛事）
        match phase {
            SeasonPhase::DouyuLadder | SeasonPhase::DouyinLadder | SeasonPhase::HuyaLadder => {
            }
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                log::debug!("发放赛事奖金: {:?}", phase);
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    for tournament in &tournaments {
                        log::debug!("为赛事 {} 发放奖金", tournament.id);
                        match self.distribute_tournament_prizes(pool, save_id, season_id, tournament.id, t_type).await {
                            Ok(distributed) => log::debug!("奖金发放成功: {:?}", distributed),
                            Err(e) => log::debug!("奖金发放失败: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }

        // 更新冠军选手的统计数据（增加冠军次数）
        match phase {
            SeasonPhase::DouyuLadder | SeasonPhase::DouyinLadder | SeasonPhase::HuyaLadder => {
            }
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    let is_international = matches!(t_type,
                        TournamentType::Msi |
                        TournamentType::MadridMasters |
                        TournamentType::ClaudeIntercontinental |
                        TournamentType::WorldChampionship |
                        TournamentType::ShanghaiMasters |
                        TournamentType::IcpIntercontinental |
                        TournamentType::SuperIntercontinental
                    );
                    for tournament in tournaments {
                        if let Err(e) = self.update_champion_player_stats(pool, save_id, season_id, tournament.id, t_type, is_international).await {
                            log::error!("Failed to update champion stats for tournament {}: {}", tournament.id, e);
                        }
                    }
                }
            }
            _ => {}
        }

        match phase {
            SeasonPhase::DouyuLadder | SeasonPhase::DouyinLadder | SeasonPhase::HuyaLadder => {
            }
            SeasonPhase::SpringRegular
            | SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerRegular
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                if let Err(e) = self.recalculate_yearly_scores_with_big_stage(pool, save_id, season_id).await {
                    log::error!("赛事结束后重算yearly_top_score失败: {}", e);
                }
            }
            _ => {}
        }

        match phase {
            SeasonPhase::DouyuLadder | SeasonPhase::DouyinLadder | SeasonPhase::HuyaLadder => {
                let event_type = match phase {
                    SeasonPhase::DouyuLadder => "douyu",
                    SeasonPhase::DouyinLadder => "douyin",
                    SeasonPhase::HuyaLadder => "huya",
                    _ => unreachable!(),
                };
                let _ = sqlx::query("UPDATE ladder_tournament SET status = 'Completed' WHERE save_id = ? AND season = ? AND event_type = ?")
                    .bind(save_id)
                    .bind(season_id as i64)
                    .bind(event_type)
                    .execute(pool)
                    .await;
                log::debug!("更新天梯赛 {} 状态为 Completed", event_type);
            }
            _ => {
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    for tournament in &tournaments {
                        let _ = sqlx::query("UPDATE tournaments SET status = 'Completed' WHERE id = ?")
                            .bind(tournament.id as i64)
                            .execute(pool)
                            .await;
                        log::debug!("更新赛事 {} (id={}) 状态为 Completed", tournament.name, tournament.id);
                    }
                }
            }
        }

        // 获取下一阶段
        let next_phase = phase.next();
        let can_advance = next_phase.is_some() || phase == SeasonPhase::SeasonEnd;

        let message = match phase {
            SeasonPhase::SeasonEnd => "赛季结束，准备开始新赛季".to_string(),
            _ => {
                if let Some(next) = next_phase {
                    format!("阶段完成，下一阶段: {}", next.display_name())
                } else {
                    "阶段完成".to_string()
                }
            }
        };

        Ok(PhaseCompleteResult {
            phase: format!("{:?}", phase),
            honors_awarded,
            can_advance,
            next_phase: next_phase.map(|p| format!("{:?}", p)),
            message,
        })
    }

    /// 获取阶段对应的赛事
    pub(crate) async fn get_phase_tournaments(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<Tournament>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM tournaments
            WHERE save_id = ? AND season_id = ? AND tournament_type = ?
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(format!("{:?}", tournament_type))
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get tournaments: {}", e))?;

        Ok(rows.iter().map(|row| {
            Tournament {
                id: row.get::<i64, _>("id") as u64,
                save_id: row.get("save_id"),
                season_id: row.get::<i64, _>("season_id") as u64,
                tournament_type,
                name: row.get("name"),
                region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
                status: parse_tournament_status_local(row.get("status")),
                current_stage: None,
                current_round: None,
            }
        }).collect())
    }

    /// 获取所有赛区 ID
    pub(crate) async fn get_all_region_ids(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<Vec<u64>, String> {
        let rows = sqlx::query_as::<_, (i64,)>(
            "SELECT DISTINCT region_id FROM teams WHERE save_id = ? ORDER BY region_id"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows.iter().map(|r| r.0 as u64).collect())
    }

    pub(crate) async fn count_tournament_matches(&self, pool: &Pool<Sqlite>, tournament_id: u64) -> Result<i64, String> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE tournament_id = ?")
            .bind(tournament_id as i64)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(count.0)
    }

}
