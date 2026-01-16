//! 开发工具命令 - 用于调试和数据修复
//!
//! 这些命令仅供开发使用，生产环境应禁用

use serde::{Deserialize, Serialize};
use sqlx::{Row, Pool, Sqlite};
use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::{SaveRepository, TournamentRepository, MatchRepository};
use crate::services::HonorService;
use crate::engines::PointsCalculationEngine;
use crate::models::{TournamentStatus, PlayerTag, Position};

/// 开发命令结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DevCommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub error: Option<String>,
}

impl<T> DevCommandResult<T> {
    pub fn ok(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: String::new(),
            error: Some(msg.into()),
        }
    }
}

impl DevCommandResult<()> {
    pub fn ok_msg(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(()),
            message: message.into(),
            error: None,
        }
    }
}

/// 数据一致性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    pub total_checks: i32,
    pub passed: i32,
    pub failed: i32,
    pub issues: Vec<ConsistencyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub category: String,
    pub description: String,
    pub severity: String, // "warning" | "error"
}

/// 同步选手场次统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub updated_count: i32,
    pub details: Vec<String>,
}

// ==================== 荣誉系统 ====================

/// 重新颁发当前赛季所有荣誉
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_reassign_honors(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取当前赛季
    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    // 清除该赛季现有荣誉
    sqlx::query("DELETE FROM honors WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season)
        .execute(&pool)
        .await
        .ok();

    // 获取该赛季所有已完成的赛事
    let tournaments = TournamentRepository::get_by_season(&pool, &save_id, season as u64)
        .await
        .map_err(|e| e.to_string())?;

    let completed_tournaments: Vec<_> = tournaments
        .into_iter()
        .filter(|t| t.status == TournamentStatus::Completed)
        .collect();

    let honor_service = HonorService::new();
    let mut honors_count = 0;

    for tournament in &completed_tournaments {
        match honor_service.process_tournament_completion(&pool, &save_id, tournament.id).await {
            Ok(honors) => {
                let count = honors.player_champions.len() + honors.player_runner_ups.len() +
                           if honors.tournament_mvp.is_some() { 1 } else { 0 } +
                           if honors.team_champion.is_some() { 1 } else { 0 } +
                           if honors.team_runner_up.is_some() { 1 } else { 0 };
                honors_count += count;
            },
            Err(e) => println!("[dev_reassign_honors] 赛事 {} 颁发荣誉失败: {}", tournament.name, e),
        }
    }

    Ok(DevCommandResult::ok(
        honors_count as i32,
        format!("成功重新颁发 {} 项荣誉（{}个已完成赛事）", honors_count, completed_tournaments.len()),
    ))
}

/// 重新计算年度积分
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_recalculate_annual_points(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    // 清除该赛季的年度积分明细
    sqlx::query("DELETE FROM annual_points_detail WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season)
        .execute(&pool)
        .await
        .ok();

    // 重置所有队伍的年度积分
    sqlx::query("UPDATE teams SET annual_points = 0 WHERE save_id = ?")
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();

    // 获取该赛季所有已完成的赛事
    let tournaments = TournamentRepository::get_by_season(&pool, &save_id, season as u64)
        .await
        .map_err(|e| e.to_string())?;

    let points_engine = PointsCalculationEngine::default();
    let mut updated_count = 0;

    for tournament in tournaments.iter().filter(|t| t.status == TournamentStatus::Completed) {
        // 获取赛事结果
        let results = sqlx::query(
            "SELECT team_id, position FROM tournament_results WHERE save_id = ? AND tournament_id = ?"
        )
        .bind(&save_id)
        .bind(tournament.id as i64)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        for row in results {
            let team_id: i64 = row.get("team_id");
            let position: i64 = row.get("position");

            // 将数字名次转换为字符串
            let position_str = match position {
                1 => "CHAMPION",
                2 => "RUNNER_UP",
                3 => "THIRD",
                4 => "FOURTH",
                5..=8 => "5TH_8TH",
                _ => continue, // 跳过其他名次
            };

            let points = points_engine.get_points(tournament.tournament_type, position_str);

            if points > 0 {
                // 更新队伍年度积分
                sqlx::query("UPDATE teams SET annual_points = annual_points + ? WHERE id = ?")
                    .bind(points as i64)
                    .bind(team_id)
                    .execute(&pool)
                    .await
                    .ok();

                // 记录积分明细
                sqlx::query(
                    "INSERT INTO annual_points_detail (save_id, season_id, team_id, tournament_id, points, position)
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&save_id)
                .bind(season)
                .bind(team_id)
                .bind(tournament.id as i64)
                .bind(points as i64)
                .bind(position)
                .execute(&pool)
                .await
                .ok();

                updated_count += 1;
            }
        }
    }

    Ok(DevCommandResult::ok(
        updated_count,
        format!("成功重新计算年度积分，更新了 {} 条记录", updated_count),
    ))
}

// ==================== 数据修复 ====================

/// 同步选手场次统计（从 game_player_performances 同步到 player_season_stats）
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_sync_player_games_played(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<SyncResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    // 统计每个选手的真实场次
    let real_counts: Vec<(i64, i64)> = sqlx::query_as(
        r#"
        SELECT gpp.player_id, COUNT(*) as real_count
        FROM game_player_performances gpp
        JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
        JOIN matches m ON mg.match_id = m.id
        JOIN tournaments t ON m.tournament_id = t.id
        WHERE gpp.save_id = ? AND t.season_id = ?
        GROUP BY gpp.player_id
        "#
    )
    .bind(&save_id)
    .bind(season)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut updated_count = 0;
    let mut details = Vec::new();

    for (player_id, real_count) in real_counts {
        // 获取当前记录的场次
        let current: Option<(i64,)> = sqlx::query_as(
            "SELECT games_played FROM player_season_stats WHERE save_id = ? AND player_id = ? AND season_id = ?"
        )
        .bind(&save_id)
        .bind(player_id)
        .bind(season)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

        let current_count = current.map(|c| c.0).unwrap_or(0);

        if current_count != real_count {
            // 更新场次
            sqlx::query(
                "UPDATE player_season_stats SET games_played = ? WHERE save_id = ? AND player_id = ? AND season_id = ?"
            )
            .bind(real_count)
            .bind(&save_id)
            .bind(player_id)
            .bind(season)
            .execute(&pool)
            .await
            .ok();

            details.push(format!("选手#{}: {} -> {}", player_id, current_count, real_count));
            updated_count += 1;
        }
    }

    Ok(DevCommandResult::ok(
        SyncResult { updated_count, details },
        format!("同步完成，更新了 {} 名选手的场次统计", updated_count),
    ))
}

/// 重新计算积分榜
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_recalculate_standings(
    state: State<'_, AppState>,
    tournament_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let tournaments = if let Some(tid) = tournament_id {
        vec![TournamentRepository::get_by_id(&pool, tid as u64)
            .await
            .map_err(|e| e.to_string())?]
    } else {
        // 获取当前赛季所有赛事
        let save = SaveRepository::get_by_id(&pool, &save_id)
            .await
            .map_err(|e| e.to_string())?;
        TournamentRepository::get_by_season(&pool, &save_id, save.current_season as u64)
            .await
            .map_err(|e| e.to_string())?
    };

    let mut updated_count = 0;

    for tournament in &tournaments {
        // 只处理常规赛类型的赛事
        if !format!("{:?}", tournament.tournament_type).contains("Regular") {
            continue;
        }

        // 获取该赛事的所有已完成比赛
        let matches = MatchRepository::get_by_tournament(&pool, tournament.id)
            .await
            .map_err(|e| e.to_string())?;

        // 重置积分榜
        sqlx::query(
            "UPDATE league_standings SET matches_played = 0, wins = 0, losses = 0, points = 0, games_won = 0, games_lost = 0, game_diff = 0 WHERE tournament_id = ?"
        )
        .bind(tournament.id as i64)
        .execute(&pool)
        .await
        .ok();

        // 重新统计
        for m in matches.iter().filter(|m| m.status == crate::models::MatchStatus::Completed) {
            let home_score = m.home_score;
            let away_score = m.away_score;
            let home_won = home_score > away_score;

            // 更新主队
            sqlx::query(
                r#"
                UPDATE league_standings SET
                    matches_played = matches_played + 1,
                    wins = wins + ?,
                    losses = losses + ?,
                    points = points + ?,
                    games_won = games_won + ?,
                    games_lost = games_lost + ?,
                    game_diff = game_diff + ?
                WHERE tournament_id = ? AND team_id = ?
                "#
            )
            .bind(if home_won { 1 } else { 0 })
            .bind(if home_won { 0 } else { 1 })
            .bind(if home_won { 3 } else { 0 })
            .bind(home_score as i64)
            .bind(away_score as i64)
            .bind((home_score as i64) - (away_score as i64))
            .bind(tournament.id as i64)
            .bind(m.home_team_id as i64)
            .execute(&pool)
            .await
            .ok();

            // 更新客队
            sqlx::query(
                r#"
                UPDATE league_standings SET
                    matches_played = matches_played + 1,
                    wins = wins + ?,
                    losses = losses + ?,
                    points = points + ?,
                    games_won = games_won + ?,
                    games_lost = games_lost + ?,
                    game_diff = game_diff + ?
                WHERE tournament_id = ? AND team_id = ?
                "#
            )
            .bind(if home_won { 0 } else { 1 })
            .bind(if home_won { 1 } else { 0 })
            .bind(if home_won { 0 } else { 3 })
            .bind(away_score as i64)
            .bind(home_score as i64)
            .bind((away_score as i64) - (home_score as i64))
            .bind(tournament.id as i64)
            .bind(m.away_team_id as i64)
            .execute(&pool)
            .await
            .ok();
        }

        updated_count += 1;
    }

    Ok(DevCommandResult::ok(
        updated_count,
        format!("成功重新计算 {} 个赛事的积分榜", updated_count),
    ))
}

/// 数据一致性检查
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_check_data_consistency(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<ConsistencyCheckResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let mut issues = Vec::new();
    let mut total_checks = 0;
    let mut passed = 0;

    // 检查1: 选手场次一致性
    total_checks += 1;
    let inconsistent_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM (
            SELECT pss.player_id, pss.games_played as recorded,
                   COALESCE(gpp.real_count, 0) as actual
            FROM player_season_stats pss
            LEFT JOIN (
                SELECT gpp.player_id, COUNT(*) as real_count
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments t ON m.tournament_id = t.id
                WHERE gpp.save_id = ?
                GROUP BY gpp.player_id
            ) gpp ON pss.player_id = gpp.player_id
            WHERE pss.save_id = ? AND pss.games_played != COALESCE(gpp.real_count, 0)
        )
        "#
    )
    .bind(&save_id)
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    if inconsistent_count.0 > 0 {
        issues.push(ConsistencyIssue {
            category: "选手统计".to_string(),
            description: format!("{} 名选手的场次记录与实际不符", inconsistent_count.0),
            severity: "warning".to_string(),
        });
    } else {
        passed += 1;
    }

    // 检查2: 比赛状态一致性（已完成的比赛应有比分）
    total_checks += 1;
    let invalid_matches: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM matches WHERE save_id = ? AND status = 'Completed' AND (home_score = 0 AND away_score = 0)"
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    if invalid_matches.0 > 0 {
        issues.push(ConsistencyIssue {
            category: "比赛数据".to_string(),
            description: format!("{} 场已完成比赛没有比分记录", invalid_matches.0),
            severity: "error".to_string(),
        });
    } else {
        passed += 1;
    }

    // 检查3: 队伍选手数量（每队应有5名首发）
    total_checks += 1;
    let incomplete_teams: Vec<(i64, String, i64)> = sqlx::query_as(
        r#"
        SELECT t.id, t.name, COUNT(p.id) as starter_count
        FROM teams t
        LEFT JOIN players p ON t.id = p.team_id AND p.is_starter = 1 AND p.status = 'Active'
        WHERE t.save_id = ?
        GROUP BY t.id
        HAVING starter_count < 5
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    if !incomplete_teams.is_empty() {
        for (_, name, count) in &incomplete_teams {
            issues.push(ConsistencyIssue {
                category: "队伍阵容".to_string(),
                description: format!("{} 只有 {} 名首发选手", name, count),
                severity: "warning".to_string(),
            });
        }
    } else {
        passed += 1;
    }

    // 检查4: 赛事比赛数量
    total_checks += 1;
    let empty_tournaments: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM tournaments t
        WHERE t.save_id = ? AND t.status != 'Upcoming'
        AND NOT EXISTS (SELECT 1 FROM matches m WHERE m.tournament_id = t.id)
        "#
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    if empty_tournaments.0 > 0 {
        issues.push(ConsistencyIssue {
            category: "赛事数据".to_string(),
            description: format!("{} 个非待开始赛事没有比赛", empty_tournaments.0),
            severity: "warning".to_string(),
        });
    } else {
        passed += 1;
    }

    let result = ConsistencyCheckResult {
        total_checks,
        passed,
        failed: total_checks - passed,
        issues,
    };

    let message = if result.failed == 0 {
        "数据一致性检查通过".to_string()
    } else {
        format!("发现 {} 个问题", result.issues.len())
    };

    Ok(DevCommandResult::ok(result, message))
}

// ==================== 赛事管理 ====================

/// 重置阶段状态
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_reset_phase(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取当前存档信息
    let save = SaveRepository::get_by_id(&pool, &save_id)
        .await
        .map_err(|e| e.to_string())?;

    // 重置当前阶段的 phase_completed 状态
    sqlx::query("UPDATE saves SET phase_completed = 0 WHERE id = ?")
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();

    Ok(DevCommandResult::ok_msg(format!(
        "已重置阶段状态: {:?}",
        save.current_phase
    )))
}

/// 模拟所有待进行比赛
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_simulate_all_matches(
    state: State<'_, AppState>,
    tournament_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取待进行的比赛
    let pending_matches: Vec<(i64,)> = if let Some(tid) = tournament_id {
        sqlx::query_as(
            "SELECT id FROM matches WHERE save_id = ? AND tournament_id = ? AND status = 'Scheduled' ORDER BY id"
        )
        .bind(&save_id)
        .bind(tid)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    } else {
        // 获取当前赛季所有待进行比赛
        let save = SaveRepository::get_by_id(&pool, &save_id)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query_as(
            r#"
            SELECT m.id FROM matches m
            JOIN tournaments t ON m.tournament_id = t.id
            WHERE m.save_id = ? AND t.season_id = ? AND m.status = 'Scheduled'
            ORDER BY m.id
            "#
        )
        .bind(&save_id)
        .bind(save.current_season as i64)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

    // 返回待模拟的比赛数量（实际模拟需要通过前端循环调用 simulate_match_detailed）
    Ok(DevCommandResult::ok(
        pending_matches.len() as i32,
        format!("找到 {} 场待进行比赛，请使用前端批量模拟功能", pending_matches.len()),
    ))
}

// ==================== 财务系统 ====================

/// 重新发放赛事奖金（清除旧记录，返回需要重新发放的赛事数量）
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_redistribute_prizes(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    // 删除该赛季的奖金交易记录
    let deleted: u64 = sqlx::query(
        "DELETE FROM financial_transactions WHERE save_id = ? AND season_id = ? AND (transaction_type = 'PlayoffBonus' OR transaction_type = 'InternationalBonus')"
    )
    .bind(&save_id)
    .bind(season)
    .execute(&pool)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);

    // 获取该赛季所有已完成的赛事数量
    let completed_tournaments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tournaments WHERE save_id = ? AND season_id = ? AND status = 'Completed'"
    )
    .bind(&save_id)
    .bind(season)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    Ok(DevCommandResult::ok(
        completed_tournaments.0 as i32,
        format!("清除 {} 条旧记录，请通过前端重新完成赛事以发放奖金（共 {} 个已完成赛事）", deleted, completed_tournaments.0),
    ))
}

/// 给所有战队发放测试资金
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_grant_funds(
    state: State<'_, AppState>,
    amount: i64,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 更新所有队伍余额
    let result = sqlx::query("UPDATE teams SET balance = balance + ? WHERE save_id = ?")
        .bind(amount)
        .bind(&save_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DevCommandResult::ok(
        result.rows_affected() as i32,
        format!("成功给 {} 支战队各发放 {} 资金", result.rows_affected(), amount),
    ))
}

// ==================== 快速测试 ====================

/// 重置存档到初始状态
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_reset_save(
    state: State<'_, AppState>,
    keep_teams: bool,
) -> Result<DevCommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 清除比赛相关数据
    let tables_to_clear = vec![
        "game_player_performances",
        "match_games",
        "matches",
        "tournaments",
        "league_standings",
        "tournament_results",
        "honors",
        "annual_points_detail",
        "player_season_stats",
        "player_tournament_stats",
        "financial_transactions",
    ];

    for table in tables_to_clear {
        sqlx::query(&format!("DELETE FROM {} WHERE save_id = ?", table))
            .bind(&save_id)
            .execute(&pool)
            .await
            .ok();
    }

    if !keep_teams {
        // 重置队伍数据
        sqlx::query(
            "UPDATE teams SET total_matches = 0, wins = 0, win_rate = 0, annual_points = 0, cross_year_points = 0 WHERE save_id = ?"
        )
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();
    }

    // 重置存档状态
    sqlx::query(
        "UPDATE saves SET current_season = 1, current_phase = 'SpringRegular', phase_completed = 0 WHERE id = ?"
    )
    .bind(&save_id)
    .execute(&pool)
    .await
    .ok();

    Ok(DevCommandResult::ok_msg("存档已重置到S1春季赛开始"))
}

