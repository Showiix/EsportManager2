use crate::commands::save_commands::{AppState, CommandResult};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 赛区信息
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct RegionInfo {
    pub id: u64,
    pub code: String,
    pub name: String,
    pub full_name: String,
    pub team_count: u32,
    pub is_major: bool,
}

/// 赛区详情（包含队伍列表）
#[derive(Debug, Serialize, Deserialize)]

pub struct RegionDetail {
    pub region: RegionInfo,
    pub teams: Vec<TeamFullInfo>,
    pub current_tournament: Option<TournamentInfoSimple>,
}

/// 队伍完整信息
#[derive(Debug, Serialize, Deserialize)]

pub struct TeamFullInfo {
    pub id: u64,
    pub region_id: u64,
    pub name: String,
    pub short_name: Option<String>,
    pub power_rating: f64,
    pub total_matches: u32,
    pub wins: u32,
    pub win_rate: f64,
    pub annual_points: i32,
    pub cross_year_points: i32,
    pub balance: i64,
}

/// 赛事简单信息
#[derive(Debug, Serialize, Deserialize)]

pub struct TournamentInfoSimple {
    pub id: u64,
    pub name: String,
    pub tournament_type: String,
    pub status: String,
}

/// 赛事信息
#[derive(Debug, Serialize, Deserialize)]

pub struct TournamentInfo {
    pub id: u64,
    pub name: String,
    pub tournament_type: String,
    pub season_id: u64,
    pub region_id: Option<u64>,
    pub status: String,
    pub match_count: u32,
    pub completed_matches: u32,
}

/// 赛事详细信息
#[derive(Debug, Serialize, Deserialize)]

pub struct TournamentDetailInfo {
    pub id: u64,
    pub name: String,
    pub tournament_type: String,
    pub season_id: u64,
    pub region_id: Option<u64>,
    pub status: String,
    pub participating_teams: Vec<TeamBriefInfo>,
    pub stages: Vec<QueryStageInfo>,
}

/// 队伍简要信息
#[derive(Debug, Serialize, Deserialize)]

pub struct TeamBriefInfo {
    pub id: u64,
    pub name: String,
    pub short_name: Option<String>,
    pub power_rating: f64,
}

/// 赛事阶段信息
#[derive(Debug, Serialize, Deserialize)]

pub struct QueryStageInfo {
    pub name: String,
    pub total_matches: u32,
    pub completed_matches: u32,
}

/// 赛季信息
#[derive(Debug, Serialize, Deserialize)]

pub struct SeasonInfo {
    pub season_id: u64,
    pub current_phase: String,
    pub phase_name: String,
    pub tournaments: Vec<TournamentBriefInfo>,
}

/// 赛事简要信息
#[derive(Debug, Serialize, Deserialize)]

pub struct TournamentBriefInfo {
    pub id: u64,
    pub name: String,
    pub tournament_type: String,
    pub status: String,
}

