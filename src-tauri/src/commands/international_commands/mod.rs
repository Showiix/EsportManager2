use serde::{Deserialize, Serialize};

mod msi;
mod worlds;
mod masters;
mod super_tournament;
mod icp;
mod bracket;
mod swiss;
mod completion;
pub(crate) mod helpers;

pub use msi::*;
pub use worlds::*;
pub use masters::*;
pub use super_tournament::*;
pub use icp::*;
pub use bracket::*;
pub use swiss::*;
pub use completion::*;

/// 参赛队伍信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantTeam {
    pub team_id: u64,
    pub team_name: String,
    pub region_id: u64,
    pub region_name: String,
    pub seed: u32,
    pub qualification_type: String,
}

/// 赛事对阵信息
#[derive(Debug, Serialize, Deserialize)]
pub struct BracketInfo {
    pub tournament_id: u64,
    pub tournament_name: String,
    pub tournament_type: String,
    pub stages: Vec<StageInfo>,
    pub matches: Vec<MatchBracketInfo>,
}

/// 阶段信息
#[derive(Debug, Serialize, Deserialize)]
pub struct StageInfo {
    pub name: String,
    pub display_name: String,
    pub order: u32,
    pub total_matches: u32,
    pub completed_matches: u32,
}

/// 对阵详情
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchBracketInfo {
    pub match_id: u64,
    pub stage: String,
    pub match_order: u32,
    pub format: String,
    pub home_team: Option<TeamBracketInfo>,
    pub away_team: Option<TeamBracketInfo>,
    pub home_score: u32,
    pub away_score: u32,
    pub winner_id: Option<u64>,
    pub status: String,
}

/// 队伍对阵信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamBracketInfo {
    pub id: u64,
    pub name: String,
    pub short_name: Option<String>,
    pub region_code: String,
}

/// 瑞士轮状态
#[derive(Debug, Serialize, Deserialize)]
pub struct SwissRoundStatus {
    pub current_round: u32,
    pub teams: Vec<SwissTeamStatus>,
    pub completed: bool,
    pub qualified_teams: Vec<u64>,
    pub eliminated_teams: Vec<u64>,
}

/// 瑞士轮队伍状态
#[derive(Debug, Serialize, Deserialize)]
pub struct SwissTeamStatus {
    pub team_id: u64,
    pub team_name: String,
    pub wins: u32,
    pub losses: u32,
    pub is_qualified: bool,
    pub is_eliminated: bool,
}
