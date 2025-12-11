//! 赛事结果模型
//!
//! 用于记录赛事的最终排名结果

use serde::{Deserialize, Serialize};
use super::honor::HonorStats;

/// 赛事结果 - 记录冠亚季殿军
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentResult {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_type: String,
    pub tournament_name: String,

    /// 冠军
    pub champion_team_id: u64,
    pub champion_team_name: String,

    /// 亚军
    pub runner_up_team_id: u64,
    pub runner_up_team_name: String,

    /// 季军（可选，国际赛事有）
    pub third_team_id: Option<u64>,
    pub third_team_name: Option<String>,

    /// 殿军（可选，国际赛事有）
    pub fourth_team_id: Option<u64>,
    pub fourth_team_name: Option<String>,

    /// 决赛比赛ID
    pub final_match_id: Option<u64>,
    /// 决赛比分 (如 "3:2")
    pub final_score: Option<String>,

    /// 赛事总比赛场数
    pub total_matches: Option<u32>,
    /// 赛事总小局数
    pub total_games: Option<u32>,

    pub created_at: Option<String>,
}

impl TournamentResult {
    /// 创建新的赛事结果
    pub fn new(
        save_id: String,
        season_id: u64,
        tournament_id: u64,
        tournament_type: String,
        tournament_name: String,
        champion_team_id: u64,
        champion_team_name: String,
        runner_up_team_id: u64,
        runner_up_team_name: String,
    ) -> Self {
        Self {
            id: 0,
            save_id,
            season_id,
            tournament_id,
            tournament_type,
            tournament_name,
            champion_team_id,
            champion_team_name,
            runner_up_team_id,
            runner_up_team_name,
            third_team_id: None,
            third_team_name: None,
            fourth_team_id: None,
            fourth_team_name: None,
            final_match_id: None,
            final_score: None,
            total_matches: None,
            total_games: None,
            created_at: None,
        }
    }

    /// 设置季军
    pub fn with_third(mut self, team_id: u64, team_name: String) -> Self {
        self.third_team_id = Some(team_id);
        self.third_team_name = Some(team_name);
        self
    }

    /// 设置殿军
    pub fn with_fourth(mut self, team_id: u64, team_name: String) -> Self {
        self.fourth_team_id = Some(team_id);
        self.fourth_team_name = Some(team_name);
        self
    }

    /// 设置决赛信息
    pub fn with_final_info(mut self, match_id: u64, score: String) -> Self {
        self.final_match_id = Some(match_id);
        self.final_score = Some(score);
        self
    }

    /// 设置统计信息
    pub fn with_stats(mut self, total_matches: u32, total_games: u32) -> Self {
        self.total_matches = Some(total_matches);
        self.total_games = Some(total_games);
        self
    }

    /// 是否有四强（季军和殿军）
    pub fn has_semi_finalists(&self) -> bool {
        self.third_team_id.is_some() && self.fourth_team_id.is_some()
    }
}

/// 选手赛事统计 - 用于MVP计算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTournamentStats {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_type: String,
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,

    /// 参与小局数
    pub games_played: u32,
    /// 胜利小局数
    pub games_won: u32,
    /// 累计影响力
    pub total_impact: f64,
    /// 平均影响力
    pub avg_impact: f64,
    /// 单局最高影响力
    pub max_impact: f64,
    /// 平均发挥值
    pub avg_performance: f64,
    /// 最佳发挥
    pub best_performance: f64,
    /// 单局MVP次数
    pub game_mvp_count: u32,

    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl PlayerTournamentStats {
    /// 创建新的选手赛事统计
    pub fn new(
        save_id: String,
        season_id: u64,
        tournament_id: u64,
        tournament_type: String,
        player_id: u64,
        player_name: String,
        team_id: u64,
        team_name: String,
        position: String,
    ) -> Self {
        Self {
            id: 0,
            save_id,
            season_id,
            tournament_id,
            tournament_type,
            player_id,
            player_name,
            team_id,
            team_name,
            position,
            games_played: 0,
            games_won: 0,
            total_impact: 0.0,
            avg_impact: 0.0,
            max_impact: 0.0,
            avg_performance: 0.0,
            best_performance: 0.0,
            game_mvp_count: 0,
            created_at: None,
            updated_at: None,
        }
    }

    /// 记录一局比赛的表现
    pub fn record_game(&mut self, impact: f64, performance: f64, won: bool, is_mvp: bool) {
        self.games_played += 1;
        if won {
            self.games_won += 1;
        }

        self.total_impact += impact;
        self.avg_impact = self.total_impact / self.games_played as f64;

        if impact > self.max_impact {
            self.max_impact = impact;
        }

        // 更新平均发挥
        let prev_total = self.avg_performance * (self.games_played - 1) as f64;
        self.avg_performance = (prev_total + performance) / self.games_played as f64;

        if performance > self.best_performance {
            self.best_performance = performance;
        }

        if is_mvp {
            self.game_mvp_count += 1;
        }
    }

    /// 计算MVP评分
    /// 公式: total_impact * 0.5 + avg_impact * 0.3 + win_rate * 0.2
    pub fn mvp_score(&self) -> f64 {
        let win_rate = if self.games_played > 0 {
            self.games_won as f64 / self.games_played as f64
        } else {
            0.0
        };

        self.total_impact * 0.5 + self.avg_impact * 0.3 + win_rate * 20.0 // win_rate * 0.2 * 100
    }

    /// 胜率
    pub fn win_rate(&self) -> f64 {
        if self.games_played > 0 {
            self.games_won as f64 / self.games_played as f64
        } else {
            0.0
        }
    }

    /// 转换为HonorStats（用于MVP荣誉记录）
    pub fn to_honor_stats(&self) -> HonorStats {
        HonorStats {
            total_impact: self.total_impact,
            mvp_count: self.game_mvp_count,
            games_played: self.games_played,
            wins: self.games_won,
            avg_performance: self.avg_performance,
        }
    }
}

/// 决赛选手表现 - 用于决赛MVP计算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalsPlayerStats {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,
    pub games_played: u32,
    pub total_impact: f64,
    pub avg_impact: f64,
    pub is_winner: bool,
}

impl FinalsPlayerStats {
    pub fn new(
        player_id: u64,
        player_name: String,
        team_id: u64,
        team_name: String,
        position: String,
        is_winner: bool,
    ) -> Self {
        Self {
            player_id,
            player_name,
            team_id,
            team_name,
            position,
            games_played: 0,
            total_impact: 0.0,
            avg_impact: 0.0,
            is_winner,
        }
    }

    pub fn add_game(&mut self, impact: f64) {
        self.games_played += 1;
        self.total_impact += impact;
        self.avg_impact = self.total_impact / self.games_played as f64;
    }
}

/// 赛事荣誉汇总
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TournamentHonors {
    pub team_champion: Option<super::Honor>,
    pub team_runner_up: Option<super::Honor>,
    pub team_third: Option<super::Honor>,
    pub team_fourth: Option<super::Honor>,
    pub player_champions: Vec<super::Honor>,
    pub tournament_mvp: Option<super::Honor>,
    pub finals_mvp: Option<super::Honor>,
}
