//! 转会市场状态模型
//!
//! 管理转会市场的整体状态，包括：
//! - 市场阶段和轮次
//! - 各球队的市场状态
//! - 自由球员池
//! - 转会事件记录

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== 市场阶段 ====================

/// 转会市场阶段（简化为5个大阶段）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketPhase {
    /// 阶段1：选手意愿生成（AI 分析每位选手的离队意愿）
    IntentionGeneration,
    /// 阶段2：战队策略生成（AI 为每支球队生成转会策略）
    StrategyGeneration,
    /// 阶段3：续约处理（处理不想离队选手的续约谈判）
    RenewalProcessing,
    /// 阶段4：自由市场（多轮报价和谈判，签约自由球员）
    FreeMarket,
    /// 阶段5：挖角转会（向有合同的85+选手发起挖人，需支付转会费）
    TransferRounds,
    /// 完成：转会窗口关闭
    Completed,
}

impl Default for MarketPhase {
    fn default() -> Self {
        MarketPhase::IntentionGeneration
    }
}

impl MarketPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            MarketPhase::IntentionGeneration => "选手意愿生成",
            MarketPhase::StrategyGeneration => "战队策略生成",
            MarketPhase::RenewalProcessing => "续约处理",
            MarketPhase::FreeMarket => "自由市场",
            MarketPhase::TransferRounds => "挖角转会",
            MarketPhase::Completed => "已完成",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MarketPhase::IntentionGeneration => "AI 分析每位选手的离队意愿和期望条件",
            MarketPhase::StrategyGeneration => "AI 为每支球队生成转会策略",
            MarketPhase::RenewalProcessing => "处理愿意留队选手的续约谈判",
            MarketPhase::FreeMarket => "多轮报价和谈判，球队签约自由球员",
            MarketPhase::TransferRounds => "向有合同的85+选手发起挖人，需支付转会费",
            MarketPhase::Completed => "转会窗口已关闭",
        }
    }

    /// 获取下一个阶段
    pub fn next(&self) -> Option<MarketPhase> {
        match self {
            MarketPhase::IntentionGeneration => Some(MarketPhase::StrategyGeneration),
            MarketPhase::StrategyGeneration => Some(MarketPhase::RenewalProcessing),
            MarketPhase::RenewalProcessing => Some(MarketPhase::FreeMarket),
            MarketPhase::FreeMarket => Some(MarketPhase::TransferRounds),
            MarketPhase::TransferRounds => Some(MarketPhase::Completed),
            MarketPhase::Completed => None,
        }
    }

    /// 阶段序号（用于进度展示）
    pub fn order(&self) -> u8 {
        match self {
            MarketPhase::IntentionGeneration => 0,
            MarketPhase::StrategyGeneration => 1,
            MarketPhase::RenewalProcessing => 2,
            MarketPhase::FreeMarket => 3,
            MarketPhase::TransferRounds => 4,
            MarketPhase::Completed => 5,
        }
    }

    /// 总阶段数
    pub fn total_phases() -> u8 {
        6
    }
}

// ==================== 球队市场状态 ====================

/// 单个球队的市场状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMarketState {
    pub team_id: u64,
    pub team_name: String,
    /// 赛季初始余额
    pub initial_balance: i64,
    /// 剩余可用预算（万）
    pub remaining_budget: u64,
    /// 已花费金额（万）
    pub spent_amount: u64,
    /// 薪资支出增量（万/年）
    pub salary_increase: u64,
    /// 当前阵容人数
    pub roster_count: u8,
    /// 最小阵容要求
    pub min_roster_size: u8,
    /// 最大阵容限制
    pub max_roster_size: u8,
    /// 进行中的谈判 ID 列表
    #[serde(default)]
    pub pending_negotiation_ids: Vec<u64>,
    /// 已完成的签约 ID 列表
    #[serde(default)]
    pub completed_signing_ids: Vec<u64>,
    /// 已离队选手 ID 列表
    #[serde(default)]
    pub departed_player_ids: Vec<u64>,
    /// 是否已生成策略
    pub strategy_generated: bool,
    /// 策略 ID
    pub strategy_id: Option<u64>,
    /// 是否需要紧急补人
    pub needs_emergency_signing: bool,
    /// 各位置的需求状态
    #[serde(default)]
    pub position_needs: HashMap<String, PositionNeed>,
}

