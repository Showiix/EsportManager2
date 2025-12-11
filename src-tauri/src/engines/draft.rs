use rand::Rng;
use crate::models::{DraftOrder, DraftPlayer, DraftResult, Player, PlayerStatus};
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

    /// 生成选秀顺位 (基于夏季赛排名的概率算法)
    /// 排名越靠后，获得高顺位的概率越高
    pub fn generate_draft_order(
        &mut self,
        save_id: &str,
        season_id: u64,
        region_id: u64,
        teams: &[(u64, u32)], // (team_id, summer_rank)
    ) -> Vec<DraftOrder> {
        let mut rng = rand::thread_rng();
        let mut draft_orders = Vec::new();
        let mut assigned_positions: Vec<u32> = Vec::new();

        // 获取概率配置
        let weights = get_lottery_weights();

        for &(team_id, summer_rank) in teams {
            // 根据夏季赛排名获取概率分布
            let probs = weights.get(&summer_rank).cloned().unwrap_or_else(|| {
                // 默认概率：排名越后，获得好签位概率越高
                let mut default_probs = vec![0.0; 14];
                let idx = (14 - summer_rank).min(13) as usize;
                default_probs[idx] = 1.0;
                default_probs
            });

            // 根据概率选择签位
            let position = self.weighted_lottery(&probs, &assigned_positions, &mut rng);
            assigned_positions.push(position);

            draft_orders.push(DraftOrder {
                id: 0,
                save_id: save_id.to_string(),
                season_id,
                region_id,
                team_id,
                summer_rank,
                draft_position: position,
                lottery_result: Some(format!(
                    "第{}名获得第{}顺位",
                    summer_rank, position
                )),
            });
        }

        // 按选秀顺位排序
        draft_orders.sort_by_key(|o| o.draft_position);
        self.draft_order = draft_orders.clone();

        draft_orders
    }

    /// 加权随机抽签
    fn weighted_lottery(
        &self,
        probs: &[f64],
        assigned: &[u32],
        rng: &mut impl Rng,
    ) -> u32 {
        let available_probs: Vec<(u32, f64)> = probs
            .iter()
            .enumerate()
            .filter_map(|(idx, &prob)| {
                let position = (idx + 1) as u32;
                if !assigned.contains(&position) && prob > 0.0 {
                    Some((position, prob))
                } else {
                    None
                }
            })
            .collect();

        if available_probs.is_empty() {
            // 如果没有可用的签位，分配第一个未被分配的
            for pos in 1..=14 {
                if !assigned.contains(&pos) {
                    return pos;
                }
            }
            return 1;
        }

        // 归一化概率
        let total: f64 = available_probs.iter().map(|(_, p)| p).sum();
        let normalized: Vec<(u32, f64)> = available_probs
            .iter()
            .map(|(pos, p)| (*pos, p / total))
            .collect();

        // 轮盘赌选择
        let roll: f64 = rng.gen();
        let mut cumulative = 0.0;

        for (position, prob) in &normalized {
            cumulative += prob;
            if roll <= cumulative {
                return *position;
            }
        }

        normalized.last().map(|(p, _)| *p).unwrap_or(1)
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
            if let Some(player) = self.draft_pool
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
            market_value: 0, // 稍后计算
            contract_end_season: Some(current_season + 3), // 默认3年合同
            join_season: current_season,
            retire_season: None,
            is_starter: false,
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
        80..=100 => 80,
        70..=79 => 50,
        60..=69 => 30,
        _ => 15,
    };

    // 潜力值加成
    let potential_bonus = if potential > ability + 10 {
        20
    } else if potential > ability + 5 {
        10
    } else {
        0
    };

    (base + potential_bonus) as u64
}

