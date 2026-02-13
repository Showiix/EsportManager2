use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{MatchRepository, StandingRepository, TeamRepository};
use sqlx::Row;
use tauri::State;
use super::{MatchInfo, StandingInfo, FixTournamentStatusResult};

#[tauri::command]
pub async fn get_tournament_matches(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<MatchInfo>>, String> {
    log::debug!("Called with tournament_id={}", tournament_id);
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let matches = match MatchRepository::get_by_tournament(&pool, tournament_id).await {
        Ok(m) => m,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get matches: {}", e))),
    };

    log::debug!("Found {} matches for tournament_id={}", matches.len(), tournament_id);

    let completed_count = matches.iter().filter(|m| m.status == crate::models::MatchStatus::Completed).count();
    let scheduled_count = matches.iter().filter(|m| m.status == crate::models::MatchStatus::Scheduled).count();
    log::debug!("Status: Completed={}, Scheduled={}", completed_count, scheduled_count);

    let infos: Vec<MatchInfo> = matches
        .into_iter()
        .map(|m| MatchInfo {
            id: m.id,
            tournament_id: m.tournament_id,
            stage: m.stage,
            round: m.round,
            match_order: m.match_order,
            format: format!("{:?}", m.format),
            home_team_id: m.home_team_id,
            away_team_id: m.away_team_id,
            home_score: m.home_score as u32,
            away_score: m.away_score as u32,
            winner_id: m.winner_id,
            status: format!("{:?}", m.status),
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

#[tauri::command]
pub async fn get_standings(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<StandingInfo>>, String> {
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

    let standings = match StandingRepository::get_by_tournament(&pool, tournament_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get standings: {}", e))),
    };

    let teams = match TeamRepository::get_all(&pool, &save_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get teams: {}", e))),
    };

    let team_names: std::collections::HashMap<u64, String> = teams
        .into_iter()
        .map(|t| (t.id, t.name))
        .collect();

    let infos: Vec<StandingInfo> = standings
        .into_iter()
        .map(|s| StandingInfo {
            team_id: s.team_id,
            team_name: team_names.get(&s.team_id).cloned(),
            rank: s.rank,
            matches_played: s.matches_played,
            wins: s.wins,
            losses: s.losses,
            points: s.points,
            game_diff: s.game_diff,
        })
        .collect();

    Ok(CommandResult::ok(infos))
}

#[tauri::command]
pub async fn fix_tournament_status(
    state: State<'_, AppState>,
) -> Result<CommandResult<FixTournamentStatusResult>, String> {
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

    let mut fixed_tournaments: Vec<String> = Vec::new();

    let tournaments = sqlx::query(
        r#"
        SELECT id, name, status FROM tournaments
        WHERE save_id = ? AND status != 'Completed'
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for tournament_row in tournaments {
        let tournament_id: i64 = tournament_row.get("id");
        let tournament_name: String = tournament_row.get("name");

        let match_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM matches WHERE tournament_id = ?"
        )
        .bind(tournament_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if match_count.0 == 0 {
            continue;
        }

        let pending_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM matches WHERE tournament_id = ? AND UPPER(status) != 'COMPLETED'"
        )
        .bind(tournament_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if pending_count.0 == 0 {
            sqlx::query("UPDATE tournaments SET status = 'Completed' WHERE id = ?")
                .bind(tournament_id)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

            fixed_tournaments.push(tournament_name);
            log::debug!("修复赛事状态: id={}, name={}", tournament_id, fixed_tournaments.last().unwrap());
        }
    }

    let count = fixed_tournaments.len() as u32;
    Ok(CommandResult::ok(FixTournamentStatusResult {
        fixed_count: count,
        fixed_tournaments,
        message: if count > 0 {
            format!("成功修复 {} 个赛事的状态", count)
        } else {
            "没有需要修复的赛事".to_string()
        },
    }))
}
