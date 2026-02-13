use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::TransferEngine;
use crate::models::transfer::*;
use sqlx::Row;
use tauri::State;

#[tauri::command]
pub async fn start_transfer_window(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferWindowResponse>, String> {
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

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let existing = sqlx::query(
        "SELECT id, status, current_round FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = existing {
        let window_id: i64 = row.get("id");
        let mut status: String = row.get("status");
        let current_round: i64 = row.get("current_round");

        if status == "PENDING" {
            sqlx::query("UPDATE transfer_windows SET status = 'IN_PROGRESS' WHERE id = ?")
                .bind(window_id)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            status = "IN_PROGRESS".to_string();
        }

        return Ok(CommandResult::ok(TransferWindowResponse {
            window_id,
            season_id: current_season,
            status,
            current_round,
        }));
    }

    let engine = TransferEngine::new();
    let response = engine.start_transfer_window(&pool, &save_id, current_season).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(response))
}

#[tauri::command]
pub async fn execute_transfer_round(
    state: State<'_, AppState>,
    window_id: i64,
    round: i64,
) -> Result<CommandResult<RoundExecutionResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    if round < 1 || round > 7 {
        return Ok(CommandResult::err("轮次必须在1-7之间"));
    }

    let engine = TransferEngine::new();
    let result = engine.execute_round(&pool, window_id, round).await
        .map_err(|e| e.to_string())?;

    let event_count = result.events.len() as i64;
    let next_round = if round < 7 { Some(round + 1) } else { None };

    Ok(CommandResult::ok(RoundExecutionResponse {
        round: result.round,
        round_name: result.round_name,
        events: result.events,
        event_count,
        next_round,
        summary: result.summary,
    }))
}

#[tauri::command]
pub async fn fast_forward_transfer(
    state: State<'_, AppState>,
    window_id: i64,
    from_round: Option<i64>,
) -> Result<CommandResult<FastForwardResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let start_round = from_round.unwrap_or(1);
    if start_round < 1 || start_round > 7 {
        return Ok(CommandResult::err("起始轮次必须在1-7之间"));
    }

    let engine = TransferEngine::new();
    let response = engine.fast_forward(&pool, window_id, start_round).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(response))
}

#[tauri::command]
pub async fn get_transfer_window_status(
    state: State<'_, AppState>,
    window_id: i64,
) -> Result<CommandResult<TransferWindowResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let row = sqlx::query(
        "SELECT id, season_id, status, current_round FROM transfer_windows WHERE id = ?"
    )
    .bind(window_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            Ok(CommandResult::ok(TransferWindowResponse {
                window_id: r.get("id"),
                season_id: r.get("season_id"),
                status: r.get("status"),
                current_round: r.get("current_round"),
            }))
        }
        None => Ok(CommandResult::err("转会期不存在")),
    }
}

#[tauri::command]
pub async fn get_current_transfer_window(
    state: State<'_, AppState>,
) -> Result<CommandResult<Option<TransferWindowResponse>>, String> {
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

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let row = sqlx::query(
        "SELECT id, season_id, status, current_round FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            Ok(CommandResult::ok(Some(TransferWindowResponse {
                window_id: r.get("id"),
                season_id: r.get("season_id"),
                status: r.get("status"),
                current_round: r.get("current_round"),
            })))
        }
        None => Ok(CommandResult::ok(None)),
    }
}

#[tauri::command]
pub async fn get_transfer_window_by_season(
    state: State<'_, AppState>,
    season_id: i64,
) -> Result<CommandResult<Option<TransferWindowResponse>>, String> {
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

    let row = sqlx::query(
        "SELECT id, season_id, status, current_round FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            Ok(CommandResult::ok(Some(TransferWindowResponse {
                window_id: r.get("id"),
                season_id: r.get("season_id"),
                status: r.get("status"),
                current_round: r.get("current_round"),
            })))
        }
        None => Ok(CommandResult::ok(None)),
    }
}

#[tauri::command]
pub async fn confirm_close_transfer_window(
    state: State<'_, AppState>,
    window_id: i64,
    force: Option<bool>,
) -> Result<CommandResult<TransferWindowCloseValidation>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let engine = TransferEngine::new();
    let result = engine.validate_and_close_window(&pool, window_id, force.unwrap_or(false)).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(result))
}