/// 获取当前游戏状态摘要
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_get_game_status(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<GameStatusSummary>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let save = SaveRepository::get_by_id(&pool, &save_id)
        .await
        .map_err(|e| e.to_string())?;

    // 统计数据
    let team_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    let player_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM players WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    let tournament_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tournaments WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    let match_stats: (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN status = 'Completed' THEN 1 ELSE 0 END) as completed,
            SUM(CASE WHEN status = 'Scheduled' THEN 1 ELSE 0 END) as scheduled
        FROM matches WHERE save_id = ?
        "#
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0, 0, 0));

    let honor_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM honors WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    Ok(DevCommandResult::ok(
        GameStatusSummary {
            current_season: save.current_season,
            current_phase: format!("{:?}", save.current_phase),
            phase_completed: save.phase_completed,
            team_count: team_count.0 as i32,
            player_count: player_count.0 as i32,
            tournament_count: tournament_count.0 as i32,
            total_matches: match_stats.0 as i32,
            completed_matches: match_stats.1 as i32,
            scheduled_matches: match_stats.2 as i32,
            honor_count: honor_count.0 as i32,
        },
        "获取游戏状态成功",
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatusSummary {
    pub current_season: u32,
    pub current_phase: String,
    pub phase_completed: bool,
    pub team_count: i32,
    pub player_count: i32,
    pub tournament_count: i32,
    pub total_matches: i32,
    pub completed_matches: i32,
    pub scheduled_matches: i32,
    pub honor_count: i32,
}

/// 检查未完成的比赛
#[tauri::command]
pub async fn dev_check_incomplete_matches(
    state: State<'_, AppState>,
    tournament_type: Option<String>,
) -> Result<DevCommandResult<Vec<IncompleteMatchInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 查询未完成的比赛
    let query = if let Some(ref t_type) = tournament_type {
        sqlx::query(
            r#"
            SELECT m.id, m.stage, m.status, m.home_team_id, m.away_team_id,
                   t.name as tournament_name, t.tournament_type,
                   ht.name as home_name, at.name as away_name
            FROM matches m
            JOIN tournaments t ON m.tournament_id = t.id
            LEFT JOIN teams ht ON m.home_team_id = ht.id
            LEFT JOIN teams at ON m.away_team_id = at.id
            WHERE t.save_id = ? AND UPPER(m.status) NOT IN ('COMPLETED', 'CANCELLED')
              AND t.tournament_type = ?
            ORDER BY t.name, m.stage, m.id
            "#
        )
        .bind(&save_id)
        .bind(t_type)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT m.id, m.stage, m.status, m.home_team_id, m.away_team_id,
                   t.name as tournament_name, t.tournament_type,
                   ht.name as home_name, at.name as away_name
            FROM matches m
            JOIN tournaments t ON m.tournament_id = t.id
            LEFT JOIN teams ht ON m.home_team_id = ht.id
            LEFT JOIN teams at ON m.away_team_id = at.id
            WHERE t.save_id = ? AND UPPER(m.status) NOT IN ('COMPLETED', 'CANCELLED')
            ORDER BY t.name, m.stage, m.id
            "#
        )
        .bind(&save_id)
        .fetch_all(&pool)
        .await
    };

    let rows = query.map_err(|e| e.to_string())?;

    let mut matches: Vec<IncompleteMatchInfo> = Vec::new();
    for row in rows {
        matches.push(IncompleteMatchInfo {
            match_id: row.get::<i64, _>("id") as u64,
            stage: row.get("stage"),
            status: row.get("status"),
            tournament_name: row.get("tournament_name"),
            tournament_type: row.get("tournament_type"),
            home_team: row.get::<Option<String>, _>("home_name"),
            away_team: row.get::<Option<String>, _>("away_name"),
        });
    }

    let count = matches.len();
    Ok(DevCommandResult::ok(
        matches,
        format!("找到 {} 场未完成的比赛", count),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteMatchInfo {
    pub match_id: u64,
    pub stage: String,
    pub status: String,
    pub tournament_name: String,
    pub tournament_type: String,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
}

/// 强制完成比赛（将状态设为 CANCELLED）
#[tauri::command]
pub async fn dev_force_complete_match(
    state: State<'_, AppState>,
    match_id: u64,
) -> Result<DevCommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    sqlx::query("UPDATE matches SET status = 'CANCELLED' WHERE id = ?")
        .bind(match_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DevCommandResult::ok_msg(format!("比赛 {} 已标记为 CANCELLED", match_id)))
}

/// 迁移选手忠诚度和满意度（根据选手属性重新计算）
#[tauri::command]
pub async fn dev_migrate_loyalty_satisfaction(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<u32>, String> {
    use crate::services::init_service::InitService;

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    match InitService::migrate_loyalty_satisfaction(&pool, &save_id).await {
        Ok(count) => Ok(DevCommandResult::ok(
            count,
            format!("成功迁移 {} 名选手的忠诚度和满意度", count)
        )),
        Err(e) => Ok(DevCommandResult::err(format!("迁移失败: {}", e))),
    }
}

/// 重新计算所有选手的身价（使用新公式，含荣誉和赛区系数）
#[tauri::command]
pub async fn dev_recalculate_market_values(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<u32>, String> {
    use crate::models::{PlayerTag, Position};

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    // 获取所有选手（包含赛区信息）
    let rows = sqlx::query(
        r#"SELECT p.id, p.game_id, p.ability, p.potential, p.age, p.tag, p.position,
                  p.market_value, p.calculated_market_value,
                  r.short_name as region_code
           FROM players p
           LEFT JOIN teams t ON p.team_id = t.id
           LEFT JOIN regions r ON t.region_id = r.id
           WHERE p.save_id = ? AND p.status = 'Active'"#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch players: {}", e))?;

    let mut updated_count = 0u32;

    for row in rows {
        let id: i64 = row.get("id");
        let game_id: String = row.get("game_id");
        let ability: i64 = row.get("ability");
        let potential: i64 = row.get("potential");
        let age: i64 = row.get("age");
        let tag_str: String = row.get("tag");
        let position_str: String = row.get("position");
        let old_calculated: i64 = row.try_get("calculated_market_value").unwrap_or(0);
        let region_code: String = row.try_get::<Option<String>, _>("region_code")
            .ok()
            .flatten()
            .unwrap_or_else(|| "LPL".to_string());

        // 解析 tag
        let tag = match tag_str.to_uppercase().as_str() {
            "GENIUS" => PlayerTag::Genius,
            "NORMAL" => PlayerTag::Normal,
            _ => PlayerTag::Ordinary,
        };

        // 解析 position
        let position = match position_str.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };

        // 计算基础身价
        let base_value = calculate_market_value(ability as u8, potential as u8, age as u8, tag, position);

        // 计算荣誉系数
        let honor_factor = calculate_honor_factor(&pool, &save_id, id as u64).await;

        // 赛区系数
        let region_factor = get_region_market_factor(&region_code);

        // 最终身价
        let new_value = ((base_value as f64) * honor_factor * region_factor) as i64;

        if new_value != old_calculated {
            // 更新计算后的身价（不覆盖基础身价）
            sqlx::query("UPDATE players SET calculated_market_value = ? WHERE id = ?")
                .bind(new_value)
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| format!("Failed to update player {}: {}", game_id, e))?;

            println!("[recalculate_market_values] {} 身价 {} -> {} 万 (荣誉×{:.2}, 赛区×{:.2})",
                game_id, old_calculated, new_value, honor_factor, region_factor);

            updated_count += 1;
        }
    }

    Ok(DevCommandResult::ok(
        updated_count,
        format!("成功更新 {} 名选手的计算身价", updated_count)
    ))
}

/// 计算荣誉系数
async fn calculate_honor_factor(pool: &Pool<Sqlite>, save_id: &str, player_id: u64) -> f64 {
    let rows = sqlx::query(
        r#"SELECT honor_type, tournament_type, tournament_name
           FROM honors WHERE save_id = ? AND player_id = ?"#
    )
    .bind(save_id)
    .bind(player_id as i64)
    .fetch_all(pool)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(_) => return 1.0,
    };

    let mut honor_bonus = 0.0;

    for row in rows {
        let honor_type: String = row.get("honor_type");
        let tournament_type: String = row.try_get::<Option<String>, _>("tournament_type")
            .ok().flatten().unwrap_or_default();
        let tournament_name: String = row.try_get::<Option<String>, _>("tournament_name")
            .ok().flatten().unwrap_or_default();

        let bonus = match honor_type.as_str() {
            "PLAYER_CHAMPION" | "TEAM_CHAMPION" => {
                if tournament_type.contains("World") || tournament_name.contains("世界赛") {
                    0.40
                } else if tournament_name.contains("MSI") {
                    0.35
                } else if tournament_name.contains("大师赛") || tournament_name.contains("Masters") {
                    0.30
                } else if tournament_name.contains("洲际") {
                    0.25
                } else {
                    0.20
                }
            }
            "PLAYER_RUNNER_UP" | "TEAM_RUNNER_UP" => {
                if tournament_type.contains("World") || tournament_name.contains("世界赛") {
                    0.20
                } else {
                    0.10
                }
            }
            "PLAYER_THIRD_PLACE" | "TEAM_THIRD_PLACE" => 0.06,
            "PLAYER_FOURTH_PLACE" | "TEAM_FOURTH_PLACE" => 0.04,
            "TOURNAMENT_MVP" | "FINALS_MVP" => {
                if tournament_name.contains("世界赛") || tournament_name.contains("World") {
                    0.25
                } else {
                    0.15
                }
            }
            "ANNUAL_MVP" => 0.35,
            "ANNUAL_BEST_PLAYER" => 0.20,
            "SEASON_MVP" => 0.15,
            _ => 0.0,
        };

        honor_bonus += bonus;
    }

    let honor_factor: f64 = (1.0_f64 + honor_bonus).min(4.0);
    honor_factor
}

