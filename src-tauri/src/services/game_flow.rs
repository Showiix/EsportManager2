use crate::db::{
    EventRepository, MatchRepository, PlayerRepository, SaveRepository,
    StandingRepository, TeamRepository, TournamentRepository,
};
use crate::engines::EventEngine;
use crate::models::{
    EventType, GameEvent, LeagueStanding, SeasonPhase, Tournament, TournamentStatus,
    TournamentType, GameTimeState, PhaseStatus, PhaseProgress, TournamentProgress,
    SeasonProgress, PhaseInfo, TimeAction, FastForwardTarget, FastForwardResult,
    CompleteAndAdvanceResult, HonorInfo,
};
use crate::services::{LeagueService, HonorService};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};

/// 游戏流程服务 - 整合赛季流程控制
pub struct GameFlowService {
    league_service: LeagueService,
    event_engine: EventEngine,
    honor_service: HonorService,
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

impl Default for GameFlowService {
    fn default() -> Self {
        Self {
            league_service: LeagueService::new(),
            event_engine: EventEngine::new(),
            honor_service: HonorService::new(),
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
        let mut tournaments_created = Vec::new();

        match phase {
            // 春季常规赛 - 为4个赛区各创建一个常规赛赛事
            SeasonPhase::SpringRegular => {
                for region_id in 1..=4 {
                    let region_name = get_region_name(region_id);
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::SpringRegular,
                        name: format!("S{} {} 春季赛", season_id, region_name),
                        region_id: Some(region_id),
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 获取赛区队伍并生成赛程
                    let teams = TeamRepository::get_by_region(pool, save_id, region_id)
                        .await
                        .map_err(|e| e.to_string())?;

                    if teams.len() >= 8 {
                        let matches = self
                            .league_service
                            .generate_regular_schedule(id, &teams);
                        MatchRepository::create_batch(pool, save_id, &matches)
                            .await
                            .map_err(|e| e.to_string())?;

                        // 初始化积分榜
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
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: tournament.name,
                        tournament_type: format!("{:?}", TournamentType::SpringRegular),
                        region: Some(region_name.to_string()),
                    });
                }
            }

            // 春季季后赛 - 为4个赛区各创建季后赛
            SeasonPhase::SpringPlayoffs => {
                for region_id in 1..=4 {
                    let region_name = get_region_name(region_id);
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::SpringPlayoffs,
                        name: format!("S{} {} 春季季后赛", season_id, region_name),
                        region_id: Some(region_id),
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 从春季常规赛积分榜获取前8名队伍
                    // TODO: 实现季后赛对阵生成

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: tournament.name,
                        tournament_type: format!("{:?}", TournamentType::SpringPlayoffs),
                        region: Some(region_name.to_string()),
                    });
                }
            }

