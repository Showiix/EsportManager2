//! AI 转会策略生成服务
//!
//! 提供 Mock AI 策略生成功能，根据 GM 人格和球队状态
//! 生成符合人格特点的转会策略。

use crate::models::{
    AITransferStrategy, BudgetAllocation, GMPersonality, Player, PlayerStatus,
    SellCandidate, Team, TeamGMProfile, TransferTarget,
    PlayerTransferStrategy, PreferredTeam, TeamPreferenceReason,
    AnalysisDataSnapshot, AnalysisStep,
};
use crate::models::player_status::DepartureReason;
use crate::engines::transfer::FreeAgentInfo;
use crate::services::llm_service::TeamInfo;
use rand::Rng;
use std::collections::HashMap;

/// AI 转会策略生成服务
pub struct AITransferService;

impl AITransferService {
    /// 为所有球队生成 Mock AI 策略
    pub fn generate_all_strategies(
        teams: &[Team],
        players_by_team: &HashMap<u64, Vec<Player>>,
        free_agents: &[FreeAgentInfo],
        gm_profiles: &HashMap<u64, TeamGMProfile>,
        save_id: &str,
        season_id: u64,
    ) -> Vec<AITransferStrategy> {
        let mut strategies = Vec::new();

        for team in teams {
            let roster = players_by_team.get(&team.id).cloned().unwrap_or_default();
            let profile = gm_profiles.get(&team.id).cloned().unwrap_or_else(|| {
                TeamGMProfile::new(team.id, save_id.to_string())
            });

            let strategy = Self::generate_mock_strategy(
                team,
                &roster,
                &profile,
                free_agents,
                players_by_team,
                save_id,
                season_id,
            );
            strategies.push(strategy);
        }

        strategies
    }

