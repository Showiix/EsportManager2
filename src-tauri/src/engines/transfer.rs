use crate::models::{
    FreeAgent, Player, PlayerStatus, PlayerTag, Position, Team,
    TransferListing, TransferRecord, TransferType,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 转会AI引擎 - 管理球员转会和自由球员签约
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
    /// 能力值权重
    pub ability_weight: f64,
    /// 潜力值权重
    pub potential_weight: f64,
    /// 年龄权重
    pub age_weight: f64,
    /// 位置需求权重
    pub position_need_weight: f64,
    /// 财务预算比例
    pub budget_ratio: f64,
}

impl Default for TransferAIConfig {
    fn default() -> Self {
        Self {
            ability_weight: 0.4,
            potential_weight: 0.3,
            age_weight: 0.15,
            position_need_weight: 0.15,
            budget_ratio: 0.3, // 默认使用30%资金用于转会
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

    pub fn with_config(config: TransferAIConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// 导入转会市场
    pub fn import_market(&mut self, listings: Vec<TransferListing>) {
        self.transfer_market = listings;
    }

    /// 导入自由球员
    pub fn import_free_agents(&mut self, agents: Vec<FreeAgent>) {
        self.free_agents = agents;
    }

    /// 计算球员市场价值
    pub fn calculate_market_value(player: &Player) -> u64 {
        let base_value = match player.ability {
            90..=100 => 500,
            85..=89 => 350,
            80..=84 => 250,
            75..=79 => 180,
            70..=74 => 120,
            65..=69 => 80,
            60..=64 => 50,
            _ => 30,
        };

        // 潜力加成
        let potential_bonus = if player.potential > player.ability + 10 {
            1.5
        } else if player.potential > player.ability + 5 {
            1.25
        } else {
            1.0
        };

        // 年龄折扣
        let age_factor = match player.age {
            17..=20 => 1.3,  // 年轻有潜力
            21..=24 => 1.2,  // 黄金年龄
            25..=27 => 1.0,  // 成熟期
            28..=30 => 0.8,  // 下滑期
            _ => 0.5,        // 老将
        };

        // 标签加成 (Ordinary/Normal/Genius)
        let tag_bonus = match player.tag {
            PlayerTag::Genius => 2.0,
            PlayerTag::Normal => 1.0,
            PlayerTag::Ordinary => 0.7,
        };

        ((base_value as f64) * potential_bonus * age_factor * tag_bonus) as u64
    }

    /// 计算球员对队伍的价值评分
    pub fn evaluate_player_value(
        &self,
        player: &Player,
        _team: &Team,
        roster: &[Player],
    ) -> f64 {
        let mut score = 0.0;

        // 能力值评分
        score += (player.ability as f64) * self.config.ability_weight;

        // 潜力值评分
        score += (player.potential as f64) * self.config.potential_weight;

        // 年龄评分 (年轻球员更有价值)
        let age_score = match player.age {
            17..=20 => 90.0,
            21..=24 => 85.0,
            25..=27 => 75.0,
            28..=30 => 60.0,
            _ => 40.0,
        };
        score += age_score * self.config.age_weight;

        // 位置需求评分
        if let Some(position) = player.position {
            let position_need = self.calculate_position_need(position, roster);
            score += position_need * self.config.position_need_weight;
        }

        score
    }

    /// 计算位置需求度 (0-100)
    fn calculate_position_need(&self, position: Position, roster: &[Player]) -> f64 {
        let position_count = roster
            .iter()
            .filter(|p| p.position == Some(position) && p.status == PlayerStatus::Active)
            .count();

        // 每个位置理想人数是2-3人
        match position_count {
            0 => 100.0, // 急需
            1 => 80.0,  // 需要
            2 => 40.0,  // 可考虑
            _ => 10.0,  // 不需要
        }
    }

    /// AI评估是否应该购买球员
    pub fn should_buy_player(
        &self,
        listing: &TransferListing,
        player: &Player,
        team: &Team,
        roster: &[Player],
    ) -> TransferDecision {
        let value_score = self.evaluate_player_value(player, team, roster);
        let market_value = Self::calculate_market_value(player);

        // 计算性价比
        let price_ratio = listing.asking_price as f64 / market_value as f64;

        // 预算检查
        let budget = (team.balance as f64 * self.config.budget_ratio) as i64;
        if listing.asking_price as i64 > budget {
            return TransferDecision::Reject(TransferRejectReason::InsufficientBudget);
        }

        // 价值评估
        if price_ratio > 1.5 {
            return TransferDecision::Reject(TransferRejectReason::OverPriced);
        }

        // 位置需求检查
        if let Some(position) = player.position {
            let position_need = self.calculate_position_need(position, roster);
            if position_need < 30.0 && price_ratio > 1.0 {
                return TransferDecision::Reject(TransferRejectReason::NoPositionNeed);
            }
        }

        // 根据评分决定
        if value_score > 75.0 {
            // 高价值球员，愿意支付溢价
            let max_offer = (market_value as f64 * 1.3) as u64;
            TransferDecision::Offer(max_offer.min(budget as u64))
        } else if value_score > 60.0 {
            // 中等价值，支付市场价
            let max_offer = market_value;
            if listing.asking_price <= max_offer {
                TransferDecision::Offer(listing.asking_price)
            } else {
                TransferDecision::Negotiate((market_value as f64 * 0.9) as u64)
            }
        } else {
            // 低价值，只接受低价
            if listing.asking_price <= (market_value as f64 * 0.7) as u64 {
                TransferDecision::Offer(listing.asking_price)
            } else {
                TransferDecision::Reject(TransferRejectReason::LowValue)
            }
        }
    }

    /// AI评估是否应该出售球员
    pub fn should_sell_player(
        &self,
        player: &Player,
        _team: &Team,
        roster: &[Player],
        offer: u64,
    ) -> bool {
        let market_value = Self::calculate_market_value(player);

        if let Some(position) = player.position {
            let position_need = self.calculate_position_need(position, roster);

            // 队内该位置人满，且报价合理
            if position_need <= 10.0 && offer >= (market_value as f64 * 0.8) as u64 {
                return true;
            }
        }

        // 高于市场价20%
        if offer >= (market_value as f64 * 1.2) as u64 {
            return true;
        }

        // 球员30岁以上，报价合理
        if player.age >= 30 && offer >= (market_value as f64 * 0.9) as u64 {
            return true;
        }

        false
    }

    /// AI评估是否应该签约自由球员
    pub fn should_sign_free_agent(
        &self,
        agent: &FreeAgent,
        player: &Player,
        team: &Team,
        roster: &[Player],
    ) -> FreeAgentDecision {
        let value_score = self.evaluate_player_value(player, team, roster);

        let position_need = if let Some(position) = player.position {
            self.calculate_position_need(position, roster)
        } else {
            50.0 // 默认需求度
        };

        // 预算检查 (年薪 * 合同年限)
        let total_cost = agent.salary_demand * 3; // 假设3年合同
        let budget = (team.balance as f64 * self.config.budget_ratio) as u64;

        if total_cost > budget {
            return FreeAgentDecision::Reject(FreeAgentRejectReason::SalaryTooHigh);
        }

        // 位置不需要
        if position_need < 20.0 {
            return FreeAgentDecision::Reject(FreeAgentRejectReason::NoPositionNeed);
        }

        // 价值评估
        if value_score > 70.0 {
            // 高价值，支付预期薪资
            FreeAgentDecision::Offer {
                salary: agent.salary_demand,
                contract_years: 3,
            }
        } else if value_score > 55.0 && position_need > 50.0 {
            // 中等价值但有需求
            FreeAgentDecision::Offer {
                salary: (agent.salary_demand as f64 * 0.85) as u64,
                contract_years: 2,
            }
        } else {
            FreeAgentDecision::Reject(FreeAgentRejectReason::LowValue)
        }
    }

    /// 生成AI转会行为
    pub fn generate_ai_transfers(
        &self,
        _save_id: &str,
        _season_id: u64,
        teams: &[Team],
        all_players: &HashMap<u64, Vec<Player>>, // team_id -> players
    ) -> Vec<TransferAction> {
        let mut rng = rand::thread_rng();
        let mut actions = Vec::new();

        for team in teams {
            let roster = all_players.get(&team.id).cloned().unwrap_or_default();

            // 1. 检查是否需要购买球员
            for listing in &self.transfer_market {
                // 跳过自己队伍的挂牌
                if listing.team_id == team.id {
                    continue;
                }

                // 找到挂牌球员信息 (需要从其他队伍的roster中找)
                if let Some(player) = all_players
                    .get(&listing.team_id)
                    .and_then(|r| r.iter().find(|p| p.id == listing.player_id))
                {
                    let decision = self.should_buy_player(listing, player, team, &roster);

                    if let TransferDecision::Offer(amount) = decision {
                        // 添加一定随机性 (80%概率执行)
                        if rng.gen::<f64>() < 0.8 {
                            actions.push(TransferAction::Buy {
                                buyer_team_id: team.id,
                                listing_id: listing.id,
                                offer_amount: amount,
                            });
                        }
                    }
                }
            }

            // 2. 检查是否应该签约自由球员
            for agent in &self.free_agents {
                // 找到自由球员信息
                // 这里简化处理，假设自由球员信息已在agent中
                let mock_player = Player {
                    id: agent.player_id,
                    game_id: String::new(),
                    real_name: None,
                    nationality: None,
                    age: 25, // 假设值
                    ability: 70,
                    potential: 75,
                    stability: 80,
                    tag: PlayerTag::Normal,
                    status: PlayerStatus::Active,
                    position: Some(Position::Mid), // 需要从实际数据获取
                    team_id: None,
                    salary: 0,
                    market_value: 0,
                    contract_end_season: None,
                    join_season: 0,
                    retire_season: None,
                    is_starter: false,
                };

                let decision = self.should_sign_free_agent(agent, &mock_player, team, &roster);

                if let FreeAgentDecision::Offer { salary, contract_years } = decision {
                    if rng.gen::<f64>() < 0.6 {
                        actions.push(TransferAction::SignFreeAgent {
                            team_id: team.id,
                            agent_id: agent.id,
                            salary_offer: salary,
                            contract_years,
                        });
                    }
                }
            }

            // 3. 检查是否应该挂牌球员
            for player in &roster {
                // 板凳球员且位置人多
                if !player.is_starter {
                    if let Some(position) = player.position {
                        let position_need = self.calculate_position_need(position, &roster);
                        if position_need < 30.0 && rng.gen::<f64>() < 0.3 {
                            let asking_price = Self::calculate_market_value(player);
                            actions.push(TransferAction::List {
                                team_id: team.id,
                                player_id: player.id,
                                asking_price,
                            });
                        }
                    }
                }
            }
        }

        actions
    }

    /// 执行转会交易
    pub fn execute_transfer(
        &mut self,
        save_id: &str,
        season_id: u64,
        action: &TransferAction,
    ) -> Option<TransferRecord> {
        match action {
            TransferAction::Buy {
                buyer_team_id,
                listing_id,
                offer_amount,
            } => {
                // 找到挂牌信息
                if let Some(listing) = self.transfer_market.iter().find(|l| l.id == *listing_id) {
                    Some(TransferRecord {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        player_id: listing.player_id,
                        from_team_id: Some(listing.team_id),
                        to_team_id: Some(*buyer_team_id),
                        transfer_type: TransferType::Purchase,
                        transfer_fee: *offer_amount,
                        new_salary: None,
                        contract_years: Some(3),
                        description: None,
                    })
                } else {
                    None
                }
            }
            TransferAction::SignFreeAgent {
                team_id,
                agent_id,
                salary_offer,
                contract_years,
            } => {
                if let Some(agent) = self.free_agents.iter().find(|a| a.id == *agent_id) {
                    Some(TransferRecord {
                        id: 0,
                        save_id: save_id.to_string(),
                        season_id,
                        player_id: agent.player_id,
                        from_team_id: None,
                        to_team_id: Some(*team_id),
                        transfer_type: TransferType::FreeAgent,
                        transfer_fee: 0,
                        new_salary: Some(*salary_offer),
                        contract_years: Some(*contract_years),
                        description: None,
                    })
                } else {
                    None
                }
            }
            TransferAction::List { .. } => {
                // 挂牌不产生转会记录
                None
            }
        }
    }

    /// 获取转会市场
    pub fn get_market(&self) -> &[TransferListing] {
        &self.transfer_market
    }

    /// 获取自由球员市场
    pub fn get_free_agents(&self) -> &[FreeAgent] {
        &self.free_agents
    }
}

/// 转会决策
#[derive(Debug, Clone)]
pub enum TransferDecision {
    /// 直接出价
    Offer(u64),
    /// 议价
    Negotiate(u64),
    /// 拒绝
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
    /// 购买挂牌球员
    Buy {
        buyer_team_id: u64,
        listing_id: u64,
        offer_amount: u64,
    },
    /// 签约自由球员
    SignFreeAgent {
        team_id: u64,
        agent_id: u64,
        salary_offer: u64,
        contract_years: u32,
    },
    /// 挂牌出售
    List {
        team_id: u64,
        player_id: u64,
        asking_price: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ListingStatus, ListingType};

    fn create_test_player(id: u64, ability: u8, potential: u8, age: u8, position: Position) -> Player {
        Player {
            id,
            game_id: format!("Player{}", id),
            real_name: Some(format!("Test Player {}", id)),
            nationality: Some("CN".to_string()),
            age,
            ability,
            potential,
            stability: 80,
            tag: PlayerTag::Normal,
            status: PlayerStatus::Active,
            position: Some(position),
            team_id: Some(1),
            salary: 50,
            market_value: 100,
            contract_end_season: Some(5),
            join_season: 1,
            retire_season: None,
            is_starter: false,
        }
    }

    fn create_test_team(id: u64, balance: i64) -> Team {
        Team {
            id,
            region_id: 1,
            name: format!("Team {}", id),
            short_name: Some(format!("T{}", id)),
            power_rating: 75.0,
            total_matches: 0,
            wins: 0,
            win_rate: 0.0,
            annual_points: 0,
            cross_year_points: 0,
            balance,
        }
    }

    #[test]
    fn test_market_value_calculation() {
        let star_player = create_test_player(1, 90, 95, 22, Position::Mid);
        let value = TransferEngine::calculate_market_value(&star_player);
        assert!(value > 500); // 高能力球员价值高

        let old_player = create_test_player(2, 80, 82, 32, Position::Top);
        let old_value = TransferEngine::calculate_market_value(&old_player);
        assert!(old_value < value); // 年龄大的价值低
    }

    #[test]
    fn test_position_need() {
        let engine = TransferEngine::new();

        // 空阵容，急需
        let empty_roster: Vec<Player> = vec![];
        let need = engine.calculate_position_need(Position::Mid, &empty_roster);
        assert_eq!(need, 100.0);

        // 有一个同位置球员
        let roster = vec![create_test_player(1, 75, 80, 23, Position::Mid)];
        let need = engine.calculate_position_need(Position::Mid, &roster);
        assert_eq!(need, 80.0);
    }

    #[test]
    fn test_buy_decision() {
        let engine = TransferEngine::new();
        let team = create_test_team(1, 1000);
        let player = create_test_player(1, 85, 90, 22, Position::Mid);
        let roster: Vec<Player> = vec![];

        let listing = TransferListing {
            id: 1,
            save_id: "save1".to_string(),
            season_id: 1,
            player_id: 1,
            team_id: 2,
            listing_type: ListingType::ForSale,
            asking_price: 200,
            min_price: None,
            status: ListingStatus::Active,
        };

        let decision = engine.should_buy_player(&listing, &player, &team, &roster);

        match decision {
            TransferDecision::Offer(_) | TransferDecision::Negotiate(_) => {
                // 合理的价格应该会出价
            }
            _ => panic!("Should make an offer for a good player at reasonable price"),
        }
    }
}
