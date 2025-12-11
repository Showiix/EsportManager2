use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

/// 存档仓库
pub struct SaveRepository;

impl SaveRepository {
    /// 创建新存档
    pub async fn create(
        pool: &Pool<Sqlite>,
        save: &Save,
    ) -> Result<String, DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO saves (id, name, current_season, current_phase, phase_completed, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&save.id)
        .bind(&save.name)
        .bind(save.current_season as i64)
        .bind(format!("{:?}", save.current_phase))
        .bind(save.phase_completed)
        .bind(save.created_at.to_rfc3339())
        .bind(save.updated_at.to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(save.id.clone())
    }

    /// 获取存档
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Save, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM saves WHERE id = ?"
        )
        .bind(save_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .ok_or_else(|| DatabaseError::NotFound(format!("Save {} not found", save_id)))?;

        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        Ok(Save {
            id: row.get("id"),
            name: row.get("name"),
            current_season: row.get::<i64, _>("current_season") as u32,
            current_phase: parse_season_phase(row.get("current_phase")),
            phase_completed: row.get("phase_completed"),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
    }

    /// 获取所有存档
    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<Save>, DatabaseError> {
        let rows = sqlx::query("SELECT * FROM saves ORDER BY updated_at DESC")
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get("created_at");
                let updated_at_str: String = row.get("updated_at");
                Save {
                    id: row.get("id"),
                    name: row.get("name"),
                    current_season: row.get::<i64, _>("current_season") as u32,
                    current_phase: parse_season_phase(row.get("current_phase")),
                    phase_completed: row.get("phase_completed"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                }
            })
            .collect())
    }

    /// 更新存档
    pub async fn update(
        pool: &Pool<Sqlite>,
        save: &Save,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE saves SET
                name = ?,
                current_season = ?,
                current_phase = ?,
                phase_completed = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&save.name)
        .bind(save.current_season as i64)
        .bind(format!("{:?}", save.current_phase))
        .bind(save.phase_completed)
        .bind(save.updated_at.to_rfc3339())
        .bind(&save.id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 删除存档
    pub async fn delete(pool: &Pool<Sqlite>, save_id: &str) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM saves WHERE id = ?")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 队伍仓库
pub struct TeamRepository;

impl TeamRepository {
    /// 创建队伍
    pub async fn create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team: &Team,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO teams (save_id, region_id, name, short_name, power_rating, balance)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(team.region_id as i64)
        .bind(&team.name)
        .bind(&team.short_name)
        .bind(team.power_rating)
        .bind(team.balance as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取队伍
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        team_id: u64,
    ) -> Result<Team, DatabaseError> {
        let row = sqlx::query("SELECT * FROM teams WHERE id = ?")
            .bind(team_id as i64)
            .fetch_optional(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?
            .ok_or_else(|| DatabaseError::NotFound(format!("Team {} not found", team_id)))?;

        Ok(row_to_team(&row))
    }

    /// 获取赛区所有队伍
    pub async fn get_by_region(
        pool: &Pool<Sqlite>,
        save_id: &str,
        region_id: u64,
    ) -> Result<Vec<Team>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM teams WHERE save_id = ? AND region_id = ? ORDER BY power_rating DESC"
        )
        .bind(save_id)
        .bind(region_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_team).collect())
    }

    /// 更新队伍
    pub async fn update(pool: &Pool<Sqlite>, team: &Team) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE teams SET
                name = ?,
                short_name = ?,
                power_rating = ?,
                total_matches = ?,
                wins = ?,
                win_rate = ?,
                annual_points = ?,
                cross_year_points = ?,
                balance = ?
            WHERE id = ?
            "#,
        )
        .bind(&team.name)
        .bind(&team.short_name)
        .bind(team.power_rating)
        .bind(team.total_matches as i64)
        .bind(team.wins as i64)
        .bind(team.win_rate)
        .bind(team.annual_points as i64)
        .bind(team.cross_year_points as i64)
        .bind(team.balance as i64)
        .bind(team.id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 更新队伍余额
    pub async fn update_balance(
        pool: &Pool<Sqlite>,
        team_id: u64,
        amount: i64,
    ) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
            .bind(amount)
            .bind(team_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 选手仓库
pub struct PlayerRepository;

impl PlayerRepository {
    /// 创建选手
    pub async fn create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player: &Player,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO players (
                save_id, game_id, real_name, nationality, age, ability, potential,
                stability, tag, status, position, team_id, salary, market_value,
                contract_end_season, join_season, is_starter
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(&player.game_id)
        .bind(&player.real_name)
        .bind(&player.nationality)
        .bind(player.age as i64)
        .bind(player.ability as i64)
        .bind(player.potential as i64)
        .bind(player.stability as i64)
        .bind(format!("{:?}", player.tag))
        .bind(format!("{:?}", player.status))
        .bind(format!("{:?}", player.position))
        .bind(player.team_id.map(|id| id as i64))
        .bind(player.salary as i64)
        .bind(player.market_value as i64)
        .bind(player.contract_end_season.map(|s| s as i64))
        .bind(player.join_season as i64)
        .bind(player.is_starter)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取选手
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        player_id: u64,
    ) -> Result<Player, DatabaseError> {
        let row = sqlx::query("SELECT * FROM players WHERE id = ?")
            .bind(player_id as i64)
            .fetch_optional(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?
            .ok_or_else(|| DatabaseError::NotFound(format!("Player {} not found", player_id)))?;

        Ok(row_to_player(&row))
    }

    /// 获取队伍选手
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        team_id: u64,
    ) -> Result<Vec<Player>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM players WHERE team_id = ? AND status = 'Active' ORDER BY position, ability DESC"
        )
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player).collect())
    }

    /// 获取首发阵容
    pub async fn get_starters(
        pool: &Pool<Sqlite>,
        team_id: u64,
    ) -> Result<Vec<Player>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM players WHERE team_id = ? AND is_starter = 1 AND status = 'Active'"
        )
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player).collect())
    }

    /// 更新选手
    pub async fn update(pool: &Pool<Sqlite>, player: &Player) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE players SET
                game_id = ?, real_name = ?, nationality = ?, age = ?,
                ability = ?, potential = ?, stability = ?, tag = ?,
                status = ?, position = ?, team_id = ?, salary = ?,
                market_value = ?, contract_end_season = ?, is_starter = ?
            WHERE id = ?
            "#,
        )
        .bind(&player.game_id)
        .bind(&player.real_name)
        .bind(&player.nationality)
        .bind(player.age as i64)
        .bind(player.ability as i64)
        .bind(player.potential as i64)
        .bind(player.stability as i64)
        .bind(format!("{:?}", player.tag))
        .bind(format!("{:?}", player.status))
        .bind(format!("{:?}", player.position))
        .bind(player.team_id.map(|id| id as i64))
        .bind(player.salary as i64)
        .bind(player.market_value as i64)
        .bind(player.contract_end_season.map(|s| s as i64))
        .bind(player.is_starter)
        .bind(player.id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 转会 - 更新队伍
    pub async fn transfer(
        pool: &Pool<Sqlite>,
        player_id: u64,
        new_team_id: Option<u64>,
        salary: u64,
        contract_end: u32,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE players SET
                team_id = ?,
                salary = ?,
                contract_end_season = ?,
                is_starter = 0
            WHERE id = ?
            "#,
        )
        .bind(new_team_id.map(|id| id as i64))
        .bind(salary as i64)
        .bind(contract_end as i64)
        .bind(player_id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 比赛仓库
pub struct MatchRepository;

impl MatchRepository {
    /// 批量创建比赛
    pub async fn create_batch(
        pool: &Pool<Sqlite>,
        save_id: &str,
        matches: &[Match],
    ) -> Result<(), DatabaseError> {
        for m in matches {
            sqlx::query(
                r#"
                INSERT INTO matches (
                    save_id, tournament_id, stage, round, match_order, format,
                    home_team_id, away_team_id, home_score, away_score, status
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(save_id)
            .bind(m.tournament_id as i64)
            .bind(&m.stage)
            .bind(m.round.map(|r| r as i64))
            .bind(m.match_order.map(|o| o as i64))
            .bind(format!("{:?}", m.format))
            .bind(m.home_team_id as i64)
            .bind(m.away_team_id as i64)
            .bind(m.home_score as i64)
            .bind(m.away_score as i64)
            .bind(format!("{:?}", m.status))
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        Ok(())
    }

    /// 获取赛事的所有比赛
    pub async fn get_by_tournament(
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Vec<Match>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM matches WHERE tournament_id = ? ORDER BY round, match_order"
        )
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_match).collect())
    }

    /// 获取待进行的比赛
    pub async fn get_pending(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Vec<Match>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM matches WHERE save_id = ? AND tournament_id = ? AND status = 'Scheduled' ORDER BY round, match_order"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_match).collect())
    }

    /// 更新比赛结果
    pub async fn update_result(
        pool: &Pool<Sqlite>,
        match_id: u64,
        home_score: u32,
        away_score: u32,
        winner_id: u64,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE matches SET
                home_score = ?,
                away_score = ?,
                winner_id = ?,
                status = 'Completed',
                played_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(home_score as i64)
        .bind(away_score as i64)
        .bind(winner_id as i64)
        .bind(match_id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 赛事仓库
pub struct TournamentRepository;

impl TournamentRepository {
    /// 创建赛事
    pub async fn create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament: &Tournament,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO tournaments (save_id, season_id, region_id, name, tournament_type, status)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(tournament.season_id as i64)
        .bind(tournament.region_id.map(|id| id as i64))
        .bind(&tournament.name)
        .bind(format!("{:?}", tournament.tournament_type))
        .bind(format!("{:?}", tournament.status))
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取赛事
    pub async fn get_by_id(
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Tournament, DatabaseError> {
        let row = sqlx::query("SELECT * FROM tournaments WHERE id = ?")
            .bind(tournament_id as i64)
            .fetch_optional(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?
            .ok_or_else(|| DatabaseError::NotFound(format!("Tournament {} not found", tournament_id)))?;

        Ok(row_to_tournament(&row))
    }

    /// 更新赛事状态
    pub async fn update_status(
        pool: &Pool<Sqlite>,
        tournament_id: u64,
        status: TournamentStatus,
    ) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE tournaments SET status = ? WHERE id = ?")
            .bind(format!("{:?}", status))
            .bind(tournament_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 积分榜仓库
pub struct StandingRepository;

impl StandingRepository {
    /// 批量创建或更新积分榜
    pub async fn upsert_batch(
        pool: &Pool<Sqlite>,
        save_id: &str,
        standings: &[LeagueStanding],
    ) -> Result<(), DatabaseError> {
        for s in standings {
            sqlx::query(
                r#"
                INSERT INTO league_standings (
                    save_id, tournament_id, team_id, rank, matches_played,
                    wins, losses, points, games_won, games_lost, game_diff
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(tournament_id, team_id) DO UPDATE SET
                    rank = excluded.rank,
                    matches_played = excluded.matches_played,
                    wins = excluded.wins,
                    losses = excluded.losses,
                    points = excluded.points,
                    games_won = excluded.games_won,
                    games_lost = excluded.games_lost,
                    game_diff = excluded.game_diff
                "#,
            )
            .bind(save_id)
            .bind(s.tournament_id as i64)
            .bind(s.team_id as i64)
            .bind(s.rank.map(|r| r as i64))
            .bind(s.matches_played as i64)
            .bind(s.wins as i64)
            .bind(s.losses as i64)
            .bind(s.points as i64)
            .bind(s.games_won as i64)
            .bind(s.games_lost as i64)
            .bind(s.game_diff as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        Ok(())
    }

    /// 获取赛事积分榜
    pub async fn get_by_tournament(
        pool: &Pool<Sqlite>,
        tournament_id: u64,
    ) -> Result<Vec<LeagueStanding>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM league_standings WHERE tournament_id = ? ORDER BY rank"
        )
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_standing).collect())
    }
}

// === 辅助函数 ===

fn parse_season_phase(s: &str) -> SeasonPhase {
    match s {
        "SpringRegular" => SeasonPhase::SpringRegular,
        "SpringPlayoffs" => SeasonPhase::SpringPlayoffs,
        "Msi" => SeasonPhase::Msi,
        "MadridMasters" => SeasonPhase::MadridMasters,
        "SummerRegular" => SeasonPhase::SummerRegular,
        "SummerPlayoffs" => SeasonPhase::SummerPlayoffs,
        "ClaudeIntercontinental" => SeasonPhase::ClaudeIntercontinental,
        "WorldChampionship" => SeasonPhase::WorldChampionship,
        "ShanghaiMasters" => SeasonPhase::ShanghaiMasters,
        "IcpIntercontinental" => SeasonPhase::IcpIntercontinental,
        "SuperIntercontinental" => SeasonPhase::SuperIntercontinental,
        "TransferWindow" => SeasonPhase::TransferWindow,
        "Draft" => SeasonPhase::Draft,
        "SeasonEnd" => SeasonPhase::SeasonEnd,
        _ => SeasonPhase::SpringRegular,
    }
}

fn row_to_team(row: &sqlx::sqlite::SqliteRow) -> Team {
    Team {
        id: row.get::<i64, _>("id") as u64,
        region_id: row.get::<i64, _>("region_id") as u64,
        name: row.get("name"),
        short_name: row.get("short_name"),
        power_rating: row.get("power_rating"),
        total_matches: row.get::<i64, _>("total_matches") as u32,
        wins: row.get::<i64, _>("wins") as u32,
        win_rate: row.get("win_rate"),
        annual_points: row.get::<i64, _>("annual_points") as u32,
        cross_year_points: row.get::<i64, _>("cross_year_points") as u32,
        balance: row.get::<i64, _>("balance"),
    }
}

fn row_to_player(row: &sqlx::sqlite::SqliteRow) -> Player {
    let pos_str: Option<String> = row.get("position");
    Player {
        id: row.get::<i64, _>("id") as u64,
        game_id: row.get("game_id"),
        real_name: row.get("real_name"),
        nationality: row.get("nationality"),
        age: row.get::<i64, _>("age") as u8,
        ability: row.get::<i64, _>("ability") as u8,
        potential: row.get::<i64, _>("potential") as u8,
        stability: row.get::<i64, _>("stability") as u8,
        tag: parse_player_tag(row.get("tag")),
        status: parse_player_status(row.get("status")),
        position: pos_str.as_deref().map(parse_position),
        team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
        salary: row.get::<i64, _>("salary") as u64,
        market_value: row.get::<i64, _>("market_value") as u64,
        contract_end_season: row.get::<Option<i64>, _>("contract_end_season").map(|v| v as u32),
        join_season: row.get::<i64, _>("join_season") as u32,
        retire_season: row.get::<Option<i64>, _>("retire_season").map(|v| v as u32),
        is_starter: row.get("is_starter"),
    }
}

fn row_to_match(row: &sqlx::sqlite::SqliteRow) -> Match {
    Match {
        id: row.get::<i64, _>("id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        stage: row.get("stage"),
        round: row.get::<Option<i64>, _>("round").map(|v| v as u32),
        match_order: row.get::<Option<i64>, _>("match_order").map(|v| v as u32),
        format: parse_match_format(row.get("format")),
        home_team_id: row.get::<i64, _>("home_team_id") as u64,
        away_team_id: row.get::<i64, _>("away_team_id") as u64,
        home_score: row.get::<i64, _>("home_score") as u8,
        away_score: row.get::<i64, _>("away_score") as u8,
        winner_id: row.get::<Option<i64>, _>("winner_id").map(|v| v as u64),
        status: parse_match_status(row.get("status")),
    }
}

fn row_to_tournament(row: &sqlx::sqlite::SqliteRow) -> Tournament {
    Tournament {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_type: parse_tournament_type(row.get("tournament_type")),
        name: row.get("name"),
        region_id: row.get::<Option<i64>, _>("region_id").map(|v| v as u64),
        status: parse_tournament_status(row.get("status")),
        current_stage: row.get("current_stage"),
        current_round: row.get::<Option<i64>, _>("current_round").map(|v| v as u32),
    }
}

fn row_to_standing(row: &sqlx::sqlite::SqliteRow) -> LeagueStanding {
    LeagueStanding {
        id: row.get::<i64, _>("id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        team_id: row.get::<i64, _>("team_id") as u64,
        rank: row.get::<Option<i64>, _>("rank").map(|v| v as u32),
        matches_played: row.get::<i64, _>("matches_played") as u32,
        wins: row.get::<i64, _>("wins") as u32,
        losses: row.get::<i64, _>("losses") as u32,
        points: row.get::<i64, _>("points") as u32,
        games_won: row.get::<i64, _>("games_won") as u32,
        games_lost: row.get::<i64, _>("games_lost") as u32,
        game_diff: row.get::<i64, _>("game_diff") as i32,
    }
}

fn parse_player_tag(s: &str) -> PlayerTag {
    match s {
        "Ordinary" => PlayerTag::Ordinary,
        "Normal" => PlayerTag::Normal,
        "Genius" => PlayerTag::Genius,
        _ => PlayerTag::Normal,
    }
}

fn parse_player_status(s: &str) -> PlayerStatus {
    match s {
        "Active" => PlayerStatus::Active,
        "Retired" => PlayerStatus::Retired,
        _ => PlayerStatus::Active,
    }
}

fn parse_position(s: &str) -> Position {
    match s {
        "Top" => Position::Top,
        "Jug" => Position::Jug,
        "Mid" => Position::Mid,
        "Adc" => Position::Adc,
        "Sup" => Position::Sup,
        _ => Position::Mid,
    }
}

fn parse_match_format(s: &str) -> MatchFormat {
    match s {
        "Bo1" => MatchFormat::Bo1,
        "Bo3" => MatchFormat::Bo3,
        "Bo5" => MatchFormat::Bo5,
        _ => MatchFormat::Bo3,
    }
}

fn parse_match_status(s: &str) -> MatchStatus {
    match s {
        "Scheduled" => MatchStatus::Scheduled,
        "InProgress" => MatchStatus::InProgress,
        "Completed" => MatchStatus::Completed,
        _ => MatchStatus::Scheduled,
    }
}

fn parse_tournament_type(s: &str) -> TournamentType {
    match s {
        "SpringRegular" => TournamentType::SpringRegular,
        "SpringPlayoffs" => TournamentType::SpringPlayoffs,
        "SummerRegular" => TournamentType::SummerRegular,
        "SummerPlayoffs" => TournamentType::SummerPlayoffs,
        "Msi" => TournamentType::Msi,
        "MadridMasters" => TournamentType::MadridMasters,
        "ClaudeIntercontinental" => TournamentType::ClaudeIntercontinental,
        "WorldChampionship" => TournamentType::WorldChampionship,
        "ShanghaiMasters" => TournamentType::ShanghaiMasters,
        "IcpIntercontinental" => TournamentType::IcpIntercontinental,
        "SuperIntercontinental" => TournamentType::SuperIntercontinental,
        _ => TournamentType::SpringRegular,
    }
}

fn parse_tournament_status(s: &str) -> TournamentStatus {
    match s {
        "Upcoming" => TournamentStatus::Upcoming,
        "InProgress" => TournamentStatus::InProgress,
        "Completed" => TournamentStatus::Completed,
        _ => TournamentStatus::Upcoming,
    }
}

fn parse_honor_type(s: &str) -> HonorType {
    match s {
        "TEAM_CHAMPION" => HonorType::TeamChampion,
        "TEAM_RUNNER_UP" => HonorType::TeamRunnerUp,
        "TEAM_THIRD" => HonorType::TeamThird,
        "TEAM_FOURTH" => HonorType::TeamFourth,
        "REGULAR_SEASON_FIRST" => HonorType::RegularSeasonFirst,
        "TOURNAMENT_MVP" => HonorType::TournamentMvp,
        "FINALS_MVP" => HonorType::FinalsMvp,
        "REGULAR_SEASON_MVP" => HonorType::RegularSeasonMvp,
        "PLAYOFFS_MVP" => HonorType::PlayoffsMvp,
        "PLAYER_CHAMPION" => HonorType::PlayerChampion,
        _ => HonorType::TeamChampion,
    }
}

fn row_to_honor(row: &sqlx::sqlite::SqliteRow) -> Honor {
    let stats_json: Option<String> = row.get("stats_json");
    let stats = stats_json.and_then(|json| serde_json::from_str(&json).ok());

    Honor {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        honor_type: parse_honor_type(row.get("honor_type")),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        tournament_name: row.get("tournament_name"),
        tournament_type: row.get("tournament_type"),
        team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
        team_name: row.get("team_name"),
        player_id: row.get::<Option<i64>, _>("player_id").map(|v| v as u64),
        player_name: row.get("player_name"),
        position: row.get("position"),
        stats,
        created_at: row.get("created_at"),
    }
}

/// 荣誉仓库
pub struct HonorRepository;

impl HonorRepository {
    /// 创建荣誉记录
    pub async fn create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        honor: &Honor,
    ) -> Result<u64, DatabaseError> {
        let stats_json = honor.stats.as_ref().map(|s| serde_json::to_string(s).unwrap_or_default());

        let result = sqlx::query(
            r#"
            INSERT INTO honors (
                save_id, honor_type, season_id, tournament_id, tournament_name, tournament_type,
                team_id, team_name, player_id, player_name, position, stats_json, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(format!("{:?}", honor.honor_type).to_uppercase().replace("HONOR", ""))
        .bind(honor.season_id as i64)
        .bind(honor.tournament_id as i64)
        .bind(&honor.tournament_name)
        .bind(&honor.tournament_type)
        .bind(honor.team_id.map(|v| v as i64))
        .bind(&honor.team_name)
        .bind(honor.player_id.map(|v| v as i64))
        .bind(&honor.player_name)
        .bind(&honor.position)
        .bind(&stats_json)
        .bind(&honor.created_at)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 批量创建荣誉记录
    pub async fn create_batch(
        pool: &Pool<Sqlite>,
        save_id: &str,
        honors: &[Honor],
    ) -> Result<Vec<u64>, DatabaseError> {
        let mut ids = Vec::new();
        for honor in honors {
            let id = Self::create(pool, save_id, honor).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// 获取战队所有荣誉
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND team_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取选手所有荣誉
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND player_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取赛季所有荣誉
    pub async fn get_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND season_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取赛事所有荣誉
    pub async fn get_by_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND tournament_id = ? ORDER BY honor_type"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取所有荣誉（荣誉殿堂）
    pub async fn get_all(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取所有冠军记录
    pub async fn get_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND honor_type = 'TEAM_CHAMPION' ORDER BY created_at DESC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取所有MVP记录
    pub async fn get_mvps(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM honors
            WHERE save_id = ? AND honor_type IN ('TOURNAMENT_MVP', 'FINALS_MVP', 'REGULAR_SEASON_MVP', 'PLAYOFFS_MVP')
            ORDER BY created_at DESC
            "#
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 统计战队冠军数量
    pub async fn count_team_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM honors WHERE save_id = ? AND team_id = ? AND honor_type = 'TEAM_CHAMPION'"
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
    }

    /// 统计选手冠军数量
    pub async fn count_player_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM honors WHERE save_id = ? AND player_id = ? AND honor_type = 'PLAYER_CHAMPION'"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
    }

    /// 统计选手MVP数量
    pub async fn count_player_mvps(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM honors
            WHERE save_id = ? AND player_id = ? AND honor_type IN ('TOURNAMENT_MVP', 'FINALS_MVP', 'REGULAR_SEASON_MVP', 'PLAYOFFS_MVP')
            "#
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
    }

    /// 按赛事类型获取冠军
    pub async fn get_champions_by_tournament_type(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_type: &str,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND honor_type = 'TEAM_CHAMPION' AND tournament_type = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(tournament_type)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }
}

/// 事件仓库
pub struct EventRepository;

impl EventRepository {
    /// 创建事件记录
    pub async fn create(
        pool: &Pool<Sqlite>,
        event: &GameEvent,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO game_events (save_id, season_id, event_type, player_id, team_id, description, details, phase)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.save_id)
        .bind(event.season_id as i64)
        .bind(format!("{:?}", event.event_type))
        .bind(event.player_id.map(|id| id as i64))
        .bind(event.team_id.map(|id| id as i64))
        .bind(&event.description)
        .bind(&event.details)
        .bind(&event.phase)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 批量创建事件记录
    pub async fn create_batch(
        pool: &Pool<Sqlite>,
        events: &[GameEvent],
    ) -> Result<Vec<u64>, DatabaseError> {
        let mut ids = Vec::new();
        for event in events {
            let id = Self::create(pool, event).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// 获取赛季所有事件
    pub async fn get_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<GameEvent>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_events WHERE save_id = ? AND season_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_event).collect())
    }

    /// 获取选手相关事件
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<GameEvent>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_events WHERE save_id = ? AND player_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_event).collect())
    }

    /// 获取特定类型的事件
    pub async fn get_by_type(
        pool: &Pool<Sqlite>,
        save_id: &str,
        event_type: &str,
    ) -> Result<Vec<GameEvent>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_events WHERE save_id = ? AND event_type = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(event_type)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_event).collect())
    }
}

fn parse_event_type(s: &str) -> EventType {
    match s {
        "PlayerGrowth" => EventType::PlayerGrowth,
        "PlayerDecline" => EventType::PlayerDecline,
        "PlayerRetirement" => EventType::PlayerRetirement,
        "RookieGeneration" => EventType::RookieGeneration,
        "ContractExpire" => EventType::ContractExpire,
        "PlayerAging" => EventType::PlayerAging,
        "SeasonSettlement" => EventType::SeasonSettlement,
        _ => EventType::SeasonSettlement,
    }
}

fn row_to_game_event(row: &sqlx::sqlite::SqliteRow) -> GameEvent {
    GameEvent {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        event_type: parse_event_type(row.get("event_type")),
        player_id: row.get::<Option<i64>, _>("player_id").map(|v| v as u64),
        team_id: row.get::<Option<i64>, _>("team_id").map(|v| v as u64),
        description: row.get("description"),
        details: row.get("details"),
        phase: row.get("phase"),
    }
}

impl PlayerRepository {
    /// 获取存档所有活跃选手
    pub async fn get_all_active(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Player>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM players WHERE save_id = ? AND status = 'Active' ORDER BY ability DESC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player).collect())
    }

    /// 批量更新选手能力值
    pub async fn batch_update_ability(
        pool: &Pool<Sqlite>,
        updates: &[(u64, u8)],
    ) -> Result<(), DatabaseError> {
        for (player_id, new_ability) in updates {
            sqlx::query("UPDATE players SET ability = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(*new_ability as i64)
                .bind(*player_id as i64)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }
        Ok(())
    }

    /// 批量更新选手年龄和稳定性
    pub async fn batch_update_age(
        pool: &Pool<Sqlite>,
        updates: &[(u64, u8, u8)],
    ) -> Result<(), DatabaseError> {
        for (player_id, new_age, new_stability) in updates {
            sqlx::query(
                "UPDATE players SET age = ?, stability = ?, updated_at = datetime('now') WHERE id = ?"
            )
            .bind(*new_age as i64)
            .bind(*new_stability as i64)
            .bind(*player_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }
        Ok(())
    }

    /// 批量设置选手退役
    pub async fn batch_retire(
        pool: &Pool<Sqlite>,
        player_ids: &[u64],
        retire_season: u32,
    ) -> Result<(), DatabaseError> {
        for player_id in player_ids {
            sqlx::query(
                r#"
                UPDATE players SET
                    status = 'Retired',
                    retire_season = ?,
                    team_id = NULL,
                    is_starter = 0,
                    updated_at = datetime('now')
                WHERE id = ?
                "#
            )
            .bind(retire_season as i64)
            .bind(*player_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }
        Ok(())
    }

    /// 批量更新合同
    pub async fn batch_update_contracts(
        pool: &Pool<Sqlite>,
        updates: &[(u64, bool, Option<u32>, Option<u64>)],
        current_season: u32,
    ) -> Result<(), DatabaseError> {
        for (player_id, renewed, contract_years, salary) in updates {
            if *renewed {
                if let (Some(years), Some(sal)) = (contract_years, salary) {
                    sqlx::query(
                        r#"
                        UPDATE players SET
                            contract_end_season = ?,
                            salary = ?,
                            updated_at = datetime('now')
                        WHERE id = ?
                        "#
                    )
                    .bind((current_season + years) as i64)
                    .bind(*sal as i64)
                    .bind(*player_id as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseError::Query(e.to_string()))?;
                }
            } else {
                // 未续约，成为自由球员
                sqlx::query(
                    r#"
                    UPDATE players SET
                        team_id = NULL,
                        is_starter = 0,
                        contract_end_season = NULL,
                        updated_at = datetime('now')
                    WHERE id = ?
                    "#
                )
                .bind(*player_id as i64)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// 更新选手身价
    pub async fn update_market_value(
        pool: &Pool<Sqlite>,
        player_id: u64,
        market_value: u64,
    ) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE players SET market_value = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(market_value as i64)
            .bind(player_id as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

impl TeamRepository {
    /// 获取存档所有队伍
    pub async fn get_all(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Team>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM teams WHERE save_id = ? ORDER BY power_rating DESC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_team).collect())
    }
}
