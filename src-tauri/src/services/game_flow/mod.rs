//! 游戏流程服务 - 整合赛季流程控制

mod phase_init;
mod phase_complete;
mod tournament_init;
mod tournament_complete;
mod time_system;
mod match_simulation;
mod annual_awards;
mod market_value;
mod season_management;
pub(crate) mod helpers;

#[cfg(test)]
mod tests;

use crate::services::{HonorService, LeagueService, TournamentService};
use serde::{Deserialize, Serialize};

/// 游戏流程服务 - 整合赛季流程控制
pub struct GameFlowService {
    pub(crate) league_service: LeagueService,
    pub(crate) honor_service: HonorService,
    pub(crate) tournament_service: TournamentService,
}

/// 阶段初始化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseInitResult {
    pub phase: String,
    pub tournaments_created: Vec<TournamentCreated>,
    pub message: String,
}

/// 创建的赛事信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentCreated {
    pub id: u64,
    pub name: String,
    pub tournament_type: String,
    pub region: Option<String>,
}

/// 阶段完成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseCompleteResult {
    pub phase: String,
    pub honors_awarded: Vec<HonorAwarded>,
    pub can_advance: bool,
    pub next_phase: Option<String>,
    pub message: String,
}

/// 颁发的荣誉
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonorAwarded {
    pub honor_type: String,
    pub recipient_name: String,
    pub tournament_name: String,
}

/// 赛季结算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSettlementResult {
    pub season: u32,
    pub players_grown: u32,
    pub players_declined: u32,
    pub players_retired: u32,
    pub contracts_expired: u32,
    pub rookies_generated: u32,
    pub events: Vec<String>,
}

/// 新赛季初始化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSeasonResult {
    pub new_season: u32,
    pub starters_confirmed: u32,
    pub message: String,
}

impl Default for GameFlowService {
    fn default() -> Self {
        Self {
            league_service: LeagueService::new(),
            honor_service: HonorService::new(),
            tournament_service: TournamentService::new(),
        }
    }
}

impl GameFlowService {
    pub fn new() -> Self {
        Self::default()
    }
}
