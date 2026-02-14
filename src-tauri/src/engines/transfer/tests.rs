use super::utils::probabilistic_round;
use super::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet};

// ==================== Helper ====================

fn seeded_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

fn make_engine() -> TransferEngine {
    TransferEngine::new()
}

fn make_cached_player(id: i64, ability: i64, age: i64, position: &str) -> CachedPlayer {
    CachedPlayer {
        id,
        game_id: format!("Player{}", id),
        ability,
        potential: 80,
        age,
        salary: 5_000_000,
        loyalty: 50,
        satisfaction: 60,
        position: position.to_string(),
        tag: "NORMAL".to_string(),
        team_id: Some(1),
        is_starter: true,
        home_region_id: Some(1),
        region_loyalty: 70,
        contract_end_season: Some(3),
        status: "Active".to_string(),
        stability: 70,
        growth_accumulator: 0.0,
        contract_role: "Starter".to_string(),
        season_games_played: 0,
        season_games_total: 0,
    }
}

fn empty_cache() -> TransferCache {
    TransferCache {
        team_names: HashMap::new(),
        team_balances: HashMap::new(),
        team_region_ids: HashMap::new(),
        team_rosters: HashMap::new(),
        team_personalities: HashMap::new(),
        player_recent_honors: HashSet::new(),
        team_annual_ranks: HashMap::new(),
        team_last_season_ranks: HashMap::new(),
        team_reputations: HashMap::new(),
        renewal_failed_pairs: HashSet::new(),
        team_spring_ranks: HashMap::new(),
        team_summer_ranks: HashMap::new(),
        player_season_stats: HashMap::new(),
    }
}

#[test]
fn test_probabilistic_round_integer_returns_self() {
    let mut rng = seeded_rng(42);
    assert_eq!(probabilistic_round(3.0, &mut rng), 3);
    assert_eq!(probabilistic_round(0.0, &mut rng), 0);
    assert_eq!(probabilistic_round(-2.0, &mut rng), -2);
}

#[test]
fn test_probabilistic_round_fractional_averages_correctly() {
    let mut rng = seeded_rng(42);
    let n = 10_000;
    let sum: i64 = (0..n)
        .map(|_| probabilistic_round(2.7, &mut rng))
        .collect::<Vec<_>>()
        .iter()
        .sum();
    let mean = sum as f64 / n as f64;
    // Mean should be close to 2.7 (within 0.1)
    assert!((mean - 2.7).abs() < 0.1, "mean was {}", mean);
}

#[test]
fn test_probabilistic_round_only_returns_floor_or_ceil() {
    let mut rng = seeded_rng(123);
    for _ in 0..1000 {
        let result = probabilistic_round(4.3, &mut rng);
        assert!(result == 4 || result == 5, "got {}", result);
    }
}

// ==================== calculate_stability_score ====================

#[test]
fn test_stability_score_champion_stays_champion() {
    let engine = make_engine();
    // 冠军→冠军: change = 0
    assert_eq!(engine.calculate_stability_score(1, 1), 100);
}

#[test]
fn test_stability_score_champion_drops_badly() {
    let engine = make_engine();
    // 冠军→6名: change = 5 (≥4)
    assert_eq!(engine.calculate_stability_score(6, 1), 30);
}

#[test]
fn test_stability_score_midtable_improves() {
    let engine = make_engine();
    // 8名→5名: change = -3 (大幅上升)
    assert_eq!(engine.calculate_stability_score(5, 8), 95);
}

#[test]
fn test_stability_score_bottom_keeps_falling() {
    let engine = make_engine();
    // 10名→13名: change = 3 (下滑)
    assert_eq!(engine.calculate_stability_score(13, 10), 45);
}

#[test]
fn test_stability_score_runner_up_maintains() {
    let engine = make_engine();
    // 亚军→冠军: change = -1
    assert_eq!(engine.calculate_stability_score(1, 2), 95);
}

// ==================== determine_team_strategy ====================

#[test]
fn test_strategy_dynasty_when_stable() {
    let engine = make_engine();
    let (strategy, urgency, _) = engine.determine_team_strategy(95, 1, 80.0, 24.0);
    assert_eq!(strategy, "DYNASTY");
    assert_eq!(urgency, "NONE");
}

#[test]
fn test_strategy_upgrade_aging_roster() {
    let engine = make_engine();
    let (strategy, urgency, reason) = engine.determine_team_strategy(50, 8, 75.0, 27.5);
    assert_eq!(strategy, "UPGRADE");
    assert_eq!(urgency, "MEDIUM");
    assert!(reason.contains("老化"), "reason: {}", reason);
}

