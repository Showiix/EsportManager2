use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{SaveRepository, MatchRepository, TeamRepository, TournamentRepository, PlayerTournamentStatsRepository};
use crate::models::{
    CompleteAndAdvanceResult, FastForwardResult, FastForwardTarget, GameTimeState, SeasonPhase,
    PlayerTournamentStats,
};
use crate::services::{GameFlowService, LeagueService};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;
use rand::{Rng, SeedableRng, rngs::StdRng};

/// 快进目标请求
#[derive(Debug, Serialize, Deserialize)]
pub struct FastForwardRequest {
    pub target: String, // "NEXT_PHASE", "SUMMER", "WORLDS", "SEASON_END", 或具体阶段名
}

/// 获取完整的游戏时间状态
#[tauri::command]
pub async fn get_time_state(
    state: State<'_, AppState>,
) -> Result<CommandResult<GameTimeState>, String> {
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

    let game_flow = GameFlowService::new();
    match game_flow.get_time_state(&pool, &save_id).await {
        Ok(time_state) => Ok(CommandResult::ok(time_state)),
        Err(e) => Ok(CommandResult::err(format!("Failed to get time state: {}", e))),
    }
}

/// 初始化当前阶段（创建赛事）
#[tauri::command]
pub async fn time_init_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<String>, String> {
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

    // 获取当前存档信息
    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    let game_flow = GameFlowService::new();
    match game_flow
        .initialize_phase(&pool, &save_id, save.current_season as u64, save.current_phase)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result.message)),
        Err(e) => Ok(CommandResult::err(format!(
            "Failed to initialize phase: {}",
            e
        ))),
    }
}

/// 完成当前阶段并推进到下一阶段
#[tauri::command]
pub async fn complete_and_advance(
    state: State<'_, AppState>,
) -> Result<CommandResult<CompleteAndAdvanceResult>, String> {
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

    let game_flow = GameFlowService::new();
    match game_flow.complete_and_advance(&pool, &save_id).await {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!(
            "Failed to complete and advance: {}",
            e
        ))),
    }
}

/// 快进到指定目标
#[tauri::command]
pub async fn fast_forward_to(
    state: State<'_, AppState>,
    target: String,
) -> Result<CommandResult<FastForwardResult>, String> {
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

    // 解析快进目标
    let ff_target = match target.to_uppercase().as_str() {
        "NEXT_PHASE" => FastForwardTarget::NextPhase,
        "SUMMER" => FastForwardTarget::ToPhase(SeasonPhase::SummerRegular),
        "WORLDS" => FastForwardTarget::ToPhase(SeasonPhase::WorldChampionship),
        "SEASON_END" => FastForwardTarget::SeasonEnd,
        "SPRING_REGULAR" => FastForwardTarget::ToPhase(SeasonPhase::SpringRegular),
        "SPRING_PLAYOFFS" => FastForwardTarget::ToPhase(SeasonPhase::SpringPlayoffs),
        "MSI" => FastForwardTarget::ToPhase(SeasonPhase::Msi),
        "MADRID_MASTERS" => FastForwardTarget::ToPhase(SeasonPhase::MadridMasters),
        "SUMMER_REGULAR" => FastForwardTarget::ToPhase(SeasonPhase::SummerRegular),
        "SUMMER_PLAYOFFS" => FastForwardTarget::ToPhase(SeasonPhase::SummerPlayoffs),
        "CLAUDE_INTERCONTINENTAL" => FastForwardTarget::ToPhase(SeasonPhase::ClaudeIntercontinental),
        "WORLD_CHAMPIONSHIP" => FastForwardTarget::ToPhase(SeasonPhase::WorldChampionship),
        "SHANGHAI_MASTERS" => FastForwardTarget::ToPhase(SeasonPhase::ShanghaiMasters),
        "ICP_INTERCONTINENTAL" => FastForwardTarget::ToPhase(SeasonPhase::IcpIntercontinental),
        "SUPER_INTERCONTINENTAL" => FastForwardTarget::ToPhase(SeasonPhase::SuperIntercontinental),
        "TRANSFER_WINDOW" => FastForwardTarget::ToPhase(SeasonPhase::TransferWindow),
        "DRAFT" => FastForwardTarget::ToPhase(SeasonPhase::Draft),
        _ => return Ok(CommandResult::err(format!("Invalid target: {}", target))),
    };

    let game_flow = GameFlowService::new();
    match game_flow.fast_forward_to(&pool, &save_id, ff_target).await {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to fast forward: {}", e))),
    }
}

