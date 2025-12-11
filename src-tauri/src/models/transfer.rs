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

// ============== 转会窗口系统 ==============

/// 转会窗口状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferWindowStatus {
    /// 准备中
    Preparing,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
}

/// 转会窗口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferWindow {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub status: TransferWindowStatus,
    pub current_round: u32,
    pub total_rounds: u32,

    // 统计数据
    pub total_transfers: u32,
    pub total_fees: u64,
    pub free_agents_signed: u32,
    pub retirements: u32,
    pub contract_expires: u32,

    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl Default for TransferWindow {
    fn default() -> Self {
        Self {
            id: 0,
            save_id: String::new(),
            season_id: 0,
            status: TransferWindowStatus::Preparing,
            current_round: 0,
            total_rounds: 5,
            total_transfers: 0,
            total_fees: 0,
            free_agents_signed: 0,
            retirements: 0,
            contract_expires: 0,
            started_at: None,
            completed_at: None,
        }
    }
}

/// 转会事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferEventType {
    /// 自由球员签约
    FreeAgent,
    /// 转会购买
    Purchase,
    /// 退役
    Retirement,
    /// 合同到期
    ContractExpire,
}

impl TransferEventType {
    pub fn name(&self) -> &'static str {
        match self {
            TransferEventType::FreeAgent => "自由签约",
            TransferEventType::Purchase => "转会引进",
            TransferEventType::Retirement => "退役",
            TransferEventType::ContractExpire => "合同到期",
        }
    }
}

/// 转会事件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferEventStatus {
    Pending,
    Completed,
    Failed,
}

/// 新闻重要程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NewsImportance {
    /// 重磅新闻（能力≥85）
    Breaking,
    /// 主要新闻（能力75-84）
    Major,
    /// 普通新闻
    Normal,
    /// 次要新闻
    Minor,
}

impl NewsImportance {
    pub fn from_ability(ability: u8) -> Self {
        match ability {
            85..=100 => NewsImportance::Breaking,
            75..=84 => NewsImportance::Major,
            65..=74 => NewsImportance::Normal,
            _ => NewsImportance::Minor,
        }
    }
}

/// 合同类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    /// 新手合同（选秀）
    Rookie,
    /// 正式合同
    Standard,
}

/// 转会事件（用于新闻播报）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub round: u32,

    // 事件信息
    pub event_type: TransferEventType,
    pub status: TransferEventStatus,

    // 选手信息
    pub player_id: u64,
    pub player_name: String,
    pub position: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub market_value: u64,

    // 转会双方
    pub from_team_id: Option<u64>,
    pub from_team_name: Option<String>,
    pub to_team_id: Option<u64>,
    pub to_team_name: Option<String>,

    // 财务信息
    pub transfer_fee: u64,
    pub new_salary: Option<u64>,
    pub contract_years: Option<u32>,
    pub contract_type: ContractType,

    // 身价相关
    pub price_ratio: Option<f64>,

    // 新闻信息
    pub headline: String,
    pub description: String,
    pub importance: NewsImportance,

    // 竞争情况
    pub competing_teams: Vec<u64>,
    pub was_bidding_war: bool,

    pub created_at: Option<String>,
}

/// 转会轮次摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRoundSummary {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub round: u32,
    pub round_name: String,

    pub events_count: u32,
    pub transfers_count: u32,
    pub total_fees: u64,

    pub summary: String,

    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// 转会轮次名称
pub fn get_round_name(round: u32) -> &'static str {
    match round {
        0 => "赛季结算",
        1 => "合同到期与退役",
        2 => "自由球员争夺战",
        3 => "财政清洗",
        4 => "强队补强",
        5 => "收尾补救",
        _ => "未知轮次",
    }
}

// FinancialStatus 已在 team.rs 中定义，此处重新导出
pub use super::team::FinancialStatus;

/// 转会策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferStrategy {
    /// 积极买人
    AggressiveBuy,
    /// 观望
    Passive,
    /// 必须卖人
    MustSell,
    /// 强制清洗
    ForceClear,
}

