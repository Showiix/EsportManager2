use sqlx::{Pool, Sqlite};

use super::error::DatabaseError;
use super::DatabaseManager;

impl DatabaseManager {
    /// 修补旧版表缺失的列，确保 SCHEMA_SQL 索引创建不会失败
    pub(super) async fn patch_legacy_tables(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查 transfer_events 表是否存在
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='transfer_events'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        if !tables.is_empty() {
            let cols: Vec<(String,)> = sqlx::query_as(
                "SELECT name FROM pragma_table_info('transfer_events')"
            )
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            let col_names: Vec<&str> = cols.iter().map(|c| c.0.as_str()).collect();

            if !col_names.contains(&"save_id") {
                sqlx::query("ALTER TABLE transfer_events ADD COLUMN save_id TEXT NOT NULL DEFAULT ''")
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;

                // 从关联的 transfer_windows 表回填 save_id
                sqlx::query(r#"
                    UPDATE transfer_events SET save_id = (
                        SELECT tw.save_id FROM transfer_windows tw WHERE tw.id = transfer_events.window_id
                    ) WHERE save_id = ''
                "#)
                .execute(pool)
                .await
                .ok();
            }

            if !col_names.contains(&"season_id") {
                sqlx::query("ALTER TABLE transfer_events ADD COLUMN season_id INTEGER NOT NULL DEFAULT 0")
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;

                // 从关联的 transfer_windows 表回填 season_id
                sqlx::query(r#"
                    UPDATE transfer_events SET season_id = (
                        SELECT tw.season_id FROM transfer_windows tw WHERE tw.id = transfer_events.window_id
                    ) WHERE season_id = 0
                "#)
                .execute(pool)
                .await
                .ok();
            }
        }

        Ok(())
    }

    /// 修复 Super 赛事已完成比赛的胜败者未填入下一轮
    pub(super) async fn repair_super_winner_routing(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 找到所有 Super 赛事（InProgress 状态，有 FIGHTER_GROUP 比赛的）
        let super_tournaments: Vec<(i64,)> = sqlx::query_as(
            "SELECT DISTINCT tournament_id FROM matches WHERE stage LIKE 'FIGHTER_GROUP%'"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        for (tournament_id,) in &super_tournaments {
            // === 1. Fighter 组胜者 → CHALLENGER_PROMOTION home ===
            // 计算 Fighter A 组积分榜
            let fighter_a_winner = self.get_fighter_group_winner(pool, *tournament_id, "FIGHTER_GROUP_A").await?;
            let fighter_b_winner = self.get_fighter_group_winner(pool, *tournament_id, "FIGHTER_GROUP_B").await?;

            if let Some(winner_a) = fighter_a_winner {
                // 填入 CHALLENGER_PROMOTION match 1 的 home
                sqlx::query(
                    "UPDATE matches SET home_team_id = ? WHERE tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = 1 AND (home_team_id IS NULL OR home_team_id = 0)"
                )
                .bind(winner_a)
                .bind(tournament_id)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
            }

            if let Some(winner_b) = fighter_b_winner {
                // 填入 CHALLENGER_PROMOTION match 2 的 home
                sqlx::query(
                    "UPDATE matches SET home_team_id = ? WHERE tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = 2 AND (home_team_id IS NULL OR home_team_id = 0)"
                )
                .bind(winner_b)
                .bind(tournament_id)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
            }

            // === 2. 定位赛胜者 → PREP_WINNERS, 败者 → CHALLENGER_PROMOTION away ===
            let positioning_matches: Vec<(i64, Option<i64>, i64, i64, i64)> = sqlx::query_as(
                r#"
                SELECT m.id, m.winner_id, m.home_team_id, m.away_team_id, m.match_order
                FROM matches m
                WHERE m.tournament_id = ? AND m.stage = 'CHALLENGER_POSITIONING' AND UPPER(m.status) = 'COMPLETED' AND m.winner_id IS NOT NULL
                ORDER BY m.match_order
                "#
            )
            .bind(tournament_id)
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            for (_, winner_id, home_id, away_id, match_order) in &positioning_matches {
                if let Some(winner) = winner_id {
                    let loser = if *winner == *home_id { *away_id } else { *home_id };
                    let is_home = *match_order == 1;

                    // 胜者 → PREP_WINNERS
                    let slot = if is_home { "home_team_id" } else { "away_team_id" };
                    sqlx::query(&format!(
                        "UPDATE matches SET {} = ? WHERE tournament_id = ? AND stage = 'PREP_WINNERS' AND match_order = 1 AND ({} IS NULL OR {} = 0)",
                        slot, slot, slot
                    ))
                    .bind(winner)
                    .bind(tournament_id)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;

                    // 败者 → CHALLENGER_PROMOTION away
                    sqlx::query(
                        "UPDATE matches SET away_team_id = ? WHERE tournament_id = ? AND stage = 'CHALLENGER_PROMOTION' AND match_order = ? AND (away_team_id IS NULL OR away_team_id = 0)"
                    )
                    .bind(loser)
                    .bind(tournament_id)
                    .bind(match_order)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;
                }
            }

            // === 3. 晋级赛胜者 → PREP_LOSERS ===
            let promotion_matches: Vec<(i64, Option<i64>, i64)> = sqlx::query_as(
                r#"
                SELECT m.id, m.winner_id, m.match_order
                FROM matches m
                WHERE m.tournament_id = ? AND m.stage = 'CHALLENGER_PROMOTION' AND UPPER(m.status) = 'COMPLETED' AND m.winner_id IS NOT NULL
                ORDER BY m.match_order
                "#
            )
            .bind(tournament_id)
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

            for (_, winner_id, match_order) in &promotion_matches {
                if let Some(winner) = winner_id {
                    let is_home = *match_order == 1;
                    let slot = if is_home { "home_team_id" } else { "away_team_id" };
                    sqlx::query(&format!(
                        "UPDATE matches SET {} = ? WHERE tournament_id = ? AND stage = 'PREP_LOSERS' AND match_order = 1 AND ({} IS NULL OR {} = 0)",
                        slot, slot, slot
                    ))
                    .bind(winner)
                    .bind(tournament_id)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// 获取 Fighter 组第一名
    async fn get_fighter_group_winner(&self, pool: &Pool<Sqlite>, tournament_id: i64, group_stage: &str) -> Result<Option<i64>, DatabaseError> {
        // 统计每队胜场数，取胜场最多的
        let rows: Vec<(i64, i64)> = sqlx::query_as(
            r#"
            SELECT team_id, COUNT(*) as wins FROM (
                SELECT winner_id as team_id FROM matches
                WHERE tournament_id = ? AND stage = ? AND UPPER(status) = 'COMPLETED' AND winner_id IS NOT NULL
            ) GROUP BY team_id ORDER BY wins DESC LIMIT 1
            "#
        )
        .bind(tournament_id)
        .bind(group_stage)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        Ok(rows.first().map(|(id, _)| *id))
    }

}
