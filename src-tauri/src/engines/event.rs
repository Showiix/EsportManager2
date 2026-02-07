use crate::models::{
    ContractExpireDetail, Player, PlayerAgingResult, PlayerDeclineDetail,
    PlayerGrowthDetail, PlayerRetirementDetail, PlayerStatus, PlayerTag, Position,
    RetirementReason, RookieGenerationDetail, SeasonSettlementResult, Team,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 事件引擎 - 处理赛季结束时的各种事件
pub struct EventEngine {
    config: EventEngineConfig,
}

/// 事件引擎配置
#[derive(Debug, Clone)]
pub struct EventEngineConfig {
    /// 退役年龄阈值
    pub retirement_age: u8,
    /// 低能力退役阈值
    pub low_ability_threshold: u8,
    /// 年龄+能力组合退役的年龄阈值
    pub age_ability_age_threshold: u8,
    /// 年龄+能力组合退役的能力阈值
    pub age_ability_ability_threshold: u8,
    /// 衰退开始年龄
    pub decline_start_age: u8,
    /// 成长停止年龄
    pub growth_stop_age: u8,
    /// 新秀最小年龄
    pub rookie_min_age: u8,
    /// 新秀最大年龄
    pub rookie_max_age: u8,
    /// 合同续约概率基础值
    pub contract_renewal_base_probability: f64,
}

impl Default for EventEngineConfig {
    fn default() -> Self {
        Self {
            retirement_age: 36,
            low_ability_threshold: 50,
            age_ability_age_threshold: 30,
            age_ability_ability_threshold: 60,
            decline_start_age: 30,
            growth_stop_age: 28,
            rookie_min_age: 17,
            rookie_max_age: 19,
            contract_renewal_base_probability: 0.7,
        }
    }
}

impl Default for EventEngine {
    fn default() -> Self {
        Self {
            config: EventEngineConfig::default(),
        }
    }
}

impl EventEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: EventEngineConfig) -> Self {
        Self { config }
    }

    // ==================== 选手能力成长 ====================

    /// 计算选手能力成长
    /// 根据标签决定成长值: 天才+3, 普通+2, 平庸+1
    /// 成长上限为潜力值
    pub fn calculate_player_growth(&self, player: &Player) -> Option<PlayerGrowthDetail> {
        // 超过成长年龄不再成长
        if player.age >= self.config.growth_stop_age {
            return None;
        }

        // 已达到潜力上限不再成长
        if player.ability >= player.potential {
            return None;
        }

        // 退役选手不成长
        if player.status == PlayerStatus::Retired {
            return None;
        }

        let growth_amount = player.tag.growth_per_season();
        let new_ability = (player.ability + growth_amount).min(player.potential);

        if new_ability > player.ability {
            Some(PlayerGrowthDetail {
                player_id: player.id,
                player_name: player.game_id.clone(),
                old_ability: player.ability,
                new_ability,
                growth_amount: new_ability - player.ability,
                tag: format!("{:?}", player.tag),
                reason: format!(
                    "赛季结束能力成长 ({} 标签 +{})",
                    match player.tag {
                        PlayerTag::Genius => "天才",
                        PlayerTag::Normal => "普通",
                        PlayerTag::Ordinary => "平庸",
                    },
                    growth_amount
                ),
            })
        } else {
            None
        }
    }

    // ==================== 选手能力衰退 ====================

    /// 计算选手能力衰退
    /// 30岁以上开始衰退，衰退量随年龄增加
    pub fn calculate_player_decline(&self, player: &Player) -> Option<PlayerDeclineDetail> {
        // 未到衰退年龄
        if player.age < self.config.decline_start_age {
            return None;
        }

        // 退役选手不衰退
        if player.status == PlayerStatus::Retired {
            return None;
        }

        // 计算衰退量: 基于年龄的渐进衰退
        // 30-31岁: 1点
        // 32-33岁: 2点
        // 34-35岁: 3点
        // 36岁以上: 4点
        let decline_amount = match player.age {
            30..=31 => 1,
            32..=33 => 2,
            34..=35 => 3,
            _ => 4,
        };

        // 添加随机因素 (±1)
        let mut rng = rand::thread_rng();
        let random_factor: i8 = rng.gen_range(-1..=1);
        let final_decline = (decline_amount as i8 + random_factor).max(0) as u8;

        if final_decline == 0 {
            return None;
        }

        let new_ability = player.ability.saturating_sub(final_decline);

        Some(PlayerDeclineDetail {
            player_id: player.id,
            player_name: player.game_id.clone(),
            old_ability: player.ability,
            new_ability,
            decline_amount: player.ability - new_ability,
            age: player.age,
            reason: format!("年龄增长导致能力下滑 ({}岁)", player.age),
        })
    }

    // ==================== 选手退役判定 ====================

    /// 判断选手是否应该退役
    pub fn should_retire(&self, player: &Player) -> Option<RetirementReason> {
        // 已退役
        if player.status == PlayerStatus::Retired {
            return None;
        }

        // 年龄过大 (>=36岁)
        if player.age >= self.config.retirement_age {
            return Some(RetirementReason::Age);
        }

        // 能力过低 (<50)
        if player.ability < self.config.low_ability_threshold {
            return Some(RetirementReason::LowAbility);
        }

        // 年龄大且能力低 (>=30岁且能力<60)
        if player.age >= self.config.age_ability_age_threshold
            && player.ability < self.config.age_ability_ability_threshold
        {
            // 添加概率判定，不是一定退役
            let mut rng = rand::thread_rng();
            let retire_probability = 0.3 + (player.age - 30) as f64 * 0.1; // 30岁30%, 35岁80%
            if rng.gen::<f64>() < retire_probability {
                return Some(RetirementReason::AgeAndAbility);
            }
        }

        None
    }

    /// 处理选手退役
    pub fn process_retirement(
        &self,
        player: &Player,
        team: Option<&Team>,
        current_season: u32,
    ) -> Option<PlayerRetirementDetail> {
        let reason = self.should_retire(player)?;

        Some(PlayerRetirementDetail {
            player_id: player.id,
            player_name: player.game_id.clone(),
            team_id: player.team_id,
            team_name: team.map(|t| t.name.clone()),
            final_ability: player.ability,
            age: player.age,
            career_seasons: current_season.saturating_sub(player.join_season),
            reason,
        })
    }

    // ==================== 选手年龄更新 ====================

    /// 更新选手年龄和稳定性
    pub fn update_player_age(&self, player: &Player) -> PlayerAgingResult {
        let new_age = player.age + 1;
        let new_stability = Player::calculate_stability(new_age);

        PlayerAgingResult {
            player_id: player.id,
            player_name: player.game_id.clone(),
            old_age: player.age,
            new_age,
            old_stability: player.stability,
            new_stability,
        }
    }

    // ==================== 新秀生成 ====================

    /// 生成新秀选手
    pub fn generate_rookie(
        &self,
        team: &Team,
        position: Position,
        _season_id: u32,
    ) -> RookieGenerationDetail {
        let mut rng = rand::thread_rng();

        // 生成年龄 (17-19岁)
        let _age = rng.gen_range(self.config.rookie_min_age..=self.config.rookie_max_age);

        // 生成能力值 (44-58，缩放后)
        let ability = rng.gen_range(44..=58);

        // 生成潜力值 (能力值+5 到 能力值+20)
        let potential_bonus = rng.gen_range(5..=20);
        let potential = (ability + potential_bonus).min(75);

        // 生成标签 (天才5%, 普通75%, 平庸20%)
        let tag_roll: f64 = rng.gen();
        let tag = if tag_roll < 0.05 {
            PlayerTag::Genius
        } else if tag_roll < 0.80 {
            PlayerTag::Normal
        } else {
            PlayerTag::Ordinary
        };

        // 生成游戏ID
        let game_id = self.generate_rookie_name(&mut rng);

        RookieGenerationDetail {
            player_id: 0, // 由数据库分配
            player_name: game_id,
            team_id: team.id,
            team_name: team.name.clone(),
            ability,
            potential,
            position: format!("{:?}", position),
            tag: format!("{:?}", tag),
        }
    }

    /// 生成随机新秀名字
    fn generate_rookie_name(&self, rng: &mut impl Rng) -> String {
        let prefixes = [
            "Shadow", "Storm", "Light", "Dark", "Fire", "Ice", "Thunder", "Wind",
            "Dragon", "Phoenix", "Wolf", "Tiger", "Eagle", "Lion", "Bear", "Fox",
            "Cyber", "Neo", "Nova", "Star", "Moon", "Sun", "Sky", "Cloud",
        ];
        let suffixes = [
            "King", "Master", "Lord", "Knight", "Blade", "Fist", "Eye", "Heart",
            "Soul", "Spirit", "Mind", "Force", "Power", "Strike", "Rush", "Flash",
            "X", "Z", "1", "7", "99", "gg", "Pro", "Jr",
        ];

        let prefix = prefixes[rng.gen_range(0..prefixes.len())];
        let suffix = suffixes[rng.gen_range(0..suffixes.len())];

        format!("{}{}", prefix, suffix)
    }

    // ==================== 合同到期处理 ====================

    /// 判断合同是否到期
    pub fn is_contract_expired(&self, player: &Player, current_season: u32) -> bool {
        match player.contract_end_season {
            Some(end_season) => current_season >= end_season,
            None => false,
        }
    }

    /// 计算续约概率
    fn calculate_renewal_probability(&self, player: &Player, team: &Team) -> f64 {
        let mut probability = self.config.contract_renewal_base_probability;

        // 能力值影响: 高能力更容易续约
        if player.ability >= 61 {
            probability += 0.15;
        } else if player.ability >= 54 {
            probability += 0.05;
        } else if player.ability < 47 {
            probability -= 0.2;
        }

        // 年龄影响: 年轻选手更容易续约
        if player.age <= 24 {
            probability += 0.1;
        } else if player.age >= 30 {
            probability -= 0.15;
        }

        // 潜力影响: 高潜力更容易续约
        if player.potential > player.ability + 10 {
            probability += 0.1;
        }

        // 球队财务影响
        if team.balance < 200 {
            probability -= 0.2; // 财务紧张减少续约概率
        }

        probability.clamp(0.1, 0.95)
    }

    /// 计算新合同参数
    fn calculate_new_contract(&self, player: &Player) -> (u32, u64) {
        let mut rng = rand::thread_rng();

        // 合同年限: 基于年龄
        let years = match player.age {
            age if age <= 24 => rng.gen_range(2..=4),
            age if age <= 28 => rng.gen_range(2..=3),
            _ => rng.gen_range(1..=2),
        };

        // 薪资: 基于能力值
        let base_salary = match player.ability {
            68..=100 => rng.gen_range(100..=200),
            62..=67 => rng.gen_range(50..=100),
            55..=61 => rng.gen_range(25..=50),
            47..=54 => rng.gen_range(10..=25),
            _ => rng.gen_range(5..=10),
        };

        (years, base_salary)
    }

    /// 处理合同到期
    pub fn process_contract_expiration(
        &self,
        player: &Player,
        team: &Team,
        _current_season: u32,
    ) -> ContractExpireDetail {
        let mut rng = rand::thread_rng();

        let renewal_probability = self.calculate_renewal_probability(player, team);
        let renewed = rng.gen::<f64>() < renewal_probability;

        let (new_contract_years, new_salary) = if renewed {
            let (years, salary) = self.calculate_new_contract(player);
            (Some(years), Some(salary))
        } else {
            (None, None)
        };

        ContractExpireDetail {
            player_id: player.id,
            player_name: player.game_id.clone(),
            team_id: team.id,
            team_name: team.name.clone(),
            ability: player.ability,
            age: player.age,
            renewed,
            new_contract_years,
            new_salary,
        }
    }

    // ==================== 赛季结算 ====================

    /// 执行完整的赛季结算
    pub fn process_season_settlement(
        &self,
        season_id: u64,
        players: &[Player],
        teams: &[Team],
        current_season: u32,
    ) -> SeasonSettlementResult {
        let mut growth_events = Vec::new();
        let mut decline_events = Vec::new();
        let mut retirement_events = Vec::new();
        let mut contract_expire_events = Vec::new();
        let rookie_events = Vec::new(); // 新秀由选秀系统生成

        // 创建队伍映射
        let team_map: std::collections::HashMap<u64, &Team> =
            teams.iter().map(|t| (t.id, t)).collect();

        for player in players {
            if player.status == PlayerStatus::Retired {
                continue;
            }

            let team = player.team_id.and_then(|tid| team_map.get(&tid).copied());

            // 1. 检查退役
            if let Some(detail) = self.process_retirement(player, team, current_season) {
                retirement_events.push(detail);
                continue; // 退役选手不再处理其他事件
            }

            // 2. 检查合同到期
            if self.is_contract_expired(player, current_season) {
                if let Some(team) = team {
                    let detail = self.process_contract_expiration(player, team, current_season);
                    contract_expire_events.push(detail);
                }
            }

            // 3. 计算能力变化 (先成长后衰退)
            if let Some(growth) = self.calculate_player_growth(player) {
                growth_events.push(growth);
            } else if let Some(decline) = self.calculate_player_decline(player) {
                decline_events.push(decline);
            }
        }

        SeasonSettlementResult {
            season_id,
            season_name: format!("S{}", season_id),
            growth_events,
            decline_events,
            retirement_events,
            contract_expire_events,
            rookie_events,
        }
    }

    /// 应用成长事件到选手
    pub fn apply_growth(&self, player: &mut Player, growth: &PlayerGrowthDetail) {
        player.ability = growth.new_ability;
        // 更新身价
        player.market_value = player.calculate_market_value();
    }

    /// 应用衰退事件到选手
    pub fn apply_decline(&self, player: &mut Player, decline: &PlayerDeclineDetail) {
        player.ability = decline.new_ability;
        // 更新身价
        player.market_value = player.calculate_market_value();
    }

    /// 应用退役事件到选手
    pub fn apply_retirement(&self, player: &mut Player, current_season: u32) {
        player.status = PlayerStatus::Retired;
        player.retire_season = Some(current_season);
        player.team_id = None;
        player.is_starter = false;
    }

    /// 应用年龄更新到选手
    pub fn apply_aging(&self, player: &mut Player, aging: &PlayerAgingResult) {
        player.age = aging.new_age;
        player.stability = aging.new_stability;
    }

    /// 应用合同续约到选手
    pub fn apply_contract_renewal(
        &self,
        player: &mut Player,
        detail: &ContractExpireDetail,
        current_season: u32,
    ) {
        if detail.renewed {
            if let Some(years) = detail.new_contract_years {
                player.contract_end_season = Some(current_season + years);
            }
            if let Some(salary) = detail.new_salary {
                player.salary = salary;
            }
        } else {
            // 未续约，成为自由球员
            player.team_id = None;
            player.is_starter = false;
            player.contract_end_season = None;
        }
    }
}