impl TeamMarketState {
    pub fn new(team_id: u64, team_name: String, initial_balance: i64, roster_count: u8) -> Self {
        // 将元转换为万，然后取 60% 作为预算
        let balance_wan = initial_balance.max(0) as f64 / 10000.0;
        let budget = (balance_wan * 0.6) as u64;

        Self {
            team_id,
            team_name,
            initial_balance,
            remaining_budget: budget,
            spent_amount: 0,
            salary_increase: 0,
            roster_count,
            min_roster_size: 5,
            max_roster_size: 10,
            pending_negotiation_ids: Vec::new(),
            completed_signing_ids: Vec::new(),
            departed_player_ids: Vec::new(),
            strategy_generated: false,
            strategy_id: None,
            needs_emergency_signing: false,
            position_needs: HashMap::new(),
        }
    }

    /// 扣除预算
    pub fn deduct_budget(&mut self, amount: u64) -> bool {
        if self.remaining_budget >= amount {
            self.remaining_budget -= amount;
            self.spent_amount += amount;
            true
        } else {
            false
        }
    }

    /// 增加薪资支出
    pub fn add_salary(&mut self, salary: u64) {
        self.salary_increase += salary;
    }

    /// 记录签约
    pub fn record_signing(&mut self, negotiation_id: u64) {
        self.completed_signing_ids.push(negotiation_id);
        self.pending_negotiation_ids.retain(|&id| id != negotiation_id);
        self.roster_count += 1;
        self.update_emergency_status();
    }

    /// 记录离队
    pub fn record_departure(&mut self, player_id: u64) {
        self.departed_player_ids.push(player_id);
        self.roster_count = self.roster_count.saturating_sub(1);
        self.update_emergency_status();
    }

    /// 更新紧急补人状态
    fn update_emergency_status(&mut self) {
        self.needs_emergency_signing = self.roster_count < self.min_roster_size;
    }

    /// 是否还能签人
    pub fn can_sign_more(&self) -> bool {
        self.roster_count < self.max_roster_size && self.remaining_budget > 0
    }

    /// 获取阵容缺口
    pub fn roster_deficit(&self) -> u8 {
        self.min_roster_size.saturating_sub(self.roster_count)
    }
}

/// 位置需求状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNeed {
    pub position: String,
    /// 当前该位置人数
    pub current_count: u8,
    /// 目标人数
    pub target_count: u8,
    /// 需求评分 (0-100)
    pub need_score: u8,
    /// 是否为优先位置
    pub is_priority: bool,
}

// ==================== 市场状态 ====================

/// 转会市场整体状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMarketState {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    /// 当前阶段
    pub current_phase: MarketPhase,
    /// 当前轮次（在谈判阶段使用）
    pub current_round: u8,
    /// 谈判阶段的最大轮次
    pub max_negotiation_rounds: u8,
    /// 转会轮次阶段的当前轮次
    #[serde(default)]
    pub transfer_round: u8,
    /// 转会轮次阶段的最大轮次
    #[serde(default = "default_max_transfer_rounds")]
    pub max_transfer_rounds: u8,
    /// 自由球员 ID 列表
    #[serde(default)]
    pub free_agent_ids: Vec<u64>,
    /// 可挖人的有合同选手 ID 列表（想离队但有合同的选手）
    #[serde(default)]
    pub poachable_player_ids: Vec<u64>,
    /// 进行中的谈判 ID 列表
    #[serde(default)]
    pub active_negotiation_ids: Vec<u64>,
    /// 已完成的转会 ID 列表
    #[serde(default)]
    pub completed_transfer_ids: Vec<u64>,
    /// 各球队状态
    #[serde(default)]
    pub team_states: HashMap<u64, TeamMarketState>,
    /// 已生成意愿的选手数量
    pub intentions_generated: u32,
    /// 总选手数量
    pub total_players: u32,
    /// 已生成策略的球队数量
    pub strategies_generated: u32,
    /// 总球队数量
    pub total_teams: u32,
    /// 市场是否稳定（连续一轮无新交易）
    pub is_market_stable: bool,
    /// 连续无交易轮次计数
    pub stable_rounds_count: u8,
    /// 转会轮次是否稳定
    #[serde(default)]
    pub is_transfer_stable: bool,
    /// 转会轮次连续无交易计数
    #[serde(default)]
    pub transfer_stable_rounds_count: u8,
    /// 当前轮次已完成评估的战队 ID（用于重试失败的战队）
    #[serde(default)]
    pub evaluated_team_ids: Vec<u64>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

