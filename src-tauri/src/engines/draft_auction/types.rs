//! 选秀权拍卖类型定义

use crate::models::FinancialStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 简化的新秀信息（供拍卖引擎决策用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRookieInfo {
    pub ability: u8,
    pub potential: u8,
    pub position: String, // "TOP" / "JUG" / "MID" / "ADC" / "SUP"
    pub draft_rank: u32,
}

/// 球队拍卖相关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAuctionInfo {
    pub team_id: u64,
    pub team_name: String,
    pub balance: i64,
    pub financial_status: FinancialStatus,
    pub roster_count: u32,
    pub position_needs: HashMap<String, u8>, // position -> need_level (0-100)
    pub avg_ability: f64,
}

/// 拍卖引擎配置
#[derive(Debug, Clone)]
pub struct DraftAuctionConfig {
    /// 最大竞拍轮数
    pub max_rounds: u32,
    /// 联盟佣金比例
    pub commission_rate: f64,
}

impl Default for DraftAuctionConfig {
    fn default() -> Self {
        Self {
            max_rounds: 3,
            commission_rate: crate::models::COMMISSION_RATE,
        }
    }
}
