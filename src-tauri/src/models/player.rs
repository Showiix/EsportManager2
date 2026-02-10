use crate::engines::market_value::MarketValueEngine;
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

    /// 获取衰退速率系数（天才衰退慢，平庸衰退快）
    pub fn decay_factor(&self) -> f64 {
        match self {
            PlayerTag::Genius => 0.7,
            PlayerTag::Normal => 1.0,
            PlayerTag::Ordinary => 1.2,
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
            RegionCode::LPL => 1.3, // 资本最雄厚
            RegionCode::LCK => 1.2, // 传统强区
            RegionCode::LEC => 1.0, // 欧洲标准
            RegionCode::LCS => 0.9, // 北美略低
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
    #[serde(default)]
    pub growth_accumulator: f64,
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
            16..=24 => 60 + (age - 16) * 2, // 60-76
            25..=29 => 75 + (age - 25) * 2, // 75-85
            30..=36 => 85 + (age - 30),     // 85-91
            _ => 70,
        }
    }

    /// 根据年龄获取状态加成上限
    pub fn max_form_bonus(age: u8) -> i8 {
        match age {
            16..=24 => 8, // 年轻选手高上限
            25..=29 => 3, // 巅峰期稳定
            _ => 2,       // 老将低上限
        }
    }

    /// 计算基础身价（不含荣誉加成）— 委托给 MarketValueEngine
    /// 返回值单位：元
    pub fn calculate_base_market_value(&self) -> u64 {
        let tag_str = match self.tag {
            PlayerTag::Genius => "GENIUS",
            PlayerTag::Normal => "NORMAL",
            PlayerTag::Ordinary => "ORDINARY",
        };
        let pos_str = match self.position {
            Some(Position::Top) => "TOP",
            Some(Position::Jug) => "JUG",
            Some(Position::Mid) => "MID",
            Some(Position::Adc) => "ADC",
            Some(Position::Sup) => "SUP",
            None => "MID",
        };
        MarketValueEngine::calculate_base_market_value(
            self.ability,
            self.age,
            self.potential,
            tag_str,
            pos_str,
        )
    }

    /// 计算完整身价（含荣誉和赛区加成）— 委托给 MarketValueEngine
    /// 参数：
    /// - region_code: 赛区代码 (LPL/LCK/LEC/LCS)
    /// - honor_factor: 荣誉系数 (由外部计算传入，范围 1.0 ~ 4.0)
    /// 返回值单位：元
    pub fn calculate_full_market_value(&self, region_code: &str, honor_factor: f64) -> u64 {
        let base_value = self.calculate_base_market_value();
        MarketValueEngine::calculate_full_market_value(base_value, honor_factor, region_code)
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
            90..=100 => 20, // 极高忠诚：满意度要降到20以下
            70..=89 => 35,  // 高忠诚：35以下
            50..=69 => 50,  // 普通：50以下
            30..=49 => 60,  // 低忠诚：60以下就想走
            _ => 70,        // 极低：70以下就想走
        }
    }

    /// 计算拒绝挖角概率
    /// 高忠诚选手可能拒绝其他球队的邀约
    pub fn reject_poaching_chance(&self) -> f64 {
        match self.loyalty {
            90..=100 => 0.7, // 70%概率拒绝
            70..=89 => 0.4,  // 40%概率拒绝
            50..=69 => 0.1,  // 10%概率拒绝
            _ => 0.0,        // 不会拒绝
        }
    }

    /// 计算忠诚度转会费溢价因子
    /// 忠诚选手要求更高转会费才肯走
    pub fn loyalty_price_factor(&self) -> f64 {
        match self.loyalty {
            80..=100 => 1.3, // 要求130%身价
            60..=79 => 1.15, // 要求115%身价
            _ => 1.0,        // 正常身价
        }
    }

    /// 计算自由球员时对老东家的偏好加成
    /// 高忠诚自由球员优先考虑老东家续约
    pub fn former_team_bonus(&self) -> f64 {
        match self.loyalty {
            80..=100 => 0.3, // 老东家吸引力+30%
            60..=79 => 0.15, // +15%
            _ => 0.0,        // 无加成
        }
    }

    /// 更新忠诚度（限制在0-100范围内）
    pub fn update_loyalty(&mut self, change: i32) {
        let new_value = (self.loyalty as i32 + change).clamp(0, 100) as u8;
        self.loyalty = new_value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== PlayerTag ====================

    #[test]
    fn test_player_tag_growth_per_season() {
        assert_eq!(PlayerTag::Ordinary.growth_per_season(), 1);
        assert_eq!(PlayerTag::Normal.growth_per_season(), 2);
        assert_eq!(PlayerTag::Genius.growth_per_season(), 3);
    }

    #[test]
    fn test_player_tag_market_value_factor() {
        assert!(PlayerTag::Genius.market_value_factor() > PlayerTag::Normal.market_value_factor());
        assert!(
            PlayerTag::Normal.market_value_factor() > PlayerTag::Ordinary.market_value_factor()
        );
    }

    #[test]
    fn test_player_tag_decay_factor() {
        // Genius decays slower
        assert!(PlayerTag::Genius.decay_factor() < PlayerTag::Normal.decay_factor());
        assert!(PlayerTag::Normal.decay_factor() < PlayerTag::Ordinary.decay_factor());
    }

    // ==================== Position ====================

    #[test]
    fn test_position_market_value_factor() {
        assert!(Position::Mid.market_value_factor() > Position::Top.market_value_factor());
        assert!(Position::Top.market_value_factor() > Position::Sup.market_value_factor());
    }

    // ==================== RegionCode ====================

    #[test]
    fn test_region_code_from_str() {
        assert_eq!(RegionCode::from_str("LPL"), RegionCode::LPL);
        assert_eq!(RegionCode::from_str("lck"), RegionCode::LCK);
        assert_eq!(RegionCode::from_str("lec"), RegionCode::LEC);
        assert_eq!(RegionCode::from_str("LCS"), RegionCode::LCS);
        assert_eq!(RegionCode::from_str("unknown"), RegionCode::Other);
    }

    #[test]
    fn test_region_code_market_value_factor() {
        assert!(RegionCode::LPL.market_value_factor() > RegionCode::LCK.market_value_factor());
        assert!(RegionCode::LCK.market_value_factor() > RegionCode::LEC.market_value_factor());
    }

    // ==================== LoyaltyType ====================

    #[test]
    fn test_loyalty_type_from_value() {
        assert_eq!(LoyaltyType::from_value(95), LoyaltyType::Devoted);
        assert_eq!(LoyaltyType::from_value(65), LoyaltyType::Loyal);
        assert_eq!(LoyaltyType::from_value(50), LoyaltyType::Neutral);
        assert_eq!(LoyaltyType::from_value(30), LoyaltyType::Opportunist);
        assert_eq!(LoyaltyType::from_value(10), LoyaltyType::Mercenary);
    }

    // ==================== Player methods ====================

    #[test]
    fn test_calculate_stability_by_age() {
        // Young: 16-24 → 60-76
        assert_eq!(Player::calculate_stability(16), 60);
        assert_eq!(Player::calculate_stability(24), 76);
        // Peak: 25-29 → 75-85
        assert_eq!(Player::calculate_stability(25), 75);
        assert_eq!(Player::calculate_stability(29), 83);
        // Veteran: 30-36 → 85-91
        assert_eq!(Player::calculate_stability(30), 85);
        assert_eq!(Player::calculate_stability(36), 91);
        // Edge case
        assert_eq!(Player::calculate_stability(15), 70);
    }

    #[test]
    fn test_max_form_bonus_by_age() {
        assert_eq!(Player::max_form_bonus(20), 8);
        assert_eq!(Player::max_form_bonus(27), 3);
        assert_eq!(Player::max_form_bonus(32), 2);
    }

    #[test]
    fn test_departure_threshold() {
        assert_eq!(Player::departure_threshold_static(95), 20);
        assert_eq!(Player::departure_threshold_static(75), 35);
        assert_eq!(Player::departure_threshold_static(55), 50);
        assert_eq!(Player::departure_threshold_static(35), 60);
        assert_eq!(Player::departure_threshold_static(15), 70);
    }

    #[test]
    fn test_update_loyalty_clamp() {
        let mut player = Player {
            id: 1,
            game_id: "test".into(),
            real_name: None,
            nationality: None,
            age: 22,
            ability: 75,
            potential: 85,
            stability: 70,
            tag: PlayerTag::Normal,
            status: PlayerStatus::Active,
            position: Some(Position::Mid),
            team_id: Some(1),
            salary: 5_000_000,
            market_value: 0,
            calculated_market_value: 0,
            contract_end_season: Some(3),
            join_season: 1,
            retire_season: None,
            is_starter: true,
            loyalty: 90,
            satisfaction: 50,
            growth_accumulator: 0.0,
        };

        player.update_loyalty(20);
        assert_eq!(player.loyalty, 100); // clamped at 100

        player.update_loyalty(-150);
        assert_eq!(player.loyalty, 0); // clamped at 0
    }

    #[test]
    fn test_reject_poaching_chance() {
        let make = |loyalty: u8| -> Player {
            Player {
                id: 1,
                game_id: "test".into(),
                real_name: None,
                nationality: None,
                age: 22,
                ability: 75,
                potential: 85,
                stability: 70,
                tag: PlayerTag::Normal,
                status: PlayerStatus::Active,
                position: Some(Position::Mid),
                team_id: Some(1),
                salary: 5_000_000,
                market_value: 0,
                calculated_market_value: 0,
                contract_end_season: Some(3),
                join_season: 1,
                retire_season: None,
                is_starter: true,
                loyalty,
                satisfaction: 50,
                growth_accumulator: 0.0,
            }
        };
        assert!((make(95).reject_poaching_chance() - 0.7).abs() < f64::EPSILON);
        assert!((make(75).reject_poaching_chance() - 0.4).abs() < f64::EPSILON);
        assert!((make(55).reject_poaching_chance() - 0.1).abs() < f64::EPSILON);
        assert!((make(30).reject_poaching_chance() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_loyalty_price_factor() {
        let make = |loyalty: u8| -> Player {
            Player {
                id: 1,
                game_id: "test".into(),
                real_name: None,
                nationality: None,
                age: 22,
                ability: 75,
                potential: 85,
                stability: 70,
                tag: PlayerTag::Normal,
                status: PlayerStatus::Active,
                position: Some(Position::Mid),
                team_id: Some(1),
                salary: 5_000_000,
                market_value: 0,
                calculated_market_value: 0,
                contract_end_season: Some(3),
                join_season: 1,
                retire_season: None,
                is_starter: true,
                loyalty,
                satisfaction: 50,
                growth_accumulator: 0.0,
            }
        };
        assert!((make(90).loyalty_price_factor() - 1.3).abs() < f64::EPSILON);
        assert!((make(65).loyalty_price_factor() - 1.15).abs() < f64::EPSILON);
        assert!((make(40).loyalty_price_factor() - 1.0).abs() < f64::EPSILON);
    }
}
