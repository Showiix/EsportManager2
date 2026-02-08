//! 日志命令模块
//!
//! 提供前端日志接收和处理的 Tauri 命令。

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};

use super::ApiResponse;

/// 前端日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendLogEntry {
    /// 日志级别: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"
    pub level: String,
    /// 模块名称: "TimeStore" | "MatchView" | "API" 等
    pub module: String,
    /// 日志消息
    pub message: String,
    /// 附加数据
    pub data: Option<serde_json::Value>,
    /// 时间戳 (ISO 8601 格式)
    pub timestamp: String,
    /// 用户操作描述
    #[serde(rename = "userAction")]
    pub user_action: Option<String>,
}

/// 前端错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendError {
    /// 错误码
    pub code: Option<String>,
    /// 错误消息
    pub message: String,
    /// 错误堆栈
    #[serde(rename = "stackTrace")]
    pub stack_trace: Option<String>,
    /// 发生组件
    pub component: Option<String>,
    /// 用户操作
    #[serde(rename = "userAction")]
    pub user_action: Option<String>,
}

/// 接收前端日志事件 (批量)
///
/// 前端会批量发送日志到后端进行持久化
#[tauri::command]
pub async fn log_frontend_event(entries: Vec<FrontendLogEntry>) -> ApiResponse<u32> {
    let count = entries.len() as u32;

    for entry in entries {
        let _target = format!("frontend::{}", entry.module);

        match entry.level.to_uppercase().as_str() {
            "ERROR" => {
                error!(
                    target: "frontend",
                    module = %entry.module,
                    data = ?entry.data,
                    user_action = ?entry.user_action,
                    timestamp = %entry.timestamp,
                    "[{}] {}",
                    entry.module,
                    entry.message
                );
            }
            "WARN" => {
                warn!(
                    target: "frontend",
                    module = %entry.module,
                    data = ?entry.data,
                    timestamp = %entry.timestamp,
                    "[{}] {}",
                    entry.module,
                    entry.message
                );
            }
            "INFO" => {
                info!(
                    target: "frontend",
                    module = %entry.module,
                    data = ?entry.data,
                    timestamp = %entry.timestamp,
                    "[{}] {}",
                    entry.module,
                    entry.message
                );
            }
            "DEBUG" => {
                debug!(
                    target: "frontend",
                    module = %entry.module,
                    data = ?entry.data,
                    timestamp = %entry.timestamp,
                    "[{}] {}",
                    entry.module,
                    entry.message
                );
            }
            _ => {
                trace!(
                    target: "frontend",
                    module = %entry.module,
                    data = ?entry.data,
                    timestamp = %entry.timestamp,
                    "[{}] {}",
                    entry.module,
                    entry.message
                );
            }
        }
    }

    ApiResponse::success(count)
}

/// 接收前端错误
///
/// 用于记录前端捕获的错误，包括未处理的异常
#[tauri::command]
pub async fn log_frontend_error(
    error_message: String,
    stack_trace: Option<String>,
    component: Option<String>,
    user_action: Option<String>,
    error_code: Option<String>,
) -> ApiResponse<bool> {
    error!(
        target: "frontend::error",
        code = ?error_code,
        component = ?component,
        user_action = ?user_action,
        stack = ?stack_trace,
        "前端错误: {}",
        error_message
    );

    ApiResponse::success(true)
}

/// 获取日志文件列表
#[tauri::command]
pub async fn get_log_files() -> ApiResponse<Vec<LogFileInfo>> {
    use std::fs;

    let log_dir = std::path::PathBuf::from("logs");

    if !log_dir.exists() {
        return ApiResponse::success(vec![]);
    }

    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    if let Ok(metadata) = entry.metadata() {
                        files.push(LogFileInfo {
                            name: name.to_string_lossy().to_string(),
                            size: metadata.len(),
                            modified: metadata
                                .modified()
                                .ok()
                                .and_then(|t| {
                                    t.duration_since(std::time::UNIX_EPOCH)
                                        .ok()
                                        .map(|d| d.as_secs())
                                }),
                        });
                    }
                }
            }
        }
    }

    // 按修改时间降序排序
    files.sort_by(|a, b| b.modified.cmp(&a.modified));

    ApiResponse::success(files)
}

/// 日志文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileInfo {
    /// 文件名
    pub name: String,
    /// 文件大小 (字节)
    pub size: u64,
    /// 修改时间 (Unix 时间戳)
    pub modified: Option<u64>,
}

/// 读取日志文件内容
#[tauri::command]
pub async fn read_log_file(
    file_name: String,
    offset: Option<u64>,
    limit: Option<u64>,
) -> ApiResponse<LogFileContent> {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};

    let log_dir = std::path::PathBuf::from("logs");
    let file_path = log_dir.join(&file_name);

    // 安全检查：确保文件在 logs 目录内
    if !file_path.starts_with(&log_dir) {
        return ApiResponse::error("非法的文件路径");
    }

    if !file_path.exists() {
        return ApiResponse::error("日志文件不存在");
    }

    let file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => return ApiResponse::error(&format!("无法打开文件: {}", e)),
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(&format!("无法读取文件信息: {}", e)),
    };

    let total_size = metadata.len();
    let mut reader = BufReader::new(file);

    // 跳过指定偏移量
    if let Some(off) = offset {
        if let Err(e) = reader.seek(SeekFrom::Start(off)) {
            return ApiResponse::error(&format!("无法跳转到指定位置: {}", e));
        }
    }

    // 读取指定行数
    let max_lines = limit.unwrap_or(100) as usize;
    let mut lines = Vec::with_capacity(max_lines);
    let mut bytes_read = 0u64;

    for line in reader.lines().take(max_lines) {
        match line {
            Ok(l) => {
                bytes_read += l.len() as u64 + 1; // +1 for newline
                lines.push(l);
            }
            Err(_) => break,
        }
    }

    ApiResponse::success(LogFileContent {
        file_name,
        lines,
        total_size,
        offset: offset.unwrap_or(0),
        bytes_read,
    })
}

/// 日志文件内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileContent {
    /// 文件名
    pub file_name: String,
    /// 日志行
    pub lines: Vec<String>,
    /// 文件总大小
    pub total_size: u64,
    /// 读取起始位置
    pub offset: u64,
    /// 本次读取的字节数
    pub bytes_read: u64,
}

/// 清理旧日志
#[tauri::command]
pub async fn cleanup_logs(max_age_days: Option<u64>) -> ApiResponse<u32> {
    use crate::services::logging_service::cleanup_old_logs;

    let log_dir = std::path::PathBuf::from("logs");
    let days = max_age_days.unwrap_or(7);

    match cleanup_old_logs(&log_dir, days) {
        Ok(count) => {
            info!(deleted_count = count, max_age_days = days, "清理旧日志完成");
            ApiResponse::success(count)
        }
        Err(e) => ApiResponse::error(&format!("清理日志失败: {}", e)),
    }
}
