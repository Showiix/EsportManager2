use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::player::Position;

// ============================================
// 转会期相关
// ============================================

/// 转会期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferWindowStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl TransferWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferWindowStatus::Pending => "PENDING",
            TransferWindowStatus::InProgress => "IN_PROGRESS",
            TransferWindowStatus::Completed => "COMPLETED",
            TransferWindowStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "IN_PROGRESS" => TransferWindowStatus::InProgress,
            "COMPLETED" => TransferWindowStatus::Completed,
            "CANCELLED" => TransferWindowStatus::Cancelled,
            _ => TransferWindowStatus::Pending,
        }
    }
}

/// 转会期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferWindow {
    pub id: i64,
    pub save_id: String,
    pub season_id: i64,
    pub status: TransferWindowStatus,
    pub current_round: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
}

// ============================================
// 转会事件相关
// ============================================

/// 转会事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferEventType {
    SeasonSettlement,
    ContractRenewal,
    ContractTermination,
    FreeAgentSigning,
    TransferPurchase,
    PlayerRetirement,
    PlayerListed,
    PlayerRequestTransfer,
    EmergencySigning,
    DraftPickAuction,
    FinancialAdjustment,
    PlayerRelease,
}

impl TransferEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferEventType::SeasonSettlement => "SEASON_SETTLEMENT",
            TransferEventType::ContractRenewal => "CONTRACT_RENEWAL",
            TransferEventType::ContractTermination => "CONTRACT_TERMINATION",
            TransferEventType::FreeAgentSigning => "FREE_AGENT_SIGNING",
            TransferEventType::TransferPurchase => "TRANSFER_PURCHASE",
            TransferEventType::PlayerRetirement => "PLAYER_RETIREMENT",
            TransferEventType::PlayerListed => "PLAYER_LISTED",
            TransferEventType::PlayerRequestTransfer => "PLAYER_REQUEST_TRANSFER",
            TransferEventType::EmergencySigning => "EMERGENCY_SIGNING",
            TransferEventType::DraftPickAuction => "DRAFT_PICK_AUCTION",
            TransferEventType::FinancialAdjustment => "FINANCIAL_ADJUSTMENT",
            TransferEventType::PlayerRelease => "PLAYER_RELEASE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "SEASON_SETTLEMENT" => TransferEventType::SeasonSettlement,
            "CONTRACT_RENEWAL" => TransferEventType::ContractRenewal,
            "CONTRACT_TERMINATION" => TransferEventType::ContractTermination,
            "FREE_AGENT_SIGNING" => TransferEventType::FreeAgentSigning,
            "TRANSFER_PURCHASE" => TransferEventType::TransferPurchase,
            "PLAYER_RETIREMENT" => TransferEventType::PlayerRetirement,
            "PLAYER_LISTED" => TransferEventType::PlayerListed,
            "PLAYER_REQUEST_TRANSFER" => TransferEventType::PlayerRequestTransfer,
            "EMERGENCY_SIGNING" => TransferEventType::EmergencySigning,
            "DRAFT_PICK_AUCTION" => TransferEventType::DraftPickAuction,
            "FINANCIAL_ADJUSTMENT" => TransferEventType::FinancialAdjustment,
            "PLAYER_RELEASE" => TransferEventType::PlayerRelease,
            _ => TransferEventType::SeasonSettlement,
        }
    }
}

/// 事件等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventLevel {
    S,
    A,
    B,
    C,
}

impl EventLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventLevel::S => "S",
            EventLevel::A => "A",
            EventLevel::B => "B",
            EventLevel::C => "C",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "S" => EventLevel::S,
            "A" => EventLevel::A,
            "B" => EventLevel::B,
            _ => EventLevel::C,
        }
    }

    /// 根据球员能力和转会费自动判定等级
    pub fn from_ability_and_fee(ability: u8, transfer_fee: i64) -> Self {
        if ability >= 68 || transfer_fee > 5_000_000 {
            EventLevel::S
        } else if ability >= 65 || transfer_fee > 3_000_000 {
            EventLevel::A
        } else if ability >= 61 || transfer_fee > 1_500_000 {
            EventLevel::B
        } else {
            EventLevel::C
        }
    }
}

/// 转会事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub id: i64,
    pub window_id: i64,
    pub round: i64,
    pub event_type: String,
    pub level: String,
    pub player_id: i64,
    pub player_name: String,
    pub player_ability: i64,
    pub from_team_id: Option<i64>,
    pub from_team_name: Option<String>,
    pub to_team_id: Option<i64>,
    pub to_team_name: Option<String>,
    pub transfer_fee: i64,
    pub salary: i64,
    pub contract_years: i64,
    pub reason: Option<String>,
    pub created_at: String,
}

