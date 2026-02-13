use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{MatchRepository, TeamRepository};
use crate::models::Team;
use crate::services::TournamentService;
use sqlx::Row;
use tauri::State;

use super::helpers::*;
use super::MsiTeamGroups;

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
