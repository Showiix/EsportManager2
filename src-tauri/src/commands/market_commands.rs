//! LLM 转会市场系统 Tauri 命令接口
//!
//! 提供以下功能：
//! - 市场状态管理（初始化、推进、查询）
//! - 谈判管理（查询活跃谈判、谈判详情）
//! - 进度事件发送（选手意愿生成、球队策略生成）
//! - 事件查询

use crate::commands::ApiResponse;
use crate::commands::save_commands::AppState;
use crate::db::{TeamRepository, PlayerRepository, PointsRepository, LLMTaskLogRepository};
use crate::engines::TransferMarketEngine;
use crate::engines::transfer::FreeAgentInfo;
use crate::engines::transfer_market::{SigningInfo, PlayerDecision};
use crate::models::{
    Player, PlayerStatus, Team, TeamGMProfile, GMPersonality,
    TransferMarketState, TeamMarketState, MarketPhase, MarketStateSummary, TeamMarketSummary,
    Negotiation, NegotiationStatus, NegotiationListInfo, NegotiationDetailInfo,
    MarketEvent, MarketEventType, RoundExecutionResult, GenerationProgress,
    PlayerTransferStrategy, AITransferStrategy, FreeAgent, FreeAgentReason, FreeAgentStatus,
    TeamOffer, Offer, OfferStatus,
    TaskType, TaskStatus, LLMTaskLog, TaskStats,
};
use crate::services::llm_service::{
    LLMTransferService, TeamInfo, PlayerHonorInfo, PlayerPerformanceInfo,
    TeamHonorInfo, RosterPlayerHonorSummary, PlayerPerformanceSummary,
    get_llm_config, is_llm_configured, create_llm_service,
};
use crate::services::ai_transfer_service::AITransferService;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use futures::future::join_all;

// ==================== 全局引擎状态 ====================

/// 全局市场引擎实例
static MARKET_ENGINE: Lazy<Arc<RwLock<Option<TransferMarketEngine>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(None))
});

/// 存储失败的球队ID列表（用于重试）
static FAILED_TEAM_IDS: Lazy<Arc<RwLock<Vec<u64>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Vec::new()))
});

// ==================== 进度事件 ====================

/// 选手意愿生成进度事件
#[derive(Clone, Serialize)]
pub struct IntentionGenerationProgress {
    pub current: u32,
    pub total: u32,
    pub player_name: String,
    pub status: String, // "generating", "success", "failed", "completed"
    pub wants_to_leave: Option<bool>,
}

/// 球队策略生成进度事件
#[derive(Clone, Serialize)]
pub struct StrategyGenerationProgressEvent {
    pub current: u32,
    pub total: u32,
    pub team_name: String,
    pub status: String,
}

/// 轮次执行进度事件
#[derive(Clone, Serialize)]
pub struct RoundProgressEvent {
    pub phase: String,
    pub round: u8,
    pub step: String, // "offers", "responses", "signings"
    pub current: u32,
    pub total: u32,
}

// ==================== 市场状态命令 ====================