/// 获取赛区身价系数
fn get_region_market_factor(region_code: &str) -> f64 {
    match region_code.to_uppercase().as_str() {
        "LPL" => 1.3,
        "LCK" => 1.2,
        "LEC" => 1.0,
        "LCS" => 0.9,
        _ => 0.8,
    }
}

/// 计算选手身价（与 init_service 保持一致）
fn calculate_market_value(ability: u8, potential: u8, age: u8, tag: PlayerTag, position: Position) -> u64 {
    let multiplier = match ability {
        95..=100 => 50,
        90..=94 => 35,
        85..=89 => 20,
        80..=84 => 12,
        75..=79 => 7,
        70..=74 => 4,
        60..=69 => 2,
        _ => 1,
    };

    let base = ability as u64 * multiplier;

    let age_factor = match age {
        17..=19 => 1.5,
        20..=22 => 1.3,
        23..=25 => 1.0,
        26..=27 => 0.85,
        28..=29 => 0.7,
        _ => 0.5,
    };

    let diff = potential.saturating_sub(ability);
    let potential_factor = if diff > 10 {
        1.25
    } else if diff >= 5 {
        1.1
    } else {
        1.0
    };

    let tag_factor = tag.market_value_factor();
    let position_factor = position.market_value_factor();

    // 返回万元
    ((base as f64) * age_factor * potential_factor * tag_factor * position_factor) as u64
}

