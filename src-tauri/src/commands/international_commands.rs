use crate::commands::save_commands::{AppState, CommandResult};
use crate::models::Team;
use crate::db::{MatchRepository, PointsRepository, TeamRepository};
use crate::services::TournamentService;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 参赛队伍信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantTeam {
    pub team_id: u64,
    pub team_name: String,
    pub region_id: u64,
    pub region_name: String,
    pub seed: u32,
    pub qualification_type: String,
}

/// 赛事对阵信息
#[derive(Debug, Serialize, Deserialize)]
pub struct BracketInfo {
    pub tournament_id: u64,
    pub tournament_name: String,
    pub tournament_type: String,
    pub stages: Vec<StageInfo>,
    pub matches: Vec<MatchBracketInfo>,
}

/// 阶段信息
#[derive(Debug, Serialize, Deserialize)]
pub struct StageInfo {
    pub name: String,
    pub display_name: String,
    pub order: u32,
    pub total_matches: u32,
    pub completed_matches: u32,
}

/// 对阵详情
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchBracketInfo {
    pub match_id: u64,
    pub stage: String,
    pub match_order: u32,
    pub format: String,
    pub home_team: Option<TeamBracketInfo>,
    pub away_team: Option<TeamBracketInfo>,
    pub home_score: u32,
    pub away_score: u32,
    pub winner_id: Option<u64>,
    pub status: String,
}

/// 队伍对阵信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamBracketInfo {
    pub id: u64,
    pub name: String,
    pub short_name: Option<String>,
    pub region_code: String,
}

/// 瑞士轮状态
#[derive(Debug, Serialize, Deserialize)]
pub struct SwissRoundStatus {
    pub current_round: u32,
    pub teams: Vec<SwissTeamStatus>,
    pub completed: bool,
    pub qualified_teams: Vec<u64>,
    pub eliminated_teams: Vec<u64>,
}

/// 瑞士轮队伍状态
#[derive(Debug, Serialize, Deserialize)]
pub struct SwissTeamStatus {
    pub team_id: u64,
    pub team_name: String,
    pub wins: u32,
    pub losses: u32,
    pub is_qualified: bool,
    pub is_eliminated: bool,
}

/// 创建MSI赛事
#[tauri::command]
pub async fn create_msi_tournament(
    state: State<'_, AppState>,
    legendary_team_ids: Vec<u64>,   // 4支冠军
    challenger_team_ids: Vec<u64>,  // 4支亚军
    qualifier_team_ids: Vec<u64>,   // 4支季军
) -> Result<CommandResult<u64>, String> {
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

    // 创建赛事
    let tournament_id: i64 = sqlx::query(
        r#"
        INSERT INTO tournaments (save_id, name, tournament_type, season_id, region_id, status)
        VALUES (?, ?, 'Msi', ?, NULL, 'InProgress')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(format!("MSI {}", current_season))
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    // 获取队伍信息
    let legendary_teams = get_teams_by_ids(&pool, &legendary_team_ids).await?;
    let challenger_teams = get_teams_by_ids(&pool, &challenger_team_ids).await?;
    let qualifier_teams = get_teams_by_ids(&pool, &qualifier_team_ids).await?;

    // 生成对阵
    let service = TournamentService::new();
    let matches = service.generate_msi_bracket(
        tournament_id as u64,
        &legendary_teams,
        &challenger_teams,
        &qualifier_teams,
    );

    // 保存比赛
    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches (
                tournament_id, stage, round, match_order, format,
                home_team_id, away_team_id, home_score, away_score, winner_id, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL, 'Scheduled')
            "#,
        )
        .bind(tournament_id)
        .bind(&m.stage)
        .bind(m.round)
        .bind(m.match_order)
        .bind(format!("{:?}", m.format))
        .bind(if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None })
        .bind(if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None })
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(tournament_id as u64))
}

/// 创建世界赛
#[tauri::command]
pub async fn create_worlds_tournament(
    state: State<'_, AppState>,
    direct_team_ids: Vec<u64>,   // 4支直接晋级队
    group_team_ids: Vec<u64>,    // 8支小组赛队
) -> Result<CommandResult<u64>, String> {
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

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let tournament_id: i64 = sqlx::query(
        r#"
        INSERT INTO tournaments (save_id, name, tournament_type, season_id, region_id, status)
        VALUES (?, ?, 'WorldChampionship', ?, NULL, 'InProgress')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(format!("World Championship {}", current_season))
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    let direct_teams = get_teams_by_ids(&pool, &direct_team_ids).await?;
    let group_teams = get_teams_by_ids(&pool, &group_team_ids).await?;

    let service = TournamentService::new();
    let matches = service.generate_worlds_bracket(
        tournament_id as u64,
        &direct_teams,
        &group_teams,
    );

    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches (
                tournament_id, stage, round, match_order, format,
                home_team_id, away_team_id, home_score, away_score, winner_id, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL, 'Scheduled')
            "#,
        )
        .bind(tournament_id)
        .bind(&m.stage)
        .bind(m.round)
        .bind(m.match_order)
        .bind(format!("{:?}", m.format))
        .bind(if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None })
        .bind(if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None })
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(tournament_id as u64))
}

/// 创建大师赛 (马德里/Claude洲际)
#[tauri::command]
pub async fn create_masters_tournament(
    state: State<'_, AppState>,
    tournament_type: String, // "MadridMasters" or "ClaudeIntercontinental"
    team_ids: Vec<u64>,      // 32队
) -> Result<CommandResult<u64>, String> {
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

    if team_ids.len() != 32 {
        return Ok(CommandResult::err("Masters tournament requires exactly 32 teams"));
    }

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let name = match tournament_type.as_str() {
        "MadridMasters" => format!("Madrid Masters {}", current_season),
        "ClaudeIntercontinental" => format!("Claude Intercontinental {}", current_season),
        _ => return Ok(CommandResult::err("Invalid tournament type")),
    };

    let tournament_id: i64 = sqlx::query(
        r#"
        INSERT INTO tournaments (save_id, name, tournament_type, season_id, region_id, status)
        VALUES (?, ?, ?, ?, NULL, 'InProgress')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(&name)
    .bind(&tournament_type)
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    let teams = get_teams_by_ids(&pool, &team_ids).await?;

    let service = TournamentService::new();
    let matches = service.generate_masters_bracket(tournament_id as u64, &teams);

    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches (
                tournament_id, stage, round, match_order, format,
                home_team_id, away_team_id, home_score, away_score, winner_id, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL, 'Scheduled')
            "#,
        )
        .bind(tournament_id)
        .bind(&m.stage)
        .bind(m.round)
        .bind(m.match_order)
        .bind(format!("{:?}", m.format))
        .bind(if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None })
        .bind(if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None })
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(tournament_id as u64))
}

/// 创建Super洲际赛
#[tauri::command]
pub async fn create_super_tournament(
    state: State<'_, AppState>,
    legendary_team_ids: Vec<u64>,   // 前4名
    challenger_team_ids: Vec<u64>,  // 5-8名
    fighter_team_ids: Vec<u64>,     // 9-16名
) -> Result<CommandResult<u64>, String> {
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

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let tournament_id: i64 = sqlx::query(
        r#"
        INSERT INTO tournaments (save_id, name, tournament_type, season_id, region_id, status)
        VALUES (?, ?, 'SuperIntercontinental', ?, NULL, 'InProgress')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(format!("Super Intercontinental {}", current_season))
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    let legendary_teams = get_teams_by_ids(&pool, &legendary_team_ids).await?;
    let challenger_teams = get_teams_by_ids(&pool, &challenger_team_ids).await?;
    let fighter_teams = get_teams_by_ids(&pool, &fighter_team_ids).await?;

    let service = TournamentService::new();
    let matches = service.generate_super_bracket(
        tournament_id as u64,
        &legendary_teams,
        &challenger_teams,
        &fighter_teams,
    );

    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches (
                tournament_id, stage, round, match_order, format,
                home_team_id, away_team_id, home_score, away_score, winner_id, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL, 'Scheduled')
            "#,
        )
        .bind(tournament_id)
        .bind(&m.stage)
        .bind(m.round)
        .bind(m.match_order)
        .bind(format!("{:?}", m.format))
        .bind(if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None })
        .bind(if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None })
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(tournament_id as u64))
}

/// 获取赛事对阵图
#[tauri::command]
pub async fn get_tournament_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<BracketInfo>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取赛事信息
    let tournament_row = sqlx::query(
        "SELECT id, name, tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournament_row = match tournament_row {
        Some(r) => r,
        None => return Ok(CommandResult::err("Tournament not found")),
    };

    // 获取阶段统计（兼容 'COMPLETED' 和 'Completed' 两种状态格式）
    let stage_rows = sqlx::query(
        r#"
        SELECT stage,
               COUNT(*) as total_matches,
               SUM(CASE WHEN UPPER(status) = 'COMPLETED' THEN 1 ELSE 0 END) as completed_matches,
               MIN(id) as min_id
        FROM matches
        WHERE tournament_id = ?
        GROUP BY stage
        ORDER BY min_id
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut stages = Vec::new();
    for (idx, row) in stage_rows.iter().enumerate() {
        let stage_name: String = row.get("stage");
        stages.push(StageInfo {
            name: stage_name.clone(),
            display_name: get_stage_display_name(&stage_name),
            order: idx as u32,
            total_matches: row.get::<i64, _>("total_matches") as u32,
            completed_matches: row.get::<i64, _>("completed_matches") as u32,
        });
    }

    // 获取比赛详情
    let match_rows = sqlx::query(
        r#"
        SELECT m.id, m.stage, m.match_order, m.format, m.home_team_id, m.away_team_id,
               m.home_score, m.away_score, m.winner_id, m.status,
               ht.name as home_name, ht.short_name as home_short,
               at.name as away_name, at.short_name as away_short,
               hr.name as home_region, ar.name as away_region
        FROM matches m
        LEFT JOIN teams ht ON m.home_team_id = ht.id
        LEFT JOIN teams at ON m.away_team_id = at.id
        LEFT JOIN regions hr ON ht.region_id = hr.id
        LEFT JOIN regions ar ON at.region_id = ar.id
        WHERE m.tournament_id = ?
        ORDER BY m.id
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let matches: Vec<MatchBracketInfo> = match_rows
        .iter()
        .map(|row| {
            let home_team_id: Option<i64> = row.get("home_team_id");
            let away_team_id: Option<i64> = row.get("away_team_id");

            MatchBracketInfo {
                match_id: row.get::<i64, _>("id") as u64,
                stage: row.get("stage"),
                match_order: row.get::<Option<i64>, _>("match_order").unwrap_or(0) as u32,
                format: row.get("format"),
                home_team: home_team_id.map(|id| TeamBracketInfo {
                    id: id as u64,
                    name: row.get("home_name"),
                    short_name: row.get("home_short"),
                    region_code: row.get::<Option<String>, _>("home_region").unwrap_or_default(),
                }),
                away_team: away_team_id.map(|id| TeamBracketInfo {
                    id: id as u64,
                    name: row.get("away_name"),
                    short_name: row.get("away_short"),
                    region_code: row.get::<Option<String>, _>("away_region").unwrap_or_default(),
                }),
                home_score: row.get::<i64, _>("home_score") as u32,
                away_score: row.get::<i64, _>("away_score") as u32,
                winner_id: row.get::<Option<i64>, _>("winner_id").map(|v| v as u64),
                status: row.get("status"),
            }
        })
        .collect();

    Ok(CommandResult::ok(BracketInfo {
        tournament_id,
        tournament_name: tournament_row.get("name"),
        tournament_type: tournament_row.get("tournament_type"),
        stages,
        matches,
    }))
}

/// 推进淘汰赛对阵
#[tauri::command]
pub async fn advance_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
    completed_match_id: u64,
    winner_id: u64,
) -> Result<CommandResult<Vec<u64>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取已完成比赛信息（包含双方队伍ID）
    let match_row = sqlx::query(
        "SELECT stage, match_order, home_team_id, away_team_id FROM matches WHERE id = ? AND tournament_id = ?"
    )
    .bind(completed_match_id as i64)
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (stage, match_order, home_team_id, away_team_id): (String, i64, Option<i64>, Option<i64>) = match match_row {
        Some(r) => (
            r.get("stage"),
            r.get::<Option<i64>, _>("match_order").unwrap_or(0),
            r.get("home_team_id"),
            r.get("away_team_id"),
        ),
        None => return Ok(CommandResult::err("Match not found")),
    };

    // 计算败者ID
    let loser_id = if home_team_id == Some(winner_id as i64) {
        away_team_id
    } else {
        home_team_id
    };

    log::debug!("stage={}, match_order={}, winner={}, loser={:?}", stage, match_order, winner_id, loser_id);

    let mut updated_match_ids = Vec::new();

    // 处理胜者
    let next_matches = determine_next_matches(&stage, match_order as u32);
    for (next_stage, next_order, is_home) in next_matches {
        let next_match = sqlx::query(
            "SELECT id FROM matches WHERE tournament_id = ? AND stage = ? AND match_order = ?"
        )
        .bind(tournament_id as i64)
        .bind(&next_stage)
        .bind(next_order as i64)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(nm) = next_match {
            let next_id: i64 = nm.get("id");
            let update_field = if is_home { "home_team_id" } else { "away_team_id" };
            sqlx::query(&format!("UPDATE matches SET {} = ? WHERE id = ?", update_field))
                .bind(winner_id as i64)
                .bind(next_id)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

            log::debug!("Winner {} -> {} (order={}, is_home={})", winner_id, next_stage, next_order, is_home);
            updated_match_ids.push(next_id as u64);
        }
    }

    // 处理败者（如果需要）
    if let Some(loser) = loser_id {
        let loser_next = determine_loser_next_match(&stage, match_order as u32);
        for (next_stage, next_order, is_home) in loser_next {
            let next_match = sqlx::query(
                "SELECT id FROM matches WHERE tournament_id = ? AND stage = ? AND match_order = ?"
            )
            .bind(tournament_id as i64)
            .bind(&next_stage)
            .bind(next_order as i64)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(nm) = next_match {
                let next_id: i64 = nm.get("id");
                let update_field = if is_home { "home_team_id" } else { "away_team_id" };
                sqlx::query(&format!("UPDATE matches SET {} = ? WHERE id = ?", update_field))
                    .bind(loser)
                    .bind(next_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;

                log::debug!("Loser {} -> {} (order={}, is_home={})", loser, next_stage, next_order, is_home);
                updated_match_ids.push(next_id as u64);
            }
        }
    }

    Ok(CommandResult::ok(updated_match_ids))
}

