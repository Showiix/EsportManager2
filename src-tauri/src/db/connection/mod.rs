mod legacy_patches;
mod incremental_migrations;
mod error;
mod schema;

pub use error::DatabaseError;
pub(crate) use schema::SCHEMA_SQL;

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
        // 首先运行 SQL 文件迁移系统（处理表结构变更）
        MigrationManager::run_pending_migrations(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e))?;

        // 修补旧表缺失列（必须在 SCHEMA_SQL 之前，否则索引创建会失败）
        self.patch_legacy_tables(pool).await?;

        // 创建基础表结构
        sqlx::query(SCHEMA_SQL)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        // 运行增量迁移
        self.run_incremental_migrations(pool).await?;

        Ok(())
    }

}
