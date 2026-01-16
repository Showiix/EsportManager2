//! AI 转会引擎
//!
//! 实现完整的 AI 自动转会系统，包括：
//! - 赛季结算（财务、合同、退役、成长）
//! - 满意度和忠诚度计算
//! - 选手意愿处理（申请转会、续约谈判）
//! - 自由球员签约
//! - 重建球队清洗
//! - 球队买卖交易
//! - 新闻生成

use crate::models::{
    AmbitionLevel, ContractType, FinancialStatus, FreeAgent, FreeAgentReason,
    FreeAgentStatus, NewsImportance, Player, PlayerStatus,
    Team, TeamTransferPlan, TransferEvent, TransferEventStatus, TransferEventType,
    TransferListing, TransferRoundSummary, TransferStrategy,
    TransferWindow, TransferWindowStatus,
    calculate_contract_years, calculate_expected_salary, check_retirement,
    get_round_name, ROSTER_MAX, ROSTER_MIN,
    // 新增：满意度和忠诚度相关
    DepartureReason, PlayerSeasonStatus, TeamSeasonPerformance,
};
use crate::engines::SatisfactionEngine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI 转会窗口引擎
pub struct TransferWindowEngine {
    /// 当前转会窗口
    pub window: TransferWindow,
    /// 所有球队的转会计划
    pub team_plans: HashMap<u64, TeamTransferPlan>,
    /// 自由球员池
    pub free_agent_pool: Vec<FreeAgentInfo>,
    /// 挂牌球员
    pub listed_players: Vec<ListedPlayerInfo>,
    /// 生成的转会事件
    pub events: Vec<TransferEvent>,
    /// 配置
    pub config: TransferEngineConfig,
    /// 选手赛季状态（新增）
    pub player_statuses: HashMap<u64, PlayerSeasonStatus>,
    /// 球队赛季表现（新增）
    pub team_performances: HashMap<u64, TeamSeasonPerformance>,
    /// 想离队的选手列表（新增）
    pub departure_candidates: Vec<u64>,
}

/// 自由球员信息（包含选手详情）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeAgentInfo {
    pub agent: FreeAgent,
    pub player: Player,
    pub market_value: u64,
    pub expected_salary: u64,
    pub minimum_salary: u64,
}

/// 挂牌球员信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListedPlayerInfo {
    pub listing: TransferListing,
    pub player: Player,
    pub team: Team,
    pub market_value: u64,
}

/// 转会引擎配置
#[derive(Debug, Clone)]
pub struct TransferEngineConfig {
    /// 转会预算占余额比例
    pub budget_ratio: f64,
    /// 薪资预算占预计收入比例
    pub salary_budget_ratio: f64,
    /// 自由球员签约：薪资权重
    pub fa_salary_weight: f64,
    /// 自由球员签约：球队实力权重
    pub fa_strength_weight: f64,
    /// 自由球员签约：上场机会权重
    pub fa_opportunity_weight: f64,
    /// 自由球员签约：随机因素权重
    pub fa_random_weight: f64,
}

impl Default for TransferEngineConfig {
    fn default() -> Self {
        Self {
            budget_ratio: 0.6,  // 提高预算比例，允许更多转会
            salary_budget_ratio: 0.6,
            fa_salary_weight: 0.4,
            fa_strength_weight: 0.3,
            fa_opportunity_weight: 0.2,
            fa_random_weight: 0.1,
        }
    }
}

impl Default for TransferWindowEngine {
    fn default() -> Self {
        Self {
            window: TransferWindow::default(),
            team_plans: HashMap::new(),
            free_agent_pool: Vec::new(),
            listed_players: Vec::new(),
            events: Vec::new(),
            config: TransferEngineConfig::default(),
            player_statuses: HashMap::new(),
            team_performances: HashMap::new(),
            departure_candidates: Vec::new(),
        }
    }
}

impl TransferWindowEngine {
    pub fn new(save_id: String, season_id: u64) -> Self {
        let mut engine = Self::default();
        engine.window.save_id = save_id;
        engine.window.season_id = season_id;
        engine
    }

