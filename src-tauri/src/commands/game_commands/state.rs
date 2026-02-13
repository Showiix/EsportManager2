use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::SaveRepository;
use crate::engines::SeasonProgressEngine;
use tauri::State;
use super::{GameStateInfo, get_phase_display_name};

#[tauri::command]
pub async fn get_game_state(
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

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

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