/// 初始化 LLM 转会市场
#[tauri::command]
pub async fn init_llm_transfer_market(
    state: State<'_, AppState>,
) -> Result<ApiResponse<MarketStateSummary>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 检查是否已有市场状态
    let existing = sqlx::query(
        "SELECT * FROM transfer_market_states WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(ApiResponse::error("该赛季转会市场已初始化"));
    }

    // 获取所有球队
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    // 获取所有选手（按球队分组）
    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in &players {
        if player.status == PlayerStatus::Active {
            if let Some(team_id) = player.team_id {
                players_by_team.entry(team_id).or_default().push(player.clone());
            }
        }
    }

    // 获取自由球员
    let fa_players: Vec<Player> = players.iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    let free_agents: Vec<FreeAgentInfo> = fa_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: FreeAgent {
                    id: 0,
                    save_id: save_id.clone(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: FreeAgentReason::ContractExpire,
                    status: FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 创建 LLM 服务
    let llm_service = if is_llm_configured() {
        get_llm_config().map(LLMTransferService::new)
    } else {
        None
    };

    // 创建市场引擎
    let mut engine = TransferMarketEngine::new(
        save_id.clone(),
        season_id as u64,
        llm_service,
    );

    // 初始化市场
    engine.initialize(&teams, &players_by_team, &free_agents);

    // 保存市场状态到数据库
    save_market_state_to_db(&pool, &engine.state).await?;

    let summary = engine.get_state_summary();

    // 保存引擎实例
    let mut engine_guard = MARKET_ENGINE.write().await;
    *engine_guard = Some(engine);

    Ok(ApiResponse::success(summary))
}

/// 获取 LLM 转会市场状态
#[tauri::command]
pub async fn get_llm_market_state(
    state: State<'_, AppState>,
) -> Result<ApiResponse<MarketStateSummary>, String> {
    // 先尝试从内存中获取
    let engine_guard = MARKET_ENGINE.read().await;
    if let Some(engine) = engine_guard.as_ref() {
        return Ok(ApiResponse::success(engine.get_state_summary()));
    }
    drop(engine_guard);

    // 否则从数据库加载
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 查询市场状态
    let state_row = sqlx::query(
        "SELECT * FROM transfer_market_states WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match state_row {
        Some(row) => {
            let market_state = parse_market_state_from_row(&row)?;

            // 从数据库恢复引擎到内存
            restore_engine_from_db(&pool, &save_id, season_id, market_state.clone()).await?;

            Ok(ApiResponse::success((&market_state).into()))
        }
        None => Ok(ApiResponse::error("转会市场未初始化")),
    }
}

/// 从数据库恢复引擎到内存
async fn restore_engine_from_db(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: i64,
    market_state: TransferMarketState,
) -> Result<(), String> {
    // 检查引擎是否已存在
    {
        let engine_guard = MARKET_ENGINE.read().await;
        if engine_guard.is_some() {
            return Ok(());
        }
    }

    log::info!("从数据库恢复市场引擎...");

    // 获取球队和选手数据
    let teams = TeamRepository::get_all(pool, save_id).await
        .map_err(|e| e.to_string())?;

    let players = PlayerRepository::get_all_active(pool, save_id).await
        .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in &players {
        if player.status == PlayerStatus::Active {
            if let Some(team_id) = player.team_id {
                players_by_team.entry(team_id).or_default().push(player.clone());
            }
        }
    }

    // 获取自由球员
    let fa_players: Vec<Player> = players.iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    let free_agents: Vec<FreeAgentInfo> = fa_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: FreeAgent {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: FreeAgentReason::ContractExpire,
                    status: FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 创建 LLM 服务
    let llm_service = if is_llm_configured() {
        get_llm_config().map(LLMTransferService::new)
    } else {
        None
    };

    // 创建市场引擎并恢复状态
    let mut engine = TransferMarketEngine::new(
        save_id.to_string(),
        season_id as u64,
        llm_service,
    );

    // 初始化球队状态
    for team in &teams {
        let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
        let active_count = roster.iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .count() as u8;

        let team_state = TeamMarketState::new(
            team.id,
            team.name.clone(),
            team.balance,
            active_count,
        );
        engine.state.init_team_state(team_state);
    }

    // 恢复市场状态
    engine.state.current_phase = market_state.current_phase;
    engine.state.current_round = market_state.current_round;
    engine.state.free_agent_ids = market_state.free_agent_ids;
    engine.state.poachable_player_ids = market_state.poachable_player_ids;  // ⭐ 恢复可挖角选手列表
    engine.state.active_negotiation_ids = market_state.active_negotiation_ids;
    engine.state.completed_transfer_ids = market_state.completed_transfer_ids;
    engine.state.intentions_generated = market_state.intentions_generated;
    engine.state.total_players = market_state.total_players;
    engine.state.strategies_generated = market_state.strategies_generated;
    engine.state.total_teams = market_state.total_teams;
    engine.state.is_market_stable = market_state.is_market_stable;
    engine.state.stable_rounds_count = market_state.stable_rounds_count;
    engine.state.is_transfer_stable = market_state.is_transfer_stable;  // ⭐ 恢复挖角稳定状态
    engine.state.transfer_stable_rounds_count = market_state.transfer_stable_rounds_count;
    engine.state.transfer_round = market_state.transfer_round;  // ⭐ 恢复挖角轮次

    log::info!(
        "市场状态已恢复: phase={:?}, round={}, free_agents={}, poachable={}, transfer_round={}",
        engine.state.current_phase,
        engine.state.current_round,
        engine.state.free_agent_ids.len(),
        engine.state.poachable_player_ids.len(),
        engine.state.transfer_round
    );

    // 加载选手策略
    let strategy_rows = sqlx::query(
        "SELECT player_id, strategy_json FROM player_transfer_strategies WHERE save_id = ? AND season_id = ?"
    )
    .bind(save_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in strategy_rows {
        let player_id: i64 = row.get("player_id");
        let strategy_json: String = row.get("strategy_json");
        if let Ok(strategy) = serde_json::from_str::<PlayerTransferStrategy>(&strategy_json) {
            engine.player_strategies.insert(player_id as u64, strategy);
        }
    }

    // 加载球队策略
    let team_strategy_rows = sqlx::query(
        "SELECT team_id, strategy_json FROM ai_transfer_strategies WHERE save_id = ? AND season_id = ?"
    )
    .bind(save_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in team_strategy_rows {
        let team_id: i64 = row.get("team_id");
        let strategy_json: String = row.get("strategy_json");
        if let Ok(strategy) = serde_json::from_str::<AITransferStrategy>(&strategy_json) {
            engine.team_strategies.insert(team_id as u64, strategy);
            // 同时更新 team_state 的 strategy_generated 标志
            if let Some(team_state) = engine.state.get_team_state_mut(team_id as u64) {
                team_state.strategy_generated = true;
            }
        }
    }

    // 加载事件
    let event_rows = sqlx::query(
        "SELECT * FROM transfer_market_events WHERE save_id = ? AND season_id = ? ORDER BY id"
    )
    .bind(save_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for row in event_rows {
        let event_type_str: String = row.get("event_type");
        let phase_str: String = row.get("phase");

        let event_type = match event_type_str.as_str() {
            "OFFERMADE" | "OFFER_MADE" => MarketEventType::OfferMade,
            "SIGNINGCOMPLETED" | "SIGNING_COMPLETED" => MarketEventType::SigningCompleted,
            "TEAMTHINKING" | "TEAM_THINKING" => MarketEventType::TeamThinking,
            "PLAYERTHINKING" | "PLAYER_THINKING" => MarketEventType::PlayerThinking,
            "TRANSFERREQUESTED" | "TRANSFER_REQUESTED" => MarketEventType::TransferRequested,
            "RENEWALSUCCESSFUL" | "RENEWAL_SUCCESSFUL" => MarketEventType::RenewalSuccessful,
            "RENEWALFAILED" | "RENEWAL_FAILED" => MarketEventType::RenewalFailed,
            "CONTRACTEXPIRED" | "CONTRACT_EXPIRED" => MarketEventType::ContractExpired,
            "OFFERACCEPTED" | "OFFER_ACCEPTED" => MarketEventType::OfferAccepted,
            "OFFERREJECTED" | "OFFER_REJECTED" => MarketEventType::OfferRejected,
            _ => MarketEventType::ContractExpired, // 默认值
        };

        let phase = match phase_str.as_str() {
            // 新的 5 个阶段
            "INTENTIONGENERATION" | "INTENTION_GENERATION" => MarketPhase::IntentionGeneration,
            "STRATEGYGENERATION" | "STRATEGY_GENERATION" => MarketPhase::StrategyGeneration,
            "RENEWALPROCESSING" | "RENEWAL_PROCESSING" => MarketPhase::RenewalProcessing,
            "FREEMARKET" | "FREE_MARKET" => MarketPhase::FreeMarket,
            "TRANSFERROUNDS" | "TRANSFER_ROUNDS" => MarketPhase::TransferRounds,
            "COMPLETED" => MarketPhase::Completed,

            // 旧阶段兼容映射
            "INITIALIZATION" => MarketPhase::IntentionGeneration,
            "DEPARTUREANNOUNCEMENT" | "DEPARTURE_ANNOUNCEMENT" => MarketPhase::FreeMarket,
            "INITIALBIDDING" | "INITIAL_BIDDING" => MarketPhase::FreeMarket,
            "NEGOTIATIONROUNDS" | "NEGOTIATION_ROUNDS" => MarketPhase::FreeMarket,
            "LASTCHANCE" | "LAST_CHANCE" => MarketPhase::FreeMarket,
            "FINALIZATION" => MarketPhase::Completed,

            _ => MarketPhase::IntentionGeneration,
        };

        let mut event = MarketEvent::new(
            save_id.to_string(),
            season_id as u64,
            event_type,
            phase,
            row.get::<i64, _>("round") as u8,
        );
        event.id = row.get::<i64, _>("id") as u64;
        event.player_id = row.get::<Option<i64>, _>("player_id").map(|id| id as u64);
        event.player_name = row.get("player_name");
        event.team_id = row.get::<Option<i64>, _>("team_id").map(|id| id as u64);
        event.team_name = row.get("team_name");
        event.title = row.get("title");
        event.description = row.get("description");
        event.ai_analysis = row.get("ai_analysis");

        engine.events.push(event);
        engine.next_event_id = engine.next_event_id.max(row.get::<i64, _>("id") as u64 + 1);
    }

    // 加载谈判和报价
    let neg_rows = sqlx::query(
        "SELECT * FROM negotiations WHERE save_id = ? AND season_id = ? ORDER BY id"
    )
    .bind(save_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for neg_row in neg_rows {
        let neg_id: i64 = neg_row.get("id");
        let status_str: String = neg_row.get("status");

        let status = match status_str.as_str() {
            "OPEN" => NegotiationStatus::Open,
            "ACCEPTED" => NegotiationStatus::Accepted,
            "REJECTED" => NegotiationStatus::Rejected,
            "EXPIRED" => NegotiationStatus::Expired,
            "WITHDRAWN" => NegotiationStatus::Withdrawn,
            _ => NegotiationStatus::Open,
        };

        let mut negotiation = Negotiation::new(
            save_id.to_string(),
            season_id as u64,
            neg_row.get::<i64, _>("player_id") as u64,
            neg_row.get("player_name"),
            neg_row.get::<Option<String>, _>("player_position").unwrap_or_default(),
            neg_row.get::<Option<i64>, _>("player_ability").unwrap_or(0) as u8,
            neg_row.get::<Option<i64>, _>("from_team_id").map(|id| id as u64),
            neg_row.get("from_team_name"),
        );
        negotiation.id = neg_id as u64;
        negotiation.status = status;
        negotiation.current_round = neg_row.get::<i64, _>("current_round") as u8;
        negotiation.final_team_id = neg_row.get::<Option<i64>, _>("final_team_id").map(|id| id as u64);
        negotiation.final_team_name = neg_row.get("final_team_name");
        negotiation.final_salary = neg_row.get::<Option<i64>, _>("final_salary").map(|s| s as u64);
        negotiation.final_years = neg_row.get::<Option<i64>, _>("final_years").map(|y| y as u8);
        negotiation.final_starter = neg_row.get::<Option<i64>, _>("final_starter").map(|s| s != 0);

        // 加载该谈判的报价
        let offer_rows = sqlx::query(
            "SELECT * FROM offers WHERE negotiation_id = ? ORDER BY id"
        )
        .bind(neg_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for offer_row in offer_rows {
            let offer_status_str: String = offer_row.get("status");
            let offer_status = match offer_status_str.as_str() {
                "PENDING" => OfferStatus::Pending,
                "ACCEPTED" => OfferStatus::Accepted,
                "REJECTED" => OfferStatus::Rejected,
                "COUNTERED" => OfferStatus::Countered,
                "WITHDRAWN" => OfferStatus::Withdrawn,
                "EXPIRED" => OfferStatus::Expired,
                _ => OfferStatus::Pending,
            };

            let mut offer = Offer::new(
                neg_id as u64,
                offer_row.get::<i64, _>("from_team_id") as u64,
                offer_row.get("from_team_name"),
                offer_row.get::<i64, _>("to_player_id") as u64,
                offer_row.get::<i64, _>("round") as u8,
            );
            offer.id = offer_row.get::<i64, _>("id") as u64;
            offer.salary_offer = offer_row.get::<i64, _>("salary_offer") as u64;
            offer.contract_years = offer_row.get::<i64, _>("contract_years") as u8;
            offer.guarantee_starter = offer_row.get::<i64, _>("guarantee_starter") != 0;
            offer.signing_bonus = offer_row.get::<i64, _>("signing_bonus") as u64;
            offer.transfer_fee = offer_row.get::<i64, _>("transfer_fee") as u64;
            offer.status = offer_status;
            offer.offer_reasoning = offer_row.get::<Option<String>, _>("offer_reasoning").unwrap_or_default();

            negotiation.offers.push(offer);
            engine.next_offer_id = engine.next_offer_id.max(offer_row.get::<i64, _>("id") as u64 + 1);
        }

        engine.negotiations.insert(neg_id as u64, negotiation);
        engine.next_negotiation_id = engine.next_negotiation_id.max(neg_id as u64 + 1);
    }

    log::info!(
        "市场引擎恢复完成: 阶段={:?}, 选手策略={}, 球队策略={}, 事件={}, 谈判={}",
        engine.state.current_phase,
        engine.player_strategies.len(),
        engine.team_strategies.len(),
        engine.events.len(),
        engine.negotiations.len()
    );

    // 保存引擎实例
    let mut engine_guard = MARKET_ENGINE.write().await;
    *engine_guard = Some(engine);

    Ok(())
}

/// 获取所有球队的市场状态
#[tauri::command]
pub async fn get_all_team_market_states(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<TeamMarketSummary>>, String> {
    let engine_guard = MARKET_ENGINE.read().await;

    if let Some(engine) = engine_guard.as_ref() {
        let summaries: Vec<TeamMarketSummary> = engine.state.team_states
            .values()
            .map(|ts| ts.into())
            .collect();
        return Ok(ApiResponse::success(summaries));
    }

    Ok(ApiResponse::error("转会市场未初始化"))
}

// ==================== 阶段推进命令 ====================

/// 生成所有选手的转会意愿（并行处理版本）
#[tauri::command]
pub async fn generate_player_intentions(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<GenerationProgress>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 检查引擎状态和阶段
    {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        // 检查阶段（只有意愿生成阶段才能调用此命令）
        if engine.state.current_phase != MarketPhase::IntentionGeneration
        {
            return Ok(ApiResponse::error("当前阶段不支持生成选手意愿"));
        }
    } // 释放锁

    // 获取数据
    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let teams_map: HashMap<u64, Team> = teams.iter().map(|t| (t.id, t.clone())).collect();

    // 查询赛区名称映射
    let region_rows = sqlx::query("SELECT id, short_name FROM regions WHERE save_id = ?")
        .bind(&save_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let region_map: HashMap<u64, String> = region_rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let short_name: String = row.get("short_name");
            (id as u64, short_name)
        })
        .collect();

    // 查询年度积分排名
    let rankings = PointsRepository::get_season_rankings(&pool, &save_id, season_id as u64)
        .await
        .unwrap_or_default();

    let ranking_map: HashMap<u64, (u32, u32)> = rankings
        .iter()
        .map(|r| (r.team_id, (r.rank, r.total_points)))
        .collect();

    // 构建可选球队列表
    let available_teams: Vec<TeamInfo> = teams.iter()
        .map(|t| {
            let (global_rank, annual_points) = ranking_map
                .get(&t.id)
                .copied()
                .unwrap_or((0, 0));

            TeamInfo {
                id: t.id,
                name: t.name.clone(),
                region_name: region_map.get(&t.region_id).cloned().unwrap_or_default(),
                avg_ability: 0.0, // 简化
                balance: t.balance,
                position_need: 50,
                annual_points,
                global_rank,
            }
        })
        .collect();

    // 查询选手荣誉
    let mut honors: HashMap<u64, PlayerHonorInfo> = HashMap::new();
    // 查询选手表现
    let mut performances: HashMap<u64, PlayerPerformanceInfo> = HashMap::new();

    for player in &players {
        if let Some(honor) = query_player_honors_simple(&pool, &save_id, player.id).await {
            honors.insert(player.id, honor);
        }
        if let Some(perf) = query_player_performance_simple(&pool, &save_id, player.id, season_id as u64, player.ability).await {
            performances.insert(player.id, perf);
        }
    }

    // 获取已经生成策略的选手ID（跳过这些选手）
    let already_generated_player_ids: std::collections::HashSet<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.player_strategies.keys().cloned().collect()
        } else {
            std::collections::HashSet::new()
        }
    };

    // 过滤出需要处理的选手（有球队的活跃选手，且没有已生成的策略）
    let all_active_players: Vec<_> = players
        .iter()
        .filter(|p| p.status == PlayerStatus::Active && p.team_id.is_some())
        .cloned()
        .collect();

    let players_to_process: Vec<_> = all_active_players
        .iter()
        .filter(|p| !already_generated_player_ids.contains(&p.id))
        .cloned()
        .collect();

    let skipped_count = all_active_players.len() - players_to_process.len();
    if skipped_count > 0 {
        log::info!("跳过 {} 名已有策略的选手，只生成剩余 {} 名选手的意愿",
            skipped_count, players_to_process.len());
    }

    let total = all_active_players.len() as u32;
    let to_generate = players_to_process.len() as u32;

    // 设置总数
    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.state.total_players = total;
        }
    }

    // 如果所有选手都已生成，直接完成
    if players_to_process.is_empty() {
        log::info!("所有选手意愿已生成，无需重新生成");
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.finish_intention_generation();
            save_market_state_to_db(&pool, &engine.state).await?;
        }
        return Ok(ApiResponse::success(GenerationProgress {
            task_type: "player_intentions".to_string(),
            current: total,
            total,
            percentage: 100,
            current_item: None,
            is_completed: true,
            errors: vec![],
        }));
    }

    // 发送开始事件
    let _ = app_handle.emit("intention-generation-progress", IntentionGenerationProgress {
        current: skipped_count as u32,
        total,
        player_name: "准备中...".to_string(),
        status: "starting".to_string(),
        wants_to_leave: None,
    });

    // 创建服务：优先LLM，降级为规则AI
    let use_llm = is_llm_configured();
    let llm_service = if use_llm {
        get_llm_config().map(|c| Arc::new(LLMTransferService::new(c)))
    } else {
        None
    };

    if use_llm {
        log::info!("使用 LLM 生成选手意愿（慢，20分钟）");
    } else {
        log::info!("使用规则AI生成选手意愿（快，<1秒）");
    }

    log::info!("开始并行生成 {} 名选手的转会意愿...", to_generate);

    // 并行生成所有选手策略
    let tasks: Vec<_> = players_to_process.iter().map(|player| {
        let player = player.clone();
        let teams_map = teams_map.clone();
        let available_teams = available_teams.clone();
        let honors = honors.clone();
        let performances = performances.clone();
        let llm_service = llm_service.clone();
        let save_id = save_id.clone();

        async move {
            let team = match player.team_id.and_then(|tid| teams_map.get(&tid)) {
                Some(t) => t.clone(),
                None => return (player.id, player.game_id.clone(), Err("选手没有所属球队".to_string())),
            };

            // 生成策略 - 优先LLM，降级为规则AI
            let strategy_result: Result<PlayerTransferStrategy, String> = if let Some(ref service) = llm_service {
                // 使用LLM
                service.generate_player_strategy(
                    &player,
                    &team,
                    &[],
                    &available_teams,
                    &save_id,
                    season_id as u64,
                    honors.get(&player.id),
                    performances.get(&player.id),
                    None,
                ).await
            } else {
                // 使用规则AI（Mock AI）
                use crate::services::ai_transfer_service::AITransferService;
                Ok(AITransferService::generate_mock_player_strategy(
                    &player,
                    &team,
                    &available_teams,
                    &save_id,
                    season_id as u64,
                ))
            };

            match strategy_result {
                Ok(s) => (player.id, player.game_id.clone(), Ok(s)),
                Err(e) => {
                    log::error!("生成选手 {} 策略失败: {}", player.game_id, e);
                    (player.id, player.game_id.clone(), Err(e))
                }
            }
        }
    }).collect();

    // 执行所有任务
    let results = join_all(tasks).await;

    log::info!("并行生成完成，开始保存结果...");

    // 保存结果（success_count 从已跳过的数量开始）
    let mut success_count = skipped_count as u32;
    let mut departure_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for (index, (player_id, player_name, strategy_result)) in results.iter().enumerate() {
        // 发送进度事件
        let (status, wants_to_leave) = match strategy_result {
            Ok(s) => ("success".to_string(), Some(s.wants_to_leave)),
            Err(e) => (format!("failed: {}", e), None),
        };

        let _ = app_handle.emit("intention-generation-progress", IntentionGenerationProgress {
            current: (skipped_count + index + 1) as u32,
            total,
            player_name: player_name.clone(),
            status,
            wants_to_leave,
        });

        match strategy_result {
            Ok(strategy) => {
                // 保存到引擎
                {
                    let mut engine_guard = MARKET_ENGINE.write().await;
                    if let Some(engine) = engine_guard.as_mut() {
                        engine.player_strategies.insert(*player_id, strategy.clone());
                        engine.state.intentions_generated += 1;
                    }
                }

                // 保存到数据库
                save_player_strategy_to_db(&pool, &save_id, season_id, *player_id, strategy).await.ok();
                success_count += 1;

                if strategy.wants_to_leave {
                    departure_count += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{}: {}", player_name, e));
            }
        }
    }

    log::info!(
        "选手意愿生成完成: 成功 {}/{}, 失败 {}, 想离队 {}",
        success_count, total, errors.len(), departure_count
    );

    // 只有全部成功才推进阶段
    let all_success = errors.is_empty();
    if all_success {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.finish_intention_generation();
            save_market_state_to_db(&pool, &engine.state).await?;
        }
    } else {
        // 有失败的选手，只保存状态但不推进阶段
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            save_market_state_to_db(&pool, &engine.state).await?;
        }
        log::warn!("有 {} 名选手意愿生成失败，请重试", errors.len());
    }

    // 发送完成事件
    let _ = app_handle.emit("intention-generation-progress", IntentionGenerationProgress {
        current: total,
        total,
        player_name: "".to_string(),
        status: if all_success { "completed".to_string() } else { "partial".to_string() },
        wants_to_leave: None,
    });

    // 返回结果
    Ok(ApiResponse::success(GenerationProgress {
        task_type: "player_intentions".to_string(),
        current: success_count,
        total,
        percentage: if total > 0 { (success_count * 100 / total) as u8 } else { 100 },
        current_item: None,
        is_completed: all_success,
        errors: errors.clone(),
    }))
}

/// 生成所有球队的转会策略
#[tauri::command]
pub async fn generate_team_strategies_llm(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<GenerationProgress>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 检查引擎状态和阶段
    {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        // 检查阶段
        if engine.state.current_phase != MarketPhase::IntentionGeneration
            && engine.state.current_phase != MarketPhase::StrategyGeneration
        {
            return Ok(ApiResponse::error("当前阶段不支持生成球队策略"));
        }
    } // 释放锁

    // 获取数据
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in &players {
        if player.status == PlayerStatus::Active {
            if let Some(team_id) = player.team_id {
                players_by_team.entry(team_id).or_default().push(player.clone());
            }
        }
    }

    // 获取自由球员（无球队的选手）
    let fa_players: Vec<Player> = players.iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 获取想离队的选手（从引擎中读取已生成的选手策略）
    let departure_player_ids: Vec<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.player_strategies.iter()
                .filter(|(_, s)| s.wants_to_leave)
                .map(|(id, _)| *id)
                .collect()
        } else {
            Vec::new()
        }
    };

    log::info!("找到 {} 个想离队的选手，将作为潜在目标", departure_player_ids.len());

    // 想离队的选手也作为可追逐目标
    let departure_players: Vec<Player> = players.iter()
        .filter(|p| departure_player_ids.contains(&p.id) && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 合并自由球员和想离队选手作为可追逐目标
    let mut all_available_players = fa_players.clone();
    all_available_players.extend(departure_players.clone());

    log::info!("可追逐目标总数: {} (自由球员: {}, 想离队: {})",
        all_available_players.len(), fa_players.len(), departure_players.len());

    let free_agents: Vec<FreeAgentInfo> = all_available_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: FreeAgent {
                    id: 0,
                    save_id: save_id.clone(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: FreeAgentReason::ContractExpire,
                    status: FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 获取 GM 配置
    let gm_profiles = load_gm_profiles(&pool, &save_id).await?;

    // 查询球队荣誉和表现数据
    let mut team_honors: HashMap<u64, TeamHonorInfo> = HashMap::new();
    let mut roster_honors: HashMap<u64, Vec<RosterPlayerHonorSummary>> = HashMap::new();
    let mut roster_performance: HashMap<u64, Vec<PlayerPerformanceSummary>> = HashMap::new();

    for team in &teams {
        let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();

        if let Some(honor) = query_team_honors_simple(&pool, &save_id, team.id, season_id as u64, &roster).await {
            team_honors.insert(team.id, honor);
        }

        let rh = query_roster_honors_simple(&pool, &save_id, &roster).await;
        roster_honors.insert(team.id, rh);

        let rp = query_roster_performance_simple(&pool, &save_id, &roster, season_id as u64).await;
        roster_performance.insert(team.id, rp);
    }

    // 获取已经生成策略的球队ID（跳过这些球队）
    let already_generated_team_ids: std::collections::HashSet<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.team_strategies.keys().cloned().collect()
        } else {
            std::collections::HashSet::new()
        }
    };

    // 过滤出需要生成策略的球队
    let teams_to_generate: Vec<_> = teams.iter()
        .filter(|t| !already_generated_team_ids.contains(&t.id))
        .cloned()
        .collect();

    let skipped_count = teams.len() - teams_to_generate.len();
    if skipped_count > 0 {
        log::info!("跳过 {} 支已有策略的球队，只生成剩余 {} 支球队的策略",
            skipped_count, teams_to_generate.len());
    }

    // 设置总数（只计算需要生成的）
    let total = teams.len() as u32;
    let to_generate = teams_to_generate.len() as u32;
    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.state.total_teams = total;
        }
    }

    // 如果所有球队都已生成，直接完成
    if teams_to_generate.is_empty() {
        log::info!("所有球队策略已生成，无需重新生成");
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.finish_strategy_generation();
            save_market_state_to_db(&pool, &engine.state).await?;
        }
        return Ok(ApiResponse::success(GenerationProgress {
            task_type: "team_strategies".to_string(),
            current: total,
            total,
            percentage: 100,
            current_item: None,
            is_completed: true,
            errors: vec![],
        }));
    }

    // 发送开始事件
    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
        current: skipped_count as u32,
        total,
        team_name: "准备中...".to_string(),
        status: "starting".to_string(),
    });

    // 创建 LLM 服务（共享）- 必须配置 LLM，否则报错
    let llm_service = if is_llm_configured() {
        match get_llm_config().map(|c| Arc::new(LLMTransferService::new(c))) {
            Some(service) => service,
            None => return Ok(ApiResponse::error("LLM 配置无效，请检查 API Key 设置")),
        }
    } else {
        return Ok(ApiResponse::error("LLM 未配置，请先在设置中配置 API Key"));
    };

    log::info!("开始并行生成 {} 支球队的转会策略...", to_generate);

    // 并行生成需要生成的球队策略（跳过已有的）
    let tasks: Vec<_> = teams_to_generate.iter().map(|team| {
        let team = team.clone();
        let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
        let profile = gm_profiles.get(&team.id).cloned().unwrap_or_else(|| {
            TeamGMProfile::new(team.id, save_id.clone())
        });
        let free_agents = free_agents.clone();
        let players_by_team = players_by_team.clone();
        let team_honor = team_honors.get(&team.id).cloned();
        let roster_honor = roster_honors.get(&team.id).cloned();
        let roster_perf = roster_performance.get(&team.id).cloned();
        let llm_service = llm_service.clone();
        let save_id = save_id.clone();

        async move {
            // 生成策略 - 必须使用 LLM，失败则报错
            match llm_service.generate_strategy(
                &team,
                &roster,
                &profile,
                &free_agents,
                &players_by_team,
                &save_id,
                season_id as u64,
                team_honor.as_ref(),
                roster_honor.as_deref(),
                roster_perf.as_deref(),
            ).await {
                Ok(s) => (team.id, team.name.clone(), Ok(s)),
                Err(e) => {
                    log::error!("LLM 生成球队 {} 策略失败: {}", team.name, e);
                    (team.id, team.name.clone(), Err(e))
                }
            }
        }
    }).collect();

    // 执行所有任务
    let results = join_all(tasks).await;

    log::info!("并行生成完成，开始保存结果...");

    // 保存结果（success_count 从已跳过的数量开始）
    let mut success_count = skipped_count as u32;
    let mut errors: Vec<String> = Vec::new();

    for (index, (team_id, team_name, strategy_result)) in results.iter().enumerate() {
        // 发送进度事件（加上已跳过的数量）
        let status = match strategy_result {
            Ok(_) => "success".to_string(),
            Err(e) => format!("failed: {}", e),
        };

        let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
            current: (skipped_count + index + 1) as u32,
            total,
            team_name: team_name.clone(),
            status,
        });

        match strategy_result {
            Ok(strategy) => {
                // 保存到引擎
                {
                    let mut engine_guard = MARKET_ENGINE.write().await;
                    if let Some(engine) = engine_guard.as_mut() {
                        engine.team_strategies.insert(*team_id, strategy.clone());
                        engine.state.strategies_generated += 1;

                        // 更新球队市场状态
                        if let Some(team_state) = engine.state.get_team_state_mut(*team_id) {
                            team_state.strategy_generated = true;
                        }
                    }
                }

                // 保存到数据库
                save_team_strategy_to_db(&pool, &save_id, season_id, *team_id, strategy).await?;
                success_count += 1;
            }
            Err(e) => {
                errors.push(format!("{}: {}", team_name, e));
                // 保存失败的球队ID用于重试
                {
                    let mut failed_ids = FAILED_TEAM_IDS.write().await;
                    if !failed_ids.contains(team_id) {
                        failed_ids.push(*team_id);
                    }
                }
            }
        }
    }

    log::info!("球队策略生成完成: 成功 {}/{}, 失败 {}", success_count, total, errors.len());

    // 只有全部成功才推进阶段
    let all_success = errors.is_empty();
    if all_success {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.finish_strategy_generation();
            save_market_state_to_db(&pool, &engine.state).await?;
        }
    } else {
        // 有失败的球队，只保存状态但不推进阶段
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            save_market_state_to_db(&pool, &engine.state).await?;
        }
        log::warn!("有 {} 支球队策略生成失败，请使用重试功能", errors.len());
    }

    // 发送完成事件
    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
        current: total,
        total,
        team_name: "".to_string(),
        status: if all_success { "completed".to_string() } else { "partial".to_string() },
    });

    // 返回结果
    Ok(ApiResponse::success(GenerationProgress {
        task_type: "team_strategies".to_string(),
        current: success_count,
        total,
        percentage: if total > 0 { (success_count * 100 / total) as u8 } else { 100 },
        current_item: None,
        is_completed: all_success, // 只有全部成功才标记完成
        errors: errors.clone(),
    }))
}