// ============================================
// AI球队性格
// ============================================

/// AI球队性格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AITeamPersonality {
    Aggressive,
    Conservative,
    Balanced,
    Development,
    WinNow,
}

impl AITeamPersonality {
    pub fn as_str(&self) -> &'static str {
        match self {
            AITeamPersonality::Aggressive => "AGGRESSIVE",
            AITeamPersonality::Conservative => "CONSERVATIVE",
            AITeamPersonality::Balanced => "BALANCED",
            AITeamPersonality::Development => "DEVELOPMENT",
            AITeamPersonality::WinNow => "WIN_NOW",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "AGGRESSIVE" => AITeamPersonality::Aggressive,
            "CONSERVATIVE" => AITeamPersonality::Conservative,
            "DEVELOPMENT" => AITeamPersonality::Development,
            "WIN_NOW" => AITeamPersonality::WinNow,
            _ => AITeamPersonality::Balanced,
        }
    }

    pub fn default_weights(&self) -> AIDecisionWeights {
        match self {
            AITeamPersonality::Aggressive => AIDecisionWeights {
                short_term_focus: 0.8,
                long_term_focus: 0.2,
                risk_tolerance: 0.7,
                youth_preference: 0.3,
                star_chasing: 0.9,
                bargain_hunting: 0.2,
            },
            AITeamPersonality::Conservative => AIDecisionWeights {
                short_term_focus: 0.4,
                long_term_focus: 0.6,
                risk_tolerance: 0.3,
                youth_preference: 0.5,
                star_chasing: 0.3,
                bargain_hunting: 0.8,
            },
            AITeamPersonality::Balanced => AIDecisionWeights::default(),
            AITeamPersonality::Development => AIDecisionWeights {
                short_term_focus: 0.3,
                long_term_focus: 0.7,
                risk_tolerance: 0.5,
                youth_preference: 0.9,
                star_chasing: 0.3,
                bargain_hunting: 0.7,
            },
            AITeamPersonality::WinNow => AIDecisionWeights {
                short_term_focus: 0.9,
                long_term_focus: 0.1,
                risk_tolerance: 0.6,
                youth_preference: 0.2,
                star_chasing: 0.8,
                bargain_hunting: 0.3,
            },
        }
    }
}

/// AI决策权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDecisionWeights {
    pub short_term_focus: f64,
    pub long_term_focus: f64,
    pub risk_tolerance: f64,
    pub youth_preference: f64,
    pub star_chasing: f64,
    pub bargain_hunting: f64,
}

impl Default for AIDecisionWeights {
    fn default() -> Self {
        Self {
            short_term_focus: 0.5,
            long_term_focus: 0.5,
            risk_tolerance: 0.5,
            youth_preference: 0.5,
            star_chasing: 0.5,
            bargain_hunting: 0.5,
        }
    }
}

/// AI球队性格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPersonalityConfig {
    pub id: i64,
    pub team_id: i64,
    pub save_id: String,
    pub personality: String,
    pub short_term_focus: f64,
    pub long_term_focus: f64,
    pub risk_tolerance: f64,
    pub youth_preference: f64,
    pub star_chasing: f64,
    pub bargain_hunting: f64,
    pub updated_at: String,
}

impl TeamPersonalityConfig {
    pub fn get_personality(&self) -> AITeamPersonality {
        AITeamPersonality::from_str(&self.personality)
    }

    pub fn get_weights(&self) -> AIDecisionWeights {
        AIDecisionWeights {
            short_term_focus: self.short_term_focus,
            long_term_focus: self.long_term_focus,
            risk_tolerance: self.risk_tolerance,
            youth_preference: self.youth_preference,
            star_chasing: self.star_chasing,
            bargain_hunting: self.bargain_hunting,
        }
    }
}

// ============================================
// 声望系统
// ============================================

/// 球队声望
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamReputation {
    pub team_id: i64,
    pub overall: i64,
    pub historical: i64,
    pub recent: i64,
    pub international: i64,
}

impl Default for TeamReputation {
    fn default() -> Self {
        Self {
            team_id: 0,
            overall: 30,
            historical: 30,
            recent: 30,
            international: 0,
        }
    }
}

// ============================================
// 需求分析
// ============================================

/// 需求优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NeedPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// 位置需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNeed {
    pub position: Position,
    pub current_count: i32,
    pub target_count: i32,
    pub gap: i32,
    pub current_avg_ability: f64,
    pub required_ability: u8,
    pub priority: NeedPriority,
}

