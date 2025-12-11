use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{MatchRepository, SaveRepository, StandingRepository, TeamRepository, TournamentRepository};
use crate::engines::SeasonProgressEngine;
use crate::models::{SeasonPhase, TournamentStatus};
use crate::services::{
    GameFlowService, LeagueService, PhaseCompleteResult, PhaseInitResult, SeasonSettlementResult,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 游戏状态信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GameStateInfo {
    pub current_season: u32,
    pub current_phase: String,
    pub phase_name: String,
    pub progress: (u32, u32),
    pub available_actions: Vec<String>,
}

/// 比赛信息
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchInfo {
    pub id: u64,
    pub tournament_id: u64,
    pub stage: String,
    pub round: Option<u32>,
    pub format: String,
    pub home_team_id: u64,
    pub away_team_id: u64,
    pub home_score: u32,
    pub away_score: u32,
    pub winner_id: Option<u64>,
    pub status: String,
}

/// 积分榜信息
#[derive(Debug, Serialize, Deserialize)]
pub struct StandingInfo {
    pub team_id: u64,
    pub team_name: Option<String>,
    pub rank: Option<u32>,
    pub matches_played: u32,
    pub wins: u32,
    pub losses: u32,
    pub points: u32,
    pub game_diff: i32,
}

/// 获取当前游戏状态
#[tauri::command]
pub async fn get_game_state(
    state: State<'_, AppState>,
) -> Result<CommandResult<GameStateInfo>, String> {
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

    let engine = SeasonProgressEngine::new(save.current_season, save.current_phase);
    let progress = engine.get_progress();
    let actions = engine.get_available_actions();

    let action_names: Vec<String> = actions
        .iter()
        .map(|a| format!("{:?}", a))
        .collect();

    Ok(CommandResult::ok(GameStateInfo {
        current_season: save.current_season,
        current_phase: format!("{:?}", save.current_phase),
        phase_name: get_phase_display_name(&save.current_phase),
        progress,
        available_actions: action_names,
    }))
}

/// 推进到下一阶段
#[tauri::command]
pub async fn advance_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<GameStateInfo>, String> {
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

    let mut save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    // 推进阶段
    if let Some(next_phase) = save.current_phase.next() {
        save.current_phase = next_phase;
        save.updated_at = Utc::now();

        if let Err(e) = SaveRepository::update(&pool, &save).await {
            return Ok(CommandResult::err(format!("Failed to update save: {}", e)));
        }
    } else {
        // 赛季结束，开始新赛季
        save.current_season += 1;
        save.current_phase = SeasonPhase::SpringRegular;
        save.updated_at = Utc::now();

        if let Err(e) = SaveRepository::update(&pool, &save).await {
            return Ok(CommandResult::err(format!("Failed to update save: {}", e)));
        }
    }

    let engine = SeasonProgressEngine::new(save.current_season, save.current_phase);
    let progress = engine.get_progress();
    let actions = engine.get_available_actions();

    let action_names: Vec<String> = actions
        .iter()
        .map(|a| format!("{:?}", a))
        .collect();

    Ok(CommandResult::ok(GameStateInfo {
        current_season: save.current_season,
        current_phase: format!("{:?}", save.current_phase),
        phase_name: get_phase_display_name(&save.current_phase),
        progress,
        available_actions: action_names,
    }))
}

/// 获取赛事比赛列表
#[tauri::command]
pub async fn get_tournament_matches(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<MatchInfo>>, String> {
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

    let infos: Vec<MatchInfo> = matches
        .into_iter()
        .map(|m| MatchInfo {
            id: m.id,
            tournament_id: m.tournament_id,
            stage: m.stage,
            round: m.round,
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

/// 获取积分榜
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

    // 获取所有队伍信息以获取队名
    let teams = match TeamRepository::get_all(&pool, &save_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get teams: {}", e))),
    };

    // 创建 team_id -> team_name 映射
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

/// 模拟下一场比赛
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

    // 获取待进行的比赛
    let pending = match MatchRepository::get_pending(&pool, &save_id, tournament_id).await {
        Ok(m) => m,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pending matches: {}", e))),
    };

    if pending.is_empty() {
        return Ok(CommandResult::err("No pending matches"));
    }

    let match_info = &pending[0];

    // 获取队伍战力
    let home_team = match crate::db::TeamRepository::get_by_id(&pool, match_info.home_team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get home team: {}", e))),
    };

    let away_team = match crate::db::TeamRepository::get_by_id(&pool, match_info.away_team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get away team: {}", e))),
    };

    // 模拟比赛
    let league_service = LeagueService::new();
    let result = league_service.simulate_match(match_info, home_team.power_rating, away_team.power_rating);

    // 更新比赛结果
    if let Err(e) = MatchRepository::update_result(
        &pool,
        match_info.id,
        result.home_score as u32,
        result.away_score as u32,
        result.winner_id,
    ).await {
        return Ok(CommandResult::err(format!("Failed to update match: {}", e)));
    }

    Ok(CommandResult::ok(MatchInfo {
        id: match_info.id,
        tournament_id: match_info.tournament_id,
        stage: match_info.stage.clone(),
        round: match_info.round,
        format: format!("{:?}", match_info.format),
        home_team_id: match_info.home_team_id,
        away_team_id: match_info.away_team_id,
        home_score: result.home_score as u32,
        away_score: result.away_score as u32,
        winner_id: Some(result.winner_id),
        status: "Completed".to_string(),
    }))
}

/// 模拟所有剩余比赛
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

    let league_service = LeagueService::new();
    let mut simulated_count = 0u32;

    loop {
        // 获取待进行的比赛
        let pending = match MatchRepository::get_pending(&pool, &save_id, tournament_id).await {
            Ok(m) => m,
            Err(e) => return Ok(CommandResult::err(format!("Failed to get pending: {}", e))),
        };

        if pending.is_empty() {
            break;
        }

        let match_info = &pending[0];

        // 获取队伍
        let home_team = match crate::db::TeamRepository::get_by_id(&pool, match_info.home_team_id).await {
            Ok(t) => t,
            Err(_) => continue,
        };

        let away_team = match crate::db::TeamRepository::get_by_id(&pool, match_info.away_team_id).await {
            Ok(t) => t,
            Err(_) => continue,
        };

        // 模拟
        let result = league_service.simulate_match(match_info, home_team.power_rating, away_team.power_rating);

        // 更新
        if MatchRepository::update_result(
            &pool,
            match_info.id,
            result.home_score as u32,
            result.away_score as u32,
            result.winner_id,
        ).await.is_ok() {
            simulated_count += 1;
        }
    }

    // 更新赛事状态为已完成
    if let Err(e) = TournamentRepository::update_status(&pool, tournament_id, TournamentStatus::Completed).await {
        return Ok(CommandResult::err(format!("Failed to update tournament: {}", e)));
    }

    Ok(CommandResult::ok(simulated_count))
}

/// 获取阶段显示名称
fn get_phase_display_name(phase: &SeasonPhase) -> String {
    match phase {
        SeasonPhase::SpringRegular => "春季常规赛".to_string(),
        SeasonPhase::SpringPlayoffs => "春季季后赛".to_string(),
        SeasonPhase::Msi => "MSI季中赛".to_string(),
        SeasonPhase::MadridMasters => "马德里大师赛".to_string(),
        SeasonPhase::SummerRegular => "夏季常规赛".to_string(),
        SeasonPhase::SummerPlayoffs => "夏季季后赛".to_string(),
        SeasonPhase::ClaudeIntercontinental => "Claude洲际赛".to_string(),
        SeasonPhase::WorldChampionship => "世界赛".to_string(),
        SeasonPhase::ShanghaiMasters => "上海大师赛".to_string(),
        SeasonPhase::IcpIntercontinental => "ICP洲际对抗赛".to_string(),
        SeasonPhase::SuperIntercontinental => "Super洲际邀请赛".to_string(),
        SeasonPhase::TransferWindow => "转会期".to_string(),
        SeasonPhase::Draft => "选秀大会".to_string(),
        SeasonPhase::SeasonEnd => "赛季结算".to_string(),
    }
}

// ==================== 游戏流程命令 ====================

/// 初始化当前阶段 - 创建对应赛事
#[tauri::command]
pub async fn initialize_current_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<PhaseInitResult>, String> {
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

    let game_flow = GameFlowService::new();
    match game_flow
        .initialize_phase(&pool, &save_id, save.current_season as u64, save.current_phase)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to initialize phase: {}", e))),
    }
}

/// 完成当前阶段 - 颁发荣誉并准备下一阶段
#[tauri::command]
pub async fn complete_current_phase(
    state: State<'_, AppState>,
) -> Result<CommandResult<PhaseCompleteResult>, String> {
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

    let game_flow = GameFlowService::new();
    match game_flow
        .complete_phase(&pool, &save_id, save.current_season as u64, save.current_phase)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to complete phase: {}", e))),
    }
}

/// 执行赛季结算 (游戏流程版)
#[tauri::command]
pub async fn run_season_settlement(
    state: State<'_, AppState>,
) -> Result<CommandResult<SeasonSettlementResult>, String> {
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

    // 只能在赛季结束阶段执行结算
    if save.current_phase != SeasonPhase::SeasonEnd {
        return Ok(CommandResult::err("只能在赛季结算阶段执行此操作"));
    }

    let game_flow = GameFlowService::new();
    match game_flow
        .execute_season_settlement(&pool, &save_id, save.current_season)
        .await
    {
        Ok(result) => Ok(CommandResult::ok(result)),
        Err(e) => Ok(CommandResult::err(format!("Failed to execute settlement: {}", e))),
    }
}

/// 开始新赛季
#[tauri::command]
pub async fn start_new_season(
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

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    // 只能在赛季结束阶段开始新赛季
    if save.current_phase != SeasonPhase::SeasonEnd {
        return Ok(CommandResult::err("只能在赛季结算阶段开始新赛季"));
    }

    let game_flow = GameFlowService::new();
    match game_flow.advance_to_new_season(&pool, &save_id).await {
        Ok(new_season) => Ok(CommandResult::ok(new_season)),
        Err(e) => Ok(CommandResult::err(format!("Failed to start new season: {}", e))),
    }
}
