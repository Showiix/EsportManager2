use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};
use super::helpers::*;

/// 存档仓库
pub struct SaveRepository;

impl SaveRepository {
    /// 创建新存档
    pub async fn create(
        pool: &Pool<Sqlite>,
        save: &Save,
    ) -> Result<String, DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO saves (id, name, current_season, current_phase, phase_completed, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&save.id)
        .bind(&save.name)
        .bind(save.current_season as i64)
        .bind(format!("{:?}", save.current_phase))
        .bind(save.phase_completed)
        .bind(save.created_at.to_rfc3339())
        .bind(save.updated_at.to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(save.id.clone())
    }

    /// 获取存档
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Save, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM saves WHERE id = ?"
        )
        .bind(save_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .ok_or_else(|| DatabaseError::NotFound(format!("Save {} not found", save_id)))?;

        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        Ok(Save {
            id: row.get("id"),
            name: row.get("name"),
            current_season: row.get::<i64, _>("current_season") as u32,
            current_phase: parse_season_phase(row.get("current_phase")),
            phase_completed: row.get("phase_completed"),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    }

    /// 获取所有存档
    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<Save>, DatabaseError> {
        let rows = sqlx::query("SELECT * FROM saves ORDER BY updated_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get("created_at");
                let updated_at_str: String = row.get("updated_at");
                Save {
                    id: row.get("id"),
                    name: row.get("name"),
                    current_season: row.get::<i64, _>("current_season") as u32,
                    current_phase: parse_season_phase(row.get("current_phase")),
                    phase_completed: row.get("phase_completed"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                }
            })
            .collect())
    }

    /// 更新存档
    pub async fn update(
        pool: &Pool<Sqlite>,
        save: &Save,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE saves SET
                name = ?,
                current_season = ?,
                current_phase = ?,
                phase_completed = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&save.name)
        .bind(save.current_season as i64)
        .bind(format!("{:?}", save.current_phase))
        .bind(save.phase_completed)
        .bind(save.updated_at.to_rfc3339())
        .bind(&save.id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 删除存档（级联删除所有关联数据）
    pub async fn delete(pool: &Pool<Sqlite>, save_id: &str) -> Result<(), DatabaseError> {
        // 获取一个连接并在其上执行所有操作
        let mut conn = pool.acquire().await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 禁用外键约束检查（在同一个连接上）
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *conn)
            .await
            .ok();

        // 删除所有关联数据（按依赖顺序）
        let tables = [
            // 最底层的表（没有被其他表引用的）
            "game_player_performances",
            "match_games",
            "player_traits",
            "player_form_factors",
            "player_season_stats",
            "player_tournament_stats",
            "tournament_results",
            "honors",
            "annual_points_detail",
            "global_rankings",
            "league_standings",
            "team_season_finances",
            "financial_transactions",
            "draft_results",
            "draft_orders",
            "draft_players",
            "transfer_events",
            "transfer_windows",
            "transfer_records",
            "transfer_listings",
            "free_agents",
            // 中间层的表
            "matches",
            "players",
            "tournaments",
            "teams",
            "regions",
        ];

        for table in tables {
            sqlx::query(&format!("DELETE FROM {} WHERE save_id = ?", table))
                .bind(save_id)
                .execute(&mut *conn)
                .await
                .ok(); // 忽略错误（表可能不存在或已经为空）
        }

        // 删除存档本身
        sqlx::query("DELETE FROM saves WHERE id = ?")
            .bind(save_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 重新启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *conn)
            .await
            .ok();

        Ok(())
    }
}
