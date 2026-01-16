//! 选手决策引擎（规则驱动）
//!
//! 选手评估所有报价并选择最优的一个

use crate::models::{PlayerTransferStrategy, Offer};
use serde::{Serialize, Deserialize};

/// 选手决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDecisionResult {
    pub player_id: u64,
    pub player_name: String,
    pub chosen_offer_id: u64,
    pub chosen_team_id: u64,
    pub chosen_team_name: String,
    pub reasoning: String,  // 规则生成的理由
}

/// 报价评分结果
#[derive(Debug, Clone)]
struct OfferScore {
    offer_id: u64,
    total_score: u8,
    salary_score: u8,
    team_score: u8,
    role_score: u8,
    other_score: u8,
}

/// 选手评估所有报价并选择最优（规则）
pub fn evaluate_offers_and_choose(
    player_id: u64,
    player_name: String,
    player_strategy: &PlayerTransferStrategy,
    offers: &[Offer],
) -> Option<PlayerDecisionResult> {
    if offers.is_empty() {
        return None;
    }

    // 如果只有1个报价
    if offers.len() == 1 {
        let offer = &offers[0];

        // 检查是否满足最低要求
        if offer.salary_offer < player_strategy.expected_min_salary {
            log::info!(
                "选手 {} 拒绝唯一报价：薪资{}万 < 最低要求{}万",
                player_name, offer.salary_offer, player_strategy.expected_min_salary
            );
            return None;
        }

        if player_strategy.requires_starter && !offer.guarantee_starter {
            log::info!(
                "选手 {} 拒绝唯一报价：要求首发但未保证",
                player_name
            );
            return None;
        }

        return Some(PlayerDecisionResult {
            player_id,
            player_name: player_name.clone(),
            chosen_offer_id: offer.id,
            chosen_team_id: offer.from_team_id,
            chosen_team_name: offer.from_team_name.clone(),
            reasoning: "唯一报价，符合基本要求，接受".to_string(),
        });
    }

    // 多个报价：计算每个报价的综合评分
    let mut scored_offers: Vec<OfferScore> = offers.iter()
        .map(|offer| {
            // 1. 薪资评分（0-40分）
            let salary_score = if offer.salary_offer >= player_strategy.expected_salary * 12 / 10 {
                40  // 超出期望20%
            } else if offer.salary_offer >= player_strategy.expected_salary {
                35  // 满足期望
            } else if offer.salary_offer >= player_strategy.expected_min_salary {
                25  // 勉强接受
            } else {
                0   // 不接受
            };

            // 2. 球队评分（0-30分）
            let team_priority = player_strategy.get_team_priority(offer.from_team_id);
            let team_score = match team_priority {
                Some(1) => 30,  // 第1偏好
                Some(2) => 25,  // 第2偏好
                Some(3) => 20,  // 第3偏好
                Some(4..=5) => 15,  // 4-5偏好
                _ => 10,  // 不在偏好列表
            };

            // 3. 角色评分（0-20分）
            let role_score = if player_strategy.requires_starter {
                if offer.guarantee_starter { 20 } else { 0 }
            } else {
                if offer.guarantee_starter { 15 } else { 10 }
            };

            // 4. 其他评分（0-10分）
            let mut other_score = 0u8;

            // 合同年限匹配
            if offer.contract_years == player_strategy.expected_years {
                other_score += 5;
            }

            // 签字费加成
            if offer.signing_bonus > 0 {
                other_score += 5;
            }

            // 综合评分
            let total_score = salary_score + team_score + role_score + other_score;

            OfferScore {
                offer_id: offer.id,
                total_score,
                salary_score,
                team_score,
                role_score,
                other_score,
            }
        })
        .collect();

    // 过滤掉不可接受的报价（薪资太低或不满足首发要求）
    scored_offers.retain(|score| score.salary_score > 0 && score.role_score > 0);

    if scored_offers.is_empty() {
        log::info!(
            "选手 {} 拒绝所有报价：所有报价都不满足基本要求",
            player_name
        );
        return None;
    }

    // 选择评分最高的报价
    scored_offers.sort_by(|a, b| b.total_score.cmp(&a.total_score));
    let best_score = &scored_offers[0];

    let chosen_offer = offers.iter()
        .find(|o| o.id == best_score.offer_id)
        .unwrap();

    // 生成选择理由（规则模板）
    let reasoning = format!(
        "综合评分: {}/100\n\n\
         评分详情：\n\
         • 薪资评分 {}/40 - 报价{}万/年{}\n\
         • 球队评分 {}/30 - {}\n\
         • 角色评分 {}/20 - {}\n\
         • 其他评分 {}/10\n\n\
         最终决定：接受 {} 的报价",
        best_score.total_score,
        best_score.salary_score,
        chosen_offer.salary_offer,
        if chosen_offer.salary_offer >= player_strategy.expected_salary {
            "（满足期望）"
        } else {
            "（可接受）"
        },
        best_score.team_score,
        if let Some(priority) = player_strategy.get_team_priority(chosen_offer.from_team_id) {
            format!("偏好球队第{}位", priority)
        } else {
            "非偏好球队".to_string()
        },
        best_score.role_score,
        if player_strategy.requires_starter {
            if chosen_offer.guarantee_starter { "首发保证" } else { "无首发保证" }
        } else {
            "不要求首发"
        },
        best_score.other_score,
        chosen_offer.from_team_name
    );

    Some(PlayerDecisionResult {
        player_id,
        player_name,
        chosen_offer_id: chosen_offer.id,
        chosen_team_id: chosen_offer.from_team_id,
        chosen_team_name: chosen_offer.from_team_name.clone(),
        reasoning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_decision() {
        // 测试选手决策逻辑
    }
}
