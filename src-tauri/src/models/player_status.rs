//! 选手满意度和赛季状态模型
//!
//! 用于追踪选手的满意度、离队意愿以及球队的赛季表现

use serde::{Deserialize, Serialize};

/// 离队原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepartureReason {
    /// 缺少上场时间
    LackOfPlaytime,
    /// 球队战绩差
    TeamPerformance,
    /// 薪资不满
    SalaryDispute,
    /// 想争冠（老将+能力强）
    SeekingChampionship,
    /// 想找机会（年轻+替补）
    SeekingOpportunity,
    /// 个人原因
    PersonalReasons,
}

impl DepartureReason {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            DepartureReason::LackOfPlaytime => "缺少上场时间",
            DepartureReason::TeamPerformance => "球队战绩差",
            DepartureReason::SalaryDispute => "薪资不满",
            DepartureReason::SeekingChampionship => "追求冠军",
            DepartureReason::SeekingOpportunity => "寻找机会",
            DepartureReason::PersonalReasons => "个人原因",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "LACK_OF_PLAYTIME" => Some(DepartureReason::LackOfPlaytime),
            "TEAM_PERFORMANCE" => Some(DepartureReason::TeamPerformance),
            "SALARY_DISPUTE" => Some(DepartureReason::SalaryDispute),
            "SEEKING_CHAMPIONSHIP" => Some(DepartureReason::SeekingChampionship),
            "SEEKING_OPPORTUNITY" => Some(DepartureReason::SeekingOpportunity),
            "PERSONAL_REASONS" => Some(DepartureReason::PersonalReasons),
            _ => None,
        }
    }
}

/// 选手赛季状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSeasonStatus {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub player_id: u64,
    /// 满意度 (0-100)
    pub satisfaction: u8,
    /// 是否想离队
    pub wants_to_leave: bool,
    /// 离队原因列表
    pub departure_reasons: Vec<DepartureReason>,
    /// 作为首发的比赛场数
    pub games_as_starter: u32,
    /// 总比赛场数
    pub total_games: u32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for PlayerSeasonStatus {
    fn default() -> Self {
        Self {
            id: 0,
            save_id: String::new(),
            season_id: 0,
            player_id: 0,
            satisfaction: 70,
            wants_to_leave: false,
            departure_reasons: Vec::new(),
            games_as_starter: 0,
            total_games: 0,
            created_at: None,
            updated_at: None,
        }
    }
}

impl PlayerSeasonStatus {
    /// 创建新的选手赛季状态
    pub fn new(save_id: String, season_id: u64, player_id: u64) -> Self {
        Self {
            save_id,
            season_id,
            player_id,
            ..Default::default()
        }
    }

    /// 更新满意度（限制在0-100范围内）
    pub fn update_satisfaction(&mut self, change: i32) {
        let new_value = (self.satisfaction as i32 + change).clamp(0, 100) as u8;
        self.satisfaction = new_value;
    }

    /// 获取满意度等级描述
    pub fn satisfaction_level(&self) -> &'static str {
        match self.satisfaction {
            80..=100 => "非常满意",
            60..=79 => "满意",
            40..=59 => "一般",
            20..=39 => "不满",
            _ => "非常不满",
        }
    }
}

/// 球队赛季表现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSeasonPerformance {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    /// 最终排名
    pub final_rank: Option<u32>,
    /// 是否进入季后赛
    pub made_playoffs: bool,
    /// 季后赛结果 (CHAMPION/RUNNER_UP/SEMI/QUARTER)
    pub playoff_result: Option<String>,
    /// 国际赛成绩
    pub international_result: Option<String>,
    /// 连续未进季后赛的赛季数
    pub consecutive_no_playoffs: u32,
    pub created_at: Option<String>,
}

impl Default for TeamSeasonPerformance {
    fn default() -> Self {
        Self {
            id: 0,
            save_id: String::new(),
            season_id: 0,
            team_id: 0,
            final_rank: None,
            made_playoffs: false,
            playoff_result: None,
            international_result: None,
            consecutive_no_playoffs: 0,
            created_at: None,
        }
    }
}

