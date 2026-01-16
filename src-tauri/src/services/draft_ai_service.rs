//! 选秀 AI 决策服务
//!
//! 提供传统选秀的智能选人逻辑，根据球队状态选择最佳新秀

use crate::models::{DraftPlayer, Player};
use std::collections::HashMap;

/// 选秀 AI 决策服务
pub struct DraftAIService;

impl DraftAIService {
    /// 为球队选择最佳选秀球员（简化版本）
    pub fn select_best_draft_player(
        available_players: &[DraftPlayer],
        roster: &[Player],
    ) -> Option<DraftPlayer> {
        if available_players.is_empty() {
            return None;
        }

        let position_needs = Self::calculate_position_needs(roster);

        let mut scored: Vec<(DraftPlayer, f64)> = available_players
            .iter()
            .map(|p| {
                let score = Self::calculate_player_score(p, &position_needs, roster);
                (p.clone(), score)
            })
            .collect();

        // 按分数降序排序
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.first().map(|(p, _)| p.clone())
    }

    /// 计算选秀球员的综合评分（公开方法供外部使用）- 简化版本
    pub fn calculate_player_score(
        player: &DraftPlayer,
        position_needs: &HashMap<String, u32>,
        roster: &[Player],
    ) -> f64 {
        // 1. 基础评分（ability 和 potential 的加权平均）
        let ability_weight = 0.6;
        let potential_weight = 0.4;
        let mut score = player.ability as f64 * ability_weight + player.potential as f64 * potential_weight;

        // 2. 年龄偏好调整（默认偏好18-22岁）
        if player.age >= 18 && player.age <= 22 {
            score *= 1.1; // 符合年龄偏好，加成10%
        }

        // 3. 位置需求调整
        let position_str = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_else(|| "Unknown".to_string());
        let pos_need = position_needs.get(&position_str).copied().unwrap_or(50);

        // 位置需求因子
        let pos_factor = pos_need as f64 / 100.0;
        score *= 1.0 + pos_factor; // 最多翻倍

        // 4. 阵容深度调整（同位置球员过多时降低优先级）
        let same_position_count = roster
            .iter()
            .filter(|p| {
                p.position
                    .map(|pos| format!("{:?}", pos).to_uppercase())
                    == Some(position_str.clone())
            })
            .count();

        if same_position_count >= 2 {
            score *= 0.7; // 该位置已有2+球员，降低优先级
        } else if same_position_count == 0 {
            score *= 1.3; // 该位置空缺，急需，大幅加成
        }

        // 5. 年轻球员偏好加成（≤21岁）
        if player.age <= 21 {
            score *= 1.1;
        }

        score
    }

    /// 计算各位置的需求分数 (0-100)（公开方法供外部使用）
    pub fn calculate_position_needs(roster: &[Player]) -> HashMap<String, u32> {
        let mut needs = HashMap::new();
        let positions = vec!["TOP", "JUG", "MID", "ADC", "SUP"];

        for pos in positions {
            let count = roster
                .iter()
                .filter(|p| {
                    p.position
                        .map(|pp| format!("{:?}", pp).to_uppercase() == pos)
                        .unwrap_or(false)
                })
                .count();

            let need = match count {
                0 => 100, // 急需
                1 => 60,  // 需要替补
                2 => 30,  // 可考虑
                _ => 10,  // 充足
            };
            needs.insert(pos.to_string(), need);
        }

        needs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_position_needs() {
        let roster: Vec<Player> = vec![];
        let needs = DraftAIService::calculate_position_needs(&roster);

        // 空阵容时所有位置需求都是100
        assert_eq!(needs.get("TOP"), Some(&100));
        assert_eq!(needs.get("JUG"), Some(&100));
    }
}