/// 获取瑞士轮状态
#[tauri::command]
pub async fn get_swiss_round_status(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<SwissRoundStatus>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取赛事类型，确定晋级/淘汰所需胜负场数
    let tournament_type: String = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_else(|| "Msi".to_string());

    // 世界赛：2胜晋级/2败淘汰；MSI：3胜晋级/3败淘汰
    let wins_to_qualify: u32 = if tournament_type == "WorldChampionship" { 2 } else { 3 };
    let losses_to_eliminate: u32 = if tournament_type == "WorldChampionship" { 2 } else { 3 };

    // 获取所有瑞士轮比赛
    let swiss_rows = sqlx::query(
        r#"
        SELECT m.id, m.stage, m.home_team_id, m.away_team_id, m.winner_id, m.status,
               ht.name as home_name, at.name as away_name
        FROM matches m
        LEFT JOIN teams ht ON m.home_team_id = ht.id
        LEFT JOIN teams at ON m.away_team_id = at.id
        WHERE m.tournament_id = ? AND m.stage LIKE 'SWISS_%'
        ORDER BY m.stage, m.id
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 统计每队胜负
    let mut team_records: std::collections::HashMap<u64, (String, u32, u32)> = std::collections::HashMap::new();

    for row in &swiss_rows {
        let home_id: Option<i64> = row.get("home_team_id");
        let away_id: Option<i64> = row.get("away_team_id");
        let winner_id: Option<i64> = row.get("winner_id");
        let status: String = row.get("status");

        if let Some(hid) = home_id {
            let home_name: String = row.get("home_name");
            team_records.entry(hid as u64).or_insert((home_name, 0, 0));
        }
        if let Some(aid) = away_id {
            let away_name: String = row.get("away_name");
            team_records.entry(aid as u64).or_insert((away_name, 0, 0));
        }

        if status == "Completed" || status == "COMPLETED" {
            if let (Some(hid), Some(aid), Some(wid)) = (home_id, away_id, winner_id) {
                if wid == hid {
                    team_records.entry(hid as u64).and_modify(|r| r.1 += 1);
                    team_records.entry(aid as u64).and_modify(|r| r.2 += 1);
                } else {
                    team_records.entry(aid as u64).and_modify(|r| r.1 += 1);
                    team_records.entry(hid as u64).and_modify(|r| r.2 += 1);
                }
            }
        }
    }

    // 确定当前轮次
    let completed_rounds: Vec<&str> = swiss_rows
        .iter()
        .filter(|r| {
            let s = r.get::<String, _>("status");
            s == "Completed" || s == "COMPLETED"
        })
        .map(|r| r.get::<&str, _>("stage"))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let current_round = completed_rounds.len() as u32 + 1;

    // 构建队伍状态
    let mut qualified = Vec::new();
    let mut eliminated = Vec::new();
    let teams: Vec<SwissTeamStatus> = team_records
        .into_iter()
        .map(|(id, (name, wins, losses))| {
            let is_qualified = wins >= wins_to_qualify;
            let is_eliminated = losses >= losses_to_eliminate;
            if is_qualified { qualified.push(id); }
            if is_eliminated { eliminated.push(id); }

            SwissTeamStatus {
                team_id: id,
                team_name: name,
                wins,
                losses,
                is_qualified,
                is_eliminated,
            }
        })
        .collect();

    let completed = qualified.len() >= 4 || teams.iter().all(|t| t.is_qualified || t.is_eliminated);

    Ok(CommandResult::ok(SwissRoundStatus {
        current_round,
        teams,
        completed,
        qualified_teams: qualified,
        eliminated_teams: eliminated,
    }))
}

/// 生成下一轮瑞士轮对阵
#[tauri::command]
pub async fn generate_next_swiss_round(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<u64>>, String> {
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

    let _pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取瑞士轮状态
    drop(guard);
    drop(current_save);
    let status_result = get_swiss_round_status(state.clone(), tournament_id).await?;
    let status = match status_result.data {
        Some(s) => s,
        None => return Ok(CommandResult::err("Failed to get swiss status")),
    };

    if status.completed {
        return Ok(CommandResult::err("Swiss rounds already completed"));
    }

    // 获取仍在竞争的队伍，按战绩分组
    let active_teams: Vec<_> = status.teams
        .iter()
        .filter(|t| !t.is_qualified && !t.is_eliminated)
        .collect();

    // 按胜场分组配对
    let mut grouped: std::collections::HashMap<u32, Vec<u64>> = std::collections::HashMap::new();
    for team in active_teams {
        grouped.entry(team.wins).or_default().push(team.team_id);
    }

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };
    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let stage = format!("SWISS_R{}", status.current_round);
    let mut new_match_ids = Vec::new();
    let mut match_order = 1u32;

    for (_wins, teams) in grouped {
        let mut teams = teams;
        while teams.len() >= 2 {
            let team1 = teams.pop().unwrap();
            let team2 = teams.pop().unwrap();

            let new_id: i64 = sqlx::query(
                r#"
                INSERT INTO matches (
                    save_id, tournament_id, stage, match_order, format,
                    home_team_id, away_team_id, home_score, away_score, status
                ) VALUES (?, ?, ?, ?, 'Bo3', ?, ?, 0, 0, 'Scheduled')
                RETURNING id
                "#,
            )
            .bind(&save_id)
            .bind(tournament_id as i64)
            .bind(&stage)
            .bind(match_order as i64)
            .bind(team1 as i64)
            .bind(team2 as i64)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?
            .get("id");

            new_match_ids.push(new_id as u64);
            match_order += 1;
        }
    }

    Ok(CommandResult::ok(new_match_ids))
}

/// 辅助函数：根据ID获取队伍
async fn get_teams_by_ids(pool: &sqlx::SqlitePool, ids: &[u64]) -> Result<Vec<crate::models::Team>, String> {
    let mut teams = Vec::new();
    for id in ids {
        let row = sqlx::query(
            "SELECT id, region_id, name, short_name, power_rating, total_matches, wins, win_rate, annual_points, cross_year_points, balance FROM teams WHERE id = ?"
        )
        .bind(*id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            teams.push(crate::models::Team {
                id: r.get::<i64, _>("id") as u64,
                region_id: r.get::<i64, _>("region_id") as u64,
                name: r.get("name"),
                short_name: r.get("short_name"),
                power_rating: r.get("power_rating"),
                total_matches: r.get::<i64, _>("total_matches") as u32,
                wins: r.get::<i64, _>("wins") as u32,
                win_rate: r.get("win_rate"),
                annual_points: r.get::<i64, _>("annual_points") as u32,
                cross_year_points: r.get::<i64, _>("cross_year_points") as u32,
                balance: r.get("balance"),
                brand_value: r.get("brand_value"),
            });
        }
    }
    Ok(teams)
}

/// 辅助函数：确定胜者晋级的下一场比赛
fn determine_next_matches(stage: &str, match_order: u32) -> Vec<(String, u32, bool)> {
    match stage {
        // MSI双败制
        "QUALIFIER_R1" => vec![("LOSERS_R1".to_string(), match_order, true)],
        "CHALLENGER_R1" => vec![("LOSERS_R2".to_string(), match_order, true)],
        "LOSERS_R1" => vec![("LOSERS_R2".to_string(), match_order, false)],
        "LOSERS_R2" => vec![("LOSERS_R3".to_string(), match_order, false)],
        "WINNERS_R1" => vec![
            ("WINNERS_FINAL".to_string(), 1, match_order == 1),
            ("LOSERS_R3".to_string(), match_order, true),
        ],
        "LOSERS_R3" => vec![("LOSERS_R4".to_string(), 1, match_order == 1)],
        "WINNERS_FINAL" => vec![("GRAND_FINAL".to_string(), 1, true)],
        "LOSERS_R4" => vec![("LOSERS_FINAL".to_string(), 1, false)],
        "LOSERS_FINAL" => vec![("GRAND_FINAL".to_string(), 1, false)],

        // 大师赛淘汰赛
        "EAST_R1" => vec![("EAST_SEMI".to_string(), (match_order + 1) / 2, match_order % 2 == 1)],
        "WEST_R1" => vec![("WEST_SEMI".to_string(), (match_order + 1) / 2, match_order % 2 == 1)],
        "EAST_SEMI" => vec![("EAST_FINAL".to_string(), 1, match_order == 1)],
        "WEST_SEMI" => vec![("WEST_FINAL".to_string(), 1, match_order == 1)],
        "EAST_FINAL" => vec![("GRAND_FINAL".to_string(), 1, true)],
        "WEST_FINAL" => vec![("GRAND_FINAL".to_string(), 1, false)],

        // 世界赛淘汰赛
        "QUARTER_FINAL" => vec![("SEMI_FINAL".to_string(), (match_order + 1) / 2, match_order % 2 == 1)],
        // 世界赛淘汰赛 - 胜者只进决赛，败者进季军赛（由 determine_loser_next_match 处理）
        "SEMI_FINAL" => vec![("GRAND_FINAL".to_string(), 1, match_order == 1)],

        // Super赛事第三阶段 - 败者组胜者进入败者组决赛
        "PREP_LOSERS" => vec![("PREP_LOSERS_FINAL".to_string(), 1, false)], // 败者组胜者进入败者组决赛 away

        // Super赛事第四阶段 - 首轮胜者进入次轮
        "FINALS_R1" => vec![("FINALS_R2".to_string(), match_order, false)], // 首轮胜者进入对应次轮的away (home已有传奇组队伍)
        // Super赛事第四阶段 - 次轮胜者进入总决赛
        "FINALS_R2" => vec![("GRAND_FINAL".to_string(), 1, match_order == 1)], // 次轮1胜者=home, 次轮2胜者=away

        _ => vec![],
    }
}

/// 确定败者的下一场比赛
fn determine_loser_next_match(stage: &str, match_order: u32) -> Vec<(String, u32, bool)> {
    match stage {
        // MSI双败制 - 败者去向
        "CHALLENGER_R1" => vec![("LOSERS_R1".to_string(), match_order, false)], // 挑战者组败者进入败者组R1的away
        "WINNERS_R1" => vec![("LOSERS_R3".to_string(), match_order, true)],     // 胜者组R1败者进入败者组R3
        "WINNERS_FINAL" => vec![("LOSERS_FINAL".to_string(), 1, true)],         // 胜者组决赛败者进入败者组决赛

        // Super赛事 - 定位赛败者进入晋级赛（away位置）
        "CHALLENGER_POSITIONING" => vec![("CHALLENGER_PROMOTION".to_string(), match_order, false)], // 定位赛败者进入对应晋级赛的away
        // Super赛事 - 胜者组败者进入败者组决赛（home位置）
        "PREP_WINNERS" => vec![("PREP_LOSERS_FINAL".to_string(), 1, true)], // 胜者组败者进入败者组决赛 home
        // Super赛事第四阶段 - 次轮败者进入季军赛
        "FINALS_R2" => vec![("THIRD_PLACE".to_string(), 1, match_order == 1)], // 次轮1败者=home, 次轮2败者=away

        // 马德里大师赛半区决赛败者进入季军赛
        "EAST_FINAL" => vec![("THIRD_PLACE".to_string(), 1, true)],             // 东半区决赛败者进入季军赛home
        "WEST_FINAL" => vec![("THIRD_PLACE".to_string(), 1, false)],            // 西半区决赛败者进入季军赛away

        // 世界赛半决赛败者进入季军赛
        "SEMI_FINAL" => vec![("THIRD_PLACE".to_string(), 1, match_order != 1)],

        _ => vec![],
    }
}

/// 创建ICP洲际对抗赛
#[tauri::command]
pub async fn create_icp_tournament(
    state: State<'_, AppState>,
    region_teams: Vec<Vec<u64>>, // 4个赛区各4队的ID [[lck_ids], [lpl_ids], [lec_ids], [lcs_ids]]
) -> Result<CommandResult<u64>, String> {
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

    // 验证参数：需要4个赛区，每赛区4队
    if region_teams.len() != 4 {
        return Ok(CommandResult::err("ICP tournament requires exactly 4 regions"));
    }
    for (i, region) in region_teams.iter().enumerate() {
        if region.len() != 4 {
            return Ok(CommandResult::err(format!("Region {} must have exactly 4 teams", i + 1)));
        }
    }

    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let current_season: i64 = save_row.get("current_season");

    let tournament_id: i64 = sqlx::query(
        r#"
        INSERT INTO tournaments (save_id, name, tournament_type, season_id, region_id, status)
        VALUES (?, ?, 'IcpIntercontinental', ?, NULL, 'InProgress')
        RETURNING id
        "#,
    )
    .bind(&save_id)
    .bind(format!("ICP Intercontinental {}", current_season))
    .bind(current_season)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?
    .get("id");

    // 获取各赛区队伍信息
    let mut all_region_teams = Vec::new();
    for region_ids in &region_teams {
        let teams = get_teams_by_ids(&pool, region_ids).await?;
        all_region_teams.push(teams);
    }

    let service = TournamentService::new();
    let matches = service.generate_icp_bracket(tournament_id as u64, all_region_teams);

    for m in matches {
        sqlx::query(
            r#"
            INSERT INTO matches (
                tournament_id, stage, round, match_order, format,
                home_team_id, away_team_id, home_score, away_score, winner_id, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, NULL, 'Scheduled')
            "#,
        )
        .bind(tournament_id)
        .bind(&m.stage)
        .bind(m.round)
        .bind(m.match_order)
        .bind(format!("{:?}", m.format))
        .bind(if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None })
        .bind(if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None })
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CommandResult::ok(tournament_id as u64))
}

