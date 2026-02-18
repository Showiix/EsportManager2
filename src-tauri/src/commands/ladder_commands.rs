use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::ladder::{LadderMatchmaker, LadderRatingEngine, LadderSimulator};
use crate::engines::ladder::matchmaker::LadderPlayer;
use crate::engines::ladder::simulator::PlayerFullData;
use crate::engines::traits::TraitType;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct LadderTournamentInfo {
    pub id: i64,
    pub save_id: String,
    pub season: i32,
    pub event_type: String,
    pub event_name: String,
    pub edition: i32,
    pub total_rounds: i32,
    pub current_round: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LadderRankingEntry {
    pub rank: i32,
    pub player_id: i64,
    pub player_name: String,
    pub game_id: String,
    pub position: String,
    pub team_name: Option<String>,
    pub rating: i32,
    pub games_played: i32,
    pub wins: i32,
    pub losses: i32,
    pub win_rate: f64,
    pub mvp_count: i32,
    pub avg_influence: f64,
    pub max_rating: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LadderMatchInfo {
    pub id: i64,
    pub round_number: i32,
    pub match_number: i32,
    pub blue_team: Vec<LadderPlayerInfo>,
    pub red_team: Vec<LadderPlayerInfo>,
    pub blue_avg_rating: i32,
    pub red_avg_rating: i32,
    pub winner_side: Option<String>,
    pub mvp_player_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LadderPlayerInfo {
    pub player_id: i64,
    pub player_name: String,
    #[serde(default)]
    pub game_id: String,
    pub position: String,
    pub team_name: Option<String>,
    pub rating: i32,
}

macro_rules! get_pool_and_save {
    ($state:expr) => {{
        let guard = $state.db.read().await;
        let db = match guard.as_ref() {
            Some(db) => db,
            None => return Ok(CommandResult::err("Database not initialized")),
        };
        let current_save = $state.current_save_id.read().await;
        let save_id = match current_save.as_ref() {
            Some(id) => id.clone(),
            None => return Ok(CommandResult::err("No save loaded")),
        };
        let pool = match db.get_pool().await {
            Ok(p) => p,
            Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
        };
        (pool, save_id)
    }};
}

#[tauri::command]
pub async fn initialize_ladder_tournament(
    state: State<'_, AppState>,
    event_type: String,
    season: i32,
) -> Result<CommandResult<LadderTournamentInfo>, String> {
    let (pool, save_id) = get_pool_and_save!(state);

    let event_name = match event_type.as_str() {
        "douyu" => "斗鱼巅峰赛",
        "douyin" => "抖音巅峰赛",
        "huya" => "虎牙巅峰赛",
        _ => return Ok(CommandResult::err("Invalid event type")),
    };

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ladder_tournament WHERE save_id = ? AND event_type = ?"
    )
    .bind(&save_id)
    .bind(&event_type)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let edition = (count + 1) as i32;

    let tournament_id: i64 = sqlx::query_scalar(
        "INSERT INTO ladder_tournament (save_id, season, event_type, event_name, edition, total_rounds, current_round, status) VALUES (?, ?, ?, ?, ?, 12, 0, 'pending') RETURNING id"
    )
    .bind(&save_id)
    .bind(season)
    .bind(&event_type)
    .bind(event_name)
    .bind(edition)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let players = sqlx::query(
        "SELECT p.id, p.real_name, p.game_id, p.position, t.name as team_name FROM players p LEFT JOIN teams t ON p.team_id = t.id WHERE p.save_id = ? AND p.status = 'Active'"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in &players {
        let player_id: i64 = row.get("id");
        let player_name: String = row.get("real_name");
        let game_id: String = row.get("game_id");
        let position: String = row.get("position");
        let team_name: Option<String> = row.get("team_name");

        sqlx::query(
            "INSERT INTO ladder_rating (save_id, ladder_tournament_id, player_id, player_name, game_id, position, team_name, rating) VALUES (?, ?, ?, ?, ?, ?, ?, 1200)"
        )
        .bind(&save_id)
        .bind(tournament_id)
        .bind(player_id)
        .bind(&player_name)
        .bind(&game_id)
        .bind(&position)
        .bind(&team_name)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(LadderTournamentInfo {
        id: tournament_id,
        save_id,
        season,
        event_type,
        event_name: event_name.to_string(),
        edition,
        total_rounds: 12,
        current_round: 0,
        status: "pending".to_string(),
    }))
}

#[tauri::command]
pub async fn simulate_ladder_round(
    state: State<'_, AppState>,
    tournament_id: i64,
) -> Result<CommandResult<String>, String> {
    let (pool, save_id) = get_pool_and_save!(state);

    let current_round: i32 = sqlx::query_scalar(
        "SELECT current_round FROM ladder_tournament WHERE id = ?"
    )
    .bind(tournament_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let next_round = current_round + 1;

    let players: Vec<LadderPlayer> = sqlx::query_as::<_, (i64, String, String, String, Option<String>, i32)>(
        "SELECT player_id, player_name, game_id, position, team_name, rating FROM ladder_rating WHERE ladder_tournament_id = ?"
    )
    .bind(tournament_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(|(player_id, player_name, game_id, position, team_name, rating)| LadderPlayer {
        player_id,
        player_name,
        game_id,
        position,
        team_name,
        rating,
    })
    .collect();

    let mut bye_players = Vec::new();
    let matches = LadderMatchmaker::create_round_matches(players, &mut bye_players);

    let version_tiers: HashMap<u8, crate::engines::champion::VersionTier> = 
        crate::engines::champion::calculate_version_tiers(crate::engines::meta_engine::MetaType::Balanced)
        .into_iter()
        .collect();
    
    let mastery_rows: Vec<(i64, i32, String)> = sqlx::query_as(
        "SELECT player_id, champion_id, mastery_tier FROM player_champion_mastery WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut mastery_map: HashMap<i64, HashMap<u8, crate::engines::champion::MasteryTier>> = HashMap::new();
    for (pid, cid, tier_str) in &mastery_rows {
        if let Some(tier) = crate::engines::champion::MasteryTier::from_id(tier_str) {
            mastery_map.entry(*pid).or_default().insert(*cid as u8, tier);
        }
    }

    let trait_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT player_id, trait_type FROM player_traits WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut trait_map: HashMap<i64, Vec<TraitType>> = HashMap::new();
    for (pid, trait_str) in &trait_rows {
        if let Ok(t) = serde_json::from_str::<TraitType>(&format!("\"{}\"", trait_str)) {
            trait_map.entry(*pid).or_default().push(t);
        }
    }

    let player_full_data: HashMap<i64, PlayerFullData> = sqlx::query_as::<_, (i64, i32)>(
        "SELECT id, ability FROM players WHERE save_id = ?"
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(|(id, ability)| {
        (id, PlayerFullData {
            ability: ability as u8,
            masteries: mastery_map.remove(&id).unwrap_or_default(),
            traits: trait_map.remove(&id).unwrap_or_default(),
        })
    })
    .collect();

    let simulator = LadderSimulator::new();

    for (match_number, ladder_match) in matches.iter().enumerate() {
        let result = simulator.simulate_match(
            ladder_match,
            next_round,
            match_number as i32 + 1,
            &player_full_data,
            &version_tiers,
        );

        let blue_team_json = serde_json::to_string(&result.blue_team).unwrap_or_default();
        let red_team_json = serde_json::to_string(&result.red_team).unwrap_or_default();
        let performances_json = serde_json::to_string(&result.performances).unwrap_or_default();

        let blue_players: Vec<(i64, i32)> = ladder_match.blue_team.players.iter()
            .map(|p| (p.player_id, p.rating))
            .collect();
        let red_players: Vec<(i64, i32)> = ladder_match.red_team.players.iter()
            .map(|p| (p.player_id, p.rating))
            .collect();

        let rating_updates = LadderRatingEngine::calculate_rating_changes(
            &blue_players,
            &red_players,
            &result.performances,
            &result.performances,
            &result.winner_side,
        );

        let rating_changes_map: HashMap<String, i32> = rating_updates.iter()
            .map(|u| (u.player_id.to_string(), u.rating_change))
            .collect();
        let rating_changes_json = serde_json::to_string(&rating_changes_map).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO ladder_match (
                save_id, ladder_tournament_id, round_number, match_number,
                blue_team_json, red_team_json, blue_avg_rating, red_avg_rating,
                blue_power, red_power, winner_side, mvp_player_id, mvp_player_name,
                game_duration, performances_json, draft_result_json, rating_changes_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&save_id)
        .bind(tournament_id)
        .bind(result.round_number)
        .bind(result.match_number)
        .bind(blue_team_json)
        .bind(red_team_json)
        .bind(result.blue_avg_rating)
        .bind(result.red_avg_rating)
        .bind(result.blue_power)
        .bind(result.red_power)
        .bind(&result.winner_side)
        .bind(result.mvp_player_id)
        .bind(&result.mvp_player_name)
        .bind(result.game_duration)
        .bind(performances_json)
        .bind(result.draft_result.as_deref())
        .bind(&rating_changes_json)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        for update in &rating_updates {
            let is_winner = (result.winner_side == "blue" && blue_players.iter().any(|(id, _)| *id == update.player_id))
                || (result.winner_side == "red" && red_players.iter().any(|(id, _)| *id == update.player_id));
            
            let is_mvp = update.player_id == result.mvp_player_id;

            sqlx::query(
                r#"
                UPDATE ladder_rating
                SET rating = ?,
                    games_played = games_played + 1,
                    wins = wins + ?,
                    losses = losses + ?,
                    mvp_count = mvp_count + ?,
                    max_rating = MAX(max_rating, ?),
                    total_influence = total_influence + ?,
                    avg_influence = (total_influence + ?) / (games_played + 1)
                WHERE ladder_tournament_id = ? AND player_id = ?
                "#
            )
            .bind(update.new_rating)
            .bind(if is_winner { 1 } else { 0 })
            .bind(if is_winner { 0 } else { 1 })
            .bind(if is_mvp { 1 } else { 0 })
            .bind(update.new_rating)
            .bind(update.influence)
            .bind(update.influence)
            .bind(tournament_id)
            .bind(update.player_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    sqlx::query(
        "UPDATE ladder_tournament SET current_round = ?, status = 'ongoing' WHERE id = ?"
    )
    .bind(next_round)
    .bind(tournament_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(format!("第 {} 轮模拟完成", next_round)))
}

#[tauri::command]
pub async fn get_ladder_rankings(
    state: State<'_, AppState>,
    tournament_id: i64,
) -> Result<CommandResult<Vec<LadderRankingEntry>>, String> {
    let (pool, _save_id) = get_pool_and_save!(state);
    let rankings: Vec<(i64, String, String, String, Option<String>, i32, i32, i32, i32, i32, f64, i32)> = sqlx::query_as(
        r#"
        SELECT lr.player_id, lr.player_name, COALESCE(NULLIF(lr.game_id, ''), p.game_id, lr.player_name) as game_id, lr.position, lr.team_name, lr.rating, lr.games_played, lr.wins, lr.losses, lr.mvp_count, lr.avg_influence, lr.max_rating
        FROM ladder_rating lr
        LEFT JOIN players p ON lr.player_id = p.id
        WHERE lr.ladder_tournament_id = ?
        ORDER BY lr.rating DESC, lr.wins DESC, lr.mvp_count DESC
        "#
    )
    .bind(tournament_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let entries: Vec<LadderRankingEntry> = rankings.into_iter().enumerate().map(|(idx, row)| {
        let win_rate = if row.6 > 0 {
            (row.7 as f64 / row.6 as f64) * 100.0
        } else {
            0.0
        };

        LadderRankingEntry {
            rank: (idx + 1) as i32,
            player_id: row.0,
            player_name: row.1,
            game_id: row.2,
            position: row.3,
            team_name: row.4,
            rating: row.5,
            games_played: row.6,
            wins: row.7,
            losses: row.8,
            win_rate,
            mvp_count: row.9,
            avg_influence: row.10,
            max_rating: row.11,
        }
    }).collect();

    Ok(CommandResult::ok(entries))
}

#[tauri::command]
pub async fn get_ladder_matches(
    state: State<'_, AppState>,
    tournament_id: i64,
    round_number: Option<i32>,
) -> Result<CommandResult<Vec<LadderMatchInfo>>, String> {
    let (pool, _save_id) = get_pool_and_save!(state);

    let rows = if let Some(round) = round_number {
        sqlx::query_as::<_, (i64, i32, i32, String, String, i32, i32, Option<String>, Option<String>)>(
            "SELECT id, round_number, match_number, blue_team_json, red_team_json, blue_avg_rating, red_avg_rating, winner_side, mvp_player_name FROM ladder_match WHERE ladder_tournament_id = ? AND round_number = ? ORDER BY match_number"
        )
        .bind(tournament_id)
        .bind(round)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as::<_, (i64, i32, i32, String, String, i32, i32, Option<String>, Option<String>)>(
            "SELECT id, round_number, match_number, blue_team_json, red_team_json, blue_avg_rating, red_avg_rating, winner_side, mvp_player_name FROM ladder_match WHERE ladder_tournament_id = ? ORDER BY round_number, match_number"
        )
        .bind(tournament_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let game_id_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT lr.player_id, p.game_id FROM ladder_rating lr JOIN players p ON lr.player_id = p.id WHERE lr.ladder_tournament_id = ?"
    )
    .bind(tournament_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let game_id_map: HashMap<i64, String> = game_id_rows.into_iter().collect();

    let match_infos: Vec<LadderMatchInfo> = rows.into_iter().map(|row| {
        let mut blue_team: Vec<LadderPlayerInfo> = serde_json::from_str(&row.3).unwrap_or_default();
        let mut red_team: Vec<LadderPlayerInfo> = serde_json::from_str(&row.4).unwrap_or_default();

        for p in blue_team.iter_mut().chain(red_team.iter_mut()) {
            if p.game_id.is_empty() {
                if let Some(gid) = game_id_map.get(&p.player_id) {
                    p.game_id = gid.clone();
                }
            }
        }

        LadderMatchInfo {
            id: row.0,
            round_number: row.1,
            match_number: row.2,
            blue_team,
            red_team,
            blue_avg_rating: row.5,
            red_avg_rating: row.6,
            winner_side: row.7,
            mvp_player_name: row.8,
        }
    }).collect();

    Ok(CommandResult::ok(match_infos))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LadderMatchDetail {
    pub id: i64,
    pub round_number: i32,
    pub match_number: i32,
    pub blue_team: Vec<LadderPlayerInfo>,
    pub red_team: Vec<LadderPlayerInfo>,
    pub blue_avg_rating: i32,
    pub red_avg_rating: i32,
    pub blue_power: f64,
    pub red_power: f64,
    pub winner_side: Option<String>,
    pub mvp_player_id: Option<i64>,
    pub mvp_player_name: Option<String>,
    pub game_duration: Option<i32>,
    pub performances: Option<HashMap<String, f64>>,
    pub draft_result_json: Option<String>,
    pub rating_changes: Option<HashMap<String, i32>>,
}

#[tauri::command]
pub async fn get_ladder_match_detail(
    state: State<'_, AppState>,
    match_id: i64,
) -> Result<CommandResult<LadderMatchDetail>, String> {
    let (pool, _save_id) = get_pool_and_save!(state);

    let row = sqlx::query(
        r#"
        SELECT id, round_number, match_number, blue_team_json, red_team_json,
               blue_avg_rating, red_avg_rating, blue_power, red_power,
               winner_side, mvp_player_id, mvp_player_name, game_duration,
               performances_json, draft_result_json, rating_changes_json
        FROM ladder_match WHERE id = ?
        "#
    )
    .bind(match_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let row = match row {
        Some(r) => r,
        None => return Ok(CommandResult::err("对局不存在")),
    };

    let mut blue_team: Vec<LadderPlayerInfo> = serde_json::from_str(row.get::<String, _>("blue_team_json").as_str()).unwrap_or_default();
    let mut red_team: Vec<LadderPlayerInfo> = serde_json::from_str(row.get::<String, _>("red_team_json").as_str()).unwrap_or_default();
    let performances: Option<HashMap<String, f64>> = row.get::<Option<String>, _>("performances_json")
        .and_then(|json| serde_json::from_str(&json).ok());
    let rating_changes: Option<HashMap<String, i32>> = row.get::<Option<String>, _>("rating_changes_json")
        .and_then(|json| serde_json::from_str(&json).ok());

    let tournament_id: i64 = sqlx::query_scalar(
        "SELECT ladder_tournament_id FROM ladder_match WHERE id = ?"
    )
    .bind(match_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let gid_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT lr.player_id, p.game_id FROM ladder_rating lr JOIN players p ON lr.player_id = p.id WHERE lr.ladder_tournament_id = ?"
    )
    .bind(tournament_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let gid_map: HashMap<i64, String> = gid_rows.into_iter().collect();

    for p in blue_team.iter_mut().chain(red_team.iter_mut()) {
        if p.game_id.is_empty() {
            if let Some(gid) = gid_map.get(&p.player_id) {
                p.game_id = gid.clone();
            }
        }
    }

    Ok(CommandResult::ok(LadderMatchDetail {
        id: row.get("id"),
        round_number: row.get("round_number"),
        match_number: row.get("match_number"),
        blue_team,
        red_team,
        blue_avg_rating: row.get("blue_avg_rating"),
        red_avg_rating: row.get("red_avg_rating"),
        blue_power: row.get("blue_power"),
        red_power: row.get("red_power"),
        winner_side: row.get("winner_side"),
        mvp_player_id: row.get("mvp_player_id"),
        mvp_player_name: row.get("mvp_player_name"),
        game_duration: row.get("game_duration"),
        performances,
        draft_result_json: row.get("draft_result_json"),
        rating_changes,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LadderCompletionResult {
    pub total_players: i32,
    pub rewards_distributed: Vec<PlayerReward>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerReward {
    pub player_id: i64,
    pub player_name: String,
    pub rank: i32,
    pub rating: i32,
    pub ability_gain: i32,
    pub trait_unlocked: Option<String>,
}

#[tauri::command]
pub async fn complete_ladder_tournament(
    state: State<'_, AppState>,
    tournament_id: i64,
) -> Result<CommandResult<LadderCompletionResult>, String> {
    let (pool, _save_id) = get_pool_and_save!(state);
    let tournament_info: (i32, i32, String) = sqlx::query_as(
        "SELECT current_round, total_rounds, status FROM ladder_tournament WHERE id = ?"
    )
    .bind(tournament_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("查询天梯赛失败: {}", e))?;

    let (current_round, total_rounds, status) = tournament_info;

    if status == "completed" {
        return Ok(CommandResult::err("天梯赛已经完成"));
    }

    if current_round < total_rounds {
        return Ok(CommandResult::err(format!("天梯赛未完成所有轮次 ({}/{})", current_round, total_rounds)));
    }

    let rankings: Vec<(i64, String, i32)> = sqlx::query_as(
        "SELECT player_id, player_name, rating FROM ladder_rating WHERE ladder_tournament_id = ? ORDER BY rating DESC, wins DESC, mvp_count DESC"
    )
    .bind(tournament_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let rewards: Vec<PlayerReward> = rankings.iter().take(10).enumerate().map(|(i, (pid, name, rating))| {
        PlayerReward {
            player_id: *pid,
            player_name: name.clone(),
            rank: (i + 1) as i32,
            rating: *rating,
            ability_gain: 0,
            trait_unlocked: None,
        }
    }).collect();

    sqlx::query(
        "UPDATE ladder_tournament SET status = 'completed' WHERE id = ?"
    )
    .bind(tournament_id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(LadderCompletionResult {
        total_players: rankings.len() as i32,
        rewards_distributed: rewards,
    }))
}

#[tauri::command]
pub async fn get_ladder_tournaments(
    state: State<'_, AppState>,
    season_id: i32,
) -> Result<CommandResult<Vec<LadderTournamentInfo>>, String> {
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

    let rows = sqlx::query(
        "SELECT id, save_id, season, event_type, event_name, edition, total_rounds, current_round, status FROM ladder_tournament WHERE save_id = ? AND season = ? ORDER BY id"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournaments: Vec<LadderTournamentInfo> = rows.iter().map(|row| {
        LadderTournamentInfo {
            id: row.get("id"),
            save_id: row.get("save_id"),
            season: row.get("season"),
            event_type: row.get("event_type"),
            event_name: row.get("event_name"),
            edition: row.get("edition"),
            total_rounds: row.get("total_rounds"),
            current_round: row.get("current_round"),
            status: row.get("status"),
        }
    }).collect();

    Ok(CommandResult::ok(tournaments))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RatingHistoryPoint {
    pub round: i32,
    pub rating: i32,
    pub change: i32,
}

#[tauri::command]
pub async fn get_player_ladder_rating_history(
    state: State<'_, AppState>,
    tournament_id: i64,
    player_id: i64,
) -> Result<CommandResult<Vec<RatingHistoryPoint>>, String> {
    let (pool, _save_id) = get_pool_and_save!(state);

    let rows = sqlx::query(
        r#"
        SELECT round_number, rating_changes_json
        FROM ladder_match
        WHERE ladder_tournament_id = ? AND rating_changes_json IS NOT NULL
        ORDER BY round_number ASC, match_number ASC
        "#
    )
    .bind(tournament_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut history: Vec<RatingHistoryPoint> = Vec::new();
    let mut current_rating = 1200i32;

    history.push(RatingHistoryPoint { round: 0, rating: current_rating, change: 0 });

    for row in &rows {
        let round_number: i32 = row.get("round_number");
        let json: Option<String> = row.get("rating_changes_json");
        if let Some(json_str) = json {
            if let Ok(changes) = serde_json::from_str::<HashMap<String, i32>>(&json_str) {
                let pid_str = player_id.to_string();
                if let Some(&change) = changes.get(&pid_str) {
                    current_rating += change;
                    history.push(RatingHistoryPoint {
                        round: round_number,
                        rating: current_rating,
                        change,
                    });
                }
            }
        }
    }

    Ok(CommandResult::ok(history))
}
