use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerFormFactors {
    pub player_id: u64,
    pub form_cycle: f64,
    pub momentum: i8,
    pub last_performance: f64,
    pub last_match_won: bool,
    #[serde(default)]
    pub perf_history: String,
    pub games_since_rest: u32,
}

impl Default for PlayerFormFactors {
    fn default() -> Self {
        Self {
            player_id: 0,
            form_cycle: 50.0,
            momentum: 0,
            last_performance: 0.0,
            last_match_won: false,
            perf_history: String::new(),
            games_since_rest: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MatchContext {
    pub tournament_type: String,
    pub round: String,
    pub game_number: u8,
    pub is_decider: bool,
    pub score_diff: i8,
}

impl MatchContext {
    pub fn is_high_pressure(&self) -> bool {
        matches!(
            self.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "claude"
        ) || self.round == "final"
            || self.is_decider
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConditionContext {
    pub match_context: Option<MatchContext>,
    pub season_games_played: u32,
    pub satisfaction: u8,
    pub international_events: u8,
    pub has_ironman: bool,
}

pub struct ConditionEngine;

impl ConditionEngine {
    pub fn calculate_condition(
        age: u8,
        ability: u8,
        factors: &PlayerFormFactors,
        context: Option<&MatchContext>,
    ) -> i8 {
        let ctx = ConditionContext {
            match_context: context.cloned(),
            ..Default::default()
        };

        Self::calculate_condition_full(age, ability, factors, &ctx)
    }

    pub fn calculate_condition_full(
        age: u8,
        ability: u8,
        factors: &PlayerFormFactors,
        ctx: &ConditionContext,
    ) -> i8 {
        let amplitude = Self::get_amplitude_by_age(age);
        let primary = (factors.form_cycle * std::f64::consts::PI / 50.0).sin() * amplitude * 0.7;
        let secondary = (factors.form_cycle * std::f64::consts::PI / 20.0).sin() * amplitude * 0.3;
        let cycle_bonus = primary + secondary;

        let momentum_bonus = factors.momentum as f64 * 0.8;
        let confidence_bonus = Self::calculate_confidence(ability, &factors.perf_history);

        let pressure_penalty = if let Some(match_ctx) = ctx.match_context.as_ref() {
            Self::calculate_pressure_penalty(match_ctx, ctx)
        } else {
            0.0
        };

        let mut season_fatigue = -((ctx.season_games_played as f64 / 20.0).min(3.0));
        if ctx.has_ironman {
            season_fatigue *= 0.5;
        }

        let satisfaction_bonus = match ctx.satisfaction {
            0..=30 => -2.0,
            31..=40 => -1.0,
            41..=60 => 0.0,
            61..=80 => 0.0,
            81..=100 => 1.0,
            _ => 0.0,
        };

        let raw_condition = cycle_bonus
            + momentum_bonus
            + confidence_bonus
            + pressure_penalty
            + season_fatigue
            + satisfaction_bonus;

        let (min, max) = Self::get_condition_range_by_age(age);
        raw_condition.round().clamp(min as f64, max as f64) as i8
    }

    fn get_amplitude_by_age(age: u8) -> f64 {
        match age {
            16..=24 => 6.0,
            25..=29 => 4.0,
            _ => 2.0,
        }
    }

    pub fn get_condition_range_by_age(age: u8) -> (i8, i8) {
        match age {
            16..=24 => (-5, 8),
            25..=29 => (-3, 3),
            _ => (-1, 3),
        }
    }

    fn calculate_pressure_penalty(match_ctx: &MatchContext, ctx: &ConditionContext) -> f64 {
        let mut penalty = 0.0;

        if matches!(
            match_ctx.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "claude"
        ) {
            penalty -= 1.5;
        }

        if match_ctx.round == "final" {
            penalty -= 1.0;
        }

        if match_ctx.is_decider {
            penalty -= 0.5;
        }

        if match_ctx.score_diff < 0 {
            penalty -= 0.5;
        }

        let experience_factor = match ctx.international_events {
            0 => 1.5,
            1 => 1.2,
            2..=3 => 1.0,
            4..=6 => 0.8,
            _ => 0.6,
        };

        penalty * experience_factor
    }

    fn calculate_confidence(ability: u8, perf_history: &str) -> f64 {
        let perfs: Vec<f64> = perf_history
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();

        if perfs.is_empty() {
            return 0.0;
        }

        let weights = [0.4, 0.25, 0.2, 0.1, 0.05];
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for (i, perf) in perfs.iter().rev().enumerate().take(5) {
            let w = weights.get(i).copied().unwrap_or(0.05);
            weighted_sum += (perf - ability as f64) * w;
            weight_total += w;
        }

        if weight_total > 0.0 {
            (weighted_sum / weight_total * 0.3).clamp(-2.0, 2.0)
        } else {
            0.0
        }
    }

    fn push_perf_history(perf_history: &str, performance: f64) -> String {
        let mut perfs: Vec<f64> = perf_history
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();

        perfs.push(performance);

        if perfs.len() > 5 {
            let keep_from = perfs.len() - 5;
            perfs = perfs[keep_from..].to_vec();
        }

        perfs
            .iter()
            .map(|p| format!("{:.1}", p))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn update_form_factors(
        mut factors: PlayerFormFactors,
        won: bool,
        performance: f64,
    ) -> PlayerFormFactors {
        let cycle_step = 3.0 + rand::random::<f64>() * 5.0;
        factors.form_cycle = (factors.form_cycle + cycle_step) % 100.0;

        if won {
            factors.momentum = (factors.momentum + 1).min(5);
        } else {
            factors.momentum = (factors.momentum - 1).max(-5);
        }

        factors.last_performance = performance;
        factors.last_match_won = won;
        factors.perf_history = Self::push_perf_history(&factors.perf_history, performance);
        factors.games_since_rest += 1;

        factors
    }

    pub fn update_form_factors_bench(mut factors: PlayerFormFactors) -> PlayerFormFactors {
        let cycle_step = 2.0 + rand::random::<f64>() * 3.0;
        factors.form_cycle = (factors.form_cycle + cycle_step) % 100.0;

        factors.momentum = (factors.momentum as f64 * 0.8).round() as i8;
        factors.games_since_rest = (factors.games_since_rest as f64 * 0.4).floor() as u32;

        factors
    }

    pub fn reset_form_factors(player_id: u64) -> PlayerFormFactors {
        PlayerFormFactors {
            player_id,
            form_cycle: rand::random::<f64>() * 100.0,
            momentum: 0,
            last_performance: 0.0,
            last_match_won: false,
            perf_history: String::new(),
            games_since_rest: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_range_by_age() {
        assert_eq!(ConditionEngine::get_condition_range_by_age(18), (-5, 8));
        assert_eq!(ConditionEngine::get_condition_range_by_age(24), (-5, 8));
        assert_eq!(ConditionEngine::get_condition_range_by_age(25), (-3, 3));
        assert_eq!(ConditionEngine::get_condition_range_by_age(29), (-3, 3));
        assert_eq!(ConditionEngine::get_condition_range_by_age(30), (-1, 3));
        assert_eq!(ConditionEngine::get_condition_range_by_age(35), (-1, 3));
    }

    #[test]
    fn test_calculate_condition_default() {
        let factors = PlayerFormFactors::default();
        let condition = ConditionEngine::calculate_condition(25, 80, &factors, None);
        assert!(condition >= -3 && condition <= 3);
    }

    #[test]
    fn test_calculate_condition_with_momentum() {
        let mut factors = PlayerFormFactors::default();
        factors.momentum = 5;
        let condition = ConditionEngine::calculate_condition(25, 80, &factors, None);
        assert!(condition > 0);
    }

    #[test]
    fn test_calculate_condition_young_player() {
        let mut factors = PlayerFormFactors::default();
        factors.form_cycle = 75.0;
        let condition = ConditionEngine::calculate_condition(20, 80, &factors, None);
        assert!(condition >= -5 && condition <= 8);
    }

    #[test]
    fn test_update_form_factors_win() {
        let factors = PlayerFormFactors::default();
        let updated = ConditionEngine::update_form_factors(factors, true, 85.0);
        assert_eq!(updated.momentum, 1);
        assert!(updated.last_match_won);
        assert_eq!(updated.last_performance, 85.0);
        assert_eq!(updated.perf_history, "85.0");
    }

    #[test]
    fn test_update_form_factors_lose() {
        let factors = PlayerFormFactors::default();
        let updated = ConditionEngine::update_form_factors(factors, false, 70.0);
        assert_eq!(updated.momentum, -1);
        assert!(!updated.last_match_won);
        assert_eq!(updated.perf_history, "70.0");
    }

    #[test]
    fn test_momentum_bounds() {
        let mut factors = PlayerFormFactors::default();
        factors.momentum = 5;
        let updated = ConditionEngine::update_form_factors(factors, true, 85.0);
        assert_eq!(updated.momentum, 5);

        let mut factors2 = PlayerFormFactors::default();
        factors2.momentum = -5;
        let updated2 = ConditionEngine::update_form_factors(factors2, false, 70.0);
        assert_eq!(updated2.momentum, -5);
    }

    #[test]
    fn test_update_form_factors_bench() {
        let mut factors = PlayerFormFactors {
            player_id: 1,
            form_cycle: 30.0,
            momentum: 4,
            last_performance: 80.0,
            last_match_won: true,
            perf_history: "76.0,80.0".to_string(),
            games_since_rest: 5,
        };
        let updated = ConditionEngine::update_form_factors_bench(factors.clone());
        assert!(updated.form_cycle >= 32.0 || updated.form_cycle < 5.0);
        assert_eq!(updated.momentum, 3);
        assert_eq!(updated.games_since_rest, 2);
        assert_eq!(updated.last_performance, 80.0);
        assert!(updated.last_match_won);
        assert_eq!(updated.perf_history, "76.0,80.0");

        factors.momentum = -3;
        let updated2 = ConditionEngine::update_form_factors_bench(factors);
        assert_eq!(updated2.momentum, -2);
    }

    #[test]
    fn test_perf_history_keeps_latest_five() {
        let mut factors = PlayerFormFactors::default();

        for perf in [70.0, 71.0, 72.0, 73.0, 74.0, 75.0] {
            factors = ConditionEngine::update_form_factors(factors, true, perf);
        }

        assert_eq!(factors.perf_history, "71.0,72.0,73.0,74.0,75.0");
    }

    #[test]
    fn test_calculate_confidence_weighted_history() {
        let confidence = ConditionEngine::calculate_confidence(80, "70.0,80.0,90.0");
        assert!((confidence - 0.706).abs() < 0.02);
    }

    #[test]
    fn test_calculate_condition_full_respects_context() {
        let factors = PlayerFormFactors {
            player_id: 1,
            form_cycle: 0.0,
            momentum: 0,
            last_performance: 0.0,
            last_match_won: false,
            perf_history: String::new(),
            games_since_rest: 0,
        };

        let low_ctx = ConditionContext {
            season_games_played: 60,
            satisfaction: 20,
            ..Default::default()
        };
        let high_ctx = ConditionContext {
            season_games_played: 60,
            satisfaction: 90,
            has_ironman: true,
            ..Default::default()
        };

        let low = ConditionEngine::calculate_condition_full(25, 80, &factors, &low_ctx);
        let high = ConditionEngine::calculate_condition_full(25, 80, &factors, &high_ctx);

        assert!(high > low);
    }

    #[test]
    fn test_pressure_penalty_scales_with_international_experience() {
        let factors = PlayerFormFactors {
            player_id: 1,
            form_cycle: 0.0,
            momentum: 0,
            last_performance: 0.0,
            last_match_won: false,
            perf_history: String::new(),
            games_since_rest: 0,
        };

        let match_context = MatchContext {
            tournament_type: "worlds".to_string(),
            round: "final".to_string(),
            game_number: 5,
            is_decider: true,
            score_diff: -1,
        };

        let rookie_ctx = ConditionContext {
            match_context: Some(match_context.clone()),
            satisfaction: 60,
            international_events: 0,
            ..Default::default()
        };
        let veteran_ctx = ConditionContext {
            match_context: Some(match_context),
            satisfaction: 60,
            international_events: 6,
            ..Default::default()
        };

        let rookie_condition =
            ConditionEngine::calculate_condition_full(20, 80, &factors, &rookie_ctx);
        let veteran_condition =
            ConditionEngine::calculate_condition_full(20, 80, &factors, &veteran_ctx);

        assert!(rookie_condition < veteran_condition);
    }

    #[test]
    fn test_reset_form_factors_clears_perf_history() {
        let factors = ConditionEngine::reset_form_factors(42);
        assert_eq!(factors.perf_history, "");
    }
}