/// 球队需求分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamNeedsAnalysis {
    pub team_id: i64,
    pub team_name: String,
    pub position_needs: HashMap<String, PositionNeed>,
    pub financial_status: String,
    pub avg_age: f64,
    pub roster_count: i32,
    pub power_rating: f64,
    pub priority_score: f64,
    pub listing_candidates: Vec<i64>,
    pub target_positions: Vec<Position>,
}

// ============================================
// 转会报价
// ============================================

/// 转会报价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOffer {
    pub team_id: i64,
    pub player_id: i64,
    pub offered_salary: i64,
    pub contract_years: i64,
    pub transfer_fee: i64,
    pub signing_bonus: i64,
    pub match_score: f64,
    pub priority: f64,
    pub target_region_id: Option<i64>,  // 目标球队赛区ID（用于跨赛区转会意愿计算）
}

// ============================================
// 转会配置
// ============================================

/// 转会配置
#[derive(Debug, Clone)]
pub struct TransferConfig {
    pub max_rounds: i64,
    pub max_transfers_per_round: i64,
    pub max_transfers_per_window: i64,
    pub negotiation_max_rounds: i64,
    /// 奢侈税起征人数（超过此数需缴税）
    pub luxury_tax_threshold: i64,
    /// 每超出1人的奢侈税金额（元/人）
    pub luxury_tax_per_player: i64,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            max_rounds: 7,
            max_transfers_per_round: 2,
            max_transfers_per_window: 10,
            negotiation_max_rounds: 3,
            luxury_tax_threshold: 10,
            luxury_tax_per_player: 5_000_000, // 500万/人
        }
    }
}

// ============================================
// 响应类型
// ============================================

/// 轮次结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round: i64,
    pub round_name: String,
    pub events: Vec<TransferEvent>,
    pub summary: String,
}

/// 转会期响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferWindowResponse {
    pub window_id: i64,
    pub current_round: i64,
    pub status: String,
    pub season_id: i64,
}

/// 转会窗口关闭验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferWindowCloseValidation {
    pub is_valid: bool,
    pub window_id: i64,
    pub issues: Vec<TransferCloseIssue>,
    pub message: String,
}

/// 转会窗口关闭问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCloseIssue {
    pub team_id: i64,
    pub team_name: String,
    pub issue_type: String,
    pub detail: String,
}

/// 轮次执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundExecutionResponse {
    pub round: i64,
    pub round_name: String,
    pub events: Vec<TransferEvent>,
    pub event_count: i64,
    pub next_round: Option<i64>,
    pub summary: String,
}

/// 快进响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastForwardResponse {
    pub completed_rounds: i64,
    pub total_events: i64,
    pub rounds: Vec<RoundResult>,
}

/// 转会报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferReport {
    pub window_id: i64,
    pub season_id: i64,
    pub total_events: i64,
    pub total_transfer_fee: i64,
    pub events_by_type: HashMap<String, i64>,
    pub events_by_level: HashMap<String, i64>,
    pub team_summaries: Vec<TeamTransferSummary>,
    pub top_events: Vec<TransferEvent>,
}

/// 球队转会摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTransferSummary {
    pub team_id: i64,
    pub team_name: String,
    pub players_in: i64,
    pub players_out: i64,
    pub money_spent: i64,
    pub money_earned: i64,
    pub net_spend: i64,
}

/// 球员挂牌记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListing {
    pub id: i64,
    pub player_id: i64,
    pub window_id: i64,
    pub listed_by_team_id: i64,
    pub listing_price: Option<i64>,
    pub min_accept_price: Option<i64>,
    pub status: String,
    pub listed_at: String,
    pub sold_at: Option<String>,
    pub sold_to_team_id: Option<i64>,
    pub actual_price: Option<i64>,
}

// ============================================
// 转会挂牌市场
// ============================================

/// 挂牌选手完整信息（用于挂牌市场页面）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMarketListingInfo {
    // 挂牌信息
    pub listing_id: i64,
    pub window_id: i64,
    pub listing_price: Option<i64>,
    pub min_accept_price: Option<i64>,
    pub listing_status: String,
    pub listed_at: String,
    pub sold_at: Option<String>,
    pub actual_price: Option<i64>,
    // 选手信息
    pub player_id: i64,
    pub player_name: String,
    pub position: Option<String>,
    pub age: i64,
    pub ability: i64,
    pub potential: i64,
    pub calculated_market_value: i64,
    // 挂牌战队信息
    pub listed_by_team_id: i64,
    pub listed_by_team_name: String,
    pub listed_by_region_code: Option<String>,
    // 买家战队信息（仅已售出时有值）
    pub sold_to_team_id: Option<i64>,
    pub sold_to_team_name: Option<String>,
    pub sold_to_region_code: Option<String>,
}

