use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};
use super::helpers::*;

pub struct PlayerRepository;

impl PlayerRepository {
    /// 创建选手
    pub async fn create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player: &Player,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO players (
                save_id, game_id, real_name, nationality, age, ability, potential,
                stability, tag, status, position, team_id, salary, market_value,
                contract_end_season, join_season, is_starter
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(&player.game_id)
        .bind(&player.real_name)
        .bind(&player.nationality)
        .bind(player.age as i64)
        .bind(player.ability as i64)
        .bind(player.potential as i64)
        .bind(player.stability as i64)
        .bind(format!("{:?}", player.tag))
        .bind(format!("{:?}", player.status))
        .bind(player.position.map(|p| format!("{:?}", p)))
        .bind(player.team_id.map(|id| id as i64))
        .bind(player.salary as i64)
        .bind(player.market_value as i64)
        .bind(player.contract_end_season.map(|s| s as i64))
        .bind(player.join_season as i64)
        .bind(player.is_starter)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取选手
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        player_id: u64,
    ) -> Result<Player, DatabaseError> {
        let row = sqlx::query("SELECT * FROM players WHERE id = ?")
            .bind(player_id as i64)
            .fetch_optional(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?
            .ok_or_else(|| DatabaseError::NotFound(format!("Player {} not found", player_id)))?;

        Ok(row_to_player(&row))
    }

    /// 获取队伍选手
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        team_id: u64,
    ) -> Result<Vec<Player>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM players WHERE team_id = ? AND status = 'Active' ORDER BY position, ability DESC"
        )
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player).collect())
    }

    /// 获取首发阵容
    pub async fn get_starters(
        pool: &Pool<Sqlite>,
        team_id: u64,
    ) -> Result<Vec<Player>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM players WHERE team_id = ? AND is_starter = 1 AND status = 'Active'"
        )
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player).collect())
    }

    /// 更新选手
    pub async fn update(pool: &Pool<Sqlite>, player: &Player) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE players SET
                game_id = ?, real_name = ?, nationality = ?, age = ?,
                ability = ?, potential = ?, stability = ?, tag = ?,
                status = ?, position = ?, team_id = ?, salary = ?,
                market_value = ?, contract_end_season = ?, is_starter = ?
            WHERE id = ?
            "#,
        )
        .bind(&player.game_id)
        .bind(&player.real_name)
        .bind(&player.nationality)
        .bind(player.age as i64)
        .bind(player.ability as i64)
        .bind(player.potential as i64)
        .bind(player.stability as i64)
        .bind(format!("{:?}", player.tag))
        .bind(format!("{:?}", player.status))
        .bind(player.position.map(|p| format!("{:?}", p)))
        .bind(player.team_id.map(|id| id as i64))
        .bind(player.salary as i64)
        .bind(player.market_value as i64)
        .bind(player.contract_end_season.map(|s| s as i64))
        .bind(player.is_starter)
        .bind(player.id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 转会 - 更新队伍
    pub async fn transfer(
        pool: &Pool<Sqlite>,
        player_id: u64,
        new_team_id: Option<u64>,
        salary: u64,
        contract_end: u32,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE players SET
                team_id = ?,
                salary = ?,
                contract_end_season = ?,
                is_starter = 0
            WHERE id = ?
            "#,
        )
        .bind(new_team_id.map(|id| id as i64))
        .bind(salary as i64)
        .bind(contract_end as i64)
        .bind(player_id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

impl PlayerRepository {
    /// 获取存档所有活跃选手
    pub async fn get_all_active(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Player>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM players WHERE save_id = ? AND status = 'Active' ORDER BY ability DESC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player).collect())
    }

    /// 批量更新选手能力值
    pub async fn batch_update_ability(
        pool: &Pool<Sqlite>,
        updates: &[(u64, u8)],
    ) -> Result<(), DatabaseError> {
        for (player_id, new_ability) in updates {
            sqlx::query("UPDATE players SET ability = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(*new_ability as i64)
                .bind(*player_id as i64)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }
        Ok(())
    }

    /// 批量更新选手年龄和稳定性
    pub async fn batch_update_age(
        pool: &Pool<Sqlite>,
        updates: &[(u64, u8, u8)],
    ) -> Result<(), DatabaseError> {
        for (player_id, new_age, new_stability) in updates {
            sqlx::query(
                "UPDATE players SET age = ?, stability = ?, updated_at = datetime('now') WHERE id = ?"
            )
            .bind(*new_age as i64)
            .bind(*new_stability as i64)
            .bind(*player_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }
        Ok(())
    }

    /// 批量设置选手退役
    pub async fn batch_retire(
        pool: &Pool<Sqlite>,
        player_ids: &[u64],
        retire_season: u32,
    ) -> Result<(), DatabaseError> {
        for player_id in player_ids {
            sqlx::query(
                r#"
                UPDATE players SET
                    status = 'Retired',
                    retire_season = ?,
                    team_id = NULL,
                    is_starter = 0,
                    updated_at = datetime('now')
                WHERE id = ?
                "#
            )
            .bind(retire_season as i64)
            .bind(*player_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }
        Ok(())
    }

    /// 批量更新合同
    pub async fn batch_update_contracts(
        pool: &Pool<Sqlite>,
        updates: &[(u64, bool, Option<u32>, Option<u64>)],
        current_season: u32,
    ) -> Result<(), DatabaseError> {
        for (player_id, renewed, contract_years, salary) in updates {
            if *renewed {
                if let (Some(years), Some(sal)) = (contract_years, salary) {
                    sqlx::query(
                        r#"
                        UPDATE players SET
                            contract_end_season = ?,
                            salary = ?,
                            updated_at = datetime('now')
                        WHERE id = ?
                        "#
                    )
                    .bind((current_season + years) as i64)
                    .bind(*sal as i64)
                    .bind(*player_id as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Query(e.to_string()))?;
                }
            } else {
                // 未续约，成为自由球员
                sqlx::query(
                    r#"
                    UPDATE players SET
                        team_id = NULL,
                        is_starter = 0,
                        contract_end_season = NULL,
                        updated_at = datetime('now')
                    WHERE id = ?
                    "#
                )
                .bind(*player_id as i64)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// 更新选手身价
    pub async fn update_market_value(
        pool: &Pool<Sqlite>,
        player_id: u64,
        market_value: u64,
    ) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE players SET market_value = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(market_value as i64)
            .bind(player_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

pub struct PlayerStatsRepository;

impl PlayerStatsRepository {
    /// 获取或创建选手赛季统计
    pub async fn get_or_create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        player_name: &str,
        season_id: i64,
        team_id: Option<i64>,
        region_id: Option<&str>,
        position: &str,
    ) -> Result<PlayerSeasonStatistics, DatabaseError> {
        // 尝试获取现有记录
        let existing = sqlx::query(
            r#"
            SELECT * FROM player_season_stats
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(save_id)
        .bind(player_id)
        .bind(season_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        if let Some(row) = existing {
            return Ok(row_to_player_stats(&row));
        }

        // 创建新记录
        let result = sqlx::query(
            r#"
            INSERT INTO player_season_stats
            (save_id, player_id, player_name, season_id, team_id, region_id, position,
             matches_played, games_played, total_impact, avg_impact, avg_performance,
             best_performance, worst_performance, consistency_score,
             international_titles, regional_titles, champion_bonus, yearly_top_score, dominance_score)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, 0.0, 0.0, 0.0, 0.0, 100.0, 100.0, 0, 0, 0.0, 0.0, 0.0)
            "#
        )
        .bind(save_id)
        .bind(player_id)
        .bind(player_name)
        .bind(season_id)
        .bind(team_id)
        .bind(region_id)
        .bind(position)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(PlayerSeasonStatistics {
            id: Some(result.last_insert_rowid()),
            save_id: save_id.to_string(),
            player_id,
            player_name: player_name.to_string(),
            season_id,
            team_id,
            region_id: region_id.map(|s| s.to_string()),
            position: position.to_string(),
            matches_played: 0,
            games_played: 0,
            total_impact: 0.0,
            avg_impact: 0.0,
            avg_performance: 0.0,
            best_performance: 0.0,
            worst_performance: 100.0,
            consistency_score: 100.0,
            international_titles: 0,
            regional_titles: 0,
            champion_bonus: 0.0,
            yearly_top_score: 0.0,
            dominance_score: 0.0,
        })
    }

    /// 更新选手赛季统计
    pub async fn update(
        pool: &Pool<Sqlite>,
        stats: &PlayerSeasonStatistics,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE player_season_stats SET
                team_id = ?,
                region_id = ?,
                matches_played = ?,
                games_played = ?,
                total_impact = ?,
                avg_impact = ?,
                avg_performance = ?,
                best_performance = ?,
                worst_performance = ?,
                consistency_score = ?,
                international_titles = ?,
                regional_titles = ?,
                champion_bonus = ?,
                yearly_top_score = ?,
                dominance_score = ?,
                updated_at = datetime('now')
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(stats.team_id)
        .bind(&stats.region_id)
        .bind(stats.matches_played)
        .bind(stats.games_played)
        .bind(stats.total_impact)
        .bind(stats.avg_impact)
        .bind(stats.avg_performance)
        .bind(stats.best_performance)
        .bind(stats.worst_performance)
        .bind(stats.consistency_score)
        .bind(stats.international_titles)
        .bind(stats.regional_titles)
        .bind(stats.champion_bonus)
        .bind(stats.yearly_top_score)
        .bind(stats.dominance_score)
        .bind(&stats.save_id)
        .bind(stats.player_id)
        .bind(stats.season_id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 获取赛季排行榜（按年度Top得分排序）
    /// 注意：games_played 从 game_player_performances 表实时计算，确保数据准确
    pub async fn get_season_ranking(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        limit: i32,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        // 先获取基本排行榜数据，JOIN teams 表以获取 region_id
        let rows = sqlx::query(
            r#"
            SELECT pss.*,
                   COALESCE(gpp_count.real_games_played, 0) as real_games_played,
                   tm.region_id as team_region_id
            FROM player_season_stats pss
            LEFT JOIN teams tm ON pss.team_id = tm.id AND tm.save_id = pss.save_id
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
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(save_id)
        .bind(season_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 转换并使用真实的 games_played 值，补充缺失的 region_id
        Ok(rows.iter().map(|row| {
            let mut stats = row_to_player_stats(row);
            // 使用从 game_player_performances 计算的真实场次数
            let real_games: i64 = row.try_get("real_games_played").unwrap_or(0);
            if real_games > 0 {
                stats.games_played = real_games as i32;
            }
            // 如果 region_id 为空或无效（如 "LEAGUE"），从 teams 表补充
            let needs_region = match &stats.region_id {
                None => true,
                Some(rid) => rid.parse::<i64>().is_err(),
            };
            if needs_region {
                let team_region: i64 = row.try_get("team_region_id").unwrap_or(0);
                if team_region > 0 {
                    stats.region_id = Some(team_region.to_string());
                    log::debug!("补充 region_id: player={}, team_id={:?}, region={}", stats.player_name, stats.team_id, team_region);
                }
            }
            stats
        }).collect())
    }

    /// 获取分位置排行榜
    /// 注意：games_played 从 game_player_performances 表实时计算，确保数据准确
    pub async fn get_position_ranking(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        position: &str,
        limit: i32,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT pss.*,
                   COALESCE(gpp_count.real_games_played, 0) as real_games_played,
                   tm.region_id as team_region_id
            FROM player_season_stats pss
            LEFT JOIN teams tm ON pss.team_id = tm.id AND tm.save_id = pss.save_id
            LEFT JOIN (
                SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments t ON m.tournament_id = t.id
                WHERE gpp.save_id = ? AND t.season_id = ?
                GROUP BY gpp.save_id, gpp.player_id
            ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.position = ?
              AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
            ORDER BY pss.yearly_top_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(save_id)
        .bind(season_id)
        .bind(position)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 转换并使用真实的 games_played 值，补充缺失的 region_id
        Ok(rows.iter().map(|row| {
            let mut stats = row_to_player_stats(row);
            let real_games: i64 = row.try_get("real_games_played").unwrap_or(0);
            if real_games > 0 {
                stats.games_played = real_games as i32;
            }
            let needs_region = match &stats.region_id {
                None => true,
                Some(rid) => rid.parse::<i64>().is_err(),
            };
            if needs_region {
                let team_region: i64 = row.try_get("team_region_id").unwrap_or(0);
                if team_region > 0 {
                    stats.region_id = Some(team_region.to_string());
                }
            }
            stats
        }).collect())
    }

    /// 获取队伍所有选手统计
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        team_id: i64,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM player_season_stats
            WHERE save_id = ? AND season_id = ? AND team_id = ?
            ORDER BY yearly_top_score DESC
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(team_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_stats).collect())
    }

    /// 获取选手的赛季统计
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        season_id: Option<i64>,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        // 使用与 get_season_ranking 相同的逻辑，从 game_player_performances 计算真实场次
        let rows = if let Some(sid) = season_id {
            sqlx::query(
                r#"
                SELECT pss.*,
                       COALESCE(gpp_count.real_games_played, 0) as real_games_played
                FROM player_season_stats pss
                LEFT JOIN (
                    SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                    FROM game_player_performances gpp
                    JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                    JOIN matches m ON mg.match_id = m.id
                    JOIN tournaments t ON m.tournament_id = t.id
                    WHERE gpp.save_id = ? AND gpp.player_id = ? AND t.season_id = ?
                    GROUP BY gpp.save_id, gpp.player_id
                ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
                WHERE pss.save_id = ? AND pss.player_id = ? AND pss.season_id = ?
                "#
            )
            .bind(save_id)
            .bind(player_id)
            .bind(sid)
            .bind(save_id)
            .bind(player_id)
            .bind(sid)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT pss.*,
                       COALESCE(gpp_count.real_games_played, 0) as real_games_played
                FROM player_season_stats pss
                LEFT JOIN (
                    SELECT gpp.save_id, gpp.player_id, t.season_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                    FROM game_player_performances gpp
                    JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                    JOIN matches m ON mg.match_id = m.id
                    JOIN tournaments t ON m.tournament_id = t.id
                    WHERE gpp.save_id = ? AND gpp.player_id = ?
                    GROUP BY gpp.save_id, gpp.player_id, t.season_id
                ) gpp_count ON pss.save_id = gpp_count.save_id
                           AND pss.player_id = gpp_count.player_id
                           AND pss.season_id = gpp_count.season_id
                WHERE pss.save_id = ? AND pss.player_id = ?
                ORDER BY pss.season_id DESC
                "#
            )
            .bind(save_id)
            .bind(player_id)
            .bind(save_id)
            .bind(player_id)
            .fetch_all(pool)
            .await
        }
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 使用真实场次覆盖
        Ok(rows.iter().map(|row| {
            let mut stats = row_to_player_stats(row);
            let real_games: i64 = row.try_get("real_games_played").unwrap_or(0);
            if real_games > 0 {
                stats.games_played = real_games as i32;
            }
            stats
        }).collect())
    }

    /// 获取赛季所有选手统计（用于批量重新计算）
    pub async fn get_all_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM player_season_stats
            WHERE save_id = ? AND season_id = ?
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_stats).collect())
    }

    /// 清除赛季统计数据
    pub async fn clear_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "DELETE FROM player_season_stats WHERE save_id = ? AND season_id = ?"
        )
        .bind(save_id)
        .bind(season_id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}
