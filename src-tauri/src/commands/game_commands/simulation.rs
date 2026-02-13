use crate::commands::save_commands::{AppState, CommandResult};
use crate::commands::match_commands::{PlayerGameStats, ActivatedTraitInfo};
use crate::db::{MatchRepository, MatchGameDetailRepository, TournamentRepository, PlayerTournamentStatsRepository};
use crate::db::repository::PlayerStatsRepository;
use crate::models::PlayerTournamentStats;
use crate::models::{TournamentStatus, MatchFormat};
use crate::engines::{ConditionEngine, PlayerFormFactors, TraitType, TraitEngine, TraitContext};
use crate::models::match_game_detail::{SaveMatchDetailsInput, SaveGameInput, SavePerformanceInput};
use crate::services::LeagueService;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use sqlx::Row;
use tauri::State;

use super::MatchInfo;

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

fn parse_position_string(pos: &str) -> String {
    let pos = pos.trim();
    if pos.starts_with("Some(") && pos.ends_with(")") {
        let inner = &pos[5..pos.len()-1];
        return inner.to_uppercase();
    }
    match pos.to_lowercase().as_str() {
        "top" => "TOP".to_string(),
        "jug" | "jungle" => "JUG".to_string(),
        "mid" | "middle" => "MID".to_string(),
        "adc" | "bot" | "bottom" => "ADC".to_string(),
        "sup" | "support" => "SUP".to_string(),
        _ => pos.to_uppercase(),
    }
}

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

        let raw_position = r.get::<Option<String>, _>("position").unwrap_or_default();
        players.push(PlayerData {
            id: player_id,
            game_id: r.get::<String, _>("game_id"),
            position: parse_position_string(&raw_position),
            ability,
            age,
            stability: r.get::<Option<i64>, _>("stability").unwrap_or(70) as u8,
            condition,
            form_factors,
            traits,
            is_first_season,
        });
    }

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

            let raw_position = r.get::<Option<String>, _>("position").unwrap_or_default();
            players.push(PlayerData {
                id: player_id,
                game_id: r.get::<String, _>("game_id"),
                position: parse_position_string(&raw_position),
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

#[tauri::command]
pub async fn simulate_next_match(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<MatchInfo>, String> {
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

    let pending = match MatchRepository::get_pending(&pool, &save_id, tournament_id).await {
        Ok(m) => m,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pending matches: {}", e))),
    };

    if pending.is_empty() {
        return Ok(CommandResult::err("No pending matches"));
    }

    let match_info = &pending[0];

    match simulate_match_core(&pool, &save_id, match_info.id).await {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to simulate match: {}", e))),
    }
}

#[tauri::command]
pub async fn simulate_all_matches(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<u32>, String> {
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

    let mut simulated_count = 0u32;

    loop {
        let pending = match MatchRepository::get_pending(&pool, &save_id, tournament_id).await {
            Ok(m) => m,
            Err(e) => return Ok(CommandResult::err(format!("Failed to get pending: {}", e))),
        };

        if pending.is_empty() {
            break;
        }

        let match_info = &pending[0];

        if simulate_match_core(&pool, &save_id, match_info.id).await.is_ok() {
            simulated_count += 1;
        }
    }

    if let Err(e) = TournamentRepository::update_status(&pool, tournament_id, TournamentStatus::Completed).await {
        return Ok(CommandResult::err(format!("Failed to update tournament: {}", e)));
    }

    Ok(CommandResult::ok(simulated_count))
}

fn simulate_game_with_players(
    home_players: &[PlayerData],
    away_players: &[PlayerData],
    duration: u32,
    trait_ctx: &TraitContext,
    rng: &mut impl Rng,
) -> (Vec<PlayerGameStats>, Vec<PlayerGameStats>, f64, f64) {
    fn gaussian_random(rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.gen::<f64>().max(0.0001);
        let v: f64 = rng.gen::<f64>().max(0.0001);
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    fn generate_team_stats(
        players: &[PlayerData],
        duration: u32,
        trait_ctx: &TraitContext,
        rng: &mut impl Rng
    ) -> (Vec<PlayerGameStats>, f64) {
        let mut stats = Vec::new();
        let mut total_actual_ability = 0.0;

        let mut player_performances: Vec<(f64, f64, f64, f64, Vec<ActivatedTraitInfo>)> = Vec::new();
        for player in players {
            let player_trait_ctx = TraitContext {
                age: player.age,
                is_first_season: player.is_first_season,
                games_since_rest: player.form_factors.games_since_rest,
                ..trait_ctx.clone()
            };

            let modifiers = TraitEngine::calculate_combined_modifiers(&player.traits, &player_trait_ctx);

            let activated_traits: Vec<ActivatedTraitInfo> = player.traits.iter().filter_map(|t| {
                let (effect_desc, value, is_positive) = match t {
                    TraitType::Clutch if player_trait_ctx.is_playoff || player_trait_ctx.is_international => {
                        ("状态 +3".to_string(), 3.0, true)
                    }
                    TraitType::SlowStarter if player_trait_ctx.game_number == 1 => {
                        ("状态 -2".to_string(), -2.0, false)
                    }
                    TraitType::SlowStarter if player_trait_ctx.game_number >= 3 => {
                        ("状态 +2".to_string(), 2.0, true)
                    }
                    TraitType::FastStarter if player_trait_ctx.game_number == 1 => {
                        ("状态 +2".to_string(), 2.0, true)
                    }
                    TraitType::FastStarter if player_trait_ctx.game_number >= 3 => {
                        ("状态 -1".to_string(), -1.0, false)
                    }
                    TraitType::Explosive => {
                        ("稳定性 -15, 上限 +5".to_string(), 5.0, true)
                    }
                    TraitType::Consistent => {
                        ("稳定性 +10, 上限 -3".to_string(), 10.0, true)
                    }
                    TraitType::ComebackKing if player_trait_ctx.score_diff < 0 => {
                        ("状态 +3".to_string(), 3.0, true)
                    }
                    TraitType::Tilter if player_trait_ctx.score_diff > 0 => {
                        ("状态 -2".to_string(), -2.0, false)
                    }
                    TraitType::Tilter if player_trait_ctx.score_diff < 0 => {
                        ("状态 -3".to_string(), -3.0, false)
                    }
                    TraitType::RisingStar if player.is_first_season => {
                        ("能力 +3".to_string(), 3.0, true)
                    }
                    TraitType::Veteran if player.age >= 30 => {
                        ("稳定性 +15".to_string(), 15.0, true)
                    }
                    _ => return None,
                };
                Some(ActivatedTraitInfo {
                    trait_type: format!("{:?}", t),
                    name: t.display_name().to_string(),
                    effect: effect_desc,
                    value,
                    is_positive,
                })
            }).collect();

            let (modified_ability, modified_stability, modified_condition, ability_ceiling) =
                TraitEngine::apply_modifiers(
                    player.ability,
                    player.stability,
                    player.condition,
                    &modifiers,
                );

            let sigma = (100.0 - modified_stability as f64) / 10.0;
            let condition_bonus = modified_condition as f64;
            let stability_noise = gaussian_random(rng) * sigma;
            let raw_ability = modified_ability as f64 + condition_bonus + stability_noise;
            let min_ability = (modified_ability as f64 - 15.0).max(0.0);
            let max_ability = (ability_ceiling as f64).min(100.0);
            let actual_ability = raw_ability.clamp(min_ability, max_ability);

            total_actual_ability += actual_ability;
            player_performances.push((player.ability as f64, condition_bonus, stability_noise, actual_ability, activated_traits));
        }

        let team_avg = if !players.is_empty() { total_actual_ability / players.len() as f64 } else { 0.0 };

        for (i, player) in players.iter().enumerate() {
            let (base_ability, condition_bonus, stability_noise, actual_ability, ref activated_traits) = player_performances[i];

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
                traits: player.traits.iter().map(|t| format!("{:?}", t)).collect(),
                activated_traits: activated_traits.clone(),
            });
        }

        (stats, team_avg)
    }

    let (home_stats, home_avg) = generate_team_stats(home_players, duration, trait_ctx, rng);
    let (away_stats, away_avg) = generate_team_stats(away_players, duration, trait_ctx, rng);

    (home_stats, away_stats, home_avg, away_avg)
}

async fn simulate_match_core(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    match_id: u64,
) -> Result<MatchInfo, String> {
    let match_row = sqlx::query(
        r#"
        SELECT m.id, m.tournament_id, m.format, m.home_team_id, m.away_team_id, m.stage, m.round,
               ht.name as home_name, at.name as away_name
        FROM matches m
        JOIN teams ht ON m.home_team_id = ht.id
        JOIN teams at ON m.away_team_id = at.id
        WHERE m.id = ?
        "#,
    )
    .bind(match_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_row = match match_row {
        Some(r) => r,
        None => return Err("Match not found".to_string()),
    };

    let tournament_id: i64 = match_row.get("tournament_id");
    let home_team_id: i64 = match_row.get("home_team_id");
    let away_team_id: i64 = match_row.get("away_team_id");
    let format_str: String = match_row.get("format");
    let stage: String = match_row.get("stage");
    let round: Option<i64> = match_row.get("round");
    let home_name: String = match_row.get("home_name");
    let away_name: String = match_row.get("away_name");

    let current_season: i64 = sqlx::query_scalar(
        "SELECT current_season FROM saves WHERE id = ?"
    )
    .bind(save_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or(1);

    let tournament_type: String = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_else(|| "league".to_string());

    let format = match format_str.as_str() {
        "Bo1" => MatchFormat::Bo1,
        "Bo3" => MatchFormat::Bo3,
        "Bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    };

    let home_players = get_starting_players(pool, save_id, home_team_id as u64, current_season).await?;
    let away_players = get_starting_players(pool, save_id, away_team_id as u64, current_season).await?;

    let mut rng = StdRng::from_entropy();

    let wins_needed = format.wins_needed();
    let mut home_score: u8 = 0;
    let mut away_score: u8 = 0;
    let mut games_data: Vec<SaveGameInput> = Vec::new();
    let mut game_number: u8 = 1;

    while home_score < wins_needed && away_score < wins_needed {
        let duration = 25 + rng.gen_range(0..25);

        let is_international = matches!(
            tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "clauch" | "Msi" | "WorldChampionship" | "MadridMasters" | "ShanghaiMasters"
        );
        let is_playoff = tournament_type.contains("playoff") ||
                         tournament_type.contains("Playoff") ||
                         tournament_type == "knockout";

        let trait_ctx = TraitContext {
            tournament_type: tournament_type.clone(),
            is_playoff,
            is_international,
            game_number,
            score_diff: home_score as i8 - away_score as i8,
            age: 0,
            is_first_season: false,
            games_since_rest: 0,
        };

        let (home_player_stats, away_player_stats, home_perf, away_perf) = simulate_game_with_players(
            &home_players, &away_players,
            duration,
            &trait_ctx,
            &mut rng,
        );

        let game_std_dev = 3.0;
        let u1: f64 = rng.gen::<f64>().max(0.0001);
        let u2: f64 = rng.gen::<f64>().max(0.0001);
        let gaussian = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let performance_diff = home_perf - away_perf;
        let final_diff = performance_diff + gaussian * game_std_dev;

        let winner_id = if final_diff > 0.0 {
            home_team_id
        } else {
            away_team_id
        };

        let loser_id = if winner_id == home_team_id { away_team_id } else { home_team_id };

        let all_stats: Vec<&PlayerGameStats> = home_player_stats.iter()
            .chain(away_player_stats.iter())
            .collect();
        let mvp = all_stats.iter()
            .max_by(|a, b| a.mvp_score.partial_cmp(&b.mvp_score).unwrap())
            .map(|p| p.player_id as i64);

        let mut performances = Vec::new();

        for p in &home_player_stats {
            performances.push(SavePerformanceInput {
                player_id: p.player_id as i64,
                player_name: p.player_name.clone(),
                team_id: home_team_id,
                team_name: home_name.clone(),
                position: p.position.clone(),
                base_ability: p.base_ability as f64,
                condition_bonus: p.condition_bonus,
                stability_noise: p.stability_noise,
                actual_ability: p.actual_ability,
                impact_score: p.impact_score,
                mvp_score: p.mvp_score,
                is_mvp: mvp == Some(p.player_id as i64),
                is_key_player: false,
                kills: Some(p.kills as i32),
                deaths: Some(p.deaths as i32),
                assists: Some(p.assists as i32),
                cs: Some(p.cs as i32),
                gold: Some(p.gold as i32),
                damage_dealt: Some(p.damage_dealt as i32),
                damage_taken: Some(p.damage_taken as i32),
                vision_score: Some(p.vision_score as i32),
                traits_json: Some(serde_json::to_string(&p.traits).unwrap_or_default()),
                activated_traits_json: Some(serde_json::to_string(&p.activated_traits).unwrap_or_default()),
            });
        }

        for p in &away_player_stats {
            performances.push(SavePerformanceInput {
                player_id: p.player_id as i64,
                player_name: p.player_name.clone(),
                team_id: away_team_id,
                team_name: away_name.clone(),
                position: p.position.clone(),
                base_ability: p.base_ability as f64,
                condition_bonus: p.condition_bonus,
                stability_noise: p.stability_noise,
                actual_ability: p.actual_ability,
                impact_score: p.impact_score,
                mvp_score: p.mvp_score,
                is_mvp: mvp == Some(p.player_id as i64),
                is_key_player: false,
                kills: Some(p.kills as i32),
                deaths: Some(p.deaths as i32),
                assists: Some(p.assists as i32),
                cs: Some(p.cs as i32),
                gold: Some(p.gold as i32),
                damage_dealt: Some(p.damage_dealt as i32),
                damage_taken: Some(p.damage_taken as i32),
                vision_score: Some(p.vision_score as i32),
                traits_json: Some(serde_json::to_string(&p.traits).unwrap_or_default()),
                activated_traits_json: Some(serde_json::to_string(&p.activated_traits).unwrap_or_default()),
            });
        }

        let home_power: f64 = if !home_player_stats.is_empty() {
            home_player_stats.iter().map(|p| p.actual_ability).sum::<f64>() / home_player_stats.len() as f64
        } else { 0.0 };
        let away_power: f64 = if !away_player_stats.is_empty() {
            away_player_stats.iter().map(|p| p.actual_ability).sum::<f64>() / away_player_stats.len() as f64
        } else { 0.0 };

        games_data.push(SaveGameInput {
            game_number: game_number as i32,
            winner_team_id: winner_id,
            loser_team_id: loser_id,
            duration_minutes: Some(duration as i32),
            mvp_player_id: mvp,
            key_player_id: None,
            home_power: Some(home_power),
            away_power: Some(away_power),
            home_meta_power: Some(home_perf),
            away_meta_power: Some(away_perf),
            performances,
        });

        if winner_id == home_team_id {
            home_score += 1;
        } else {
            away_score += 1;
        }

        game_number += 1;
    }

    let final_winner_id = if home_score > away_score {
        home_team_id
    } else {
        away_team_id
    };

    sqlx::query(
        "UPDATE matches SET home_score = ?, away_score = ?, winner_id = ?, status = 'COMPLETED' WHERE id = ?"
    )
    .bind(home_score as i64)
    .bind(away_score as i64)
    .bind(final_winner_id)
    .bind(match_id as i64)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let loser_id = if final_winner_id == home_team_id {
        away_team_id
    } else {
        home_team_id
    };

    let (winner_games_won, winner_games_lost) = if final_winner_id == home_team_id {
        (home_score as i32, away_score as i32)
    } else {
        (away_score as i32, home_score as i32)
    };

    let (loser_games_won, loser_games_lost) = (winner_games_lost, winner_games_won);

    let (winner_points, loser_points) = if winner_games_lost == 0 {
        (3, 0)
    } else {
        (2, 1)
    };

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
    .bind(final_winner_id)
    .execute(pool)
    .await
    .ok();

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
    .bind(loser_id)
    .execute(pool)
    .await
    .ok();

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
    .execute(pool)
    .await
    .ok();

    let save_input = SaveMatchDetailsInput {
        match_id: match_id as i64,
        games: games_data,
    };

    MatchGameDetailRepository::save_match_details(pool, save_id, &save_input)
        .await
        .map_err(|e| e.to_string())?;

    for game_input in &save_input.games {
        for perf in &game_input.performances {
            if let Ok(mut stats) = PlayerStatsRepository::get_or_create(
                pool,
                save_id,
                perf.player_id,
                &perf.player_name,
                current_season,
                Some(perf.team_id),
                None,
                &perf.position,
            ).await {
                stats.record_game(perf.impact_score, perf.actual_ability);
                PlayerStatsRepository::update(pool, &stats).await.ok();
            }
        }
    }

    let _ = save_player_tournament_stats_from_match(
        pool,
        save_id,
        current_season as u64,
        tournament_id as u64,
        &tournament_type,
        home_team_id as u64,
        &home_name,
        away_team_id as u64,
        &away_name,
        home_score,
        away_score,
        final_winner_id as u64,
        &home_players,
        &away_players,
        &save_input.games,
    ).await;

    let home_won = final_winner_id == home_team_id;
    update_player_form_factors_internal(pool, save_id, &home_players, home_won).await.ok();
    update_player_form_factors_internal(pool, save_id, &away_players, !home_won).await.ok();

    let is_playoff = stage.contains("WINNERS")
        || stage.contains("LOSERS")
        || stage.contains("GRAND_FINAL");

    log::debug!("[Playoffs Debug] stage={}, is_playoff={}", stage, is_playoff);

    if is_playoff {
        let all_matches = MatchRepository::get_by_tournament(pool, tournament_id as u64)
            .await
            .unwrap_or_default();

        log::debug!("[Playoffs Debug] 获取到 {} 场比赛", all_matches.len());
        for m in &all_matches {
            log::debug!("[Playoffs Debug] Match id={}, stage={}, order={:?}, status={:?}, winner={:?}",
                m.id, m.stage, m.match_order, m.status, m.winner_id);
        }

        let league_service = LeagueService::new();
        let new_matches = league_service.advance_playoff_bracket(tournament_id as u64, &all_matches);

        log::debug!("[Playoffs Debug] advance_playoff_bracket 返回 {} 场新比赛", new_matches.len());

        if !new_matches.is_empty() {
            log::debug!("生成 {} 场新比赛", new_matches.len());
            for nm in &new_matches {
                log::debug!("新比赛: stage={}, home={}, away={}", nm.stage, nm.home_team_id, nm.away_team_id);
            }
            match MatchRepository::create_batch(pool, save_id, &new_matches).await {
                Ok(_) => log::debug!("比赛保存成功"),
                Err(e) => log::debug!("比赛保存失败: {:?}", e),
            }
        }
    }

    Ok(MatchInfo {
        id: match_id,
        tournament_id: tournament_id as u64,
        stage,
        round: round.map(|r| r as u32),
        match_order: None,
        format: format_str,
        home_team_id: home_team_id as u64,
        away_team_id: away_team_id as u64,
        home_score: home_score as u32,
        away_score: away_score as u32,
        winner_id: Some(final_winner_id as u64),
        status: "Completed".to_string(),
    })
}

async fn update_player_form_factors_internal(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    players: &[PlayerData],
    won: bool,
) -> Result<(), String> {
    for player in players {
        let updated_factors = ConditionEngine::update_form_factors(
            player.form_factors.clone(),
            won,
            player.ability as f64,
        );

        sqlx::query(
            r#"
            INSERT INTO player_form_factors (save_id, player_id, form_cycle, momentum, last_performance, last_match_won, games_since_rest, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(save_id, player_id) DO UPDATE SET
                form_cycle = excluded.form_cycle,
                momentum = excluded.momentum,
                last_performance = excluded.last_performance,
                last_match_won = excluded.last_match_won,
                games_since_rest = excluded.games_since_rest,
                updated_at = datetime('now')
            "#,
        )
        .bind(save_id)
        .bind(player.id as i64)
        .bind(updated_factors.form_cycle)
        .bind(updated_factors.momentum as i64)
        .bind(updated_factors.last_performance)
        .bind(if updated_factors.last_match_won { 1i64 } else { 0i64 })
        .bind(updated_factors.games_since_rest as i64)
        .execute(pool)
        .await
        .ok();
    }

    Ok(())
}

async fn save_player_tournament_stats_from_match(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season: u64,
    tournament_id: u64,
    tournament_type: &str,
    home_team_id: u64,
    home_team_name: &str,
    away_team_id: u64,
    away_team_name: &str,
    _home_score: u8,
    _away_score: u8,
    _winner_id: u64,
    home_players: &[PlayerData],
    away_players: &[PlayerData],
    games: &[SaveGameInput],
) -> Result<(), String> {
    let mut player_stats_map: std::collections::HashMap<u64, (u32, u32, f64, f64, f64, f64, u32)> =
        std::collections::HashMap::new();

    for game in games {
        let game_winner_team_id = game.winner_team_id;

        for perf in &game.performances {
            let player_id = perf.player_id as u64;
            let entry = player_stats_map.entry(player_id).or_insert((0, 0, 0.0, 0.0, f64::MIN, f64::MIN, 0));

            entry.0 += 1;
            if perf.team_id == game_winner_team_id {
                entry.1 += 1;
            }
            entry.2 += perf.impact_score;
            entry.3 += perf.actual_ability;
            if perf.impact_score > entry.4 {
                entry.4 = perf.impact_score;
            }
            if perf.actual_ability > entry.5 {
                entry.5 = perf.actual_ability;
            }
            if perf.is_mvp {
                entry.6 += 1;
            }
        }
    }

    for player in home_players {
        if let Some(&(games_played, games_won, total_impact, total_perf, max_impact, best_perf, mvp_count)) =
            player_stats_map.get(&player.id)
        {
            let avg_impact = if games_played > 0 { total_impact / games_played as f64 } else { 0.0 };
            let avg_performance = if games_played > 0 { total_perf / games_played as f64 } else { 0.0 };

            save_or_update_player_tournament_stats_v2(
                pool, save_id, season, tournament_id, tournament_type,
                player.id, &player.game_id, &player.position,
                home_team_id, home_team_name,
                games_played, games_won, total_impact, avg_impact, max_impact,
                avg_performance, best_perf, mvp_count,
            ).await?;
        }
    }

    for player in away_players {
        if let Some(&(games_played, games_won, total_impact, total_perf, max_impact, best_perf, mvp_count)) =
            player_stats_map.get(&player.id)
        {
            let avg_impact = if games_played > 0 { total_impact / games_played as f64 } else { 0.0 };
            let avg_performance = if games_played > 0 { total_perf / games_played as f64 } else { 0.0 };

            save_or_update_player_tournament_stats_v2(
                pool, save_id, season, tournament_id, tournament_type,
                player.id, &player.game_id, &player.position,
                away_team_id, away_team_name,
                games_played, games_won, total_impact, avg_impact, max_impact,
                avg_performance, best_perf, mvp_count,
            ).await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn save_or_update_player_tournament_stats_v2(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season: u64,
    tournament_id: u64,
    tournament_type: &str,
    player_id: u64,
    player_name: &str,
    position: &str,
    team_id: u64,
    team_name: &str,
    games_played: u32,
    games_won: u32,
    total_impact: f64,
    avg_impact: f64,
    max_impact: f64,
    avg_performance: f64,
    best_performance: f64,
    mvp_count: u32,
) -> Result<(), String> {
    let existing = PlayerTournamentStatsRepository::get_by_player_tournament(
        pool,
        save_id,
        tournament_id,
        player_id,
    ).await.ok().flatten();

    if let Some(mut stats) = existing {
        let new_games_played = stats.games_played + games_played;
        let new_total_impact = stats.total_impact + total_impact;

        stats.games_played = new_games_played;
        stats.games_won += games_won;
        stats.total_impact = new_total_impact;
        stats.avg_impact = if new_games_played > 0 { new_total_impact / new_games_played as f64 } else { 0.0 };
        if max_impact > stats.max_impact {
            stats.max_impact = max_impact;
        }

        let new_total_perf = stats.avg_performance * (stats.games_played - games_played) as f64 + avg_performance * games_played as f64;
        stats.avg_performance = if new_games_played > 0 { new_total_perf / new_games_played as f64 } else { 0.0 };

        if best_performance > stats.best_performance {
            stats.best_performance = best_performance;
        }
        stats.game_mvp_count += mvp_count;

        PlayerTournamentStatsRepository::upsert(pool, &stats)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        let stats = PlayerTournamentStats {
            id: 0,
            save_id: save_id.to_string(),
            season_id: season,
            tournament_id,
            tournament_type: tournament_type.to_string(),
            player_id,
            player_name: player_name.to_string(),
            team_id,
            team_name: team_name.to_string(),
            position: position.to_string(),
            games_played,
            games_won,
            total_impact,
            avg_impact,
            max_impact: if max_impact > f64::MIN { max_impact } else { 0.0 },
            avg_performance,
            best_performance: if best_performance > f64::MIN { best_performance } else { 0.0 },
            game_mvp_count: mvp_count,
            created_at: None,
            updated_at: None,
        };

        PlayerTournamentStatsRepository::upsert(pool, &stats)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
