//! AI 驱动转会系统数据模型
//!
//! 实现 GM 人格配置和 AI 策略生成结构，支持：
//! - 多种 GM 人格类型（争冠型、青训型、稳健型等）
//! - AI 生成的转会策略（目标选手、出售候选、预算分配）
//! - 策略指导转会引擎决策

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== GM 人格配置 ====================

/// 出售策略激进度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SellAggressiveness {
    /// 保守：只出售低价值/老将
    Conservative,
    /// 正常：平衡出售
    Normal,
    /// 激进：积极清洗阵容
    Aggressive,
}

impl Default for SellAggressiveness {
    fn default() -> Self {
        SellAggressiveness::Normal
    }
}

impl SellAggressiveness {
    pub fn name(&self) -> &'static str {
        match self {
            SellAggressiveness::Conservative => "保守",
            SellAggressiveness::Normal => "正常",
            SellAggressiveness::Aggressive => "激进",
        }
    }
}

/// GM 人格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GMPersonality {
    /// 争冠型：追求顶级选手，愿意高价签人
    Championship,
    /// 青训型：培养新人，低预算运营
    YouthDevelopment,
    /// 稳健型：平衡发展，不冒险
    Balanced,
    /// 投机型：买低卖高，赚取差价
    Speculator,
    /// 重建型：清洗老将，重新开始
    Rebuilding,
    /// 自定义：完全由 custom_prompt 控制
    Custom,
}

impl Default for GMPersonality {
    fn default() -> Self {
        GMPersonality::Balanced
    }
}

impl GMPersonality {
    /// 获取人格名称（中文）
    pub fn name(&self) -> &'static str {
        match self {
            GMPersonality::Championship => "争冠型",
            GMPersonality::YouthDevelopment => "青训型",
            GMPersonality::Balanced => "稳健型",
            GMPersonality::Speculator => "投机型",
            GMPersonality::Rebuilding => "重建型",
            GMPersonality::Custom => "自定义",
        }
    }

    /// 获取人格描述
    pub fn description(&self) -> &'static str {
        match self {
            GMPersonality::Championship => "追求顶级选手，愿意高价签人，以争夺世界冠军为目标",
            GMPersonality::YouthDevelopment => "专注培养年轻选手，低预算运营，挖掘未来之星",
            GMPersonality::Balanced => "平衡发展，控制成本，稳定阵容，追求性价比",
            GMPersonality::Speculator => "擅长买低卖高，通过球员交易赚取差价",
            GMPersonality::Rebuilding => "清洗高薪老将，为年轻选手让路，重新开始",
            GMPersonality::Custom => "完全自定义的转会风格，由提示词控制",
        }
    }

    /// 获取默认预算比例（占总余额的百分比）
    pub fn default_budget_ratio(&self) -> f64 {
        match self {
            GMPersonality::Championship => 0.8,      // 争冠型愿意花更多钱
            GMPersonality::YouthDevelopment => 0.4,  // 青训型控制支出
            GMPersonality::Balanced => 0.6,          // 稳健型中等
            GMPersonality::Speculator => 0.5,        // 投机型保留资金
            GMPersonality::Rebuilding => 0.3,        // 重建型减少支出
            GMPersonality::Custom => 0.6,            // 自定义默认中等
        }
    }

    /// 获取目标能力值阈值（优先签约的选手能力下限）
    pub fn target_ability_threshold(&self) -> u8 {
        match self {
            GMPersonality::Championship => 85,      // 争冠型只要顶级
            GMPersonality::YouthDevelopment => 65,  // 青训型看潜力不看能力
            GMPersonality::Balanced => 75,          // 稳健型要中上水平
            GMPersonality::Speculator => 70,        // 投机型寻找被低估的
            GMPersonality::Rebuilding => 60,        // 重建型接受较低能力
            GMPersonality::Custom => 70,            // 自定义默认
        }
    }

    /// 获取目标年龄上限
    pub fn target_max_age(&self) -> u8 {
        match self {
            GMPersonality::Championship => 30,      // 争冠型接受老将
            GMPersonality::YouthDevelopment => 22,  // 青训型只要年轻人
            GMPersonality::Balanced => 28,          // 稳健型避免太老
            GMPersonality::Speculator => 25,        // 投机型要有升值空间
            GMPersonality::Rebuilding => 24,        // 重建型培养新人
            GMPersonality::Custom => 28,            // 自定义默认
        }
    }

    /// 获取溢价容忍度（愿意支付身价的倍数）
    pub fn price_premium_tolerance(&self) -> f64 {
        match self {
            GMPersonality::Championship => 1.3,     // 争冠型愿意溢价30%
            GMPersonality::YouthDevelopment => 0.9, // 青训型要低于身价
            GMPersonality::Balanced => 1.0,         // 稳健型按身价
            GMPersonality::Speculator => 0.85,      // 投机型必须折价
            GMPersonality::Rebuilding => 0.8,       // 重建型接受折价
            GMPersonality::Custom => 1.0,           // 自定义默认
        }
    }

    /// 获取选秀时的 ability 权重（用于计算选秀评分）
    pub fn default_ability_weight(&self) -> f64 {
        match self {
            GMPersonality::Championship => 0.7,     // 争冠型重视即战力
            GMPersonality::YouthDevelopment => 0.2, // 青训型重视潜力
            GMPersonality::Balanced => 0.4,         // 稳健型平衡（BPA）
            GMPersonality::Speculator => 0.3,       // 投机型选潜力股
            GMPersonality::Rebuilding => 0.1,       // 重建型只看潜力
            GMPersonality::Custom => 0.4,           // 自定义默认
        }
    }
}

