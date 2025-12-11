use crate::commands::{AppState, CommandResult};
use crate::db::{EventRepository, PlayerRepository, TeamRepository};
use crate::engines::EventEngine;
use crate::models::{
    ContractExpireDetail, EventType, GameEvent, PlayerDeclineDetail, PlayerGrowthDetail,
    PlayerRetirementDetail,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// 赛季结算结果响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonSettlementResponse {
    pub season_id: u64,
    pub season_name: String,
    pub summary: SettlementSummary,
    pub growth_events: Vec<PlayerGrowthDetail>,
    pub decline_events: Vec<PlayerDeclineDetail>,
    pub retirement_events: Vec<PlayerRetirementDetail>,
    pub contract_expire_events: Vec<ContractExpireDetail>,
}

/// 结算摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct SettlementSummary {
    pub total_players_processed: u32,
    pub players_grown: u32,
    pub players_declined: u32,
    pub players_retired: u32,
    pub contracts_renewed: u32,
    pub contracts_expired: u32,
}

/// 预览赛季结算（不实际应用）
#[tauri::command]
pub async fn preview_season_settlement(
    state: State<'_, AppState>,
    current_season: u32,
) -> Result<CommandResult<SeasonSettlementResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    // 获取所有活跃选手
    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("获取选手失败: {}", e))),
    };

    // 获取所有队伍
    let teams = match TeamRepository::get_all(&pool, &save_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("获取队伍失败: {}", e))),
    };

    // 使用事件引擎计算结算结果
    let engine = EventEngine::new();
    let result = engine.process_season_settlement(
        current_season as u64,
        &players,
        &teams,
        current_season,
    );

    let contracts_renewed = result
        .contract_expire_events
        .iter()
        .filter(|e| e.renewed)
        .count() as u32;
    let contracts_expired = result.contract_expire_events.len() as u32 - contracts_renewed;

    let response = SeasonSettlementResponse {
        season_id: result.season_id,
        season_name: result.season_name,
        summary: SettlementSummary {
            total_players_processed: players.len() as u32,
            players_grown: result.growth_events.len() as u32,
            players_declined: result.decline_events.len() as u32,
            players_retired: result.retirement_events.len() as u32,
            contracts_renewed,
            contracts_expired,
        },
        growth_events: result.growth_events,
        decline_events: result.decline_events,
        retirement_events: result.retirement_events,
        contract_expire_events: result.contract_expire_events,
    };

    Ok(CommandResult::ok(response))
}