/// 获取所有赛区
#[tauri::command]
pub async fn get_all_regions(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<RegionInfo>>, String> {
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

    let rows = sqlx::query(
        r#"
        SELECT r.id, r.name, r.short_name, r.team_count
        FROM regions r
        WHERE r.save_id = ?
        ORDER BY r.id ASC
        "#,
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let regions: Vec<RegionInfo> = rows
        .iter()
        .map(|row| RegionInfo {
            id: row.get::<i64, _>("id") as u64,
            code: row.get("name"),  // 使用 name (如 "LPL") 作为 code
            name: row.get("name"),
            full_name: row.get("name"),
            team_count: row.get::<i64, _>("team_count") as u32,
            is_major: true,
        })
        .collect();

    Ok(CommandResult::ok(regions))
}

/// 获取赛区详情
#[tauri::command]
pub async fn get_region_detail(
    state: State<'_, AppState>,
    region_id: u64,
) -> Result<CommandResult<RegionDetail>, String> {
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

    // 获取赛区信息
    let region_row = sqlx::query(
        r#"
        SELECT r.id, r.name, r.short_name, r.team_count
        FROM regions r
        WHERE r.save_id = ? AND r.id = ?
        "#,
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let region_row = match region_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Region not found")),
    };

    let region = RegionInfo {
        id: region_row.get::<i64, _>("id") as u64,
        code: region_row.get("name"),  // 使用 name (如 "LPL") 作为 code
        name: region_row.get("name"),
        full_name: region_row.get("name"),
        team_count: region_row.get::<i64, _>("team_count") as u32,
        is_major: true,
    };

    // 获取该赛区的队伍
    let team_rows = sqlx::query(
        r#"
        SELECT id, region_id, name, short_name, power_rating,
               total_matches, wins, win_rate, annual_points, cross_year_points, balance
        FROM teams
        WHERE save_id = ? AND region_id = ?
        ORDER BY power_rating DESC
        "#,
    )
    .bind(&save_id)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let teams: Vec<TeamFullInfo> = team_rows
        .iter()
        .map(|row| TeamFullInfo {
            id: row.get::<i64, _>("id") as u64,
            region_id: row.get::<i64, _>("region_id") as u64,
            name: row.get("name"),
            short_name: row.get("short_name"),
            power_rating: row.get("power_rating"),
            total_matches: row.get::<i64, _>("total_matches") as u32,
            wins: row.get::<i64, _>("wins") as u32,
            win_rate: row.get("win_rate"),
            annual_points: row.get::<i64, _>("annual_points") as i32,
            cross_year_points: row.get::<i64, _>("cross_year_points") as i32,
            balance: row.get("balance"),
        })
        .collect();

    // 暂时不获取当前赛事，返回 None
    let current_tournament: Option<TournamentInfoSimple> = None;

    Ok(CommandResult::ok(RegionDetail {
        region,
        teams,
        current_tournament,
    }))
}

/// 获取赛季所有赛事
#[tauri::command]
pub async fn get_season_tournaments(
    state: State<'_, AppState>,
    season_id: Option<u64>,
) -> Result<CommandResult<Vec<TournamentInfo>>, String> {
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

    // 获取目标赛季
    let target_season = if let Some(sid) = season_id {
        sid as i64
    } else {
        let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
            .bind(&save_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
        save_row.get("current_season")
    };

    let rows = sqlx::query(
        r#"
        SELECT t.id, t.name, t.tournament_type, t.season_id, t.region_id, t.status,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id) as match_count,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id AND (m.status = 'Completed' OR m.status = 'COMPLETED')) as completed_matches
        FROM tournaments t
        WHERE t.save_id = ? AND t.season_id = ?
        ORDER BY t.id ASC
        "#,
    )
    .bind(&save_id)
    .bind(target_season)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournaments: Vec<TournamentInfo> = rows
        .iter()
        .map(|row| TournamentInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            tournament_type: row.get("tournament_type"),
            season_id: row.get::<i64, _>("season_id") as u64,
            region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
            status: row.get("status"),
            match_count: row.get::<i64, _>("match_count") as u32,
            completed_matches: row.get::<i64, _>("completed_matches") as u32,
        })
        .collect();

    Ok(CommandResult::ok(tournaments))
}

/// 获取赛区赛事
#[tauri::command]
pub async fn get_region_tournaments(
    state: State<'_, AppState>,
    region_id: u64,
    season_id: Option<u64>,
) -> Result<CommandResult<Vec<TournamentInfo>>, String> {
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

    let target_season = if let Some(sid) = season_id {
        sid as i64
    } else {
        let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
            .bind(&save_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
        save_row.get("current_season")
    };

    let rows = sqlx::query(
        r#"
        SELECT t.id, t.name, t.tournament_type, t.season_id, t.region_id, t.status,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id) as match_count,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id AND (m.status = 'Completed' OR m.status = 'COMPLETED')) as completed_matches
        FROM tournaments t
        WHERE t.save_id = ? AND t.season_id = ? AND t.region_id = ?
        ORDER BY t.id ASC
        "#,
    )
    .bind(&save_id)
    .bind(target_season)
    .bind(region_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournaments: Vec<TournamentInfo> = rows
        .iter()
        .map(|row| TournamentInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            tournament_type: row.get("tournament_type"),
            season_id: row.get::<i64, _>("season_id") as u64,
            region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
            status: row.get("status"),
            match_count: row.get::<i64, _>("match_count") as u32,
            completed_matches: row.get::<i64, _>("completed_matches") as u32,
        })
        .collect();

    log::debug!("region_id={}, season_id={:?}, found {} tournaments", region_id, season_id, tournaments.len());
    for t in &tournaments {
        log::debug!("- {} (type={}, matches={}/{})", t.name, t.tournament_type, t.completed_matches, t.match_count);
    }

    Ok(CommandResult::ok(tournaments))
}

