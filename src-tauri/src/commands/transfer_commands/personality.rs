use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::TransferEngine;
use crate::models::transfer::*;
use sqlx::Row;
use tauri::State;

use super::UpdatePersonalityRequest;

#[tauri::command]
pub async fn get_team_personality(
    state: State<'_, AppState>,
    team_id: i64,
) -> Result<CommandResult<Option<TeamPersonalityConfig>>, String> {
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
        "SELECT * FROM team_personality_configs WHERE team_id = ?"
    )
    .bind(team_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            Ok(CommandResult::ok(Some(TeamPersonalityConfig {
                id: r.get("id"),
                team_id: r.get("team_id"),
                save_id: r.get("save_id"),
                personality: r.get("personality"),
                short_term_focus: r.get("short_term_focus"),
                long_term_focus: r.get("long_term_focus"),
                risk_tolerance: r.get("risk_tolerance"),
                youth_preference: r.get("youth_preference"),
                star_chasing: r.get("star_chasing"),
                bargain_hunting: r.get("bargain_hunting"),
                updated_at: r.get("updated_at"),
            })))
        }
        None => Ok(CommandResult::ok(None)),
    }
}

#[tauri::command]
pub async fn update_team_personality(
    state: State<'_, AppState>,
    team_id: i64,
    request: UpdatePersonalityRequest,
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

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };

    let personality = AITeamPersonality::from_str(&request.personality);
    let defaults = personality.default_weights();

    let short_term = request.short_term_focus.unwrap_or(defaults.short_term_focus);
    let long_term = request.long_term_focus.unwrap_or(defaults.long_term_focus);
    let risk = request.risk_tolerance.unwrap_or(defaults.risk_tolerance);
    let youth = request.youth_preference.unwrap_or(defaults.youth_preference);
    let star = request.star_chasing.unwrap_or(defaults.star_chasing);
    let bargain = request.bargain_hunting.unwrap_or(defaults.bargain_hunting);

    sqlx::query(
        r#"INSERT INTO team_personality_configs
           (team_id, save_id, personality, short_term_focus, long_term_focus,
            risk_tolerance, youth_preference, star_chasing, bargain_hunting, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
           ON CONFLICT(team_id) DO UPDATE SET
               personality = excluded.personality,
               short_term_focus = excluded.short_term_focus,
               long_term_focus = excluded.long_term_focus,
               risk_tolerance = excluded.risk_tolerance,
               youth_preference = excluded.youth_preference,
               star_chasing = excluded.star_chasing,
               bargain_hunting = excluded.bargain_hunting,
               updated_at = datetime('now')"#
    )
    .bind(team_id)
    .bind(&save_id)
    .bind(personality.as_str())
    .bind(short_term)
    .bind(long_term)
    .bind(risk)
    .bind(youth)
    .bind(star)
    .bind(bargain)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(true))
}

#[tauri::command]
pub async fn get_team_reputation(
    state: State<'_, AppState>,
    team_id: i64,
) -> Result<CommandResult<TeamReputation>, String> {
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

    let engine = TransferEngine::new();
    let reputation = engine.calculate_team_reputation(&pool, team_id, &save_id, current_season).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(reputation))
}
