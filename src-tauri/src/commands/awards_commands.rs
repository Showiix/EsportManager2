//! 年度颁奖典礼命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::SaveRepository;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 年度最佳阵容选手信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllProPlayer {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,
    pub yearly_score: f64,
    pub avg_impact: f64,
    pub games_played: u32,
}

/// 年度Top20选手信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Top20Player {
    pub rank: u32,
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,
    pub yearly_score: f64,
    pub avg_impact: f64,
    pub games_played: u32,
}

/// 年度最佳新秀信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RookiePlayer {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,
    pub age: u8,
    pub yearly_score: f64,
    pub avg_impact: f64,
    pub games_played: u32,
}

/// 年度颁奖数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualAwardsData {
    pub season_id: u64,
    /// 年度Top20
    pub top20: Vec<Top20Player>,
    /// 年度最佳阵容（各位置第一）
    pub all_pro_team: Vec<AllProPlayer>,
    /// 年度最佳新秀
    pub rookie_of_the_year: Option<RookiePlayer>,
    /// 是否已颁发过奖项
    pub already_awarded: bool,
}

/// 获取年度颁奖数据（纯查询，不写入）
#[tauri::command]
pub async fn get_annual_awards_data(
    state: State<'_, AppState>,
    season_id: Option<u64>,
) -> Result<CommandResult<AnnualAwardsData>, String> {
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
    let current_season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as u64
        }
    };

    // 检查是否已颁发过年度奖项
    let already_awarded = check_annual_awards_exist(&pool, &save_id, current_season).await;

    // 获取年度Top20（按 yearly_top_score 排序）
    let top20 = get_top20_players(&pool, &save_id, current_season).await?;

    // 获取各位置最佳
    let all_pro_team = get_all_pro_team(&pool, &save_id, current_season).await?;

    // 获取年度最佳新秀（20岁及以下）
    let rookie_of_the_year = get_rookie_of_the_year(&pool, &save_id, current_season).await?;

    Ok(CommandResult::ok(AnnualAwardsData {
        season_id: current_season,
        top20,
        all_pro_team,
        rookie_of_the_year,
        already_awarded,
    }))
}

/// 检查是否已颁发过年度奖项
async fn check_annual_awards_exist(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> bool {
    let result = sqlx::query(
        r#"
        SELECT COUNT(*) as count FROM honors
        WHERE save_id = ? AND season_id = ? AND honor_type LIKE 'ANNUAL%'
        "#
    )
    .bind(save_id)
    .bind(season_id as i64)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => row.get::<i64, _>("count") > 0,
        Err(_) => false,
    }
}

/// 获取年度Top20选手
async fn get_top20_players(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<Vec<Top20Player>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            pss.player_id,
            pss.player_name,
            pss.team_id,
            COALESCE(t.name, '未知') as team_name,
            pss.position,
            pss.yearly_top_score,
            pss.avg_impact,
            COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
        FROM player_season_stats pss
        LEFT JOIN teams t ON pss.team_id = t.id
        LEFT JOIN (
            SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
            FROM game_player_performances gpp
            JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
            JOIN matches m ON mg.match_id = m.id
            JOIN tournaments t ON m.tournament_id = t.id
            WHERE gpp.save_id = ? AND t.season_id = ?
            GROUP BY gpp.save_id, gpp.player_id
        ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
        WHERE pss.save_id = ? AND pss.season_id = ?
          AND COALESCE(gpp_count.real_games_played, pss.games_played) >= 10
        ORDER BY pss.yearly_top_score DESC
        LIMIT 20
        "#
    )
    .bind(save_id)
    .bind(season_id as i64)
    .bind(save_id)
    .bind(season_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut players = Vec::new();
    for (idx, row) in rows.iter().enumerate() {
        players.push(Top20Player {
            rank: (idx + 1) as u32,
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get::<String, _>("player_name"),
            team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: row.get::<String, _>("team_name"),
            position: row.get::<String, _>("position"),
            yearly_score: row.get::<f64, _>("yearly_top_score"),
            avg_impact: row.get::<f64, _>("avg_impact"),
            games_played: row.get::<i64, _>("games_played") as u32,
        });
    }

    Ok(players)
}

/// 获取年度最佳阵容（各位置第一名）
async fn get_all_pro_team(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<Vec<AllProPlayer>, String> {
    let positions = vec!["TOP", "JUG", "MID", "ADC", "SUP"];
    let mut all_pro = Vec::new();

    for position in positions {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                pss.avg_impact,
                COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN (
                SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments tr ON m.tournament_id = tr.id
                WHERE gpp.save_id = ? AND tr.season_id = ?
                GROUP BY gpp.save_id, gpp.player_id
            ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.position = ?
              AND COALESCE(gpp_count.real_games_played, pss.games_played) >= 10
            ORDER BY pss.yearly_top_score DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(save_id)
        .bind(season_id as i64)
        .bind(position)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(row) = row {
            all_pro.push(AllProPlayer {
                player_id: row.get::<i64, _>("player_id") as u64,
                player_name: row.get::<String, _>("player_name"),
                team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
                team_name: row.get::<String, _>("team_name"),
                position: row.get::<String, _>("position"),
                yearly_score: row.get::<f64, _>("yearly_top_score"),
                avg_impact: row.get::<f64, _>("avg_impact"),
                games_played: row.get::<i64, _>("games_played") as u32,
            });
        }
    }

    Ok(all_pro)
}

/// 获取年度最佳新秀（20岁及以下，IM得分最高）
async fn get_rookie_of_the_year(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<Option<RookiePlayer>, String> {
    let row = sqlx::query(
        r#"
        SELECT
            pss.player_id,
            pss.player_name,
            pss.team_id,
            COALESCE(t.name, '未知') as team_name,
            pss.position,
            p.age,
            pss.yearly_top_score,
            pss.avg_impact,
            COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
        FROM player_season_stats pss
        JOIN players p ON pss.player_id = p.id
        LEFT JOIN teams t ON pss.team_id = t.id
        LEFT JOIN (
            SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
            FROM game_player_performances gpp
            JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
            JOIN matches m ON mg.match_id = m.id
            JOIN tournaments tr ON m.tournament_id = tr.id
            WHERE gpp.save_id = ? AND tr.season_id = ?
            GROUP BY gpp.save_id, gpp.player_id
        ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
        WHERE pss.save_id = ? AND pss.season_id = ? AND p.age <= 20
          AND COALESCE(gpp_count.real_games_played, pss.games_played) >= 10
        ORDER BY pss.yearly_top_score DESC
        LIMIT 1
        "#
    )
    .bind(save_id)
    .bind(season_id as i64)
    .bind(save_id)
    .bind(season_id as i64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|r| RookiePlayer {
        player_id: r.get::<i64, _>("player_id") as u64,
        player_name: r.get::<String, _>("player_name"),
        team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
        team_name: r.get::<String, _>("team_name"),
        position: r.get::<String, _>("position"),
        age: r.get::<i64, _>("age") as u8,
        yearly_score: r.get::<f64, _>("yearly_top_score"),
        avg_impact: r.get::<f64, _>("avg_impact"),
        games_played: r.get::<i64, _>("games_played") as u32,
    }))
}
