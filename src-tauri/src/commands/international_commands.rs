use crate::commands::save_commands::{AppState, CommandResult};
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

    // 获取阶段统计
    let stage_rows = sqlx::query(
        r#"
        SELECT stage,
               COUNT(*) as total_matches,
               SUM(CASE WHEN status = 'Completed' THEN 1 ELSE 0 END) as completed_matches,
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
               hr.code as home_region, ar.code as away_region
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

    // 获取已完成比赛信息
    let match_row = sqlx::query(
        "SELECT stage, match_order FROM matches WHERE id = ? AND tournament_id = ?"
    )
    .bind(completed_match_id as i64)
    .bind(tournament_id as i64)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (stage, match_order): (String, i64) = match match_row {
        Some(r) => (r.get("stage"), r.get::<Option<i64>, _>("match_order").unwrap_or(0)),
        None => return Ok(CommandResult::err("Match not found")),
    };

    // 根据阶段确定下一场比赛
    let next_matches = determine_next_matches(&stage, match_order as u32);
    let mut updated_match_ids = Vec::new();

    for (next_stage, next_order, is_home) in next_matches {
        // 查找下一场比赛
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

            // 更新下一场比赛的队伍
            let update_field = if is_home { "home_team_id" } else { "away_team_id" };
            sqlx::query(&format!("UPDATE matches SET {} = ? WHERE id = ?", update_field))
                .bind(winner_id as i64)
                .bind(next_id)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

            updated_match_ids.push(next_id as u64);
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

        if status == "Completed" {
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
        .filter(|r| r.get::<String, _>("status") == "Completed")
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
            let is_qualified = wins >= 3;
            let is_eliminated = losses >= 3;
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

    let _pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 获取瑞士轮状态
    drop(guard);
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
                    tournament_id, stage, match_order, format,
                    home_team_id, away_team_id, home_score, away_score, status
                ) VALUES (?, ?, ?, 'Bo1', ?, ?, 0, 0, 'Scheduled')
                RETURNING id
                "#,
            )
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
        "CHALLENGER_R1" => {
            if match_order <= 2 {
                vec![("LOSERS_R2".to_string(), match_order, true)]
            } else {
                vec![]
            }
        }
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
        "SEMI_FINAL" => vec![
            ("GRAND_FINAL".to_string(), 1, match_order == 1),
            ("THIRD_PLACE".to_string(), 1, match_order == 1),
        ],

        _ => vec![],
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
