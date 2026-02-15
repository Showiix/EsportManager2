use super::lineup_engine::{LineupCandidate, LineupEngine, SubstitutionContext};
use super::meta_engine::{MetaEngine, MetaWeights};
use super::traits::{TraitContext, TraitEngine, TraitType};
use super::PlayerFormFactors;
use crate::models::player::Position;
use crate::models::transfer::AITeamPersonality;
use crate::models::{Match, MatchFormat, MatchGame, MatchResult, MatchStatus};
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use std::sync::Mutex;

/// 比赛用选手信息（用于快速模拟路径的特性感知）
#[derive(Debug, Clone)]
pub struct MatchPlayerInfo {
    pub player_id: u64,
    pub ability: u8,
    pub stability: u8,
    pub condition: i8,
    pub age: u8,
    pub position: String,
    pub traits: Vec<TraitType>,
    pub is_first_season: bool,
    pub is_starter: bool,
    pub join_season: i64,
    pub potential: u8,
    pub satisfaction: u8,
    pub form_factors: Option<super::PlayerFormFactors>,
    pub bp_modifier: f64,
    pub champion_version_score: f64,
}

/// 比赛情境信息（用于快速模拟路径的特性触发判断）
#[derive(Debug, Clone)]
pub struct MatchSimContext {
    pub is_playoff: bool,
    pub is_international: bool,
    pub tournament_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct TeamPowerBreakdown {
    pub base_power: f64,
    pub synergy_bonus: f64,
    pub bp_bonus: f64,
    pub version_bonus: f64,
    pub meta_power: f64,
}

/// 比赛模拟引擎 - 基于正态分布的胜负判定
pub struct MatchSimulationEngine {
    /// 标准差 (控制发挥波动程度)
    std_dev: f64,
    last_games_played: Mutex<HashMap<u64, u8>>,
}

impl Default for MatchSimulationEngine {
    fn default() -> Self {
        Self {
            std_dev: 6.0,
            last_games_played: Mutex::new(HashMap::new()),
        }
    }
}

impl MatchSimulationEngine {
    pub fn new(std_dev: f64) -> Self {
        Self {
            std_dev,
            last_games_played: Mutex::new(HashMap::new()),
        }
    }

    pub fn take_last_games_played(&self) -> HashMap<u64, u8> {
        std::mem::take(&mut *self.last_games_played.lock().unwrap())
    }

    /// 基于Box-Muller变换生成正态分布随机数
    fn gaussian_random(&self, mean: f64) -> f64 {
        let normal = Normal::new(mean, self.std_dev).unwrap();
        let mut rng = rand::thread_rng();
        normal.sample(&mut rng)
    }

    /// 模拟单局比赛
    /// 返回: (主队发挥值, 客队发挥值, 获胜队伍ID)
    pub fn simulate_game(
        &self,
        home_power: f64,
        away_power: f64,
        home_team_id: u64,
        away_team_id: u64,
    ) -> (f64, f64, u64) {
        let home_performance = self.gaussian_random(home_power);
        let away_performance = self.gaussian_random(away_power);

        let winner_id = if home_performance > away_performance {
            home_team_id
        } else {
            away_team_id
        };

        (home_performance, away_performance, winner_id)
    }

