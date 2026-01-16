use serde::{Deserialize, Serialize};
use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::repository::PlayerStatsRepository;
use crate::db::{MatchGameDetailRepository, PlayerTournamentStatsRepository};
use crate::models::{PlayerSeasonStatistics, PlayerRankingItem};
use crate::models::tournament_result::PlayerTournamentStats;

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
#[tauri::command(rename_all = "camelCase")]
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
#[tauri::command(rename_all = "camelCase")]
pub async fn get_season_impact_ranking(
    state: State<'_, AppState>,
    season_id: i64,
    limit: Option<i32>,
) -> Result<StatsCommandResult<Vec<PlayerRankingItem>>, String> {
    println!("[get_season_impact_ranking] Called with season_id={}, limit={:?}", season_id, limit);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => {
            println!("[get_season_impact_ranking] ERROR: Database not initialized");
            return Ok(StatsCommandResult::err("Database not initialized"));
        }
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => {
            println!("[get_season_impact_ranking] save_id={}", id);
            id.clone()
        },
        None => {
            println!("[get_season_impact_ranking] ERROR: No save loaded");
            return Ok(StatsCommandResult::err("No save loaded"));
        }
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let limit = limit.unwrap_or(20);
    println!("[get_season_impact_ranking] Querying with save_id={}, season_id={}, limit={}", save_id, season_id, limit);

    let stats_list = match PlayerStatsRepository::get_season_ranking(&pool, &save_id, season_id, limit).await {
        Ok(list) => {
            println!("[get_season_impact_ranking] Got {} records from database", list.len());
            list
        },
        Err(e) => {
            println!("[get_season_impact_ranking] ERROR: {}", e);
            return Ok(StatsCommandResult::err(format!("Failed to get ranking: {}", e)));
        }
    };

    let ranking: Vec<PlayerRankingItem> = stats_list.into_iter().map(|s| s.into()).collect();
    Ok(StatsCommandResult::ok(ranking))
}

/// 获取分位置排行榜
#[tauri::command(rename_all = "camelCase")]
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
#[tauri::command(rename_all = "camelCase")]
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
#[tauri::command(rename_all = "camelCase")]
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
#[tauri::command(rename_all = "camelCase")]
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

/// 获取选手影响力历史数据
#[tauri::command(rename_all = "camelCase")]
pub async fn get_player_impact_history(
    state: State<'_, AppState>,
    player_id: i64,
    season_id: Option<i64>,
) -> Result<StatsCommandResult<Vec<f64>>, String> {
    println!("[get_player_impact_history] Called with player_id={}, season_id={:?}", player_id, season_id);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => {
            println!("[get_player_impact_history] ERROR: Database not initialized");
            return Ok(StatsCommandResult::err("Database not initialized"));
        }
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => {
            println!("[get_player_impact_history] save_id={}", id);
            id.clone()
        },
        None => {
            println!("[get_player_impact_history] ERROR: No save loaded");
            return Ok(StatsCommandResult::err("No save loaded"));
        }
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => {
            println!("[get_player_impact_history] ERROR: Failed to get pool: {}", e);
            return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e)));
        }
    };

    match MatchGameDetailRepository::get_player_impact_history(&pool, &save_id, player_id, season_id).await {
        Ok(history) => {
            println!("[get_player_impact_history] Got {} records", history.len());
            Ok(StatsCommandResult::ok(history))
        },
        Err(e) => {
            println!("[get_player_impact_history] ERROR: {}", e);
            Ok(StatsCommandResult::err(format!("Failed to get impact history: {}", e)))
        }
    }
}