/// 自动修复队伍首发阵容
/// 确保每个队伍每个位置都有一名首发（选择该位置能力最高的选手）
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_fix_starters(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<FixStartersResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let positions = vec!["Top", "Jug", "Mid", "Adc", "Sup"];
    let mut fixed_teams = Vec::new();
    let mut total_fixed = 0;

    // 获取所有队伍
    let teams: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, name FROM teams WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    for (team_id, team_name) in teams {
        let mut team_fixes = Vec::new();

        for pos in &positions {
            // 检查该位置是否有首发
            let has_starter: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM players
                WHERE team_id = ? AND save_id = ? AND status = 'Active'
                AND UPPER(position) = UPPER(?) AND is_starter = 1
                "#
            )
            .bind(team_id)
            .bind(&save_id)
            .bind(pos)
            .fetch_one(&pool)
            .await
            .unwrap_or((0,));

            if has_starter.0 == 0 {
                // 没有首发，找该位置能力最高的选手
                let best_player: Option<(i64, String, i64)> = sqlx::query_as(
                    r#"
                    SELECT id, game_id, ability FROM players
                    WHERE team_id = ? AND save_id = ? AND status = 'Active'
                    AND UPPER(position) = UPPER(?)
                    ORDER BY ability DESC
                    LIMIT 1
                    "#
                )
                .bind(team_id)
                .bind(&save_id)
                .bind(pos)
                .fetch_optional(&pool)
                .await
                .unwrap_or(None);

                if let Some((player_id, game_id, ability)) = best_player {
                    // 设为首发
                    sqlx::query("UPDATE players SET is_starter = 1 WHERE id = ?")
                        .bind(player_id)
                        .execute(&pool)
                        .await
                        .ok();

                    team_fixes.push(format!("{}: {} (能力{})", pos, game_id, ability));
                    total_fixed += 1;
                }
            }
        }

        if !team_fixes.is_empty() {
            fixed_teams.push(TeamFixInfo {
                team_name,
                fixes: team_fixes,
            });
        }
    }

    let teams_count = fixed_teams.len();
    Ok(DevCommandResult::ok(
        FixStartersResult {
            teams_fixed: teams_count as i32,
            players_fixed: total_fixed,
            details: fixed_teams,
        },
        format!("修复了 {} 支队伍的 {} 个首发位置", teams_count, total_fixed),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStartersResult {
    pub teams_fixed: i32,
    pub players_fixed: i32,
    pub details: Vec<TeamFixInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamFixInfo {
    pub team_name: String,
    pub fixes: Vec<String>,
}
