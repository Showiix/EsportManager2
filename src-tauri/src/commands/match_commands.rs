use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::MatchSimulationEngine;
use crate::models::MatchFormat;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 详细比赛结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedMatchResult {
    pub match_id: u64,
    pub tournament_id: u64,
    pub home_team_id: u64,
    pub away_team_id: u64,
    pub home_score: u8,
    pub away_score: u8,
    pub winner_id: u64,
    pub games: Vec<DetailedGameResult>,
    pub match_mvp: Option<PlayerMvpInfo>,
    pub home_team_stats: TeamMatchStats,
    pub away_team_stats: TeamMatchStats,
}

/// 详细小局结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedGameResult {
    pub game_number: u8,
    pub winner_id: u64,
    pub duration_minutes: u32,
    pub home_performance: f64,
    pub away_performance: f64,
    pub game_mvp: PlayerMvpInfo,
    pub home_players: Vec<PlayerGameStats>,
    pub away_players: Vec<PlayerGameStats>,
    pub key_events: Vec<GameEvent>,
}

/// 球员MVP信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMvpInfo {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub position: String,
    pub mvp_score: f64,
}

/// 队伍比赛统计
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMatchStats {
    pub team_id: u64,
    pub total_kills: u32,
    pub total_deaths: u32,
    pub total_assists: u32,
    pub total_gold: u64,
    pub average_game_duration: u32,
    pub first_blood_rate: f64,
    pub first_tower_rate: f64,
    pub baron_rate: f64,
    pub dragon_rate: f64,
}

/// 球员单局统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameStats {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub cs: u32,
    pub gold: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub vision_score: u32,
    pub mvp_score: f64,
}

/// 比赛关键事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub time_minutes: u32,
    pub event_type: String,
    pub description: String,
    pub team_id: u64,
}

/// 球员赛季统计
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSeasonStats {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub position: String,
    pub games_played: u32,
    pub total_kills: u32,
    pub total_deaths: u32,
    pub total_assists: u32,
    pub average_kda: f64,
    pub average_cs_per_min: f64,
    pub average_damage: u64,
    pub mvp_count: u32,
    pub win_rate: f64,
}

