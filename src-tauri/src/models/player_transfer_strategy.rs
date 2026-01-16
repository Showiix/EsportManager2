//! AI选手转会策略模型
//!
//! 用于存储和展示AI（LLM或Mock）生成的选手转会决策，包括：
//! - 选手是否想离队及原因
//! - 偏好的目标球队列表
//! - 期望的合同条件
//! - 转会申请及邀约处理

use serde::{Deserialize, Serialize};
use super::player_status::DepartureReason;

// ==================== 选手偏好球队 ====================

/// 选手偏好球队的原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TeamPreferenceReason {
    /// 争冠球队（实力强）
    ContendingTeam,
    /// 首发机会（同位置竞争小）
    StarterOpportunity,
    /// 高薪待遇
    HighSalary,
    /// 老东家情怀
    FormerTeam,
    /// 赛区偏好
    RegionPreference,
    /// 知名俱乐部
    FamousOrg,
    /// 年轻球队（培养环境好）
    YouthFriendly,
}

impl TeamPreferenceReason {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TeamPreferenceReason::ContendingTeam => "争冠球队",
            TeamPreferenceReason::StarterOpportunity => "首发机会",
            TeamPreferenceReason::HighSalary => "高薪待遇",
            TeamPreferenceReason::FormerTeam => "老东家",
            TeamPreferenceReason::RegionPreference => "赛区偏好",
            TeamPreferenceReason::FamousOrg => "知名俱乐部",
            TeamPreferenceReason::YouthFriendly => "青训环境",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().replace("_", "").as_str() {
            "CONTENDINGTEAM" | "CONTENDING_TEAM" => Some(Self::ContendingTeam),
            "STARTEROPPORTUNITY" | "STARTER_OPPORTUNITY" => Some(Self::StarterOpportunity),
            "HIGHSALARY" | "HIGH_SALARY" => Some(Self::HighSalary),
            "FORMERTEAM" | "FORMER_TEAM" => Some(Self::FormerTeam),
            "REGIONPREFERENCE" | "REGION_PREFERENCE" => Some(Self::RegionPreference),
            "FAMOUSORG" | "FAMOUS_ORG" => Some(Self::FamousOrg),
            "YOUTHFRIENDLY" | "YOUTH_FRIENDLY" => Some(Self::YouthFriendly),
            _ => None,
        }
    }
}

/// 选手偏好的目标球队
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferredTeam {
    /// 球队ID
    pub team_id: u64,
    /// 球队名称
    pub team_name: String,
    /// 优先级 (1最高，数字越小优先级越高)
    pub priority: u8,
    /// 偏好原因类型
    pub reason: TeamPreferenceReason,
    /// LLM生成的详细原因说明
    pub reason_detail: String,
    /// 愿意接受的降薪比例 (0.0-0.3，如0.1表示愿意降薪10%)
    pub willing_salary_reduction: f64,
    /// 对该球队的吸引力评分 (1-100)
    pub attractiveness_score: u8,
}

impl PreferredTeam {
    /// 创建新的偏好球队
    pub fn new(team_id: u64, team_name: String, priority: u8, reason: TeamPreferenceReason) -> Self {
        Self {
            team_id,
            team_name,
            priority,
            reason,
            reason_detail: String::new(),
            willing_salary_reduction: 0.0,
            attractiveness_score: 50,
        }
    }
}

// ==================== 分析依据数据结构 ====================

/// AI分析时参考的选手数据快照
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisDataSnapshot {
    // 选手基础数据
    pub player_name: String,
    pub position: String,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,

    // 状态数据
    pub satisfaction: u8,
    pub loyalty: u8,
    pub is_starter: bool,

    // 合同数据
    pub current_salary: u64,  // 万/年
    pub contract_end_season: Option<u32>,

    // 球队数据
    pub team_name: String,
    pub team_avg_ability: f64,

    // 计算出的阈值和类型
    pub loyalty_type: String,       // 忠心耿耿/忠诚/中立/机会主义/雇佣兵
    pub departure_threshold: u8,    // 离队阈值
}

/// AI分析步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStep {
    /// 步骤名称，如"满意度分析"
    pub step_name: String,
    /// 使用的数据，如"满意度: 35"
    pub data_used: String,
    /// 阈值说明，如"离队阈值: 50"
    pub threshold: String,
    /// 结论，如"低于阈值，判断想离队"
    pub result: String,
    /// 影响，如"基础离队概率 +70%"
    pub impact: String,
}

// ==================== 选手转会策略 ====================

