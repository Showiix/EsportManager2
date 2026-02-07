use crate::commands::{ApiResponse, AppState};
use crate::db::{HonorRepository, PlayerRepository, TeamRepository, TournamentResultRepository};
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

// ==================== 荣誉殿堂新增数据结构 ====================

/// 国际赛事冠军卡片
#[derive(Debug, Serialize, Deserialize)]
pub struct InternationalChampionCard {
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_name: String,
    pub tournament_type: String,
    pub champion_team_id: u64,
    pub champion_team_name: String,
    pub final_score: Option<String>,
}

/// 国际赛事冠军详情（展开后）
#[derive(Debug, Serialize, Deserialize)]
pub struct ChampionDetail {
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_name: String,
    pub tournament_type: String,
    pub champion_team_id: u64,
    pub champion_team_name: String,
    pub champion_roster: Vec<RosterMember>,
    pub runner_up_team_id: u64,
    pub runner_up_team_name: String,
    pub third_team_id: Option<u64>,
    pub third_team_name: Option<String>,
    pub fourth_team_id: Option<u64>,
    pub fourth_team_name: Option<String>,
    pub final_score: Option<String>,
}

/// 阵容成员
#[derive(Debug, Serialize, Deserialize)]
pub struct RosterMember {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
}

/// 选手荣誉排行榜条目
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerHonorRanking {
    pub rank: u32,
    pub player_id: u64,
    pub player_name: String,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub position: Option<String>,
    pub champion_count: u32,
    pub mvp_count: u32,
    pub international_champion_count: u32,
}

/// 战队荣誉排行榜条目
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamHonorRanking {
    pub rank: u32,
    pub team_id: u64,
    pub team_name: String,
    pub champion_count: u32,
    pub international_champion_count: u32,
    pub runner_up_count: u32,
}

/// 选手荣誉详情
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerHonorDetail {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub position: Option<String>,
    pub champion_count: u32,
    pub mvp_count: u32,
    pub international_champion_count: u32,
    pub honors: Vec<Honor>,
}

