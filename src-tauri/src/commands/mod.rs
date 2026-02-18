pub mod save_commands;
pub mod team_commands;
pub mod game_commands;
pub mod honor_commands;
pub mod draft_commands;
pub mod draft_auction_commands;
pub mod finance_commands;
pub mod query_commands;
pub mod international_commands;
pub mod match_commands;
pub mod event_commands;
pub mod stats_commands;
pub mod time_commands;
pub mod points_commands;
pub mod match_detail_commands;
pub mod dev_commands;
pub mod awards_commands;
pub mod transfer_commands;
pub mod log_commands;
pub mod perf_commands;
pub mod meta_commands;
pub mod champion_commands;
pub mod ladder_commands;

pub use save_commands::*;
pub use team_commands::*;
pub use game_commands::{
    GameStateInfo, MatchInfo, StandingInfo, FixTournamentStatusResult,
    get_phase_display_name,
    get_game_state,
    advance_phase, initialize_current_phase, complete_current_phase, start_new_season,
    get_tournament_matches, get_standings, fix_tournament_status,
    simulate_next_match, simulate_all_matches,
};
pub use match_commands::{
    DetailedMatchResult, DetailedGameResult, PlayerMvpInfo, TeamMatchStats,
    PlayerGameStats, ActivatedTraitInfo, GameEvent, BatchDetailedResult,
    PlayerSeasonStats, MatchPrediction,
    simulate_match_detailed, simulate_all_matches_detailed,
    get_player_season_stats, get_match_prediction, get_match_lineups,
    update_match_result, update_match_teams, cancel_match,
};
pub use honor_commands::*;
pub use draft_commands::*;
pub use draft_auction_commands::*;
pub use finance_commands::*;
pub use query_commands::*;
pub use international_commands::*;
pub use event_commands::*;
pub use stats_commands::*;
pub use time_commands::*;
pub use points_commands::*;
pub use match_detail_commands::*;
pub use dev_commands::*;
pub use awards_commands::*;
pub use transfer_commands::*;
pub use log_commands::*;
pub use perf_commands::*;
pub use meta_commands::*;
pub use champion_commands::*;
pub use ladder_commands::*;

use serde::{Deserialize, Serialize};

/// 通用响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

/// 获取应用信息
#[tauri::command]
pub fn get_app_info() -> ApiResponse<AppInfo> {
    ApiResponse::success(AppInfo {
        name: "电竞比赛模拟器 2".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string    (),
        description: "一款基于战力值和正态分布算法的电竞赛事模拟游戏".to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// 模拟单场比赛 (测试用)
#[tauri::command]
pub fn simulate_test_match(
    home_power: f64,
    away_power: f64,
    format: String,
) -> ApiResponse<TestMatchResult> {
    use crate::engines::MatchSimulationEngine;
    use crate::models::MatchFormat;

    let match_format = match format.to_uppercase().as_str() {
        "BO1" => MatchFormat::Bo1,
        "BO3" => MatchFormat::Bo3,
        "BO5" => MatchFormat::Bo5,
        _ => return ApiResponse::error("Invalid format. Use BO1, BO3, or BO5"),
    };

    let engine = MatchSimulationEngine::default();
    let result = engine.simulate_match(
        0, 0, "TEST", match_format, 1, 2, home_power, away_power
    );

    let games: Vec<GameDetail> = result.games.iter().map(|g| GameDetail {
        game_number: g.game_number,
        home_performance: g.home_performance,
        away_performance: g.away_performance,
        winner: if g.winner_id == 1 { "HOME" } else { "AWAY" }.to_string(),
    }).collect();

    ApiResponse::success(TestMatchResult {
        home_score: result.home_score,
        away_score: result.away_score,
        winner: if result.winner_id == 1 { "HOME" } else { "AWAY" }.to_string(),
        games,
        win_probability: engine.calculate_win_probability(home_power, away_power),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestMatchResult {
    pub home_score: u8,
    pub away_score: u8,
    pub winner: String,
    pub games: Vec<GameDetail>,
    pub win_probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameDetail {
    pub game_number: u8,
    pub home_performance: f64,
    pub away_performance: f64,
    pub winner: String,
}