/// AI生成的选手转会策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTransferStrategy {
    pub id: u64,
    pub player_id: u64,
    pub save_id: String,
    pub season_id: u64,

    // === LLM决策结果 ===
    /// 是否想离队
    pub wants_to_leave: bool,
    /// 决策置信度 (1-100)
    pub decision_confidence: u8,
    /// 离队原因列表
    pub departure_reasons: Vec<DepartureReason>,
    /// LLM生成的离队理由（展示用）
    pub leave_reasoning: String,

    // === 目标球队偏好 ===
    /// 偏好球队列表 (按优先级排序，通常3-5支)
    pub preferred_teams: Vec<PreferredTeam>,
    /// LLM生成的选队整体理由（展示用）
    pub team_preference_reasoning: String,

    // === 期望条件 ===
    /// 期望薪资 (万/年)
    pub expected_salary: u64,
    /// 最低接受薪资 (万/年)
    pub expected_min_salary: u64,
    /// 期望合同年限
    pub expected_years: u8,
    /// 是否要求首发位置
    pub requires_starter: bool,

    // === 分析依据（新增） ===
    /// AI分析时参考的数据快照
    #[serde(default)]
    pub analysis_data: Option<AnalysisDataSnapshot>,
    /// AI分析步骤详情
    #[serde(default)]
    pub analysis_steps: Vec<AnalysisStep>,

    // === 元数据 ===
    /// 是否为Mock生成（非真实LLM）
    pub is_mock: bool,
    /// 生成时间戳
    pub generated_at: String,
}

impl PlayerTransferStrategy {
    /// 创建空策略（选手不想离队）
    pub fn not_leaving(player_id: u64, save_id: String, season_id: u64) -> Self {
        Self {
            id: 0,
            player_id,
            save_id,
            season_id,
            wants_to_leave: false,
            decision_confidence: 80,
            departure_reasons: Vec::new(),
            leave_reasoning: "选手对当前状况满意，暂无离队意愿".to_string(),
            preferred_teams: Vec::new(),
            team_preference_reasoning: String::new(),
            expected_salary: 0,
            expected_min_salary: 0,
            expected_years: 0,
            requires_starter: false,
            analysis_data: None,
            analysis_steps: Vec::new(),
            is_mock: true,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 检查某球队是否在偏好列表中
    pub fn prefers_team(&self, team_id: u64) -> bool {
        self.preferred_teams.iter().any(|t| t.team_id == team_id)
    }

    /// 获取对某球队的偏好优先级 (None表示不在列表中)
    pub fn get_team_priority(&self, team_id: u64) -> Option<u8> {
        self.preferred_teams
            .iter()
            .find(|t| t.team_id == team_id)
            .map(|t| t.priority)
    }

    /// 获取对某球队愿意接受的最低薪资
    pub fn get_min_salary_for_team(&self, team_id: u64) -> u64 {
        if let Some(team) = self.preferred_teams.iter().find(|t| t.team_id == team_id) {
            let reduction = team.willing_salary_reduction.min(0.3);
            ((self.expected_min_salary as f64) * (1.0 - reduction)) as u64
        } else {
            self.expected_min_salary
        }
    }
}

// ==================== 转会申请 ====================

/// 转会申请状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationStatus {
    /// 等待中（已提交，等待球队响应）
    Pending,
    /// 有邀约（收到至少一个邀约）
    HasOffers,
    /// 已接受（选手接受了某个邀约）
    Accepted,
    /// 全部拒绝（选手拒绝了所有邀约）
    AllRejected,
    /// 已撤回（选手撤回申请）
    Withdrawn,
    /// 已过期（转会窗口结束）
    Expired,
}

impl ApplicationStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            ApplicationStatus::Pending => "等待中",
            ApplicationStatus::HasOffers => "有邀约",
            ApplicationStatus::Accepted => "已接受",
            ApplicationStatus::AllRejected => "全部拒绝",
            ApplicationStatus::Withdrawn => "已撤回",
            ApplicationStatus::Expired => "已过期",
        }
    }
}

/// 球队发出的邀约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamOffer {
    pub id: u64,
    /// 发出邀约的球队ID
    pub team_id: u64,
    /// 球队名称
    pub team_name: String,
    /// 转会费 (万元)
    pub transfer_fee: u64,
    /// 提供的年薪 (万/年)
    pub salary_offer: u64,
    /// 合同年限
    pub contract_years: u8,
    /// 是否保证首发位置
    pub starter_guarantee: bool,
    /// 邀约时间
    pub offered_at: String,
}

/// 被拒绝的邀约（包含拒绝原因）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedOffer {
    pub offer: TeamOffer,
    /// LLM生成的拒绝理由
    pub rejection_reason: String,
    /// 拒绝时间
    pub rejected_at: String,
}

/// 转会申请
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferApplication {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    /// 申请转会的选手ID
    pub player_id: u64,
    /// 选手当前所在球队ID
    pub from_team_id: u64,

    /// 申请状态
    pub status: ApplicationStatus,
    /// 关联的选手策略ID
    pub player_strategy_id: u64,

    /// 收到的所有邀约
    pub received_offers: Vec<TeamOffer>,
    /// 接受的邀约 (如果有)
    pub accepted_offer: Option<TeamOffer>,
    /// 拒绝的邀约列表
    pub rejected_offers: Vec<RejectedOffer>,

    pub created_at: String,
    pub updated_at: String,
}