/// 获取小组赛积分榜
#[tauri::command]
pub async fn get_group_standings(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<GroupStandingInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取所有小组赛比赛（支持 GROUP_X, ICP_GROUP_X 和 FIGHTER_GROUP_X 格式）
    let match_rows = sqlx::query(
        r#"
        SELECT m.stage, m.home_team_id, m.away_team_id, m.home_score, m.away_score, m.winner_id, m.status,
               ht.name as home_name, at.name as away_name,
               COALESCE(hr.name, '') as home_region, COALESCE(ar.name, '') as away_region
        FROM matches m
        LEFT JOIN teams ht ON m.home_team_id = ht.id
        LEFT JOIN teams at ON m.away_team_id = at.id
        LEFT JOIN regions hr ON ht.region_id = hr.id
        LEFT JOIN regions ar ON at.region_id = ar.id
        WHERE m.tournament_id = ? AND (m.stage LIKE 'GROUP_%' OR m.stage LIKE 'ICP_GROUP_%' OR m.stage LIKE 'FIGHTER_GROUP_%')
        ORDER BY m.stage, m.id
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 按小组统计
    let mut group_stats: std::collections::HashMap<String, std::collections::HashMap<u64, TeamGroupStats>> =
        std::collections::HashMap::new();

    for row in &match_rows {
        let stage: String = row.get("stage");
        let home_id: Option<i64> = row.get("home_team_id");
        let away_id: Option<i64> = row.get("away_team_id");
        let home_score: i64 = row.get("home_score");
        let away_score: i64 = row.get("away_score");
        let winner_id: Option<i64> = row.get("winner_id");
        let status: String = row.get("status");

        let group_entry = group_stats.entry(stage.clone()).or_default();

        // 初始化队伍
        if let Some(hid) = home_id {
            let home_name: String = row.get("home_name");
            let home_region: String = row.try_get("home_region").unwrap_or_default();
            group_entry.entry(hid as u64).or_insert(TeamGroupStats {
                team_id: hid as u64,
                team_name: home_name,
                region_code: home_region,
                wins: 0,
                losses: 0,
                games_won: 0,
                games_lost: 0,
                points: 0,
            });
        }
        if let Some(aid) = away_id {
            let away_name: String = row.get("away_name");
            let away_region: String = row.try_get("away_region").unwrap_or_default();
            group_entry.entry(aid as u64).or_insert(TeamGroupStats {
                team_id: aid as u64,
                team_name: away_name,
                region_code: away_region,
                wins: 0,
                losses: 0,
                games_won: 0,
                games_lost: 0,
                points: 0,
            });
        }

        // 更新统计
        if status == "Completed" || status == "COMPLETED" {
            if let (Some(hid), Some(aid), Some(wid)) = (home_id, away_id, winner_id) {
                let home_stats = group_entry.get_mut(&(hid as u64)).unwrap();
                home_stats.games_won += home_score as u32;
                home_stats.games_lost += away_score as u32;

                if wid == hid {
                    home_stats.wins += 1;
                    // 2:0胜积3分，2:1胜积2分
                    home_stats.points += if away_score == 0 { 3 } else { 2 };
                } else {
                    home_stats.losses += 1;
                    // 2:1负积1分，2:0负积0分
                    if home_score > 0 {
                        home_stats.points += 1;
                    }
                }

                let away_stats = group_entry.get_mut(&(aid as u64)).unwrap();
                away_stats.games_won += away_score as u32;
                away_stats.games_lost += home_score as u32;

                if wid == aid {
                    away_stats.wins += 1;
                    away_stats.points += if home_score == 0 { 3 } else { 2 };
                } else {
                    away_stats.losses += 1;
                    if away_score > 0 {
                        away_stats.points += 1;
                    }
                }
            }
        }
    }

    // 转换为返回格式
    let mut standings: Vec<GroupStandingInfo> = Vec::new();
    for (group_name, teams) in group_stats {
        let mut team_list: Vec<TeamGroupStats> = teams.into_values().collect();
        // 按积分、净胜场、胜场、team_id（确保稳定排序）
        team_list.sort_by(|a, b| {
            let a_diff = a.games_won as i32 - a.games_lost as i32;
            let b_diff = b.games_won as i32 - b.games_lost as i32;
            b.points.cmp(&a.points)
                .then(b_diff.cmp(&a_diff))
                .then(b.wins.cmp(&a.wins))
                .then(a.team_id.cmp(&b.team_id)) // 使用 team_id 作为最终 tiebreaker 确保稳定排序
        });

        standings.push(GroupStandingInfo {
            group_name: group_name.replace("GROUP_", ""),
            teams: team_list,
        });
    }

    // 按组名排序
    standings.sort_by(|a, b| a.group_name.cmp(&b.group_name));

    Ok(CommandResult::ok(standings))
}

/// 小组积分榜信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupStandingInfo {
    pub group_name: String,
    pub teams: Vec<TeamGroupStats>,
}

/// 队伍小组赛统计
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamGroupStats {
    pub team_id: u64,
    pub team_name: String,
    pub region_code: String,
    pub wins: u32,
    pub losses: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub points: u32,
}

