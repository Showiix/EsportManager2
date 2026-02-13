//! AI 卖签决策逻辑

use super::{DraftAuctionEngine, TeamAuctionInfo};
use crate::models::FinancialStatus;
use rand::Rng;

impl DraftAuctionEngine {
    pub(super) fn evaluate_sell_decision(
        &self,
        team_info: &TeamAuctionInfo,
        position: u32,
        rng: &mut impl Rng,
    ) -> bool {
        let financial_motivation: f64 = match team_info.financial_status {
            FinancialStatus::Bankrupt => 0.80,
            FinancialStatus::Deficit => 0.60,
            FinancialStatus::Tight => 0.40,
            FinancialStatus::Healthy => 0.25,
            FinancialStatus::Wealthy => 0.15,
        };

        let pick_retention = match position {
            1 => 0.10,
            2 => 0.20,
            3 => 0.30,
            4..=5 => 0.50,
            6..=8 => 0.75,
            9..=10 => 1.00,
            11..=12 => 1.30,
            _ => 1.50,
        };

        let roster_factor = if team_info.roster_count < 5 {
            0.10
        } else if team_info.roster_count < 7 {
            0.50
        } else if team_info.roster_count >= 9 {
            1.50
        } else {
            1.00
        };

        let strength_factor = if team_info.avg_ability > 65.0 && position <= 5 {
            1.30
        } else if team_info.avg_ability < 55.0 && position <= 5 {
            0.50
        } else {
            1.00
        };

        let rookie_match_factor = self.calculate_rookie_match_factor(team_info, position);

        let sell_prob: f64 = (financial_motivation
            * pick_retention
            * roster_factor
            * strength_factor
            * rookie_match_factor)
            .clamp(0.0, 0.90);
        rng.gen::<f64>() < sell_prob
    }

    pub(super) fn calculate_rookie_match_factor(
        &self,
        team_info: &TeamAuctionInfo,
        draft_position: u32,
    ) -> f64 {
        if self.draft_rookies.is_empty() {
            return 1.0;
        }

        let reachable_rookies: Vec<&super::DraftRookieInfo> = self
            .draft_rookies
            .iter()
            .filter(|r| r.draft_rank <= draft_position + 1)
            .collect();

        if reachable_rookies.is_empty() {
            return 1.0;
        }

        let target_rookie = reachable_rookies
            .iter()
            .min_by_key(|r| (r.draft_rank as i32 - draft_position as i32).unsigned_abs())
            .unwrap();

        let pos_need = team_info
            .position_needs
            .get(&target_rookie.position)
            .copied()
            .unwrap_or(50);

        let rookie_score =
            target_rookie.ability as f64 * 0.4 + target_rookie.potential as f64 * 0.6;

        let best_match_score = reachable_rookies
            .iter()
            .map(|r| {
                let need = team_info
                    .position_needs
                    .get(&r.position)
                    .copied()
                    .unwrap_or(50) as f64;
                let score = r.ability as f64 * 0.4 + r.potential as f64 * 0.6;
                need / 100.0 * score
            })
            .fold(0.0_f64, f64::max);

        if pos_need >= 80 && rookie_score >= 70.0 {
            0.20
        } else if pos_need >= 60 && rookie_score >= 60.0 {
            0.45
        } else if best_match_score >= 50.0 {
            0.70
        } else if pos_need <= 30 && rookie_score < 55.0 {
            1.30
        } else {
            1.0
        }
    }
}
