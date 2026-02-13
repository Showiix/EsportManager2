use crate::engines::market_value::{MarketValueEngine, PlayerHonorRecord};
use sqlx::{Pool, Row, Sqlite};

use super::GameFlowService;

impl GameFlowService {
    /// 计算选手的荣誉系数（查询 DB + 委托 MarketValueEngine）
    pub(crate) async fn calculate_honor_factor(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
        current_season: u32,
    ) -> Result<f64, String> {
        let rows = sqlx::query(
            r#"
            SELECT honor_type, tournament_type, tournament_name, season_id
            FROM honors
            WHERE save_id = ? AND player_id = ?
            "#
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get player honors: {}", e))?;

        let honors: Vec<PlayerHonorRecord> = rows.iter().filter_map(|row| {
            let honor_type: String = row.get("honor_type");
            let tournament_type: String = row.get::<Option<String>, _>("tournament_type").unwrap_or_default();
            let tournament_name: String = row.get::<Option<String>, _>("tournament_name").unwrap_or_default();
            let season_obtained: u32 = row.get::<i64, _>("season_id") as u32;
            MarketValueEngine::parse_honor_category(&honor_type, &tournament_type, &tournament_name)
                .map(|cat| PlayerHonorRecord::new(cat, season_obtained, &tournament_name))
        }).collect();

        Ok(MarketValueEngine::calculate_honor_factor(&honors, current_season))
    }

    /// 完整重算单个选手身价（基础 × 荣誉 × 赛区）
    pub(crate) async fn recalculate_player_market_value_full(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        player_id: u64,
    ) -> Result<(), String> {
        // 获取选手信息
        let row = sqlx::query(
            r#"
            SELECT p.game_id, p.ability, p.potential, p.age, p.tag, p.position,
                   p.market_value, p.calculated_market_value,
                   r.short_name as region_code
            FROM players p
            LEFT JOIN teams t ON p.team_id = t.id
            LEFT JOIN regions r ON t.region_id = r.id
            WHERE p.id = ?
            "#
        )
        .bind(player_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get player: {}", e))?;

        let row = match row {
            Some(r) => r,
            None => return Ok(()),
        };

        let player_name: String = row.get("game_id");
        let ability: i64 = row.get("ability");
        let potential: i64 = row.get("potential");
        let age: i64 = row.get("age");
        let tag: String = row.get("tag");
        let position: String = row.get("position");
        let old_calculated: i64 = row.get("calculated_market_value");
        let region_code: String = row.get::<Option<String>, _>("region_code").unwrap_or_else(|| "LPL".to_string());

        // 计算基础身价（委托引擎）
        let base_value = MarketValueEngine::calculate_base_market_value(
            ability as u8, age as u8, potential as u8, &tag, &position
        );

        // 计算荣誉系数（委托引擎）
        let honor_factor = self.calculate_honor_factor(pool, save_id, player_id, season_id as u32).await?;

        // 查询赛季表现
        let perf_factor = {
            let stats_row = sqlx::query(
                r#"SELECT avg_impact, consistency_score, games_played
                   FROM player_season_stats
                   WHERE save_id = ? AND player_id = ? AND season_id = ?"#
            )
            .bind(save_id)
            .bind(player_id as i64)
            .bind(season_id as i64)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            if let Some(sr) = stats_row {
                let avg_impact: f64 = sr.try_get("avg_impact").unwrap_or(0.0);
                let consistency: f64 = sr.try_get("consistency_score").unwrap_or(60.0);
                let gp: i64 = sr.try_get("games_played").unwrap_or(0);
                MarketValueEngine::performance_factor(avg_impact, consistency, gp as u32)
            } else {
                1.0
            }
        };

        // 完整身价（委托引擎）× 表现系数
        let new_value = (MarketValueEngine::calculate_full_market_value(base_value, honor_factor, &region_code) as f64 * perf_factor) as i64;

        if new_value != old_calculated {
            // 同时更新两个列：market_value = 基础值，calculated_market_value = 完整值
            sqlx::query("UPDATE players SET market_value = ?, calculated_market_value = ? WHERE id = ?")
                .bind(base_value as i64)
                .bind(new_value)
                .bind(player_id as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update market value: {}", e))?;

            // 记录变化
            let change_amount = new_value - old_calculated;
            let change_percent = if old_calculated > 0 {
                ((new_value as f64 / old_calculated as f64) - 1.0) * 100.0
            } else {
                100.0
            };

            sqlx::query(
                r#"INSERT INTO market_value_changes
                   (save_id, season_id, player_id, player_name, old_value, new_value, change_amount, change_percent, reason)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#
            )
            .bind(save_id)
            .bind(season_id as i64)
            .bind(player_id as i64)
            .bind(&player_name)
            .bind(old_calculated)
            .bind(new_value)
            .bind(change_amount)
            .bind(change_percent)
            .bind("年度身价重算")
            .execute(pool)
            .await
            .ok();

            log::debug!("{} 身价重算: {} -> {} (荣誉×{:.2}, 赛区×{})",
                player_name, old_calculated / 10000, new_value / 10000, honor_factor, &region_code);
        }

        Ok(())
    }

    /// 年度结束时重算所有选手身价
    pub async fn recalculate_all_market_values(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<u32, String> {
        // 获取所有活跃选手
        let rows = sqlx::query("SELECT id FROM players WHERE save_id = ? AND status = 'Active'")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get players: {}", e))?;

        let mut count = 0u32;
        for row in rows {
            let player_id: i64 = row.get("id");
            self.recalculate_player_market_value_full(pool, save_id, season_id, player_id as u64).await?;
            count += 1;
        }

        log::debug!("完成 {} 名选手身价重算", count);
        Ok(count)
    }

    /// 更新所有队伍的品牌价值
    pub(crate) async fn update_all_brand_values(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<u32, String> {
        // 获取所有队伍
        let team_rows = sqlx::query(
            "SELECT id, brand_value FROM teams WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get teams: {}", e))?;

        let mut count = 0u32;

        for row in &team_rows {
            let team_id: i64 = row.get("id");
            let old_brand: f64 = row.get("brand_value");

            // 15%自然衰减
            let mut new_brand = old_brand * 0.85;

            // 查询本赛季该队伍的所有荣誉记录
            let honors = sqlx::query(
                r#"
                SELECT h.honor_type, t.tournament_type
                FROM honors h
                LEFT JOIN tournaments t ON h.tournament_id = t.id
                WHERE h.save_id = ? AND h.season_id = ? AND h.team_id = ?
                  AND h.honor_type IN ('TEAM_CHAMPION', 'TEAM_RUNNER_UP', 'TEAM_THIRD', 'TEAM_FOURTH')
                "#
            )
            .bind(save_id)
            .bind(season_id as i64)
            .bind(team_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            for honor in &honors {
                let honor_type: String = honor.get("honor_type");
                let tournament_type: Option<String> = honor.try_get("tournament_type").ok();
                let tt = tournament_type.as_deref().unwrap_or("");

                let bonus = match (tt, honor_type.as_str()) {
                    // 世界赛
                    ("WorldChampionship", "TEAM_CHAMPION") => 25.0,
                    ("WorldChampionship", "TEAM_RUNNER_UP") => 15.0,
                    ("WorldChampionship", "TEAM_THIRD") => 10.0,
                    ("WorldChampionship", "TEAM_FOURTH") => 5.0,
                    // Super洲际赛
                    ("SuperIntercontinental", "TEAM_CHAMPION") => 20.0,
                    ("SuperIntercontinental", "TEAM_RUNNER_UP") => 12.0,
                    ("SuperIntercontinental", "TEAM_THIRD") => 8.0,
                    ("SuperIntercontinental", "TEAM_FOURTH") => 5.0,
                    // MSI
                    ("Msi", "TEAM_CHAMPION") => 10.0,
                    ("Msi", "TEAM_RUNNER_UP") => 6.0,
                    ("Msi", "TEAM_THIRD") => 3.0,
                    // 联赛季后赛（夏季）
                    ("SummerPlayoffs", "TEAM_CHAMPION") => 8.0,
                    ("SummerPlayoffs", "TEAM_RUNNER_UP") => 4.0,
                    ("SummerPlayoffs", "TEAM_THIRD") => 3.0,
                    // 联赛季后赛（春季）
                    ("SpringPlayoffs", "TEAM_CHAMPION") => 5.0,
                    ("SpringPlayoffs", "TEAM_RUNNER_UP") => 3.0,
                    ("SpringPlayoffs", "TEAM_THIRD") => 2.0,
                    // 其他国际赛事四强
                    (_, "TEAM_CHAMPION") => 5.0,
                    (_, "TEAM_RUNNER_UP") => 3.0,
                    (_, "TEAM_THIRD") | (_, "TEAM_FOURTH") => 2.0,
                    _ => 0.0,
                };

                new_brand += bonus;
            }

            new_brand = new_brand.clamp(0.0, 100.0);

            // 更新品牌价值
            sqlx::query("UPDATE teams SET brand_value = ? WHERE id = ? AND save_id = ?")
                .bind(new_brand)
                .bind(team_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update brand_value: {}", e))?;

            if (new_brand - old_brand).abs() > 0.1 {
                log::debug!("队伍 {} 品牌价值: {:.1} -> {:.1}", team_id, old_brand, new_brand);
            }

            count += 1;
        }

        Ok(count)
    }
}
