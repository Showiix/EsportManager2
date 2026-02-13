#[cfg(test)]
mod test_helpers {
    use crate::models::{Player, PlayerStatus, PlayerTag, Position, Team};

    pub fn create_test_player(
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
            growth_accumulator: 0.0,
        }
    }

    pub fn create_test_player_with_contract(
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
            growth_accumulator: 0.0,
        }
    }

    pub fn create_test_team(id: u64) -> Team {
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
            brand_value: 50.0,
        }
    }

    pub fn create_poor_team(id: u64) -> Team {
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
            balance: 100,
            brand_value: 30.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::test_helpers::*;
    use crate::models::{
        ContractExpireDetail, PlayerDeclineDetail, PlayerGrowthDetail, PlayerStatus, PlayerTag,
        Position, RetirementReason,
    };

    #[test]
    fn test_player_growth_genius() {
        let engine = EventEngine::new();
        let young_genius = create_test_player(1, 20, 70, 95, PlayerTag::Genius);
        let growth = engine.calculate_player_growth(&young_genius);

        assert!(growth.is_some());
        let growth = growth.unwrap();
        assert_eq!(growth.growth_amount, 3);
        assert_eq!(growth.new_ability, 73);
    }

    #[test]
    fn test_player_growth_normal() {
        let engine = EventEngine::new();
        let normal_player = create_test_player(2, 22, 75, 90, PlayerTag::Normal);
        let growth = engine.calculate_player_growth(&normal_player);

        assert!(growth.is_some());
        assert_eq!(growth.unwrap().growth_amount, 2);
    }

    #[test]
    fn test_player_growth_ordinary() {
        let engine = EventEngine::new();
        let ordinary_player = create_test_player(3, 21, 65, 80, PlayerTag::Ordinary);
        let growth = engine.calculate_player_growth(&ordinary_player);

        assert!(growth.is_some());
        assert_eq!(growth.unwrap().growth_amount, 1);
    }

    #[test]
    fn test_player_growth_capped_by_potential() {
        let engine = EventEngine::new();
        let near_max = create_test_player(1, 22, 88, 90, PlayerTag::Genius);
        let growth = engine.calculate_player_growth(&near_max);

        assert!(growth.is_some());
        let growth = growth.unwrap();
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
        assert!(growth.is_none());
    }

    #[test]
    fn test_retired_player_no_growth() {
        let engine = EventEngine::new();
        let mut retired = create_test_player(1, 22, 70, 90, PlayerTag::Genius);
        retired.status = PlayerStatus::Retired;

        let growth = engine.calculate_player_growth(&retired);
        assert!(growth.is_none());
    }

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

        let mut decline_count = 0;
        for _ in 0..20 {
            if engine.calculate_player_decline(&aging_player).is_some() {
                decline_count += 1;
            }
        }
        assert!(
            decline_count > 0,
            "expected at least 1 decline in 20 attempts, got {}",
            decline_count
        );
    }

    #[test]
    fn test_player_decline_increases_with_age() {
        let engine = EventEngine::new();

        let mut decline_30_total = 0u32;
        let mut decline_34_total = 0u32;

        for _ in 0..1000 {
            let player_30 = create_test_player(1, 30, 80, 85, PlayerTag::Normal);
            if let Some(d) = engine.calculate_player_decline(&player_30) {
                decline_30_total += d.decline_amount as u32;
            }

            let player_34 = create_test_player(2, 34, 80, 85, PlayerTag::Normal);
            if let Some(d) = engine.calculate_player_decline(&player_34) {
                decline_34_total += d.decline_amount as u32;
            }
        }

        assert!(
            decline_34_total > decline_30_total,
            "34yo: {}, 30yo: {}",
            decline_34_total,
            decline_30_total
        );
    }

    #[test]
    fn test_retired_player_no_decline() {
        let engine = EventEngine::new();
        let mut retired = create_test_player(1, 32, 70, 75, PlayerTag::Normal);
        retired.status = PlayerStatus::Retired;

        let decline = engine.calculate_player_decline(&retired);
        assert!(decline.is_none());
    }

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
        assert!(reason.is_none());
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
        for _ in 0..1000 {
            let detail = engine.process_contract_expiration(&star_player, &team, 5);
            if detail.renewed {
                renewed_count += 1;
            }
        }

        assert!(
            renewed_count > 500,
            "renewed {} times out of 1000",
            renewed_count
        );
    }

    #[test]
    fn test_contract_renewal_poor_team() {
        let engine = EventEngine::new();
        let poor_team = create_poor_team(1);
        let player = create_test_player_with_contract(1, 26, 75, Some(5));

        let mut renewed_count = 0;
        for _ in 0..1000 {
            let detail = engine.process_contract_expiration(&player, &poor_team, 5);
            if detail.renewed {
                renewed_count += 1;
            }
        }

        assert!(
            renewed_count < 750,
            "renewed {} times out of 1000",
            renewed_count
        );
    }

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

        assert!(mature_aging.new_stability >= young_aging.new_stability);
    }

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
            assert!(
                rookie.ability >= 59 && rookie.ability <= 67,
                "ability {} out of range 59-67",
                rookie.ability
            );
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

    #[test]
    fn test_season_settlement_basic() {
        let engine = EventEngine::new();
        let team = create_test_team(1);
        let players = vec![
            create_test_player(1, 22, 70, 90, PlayerTag::Normal),
            create_test_player(2, 32, 75, 80, PlayerTag::Normal),
            create_test_player(3, 36, 65, 70, PlayerTag::Normal),
        ];

        let result = engine.process_season_settlement(1, &players, &[team], 5);

        assert!(!result.growth_events.is_empty());
        assert!(!result.decline_events.is_empty());
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

        assert_eq!(result.age_updates.len(), 2);
        assert!(result
            .age_updates
            .iter()
            .any(|(id, age, _)| *id == 1 && *age == 23));
        assert!(result
            .age_updates
            .iter()
            .any(|(id, age, _)| *id == 2 && *age == 31));
    }

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

        assert_eq!(player.contract_end_season, Some(8));
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

        assert!(player.team_id.is_none());
        assert!(player.contract_end_season.is_none());
        assert!(!player.is_starter);
    }
}
