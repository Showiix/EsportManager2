use serde::{Deserialize, Serialize};

/// 选手标签
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayerTag {
    /// 平庸 - 每赛季+1能力
    Ordinary,
    /// 一般 - 每赛季+2能力
    Normal,
    /// 天才 - 每赛季+3能力
    Genius,
}

impl PlayerTag {
    /// 获取每赛季能力增长值
    pub fn growth_per_season(&self) -> u8 {
        match self {
            PlayerTag::Ordinary => 1,
            PlayerTag::Normal => 2,
            PlayerTag::Genius => 3,
        }
    }

    /// 获取身价系数
    pub fn market_value_factor(&self) -> f64 {
        match self {
            PlayerTag::Genius => 1.2,
            PlayerTag::Normal => 1.0,
            PlayerTag::Ordinary => 0.9,
        }
    }
}

/// 选手状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayerStatus {
    /// 在役
    Active,
    /// 退役
    Retired,
}

/// 选手位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Position {
    Top,
    Jug,
    Mid,
    Adc,
    Sup,
}

impl Position {
    /// 获取位置身价系数
    pub fn market_value_factor(&self) -> f64 {
        match self {
            Position::Mid => 1.2,  // 核心C位，最高溢价
            Position::Adc => 1.15, // 团战输出核心
            Position::Jug => 1.1,  // 节奏发动机
            Position::Top => 1.0,  // 标准
            Position::Sup => 0.9,  // 辅助位置较低
        }
    }
}

/// 赛区代码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionCode {
    LPL,
    LCK,
    LEC,
    LCS,
    Other,
}

impl RegionCode {
    /// 从字符串解析赛区代码
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "LPL" => RegionCode::LPL,
            "LCK" => RegionCode::LCK,
            "LEC" => RegionCode::LEC,
            "LCS" => RegionCode::LCS,
            _ => RegionCode::Other,
        }
    }

    /// 获取赛区名称
    pub fn name(&self) -> &'static str {
        match self {
            RegionCode::LPL => "中国赛区",
            RegionCode::LCK => "韩国赛区",
            RegionCode::LEC => "欧洲赛区",
            RegionCode::LCS => "北美赛区",
            RegionCode::Other => "其他赛区",
        }
    }

    /// 获取赛区身价系数
    pub fn market_value_factor(&self) -> f64 {
        match self {
            RegionCode::LPL => 1.3,  // 资本最雄厚
            RegionCode::LCK => 1.2,  // 传统强区
            RegionCode::LEC => 1.0,  // 欧洲标准
            RegionCode::LCS => 0.9,  // 北美略低
            RegionCode::Other => 0.8,
        }
    }
}

/// 忠诚度类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoyaltyType {
    /// 忠心耿耿 (80-100) - 几乎不会主动离队
    Devoted,
    /// 忠诚 (60-79) - 稳定
    Loyal,
    /// 中立 (40-59) - 看情况
    Neutral,
    /// 机会主义 (20-39) - 追求更好待遇
    Opportunist,
    /// 雇佣兵 (0-19) - 哪里钱多去哪里
    Mercenary,
}

impl LoyaltyType {
    /// 从忠诚度数值获取类型
    pub fn from_value(loyalty: u8) -> Self {
        match loyalty {
            80..=100 => LoyaltyType::Devoted,
            60..=79 => LoyaltyType::Loyal,
            40..=59 => LoyaltyType::Neutral,
            20..=39 => LoyaltyType::Opportunist,
            _ => LoyaltyType::Mercenary,
        }
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            LoyaltyType::Devoted => "忠心耿耿",
            LoyaltyType::Loyal => "忠诚",
            LoyaltyType::Neutral => "中立",
            LoyaltyType::Opportunist => "机会主义",
            LoyaltyType::Mercenary => "雇佣兵",
        }
    }
}

/// 选手数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u64,
    pub game_id: String,
    pub real_name: Option<String>,
    pub nationality: Option<String>,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub stability: u8,
    pub tag: PlayerTag,
    pub status: PlayerStatus,
    pub position: Option<Position>,
    pub team_id: Option<u64>,
    pub salary: u64,
    pub market_value: u64,
    /// 计算后的身价（含荣誉和赛区系数）
    #[serde(default)]
    pub calculated_market_value: u64,
    pub contract_end_season: Option<u32>,
    pub join_season: u32,
    pub retire_season: Option<u32>,
    pub is_starter: bool,
    /// 忠诚度 (0-100)，默认50
    #[serde(default = "default_loyalty")]
    pub loyalty: u8,
    /// 满意度 (0-100)，默认50
    #[serde(default = "default_satisfaction")]
    pub satisfaction: u8,
}

/// 默认忠诚度值
fn default_loyalty() -> u8 {
    50
}

/// 默认满意度值
fn default_satisfaction() -> u8 {
    50
}

impl Player {
    /// 根据年龄计算稳定性
    pub fn calculate_stability(age: u8) -> u8 {
        match age {
            16..=24 => 60 + (age - 16) * 2,  // 60-76
            25..=29 => 75 + (age - 25) * 2,  // 75-85
            30..=36 => 85 + (age - 30),      // 85-91
            _ => 70,
        }
    }

    /// 根据年龄获取状态加成上限
    pub fn max_form_bonus(age: u8) -> i8 {
        match age {
            16..=24 => 8,   // 年轻选手高上限
            25..=29 => 3,   // 巅峰期稳定
            _ => 2,         // 老将低上限
        }
    }

