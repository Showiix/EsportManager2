//! 转会系统命令
//!
//! 实现前端调用的转会相关命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::engines::TransferEngine;
use crate::models::transfer::*;
use serde::Deserialize;
use sqlx::Row;
use tauri::State;

// ============================================
// 响应类型
// ============================================

/// AI性格更新请求
#[derive(Debug, Deserialize)]
pub struct UpdatePersonalityRequest {
    pub personality: String,
    pub short_term_focus: Option<f64>,
    pub long_term_focus: Option<f64>,
    pub risk_tolerance: Option<f64>,
    pub youth_preference: Option<f64>,
    pub star_chasing: Option<f64>,
    pub bargain_hunting: Option<f64>,
}

// ============================================
// 转会期管理命令
// ============================================

/// 开始转会期
#[tauri::command]
pub async fn start_transfer_window(
    state: State<'_, AppState>,
) -> Result<CommandResult<TransferWindowResponse>, String> {
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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    // 检查是否已有进行中的转会期
    let existing = sqlx::query(
        "SELECT id FROM transfer_windows WHERE save_id = ? AND season_id = ? AND status = 'IN_PROGRESS'"
    )
    .bind(&save_id)
    .bind(current_season)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(CommandResult::err("本赛季已有进行中的转会期"));
    }

    let engine = TransferEngine::new();
    let response = engine.start_transfer_window(&pool, &save_id, current_season).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(response))
}

/// 执行单轮转会
#[tauri::command]
pub async fn execute_transfer_round(
    state: State<'_, AppState>,
    window_id: i64,
    round: i64,
) -> Result<CommandResult<RoundExecutionResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    if round < 1 || round > 8 {
        return Ok(CommandResult::err("轮次必须在1-8之间"));
    }

    let engine = TransferEngine::new();
    let result = engine.execute_round(&pool, window_id, round).await
        .map_err(|e| e.to_string())?;

    let event_count = result.events.len() as i64;
    let next_round = if round < 8 { Some(round + 1) } else { None };

    Ok(CommandResult::ok(RoundExecutionResponse {
        round: result.round,
        round_name: result.round_name,
        events: result.events,
        event_count,
        next_round,
        summary: result.summary,
    }))
}

/// 快进转会期
#[tauri::command]
pub async fn fast_forward_transfer(
    state: State<'_, AppState>,
    window_id: i64,
    from_round: Option<i64>,
) -> Result<CommandResult<FastForwardResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let start_round = from_round.unwrap_or(1);
    if start_round < 1 || start_round > 8 {
        return Ok(CommandResult::err("起始轮次必须在1-8之间"));
    }

    let engine = TransferEngine::new();
    let response = engine.fast_forward(&pool, window_id, start_round).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(response))
}

// ============================================
// 转会事件查询命令
// ============================================

/// 获取转会事件
#[tauri::command]
pub async fn get_transfer_events(
    state: State<'_, AppState>,
    window_id: i64,
    round: Option<i64>,
    level: Option<String>,
) -> Result<CommandResult<Vec<TransferEvent>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let engine = TransferEngine::new();
    let events = engine.get_events(&pool, window_id, round, level.as_deref()).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(events))
}

/// 获取转会报告
#[tauri::command]
pub async fn get_transfer_report(
    state: State<'_, AppState>,
    window_id: i64,
) -> Result<CommandResult<TransferReport>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let engine = TransferEngine::new();
    let report = engine.generate_transfer_report(&pool, window_id).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(report))
}

// ============================================
// 转会期状态命令
// ============================================

/// 获取转会期状态
#[tauri::command]
pub async fn get_transfer_window_status(
    state: State<'_, AppState>,
    window_id: i64,
) -> Result<CommandResult<TransferWindowResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let row = sqlx::query(
        "SELECT id, season_id, status, current_round FROM transfer_windows WHERE id = ?"
    )
    .bind(window_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            Ok(CommandResult::ok(TransferWindowResponse {
                window_id: r.get("id"),
                season_id: r.get("season_id"),
                status: r.get("status"),
                current_round: r.get("current_round"),
            }))
        }
        None => Ok(CommandResult::err("转会期不存在")),
    }
}

// ============================================
// AI球队性格命令
// ============================================

