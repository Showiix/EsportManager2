use serde::{Deserialize, Serialize};
use super::SeasonPhase;

/// 阶段状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PhaseStatus {
    /// 未初始化 - 阶段刚进入，赛事未创建
    NotInitialized,
    /// 进行中 - 赛事已创建，比赛进行中
    InProgress,
    /// 已完成 - 所有比赛完成，可以推进
    Completed,
}

/// 游戏时间状态 - 统一的时间状态返回结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTimeState {
    /// 存档ID
    pub save_id: String,
    /// 当前赛季
    pub current_season: u32,
    /// 当前阶段
    pub current_phase: SeasonPhase,
    /// 当前阶段显示名称
    pub phase_display_name: String,
    /// 阶段状态
    pub phase_status: PhaseStatus,
    /// 阶段进度
    pub phase_progress: PhaseProgress,
    /// 赛季进度
    pub season_progress: SeasonProgress,
    /// 可用操作
    pub available_actions: Vec<TimeAction>,
    /// 是否可以推进到下一阶段
    pub can_advance: bool,
    /// 下一阶段
    pub next_phase: Option<String>,
}

/// 阶段进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProgress {
    /// 当前阶段的赛事列表
    pub tournaments: Vec<TournamentProgress>,
    /// 总比赛数
    pub total_matches: u32,
    /// 已完成比赛数
    pub completed_matches: u32,
    /// 进度百分比
    pub percentage: f32,
}

/// 赛事进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentProgress {
    pub tournament_id: u64,
    pub tournament_name: String,
    pub region: Option<String>,
    pub total_matches: u32,
    pub completed_matches: u32,
    pub status: String,
}

/// 赛季进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonProgress {
    /// 所有阶段列表
    pub phases: Vec<PhaseInfo>,
    /// 当前阶段索引（0-13）
    pub current_phase_index: u32,
    /// 总阶段数
    pub total_phases: u32,
    /// 赛季进度百分比
    pub percentage: f32,
}

/// 阶段信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseInfo {
    pub phase: String,
    pub display_name: String,
    pub status: String, // "completed", "current", "upcoming"
    pub index: u32,
}

/// 可用的时间操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeAction {
    /// 初始化当前阶段
    InitializePhase,
    /// 模拟下一场比赛
    SimulateNextMatch,
    /// 模拟所有比赛
    SimulateAllMatches,
    /// 完成当前阶段并推进
    CompleteAndAdvance,
    /// 快进到下一阶段
    FastForwardPhase,
    /// 快进到夏季赛
    FastForwardToSummer,
    /// 快进到世界赛
    FastForwardToWorlds,
    /// 快进到赛季结束
    FastForwardToSeasonEnd,
    /// 开始转会窗口
    StartTransferWindow,
    /// 执行转会轮次
    ExecuteTransferRound,
    /// 开始选秀
    StartDraft,
    /// 开始新赛季
    StartNewSeason,
}

impl TimeAction {
    /// 获取操作显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeAction::InitializePhase => "初始化阶段",
            TimeAction::SimulateNextMatch => "模拟下一场",
            TimeAction::SimulateAllMatches => "模拟所有比赛",
            TimeAction::CompleteAndAdvance => "完成并推进",
            TimeAction::FastForwardPhase => "快进下一阶段",
            TimeAction::FastForwardToSummer => "快进到夏季赛",
            TimeAction::FastForwardToWorlds => "快进到世界赛",
            TimeAction::FastForwardToSeasonEnd => "快进到赛季结束",
            TimeAction::StartTransferWindow => "开始转会",
            TimeAction::ExecuteTransferRound => "执行转会轮次",
            TimeAction::StartDraft => "开始选秀",
            TimeAction::StartNewSeason => "开始新赛季",
        }
    }
}

/// 快进目标
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FastForwardTarget {
    /// 快进到下一阶段
    NextPhase,
    /// 快进到指定阶段
    ToPhase(SeasonPhase),
    /// 快进到赛季结束
    SeasonEnd,
}

/// 快进结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastForwardResult {
    pub success: bool,
    pub start_phase: String,
    pub end_phase: String,
    pub phases_advanced: u32,
    pub matches_simulated: u32,
    pub message: String,
}

/// 完成并推进结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteAndAdvanceResult {
    pub success: bool,
    pub completed_phase: String,
    pub new_phase: Option<String>,
    pub honors_awarded: Vec<HonorInfo>,
    pub message: String,
    /// 更新后的时间状态
    pub new_time_state: GameTimeState,
}

/// 荣誉信息（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonorInfo {
    pub honor_type: String,
    pub recipient_name: String,
    pub tournament_name: String,
}