            // MSI - 创建全球性赛事
            SeasonPhase::Msi => {
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

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::Msi),
                    region: None,
                });
            }

            // 马德里大师赛
            SeasonPhase::MadridMasters => {
                let tournament = Tournament {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    tournament_type: TournamentType::MadridMasters,
                    name: format!("S{} 马德里大师赛", season_id),
                    region_id: None,
                    status: TournamentStatus::Upcoming,
                    current_stage: None,
                    current_round: None,
                };

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::MadridMasters),
                    region: None,
                });
            }

            // 夏季常规赛
            SeasonPhase::SummerRegular => {
                for region_id in 1..=4 {
                    let region_name = get_region_name(region_id);
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::SummerRegular,
                        name: format!("S{} {} 夏季赛", season_id, region_name),
                        region_id: Some(region_id),
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;

                    // 生成赛程
                    let teams = TeamRepository::get_by_region(pool, save_id, region_id)
                        .await
                        .map_err(|e| e.to_string())?;

                    if teams.len() >= 8 {
                        let matches = self
                            .league_service
                            .generate_regular_schedule(id, &teams);
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
                    }

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: tournament.name,
                        tournament_type: format!("{:?}", TournamentType::SummerRegular),
                        region: Some(region_name.to_string()),
                    });
                }
            }

            // 夏季季后赛
            SeasonPhase::SummerPlayoffs => {
                for region_id in 1..=4 {
                    let region_name = get_region_name(region_id);
                    let tournament = Tournament {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        tournament_type: TournamentType::SummerPlayoffs,
                        name: format!("S{} {} 夏季季后赛", season_id, region_name),
                        region_id: Some(region_id),
                        status: TournamentStatus::Upcoming,
                        current_stage: None,
                        current_round: None,
                    };

                    let id = TournamentRepository::create(pool, save_id, &tournament)
                        .await
                        .map_err(|e| e.to_string())?;

                    tournaments_created.push(TournamentCreated {
                        id,
                        name: tournament.name,
                        tournament_type: format!("{:?}", TournamentType::SummerPlayoffs),
                        region: Some(region_name.to_string()),
                    });
                }
            }

            // Claude洲际赛
            SeasonPhase::ClaudeIntercontinental => {
                let tournament = Tournament {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    tournament_type: TournamentType::ClaudeIntercontinental,
                    name: format!("S{} Claude洲际赛", season_id),
                    region_id: None,
                    status: TournamentStatus::Upcoming,
                    current_stage: None,
                    current_round: None,
                };

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::ClaudeIntercontinental),
                    region: None,
                });
            }

            // 世界赛
            SeasonPhase::WorldChampionship => {
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

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::WorldChampionship),
                    region: None,
                });
            }

            // 上海大师赛
            SeasonPhase::ShanghaiMasters => {
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

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::ShanghaiMasters),
                    region: None,
                });
            }

            // ICP洲际对抗赛
            SeasonPhase::IcpIntercontinental => {
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

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::IcpIntercontinental),
                    region: None,
                });
            }

            // Super洲际邀请赛
            SeasonPhase::SuperIntercontinental => {
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

                let id = TournamentRepository::create(pool, save_id, &tournament)
                    .await
                    .map_err(|e| e.to_string())?;

                tournaments_created.push(TournamentCreated {
                    id,
                    name: tournament.name,
                    tournament_type: format!("{:?}", TournamentType::SuperIntercontinental),
                    region: None,
                });
            }

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
            SeasonPhase::TransferWindow | SeasonPhase::Draft | SeasonPhase::SeasonEnd => {
                // 这些阶段需要手动确认完成
                Ok(false)
            }
            _ => {
                // 检查该阶段所有赛事是否完成
                let tournament_type = phase_to_tournament_type(phase);
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
        let mut honors_awarded = Vec::new();

        // 根据阶段颁发荣誉
        match phase {
            // 常规赛结束 - 颁发常规赛第一名和常规赛MVP
            SeasonPhase::SpringRegular | SeasonPhase::SummerRegular => {
                let tournament_type = phase_to_tournament_type(phase);
                if let Some(t_type) = tournament_type {
                    // 获取该阶段的所有赛事
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;

                    for tournament in tournaments {
                        match self.honor_service.process_regular_season_honors(pool, save_id, tournament.id).await {
                            Ok(honors) => {
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
                                eprintln!("Failed to process regular season honors for tournament {}: {}", tournament.id, e);
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
                let tournament_type = phase_to_tournament_type(phase);
                if let Some(t_type) = tournament_type {
                    let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;

                    for tournament in tournaments {
                        match self.honor_service.process_tournament_completion(pool, save_id, tournament.id).await {
                            Ok(tournament_honors) => {
                                // 收集战队荣誉
                                if let Some(ref honor) = tournament_honors.team_champion {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: "冠军".to_string(),
                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                                if let Some(ref honor) = tournament_honors.team_runner_up {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: "亚军".to_string(),
                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                                if let Some(ref honor) = tournament_honors.team_third {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: "季军".to_string(),
                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                                if let Some(ref honor) = tournament_honors.team_fourth {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: "殿军".to_string(),
                                        recipient_name: honor.team_name.clone().unwrap_or_default(),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                                // 收集MVP荣誉
                                if let Some(ref honor) = tournament_honors.tournament_mvp {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: "赛事MVP".to_string(),
                                        recipient_name: honor.player_name.clone().unwrap_or_default(),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                                if let Some(ref honor) = tournament_honors.finals_mvp {
                                    honors_awarded.push(HonorAwarded {
                                        honor_type: "决赛MVP".to_string(),
                                        recipient_name: honor.player_name.clone().unwrap_or_default(),
                                        tournament_name: honor.tournament_name.clone(),
                                    });
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to process tournament honors for tournament {}: {}", tournament.id, e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // 获取下一阶段
        let next_phase = phase.next();
        let can_advance = next_phase.is_some() || phase == SeasonPhase::SeasonEnd;

        let message = match phase {
            SeasonPhase::SeasonEnd => "赛季结束，准备开始新赛季".to_string(),
            _ => {
                if let Some(next) = next_phase {
                    format!("阶段完成，下一阶段: {}", get_phase_display_name(&next))
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

    /// 执行赛季结算
    pub async fn execute_season_settlement(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: u32,
    ) -> Result<SeasonSettlementResult, String> {
        // 获取所有活跃选手
        let players = PlayerRepository::get_all_active(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        // 获取所有队伍
        let teams = TeamRepository::get_all(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        // 执行赛季结算
        let settlement = self.event_engine.process_season_settlement(
            current_season as u64,
            &players,
            &teams,
            current_season,
        );

        let mut events = Vec::new();

        // 处理选手成长
        let mut ability_updates = Vec::new();
        for growth in &settlement.growth_events {
            ability_updates.push((growth.player_id, growth.new_ability));
            events.push(format!(
                "成长: {} 能力 {} -> {}",
                growth.player_name, growth.old_ability, growth.new_ability
            ));
        }
        if !ability_updates.is_empty() {
            PlayerRepository::batch_update_ability(pool, &ability_updates)
                .await
                .map_err(|e| e.to_string())?;
        }

        // 处理选手衰退
        let mut decline_updates = Vec::new();
        for decline in &settlement.decline_events {
            decline_updates.push((decline.player_id, decline.new_ability));
            events.push(format!(
                "衰退: {} 能力 {} -> {}",
                decline.player_name, decline.old_ability, decline.new_ability
            ));
        }
        if !decline_updates.is_empty() {
            PlayerRepository::batch_update_ability(pool, &decline_updates)
                .await
                .map_err(|e| e.to_string())?;
        }

        // 处理退役
        let retire_ids: Vec<u64> = settlement
            .retirement_events
            .iter()
            .map(|r| r.player_id)
            .collect();
        if !retire_ids.is_empty() {
            PlayerRepository::batch_retire(pool, &retire_ids, current_season)
                .await
                .map_err(|e| e.to_string())?;
            for retire in &settlement.retirement_events {
                events.push(format!(
                    "退役: {} ({:?})",
                    retire.player_name, retire.reason
                ));
            }
        }

        // 处理合同到期
        let contract_updates: Vec<(u64, bool, Option<u32>, Option<u64>)> = settlement
            .contract_expire_events
            .iter()
            .map(|c| {
                (
                    c.player_id,
                    c.renewed,
                    c.new_contract_years,
                    c.new_salary,
                )
            })
            .collect();
        if !contract_updates.is_empty() {
            PlayerRepository::batch_update_contracts(pool, &contract_updates, current_season)
                .await
                .map_err(|e| e.to_string())?;
            for contract in &settlement.contract_expire_events {
                if contract.renewed {
                    events.push(format!(
                        "续约: {} 续约 {} 年",
                        contract.player_name,
                        contract.new_contract_years.unwrap_or(0)
                    ));
                } else {
                    events.push(format!("合同到期: {} 成为自由球员", contract.player_name));
                }
            }
        }

        // 记录新秀信息 (新秀由 EventEngine 生成)
        let rookies_count = settlement.rookie_events.len() as u32;
        for rookie in &settlement.rookie_events {
            events.push(format!(
                "新秀: {} ({}) 能力:{} 潜力:{}",
                rookie.player_name, rookie.position, rookie.ability, rookie.potential
            ));
        }

        // 记录事件到数据库
        for event_desc in &events {
            let game_event = GameEvent {
                id: 0,
                save_id: save_id.to_string(),
                season_id: current_season as u64,
                event_type: EventType::SeasonSettlement,
                player_id: None,
                team_id: None,
                description: event_desc.clone(),
                details: None,
                phase: Some("SeasonEnd".to_string()),
            };

            EventRepository::create(pool, &game_event)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(SeasonSettlementResult {
            season: current_season,
            players_grown: settlement.growth_events.len() as u32,
            players_declined: settlement.decline_events.len() as u32,
            players_retired: settlement.retirement_events.len() as u32,
            contracts_expired: settlement.contract_expire_events.len() as u32,
            rookies_generated: rookies_count,
            events,
        })
    }

    /// 推进到新赛季
    pub async fn advance_to_new_season(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<u32, String> {
        // 获取当前存档
        let mut save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        // 更新赛季
        save.current_season += 1;
        save.current_phase = SeasonPhase::SpringRegular;
        save.phase_completed = false;
        save.updated_at = chrono::Utc::now();

        SaveRepository::update(pool, &save)
            .await
            .map_err(|e| e.to_string())?;

        // 重置所有队伍的年度积分
        let teams = TeamRepository::get_all(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        for mut team in teams {
            // 将年度积分转移到跨年积分（半衰期）
            team.cross_year_points = team.cross_year_points / 2 + team.annual_points / 2;
            team.annual_points = 0;

            TeamRepository::update(pool, &team)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(save.current_season)
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
        let phase_status = if phase_progress.total_matches == 0 && !is_non_tournament_phase(current_phase) {
            PhaseStatus::NotInitialized
        } else if phase_progress.completed_matches >= phase_progress.total_matches {
            PhaseStatus::Completed
        } else {
            PhaseStatus::InProgress
        };

        // 获取赛季进度
        let season_progress = self.get_season_progress(current_phase);

        // 获取可用操作
        let available_actions = self.get_available_actions(current_phase, &phase_status);

        // 判断是否可以推进
        let can_advance = phase_status == PhaseStatus::Completed;

        // 获取下一阶段
        let next_phase = current_phase.next().map(|p| p.name().to_string());

        Ok(GameTimeState {
            save_id: save_id.to_string(),
            current_season,
            current_phase,
            phase_display_name: current_phase.name().to_string(),
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
        let tournament_type = phase_to_tournament_type(phase);

        if let Some(t_type) = tournament_type {
            let tournaments = self.get_phase_tournaments(pool, save_id, season_id, t_type).await?;

            let mut tournament_progress_list = Vec::new();
            let mut total_matches = 0u32;
            let mut completed_matches = 0u32;

            for tournament in tournaments {
                // 查询比赛数量
                let counts = sqlx::query(
                    r#"
                    SELECT
                        COUNT(*) as total,
                        SUM(CASE WHEN status = 'COMPLETED' THEN 1 ELSE 0 END) as completed
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
                    region: tournament.region_id.map(|r| get_region_name(r).to_string()),
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
            // 非赛事阶段（转会期、选秀、赛季结束）
            Ok(PhaseProgress {
                tournaments: Vec::new(),
                total_matches: 0,
                completed_matches: 0,
                percentage: 0.0,
            })
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
                display_name: phase.name().to_string(),
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
                actions.push(TimeAction::StartTransferWindow);
                actions.push(TimeAction::ExecuteTransferRound);
            }
            SeasonPhase::Draft => {
                actions.push(TimeAction::StartDraft);
            }
            SeasonPhase::SeasonEnd => {
                actions.push(TimeAction::ExecuteSeasonSettlement);
                actions.push(TimeAction::StartNewSeason);
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
            if is_before_phase(phase, SeasonPhase::SummerRegular) {
                actions.push(TimeAction::FastForwardToSummer);
            }
            if is_before_phase(phase, SeasonPhase::WorldChampionship) {
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

        // 完成当前阶段（颁发荣誉）
        let complete_result = self.complete_phase(pool, save_id, season_id, current_phase).await?;

        let mut honors_awarded: Vec<HonorInfo> = complete_result.honors_awarded.iter().map(|h| {
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
            let init_result = self.initialize_phase(pool, save_id, season_id, next).await?;

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
                    // 模拟所有比赛
                    let matches_simulated = self.simulate_all_phase_matches(pool, save_id, current_phase).await?;
                    total_matches_simulated += matches_simulated;
                }
                PhaseStatus::Completed => {
                    // 完成并推进
                    let result = self.complete_and_advance(pool, save_id).await?;
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

    /// 模拟当前阶段的所有比赛（简化版，实际应调用比赛模拟引擎）
    async fn simulate_all_phase_matches(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        phase: SeasonPhase,
    ) -> Result<u32, String> {
        // 获取当前阶段所有未完成的比赛
        let tournament_type = phase_to_tournament_type(phase);

        if tournament_type.is_none() {
            return Ok(0);
        }

        // 将所有未完成比赛标记为已完成（简化处理，实际应该调用比赛模拟引擎）
        let result = sqlx::query(
            r#"
            UPDATE matches
            SET status = 'COMPLETED',
                home_score = CASE WHEN home_score IS NULL THEN 2 ELSE home_score END,
                away_score = CASE WHEN away_score IS NULL THEN 1 ELSE away_score END
            WHERE tournament_id IN (
                SELECT id FROM tournaments
                WHERE save_id = ? AND tournament_type = ?
            )
            AND status != 'COMPLETED'
            "#
        )
        .bind(save_id)
        .bind(format!("{:?}", tournament_type.unwrap()))
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() as u32)
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

/// 获取阶段显示名称
fn get_phase_display_name(phase: &SeasonPhase) -> &'static str {
    match phase {
        SeasonPhase::SpringRegular => "春季常规赛",
        SeasonPhase::SpringPlayoffs => "春季季后赛",
        SeasonPhase::Msi => "MSI季中赛",
        SeasonPhase::MadridMasters => "马德里大师赛",
        SeasonPhase::SummerRegular => "夏季常规赛",
        SeasonPhase::SummerPlayoffs => "夏季季后赛",
        SeasonPhase::ClaudeIntercontinental => "Claude洲际赛",
        SeasonPhase::WorldChampionship => "世界赛",
        SeasonPhase::ShanghaiMasters => "上海大师赛",
        SeasonPhase::IcpIntercontinental => "ICP洲际对抗赛",
        SeasonPhase::SuperIntercontinental => "Super洲际邀请赛",
        SeasonPhase::TransferWindow => "转会期",
        SeasonPhase::Draft => "选秀大会",
        SeasonPhase::SeasonEnd => "赛季结算",
    }
}

/// 阶段到赛事类型映射
fn phase_to_tournament_type(phase: SeasonPhase) -> Option<TournamentType> {
    match phase {
        SeasonPhase::SpringRegular => Some(TournamentType::SpringRegular),
        SeasonPhase::SpringPlayoffs => Some(TournamentType::SpringPlayoffs),
        SeasonPhase::Msi => Some(TournamentType::Msi),
        SeasonPhase::MadridMasters => Some(TournamentType::MadridMasters),
        SeasonPhase::SummerRegular => Some(TournamentType::SummerRegular),
        SeasonPhase::SummerPlayoffs => Some(TournamentType::SummerPlayoffs),
        SeasonPhase::ClaudeIntercontinental => Some(TournamentType::ClaudeIntercontinental),
        SeasonPhase::WorldChampionship => Some(TournamentType::WorldChampionship),
        SeasonPhase::ShanghaiMasters => Some(TournamentType::ShanghaiMasters),
        SeasonPhase::IcpIntercontinental => Some(TournamentType::IcpIntercontinental),
        SeasonPhase::SuperIntercontinental => Some(TournamentType::SuperIntercontinental),
        _ => None,
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

/// 判断是否为非赛事阶段（转会期、选秀、赛季结束）
fn is_non_tournament_phase(phase: SeasonPhase) -> bool {
    matches!(
        phase,
        SeasonPhase::TransferWindow | SeasonPhase::Draft | SeasonPhase::SeasonEnd
    )
}

/// 判断当前阶段是否在目标阶段之前
fn is_before_phase(current: SeasonPhase, target: SeasonPhase) -> bool {
    let phase_order = [
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
        SeasonPhase::TransferWindow,
        SeasonPhase::Draft,
        SeasonPhase::SeasonEnd,
    ];

    let current_idx = phase_order.iter().position(|&p| p == current);
    let target_idx = phase_order.iter().position(|&p| p == target);

    match (current_idx, target_idx) {
        (Some(c), Some(t)) => c < t,
        _ => false,
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
            phase_to_tournament_type(SeasonPhase::SpringRegular),
            Some(TournamentType::SpringRegular)
        );
        assert_eq!(
            phase_to_tournament_type(SeasonPhase::Msi),
            Some(TournamentType::Msi)
        );
        assert_eq!(phase_to_tournament_type(SeasonPhase::TransferWindow), None);
        assert_eq!(phase_to_tournament_type(SeasonPhase::Draft), None);
    }

    #[test]
    fn test_get_phase_display_name() {
        assert_eq!(
            get_phase_display_name(&SeasonPhase::SpringRegular),
            "春季常规赛"
        );
        assert_eq!(get_phase_display_name(&SeasonPhase::Msi), "MSI季中赛");
        assert_eq!(
            get_phase_display_name(&SeasonPhase::WorldChampionship),
            "世界赛"
        );
    }
}