/// 获取球队AI性格
#[tauri::command]
pub async fn get_team_personality(
    state: State<'_, AppState>,
    team_id: i64,
) -> Result<CommandResult<Option<TeamPersonalityConfig>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let row = sqlx::query(
        "SELECT * FROM team_personality_configs WHERE team_id = ?"
    )
    .bind(team_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    match row {
        Some(r) => {
            Ok(CommandResult::ok(Some(TeamPersonalityConfig {
                id: r.get("id"),
                team_id: r.get("team_id"),
                save_id: r.get("save_id"),
                personality: r.get("personality"),
                short_term_focus: r.get("short_term_focus"),
                long_term_focus: r.get("long_term_focus"),
                risk_tolerance: r.get("risk_tolerance"),
                youth_preference: r.get("youth_preference"),
                star_chasing: r.get("star_chasing"),
                bargain_hunting: r.get("bargain_hunting"),
                updated_at: r.get("updated_at"),
            })))
        }
        None => Ok(CommandResult::ok(None)),
    }
}

/// 更新球队AI性格
#[tauri::command]
pub async fn update_team_personality(
    state: State<'_, AppState>,
    team_id: i64,
    request: UpdatePersonalityRequest,
) -> Result<CommandResult<bool>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取当前存档 ID
    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };

    // 获取默认权重
    let personality = AITeamPersonality::from_str(&request.personality);
    let defaults = personality.default_weights();

    let short_term = request.short_term_focus.unwrap_or(defaults.short_term_focus);
    let long_term = request.long_term_focus.unwrap_or(defaults.long_term_focus);
    let risk = request.risk_tolerance.unwrap_or(defaults.risk_tolerance);
    let youth = request.youth_preference.unwrap_or(defaults.youth_preference);
    let star = request.star_chasing.unwrap_or(defaults.star_chasing);
    let bargain = request.bargain_hunting.unwrap_or(defaults.bargain_hunting);

    // 使用 INSERT OR REPLACE (UPSERT)
    sqlx::query(
        r#"INSERT INTO team_personality_configs
           (team_id, save_id, personality, short_term_focus, long_term_focus,
            risk_tolerance, youth_preference, star_chasing, bargain_hunting, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
           ON CONFLICT(team_id) DO UPDATE SET
               personality = excluded.personality,
               short_term_focus = excluded.short_term_focus,
               long_term_focus = excluded.long_term_focus,
               risk_tolerance = excluded.risk_tolerance,
               youth_preference = excluded.youth_preference,
               star_chasing = excluded.star_chasing,
               bargain_hunting = excluded.bargain_hunting,
               updated_at = datetime('now')"#
    )
    .bind(team_id)
    .bind(&save_id)
    .bind(personality.as_str())
    .bind(short_term)
    .bind(long_term)
    .bind(risk)
    .bind(youth)
    .bind(star)
    .bind(bargain)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(true))
}

// ============================================
// 声望系统命令
// ============================================

/// 获取球队声望
#[tauri::command]
pub async fn get_team_reputation(
    state: State<'_, AppState>,
    team_id: i64,
) -> Result<CommandResult<TeamReputation>, String> {
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

    // 获取当前赛季
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let engine = TransferEngine::new();
    let reputation = engine.calculate_team_reputation(&pool, team_id, &save_id, current_season).await
        .map_err(|e| e.to_string())?;

    Ok(CommandResult::ok(reputation))
}

// ============================================
// 选手合同中心命令
// ============================================

/// 获取选手合同列表（合同中心）
#[tauri::command]
pub async fn get_player_market_list(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<PlayerContractInfo>>, String> {
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

    // 查询所有选手的合同信息
    let rows = sqlx::query(
        r#"
        SELECT
            p.id as player_id,
            p.game_id as player_name,
            p.position,
            p.age,
            p.ability,
            p.potential,
            p.team_id,
            t.name as team_name,
            r.short_name as region_code,
            p.salary,
            p.contract_end_season,
            p.join_season,
            p.market_value as base_market_value,
            p.calculated_market_value,
            p.satisfaction,
            p.loyalty,
            p.is_starter,
            p.status
        FROM players p
        LEFT JOIN teams t ON p.team_id = t.id
        LEFT JOIN regions r ON t.region_id = r.id
        WHERE p.save_id = ? AND p.status != 'RETIRED'
        ORDER BY p.ability DESC, p.potential DESC
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let players: Vec<PlayerContractInfo> = rows
        .iter()
        .map(|row| PlayerContractInfo {
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            position: row.get("position"),
            age: row.get("age"),
            ability: row.get("ability"),
            potential: row.get("potential"),
            team_id: row.get("team_id"),
            team_name: row.get("team_name"),
            region_code: row.get("region_code"),
            salary: row.get("salary"),
            contract_end_season: row.get("contract_end_season"),
            join_season: row.get("join_season"),
            base_market_value: row.get("base_market_value"),
            calculated_market_value: row.get("calculated_market_value"),
            satisfaction: row.get("satisfaction"),
            loyalty: row.get("loyalty"),
            is_starter: row.get("is_starter"),
            status: row.get("status"),
        })
        .collect();

    Ok(CommandResult::ok(players))
}
