//! 报价决策引擎
//!
//! 从球探报告中决定是否报价以及报价条件

use crate::models::{Player, Team, TeamGMProfile, PlayerTransferStrategy, GMPersonality};
use crate::engines::scouting::PlayerCandidate;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 报价决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferDecision {
    pub player_id: u64,
    pub player_name: String,
    pub interest_score: u8,
    pub salary_offer: u64,          // 年薪报价（万）
    pub contract_years: u8,
    pub guarantee_starter: bool,
    pub transfer_fee: u64,          // 转会费（万，挖角阶段）
    pub reasoning: String,          // 报价理由（规则生成）
}

/// 从候选名单中决定报价
pub fn decide_offer(
    team: &Team,
    candidates: &[PlayerCandidate],
    gm_profile: &TeamGMProfile,  // 改用 GM 配置
    transfer_budget: u64,        // 转会预算
    total_budget: u64,           // 总预算
    player_strategies: &HashMap<u64, PlayerTransferStrategy>,
    is_poaching: bool,
) -> Option<OfferDecision> {
    if candidates.is_empty() {
        return None;
    }

    // 只选择兴趣度最高的1个
    let best = &candidates[0];
    let player = &best.player;

    // 检查转会费预算（挖角阶段）
    let transfer_fee = best.estimated_transfer_fee.unwrap_or(0);

    if is_poaching && transfer_fee > transfer_budget {
        log::info!(
            "○ {} 转会费预算不足：{}万（需要{}万）",
            team.name, transfer_budget, transfer_fee
        );
        return None;
    }

    // 检查挖角难度（只在挖角阶段）
    if is_poaching {
        let difficulty = calculate_poaching_difficulty(player, player_strategies.get(&player.id));

        // 难度太大 + 兴趣度不够高 → 放弃
        if (difficulty >= 90 && best.interest_score < 85)
            || (difficulty >= 80 && best.interest_score < 75) {
            log::info!(
                "○ {} 挖角难度太大：{}（兴趣度{}），放弃 {}",
                team.name, difficulty, best.interest_score, player.game_id
            );
            return None;
        }
    }

    // 决定报价金额（基于兴趣度和人格）
    let player_strategy = player_strategies.get(&player.id);
    let expected_salary = player_strategy
        .map(|s| s.expected_salary)
        .unwrap_or(player.salary / 10000);

    // 计算溢价系数
    let max_premium = gm_profile.price_premium_max;  // 从 GM 配置获取

    let salary_multiplier = match best.interest_score {
        90..=100 => max_premium,  // 极高兴趣：最大溢价
        80..=89 => 1.0 + (max_premium - 1.0) * 0.7,  // 高兴趣：70%溢价
        70..=79 => 1.0 + (max_premium - 1.0) * 0.4,  // 中等兴趣：40%溢价
        _ => 1.0,  // 正常报价
    };

    let salary_offer = (expected_salary as f64 * salary_multiplier) as u64;

    // 检查总预算
    let estimated_cost = salary_offer * 2 + transfer_fee;
    if estimated_cost > total_budget {
        log::info!(
            "○ {} 总成本超预算：{}万（预算{}万）",
            team.name, estimated_cost, total_budget
        );
        return None;
    }

    // 生成报价理由（规则模板）
    let reasoning = format_offer_reasoning(team, player, best, salary_multiplier, is_poaching);

    Some(OfferDecision {
        player_id: player.id,
        player_name: player.game_id.clone(),
        interest_score: best.interest_score,
        salary_offer,
        contract_years: 2,  // 默认2年
        guarantee_starter: best.interest_score >= 80,  // 高兴趣保证首发
        transfer_fee,
        reasoning,
    })
}

/// 计算挖角难度（0-100）
pub fn calculate_poaching_difficulty(
    player: &Player,
    player_strategy: Option<&PlayerTransferStrategy>,
) -> u8 {
    let mut difficulty = 50u8;  // 基础难度

    // 满意度影响
    match player.satisfaction {
        85.. => difficulty = difficulty.saturating_add(30),   // 非常满意，很难挖
        70..=84 => difficulty = difficulty.saturating_add(15), // 满意，有点难
        50..=69 => { /* 不变 */ }    // 一般，正常难度
        30..=49 => difficulty = difficulty.saturating_sub(15), // 不满，容易
        _ => difficulty = difficulty.saturating_sub(30),       // 极度不满，很容易
    }

    // 忠诚度影响
    match player.loyalty {
        90.. => difficulty = difficulty.saturating_add(30),   // 忠心耿耿，极难挖
        70..=89 => difficulty = difficulty.saturating_add(15), // 忠诚，较难
        50..=69 => { /* 不变 */ }    // 中立，正常
        30..=49 => difficulty = difficulty.saturating_sub(10), // 机会主义，容易
        _ => difficulty = difficulty.saturating_sub(20),       // 雇佣兵，很容易
    }

    // 是否想离队（最大影响）
    if let Some(strategy) = player_strategy {
        if strategy.wants_to_leave {
            difficulty = difficulty.saturating_sub(40);  // 想离队，难度大幅降低
        }
    }

    difficulty.clamp(10, 100)
}

/// 格式化报价理由（规则生成的文案）
fn format_offer_reasoning(
    team: &Team,
    player: &Player,
    candidate: &PlayerCandidate,
    salary_multiplier: f64,
    is_poaching: bool,
) -> String {
    let breakdown = &candidate.score_breakdown;

    let strategy_text = if salary_multiplier > 1.1 {
        format!("高兴趣，溢价{:.0}%报价", (salary_multiplier - 1.0) * 100.0)
    } else if salary_multiplier < 0.95 {
        format!("折价{:.0}%报价", (1.0 - salary_multiplier) * 100.0)
    } else {
        "正常报价".to_string()
    };

    format!(
        "【球探报告】兴趣度评分: {}/100\n\n\
         评分详情：\n\
         • 位置匹配 {:.0}分 - 该位置需求度高\n\
         • 能力提升 {:.0}分 - 能力{}，提升球队实力\n\
         • 年龄偏好 {:.0}分 - {}岁，符合球队策略\n\
         • 性价比 {:.0}分 - 每能力点成本{:.0}万\n\
         • 荣誉加成 {:.0}分{}\n\
         • 表现加成 {:.0}分{}\n\
         • 潜力加成 {:.0}分{}\n\n\
         报价策略：{}\n\
         预算状况：剩余转会预算{}万{}",
        candidate.interest_score,
        breakdown.position_match,
        breakdown.ability_match, player.ability,
        breakdown.age_preference, player.age,
        breakdown.value_for_money,
        candidate.estimated_total_cost as f64 / player.ability as f64,
        breakdown.honor_bonus,
        if breakdown.honor_bonus > 0.0 {
            format!(" - 国际赛冠军选手")
        } else {
            String::new()
        },
        breakdown.performance_bonus,
        if breakdown.performance_bonus > 0.0 {
            format!(" - 本赛季表现优秀")
        } else {
            String::new()
        },
        breakdown.potential_bonus,
        if breakdown.potential_bonus > 0.0 {
            format!(" - 高潜力新星")
        } else {
            String::new()
        },
        strategy_text,
        team.balance / 10000,
        if is_poaching {
            format!("，预估转会费{}万", candidate.estimated_transfer_fee.unwrap_or(0))
        } else {
            String::new()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poaching_difficulty() {
        // 测试挖角难度计算
    }
}
