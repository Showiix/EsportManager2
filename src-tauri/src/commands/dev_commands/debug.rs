use sqlx::Row;
use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::SaveRepository;
use super::{DevCommandResult, ConsistencyCheckResult, ConsistencyIssue, IncompleteMatchInfo, GameStatusSummary};

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_check_data_consistency(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<ConsistencyCheckResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let mut issues = Vec::new();
    let mut total_checks = 0;
    let mut passed = 0;

    total_checks += 1;
    let inconsistent_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM (
            SELECT pss.player_id, pss.games_played as recorded,
                   COALESCE(gpp.real_count, 0) as actual
            FROM player_season_stats pss
            LEFT JOIN (
                SELECT gpp.player_id, COUNT(*) as real_count
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments t ON m.tournament_id = t.id
                WHERE gpp.save_id = ?
                GROUP BY gpp.player_id
            ) gpp ON pss.player_id = gpp.player_id
            WHERE pss.save_id = ? AND pss.games_played != COALESCE(gpp.real_count, 0)
        )
        "#
    )
    .bind(&save_id)
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    if inconsistent_count.0 > 0 {
        issues.push(ConsistencyIssue {
            category: "选手统计".to_string(),
            description: format!("{} 名选手的场次记录与实际不符", inconsistent_count.0),
            severity: "warning".to_string(),
        });
    } else {
        passed += 1;
    }

    total_checks += 1;
    let invalid_matches: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM matches WHERE save_id = ? AND status = 'Completed' AND (home_score = 0 AND away_score = 0)"
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    if invalid_matches.0 > 0 {
        issues.push(ConsistencyIssue {
            category: "比赛数据".to_string(),
            description: format!("{} 场已完成比赛没有比分记录", invalid_matches.0),
            severity: "error".to_string(),
        });
    } else {
        passed += 1;
    }

    total_checks += 1;
    let incomplete_teams: Vec<(i64, String, i64)> = sqlx::query_as(
        r#"
        SELECT t.id, t.name, COUNT(p.id) as starter_count
        FROM teams t
        LEFT JOIN players p ON t.id = p.team_id AND p.is_starter = 1 AND p.status = 'Active'
        WHERE t.save_id = ?
        GROUP BY t.id
        HAVING starter_count < 5
        "#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    if !incomplete_teams.is_empty() {
        for (_, name, count) in &incomplete_teams {
            issues.push(ConsistencyIssue {
                category: "队伍阵容".to_string(),
                description: format!("{} 只有 {} 名首发选手", name, count),
                severity: "warning".to_string(),
            });
        }
    } else {
        passed += 1;
    }

    total_checks += 1;
    let empty_tournaments: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM tournaments t
        WHERE t.save_id = ? AND t.status != 'Upcoming'
        AND NOT EXISTS (SELECT 1 FROM matches m WHERE m.tournament_id = t.id)
        "#
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    if empty_tournaments.0 > 0 {
        issues.push(ConsistencyIssue {
            category: "赛事数据".to_string(),
            description: format!("{} 个非待开始赛事没有比赛", empty_tournaments.0),
            severity: "warning".to_string(),
        });
    } else {
        passed += 1;
    }

    let result = ConsistencyCheckResult {
        total_checks,
        passed,
        failed: total_checks - passed,
        issues,
    };

    let message = if result.failed == 0 {
        "数据一致性检查通过".to_string()
    } else {
        format!("发现 {} 个问题", result.issues.len())
    };

    Ok(DevCommandResult::ok(result, message))
}

#[tauri::command]
pub async fn dev_check_incomplete_matches(
    state: State<'_, AppState>,
    tournament_type: Option<String>,
) -> Result<DevCommandResult<Vec<IncompleteMatchInfo>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let query = if let Some(ref t_type) = tournament_type {
        sqlx::query(
            r#"
            SELECT m.id, m.stage, m.status, m.home_team_id, m.away_team_id,
                   t.name as tournament_name, t.tournament_type,
                   ht.name as home_name, at.name as away_name
            FROM matches m
            JOIN tournaments t ON m.tournament_id = t.id
            LEFT JOIN teams ht ON m.home_team_id = ht.id
            LEFT JOIN teams at ON m.away_team_id = at.id
            WHERE t.save_id = ? AND UPPER(m.status) NOT IN ('COMPLETED', 'CANCELLED')
              AND t.tournament_type = ?
            ORDER BY t.name, m.stage, m.id
            "#
        )
        .bind(&save_id)
        .bind(t_type)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT m.id, m.stage, m.status, m.home_team_id, m.away_team_id,
                   t.name as tournament_name, t.tournament_type,
                   ht.name as home_name, at.name as away_name
            FROM matches m
            JOIN tournaments t ON m.tournament_id = t.id
            LEFT JOIN teams ht ON m.home_team_id = ht.id
            LEFT JOIN teams at ON m.away_team_id = at.id
            WHERE t.save_id = ? AND UPPER(m.status) NOT IN ('COMPLETED', 'CANCELLED')
            ORDER BY t.name, m.stage, m.id
            "#
        )
        .bind(&save_id)
        .fetch_all(&pool)
        .await
    };

    let rows = query.map_err(|e| e.to_string())?;

    let mut matches: Vec<IncompleteMatchInfo> = Vec::new();
    for row in rows {
        matches.push(IncompleteMatchInfo {
            match_id: row.get::<i64, _>("id") as u64,
            stage: row.get("stage"),
            status: row.get("status"),
            tournament_name: row.get("tournament_name"),
            tournament_type: row.get("tournament_type"),
            home_team: row.get::<Option<String>, _>("home_name"),
            away_team: row.get::<Option<String>, _>("away_name"),
        });
    }

    let count = matches.len();
    Ok(DevCommandResult::ok(
        matches,
        format!("找到 {} 场未完成的比赛", count),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_get_game_status(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<GameStatusSummary>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let save = SaveRepository::get_by_id(&pool, &save_id)
        .await
        .map_err(|e| e.to_string())?;

    let team_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    let player_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM players WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    let tournament_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tournaments WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    let match_stats: (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN status = 'Completed' THEN 1 ELSE 0 END) as completed,
            SUM(CASE WHEN status = 'Scheduled' THEN 1 ELSE 0 END) as scheduled
        FROM matches WHERE save_id = ?
        "#
    )
    .bind(&save_id)
    .fetch_one(&pool)
    .await
    .unwrap_or((0, 0, 0));

    let honor_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM honors WHERE save_id = ?")
        .bind(&save_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    Ok(DevCommandResult::ok(
        GameStatusSummary {
            current_season: save.current_season,
            current_phase: format!("{:?}", save.current_phase),
            phase_completed: save.phase_completed,
            team_count: team_count.0 as i32,
            player_count: player_count.0 as i32,
            tournament_count: tournament_count.0 as i32,
            total_matches: match_stats.0 as i32,
            completed_matches: match_stats.1 as i32,
            scheduled_matches: match_stats.2 as i32,
            honor_count: honor_count.0 as i32,
        },
        "获取游戏状态成功",
    ))
}
