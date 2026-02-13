use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};
use super::helpers::*;

pub struct LLMTaskLogRepository;

impl LLMTaskLogRepository {
    /// 插入或更新任务
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        task: &LLMTaskLog,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO llm_task_log
            (save_id, season_id, task_type, entity_id, entity_type, status, attempt_count,
             max_attempts, error_msg, last_error_at, created_at, updated_at, completed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(save_id, season_id, task_type, entity_id)
            DO UPDATE SET
                status = excluded.status,
                attempt_count = excluded.attempt_count,
                error_msg = excluded.error_msg,
                last_error_at = excluded.last_error_at,
                updated_at = excluded.updated_at,
                completed_at = excluded.completed_at
            RETURNING id
            "#,
        )
        .bind(&task.save_id)
        .bind(task.season_id as i64)
        .bind(task.task_type.as_str())
        .bind(task.entity_id as i64)
        .bind(&task.entity_type)
        .bind(task.status.as_str())
        .bind(task.attempt_count as i64)
        .bind(task.max_attempts as i64)
        .bind(&task.error_msg)
        .bind(&task.last_error_at)
        .bind(&task.created_at)
        .bind(&task.updated_at)
        .bind(&task.completed_at)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.get::<i64, _>(0) as u64)
    }

    /// 查询失败的任务
    pub async fn get_failed_tasks(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        task_type: Option<TaskType>,
    ) -> Result<Vec<LLMTaskLog>, DatabaseError> {
        let mut query = "SELECT * FROM llm_task_log WHERE save_id = ? AND season_id = ? AND status = 'failed'".to_string();

        if task_type.is_some() {
            query.push_str(" AND task_type = ?");
        }

        let mut sql = sqlx::query(&query)
            .bind(save_id)
            .bind(season_id as i64);

        if let Some(tt) = task_type {
            sql = sql.bind(tt.as_str());
        }

        let rows = sql
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_llm_task_log).collect())
    }

    /// 查询成功的任务 ID
    pub async fn get_success_ids(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        task_type: TaskType,
    ) -> Result<Vec<u64>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT entity_id FROM llm_task_log WHERE save_id = ? AND season_id = ? AND task_type = ? AND status = 'success'"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(task_type.as_str())
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get::<i64, _>(0) as u64).collect())
    }

    /// 获取任务统计
    pub async fn get_task_stats(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        task_type: TaskType,
    ) -> Result<TaskStats, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END) as running,
                SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed
            FROM llm_task_log
            WHERE save_id = ? AND season_id = ? AND task_type = ?
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(task_type.as_str())
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        if let Some(row) = row {
            Ok(TaskStats {
                total: row.get::<i64, _>(0) as u32,
                pending: row.get::<i64, _>(1) as u32,
                running: row.get::<i64, _>(2) as u32,
                success: row.get::<i64, _>(3) as u32,
                failed: row.get::<i64, _>(4) as u32,
            })
        } else {
            Ok(TaskStats::new())
        }
    }

    /// 清理旧任务（可选，防止表过大）
    pub async fn clean_old_tasks(
        pool: &Pool<Sqlite>,
        days_to_keep: i64,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            "DELETE FROM llm_task_log WHERE datetime(created_at) < datetime('now', ?)"
        )
        .bind(format!("-{} days", days_to_keep))
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