/// 生成淘汰赛对阵 (小组赛结束后调用)
#[tauri::command]
pub async fn generate_knockout_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<u64>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取赛事类型
    let tournament_row = sqlx::query(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournament_type: String = match tournament_row {
        Some(r) => r.get("tournament_type"),
        None => return Ok(CommandResult::err("Tournament not found")),
    };

    // 获取小组积分榜
    drop(guard);
    let standings_result = get_group_standings(state.clone(), tournament_id).await?;
    let standings = match standings_result.data {
        Some(s) => s,
        None => return Ok(CommandResult::err("Failed to get group standings")),
    };

    // 根据赛事类型确定晋级规则
    let mut qualified_teams: Vec<(u64, u32)> = Vec::new(); // (team_id, seed)

    match tournament_type.as_str() {
        "SuperIntercontinental" => {
            // Super赛事：从 Fighter 组积分榜获取每组第1名
            // 查找 FIGHTER_GROUP_A 和 FIGHTER_GROUP_B 组
            let fighter_a = standings.iter().find(|g| g.group_name == "FIGHTER_GROUP_A");
            let fighter_b = standings.iter().find(|g| g.group_name == "FIGHTER_GROUP_B");

            let fighter_a_winner = fighter_a.and_then(|g| g.teams.first()).map(|t| t.team_id);
            let fighter_b_winner = fighter_b.and_then(|g| g.teams.first()).map(|t| t.team_id);

            if fighter_a_winner.is_none() || fighter_b_winner.is_none() {
                return Ok(CommandResult::err("Fighter组积分榜数据不完整"));
            }

            // 获取 save_id
            let current_save_id = state.current_save_id.read().await;
            let save_id = match current_save_id.as_ref() {
                Some(id) => id.clone(),
                None => return Ok(CommandResult::err("No save loaded")),
            };
            drop(current_save_id);

            let guard = state.db.read().await;
            let db = match guard.as_ref() {
                Some(db) => db,
                None => return Ok(CommandResult::err("Database not initialized")),
            };
            let pool = match db.get_pool().await {
                Ok(p) => p,
                Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
            };

            // 获取挑战者组队伍（5-8名，用于定位赛）
            let challenger_rows = sqlx::query(
                r#"
                SELECT m.home_team_id, m.away_team_id
                FROM matches m
                WHERE m.tournament_id = ? AND m.stage = 'CHALLENGER_QUALIFIER'
                ORDER BY m.match_order
                "#
            )
            .bind(tournament_id as i64)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;

            // 如果 CHALLENGER_QUALIFIER 比赛已存在，说明已初始化过
            // 需要创建挑战者组阶段比赛：CHALLENGER_POSITIONING 和 CHALLENGER_PROMOTION

            // 获取挑战者组队伍 ID（5-8名）
            let mut challenger_team_ids: Vec<u64> = Vec::new();
            for row in &challenger_rows {
                let home_id: Option<i64> = row.get("home_team_id");
                let away_id: Option<i64> = row.get("away_team_id");
                if let Some(id) = home_id {
                    if id > 0 && !challenger_team_ids.contains(&(id as u64)) {
                        challenger_team_ids.push(id as u64);
                    }
                }
                if let Some(id) = away_id {
                    if id > 0 && !challenger_team_ids.contains(&(id as u64)) {
                        challenger_team_ids.push(id as u64);
                    }
                }
            }

            // 如果没有从 CHALLENGER_QUALIFIER 获取到，尝试从初始化数据获取
            if challenger_team_ids.len() < 4 {
                // 重新查询：获取 Super 赛事中排名 5-8 的队伍
                let annual_teams: Vec<(i64,)> = sqlx::query_as(
                    r#"
                    SELECT DISTINCT m.home_team_id as team_id
                    FROM matches m
                    WHERE m.tournament_id = ? AND m.stage LIKE 'CHALLENGER%'
                    UNION
                    SELECT DISTINCT m.away_team_id as team_id
                    FROM matches m
                    WHERE m.tournament_id = ? AND m.stage LIKE 'CHALLENGER%'
                    "#
                )
                .bind(tournament_id as i64)
                .bind(tournament_id as i64)
                .fetch_all(&pool)
                .await
                .map_err(|e| e.to_string())?;

                challenger_team_ids = annual_teams.iter().filter_map(|r| {
                    if r.0 > 0 { Some(r.0 as u64) } else { None }
                }).collect();
            }

            // 创建 CHALLENGER_POSITIONING 比赛（定位赛：5 vs 8, 6 vs 7）
            // 定位赛胜者直接进入冠军预备战胜者组
            if challenger_team_ids.len() >= 4 {
                // 定位赛1: 5 vs 8
                sqlx::query(
                    r#"
                    INSERT INTO matches (save_id, tournament_id, stage, match_order, home_team_id, away_team_id, format, status)
                    VALUES (?, ?, 'CHALLENGER_POSITIONING', 1, ?, ?, 'BO5', 'Scheduled')
                    "#
                )
                .bind(&save_id)
                .bind(tournament_id as i64)
                .bind(challenger_team_ids[0] as i64)
                .bind(challenger_team_ids[3] as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

                // 定位赛2: 6 vs 7
                sqlx::query(
                    r#"
                    INSERT INTO matches (save_id, tournament_id, stage, match_order, home_team_id, away_team_id, format, status)
                    VALUES (?, ?, 'CHALLENGER_POSITIONING', 2, ?, ?, 'BO5', 'Scheduled')
                    "#
                )
                .bind(&save_id)
                .bind(tournament_id as i64)
                .bind(challenger_team_ids[1] as i64)
                .bind(challenger_team_ids[2] as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            }

            // 更新 CHALLENGER_PROMOTION 比赛（晋级赛：Fighter胜者 vs 定位赛败者）
            // 晋级赛1: Fighter A组第1 vs 定位赛1败者
            sqlx::query(
                r#"
                UPDATE matches
                SET home_team_id = ?
                WHERE save_id = ? AND tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = 1
                "#
            )
            .bind(fighter_a_winner.unwrap() as i64)
            .bind(&save_id)
            .bind(tournament_id as i64)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

            // 晋级赛2: Fighter B组第1 vs 定位赛2败者
            sqlx::query(
                r#"
                UPDATE matches
                SET home_team_id = ?
                WHERE save_id = ? AND tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = 2
                "#
            )
            .bind(fighter_b_winner.unwrap() as i64)
            .bind(&save_id)
            .bind(tournament_id as i64)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

            return Ok(CommandResult::ok(vec![]));
        }
        "MadridMasters" | "ClaudeIntercontinental" => {
            // 每组前2名晋级，共16队
            for group in &standings {
                for (idx, team) in group.teams.iter().enumerate() {
                    if idx < 2 {
                        qualified_teams.push((team.team_id, (idx + 1) as u32));
                    }
                }
            }
        }
        _ => {
            // 默认每组前2名
            for group in &standings {
                for (idx, team) in group.teams.iter().enumerate() {
                    if idx < 2 {
                        qualified_teams.push((team.team_id, (idx + 1) as u32));
                    }
                }
            }
        }
    }

    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };
    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 更新淘汰赛对阵
    // 东半区：A组1 vs D组2, B组1 vs C组2, A组2 vs D组1, B组2 vs C组1
    // 西半区：E组1 vs H组2, F组1 vs G组2, E组2 vs H组1, F组2 vs G组1
    let east_matches = vec![
        ("EAST_R1", 1), ("EAST_R1", 2), ("EAST_R1", 3), ("EAST_R1", 4),
    ];
    let west_matches = vec![
        ("WEST_R1", 1), ("WEST_R1", 2), ("WEST_R1", 3), ("WEST_R1", 4),
    ];

    let mut updated_match_ids = Vec::new();

    // 简化：按顺序填充淘汰赛
    let mut east_teams: Vec<u64> = Vec::new();
    let mut west_teams: Vec<u64> = Vec::new();

    for (idx, group) in standings.iter().enumerate() {
        if idx < 4 {
            // 东半区 A-D组
            if group.teams.len() >= 2 {
                east_teams.push(group.teams[0].team_id);
                east_teams.push(group.teams[1].team_id);
            }
        } else {
            // 西半区 E-H组
            if group.teams.len() >= 2 {
                west_teams.push(group.teams[0].team_id);
                west_teams.push(group.teams[1].team_id);
            }
        }
    }

    // 更新东半区
    for (i, (stage, order)) in east_matches.iter().enumerate() {
        if i * 2 + 1 < east_teams.len() {
            let home_id = east_teams[i * 2];
            let away_id = east_teams[i * 2 + 1];

            let result = sqlx::query(
                "UPDATE matches SET home_team_id = ?, away_team_id = ? WHERE tournament_id = ? AND stage = ? AND match_order = ? RETURNING id"
            )
            .bind(home_id as i64)
            .bind(away_id as i64)
            .bind(tournament_id as i64)
            .bind(stage)
            .bind(*order as i64)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(row) = result {
                let id: i64 = row.get("id");
                updated_match_ids.push(id as u64);
            }
        }
    }

    // 更新西半区
    for (i, (stage, order)) in west_matches.iter().enumerate() {
        if i * 2 + 1 < west_teams.len() {
            let home_id = west_teams[i * 2];
            let away_id = west_teams[i * 2 + 1];

            let result = sqlx::query(
                "UPDATE matches SET home_team_id = ?, away_team_id = ? WHERE tournament_id = ? AND stage = ? AND match_order = ? RETURNING id"
            )
            .bind(home_id as i64)
            .bind(away_id as i64)
            .bind(tournament_id as i64)
            .bind(stage)
            .bind(*order as i64)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(row) = result {
                let id: i64 = row.get("id");
                updated_match_ids.push(id as u64);
            }
        }
    }

    Ok(CommandResult::ok(updated_match_ids))
}

/// 赛事完成结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentCompletionResult {
    pub tournament_id: u64,
    pub tournament_name: String,
    pub honors_awarded: Vec<HonorAwardedInfo>,
    pub points_awarded: Vec<PointsAwardedInfo>,
    pub message: String,
}

/// 颁发的荣誉信息
#[derive(Debug, Serialize, Deserialize)]
pub struct HonorAwardedInfo {
    pub honor_type: String,
    pub recipient_name: String,
    pub recipient_type: String, // "team" or "player"
}

/// 颁发的积分信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PointsAwardedInfo {
    pub team_id: u64,
    pub team_name: String,
    pub points: u32,
    pub position: String,
}

/// 完成赛事 - 处理荣誉殿堂和年度积分
/// 当国际赛事（如MSI、上海大师赛等）完成后调用此命令
#[tauri::command]
pub async fn complete_tournament(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<TournamentCompletionResult>, String> {
    use crate::services::HonorService;
    use crate::engines::PointsCalculationEngine;
    use crate::db::{PointsRepository, TeamRepository, TournamentRepository};

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

    // 1. 获取赛事信息
    let tournament = TournamentRepository::get_by_id(&pool, tournament_id)
        .await
        .map_err(|e| format!("Failed to get tournament: {}", e))?;

    let tournament_name = tournament.name.clone();
    let tournament_type = tournament.tournament_type;
    let season_id = tournament.season_id;

    // 2. 处理荣誉殿堂
    let honor_service = HonorService::new();
    let mut honors_awarded = Vec::new();

    match honor_service.process_tournament_completion(&pool, &save_id, tournament_id).await {
        Ok(tournament_honors) => {
            // 收集战队荣誉
            if let Some(ref honor) = tournament_honors.team_champion {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "冠军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.team_runner_up {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "亚军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.team_third {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "季军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.team_fourth {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "殿军".to_string(),
                    recipient_name: honor.team_name.clone().unwrap_or_default(),
                    recipient_type: "team".to_string(),
                });
            }
            // 收集选手冠军荣誉
            for honor in &tournament_honors.player_champions {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "冠军成员".to_string(),
                    recipient_name: honor.player_name.clone().unwrap_or_default(),
                    recipient_type: "player".to_string(),
                });
            }
            // 收集MVP荣誉
            if let Some(ref honor) = tournament_honors.tournament_mvp {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "赛事MVP".to_string(),
                    recipient_name: honor.player_name.clone().unwrap_or_default(),
                    recipient_type: "player".to_string(),
                });
            }
            if let Some(ref honor) = tournament_honors.finals_mvp {
                honors_awarded.push(HonorAwardedInfo {
                    honor_type: "决赛MVP".to_string(),
                    recipient_name: honor.player_name.clone().unwrap_or_default(),
                    recipient_type: "player".to_string(),
                });
            }
        }
        Err(e) => {
            log::error!("Failed to process tournament honors: {}", e);
        }
    }

    // 3. 处理年度积分（Super赛除外，因为Super赛是年度积分的奖励）
    let mut points_awarded = Vec::new();

    if !matches!(tournament_type, crate::models::TournamentType::SuperIntercontinental) {
        let points_engine = PointsCalculationEngine::new();

        // 获取赛事最终排名
        let results = get_tournament_final_rankings(&pool, tournament_id).await?;

        for (team_id, position) in &results {
            let points = points_engine.get_points(tournament_type, position);
            if points > 0 {
                // 获取队伍名称
                let team = TeamRepository::get_by_id(&pool, *team_id)
                    .await
                    .map_err(|e| format!("Failed to get team: {}", e))?;

                // 保存积分明细（带去重检查）
                let is_new = match PointsRepository::add_points_detail(
                    &pool,
                    &save_id,
                    season_id,
                    *team_id,
                    tournament_id,
                    points,
                    position_to_rank(position),
                ).await {
                    Ok((_, new)) => new,
                    Err(e) => {
                        log::error!("Failed to add points detail: {}", e);
                        continue;
                    }
                };

                // 只有新记录才更新队伍的年度积分
                if is_new {
                    let mut team_to_update = team.clone();
                    team_to_update.annual_points += points;
                    if let Err(e) = TeamRepository::update(&pool, &team_to_update).await {
                        log::error!("Failed to update team annual points: {}", e);
                        continue;
                    }

                    points_awarded.push(PointsAwardedInfo {
                        team_id: *team_id,
                        team_name: team.name.clone(),
                        points,
                        position: position.clone(),
                    });

                    log::debug!("Awarded {} points to {} for position {}",
                        points, team.name, position);
                } else {
                    log::debug!("Skipped duplicate points for {} in tournament {}",
                        team.name, tournament_id);
                }
            }
        }
    }

    // 4. 更新赛事状态为已完成
    let _ = sqlx::query("UPDATE tournaments SET status = 'Completed' WHERE id = ?")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await;

    let message = format!(
        "赛事 {} 完成！颁发了 {} 个荣誉和 {} 条积分记录",
        tournament_name,
        honors_awarded.len(),
        points_awarded.len()
    );

    Ok(CommandResult::ok(TournamentCompletionResult {
        tournament_id,
        tournament_name,
        honors_awarded,
        points_awarded,
        message,
    }))
}