/// 获取赛事详情
#[tauri::command]
pub async fn get_tournament_detail(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<TournamentDetailInfo>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取赛事基本信息
    let tournament_row = sqlx::query(
        "SELECT id, name, tournament_type, season_id, region_id, status FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournament_row = match tournament_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Tournament not found")),
    };

    // 获取参赛队伍
    let team_rows = sqlx::query(
        r#"
        SELECT DISTINCT t.id, t.name, t.short_name, t.power_rating
        FROM teams t
        JOIN matches m ON (m.home_team_id = t.id OR m.away_team_id = t.id)
        WHERE m.tournament_id = ?
        ORDER BY t.power_rating DESC
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let teams: Vec<TeamBriefInfo> = team_rows
        .iter()
        .map(|row| TeamBriefInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            short_name: row.get("short_name"),
            power_rating: row.get("power_rating"),
        })
        .collect();

    // 获取阶段信息
    let stage_rows = sqlx::query(
        r#"
        SELECT stage,
               COUNT(*) as total_matches,
               SUM(CASE WHEN status = 'Completed' OR status = 'COMPLETED' THEN 1 ELSE 0 END) as completed_matches
        FROM matches
        WHERE tournament_id = ?
        GROUP BY stage
        ORDER BY MIN(id)
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let stages: Vec<QueryStageInfo> = stage_rows
        .iter()
        .map(|row| QueryStageInfo {
            name: row.get("stage"),
            total_matches: row.get::<i64, _>("total_matches") as u32,
            completed_matches: row.get::<i64, _>("completed_matches") as u32,
        })
        .collect();

    Ok(CommandResult::ok(TournamentDetailInfo {
        id: tournament_row.get::<i64, _>("id") as u64,
        name: tournament_row.get("name"),
        tournament_type: tournament_row.get("tournament_type"),
        season_id: tournament_row.get::<i64, _>("season_id") as u64,
        region_id: tournament_row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
        status: tournament_row.get("status"),
        participating_teams: teams,
        stages,
    }))
}

/// 获取国际赛事列表
#[tauri::command]
pub async fn get_international_tournaments(
    state: State<'_, AppState>,
    season_id: Option<u64>,
) -> Result<CommandResult<Vec<TournamentInfo>>, String> {
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

    let target_season = if let Some(sid) = season_id {
        sid as i64
    } else {
        let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
            .bind(&save_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
        save_row.get("current_season")
    };

    // 国际赛事类型
    let international_types = vec![
        "Msi", "WorldChampionship", "MadridMasters",
        "ClaudeIntercontinental", "ShanghaiMasters",
        "IcpIntercontinental", "SuperIntercontinental"
    ];

    let placeholders: String = international_types.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query = format!(
        r#"
        SELECT t.id, t.name, t.tournament_type, t.season_id, t.region_id, t.status,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id) as match_count,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id AND (m.status = 'Completed' OR m.status = 'COMPLETED')) as completed_matches
        FROM tournaments t
        WHERE t.save_id = ? AND t.season_id = ? AND t.tournament_type IN ({})
        ORDER BY t.id ASC
        "#,
        placeholders
    );

    let mut query_builder = sqlx::query(&query)
        .bind(&save_id)
        .bind(target_season);

    for tt in &international_types {
        query_builder = query_builder.bind(*tt);
    }

    let rows = query_builder
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let tournaments: Vec<TournamentInfo> = rows
        .iter()
        .map(|row| TournamentInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            tournament_type: row.get("tournament_type"),
            season_id: row.get::<i64, _>("season_id") as u64,
            region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
            status: row.get("status"),
            match_count: row.get::<i64, _>("match_count") as u32,
            completed_matches: row.get::<i64, _>("completed_matches") as u32,
        })
        .collect();

    Ok(CommandResult::ok(tournaments))
}

