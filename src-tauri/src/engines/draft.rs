use crate::models::{DraftOrder, DraftPlayer, DraftResult, Player, PlayerStatus};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 选秀系统引擎
pub struct DraftEngine {
    /// 选秀池
    draft_pool: Vec<DraftPlayer>,
    /// 选秀顺位
    draft_order: Vec<DraftOrder>,
}

impl DraftEngine {
    pub fn new() -> Self {
        Self {
            draft_pool: Vec::new(),
            draft_order: Vec::new(),
        }
    }

    /// 检查是否是选秀年 (每年进行一次)
    pub fn is_draft_year(_season: u32) -> bool {
        // 每年都有选秀
        true
    }

    /// 导入选秀池
    pub fn import_draft_pool(&mut self, players: Vec<DraftPlayer>) {
        self.draft_pool = players;
    }

    /// 生成选秀顺位 (基于夏季赛排名的加权概率算法)
    /// 排名越靠后，获得高顺位的概率越高
    /// 权重公式: weight = summer_rank^2，即排名数字越大（成绩越差）权重越高
    pub fn generate_draft_order(
        &mut self,
        save_id: &str,
        season_id: u64,
        region_id: u64,
        teams: &[(u64, u32)], // (team_id, summer_rank)
    ) -> Vec<DraftOrder> {
        let mut rng = rand::thread_rng();
        let mut draft_orders = Vec::new();

        // 构建候选池: (team_id, summer_rank, weight)
        // weight = summer_rank^2 → 排名越靠后权重越大
        let mut remaining: Vec<(u64, u32, f64)> = teams
            .iter()
            .map(|&(team_id, rank)| (team_id, rank, (rank as f64).powi(2)))
            .collect();

        // 按顺位从第1顺位开始逐个抽取
        for position in 1..=(teams.len() as u32) {
            let total_weight: f64 = remaining.iter().map(|(_, _, w)| w).sum();

            // 轮盘赌选择
            let roll: f64 = rng.gen::<f64>() * total_weight;
            let mut cumulative = 0.0;
            let mut selected_idx = remaining.len() - 1;

            for (idx, (_, _, weight)) in remaining.iter().enumerate() {
                cumulative += weight;
                if roll <= cumulative {
                    selected_idx = idx;
                    break;
                }
            }

            let (team_id, summer_rank, _) = remaining.remove(selected_idx);

            draft_orders.push(DraftOrder {
                id: 0,
                save_id: save_id.to_string(),
                season_id,
                region_id,
                team_id,
                summer_rank,
                draft_position: position,
                lottery_result: Some(format!("第{}名获得第{}顺位", summer_rank, position)),
            });
        }

        // 按选秀顺位排序
        draft_orders.sort_by_key(|o| o.draft_position);
        self.draft_order = draft_orders.clone();

        draft_orders
    }

    /// 执行选秀
    pub fn execute_draft(
        &mut self,
        save_id: &str,
        season_id: u64,
        region_id: u64,
    ) -> Vec<DraftResult> {
        let mut results = Vec::new();

        // 按选秀顺位进行选秀
        for order in &self.draft_order {
            // 找到最高排名的未被选中球员
            if let Some(player) = self
                .draft_pool
                .iter_mut()
                .filter(|p| !p.is_picked && p.region_id == region_id)
                .min_by_key(|p| p.draft_rank)
            {
                player.is_picked = true;
                player.picked_by_team_id = Some(order.team_id);

                results.push(DraftResult {
                    id: 0,
                    save_id: save_id.to_string(),
                    season_id,
                    region_id,
                    draft_player_id: player.id,
                    team_id: order.team_id,
                    pick_number: order.draft_position,
                    player_id: None, // 稍后创建选手时填充
                });
            }
        }

        results
    }

    /// 从选秀球员创建正式选手
    pub fn create_player_from_draft(
        draft_player: &DraftPlayer,
        team_id: u64,
        current_season: u32,
    ) -> Player {
        Player {
            id: 0, // 由数据库生成
            game_id: draft_player.game_id.clone(),
            real_name: draft_player.real_name.clone(),
            nationality: draft_player.nationality.clone(),
            age: draft_player.age,
            ability: draft_player.ability,
            potential: draft_player.potential,
            stability: Player::calculate_stability(draft_player.age),
            tag: draft_player.tag,
            status: PlayerStatus::Active,
            position: draft_player.position,
            team_id: Some(team_id),
            salary: calculate_draft_salary(draft_player.ability, draft_player.potential),
            market_value: 0,                               // 稍后计算
            calculated_market_value: 0,                    // 选秀新秀尚无荣誉加成
            contract_end_season: Some(current_season + 3), // 默认3年合同
            join_season: current_season,
            retire_season: None,
            is_starter: false,
            loyalty: 50,      // 新秀默认忠诚度50
            satisfaction: 50, // 新秀默认满意度50
        }
    }

