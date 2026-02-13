use serde::{Deserialize, Serialize};
use crate::models::SeasonPhase;

// ==================== 共享类型定义 ====================

/// 游戏状态信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GameStateInfo {
    pub current_season: u32,
    pub current_phase: String,
    pub phase_name: String,
    pub progress: (u32, u32),
    pub available_actions: Vec<String>,
}

/// 比赛信息
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchInfo {
    pub id: u64,
    pub tournament_id: u64,
    pub stage: String,
    pub round: Option<u32>,
    pub match_order: Option<u32>,
    pub format: String,
    pub home_team_id: u64,
    pub away_team_id: u64,
    pub home_score: u32,
    pub away_score: u32,
    pub winner_id: Option<u64>,
    pub status: String,
}

/// 积分榜信息
#[derive(Debug, Serialize, Deserialize)]
pub struct StandingInfo {
    pub team_id: u64,
    pub team_name: Option<String>,
    pub rank: Option<u32>,
    pub matches_played: u32,
    pub wins: u32,
    pub losses: u32,
    pub points: u32,
    pub game_diff: i32,
}

/// 修复赛事状态结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixTournamentStatusResult {
    pub fixed_count: u32,
    pub fixed_tournaments: Vec<String>,
    pub message: String,
}

// ==================== 辅助函数 ====================

/// 获取阶段显示名称
pub fn get_phase_display_name(phase: &SeasonPhase) -> String {
    match phase {
        SeasonPhase::SpringRegular => "春季常规赛".to_string(),
        SeasonPhase::SpringPlayoffs => "春季季后赛".to_string(),
        SeasonPhase::Msi => "MSI季中赛".to_string(),
        SeasonPhase::MadridMasters => "马德里大师赛".to_string(),
        SeasonPhase::SummerRegular => "夏季常规赛".to_string(),
        SeasonPhase::SummerPlayoffs => "夏季季后赛".to_string(),
        SeasonPhase::ClaudeIntercontinental => "Claude洲际赛".to_string(),
        SeasonPhase::WorldChampionship => "世界赛".to_string(),
        SeasonPhase::ShanghaiMasters => "上海大师赛".to_string(),
        SeasonPhase::IcpIntercontinental => "ICP洲际对抗赛".to_string(),
        SeasonPhase::SuperIntercontinental => "Super洲际邀请赛".to_string(),
        SeasonPhase::AnnualAwards => "年度颁奖典礼".to_string(),
        SeasonPhase::TransferWindow => "转会期".to_string(),
        SeasonPhase::Draft => "选秀大会".to_string(),
        SeasonPhase::SeasonEnd => "赛季总结".to_string(),
    }
}

// ==================== 模块声明 ====================

pub mod state;
pub mod phase;
pub mod tournament;
pub mod simulation;

// ==================== Re-export ====================

pub use state::get_game_state;
pub use phase::{advance_phase, initialize_current_phase, complete_current_phase, start_new_season};
pub use tournament::{get_tournament_matches, get_standings, fix_tournament_status};
pub use simulation::{simulate_next_match, simulate_all_matches};