#[test]
fn test_strategy_rebuild_when_catastrophic() {
    let engine = make_engine();
    let (strategy, urgency, _) = engine.determine_team_strategy(30, 14, 60.0, 25.0);
    assert_eq!(strategy, "REBUILD");
    assert_eq!(urgency, "HIGH");
}

#[test]
fn test_strategy_maintain_when_decent() {
    let engine = make_engine();
    let (strategy, urgency, _) = engine.determine_team_strategy(75, 4, 78.0, 25.0);
    assert_eq!(strategy, "MAINTAIN");
    assert_eq!(urgency, "LOW");
}

// ==================== calculate_willingness ====================

#[test]
fn test_willingness_same_region_high_offer() {
    let engine = make_engine();
    let mut rng = seeded_rng(42);
    let roster = vec![
        make_cached_player(10, 70, 25, "MID"),
        make_cached_player(11, 68, 24, "TOP"),
    ];
    let w = engine.calculate_willingness(
        75,        // ability
        50,        // loyalty
        26,        // age (巅峰期)
        8_000_000, // offered_salary (高于当前)
        5_000_000, // current_salary
        Some(1),   // home_region_id
        Some(1),   // target_region_id (同赛区)
        70,        // region_loyalty
        2,         // target_team_rank (强队)
        80,        // target_team_reputation
        &roster,
        "JUG",
        None,
        &mut rng,
    );
    assert!(
        w > 60.0,
        "same region high offer should yield high willingness, got {}",
        w
    );
}

#[test]
fn test_willingness_cross_region_penalty() {
    let engine = make_engine();
    let mut rng1 = seeded_rng(42);
    let mut rng2 = seeded_rng(42);
    let roster = vec![make_cached_player(10, 70, 25, "MID")];

    let same_region = engine.calculate_willingness(
        75,
        50,
        26,
        7_000_000,
        5_000_000,
        Some(1),
        Some(1),
        85,
        3,
        70,
        &roster,
        "JUG",
        None,
        &mut rng1,
    );
    let cross_region = engine.calculate_willingness(
        75,
        50,
        26,
        7_000_000,
        5_000_000,
        Some(1),
        Some(2),
        85,
        3,
        70,
        &roster,
        "JUG",
        None,
        &mut rng2,
    );
    assert!(
        cross_region < same_region,
        "cross-region ({}) should be less than same-region ({})",
        cross_region,
        same_region
    );
}

#[test]
fn test_willingness_young_values_development() {
    let engine = make_engine();
    let mut rng1 = seeded_rng(42);
    let mut rng2 = seeded_rng(42);

    // Team with mentor (high-ability veteran at JUG) + strong avg ability
    // The young MID player benefits from development environment
    let roster_with_mentor = vec![
        make_cached_player(10, 78, 28, "JUG"), // veteran mentor at different pos
        make_cached_player(11, 72, 25, "TOP"),
        make_cached_player(12, 70, 24, "ADC"),
        make_cached_player(13, 68, 23, "SUP"),
    ];
    // Weak team with no mentor, low avg ability
    let roster_no_mentor = vec![
        make_cached_player(10, 55, 20, "JUG"),
        make_cached_player(11, 52, 20, "TOP"),
        make_cached_player(12, 50, 19, "ADC"),
        make_cached_player(13, 48, 19, "SUP"),
    ];

    let w_mentor = engine.calculate_willingness(
        65,
        50,
        20,
        4_000_000,
        3_000_000,
        Some(1),
        Some(1),
        50,
        5,
        60,
        &roster_with_mentor,
        "MID",
        None,
        &mut rng1,
    );
    let w_no_mentor = engine.calculate_willingness(
        65,
        50,
        20,
        4_000_000,
        3_000_000,
        Some(1),
        Some(1),
        50,
        12,
        30,
        &roster_no_mentor,
        "MID",
        None,
        &mut rng2,
    );
    assert!(
        w_mentor > w_no_mentor,
        "young player should prefer team with mentor and better environment: {} vs {}",
        w_mentor,
        w_no_mentor
    );
}

// ==================== calculate_match_score ====================

#[test]
fn test_match_score_high_ability_strong_team() {
    let engine = make_engine();
    let weights = AIDecisionWeights::default();
    let roster = vec![make_cached_player(10, 72, 25, "TOP")];
    let score = engine.calculate_match_score(
        85, 25, "MID", &weights, 50_000_000, &roster, 3, 85, 75, "NORMAL",
    );
    assert!(
        score > 60.0,
        "high ability should yield high match score, got {}",
        score
    );
}

