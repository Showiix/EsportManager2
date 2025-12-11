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
    pub contract_end_season: Option<u32>,
    pub join_season: u32,
    pub retire_season: Option<u32>,
    pub is_starter: bool,
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

    /// 计算身价
    pub fn calculate_market_value(&self) -> u64 {
        // 基础系数
        let base_multiplier = match self.ability {
            90..=100 => 15,
            80..=89 => 10,
            70..=79 => 6,
            60..=69 => 3,
            _ => 1,
        };

        let base_value = self.ability as u64 * base_multiplier;

        // 年龄修正
        let age_factor = match self.age {
            16..=22 => 1.3,
            23..=26 => 1.0,
            27..=29 => 0.8,
            _ => 0.5,
        };

        // 潜力修正
        let potential_diff = self.potential.saturating_sub(self.ability);
        let potential_factor = if potential_diff > 10 {
            1.2
        } else if potential_diff >= 5 {
            1.1
        } else {
            1.0
        };

        // 标签修正
        let tag_factor = match self.tag {
            PlayerTag::Genius => 1.2,
            PlayerTag::Normal => 1.0,
            PlayerTag::Ordinary => 0.9,
        };

        (base_value as f64 * age_factor * potential_factor * tag_factor) as u64
    }
}
