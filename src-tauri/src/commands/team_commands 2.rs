use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{PlayerRepository, TeamRepository};
use crate::models::{Player, Team};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Serialize, Deserialize)]
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
    pub contract_end_season: Option<u32>,
    pub is_starter: bool,
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
            contract_end_season: p.contract_end_season,
            is_starter: p.is_starter,
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

    let infos: Vec<TeamInfo> = teams.into_iter().map(|t| t.into()).collect();
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

    let infos: Vec<TeamInfo> = teams.into_iter().map(|t| t.into()).collect();
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

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let team = match TeamRepository::get_by_id(&pool, team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get team: {}", e))),
    };

    Ok(CommandResult::ok(team.into()))
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

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取队伍信息
    let team = match TeamRepository::get_by_id(&pool, team_id).await {
        Ok(t) => t,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get team: {}", e))),
    };

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
        team: team.into(),
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
