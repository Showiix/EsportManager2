//! 简化版转会系统数据模型
//!
//! 移除了 LLM 和 GM 人格系统，使用纯规则引擎。
//! 4阶段流程：市场分析 → 策略生成 → 续约窗口 → 自由市场

use serde::{Deserialize, Serialize};

// ==================== 转会阶段 ====================

/// 转会市场阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferPhase {
    /// 阶段1: 市场分析 - 计算选手意愿
    MarketAnalysis,
    /// 阶段2: 策略生成 - 球队策略自动计算
    StrategyGeneration,
    /// 阶段3: 续约窗口 - 处理合同到期选手
    RenewalWindow,
    /// 阶段4: 自由市场 - 报价和签约
    FreeMarket,
    /// 已完成
    Completed,
}

impl TransferPhase {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MarketAnalysis => "市场分析",
            Self::StrategyGeneration => "策略生成",
            Self::RenewalWindow => "续约窗口",
            Self::FreeMarket => "自由市场",
            Self::Completed => "已完成",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::MarketAnalysis => "分析选手转会意愿和市场状态",
            Self::StrategyGeneration => "根据球队实力自动生成转会策略",
            Self::RenewalWindow => "处理合同到期选手的续约谈判",
            Self::FreeMarket => "球队报价、选手选择、完成签约",
            Self::Completed => "转会窗口已关闭",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::MarketAnalysis => Some(Self::StrategyGeneration),
            Self::StrategyGeneration => Some(Self::RenewalWindow),
            Self::RenewalWindow => Some(Self::FreeMarket),
            Self::FreeMarket => Some(Self::Completed),
            Self::Completed => None,
        }
    }

    pub fn index(&self) -> u8 {
        match self {
            Self::MarketAnalysis => 0,
            Self::StrategyGeneration => 1,
            Self::RenewalWindow => 2,
            Self::FreeMarket => 3,
            Self::Completed => 4,
        }
    }
}

impl Default for TransferPhase {
    fn default() -> Self {
        Self::MarketAnalysis
    }
}

// ==================== 选手转会意愿 ====================

/// 选手转会意愿
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferIntent {
    /// 愿意留队（满意度80+ 且 忠诚度70+）
    StayHappy,
    /// 中立（满意度60-79）
    StayNeutral,
    /// 开放报价（满意度40-59）
    OpenToOffers,
    /// 想离队（满意度20-39）
    WantsOut,
    /// 必须离队（满意度<20 或 合同到期）
    MustLeave,
}

impl TransferIntent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StayHappy => "愿意留队",
            Self::StayNeutral => "中立",
            Self::OpenToOffers => "开放报价",
            Self::WantsOut => "想离队",
            Self::MustLeave => "必须离队",
        }
    }

    /// 是否愿意考虑其他球队的报价
    pub fn accepts_offers(&self) -> bool {
        matches!(self, Self::OpenToOffers | Self::WantsOut | Self::MustLeave)
    }

    /// 是否可以续约
    pub fn can_renew(&self) -> bool {
        matches!(self, Self::StayHappy | Self::StayNeutral | Self::OpenToOffers)
    }

    /// 计算转会意愿
    pub fn from_satisfaction_loyalty(satisfaction: u8, loyalty: u8, contract_expired: bool) -> Self {
        if contract_expired && satisfaction < 50 {
            return Self::MustLeave;
        }
        if satisfaction < 20 {
            return Self::MustLeave;
        }
        if satisfaction < 40 {
            return Self::WantsOut;
        }
        if satisfaction < 60 {
            return Self::OpenToOffers;
        }
        if satisfaction < 80 {
            return Self::StayNeutral;
        }
        if loyalty >= 70 {
            Self::StayHappy
        } else {
            Self::StayNeutral
        }
    }
}

/// 离队原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepartureReason {
    /// 缺少上场时间
    LackOfPlaytime,
    /// 球队战绩差
    PoorTeamPerformance,
    /// 薪资不满
    SalaryDissatisfaction,
    /// 追求冠军
    SeekingChampionship,
    /// 寻找发展机会
    SeekingOpportunity,
    /// 合同到期
    ContractExpired,
    /// 无特定原因
    None,
}

impl DepartureReason {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LackOfPlaytime => "缺少上场时间",
            Self::PoorTeamPerformance => "球队战绩差",
            Self::SalaryDissatisfaction => "薪资不满",
            Self::SeekingChampionship => "追求冠军",
            Self::SeekingOpportunity => "寻找机会",
            Self::ContractExpired => "合同到期",
            Self::None => "无",
        }
    }
}

