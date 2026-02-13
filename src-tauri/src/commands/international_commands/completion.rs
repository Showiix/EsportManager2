use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{PointsRepository, TeamRepository, TournamentRepository};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

use super::helpers::position_to_rank;

/// 赛事完成结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentCompletionResult {
    pub tournament_id: u64,
    pub tournament_name: String,
    pub honors_awarded: Vec<HonorAwardedInfo>,
    pub points_awarded: Vec<PointsAwardedInfo>,
    pub message: String,
}

/// 颁发的荣誉信息
#[derive(Debug, Serialize, Deserialize)]
pub struct HonorAwardedInfo {
    pub honor_type: String,
    pub recipient_name: String,
    pub recipient_type: String, // "team" or "player"
}

/// 颁发的积分信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PointsAwardedInfo {
    pub team_id: u64,
    pub team_name: String,
    pub points: u32,
    pub position: String,
}

/// 完成赛事 - 处理荣誉殿堂和年度积分
/// 当国际赛事（如MSI、上海大师赛等）完成后调用此命令
#[tauri::command]
pub async fn complete_tournament(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<TournamentCompletionResult>, String> {
    use crate::services::HonorService;
    use crate::engines::PointsCalculationEngine;

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 1. 获取赛事信息
    let tournament = TournamentRepository::get_by_id(&pool, tournament_id)
        .await
        .map_err(|e| format!("Failed to get tournament: {}", e))?;

    let tournament_name = tournament.name.clone();
    let tournament_type = tournament.tournament_type;
    let season_id = tournament.season_id;

    // 2. 处理荣誉殿堂
    let honor_service = HonorService::new();
    let mut honors_awarded = Vec::new();

    match honor_service.process_tournament_completion(&pool, &save_id, tournament_id).await {
        Ok(tournament_honors) => {
            // 收集战队荣誉
            if let Some(ref honor) = tournament_honors.team_champion {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "冠军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.team_runner_up {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "亚军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.team_third {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "季军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.team_fourth {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "殿军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            // 收集选手冠军荣誉
            for honor in &tournament_honors.player_champions {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "冠军成员".to_string(),
                    recipient_name: honor.player_name.clone().unwrap_or_default(),
                    recipient_type: "player".to_string(),
                });
            }
            // 收集MVP荣誉
            if let Some(ref honor) = tournament_honors.tournament_mvp {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "赛事MVP".to_string(),
                    recipient_name: honor.player_name.clone().unwrap_or_default(),
                    recipient_type: "player".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.finals_mvp {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "决赛MVP".to_string(),
                    recipient_name: honor.player_name.clone().unwrap_or_default(),
                    recipient_type: "player".to_string(),
                });
            }
        }
        Err(e) => {
            log::error!("Failed to process tournament honors: {}", e);
        }
    }

    // 3. 处理年度积分（Super赛除外，因为Super赛是年度积分的奖励）
    let mut points_awarded = Vec::new();

    if !matches!(tournament_type, crate::models::TournamentType::SuperIntercontinental) {
        let points_engine = PointsCalculationEngine::new();

        // 获取赛事最终排名
        let results = get_tournament_final_rankings(&pool, tournament_id).await?;

        for (team_id, position) in &results {
            let points = points_engine.get_points(tournament_type, position);
            if points > 0 {
                // 获取队伍名称
                let team = TeamRepository::get_by_id(&pool, *team_id)
                    .await
                    .map_err(|e| format!("Failed to get team: {}", e))?;

                // 保存积分明细（带去重检查）
                let is_new = match PointsRepository::add_points_detail(
                    &pool,
                    &save_id,
                    season_id,
                    *team_id,
                    tournament_id,
                    points,
                    position_to_rank(position),
                ).await {
                    Ok((_, new)) => new,
                    Err(e) => {
                        log::error!("Failed to add points detail: {}", e);
                        continue;
                    }
                };

                // 只有新记录才更新队伍的年度积分
                if is_new {
                    let mut team_to_update = team.clone();
                    team_to_update.annual_points += points;
                    if let Err(e) = TeamRepository::update(&pool, &team_to_update).await {
                        log::error!("Failed to update team annual points: {}", e);
                        continue;
                    }

                    points_awarded.push(PointsAwardedInfo {
                        team_id: *team_id,
                        team_name: team.name.clone(),
                        points,
                        position: position.clone(),
                    });

                    log::debug!("Awarded {} points to {} for position {}",
                        points, team.name, position);
                } else {
                    log::debug!("Skipped duplicate points for {} in tournament {}",
                        team.name, tournament_id);
                }
            }
        }
    }

    // 4. 更新赛事状态为已完成
    let _ = sqlx::query("UPDATE tournaments SET status = 'Completed' WHERE id = ?")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await;

    let message = format!(
        "赛事 {} 完成！颁发了 {} 个荣誉和 {} 条积分记录",
        tournament_name,
        honors_awarded.len(),
        points_awarded.len()
    );

    Ok(CommandResult::ok(TournamentCompletionResult {
        tournament_id,
        tournament_name,
        honors_awarded,
        points_awarded,
        message,
    }))
}