/// 根据类型获取赛事列表
#[tauri::command]
pub async fn get_tournaments_by_type(
    state: State<'_, AppState>,
    tournament_type: String,
    season_id: i64,
) -> Result<CommandResult<Vec<TournamentInfo>>, String> {
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

    // 支持简写类型名映射到完整类型名
    let full_type = match tournament_type.as_str() {
        "Clauch" | "Claude" => "ClaudeIntercontinental",
        "Madrid" => "MadridMasters",
        "Shanghai" => "ShanghaiMasters",
        "Icp" | "ICP" => "IcpIntercontinental",
        "Super" | "SuperCup" => "SuperIntercontinental",
        "Worlds" => "WorldChampionship",
        "MSI" | "Msi" => "Msi",
        other => other,
    };

    let rows = sqlx::query(
        r#"
        SELECT t.id, t.name, t.tournament_type, t.season_id, t.region_id, t.status,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id) as match_count,
               (SELECT COUNT(*) FROM matches m WHERE m.tournament_id = t.id AND (m.status = 'Completed' OR m.status = 'COMPLETED')) as completed_matches
        FROM tournaments t
        WHERE t.save_id = ? AND t.season_id = ? AND t.tournament_type = ?
        ORDER BY t.id ASC
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .bind(full_type)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournaments: Vec<TournamentInfo> = rows
        .iter()
        .map(|row| TournamentInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            tournament_type: row.get("tournament_type"),
            season_id: row.get::<i64, _>("season_id") as u64,
            region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
            status: row.get("status"),
            match_count: row.get::<i64, _>("match_count") as u32,
            completed_matches: row.get::<i64, _>("completed_matches") as u32,
        })
        .collect();

    Ok(CommandResult::ok(tournaments))
}

/// 获取赛季概览
#[tauri::command]
pub async fn get_season_overview(
    state: State<'_, AppState>,
    season_id: Option<u64>,
) -> Result<CommandResult<SeasonInfo>, String> {
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

    // 获取存档信息
    let save_row = sqlx::query("SELECT current_season, current_phase FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let current_season: i64 = save_row.get("current_season");
    let current_phase: String = save_row.get("current_phase");

    let target_season = season_id.map(|s| s as i64).unwrap_or(current_season);

    // 获取赛季赛事
    let tournament_rows = sqlx::query(
        "SELECT id, name, tournament_type, status FROM tournaments WHERE save_id = ? AND season_id = ? ORDER BY id"
    )
    .bind(&save_id)
    .bind(target_season)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournaments: Vec<TournamentBriefInfo> = tournament_rows
        .iter()
        .map(|row| TournamentBriefInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            tournament_type: row.get("tournament_type"),
            status: row.get("status"),
        })
        .collect();

    let phase_name = get_phase_display_name(&current_phase);

    Ok(CommandResult::ok(SeasonInfo {
        season_id: target_season as u64,
        current_phase,
        phase_name,
        tournaments,
    }))
}

/// 搜索队伍
#[tauri::command]
pub async fn search_teams(
    state: State<'_, AppState>,
    keyword: String,
    region_id: Option<u64>,
) -> Result<CommandResult<Vec<TeamBriefInfo>>, String> {
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

    let search_pattern = format!("%{}%", keyword);

    let rows = if let Some(rid) = region_id {
        sqlx::query(
            r#"
            SELECT id, name, short_name, power_rating
            FROM teams
            WHERE save_id = ? AND region_id = ? AND (name LIKE ? OR short_name LIKE ?)
            ORDER BY power_rating DESC
            LIMIT 20
            "#,
        )
        .bind(&save_id)
        .bind(rid as i64)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query(
            r#"
            SELECT id, name, short_name, power_rating
            FROM teams
            WHERE save_id = ? AND (name LIKE ? OR short_name LIKE ?)
            ORDER BY power_rating DESC
            LIMIT 20
            "#,
        )
        .bind(&save_id)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    };

    let teams: Vec<TeamBriefInfo> = rows
        .iter()
        .map(|row| TeamBriefInfo {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            short_name: row.get("short_name"),
            power_rating: row.get("power_rating"),
        })
        .collect();

    Ok(CommandResult::ok(teams))
}

