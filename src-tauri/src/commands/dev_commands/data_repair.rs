use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::{SaveRepository, TournamentRepository};
use crate::services::HonorService;
use crate::models::TournamentStatus;
use super::{DevCommandResult, SyncResult, FixStartersResult, TeamFixInfo};

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_reassign_honors(
    state: State<'_, AppState>,
    season_id: Option<i64>,
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

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    sqlx::query("DELETE FROM honors WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season)
        .execute(&pool)
        .await
        .ok();

    let tournaments = TournamentRepository::get_by_season(&pool, &save_id, season as u64)
        .await
        .map_err(|e| e.to_string())?;

    let completed_tournaments: Vec<_> = tournaments
        .into_iter()
        .filter(|t| t.status == TournamentStatus::Completed)
        .collect();

    let honor_service = HonorService::new();
    let mut honors_count = 0;

    for tournament in &completed_tournaments {
        match honor_service.process_tournament_completion(&pool, &save_id, tournament.id).await {
            Ok(honors) => {
                let count = honors.player_champions.len() + honors.player_runner_ups.len() +
                           if honors.tournament_mvp.is_some() { 1 } else { 0 } +
                           if honors.team_champion.is_some() { 1 } else { 0 } +
                           if honors.team_runner_up.is_some() { 1 } else { 0 };
                honors_count += count;
            },
            Err(e) => log::debug!("赛事 {} 颁发荣誉失败: {}", tournament.name, e),
        }
    }

    Ok(DevCommandResult::ok(
        honors_count as i32,
        format!("成功重新颁发 {} 项荣誉（{}个已完成赛事）", honors_count, completed_tournaments.len()),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_sync_player_games_played(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<SyncResult>, String> {
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

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    let real_counts: Vec<(i64, i64)> = sqlx::query_as(
        r#"
        SELECT gpp.player_id, COUNT(*) as real_count
        FROM game_player_performances gpp
        JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
        JOIN matches m ON mg.match_id = m.id
        JOIN tournaments t ON m.tournament_id = t.id
        WHERE gpp.save_id = ? AND t.season_id = ?
        GROUP BY gpp.player_id
        "#
    )
    .bind(&save_id)
    .bind(season)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut updated_count = 0;
    let mut details = Vec::new();

    for (player_id, real_count) in real_counts {
        let current: Option<(i64,)> = sqlx::query_as(
            "SELECT games_played FROM player_season_stats WHERE save_id = ? AND player_id = ? AND season_id = ?"
        )
        .bind(&save_id)
        .bind(player_id)
        .bind(season)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

        let current_count = current.map(|c| c.0).unwrap_or(0);

        if current_count != real_count {
            sqlx::query(
                "UPDATE player_season_stats SET games_played = ? WHERE save_id = ? AND player_id = ? AND season_id = ?"
            )
            .bind(real_count)
            .bind(&save_id)
            .bind(player_id)
            .bind(season)
            .execute(&pool)
            .await
            .ok();

            details.push(format!("选手#{}: {} -> {}", player_id, current_count, real_count));
            updated_count += 1;
        }
    }

    Ok(DevCommandResult::ok(
        SyncResult { updated_count, details },
        format!("同步完成，更新了 {} 名选手的场次统计", updated_count),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_fix_starters(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<FixStartersResult>, String> {
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

    let positions = vec!["Top", "Jug", "Mid", "Adc", "Sup"];
    let mut fixed_teams = Vec::new();
    let mut total_fixed = 0;

    let teams: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, name FROM teams WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    for (team_id, team_name) in teams {
        let mut team_fixes = Vec::new();

        for pos in &positions {
            let has_starter: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM players
                WHERE team_id = ? AND save_id = ? AND status = 'Active'
                AND UPPER(position) = UPPER(?) AND is_starter = 1
                "#
            )
            .bind(team_id)
            .bind(&save_id)
            .bind(pos)
            .fetch_one(&pool)
            .await
            .unwrap_or((0,));

            if has_starter.0 == 0 {
                let best_player: Option<(i64, String, i64)> = sqlx::query_as(
                    r#"
                    SELECT id, game_id, ability FROM players
                    WHERE team_id = ? AND save_id = ? AND status = 'Active'
                    AND UPPER(position) = UPPER(?)
                    ORDER BY ability DESC
                    LIMIT 1
                    "#
                )
                .bind(team_id)
                .bind(&save_id)
                .bind(pos)
                .fetch_optional(&pool)
                .await
                .unwrap_or(None);

                if let Some((player_id, game_id, ability)) = best_player {
                    sqlx::query("UPDATE players SET is_starter = 1 WHERE id = ?")
                        .bind(player_id)
                        .execute(&pool)
                        .await
                        .ok();

                    team_fixes.push(format!("{}: {} (能力{})", pos, game_id, ability));
                    total_fixed += 1;
                }
            }
        }

        if !team_fixes.is_empty() {
            fixed_teams.push(TeamFixInfo {
                team_name,
                fixes: team_fixes,
            });
        }
    }

    let teams_count = fixed_teams.len();
    Ok(DevCommandResult::ok(
        FixStartersResult {
            teams_fixed: teams_count as i32,
            players_fixed: total_fixed,
            details: fixed_teams,
        },
        format!("修复了 {} 支队伍的 {} 个首发位置", teams_count, total_fixed),
    ))
}

#[tauri::command]
pub async fn dev_force_complete_match(
    state: State<'_, AppState>,
    match_id: u64,
) -> Result<DevCommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    sqlx::query("UPDATE matches SET status = 'CANCELLED' WHERE id = ?")
        .bind(match_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DevCommandResult::ok_msg(format!("比赛 {} 已标记为 CANCELLED", match_id)))
}

#[tauri::command]
pub async fn dev_migrate_loyalty_satisfaction(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<u32>, String> {
    use crate::services::init_service::InitService;

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    match InitService::migrate_loyalty_satisfaction(&pool, &save_id).await {
        Ok(count) => Ok(DevCommandResult::ok(
            count,
            format!("成功迁移 {} 名选手的忠诚度和满意度", count)
        )),
        Err(e) => Ok(DevCommandResult::err(format!("迁移失败: {}", e))),
    }
}
