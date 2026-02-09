use sqlx::{Pool, Sqlite};

/// 数据库迁移管理
pub struct MigrationManager;

impl MigrationManager {
    /// 检查并运行需要的迁移
    pub async fn run_pending_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
        // 创建迁移记录表
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

        // 检查并运行迁移
        let migrations = vec![
            ("001_initial", include_str!("../../migrations/001_initial.sql")),
            ("010_transfer_system", include_str!("../../migrations/010_transfer_system.sql")),
            ("011_fix_transfer_events", include_str!("../../migrations/011_fix_transfer_events.sql")),
            ("012_add_satisfaction", include_str!("../../migrations/012_add_satisfaction.sql")),
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
