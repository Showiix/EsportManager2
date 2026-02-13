use crate::commands::save_commands::{AppState, CommandResult};
use crate::services::TournamentService;
use sqlx::Row;
use tauri::State;

use super::helpers::*;

/// 创建世界赛
#[tauri::command]
pub async fn create_worlds_tournament(
    state: State<'_, AppState>,
    direct_team_ids: Vec<u64>,   // 4支直接晋级队
    group_team_ids: Vec<u64>,    // 8支小组赛队
) -> Result<CommandResult<u64>, String> {
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

    let tournament_id: i64 = sqlx::query(
        r#"
        INSERT INTO tournaments (save_id, name, tournament_type, season_id, region_id, status)
        VALUES (?, ?, 'WorldChampionship', ?, NULL, 'InProgress')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(format!("World Championship {}", current_season))
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    let direct_teams = get_teams_by_ids(&pool, &direct_team_ids).await?;
    let group_teams = get_teams_by_ids(&pool, &group_team_ids).await?;

    let service = TournamentService::new();
    let matches = service.generate_worlds_bracket(
        tournament_id as u64,
        &direct_teams,
        &group_teams,
    );

    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches (
                tournament_id, stage, round, match_order, format,
                home_team_id, away_team_id, home_score, away_score, winner_id, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL, 'Scheduled')
            "#,
        )
        .bind(tournament_id)
        .bind(&m.stage)
        .bind(m.round)
        .bind(m.match_order)
        .bind(format!("{:?}", m.format))
        .bind(if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None })
        .bind(if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None })
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(tournament_id as u64))
}

/// 填充世界赛淘汰赛对阵（瑞士轮完成后调用）
#[tauri::command]
pub async fn fill_worlds_knockout_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
    qualified_team_ids: Vec<u64>, // 瑞士轮晋级的4支队伍ID
) -> Result<CommandResult<Vec<u64>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    if qualified_team_ids.len() != 4 {
        return Ok(CommandResult::err(format!(
            "Expected 4 qualified teams, got {}",
            qualified_team_ids.len()
        )));
    }

    // 获取八强赛比赛（按 match_order 排序）
    let quarter_matches = sqlx::query(
        r#"
        SELECT id, match_order FROM matches
        WHERE tournament_id = ? AND stage = 'QUARTER_FINAL'
        ORDER BY match_order
        "#
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if quarter_matches.len() != 4 {
        return Ok(CommandResult::err(format!(
            "Expected 4 quarter-final matches, found {}",
            quarter_matches.len()
        )));
    }

    let mut updated_ids = Vec::new();

    // 更新每场八强赛的 away_team_id
    for (i, qm) in quarter_matches.iter().enumerate() {
        let match_id: i64 = qm.get("id");
        let qualified_team_id = qualified_team_ids[i];

        sqlx::query("UPDATE matches SET away_team_id = ? WHERE id = ?")
            .bind(qualified_team_id as i64)
            .bind(match_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        updated_ids.push(match_id as u64);
        log::debug!("Match {} updated with away_team_id {}", match_id, qualified_team_id);
    }

    Ok(CommandResult::ok(updated_ids))
}
