//! 球探筛选系统
//!
//! 参考 NBA 2K 的球探系统，为每支球队筛选出高兴趣的候选名单

use crate::models::{Player, Team, TeamGMProfile};
use crate::services::llm_service::{PlayerHonorInfo, PlayerPerformanceInfo};
use crate::engines::interest_scoring::{calculate_interest_score, InterestScore, ScoreBreakdown};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 球员候选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCandidate {
    pub player: Player,
    pub interest_score: u8,              // 兴趣度总分
    pub score_breakdown: ScoreBreakdown, // 评分明细
    pub estimated_transfer_fee: Option<u64>, // 预估转会费（挖角阶段）
    pub estimated_total_cost: u64,       // 预估总成本（2年）
}

/// 球探报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoutingReport {
    pub team_id: u64,
    pub team_name: String,
    pub candidates: Vec<PlayerCandidate>,  // 最多5-8个
    pub total_evaluated: usize,            // 总评估数
    pub filtered_count: usize,             // 基础过滤后数量
    pub high_interest_count: usize,        // 高兴趣数量
}

/// 为球队筛选转会候选名单
pub fn scout_transfer_candidates(
    team: &Team,
    team_roster: &[Player],
    gm_profile: &TeamGMProfile,  // 改用 GM 配置
    total_budget: u64,           // 总预算（从外部传入）
    all_available_players: &[Player],  // 自由球员或85+选手
    position_needs: &HashMap<String, u32>,
    player_honors: &HashMap<u64, PlayerHonorInfo>,
    player_performances: &HashMap<u64, PlayerPerformanceInfo>,
    is_poaching: bool,  // 是否为挖角阶段
) -> ScoutingReport {
    let total_evaluated = all_available_players.len();

    // 步骤1：基础过滤
    let filtered: Vec<&Player> = all_available_players.iter()
        .filter(|p| {
            // 不签自己队的选手
            if p.team_id == Some(team.id) {
                return false;
            }

            // 能力阈值
            if p.ability < gm_profile.min_ability_threshold {
                return false;
            }

            // 年龄范围
            if p.age < gm_profile.preferred_age_min || p.age > gm_profile.preferred_age_max {
                return false;
            }

            // 预算粗筛
            let market_value = p.calculate_market_value();
            let estimated_cost = if is_poaching {
                market_value / 10000 + p.salary / 10000 * 2  // 转会费 + 2年薪资
            } else {
                p.salary / 10000 * 2  // 只需薪资
            };

            if estimated_cost > total_budget {
                return false;
            }

            true
        })
        .collect();

    let filtered_count = filtered.len();

    // 步骤2：计算兴趣度
    let mut scored: Vec<(Player, InterestScore, u64)> = filtered.iter()
        .map(|p| {
            let score = calculate_interest_score(
                team,
                team_roster,
                gm_profile,  // 使用 GM 配置
                p,
                position_needs,
                player_honors.get(&p.id),
                player_performances.get(&p.id),
                is_poaching,
            );

            let market_value = p.calculate_market_value();
            let transfer_fee = if is_poaching { market_value / 10000 } else { 0 };
            let total_cost = transfer_fee + p.salary / 10000 * 2;

            ((*p).clone(), score, total_cost)
        })
        .collect();

    // 步骤3：只保留高兴趣
    scored.retain(|(_, score, _)| score.is_high_interest);

    let high_interest_count = scored.len();

    // 步骤4：排序（兴趣度降序）
    scored.sort_by(|(_, a, _), (_, b, _)| b.total.cmp(&a.total));

    // 步骤5：取前 N 个候选
    let max_candidates = if is_poaching { 5 } else { 8 };

    let top_candidates: Vec<PlayerCandidate> = scored.into_iter()
        .take(max_candidates)
        .map(|(p, score, total_cost)| PlayerCandidate {
            estimated_transfer_fee: if is_poaching {
                Some(p.calculate_market_value() / 10000)
            } else {
                None
            },
            estimated_total_cost: total_cost,
            interest_score: score.total,
            score_breakdown: score.breakdown,
            player: p,
        })
        .collect();

    ScoutingReport {
        team_id: team.id,
        team_name: team.name.clone(),
        candidates: top_candidates,
        total_evaluated,
        filtered_count,
        high_interest_count,
    }
}

/// 计算球队平均能力（辅助函数）
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
    fn test_scouting_filter() {
        // 测试球探筛选逻辑
    }
}