    /// 模拟BO系列赛
    pub fn simulate_match(
        &self,
        match_id: u64,
        tournament_id: u64,
        stage: &str,
        format: MatchFormat,
        home_team_id: u64,
        away_team_id: u64,
        home_power: f64,
        away_power: f64,
    ) -> MatchResult {
        let wins_needed = format.wins_needed();
        let mut home_score: u8 = 0;
        let mut away_score: u8 = 0;
        let mut games = Vec::new();
        let mut game_number: u8 = 1;

        // 模拟每一局直到决出胜负
        while home_score < wins_needed && away_score < wins_needed {
            let (home_perf, away_perf, winner_id) =
                self.simulate_game(home_power, away_power, home_team_id, away_team_id);

            let game = MatchGame {
                id: 0, // 由数据库生成
                match_id,
                game_number,
                home_power,
                away_power,
                home_base_power: None,
                away_base_power: None,
                home_synergy_bonus: None,
                away_synergy_bonus: None,
                home_bp_bonus: None,
                away_bp_bonus: None,
                home_version_bonus: None,
                away_version_bonus: None,
                home_performance: home_perf,
                away_performance: away_perf,
                winner_id,
                duration_minutes: Some(30 + rand::random::<u32>() % 20), // 30-50分钟
            };

            games.push(game);

            if winner_id == home_team_id {
                home_score += 1;
            } else {
                away_score += 1;
            }

            game_number += 1;
        }

        let winner_id = if home_score > away_score {
            home_team_id
        } else {
            away_team_id
        };

        let match_info = Match {
            id: match_id,
            tournament_id,
            stage: stage.to_string(),
            round: None,
            match_order: None,
            format,
            home_team_id,
            away_team_id,
            home_score,
            away_score,
            winner_id: Some(winner_id),
            status: MatchStatus::Completed,
        };

        MatchResult {
            match_info,
            games,
            winner_id,
            home_score,
            away_score,
        }
    }

    /// 计算胜率 (基于战力值差距)
    /// 用于显示预测胜率
    pub fn calculate_win_probability(&self, team_power: f64, opponent_power: f64) -> f64 {
        let diff = team_power - opponent_power;
        // 使用简化的正态分布CDF近似
        let z = diff / (self.std_dev * std::f64::consts::SQRT_2);
        0.5 * (1.0 + erf_approx(z))
    }