fn default_max_transfer_rounds() -> u8 {
    3
}

impl TransferMarketState {
    pub fn new(save_id: String, season_id: u64) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            save_id,
            season_id,
            current_phase: MarketPhase::IntentionGeneration,
            current_round: 0,
            max_negotiation_rounds: 5,
            transfer_round: 0,
            max_transfer_rounds: 3,
            free_agent_ids: Vec::new(),
            poachable_player_ids: Vec::new(),
            active_negotiation_ids: Vec::new(),
            completed_transfer_ids: Vec::new(),
            team_states: HashMap::new(),
            intentions_generated: 0,
            total_players: 0,
            strategies_generated: 0,
            total_teams: 0,
            is_market_stable: false,
            stable_rounds_count: 0,
            is_transfer_stable: false,
            transfer_stable_rounds_count: 0,
            evaluated_team_ids: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 进入下一个阶段
    pub fn advance_phase(&mut self) -> bool {
        if let Some(next) = self.current_phase.next() {
            self.current_phase = next;
            // 进入新阶段时重置轮次
            self.current_round = 0;
            // 重置已评估战队列表
            self.evaluated_team_ids.clear();
            // 如果进入 TransferRounds 阶段，也重置 transfer_round
            if next == MarketPhase::TransferRounds {
                self.transfer_round = 0;
            }
            self.updated_at = chrono::Utc::now().to_rfc3339();
            true
        } else {
            false
        }
    }