/// 战队荣誉详情
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamHonorDetail {
    pub team_id: u64,
    pub team_name: String,
    pub champion_count: u32,
    pub international_champion_count: u32,
    pub runner_up_count: u32,
    pub third_count: u32,
    pub honors: Vec<Honor>,
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

// ==================== 荣誉殿堂新增命令 ====================

/// 获取国际赛事冠军列表（旗帜墙）
#[tauri::command]
pub async fn get_international_champions(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<InternationalChampionCard>>, String> {
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

    // 获取国际赛事冠军
    let honors = match HonorRepository::get_international_champions(&pool, &save_id).await {
        Ok(h) => h,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get champions: {}", e))),
    };

    // 获取赛事结果信息（用于决赛比分）
    let results = match TournamentResultRepository::get_all(&pool, &save_id).await {
        Ok(r) => r,
        Err(_) => vec![],
    };

    // 构建结果映射
    let result_map: std::collections::HashMap<u64, _> = results
        .into_iter()
        .map(|r| (r.tournament_id, r))
        .collect();

    // 转换为卡片格式
    let cards: Vec<InternationalChampionCard> = honors
        .into_iter()
        .filter_map(|h| {
            let tid = h.tournament_id?;
            let final_score = result_map
                .get(&tid)
                .and_then(|r| r.final_score.clone());

            Some(InternationalChampionCard {
                season_id: h.season_id,
                tournament_id: tid,
                tournament_name: h.tournament_name,
                tournament_type: h.tournament_type,
                champion_team_id: h.team_id.unwrap_or(0),
                champion_team_name: h.team_name.unwrap_or_default(),
                final_score,
            })
        })
        .collect();

    Ok(ApiResponse::success(cards))
}

/// 获取冠军详情（展开后显示阵容等）
#[tauri::command]
pub async fn get_champion_detail(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<ApiResponse<ChampionDetail>, String> {
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

    // 获取赛事结果
    let result = match TournamentResultRepository::get_by_tournament(&pool, &save_id, tournament_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(ApiResponse::error("Tournament result not found")),
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get result: {}", e))),
    };

    // 获取冠军阵容（从选手荣誉记录中获取）
    let champion_honors = match HonorRepository::get_by_tournament(&pool, &save_id, tournament_id).await {
        Ok(h) => h,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get honors: {}", e))),
    };

    let champion_roster: Vec<RosterMember> = champion_honors
        .iter()
        .filter(|h| h.honor_type == crate::models::HonorType::PlayerChampion)
        .map(|h| RosterMember {
            player_id: h.player_id.unwrap_or(0),
            player_name: h.player_name.clone().unwrap_or_default(),
            position: h.position.clone().unwrap_or_default(),
        })
        .collect();

    let detail = ChampionDetail {
        season_id: result.season_id,
        tournament_id: result.tournament_id,
        tournament_name: result.tournament_name,
        tournament_type: result.tournament_type,
        champion_team_id: result.champion_team_id,
        champion_team_name: result.champion_team_name,
        champion_roster,
        runner_up_team_id: result.runner_up_team_id,
        runner_up_team_name: result.runner_up_team_name,
        third_team_id: result.third_team_id,
        third_team_name: result.third_team_name,
        fourth_team_id: result.fourth_team_id,
        fourth_team_name: result.fourth_team_name,
        final_score: result.final_score,
    };

    Ok(ApiResponse::success(detail))
}

/// 获取选手荣誉排行榜
#[tauri::command]
pub async fn get_player_honor_rankings(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<ApiResponse<Vec<PlayerHonorRanking>>, String> {
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

    let limit = limit.unwrap_or(50);

    let rankings = match HonorRepository::get_player_honor_rankings(&pool, &save_id, limit).await {
        Ok(r) => r,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get rankings: {}", e))),
    };

    // 获取选手当前信息
    let mut result = Vec::new();
    for (i, (player_id, player_name, champion_count, mvp_count, intl_count)) in rankings.into_iter().enumerate() {
        // 尝试获取选手当前信息
        let (team_id, team_name, position) = match PlayerRepository::get_by_id(&pool, player_id).await {
            Ok(player) => {
                let pos = player.position.map(|p| format!("{:?}", p));
                // 获取队伍名称
                let t_name = if let Some(tid) = player.team_id {
                    TeamRepository::get_by_id(&pool, tid)
                        .await
                        .ok()
                        .map(|t| t.name)
                } else {
                    None
                };
                (player.team_id, t_name, pos)
            }
            Err(_) => (None, None, None),
        };

        result.push(PlayerHonorRanking {
            rank: (i + 1) as u32,
            player_id,
            player_name,
            team_id,
            team_name,
            position,
            champion_count,
            mvp_count,
            international_champion_count: intl_count,
        });
    }

    Ok(ApiResponse::success(result))
}

/// 获取战队荣誉排行榜
#[tauri::command]
pub async fn get_team_honor_rankings(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<ApiResponse<Vec<TeamHonorRanking>>, String> {
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

    let limit = limit.unwrap_or(50);

    let rankings = match HonorRepository::get_team_honor_rankings(&pool, &save_id, limit).await {
        Ok(r) => r,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get rankings: {}", e))),
    };

    let result: Vec<TeamHonorRanking> = rankings
        .into_iter()
        .enumerate()
        .map(|(i, (team_id, team_name, champion_count, intl_count, runner_up_count))| {
            TeamHonorRanking {
                rank: (i + 1) as u32,
                team_id,
                team_name,
                champion_count,
                international_champion_count: intl_count,
                runner_up_count,
            }
        })
        .collect();

    Ok(ApiResponse::success(result))
}

/// 获取选手荣誉详情
#[tauri::command]
pub async fn get_player_honor_detail(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<ApiResponse<PlayerHonorDetail>, String> {
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

    // 获取选手信息
    let player = match PlayerRepository::get_by_id(&pool, player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(&format!("Player not found: {}", e))),
    };

    // 获取所有荣誉
    let honors = match HonorRepository::get_by_player(&pool, &save_id, player_id).await {
        Ok(h) => h,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get honors: {}", e))),
    };

    // 统计数量
    let champion_count = match HonorRepository::count_player_champions(&pool, &save_id, player_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let mvp_count = match HonorRepository::count_player_mvps(&pool, &save_id, player_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let intl_count = match HonorRepository::count_player_international_champions(&pool, &save_id, player_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let detail = PlayerHonorDetail {
        player_id,
        player_name: player.game_id,
        team_id: player.team_id,
        team_name: None, // 需要额外查询
        position: player.position.map(|p| format!("{:?}", p)),
        champion_count,
        mvp_count,
        international_champion_count: intl_count,
        honors,
    };

    Ok(ApiResponse::success(detail))
}

/// 获取战队荣誉详情
#[tauri::command]
pub async fn get_team_honor_detail(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<ApiResponse<TeamHonorDetail>, String> {
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

    // 获取战队信息
    let team = match crate::db::TeamRepository::get_by_id(&pool, team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(ApiResponse::error(&format!("Team not found: {}", e))),
    };

    // 获取所有荣誉
    let honors = match HonorRepository::get_by_team(&pool, &save_id, team_id).await {
        Ok(h) => h,
        Err(e) => return Ok(ApiResponse::error(&format!("Failed to get honors: {}", e))),
    };

    // 统计数量
    let champion_count = match HonorRepository::count_team_champions(&pool, &save_id, team_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let intl_count = match HonorRepository::count_team_international_champions(&pool, &save_id, team_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let runner_up_count = match HonorRepository::count_team_runner_ups(&pool, &save_id, team_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let third_count = match HonorRepository::count_team_thirds(&pool, &save_id, team_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    let detail = TeamHonorDetail {
        team_id,
        team_name: team.name,
        champion_count,
        international_champion_count: intl_count,
        runner_up_count,
        third_count,
        honors,
    };

    Ok(ApiResponse::success(detail))
}

/// 清理重复数据响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupDuplicatesResponse {
    pub deleted_count: u32,
    pub message: String,
}

/// 清理重复的荣誉记录
/// 保留每个(tournament_id, honor_type, player_id, team_id)组合的第一条记录，删除其余重复记录
#[tauri::command]
pub async fn cleanup_duplicate_honors(
    state: State<'_, AppState>,
) -> Result<ApiResponse<CleanupDuplicatesResponse>, String> {
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

    // 使用 SQL 删除重复记录，保留每组的第一条（id 最小的）
    let result = sqlx::query(
        r#"
        DELETE FROM honors
        WHERE id NOT IN (
            SELECT MIN(id)
            FROM honors
            WHERE save_id = ?
            GROUP BY save_id, tournament_id, honor_type,
                     COALESCE(player_id, 0), COALESCE(team_id, 0)
        ) AND save_id = ?
        "#
    )
    .bind(&save_id)
    .bind(&save_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to cleanup duplicates: {}", e))?;

    let deleted_count = result.rows_affected() as u32;
    let message = if deleted_count > 0 {
        format!("成功清理 {} 条重复荣誉记录", deleted_count)
    } else {
        "没有发现重复的荣誉记录".to_string()
    };

    log::debug!("{}", message);

    Ok(ApiResponse::success(CleanupDuplicatesResponse {
        deleted_count,
        message,
    }))
}

/// 重新生成赛事荣誉响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RegenerateHonorsResponse {
    pub deleted_count: u32,
    pub created_count: u32,
    pub message: String,
}

/// 重新生成指定赛事的荣誉（删除旧荣誉后重新颁发）
#[tauri::command(rename_all = "camelCase")]
pub async fn regenerate_tournament_honors(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<ApiResponse<RegenerateHonorsResponse>, String> {
    use crate::services::HonorService;

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

    // 1. 删除该赛事的所有现有荣誉
    let delete_result = sqlx::query(
        "DELETE FROM honors WHERE save_id = ? AND tournament_id = ?"
    )
    .bind(&save_id)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to delete existing honors: {}", e))?;

    let deleted_count = delete_result.rows_affected() as u32;
    log::debug!("删除了 {} 条旧荣誉记录, tournament_id={}", deleted_count, tournament_id);

    // 2. 重新生成荣誉
    let honor_service = HonorService::new();
    let tournament_honors = honor_service
        .process_tournament_completion(&pool, &save_id, tournament_id)
        .await
        .map_err(|e| format!("Failed to regenerate honors: {}", e))?;

    // 计算创建的荣誉数量
    let mut created_count = 0u32;
    if tournament_honors.team_champion.is_some() { created_count += 1; }
    if tournament_honors.team_runner_up.is_some() { created_count += 1; }
    if tournament_honors.team_third.is_some() { created_count += 1; }
    if tournament_honors.team_fourth.is_some() { created_count += 1; }
    created_count += tournament_honors.player_champions.len() as u32;
    created_count += tournament_honors.player_runner_ups.len() as u32;
    created_count += tournament_honors.player_thirds.len() as u32;
    created_count += tournament_honors.player_fourths.len() as u32;
    if tournament_honors.tournament_mvp.is_some() { created_count += 1; }
    if tournament_honors.finals_mvp.is_some() { created_count += 1; }

    let message = format!(
        "成功重新生成赛事荣誉：删除 {} 条旧记录，创建 {} 条新记录",
        deleted_count, created_count
    );
    log::debug!("{}", message);

    Ok(ApiResponse::success(RegenerateHonorsResponse {
        deleted_count,
        created_count,
        message,
    }))
}

/// 批量重新生成所有已完成赛事的荣誉
#[tauri::command(rename_all = "camelCase")]
pub async fn regenerate_all_honors(
    state: State<'_, AppState>,
) -> Result<ApiResponse<RegenerateHonorsResponse>, String> {
    use crate::services::HonorService;
    use crate::db::TournamentRepository;

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

    // 1. 获取所有已完成的赛事
    let completed_tournaments = TournamentRepository::get_completed(&pool, &save_id)
        .await
        .map_err(|e| format!("Failed to get completed tournaments: {}", e))?;

    log::debug!("找到 {} 个已完成的赛事", completed_tournaments.len());

    let mut total_deleted = 0u32;
    let mut total_created = 0u32;

    let honor_service = HonorService::new();

    for tournament in &completed_tournaments {
        // 删除旧荣誉
        let delete_result = sqlx::query(
            "DELETE FROM honors WHERE save_id = ? AND tournament_id = ?"
        )
        .bind(&save_id)
        .bind(tournament.id as i64)
        .execute(&pool)
        .await;

        if let Ok(result) = delete_result {
            total_deleted += result.rows_affected() as u32;
        }

        // 重新生成荣誉
        match honor_service.process_tournament_completion(&pool, &save_id, tournament.id).await {
            Ok(honors) => {
                let mut count = 0u32;
                if honors.team_champion.is_some() { count += 1; }
                if honors.team_runner_up.is_some() { count += 1; }
                if honors.team_third.is_some() { count += 1; }
                if honors.team_fourth.is_some() { count += 1; }
                count += honors.player_champions.len() as u32;
                count += honors.player_runner_ups.len() as u32;
                count += honors.player_thirds.len() as u32;
                count += honors.player_fourths.len() as u32;
                if honors.tournament_mvp.is_some() { count += 1; }
                if honors.finals_mvp.is_some() { count += 1; }
                total_created += count;
                log::debug!("赛事 {} ({}) 生成了 {} 条荣誉",
                    tournament.name, tournament.id, count);
            }
            Err(e) => {
                log::debug!("赛事 {} ({}) 生成荣誉失败: {}",
                    tournament.name, tournament.id, e);
            }
        }
    }

    let message = format!(
        "成功重新生成 {} 个赛事的荣誉：删除 {} 条旧记录，创建 {} 条新记录",
        completed_tournaments.len(), total_deleted, total_created
    );
    log::debug!("{}", message);

    Ok(ApiResponse::success(RegenerateHonorsResponse {
        deleted_count: total_deleted,
        created_count: total_created,
        message,
    }))
}