/// 获取抽签概率配置
fn get_lottery_weights() -> std::collections::HashMap<u32, Vec<f64>> {
    let mut weights = std::collections::HashMap::new();

    // 排名越靠后，获得高顺位（数字小）的概率越高
    // 概率数组：[获得第1顺位概率, 第2顺位概率, ...]

    // 第14名：最高概率获得状元签
    weights.insert(14, vec![0.25, 0.20, 0.15, 0.10, 0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.01, 0.005, 0.003, 0.002]);

    // 第13名
    weights.insert(13, vec![0.20, 0.22, 0.15, 0.10, 0.08, 0.07, 0.06, 0.05, 0.03, 0.02, 0.01, 0.005, 0.003, 0.002]);

    // 第12名
    weights.insert(12, vec![0.15, 0.18, 0.18, 0.12, 0.10, 0.08, 0.06, 0.05, 0.04, 0.02, 0.01, 0.005, 0.003, 0.002]);

    // 第11名
    weights.insert(11, vec![0.12, 0.15, 0.16, 0.15, 0.12, 0.10, 0.08, 0.05, 0.04, 0.02, 0.005, 0.003, 0.002, 0.001]);

    // 第10名
    weights.insert(10, vec![0.10, 0.10, 0.14, 0.15, 0.15, 0.12, 0.10, 0.06, 0.04, 0.02, 0.01, 0.005, 0.003, 0.002]);

    // 第9名
    weights.insert(9, vec![0.08, 0.08, 0.10, 0.12, 0.15, 0.15, 0.12, 0.10, 0.06, 0.02, 0.01, 0.005, 0.003, 0.002]);

    // 第8名
    weights.insert(8, vec![0.05, 0.04, 0.06, 0.10, 0.12, 0.15, 0.16, 0.15, 0.10, 0.05, 0.01, 0.005, 0.003, 0.002]);

    // 第7名
    weights.insert(7, vec![0.03, 0.02, 0.04, 0.08, 0.10, 0.13, 0.15, 0.18, 0.15, 0.08, 0.02, 0.01, 0.005, 0.002]);

    // 第6名
    weights.insert(6, vec![0.015, 0.01, 0.015, 0.05, 0.08, 0.10, 0.14, 0.17, 0.20, 0.13, 0.04, 0.02, 0.01, 0.005]);

    // 第5名
    weights.insert(5, vec![0.005, 0.005, 0.01, 0.02, 0.05, 0.07, 0.10, 0.15, 0.22, 0.20, 0.08, 0.04, 0.02, 0.01]);

    // 第4名
    weights.insert(4, vec![0.002, 0.003, 0.005, 0.01, 0.02, 0.05, 0.08, 0.12, 0.18, 0.25, 0.15, 0.08, 0.04, 0.02]);

    // 第3名
    weights.insert(3, vec![0.001, 0.002, 0.003, 0.005, 0.01, 0.02, 0.05, 0.08, 0.12, 0.20, 0.25, 0.15, 0.08, 0.04]);

    // 第2名
    weights.insert(2, vec![0.0, 0.001, 0.002, 0.003, 0.005, 0.01, 0.02, 0.05, 0.08, 0.15, 0.25, 0.25, 0.12, 0.08]);

    // 第1名：最低概率获得状元签，但保底获得最后顺位
    weights.insert(1, vec![0.0, 0.0, 0.001, 0.002, 0.003, 0.005, 0.01, 0.02, 0.05, 0.10, 0.20, 0.30, 0.20, 0.08]);

    weights
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
        let positions: std::collections::HashSet<_> = orders.iter().map(|o| o.draft_position).collect();
        assert_eq!(positions.len(), 14);
    }

    #[test]
    fn test_draft_order_all_positions_assigned() {
        let mut engine = DraftEngine::new();
        let teams: Vec<(u64, u32)> = (1..=14).map(|i| (i as u64, i as u32)).collect();

        let orders = engine.generate_draft_order("save1", 1, 1, &teams);

        // 验证所有位置1-14都被分配
        for pos in 1..=14 {
            assert!(orders.iter().any(|o| o.draft_position == pos),
                "Position {} not assigned", pos);
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
        let salary_med_pot = calculate_draft_salary(70, 78);  // +8潜力
        let salary_low_pot = calculate_draft_salary(70, 72);  // +2潜力

        assert!(salary_high_pot > salary_med_pot);
        assert!(salary_med_pot > salary_low_pot);
    }

    #[test]
    fn test_import_draft_pool() {
        let mut engine = DraftEngine::new();

        let players = vec![
            DraftPlayer {
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
            },
        ];

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
        let players: Vec<DraftPlayer> = (1..=5).map(|i| DraftPlayer {
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
        }).collect();
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
    fn test_lottery_weights_exist() {
        let weights = get_lottery_weights();

        // 验证所有14个排名都有权重
        for rank in 1..=14 {
            assert!(weights.contains_key(&rank), "Missing weight for rank {}", rank);
        }

        // 验证权重数组长度
        for (rank, probs) in &weights {
            assert_eq!(probs.len(), 14,
                "Rank {} should have 14 probability values, got {}", rank, probs.len());
        }
    }

    #[test]
    fn test_lottery_weights_sum_approximately_one() {
        let weights = get_lottery_weights();

        for (rank, probs) in &weights {
            let sum: f64 = probs.iter().sum();
            // 权重总和应该接近1 (允许小误差)
            assert!((sum - 1.0).abs() < 0.1,
                "Rank {} weights sum to {}, expected ~1.0", rank, sum);
        }
    }
}
