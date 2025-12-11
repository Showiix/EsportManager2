//! 战力引擎 - 基于5名选手计算队伍战力并模拟比赛
//!
//! 算法（策划案规定）：
//! - 队伍战力 = Σ(5名选手实际能力) / 5
//! - 发挥值 = 正态分布(队伍战力, σ=6)
//! - 发挥值高者获胜

use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 选手位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerPosition {
    TOP,
    JUG,
    MID,
    ADC,
    SUP,
}

impl std::fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerPosition::TOP => write!(f, "TOP"),
            PlayerPosition::JUG => write!(f, "JUG"),
            PlayerPosition::MID => write!(f, "MID"),
            PlayerPosition::ADC => write!(f, "ADC"),
            PlayerPosition::SUP => write!(f, "SUP"),
        }
    }
}

/// 选手数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub game_id: String,        // 游戏ID（如 "Faker"）
    pub name: String,           // 真实姓名
    pub team_id: String,
    pub team_name: Option<String>,
    pub position: PlayerPosition,
    pub region_id: String,
    pub region_name: Option<String>,
    pub ability: u8,            // 0-100
    pub potential: u8,          // 0-100
    pub stability: u8,          // 0-100
    pub condition: i8,          // -10 ~ +10
    pub age: u8,
    pub tag: String,            // GENIUS/NORMAL/ORDINARY
}

/// 选手单场发挥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPerformance {
    pub player_id: String,
    pub player_name: String,
    pub position: PlayerPosition,
    pub team_id: String,
    pub base_ability: u8,
    pub condition_bonus: i8,
    pub stability_noise: f64,
    pub actual_ability: f64,
    pub impact_score: f64,      // actualAbility - teamAverage
}

/// 单局详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetail {
    pub game_number: u8,
    pub team_a_id: String,
    pub team_a_name: String,
    pub team_a_power: f64,
    pub team_a_performance: f64,
    pub team_a_players: Vec<PlayerPerformance>,
    pub team_b_id: String,
    pub team_b_name: String,
    pub team_b_power: f64,
    pub team_b_performance: f64,
    pub team_b_players: Vec<PlayerPerformance>,
    pub winner_id: String,
    pub winner_name: String,
    pub power_difference: f64,
    pub performance_difference: f64,
    pub is_upset: bool,
}

/// 关键选手信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPlayer {
    pub player_id: String,
    pub player_name: String,
    pub team_id: String,
    pub reason: String,         // "高发挥" | "低发挥"
    pub impact_score: f64,
    pub game_number: u8,
}

/// MVP 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvpInfo {
    pub player_id: String,
    pub player_name: String,
    pub team_id: String,
    pub total_impact: f64,
}

/// 完整比赛详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDetail {
    pub match_id: String,
    pub team_a_id: String,
    pub team_a_name: String,
    pub team_b_id: String,
    pub team_b_name: String,
    pub best_of: u8,
    pub games: Vec<GameDetail>,
    pub final_score_a: u8,
    pub final_score_b: u8,
    pub winner_id: String,
    pub winner_name: String,
    pub mvp: Option<MvpInfo>,
    pub key_player: Option<KeyPlayer>,
    pub played_at: DateTime<Utc>,
    pub tournament_type: Option<String>,
    pub season_id: Option<String>,
}

/// 选手引擎 - 计算每位选手每局的实际发挥
pub struct PlayerEngine;

