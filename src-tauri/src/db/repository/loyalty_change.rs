use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

pub struct LoyaltyChangeRepository;

impl LoyaltyChangeRepository {
    /// 创建忠诚度变化记录
    pub async fn create(
        pool: &Pool<Sqlite>,
        change: &LoyaltyChange,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO loyalty_changes (save_id, season_id, player_id, change_amount, reason)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&change.save_id)
        .bind(change.season_id as i64)
        .bind(change.player_id as i64)
        .bind(change.change_amount)
        .bind(&change.reason)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取选手的忠诚度变化历史
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<LoyaltyChange>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM loyalty_changes WHERE save_id = ? AND player_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_loyalty_change).collect())
    }
}
