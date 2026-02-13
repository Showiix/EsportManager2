use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::PointsRepository;
use crate::services::TournamentService;
use sqlx::Row;
use tauri::State;

use super::helpers::*;

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
