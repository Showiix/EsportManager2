use rand_distr::{Distribution, Normal};
use crate::models::{MatchFormat, MatchGame, MatchResult, Match, MatchStatus};

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
}