    /// 特性感知的BO系列赛模拟
    ///
    /// 在每局比赛中根据当前局数、比分差、赛事类型构建 TraitContext，
    /// 对每位选手应用特性修正后计算队伍战力，再用正态分布采样决定胜负。
    pub fn simulate_match_with_traits(
        &self,
        match_id: u64,
        tournament_id: u64,
        stage: &str,
        format: MatchFormat,
        home_team_id: u64,
        away_team_id: u64,
        home_players: &[MatchPlayerInfo],
        away_players: &[MatchPlayerInfo],
        home_bench: &[MatchPlayerInfo],
        away_bench: &[MatchPlayerInfo],
        sim_ctx: &MatchSimContext,
        meta_weights: &MetaWeights,
        home_personality: &AITeamPersonality,
        away_personality: &AITeamPersonality,
        current_season: u32,
    ) -> MatchResult {
        self.last_games_played.lock().unwrap().clear();

        if home_players.is_empty() || away_players.is_empty() {
            return self.simulate_match(
                match_id,
                tournament_id,
                stage,
                format,
                home_team_id,
                away_team_id,
                70.0,
                70.0,
            );
        }

        let mut current_home: Vec<MatchPlayerInfo> = home_players.to_vec();
        let mut current_away: Vec<MatchPlayerInfo> = away_players.to_vec();
        let mut home_bench_mut: Vec<MatchPlayerInfo> = home_bench.to_vec();
        let mut away_bench_mut: Vec<MatchPlayerInfo> = away_bench.to_vec();

        let bo_count: u8 = match format {
            MatchFormat::Bo1 => 1,
            MatchFormat::Bo3 => 3,
            MatchFormat::Bo5 => 5,
        };

        let wins_needed = format.wins_needed();
        let mut home_score: u8 = 0;
        let mut away_score: u8 = 0;
        let mut games = Vec::new();
        let mut game_number: u8 = 1;
        let mut games_played_series: HashMap<u64, u8> = HashMap::new();

        while home_score < wins_needed && away_score < wins_needed {
            let home_has_leader = current_home
                .iter()
                .any(|p| p.traits.contains(&TraitType::TeamLeader));
            let away_has_leader = current_away
                .iter()
                .any(|p| p.traits.contains(&TraitType::TeamLeader));

            let score_diff_home = home_score as i8 - away_score as i8;

            let (home_power, home_breakdown) = self.calculate_trait_adjusted_power(
                &current_home,
                game_number,
                score_diff_home,
                sim_ctx,
                meta_weights,
                home_has_leader,
            );
            let (away_power, away_breakdown) = self.calculate_trait_adjusted_power(
                &current_away,
                game_number,
                -score_diff_home,
                sim_ctx,
                meta_weights,
                away_has_leader,
            );

            let (home_perf, away_perf, winner_id) =
                self.simulate_game(home_power, away_power, home_team_id, away_team_id);

            let game = MatchGame {
                id: 0,
                match_id,
                game_number,
                home_power: home_breakdown.meta_power,
                away_power: away_breakdown.meta_power,
                home_base_power: Some(home_breakdown.base_power),
                away_base_power: Some(away_breakdown.base_power),
                home_synergy_bonus: Some(home_breakdown.synergy_bonus),
                away_synergy_bonus: Some(away_breakdown.synergy_bonus),
                home_bp_bonus: Some(home_breakdown.bp_bonus),
                away_bp_bonus: Some(away_breakdown.bp_bonus),
                home_version_bonus: Some(home_breakdown.version_bonus),
                away_version_bonus: Some(away_breakdown.version_bonus),
                home_performance: home_perf,
                away_performance: away_perf,
                winner_id,
                duration_minutes: Some(30 + rand::random::<u32>() % 20),
            };

            games.push(game);

            for p in current_home.iter().chain(current_away.iter()) {
                *games_played_series.entry(p.player_id).or_insert(0) += 1;
            }

            if winner_id == home_team_id {
                home_score += 1;
            } else {
                away_score += 1;
            }

            // 局间换人（BO系列赛且比赛未结束）
            if bo_count > 1 && home_score < wins_needed && away_score < wins_needed {
                Self::try_substitution(
                    &mut current_home,
                    &mut home_bench_mut,
                    &sim_ctx.tournament_type,
                    stage,
                    bo_count,
                    game_number,
                    home_score,
                    away_score,
                    true,
                    home_personality,
                    current_season,
                    &games_played_series,
                );
                Self::try_substitution(
                    &mut current_away,
                    &mut away_bench_mut,
                    &sim_ctx.tournament_type,
                    stage,
                    bo_count,
                    game_number,
                    home_score,
                    away_score,
                    false,
                    away_personality,
                    current_season,
                    &games_played_series,
                );
            }

            game_number += 1;
        }

        *self.last_games_played.lock().unwrap() = games_played_series;

        let winner_id = if home_score > away_score {
            home_team_id
        } else {
            away_team_id
        };

        let match_info = Match {
            id: match_id,
            tournament_id,
            stage: stage.to_string(),
            round: None,
            match_order: None,
            format,
            home_team_id,
            away_team_id,
            home_score,
            away_score,
            winner_id: Some(winner_id),
            status: MatchStatus::Completed,
        };

        MatchResult {
            match_info,
            games,
            winner_id,
            home_score,
            away_score,
        }
    }

    fn try_substitution(
        lineup: &mut Vec<MatchPlayerInfo>,
        bench: &mut Vec<MatchPlayerInfo>,
        tournament_type: &str,
        stage: &str,
        bo_count: u8,
        game_number: u8,
        home_score: u8,
        away_score: u8,
        is_home: bool,
        personality: &AITeamPersonality,
        current_season: u32,
        games_played_series: &HashMap<u64, u8>,
    ) {
        if bench.is_empty() {
            return;
        }

        let sub_ctx = SubstitutionContext {
            tournament_type: tournament_type.to_string(),
            round: stage.to_string(),
            bo_count,
            game_number,
            home_score,
            away_score,
            is_home,
            current_season,
        };

        let starter_candidates: Vec<LineupCandidate> = lineup
            .iter()
            .map(|p| Self::to_candidate(p, current_season))
            .collect();
        let bench_candidates: Vec<LineupCandidate> = bench
            .iter()
            .map(|p| Self::to_candidate(p, current_season))
            .collect();

        let decisions = LineupEngine::check_substitutions(
            &starter_candidates,
            &bench_candidates,
            &sub_ctx,
            personality,
            current_season,
            games_played_series,
        );

        for decision in &decisions {
            let sub_out_idx = lineup
                .iter()
                .position(|p| p.player_id == decision.sub_out_player_id);
            let sub_in_idx = bench
                .iter()
                .position(|p| p.player_id == decision.sub_in_player_id);

            if let (Some(out_idx), Some(in_idx)) = (sub_out_idx, sub_in_idx) {
                let mut sub_in = bench.remove(in_idx);
                sub_in.is_starter = true;
                let mut sub_out = lineup[out_idx].clone();
                sub_out.is_starter = false;
                lineup[out_idx] = sub_in;
                bench.push(sub_out);
            }
        }
    }

