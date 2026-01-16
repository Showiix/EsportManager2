//! LLM 深度集成转会市场引擎
//!
//! 实现 Agent 化的转会市场系统，每个选手和球队都是独立的 AI Agent：
//! - 多阶段市场流程管理
//! - LLM 驱动的选手意愿生成
//! - LLM 驱动的球队策略生成
//! - 多轮谈判和报价博弈
//! - 连锁反应处理
//! - 完整的分析步骤展示

use crate::models::{
    Player, PlayerStatus, Team, AITransferStrategy, TeamGMProfile,
    PlayerTransferStrategy, TransferMarketState, TeamMarketState, MarketPhase,
    Negotiation, NegotiationStatus, Offer, OfferStatus, OfferResponse,
    MarketEvent, MarketEventType, GenerationProgress, RoundExecutionResult,
};
use crate::services::create_llm_service;
use serde::{Serialize, Deserialize};

/// 签约信息（用于反馈给战队）
#[derive(Debug, Clone)]
pub struct SigningInfo {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub salary: u64,
    pub years: u8,
}

/// 选手决策结果
#[derive(Debug, Clone)]
pub struct PlayerDecision {
    /// 接受的报价ID
    pub accepted_offer_id: u64,
    /// 思考过程
    pub reasoning: String,
}
use crate::engines::transfer::FreeAgentInfo;
use crate::services::llm_service::{
    LLMTransferService, TeamInfo, PlayerHonorInfo, PlayerPerformanceInfo,
    TeamHonorInfo, RosterPlayerHonorSummary, PlayerPerformanceSummary,
};
use crate::services::ai_transfer_service::AITransferService;
use std::collections::HashMap;

/// LLM 转会市场引擎
pub struct TransferMarketEngine {
    /// 市场状态
    pub state: TransferMarketState,
    /// 所有选手策略（player_id -> strategy）
    pub player_strategies: HashMap<u64, PlayerTransferStrategy>,
    /// 所有球队策略（team_id -> strategy）
    pub team_strategies: HashMap<u64, AITransferStrategy>,
    /// 所有谈判记录（negotiation_id -> negotiation）
    pub negotiations: HashMap<u64, Negotiation>,
    /// 所有事件
    pub events: Vec<MarketEvent>,
    /// LLM 服务（可选）
    llm_service: Option<LLMTransferService>,
    /// 下一个谈判 ID
    pub next_negotiation_id: u64,
    /// 下一个报价 ID
    pub next_offer_id: u64,
    /// 下一个回应 ID
    pub next_response_id: u64,
    /// 下一个事件 ID
    pub next_event_id: u64,
}

impl TransferMarketEngine {
    /// 创建新的市场引擎
    pub fn new(save_id: String, season_id: u64, llm_service: Option<LLMTransferService>) -> Self {
        Self {
            state: TransferMarketState::new(save_id, season_id),
            player_strategies: HashMap::new(),
            team_strategies: HashMap::new(),
            negotiations: HashMap::new(),
            events: Vec::new(),
            llm_service,
            next_negotiation_id: 1,
            next_offer_id: 1,
            next_response_id: 1,
            next_event_id: 1,
        }
    }

    /// 检查 LLM 是否可用
    pub fn is_llm_available(&self) -> bool {
        self.llm_service.as_ref().map(|s| s.is_configured()).unwrap_or(false)
    }

    // ==================== 阶段 0: 初始化 ====================

    /// 初始化市场状态
    pub fn initialize(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
        free_agents: &[FreeAgentInfo],
    ) {
        log::info!("初始化转会市场...");

        // 初始化球队状态
        for team in teams {
            let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
            let active_count = roster.iter()
                .filter(|p| p.status == PlayerStatus::Active)
                .count() as u8;

            let team_state = TeamMarketState::new(
                team.id,
                team.name.clone(),
                team.balance,
                active_count,
            );
            self.state.init_team_state(team_state);
        }

        // 计算总选手数
        self.state.total_players = players_by_team.values()
            .flat_map(|ps| ps.iter())
            .filter(|p| p.status == PlayerStatus::Active)
            .count() as u32;

        // 添加自由球员
        for fa in free_agents {
            self.state.add_free_agent(fa.player.id);
        }

        log::info!(
            "市场初始化完成: {}支球队, {}名选手, {}名自由球员",
            self.state.total_teams,
            self.state.total_players,
            self.state.free_agent_ids.len()
        );

        // 生成初始化事件
        self.record_event(
            MarketEventType::ContractExpired, // 用于标记市场开启
            "转会市场开启".to_string(),
            format!(
                "{}赛季转会窗口正式开启，共有{}名自由球员进入市场",
                self.state.season_id,
                self.state.free_agent_ids.len()
            ),
        );
    }

    // ==================== 阶段 1: 选手意愿生成 ====================

    /// 为所有选手生成转会意愿（异步版本）
    pub async fn generate_all_player_intentions(
        &mut self,
        players: &[Player],
        teams: &HashMap<u64, Team>,
        available_teams: &[TeamInfo],
        honors: &HashMap<u64, PlayerHonorInfo>,
        performances: &HashMap<u64, PlayerPerformanceInfo>,
    ) -> Result<(), String> {
        let total = players.len() as u32;
        let mut progress = GenerationProgress::new("player_intentions", total);

        log::info!("开始生成 {} 名选手的转会意愿...", total);

        for player in players {
            if player.status != PlayerStatus::Active {
                progress.advance(None);
                continue;
            }

            progress.advance(Some(player.game_id.clone()));

            // 获取选手所在球队
            let team = match player.team_id.and_then(|tid| teams.get(&tid)) {
                Some(t) => t,
                None => {
                    // 自由球员不需要生成意愿
                    continue;
                }
            };

            // 生成策略
            let strategy = if self.is_llm_available() {
                // 使用 LLM 生成
                match self.generate_player_strategy_llm(
                    player,
                    team,
                    available_teams,
                    honors.get(&player.id),
                    performances.get(&player.id),
                ).await {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("LLM 生成选手 {} 策略失败: {}, 使用 Mock", player.game_id, e);
                        progress.add_error(format!("{}: {}", player.game_id, e));
                        self.generate_player_strategy_mock(player, team, available_teams)
                    }
                }
            } else {
                // 使用 Mock 生成
                self.generate_player_strategy_mock(player, team, available_teams)
            };

            // 保存策略
            self.player_strategies.insert(player.id, strategy.clone());
            self.state.intentions_generated += 1;

            // 如果选手想离队，记录事件
            if strategy.wants_to_leave {
                self.record_event_with_player(
                    MarketEventType::TransferRequested,
                    player.id,
                    player.game_id.clone(),
                    team.id,
                    team.name.clone(),
                    format!("{} 申请转会", player.game_id),
                    strategy.leave_reasoning.clone(),
                );
            }
        }

        log::info!(
            "选手意愿生成完成: {}/{}, 错误: {}",
            self.state.intentions_generated,
            total,
            progress.errors.len()
        );

        // 推进阶段
        self.state.advance_phase();

