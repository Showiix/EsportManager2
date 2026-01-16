use crate::db::{DatabaseManager, SaveRepository};
use crate::models::Save;
use crate::services::InitService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// 应用状态
pub struct AppState {
    pub db: Arc<RwLock<Option<DatabaseManager>>>,
    pub current_save_id: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            current_save_id: Arc::new(RwLock::new(None)),
        }
    }
}

/// 响应类型
#[derive(Debug, Serialize)]
pub struct CommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// 存档列表项
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveListItem {
    pub id: String,
    pub name: String,
    pub current_season: u32,
    pub current_phase: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 初始化数据库
#[tauri::command]
pub async fn init_database(
    state: State<'_, AppState>,
    db_path: String,
) -> Result<CommandResult<()>, String> {
    let db = DatabaseManager::new(std::path::PathBuf::from(db_path));

    if let Err(e) = db.init().await {
        return Ok(CommandResult::err(format!("Failed to init database: {}", e)));
    }

    let mut guard = state.db.write().await;
    *guard = Some(db);

    Ok(CommandResult::ok(()))
}

/// 创建新存档
#[tauri::command]
pub async fn create_save(
    state: State<'_, AppState>,
    name: String,
) -> Result<CommandResult<SaveListItem>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    // 检查是否已存在同名存档
    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM saves WHERE name = ?")
        .bind(&name)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(CommandResult::err(format!("存档名称「{}」已存在，请使用其他名称", name)));
    }

    let save = Save::new(name.clone());

    if let Err(e) = SaveRepository::create(&pool, &save).await {
        return Ok(CommandResult::err(format!("Failed to create save: {}", e)));
    }

    // 初始化游戏数据 (队伍、选手等)
    if let Err(e) = InitService::initialize_game_data(&pool, &save.id, save.current_season).await {
        // 如果初始化失败，删除已创建的存档
        let _ = SaveRepository::delete(&pool, &save.id).await;
        return Ok(CommandResult::err(format!("Failed to initialize game data: {}", e)));
    }

    // 设置当前存档
    let mut current = state.current_save_id.write().await;
    *current = Some(save.id.clone());

    Ok(CommandResult::ok(SaveListItem {
        id: save.id,
        name: save.name,
        current_season: save.current_season,
        current_phase: format!("{:?}", save.current_phase),
        created_at: save.created_at.to_rfc3339(),
        updated_at: save.updated_at.to_rfc3339(),
    }))
}

/// 获取所有存档
#[tauri::command]
pub async fn get_saves(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<SaveListItem>>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let saves = match SaveRepository::get_all(&pool).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get saves: {}", e))),
    };

    let items: Vec<SaveListItem> = saves
        .into_iter()
        .map(|s| SaveListItem {
            id: s.id,
            name: s.name,
            current_season: s.current_season,
            current_phase: format!("{:?}", s.current_phase),
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(CommandResult::ok(items))
}

/// 加载存档
#[tauri::command]
pub async fn load_save(
    state: State<'_, AppState>,
    save_id: String,
) -> Result<CommandResult<SaveListItem>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let save = match SaveRepository::get_by_id(&pool, &save_id).await {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Failed to load save: {}", e))),
    };

    // 设置当前存档
    let mut current = state.current_save_id.write().await;
    *current = Some(save.id.clone());

    Ok(CommandResult::ok(SaveListItem {
        id: save.id,
        name: save.name,
        current_season: save.current_season,
        current_phase: format!("{:?}", save.current_phase),
        created_at: save.created_at.to_rfc3339(),
        updated_at: save.updated_at.to_rfc3339(),
    }))
}

/// 删除存档
#[tauri::command]
pub async fn delete_save(
    state: State<'_, AppState>,
    save_id: String,
) -> Result<CommandResult<()>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(CommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(CommandResult::err(format!("Failed to get pool: {}", e))),
    };

    if let Err(e) = SaveRepository::delete(&pool, &save_id).await {
        return Ok(CommandResult::err(format!("Failed to delete save: {}", e)));
    }

    // 如果删除的是当前存档，清除
    let mut current = state.current_save_id.write().await;
    if current.as_ref() == Some(&save_id) {
        *current = None;
    }

    Ok(CommandResult::ok(()))
}

/// 获取当前存档ID
#[tauri::command]
pub async fn get_current_save_id(
    state: State<'_, AppState>,
) -> Result<CommandResult<Option<String>>, String> {
    let current = state.current_save_id.read().await;
    Ok(CommandResult::ok(current.clone()))
}

/// 删除数据库文件（开发调试用）
#[tauri::command]
pub async fn delete_database(
    state: State<'_, AppState>,
    db_path: String,
) -> Result<CommandResult<()>, String> {
    // 先关闭数据库连接
    {
        let mut guard = state.db.write().await;
        if let Some(db) = guard.take() {
            db.close().await;
        }
    }

    // 清除当前存档ID
    {
        let mut current = state.current_save_id.write().await;
        *current = None;
    }

    // 删除数据库文件
    let path = std::path::PathBuf::from(&db_path);
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            return Ok(CommandResult::err(format!("Failed to delete database file: {}", e)));
        }
    }

    // 同时删除 SQLite 的 WAL 和 SHM 文件（如果存在）
    let wal_path = format!("{}-wal", db_path);
    let shm_path = format!("{}-shm", db_path);
    let _ = std::fs::remove_file(&wal_path);
    let _ = std::fs::remove_file(&shm_path);

    Ok(CommandResult::ok(()))
}