/// 球队 GM 人格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamGMProfile {
    pub id: u64,
    pub team_id: u64,
    pub save_id: String,

    /// GM 人格类型
    pub personality: GMPersonality,

    /// 自定义提示词（可选，覆盖默认行为）
    pub custom_prompt: Option<String>,

    /// 风险偏好 (0-100)，影响决策的激进程度
    pub risk_tolerance: u8,

    // ========== 新增配置项 ==========

    /// 预算分配比例 (0.0-1.0)，控制转会窗口可用预算占总余额的百分比
    pub budget_ratio: f64,

    /// 出售策略激进度
    pub sell_aggressiveness: SellAggressiveness,

    /// 偏好的球员最小年龄
    pub preferred_age_min: u8,

    /// 偏好的球员最大年龄
    pub preferred_age_max: u8,

    /// 目标球员的最低能力值要求
    pub min_ability_threshold: u8,

    /// 溢价容忍度 (1.0 = 按身价, 1.3 = 最多溢价30%)
    pub price_premium_max: f64,

    /// 位置优先级 (TOP/JUG/MID/ADC/SUP -> 0-100 分数)
    pub position_priorities: HashMap<String, u8>,

    // ========== 选秀专属配置 ==========

    /// 选秀权售卖倾向调整系数 (0.0-1.0)
    pub draft_pick_sell_threshold: f64,

    /// 选秀权竞拍激进度 (0.5-2.0)
    pub draft_pick_bid_aggressiveness: f64,

    /// 传统选秀 ability 权重 (0.0-1.0)，potential 权重 = 1.0 - ability_weight
    pub draft_preference_ability_weight: f64,

    /// 年轻球员偏好加成 (-0.2 到 0.2)
    pub draft_young_bias: f64,

    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl TeamGMProfile {
    /// 创建新的 GM 配置
    pub fn new(team_id: u64, save_id: String) -> Self {
        let personality = GMPersonality::default();
        let mut position_priorities = HashMap::new();
        position_priorities.insert("TOP".to_string(), 50);
        position_priorities.insert("JUG".to_string(), 50);
        position_priorities.insert("MID".to_string(), 50);
        position_priorities.insert("ADC".to_string(), 50);
        position_priorities.insert("SUP".to_string(), 50);

        Self {
            id: 0,
            team_id,
            save_id,
            personality,
            custom_prompt: None,
            risk_tolerance: 50,
            budget_ratio: personality.default_budget_ratio(),
            sell_aggressiveness: SellAggressiveness::Normal,
            preferred_age_min: 18,
            preferred_age_max: 30,
            min_ability_threshold: personality.target_ability_threshold(),
            price_premium_max: personality.price_premium_tolerance(),
            position_priorities,
            // 选秀专属配置默认值
            draft_pick_sell_threshold: 0.5,
            draft_pick_bid_aggressiveness: 1.0,
            draft_preference_ability_weight: personality.default_ability_weight(),
            draft_young_bias: 0.0,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: None,
        }
    }

    /// 创建指定人格的 GM 配置
    pub fn with_personality(team_id: u64, save_id: String, personality: GMPersonality) -> Self {
        let mut profile = Self::new(team_id, save_id);
        profile.personality = personality;
        profile.budget_ratio = personality.default_budget_ratio();
        profile.min_ability_threshold = personality.target_ability_threshold();
        profile.price_premium_max = personality.price_premium_tolerance();
        profile.preferred_age_max = personality.target_max_age();
        profile
    }

    /// 获取有效的预算比例（可被用户自定义覆盖）
    pub fn effective_budget_ratio(&self) -> f64 {
        self.budget_ratio.clamp(0.1, 1.0)
    }
}

