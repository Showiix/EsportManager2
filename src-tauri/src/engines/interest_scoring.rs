//! 兴趣度评分引擎
//!
//! 参考 NBA 2K 的设计，计算球队对每个选手的兴趣度（0-100分）

use crate::models::{Player, Team, TeamGMProfile, Position, GMPersonality};
use crate::services::llm_service::{PlayerHonorInfo, PlayerPerformanceInfo};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 兴趣度评分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestScore {
    /// 总分 0-100
    pub total: u8,
    /// 分数明细
    pub breakdown: ScoreBreakdown,
    /// 是否高兴趣（>70分）
    pub is_high_interest: bool,
}

/// 评分明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// 位置匹配 0-30
    pub position_match: f64,
    /// 能力匹配 0-30
    pub ability_match: f64,
    /// 年龄偏好 0-15
    pub age_preference: f64,
    /// 性价比 0-15
    pub value_for_money: f64,
    /// 荣誉加成 0-10
    pub honor_bonus: f64,
    /// 表现加成 0-10
    pub performance_bonus: f64,
    /// 潜力加成 0-10（青训/投机型）
    pub potential_bonus: f64,
}

impl ScoreBreakdown {
    pub fn new() -> Self {
        Self {
            position_match: 0.0,
            ability_match: 0.0,
            age_preference: 0.0,
            value_for_money: 0.0,
            honor_bonus: 0.0,
            performance_bonus: 0.0,
            potential_bonus: 0.0,
        }
    }
}

