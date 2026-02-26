use sqlx::{Pool, Sqlite};

/// 数据库迁移管理
pub struct MigrationManager;

impl MigrationManager {
    /// 创建迁移记录表
    async fn ensure_migrations_table(pool: &Pool<Sqlite>) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 标记 baseline 已应用
    pub async fn mark_baseline_applied(pool: &Pool<Sqlite>) -> Result<(), String> {
        Self::ensure_migrations_table(pool).await?;
        
        sqlx::query("INSERT OR IGNORE INTO _migrations (name) VALUES (?)")
            .bind("000_baseline")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 确保旧数据库的 baseline 被标记为已应用
    pub async fn ensure_baseline_marked(pool: &Pool<Sqlite>) -> Result<(), String> {
        Self::ensure_migrations_table(pool).await?;
        
        let applied: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM _migrations WHERE name = ?"
        )
        .bind("000_baseline")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if applied.is_none() {
            sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                .bind("000_baseline")
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 检查并运行需要的迁移
    pub async fn run_pending_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
        Self::ensure_migrations_table(pool).await?;

        let migrations: Vec<(&str, &str)> = vec![
            // 未来的增量迁移在这里注册
            // ("001_feature_name", include_str!("../../migrations/001_feature_name.sql")),
        ];

        for (name, sql) in migrations {
            let applied: Option<(i64,)> = sqlx::query_as(
                "SELECT id FROM _migrations WHERE name = ?"
            )
            .bind(name)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

            if applied.is_none() {
                // 执行迁移 SQL（逐条执行，跳过空语句）
                for statement in sql.split(';') {
                    let trimmed = statement.trim();
                    if !trimmed.is_empty() {
                        let result = sqlx::query(trimmed)
                            .execute(pool)
                            .await;

                        // 处理错误：忽略 "duplicate column name" 错误（列已存在）
                        if let Err(e) = result {
                            let err_msg = e.to_string();
                            if err_msg.contains("duplicate column name") {
                                // 列已存在，跳过这条语句
                                continue;
                            } else {
                                // 其他错误，中断迁移
                                return Err(format!("Migration {} failed: {}", name, err_msg));
                            }
                        }
                    }
                }

                // 记录迁移已应用
                sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                    .bind(name)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    /// 获取当前迁移版本
    pub async fn get_current_version(pool: &Pool<Sqlite>) -> Result<i32, String> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM _migrations"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count.0 as i32)
    }
}
