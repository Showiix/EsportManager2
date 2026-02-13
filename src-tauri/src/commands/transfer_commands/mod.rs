//! 转会系统命令模块
//!
//! 实现前端调用的转会相关命令

use serde::Deserialize;

// 子模块声明
mod window;
mod evaluation;
mod market;
mod personality;
mod bids;

// 重新导出所有命令函数
pub use window::*;
pub use evaluation::*;
pub use market::*;
pub use personality::*;
pub use bids::*;

// ============================================
// 共享类型定义
// ============================================

/// AI性格更新请求
#[derive(Debug, Deserialize)]
pub struct UpdatePersonalityRequest {
    pub personality: String,
    pub short_term_focus: Option<f64>,
    pub long_term_focus: Option<f64>,
    pub risk_tolerance: Option<f64>,
    pub youth_preference: Option<f64>,
    pub star_chasing: Option<f64>,
    pub bargain_hunting: Option<f64>,
}

/// 战队赛季评估信息（用于前端展示）
#[derive(Debug, Clone, serde::Serialize)]
pub struct TeamSeasonEvaluationInfo {
    pub evaluation_id: i64,
    pub team_id: i64,
    pub team_name: String,
    pub team_short_name: String,
    pub region_code: String,
    pub season_id: i64,
    pub current_rank: i32,
    pub last_rank: i32,
    pub spring_rank: Option<i32>,
    pub summer_rank: Option<i32>,
    pub stability_score: i32,
    pub strategy: String,
    pub urgency_level: String,
    pub roster_power: f64,
    pub roster_count: i32,
    pub avg_age: f64,
    pub avg_ability: f64,
    pub budget_remaining: i64,
    pub evaluation_reason: String,
    pub created_at: String,
}

/// 位置需求信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PositionNeedInfo {
    pub position: String,
    pub current_starter_name: Option<String>,
    pub current_starter_ability: Option<i32>,
    pub current_starter_age: Option<i32>,
    pub need_level: String,
    pub min_ability_target: Option<i32>,
    pub reason: Option<String>,
}

/// 选手挂牌评估信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerListingEvaluationInfo {
    pub player_id: i64,
    pub player_name: String,
    pub position: String,
    pub age: i64,
    pub ability: i64,
    pub team_id: i64,
    pub team_name: String,
    pub should_list: bool,
    pub list_reason: String,
    pub is_protected: bool,
    pub protect_reason: String,
    pub estimated_value: i64,
}

/// 选手留队评估信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerStayEvaluationInfo {
    pub player_id: i64,
    pub player_name: String,
    pub position: String,
    pub age: i64,
    pub ability: i64,
    pub team_id: i64,
    pub team_name: String,
    pub team_short_name: Option<String>,
    pub region_code: String,
    pub stay_score: f64,
    pub wants_to_leave: bool,
    pub leave_reason: String,
    pub salary: i64,
    pub satisfaction: i64,
    pub loyalty: i64,
}

/// 选手解约结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReleasePlayerResult {
    pub player_id: i64,
    pub player_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub release_fee: i64,
    pub remaining_balance: i64,
}