/// 执行赛季结算（实际应用变更）
#[tauri::command]
pub async fn execute_season_settlement(
    state: State<'_, AppState>,
    current_season: u32,
) -> Result<CommandResult<SeasonSettlementResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    // 获取所有活跃选手
    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("获取选手失败: {}", e))),
    };

    // 获取所有队伍
    let teams = match TeamRepository::get_all(&pool, &save_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("获取队伍失败: {}", e))),
    };

    // 使用事件引擎处理
    let engine = EventEngine::new();
    let batch_result = engine.batch_process_season_end(&players, &teams, current_season);

    // 应用年龄更新
    if let Err(e) = PlayerRepository::batch_update_age(&pool, &batch_result.age_updates).await {
        return Ok(CommandResult::err(format!("更新年龄失败: {}", e)));
    }

    // 应用退役
    if let Err(e) =
        PlayerRepository::batch_retire(&pool, &batch_result.retirements, current_season).await
    {
        return Ok(CommandResult::err(format!("处理退役失败: {}", e)));
    }

    // 应用合同更新
    if let Err(e) =
        PlayerRepository::batch_update_contracts(&pool, &batch_result.contract_updates, current_season)
            .await
    {
        return Ok(CommandResult::err(format!("更新合同失败: {}", e)));
    }

    // 应用能力值更新
    if let Err(e) = PlayerRepository::batch_update_ability(&pool, &batch_result.ability_updates).await
    {
        return Ok(CommandResult::err(format!("更新能力值失败: {}", e)));
    }

    // 重新计算结算结果用于返回
    let result = engine.process_season_settlement(
        current_season as u64,
        &players,
        &teams,
        current_season,
    );

    // 创建事件记录
    let mut events = Vec::new();

    // 成长事件
    for growth in &result.growth_events {
        events.push(GameEvent {
            id: 0,
            save_id: save_id.clone(),
            season_id: current_season as u64,
            event_type: EventType::PlayerGrowth,
            player_id: Some(growth.player_id),
            team_id: None,
            description: format!(
                "{} 能力成长: {} → {} (+{})",
                growth.player_name, growth.old_ability, growth.new_ability, growth.growth_amount
            ),
            details: serde_json::to_string(growth).ok(),
            phase: Some("SeasonEnd".to_string()),
        });
    }

    // 衰退事件
    for decline in &result.decline_events {
        events.push(GameEvent {
            id: 0,
            save_id: save_id.clone(),
            season_id: current_season as u64,
            event_type: EventType::PlayerDecline,
            player_id: Some(decline.player_id),
            team_id: None,
            description: format!(
                "{} 能力衰退: {} → {} (-{})",
                decline.player_name, decline.old_ability, decline.new_ability, decline.decline_amount
            ),
            details: serde_json::to_string(decline).ok(),
            phase: Some("SeasonEnd".to_string()),
        });
    }

    // 退役事件
    for retirement in &result.retirement_events {
        events.push(GameEvent {
            id: 0,
            save_id: save_id.clone(),
            season_id: current_season as u64,
            event_type: EventType::PlayerRetirement,
            player_id: Some(retirement.player_id),
            team_id: retirement.team_id,
            description: format!(
                "{} 宣布退役，{}岁，最终能力 {}，职业生涯 {} 个赛季",
                retirement.player_name,
                retirement.age,
                retirement.final_ability,
                retirement.career_seasons
            ),
            details: serde_json::to_string(retirement).ok(),
            phase: Some("SeasonEnd".to_string()),
        });
    }

    // 合同到期事件
    for contract in &result.contract_expire_events {
        let description = if contract.renewed {
            format!(
                "{} 与 {} 续约，{}年合同，年薪 {} 万",
                contract.player_name,
                contract.team_name,
                contract.new_contract_years.unwrap_or(0),
                contract.new_salary.unwrap_or(0)
            )
        } else {
            format!(
                "{} 合同到期离开 {}，成为自由球员",
                contract.player_name, contract.team_name
            )
        };

        events.push(GameEvent {
            id: 0,
            save_id: save_id.clone(),
            season_id: current_season as u64,
            event_type: EventType::ContractExpire,
            player_id: Some(contract.player_id),
            team_id: Some(contract.team_id),
            description,
            details: serde_json::to_string(contract).ok(),
            phase: Some("SeasonEnd".to_string()),
        });
    }

    // 保存事件记录
    if let Err(e) = EventRepository::create_batch(&pool, &events).await {
        log::warn!("保存事件记录失败: {}", e);
    }

    let contracts_renewed = result
        .contract_expire_events
        .iter()
        .filter(|e| e.renewed)
        .count() as u32;
    let contracts_expired = result.contract_expire_events.len() as u32 - contracts_renewed;

    let response = SeasonSettlementResponse {
        season_id: result.season_id,
        season_name: result.season_name,
        summary: SettlementSummary {
            total_players_processed: players.len() as u32,
            players_grown: result.growth_events.len() as u32,
            players_declined: result.decline_events.len() as u32,
            players_retired: result.retirement_events.len() as u32,
            contracts_renewed,
            contracts_expired,
        },
        growth_events: result.growth_events,
        decline_events: result.decline_events,
        retirement_events: result.retirement_events,
        contract_expire_events: result.contract_expire_events,
    };

    Ok(CommandResult::ok(response))
}

/// 获取赛季事件列表
#[tauri::command]
pub async fn get_season_events(
    state: State<'_, AppState>,
    season_id: u64,
) -> Result<CommandResult<Vec<GameEvent>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    match EventRepository::get_by_season(&pool, &save_id, season_id).await {
        Ok(events) => Ok(CommandResult::ok(events)),
        Err(e) => Ok(CommandResult::err(format!("获取事件失败: {}", e))),
    }
}