impl TeamSeasonPerformance {
    /// 创建新的球队赛季表现记录
    pub fn new(save_id: String, season_id: u64, team_id: u64) -> Self {
        Self {
            save_id,
            season_id,
            team_id,
            ..Default::default()
        }
    }

    /// 是否战绩很差（排名后3）
    pub fn is_poor_performance(&self) -> bool {
        self.final_rank.map(|r| r >= 8).unwrap_or(false)
    }

    /// 是否夺冠
    pub fn is_champion(&self) -> bool {
        self.playoff_result.as_deref() == Some("CHAMPION")
    }

    /// 是否有国际赛成绩
    pub fn has_international_achievement(&self) -> bool {
        self.international_result.is_some()
    }
}

/// 忠诚度变化记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyChange {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub player_id: u64,
    /// 变化量（正数增加，负数减少）
    pub change_amount: i32,
    /// 变化原因
    pub reason: String,
    pub created_at: Option<String>,
}

/// 忠诚度变化原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoyaltyChangeReason {
    /// 每赛季自然增长
    SeasonPassed,
    /// 青训出身加成
    DraftOrigin,
    /// 球队夺冠
    TeamChampion,
    /// 进入季后赛
    MadePlayoffs,
    /// 球队战绩差
    PoorTeamPerformance,
    /// 长期替补
    LongTermBench,
    /// 被球队挂牌出售
    ListedForSale,
    /// 续约涨薪
    SalaryRaise,
    /// 续约谈崩
    RenewalFailed,
    /// 成为首发
    BecameStarter,
}

impl LoyaltyChangeReason {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            LoyaltyChangeReason::SeasonPassed => "赛季结束",
            LoyaltyChangeReason::DraftOrigin => "青训出身",
            LoyaltyChangeReason::TeamChampion => "球队夺冠",
            LoyaltyChangeReason::MadePlayoffs => "进入季后赛",
            LoyaltyChangeReason::PoorTeamPerformance => "球队战绩差",
            LoyaltyChangeReason::LongTermBench => "长期替补",
            LoyaltyChangeReason::ListedForSale => "被挂牌出售",
            LoyaltyChangeReason::SalaryRaise => "续约涨薪",
            LoyaltyChangeReason::RenewalFailed => "续约谈崩",
            LoyaltyChangeReason::BecameStarter => "成为首发",
        }
    }

    /// 获取默认变化值
    pub fn default_change(&self) -> i32 {
        match self {
            LoyaltyChangeReason::SeasonPassed => 3,
            LoyaltyChangeReason::DraftOrigin => 15,
            LoyaltyChangeReason::TeamChampion => 8,
            LoyaltyChangeReason::MadePlayoffs => 5,
            LoyaltyChangeReason::PoorTeamPerformance => -3,
            LoyaltyChangeReason::LongTermBench => -5,
            LoyaltyChangeReason::ListedForSale => -15,
            LoyaltyChangeReason::SalaryRaise => 5,
            LoyaltyChangeReason::RenewalFailed => -10,
            LoyaltyChangeReason::BecameStarter => 3,
        }
    }
}

/// 选手状态信息（用于API返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatusInfo {
    pub player_id: u64,
    pub player_name: String,
    pub satisfaction: u8,
    /// 相比上赛季的变化
    pub satisfaction_trend: i8,
    pub loyalty: u8,
    pub loyalty_type: String,
    pub wants_to_leave: bool,
    pub departure_reasons: Vec<String>,
}

/// 离队候选人信息（用于API返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartureCandidateInfo {
    pub player: PlayerStatusInfo,
    pub team_id: u64,
    pub team_name: String,
    pub market_value: u64,
    pub primary_reason: String,
}

/// 满意度/忠诚度赛季结算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatisfactionSettlementResult {
    /// 处理的选手数量
    pub players_processed: u32,
    /// 想离队的选手数量
    pub departure_candidates: u32,
    /// 忠诚度变化记录数
    pub loyalty_changes: u32,
    /// 满意度平均变化
    pub avg_satisfaction_change: f64,
}
