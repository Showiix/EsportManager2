// Type definitions and public API re-export
use serde::{Deserialize, Serialize};

// ==================== 类型定义 ====================

/// 详细比赛结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedMatchResult {
    pub match_id: u64,
    pub tournament_id: u64,
    pub home_team_id: u64,
    pub away_team_id: u64,
    pub home_team_name: String,
    pub away_team_name: String,
    pub home_score: u8,
    pub away_score: u8,
    pub winner_id: u64,
    pub games: Vec<DetailedGameResult>,
    pub match_mvp: Option<PlayerMvpInfo>,
    pub home_team_stats: TeamMatchStats,
    pub away_team_stats: TeamMatchStats,
}

/// 详细小局结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedGameResult {
    pub game_number: u8,
    pub winner_id: u64,
    pub duration_minutes: u32,
    pub home_performance: f64,
    pub away_performance: f64,
    pub game_mvp: PlayerMvpInfo,
    pub home_players: Vec<PlayerGameStats>,
    pub away_players: Vec<PlayerGameStats>,
    pub key_events: Vec<GameEvent>,
}

/// 球员MVP信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMvpInfo {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub position: String,
    pub mvp_score: f64,
}

/// 队伍比赛统计
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMatchStats {
    pub team_id: u64,
    pub total_kills: u32,
    pub total_deaths: u32,
    pub total_assists: u32,
    pub total_gold: u64,
    pub average_game_duration: u32,
    pub first_blood_rate: f64,
    pub first_tower_rate: f64,
    pub baron_rate: f64,
    pub dragon_rate: f64,
}

/// 球员单局统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameStats {
    pub player_id: u64,
    pub player_name: String,
    pub position: String,
    pub base_ability: u8,        // 选手基础能力值
    pub condition_bonus: f64,    // 状态加成
    pub stability_noise: f64,    // 稳定性波动
    pub actual_ability: f64,     // 实际发挥 = base + condition + noise
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub cs: u32,
    pub gold: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub vision_score: u32,
    pub mvp_score: f64,
    pub impact_score: f64,       // 影响力分数
    pub traits: Vec<String>,     // 选手特性列表
    pub activated_traits: Vec<ActivatedTraitInfo>,  // 本局激活的特性效果
}

/// 激活的特性效果信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatedTraitInfo {
    pub trait_type: String,
    pub name: String,
    pub effect: String,
    pub value: f64,
    pub is_positive: bool,
}

/// 比赛关键事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub time_minutes: u32,
    pub event_type: String,
    pub description: String,
    pub team_id: u64,
}

/// 批量模拟结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchDetailedResult {
    pub results: Vec<DetailedMatchResult>,
    pub total: u32,
    pub success: u32,
    pub failed: u32,
}

/// 球员赛季统计
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSeasonStats {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub position: String,
    pub games_played: u32,
    pub total_kills: u32,
    pub total_deaths: u32,
    pub total_assists: u32,
    pub average_kda: f64,
    pub average_cs_per_min: f64,
    pub average_damage: u64,
    pub mvp_count: u32,
    pub win_rate: f64,
}

/// 比赛预测结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchPrediction {
    pub match_id: u64,
    pub home_team_id: u64,
    pub home_team_name: String,
    pub home_power: f64,
    pub home_win_probability: f64,
    pub away_team_id: u64,
    pub away_team_name: String,
    pub away_power: f64,
    pub away_win_probability: f64,
    pub predicted_score: String,
}

// ==================== 实现 ====================

impl TeamMatchStats {
    pub fn default(team_id: u64) -> Self {
        Self {
            team_id,
            total_kills: 0,
            total_deaths: 0,
            total_assists: 0,
            total_gold: 0,
            average_game_duration: 0,
            first_blood_rate: 0.0,
            first_tower_rate: 0.0,
            baron_rate: 0.0,
            dragon_rate: 0.0,
        }
    }
}

// ==================== 模块声明 ====================

pub mod simulation;
pub mod queries;
pub mod mutations;

// ==================== 公开接口 re-export ====================

// 从 simulation 模块导出
pub use simulation::{
    simulate_match_detailed,
    simulate_all_matches_detailed,
};

// 从 queries 模块导出
pub use queries::{
    get_player_season_stats,
    get_match_prediction,
};

// 从 mutations 模块导出
pub use mutations::{
    update_match_result,
    update_match_teams,
    cancel_match,
};
