use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{PlayerRepository, TeamRepository};
use crate::engines::{MarketValueEngine, PlayerHonorRecord, PlayerFormFactors, ConditionEngine, TraitType};
use crate::models::{Player, Team};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use tauri::State;

/// 队伍信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInfo {
    pub id: u64,
    pub region_id: u64,
    pub name: String,
    pub short_name: Option<String>,
    pub power_rating: f64,
    pub total_matches: u32,
    pub wins: u32,
    pub win_rate: f64,
    pub annual_points: u32,
    pub balance: i64,
}

impl From<Team> for TeamInfo {
    fn from(t: Team) -> Self {
        Self {
            id: t.id,
            region_id: t.region_id,
            name: t.name,
            short_name: t.short_name,
            power_rating: t.power_rating,
            total_matches: t.total_matches,
            wins: t.wins,
            win_rate: t.win_rate,
            annual_points: t.annual_points,
            balance: t.balance,
        }
    }
}

/// 队伍阵容（包含首发和替补）
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamRoster {
    pub team: TeamInfo,
    pub starters: Vec<PlayerInfo>,
    pub substitutes: Vec<PlayerInfo>,
}

/// 选手信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: u64,
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub stability: u8,
    pub tag: String,
    pub status: String,
    pub position: Option<String>,
    pub team_id: Option<u64>,
    pub salary: u64,
    pub market_value: u64,
    pub calculated_market_value: u64,
    pub contract_end_season: Option<u32>,
    pub is_starter: bool,
    pub satisfaction: u8,
    pub loyalty: u8,
}

impl From<Player> for PlayerInfo {
    fn from(p: Player) -> Self {
        Self {
            id: p.id,
            game_id: p.game_id,
            real_name: p.real_name,
            nationality: p.nationality,
            age: p.age,
            ability: p.ability,
            potential: p.potential,
            stability: p.stability,
            tag: format!("{:?}", p.tag),
            status: format!("{:?}", p.status),
            position: p.position.map(|pos| format!("{:?}", pos)),
            team_id: p.team_id,
            salary: p.salary,
            market_value: p.market_value,
            calculated_market_value: p.calculated_market_value,
            contract_end_season: p.contract_end_season,
            is_starter: p.is_starter,
            satisfaction: p.satisfaction,
            loyalty: p.loyalty,
        }
    }
}

