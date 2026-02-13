use sqlx::{Pool, Row, Sqlite};

use super::GameFlowService;

/// 年度选手信息结构
#[allow(dead_code)]
pub(crate) struct AnnualPlayerInfo {
    pub(crate) player_id: u64,
    pub(crate) player_name: String,
    pub(crate) team_id: u64,
    pub(crate) team_name: String,
    pub(crate) position: String,
    pub(crate) yearly_score: f64,
    pub(crate) age: u8,
    pub(crate) consistency_score: f64,
    pub(crate) dominance_score: f64,
    pub(crate) games_played: i32,
}

impl GameFlowService {
    pub(crate) async fn recalculate_yearly_scores_with_big_stage(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<(), String> {
        let sid = season_id as i64;

        let big_stage_rows = sqlx::query(
            r#"SELECT gpp.player_id,
                      COUNT(DISTINCT gpp.game_id) as games_played,
                      AVG(gpp.impact_score) as avg_impact
               FROM game_player_performances gpp
               JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
               JOIN matches m ON mg.match_id = m.id AND mg.save_id = m.save_id
               JOIN tournaments t ON m.tournament_id = t.id AND m.save_id = t.save_id
               WHERE gpp.save_id = ? AND t.season_id = ?
               AND t.tournament_type NOT IN ('SpringRegular', 'SpringPlayoffs', 'SummerRegular', 'SummerPlayoffs')
               GROUP BY gpp.player_id"#
        )
        .bind(save_id)
        .bind(sid)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut big_stage_map: std::collections::HashMap<i64, (f64, f64)> = std::collections::HashMap::new();
        for row in &big_stage_rows {
            let pid: i64 = row.get("player_id");
            let games: i64 = row.get("games_played");
            let avg_impact: f64 = row.get("avg_impact");
            big_stage_map.insert(pid, (avg_impact, games as f64));
        }

        let all_stats = sqlx::query(
            r#"SELECT id, player_id, avg_impact, avg_performance, consistency_score,
                      games_played, champion_bonus
               FROM player_season_stats
               WHERE save_id = ? AND season_id = ?"#
        )
        .bind(save_id)
        .bind(sid)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询赛季统计失败: {}", e))?;

        for row in &all_stats {
            let stat_id: i64 = row.get("id");
            let pid: i64 = row.get("player_id");
            let avg_impact: f64 = row.get("avg_impact");
            let avg_performance: f64 = row.get("avg_performance");
            let consistency_score: f64 = row.get("consistency_score");
            let games_played: i32 = row.get("games_played");
            let champion_bonus: f64 = row.get("champion_bonus");

            let (bs_score, has_intl) = if let Some((impact, games)) = big_stage_map.get(&pid) {
                let raw = if *games > 0.0 { impact } else { &0.0 };
                let confidence = (*games / 70.0).min(1.0);
                (raw * confidence, true)
            } else {
                (0.0, false)
            };

            let new_score = crate::models::PlayerSeasonStatistics::calculate_yearly_top_score_6dim(
                avg_impact, avg_performance, consistency_score,
                games_played, champion_bonus, bs_score, has_intl,
            );

            sqlx::query("UPDATE player_season_stats SET yearly_top_score = ? WHERE id = ?")
                .bind(new_score)
                .bind(stat_id)
                .execute(pool)
                .await
                .ok();
        }

        log::debug!("重算 {} 位选手的 yearly_top_score（含大赛表现）", all_stats.len());
        Ok(())
    }

    /// 获取年度Top20选手
    pub(crate) async fn get_annual_top20(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<AnnualPlayerInfo>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                COALESCE(p.age, 0) as age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.games_played >= 10
            ORDER BY pss.yearly_top_score DESC
            LIMIT 20
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get top20: {}", e))?;

        Ok(rows.iter().map(|row| AnnualPlayerInfo {
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get::<String, _>("player_name"),
            team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: row.get::<String, _>("team_name"),
            position: row.get::<String, _>("position"),
            yearly_score: row.get::<f64, _>("yearly_top_score"),
            age: row.get::<i64, _>("age") as u8,
            consistency_score: row.get::<f64, _>("consistency_score"),
            dominance_score: row.get::<f64, _>("dominance_score"),
            games_played: row.get::<i32, _>("games_played"),
        }).collect())
    }

    /// 获取年度最佳阵容三阵（每位置Top3）
    pub(crate) async fn get_annual_all_pro_3teams(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<(u8, AnnualPlayerInfo)>, String> {
        let positions = vec!["TOP", "JUG", "MID", "ADC", "SUP"];
        let mut results = Vec::new();

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
                    COALESCE(p.age, 0) as age,
                    pss.consistency_score,
                    COALESCE(pss.dominance_score, 0.0) as dominance_score,
                    pss.games_played
                FROM player_season_stats pss
                LEFT JOIN teams t ON pss.team_id = t.id
                LEFT JOIN players p ON pss.player_id = p.id
                WHERE pss.save_id = ? AND pss.season_id = ? AND UPPER(pss.position) = UPPER(?) AND pss.games_played >= 10
                ORDER BY pss.yearly_top_score DESC
                LIMIT 3
                "#
            )
            .bind(save_id)
            .bind(season_id as i64)
            .bind(position)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get best {}: {}", position, e))?;

            for (tier_idx, row) in rows.iter().enumerate() {
                let tier = (tier_idx + 1) as u8;
                results.push((tier, AnnualPlayerInfo {
                    player_id: row.get::<i64, _>("player_id") as u64,
                    player_name: row.get::<String, _>("player_name"),
                    team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
                    team_name: row.get::<String, _>("team_name"),
                    position: row.get::<String, _>("position"),
                    yearly_score: row.get::<f64, _>("yearly_top_score"),
                    age: row.get::<i64, _>("age") as u8,
                    consistency_score: row.get::<f64, _>("consistency_score"),
                    dominance_score: row.get::<f64, _>("dominance_score"),
                    games_played: row.get::<i32, _>("games_played"),
                }));
            }
        }

        Ok(results)
    }

