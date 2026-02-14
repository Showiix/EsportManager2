use serde::{Deserialize, Serialize};

/// 比赛赛制
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchFormat {
    Bo1,
    Bo3,
    Bo5,
}

impl MatchFormat {
    /// 获取需要赢的局数
    pub fn wins_needed(&self) -> u8 {
        match self {
            MatchFormat::Bo1 => 1,
            MatchFormat::Bo3 => 2,
            MatchFormat::Bo5 => 3,
        }
    }
}

/// 比赛状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchStatus {
    Scheduled,
    InProgress,
    Completed,
}

/// 比赛记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: u64,
    pub tournament_id: u64,
    pub stage: String,
    pub round: Option<u32>,
    pub match_order: Option<u32>,
    pub format: MatchFormat,
    pub home_team_id: u64,
    pub away_team_id: u64,
    pub home_score: u8,
    pub away_score: u8,
    pub winner_id: Option<u64>,
    pub status: MatchStatus,
}

/// 比赛小局记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchGame {
    pub id: u64,
    pub match_id: u64,
    pub game_number: u8,
    pub home_power: f64,
    pub away_power: f64,
    pub home_base_power: Option<f64>,
    pub away_base_power: Option<f64>,
    pub home_synergy_bonus: Option<f64>,
    pub away_synergy_bonus: Option<f64>,
    pub home_bp_bonus: Option<f64>,
    pub away_bp_bonus: Option<f64>,
    pub home_version_bonus: Option<f64>,
    pub away_version_bonus: Option<f64>,
    pub home_performance: f64,
    pub away_performance: f64,
    pub winner_id: u64,
    pub duration_minutes: Option<u32>,
}

/// 比赛结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub match_info: Match,
    pub games: Vec<MatchGame>,
    pub winner_id: u64,
    pub home_score: u8,
    pub away_score: u8,
}
