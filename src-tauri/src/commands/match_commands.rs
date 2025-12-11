use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::{MatchSimulationEngine, ConditionEngine, PlayerFormFactors, TraitType, TraitEngine, TraitContext};
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
    pub base_ability: u8,        // 选手基础能力值
    pub condition_bonus: f64,    // 状态加成
    pub stability_noise: f64,    // 稳定性波动
    pub actual_ability: f64,     // 实际发挥 = base + condition + noise
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub cs: u32,
    pub gold: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub vision_score: u32,
    pub mvp_score: f64,
    pub impact_score: f64,       // 影响力分数
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

    // 获取当前赛季
    let current_season: i64 = sqlx::query_scalar(
        "SELECT current_season FROM saves WHERE id = ?"
    )
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or(1);

    // 获取赛事类型用于特性判断
    let tournament_type: String = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_else(|| "league".to_string());

    let format = match format_str.as_str() {
        "Bo1" => MatchFormat::Bo1,
        "Bo3" => MatchFormat::Bo3,
        "Bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    };

    // 获取双方首发球员
    let home_players = get_starting_players(&pool, &save_id, home_team_id as u64, current_season).await?;
    let away_players = get_starting_players(&pool, &save_id, away_team_id as u64, current_season).await?;

    // 模拟比赛 - 核心逻辑：基于选手真实属性决定胜负
    let mut rng = StdRng::from_entropy();

    let wins_needed = format.wins_needed();
    let mut home_score: u8 = 0;
    let mut away_score: u8 = 0;
    let mut games = Vec::new();
    let mut game_number: u8 = 1;

    let mut total_home_stats = TeamMatchStats::default(home_team_id as u64);
    let mut total_away_stats = TeamMatchStats::default(away_team_id as u64);

    while home_score < wins_needed && away_score < wins_needed {
        let duration = 25 + rng.gen_range(0..25); // 25-50分钟

        // 构建特性上下文
        let is_international = matches!(
            tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "clauch"
        );
        let is_playoff = tournament_type.contains("playoff") ||
                         tournament_type == "knockout";

        let trait_ctx = TraitContext {
            tournament_type: tournament_type.clone(),
            is_playoff,
            is_international,
            game_number,
            score_diff: home_score as i8 - away_score as i8,
            age: 0,  // 每个选手单独设置
            is_first_season: false,  // 每个选手单独设置
            games_since_rest: 0,
        };

        // 核心：先计算每个选手的发挥值，返回选手统计和队伍总发挥
        let (home_player_stats, away_player_stats, home_perf, away_perf) = simulate_game_with_players(
            &home_players, &away_players,
            duration,
            &trait_ctx,
            &mut rng,
        );

        // 根据队伍发挥战力决定胜负（加入正态分布增加不确定性）
        // 使用 Box-Muller 变换生成高斯随机数
        let game_std_dev = 3.0; // 局内波动标准差
        let u1: f64 = rng.gen::<f64>().max(0.0001);
        let u2: f64 = rng.gen::<f64>().max(0.0001);
        let gaussian = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

        // 双方发挥战力差值 + 随机波动
        let performance_diff = home_perf - away_perf;
        let final_diff = performance_diff + gaussian * game_std_dev;

        let winner_id = if final_diff > 0.0 {
            home_team_id as u64
        } else {
            away_team_id as u64
        };

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

    // 更新积分榜
    // 积分规则（BO3）：2:0胜积3分、2:1胜积2分、2:1负积1分、2:0负积0分
    let loser_id = if winner_id == home_team_id as u64 {
        away_team_id as u64
    } else {
        home_team_id as u64
    };

    let (winner_games_won, winner_games_lost) = if winner_id == home_team_id as u64 {
        (home_score as i32, away_score as i32)
    } else {
        (away_score as i32, home_score as i32)
    };

    let (loser_games_won, loser_games_lost) = (winner_games_lost, winner_games_won);

    // 根据比分计算积分
    // 2:0胜 → 胜方3分，负方0分
    // 2:1胜 → 胜方2分，负方1分
    let (winner_points, loser_points) = if winner_games_lost == 0 {
        (3, 0)  // 2:0
    } else {
        (2, 1)  // 2:1
    };

    // 更新胜方积分榜
    sqlx::query(
        r#"
        UPDATE league_standings
        SET matches_played = matches_played + 1,
            wins = wins + 1,
            points = points + ?,
            games_won = games_won + ?,
            games_lost = games_lost + ?,
            game_diff = game_diff + ?
        WHERE tournament_id = ? AND team_id = ?
        "#
    )
    .bind(winner_points as i64)
    .bind(winner_games_won as i64)
    .bind(winner_games_lost as i64)
    .bind((winner_games_won - winner_games_lost) as i64)
    .bind(tournament_id)
    .bind(winner_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 更新负方积分榜
    sqlx::query(
        r#"
        UPDATE league_standings
        SET matches_played = matches_played + 1,
            losses = losses + 1,
            points = points + ?,
            games_won = games_won + ?,
            games_lost = games_lost + ?,
            game_diff = game_diff + ?
        WHERE tournament_id = ? AND team_id = ?
        "#
    )
    .bind(loser_points as i64)
    .bind(loser_games_won as i64)
    .bind(loser_games_lost as i64)
    .bind((loser_games_won - loser_games_lost) as i64)
    .bind(tournament_id)
    .bind(loser_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 重新计算排名 (按积分降序, 然后按净胜场降序)
    sqlx::query(
        r#"
        WITH ranked AS (
            SELECT team_id,
                   ROW_NUMBER() OVER (ORDER BY points DESC, game_diff DESC, wins DESC) as new_rank
            FROM league_standings
            WHERE tournament_id = ?
        )
        UPDATE league_standings
        SET rank = (SELECT new_rank FROM ranked WHERE ranked.team_id = league_standings.team_id)
        WHERE tournament_id = ?
        "#
    )
    .bind(tournament_id)
    .bind(tournament_id)
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

    // 更新选手状态因子（比赛后动态调整）
    let home_won = winner_id == home_team_id as u64;
    let home_avg_perf = if !games.is_empty() {
        games.iter().map(|g| g.home_performance).sum::<f64>() / games.len() as f64
    } else {
        0.0
    };
    let away_avg_perf = if !games.is_empty() {
        games.iter().map(|g| g.away_performance).sum::<f64>() / games.len() as f64
    } else {
        0.0
    };

    update_player_form_factors(&pool, &home_players, home_won, home_avg_perf).await.ok();
    update_player_form_factors(&pool, &away_players, !home_won, away_avg_perf).await.ok();

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

/// 选手数据结构（包含能力值和稳定性）
#[derive(Debug, Clone)]
struct PlayerData {
    id: u64,
    game_id: String,
    position: String,
    ability: u8,
    age: u8,
    stability: u8,
    condition: i8,
    form_factors: PlayerFormFactors,
    traits: Vec<TraitType>,
    is_first_season: bool,
}

/// 获取首发球员（包含 ability, stability, condition, traits）
async fn get_starting_players(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
    current_season: i64,
) -> Result<Vec<PlayerData>, String> {
    let rows = sqlx::query(
        r#"
        SELECT p.id, p.game_id, p.position, p.ability, p.age, p.stability, p.join_season,
               pff.form_cycle, pff.momentum, pff.last_performance, pff.last_match_won
        FROM players p
        LEFT JOIN player_form_factors pff ON p.id = pff.player_id
        WHERE p.save_id = ? AND p.team_id = ? AND p.status = 'Active' AND p.is_starter = 1
        ORDER BY p.position
        "#,
    )
    .bind(save_id)
    .bind(team_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut players: Vec<PlayerData> = Vec::new();

    for r in &rows {
        let player_id = r.get::<i64, _>("id") as u64;
        let age = r.get::<i64, _>("age") as u8;
        let ability = r.get::<i64, _>("ability") as u8;
        let join_season = r.get::<Option<i64>, _>("join_season").unwrap_or(1);
        let is_first_season = join_season == current_season;

        // 获取状态因子，如果不存在则使用默认值
        let form_factors = PlayerFormFactors {
            player_id,
            form_cycle: r.get::<Option<f64>, _>("form_cycle").unwrap_or(50.0),
            momentum: r.get::<Option<i64>, _>("momentum").unwrap_or(0) as i8,
            last_performance: r.get::<Option<f64>, _>("last_performance").unwrap_or(0.0),
            last_match_won: r.get::<Option<i64>, _>("last_match_won").unwrap_or(0) == 1,
            games_since_rest: 0,
        };

        // 加载选手特性
        let traits = load_player_traits(pool, player_id).await?;

        // 使用 ConditionEngine 计算动态 condition
        let condition = ConditionEngine::calculate_condition(
            age,
            ability,
            &form_factors,
            None,  // 比赛情境在模拟时传入
        );

        players.push(PlayerData {
            id: player_id,
            game_id: r.get::<String, _>("game_id"),
            position: r.get::<Option<String>, _>("position").unwrap_or_default(),
            ability,
            age,
            stability: r.get::<Option<i64>, _>("stability").unwrap_or(70) as u8,
            condition,
            form_factors,
            traits,
            is_first_season,
        });
    }

    // 如果首发不足5人，补充板凳
    if players.len() < 5 {
        let bench_rows = sqlx::query(
            r#"
            SELECT p.id, p.game_id, p.position, p.ability, p.age, p.stability, p.join_season,
                   pff.form_cycle, pff.momentum, pff.last_performance, pff.last_match_won
            FROM players p
            LEFT JOIN player_form_factors pff ON p.id = pff.player_id
            WHERE p.save_id = ? AND p.team_id = ? AND p.status = 'Active' AND p.is_starter = 0
            ORDER BY p.ability DESC
            LIMIT ?
            "#,
        )
        .bind(save_id)
        .bind(team_id as i64)
        .bind((5 - players.len()) as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for r in bench_rows {
            let player_id = r.get::<i64, _>("id") as u64;
            let age = r.get::<i64, _>("age") as u8;
            let ability = r.get::<i64, _>("ability") as u8;
            let join_season = r.get::<Option<i64>, _>("join_season").unwrap_or(1);
            let is_first_season = join_season == current_season;

            let form_factors = PlayerFormFactors {
                player_id,
                form_cycle: r.get::<Option<f64>, _>("form_cycle").unwrap_or(50.0),
                momentum: r.get::<Option<i64>, _>("momentum").unwrap_or(0) as i8,
                last_performance: r.get::<Option<f64>, _>("last_performance").unwrap_or(0.0),
                last_match_won: r.get::<Option<i64>, _>("last_match_won").unwrap_or(0) == 1,
                games_since_rest: 0,
            };

            let traits = load_player_traits(pool, player_id).await?;

            let condition = ConditionEngine::calculate_condition(
                age,
                ability,
                &form_factors,
                None,
            );

            players.push(PlayerData {
                id: player_id,
                game_id: r.get::<String, _>("game_id"),
                position: r.get::<Option<String>, _>("position").unwrap_or_default(),
                ability,
                age,
                stability: r.get::<Option<i64>, _>("stability").unwrap_or(70) as u8,
                condition,
                form_factors,
                traits,
                is_first_season,
            });
        }
    }

    Ok(players)
}

/// 加载选手特性
async fn load_player_traits(
    pool: &sqlx::SqlitePool,
    player_id: u64,
) -> Result<Vec<TraitType>, String> {
    let rows = sqlx::query(
        "SELECT trait_type FROM player_traits WHERE player_id = ?"
    )
    .bind(player_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut traits = Vec::new();
    for r in rows {
        let trait_str: String = r.get("trait_type");
        if let Some(trait_type) = parse_trait_type(&trait_str) {
            traits.push(trait_type);
        }
    }
    Ok(traits)
}

/// 解析特性类型字符串
fn parse_trait_type(s: &str) -> Option<TraitType> {
    match s.to_lowercase().as_str() {
        "clutch" => Some(TraitType::Clutch),
        "slow_starter" | "slowstarter" => Some(TraitType::SlowStarter),
        "fast_starter" | "faststarter" => Some(TraitType::FastStarter),
        "explosive" => Some(TraitType::Explosive),
        "consistent" => Some(TraitType::Consistent),
        "comeback_king" | "comebackking" => Some(TraitType::ComebackKing),
        "tilter" => Some(TraitType::Tilter),
        "mental_fortress" | "mentalfortress" => Some(TraitType::MentalFortress),
        "fragile" => Some(TraitType::Fragile),
        "ironman" => Some(TraitType::Ironman),
        "volatile" => Some(TraitType::Volatile),
        "rising_star" | "risingstar" => Some(TraitType::RisingStar),
        "veteran" => Some(TraitType::Veteran),
        "team_leader" | "teamleader" => Some(TraitType::TeamLeader),
        _ => None,
    }
}

/// 核心比赛模拟函数：基于选手真实属性模拟单局比赛
/// 返回: (主队选手统计, 客队选手统计, 主队总发挥, 客队总发挥)
fn simulate_game_with_players(
    home_players: &[PlayerData],
    away_players: &[PlayerData],
    duration: u32,
    trait_ctx: &TraitContext,
    rng: &mut impl Rng,
) -> (Vec<PlayerGameStats>, Vec<PlayerGameStats>, f64, f64) {
    // 使用 Box-Muller 变换生成高斯随机数
    fn gaussian_random(rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.gen::<f64>().max(0.0001);
        let v: f64 = rng.gen::<f64>().max(0.0001);
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    // 生成单队统计，返回选手统计和队伍总发挥值
    fn generate_team_stats(
        players: &[PlayerData],
        duration: u32,
        trait_ctx: &TraitContext,
        rng: &mut impl Rng
    ) -> (Vec<PlayerGameStats>, f64) {
        let mut stats = Vec::new();
        let mut total_actual_ability = 0.0;

        // 第一遍：计算每个选手的发挥值（应用特性修正）
        let mut player_performances: Vec<(f64, f64, f64, f64)> = Vec::new();
        for player in players {
            // 构建选手专属的特性上下文
            let player_trait_ctx = TraitContext {
                age: player.age,
                is_first_season: player.is_first_season,
                games_since_rest: player.form_factors.games_since_rest,
                ..trait_ctx.clone()
            };

            // 计算特性修正
            let modifiers = TraitEngine::calculate_combined_modifiers(&player.traits, &player_trait_ctx);

            // 应用特性修正到基础属性
            let (modified_ability, modified_stability, modified_condition, ability_ceiling) =
                TraitEngine::apply_modifiers(
                    player.ability,
                    player.stability,
                    player.condition,
                    &modifiers,
                );

            // 稳定性标准差: σ = (100 - stability) / 10
            let sigma = (100.0 - modified_stability as f64) / 10.0;

            // 状态加成（应用特性修正后的 condition）
            let condition_bonus = modified_condition as f64;

            // 稳定性波动（高斯噪声）
            let stability_noise = gaussian_random(rng) * sigma;

            // 原始实际能力
            let raw_ability = modified_ability as f64 + condition_bonus + stability_noise;

            // 钳位到合理范围，考虑特性修正的能力上限
            let min_ability = (modified_ability as f64 - 15.0).max(0.0);
            let max_ability = (ability_ceiling as f64).min(100.0);
            let actual_ability = raw_ability.clamp(min_ability, max_ability);

            total_actual_ability += actual_ability;
            player_performances.push((player.ability as f64, condition_bonus, stability_noise, actual_ability));
        }

        let team_avg = if !players.is_empty() { total_actual_ability / players.len() as f64 } else { 0.0 };

        // 第二遍：生成详细统计
        for (i, player) in players.iter().enumerate() {
            let (base_ability, condition_bonus, stability_noise, actual_ability) = player_performances[i];

            // 根据发挥值生成KDA等统计
            let base = actual_ability / 100.0;
            let kills = (base * (3.0 + rng.gen::<f64>() * 5.0)) as u32;
            let deaths = ((1.0 - base * 0.5) * (2.0 + rng.gen::<f64>() * 4.0)) as u32;
            let assists = (base * (4.0 + rng.gen::<f64>() * 8.0)) as u32;
            let cs = (duration as f64 * (7.0 + rng.gen::<f64>() * 3.0) * base) as u32;
            let gold = cs as u64 * 20 + kills as u64 * 300 + assists as u64 * 150;
            let damage = (actual_ability * 1000.0 * (0.8 + rng.gen::<f64>() * 0.4)) as u64;

            let kda = if deaths > 0 {
                (kills + assists) as f64 / deaths as f64
            } else {
                (kills + assists) as f64
            };

            let mvp_score = kda * 0.4 + (damage as f64 / 10000.0) * 0.3 + (gold as f64 / 10000.0) * 0.3;

            // 影响力分数：相对于队伍平均值的偏差
            let impact_score = ((actual_ability - team_avg) * 10.0).round() / 10.0;

            stats.push(PlayerGameStats {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position.clone(),
                base_ability: base_ability as u8,
                condition_bonus: (condition_bonus * 10.0).round() / 10.0,
                stability_noise: (stability_noise * 100.0).round() / 100.0,
                actual_ability: (actual_ability * 10.0).round() / 10.0,
                kills,
                deaths,
                assists,
                cs,
                gold,
                damage_dealt: damage,
                damage_taken: (damage as f64 * 0.8) as u64,
                vision_score: (duration as f64 * (0.5 + rng.gen::<f64>() * 1.5)) as u32,
                mvp_score,
                impact_score,
            });
        }

        (stats, team_avg)
    }

    // 生成双方统计
    let (home_stats, home_avg) = generate_team_stats(home_players, duration, trait_ctx, rng);
    let (away_stats, away_avg) = generate_team_stats(away_players, duration, trait_ctx, rng);

    (home_stats, away_stats, home_avg, away_avg)
}

/// 生成球员统计（保留兼容性，但不再使用）
#[allow(dead_code)]
fn generate_player_stats(
    home_players: &[PlayerData],
    away_players: &[PlayerData],
    _home_perf: f64,  // 保留参数兼容性，但不再使用
    _away_perf: f64,
    home_won: bool,
    duration: u32,
    rng: &mut impl Rng,
) -> (Vec<PlayerGameStats>, Vec<PlayerGameStats>) {
    // 使用 Box-Muller 变换生成高斯随机数
    fn gaussian_random(rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.gen::<f64>().max(0.0001);
        let v: f64 = rng.gen::<f64>().max(0.0001);
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    // 计算单个选手的实际发挥
    fn calculate_player_performance(player: &PlayerData, rng: &mut impl Rng) -> (f64, f64, f64, f64) {
        // 稳定性标准差: σ = (100 - stability) / 10
        let sigma = (100.0 - player.stability as f64) / 10.0;

        // 状态加成（使用数据库中存储的 condition）
        let condition_bonus = player.condition as f64;

        // 稳定性波动（高斯噪声）
        let stability_noise = gaussian_random(rng) * sigma;

        // 原始实际能力
        let raw_ability = player.ability as f64 + condition_bonus + stability_noise;

        // 钳位到合理范围 [ability - 15, ability + 10]
        let min_ability = (player.ability as f64 - 15.0).max(0.0);
        let max_ability = (player.ability as f64 + 10.0).min(100.0);
        let actual_ability = raw_ability.clamp(min_ability, max_ability);

        (player.ability as f64, condition_bonus, stability_noise, actual_ability)
    }

    // 生成单队统计
    fn generate_team_stats(
        players: &[PlayerData],
        is_winner: bool,
        duration: u32,
        rng: &mut impl Rng
    ) -> Vec<PlayerGameStats> {
        let winner_multiplier = if is_winner { 1.2 } else { 0.8 };

        let mut stats = Vec::new();
        let mut total_actual_ability = 0.0;

        for player in players {
            let (base_ability, condition_bonus, stability_noise, actual_ability) = calculate_player_performance(player, rng);
            total_actual_ability += actual_ability;

            let base = (actual_ability / 100.0) * winner_multiplier;
            let kills = (base * (3.0 + rng.gen::<f64>() * 5.0)) as u32;
            let deaths = ((1.0 - base * 0.5) * (2.0 + rng.gen::<f64>() * 4.0)) as u32;
            let assists = (base * (4.0 + rng.gen::<f64>() * 8.0)) as u32;
            let cs = (duration as f64 * (7.0 + rng.gen::<f64>() * 3.0) * base) as u32;
            let gold = cs as u64 * 20 + kills as u64 * 300 + assists as u64 * 150;
            let damage = (actual_ability * 1000.0 * (0.8 + rng.gen::<f64>() * 0.4)) as u64;

            let kda = if deaths > 0 {
                (kills + assists) as f64 / deaths as f64
            } else {
                (kills + assists) as f64
            };

            let mvp_score = kda * 0.4 + (damage as f64 / 10000.0) * 0.3 + (gold as f64 / 10000.0) * 0.3;

            stats.push(PlayerGameStats {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position.clone(),
                base_ability: base_ability as u8,
                condition_bonus: (condition_bonus * 10.0).round() / 10.0,
                stability_noise: (stability_noise * 100.0).round() / 100.0,
                actual_ability: (actual_ability * 10.0).round() / 10.0,
                kills,
                deaths,
                assists,
                cs,
                gold,
                damage_dealt: damage,
                damage_taken: (damage as f64 * 0.8) as u64,
                vision_score: (duration as f64 * (0.5 + rng.gen::<f64>() * 1.5)) as u32,
                mvp_score,
                impact_score: 0.0, // 稍后计算
            });
        }

        let team_avg = if !players.is_empty() { total_actual_ability / players.len() as f64 } else { 0.0 };

        // 计算影响力分数（相对于队伍平均）
        for stat in &mut stats {
            stat.impact_score = ((stat.actual_ability - team_avg) * 10.0).round() / 10.0;
        }

        stats
    }

    let home_stats = generate_team_stats(home_players, home_won, duration, rng);
    let away_stats = generate_team_stats(away_players, !home_won, duration, rng);

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

/// 比赛结束后更新选手状态因子
///
/// # 参数
/// - `pool`: 数据库连接池
/// - `players`: 参赛选手列表
/// - `won`: 队伍是否获胜
/// - `avg_performance`: 队伍平均发挥值
async fn update_player_form_factors(
    pool: &sqlx::SqlitePool,
    players: &[PlayerData],
    won: bool,
    avg_performance: f64,
) -> Result<(), String> {
    for player in players {
        // 使用 ConditionEngine 更新状态因子
        let updated_factors = ConditionEngine::update_form_factors(
            player.form_factors.clone(),
            won,
            avg_performance,
        );

        // 更新或插入到数据库
        sqlx::query(
            r#"
            INSERT INTO player_form_factors (player_id, form_cycle, momentum, last_performance, last_match_won, games_since_rest, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(player_id) DO UPDATE SET
                form_cycle = excluded.form_cycle,
                momentum = excluded.momentum,
                last_performance = excluded.last_performance,
                last_match_won = excluded.last_match_won,
                games_since_rest = excluded.games_since_rest,
                updated_at = datetime('now')
            "#,
        )
        .bind(player.id as i64)
        .bind(updated_factors.form_cycle)
        .bind(updated_factors.momentum as i64)
        .bind(updated_factors.last_performance)
        .bind(if updated_factors.last_match_won { 1i64 } else { 0i64 })
        .bind(updated_factors.games_since_rest as i64)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
