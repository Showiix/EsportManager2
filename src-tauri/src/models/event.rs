use serde::{Deserialize, Serialize};

/// 游戏事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    /// 选手能力成长
    PlayerGrowth,
    /// 选手能力衰退
    PlayerDecline,
    /// 选手退役
    PlayerRetirement,
    /// 新秀生成
    RookieGeneration,
    /// 合同到期
    ContractExpire,
    /// 选手年龄增长
    PlayerAging,
    /// 赛季结算
    SeasonSettlement,
}

impl EventType {
    pub fn name(&self) -> &'static str {
        match self {
            EventType::PlayerGrowth => "能力成长",
            EventType::PlayerDecline => "能力衰退",
            EventType::PlayerRetirement => "选手退役",
            EventType::RookieGeneration => "新秀加入",
            EventType::ContractExpire => "合同到期",
            EventType::PlayerAging => "年龄增长",
            EventType::SeasonSettlement => "赛季结算",
        }
    }
}

/// 游戏事件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub event_type: EventType,
    /// 关联的选手ID (如适用)
    pub player_id: Option<u64>,
    /// 关联的队伍ID (如适用)
    pub team_id: Option<u64>,
    /// 事件描述
    pub description: String,
    /// 事件详情 (JSON格式存储额外数据)
    pub details: Option<String>,
    /// 事件发生的游戏阶段
    pub phase: Option<String>,
}

/// 选手成长事件详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGrowthDetail {
    pub player_id: u64,
    pub player_name: String,
    pub old_ability: u8,
    pub new_ability: u8,
    pub growth_amount: u8,
    pub tag: String,
    pub reason: String,
}

/// 选手衰退事件详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDeclineDetail {
    pub player_id: u64,
    pub player_name: String,
    pub old_ability: u8,
    pub new_ability: u8,
    pub decline_amount: u8,
    pub age: u8,
    pub reason: String,
}

/// 选手退役事件详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRetirementDetail {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub final_ability: u8,
    pub age: u8,
    pub career_seasons: u32,
    pub reason: RetirementReason,
}

/// 退役原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetirementReason {
    /// 年龄过大 (>=36岁)
    Age,
    /// 能力过低 (<50)
    LowAbility,
    /// 年龄大且能力低 (>=30岁且能力<60)
    AgeAndAbility,
    /// 主动退役
    Voluntary,
}

impl RetirementReason {
    pub fn description(&self) -> &'static str {
        match self {
            RetirementReason::Age => "年龄过大，选择退役",
            RetirementReason::LowAbility => "竞技状态下滑严重，选择退役",
            RetirementReason::AgeAndAbility => "年龄渐长且状态下滑，选择退役",
            RetirementReason::Voluntary => "主动宣布退役",
        }
    }
}

/// 新秀生成事件详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RookieGenerationDetail {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub ability: u8,
    pub potential: u8,
    pub position: String,
    pub tag: String,
}

/// 合同到期事件详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExpireDetail {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub ability: u8,
    pub age: u8,
    /// 是否续约
    pub renewed: bool,
    /// 新合同年限 (如果续约)
    pub new_contract_years: Option<u32>,
    /// 新薪资 (如果续约)
    pub new_salary: Option<u64>,
}

/// 赛季结算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSettlementResult {
    pub season_id: u64,
    pub season_name: String,
    /// 能力成长的选手
    pub growth_events: Vec<PlayerGrowthDetail>,
    /// 能力衰退的选手
    pub decline_events: Vec<PlayerDeclineDetail>,
    /// 退役的选手
    pub retirement_events: Vec<PlayerRetirementDetail>,
    /// 合同到期的选手
    pub contract_expire_events: Vec<ContractExpireDetail>,
    /// 生成的新秀
    pub rookie_events: Vec<RookieGenerationDetail>,
}

/// 选手年龄更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAgingResult {
    pub player_id: u64,
    pub player_name: String,
    pub old_age: u8,
    pub new_age: u8,
    pub old_stability: u8,
    pub new_stability: u8,
}
