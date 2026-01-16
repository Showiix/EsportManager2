//! LLM 任务日志模型
//!
//! 追踪所有 LLM 调用的状态，支持并发执行和失败重试

use serde::{Deserialize, Serialize};

/// LLM 任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    /// 选手意愿生成
    Intention,
    /// 战队策略生成
    Strategy,
    /// 续约评估
    Renewal,
    /// 自由市场评估
    FreeMarket,
    /// 挖角评估
    Poaching,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Intention => "intention",
            TaskType::Strategy => "strategy",
            TaskType::Renewal => "renewal",
            TaskType::FreeMarket => "free_market",
            TaskType::Poaching => "poaching",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "intention" => Some(TaskType::Intention),
            "strategy" => Some(TaskType::Strategy),
            "renewal" => Some(TaskType::Renewal),
            "free_market" => Some(TaskType::FreeMarket),
            "poaching" => Some(TaskType::Poaching),
            _ => None,
        }
    }
}

/// LLM 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 执行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Success => "success",
            TaskStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "running" => Some(TaskStatus::Running),
            "success" => Some(TaskStatus::Success),
            "failed" => Some(TaskStatus::Failed),
            _ => None,
        }
    }
}

/// LLM 任务日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMTaskLog {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub task_type: TaskType,
    pub entity_id: u64,
    pub entity_type: String,  // "player" | "team"
    pub status: TaskStatus,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub error_msg: Option<String>,
    pub last_error_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

impl LLMTaskLog {
    /// 创建新的任务日志
    pub fn new(
        save_id: String,
        season_id: u64,
        task_type: TaskType,
        entity_id: u64,
        entity_type: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            save_id,
            season_id,
            task_type,
            entity_id,
            entity_type,
            status: TaskStatus::Pending,
            attempt_count: 0,
            max_attempts: 3,
            error_msg: None,
            last_error_at: None,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
        }
    }

    /// 标记为运行中
    pub fn mark_running(&mut self) {
        self.status = TaskStatus::Running;
        self.attempt_count += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 标记为成功
    pub fn mark_success(&mut self) {
        self.status = TaskStatus::Success;
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error_msg = Some(error);
        self.last_error_at = Some(chrono::Utc::now().to_rfc3339());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 是否可以重试
    pub fn can_retry(&self) -> bool {
        self.status == TaskStatus::Failed && self.attempt_count < self.max_attempts
    }
}

/// 任务统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: u32,
    pub pending: u32,
    pub running: u32,
    pub success: u32,
    pub failed: u32,
}

impl TaskStats {
    pub fn new() -> Self {
        Self {
            total: 0,
            pending: 0,
            running: 0,
            success: 0,
            failed: 0,
        }
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }

    /// 是否有失败任务
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }

    /// 是否全部完成
    pub fn is_all_done(&self) -> bool {
        self.pending == 0 && self.running == 0
    }
}