/// 从 league_standings 聚合战队比赛数据
async fn populate_team_stats(pool: &Pool<Sqlite>, save_id: &str, teams: &mut Vec<TeamInfo>) {
    let rows = sqlx::query(
        r#"
        SELECT s.team_id,
               COALESCE(SUM(s.matches_played), 0) as total_matches,
               COALESCE(SUM(s.wins), 0) as total_wins
        FROM league_standings s
        WHERE s.save_id = ?
        GROUP BY s.team_id
        "#,
    )
    .bind(save_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let stats: std::collections::HashMap<u64, (u32, u32)> = rows
        .iter()
        .map(|row| {
            let team_id = row.get::<i64, _>("team_id") as u64;
            let total = row.get::<i64, _>("total_matches") as u32;
            let wins = row.get::<i64, _>("total_wins") as u32;
            (team_id, (total, wins))
        })
        .collect();

    for team in teams.iter_mut() {
        if let Some(&(total, wins)) = stats.get(&team.id) {
            team.total_matches = total;
            team.wins = wins;
            team.win_rate = if total > 0 {
                wins as f64 / total as f64
            } else {
                0.0
            };
        }
    }
}

/// 获取赛区队伍列表
#[tauri::command]
pub async fn get_teams_by_region(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<Vec<TeamInfo>>, String> {
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

    let teams = match TeamRepository::get_by_region(&pool, &save_id, region_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get teams: {}", e))),
    };

    let mut infos: Vec<TeamInfo> = teams.into_iter().map(|t| t.into()).collect();
    populate_team_stats(&pool, &save_id, &mut infos).await;
    Ok(CommandResult::ok(infos))
}

/// 获取所有队伍
#[tauri::command]
pub async fn get_all_teams(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<TeamInfo>>, String> {
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

    let teams = match TeamRepository::get_all(&pool, &save_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get teams: {}", e))),
    };

    let mut infos: Vec<TeamInfo> = teams.into_iter().map(|t| t.into()).collect();
    populate_team_stats(&pool, &save_id, &mut infos).await;
    Ok(CommandResult::ok(infos))
}

/// 获取所有活跃选手（带队伍信息）
#[tauri::command]
pub async fn get_all_players(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<PlayerInfo>>, String> {
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

    // 获取所有活跃选手
    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get players: {}", e))),
    };

    let infos: Vec<PlayerInfo> = players.into_iter().map(|p| p.into()).collect();
    Ok(CommandResult::ok(infos))
}

/// 获取单个队伍
#[tauri::command]
pub async fn get_team(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<CommandResult<TeamInfo>, String> {
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

    let team = match TeamRepository::get_by_id(&pool, team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get team: {}", e))),
    };

    let mut infos = vec![TeamInfo::from(team)];
    populate_team_stats(&pool, &save_id, &mut infos).await;
    Ok(CommandResult::ok(infos.into_iter().next().unwrap()))
}

/// 获取队伍阵容（首发+替补）
#[tauri::command]
pub async fn get_team_roster(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<CommandResult<TeamRoster>, String> {
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

    // 获取队伍信息
    let team = match TeamRepository::get_by_id(&pool, team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get team: {}", e))),
    };

    let mut team_info = TeamInfo::from(team);
    let mut infos = vec![team_info];
    populate_team_stats(&pool, &save_id, &mut infos).await;
    team_info = infos.into_iter().next().unwrap();

    // 获取所有选手
    let players = match PlayerRepository::get_by_team(&pool, team_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get players: {}", e))),
    };

    // 分为首发和替补
    let (starters, substitutes): (Vec<_>, Vec<_>) = players
        .into_iter()
        .partition(|p| p.is_starter);

    Ok(CommandResult::ok(TeamRoster {
        team: team_info,
        starters: starters.into_iter().map(|p| p.into()).collect(),
        substitutes: substitutes.into_iter().map(|p| p.into()).collect(),
    }))
}

/// 获取首发阵容
#[tauri::command]
pub async fn get_team_starters(
    state: State<'_, AppState>,
    team_id: u64,
) -> Result<CommandResult<Vec<PlayerInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let players = match PlayerRepository::get_starters(&pool, team_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get starters: {}", e))),
    };

    let infos: Vec<PlayerInfo> = players.into_iter().map(|p| p.into()).collect();
    Ok(CommandResult::ok(infos))
}

/// 获取选手详情
#[tauri::command]
pub async fn get_player(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<CommandResult<PlayerInfo>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let player = match PlayerRepository::get_by_id(&pool, player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get player: {}", e))),
    };

    Ok(CommandResult::ok(player.into()))
}

/// 设置首发阵容
#[tauri::command]
pub async fn set_starter(
    state: State<'_, AppState>,
    player_id: u64,
    is_starter: bool,
) -> Result<CommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取选手并更新
    let mut player = match PlayerRepository::get_by_id(&pool, player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get player: {}", e))),
    };

    player.is_starter = is_starter;

    if let Err(e) = PlayerRepository::update(&pool, &player).await {
        return Ok(CommandResult::err(format!("Failed to update player: {}", e)));
    }

    Ok(CommandResult::ok(()))
}

/// 更新战队基本信息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamRequest {
    pub team_id: u64,
    pub name: Option<String>,
    pub short_name: Option<String>,
}

/// 更新战队基本信息
#[tauri::command]
pub async fn update_team(
    state: State<'_, AppState>,
    request: UpdateTeamRequest,
) -> Result<CommandResult<TeamInfo>, String> {
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

    let mut team = match TeamRepository::get_by_id(&pool, request.team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get team: {}", e))),
    };

    if let Some(name) = request.name {
        if !name.trim().is_empty() {
            team.name = name.trim().to_string();
        }
    }
    if let Some(short_name) = request.short_name {
        team.short_name = if short_name.trim().is_empty() {
            None
        } else {
            Some(short_name.trim().to_string())
        };
    }

    if let Err(e) = TeamRepository::update(&pool, &team).await {
        return Ok(CommandResult::err(format!("Failed to update team: {}", e)));
    }

    log::debug!("战队 {} 信息已更新: name={}, short_name={:?}",
        team.id, team.name, team.short_name);

    let mut infos = vec![TeamInfo::from(team)];
    populate_team_stats(&pool, &save_id, &mut infos).await;
    Ok(CommandResult::ok(infos.into_iter().next().unwrap()))
}

/// 身价更新结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketValueUpdateResult {
    pub player_id: u64,
    pub player_name: String,
    pub old_value: u64,
    pub new_value: u64,
    pub honor_factor: f64,
}

/// 更新单个选手身价
#[tauri::command]
pub async fn update_player_market_value(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<CommandResult<MarketValueUpdateResult>, String> {
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

    // 获取选手信息
    let mut player = match PlayerRepository::get_by_id(&pool, player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get player: {}", e))),
    };

    let old_value = player.market_value;

    // 获取当前赛季
    let current_season: u32 = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map(|row| row.get::<i64, _>("current_season") as u32)
        .unwrap_or(1);

    // 获取选手所属赛区
    let region_code = if let Some(team_id) = player.team_id {
        sqlx::query(
            r#"
            SELECT r.short_name FROM teams t
            JOIN regions r ON t.region_id = r.id
            WHERE t.id = ?
            "#
        )
        .bind(team_id as i64)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten()
        .map(|row| row.get::<String, _>("short_name"))
        .unwrap_or_else(|| "LPL".to_string())
    } else {
        "LPL".to_string()
    };

    // 获取选手荣誉记录
    let honors = get_player_honors(&pool, &save_id, player_id).await;

    // 计算荣誉系数
    let honor_factor = MarketValueEngine::calculate_honor_factor(&honors, current_season);

    // 计算新身价
    let new_value = player.calculate_full_market_value(&region_code, honor_factor);

    // 更新数据库
    player.market_value = new_value;
    if let Err(e) = PlayerRepository::update(&pool, &player).await {
        return Ok(CommandResult::err(format!("Failed to update player: {}", e)));
    }

    Ok(CommandResult::ok(MarketValueUpdateResult {
        player_id,
        player_name: player.game_id,
        old_value,
        new_value,
        honor_factor,
    }))
}

/// 批量更新所有选手身价
#[tauri::command]
pub async fn update_all_market_values(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<MarketValueUpdateResult>>, String> {
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
    let current_season: u32 = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map(|row| row.get::<i64, _>("current_season") as u32)
        .unwrap_or(1);

    // 获取所有选手
    let players = match PlayerRepository::get_all_active(&pool, &save_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get players: {}", e))),
    };

    // 预加载所有队伍的赛区信息
    let team_regions: std::collections::HashMap<u64, String> = sqlx::query(
        r#"
        SELECT t.id, r.short_name FROM teams t
        JOIN regions r ON t.region_id = r.id
        WHERE t.save_id = ?
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| (row.get::<i64, _>("id") as u64, row.get::<String, _>("short_name")))
    .collect();

    let mut results = Vec::new();

    for mut player in players {
        let old_value = player.market_value;

        // 获取赛区代码
        let region_code = player.team_id
            .and_then(|tid| team_regions.get(&tid).cloned())
            .unwrap_or_else(|| "LPL".to_string());

        // 获取选手荣誉记录
        let honors = get_player_honors(&pool, &save_id, player.id).await;

        // 计算荣誉系数
        let honor_factor = MarketValueEngine::calculate_honor_factor(&honors, current_season);

        // 计算新身价
        let new_value = player.calculate_full_market_value(&region_code, honor_factor);

        // 如果身价有变化，更新数据库
        if new_value != old_value {
            player.market_value = new_value;
            if let Err(e) = PlayerRepository::update(&pool, &player).await {
                log::error!("Failed to update player {}: {}", player.id, e);
                continue;
            }

            results.push(MarketValueUpdateResult {
                player_id: player.id,
                player_name: player.game_id,
                old_value,
                new_value,
                honor_factor,
            });
        }
    }

    Ok(CommandResult::ok(results))
}

/// 获取选手荣誉记录（内部函数）
async fn get_player_honors(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    player_id: u64,
) -> Vec<PlayerHonorRecord> {
    // 查询选手的荣誉记录，通过 JOIN tournaments 获取赛事类型
    let rows = sqlx::query(
        r#"
        SELECT h.honor_type, t.tournament_type, s.season_number,
               COALESCE(h.description, t.name) as description
        FROM honors h
        JOIN player_honors ph ON h.id = ph.honor_id
        JOIN seasons s ON h.season_id = s.id
        JOIN tournaments t ON h.tournament_id = t.id
        WHERE h.save_id = ? AND ph.player_id = ?
        "#
    )
    .bind(save_id)
    .bind(player_id as i64)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .filter_map(|row| {
            let honor_type: String = row.get("honor_type");
            let tournament_type: String = row.get("tournament_type");
            let season: u32 = row.get::<i64, _>("season_number") as u32;
            let description: String = row.get("description");

            MarketValueEngine::parse_honor_category(&honor_type, &tournament_type, &description)
                .map(|category| PlayerHonorRecord::new(category, season, &description))
        })
        .collect()
}

// ==================== 特性系统 API ====================

/// 特性信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitInfo {
    pub trait_type: String,
    pub name: String,
    pub description: String,
    pub rarity: u8,
    pub is_negative: bool,
}

impl From<TraitType> for TraitInfo {
    fn from(t: TraitType) -> Self {
        Self {
            trait_type: format!("{:?}", t).to_lowercase(),
            name: t.display_name().to_string(),
            description: t.description().to_string(),
            rarity: t.rarity(),
            is_negative: t.is_negative(),
        }
    }
}

/// 状态因子响应（包含计算后的 condition）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConditionInfo {
    pub player_id: u64,
    pub form_cycle: f64,
    pub momentum: i8,
    pub last_performance: f64,
    pub last_match_won: bool,
    pub games_since_rest: u32,
    /// 计算后的 condition 值
    pub condition: i8,
    /// 年龄对应的 condition 范围
    pub condition_range: (i8, i8),
}

/// 选手完整详情（包含特性和状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerFullDetail {
    pub player: PlayerInfo,
    pub traits: Vec<TraitInfo>,
    pub condition_info: PlayerConditionInfo,
}