    /// 获取基础身价系数（按能力值分档）
    /// 返回值单位：万元
    fn base_value_multiplier(ability: u8) -> u64 {
        match ability {
            95..=100 => 50,  // 顶级选手：4750-5000万
            90..=94 => 35,   // 世界级：3150-3290万
            85..=89 => 20,   // 顶尖：1700-1780万
            80..=84 => 12,   // 优秀：960-1008万
            75..=79 => 7,    // 合格首发：525-553万
            70..=74 => 4,    // 替补级：280-296万
            60..=69 => 2,    // 新人：120-138万
            _ => 1,          // 青训：<60万
        }
    }

    /// 获取年龄身价系数
    fn age_value_factor(age: u8) -> f64 {
        match age {
            17..=19 => 1.5,  // 超新星溢价
            20..=22 => 1.3,  // 年轻潜力股
            23..=25 => 1.0,  // 黄金年龄
            26..=27 => 0.85, // 巅峰末期
            28..=29 => 0.7,  // 开始下滑
            _ => 0.5,        // 老将或太年轻
        }
    }

    /// 获取潜力身价系数
    fn potential_value_factor(ability: u8, potential: u8) -> f64 {
        let diff = potential.saturating_sub(ability);
        if diff > 10 {
            1.25
        } else if diff >= 5 {
            1.1
        } else {
            1.0
        }
    }

    /// 计算基础身价（不含荣誉加成）
    /// 返回值单位：万元
    pub fn calculate_base_market_value(&self) -> u64 {
        let base_value = self.ability as u64 * Self::base_value_multiplier(self.ability);
        let age_factor = Self::age_value_factor(self.age);
        let potential_factor = Self::potential_value_factor(self.ability, self.potential);
        let tag_factor = self.tag.market_value_factor();
        let position_factor = self.position.map(|p| p.market_value_factor()).unwrap_or(1.0);

        (base_value as f64 * age_factor * potential_factor * tag_factor * position_factor) as u64
    }

    /// 计算完整身价（含荣誉和赛区加成）
    /// 参数：
    /// - region_code: 赛区代码 (LPL/LCK/LEC/LCS)
    /// - honor_factor: 荣誉系数 (由外部计算传入，范围 1.0 ~ 3.0)
    /// 返回值单位：万元
    pub fn calculate_full_market_value(&self, region_code: &str, honor_factor: f64) -> u64 {
        let base_value = self.calculate_base_market_value();
        let region_factor = RegionCode::from_str(region_code).market_value_factor();
        // 荣誉系数上限为 3.0
        let clamped_honor_factor = honor_factor.min(3.0).max(1.0);

        (base_value as f64 * region_factor * clamped_honor_factor) as u64
    }

    /// 计算身价（优先使用计算后的完整身价，否则使用基础身价）
    pub fn calculate_market_value(&self) -> u64 {
        // 优先返回已计算的完整身价（含荣誉和赛区系数）
        if self.calculated_market_value > 0 {
            self.calculated_market_value
        } else {
            self.calculate_base_market_value()
        }
    }

    // ==================== 忠诚度相关方法 ====================

    /// 获取忠诚度类型
    pub fn loyalty_type(&self) -> LoyaltyType {
        LoyaltyType::from_value(self.loyalty)
    }

    /// 计算离队意愿阈值（满意度低于此值才想离队）
    /// 高忠诚度的选手需要更低的满意度才会想离队
    pub fn departure_threshold(&self) -> u8 {
        Self::departure_threshold_static(self.loyalty)
    }

    /// 静态方法：根据忠诚度计算离队意愿阈值
    pub fn departure_threshold_static(loyalty: u8) -> u8 {
        match loyalty {
            90..=100 => 20,  // 极高忠诚：满意度要降到20以下
            70..=89 => 35,   // 高忠诚：35以下
            50..=69 => 50,   // 普通：50以下
            30..=49 => 60,   // 低忠诚：60以下就想走
            _ => 70,         // 极低：70以下就想走
        }
    }

    /// 计算拒绝挖角概率
    /// 高忠诚选手可能拒绝其他球队的邀约
    pub fn reject_poaching_chance(&self) -> f64 {
        match self.loyalty {
            90..=100 => 0.7,  // 70%概率拒绝
            70..=89 => 0.4,   // 40%概率拒绝
            50..=69 => 0.1,   // 10%概率拒绝
            _ => 0.0,         // 不会拒绝
        }
    }

    /// 计算忠诚度转会费溢价因子
    /// 忠诚选手要求更高转会费才肯走
    pub fn loyalty_price_factor(&self) -> f64 {
        match self.loyalty {
            80..=100 => 1.3,  // 要求130%身价
            60..=79 => 1.15,  // 要求115%身价
            _ => 1.0,         // 正常身价
        }
    }

    /// 计算自由球员时对老东家的偏好加成
    /// 高忠诚自由球员优先考虑老东家续约
    pub fn former_team_bonus(&self) -> f64 {
        match self.loyalty {
            80..=100 => 0.3,  // 老东家吸引力+30%
            60..=79 => 0.15,  // +15%
            _ => 0.0,         // 无加成
        }
    }

    /// 更新忠诚度（限制在0-100范围内）
    pub fn update_loyalty(&mut self, change: i32) {
        let new_value = (self.loyalty as i32 + change).clamp(0, 100) as u8;
        self.loyalty = new_value;
    }
}