/// 模拟所有当前阶段的比赛
#[tauri::command]
pub async fn time_simulate_all(
    state: State<'_, AppState>,
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

    // 获取当前存档
    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    // 使用快进到下一阶段的方式来模拟所有比赛
    let game_flow = GameFlowService::new();
    let league_service = LeagueService::new();

    // 先获取当前状态
    let time_state = match game_flow.get_time_state(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get time state: {}", e))),
    };

    // 如果阶段已完成，返回0
    if time_state.phase_progress.completed_matches >= time_state.phase_progress.total_matches
        && time_state.phase_progress.total_matches > 0
    {
        return Ok(CommandResult::ok(0));
    }

    // 检查是否是季后赛阶段
    let is_playoff_phase = matches!(
        save.current_phase,
        SeasonPhase::SpringPlayoffs | SeasonPhase::SummerPlayoffs
    );

    if is_playoff_phase {
        // 季后赛：逐场模拟以确保正确生成后续对阵
        let mut simulated_count = 0u32;
        let tournament_type = match save.current_phase {
            SeasonPhase::SpringPlayoffs => "SpringPlayoffs",
            SeasonPhase::SummerPlayoffs => "SummerPlayoffs",
            _ => return Ok(CommandResult::err("Invalid playoff phase")),
        };

        // 获取所有季后赛赛事
        let tournaments = TournamentRepository::get_by_season_and_type(
            &pool,
            &save_id,
            save.current_season as u64,
            tournament_type,
        )
        .await
        .map_err(|e| e.to_string())?;

        // 循环模拟直到所有比赛完成
        loop {
            let mut found_pending = false;

            for tournament in &tournaments {
                // 获取待进行的比赛
                let pending = MatchRepository::get_pending(&pool, &save_id, tournament.id)
                    .await
                    .map_err(|e| e.to_string())?;

                if pending.is_empty() {
                    continue;
                }

                found_pending = true;
                let match_info = &pending[0];

                // 获取队伍并模拟比赛
                let home_team = TeamRepository::get_by_id(&pool, match_info.home_team_id)
                    .await
                    .map_err(|e| e.to_string())?;
                let away_team = TeamRepository::get_by_id(&pool, match_info.away_team_id)
                    .await
                    .map_err(|e| e.to_string())?;

                let result = league_service.simulate_match(
                    match_info,
                    home_team.power_rating,
                    away_team.power_rating,
                );

                // 更新比赛结果
                MatchRepository::update_result(
                    &pool,
                    match_info.id,
                    result.home_score as u32,
                    result.away_score as u32,
                    result.winner_id,
                )
                .await
                .map_err(|e| e.to_string())?;

                simulated_count += 1;

                // 检查并生成下一轮对阵
                let all_matches = MatchRepository::get_by_tournament(&pool, tournament.id)
                    .await
                    .map_err(|e| e.to_string())?;

                let new_matches =
                    league_service.advance_playoff_bracket(tournament.id, &all_matches);

                if !new_matches.is_empty() {
                    println!(
                        "[time_simulate_all] 季后赛生成 {} 场新比赛",
                        new_matches.len()
                    );
                    MatchRepository::create_batch(&pool, &save_id, &new_matches)
                        .await
                        .map_err(|e| e.to_string())?;
                }

                break; // 每次只模拟一场，然后重新检查
            }

            if !found_pending {
                break;
            }
        }

        Ok(CommandResult::ok(simulated_count))
    } else {
        // 非季后赛：直接批量更新
        // 需要为每场比赛生成合理的比分
        let tournament_type = match save.current_phase {
            SeasonPhase::SpringRegular => "SpringRegular",
            SeasonPhase::SummerRegular => "SummerRegular",
            SeasonPhase::Msi => "Msi",
            SeasonPhase::MadridMasters => "MadridMasters",
            SeasonPhase::ClaudeIntercontinental => "ClaudeIntercontinental",
            SeasonPhase::WorldChampionship => "WorldChampionship",
            SeasonPhase::ShanghaiMasters => "ShanghaiMasters",
            SeasonPhase::IcpIntercontinental => "IcpIntercontinental",
            SeasonPhase::SuperIntercontinental => "SuperIntercontinental",
            _ => return Ok(CommandResult::err("当前阶段没有比赛可模拟")),
        };

        let tournaments = TournamentRepository::get_by_season_and_type(
            &pool,
            &save_id,
            save.current_season as u64,
            tournament_type,
        )
        .await
        .map_err(|e| e.to_string())?;

        let mut simulated_count = 0u32;

        for tournament in &tournaments {
            let pending = MatchRepository::get_pending(&pool, &save_id, tournament.id)
                .await
                .map_err(|e| e.to_string())?;

            for match_info in &pending {
                let home_team = TeamRepository::get_by_id(&pool, match_info.home_team_id)
                    .await
                    .map_err(|e| e.to_string())?;
                let away_team = TeamRepository::get_by_id(&pool, match_info.away_team_id)
                    .await
                    .map_err(|e| e.to_string())?;

                let result = league_service.simulate_match(
                    match_info,
                    home_team.power_rating,
                    away_team.power_rating,
                );

                MatchRepository::update_result(
                    &pool,
                    match_info.id,
                    result.home_score as u32,
                    result.away_score as u32,
                    result.winner_id,
                )
                .await
                .map_err(|e| e.to_string())?;

                simulated_count += 1;
            }
        }

        Ok(CommandResult::ok(simulated_count))
    }
}

