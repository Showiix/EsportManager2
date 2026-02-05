//! 错误码系统
//!
//! 提供统一的错误码定义和错误处理机制。
//!
//! 错误码格式: E-[MODULE]-[TYPE]-[SEQ]
//! - MODULE: 模块代码 (2位)
//! - TYPE: 错误类型 (1位)
//! - SEQ: 序号 (3位)

use serde::{Deserialize, Serialize};

/// 应用错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    /// 错误码 (E-TR-B-001)
    pub code: String,
    /// 用户友好消息
    pub message: String,
    /// 技术细节
    pub details: Option<String>,
    /// 解决建议
    pub suggestion: Option<String>,
    /// 帮助文档链接
    pub doc_url: Option<String>,
}

impl AppError {
    /// 创建新错误
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            suggestion: None,
            doc_url: None,
        }
    }

    /// 添加技术细节
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// 添加解决建议
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// 添加帮助文档链接
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.doc_url = Some(url.into());
        self
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

// ============================================================================
// 转会系统错误 (TR)
// ============================================================================

pub mod transfer {
    /// 预算不足
    pub const INSUFFICIENT_BUDGET: &str = "E-TR-B-001";
    /// 选手不可转会
    pub const PLAYER_NOT_AVAILABLE: &str = "E-TR-B-002";
    /// 合同冲突
    pub const CONTRACT_CONFLICT: &str = "E-TR-B-003";
    /// 薪资上限超出
    pub const SALARY_CAP_EXCEEDED: &str = "E-TR-B-004";
    /// 转会窗口已关闭
    pub const TRANSFER_WINDOW_CLOSED: &str = "E-TR-B-005";
    /// 阵容已满
    pub const TEAM_ROSTER_FULL: &str = "E-TR-B-006";
    /// 选手拒绝
    pub const PLAYER_REFUSED: &str = "E-TR-B-007";
    /// 无效报价
    pub const INVALID_OFFER: &str = "E-TR-V-001";
    /// 选手不存在
    pub const PLAYER_NOT_FOUND: &str = "E-TR-D-001";
    /// 球队不存在
    pub const TEAM_NOT_FOUND: &str = "E-TR-D-002";
}

// ============================================================================
// 比赛系统错误 (MT)
// ============================================================================

pub mod match_sys {
    /// 比赛不存在
    pub const MATCH_NOT_FOUND: &str = "E-MT-D-001";
    /// 比赛已进行
    pub const MATCH_ALREADY_PLAYED: &str = "E-MT-B-001";
    /// 队伍不完整
    pub const TEAM_INCOMPLETE: &str = "E-MT-B-002";
    /// 赛事未就绪
    pub const TOURNAMENT_NOT_READY: &str = "E-MT-B-003";
    /// 赛事不存在
    pub const TOURNAMENT_NOT_FOUND: &str = "E-MT-D-002";
    /// 无效的比赛格式
    pub const INVALID_FORMAT: &str = "E-MT-V-001";
}

// ============================================================================
// 时间系统错误 (TM)
// ============================================================================

pub mod time {
    /// 阶段未完成
    pub const PHASE_NOT_COMPLETE: &str = "E-TM-B-001";
    /// 无效阶段切换
    pub const INVALID_PHASE_TRANSITION: &str = "E-TM-B-002";
    /// 存档不存在
    pub const SAVE_NOT_FOUND: &str = "E-TM-D-001";
    /// 赛季不存在
    pub const SEASON_NOT_FOUND: &str = "E-TM-D-002";
    /// 阶段未初始化
    pub const PHASE_NOT_INITIALIZED: &str = "E-TM-B-003";
}

// ============================================================================
// 选手系统错误 (PL)
// ============================================================================

pub mod player {
    /// 选手不存在
    pub const PLAYER_NOT_FOUND: &str = "E-PL-D-001";
    /// 无效的选手状态
    pub const INVALID_STATUS: &str = "E-PL-V-001";
    /// 选手已退役
    pub const PLAYER_RETIRED: &str = "E-PL-B-001";
    /// 无效的位置
    pub const INVALID_POSITION: &str = "E-PL-V-002";
    /// 选手属性无效
    pub const INVALID_ATTRIBUTE: &str = "E-PL-V-003";
}

// ============================================================================
// 财政系统错误 (FN)
// ============================================================================

pub mod finance {
    /// 余额不足
    pub const INSUFFICIENT_BALANCE: &str = "E-FN-B-001";
    /// 无效交易类型
    pub const INVALID_TRANSACTION_TYPE: &str = "E-FN-V-001";
    /// 交易记录不存在
    pub const TRANSACTION_NOT_FOUND: &str = "E-FN-D-001";
    /// 财务危机
    pub const FINANCIAL_CRISIS: &str = "E-FN-B-002";
    /// 薪资超限
    pub const SALARY_LIMIT_EXCEEDED: &str = "E-FN-B-003";
}