// ==================== AI 转会策略 ====================

/// 转会目标选手
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTarget {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub potential: u8,
    pub age: u8,
    pub current_team_id: Option<u64>,
    pub current_team_name: Option<String>,
    pub market_value: u64,
    /// 最高出价（万元）
    pub max_offer: u64,
    /// 优先级 1-10，10为最高
    pub priority: u8,
    /// 为什么想签这个人
    pub reasoning: String,
}

/// 出售候选选手
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellCandidate {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub ability: u8,
    pub age: u8,
    pub salary: u64,
    pub market_value: u64,
    /// 最低接受价（万元）
    pub min_price: u64,
    /// 出售紧迫度 1-10，10为最急
    pub urgency: u8,
    /// 出售原因
    pub reasoning: String,
}

/// AI分析步骤（战队策略）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnalysisStep {
    /// 步骤名称，如"阵容评估"
    pub step_name: String,
    /// 使用的数据，如"平均能力: 85.2, 平均年龄: 24.1"
    pub data_used: String,
    /// 阈值说明，如"争冠型目标能力阈值: 85"
    pub threshold: String,
    /// 结论，如"当前阵容竞争力不足"
    pub result: String,
    /// 影响，如"需要补强核心位置"
    pub impact: String,
}

/// 预算分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    /// 总预算（万元）
    pub total_budget: u64,
    /// 用于转会费（万元）
    pub transfer_spend: u64,
    /// 用于薪资（万元/年）
    pub salary_spend: u64,
    /// 预留资金（万元）
    pub reserve: u64,
}

impl BudgetAllocation {
    /// 创建默认预算分配
    pub fn from_balance(balance: i64, budget_ratio: f64) -> Self {
        let total = (balance.max(0) as f64 * budget_ratio) as u64;
        Self {
            total_budget: total,
            transfer_spend: (total as f64 * 0.7) as u64, // 70% 用于转会费
            salary_spend: (total as f64 * 0.2) as u64,   // 20% 用于薪资
            reserve: (total as f64 * 0.1) as u64,        // 10% 预留
        }
    }
}

/// AI 生成的转会策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITransferStrategy {
    pub id: u64,
    pub team_id: u64,
    /// 球队名称（用于前端显示）
    #[serde(default)]
    pub team_name: String,
    pub save_id: String,
    pub season_id: u64,

    /// 整体策略名称
    pub overall_strategy: String,

    /// 策略描述
    pub strategy_description: String,

    /// 目标签约选手（按优先级排序）
    pub targets: Vec<TransferTarget>,

    /// 愿意出售的选手
    pub willing_to_sell: Vec<SellCandidate>,

    /// 优先补强位置（按优先级排序）
    pub priority_positions: Vec<String>,

    /// 预算分配
    pub budget_allocation: BudgetAllocation,

    /// AI 的决策理由（可展示给用户）
    pub reasoning: String,

    /// AI分析步骤详情
    #[serde(default)]
    pub analysis_steps: Vec<TeamAnalysisStep>,

    /// 是否为 Mock 生成（非真实 AI）
    pub is_mock: bool,

    /// 生成时间戳
    pub generated_at: String,
}