/// 获取赛事MVP排行榜（按MVP次数排序）
#[tauri::command(rename_all = "camelCase")]
pub async fn get_tournament_mvp_ranking(
    state: State<'_, AppState>,
    tournament_id: i64,
    limit: Option<i32>,
) -> Result<StatsCommandResult<Vec<PlayerTournamentStats>>, String> {
    println!("[get_tournament_mvp_ranking] Called with tournament_id={}, limit={:?}", tournament_id, limit);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => {
            println!("[get_tournament_mvp_ranking] ERROR: Database not initialized");
            return Ok(StatsCommandResult::err("Database not initialized"));
        }
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => {
            println!("[get_tournament_mvp_ranking] save_id={}", id);
            id.clone()
        },
        None => {
            println!("[get_tournament_mvp_ranking] ERROR: No save loaded");
            return Ok(StatsCommandResult::err("No save loaded"));
        }
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let limit = limit.unwrap_or(20);
    println!("[get_tournament_mvp_ranking] Querying with tournament_id={}, limit={}", tournament_id, limit);

    match PlayerTournamentStatsRepository::get_mvp_candidates(&pool, &save_id, tournament_id as u64, limit).await {
        Ok(stats_list) => {
            println!("[get_tournament_mvp_ranking] Got {} records from database", stats_list.len());
            Ok(StatsCommandResult::ok(stats_list))
        },
        Err(e) => {
            println!("[get_tournament_mvp_ranking] ERROR: {}", e);
            Ok(StatsCommandResult::err(format!("Failed to get MVP ranking: {}", e)))
        }
    }
}

/// 重新计算所有选手的年度Top得分（使用新公式：50% 影响力 + 50% 冠军加成）
#[tauri::command(rename_all = "camelCase")]
pub async fn recalculate_yearly_scores(
    state: State<'_, AppState>,
    season_id: i64,
) -> Result<StatsCommandResult<i32>, String> {
    println!("[recalculate_yearly_scores] Called with season_id={}", season_id);

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

    // 获取所有选手统计并重新计算
    let all_stats = match PlayerStatsRepository::get_all_by_season(&pool, &save_id, season_id).await {
        Ok(list) => list,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get stats: {}", e))),
    };

    let mut updated_count = 0;
    for mut stats in all_stats {
        // 重新计算（综合三要素：影响力40% + 出场30% + 冠军30%）
        stats.champion_bonus = (stats.international_titles * 3 + stats.regional_titles) as f64;
        let games_bonus = stats.games_played as f64 / 10.0;
        stats.yearly_top_score = stats.avg_impact * 0.4 + games_bonus * 0.3 + stats.champion_bonus * 0.3;

        if PlayerStatsRepository::update(&pool, &stats).await.is_ok() {
            updated_count += 1;
        }
    }

    println!("[recalculate_yearly_scores] Updated {} records", updated_count);
    Ok(StatsCommandResult::ok(updated_count))
}

/// 身价变化记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketValueChange {
    pub id: i64,
    pub season_id: i64,
    pub player_id: i64,
    pub player_name: String,
    pub old_value: i64,
    pub new_value: i64,
    pub change_amount: i64,
    pub change_percent: f64,
    pub reason: String,
    pub created_at: String,
}

/// 获取选手身价变化记录
#[tauri::command]
pub async fn get_player_market_value_changes(
    state: State<'_, AppState>,
    player_id: i64,
) -> Result<StatsCommandResult<Vec<MarketValueChange>>, String> {
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
        Err(e) => return Ok(StatsCommandResult::err(format!("Database error: {}", e))),
    };

    let rows = sqlx::query(
        r#"
        SELECT id, season_id, player_id, player_name, old_value, new_value,
               change_amount, change_percent, reason, created_at
        FROM market_value_changes
        WHERE save_id = ? AND player_id = ?
        ORDER BY created_at DESC
        "#
    )
    .bind(&save_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get market value changes: {}", e))?;

    use sqlx::Row;
    let changes: Vec<MarketValueChange> = rows.iter().map(|row| {
        MarketValueChange {
            id: row.get("id"),
            season_id: row.get("season_id"),
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            old_value: row.get("old_value"),
            new_value: row.get("new_value"),
            change_amount: row.get("change_amount"),
            change_percent: row.get("change_percent"),
            reason: row.get("reason"),
            created_at: row.get("created_at"),
        }
    }).collect();

    Ok(StatsCommandResult::ok(changes))
}