/// 获取赛事最终排名
async fn get_tournament_final_rankings(
    pool: &sqlx::SqlitePool,
    tournament_id: u64,
) -> Result<Vec<(u64, String)>, String> {
    let mut results: Vec<(u64, String)> = Vec::new();

    // 获取赛事类型
    let tournament_type: Option<String> = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let is_double_elimination = matches!(
        tournament_type.as_deref(),
        Some("Msi") | Some("ShanghaiMasters")
    );

    // 获取所有已完成的比赛
    let knockout_matches = sqlx::query(
        r#"
        SELECT * FROM matches
        WHERE tournament_id = ? AND UPPER(status) = 'COMPLETED'
        ORDER BY stage DESC, match_order
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    log::debug!("tournament_id={}, is_double_elimination={}, matches={}",
        tournament_id, is_double_elimination, knockout_matches.len());

    // 找到决赛 - 冠军和亚军
    for m in &knockout_matches {
        let stage: String = m.get("stage");
        if stage == "FINALS" || stage == "GRAND_FINALS" || stage == "GRAND_FINAL" {
            let winner_id = m.get::<Option<i64>, _>("winner_id");
            let home_id = m.get::<i64, _>("home_team_id") as u64;
            let away_id = m.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let runner_up = if winner == home_id { away_id } else { home_id };
                results.push((winner, "CHAMPION".to_string()));
                results.push((runner_up, "RUNNER_UP".to_string()));
                log::debug!("CHAMPION={}, RUNNER_UP={}", winner, runner_up);
            }
            break;
        }
    }

    if is_double_elimination {
        // === 双败制赛事（MSI/上海大师赛）===

        // 季军 = 败者组决赛的败者
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_FINAL" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "THIRD".to_string()));
                        log::debug!("THIRD={}", loser);
                    }
                }
                break;
            }
        }

        // 殿军 = 败者组R4的败者
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R4" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "FOURTH".to_string()));
                        log::debug!("FOURTH={}", loser);
                    }
                }
                break;
            }
        }

        // 败者组R3败者 (5-6名) = LOSERS_R2积分
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R3" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "LOSERS_R2".to_string()));
                        log::debug!("LOSERS_R2={}", loser);
                    }
                }
            }
        }

        // 败者组R2败者 (7-8名) = LOSERS_R1积分
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R2" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "LOSERS_R1".to_string()));
                        log::debug!("LOSERS_R1={}", loser);
                    }
                }
            }
        }

        // 败者组R1败者 (9-10名) - 资格赛被淘汰
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R1" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "QUALIFIER_OUT".to_string()));
                        log::debug!("QUALIFIER_OUT={}", loser);
                    }
                }
            }
        }

        // 资格赛R1败者 (11-12名)
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "QUALIFIER_R1" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "QUALIFIER_R1_OUT".to_string()));
                        log::debug!("QUALIFIER_R1_OUT={}", loser);
                    }
                }
            }
        }

    } else {
        // === 标准淘汰赛赛事 ===

        // 找半决赛败者（季军/殿军）
        let mut semi_losers: Vec<u64> = Vec::new();
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "SEMI_FINALS" || stage == "SEMI_FINAL" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        semi_losers.push(loser);
                    }
                }
            }
        }

        // 分配季军和殿军
        if semi_losers.len() >= 2 {
            results.push((semi_losers[0], "THIRD".to_string()));
            results.push((semi_losers[1], "FOURTH".to_string()));
        } else if semi_losers.len() == 1 {
            results.push((semi_losers[0], "THIRD".to_string()));
        }

        // 八强失败者
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "QUARTER_FINALS" || stage == "QUARTER_FINAL" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "QUARTER_FINAL".to_string()));
                    }
                }
            }
        }
    }

    log::debug!("Final results: {:?}", results);
    Ok(results)
}

/// 清理重复的赛事（按类型）
/// 保留每种类型的第一个赛事，删除多余的
#[tauri::command]
pub async fn cleanup_duplicate_tournaments(
    state: State<'_, AppState>,
    tournament_type: String,
) -> Result<CommandResult<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized".to_string())),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded".to_string())),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取游戏状态
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let season_id: i64 = match save_row {
        Some(row) => row.get("current_season"),
        None => return Ok(CommandResult::err("Save not found".to_string())),
    };

    // 查找该类型的所有赛事
    let tournaments: Vec<(i64, String)> = sqlx::query_as(
        r#"
        SELECT id, status FROM tournaments
        WHERE save_id = ? AND season_id = ? AND tournament_type = ?
        ORDER BY id ASC
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .bind(&tournament_type)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if tournaments.len() <= 1 {
        log::debug!("No duplicates found for type {}", tournament_type);
        return Ok(CommandResult::ok(0));
    }

    log::debug!("Found {} tournaments of type {}, cleaning duplicates...",
        tournaments.len(), tournament_type);

    let mut deleted_count = 0u32;

    // 保留第一个，删除其余
    for (tournament_id, status) in tournaments.iter().skip(1) {
        log::debug!("Deleting tournament id={}, status={}", tournament_id, status);

        // 删除相关比赛
        sqlx::query("DELETE FROM matches WHERE tournament_id = ?")
            .bind(tournament_id)
            .execute(&pool)
            .await
            .ok();

        // 删除积分榜
        sqlx::query("DELETE FROM league_standings WHERE tournament_id = ?")
            .bind(tournament_id)
            .execute(&pool)
            .await
            .ok();

        // 删除赛事本身
        let result = sqlx::query("DELETE FROM tournaments WHERE id = ?")
            .bind(tournament_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected() > 0 {
            deleted_count += 1;
        }
    }

    log::debug!("Deleted {} duplicate tournaments", deleted_count);
    Ok(CommandResult::ok(deleted_count))
}