/// 计算球队对选手的兴趣度（0-100分）
pub fn calculate_interest_score(
    team: &Team,
    team_roster: &[Player],
    gm_profile: &TeamGMProfile,  // 改用 GM 配置
    player: &Player,
    position_needs: &HashMap<String, u32>,
    player_honors: Option<&PlayerHonorInfo>,
    player_performance: Option<&PlayerPerformanceInfo>,
    is_poaching: bool,  // 是否为挖角阶段
) -> InterestScore {
    let mut score = 0f64;
    let mut breakdown = ScoreBreakdown::new();

    // 1. 位置需求匹配（0-30分）
    let position_str = player.position
        .map(|p| format!("{:?}", p).to_uppercase())
        .unwrap_or_else(|| "MID".to_string());

    let need = position_needs.get(&position_str).unwrap_or(&0);
    let position_score = (*need as f64 / 100.0) * 30.0;
    score += position_score;
    breakdown.position_match = position_score;

    // 2. 能力匹配（0-30分）
    let avg_ability = calculate_team_avg_ability(team_roster);
    let ability_gap = player.ability as i16 - avg_ability as i16;
    let mut ability_score = match ability_gap {
        15.. => 30.0,      // 显著提升
        10..=14 => 25.0,   // 大幅提升
        5..=9 => 20.0,     // 中等提升
        0..=4 => 12.0,     // 小幅提升
        -5..=-1 => 5.0,    // 基本持平
        _ => 0.0,          // 不如现有
    };

    // GM 人格调整
    match gm_profile.personality {
        GMPersonality::Championship => {
            // 争冠型：追求明星选手
            if player.ability >= 90 {
                ability_score += 5.0;
            }
        }
        GMPersonality::YouthDevelopment => {
            // 青训型：中等实力好培养
            if player.ability >= 70 && player.ability <= 80 {
                ability_score += 5.0;
            }
        }
        _ => {}
    }

    score += ability_score;
    breakdown.ability_match = ability_score;

    // 3. 年龄偏好（0-15分）
    let age_score = if player.age >= gm_profile.preferred_age_min
        && player.age <= gm_profile.preferred_age_max {
        15.0
    } else {
        let gap_min = (player.age as i16 - gm_profile.preferred_age_min as i16).abs();
        let gap_max = (player.age as i16 - gm_profile.preferred_age_max as i16).abs();
        let gap = gap_min.min(gap_max);
        (15.0 - gap as f64 * 2.0).max(0.0)
    };
    score += age_score;
    breakdown.age_preference = age_score;

    // 4. 性价比（0-15分）
    let market_value = player.calculate_market_value();
    let transfer_fee = if is_poaching { market_value / 10000 } else { 0 };
    let annual_salary = player.salary / 10000;  // 年薪（万）
    let total_cost = transfer_fee + annual_salary * 2;  // 假设2年合同

    let cost_per_ability = if player.ability > 0 {
        total_cost as f64 / player.ability as f64
    } else {
        999.0
    };

    let value_score = if cost_per_ability < 30.0 { 15.0 }      // 极高性价比
                      else if cost_per_ability < 50.0 { 12.0 }  // 高性价比
                      else if cost_per_ability < 80.0 { 8.0 }   // 中等性价比
                      else if cost_per_ability < 120.0 { 4.0 }  // 略低
                      else { 0.0 };                              // 性价比差

    score += value_score;
    breakdown.value_for_money = value_score;

    // 5. 荣誉加成（0-10分）
    let honor_score = if let Some(honors) = player_honors {
        let total_championships = honors.worlds_championships + honors.msi_championships;
        let total_mvps = honors.tournament_mvps + honors.finals_mvps + honors.yearly_mvps;

        (total_championships as f64 * 2.0 + total_mvps as f64).min(10.0)
    } else {
        0.0
    };
    score += honor_score;
    breakdown.honor_bonus = honor_score;

    // 6. 表现加成（0-10分）
    let mut performance_score = if let Some(perf) = player_performance {
        match perf.performance_tier.as_str() {
            "顶级表现" => 10.0,
            "优秀表现" => 7.0,
            "合格表现" => 4.0,
            "一般表现" => 0.0,
            _ => -5.0,  // 表现欠佳扣分
        }
    } else {
        0.0
    };

    // ability_diff 调整
    if let Some(perf) = player_performance {
        if perf.ability_diff > 5.0 {
            performance_score += 2.0;  // 超常发挥
        } else if perf.ability_diff < -5.0 {
            performance_score -= 2.0;  // 表现不佳
        }
    }

    score += performance_score;
    breakdown.performance_bonus = performance_score;

    // 7. 潜力加成（0-10分，青训型和投机型看重）
    let potential_gap = player.potential as i16 - player.ability as i16;
    let potential_score = match gm_profile.personality {
        GMPersonality::YouthDevelopment | GMPersonality::Speculator => {
            match potential_gap {
                20.. => 10.0,
                15..=19 => 8.0,
                10..=14 => 5.0,
                5..=9 => 3.0,
                _ => 0.0,
            }
        }
        _ => 0.0,
    };
    score += potential_score;
    breakdown.potential_bonus = potential_score;

    // 根据 GM 人格调整兴趣阈值
    let interest_threshold = match gm_profile.personality {
        GMPersonality::Championship => 80.0,      // 争冠型：只要最好的
        GMPersonality::YouthDevelopment => 60.0,  // 青训型：更宽容
        GMPersonality::Balanced => 75.0,          // 稳健型：较保守
        GMPersonality::Speculator => 65.0,        // 投机型：机会主义
        GMPersonality::Rebuilding => 65.0,        // 重建型：广撒网
        GMPersonality::Custom => 70.0,            // 自定义：默认
    };

    InterestScore {
        total: score.min(100.0) as u8,
        breakdown,
        is_high_interest: score >= interest_threshold,
    }
}

/// 计算球队平均能力
fn calculate_team_avg_ability(roster: &[Player]) -> u8 {
    if roster.is_empty() {
        return 75;  // 默认值
    }

    let total: u32 = roster.iter().map(|p| p.ability as u32).sum();
    (total / roster.len() as u32) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_score_calculation() {
        // 测试兴趣度计算逻辑
    }

    #[test]
    fn test_team_avg_ability() {
        // 测试球队平均能力计算
    }
}