        Ok(())
    }

    /// 为单个选手生成转会意愿（用于实时进度反馈）
    pub async fn generate_single_player_intention(
        &mut self,
        player: &Player,
        teams: &HashMap<u64, Team>,
        available_teams: &[TeamInfo],
        honors: Option<&PlayerHonorInfo>,
        performance: Option<&PlayerPerformanceInfo>,
    ) -> Result<Option<PlayerTransferStrategy>, String> {
        if player.status != PlayerStatus::Active {
            return Ok(None);
        }

        // 获取选手所在球队
        let team = match player.team_id.and_then(|tid| teams.get(&tid)) {
            Some(t) => t,
            None => {
                // 自由球员不需要生成意愿
                return Ok(None);
            }
        };

        // 生成策略
        let strategy = if self.is_llm_available() {
            match self.generate_player_strategy_llm(
                player,
                team,
                available_teams,
                honors,
                performance,
            ).await {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("LLM 生成选手 {} 策略失败: {}, 使用 Mock", player.game_id, e);
                    self.generate_player_strategy_mock(player, team, available_teams)
                }
            }
        } else {
            self.generate_player_strategy_mock(player, team, available_teams)
        };

        // 保存策略
        self.player_strategies.insert(player.id, strategy.clone());
        self.state.intentions_generated += 1;

        // 如果选手想离队，记录事件
        if strategy.wants_to_leave {
            self.record_event_with_player(
                MarketEventType::TransferRequested,
                player.id,
                player.game_id.clone(),
                team.id,
                team.name.clone(),
                format!("{} 申请转会", player.game_id),
                strategy.leave_reasoning.clone(),
            );
        }

        Ok(Some(strategy))
    }

    /// 完成选手意愿生成阶段 - 直接跳到策略生成阶段
    pub fn finish_intention_generation(&mut self) {
        self.state.current_phase = MarketPhase::StrategyGeneration;
        self.state.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 使用 LLM 生成选手策略
    async fn generate_player_strategy_llm(
        &self,
        player: &Player,
        team: &Team,
        available_teams: &[TeamInfo],
        honors: Option<&PlayerHonorInfo>,
        performance: Option<&PlayerPerformanceInfo>,
    ) -> Result<PlayerTransferStrategy, String> {
        let llm = self.llm_service.as_ref()
            .ok_or("LLM 服务未配置")?;

        // 获取球队阵容（简化版本）
        let roster = vec![]; // 实际使用时需要传入

        llm.generate_player_strategy(
            player,
            team,
            &roster,
            available_teams,
            &self.state.save_id,
            self.state.season_id,
            honors,
            performance,
            None, // current_team_rank
        ).await
    }

    /// 使用 Mock 生成选手策略
    fn generate_player_strategy_mock(
        &self,
        player: &Player,
        team: &Team,
        available_teams: &[TeamInfo],
    ) -> PlayerTransferStrategy {
        AITransferService::generate_mock_player_strategy(
            player,
            team,
            available_teams,
            &self.state.save_id,
            self.state.season_id,
        )
    }

    // ==================== 阶段 2: 球队策略生成 ====================

    /// 为所有球队生成转会策略（异步版本）
    pub async fn generate_all_team_strategies(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
        free_agents: &[FreeAgentInfo],
        gm_profiles: &HashMap<u64, TeamGMProfile>,
        team_honors: &HashMap<u64, TeamHonorInfo>,
        roster_honors: &HashMap<u64, Vec<RosterPlayerHonorSummary>>,
        roster_performance: &HashMap<u64, Vec<PlayerPerformanceSummary>>,
    ) -> Result<(), String> {
        let total = teams.len() as u32;
        let mut progress = GenerationProgress::new("team_strategies", total);

        log::info!("开始生成 {} 支球队的转会策略...", total);

        for team in teams {
            progress.advance(Some(team.name.clone()));

            let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
            let profile = gm_profiles.get(&team.id).cloned().unwrap_or_else(|| {
                TeamGMProfile::new(team.id, self.state.save_id.clone())
            });

            // 生成策略
            let strategy = if self.is_llm_available() {
                match self.generate_team_strategy_llm(
                    team,
                    &roster,
                    &profile,
                    free_agents,
                    players_by_team,
                    team_honors.get(&team.id),
                    roster_honors.get(&team.id).map(|v| v.as_slice()),
                    roster_performance.get(&team.id).map(|v| v.as_slice()),
                ).await {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("LLM 生成球队 {} 策略失败: {}, 使用 Mock", team.name, e);
                        progress.add_error(format!("{}: {}", team.name, e));
                        AITransferService::generate_mock_strategy(
                            team, &roster, &profile, free_agents, players_by_team,
                            &self.state.save_id, self.state.season_id,
                        )
                    }
                }
            } else {
                AITransferService::generate_mock_strategy(
                    team, &roster, &profile, free_agents, players_by_team,
                    &self.state.save_id, self.state.season_id,
                )
            };

            // 保存策略
            self.team_strategies.insert(team.id, strategy);
            self.state.strategies_generated += 1;

            // 更新球队市场状态
            if let Some(team_state) = self.state.get_team_state_mut(team.id) {
                team_state.strategy_generated = true;
            }
        }

        log::info!(
            "球队策略生成完成: {}/{}",
            self.state.strategies_generated,
            total
        );

        // 推进阶段
        self.state.advance_phase();

        Ok(())
    }

    /// 为单个球队生成转会策略（用于实时进度反馈）
    pub async fn generate_single_team_strategy(
        &mut self,
        team: &Team,
        roster: &[Player],
        profile: &TeamGMProfile,
        free_agents: &[FreeAgentInfo],
        all_players_by_team: &HashMap<u64, Vec<Player>>,
        team_honors: Option<&TeamHonorInfo>,
        roster_honors: Option<&[RosterPlayerHonorSummary]>,
        roster_performance: Option<&[PlayerPerformanceSummary]>,
    ) -> Result<AITransferStrategy, String> {
        // 生成策略
        let strategy = if self.is_llm_available() {
            match self.generate_team_strategy_llm(
                team,
                roster,
                profile,
                free_agents,
                all_players_by_team,
                team_honors,
                roster_honors,
                roster_performance,
            ).await {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("LLM 生成球队 {} 策略失败: {}, 使用 Mock", team.name, e);
                    AITransferService::generate_mock_strategy(
                        team, roster, profile, free_agents, all_players_by_team,
                        &self.state.save_id, self.state.season_id,
                    )
                }
            }
        } else {
            AITransferService::generate_mock_strategy(
                team, roster, profile, free_agents, all_players_by_team,
                &self.state.save_id, self.state.season_id,
            )
        };

        // 保存策略
        self.team_strategies.insert(team.id, strategy.clone());
        self.state.strategies_generated += 1;

        // 更新球队市场状态
        if let Some(team_state) = self.state.get_team_state_mut(team.id) {
            team_state.strategy_generated = true;
        }

        Ok(strategy)
    }

    /// 完成球队策略生成阶段 - 直接跳到续约处理阶段
    pub fn finish_strategy_generation(&mut self) {
        log::info!(
            "球队策略生成完成: {}/{}",
            self.state.strategies_generated,
            self.state.total_teams
        );
        self.state.current_phase = MarketPhase::RenewalProcessing;
        self.state.updated_at = chrono::Utc::now().to_rfc3339();
    }

    // ==================== 阶段 3: 续约处理 ====================

    /// 获取需要续约的选手列表（不想离队的选手）
    pub fn get_renewal_candidates(&self) -> Vec<u64> {
        self.player_strategies.iter()
            .filter(|(_, s)| !s.wants_to_leave)
            .map(|(id, _)| *id)
            .collect()
    }

    /// 处理单个选手的续约
    pub async fn process_single_renewal(
        &mut self,
        player: &Player,
        team: &Team,
        player_honors: Option<&crate::services::llm_service::PlayerHonorInfo>,
        player_performance: Option<&crate::services::llm_service::PlayerPerformanceInfo>,
    ) -> Result<crate::models::RenewalDecision, String> {
        // 获取选手策略
        let player_strategy = self.player_strategies.get(&player.id)
            .ok_or("选手策略不存在")?
            .clone();

        // 获取球队策略
        let team_strategy = self.team_strategies.get(&team.id);

        // 检查 LLM 服务是否可用
        if !self.is_llm_available() {
            return Err("LLM 服务未配置，无法处理续约".to_string());
        }

        // 调用 LLM 评估续约
        let llm = self.llm_service.as_ref()
            .ok_or("LLM 服务未配置")?;

        let decision = llm.evaluate_renewal(
            player,
            team,
            &player_strategy,
            team_strategy,
            player_honors,
            player_performance,
        ).await?;

        // 根据续约结果记录事件
        if decision.renewal_successful {
            self.record_event_with_player(
                MarketEventType::RenewalSuccessful,
                player.id,
                player.game_id.clone(),
                team.id,
                team.name.clone(),
                format!("{} 与 {} 续约成功", player.game_id, team.name),
                format!(
                    "新合同: {}万/年，{}年，{}",
                    decision.final_salary.unwrap_or(0),
                    decision.final_years.unwrap_or(0),
                    decision.summary
                ),
            );
        } else if !decision.team_wants_renewal {
            // 球队不想续约 -> 选手成为自由球员
            self.record_event_with_player(
                MarketEventType::RenewalFailed,
                player.id,
                player.game_id.clone(),
                team.id,
                team.name.clone(),
                format!("{} 遭 {} 放弃续约", player.game_id, team.name),
                decision.team_rejection_reason.clone().unwrap_or_else(|| decision.summary.clone()),
            );
            // 添加到自由球员列表
            self.state.add_free_agent(player.id);
        } else if !decision.player_accepts {
            // 选手拒绝续约 -> 选手成为自由球员
            self.record_event_with_player(
                MarketEventType::RenewalFailed,
                player.id,
                player.game_id.clone(),
                team.id,
                team.name.clone(),
                format!("{} 拒绝 {} 的续约报价", player.game_id, team.name),
                decision.player_rejection_reason.clone().unwrap_or_else(|| decision.summary.clone()),
            );
            // 添加到自由球员列表
            self.state.add_free_agent(player.id);
        }

        Ok(decision)
    }

    /// 完成续约处理阶段 - 推进到离队播报
    pub fn finish_renewal_processing(&mut self) {
        log::info!("续约处理阶段完成");

        // 将所有想离队的选手加入自由球员列表
        let departure_candidates: Vec<u64> = self.player_strategies.iter()
            .filter(|(_, s)| s.wants_to_leave)
            .map(|(id, _)| *id)
            .collect();

        for player_id in &departure_candidates {
            self.state.add_free_agent(*player_id);
        }

        log::info!("已将 {} 名想离队的选手加入自由球员市场", departure_candidates.len());

        // 续约处理完成后，直接进入自由市场阶段
        self.state.current_phase = MarketPhase::FreeMarket;
        self.state.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 使用 LLM 生成球队策略
    async fn generate_team_strategy_llm(
        &self,
        team: &Team,
        roster: &[Player],
        profile: &TeamGMProfile,
        free_agents: &[FreeAgentInfo],
        all_players_by_team: &HashMap<u64, Vec<Player>>,
        team_honors: Option<&TeamHonorInfo>,
        roster_honors: Option<&[RosterPlayerHonorSummary]>,
        roster_performance: Option<&[PlayerPerformanceSummary]>,
    ) -> Result<AITransferStrategy, String> {
        let llm = self.llm_service.as_ref()
            .ok_or("LLM 服务未配置")?;

        llm.generate_strategy(
            team,
            roster,
            profile,
            free_agents,
            all_players_by_team,
            &self.state.save_id,
            self.state.season_id,
            team_honors,
            roster_honors,
            roster_performance,
        ).await
    }

    // ==================== 阶段 5-6: 报价和谈判（新设计） ====================
    //
    // 新的轮次设计：
    // 1. 阶段1（战队报价）：战队 LLM 向选手发起报价
    // 2. 阶段2（选手决策）：选手 Agent LLM 处理所有收到的报价，决定加入哪支队伍
    // 3. 反馈循环：签约信息反馈给战队 LLM，继续发起新报价
    // 4. 等待机制：没有收到报价的选手继续等待下一轮

    /// 执行一轮报价（分为两个阶段）
    pub async fn execute_bidding_round(
        &mut self,
        teams: &HashMap<u64, Team>,
        players: &HashMap<u64, Player>,
    ) -> RoundExecutionResult {
        let phase = self.state.current_phase;
        let round = self.state.current_round;

        log::info!("========== 执行第 {} 轮报价 ==========", round);

        // 阶段1：战队发起报价
        log::info!("--- 阶段1：战队 LLM 发起报价 ---");
        let new_negotiations = self.execute_team_offering_phase(teams, players, round).await;

        // 阶段2：选手处理报价
        log::info!("--- 阶段2：选手 Agent LLM 处理报价 ---");
        let (completed_signings, signed_players) = self.execute_player_decision_phase(players, round).await;

        // 阶段3：反馈给战队（更新战队状态，让他们知道哪些选手已签约）
        log::info!("--- 阶段3：反馈签约信息给战队 ---");
        self.provide_feedback_to_teams(&signed_players, teams);

        // 统计等待中的选手
        let waiting_players = self.get_waiting_players_count();
        log::info!(
            "本轮统计: 新增{}个谈判, 完成{}笔签约, {}名选手等待中",
            new_negotiations, completed_signings, waiting_players
        );

        // 检查市场稳定性
        if new_negotiations == 0 && completed_signings == 0 {
            self.state.record_stable_round();
        }

        // 推进轮次
        self.state.advance_round();

        // 检查是否需要切换阶段
        let phase_changed = self.state.should_enter_last_chance();
        let new_phase = if phase_changed {
            self.state.advance_phase();
            Some(self.state.current_phase)
        } else {
            None
        };

        RoundExecutionResult {
            phase,
            round,
            phase_changed,
            new_phase,
            events: Vec::new(),
            new_negotiations: new_negotiations as usize,
            completed_signings: completed_signings as usize,
            summary: format!(
                "第{}轮: 新增{}个谈判, 完成{}笔签约, {}名选手等待报价",
                round, new_negotiations, completed_signings, waiting_players
            ),
        }
    }

    /// 阶段1：战队 LLM 发起报价
    ///
    /// 每支战队根据自己的策略，选择1个目标选手发出报价（每队每轮限1人）
    async fn execute_team_offering_phase(
        &mut self,
        teams: &HashMap<u64, Team>,
        players: &HashMap<u64, Player>,
        round: u8,
    ) -> u32 {
        let mut new_negotiations = 0;

        // 获取已签约的选手（避免向已签约选手发送报价）
        let signed_player_ids: std::collections::HashSet<u64> = self.negotiations.values()
            .filter(|n| n.status == NegotiationStatus::Accepted)
            .map(|n| n.player_id)
            .collect();

        for (team_id, strategy) in &self.team_strategies.clone() {
            let team = match teams.get(team_id) {
                Some(t) => t,
                None => continue,
            };

            log::info!("战队 {} 评估本轮报价目标...", team.name);

            // 找到最优先的、可报价的目标选手（每队每轮只能报价1人）
            let mut best_target: Option<&Player> = None;

            for target in &strategy.targets {
                let player = match players.get(&target.player_id) {
                    Some(p) => p,
                    None => continue,
                };

                // 跳过已签约的选手
                if signed_player_ids.contains(&target.player_id) {
                    continue;
                }

                // 检查选手是否仍是自由球员
                if !self.state.free_agent_ids.contains(&target.player_id) {
                    continue;
                }

                // 检查是否已经向该选手发过报价（避免重复报价）
                let already_offered = self.negotiations.values()
                    .any(|n| n.player_id == target.player_id &&
                         n.offers.iter().any(|o| o.from_team_id == *team_id));

                if already_offered {
                    continue;
                }

                // 找到第一个可报价的目标（targets 已按优先级排序）
                best_target = Some(player);
                break;
            }

            // 如果有可报价目标，发起报价
            if let Some(player) = best_target {
                let neg_id = self.find_or_create_negotiation(player);

                // 生成报价（调用 LLM）
                let offer_created = self.generate_team_offer_llm(
                    neg_id,
                    team,
                    player,
                    strategy,
                    round,
                ).await;

                if offer_created {
                    // 记录新谈判（如果是首次）
                    if self.negotiations.get(&neg_id).map(|n| n.offers.len() == 1).unwrap_or(false) {
                        new_negotiations += 1;
                    }

                    // 记录报价事件
                    self.record_event_with_player(
                        MarketEventType::OfferMade,
                        player.id,
                        player.game_id.clone(),
                        team.id,
                        team.name.clone(),
                        format!("{} 向 {} 发出报价", team.name, player.game_id),
                        format!("第{}轮报价", round),
                    );

                    log::info!("✓ {} 本轮选择向 {} 发出报价", team.name, player.game_id);
                }
            } else {
                log::info!("○ {} 本轮没有合适的报价目标", team.name);
            }
        }

        new_negotiations
    }

    /// 阶段2：选手 Agent LLM 处理报价（可完全并发）
    ///
    /// 每个收到报价的选手，由 LLM 评估所有报价，必须选择一个接受
    /// 因为每个战队每轮只发1个报价，所以不存在"两个选手同时接受同一战队"的问题
    async fn execute_player_decision_phase(
        &mut self,
        players: &HashMap<u64, Player>,
        round: u8,
    ) -> (u32, Vec<SigningInfo>) {
        let mut completed_signings = 0;
        let mut signed_players: Vec<SigningInfo> = Vec::new();

        // 收集所有需要处理的谈判
        let negotiation_ids: Vec<u64> = self.negotiations.iter()
            .filter(|(_, n)| n.status == NegotiationStatus::Open)
            .map(|(id, _)| *id)
            .collect();

        // TODO: 这里可以改成并发调用 LLM，因为每个选手的决策互不影响
        for neg_id in negotiation_ids {
            let negotiation = match self.negotiations.get(&neg_id) {
                Some(n) => n.clone(),
                None => continue,
            };

            let player = match players.get(&negotiation.player_id) {
                Some(p) => p,
                None => continue,
            };

            // 检查选手是否仍是自由球员
            if !self.state.free_agent_ids.contains(&player.id) {
                continue;
            }

            // 获取所有待处理的报价
            let pending_offers: Vec<_> = negotiation.offers.iter()
                .filter(|o| o.status == OfferStatus::Pending)
                .cloned()
                .collect();

            // 没有收到报价，继续等待下一轮
            if pending_offers.is_empty() {
                log::info!("○ {} 本轮没有收到报价，继续等待", player.game_id);
                continue;
            }

            log::info!("选手 {} 收到 {} 份报价，必须做出选择...", player.game_id, pending_offers.len());

            // 调用 LLM 评估所有报价（必须选择一个）
            let decision = self.evaluate_player_offers_llm(
                player,
                &pending_offers,
                round,
            ).await;

            // 构建报价列表描述
            let offers_desc = pending_offers.iter()
                .map(|o| format!("{}({}万/年)", o.from_team_name, o.salary_offer))
                .collect::<Vec<_>>()
                .join("、");

            // 找到被接受的报价
            if let Some(accepted_offer) = pending_offers.iter().find(|o| o.id == decision.accepted_offer_id) {
                // 记录选手思考过程
                self.record_event_with_player(
                    MarketEventType::PlayerThinking,
                    player.id,
                    player.game_id.clone(),
                    accepted_offer.from_team_id,
                    accepted_offer.from_team_name.clone(),
                    format!("{} 做出选择", player.game_id),
                    format!(
                        "收到报价: {}\n\n💭 思考过程:\n{}\n\n✅ 决定: 加盟 {}",
                        offers_desc, decision.reasoning, accepted_offer.from_team_name
                    ),
                );

                // 更新谈判状态
                self.complete_player_signing(
                    neg_id,
                    player,
                    accepted_offer,
                    &pending_offers,
                    &decision.reasoning,
                );

                completed_signings += 1;
                signed_players.push(SigningInfo {
                    player_id: player.id,
                    player_name: player.game_id.clone(),
                    team_id: accepted_offer.from_team_id,
                    team_name: accepted_offer.from_team_name.clone(),
                    salary: accepted_offer.salary_offer,
                    years: accepted_offer.contract_years,
                });

                log::info!(
                    "✓ {} 选择加盟 {}，{}万/年，{}年合同",
                    player.game_id, accepted_offer.from_team_name,
                    accepted_offer.salary_offer, accepted_offer.contract_years
                );
            }
        }

        (completed_signings, signed_players)
    }

    /// 阶段3：反馈签约信息给战队
    ///
    /// 通知战队哪些选手已经签约，让他们在下一轮调整策略
    fn provide_feedback_to_teams(&mut self, signed_players: &[SigningInfo], teams: &HashMap<u64, Team>) {
        for signing in signed_players {
            log::info!(
                "反馈: {} 已加盟 {}",
                signing.player_name, signing.team_name
            );

            // 更新所有战队的目标列表（从目标中移除已签约的选手）
            for (team_id, strategy) in &mut self.team_strategies {
                if *team_id == signing.team_id {
                    // 签约成功的战队：更新预算
                    if let Some(team_state) = self.state.get_team_state_mut(*team_id) {
                        let cost = signing.salary * signing.years as u64 * 10000;
                        team_state.remaining_budget = team_state.remaining_budget.saturating_sub(cost);
                        team_state.roster_count += 1;
                    }
                } else {
                    // 其他战队：移除该选手作为目标（将在下一轮重新评估）
                    // 这里不实际移除，而是依靠 signed_player_ids 检查来跳过
                }
            }
        }
    }

    /// 查找或创建谈判
    pub fn find_or_create_negotiation(&mut self, player: &Player) -> u64 {
        // 查找现有的活跃谈判
        if let Some(neg) = self.negotiations.values().find(|n| {
            n.player_id == player.id && n.status == NegotiationStatus::Open
        }) {
            return neg.id;
        }

        // 创建新谈判
        self.create_negotiation(player)
    }

    /// 使用 LLM 生成战队报价
    async fn generate_team_offer_llm(
        &mut self,
        negotiation_id: u64,
        team: &Team,
        player: &Player,
        team_strategy: &AITransferStrategy,
        round: u8,
    ) -> bool {
        // 获取选手策略
        let player_strategy = match self.player_strategies.get(&player.id) {
            Some(s) => s.clone(),
            None => return false,
        };

        // TODO: 调用 LLM 生成报价决策
        // 目前使用简化逻辑：基于选手期望和战队策略生成报价

        // 构建战队思考过程
        let mut thinking_steps = Vec::new();

        // 获取战队对该选手的最高出价预算
        let max_offer = team_strategy.get_max_offer(player.id).unwrap_or(0);
        let target_priority = team_strategy.get_target_priority(player.id);

        thinking_steps.push(format!(
            "🎯 目标选手: {} (能力值: {})\n📊 目标优先级: {}",
            player.game_id,
            player.ability,
            target_priority.map(|p| format!("第{}位", p)).unwrap_or("未列入目标".to_string())
        ));

        if max_offer == 0 {
            // 不在战队策略的目标列表中
            return false;
        }

        thinking_steps.push(format!(
            "💰 预算分析:\n  - 对该选手最高出价: {}万/年\n  - 选手期望薪资: {}万/年\n  - 选手最低接受: {}万/年",
            max_offer,
            player_strategy.expected_salary,
            player_strategy.expected_min_salary
        ));

        // 计算报价金额
        let base_salary = player_strategy.expected_salary;
        let offer_salary = if base_salary <= max_offer {
            // 如果期望在预算内，直接满足期望
            base_salary
        } else {
            // 否则出最高预算
            max_offer
        };

        // 检查是否低于选手最低接受线
        let min_acceptable = player_strategy.expected_min_salary;
        if offer_salary < min_acceptable {
            thinking_steps.push(format!(
                "\n🤔 结论: 我们的预算 {}万/年 低于选手最低接受 {}万/年，放弃报价。",
                offer_salary, min_acceptable
            ));

            // 记录战队放弃报价的思考过程
            self.record_event_with_player(
                MarketEventType::TeamThinking,
                player.id,
                player.game_id.clone(),
                team.id,
                team.name.clone(),
                format!("{} 评估 {}", team.name, player.game_id),
                thinking_steps.join("\n"),
            );

            log::debug!(
                "{} 预算 {}万 低于 {} 最低接受 {}万，放弃报价",
                team.name, offer_salary, player.game_id, min_acceptable
            );
            return false;
        }

        thinking_steps.push(format!(
            "\n📝 报价决策:\n  - 报价金额: {}万/年\n  - 合同年限: {}年\n  - 首发保证: {}",
            offer_salary,
            player_strategy.expected_years.max(2),
            if player_strategy.requires_starter { "是" } else { "否" }
        ));

        thinking_steps.push(format!(
            "\n🤔 结论: {} 符合我们的补强需求，决定发出报价！",
            player.game_id
        ));

        // 记录战队思考过程
        self.record_event_with_player(
            MarketEventType::TeamThinking,
            player.id,
            player.game_id.clone(),
            team.id,
            team.name.clone(),
            format!("{} 评估 {}", team.name, player.game_id),
            thinking_steps.join("\n"),
        );

        let offer_id = self.next_offer_id;
        self.next_offer_id += 1;

        let mut offer = Offer::new(
            negotiation_id,
            team.id,
            team.name.clone(),
            player.id,
            round,
        );
        offer.id = offer_id;
        offer.salary_offer = offer_salary;
        offer.contract_years = player_strategy.expected_years.max(2);
        offer.guarantee_starter = player_strategy.requires_starter;
        offer.offer_reasoning = thinking_steps.join("\n");

        // 添加到谈判
        if let Some(neg) = self.negotiations.get_mut(&negotiation_id) {
            neg.add_offer(offer);
            return true;
        }

        false
    }

    /// 使用 LLM 评估选手收到的所有报价（必须选择一个接受）
    async fn evaluate_player_offers_llm(
        &self,
        player: &Player,
        offers: &[Offer],
        _round: u8,
    ) -> PlayerDecision {
        // 获取选手策略
        let strategy = self.player_strategies.get(&player.id);

        // 构建详细的思考过程
        let mut thinking_steps = Vec::new();

        if let Some(strategy) = strategy {
            thinking_steps.push(format!(
                "📊 我的期望条件：\n  - 期望薪资: {}万/年\n  - 最低接受: {}万/年\n  - 期望年限: {}年\n  - 需要首发: {}",
                strategy.expected_salary,
                strategy.expected_min_salary,
                strategy.expected_years,
                if strategy.requires_starter { "是" } else { "否" }
            ));

            // 分析每个报价
            thinking_steps.push("\n📋 报价分析:".to_string());
            for (i, offer) in offers.iter().enumerate() {
                let min_salary = strategy.get_min_salary_for_team(offer.from_team_id);
                let salary_ok = offer.salary_offer >= min_salary;
                let starter_ok = !strategy.requires_starter || offer.guarantee_starter;
                let priority = strategy.get_team_priority(offer.from_team_id);

                thinking_steps.push(format!(
                    "  {}. {} 的报价:\n     薪资: {}万/年 {} (最低要求{}万)\n     合同: {}年\n     首发: {} {}\n     球队偏好: {}",
                    i + 1,
                    offer.from_team_name,
                    offer.salary_offer,
                    if salary_ok { "✓" } else { "✗" },
                    min_salary,
                    offer.contract_years,
                    if offer.guarantee_starter { "保证" } else { "不保证" },
                    if starter_ok { "✓" } else { "✗" },
                    priority.map(|p| format!("第{}位", p)).unwrap_or("无偏好".to_string())
                ));
            }

            // 找出满足最低要求的报价
            let acceptable_offers: Vec<_> = offers.iter()
                .filter(|o| {
                    let min_salary = strategy.get_min_salary_for_team(o.from_team_id);
                    o.salary_offer >= min_salary &&
                    (!strategy.requires_starter || o.guarantee_starter)
                })
                .collect();

            if !acceptable_offers.is_empty() {
                // 从可接受的报价中选择最优的
                // 优先选择：1) 在偏好列表中的球队 2) 薪资最高的
                let best_offer = acceptable_offers.iter()
                    .max_by(|a, b| {
                        let a_priority = strategy.get_team_priority(a.from_team_id).unwrap_or(255);
                        let b_priority = strategy.get_team_priority(b.from_team_id).unwrap_or(255);

                        match a_priority.cmp(&b_priority) {
                            std::cmp::Ordering::Equal => a.salary_offer.cmp(&b.salary_offer),
                            other => other.reverse()
                        }
                    });

                if let Some(offer) = best_offer {
                    let priority_info = strategy.get_team_priority(offer.from_team_id)
                        .map(|p| format!("（我偏好的第{}位球队）", p))
                        .unwrap_or_default();

                    thinking_steps.push(format!(
                        "\n🤔 结论: 在{}份可接受的报价中，{} 的报价最符合我的期望{}。决定加盟！",
                        acceptable_offers.len(),
                        offer.from_team_name,
                        priority_info
                    ));

                    return PlayerDecision {
                        accepted_offer_id: offer.id,
                        reasoning: thinking_steps.join("\n"),
                    };
                }
            }

            // 没有完全满足条件的报价，但必须选一个（选薪资最高的）
            thinking_steps.push("\n⚠️ 没有完全符合期望的报价，但必须做出选择...".to_string());
        } else {
            thinking_steps.push("📊 没有明确的期望条件，直接选择最佳报价".to_string());
        }

        // 选择薪资最高的报价（无论是否满足期望）
        let best_offer = offers.iter()
            .max_by_key(|o| o.salary_offer);

        match best_offer {
            Some(offer) => {
                thinking_steps.push(format!(
                    "\n🤔 结论: 选择薪资最高的 {} 的报价（{}万/年）。",
                    offer.from_team_name,
                    offer.salary_offer
                ));

                PlayerDecision {
                    accepted_offer_id: offer.id,
                    reasoning: thinking_steps.join("\n"),
                }
            }
            None => {
                // 理论上不应该到这里，因为offers不为空才会调用此函数
                // 但为了安全，返回第一个报价
                let first_offer = &offers[0];
                thinking_steps.push(format!(
                    "\n🤔 结论: 选择 {} 的报价。",
                    first_offer.from_team_name
                ));

                PlayerDecision {
                    accepted_offer_id: first_offer.id,
                    reasoning: thinking_steps.join("\n"),
                }
            }
        }
    }

    /// 完成选手签约
    fn complete_player_signing(
        &mut self,
        neg_id: u64,
        player: &Player,
        accepted_offer: &Offer,
        all_offers: &[Offer],
        reasoning: &str,
    ) {
        // 更新谈判状态
        if let Some(neg) = self.negotiations.get_mut(&neg_id) {
            // 将接受的报价标记为 Accepted
            if let Some(off) = neg.offers.iter_mut().find(|o| o.id == accepted_offer.id) {
                off.status = OfferStatus::Accepted;
            }

            // 将其他报价标记为 Rejected
            for offer in all_offers {
                if offer.id != accepted_offer.id {
                    if let Some(off) = neg.offers.iter_mut().find(|o| o.id == offer.id) {
                        off.status = OfferStatus::Rejected;
                    }
                    neg.add_response(OfferResponse::reject(
                        offer.id,
                        player.id,
                        "选手已接受其他球队的报价".to_string(),
                    ));
                }
            }

            // 添加接受回应
            neg.add_response(OfferResponse::accept(
                accepted_offer.id,
                player.id,
                reasoning.to_string(),
            ));

            // 完成签约
            neg.complete_signing(
                accepted_offer.from_team_id,
                accepted_offer.from_team_name.clone(),
                accepted_offer.salary_offer,
                accepted_offer.contract_years,
                accepted_offer.guarantee_starter,
            );
        }

        // 移除自由球员
        self.state.remove_free_agent(player.id);
        self.state.complete_negotiation(neg_id);

        // 记录签约事件
        self.record_event_with_player(
            MarketEventType::SigningCompleted,
            player.id,
            player.game_id.clone(),
            accepted_offer.from_team_id,
            accepted_offer.from_team_name.clone(),
            format!("{} 加盟 {}", player.game_id, accepted_offer.from_team_name),
            format!(
                "{}万/年，{}年合同{}",
                accepted_offer.salary_offer,
                accepted_offer.contract_years,
                if accepted_offer.guarantee_starter { "，首发保证" } else { "" }
            ),
        );
    }

    /// 拒绝所有报价
    fn reject_all_offers(
        &mut self,
        neg_id: u64,
        player_id: u64,
        offers: &[Offer],
        reasoning: &str,
    ) {
        if let Some(neg) = self.negotiations.get_mut(&neg_id) {
            for offer in offers {
                if let Some(off) = neg.offers.iter_mut().find(|o| o.id == offer.id) {
                    off.status = OfferStatus::Rejected;
                }
                neg.add_response(OfferResponse::reject(
                    offer.id,
                    player_id,
                    reasoning.to_string(),
                ));
            }
        }
    }

    /// 获取等待中的选手数量
    fn get_waiting_players_count(&self) -> u32 {
        // 自由球员中，没有收到任何报价或报价被拒绝后仍在等待的选手
        let players_with_open_negotiations: std::collections::HashSet<u64> = self.negotiations.values()
            .filter(|n| n.status == NegotiationStatus::Open)
            .map(|n| n.player_id)
            .collect();

        // 没有收到任何报价的自由球员
        let no_offer_count = self.state.free_agent_ids.iter()
            .filter(|id| !players_with_open_negotiations.contains(*id))
            .count() as u32;

        // 有开放谈判但未做出决定的选手
        let waiting_with_offers = players_with_open_negotiations.len() as u32;

        no_offer_count + waiting_with_offers
    }

    /// 创建谈判
    fn create_negotiation(&mut self, player: &Player) -> u64 {
        let id = self.next_negotiation_id;
        self.next_negotiation_id += 1;

        let position = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        let mut negotiation = Negotiation::new(
            self.state.save_id.clone(),
            self.state.season_id,
            player.id,
            player.game_id.clone(),
            position,
            player.ability,
            player.team_id,
            None, // from_team_name 需要单独填充
        );
        negotiation.id = id;

        self.negotiations.insert(id, negotiation);
        self.state.add_active_negotiation(id);

        id
    }

    // ==================== 事件记录 ====================

    /// 记录事件
    fn record_event(
        &mut self,
        event_type: MarketEventType,
        title: String,
        description: String,
    ) {
        let id = self.next_event_id;
        self.next_event_id += 1;

        let mut event = MarketEvent::new(
            self.state.save_id.clone(),
            self.state.season_id,
            event_type,
            self.state.current_phase,
            self.state.current_round,
        );
        event.id = id;
        event.title = title;
        event.description = description;

        self.events.push(event);
    }

    /// 记录选手相关事件
    pub fn record_event_with_player(
        &mut self,
        event_type: MarketEventType,
        player_id: u64,
        player_name: String,
        team_id: u64,
        team_name: String,
        title: String,
        description: String,
    ) {
        let id = self.next_event_id;
        self.next_event_id += 1;

        let event = MarketEvent::new(
            self.state.save_id.clone(),
            self.state.season_id,
            event_type,
            self.state.current_phase,
            self.state.current_round,
        )
        .with_player(player_id, player_name)
        .with_team(team_id, team_name)
        .with_content(title, description);

        let mut event = event;
        event.id = id;

        self.events.push(event);
    }

    // ==================== 查询方法 ====================

    /// 获取市场状态摘要
    pub fn get_state_summary(&self) -> crate::models::MarketStateSummary {
        (&self.state).into()
    }

    /// 获取所有活跃谈判
    pub fn get_active_negotiations(&self) -> Vec<&Negotiation> {
        self.negotiations.values()
            .filter(|n| n.status == NegotiationStatus::Open)
            .collect()
    }

    /// 获取谈判详情
    pub fn get_negotiation(&self, id: u64) -> Option<&Negotiation> {
        self.negotiations.get(&id)
    }

    /// 获取选手策略
    pub fn get_player_strategy(&self, player_id: u64) -> Option<&PlayerTransferStrategy> {
        self.player_strategies.get(&player_id)
    }

    /// 获取球队策略
    pub fn get_team_strategy(&self, team_id: u64) -> Option<&AITransferStrategy> {
        self.team_strategies.get(&team_id)
    }

    /// 获取所有事件
    pub fn get_events(&self) -> &[MarketEvent] {
        &self.events
    }

    /// 获取指定轮次的事件
    pub fn get_events_for_round(&self, round: u8) -> Vec<&MarketEvent> {
        self.events.iter()
            .filter(|e| e.round == round)
            .collect()
    }

    /// 获取想离队的选手列表
    pub fn get_departure_candidates(&self) -> Vec<u64> {
        self.player_strategies.iter()
            .filter(|(_, s)| s.wants_to_leave)
            .map(|(id, _)| *id)
            .collect()
    }

    // ==================== 阶段 7: 转会轮次（挖人） ====================
    //
    // 挖人阶段设计（参考自由市场）：
    // 1. 阶段1（战队报价）：战队评估所有85+能力值的选手，决定是否发起挖角
    // 2. 阶段2（选手决策）：选手评估报价，决定是否接受
    // 3. 反馈循环：转会信息反馈给战队，继续发起新报价
    //
    // 关键区别：
    // - 需要支付转会费（约等于选手市场价值）
    // - 年薪需要与原合同保持一致

    /// 初始化挖人阶段
    ///
    /// 将所有能力值85+的有合同选手添加到可挖角评估列表
    pub fn initialize_transfer_rounds(&mut self, players: &HashMap<u64, Player>) {
        log::info!("初始化转会轮次（挖人阶段）...");

        // 清空旧列表
        self.state.poachable_player_ids.clear();

        // 找出所有能力值 >= 85 且有合同的选手作为潜在挖角目标
        for (player_id, player) in players {
            // 跳过自由球员
            if self.state.free_agent_ids.contains(player_id) {
                continue;
            }

            // 必须有球队（有合同）且状态为活跃
            if player.team_id.is_none() || player.status != PlayerStatus::Active {
                continue;
            }

            // 能力值必须 >= 85
            if player.ability < 85 {
                continue;
            }

            self.state.add_poachable_player(*player_id);
            log::info!(
                "添加可挖角候选: {} (ID: {}, 能力: {}, 球队: {:?})",
                player.game_id, player_id, player.ability, player.team_id
            );
        }

        log::info!(
            "挖人阶段初始化完成: {} 名85+能力值选手可供挖角评估",
            self.state.poachable_player_ids.len()
        );

        // 重置转会轮次计数
        self.state.transfer_round = 0;
        self.state.is_transfer_stable = false;
        self.state.transfer_stable_rounds_count = 0;
    }

    /// 执行一轮挖人（转会轮次）
    ///
    /// 流程与自由市场类似：
    /// 1. 战队向可挖人选手发出报价（需要转会费）
    /// 2. 选手评估报价并做出决策
    /// 3. 反馈给战队
    pub async fn execute_transfer_round(
        &mut self,
        teams: &HashMap<u64, Team>,
        players: &HashMap<u64, Player>,
        player_honors: &HashMap<u64, crate::services::llm_service::PlayerHonorInfo>,
        player_performances: &HashMap<u64, crate::services::llm_service::PlayerPerformanceInfo>,
    ) -> RoundExecutionResult {
        let phase = self.state.current_phase;
        let round = self.state.transfer_round;

        log::info!("========== 执行转会轮次第 {} 轮（挖人） ==========", round + 1);

        // 阶段1：战队发起挖人报价
        log::info!("--- 阶段1：战队 LLM 发起挖人报价 ---");
        let new_negotiations = self.execute_team_poaching_phase(teams, players, round, player_honors, player_performances).await;

        // 阶段2：选手处理挖人报价
        log::info!("--- 阶段2：选手 Agent LLM 处理挖人报价 ---");
        let (completed_transfers, transfer_infos) = self.execute_player_transfer_decision_phase(players, round).await;

        // 阶段3：反馈给战队
        log::info!("--- 阶段3：反馈转会信息给战队 ---");
        self.provide_transfer_feedback_to_teams(&transfer_infos, teams);

        // 统计等待中的可挖人选手
        let waiting_players = self.state.poachable_player_ids.len();
        log::info!(
            "本轮统计: 新增{}个谈判, 完成{}笔转会, {}名选手仍可被挖",
            new_negotiations, completed_transfers, waiting_players
        );

        // 检查市场稳定性
        if new_negotiations == 0 && completed_transfers == 0 {
            self.state.record_transfer_stable_round();
        }

        // 推进转会轮次
        self.state.advance_transfer_round();

        // 检查是否需要结束转会轮次
        let phase_changed = self.state.should_end_transfer_rounds();
        let new_phase = if phase_changed {
            self.state.advance_phase();
            Some(self.state.current_phase)
        } else {
            None
        };

        RoundExecutionResult {
            phase,
            round,
            phase_changed,
            new_phase,
            events: Vec::new(),
            new_negotiations: new_negotiations as usize,
            completed_signings: completed_transfers as usize,
            summary: format!(
                "转会轮次第{}轮: 新增{}个谈判, 完成{}笔转会, {}名选手仍可被挖",
                round + 1, new_negotiations, completed_transfers, waiting_players
            ),
        }
    }

    /// 战队发起挖人报价（需要转会费）
    ///
    /// 每支战队调用 LLM 评估所有85+选手，选择1个发出报价
    /// 报价需要包含：
    /// - 转会费（约等于选手市场价值）
    /// - 年薪（与原合同一致）
    async fn execute_team_poaching_phase(
        &mut self,
        teams: &HashMap<u64, Team>,
        players: &HashMap<u64, Player>,
        round: u8,
        player_honors: &HashMap<u64, crate::services::llm_service::PlayerHonorInfo>,
        player_performances: &HashMap<u64, crate::services::llm_service::PlayerPerformanceInfo>,
    ) -> u32 {
        let mut new_negotiations = 0;

        // 构建可挖角选手列表（所有85+且有合同的选手）
        let poachable_players: Vec<&Player> = self.state.poachable_player_ids
            .iter()
            .filter_map(|&pid| players.get(&pid))
            .collect();

        log::info!("当前可挖角选手数量: {}", poachable_players.len());

        if poachable_players.is_empty() {
            log::info!("没有可挖角的选手，跳过本轮");
            return 0;
        }

        // 获取已完成转会的选手
        let transferred_player_ids: std::collections::HashSet<u64> = self.negotiations.values()
            .filter(|n| n.status == NegotiationStatus::Accepted && n.transfer_fee.is_some())
            .map(|n| n.player_id)
            .collect();

        // 获取已报价过的选手（避免重复报价）
        let already_offered_ids: std::collections::HashSet<u64> = self.negotiations.values()
            .filter(|n| n.transfer_fee.is_some())  // 挖人报价都有转会费
            .flat_map(|n| n.offers.iter().map(|o| (n.player_id, o.from_team_id)))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|(pid, _)| pid)
            .collect();

        // 并发：每个战队调用 LLM 评估挖角市场
        let llm_service = match create_llm_service() {
            Some(service) => service,
            None => {
                log::warn!("LLM 服务未配置，跳过挖角市场评估");
                return 0;
            }
        };

        let llm_service = std::sync::Arc::new(llm_service);

        // 构建并发任务
        let evaluation_futures: Vec<_> = teams.iter()
            .filter_map(|(team_id, team)| {
                let strategy = self.team_strategies.get(team_id)?;
                let team = team.clone();
                let strategy = strategy.clone();
                let poachable_players_clone = poachable_players.clone();
                let player_strategies = self.player_strategies.clone();
                let transferred_ids = transferred_player_ids.clone();
                let offered_ids = already_offered_ids.clone();
                let honors = player_honors.clone();
                let performances = player_performances.clone();
                let llm = llm_service.clone();

                Some(async move {
                    log::info!("战队 {} 开始 LLM 评估挖角市场...", team.name);
                    let result = llm.evaluate_poaching_market(
                        &team,
                        &strategy,
                        &poachable_players_clone,
                        &player_strategies,
                        &transferred_ids,
                        &offered_ids,
                        round,
                        &honors,
                        &performances,
                    ).await;

                    match &result {
                        Ok(eval) => {
                            log::info!(
                                "✓ {} LLM 评估完成，选中: {}",
                                team.name,
                                eval.chosen_player_id.map(|id| format!("ID:{}", id)).unwrap_or("无".to_string())
                            );
                        }
                        Err(e) => {
                            log::error!("✗ {} LLM 评估失败: {}", team.name, e);
                        }
                    }

                    (*team_id, result)
                })
            })
            .collect();

        log::info!("并发执行 {} 个战队的挖角市场 LLM 评估", evaluation_futures.len());
        let evaluations = futures::future::join_all(evaluation_futures).await;

        // 处理评估结果，发起报价
        for (team_id, eval_result) in evaluations {
            let team = match teams.get(&team_id) {
                Some(t) => t,
                None => continue,
            };

            match eval_result {
                Ok(evaluation) => {
                    if let Some(chosen_player_id) = evaluation.chosen_player_id {
                        if let Some(player) = players.get(&chosen_player_id) {
                            // 计算转会费
                            let transfer_fee = player.calculate_market_value() / 10000;

                            // 检查预算
                            let team_state = self.state.get_team_state(team_id);
                            let remaining_budget = team_state.map(|s| s.remaining_budget).unwrap_or(0);

                            if transfer_fee > remaining_budget {
                                log::warn!(
                                    "○ {} 预算不足以支付 {} 的转会费 (需要{}万, 剩余{}万)，LLM评估被忽略",
                                    team.name, player.game_id, transfer_fee, remaining_budget
                                );
                                continue;
                            }

                            // 创建挖人谈判
                            let neg_id = self.find_or_create_transfer_negotiation(player, transfer_fee);

                            // 生成挖人报价
                            let offer_created = self.generate_transfer_offer(
                                neg_id,
                                team,
                                player,
                                transfer_fee,
                                round,
                            );

                            if offer_created {
                                if self.negotiations.get(&neg_id).map(|n| n.offers.len() == 1).unwrap_or(false) {
                                    new_negotiations += 1;
                                }

                                // 记录挖人报价事件（包含 LLM 分析）
                                self.record_event_with_player(
                                    MarketEventType::TransferOfferMade,
                                    player.id,
                                    player.game_id.clone(),
                                    team.id,
                                    team.name.clone(),
                                    format!("{} 向 {} 发出挖人报价", team.name, player.game_id),
                                    format!("第{}轮挖人报价，转会费{}万\n\nLLM分析：{}",
                                        round + 1, transfer_fee, evaluation.overall_reasoning),
                                );

                                log::info!(
                                    "✓ {} 向 {} 发出挖人报价，转会费 {}万 (LLM评估)",
                                    team.name, player.game_id, transfer_fee
                                );
                            }
                        } else {
                            log::warn!("LLM 选中的选手 ID:{} 不存在", chosen_player_id);
                        }
                    } else {
                        log::info!("○ {} 本轮 LLM 评估后决定不报价", team.name);
                    }
                }
                Err(e) => {
                    log::error!("✗ {} LLM 评估失败: {}", team.name, e);
                }
            }
        }

        log::info!("挖角市场评估完成，新增 {} 个挖人报价", new_negotiations);
        new_negotiations
    }

    /// 选手处理挖人报价
    ///
    /// 每个收到挖人报价的选手评估并做出决策
    async fn execute_player_transfer_decision_phase(
        &mut self,
        players: &HashMap<u64, Player>,
        round: u8,
    ) -> (u32, Vec<TransferInfo>) {
        let mut completed_transfers = 0;
        let mut transfer_infos: Vec<TransferInfo> = Vec::new();

        // 收集所有需要处理的挖人谈判
        let negotiation_ids: Vec<u64> = self.negotiations.iter()
            .filter(|(_, n)| {
                n.status == NegotiationStatus::Open &&
                n.transfer_fee.is_some() // 只处理有转会费的谈判（挖人）
            })
            .map(|(id, _)| *id)
            .collect();

        for neg_id in negotiation_ids {
            let negotiation = match self.negotiations.get(&neg_id) {
                Some(n) => n.clone(),
                None => continue,
            };

            let player = match players.get(&negotiation.player_id) {
                Some(p) => p,
                None => continue,
            };

            // 检查选手是否仍在可挖人列表中
            if !self.state.poachable_player_ids.contains(&player.id) {
                continue;
            }

            // 获取所有待处理的报价
            let pending_offers: Vec<_> = negotiation.offers.iter()
                .filter(|o| o.status == OfferStatus::Pending)
                .cloned()
                .collect();

            if pending_offers.is_empty() {
                continue;
            }

            log::info!(
                "选手 {} 收到 {} 份挖人报价，正在评估...",
                player.game_id, pending_offers.len()
            );

            // 评估挖人报价
            let decision = self.evaluate_transfer_offers(player, &pending_offers, round);

            // 构建报价列表描述
            let offers_desc = pending_offers.iter()
                .map(|o| format!(
                    "{}(转会费{}万, 年薪{}万)",
                    o.from_team_name, o.transfer_fee, o.salary_offer
                ))
                .collect::<Vec<_>>()
                .join("、");

            if let Some(accepted_offer) = pending_offers.iter().find(|o| o.id == decision.accepted_offer_id) {
                // 记录选手思考过程
                self.record_event_with_player(
                    MarketEventType::PlayerThinking,
                    player.id,
                    player.game_id.clone(),
                    accepted_offer.from_team_id,
                    accepted_offer.from_team_name.clone(),
                    format!("{} 决定转会", player.game_id),
                    format!(
                        "收到挖人报价: {}\n\n💭 思考过程:\n{}\n\n✅ 决定: 转会至 {}",
                        offers_desc, decision.reasoning, accepted_offer.from_team_name
                    ),
                );

                // 完成转会
                let transfer_fee = negotiation.transfer_fee.unwrap_or(0);
                self.complete_player_transfer(
                    neg_id,
                    player,
                    accepted_offer,
                    &pending_offers,
                    transfer_fee,
                    &decision.reasoning,
                );

                completed_transfers += 1;
                transfer_infos.push(TransferInfo {
                    player_id: player.id,
                    player_name: player.game_id.clone(),
                    from_team_id: player.team_id.unwrap_or(0),
                    to_team_id: accepted_offer.from_team_id,
                    to_team_name: accepted_offer.from_team_name.clone(),
                    transfer_fee,
                    salary: accepted_offer.salary_offer,
                    years: accepted_offer.contract_years,
                });

                log::info!(
                    "✓ {} 转会至 {}，转会费 {}万，年薪 {}万/年",
                    player.game_id, accepted_offer.from_team_name,
                    transfer_fee, accepted_offer.salary_offer
                );
            }
        }

        (completed_transfers, transfer_infos)
    }

    /// 查找或创建挖人谈判
    /// 查找或创建挖人谈判（带转会费）
    pub fn find_or_create_transfer_negotiation(&mut self, player: &Player, transfer_fee: u64) -> u64 {
        // 查找现有的活跃挖人谈判
        if let Some(neg) = self.negotiations.values().find(|n| {
            n.player_id == player.id &&
            n.status == NegotiationStatus::Open &&
            n.transfer_fee.is_some()
        }) {
            return neg.id;
        }

        // 创建新的挖人谈判
        self.create_transfer_negotiation(player, transfer_fee)
    }

    /// 创建挖人谈判
    fn create_transfer_negotiation(&mut self, player: &Player, transfer_fee: u64) -> u64 {
        let id = self.next_negotiation_id;
        self.next_negotiation_id += 1;

        let position = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        let mut negotiation = Negotiation::new(
            self.state.save_id.clone(),
            self.state.season_id,
            player.id,
            player.game_id.clone(),
            position,
            player.ability,
            player.team_id,
            None,
        );
        negotiation.id = id;
        negotiation.transfer_fee = Some(transfer_fee);
        negotiation.is_transfer = true;

        self.negotiations.insert(id, negotiation);
        self.state.add_active_negotiation(id);

        id
    }

    /// 生成挖人报价
    ///
    /// 年薪与原合同保持一致
    fn generate_transfer_offer(
        &mut self,
        negotiation_id: u64,
        team: &Team,
        player: &Player,
        transfer_fee: u64,
        round: u8,
    ) -> bool {
        // 获取选手原合同年薪（保持一致）
        let original_salary = player.salary / 10000; // 转换为万

        // 获取选手策略
        let player_strategy = self.player_strategies.get(&player.id);
        let expected_years = player_strategy.map(|s| s.expected_years).unwrap_or(2);
        let requires_starter = player_strategy.map(|s| s.requires_starter).unwrap_or(false);

        let offer_id = self.next_offer_id;
        self.next_offer_id += 1;

        let mut offer = Offer::new(
            negotiation_id,
            team.id,
            team.name.clone(),
            player.id,
            round,
        );
        offer.id = offer_id;
        offer.salary_offer = original_salary; // 年薪与原合同一致
        offer.contract_years = expected_years;
        offer.guarantee_starter = requires_starter;
        offer.transfer_fee = transfer_fee;
        offer.offer_reasoning = format!(
            "向 {} 发出挖人报价，转会费 {}万，年薪 {}万/年（与原合同一致）",
            player.game_id, transfer_fee, original_salary
        );

        // 添加到谈判
        if let Some(neg) = self.negotiations.get_mut(&negotiation_id) {
            neg.offers.push(offer);
            return true;
        }

        false
    }

    /// 评估挖人报价（简化版，可后续接入 LLM）
    fn evaluate_transfer_offers(
        &self,
        player: &Player,
        offers: &[Offer],
        _round: u8,
    ) -> PlayerDecision {
        let mut thinking_steps = Vec::new();

        thinking_steps.push(format!(
            "📋 选手 {} 评估挖人报价\n当前球队: {}\n能力值: {}\n当前年薪: {}万",
            player.game_id,
            player.team_id.map(|_| "有合同").unwrap_or("自由身"),
            player.ability,
            player.salary / 10000
        ));

        // 获取选手策略
        let strategy = self.player_strategies.get(&player.id);

        thinking_steps.push("\n📊 收到的挖人报价:".to_string());
        for (i, offer) in offers.iter().enumerate() {
            let priority = strategy.and_then(|s| s.get_team_priority(offer.from_team_id));
            thinking_steps.push(format!(
                "  {}. {} (转会费{}万, 年薪{}万/年, {}年, 偏好{})",
                i + 1,
                offer.from_team_name,
                offer.transfer_fee,
                offer.salary_offer,
                offer.contract_years,
                priority.map(|p| format!("第{}位", p)).unwrap_or("无".to_string())
            ));
        }

        // 选择逻辑：优先选择偏好球队，其次选择转会费最高的
        let best_offer = if let Some(strat) = strategy {
            // 如果有偏好球队在报价中，优先选择
            let preferred_offer = offers.iter()
                .filter(|o| strat.get_team_priority(o.from_team_id).is_some())
                .min_by_key(|o| strat.get_team_priority(o.from_team_id).unwrap_or(255));

            if let Some(offer) = preferred_offer {
                thinking_steps.push(format!(
                    "\n🤔 结论: {} 是我偏好的球队，决定接受转会！",
                    offer.from_team_name
                ));
                offer
            } else {
                // 没有偏好球队，选择条件最好的
                let best = offers.iter()
                    .max_by_key(|o| o.salary_offer)
                    .unwrap_or(&offers[0]);

                thinking_steps.push(format!(
                    "\n🤔 结论: 虽然没有偏好球队，但 {} 的条件不错，决定接受转会！",
                    best.from_team_name
                ));
                best
            }
        } else {
            // 没有策略信息，选择第一个报价
            let best = &offers[0];
            thinking_steps.push(format!(
                "\n🤔 结论: 选择 {} 的报价。",
                best.from_team_name
            ));
            best
        };

        PlayerDecision {
            accepted_offer_id: best_offer.id,
            reasoning: thinking_steps.join("\n"),
        }
    }

    /// 完成选手转会
    fn complete_player_transfer(
        &mut self,
        neg_id: u64,
        player: &Player,
        accepted_offer: &Offer,
        all_offers: &[Offer],
        transfer_fee: u64,
        reasoning: &str,
    ) {
        // 更新谈判状态
        if let Some(neg) = self.negotiations.get_mut(&neg_id) {
            // 将接受的报价标记为 Accepted
            if let Some(off) = neg.offers.iter_mut().find(|o| o.id == accepted_offer.id) {
                off.status = OfferStatus::Accepted;
            }

            // 将其他报价标记为 Rejected
            for offer in all_offers {
                if offer.id != accepted_offer.id {
                    if let Some(off) = neg.offers.iter_mut().find(|o| o.id == offer.id) {
                        off.status = OfferStatus::Rejected;
                    }
                    neg.add_response(OfferResponse::reject(
                        offer.id,
                        player.id,
                        "选手已接受其他球队的挖人报价".to_string(),
                    ));
                }
            }

            // 添加接受回应
            neg.add_response(OfferResponse::accept(
                accepted_offer.id,
                player.id,
                reasoning.to_string(),
            ));

            // 完成转会
            neg.complete_transfer(
                accepted_offer.from_team_id,
                accepted_offer.from_team_name.clone(),
                accepted_offer.salary_offer,
                accepted_offer.contract_years,
                accepted_offer.guarantee_starter,
                transfer_fee,
            );
        }

        // 从可挖人列表中移除
        self.state.remove_poachable_player(player.id);
        self.state.complete_transfer(neg_id);

        // 记录转会完成事件
        self.record_event_with_player(
            MarketEventType::TransferCompleted,
            player.id,
            player.game_id.clone(),
            accepted_offer.from_team_id,
            accepted_offer.from_team_name.clone(),
            format!("{} 转会至 {}", player.game_id, accepted_offer.from_team_name),
            format!(
                "转会费{}万，年薪{}万/年，{}年合同",
                transfer_fee,
                accepted_offer.salary_offer,
                accepted_offer.contract_years
            ),
        );
    }

    /// 反馈转会信息给战队
    fn provide_transfer_feedback_to_teams(&mut self, transfer_infos: &[TransferInfo], teams: &HashMap<u64, Team>) {
        for transfer in transfer_infos {
            log::info!(
                "反馈: {} 已转会至 {}，转会费 {}万",
                transfer.player_name, transfer.to_team_name, transfer.transfer_fee
            );

            // 更新原球队状态（阵容减少，收到转会费）
            if let Some(team_state) = self.state.get_team_state_mut(transfer.from_team_id) {
                team_state.roster_count = team_state.roster_count.saturating_sub(1);
                team_state.remaining_budget += transfer.transfer_fee;
                team_state.record_departure(transfer.player_id);
            }

            // 更新新球队状态（阵容增加，支付转会费）
            if let Some(team_state) = self.state.get_team_state_mut(transfer.to_team_id) {
                team_state.remaining_budget = team_state.remaining_budget.saturating_sub(transfer.transfer_fee);
                team_state.roster_count += 1;
                // 记录转会费支出
                team_state.spent_amount += transfer.transfer_fee;
            }
        }
    }
}