/// 获取赛事最终排名
async fn get_tournament_final_rankings(
    pool: &sqlx::SqlitePool,
    tournament_id: u64,
) -> Result<Vec<(u64, String)>, String> {
    let mut results: Vec<(u64, String)> = Vec::new();

    // 获取赛事类型
    let tournament_type: Option<String> = sqlx::query_scalar(
        "SELECT tournament_type FROM tournaments WHERE id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let is_double_elimination = matches!(
        tournament_type.as_deref(),
        Some("Msi") | Some("ShanghaiMasters")
    );

    // 获取所有已完成的比赛
    let knockout_matches = sqlx::query(
        r#"
        SELECT * FROM matches
        WHERE tournament_id = ? AND UPPER(status) = 'COMPLETED'
        ORDER BY stage DESC, match_order
        "#,
    )
    .bind(tournament_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    log::debug!("tournament_id={}, is_double_elimination={}, matches={}",
        tournament_id, is_double_elimination, knockout_matches.len());

    // 找到决赛 - 冠军和亚军
    for m in &knockout_matches {
        let stage: String = m.get("stage");
        if stage == "FINALS" || stage == "GRAND_FINALS" || stage == "GRAND_FINAL" {
            let winner_id = m.get::<Option<i64>, _>("winner_id");
            let home_id = m.get::<i64, _>("home_team_id") as u64;
            let away_id = m.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let runner_up = if winner == home_id { away_id } else { home_id };
                results.push((winner, "CHAMPION".to_string()));
                results.push((runner_up, "RUNNER_UP".to_string()));
                log::debug!("CHAMPION={}, RUNNER_UP={}", winner, runner_up);
            }
            break;
        }
    }

    if is_double_elimination {
        // === 双败制赛事（MSI/上海大师赛）===

        // 季军 = 败者组决赛的败者
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_FINAL" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "THIRD".to_string()));
                        log::debug!("THIRD={}", loser);
                    }
                }
                break;
            }
        }

        // 殿军 = 败者组R4的败者
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R4" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "FOURTH".to_string()));
                        log::debug!("FOURTH={}", loser);
                    }
                }
                break;
            }
        }

        // 败者组R3败者 (5-6名) = LOSERS_R2积分
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R3" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "LOSERS_R2".to_string()));
                        log::debug!("LOSERS_R2={}", loser);
                    }
                }
            }
        }

        // 败者组R2败者 (7-8名) = LOSERS_R1积分
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R2" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "LOSERS_R1".to_string()));
                        log::debug!("LOSERS_R1={}", loser);
                    }
                }
            }
        }

        // 败者组R1败者 (9-10名) - 资格赛被淘汰
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "LOSERS_R1" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "QUALIFIER_OUT".to_string()));
                        log::debug!("QUALIFIER_OUT={}", loser);
                    }
                }
            }
        }

        // 资格赛R1败者 (11-12名)
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "QUALIFIER_R1" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "QUALIFIER_R1_OUT".to_string()));
                        log::debug!("QUALIFIER_R1_OUT={}", loser);
                    }
                }
            }
        }

    } else {
        // === 标准淘汰赛赛事 ===

        // 找半决赛败者（季军/殿军）
        let mut semi_losers: Vec<u64> = Vec::new();
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "SEMI_FINALS" || stage == "SEMI_FINAL" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        semi_losers.push(loser);
                    }
                }
            }
        }

        // 分配季军和殿军
        if semi_losers.len() >= 2 {
            results.push((semi_losers[0], "THIRD".to_string()));
            results.push((semi_losers[1], "FOURTH".to_string()));
        } else if semi_losers.len() == 1 {
            results.push((semi_losers[0], "THIRD".to_string()));
        }

        // 八强失败者
        for m in &knockout_matches {
            let stage: String = m.get("stage");
            if stage == "QUARTER_FINALS" || stage == "QUARTER_FINAL" {
                let winner_id = m.get::<Option<i64>, _>("winner_id");
                let home_id = m.get::<i64, _>("home_team_id") as u64;
                let away_id = m.get::<i64, _>("away_team_id") as u64;

                if let Some(winner) = winner_id {
                    let loser = if winner as u64 == home_id { away_id } else { home_id };
                    if !results.iter().any(|(id, _)| *id == loser) {
                        results.push((loser, "QUARTER_FINAL".to_string()));
                    }
                }
            }
        }
    }

    log::debug!("Final results: {:?}", results);
    Ok(results)
}

/// 位置转排名数字
fn position_to_rank(position: &str) -> Option<u32> {
    match position {
        "CHAMPION" => Some(1),
        "RUNNER_UP" => Some(2),
        "THIRD" => Some(3),
        "FOURTH" => Some(4),
        "5TH_8TH" | "QUARTER_FINAL" => Some(5),
        _ => None,
    }
}

/// 获取阶段显示名称
fn get_stage_display_name(stage: &str) -> String {
    match stage {
        "QUALIFIER_R1" => "资格赛".to_string(),
        "CHALLENGER_R1" => "挑战者组".to_string(),
        "LOSERS_R1" | "LOSERS_R2" | "LOSERS_R3" | "LOSERS_R4" => "败者组".to_string(),
        "LOSERS_FINAL" => "败者组决赛".to_string(),
        "WINNERS_R1" => "胜者组首轮".to_string(),
        "WINNERS_FINAL" => "胜者组决赛".to_string(),
        "GRAND_FINAL" => "总决赛".to_string(),
        "SWISS_R1" | "SWISS_R2" | "SWISS_R3" | "SWISS_R4" | "SWISS_R5" => "瑞士轮".to_string(),
        "QUARTER_FINAL" => "八强赛".to_string(),
        "SEMI_FINAL" => "半决赛".to_string(),
        "THIRD_PLACE" => "季军赛".to_string(),
        "EAST_R1" | "WEST_R1" => "淘汰赛首轮".to_string(),
        "EAST_SEMI" | "WEST_SEMI" => "半决赛".to_string(),
        "EAST_FINAL" | "WEST_FINAL" => "半区决赛".to_string(),
        s if s.starts_with("GROUP_") => format!("{}组", &s[6..]),
        s if s.starts_with("FIGHTER_GROUP_") => format!("Fighter {}组", &s[14..]),
        _ => stage.to_string(),
    }
}

/// MSI参赛队伍分组信息
#[derive(Debug, Serialize, Deserialize)]
pub struct MsiTeamGroups {
    /// 传奇组队伍（各赛区冠军）
    pub legendary: Vec<MsiTeamInfo>,
    /// 挑战者组队伍（各赛区亚军）
    pub challenger: Vec<MsiTeamInfo>,
    /// 资格赛组队伍（各赛区季军）
    pub qualifier: Vec<MsiTeamInfo>,
}

/// MSI队伍信息
#[derive(Debug, Serialize, Deserialize)]
pub struct MsiTeamInfo {
    pub team_id: u64,
    pub team_name: String,
    pub short_name: String,
    pub region_id: u64,
    pub region_name: String,
}