// ============================================================================
// 选秀系统错误 (DR)
// ============================================================================

pub mod draft {
    /// 选秀池为空
    pub const POOL_EMPTY: &str = "E-DR-B-001";
    /// 选秀顺位无效
    pub const INVALID_PICK: &str = "E-DR-V-001";
    /// 选手已被选中
    pub const PLAYER_ALREADY_PICKED: &str = "E-DR-B-002";
    /// 选秀未开始
    pub const DRAFT_NOT_STARTED: &str = "E-DR-B-003";
    /// 选秀已结束
    pub const DRAFT_ENDED: &str = "E-DR-B-004";
}

// ============================================================================
// 荣誉系统错误 (HN)
// ============================================================================

pub mod honor {
    /// 荣誉不存在
    pub const HONOR_NOT_FOUND: &str = "E-HN-D-001";
    /// 重复荣誉
    pub const DUPLICATE_HONOR: &str = "E-HN-B-001";
    /// 无效荣誉类型
    pub const INVALID_HONOR_TYPE: &str = "E-HN-V-001";
}

// ============================================================================
// 数据库错误 (DB)
// ============================================================================

pub mod database {
    /// 连接失败
    pub const CONNECTION_FAILED: &str = "E-DB-S-001";
    /// 查询失败
    pub const QUERY_FAILED: &str = "E-DB-S-002";
    /// 数据损坏
    pub const DATA_CORRUPTION: &str = "E-DB-D-001";
    /// 迁移失败
    pub const MIGRATION_FAILED: &str = "E-DB-S-003";
    /// 事务失败
    pub const TRANSACTION_FAILED: &str = "E-DB-S-004";
    /// 约束冲突
    pub const CONSTRAINT_VIOLATION: &str = "E-DB-D-002";
}

// ============================================================================
// 系统错误 (SY)
// ============================================================================

pub mod system {
    /// 未知错误
    pub const UNKNOWN_ERROR: &str = "E-SY-S-001";
    /// 配置错误
    pub const CONFIG_ERROR: &str = "E-SY-S-002";
    /// 初始化失败
    pub const INITIALIZATION_FAILED: &str = "E-SY-S-003";
    /// 内部错误
    pub const INTERNAL_ERROR: &str = "E-SY-S-004";
    /// 无效参数
    pub const INVALID_PARAMETER: &str = "E-SY-V-001";
    /// 权限不足
    pub const PERMISSION_DENIED: &str = "E-SY-S-005";
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 根据错误码获取错误信息
pub fn get_error_info(code: &str) -> Option<(&'static str, &'static str)> {
    match code {
        // 转会系统
        transfer::INSUFFICIENT_BUDGET => Some(("预算不足", "请先出售其他选手或等待赛季奖金发放后再尝试")),
        transfer::PLAYER_NOT_AVAILABLE => Some(("选手不可转会", "选手可能在合同保护期内，或已被其他球队锁定")),
        transfer::SALARY_CAP_EXCEEDED => Some(("薪资上限超出", "请先降低其他选手薪资或出售高薪选手")),
        transfer::PLAYER_REFUSED => Some(("选手拒绝", "尝试提高薪资待遇，或选择对球队更感兴趣的选手")),
        transfer::TRANSFER_WINDOW_CLOSED => Some(("转会窗口已关闭", "请等待下一个转会窗口期")),

        // 比赛系统
        match_sys::TEAM_INCOMPLETE => Some(("队伍不完整", "请确保每支队伍都有5名首发选手")),
        match_sys::MATCH_ALREADY_PLAYED => Some(("比赛已进行", "该比赛已经完成，无法重复模拟")),

        // 数据库
        database::CONNECTION_FAILED => Some(("数据库连接失败", "请尝试重启游戏。如问题持续，请检查游戏文件完整性")),
        database::DATA_CORRUPTION => Some(("存档数据损坏", "请尝试加载备份存档，或联系技术支持")),

        // 系统
        system::UNKNOWN_ERROR => Some(("未知错误", "请查看日志文件获取更多信息，或联系技术支持")),

        _ => None,
    }
}

/// 创建带详细信息的应用错误
pub fn create_error(code: &str, message: impl Into<String>) -> AppError {
    let mut error = AppError::new(code, message);

    if let Some((_, suggestion)) = get_error_info(code) {
        error = error.with_suggestion(suggestion);
    }

    error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_creation() {
        let error = AppError::new(transfer::INSUFFICIENT_BUDGET, "预算不足")
            .with_details("当前余额: 100万, 需要: 500万")
            .with_suggestion("请先出售选手");

        assert_eq!(error.code, "E-TR-B-001");
        assert!(error.details.is_some());
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn test_get_error_info() {
        let info = get_error_info(transfer::INSUFFICIENT_BUDGET);
        assert!(info.is_some());
        assert_eq!(info.unwrap().0, "预算不足");
    }
}