/// 搜索球员
#[tauri::command]
pub async fn search_players(
    state: State<'_, AppState>,
    keyword: String,
    position: Option<String>,
    team_id: Option<u64>,
) -> Result<CommandResult<Vec<PlayerSearchResult>>, String> {
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

    let search_pattern = format!("%{}%", keyword);

    // 构建动态查询
    let mut conditions = vec!["save_id = ?", "(game_id LIKE ? OR real_name LIKE ?)"];
    let mut bindings: Vec<String> = vec![save_id.clone(), search_pattern.clone(), search_pattern.clone()];

    if let Some(ref pos) = position {
        conditions.push("position = ?");
        bindings.push(pos.clone());
    }

    if let Some(tid) = team_id {
        conditions.push("team_id = ?");
        bindings.push(tid.to_string());
    }

    let _query = format!(
        r#"
        SELECT p.id, p.game_id, p.real_name, p.position, p.ability, p.potential, p.age,
               p.team_id, t.name as team_name
        FROM players p
        LEFT JOIN teams t ON p.team_id = t.id
        WHERE {}
        ORDER BY p.ability DESC
        LIMIT 30
        "#,
        conditions.join(" AND ")
    );

    // 由于SQLx不支持动态绑定数量，我们使用固定查询
    let rows = sqlx::query(
        r#"
        SELECT p.id, p.game_id, p.real_name, p.position, p.ability, p.potential, p.age,
               p.team_id, t.name as team_name
        FROM players p
        LEFT JOIN teams t ON p.team_id = t.id
        WHERE p.save_id = ? AND (p.game_id LIKE ? OR p.real_name LIKE ?)
        ORDER BY p.ability DESC
        LIMIT 30
        "#,
    )
    .bind(&save_id)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let players: Vec<PlayerSearchResult> = rows
        .iter()
        .filter(|row| {
            // 过滤position
            if let Some(ref pos) = position {
                let player_pos: Option<String> = row.get("position");
                if player_pos.as_ref() != Some(pos) {
                    return false;
                }
            }
            // 过滤team_id
            if let Some(tid) = team_id {
                let player_team: Option<i64> = row.get("team_id");
                if player_team != Some(tid as i64) {
                    return false;
                }
            }
            true
        })
        .map(|row| PlayerSearchResult {
            id: row.get::<i64, _>("id") as u64,
            game_id: row.get("game_id"),
            real_name: row.get("real_name"),
            position: row.get("position"),
            ability: row.get::<i64, _>("ability") as u8,
            potential: row.get::<i64, _>("potential") as u8,
            age: row.get::<i64, _>("age") as u8,
            team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
            team_name: row.get("team_name"),
        })
        .collect();

    Ok(CommandResult::ok(players))
}

/// 球员搜索结果
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSearchResult {
    pub id: u64,
    pub game_id: String,
    pub real_name: Option<String>,
    pub position: Option<String>,
    pub ability: u8,
    pub potential: u8,
    pub age: u8,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
}

/// 获取阶段显示名称
fn get_phase_display_name(phase: &str) -> String {
    match phase {
        "SpringRegular" => "春季常规赛".to_string(),
        "SpringPlayoffs" => "春季季后赛".to_string(),
        "Msi" => "MSI季中赛".to_string(),
        "MadridMasters" => "马德里大师赛".to_string(),
        "SummerRegular" => "夏季常规赛".to_string(),
        "SummerPlayoffs" => "夏季季后赛".to_string(),
        "ClaudeIntercontinental" => "Claude洲际赛".to_string(),
        "WorldChampionship" => "世界赛".to_string(),
        "ShanghaiMasters" => "上海大师赛".to_string(),
        "IcpIntercontinental" => "ICP洲际对抗赛".to_string(),
        "SuperIntercontinental" => "Super洲际邀请赛".to_string(),
        "TransferWindow" => "转会期".to_string(),
        "Draft" => "选秀大会".to_string(),
        "SeasonEnd" => "赛季结算".to_string(),
        _ => phase.to_string(),
    }
}
