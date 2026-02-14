use serde::{Deserialize, Serialize};
use sqlx::Row;
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
    log::debug!("Called with season_id={}, limit={:?}", season_id, limit);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => {
            log::debug!("ERROR: Database not initialized");
            return Ok(StatsCommandResult::err("Database not initialized"));
        }
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => {
            log::debug!("save_id={}", id);
            id.clone()
        },
        None => {
            log::debug!("ERROR: No save loaded");
            return Ok(StatsCommandResult::err("No save loaded"));
        }
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let limit = limit.unwrap_or(20);
    log::debug!("Querying with save_id={}, season_id={}, limit={}", save_id, season_id, limit);

    let stats_list = match PlayerStatsRepository::get_season_ranking(&pool, &save_id, season_id, limit).await {
        Ok(list) => {
            log::debug!("Got {} records from database", list.len());
            list
        },
        Err(e) => {
            log::debug!("ERROR: {}", e);
            return Ok(StatsCommandResult::err(format!("Failed to get ranking: {}", e)));
        }
    };

    let mut ranking: Vec<PlayerRankingItem> = stats_list.into_iter().map(|s| s.into()).collect();

    let big_stage_map = query_all_players_big_stage(&pool, &save_id, season_id).await;
    for player in &mut ranking {
        let (bs_score, bs_intl) = big_stage_map.get(&player.player_id).copied().unwrap_or((0.0, false));
        player.big_stage_score = bs_score;
        player.has_international = bs_intl;
    }
    ranking.sort_by(|a, b| b.yearly_top_score.partial_cmp(&a.yearly_top_score).unwrap_or(std::cmp::Ordering::Equal));

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

    let mut ranking: Vec<PlayerRankingItem> = stats_list.into_iter().map(|s| s.into()).collect();

    let big_stage_map = query_all_players_big_stage(&pool, &save_id, season_id).await;
    for player in &mut ranking {
        let (bs_score, bs_intl) = big_stage_map.get(&player.player_id).copied().unwrap_or((0.0, false));
        player.big_stage_score = bs_score;
        player.has_international = bs_intl;
    }
    ranking.sort_by(|a, b| b.yearly_top_score.partial_cmp(&a.yearly_top_score).unwrap_or(std::cmp::Ordering::Equal));

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
    log::debug!("Called with player_id={}, season_id={:?}", player_id, season_id);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => {
            log::debug!("ERROR: Database not initialized");
            return Ok(StatsCommandResult::err("Database not initialized"));
        }
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => {
            log::debug!("save_id={}", id);
            id.clone()
        },
        None => {
            log::debug!("ERROR: No save loaded");
            return Ok(StatsCommandResult::err("No save loaded"));
        }
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => {
            log::debug!("ERROR: Failed to get pool: {}", e);
            return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e)));
        }
    };

    match MatchGameDetailRepository::get_player_impact_history(&pool, &save_id, player_id, season_id).await {
        Ok(history) => {
            log::debug!("Got {} records", history.len());
            Ok(StatsCommandResult::ok(history))
        },
        Err(e) => {
            log::debug!("ERROR: {}", e);
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
    log::debug!("Called with tournament_id={}, limit={:?}", tournament_id, limit);

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => {
            log::debug!("ERROR: Database not initialized");
            return Ok(StatsCommandResult::err("Database not initialized"));
        }
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => {
            log::debug!("save_id={}", id);
            id.clone()
        },
        None => {
            log::debug!("ERROR: No save loaded");
            return Ok(StatsCommandResult::err("No save loaded"));
        }
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(StatsCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let limit = limit.unwrap_or(20);
    log::debug!("Querying with tournament_id={}, limit={}", tournament_id, limit);

    match PlayerTournamentStatsRepository::get_mvp_candidates(&pool, &save_id, tournament_id as u64, limit).await {
        Ok(stats_list) => {
            log::debug!("Got {} records from database", stats_list.len());
            Ok(StatsCommandResult::ok(stats_list))
        },
        Err(e) => {
            log::debug!("ERROR: {}", e);
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
    log::debug!("Called with season_id={}", season_id);

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
        stats.yearly_top_score = PlayerSeasonStatistics::calculate_yearly_top_score(
            stats.avg_impact,
            stats.avg_performance,
            stats.consistency_score,
            stats.games_played,
            stats.champion_bonus,
        );
        stats.dominance_score = PlayerSeasonStatistics::calculate_dominance_score(
            stats.best_performance,
            stats.avg_impact,
            stats.avg_performance,
        );

        if PlayerStatsRepository::update(&pool, &stats).await.is_ok() {
            updated_count += 1;
        }
    }

    log::debug!("Updated {} records", updated_count);
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSeasonHistoryEntry {
    pub season: String,
    pub team_name: String,
    pub ability: i64,
    pub potential: i64,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_player_season_history(
    state: State<'_, AppState>,
    player_id: i64,
) -> Result<StatsCommandResult<Vec<PlayerSeasonHistoryEntry>>, String> {
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

    let current_season: i64 = sqlx::query_scalar("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let player_row = sqlx::query(
        "SELECT ability, potential, tag, age, join_season FROM players WHERE id = ? AND save_id = ?"
    )
    .bind(player_id)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let player_row = match player_row {
        Some(r) => r,
        None => return Ok(StatsCommandResult::err("Player not found")),
    };

    let current_ability: i64 = player_row.get("ability");
    let potential: i64 = player_row.get("potential");
    let tag_str: String = player_row.get("tag");
    let current_age: i64 = player_row.get("age");
    let join_season: Option<i64> = player_row.get("join_season");

    let growth_per_season: i64 = match tag_str.as_str() {
        "Genius" => 3,
        "Normal" => 2,
        "Ordinary" => 1,
        _ => 2,
    };

    let start_season = join_season.unwrap_or(1);

    // 从 player_season_stats 获取每赛季的 team_id
    let stats_rows = sqlx::query(
        r#"
        SELECT pss.season_id, pss.team_id, COALESCE(t.name, '未知') as team_name
        FROM player_season_stats pss
        LEFT JOIN teams t ON pss.team_id = t.id
        WHERE pss.save_id = ? AND pss.player_id = ?
        ORDER BY pss.season_id
        "#
    )
    .bind(&save_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut season_team_map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    for row in &stats_rows {
        let season_id: i64 = row.get("season_id");
        let team_name: String = row.get("team_name");
        season_team_map.insert(season_id, team_name);
    }

    // 如果 player_season_stats 没有数据，用当前队伍填充
    if season_team_map.is_empty() {
        let team_name: String = sqlx::query_scalar(
            "SELECT COALESCE(t.name, '未知') FROM players p LEFT JOIN teams t ON p.team_id = t.id WHERE p.id = ? AND p.save_id = ?"
        )
        .bind(player_id)
        .bind(&save_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "未知".to_string());

        for s in start_season..=current_season {
            season_team_map.insert(s, team_name.clone());
        }
    }

    // 从当前赛季反推每赛季的 ability
    // 当前赛季的 ability 是已知的，往前推
    let mut history: Vec<PlayerSeasonHistoryEntry> = Vec::new();

    for season in start_season..=current_season {
        let seasons_ago = current_season - season;
        let _age_at_season = current_age - seasons_ago;

        // 反推 ability：从当前值往回减去成长/加上衰退
        let mut ability = current_ability;
        for s in (season..current_season).rev() {
            let age_at_s = current_age - (current_season - s);
            // 这个赛季结束时发生了什么变化？（s → s+1 的变化）
            let age_after = age_at_s + 1; // 赛季结束后年龄+1
            if age_at_s < 28 {
                // 成长阶段：s+1 的 ability = s 的 ability + growth（上限 potential）
                // 反推：s 的 ability = s+1 的 ability - growth
                ability -= growth_per_season;
            } else if age_after >= 30 {
                // 衰退阶段：s+1 的 ability = s 的 ability - decline
                // 反推：s 的 ability = s+1 的 ability + decline
                let decline = match age_after {
                    30..=31 => 1,
                    32..=33 => 2,
                    34..=35 => 3,
                    _ => 4,
                };
                ability += decline;
            }
            // 28-29岁：不变
        }

        let team_name = season_team_map.get(&season)
            .or_else(|| {
                // 如果该赛季没有统计记录，找最近的前一个赛季的队伍
                (start_season..season).rev()
                    .find_map(|s| season_team_map.get(&s))
            })
            .cloned()
            .unwrap_or_else(|| {
                // 最后兜底：用当前赛季的队伍
                season_team_map.values().next().cloned().unwrap_or_else(|| "未知".to_string())
            });

        history.push(PlayerSeasonHistoryEntry {
            season: format!("S{}", season),
            team_name,
            ability: ability.max(0),
            potential,
        });
    }

    Ok(StatsCommandResult::ok(history))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerContractRecord {
    pub season: String,
    pub event_type: String,
    pub team_name: String,
    pub salary: i64,
    pub contract_years: i64,
    pub transfer_fee: i64,
    pub reason: Option<String>,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_player_contract_history(
    state: State<'_, AppState>,
    player_id: i64,
) -> Result<StatsCommandResult<Vec<PlayerContractRecord>>, String> {
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

    let rows = sqlx::query(
        r#"
        SELECT season_id, event_type, 
               COALESCE(to_team_name, from_team_name, '') as team_name,
               salary, COALESCE(contract_years, 0) as contract_years,
               transfer_fee, reason
        FROM transfer_events
        WHERE save_id = ? AND player_id = ?
          AND event_type IN ('CONTRACT_RENEWAL', 'FREE_AGENT_SIGNING', 'TRANSFER_PURCHASE', 'EMERGENCY_SIGNING', 'SEASON_SETTLEMENT')
        ORDER BY season_id ASC, id ASC
        "#
    )
    .bind(&save_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get contract history: {}", e))?;

    let records: Vec<PlayerContractRecord> = rows.iter().map(|row| {
        let event_type: String = row.get("event_type");
        let display_type = match event_type.as_str() {
            "CONTRACT_RENEWAL" => "续约",
            "FREE_AGENT_SIGNING" => "自由签约",
            "TRANSFER_PURCHASE" => "转会加盟",
            "EMERGENCY_SIGNING" => "紧急签约",
            "SEASON_SETTLEMENT" => "赛季结算",
            _ => &event_type,
        };
        PlayerContractRecord {
            season: format!("S{}", row.get::<i64, _>("season_id")),
            event_type: display_type.to_string(),
            team_name: row.get("team_name"),
            salary: row.get("salary"),
            contract_years: row.get("contract_years"),
            transfer_fee: row.get("transfer_fee"),
            reason: row.get("reason"),
        }
    }).collect();

    Ok(StatsCommandResult::ok(records))
}

/// 选手赛事表现历史（用于箱线图 —— 每个赛事的 avg_impact / max_impact / games_played 等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTournamentHistoryItem {
    pub tournament_type: String,
    pub season_id: i64,
    pub games_played: i64,
    pub avg_impact: f64,
    pub max_impact: f64,
    pub avg_performance: f64,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_player_tournament_history(
    state: State<'_, AppState>,
    player_id: i64,
    season_id: Option<i64>,
) -> Result<StatsCommandResult<Vec<PlayerTournamentHistoryItem>>, String> {
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

    let rows = if let Some(sid) = season_id {
        sqlx::query(
            r#"
            SELECT tournament_type, season_id, games_played, avg_impact, max_impact, avg_performance
            FROM player_tournament_stats
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            ORDER BY tournament_id ASC
            "#
        )
        .bind(&save_id)
        .bind(player_id)
        .bind(sid)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT tournament_type, season_id, games_played, avg_impact, max_impact, avg_performance
            FROM player_tournament_stats
            WHERE save_id = ? AND player_id = ?
            ORDER BY season_id ASC, tournament_id ASC
            "#
        )
        .bind(&save_id)
        .bind(player_id)
        .fetch_all(&pool)
        .await
    }.map_err(|e| format!("Failed to get tournament history: {}", e))?;

    use sqlx::Row;
    let items: Vec<PlayerTournamentHistoryItem> = rows.iter().map(|row| {
        PlayerTournamentHistoryItem {
            tournament_type: row.get("tournament_type"),
            season_id: row.get("season_id"),
            games_played: row.get::<i64, _>("games_played"),
            avg_impact: row.get("avg_impact"),
            max_impact: row.get("max_impact"),
            avg_performance: row.get("avg_performance"),
        }
    }).collect();

    Ok(StatsCommandResult::ok(items))
}

/// 选手年度 Top 排名走势（每赛季的 yearly_top_score + 排名）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerYearlyTopItem {
    pub season: String,
    pub yearly_top_score: f64,
    pub rank: i64,       // 0 表示未上榜
    pub total_players: i64,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_player_yearly_top_history(
    state: State<'_, AppState>,
    player_id: i64,
) -> Result<StatsCommandResult<Vec<PlayerYearlyTopItem>>, String> {
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

    // 查出该选手所有赛季的 yearly_top_score
    let player_rows = sqlx::query(
        r#"
        SELECT season_id, yearly_top_score
        FROM player_season_stats
        WHERE save_id = ? AND player_id = ? AND season_id <= 100
        ORDER BY season_id ASC
        "#
    )
    .bind(&save_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get yearly top history: {}", e))?;

    use sqlx::Row;
    let mut items: Vec<PlayerYearlyTopItem> = Vec::new();

    for row in &player_rows {
        let season_id: i64 = row.get("season_id");
        let score: f64 = row.get("yearly_top_score");

        // 查该赛季所有选手的排名（按 yearly_top_score DESC，只计有比赛记录的选手）
        let rank_row = sqlx::query(
            r#"
            SELECT COUNT(*) as rank
            FROM player_season_stats
            WHERE save_id = ? AND season_id = ? AND season_id <= 100 AND games_played > 0 AND yearly_top_score > ?
            "#
        )
        .bind(&save_id)
        .bind(season_id)
        .bind(score)
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to calculate rank: {}", e))?;

        let rank: i64 = rank_row.get::<i64, _>("rank") + 1;

        let total_row = sqlx::query(
            r#"
            SELECT COUNT(*) as total
            FROM player_season_stats
            WHERE save_id = ? AND season_id = ? AND season_id <= 100 AND games_played > 0
            "#
        )
        .bind(&save_id)
        .bind(season_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to count total: {}", e))?;

        let total: i64 = total_row.get("total");

        items.push(PlayerYearlyTopItem {
            season: format!("S{}", season_id),
            yearly_top_score: score,
            rank,
            total_players: total,
        });
    }

    Ok(StatsCommandResult::ok(items))
}

async fn query_all_players_big_stage(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: i64,
) -> std::collections::HashMap<i64, (f64, bool)> {
    let rows = sqlx::query(
        r#"
        SELECT gpp.player_id, t.tournament_type,
               COUNT(DISTINCT gpp.game_id) as games_played,
               AVG(gpp.impact_score) as avg_impact
        FROM game_player_performances gpp
        JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
        JOIN matches m ON mg.match_id = m.id AND mg.save_id = m.save_id
        JOIN tournaments t ON m.tournament_id = t.id AND m.save_id = t.save_id
        WHERE gpp.save_id = ? AND t.season_id = ?
        AND t.tournament_type NOT IN ('SpringRegular', 'SpringPlayoffs', 'SummerRegular', 'SummerPlayoffs')
        GROUP BY gpp.player_id, t.tournament_type
        "#
    )
    .bind(save_id)
    .bind(season_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut player_intl: std::collections::HashMap<i64, (f64, f64)> = std::collections::HashMap::new();
    for row in &rows {
        let pid: i64 = row.get("player_id");
        let games = row.get::<i64, _>("games_played") as f64;
        let avg_impact: f64 = row.get("avg_impact");
        let entry = player_intl.entry(pid).or_insert((0.0, 0.0));
        entry.0 += avg_impact * games;
        entry.1 += games;
    }

    player_intl.into_iter().map(|(pid, (impact_sum, games))| {
        let raw_score = if games > 0.0 { impact_sum / games } else { 0.0 };
        let confidence = (games / 70.0).min(1.0);
        (pid, (raw_score * confidence, true))
    }).collect()
}

#[allow(dead_code)]
fn calc_6dim_score(
    avg_impact: f64,
    avg_performance: f64,
    consistency_score: f64,
    games_played: i32,
    champion_bonus: f64,
    big_stage_score: f64,
    has_international: bool,
) -> f64 {
    PlayerSeasonStatistics::calculate_yearly_top_score_6dim(
        avg_impact, avg_performance, consistency_score,
        games_played, champion_bonus, big_stage_score, has_international,
    )
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerGrowthLogItem {
    pub season_id: i64,
    pub player_name: String,
    pub team_name: String,
    pub age: i64,
    pub old_ability: i64,
    pub new_ability: i64,
    pub old_potential: i64,
    pub new_potential: i64,
    pub base_growth: f64,
    pub age_coeff: f64,
    pub playtime_coeff: f64,
    pub mentor_coeff: f64,
    pub synergy_coeff: f64,
    pub facility_coeff: f64,
    pub prodigy_mod: f64,
    pub perf_bonus: f64,
    pub fluctuation: f64,
    pub growth_event: Option<String>,
    pub description: String,
}

#[tauri::command]
pub async fn get_player_growth_logs(
    state: State<'_, AppState>,
    save_id: String,
    player_id: i64,
) -> Result<StatsCommandResult<Vec<PlayerGrowthLogItem>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(StatsCommandResult::err("Database not initialized")),
    };
    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    let rows = sqlx::query(
        "SELECT season_id, player_name, team_name, age, old_ability, new_ability,
                old_potential, new_potential, base_growth, age_coeff, playtime_coeff,
                mentor_coeff, synergy_coeff, facility_coeff, prodigy_mod,
                perf_bonus, fluctuation, growth_event, description
         FROM player_growth_logs
         WHERE save_id = ? AND player_id = ?
         ORDER BY season_id ASC"
    )
    .bind(&save_id)
    .bind(player_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let result: Vec<PlayerGrowthLogItem> = rows.iter().map(|r| {
        PlayerGrowthLogItem {
            season_id: r.get("season_id"),
            player_name: r.get("player_name"),
            team_name: r.get("team_name"),
            age: r.get("age"),
            old_ability: r.get("old_ability"),
            new_ability: r.get("new_ability"),
            old_potential: r.get("old_potential"),
            new_potential: r.get("new_potential"),
            base_growth: r.get("base_growth"),
            age_coeff: r.get("age_coeff"),
            playtime_coeff: r.get("playtime_coeff"),
            mentor_coeff: r.get("mentor_coeff"),
            synergy_coeff: r.get("synergy_coeff"),
            facility_coeff: r.get("facility_coeff"),
            prodigy_mod: r.get("prodigy_mod"),
            perf_bonus: r.get("perf_bonus"),
            fluctuation: r.get("fluctuation"),
            growth_event: r.get("growth_event"),
            description: r.get("description"),
        }
    }).collect();

    Ok(StatsCommandResult::ok(result))
}
