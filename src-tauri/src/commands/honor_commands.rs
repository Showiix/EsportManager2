use crate::commands::{ApiResponse, AppState};
use crate::db::HonorRepository;
use crate::engines::HonorEngine;
use crate::models::Honor;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 荣誉殿堂数据响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HonorHallResponse {
    pub champions: Vec<Honor>,
    pub mvps: Vec<Honor>,
    pub champions_by_type: std::collections::HashMap<String, Vec<Honor>>,
}

/// 荣誉统计响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HonorCountResponse {
    pub champion_count: u32,
    pub mvp_count: u32,
}

/// 获取荣誉殿堂数据
#[tauri::command]
pub async fn get_honor_hall(
    state: State<'_, AppState>,
    save_id: String,
) -> Result<ApiResponse<HonorHallResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    let all_honors = match HonorRepository::get_all(&pool, &save_id).await {
        Ok(h) => h,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get honors: {}", e))),
    };

    let engine = HonorEngine::new();
    let hall_data = engine.build_honor_hall(all_honors);

    Ok(ApiResponse::success(HonorHallResponse {
        champions: hall_data.champions,
        mvps: hall_data.mvps,
        champions_by_type: hall_data.champions_by_type,
    }))
}

/// 获取战队所有荣誉
#[tauri::command]
pub async fn get_team_honors(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_by_team(&pool, &save_id, team_id).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get team honors: {}", e))),
    }
}

/// 获取选手所有荣誉
#[tauri::command]
pub async fn get_player_honors(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_by_player(&pool, &save_id, player_id).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get player honors: {}", e))),
    }
}

/// 获取赛季所有荣誉
#[tauri::command]
pub async fn get_season_honors(
    state: State<'_, AppState>,
    save_id: String,
    season_id: u64,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_by_season(&pool, &save_id, season_id).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get season honors: {}", e))),
    }
}

/// 获取赛事所有荣誉
#[tauri::command]
pub async fn get_tournament_honors(
    state: State<'_, AppState>,
    save_id: String,
    tournament_id: u64,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_by_tournament(&pool, &save_id, tournament_id).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get tournament honors: {}", e))),
    }
}

/// 获取战队冠军数量
#[tauri::command]
pub async fn get_team_champion_count(
    state: State<'_, AppState>,
    save_id: String,
    team_id: u64,
) -> Result<ApiResponse<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::count_team_champions(&pool, &save_id, team_id).await {
        Ok(count) => Ok(ApiResponse::success(count)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to count team champions: {}", e))),
    }
}

/// 获取选手冠军数量
#[tauri::command]
pub async fn get_player_champion_count(
    state: State<'_, AppState>,
    save_id: String,
    player_id: u64,
) -> Result<ApiResponse<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::count_player_champions(&pool, &save_id, player_id).await {
        Ok(count) => Ok(ApiResponse::success(count)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to count player champions: {}", e))),
    }
}

/// 获取选手MVP数量
#[tauri::command]
pub async fn get_player_mvp_count(
    state: State<'_, AppState>,
    save_id: String,
    player_id: u64,
) -> Result<ApiResponse<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::count_player_mvps(&pool, &save_id, player_id).await {
        Ok(count) => Ok(ApiResponse::success(count)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to count player MVPs: {}", e))),
    }
}

/// 获取战队荣誉统计
#[tauri::command]
pub async fn get_team_honor_stats(
    state: State<'_, AppState>,
    save_id: String,
    team_id: u64,
) -> Result<ApiResponse<HonorCountResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    let champion_count = match HonorRepository::count_team_champions(&pool, &save_id, team_id).await {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to count champions: {}", e))),
    };

    Ok(ApiResponse::success(HonorCountResponse {
        champion_count,
        mvp_count: 0,
    }))
}

/// 获取选手荣誉统计
#[tauri::command]
pub async fn get_player_honor_stats(
    state: State<'_, AppState>,
    save_id: String,
    player_id: u64,
) -> Result<ApiResponse<HonorCountResponse>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    let champion_count = match HonorRepository::count_player_champions(&pool, &save_id, player_id).await {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to count champions: {}", e))),
    };

    let mvp_count = match HonorRepository::count_player_mvps(&pool, &save_id, player_id).await {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to count MVPs: {}", e))),
    };

    Ok(ApiResponse::success(HonorCountResponse {
        champion_count,
        mvp_count,
    }))
}

/// 按赛事类型获取冠军列表
#[tauri::command]
pub async fn get_champions_by_type(
    state: State<'_, AppState>,
    save_id: String,
    tournament_type: String,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_champions_by_tournament_type(&pool, &save_id, &tournament_type).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get champions: {}", e))),
    }
}

/// 获取所有冠军记录
#[tauri::command]
pub async fn get_all_champions(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_champions(&pool, &save_id).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get champions: {}", e))),
    }
}

/// 获取所有MVP记录
#[tauri::command]
pub async fn get_all_mvps(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Honor>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(ApiResponse::error("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(ApiResponse::error("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Database error: {}", e))),
    };

    match HonorRepository::get_mvps(&pool, &save_id).await {
        Ok(honors) => Ok(ApiResponse::success(honors)),
        Err(e) => Ok(ApiResponse::error(&format!("Failed to get MVPs: {}", e))),
    }
}
