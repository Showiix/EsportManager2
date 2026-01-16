/**
 * 年度积分系统命令
 */
use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{PointsRepository, SaveRepository, TeamAnnualPoints};
use crate::models::AnnualPointsDetail;
use tauri::State;
use serde::{Deserialize, Serialize};

/// 积分明细（带赛事名称）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsDetailWithTournament {
    pub id: u64,
    pub tournament_id: u64,
    pub tournament_name: String,
    pub points: u32,
    pub final_rank: Option<u32>,
}

/// 队伍积分详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPointsDetail {
    pub team_id: u64,
    pub team_name: String,
    pub region_code: String,
    pub total_points: u32,
    pub rank: u32,
    pub details: Vec<PointsDetailWithTournament>,
}

/// 获取年度积分排名
#[tauri::command]
pub async fn get_annual_points_ranking(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<TeamAnnualPoints>>, String> {
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
    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    match PointsRepository::get_season_rankings(&pool, &save_id, save.current_season as u64).await {
        Ok(rankings) => Ok(CommandResult::ok(rankings)),
        Err(e) => Ok(CommandResult::err(format!("Failed to get rankings: {}", e))),
    }
}

/// 获取队伍的年度积分明细
#[tauri::command]
pub async fn get_team_points_detail(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<CommandResult<Vec<AnnualPointsDetail>>, String> {
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
    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    match PointsRepository::get_team_season_points(&pool, &save_id, save.current_season as u64, team_id).await {
        Ok(details) => Ok(CommandResult::ok(details)),
        Err(e) => Ok(CommandResult::err(format!("Failed to get team points: {}", e))),
    }
}

/// 获取赛事的积分发放记录
#[tauri::command]
pub async fn get_tournament_points(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<AnnualPointsDetail>>, String> {
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

    match PointsRepository::get_tournament_points(&pool, &save_id, tournament_id).await {
        Ok(details) => Ok(CommandResult::ok(details)),
        Err(e) => Ok(CommandResult::err(format!("Failed to get tournament points: {}", e))),
    }
}

/// 获取Super资格队伍（年度积分Top16）
#[tauri::command]
pub async fn get_super_qualified_teams(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<TeamAnnualPoints>>, String> {
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
    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get save: {}", e))),
    };

    match PointsRepository::get_season_rankings(&pool, &save_id, save.current_season as u64).await {
        Ok(rankings) => {
            // 返回Top16
            let top16: Vec<TeamAnnualPoints> = rankings.into_iter().take(16).collect();
            Ok(CommandResult::ok(top16))
        }
        Err(e) => Ok(CommandResult::err(format!("Failed to get rankings: {}", e))),
    }
}
