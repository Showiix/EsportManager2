use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::{ConditionContext, ConditionEngine, PlayerFormFactors, TraitType, TraitEngine, TraitContext, MetaEngine, MetaWeights};
use crate::engines::lineup_engine::{LineupCandidate, LineupEngine, SubstitutionContext, SubstitutionDecision};
use crate::engines::bp_engine::{BpEngine, CompType, PlayerChampionPool, SeriesContext, TeamSide};
use crate::engines::champion::{self, MasteryTier, VersionTier};
use crate::engines::meta_engine::MetaType;
use crate::models::MatchFormat;
use crate::models::player::Position;
use crate::models::transfer::AITeamPersonality;
use crate::models::tournament_result::PlayerTournamentStats;
use crate::services::LeagueService;
use crate::db::{MatchRepository, PlayerTournamentStatsRepository};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use sqlx::Row;
use tauri::State;

use super::{
    DetailedMatchResult, DetailedGameResult, PlayerMvpInfo, TeamMatchStats, PlayerGameStats,
    ActivatedTraitInfo, GameEvent, BatchDetailedResult,
};

// ==================== 内部类型 ====================

struct SharedSimulationContext {
    save_id: String,
    current_season: i64,
    _tournament_id: u64,
    tournament_type: String,
    meta_weights: MetaWeights,
}

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
    potential: u8,
    satisfaction: u8,
    season_games_played: u32,
    join_season: i64,
    is_starter: bool,
}

