mod error;

pub use error::DatabaseError;

use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::migrations::MigrationManager;

/// 数据库连接管理器
pub struct DatabaseManager {
    /// SQLite连接池
    pool: Arc<RwLock<Option<Pool<Sqlite>>>>,
    /// 数据库文件路径
    db_path: PathBuf,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            db_path,
        }
    }

    /// 初始化数据库连接
    pub async fn init(&self) -> Result<(), DatabaseError> {
        // 确保目录存在
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::Io(e.to_string()))?;
        }

        // 构建连接字符串
        let db_url = format!("sqlite:{}?mode=rwc", self.db_path.display());

        // 创建连接池
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        // 设置 SQLite 性能优化 PRAGMA（WAL 模式 + 降低 fsync 频率）
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::Migration(format!("设置WAL模式失败: {}", e)))?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::Migration(format!("设置synchronous失败: {}", e)))?;

        // 运行迁移
        self.run_migrations(&pool).await?;

        // 存储连接池
        let mut guard = self.pool.write().await;
        *guard = Some(pool);

        Ok(())
    }

    /// 获取连接池
    pub async fn get_pool(&self) -> Result<Pool<Sqlite>, DatabaseError> {
        let guard = self.pool.read().await;
        guard.clone().ok_or(DatabaseError::NotInitialized)
    }

    /// 关闭数据库连接
    pub async fn close(&self) {
        let mut guard = self.pool.write().await;
        if let Some(pool) = guard.take() {
            pool.close().await;
        }
    }

    /// 运行数据库迁移
    async fn run_migrations(&self, pool: &Pool<Sqlite>) -> Result<(), DatabaseError> {
        // 检查数据库是否为空（全新数据库）
        let table_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        let is_fresh = table_count.0 == 0;

        if is_fresh {
            let baseline_sql = include_str!("../../migrations/000_baseline.sql");
            
            for statement in baseline_sql.split(';') {
                let trimmed = statement.trim();
                if trimmed.is_empty() || trimmed.lines().all(|l| l.trim().starts_with("--") || l.trim().is_empty()) {
                    continue;
                }
                
                sqlx::query(trimmed)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Migration(format!("Baseline migration failed at statement: {}\nError: {}", 
                        &trimmed[..trimmed.len().min(100)], e)))?;
            }
            
            MigrationManager::mark_baseline_applied(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e))?;
        } else {
            // 旧数据库：确保 baseline 被标记为已应用
            MigrationManager::ensure_baseline_marked(pool)
                .await
                .map_err(|e| DatabaseError::Migration(e))?;
        }

        // 运行后续增量迁移（001_*.sql, 002_*.sql, ...）
        MigrationManager::run_pending_migrations(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e))?;

        Ok(())
    }

}
