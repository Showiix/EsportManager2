use crate::db::*;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::helpers::*;
use super::GameFlowService;

impl GameFlowService {
    /// 获取完整的游戏时间状态（统一入口）
    pub async fn get_time_state(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<GameTimeState, String> {
        // 获取存档信息
        let save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let current_phase = save.current_phase;
        let current_season = save.current_season;

        // 获取阶段进度
        let phase_progress = self.get_phase_progress(pool, save_id, current_season as u64, current_phase).await?;

        // 判断阶段状态
        let phase_status = if phase_progress.total_matches == 0 && !current_phase.is_non_tournament() {
            // 赛事阶段未初始化
            PhaseStatus::NotInitialized
        } else if current_phase == SeasonPhase::TransferWindow && phase_progress.total_matches == 0 {
            // 转会窗口未创建
            PhaseStatus::NotInitialized
        } else if current_phase == SeasonPhase::Draft && phase_progress.total_matches == 0 && phase_progress.completed_matches == 0 {
            // 选秀阶段：未开始选秀 → InProgress（等待用户在选秀页面操作）
            PhaseStatus::InProgress
        } else if phase_progress.completed_matches >= phase_progress.total_matches && phase_progress.total_matches > 0 {
            // 所有任务完成（必须有实际任务才算完成）
            PhaseStatus::Completed
        } else {
            PhaseStatus::InProgress
        };

        // 获取赛季进度
        let season_progress = self.get_season_progress(current_phase);

        // 获取可用操作
        let available_actions = self.get_available_actions(current_phase, &phase_status);

        // 判断是否可以推进（需要已完成且存在下一阶段）
        let can_advance = phase_status == PhaseStatus::Completed && current_phase.next().is_some();

        // 获取下一阶段
        let next_phase = current_phase.next().map(|p| p.name().to_string());

        Ok(GameTimeState {
            save_id: save_id.to_string(),
            current_season,
            current_phase,
            phase_display_name: current_phase.display_name().to_string(),
            phase_status,
            phase_progress,
            season_progress,
            available_actions,
            can_advance,
            next_phase,
        })
    }

    /// 获取阶段进度
    async fn get_phase_progress(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        phase: SeasonPhase,
    ) -> Result<PhaseProgress, String> {
        let tournament_type = phase.to_tournament_type();

        if let Some(t_type) = tournament_type {
            let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;

            // 预加载赛区名称映射（避免硬编码ID）
            let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
                .bind(save_id)
                .fetch_all(pool)
                .await
                .unwrap_or_default();
            let region_map: std::collections::HashMap<u64, String> = region_rows.iter()
                .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
                .collect();

            let mut tournament_progress_list = Vec::new();
            let mut total_matches = 0u32;
            let mut completed_matches = 0u32;

            for tournament in tournaments {
                // 查询比赛数量（兼容 'COMPLETED' 和 'Completed' 两种状态格式）
                // CANCELLED 的比赛也算作已完成（不需要进行的比赛）
                let counts = sqlx::query(
                    r#"
                    SELECT
                        COUNT(*) as total,
                        SUM(CASE WHEN UPPER(status) IN ('COMPLETED', 'CANCELLED') THEN 1 ELSE 0 END) as completed
                    FROM matches
                    WHERE tournament_id = ?
                    "#
                )
                .bind(tournament.id as i64)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

                let t_total: i64 = counts.get("total");
                let t_completed: i64 = counts.get("completed");

                total_matches += t_total as u32;
                completed_matches += t_completed as u32;

                let status = if t_completed >= t_total && t_total > 0 {
                    "completed"
                } else if t_completed > 0 {
                    "in_progress"
                } else {
                    "upcoming"
                };

                tournament_progress_list.push(TournamentProgress {
                    tournament_id: tournament.id,
                    tournament_name: tournament.name.clone(),
                    region: tournament.region_id.map(|r| region_map.get(&r).cloned().unwrap_or_else(|| get_region_name(r).to_string())),
                    total_matches: t_total as u32,
                    completed_matches: t_completed as u32,
                    status: status.to_string(),
                });
            }

            let percentage = if total_matches > 0 {
                (completed_matches as f32 / total_matches as f32) * 100.0
            } else {
                0.0
            };

            Ok(PhaseProgress {
                tournaments: tournament_progress_list,
                total_matches,
                completed_matches,
                percentage,
            })
        } else {
            // 非赛事阶段
            match phase {
                SeasonPhase::TransferWindow => {
                    // 查询转会窗口状态
                    let window_row = sqlx::query(
                        "SELECT status, current_round FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
                    )
                    .bind(save_id)
                    .bind(season_id as i64)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| format!("查询转会窗口失败: {}", e))?;

                    match window_row {
                        None => {
                            // 无窗口 → NotInitialized (total=0, completed=0 但不会走到 Completed 判断)
                            Ok(PhaseProgress {
                                tournaments: Vec::new(),
                                total_matches: 0,
                                completed_matches: 0,
                                percentage: 0.0,
                            })
                        }
                        Some(row) => {
                            let status: String = row.get("status");
                            let current_round: i64 = row.get("current_round");
                            if status == "COMPLETED" {
                                // 已完成
                                Ok(PhaseProgress {
                                    tournaments: Vec::new(),
                                    total_matches: 1,
                                    completed_matches: 1,
                                    percentage: 100.0,
                                })
                            } else if status == "IN_PROGRESS" && current_round > 0 {
                                // 进行中
                                Ok(PhaseProgress {
                                    tournaments: Vec::new(),
                                    total_matches: 1,
                                    completed_matches: 0,
                                    percentage: 0.0,
                                })
                            } else {
                                // PENDING 或其他
                                Ok(PhaseProgress {
                                    tournaments: Vec::new(),
                                    total_matches: 0,
                                    completed_matches: 0,
                                    percentage: 0.0,
                                })
                            }
                        }
                    }
                }
                SeasonPhase::Draft => {
                    // 选秀每年都有：检查各赛区是否完成选秀
                    let draft_regions: i64 = sqlx::query_scalar(
                        "SELECT COUNT(DISTINCT region_id) FROM draft_results WHERE save_id = ? AND season_id = ?"
                    )
                    .bind(save_id)
                    .bind(season_id as i64)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| format!("查询选秀结果失败: {}", e))?;

                    if draft_regions >= 4 {
                        // 4赛区都完成
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 1,
                            percentage: 100.0,
                        })
                    } else if draft_regions > 0 {
                        // 部分完成
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 0,
                            percentage: 0.0,
                        })
                    } else {
                        // 未开始
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 0,
                            completed_matches: 0,
                            percentage: 0.0,
                        })
                    }
                }
                SeasonPhase::AnnualAwards => {
                    // 检查是否已颁发年度荣誉
                    let awarded: (i64,) = sqlx::query_as(
                        "SELECT COUNT(*) FROM honors WHERE save_id = ? AND season_id = ? AND honor_type LIKE 'ANNUAL%'"
                    )
                    .bind(save_id)
                    .bind(season_id as i64)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

                    if awarded.0 > 0 {
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 1,
                            percentage: 100.0,
                        })
                    } else {
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 0,
                            percentage: 0.0,
                        })
                    }
                }
                _ => {
                    // SeasonEnd → 立即可推进
                    Ok(PhaseProgress {
                        tournaments: Vec::new(),
                        total_matches: 1,
                        completed_matches: 1,
                        percentage: 100.0,
                    })
                }
            }
        }
    }

    /// 获取赛季进度
    fn get_season_progress(&self, current_phase: SeasonPhase) -> SeasonProgress {
        let all_phases = vec![
            SeasonPhase::SpringRegular,
            SeasonPhase::SpringPlayoffs,
            SeasonPhase::Msi,
            SeasonPhase::MadridMasters,
            SeasonPhase::SummerRegular,
            SeasonPhase::SummerPlayoffs,
            SeasonPhase::ClaudeIntercontinental,
            SeasonPhase::WorldChampionship,
            SeasonPhase::ShanghaiMasters,
            SeasonPhase::IcpIntercontinental,
            SeasonPhase::SuperIntercontinental,
            SeasonPhase::AnnualAwards,
            SeasonPhase::TransferWindow,
            SeasonPhase::Draft,
            SeasonPhase::SeasonEnd,
        ];

        let current_index = all_phases.iter().position(|&p| p == current_phase).unwrap_or(0) as u32;
        let total_phases = all_phases.len() as u32;

        let phases: Vec<PhaseInfo> = all_phases.iter().enumerate().map(|(i, &phase)| {
            let status = if (i as u32) < current_index {
                "completed"
            } else if (i as u32) == current_index {
                "current"
            } else {
                "upcoming"
            };

            PhaseInfo {
                phase: format!("{:?}", phase),
                display_name: phase.display_name().to_string(),
                status: status.to_string(),
                index: i as u32,
            }
        }).collect();

        let percentage = (current_index as f32 / total_phases as f32) * 100.0;

        SeasonProgress {
            phases,
            current_phase_index: current_index,
            total_phases,
            percentage,
        }
    }

    /// 获取当前阶段可用的操作
    fn get_available_actions(&self, phase: SeasonPhase, status: &PhaseStatus) -> Vec<TimeAction> {
        let mut actions = Vec::new();

        match phase {
            SeasonPhase::TransferWindow => {
                match status {
                    PhaseStatus::NotInitialized => {
                        actions.push(TimeAction::StartTransferWindow);
                    }
                    PhaseStatus::InProgress => {
                        actions.push(TimeAction::ExecuteTransferRound);
                    }
                    PhaseStatus::Completed => {
                        actions.push(TimeAction::CompleteAndAdvance);
                    }
                }
            }
            SeasonPhase::Draft => {
                match status {
                    PhaseStatus::Completed => {
                        actions.push(TimeAction::CompleteAndAdvance);
                    }
                    _ => {
                        actions.push(TimeAction::StartDraft);
                    }
                }
            }
            SeasonPhase::SeasonEnd => {
                actions.push(TimeAction::StartNewSeason);
            }
            SeasonPhase::AnnualAwards => {
                // 颁奖典礼：只有完成颁奖后才能推进
                if *status == PhaseStatus::Completed {
                    actions.push(TimeAction::CompleteAndAdvance);
                }
            }
            _ => {
                // 赛事阶段
                match status {
                    PhaseStatus::NotInitialized => {
                        actions.push(TimeAction::InitializePhase);
                    }
                    PhaseStatus::InProgress => {
                        actions.push(TimeAction::SimulateNextMatch);
                        actions.push(TimeAction::SimulateAllMatches);
                    }
                    PhaseStatus::Completed => {
                        actions.push(TimeAction::CompleteAndAdvance);
                    }
                }
            }
        }

        // 快进选项（始终可用，除了赛季结束阶段）
        if phase != SeasonPhase::SeasonEnd {
            actions.push(TimeAction::FastForwardPhase);

            // 根据当前阶段添加快进目标
            if phase.is_before(SeasonPhase::SummerRegular) {
                actions.push(TimeAction::FastForwardToSummer);
            }
            if phase.is_before(SeasonPhase::WorldChampionship) {
                actions.push(TimeAction::FastForwardToWorlds);
            }
            actions.push(TimeAction::FastForwardToSeasonEnd);
        }

        actions
    }

    /// 完成当前阶段并推进到下一阶段
    pub async fn complete_and_advance(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<CompleteAndAdvanceResult, String> {
        // 获取当前存档
        let mut save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let current_phase = save.current_phase;
        let season_id = save.current_season as u64;

        // 转会期必须已确认关闭才能推进
        if current_phase == SeasonPhase::TransferWindow {
            let window_row = sqlx::query(
                "SELECT status FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
            )
            .bind(save_id)
            .bind(save.current_season)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("查询转会窗口失败: {}", e))?;

            match window_row {
                Some(row) => {
                    let status: String = row.get("status");
                    if status != "COMPLETED" {
                        return Err("转会窗口尚未关闭，请先确认关闭转会窗口后再推进".to_string());
                    }
                }
                None => {
                    return Err("转会窗口未找到，无法推进".to_string());
                }
            }
        }

        // 完成当前阶段（颁发荣誉）
        let complete_result = self.complete_phase(pool, save_id, season_id, current_phase).await?;

        let honors_awarded: Vec<HonorInfo> = complete_result.honors_awarded.iter().map(|h| {
            HonorInfo {
                honor_type: h.honor_type.clone(),
                recipient_name: h.recipient_name.clone(),
                tournament_name: h.tournament_name.clone(),
            }
        }).collect();

        // 推进到下一阶段
        let new_phase = if let Some(next) = current_phase.next() {
            save.current_phase = next;
            save.phase_completed = false;
            save.updated_at = chrono::Utc::now();

            SaveRepository::update(pool, &save)
                .await
                .map_err(|e| e.to_string())?;

            // 自动初始化下一阶段
            let _init_result = self.initialize_phase(pool, save_id, season_id, next).await?;

            Some(next.name().to_string())
        } else {
            // 赛季结束
            None
        };

        // 获取更新后的时间状态
        let new_time_state = self.get_time_state(pool, save_id).await?;

        Ok(CompleteAndAdvanceResult {
            success: true,
            completed_phase: current_phase.name().to_string(),
            new_phase,
            honors_awarded,
            message: complete_result.message,
            new_time_state,
        })
    }

    /// 快进到目标阶段
    pub async fn fast_forward_to(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        target: FastForwardTarget,
    ) -> Result<FastForwardResult, String> {
        let save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let start_phase = save.current_phase;
        let target_phase = match target {
            FastForwardTarget::NextPhase => start_phase.next(),
            FastForwardTarget::ToPhase(phase) => Some(phase),
            FastForwardTarget::SeasonEnd => Some(SeasonPhase::SeasonEnd),
        };

        if target_phase.is_none() {
            return Ok(FastForwardResult {
                success: false,
                start_phase: start_phase.name().to_string(),
                end_phase: start_phase.name().to_string(),
                phases_advanced: 0,
                matches_simulated: 0,
                message: "已经是赛季最后阶段".to_string(),
            });
        }

        let target_phase = target_phase.unwrap();
        let mut current_phase = start_phase;
        let mut phases_advanced = 0u32;
        let mut total_matches_simulated = 0u32;

        // 循环推进直到到达目标阶段
        while current_phase != target_phase {
            // 获取当前状态
            let time_state = self.get_time_state(pool, save_id).await?;

            // 根据状态执行操作
            match time_state.phase_status {
                PhaseStatus::NotInitialized => {
                    // 初始化阶段
                    self.initialize_phase(pool, save_id, time_state.current_season as u64, current_phase).await?;
                }
                PhaseStatus::InProgress => {
                    // 年度颁奖典礼没有比赛可模拟，直接完成并推进
                    if current_phase == SeasonPhase::AnnualAwards {
                        let _result = self.complete_and_advance(pool, save_id).await?;
                        phases_advanced += 1;
                        let save = SaveRepository::get_by_id(pool, save_id)
                            .await
                            .map_err(|e| e.to_string())?;
                        current_phase = save.current_phase;
                    } else {
                        // 模拟所有比赛
                        let matches_simulated = self.simulate_all_phase_matches(pool, save_id, current_phase).await?;
                        total_matches_simulated += matches_simulated;
                    }
                }
                PhaseStatus::Completed => {
                    // 完成并推进
                    let _result = self.complete_and_advance(pool, save_id).await?;
                    phases_advanced += 1;

                    // 更新当前阶段
                    let save = SaveRepository::get_by_id(pool, save_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    current_phase = save.current_phase;
                }
            }

            // 防止无限循环
            if phases_advanced > 20 {
                break;
            }
        }

        Ok(FastForwardResult {
            success: true,
            start_phase: start_phase.name().to_string(),
            end_phase: current_phase.name().to_string(),
            phases_advanced,
            matches_simulated: total_matches_simulated,
            message: format!("快进完成：从{}到{}", start_phase.name(), current_phase.name()),
        })
    }
}
