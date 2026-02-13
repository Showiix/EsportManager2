use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::MatchRepository;
use crate::models::Team;
use crate::services::TournamentService;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

use super::helpers::*;

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
