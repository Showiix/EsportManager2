use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;
use serde_json;

pub struct PlayerStatusRepository;

impl PlayerStatusRepository {
    /// 创建或更新选手赛季状态
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        status: &PlayerSeasonStatus,
    ) -> Result<u64, DatabaseError> {
        let reasons_json = serde_json::to_string(&status.departure_reasons)
            .unwrap_or_else(|_| "[]".to_string());

        let result = sqlx::query(
            r#"
            INSERT INTO player_season_status (
                save_id, season_id, player_id, satisfaction, wants_to_leave,
                departure_reasons, games_as_starter, total_games, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(save_id, season_id, player_id) DO UPDATE SET
                satisfaction = excluded.satisfaction,
                wants_to_leave = excluded.wants_to_leave,
                departure_reasons = excluded.departure_reasons,
                games_as_starter = excluded.games_as_starter,
                total_games = excluded.total_games,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&status.save_id)
        .bind(status.season_id as i64)
        .bind(status.player_id as i64)
        .bind(status.satisfaction as i32)
        .bind(status.wants_to_leave)
        .bind(&reasons_json)
        .bind(status.games_as_starter as i32)
        .bind(status.total_games as i32)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取选手赛季状态
    pub async fn get(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        player_id: u64,
    ) -> Result<Option<PlayerSeasonStatus>, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM player_season_status WHERE save_id = ? AND season_id = ? AND player_id = ?"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(player_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| row_to_player_season_status(&r)))
    }

    /// 获取球队所有选手的赛季状态
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        team_id: u64,
    ) -> Result<Vec<PlayerSeasonStatus>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT pss.* FROM player_season_status pss
            INNER JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND p.team_id = ?
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_season_status).collect())
    }

    /// 获取所有想离队的选手
    pub async fn get_departure_candidates(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<PlayerSeasonStatus>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_season_status WHERE save_id = ? AND season_id = ? AND wants_to_leave = TRUE"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_season_status).collect())
    }

    /// 批量创建或更新选手赛季状态
    pub async fn batch_upsert(
        pool: &Pool<Sqlite>,
        statuses: &[PlayerSeasonStatus],
    ) -> Result<u32, DatabaseError> {
        let mut count = 0;
        for status in statuses {
            Self::upsert(pool, status).await?;
            count += 1;
        }
        Ok(count)
    }
}
