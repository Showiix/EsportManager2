use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 单次命令执行的性能记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfRecord {
    /// 命令名称
    pub command: String,
    /// 执行时长 (毫秒)
    pub duration_ms: f64,
    /// ISO 8601 时间戳
    pub timestamp: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 子阶段耗时（大操作用）
    pub phases: Option<Vec<PerfPhase>>,
}

/// 子阶段耗时
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfPhase {
    pub name: String,
    pub duration_ms: f64,
}

/// 性能统计摘要
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerfSummary {
    pub total_requests: usize,
    pub avg_duration_ms: f64,
    pub max_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub slow_requests: usize,
    pub error_count: usize,
    pub error_rate: f64,
}

/// 全局性能收集器，存储在 AppState 中
pub struct PerfCollector {
    pub enabled: Mutex<bool>,
    pub records: Mutex<Vec<PerfRecord>>,
    pub max_records: usize,
}

impl PerfCollector {
    pub fn new() -> Self {
        Self {
            enabled: Mutex::new(false),
            records: Mutex::new(Vec::with_capacity(2000)),
            max_records: 5000,
        }
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    pub fn set_enabled(&self, val: bool) {
        *self.enabled.lock().unwrap() = val;
    }

    pub fn record(&self, rec: PerfRecord) {
        if !self.is_enabled() {
            return;
        }
        let mut records = self.records.lock().unwrap();
        if records.len() >= self.max_records {
            let drain_count = self.max_records / 5;
            records.drain(0..drain_count);
        }
        records.push(rec);
    }

    pub fn get_records(&self, limit: Option<usize>, command_filter: Option<&str>) -> Vec<PerfRecord> {
        let records = self.records.lock().unwrap();
        let limit = limit.unwrap_or(500);
        let result: Vec<PerfRecord> = records
            .iter()
            .rev()
            .filter(|r| {
                if let Some(filter) = command_filter {
                    r.command.contains(filter)
                } else {
                    true
                }
            })
            .take(limit)
            .cloned()
            .collect();
        result
    }

    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }

    pub fn summary(&self) -> PerfSummary {
        let records = self.records.lock().unwrap();
        let total = records.len();
        if total == 0 {
            return PerfSummary::default();
        }

        let total_duration: f64 = records.iter().map(|r| r.duration_ms).sum();
        let avg = total_duration / total as f64;
        let slow_count = records.iter().filter(|r| r.duration_ms > 500.0).count();
        let error_count = records.iter().filter(|r| !r.success).count();
        let max_duration = records.iter().map(|r| r.duration_ms).fold(0.0_f64, f64::max);

        let mut durations: Vec<f64> = records.iter().map(|r| r.duration_ms).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = ((durations.len() as f64 * 0.95) as usize).min(durations.len() - 1);
        let p95 = durations[p95_index];

        PerfSummary {
            total_requests: total,
            avg_duration_ms: (avg * 100.0).round() / 100.0,
            max_duration_ms: max_duration,
            p95_duration_ms: p95,
            slow_requests: slow_count,
            error_count,
            error_rate: ((error_count as f64 / total as f64) * 10000.0).round() / 100.0,
        }
    }
}