/// 执行赛季结算
#[tauri::command]
pub async fn time_season_settlement(
    state: State<'_, AppState>,
) -> Result<CommandResult<crate::services::SeasonSettlementResult>, String> {
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

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    // 检查是否在赛季结束阶段
    if save.current_phase != SeasonPhase::SeasonEnd {
        return Ok(CommandResult::err(
            "Season settlement can only be executed at SeasonEnd phase",
        ));
    }

    let game_flow = GameFlowService::new();
    match game_flow
        .execute_season_settlement(&pool, &save_id, save.current_season)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!(
            "Failed to execute season settlement: {}",
            e
        ))),
    }
}

/// 推进到新赛季
#[tauri::command]
pub async fn time_start_new_season(
    state: State<'_, AppState>,
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

    let game_flow = GameFlowService::new();
    match game_flow.advance_to_new_season(&pool, &save_id).await {
        Ok(new_season) => Ok(CommandResult::ok(new_season)),
        Err(e) => Ok(CommandResult::err(format!(
            "Failed to start new season: {}",
            e
        ))),
    }
}

/// 单场比赛模拟结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateNextResult {
    pub match_id: u64,
    pub tournament_name: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub home_score: u32,
    pub away_score: u32,
    pub winner_name: String,
    pub remaining_matches: u32,
    pub phase_completed: bool,
}