/// 球队转会计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTransferPlan {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,

    // 财务状况
    pub balance: i64,
    pub financial_status: FinancialStatus,
    pub transfer_budget: i64,
    pub salary_space: i64,
    pub current_total_salary: i64,

    // 阵容状况
    pub roster_count: u32,
    pub avg_ability: f64,
    pub avg_age: f64,

    // 位置需求 (position -> need_score 0-100)
    pub position_needs: std::collections::HashMap<String, u32>,

    // 策略
    pub strategy: TransferStrategy,
    pub ambition: AmbitionLevel,

    // 标记
    pub must_sign: bool,   // 阵容不足5人
    pub must_clear: bool,  // 阵容超过10人
}

/// 阵容限制
pub const ROSTER_MIN: u32 = 5;
pub const ROSTER_MAX: u32 = 10;

/// 新手合同配置
pub const ROOKIE_CONTRACT_YEARS: u32 = 2;
pub const ROOKIE_SALARY_CAP: u64 = 50;  // 万元

/// 薪资计算：根据身价计算期望薪资
pub fn calculate_expected_salary(market_value: u64) -> SalaryExpectation {
    // 薪资约为身价的 5%-10%
    let ratio = if market_value > 3000 {
        0.055  // 顶级巨星：5.5%
    } else if market_value > 1000 {
        0.075  // 主力选手：7.5%
    } else {
        0.09   // 替补/新人：9%
    };

    let expected = (market_value as f64 * ratio) as u64;
    let expected = expected.max(15).min(400);  // 15-400万范围

    SalaryExpectation {
        expected,
        minimum: (expected as f64 * 0.75) as u64,
        ideal: (expected as f64 * 1.15) as u64,
    }
}

/// 薪资期望
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryExpectation {
    /// 期望薪资
    pub expected: u64,
    /// 最低接受
    pub minimum: u64,
    /// 理想薪资
    pub ideal: u64,
}

/// 退役判定结果
#[derive(Debug, Clone)]
pub struct RetirementCheck {
    pub should_retire: bool,
    pub probability: f64,
    pub reason: Option<String>,
}

/// 检查选手是否应该退役
pub fn check_retirement(age: u8, ability: u8) -> RetirementCheck {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // 35岁强制退役
    if age >= 35 {
        return RetirementCheck {
            should_retire: true,
            probability: 1.0,
            reason: Some("年龄达到35岁，强制退役".to_string()),
        };
    }

    // 32岁以上有30%概率退役
    if age >= 32 {
        let prob = 0.3;
        let roll: f64 = rng.gen();
        if roll < prob {
            return RetirementCheck {
                should_retire: true,
                probability: prob,
                reason: Some("年龄较大，选择退役".to_string()),
            };
        }
    }

    // 30岁以上且能力低于55，60%概率退役
    if age >= 30 && ability <= 55 {
        let prob = 0.6;
        let roll: f64 = rng.gen();
        if roll < prob {
            return RetirementCheck {
                should_retire: true,
                probability: prob,
                reason: Some("年龄较大且状态下滑，选择退役".to_string()),
            };
        }
    }

    // 30岁以上且能力低于65，30%概率退役
    if age >= 30 && ability <= 65 {
        let prob = 0.3;
        let roll: f64 = rng.gen();
        if roll < prob {
            return RetirementCheck {
                should_retire: true,
                probability: prob,
                reason: Some("年龄较大，选择退役".to_string()),
            };
        }
    }

    RetirementCheck {
        should_retire: false,
        probability: 0.0,
        reason: None,
    }
}

/// 计算合同年限
pub fn calculate_contract_years(age: u8, potential: u8, is_aggressive_buyer: bool) -> u32 {
    match age {
        // 年轻高潜力
        a if a <= 22 && potential >= 85 => if is_aggressive_buyer { 3 } else { 2 },
        // 巅峰期
        23..=27 => if is_aggressive_buyer { 3 } else { 2 },
        // 老将
        28..=30 => if rand::random::<bool>() { 2 } else { 1 },
        // 高龄
        _ => 1,
    }
}