#[test]
fn test_match_score_low_ability() {
    let engine = make_engine();
    let weights = AIDecisionWeights::default();
    let roster = vec![make_cached_player(10, 72, 25, "MID")];
    let score = engine.calculate_match_score(
        45, 30, "MID", &weights, 50_000_000, &roster, 3, 50, 60, "ORDINARY",
    );
    assert!(
        score < 50.0,
        "low ability should yield low match score, got {}",
        score
    );
}

#[test]
fn test_match_score_empty_position_high_need() {
    let engine = make_engine();
    let weights = AIDecisionWeights::default();
    // No one at ADC position
    let roster = vec![make_cached_player(10, 72, 25, "MID")];
    let score = engine.calculate_match_score(
        70, 24, "ADC", &weights, 30_000_000, &roster, 5, 75, 70, "NORMAL",
    );
    assert!(
        score > 55.0,
        "empty position should boost match score, got {}",
        score
    );
}

#[test]
fn test_match_score_genius_tag_multiplier() {
    let engine = make_engine();
    let weights = AIDecisionWeights::default();
    let roster = vec![make_cached_player(10, 65, 25, "TOP")];

    let score_genius = engine.calculate_match_score(
        75, 22, "MID", &weights, 30_000_000, &roster, 5, 80, 70, "GENIUS",
    );
    let score_ordinary = engine.calculate_match_score(
        75, 22, "MID", &weights, 30_000_000, &roster, 5, 80, 70, "ORDINARY",
    );
    assert!(
        score_genius > score_ordinary,
        "genius ({}) should score higher than ordinary ({})",
        score_genius,
        score_ordinary
    );
}

// ==================== TransferCache ====================

#[test]
fn test_cache_transfer_player() {
    let mut cache = empty_cache();
    let player = make_cached_player(1, 75, 24, "MID");
    cache.team_rosters.insert(100, vec![player]);
    cache.team_rosters.insert(200, vec![]);

    cache.transfer_player(1, Some(100), Some(200), None);

    assert_eq!(
        cache.get_roster(100).len(),
        0,
        "player should be removed from old team"
    );
    assert_eq!(
        cache.get_roster(200).len(),
        1,
        "player should be added to new team"
    );
    assert_eq!(cache.get_roster(200)[0].team_id, Some(200));
}

#[test]
fn test_cache_transfer_player_with_updates() {
    let mut cache = empty_cache();
    let player = make_cached_player(1, 75, 24, "MID");
    cache.team_rosters.insert(100, vec![player]);
    cache.team_rosters.insert(200, vec![]);

    cache.transfer_player(
        1,
        Some(100),
        Some(200),
        Some(PlayerCacheUpdate {
            salary: Some(10_000_000),
            loyalty: Some(40),
            satisfaction: Some(80),
            contract_end_season: Some(5),
        }),
    );

    let roster = cache.get_roster(200);
    assert_eq!(roster[0].salary, 10_000_000);
    assert_eq!(roster[0].loyalty, 40);
    assert_eq!(roster[0].satisfaction, 80);
    assert_eq!(roster[0].contract_end_season, Some(5));
}

#[test]
fn test_cache_update_balance() {
    let mut cache = empty_cache();
    cache.team_balances.insert(100, 50_000_000);

    cache.update_balance(100, -20_000_000);
    assert_eq!(*cache.team_balances.get(&100).unwrap(), 30_000_000);

    cache.update_balance(100, 10_000_000);
    assert_eq!(*cache.team_balances.get(&100).unwrap(), 40_000_000);
}

#[test]
fn test_cache_retire_player() {
    let mut cache = empty_cache();
    let p1 = make_cached_player(1, 75, 34, "MID");
    let p2 = make_cached_player(2, 70, 25, "TOP");
    cache.team_rosters.insert(100, vec![p1, p2]);

    cache.retire_player(1, Some(100));

    let roster = cache.get_roster(100);
    assert_eq!(roster.len(), 1);
    assert_eq!(roster[0].id, 2);
}

#[test]
fn test_cache_get_team_rank_default() {
    let cache = empty_cache();
    assert_eq!(cache.get_team_rank(999), 99);
}

#[test]
fn test_cache_get_team_reputation_default() {
    let cache = empty_cache();
    assert_eq!(cache.get_team_reputation(999), 30);
}
