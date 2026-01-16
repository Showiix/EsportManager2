//! 简化版转会系统 Tauri 命令
//!
//! 提供前端调用的 API 接口

use crate::commands::{AppState, CommandResult};
use crate::engines::simple_transfer_engine::SimpleTransferEngine;
use crate::models::simple_transfer::*;
use std::sync::Mutex;
use tauri::State;

// 全局引擎状态
lazy_static::lazy_static! {
    static ref TRANSFER_ENGINE: Mutex<Option<SimpleTransferEngine>> = Mutex::new(None);
}

/// 初始化转会市场
#[tauri::command]
pub fn simple_init_market(
    state: State<'_, AppState>,
) -> CommandResult<TransferMarketSummary> {
    let conn = state.get_connection()?;
    let save_id = state.get_current_save_id()?;
    let season_id = state.get_current_season(&conn)?;

    // 创建新引擎
    let engine = SimpleTransferEngine::new(save_id, season_id);
    let summary = engine.summary();

    // 保存引擎状态
    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;
    *engine_guard = Some(engine);

    CommandResult::success(summary)
}

/// 获取市场状态摘要
#[tauri::command]
pub fn simple_get_market_state(
    state: State<'_, AppState>,
) -> CommandResult<TransferMarketSummary> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    match engine_guard.as_ref() {
        Some(engine) => CommandResult::success(engine.summary()),
        None => {
            // 如果引擎未初始化，创建新引擎
            drop(engine_guard);
            simple_init_market(state)
        }
    }
}

/// 执行市场分析阶段
#[tauri::command]
pub fn simple_execute_market_analysis(
    state: State<'_, AppState>,
) -> CommandResult<TransferMarketSummary> {
    let conn = state.get_connection()?;

    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_mut()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    engine.execute_market_analysis(&conn)?;

    CommandResult::success(engine.summary())
}

/// 执行策略生成阶段
#[tauri::command]
pub fn simple_execute_strategy_generation(
    state: State<'_, AppState>,
) -> CommandResult<TransferMarketSummary> {
    let conn = state.get_connection()?;

    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_mut()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    engine.execute_strategy_generation(&conn)?;

    CommandResult::success(engine.summary())
}

/// 执行续约窗口阶段
#[tauri::command]
pub fn simple_execute_renewal_window(
    state: State<'_, AppState>,
) -> CommandResult<Vec<RenewalResult>> {
    let conn = state.get_connection()?;

    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_mut()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let results = engine.execute_renewal_window(&conn)?;

    CommandResult::success(results)
}

/// 执行一轮自由市场
#[tauri::command]
pub fn simple_execute_market_round(
    state: State<'_, AppState>,
) -> CommandResult<RoundResult> {
    let conn = state.get_connection()?;

    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_mut()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let result = engine.execute_free_market_round(&conn)?;

    CommandResult::success(result)
}

/// 快进完成自由市场
#[tauri::command]
pub fn simple_fast_forward_market(
    state: State<'_, AppState>,
) -> CommandResult<Vec<RoundResult>> {
    let conn = state.get_connection()?;

    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_mut()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let results = engine.fast_forward(&conn)?;

    CommandResult::success(results)
}

/// 获取所有选手转会信息
#[tauri::command]
pub fn simple_get_all_players(
    _state: State<'_, AppState>,
) -> CommandResult<Vec<PlayerTransferInfo>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let players: Vec<PlayerTransferInfo> = engine.players()
        .into_iter()
        .cloned()
        .collect();

    CommandResult::success(players)
}

/// 获取所有球队策略
#[tauri::command]
pub fn simple_get_all_team_strategies(
    _state: State<'_, AppState>,
) -> CommandResult<Vec<TeamTransferStrategy>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let teams: Vec<TeamTransferStrategy> = engine.teams()
        .into_iter()
        .cloned()
        .collect();

    CommandResult::success(teams)
}

/// 获取单个球队策略
#[tauri::command]
pub fn simple_get_team_strategy(
    _state: State<'_, AppState>,
    team_id: u64,
) -> CommandResult<Option<TeamTransferStrategy>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let strategy = engine.get_team(team_id).cloned();

    CommandResult::success(strategy)
}

/// 获取单个选手转会信息
#[tauri::command]
pub fn simple_get_player_transfer_info(
    _state: State<'_, AppState>,
    player_id: u64,
) -> CommandResult<Option<PlayerTransferInfo>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let player = engine.get_player(player_id).cloned();

    CommandResult::success(player)
}

/// 获取所有转会事件
#[tauri::command]
pub fn simple_get_all_events(
    _state: State<'_, AppState>,
) -> CommandResult<Vec<TransferEvent>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let events: Vec<TransferEvent> = engine.events().to_vec();

    CommandResult::success(events)
}

/// 获取指定轮次的事件
#[tauri::command]
pub fn simple_get_events_by_round(
    _state: State<'_, AppState>,
    round: u32,
) -> CommandResult<Vec<TransferEvent>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let events: Vec<TransferEvent> = engine.events()
        .iter()
        .filter(|e| e.round == round)
        .cloned()
        .collect();

    CommandResult::success(events)
}

/// 重置转会市场
#[tauri::command]
pub fn simple_reset_market(
    _state: State<'_, AppState>,
) -> CommandResult<()> {
    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;
    *engine_guard = None;

    CommandResult::success(())
}

/// 执行下一步（自动判断当前阶段）
#[tauri::command]
pub fn simple_execute_next_step(
    state: State<'_, AppState>,
) -> CommandResult<TransferMarketSummary> {
    let conn = state.get_connection()?;

    let mut engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    // 如果引擎未初始化，先初始化
    if engine_guard.is_none() {
        let save_id = state.get_current_save_id()?;
        let season_id = state.get_current_season(&conn)?;
        *engine_guard = Some(SimpleTransferEngine::new(save_id, season_id));
    }

    let engine = engine_guard.as_mut().unwrap();

    match engine.state().phase {
        TransferPhase::MarketAnalysis => {
            engine.execute_market_analysis(&conn)?;
        }
        TransferPhase::StrategyGeneration => {
            engine.execute_strategy_generation(&conn)?;
        }
        TransferPhase::RenewalWindow => {
            engine.execute_renewal_window(&conn)?;
        }
        TransferPhase::FreeMarket => {
            engine.execute_free_market_round(&conn)?;
        }
        TransferPhase::Completed => {
            // 已完成，不做任何操作
        }
    }

    CommandResult::success(engine.summary())
}

/// 获取自由球员列表
#[tauri::command]
pub fn simple_get_free_agents(
    _state: State<'_, AppState>,
) -> CommandResult<Vec<PlayerTransferInfo>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let free_agents: Vec<PlayerTransferInfo> = engine.players()
        .into_iter()
        .filter(|p| p.status == PlayerMarketStatus::FreeAgent)
        .cloned()
        .collect();

    CommandResult::success(free_agents)
}

/// 获取愿意转会的选手列表
#[tauri::command]
pub fn simple_get_willing_to_transfer(
    _state: State<'_, AppState>,
) -> CommandResult<Vec<PlayerTransferInfo>> {
    let engine_guard = TRANSFER_ENGINE.lock().map_err(|e| e.to_string())?;

    let engine = engine_guard.as_ref()
        .ok_or_else(|| "转会市场未初始化".to_string())?;

    let willing: Vec<PlayerTransferInfo> = engine.players()
        .into_iter()
        .filter(|p| p.status == PlayerMarketStatus::WillingToTransfer || p.intent.accepts_offers())
        .cloned()
        .collect();

    CommandResult::success(willing)
}