/// 模拟当前阶段的下一场比赛
#[tauri::command]
pub async fn time_simulate_next(
    state: State<'_, AppState>,
) -> Result<CommandResult<SimulateNextResult>, String> {
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

    // 获取当前存档状态
    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    // 获取当前阶段对应的赛事类型
    let tournament_type = match save.current_phase {
        SeasonPhase::SpringRegular => "SpringRegular",
        SeasonPhase::SpringPlayoffs => "SpringPlayoffs",
        SeasonPhase::Msi => "Msi",
        SeasonPhase::MadridMasters => "MadridMasters",
        SeasonPhase::SummerRegular => "SummerRegular",
        SeasonPhase::SummerPlayoffs => "SummerPlayoffs",
        SeasonPhase::ClaudeIntercontinental => "ClaudeIntercontinental",
        SeasonPhase::WorldChampionship => "WorldChampionship",
        SeasonPhase::ShanghaiMasters => "ShanghaiMasters",
        SeasonPhase::IcpIntercontinental => "IcpIntercontinental",
        SeasonPhase::SuperIntercontinental => "SuperIntercontinental",
        _ => return Ok(CommandResult::err("当前阶段没有比赛可模拟")),
    };

    // 查找当前阶段的所有赛事
    let tournaments = TournamentRepository::get_by_season_and_type(
        &pool, &save_id, save.current_season as u64, tournament_type
    ).await.map_err(|e| e.to_string())?;

    if tournaments.is_empty() {
        return Ok(CommandResult::err("当前阶段没有赛事"));
    }

    // 遍历所有赛事找到第一场待进行的比赛
    let mut pending_match = None;
    let mut tournament_name = String::new();

    for tournament in &tournaments {
        let pending = MatchRepository::get_pending(&pool, &save_id, tournament.id)
            .await
            .map_err(|e| e.to_string())?;

        if !pending.is_empty() {
            pending_match = Some(pending[0].clone());
            tournament_name = tournament.name.clone();
            break;
        }
    }

    let match_info = match pending_match {
        Some(m) => m,
        None => return Ok(CommandResult::err("当前阶段所有比赛已完成")),
    };

    // 获取队伍信息
    let home_team = TeamRepository::get_by_id(&pool, match_info.home_team_id)
        .await
        .map_err(|e| e.to_string())?;
    let away_team = TeamRepository::get_by_id(&pool, match_info.away_team_id)
        .await
        .map_err(|e| e.to_string())?;

    // 模拟比赛
    let league_service = LeagueService::new();
    let result = league_service.simulate_match(&match_info, home_team.power_rating, away_team.power_rating);

    // 更新比赛结果
    MatchRepository::update_result(
        &pool,
        match_info.id,
        result.home_score as u32,
        result.away_score as u32,
        result.winner_id,
    ).await.map_err(|e| e.to_string())?;

    // 保存选手赛事统计（用于MVP计算）
    let _ = save_quick_player_stats(
        &pool,
        &save_id,
        save.current_season as u64,
        match_info.tournament_id,
        tournament_type,  // 使用正确的 tournament_type 而不是 tournament_name
        home_team.id,
        &home_team.name,
        away_team.id,
        &away_team.name,
        result.home_score,
        result.away_score,
        result.winner_id,
    ).await;

    // 如果是季后赛比赛，检查并生成下一轮对阵
    let is_playoff = match_info.stage.contains("WINNERS")
        || match_info.stage.contains("LOSERS")
        || match_info.stage.contains("GRAND_FINAL");

    if is_playoff {
        // 获取该赛事的所有比赛
        let all_matches = MatchRepository::get_by_tournament(&pool, match_info.tournament_id)
            .await
            .map_err(|e| e.to_string())?;

        // 检查是否需要生成新比赛
        let new_matches = league_service.advance_playoff_bracket(match_info.tournament_id, &all_matches);

        if !new_matches.is_empty() {
            println!("[Playoffs] 生成 {} 场新比赛", new_matches.len());
            MatchRepository::create_batch(&pool, &save_id, &new_matches)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    // 计算剩余比赛数
    let mut remaining = 0u32;
    for tournament in &tournaments {
        let pending = MatchRepository::get_pending(&pool, &save_id, tournament.id)
            .await
            .map_err(|e| e.to_string())?;
        remaining += pending.len() as u32;
    }
    // 减去刚模拟的这场
    remaining = remaining.saturating_sub(1);

    let winner_name = if result.winner_id == home_team.id {
        home_team.name.clone()
    } else {
        away_team.name.clone()
    };

    Ok(CommandResult::ok(SimulateNextResult {
        match_id: match_info.id,
        tournament_name,
        home_team_name: home_team.name,
        away_team_name: away_team.name,
        home_score: result.home_score as u32,
        away_score: result.away_score as u32,
        winner_name,
        remaining_matches: remaining,
        phase_completed: remaining == 0,
    }))
}

/// 简化版选手信息
struct QuickPlayerInfo {
    id: u64,
    game_id: String,
    position: String,
    ability: u8,
}

/// 保存简化的选手赛事统计（用于快速模拟）
async fn save_quick_player_stats(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
    tournament_id: u64,
    tournament_type: &str,
    home_team_id: u64,
    home_team_name: &str,
    away_team_id: u64,
    away_team_name: &str,
    home_score: u8,
    away_score: u8,
    winner_id: u64,
) -> Result<(), String> {
    // 获取双方首发选手
    let home_players = get_quick_starters(pool, save_id, home_team_id).await?;
    let away_players = get_quick_starters(pool, save_id, away_team_id).await?;

    if home_players.is_empty() || away_players.is_empty() {
        return Ok(()); // 没有首发选手数据，跳过
    }

    let total_games = (home_score + away_score) as u32;
    let mut rng = StdRng::from_entropy();

    // 为每个选手生成简化的统计数据
    for player in &home_players {
        let won = winner_id == home_team_id;
        let games_won = if won { home_score as u32 } else { away_score as u32 };

        // 基于能力值生成影响力
        let base_impact = player.ability as f64 * 0.8 + rng.gen_range(0.0..20.0);
        let total_impact = base_impact * total_games as f64;
        let avg_impact = base_impact;
        let max_impact = base_impact + rng.gen_range(0.0..10.0);
        let avg_performance = player.ability as f64 + rng.gen_range(-5.0..5.0);
        let best_performance = avg_performance + rng.gen_range(0.0..8.0);

        // 从胜方随机选择MVP（每局一个）
        let game_mvp_count = if won {
            // 胜方队伍的选手有机会获得MVP
            let mut mvp_count = 0u32;
            for _ in 0..total_games {
                // 约20%概率获得本局MVP（5个选手平分）
                if rng.gen_range(0..5) == 0 {
                    mvp_count += 1;
                }
            }
            mvp_count
        } else {
            0
        };

        save_or_update_player_stats(
            pool,
            save_id,
            season_id,
            tournament_id,
            tournament_type,
            player.id,
            &player.game_id,
            home_team_id,
            home_team_name,
            &player.position,
            total_games,
            games_won,
            total_impact,
            avg_impact,
            max_impact,
            avg_performance,
            best_performance,
            game_mvp_count,
        ).await?;
    }

    for player in &away_players {
        let won = winner_id == away_team_id;
        let games_won = if won { away_score as u32 } else { home_score as u32 };

        let base_impact = player.ability as f64 * 0.8 + rng.gen_range(0.0..20.0);
        let total_impact = base_impact * total_games as f64;
        let avg_impact = base_impact;
        let max_impact = base_impact + rng.gen_range(0.0..10.0);
        let avg_performance = player.ability as f64 + rng.gen_range(-5.0..5.0);
        let best_performance = avg_performance + rng.gen_range(0.0..8.0);

        let game_mvp_count = if won {
            let mut mvp_count = 0u32;
            for _ in 0..total_games {
                if rng.gen_range(0..5) == 0 {
                    mvp_count += 1;
                }
            }
            mvp_count
        } else {
            0
        };

        save_or_update_player_stats(
            pool,
            save_id,
            season_id,
            tournament_id,
            tournament_type,
            player.id,
            &player.game_id,
            away_team_id,
            away_team_name,
            &player.position,
            total_games,
            games_won,
            total_impact,
            avg_impact,
            max_impact,
            avg_performance,
            best_performance,
            game_mvp_count,
        ).await?;
    }

    Ok(())
}

/// 获取队伍首发选手的简化信息
async fn get_quick_starters(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
) -> Result<Vec<QuickPlayerInfo>, String> {
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

    let mut players = Vec::new();
    for r in rows {
        players.push(QuickPlayerInfo {
            id: r.get::<i64, _>("id") as u64,
            game_id: r.get::<String, _>("game_id"),
            position: r.get::<Option<String>, _>("position")
                .unwrap_or_default()
                .trim_start_matches("Some(")
                .trim_end_matches(")")
                .to_string(),
            ability: r.get::<i64, _>("ability") as u8,
        });
    }

    Ok(players)
}

/// 保存或更新选手统计数据
#[allow(clippy::too_many_arguments)]
async fn save_or_update_player_stats(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
    tournament_id: u64,
    tournament_type: &str,
    player_id: u64,
    player_name: &str,
    team_id: u64,
    team_name: &str,
    position: &str,
    games_played: u32,
    games_won: u32,
    total_impact: f64,
    avg_impact: f64,
    max_impact: f64,
    avg_performance: f64,
    best_performance: f64,
    game_mvp_count: u32,
) -> Result<(), String> {
    // 查找现有数据
    let existing = PlayerTournamentStatsRepository::get_by_player_tournament(
        pool, save_id, tournament_id, player_id
    ).await;

    let final_stats = if let Ok(Some(existing_stats)) = existing {
        // 合并数据
        let total_games = existing_stats.games_played + games_played;
        let total_won = existing_stats.games_won + games_won;
        let combined_total_impact = existing_stats.total_impact + total_impact;
        let combined_avg_impact = if total_games > 0 {
            combined_total_impact / total_games as f64
        } else {
            0.0
        };
        let combined_max_impact = existing_stats.max_impact.max(max_impact);
        let combined_avg_perf = if total_games > 0 {
            (existing_stats.avg_performance * existing_stats.games_played as f64
                + avg_performance * games_played as f64) / total_games as f64
        } else {
            0.0
        };
        let combined_best_perf = existing_stats.best_performance.max(best_performance);
        let combined_mvp_count = existing_stats.game_mvp_count + game_mvp_count;

        PlayerTournamentStats {
            id: existing_stats.id,
            save_id: save_id.to_string(),
            season_id,
            tournament_id,
            tournament_type: tournament_type.to_string(),
            player_id,
            player_name: player_name.to_string(),
            team_id,
            team_name: team_name.to_string(),
            position: position.to_string(),
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
        // 创建新数据
        PlayerTournamentStats {
            id: 0,
            save_id: save_id.to_string(),
            season_id,
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
            max_impact,
            avg_performance,
            best_performance,
            game_mvp_count,
            created_at: None,
            updated_at: None,
        }
    };

    PlayerTournamentStatsRepository::upsert(pool, &final_stats)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
