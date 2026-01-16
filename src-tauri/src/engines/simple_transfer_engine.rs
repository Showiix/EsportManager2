//! 简化版转会系统规则引擎
//!
//! 实现4阶段转会流程的核心逻辑

use crate::models::{
    simple_transfer::*,
    Player, Team, PlayerStatus,
};
use rusqlite::Connection;
use std::collections::HashMap;
use rand::Rng;

/// 简化版转会引擎
pub struct SimpleTransferEngine {
    state: TransferMarketState,
    players: HashMap<u64, PlayerTransferInfo>,
    teams: HashMap<u64, TeamTransferStrategy>,
    offers: Vec<TransferOffer>,
    events: Vec<TransferEvent>,
    next_offer_id: u64,
    next_event_id: u64,
}

impl SimpleTransferEngine {
    pub fn new(save_id: String, season_id: u64) -> Self {
        Self {
            state: TransferMarketState::new(save_id, season_id),
            players: HashMap::new(),
            teams: HashMap::new(),
            offers: Vec::new(),
            events: Vec::new(),
            next_offer_id: 1,
            next_event_id: 1,
        }
    }

    pub fn state(&self) -> &TransferMarketState {
        &self.state
    }

    pub fn summary(&self) -> TransferMarketSummary {
        TransferMarketSummary::from(&self.state)
    }

    pub fn players(&self) -> Vec<&PlayerTransferInfo> {
        self.players.values().collect()
    }

    pub fn teams(&self) -> Vec<&TeamTransferStrategy> {
        self.teams.values().collect()
    }

    pub fn events(&self) -> &[TransferEvent] {
        &self.events
    }

    pub fn get_player(&self, player_id: u64) -> Option<&PlayerTransferInfo> {
        self.players.get(&player_id)
    }

    pub fn get_team(&self, team_id: u64) -> Option<&TeamTransferStrategy> {
        self.teams.get(&team_id)
    }

    // ==================== 阶段1: 市场分析 ====================

    /// 执行市场分析阶段
    pub fn execute_market_analysis(&mut self, conn: &Connection) -> Result<(), String> {
        if self.state.phase != TransferPhase::MarketAnalysis {
            return Err("当前不在市场分析阶段".to_string());
        }

        // 加载所有活跃选手
        let players = self.load_players(conn)?;
        let current_season = self.state.season_id;

        let mut free_agents = 0;
        let mut willing_to_transfer = 0;

        for player in players {
            let contract_expired = player.contract_end_season
                .map(|end| end <= current_season)
                .unwrap_or(true);

            // 计算转会意愿
            let intent = TransferIntent::from_satisfaction_loyalty(
                player.satisfaction,
                player.loyalty,
                contract_expired,
            );

            // 计算离队原因
            let departure_reason = self.calculate_departure_reason(&player, contract_expired);

            // 确定市场状态
            let status = if contract_expired || player.team_id.is_none() {
                free_agents += 1;
                PlayerMarketStatus::FreeAgent
            } else if intent.accepts_offers() {
                willing_to_transfer += 1;
                PlayerMarketStatus::WillingToTransfer
            } else {
                PlayerMarketStatus::Contracted
            };

            // 计算期望薪资
            let expected_salary = self.calculate_expected_salary(&player);

            let info = PlayerTransferInfo {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position.map(|p| format!("{:?}", p)).unwrap_or_default(),
                age: player.age,
                ability: player.ability,
                potential: player.potential,
                team_id: player.team_id,
                team_name: None, // 后续填充
                salary: player.salary,
                market_value: player.calculate_market_value(),
                contract_end_season: player.contract_end_season,
                satisfaction: player.satisfaction,
                loyalty: player.loyalty,
                intent,
                departure_reason,
                status,
                expected_salary,
            };

            self.players.insert(player.id, info);
        }

        // 更新状态
        self.state.free_agents_count = free_agents;
        self.state.willing_to_transfer_count = willing_to_transfer;
        self.state.is_initialized = true;

        // 添加事件
        self.add_event(TransferEventType::PhaseCompleted, None, None, None,
            "市场分析完成".to_string(),
            format!("识别{}名自由球员，{}名可转会选手", free_agents, willing_to_transfer),
        );

        // 推进到下一阶段
        self.state.phase = TransferPhase::StrategyGeneration;

        Ok(())
    }

