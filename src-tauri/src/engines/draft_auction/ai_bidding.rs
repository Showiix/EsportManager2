//! AI 竞价决策逻辑

use super::{DraftAuctionEngine, TeamAuctionInfo};
use crate::models::{get_price_for_position, FinancialStatus};
use rand::Rng;

impl DraftAuctionEngine {
    /// AI 竞拍决策
    pub(super) fn evaluate_bid_for_listing(
        &self,
        team_info: &TeamAuctionInfo,
        draft_position: u32,
        current_price: i64,
        min_increment: i64,
        _current_bid_round: u32,
        buyer_team_id: Option<u64>,
        current_round: u32,
        rng: &mut impl Rng,
    ) -> Option<i64> {
        // 已持有更靠前签位且新秀位置重叠 → 跳过
        if let Some(&owned_pos) = self.draft_orders.get(&team_info.team_id) {
            if owned_pos < draft_position {
                let owned_rookie_position = self
                    .draft_rookies
                    .iter()
                    .filter(|r| r.draft_rank <= owned_pos + 1)
                    .min_by_key(|r| (r.draft_rank as i32 - owned_pos as i32).unsigned_abs())
                    .map(|r| r.position.clone());
                let target_rookie_position = self
                    .draft_rookies
                    .iter()
                    .filter(|r| r.draft_rank <= draft_position + 1)
                    .min_by_key(|r| (r.draft_rank as i32 - draft_position as i32).unsigned_abs())
                    .map(|r| r.position.clone());
                if let (Some(owned_rp), Some(target_rp)) =
                    (owned_rookie_position, target_rookie_position)
                {
                    if owned_rp == target_rp {
                        return None;
                    }
                }
            }
        }

        let budget_ratio = match team_info.financial_status {
            FinancialStatus::Wealthy => 0.40,
            FinancialStatus::Healthy => 0.30,
            FinancialStatus::Tight => 0.15,
            FinancialStatus::Deficit => 0.05,
            FinancialStatus::Bankrupt => return None,
        };
        let available_budget = (team_info.balance as f64 * budget_ratio) as i64;
        let min_bid = current_price + min_increment;

        if available_budget < min_bid || team_info.roster_count >= 8 {
            return None;
        }

        let pick_value: f64 = match draft_position {
            1 => 100.0,
            2 => 92.0,
            3 => 85.0,
            4 => 78.0,
            5 => 72.0,
            6 => 65.0,
            7 => 58.0,
            8 => 52.0,
            9 => 45.0,
            10 => 40.0,
            11 => 35.0,
            12 => 30.0,
            13 => 25.0,
            _ => 20.0,
        };

        let need_score = if team_info.roster_count < 5 {
            1.00
        } else if team_info.roster_count < 7 {
            0.60
        } else {
            0.30
        };

        let strength_desire = if team_info.avg_ability < 55.0 {
            1.40
        } else if team_info.avg_ability < 60.0 {
            1.15
        } else if team_info.avg_ability > 65.0 {
            0.70
        } else {
            1.00
        };

        let rookie_desire = self.calculate_rookie_bid_factor(team_info, draft_position);

        let mut bid_prob =
            (pick_value / 100.0) * 0.50 * need_score * strength_desire * rookie_desire;

        if let Some(pricing) = get_price_for_position(draft_position) {
            let price_ratio = current_price as f64 / pricing.starting_price as f64;
            if price_ratio > 1.0 {
                bid_prob *= (0.75_f64).powf(price_ratio - 1.0);
            }
        }

        bid_prob *= match team_info.financial_status {
            FinancialStatus::Wealthy => 1.30,
            FinancialStatus::Healthy => 1.00,
            FinancialStatus::Tight => 0.60,
            FinancialStatus::Deficit => 0.25,
            FinancialStatus::Bankrupt => 0.00,
        };

        if current_round > 1 && buyer_team_id.is_some() {
            bid_prob *= 0.65;
        }

        if rng.gen::<f64>() >= bid_prob {
            return None;
        }

        let aggression = match team_info.financial_status {
            FinancialStatus::Wealthy => 1.6,
            FinancialStatus::Healthy => 1.4,
            FinancialStatus::Tight => 1.2,
            _ => 1.05,
        };
        let max_bid = (min_bid as f64 * aggression).min(available_budget as f64) as i64;
        if max_bid <= min_bid {
            return None;
        }

        Some(rng.gen_range(min_bid..=max_bid))
    }

    /// 计算新秀匹配度对买签意愿的影响
    pub(super) fn calculate_rookie_bid_factor(
        &self,
        team_info: &TeamAuctionInfo,
        draft_position: u32,
    ) -> f64 {
        if self.draft_rookies.is_empty() {
            return 1.0;
        }

        let target_rookie = self
            .draft_rookies
            .iter()
            .filter(|r| r.draft_rank <= draft_position + 1)
            .min_by_key(|r| (r.draft_rank as i32 - draft_position as i32).unsigned_abs());

        let target_rookie = match target_rookie {
            Some(r) => r,
            None => return 0.70,
        };

        let pos_need = team_info
            .position_needs
            .get(&target_rookie.position)
            .copied()
            .unwrap_or(50);
        let rookie_score =
            target_rookie.ability as f64 * 0.4 + target_rookie.potential as f64 * 0.6;

        if pos_need >= 80 && rookie_score >= 70.0 {
            1.60
        } else if pos_need >= 60 && rookie_score >= 60.0 {
            1.30
        } else if pos_need <= 20 {
            0.50
        } else {
            1.0
        }
    }
}
