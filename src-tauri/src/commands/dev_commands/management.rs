use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::SaveRepository;
use super::DevCommandResult;

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

    let save = SaveRepository::get_by_id(&pool, &save_id)
        .await
        .map_err(|e| e.to_string())?;

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

    Ok(DevCommandResult::ok(
        pending_matches.len() as i32,
        format!("找到 {} 场待进行比赛，请使用前端批量模拟功能", pending_matches.len()),
    ))
}

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
        sqlx::query(
            "UPDATE teams SET total_matches = 0, wins = 0, win_rate = 0, annual_points = 0, cross_year_points = 0 WHERE save_id = ?"
        )
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();
    }

    sqlx::query(
        "UPDATE saves SET current_season = 1, current_phase = 'SpringRegular', phase_completed = 0 WHERE id = ?"
    )
    .bind(&save_id)
    .execute(&pool)
    .await
    .ok();

    Ok(DevCommandResult::ok_msg("存档已重置到S1春季赛开始"))
}
