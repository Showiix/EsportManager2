//! 开发工具命令 - 用于调试和数据修复
//!
//! 这些命令仅供开发使用，生产环境应禁用

use serde::{Deserialize, Serialize};

mod data_repair;
mod recalculation;
mod debug;
mod management;

// ==================== 共享类型定义 ====================

/// 开发命令结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DevCommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub error: Option<String>,
}

impl<T> DevCommandResult<T> {
    pub fn ok(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: String::new(),
            error: Some(msg.into()),
        }
    }
}

impl DevCommandResult<()> {
    pub fn ok_msg(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(()),
            message: message.into(),
            error: None,
        }
    }
}

/// 数据一致性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    pub total_checks: i32,
    pub passed: i32,
    pub failed: i32,
    pub issues: Vec<ConsistencyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub category: String,
    pub description: String,
    pub severity: String, // "warning" | "error"
}

/// 同步选手场次统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub updated_count: i32,
    pub details: Vec<String>,
}

/// 游戏状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatusSummary {
    pub current_season: u32,
    pub current_phase: String,
    pub phase_completed: bool,
    pub team_count: i32,
    pub player_count: i32,
    pub tournament_count: i32,
    pub total_matches: i32,
    pub completed_matches: i32,
    pub scheduled_matches: i32,
    pub honor_count: i32,
}

/// 未完成比赛信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteMatchInfo {
    pub match_id: u64,
    pub stage: String,
    pub status: String,
    pub tournament_name: String,
    pub tournament_type: String,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
}

/// 首发修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStartersResult {
    pub teams_fixed: i32,
    pub players_fixed: i32,
    pub details: Vec<TeamFixInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamFixInfo {
    pub team_name: String,
    pub fixes: Vec<String>,
}

/// 统计数据重建结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuildStatsResult {
    pub created_count: i32,
    pub updated_count: i32,
    pub players: Vec<String>,
}

// ==================== Re-export 命令函数 ====================

pub use data_repair::{
    dev_reassign_honors,
    dev_sync_player_games_played,
    dev_fix_starters,
    dev_force_complete_match,
    dev_migrate_loyalty_satisfaction,
};

pub use recalculation::{
    dev_recalculate_annual_points,
    dev_recalculate_standings,
    dev_recalculate_market_values,
    dev_redistribute_prizes,
    dev_rebuild_player_season_stats,
};

pub use debug::{
    dev_check_data_consistency,
    dev_check_incomplete_matches,
    dev_get_game_status,
};

pub use management::{
    dev_reset_phase,
    dev_simulate_all_matches,
    dev_grant_funds,
    dev_reset_save,
};
