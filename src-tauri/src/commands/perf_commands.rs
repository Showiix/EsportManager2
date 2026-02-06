use crate::commands::save_commands::AppState;
use crate::services::perf_service::{PerfRecord, PerfSummary};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct PerfCommandResult<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> PerfCommandResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

/// 获取性能记录
#[tauri::command]
pub fn get_perf_records(
    state: State<'_, AppState>,
    limit: Option<usize>,
    command_filter: Option<String>,
) -> PerfCommandResult<Vec<PerfRecord>> {
    let records = state.perf.get_records(limit, command_filter.as_deref());
    PerfCommandResult::ok(records)
}

/// 获取性能统计摘要
#[tauri::command]
pub fn get_perf_summary(
    state: State<'_, AppState>,
) -> PerfCommandResult<PerfSummary> {
    PerfCommandResult::ok(state.perf.summary())
}

/// 开关性能监测
#[tauri::command]
pub fn toggle_perf_monitoring(
    state: State<'_, AppState>,
    enabled: bool,
) -> PerfCommandResult<bool> {
    state.perf.set_enabled(enabled);
    PerfCommandResult::ok(enabled)
}

/// 清空性能记录
#[tauri::command]
pub fn clear_perf_records(
    state: State<'_, AppState>,
) -> PerfCommandResult<bool> {
    state.perf.clear();
    PerfCommandResult::ok(true)
}