// ==================== 球队策略 ====================

/// 球队策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    /// 争冠型：平均能力85+ 且 排名前4 且 有预算
    Contender,
    /// 发展型：平均能力70-80，有发展空间
    Developing,
    /// 重建型：平均能力<70 或 排名垫底
    Rebuilding,
    /// 维持型：其他情况
    Maintaining,
}

impl StrategyType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Contender => "争冠",
            Self::Developing => "发展",
            Self::Rebuilding => "重建",
            Self::Maintaining => "维持",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Contender => "追求冠军，引进顶级选手",
            Self::Developing => "培养新人，稳步提升",
            Self::Rebuilding => "大换血，清洗高薪老将",
            Self::Maintaining => "保持阵容，小幅调整",
        }
    }

    /// 最低目标能力
    pub fn min_target_ability(&self) -> u8 {
        match self {
            Self::Contender => 80,
            Self::Developing => 65,
            Self::Rebuilding => 55,
            Self::Maintaining => 70,
        }
    }

    /// 最大目标年龄
    pub fn max_target_age(&self) -> u8 {
        match self {
            Self::Contender => 30,
            Self::Developing => 24,
            Self::Rebuilding => 23,
            Self::Maintaining => 28,
        }
    }

    /// 出价上限（相对身价倍数）
    pub fn max_offer_multiplier(&self) -> f64 {
        match self {
            Self::Contender => 1.3,
            Self::Developing => 1.0,
            Self::Rebuilding => 0.8,
            Self::Maintaining => 1.1,
        }
    }
}

// ==================== 选手市场状态 ====================

/// 选手在转会市场的状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerMarketStatus {
    /// 有合同，不可转会
    Contracted,
    /// 自由球员
    FreeAgent,
    /// 愿意转会（有合同但想离队）
    WillingToTransfer,
    /// 已收到报价
    HasOffers,
    /// 已签约
    Signed,
}

impl PlayerMarketStatus {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Contracted => "有合同",
            Self::FreeAgent => "自由球员",
            Self::WillingToTransfer => "可转会",
            Self::HasOffers => "有报价",
            Self::Signed => "已签约",
        }
    }
}

// ==================== 数据结构 ====================

/// 转会市场状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMarketState {
    pub save_id: String,
    pub season_id: u64,
    pub phase: TransferPhase,
    pub current_round: u32,
    pub max_rounds: u32,
    pub no_signing_rounds: u32,  // 连续无签约轮数
    pub free_agents_count: u32,
    pub willing_to_transfer_count: u32,
    pub active_negotiations: u32,
    pub completed_signings: u32,
    pub is_initialized: bool,
}

impl TransferMarketState {
    pub fn new(save_id: String, season_id: u64) -> Self {
        Self {
            save_id,
            season_id,
            phase: TransferPhase::MarketAnalysis,
            current_round: 0,
            max_rounds: 5,
            no_signing_rounds: 0,
            free_agents_count: 0,
            willing_to_transfer_count: 0,
            active_negotiations: 0,
            completed_signings: 0,
            is_initialized: false,
        }
    }

    /// 检查自由市场是否应该结束
    pub fn should_end_free_market(&self) -> bool {
        self.current_round >= self.max_rounds || self.no_signing_rounds >= 2
    }
}

/// 选手转会信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTransferInfo {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub salary: u64,
    pub market_value: u64,
    pub contract_end_season: Option<u64>,
    pub satisfaction: u8,
    pub loyalty: u8,
    pub intent: TransferIntent,
    pub departure_reason: DepartureReason,
    pub status: PlayerMarketStatus,
    pub expected_salary: u64,  // 期望薪资（万/年）
}

/// 球队转会策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTransferStrategy {
    pub team_id: u64,
    pub team_name: String,
    pub region_id: u64,
    pub strategy_type: StrategyType,
    pub avg_ability: f64,
    pub roster_count: u32,
    pub budget: u64,
    pub salary_cap_space: i64,
    pub position_needs: Vec<PositionNeed>,
    pub targets: Vec<TransferTarget>,
    pub sell_candidates: Vec<SellCandidate>,
    pub summary: String,
}

/// 位置需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNeed {
    pub position: String,
    pub current_count: u32,
    pub need_score: u32,  // 0-100
    pub reason: String,
}