    fn to_candidate(p: &MatchPlayerInfo, _current_season: u32) -> LineupCandidate {
        let position = match p.position.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };
        LineupCandidate {
            player_id: p.player_id,
            game_id: format!("P{}", p.player_id),
            position,
            ability: p.ability,
            age: p.age,
            potential: p.potential,
            condition: p.condition,
            form_factors: p.form_factors.clone().unwrap_or(PlayerFormFactors {
                player_id: p.player_id,
                form_cycle: 50.0,
                momentum: 0,
                last_performance: 0.0,
                last_match_won: false,
                perf_history: String::new(),
                games_since_rest: 0,
            }),
            is_starter: p.is_starter,
            join_season: p.join_season as u32,
            traits: p.traits.clone(),
            satisfaction: p.satisfaction,
            champion_version_score: p.champion_version_score,
        }
    }

    /// 计算特性修正后的队伍战力
    ///
    /// 对每位选手：构建 TraitContext → 计算特性修正 → 应用修正 →
    /// TeamLeader 队友加成 → 稳定性噪声 → 钳位 → Meta 加权
    fn calculate_trait_adjusted_power(
        &self,
        players: &[MatchPlayerInfo],
        game_number: u8,
        score_diff: i8,
        sim_ctx: &MatchSimContext,
        meta_weights: &MetaWeights,
        team_leader_bonus: bool,
    ) -> (f64, TeamPowerBreakdown) {
        let mut rng = rand::thread_rng();
        let mut player_abilities: Vec<(f64, &str)> = Vec::with_capacity(players.len());
        let mut raw_modified_abilities: Vec<f64> = Vec::with_capacity(players.len());
        let mut total_bp_bonus = 0.0;
        let mut total_version_bonus = 0.0;

        for player in players {
            // 构建选手专属的 TraitContext
            let trait_ctx = TraitContext {
                tournament_type: sim_ctx.tournament_type.clone(),
                is_playoff: sim_ctx.is_playoff,
                is_international: sim_ctx.is_international,
                game_number,
                score_diff,
                age: player.age,
                is_first_season: player.is_first_season,
                games_since_rest: 0,
            };

            // 计算特性综合修正
            let modifiers = TraitEngine::calculate_combined_modifiers(&player.traits, &trait_ctx);

            // 应用修正到基础属性
            let (modified_ability, modified_stability, mut modified_condition, ability_ceiling) =
                TraitEngine::apply_modifiers(
                    player.ability,
                    player.stability,
                    player.condition,
                    &modifiers,
                );
            raw_modified_abilities.push(modified_ability as f64);
            total_bp_bonus += player.bp_modifier;
            total_version_bonus += player.champion_version_score;

            // TeamLeader 队友加成：condition +1（对非 TeamLeader 本人）
            if team_leader_bonus && !player.traits.contains(&TraitType::TeamLeader) {
                modified_condition = (modified_condition as i16 + 1).clamp(-10, 10) as i8;
            }

            // 稳定性噪声: N(0, (100 - modified_stability) / 10)
            let sigma = (100.0 - modified_stability as f64) / 10.0;
            let normal = Normal::new(0.0, sigma.max(0.1)).unwrap();
            let noise: f64 = normal.sample(&mut rng);

            // 实际能力 = modified_ability + modified_condition + noise
            let raw_ability =
                modified_ability as f64 + player.bp_modifier + modified_condition as f64 + noise;

            // 钳位到 [ability - 15, ability_ceiling]
            let min_ability = (modified_ability as f64 - 15.0).max(0.0);
            let max_ability = (ability_ceiling as f64).min(100.0);
            let actual_ability = raw_ability.clamp(min_ability, max_ability);

            player_abilities.push((actual_ability, player.position.as_str()));
        }

        // 使用 Meta 加权计算队伍战力
        let meta_power = MetaEngine::calculate_team_power_weighted(&player_abilities, meta_weights);
        let player_count = players.len() as f64;
        let breakdown = if player_count > 0.0 {
            TeamPowerBreakdown {
                base_power: raw_modified_abilities.iter().sum::<f64>() / player_count,
                synergy_bonus: 0.0,
                bp_bonus: total_bp_bonus / player_count,
                version_bonus: total_version_bonus / player_count,
                meta_power,
            }
        } else {
            TeamPowerBreakdown {
                meta_power,
                ..TeamPowerBreakdown::default()
            }
        };

        (meta_power, breakdown)
    }
}

