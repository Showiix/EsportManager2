//! 转会谈判模型
//!
//! 用于存储和管理转会市场中的谈判记录，包括：
//! - 球队发出的报价
//! - 选手对报价的回应
//! - 多轮谈判的完整历史
//! - AI 分析步骤展示

use serde::{Deserialize, Serialize};
use super::player_transfer_strategy::AnalysisStep;

// ==================== 枚举定义 ====================

/// 谈判状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NegotiationStatus {
    /// 进行中
    Open,
    /// 已接受（选手接受了某个报价）
    Accepted,
    /// 已拒绝（选手拒绝所有报价）
    Rejected,
    /// 已过期（转会窗口结束）
    Expired,
    /// 已撤回（球队或选手撤回）
    Withdrawn,
}

impl Default for NegotiationStatus {
    fn default() -> Self {
        NegotiationStatus::Open
    }
}

impl NegotiationStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            NegotiationStatus::Open => "进行中",
            NegotiationStatus::Accepted => "已成交",
            NegotiationStatus::Rejected => "已拒绝",
            NegotiationStatus::Expired => "已过期",
            NegotiationStatus::Withdrawn => "已撤回",
        }
    }

    /// 是否为终态（不可再变更）
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            NegotiationStatus::Accepted
                | NegotiationStatus::Rejected
                | NegotiationStatus::Expired
                | NegotiationStatus::Withdrawn
        )
    }
}

/// 单次报价状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OfferStatus {
    /// 等待回应
    Pending,
    /// 已接受
    Accepted,
    /// 已拒绝
    Rejected,
    /// 选手还价
    Countered,
    /// 已撤回
    Withdrawn,
    /// 已过期
    Expired,
}

impl Default for OfferStatus {
    fn default() -> Self {
        OfferStatus::Pending
    }
}

impl OfferStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            OfferStatus::Pending => "等待回应",
            OfferStatus::Accepted => "已接受",
            OfferStatus::Rejected => "已拒绝",
            OfferStatus::Countered => "已还价",
            OfferStatus::Withdrawn => "已撤回",
            OfferStatus::Expired => "已过期",
        }
    }
}

/// 选手回应类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResponseType {
    /// 接受报价
    Accept,
    /// 拒绝报价
    Reject,
    /// 还价（提出反报价）
    Counter,
    /// 继续等待（等其他球队报价）
    Wait,
}

impl ResponseType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ResponseType::Accept => "接受",
            ResponseType::Reject => "拒绝",
            ResponseType::Counter => "还价",
            ResponseType::Wait => "继续等待",
        }
    }
}

// ==================== 报价结构 ====================

/// 球队发出的单次报价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: u64,
    /// 所属谈判 ID
    pub negotiation_id: u64,
    /// 发出报价的球队 ID
    pub from_team_id: u64,
    /// 球队名称
    pub from_team_name: String,
    /// 目标选手 ID
    pub to_player_id: u64,
    /// 第几轮报价（从 1 开始）
    pub round: u8,
    /// 薪资报价（万/年）
    pub salary_offer: u64,
    /// 合同年限
    pub contract_years: u8,
    /// 是否保证首发位置
    pub guarantee_starter: bool,
    /// 签字费（万）
    pub signing_bonus: u64,
    /// 转会费（万，如果选手有合同）
    pub transfer_fee: u64,
    /// 报价状态
    pub status: OfferStatus,
    /// 球队的报价理由（AI 生成）
    pub offer_reasoning: String,
    /// AI 分析步骤（球队为什么出这个价）
    #[serde(default)]
    pub analysis_steps: Vec<AnalysisStep>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl Offer {
    /// 创建新报价
    pub fn new(
        negotiation_id: u64,
        from_team_id: u64,
        from_team_name: String,
        to_player_id: u64,
        round: u8,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            negotiation_id,
            from_team_id,
            from_team_name,
            to_player_id,
            round,
            salary_offer: 0,
            contract_years: 1,
            guarantee_starter: false,
            signing_bonus: 0,
            transfer_fee: 0,
            status: OfferStatus::Pending,
            offer_reasoning: String::new(),
            analysis_steps: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 计算报价的总价值（用于比较）
    pub fn total_value(&self) -> u64 {
        self.salary_offer * self.contract_years as u64 + self.signing_bonus
    }

    /// 是否为待处理状态
    pub fn is_pending(&self) -> bool {
        self.status == OfferStatus::Pending
    }
}

// ==================== 回应结构 ====================

/// 选手对报价的回应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferResponse {
    pub id: u64,
    /// 回应的报价 ID
    pub offer_id: u64,
    /// 选手 ID
    pub player_id: u64,
    /// 回应类型
    pub response_type: ResponseType,
    /// 还价薪资（当 response_type 为 Counter 时有效）
    pub counter_salary: Option<u64>,
    /// 还价年限
    pub counter_years: Option<u8>,
    /// 是否要求首发
    pub counter_starter: Option<bool>,
    /// 回应理由（AI 生成，用于展示）
    pub reasoning: String,
    /// AI 分析步骤（选手是如何做出决定的）
    #[serde(default)]
    pub analysis_steps: Vec<AnalysisStep>,
    /// 回应时间
    pub responded_at: String,
}