/// 使用规则引擎生成球队策略（不需要 LLM 配置）
#[tauri::command]
pub async fn generate_rule_based_team_strategies(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<GenerationProgress>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 检查引擎状态和阶段
    {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        // 检查阶段
        if engine.state.current_phase != MarketPhase::IntentionGeneration
            && engine.state.current_phase != MarketPhase::StrategyGeneration
        {
            return Ok(ApiResponse::error("当前阶段不支持生成球队策略"));
        }
    } // 释放锁

    // 获取数据
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in &players {
        if player.status == PlayerStatus::Active {
            if let Some(team_id) = player.team_id {
                players_by_team.entry(team_id).or_default().push(player.clone());
            }
        }
    }

    // 获取自由球员（无球队的选手）
    let fa_players: Vec<Player> = players.iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 获取想离队的选手（从引擎中读取已生成的选手策略）
    let departure_player_ids: Vec<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.player_strategies.iter()
                .filter(|(_, s)| s.wants_to_leave)
                .map(|(id, _)| *id)
                .collect()
        } else {
            Vec::new()
        }
    };

    log::info!("[规则引擎] 找到 {} 个想离队的选手，将作为潜在目标", departure_player_ids.len());

    // 想离队的选手也作为可追逐目标
    let departure_players: Vec<Player> = players.iter()
        .filter(|p| departure_player_ids.contains(&p.id) && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 合并自由球员和想离队选手作为可追逐目标
    let mut all_available_players = fa_players.clone();
    all_available_players.extend(departure_players.clone());

    log::info!("[规则引擎] 可追逐目标总数: {} (自由球员: {}, 想离队: {})",
        all_available_players.len(), fa_players.len(), departure_players.len());

    let free_agents: Vec<FreeAgentInfo> = all_available_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: FreeAgent {
                    id: 0,
                    save_id: save_id.clone(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: FreeAgentReason::ContractExpire,
                    status: FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 获取 GM 配置
    let gm_profiles = load_gm_profiles(&pool, &save_id).await?;

    let total = teams.len() as u32;

    // 发送开始事件
    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
        current: 0,
        total,
        team_name: "准备中...".to_string(),
        status: "starting".to_string(),
    });

    log::info!("[规则引擎] 开始生成 {} 支球队的转会策略...", total);

    // 生成策略
    let mut success_count = 0u32;

    for (index, team) in teams.iter().enumerate() {
        let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
        let profile = gm_profiles.get(&team.id).cloned().unwrap_or_else(|| {
            TeamGMProfile::new(team.id, save_id.clone())
        });

        // 使用规则引擎生成策略
        let strategy = AITransferService::generate_mock_strategy(
            team,
            &roster,
            &profile,
            &free_agents,
            &players_by_team,
            &save_id,
            season_id as u64,
        );

        // 发送进度事件
        let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
            current: (index + 1) as u32,
            total,
            team_name: team.name.clone(),
            status: "success".to_string(),
        });

        // 保存到引擎
        {
            let mut engine_guard = MARKET_ENGINE.write().await;
            if let Some(engine) = engine_guard.as_mut() {
                engine.team_strategies.insert(team.id, strategy.clone());
                engine.state.strategies_generated += 1;

                // 更新球队市场状态
                if let Some(team_state) = engine.state.get_team_state_mut(team.id) {
                    team_state.strategy_generated = true;
                }
            }
        }

        // 保存到数据库
        save_team_strategy_to_db(&pool, &save_id, season_id, team.id, &strategy).await?;
        success_count += 1;
    }

    log::info!("[规则引擎] 球队策略生成完成: 成功 {}/{}", success_count, total);

    // 推进阶段
    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.finish_strategy_generation();
            save_market_state_to_db(&pool, &engine.state).await?;
        }
    }

    // 发送完成事件
    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
        current: total,
        total,
        team_name: "".to_string(),
        status: "completed".to_string(),
    });

    // 返回结果
    Ok(ApiResponse::success(GenerationProgress {
        task_type: "team_strategies".to_string(),
        current: success_count,
        total,
        percentage: 100,
        current_item: None,
        is_completed: true,
        errors: vec![],
    }))
}

/// 续约处理进度事件
#[derive(Clone, Serialize)]
pub struct RenewalProgressEvent {
    pub current: u32,
    pub total: u32,
    pub player_name: String,
    pub team_name: String,
    pub status: String, // "processing", "success", "failed", "completed"
    pub renewal_successful: Option<bool>,
}

/// 处理所有续约（并行处理版本）
#[tauri::command]
pub async fn process_renewals(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<crate::models::RenewalProcessingResult>, String> {
    use crate::models::{RenewalDecision, RenewalProcessingResult};

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 检查引擎状态和阶段
    {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        // 检查阶段
        if engine.state.current_phase != MarketPhase::RenewalProcessing {
            return Ok(ApiResponse::error(&format!(
                "当前阶段 {:?} 不支持续约处理",
                engine.state.current_phase
            )));
        }
    } // 释放锁

    // 获取数据
    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let players_map: HashMap<u64, Player> = players.iter().map(|p| (p.id, p.clone())).collect();

    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let teams_map: HashMap<u64, Team> = teams.iter().map(|t| (t.id, t.clone())).collect();

    // 获取需要续约的选手列表
    let renewal_candidates: Vec<u64>;
    {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = engine_guard.as_ref().unwrap();
        renewal_candidates = engine.get_renewal_candidates();
    }

    // 查询已有续约记录的选手（排除已处理的）
    let existing_renewal_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT player_id FROM renewal_decisions WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let existing_set: std::collections::HashSet<u64> = existing_renewal_ids
        .into_iter()
        .map(|id| id as u64)
        .collect();

    log::info!("续约候选人: {} 个, 已有续约记录: {} 个", renewal_candidates.len(), existing_set.len());

    // 过滤出有球队的选手，并排除已有续约记录的
    let candidates_with_teams: Vec<(u64, u64)> = renewal_candidates.iter()
        .filter(|player_id| !existing_set.contains(player_id))  // 排除已有记录的
        .filter_map(|player_id| {
            players_map.get(player_id)
                .and_then(|p| p.team_id.map(|tid| (*player_id, tid)))
        })
        .collect();

    let total = candidates_with_teams.len() as u32;

    if total == 0 {
        // 没有需要续约的选手，直接完成
        {
            let mut engine_guard = MARKET_ENGINE.write().await;
            if let Some(engine) = engine_guard.as_mut() {
                engine.finish_renewal_processing();
                save_market_state_to_db(&pool, &engine.state).await?;
            }
        }

        return Ok(ApiResponse::success(RenewalProcessingResult {
            total_processed: 0,
            successful_renewals: 0,
            team_rejections: 0,
            player_rejections: 0,
            decisions: vec![],
            errors: vec![],
        }));
    }

    // 发送开始事件
    let _ = app_handle.emit("renewal-progress", RenewalProgressEvent {
        current: 0,
        total,
        player_name: "准备中...".to_string(),
        team_name: "".to_string(),
        status: "starting".to_string(),
        renewal_successful: None,
    });

    // 查询选手荣誉和表现
    let mut honors: HashMap<u64, crate::services::llm_service::PlayerHonorInfo> = HashMap::new();
    let mut performances: HashMap<u64, crate::services::llm_service::PlayerPerformanceInfo> = HashMap::new();

    for (player_id, _) in &candidates_with_teams {
        if let Some(player) = players_map.get(player_id) {
            if let Some(honor) = query_player_honors_simple(&pool, &save_id, *player_id).await {
                honors.insert(*player_id, honor);
            }
            if let Some(perf) = query_player_performance_simple(&pool, &save_id, *player_id, season_id as u64, player.ability).await {
                performances.insert(*player_id, perf);
            }
        }
    }

    // 获取选手策略和球队策略（在并行处理前）
    let (player_strategies, team_strategies): (HashMap<u64, PlayerTransferStrategy>, HashMap<u64, AITransferStrategy>) = {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = engine_guard.as_ref().unwrap();
        (engine.player_strategies.clone(), engine.team_strategies.clone())
    };

    // 创建 LLM 服务（共享）
    let llm_service = if is_llm_configured() {
        match get_llm_config().map(|c| Arc::new(LLMTransferService::new(c))) {
            Some(service) => service,
            None => return Ok(ApiResponse::error("LLM 配置无效，请检查 API Key 设置")),
        }
    } else {
        return Ok(ApiResponse::error("LLM 未配置，请先在设置中配置 API Key"));
    };

    log::info!("开始并行处理 {} 名选手的续约...", total);

    // 并行处理所有续约
    let tasks: Vec<_> = candidates_with_teams.iter().map(|(player_id, team_id)| {
        let player = players_map.get(player_id).cloned();
        let team = teams_map.get(team_id).cloned();
        let player_strategy = player_strategies.get(player_id).cloned();
        let team_strategy = team_strategies.get(team_id).cloned();
        let player_honors = honors.get(player_id).cloned();
        let player_performance = performances.get(player_id).cloned();
        let llm_service = llm_service.clone();

        async move {
            let player = match player {
                Some(p) => p,
                None => return (*player_id, "未知选手".to_string(), "".to_string(), Err("选手不存在".to_string())),
            };
            let team = match team {
                Some(t) => t,
                None => return (*player_id, player.game_id.clone(), "".to_string(), Err("球队不存在".to_string())),
            };
            let player_strategy = match player_strategy {
                Some(s) => s,
                None => return (*player_id, player.game_id.clone(), team.name.clone(), Err("选手策略不存在".to_string())),
            };

            // 调用 LLM 评估续约
            match llm_service.evaluate_renewal(
                &player,
                &team,
                &player_strategy,
                team_strategy.as_ref(),
                player_honors.as_ref(),
                player_performance.as_ref(),
            ).await {
                Ok(decision) => (*player_id, player.game_id.clone(), team.name.clone(), Ok((decision, player, team))),
                Err(e) => {
                    log::error!("LLM 评估 {} 续约失败: {}", player.game_id, e);
                    (*player_id, player.game_id.clone(), team.name.clone(), Err(e))
                }
            }
        }
    }).collect();

    // 执行所有任务
    let results = futures::future::join_all(tasks).await;

    log::info!("并行续约评估完成，开始保存结果...");

    // 保存结果
    let mut decisions: Vec<RenewalDecision> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut successful_renewals = 0u32;
    let mut team_rejections = 0u32;
    let mut player_rejections = 0u32;

    for (index, (player_id, player_name, team_name, result)) in results.iter().enumerate() {
        // 发送进度事件
        let (status, renewal_successful) = match result {
            Ok((decision, _, _)) => ("success".to_string(), Some(decision.renewal_successful)),
            Err(e) => (format!("failed: {}", e), None),
        };

        let _ = app_handle.emit("renewal-progress", RenewalProgressEvent {
            current: (index + 1) as u32,
            total,
            player_name: player_name.clone(),
            team_name: team_name.clone(),
            status,
            renewal_successful,
        });

        match result {
            Ok((decision, player, team)) => {
                // 更新统计
                if decision.renewal_successful {
                    successful_renewals += 1;
                } else if !decision.team_wants_renewal {
                    team_rejections += 1;
                } else {
                    player_rejections += 1;
                }

                // 更新引擎状态（记录事件、更新自由球员列表等）
                {
                    let mut engine_guard = MARKET_ENGINE.write().await;
                    if let Some(engine) = engine_guard.as_mut() {
                        if decision.renewal_successful {
                            engine.record_event_with_player(
                                crate::models::MarketEventType::RenewalSuccessful,
                                *player_id,
                                player.game_id.clone(),
                                team.id,
                                team.name.clone(),
                                format!("{} 与 {} 续约成功", player.game_id, team.name),
                                format!(
                                    "新合同: {}万/年，{}年，{}",
                                    decision.final_salary.unwrap_or(0),
                                    decision.final_years.unwrap_or(0),
                                    decision.summary
                                ),
                            );

                            // 更新选手合同（薪资和合同结束赛季）
                            if let (Some(new_salary), Some(years)) = (decision.final_salary, decision.final_years) {
                                let new_contract_end = season_id + years as i64;
                                if let Err(e) = sqlx::query(
                                    "UPDATE players SET salary = ?, contract_end_season = ? WHERE id = ?"
                                )
                                .bind(new_salary as i64)
                                .bind(new_contract_end)
                                .bind(*player_id as i64)
                                .execute(&pool)
                                .await {
                                    log::error!("更新选手 {} 合同失败: {}", player.game_id, e);
                                } else {
                                    log::info!(
                                        "选手 {} 合同已更新: 薪资 {}万/年, 合同至第{}赛季",
                                        player.game_id, new_salary, new_contract_end
                                    );
                                }
                            }
                        } else if !decision.team_wants_renewal {
                            engine.record_event_with_player(
                                crate::models::MarketEventType::RenewalFailed,
                                *player_id,
                                player.game_id.clone(),
                                team.id,
                                team.name.clone(),
                                format!("{} 遭 {} 放弃续约", player.game_id, team.name),
                                decision.team_rejection_reason.clone().unwrap_or_else(|| decision.summary.clone()),
                            );
                            engine.state.add_free_agent(*player_id);
                        } else {
                            engine.record_event_with_player(
                                crate::models::MarketEventType::RenewalFailed,
                                *player_id,
                                player.game_id.clone(),
                                team.id,
                                team.name.clone(),
                                format!("{} 拒绝 {} 的续约报价", player.game_id, team.name),
                                decision.player_rejection_reason.clone().unwrap_or_else(|| decision.summary.clone()),
                            );
                            engine.state.add_free_agent(*player_id);
                        }
                    }
                }

                decisions.push(decision.clone());
            }
            Err(e) => {
                errors.push(format!("{}: {}", player_name, e));
            }
        }
    }

    log::info!(
        "续约处理完成: 成功续约 {}, 球队拒绝 {}, 选手拒绝 {}, 错误 {}",
        successful_renewals, team_rejections, player_rejections, errors.len()
    );

    // 保存续约决策到数据库
    for decision in &decisions {
        save_renewal_decision_to_db(&pool, &save_id, season_id, decision).await.ok();
    }

    // 检查是否所有选手都已处理完成（没有错误）
    // 重新查询数据库确认已处理的数量
    let processed_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM renewal_decisions WHERE save_id = ? AND season_id = ?"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    // renewal_candidates 已经是不想离队的选手列表，所以直接比较即可
    let total_candidates = renewal_candidates.len() as i64;
    let all_processed = processed_count >= total_candidates;

    log::info!(
        "续约处理状态: 已处理 {}/{}, 本次错误 {}, 全部完成: {}",
        processed_count, total_candidates, errors.len(), all_processed
    );

    // 只有全部处理完成才推进到下一阶段
    if all_processed && errors.is_empty() {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.finish_renewal_processing();
            save_market_state_to_db(&pool, &engine.state).await?;
            log::info!("续约阶段完成，推进到下一阶段");
        }
    } else {
        // 只保存当前状态，不推进阶段
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            save_market_state_to_db(&pool, &engine.state).await?;
        }
        if !errors.is_empty() {
            log::info!("还有 {} 个选手续约失败，用户可再次点击处理", errors.len());
        }
    }

    // 发送完成事件
    let _ = app_handle.emit("renewal-progress", RenewalProgressEvent {
        current: total,
        total,
        player_name: "".to_string(),
        team_name: "".to_string(),
        status: "completed".to_string(),
        renewal_successful: None,
    });

    // 从数据库加载所有续约记录（包括之前处理的）
    let all_decisions_result = load_all_renewal_decisions(&pool, &save_id, season_id).await;
    let (all_decisions, all_successful, all_team_rejections, all_player_rejections) = match all_decisions_result {
        Ok(data) => data,
        Err(_) => (decisions.clone(), successful_renewals, team_rejections, player_rejections),
    };

    Ok(ApiResponse::success(RenewalProcessingResult {
        total_processed: all_decisions.len() as u32,
        successful_renewals: all_successful,
        team_rejections: all_team_rejections,
        player_rejections: all_player_rejections,
        decisions: all_decisions,
        errors,
    }))
}

