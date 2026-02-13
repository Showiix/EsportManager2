use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

pub struct EventRepository;

impl EventRepository {
    /// 创建事件记录
    pub async fn create(
        pool: &Pool<Sqlite>,
        event: &GameEvent,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO game_events (save_id, season_id, event_type, player_id, team_id, description, details, phase)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.save_id)
        .bind(event.season_id as i64)
        .bind(format!("{:?}", event.event_type))
        .bind(event.player_id.map(|id| id as i64))
        .bind(event.team_id.map(|id| id as i64))
        .bind(&event.description)
        .bind(&event.details)
        .bind(&event.phase)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 批量创建事件记录
    pub async fn create_batch(
        pool: &Pool<Sqlite>,
        events: &[GameEvent],
    ) -> Result<Vec<u64>, DatabaseError> {
        let mut ids = Vec::new();
        for event in events {
            let id = Self::create(pool, event).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// 获取赛季所有事件
    pub async fn get_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<GameEvent>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_events WHERE save_id = ? AND season_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_event).collect())
    }

    /// 获取选手相关事件
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<GameEvent>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_events WHERE save_id = ? AND player_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_event).collect())
    }

    /// 获取特定类型的事件
    pub async fn get_by_type(
        pool: &Pool<Sqlite>,
        save_id: &str,
        event_type: &str,
    ) -> Result<Vec<GameEvent>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_events WHERE save_id = ? AND event_type = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(event_type)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_event).collect())
    }
}