    /// 为单个球队生成 Mock AI 策略
    pub fn generate_mock_strategy(
        team: &Team,
        roster: &[Player],
        gm_profile: &TeamGMProfile,
        free_agents: &[FreeAgentInfo],
        all_players_by_team: &HashMap<u64, Vec<Player>>,
        save_id: &str,
        season_id: u64,
    ) -> AITransferStrategy {
        let mut rng = rand::thread_rng();

        // 计算阵容统计
        let active_roster: Vec<_> = roster
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .collect();

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

        // 计算位置需求
        let position_needs = Self::calculate_position_needs(&active_roster);

        // 根据人格生成策略
        let (targets, willing_to_sell, reasoning_parts) = match gm_profile.personality {
            GMPersonality::Championship => {
                Self::generate_championship_strategy(
                    team, &active_roster, free_agents, all_players_by_team, &position_needs, &mut rng
                )
            }
            GMPersonality::YouthDevelopment => {
                Self::generate_youth_strategy(
                    team, &active_roster, free_agents, &position_needs, &mut rng
                )
            }
            GMPersonality::Balanced => {
                Self::generate_balanced_strategy(
                    team, &active_roster, free_agents, &position_needs, &mut rng
                )
            }
            GMPersonality::Speculator => {
                Self::generate_speculator_strategy(
                    team, &active_roster, free_agents, &position_needs, &mut rng
                )
            }
            GMPersonality::Rebuilding => {
                Self::generate_rebuilding_strategy(
                    team, &active_roster, free_agents, &position_needs, &mut rng
                )
            }
            GMPersonality::Custom => {
                // 自定义人格使用稳健型逻辑
                Self::generate_balanced_strategy(
                    team, &active_roster, free_agents, &position_needs, &mut rng
                )
            }
        };

        // 计算优先补强位置
        let mut priority_positions: Vec<_> = position_needs
            .iter()
            .filter(|(_, &need)| need >= 50)
            .collect();
        priority_positions.sort_by(|a, b| b.1.cmp(a.1));
        let priority_positions: Vec<String> = priority_positions
            .into_iter()
            .take(3)
            .map(|(pos, _)| pos.clone())
            .collect();

        // 计算预算分配
        let budget_ratio = gm_profile.effective_budget_ratio();
        let budget_allocation = BudgetAllocation::from_balance(team.balance, budget_ratio);

        // 生成策略描述
        let strategy_name = format!("{}策略", gm_profile.personality.name());
        let strategy_desc = gm_profile.personality.description().to_string();

        AITransferStrategy {
            id: 0,
            team_id: team.id,
            team_name: team.name.clone(),
            save_id: save_id.to_string(),
            season_id,
            overall_strategy: strategy_name,
            strategy_description: strategy_desc,
            targets,
            willing_to_sell,
            priority_positions,
            budget_allocation,
            reasoning: reasoning_parts.join("；"),
            analysis_steps: Vec::new(),  // 规则 AI 不生成分析步骤
            is_mock: true,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 计算位置需求分数 (0-100)
    fn calculate_position_needs(roster: &[&Player]) -> HashMap<String, u32> {
        let mut needs = HashMap::new();
        let positions = vec!["TOP", "JUG", "MID", "ADC", "SUP"];

        for pos in positions {
            let count = roster.iter()
                .filter(|p| {
                    p.position
                        .map(|pp| format!("{:?}", pp).to_uppercase() == pos)
                        .unwrap_or(false)
                })
                .count();

            let need = match count {
                0 => 100,  // 急需
                1 => 60,   // 需要替补
                2 => 30,   // 可考虑
                _ => 10,   // 充足
            };
            needs.insert(pos.to_string(), need);
        }

        needs
    }

    /// 争冠型策略生成
    fn generate_championship_strategy(
        team: &Team,
        roster: &[&Player],
        free_agents: &[FreeAgentInfo],
        all_players_by_team: &HashMap<u64, Vec<Player>>,
        position_needs: &HashMap<String, u32>,
        rng: &mut impl Rng,
    ) -> (Vec<TransferTarget>, Vec<SellCandidate>, Vec<String>) {
        let mut targets = Vec::new();
        let mut willing_to_sell = Vec::new();
        let mut reasoning = Vec::new();

        // 计算球队平均能力和最低首发能力
        let avg_ability = if !roster.is_empty() {
            roster.iter().map(|p| p.ability as u32).sum::<u32>() / roster.len() as u32
        } else {
            75
        };
        let min_starter = roster.iter()
            .filter(|p| p.starter)
            .map(|p| p.ability)
            .min()
            .unwrap_or(75);

        reasoning.push(format!(
            "{}作为争冠球队，目标是引进顶级选手，冲击冠军",
            team.name
        ));

        // 1. 寻找能提升阵容的自由球员（放宽到能力 >= 75 或高于最低首发）
        for fa in free_agents.iter().filter(|f| f.player.ability >= 75 || f.player.ability > min_starter) {
            let position = fa.player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default();

            let position_need = position_needs.get(&position).copied().unwrap_or(0);
            // 放宽位置需求，或者能力很高就不管位置
            if position_need < 20 && fa.player.ability < 85 {
                continue;
            }

            let premium = 1.0 + rng.gen_range(0.1..0.3); // 愿意溢价10-30%
            let max_offer = (fa.market_value as f64 * premium) as u64;

            targets.push(TransferTarget {
                player_id: fa.player.id,
                player_name: fa.player.game_id.clone(),
                position: position.clone(),
                ability: fa.player.ability,
                potential: fa.player.potential,
                age: fa.player.age,
                current_team_id: None,
                current_team_name: None,
                market_value: fa.market_value,
                max_offer,
                priority: fa.player.ability.saturating_sub(70),
                reasoning: format!(
                    "{}选手，能力{}，可提升{}位置实力",
                    if fa.player.ability >= 85 { "顶级" } else { "优质" },
                    fa.player.ability, position
                ),
            });
        }

        // 2. 寻找其他球队的顶级选手（挖角）- 放宽到 >= 82
        for (other_team_id, players) in all_players_by_team {
            if *other_team_id == team.id {
                continue;
            }

            for player in players.iter().filter(|p| p.ability >= 82 && p.status == PlayerStatus::Active) {
                let position = player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default();

                let position_need = position_needs.get(&position).copied().unwrap_or(0);
                if position_need < 30 && player.ability < 88 {
                    continue;
                }

                // 根据能力决定挖角概率
                let poach_chance = if player.ability >= 90 { 0.4 } else { 0.25 };
                if rng.gen::<f64>() > poach_chance {
                    continue;
                }

                let market_value = player.calculate_market_value();
                let premium = 1.2 + rng.gen_range(0.0..0.2); // 溢价20-40%
                let max_offer = (market_value as f64 * premium) as u64;

                targets.push(TransferTarget {
                    player_id: player.id,
                    player_name: player.game_id.clone(),
                    position: position.clone(),
                    ability: player.ability,
                    potential: player.potential,
                    age: player.age,
                    current_team_id: Some(*other_team_id),
                    current_team_name: None,
                    market_value,
                    max_offer,
                    priority: player.ability.saturating_sub(80) + 5, // 挖角优先级高
                    reasoning: format!(
                        "其他球队核心选手，能力{}，值得高价挖角",
                        player.ability
                    ),
                });
            }
        }

        // 3. 出售条件放宽：老将替补、能力下滑、高薪低能
        for player in roster.iter() {
            let should_sell =
                (player.age >= 27 && !player.starter) ||  // 老将替补
                (player.ability < avg_ability as u8 - 10) ||  // 远低于平均
                (player.salary >= 180 && player.ability < 78) ||  // 高薪低能
                (player.ability < 70);  // 能力太低

            if should_sell {
                let market_value = player.calculate_market_value();
                let min_price = (market_value as f64 * 0.8) as u64;

                let sell_reason = if player.age >= 27 && !player.starter {
                    format!("{}岁老将替补，可腾出薪资空间追逐巨星", player.age)
                } else if player.ability < 70 {
                    format!("能力{}不符合争冠要求", player.ability)
                } else {
                    format!("能力{}性价比不足，可优化阵容", player.ability)
                };

                willing_to_sell.push(SellCandidate {
                    player_id: player.id,
                    player_name: player.game_id.clone(),
                    position: player.position
                        .map(|p| format!("{:?}", p).to_uppercase())
                        .unwrap_or_default(),
                    ability: player.ability,
                    age: player.age,
                    salary: player.salary,
                    market_value,
                    min_price,
                    urgency: if player.ability < 70 { 8 } else { 5 },
                    reasoning: sell_reason,
                });
            }
        }

        // 生成策略说明
        if !targets.is_empty() {
            let top_targets: Vec<_> = targets.iter()
                .filter(|t| t.ability >= 85)
                .map(|t| t.player_name.clone())
                .take(3)
                .collect();
            if !top_targets.is_empty() {
                reasoning.push(format!("重点关注: {}", top_targets.join("、")));
            }
            reasoning.push(format!("目标引进{}名优质选手", targets.len()));
        }
        if !willing_to_sell.is_empty() {
            reasoning.push(format!("计划清理{}名边缘球员", willing_to_sell.len()));
        }

        // 按优先级排序
        targets.sort_by(|a, b| b.priority.cmp(&a.priority));
        targets.truncate(5); // 最多5个目标

        (targets, willing_to_sell, reasoning)
    }

    /// 青训型策略生成
    fn generate_youth_strategy(
        team: &Team,
        roster: &[&Player],
        free_agents: &[FreeAgentInfo],
        position_needs: &HashMap<String, u32>,
        rng: &mut impl Rng,
    ) -> (Vec<TransferTarget>, Vec<SellCandidate>, Vec<String>) {
        let mut targets = Vec::new();
        let mut willing_to_sell = Vec::new();
        let mut reasoning = Vec::new();

        reasoning.push("专注培养年轻选手，控制薪资支出".to_string());

        // 1. 寻找年轻高潜选手 (22岁以下，潜力80+)
        for fa in free_agents.iter().filter(|f| f.player.age <= 22 && f.player.potential >= 80) {
            let position = fa.player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default();

            let position_need = position_needs.get(&position).copied().unwrap_or(0);
            if position_need < 30 {
                continue;
            }

            // 青训型不愿意溢价
            let max_offer = (fa.market_value as f64 * 0.9) as u64;

            targets.push(TransferTarget {
                player_id: fa.player.id,
                player_name: fa.player.game_id.clone(),
                position: position.clone(),
                ability: fa.player.ability,
                potential: fa.player.potential,
                age: fa.player.age,
                current_team_id: None,
                current_team_name: None,
                market_value: fa.market_value,
                max_offer,
                priority: fa.player.potential.saturating_sub(70),
                reasoning: format!(
                    "年轻高潜新星，{}岁，潜力{}，符合青训理念",
                    fa.player.age, fa.player.potential
                ),
            });
        }

        // 2. 愿意出售巅峰期高薪选手 (25-28岁)
        for player in roster.iter().filter(|p| p.age >= 25 && p.age <= 28 && p.salary >= 100) {
            let market_value = player.calculate_market_value();
            // 巅峰期可以溢价出售
            let min_price = (market_value as f64 * 1.1) as u64;

            willing_to_sell.push(SellCandidate {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default(),
                ability: player.ability,
                age: player.age,
                salary: player.salary,
                market_value,
                min_price,
                urgency: 3,
                reasoning: "巅峰期选手，可卖出换取资金培养新人".to_string(),
            });
        }

        // 3. 也愿意出售老将
        for player in roster.iter().filter(|p| p.age >= 28) {
            let market_value = player.calculate_market_value();
            let min_price = (market_value as f64 * 0.85) as u64;

            // 避免重复添加
            if willing_to_sell.iter().any(|s| s.player_id == player.id) {
                continue;
            }

            willing_to_sell.push(SellCandidate {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default(),
                ability: player.ability,
                age: player.age,
                salary: player.salary,
                market_value,
                min_price,
                urgency: 4,
                reasoning: "老将选手，为年轻人让路".to_string(),
            });
        }

        if !targets.is_empty() {
            reasoning.push(format!("目标签约{}名年轻潜力新星", targets.len()));
        }
        if !willing_to_sell.is_empty() {
            reasoning.push(format!("愿意出售{}名选手换取培养资金", willing_to_sell.len()));
        }

        // 按潜力排序
        targets.sort_by(|a, b| b.potential.cmp(&a.potential));
        targets.truncate(5);

        (targets, willing_to_sell, reasoning)
    }

    /// 稳健型策略生成
    fn generate_balanced_strategy(
        team: &Team,
        roster: &[&Player],
        free_agents: &[FreeAgentInfo],
        position_needs: &HashMap<String, u32>,
        rng: &mut impl Rng,
    ) -> (Vec<TransferTarget>, Vec<SellCandidate>, Vec<String>) {
        let mut targets = Vec::new();
        let mut willing_to_sell = Vec::new();
        let mut reasoning = Vec::new();

        // 计算球队平均能力
        let avg_ability = if !roster.is_empty() {
            roster.iter().map(|p| p.ability as u32).sum::<u32>() / roster.len() as u32
        } else {
            70
        };

        reasoning.push(format!(
            "{}作为稳健型球队，保持竞争力的同时控制成本",
            team.name
        ));

        // 1. 寻找能提升阵容的选手（放宽条件）
        for fa in free_agents.iter().filter(|f| {
            f.player.ability >= 65 && f.player.age <= 30
        }) {
            let position = fa.player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default();

            let position_need = position_needs.get(&position).copied().unwrap_or(0);

            // 放宽位置需求阈值，或者能力高于球队平均
            if position_need < 20 && (fa.player.ability as u32) <= avg_ability {
                continue;
            }

            // 稳健型按身价出价
            let max_offer = fa.market_value;

            targets.push(TransferTarget {
                player_id: fa.player.id,
                player_name: fa.player.game_id.clone(),
                position: position.clone(),
                ability: fa.player.ability,
                potential: fa.player.potential,
                age: fa.player.age,
                current_team_id: None,
                current_team_name: None,
                market_value: fa.market_value,
                max_offer,
                priority: ((position_need / 10) as u8).max(fa.player.ability.saturating_sub(70) / 3),
                reasoning: format!(
                    "性价比选手，能力{}，{}岁，可补强{}位置",
                    fa.player.ability, fa.player.age, position
                ),
            });
        }

        // 2. 出售条件放宽：能力较低、年龄大、或非首发的替补
        let starter_abilities: Vec<u8> = roster.iter()
            .filter(|p| p.starter)
            .map(|p| p.ability)
            .collect();
        let min_starter_ability = starter_abilities.iter().min().copied().unwrap_or(0);

        for player in roster.iter() {
            let should_sell =
                player.ability < 65 ||  // 能力低
                (player.ability < 75 && player.age >= 28) ||  // 老将能力一般
                (!player.starter && player.ability < min_starter_ability.saturating_sub(5)) ||  // 替补且能力差距大
                (player.salary >= 200 && player.ability < 80);  // 高薪低能

            if should_sell {
                let market_value = player.calculate_market_value();
                let min_price = (market_value as f64 * 0.85) as u64;

                let sell_reason = if player.ability < 65 {
                    format!("能力{}较低，不符合球队要求", player.ability)
                } else if player.age >= 28 {
                    format!("{}岁年龄偏大，能力{}，性价比下降", player.age, player.ability)
                } else if !player.starter {
                    "长期替补，可释放薪资空间".to_string()
                } else {
                    "薪资与能力不匹配".to_string()
                };

                willing_to_sell.push(SellCandidate {
                    player_id: player.id,
                    player_name: player.game_id.clone(),
                    position: player.position
                        .map(|p| format!("{:?}", p).to_uppercase())
                        .unwrap_or_default(),
                    ability: player.ability,
                    age: player.age,
                    salary: player.salary,
                    market_value,
                    min_price,
                    urgency: if player.ability < 65 { 7 } else { 4 },
                    reasoning: sell_reason,
                });
            }
        }

        // 生成具体的策略说明
        let weak_positions: Vec<_> = position_needs.iter()
            .filter(|(_, &need)| need >= 50)
            .map(|(pos, _)| pos.clone())
            .collect();

        if !weak_positions.is_empty() {
            reasoning.push(format!("重点补强位置: {}", weak_positions.join("、")));
        }
        if !targets.is_empty() {
            reasoning.push(format!("已锁定{}名潜在引援目标", targets.len()));
        }
        if !willing_to_sell.is_empty() {
            reasoning.push(format!("考虑出售{}名球员以优化阵容", willing_to_sell.len()));
        }

        targets.sort_by(|a, b| b.priority.cmp(&a.priority));
        targets.truncate(5); // 增加目标数量
        willing_to_sell.sort_by(|a, b| b.urgency.cmp(&a.urgency));

        (targets, willing_to_sell, reasoning)
    }

    /// 投机型策略生成
    fn generate_speculator_strategy(
        team: &Team,
        roster: &[&Player],
        free_agents: &[FreeAgentInfo],
        position_needs: &HashMap<String, u32>,
        rng: &mut impl Rng,
    ) -> (Vec<TransferTarget>, Vec<SellCandidate>, Vec<String>) {
        let mut targets = Vec::new();
        let mut willing_to_sell = Vec::new();
        let mut reasoning = Vec::new();

        reasoning.push("寻找被低估的选手，买低卖高赚取差价".to_string());

        // 1. 寻找被低估的年轻选手（潜力高于能力较多）
        for fa in free_agents.iter().filter(|f| {
            f.player.potential as i32 - f.player.ability as i32 >= 10 &&
            f.player.age <= 23
        }) {
            let position = fa.player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default();

            // 投机型不管位置需求，只看潜力差价
            let discount = rng.gen_range(0.75..0.85);
            let max_offer = (fa.market_value as f64 * discount) as u64;

            targets.push(TransferTarget {
                player_id: fa.player.id,
                player_name: fa.player.game_id.clone(),
                position: position.clone(),
                ability: fa.player.ability,
                potential: fa.player.potential,
                age: fa.player.age,
                current_team_id: None,
                current_team_name: None,
                market_value: fa.market_value,
                max_offer,
                priority: (fa.player.potential - fa.player.ability) / 2,
                reasoning: format!(
                    "被低估新星，能力{}但潜力{}，有升值空间",
                    fa.player.ability, fa.player.potential
                ),
            });
        }

        // 2. 出售已达巅峰的选手（能力接近或超过潜力）
        for player in roster.iter().filter(|p| {
            p.ability as i32 >= p.potential as i32 - 5 && p.ability >= 75
        }) {
            let market_value = player.calculate_market_value();
            // 巅峰期要求溢价出售
            let min_price = (market_value as f64 * 1.15) as u64;

            willing_to_sell.push(SellCandidate {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default(),
                ability: player.ability,
                age: player.age,
                salary: player.salary,
                market_value,
                min_price,
                urgency: 6,
                reasoning: "已达巅峰，是出售套现的好时机".to_string(),
            });
        }

        if !targets.is_empty() {
            reasoning.push(format!("发现{}名被低估的潜力股", targets.len()));
        }
        if !willing_to_sell.is_empty() {
            reasoning.push(format!("{}名选手已达巅峰，可考虑高价出售", willing_to_sell.len()));
        }

        targets.sort_by(|a, b| b.priority.cmp(&a.priority));
        targets.truncate(4);

        (targets, willing_to_sell, reasoning)
    }

    /// 重建型策略生成
    fn generate_rebuilding_strategy(
        team: &Team,
        roster: &[&Player],
        free_agents: &[FreeAgentInfo],
        position_needs: &HashMap<String, u32>,
        rng: &mut impl Rng,
    ) -> (Vec<TransferTarget>, Vec<SellCandidate>, Vec<String>) {
        let mut targets = Vec::new();
        let mut willing_to_sell = Vec::new();
        let mut reasoning = Vec::new();

        reasoning.push("全面重建，清洗高薪老将，为年轻选手让路".to_string());

        // 1. 寻找低价年轻选手
        for fa in free_agents.iter().filter(|f| f.player.age <= 22) {
            let position = fa.player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default();

            let position_need = position_needs.get(&position).copied().unwrap_or(0);
            if position_need < 40 {
                continue;
            }

            // 重建期只出低价
            let max_offer = (fa.market_value as f64 * 0.8) as u64;

            targets.push(TransferTarget {
                player_id: fa.player.id,
                player_name: fa.player.game_id.clone(),
                position: position.clone(),
                ability: fa.player.ability,
                potential: fa.player.potential,
                age: fa.player.age,
                current_team_id: None,
                current_team_name: None,
                market_value: fa.market_value,
                max_offer,
                priority: (fa.player.potential / 10) as u8,
                reasoning: format!(
                    "年轻选手，{}岁，可作为重建基石",
                    fa.player.age
                ),
            });
        }

        // 2. 大规模清洗老将和高薪选手
        for player in roster.iter().filter(|p| p.age >= 26 || p.salary >= 100) {
            let market_value = player.calculate_market_value();
            // 重建期愿意打折出售
            let min_price = (market_value as f64 * 0.7) as u64;

            willing_to_sell.push(SellCandidate {
                player_id: player.id,
                player_name: player.game_id.clone(),
                position: player.position
                    .map(|p| format!("{:?}", p).to_uppercase())
                    .unwrap_or_default(),
                ability: player.ability,
                age: player.age,
                salary: player.salary,
                market_value,
                min_price,
                urgency: 8, // 高紧迫度
                reasoning: "重建清洗，低价出售换取薪资空间".to_string(),
            });
        }

        if !willing_to_sell.is_empty() {
            reasoning.push(format!("计划清洗{}名选手", willing_to_sell.len()));
        }

        targets.sort_by(|a, b| b.potential.cmp(&a.potential));
        targets.truncate(4);

        (targets, willing_to_sell, reasoning)
    }

    /// 根据球队战绩自动分配 GM 人格
    pub fn auto_assign_personality(
        avg_ability: f64,
        balance: i64,
        made_playoffs: bool,
        consecutive_no_playoffs: u32,
    ) -> GMPersonality {
        // 连续2赛季未进季后赛 -> 重建
        if consecutive_no_playoffs >= 2 {
            return GMPersonality::Rebuilding;
        }

        // 高能力阵容 + 财力充足 -> 争冠
        if avg_ability >= 82.0 && balance >= 3000 {
            return GMPersonality::Championship;
        }

        // 中等能力但进了季后赛 -> 稳健
        if made_playoffs && avg_ability >= 75.0 {
            return GMPersonality::Balanced;
        }

        // 低预算 -> 青训
        if balance < 1500 {
            return GMPersonality::YouthDevelopment;
        }

        // 默认稳健型
        GMPersonality::Balanced
    }

    // ==================== Mock 选手策略生成 ====================

    /// 为选手生成 Mock 转会策略（无LLM时使用）
    pub fn generate_mock_player_strategy(
        player: &Player,
        current_team: &Team,
        available_teams: &[TeamInfo],
        save_id: &str,
        season_id: u64,
    ) -> PlayerTransferStrategy {
        let mut rng = rand::thread_rng();
        let mut analysis_steps = Vec::new();

        // 计算忠诚度类型和离队阈值
        let (loyalty_type, departure_threshold) = Self::get_loyalty_info(player.loyalty);

        // 创建分析数据快照
        let team_avg_ability = available_teams.iter()
            .find(|t| t.id == current_team.id)
            .map(|t| t.avg_ability)
            .unwrap_or(0.0);

        let analysis_data = AnalysisDataSnapshot {
            player_name: player.game_id.clone(),
            position: player.position
                .map(|p| format!("{:?}", p).to_uppercase())
                .unwrap_or_default(),
            age: player.age,
            ability: player.ability,
            potential: player.potential,
            satisfaction: player.satisfaction,
            loyalty: player.loyalty,
            is_starter: player.is_starter,
            current_salary: player.salary / 10000,  // 转换为万
            contract_end_season: player.contract_end_season,
            team_name: current_team.name.clone(),
            team_avg_ability,
            loyalty_type: loyalty_type.clone(),
            departure_threshold,
        };

        // 1. 判断是否想离队（带分析步骤）
        let (wants_to_leave, departure_reasons, leave_reasoning) =
            Self::evaluate_departure_will_with_steps(player, &loyalty_type, departure_threshold, &mut analysis_steps, &mut rng);

        if !wants_to_leave {
            let mut strategy = PlayerTransferStrategy::not_leaving(player.id, save_id.to_string(), season_id);
            strategy.analysis_data = Some(analysis_data);
            strategy.analysis_steps = analysis_steps;
            return strategy;
        }

        // 2. 生成偏好球队列表
        let (preferred_teams, team_reasoning) =
            Self::generate_preferred_teams(player, available_teams, &mut rng);

        // 3. 计算期望条件（带分析步骤）
        let (expected_salary, expected_min_salary, expected_years, requires_starter) =
            Self::calculate_expectations_with_steps(player, &loyalty_type, &mut analysis_steps, &mut rng);

        // 4. 生成决策置信度
        let decision_confidence = Self::calculate_decision_confidence(player);

        PlayerTransferStrategy {
            id: 0,
            player_id: player.id,
            save_id: save_id.to_string(),
            season_id,
            wants_to_leave,
            decision_confidence,
            departure_reasons,
            leave_reasoning,
            preferred_teams,
            team_preference_reasoning: team_reasoning,
            expected_salary,
            expected_min_salary,
            expected_years,
            requires_starter,
            analysis_data: Some(analysis_data),
            analysis_steps,
            is_mock: true,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 获取忠诚度类型和离队阈值
    fn get_loyalty_info(loyalty: u8) -> (String, u8) {
        let (loyalty_type, departure_threshold) = match loyalty {
            90..=100 => ("忠心耿耿", 20),
            70..=89 => ("忠诚", 35),
            50..=69 => ("中立", 50),
            30..=49 => ("机会主义", 60),
            _ => ("雇佣兵", 70),
        };
        (loyalty_type.to_string(), departure_threshold)
    }

    /// 评估选手离队意愿（带分析步骤）
    fn evaluate_departure_will_with_steps(
        player: &Player,
        loyalty_type: &str,
        departure_threshold: u8,
        analysis_steps: &mut Vec<AnalysisStep>,
        rng: &mut impl Rng,
    ) -> (bool, Vec<DepartureReason>, String) {
        let mut reasons = Vec::new();
        let mut reasoning_parts = Vec::new();

        // 步骤1: 忠诚度分析
        let loyalty_factor = if player.loyalty > 90 {
            analysis_steps.push(AnalysisStep {
                step_name: "忠诚度分析".to_string(),
                data_used: format!("忠诚度: {}", player.loyalty),
                threshold: format!("类型: {} (>90)", loyalty_type),
                result: "极高忠诚，几乎不会主动离队".to_string(),
                impact: "离队概率因子: 2%".to_string(),
            });
            0.02
        } else if player.loyalty > 70 {
            analysis_steps.push(AnalysisStep {
                step_name: "忠诚度分析".to_string(),
                data_used: format!("忠诚度: {}", player.loyalty),
                threshold: format!("类型: {} (70-90)", loyalty_type),
                result: "高忠诚度，倾向留队".to_string(),
                impact: "离队概率因子: 10%".to_string(),
            });
            0.1
        } else if player.loyalty > 40 {
            analysis_steps.push(AnalysisStep {
                step_name: "忠诚度分析".to_string(),
                data_used: format!("忠诚度: {}", player.loyalty),
                threshold: format!("类型: {} (40-70)", loyalty_type),
                result: "中立态度，视情况决定".to_string(),
                impact: "离队概率因子: 40%".to_string(),
            });
            0.4
        } else if player.loyalty > 20 {
            analysis_steps.push(AnalysisStep {
                step_name: "忠诚度分析".to_string(),
                data_used: format!("忠诚度: {}", player.loyalty),
                threshold: format!("类型: {} (20-40)", loyalty_type),
                result: "机会主义，容易被挖角".to_string(),
                impact: "离队概率因子: 70%".to_string(),
            });
            0.7
        } else {
            analysis_steps.push(AnalysisStep {
                step_name: "忠诚度分析".to_string(),
                data_used: format!("忠诚度: {}", player.loyalty),
                threshold: format!("类型: {} (<20)", loyalty_type),
                result: "雇佣兵心态，哪里给钱去哪里".to_string(),
                impact: "离队概率因子: 100%".to_string(),
            });
            1.0
        };

        // 步骤2: 满意度分析
        let satisfaction_factor = if player.satisfaction < 30 {
            reasoning_parts.push(format!("满意度仅{}，非常不满", player.satisfaction));
            analysis_steps.push(AnalysisStep {
                step_name: "满意度分析".to_string(),
                data_used: format!("满意度: {}", player.satisfaction),
                threshold: format!("离队阈值: {} (基于忠诚度)", departure_threshold),
                result: format!("满意度{}远低于阈值{}，极度不满", player.satisfaction, departure_threshold),
                impact: "满意度因子: 100%".to_string(),
            });
            1.0
        } else if player.satisfaction < 50 {
            reasoning_parts.push(format!("满意度{}，有些不满", player.satisfaction));
            analysis_steps.push(AnalysisStep {
                step_name: "满意度分析".to_string(),
                data_used: format!("满意度: {}", player.satisfaction),
                threshold: format!("离队阈值: {} (基于忠诚度)", departure_threshold),
                result: if player.satisfaction < departure_threshold {
                    format!("满意度{}低于阈值{}，有离队倾向", player.satisfaction, departure_threshold)
                } else {
                    format!("满意度{}达到阈值{}，但仍有不满", player.satisfaction, departure_threshold)
                },
                impact: "满意度因子: 70%".to_string(),
            });
            0.7
        } else if player.satisfaction < 70 {
            analysis_steps.push(AnalysisStep {
                step_name: "满意度分析".to_string(),
                data_used: format!("满意度: {}", player.satisfaction),
                threshold: format!("离队阈值: {} (基于忠诚度)", departure_threshold),
                result: format!("满意度{}高于阈值{}，基本满意", player.satisfaction, departure_threshold),
                impact: "满意度因子: 30%".to_string(),
            });
            0.3
        } else {
            analysis_steps.push(AnalysisStep {
                step_name: "满意度分析".to_string(),
                data_used: format!("满意度: {}", player.satisfaction),
                threshold: format!("离队阈值: {} (基于忠诚度)", departure_threshold),
                result: format!("满意度{}很高，对现状满意", player.satisfaction),
                impact: "满意度因子: 10%".to_string(),
            });
            0.1
        };

        // 步骤3: 具体原因分析
        // 替补想首发
        if !player.is_starter && player.ability >= 70 && player.satisfaction < 60 {
            reasons.push(DepartureReason::LackOfPlaytime);
            reasoning_parts.push("作为替补缺乏出场机会".to_string());
            analysis_steps.push(AnalysisStep {
                step_name: "上场机会分析".to_string(),
                data_used: format!("首发: {}, 能力: {}, 满意度: {}",
                    if player.is_starter { "是" } else { "否" },
                    player.ability, player.satisfaction),
                threshold: "条件: 非首发 + 能力>=70 + 满意度<60".to_string(),
                result: "符合条件，认为缺乏上场机会".to_string(),
                impact: "新增离队原因: 缺乏上场时间".to_string(),
            });
        }

        // 高能力选手想争冠
        if player.ability >= 85 && player.age >= 25 && player.satisfaction < 50 {
            reasons.push(DepartureReason::SeekingChampionship);
            reasoning_parts.push("能力出众但球队表现不佳，渴望争夺冠军".to_string());
            analysis_steps.push(AnalysisStep {
                step_name: "争冠意愿分析".to_string(),
                data_used: format!("能力: {}, 年龄: {}, 满意度: {}",
                    player.ability, player.age, player.satisfaction),
                threshold: "条件: 能力>=85 + 年龄>=25 + 满意度<50".to_string(),
                result: "符合条件，巅峰期选手渴望冠军".to_string(),
                impact: "新增离队原因: 追求冠军".to_string(),
            });
        }

        // 年轻选手想机会
        if player.age <= 22 && !player.is_starter && player.satisfaction < 60 {
            reasons.push(DepartureReason::SeekingOpportunity);
            reasoning_parts.push("年轻选手渴望更多发展机会".to_string());
            analysis_steps.push(AnalysisStep {
                step_name: "发展机会分析".to_string(),
                data_used: format!("年龄: {}, 首发: {}, 满意度: {}",
                    player.age,
                    if player.is_starter { "是" } else { "否" },
                    player.satisfaction),
                threshold: "条件: 年龄<=22 + 非首发 + 满意度<60".to_string(),
                result: "符合条件，年轻选手需要成长空间".to_string(),
                impact: "新增离队原因: 寻求发展机会".to_string(),
            });
        }

        // 步骤4: 综合概率计算
        let base_prob = satisfaction_factor * loyalty_factor;
        let loyalty_modifier = if player.loyalty > 80 { 0.3 } else if player.loyalty > 60 { 0.6 } else { 1.0 };
        let reason_bonus = reasons.len() as f64 * 0.1 * loyalty_modifier;
        let final_prob = (base_prob + reason_bonus).min(0.85);

        analysis_steps.push(AnalysisStep {
            step_name: "综合概率计算".to_string(),
            data_used: format!("满意度因子: {:.0}%, 忠诚度因子: {:.0}%, 原因数: {}",
                satisfaction_factor * 100.0, loyalty_factor * 100.0, reasons.len()),
            threshold: format!("忠诚度修正系数: {:.1}", loyalty_modifier),
            result: format!("基础概率 = {:.0}% × {:.0}% = {:.1}%",
                satisfaction_factor * 100.0, loyalty_factor * 100.0, base_prob * 100.0),
            impact: format!("最终离队概率: {:.1}% (上限85%)", final_prob * 100.0),
        });

        let wants_to_leave = rng.gen::<f64>() < final_prob;

        // 步骤5: 最终决策
        let leave_reasoning = if wants_to_leave {
            analysis_steps.push(AnalysisStep {
                step_name: "最终决策".to_string(),
                data_used: format!("随机数命中概率 {:.1}%", final_prob * 100.0),
                threshold: "".to_string(),
                result: "决定离队".to_string(),
                impact: format!("离队原因: {}", if reasoning_parts.is_empty() {
                    "寻找新的挑战".to_string()
                } else {
                    reasoning_parts.join("；")
                }),
            });
            if reasoning_parts.is_empty() {
                "经过深思熟虑，决定寻找新的挑战".to_string()
            } else {
                reasoning_parts.join("；")
            }
        } else {
            let stay_reason = if player.loyalty > 80 {
                "对球队忠诚度很高，愿意继续效力"
            } else if player.satisfaction > 70 {
                "对当前状况非常满意，没有离队意愿"
            } else {
                "综合考虑后决定留队"
            };
            analysis_steps.push(AnalysisStep {
                step_name: "最终决策".to_string(),
                data_used: format!("随机数未命中概率 {:.1}%", final_prob * 100.0),
                threshold: "".to_string(),
                result: "决定留队".to_string(),
                impact: stay_reason.to_string(),
            });
            stay_reason.to_string()
        };

        (wants_to_leave, reasons, leave_reasoning)
    }

    /// 计算期望条件（带分析步骤）
    fn calculate_expectations_with_steps(
        player: &Player,
        loyalty_type: &str,
        analysis_steps: &mut Vec<AnalysisStep>,
        rng: &mut impl Rng,
    ) -> (u64, u64, u8, bool) {
        // 基于当前薪资计算期望（salary 存储单位是元，转换为万元）
        let base_salary_wan = (player.salary / 10000).max(50); // 最低50万

        // 期望薪资：根据忠诚度调整
        let (expected_salary, salary_change) = match player.loyalty {
            l if l > 70 => (base_salary_wan, "不要求涨薪"),
            l if l > 40 => ((base_salary_wan as f64 * 1.1) as u64, "要求涨薪10%"),
            l if l > 20 => ((base_salary_wan as f64 * 1.2) as u64, "要求涨薪20%"),
            _ => ((base_salary_wan as f64 * 1.3) as u64, "要求涨薪30%"),
        };

        // 最低接受薪资
        let (min_ratio, min_desc) = match player.loyalty {
            l if l > 70 => (0.7, "可接受降薪30%"),
            l if l > 40 => (0.85, "可接受降薪15%"),
            _ => (0.95, "几乎不接受降薪"),
        };
        let expected_min_salary = (expected_salary as f64 * min_ratio) as u64;

        analysis_steps.push(AnalysisStep {
            step_name: "期望薪资计算".to_string(),
            data_used: format!("当前薪资: {}万/年, 忠诚度类型: {}", base_salary_wan, loyalty_type),
            threshold: format!("涨薪规则: 忠诚>70不涨, >40涨10%, >20涨20%, 其他涨30%"),
            result: format!("{}, 期望薪资: {}万/年", salary_change, expected_salary),
            impact: format!("{}, 最低接受: {}万/年", min_desc, expected_min_salary),
        });

        // 期望合同年限
        let expected_years = match player.age {
            a if a >= 28 => 1,
            a if a >= 25 => rng.gen_range(1..3),
            _ => rng.gen_range(2..4),
        };

        analysis_steps.push(AnalysisStep {
            step_name: "合同年限期望".to_string(),
            data_used: format!("年龄: {}", player.age),
            threshold: "规则: >=28岁短约1年, 25-27岁中约1-2年, <25岁长约2-3年".to_string(),
            result: format!("期望合同年限: {}年", expected_years),
            impact: if player.age >= 28 {
                "老将偏好短约保持灵活".to_string()
            } else if player.age >= 25 {
                "巅峰期寻求稳定".to_string()
            } else {
                "年轻选手愿意长期发展".to_string()
            },
        });

        // 是否要求首发
        let requires_starter = !player.is_starter && player.ability >= 75;

        if requires_starter {
            analysis_steps.push(AnalysisStep {
                step_name: "首发要求分析".to_string(),
                data_used: format!("当前首发: {}, 能力: {}",
                    if player.is_starter { "是" } else { "否" }, player.ability),
                threshold: "条件: 非首发 + 能力>=75".to_string(),
                result: "要求新球队给予首发位置".to_string(),
                impact: "将拒绝无法保证首发的球队".to_string(),
            });
        }

        (expected_salary, expected_min_salary, expected_years, requires_starter)
    }

    /// 生成偏好球队列表
    fn generate_preferred_teams(
        player: &Player,
        available_teams: &[TeamInfo],
        rng: &mut impl Rng,
    ) -> (Vec<PreferredTeam>, String) {
        let mut scored_teams: Vec<(u64, f64, TeamPreferenceReason, String)> = Vec::new();

        for team in available_teams {
            let mut score = 0.0;
            let mut reason = TeamPreferenceReason::ContendingTeam;
            let mut detail = String::new();

            // 1. 争冠机会评分（高能力选手更看重）
            if player.ability >= 85 {
                let championship_score = team.avg_ability / 100.0 * 40.0;
                if championship_score > score {
                    score = championship_score;
                    reason = TeamPreferenceReason::ContendingTeam;
                    detail = format!("球队平均能力{:.1}，有争冠实力", team.avg_ability);
                }
            }

            // 2. 首发机会评分（替补选手更看重）
            if !player.is_starter && team.position_need > 50 {
                let starter_score = team.position_need as f64 / 100.0 * 35.0;
                if starter_score > score {
                    score = starter_score;
                    reason = TeamPreferenceReason::StarterOpportunity;
                    detail = format!("该位置需求度高({}%)，有望获得首发", team.position_need);
                }
            }

            // 3. 薪资能力评分（雇佣兵型选手更看重）
            if player.loyalty < 40 {
                let salary_score = (team.balance as f64 / 10000.0).min(50.0);
                if salary_score > score {
                    score = salary_score;
                    reason = TeamPreferenceReason::HighSalary;
                    detail = format!("球队财务状况良好(余额{}万)，能提供高薪", team.balance / 10000);
                }
            }

            // 添加随机因素
            score += rng.gen::<f64>() * 10.0;

            if score > 20.0 {
                scored_teams.push((team.id, score, reason, detail));
            }
        }

        // 按分数排序
        scored_teams.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 取前5支球队
        let preferred_teams: Vec<PreferredTeam> = scored_teams
            .into_iter()
            .take(5)
            .enumerate()
            .map(|(i, (team_id, _, reason, detail))| {
                let team = available_teams.iter().find(|t| t.id == team_id).unwrap();
                let willing_reduction = match player.loyalty {
                    l if l > 70 => rng.gen_range(0.1..0.25), // 忠诚选手愿意降薪
                    l if l > 40 => rng.gen_range(0.0..0.1),
                    _ => 0.0, // 雇佣兵不接受降薪
                };

                PreferredTeam {
                    team_id,
                    team_name: team.name.clone(),
                    priority: (i + 1) as u8,
                    reason,
                    reason_detail: detail,
                    willing_salary_reduction: willing_reduction,
                    attractiveness_score: (100 - i * 15) as u8,
                }
            })
            .collect();

        let team_reasoning = if preferred_teams.is_empty() {
            "市场上暂无合适的目标球队".to_string()
        } else {
            format!(
                "综合考虑{}等因素，筛选出{}支目标球队",
                if player.ability >= 85 { "争冠机会" } else { "首发机会" },
                preferred_teams.len()
            )
        };

        (preferred_teams, team_reasoning)
    }

    /// 计算期望条件
    fn calculate_expectations(
        player: &Player,
        rng: &mut impl Rng,
    ) -> (u64, u64, u8, bool) {
        // 基于当前薪资计算期望（salary 存储单位是元，转换为万元）
        let base_salary_wan = (player.salary / 10000).max(50); // 最低50万

        // 期望薪资：根据忠诚度调整
        let expected_salary = match player.loyalty {
            l if l > 70 => base_salary_wan,                          // 忠诚选手不要求涨薪
            l if l > 40 => (base_salary_wan as f64 * 1.1) as u64,    // 中立10%涨幅
            l if l > 20 => (base_salary_wan as f64 * 1.2) as u64,    // 机会主义20%涨幅
            _ => (base_salary_wan as f64 * 1.3) as u64,              // 雇佣兵30%涨幅
        };

        // 最低接受薪资
        let min_ratio = match player.loyalty {
            l if l > 70 => 0.7,   // 忠诚选手可接受降薪30%
            l if l > 40 => 0.85,  // 中立可降15%
            _ => 0.95,            // 雇佣兵几乎不降
        };
        let expected_min_salary = (expected_salary as f64 * min_ratio) as u64;

        // 期望合同年限
        let expected_years = match player.age {
            a if a >= 28 => 1,                        // 老将短合同
            a if a >= 25 => rng.gen_range(1..3),     // 巅峰期中等
            _ => rng.gen_range(2..4),                 // 年轻选手长合同
        };

        // 是否要求首发
        let requires_starter = !player.is_starter && player.ability >= 75;

        (expected_salary, expected_min_salary, expected_years, requires_starter)
    }

    /// 计算决策置信度
    fn calculate_decision_confidence(player: &Player) -> u8 {
        // 满意度越极端，置信度越高
        let satisfaction_conf = if player.satisfaction < 30 || player.satisfaction > 80 {
            90
        } else if player.satisfaction < 50 || player.satisfaction > 60 {
            70
        } else {
            50
        };

        // 忠诚度越极端，置信度越高
        let loyalty_conf = if player.loyalty < 20 || player.loyalty > 80 {
            90
        } else {
            60
        };

        ((satisfaction_conf + loyalty_conf) / 2).min(95).max(40)
    }
}