impl PlayerEngine {
    /// Box-Muller 变换生成标准正态分布随机数
    pub fn gaussian_random() -> f64 {
        let mut rng = rand::thread_rng();
        let mut u: f64 = 0.0;
        let mut v: f64 = 0.0;

        while u == 0.0 {
            u = rng.gen();
        }
        while v == 0.0 {
            v = rng.gen();
        }

        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    /// 计算稳定性对应的标准差
    /// σ = (100 - stability) / 10
    pub fn calculate_stability_sigma(stability: u8) -> f64 {
        (100.0 - stability.min(100) as f64) / 10.0
    }

    /// 计算单个选手在一局比赛中的实际发挥
    pub fn calculate_performance(player: &Player) -> PlayerPerformance {
        // 1. 计算稳定性标准差
        let sigma = Self::calculate_stability_sigma(player.stability);

        // 2. 生成高斯噪声
        let noise = Self::gaussian_random() * sigma;

        // 3. 计算原始实际能力
        let raw_ability = player.ability as f64 + player.condition as f64 + noise;

        // 4. 钳位到合理范围 [ability - 15, ability + 10]
        let min_ability = (player.ability as f64 - 15.0).max(0.0);
        let max_ability = (player.ability as f64 + 10.0).min(100.0);
        let actual_ability = raw_ability.clamp(min_ability, max_ability);

        PlayerPerformance {
            player_id: player.id.clone(),
            player_name: player.game_id.clone(),
            position: player.position,
            team_id: player.team_id.clone(),
            base_ability: player.ability,
            condition_bonus: player.condition,
            stability_noise: (noise * 100.0).round() / 100.0,
            actual_ability: (actual_ability * 10.0).round() / 10.0,
            impact_score: 0.0, // 稍后由 PowerEngine 计算
        }
    }

    /// 批量计算队伍选手的发挥
    pub fn calculate_team_performances(players: &[Player]) -> Vec<PlayerPerformance> {
        players.iter().map(|p| Self::calculate_performance(p)).collect()
    }

    /// 根据年龄获取稳定性基础值
    pub fn get_base_stability_by_age(age: u8) -> u8 {
        match age {
            0..=17 => 55,
            18..=24 => 60 + ((age - 18) as f64 * 2.5) as u8, // 60-75
            25..=29 => 75 + (age - 25) * 2,                   // 75-85
            30..=36 => 85 + ((age - 30) as f64 * 1.5) as u8, // 85-95
            _ => 95,
        }
    }

    /// 根据年龄获取状态加成范围
    pub fn get_condition_range(age: u8) -> (i8, i8) {
        match age {
            0..=24 => (-5, 8),
            25..=29 => (-3, 3),
            _ => (0, 2),
        }
    }

    /// 随机生成选手当前状态
    pub fn generate_random_condition(age: u8) -> i8 {
        let (min, max) = Self::get_condition_range(age);
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }
}

/// 战力引擎 - 模拟比赛
pub struct PowerEngine;

impl PowerEngine {
    /// 队伍发挥的标准差（策划案规定）
    const PERFORMANCE_SIGMA: f64 = 6.0;

    /// 计算队伍战力（基于5名选手的实际发挥）
    pub fn calculate_team_power(performances: &[PlayerPerformance]) -> f64 {
        if performances.is_empty() {
            return 0.0;
        }
        let total: f64 = performances.iter().map(|p| p.actual_ability).sum();
        total / performances.len() as f64
    }

    /// 计算队伍发挥值（正态分布）
    pub fn calculate_team_performance_value(team_power: f64) -> f64 {
        let normal = Normal::new(team_power, Self::PERFORMANCE_SIGMA).unwrap();
        let mut rng = rand::thread_rng();
        normal.sample(&mut rng)
    }

    /// 计算选手影响力分数
    /// 影响力 = 选手实际能力 - 队伍平均能力
    pub fn calculate_impact_scores(performances: &mut [PlayerPerformance], team_average: f64) {
        for p in performances.iter_mut() {
            p.impact_score = ((p.actual_ability - team_average) * 10.0).round() / 10.0;
        }
    }