/// 自由球员信息（用于挂牌市场页面）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeAgentInfo {
    pub player_id: i64,
    pub player_name: String,
    pub position: Option<String>,
    pub age: i64,
    pub ability: i64,
    pub potential: i64,
    pub calculated_market_value: i64,
    pub salary: i64,
}

/// 转会挂牌市场综合数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMarketData {
    pub listings: Vec<TransferMarketListingInfo>,
    pub free_agents: Vec<FreeAgentInfo>,
    pub window_status: Option<String>,
    pub window_id: Option<i64>,
    pub current_round: Option<i64>,
    pub season_id: i64,
}

// ============================================
// 选手合同中心
// ============================================

/// 选手合同信息（用于合同中心页面）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerContractInfo {
    // 基本信息
    pub player_id: i64,
    pub player_name: String,
    pub position: Option<String>,
    pub age: i64,
    pub ability: i64,
    pub potential: i64,
    // 战队信息
    pub team_id: Option<i64>,
    pub team_name: Option<String>,
    pub team_short_name: Option<String>,
    pub region_code: Option<String>,
    // 合同信息
    pub salary: i64,
    pub contract_end_season: Option<i64>,
    pub join_season: Option<i64>,
    // 身价信息
    pub base_market_value: i64,
    pub calculated_market_value: i64,
    // 状态信息
    pub satisfaction: i64,
    pub loyalty: i64,
    pub is_starter: bool,
    pub status: String,
}

// ============================================
// 双向评估系统
// ============================================

/// 战队赛季评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamEvaluation {
    pub evaluation_id: i64,
    pub team_id: i64,
    pub current_rank: i32,
    pub last_rank: i32,
    pub stability_score: i32,
    pub strategy: String,
    pub urgency_level: String,
    pub roster_power: f64,
}

/// 选手评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEvaluation {
    pub player_id: i64,
    pub stay_score: f64,
    pub wants_to_leave: bool,
    pub leave_reason: String,
    pub should_list: bool,
    pub list_reason: String,
}

// ============================================
// 竞价分析
// ============================================

/// 单条竞价记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferBid {
    pub id: i64,
    pub window_id: i64,
    pub round: i64,
    pub player_id: i64,
    pub player_name: String,
    pub player_ability: i64,
    pub player_age: i64,
    pub player_position: Option<String>,
    pub from_team_id: Option<i64>,
    pub from_team_name: Option<String>,
    pub bid_team_id: i64,
    pub bid_team_name: String,
    pub bid_team_region_id: Option<i64>,
    pub offered_salary: i64,
    pub contract_years: i64,
    pub transfer_fee: i64,
    pub signing_bonus: i64,
    pub match_score: f64,
    pub willingness: f64,
    pub is_winner: bool,
    pub reject_reason: Option<String>,
}

/// 单个选手的竞价分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBidAnalysis {
    pub player_id: i64,
    pub player_name: String,
    pub player_ability: i64,
    pub player_age: i64,
    pub player_position: Option<String>,
    pub from_team_id: Option<i64>,
    pub from_team_name: Option<String>,
    pub round: i64,
    pub total_bids: i64,
    pub bids: Vec<TransferBid>,
    pub winner_team_name: Option<String>,
    pub outcome: String,
}

/// 竞价总览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidOverview {
    pub window_id: i64,
    pub round: Option<i64>,
    pub total_players: i64,
    pub total_bids: i64,
    pub successful_signings: i64,
    pub failed_signings: i64,
    pub avg_bids_per_player: f64,
    pub player_analyses: Vec<PlayerBidAnalysis>,
}

// ============================================
// 选手合同历史
// ============================================

/// 选手合同记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerContract {
    pub id: i64,
    pub save_id: String,
    pub player_id: i64,
    pub team_id: i64,
    pub contract_type: String,      // INITIAL / RENEWAL / FREE_AGENT / TRANSFER / EMERGENCY / DRAFT
    pub total_salary: i64,          // 总合同薪资（元）
    pub annual_salary: i64,         // 年薪 = total_salary / contract_years（元）
    pub contract_years: i64,        // 合同总年数
    pub start_season: i64,          // 签约赛季
    pub end_season: i64,            // 合同结束赛季
    pub transfer_fee: i64,          // 转会费（元，自由球员/续约为0）
    pub signing_bonus: i64,         // 签约奖金（元）
    pub is_active: bool,            // 是否为当前生效合同
    pub created_at: String,
}
