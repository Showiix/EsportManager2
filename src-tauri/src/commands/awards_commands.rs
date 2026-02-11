//! 年度颁奖典礼命令

use crate::commands::save_commands::{AppState, CommandResult};
use crate::db::SaveRepository;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

/// 五维评分维度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDimensions {
    pub impact_norm: f64,
    pub performance_norm: f64,
    pub stability_norm: f64,
    pub appearance_norm: f64,
    pub honor_norm: f64,
    pub big_stage_norm: f64,
}

/// 选手评语
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCommentary {
    pub description: String,
    pub tags: Vec<String>,
}

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
    pub tier: u8,
    pub commentary: PlayerCommentary,
}

/// 选手单赛事表现明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentDetail {
    pub tournament_name: String,
    pub tournament_type: String,
    pub games_played: u32,
    pub avg_impact: f64,
    pub avg_performance: f64,
    pub best_performance: f64,
    pub mvp_count: u32,
    pub weight: f64,
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
    pub age: u8,
    pub avg_performance: f64,
    pub best_performance: f64,
    pub consistency_score: f64,
    pub champion_bonus: f64,
    pub international_titles: i32,
    pub regional_titles: i32,
    pub dimensions: ScoreDimensions,
    pub commentary: PlayerCommentary,
    pub tournament_details: Vec<TournamentDetail>,
    pub big_stage_score: f64,
    pub has_international: bool,
}

/// 特别奖选手信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialAwardPlayer {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,
    pub age: u8,
    pub score: f64,
    pub games_played: u32,
    pub commentary: PlayerCommentary,
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
    pub dimensions: ScoreDimensions,
    pub commentary: PlayerCommentary,
}

/// 年度颁奖数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualAwardsData {
    pub season_id: u64,
    /// 年度Top20
    pub top20: Vec<Top20Player>,
    /// 年度最佳阵容一阵
    pub all_pro_1st: Vec<AllProPlayer>,
    /// 年度最佳阵容二阵
    pub all_pro_2nd: Vec<AllProPlayer>,
    /// 年度最佳阵容三阵
    pub all_pro_3rd: Vec<AllProPlayer>,
    /// 最稳定选手
    pub most_consistent: Option<SpecialAwardPlayer>,
    /// 最具统治力选手
    pub most_dominant: Option<SpecialAwardPlayer>,
    /// 年度最佳新秀
    pub rookie_of_the_year: Option<RookiePlayer>,
    /// 是否已颁发过奖项
    pub already_awarded: bool,
}