/// 转会信息（用于反馈给战队）
#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub player_id: u64,
    pub player_name: String,
    pub from_team_id: u64,
    pub to_team_id: u64,
    pub to_team_name: String,
    pub transfer_fee: u64,
    pub salary: u64,
    pub years: u8,
}

/// 紧急签约信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencySigningInfo {
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub salary: u64,
    pub years: u8,
}

// ==================== 阶段 8: Finalization（收尾阶段） ====================

impl TransferMarketEngine {
    /// 执行 Finalization 阶段
    ///
    /// 确保：
    /// 1. 每支球队至少有 5 名选手
    /// 2. 处理剩余的未签约自由球员
    pub async fn execute_finalization(
        &mut self,
        teams: &HashMap<u64, Team>,
        players: &HashMap<u64, Player>,
    ) -> FinalizationResult {
        log::info!("========== 执行收尾阶段（Finalization） ==========");

        let mut result = FinalizationResult {
            emergency_signings: Vec::new(),
            retired_players: Vec::new(),
            remaining_free_agents: Vec::new(),
            summary: String::new(),
        };

        // 步骤1：检查并补全阵容不足的球队
        log::info!("--- 步骤1：补全阵容不足的球队（最少5人） ---");
        let teams_needing_players = self.get_teams_needing_players();

        for team_id in teams_needing_players {
            let team = match teams.get(&team_id) {
                Some(t) => t,
                None => continue,
            };

            let team_state = match self.state.get_team_state(team_id) {
                Some(s) => s.clone(),
                None => continue,
            };

            let deficit = team_state.roster_deficit();
            log::info!(
                "球队 {} 阵容不足，需要紧急补充 {} 名选手",
                team.name, deficit
            );

            // 尝试从自由球员中签约
            for _ in 0..deficit {
                if self.state.free_agent_ids.is_empty() {
                    log::warn!("自由球员池已空，无法继续补充 {} 的阵容", team.name);
                    break;
                }

                // 选择一个自由球员（优先选择能力较高的）
                let best_free_agent = self.find_best_available_free_agent(players);

                if let Some(player_id) = best_free_agent {
                    if let Some(player) = players.get(&player_id) {
                        let signing = self.execute_emergency_signing(team, player);
                        result.emergency_signings.push(signing);
                    }
                }
            }
        }

        // 步骤2：处理剩余的自由球员
        log::info!("--- 步骤2：处理剩余的自由球员 ---");
        let remaining_free_agent_ids = self.state.free_agent_ids.clone();

        for player_id in remaining_free_agent_ids {
            if let Some(player) = players.get(&player_id) {
                // 检查选手年龄，超过38岁的选手退役
                if player.age >= 38 {
                    result.retired_players.push(player_id);
                    self.state.remove_free_agent(player_id);

                    self.record_event_with_player(
                        MarketEventType::PlayerRetired,
                        player_id,
                        player.game_id.clone(),
                        0,
                        String::new(),
                        format!("{} 宣布退役", player.game_id),
                        format!("年龄 {} 岁，未能找到新东家，选择退役", player.age),
                    );

                    log::info!("选手 {} 年龄 {} 岁，宣布退役", player.game_id, player.age);
                } else {
                    // 年轻选手保持自由身状态，等待下赛季
                    result.remaining_free_agents.push(player_id);
                    log::info!("选手 {} 保持自由身状态", player.game_id);
                }
            }
        }

        // 生成总结
        result.summary = format!(
            "收尾阶段完成：紧急签约 {} 人，退役 {} 人，保持自由身 {} 人",
            result.emergency_signings.len(),
            result.retired_players.len(),
            result.remaining_free_agents.len()
        );

        log::info!("{}", result.summary);

        // 推进到完成阶段
        self.state.current_phase = MarketPhase::Completed;
        self.state.updated_at = chrono::Utc::now().to_rfc3339();

        result
    }

