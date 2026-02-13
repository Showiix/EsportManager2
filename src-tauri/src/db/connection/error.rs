/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database not initialized")]
    NotInitialized,

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