/// 模拟比赛并返回详细结果
#[tauri::command]
pub async fn simulate_match_detailed(
    state: State<'_, AppState>,
    match_id: u64,
) -> Result<CommandResult<DetailedMatchResult>, String> {
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

    // 获取比赛信息
    let match_row = sqlx::query(
        r#"
        SELECT m.id, m.tournament_id, m.format, m.home_team_id, m.away_team_id,
               ht.power_rating as home_power, at.power_rating as away_power
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

    let tournament_id: i64 = match_row.get("tournament_id");
    let home_team_id: i64 = match_row.get("home_team_id");
    let away_team_id: i64 = match_row.get("away_team_id");
    let home_power: f64 = match_row.get("home_power");
    let away_power: f64 = match_row.get("away_power");
    let format_str: String = match_row.get("format");

    let format = match format_str.as_str() {
        "Bo1" => MatchFormat::Bo1,
        "Bo3" => MatchFormat::Bo3,
        "Bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    };

    // 获取双方首发球员
    let home_players = get_starting_players(&pool, &save_id, home_team_id as u64).await?;
    let away_players = get_starting_players(&pool, &save_id, away_team_id as u64).await?;

    // 模拟比赛
    let engine = MatchSimulationEngine::default();
    let mut rng = StdRng::from_entropy();

    let wins_needed = format.wins_needed();
    let mut home_score: u8 = 0;
    let mut away_score: u8 = 0;
    let mut games = Vec::new();
    let mut game_number: u8 = 1;

    let mut total_home_stats = TeamMatchStats::default(home_team_id as u64);
    let mut total_away_stats = TeamMatchStats::default(away_team_id as u64);

    while home_score < wins_needed && away_score < wins_needed {
        // 模拟单局
        let (home_perf, away_perf, winner_id) = engine.simulate_game(
            home_power, away_power,
            home_team_id as u64, away_team_id as u64
        );

        let duration = 25 + rng.gen_range(0..25); // 25-50分钟

        // 生成球员统计
        let (home_player_stats, away_player_stats) = generate_player_stats(
            &home_players, &away_players,
            home_perf, away_perf,
            winner_id == home_team_id as u64,
            duration,
            &mut rng,
        );

        // 选择MVP
        let all_stats: Vec<&PlayerGameStats> = home_player_stats.iter()
            .chain(away_player_stats.iter())
            .collect();
        let game_mvp = select_mvp(&all_stats);

        // 生成关键事件
        let events = generate_key_events(
            home_team_id as u64, away_team_id as u64,
            winner_id == home_team_id as u64,
            duration,
            &mut rng,
        );

        // 更新队伍总统计
        update_team_stats(&mut total_home_stats, &home_player_stats, &events, true);
        update_team_stats(&mut total_away_stats, &away_player_stats, &events, false);

        games.push(DetailedGameResult {
            game_number,
            winner_id,
            duration_minutes: duration,
            home_performance: home_perf,
            away_performance: away_perf,
            game_mvp,
            home_players: home_player_stats,
            away_players: away_player_stats,
            key_events: events,
        });

        if winner_id == home_team_id as u64 {
            home_score += 1;
        } else {
            away_score += 1;
        }

        game_number += 1;
    }

    let winner_id = if home_score > away_score {
        home_team_id as u64
    } else {
        away_team_id as u64
    };

    // 计算平均时长
    let total_duration: u32 = games.iter().map(|g| g.duration_minutes).sum();
    total_home_stats.average_game_duration = total_duration / games.len() as u32;
    total_away_stats.average_game_duration = total_duration / games.len() as u32;

    // 选择比赛MVP
    let match_mvp = select_match_mvp(&games);

    // 更新数据库中的比赛结果
    sqlx::query(
        "UPDATE matches SET home_score = ?, away_score = ?, winner_id = ?, status = 'Completed' WHERE id = ?"
    )
    .bind(home_score as i64)
    .bind(away_score as i64)
    .bind(winner_id as i64)
    .bind(match_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 保存小局详情
    for game in &games {
        sqlx::query(
            r#"
            INSERT INTO match_games (
                match_id, game_number, home_power, away_power,
                home_performance, away_performance, winner_id, duration_minutes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(match_id as i64)
        .bind(game.game_number as i64)
        .bind(home_power)
        .bind(away_power)
        .bind(game.home_performance)
        .bind(game.away_performance)
        .bind(game.winner_id as i64)
        .bind(game.duration_minutes as i64)
        .execute(&pool)
        .await
        .ok(); // 忽略错误（表可能不存在）
    }

    Ok(CommandResult::ok(DetailedMatchResult {
        match_id,
        tournament_id: tournament_id as u64,
        home_team_id: home_team_id as u64,
        away_team_id: away_team_id as u64,
        home_score,
        away_score,
        winner_id,
        games,
        match_mvp,
        home_team_stats: total_home_stats,
        away_team_stats: total_away_stats,
    }))
}

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

/// 比赛预测结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchPrediction {
    pub match_id: u64,
    pub home_team_id: u64,
    pub home_team_name: String,
    pub home_power: f64,
    pub home_win_probability: f64,
    pub away_team_id: u64,
    pub away_team_name: String,
    pub away_power: f64,
    pub away_win_probability: f64,
    pub predicted_score: String,
}

// ==================== 辅助函数 ====================

/// 获取首发球员
async fn get_starting_players(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
) -> Result<Vec<(u64, String, String, u8)>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id, game_id, position, ability
        FROM players
        WHERE save_id = ? AND team_id = ? AND status = 'Active' AND is_starter = 1
        ORDER BY position
        "#,
    )
    .bind(save_id)
    .bind(team_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let players: Vec<_> = rows
        .iter()
        .map(|r| {
            (
                r.get::<i64, _>("id") as u64,
                r.get::<String, _>("game_id"),
                r.get::<Option<String>, _>("position").unwrap_or_default(),
                r.get::<i64, _>("ability") as u8,
            )
        })
        .collect();

    // 如果首发不足5人，补充板凳
    if players.len() < 5 {
        let bench_rows = sqlx::query(
            r#"
            SELECT id, game_id, position, ability
            FROM players
            WHERE save_id = ? AND team_id = ? AND status = 'Active' AND is_starter = 0
            ORDER BY ability DESC
            LIMIT ?
            "#,
        )
        .bind(save_id)
        .bind(team_id as i64)
        .bind((5 - players.len()) as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut all_players = players;
        for r in bench_rows {
            all_players.push((
                r.get::<i64, _>("id") as u64,
                r.get::<String, _>("game_id"),
                r.get::<Option<String>, _>("position").unwrap_or_default(),
                r.get::<i64, _>("ability") as u8,
            ));
        }
        return Ok(all_players);
    }

    Ok(players)
}

/// 生成球员统计
fn generate_player_stats(
    home_players: &[(u64, String, String, u8)],
    away_players: &[(u64, String, String, u8)],
    home_perf: f64,
    away_perf: f64,
    home_won: bool,
    duration: u32,
    rng: &mut impl Rng,
) -> (Vec<PlayerGameStats>, Vec<PlayerGameStats>) {
    let _perf_diff = home_perf - away_perf;
    let winner_multiplier = if home_won { 1.2 } else { 0.8 };
    let loser_multiplier = if home_won { 0.8 } else { 1.2 };

    let mut generate_stats = |players: &[(u64, String, String, u8)], is_winner: bool, perf: f64| -> Vec<PlayerGameStats> {
        let mult = if is_winner { winner_multiplier } else { loser_multiplier };

        players.iter().map(|(id, name, pos, ability)| {
            let base = (*ability as f64 / 100.0) * mult;
            let kills = (base * (3.0 + rng.gen::<f64>() * 5.0)) as u32;
            let deaths = ((1.0 - base * 0.5) * (2.0 + rng.gen::<f64>() * 4.0)) as u32;
            let assists = (base * (4.0 + rng.gen::<f64>() * 8.0)) as u32;
            let cs = (duration as f64 * (7.0 + rng.gen::<f64>() * 3.0) * base) as u32;
            let gold = cs as u64 * 20 + kills as u64 * 300 + assists as u64 * 150;
            let damage = (perf * 1000.0 * (0.8 + rng.gen::<f64>() * 0.4)) as u64;

            let kda = if deaths > 0 {
                (kills + assists) as f64 / deaths as f64
            } else {
                (kills + assists) as f64
            };

            let mvp_score = kda * 0.4 + (damage as f64 / 10000.0) * 0.3 + (gold as f64 / 10000.0) * 0.3;

            PlayerGameStats {
                player_id: *id,
                player_name: name.clone(),
                position: pos.clone(),
                kills,
                deaths,
                assists,
                cs,
                gold,
                damage_dealt: damage,
                damage_taken: (damage as f64 * 0.8) as u64,
                vision_score: (duration as f64 * (0.5 + rng.gen::<f64>() * 1.5)) as u32,
                mvp_score,
            }
        }).collect()
    };

    let home_stats = generate_stats(home_players, home_won, home_perf);
    let away_stats = generate_stats(away_players, !home_won, away_perf);

    (home_stats, away_stats)
}

/// 选择MVP
fn select_mvp(stats: &[&PlayerGameStats]) -> PlayerMvpInfo {
    let best = stats.iter()
        .max_by(|a, b| a.mvp_score.partial_cmp(&b.mvp_score).unwrap())
        .unwrap();

    PlayerMvpInfo {
        player_id: best.player_id,
        player_name: best.player_name.clone(),
        team_id: 0, // 需要从外部传入
        position: best.position.clone(),
        mvp_score: best.mvp_score,
    }
}

/// 选择比赛MVP
fn select_match_mvp(games: &[DetailedGameResult]) -> Option<PlayerMvpInfo> {
    let mut player_scores: std::collections::HashMap<u64, (String, String, f64, u32)> = std::collections::HashMap::new();

    for game in games {
        for p in &game.home_players {
            let entry = player_scores.entry(p.player_id).or_insert((p.player_name.clone(), p.position.clone(), 0.0, 0));
            entry.2 += p.mvp_score;
            entry.3 += 1;
        }
        for p in &game.away_players {
            let entry = player_scores.entry(p.player_id).or_insert((p.player_name.clone(), p.position.clone(), 0.0, 0));
            entry.2 += p.mvp_score;
            entry.3 += 1;
        }
    }

    player_scores.into_iter()
        .max_by(|a, b| (a.1.2 / a.1.3 as f64).partial_cmp(&(b.1.2 / b.1.3 as f64)).unwrap())
        .map(|(id, (name, pos, score, count))| PlayerMvpInfo {
            player_id: id,
            player_name: name,
            team_id: 0,
            position: pos,
            mvp_score: score / count as f64,
        })
}

/// 生成关键事件
fn generate_key_events(
    home_id: u64,
    away_id: u64,
    home_won: bool,
    duration: u32,
    rng: &mut impl Rng,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    let winner_id = if home_won { home_id } else { away_id };
    let loser_id = if home_won { away_id } else { home_id };

    // 一血
    let fb_time = 2 + rng.gen_range(0..5);
    let fb_team = if rng.gen::<f64>() < 0.6 { winner_id } else { loser_id };
    events.push(GameEvent {
        time_minutes: fb_time,
        event_type: "FirstBlood".to_string(),
        description: "拿下一血".to_string(),
        team_id: fb_team,
    });

    // 一塔
    let ft_time = 8 + rng.gen_range(0..7);
    let ft_team = if rng.gen::<f64>() < 0.65 { winner_id } else { loser_id };
    events.push(GameEvent {
        time_minutes: ft_time,
        event_type: "FirstTower".to_string(),
        description: "推掉一塔".to_string(),
        team_id: ft_team,
    });

    // 龙
    for i in 0..3 {
        let dragon_time = 6 + i * 6 + rng.gen_range(0..4);
        if dragon_time < duration {
            let dragon_team = if rng.gen::<f64>() < 0.55 { winner_id } else { loser_id };
            events.push(GameEvent {
                time_minutes: dragon_time,
                event_type: "Dragon".to_string(),
                description: format!("击杀第{}条龙", i + 1),
                team_id: dragon_team,
            });
        }
    }

    // 大龙
    if duration > 20 {
        let baron_time = 20 + rng.gen_range(0..10);
        if baron_time < duration {
            events.push(GameEvent {
                time_minutes: baron_time,
                event_type: "Baron".to_string(),
                description: "击杀大龙".to_string(),
                team_id: winner_id,
            });
        }
    }

    events.sort_by_key(|e| e.time_minutes);
    events
}

/// 更新队伍统计
fn update_team_stats(
    stats: &mut TeamMatchStats,
    player_stats: &[PlayerGameStats],
    events: &[GameEvent],
    _is_home: bool,
) {
    for p in player_stats {
        stats.total_kills += p.kills;
        stats.total_deaths += p.deaths;
        stats.total_assists += p.assists;
        stats.total_gold += p.gold;
    }

    for e in events {
        if e.team_id == stats.team_id {
            match e.event_type.as_str() {
                "FirstBlood" => stats.first_blood_rate = 1.0,
                "FirstTower" => stats.first_tower_rate = 1.0,
                "Baron" => stats.baron_rate = 1.0,
                "Dragon" => stats.dragon_rate += 0.33,
                _ => {}
            }
        }
    }
}

/// 预测比分
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

impl TeamMatchStats {
    fn default(team_id: u64) -> Self {
        Self {
            team_id,
            total_kills: 0,
            total_deaths: 0,
            total_assists: 0,
            total_gold: 0,
            average_game_duration: 0,
            first_blood_rate: 0.0,
            first_tower_rate: 0.0,
            baron_rate: 0.0,
            dragon_rate: 0.0,
        }
    }
}
