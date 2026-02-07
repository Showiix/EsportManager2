use rand_distr::{Distribution, Normal};
use crate::models::{MatchFormat, MatchGame, MatchResult, Match, MatchStatus};
use super::traits::{TraitType, TraitContext, TraitEngine};
use super::meta_engine::{MetaWeights, MetaEngine};

/// 比赛用选手信息（用于快速模拟路径的特性感知）
#[derive(Debug, Clone)]
pub struct MatchPlayerInfo {
    pub ability: u8,
    pub stability: u8,
    pub condition: i8,
    pub age: u8,
    pub position: String,
    pub traits: Vec<TraitType>,
    pub is_first_season: bool,
}

/// 比赛情境信息（用于快速模拟路径的特性触发判断）
#[derive(Debug, Clone)]
pub struct MatchSimContext {
    pub is_playoff: bool,
    pub is_international: bool,
    pub tournament_type: String,
}

/// 比赛模拟引擎 - 基于正态分布的胜负判定
pub struct MatchSimulationEngine {
    /// 标准差 (控制发挥波动程度)
    std_dev: f64,
}

impl Default for MatchSimulationEngine {
    fn default() -> Self {
        Self { std_dev: 6.0 }
    }
}

impl MatchSimulationEngine {
    pub fn new(std_dev: f64) -> Self {
        Self { std_dev }
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
        sim_ctx: &MatchSimContext,
        meta_weights: &MetaWeights,
    ) -> MatchResult {
        // 如果任一方没有选手数据，回退到默认战力模拟
        if home_players.is_empty() || away_players.is_empty() {
            return self.simulate_match(
                match_id, tournament_id, stage, format,
                home_team_id, away_team_id, 70.0, 70.0,
            );
        }

        // 检测 TeamLeader 特性：如果队内有人拥有 TeamLeader，其余队友 condition +1
        let home_has_leader = home_players.iter().any(|p| p.traits.contains(&TraitType::TeamLeader));
        let away_has_leader = away_players.iter().any(|p| p.traits.contains(&TraitType::TeamLeader));

        let wins_needed = format.wins_needed();
        let mut home_score: u8 = 0;
        let mut away_score: u8 = 0;
        let mut games = Vec::new();
        let mut game_number: u8 = 1;

        while home_score < wins_needed && away_score < wins_needed {
            let score_diff_home = home_score as i8 - away_score as i8;

            // 计算双方特性修正后的队伍战力
            let home_power = self.calculate_trait_adjusted_power(
                home_players, game_number, score_diff_home,
                sim_ctx, meta_weights, home_has_leader,
            );
            let away_power = self.calculate_trait_adjusted_power(
                away_players, game_number, -score_diff_home,
                sim_ctx, meta_weights, away_has_leader,
            );

            // 使用已有的正态分布采样决定胜负
            let (home_perf, away_perf, winner_id) =
                self.simulate_game(home_power, away_power, home_team_id, away_team_id);

            let game = MatchGame {
                id: 0,
                match_id,
                game_number,
                home_power,
                away_power,
                home_performance: home_perf,
                away_performance: away_perf,
                winner_id,
                duration_minutes: Some(30 + rand::random::<u32>() % 20),
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
    ) -> f64 {
        let mut rng = rand::thread_rng();
        let mut player_abilities: Vec<(f64, &str)> = Vec::with_capacity(players.len());

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

            // TeamLeader 队友加成：condition +1（对非 TeamLeader 本人）
            if team_leader_bonus && !player.traits.contains(&TraitType::TeamLeader) {
                modified_condition = (modified_condition as i16 + 1).clamp(-10, 10) as i8;
            }

            // 稳定性噪声: N(0, (100 - modified_stability) / 10)
            let sigma = (100.0 - modified_stability as f64) / 10.0;
            let normal = Normal::new(0.0, sigma.max(0.1)).unwrap();
            let noise: f64 = normal.sample(&mut rng);

            // 实际能力 = modified_ability + modified_condition + noise
            let raw_ability = modified_ability as f64 + modified_condition as f64 + noise;

            // 钳位到 [ability - 15, ability_ceiling]
            let min_ability = (modified_ability as f64 - 15.0).max(0.0);
            let max_ability = (ability_ceiling as f64).min(100.0);
            let actual_ability = raw_ability.clamp(min_ability, max_ability);

            player_abilities.push((actual_ability, player.position.as_str()));
        }

        // 使用 Meta 加权计算队伍战力
        MetaEngine::calculate_team_power_weighted(&player_abilities, meta_weights)
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
        let result = engine.simulate_match(
            1, 1, "GROUP", MatchFormat::Bo1, 1, 2, 75.0, 75.0
        );

        // BO1只有1局
        assert_eq!(result.games.len(), 1);
        // 比分应该是1:0
        assert!((result.home_score == 1 && result.away_score == 0) ||
                (result.home_score == 0 && result.away_score == 1));
    }

    #[test]
    fn test_simulate_match_bo3() {
        let engine = MatchSimulationEngine::default();
        let result = engine.simulate_match(
            1, 1, "GROUP", MatchFormat::Bo3, 1, 2, 75.0, 75.0
        );

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
        let result = engine.simulate_match(
            1, 1, "FINAL", MatchFormat::Bo5, 1, 2, 75.0, 75.0
        );

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
        let result = engine.simulate_match(
            1, 1, "PLAYOFF", MatchFormat::Bo5, 1, 2, 80.0, 75.0
        );

        // 验证结果一致性
        assert_eq!(result.match_info.home_score, result.home_score);
        assert_eq!(result.match_info.away_score, result.away_score);
        assert_eq!(result.match_info.winner_id, Some(result.winner_id));
        assert_eq!(result.match_info.status, MatchStatus::Completed);

        // 验证小局数量
        let game_wins: u8 = result.games.iter()
            .filter(|g| g.winner_id == result.winner_id)
            .count() as u8;
        let winner_score = if result.winner_id == 1 { result.home_score } else { result.away_score };
        assert_eq!(game_wins, winner_score);
    }

    // ===== 特性感知模拟测试 =====

    fn make_player(ability: u8, position: &str, traits: Vec<TraitType>) -> MatchPlayerInfo {
        MatchPlayerInfo {
            ability,
            stability: 70,
            condition: 0,
            age: 24,
            position: position.to_string(),
            traits,
            is_first_season: false,
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
            1, 1, "GROUP", MatchFormat::Bo3, 1, 2,
            &home, &away, &ctx, &weights,
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
            1, 1, "PLAYOFF", MatchFormat::Bo5, 1, 2,
            &home, &away, &ctx, &weights,
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
                1, 1, "PLAYOFF", MatchFormat::Bo3, 1, 2,
                &home, &away, &ctx, &weights,
            );
            if result.winner_id == 1 {
                home_wins += 1;
            }
        }

        // Clutch 在季后赛应提升胜率，主队应赢得超过 50%
        assert!(home_wins > 250, "Clutch 特性在季后赛中应有提升，实际胜场: {}/500", home_wins);
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
                1, 1, "GROUP", MatchFormat::Bo3, 1, 2,
                &team_with_leader, &team_without, &ctx, &weights,
            );
            if result.winner_id == 1 {
                leader_wins += 1;
            }
        }

        assert!(leader_wins > 230, "TeamLeader 应有微弱提升，实际胜场: {}/500", leader_wins);
    }
}
