use serde::{Deserialize, Serialize};
use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::repository::PlayerStatsRepository;
use crate::models::{PlayerSeasonStatistics, PlayerRankingItem};

/// 命令结果包装
#[derive(Debug, Serialize, Deserialize)]
pub struct StatsCommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> StatsCommandResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// 记录选手表现请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPerformanceParams {
    pub player_id: i64,
    pub player_name: String,
    pub team_id: i64,
    pub position: String,
    pub impact_score: f64,
    pub actual_ability: f64,
    pub season_id: i64,
    pub region_id: Option<String>,
}

/// 记录选手表现
#[tauri::command]
pub async fn record_player_performance(
    state: State<'_, AppState>,
    params: RecordPerformanceParams,
) -> Result<StatsCommandResult<PlayerSeasonStatistics>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取或创建选手统计记录
    let mut stats = match PlayerStatsRepository::get_or_create(
        &pool,
        &save_id,
        params.player_id,
        &params.player_name,
        params.season_id,
        Some(params.team_id),
        params.region_id.as_deref(),
        &params.position,
    ).await {
        Ok(s) => s,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get/create stats: {}", e))),
    };

    // 记录表现
    stats.record_game(params.impact_score, params.actual_ability);

    // 更新数据库
    if let Err(e) = PlayerStatsRepository::update(&pool, &stats).await {
        return Ok(StatsCommandResult::err(format!("Failed to update stats: {}", e)));
    }

    Ok(StatsCommandResult::ok(stats))
}

/// 批量记录选手表现
#[tauri::command]
pub async fn batch_record_player_performance(
    state: State<'_, AppState>,
    performances: Vec<RecordPerformanceParams>,
) -> Result<StatsCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let mut count = 0;
    for params in performances {
        // 获取或创建选手统计记录
        let mut stats = match PlayerStatsRepository::get_or_create(
            &pool,
            &save_id,
            params.player_id,
            &params.player_name,
            params.season_id,
            Some(params.team_id),
            params.region_id.as_deref(),
            &params.position,
        ).await {
            Ok(s) => s,
            Err(_) => continue,
        };

        // 记录表现
        stats.record_game(params.impact_score, params.actual_ability);

        // 更新数据库
        if PlayerStatsRepository::update(&pool, &stats).await.is_ok() {
            count += 1;
        }
    }

    Ok(StatsCommandResult::ok(count))
}

/// 记录冠军荣誉
#[tauri::command]
pub async fn record_championship(
    state: State<'_, AppState>,
    team_id: i64,
    is_international: bool,
    season_id: i64,
) -> Result<StatsCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取队伍所有选手的统计
    let stats_list = match PlayerStatsRepository::get_by_team(&pool, &save_id, season_id, team_id).await {
        Ok(list) => list,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get team stats: {}", e))),
    };

    let mut updated_count = 0;
    for mut stats in stats_list {
        stats.record_championship(is_international);
        if PlayerStatsRepository::update(&pool, &stats).await.is_ok() {
            updated_count += 1;
        }
    }

    Ok(StatsCommandResult::ok(updated_count))
}

/// 获取赛季排行榜
#[tauri::command]
pub async fn get_season_impact_ranking(
    state: State<'_, AppState>,
    season_id: i64,
    limit: Option<i32>,
) -> Result<StatsCommandResult<Vec<PlayerRankingItem>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let limit = limit.unwrap_or(20);
    let stats_list = match PlayerStatsRepository::get_season_ranking(&pool, &save_id, season_id, limit).await {
        Ok(list) => list,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get ranking: {}", e))),
    };

    let ranking: Vec<PlayerRankingItem> = stats_list.into_iter().map(|s| s.into()).collect();
    Ok(StatsCommandResult::ok(ranking))
}

/// 获取分位置排行榜
#[tauri::command]
pub async fn get_position_ranking(
    state: State<'_, AppState>,
    season_id: i64,
    position: String,
    limit: Option<i32>,
) -> Result<StatsCommandResult<Vec<PlayerRankingItem>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let limit = limit.unwrap_or(10);
    let stats_list = match PlayerStatsRepository::get_position_ranking(&pool, &save_id, season_id, &position, limit).await {
        Ok(list) => list,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get ranking: {}", e))),
    };

    let ranking: Vec<PlayerRankingItem> = stats_list.into_iter().map(|s| s.into()).collect();
    Ok(StatsCommandResult::ok(ranking))
}

/// 获取选手统计数据
#[tauri::command]
pub async fn get_player_stats(
    state: State<'_, AppState>,
    player_id: i64,
    season_id: Option<i64>,
) -> Result<StatsCommandResult<Vec<PlayerSeasonStatistics>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let stats_list = match PlayerStatsRepository::get_by_player(&pool, &save_id, player_id, season_id).await {
        Ok(list) => list,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get stats: {}", e))),
    };

    Ok(StatsCommandResult::ok(stats_list))
}

/// 获取队伍选手统计
#[tauri::command]
pub async fn get_team_player_stats(
    state: State<'_, AppState>,
    team_id: i64,
    season_id: i64,
) -> Result<StatsCommandResult<Vec<PlayerSeasonStatistics>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let stats_list = match PlayerStatsRepository::get_by_team(&pool, &save_id, season_id, team_id).await {
        Ok(list) => list,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get stats: {}", e))),
    };

    Ok(StatsCommandResult::ok(stats_list))
}

/// 清除赛季统计数据
#[tauri::command]
pub async fn clear_season_stats(
    state: State<'_, AppState>,
    season_id: i64,
) -> Result<StatsCommandResult<bool>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(StatsCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    if let Err(e) = PlayerStatsRepository::clear_season(&pool, &save_id, season_id).await {
        return Ok(StatsCommandResult::err(format!("Failed to clear stats: {}", e)));
    }

    Ok(StatsCommandResult::ok(true))
}