    /// 模拟单局比赛
    pub fn simulate_game(
        team_a_id: &str,
        team_a_name: &str,
        team_a_players: &[Player],
        team_b_id: &str,
        team_b_name: &str,
        team_b_players: &[Player],
        game_number: u8,
    ) -> GameDetail {
        // 1. 计算每位选手的实际发挥
        let mut team_a_performances = PlayerEngine::calculate_team_performances(team_a_players);
        let mut team_b_performances = PlayerEngine::calculate_team_performances(team_b_players);

        // 2. 计算队伍战力
        let team_a_power = Self::calculate_team_power(&team_a_performances);
        let team_b_power = Self::calculate_team_power(&team_b_performances);

        // 3. 计算影响力分数
        Self::calculate_impact_scores(&mut team_a_performances, team_a_power);
        Self::calculate_impact_scores(&mut team_b_performances, team_b_power);

        // 4. 计算队伍发挥值
        let team_a_performance_value = Self::calculate_team_performance_value(team_a_power);
        let team_b_performance_value = Self::calculate_team_performance_value(team_b_power);

        // 5. 决定胜负（发挥值高者获胜）
        let team_a_wins = team_a_performance_value > team_b_performance_value;
        let (winner_id, winner_name) = if team_a_wins {
            (team_a_id.to_string(), team_a_name.to_string())
        } else {
            (team_b_id.to_string(), team_b_name.to_string())
        };

        // 6. 判断是否爆冷（战力低的队伍赢了）
        let is_upset = (team_a_power > team_b_power && !team_a_wins)
            || (team_b_power > team_a_power && team_a_wins);

        GameDetail {
            game_number,
            team_a_id: team_a_id.to_string(),
            team_a_name: team_a_name.to_string(),
            team_a_power: (team_a_power * 10.0).round() / 10.0,
            team_a_performance: (team_a_performance_value * 10.0).round() / 10.0,
            team_a_players: team_a_performances,
            team_b_id: team_b_id.to_string(),
            team_b_name: team_b_name.to_string(),
            team_b_power: (team_b_power * 10.0).round() / 10.0,
            team_b_performance: (team_b_performance_value * 10.0).round() / 10.0,
            team_b_players: team_b_performances,
            winner_id,
            winner_name,
            power_difference: ((team_a_power - team_b_power) * 10.0).round() / 10.0,
            performance_difference: ((team_a_performance_value - team_b_performance_value) * 10.0).round() / 10.0,
            is_upset,
        }
    }

    /// 模拟完整比赛（BO1/BO3/BO5）
    pub fn simulate_match(
        team_a_id: &str,
        team_a_name: &str,
        team_a_players: &[Player],
        team_b_id: &str,
        team_b_name: &str,
        team_b_players: &[Player],
        best_of: u8,
    ) -> MatchDetail {
        let wins_needed = (best_of + 1) / 2;
        let mut games: Vec<GameDetail> = Vec::new();
        let mut score_a: u8 = 0;
        let mut score_b: u8 = 0;
        let mut game_number: u8 = 0;

        // 模拟直到一方获得足够胜利
        while score_a < wins_needed && score_b < wins_needed {
            game_number += 1;
            let game = Self::simulate_game(
                team_a_id,
                team_a_name,
                team_a_players,
                team_b_id,
                team_b_name,
                team_b_players,
                game_number,
            );

            if game.winner_id == team_a_id {
                score_a += 1;
            } else {
                score_b += 1;
            }

            games.push(game);
        }

        // 分析MVP和关键选手
        let (mvp, key_player) = Self::analyze_match(&games, team_a_id);

        let (winner_id, winner_name) = if score_a > score_b {
            (team_a_id.to_string(), team_a_name.to_string())
        } else {
            (team_b_id.to_string(), team_b_name.to_string())
        };

        MatchDetail {
            match_id: String::new(), // 由调用方设置
            team_a_id: team_a_id.to_string(),
            team_a_name: team_a_name.to_string(),
            team_b_id: team_b_id.to_string(),
            team_b_name: team_b_name.to_string(),
            best_of,
            games,
            final_score_a: score_a,
            final_score_b: score_b,
            winner_id,
            winner_name,
            mvp,
            key_player,
            played_at: Utc::now(),
            tournament_type: None,
            season_id: None,
        }
    }