impl TransferApplication {
    /// 创建新的转会申请
    pub fn new(
        save_id: String,
        season_id: u64,
        player_id: u64,
        from_team_id: u64,
        player_strategy_id: u64,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            save_id,
            season_id,
            player_id,
            from_team_id,
            status: ApplicationStatus::Pending,
            player_strategy_id,
            received_offers: Vec::new(),
            accepted_offer: None,
            rejected_offers: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 添加新邀约
    pub fn add_offer(&mut self, offer: TeamOffer) {
        self.received_offers.push(offer);
        if self.status == ApplicationStatus::Pending {
            self.status = ApplicationStatus::HasOffers;
        }
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 接受邀约
    pub fn accept_offer(&mut self, offer_id: u64) -> Result<(), String> {
        let offer = self
            .received_offers
            .iter()
            .find(|o| o.id == offer_id)
            .cloned()
            .ok_or("邀约不存在")?;

        self.accepted_offer = Some(offer);
        self.status = ApplicationStatus::Accepted;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// 拒绝邀约
    pub fn reject_offer(&mut self, offer_id: u64, reason: String) -> Result<(), String> {
        let offer = self
            .received_offers
            .iter()
            .find(|o| o.id == offer_id)
            .cloned()
            .ok_or("邀约不存在")?;

        self.rejected_offers.push(RejectedOffer {
            offer,
            rejection_reason: reason,
            rejected_at: chrono::Utc::now().to_rfc3339(),
        });

        // 检查是否所有邀约都被拒绝
        if self.rejected_offers.len() == self.received_offers.len() {
            self.status = ApplicationStatus::AllRejected;
        }

        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// 获取未处理的邀约数量
    pub fn pending_offers_count(&self) -> usize {
        self.received_offers.len() - self.rejected_offers.len()
            - if self.accepted_offer.is_some() { 1 } else { 0 }
    }
}

// ==================== API响应类型 ====================

/// 选手转会策略展示信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTransferStrategyInfo {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub age: u8,
    pub team_id: u64,
    pub team_name: String,

    pub wants_to_leave: bool,
    pub decision_confidence: u8,
    pub departure_reasons: Vec<String>,
    pub leave_reasoning: String,

    pub preferred_teams_count: usize,
    pub preferred_teams: Vec<PreferredTeamInfo>,
    pub team_preference_reasoning: String,

    pub expected_salary: u64,
    pub expected_min_salary: u64,
    pub expected_years: u8,
    pub requires_starter: bool,

    pub is_mock: bool,
    pub generated_at: String,
}

/// 偏好球队展示信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferredTeamInfo {
    pub team_id: u64,
    pub team_name: String,
    pub priority: u8,
    pub reason: String,
    pub reason_detail: String,
    pub attractiveness_score: u8,
}

/// 转会申请展示信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferApplicationInfo {
    pub id: u64,
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub from_team_id: u64,
    pub from_team_name: String,
    pub market_value: u64,

    pub status: ApplicationStatus,
    pub status_name: String,
    pub offers_count: usize,
    pub pending_offers_count: usize,

    pub created_at: String,
}

// ==================== LLM响应解析结构 ====================

/// LLM返回的分析步骤
#[derive(Debug, Deserialize)]
pub struct LLMAnalysisStep {
    pub step_name: String,
    pub data_used: String,
    #[serde(default)]
    pub threshold: String,
    pub result: String,
    pub impact: String,
}

/// LLM返回的选手策略JSON结构
#[derive(Debug, Deserialize)]
pub struct LLMPlayerStrategyResponse {
    pub wants_to_leave: bool,
    pub decision_confidence: u8,
    pub departure_reasons: Vec<String>,
    pub leave_reasoning: String,
    /// LLM 的分析思考过程
    #[serde(default)]
    pub analysis_steps: Vec<LLMAnalysisStep>,
    pub preferred_teams: Vec<LLMPreferredTeam>,
    pub expected_salary: f64,
    pub expected_min_salary: f64,
    pub expected_years: u8,
    pub requires_starter: bool,
    pub team_preference_reasoning: String,
}

#[derive(Debug, Deserialize)]
pub struct LLMPreferredTeam {
    pub team_id: u64,
    /// 球队名称（用于后备匹配，当 team_id 无效时）
    #[serde(default)]
    pub team_name: Option<String>,
    pub priority: u8,
    pub reason: String,
    #[serde(alias = "detail")]
    pub reason_detail: String,
    #[serde(default)]
    pub willing_salary_reduction: f64,
}

/// 邀约评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferEvaluation {
    /// 是否接受
    pub accept: bool,
    /// 评估置信度
    pub confidence: u8,
    /// 评估理由（展示用）
    pub reasoning: String,
    /// 各项条件的评分
    pub salary_score: u8,
    pub team_score: u8,
    pub role_score: u8,
    pub overall_score: u8,
}