impl AITransferStrategy {
    /// 创建空策略
    pub fn empty(team_id: u64, team_name: String, save_id: String, season_id: u64) -> Self {
        Self {
            id: 0,
            team_id,
            team_name,
            save_id,
            season_id,
            overall_strategy: "无策略".to_string(),
            strategy_description: String::new(),
            targets: Vec::new(),
            willing_to_sell: Vec::new(),
            priority_positions: Vec::new(),
            budget_allocation: BudgetAllocation {
                total_budget: 0,
                transfer_spend: 0,
                salary_spend: 0,
                reserve: 0,
            },
            reasoning: String::new(),
            analysis_steps: Vec::new(),
            is_mock: true,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 检查选手是否在目标列表中
    pub fn is_target(&self, player_id: u64) -> bool {
        self.targets.iter().any(|t| t.player_id == player_id)
    }

    /// 获取对某选手的最高出价
    pub fn get_max_offer(&self, player_id: u64) -> Option<u64> {
        self.targets.iter()
            .find(|t| t.player_id == player_id)
            .map(|t| t.max_offer)
    }

    /// 获取选手在目标列表中的优先级（1为最高优先级）
    pub fn get_target_priority(&self, player_id: u64) -> Option<usize> {
        self.targets.iter()
            .position(|t| t.player_id == player_id)
            .map(|i| i + 1) // 转为1-based
    }

    /// 检查选手是否在出售列表中
    pub fn is_willing_to_sell(&self, player_id: u64) -> bool {
        self.willing_to_sell.iter().any(|s| s.player_id == player_id)
    }

    /// 获取某选手的最低接受价
    pub fn get_min_price(&self, player_id: u64) -> Option<u64> {
        self.willing_to_sell.iter()
            .find(|s| s.player_id == player_id)
            .map(|s| s.min_price)
    }

    /// 检查位置是否为优先补强位置
    pub fn is_priority_position(&self, position: &str) -> bool {
        self.priority_positions.iter()
            .any(|p| p.eq_ignore_ascii_case(position))
    }

    /// 获取位置优先级（返回排名，0为最高优先级）
    pub fn get_position_priority(&self, position: &str) -> Option<usize> {
        self.priority_positions.iter()
            .position(|p| p.eq_ignore_ascii_case(position))
    }
}

// ==================== API 响应类型 ====================

/// 前端展示的 GM 配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamGMProfileInfo {
    pub team_id: u64,
    pub team_name: String,
    pub team_short_name: Option<String>,
    pub region_id: u64,
    pub region_name: String,  // 赛区名称，如 "LPL", "LCK"
    pub personality: GMPersonality,
    pub personality_name: String,
    pub personality_description: String,
    pub custom_prompt: Option<String>,
    pub risk_tolerance: u8,
    // 转会配置
    pub budget_ratio: f64,
    pub sell_aggressiveness: SellAggressiveness,
    pub preferred_age_min: u8,
    pub preferred_age_max: u8,
    pub min_ability_threshold: u8,
    pub price_premium_max: f64,
    pub position_priorities: HashMap<String, u8>,
    // 选秀配置
    pub draft_pick_sell_threshold: f64,
    pub draft_pick_bid_aggressiveness: f64,
    pub draft_preference_ability_weight: f64,
    pub draft_young_bias: f64,
}

/// 前端展示的 AI 策略信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStrategyInfo {
    pub team_id: u64,
    pub team_name: String,
    pub season_id: u64,
    pub overall_strategy: String,
    pub strategy_description: String,
    pub reasoning: String,
    pub targets_count: usize,
    pub sell_count: usize,
    pub priority_positions: Vec<String>,
    pub budget: BudgetAllocation,
    pub is_mock: bool,
    pub generated_at: String,
}

impl From<&AITransferStrategy> for AIStrategyInfo {
    fn from(strategy: &AITransferStrategy) -> Self {
        Self {
            team_id: strategy.team_id,
            team_name: String::new(), // 需要从外部填充
            season_id: strategy.season_id,
            overall_strategy: strategy.overall_strategy.clone(),
            strategy_description: strategy.strategy_description.clone(),
            reasoning: strategy.reasoning.clone(),
            targets_count: strategy.targets.len(),
            sell_count: strategy.willing_to_sell.len(),
            priority_positions: strategy.priority_positions.clone(),
            budget: strategy.budget_allocation.clone(),
            is_mock: strategy.is_mock,
            generated_at: strategy.generated_at.clone(),
        }
    }
}

// ==================== 策略生成参数 ====================

/// 策略生成上下文
#[derive(Debug, Clone)]
pub struct StrategyGenerationContext {
    /// 球队 ID
    pub team_id: u64,
    /// 球队名称
    pub team_name: String,
    /// 球队余额
    pub balance: i64,
    /// 当前阵容能力均值
    pub avg_ability: f64,
    /// 当前阵容年龄均值
    pub avg_age: f64,
    /// 阵容人数
    pub roster_count: u32,
    /// 上赛季排名
    pub last_season_rank: Option<u32>,
    /// 是否进入季后赛
    pub made_playoffs: bool,
    /// 连续未进季后赛赛季数
    pub consecutive_no_playoffs: u32,
    /// GM 人格配置
    pub gm_profile: TeamGMProfile,
}
