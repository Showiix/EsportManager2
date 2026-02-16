use std::collections::HashMap;

use crate::engines::condition::{ConditionEngine, MatchContext, PlayerFormFactors};
use crate::engines::traits::types::TraitType;
use crate::models::player::Position;
use crate::models::transfer::AITeamPersonality;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LineupCandidate {
    pub player_id: u64,
    pub game_id: String,
    pub position: Position,
    pub ability: u8,
    pub age: u8,
    pub potential: u8,
    pub condition: i8,
    pub form_factors: PlayerFormFactors,
    pub is_starter: bool,
    pub join_season: u32,
    pub traits: Vec<TraitType>,
    pub satisfaction: u8,
    pub champion_version_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionDecision {
    pub position: String,
    pub sub_in_player_id: u64,
    pub sub_in_name: String,
    pub sub_out_player_id: u64,
    pub sub_out_name: String,
    pub reason: String,
    pub delta_score: f64,
}

#[derive(Debug, Clone)]
pub struct SubstitutionContext {
    pub tournament_type: String,
    pub round: String,
    pub bo_count: u8,
    pub game_number: u8,
    pub home_score: u8,
    pub away_score: u8,
    pub is_home: bool,
    pub current_season: u32,
}

pub struct LineupEngine;

#[derive(Debug, Clone)]
struct EvaluatedSubstitution {
    decision: SubstitutionDecision,
    is_forced: bool,
}

impl LineupEngine {
    pub fn recalculate_condition(
        candidate: &LineupCandidate,
        context: Option<&MatchContext>,
    ) -> i8 {
        ConditionEngine::calculate_condition(
            candidate.age,
            candidate.ability,
            &candidate.form_factors,
            context,
        )
    }

    pub fn submit_roster(
        all_players: &[LineupCandidate],
        personality: &AITeamPersonality,
        weights: &crate::models::transfer::AIDecisionWeights,
        tournament_type: &str,
        current_season: u32,
    ) -> (Vec<LineupCandidate>, Vec<LineupCandidate>) {
        let max_subs = Self::max_subs_by_personality(personality);
        let mut starters = Vec::new();
        let mut bench_pool: Vec<(LineupCandidate, f64)> = Vec::new();

        for position in Self::positions() {
            let mut position_candidates: Vec<(LineupCandidate, f64)> = all_players
                .iter()
                .filter(|p| p.position == position)
                .map(|p| {
                    (
                        p.clone(),
                        Self::calculate_roster_score(p, weights, tournament_type, current_season),
                    )
                })
                .collect();

            position_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));

            if let Some((starter, _)) = position_candidates.first().cloned() {
                starters.push(starter);
            }

            for (idx, (candidate, score)) in position_candidates.into_iter().enumerate() {
                if idx == 0 {
                    continue;
                }
                bench_pool.push((candidate, score));
            }
        }

        bench_pool.sort_by(|a, b| b.1.total_cmp(&a.1));
        let subs = bench_pool
            .into_iter()
            .take(max_subs)
            .map(|(candidate, _)| candidate)
            .collect();

        (starters, subs)
    }

    pub fn check_substitutions(
        starters: &[LineupCandidate],
        subs: &[LineupCandidate],
        context: &SubstitutionContext,
        personality: &AITeamPersonality,
        current_season: u32,
        games_played_this_series: &HashMap<u64, u8>,
    ) -> Vec<SubstitutionDecision> {
        if context.bo_count == 1 || context.game_number <= 1 {
            return Vec::new();
        }

        let threshold = Self::calculate_threshold(context, personality);
        let mut evaluated = Vec::new();

        for position in Self::positions() {
            let Some(starter) = starters.iter().find(|p| p.position == position) else {
                continue;
            };

            let position_subs: Vec<&LineupCandidate> = subs
                .iter()
                .filter(|p| p.position == position)
                .filter(|p| games_played_this_series.get(&p.player_id) != Some(&1))
                .collect();

            if position_subs.is_empty() {
                continue;
            }

            let mut best_for_position: Option<EvaluatedSubstitution> = None;

            for sub in position_subs {
                let is_forced = Self::is_forced_substitution(starter, sub);

                let condition_gain = sub.condition as f64 - starter.condition as f64;
                let trait_gain = Self::evaluate_trait_fitness(&sub.traits, context)
                    - Self::evaluate_trait_fitness(&starter.traits, context);
                let fatigue_gain = if starter.traits.contains(&TraitType::Ironman) {
                    0.0
                } else {
                    let games = starter.form_factors.games_since_rest as f64;
                    // 3局以下无疲劳，3局以上加速累积
                    if games <= 3.0 {
                        0.0
                    } else {
                        (games - 3.0) * 1.2
                    }
                };
                let version_gain =
                    (sub.champion_version_score - starter.champion_version_score).max(0.0);
                let synergy_cost =
                    Self::calculate_synergy_cost(starters, starter, sub, current_season);
                let inertia_penalty = Self::inertia_penalty(personality);

                let ability_gap = sub.ability as f64 - starter.ability as f64;
                let ability_penalty = if ability_gap < -5.0 {
                    (ability_gap + 5.0) * 0.5
                } else {
                    0.0
                };

                let delta =
                    condition_gain + trait_gain + fatigue_gain + version_gain + ability_penalty
                        - synergy_cost
                        - inertia_penalty;

                if !is_forced && delta < threshold {
                    continue;
                }

                let decision = SubstitutionDecision {
                    position: Self::position_name(position).to_string(),
                    sub_in_player_id: sub.player_id,
                    sub_in_name: sub.game_id.clone(),
                    sub_out_player_id: starter.player_id,
                    sub_out_name: starter.game_id.clone(),
                    reason: Self::generate_reason(
                        starter,
                        sub,
                        condition_gain,
                        trait_gain,
                        fatigue_gain,
                        version_gain,
                        is_forced,
                        context,
                    ),
                    delta_score: delta,
                };

                let evaluated_decision = EvaluatedSubstitution {
                    decision,
                    is_forced,
                };
                if Self::is_better_sub(&evaluated_decision, best_for_position.as_ref()) {
                    best_for_position = Some(evaluated_decision);
                }
            }

            if let Some(best) = best_for_position {
                evaluated.push(best);
            }
        }

        evaluated.sort_by(|a, b| {
            b.is_forced
                .cmp(&a.is_forced)
                .then_with(|| b.decision.delta_score.total_cmp(&a.decision.delta_score))
        });

        evaluated
            .into_iter()
            .take(2)
            .map(|entry| entry.decision)
            .collect()
    }

    fn positions() -> [Position; 5] {
        [
            Position::Top,
            Position::Jug,
            Position::Mid,
            Position::Adc,
            Position::Sup,
        ]
    }

    fn position_name(position: Position) -> &'static str {
        match position {
            Position::Top => "Top",
            Position::Jug => "Jungle",
            Position::Mid => "Mid",
            Position::Adc => "ADC",
            Position::Sup => "Support",
        }
    }

    fn max_subs_by_personality(personality: &AITeamPersonality) -> usize {
        match personality {
            AITeamPersonality::WinNow => 1,
            AITeamPersonality::Aggressive => 3,
            AITeamPersonality::Conservative => 2,
            AITeamPersonality::Development => 3,
            AITeamPersonality::Balanced => 2,
        }
    }

    fn calculate_roster_score(
        player: &LineupCandidate,
        weights: &crate::models::transfer::AIDecisionWeights,
        tournament_type: &str,
        current_season: u32,
    ) -> f64 {
        let synergy_value = ((current_season as f64 - player.join_season as f64 + 1.0) * 0.4)
            .min(2.0)
            .max(0.0);

        player.ability as f64
            + synergy_value * 2.0
            + Self::trait_bonus_for_tournament(&player.traits, tournament_type)
            + player.potential as f64 * weights.youth_preference * 0.3
            + player.condition as f64 * 0.5
            + if player.is_starter { 5.0 } else { 0.0 }
    }

    fn trait_bonus_for_tournament(traits: &[TraitType], tournament_type: &str) -> f64 {
        let is_playoff_or_intl = matches!(
            tournament_type,
            "playoff" | "msi" | "worlds" | "masters" | "shanghai" | "claude" | "super" | "icp"
        );
        let is_bo5 = matches!(tournament_type, "playoff" | "worlds" | "super");

        let mut bonus = 0.0;
        for t in traits {
            match t {
                TraitType::Clutch if is_playoff_or_intl => bonus += 3.0,
                TraitType::Choker if is_playoff_or_intl => bonus -= 3.0,
                TraitType::Endurance if is_bo5 => bonus += 2.0,
                TraitType::SlowStarter if is_bo5 => bonus += 2.0,
                TraitType::FastStarter if !is_bo5 => bonus += 2.0,
                TraitType::Sprinter if !is_bo5 => bonus += 2.0,
                TraitType::FinalsKiller => bonus += 1.0,
                TraitType::TeamLeader => bonus += 1.5,
                _ => {}
            }
        }

        bonus
    }

    fn is_forced_substitution(starter: &LineupCandidate, sub: &LineupCandidate) -> bool {
        let ability_gap = starter.ability as i64 - sub.ability as i64;
        if ability_gap > 15 {
            return false;
        }

        // 状态极差：starter condition <= -3 且替补状态明显更好（差距>=3）
        if starter.condition <= -3 && sub.condition - starter.condition >= 3 {
            return true;
        }

        // 疲劳累积：连续出场6局以上（跨系列赛累积）
        if starter.form_factors.games_since_rest >= 6
            && !starter.traits.contains(&TraitType::Ironman)
        {
            return true;
        }

        // 心态崩盘
        starter.form_factors.momentum <= -4 && starter.condition < -2
    }

    fn inertia_penalty(personality: &AITeamPersonality) -> f64 {
        match personality {
            AITeamPersonality::Conservative => 1.5,
            AITeamPersonality::Aggressive => 0.2,
            AITeamPersonality::WinNow => 0.5,
            AITeamPersonality::Development => 0.8,
            AITeamPersonality::Balanced => 0.5,
        }
    }

    fn calculate_threshold(context: &SubstitutionContext, personality: &AITeamPersonality) -> f64 {
        let mut threshold = match personality {
            AITeamPersonality::Aggressive => 1.0,
            AITeamPersonality::WinNow => 1.5,
            AITeamPersonality::Balanced => 2.0,
            AITeamPersonality::Development => 1.5,
            AITeamPersonality::Conservative => 3.0,
        };

        if Self::is_trailing_match_point(context) {
            threshold *= 0.4;
        } else if context.game_number >= 3 {
            threshold *= 0.7;
        }

        threshold
    }

    fn is_trailing_match_point(context: &SubstitutionContext) -> bool {
        let (my_score, opp_score) = Self::series_score(context);
        let wins_needed = (context.bo_count / 2) + 1;
        opp_score + 1 >= wins_needed && my_score < opp_score
    }

    fn series_score(context: &SubstitutionContext) -> (u8, u8) {
        if context.is_home {
            (context.home_score, context.away_score)
        } else {
            (context.away_score, context.home_score)
        }
    }

    fn is_better_sub(
        candidate: &EvaluatedSubstitution,
        current_best: Option<&EvaluatedSubstitution>,
    ) -> bool {
        let Some(best) = current_best else {
            return true;
        };

        if candidate.is_forced != best.is_forced {
            return candidate.is_forced;
        }

        candidate.decision.delta_score > best.decision.delta_score
    }

    fn evaluate_trait_fitness(traits: &[TraitType], context: &SubstitutionContext) -> f64 {
        let is_playoff_or_intl = matches!(
            context.tournament_type.as_str(),
            "playoff" | "msi" | "worlds" | "masters" | "shanghai" | "claude" | "super" | "icp"
        );

        let (my_score, opp_score) = Self::series_score(context);
        let trailing = (my_score as i8) < (opp_score as i8);

        let mut fitness = 0.0;
        for t in traits {
            match t {
                TraitType::Clutch if is_playoff_or_intl => fitness += 3.0,
                TraitType::Choker if is_playoff_or_intl => fitness -= 3.0,
                TraitType::ComebackKing if trailing => fitness += 3.0,
                TraitType::Tilter if trailing => fitness -= 3.0,
                TraitType::Endurance if context.game_number >= 4 => fitness += 2.0,
                TraitType::Sprinter if context.game_number >= 4 => fitness -= 2.0,
                TraitType::FastStarter if context.game_number <= 2 => fitness += 2.0,
                TraitType::FinalsKiller if context.round == "final" => fitness += 3.0,
                TraitType::TeamLeader => fitness += 1.0,
                _ => {}
            }
        }

        fitness
    }

    fn calculate_synergy_cost(
        current_starters: &[LineupCandidate],
        starter_out: &LineupCandidate,
        sub_in: &LineupCandidate,
        current_season: u32,
    ) -> f64 {
        if current_starters.is_empty() {
            return 0.0;
        }

        let calc_tenure =
            |join: u32| -> f64 { ((current_season as f64) - (join as f64) + 1.0).max(0.0) };

        let current_avg: f64 = current_starters
            .iter()
            .map(|p| calc_tenure(p.join_season))
            .sum::<f64>()
            / current_starters.len() as f64;

        let new_avg: f64 = current_starters
            .iter()
            .map(|p| {
                if p.player_id == starter_out.player_id {
                    calc_tenure(sub_in.join_season)
                } else {
                    calc_tenure(p.join_season)
                }
            })
            .sum::<f64>()
            / current_starters.len() as f64;

        let tenure_loss = current_avg - new_avg;
        (tenure_loss * 0.4 * 2.5).max(0.0)
    }

    fn generate_reason(
        starter: &LineupCandidate,
        sub: &LineupCandidate,
        condition_gain: f64,
        trait_gain: f64,
        fatigue_gain: f64,
        version_gain: f64,
        is_forced: bool,
        context: &SubstitutionContext,
    ) -> String {
        if is_forced {
            if starter.condition <= -3 && sub.condition - starter.condition >= 3 {
                return format!(
                    "{}状态低迷(condition={})，{}状态更佳(condition={})，紧急替换",
                    starter.game_id, starter.condition, sub.game_id, sub.condition
                );
            }

            if starter.form_factors.games_since_rest >= 6
                && !starter.traits.contains(&TraitType::Ironman)
            {
                return format!(
                    "{}已连续出场{}局疲劳严重，{}体力充沛替换上阵",
                    starter.game_id, starter.form_factors.games_since_rest, sub.game_id
                );
            }

            if starter.form_factors.momentum <= -4 && starter.condition < -2 {
                return format!(
                    "{}连败{}场心态崩盘，换上{}调整",
                    starter.game_id,
                    starter.form_factors.momentum.abs(),
                    sub.game_id
                );
            }
        }

        if Self::is_trailing_match_point(context) {
            let (my_score, opp_score) = Self::series_score(context);
            return format!(
                "系列赛{}-{}落后，换上{}搏一搏",
                my_score, opp_score, sub.game_id
            );
        }

        let dominant_gain = condition_gain.max(trait_gain).max(fatigue_gain);

        if trait_gain > 0.0 && (trait_gain - dominant_gain).abs() < f64::EPSILON {
            let trait_name = Self::best_context_trait_name(&sub.traits, context).unwrap_or("关键");
            return format!(
                "战术调整：{}拥有{}特性，在{}中更具优势",
                sub.game_id,
                trait_name,
                Self::context_description(context)
            );
        }

        if condition_gain > 0.0 && (condition_gain - dominant_gain).abs() < f64::EPSILON {
            return format!(
                "状态轮换：{}状态值({})优于{}({})",
                sub.game_id, sub.condition, starter.game_id, starter.condition
            );
        }

        if fatigue_gain > 0.0 {
            return format!(
                "体能轮换：{}连续作战压力偏高，换上{}保持强度",
                starter.game_id, sub.game_id
            );
        }

        if version_gain >= 3.0 {
            return format!(
                "版本适配：{}的英雄池更契合当前版本，换上以获取BP优势",
                sub.game_id
            );
        }

        format!(
            "综合评估后换上{}，预计净收益{:.1}",
            sub.game_id,
            condition_gain + trait_gain + fatigue_gain + version_gain
        )
    }

    fn best_context_trait_name(
        traits: &[TraitType],
        context: &SubstitutionContext,
    ) -> Option<&'static str> {
        traits
            .iter()
            .map(|t| (t, Self::single_trait_fitness(t, context)))
            .filter(|(_, score)| *score > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(trait_type, _)| trait_type.display_name())
    }

    fn single_trait_fitness(trait_type: &TraitType, context: &SubstitutionContext) -> f64 {
        let is_playoff_or_intl = matches!(
            context.tournament_type.as_str(),
            "playoff" | "msi" | "worlds" | "masters" | "shanghai" | "claude" | "super" | "icp"
        );

        let (my_score, opp_score) = Self::series_score(context);
        let trailing = (my_score as i8) < (opp_score as i8);

        match trait_type {
            TraitType::Clutch if is_playoff_or_intl => 3.0,
            TraitType::Choker if is_playoff_or_intl => -3.0,
            TraitType::ComebackKing if trailing => 3.0,
            TraitType::Tilter if trailing => -3.0,
            TraitType::Endurance if context.game_number >= 4 => 2.0,
            TraitType::Sprinter if context.game_number >= 4 => -2.0,
            TraitType::FastStarter if context.game_number <= 2 => 2.0,
            TraitType::FinalsKiller if context.round == "final" => 3.0,
            TraitType::TeamLeader => 1.0,
            _ => 0.0,
        }
    }

    fn context_description(context: &SubstitutionContext) -> String {
        if context.round == "final" {
            return "决赛关键局".to_string();
        }

        if context.game_number >= 4 {
            return format!("BO{}后半段", context.bo_count);
        }

        format!("第{}局", context.game_number)
    }
}