/// 获取MSI参赛队伍分组（基于春季季后赛结果）
#[tauri::command]
pub async fn get_msi_qualified_teams(
    state: State<'_, AppState>,
    season_id: u64,
) -> Result<CommandResult<MsiTeamGroups>, String> {
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

    let mut legendary = Vec::new();
    let mut challenger = Vec::new();
    let mut qualifier = Vec::new();

    // 获取所有春季季后赛
    let playoffs = sqlx::query(
        r#"
        SELECT t.id, t.region_id, r.name as region_name
        FROM tournaments t
        LEFT JOIN regions r ON t.region_id = r.id
        WHERE t.save_id = ? AND t.season_id = ? AND t.tournament_type = 'SpringPlayoffs'
        "#
    )
    .bind(&save_id)
    .bind(season_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for playoff in playoffs {
        let tournament_id: i64 = playoff.get("id");
        let region_id: Option<i64> = playoff.get("region_id");
        let region_name: Option<String> = playoff.get("region_name");

        // 获取总决赛结果
        let grand_final = sqlx::query(
            r#"
            SELECT m.winner_id, m.home_team_id, m.away_team_id
            FROM matches m
            WHERE m.tournament_id = ? AND m.stage = 'GRAND_FINAL' AND UPPER(m.status) = 'COMPLETED'
            "#
        )
        .bind(tournament_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 获取败者组决赛结果（用于确定季军）
        let losers_final = sqlx::query(
            r#"
            SELECT m.winner_id, m.home_team_id, m.away_team_id
            FROM matches m
            WHERE m.tournament_id = ? AND m.stage = 'LOSERS_FINAL' AND UPPER(m.status) = 'COMPLETED'
            "#
        )
        .bind(tournament_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(gf) = grand_final {
            let winner_id: Option<i64> = gf.get("winner_id");
            let home_id: i64 = gf.get("home_team_id");
            let away_id: i64 = gf.get("away_team_id");

            if let Some(champion_id) = winner_id {
                // 冠军 -> 传奇组
                let runner_up_id = if champion_id == home_id { away_id } else { home_id };

                if let Ok(Some(team_info)) = get_team_info(&pool, champion_id as u64, region_id.map(|r| r as u64), &region_name).await {
                    legendary.push(team_info);
                }

                // 亚军 -> 挑战者组
                if let Ok(Some(team_info)) = get_team_info(&pool, runner_up_id as u64, region_id.map(|r| r as u64), &region_name).await {
                    challenger.push(team_info);
                }
            }
        }

        // 季军 -> 资格赛组（败者组决赛的败者）
        if let Some(lf) = losers_final {
            let winner_id: Option<i64> = lf.get("winner_id");
            let home_id: i64 = lf.get("home_team_id");
            let away_id: i64 = lf.get("away_team_id");

            if let Some(winner) = winner_id {
                let third_id = if winner == home_id { away_id } else { home_id };

                if let Ok(Some(team_info)) = get_team_info(&pool, third_id as u64, region_id.map(|r| r as u64), &region_name).await {
                    qualifier.push(team_info);
                }
            }
        }
    }

    log::debug!("legendary={}, challenger={}, qualifier={}",
        legendary.len(), challenger.len(), qualifier.len());

    Ok(CommandResult::ok(MsiTeamGroups {
        legendary,
        challenger,
        qualifier,
    }))
}

/// 获取队伍信息的辅助函数
async fn get_team_info(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    team_id: u64,
    region_id: Option<u64>,
    region_name: &Option<String>,
) -> Result<Option<MsiTeamInfo>, String> {
    let row = sqlx::query(
        "SELECT id, name, short_name FROM teams WHERE id = ?"
    )
    .bind(team_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| {
        let name: String = r.get("name");
        let short_name: Option<String> = r.get("short_name");
        MsiTeamInfo {
            team_id,
            team_name: name.clone(),
            short_name: short_name.unwrap_or(name),
            region_id: region_id.unwrap_or(0),
            region_name: region_name.clone().unwrap_or_default(),
        }
    }))
}

/// 重新生成 MSI 对阵 - 当队伍就绪但比赛未生成时使用
#[tauri::command]
pub async fn regenerate_msi_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<u32>, String> {
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

    // 检查是否是 MSI 赛事
    let tournament = sqlx::query(
        "SELECT id, season_id, tournament_type FROM tournaments WHERE id = ? AND save_id = ?"
    )
    .bind(tournament_id as i64)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournament = match tournament {
        Some(t) => t,
        None => return Ok(CommandResult::err("Tournament not found")),
    };

    let tournament_type: String = tournament.get("tournament_type");
    if tournament_type != "Msi" {
        return Ok(CommandResult::err("Not an MSI tournament"));
    }

    let season_id: i64 = tournament.get("season_id");

    // 检查是否已有比赛
    let existing_matches = sqlx::query(
        "SELECT COUNT(*) as cnt FROM matches WHERE tournament_id = ?"
    )
    .bind(tournament_id as i64)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let count: i64 = existing_matches.get("cnt");
    if count > 0 {
        return Ok(CommandResult::err(format!("Tournament already has {} matches", count)));
    }

    // 获取所有春季季后赛的前3名
    let playoffs = sqlx::query(
        r#"
        SELECT t.id, t.region_id
        FROM tournaments t
        WHERE t.save_id = ? AND t.season_id = ? AND t.tournament_type = 'SpringPlayoffs'
        AND UPPER(t.status) = 'COMPLETED'
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    log::debug!("找到 {} 个已完成的春季季后赛", playoffs.len());

    let mut legendary_teams: Vec<Team> = Vec::new();
    let mut challenger_teams: Vec<Team> = Vec::new();
    let mut qualifier_teams: Vec<Team> = Vec::new();

    for playoff in &playoffs {
        let playoff_id: i64 = playoff.get("id");
        let region_id: Option<i64> = playoff.get("region_id");

        // 获取总决赛结果 - 冠亚军
        let grand_final = sqlx::query(
            r#"
            SELECT winner_id, home_team_id, away_team_id
            FROM matches
            WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
            "#
        )
        .bind(playoff_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 获取败者组决赛结果 - 季军
        let losers_final = sqlx::query(
            r#"
            SELECT winner_id, home_team_id, away_team_id
            FROM matches
            WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
            "#
        )
        .bind(playoff_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let (Some(gf), Some(lf)) = (grand_final, losers_final) {
            let champion_id: i64 = gf.get("winner_id");
            let gf_home: i64 = gf.get("home_team_id");
            let gf_away: i64 = gf.get("away_team_id");
            let runner_up_id = if champion_id == gf_home { gf_away } else { gf_home };

            let lf_winner: i64 = lf.get("winner_id");
            let third_place_id = lf_winner;

            // 获取队伍信息
            let champion = sqlx::query(
                "SELECT id, name, short_name, save_id FROM teams WHERE id = ?"
            )
            .bind(champion_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            let runner_up = sqlx::query(
                "SELECT id, name, short_name, save_id FROM teams WHERE id = ?"
            )
            .bind(runner_up_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            let third = sqlx::query(
                "SELECT id, name, short_name, save_id FROM teams WHERE id = ?"
            )
            .bind(third_place_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(c) = champion {
                legendary_teams.push(Team {
                    id: c.get::<i64, _>("id") as u64,
                    region_id: region_id.unwrap_or(0) as u64,
                    name: c.get("name"),
                    short_name: c.get("short_name"),
                    power_rating: 0.0,
                    total_matches: 0,
                    wins: 0,
                    win_rate: 0.0,
                    annual_points: 0,
                    cross_year_points: 0,
                    balance: 0,
                    brand_value: 0.0,
                });
            }

            if let Some(ru) = runner_up {
                challenger_teams.push(Team {
                    id: ru.get::<i64, _>("id") as u64,
                    region_id: region_id.unwrap_or(0) as u64,
                    name: ru.get("name"),
                    short_name: ru.get("short_name"),
                    power_rating: 0.0,
                    total_matches: 0,
                    wins: 0,
                    win_rate: 0.0,
                    annual_points: 0,
                    cross_year_points: 0,
                    balance: 0,
                    brand_value: 0.0,
                });
            }

            if let Some(t) = third {
                qualifier_teams.push(Team {
                    id: t.get::<i64, _>("id") as u64,
                    region_id: region_id.unwrap_or(0) as u64,
                    name: t.get("name"),
                    short_name: t.get("short_name"),
                    power_rating: 0.0,
                    total_matches: 0,
                    wins: 0,
                    win_rate: 0.0,
                    annual_points: 0,
                    cross_year_points: 0,
                    balance: 0,
                    brand_value: 0.0,
                });
            }
        }
    }

    log::debug!("legendary={}, challenger={}, qualifier={}",
        legendary_teams.len(), challenger_teams.len(), qualifier_teams.len());

    if legendary_teams.len() != 4 || challenger_teams.len() != 4 || qualifier_teams.len() != 4 {
        return Ok(CommandResult::err(format!(
            "Not enough teams: legendary={}, challenger={}, qualifier={}",
            legendary_teams.len(), challenger_teams.len(), qualifier_teams.len()
        )));
    }

    // 生成 MSI 对阵
    let tournament_service = TournamentService::new();
    let matches = tournament_service.generate_msi_bracket(
        tournament_id,
        &legendary_teams,
        &challenger_teams,
        &qualifier_teams,
    );

    let match_count = matches.len();

    // 保存比赛
    MatchRepository::create_batch(&pool, &save_id, &matches)
        .await
        .map_err(|e| format!("Failed to save matches: {}", e))?;

    log::debug!("成功生成 {} 场比赛", match_count);

    Ok(CommandResult::ok(match_count as u32))
}

/// 填充世界赛淘汰赛对阵（瑞士轮完成后调用）
#[tauri::command]
pub async fn fill_worlds_knockout_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
    qualified_team_ids: Vec<u64>, // 瑞士轮晋级的4支队伍ID
) -> Result<CommandResult<Vec<u64>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    if qualified_team_ids.len() != 4 {
        return Ok(CommandResult::err(format!(
            "Expected 4 qualified teams, got {}",
            qualified_team_ids.len()
        )));
    }

    // 获取八强赛比赛（按 match_order 排序）
    let quarter_matches = sqlx::query(
        r#"
        SELECT id, match_order FROM matches
        WHERE tournament_id = ? AND stage = 'QUARTER_FINAL'
        ORDER BY match_order
        "#
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if quarter_matches.len() != 4 {
        return Ok(CommandResult::err(format!(
            "Expected 4 quarter-final matches, found {}",
            quarter_matches.len()
        )));
    }

    let mut updated_ids = Vec::new();

    // 更新每场八强赛的 away_team_id
    for (i, qm) in quarter_matches.iter().enumerate() {
        let match_id: i64 = qm.get("id");
        let qualified_team_id = qualified_team_ids[i];

        sqlx::query("UPDATE matches SET away_team_id = ? WHERE id = ?")
            .bind(qualified_team_id as i64)
            .bind(match_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        updated_ids.push(match_id as u64);
        log::debug!("Match {} updated with away_team_id {}", match_id, qualified_team_id);
    }

    Ok(CommandResult::ok(updated_ids))
}

/// 清理重复的赛事（按类型）
/// 保留每种类型的第一个赛事，删除多余的
#[tauri::command]
pub async fn cleanup_duplicate_tournaments(
    state: State<'_, AppState>,
    tournament_type: String,
) -> Result<CommandResult<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized".to_string())),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded".to_string())),
    };

    let pool = db.get_pool().await.map_err(|e| e.to_string())?;

    // 获取游戏状态
    let save_row = sqlx::query("SELECT current_season FROM saves WHERE id = ?")
        .bind(&save_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let season_id: i64 = match save_row {
        Some(row) => row.get("current_season"),
        None => return Ok(CommandResult::err("Save not found".to_string())),
    };

    // 查找该类型的所有赛事
    let tournaments: Vec<(i64, String)> = sqlx::query_as(
        r#"
        SELECT id, status FROM tournaments
        WHERE save_id = ? AND season_id = ? AND tournament_type = ?
        ORDER BY id ASC
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .bind(&tournament_type)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if tournaments.len() <= 1 {
        log::debug!("No duplicates found for type {}", tournament_type);
        return Ok(CommandResult::ok(0));
    }

    log::debug!("Found {} tournaments of type {}, cleaning duplicates...",
        tournaments.len(), tournament_type);

    let mut deleted_count = 0u32;

    // 保留第一个，删除其余
    for (tournament_id, status) in tournaments.iter().skip(1) {
        log::debug!("Deleting tournament id={}, status={}", tournament_id, status);

        // 删除相关比赛
        sqlx::query("DELETE FROM matches WHERE tournament_id = ?")
            .bind(tournament_id)
            .execute(&pool)
            .await
            .ok();

        // 删除积分榜
        sqlx::query("DELETE FROM league_standings WHERE tournament_id = ?")
            .bind(tournament_id)
            .execute(&pool)
            .await
            .ok();

        // 删除赛事本身
        let result = sqlx::query("DELETE FROM tournaments WHERE id = ?")
            .bind(tournament_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected() > 0 {
            deleted_count += 1;
        }
    }

    log::debug!("Deleted {} duplicate tournaments", deleted_count);
    Ok(CommandResult::ok(deleted_count))
}

/// 获取上海大师赛参赛队伍分组（基于夏季季后赛结果）
#[tauri::command]
pub async fn get_shanghai_qualified_teams(
    state: State<'_, AppState>,
    season_id: u64,
) -> Result<CommandResult<MsiTeamGroups>, String> {
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

    let mut legendary = Vec::new();
    let mut challenger = Vec::new();
    let mut qualifier = Vec::new();

    // 获取所有夏季季后赛（与MSI不同，上海大师赛使用夏季赛结果）
    let playoffs = sqlx::query(
        r#"
        SELECT t.id, t.region_id, r.name as region_name
        FROM tournaments t
        LEFT JOIN regions r ON t.region_id = r.id
        WHERE t.save_id = ? AND t.season_id = ? AND t.tournament_type = 'SummerPlayoffs'
        "#
    )
    .bind(&save_id)
    .bind(season_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    log::debug!("Found {} SummerPlayoffs", playoffs.len());

    for playoff in playoffs {
        let tournament_id: i64 = playoff.get("id");
        let region_id: Option<i64> = playoff.get("region_id");
        let region_name: Option<String> = playoff.get("region_name");

        // 获取总决赛结果
        let grand_final = sqlx::query(
            r#"
            SELECT m.winner_id, m.home_team_id, m.away_team_id
            FROM matches m
            WHERE m.tournament_id = ? AND m.stage = 'GRAND_FINAL' AND UPPER(m.status) = 'COMPLETED'
            "#
        )
        .bind(tournament_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 获取败者组决赛结果（用于确定季军）
        let losers_final = sqlx::query(
            r#"
            SELECT m.winner_id, m.home_team_id, m.away_team_id
            FROM matches m
            WHERE m.tournament_id = ? AND m.stage = 'LOSERS_FINAL' AND UPPER(m.status) = 'COMPLETED'
            "#
        )
        .bind(tournament_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(gf) = grand_final {
            let winner_id: Option<i64> = gf.get("winner_id");
            let home_id: i64 = gf.get("home_team_id");
            let away_id: i64 = gf.get("away_team_id");

            if let Some(champion_id) = winner_id {
                // 冠军 -> 传奇组
                let runner_up_id = if champion_id == home_id { away_id } else { home_id };

                if let Ok(Some(team_info)) = get_team_info(&pool, champion_id as u64, region_id.map(|r| r as u64), &region_name).await {
                    legendary.push(team_info);
                }

                // 亚军 -> 挑战者组
                if let Ok(Some(team_info)) = get_team_info(&pool, runner_up_id as u64, region_id.map(|r| r as u64), &region_name).await {
                    challenger.push(team_info);
                }
            }
        }

        // 季军 -> 资格赛组（败者组决赛的败者）
        if let Some(lf) = losers_final {
            let winner_id: Option<i64> = lf.get("winner_id");
            let home_id: i64 = lf.get("home_team_id");
            let away_id: i64 = lf.get("away_team_id");

            if let Some(winner) = winner_id {
                let third_id = if winner == home_id { away_id } else { home_id };

                if let Ok(Some(team_info)) = get_team_info(&pool, third_id as u64, region_id.map(|r| r as u64), &region_name).await {
                    qualifier.push(team_info);
                }
            }
        }
    }

    log::debug!("legendary={}, challenger={}, qualifier={}",
        legendary.len(), challenger.len(), qualifier.len());

    Ok(CommandResult::ok(MsiTeamGroups {
        legendary,
        challenger,
        qualifier,
    }))
}

/// 重新生成上海大师赛对阵（删除现有比赛并重新初始化）
#[tauri::command]
pub async fn regenerate_shanghai_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<u32>, String> {
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

    // 检查是否是上海大师赛
    let tournament = sqlx::query(
        "SELECT id, season_id, tournament_type FROM tournaments WHERE id = ? AND save_id = ?"
    )
    .bind(tournament_id as i64)
    .bind(&save_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let tournament = match tournament {
        Some(t) => t,
        None => return Ok(CommandResult::err("Tournament not found")),
    };

    let tournament_type: String = tournament.get("tournament_type");
    if tournament_type != "ShanghaiMasters" {
        return Ok(CommandResult::err(format!(
            "Tournament is not ShanghaiMasters: {}",
            tournament_type
        )));
    }

    let season_id: i64 = tournament.get("season_id");

    log::debug!("重置上海大师赛: tournament_id={}, season_id={}", tournament_id, season_id);

    // 1. 删除现有比赛的详细数据
    sqlx::query("DELETE FROM game_player_performances WHERE game_id LIKE ?")
        .bind(format!("{}_%", tournament_id))
        .execute(&pool)
        .await
        .ok();

    sqlx::query("DELETE FROM match_games WHERE match_id IN (SELECT id FROM matches WHERE tournament_id = ?)")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .ok();

    // 2. 删除现有比赛
    sqlx::query("DELETE FROM matches WHERE tournament_id = ?")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 3. 重置赛事状态
    sqlx::query("UPDATE tournaments SET status = 'Pending' WHERE id = ?")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .ok();

    // 4. 获取夏季季后赛结果
    let playoffs = sqlx::query(
        r#"
        SELECT t.id, t.region_id, r.name as region_name
        FROM tournaments t
        LEFT JOIN regions r ON t.region_id = r.id
        WHERE t.save_id = ?
          AND t.season_id = ?
          AND t.tournament_type = 'SummerPlayoffs'
          AND UPPER(t.status) = 'COMPLETED'
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    log::debug!("找到 {} 个已完成的夏季季后赛", playoffs.len());

    let mut legendary_teams: Vec<Team> = Vec::new();
    let mut challenger_teams: Vec<Team> = Vec::new();
    let mut qualifier_teams: Vec<Team> = Vec::new();

    for playoff in &playoffs {
        let playoff_id: i64 = playoff.get("id");

        // 获取总决赛结果 - 冠亚军
        let grand_final = sqlx::query(
            r#"
            SELECT winner_id, home_team_id, away_team_id
            FROM matches
            WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
            "#
        )
        .bind(playoff_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // 获取败者组决赛结果 - 季军
        let losers_final = sqlx::query(
            r#"
            SELECT winner_id, home_team_id, away_team_id
            FROM matches
            WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
            "#
        )
        .bind(playoff_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(gf) = grand_final {
            let winner_id: i64 = gf.get("winner_id");
            let home_id: i64 = gf.get("home_team_id");
            let away_id: i64 = gf.get("away_team_id");
            let runner_up_id = if winner_id == home_id { away_id } else { home_id };

            // 冠军 -> 传奇组
            if let Ok(team) = TeamRepository::get_by_id(&pool, winner_id as u64).await {
                legendary_teams.push(team);
            }
            // 亚军 -> 挑战者组
            if let Ok(team) = TeamRepository::get_by_id(&pool, runner_up_id as u64).await {
                challenger_teams.push(team);
            }
        }

        if let Some(lf) = losers_final {
            let winner_id: i64 = lf.get("winner_id");
            let home_id: i64 = lf.get("home_team_id");
            let away_id: i64 = lf.get("away_team_id");
            let third_place_id = if winner_id == home_id { away_id } else { home_id };

            // 季军 -> 资格赛组
            if let Ok(team) = TeamRepository::get_by_id(&pool, third_place_id as u64).await {
                qualifier_teams.push(team);
            }
        }
    }

    log::debug!("legendary={}, challenger={}, qualifier={}",
        legendary_teams.len(), challenger_teams.len(), qualifier_teams.len());

    if legendary_teams.len() != 4 || challenger_teams.len() != 4 || qualifier_teams.len() != 4 {
        return Ok(CommandResult::err(format!(
            "Not enough teams: legendary={}, challenger={}, qualifier={}",
            legendary_teams.len(), challenger_teams.len(), qualifier_teams.len()
        )));
    }

    // 5. 生成上海大师赛对阵（使用与MSI相同的赛制）
    let tournament_service = TournamentService::new();
    let matches = tournament_service.generate_msi_bracket(
        tournament_id,
        &legendary_teams,
        &challenger_teams,
        &qualifier_teams,
    );

    let match_count = matches.len();

    // 6. 保存比赛
    MatchRepository::create_batch(&pool, &save_id, &matches)
        .await
        .map_err(|e| format!("Failed to save matches: {}", e))?;

    log::debug!("成功重新生成 {} 场比赛", match_count);

    Ok(CommandResult::ok(match_count as u32))
}

/// 重新生成ICP洲际对抗赛对阵（重置赛事）
#[tauri::command]
pub async fn regenerate_icp_bracket(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<u32>, String> {
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

    // 获取赛季ID
    let tournament_row = sqlx::query("SELECT season_id FROM tournaments WHERE id = ?")
        .bind(tournament_id as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = tournament_row.get("season_id");

    log::debug!("重置ICP赛事: tournament_id={}, season_id={}", tournament_id, season_id);

    // 1. 删除现有比赛的详细数据
    sqlx::query("DELETE FROM game_player_performances WHERE game_id IN (SELECT id FROM match_games WHERE match_id IN (SELECT id FROM matches WHERE tournament_id = ?))")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .ok();

    sqlx::query("DELETE FROM match_games WHERE match_id IN (SELECT id FROM matches WHERE tournament_id = ?)")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .ok();

    // 2. 删除现有比赛
    sqlx::query("DELETE FROM matches WHERE tournament_id = ?")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // 2.1 删除现有荣誉和赛事结果（允许重新颁发）
    sqlx::query("DELETE FROM honors WHERE tournament_id = ? AND save_id = ?")
        .bind(tournament_id as i64)
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();

    sqlx::query("DELETE FROM tournament_results WHERE tournament_id = ? AND save_id = ?")
        .bind(tournament_id as i64)
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();

    sqlx::query("DELETE FROM player_tournament_stats WHERE tournament_id = ? AND save_id = ?")
        .bind(tournament_id as i64)
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();

    log::debug!("已清除荣誉、赛事结果和选手统计");

    // 3. 从夏季季后赛获取各赛区前4名
    let playoffs = sqlx::query(
        r#"
        SELECT t.id, t.region_id, r.short_name as region_code
        FROM tournaments t
        JOIN regions r ON t.region_id = r.id
        WHERE t.save_id = ? AND t.season_id = ?
        AND t.tournament_type = 'SummerPlayoffs'
        AND UPPER(t.status) = 'COMPLETED'
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    log::debug!("找到 {} 个已完成的夏季季后赛", playoffs.len());

    if playoffs.len() != 4 {
        return Ok(CommandResult::err(format!(
            "需要4个赛区的夏季季后赛结果，但只找到 {} 个",
            playoffs.len()
        )));
    }

    // 4. 获取各赛区前4名队伍
    let mut all_region_teams: Vec<Vec<Team>> = Vec::new();

    for playoff in &playoffs {
        let playoff_id: i64 = playoff.get("id");
        let region_code: String = playoff.get("region_code");
        let mut region_teams: Vec<Team> = Vec::new();

        // 获取总决赛结果 - 冠亚军
        let grand_final = sqlx::query(
            r#"
            SELECT winner_id, home_team_id, away_team_id
            FROM matches
            WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
            "#
        )
        .bind(playoff_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(gf) = grand_final {
            let winner_id: i64 = gf.get("winner_id");
            let home_id: i64 = gf.get("home_team_id");
            let away_id: i64 = gf.get("away_team_id");
            let loser_id = if winner_id == home_id { away_id } else { home_id };

            // 冠军
            if let Ok(t) = TeamRepository::get_by_id(&pool, winner_id as u64).await {
                region_teams.push(t);
            }
            // 亚军
            if let Ok(t) = TeamRepository::get_by_id(&pool, loser_id as u64).await {
                region_teams.push(t);
            }
        }

        // 获取季军赛结果 - 季殿军
        // 先尝试 THIRD_PLACE（单败淘汰赛），如果没有则从双败淘汰赛获取
        let third_place = sqlx::query(
            r#"
            SELECT winner_id, home_team_id, away_team_id
            FROM matches
            WHERE tournament_id = ? AND stage = 'THIRD_PLACE' AND UPPER(status) = 'COMPLETED'
            "#
        )
        .bind(playoff_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(tp) = third_place {
            let winner_id: i64 = tp.get("winner_id");
            let home_id: i64 = tp.get("home_team_id");
            let away_id: i64 = tp.get("away_team_id");
            let loser_id = if winner_id == home_id { away_id } else { home_id };

            // 季军
            if let Ok(t) = TeamRepository::get_by_id(&pool, winner_id as u64).await {
                region_teams.push(t);
            }
            // 殿军
            if let Ok(t) = TeamRepository::get_by_id(&pool, loser_id as u64).await {
                region_teams.push(t);
            }
        } else {
            // 双败淘汰赛：从 LOSERS_FINAL 获取季军（败者），从 LOSERS_R3 获取殿军（败者）
            // 季军 = LOSERS_FINAL 的败者（他输给了亚军）
            let losers_final = sqlx::query(
                r#"
                SELECT winner_id, home_team_id, away_team_id
                FROM matches
                WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
                "#
            )
            .bind(playoff_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(lf) = losers_final {
                let winner_id: i64 = lf.get("winner_id");
                let home_id: i64 = lf.get("home_team_id");
                let away_id: i64 = lf.get("away_team_id");
                let loser_id = if winner_id == home_id { away_id } else { home_id };

                // 季军 = LOSERS_FINAL 败者
                if let Ok(t) = TeamRepository::get_by_id(&pool, loser_id as u64).await {
                    region_teams.push(t);
                }
            }

            // 殿军 = LOSERS_R3 的败者（他输给了季军）
            let losers_r3 = sqlx::query(
                r#"
                SELECT winner_id, home_team_id, away_team_id
                FROM matches
                WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'
                "#
            )
            .bind(playoff_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(lr3) = losers_r3 {
                let winner_id: i64 = lr3.get("winner_id");
                let home_id: i64 = lr3.get("home_team_id");
                let away_id: i64 = lr3.get("away_team_id");
                let loser_id = if winner_id == home_id { away_id } else { home_id };

                // 殿军 = LOSERS_R3 败者
                if let Ok(t) = TeamRepository::get_by_id(&pool, loser_id as u64).await {
                    region_teams.push(t);
                }
            }
        }

        log::debug!("{} 赛区: {} 支队伍", region_code, region_teams.len());

        if region_teams.len() != 4 {
            return Ok(CommandResult::err(format!(
                "{} 赛区队伍不足4支 (found {})",
                region_code, region_teams.len()
            )));
        }

        all_region_teams.push(region_teams);
    }

    // 5. 重新生成ICP对阵
    let tournament_service = TournamentService::new();
    let matches = tournament_service.generate_icp_bracket(tournament_id, all_region_teams);
    let match_count = matches.len();

    // 6. 保存比赛
    MatchRepository::create_batch(&pool, &save_id, &matches)
        .await
        .map_err(|e| format!("Failed to save matches: {}", e))?;

    // 7. 重置赛事状态为进行中
    sqlx::query("UPDATE tournaments SET status = 'InProgress' WHERE id = ?")
        .bind(tournament_id as i64)
        .execute(&pool)
        .await
        .ok();

    log::debug!("成功重新生成 {} 场比赛", match_count);

    Ok(CommandResult::ok(match_count as u32))
}

/// 生成Super赛事第三阶段（冠军预备战）
#[tauri::command]
pub async fn generate_champion_prep_stage(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<u64>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let _save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(CommandResult::err("No save loaded")),
    };
    drop(current_save);

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 检查 PREP_WINNERS 是否已经有队伍（说明已生成过）
    let existing: Option<(i64, i64)> = sqlx::query_as(
        "SELECT home_team_id, away_team_id FROM matches WHERE tournament_id = ? AND stage = 'PREP_WINNERS' LIMIT 1"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((home, away)) = existing {
        if home > 0 && away > 0 {
            return Ok(CommandResult::err("第三阶段比赛已存在"));
        }
    }

    // 获取定位赛结果
    let positioning_matches: Vec<(i64, Option<i64>, i64, i64)> = sqlx::query_as(
        r#"
        SELECT id, winner_id, home_team_id, away_team_id
        FROM matches
        WHERE tournament_id = ? AND stage = 'CHALLENGER_POSITIONING' AND UPPER(status) = 'COMPLETED'
        ORDER BY match_order
        "#
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if positioning_matches.len() < 2 {
        return Ok(CommandResult::err("定位赛尚未全部完成"));
    }

    // 获取晋级赛结果
    let promotion_matches: Vec<(i64, Option<i64>, i64, i64)> = sqlx::query_as(
        r#"
        SELECT id, winner_id, home_team_id, away_team_id
        FROM matches
        WHERE tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND UPPER(status) = 'COMPLETED'
        ORDER BY match_order
        "#
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if promotion_matches.len() < 2 {
        return Ok(CommandResult::err("晋级赛尚未全部完成"));
    }

    // 提取定位赛胜者（进入胜者组）
    let pos1_winner = positioning_matches[0].1.ok_or("定位赛1没有胜者")? as u64;
    let pos2_winner = positioning_matches[1].1.ok_or("定位赛2没有胜者")? as u64;

    // 提取晋级赛胜者（进入败者组）
    let promo1_winner = promotion_matches[0].1.ok_or("晋级赛1没有胜者")? as u64;
    let promo2_winner = promotion_matches[1].1.ok_or("晋级赛2没有胜者")? as u64;

    let mut updated_match_ids: Vec<u64> = Vec::new();

    // 更新胜者组对决：定位赛胜者1 vs 定位赛胜者2
    let _result = sqlx::query(
        r#"
        UPDATE matches
        SET home_team_id = ?, away_team_id = ?, status = 'Scheduled'
        WHERE tournament_id = ? AND stage = 'PREP_WINNERS'
        "#
    )
    .bind(pos1_winner as i64)
    .bind(pos2_winner as i64)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 获取更新的比赛ID
    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'PREP_WINNERS'"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    // 更新败者组对决：晋级赛胜者1 vs 晋级赛胜者2
    sqlx::query(
        r#"
        UPDATE matches
        SET home_team_id = ?, away_team_id = ?, status = 'Scheduled'
        WHERE tournament_id = ? AND stage = 'PREP_LOSERS'
        "#
    )
    .bind(promo1_winner as i64)
    .bind(promo2_winner as i64)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'PREP_LOSERS'"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    // PREP_LOSERS_FINAL 保持队伍待定，等前两场完成后通过 advance_bracket 填充
    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'PREP_LOSERS_FINAL'"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    log::debug!("成功更新第三阶段比赛: {:?}", updated_match_ids);

    Ok(CommandResult::ok(updated_match_ids))
}

/// 生成Super赛事第四阶段（终极冠军赛）
#[tauri::command]
pub async fn generate_final_stage(
    state: State<'_, AppState>,
    tournament_id: u64,
) -> Result<CommandResult<Vec<u64>>, String> {
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
    drop(current_save);

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 检查 FINALS_R2 是否已经正确设置（away_team_id > 0 说明已生成过）
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT away_team_id FROM matches WHERE tournament_id = ? AND stage = 'FINALS_R2' AND match_order = 1 LIMIT 1"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some((away,)) = existing {
        if away > 0 {
            return Ok(CommandResult::err("第四阶段比赛已存在"));
        }
    }

    // === 获取 PREP 阶段晋级者 (只有胜者晋级，败者淘汰) ===

    // PREP_WINNERS 胜者 = 晋级者 1
    let prep_winners: Option<(i64, i64, Option<i64>)> = sqlx::query_as(
        "SELECT home_team_id, away_team_id, winner_id FROM matches WHERE tournament_id = ? AND stage = 'PREP_WINNERS' AND UPPER(status) = 'COMPLETED'"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let qualifier_1 = match prep_winners {
        Some((_, _, Some(winner))) => winner as u64,
        _ => return Ok(CommandResult::err("胜者组对决尚未完成")),
    };

    // PREP_LOSERS_FINAL 胜者 = 晋级者 2
    let prep_losers_final: Option<(i64, i64, Option<i64>)> = sqlx::query_as(
        "SELECT home_team_id, away_team_id, winner_id FROM matches WHERE tournament_id = ? AND stage = 'PREP_LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let qualifier_2 = match prep_losers_final {
        Some((_, _, Some(winner))) => winner as u64,
        _ => return Ok(CommandResult::err("败者组决赛尚未完成")),
    };

    log::debug!("PREP 晋级者: 1={}, 2={} (其他队伍被淘汰)",
             qualifier_1, qualifier_2);

    // === 获取传奇组队伍（年度积分前4名）===
    // 获取赛季ID
    let tournament_row = sqlx::query("SELECT season_id FROM tournaments WHERE id = ?")
        .bind(tournament_id as i64)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let season_id: i64 = tournament_row.get("season_id");

    // 获取年度积分排名
    let rankings = PointsRepository::get_season_rankings(&pool, &save_id, season_id as u64)
        .await
        .map_err(|e| e.to_string())?;

    if rankings.len() < 4 {
        return Ok(CommandResult::err(format!("年度积分排名队伍不足: {}", rankings.len())));
    }

    // 传奇组：年度积分前4名
    let legendary_1 = rankings[0].team_id; // 第1名
    let legendary_2 = rankings[1].team_id; // 第2名
    let legendary_3 = rankings[2].team_id; // 第3名
    let legendary_4 = rankings[3].team_id; // 第4名

    log::debug!("传奇组: 1={}, 2={}, 3={}, 4={}",
             legendary_1, legendary_2, legendary_3, legendary_4);

    let mut updated_match_ids: Vec<u64> = Vec::new();

    // === 更新 FINALS_R1 (home = 传奇组第4/3名，away = PREP 晋级者) ===
    // FINALS_R1 match 1: 传奇组第4名 vs PREP 晋级者 1
    sqlx::query(
        r#"
        UPDATE matches
        SET home_team_id = ?, away_team_id = ?, status = 'Scheduled'
        WHERE tournament_id = ? AND stage = 'FINALS_R1' AND match_order = 1
        "#
    )
    .bind(legendary_4 as i64)
    .bind(qualifier_1 as i64)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'FINALS_R1' AND match_order = 1"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    // FINALS_R1 match 2: 传奇组第3名 vs PREP 晋级者 2
    sqlx::query(
        r#"
        UPDATE matches
        SET home_team_id = ?, away_team_id = ?, status = 'Scheduled'
        WHERE tournament_id = ? AND stage = 'FINALS_R1' AND match_order = 2
        "#
    )
    .bind(legendary_3 as i64)
    .bind(qualifier_2 as i64)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'FINALS_R1' AND match_order = 2"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    // === 更新 FINALS_R2 (home = 传奇组第1/2名，away = 0 等待 FINALS_R1 胜者填充) ===
    // FINALS_R2 match 1: 传奇组第1名 vs FINALS_R1 match 1 胜者
    sqlx::query(
        r#"
        UPDATE matches
        SET home_team_id = ?, away_team_id = 0, status = 'Scheduled'
        WHERE tournament_id = ? AND stage = 'FINALS_R2' AND match_order = 1
        "#
    )
    .bind(legendary_1 as i64)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'FINALS_R2' AND match_order = 1"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    // FINALS_R2 match 2: 传奇组第2名 vs FINALS_R1 match 2 胜者
    sqlx::query(
        r#"
        UPDATE matches
        SET home_team_id = ?, away_team_id = 0, status = 'Scheduled'
        WHERE tournament_id = ? AND stage = 'FINALS_R2' AND match_order = 2
        "#
    )
    .bind(legendary_2 as i64)
    .bind(tournament_id as i64)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let match_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage = 'FINALS_R2' AND match_order = 2"
    )
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = match_id {
        updated_match_ids.push(id as u64);
    }

    // THIRD_PLACE, GRAND_FINAL 已在初始化时创建，通过 advance_bracket 填充
    let remaining_matches: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM matches WHERE tournament_id = ? AND stage IN ('THIRD_PLACE', 'GRAND_FINAL') ORDER BY stage, match_order"
    )
    .bind(tournament_id as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for (id,) in remaining_matches {
        updated_match_ids.push(id as u64);
    }

    log::debug!("成功更新第四阶段比赛: {:?}", updated_match_ids);

    Ok(CommandResult::ok(updated_match_ids))
}