/// 获取选手特性列表
#[tauri::command]
pub async fn get_player_traits(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<CommandResult<Vec<TraitInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 查询选手特性
    let rows = sqlx::query(
        r#"
        SELECT trait_type FROM player_traits WHERE player_id = ?
        "#
    )
    .bind(player_id as i64)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let traits: Vec<TraitInfo> = rows.into_iter()
        .filter_map(|row| {
            let trait_str: String = row.get("trait_type");
            parse_trait_type(&trait_str).map(|t| t.into())
        })
        .collect();

    Ok(CommandResult::ok(traits))
}

/// 获取选手状态因子和计算后的 condition
#[tauri::command]
pub async fn get_player_condition(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<CommandResult<PlayerConditionInfo>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取选手基本信息（需要 age 和 ability 来计算 condition）
    let player = match PlayerRepository::get_by_id(&pool, player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get player: {}", e))),
    };

    // 查询状态因子
    let factors = get_or_create_form_factors(&pool, player_id).await;

    // 计算 condition
    let condition = ConditionEngine::calculate_condition(
        player.age,
        player.ability,
        &factors,
        None, // 非比赛情境
    );

    let condition_range = ConditionEngine::get_condition_range_by_age(player.age);

    Ok(CommandResult::ok(PlayerConditionInfo {
        player_id,
        form_cycle: factors.form_cycle,
        momentum: factors.momentum,
        last_performance: factors.last_performance,
        last_match_won: factors.last_match_won,
        games_since_rest: factors.games_since_rest,
        condition,
        condition_range,
    }))
}