    /// 获取阵容不足的球队列表
    fn get_teams_needing_players(&self) -> Vec<u64> {
        self.state.team_states
            .values()
            .filter(|ts| ts.roster_count < ts.min_roster_size)
            .map(|ts| ts.team_id)
            .collect()
    }

    /// 找到最佳的可用自由球员（能力最高的）
    fn find_best_available_free_agent(&self, players: &HashMap<u64, Player>) -> Option<u64> {
        self.state.free_agent_ids.iter()
            .filter_map(|id| players.get(id).map(|p| (id, p)))
            .max_by_key(|(_, p)| p.ability)
            .map(|(id, _)| *id)
    }

    /// 执行紧急签约
    fn execute_emergency_signing(
        &mut self,
        team: &Team,
        player: &Player,
    ) -> EmergencySigningInfo {
        // 计算紧急签约的薪资（市场价值的50%作为年薪，最低保底）
        let base_salary = player.calculate_market_value() / 10000 / 2;
        let min_salary = 50; // 最低50万年薪
        let salary = base_salary.max(min_salary);
        let years = 1; // 紧急签约默认1年合同

        // 创建谈判记录
        let neg_id = self.create_negotiation(player);

        // 创建报价
        let offer_id = self.next_offer_id;
        self.next_offer_id += 1;

        let mut offer = Offer::new(
            neg_id,
            team.id,
            team.name.clone(),
            player.id,
            0, // round 0 表示紧急签约
        );
        offer.id = offer_id;
        offer.salary_offer = salary;
        offer.contract_years = years;
        offer.offer_reasoning = "紧急补充阵容".to_string();
        offer.status = OfferStatus::Accepted;

        // 更新谈判
        if let Some(neg) = self.negotiations.get_mut(&neg_id) {
            neg.add_offer(offer);
            neg.complete_signing(
                team.id,
                team.name.clone(),
                salary,
                years,
                false,
            );
        }

        // 更新状态
        self.state.remove_free_agent(player.id);
        self.state.complete_negotiation(neg_id);

        // 更新球队状态
        if let Some(team_state) = self.state.get_team_state_mut(team.id) {
            team_state.roster_count += 1;
            team_state.salary_increase += salary;
            team_state.needs_emergency_signing = team_state.roster_count < team_state.min_roster_size;
        }

        // 记录事件
        self.record_event_with_player(
            MarketEventType::EmergencySigning,
            player.id,
            player.game_id.clone(),
            team.id,
            team.name.clone(),
            format!("{} 紧急签约 {}", team.name, player.game_id),
            format!("{}万/年，{}年合同（紧急补充阵容）", salary, years),
        );

        log::info!(
            "✓ {} 紧急签约 {}，{}万/年，{}年合同",
            team.name, player.game_id, salary, years
        );

        EmergencySigningInfo {
            player_id: player.id,
            player_name: player.game_id.clone(),
            team_id: team.id,
            team_name: team.name.clone(),
            salary,
            years,
        }
    }

    /// 完成转会市场（标记为已完成）
    pub fn finish_market(&mut self) {
        self.state.current_phase = MarketPhase::Completed;
        self.state.updated_at = chrono::Utc::now().to_rfc3339();

        self.record_event(
            MarketEventType::ContractExpired, // 复用此类型标记市场关闭
            "转会窗口关闭".to_string(),
            format!(
                "{}赛季转会窗口正式关闭，共完成 {} 笔交易",
                self.state.season_id,
                self.state.completed_transfer_ids.len()
            ),
        );

        log::info!(
            "转会市场已关闭，本赛季共完成 {} 笔交易",
            self.state.completed_transfer_ids.len()
        );
    }
}

/// Finalization 阶段结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizationResult {
    /// 紧急签约列表
    pub emergency_signings: Vec<EmergencySigningInfo>,
    /// 退役选手 ID 列表
    pub retired_players: Vec<u64>,
    /// 保持自由身的选手 ID 列表
    pub remaining_free_agents: Vec<u64>,
    /// 总结
    pub summary: String,
}