/// 执行下一轮（并发 LLM 调用）
#[tauri::command]
pub async fn execute_llm_market_round(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<RoundExecutionResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 获取球队和选手数据
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let teams_map: HashMap<u64, Team> = teams.iter().map(|t| (t.id, t.clone())).collect();

    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let players_map: HashMap<u64, Player> = players.iter().map(|p| (p.id, p.clone())).collect();

    // 查询选手荣誉和表现数据（用于 LLM 决策）
    let mut player_honors: HashMap<u64, PlayerHonorInfo> = HashMap::new();
    let mut player_performances: HashMap<u64, PlayerPerformanceInfo> = HashMap::new();

    for player in &players {
        if let Some(honor) = query_player_honors_simple(&pool, &save_id, player.id).await {
            player_honors.insert(player.id, honor);
        }
        if let Some(perf) = query_player_performance_simple(&pool, &save_id, player.id, season_id as u64, player.ability).await {
            player_performances.insert(player.id, perf);
        }
    }

    log::info!("已加载 {} 名选手的荣誉数据，{} 名选手的表现数据",
        player_honors.len(), player_performances.len());

    // 创建 LLM 服务
    let llm_service = if is_llm_configured() {
        match get_llm_config().map(|c| Arc::new(LLMTransferService::new(c))) {
            Some(service) => service,
            None => return Ok(ApiResponse::error("LLM 配置无效")),
        }
    } else {
        return Ok(ApiResponse::error("LLM 未配置"));
    };

    // 从引擎获取当前状态和策略
    let (current_round, current_phase, team_strategies, player_strategies, free_agent_ids, poachable_player_ids) = {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };
        (
            engine.state.current_round + 1, // 下一轮
            engine.state.current_phase,
            engine.team_strategies.clone(),
            engine.player_strategies.clone(),
            engine.state.free_agent_ids.clone(),
            engine.state.poachable_player_ids.clone(),
        )
    };

    // 判断是自由市场阶段还是挖角阶段
    let is_transfer_round = current_phase == MarketPhase::TransferRounds;

    log::info!("========== 执行第 {} 轮{}（并发 LLM） ==========",
        current_round,
        if is_transfer_round { "挖角评估" } else { "报价" }
    );

    // 如果是挖角阶段，显示可挖角选手信息
    if is_transfer_round {
        log::info!("--- 挖角阶段：评估 {} 名85+能力值选手 ---", poachable_player_ids.len());
    } else {
        log::info!("--- 阶段1：并发调用 LLM 评估自由市场（每个球队评估所有可用选手） ---");
    }

    // 收集已签约的选手（避免向已签约选手发送报价）
    let signed_player_ids: std::collections::HashSet<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.negotiations.values()
                .filter(|n| n.status == NegotiationStatus::Accepted)
                .map(|n| n.player_id)
                .collect()
        } else {
            std::collections::HashSet::new()
        }
    };

    // 收集每个战队已发送报价的选手（避免重复报价）
    let team_offered_players: std::collections::HashMap<u64, std::collections::HashSet<u64>> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            let mut map: std::collections::HashMap<u64, std::collections::HashSet<u64>> = std::collections::HashMap::new();
            for neg in engine.negotiations.values() {
                for offer in &neg.offers {
                    map.entry(offer.from_team_id)
                        .or_insert_with(std::collections::HashSet::new)
                        .insert(neg.player_id);
                }
            }
            map
        } else {
            std::collections::HashMap::new()
        }
    };

    // 根据阶段选择评估目标
    let target_player_ids: Vec<u64> = if is_transfer_round {
        poachable_player_ids.clone()
    } else {
        free_agent_ids.clone()
    };

    // 构建评估目标信息列表（用于 LLM 评估）
    let free_agents_info: Vec<FreeAgentInfo> = target_player_ids.iter()
        .filter_map(|player_id| {
            players_map.get(player_id).map(|p| {
                let market_value = p.calculate_market_value();
                let salary_exp = crate::models::calculate_expected_salary(market_value);
                // 挖角阶段使用 Poached 原因，自由市场使用 ContractExpire
                let reason = if is_transfer_round {
                    FreeAgentReason::Poached
                } else {
                    FreeAgentReason::ContractExpire
                };
                FreeAgentInfo {
                    agent: FreeAgent {
                        id: 0,
                        save_id: save_id.clone(),
                        season_id: season_id as u64,
                        player_id: p.id,
                        salary_demand: salary_exp.expected,
                        reason,
                        status: FreeAgentStatus::Available,
                    },
                    player: p.clone(),
                    market_value,
                    expected_salary: salary_exp.expected,
                    minimum_salary: salary_exp.minimum,
                }
            })
        })
        .collect();

    log::info!("{}共有 {} 名可评估选手",
        if is_transfer_round { "挖角市场" } else { "自由市场" },
        free_agents_info.len()
    );

    // 获取已评估的战队列表（用于重试机制）
    let evaluated_team_ids: std::collections::HashSet<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.state.evaluated_team_ids.iter().cloned().collect()
        } else {
            std::collections::HashSet::new()
        }
    };

    let total_teams = team_strategies.len();
    let already_evaluated = evaluated_team_ids.len();
    log::info!("战队评估进度: {}/{} 已完成", already_evaluated, total_teams);

    // 为每个战队创建 LLM 评估任务（跳过已评估的战队）
    let mut team_market_eval_tasks = Vec::new();

    for (team_id, strategy) in &team_strategies {
        // 跳过已评估的战队
        if evaluated_team_ids.contains(team_id) {
            log::info!("跳过已评估的战队 ID: {}", team_id);
            continue;
        }

        let team = match teams_map.get(team_id) {
            Some(t) => t.clone(),
            None => continue,
        };

        // 获取该战队已报价过的选手
        let already_offered = team_offered_players.get(team_id)
            .cloned()
            .unwrap_or_default();

        let llm = llm_service.clone();
        let strategy = strategy.clone();
        let free_agents = free_agents_info.clone();
        let p_strategies = player_strategies.clone();
        let signed_ids = signed_player_ids.clone();
        let p_honors = player_honors.clone();
        let p_performances = player_performances.clone();

        let team_name_log = team.name.clone();

        team_market_eval_tasks.push(async move {
            log::info!("球队 {} 开始评估{}...", team_name_log, if is_transfer_round { "挖角目标" } else { "自由市场" });

            // 根据阶段调用不同的 LLM 评估方法
            let result = if is_transfer_round {
                // 挖角阶段：调用 evaluate_poaching_market，传入所有85+选手
                // 需要将 FreeAgentInfo 转换为 Player 引用
                let players_refs: Vec<&Player> = free_agents.iter()
                    .map(|fa| &fa.player)
                    .collect();

                llm.evaluate_poaching_market(
                    &team,
                    &strategy,
                    &players_refs,
                    &p_strategies,
                    &signed_ids,
                    &already_offered,
                    current_round,
                    &p_honors,
                    &p_performances,
                ).await
            } else {
                // 自由市场阶段：调用 evaluate_free_market
                llm.evaluate_free_market(
                    &team,
                    &strategy,
                    &free_agents,
                    &p_strategies,
                    &signed_ids,
                    &already_offered,
                    current_round,
                    &p_honors,
                    &p_performances,
                ).await
            };

            (team.id, team.name.clone(), result)
        });
    }

    // 并发执行所有战队的市场评估任务
    let team_market_results = join_all(team_market_eval_tasks).await;

    // 应用战队评估结果到引擎
    let mut new_negotiations = 0u32;
    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        let engine = match engine_guard.as_mut() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        for (team_id, team_name, result) in &team_market_results {
            match result {
                Ok(evaluation) => {
                    // 记录该战队已成功评估
                    if !engine.state.evaluated_team_ids.contains(team_id) {
                        engine.state.evaluated_team_ids.push(*team_id);
                    }

                    // 记录对每个选手的评估思考过程
                    let mut eval_summary_parts: Vec<String> = Vec::new();

                    for player_eval in &evaluation.player_evaluations {
                        // 获取选手名称
                        let player_name = players_map.get(&player_eval.player_id)
                            .map(|p| p.game_id.clone())
                            .unwrap_or_else(|| format!("ID:{}", player_eval.player_id));

                        // 构建评估摘要
                        let eval_line = format!(
                            "• {} - 适配度{}分: {}{}",
                            player_name,
                            player_eval.fit_score,
                            player_eval.evaluation,
                            player_eval.rejection_reason.as_ref()
                                .map(|r| format!(" [不考虑: {}]", r))
                                .unwrap_or_default()
                        );
                        eval_summary_parts.push(eval_line);
                    }

                    // 添加最终决策
                    let final_decision = if let Some(chosen_id) = evaluation.chosen_player_id {
                        let chosen_name = players_map.get(&chosen_id)
                            .map(|p| p.game_id.clone())
                            .unwrap_or_else(|| format!("ID:{}", chosen_id));
                        format!("\n\n✅ 最终决定: 向 {} 发出报价", chosen_name)
                    } else {
                        "\n\n❌ 最终决定: 本轮不报价".to_string()
                    };
                    eval_summary_parts.push(final_decision);

                    // 记录战队的完整市场思考过程
                    let event_title = if is_transfer_round {
                        format!("{} 评估挖角市场（{}名85+选手）", team_name, evaluation.player_evaluations.len())
                    } else {
                        format!("{} 评估自由市场（{}名选手）", team_name, evaluation.player_evaluations.len())
                    };

                    engine.record_event_with_player(
                        MarketEventType::TeamThinking,
                        evaluation.chosen_player_id.unwrap_or(0),
                        evaluation.chosen_player_id
                            .and_then(|id| players_map.get(&id))
                            .map(|p| p.game_id.clone())
                            .unwrap_or_default(),
                        *team_id,
                        team_name.clone(),
                        event_title,
                        format!(
                            "## 选手逐一评估\n{}\n\n## 整体分析\n{}",
                            eval_summary_parts.join("\n"),
                            evaluation.overall_reasoning
                        ),
                    );

                    // 如果决定报价，创建报价
                    if let (Some(chosen_player_id), Some(offer_details)) =
                        (evaluation.chosen_player_id, &evaluation.offer_details)
                    {
                        let player = match players_map.get(&chosen_player_id) {
                            Some(p) => p,
                            None => {
                                log::error!("选手 {} 不存在", chosen_player_id);
                                continue;
                            }
                        };

                        // 创建谈判（挖角和自由市场都用同一个方法）
                        let neg_id = engine.find_or_create_negotiation(player);

                        // 挖角阶段：设置转会费标志
                        if is_transfer_round {
                            if let Some(neg) = engine.negotiations.get_mut(&neg_id) {
                                let transfer_fee = player.calculate_market_value() / 10000;
                                neg.is_transfer = true;
                                neg.transfer_fee = Some(transfer_fee);
                                log::info!("设置挖人谈判 {} 的转会费: {}万", neg_id, transfer_fee);
                            }
                        }

                        let offer_id = engine.next_offer_id;
                        engine.next_offer_id += 1;

                        let mut offer = Offer::new(
                            neg_id,
                            *team_id,
                            team_name.clone(),
                            chosen_player_id,
                            current_round,
                        );
                        offer.id = offer_id;
                        offer.salary_offer = offer_details.salary_offer;
                        offer.contract_years = offer_details.contract_years;
                        offer.guarantee_starter = offer_details.guarantee_starter;
                        offer.signing_bonus = offer_details.signing_bonus;
                        offer.offer_reasoning = offer_details.reasoning.clone();

                        // 挖角阶段：设置转会费
                        if is_transfer_round {
                            offer.transfer_fee = player.calculate_market_value() / 10000;
                        }

                        // 添加到谈判
                        if let Some(neg) = engine.negotiations.get_mut(&neg_id) {
                            neg.add_offer(offer.clone());

                            // 如果是首次创建谈判
                            if neg.offers.len() == 1 {
                                new_negotiations += 1;
                            }
                        }

                        // 记录报价事件
                        let event_type = if is_transfer_round {
                            MarketEventType::TransferOfferMade
                        } else {
                            MarketEventType::OfferMade
                        };

                        let event_desc = if is_transfer_round {
                            format!(
                                "{}万/年，{}年合同{}，转会费{}万。\n\n报价理由: {}",
                                offer_details.salary_offer,
                                offer_details.contract_years,
                                if offer_details.guarantee_starter { "，首发保证" } else { "" },
                                offer.transfer_fee,
                                offer_details.reasoning
                            )
                        } else {
                            format!(
                                "{}万/年，{}年合同{}。\n\n报价理由: {}",
                                offer_details.salary_offer,
                                offer_details.contract_years,
                                if offer_details.guarantee_starter { "，首发保证" } else { "" },
                                offer_details.reasoning
                            )
                        };

                        engine.record_event_with_player(
                            event_type,
                            chosen_player_id,
                            player.game_id.clone(),
                            *team_id,
                            team_name.clone(),
                            format!("{} 向 {} 发出{}报价", team_name, player.game_id, if is_transfer_round { "挖人" } else { "" }),
                            event_desc,
                        );

                        let transfer_fee_log = if is_transfer_round {
                            format!("，转会费{}万", offer.transfer_fee)
                        } else {
                            String::new()
                        };

                        log::info!(
                            "✓ {} 向 {} 发出{}报价: {}万/年，{}年{}",
                            team_name, player.game_id,
                            if is_transfer_round { "挖人" } else { "" },
                            offer_details.salary_offer, offer_details.contract_years,
                            transfer_fee_log
                        );
                    } else {
                        log::info!("○ {} 本轮决定不报价", team_name);
                    }
                }
                Err(e) => {
                    log::error!("战队 {} LLM 市场评估失败: {}", team_name, e);

                    // 记录失败事件
                    engine.record_event_with_player(
                        MarketEventType::TeamThinking,
                        0,
                        String::new(),
                        *team_id,
                        team_name.clone(),
                        format!("{} 市场评估失败", team_name),
                        format!("LLM 调用失败: {}", e),
                    );
                }
            }
        }
    }

    // ========== 阶段 2：并发获取选手决策 ==========
    log::info!("--- 阶段2：并发调用 LLM 获取选手决策 ---");

    // 收集所有收到报价的选手
    let players_with_offers: Vec<(u64, u64, Vec<TeamOffer>)> = {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        engine.negotiations.iter()
            .filter(|(_, n)| n.status == NegotiationStatus::Open)
            .filter_map(|(neg_id, n)| {
                let pending_offers: Vec<TeamOffer> = n.offers.iter()
                    .filter(|o| o.status == OfferStatus::Pending)
                    .map(|o| TeamOffer {
                        id: o.id,
                        team_id: o.from_team_id,
                        team_name: o.from_team_name.clone(),
                        transfer_fee: 0,
                        salary_offer: o.salary_offer,
                        contract_years: o.contract_years,
                        starter_guarantee: o.guarantee_starter,
                        offered_at: chrono::Utc::now().to_rfc3339(),
                    })
                    .collect();

                if pending_offers.is_empty() {
                    None
                } else {
                    Some((*neg_id, n.player_id, pending_offers))
                }
            })
            .collect()
    };

    // 为每个选手创建决策任务
    let mut player_decision_tasks = Vec::new();

    for (neg_id, player_id, offers) in &players_with_offers {
        let player = match players_map.get(player_id) {
            Some(p) => p.clone(),
            None => continue,
        };

        let strategy = match player_strategies.get(player_id) {
            Some(s) => s.clone(),
            None => continue,
        };

        let llm = llm_service.clone();
        let offers = offers.clone();
        let neg_id = *neg_id;

        // 获取选手荣誉和表现数据
        let honors = player_honors.get(player_id).cloned();
        let performance = player_performances.get(player_id).cloned();

        // 在移动到 async 块之前先克隆用于日志的值
        let player_name_log = player.game_id.clone();
        let offers_count_log = offers.len();

        player_decision_tasks.push(async move {
            // 调用 LLM 评估所有报价（传入荣誉和表现数据）
            let result = llm.evaluate_multiple_offers(
                &player,
                &strategy,
                &offers,
                honors.as_ref(),
                performance.as_ref(),
            ).await;
            (neg_id, player.id, player.game_id.clone(), offers, result)
        });

        log::info!("选手 {} 收到 {} 份报价，准备决策", player_name_log, offers_count_log);
    }

    // 并发执行所有选手决策任务
    let player_decision_results = join_all(player_decision_tasks).await;

    // 应用选手决策结果到引擎
    let mut completed_signings = 0u32;
    let mut signed_players: Vec<SigningInfo> = Vec::new();

    let all_teams_evaluated_inner = {
        let mut engine_guard = MARKET_ENGINE.write().await;
        let engine = match engine_guard.as_mut() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };

        for (neg_id, player_id, player_name, offers, result) in player_decision_results {
            match result {
                Ok(decision) => {
                    // 选手必须选择一个报价（不能全部拒绝）
                    // 如果 LLM 返回 None，强制选择薪资最高的
                    let chosen_offer_id = decision.chosen_offer_id.or_else(|| {
                        offers.iter().max_by_key(|o| o.salary_offer).map(|o| o.id)
                    });

                    if let Some(offer_id) = chosen_offer_id {
                        if let Some(accepted_offer) = offers.iter().find(|o| o.id == offer_id) {
                            // 构建报价列表描述
                            let offers_desc = offers.iter()
                                .map(|o| format!("{}({}万/年)", o.team_name, o.salary_offer))
                                .collect::<Vec<_>>()
                                .join("、");

                            // 记录选手思考过程
                            engine.record_event_with_player(
                                MarketEventType::PlayerThinking,
                                player_id,
                                player_name.clone(),
                                accepted_offer.team_id,
                                accepted_offer.team_name.clone(),
                                format!("{} 做出选择", player_name),
                                format!(
                                    "收到报价: {}\n\n💭 思考过程:\n{}\n\n✅ 决定: 加盟 {}",
                                    offers_desc, decision.reasoning, accepted_offer.team_name
                                ),
                            );

                            // 找到引擎中的 Offer 并更新状态
                            if let Some(neg) = engine.negotiations.get_mut(&neg_id) {
                                // 将接受的报价标记为 Accepted
                                for offer in &mut neg.offers {
                                    if offer.id == offer_id {
                                        offer.status = OfferStatus::Accepted;
                                    } else if offer.status == OfferStatus::Pending {
                                        offer.status = OfferStatus::Rejected;
                                    }
                                }

                                // 完成签约
                                neg.complete_signing(
                                    accepted_offer.team_id,
                                    accepted_offer.team_name.clone(),
                                    accepted_offer.salary_offer,
                                    accepted_offer.contract_years,
                                    accepted_offer.starter_guarantee,
                                );
                            }

                            // 移除自由球员
                            engine.state.remove_free_agent(player_id);
                            engine.state.complete_negotiation(neg_id);

                            // 记录签约事件
                            engine.record_event_with_player(
                                MarketEventType::SigningCompleted,
                                player_id,
                                player_name.clone(),
                                accepted_offer.team_id,
                                accepted_offer.team_name.clone(),
                                format!("{} 加盟 {}", player_name, accepted_offer.team_name),
                                format!(
                                    "{}万/年，{}年合同{}",
                                    accepted_offer.salary_offer,
                                    accepted_offer.contract_years,
                                    if accepted_offer.starter_guarantee { "，首发保证" } else { "" }
                                ),
                            );

                            completed_signings += 1;
                            signed_players.push(SigningInfo {
                                player_id,
                                player_name: player_name.clone(),
                                team_id: accepted_offer.team_id,
                                team_name: accepted_offer.team_name.clone(),
                                salary: accepted_offer.salary_offer,
                                years: accepted_offer.contract_years,
                            });

                            log::info!(
                                "✓ {} 选择加盟 {}，{}万/年，{}年合同",
                                player_name, accepted_offer.team_name,
                                accepted_offer.salary_offer, accepted_offer.contract_years
                            );
                        }
                    }
                }
                Err(e) => {
                    log::error!("选手 {} LLM 决策失败: {}", player_name, e);
                }
            }
        }

        // ========== 阶段 3：反馈签约信息给战队 ==========
        log::info!("--- 阶段3：反馈签约信息给战队 ---");

        for signing in &signed_players {
            log::info!("反馈: {} 已加盟 {}", signing.player_name, signing.team_name);

            // 更新签约战队的状态
            if let Some(team_state) = engine.state.get_team_state_mut(signing.team_id) {
                let cost = signing.salary * signing.years as u64 * 10000;
                team_state.remaining_budget = team_state.remaining_budget.saturating_sub(cost);
                team_state.roster_count += 1;
            }
        }

        // 统计等待中的选手
        let waiting_players = engine.state.free_agent_ids.len();
        let evaluated_count = engine.state.evaluated_team_ids.len();
        let all_teams_evaluated_inner = evaluated_count >= total_teams;

        log::info!(
            "本轮统计: 新增{}个谈判, 完成{}笔签约, {}名选手等待中, 战队评估进度: {}/{}",
            new_negotiations, completed_signings, waiting_players, evaluated_count, total_teams
        );

        // 检查市场稳定性
        if new_negotiations == 0 && completed_signings == 0 {
            engine.state.record_stable_round();
        }

        // 只有所有战队都评估完成后才推进轮次
        if all_teams_evaluated_inner {
            engine.state.advance_round();
            log::info!("所有战队评估完成，推进到下一轮");
            // 注意：不在这里清空 evaluated_team_ids，等阶段检查完成后再清空
        } else {
            log::info!("还有 {} 个战队未完成评估，请再次点击执行", total_teams - evaluated_count);
        }

        all_teams_evaluated_inner
    };

    // 构建结果（使用之前计算的 all_teams_evaluated）
    let (phase, round, phase_changed, new_phase) = {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = engine_guard.as_ref().unwrap();

        // 检查是否需要切换阶段（只有所有战队评估完成后才考虑）
        // 条件1：各阶段的特定条件
        // 条件2：自由球员为空 = 自由市场结束
        // 条件3：达到最大轮次（7轮）强制推进
        let free_agents_empty = engine.state.free_agent_ids.is_empty();
        let max_round_reached = engine.state.current_round >= 7; // 最多7轮

        // 只有所有战队评估完成后才考虑阶段切换（使用之前的判断结果）
        let phase_changed = if !all_teams_evaluated_inner {
            false
        } else {
            match engine.state.current_phase {
                // 自由市场阶段：市场稳定或自由球员为空或达到最大轮次
                MarketPhase::FreeMarket => {
                    engine.state.should_enter_last_chance() || free_agents_empty || max_round_reached
                },
                // 转会轮次阶段：至少执行1轮，达到3轮或市场稳定后完成
                MarketPhase::TransferRounds => {
                    // 必须至少执行1轮
                    engine.state.current_round >= 1 &&
                    (engine.state.should_end_transfer_rounds() || engine.state.current_round >= 3)
                },
                // 其他阶段不自动推进（需要用户手动点击"进入下一阶段"）
                _ => false,
            }
        };

        log::info!(
            "阶段检查: phase={:?}, round={}, free_agents={}, max_round_reached={}, all_evaluated={}, phase_changed={}",
            engine.state.current_phase, engine.state.current_round,
            engine.state.free_agent_ids.len(), max_round_reached, all_teams_evaluated_inner, phase_changed
        );

        let new_phase = if phase_changed {
            engine.state.current_phase.next()
        } else {
            None
        };

        (current_phase, current_round, phase_changed, new_phase)
    };

    // 如果阶段需要切换
    if phase_changed {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            let old_phase = engine.state.current_phase;
            engine.state.advance_phase();

            // 如果进入 TransferRounds 阶段，初始化挖角列表
            if engine.state.current_phase == MarketPhase::TransferRounds {
                log::info!("进入转会轮次阶段，初始化85+能力值选手挖角列表...");
                engine.initialize_transfer_rounds(&players_map);
            }

            log::info!("阶段切换: {:?} -> {:?}", old_phase, engine.state.current_phase);
        }
    }

    // 清空已评估列表（为下一轮做准备）
    if all_teams_evaluated_inner {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.state.evaluated_team_ids.clear();
        }
    }

    // 保存状态、谈判和事件到数据库
    {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            save_market_state_to_db(&pool, &engine.state).await?;
            save_all_negotiations_to_db(&pool, &engine.negotiations).await?;
            save_all_events_to_db(&pool, &engine.events).await?;
        }
    }

    let result = RoundExecutionResult {
        phase,
        round,
        phase_changed,
        new_phase,
        events: Vec::new(),
        new_negotiations: new_negotiations as usize,
        completed_signings: completed_signings as usize,
        summary: format!(
            "第{}轮: 新增{}个谈判, 完成{}笔签约",
            round, new_negotiations, completed_signings
        ),
    };

    // 发送进度事件
    let _ = app_handle.emit("round-execution-complete", &result);

    Ok(ApiResponse::success(result))
}

