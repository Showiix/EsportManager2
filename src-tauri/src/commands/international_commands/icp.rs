use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::{MatchRepository, TeamRepository};
use crate::models::Team;
use crate::services::TournamentService;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

use super::helpers::*;

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
