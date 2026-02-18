use super::matchmaker::{LadderMatch, LadderPlayer, LadderTeam};
use crate::engines::bp_engine::{BpEngine, PlayerChampionPool};
use crate::engines::champion::VersionTier;
use crate::engines::match_simulation::MatchSimulationEngine;
use crate::engines::meta_engine::MetaType;
use crate::engines::traits::TraitType;
use crate::models::player::Position;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderMatchResult {
    pub round_number: i32,
    pub match_number: i32,
    pub blue_team: Vec<LadderPlayerSnapshot>,
    pub red_team: Vec<LadderPlayerSnapshot>,
    pub blue_avg_rating: i32,
    pub red_avg_rating: i32,
    pub blue_power: f64,
    pub red_power: f64,
    pub winner_side: String,
    pub mvp_player_id: i64,
    pub mvp_player_name: String,
    pub game_duration: i32,
    pub performances: HashMap<i64, f64>,
    pub draft_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderPlayerSnapshot {
    pub player_id: i64,
    pub player_name: String,
    pub game_id: String,
    pub position: String,
    pub team_name: Option<String>,
    pub rating: i32,
}

pub struct LadderSimulator {
    match_sim_engine: MatchSimulationEngine,
}

impl LadderSimulator {
    pub fn new() -> Self {
        Self {
            match_sim_engine: MatchSimulationEngine::default(),
        }
    }

    pub fn simulate_match(
        &self,
        ladder_match: &LadderMatch,
        round_number: i32,
        match_number: i32,
        player_full_data: &HashMap<i64, PlayerFullData>,
        version_tiers: &HashMap<u8, VersionTier>,
    ) -> LadderMatchResult {
        let mut rng = StdRng::from_entropy();

        let blue_pools: Vec<PlayerChampionPool> = ladder_match
            .blue_team
            .players
            .iter()
            .filter_map(|p| self.build_champion_pool(p, player_full_data))
            .collect();

        let red_pools: Vec<PlayerChampionPool> = ladder_match
            .red_team
            .players
            .iter()
            .filter_map(|p| self.build_champion_pool(p, player_full_data))
            .collect();

        let draft_result = if blue_pools.len() == 5 && red_pools.len() == 5 {
            BpEngine::run_draft(
                &blue_pools,
                &red_pools,
                version_tiers,
                MetaType::Balanced,
                &mut rng,
                &[],
                &[],
                None,
            )
        } else {
            return self.create_error_result(ladder_match, round_number, match_number);
        };

        let blue_power = self.calculate_team_power(
            &ladder_match.blue_team,
            player_full_data,
            &draft_result.home_bp_modifiers,
        );
        let red_power = self.calculate_team_power(
            &ladder_match.red_team,
            player_full_data,
            &draft_result.away_bp_modifiers,
        );

        let (blue_perf, red_perf, winner_id) = self
            .match_sim_engine
            .simulate_game(blue_power, red_power, 1, 2);

        let winner_side = if winner_id == 1 { "blue" } else { "red" };

        // 给每个选手基于 ability 的独立 performance，而非共享同一个值
        // 否则 influence = player_perf - team_avg 永远为 0
        let mut performances = HashMap::new();
        let mut rng_perf = StdRng::from_entropy();
        for player in &ladder_match.blue_team.players {
            let ability = player_full_data
                .get(&player.player_id)
                .map(|d| d.ability as f64)
                .unwrap_or(50.0);
            let noise: f64 = (rng_perf.gen::<f64>() - 0.5) * 6.0;
            let player_perf = blue_perf + (ability - blue_power) + noise;
            performances.insert(player.player_id, player_perf);
        }
        for player in &ladder_match.red_team.players {
            let ability = player_full_data
                .get(&player.player_id)
                .map(|d| d.ability as f64)
                .unwrap_or(50.0);
            let noise: f64 = (rng_perf.gen::<f64>() - 0.5) * 6.0;
            let player_perf = red_perf + (ability - red_power) + noise;
            performances.insert(player.player_id, player_perf);
        }

        let (mvp_player_id, mvp_player_name) = self.find_mvp(
            &ladder_match.blue_team,
            &ladder_match.red_team,
            &performances,
            winner_side,
        );

        LadderMatchResult {
            round_number,
            match_number,
            blue_team: ladder_match
                .blue_team
                .players
                .iter()
                .map(|p| LadderPlayerSnapshot {
                    player_id: p.player_id,
                    player_name: p.player_name.clone(),
                    game_id: p.game_id.clone(),
                    position: p.position.clone(),
                    team_name: p.team_name.clone(),
                    rating: p.rating,
                })
                .collect(),
            red_team: ladder_match
                .red_team
                .players
                .iter()
                .map(|p| LadderPlayerSnapshot {
                    player_id: p.player_id,
                    player_name: p.player_name.clone(),
                    game_id: p.game_id.clone(),
                    position: p.position.clone(),
                    team_name: p.team_name.clone(),
                    rating: p.rating,
                })
                .collect(),
            blue_avg_rating: ladder_match.blue_avg_rating,
            red_avg_rating: ladder_match.red_avg_rating,
            blue_power,
            red_power,
            winner_side: winner_side.to_string(),
            mvp_player_id,
            mvp_player_name,
            game_duration: 30 + (rand::random::<i32>() % 20),
            performances,
            draft_result: Some(serde_json::to_string(&draft_result).unwrap_or_default()),
        }
    }

    fn build_champion_pool(
        &self,
        player: &LadderPlayer,
        player_full_data: &HashMap<i64, PlayerFullData>,
    ) -> Option<PlayerChampionPool> {
        let full_data = player_full_data.get(&player.player_id)?;

        Some(PlayerChampionPool {
            player_id: player.player_id as u64,
            position: Position::from_str(&player.position),
            ability: full_data.ability,
            masteries: full_data.masteries.clone(),
            games_played: HashMap::new(),
            games_won: HashMap::new(),
            traits: full_data.traits.clone(),
        })
    }

    fn calculate_team_power(
        &self,
        team: &LadderTeam,
        player_full_data: &HashMap<i64, PlayerFullData>,
        bp_modifiers: &HashMap<u64, f64>,
    ) -> f64 {
        let mut total_power = 0.0;
        let mut count = 0;

        for player in &team.players {
            if let Some(full_data) = player_full_data.get(&player.player_id) {
                let base_power = full_data.ability as f64;
                let bp_modifier = bp_modifiers
                    .get(&(player.player_id as u64))
                    .copied()
                    .unwrap_or(0.0);
                total_power += base_power + bp_modifier;
                count += 1;
            }
        }

        if count > 0 {
            total_power / count as f64
        } else {
            50.0
        }
    }

    fn find_mvp(
        &self,
        blue_team: &LadderTeam,
        red_team: &LadderTeam,
        performances: &HashMap<i64, f64>,
        winner_side: &str,
    ) -> (i64, String) {
        let winning_team = if winner_side == "blue" {
            &blue_team.players
        } else {
            &red_team.players
        };

        let mut best_player = winning_team.first().unwrap();
        let mut best_perf = performances
            .get(&best_player.player_id)
            .copied()
            .unwrap_or(0.0);

        for player in winning_team {
            let perf = performances.get(&player.player_id).copied().unwrap_or(0.0);
            if perf > best_perf {
                best_perf = perf;
                best_player = player;
            }
        }

        (best_player.player_id, best_player.player_name.clone())
    }

    fn create_error_result(
        &self,
        ladder_match: &LadderMatch,
        round_number: i32,
        match_number: i32,
    ) -> LadderMatchResult {
        LadderMatchResult {
            round_number,
            match_number,
            blue_team: ladder_match
                .blue_team
                .players
                .iter()
                .map(|p| LadderPlayerSnapshot {
                    player_id: p.player_id,
                    player_name: p.player_name.clone(),
                    game_id: p.game_id.clone(),
                    position: p.position.clone(),
                    team_name: p.team_name.clone(),
                    rating: p.rating,
                })
                .collect(),
            red_team: ladder_match
                .red_team
                .players
                .iter()
                .map(|p| LadderPlayerSnapshot {
                    player_id: p.player_id,
                    player_name: p.player_name.clone(),
                    game_id: p.game_id.clone(),
                    position: p.position.clone(),
                    team_name: p.team_name.clone(),
                    rating: p.rating,
                })
                .collect(),
            blue_avg_rating: ladder_match.blue_avg_rating,
            red_avg_rating: ladder_match.red_avg_rating,
            blue_power: 50.0,
            red_power: 50.0,
            winner_side: "blue".to_string(),
            mvp_player_id: 0,
            mvp_player_name: "Unknown".to_string(),
            game_duration: 35,
            performances: HashMap::new(),
            draft_result: None,
        }
    }

    pub fn calculate_influence(
        &self,
        _player_id: i64,
        player_performance: f64,
        team_performances: &[f64],
    ) -> f64 {
        let team_avg = team_performances.iter().sum::<f64>() / team_performances.len() as f64;
        player_performance - team_avg
    }
}

#[derive(Debug, Clone)]
pub struct PlayerFullData {
    pub ability: u8,
    pub masteries: HashMap<u8, crate::engines::champion::MasteryTier>,
    pub traits: Vec<TraitType>,
}

impl Default for LadderSimulator {
    fn default() -> Self {
        Self::new()
    }
}