    /// 获取年度最稳定选手（consistency_score最高，>=30场）
    pub(crate) async fn get_annual_most_consistent(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Option<AnnualPlayerInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                COALESCE(p.age, 0) as age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.games_played >= 30
            ORDER BY pss.consistency_score DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get most consistent: {}", e))?;

        Ok(row.map(|r| AnnualPlayerInfo {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            age: r.get::<i64, _>("age") as u8,
            consistency_score: r.get::<f64, _>("consistency_score"),
            dominance_score: r.get::<f64, _>("dominance_score"),
            games_played: r.get::<i32, _>("games_played"),
        }))
    }

    /// 获取年度最具统治力选手（dominance_score最高，>=20场）
    pub(crate) async fn get_annual_most_dominant(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Option<AnnualPlayerInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                COALESCE(p.age, 0) as age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.games_played >= 20
            ORDER BY COALESCE(pss.dominance_score, 0.0) DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get most dominant: {}", e))?;

        Ok(row.map(|r| AnnualPlayerInfo {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            age: r.get::<i64, _>("age") as u8,
            consistency_score: r.get::<f64, _>("consistency_score"),
            dominance_score: r.get::<f64, _>("dominance_score"),
            games_played: r.get::<i32, _>("games_played"),
        }))
    }

    /// 获取年度最佳新秀（20岁及以下）
    pub(crate) async fn get_annual_rookie(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Option<AnnualPlayerInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                p.age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            JOIN players p ON pss.player_id = p.id
            LEFT JOIN teams t ON pss.team_id = t.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND p.age <= 20 AND pss.games_played >= 10
            ORDER BY pss.yearly_top_score DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get rookie of the year: {}", e))?;

        Ok(row.map(|r| AnnualPlayerInfo {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            age: r.get::<i64, _>("age") as u8,
            consistency_score: r.get::<f64, _>("consistency_score"),
            dominance_score: r.get::<f64, _>("dominance_score"),
            games_played: r.get::<i32, _>("games_played"),
        }))
    }

}