/// 获取选手相关事件
#[tauri::command]
pub async fn get_player_events(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<CommandResult<Vec<GameEvent>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    match EventRepository::get_by_player(&pool, &save_id, player_id).await {
        Ok(events) => Ok(CommandResult::ok(events)),
        Err(e) => Ok(CommandResult::err(format!("获取事件失败: {}", e))),
    }
}

/// 获取特定类型事件
#[tauri::command]
pub async fn get_events_by_type(
    state: State<'_, AppState>,
    event_type: String,
) -> Result<CommandResult<Vec<GameEvent>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    match EventRepository::get_by_type(&pool, &save_id, &event_type).await {
        Ok(events) => Ok(CommandResult::ok(events)),
        Err(e) => Ok(CommandResult::err(format!("获取事件失败: {}", e))),
    }
}

/// 单独更新选手年龄
#[tauri::command]
pub async fn update_players_age(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<PlayerAgeUpdateResult>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    // 获取所有活跃选手
    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("获取选手失败: {}", e))),
    };

    let engine = EventEngine::new();
    let mut results = Vec::new();
    let mut updates = Vec::new();

    for player in &players {
        let aging = engine.update_player_age(player);
        updates.push((player.id, aging.new_age, aging.new_stability));
        results.push(PlayerAgeUpdateResult {
            player_id: player.id,
            player_name: player.game_id.clone(),
            old_age: aging.old_age,
            new_age: aging.new_age,
            old_stability: aging.old_stability,
            new_stability: aging.new_stability,
        });
    }

    if let Err(e) = PlayerRepository::batch_update_age(&pool, &updates).await {
        return Ok(CommandResult::err(format!("更新年龄失败: {}", e)));
    }

    Ok(CommandResult::ok(results))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerAgeUpdateResult {
    pub player_id: u64,
    pub player_name: String,
    pub old_age: u8,
    pub new_age: u8,
    pub old_stability: u8,
    pub new_stability: u8,
}

/// 获取即将退役的选手列表
#[tauri::command]
pub async fn get_retiring_candidates(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<RetiringCandidate>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("获取选手失败: {}", e))),
    };

    let engine = EventEngine::new();
    let mut candidates = Vec::new();

    for player in &players {
        if let Some(reason) = engine.should_retire(player) {
            candidates.push(RetiringCandidate {
                player_id: player.id,
                player_name: player.game_id.clone(),
                team_id: player.team_id,
                age: player.age,
                ability: player.ability,
                reason: format!("{:?}", reason),
                reason_description: reason.description().to_string(),
            });
        }
    }

    Ok(CommandResult::ok(candidates))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetiringCandidate {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: Option<u64>,
    pub age: u8,
    pub ability: u8,
    pub reason: String,
    pub reason_description: String,
}

/// 获取合同即将到期的选手
#[tauri::command]
pub async fn get_expiring_contracts(
    state: State<'_, AppState>,
    current_season: u32,
) -> Result<CommandResult<Vec<ExpiringContract>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("数据库未初始化")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("数据库连接失败: {}", e))),
    };

    let save_id_guard = state.current_save_id.read().await;
    let save_id = match save_id_guard.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("未选择存档")),
    };

    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("获取选手失败: {}", e))),
    };

    let engine = EventEngine::new();
    let mut expiring = Vec::new();

    for player in &players {
        if engine.is_contract_expired(&player, current_season) {
            expiring.push(ExpiringContract {
                player_id: player.id,
                player_name: player.game_id.clone(),
                team_id: player.team_id,
                age: player.age,
                ability: player.ability,
                contract_end_season: player.contract_end_season,
                salary: player.salary,
            });
        }
    }

    Ok(CommandResult::ok(expiring))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpiringContract {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: Option<u64>,
    pub age: u8,
    pub ability: u8,
    pub contract_end_season: Option<u32>,
    pub salary: u64,
}
