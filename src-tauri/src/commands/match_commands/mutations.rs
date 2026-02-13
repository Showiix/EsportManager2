use crate::commands::save_commands::{AppState, CommandResult};
use tauri::State;

/// 更新比赛结果（用于前端本地模拟后同步数据库）
#[tauri::command]
pub async fn update_match_result(
    state: State<'_, AppState>,
    match_id: u64,
    home_score: u32,
    away_score: u32,
    winner_id: u64,
) -> Result<CommandResult<bool>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 更新比赛结果
    sqlx::query(
        r#"
        UPDATE matches SET
            home_score = ?,
            away_score = ?,
            winner_id = ?,
            status = 'COMPLETED',
            played_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(home_score as i64)
    .bind(away_score as i64)
    .bind(winner_id as i64)
    .bind(match_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update match result: {}", e))?;

    log::debug!("Match {} updated: {}:{}, winner={}",
        match_id, home_score, away_score, winner_id);

    Ok(CommandResult::ok(true))
}

/// 更新比赛队伍（用于填充淘汰赛待定队伍）
#[tauri::command]
pub async fn update_match_teams(
    state: State<'_, AppState>,
    match_id: u64,
    home_team_id: u64,
    away_team_id: u64,
) -> Result<CommandResult<bool>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 更新比赛队伍
    sqlx::query(
        r#"
        UPDATE matches SET
            home_team_id = ?,
            away_team_id = ?
        WHERE id = ?
        "#,
    )
    .bind(home_team_id as i64)
    .bind(away_team_id as i64)
    .bind(match_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update match teams: {}", e))?;

    log::debug!("Match {} updated: home={}, away={}",
        match_id, home_team_id, away_team_id);

    Ok(CommandResult::ok(true))
}

/// 取消比赛（标记为 CANCELLED）
#[tauri::command]
pub async fn cancel_match(
    state: State<'_, AppState>,
    match_id: u64,
) -> Result<CommandResult<bool>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 更新比赛状态为 CANCELLED
    sqlx::query(
        r#"
        UPDATE matches SET
            status = 'CANCELLED'
        WHERE id = ?
        "#,
    )
    .bind(match_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to cancel match: {}", e))?;

    log::debug!("Match {} cancelled", match_id);

    Ok(CommandResult::ok(true))
}