    /// 分析比赛，找出MVP和关键选手
    fn analyze_match(games: &[GameDetail], _team_a_id: &str) -> (Option<MvpInfo>, Option<KeyPlayer>) {
        use std::collections::HashMap;

        // 统计每位选手的累计影响力
        let mut player_impacts: HashMap<String, (String, String, String, f64, u32)> = HashMap::new();

        for game in games {
            let all_players: Vec<&PlayerPerformance> = game
                .team_a_players
                .iter()
                .chain(game.team_b_players.iter())
                .collect();

            for p in all_players {
                let entry = player_impacts
                    .entry(p.player_id.clone())
                    .or_insert((
                        p.player_id.clone(),
                        p.player_name.clone(),
                        p.team_id.clone(),
                        0.0,
                        0,
                    ));
                entry.3 += p.impact_score;
                entry.4 += 1;
            }
        }

        // 找出影响力最高的选手作为MVP
        let mvp = player_impacts
            .values()
            .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, name, team_id, total_impact, _)| MvpInfo {
                player_id: id.clone(),
                player_name: name.clone(),
                team_id: team_id.clone(),
                total_impact: (*total_impact * 10.0).round() / 10.0,
            });

        // 找出关键选手（最后一局影响最大的）
        let key_player = games.last().and_then(|last_game| {
            let last_game_players: Vec<&PlayerPerformance> = last_game
                .team_a_players
                .iter()
                .chain(last_game.team_b_players.iter())
                .collect();

            last_game_players
                .iter()
                .max_by(|a, b| {
                    a.impact_score
                        .abs()
                        .partial_cmp(&b.impact_score.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|p| KeyPlayer {
                    player_id: p.player_id.clone(),
                    player_name: p.player_name.clone(),
                    team_id: p.team_id.clone(),
                    reason: if p.impact_score > 0.0 {
                        "高发挥".to_string()
                    } else {
                        "低发挥".to_string()
                    },
                    impact_score: p.impact_score,
                    game_number: last_game.game_number,
                })
        });

        (mvp, key_player)
    }

    /// 根据比赛详情生成简要比分（兼容旧系统）
    pub fn get_match_score(detail: &MatchDetail) -> (u8, u8, String) {
        (
            detail.final_score_a,
            detail.final_score_b,
            detail.winner_id.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_players(team_id: &str, team_name: &str) -> Vec<Player> {
        let positions = [
            PlayerPosition::TOP,
            PlayerPosition::JUG,
            PlayerPosition::MID,
            PlayerPosition::ADC,
            PlayerPosition::SUP,
        ];

        positions
            .iter()
            .enumerate()
            .map(|(i, pos)| Player {
                id: format!("{}-{:?}", team_id, pos),
                game_id: format!("Player{}", i + 1),
                name: format!("Player{}", i + 1),
                team_id: team_id.to_string(),
                team_name: Some(team_name.to_string()),
                position: *pos,
                region_id: "LPL".to_string(),
                region_name: Some("LPL".to_string()),
                ability: 75 + (i * 2) as u8,
                potential: 85,
                stability: 70,
                condition: 0,
                age: 22,
                tag: "NORMAL".to_string(),
            })
            .collect()
    }

    #[test]
    fn test_player_performance() {
        let players = create_test_players("team1", "Team A");
        let performances = PlayerEngine::calculate_team_performances(&players);

        assert_eq!(performances.len(), 5);

        for p in &performances {
            // 实际能力应该在合理范围内
            assert!(p.actual_ability >= 0.0 && p.actual_ability <= 100.0);
        }
    }

    #[test]
    fn test_simulate_game() {
        let team_a = create_test_players("team_a", "Team A");
        let team_b = create_test_players("team_b", "Team B");

        let game = PowerEngine::simulate_game(
            "team_a", "Team A", &team_a,
            "team_b", "Team B", &team_b,
            1,
        );

        assert_eq!(game.game_number, 1);
        assert!(game.winner_id == "team_a" || game.winner_id == "team_b");
        assert_eq!(game.team_a_players.len(), 5);
        assert_eq!(game.team_b_players.len(), 5);
    }

    #[test]
    fn test_simulate_match_bo3() {
        let team_a = create_test_players("team_a", "Team A");
        let team_b = create_test_players("team_b", "Team B");

        let result = PowerEngine::simulate_match(
            "team_a", "Team A", &team_a,
            "team_b", "Team B", &team_b,
            3,
        );

        // BO3 应该有2-3局
        assert!(result.games.len() >= 2 && result.games.len() <= 3);
        // 胜者应该赢2局
        assert!(result.final_score_a == 2 || result.final_score_b == 2);
        // 应该有MVP
        assert!(result.mvp.is_some());
    }

    #[test]
    fn test_simulate_match_bo5() {
        let team_a = create_test_players("team_a", "Team A");
        let team_b = create_test_players("team_b", "Team B");

        let result = PowerEngine::simulate_match(
            "team_a", "Team A", &team_a,
            "team_b", "Team B", &team_b,
            5,
        );

        // BO5 应该有3-5局
        assert!(result.games.len() >= 3 && result.games.len() <= 5);
        // 胜者应该赢3局
        assert!(result.final_score_a == 3 || result.final_score_b == 3);
    }

    #[test]
    fn test_stability_sigma() {
        assert_eq!(PlayerEngine::calculate_stability_sigma(100), 0.0);
        assert_eq!(PlayerEngine::calculate_stability_sigma(60), 4.0);
        assert_eq!(PlayerEngine::calculate_stability_sigma(0), 10.0);
    }

    #[test]
    fn test_team_power_calculation() {
        let players = create_test_players("team1", "Team A");
        let performances = PlayerEngine::calculate_team_performances(&players);
        let power = PowerEngine::calculate_team_power(&performances);

        // 战力应该在合理范围内
        assert!(power >= 50.0 && power <= 100.0);
    }

    #[test]
    fn test_higher_power_wins_more() {
        // 创建一个强队和一个弱队
        let strong_team: Vec<Player> = (0..5).map(|i| Player {
            id: format!("strong-{}", i),
            game_id: format!("StrongPlayer{}", i),
            name: format!("Player{}", i),
            team_id: "strong".to_string(),
            team_name: Some("Strong Team".to_string()),
            position: match i {
                0 => PlayerPosition::TOP,
                1 => PlayerPosition::JUG,
                2 => PlayerPosition::MID,
                3 => PlayerPosition::ADC,
                _ => PlayerPosition::SUP,
            },
            region_id: "LPL".to_string(),
            region_name: Some("LPL".to_string()),
            ability: 90,
            potential: 95,
            stability: 80,
            condition: 5,
            age: 24,
            tag: "GENIUS".to_string(),
        }).collect();

        let weak_team: Vec<Player> = (0..5).map(|i| Player {
            id: format!("weak-{}", i),
            game_id: format!("WeakPlayer{}", i),
            name: format!("Player{}", i),
            team_id: "weak".to_string(),
            team_name: Some("Weak Team".to_string()),
            position: match i {
                0 => PlayerPosition::TOP,
                1 => PlayerPosition::JUG,
                2 => PlayerPosition::MID,
                3 => PlayerPosition::ADC,
                _ => PlayerPosition::SUP,
            },
            region_id: "LCS".to_string(),
            region_name: Some("LCS".to_string()),
            ability: 60,
            potential: 70,
            stability: 60,
            condition: -2,
            age: 22,
            tag: "ORDINARY".to_string(),
        }).collect();

        // 模拟100场比赛
        let mut strong_wins = 0;
        for _ in 0..100 {
            let result = PowerEngine::simulate_match(
                "strong", "Strong Team", &strong_team,
                "weak", "Weak Team", &weak_team,
                1, // BO1
            );
            if result.winner_id == "strong" {
                strong_wins += 1;
            }
        }

        // 强队应该赢得大多数比赛（至少60%）
        assert!(strong_wins >= 60, "Strong team won {} out of 100 games", strong_wins);
    }
}