impl OfferResponse {
    /// 创建接受回应
    pub fn accept(offer_id: u64, player_id: u64, reasoning: String) -> Self {
        Self {
            id: 0,
            offer_id,
            player_id,
            response_type: ResponseType::Accept,
            counter_salary: None,
            counter_years: None,
            counter_starter: None,
            reasoning,
            analysis_steps: Vec::new(),
            responded_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建拒绝回应
    pub fn reject(offer_id: u64, player_id: u64, reasoning: String) -> Self {
        Self {
            id: 0,
            offer_id,
            player_id,
            response_type: ResponseType::Reject,
            counter_salary: None,
            counter_years: None,
            counter_starter: None,
            reasoning,
            analysis_steps: Vec::new(),
            responded_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建还价回应
    pub fn counter(
        offer_id: u64,
        player_id: u64,
        counter_salary: u64,
        counter_years: u8,
        counter_starter: bool,
        reasoning: String,
    ) -> Self {
        Self {
            id: 0,
            offer_id,
            player_id,
            response_type: ResponseType::Counter,
            counter_salary: Some(counter_salary),
            counter_years: Some(counter_years),
            counter_starter: Some(counter_starter),
            reasoning,
            analysis_steps: Vec::new(),
            responded_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建等待回应
    pub fn wait(offer_id: u64, player_id: u64, reasoning: String) -> Self {
        Self {
            id: 0,
            offer_id,
            player_id,
            response_type: ResponseType::Wait,
            counter_salary: None,
            counter_years: None,
            counter_starter: None,
            reasoning,
            analysis_steps: Vec::new(),
            responded_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ==================== 谈判记录 ====================

/// 完整的谈判记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Negotiation {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    /// 选手 ID
    pub player_id: u64,
    /// 选手名称
    pub player_name: String,
    /// 选手位置
    pub player_position: String,
    /// 选手能力
    pub player_ability: u8,
    /// 原球队 ID（自由球员为 None）
    pub from_team_id: Option<u64>,
    /// 原球队名称
    pub from_team_name: Option<String>,
    /// 谈判状态
    pub status: NegotiationStatus,
    /// 当前轮次
    pub current_round: u8,
    /// 最大轮次限制
    pub max_rounds: u8,
    /// 是否为挖人谈判（有合同选手转会）
    #[serde(default)]
    pub is_transfer: bool,
    /// 转会费（万，仅挖人谈判有效）
    #[serde(default)]
    pub transfer_fee: Option<u64>,
    /// 所有报价历史（按时间排序）
    #[serde(default)]
    pub offers: Vec<Offer>,
    /// 所有回应历史
    #[serde(default)]
    pub responses: Vec<OfferResponse>,
    /// 最终签约球队 ID
    pub final_team_id: Option<u64>,
    /// 最终签约球队名称
    pub final_team_name: Option<String>,
    /// 最终签约条件
    pub final_salary: Option<u64>,
    pub final_years: Option<u8>,
    pub final_starter: Option<bool>,
    /// 最终转会费（万，仅挖人谈判）
    #[serde(default)]
    pub final_transfer_fee: Option<u64>,
    /// 参与竞争的球队 ID 列表
    #[serde(default)]
    pub competing_team_ids: Vec<u64>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl Negotiation {
    /// 创建新的谈判记录
    pub fn new(
        save_id: String,
        season_id: u64,
        player_id: u64,
        player_name: String,
        player_position: String,
        player_ability: u8,
        from_team_id: Option<u64>,
        from_team_name: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            save_id,
            season_id,
            player_id,
            player_name,
            player_position,
            player_ability,
            from_team_id,
            from_team_name,
            status: NegotiationStatus::Open,
            current_round: 0,
            max_rounds: 5, // 默认最多 5 轮谈判
            is_transfer: false,
            transfer_fee: None,
            offers: Vec::new(),
            responses: Vec::new(),
            final_team_id: None,
            final_team_name: None,
            final_salary: None,
            final_years: None,
            final_starter: None,
            final_transfer_fee: None,
            competing_team_ids: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 添加竞争球队
    pub fn add_competing_team(&mut self, team_id: u64) {
        if !self.competing_team_ids.contains(&team_id) {
            self.competing_team_ids.push(team_id);
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 添加报价
    pub fn add_offer(&mut self, offer: Offer) {
        self.add_competing_team(offer.from_team_id);
        self.offers.push(offer);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 添加回应
    pub fn add_response(&mut self, response: OfferResponse) {
        self.responses.push(response);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 进入下一轮
    pub fn advance_round(&mut self) {
        self.current_round += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 完成签约
    pub fn complete_signing(
        &mut self,
        team_id: u64,
        team_name: String,
        salary: u64,
        years: u8,
        is_starter: bool,
    ) {
        self.status = NegotiationStatus::Accepted;
        self.final_team_id = Some(team_id);
        self.final_team_name = Some(team_name);
        self.final_salary = Some(salary);
        self.final_years = Some(years);
        self.final_starter = Some(is_starter);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 完成转会（挖人，需要转会费）
    pub fn complete_transfer(
        &mut self,
        team_id: u64,
        team_name: String,
        salary: u64,
        years: u8,
        is_starter: bool,
        transfer_fee: u64,
    ) {
        self.status = NegotiationStatus::Accepted;
        self.final_team_id = Some(team_id);
        self.final_team_name = Some(team_name);
        self.final_salary = Some(salary);
        self.final_years = Some(years);
        self.final_starter = Some(is_starter);
        self.final_transfer_fee = Some(transfer_fee);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 标记为拒绝（选手拒绝所有报价）
    pub fn mark_rejected(&mut self) {
        self.status = NegotiationStatus::Rejected;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 标记为过期
    pub fn mark_expired(&mut self) {
        self.status = NegotiationStatus::Expired;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 标记为撤回
    pub fn mark_withdrawn(&mut self) {
        self.status = NegotiationStatus::Withdrawn;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 获取某球队的最新报价
    pub fn get_latest_offer_from_team(&self, team_id: u64) -> Option<&Offer> {
        self.offers
            .iter()
            .filter(|o| o.from_team_id == team_id)
            .last()
    }

    /// 获取所有待处理的报价
    pub fn get_pending_offers(&self) -> Vec<&Offer> {
        self.offers.iter().filter(|o| o.is_pending()).collect()
    }

    /// 获取当前最高报价
    pub fn get_highest_offer(&self) -> Option<&Offer> {
        self.get_pending_offers()
            .into_iter()
            .max_by_key(|o| o.total_value())
    }

    /// 检查是否已达到最大轮次
    pub fn is_max_rounds_reached(&self) -> bool {
        self.current_round >= self.max_rounds
    }

    /// 检查是否可以继续谈判
    pub fn can_continue(&self) -> bool {
        !self.status.is_final() && !self.is_max_rounds_reached()
    }

    /// 获取竞争球队数量
    pub fn competing_teams_count(&self) -> usize {
        self.competing_team_ids.len()
    }
}

// ==================== API 响应类型 ====================

/// 谈判列表展示信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationListInfo {
    pub id: u64,
    pub player_id: u64,
    pub player_name: String,
    pub player_position: String,
    pub player_ability: u8,
    pub from_team_name: Option<String>,
    pub status: NegotiationStatus,
    pub status_name: String,
    pub current_round: u8,
    pub offers_count: usize,
    pub competing_teams_count: usize,
    pub highest_offer: Option<u64>,
    pub final_team_name: Option<String>,
    pub created_at: String,
}

impl From<&Negotiation> for NegotiationListInfo {
    fn from(n: &Negotiation) -> Self {
        Self {
            id: n.id,
            player_id: n.player_id,
            player_name: n.player_name.clone(),
            player_position: n.player_position.clone(),
            player_ability: n.player_ability,
            from_team_name: n.from_team_name.clone(),
            status: n.status,
            status_name: n.status.display_name().to_string(),
            current_round: n.current_round,
            offers_count: n.offers.len(),
            competing_teams_count: n.competing_teams_count(),
            highest_offer: n.get_highest_offer().map(|o| o.salary_offer),
            final_team_name: n.final_team_name.clone(),
            created_at: n.created_at.clone(),
        }
    }
}

/// 谈判详情展示信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationDetailInfo {
    pub negotiation: Negotiation,
    /// 按球队分组的报价历史
    pub offers_by_team: Vec<TeamOfferHistory>,
    /// 时间线事件
    pub timeline: Vec<NegotiationEvent>,
}

/// 单个球队的报价历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamOfferHistory {
    pub team_id: u64,
    pub team_name: String,
    pub offers: Vec<Offer>,
    pub responses: Vec<OfferResponse>,
    pub latest_status: String,
    pub is_leading: bool, // 是否为当前最高出价
}

/// 谈判事件（用于时间线展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationEvent {
    pub event_type: NegotiationEventType,
    pub round: u8,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub description: String,
    pub details: Option<String>,
    pub timestamp: String,
}

/// 谈判事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NegotiationEventType {
    /// 谈判开始
    Started,
    /// 收到报价
    OfferReceived,
    /// 选手回应
    PlayerResponse,
    /// 球队加价
    OfferRaised,
    /// 球队退出
    TeamWithdrew,
    /// 选手接受
    Accepted,
    /// 选手拒绝
    Rejected,
    /// 谈判过期
    Expired,
}

impl NegotiationEventType {
    pub fn display_name(&self) -> &'static str {
        match self {
            NegotiationEventType::Started => "谈判开始",
            NegotiationEventType::OfferReceived => "收到报价",
            NegotiationEventType::PlayerResponse => "选手回应",
            NegotiationEventType::OfferRaised => "球队加价",
            NegotiationEventType::TeamWithdrew => "球队退出",
            NegotiationEventType::Accepted => "达成协议",
            NegotiationEventType::Rejected => "谈判破裂",
            NegotiationEventType::Expired => "谈判过期",
        }
    }
}

impl From<&Negotiation> for NegotiationDetailInfo {
    fn from(n: &Negotiation) -> Self {
        use std::collections::HashMap;

        // 按球队分组报价
        let mut offers_by_team_map: HashMap<u64, (String, Vec<Offer>, Vec<OfferResponse>)> = HashMap::new();

        for offer in &n.offers {
            let entry = offers_by_team_map
                .entry(offer.from_team_id)
                .or_insert_with(|| (offer.from_team_name.clone(), Vec::new(), Vec::new()));
            entry.1.push(offer.clone());
        }

        for response in &n.responses {
            // 找到对应的报价来获取球队信息
            if let Some(offer) = n.offers.iter().find(|o| o.id == response.offer_id) {
                if let Some(entry) = offers_by_team_map.get_mut(&offer.from_team_id) {
                    entry.2.push(response.clone());
                }
            }
        }

        // 找出最高出价的球队
        let highest_offer = n.get_highest_offer();
        let leading_team_id = highest_offer.map(|o| o.from_team_id);

        let offers_by_team: Vec<TeamOfferHistory> = offers_by_team_map
            .into_iter()
            .map(|(team_id, (team_name, offers, responses))| {
                let latest_status = offers.last()
                    .map(|o| format!("{:?}", o.status))
                    .unwrap_or_else(|| "无".to_string());

                TeamOfferHistory {
                    team_id,
                    team_name,
                    offers,
                    responses,
                    latest_status,
                    is_leading: Some(team_id) == leading_team_id,
                }
            })
            .collect();

        // 构建时间线
        let mut timeline: Vec<NegotiationEvent> = Vec::new();

        // 添加开始事件
        timeline.push(NegotiationEvent {
            event_type: NegotiationEventType::Started,
            round: 0,
            team_id: None,
            team_name: None,
            description: format!("{}的转会谈判开始", n.player_name),
            details: None,
            timestamp: n.created_at.clone(),
        });

        // 添加报价事件
        for offer in &n.offers {
            timeline.push(NegotiationEvent {
                event_type: NegotiationEventType::OfferReceived,
                round: offer.round,
                team_id: Some(offer.from_team_id),
                team_name: Some(offer.from_team_name.clone()),
                description: format!(
                    "{}发出报价: {}万/年, {}年",
                    offer.from_team_name, offer.salary_offer, offer.contract_years
                ),
                details: Some(offer.offer_reasoning.clone()),
                timestamp: offer.created_at.clone(),
            });
        }

        // 添加回应事件
        for response in &n.responses {
            if let Some(offer) = n.offers.iter().find(|o| o.id == response.offer_id) {
                let event_type = match response.response_type {
                    ResponseType::Accept => NegotiationEventType::Accepted,
                    ResponseType::Reject => NegotiationEventType::Rejected,
                    ResponseType::Counter => NegotiationEventType::PlayerResponse,
                    ResponseType::Wait => NegotiationEventType::PlayerResponse,
                };

                let description = match response.response_type {
                    ResponseType::Accept => format!("接受{}的报价", offer.from_team_name),
                    ResponseType::Reject => format!("拒绝{}的报价", offer.from_team_name),
                    ResponseType::Counter => format!(
                        "向{}还价: {}万/年",
                        offer.from_team_name,
                        response.counter_salary.unwrap_or(0)
                    ),
                    ResponseType::Wait => format!("正在考虑{}的报价", offer.from_team_name),
                };

                timeline.push(NegotiationEvent {
                    event_type,
                    round: offer.round,
                    team_id: Some(offer.from_team_id),
                    team_name: Some(offer.from_team_name.clone()),
                    description,
                    details: Some(response.reasoning.clone()),
                    timestamp: response.responded_at.clone(),
                });
            }
        }

        // 按时间排序
        timeline.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Self {
            negotiation: n.clone(),
            offers_by_team,
            timeline,
        }
    }
}

// ==================== LLM 响应解析结构 ====================

/// LLM 返回的报价决策
#[derive(Debug, Clone, Deserialize)]
pub struct LLMOfferDecision {
    /// 是否发出报价
    pub should_offer: bool,
    /// 报价薪资（万/年）
    pub salary_offer: f64,
    /// 合同年限
    pub contract_years: u8,
    /// 是否保证首发
    pub guarantee_starter: bool,
    /// 签字费（万）
    pub signing_bonus: f64,
    /// 决策理由
    pub reasoning: String,
    /// 分析步骤
    #[serde(default)]
    pub analysis_steps: Vec<LLMAnalysisStep>,
}

/// LLM 返回的回应决策
#[derive(Debug, Clone, Deserialize)]
pub struct LLMResponseDecision {
    /// 回应类型：ACCEPT / REJECT / COUNTER / WAIT
    pub response_type: String,
    /// 还价薪资（当 COUNTER 时）
    pub counter_salary: Option<f64>,
    /// 还价年限
    pub counter_years: Option<u8>,
    /// 是否要求首发
    pub counter_starter: Option<bool>,
    /// 决策理由
    pub reasoning: String,
    /// 分析步骤
    #[serde(default)]
    pub analysis_steps: Vec<LLMAnalysisStep>,
}

/// LLM 分析步骤（与 player_transfer_strategy 中的一致）
#[derive(Debug, Clone, Deserialize)]
pub struct LLMAnalysisStep {
    pub step_name: String,
    pub data_used: String,
    #[serde(default)]
    pub threshold: String,
    pub result: String,
    pub impact: String,
}

impl From<LLMAnalysisStep> for AnalysisStep {
    fn from(llm: LLMAnalysisStep) -> Self {
        AnalysisStep {
            step_name: llm.step_name,
            data_used: llm.data_used,
            threshold: llm.threshold,
            result: llm.result,
            impact: llm.impact,
        }
    }
}

// ==================== 自由市场评估结构体 ====================

/// 对单个自由球员的评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEvaluation {
    /// 选手ID
    pub player_id: u64,
    /// 选手名称
    pub player_name: String,
    /// 位置
    pub position: String,
    /// 能力值
    pub ability: u8,
    /// 年龄
    pub age: u8,
    /// 适配度评分 (0-100)
    pub fit_score: u8,
    /// 是否考虑报价
    pub consider_offer: bool,
    /// 评估理由
    pub evaluation: String,
    /// 不考虑的原因（如果 consider_offer 为 false）
    pub rejection_reason: Option<String>,
}

/// LLM 返回的自由市场评估结果
#[derive(Debug, Clone, Deserialize)]
pub struct LLMMarketEvaluation {
    /// 对所有自由球员的评估
    pub player_evaluations: Vec<LLMPlayerEvaluation>,
    /// 选择报价的选手ID（如果决定报价的话）
    pub chosen_player_id: Option<u64>,
    /// 报价详情（如果决定报价的话）
    pub offer_details: Option<LLMOfferDetails>,
    /// 整体思考过程
    pub overall_reasoning: String,
}

/// LLM 对单个选手的评估
#[derive(Debug, Clone, Deserialize)]
pub struct LLMPlayerEvaluation {
    /// 选手ID
    pub player_id: u64,
    /// 适配度评分 (0-100)
    pub fit_score: u8,
    /// 是否考虑报价
    pub consider_offer: bool,
    /// 评估理由
    pub evaluation: String,
    /// 不考虑的原因
    #[serde(default)]
    pub rejection_reason: Option<String>,
}

/// LLM 返回的报价详情
#[derive(Debug, Clone, Deserialize)]
pub struct LLMOfferDetails {
    /// 报价薪资（万/年）
    pub salary_offer: u64,
    /// 合同年限
    pub contract_years: u8,
    /// 是否保证首发
    pub guarantee_starter: bool,
    /// 签字费（万）
    #[serde(default)]
    pub signing_bonus: u64,
    /// 报价理由
    pub reasoning: String,
}