/// 获取选手完整详情（包含特性和状态）
#[tauri::command]
pub async fn get_player_full_detail(
    state: State<'_, AppState>,
    player_id: u64,
) -> Result<CommandResult<PlayerFullDetail>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取选手基本信息
    let player = match PlayerRepository::get_by_id(&pool, player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get player: {}", e))),
    };

    // 获取特性
    let trait_rows = sqlx::query(
        r#"SELECT trait_type FROM player_traits WHERE player_id = ?"#
    )
    .bind(player_id as i64)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let traits: Vec<TraitInfo> = trait_rows.into_iter()
        .filter_map(|row| {
            let trait_str: String = row.get("trait_type");
            parse_trait_type(&trait_str).map(|t| t.into())
        })
        .collect();

    // 获取状态因子
    let factors = get_or_create_form_factors(&pool, player_id).await;

    // 计算 condition
    let condition = ConditionEngine::calculate_condition(
        player.age,
        player.ability,
        &factors,
        None,
    );

    let condition_range = ConditionEngine::get_condition_range_by_age(player.age);

    Ok(CommandResult::ok(PlayerFullDetail {
        player: player.into(),
        traits,
        condition_info: PlayerConditionInfo {
            player_id,
            form_cycle: factors.form_cycle,
            momentum: factors.momentum,
            last_performance: factors.last_performance,
            last_match_won: factors.last_match_won,
            games_since_rest: factors.games_since_rest,
            condition,
            condition_range,
        },
    }))
}

