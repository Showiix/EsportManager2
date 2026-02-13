use crate::commands::save_commands::{AppState, CommandResult};
use sqlx::Row;
use tauri::State;

use super::{
    TeamSeasonEvaluationInfo, PositionNeedInfo, 
    PlayerListingEvaluationInfo, PlayerStayEvaluationInfo
};

#[tauri::command]
pub async fn get_team_evaluations(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<TeamSeasonEvaluationInfo>>, String> {
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

    let target_season = match season_id {
        Some(s) => s,
        None => {
            let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            save_row.get("current_season")
        }
    };

    let rows = sqlx::query(
        r#"
        SELECT
            e.id as evaluation_id,
            e.team_id,
            t.name as team_name,
            COALESCE(t.short_name, t.name) as team_short_name,
            r.short_name as region_code,
            e.season_id,
            e.current_rank,
            COALESCE(e.last_season_rank, e.current_rank) as last_rank,
            (SELECT ls.rank FROM league_standings ls
             JOIN tournaments tr ON ls.tournament_id = tr.id
             WHERE tr.save_id = e.save_id AND tr.season_id = e.season_id
             AND tr.tournament_type = 'SpringRegular' AND ls.team_id = e.team_id
             LIMIT 1) as spring_rank,
            (SELECT ls.rank FROM league_standings ls
             JOIN tournaments tr ON ls.tournament_id = tr.id
             WHERE tr.save_id = e.save_id AND tr.season_id = e.season_id
             AND tr.tournament_type = 'SummerRegular' AND ls.team_id = e.team_id
             LIMIT 1) as summer_rank,
            e.stability_score,
            e.strategy,
            e.urgency_level,
            e.roster_power,
            e.roster_count,
            e.roster_age_avg as avg_age,
            COALESCE((
                SELECT AVG(p.ability)
                FROM players p
                WHERE p.team_id = e.team_id AND p.is_starter = 1
            ), 0) as avg_ability,
            e.budget_remaining,
            COALESCE(e.strategy_reason, '') as evaluation_reason,
            e.created_at
        FROM team_season_evaluations e
        JOIN teams t ON e.team_id = t.id
        JOIN regions r ON t.region_id = r.id
        WHERE e.save_id = ? AND e.season_id = ?
        ORDER BY e.stability_score DESC, e.roster_power DESC
        "#
    )
    .bind(&save_id)
    .bind(target_season)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let evaluations: Vec<TeamSeasonEvaluationInfo> = rows
        .iter()
        .map(|row| TeamSeasonEvaluationInfo {
            evaluation_id: row.get("evaluation_id"),
            team_id: row.get("team_id"),
            team_name: row.get("team_name"),
            team_short_name: row.get("team_short_name"),
            region_code: row.get("region_code"),
            season_id: row.get("season_id"),
            current_rank: row.get("current_rank"),
            last_rank: row.get("last_rank"),
            spring_rank: row.try_get("spring_rank").ok(),
            summer_rank: row.try_get("summer_rank").ok(),
            stability_score: row.get("stability_score"),
            strategy: row.get("strategy"),
            urgency_level: row.get("urgency_level"),
            roster_power: row.get("roster_power"),
            roster_count: row.get("roster_count"),
            avg_age: row.get("avg_age"),
            avg_ability: row.get("avg_ability"),
            budget_remaining: row.get("budget_remaining"),
            evaluation_reason: row.get("evaluation_reason"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(CommandResult::ok(evaluations))
}

#[tauri::command]
pub async fn get_team_position_needs(
    state: State<'_, AppState>,
    team_id: i64,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<PositionNeedInfo>>, String> {
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

    let target_season = match season_id {
        Some(s) => s,
        None => {
            let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            save_row.get("current_season")
        }
    };

    let eval_row = sqlx::query(
        "SELECT id FROM team_season_evaluations WHERE team_id = ? AND season_id = ? AND save_id = ?"
    )
    .bind(team_id)
    .bind(target_season)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let evaluation_id: i64 = match eval_row {
        Some(row) => row.get("id"),
        None => return Ok(CommandResult::ok(vec![])),
    };

    let rows = sqlx::query(
        r#"
        SELECT
            position,
            current_starter_name,
            current_starter_ability,
            current_starter_age,
            need_level,
            min_ability_target,
            reason
        FROM team_position_needs
        WHERE evaluation_id = ?
        ORDER BY
            CASE need_level
                WHEN 'CRITICAL' THEN 1
                WHEN 'HIGH' THEN 2
                WHEN 'MEDIUM' THEN 3
                ELSE 4
            END
        "#
    )
    .bind(evaluation_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let needs: Vec<PositionNeedInfo> = rows
        .iter()
        .map(|row| PositionNeedInfo {
            position: row.get("position"),
            current_starter_name: row.get("current_starter_name"),
            current_starter_ability: row.get("current_starter_ability"),
            current_starter_age: row.get("current_starter_age"),
            need_level: row.get::<Option<String>, _>("need_level").unwrap_or_else(|| "LOW".to_string()),
            min_ability_target: row.get("min_ability_target"),
            reason: row.get("reason"),
        })
        .collect();

    Ok(CommandResult::ok(needs))
}

#[tauri::command]
pub async fn get_player_listing_evaluations(
    state: State<'_, AppState>,
    team_id: Option<i64>,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<PlayerListingEvaluationInfo>>, String> {
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

    let target_season = match season_id {
        Some(s) => s,
        None => {
            let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            save_row.get("current_season")
        }
    };

    let base_query = r#"
        SELECT
            le.player_id,
            p.game_id as player_name,
            p.position,
            p.age,
            p.ability,
            le.team_id,
            t.name as team_name,
            le.should_list,
            le.list_reason,
            le.is_protected,
            le.protect_reason,
            le.estimated_value
        FROM team_listing_evaluations le
        JOIN players p ON le.player_id = p.id
        JOIN teams t ON le.team_id = t.id
        WHERE le.save_id = ? AND le.season_id = ?
    "#;

    let rows = if let Some(tid) = team_id {
        sqlx::query(&format!("{} AND le.team_id = ? ORDER BY le.should_list DESC, p.ability DESC", base_query))
            .bind(&save_id)
            .bind(target_season)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query(&format!("{} ORDER BY le.should_list DESC, p.ability DESC", base_query))
            .bind(&save_id)
            .bind(target_season)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    };

    let evaluations: Vec<PlayerListingEvaluationInfo> = rows
        .iter()
        .map(|row| PlayerListingEvaluationInfo {
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            position: row.get("position"),
            age: row.get("age"),
            ability: row.get("ability"),
            team_id: row.get("team_id"),
            team_name: row.get("team_name"),
            should_list: row.get("should_list"),
            list_reason: row.get("list_reason"),
            is_protected: row.get("is_protected"),
            protect_reason: row.get("protect_reason"),
            estimated_value: row.get("estimated_value"),
        })
        .collect();

    Ok(CommandResult::ok(evaluations))
}

#[tauri::command]
pub async fn get_player_stay_evaluations(
    state: State<'_, AppState>,
    team_id: Option<i64>,
    season_id: Option<i64>,
) -> Result<CommandResult<Vec<PlayerStayEvaluationInfo>>, String> {
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

    let target_season = match season_id {
        Some(s) => s,
        None => {
            let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            save_row.get("current_season")
        }
    };

    let window_row = sqlx::query(
        "SELECT id FROM transfer_windows WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(target_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let window_id: i64 = match window_row {
        Some(row) => row.get("id"),
        None => return Ok(CommandResult::ok(vec![])),
    };

    let base_query = r#"
        SELECT
            se.player_id,
            p.game_id as player_name,
            p.position,
            p.age,
            p.ability,
            se.team_id,
            t.name as team_name,
            t.short_name as team_short_name,
            r.short_name as region_code,
            se.stay_score,
            se.wants_to_leave,
            se.leave_reason,
            p.salary,
            p.satisfaction,
            p.loyalty
        FROM player_season_evaluations se
        JOIN players p ON se.player_id = p.id
        JOIN teams t ON se.team_id = t.id
        JOIN regions r ON t.region_id = r.id
        WHERE se.save_id = ? AND se.window_id = ?
    "#;

    let rows = if let Some(tid) = team_id {
        sqlx::query(&format!("{} AND se.team_id = ? ORDER BY se.stay_score ASC, p.ability DESC", base_query))
            .bind(&save_id)
            .bind(window_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query(&format!("{} ORDER BY se.stay_score ASC, p.ability DESC", base_query))
            .bind(&save_id)
            .bind(window_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    };

    let evaluations: Vec<PlayerStayEvaluationInfo> = rows
        .iter()
        .map(|row| PlayerStayEvaluationInfo {
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            position: row.get("position"),
            age: row.get("age"),
            ability: row.get("ability"),
            team_id: row.get("team_id"),
            team_name: row.get("team_name"),
            team_short_name: row.get("team_short_name"),
            region_code: row.get("region_code"),
            stay_score: row.get("stay_score"),
            wants_to_leave: row.get("wants_to_leave"),
            leave_reason: row.get("leave_reason"),
            salary: row.get("salary"),
            satisfaction: row.get("satisfaction"),
            loyalty: row.get("loyalty"),
        })
        .collect();

    Ok(CommandResult::ok(evaluations))
}

#[tauri::command]
pub async fn clear_evaluation_data(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<CommandResult<i64>, String> {
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

    let target_season = match season_id {
        Some(s) => s,
        None => {
            let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?;
            save_row.get("current_season")
        }
    };

    let window_row = sqlx::query(
        "SELECT id FROM transfer_windows WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(target_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut deleted_count: i64 = 0;

    if let Some(row) = window_row {
        let window_id: i64 = row.get("id");

        let result1 = sqlx::query("DELETE FROM player_season_evaluations WHERE window_id = ?")
            .bind(window_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
        deleted_count += result1.rows_affected() as i64;

        let result2 = sqlx::query("DELETE FROM team_season_evaluations WHERE window_id = ?")
            .bind(window_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
        deleted_count += result2.rows_affected() as i64;

        sqlx::query("UPDATE transfer_windows SET current_round = 1 WHERE id = ?")
            .bind(window_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(deleted_count))
}