    /// 进入下一轮（在谈判阶段）
    pub fn advance_round(&mut self) {
        self.current_round += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 添加自由球员
    pub fn add_free_agent(&mut self, player_id: u64) {
        if !self.free_agent_ids.contains(&player_id) {
            self.free_agent_ids.push(player_id);
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 移除自由球员（已签约）
    pub fn remove_free_agent(&mut self, player_id: u64) {
        self.free_agent_ids.retain(|&id| id != player_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 添加活跃谈判
    pub fn add_active_negotiation(&mut self, negotiation_id: u64) {
        if !self.active_negotiation_ids.contains(&negotiation_id) {
            self.active_negotiation_ids.push(negotiation_id);
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 完成谈判
    pub fn complete_negotiation(&mut self, negotiation_id: u64) {
        self.active_negotiation_ids.retain(|&id| id != negotiation_id);
        self.completed_transfer_ids.push(negotiation_id);
        self.stable_rounds_count = 0; // 有交易，重置稳定计数
        self.is_market_stable = false;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 记录无交易轮次
    pub fn record_stable_round(&mut self) {
        self.stable_rounds_count += 1;
        if self.stable_rounds_count >= 2 {
            self.is_market_stable = true;
        }
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    // ==================== 转会轮次（挖人）相关方法 ====================

    /// 添加可挖人选手（想离队但有合同的选手）
    pub fn add_poachable_player(&mut self, player_id: u64) {
        if !self.poachable_player_ids.contains(&player_id) {
            self.poachable_player_ids.push(player_id);
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 移除可挖人选手（已完成交易）
    pub fn remove_poachable_player(&mut self, player_id: u64) {
        self.poachable_player_ids.retain(|&id| id != player_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 进入下一转会轮次
    pub fn advance_transfer_round(&mut self) {
        self.transfer_round += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 完成挖人交易（重置稳定计数）
    pub fn complete_transfer(&mut self, negotiation_id: u64) {
        self.active_negotiation_ids.retain(|&id| id != negotiation_id);
        self.completed_transfer_ids.push(negotiation_id);
        self.transfer_stable_rounds_count = 0; // 有交易，重置稳定计数
        self.is_transfer_stable = false;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 记录转会轮次无交易
    pub fn record_transfer_stable_round(&mut self) {
        self.transfer_stable_rounds_count += 1;
        if self.transfer_stable_rounds_count >= 2 {
            self.is_transfer_stable = true;
        }
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 检查是否应该结束转会轮次
    pub fn should_end_transfer_rounds(&self) -> bool {
        self.current_phase == MarketPhase::TransferRounds
            && (self.is_transfer_stable
                || self.transfer_round >= self.max_transfer_rounds
                || self.poachable_player_ids.is_empty())
    }

    /// 检查自由市场阶段是否应该结束
    /// （原 should_enter_last_chance，现用于判断 FreeMarket -> TransferRounds）
    pub fn should_enter_last_chance(&self) -> bool {
        self.current_phase == MarketPhase::FreeMarket
            && (self.is_market_stable || self.current_round >= self.max_negotiation_rounds)
    }

    /// 获取球队状态
    pub fn get_team_state(&self, team_id: u64) -> Option<&TeamMarketState> {
        self.team_states.get(&team_id)
    }

    /// 获取可变球队状态
    pub fn get_team_state_mut(&mut self, team_id: u64) -> Option<&mut TeamMarketState> {
        self.team_states.get_mut(&team_id)
    }

    /// 初始化球队状态
    pub fn init_team_state(&mut self, state: TeamMarketState) {
        self.team_states.insert(state.team_id, state);
        self.total_teams = self.team_states.len() as u32;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 获取整体进度百分比
    pub fn progress_percentage(&self) -> u8 {
        let phase_progress = self.current_phase.order() as f64 / MarketPhase::total_phases() as f64;
        (phase_progress * 100.0) as u8
    }

    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.current_phase == MarketPhase::Completed
    }

    /// 获取需要紧急补人的球队
    pub fn teams_needing_emergency_signing(&self) -> Vec<u64> {
        self.team_states
            .values()
            .filter(|ts| ts.needs_emergency_signing)
            .map(|ts| ts.team_id)
            .collect()
    }
}

// ==================== 转会事件 ====================

/// 市场事件类型（用于 LLM 转会市场）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketEventType {
    /// 合同到期，成为自由球员
    ContractExpired,
    /// 选手退役
    PlayerRetired,
    /// 选手申请转会
    TransferRequested,
    /// 续约成功
    RenewalSuccessful,
    /// 续约失败
    RenewalFailed,
    /// 球队发出报价（自由市场）
    OfferMade,
    /// 选手接受报价
    OfferAccepted,
    /// 选手拒绝报价
    OfferRejected,
    /// 选手还价
    CounterOffer,
    /// 球队加价
    OfferRaised,
    /// 球队退出竞争
    TeamWithdrew,
    /// 签约完成（自由球员）
    SigningCompleted,
    /// 挖人报价（有转会费）
    TransferOfferMade,
    /// 挖人成功（有转会费）
    TransferCompleted,
    /// 紧急签约
    EmergencySigning,
    /// 战队思考过程（用于弹窗展示）
    TeamThinking,
    /// 选手思考过程（用于弹窗展示）
    PlayerThinking,
}

impl MarketEventType {
    pub fn display_name(&self) -> &'static str {
        match self {
            MarketEventType::ContractExpired => "合同到期",
            MarketEventType::PlayerRetired => "选手退役",
            MarketEventType::TransferRequested => "申请转会",
            MarketEventType::RenewalSuccessful => "续约成功",
            MarketEventType::RenewalFailed => "续约失败",
            MarketEventType::OfferMade => "发出报价",
            MarketEventType::OfferAccepted => "接受报价",
            MarketEventType::OfferRejected => "拒绝报价",
            MarketEventType::CounterOffer => "选手还价",
            MarketEventType::OfferRaised => "球队加价",
            MarketEventType::TeamWithdrew => "退出竞争",
            MarketEventType::SigningCompleted => "签约完成",
            MarketEventType::TransferOfferMade => "挖人报价",
            MarketEventType::TransferCompleted => "转会完成",
            MarketEventType::EmergencySigning => "紧急签约",
            MarketEventType::TeamThinking => "战队分析",
            MarketEventType::PlayerThinking => "选手考虑",
        }
    }

    /// 事件重要程度 (1-5)
    pub fn importance(&self) -> u8 {
        match self {
            MarketEventType::SigningCompleted => 5,
            MarketEventType::TransferCompleted => 5,
            MarketEventType::OfferAccepted => 4,
            MarketEventType::TransferRequested => 4,
            MarketEventType::TransferOfferMade => 3,
            MarketEventType::RenewalSuccessful => 3,
            MarketEventType::RenewalFailed => 3,
            MarketEventType::OfferMade => 2,
            MarketEventType::OfferRaised => 2,
            MarketEventType::CounterOffer => 2,
            MarketEventType::OfferRejected => 2,
            MarketEventType::TeamWithdrew => 2,
            MarketEventType::ContractExpired => 1,
            MarketEventType::PlayerRetired => 1,
            MarketEventType::EmergencySigning => 3,
            MarketEventType::TeamThinking => 3,
            MarketEventType::PlayerThinking => 3,
        }
    }
}

/// 市场事件（用于 LLM 转会市场）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    /// 事件类型
    pub event_type: MarketEventType,
    /// 发生的阶段
    pub phase: MarketPhase,
    /// 发生的轮次
    pub round: u8,
    /// 相关选手 ID
    pub player_id: Option<u64>,
    /// 选手名称
    pub player_name: Option<String>,
    /// 相关球队 ID（主要）
    pub team_id: Option<u64>,
    /// 球队名称
    pub team_name: Option<String>,
    /// 第二相关球队 ID（如交易对手）
    pub secondary_team_id: Option<u64>,
    /// 第二球队名称
    pub secondary_team_name: Option<String>,
    /// 涉及金额（万）
    pub amount: Option<u64>,
    /// 事件标题
    pub title: String,
    /// 事件描述
    pub description: String,
    /// AI 分析（可选）
    pub ai_analysis: Option<String>,
    /// 创建时间
    pub created_at: String,
}

impl MarketEvent {
    pub fn new(
        save_id: String,
        season_id: u64,
        event_type: MarketEventType,
        phase: MarketPhase,
        round: u8,
    ) -> Self {
        Self {
            id: 0,
            save_id,
            season_id,
            event_type,
            phase,
            round,
            player_id: None,
            player_name: None,
            team_id: None,
            team_name: None,
            secondary_team_id: None,
            secondary_team_name: None,
            amount: None,
            title: String::new(),
            description: String::new(),
            ai_analysis: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 设置选手信息
    pub fn with_player(mut self, player_id: u64, player_name: String) -> Self {
        self.player_id = Some(player_id);
        self.player_name = Some(player_name);
        self
    }

    /// 设置球队信息
    pub fn with_team(mut self, team_id: u64, team_name: String) -> Self {
        self.team_id = Some(team_id);
        self.team_name = Some(team_name);
        self
    }

    /// 设置第二球队信息
    pub fn with_secondary_team(mut self, team_id: u64, team_name: String) -> Self {
        self.secondary_team_id = Some(team_id);
        self.secondary_team_name = Some(team_name);
        self
    }

    /// 设置金额
    pub fn with_amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// 设置标题和描述
    pub fn with_content(mut self, title: String, description: String) -> Self {
        self.title = title;
        self.description = description;
        self
    }

    /// 设置 AI 分析
    pub fn with_ai_analysis(mut self, analysis: String) -> Self {
        self.ai_analysis = Some(analysis);
        self
    }
}

// ==================== API 响应类型 ====================

/// 市场状态摘要（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStateSummary {
    pub save_id: String,
    pub season_id: u64,
    pub current_phase: MarketPhase,
    pub phase_name: String,
    pub phase_description: String,
    pub current_round: u8,
    pub max_rounds: u8,
    /// 转会轮次阶段的当前轮次
    pub transfer_round: u8,
    /// 转会轮次阶段的最大轮次
    pub max_transfer_rounds: u8,
    pub progress_percentage: u8,
    pub free_agents_count: usize,
    /// 可挖人选手数量（有合同但想离队）
    pub poachable_players_count: usize,
    pub active_negotiations_count: usize,
    pub completed_transfers_count: usize,
    pub intentions_progress: String,
    pub strategies_progress: String,
    pub is_market_stable: bool,
    /// 转会轮次是否稳定
    pub is_transfer_stable: bool,
    pub is_completed: bool,
}

impl From<&TransferMarketState> for MarketStateSummary {
    fn from(state: &TransferMarketState) -> Self {
        Self {
            save_id: state.save_id.clone(),
            season_id: state.season_id,
            current_phase: state.current_phase,
            phase_name: state.current_phase.display_name().to_string(),
            phase_description: state.current_phase.description().to_string(),
            current_round: state.current_round,
            max_rounds: state.max_negotiation_rounds,
            transfer_round: state.transfer_round,
            max_transfer_rounds: state.max_transfer_rounds,
            progress_percentage: state.progress_percentage(),
            free_agents_count: state.free_agent_ids.len(),
            poachable_players_count: state.poachable_player_ids.len(),
            active_negotiations_count: state.active_negotiation_ids.len(),
            completed_transfers_count: state.completed_transfer_ids.len(),
            intentions_progress: format!("{}/{}", state.intentions_generated, state.total_players),
            strategies_progress: format!("{}/{}", state.strategies_generated, state.total_teams),
            is_market_stable: state.is_market_stable,
            is_transfer_stable: state.is_transfer_stable,
            is_completed: state.is_completed(),
        }
    }
}

/// 球队市场状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMarketSummary {
    pub team_id: u64,
    pub team_name: String,
    pub remaining_budget: u64,
    pub spent_amount: u64,
    pub roster_count: u8,
    pub min_roster_size: u8,
    pub pending_negotiations: usize,
    pub completed_signings: usize,
    pub departed_players: usize,
    pub needs_emergency_signing: bool,
    pub strategy_generated: bool,
}

impl From<&TeamMarketState> for TeamMarketSummary {
    fn from(state: &TeamMarketState) -> Self {
        Self {
            team_id: state.team_id,
            team_name: state.team_name.clone(),
            remaining_budget: state.remaining_budget,
            spent_amount: state.spent_amount,
            roster_count: state.roster_count,
            min_roster_size: state.min_roster_size,
            pending_negotiations: state.pending_negotiation_ids.len(),
            completed_signings: state.completed_signing_ids.len(),
            departed_players: state.departed_player_ids.len(),
            needs_emergency_signing: state.needs_emergency_signing,
            strategy_generated: state.strategy_generated,
        }
    }
}

/// 轮次执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundExecutionResult {
    pub phase: MarketPhase,
    pub round: u8,
    pub phase_changed: bool,
    pub new_phase: Option<MarketPhase>,
    pub events: Vec<MarketEvent>,
    pub new_negotiations: usize,
    pub completed_signings: usize,
    pub summary: String,
}

/// 生成进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationProgress {
    pub task_type: String,
    pub current: u32,
    pub total: u32,
    pub percentage: u8,
    pub current_item: Option<String>,
    pub is_completed: bool,
    pub errors: Vec<String>,
}

impl GenerationProgress {
    pub fn new(task_type: &str, total: u32) -> Self {
        Self {
            task_type: task_type.to_string(),
            current: 0,
            total,
            percentage: 0,
            current_item: None,
            is_completed: false,
            errors: Vec::new(),
        }
    }

    pub fn advance(&mut self, item_name: Option<String>) {
        self.current += 1;
        self.current_item = item_name;
        self.percentage = ((self.current as f64 / self.total as f64) * 100.0) as u8;
        if self.current >= self.total {
            self.is_completed = true;
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

// ==================== 续约决策 ====================

/// LLM 生成的续约决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalDecision {
    /// 选手 ID
    pub player_id: u64,
    /// 选手名称
    pub player_name: String,
    /// 球队 ID
    pub team_id: u64,
    /// 球队名称
    pub team_name: String,
    /// 球队是否想续约
    pub team_wants_renewal: bool,
    /// 球队不想续约的原因
    pub team_rejection_reason: Option<String>,
    /// 球队报价薪资（万/年）
    pub offered_salary: u64,
    /// 球队报价合同年限
    pub offered_years: u8,
    /// 选手是否接受续约
    pub player_accepts: bool,
    /// 选手拒绝原因
    pub player_rejection_reason: Option<String>,
    /// 最终续约成功
    pub renewal_successful: bool,
    /// 最终合同薪资（如果成功）
    pub final_salary: Option<u64>,
    /// 最终合同年限（如果成功）
    pub final_years: Option<u8>,
    /// 球队分析步骤
    pub team_analysis: Vec<RenewalAnalysisStep>,
    /// 选手分析步骤
    pub player_analysis: Vec<RenewalAnalysisStep>,
    /// 综合说明
    pub summary: String,
}

/// 续约分析步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalAnalysisStep {
    /// 步骤名称
    pub step_name: String,
    /// 使用的数据
    pub data_used: String,
    /// 评估结果
    pub result: String,
    /// 影响
    pub impact: String,
}

/// 续约处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalProcessingResult {
    /// 处理的选手数量
    pub total_processed: u32,
    /// 续约成功数量
    pub successful_renewals: u32,
    /// 续约失败数量（球队不想续约）
    pub team_rejections: u32,
    /// 续约失败数量（选手拒绝）
    pub player_rejections: u32,
    /// 每个选手的续约决策
    pub decisions: Vec<RenewalDecision>,
    /// 错误列表
    pub errors: Vec<String>,
}
