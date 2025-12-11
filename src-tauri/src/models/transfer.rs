use serde::{Deserialize, Serialize};

/// 转会类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferType {
    /// 自由球员签约
    FreeAgent,
    /// 主动求购
    Purchase,
    /// 被动出售
    Sale,
    /// 退役
    Retirement,
    /// 租借
    Loan,
    /// 选秀
    Draft,
    /// 合同到期
    ContractExpire,
}

impl TransferType {
    pub fn name(&self) -> &'static str {
        match self {
            TransferType::FreeAgent => "自由签约",
            TransferType::Purchase => "转会引进",
            TransferType::Sale => "转会出售",
            TransferType::Retirement => "退役",
            TransferType::Loan => "租借",
            TransferType::Draft => "选秀加入",
            TransferType::ContractExpire => "合同到期",
        }
    }
}

/// 转会记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub player_id: u64,
    /// 原球队 (NULL=自由球员/新秀)
    pub from_team_id: Option<u64>,
    /// 新球队 (NULL=退役/解约)
    pub to_team_id: Option<u64>,
    pub transfer_type: TransferType,
    /// 转会费 (万元)
    pub transfer_fee: u64,
    /// 新薪资 (万元/年)
    pub new_salary: Option<u64>,
    /// 合同年限
    pub contract_years: Option<u32>,
    pub description: Option<String>,
}

/// 转会市场挂牌状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ListingStatus {
    Active,
    Sold,
    Withdrawn,
}

/// 转会市场挂牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferListing {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub player_id: u64,
    pub team_id: u64,
    /// 挂牌类型
    pub listing_type: ListingType,
    /// 要价 (万元)
    pub asking_price: u64,
    /// 最低接受价
    pub min_price: Option<u64>,
    pub status: ListingStatus,
}

/// 挂牌类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ListingType {
    ForSale,
    LoanAvailable,
}

/// 自由球员状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FreeAgentStatus {
    Available,
    Signed,
    Retired,
}

/// 自由球员原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FreeAgentReason {
    ContractExpire,
    Released,
    RetiredTeam,
}

/// 自由球员
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeAgent {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub player_id: u64,
    /// 期望年薪 (万元)
    pub salary_demand: u64,
    pub reason: FreeAgentReason,
    pub status: FreeAgentStatus,
}

/// 球队转会需求评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTransferNeeds {
    pub team_id: u64,
    /// 空缺位置
    pub vacant_positions: Vec<String>,
    /// 短板位置 (能力值最低的位置)
    pub weak_positions: Vec<(String, u8)>,
    /// 平均年龄
    pub avg_age: f64,
    /// 是否需要年轻化
    pub needs_youth: bool,
    /// 是否需要老将
    pub needs_veteran: bool,
    /// 预算
    pub budget: i64,
    /// 野心程度 (基于上赛季排名)
    pub ambition_level: AmbitionLevel,
}

/// 野心程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmbitionLevel {
    /// 争冠
    Championship,
    /// 争夺季后赛
    Playoff,
    /// 重建
    Rebuild,
}