/// 引援目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTarget {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub current_team_id: Option<u64>,
    pub current_team_name: Option<String>,
    pub market_value: u64,
    pub max_offer: u64,
    pub priority: u8,  // 1-10
    pub reason: String,
}

/// 出售候选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellCandidate {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub age: u8,
    pub ability: u8,
    pub salary: u64,
    pub market_value: u64,
    pub min_price: u64,
    pub sell_reason: String,
}

/// 报价信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOffer {
    pub id: u64,
    pub player_id: u64,
    pub from_team_id: u64,
    pub from_team_name: String,
    pub salary_offer: u64,  // 万/年
    pub contract_years: u8,
    pub transfer_fee: u64,  // 转会费（如果有合同）
    pub is_starter_promised: bool,
    pub round: u32,
    pub status: OfferStatus,
    pub created_at: String,
}

/// 报价状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferStatus {
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

impl OfferStatus {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pending => "待处理",
            Self::Accepted => "已接受",
            Self::Rejected => "已拒绝",
            Self::Withdrawn => "已撤回",
        }
    }
}

/// 续约结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalResult {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub success: bool,
    pub new_salary: Option<u64>,
    pub new_years: Option<u8>,
    pub failure_reason: Option<String>,
}

/// 转会事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub id: u64,
    pub event_type: TransferEventType,
    pub round: u32,
    pub player_id: Option<u64>,
    pub player_name: Option<String>,
    pub from_team_id: Option<u64>,
    pub from_team_name: Option<String>,
    pub to_team_id: Option<u64>,
    pub to_team_name: Option<String>,
    pub amount: Option<u64>,
    pub title: String,
    pub description: String,
    pub created_at: String,
}

/// 转会事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferEventType {
    /// 合同到期
    ContractExpired,
    /// 续约成功
    RenewalSuccess,
    /// 续约失败
    RenewalFailed,
    /// 发出报价
    OfferMade,
    /// 接受报价
    OfferAccepted,
    /// 拒绝报价
    OfferRejected,
    /// 签约完成
    SigningCompleted,
    /// 阶段完成
    PhaseCompleted,
}

impl TransferEventType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ContractExpired => "合同到期",
            Self::RenewalSuccess => "续约成功",
            Self::RenewalFailed => "续约失败",
            Self::OfferMade => "发出报价",
            Self::OfferAccepted => "接受报价",
            Self::OfferRejected => "拒绝报价",
            Self::SigningCompleted => "签约完成",
            Self::PhaseCompleted => "阶段完成",
        }
    }
}

/// 转会市场摘要（返回给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMarketSummary {
    pub save_id: String,
    pub season_id: u64,
    pub phase: TransferPhase,
    pub phase_name: String,
    pub phase_description: String,
    pub current_round: u32,
    pub max_rounds: u32,
    pub progress_percentage: u8,
    pub free_agents_count: u32,
    pub willing_to_transfer_count: u32,
    pub active_negotiations: u32,
    pub completed_signings: u32,
    pub can_advance: bool,
    pub is_completed: bool,
}

impl From<&TransferMarketState> for TransferMarketSummary {
    fn from(state: &TransferMarketState) -> Self {
        let progress = match state.phase {
            TransferPhase::MarketAnalysis => 0,
            TransferPhase::StrategyGeneration => 25,
            TransferPhase::RenewalWindow => 50,
            TransferPhase::FreeMarket => {
                50 + (state.current_round as u8 * 10).min(40)
            }
            TransferPhase::Completed => 100,
        };

        Self {
            save_id: state.save_id.clone(),
            season_id: state.season_id,
            phase: state.phase,
            phase_name: state.phase.name().to_string(),
            phase_description: state.phase.description().to_string(),
            current_round: state.current_round,
            max_rounds: state.max_rounds,
            progress_percentage: progress,
            free_agents_count: state.free_agents_count,
            willing_to_transfer_count: state.willing_to_transfer_count,
            active_negotiations: state.active_negotiations,
            completed_signings: state.completed_signings,
            can_advance: state.phase != TransferPhase::Completed,
            is_completed: state.phase == TransferPhase::Completed,
        }
    }
}

/// 轮次执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round: u32,
    pub phase: TransferPhase,
    pub phase_changed: bool,
    pub new_phase: Option<TransferPhase>,
    pub offers_made: u32,
    pub signings_completed: u32,
    pub events: Vec<TransferEvent>,
    pub summary: String,
}
