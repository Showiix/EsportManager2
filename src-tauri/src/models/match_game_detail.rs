use serde::{Deserialize, Serialize};

/// 比赛每局详情 (数据库持久化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchGameDetail {
    pub id: String,
    pub save_id: String,
    pub match_id: i64,
    pub game_number: i32,
    pub winner_team_id: i64,
    pub loser_team_id: i64,
    pub duration_minutes: Option<i32>,
    pub mvp_player_id: Option<i64>,
    pub key_player_id: Option<i64>,
    pub home_power: Option<f64>,
    pub away_power: Option<f64>,
    pub home_meta_power: Option<f64>,
    pub away_meta_power: Option<f64>,
    pub created_at: Option<String>,
}

/// 每局选手表现 (数据库持久化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerPerformance {
    pub id: String,
    pub save_id: String,
    pub game_id: String,
    pub player_id: i64,
    pub player_name: String,           // 选手名称（快照）
    pub team_id: i64,
    pub team_name: String,             // 队伍名称（快照）
    pub position: String,
    pub base_ability: f64,             // 基础能力值
    pub condition_bonus: f64,          // 状态加成
    pub stability_noise: f64,          // 稳定性波动
    pub actual_ability: f64,           // 实际发挥值
    pub impact_score: f64,             // 影响力得分
    pub mvp_score: f64,                // MVP 得分
    pub is_mvp: bool,
    pub is_key_player: bool,
    // 详细战斗数据
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub assists: Option<i32>,
    pub cs: Option<i32>,
    pub gold: Option<i32>,
    pub damage_dealt: Option<i32>,
    pub damage_taken: Option<i32>,
    pub vision_score: Option<i32>,
    // 特性系统
    pub traits_json: Option<String>,              // 选手拥有的特性 (JSON数组)
    pub activated_traits_json: Option<String>,    // 本局激活的特性效果 (JSON数组)
    pub created_at: Option<String>,
}

/// 完整比赛详情 (包含所有局和选手表现)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchFullDetails {
    pub match_id: i64,
    pub games: Vec<GameDetailWithPerformances>,
}

/// 单局详情 (包含选手表现)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetailWithPerformances {
    pub game: MatchGameDetail,
    pub performances: Vec<GamePlayerPerformance>,
}

/// 保存比赛详情输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMatchDetailsInput {
    pub match_id: i64,
    pub games: Vec<SaveGameInput>,
}

/// 保存单局输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGameInput {
    pub game_number: i32,
    pub winner_team_id: i64,
    pub loser_team_id: i64,
    pub duration_minutes: Option<i32>,
    pub mvp_player_id: Option<i64>,
    pub key_player_id: Option<i64>,
    pub home_power: Option<f64>,
    pub away_power: Option<f64>,
    pub home_meta_power: Option<f64>,
    pub away_meta_power: Option<f64>,
    pub performances: Vec<SavePerformanceInput>,
}

/// 保存选手表现输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePerformanceInput {
    pub player_id: i64,
    pub player_name: String,           // 选手名称（快照）
    pub team_id: i64,
    pub team_name: String,             // 队伍名称（快照）
    pub position: String,
    pub base_ability: f64,             // 基础能力值
    pub condition_bonus: f64,          // 状态加成
    pub stability_noise: f64,          // 稳定性波动
    pub actual_ability: f64,           // 实际发挥值
    pub impact_score: f64,             // 影响力得分
    pub mvp_score: f64,                // MVP 得分
    pub is_mvp: bool,
    pub is_key_player: bool,
    // 详细战斗数据
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub assists: Option<i32>,
    pub cs: Option<i32>,
    pub gold: Option<i32>,
    pub damage_dealt: Option<i32>,
    pub damage_taken: Option<i32>,
    pub vision_score: Option<i32>,
    // 特性系统
    pub traits_json: Option<String>,              // 选手拥有的特性 (JSON数组)
    pub activated_traits_json: Option<String>,    // 本局激活的特性效果 (JSON数组)
}