/// 解析特性类型字符串
fn parse_trait_type(s: &str) -> Option<TraitType> {
    match s.to_lowercase().as_str() {
        "clutch" => Some(TraitType::Clutch),
        "slowstarter" | "slow_starter" => Some(TraitType::SlowStarter),
        "faststarter" | "fast_starter" => Some(TraitType::FastStarter),
        "explosive" => Some(TraitType::Explosive),
        "consistent" => Some(TraitType::Consistent),
        "comebackking" | "comeback_king" => Some(TraitType::ComebackKing),
        "tilter" => Some(TraitType::Tilter),
        "mentalfortress" | "mental_fortress" => Some(TraitType::MentalFortress),
        "fragile" => Some(TraitType::Fragile),
        "ironman" => Some(TraitType::Ironman),
        "volatile" => Some(TraitType::Volatile),
        "risingstar" | "rising_star" => Some(TraitType::RisingStar),
        "veteran" => Some(TraitType::Veteran),
        "teamleader" | "team_leader" => Some(TraitType::TeamLeader),
        _ => None,
    }
}

/// 选手属性更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlayerRequest {
    pub player_id: u64,
    pub ability: Option<u8>,
    pub potential: Option<u8>,
    pub stability: Option<u8>,
    pub age: Option<u8>,
}

/// 更新选手属性（能力值、潜力值、稳定性、年龄）
#[tauri::command]
pub async fn update_player(
    state: State<'_, AppState>,
    request: UpdatePlayerRequest,
) -> Result<CommandResult<PlayerInfo>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取选手信息
    let mut player = match PlayerRepository::get_by_id(&pool, request.player_id).await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get player: {}", e))),
    };

    // 更新属性（仅更新提供的字段）
    if let Some(ability) = request.ability {
        player.ability = ability.clamp(1, 100);
    }
    if let Some(potential) = request.potential {
        player.potential = potential.clamp(1, 100);
    }
    if let Some(stability) = request.stability {
        player.stability = stability.clamp(1, 100);
    }
    if let Some(age) = request.age {
        player.age = age.clamp(16, 45);
    }

    // 保存到数据库
    if let Err(e) = PlayerRepository::update(&pool, &player).await {
        return Ok(CommandResult::err(format!("Failed to update player: {}", e)));
    }

    log::debug!("✅ 选手 {} 属性已更新: ability={}, potential={}, stability={}, age={}",
        player.game_id, player.ability, player.potential, player.stability, player.age);

    Ok(CommandResult::ok(player.into()))
}

/// 获取或创建选手状态因子
async fn get_or_create_form_factors(pool: &sqlx::SqlitePool, player_id: u64) -> PlayerFormFactors {
    let row = sqlx::query(
        r#"
        SELECT form_cycle, momentum, last_performance, last_match_won, games_since_rest
        FROM player_form_factors WHERE player_id = ?
        "#
    )
    .bind(player_id as i64)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match row {
        Some(row) => PlayerFormFactors {
            player_id,
            form_cycle: row.get("form_cycle"),
            momentum: row.get::<i32, _>("momentum") as i8,
            last_performance: row.get("last_performance"),
            last_match_won: row.get::<i32, _>("last_match_won") != 0,
            games_since_rest: row.get::<i32, _>("games_since_rest") as u32,
        },
        None => {
            // 创建默认状态因子
            let factors = ConditionEngine::reset_form_factors(player_id);

            // 插入数据库
            let _ = sqlx::query(
                r#"
                INSERT INTO player_form_factors (player_id, form_cycle, momentum, last_performance, last_match_won, games_since_rest)
                VALUES (?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(player_id as i64)
            .bind(factors.form_cycle)
            .bind(factors.momentum as i32)
            .bind(factors.last_performance)
            .bind(if factors.last_match_won { 1 } else { 0 })
            .bind(factors.games_since_rest as i32)
            .execute(pool)
            .await;

            factors
        }
    }
}