/// 计算六维归一化维度
fn calculate_dimensions(
    avg_impact: f64,
    avg_performance: f64,
    consistency_score: f64,
    games_played: u32,
    champion_bonus: f64,
    big_stage_score: f64,
    has_international: bool,
) -> ScoreDimensions {
    let honor_raw = (champion_bonus * 6.67).clamp(0.0, 100.0);
    let honor_discount = if avg_impact >= 0.0 {
        1.0
    } else {
        ((avg_impact + 5.0) / 5.0).clamp(0.2, 1.0)
    };
    // 没参加国际赛：big_stage_norm 直接归零
    let big_stage_norm = if has_international {
        ((big_stage_score + 5.0) * 5.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    ScoreDimensions {
        impact_norm: ((avg_impact + 5.0) * 5.0).clamp(0.0, 100.0),
        performance_norm: ((avg_performance - 50.0) * 2.0).clamp(0.0, 100.0),
        stability_norm: consistency_score.clamp(0.0, 100.0),
        appearance_norm: (games_played as f64 * 0.83).clamp(0.0, 100.0),
        honor_norm: honor_raw * honor_discount,
        big_stage_norm,
    }
}

#[allow(dead_code)]
fn calculate_final_yearly_score(dims: &ScoreDimensions, has_international: bool) -> f64 {
    let base = dims.impact_norm * 0.40
        + dims.performance_norm * 0.18
        + dims.stability_norm * 0.12
        + dims.appearance_norm * 0.10
        + dims.honor_norm * 0.05
        + dims.big_stage_norm * 0.15;
    if has_international { base } else { base * 0.95 }
}

/// 基于数据生成评语和标签
fn generate_commentary(
    avg_impact: f64,
    avg_performance: f64,
    best_performance: f64,
    consistency_score: f64,
    champion_bonus: f64,
    games_played: u32,
    age: u8,
    rank: Option<u32>,
) -> PlayerCommentary {
    let mut tags = Vec::new();
    let mut description = String::new();

    // 高影响力 + 高稳定
    if avg_impact > 5.0 && consistency_score > 80.0 {
        description = "全年表现如磐石般稳定，场均影响力排名前列".to_string();
        tags.push("稳定核心".to_string());
    }
    // 高影响力 + 低稳定
    else if avg_impact > 5.0 && consistency_score < 60.0 {
        description = "巅峰时刻无人可挡，低谷时也让人揪心".to_string();
        tags.push("大心脏".to_string());
    }
    // 普通高影响力
    else if avg_impact > 3.0 {
        description = "赛季整体表现出色，是队伍不可或缺的核心".to_string();
        tags.push("核心选手".to_string());
    }

    // 冠军加成
    if champion_bonus >= 6.0 {
        if description.is_empty() {
            description = "冠军荣耀加身，赛场统治力无可匹敌".to_string();
        }
        tags.push("冠军基因".to_string());
    } else if champion_bonus >= 3.0 {
        tags.push("冠军成员".to_string());
    }

    // 年龄标签
    if age <= 20 {
        if let Some(r) = rank {
            if r <= 10 {
                description = format!("年仅{}岁便跻身年度Top{}，未来不可限量", age, r);
                tags.push("年少成名".to_string());
            }
        }
        tags.push("新星".to_string());
    } else if age >= 30 {
        if description.is_empty() {
            description = "老将弥坚，用经验弥补了体能的下滑".to_string();
        }
        tags.push("老将风范".to_string());
    }

    // 出场数
    if games_played >= 100 {
        tags.push("铁人".to_string());
    }

    // 稳定性
    if consistency_score > 85.0 {
        tags.push("稳如泰山".to_string());
    }

    // 巅峰发挥
    if best_performance > 90.0 {
        tags.push("超级carry".to_string());
    }

    // 高发挥均值
    if avg_performance > 75.0 {
        tags.push("高水准".to_string());
    }

    if description.is_empty() {
        description = "赛季表现可圈可点，用实力证明了自己的价值".to_string();
    }

    PlayerCommentary { description, tags }
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

    log::info!("get_annual_awards_data: season_id={:?}, resolved_season={}", season_id, current_season);

    // 检查是否已颁发过年度奖项
    let already_awarded = check_annual_awards_exist(&pool, &save_id, current_season).await;

    // 获取年度Top20（按 yearly_top_score 排序）
    let top20 = get_top20_players(&pool, &save_id, current_season).await?;
    log::info!("get_annual_awards_data: top20 count={}", top20.len());

    // 获取三阵
    let (all_pro_1st, all_pro_2nd, all_pro_3rd) = get_all_pro_teams(&pool, &save_id, current_season).await?;

    // 获取最稳定选手
    let most_consistent = get_most_consistent(&pool, &save_id, current_season).await?;

    // 获取最具统治力选手
    let most_dominant = get_most_dominant(&pool, &save_id, current_season).await?;

    // 获取年度最佳新秀（20岁及以下）
    let rookie_of_the_year = get_rookie_of_the_year(&pool, &save_id, current_season).await?;

    Ok(CommandResult::ok(AnnualAwardsData {
        season_id: current_season,
        top20,
        all_pro_1st,
        all_pro_2nd,
        all_pro_3rd,
        most_consistent,
        most_dominant,
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
            pss.avg_performance,
            pss.best_performance,
            pss.consistency_score,
            pss.champion_bonus,
            pss.international_titles,
            pss.regional_titles,
            COALESCE(p.age, 0) as age,
            COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
        FROM player_season_stats pss
        LEFT JOIN teams t ON pss.team_id = t.id
        LEFT JOIN players p ON pss.player_id = p.id
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
          AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
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
        let rank = (idx + 1) as u32;
        let avg_impact = row.get::<f64, _>("avg_impact");
        let avg_performance = row.get::<f64, _>("avg_performance");
        let best_performance = row.get::<f64, _>("best_performance");
        let consistency_score = row.get::<f64, _>("consistency_score");
        let champion_bonus = row.get::<f64, _>("champion_bonus");
        let games_played = row.get::<i64, _>("games_played") as u32;
        let age = row.get::<i64, _>("age") as u8;
        let international_titles = row.get::<i32, _>("international_titles");
        let regional_titles = row.get::<i32, _>("regional_titles");

        let dimensions = calculate_dimensions(
            avg_impact, avg_performance, consistency_score, games_played, champion_bonus, 0.0, true,
        );
        let commentary = generate_commentary(
            avg_impact, avg_performance, best_performance, consistency_score,
            champion_bonus, games_played, age, Some(rank),
        );

        players.push(Top20Player {
            rank,
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get::<String, _>("player_name"),
            team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: row.get::<String, _>("team_name"),
            position: row.get::<String, _>("position"),
            yearly_score: row.get::<f64, _>("yearly_top_score"),
            avg_impact,
            games_played,
            age,
            avg_performance,
            best_performance,
            consistency_score,
            champion_bonus,
            international_titles,
            regional_titles,
            dimensions,
            commentary,
            tournament_details: Vec::new(),
            big_stage_score: 0.0,
            has_international: true,
        });
    }

    let all_breakdowns = get_all_tournament_breakdowns(pool, save_id, season_id).await.unwrap_or_default();
    for player in &mut players {
        if let Some((details, big_stage, has_intl)) = all_breakdowns.get(&(player.player_id as i64)) {
            player.tournament_details = details.clone();
            player.big_stage_score = *big_stage;
            player.has_international = *has_intl;
            player.dimensions = calculate_dimensions(
                player.avg_impact, player.avg_performance, player.consistency_score,
                player.games_played, player.champion_bonus, *big_stage, *has_intl,
            );
            if !has_intl {
                player.commentary.tags.push("未参加国际赛".to_string());
            }
        }
    }

    players.sort_by(|a, b| b.yearly_score.partial_cmp(&a.yearly_score).unwrap_or(std::cmp::Ordering::Equal));
    for (i, player) in players.iter_mut().enumerate() {
        player.rank = (i + 1) as u32;
    }

    Ok(players)
}

/// 获取年度最佳阵容三阵（每位置Top3）
async fn get_all_pro_teams(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<(Vec<AllProPlayer>, Vec<AllProPlayer>, Vec<AllProPlayer>), String> {
    let positions = vec!["TOP", "JUG", "MID", "ADC", "SUP"];
    let mut first_team = Vec::new();
    let mut second_team = Vec::new();
    let mut third_team = Vec::new();

    for position in positions {
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
                pss.avg_performance,
                pss.best_performance,
                pss.consistency_score,
                pss.champion_bonus,
                COALESCE(p.age, 0) as age,
                COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            LEFT JOIN (
                SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments tr ON m.tournament_id = tr.id
                WHERE gpp.save_id = ? AND tr.season_id = ?
                GROUP BY gpp.save_id, gpp.player_id
            ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
            WHERE pss.save_id = ? AND pss.season_id = ? AND UPPER(pss.position) = UPPER(?)
              AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
            ORDER BY pss.yearly_top_score DESC
            LIMIT 3
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(save_id)
        .bind(season_id as i64)
        .bind(position)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        for (tier_idx, row) in rows.iter().enumerate() {
            let tier = (tier_idx + 1) as u8;
            let avg_impact = row.get::<f64, _>("avg_impact");
            let avg_performance = row.get::<f64, _>("avg_performance");
            let best_performance = row.get::<f64, _>("best_performance");
            let consistency_score = row.get::<f64, _>("consistency_score");
            let champion_bonus = row.get::<f64, _>("champion_bonus");
            let games_played = row.get::<i64, _>("games_played") as u32;
            let age = row.get::<i64, _>("age") as u8;

            let commentary = generate_commentary(
                avg_impact, avg_performance, best_performance, consistency_score,
                champion_bonus, games_played, age, None,
            );

            let player = AllProPlayer {
                player_id: row.get::<i64, _>("player_id") as u64,
                player_name: row.get::<String, _>("player_name"),
                team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
                team_name: row.get::<String, _>("team_name"),
                position: row.get::<String, _>("position"),
                yearly_score: row.get::<f64, _>("yearly_top_score"),
                avg_impact,
                games_played,
                tier,
                commentary,
            };

            match tier {
                1 => first_team.push(player),
                2 => second_team.push(player),
                3 => third_team.push(player),
                _ => {}
            }
        }
    }

    Ok((first_team, second_team, third_team))
}

/// 获取最稳定选手（consistency_score 最高，>=30场）
async fn get_most_consistent(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<Option<SpecialAwardPlayer>, String> {
    let row = sqlx::query(
        r#"
        SELECT
            pss.player_id,
            pss.player_name,
            pss.team_id,
            COALESCE(t.name, '未知') as team_name,
            pss.position,
            COALESCE(p.age, 0) as age,
            pss.consistency_score,
            pss.avg_impact,
            pss.avg_performance,
            pss.best_performance,
            pss.champion_bonus,
            COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
        FROM player_season_stats pss
        LEFT JOIN teams t ON pss.team_id = t.id
        LEFT JOIN players p ON pss.player_id = p.id
        LEFT JOIN (
            SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
            FROM game_player_performances gpp
            JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
            JOIN matches m ON mg.match_id = m.id
            JOIN tournaments tr ON m.tournament_id = tr.id
            WHERE gpp.save_id = ? AND tr.season_id = ?
            GROUP BY gpp.save_id, gpp.player_id
        ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
        WHERE pss.save_id = ? AND pss.season_id = ?
          AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
        ORDER BY pss.consistency_score DESC
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

    Ok(row.map(|r| {
        let games_played = r.get::<i64, _>("games_played") as u32;
        let age = r.get::<i64, _>("age") as u8;
        let consistency_score = r.get::<f64, _>("consistency_score");
        let avg_impact = r.get::<f64, _>("avg_impact");
        let avg_performance = r.get::<f64, _>("avg_performance");
        let best_performance = r.get::<f64, _>("best_performance");
        let champion_bonus = r.get::<f64, _>("champion_bonus");

        let mut commentary = generate_commentary(
            avg_impact, avg_performance, best_performance, consistency_score,
            champion_bonus, games_played, age, None,
        );
        commentary.description = format!(
            "稳定性评分高达{:.1}，全赛季表现几乎没有波动，是队伍最可靠的基石",
            consistency_score,
        );
        if !commentary.tags.contains(&"稳如泰山".to_string()) {
            commentary.tags.insert(0, "稳如泰山".to_string());
        }

        SpecialAwardPlayer {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            age,
            score: consistency_score,
            games_played,
            commentary,
        }
    }))
}

/// 获取最具统治力选手（dominance_score 最高，>=20场）
async fn get_most_dominant(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<Option<SpecialAwardPlayer>, String> {
    let row = sqlx::query(
        r#"
        SELECT
            pss.player_id,
            pss.player_name,
            pss.team_id,
            COALESCE(t.name, '未知') as team_name,
            pss.position,
            COALESCE(p.age, 0) as age,
            COALESCE(pss.dominance_score, 0.0) as dominance_score,
            pss.avg_impact,
            pss.avg_performance,
            pss.best_performance,
            pss.consistency_score,
            pss.champion_bonus,
            COALESCE(gpp_count.real_games_played, pss.games_played) as games_played
        FROM player_season_stats pss
        LEFT JOIN teams t ON pss.team_id = t.id
        LEFT JOIN players p ON pss.player_id = p.id
        LEFT JOIN (
            SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
            FROM game_player_performances gpp
            JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
            JOIN matches m ON mg.match_id = m.id
            JOIN tournaments tr ON m.tournament_id = tr.id
            WHERE gpp.save_id = ? AND tr.season_id = ?
            GROUP BY gpp.save_id, gpp.player_id
        ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
        WHERE pss.save_id = ? AND pss.season_id = ?
          AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
        ORDER BY COALESCE(pss.dominance_score, 0.0) DESC
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

    Ok(row.map(|r| {
        let games_played = r.get::<i64, _>("games_played") as u32;
        let age = r.get::<i64, _>("age") as u8;
        let dominance_score = r.get::<f64, _>("dominance_score");
        let avg_impact = r.get::<f64, _>("avg_impact");
        let avg_performance = r.get::<f64, _>("avg_performance");
        let best_performance = r.get::<f64, _>("best_performance");
        let consistency_score = r.get::<f64, _>("consistency_score");
        let champion_bonus = r.get::<f64, _>("champion_bonus");

        let mut commentary = generate_commentary(
            avg_impact, avg_performance, best_performance, consistency_score,
            champion_bonus, games_played, age, None,
        );
        commentary.description = format!(
            "统治力评分{:.1}，巅峰发挥{:.1}，以压倒性的实力碾压对手",
            dominance_score, best_performance,
        );
        commentary.tags.insert(0, "统治力".to_string());

        SpecialAwardPlayer {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            age,
            score: dominance_score,
            games_played,
            commentary,
        }
    }))
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
            pss.avg_performance,
            pss.best_performance,
            pss.consistency_score,
            pss.champion_bonus,
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
          AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
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

    Ok(row.map(|r| {
        let avg_impact = r.get::<f64, _>("avg_impact");
        let avg_performance = r.get::<f64, _>("avg_performance");
        let best_performance = r.get::<f64, _>("best_performance");
        let consistency_score = r.get::<f64, _>("consistency_score");
        let champion_bonus = r.get::<f64, _>("champion_bonus");
        let games_played = r.get::<i64, _>("games_played") as u32;
        let age = r.get::<i64, _>("age") as u8;

        let dimensions = calculate_dimensions(
            avg_impact, avg_performance, consistency_score, games_played, champion_bonus, 0.0, true,
        );
        let mut commentary = generate_commentary(
            avg_impact, avg_performance, best_performance, consistency_score,
            champion_bonus, games_played, age, None,
        );
        commentary.description = format!(
            "年仅{}岁便展现出超越年龄的成熟与实力，未来前途无量",
            age,
        );
        commentary.tags.insert(0, "最佳新秀".to_string());

        RookiePlayer {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            age,
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            avg_impact,
            games_played,
            dimensions,
            commentary,
        }
    }))
}

fn tournament_type_weight(t_type: &str) -> f64 {
    match t_type {
        "WorldChampionship" => 1.5,
        "SuperIntercontinental" => 1.4,
        "Msi" => 1.3,
        "ClaudeIntercontinental" | "IcpIntercontinental" => 1.2,
        "MadridMasters" | "ShanghaiMasters" => 1.1,
        "SpringPlayoffs" | "SummerPlayoffs" => 1.05,
        _ => 0.9,
    }
}

#[allow(dead_code)]
fn tournament_type_display_name(t_type: &str) -> &str {
    match t_type {
        "SpringRegular" => "春季赛常规赛",
        "SpringPlayoffs" => "春季赛季后赛",
        "SummerRegular" => "夏季赛常规赛",
        "SummerPlayoffs" => "夏季赛季后赛",
        "Msi" => "MSI季中赛",
        "MadridMasters" => "马德里大师赛",
        "ClaudeIntercontinental" => "Claude洲际赛",
        "WorldChampionship" => "S世界赛",
        "ShanghaiMasters" => "上海大师赛",
        "IcpIntercontinental" => "ICP洲际对抗赛",
        "SuperIntercontinental" => "Super洲际年度邀请赛",
        _ => "未知赛事",
    }
}

/// 批量查询所有选手的赛事表现明细（1次SQL替代N次）
async fn get_all_tournament_breakdowns(
    pool: &sqlx::SqlitePool,
    save_id: &str,
    season_id: u64,
) -> Result<std::collections::HashMap<i64, (Vec<TournamentDetail>, f64, bool)>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            gpp.player_id,
            t.tournament_type,
            t.name as tournament_name,
            COUNT(DISTINCT gpp.game_id) as games_played,
            AVG(gpp.impact_score) as avg_impact,
            AVG(gpp.actual_ability) as avg_performance,
            MAX(gpp.actual_ability) as best_performance,
            SUM(CASE WHEN gpp.is_mvp = 1 THEN 1 ELSE 0 END) as mvp_count
        FROM game_player_performances gpp
        JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
        JOIN matches m ON mg.match_id = m.id AND mg.save_id = m.save_id
        JOIN tournaments t ON m.tournament_id = t.id AND m.save_id = t.save_id
        WHERE gpp.save_id = ? AND t.season_id = ?
        GROUP BY gpp.player_id, t.tournament_type, t.name
        ORDER BY gpp.player_id, t.id ASC
        "#
    )
    .bind(save_id)
    .bind(season_id as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result: std::collections::HashMap<i64, (Vec<TournamentDetail>, f64, f64)> =
        std::collections::HashMap::new();

    for row in &rows {
        let pid: i64 = row.get("player_id");
        let t_type: String = row.get("tournament_type");
        let games = row.get::<i64, _>("games_played") as u32;
        let avg_impact: f64 = row.get("avg_impact");
        let weight = tournament_type_weight(&t_type);

        let entry = result.entry(pid).or_insert_with(|| (Vec::new(), 0.0, 0.0));

        entry.0.push(TournamentDetail {
            tournament_name: row.get::<String, _>("tournament_name"),
            tournament_type: t_type.clone(),
            games_played: games,
            avg_impact,
            avg_performance: row.get("avg_performance"),
            best_performance: row.get("best_performance"),
            mvp_count: row.get::<i64, _>("mvp_count") as u32,
            weight,
        });

        let is_intl = !matches!(t_type.as_str(),
            "SpringRegular" | "SpringPlayoffs" | "SummerRegular" | "SummerPlayoffs"
        );
        if is_intl {
            entry.1 += avg_impact * games as f64;
            entry.2 += games as f64;
        }
    }

    Ok(result.into_iter().map(|(pid, (details, intl_sum, intl_games))| {
        let has_intl = intl_games > 0.0;
        let raw_score = if has_intl { intl_sum / intl_games } else { 0.0 };
        let confidence = (intl_games / 70.0).min(1.0);
        (pid, (details, raw_score * confidence, has_intl))
    }).collect())
}
