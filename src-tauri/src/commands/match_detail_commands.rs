use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::MatchGameDetailRepository;
use crate::models::match_game_detail::{
    MatchFullDetails, SaveMatchDetailsInput,
};
use tauri::State;

/// 保存比赛详情
#[tauri::command]
pub async fn save_match_details(
    state: State<'_, AppState>,
    save_id: String,
    input: SaveMatchDetailsInput,
) -> Result<CommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    match MatchGameDetailRepository::save_match_details(&pool, &save_id, &input).await {
        Ok(_) => Ok(CommandResult::ok(())),
        Err(e) => Ok(CommandResult::err(format!("Failed to save match details: {}", e))),
    }
}

/// 获取比赛详情
#[tauri::command]
pub async fn get_match_details(
    state: State<'_, AppState>,
    save_id: String,
    match_id: i64,
) -> Result<CommandResult<Option<MatchFullDetails>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    match MatchGameDetailRepository::get_match_full_details(&pool, &save_id, match_id).await {
        Ok(details) => Ok(CommandResult::ok(details)),
        Err(e) => Ok(CommandResult::err(format!("Failed to get match details: {}", e))),
    }
}

/// 删除比赛详情
#[tauri::command]
pub async fn delete_match_details(
    state: State<'_, AppState>,
    save_id: String,
    match_id: i64,
) -> Result<CommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    match MatchGameDetailRepository::delete_match_details(&pool, &save_id, match_id).await {
        Ok(_) => Ok(CommandResult::ok(())),
        Err(e) => Ok(CommandResult::err(format!("Failed to delete match details: {}", e))),
    }
}
