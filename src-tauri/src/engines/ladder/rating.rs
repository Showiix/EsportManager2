use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RatingUpdate {
    pub player_id: i64,
    pub old_rating: i32,
    pub new_rating: i32,
    pub rating_change: i32,
    pub performance_bonus: i32,
    pub influence: f64,
}

pub struct LadderRatingEngine;

impl LadderRatingEngine {
    pub fn calculate_rating_changes(
        blue_players: &[(i64, i32)],
        red_players: &[(i64, i32)],
        blue_performances: &HashMap<i64, f64>,
        red_performances: &HashMap<i64, f64>,
        winner_side: &str,
    ) -> Vec<RatingUpdate> {
        let mut rng = StdRng::from_entropy();
        let mut updates = Vec::new();

        let blue_avg_perf = blue_performances.values().copied().sum::<f64>()
            / blue_performances.len().max(1) as f64;
        let red_avg_perf =
            red_performances.values().copied().sum::<f64>() / red_performances.len().max(1) as f64;

        for (player_id, rating) in blue_players {
            let is_winner = winner_side == "blue";
            let perf = blue_performances.get(player_id).copied().unwrap_or(0.0);
            let influence = perf - blue_avg_perf;
            let change = Self::calc_change(is_winner, influence, &mut rng);
            let new_rating = (*rating + change).max(0);

            updates.push(RatingUpdate {
                player_id: *player_id,
                old_rating: *rating,
                new_rating,
                rating_change: change,
                performance_bonus: (influence * 0.8).clamp(-6.0, 6.0).round() as i32,
                influence,
            });
        }

        for (player_id, rating) in red_players {
            let is_winner = winner_side == "red";
            let perf = red_performances.get(player_id).copied().unwrap_or(0.0);
            let influence = perf - red_avg_perf;
            let change = Self::calc_change(is_winner, influence, &mut rng);
            let new_rating = (*rating + change).max(0);

            updates.push(RatingUpdate {
                player_id: *player_id,
                old_rating: *rating,
                new_rating,
                rating_change: change,
                performance_bonus: (influence * 0.8).clamp(-6.0, 6.0).round() as i32,
                influence,
            });
        }

        updates
    }

    fn calc_change(is_winner: bool, influence: f64, rng: &mut StdRng) -> i32 {
        // 基础分: 赢 +22, 输 -14 (非对称，赢的收益大于输的损失)
        let base = if is_winner { 22.0 } else { -14.0 };
        // 影响力加权: influence 范围大约 -8~+8, 乘 0.8 得 -6.4~+6.4
        let influence_bonus = (influence * 0.8).clamp(-6.0, 6.0);
        // 随机因子: ±3
        let noise: f64 = rng.gen_range(-3.0..=3.0);
        (base + influence_bonus + noise).round() as i32
    }
}