/// 误差函数的近似计算
fn erf_approx(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_game() {
        let engine = MatchSimulationEngine::default();

        // 运行多次测试
        let mut home_wins = 0;
        let mut away_wins = 0;

        for _ in 0..1000 {
            let (_, _, winner) = engine.simulate_game(80.0, 70.0, 1, 2);
            if winner == 1 {
                home_wins += 1;
            } else {
                away_wins += 1;
            }
        }

        // 战力值高的队伍应该赢得更多
        assert!(home_wins > away_wins);
    }

    #[test]
    fn test_simulate_game_equal_power() {
        let engine = MatchSimulationEngine::default();

        let mut home_wins = 0;
        let mut away_wins = 0;

        for _ in 0..1000 {
            let (_, _, winner) = engine.simulate_game(75.0, 75.0, 1, 2);
            if winner == 1 {
                home_wins += 1;
            } else {
                away_wins += 1;
            }
        }

        // 战力相等时应该接近50/50
        let home_rate = home_wins as f64 / 1000.0;
        assert!(home_rate > 0.4 && home_rate < 0.6);
    }

    #[test]
    fn test_simulate_game_large_power_gap() {
        let engine = MatchSimulationEngine::default();

        let mut strong_wins = 0;

        for _ in 0..100 {
            let (_, _, winner) = engine.simulate_game(95.0, 60.0, 1, 2);
            if winner == 1 {
                strong_wins += 1;
            }
        }

        // 强队应该赢得绝大多数比赛
        assert!(strong_wins > 85);
    }

    #[test]
    fn test_simulate_match_bo1() {
        let engine = MatchSimulationEngine::default();
        let result = engine.simulate_match(1, 1, "GROUP", MatchFormat::Bo1, 1, 2, 75.0, 75.0);

        // BO1只有1局
        assert_eq!(result.games.len(), 1);
        // 比分应该是1:0
        assert!(
            (result.home_score == 1 && result.away_score == 0)
                || (result.home_score == 0 && result.away_score == 1)
        );
    }

    #[test]
    fn test_simulate_match_bo3() {
        let engine = MatchSimulationEngine::default();
        let result = engine.simulate_match(1, 1, "GROUP", MatchFormat::Bo3, 1, 2, 75.0, 75.0);

        // BO3应该有2-3局
        assert!(result.games.len() >= 2 && result.games.len() <= 3);
        // 胜者应该赢2局
        assert!(result.home_score == 2 || result.away_score == 2);
        // 败者最多1局
        assert!(result.home_score.min(result.away_score) <= 1);
    }

    #[test]
    fn test_simulate_match_bo5() {
        let engine = MatchSimulationEngine::default();
        let result = engine.simulate_match(1, 1, "FINAL", MatchFormat::Bo5, 1, 2, 75.0, 75.0);

        // BO5应该有3-5局
        assert!(result.games.len() >= 3 && result.games.len() <= 5);
        // 胜者应该赢3局
        assert!(result.home_score == 3 || result.away_score == 3);
        // 败者最多2局
        assert!(result.home_score.min(result.away_score) <= 2);
    }

    #[test]
    fn test_win_probability() {
        let engine = MatchSimulationEngine::default();

        // 战力相等时胜率应该接近50%
        let prob_equal = engine.calculate_win_probability(70.0, 70.0);
        assert!((prob_equal - 0.5).abs() < 0.01);

        // 战力高时胜率应该更高
        let prob_higher = engine.calculate_win_probability(80.0, 70.0);
        assert!(prob_higher > 0.5);

        // 战力低时胜率应该更低
        let prob_lower = engine.calculate_win_probability(60.0, 70.0);
        assert!(prob_lower < 0.5);
    }

    #[test]
    fn test_win_probability_symmetry() {
        let engine = MatchSimulationEngine::default();

        let prob_a = engine.calculate_win_probability(80.0, 70.0);
        let prob_b = engine.calculate_win_probability(70.0, 80.0);

        // 对称性: A vs B 的胜率 + B vs A 的胜率 = 1
        assert!((prob_a + prob_b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_custom_std_dev() {
        let engine_low_variance = MatchSimulationEngine::new(3.0);
        let engine_high_variance = MatchSimulationEngine::new(10.0);

        // 低方差时强队优势更明显
        let mut low_var_wins = 0;
        let mut high_var_wins = 0;

        for _ in 0..500 {
            let (_, _, winner) = engine_low_variance.simulate_game(80.0, 70.0, 1, 2);
            if winner == 1 {
                low_var_wins += 1;
            }

            let (_, _, winner) = engine_high_variance.simulate_game(80.0, 70.0, 1, 2);
            if winner == 1 {
                high_var_wins += 1;
            }
        }

        // 低方差引擎中强队胜率应该更高
        assert!(low_var_wins >= high_var_wins - 50);
    }

    #[test]
    fn test_match_result_consistency() {
        let engine = MatchSimulationEngine::default();
        let result = engine.simulate_match(1, 1, "PLAYOFF", MatchFormat::Bo5, 1, 2, 80.0, 75.0);

        // 验证结果一致性
        assert_eq!(result.match_info.home_score, result.home_score);
        assert_eq!(result.match_info.away_score, result.away_score);
        assert_eq!(result.match_info.winner_id, Some(result.winner_id));
        assert_eq!(result.match_info.status, MatchStatus::Completed);

        // 验证小局数量
        let game_wins: u8 = result
            .games
            .iter()
            .filter(|g| g.winner_id == result.winner_id)
            .count() as u8;
        let winner_score = if result.winner_id == 1 {
            result.home_score
        } else {
            result.away_score
        };
        assert_eq!(game_wins, winner_score);
    }

    // ===== 特性感知模拟测试 =====

    fn make_player(ability: u8, position: &str, traits: Vec<TraitType>) -> MatchPlayerInfo {
        MatchPlayerInfo {
            player_id: 0,
            ability,
            stability: 70,
            condition: 0,
            age: 24,
            position: position.to_string(),
            traits,
            is_first_season: false,
            is_starter: true,
            join_season: 1,
            potential: ability,
            satisfaction: 60,
            form_factors: None,
            bp_modifier: 0.0,
            champion_version_score: 0.0,
        }
    }

    fn make_team(ability: u8, traits: Vec<TraitType>) -> Vec<MatchPlayerInfo> {
        vec![
            make_player(ability, "TOP", traits.clone()),
            make_player(ability, "JUG", traits.clone()),
            make_player(ability, "MID", traits.clone()),
            make_player(ability, "ADC", traits.clone()),
            make_player(ability, "SUP", traits.clone()),
        ]
    }

    #[test]
    fn test_simulate_match_with_traits_bo3() {
        let engine = MatchSimulationEngine::default();
        let home = make_team(75, vec![]);
        let away = make_team(75, vec![]);
        let ctx = MatchSimContext {
            is_playoff: false,
            is_international: false,
            tournament_type: "league".to_string(),
        };
        let weights = MetaWeights::balanced();

        let result = engine.simulate_match_with_traits(
            1,
            1,
            "GROUP",
            MatchFormat::Bo3,
            1,
            2,
            &home,
            &away,
            &[],
            &[],
            &ctx,
            &weights,
            &AITeamPersonality::Balanced,
            &AITeamPersonality::Balanced,
            1,
        );

        assert!(result.games.len() >= 2 && result.games.len() <= 3);
        assert!(result.home_score == 2 || result.away_score == 2);
    }

    #[test]
    fn test_simulate_match_with_traits_bo5() {
        let engine = MatchSimulationEngine::default();
        let home = make_team(80, vec![]);
        let away = make_team(80, vec![]);
        let ctx = MatchSimContext {
            is_playoff: true,
            is_international: false,
            tournament_type: "league".to_string(),
        };
        let weights = MetaWeights::balanced();

        let result = engine.simulate_match_with_traits(
            1,
            1,
            "PLAYOFF",
            MatchFormat::Bo5,
            1,
            2,
            &home,
            &away,
            &[],
            &[],
            &ctx,
            &weights,
            &AITeamPersonality::Balanced,
            &AITeamPersonality::Balanced,
            1,
        );

        assert!(result.games.len() >= 3 && result.games.len() <= 5);
        assert!(result.home_score == 3 || result.away_score == 3);
    }

    #[test]
    fn test_clutch_trait_helps_in_playoff() {
        let engine = MatchSimulationEngine::default();
        let home = make_team(75, vec![TraitType::Clutch]);
        let away = make_team(75, vec![]);
        let ctx = MatchSimContext {
            is_playoff: true,
            is_international: false,
            tournament_type: "league".to_string(),
        };
        let weights = MetaWeights::balanced();

        let mut home_wins = 0;
        for _ in 0..500 {
            let result = engine.simulate_match_with_traits(
                1,
                1,
                "PLAYOFF",
                MatchFormat::Bo3,
                1,
                2,
                &home,
                &away,
                &[],
                &[],
                &ctx,
                &weights,
                &AITeamPersonality::Balanced,
                &AITeamPersonality::Balanced,
                1,
            );
            if result.winner_id == 1 {
                home_wins += 1;
            }
        }

        // Clutch 在季后赛应提升胜率，主队应赢得超过 50%
        assert!(
            home_wins > 250,
            "Clutch 特性在季后赛中应有提升，实际胜场: {}/500",
            home_wins
        );
    }

    #[test]
    fn test_trait_adjusted_power_with_team_leader() {
        let engine = MatchSimulationEngine::default();
        let mut team_with_leader = make_team(75, vec![]);
        team_with_leader[2].traits = vec![TraitType::TeamLeader]; // 中单是团队核心

        let team_without = make_team(75, vec![]);
        let ctx = MatchSimContext {
            is_playoff: false,
            is_international: false,
            tournament_type: "league".to_string(),
        };
        let weights = MetaWeights::balanced();

        let mut leader_wins = 0;
        for _ in 0..500 {
            let result = engine.simulate_match_with_traits(
                1,
                1,
                "GROUP",
                MatchFormat::Bo3,
                1,
                2,
                &team_with_leader,
                &team_without,
                &[],
                &[],
                &ctx,
                &weights,
                &AITeamPersonality::Balanced,
                &AITeamPersonality::Balanced,
                1,
            );
            if result.winner_id == 1 {
                leader_wins += 1;
            }
        }

        assert!(
            leader_wins > 230,
            "TeamLeader 应有微弱提升，实际胜场: {}/500",
            leader_wins
        );
    }
}
