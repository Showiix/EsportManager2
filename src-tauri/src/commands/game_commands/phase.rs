use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::SaveRepository;
use crate::engines::SeasonProgressEngine;
use crate::models::SeasonPhase;
use crate::services::{GameFlowService, PhaseCompleteResult, PhaseInitResult};
use chrono::Utc;
use tauri::State;
use super::{GameStateInfo, get_phase_display_name};

#[tauri::command]
pub async fn advance_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<GameStateInfo>, String> {
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

    let mut save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    if let Some(next_phase) = save.current_phase.next() {
        save.current_phase = next_phase;
        save.updated_at = Utc::now();

        if let Err(e) = SaveRepository::update(&pool, &save).await {
            return Ok(CommandResult::err(format!("Failed to update save: {}", e)));
        }
    } else {
        save.current_season += 1;
        save.current_phase = SeasonPhase::SpringRegular;
        save.updated_at = Utc::now();

        if let Err(e) = SaveRepository::update(&pool, &save).await {
            return Ok(CommandResult::err(format!("Failed to update save: {}", e)));
        }
    }

    let engine = SeasonProgressEngine::new(save.current_season, save.current_phase);
    let progress = engine.get_progress();
    let actions = engine.get_available_actions();

    let action_names: Vec<String> = actions
        .iter()
        .map(|a| format!("{:?}", a))
        .collect();

    Ok(CommandResult::ok(GameStateInfo {
        current_season: save.current_season,
        current_phase: format!("{:?}", save.current_phase),
        phase_name: get_phase_display_name(&save.current_phase),
        progress,
        available_actions: action_names,
    }))
}

#[tauri::command]
pub async fn initialize_current_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<PhaseInitResult>, String> {
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

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    let game_flow = GameFlowService::new();
    match game_flow
        .initialize_phase(&pool, &save_id, save.current_season as u64, save.current_phase)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to initialize phase: {}", e))),
    }
}

#[tauri::command]
pub async fn complete_current_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<PhaseCompleteResult>, String> {
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

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    let game_flow = GameFlowService::new();
    match game_flow
        .complete_phase(&pool, &save_id, save.current_season as u64, save.current_phase)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to complete phase: {}", e))),
    }
}

#[tauri::command]
pub async fn start_new_season(
    state: State<'_, AppState>,
) -> Result<CommandResult<crate::services::NewSeasonResult>, String> {
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

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    if save.current_phase != SeasonPhase::SeasonEnd {
        return Ok(CommandResult::err("只能在赛季结算阶段开始新赛季"));
    }

    let game_flow = GameFlowService::new();
    match game_flow.advance_to_new_season(&pool, &save_id).await {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to start new season: {}", e))),
    }
}