    /// 获取选秀池
    pub fn get_draft_pool(&self) -> &[DraftPlayer] {
        &self.draft_pool
    }

    /// 获取选秀顺位
    pub fn get_draft_order(&self) -> &[DraftOrder] {
        &self.draft_order
    }
}

/// 计算选秀球员的初始薪资
fn calculate_draft_salary(ability: u8, potential: u8) -> u64 {
    let base = match ability {
        85..=100 => 60,
        75..=84 => 40,
        65..=74 => 25,
        55..=64 => 15,
        _ => 8,
    };

    // 潜力值加成
    let potential_bonus = if potential > ability + 10 {
        20
    } else if potential > ability + 5 {
        10
    } else {
        0
    };

    (base + potential_bonus) as u64 * 10000
}

/// 选秀抽签结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftLotteryResult {
    pub team_id: u64,
    pub team_name: String,
    pub summer_rank: u32,
    pub draft_position: u32,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PlayerTag, Position};

    #[test]
    fn test_is_draft_year() {
        assert!(DraftEngine::is_draft_year(1));
        assert!(DraftEngine::is_draft_year(2));
        assert!(DraftEngine::is_draft_year(3));
    }

    #[test]
    fn test_draft_order_generation() {
        let mut engine = DraftEngine::new();

        let teams: Vec<(u64, u32)> = (1..=14).map(|i| (i as u64, i as u32)).collect();

        let orders = engine.generate_draft_order("save1", 1, 1, &teams);

        // 应该有14个选秀顺位
        assert_eq!(orders.len(), 14);

        // 每个顺位应该唯一
        let positions: std::collections::HashSet<_> =
            orders.iter().map(|o| o.draft_position).collect();
        assert_eq!(positions.len(), 14);
    }

    #[test]
    fn test_draft_order_all_positions_assigned() {
        let mut engine = DraftEngine::new();
        let teams: Vec<(u64, u32)> = (1..=14).map(|i| (i as u64, i as u32)).collect();

        let orders = engine.generate_draft_order("save1", 1, 1, &teams);

        // 验证所有位置1-14都被分配
        for pos in 1..=14 {
            assert!(
                orders.iter().any(|o| o.draft_position == pos),
                "Position {} not assigned",
                pos
            );
        }
    }

    #[test]
    fn test_draft_order_has_lottery_result() {
        let mut engine = DraftEngine::new();
        let teams: Vec<(u64, u32)> = (1..=14).map(|i| (i as u64, i as u32)).collect();
        let orders = engine.generate_draft_order("save1", 1, 1, &teams);

        // 验证每个选秀顺位都有抽签结果描述
        for order in &orders {
            assert!(order.lottery_result.is_some(), "Should have lottery result");
            let result = order.lottery_result.as_ref().unwrap();
            assert!(!result.is_empty(), "Lottery result should not be empty");
        }
    }

    #[test]
    fn test_draft_salary() {
        // 高能力值选手
        assert!(calculate_draft_salary(85, 95) > calculate_draft_salary(70, 80));

        // 高潜力加成
        assert!(calculate_draft_salary(70, 85) > calculate_draft_salary(70, 72));
    }

    #[test]
    fn test_draft_salary_tiers() {
        // 测试各能力档位的薪资
        let salary_elite = calculate_draft_salary(85, 90);
        let salary_good = calculate_draft_salary(75, 80);
        let salary_avg = calculate_draft_salary(65, 70);
        let salary_low = calculate_draft_salary(55, 60);

        assert!(salary_elite > salary_good);
        assert!(salary_good > salary_avg);
        assert!(salary_avg > salary_low);
    }

    #[test]
    fn test_draft_salary_potential_bonus() {
        // 相同能力，高潜力应该薪资更高
        let salary_high_pot = calculate_draft_salary(70, 85); // +15潜力
        let salary_med_pot = calculate_draft_salary(70, 78); // +8潜力
        let salary_low_pot = calculate_draft_salary(70, 72); // +2潜力

        assert!(salary_high_pot > salary_med_pot);
        assert!(salary_med_pot > salary_low_pot);
    }

    #[test]
    fn test_import_draft_pool() {
        let mut engine = DraftEngine::new();

        let players = vec![DraftPlayer {
            id: 1,
            save_id: "save1".to_string(),
            season_id: 1,
            region_id: 1,
            game_id: "Rookie1".to_string(),
            real_name: None,
            nationality: None,
            age: 18,
            ability: 65,
            potential: 80,
            position: Some(Position::Mid),
            tag: PlayerTag::Normal,
            draft_rank: 1,
            is_picked: false,
            picked_by_team_id: None,
        }];

        engine.import_draft_pool(players.clone());

        assert_eq!(engine.get_draft_pool().len(), 1);
        assert_eq!(engine.get_draft_pool()[0].game_id, "Rookie1");
    }

    #[test]
    fn test_create_player_from_draft() {
        let draft_player = DraftPlayer {
            id: 1,
            save_id: "save1".to_string(),
            season_id: 1,
            region_id: 1,
            game_id: "DraftStar".to_string(),
            real_name: Some("Draft Star".to_string()),
            nationality: Some("KR".to_string()),
            age: 18,
            ability: 70,
            potential: 90,
            position: Some(Position::Mid),
            tag: PlayerTag::Genius,
            draft_rank: 1,
            is_picked: false,
            picked_by_team_id: None,
        };

        let player = DraftEngine::create_player_from_draft(&draft_player, 1, 5);

        assert_eq!(player.game_id, "DraftStar");
        assert_eq!(player.age, 18);
        assert_eq!(player.ability, 70);
        assert_eq!(player.potential, 90);
        assert_eq!(player.team_id, Some(1));
        assert_eq!(player.tag, PlayerTag::Genius);
        assert_eq!(player.position, Some(Position::Mid));
        assert_eq!(player.contract_end_season, Some(8)); // 5 + 3
        assert_eq!(player.join_season, 5);
        assert!(!player.is_starter);
    }

    #[test]
    fn test_execute_draft() {
        let mut engine = DraftEngine::new();

        // 设置选秀池
        let players: Vec<DraftPlayer> = (1..=5)
            .map(|i| DraftPlayer {
                id: i,
                save_id: "save1".to_string(),
                season_id: 1,
                region_id: 1,
                game_id: format!("Player{}", i),
                real_name: None,
                nationality: None,
                age: 18,
                ability: 60 + i as u8,
                potential: 80 + i as u8,
                position: Some(Position::Mid),
                tag: PlayerTag::Normal,
                draft_rank: i as u8,
                is_picked: false,
                picked_by_team_id: None,
            })
            .collect();
        engine.import_draft_pool(players);

        // 设置选秀顺位
        let teams: Vec<(u64, u32)> = vec![(100, 1), (101, 2), (102, 3)];
        engine.generate_draft_order("save1", 1, 1, &teams);

        // 执行选秀
        let results = engine.execute_draft("save1", 1, 1);

        // 验证结果
        assert_eq!(results.len(), 3); // 3支球队各选一人
    }

    #[test]
    fn test_weighted_lottery_probability() {
        // 跑 1000 次抽签，验证第14名（垫底）获得状元签的概率远高于第1名
        let mut rank14_got_first = 0;
        let mut rank1_got_first = 0;

        for _ in 0..1000 {
            let mut engine = DraftEngine::new();
            let teams: Vec<(u64, u32)> = (1..=14).map(|i| (i as u64, i as u32)).collect();
            let orders = engine.generate_draft_order("save1", 1, 1, &teams);

            for order in &orders {
                if order.draft_position == 1 {
                    if order.summer_rank == 14 {
                        rank14_got_first += 1;
                    } else if order.summer_rank == 1 {
                        rank1_got_first += 1;
                    }
                }
            }
        }

        // 第14名获得状元签的次数应远超第1名
        assert!(
            rank14_got_first > rank1_got_first * 5,
            "Rank 14 got #1 pick {} times, Rank 1 got {} times",
            rank14_got_first,
            rank1_got_first
        );
        // 第1名也应有机会（非零）
        // 注意: 概率极低 (~0.1%), 1000次可能为0，所以不强制断言非零
    }
}
