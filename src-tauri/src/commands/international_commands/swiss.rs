use crate::commands::save_commands::{AppState, CommandResult};
use sqlx::Row;
use tauri::State;

use super::{SwissRoundStatus, SwissTeamStatus};

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