    /// 初始化转会窗口
    pub fn initialize(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
        free_agents: Vec<FreeAgent>,
        listings: Vec<TransferListing>,
    ) {
        // 生成球队转会计划
        for team in teams {
            let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
            let plan = self.generate_team_plan(team, &roster);
            self.team_plans.insert(team.id, plan);
        }

        // 处理自由球员
        for agent in free_agents {
            if let Some(players) = players_by_team.values().flatten().find(|p| p.id == agent.player_id) {
                let player = players.clone();
                let market_value = player.calculate_market_value();
                let salary_exp = calculate_expected_salary(market_value);

                self.free_agent_pool.push(FreeAgentInfo {
                    agent,
                    player,
                    market_value,
                    expected_salary: salary_exp.expected,
                    minimum_salary: salary_exp.minimum,
                });
            }
        }

        // 处理挂牌球员
        for listing in listings {
            if let Some(team) = teams.iter().find(|t| t.id == listing.team_id) {
                if let Some(player) = players_by_team
                    .get(&listing.team_id)
                    .and_then(|r| r.iter().find(|p| p.id == listing.player_id))
                {
                    let market_value = player.calculate_market_value();
                    self.listed_players.push(ListedPlayerInfo {
                        listing,
                        player: player.clone(),
                        team: team.clone(),
                        market_value,
                    });
                }
            }
        }

        self.window.status = TransferWindowStatus::InProgress;
        self.window.started_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// 生成球队转会计划
    fn generate_team_plan(&self, team: &Team, roster: &[Player]) -> TeamTransferPlan {
        let active_roster: Vec<_> = roster
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .collect();

        let roster_count = active_roster.len() as u32;

        // 计算平均能力和年龄
        let (avg_ability, avg_age) = if !active_roster.is_empty() {
            let total_ability: u32 = active_roster.iter().map(|p| p.ability as u32).sum();
            let total_age: u32 = active_roster.iter().map(|p| p.age as u32).sum();
            (
                total_ability as f64 / active_roster.len() as f64,
                total_age as f64 / active_roster.len() as f64,
            )
        } else {
            (0.0, 0.0)
        };

        // 计算总薪资
        let current_total_salary: i64 = active_roster.iter().map(|p| p.salary as i64).sum();

        // 财务状态
        let balance = team.balance;
        let financial_status = FinancialStatus::from_balance(balance);
        let transfer_budget = if financial_status.can_buy() {
            (balance as f64 * self.config.budget_ratio) as i64
        } else {
            0
        };

        // 估算薪资空间（简化计算）
        let estimated_income = 500; // 假设基础收入500万
        let max_salary_budget = (estimated_income as f64 * self.config.salary_budget_ratio) as i64;
        let salary_space = (max_salary_budget - current_total_salary).max(0);

        // 计算位置需求
        let mut position_needs = HashMap::new();
        for pos in &["TOP", "JUG", "MID", "ADC", "SUP"] {
            let count = active_roster
                .iter()
                .filter(|p| p.position.map(|pp| format!("{:?}", pp).to_uppercase() == *pos).unwrap_or(false))
                .count();

            let need = match count {
                0 => 100,  // 急需
                1 => 70,   // 需要
                2 => 30,   // 可考虑
                _ => 0,    // 不需要
            };
            position_needs.insert(pos.to_string(), need);
        }

        // 确定策略
        let must_sign = roster_count < ROSTER_MIN;
        let must_clear = roster_count > ROSTER_MAX;

        let strategy = if must_clear {
            TransferStrategy::ForceClear
        } else if financial_status.must_sell() {
            TransferStrategy::MustSell
        } else if must_sign || position_needs.values().any(|&n| n >= 100) {
            TransferStrategy::AggressiveBuy
        } else if financial_status == FinancialStatus::Wealthy && position_needs.values().any(|&n| n >= 70) {
            TransferStrategy::AggressiveBuy
        } else {
            TransferStrategy::Passive
        };

        // 确定野心（基于平均能力）
        let ambition = if avg_ability >= 85.0 {
            AmbitionLevel::Championship
        } else if avg_ability >= 75.0 {
            AmbitionLevel::Playoff
        } else {
            AmbitionLevel::Rebuild
        };

        TeamTransferPlan {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            team_id: team.id,
            balance,
            financial_status,
            transfer_budget,
            salary_space,
            current_total_salary,
            roster_count,
            avg_ability,
            avg_age,
            position_needs,
            strategy,
            ambition,
            must_sign,
            must_clear,
        }
    }

    /// 执行第0轮：赛季结算（满意度/忠诚度计算）
    ///
    /// 计算所有选手的满意度变化和忠诚度变化，
    /// 识别想要离队的选手
    pub fn execute_round_0_season_settlement(
        &mut self,
        players: &mut [Player],
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 0;

        for player in players.iter_mut() {
            if player.status != PlayerStatus::Active {
                continue;
            }

            let team_id = match player.team_id {
                Some(id) => id,
                None => continue,
            };

            let team = match teams.iter().find(|t| t.id == team_id) {
                Some(t) => t,
                None => continue,
            };

            // 获取球队赛季表现
            let team_perf = self.team_performances
                .get(&team_id)
                .cloned()
                .unwrap_or_else(|| TeamSeasonPerformance::new(
                    self.window.save_id.clone(),
                    self.window.season_id,
                    team_id,
                ));

            // 获取或创建选手赛季状态
            let status = self.player_statuses
                .entry(player.id)
                .or_insert_with(|| {
                    let mut s = PlayerSeasonStatus::new(
                        self.window.save_id.clone(),
                        self.window.season_id,
                        player.id,
                    );
                    s.satisfaction = SatisfactionEngine::initial_satisfaction(player, false);
                    s
                });

            // 计算满意度变化
            let market_value = player.calculate_market_value();
            let satisfaction_change = SatisfactionEngine::calculate_season_changes(
                player,
                &team_perf,
                status.games_as_starter,
                status.total_games,
                market_value,
            );
            status.update_satisfaction(satisfaction_change);

            // 判断是否想离队
            let (wants_to_leave, reasons) = SatisfactionEngine::check_departure_intent(
                status.satisfaction,
                player.loyalty,
                player,
                &team_perf,
            );
            status.wants_to_leave = wants_to_leave;
            status.departure_reasons = reasons.clone();

            if wants_to_leave {
                self.departure_candidates.push(player.id);
            }
        }

        // 更新球队策略（基于球队表现）
        for (team_id, perf) in &self.team_performances {
            if let Some(plan) = self.team_plans.get_mut(team_id) {
                // 连续2赛季没进季后赛，触发重建
                if perf.consecutive_no_playoffs >= 2 {
                    plan.strategy = TransferStrategy::FullRebuild;
                }
                // 战绩优秀的球队可能追逐巨星
                else if perf.made_playoffs && plan.financial_status == FinancialStatus::Wealthy {
                    plan.strategy = TransferStrategy::StarHunting;
                }
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第1轮：合同到期与退役
    pub fn execute_round_1_contracts_and_retirements(
        &mut self,
        players: &mut [Player],
        teams: &[Team],
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 1;

        for player in players.iter_mut() {
            if player.status != PlayerStatus::Active {
                continue;
            }

            // 检查退役
            let retirement = check_retirement(player.age, player.ability);
            if retirement.should_retire {
                let team = player.team_id.and_then(|tid| teams.iter().find(|t| t.id == tid));

                let event = self.create_retirement_event(round, player, team);
                events.push(event);

                player.status = PlayerStatus::Retired;
                player.retire_season = Some(self.window.season_id as u32);
                self.window.retirements += 1;
                continue;
            }

            // 检查合同到期
            if let Some(contract_end) = player.contract_end_season {
                if contract_end <= self.window.season_id as u32 {
                    let team = player.team_id.and_then(|tid| teams.iter().find(|t| t.id == tid));

                    let event = self.create_contract_expire_event(round, player, team);
                    events.push(event);

                    // 加入自由球员池
                    let market_value = player.calculate_market_value();
                    let salary_exp = calculate_expected_salary(market_value);

                    self.free_agent_pool.push(FreeAgentInfo {
                        agent: FreeAgent {
                            id: 0,
                            save_id: self.window.save_id.clone(),
                            season_id: self.window.season_id,
                            player_id: player.id,
                            salary_demand: salary_exp.expected,
                            reason: FreeAgentReason::ContractExpire,
                            status: FreeAgentStatus::Available,
                        },
                        player: player.clone(),
                        market_value,
                        expected_salary: salary_exp.expected,
                        minimum_salary: salary_exp.minimum,
                    });

                    player.team_id = None;
                    self.window.contract_expires += 1;
                }
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第2轮：选手意愿处理
    ///
    /// 处理选手的转会申请、续约谈判和离队意愿
    pub fn execute_round_2_player_intentions(
        &mut self,
        players: &mut [Player],
        teams: &[Team],
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 2;
        let mut rng = rand::thread_rng();

        // 如果 departure_candidates 为空，基于选手属性初始化离队意愿
        if self.departure_candidates.is_empty() {
            for player in players.iter() {
                if player.status != PlayerStatus::Active || player.team_id.is_none() {
                    continue;
                }
                // 低满意度或低忠诚度的选手想离队
                // 满意度 < 40 或 忠诚度 < 35 的选手有离队倾向
                let wants_to_leave = player.satisfaction < 40 || player.loyalty < 35;
                if wants_to_leave {
                    self.departure_candidates.push(player.id);
                    // 根据选手情况生成离队原因
                    let mut reasons = Vec::new();
                    if player.satisfaction < 30 {
                        reasons.push(DepartureReason::LackOfPlaytime);
                    }
                    if player.ability >= 85 {
                        reasons.push(DepartureReason::SeekingChampionship);
                    }
                    if reasons.is_empty() {
                        reasons.push(DepartureReason::SeekingOpportunity);
                    }
                    // 创建临时状态
                    let status = PlayerSeasonStatus {
                        id: 0,
                        save_id: self.window.save_id.clone(),
                        season_id: self.window.season_id,
                        player_id: player.id,
                        satisfaction: player.satisfaction,
                        wants_to_leave: true,
                        departure_reasons: reasons,
                        games_as_starter: 0,
                        total_games: 0,
                        created_at: None,
                        updated_at: None,
                    };
                    self.player_statuses.insert(player.id, status);
                }
            }
        }

        // 处理想离队的选手
        let departure_ids = self.departure_candidates.clone();
        for player_id in departure_ids {
            let player = match players.iter_mut().find(|p| p.id == player_id) {
                Some(p) => p,
                None => continue,
            };

            let status = match self.player_statuses.get(&player_id) {
                Some(s) => s.clone(),
                None => continue,
            };

            let current_team_id = match player.team_id {
                Some(id) => id,
                None => continue,
            };

            let current_team = match teams.iter().find(|t| t.id == current_team_id) {
                Some(t) => t,
                None => continue,
            };

            // 高忠诚度选手有机会改变主意
            if player.loyalty >= 80 {
                let stay_chance = player.reject_poaching_chance();
                if rng.gen::<f64>() < stay_chance {
                    // 选手决定留下
                    let event = self.create_loyalty_stay_event(round, player, current_team);
                    events.push(event);

                    // 从离队候选人中移除
                    self.departure_candidates.retain(|&id| id != player_id);
                    if let Some(s) = self.player_statuses.get_mut(&player_id) {
                        s.wants_to_leave = false;
                    }
                    continue;
                }
            }

            // 根据离队原因寻找合适的球队
            let primary_reason = status.departure_reasons.first().cloned();

            match primary_reason {
                Some(DepartureReason::SeekingChampionship) => {
                    // 追求冠军：寻找强队
                    if let Some((buyer_team, price)) = self.find_contender_for_star(player, teams) {
                        let position_key = player.position
                            .map(|p| format!("{:?}", p).to_uppercase())
                            .unwrap_or_default();

                        // 检查忠诚度溢价
                        let final_price = (price as f64 * player.loyalty_price_factor()) as u64;
                        let event = self.create_transfer_request_event(
                            round,
                            player,
                            current_team,
                            &buyer_team,
                            final_price,
                        );
                        events.push(event);

                        // 更新计划
                        if let Some(sp) = self.team_plans.get_mut(&current_team_id) {
                            sp.roster_count -= 1;
                            sp.balance += final_price as i64;
                            // 卖出后增加该位置需求
                            *sp.position_needs.entry(position_key.clone()).or_insert(0) += 30;
                        }
                        if let Some(bp) = self.team_plans.get_mut(&buyer_team.id) {
                            bp.roster_count += 1;
                            bp.balance -= final_price as i64;
                            bp.transfer_budget -= final_price as i64;
                            // 买入后大幅降低该位置需求
                            if let Some(need) = bp.position_needs.get_mut(&position_key) {
                                *need = (*need).saturating_sub(50);
                            }
                        }

                        self.window.total_transfers += 1;
                        self.window.total_fees += final_price;
                    }
                }
                Some(DepartureReason::SeekingOpportunity) | Some(DepartureReason::LackOfPlaytime) => {
                    // 寻找上场机会：寻找需要该位置的球队
                    if let Some((buyer_team, price)) = self.find_team_needing_position(player, teams) {
                        let position_key = player.position
                            .map(|p| format!("{:?}", p).to_uppercase())
                            .unwrap_or_default();

                        let event = self.create_transfer_request_event(
                            round,
                            player,
                            current_team,
                            &buyer_team,
                            price,
                        );
                        events.push(event);

                        if let Some(sp) = self.team_plans.get_mut(&current_team_id) {
                            sp.roster_count -= 1;
                            sp.balance += price as i64;
                            // 卖出后增加该位置需求
                            *sp.position_needs.entry(position_key.clone()).or_insert(0) += 30;
                        }
                        if let Some(bp) = self.team_plans.get_mut(&buyer_team.id) {
                            bp.roster_count += 1;
                            bp.balance -= price as i64;
                            bp.transfer_budget -= price as i64;
                            // 买入后大幅降低该位置需求
                            if let Some(need) = bp.position_needs.get_mut(&position_key) {
                                *need = (*need).saturating_sub(50);
                            }
                        }

                        self.window.total_transfers += 1;
                        self.window.total_fees += price;
                    }
                }
                _ => {}
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第3轮：自由球员争夺战
    pub fn execute_round_3_free_agents(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 3;

        // 按能力值降序排序自由球员
        self.free_agent_pool.sort_by(|a, b| b.player.ability.cmp(&a.player.ability));

        let free_agents = std::mem::take(&mut self.free_agent_pool);

        for fa_info in free_agents {
            if fa_info.agent.status != FreeAgentStatus::Available {
                continue;
            }

            // 收集感兴趣的球队
            let mut interested_teams: Vec<(u64, u64, f64)> = Vec::new(); // (team_id, offer, score)

            for team in teams {
                let plan = match self.team_plans.get(&team.id) {
                    Some(p) => p,
                    None => continue,
                };

                // 检查是否可以签人
                if !plan.financial_status.can_buy() && !plan.must_sign {
                    continue;
                }

                // 检查阵容是否已满
                if plan.roster_count >= ROSTER_MAX {
                    continue;
                }

                // 检查位置需求
                let position_key = fa_info.player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default();
                let position_need = plan.position_needs.get(&position_key).copied().unwrap_or(0);

                if position_need < 30 && !plan.must_sign {
                    continue;
                }

                // 检查薪资空间
                if plan.salary_space < fa_info.minimum_salary as i64 {
                    continue;
                }

                // 计算报价
                let offer = self.calculate_salary_offer(plan, &fa_info);
                if offer < fa_info.minimum_salary {
                    continue;
                }

                // 计算吸引力评分
                let score = self.calculate_team_attractiveness(
                    team,
                    plan,
                    offer,
                    &fa_info,
                    players_by_team.get(&team.id).map(|v| v.as_slice()).unwrap_or(&[]),
                );

                interested_teams.push((team.id, offer, score));
            }

            if interested_teams.is_empty() {
                // 没人要，留在自由球员池
                self.free_agent_pool.push(fa_info);
                continue;
            }

            // 选择得分最高的球队
            interested_teams.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

            let (winner_team_id, final_salary, _) = interested_teams[0];
            let winner_team = teams.iter().find(|t| t.id == winner_team_id).unwrap();
            let winner_plan = self.team_plans.get(&winner_team_id).unwrap();

            // 确定合同年限
            let contract_years = calculate_contract_years(
                fa_info.player.age,
                fa_info.player.potential,
                winner_plan.strategy == TransferStrategy::AggressiveBuy,
            );

            // 创建事件
            let was_bidding_war = interested_teams.len() > 1;
            let competing_teams: Vec<u64> = interested_teams.iter().skip(1).take(3).map(|(id, _, _)| *id).collect();

            let event = self.create_free_agent_signing_event(
                round,
                &fa_info,
                winner_team,
                final_salary,
                contract_years,
                was_bidding_war,
                competing_teams,
            );
            events.push(event);

            // 更新统计
            self.window.free_agents_signed += 1;
            self.window.total_transfers += 1;

            // 更新球队计划
            if let Some(plan) = self.team_plans.get_mut(&winner_team_id) {
                plan.roster_count += 1;
                plan.current_total_salary += final_salary as i64;
                plan.salary_space -= final_salary as i64;
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第4轮：重建球队清洗
    ///
    /// 战绩差的球队清洗高薪老将，开始重建
    pub fn execute_round_4_rebuild(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 4;

        // 找出需要重建的球队
        let rebuilding_teams: Vec<_> = self.team_plans
            .iter()
            .filter(|(_, plan)| plan.strategy == TransferStrategy::FullRebuild)
            .map(|(id, _)| *id)
            .collect();

        for team_id in rebuilding_teams {
            let team = match teams.iter().find(|t| t.id == team_id) {
                Some(t) => t,
                None => continue,
            };

            let roster = match players_by_team.get(&team_id) {
                Some(r) => r,
                None => continue,
            };

            // 找出高薪老将（28岁以上，薪资100万以上）
            let mut veterans_to_sell: Vec<_> = roster
                .iter()
                .filter(|p| {
                    p.status == PlayerStatus::Active
                        && p.age >= 28
                        && p.salary >= 100
                })
                .collect();

            // 按薪资降序排序
            veterans_to_sell.sort_by(|a, b| b.salary.cmp(&a.salary));

            // 最多卖2个高薪老将
            for veteran in veterans_to_sell.into_iter().take(2) {
                let market_value = veteran.calculate_market_value();
                // 重建清洗打折出售
                let discount_price = (market_value as f64 * 0.75) as u64;

                // 寻找买家（争冠球队）
                if let Some(buyer_team) = self.find_contender_buyer(veteran, teams, discount_price) {
                    let event = self.create_rebuild_sale_event(
                        round,
                        veteran,
                        team,
                        &buyer_team,
                        discount_price,
                    );
                    events.push(event);

                    self.window.total_transfers += 1;
                    self.window.total_fees += discount_price;

                    // 更新计划
                    if let Some(sp) = self.team_plans.get_mut(&team_id) {
                        sp.roster_count -= 1;
                        sp.balance += discount_price as i64;
                        sp.current_total_salary -= veteran.salary as i64;
                    }
                    if let Some(bp) = self.team_plans.get_mut(&buyer_team.id) {
                        bp.roster_count += 1;
                        bp.balance -= discount_price as i64;
                    }
                }
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第5轮：财政清洗
    pub fn execute_round_5_financial_clearance(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 5;

        // 找出需要卖人的球队
        let teams_need_sell: Vec<_> = self.team_plans
            .iter()
            .filter(|(_, plan)| plan.strategy == TransferStrategy::MustSell || plan.strategy == TransferStrategy::ForceClear)
            .map(|(id, _)| *id)
            .collect();

        for team_id in teams_need_sell {
            let team = match teams.iter().find(|t| t.id == team_id) {
                Some(t) => t,
                None => continue,
            };

            let roster = match players_by_team.get(&team_id) {
                Some(r) => r,
                None => continue,
            };

            let plan = match self.team_plans.get(&team_id) {
                Some(p) => p.clone(),
                None => continue,
            };

            // 选择要卖的球员（低能力高薪资优先）
            let mut sellable: Vec<_> = roster
                .iter()
                .filter(|p| p.status == PlayerStatus::Active && !p.is_starter)
                .collect();

            sellable.sort_by(|a, b| {
                // 优先卖：能力低、薪资高
                let score_a = a.salary as i32 - a.ability as i32 * 5;
                let score_b = b.salary as i32 - b.ability as i32 * 5;
                score_b.cmp(&score_a)
            });

            // 卖1-2个人
            let sell_count = if plan.strategy == TransferStrategy::ForceClear {
                (plan.roster_count - ROSTER_MAX + 1).min(2)
            } else {
                1
            };

            for player in sellable.into_iter().take(sell_count as usize) {
                // 找买家
                let buyer = self.find_buyer_for_clearance(player, teams, &plan);

                if let Some((buyer_team, offer)) = buyer {
                    let event = self.create_clearance_sale_event(
                        round,
                        player,
                        team,
                        &buyer_team,
                        offer,
                    );
                    events.push(event);

                    self.window.total_transfers += 1;
                    self.window.total_fees += offer;

                    // 更新双方计划
                    if let Some(seller_plan) = self.team_plans.get_mut(&team_id) {
                        seller_plan.roster_count -= 1;
                        seller_plan.balance += offer as i64;
                    }
                    if let Some(buyer_plan) = self.team_plans.get_mut(&buyer_team.id) {
                        buyer_plan.roster_count += 1;
                        buyer_plan.balance -= offer as i64;
                        buyer_plan.transfer_budget -= offer as i64;
                    }
                }
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第6轮：强队补强
    pub fn execute_round_6_reinforcement(
        &mut self,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 6;

        // 找出想买人的球队
        let teams_want_buy: Vec<_> = self.team_plans
            .iter()
            .filter(|(_, plan)| {
                plan.strategy == TransferStrategy::AggressiveBuy
                && plan.transfer_budget > 100  // 至少100万预算
            })
            .map(|(id, plan)| (*id, plan.clone()))
            .collect();

        for (buyer_team_id, buyer_plan) in teams_want_buy {
            let buyer_team = match teams.iter().find(|t| t.id == buyer_team_id) {
                Some(t) => t,
                None => continue,
            };

            // 找最需要的位置
            let mut needs: Vec<_> = buyer_plan.position_needs.iter().collect();
            needs.sort_by(|a, b| b.1.cmp(a.1));

            for (position, need) in needs.into_iter().take(2) {
                if *need < 50 {
                    continue;
                }

                // 在其他球队找合适的球员
                let target = self.find_transfer_target(
                    position,
                    &buyer_plan,
                    teams,
                    players_by_team,
                );

                if let Some((player, seller_team, price)) = target {
                    // 检查卖家是否愿意卖
                    if !self.seller_accepts(&player, &seller_team, price) {
                        continue;
                    }

                    // 薪资谈判
                    let market_value = player.calculate_market_value();
                    let salary_exp = calculate_expected_salary(market_value);
                    let new_salary = salary_exp.expected;
                    let contract_years = calculate_contract_years(
                        player.age,
                        player.potential,
                        true,
                    );

                    let event = self.create_purchase_event(
                        round,
                        &player,
                        &seller_team,
                        buyer_team,
                        price,
                        new_salary,
                        contract_years,
                    );
                    events.push(event);

                    self.window.total_transfers += 1;
                    self.window.total_fees += price;

                    // 更新计划
                    if let Some(bp) = self.team_plans.get_mut(&buyer_team_id) {
                        bp.roster_count += 1;
                        bp.transfer_budget -= price as i64;
                        bp.balance -= price as i64;
                    }
                    if let Some(sp) = self.team_plans.get_mut(&seller_team.id) {
                        sp.roster_count -= 1;
                        sp.balance += price as i64;
                    }

                    break; // 每队每轮最多买一人
                }
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 执行第7轮：收尾补救
    pub fn execute_round_7_finalize(
        &mut self,
        teams: &[Team],
    ) -> Vec<TransferEvent> {
        let mut events = Vec::new();
        let round = 7;

        // 检查阵容不足5人的球队
        let teams_need_fill: Vec<_> = self.team_plans
            .iter()
            .filter(|(_, plan)| plan.roster_count < ROSTER_MIN)
            .map(|(id, plan)| (*id, plan.roster_count))
            .collect();

        for (team_id, current_count) in teams_need_fill {
            let team = match teams.iter().find(|t| t.id == team_id) {
                Some(t) => t,
                None => continue,
            };

            let needed = ROSTER_MIN - current_count;

            // 从剩余自由球员中签人
            for _ in 0..needed {
                if let Some(fa_info) = self.free_agent_pool.pop() {
                    let event = self.create_free_agent_signing_event(
                        round,
                        &fa_info,
                        team,
                        fa_info.minimum_salary,
                        2,
                        false,
                        vec![],
                    );
                    events.push(event);

                    self.window.free_agents_signed += 1;
                    self.window.total_transfers += 1;

                    if let Some(plan) = self.team_plans.get_mut(&team_id) {
                        plan.roster_count += 1;
                    }
                }
                // TODO: 如果自由球员池空了，生成临时选手
            }
        }

        self.events.extend(events.clone());
        events
    }

    /// 完成转会窗口
    pub fn finalize(&mut self) -> TransferWindow {
        self.window.status = TransferWindowStatus::Completed;
        self.window.completed_at = Some(chrono::Utc::now().to_rfc3339());
        self.window.clone()
    }

    // ==================== 辅助方法 ====================

    /// 计算薪资报价
    fn calculate_salary_offer(&self, plan: &TeamTransferPlan, fa_info: &FreeAgentInfo) -> u64 {
        let position_key = fa_info.player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();
        let position_need = plan.position_needs.get(&position_key).copied().unwrap_or(0);

        let base_ratio = match (plan.financial_status, position_need) {
            (FinancialStatus::Wealthy, n) if n >= 80 => 1.15,
            (FinancialStatus::Wealthy, _) => 1.0,
            (FinancialStatus::Healthy, n) if n >= 70 => 1.0,
            (FinancialStatus::Healthy, _) => 0.9,
            (FinancialStatus::Tight, _) => 0.8,
            _ => 0.75,
        };

        let offer = (fa_info.expected_salary as f64 * base_ratio) as u64;
        offer.min(plan.salary_space as u64).max(fa_info.minimum_salary)
    }

    /// 计算球队吸引力
    fn calculate_team_attractiveness(
        &self,
        _team: &Team,
        plan: &TeamTransferPlan,
        offer: u64,
        fa_info: &FreeAgentInfo,
        roster: &[Player],
    ) -> f64 {
        let mut rng = rand::thread_rng();

        // 薪资评分
        let salary_score = (offer as f64 / fa_info.expected_salary as f64).min(1.2);

        // 球队实力评分
        let strength_score = plan.avg_ability / 100.0;

        // 上场机会评分
        let same_position_count = roster
            .iter()
            .filter(|p| p.position == fa_info.player.position && p.status == PlayerStatus::Active)
            .count();
        let opportunity_score = match same_position_count {
            0 => 1.0,
            1 => if fa_info.player.ability > roster.iter()
                .filter(|p| p.position == fa_info.player.position)
                .map(|p| p.ability)
                .max()
                .unwrap_or(0) { 0.9 } else { 0.5 },
            _ => 0.3,
        };

        // 随机因素
        let random_score: f64 = rng.gen();

        salary_score * self.config.fa_salary_weight
            + strength_score * self.config.fa_strength_weight
            + opportunity_score * self.config.fa_opportunity_weight
            + random_score * self.config.fa_random_weight
    }

    /// 为清洗找买家
    fn find_buyer_for_clearance(
        &self,
        player: &Player,
        teams: &[Team],
        _seller_plan: &TeamTransferPlan,
    ) -> Option<(Team, u64)> {
        let market_value = player.calculate_market_value();
        let discount_price = (market_value as f64 * 0.7) as u64;

        let position_key = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        for team in teams {
            if team.id == player.team_id.unwrap_or(0) {
                continue;
            }

            let plan = match self.team_plans.get(&team.id) {
                Some(p) => p,
                None => continue,
            };

            if !plan.financial_status.can_buy() {
                continue;
            }

            if plan.roster_count >= ROSTER_MAX {
                continue;
            }

            let position_need = plan.position_needs.get(&position_key).copied().unwrap_or(0);
            if position_need < 30 {
                continue;
            }

            if plan.transfer_budget >= discount_price as i64 {
                return Some((team.clone(), discount_price));
            }
        }

        None
    }

    /// 找转会目标
    fn find_transfer_target(
        &self,
        position: &str,
        buyer_plan: &TeamTransferPlan,
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
    ) -> Option<(Player, Team, u64)> {
        let mut candidates: Vec<(Player, Team, u64)> = Vec::new();

        for team in teams {
            if team.id == buyer_plan.team_id {
                continue;
            }

            let roster = match players_by_team.get(&team.id) {
                Some(r) => r,
                None => continue,
            };

            for player in roster {
                if player.status != PlayerStatus::Active {
                    continue;
                }

                let player_position = player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default();

                if player_position != position {
                    continue;
                }

                // 计算价格
                let market_value = player.calculate_market_value();
                let asking_price = (market_value as f64 * 1.2) as u64; // 溢价20%

                if asking_price as i64 > buyer_plan.transfer_budget {
                    continue;
                }

                candidates.push((player.clone(), team.clone(), asking_price));
            }
        }

        // 按能力值排序，选最好的
        candidates.sort_by(|a, b| b.0.ability.cmp(&a.0.ability));
        candidates.into_iter().next()
    }

    /// 判断卖家是否接受
    fn seller_accepts(&self, player: &Player, _team: &Team, offer: u64) -> bool {
        let market_value = player.calculate_market_value();

        // 报价至少是身价的90%
        if offer < (market_value as f64 * 0.9) as u64 {
            return false;
        }

        // 非首发更容易被卖
        if !player.is_starter && offer >= (market_value as f64 * 0.8) as u64 {
            return true;
        }

        // 高溢价接受
        if offer >= (market_value as f64 * 1.3) as u64 {
            return true;
        }

        // 老将更容易被卖
        if player.age >= 28 && offer >= market_value {
            return true;
        }

        false
    }

    /// 为追求冠军的明星选手寻找强队
    fn find_contender_for_star(&self, player: &Player, teams: &[Team]) -> Option<(Team, u64)> {
        let market_value = player.calculate_market_value();
        let position_key = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        // 寻找争冠球队（强队补强或追逐巨星策略）
        let mut candidates: Vec<(Team, u64)> = Vec::new();

        for team in teams {
            // 不能卖给自己的队
            if team.id == player.team_id.unwrap_or(0) {
                continue;
            }

            let plan = match self.team_plans.get(&team.id) {
                Some(p) => p,
                None => continue,
            };

            // 寻找有钱的强队
            if !plan.financial_status.can_buy() {
                continue;
            }

            // 需要是争冠球队或追逐巨星的球队
            if plan.ambition != AmbitionLevel::Championship
                && plan.strategy != TransferStrategy::StarHunting
            {
                continue;
            }

            // 检查位置需求
            let position_need = plan.position_needs.get(&position_key).copied().unwrap_or(0);
            if position_need < 30 {
                continue;
            }

            // 检查预算
            let offer = (market_value as f64 * 1.2) as u64; // 溢价20%
            if plan.transfer_budget < offer as i64 {
                continue;
            }

            candidates.push((team.clone(), offer));
        }

        // 返回出价最高的球队
        candidates.into_iter().max_by_key(|(_, offer)| *offer)
    }

    /// 为寻找上场机会的选手找球队
    fn find_team_needing_position(&self, player: &Player, teams: &[Team]) -> Option<(Team, u64)> {
        use rand::seq::SliceRandom;

        let market_value = player.calculate_market_value();
        let position_key = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        let mut candidates: Vec<(Team, u64, u32)> = Vec::new();

        for team in teams {
            if team.id == player.team_id.unwrap_or(0) {
                continue;
            }

            let plan = match self.team_plans.get(&team.id) {
                Some(p) => p,
                None => continue,
            };

            if !plan.financial_status.can_buy() {
                continue;
            }

            // 寻找急需该位置的球队
            let position_need = plan.position_needs.get(&position_key).copied().unwrap_or(0);
            if position_need < 70 {
                continue;
            }

            let offer = market_value;
            if plan.transfer_budget >= offer as i64 {
                candidates.push((team.clone(), offer, position_need));
            }
        }

        // 随机选择一个候选球队（优先需求度高的）
        if candidates.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        // 按需求度排序后随机选择前几个
        candidates.sort_by(|a, b| b.2.cmp(&a.2));
        let top_candidates: Vec<_> = candidates.into_iter().take(3).collect();
        top_candidates.choose(&mut rng).map(|(t, o, _)| (t.clone(), *o))
    }

    /// 为重建清洗找争冠球队买家
    fn find_contender_buyer(&self, player: &Player, teams: &[Team], price: u64) -> Option<Team> {
        let position_key = player.position
            .map(|p| format!("{:?}", p).to_uppercase())
            .unwrap_or_default();

        for team in teams {
            if team.id == player.team_id.unwrap_or(0) {
                continue;
            }

            let plan = match self.team_plans.get(&team.id) {
                Some(p) => p,
                None => continue,
            };

            // 争冠球队才会买老将
            if plan.ambition != AmbitionLevel::Championship {
                continue;
            }

            if !plan.financial_status.can_buy() {
                continue;
            }

            // 检查位置需求
            let position_need = plan.position_needs.get(&position_key).copied().unwrap_or(0);
            if position_need < 50 {
                continue;
            }

            if plan.transfer_budget >= price as i64 {
                return Some(team.clone());
            }
        }

        None
    }

    // ==================== 事件创建方法 ====================

    fn create_retirement_event(&self, round: u32, player: &Player, team: Option<&Team>) -> TransferEvent {
        let importance = NewsImportance::from_ability(player.ability);
        let headline = if player.ability >= 85 {
            format!("传奇落幕！{} 宣布退役", player.game_id)
        } else if player.ability >= 75 {
            format!("{} 正式宣布退役", player.game_id)
        } else {
            format!("{} 结束职业生涯", player.game_id)
        };

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::Retirement,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value: player.calculate_market_value(),
            from_team_id: team.map(|t| t.id),
            from_team_name: team.map(|t| t.name.clone()),
            to_team_id: None,
            to_team_name: None,
            transfer_fee: 0,
            new_salary: None,
            contract_years: None,
            contract_type: ContractType::Standard,
            price_ratio: None,
            headline,
            description: format!("{} 结束了自己的职业生涯", player.game_id),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    fn create_contract_expire_event(&self, round: u32, player: &Player, team: Option<&Team>) -> TransferEvent {
        let importance = NewsImportance::from_ability(player.ability);
        let headline = format!("{} 合同到期，成为自由球员", player.game_id);

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::ContractExpire,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value: player.calculate_market_value(),
            from_team_id: team.map(|t| t.id),
            from_team_name: team.map(|t| t.name.clone()),
            to_team_id: None,
            to_team_name: None,
            transfer_fee: 0,
            new_salary: None,
            contract_years: None,
            contract_type: ContractType::Standard,
            price_ratio: None,
            headline,
            description: format!("{} 与 {} 的合同到期", player.game_id, team.map(|t| t.name.as_str()).unwrap_or("球队")),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    fn create_free_agent_signing_event(
        &self,
        round: u32,
        fa_info: &FreeAgentInfo,
        team: &Team,
        salary: u64,
        contract_years: u32,
        was_bidding_war: bool,
        competing_teams: Vec<u64>,
    ) -> TransferEvent {
        let importance = NewsImportance::from_ability(fa_info.player.ability);

        let headline = if was_bidding_war {
            format!("多队争抢！{} 最终加盟 {}", fa_info.player.game_id, team.name)
        } else {
            format!("{} 签约 {}", fa_info.player.game_id, team.name)
        };

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::FreeAgent,
            status: TransferEventStatus::Completed,
            player_id: fa_info.player.id,
            player_name: fa_info.player.game_id.clone(),
            position: fa_info.player.position.map(|p| format!("{:?}", p)),
            age: fa_info.player.age,
            ability: fa_info.player.ability,
            potential: fa_info.player.potential,
            market_value: fa_info.market_value,
            from_team_id: None,
            from_team_name: None,
            to_team_id: Some(team.id),
            to_team_name: Some(team.name.clone()),
            transfer_fee: 0,
            new_salary: Some(salary),
            contract_years: Some(contract_years),
            contract_type: ContractType::Standard,
            price_ratio: None,
            headline,
            description: format!(
                "自由球员 {} 与 {} 签下{}年合同，年薪{}万",
                fa_info.player.game_id, team.name, contract_years, salary / 10000
            ),
            importance,
            competing_teams,
            was_bidding_war,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    fn create_clearance_sale_event(
        &self,
        round: u32,
        player: &Player,
        seller: &Team,
        buyer: &Team,
        price: u64,
    ) -> TransferEvent {
        let market_value = player.calculate_market_value();
        let price_ratio = price as f64 / market_value as f64;
        let importance = NewsImportance::from_ability(player.ability);

        let headline = format!(
            "{} 低价出售 {}，{} 接手",
            seller.name, player.game_id, buyer.name
        );

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::Purchase,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value,
            from_team_id: Some(seller.id),
            from_team_name: Some(seller.name.clone()),
            to_team_id: Some(buyer.id),
            to_team_name: Some(buyer.name.clone()),
            transfer_fee: price,
            new_salary: Some(player.salary),
            contract_years: Some(2),
            contract_type: ContractType::Standard,
            price_ratio: Some(price_ratio),
            headline,
            description: format!(
                "{} 以{}万（身价{}折）从 {} 转会至 {}",
                player.game_id,
                price / 10000,
                (price_ratio * 100.0) as u32,
                seller.name,
                buyer.name
            ),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    fn create_purchase_event(
        &self,
        round: u32,
        player: &Player,
        seller: &Team,
        buyer: &Team,
        price: u64,
        salary: u64,
        contract_years: u32,
    ) -> TransferEvent {
        let market_value = player.calculate_market_value();
        let price_ratio = price as f64 / market_value as f64;
        let importance = NewsImportance::from_ability(player.ability);

        let headline = if player.ability >= 85 {
            format!("重磅！{} 以{}万加盟 {}", player.game_id, price / 10000, buyer.name)
        } else {
            format!("{} 转会 {}，转会费{}万", player.game_id, buyer.name, price / 10000)
        };

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::Purchase,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value,
            from_team_id: Some(seller.id),
            from_team_name: Some(seller.name.clone()),
            to_team_id: Some(buyer.id),
            to_team_name: Some(buyer.name.clone()),
            transfer_fee: price,
            new_salary: Some(salary),
            contract_years: Some(contract_years),
            contract_type: ContractType::Standard,
            price_ratio: Some(price_ratio),
            headline,
            description: format!(
                "{} 从 {} 转会至 {}，转会费{}万，{}年合同，年薪{}万",
                player.game_id,
                seller.name,
                buyer.name,
                price / 10000,
                contract_years,
                salary / 10000
            ),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// 创建忠诚留队事件
    fn create_loyalty_stay_event(
        &self,
        round: u32,
        player: &Player,
        team: &Team,
    ) -> TransferEvent {
        let importance = NewsImportance::from_ability(player.ability);
        let headline = format!(
            "忠诚！{} 拒绝离队，选择留在 {}",
            player.game_id, team.name
        );

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::LoyaltyStay,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value: player.calculate_market_value(),
            from_team_id: Some(team.id),
            from_team_name: Some(team.name.clone()),
            to_team_id: Some(team.id),
            to_team_name: Some(team.name.clone()),
            transfer_fee: 0,
            new_salary: None,
            contract_years: None,
            contract_type: ContractType::Standard,
            price_ratio: None,
            headline,
            description: format!(
                "尽管有离队意愿，{} 出于对球队的忠诚选择继续留在 {}",
                player.game_id, team.name
            ),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// 创建申请转会事件
    fn create_transfer_request_event(
        &self,
        round: u32,
        player: &Player,
        from_team: &Team,
        to_team: &Team,
        price: u64,
    ) -> TransferEvent {
        let market_value = player.calculate_market_value();
        let price_ratio = price as f64 / market_value as f64;
        let importance = NewsImportance::from_ability(player.ability);

        let headline = format!(
            "{} 申请转会成功！以{}万加盟 {}",
            player.game_id, price / 10000, to_team.name
        );

        let salary_exp = calculate_expected_salary(market_value);
        let new_salary = salary_exp.expected;
        let contract_years = calculate_contract_years(player.age, player.potential, true);

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::TransferRequest,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value,
            from_team_id: Some(from_team.id),
            from_team_name: Some(from_team.name.clone()),
            to_team_id: Some(to_team.id),
            to_team_name: Some(to_team.name.clone()),
            transfer_fee: price,
            new_salary: Some(new_salary),
            contract_years: Some(contract_years),
            contract_type: ContractType::Standard,
            price_ratio: Some(price_ratio),
            headline,
            description: format!(
                "{} 主动申请离开 {}，以{}万转会费加盟 {}",
                player.game_id,
                from_team.name,
                price / 10000,
                to_team.name
            ),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// 创建重建出售事件
    fn create_rebuild_sale_event(
        &self,
        round: u32,
        player: &Player,
        seller: &Team,
        buyer: &Team,
        price: u64,
    ) -> TransferEvent {
        let market_value = player.calculate_market_value();
        let price_ratio = price as f64 / market_value as f64;
        let importance = NewsImportance::from_ability(player.ability);

        let headline = format!(
            "{} 重建！{} 以{}万出售 {}",
            seller.name, player.game_id, price / 10000, buyer.name
        );

        let salary_exp = calculate_expected_salary(market_value);
        let new_salary = salary_exp.expected;

        TransferEvent {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            event_type: TransferEventType::RebuildSale,
            status: TransferEventStatus::Completed,
            player_id: player.id,
            player_name: player.game_id.clone(),
            position: player.position.map(|p| format!("{:?}", p)),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            market_value,
            from_team_id: Some(seller.id),
            from_team_name: Some(seller.name.clone()),
            to_team_id: Some(buyer.id),
            to_team_name: Some(buyer.name.clone()),
            transfer_fee: price,
            new_salary: Some(new_salary),
            contract_years: Some(2),
            contract_type: ContractType::Standard,
            price_ratio: Some(price_ratio),
            headline,
            description: format!(
                "{} 因战绩不佳开始重建，以{}万（身价{}%）将 {} 出售给 {}",
                seller.name,
                price / 10000,
                (price_ratio * 100.0) as u32,
                player.game_id,
                buyer.name
            ),
            importance,
            competing_teams: vec![],
            was_bidding_war: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// 生成轮次摘要
    pub fn generate_round_summary(&self, round: u32) -> TransferRoundSummary {
        let round_events: Vec<_> = self.events.iter().filter(|e| e.round == round).collect();

        let events_count = round_events.len() as u32;
        let transfers_count = round_events
            .iter()
            .filter(|e| matches!(
                e.event_type,
                TransferEventType::FreeAgent
                    | TransferEventType::Purchase
                    | TransferEventType::TransferRequest
                    | TransferEventType::RebuildSale
            ))
            .count() as u32;
        let total_fees: u64 = round_events.iter().map(|e| e.transfer_fee).sum();

        let summary = match round {
            0 => format!(
                "赛季结算完成，{}名选手计算了满意度和忠诚度变化，{}名选手表达了离队意愿",
                self.player_statuses.len(),
                self.departure_candidates.len()
            ),
            1 => format!(
                "本轮共有{}名选手合同到期，{}名选手宣布退役",
                self.window.contract_expires,
                self.window.retirements
            ),
            2 => {
                let transfer_requests = round_events
                    .iter()
                    .filter(|e| e.event_type == TransferEventType::TransferRequest)
                    .count();
                let loyalty_stays = round_events
                    .iter()
                    .filter(|e| e.event_type == TransferEventType::LoyaltyStay)
                    .count();
                format!(
                    "选手意愿处理完成，{}名选手申请转会成功，{}名选手因忠诚选择留队",
                    transfer_requests, loyalty_stays
                )
            }
            3 => format!(
                "自由球员争夺战结束！本轮共签约{}名自由球员",
                round_events.iter().filter(|e| e.event_type == TransferEventType::FreeAgent).count()
            ),
            4 => {
                let rebuild_sales = round_events
                    .iter()
                    .filter(|e| e.event_type == TransferEventType::RebuildSale)
                    .count();
                format!(
                    "重建清洗阶段完成，{}支球队开始重建，共出售{}名老将，总金额{}万",
                    self.team_plans.values().filter(|p| p.strategy == TransferStrategy::FullRebuild).count(),
                    rebuild_sales,
                    total_fees / 10000
                )
            }
            5 => format!(
                "财政清洗阶段完成，本轮发生{}笔交易，总金额{}万",
                transfers_count, total_fees / 10000
            ),
            6 => format!(
                "强队补强阶段完成，本轮发生{}笔重磅转会，总金额{}万",
                transfers_count, total_fees / 10000
            ),
            7 => format!("转会窗口收尾完成，所有球队阵容已补齐"),
            _ => String::new(),
        };

        TransferRoundSummary {
            id: 0,
            save_id: self.window.save_id.clone(),
            season_id: self.window.season_id,
            round,
            round_name: get_round_name(round).to_string(),
            events_count,
            transfers_count,
            total_fees,
            summary,
            started_at: None,
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

// ==================== 旧版兼容 ====================

/// 转会AI引擎（保留旧版接口）
pub struct TransferEngine {
    /// 转会市场列表
    transfer_market: Vec<TransferListing>,
    /// 自由球员市场
    free_agents: Vec<FreeAgent>,
    /// AI决策参数
    config: TransferAIConfig,
}

/// AI转会决策配置
#[derive(Debug, Clone)]
pub struct TransferAIConfig {
    pub ability_weight: f64,
    pub potential_weight: f64,
    pub age_weight: f64,
    pub position_need_weight: f64,
    pub budget_ratio: f64,
}

impl Default for TransferAIConfig {
    fn default() -> Self {
        Self {
            ability_weight: 0.4,
            potential_weight: 0.3,
            age_weight: 0.15,
            position_need_weight: 0.15,
            budget_ratio: 0.6,  // 与 TransferEngineConfig 保持一致
        }
    }
}

impl Default for TransferEngine {
    fn default() -> Self {
        Self {
            transfer_market: Vec::new(),
            free_agents: Vec::new(),
            config: TransferAIConfig::default(),
        }
    }
}

impl TransferEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn import_market(&mut self, listings: Vec<TransferListing>) {
        self.transfer_market = listings;
    }

    pub fn import_free_agents(&mut self, agents: Vec<FreeAgent>) {
        self.free_agents = agents;
    }

    pub fn get_market(&self) -> &[TransferListing] {
        &self.transfer_market
    }

    pub fn get_free_agents(&self) -> &[FreeAgent] {
        &self.free_agents
    }

    /// 计算球员市场价值（使用 Player 的方法）
    pub fn calculate_market_value(player: &Player) -> u64 {
        player.calculate_market_value()
    }
}

/// 转会决策
#[derive(Debug, Clone)]
pub enum TransferDecision {
    Offer(u64),
    Negotiate(u64),
    Reject(TransferRejectReason),
}

/// 拒绝转会原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferRejectReason {
    InsufficientBudget,
    OverPriced,
    NoPositionNeed,
    LowValue,
}

/// 自由球员签约决策
#[derive(Debug, Clone)]
pub enum FreeAgentDecision {
    Offer { salary: u64, contract_years: u32 },
    Reject(FreeAgentRejectReason),
}

/// 拒绝签约原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FreeAgentRejectReason {
    SalaryTooHigh,
    NoPositionNeed,
    LowValue,
}

/// 转会行为
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferAction {
    Buy {
        buyer_team_id: u64,
        listing_id: u64,
        offer_amount: u64,
    },
    SignFreeAgent {
        team_id: u64,
        agent_id: u64,
        salary_offer: u64,
        contract_years: u32,
    },
    List {
        team_id: u64,
        player_id: u64,
        asking_price: u64,
    },
}