// ==================== 公开命令 ====================

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

    // 获取比赛的 tournament_id
    let tournament_id: i64 = sqlx::query_scalar(
        "SELECT tournament_id FROM matches WHERE id = ?"
    )
    .bind(match_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Match not found".to_string())?;

    // 获取当前赛季
    let current_season: i64 = sqlx::query_scalar(
        "SELECT current_season FROM saves WHERE id = ?"
    )
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or(1);

    // 获取赛事类型
    let tournament_type: String = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_else(|| "league".to_string());

    // 获取 Meta 权重
    let meta_weights = MetaEngine::get_current_weights(&pool, &save_id, current_season)
        .await
        .unwrap_or_else(|_| MetaWeights::balanced());

    let ctx = SharedSimulationContext {
        save_id,
        current_season,
        _tournament_id: tournament_id as u64,
        tournament_type,
        meta_weights,
    };

    match simulate_single_match_internal(&pool, &ctx, match_id).await {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 批量模拟赛事所有待模拟比赛并返回详细结果
#[tauri::command]
pub async fn simulate_all_matches_detailed(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<BatchDetailedResult>, String> {
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

    // 一次性查询共享数据
    let current_season: i64 = sqlx::query_scalar(
        "SELECT current_season FROM saves WHERE id = ?"
    )
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or(1);

    let tournament_type: String = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_else(|| "league".to_string());

    let meta_weights = MetaEngine::get_current_weights(&pool, &save_id, current_season)
        .await
        .unwrap_or_else(|_| MetaWeights::balanced());

    let ctx = SharedSimulationContext {
        save_id,
        current_season,
        _tournament_id: tournament_id,
        tournament_type,
        meta_weights,
    };

    // 查询所有待模拟比赛
    let pending_rows = sqlx::query(
        "SELECT id FROM matches WHERE tournament_id = ? AND status = 'Scheduled' ORDER BY id"
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let total = pending_rows.len() as u32;
    let mut results = Vec::with_capacity(total as usize);
    let mut success: u32 = 0;
    let mut failed: u32 = 0;

    for row in &pending_rows {
        let match_id: i64 = row.get("id");
        match simulate_single_match_internal(&pool, &ctx, match_id as u64).await {
            Ok(result) => {
                results.push(result);
                success += 1;
            }
            Err(e) => {
                log::error!("批量模拟比赛 {} 失败: {}", match_id, e);
                failed += 1;
            }
        }
    }

    Ok(CommandResult::ok(BatchDetailedResult {
        results,
        total,
        success,
        failed,
    }))
}

// ==================== 内部实现 ====================

/// 内部函数：模拟单场比赛（使用预获取的共享数据）
async fn simulate_single_match_internal(
    pool: &sqlx::SqlitePool,
    ctx: &SharedSimulationContext,
    match_id: u64,
) -> Result<DetailedMatchResult, String> {
    // 获取比赛信息
    let match_row = sqlx::query(
        r#"
        SELECT m.id, m.tournament_id, m.format, m.home_team_id, m.away_team_id, m.stage,
               ht.power_rating as home_power, at.power_rating as away_power,
               ht.name as home_team_name, at.name as away_team_name,
               ht.region_id as home_region_id, at.region_id as away_region_id
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
    let _home_power: f64 = match_row.get("home_power");
    let _away_power: f64 = match_row.get("away_power");
    let format_str: String = match_row.get("format");
    let stage: String = match_row.get("stage");
    let home_team_name: String = match_row.get("home_team_name");
    let away_team_name: String = match_row.get("away_team_name");
    let _home_region_id: Option<i64> = match_row.get("home_region_id");
    let _away_region_id: Option<i64> = match_row.get("away_region_id");

    let format = match format_str.as_str() {
        "Bo1" => MatchFormat::Bo1,
        "Bo3" => MatchFormat::Bo3,
        "Bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    };

    // 获取双方大名单（首发+替补）
    let (mut home_players, mut home_bench) = get_match_roster(pool, &ctx.save_id, home_team_id as u64, ctx.current_season).await?;
    let (mut away_players, mut away_bench) = get_match_roster(pool, &ctx.save_id, away_team_id as u64, ctx.current_season).await?;

    // 获取队伍 personality（用于换人决策）
    let home_personality = get_team_personality(pool, home_team_id as u64).await;
    let away_personality = get_team_personality(pool, away_team_id as u64).await;

    let bo_count: u8 = match format {
        MatchFormat::Bo1 => 1,
        MatchFormat::Bo3 => 3,
        MatchFormat::Bo5 => 5,
    };

    let mut rng = StdRng::from_entropy();

    let wins_needed = format.wins_needed();
    let mut home_score: u8 = 0;
    let mut away_score: u8 = 0;
    let mut games = Vec::new();
    let mut game_number: u8 = 1;

    // 追踪系列赛中每个选手的出场局数
    let mut games_played_series: HashMap<u64, u8> = HashMap::new();
    // 追踪每个选手在整场比赛中的总出场局数（用于赛后更新 season_games_played）
    let mut player_games_in_match: HashMap<u64, u32> = HashMap::new();
    let mut pending_home_subs: HashMap<u64, (u64, String)> = HashMap::new();
    let mut pending_away_subs: HashMap<u64, (u64, String)> = HashMap::new();

    let mut total_home_stats = TeamMatchStats::default(home_team_id as u64);
    let mut total_away_stats = TeamMatchStats::default(away_team_id as u64);

    sqlx::query("DELETE FROM match_lineups WHERE save_id = ? AND match_id = ?")
        .bind(&ctx.save_id)
        .bind(match_id as i64)
        .execute(pool)
        .await
        .ok();

    // === BP系统：加载选手英雄池 ===
    let home_player_ids: Vec<u64> = home_players.iter().map(|p| p.id).collect();
    let away_player_ids: Vec<u64> = away_players.iter().map(|p| p.id).collect();
    let home_bench_ids: Vec<u64> = home_bench.iter().map(|p| p.id).collect();
    let away_bench_ids: Vec<u64> = away_bench.iter().map(|p| p.id).collect();
    let all_player_ids: Vec<u64> = home_player_ids.iter()
        .chain(away_player_ids.iter())
        .chain(home_bench_ids.iter())
        .chain(away_bench_ids.iter())
        .cloned().collect();
    
    // 创建选手的英雄池映射
    let mut home_champion_pools: HashMap<u64, PlayerChampionPool> = HashMap::new();
    let mut away_champion_pools: HashMap<u64, PlayerChampionPool> = HashMap::new();
    
    // 初始化所有选手的英雄池
    for p in &home_players {
        let pos = match p.position.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" | "MIDDLE" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };
        home_champion_pools.insert(p.id, PlayerChampionPool {
            player_id: p.id,
            position: pos,
            ability: p.ability,
            masteries: HashMap::new(),
            games_played: HashMap::new(),
            games_won: HashMap::new(),
            traits: p.traits.clone(),
        });
    }
    for p in &away_players {
        let pos = match p.position.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" | "MIDDLE" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };
        away_champion_pools.insert(p.id, PlayerChampionPool {
            player_id: p.id,
            position: pos,
            ability: p.ability,
            masteries: HashMap::new(),
            games_played: HashMap::new(),
            games_won: HashMap::new(),
            traits: p.traits.clone(),
        });
    }
    for p in &home_bench {
        let pos = match p.position.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" | "MIDDLE" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };
        home_champion_pools.insert(p.id, PlayerChampionPool {
            player_id: p.id,
            position: pos,
            ability: p.ability,
            masteries: HashMap::new(),
            games_played: HashMap::new(),
            games_won: HashMap::new(),
            traits: p.traits.clone(),
        });
    }
    for p in &away_bench {
        let pos = match p.position.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" | "MIDDLE" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };
        away_champion_pools.insert(p.id, PlayerChampionPool {
            player_id: p.id,
            position: pos,
            ability: p.ability,
            masteries: HashMap::new(),
            games_played: HashMap::new(),
            games_won: HashMap::new(),
            traits: p.traits.clone(),
        });
    }
    
    // 加载选手的英雄池数据
    if !all_player_ids.is_empty() {
        let placeholders: String = all_player_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT player_id, champion_id, mastery_tier, games_played, games_won FROM player_champion_mastery WHERE save_id = ? AND player_id IN ({})",
            placeholders
        );
        
        let mut query_builder = sqlx::query(&query).bind(&ctx.save_id);
        for id in &all_player_ids {
            query_builder = query_builder.bind(*id as i64);
        }
        
        let mastery_rows = query_builder.fetch_all(pool).await;
        
        if let Ok(rows) = mastery_rows {
            for row in &rows {
                let player_id: i64 = row.get("player_id");
                let champion_id: i64 = row.get("champion_id");
                let tier_str: String = row.get("mastery_tier");
                let games_played: i64 = row.get("games_played");
                let games_won_val: i64 = row.get("games_won");
                
                if let Some(tier) = MasteryTier::from_id(&tier_str) {
                    let player_id = player_id as u64;
                    let champion_id = champion_id as u8;
                    let games_played = games_played.max(0) as u32;
                    let games_won_val = games_won_val.max(0) as u32;
                    
                    if let Some(pool_entry) = home_champion_pools.get_mut(&player_id) {
                        pool_entry.masteries.insert(champion_id, tier);
                        pool_entry.games_played.insert(champion_id, games_played);
                        pool_entry.games_won.insert(champion_id, games_won_val);
                    } else if let Some(pool_entry) = away_champion_pools.get_mut(&player_id) {
                        pool_entry.masteries.insert(champion_id, tier);
                        pool_entry.games_played.insert(champion_id, games_played);
                        pool_entry.games_won.insert(champion_id, games_won_val);
                    }
                }
            }
        }
    }
    
    // 获取Meta类型和版本权重
    let meta_type_row: Option<String> = sqlx::query_scalar(
        "SELECT meta_type FROM meta_versions WHERE save_id = ? AND season_id = ? LIMIT 1"
    )
    .bind(&ctx.save_id)
    .bind(ctx.current_season)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let meta_type = meta_type_row
        .and_then(|s| MetaType::from_id(&s))
        .unwrap_or(MetaType::Balanced);
    let version_tiers: HashMap<u8, VersionTier> = champion::calculate_version_tiers(meta_type)
        .into_iter()
        .collect();
    
    let mut bp_rng = StdRng::from_entropy();
    let home_team_comp_history = load_team_comp_history(pool, &ctx.save_id, home_team_id).await;
    let away_team_comp_history = load_team_comp_history(pool, &ctx.save_id, away_team_id).await;
    let mut series_ctx: Option<SeriesContext> = None;

    while home_score < wins_needed && away_score < wins_needed {
        // === BP系统：每局比赛前运行BP ===
        let mut home_pool_vec: Vec<PlayerChampionPool> = Vec::new();
        for p in &home_players {
            if let Some(pool) = home_champion_pools.get(&p.id).cloned() {
                home_pool_vec.push(pool);
            }
        }
        
        let mut away_pool_vec: Vec<PlayerChampionPool> = Vec::new();
        for p in &away_players {
            if let Some(pool) = away_champion_pools.get(&p.id).cloned() {
                away_pool_vec.push(pool);
            }
        }
        
        // 运行BP
        let draft = BpEngine::run_draft(
            &home_pool_vec,
            &away_pool_vec,
            &version_tiers,
            meta_type,
            &mut bp_rng,
            &home_team_comp_history,
            &away_team_comp_history,
            series_ctx.as_ref(),
        );
        
        // 保存BP结果到数据库
        let bans_json = serde_json::to_string(&draft.bans).unwrap_or_default();
        let home_picks_json = serde_json::to_string(&draft.home_picks).unwrap_or_default();
        let away_picks_json = serde_json::to_string(&draft.away_picks).unwrap_or_default();
        let home_comp = draft.home_comp.as_ref().map(|c| format!("{:?}", c));
        let away_comp = draft.away_comp.as_ref().map(|c| format!("{:?}", c));
        let narrative_json = draft
            .narrative
            .as_ref()
            .map(|n| serde_json::to_string(n).unwrap_or_default());
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO game_draft_results
                (save_id, match_id, game_number, bans_json, home_picks_json, away_picks_json, home_comp, away_comp, draft_narrative_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&ctx.save_id)
        .bind(match_id as i64)
        .bind(game_number)
        .bind(&bans_json)
        .bind(&home_picks_json)
        .bind(&away_picks_json)
        .bind(&home_comp)
        .bind(&away_comp)
        .bind(&narrative_json)
        .execute(pool)
        .await
        .ok();
        
        insert_match_lineups_for_team(
            pool,
            &ctx.save_id,
            match_id,
            game_number,
            home_team_id as u64,
            &home_players,
            &pending_home_subs,
        )
        .await;
        insert_match_lineups_for_team(
            pool,
            &ctx.save_id,
            match_id,
            game_number,
            away_team_id as u64,
            &away_players,
            &pending_away_subs,
        )
        .await;
        pending_home_subs.clear();
        pending_away_subs.clear();

        let duration = 25 + rng.gen_range(0..25);

        // 记录本局出场选手
        for p in home_players.iter().chain(away_players.iter()) {
            *games_played_series.entry(p.id).or_insert(0) += 1;
            *player_games_in_match.entry(p.id).or_insert(0) += 1;
        }

        let is_international = matches!(
            ctx.tournament_type.as_str(),
            "msi" | "Msi" | "worlds" | "WorldChampionship" | "masters" | "MadridMasters"
            | "shanghai" | "ShanghaiMasters" | "clauch" | "ClaudeIntercontinental"
            | "icp" | "Icp" | "IcpIntercontinental" | "super" | "SuperIntercontinental"
        );
        let is_playoff = ctx.tournament_type.contains("playoff") ||
                         ctx.tournament_type == "knockout";

        let trait_ctx = TraitContext {
            tournament_type: ctx.tournament_type.clone(),
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
            &ctx.meta_weights,
            &mut rng,
        );

        // 根据队伍发挥战力决定胜负（加入正态分布增加不确定性）
        // 使用 Box-Muller 变换生成高斯随机数
        let game_std_dev = 3.0; // 局内波动标准差
        let u1: f64 = rng.r#gen::<f64>().max(0.0001);
        let u2: f64 = rng.r#gen::<f64>().max(0.0001);
        let gaussian = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

        // 双方发挥战力差值 + 随机波动
        let performance_diff = home_perf - away_perf;
        let final_diff = performance_diff + gaussian * game_std_dev;

        let winner_id = if final_diff > 0.0 {
            home_team_id as u64
        } else {
            away_team_id as u64
        };

        // 选择MVP（仅从胜方队伍中选择）
        let winner_player_stats: Vec<&PlayerGameStats> = if winner_id == home_team_id as u64 {
            home_player_stats.iter().collect()
        } else {
            away_player_stats.iter().collect()
        };
        let game_mvp = select_mvp(&winner_player_stats, winner_id);

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

        let home_count = home_player_stats.len().max(1) as f64;
        let away_count = away_player_stats.len().max(1) as f64;
        let home_base_avg = home_player_stats.iter().map(|p| p.base_ability as f64).sum::<f64>() / home_count;
        let away_base_avg = away_player_stats.iter().map(|p| p.base_ability as f64).sum::<f64>() / away_count;
        let home_bp_avg = draft.home_bp_modifiers.values().sum::<f64>() / home_count;
        let away_bp_avg = draft.away_bp_modifiers.values().sum::<f64>() / away_count;

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
            home_base_power: Some(home_base_avg),
            away_base_power: Some(away_base_avg),
            home_synergy_bonus: Some(0.0),
            away_synergy_bonus: Some(0.0),
            home_bp_bonus: Some(home_bp_avg),
            away_bp_bonus: Some(away_bp_avg),
            home_version_bonus: Some(0.0),
            away_version_bonus: Some(0.0),
        });

        let home_won_this_game = winner_id == home_team_id as u64;
        if home_won_this_game {
            home_score += 1;
        } else {
            away_score += 1;
        }

        let prev_winner_picks: Vec<u8> = if home_won_this_game {
            draft.home_picks.iter().map(|p| p.champion_id).collect()
        } else {
            draft.away_picks.iter().map(|p| p.champion_id).collect()
        };
        let prev_loser_side = if home_won_this_game {
            Some(TeamSide::Away)
        } else {
            Some(TeamSide::Home)
        };
        series_ctx = Some(SeriesContext {
            prev_winner_picks,
            prev_loser_side,
            prev_home_comp: draft.home_comp,
            prev_away_comp: draft.away_comp,
            home_score,
            away_score,
            game_number,
            wins_needed,
        });

        // === 更新选手英雄池 games_played / games_won ===
        for pick in &draft.home_picks {
            let won_val: i64 = if home_won_this_game { 1 } else { 0 };
            sqlx::query(
                "UPDATE player_champion_mastery SET games_played = games_played + 1, games_won = games_won + ? WHERE save_id = ? AND player_id = ? AND champion_id = ?"
            )
            .bind(won_val)
            .bind(&ctx.save_id)
            .bind(pick.player_id as i64)
            .bind(pick.champion_id as i64)
            .execute(pool)
            .await
            .ok();
        }
        for pick in &draft.away_picks {
            let won_val: i64 = if !home_won_this_game { 1 } else { 0 };
            sqlx::query(
                "UPDATE player_champion_mastery SET games_played = games_played + 1, games_won = games_won + ? WHERE save_id = ? AND player_id = ? AND champion_id = ?"
            )
            .bind(won_val)
            .bind(&ctx.save_id)
            .bind(pick.player_id as i64)
            .bind(pick.champion_id as i64)
            .execute(pool)
            .await
            .ok();
        }

        // 局间状态更新：出场选手累加疲劳，板凳选手休息恢复
        if bo_count > 1 && home_score < wins_needed && away_score < wins_needed {
            // 出场选手：games_since_rest +1，更新 condition
            for player in home_players.iter_mut().chain(away_players.iter_mut()) {
                player.form_factors.games_since_rest += 1;
                let season_games_played = player.season_games_played.saturating_add(
                    player_games_in_match.get(&player.id).copied().unwrap_or(0),
                );
                let condition_ctx = build_condition_context(
                    player.satisfaction,
                    season_games_played,
                    player.traits.contains(&TraitType::Ironman),
                );
                player.condition = ConditionEngine::calculate_condition_full(
                    player.age,
                    player.ability,
                    &player.form_factors,
                    &condition_ctx,
                );
            }
            // 板凳选手：rest recovery（games_since_rest=0, momentum衰减, 重算condition）
            for player in home_bench.iter_mut().chain(away_bench.iter_mut()) {
                player.form_factors = ConditionEngine::update_form_factors_bench(player.form_factors.clone());
                let season_games_played = player.season_games_played.saturating_add(
                    player_games_in_match.get(&player.id).copied().unwrap_or(0),
                );
                let condition_ctx = build_condition_context(
                    player.satisfaction,
                    season_games_played,
                    player.traits.contains(&TraitType::Ironman),
                );
                player.condition = ConditionEngine::calculate_condition_full(
                    player.age,
                    player.ability,
                    &player.form_factors,
                    &condition_ctx,
                );
            }
        }

        // 局间换人：仅在 BO 系列赛且比赛未结束时执行
        if bo_count > 1 && home_score < wins_needed && away_score < wins_needed {
            let sub_ctx_home = SubstitutionContext {
                tournament_type: ctx.tournament_type.clone(),
                round: stage.clone(),
                bo_count,
                game_number,
                home_score,
                away_score,
                is_home: true,
                current_season: ctx.current_season as u32,
            };
            let home_starter_candidates: Vec<LineupCandidate> = home_players.iter().map(player_data_to_candidate).collect();
            let home_bench_candidates: Vec<LineupCandidate> = home_bench.iter().map(player_data_to_candidate).collect();
            let home_subs = LineupEngine::check_substitutions(
                &home_starter_candidates, &home_bench_candidates,
                &sub_ctx_home, &home_personality,
                ctx.current_season as u32, &games_played_series,
            );
            if !home_subs.is_empty() {
                let applied = apply_substitutions(&mut home_players, &mut home_bench, &home_subs);
                for d in applied {
                    pending_home_subs
                        .insert(d.sub_in_player_id, (d.sub_out_player_id, d.reason.clone()));
                }
            }

            let sub_ctx_away = SubstitutionContext {
                tournament_type: ctx.tournament_type.clone(),
                round: stage.clone(),
                bo_count,
                game_number,
                home_score,
                away_score,
                is_home: false,
                current_season: ctx.current_season as u32,
            };
            let away_starter_candidates: Vec<LineupCandidate> = away_players.iter().map(player_data_to_candidate).collect();
            let away_bench_candidates: Vec<LineupCandidate> = away_bench.iter().map(player_data_to_candidate).collect();
            let away_subs = LineupEngine::check_substitutions(
                &away_starter_candidates, &away_bench_candidates,
                &sub_ctx_away, &away_personality,
                ctx.current_season as u32, &games_played_series,
            );
            if !away_subs.is_empty() {
                let applied = apply_substitutions(&mut away_players, &mut away_bench, &away_subs);
                for d in applied {
                    pending_away_subs
                        .insert(d.sub_in_player_id, (d.sub_out_player_id, d.reason.clone()));
                }
            }
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

    // 选择比赛MVP（仅从胜方队伍中选择）
    let match_mvp = select_match_mvp(&games, home_team_id as u64, away_team_id as u64, winner_id);

    // 更新数据库中的比赛结果
    sqlx::query(
        "UPDATE matches SET home_score = ?, away_score = ?, winner_id = ?, status = 'COMPLETED' WHERE id = ?"
    )
    .bind(home_score as i64)
    .bind(away_score as i64)
    .bind(winner_id as i64)
    .bind(match_id as i64)
    .execute(pool)
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

    // 更新胜方积分榜 (使用 upsert，如果记录不存在则创建)
    sqlx::query(
        r#"
        INSERT INTO league_standings (save_id, tournament_id, team_id, rank, matches_played, wins, losses, points, games_won, games_lost, game_diff)
        VALUES (?, ?, ?, NULL, 1, 1, 0, ?, ?, ?, ?)
        ON CONFLICT(tournament_id, team_id) DO UPDATE SET
            matches_played = matches_played + 1,
            wins = wins + 1,
            points = points + excluded.points,
            games_won = games_won + excluded.games_won,
            games_lost = games_lost + excluded.games_lost,
            game_diff = game_diff + excluded.game_diff
        "#
    )
    .bind(&ctx.save_id)
    .bind(tournament_id)
    .bind(winner_id as i64)
    .bind(winner_points as i64)
    .bind(winner_games_won as i64)
    .bind(winner_games_lost as i64)
    .bind((winner_games_won - winner_games_lost) as i64)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 更新负方积分榜 (使用 upsert，如果记录不存在则创建)
    sqlx::query(
        r#"
        INSERT INTO league_standings (save_id, tournament_id, team_id, rank, matches_played, wins, losses, points, games_won, games_lost, game_diff)
        VALUES (?, ?, ?, NULL, 1, 0, 1, ?, ?, ?, ?)
        ON CONFLICT(tournament_id, team_id) DO UPDATE SET
            matches_played = matches_played + 1,
            losses = losses + 1,
            points = points + excluded.points,
            games_won = games_won + excluded.games_won,
            games_lost = games_lost + excluded.games_lost,
            game_diff = game_diff + excluded.game_diff
        "#
    )
    .bind(&ctx.save_id)
    .bind(tournament_id)
    .bind(loser_id as i64)
    .bind(loser_points as i64)
    .bind(loser_games_won as i64)
    .bind(loser_games_lost as i64)
    .bind((loser_games_won - loser_games_lost) as i64)
    .execute(pool)
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
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 保存小局详情（使用事务批量写入，减少磁盘 fsync）
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    for game in &games {
        let game_id = format!("{}_{}", match_id, game.game_number);
        let loser_id = if game.winner_id == home_team_id as u64 {
            away_team_id
        } else {
            home_team_id
        };

        // 找出本局 MVP（影响力最高的选手）
        let all_players: Vec<_> = game.home_players.iter().chain(game.away_players.iter()).collect();
        let mvp_player_id = all_players.iter()
            .max_by(|a, b| a.mvp_score.partial_cmp(&b.mvp_score).unwrap_or(std::cmp::Ordering::Equal))
            .map(|p| p.player_id as i64)
            .unwrap_or(0);

        // 计算队伍战力（选手实际发挥值平均）
        let home_power: f64 = if !game.home_players.is_empty() {
            game.home_players.iter().map(|p| p.actual_ability).sum::<f64>() / game.home_players.len() as f64
        } else { 0.0 };
        let away_power: f64 = if !game.away_players.is_empty() {
            game.away_players.iter().map(|p| p.actual_ability).sum::<f64>() / game.away_players.len() as f64
        } else { 0.0 };

        sqlx::query(
            r#"
            INSERT INTO match_games (
                id, save_id, match_id, game_number, winner_team_id, loser_team_id,
                duration_minutes, mvp_player_id, key_player_id,
                home_power, away_power, home_meta_power, away_meta_power
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                winner_team_id = excluded.winner_team_id,
                loser_team_id = excluded.loser_team_id,
                duration_minutes = excluded.duration_minutes,
                mvp_player_id = excluded.mvp_player_id,
                home_power = excluded.home_power,
                away_power = excluded.away_power,
                home_meta_power = excluded.home_meta_power,
                away_meta_power = excluded.away_meta_power
            "#,
        )
        .bind(&game_id)
        .bind(&ctx.save_id)
        .bind(match_id as i64)
        .bind(game.game_number as i64)
        .bind(game.winner_id as i64)
        .bind(loser_id)
        .bind(game.duration_minutes as i64)
        .bind(mvp_player_id)
        .bind(mvp_player_id) // key_player 暂时与 mvp 相同
        .bind(home_power)
        .bind(away_power)
        .bind(game.home_performance)
        .bind(game.away_performance)
        .execute(&mut *tx)
        .await
        .ok();

        // 保存选手详细表现数据
        for player in game.home_players.iter().chain(game.away_players.iter()) {
            let perf_id = format!("{}_{}_{}", game_id, player.player_id, player.position);
            let is_home = game.home_players.iter().any(|p| p.player_id == player.player_id);
            let team_id = if is_home { home_team_id } else { away_team_id };
            let team_name = if is_home { &home_team_name } else { &away_team_name };

            sqlx::query(
                r#"
                INSERT INTO game_player_performances (
                    id, save_id, game_id, player_id, player_name, team_id, team_name, position,
                    base_ability, condition_bonus, stability_noise, actual_ability,
                    impact_score, mvp_score, is_mvp, is_key_player,
                    kills, deaths, assists, cs, gold, damage_dealt, damage_taken, vision_score
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    actual_ability = excluded.actual_ability,
                    impact_score = excluded.impact_score,
                    mvp_score = excluded.mvp_score,
                    is_mvp = excluded.is_mvp,
                    kills = excluded.kills,
                    deaths = excluded.deaths,
                    assists = excluded.assists
                "#,
            )
            .bind(&perf_id)
            .bind(&ctx.save_id)
            .bind(&game_id)
            .bind(player.player_id as i64)
            .bind(&player.player_name)
            .bind(team_id)
            .bind(team_name)
            .bind(&player.position)
            .bind(player.base_ability)
            .bind(player.condition_bonus)
            .bind(player.stability_noise)
            .bind(player.actual_ability)
            .bind(player.impact_score)
            .bind(player.mvp_score)
            .bind(player.player_id as i64 == mvp_player_id)
            .bind(player.player_id as i64 == mvp_player_id)
            .bind(player.kills as i64)
            .bind(player.deaths as i64)
            .bind(player.assists as i64)
            .bind(player.cs as i64)
            .bind(player.gold as i64)
            .bind(player.damage_dealt as i64)
            .bind(player.damage_taken as i64)
            .bind(player.vision_score as i64)
            .execute(&mut *tx)
            .await
            .ok();
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

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

    update_player_form_factors(pool, &ctx.save_id, &home_players, home_won, home_avg_perf).await.ok();
    update_player_form_factors(pool, &ctx.save_id, &away_players, !home_won, away_avg_perf).await.ok();

    // 更新未出场替补的 form_factors（慢推进 + 疲劳清零）
    update_bench_form_factors(pool, &ctx.save_id, &home_bench).await.ok();
    update_bench_form_factors(pool, &ctx.save_id, &away_bench).await.ok();

    // 更新出场选手的 season_games_played
    for (player_id, games_count) in &player_games_in_match {
        sqlx::query(
            "UPDATE players SET season_games_played = season_games_played + ? WHERE id = ? AND save_id = ?"
        )
        .bind(*games_count as i64)
        .bind(*player_id as i64)
        .bind(&ctx.save_id)
        .execute(pool)
        .await
        .ok();
    }

    // 更新两支队伍全体选手的 season_games_total（本场比赛局数）
    let total_games_this_match = games.len() as i64;
    sqlx::query(
        "UPDATE players SET season_games_total = season_games_total + ? WHERE team_id = ? AND save_id = ? AND status = 'Active'"
    )
    .bind(total_games_this_match)
    .bind(home_team_id)
    .bind(&ctx.save_id)
    .execute(pool)
    .await
    .ok();
    sqlx::query(
        "UPDATE players SET season_games_total = season_games_total + ? WHERE team_id = ? AND save_id = ? AND status = 'Active'"
    )
    .bind(total_games_this_match)
    .bind(away_team_id)
    .bind(&ctx.save_id)
    .execute(pool)
    .await
    .ok();

    // 保存选手赛事统计（用于MVP计算）
    save_player_tournament_stats(
        pool,
        &ctx.save_id,
        ctx.current_season as u64,
        tournament_id as u64,
        &ctx.tournament_type,
        home_team_id as u64,
        &home_team_name,
        away_team_id as u64,
        &away_team_name,
        &games,
        winner_id,
    ).await.ok();

    // 如果是季后赛比赛，推进对阵生成后续比赛
    let is_playoff = stage.contains("WINNERS")
        || stage.contains("LOSERS")
        || stage.contains("GRAND_FINAL");

    if is_playoff {
        log::debug!("检测到季后赛比赛完成: stage={}", stage);

        let all_matches = MatchRepository::get_by_tournament(pool, tournament_id as u64)
            .await
            .unwrap_or_default();

        log::debug!("获取到 {} 场比赛", all_matches.len());

        let league_service = LeagueService::new();
        let new_matches = league_service.advance_playoff_bracket(tournament_id as u64, &all_matches);

        if !new_matches.is_empty() {
            log::debug!("生成 {} 场新比赛", new_matches.len());
            for nm in &new_matches {
                log::debug!("新比赛: stage={}, home={}, away={}", nm.stage, nm.home_team_id, nm.away_team_id);
            }
            match MatchRepository::create_batch(pool, &ctx.save_id, &new_matches).await {
                Ok(_) => log::debug!("比赛保存成功"),
                Err(e) => log::debug!("比赛保存失败: {:?}", e),
            }
        } else {
            log::debug!("条件不满足，未生成新比赛");
        }
    }

    Ok(DetailedMatchResult {
        match_id,
        tournament_id: tournament_id as u64,
        home_team_id: home_team_id as u64,
        away_team_id: away_team_id as u64,
        home_team_name,
        away_team_name,
        home_score,
        away_score,
        winner_id,
        games,
        match_mvp,
        home_team_stats: total_home_stats,
        away_team_stats: total_away_stats,
    })
}

// ==================== 辅助函数 ====================

async fn load_team_comp_history(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: i64,
) -> Vec<(CompType, u32)> {
    let rows = match sqlx::query(
        r#"
        SELECT d.home_comp, d.away_comp, m.home_team_id, m.away_team_id
        FROM game_draft_results d
        JOIN matches m ON m.id = d.match_id
        WHERE d.save_id = ? AND (m.home_team_id = ? OR m.away_team_id = ?)
        "#,
    )
    .bind(save_id)
    .bind(team_id)
    .bind(team_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return Vec::new(),
    };

    let mut comp_counts: HashMap<CompType, u32> = HashMap::new();
    for row in rows {
        let home_team_id: i64 = row.get("home_team_id");
        let away_team_id: i64 = row.get("away_team_id");

        let comp_str: Option<String> = if team_id == home_team_id {
            row.get("home_comp")
        } else if team_id == away_team_id {
            row.get("away_comp")
        } else {
            None
        };

        if let Some(comp_str) = comp_str {
            if let Some(comp) = CompType::from_id(comp_str.trim()) {
                *comp_counts.entry(comp).or_insert(0) += 1;
            }
        }
    }

    let mut history: Vec<(CompType, u32)> = comp_counts.into_iter().collect();
    history.sort_by(|left, right| right.1.cmp(&left.1));
    history
}

fn build_condition_context(
    satisfaction: u8,
    season_games_played: u32,
    has_ironman: bool,
) -> ConditionContext {
    ConditionContext {
        season_games_played,
        satisfaction,
        international_events: 0,
        has_ironman,
        ..Default::default()
    }
}

/// 从 SQL Row 构建 PlayerData
fn build_player_data(
    r: &sqlx::sqlite::SqliteRow,
    player_id: u64,
    traits: Vec<TraitType>,
    current_season: i64,
    is_starter: bool,
) -> PlayerData {
    let age = r.get::<i64, _>("age") as u8;
    let ability = r.get::<i64, _>("ability") as u8;
    let satisfaction = r.get::<Option<i64>, _>("satisfaction").unwrap_or(60) as u8;
    let season_games_played = r
        .get::<Option<i64>, _>("season_games_played")
        .unwrap_or(0) as u32;
    let join_season = r.get::<Option<i64>, _>("join_season").unwrap_or(1);
    let is_first_season = join_season == current_season;

    let form_factors = PlayerFormFactors {
        player_id,
        form_cycle: r.get::<Option<f64>, _>("form_cycle").unwrap_or(50.0),
        momentum: r.get::<Option<i64>, _>("momentum").unwrap_or(0) as i8,
        last_performance: r.get::<Option<f64>, _>("last_performance").unwrap_or(0.0),
        last_match_won: r.get::<Option<i64>, _>("last_match_won").unwrap_or(0) == 1,
        perf_history: r
            .get::<Option<String>, _>("perf_history")
            .unwrap_or_default(),
        games_since_rest: r.get::<Option<i64>, _>("games_since_rest").unwrap_or(0) as u32,
    };

    let condition_ctx = build_condition_context(
        satisfaction,
        season_games_played,
        traits.contains(&TraitType::Ironman),
    );
    let condition =
        ConditionEngine::calculate_condition_full(age, ability, &form_factors, &condition_ctx);

    PlayerData {
        id: player_id,
        game_id: r.get::<String, _>("game_id"),
        position: r.get::<Option<String>, _>("position")
            .unwrap_or_default()
            .trim_start_matches("Some(")
            .trim_end_matches(")")
            .to_uppercase(),
        ability,
        age,
        stability: r.get::<Option<i64>, _>("stability").unwrap_or(70) as u8,
        condition,
        form_factors,
        traits,
        is_first_season,
        potential: r.get::<Option<i64>, _>("potential").unwrap_or(ability as i64) as u8,
        satisfaction,
        season_games_played,
        join_season,
        is_starter,
    }
}

/// 加载队伍完整大名单：(首发, 替补)
async fn get_match_roster(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
    current_season: i64,
) -> Result<(Vec<PlayerData>, Vec<PlayerData>), String> {
    // 加载所有 Active 选手
    let rows = sqlx::query(
        r#"
        SELECT p.id, p.game_id, p.position, p.ability, p.age, p.stability, p.join_season,
               p.potential, p.satisfaction, p.season_games_played, p.is_starter,
               pff.form_cycle, pff.momentum, pff.last_performance, pff.last_match_won, pff.perf_history, pff.games_since_rest
        FROM players p
        LEFT JOIN player_form_factors pff ON p.id = pff.player_id
        WHERE p.save_id = ? AND p.team_id = ? AND p.status = 'Active'
        ORDER BY p.is_starter DESC, p.ability DESC
        "#,
    )
    .bind(save_id)
    .bind(team_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut starters: Vec<PlayerData> = Vec::new();
    let mut bench: Vec<PlayerData> = Vec::new();

    for r in &rows {
        let player_id = r.get::<i64, _>("id") as u64;
        let is_starter = r.get::<Option<i64>, _>("is_starter").unwrap_or(0) == 1;
        let traits = load_player_traits(pool, player_id).await?;
        let player = build_player_data(r, player_id, traits, current_season, is_starter);

        if is_starter {
            starters.push(player);
        } else {
            bench.push(player);
        }
    }

    // 如果首发不足5人，从板凳补充
    while starters.len() < 5 && !bench.is_empty() {
        // 找一个板凳选手补首发（按 ability 降序，bench 已按 ability DESC 排序）
        let player = bench.remove(0);
        starters.push(PlayerData { is_starter: true, ..player });
    }

    // 替补只保留每个位置最优的1名（不能重复位置过多）
    let mut filtered_bench: Vec<PlayerData> = Vec::new();
    let mut bench_positions: HashMap<String, usize> = HashMap::new();
    for player in bench {
        let count = bench_positions.entry(player.position.clone()).or_insert(0);
        if *count < 1 {
            *count += 1;
            filtered_bench.push(player);
        }
    }

    Ok((starters, filtered_bench))
}

#[allow(dead_code)]
async fn get_starting_players(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
    current_season: i64,
) -> Result<Vec<PlayerData>, String> {
    let (starters, _) = get_match_roster(pool, save_id, team_id, current_season).await?;
    Ok(starters)
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
    TraitType::from_str(s)
}

fn parse_position(s: &str) -> Position {
    match s.to_uppercase().as_str() {
        "TOP" => Position::Top,
        "JUG" | "JUNGLE" => Position::Jug,
        "MID" => Position::Mid,
        "ADC" | "BOT" => Position::Adc,
        "SUP" | "SUPPORT" => Position::Sup,
        _ => Position::Mid,
    }
}

fn player_data_to_candidate(p: &PlayerData) -> LineupCandidate {
    LineupCandidate {
        player_id: p.id,
        game_id: p.game_id.clone(),
        position: parse_position(&p.position),
        ability: p.ability,
        age: p.age,
        potential: p.potential,
        condition: p.condition,
        form_factors: p.form_factors.clone(),
        is_starter: p.is_starter,
        join_season: p.join_season as u32,
        traits: p.traits.clone(),
        satisfaction: p.satisfaction,
        champion_version_score: 0.0,
    }
}

async fn insert_match_lineups_for_team(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    match_id: u64,
    game_number: u8,
    team_id: u64,
    lineup: &[PlayerData],
    substitutions: &HashMap<u64, (u64, String)>,
) {
    for p in lineup {
        let (is_sub, replaced_id, reason) = if let Some((out_id, reason)) = substitutions.get(&p.id)
        {
            (1i64, Some(*out_id as i64), Some(reason.clone()))
        } else {
            (0i64, None, None)
        };

        sqlx::query(
            "INSERT INTO match_lineups (save_id, match_id, game_number, team_id, player_id, position, is_substitution, replaced_player_id, substitution_reason) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(save_id)
        .bind(match_id as i64)
        .bind(game_number as i64)
        .bind(team_id as i64)
        .bind(p.id as i64)
        .bind(&p.position)
        .bind(is_sub)
        .bind(replaced_id)
        .bind(reason)
        .execute(pool)
        .await
        .ok();
    }
}

fn apply_substitutions(
    lineup: &mut Vec<PlayerData>,
    bench: &mut Vec<PlayerData>,
    decisions: &[SubstitutionDecision],
) -> Vec<SubstitutionDecision> {
    let mut applied = Vec::new();

    for decision in decisions {
        let sub_out_idx = lineup.iter().position(|p| p.id == decision.sub_out_player_id);
        let sub_in_idx = bench.iter().position(|p| p.id == decision.sub_in_player_id);

        if let (Some(out_idx), Some(in_idx)) = (sub_out_idx, sub_in_idx) {
            let mut sub_in = bench.remove(in_idx);
            sub_in.is_starter = true;
            let mut sub_out = lineup[out_idx].clone();
            sub_out.is_starter = false;
            lineup[out_idx] = sub_in;
            bench.push(sub_out);
            applied.push(decision.clone());
        }
    }

    applied
}

async fn get_team_personality(
    pool: &sqlx::SqlitePool,
    team_id: u64,
) -> AITeamPersonality {
    let result: Option<String> = sqlx::query_scalar(
        "SELECT personality FROM team_personality_configs WHERE team_id = ? LIMIT 1"
    )
    .bind(team_id as i64)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    result
        .map(|s| AITeamPersonality::from_str(&s))
        .unwrap_or(AITeamPersonality::Balanced)
}

fn simulate_game_with_players(
    home_players: &[PlayerData],
    away_players: &[PlayerData],
    duration: u32,
    trait_ctx: &TraitContext,
    meta_weights: &MetaWeights,
    rng: &mut impl Rng,
) -> (Vec<PlayerGameStats>, Vec<PlayerGameStats>, f64, f64) {
    fn gaussian_random(rng: &mut impl Rng) -> f64 {
        let u: f64 = rng.r#gen::<f64>().max(0.0001);
        let v: f64 = rng.r#gen::<f64>().max(0.0001);
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    fn generate_team_stats(
        players: &[PlayerData],
        duration: u32,
        trait_ctx: &TraitContext,
        meta_weights: &MetaWeights,
        rng: &mut impl Rng
    ) -> (Vec<PlayerGameStats>, f64) {
        let mut stats = Vec::new();

        // 第一遍：计算每个选手的发挥值（应用特性修正）
        let mut player_performances: Vec<(f64, f64, f64, f64, Vec<ActivatedTraitInfo>)> = Vec::new();
        let mut player_abilities_with_pos: Vec<(f64, String)> = Vec::new();
        let mut total_actual_ability = 0.0;
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

            // 构建激活特性列表
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
            player_abilities_with_pos.push((actual_ability, player.position.clone()));
            player_performances.push((player.ability as f64, condition_bonus, stability_noise, actual_ability, activated_traits));
        }

        // 使用 Meta 加权计算队伍战力（与快进模拟一致）
        let weighted_input: Vec<(f64, &str)> = player_abilities_with_pos.iter()
            .map(|(a, p)| (*a, p.as_str()))
            .collect();
        let team_power = MetaEngine::calculate_team_power_weighted(&weighted_input, meta_weights);
        let team_avg = if !players.is_empty() { total_actual_ability / players.len() as f64 } else { 0.0 };

        // 第二遍：生成详细统计
        for (i, player) in players.iter().enumerate() {
            let (base_ability, condition_bonus, stability_noise, actual_ability, ref activated_traits) = player_performances[i];

            // 根据发挥值生成KDA等统计
            let base = actual_ability / 100.0;
            let kills = (base * (3.0 + rng.r#gen::<f64>() * 5.0)) as u32;
            let deaths = ((1.0 - base * 0.5) * (2.0 + rng.r#gen::<f64>() * 4.0)) as u32;
            let assists = (base * (4.0 + rng.r#gen::<f64>() * 8.0)) as u32;
            let cs = (duration as f64 * (7.0 + rng.r#gen::<f64>() * 3.0) * base) as u32;
            let gold = cs as u64 * 20 + kills as u64 * 300 + assists as u64 * 150;
            let damage = (actual_ability * 1000.0 * (0.8 + rng.r#gen::<f64>() * 0.4)) as u64;

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
                vision_score: (duration as f64 * (0.5 + rng.r#gen::<f64>() * 1.5)) as u32,
                mvp_score,
                impact_score,
                traits: player.traits.iter().map(|t| format!("{:?}", t)).collect(),
                activated_traits: activated_traits.clone(),
            });
        }

        (stats, team_power)
    }

    // 生成双方统计
    let (home_stats, home_avg) = generate_team_stats(home_players, duration, trait_ctx, meta_weights, rng);
    let (away_stats, away_avg) = generate_team_stats(away_players, duration, trait_ctx, meta_weights, rng);

    (home_stats, away_stats, home_avg, away_avg)
}

fn select_mvp(stats: &[&PlayerGameStats], team_id: u64) -> PlayerMvpInfo {
    let best = stats.iter()
        .max_by(|a, b| a.mvp_score.partial_cmp(&b.mvp_score).unwrap())
        .unwrap();

    PlayerMvpInfo {
        player_id: best.player_id,
        player_name: best.player_name.clone(),
        team_id,
        position: best.position.clone(),
        mvp_score: best.mvp_score,
    }
}

fn select_match_mvp(
    games: &[DetailedGameResult],
    home_team_id: u64,
    _away_team_id: u64,
    winner_id: u64,
) -> Option<PlayerMvpInfo> {
    // 确定胜方是主队还是客队
    let is_home_winner = winner_id == home_team_id;

    // 只统计胜方队伍选手的数据: (player_name, position, total_mvp_score, game_count)
    let mut player_scores: std::collections::HashMap<u64, (String, String, f64, u32)> = std::collections::HashMap::new();

    for game in games {
        // 只收集胜方队伍的选手数据
        let winner_players = if is_home_winner {
            &game.home_players
        } else {
            &game.away_players
        };

        for p in winner_players {
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
            team_id: winner_id,
            position: pos,
            mvp_score: score / count as f64,
        })
}

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
    let fb_team = if rng.r#gen::<f64>() < 0.6 { winner_id } else { loser_id };
    events.push(GameEvent {
        time_minutes: fb_time,
        event_type: "FirstBlood".to_string(),
        description: "拿下一血".to_string(),
        team_id: fb_team,
    });

    // 一塔
    let ft_time = 8 + rng.gen_range(0..7);
    let ft_team = if rng.r#gen::<f64>() < 0.65 { winner_id } else { loser_id };
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
            let dragon_team = if rng.r#gen::<f64>() < 0.55 { winner_id } else { loser_id };
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

async fn update_player_form_factors(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    players: &[PlayerData],
    won: bool,
    avg_performance: f64,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

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
            INSERT INTO player_form_factors (save_id, player_id, form_cycle, momentum, last_performance, last_match_won, perf_history, games_since_rest, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(save_id, player_id) DO UPDATE SET
                form_cycle = excluded.form_cycle,
                momentum = excluded.momentum,
                last_performance = excluded.last_performance,
                last_match_won = excluded.last_match_won,
                perf_history = excluded.perf_history,
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
        .bind(&updated_factors.perf_history)
        .bind(updated_factors.games_since_rest as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn update_bench_form_factors(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    bench_players: &[PlayerData],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    for player in bench_players {
        let updated = ConditionEngine::update_form_factors_bench(player.form_factors.clone());

        sqlx::query(
            r#"
            INSERT INTO player_form_factors (save_id, player_id, form_cycle, momentum, last_performance, last_match_won, perf_history, games_since_rest, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(save_id, player_id) DO UPDATE SET
                form_cycle = excluded.form_cycle,
                momentum = excluded.momentum,
                games_since_rest = excluded.games_since_rest,
                updated_at = datetime('now')
            "#,
        )
        .bind(save_id)
        .bind(player.id as i64)
        .bind(updated.form_cycle)
        .bind(updated.momentum as i64)
        .bind(updated.last_performance)
        .bind(if updated.last_match_won { 1i64 } else { 0i64 })
        .bind(&updated.perf_history)
        .bind(updated.games_since_rest as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn save_player_tournament_stats(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
    tournament_id: u64,
    tournament_type: &str,
    home_team_id: u64,
    home_team_name: &str,
    away_team_id: u64,
    away_team_name: &str,
    games: &[DetailedGameResult],
    _winner_id: u64,
) -> Result<(), String> {
    use std::collections::HashMap;

    struct PlayerAggregatedStats {
        player_name: String,
        team_id: u64,
        team_name: String,
        position: String,
        games_played: u32,
        games_won: u32,
        total_impact: f64,
        max_impact: f64,
        performances: Vec<f64>,
        game_mvp_count: u32,
    }

    let mut player_stats_map: HashMap<u64, PlayerAggregatedStats> = HashMap::new();

    // 遍历所有小局，汇总选手数据
    for game in games {
        let game_winner_id = game.winner_id;

        // 处理主队选手
        for player in &game.home_players {
            let entry = player_stats_map.entry(player.player_id).or_insert(PlayerAggregatedStats {
                player_name: player.player_name.clone(),
                team_id: home_team_id,
                team_name: home_team_name.to_string(),
                position: player.position.clone(),
                games_played: 0,
                games_won: 0,
                total_impact: 0.0,
                max_impact: f64::NEG_INFINITY,
                performances: Vec::new(),
                game_mvp_count: 0,
            });

            entry.games_played += 1;
            if game_winner_id == home_team_id {
                entry.games_won += 1;
            }
            entry.total_impact += player.impact_score;
            if player.impact_score > entry.max_impact {
                entry.max_impact = player.impact_score;
            }
            entry.performances.push(player.actual_ability);

            // 检查是否是本局MVP
            if game.game_mvp.player_id == player.player_id {
                entry.game_mvp_count += 1;
            }
        }

        // 处理客队选手
        for player in &game.away_players {
            let entry = player_stats_map.entry(player.player_id).or_insert(PlayerAggregatedStats {
                player_name: player.player_name.clone(),
                team_id: away_team_id,
                team_name: away_team_name.to_string(),
                position: player.position.clone(),
                games_played: 0,
                games_won: 0,
                total_impact: 0.0,
                max_impact: f64::NEG_INFINITY,
                performances: Vec::new(),
                game_mvp_count: 0,
            });

            entry.games_played += 1;
            if game_winner_id == away_team_id {
                entry.games_won += 1;
            }
            entry.total_impact += player.impact_score;
            if player.impact_score > entry.max_impact {
                entry.max_impact = player.impact_score;
            }
            entry.performances.push(player.actual_ability);

            // 检查是否是本局MVP
            if game.game_mvp.player_id == player.player_id {
                entry.game_mvp_count += 1;
            }
        }
    }

    // 保存每个选手的汇总统计
    for (player_id, stats) in player_stats_map {
        let games_count = stats.games_played as f64;
        let avg_impact = if games_count > 0.0 { stats.total_impact / games_count } else { 0.0 };
        let avg_performance = if !stats.performances.is_empty() {
            stats.performances.iter().sum::<f64>() / stats.performances.len() as f64
        } else {
            0.0
        };
        let best_performance = stats.performances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let max_impact = if stats.max_impact == f64::NEG_INFINITY { 0.0 } else { stats.max_impact };
        let best_perf_final = if best_performance == f64::NEG_INFINITY { 0.0 } else { best_performance };

        let player_stats = PlayerTournamentStats {
            id: 0,
            save_id: save_id.to_string(),
            season_id,
            tournament_id,
            tournament_type: tournament_type.to_string(),
            player_id,
            player_name: stats.player_name,
            team_id: stats.team_id,
            team_name: stats.team_name,
            position: stats.position,
            games_played: stats.games_played,
            games_won: stats.games_won,
            total_impact: stats.total_impact,
            avg_impact,
            max_impact,
            avg_performance,
            best_performance: best_perf_final,
            game_mvp_count: stats.game_mvp_count,
            created_at: None,
            updated_at: None,
        };

        // 使用 upsert 保存或更新（累加统计）
        // 由于 upsert 会覆盖而非累加，我们需要先获取现有数据再合并
        let existing = PlayerTournamentStatsRepository::get_by_player_tournament(
            pool, save_id, tournament_id, player_id
        ).await;

        let final_stats = if let Ok(Some(existing_stats)) = existing {
            // 合并现有数据
            let total_games = existing_stats.games_played + stats.games_played;
            let total_won = existing_stats.games_won + stats.games_won;
            let combined_total_impact = existing_stats.total_impact + stats.total_impact;
            let combined_avg_impact = if total_games > 0 {
                combined_total_impact / total_games as f64
            } else {
                0.0
            };
            let combined_max_impact = existing_stats.max_impact.max(max_impact);
            // 对于平均发挥值，做加权平均
            let combined_avg_perf = if total_games > 0 {
                (existing_stats.avg_performance * existing_stats.games_played as f64
                    + avg_performance * stats.games_played as f64) / total_games as f64
            } else {
                0.0
            };
            let combined_best_perf = existing_stats.best_performance.max(best_perf_final);
            let combined_mvp_count = existing_stats.game_mvp_count + stats.game_mvp_count;

            PlayerTournamentStats {
                id: existing_stats.id,
                save_id: save_id.to_string(),
                season_id,
                tournament_id,
                tournament_type: tournament_type.to_string(),
                player_id,
                player_name: player_stats.player_name,
                team_id: player_stats.team_id,
                team_name: player_stats.team_name,
                position: player_stats.position,
                games_played: total_games,
                games_won: total_won,
                total_impact: combined_total_impact,
                avg_impact: combined_avg_impact,
                max_impact: combined_max_impact,
                avg_performance: combined_avg_perf,
                best_performance: combined_best_perf,
                game_mvp_count: combined_mvp_count,
                created_at: None,
                updated_at: None,
            }
        } else {
            player_stats
        };

        if let Err(e) = PlayerTournamentStatsRepository::upsert(pool, &final_stats).await {
            log::error!("[save_player_tournament_stats] 保存选手 {} 统计失败: {}", player_id, e);
        }
    }

    Ok(())
}