/// 推进到下一阶段
#[tauri::command]
pub async fn advance_market_phase(
    state: State<'_, AppState>,
) -> Result<ApiResponse<MarketStateSummary>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取引擎
    let mut engine_guard = MARKET_ENGINE.write().await;
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    // 推进阶段
    if engine.state.advance_phase() {
        save_market_state_to_db(&pool, &engine.state).await?;
        Ok(ApiResponse::success(engine.get_state_summary()))
    } else {
        Ok(ApiResponse::error("已是最后阶段"))
    }
}

// ==================== 谈判查询命令 ====================

/// 获取所有活跃谈判
#[tauri::command]
pub async fn get_active_negotiations_llm(
    _state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<NegotiationListInfo>>, String> {
    let engine_guard = MARKET_ENGINE.read().await;
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    let negotiations: Vec<NegotiationListInfo> = engine.get_active_negotiations()
        .iter()
        .map(|n| NegotiationListInfo::from(*n))
        .collect();

    Ok(ApiResponse::success(negotiations))
}

/// 获取谈判详情
#[tauri::command]
pub async fn get_negotiation_detail_llm(
    negotiation_id: u64,
    _state: State<'_, AppState>,
) -> Result<ApiResponse<NegotiationDetailInfo>, String> {
    let engine_guard = MARKET_ENGINE.read().await;
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    match engine.get_negotiation(negotiation_id) {
        Some(neg) => Ok(ApiResponse::success(NegotiationDetailInfo::from(neg))),
        None => Ok(ApiResponse::error("谈判不存在")),
    }
}

// ==================== 策略查询命令 ====================

/// 获取选手转会策略（优先从内存，否则从数据库）
#[tauri::command]
pub async fn get_player_strategy_llm(
    player_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Option<PlayerTransferStrategy>>, String> {
    // 先尝试从内存获取
    {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            if let Some(strategy) = engine.get_player_strategy(player_id) {
                return Ok(ApiResponse::success(Some(strategy.clone())));
            }
        }
    }

    // 内存中没有，从数据库查询
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 从数据库查询策略
    let strategy_row = sqlx::query(
        "SELECT strategy_json FROM player_transfer_strategies WHERE player_id = ? AND save_id = ? AND season_id = ?"
    )
    .bind(player_id as i64)
    .bind(&save_id)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match strategy_row {
        Some(row) => {
            let strategy_json: String = row.get("strategy_json");
            match serde_json::from_str::<PlayerTransferStrategy>(&strategy_json) {
                Ok(strategy) => Ok(ApiResponse::success(Some(strategy))),
                Err(e) => {
                    log::warn!("解析选手策略失败: {}", e);
                    Ok(ApiResponse::success(None))
                }
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 获取球队转会策略（优先从内存，否则从数据库）
#[tauri::command]
pub async fn get_team_strategy_llm(
    team_id: u64,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Option<AITransferStrategy>>, String> {
    log::info!("获取球队策略: team_id={}", team_id);

    // 先尝试从内存获取
    {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            if let Some(strategy) = engine.get_team_strategy(team_id) {
                log::info!("从内存获取到球队 {} 的策略", team_id);
                return Ok(ApiResponse::success(Some(strategy.clone())));
            }
            log::info!("内存中没有球队 {} 的策略", team_id);
        } else {
            log::info!("市场引擎未在内存中");
        }
    }

    // 内存中没有，从数据库查询
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    log::info!("从数据库查询球队策略: team_id={}, save_id={}, season_id={}", team_id, save_id, season_id);

    // 从数据库查询策略
    let strategy_row = sqlx::query(
        "SELECT strategy_json FROM ai_transfer_strategies WHERE team_id = ? AND save_id = ? AND season_id = ?"
    )
    .bind(team_id as i64)
    .bind(&save_id)
    .bind(season_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match strategy_row {
        Some(row) => {
            let strategy_json: String = row.get("strategy_json");
            log::info!("从数据库获取到球队 {} 的策略, JSON长度: {}", team_id, strategy_json.len());
            match serde_json::from_str::<AITransferStrategy>(&strategy_json) {
                Ok(strategy) => Ok(ApiResponse::success(Some(strategy))),
                Err(e) => {
                    log::warn!("解析球队策略失败: {}", e);
                    Ok(ApiResponse::success(None))
                }
            }
        }
        None => {
            log::warn!("数据库中没有球队 {} 的策略", team_id);
            Ok(ApiResponse::success(None))
        }
    }
}

/// 获取所有想离队的选手
#[tauri::command]
pub async fn get_departure_candidates_llm(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<DepartureCandidateInfo>>, String> {
    let engine_guard = MARKET_ENGINE.read().await;
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    // 调试日志
    log::info!(
        "查询离队候选人: 内存中有 {} 个选手策略",
        engine.player_strategies.len()
    );

    let wants_leave_count = engine.player_strategies.iter()
        .filter(|(_, s)| s.wants_to_leave)
        .count();
    log::info!("其中 {} 人想离队", wants_leave_count);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    let candidates = engine.get_departure_candidates();
    let mut result = Vec::new();

    for player_id in candidates {
        if let Some(strategy) = engine.get_player_strategy(player_id) {
            // 获取选手信息
            let player = PlayerRepository::get_by_id(&pool, player_id).await.ok();

            if let Some(p) = player {
                let team_name = if let Some(tid) = p.team_id {
                    TeamRepository::get_by_id(&pool, tid).await.ok()
                        .map(|t| t.name)
                        .unwrap_or_default()
                } else {
                    "自由球员".to_string()
                };

                result.push(DepartureCandidateInfo {
                    player_id: p.id,
                    player_name: p.game_id.clone(),
                    position: p.position.map(|pos| format!("{:?}", pos).to_uppercase()).unwrap_or_default(),
                    ability: p.ability,
                    age: p.age,
                    team_id: p.team_id,
                    team_name,
                    decision_confidence: strategy.decision_confidence,
                    leave_reasoning: strategy.leave_reasoning.clone(),
                    expected_salary: strategy.expected_salary,
                    preferred_teams_count: strategy.preferred_teams.len(),
                });
            }
        }
    }

    Ok(ApiResponse::success(result))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartureCandidateInfo {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub age: u8,
    pub team_id: Option<u64>,
    pub team_name: String,
    pub decision_confidence: u8,
    pub leave_reasoning: String,
    pub expected_salary: u64,
    pub preferred_teams_count: usize,
}

// ==================== 事件查询命令 ====================

/// 获取所有转会市场事件
#[tauri::command]
pub async fn get_llm_market_events(
    _state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<MarketEvent>>, String> {
    let engine_guard = MARKET_ENGINE.read().await;
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    Ok(ApiResponse::success(engine.get_events().to_vec()))
}

/// 获取指定轮次的事件
#[tauri::command]
pub async fn get_llm_market_events_for_round(
    round: u8,
    _state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<MarketEvent>>, String> {
    let engine_guard = MARKET_ENGINE.read().await;
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    let events: Vec<MarketEvent> = engine.get_events_for_round(round)
        .into_iter()
        .cloned()
        .collect();

    Ok(ApiResponse::success(events))
}

// ==================== 重置命令 ====================

/// 重置 LLM 转会市场（清除所有数据，重新开始）
#[tauri::command]
pub async fn reset_llm_transfer_market(
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    log::info!("重置转会市场: save_id={}, season_id={}", save_id, season_id);

    // 清除内存中的引擎
    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        *engine_guard = None;
    }

    // 清除数据库中的转会市场数据（按依赖顺序删除）

    // 1. 先删除有外键依赖的表
    sqlx::query("DELETE FROM offer_responses WHERE offer_id IN (SELECT id FROM offers WHERE negotiation_id IN (SELECT id FROM negotiations WHERE save_id = ? AND season_id = ?))")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM offers WHERE negotiation_id IN (SELECT id FROM negotiations WHERE save_id = ? AND season_id = ?)")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM negotiations WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 2. 删除市场事件
    sqlx::query("DELETE FROM transfer_market_events WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 3. 删除球队市场状态
    sqlx::query("DELETE FROM team_market_states WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 4. 删除主市场状态
    sqlx::query("DELETE FROM transfer_market_states WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 5. 删除选手策略
    sqlx::query("DELETE FROM player_transfer_strategies WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 6. 删除球队策略（两个不同的表）
    sqlx::query("DELETE FROM ai_transfer_strategies WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM team_transfer_strategies WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .ok(); // 表可能不存在，忽略错误

    // 7. 删除续约决策
    sqlx::query("DELETE FROM renewal_decisions WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("转会市场重置完成");

    Ok(ApiResponse::success(()))
}

// ==================== 辅助函数 ====================

/// 保存市场状态到数据库
async fn save_market_state_to_db(
    pool: &sqlx::SqlitePool,
    state: &TransferMarketState,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let phase_str = format!("{:?}", state.current_phase).to_uppercase();
    let free_agent_ids_json = serde_json::to_string(&state.free_agent_ids).unwrap_or_default();
    let active_negotiation_ids_json = serde_json::to_string(&state.active_negotiation_ids).unwrap_or_default();
    let completed_transfer_ids_json = serde_json::to_string(&state.completed_transfer_ids).unwrap_or_default();
    let poachable_player_ids_json = serde_json::to_string(&state.poachable_player_ids).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO transfer_market_states (
            save_id, season_id, current_phase, current_round, max_negotiation_rounds,
            transfer_round, max_transfer_rounds, poachable_player_ids,
            free_agent_ids, active_negotiation_ids, completed_transfer_ids,
            intentions_generated, total_players, strategies_generated, total_teams,
            is_market_stable, stable_rounds_count, is_transfer_stable, transfer_stable_rounds_count,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(save_id, season_id) DO UPDATE SET
            current_phase = excluded.current_phase,
            current_round = excluded.current_round,
            transfer_round = excluded.transfer_round,
            max_transfer_rounds = excluded.max_transfer_rounds,
            poachable_player_ids = excluded.poachable_player_ids,
            free_agent_ids = excluded.free_agent_ids,
            active_negotiation_ids = excluded.active_negotiation_ids,
            completed_transfer_ids = excluded.completed_transfer_ids,
            intentions_generated = excluded.intentions_generated,
            strategies_generated = excluded.strategies_generated,
            is_market_stable = excluded.is_market_stable,
            stable_rounds_count = excluded.stable_rounds_count,
            is_transfer_stable = excluded.is_transfer_stable,
            transfer_stable_rounds_count = excluded.transfer_stable_rounds_count,
            updated_at = excluded.updated_at
        "#
    )
    .bind(&state.save_id)
    .bind(state.season_id as i64)
    .bind(&phase_str)
    .bind(state.current_round as i64)
    .bind(state.max_negotiation_rounds as i64)
    .bind(state.transfer_round as i64)
    .bind(state.max_transfer_rounds as i64)
    .bind(&poachable_player_ids_json)
    .bind(&free_agent_ids_json)
    .bind(&active_negotiation_ids_json)
    .bind(&completed_transfer_ids_json)
    .bind(state.intentions_generated as i64)
    .bind(state.total_players as i64)
    .bind(state.strategies_generated as i64)
    .bind(state.total_teams as i64)
    .bind(if state.is_market_stable { 1i64 } else { 0i64 })
    .bind(state.stable_rounds_count as i64)
    .bind(if state.is_transfer_stable { 1i64 } else { 0i64 })
    .bind(state.transfer_stable_rounds_count as i64)
    .bind(&state.created_at)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 从数据库行解析市场状态
fn parse_market_state_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<TransferMarketState, String> {
    use sqlx::Row;

    let save_id: String = row.get("save_id");
    let season_id: i64 = row.get("season_id");
    let phase_str: String = row.get("current_phase");
    let current_round: i64 = row.get("current_round");
    let max_rounds: i64 = row.get("max_negotiation_rounds");
    let free_agent_ids_json: String = row.get("free_agent_ids");
    let active_negotiation_ids_json: String = row.get("active_negotiation_ids");
    let completed_transfer_ids_json: String = row.get("completed_transfer_ids");
    let intentions_generated: i64 = row.get("intentions_generated");
    let total_players: i64 = row.get("total_players");
    let strategies_generated: i64 = row.get("strategies_generated");
    let total_teams: i64 = row.get("total_teams");
    let is_market_stable: i64 = row.get("is_market_stable");
    let stable_rounds_count: i64 = row.get("stable_rounds_count");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");

    // 新增字段，使用 try_get 以兼容旧数据库
    let transfer_round: i64 = row.try_get("transfer_round").unwrap_or(0);
    let max_transfer_rounds: i64 = row.try_get("max_transfer_rounds").unwrap_or(3);
    let poachable_player_ids_json: String = row.try_get("poachable_player_ids").unwrap_or_default();
    let is_transfer_stable: i64 = row.try_get("is_transfer_stable").unwrap_or(0);
    let transfer_stable_rounds_count: i64 = row.try_get("transfer_stable_rounds_count").unwrap_or(0);

    let current_phase = match phase_str.as_str() {
        // 新的 5 个阶段
        "INTENTION_GENERATION" | "INTENTIONGENERATION" => MarketPhase::IntentionGeneration,
        "STRATEGY_GENERATION" | "STRATEGYGENERATION" => MarketPhase::StrategyGeneration,
        "RENEWAL_PROCESSING" | "RENEWALPROCESSING" => MarketPhase::RenewalProcessing,
        "FREE_MARKET" | "FREEMARKET" => MarketPhase::FreeMarket,
        "TRANSFER_ROUNDS" | "TRANSFERROUNDS" => MarketPhase::TransferRounds,
        "COMPLETED" => MarketPhase::Completed,

        // 旧阶段兼容映射（自动转换为新阶段）
        "INITIALIZATION" => MarketPhase::IntentionGeneration,
        "DEPARTURE_ANNOUNCEMENT" | "DEPARTUREANNOUNCEMENT" => MarketPhase::FreeMarket,
        "INITIAL_BIDDING" | "INITIALBIDDING" => MarketPhase::FreeMarket,
        "NEGOTIATION_ROUNDS" | "NEGOTIATIONROUNDS" => MarketPhase::FreeMarket,
        "LAST_CHANCE" | "LASTCHANCE" => MarketPhase::FreeMarket,
        "FINALIZATION" => MarketPhase::Completed,

        _ => MarketPhase::IntentionGeneration,
    };

    let free_agent_ids: Vec<u64> = serde_json::from_str(&free_agent_ids_json).unwrap_or_default();
    let active_negotiation_ids: Vec<u64> = serde_json::from_str(&active_negotiation_ids_json).unwrap_or_default();
    let completed_transfer_ids: Vec<u64> = serde_json::from_str(&completed_transfer_ids_json).unwrap_or_default();
    let poachable_player_ids: Vec<u64> = serde_json::from_str(&poachable_player_ids_json).unwrap_or_default();

    Ok(TransferMarketState {
        id: 0,
        save_id,
        season_id: season_id as u64,
        current_phase,
        current_round: current_round as u8,
        max_negotiation_rounds: max_rounds as u8,
        transfer_round: transfer_round as u8,
        max_transfer_rounds: max_transfer_rounds as u8,
        free_agent_ids,
        poachable_player_ids,
        active_negotiation_ids,
        completed_transfer_ids,
        team_states: HashMap::new(), // 需要单独加载
        intentions_generated: intentions_generated as u32,
        total_players: total_players as u32,
        strategies_generated: strategies_generated as u32,
        total_teams: total_teams as u32,
        is_market_stable: is_market_stable != 0,
        stable_rounds_count: stable_rounds_count as u8,
        is_transfer_stable: is_transfer_stable != 0,
        transfer_stable_rounds_count: transfer_stable_rounds_count as u8,
        evaluated_team_ids: Vec::new(), // 从数据库加载时默认为空
        created_at,
        updated_at,
    })
}

/// 保存选手策略到数据库
async fn save_player_strategy_to_db(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: i64,
    player_id: u64,
    strategy: &PlayerTransferStrategy,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let strategy_json = serde_json::to_string(strategy).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO player_transfer_strategies (player_id, save_id, season_id, strategy_json, generated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(player_id, save_id, season_id) DO UPDATE SET
            strategy_json = excluded.strategy_json,
            generated_at = excluded.generated_at
        "#
    )
    .bind(player_id as i64)
    .bind(save_id)
    .bind(season_id)
    .bind(&strategy_json)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 保存球队策略到数据库
async fn save_team_strategy_to_db(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: i64,
    team_id: u64,
    strategy: &AITransferStrategy,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let strategy_json = serde_json::to_string(strategy).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO ai_transfer_strategies (team_id, save_id, season_id, strategy_json, generated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(team_id, save_id, season_id) DO UPDATE SET
            strategy_json = excluded.strategy_json,
            generated_at = excluded.generated_at
        "#
    )
    .bind(team_id as i64)
    .bind(save_id)
    .bind(season_id)
    .bind(&strategy_json)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 加载 GM 配置
async fn load_gm_profiles(
    pool: &sqlx::SqlitePool,
    save_id: &str,
) -> Result<HashMap<u64, TeamGMProfile>, String> {
    let rows = sqlx::query("SELECT * FROM team_gm_profiles WHERE save_id = ?")
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut profiles = HashMap::new();

    for row in rows {
        let team_id: i64 = row.get("team_id");
        let personality_str: String = row.get("personality");

        let personality = match personality_str.to_uppercase().as_str() {
            "CHAMPIONSHIP" => GMPersonality::Championship,
            "YOUTH_DEVELOPMENT" | "YOUTHDEVELOPMENT" => GMPersonality::YouthDevelopment,
            "BALANCED" => GMPersonality::Balanced,
            "SPECULATOR" => GMPersonality::Speculator,
            "REBUILDING" => GMPersonality::Rebuilding,
            "CUSTOM" => GMPersonality::Custom,
            _ => GMPersonality::Balanced,
        };

        let sell_agg_str: String = row.get("sell_aggressiveness");
        let sell_aggressiveness = match sell_agg_str.to_uppercase().as_str() {
            "CONSERVATIVE" => crate::models::SellAggressiveness::Conservative,
            "AGGRESSIVE" => crate::models::SellAggressiveness::Aggressive,
            _ => crate::models::SellAggressiveness::Normal,
        };

        let position_priorities_json: String = row.get("position_priorities");
        let position_priorities: HashMap<String, u8> = serde_json::from_str(&position_priorities_json)
            .unwrap_or_default();

        profiles.insert(team_id as u64, TeamGMProfile {
            id: row.get::<i64, _>("id") as u64,
            team_id: team_id as u64,
            save_id: save_id.to_string(),
            personality,
            custom_prompt: row.get("custom_prompt"),
            risk_tolerance: row.get::<i64, _>("risk_tolerance") as u8,
            budget_ratio: row.get::<f64, _>("budget_ratio"),
            sell_aggressiveness,
            preferred_age_min: row.get::<i64, _>("preferred_age_min") as u8,
            preferred_age_max: row.get::<i64, _>("preferred_age_max") as u8,
            min_ability_threshold: row.get::<i64, _>("min_ability_threshold") as u8,
            price_premium_max: row.get::<f64, _>("price_premium_max"),
            position_priorities,
            draft_pick_sell_threshold: row.get::<f64, _>("draft_pick_sell_threshold"),
            draft_pick_bid_aggressiveness: row.get::<f64, _>("draft_pick_bid_aggressiveness"),
            draft_preference_ability_weight: row.get::<f64, _>("draft_preference_ability_weight"),
            draft_young_bias: row.get::<f64, _>("draft_young_bias"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }

    Ok(profiles)
}

/// 查询选手荣誉（简化版）
async fn query_player_honors_simple(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    player_id: u64,
) -> Option<PlayerHonorInfo> {
    let rows = sqlx::query(
        r#"
        SELECT honor_type, tournament_type, tournament_name
        FROM honors
        WHERE save_id = ? AND player_id = ?
        "#
    )
    .bind(save_id)
    .bind(player_id as i64)
    .fetch_all(pool)
    .await
    .ok()?;

    let mut honors = PlayerHonorInfo::default();

    for row in rows {
        let honor_type: String = row.get("honor_type");
        let tournament_type: Option<String> = row.get("tournament_type");

        let honor_upper = honor_type.to_uppercase();
        if honor_upper.contains("CHAMPION") {
            if let Some(ref t_type) = tournament_type {
                match t_type.to_uppercase().as_str() {
                    "WORLDS" | "WORLDCHAMPIONSHIP" | "WORLD_CHAMPIONSHIP" => honors.worlds_championships += 1,
                    "MSI" => honors.msi_championships += 1,
                    _ => honors.regional_championships += 1,
                }
            } else {
                honors.regional_championships += 1;
            }
        } else if honor_upper.contains("MVP") {
            if honor_upper.contains("TOURNAMENT") {
                honors.tournament_mvps += 1;
            } else if honor_upper.contains("FINALS") || honor_upper.contains("FMVP") {
                honors.finals_mvps += 1;
            } else if honor_upper.contains("ANNUAL") || honor_upper.contains("YEARLY") {
                honors.yearly_mvps += 1;
            }
        }
    }

    Some(honors)
}

/// 查询选手表现（简化版）
async fn query_player_performance_simple(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    player_id: u64,
    season_id: u64,
    player_ability: u8,
) -> Option<PlayerPerformanceInfo> {
    let row = sqlx::query(
        r#"
        SELECT games_played, avg_impact, avg_performance, best_performance, consistency_score
        FROM player_season_stats
        WHERE save_id = ? AND player_id = ? AND season_id = ?
        "#
    )
    .bind(save_id)
    .bind(player_id as i64)
    .bind(season_id as i64)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let games_played: i64 = row.try_get("games_played").unwrap_or(0);
    let avg_impact: f64 = row.try_get("avg_impact").unwrap_or(0.0);
    let avg_performance: f64 = row.try_get("avg_performance").unwrap_or(0.0);
    let best_performance: f64 = row.try_get("best_performance").unwrap_or(0.0);
    let consistency_score: f64 = row.try_get("consistency_score").unwrap_or(50.0);

    let ability_diff = avg_performance - player_ability as f64;
    let performance_tier = if avg_performance >= 90.0 { "顶级表现" }
    else if avg_performance >= 80.0 { "优秀表现" }
    else if avg_performance >= 70.0 { "合格表现" }
    else if avg_performance >= 60.0 { "一般表现" }
    else if games_played > 0 { "表现欠佳" }
    else { "无数据" };

    Some(PlayerPerformanceInfo {
        games_played: games_played as u32,
        avg_impact,
        avg_performance,
        best_performance,
        consistency_score,
        performance_tier: performance_tier.to_string(),
        ability_diff,
    })
}

/// 查询球队荣誉（简化版）
async fn query_team_honors_simple(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    team_id: u64,
    _current_season: u64,
    roster: &[Player],
) -> Option<TeamHonorInfo> {
    let rows = sqlx::query(
        r#"
        SELECT honor_type, tournament_type
        FROM honors
        WHERE save_id = ? AND team_id = ?
        "#
    )
    .bind(save_id)
    .bind(team_id as i64)
    .fetch_all(pool)
    .await
    .ok()?;

    let mut honors = TeamHonorInfo::default();
    honors.star_player_count = roster.iter().filter(|p| p.ability >= 90).count() as u32;
    honors.has_star_players = honors.star_player_count > 0;

    for row in rows {
        let honor_type: String = row.get("honor_type");
        let tournament_type: Option<String> = row.get("tournament_type");

        let honor_upper = honor_type.to_uppercase();
        if honor_upper.contains("CHAMPION") {
            if let Some(ref t_type) = tournament_type {
                match t_type.to_uppercase().as_str() {
                    "WORLDS" | "WORLDCHAMPIONSHIP" | "WORLD_CHAMPIONSHIP" => honors.worlds_championships += 1,
                    "MSI" => honors.msi_championships += 1,
                    _ => honors.regional_championships += 1,
                }
            } else {
                honors.regional_championships += 1;
            }
        }
    }

    Some(honors)
}

/// 查询阵容荣誉（简化版）
async fn query_roster_honors_simple(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    roster: &[Player],
) -> Vec<RosterPlayerHonorSummary> {
    let mut summaries = Vec::new();

    for player in roster {
        if player.status != PlayerStatus::Active {
            continue;
        }

        let honor_stats = sqlx::query(
            r#"
            SELECT
                COUNT(CASE WHEN honor_type LIKE '%CHAMPION%' OR honor_type = 'Champion' THEN 1 END) as championship_count,
                COUNT(CASE WHEN honor_type LIKE '%MVP%' THEN 1 END) as mvp_count
            FROM honors
            WHERE save_id = ? AND player_id = ?
            "#
        )
        .bind(save_id)
        .bind(player.id as i64)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let (championship_count, mvp_count) = honor_stats
            .map(|row| (
                row.get::<i64, _>("championship_count") as u32,
                row.get::<i64, _>("mvp_count") as u32,
            ))
            .unwrap_or((0, 0));

        let honor_summary = if championship_count > 0 || mvp_count > 0 {
            let mut parts = Vec::new();
            if championship_count > 0 { parts.push(format!("{}冠", championship_count)); }
            if mvp_count > 0 { parts.push(format!("{}MVP", mvp_count)); }
            parts.join(" ")
        } else {
            "无荣誉".to_string()
        };

        let position_str = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        summaries.push(RosterPlayerHonorSummary {
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: position_str,
            ability: player.ability,
            is_core: player.is_starter,
            championship_count,
            mvp_count,
            honor_summary,
        });
    }

    summaries
}

/// 查询阵容表现（简化版）
async fn query_roster_performance_simple(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    roster: &[Player],
    season_id: u64,
) -> Vec<PlayerPerformanceSummary> {
    let mut summaries = Vec::new();

    for player in roster {
        if player.status != PlayerStatus::Active {
            continue;
        }

        let perf_stats = sqlx::query(
            r#"
            SELECT games_played, avg_impact, avg_performance, best_performance, consistency_score
            FROM player_season_stats
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(save_id)
        .bind(player.id as i64)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let (games_played, avg_impact, avg_performance, best_performance, consistency_score) =
            perf_stats
                .map(|row| (
                    row.try_get::<i64, _>("games_played").unwrap_or(0) as u32,
                    row.try_get::<f64, _>("avg_impact").unwrap_or(0.0),
                    row.try_get::<f64, _>("avg_performance").unwrap_or(0.0),
                    row.try_get::<f64, _>("best_performance").unwrap_or(0.0),
                    row.try_get::<f64, _>("consistency_score").unwrap_or(50.0),
                ))
                .unwrap_or((0, 0.0, 0.0, 0.0, 50.0));

        let ability_diff = avg_performance - player.ability as f64;
        let performance_tier = if avg_performance >= 90.0 { "顶级表现" }
        else if avg_performance >= 80.0 { "优秀表现" }
        else if avg_performance >= 70.0 { "合格表现" }
        else if avg_performance >= 60.0 { "一般表现" }
        else if games_played > 0 { "表现欠佳" }
        else { "无数据" };

        let position_str = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        summaries.push(PlayerPerformanceSummary {
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: position_str,
            games_played,
            avg_impact,
            avg_performance,
            best_performance,
            consistency_score,
            performance_tier: performance_tier.to_string(),
            ability_diff,
        });
    }

    summaries
}

// ==================== 重试失败球队 ====================

/// 获取失败的球队列表
#[tauri::command]
pub async fn get_failed_teams() -> Result<ApiResponse<Vec<u64>>, String> {
    let failed_ids = FAILED_TEAM_IDS.read().await;
    Ok(ApiResponse::success(failed_ids.clone()))
}

/// 重试生成失败球队的策略
#[tauri::command]
pub async fn retry_failed_team_strategies(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<GenerationProgress>, String> {
    // 获取失败的球队ID
    let failed_ids: Vec<u64> = {
        let ids = FAILED_TEAM_IDS.read().await;
        ids.clone()
    };

    if failed_ids.is_empty() {
        return Ok(ApiResponse::success(GenerationProgress {
            task_type: "retry_team_strategies".to_string(),
            current: 0,
            total: 0,
            percentage: 100,
            current_item: None,
            is_completed: true,
            errors: vec![],
        }));
    }

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 获取 LLM 配置
    let llm_config = get_llm_config()
        .ok_or("LLM 未配置，请先在设置中配置 API Key")?;
    let llm_service = LLMTransferService::new(llm_config);

    // 获取所有球队
    let teams = TeamRepository::get_all(&pool, &save_id).await
        .map_err(|e| e.to_string())?;
    let teams_map: HashMap<u64, Team> = teams.iter().map(|t| (t.id, t.clone())).collect();

    // 获取所有选手
    let players = PlayerRepository::get_all_active(&pool, &save_id).await
        .map_err(|e| e.to_string())?;

    let mut players_by_team: HashMap<u64, Vec<Player>> = HashMap::new();
    for player in &players {
        if player.status == PlayerStatus::Active {
            if let Some(team_id) = player.team_id {
                players_by_team.entry(team_id).or_default().push(player.clone());
            }
        }
    }

    // 获取自由球员（无球队的选手）
    let fa_players: Vec<Player> = players.iter()
        .filter(|p| p.team_id.is_none() && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 获取想离队的选手（从引擎中读取已生成的选手策略）
    let departure_player_ids: Vec<u64> = {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            engine.player_strategies.iter()
                .filter(|(_, s)| s.wants_to_leave)
                .map(|(id, _)| *id)
                .collect()
        } else {
            Vec::new()
        }
    };

    // 想离队的选手也作为可追逐目标
    let departure_players: Vec<Player> = players.iter()
        .filter(|p| departure_player_ids.contains(&p.id) && p.status == PlayerStatus::Active)
        .cloned()
        .collect();

    // 合并自由球员和想离队选手作为可追逐目标
    let mut all_available_players = fa_players.clone();
    all_available_players.extend(departure_players.clone());

    let free_agents: Vec<FreeAgentInfo> = all_available_players
        .into_iter()
        .map(|p| {
            let market_value = p.calculate_market_value();
            let salary_exp = crate::models::calculate_expected_salary(market_value);
            FreeAgentInfo {
                agent: FreeAgent {
                    id: 0,
                    save_id: save_id.clone(),
                    season_id: season_id as u64,
                    player_id: p.id,
                    salary_demand: salary_exp.expected,
                    reason: FreeAgentReason::ContractExpire,
                    status: FreeAgentStatus::Available,
                },
                player: p.clone(),
                market_value,
                expected_salary: salary_exp.expected,
                minimum_salary: salary_exp.minimum,
            }
        })
        .collect();

    // 获取 GM 配置
    let gm_profiles = load_gm_profiles(&pool, &save_id).await?;

    let total = failed_ids.len() as u32;
    let mut success_count = 0u32;
    let mut new_errors: Vec<String> = Vec::new();

    for (index, team_id) in failed_ids.iter().enumerate() {
        // 获取球队
        let team = match teams_map.get(team_id) {
            Some(t) => t.clone(),
            None => {
                new_errors.push(format!("球队 {} 不存在", team_id));
                continue;
            }
        };

        log::info!("重试生成球队 {} 的策略 ({}/{})", team.name, index + 1, total);

        // 发送进度事件
        let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
            current: (index + 1) as u32,
            total,
            team_name: team.name.clone(),
            status: "processing".to_string(),
        });

        // 获取球队阵容
        let roster = players_by_team.get(team_id).cloned().unwrap_or_default();

        // 获取 GM 配置
        let profile = gm_profiles.get(team_id).cloned().unwrap_or_else(|| {
            TeamGMProfile::new(*team_id, save_id.clone())
        });

        // 调用 LLM 生成策略
        match llm_service.generate_strategy(
            &team,
            &roster,
            &profile,
            &free_agents,
            &players_by_team,
            &save_id,
            season_id as u64,
            None, // team_honors
            None, // roster_honors
            None, // roster_performance
        ).await {
            Ok(strategy) => {
                // 保存到引擎
                {
                    let mut engine_guard = MARKET_ENGINE.write().await;
                    if let Some(engine) = engine_guard.as_mut() {
                        engine.team_strategies.insert(*team_id, strategy.clone());
                        if let Some(team_state) = engine.state.get_team_state_mut(*team_id) {
                            team_state.strategy_generated = true;
                        }
                    }
                }

                // 保存到数据库
                save_team_strategy_to_db(&pool, &save_id, season_id, *team_id, &strategy).await?;
                success_count += 1;

                // 从失败列表中移除
                {
                    let mut failed = FAILED_TEAM_IDS.write().await;
                    failed.retain(|id| id != team_id);
                }

                let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
                    current: (index + 1) as u32,
                    total,
                    team_name: team.name.clone(),
                    status: "success".to_string(),
                });
            }
            Err(e) => {
                new_errors.push(format!("{}: {}", team.name, e));
                let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
                    current: (index + 1) as u32,
                    total,
                    team_name: team.name.clone(),
                    status: "failed".to_string(),
                });
            }
        }
    }

    log::info!("重试完成: 成功 {}/{}, 仍失败 {}", success_count, total, new_errors.len());

    // 发送完成事件
    let _ = app_handle.emit("strategy-generation-progress", StrategyGenerationProgressEvent {
        current: total,
        total,
        team_name: "".to_string(),
        status: "completed".to_string(),
    });

    Ok(ApiResponse::success(GenerationProgress {
        task_type: "retry_team_strategies".to_string(),
        current: success_count,
        total,
        percentage: 100,
        current_item: None,
        is_completed: true,
        errors: new_errors,
    }))
}

// ==================== 续约结果持久化 ====================

/// 保存续约决策到数据库
async fn save_renewal_decision_to_db(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: i64,
    decision: &crate::models::RenewalDecision,
) -> Result<(), String> {
    let team_analysis_json = serde_json::to_string(&decision.team_analysis).unwrap_or_default();
    let player_analysis_json = serde_json::to_string(&decision.player_analysis).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO renewal_decisions (
            save_id, season_id, player_id, player_name, team_id, team_name,
            team_wants_renewal, team_rejection_reason, offered_salary, offered_years,
            player_accepts, player_rejection_reason, renewal_successful,
            final_salary, final_years, team_analysis, player_analysis, summary
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(save_id)
    .bind(season_id)
    .bind(decision.player_id as i64)
    .bind(&decision.player_name)
    .bind(decision.team_id as i64)
    .bind(&decision.team_name)
    .bind(if decision.team_wants_renewal { 1i64 } else { 0i64 })
    .bind(&decision.team_rejection_reason)
    .bind(decision.offered_salary as i64)
    .bind(decision.offered_years as i64)
    .bind(if decision.player_accepts { 1i64 } else { 0i64 })
    .bind(&decision.player_rejection_reason)
    .bind(if decision.renewal_successful { 1i64 } else { 0i64 })
    .bind(decision.final_salary.map(|s| s as i64))
    .bind(decision.final_years.map(|y| y as i64))
    .bind(&team_analysis_json)
    .bind(&player_analysis_json)
    .bind(&decision.summary)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 从数据库加载所有续约决策（内部辅助函数）
async fn load_all_renewal_decisions(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: i64,
) -> Result<(Vec<crate::models::RenewalDecision>, u32, u32, u32), String> {
    use crate::models::{RenewalDecision, RenewalAnalysisStep};

    let rows = sqlx::query(
        "SELECT * FROM renewal_decisions WHERE save_id = ? AND season_id = ? ORDER BY id"
    )
    .bind(save_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut decisions: Vec<RenewalDecision> = Vec::new();
    let mut successful_renewals = 0u32;
    let mut team_rejections = 0u32;
    let mut player_rejections = 0u32;

    for row in rows {
        let team_analysis_json: String = row.get("team_analysis");
        let player_analysis_json: String = row.get("player_analysis");

        let team_analysis: Vec<RenewalAnalysisStep> = serde_json::from_str(&team_analysis_json)
            .unwrap_or_default();
        let player_analysis: Vec<RenewalAnalysisStep> = serde_json::from_str(&player_analysis_json)
            .unwrap_or_default();

        let team_wants_renewal: i64 = row.get("team_wants_renewal");
        let player_accepts: i64 = row.get("player_accepts");
        let renewal_successful: i64 = row.get("renewal_successful");

        let decision = RenewalDecision {
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get("player_name"),
            team_id: row.get::<i64, _>("team_id") as u64,
            team_name: row.get("team_name"),
            team_wants_renewal: team_wants_renewal != 0,
            team_rejection_reason: row.get("team_rejection_reason"),
            offered_salary: row.get::<i64, _>("offered_salary") as u64,
            offered_years: row.get::<i64, _>("offered_years") as u8,
            player_accepts: player_accepts != 0,
            player_rejection_reason: row.get("player_rejection_reason"),
            renewal_successful: renewal_successful != 0,
            final_salary: row.get::<Option<i64>, _>("final_salary").map(|s| s as u64),
            final_years: row.get::<Option<i64>, _>("final_years").map(|y| y as u8),
            team_analysis,
            player_analysis,
            summary: row.get("summary"),
        };

        if decision.renewal_successful {
            successful_renewals += 1;
        } else if !decision.team_wants_renewal {
            team_rejections += 1;
        } else {
            player_rejections += 1;
        }

        decisions.push(decision);
    }

    Ok((decisions, successful_renewals, team_rejections, player_rejections))
}

/// 从数据库加载续约结果
#[tauri::command]
pub async fn get_renewal_results(
    state: State<'_, AppState>,
) -> Result<ApiResponse<crate::models::RenewalProcessingResult>, String> {
    use crate::models::{RenewalDecision, RenewalAnalysisStep, RenewalProcessingResult};

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 从数据库加载续约决策
    let rows = sqlx::query(
        "SELECT * FROM renewal_decisions WHERE save_id = ? AND season_id = ? ORDER BY id"
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut decisions: Vec<RenewalDecision> = Vec::new();
    let mut successful_renewals = 0u32;
    let mut team_rejections = 0u32;
    let mut player_rejections = 0u32;

    for row in rows {
        let team_analysis_json: String = row.get("team_analysis");
        let player_analysis_json: String = row.get("player_analysis");

        let team_analysis: Vec<RenewalAnalysisStep> = serde_json::from_str(&team_analysis_json)
            .unwrap_or_default();
        let player_analysis: Vec<RenewalAnalysisStep> = serde_json::from_str(&player_analysis_json)
            .unwrap_or_default();

        let team_wants_renewal: i64 = row.get("team_wants_renewal");
        let player_accepts: i64 = row.get("player_accepts");
        let renewal_successful: i64 = row.get("renewal_successful");

        let decision = RenewalDecision {
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get("player_name"),
            team_id: row.get::<i64, _>("team_id") as u64,
            team_name: row.get("team_name"),
            team_wants_renewal: team_wants_renewal != 0,
            team_rejection_reason: row.get("team_rejection_reason"),
            offered_salary: row.get::<i64, _>("offered_salary") as u64,
            offered_years: row.get::<i64, _>("offered_years") as u8,
            player_accepts: player_accepts != 0,
            player_rejection_reason: row.get("player_rejection_reason"),
            renewal_successful: renewal_successful != 0,
            final_salary: row.get::<Option<i64>, _>("final_salary").map(|s| s as u64),
            final_years: row.get::<Option<i64>, _>("final_years").map(|y| y as u8),
            team_analysis,
            player_analysis,
            summary: row.get("summary"),
        };

        if decision.renewal_successful {
            successful_renewals += 1;
        } else if !decision.team_wants_renewal {
            team_rejections += 1;
        } else {
            player_rejections += 1;
        }

        decisions.push(decision);
    }

    Ok(ApiResponse::success(RenewalProcessingResult {
        total_processed: decisions.len() as u32,
        successful_renewals,
        team_rejections,
        player_rejections,
        decisions,
        errors: vec![],
    }))
}

/// 修复：将想离队的选手加入自由球员列表（临时命令）
#[tauri::command]
pub async fn fix_add_departure_to_free_agents(
    state: State<'_, AppState>,
) -> Result<ApiResponse<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    let mut engine_guard = MARKET_ENGINE.write().await;
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return Ok(ApiResponse::error("转会市场未初始化")),
    };

    // 获取所有想离队的选手
    let departure_candidates: Vec<u64> = engine.player_strategies.iter()
        .filter(|(_, s)| s.wants_to_leave)
        .map(|(id, _)| *id)
        .collect();

    let count = departure_candidates.len() as u32;

    // 加入自由球员列表
    for player_id in departure_candidates {
        engine.state.add_free_agent(player_id);
    }

    // 保存状态
    save_market_state_to_db(&pool, &engine.state).await?;

    log::info!("已修复：将 {} 名想离队的选手加入自由球员市场", count);

    Ok(ApiResponse::success(count))
}

// ==================== 谈判和报价持久化 ====================

/// 保存谈判到数据库
async fn save_negotiation_to_db(
    pool: &sqlx::SqlitePool,
    negotiation: &Negotiation,
) -> Result<i64, String> {
    let status_str = format!("{:?}", negotiation.status).to_uppercase();
    let competing_team_ids_json = serde_json::to_string(&negotiation.competing_team_ids).unwrap_or_default();

    // 先尝试查找是否存在
    let existing = sqlx::query(
        "SELECT id FROM negotiations WHERE save_id = ? AND season_id = ? AND player_id = ?"
    )
    .bind(&negotiation.save_id)
    .bind(negotiation.season_id as i64)
    .bind(negotiation.player_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = existing {
        // 更新现有记录
        let id: i64 = row.get("id");
        sqlx::query(
            r#"
            UPDATE negotiations SET
                status = ?, current_round = ?, competing_team_ids = ?,
                final_team_id = ?, final_team_name = ?, final_salary = ?,
                final_years = ?, final_starter = ?, updated_at = datetime('now')
            WHERE id = ?
            "#
        )
        .bind(&status_str)
        .bind(negotiation.current_round as i64)
        .bind(&competing_team_ids_json)
        .bind(negotiation.final_team_id.map(|id| id as i64))
        .bind(&negotiation.final_team_name)
        .bind(negotiation.final_salary.map(|s| s as i64))
        .bind(negotiation.final_years.map(|y| y as i64))
        .bind(negotiation.final_starter.map(|s| if s { 1i64 } else { 0i64 }))
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        // 插入新记录
        let result = sqlx::query(
            r#"
            INSERT INTO negotiations (
                save_id, season_id, player_id, player_name, player_position, player_ability,
                from_team_id, from_team_name, status, current_round, max_rounds, competing_team_ids,
                final_team_id, final_team_name, final_salary, final_years, final_starter
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&negotiation.save_id)
        .bind(negotiation.season_id as i64)
        .bind(negotiation.player_id as i64)
        .bind(&negotiation.player_name)
        .bind(&negotiation.player_position)
        .bind(negotiation.player_ability as i64)
        .bind(negotiation.from_team_id.map(|id| id as i64))
        .bind(&negotiation.from_team_name)
        .bind(&status_str)
        .bind(negotiation.current_round as i64)
        .bind(negotiation.max_rounds as i64)
        .bind(&competing_team_ids_json)
        .bind(negotiation.final_team_id.map(|id| id as i64))
        .bind(&negotiation.final_team_name)
        .bind(negotiation.final_salary.map(|s| s as i64))
        .bind(negotiation.final_years.map(|y| y as i64))
        .bind(negotiation.final_starter.map(|s| if s { 1i64 } else { 0i64 }))
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(result.last_insert_rowid())
    }
}

/// 保存报价到数据库
async fn save_offer_to_db(
    pool: &sqlx::SqlitePool,
    offer: &Offer,
    db_negotiation_id: i64,
) -> Result<i64, String> {
    let status_str = format!("{:?}", offer.status).to_uppercase();
    let analysis_steps_json = serde_json::to_string(&offer.analysis_steps).unwrap_or_default();

    // 先尝试查找是否存在（根据 negotiation_id, from_team_id, round 唯一标识）
    let existing = sqlx::query(
        "SELECT id FROM offers WHERE negotiation_id = ? AND from_team_id = ? AND round = ?"
    )
    .bind(db_negotiation_id)
    .bind(offer.from_team_id as i64)
    .bind(offer.round as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = existing {
        // 更新现有记录
        let id: i64 = row.get("id");
        sqlx::query(
            r#"
            UPDATE offers SET
                salary_offer = ?, contract_years = ?, guarantee_starter = ?,
                signing_bonus = ?, transfer_fee = ?, status = ?,
                offer_reasoning = ?, analysis_steps = ?, updated_at = datetime('now')
            WHERE id = ?
            "#
        )
        .bind(offer.salary_offer as i64)
        .bind(offer.contract_years as i64)
        .bind(if offer.guarantee_starter { 1i64 } else { 0i64 })
        .bind(offer.signing_bonus as i64)
        .bind(offer.transfer_fee as i64)
        .bind(&status_str)
        .bind(&offer.offer_reasoning)
        .bind(&analysis_steps_json)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        // 插入新记录
        let result = sqlx::query(
            r#"
            INSERT INTO offers (
                negotiation_id, from_team_id, from_team_name, to_player_id,
                round, salary_offer, contract_years, guarantee_starter,
                signing_bonus, transfer_fee, status, offer_reasoning, analysis_steps
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(db_negotiation_id)
        .bind(offer.from_team_id as i64)
        .bind(&offer.from_team_name)
        .bind(offer.to_player_id as i64)
        .bind(offer.round as i64)
        .bind(offer.salary_offer as i64)
        .bind(offer.contract_years as i64)
        .bind(if offer.guarantee_starter { 1i64 } else { 0i64 })
        .bind(offer.signing_bonus as i64)
        .bind(offer.transfer_fee as i64)
        .bind(&status_str)
        .bind(&offer.offer_reasoning)
        .bind(&analysis_steps_json)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!(
                "保存报价失败: negotiation_id={}, from_team_id={}, to_player_id={}, round={}, error={}",
                db_negotiation_id, offer.from_team_id, offer.to_player_id, offer.round, e
            );
            e.to_string()
        })?;
        Ok(result.last_insert_rowid())
    }
}

/// 保存市场事件到数据库
async fn save_market_event_to_db(
    pool: &sqlx::SqlitePool,
    event: &MarketEvent,
) -> Result<i64, String> {
    let event_type_str = format!("{:?}", event.event_type).to_uppercase();
    let phase_str = format!("{:?}", event.phase).to_uppercase();

    // 检查是否已存在相同事件（基于 save_id, season_id, event_type, round, title）
    let existing = sqlx::query(
        "SELECT id FROM transfer_market_events WHERE save_id = ? AND season_id = ? AND event_type = ? AND round = ? AND title = ?"
    )
    .bind(&event.save_id)
    .bind(event.season_id as i64)
    .bind(&event_type_str)
    .bind(event.round as i64)
    .bind(&event.title)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing.is_some() {
        // 事件已存在，跳过
        return Ok(0);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO transfer_market_events (
            save_id, season_id, event_type, phase, round,
            player_id, player_name, team_id, team_name,
            secondary_team_id, secondary_team_name,
            amount, title, description, ai_analysis
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&event.save_id)
    .bind(event.season_id as i64)
    .bind(&event_type_str)
    .bind(&phase_str)
    .bind(event.round as i64)
    .bind(event.player_id.and_then(|id| if id > 0 { Some(id as i64) } else { None }))
    .bind(&event.player_name)
    .bind(event.team_id.and_then(|id| if id > 0 { Some(id as i64) } else { None }))
    .bind(&event.team_name)
    .bind(event.secondary_team_id.and_then(|id| if id > 0 { Some(id as i64) } else { None }))
    .bind(&event.secondary_team_name)
    .bind(event.amount.map(|a| a as i64))
    .bind(&event.title)
    .bind(&event.description)
    .bind(&event.ai_analysis)
    .execute(pool)
    .await
    .map_err(|e| {
        log::error!(
            "保存事件失败: type={:?}, player_id={:?}, team_id={:?}, title={}, error={}",
            event.event_type, event.player_id, event.team_id, event.title, e
        );
        e.to_string()
    })?;

    Ok(result.last_insert_rowid())
}

/// 保存所有谈判和报价到数据库
async fn save_all_negotiations_to_db(
    pool: &sqlx::SqlitePool,
    negotiations: &HashMap<u64, Negotiation>,
) -> Result<(), String> {
    for (_, negotiation) in negotiations {
        let db_neg_id = save_negotiation_to_db(pool, negotiation).await?;

        // 保存该谈判的所有报价
        for offer in &negotiation.offers {
            save_offer_to_db(pool, offer, db_neg_id).await?;
        }
    }
    Ok(())
}

/// 保存所有市场事件到数据库
async fn save_all_events_to_db(
    pool: &sqlx::SqlitePool,
    events: &[MarketEvent],
) -> Result<(), String> {
    for event in events {
        save_market_event_to_db(pool, event).await?;
    }
    Ok(())
}

// ==================== LLM 任务追踪命令 ====================

/// 获取 LLM 任务进度
#[tauri::command]
pub async fn get_llm_task_progress(
    save_id: String,
    season_id: u64,
    task_type: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<TaskStats>, String> {
    let guard = state.db.read().await;
    let db_manager = guard.as_ref().ok_or("数据库未初始化")?;
    let pool = db_manager.get_pool().await.map_err(|e| e.to_string())?;

    let task_type_enum = match task_type.as_str() {
        "intention" => TaskType::Intention,
        "strategy" => TaskType::Strategy,
        "renewal" => TaskType::Renewal,
        "free_market" => TaskType::FreeMarket,
        "poaching" => TaskType::Poaching,
        _ => return Ok(ApiResponse::error("无效的任务类型")),
    };

    let stats = LLMTaskLogRepository::get_task_stats(&pool, &save_id, season_id, task_type_enum)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(stats))
}

/// 重试失败的 LLM 任务
#[tauri::command]
pub async fn retry_failed_llm_tasks(
    save_id: String,
    season_id: u64,
    task_type: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<u32>, String> {
    let guard = state.db.read().await;
    let db_manager = guard.as_ref().ok_or("数据库未初始化")?;
    let pool = db_manager.get_pool().await.map_err(|e| e.to_string())?;

    let task_type_enum = match task_type.as_str() {
        "intention" => TaskType::Intention,
        "strategy" => TaskType::Strategy,
        "renewal" => TaskType::Renewal,
        _ => return Ok(ApiResponse::error("当前只支持重试 intention/strategy/renewal 任务")),
    };

    // 查询失败的任务
    let failed_tasks = LLMTaskLogRepository::get_failed_tasks(&pool, &save_id, season_id, Some(task_type_enum))
        .await
        .map_err(|e| e.to_string())?;

    // 过滤掉超过最大重试次数的
    let retryable: Vec<_> = failed_tasks.iter()
        .filter(|t| t.can_retry())
        .collect();

    if retryable.is_empty() {
        if failed_tasks.is_empty() {
            return Ok(ApiResponse { success: true, data: Some(0), error: Some("没有失败的任务".to_string()) });
        } else {
            return Ok(ApiResponse { success: true, data: Some(0), error: Some("所有失败任务都已达到最大重试次数".to_string()) });
        }
    }

    log::info!("重试 {} 个失败的 {} 任务", retryable.len(), task_type);

    // 根据任务类型调用相应的重试逻辑
    let success_count = match task_type_enum {
        TaskType::Intention => retry_player_intentions(&pool, &save_id, season_id, &retryable, &app_handle).await?,
        TaskType::Strategy => retry_team_strategies(&pool, &save_id, season_id, &retryable, &app_handle).await?,
        TaskType::Renewal => retry_renewals(&pool, &save_id, season_id, &retryable, &app_handle).await?,
        _ => 0,
    };

    log::info!("重试完成，成功 {} 个，失败 {} 个", success_count, retryable.len() as u32 - success_count);
    Ok(ApiResponse::success(success_count))
}

/// 重试选手意愿生成
async fn retry_player_intentions(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
    failed_tasks: &[&LLMTaskLog],
    app_handle: &tauri::AppHandle,
) -> Result<u32, String> {
    use futures::future::join_all;

    let llm_service = std::sync::Arc::new(
        create_llm_service().ok_or("LLM 服务未配置")?
    );

    // 获取需要重试的选手
    let player_ids: Vec<u64> = failed_tasks.iter().map(|t| t.entity_id).collect();
    let players = PlayerRepository::get_all_active(pool, save_id).await.map_err(|e| e.to_string())?;
    let players_map: HashMap<u64, Player> = players.iter().map(|p| (p.id, p.clone())).collect();

    let teams = TeamRepository::get_all(pool, save_id).await.map_err(|e| e.to_string())?;
    let teams_map: HashMap<u64, Team> = teams.iter().map(|t| (t.id, t.clone())).collect();

    // 并发重试
    let retry_futures: Vec<_> = player_ids.iter().filter_map(|&pid| {
        let player = players_map.get(&pid)?;
        let team = player.team_id.and_then(|tid| teams_map.get(&tid))?;

        let llm = llm_service.clone();
        let player = player.clone();
        let team = team.clone();
        let save_id = save_id.to_string();

        Some(async move {
            // 更新状态为 running
            let mut task = LLMTaskLog::new(save_id.clone(), season_id, TaskType::Intention, player.id, "player".to_string());
            task.mark_running();
            let _ = LLMTaskLogRepository::upsert(pool, &task).await;

            // 调用 LLM
            let result = llm.generate_player_strategy(
                &player,
                &team,
                &[], // roster
                &[], // available_teams
                &save_id,
                season_id,
                None, // honors
                None, // performance
                None, // current_team_rank
            ).await;

            // 更新结果
            match result {
                Ok(strategy) => {
                    // 保存策略
                    save_player_strategy_to_db(pool, &save_id, season_id as i64, player.id, &strategy).await.ok();

                    // 标记成功
                    task.mark_success();
                    let _ = LLMTaskLogRepository::upsert(pool, &task).await;

                    Ok(player.id)
                }
                Err(e) => {
                    // 标记失败
                    task.mark_failed(e.clone());
                    let _ = LLMTaskLogRepository::upsert(pool, &task).await;

                    Err(e)
                }
            }
        })
    }).collect();

    let results = join_all(retry_futures).await;
    let success_count = results.iter().filter(|r| r.is_ok()).count() as u32;

    // 发送进度事件
    let _ = app_handle.emit("intention-retry-complete", success_count);

    Ok(success_count)
}

/// 重试战队策略生成
async fn retry_team_strategies(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
    failed_tasks: &[&LLMTaskLog],
    app_handle: &tauri::AppHandle,
) -> Result<u32, String> {
    // 类似实现...
    log::warn!("retry_team_strategies 尚未实现");
    Ok(0)
}

/// 重试续约评估
async fn retry_renewals(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
    failed_tasks: &[&LLMTaskLog],
    app_handle: &tauri::AppHandle,
) -> Result<u32, String> {
    // 类似实现...
    log::warn!("retry_renewals 尚未实现");
    Ok(0)
}


// ==================== 规则驱动转会市场命令 ====================

/// 执行规则驱动的转会市场轮次（高效版本）
#[tauri::command]
pub async fn execute_rule_based_market_round(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<RoundExecutionResult>, String> {
    use crate::engines::scouting::scout_transfer_candidates;
    use crate::engines::offer_decision::decide_offer;
    use crate::engines::player_decision::evaluate_offers_and_choose;

    let start_time = std::time::Instant::now();
    log::info!("========== 🚀 规则驱动转会市场轮次 ==========");

    // ========== 步骤1：加载基础数据 ==========

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("数据库未初始化")),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("没有活动的存档")),
    };

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = save_row.get("current_season");

    // 加载球队和选手
    let teams = TeamRepository::get_all(&pool, &save_id).await.map_err(|e| e.to_string())?;
    let teams_map: HashMap<u64, Team> = teams.iter().map(|t| (t.id, t.clone())).collect();

    let players = PlayerRepository::get_all_active(&pool, &save_id).await.map_err(|e| e.to_string())?;
    let players_map: HashMap<u64, Player> = players.iter().map(|p| (p.id, p.clone())).collect();

    // 获取当前阶段
    let (current_phase, current_round, poachable_player_ids) = {
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场未初始化")),
        };
        (
            engine.state.current_phase,
            engine.state.current_round + 1,
            engine.state.poachable_player_ids.clone(),
        )
    };

    let is_poaching = current_phase == MarketPhase::TransferRounds;

    log::info!("当前阶段: {:?}，轮次: {}", current_phase, current_round);
    log::info!("使用规则引擎（NBA 2K 风格）");

    // 获取可用选手列表
    let available_players: Vec<Player> = if is_poaching {
        poachable_player_ids.iter()
            .filter_map(|&id| players_map.get(&id).cloned())
            .collect()
    } else {
        // 自由市场
        let engine_guard = MARKET_ENGINE.read().await;
        let engine = engine_guard.as_ref().unwrap();
        engine.state.free_agent_ids.iter()
            .filter_map(|&id| players_map.get(&id).cloned())
            .collect()
    };

    log::info!("可用选手数量: {}", available_players.len());

    if available_players.is_empty() {
        return Ok(ApiResponse::error("没有可用选手"));
    }

    // 加载GM配置（使用默认值）
    let gm_profiles: HashMap<u64, TeamGMProfile> = teams.iter()
        .map(|t| {
            let profile = TeamGMProfile {
                id: 0,
                team_id: t.id,
                save_id: save_id.clone(),
                personality: GMPersonality::Balanced,  // 默认稳健型
                custom_prompt: None,
                risk_tolerance: 50,
                budget_ratio: 0.6,
                preferred_age_min: 20,
                preferred_age_max: 28,
                min_ability_threshold: 75,
                price_premium_max: 1.1,
                sell_aggressiveness: crate::models::SellAggressiveness::Normal,
                position_priorities: HashMap::new(),
                draft_pick_sell_threshold: 0.5,
                draft_pick_bid_aggressiveness: 1.0,
                draft_preference_ability_weight: 0.5,
                draft_young_bias: 0.0,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                updated_at: Some(chrono::Utc::now().to_rfc3339()),
            };
            (t.id, profile)
        })
        .collect();

    // 加载选手策略
    let player_strategies = {
        let engine_guard = MARKET_ENGINE.read().await;
        engine_guard.as_ref().unwrap().player_strategies.clone()
    };

    // 加载荣誉和表现数据（简化：暂时为空，后续可补充）
    let player_honors: HashMap<u64, PlayerHonorInfo> = HashMap::new();
    let player_performances: HashMap<u64, PlayerPerformanceInfo> = HashMap::new();

    log::info!("数据加载完成，耗时 {:.2}ms", start_time.elapsed().as_millis());

    // ========== 步骤2：并发球探筛选（规则引擎，<100ms）==========

    let scouting_start = std::time::Instant::now();

    let scouting_reports: Vec<crate::engines::scouting::ScoutingReport> = teams.iter()
        .map(|team| {
            let roster: Vec<Player> = players.iter()
                .filter(|p| p.team_id == Some(team.id))
                .cloned()
                .collect();

            let gm_profile = gm_profiles.get(&team.id).unwrap();

            // 计算位置需求
            let mut position_needs = HashMap::new();
            for pos in &["TOP", "JUG", "MID", "ADC", "SUP"] {
                let count = roster.iter()
                    .filter(|p| p.position.map(|pp| format!("{:?}", pp).to_uppercase() == *pos).unwrap_or(false))
                    .count();
                let need = match count {
                    0 => 100,
                    1 => 60,
                    2 => 30,
                    _ => 10,
                };
                position_needs.insert(pos.to_string(), need);
            }

            // 计算转会预算（简化）
            let budget_ratio = if team.balance > 5000000 { 0.6 }      // Rich
                              else if team.balance > 1000000 { 0.5 }  // Normal
                              else { 0.3 };                           // Poor
            let total_budget = (team.balance as f64 * budget_ratio / 10000.0) as u64;  // 转为万

            scout_transfer_candidates(
                team,
                &roster,
                gm_profile,
                total_budget,
                &available_players,
                &position_needs,
                &player_honors,
                &player_performances,
                is_poaching,
            )
        })
        .collect();

    log::info!("✅ 球探筛选完成，耗时 {:.2}ms", scouting_start.elapsed().as_millis());
    log::info!("共筛选出 {} 个候选", scouting_reports.iter().map(|r| r.candidates.len()).sum::<usize>());

    // ========== 步骤3：报价决策（规则引擎，瞬间）==========

    let mut new_offers = 0u32;

    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        let engine = match engine_guard.as_mut() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场引擎未初始化")),
        };

        for report in &scouting_reports {
            let team = teams_map.get(&report.team_id).unwrap();
            let gm_profile = gm_profiles.get(&report.team_id).unwrap();

            // 计算预算
            let budget_ratio = if team.balance > 5000000 { 0.6 } else if team.balance > 1000000 { 0.5 } else { 0.3 };
            let total_budget = (team.balance as f64 * budget_ratio / 10000.0) as u64;
            let transfer_budget = total_budget * 7 / 10;

            if let Some(decision) = decide_offer(
                team,
                &report.candidates,
                gm_profile,
                transfer_budget,
                total_budget,
                &player_strategies,
                is_poaching,
            ) {
                // 创建报价
                let player = players_map.get(&decision.player_id).unwrap();
                let neg_id = engine.find_or_create_negotiation(player);

                // 挖角阶段：设置转会费
                if is_poaching {
                    if let Some(neg) = engine.negotiations.get_mut(&neg_id) {
                        neg.is_transfer = true;
                        neg.transfer_fee = Some(decision.transfer_fee);
                    }
                }

                // 创建 Offer
                let offer_id = engine.next_offer_id;
                engine.next_offer_id += 1;

                let mut offer = Offer::new(
                    neg_id,
                    team.id,
                    team.name.clone(),
                    player.id,
                    current_round - 1,
                );
                offer.id = offer_id;
                offer.salary_offer = decision.salary_offer;
                offer.contract_years = decision.contract_years;
                offer.guarantee_starter = decision.guarantee_starter;
                offer.transfer_fee = decision.transfer_fee;
                offer.offer_reasoning = decision.reasoning;

                if let Some(neg) = engine.negotiations.get_mut(&neg_id) {
                    neg.add_offer(offer);
                    new_offers += 1;
                }

                log::info!(
                    "✓ {} 向 {} 发出报价: {}万/年{}",
                    team.name, player.game_id, decision.salary_offer,
                    if is_poaching { format!("，转会费{}万", decision.transfer_fee) } else { String::new() }
                );
            } else {
                log::info!("○ {} 本轮无合适目标", team.name);
            }
        }
    }

    log::info!("报价决策完成，新增 {} 个报价", new_offers);

    // ========== 步骤4：选手决策（规则引擎，瞬间）==========

    let mut new_signings = 0u32;

    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        let engine = match engine_guard.as_mut() {
            Some(e) => e,
            None => return Ok(ApiResponse::error("转会市场引擎未初始化")),
        };

        // 收集所有有报价的选手
        let mut players_with_offers: HashMap<u64, Vec<Offer>> = HashMap::new();

        for neg in engine.negotiations.values() {
            if neg.status == NegotiationStatus::Open && !neg.offers.is_empty() {
                players_with_offers.insert(neg.player_id, neg.offers.clone());
            }
        }

        log::info!("{} 名选手收到报价", players_with_offers.len());

        // 每个选手评估报价并选择
        for (player_id, offers) in &players_with_offers {
            let player = match players_map.get(player_id) {
                Some(p) => p,
                None => continue,
            };

            let player_strategy = match player_strategies.get(player_id) {
                Some(s) => s,
                None => continue,
            };

            if let Some(choice) = evaluate_offers_and_choose(
                *player_id,
                player.game_id.clone(),
                player_strategy,
                offers,
            ) {
                // 接受报价（手动设置状态）
                if let Some(neg_id) = engine.negotiations.iter()
                    .find(|(_, n)| n.player_id == *player_id)
                    .map(|(id, _)| *id) {

                    if let Some(neg) = engine.negotiations.get_mut(&neg_id) {
                        neg.status = NegotiationStatus::Accepted;
                        neg.final_team_id = Some(choice.chosen_team_id);
                        neg.final_team_name = Some(choice.chosen_team_name.clone());

                        // 找到被接受的报价
                        if let Some(accepted_offer) = neg.offers.iter_mut().find(|o| o.id == choice.chosen_offer_id) {
                            accepted_offer.status = OfferStatus::Accepted;
                            neg.final_salary = Some(accepted_offer.salary_offer);
                            neg.final_years = Some(accepted_offer.contract_years);
                            neg.final_starter = Some(accepted_offer.guarantee_starter);
                        }

                        new_signings += 1;
                    }
                }

                log::info!("✓ {} 选择加盟 {}", player.game_id, choice.chosen_team_name);
            }
        }
    }

    log::info!("选手决策完成，新增 {} 个签约", new_signings);

    // ========== 步骤5：更新引擎状态 ==========

    {
        let mut engine_guard = MARKET_ENGINE.write().await;
        if let Some(engine) = engine_guard.as_mut() {
            // 推进轮次
            engine.state.advance_round();

            // 从可用列表中移除已签约选手
            let signed_player_ids: Vec<u64> = engine.negotiations.values()
                .filter(|n| n.status == NegotiationStatus::Accepted)
                .map(|n| n.player_id)
                .collect();

            if is_poaching {
                engine.state.poachable_player_ids.retain(|id| !signed_player_ids.contains(id));
            } else {
                engine.state.free_agent_ids.retain(|id| !signed_player_ids.contains(id));
            }
        }
    }

    // ========== 步骤6：保存状态到数据库 ==========

    {
        let engine_guard = MARKET_ENGINE.read().await;
        if let Some(engine) = engine_guard.as_ref() {
            save_market_state_to_db(&pool, &engine.state).await?;
            save_all_negotiations_to_db(&pool, &engine.negotiations).await?;
        }
    }

    let total_elapsed = start_time.elapsed();

    log::info!("========== ✅ 规则引擎执行完成 ==========");
    log::info!("总耗时: {:.2}ms", total_elapsed.as_millis());
    log::info!("新增报价: {}", new_offers);
    log::info!("新增签约: {}", new_signings);

    let result = RoundExecutionResult {
        phase: current_phase,
        round: (current_round - 1) as u8,
        phase_changed: false,
        new_phase: None,
        events: Vec::new(),
        new_negotiations: new_offers as usize,
        completed_signings: new_signings as usize,
        summary: format!(
            "规则引擎：第{}轮，新增{}个报价，{}个签约，耗时{:.2}ms",
            current_round, new_offers, new_signings, total_elapsed.as_millis()
        ),
    };

    Ok(ApiResponse::success(result))
}
