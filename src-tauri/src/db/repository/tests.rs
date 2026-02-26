use super::*;
use crate::models::*;
use sqlx::{Pool, Sqlite};

const BASELINE_SQL: &str = include_str!("../../../migrations/000_baseline.sql");

async fn setup_test_db() -> Pool<Sqlite> {
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    for stmt in BASELINE_SQL.split(';') {
        let trimmed = stmt.trim();
        if trimmed.is_empty() || trimmed.lines().all(|l| l.trim().starts_with("--") || l.trim().is_empty()) {
            continue;
        }
        let result = sqlx::query(trimmed).execute(&pool).await;
        if let Err(e) = result {
            let is_index = trimmed.to_uppercase().contains("CREATE INDEX");
            if !is_index {
                panic!("Failed to execute baseline statement: {}\nSQL: {}", e, &trimmed[..trimmed.len().min(120)]);
            }
        }
    }

    pool
}

fn make_save(name: &str) -> Save {
    Save::new(name.to_string())
}

fn make_team(region_id: u64) -> Team {
    Team {
        id: 0,
        region_id,
        name: "Test Team".to_string(),
        short_name: Some("TT".to_string()),
        power_rating: 75.0,
        total_matches: 0,
        wins: 0,
        win_rate: 0.0,
        annual_points: 0,
        cross_year_points: 0,
        balance: 30_000_000,
        brand_value: 60.0,
    }
}

fn make_player(team_id: u64) -> Player {
    Player {
        id: 0,
        game_id: "TestPlayer".to_string(),
        real_name: Some("Test Real Name".to_string()),
        nationality: Some("CN".to_string()),
        age: 22,
        ability: 75,
        potential: 85,
        stability: 70,
        tag: PlayerTag::Normal,
        status: PlayerStatus::Active,
        position: Some(Position::Mid),
        team_id: Some(team_id),
        salary: 5_000_000,
        market_value: 30_000_000,
        calculated_market_value: 0,
        contract_end_season: Some(3),
        join_season: 1,
        retire_season: None,
        is_starter: true,
        loyalty: 50,
        satisfaction: 50,
        growth_accumulator: 0.0,
    }
}

// ==================== Save CRUD ====================

#[tokio::test]
async fn test_save_create_and_get() {
    let pool = setup_test_db().await;
    let save = make_save("Test Save");
    let id = SaveRepository::create(&pool, &save).await.unwrap();
    assert_eq!(id, save.id);

    let fetched = SaveRepository::get_by_id(&pool, &id).await.unwrap();
    assert_eq!(fetched.name, "Test Save");
    assert_eq!(fetched.current_season, 1);
}

