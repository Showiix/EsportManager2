use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::MatchSimulationEngine;
use sqlx::Row;
use tauri::State;

use super::{
    MatchGameLineup, MatchLineupEntry, MatchLineupsResult, MatchPrediction, PlayerSeasonStats,
};

/// 获取球员赛季统计
#[tauri::command]
pub async fn get_player_season_stats(
    state: State<'_, AppState>,
    player_id: u64,
    _season_id: Option<u64>,
) -> Result<CommandResult<PlayerSeasonStats>, String> {
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

    // 获取球员信息
    let player_row = sqlx::query(
        "SELECT id, game_id, team_id, position FROM players WHERE id = ? AND save_id = ?"
    )
    .bind(player_id as i64)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let player_row = match player_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Player not found")),
    };

    // 查询球员统计（如果有统计表的话）
    // 这里返回模拟数据
    Ok(CommandResult::ok(PlayerSeasonStats {
        player_id,
        player_name: player_row.get("game_id"),
        team_id: player_row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
        position: player_row.get::<Option<String>, _>("position").unwrap_or_default(),
        games_played: 0,
        total_kills: 0,
        total_deaths: 0,
        total_assists: 0,
        average_kda: 0.0,
        average_cs_per_min: 0.0,
        average_damage: 0,
        mvp_count: 0,
        win_rate: 0.0,
    }))
}

/// 获取比赛预测
#[tauri::command]
pub async fn get_match_prediction(
    state: State<'_, AppState>,
    match_id: u64,
) -> Result<CommandResult<MatchPrediction>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let match_row = sqlx::query(
        r#"
        SELECT m.format,
               ht.id as home_id, ht.name as home_name, ht.power_rating as home_power,
               at.id as away_id, at.name as away_name, at.power_rating as away_power
        FROM matches m
        JOIN teams ht ON m.home_team_id = ht.id
        JOIN teams at ON m.away_team_id = at.id
        WHERE m.id = ?
        "#,
    )
    .bind(match_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_row = match match_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Match not found")),
    };

    let engine = MatchSimulationEngine::default();
    let home_power: f64 = match_row.get("home_power");
    let away_power: f64 = match_row.get("away_power");

    let home_win_prob = engine.calculate_win_probability(home_power, away_power);

    Ok(CommandResult::ok(MatchPrediction {
        match_id,
        home_team_id: match_row.get::<i64, _>("home_id") as u64,
        home_team_name: match_row.get("home_name"),
        home_power,
        home_win_probability: home_win_prob,
        away_team_id: match_row.get::<i64, _>("away_id") as u64,
        away_team_name: match_row.get("away_name"),
        away_power,
        away_win_probability: 1.0 - home_win_prob,
        predicted_score: predict_score(home_win_prob, &match_row.get::<String, _>("format")),
    }))
}

#[tauri::command]
pub async fn get_match_lineups(
    state: State<'_, AppState>,
    match_id: u64,
) -> Result<CommandResult<MatchLineupsResult>, String> {
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

    let match_row = sqlx::query(
        "SELECT home_team_id, away_team_id FROM matches WHERE id = ?"
    )
    .bind(match_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_row = match match_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Match not found")),
    };

    let home_team_id = match_row.get::<i64, _>("home_team_id") as u64;
    let away_team_id = match_row.get::<i64, _>("away_team_id") as u64;

    let rows = sqlx::query(
        r#"
        SELECT ml.game_number, ml.team_id, ml.player_id, ml.position,
               ml.is_substitution, ml.replaced_player_id, ml.substitution_reason,
               p.game_id as player_name,
               rp.game_id as replaced_player_name
        FROM match_lineups ml
        JOIN players p ON ml.player_id = p.id AND ml.save_id = p.save_id
        LEFT JOIN players rp ON ml.replaced_player_id = rp.id AND ml.save_id = rp.save_id
        WHERE ml.save_id = ? AND ml.match_id = ?
        ORDER BY ml.game_number, ml.team_id, ml.is_substitution, ml.position
        "#,
    )
    .bind(&save_id)
    .bind(match_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut games_map: std::collections::BTreeMap<u8, (Vec<MatchLineupEntry>, Vec<MatchLineupEntry>, Vec<MatchLineupEntry>)> =
        std::collections::BTreeMap::new();

    for row in &rows {
        let game_number = row.get::<i64, _>("game_number") as u8;
        let team_id = row.get::<i64, _>("team_id") as u64;
        let is_sub = row.get::<i64, _>("is_substitution") != 0;

        let entry = MatchLineupEntry {
            game_number,
            team_id,
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get::<String, _>("player_name"),
            position: row.get::<String, _>("position"),
            is_substitution: is_sub,
            replaced_player_id: row
                .get::<Option<i64>, _>("replaced_player_id")
                .map(|id| id as u64),
            replaced_player_name: row.get::<Option<String>, _>("replaced_player_name"),
            substitution_reason: row.get::<Option<String>, _>("substitution_reason"),
        };

        let (home, away, subs) = games_map.entry(game_number).or_insert_with(|| {
            (Vec::new(), Vec::new(), Vec::new())
        });

        if is_sub {
            subs.push(entry);
        } else if team_id == home_team_id {
            home.push(entry);
        } else if team_id == away_team_id {
            away.push(entry);
        }
    }

    let games: Vec<MatchGameLineup> = games_map
        .into_iter()
        .map(|(game_number, (home_players, away_players, substitutions))| {
            MatchGameLineup {
                game_number,
                home_players,
                away_players,
                substitutions,
            }
        })
        .collect();

    Ok(CommandResult::ok(MatchLineupsResult {
        match_id,
        games,
    }))
}

// ==================== 辅助函数 ====================

fn predict_score(win_prob: f64, format: &str) -> String {
    let wins_needed = match format {
        "Bo1" => 1,
        "Bo3" => 2,
        "Bo5" => 3,
        _ => 2,
    };

    if win_prob > 0.65 {
        format!("{}-0", wins_needed)
    } else if win_prob > 0.55 {
        format!("{}-1", wins_needed)
    } else if win_prob > 0.45 {
        format!("{}-{}", wins_needed, wins_needed - 1)
    } else if win_prob > 0.35 {
        format!("1-{}", wins_needed)
    } else {
        format!("0-{}", wins_needed)
    }
}