/// 事件引擎的辅助结构，用于批量处理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonEndProcessResult {
    /// 需要更新的选手ID和新能力值
    pub ability_updates: Vec<(u64, u8)>,
    /// 需要更新的选手ID和新年龄/稳定性
    pub age_updates: Vec<(u64, u8, u8)>,
    /// 需要标记退役的选手ID
    pub retirements: Vec<u64>,
    /// 合同续约信息: (选手ID, 是否续约, 新合同年限, 新薪资)
    pub contract_updates: Vec<(u64, bool, Option<u32>, Option<u64>)>,
    /// 成为自由球员的选手ID
    pub free_agents: Vec<u64>,
}

impl EventEngine {
    /// 批量处理赛季结束事件，返回所有需要的数据库更新
    pub fn batch_process_season_end(
        &self,
        players: &[Player],
        teams: &[Team],
        current_season: u32,
    ) -> SeasonEndProcessResult {
        let mut result = SeasonEndProcessResult {
            ability_updates: Vec::new(),
            age_updates: Vec::new(),
            retirements: Vec::new(),
            contract_updates: Vec::new(),
            free_agents: Vec::new(),
        };

        let team_map: std::collections::HashMap<u64, &Team> =
            teams.iter().map(|t| (t.id, t)).collect();

        for player in players {
            if player.status == PlayerStatus::Retired {
                continue;
            }

            let team = player.team_id.and_then(|tid| team_map.get(&tid).copied());

            // 1. 年龄更新
            let aging = self.update_player_age(player);
            result.age_updates.push((player.id, aging.new_age, aging.new_stability));

            // 使用更新后的年龄进行判断
            let aged_player = Player {
                age: aging.new_age,
                stability: aging.new_stability,
                ..player.clone()
            };

            // 2. 退役判定
            if self.should_retire(&aged_player).is_some() {
                result.retirements.push(player.id);
                continue;
            }

            // 3. 合同到期处理
            if self.is_contract_expired(player, current_season) {
                if let Some(team) = team {
                    let detail = self.process_contract_expiration(&aged_player, team, current_season);
                    result.contract_updates.push((
                        player.id,
                        detail.renewed,
                        detail.new_contract_years,
                        detail.new_salary,
                    ));
                    if !detail.renewed {
                        result.free_agents.push(player.id);
                    }
                }
            }

            // 4. 能力变化
            if let Some(growth) = self.calculate_player_growth(&aged_player) {
                result.ability_updates.push((player.id, growth.new_ability));
            } else if let Some(decline) = self.calculate_player_decline(&aged_player) {
                result.ability_updates.push((player.id, decline.new_ability));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_player(
        id: u64,
        age: u8,
        ability: u8,
        potential: u8,
        tag: PlayerTag,
    ) -> Player {
        Player {
            id,
            game_id: format!("Player{}", id),
            real_name: None,
            nationality: None,
            age,
            ability,
            potential,
            stability: Player::calculate_stability(age),
            tag,
            status: PlayerStatus::Active,
            position: Some(Position::Mid),
            team_id: Some(1),
            salary: 100,
            market_value: 500,
            calculated_market_value: 500,
            contract_end_season: Some(5),
            join_season: 1,
            retire_season: None,
            is_starter: true,
            loyalty: 50,
            satisfaction: 50,
        }
    }

    fn create_test_player_with_contract(
        id: u64,
        age: u8,
        ability: u8,
        contract_end: Option<u32>,
    ) -> Player {
        Player {
            id,
            game_id: format!("Player{}", id),
            real_name: None,
            nationality: None,
            age,
            ability,
            potential: ability + 10,
            stability: Player::calculate_stability(age),
            tag: PlayerTag::Normal,
            status: PlayerStatus::Active,
            position: Some(Position::Mid),
            team_id: Some(1),
            salary: 100,
            market_value: 500,
            calculated_market_value: 500,
            contract_end_season: contract_end,
            join_season: 1,
            retire_season: None,
            is_starter: true,
            loyalty: 50,
            satisfaction: 50,
        }
    }

    fn create_test_team(id: u64) -> Team {
        Team {
            id,
            region_id: 1,
            name: format!("Team{}", id),
            short_name: Some(format!("T{}", id)),
            power_rating: 75.0,
            total_matches: 0,
            wins: 0,
            win_rate: 0.0,
            annual_points: 0,
            cross_year_points: 0,
            balance: 1000,
        }
    }

    fn create_poor_team(id: u64) -> Team {
        Team {
            id,
            region_id: 1,
            name: format!("PoorTeam{}", id),
            short_name: Some(format!("PT{}", id)),
            power_rating: 65.0,
            total_matches: 0,
            wins: 0,
            win_rate: 0.0,
            annual_points: 0,
            cross_year_points: 0,
            balance: 100, // 财务紧张
        }
    }

    // ==================== 成长测试 ====================

    #[test]
    fn test_player_growth_genius() {
        let engine = EventEngine::new();
        let young_genius = create_test_player(1, 20, 70, 95, PlayerTag::Genius);
        let growth = engine.calculate_player_growth(&young_genius);

        assert!(growth.is_some());
        let growth = growth.unwrap();
        assert_eq!(growth.growth_amount, 3); // 天才+3
        assert_eq!(growth.new_ability, 73);
    }

    #[test]
    fn test_player_growth_normal() {
        let engine = EventEngine::new();
        let normal_player = create_test_player(2, 22, 75, 90, PlayerTag::Normal);
        let growth = engine.calculate_player_growth(&normal_player);

        assert!(growth.is_some());
        assert_eq!(growth.unwrap().growth_amount, 2); // 普通+2
    }

    #[test]
    fn test_player_growth_ordinary() {
        let engine = EventEngine::new();
        let ordinary_player = create_test_player(3, 21, 65, 80, PlayerTag::Ordinary);
        let growth = engine.calculate_player_growth(&ordinary_player);

        assert!(growth.is_some());
        assert_eq!(growth.unwrap().growth_amount, 1); // 平庸+1
    }

    #[test]
    fn test_player_growth_capped_by_potential() {
        let engine = EventEngine::new();
        // 能力接近潜力上限
        let near_max = create_test_player(1, 22, 88, 90, PlayerTag::Genius);
        let growth = engine.calculate_player_growth(&near_max);

        assert!(growth.is_some());
        let growth = growth.unwrap();
        // 成长被潜力限制为2点而非3点
        assert_eq!(growth.new_ability, 90);
        assert_eq!(growth.growth_amount, 2);
    }

    #[test]
    fn test_player_growth_at_max_potential() {
        let engine = EventEngine::new();
        let maxed_player = create_test_player(3, 24, 85, 85, PlayerTag::Genius);
        let growth = engine.calculate_player_growth(&maxed_player);
        assert!(growth.is_none());
    }

    #[test]
    fn test_player_growth_over_age_limit() {
        let engine = EventEngine::new();
        let old_player = create_test_player(4, 29, 70, 90, PlayerTag::Genius);
        let growth = engine.calculate_player_growth(&old_player);
        assert!(growth.is_none()); // 28岁以上不再成长
    }

    #[test]
    fn test_retired_player_no_growth() {
        let engine = EventEngine::new();
        let mut retired = create_test_player(1, 22, 70, 90, PlayerTag::Genius);
        retired.status = PlayerStatus::Retired;

        let growth = engine.calculate_player_growth(&retired);
        assert!(growth.is_none());
    }

    // ==================== 衰退测试 ====================

    #[test]
    fn test_player_no_decline_under_30() {
        let engine = EventEngine::new();
        let young_player = create_test_player(1, 29, 80, 90, PlayerTag::Normal);
        let decline = engine.calculate_player_decline(&young_player);
        assert!(decline.is_none());
    }

    #[test]
    fn test_player_decline_at_30() {
        let engine = EventEngine::new();
        let aging_player = create_test_player(2, 30, 80, 85, PlayerTag::Normal);

        // 30岁选手衰退有随机因素，多次测试应该至少有一次衰退
        let mut decline_count = 0;
        for _ in 0..20 {
            if engine.calculate_player_decline(&aging_player).is_some() {
                decline_count += 1;
            }
        }
        assert!(decline_count > 0, "30-year-old player should decline at least sometimes");
    }

    #[test]
    fn test_player_decline_increases_with_age() {
        let engine = EventEngine::new();

        // 收集多次衰退数据以验证趋势
        let mut decline_30_total = 0u32;
        let mut decline_34_total = 0u32;

        for _ in 0..100 {
            let player_30 = create_test_player(1, 30, 80, 85, PlayerTag::Normal);
            if let Some(d) = engine.calculate_player_decline(&player_30) {
                decline_30_total += d.decline_amount as u32;
            }

            let player_34 = create_test_player(2, 34, 80, 85, PlayerTag::Normal);
            if let Some(d) = engine.calculate_player_decline(&player_34) {
                decline_34_total += d.decline_amount as u32;
            }
        }

        // 34岁的平均衰退应该大于30岁
        assert!(decline_34_total > decline_30_total);
    }

    #[test]
    fn test_retired_player_no_decline() {
        let engine = EventEngine::new();
        let mut retired = create_test_player(1, 32, 70, 75, PlayerTag::Normal);
        retired.status = PlayerStatus::Retired;

        let decline = engine.calculate_player_decline(&retired);
        assert!(decline.is_none());
    }

    // ==================== 退役测试 ====================

    #[test]
    fn test_retirement_by_age() {
        let engine = EventEngine::new();
        let old_player = create_test_player(1, 36, 70, 75, PlayerTag::Normal);
        let reason = engine.should_retire(&old_player);
        assert_eq!(reason, Some(RetirementReason::Age));
    }

    #[test]
    fn test_retirement_by_low_ability() {
        let engine = EventEngine::new();
        let low_ability_player = create_test_player(2, 25, 45, 50, PlayerTag::Ordinary);
        let reason = engine.should_retire(&low_ability_player);
        assert_eq!(reason, Some(RetirementReason::LowAbility));
    }

    #[test]
    fn test_no_retirement_normal_player() {
        let engine = EventEngine::new();
        let normal_player = create_test_player(3, 25, 80, 90, PlayerTag::Normal);
        let reason = engine.should_retire(&normal_player);
        assert!(reason.is_none());
    }

    #[test]
    fn test_already_retired_player() {
        let engine = EventEngine::new();
        let mut retired = create_test_player(1, 36, 70, 75, PlayerTag::Normal);
        retired.status = PlayerStatus::Retired;

        let reason = engine.should_retire(&retired);
        assert!(reason.is_none()); // 已退役不再判定
    }

    #[test]
    fn test_process_retirement() {
        let engine = EventEngine::new();
        let old_player = create_test_player(1, 36, 70, 75, PlayerTag::Normal);
        let team = create_test_team(1);

        let detail = engine.process_retirement(&old_player, Some(&team), 10);
        assert!(detail.is_some());

        let detail = detail.unwrap();
        assert_eq!(detail.player_id, old_player.id);
        assert_eq!(detail.age, 36);
        assert_eq!(detail.reason, RetirementReason::Age);
    }

    // ==================== 合同测试 ====================

    #[test]
    fn test_contract_not_expired() {
        let engine = EventEngine::new();
        let player = create_test_player_with_contract(1, 25, 80, Some(5));
        assert!(!engine.is_contract_expired(&player, 3));
        assert!(!engine.is_contract_expired(&player, 4));
    }

    #[test]
    fn test_contract_expired() {
        let engine = EventEngine::new();
        let player = create_test_player_with_contract(1, 25, 80, Some(5));
        assert!(engine.is_contract_expired(&player, 5));
        assert!(engine.is_contract_expired(&player, 6));
    }

    #[test]
    fn test_no_contract_never_expires() {
        let engine = EventEngine::new();
        let player = create_test_player_with_contract(1, 25, 80, None);
        assert!(!engine.is_contract_expired(&player, 100));
    }

    #[test]
    fn test_contract_renewal_high_ability() {
        let engine = EventEngine::new();
        let team = create_test_team(1);
        let star_player = create_test_player_with_contract(1, 24, 85, Some(5));

        let mut renewed_count = 0;
        for _ in 0..100 {
            let detail = engine.process_contract_expiration(&star_player, &team, 5);
            if detail.renewed {
                renewed_count += 1;
            }
        }

        // 高能力年轻选手续约率应该很高
        assert!(renewed_count > 70);
    }

    #[test]
    fn test_contract_renewal_poor_team() {
        let engine = EventEngine::new();
        let poor_team = create_poor_team(1);
        let player = create_test_player_with_contract(1, 26, 75, Some(5));

        let mut renewed_count = 0;
        for _ in 0..100 {
            let detail = engine.process_contract_expiration(&player, &poor_team, 5);
            if detail.renewed {
                renewed_count += 1;
            }
        }

        // 财务紧张的队伍续约率应该较低
        assert!(renewed_count < 70);
    }

    // ==================== 年龄更新测试 ====================

    #[test]
    fn test_age_update() {
        let engine = EventEngine::new();
        let player = create_test_player(1, 24, 80, 90, PlayerTag::Normal);

        let aging = engine.update_player_age(&player);
        assert_eq!(aging.new_age, 25);
        assert_eq!(aging.old_age, 24);
    }

    #[test]
    fn test_stability_increases_with_age() {
        let engine = EventEngine::new();
        let young_player = create_test_player(1, 20, 80, 90, PlayerTag::Normal);
        let mature_player = create_test_player(2, 28, 80, 90, PlayerTag::Normal);

        let young_aging = engine.update_player_age(&young_player);
        let mature_aging = engine.update_player_age(&mature_player);

        // 成熟选手稳定性应该更高
        assert!(mature_aging.new_stability >= young_aging.new_stability);
    }

    // ==================== 新秀生成测试 ====================

    #[test]
    fn test_rookie_generation() {
        let engine = EventEngine::new();
        let team = create_test_team(1);

        let rookie = engine.generate_rookie(&team, Position::Mid, 1);

        assert_eq!(rookie.team_id, team.id);
        assert_eq!(rookie.team_name, team.name);
        assert!(!rookie.player_name.is_empty());
    }

    #[test]
    fn test_rookie_ability_range() {
        let engine = EventEngine::new();
        let team = create_test_team(1);

        for _ in 0..50 {
            let rookie = engine.generate_rookie(&team, Position::Top, 1);
            // 能力值应在55-75范围内
            assert!(rookie.ability >= 44 && rookie.ability <= 58);
            // 潜力应高于能力
            assert!(rookie.potential > rookie.ability);
            assert!(rookie.potential <= 100);
        }
    }

    #[test]
    fn test_rookie_position() {
        let engine = EventEngine::new();
        let team = create_test_team(1);

        let rookie_mid = engine.generate_rookie(&team, Position::Mid, 1);
        assert_eq!(rookie_mid.position, "Mid");

        let rookie_top = engine.generate_rookie(&team, Position::Top, 1);
        assert_eq!(rookie_top.position, "Top");
    }

    // ==================== 赛季结算测试 ====================

    #[test]
    fn test_season_settlement_basic() {
        let engine = EventEngine::new();
        let team = create_test_team(1);
        let players = vec![
            create_test_player(1, 22, 70, 90, PlayerTag::Normal),
            create_test_player(2, 32, 75, 80, PlayerTag::Normal),
            create_test_player(3, 36, 65, 70, PlayerTag::Normal), // 将退役
        ];

        let result = engine.process_season_settlement(1, &players, &[team], 5);

        // 应该有成长事件（22岁选手）
        assert!(!result.growth_events.is_empty());
        // 应该有衰退事件（32岁选手）
        assert!(!result.decline_events.is_empty());
        // 应该有退役事件（36岁选手）
        assert!(!result.retirement_events.is_empty());
    }

    #[test]
    fn test_batch_process_season_end() {
        let engine = EventEngine::new();
        let team = create_test_team(1);
        let players = vec![
            create_test_player(1, 22, 70, 90, PlayerTag::Normal),
            create_test_player(2, 30, 75, 80, PlayerTag::Normal),
        ];

        let result = engine.batch_process_season_end(&players, &[team], 5);

        // 应该有年龄更新
        assert_eq!(result.age_updates.len(), 2);
        // 验证年龄更新正确
        assert!(result.age_updates.iter().any(|(id, age, _)| *id == 1 && *age == 23));
        assert!(result.age_updates.iter().any(|(id, age, _)| *id == 2 && *age == 31));
    }

    // ==================== 应用事件测试 ====================

    #[test]
    fn test_apply_growth() {
        let engine = EventEngine::new();
        let mut player = create_test_player(1, 22, 70, 90, PlayerTag::Normal);
        let growth = PlayerGrowthDetail {
            player_id: 1,
            player_name: "Test".to_string(),
            old_ability: 70,
            new_ability: 72,
            growth_amount: 2,
            tag: "Normal".to_string(),
            reason: "Test".to_string(),
        };

        engine.apply_growth(&mut player, &growth);
        assert_eq!(player.ability, 72);
    }

    #[test]
    fn test_apply_decline() {
        let engine = EventEngine::new();
        let mut player = create_test_player(1, 32, 80, 85, PlayerTag::Normal);
        let decline = PlayerDeclineDetail {
            player_id: 1,
            player_name: "Test".to_string(),
            old_ability: 80,
            new_ability: 78,
            decline_amount: 2,
            age: 32,
            reason: "Test".to_string(),
        };

        engine.apply_decline(&mut player, &decline);
        assert_eq!(player.ability, 78);
    }

    #[test]
    fn test_apply_retirement() {
        let engine = EventEngine::new();
        let mut player = create_test_player(1, 36, 70, 75, PlayerTag::Normal);

        engine.apply_retirement(&mut player, 10);

        assert_eq!(player.status, PlayerStatus::Retired);
        assert_eq!(player.retire_season, Some(10));
        assert!(player.team_id.is_none());
        assert!(!player.is_starter);
    }

    #[test]
    fn test_apply_contract_renewal() {
        let engine = EventEngine::new();
        let mut player = create_test_player_with_contract(1, 25, 80, Some(5));
        let detail = ContractExpireDetail {
            player_id: 1,
            player_name: "Test".to_string(),
            team_id: 1,
            team_name: "Team1".to_string(),
            ability: 80,
            age: 25,
            renewed: true,
            new_contract_years: Some(3),
            new_salary: Some(150),
        };

        engine.apply_contract_renewal(&mut player, &detail, 5);

        assert_eq!(player.contract_end_season, Some(8)); // 5 + 3
        assert_eq!(player.salary, 150);
    }

    #[test]
    fn test_apply_contract_not_renewed() {
        let engine = EventEngine::new();
        let mut player = create_test_player_with_contract(1, 32, 65, Some(5));
        let detail = ContractExpireDetail {
            player_id: 1,
            player_name: "Test".to_string(),
            team_id: 1,
            team_name: "Team1".to_string(),
            ability: 65,
            age: 32,
            renewed: false,
            new_contract_years: None,
            new_salary: None,
        };

        engine.apply_contract_renewal(&mut player, &detail, 5);

        // 成为自由球员
        assert!(player.team_id.is_none());
        assert!(player.contract_end_season.is_none());
        assert!(!player.is_starter);
    }
}