#[tokio::test]
async fn test_save_get_all() {
    let pool = setup_test_db().await;
    let s1 = make_save("Save A");
    let s2 = make_save("Save B");
    SaveRepository::create(&pool, &s1).await.unwrap();
    SaveRepository::create(&pool, &s2).await.unwrap();

    let all = SaveRepository::get_all(&pool).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_save_update() {
    let pool = setup_test_db().await;
    let mut save = make_save("Original");
    SaveRepository::create(&pool, &save).await.unwrap();

    save.name = "Updated".to_string();
    save.current_season = 5;
    SaveRepository::update(&pool, &save).await.unwrap();

    let fetched = SaveRepository::get_by_id(&pool, &save.id).await.unwrap();
    assert_eq!(fetched.name, "Updated");
    assert_eq!(fetched.current_season, 5);
}

#[tokio::test]
async fn test_save_delete() {
    let pool = setup_test_db().await;
    let save = make_save("To Delete");
    SaveRepository::create(&pool, &save).await.unwrap();

    SaveRepository::delete(&pool, &save.id).await.unwrap();

    let result = SaveRepository::get_by_id(&pool, &save.id).await;
    assert!(result.is_err());
}

// ==================== Team ====================

#[tokio::test]
async fn test_team_create_and_get() {
    let pool = setup_test_db().await;
    let save = make_save("Team Test");
    SaveRepository::create(&pool, &save).await.unwrap();

    // Insert a region first
    sqlx::query("INSERT INTO regions (save_id, name, short_name) VALUES (?, ?, ?)")
        .bind(&save.id)
        .bind("LPL")
        .bind("LPL")
        .execute(&pool)
        .await
        .unwrap();
    let region_id: u64 = 1;

    let team = make_team(region_id);
    let team_id = TeamRepository::create(&pool, &save.id, &team).await.unwrap();
    assert!(team_id > 0);

    let fetched = TeamRepository::get_by_id(&pool, team_id).await.unwrap();
    assert_eq!(fetched.name, "Test Team");
    assert_eq!(fetched.balance, 30_000_000);
    assert!((fetched.brand_value - 60.0).abs() < 0.01);
}

#[tokio::test]
async fn test_team_get_by_region() {
    let pool = setup_test_db().await;
    let save = make_save("Region Test");
    SaveRepository::create(&pool, &save).await.unwrap();

    sqlx::query("INSERT INTO regions (save_id, name, short_name) VALUES (?, ?, ?)")
        .bind(&save.id)
        .bind("LPL")
        .bind("LPL")
        .execute(&pool)
        .await
        .unwrap();

    let mut t1 = make_team(1);
    t1.name = "Team Alpha".to_string();
    t1.power_rating = 80.0;
    let mut t2 = make_team(1);
    t2.name = "Team Beta".to_string();
    t2.power_rating = 70.0;

    TeamRepository::create(&pool, &save.id, &t1).await.unwrap();
    TeamRepository::create(&pool, &save.id, &t2).await.unwrap();

    let teams = TeamRepository::get_by_region(&pool, &save.id, 1).await.unwrap();
    assert_eq!(teams.len(), 2);
    // Should be ordered by power_rating DESC
    assert_eq!(teams[0].name, "Team Alpha");
    assert_eq!(teams[1].name, "Team Beta");
}

// ==================== Player ====================

#[tokio::test]
async fn test_player_create_and_get() {
    let pool = setup_test_db().await;
    let save = make_save("Player Test");
    SaveRepository::create(&pool, &save).await.unwrap();

    sqlx::query("INSERT INTO regions (save_id, name, short_name) VALUES (?, ?, ?)")
        .bind(&save.id)
        .bind("LPL")
        .bind("LPL")
        .execute(&pool)
        .await
        .unwrap();

    let team = make_team(1);
    let team_id = TeamRepository::create(&pool, &save.id, &team).await.unwrap();

    let player = make_player(team_id);
    let player_id = PlayerRepository::create(&pool, &save.id, &player).await.unwrap();
    assert!(player_id > 0);

    let fetched = PlayerRepository::get_by_id(&pool, player_id).await.unwrap();
    assert_eq!(fetched.game_id, "TestPlayer");
    assert_eq!(fetched.ability, 75);
    assert_eq!(fetched.age, 22);
}

#[tokio::test]
async fn test_player_get_by_team() {
    let pool = setup_test_db().await;
    let save = make_save("Player Team Test");
    SaveRepository::create(&pool, &save).await.unwrap();

    sqlx::query("INSERT INTO regions (save_id, name, short_name) VALUES (?, ?, ?)")
        .bind(&save.id)
        .bind("LPL")
        .bind("LPL")
        .execute(&pool)
        .await
        .unwrap();

    let team = make_team(1);
    let team_id = TeamRepository::create(&pool, &save.id, &team).await.unwrap();

    let mut p1 = make_player(team_id);
    p1.game_id = "Player1".to_string();
    p1.position = Some(Position::Top);
    let mut p2 = make_player(team_id);
    p2.game_id = "Player2".to_string();
    p2.position = Some(Position::Jug);

    PlayerRepository::create(&pool, &save.id, &p1).await.unwrap();
    PlayerRepository::create(&pool, &save.id, &p2).await.unwrap();

    let players = PlayerRepository::get_by_team(&pool, team_id).await.unwrap();
    assert_eq!(players.len(), 2);
}
