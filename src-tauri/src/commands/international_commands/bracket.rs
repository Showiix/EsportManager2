use crate::commands::save_commands::{AppState, CommandResult};
use sqlx::Row;
use tauri::State;

use super::helpers::*;
use super::icp::get_group_standings;
use super::{BracketInfo, MatchBracketInfo, StageInfo, TeamBracketInfo};

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
            // 注意: get_group_standings 返回的 group_name 已经被 replace("GROUP_", "") 处理过
            // FIGHTER_GROUP_A → FIGHTER_A, FIGHTER_GROUP_B → FIGHTER_B
            let fighter_a = standings.iter().find(|g| g.group_name == "FIGHTER_A");
            let fighter_b = standings.iter().find(|g| g.group_name == "FIGHTER_B");

            let fighter_a_winner = fighter_a.and_then(|g| g.teams.first()).map(|t| t.team_id);
            let fighter_b_winner = fighter_b.and_then(|g| g.teams.first()).map(|t| t.team_id);

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

            // 如果 Fighter 积分榜数据完整，填入晋级赛的 home_team_id
            if let (Some(fa), Some(fb)) = (fighter_a_winner, fighter_b_winner) {
                sqlx::query(
                    r#"
                    UPDATE matches
                    SET home_team_id = ?
                    WHERE save_id = ? AND tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = 1
                    AND (home_team_id IS NULL OR home_team_id = 0)
                    "#
                )
                .bind(fa as i64)
                .bind(&save_id)
                .bind(tournament_id as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

                sqlx::query(
                    r#"
                    UPDATE matches
                    SET home_team_id = ?
                    WHERE save_id = ? AND tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = 2
                    AND (home_team_id IS NULL OR home_team_id = 0)
                    "#
                )
                .bind(fb as i64)
                .bind(&save_id)
                .bind(tournament_id as i64)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

                log::debug!("填入Fighter胜者: A={}, B={}", fa, fb);
            } else {
                log::debug!("Fighter积分榜不完整，跳过home_team_id填充，尝试补填定位赛败者");
            }

            // 补填定位赛败者到晋级赛的 away_team_id（如果定位赛已完成但 advance_bracket 未正确执行）
            let positioning_matches = sqlx::query(
                r#"
                SELECT match_order, home_team_id, away_team_id, winner_id
                FROM matches
                WHERE save_id = ? AND tournament_id = ? AND stage = 'CHALLENGER_POSITIONING'
                AND (status = 'Completed' OR UPPER(status) = 'COMPLETED')
                ORDER BY match_order
                "#
            )
            .bind(&save_id)
            .bind(tournament_id as i64)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;

            for row in &positioning_matches {
                let match_order: i64 = row.get::<Option<i64>, _>("match_order").unwrap_or(0);
                let home_team_id: Option<i64> = row.get("home_team_id");
                let away_team_id: Option<i64> = row.get("away_team_id");
                let winner_id: Option<i64> = row.get("winner_id");

                if let (Some(home), Some(away), Some(winner)) = (home_team_id, away_team_id, winner_id) {
                    let loser_id = if winner == home { away } else { home };

                    // 将定位赛败者填入对应晋级赛的 away_team_id
                    sqlx::query(
                        r#"
                        UPDATE matches
                        SET away_team_id = ?
                        WHERE save_id = ? AND tournament_id = ? AND stage = 'CHALLENGER_PROMOTION'
                        AND match_order = ? AND (away_team_id IS NULL OR away_team_id = 0)
                        "#
                    )
                    .bind(loser_id)
                    .bind(&save_id)
                    .bind(tournament_id as i64)
                    .bind(match_order)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;

                    log::debug!("补填晋级赛{} away_team_id={}", match_order, loser_id);
                }
            }

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
