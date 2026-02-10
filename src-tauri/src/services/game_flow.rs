use crate::db::{
    HonorRepository, MatchRepository, PlayerStatsRepository, PointsRepository, SaveRepository,
    StandingRepository, TeamRepository, TournamentRepository,
};
use crate::engines::{FinancialEngine, PointsCalculationEngine, MetaEngine, MatchSimulationEngine, MatchPlayerInfo, MatchSimContext, TraitType, ConditionEngine, PlayerFormFactors};
use crate::engines::market_value::{MarketValueEngine, PlayerHonorRecord};
use std::collections::HashMap;
use crate::models::{
    HonorType, LeagueStanding, SeasonPhase, Tournament, TournamentStatus,
    TournamentType, GameTimeState, PhaseStatus, PhaseProgress, TournamentProgress,
    SeasonProgress, PhaseInfo, TimeAction, FastForwardTarget, FastForwardResult,
    CompleteAndAdvanceResult, HonorInfo, PlayerSeasonStatistics,
};
use crate::services::{LeagueService, HonorService, TournamentService};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};

/// 游戏流程服务 - 整合赛季流程控制
pub struct GameFlowService {
    league_service: LeagueService,
    honor_service: HonorService,
    tournament_service: TournamentService,
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

    /// 初始化阶段 - 为当前阶段创建对应的赛事
    pub async fn initialize_phase(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        phase: SeasonPhase,
    ) -> Result<PhaseInitResult, String> {
        // 检查该阶段的赛事是否已经存在
        if let Some(tournament_type) = phase.to_tournament_type() {
            let existing = self.get_phase_tournaments(pool, save_id, season_id, tournament_type).await?;
            log::debug!("阶段 {:?}, 已存在赛事数: {}", phase, existing.len());
            if !existing.is_empty() {
                // 检查是否【所有】赛事都有比赛
                // 只有所有赛事都有比赛时才跳过，否则继续初始化（为没有比赛的赛事生成比赛）
                let mut all_have_matches = true;
                let mut total_matches = 0i64;
                for tournament in &existing {
                    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE tournament_id = ?")
                        .bind(tournament.id as i64)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("赛事 {} (id={}) 比赛数: {}", tournament.name, tournament.id, count.0);
                    total_matches += count.0;
                    if count.0 == 0 {
                        all_have_matches = false;
                        // 不要 break，继续统计总数
                    }
                }
                log::debug!("all_have_matches={}, total_matches={}", all_have_matches, total_matches);

                if all_have_matches && total_matches > 0 {
                    log::debug!("所有赛事都有比赛，跳过初始化");
                    // 预加载赛区名称，避免硬编码ID映射
                    let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
                        .bind(save_id)
                        .fetch_all(pool)
                        .await
                        .unwrap_or_default();
                    let region_map: std::collections::HashMap<u64, String> = region_rows.iter()
                        .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
                        .collect();
                    return Ok(PhaseInitResult {
                        phase: format!("{:?}", phase),
                        tournaments_created: existing.iter().map(|t| TournamentCreated {
                            id: t.id,
                            name: t.name.clone(),
                            tournament_type: format!("{:?}", t.tournament_type),
                            region: t.region_id.map(|r| region_map.get(&r).cloned().unwrap_or_else(|| get_region_name(r).to_string())),
                        }).collect(),
                        message: format!("阶段 {:?} 的赛事已存在，跳过初始化", phase),
                    });
                }
                log::debug!("有赛事没有比赛，继续初始化");
                // 有赛事没有比赛，继续生成比赛
            }
        }

        let mut tournaments_created = Vec::new();

        match phase {
            // 春季常规赛 - 为4个赛区各创建一个常规赛赛事
            SeasonPhase::SpringRegular => {
                tournaments_created = self.init_regional_regular_season(
                    pool, save_id, season_id, TournamentType::SpringRegular, "春季赛"
                ).await?;
            }

            // 春季季后赛 - 为4个赛区各创建季后赛
            SeasonPhase::SpringPlayoffs => {
                tournaments_created = self.init_regional_playoffs(
                    pool, save_id, season_id, TournamentType::SpringPlayoffs, "SpringRegular", "春季季后赛"
                ).await?;
            }

            // MSI - 创建全球性赛事并自动生成对阵
            SeasonPhase::Msi => {
                // 检查是否已存在MSI赛事（避免重复创建）
                let existing_msi = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "Msi"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(existing) = existing_msi.into_iter().find(|t| t.status == TournamentStatus::Upcoming) {
                    log::debug!("[MSI Init] 使用已存在的MSI赛事: id={}, name={}", existing.id, existing.name);
                    existing.id
                } else {
                    // 创建新的MSI赛事
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::Msi,
                        name: format!("S{} MSI季中赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[MSI Init] 创建新MSI赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[MSI Init] MSI赛事已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} MSI季中赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::Msi),
                        region: None,
                    });
                } else {
                    // 从春季季后赛获取各赛区的冠亚季军
                    let mut legendary_teams = Vec::new();  // 冠军队伍
                    let mut challenger_teams = Vec::new(); // 亚军队伍
                    let mut qualifier_teams = Vec::new();  // 季军队伍

                    // 直接获取所有春季季后赛，不依赖固定的 region_id
                    let all_playoffs = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SpringPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("[MSI Init] 找到 {} 个春季季后赛赛事", all_playoffs.len());
                    for playoff in &all_playoffs {
                        log::debug!("[MSI Init] 季后赛: id={}, name={}, status={:?}",
                            playoff.id, playoff.name, playoff.status);
                    }

                    for playoff_tournament in all_playoffs {
                        // 获取该季后赛的最终排名
                        let results = self.get_playoffs_top3(pool, playoff_tournament.id).await?;

                        if results.len() >= 3 {
                            legendary_teams.push(results[0].clone());  // 冠军
                            challenger_teams.push(results[1].clone()); // 亚军
                            qualifier_teams.push(results[2].clone());  // 季军
                            log::debug!("从赛事 {} 获取到前3名队伍", playoff_tournament.name);
                        } else {
                            log::debug!("赛事 {} 结果不足3支队伍: {}", playoff_tournament.name, results.len());
                        }
                    }

                    // 如果所有队伍都已就位，生成MSI对阵
                    if legendary_teams.len() == 4 && challenger_teams.len() == 4 && qualifier_teams.len() == 4 {
                        let matches = self.tournament_service.generate_msi_bracket(
                            id,
                            &legendary_teams,
                            &challenger_teams,
                            &qualifier_teams,
                        );

                        MatchRepository::create_batch(pool, save_id, &matches)
                            .await
                            .map_err(|e| e.to_string())?;

                        log::debug!("MSI 生成了 {} 场比赛", matches.len());
                    } else {
                        log::debug!("MSI 队伍不足: legendary={}, challenger={}, qualifier={}",
                            legendary_teams.len(), challenger_teams.len(), qualifier_teams.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} MSI季中赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::Msi),
                        region: None,
                    });
                }
            }

            // 马德里大师赛
            SeasonPhase::MadridMasters => {
                tournaments_created = self.init_32team_masters(
                    pool, save_id, season_id, TournamentType::MadridMasters, "SpringRegular", "马德里大师赛"
                ).await?;
            }

            // 夏季常规赛
            SeasonPhase::SummerRegular => {
                tournaments_created = self.init_regional_regular_season(
                    pool, save_id, season_id, TournamentType::SummerRegular, "夏季赛"
                ).await?;
            }

            // 夏季季后赛
            SeasonPhase::SummerPlayoffs => {
                tournaments_created = self.init_regional_playoffs(
                    pool, save_id, season_id, TournamentType::SummerPlayoffs, "SummerRegular", "夏季季后赛"
                ).await?;
            }

            // Claude洲际赛 (类似马德里大师赛：32队分组+东西半区淘汰)
            SeasonPhase::ClaudeIntercontinental => {
                tournaments_created = self.init_32team_masters(
                    pool, save_id, season_id, TournamentType::ClaudeIntercontinental, "SummerRegular", "Claude洲际赛"
                ).await?;
            }

            // 世界赛
            SeasonPhase::WorldChampionship => {
                // 先检查是否已存在
                let existing = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "WorldChampionship"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(existing_tournament) = existing.first() {
                    log::debug!("赛事已存在, id={}", existing_tournament.id);
                    existing_tournament.id
                } else {
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::WorldChampionship,
                        name: format!("S{} 世界赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?
                };

                // 检查是否已有比赛
                let match_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE tournament_id = ?")
                    .bind(id as i64)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

                if match_count.0 == 0 {
                    // 获取参赛队伍：各赛区夏季季后赛冠军(4队)直接进淘汰赛，亚军+季军(8队)进瑞士轮
                    let mut direct_teams = Vec::new();  // 冠军队伍 - 直接进淘汰赛
                    let mut swiss_teams = Vec::new();   // 亚军+季军队伍 - 进瑞士轮

                    // 从各赛区夏季季后赛获取排名
                    let playoffs_tournaments = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SummerPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("找到 {} 个夏季季后赛", playoffs_tournaments.len());

                    for playoffs in playoffs_tournaments {
                        // 获取季后赛最终排名（从荣誉记录获取）
                        // 冠军
                        let champion_row = sqlx::query(
                            "SELECT team_id FROM honors WHERE tournament_id = ? AND honor_type = 'TEAM_CHAMPION'"
                        )
                        .bind(playoffs.id as i64)
                        .fetch_optional(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                        if let Some(row) = champion_row {
                            let team_id: i64 = row.get("team_id");
                            if let Ok(team) = TeamRepository::get_by_id(pool, team_id as u64).await {
                                if !direct_teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                                    direct_teams.push(team);
                                    log::debug!("赛区 {:?} 冠军: team_id={}", playoffs.region_id, team_id);
                                }
                            }
                        }

                        // 亚军
                        let runner_up_row = sqlx::query(
                            "SELECT team_id FROM honors WHERE tournament_id = ? AND honor_type = 'TEAM_RUNNER_UP'"
                        )
                        .bind(playoffs.id as i64)
                        .fetch_optional(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                        if let Some(row) = runner_up_row {
                            let team_id: i64 = row.get("team_id");
                            if let Ok(team) = TeamRepository::get_by_id(pool, team_id as u64).await {
                                if !swiss_teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                                    swiss_teams.push(team);
                                    log::debug!("赛区 {:?} 亚军: team_id={}", playoffs.region_id, team_id);
                                }
                            }
                        }

                        // 季军
                        let third_row = sqlx::query(
                            "SELECT team_id FROM honors WHERE tournament_id = ? AND honor_type = 'TEAM_THIRD'"
                        )
                        .bind(playoffs.id as i64)
                        .fetch_optional(pool)
                        .await
                        .map_err(|e| e.to_string())?;

                        if let Some(row) = third_row {
                            let team_id: i64 = row.get("team_id");
                            if let Ok(team) = TeamRepository::get_by_id(pool, team_id as u64).await {
                                if !swiss_teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                                    swiss_teams.push(team);
                                    log::debug!("赛区 {:?} 季军: team_id={}", playoffs.region_id, team_id);
                                }
                            }
                        }
                    }

                    log::debug!("冠军队伍: {} 支, 瑞士轮队伍: {} 支", direct_teams.len(), swiss_teams.len());

                    // 如果队伍不足，从夏季常规赛积分榜补充
                    if direct_teams.len() < 4 || swiss_teams.len() < 8 {
                        log::debug!("队伍不足，从常规赛积分榜补充");
                        let regular_tournaments = TournamentRepository::get_by_season_and_type(
                            pool, save_id, season_id, "SummerRegular"
                        ).await.map_err(|e| e.to_string())?;

                        for regular in regular_tournaments {
                            let standings = StandingRepository::get_by_tournament(pool, regular.id)
                                .await
                                .map_err(|e| e.to_string())?;

                            for (idx, standing) in standings.iter().enumerate() {
                                if let Ok(team) = TeamRepository::get_by_id(pool, standing.team_id).await {
                                    // 前1名补充到直接晋级
                                    if idx == 0 && direct_teams.len() < 4 {
                                        if !direct_teams.iter().any(|t| t.id == team.id) && !swiss_teams.iter().any(|t| t.id == team.id) {
                                            direct_teams.push(team);
                                            continue;
                                        }
                                    }
                                    // 2-3名补充到瑞士轮
                                    if idx < 3 && swiss_teams.len() < 8 {
                                        if !direct_teams.iter().any(|t| t.id == team.id) && !swiss_teams.iter().any(|t| t.id == team.id) {
                                            swiss_teams.push(team.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    log::debug!("最终: 冠军队伍 {} 支, 瑞士轮队伍 {} 支", direct_teams.len(), swiss_teams.len());

                    if direct_teams.len() >= 4 && swiss_teams.len() >= 8 {
                        let matches = self.tournament_service.generate_worlds_bracket(
                            id,
                            &direct_teams[..4],
                            &swiss_teams[..8]
                        );
                        log::debug!("生成 {} 场比赛", matches.len());
                        if !matches.is_empty() {
                            MatchRepository::create_batch(pool, save_id, &matches)
                                .await
                                .map_err(|e| e.to_string())?;

                            // 初始化积分榜（瑞士轮阶段需要）
                            let mut all_teams = swiss_teams.clone();
                            all_teams.extend(direct_teams.clone());
                            let standings: Vec<LeagueStanding> = all_teams
                                .iter()
                                .map(|team| LeagueStanding {
                                    id: 0,
                                    tournament_id: id,
                                    team_id: team.id,
                                    rank: None,
                                    matches_played: 0,
                                    wins: 0,
                                    losses: 0,
                                    points: 0,
                                    games_won: 0,
                                    games_lost: 0,
                                    game_diff: 0,
                                })
                                .collect();
                            StandingRepository::upsert_batch(pool, save_id, &standings)
                                .await
                                .map_err(|e| e.to_string())?;
                            log::debug!("初始化 {} 支队伍的积分榜", standings.len());
                        }
                    } else {
                        log::debug!("队伍不足，无法生成比赛: direct={}, swiss={}", direct_teams.len(), swiss_teams.len());
                    }
                }

                tournaments_created.push(TournamentCreated {
                    id,
                    name: format!("S{} 世界赛", season_id),
                    tournament_type: format!("{:?}", TournamentType::WorldChampionship),
                    region: None,
                });
            }

            // 上海大师赛 - 与MSI相同的双败淘汰赛制，参赛队伍来自夏季季后赛
            SeasonPhase::ShanghaiMasters => {
                // 检查是否已存在上海大师赛赛事（避免重复创建）
                let existing_shanghai = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "ShanghaiMasters"
                ).await.map_err(|e| e.to_string())?;

                // 清理重复的上海大师赛赛事 (保留第一个有效的，删除其他的)
                let valid_tournaments: Vec<_> = existing_shanghai.into_iter()
                    .filter(|t| t.status == TournamentStatus::Upcoming || t.status == TournamentStatus::InProgress)
                    .collect();

                // 获取第一个有效赛事的ID（如果存在）
                let first_valid_id = valid_tournaments.first().map(|t| t.id);

                // 如果有多个，删除多余的
                if valid_tournaments.len() > 1 {
                    log::debug!("[ShanghaiMasters Init] 发现 {} 个重复的上海大师赛，清理中...", valid_tournaments.len());
                    // 保留第一个，删除其他
                    for extra_tournament in valid_tournaments.iter().skip(1) {
                        log::debug!("[ShanghaiMasters Init] 删除重复赛事: id={}", extra_tournament.id);
                        // 先删除比赛
                        sqlx::query("DELETE FROM matches WHERE tournament_id = ?")
                            .bind(extra_tournament.id as i64)
                            .execute(pool)
                            .await
                            .ok();
                        // 删除赛事
                        sqlx::query("DELETE FROM tournaments WHERE id = ?")
                            .bind(extra_tournament.id as i64)
                            .execute(pool)
                            .await
                            .ok();
                    }
                }

                let id = if let Some(existing_id) = first_valid_id {
                    log::debug!("[ShanghaiMasters Init] 使用已存在的上海大师赛赛事: id={}", existing_id);
                    existing_id
                } else {
                    // 创建新的上海大师赛赛事
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::ShanghaiMasters,
                        name: format!("S{} 上海大师赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[ShanghaiMasters Init] 创建新上海大师赛赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[ShanghaiMasters Init] 上海大师赛已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} 上海大师赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::ShanghaiMasters),
                        region: None,
                    });
                } else {
                    // 从夏季季后赛获取各赛区的冠亚季军
                    let mut legendary_teams = Vec::new();  // 冠军队伍
                    let mut challenger_teams = Vec::new(); // 亚军队伍
                    let mut qualifier_teams = Vec::new();  // 季军队伍

                    // 获取所有夏季季后赛
                    let all_playoffs = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SummerPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("[ShanghaiMasters Init] 找到 {} 个夏季季后赛赛事", all_playoffs.len());
                    for playoff in &all_playoffs {
                        log::debug!("[ShanghaiMasters Init] 季后赛: id={}, name={}, status={:?}",
                            playoff.id, playoff.name, playoff.status);
                    }

                    for playoff_tournament in all_playoffs {
                        // 获取该季后赛的最终排名
                        let results = self.get_playoffs_top3(pool, playoff_tournament.id).await?;

                        if results.len() >= 3 {
                            legendary_teams.push(results[0].clone());  // 冠军
                            challenger_teams.push(results[1].clone()); // 亚军
                            qualifier_teams.push(results[2].clone());  // 季军
                            log::debug!("从赛事 {} 获取到前3名队伍", playoff_tournament.name);
                        } else {
                            log::debug!("赛事 {} 结果不足3支队伍: {}", playoff_tournament.name, results.len());
                        }
                    }

                    // 如果所有队伍都已就位，生成上海大师赛对阵（与MSI相同格式）
                    if legendary_teams.len() == 4 && challenger_teams.len() == 4 && qualifier_teams.len() == 4 {
                        let matches = self.tournament_service.generate_msi_bracket(
                            id,
                            &legendary_teams,
                            &challenger_teams,
                            &qualifier_teams,
                        );

                        if !matches.is_empty() {
                            MatchRepository::create_batch(pool, save_id, &matches)
                                .await
                                .map_err(|e| e.to_string())?;

                            // 更新赛事状态为进行中
                            TournamentRepository::update_status(pool, id, TournamentStatus::InProgress)
                                .await
                                .map_err(|e| e.to_string())?;

                            log::debug!("成功生成 {} 场比赛并更新状态为进行中", matches.len());
                        }
                    } else {
                        log::debug!("队伍不足: legendary={}, challenger={}, qualifier={}",
                            legendary_teams.len(), challenger_teams.len(), qualifier_teams.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} 上海大师赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::ShanghaiMasters),
                        region: None,
                    });
                }
            }

            // ICP洲际对抗赛
            SeasonPhase::IcpIntercontinental => {
                // 检查是否已存在ICP赛事
                let existing = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "IcpIntercontinental"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(t) = existing.first() {
                    log::debug!("[ICP Init] 找到已存在的ICP赛事: id={}", t.id);
                    t.id
                } else {
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::IcpIntercontinental,
                        name: format!("S{} ICP洲际对抗赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[ICP Init] 创建新ICP赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[ICP Init] ICP赛事已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} ICP洲际对抗赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::IcpIntercontinental),
                        region: None,
                    });
                } else {
                    // 从夏季季后赛获取各赛区的前4名
                    let mut region_teams: Vec<Vec<crate::models::Team>> = Vec::new();

                    // 获取所有夏季季后赛
                    let all_playoffs = TournamentRepository::get_by_season_and_type(
                        pool, save_id, season_id, "SummerPlayoffs"
                    ).await.map_err(|e| e.to_string())?;

                    log::debug!("[ICP Init] 找到 {} 个夏季季后赛赛事", all_playoffs.len());

                    for playoff_tournament in all_playoffs {
                        // 获取该季后赛的前4名
                        let results = self.get_playoffs_top4(pool, playoff_tournament.id).await?;

                        if results.len() >= 4 {
                            region_teams.push(results);
                            log::debug!("[ICP Init] 从赛事 {} 获取到前4名队伍", playoff_tournament.name);
                        } else {
                            log::debug!("[ICP Init] 赛事 {} 结果不足4支队伍: {}", playoff_tournament.name, results.len());
                        }
                    }

                    // 如果所有4个赛区的前4名都已就位，生成ICP对阵
                    if region_teams.len() == 4 {
                        let matches = self.tournament_service.generate_icp_bracket(
                            id,
                            region_teams,
                        );

                        if !matches.is_empty() {
                            MatchRepository::create_batch(pool, save_id, &matches)
                                .await
                                .map_err(|e| e.to_string())?;

                            // 更新赛事状态为进行中
                            TournamentRepository::update_status(pool, id, TournamentStatus::InProgress)
                                .await
                                .map_err(|e| e.to_string())?;

                            log::debug!("[ICP Init] 成功生成 {} 场比赛并更新状态为进行中", matches.len());
                        }
                    } else {
                        log::debug!("[ICP Init] 赛区队伍不足: 需要4个赛区各4队，当前有 {} 个赛区数据", region_teams.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} ICP洲际对抗赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::IcpIntercontinental),
                        region: None,
                    });
                }
            }

            // Super洲际邀请赛
            SeasonPhase::SuperIntercontinental => {
                // 检查是否已存在Super赛事
                let existing = TournamentRepository::get_by_season_and_type(
                    pool, save_id, season_id, "SuperIntercontinental"
                ).await.map_err(|e| e.to_string())?;

                let id = if let Some(t) = existing.first() {
                    log::debug!("[Super Init] 找到已存在的Super赛事: id={}", t.id);
                    t.id
                } else {
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::SuperIntercontinental,
                        name: format!("S{} Super洲际邀请赛", season_id),
                        region_id: None,
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let new_id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("[Super Init] 创建新Super赛事: id={}", new_id);
                    new_id
                };

                // 检查是否已有比赛（避免重复生成）
                let existing_matches = MatchRepository::get_by_tournament(pool, id)
                    .await
                    .map_err(|e| e.to_string())?;

                if existing_matches.len() > 0 {
                    log::debug!("[Super Init] Super赛事已有 {} 场比赛，跳过生成", existing_matches.len());
                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} Super洲际邀请赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::SuperIntercontinental),
                        region: None,
                    });
                } else {
                    // 从年度积分获取Top16队伍
                    let rankings = PointsRepository::get_season_rankings(pool, save_id, season_id)
                        .await
                        .map_err(|e| e.to_string())?;

                    log::debug!("[Super Init] 年度积分排名队伍数: {}", rankings.len());

                    if rankings.len() >= 16 {
                        // 获取队伍信息
                        let mut legendary_teams = Vec::new();
                        let mut challenger_teams = Vec::new();
                        let mut fighter_teams = Vec::new();

                        for (idx, ranking) in rankings.iter().take(16).enumerate() {
                            if let Ok(team) = TeamRepository::get_by_id(pool, ranking.team_id).await {
                                match idx {
                                    0..=3 => legendary_teams.push(team),    // 1-4名: 传奇组
                                    4..=7 => challenger_teams.push(team),   // 5-8名: 挑战者组
                                    8..=15 => fighter_teams.push(team),     // 9-16名: Fighter组
                                    _ => {}
                                }
                            }
                        }

                        log::debug!("[Super Init] 传奇组: {} 队, 挑战者组: {} 队, Fighter组: {} 队",
                            legendary_teams.len(), challenger_teams.len(), fighter_teams.len());

                        if legendary_teams.len() == 4 && challenger_teams.len() == 4 && fighter_teams.len() == 8 {
                            let matches = self.tournament_service.generate_super_bracket(
                                id,
                                &legendary_teams,
                                &challenger_teams,
                                &fighter_teams,
                            );

                            if !matches.is_empty() {
                                MatchRepository::create_batch(pool, save_id, &matches)
                                    .await
                                    .map_err(|e| e.to_string())?;

                                // 更新赛事状态为进行中
                                TournamentRepository::update_status(pool, id, TournamentStatus::InProgress)
                                    .await
                                    .map_err(|e| e.to_string())?;

                                log::debug!("[Super Init] 成功生成 {} 场比赛并更新状态为进行中", matches.len());
                            }
                        } else {
                            log::debug!("[Super Init] 队伍数量不足: 传奇组需4队，挑战者组需4队，Fighter组需8队");
                        }
                    } else {
                        log::debug!("[Super Init] 年度积分排名不足16队: 当前 {} 队", rankings.len());
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: format!("S{} Super洲际邀请赛", season_id),
                        tournament_type: format!("{:?}", TournamentType::SuperIntercontinental),
                        region: None,
                    });
                }
            }

            // 年度颁奖典礼 - 不创建赛事，仅显示页面
            SeasonPhase::AnnualAwards => {}

            // 转会期 - 不创建赛事
            SeasonPhase::TransferWindow => {}

            // 选秀 - 不创建赛事
            SeasonPhase::Draft => {}

            // 赛季结束 - 不创建赛事
            SeasonPhase::SeasonEnd => {}
        }

        let message = if tournaments_created.is_empty() {
            format!("{:?} 阶段无需创建赛事", phase)
        } else {
            format!(
                "成功创建 {} 个赛事",
                tournaments_created.len()
            )
        };

        Ok(PhaseInitResult {
            phase: format!("{:?}", phase),
            tournaments_created,
            message,
        })
    }

    /// 检查阶段是否完成
    pub async fn check_phase_completion(
        &self,
        _pool: &Pool<Sqlite>,
        _save_id: &str,
        _season_id: u64,
        phase: SeasonPhase,
    ) -> Result<bool, String> {
        match phase {
            SeasonPhase::AnnualAwards | SeasonPhase::TransferWindow | SeasonPhase::Draft | SeasonPhase::SeasonEnd => {
                // 这些阶段需要手动确认完成
                Ok(false)
            }
            _ => {
                // 检查该阶段所有赛事是否完成
                let tournament_type = phase.to_tournament_type();
                if let Some(_t_type) = tournament_type {
                    // 查询该类型的所有赛事
                    // 检查是否有待进行的比赛
                    // 简化实现：返回false让用户手动推进
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
    }

    /// 完成阶段 - 颁发荣誉并准备下一阶段
    pub async fn complete_phase(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        phase: SeasonPhase,
    ) -> Result<PhaseCompleteResult, String> {
        log::debug!("开始处理阶段: {:?}, season_id={}", phase, season_id);

        // 检查该阶段是否已经完成过（详情页可能已提前处理荣誉和积分）
        // 即使已完成，仍然继续执行：读取已有荣誉、补发奖金和统计等
        let _phase_already_completed = if let Some(t_type) = phase.to_tournament_type() {
            let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
            if !tournaments.is_empty() {
                let all_completed = tournaments.iter().all(|t| t.status == TournamentStatus::Completed);
                if all_completed {
                    log::info!("阶段 {:?} 赛事已标记为完成，将读取已有荣誉并补发奖金/统计", phase);
                }
                all_completed
            } else {
                false
            }
        } else {
            false
        };

        let mut honors_awarded = Vec::new();

        // 根据阶段颁发荣誉
        match phase {
            // 常规赛结束 - 颁发常规赛第一名和常规赛MVP
            SeasonPhase::SpringRegular | SeasonPhase::SummerRegular => {
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    // 获取该阶段的所有赛事
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    log::debug!("找到 {} 个常规赛赛事", tournaments.len());

                    for tournament in tournaments {
                        log::debug!("处理常规赛荣誉: tournament_id={}", tournament.id);
                        match self.honor_service.process_regular_season_honors(pool, save_id, tournament.id).await {
                            Ok(honors) => {
                                log::debug!("常规赛荣誉处理成功，获得 {} 个荣誉", honors.len());
                                for honor in honors {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: format!("{:?}", honor.honor_type),
                                        recipient_name: honor.team_name.clone()
                                            .or(honor.player_name.clone())
                                            .unwrap_or_else(|| "Unknown".to_string()),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to process regular season honors for tournament {}: {}", tournament.id, e);
                            }
                        }
                    }
                }
            }

            // 季后赛/国际赛事结束 - 颁发冠亚季军、MVP等荣誉
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                let tournament_type = phase.to_tournament_type();
                log::debug!("处理季后赛/国际赛事: {:?}, tournament_type={:?}", phase, tournament_type);
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    log::debug!("找到 {} 个赛事", tournaments.len());

                    for tournament in &tournaments {
                        log::debug!("处理赛事荣誉: id={}, name={}", tournament.id, tournament.name);
                        match self.honor_service.process_tournament_completion(pool, save_id, tournament.id).await {
                            Ok(_) => {
                                // 从数据库查询该赛事的所有荣誉（支持ICP等多队伍同名次的赛事）
                                match HonorRepository::get_by_tournament(pool, save_id, tournament.id).await {
                                    Ok(all_honors) => {
                                        log::debug!("从数据库获取到 {} 个荣誉", all_honors.len());
                                        // 只收集战队荣誉和MVP用于显示（选手荣誉太多）
                                        for honor in all_honors {
                                            match honor.honor_type {
                                                // 战队荣誉
                                                HonorType::TeamChampion => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "冠军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::TeamRunnerUp => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "亚军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::TeamThird => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "季军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::TeamFourth => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "殿军".to_string(),
                                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                // MVP荣誉
                                                HonorType::TournamentMvp => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "赛事MVP".to_string(),
                                                        recipient_name: honor.player_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                HonorType::FinalsMvp => {
                                                    honors_awarded.push(HonorAwarded {
                                                        honor_type: "决赛MVP".to_string(),
                                                        recipient_name: honor.player_name.clone().unwrap_or_default(),
                                                        tournament_name: honor.tournament_name.clone(),
                                                    });
                                                }
                                                // 其他荣誉类型（选手荣誉等）不显示
                                                _ => {}
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::debug!("获取荣誉列表失败: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::debug!("荣誉处理失败: tournament_id={}, error={}", tournament.id, e);
                            }
                        }
                    }
                }
            }

            // 年度颁奖典礼 - 颁发年度荣誉
            SeasonPhase::AnnualAwards => {
                log::debug!("处理年度颁奖典礼");

                // 获取年度Top20选手并颁发荣誉
                let top20 = self.get_annual_top20(pool, save_id, season_id).await?;
                log::debug!("获取到 {} 位Top20选手", top20.len());

                for (idx, player) in top20.iter().enumerate() {
                    // 第一名同时获得年度MVP
                    if idx == 0 {
                        let mvp_honor = crate::models::Honor::new_player_honor(
                            save_id,
                            HonorType::AnnualMvp,
                            season_id,
                            None,
                            "年度颁奖典礼",
                            "ANNUAL",
                            player.team_id,
                            &player.team_name,
                            player.player_id,
                            &player.player_name,
                            &player.position,
                            None,
                        );
                        if let Err(e) = HonorRepository::create(pool, save_id, &mvp_honor).await {
                            log::error!("Failed to create annual MVP honor: {}", e);
                        } else {
                            honors_awarded.push(HonorAwarded {
                                honor_type: "年度MVP".to_string(),
                                recipient_name: player.player_name.clone(),
                                tournament_name: "年度颁奖典礼".to_string(),
                            });
                        }
                    }

                    // Top20荣誉
                    let rank = idx + 1;
                    let top20_honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualTop20,
                        season_id,
                        None,
                        &format!("年度Top{}", rank),
                        "ANNUAL",
                        player.team_id,
                        &player.team_name,
                        player.player_id,
                        &player.player_name,
                        &player.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &top20_honor).await {
                        log::error!("Failed to create annual top20 honor for player {}: {}", player.player_id, e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: format!("年度Top{}", idx + 1),
                            recipient_name: player.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取三阵选手并颁发荣誉
                let all_pro_3teams = self.get_annual_all_pro_3teams(pool, save_id, season_id).await?;
                log::debug!("获取到 {} 位最佳阵容选手", all_pro_3teams.len());

                for (tier, player) in &all_pro_3teams {
                    let honor_type = match tier {
                        1 => HonorType::AnnualAllPro1st,
                        2 => HonorType::AnnualAllPro2nd,
                        3 => HonorType::AnnualAllPro3rd,
                        _ => continue,
                    };

                    let tier_name = match tier {
                        1 => "最佳阵容一阵",
                        2 => "最佳阵容二阵",
                        3 => "最佳阵容三阵",
                        _ => continue,
                    };

                    let position_honor = crate::models::Honor::new_player_honor(
                        save_id,
                        honor_type,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        player.team_id,
                        &player.team_name,
                        player.player_id,
                        &player.player_name,
                        &player.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &position_honor).await {
                        log::error!("Failed to create annual all-pro honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: tier_name.to_string(),
                            recipient_name: player.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取最稳定选手
                if let Ok(Some(consistent)) = self.get_annual_most_consistent(pool, save_id, season_id).await {
                    log::debug!("年度最稳定选手: {}", consistent.player_name);

                    let honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualMostConsistent,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        consistent.team_id,
                        &consistent.team_name,
                        consistent.player_id,
                        &consistent.player_name,
                        &consistent.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &honor).await {
                        log::error!("Failed to create annual most consistent honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: "年度最稳定选手".to_string(),
                            recipient_name: consistent.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取最具统治力选手
                if let Ok(Some(dominant)) = self.get_annual_most_dominant(pool, save_id, season_id).await {
                    log::debug!("年度最具统治力选手: {}", dominant.player_name);

                    let honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualMostDominant,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        dominant.team_id,
                        &dominant.team_name,
                        dominant.player_id,
                        &dominant.player_name,
                        &dominant.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &honor).await {
                        log::error!("Failed to create annual most dominant honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: "年度最具统治力".to_string(),
                            recipient_name: dominant.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }

                // 获取年度最佳新秀
                if let Ok(Some(rookie)) = self.get_annual_rookie(pool, save_id, season_id).await {
                    log::debug!("年度最佳新秀: {}", rookie.player_name);

                    let rookie_honor = crate::models::Honor::new_player_honor(
                        save_id,
                        HonorType::AnnualRookie,
                        season_id,
                        None,
                        "年度颁奖典礼",
                        "ANNUAL",
                        rookie.team_id,
                        &rookie.team_name,
                        rookie.player_id,
                        &rookie.player_name,
                        &rookie.position,
                        None,
                    );
                    if let Err(e) = HonorRepository::create(pool, save_id, &rookie_honor).await {
                        log::error!("Failed to create annual rookie honor: {}", e);
                    } else {
                        honors_awarded.push(HonorAwarded {
                            honor_type: "年度最佳新秀".to_string(),
                            recipient_name: rookie.player_name.clone(),
                            tournament_name: "年度颁奖典礼".to_string(),
                        });
                    }
                }
            }

            _ => {}
        }

        log::debug!("荣誉处理完成，共 {} 个荣誉", honors_awarded.len());

        // 年度颁奖典礼后重算所有选手身价（考虑累积荣誉）
        if phase == SeasonPhase::AnnualAwards {
            log::debug!("开始年度身价重算...");
            match self.recalculate_all_market_values(pool, save_id, season_id).await {
                Ok(count) => log::debug!("年度身价重算完成，共更新 {} 名选手", count),
                Err(e) => log::error!("[complete_phase] 年度身价重算失败: {}", e),
            }

            // 更新所有队伍的品牌价值
            log::debug!("开始更新品牌价值...");
            match self.update_all_brand_values(pool, save_id, season_id).await {
                Ok(count) => log::debug!("品牌价值更新完成，共更新 {} 支队伍", count),
                Err(e) => log::error!("[complete_phase] 品牌价值更新失败: {}", e),
            }
        }

        // 颁发年度积分（季后赛和国际赛事，Super赛除外）
        // Super赛是年度积分的奖励，不颁发积分
        match phase {
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental => {
                log::debug!("颁发年度积分: {:?}", phase);
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    for tournament in &tournaments {
                        log::debug!("为赛事 {} 颁发年度积分", tournament.id);
                        match self.award_tournament_points(pool, save_id, season_id, tournament.id, t_type).await {
                            Ok(awarded) => log::debug!("积分颁发成功: {:?}", awarded),
                            Err(e) => log::debug!("积分颁发失败: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }

        // 发放赛事奖金（季后赛和国际赛事）
        match phase {
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                log::debug!("发放赛事奖金: {:?}", phase);
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    for tournament in &tournaments {
                        log::debug!("为赛事 {} 发放奖金", tournament.id);
                        match self.distribute_tournament_prizes(pool, save_id, season_id, tournament.id, t_type).await {
                            Ok(distributed) => log::debug!("奖金发放成功: {:?}", distributed),
                            Err(e) => log::debug!("奖金发放失败: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }

        // 更新冠军选手的统计数据（增加冠军次数）
        match phase {
            SeasonPhase::SpringPlayoffs
            | SeasonPhase::SummerPlayoffs
            | SeasonPhase::Msi
            | SeasonPhase::MadridMasters
            | SeasonPhase::ClaudeIntercontinental
            | SeasonPhase::WorldChampionship
            | SeasonPhase::ShanghaiMasters
            | SeasonPhase::IcpIntercontinental
            | SeasonPhase::SuperIntercontinental => {
                let tournament_type = phase.to_tournament_type();
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
                    let is_international = matches!(t_type,
                        TournamentType::Msi |
                        TournamentType::MadridMasters |
                        TournamentType::ClaudeIntercontinental |
                        TournamentType::WorldChampionship |
                        TournamentType::ShanghaiMasters |
                        TournamentType::IcpIntercontinental |
                        TournamentType::SuperIntercontinental
                    );
                    for tournament in tournaments {
                        if let Err(e) = self.update_champion_player_stats(pool, save_id, season_id, tournament.id, t_type, is_international).await {
                            log::error!("Failed to update champion stats for tournament {}: {}", tournament.id, e);
                        }
                    }
                }
            }
            _ => {}
        }

        // 更新赛事状态为已完成
        let tournament_type = phase.to_tournament_type();
        if let Some(t_type) = tournament_type {
            let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;
            for tournament in &tournaments {
                let _ = sqlx::query("UPDATE tournaments SET status = 'Completed' WHERE id = ?")
                    .bind(tournament.id as i64)
                    .execute(pool)
                    .await;
                log::debug!("更新赛事 {} (id={}) 状态为 Completed", tournament.name, tournament.id);
            }
        }

        // 获取下一阶段
        let next_phase = phase.next();
        let can_advance = next_phase.is_some() || phase == SeasonPhase::SeasonEnd;

        let message = match phase {
            SeasonPhase::SeasonEnd => "赛季结束，准备开始新赛季".to_string(),
            _ => {
                if let Some(next) = next_phase {
                    format!("阶段完成，下一阶段: {}", next.display_name())
                } else {
                    "阶段完成".to_string()
                }
            }
        };

        Ok(PhaseCompleteResult {
            phase: format!("{:?}", phase),
            honors_awarded,
            can_advance,
            next_phase: next_phase.map(|p| format!("{:?}", p)),
            message,
        })
    }

    /// 获取阶段对应的赛事
    async fn get_phase_tournaments(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<Tournament>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM tournaments
            WHERE save_id = ? AND season_id = ? AND tournament_type = ?
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(format!("{:?}", tournament_type))
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get tournaments: {}", e))?;

        Ok(rows.iter().map(|row| {
            Tournament {
                id: row.get::<i64, _>("id") as u64,
                save_id: row.get("save_id"),
                season_id: row.get::<i64, _>("season_id") as u64,
                tournament_type,
                name: row.get("name"),
                region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
                status: parse_tournament_status_local(row.get("status")),
                current_stage: None,
                current_round: None,
            }
        }).collect())
    }

    /// 获取所有赛区 ID
    async fn get_all_region_ids(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<Vec<u64>, String> {
        let rows = sqlx::query_as::<_, (i64,)>(
            "SELECT DISTINCT region_id FROM teams WHERE save_id = ? ORDER BY region_id"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows.iter().map(|r| r.0 as u64).collect())
    }

    /// 统计赛事的比赛数
    async fn count_tournament_matches(&self, pool: &Pool<Sqlite>, tournament_id: u64) -> Result<i64, String> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE tournament_id = ?")
            .bind(tournament_id as i64)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(count.0)
    }

    /// 初始化赛区常规赛（春季 / 夏季）
    async fn init_regional_regular_season(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
        season_label: &str,
    ) -> Result<Vec<TournamentCreated>, String> {
        let type_str = format!("{:?}", tournament_type);
        let mut tournaments_created = Vec::new();

        // 获取已存在的该类型赛事
        let existing_tournaments = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, &type_str
        ).await.map_err(|e| e.to_string())?;
        log::debug!("已存在的{}数量: {}", season_label, existing_tournaments.len());

        // 获取所有赛区ID
        let region_ids = self.get_all_region_ids(pool, save_id).await?;
        log::debug!("实际赛区ID: {:?}", region_ids);

        // 预加载赛区名称映射
        let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
        let region_name_map: std::collections::HashMap<u64, String> = region_rows.iter()
            .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
            .collect();

        for region_id in region_ids {
            let region_name = region_name_map.get(&region_id).map(|s| s.as_str()).unwrap_or(get_region_name(region_id));
            log::debug!("处理赛区: {} (region_id={})", region_name, region_id);

            // 检查该赛区的赛事是否已存在
            let existing = existing_tournaments.iter()
                .find(|t| t.region_id == Some(region_id));

            let id = if let Some(t) = existing {
                log::debug!("{} {}已存在, id={}", region_name, season_label, t.id);
                t.id
            } else {
                log::debug!("{} {}不存在，创建新赛事", region_name, season_label);
                let tournament = Tournament {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    tournament_type,
                    name: format!("S{} {} {}", season_id, region_name, season_label),
                    region_id: Some(region_id),
                    status: TournamentStatus::Upcoming,
                    current_stage: None,
                    current_round: None,
                };

                TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?
            };

            // 检查是否已有比赛
            let match_count = self.count_tournament_matches(pool, id).await?;
            log::debug!("{} {} id={} 已有比赛数: {}", region_name, season_label, id, match_count);

            if match_count == 0 {
                let teams = TeamRepository::get_by_region(pool, save_id, region_id)
                    .await
                    .map_err(|e| e.to_string())?;

                if teams.len() >= 8 {
                    let matches = self.league_service.generate_regular_schedule(id, &teams);
                    log::debug!("{} 生成比赛数: {}", region_name, matches.len());
                    MatchRepository::create_batch(pool, save_id, &matches)
                        .await
                        .map_err(|e| e.to_string())?;

                    let standings: Vec<LeagueStanding> = teams
                        .iter()
                        .map(|team| LeagueStanding {
                            id: 0,
                            tournament_id: id,
                            team_id: team.id,
                            rank: None,
                            matches_played: 0,
                            wins: 0,
                            losses: 0,
                            points: 0,
                            games_won: 0,
                            games_lost: 0,
                            game_diff: 0,
                        })
                        .collect();

                    StandingRepository::upsert_batch(pool, save_id, &standings)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 生成比赛后将赛事状态更新为 InProgress
                    sqlx::query("UPDATE tournaments SET status = 'InProgress' WHERE id = ?")
                        .bind(id as i64)
                        .execute(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("{} {} id={} 状态更新为 InProgress", region_name, season_label, id);
                }
            }

            tournaments_created.push(TournamentCreated {
                id,
                name: format!("S{} {} {}", season_id, region_name, season_label),
                tournament_type: type_str.clone(),
                region: Some(region_name.to_string()),
            });
        }

        Ok(tournaments_created)
    }

    /// 初始化赛区季后赛（春季 / 夏季）
    async fn init_regional_playoffs(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        playoff_type: TournamentType,
        regular_type_str: &str,
        season_label: &str,
    ) -> Result<Vec<TournamentCreated>, String> {
        let type_str = format!("{:?}", playoff_type);
        let mut tournaments_created = Vec::new();

        // 获取已存在的季后赛赛事
        let existing_playoffs = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, &type_str
        ).await.map_err(|e| e.to_string())?;
        log::debug!("已存在的{}数量: {}", season_label, existing_playoffs.len());

        // 从常规赛获取实际的 region_id
        let regular_tournaments = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, regular_type_str
        ).await.map_err(|e| e.to_string())?;
        log::debug!("常规赛数量: {}", regular_tournaments.len());

        // 预加载赛区名称映射
        let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
        let region_name_map: std::collections::HashMap<u64, String> = region_rows.iter()
            .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
            .collect();

        for regular_tournament in &regular_tournaments {
            let region_id = match regular_tournament.region_id {
                Some(id) => id,
                None => continue,
            };
            let region_name = region_name_map.get(&region_id).map(|s| s.as_str()).unwrap_or(get_region_name(region_id));
            log::debug!("处理赛区: {} (region_id={})", region_name, region_id);

            // 检查该赛区的季后赛是否已存在
            let existing = existing_playoffs.iter()
                .find(|t| t.region_id == Some(region_id));

            let id = if let Some(t) = existing {
                log::debug!("{} {}已存在, id={}", region_name, season_label, t.id);
                t.id
            } else {
                log::debug!("{} {}不存在，创建新赛事", region_name, season_label);
                let tournament = Tournament {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    tournament_type: playoff_type,
                    name: format!("S{} {} {}", season_id, region_name, season_label),
                    region_id: Some(region_id),
                    status: TournamentStatus::Upcoming,
                    current_stage: None,
                    current_round: None,
                };
                TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?
            };

            // 检查是否已有比赛
            let match_count = self.count_tournament_matches(pool, id).await?;
            log::debug!("{} {} id={} 已有比赛数: {}", region_name, season_label, id, match_count);

            if match_count == 0 {
                let standings = StandingRepository::get_by_tournament(pool, regular_tournament.id)
                    .await
                    .map_err(|e| e.to_string())?;
                log::debug!("{} 常规赛积分榜队伍数: {}", region_name, standings.len());

                if standings.len() >= 8 {
                    let matches = self.league_service.generate_playoff_bracket(id, &standings);
                    log::debug!("{} 生成比赛数: {}", region_name, matches.len());
                    if !matches.is_empty() {
                        MatchRepository::create_batch(pool, save_id, &matches)
                            .await
                            .map_err(|e| e.to_string())?;
                        log::debug!("{} 比赛已创建", region_name);
                    }
                } else {
                    log::debug!("{} 积分榜队伍不足8支，跳过", region_name);
                }
            } else {
                log::debug!("{} 已有比赛，跳过生成", region_name);
            }

            tournaments_created.push(TournamentCreated {
                id,
                name: format!("S{} {} {}", season_id, region_name, season_label),
                tournament_type: type_str.clone(),
                region: Some(region_name.to_string()),
            });
        }

        Ok(tournaments_created)
    }

    /// 初始化 32 队大师赛（马德里大师赛 / Claude洲际赛）
    async fn init_32team_masters(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: TournamentType,
        source_regular: &str,
        tournament_name: &str,
    ) -> Result<Vec<TournamentCreated>, String> {
        let type_str = format!("{:?}", tournament_type);
        let mut tournaments_created = Vec::new();

        // 先检查是否已存在
        let existing = TournamentRepository::get_by_season_and_type(
            pool, save_id, season_id, &type_str
        ).await.map_err(|e| e.to_string())?;

        let id = if let Some(t) = existing.first() {
            log::debug!("赛事已存在, id={}", t.id);
            t.id
        } else {
            let tournament = Tournament {
                id: 0,
                save_id: save_id.to_string(),
                season_id,
                tournament_type,
                name: format!("S{} {}", season_id, tournament_name),
                region_id: None,
                status: TournamentStatus::Upcoming,
                current_stage: None,
                current_round: None,
            };

            TournamentRepository::create(pool, save_id, &tournament)
                .await
                .map_err(|e| e.to_string())?
        };

        // 检查是否已有比赛
        let match_count = self.count_tournament_matches(pool, id).await?;

        if match_count == 0 {
            // 获取参赛队伍：各赛区常规赛前8名 (共32队)
            let mut teams = Vec::new();

            let regular_tournaments = TournamentRepository::get_by_season_and_type(
                pool, save_id, season_id, source_regular
            ).await.map_err(|e| e.to_string())?;

            log::debug!("找到 {} 个常规赛", regular_tournaments.len());

            for regular in regular_tournaments {
                let standings = StandingRepository::get_by_tournament(pool, regular.id)
                    .await
                    .map_err(|e| e.to_string())?;

                log::debug!("赛区 {:?} 积分榜有 {} 支队伍", regular.region_id, standings.len());

                for standing in standings.iter().take(8) {
                    if let Ok(team) = TeamRepository::get_by_id(pool, standing.team_id).await {
                        if !teams.iter().any(|t: &crate::models::Team| t.id == team.id) {
                            teams.push(team);
                        }
                    }
                }
            }

            log::debug!("找到 {} 支参赛队伍", teams.len());

            // 如果队伍不足32支，用各赛区其他队伍填充
            if teams.len() < 32 {
                let fill_region_ids = self.get_all_region_ids(pool, save_id).await?;
                for region_id in fill_region_ids {
                    let region_teams = TeamRepository::get_by_region(pool, save_id, region_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    for team in region_teams {
                        if teams.len() >= 32 {
                            break;
                        }
                        if !teams.iter().any(|t| t.id == team.id) {
                            teams.push(team);
                        }
                    }
                }
            }

            if teams.len() >= 32 {
                let matches = self.tournament_service.generate_masters_bracket(id, &teams[..32]);
                log::debug!("生成 {} 场比赛", matches.len());
                if !matches.is_empty() {
                    MatchRepository::create_batch(pool, save_id, &matches)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 初始化积分榜（小组赛阶段需要）
                    let standings: Vec<LeagueStanding> = teams[..32]
                        .iter()
                        .map(|team| LeagueStanding {
                            id: 0,
                            tournament_id: id,
                            team_id: team.id,
                            rank: None,
                            matches_played: 0,
                            wins: 0,
                            losses: 0,
                            points: 0,
                            games_won: 0,
                            games_lost: 0,
                            game_diff: 0,
                        })
                        .collect();

                    StandingRepository::upsert_batch(pool, save_id, &standings)
                        .await
                        .map_err(|e| e.to_string())?;
                    log::debug!("初始化 {} 支队伍的积分榜", standings.len());
                }
            } else {
                log::debug!("队伍不足32支 (只有{}支)，跳过比赛生成", teams.len());
            }
        }

        tournaments_created.push(TournamentCreated {
            id,
            name: format!("S{} {}", season_id, tournament_name),
            tournament_type: type_str,
            region: None,
        });

        Ok(tournaments_created)
    }

    /// 获取季后赛前3名队伍（冠亚季军）
    async fn get_playoffs_top3(
        &self,
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Vec<crate::models::Team>, String> {
        use crate::models::Team;
        let mut results: Vec<Team> = Vec::new();

        // 获取总决赛（GRAND_FINAL）
        let grand_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let (champion_id, runner_up_id) = if let Some(gf) = grand_final {
            let winner_id = gf.get::<Option<i64>, _>("winner_id");
            let home_id = gf.get::<i64, _>("home_team_id") as u64;
            let away_id = gf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                (Some(winner), Some(loser))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // 获取败者组决赛失败者（季军）
        let losers_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let third_id = if let Some(lf) = losers_final {
            let winner_id = lf.get::<Option<i64>, _>("winner_id");
            let home_id = lf.get::<i64, _>("home_team_id") as u64;
            let away_id = lf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                Some(loser)
            } else {
                None
            }
        } else {
            None
        };

        // 加载队伍详情
        if let Some(cid) = champion_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, cid).await {
                results.push(team);
            }
        }
        if let Some(rid) = runner_up_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, rid).await {
                results.push(team);
            }
        }
        if let Some(tid) = third_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, tid).await {
                results.push(team);
            }
        }

        log::debug!("tournament_id={}, found {} teams", tournament_id, results.len());
        Ok(results)
    }

    /// 获取季后赛前4名队伍 (用于ICP洲际赛)
    async fn get_playoffs_top4(
        &self,
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Vec<crate::models::Team>, String> {
        use crate::models::Team;
        let mut results: Vec<Team> = Vec::new();

        log::debug!("开始获取 tournament_id={} 的前4名", tournament_id);

        // 获取总决赛（GRAND_FINAL）
        let grand_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let (champion_id, runner_up_id) = if let Some(gf) = grand_final {
            let winner_id = gf.get::<Option<i64>, _>("winner_id");
            let home_id = gf.get::<i64, _>("home_team_id") as u64;
            let away_id = gf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, loser);
                (Some(winner), Some(loser))
            } else {
                log::debug!("GRAND_FINAL 存在但无 winner_id");
                (None, None)
            }
        } else {
            log::debug!("未找到 GRAND_FINAL");
            (None, None)
        };

        // 获取败者组决赛失败者（季军）
        let losers_final = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let third_id = if let Some(lf) = losers_final {
            let winner_id = lf.get::<Option<i64>, _>("winner_id");
            let home_id = lf.get::<i64, _>("home_team_id") as u64;
            let away_id = lf.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                log::debug!("LOSERS_FINAL: third={}", loser);
                Some(loser)
            } else {
                log::debug!("LOSERS_FINAL 存在但无 winner_id");
                None
            }
        } else {
            log::debug!("未找到 LOSERS_FINAL");
            None
        };

        // 获取败者组R3失败者（殿军/第4名）
        // 注意：季后赛的败者组结构是 R1 -> R2 -> R3(1场) -> FINAL
        // LOSERS_R3 只有1场比赛，败者是第4名
        let losers_r3 = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'"
        )
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let fourth_id = if let Some(lr3) = losers_r3 {
            let winner_id = lr3.get::<Option<i64>, _>("winner_id");
            let home_id = lr3.get::<i64, _>("home_team_id") as u64;
            let away_id = lr3.get::<i64, _>("away_team_id") as u64;

            if let Some(winner) = winner_id {
                let winner = winner as u64;
                let loser = if winner == home_id { away_id } else { home_id };
                log::debug!("LOSERS_R3: fourth={}", loser);
                Some(loser)
            } else {
                log::debug!("LOSERS_R3 存在但无 winner_id");
                None
            }
        } else {
            log::debug!("未找到 LOSERS_R3");
            None
        };

        // 加载队伍详情（按排名顺序）
        if let Some(cid) = champion_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, cid).await {
                results.push(team);
            }
        }
        if let Some(rid) = runner_up_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, rid).await {
                results.push(team);
            }
        }
        if let Some(tid) = third_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, tid).await {
                results.push(team);
            }
        }
        if let Some(fid) = fourth_id {
            if let Ok(team) = TeamRepository::get_by_id(pool, fid).await {
                results.push(team);
            }
        }

        log::debug!("tournament_id={}, found {} teams", tournament_id, results.len());
        Ok(results)
    }

    /// 颁发赛事年度积分
    async fn award_tournament_points(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<(u64, u32)>, String> {
        let points_engine = PointsCalculationEngine::new();
        let mut awarded: Vec<(u64, u32)> = Vec::new();

        // 根据赛事类型获取排名结果
        let results = self.get_tournament_final_results(pool, save_id, tournament_id, tournament_type).await?;

        for (team_id, position) in &results {
            let points = points_engine.get_points(tournament_type, position);
            if points > 0 {
                // 保存积分明细（带去重检查）
                let (_, is_new) = PointsRepository::add_points_detail(
                    pool,
                    save_id,
                    season_id,
                    *team_id,
                    tournament_id,
                    points,
                    position_to_rank(position),
                )
                .await
                .map_err(|e| e.to_string())?;

                // 只有新记录才更新队伍的年度积分
                if is_new {
                    let mut team = TeamRepository::get_by_id(pool, *team_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    team.annual_points += points;
                    TeamRepository::update(pool, &team)
                        .await
                        .map_err(|e| e.to_string())?;

                    awarded.push((*team_id, points));
                    log::debug!("Awarded {} points to team {} for position {} in tournament {}",
                        points, team_id, position, tournament_id);
                } else {
                    log::debug!("Skipped duplicate points for team {} in tournament {}", team_id, tournament_id);
                }
            }
        }

        Ok(awarded)
    }

    /// 发放赛事奖金
    async fn distribute_tournament_prizes(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<(u64, u64)>, String> {
        // 幂等检查：如果该赛事奖金已发放过，跳过
        let existing: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM financial_transactions WHERE save_id = ? AND related_tournament_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to check existing prizes: {}", e))?;

        if existing.0 > 0 {
            log::debug!("奖金已发放过，跳过: tournament_id={}", tournament_id);
            return Ok(vec![]);
        }

        let financial_engine = FinancialEngine::new();
        let mut distributed: Vec<(u64, u64)> = Vec::new();

        // 获取赛事排名结果
        let results = self.get_tournament_final_results(pool, save_id, tournament_id, tournament_type).await?;

        for (team_id, position) in &results {
            let prize = financial_engine.calculate_prize_money(tournament_type, position);
            if prize > 0 {
                // 确定交易类型描述
                let transaction_type = if tournament_type.is_regional() {
                    "PlayoffBonus"
                } else {
                    "InternationalBonus"
                };

                let description = format!("{:?} - {} 奖金", tournament_type, position);

                // 记录财务交易（含 related_tournament_id 用于幂等检查）
                sqlx::query(
                    r#"
                    INSERT INTO financial_transactions (
                        save_id, team_id, season_id, transaction_type, amount, description, related_tournament_id
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(save_id)
                .bind(*team_id as i64)
                .bind(season_id as i64)
                .bind(transaction_type)
                .bind(prize as i64)
                .bind(&description)
                .bind(tournament_id as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to record prize transaction: {}", e))?;

                // 更新队伍余额
                sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
                    .bind(prize as i64)
                    .bind(*team_id as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update team balance: {}", e))?;

                distributed.push((*team_id, prize));
                log::debug!("Awarded {} prize to team {} for position {} in tournament {}",
                    prize, team_id, position, tournament_id);
            }
        }

        log::debug!("Total {} prizes distributed for tournament {}", distributed.len(), tournament_id);
        Ok(distributed)
    }

    /// 更新冠军队伍选手的统计数据（增加冠军次数）
    async fn update_champion_player_stats(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_id: u64,
        tournament_type: TournamentType,
        is_international: bool,
    ) -> Result<(), String> {
        // 获取赛事排名结果
        let results = self.get_tournament_final_results(pool, save_id, tournament_id, tournament_type).await?;

        // 找到冠军队伍
        let champion_team_id = results.iter()
            .find(|(_, pos)| pos == "CHAMPION")
            .map(|(team_id, _)| *team_id);

        if let Some(team_id) = champion_team_id {
            log::debug!("Updating stats for champion team {} in tournament {}", team_id, tournament_id);

            // 获取队伍的所有选手（从 players 表）
            let players = sqlx::query(
                r#"
                SELECT id, game_id, position FROM players
                WHERE save_id = ? AND team_id = ? AND status = 'Active'
                "#
            )
            .bind(save_id)
            .bind(team_id as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get team players: {}", e))?;

            // 更新每个选手的冠军统计
            for player_row in &players {
                let player_id: i64 = player_row.get("id");
                let player_name: String = player_row.get("game_id");
                let position: String = player_row.get("position");

                // 获取或创建选手的赛季统计
                let mut stats = PlayerStatsRepository::get_or_create(
                    pool,
                    save_id,
                    player_id,
                    &player_name,
                    season_id as i64,
                    Some(team_id as i64),
                    None,  // region_id
                    &position
                )
                .await
                .map_err(|e| e.to_string())?;

                // 更新冠军次数
                if is_international {
                    stats.international_titles += 1;
                    log::debug!("Player {} now has {} international titles", player_id, stats.international_titles);
                } else {
                    stats.regional_titles += 1;
                    log::debug!("Player {} now has {} regional titles", player_id, stats.regional_titles);
                }

                // 重新计算冠军加成和年度Top得分（五维归一化加权）
                stats.champion_bonus = (stats.international_titles * 3 + stats.regional_titles) as f64;
                stats.yearly_top_score = PlayerSeasonStatistics::calculate_yearly_top_score(
                    stats.avg_impact,
                    stats.avg_performance,
                    stats.consistency_score,
                    stats.games_played,
                    stats.champion_bonus,
                );
                stats.dominance_score = PlayerSeasonStatistics::calculate_dominance_score(
                    stats.best_performance,
                    stats.avg_impact,
                    stats.avg_performance,
                );

                // 保存更新
                PlayerStatsRepository::update(pool, &stats)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            log::debug!("Successfully updated champion stats for tournament {}", tournament_id);
        } else {
            log::debug!("No champion found for tournament {}", tournament_id);
        }

        Ok(())
    }

    /// 获取赛事最终排名结果
    async fn get_tournament_final_results(
        &self,
        pool: &Pool<Sqlite>,
        _save_id: &str,
        tournament_id: u64,
        tournament_type: TournamentType,
    ) -> Result<Vec<(u64, String)>, String> {
        let mut results: Vec<(u64, String)> = Vec::new();

        match tournament_type {
            // 季后赛：从双败淘汰赛结果获取排名
            // 使用的 stage: WINNERS_R1, LOSERS_R1, WINNERS_FINAL, LOSERS_R2, LOSERS_R3, LOSERS_FINAL, GRAND_FINAL
            TournamentType::SpringPlayoffs | TournamentType::SummerPlayoffs => {
                // 获取总决赛 (GRAND_FINAL)
                let grand_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(final_match) = grand_final {
                    let winner_id = final_match.get::<Option<i64>, _>("winner_id");
                    let home_id = final_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = final_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let winner = winner as u64;
                        let runner_up = if winner == home_id { away_id } else { home_id };
                        results.push((winner, "CHAMPION".to_string()));
                        results.push((runner_up, "RUNNER_UP".to_string()));
                        log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, runner_up);
                    }
                } else {
                    log::debug!("No GRAND_FINAL match found for tournament {}", tournament_id);
                }

                // 获取败者组决赛失败者（季军）
                let losers_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lf_match) = losers_final {
                    let winner_id = lf_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lf_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lf_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        // 败者组决赛的败者是季军
                        let third = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((third, "THIRD".to_string()));
                        log::debug!("LOSERS_FINAL loser (third): {}", third);
                    }
                }

                // 获取败者组第三轮失败者（殿军）
                let losers_r3 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lr3_match) = losers_r3 {
                    let winner_id = lr3_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr3_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr3_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        // 败者组R3的败者是殿军
                        let fourth = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((fourth, "FOURTH".to_string()));
                        log::debug!("LOSERS_R3 loser (fourth): {}", fourth);
                    }
                }

                // 获取败者组第二轮失败者（5-6名）
                let losers_r2 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R2' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr2_match in losers_r2 {
                    let winner_id = lr2_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr2_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr2_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "5TH_8TH".to_string()));
                        log::debug!("LOSERS_R2 loser (5th-8th): {}", loser);
                    }
                }

                // 获取败者组第一轮失败者（7-8名）
                let losers_r1 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R1' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr1_match in losers_r1 {
                    let winner_id = lr1_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr1_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr1_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "5TH_8TH".to_string()));
                        log::debug!("LOSERS_R1 loser (5th-8th): {}", loser);
                    }
                }

                log::debug!("Total results for tournament {}: {:?}", tournament_id, results);
            }

            // MSI - 双败赛制，需要单独处理
            TournamentType::Msi => {
                // 获取总决赛 (GRAND_FINAL)
                let grand_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'GRAND_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(final_match) = grand_final {
                    let winner_id = final_match.get::<Option<i64>, _>("winner_id");
                    let home_id = final_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = final_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let winner = winner as u64;
                        let runner_up = if winner == home_id { away_id } else { home_id };
                        results.push((winner, "CHAMPION".to_string()));
                        results.push((runner_up, "RUNNER_UP".to_string()));
                        log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, runner_up);
                    }
                }

                // 获取败者组决赛失败者（季军）
                let losers_final = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_FINAL' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lf_match) = losers_final {
                    let winner_id = lf_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lf_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lf_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let third = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((third, "THIRD".to_string()));
                        log::debug!("LOSERS_FINAL loser (third): {}", third);
                    }
                }

                // 获取 LOSERS_R4 失败者（殿军）- MSI的败者组R4只有1场
                let losers_r4 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R4' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;

                if let Some(lr4_match) = losers_r4 {
                    let winner_id = lr4_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr4_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr4_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let fourth = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((fourth, "FOURTH".to_string()));
                        log::debug!("LOSERS_R4 loser (fourth): {}", fourth);
                    }
                }

                // 获取 LOSERS_R3 失败者（5-6名）
                let losers_r3 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R3' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr3_match in losers_r3 {
                    let winner_id = lr3_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr3_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr3_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "LOSERS_R2".to_string())); // 积分配置中是 LOSERS_R2
                        log::debug!("LOSERS_R3 loser: {}", loser);
                    }
                }

                // 获取 LOSERS_R2 失败者（7-8名）
                let losers_r2 = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND stage = 'LOSERS_R2' AND UPPER(status) = 'COMPLETED'
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                for lr2_match in losers_r2 {
                    let winner_id = lr2_match.get::<Option<i64>, _>("winner_id");
                    let home_id = lr2_match.get::<i64, _>("home_team_id") as u64;
                    let away_id = lr2_match.get::<i64, _>("away_team_id") as u64;

                    if let Some(winner) = winner_id {
                        let loser = if winner as u64 == home_id { away_id } else { home_id };
                        results.push((loser, "LOSERS_R1".to_string())); // 积分配置中是 LOSERS_R1
                        log::debug!("LOSERS_R2 loser: {}", loser);
                    }
                }

                log::debug!("Total results: {:?}", results);
            }

            // 马德里大师赛/Claude洲际赛 (32队分组+东西半区淘汰)
            TournamentType::MadridMasters | TournamentType::ClaudeIntercontinental => {
                let all_matches = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND UPPER(status) = 'COMPLETED'
                    ORDER BY stage DESC, match_order
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 找到总决赛 (GRAND_FINAL)
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "GRAND_FINAL" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let runner_up = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "CHAMPION".to_string()));
                            results.push((runner_up, "RUNNER_UP".to_string()));
                            log::debug!("GRAND_FINAL: champion={}, runner_up={}", winner, runner_up);
                        }
                        break;
                    }
                }

                // 找到季军赛 (THIRD_PLACE) - 获取季军和殿军
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "THIRD_PLACE" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let loser = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "THIRD".to_string()));
                            results.push((loser, "FOURTH".to_string()));
                            log::debug!("THIRD_PLACE: third={}, fourth={}", winner, loser);
                        }
                        break;
                    }
                }

                // 东西半区半决赛失败者 (EAST_SEMI, WEST_SEMI) -> SEMI_LOSER
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "EAST_SEMI" || stage == "WEST_SEMI" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "SEMI_LOSER".to_string()));
                            log::debug!("{} loser (SEMI_LOSER): {}", stage, loser);
                        }
                    }
                }

                // 东西半区第一轮失败者 (EAST_R1, WEST_R1) -> QUARTER_LOSER
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "EAST_R1" || stage == "WEST_R1" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "QUARTER_LOSER".to_string()));
                            log::debug!("{} loser (QUARTER_LOSER): {}", stage, loser);
                        }
                    }
                }

                log::debug!("Total results: {:?}", results);
            }

            // 其他国际赛事 (标准淘汰赛制)
            TournamentType::WorldChampionship
            | TournamentType::ShanghaiMasters => {
                // 获取淘汰赛阶段的比赛结果
                let knockout_matches = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND UPPER(status) = 'COMPLETED'
                    ORDER BY stage DESC, match_order
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 找到决赛
                for m in &knockout_matches {
                    let stage: String = m.get("stage");
                    if stage == "FINALS" || stage == "GRAND_FINALS" || stage == "GRAND_FINAL" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let runner_up = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "CHAMPION".to_string()));
                            results.push((runner_up, "RUNNER_UP".to_string()));
                        }
                        break;
                    }
                }

                // 找半决赛失败者
                let mut semi_losers: Vec<u64> = Vec::new();
                for m in &knockout_matches {
                    let stage: String = m.get("stage");
                    if stage == "SEMI_FINALS" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            semi_losers.push(loser);
                        }
                    }
                }

                if semi_losers.len() >= 2 {
                    results.push((semi_losers[0], "THIRD".to_string()));
                    results.push((semi_losers[1], "FOURTH".to_string()));
                } else if semi_losers.len() == 1 {
                    results.push((semi_losers[0], "THIRD".to_string()));
                }

                // 八强失败者
                for m in &knockout_matches {
                    let stage: String = m.get("stage");
                    if stage == "QUARTER_FINALS" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            results.push((loser, "QUARTER_FINAL".to_string()));
                        }
                    }
                }
            }

            // ICP洲际对抗赛 - 按赛区排名分配积分
            TournamentType::IcpIntercontinental => {
                // 1. 获取所有参赛队伍及其赛区
                let team_rows = sqlx::query(
                    r#"
                    SELECT DISTINCT t.id as team_id, t.region_id
                    FROM matches m
                    JOIN teams t ON t.id = m.home_team_id OR t.id = m.away_team_id
                    WHERE m.tournament_id = ?
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 收集所有参赛队伍ID
                let mut participant_team_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
                let mut team_region_map: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();

                for row in &team_rows {
                    let team_id: i64 = row.get("team_id");
                    let region_id: i64 = row.get("region_id");
                    participant_team_ids.insert(team_id as u64);
                    team_region_map.insert(team_id as u64, region_id as u64);
                }

                // 2. 从决赛和半决赛结果确定赛区排名
                let final_matches = sqlx::query(
                    r#"
                    SELECT m.stage, m.home_team_id, m.away_team_id, m.winner_id,
                           ht.region_id as home_region_id, at.region_id as away_region_id
                    FROM matches m
                    LEFT JOIN teams ht ON m.home_team_id = ht.id
                    LEFT JOIN teams at ON m.away_team_id = at.id
                    WHERE m.tournament_id = ? AND (m.stage LIKE 'ICP_FINAL%' OR m.stage LIKE 'ICP_SEMI%')
                    AND UPPER(m.status) = 'COMPLETED'
                    ORDER BY m.stage
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 统计决赛和半决赛的胜场
                let mut final_wins: std::collections::HashMap<u64, i32> = std::collections::HashMap::new();
                let mut semi_wins: std::collections::HashMap<u64, i32> = std::collections::HashMap::new();
                let mut final_regions: (Option<u64>, Option<u64>) = (None, None);
                let mut semi_regions: Vec<u64> = Vec::new();

                for row in &final_matches {
                    let stage: String = row.get("stage");
                    let winner_id: Option<i64> = row.get("winner_id");
                    let home_region_id: Option<i64> = row.get("home_region_id");
                    let away_region_id: Option<i64> = row.get("away_region_id");

                    if let (Some(winner), Some(home_region), Some(away_region)) = (winner_id, home_region_id, away_region_id) {
                        let home_team_id: Option<i64> = row.get("home_team_id");
                        let winner_region = if home_team_id == Some(winner) {
                            home_region as u64
                        } else {
                            away_region as u64
                        };

                        if stage.starts_with("ICP_FINAL") {
                            *final_wins.entry(winner_region).or_insert(0) += 1;
                            final_regions = (Some(home_region as u64), Some(away_region as u64));
                        } else if stage.starts_with("ICP_SEMI") {
                            *semi_wins.entry(winner_region).or_insert(0) += 1;
                            if !semi_regions.contains(&(home_region as u64)) {
                                semi_regions.push(home_region as u64);
                            }
                            if !semi_regions.contains(&(away_region as u64)) {
                                semi_regions.push(away_region as u64);
                            }
                        }
                    }
                }

                // 确定赛区排名
                let (champion_region, runner_up_region) = {
                    let mut sorted: Vec<_> = final_wins.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(a.1));

                    if sorted.len() >= 2 {
                        (*sorted[0].0, *sorted[1].0)
                    } else if sorted.len() == 1 {
                        if let (Some(r1), Some(r2)) = final_regions {
                            if *sorted[0].0 == r1 { (r1, r2) } else { (r2, r1) }
                        } else {
                            log::debug!("[ICP Points] 无法确定冠亚军赛区");
                            return Ok(results);
                        }
                    } else {
                        log::debug!("[ICP Points] 没有决赛结果");
                        return Ok(results);
                    }
                };

                // 第三、第四赛区是半决赛中未进入决赛的赛区
                let (third_region, fourth_region) = {
                    let losers: Vec<u64> = semi_regions.iter()
                        .filter(|&&r| r != champion_region && r != runner_up_region)
                        .copied()
                        .collect();

                    if losers.len() >= 2 {
                        // 根据半决赛胜场数排名
                        let r1_wins = semi_wins.get(&losers[0]).copied().unwrap_or(0);
                        let r2_wins = semi_wins.get(&losers[1]).copied().unwrap_or(0);
                        if r1_wins >= r2_wins {
                            (losers[0], losers[1])
                        } else {
                            (losers[1], losers[0])
                        }
                    } else if losers.len() == 1 {
                        (losers[0], 0)
                    } else {
                        (0, 0)
                    }
                };

                log::debug!("[ICP Points] 赛区排名: 冠军={}, 亚军={}, 季军={}, 殿军={}",
                    champion_region, runner_up_region, third_region, fourth_region);

                // 3. 获取每个赛区的所有队伍
                let all_region_teams = sqlx::query(
                    r#"
                    SELECT t.id as team_id, t.region_id
                    FROM teams t
                    WHERE t.region_id IN (?, ?, ?, ?)
                    "#
                )
                .bind(champion_region as i64)
                .bind(runner_up_region as i64)
                .bind(third_region as i64)
                .bind(fourth_region as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 按赛区分组
                let mut region_teams: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
                for row in &all_region_teams {
                    let team_id: i64 = row.get("team_id");
                    let region_id: i64 = row.get("region_id");
                    region_teams.entry(region_id as u64).or_default().push(team_id as u64);
                }

                // 4. 为每个队伍分配积分
                // 冠军赛区
                if let Some(teams) = region_teams.get(&champion_region) {
                    for &team_id in teams {
                        let position = if participant_team_ids.contains(&team_id) {
                            "FIRST_PARTICIPANT"
                        } else {
                            "FIRST_NON_PARTICIPANT"
                        };
                        results.push((team_id, position.to_string()));
                    }
                }

                // 亚军赛区
                if let Some(teams) = region_teams.get(&runner_up_region) {
                    for &team_id in teams {
                        let position = if participant_team_ids.contains(&team_id) {
                            "SECOND_PARTICIPANT"
                        } else {
                            "SECOND_NON_PARTICIPANT"
                        };
                        results.push((team_id, position.to_string()));
                    }
                }

                // 季军赛区
                if third_region > 0 {
                    if let Some(teams) = region_teams.get(&third_region) {
                        for &team_id in teams {
                            let position = if participant_team_ids.contains(&team_id) {
                                "THIRD_PARTICIPANT"
                            } else {
                                "THIRD_NON_PARTICIPANT"
                            };
                            results.push((team_id, position.to_string()));
                        }
                    }
                }

                // 殿军赛区
                if fourth_region > 0 {
                    if let Some(teams) = region_teams.get(&fourth_region) {
                        for &team_id in teams {
                            let position = if participant_team_ids.contains(&team_id) {
                                "FOURTH_PARTICIPANT"
                            } else {
                                "FOURTH_NON_PARTICIPANT"
                            };
                            results.push((team_id, position.to_string()));
                        }
                    }
                }

                log::debug!("[ICP Points] 共 {} 个队伍需要颁发积分", results.len());
            }

            // Super洲际邀请赛
            TournamentType::SuperIntercontinental => {
                // Super赛的积分配置更复杂，需要追踪每个队伍的淘汰阶段
                let all_matches = sqlx::query(
                    r#"
                    SELECT * FROM matches
                    WHERE tournament_id = ? AND status = 'COMPLETED'
                    ORDER BY stage, match_order
                    "#
                )
                .bind(tournament_id as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;

                // 决赛
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "GRAND_FINALS" || stage == "FINALS" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let winner = winner as u64;
                            let runner_up = if winner == home_id { away_id } else { home_id };
                            results.push((winner, "CHAMPION".to_string()));
                            results.push((runner_up, "RUNNER_UP".to_string()));
                        }
                        break;
                    }
                }

                // 半决赛失败者
                let mut semi_losers: Vec<u64> = Vec::new();
                for m in &all_matches {
                    let stage: String = m.get("stage");
                    if stage == "SEMI_FINALS" {
                        let winner_id = m.get::<Option<i64>, _>("winner_id");
                        let home_id = m.get::<i64, _>("home_team_id") as u64;
                        let away_id = m.get::<i64, _>("away_team_id") as u64;

                        if let Some(winner) = winner_id {
                            let loser = if winner as u64 == home_id { away_id } else { home_id };
                            semi_losers.push(loser);
                        }
                    }
                }

                if semi_losers.len() >= 2 {
                    results.push((semi_losers[0], "THIRD".to_string()));
                    results.push((semi_losers[1], "FOURTH".to_string()));
                }
            }

            _ => {}
        }

        Ok(results)
    }

    /// 自动确认首发：为每支队伍的每个位置选能力最高的选手设为首发
    pub async fn auto_confirm_starters(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<u32, String> {
        // 清除所有首发标记
        sqlx::query("UPDATE players SET is_starter = 0 WHERE save_id = ? AND status = 'Active'")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("清除首发失败: {}", e))?;

        // 获取所有队伍
        let teams = TeamRepository::get_all(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let positions = ["Top", "Jug", "Mid", "Adc", "Sup"];
        let mut confirmed_count = 0u32;

        for team in &teams {
            let mut team_confirmed = 0u32;
            for pos in &positions {
                // 找到该队伍该位置能力最高的选手
                let result = sqlx::query(
                    r#"
                    SELECT id, game_id, ability, position FROM players
                    WHERE save_id = ? AND team_id = ? AND status = 'Active'
                      AND UPPER(position) = UPPER(?)
                    ORDER BY ability DESC
                    LIMIT 1
                    "#,
                )
                .bind(save_id)
                .bind(team.id as i64)
                .bind(pos)
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("查询最强选手失败: {}", e))?;

                if let Some(row) = result {
                    let player_id: i64 = row.get("id");
                    sqlx::query("UPDATE players SET is_starter = 1 WHERE id = ?")
                        .bind(player_id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("设置首发失败: {}", e))?;
                    confirmed_count += 1;
                    team_confirmed += 1;
                } else {
                    // 打印该队伍所有选手的位置信息，帮助排查
                    let all_players: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
                        "SELECT id, game_id, position, status, ability FROM players WHERE save_id = ? AND team_id = ? ORDER BY position"
                    )
                    .bind(save_id)
                    .bind(team.id as i64)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

                    let player_info: Vec<String> = all_players.iter().map(|p| {
                        format!("{}({}, {}, ability={})",
                            p.get::<String, _>("game_id"),
                            p.get::<String, _>("position"),
                            p.get::<String, _>("status"),
                            p.get::<i64, _>("ability"))
                    }).collect();

                    log::debug!("警告: 战队 {} (id={}) 缺少 {} 位置的选手! 该队所有选手: {:?}",
                        team.name, team.id, pos, player_info);
                }
            }
            if team_confirmed < 5 {
                log::debug!("战队 {} (id={}) 只确认了 {}/5 名首发!", team.name, team.id, team_confirmed);
            }
        }

        log::debug!("确认了 {} 名首发选手", confirmed_count);
        Ok(confirmed_count)
    }

    /// 重新计算所有队伍的战力值：取首发选手能力值的平均值
    pub async fn recalculate_team_powers(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<(), String> {
        let teams = TeamRepository::get_all(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        for team in &teams {
            let avg_ability: f64 = sqlx::query_scalar(
                r#"
                SELECT COALESCE(AVG(ability), 60.0) FROM players
                WHERE save_id = ? AND team_id = ? AND status = 'Active' AND is_starter = 1
                "#,
            )
            .bind(save_id)
            .bind(team.id as i64)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("计算队伍战力失败: {}", e))?;

            sqlx::query("UPDATE teams SET power_rating = ? WHERE id = ?")
                .bind(avg_ability)
                .bind(team.id as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("更新队伍战力失败: {}", e))?;
        }

        log::debug!("更新了 {} 支队伍的战力", teams.len());
        Ok(())
    }

    /// 推进到新赛季
    pub async fn advance_to_new_season(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<NewSeasonResult, String> {
        // 获取当前存档
        let mut save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        // 1. 更新赛季号和阶段
        save.current_season += 1;
        save.current_phase = SeasonPhase::SpringRegular;
        save.phase_completed = false;
        save.updated_at = chrono::Utc::now();

        SaveRepository::update(pool, &save)
            .await
            .map_err(|e| e.to_string())?;

        // 2. 批量重置年度积分
        sqlx::query("UPDATE teams SET annual_points = 0 WHERE save_id = ?")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("重置年度积分失败: {}", e))?;

        // 2.5 重置选手 form factors（新赛季状态重置）
        sqlx::query(
            r#"
            UPDATE player_form_factors
            SET momentum = 0,
                last_performance = 0.0,
                last_match_won = 0,
                games_since_rest = 0,
                form_cycle = (ABS(RANDOM()) % 10000) / 100.0,
                updated_at = datetime('now')
            WHERE save_id = ?
            "#,
        )
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| format!("重置 form factors 失败: {}", e))?;

        // 2.6 为新赛季生成 Meta 版本
        MetaEngine::roll_new_meta(pool, save_id, save.current_season as i64).await
            .map_err(|e| format!("生成 Meta 版本失败: {}", e))?;

        // 3. 自动确认首发
        let starters_confirmed = self.auto_confirm_starters(pool, save_id).await?;

        // 4. 更新战力
        self.recalculate_team_powers(pool, save_id).await?;

        // 5. 初始化春季赛（创建4个赛区的赛事、赛程、积分榜）
        self.initialize_phase(pool, save_id, save.current_season as u64, SeasonPhase::SpringRegular).await?;

        let message = format!(
            "已进入第 {} 赛季，确认了 {} 名首发选手，已更新战力并创建春季赛",
            save.current_season, starters_confirmed
        );

        Ok(NewSeasonResult {
            new_season: save.current_season,
            starters_confirmed,
            message,
        })
    }

    // ========== 时间推进系统核心方法 ==========

    /// 获取完整的游戏时间状态（统一入口）
    pub async fn get_time_state(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<GameTimeState, String> {
        // 获取存档信息
        let save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let current_phase = save.current_phase;
        let current_season = save.current_season;

        // 获取阶段进度
        let phase_progress = self.get_phase_progress(pool, save_id, current_season as u64, current_phase).await?;

        // 判断阶段状态
        let phase_status = if phase_progress.total_matches == 0 && !current_phase.is_non_tournament() {
            // 赛事阶段未初始化
            PhaseStatus::NotInitialized
        } else if current_phase == SeasonPhase::TransferWindow && phase_progress.total_matches == 0 {
            // 转会窗口未创建
            PhaseStatus::NotInitialized
        } else if current_phase == SeasonPhase::Draft && phase_progress.total_matches == 0 && phase_progress.completed_matches == 0 {
            // 选秀阶段：未开始选秀 → InProgress（等待用户在选秀页面操作）
            PhaseStatus::InProgress
        } else if phase_progress.completed_matches >= phase_progress.total_matches && phase_progress.total_matches > 0 {
            // 所有任务完成（必须有实际任务才算完成）
            PhaseStatus::Completed
        } else {
            PhaseStatus::InProgress
        };

        // 获取赛季进度
        let season_progress = self.get_season_progress(current_phase);

        // 获取可用操作
        let available_actions = self.get_available_actions(current_phase, &phase_status);

        // 判断是否可以推进（需要已完成且存在下一阶段）
        let can_advance = phase_status == PhaseStatus::Completed && current_phase.next().is_some();

        // 获取下一阶段
        let next_phase = current_phase.next().map(|p| p.name().to_string());

        Ok(GameTimeState {
            save_id: save_id.to_string(),
            current_season,
            current_phase,
            phase_display_name: current_phase.display_name().to_string(),
            phase_status,
            phase_progress,
            season_progress,
            available_actions,
            can_advance,
            next_phase,
        })
    }

    /// 获取阶段进度
    async fn get_phase_progress(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        phase: SeasonPhase,
    ) -> Result<PhaseProgress, String> {
        let tournament_type = phase.to_tournament_type();

        if let Some(t_type) = tournament_type {
            let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;

            // 预加载赛区名称映射（避免硬编码ID）
            let region_rows = sqlx::query("SELECT id, name FROM regions WHERE save_id = ?")
                .bind(save_id)
                .fetch_all(pool)
                .await
                .unwrap_or_default();
            let region_map: std::collections::HashMap<u64, String> = region_rows.iter()
                .map(|r| (r.get::<i64, _>("id") as u64, r.get::<String, _>("name")))
                .collect();

            let mut tournament_progress_list = Vec::new();
            let mut total_matches = 0u32;
            let mut completed_matches = 0u32;

            for tournament in tournaments {
                // 查询比赛数量（兼容 'COMPLETED' 和 'Completed' 两种状态格式）
                // CANCELLED 的比赛也算作已完成（不需要进行的比赛）
                let counts = sqlx::query(
                    r#"
                    SELECT
                        COUNT(*) as total,
                        SUM(CASE WHEN UPPER(status) IN ('COMPLETED', 'CANCELLED') THEN 1 ELSE 0 END) as completed
                    FROM matches
                    WHERE tournament_id = ?
                    "#
                )
                .bind(tournament.id as i64)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

                let t_total: i64 = counts.get("total");
                let t_completed: i64 = counts.get("completed");

                total_matches += t_total as u32;
                completed_matches += t_completed as u32;

                let status = if t_completed >= t_total && t_total > 0 {
                    "completed"
                } else if t_completed > 0 {
                    "in_progress"
                } else {
                    "upcoming"
                };

                tournament_progress_list.push(TournamentProgress {
                    tournament_id: tournament.id,
                    tournament_name: tournament.name.clone(),
                    region: tournament.region_id.map(|r| region_map.get(&r).cloned().unwrap_or_else(|| get_region_name(r).to_string())),
                    total_matches: t_total as u32,
                    completed_matches: t_completed as u32,
                    status: status.to_string(),
                });
            }

            let percentage = if total_matches > 0 {
                (completed_matches as f32 / total_matches as f32) * 100.0
            } else {
                0.0
            };

            Ok(PhaseProgress {
                tournaments: tournament_progress_list,
                total_matches,
                completed_matches,
                percentage,
            })
        } else {
            // 非赛事阶段
            match phase {
                SeasonPhase::TransferWindow => {
                    // 查询转会窗口状态
                    let window_row = sqlx::query(
                        "SELECT status, current_round FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
                    )
                    .bind(save_id)
                    .bind(season_id as i64)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| format!("查询转会窗口失败: {}", e))?;

                    match window_row {
                        None => {
                            // 无窗口 → NotInitialized (total=0, completed=0 但不会走到 Completed 判断)
                            Ok(PhaseProgress {
                                tournaments: Vec::new(),
                                total_matches: 0,
                                completed_matches: 0,
                                percentage: 0.0,
                            })
                        }
                        Some(row) => {
                            let status: String = row.get("status");
                            let current_round: i64 = row.get("current_round");
                            if status == "COMPLETED" {
                                // 已完成
                                Ok(PhaseProgress {
                                    tournaments: Vec::new(),
                                    total_matches: 1,
                                    completed_matches: 1,
                                    percentage: 100.0,
                                })
                            } else if status == "IN_PROGRESS" && current_round > 0 {
                                // 进行中
                                Ok(PhaseProgress {
                                    tournaments: Vec::new(),
                                    total_matches: 1,
                                    completed_matches: 0,
                                    percentage: 0.0,
                                })
                            } else {
                                // PENDING 或其他
                                Ok(PhaseProgress {
                                    tournaments: Vec::new(),
                                    total_matches: 0,
                                    completed_matches: 0,
                                    percentage: 0.0,
                                })
                            }
                        }
                    }
                }
                SeasonPhase::Draft => {
                    // 选秀每年都有：检查各赛区是否完成选秀
                    let draft_regions: i64 = sqlx::query_scalar(
                        "SELECT COUNT(DISTINCT region_id) FROM draft_results WHERE save_id = ? AND season_id = ?"
                    )
                    .bind(save_id)
                    .bind(season_id as i64)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| format!("查询选秀结果失败: {}", e))?;

                    if draft_regions >= 4 {
                        // 4赛区都完成
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 1,
                            percentage: 100.0,
                        })
                    } else if draft_regions > 0 {
                        // 部分完成
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 0,
                            percentage: 0.0,
                        })
                    } else {
                        // 未开始
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 0,
                            completed_matches: 0,
                            percentage: 0.0,
                        })
                    }
                }
                SeasonPhase::AnnualAwards => {
                    // 检查是否已颁发年度荣誉
                    let awarded: (i64,) = sqlx::query_as(
                        "SELECT COUNT(*) FROM honors WHERE save_id = ? AND season_id = ? AND honor_type LIKE 'ANNUAL%'"
                    )
                    .bind(save_id)
                    .bind(season_id as i64)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

                    if awarded.0 > 0 {
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 1,
                            percentage: 100.0,
                        })
                    } else {
                        Ok(PhaseProgress {
                            tournaments: Vec::new(),
                            total_matches: 1,
                            completed_matches: 0,
                            percentage: 0.0,
                        })
                    }
                }
                _ => {
                    // SeasonEnd → 立即可推进
                    Ok(PhaseProgress {
                        tournaments: Vec::new(),
                        total_matches: 1,
                        completed_matches: 1,
                        percentage: 100.0,
                    })
                }
            }
        }
    }

    /// 获取赛季进度
    fn get_season_progress(&self, current_phase: SeasonPhase) -> SeasonProgress {
        let all_phases = vec![
            SeasonPhase::SpringRegular,
            SeasonPhase::SpringPlayoffs,
            SeasonPhase::Msi,
            SeasonPhase::MadridMasters,
            SeasonPhase::SummerRegular,
            SeasonPhase::SummerPlayoffs,
            SeasonPhase::ClaudeIntercontinental,
            SeasonPhase::WorldChampionship,
            SeasonPhase::ShanghaiMasters,
            SeasonPhase::IcpIntercontinental,
            SeasonPhase::SuperIntercontinental,
            SeasonPhase::AnnualAwards,
            SeasonPhase::TransferWindow,
            SeasonPhase::Draft,
            SeasonPhase::SeasonEnd,
        ];

        let current_index = all_phases.iter().position(|&p| p == current_phase).unwrap_or(0) as u32;
        let total_phases = all_phases.len() as u32;

        let phases: Vec<PhaseInfo> = all_phases.iter().enumerate().map(|(i, &phase)| {
            let status = if (i as u32) < current_index {
                "completed"
            } else if (i as u32) == current_index {
                "current"
            } else {
                "upcoming"
            };

            PhaseInfo {
                phase: format!("{:?}", phase),
                display_name: phase.display_name().to_string(),
                status: status.to_string(),
                index: i as u32,
            }
        }).collect();

        let percentage = (current_index as f32 / total_phases as f32) * 100.0;

        SeasonProgress {
            phases,
            current_phase_index: current_index,
            total_phases,
            percentage,
        }
    }

    /// 获取当前阶段可用的操作
    fn get_available_actions(&self, phase: SeasonPhase, status: &PhaseStatus) -> Vec<TimeAction> {
        let mut actions = Vec::new();

        match phase {
            SeasonPhase::TransferWindow => {
                match status {
                    PhaseStatus::NotInitialized => {
                        actions.push(TimeAction::StartTransferWindow);
                    }
                    PhaseStatus::InProgress => {
                        actions.push(TimeAction::ExecuteTransferRound);
                    }
                    PhaseStatus::Completed => {
                        actions.push(TimeAction::CompleteAndAdvance);
                    }
                }
            }
            SeasonPhase::Draft => {
                match status {
                    PhaseStatus::Completed => {
                        actions.push(TimeAction::CompleteAndAdvance);
                    }
                    _ => {
                        actions.push(TimeAction::StartDraft);
                    }
                }
            }
            SeasonPhase::SeasonEnd => {
                actions.push(TimeAction::StartNewSeason);
            }
            SeasonPhase::AnnualAwards => {
                // 颁奖典礼：只有完成颁奖后才能推进
                if *status == PhaseStatus::Completed {
                    actions.push(TimeAction::CompleteAndAdvance);
                }
            }
            _ => {
                // 赛事阶段
                match status {
                    PhaseStatus::NotInitialized => {
                        actions.push(TimeAction::InitializePhase);
                    }
                    PhaseStatus::InProgress => {
                        actions.push(TimeAction::SimulateNextMatch);
                        actions.push(TimeAction::SimulateAllMatches);
                    }
                    PhaseStatus::Completed => {
                        actions.push(TimeAction::CompleteAndAdvance);
                    }
                }
            }
        }

        // 快进选项（始终可用，除了赛季结束阶段）
        if phase != SeasonPhase::SeasonEnd {
            actions.push(TimeAction::FastForwardPhase);

            // 根据当前阶段添加快进目标
            if phase.is_before(SeasonPhase::SummerRegular) {
                actions.push(TimeAction::FastForwardToSummer);
            }
            if phase.is_before(SeasonPhase::WorldChampionship) {
                actions.push(TimeAction::FastForwardToWorlds);
            }
            actions.push(TimeAction::FastForwardToSeasonEnd);
        }

        actions
    }

    /// 完成当前阶段并推进到下一阶段
    pub async fn complete_and_advance(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<CompleteAndAdvanceResult, String> {
        // 获取当前存档
        let mut save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let current_phase = save.current_phase;
        let season_id = save.current_season as u64;

        // 转会期必须已确认关闭才能推进
        if current_phase == SeasonPhase::TransferWindow {
            let window_row = sqlx::query(
                "SELECT status FROM transfer_windows WHERE save_id = ? AND season_id = ? ORDER BY id DESC LIMIT 1"
            )
            .bind(save_id)
            .bind(save.current_season)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("查询转会窗口失败: {}", e))?;

            match window_row {
                Some(row) => {
                    let status: String = row.get("status");
                    if status != "COMPLETED" {
                        return Err("转会窗口尚未关闭，请先确认关闭转会窗口后再推进".to_string());
                    }
                }
                None => {
                    return Err("转会窗口未找到，无法推进".to_string());
                }
            }
        }

        // 完成当前阶段（颁发荣誉）
        let complete_result = self.complete_phase(pool, save_id, season_id, current_phase).await?;

        let honors_awarded: Vec<HonorInfo> = complete_result.honors_awarded.iter().map(|h| {
            HonorInfo {
                honor_type: h.honor_type.clone(),
                recipient_name: h.recipient_name.clone(),
                tournament_name: h.tournament_name.clone(),
            }
        }).collect();

        // 推进到下一阶段
        let new_phase = if let Some(next) = current_phase.next() {
            save.current_phase = next;
            save.phase_completed = false;
            save.updated_at = chrono::Utc::now();

            SaveRepository::update(pool, &save)
                .await
                .map_err(|e| e.to_string())?;

            // 自动初始化下一阶段
            let _init_result = self.initialize_phase(pool, save_id, season_id, next).await?;

            Some(next.name().to_string())
        } else {
            // 赛季结束
            None
        };

        // 获取更新后的时间状态
        let new_time_state = self.get_time_state(pool, save_id).await?;

        Ok(CompleteAndAdvanceResult {
            success: true,
            completed_phase: current_phase.name().to_string(),
            new_phase,
            honors_awarded,
            message: complete_result.message,
            new_time_state,
        })
    }

    /// 快进到目标阶段
    pub async fn fast_forward_to(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        target: FastForwardTarget,
    ) -> Result<FastForwardResult, String> {
        let save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let start_phase = save.current_phase;
        let target_phase = match target {
            FastForwardTarget::NextPhase => start_phase.next(),
            FastForwardTarget::ToPhase(phase) => Some(phase),
            FastForwardTarget::SeasonEnd => Some(SeasonPhase::SeasonEnd),
        };

        if target_phase.is_none() {
            return Ok(FastForwardResult {
                success: false,
                start_phase: start_phase.name().to_string(),
                end_phase: start_phase.name().to_string(),
                phases_advanced: 0,
                matches_simulated: 0,
                message: "已经是赛季最后阶段".to_string(),
            });
        }

        let target_phase = target_phase.unwrap();
        let mut current_phase = start_phase;
        let mut phases_advanced = 0u32;
        let mut total_matches_simulated = 0u32;

        // 循环推进直到到达目标阶段
        while current_phase != target_phase {
            // 获取当前状态
            let time_state = self.get_time_state(pool, save_id).await?;

            // 根据状态执行操作
            match time_state.phase_status {
                PhaseStatus::NotInitialized => {
                    // 初始化阶段
                    self.initialize_phase(pool, save_id, time_state.current_season as u64, current_phase).await?;
                }
                PhaseStatus::InProgress => {
                    // 年度颁奖典礼没有比赛可模拟，直接完成并推进
                    if current_phase == SeasonPhase::AnnualAwards {
                        let _result = self.complete_and_advance(pool, save_id).await?;
                        phases_advanced += 1;
                        let save = SaveRepository::get_by_id(pool, save_id)
                            .await
                            .map_err(|e| e.to_string())?;
                        current_phase = save.current_phase;
                    } else {
                        // 模拟所有比赛
                        let matches_simulated = self.simulate_all_phase_matches(pool, save_id, current_phase).await?;
                        total_matches_simulated += matches_simulated;
                    }
                }
                PhaseStatus::Completed => {
                    // 完成并推进
                    let _result = self.complete_and_advance(pool, save_id).await?;
                    phases_advanced += 1;

                    // 更新当前阶段
                    let save = SaveRepository::get_by_id(pool, save_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    current_phase = save.current_phase;
                }
            }

            // 防止无限循环
            if phases_advanced > 20 {
                break;
            }
        }

        Ok(FastForwardResult {
            success: true,
            start_phase: start_phase.name().to_string(),
            end_phase: current_phase.name().to_string(),
            phases_advanced,
            matches_simulated: total_matches_simulated,
            message: format!("快进完成：从{}到{}", start_phase.name(), current_phase.name()),
        })
    }

    /// 模拟当前阶段的所有比赛
    async fn simulate_all_phase_matches(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        phase: SeasonPhase,
    ) -> Result<u32, String> {
        let tournament_type = phase.to_tournament_type();

        if tournament_type.is_none() {
            return Ok(0);
        }

        let t_type = tournament_type.unwrap();
        let save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let tournaments = TournamentRepository::get_by_season_and_type(
            pool, save_id, save.current_season as u64, &format!("{:?}", t_type)
        ).await.map_err(|e| e.to_string())?;

        // 检查是否是季后赛阶段
        let is_playoff = matches!(phase, SeasonPhase::SpringPlayoffs | SeasonPhase::SummerPlayoffs);

        // === 特性系统：预加载选手数据 + form factors ===
        let (mut team_players, mut form_factors_map) = self.load_team_players(pool, save_id, save.current_season as i64).await?;
        let meta_weights = MetaEngine::get_current_weights(pool, save_id, save.current_season as i64)
            .await
            .unwrap_or_else(|_| crate::engines::MetaWeights::balanced());

        // 构建比赛情境
        let is_international = matches!(
            phase,
            SeasonPhase::Msi | SeasonPhase::MadridMasters |
            SeasonPhase::ClaudeIntercontinental | SeasonPhase::WorldChampionship |
            SeasonPhase::ShanghaiMasters | SeasonPhase::IcpIntercontinental |
            SeasonPhase::SuperIntercontinental
        );
        let tournament_type_str = format!("{:?}", t_type).to_lowercase();
        let sim_ctx = MatchSimContext {
            is_playoff,
            is_international,
            tournament_type: tournament_type_str,
        };
        let match_engine = MatchSimulationEngine::default();

        if is_playoff {
            // 季后赛：逐场模拟以确保正确生成后续对阵
            let mut simulated_count = 0u32;

            loop {
                let mut found_pending = false;

                for tournament in &tournaments {
                    let pending = MatchRepository::get_pending(pool, save_id, tournament.id)
                        .await
                        .map_err(|e| e.to_string())?;

                    if pending.is_empty() {
                        continue;
                    }

                    found_pending = true;
                    let match_info = &pending[0];

                    // 特性感知模拟
                    let home_players = team_players.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let away_players = team_players.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);

                    let result = if !home_players.is_empty() && !away_players.is_empty() {
                        match_engine.simulate_match_with_traits(
                            match_info.id, match_info.tournament_id, &match_info.stage,
                            match_info.format.clone(), match_info.home_team_id, match_info.away_team_id,
                            home_players, away_players, &sim_ctx, &meta_weights,
                        )
                    } else {
                        let home_team = TeamRepository::get_by_id(pool, match_info.home_team_id)
                            .await.map_err(|e| e.to_string())?;
                        let away_team = TeamRepository::get_by_id(pool, match_info.away_team_id)
                            .await.map_err(|e| e.to_string())?;
                        self.league_service.simulate_match(
                            match_info, home_team.power_rating, away_team.power_rating,
                        )
                    };

                    // 更新比赛结果
                    MatchRepository::update_result(
                        pool,
                        match_info.id,
                        result.home_score as u32,
                        result.away_score as u32,
                        result.winner_id,
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                    // 比赛后更新 form factors
                    let home_won = result.winner_id == match_info.home_team_id;
                    let home_avg = Self::calculate_avg_performance(&result, match_info.home_team_id);
                    let away_avg = Self::calculate_avg_performance(&result, match_info.away_team_id);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.home_team_id, home_won, home_avg);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.away_team_id, !home_won, away_avg);

                    simulated_count += 1;

                    // 检查并生成下一轮对阵
                    let all_matches = MatchRepository::get_by_tournament(pool, tournament.id)
                        .await
                        .map_err(|e| e.to_string())?;

                    let new_matches =
                        self.league_service.advance_playoff_bracket(tournament.id, &all_matches);

                    if !new_matches.is_empty() {
                        log::debug!(
                            "[simulate_all_phase_matches] 季后赛生成 {} 场新比赛",
                            new_matches.len()
                        );
                        MatchRepository::create_batch(pool, save_id, &new_matches)
                            .await
                            .map_err(|e| e.to_string())?;
                    }

                    break; // 每次只模拟一场，然后重新检查
                }

                if !found_pending {
                    break;
                }
            }

            // 阶段结束，批量写回 form factors
            Self::flush_form_factors_to_db(pool, save_id, &form_factors_map).await?;

            Ok(simulated_count)
        } else {
            // 非季后赛：批量模拟
            let mut simulated_count = 0u32;

            for tournament in &tournaments {
                let pending = MatchRepository::get_pending(pool, save_id, tournament.id)
                    .await
                    .map_err(|e| e.to_string())?;

                for match_info in &pending {
                    // 特性感知模拟
                    let home_players = team_players.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let away_players = team_players.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);

                    let result = if !home_players.is_empty() && !away_players.is_empty() {
                        match_engine.simulate_match_with_traits(
                            match_info.id, match_info.tournament_id, &match_info.stage,
                            match_info.format.clone(), match_info.home_team_id, match_info.away_team_id,
                            home_players, away_players, &sim_ctx, &meta_weights,
                        )
                    } else {
                        let home_team = TeamRepository::get_by_id(pool, match_info.home_team_id)
                            .await.map_err(|e| e.to_string())?;
                        let away_team = TeamRepository::get_by_id(pool, match_info.away_team_id)
                            .await.map_err(|e| e.to_string())?;
                        self.league_service.simulate_match(
                            match_info, home_team.power_rating, away_team.power_rating,
                        )
                    };

                    MatchRepository::update_result(
                        pool,
                        match_info.id,
                        result.home_score as u32,
                        result.away_score as u32,
                        result.winner_id,
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                    // 比赛后更新 form factors
                    let home_won = result.winner_id == match_info.home_team_id;
                    let home_avg = Self::calculate_avg_performance(&result, match_info.home_team_id);
                    let away_avg = Self::calculate_avg_performance(&result, match_info.away_team_id);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.home_team_id, home_won, home_avg);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.away_team_id, !home_won, away_avg);

                    simulated_count += 1;
                }
            }

            // 阶段结束，批量写回 form factors
            Self::flush_form_factors_to_db(pool, save_id, &form_factors_map).await?;

            Ok(simulated_count)
        }
    }

    /// 预加载所有队伍的首发选手数据（含特性+动态condition），用于特性感知模拟
    async fn load_team_players(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: i64,
    ) -> Result<(HashMap<u64, Vec<MatchPlayerInfo>>, HashMap<u64, PlayerFormFactors>), String> {
        // 查询所有在役首发选手，LEFT JOIN form factors
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.ability, p.stability, p.age, p.position, p.team_id, p.join_season,
                   COALESCE(pff.form_cycle, 50.0) as form_cycle,
                   COALESCE(pff.momentum, 0) as momentum,
                   COALESCE(pff.last_performance, 0.0) as last_performance,
                   COALESCE(pff.last_match_won, 0) as last_match_won,
                   COALESCE(pff.games_since_rest, 0) as games_since_rest
            FROM players p
            LEFT JOIN player_form_factors pff ON p.id = pff.player_id AND pff.save_id = ?
            WHERE p.save_id = ? AND p.status = 'Active' AND p.is_starter = 1
            "#,
        )
        .bind(save_id)
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("加载选手数据失败: {}", e))?;

        // 收集所有 player_id
        let player_ids: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>("id")).collect();

        // 批量查询特性
        let mut player_traits_map: HashMap<u64, Vec<TraitType>> = HashMap::new();
        if !player_ids.is_empty() {
            let placeholders: String = player_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!(
                "SELECT player_id, trait_type FROM player_traits WHERE save_id = ? AND player_id IN ({})",
                placeholders
            );
            let mut query = sqlx::query(&query_str).bind(save_id);
            for pid in &player_ids {
                query = query.bind(pid);
            }
            let trait_rows = query.fetch_all(pool).await.map_err(|e| format!("加载特性失败: {}", e))?;

            for row in &trait_rows {
                let pid = row.get::<i64, _>("player_id") as u64;
                let trait_str: String = row.get("trait_type");
                if let Some(tt) = TraitType::from_str(&trait_str) {
                    player_traits_map.entry(pid).or_default().push(tt);
                }
            }
        }

        // 按 team_id 分组，同时构建 form_factors_map
        let mut team_players: HashMap<u64, Vec<MatchPlayerInfo>> = HashMap::new();
        let mut form_factors_map: HashMap<u64, PlayerFormFactors> = HashMap::new();
        for row in &rows {
            let player_id = row.get::<i64, _>("id") as u64;
            let team_id = row.get::<i64, _>("team_id") as u64;
            let join_season: i64 = row.get("join_season");
            let ability = row.get::<i64, _>("ability") as u8;
            let age = row.get::<i64, _>("age") as u8;

            // 构建 form factors
            let factors = PlayerFormFactors {
                player_id,
                form_cycle: row.get::<f64, _>("form_cycle"),
                momentum: row.get::<i64, _>("momentum") as i8,
                last_performance: row.get::<f64, _>("last_performance"),
                last_match_won: row.get::<i64, _>("last_match_won") != 0,
                games_since_rest: row.get::<i64, _>("games_since_rest") as u32,
            };

            // 动态计算 condition
            let condition = ConditionEngine::calculate_condition(age, ability, &factors, None);

            form_factors_map.insert(player_id, factors);

            let player_info = MatchPlayerInfo {
                player_id,
                ability,
                stability: row.get::<i64, _>("stability") as u8,
                condition,
                age,
                position: row.get::<String, _>("position"),
                traits: player_traits_map.get(&player_id).cloned().unwrap_or_default(),
                is_first_season: join_season == current_season,
            };

            team_players.entry(team_id).or_default().push(player_info);
        }

        Ok((team_players, form_factors_map))
    }

    /// 比赛后更新内存中的 form factors 并重算 condition
    fn update_form_factors_after_match(
        team_players: &mut HashMap<u64, Vec<MatchPlayerInfo>>,
        form_factors_map: &mut HashMap<u64, PlayerFormFactors>,
        team_id: u64,
        won: bool,
        avg_performance: f64,
    ) {
        if let Some(players) = team_players.get_mut(&team_id) {
            for player in players.iter_mut() {
                if let Some(factors) = form_factors_map.remove(&player.player_id) {
                    let updated = ConditionEngine::update_form_factors(factors, won, avg_performance);
                    let new_condition = ConditionEngine::calculate_condition(
                        player.age, player.ability, &updated, None,
                    );
                    player.condition = new_condition;
                    form_factors_map.insert(player.player_id, updated);
                }
            }
        }
    }

    /// 批量将 form factors 写回数据库
    async fn flush_form_factors_to_db(
        pool: &Pool<Sqlite>,
        save_id: &str,
        form_factors_map: &HashMap<u64, PlayerFormFactors>,
    ) -> Result<(), String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        for (player_id, factors) in form_factors_map {
            sqlx::query(
                r#"
                INSERT INTO player_form_factors (save_id, player_id, form_cycle, momentum, last_performance, last_match_won, games_since_rest, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
                ON CONFLICT(save_id, player_id) DO UPDATE SET
                    form_cycle = excluded.form_cycle,
                    momentum = excluded.momentum,
                    last_performance = excluded.last_performance,
                    last_match_won = excluded.last_match_won,
                    games_since_rest = excluded.games_since_rest,
                    updated_at = datetime('now')
                "#,
            )
            .bind(save_id)
            .bind(*player_id as i64)
            .bind(factors.form_cycle)
            .bind(factors.momentum as i64)
            .bind(factors.last_performance)
            .bind(if factors.last_match_won { 1i64 } else { 0i64 })
            .bind(factors.games_since_rest as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 从 MatchResult 中计算某队的平均 performance
    fn calculate_avg_performance(result: &crate::models::MatchResult, team_id: u64) -> f64 {
        if result.games.is_empty() {
            return 0.0;
        }
        let total: f64 = result.games.iter().map(|g| {
            if g.winner_id == team_id {
                // 该队赢的局，取对应方的 performance
                if team_id == result.match_info.home_team_id {
                    g.home_performance
                } else {
                    g.away_performance
                }
            } else {
                if team_id == result.match_info.home_team_id {
                    g.home_performance
                } else {
                    g.away_performance
                }
            }
        }).sum();
        total / result.games.len() as f64
    }
}

// ===== 年度颁奖典礼辅助结构和方法 =====

/// 年度选手信息结构
#[allow(dead_code)]
struct AnnualPlayerInfo {
    player_id: u64,
    player_name: String,
    team_id: u64,
    team_name: String,
    position: String,
    yearly_score: f64,
    age: u8,
    consistency_score: f64,
    dominance_score: f64,
    games_played: i32,
}

impl GameFlowService {
    /// 获取年度Top20选手
    async fn get_annual_top20(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<AnnualPlayerInfo>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                COALESCE(p.age, 0) as age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.games_played >= 10
            ORDER BY pss.yearly_top_score DESC
            LIMIT 20
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get top20: {}", e))?;

        Ok(rows.iter().map(|row| AnnualPlayerInfo {
            player_id: row.get::<i64, _>("player_id") as u64,
            player_name: row.get::<String, _>("player_name"),
            team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: row.get::<String, _>("team_name"),
            position: row.get::<String, _>("position"),
            yearly_score: row.get::<f64, _>("yearly_top_score"),
            age: row.get::<i64, _>("age") as u8,
            consistency_score: row.get::<f64, _>("consistency_score"),
            dominance_score: row.get::<f64, _>("dominance_score"),
            games_played: row.get::<i32, _>("games_played"),
        }).collect())
    }

    /// 获取年度最佳阵容三阵（每位置Top3）
    async fn get_annual_all_pro_3teams(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<(u8, AnnualPlayerInfo)>, String> {
        let positions = vec!["TOP", "JUG", "MID", "ADC", "SUP"];
        let mut results = Vec::new();

        for position in positions {
            let rows = sqlx::query(
                r#"
                SELECT
                    pss.player_id,
                    pss.player_name,
                    pss.team_id,
                    COALESCE(t.name, '未知') as team_name,
                    pss.position,
                    pss.yearly_top_score,
                    COALESCE(p.age, 0) as age,
                    pss.consistency_score,
                    COALESCE(pss.dominance_score, 0.0) as dominance_score,
                    pss.games_played
                FROM player_season_stats pss
                LEFT JOIN teams t ON pss.team_id = t.id
                LEFT JOIN players p ON pss.player_id = p.id
                WHERE pss.save_id = ? AND pss.season_id = ? AND pss.position = ? AND pss.games_played >= 10
                ORDER BY pss.yearly_top_score DESC
                LIMIT 3
                "#
            )
            .bind(save_id)
            .bind(season_id as i64)
            .bind(position)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get best {}: {}", position, e))?;

            for (tier_idx, row) in rows.iter().enumerate() {
                let tier = (tier_idx + 1) as u8;
                results.push((tier, AnnualPlayerInfo {
                    player_id: row.get::<i64, _>("player_id") as u64,
                    player_name: row.get::<String, _>("player_name"),
                    team_id: row.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
                    team_name: row.get::<String, _>("team_name"),
                    position: row.get::<String, _>("position"),
                    yearly_score: row.get::<f64, _>("yearly_top_score"),
                    age: row.get::<i64, _>("age") as u8,
                    consistency_score: row.get::<f64, _>("consistency_score"),
                    dominance_score: row.get::<f64, _>("dominance_score"),
                    games_played: row.get::<i32, _>("games_played"),
                }));
            }
        }

        Ok(results)
    }

    /// 获取年度最稳定选手（consistency_score最高，>=30场）
    async fn get_annual_most_consistent(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Option<AnnualPlayerInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                COALESCE(p.age, 0) as age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.games_played >= 30
            ORDER BY pss.consistency_score DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get most consistent: {}", e))?;

        Ok(row.map(|r| AnnualPlayerInfo {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            age: r.get::<i64, _>("age") as u8,
            consistency_score: r.get::<f64, _>("consistency_score"),
            dominance_score: r.get::<f64, _>("dominance_score"),
            games_played: r.get::<i32, _>("games_played"),
        }))
    }

    /// 获取年度最具统治力选手（dominance_score最高，>=20场）
    async fn get_annual_most_dominant(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Option<AnnualPlayerInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                COALESCE(p.age, 0) as age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            LEFT JOIN teams t ON pss.team_id = t.id
            LEFT JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.games_played >= 20
            ORDER BY COALESCE(pss.dominance_score, 0.0) DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get most dominant: {}", e))?;

        Ok(row.map(|r| AnnualPlayerInfo {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            age: r.get::<i64, _>("age") as u8,
            consistency_score: r.get::<f64, _>("consistency_score"),
            dominance_score: r.get::<f64, _>("dominance_score"),
            games_played: r.get::<i32, _>("games_played"),
        }))
    }

    /// 获取年度最佳新秀（20岁及以下）
    async fn get_annual_rookie(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Option<AnnualPlayerInfo>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                pss.player_id,
                pss.player_name,
                pss.team_id,
                COALESCE(t.name, '未知') as team_name,
                pss.position,
                pss.yearly_top_score,
                p.age,
                pss.consistency_score,
                COALESCE(pss.dominance_score, 0.0) as dominance_score,
                pss.games_played
            FROM player_season_stats pss
            JOIN players p ON pss.player_id = p.id
            LEFT JOIN teams t ON pss.team_id = t.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND p.age <= 20 AND pss.games_played >= 10
            ORDER BY pss.yearly_top_score DESC
            LIMIT 1
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get rookie of the year: {}", e))?;

        Ok(row.map(|r| AnnualPlayerInfo {
            player_id: r.get::<i64, _>("player_id") as u64,
            player_name: r.get::<String, _>("player_name"),
            team_id: r.get::<Option<i64>, _>("team_id").unwrap_or(0) as u64,
            team_name: r.get::<String, _>("team_name"),
            position: r.get::<String, _>("position"),
            yearly_score: r.get::<f64, _>("yearly_top_score"),
            age: r.get::<i64, _>("age") as u8,
            consistency_score: r.get::<f64, _>("consistency_score"),
            dominance_score: r.get::<f64, _>("dominance_score"),
            games_played: r.get::<i32, _>("games_played"),
        }))
    }

    /// 计算选手的荣誉系数（查询 DB + 委托 MarketValueEngine）
    async fn calculate_honor_factor(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
        current_season: u32,
    ) -> Result<f64, String> {
        let rows = sqlx::query(
            r#"
            SELECT honor_type, tournament_type, tournament_name, season_id
            FROM honors
            WHERE save_id = ? AND player_id = ?
            "#
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get player honors: {}", e))?;

        let honors: Vec<PlayerHonorRecord> = rows.iter().filter_map(|row| {
            let honor_type: String = row.get("honor_type");
            let tournament_type: String = row.get::<Option<String>, _>("tournament_type").unwrap_or_default();
            let tournament_name: String = row.get::<Option<String>, _>("tournament_name").unwrap_or_default();
            let season_obtained: u32 = row.get::<i64, _>("season_id") as u32;
            MarketValueEngine::parse_honor_category(&honor_type, &tournament_type, &tournament_name)
                .map(|cat| PlayerHonorRecord::new(cat, season_obtained, &tournament_name))
        }).collect();

        Ok(MarketValueEngine::calculate_honor_factor(&honors, current_season))
    }

    /// 完整重算单个选手身价（基础 × 荣誉 × 赛区）
    async fn recalculate_player_market_value_full(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        player_id: u64,
    ) -> Result<(), String> {
        // 获取选手信息
        let row = sqlx::query(
            r#"
            SELECT p.game_id, p.ability, p.potential, p.age, p.tag, p.position,
                   p.market_value, p.calculated_market_value,
                   r.short_name as region_code
            FROM players p
            LEFT JOIN teams t ON p.team_id = t.id
            LEFT JOIN regions r ON t.region_id = r.id
            WHERE p.id = ?
            "#
        )
        .bind(player_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get player: {}", e))?;

        let row = match row {
            Some(r) => r,
            None => return Ok(()),
        };

        let player_name: String = row.get("game_id");
        let ability: i64 = row.get("ability");
        let potential: i64 = row.get("potential");
        let age: i64 = row.get("age");
        let tag: String = row.get("tag");
        let position: String = row.get("position");
        let old_calculated: i64 = row.get("calculated_market_value");
        let region_code: String = row.get::<Option<String>, _>("region_code").unwrap_or_else(|| "LPL".to_string());

        // 计算基础身价（委托引擎）
        let base_value = MarketValueEngine::calculate_base_market_value(
            ability as u8, age as u8, potential as u8, &tag, &position
        );

        // 计算荣誉系数（委托引擎）
        let honor_factor = self.calculate_honor_factor(pool, save_id, player_id, season_id as u32).await?;

        // 查询赛季表现
        let perf_factor = {
            let stats_row = sqlx::query(
                r#"SELECT avg_impact, consistency_score, games_played
                   FROM player_season_stats
                   WHERE save_id = ? AND player_id = ? AND season_id = ?"#
            )
            .bind(save_id)
            .bind(player_id as i64)
            .bind(season_id as i64)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            if let Some(sr) = stats_row {
                let avg_impact: f64 = sr.try_get("avg_impact").unwrap_or(0.0);
                let consistency: f64 = sr.try_get("consistency_score").unwrap_or(60.0);
                let gp: i64 = sr.try_get("games_played").unwrap_or(0);
                MarketValueEngine::performance_factor(avg_impact, consistency, gp as u32)
            } else {
                1.0
            }
        };

        // 完整身价（委托引擎）× 表现系数
        let new_value = (MarketValueEngine::calculate_full_market_value(base_value, honor_factor, &region_code) as f64 * perf_factor) as i64;

        if new_value != old_calculated {
            // 同时更新两个列：market_value = 基础值，calculated_market_value = 完整值
            sqlx::query("UPDATE players SET market_value = ?, calculated_market_value = ? WHERE id = ?")
                .bind(base_value as i64)
                .bind(new_value)
                .bind(player_id as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update market value: {}", e))?;

            // 记录变化
            let change_amount = new_value - old_calculated;
            let change_percent = if old_calculated > 0 {
                ((new_value as f64 / old_calculated as f64) - 1.0) * 100.0
            } else {
                100.0
            };

            sqlx::query(
                r#"INSERT INTO market_value_changes
                   (save_id, season_id, player_id, player_name, old_value, new_value, change_amount, change_percent, reason)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#
            )
            .bind(save_id)
            .bind(season_id as i64)
            .bind(player_id as i64)
            .bind(&player_name)
            .bind(old_calculated)
            .bind(new_value)
            .bind(change_amount)
            .bind(change_percent)
            .bind("年度身价重算")
            .execute(pool)
            .await
            .ok();

            log::debug!("{} 身价重算: {} -> {} (荣誉×{:.2}, 赛区×{})",
                player_name, old_calculated / 10000, new_value / 10000, honor_factor, &region_code);
        }

        Ok(())
    }

    /// 年度结束时重算所有选手身价
    pub async fn recalculate_all_market_values(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<u32, String> {
        // 获取所有活跃选手
        let rows = sqlx::query("SELECT id FROM players WHERE save_id = ? AND status = 'Active'")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Failed to get players: {}", e))?;

        let mut count = 0u32;
        for row in rows {
            let player_id: i64 = row.get("id");
            self.recalculate_player_market_value_full(pool, save_id, season_id, player_id as u64).await?;
            count += 1;
        }

        log::debug!("完成 {} 名选手身价重算", count);
        Ok(count)
    }

    /// 更新所有队伍的品牌价值
    async fn update_all_brand_values(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<u32, String> {
        // 获取所有队伍
        let team_rows = sqlx::query(
            "SELECT id, brand_value FROM teams WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get teams: {}", e))?;

        let mut count = 0u32;

        for row in &team_rows {
            let team_id: i64 = row.get("id");
            let old_brand: f64 = row.get("brand_value");

            // 15%自然衰减
            let mut new_brand = old_brand * 0.85;

            // 查询本赛季该队伍的所有荣誉记录
            let honors = sqlx::query(
                r#"
                SELECT h.honor_type, t.tournament_type
                FROM honors h
                LEFT JOIN tournaments t ON h.tournament_id = t.id
                WHERE h.save_id = ? AND h.season_id = ? AND h.team_id = ?
                  AND h.honor_type IN ('TEAM_CHAMPION', 'TEAM_RUNNER_UP', 'TEAM_THIRD', 'TEAM_FOURTH')
                "#
            )
            .bind(save_id)
            .bind(season_id as i64)
            .bind(team_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            for honor in &honors {
                let honor_type: String = honor.get("honor_type");
                let tournament_type: Option<String> = honor.try_get("tournament_type").ok();
                let tt = tournament_type.as_deref().unwrap_or("");

                let bonus = match (tt, honor_type.as_str()) {
                    // 世界赛
                    ("WorldChampionship", "TEAM_CHAMPION") => 25.0,
                    ("WorldChampionship", "TEAM_RUNNER_UP") => 15.0,
                    ("WorldChampionship", "TEAM_THIRD") => 10.0,
                    ("WorldChampionship", "TEAM_FOURTH") => 5.0,
                    // Super洲际赛
                    ("SuperIntercontinental", "TEAM_CHAMPION") => 20.0,
                    ("SuperIntercontinental", "TEAM_RUNNER_UP") => 12.0,
                    ("SuperIntercontinental", "TEAM_THIRD") => 8.0,
                    ("SuperIntercontinental", "TEAM_FOURTH") => 5.0,
                    // MSI
                    ("Msi", "TEAM_CHAMPION") => 10.0,
                    ("Msi", "TEAM_RUNNER_UP") => 6.0,
                    ("Msi", "TEAM_THIRD") => 3.0,
                    // 联赛季后赛（夏季）
                    ("SummerPlayoffs", "TEAM_CHAMPION") => 8.0,
                    ("SummerPlayoffs", "TEAM_RUNNER_UP") => 4.0,
                    ("SummerPlayoffs", "TEAM_THIRD") => 3.0,
                    // 联赛季后赛（春季）
                    ("SpringPlayoffs", "TEAM_CHAMPION") => 5.0,
                    ("SpringPlayoffs", "TEAM_RUNNER_UP") => 3.0,
                    ("SpringPlayoffs", "TEAM_THIRD") => 2.0,
                    // 其他国际赛事四强
                    (_, "TEAM_CHAMPION") => 5.0,
                    (_, "TEAM_RUNNER_UP") => 3.0,
                    (_, "TEAM_THIRD") | (_, "TEAM_FOURTH") => 2.0,
                    _ => 0.0,
                };

                new_brand += bonus;
            }

            new_brand = new_brand.clamp(0.0, 100.0);

            // 更新品牌价值
            sqlx::query("UPDATE teams SET brand_value = ? WHERE id = ? AND save_id = ?")
                .bind(new_brand)
                .bind(team_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update brand_value: {}", e))?;

            if (new_brand - old_brand).abs() > 0.1 {
                log::debug!("队伍 {} 品牌价值: {:.1} -> {:.1}", team_id, old_brand, new_brand);
            }

            count += 1;
        }

        Ok(count)
    }
}

/// 获取赛区名称
fn get_region_name(region_id: u64) -> &'static str {
    match region_id {
        1 => "LPL",
        2 => "LCK",
        3 => "LEC",
        4 => "LCS",
        _ => "Unknown",
    }
}

/// 解析赛事状态（本地版本避免循环依赖）
fn parse_tournament_status_local(s: &str) -> TournamentStatus {
    match s {
        "Upcoming" => TournamentStatus::Upcoming,
        "InProgress" => TournamentStatus::InProgress,
        "Completed" => TournamentStatus::Completed,
        _ => TournamentStatus::Upcoming,
    }
}

/// 位置转排名数字
fn position_to_rank(position: &str) -> Option<u32> {
    match position {
        "CHAMPION" => Some(1),
        "RUNNER_UP" => Some(2),
        "THIRD" => Some(3),
        "FOURTH" => Some(4),
        "5TH_8TH" | "QUARTER_FINAL" => Some(5),
        // ICP积分位置
        "FIRST_PARTICIPANT" | "FIRST_NON_PARTICIPANT" => Some(1),
        "SECOND_PARTICIPANT" | "SECOND_NON_PARTICIPANT" => Some(2),
        "THIRD_PARTICIPANT" | "THIRD_NON_PARTICIPANT" => Some(3),
        "FOURTH_PARTICIPANT" | "FOURTH_NON_PARTICIPANT" => Some(4),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_region_name() {
        assert_eq!(get_region_name(1), "LPL");
        assert_eq!(get_region_name(2), "LCK");
        assert_eq!(get_region_name(3), "LEC");
        assert_eq!(get_region_name(4), "LCS");
        assert_eq!(get_region_name(99), "Unknown");
    }

    #[test]
    fn test_phase_to_tournament_type() {
        assert_eq!(
            SeasonPhase::SpringRegular.to_tournament_type(),
            Some(TournamentType::SpringRegular)
        );
        assert_eq!(
            SeasonPhase::Msi.to_tournament_type(),
            Some(TournamentType::Msi)
        );
        assert_eq!(SeasonPhase::TransferWindow.to_tournament_type(), None);
        assert_eq!(SeasonPhase::Draft.to_tournament_type(), None);
    }

    #[test]
    fn test_get_phase_display_name() {
        assert_eq!(
            SeasonPhase::SpringRegular.display_name(),
            "春季常规赛"
        );
        assert_eq!(SeasonPhase::Msi.display_name(), "MSI季中赛");
        assert_eq!(
            SeasonPhase::WorldChampionship.display_name(),
            "世界赛"
        );
    }
}
