use crate::db::DatabaseError;
use crate::models::tournament_result::PlayerTournamentStats;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

pub struct PlayerTournamentStatsRepository;

impl PlayerTournamentStatsRepository {
    /// 创建或更新选手赛事统计
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        stats: &PlayerTournamentStats,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO player_tournament_stats (
                save_id, season_id, tournament_id, tournament_type, player_id, player_name,
                team_id, team_name, position, games_played, games_won, total_impact,
                avg_impact, max_impact, avg_performance, best_performance, game_mvp_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(save_id, tournament_id, player_id) DO UPDATE SET
                games_played = excluded.games_played,
                games_won = excluded.games_won,
                total_impact = excluded.total_impact,
                avg_impact = excluded.avg_impact,
                max_impact = excluded.max_impact,
                avg_performance = excluded.avg_performance,
                best_performance = excluded.best_performance,
                game_mvp_count = excluded.game_mvp_count,
                updated_at = datetime('now')
            "#,
        )
        .bind(&stats.save_id)
        .bind(stats.season_id as i64)
        .bind(stats.tournament_id as i64)
        .bind(&stats.tournament_type)
        .bind(stats.player_id as i64)
        .bind(&stats.player_name)
        .bind(stats.team_id as i64)
        .bind(&stats.team_name)
        .bind(&stats.position)
        .bind(stats.games_played as i64)
        .bind(stats.games_won as i64)
        .bind(stats.total_impact)
        .bind(stats.avg_impact)
        .bind(stats.max_impact)
        .bind(stats.avg_performance)
        .bind(stats.best_performance)
        .bind(stats.game_mvp_count as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 批量创建/更新
    pub async fn upsert_batch(
        pool: &Pool<Sqlite>,
        stats_list: &[PlayerTournamentStats],
    ) -> Result<(), DatabaseError> {
        for stats in stats_list {
            Self::upsert(pool, stats).await?;
        }
        Ok(())
    }

    /// 获取选手在赛事中的统计
    pub async fn get_by_player_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        player_id: u64,
    ) -> Result<Option<PlayerTournamentStats>, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ? AND player_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(player_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| row_to_player_tournament_stats(&r)))
    }

    /// 获取赛事的所有选手统计（用于MVP计算）
    pub async fn get_by_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ? ORDER BY avg_impact DESC"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取赛事MVP候选（按MVP得分排序）
    pub async fn get_mvp_candidates(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        limit: i32,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        // MVP 计算: MVP次数 * 10 + 平均影响力 (MVP次数优先，影响力作为次要排序依据)
        // 降低最低比赛场数要求到1场，以支持季后赛等比赛场数较少的赛事
        let rows = sqlx::query(
            r#"
            SELECT *,
                   (game_mvp_count * 10.0 + avg_impact) as mvp_score
            FROM player_tournament_stats
            WHERE save_id = ? AND tournament_id = ? AND games_played >= 1
            ORDER BY mvp_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取选手在所有赛事中的统计
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND player_id = ? ORDER BY season_id DESC, tournament_id DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取队伍在赛事中的所有选手统计
    pub async fn get_by_team_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        team_id: u64,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ? AND team_id = ? ORDER BY avg_impact DESC"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取指定队伍的赛事MVP候选（按MVP得分排序）
    /// 用于国际赛事（MSI、马德里大师赛等）从冠军队伍中选择MVP
    pub async fn get_mvp_candidates_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        team_id: u64,
        limit: i32,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        // MVP 计算: MVP次数 * 10 + 平均影响力 (MVP次数优先，影响力作为次要排序依据)
        let rows = sqlx::query(
            r#"
            SELECT *,
                   (game_mvp_count * 10.0 + avg_impact) as mvp_score
            FROM player_tournament_stats
            WHERE save_id = ? AND tournament_id = ? AND team_id = ? AND games_played >= 1
            ORDER BY mvp_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(team_id as i64)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 删除赛事的所有选手统计
    pub async fn delete_by_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "DELETE FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}