    fn load_players(&self, conn: &Connection) -> Result<Vec<Player>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, game_id, real_name, team_id, position, nationality, age,
                    ability, potential, stability, salary, market_value,
                    contract_end_season, status, tag, is_starter, satisfaction, loyalty
             FROM players
             WHERE status = 'Active' AND save_id = ?"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&self.state.save_id], |row| {
            Ok(Player {
                id: row.get(0)?,
                game_id: row.get(1)?,
                real_name: row.get(2)?,
                team_id: row.get(3)?,
                position: row.get::<_, Option<String>>(4)?
                    .and_then(|s| match s.to_uppercase().as_str() {
                        "TOP" => Some(crate::models::player::Position::Top),
                        "JUG" | "JUNGLE" => Some(crate::models::player::Position::Jug),
                        "MID" => Some(crate::models::player::Position::Mid),
                        "ADC" | "BOT" => Some(crate::models::player::Position::Adc),
                        "SUP" | "SUPPORT" => Some(crate::models::player::Position::Sup),
                        _ => None,
                    }),
                nationality: row.get(5)?,
                age: row.get(6)?,
                ability: row.get(7)?,
                potential: row.get(8)?,
                stability: row.get(9)?,
                salary: row.get(10)?,
                market_value: row.get(11)?,
                contract_end_season: row.get(12)?,
                status: PlayerStatus::Active,
                tag: row.get(14)?,
                is_starter: row.get::<_, i32>(15)? != 0,
                satisfaction: row.get::<_, Option<u8>>(16)?.unwrap_or(70),
                loyalty: row.get::<_, Option<u8>>(17)?.unwrap_or(50),
                starter: row.get::<_, i32>(15)? != 0,
                save_id: self.state.save_id.clone(),
            })
        }).map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn calculate_departure_reason(&self, player: &Player, contract_expired: bool) -> DepartureReason {
        if contract_expired {
            return DepartureReason::ContractExpired;
        }

        // 缺少上场时间：替补且能力≥70
        if !player.is_starter && player.ability >= 70 && player.satisfaction < 60 {
            return DepartureReason::LackOfPlaytime;
        }

        // 追求冠军：老将+能力强
        if player.age >= 26 && player.ability >= 85 && player.satisfaction < 50 {
            return DepartureReason::SeekingChampionship;
        }

        // 寻找机会：年轻替补
        if player.age <= 22 && !player.is_starter && player.satisfaction < 60 {
            return DepartureReason::SeekingOpportunity;
        }

        // 薪资不满
        if player.satisfaction < 40 {
            return DepartureReason::SalaryDissatisfaction;
        }

        DepartureReason::None
    }

    fn calculate_expected_salary(&self, player: &Player) -> u64 {
        // 基于能力和市场价值计算期望薪资
        let base = player.calculate_market_value() / 100; // 市值的1%作为年薪基础
        let ability_factor = (player.ability as f64 / 80.0).max(0.8).min(1.5);
        let age_factor = if player.age <= 24 {
            1.1  // 年轻有潜力
        } else if player.age >= 28 {
            0.9  // 老将降薪
        } else {
            1.0
        };

        ((base as f64 * ability_factor * age_factor) as u64).max(50)  // 最低50万
    }

    // ==================== 阶段2: 策略生成 ====================

    /// 执行策略生成阶段
    pub fn execute_strategy_generation(&mut self, conn: &Connection) -> Result<(), String> {
        if self.state.phase != TransferPhase::StrategyGeneration {
            return Err("当前不在策略生成阶段".to_string());
        }

        // 加载所有球队
        let teams = self.load_teams(conn)?;

        for team in teams {
            let strategy = self.generate_team_strategy(&team, conn)?;
            self.teams.insert(team.id, strategy);
        }

        // 添加事件
        self.add_event(TransferEventType::PhaseCompleted, None, None, None,
            "策略生成完成".to_string(),
            format!("为{}支球队生成转会策略", self.teams.len()),
        );

        // 推进到下一阶段
        self.state.phase = TransferPhase::RenewalWindow;

        Ok(())
    }

    fn load_teams(&self, conn: &Connection) -> Result<Vec<Team>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, region_id, name, short_name, power_rating,
                    total_matches, wins, win_rate, annual_points,
                    cross_year_points, balance
             FROM teams
             WHERE save_id = ?"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([&self.state.save_id], |row| {
            Ok(Team {
                id: row.get(0)?,
                region_id: row.get(1)?,
                name: row.get(2)?,
                short_name: row.get(3)?,
                power_rating: row.get(4)?,
                total_matches: row.get(5)?,
                wins: row.get(6)?,
                win_rate: row.get(7)?,
                annual_points: row.get(8)?,
                cross_year_points: row.get(9)?,
                balance: row.get(10)?,
                save_id: self.state.save_id.clone(),
            })
        }).map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn generate_team_strategy(&self, team: &Team, conn: &Connection) -> Result<TeamTransferStrategy, String> {
        // 获取球队阵容
        let roster: Vec<&PlayerTransferInfo> = self.players.values()
            .filter(|p| p.team_id == Some(team.id))
            .collect();

        // 计算平均能力
        let avg_ability = if roster.is_empty() {
            0.0
        } else {
            roster.iter().map(|p| p.ability as f64).sum::<f64>() / roster.len() as f64
        };

        // 确定策略类型
        let strategy_type = self.determine_strategy_type(avg_ability, team.balance, &roster);

        // 计算位置需求
        let position_needs = self.calculate_position_needs(&roster);

        // 计算预算
        let budget = self.calculate_transfer_budget(team, &strategy_type);
        let salary_cap = 5000_0000; // 5000万年薪上限
        let current_salary: u64 = roster.iter().map(|p| p.salary).sum();
        let salary_cap_space = salary_cap as i64 - current_salary as i64;

        // 筛选引援目标
        let targets = self.find_transfer_targets(team, &strategy_type, &position_needs, budget);

        // 筛选出售候选
        let sell_candidates = self.find_sell_candidates(team, &roster, &strategy_type);

        // 生成策略总结
        let summary = format!(
            "{}型策略：平均能力{:.1}，预算{}万，需补强{}，可出售{}人",
            strategy_type.name(),
            avg_ability,
            budget / 10000,
            position_needs.iter().filter(|n| n.need_score >= 60).count(),
            sell_candidates.len()
        );

        Ok(TeamTransferStrategy {
            team_id: team.id,
            team_name: team.name.clone(),
            region_id: team.region_id,
            strategy_type,
            avg_ability,
            roster_count: roster.len() as u32,
            budget,
            salary_cap_space,
            position_needs,
            targets,
            sell_candidates,
            summary,
        })
    }

    fn determine_strategy_type(&self, avg_ability: f64, balance: i64, roster: &[&PlayerTransferInfo]) -> StrategyType {
        let has_budget = balance >= 1000_0000; // 至少1000万

        if avg_ability >= 85.0 && has_budget {
            StrategyType::Contender
        } else if avg_ability < 70.0 || roster.len() < 5 {
            StrategyType::Rebuilding
        } else if avg_ability >= 70.0 && avg_ability < 80.0 {
            StrategyType::Developing
        } else {
            StrategyType::Maintaining
        }
    }

    fn calculate_position_needs(&self, roster: &[&PlayerTransferInfo]) -> Vec<PositionNeed> {
        let positions = ["TOP", "JUG", "MID", "ADC", "SUP"];
        let mut needs = Vec::new();

        for pos in positions {
            let count = roster.iter().filter(|p| p.position == pos).count() as u32;
            let (need_score, reason) = match count {
                0 => (100, "急需补强".to_string()),
                1 => (70, "需要替补".to_string()),
                2 => (30, "阵容充足".to_string()),
                _ => (10, "位置冗余".to_string()),
            };

            needs.push(PositionNeed {
                position: pos.to_string(),
                current_count: count,
                need_score,
                reason,
            });
        }

        needs
    }

    fn calculate_transfer_budget(&self, team: &Team, strategy: &StrategyType) -> u64 {
        let ratio = match strategy {
            StrategyType::Contender => 0.7,
            StrategyType::Developing => 0.4,
            StrategyType::Rebuilding => 0.3,
            StrategyType::Maintaining => 0.5,
        };

        ((team.balance as f64 * ratio) as u64).max(500_0000) // 最低500万
    }

    fn find_transfer_targets(
        &self,
        team: &Team,
        strategy: &StrategyType,
        position_needs: &[PositionNeed],
        budget: u64,
    ) -> Vec<TransferTarget> {
        let mut targets = Vec::new();
        let min_ability = strategy.min_target_ability();
        let max_age = strategy.max_target_age();
        let max_multiplier = strategy.max_offer_multiplier();

        for player in self.players.values() {
            // 跳过自己球队的选手
            if player.team_id == Some(team.id) {
                continue;
            }

            // 只考虑自由球员和愿意转会的选手
            if !matches!(player.status, PlayerMarketStatus::FreeAgent | PlayerMarketStatus::WillingToTransfer) {
                continue;
            }

            // 能力筛选
            if player.ability < min_ability {
                continue;
            }

            // 年龄筛选
            if player.age > max_age {
                continue;
            }

            // 预算筛选
            let max_offer = (player.market_value as f64 * max_multiplier) as u64;
            if max_offer > budget {
                continue;
            }

            // 计算优先级
            let position_need = position_needs.iter()
                .find(|n| n.position == player.position)
                .map(|n| n.need_score)
                .unwrap_or(0);

            let priority = ((position_need / 10) as u8)
                .saturating_add(player.ability / 10)
                .min(10);

            if priority < 3 {
                continue;  // 优先级太低
            }

            let reason = format!(
                "{}位置需补强，能力{}，年龄{}岁",
                player.position, player.ability, player.age
            );

            targets.push(TransferTarget {
                player_id: player.player_id,
                player_name: player.player_name.clone(),
                position: player.position.clone(),
                age: player.age,
                ability: player.ability,
                potential: player.potential,
                current_team_id: player.team_id,
                current_team_name: player.team_name.clone(),
                market_value: player.market_value,
                max_offer,
                priority,
                reason,
            });
        }

        // 按优先级排序，取前8个
        targets.sort_by(|a, b| b.priority.cmp(&a.priority));
        targets.truncate(8);
        targets
    }

    fn find_sell_candidates(
        &self,
        team: &Team,
        roster: &[&PlayerTransferInfo],
        strategy: &StrategyType,
    ) -> Vec<SellCandidate> {
        let mut candidates = Vec::new();

        for player in roster {
            let mut should_sell = false;
            let mut sell_reason = String::new();

            // 选手主动要求离队
            if player.intent.accepts_offers() {
                should_sell = true;
                sell_reason = format!("选手{}，希望离队", player.intent.name());
            }

            // 高薪低能
            if player.salary >= 200_0000 && player.ability < 75 {
                should_sell = true;
                sell_reason = format!("高薪低能：薪资{}万/能力{}", player.salary / 10000, player.ability);
            }

            // 年龄偏大且能力下滑
            if player.age >= 28 && player.ability < 80 {
                should_sell = true;
                sell_reason = format!("{}岁老将，能力{}", player.age, player.ability);
            }

            // 重建型球队清洗高薪选手
            if *strategy == StrategyType::Rebuilding && player.salary >= 150_0000 {
                should_sell = true;
                sell_reason = format!("重建清洗：薪资{}万", player.salary / 10000);
            }

            if should_sell {
                let min_price = (player.market_value as f64 * 0.7) as u64;

                candidates.push(SellCandidate {
                    player_id: player.player_id,
                    player_name: player.player_name.clone(),
                    position: player.position.clone(),
                    age: player.age,
                    ability: player.ability,
                    salary: player.salary,
                    market_value: player.market_value,
                    min_price,
                    sell_reason,
                });
            }
        }

        candidates
    }

    // ==================== 阶段3: 续约窗口 ====================

    /// 执行续约窗口阶段
    pub fn execute_renewal_window(&mut self, conn: &Connection) -> Result<Vec<RenewalResult>, String> {
        if self.state.phase != TransferPhase::RenewalWindow {
            return Err("当前不在续约窗口阶段".to_string());
        }

        let mut results = Vec::new();
        let current_season = self.state.season_id;

        // 找出合同到期但愿意留队的选手
        let renewal_candidates: Vec<u64> = self.players.values()
            .filter(|p| {
                p.contract_end_season.map(|end| end <= current_season).unwrap_or(false)
                    && p.intent.can_renew()
                    && p.team_id.is_some()
            })
            .map(|p| p.player_id)
            .collect();

        let mut rng = rand::thread_rng();

        for player_id in renewal_candidates {
            let player = self.players.get(&player_id).unwrap().clone();
            let team_id = player.team_id.unwrap();

            // 获取球队策略
            let team_strategy = self.teams.get(&team_id);
            let team_name = team_strategy.map(|t| t.team_name.clone()).unwrap_or_default();

            // 检查球队是否愿意续约
            let team_wants_renewal = team_strategy.map(|t| {
                // 重建型球队不愿意续约高薪选手
                if t.strategy_type == StrategyType::Rebuilding && player.salary >= 150_0000 {
                    return false;
                }
                // 检查是否在出售名单中
                if t.sell_candidates.iter().any(|c| c.player_id == player_id) {
                    return false;
                }
                true
            }).unwrap_or(true);

            if !team_wants_renewal {
                results.push(RenewalResult {
                    player_id,
                    player_name: player.player_name.clone(),
                    team_id,
                    team_name: team_name.clone(),
                    success: false,
                    new_salary: None,
                    new_years: None,
                    failure_reason: Some("球队不愿续约".to_string()),
                });

                // 更新选手状态为自由球员
                if let Some(p) = self.players.get_mut(&player_id) {
                    p.status = PlayerMarketStatus::FreeAgent;
                }
                self.state.free_agents_count += 1;

                self.add_event(TransferEventType::RenewalFailed,
                    Some(&player), Some(team_id), Some(&team_name),
                    format!("{}续约失败", player.player_name),
                    "球队决定不续约".to_string(),
                );

                continue;
            }

            // 计算续约条件
            let offered_salary = player.expected_salary;
            let offered_years = if player.age >= 28 { 1 } else if player.age >= 25 { 2 } else { 3 };

            // 选手是否接受
            let accept_prob = match player.intent {
                TransferIntent::StayHappy => 0.95,
                TransferIntent::StayNeutral => 0.7,
                TransferIntent::OpenToOffers => 0.4,
                _ => 0.1,
            };

            let player_accepts = rng.gen::<f64>() < accept_prob;

            if player_accepts {
                results.push(RenewalResult {
                    player_id,
                    player_name: player.player_name.clone(),
                    team_id,
                    team_name: team_name.clone(),
                    success: true,
                    new_salary: Some(offered_salary),
                    new_years: Some(offered_years),
                    failure_reason: None,
                });

                // 更新数据库
                self.update_contract(conn, player_id, offered_salary, offered_years)?;

                // 更新选手状态
                if let Some(p) = self.players.get_mut(&player_id) {
                    p.status = PlayerMarketStatus::Contracted;
                    p.salary = offered_salary * 10000;
                    p.contract_end_season = Some(self.state.season_id + offered_years as u64);
                }

                self.add_event(TransferEventType::RenewalSuccess,
                    Some(&player), Some(team_id), Some(&team_name),
                    format!("{}续约成功", player.player_name),
                    format!("{}万/年，{}年合同", offered_salary, offered_years),
                );
            } else {
                results.push(RenewalResult {
                    player_id,
                    player_name: player.player_name.clone(),
                    team_id,
                    team_name: team_name.clone(),
                    success: false,
                    new_salary: None,
                    new_years: None,
                    failure_reason: Some("选手拒绝续约".to_string()),
                });

                // 更新选手状态为自由球员
                if let Some(p) = self.players.get_mut(&player_id) {
                    p.status = PlayerMarketStatus::FreeAgent;
                    p.team_id = None;
                }
                self.state.free_agents_count += 1;

                self.add_event(TransferEventType::RenewalFailed,
                    Some(&player), Some(team_id), Some(&team_name),
                    format!("{}续约失败", player.player_name),
                    "选手拒绝续约条件".to_string(),
                );
            }
        }

        // 添加阶段完成事件
        let success_count = results.iter().filter(|r| r.success).count();
        self.add_event(TransferEventType::PhaseCompleted, None, None, None,
            "续约窗口完成".to_string(),
            format!("处理{}名选手，{}人续约成功", results.len(), success_count),
        );

        // 推进到下一阶段
        self.state.phase = TransferPhase::FreeMarket;
        self.state.current_round = 0;

        Ok(results)
    }

    fn update_contract(&self, conn: &Connection, player_id: u64, salary: u64, years: u8) -> Result<(), String> {
        let new_end_season = self.state.season_id + years as u64;
        let salary_value = salary * 10000;

        conn.execute(
            "UPDATE players SET salary = ?, contract_end_season = ? WHERE id = ? AND save_id = ?",
            rusqlite::params![salary_value, new_end_season, player_id, &self.state.save_id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    // ==================== 阶段4: 自由市场 ====================

    /// 执行一轮自由市场
    pub fn execute_free_market_round(&mut self, conn: &Connection) -> Result<RoundResult, String> {
        if self.state.phase != TransferPhase::FreeMarket {
            return Err("当前不在自由市场阶段".to_string());
        }

        self.state.current_round += 1;
        let current_round = self.state.current_round;
        let mut offers_made = 0;
        let mut signings_completed = 0;

        // 1. 球队发出报价
        let team_ids: Vec<u64> = self.teams.keys().cloned().collect();
        let mut new_offers = Vec::new();

        for team_id in team_ids {
            let team_offers = self.generate_team_offers(team_id, current_round);
            offers_made += team_offers.len() as u32;
            new_offers.extend(team_offers);
        }

        self.offers.extend(new_offers);

        // 2. 选手处理报价，选择最佳报价
        let signings = self.process_offers(conn)?;
        signings_completed = signings.len() as u32;

        // 3. 检查是否结束
        if signings_completed == 0 {
            self.state.no_signing_rounds += 1;
        } else {
            self.state.no_signing_rounds = 0;
        }

        self.state.completed_signings += signings_completed;

        let phase_changed = self.state.should_end_free_market();
        let new_phase = if phase_changed {
            self.state.phase = TransferPhase::Completed;
            self.add_event(TransferEventType::PhaseCompleted, None, None, None,
                "转会窗口关闭".to_string(),
                format!("共完成{}笔签约", self.state.completed_signings),
            );
            Some(TransferPhase::Completed)
        } else {
            None
        };

        let summary = format!(
            "第{}轮：发出{}份报价，完成{}笔签约",
            current_round, offers_made, signings_completed
        );

        Ok(RoundResult {
            round: current_round,
            phase: self.state.phase,
            phase_changed,
            new_phase,
            offers_made,
            signings_completed,
            events: self.events.iter()
                .filter(|e| e.round == current_round)
                .cloned()
                .collect(),
            summary,
        })
    }

    fn generate_team_offers(&mut self, team_id: u64, round: u32) -> Vec<TransferOffer> {
        let mut offers = Vec::new();

        let strategy = match self.teams.get(&team_id) {
            Some(s) => s.clone(),
            None => return offers,
        };

        // 最多发2个报价
        let max_offers = 2;
        let mut made_offers = 0;

        for target in &strategy.targets {
            if made_offers >= max_offers {
                break;
            }

            // 检查选手是否仍可获得
            let player = match self.players.get(&target.player_id) {
                Some(p) if matches!(p.status, PlayerMarketStatus::FreeAgent | PlayerMarketStatus::WillingToTransfer | PlayerMarketStatus::HasOffers) => p,
                _ => continue,
            };

            // 检查是否已发过报价
            if self.offers.iter().any(|o| o.player_id == target.player_id && o.from_team_id == team_id && o.status == OfferStatus::Pending) {
                continue;
            }

            // 生成报价
            let salary_offer = player.expected_salary;
            let contract_years = if player.age >= 28 { 1 } else if player.age >= 25 { 2 } else { 3 };
            let transfer_fee = if player.status == PlayerMarketStatus::WillingToTransfer {
                (player.market_value as f64 * 0.8) as u64  // 有合同需转会费
            } else {
                0
            };

            let offer = TransferOffer {
                id: self.next_offer_id,
                player_id: target.player_id,
                from_team_id: team_id,
                from_team_name: strategy.team_name.clone(),
                salary_offer,
                contract_years,
                transfer_fee,
                is_starter_promised: target.priority >= 7,
                round,
                status: OfferStatus::Pending,
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            self.next_offer_id += 1;
            made_offers += 1;

            // 更新选手状态
            if let Some(p) = self.players.get_mut(&target.player_id) {
                p.status = PlayerMarketStatus::HasOffers;
            }

            self.add_event(TransferEventType::OfferMade,
                Some(player), Some(team_id), Some(&strategy.team_name),
                format!("{}向{}发出报价", strategy.team_name, player.player_name),
                format!("{}万/年，{}年合同", salary_offer, contract_years),
            );

            offers.push(offer);
        }

        offers
    }

    fn process_offers(&mut self, conn: &Connection) -> Result<Vec<(u64, u64)>, String> {
        let mut signings = Vec::new();

        // 找出有报价的选手
        let players_with_offers: Vec<u64> = self.players.values()
            .filter(|p| p.status == PlayerMarketStatus::HasOffers)
            .map(|p| p.player_id)
            .collect();

        let mut rng = rand::thread_rng();

        for player_id in players_with_offers {
            // 获取该选手的所有待处理报价
            let pending_offers: Vec<_> = self.offers.iter()
                .filter(|o| o.player_id == player_id && o.status == OfferStatus::Pending)
                .cloned()
                .collect();

            if pending_offers.is_empty() {
                continue;
            }

            let player = self.players.get(&player_id).unwrap().clone();

            // 评分并选择最佳报价
            let best_offer = pending_offers.iter()
                .max_by_key(|o| {
                    let salary_score = (o.salary_offer as i64 * 100 / player.expected_salary.max(1) as i64) as i32;
                    let starter_score = if o.is_starter_promised { 20 } else { 0 };
                    let years_score = o.contract_years as i32 * 5;
                    salary_score + starter_score + years_score
                });

            if let Some(chosen) = best_offer {
                // 决定是否接受
                let accept_prob = match player.intent {
                    TransferIntent::MustLeave => 0.95,
                    TransferIntent::WantsOut => 0.85,
                    TransferIntent::OpenToOffers => 0.6,
                    TransferIntent::StayNeutral => 0.3,
                    TransferIntent::StayHappy => 0.1,
                };

                let salary_factor = (chosen.salary_offer as f64 / player.expected_salary.max(1) as f64).min(1.5);
                let final_prob = accept_prob * salary_factor;

                if rng.gen::<f64>() < final_prob {
                    // 接受报价
                    signings.push((player_id, chosen.from_team_id));

                    // 更新数据库
                    self.execute_signing(conn, &player, chosen)?;

                    // 更新选手状态
                    if let Some(p) = self.players.get_mut(&player_id) {
                        p.status = PlayerMarketStatus::Signed;
                        p.team_id = Some(chosen.from_team_id);
                        p.team_name = Some(chosen.from_team_name.clone());
                        p.salary = chosen.salary_offer * 10000;
                        p.contract_end_season = Some(self.state.season_id + chosen.contract_years as u64);
                    }

                    // 更新报价状态
                    for offer in &mut self.offers {
                        if offer.player_id == player_id {
                            if offer.id == chosen.id {
                                offer.status = OfferStatus::Accepted;
                            } else {
                                offer.status = OfferStatus::Rejected;
                            }
                        }
                    }

                    self.add_event(TransferEventType::SigningCompleted,
                        Some(&player), Some(chosen.from_team_id), Some(&chosen.from_team_name),
                        format!("{} 加盟 {}", player.player_name, chosen.from_team_name),
                        format!("{}万/年，{}年合同", chosen.salary_offer, chosen.contract_years),
                    );
                } else {
                    // 拒绝所有报价
                    for offer in &mut self.offers {
                        if offer.player_id == player_id && offer.status == OfferStatus::Pending {
                            offer.status = OfferStatus::Rejected;
                        }
                    }

                    // 如果是自由球员，恢复可报价状态
                    if let Some(p) = self.players.get_mut(&player_id) {
                        if p.status == PlayerMarketStatus::HasOffers {
                            p.status = if p.team_id.is_none() {
                                PlayerMarketStatus::FreeAgent
                            } else {
                                PlayerMarketStatus::WillingToTransfer
                            };
                        }
                    }

                    self.add_event(TransferEventType::OfferRejected,
                        Some(&player), None, None,
                        format!("{} 拒绝报价", player.player_name),
                        "选手拒绝了所有报价".to_string(),
                    );
                }
            }
        }

        Ok(signings)
    }

    fn execute_signing(&self, conn: &Connection, player: &PlayerTransferInfo, offer: &TransferOffer) -> Result<(), String> {
        let new_end_season = self.state.season_id + offer.contract_years as u64;
        let salary_value = offer.salary_offer * 10000;

        // 更新选手
        conn.execute(
            "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, status = 'Active' WHERE id = ? AND save_id = ?",
            rusqlite::params![offer.from_team_id, salary_value, new_end_season, player.player_id, &self.state.save_id],
        ).map_err(|e| e.to_string())?;

        // 更新球队余额
        if offer.transfer_fee > 0 {
            conn.execute(
                "UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?",
                rusqlite::params![offer.transfer_fee as i64, offer.from_team_id, &self.state.save_id],
            ).map_err(|e| e.to_string())?;

            if let Some(from_team_id) = player.team_id {
                conn.execute(
                    "UPDATE teams SET balance = balance + ? WHERE id = ? AND save_id = ?",
                    rusqlite::params![offer.transfer_fee as i64, from_team_id, &self.state.save_id],
                ).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    // ==================== 辅助方法 ====================

    fn add_event(
        &mut self,
        event_type: TransferEventType,
        player: Option<&PlayerTransferInfo>,
        team_id: Option<u64>,
        team_name: Option<&str>,
        title: String,
        description: String,
    ) {
        let event = TransferEvent {
            id: self.next_event_id,
            event_type,
            round: self.state.current_round,
            player_id: player.map(|p| p.player_id),
            player_name: player.map(|p| p.player_name.clone()),
            from_team_id: player.and_then(|p| p.team_id),
            from_team_name: player.and_then(|p| p.team_name.clone()),
            to_team_id: team_id,
            to_team_name: team_name.map(|s| s.to_string()),
            amount: None,
            title,
            description,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.next_event_id += 1;
        self.events.push(event);
    }

    /// 快进完成当前阶段
    pub fn fast_forward(&mut self, conn: &Connection) -> Result<Vec<RoundResult>, String> {
        let mut results = Vec::new();

        while self.state.phase == TransferPhase::FreeMarket && !self.state.should_end_free_market() {
            let result = self.execute_free_market_round(conn)?;
            results.push(result);
        }

        Ok(results)
    }
}
