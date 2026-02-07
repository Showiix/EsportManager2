use crate::db::DatabaseError;
use crate::models::*;
// 显式导入以解决歧义
use crate::models::tournament_result::PlayerTournamentStats;
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

    /// 删除存档（级联删除所有关联数据）
    pub async fn delete(pool: &Pool<Sqlite>, save_id: &str) -> Result<(), DatabaseError> {
        // 获取一个连接并在其上执行所有操作
        let mut conn = pool.acquire().await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 禁用外键约束检查（在同一个连接上）
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *conn)
            .await
            .ok();

        // 删除所有关联数据（按依赖顺序）
        let tables = [
            // 最底层的表（没有被其他表引用的）
            "game_player_performances",
            "match_games",
            "player_traits",
            "player_form_factors",
            "player_season_stats",
            "player_tournament_stats",
            "tournament_results",
            "honors",
            "annual_points_detail",
            "global_rankings",
            "league_standings",
            "team_season_finances",
            "financial_transactions",
            "draft_results",
            "draft_orders",
            "draft_players",
            "transfer_events",
            "transfer_windows",
            "transfer_records",
            "transfer_listings",
            "free_agents",
            // 中间层的表
            "matches",
            "players",
            "tournaments",
            "teams",
            "regions",
        ];

        for table in tables {
            sqlx::query(&format!("DELETE FROM {} WHERE save_id = ?", table))
                .bind(save_id)
                .execute(&mut *conn)
                .await
                .ok(); // 忽略错误（表可能不存在或已经为空）
        }

        // 删除存档本身
        sqlx::query("DELETE FROM saves WHERE id = ?")
            .bind(save_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 重新启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *conn)
            .await
            .ok();

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
        .bind(player.position.map(|p| format!("{:?}", p)))
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
        .bind(player.position.map(|p| format!("{:?}", p)))
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
            // 当 team_id=0 时表示队伍待定，绑定 NULL
            let home_team = if m.home_team_id > 0 { Some(m.home_team_id as i64) } else { None };
            let away_team = if m.away_team_id > 0 { Some(m.away_team_id as i64) } else { None };

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
            .bind(home_team)
            .bind(away_team)
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
                status = 'COMPLETED',
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

    /// 按赛季和赛事类型获取赛事
    pub async fn get_by_season_and_type(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        tournament_type: &str,
    ) -> Result<Vec<Tournament>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournaments WHERE save_id = ? AND season_id = ? AND tournament_type = ?"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(tournament_type)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament).collect())
    }

    /// 获取所有已完成的赛事
    pub async fn get_completed(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Tournament>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournaments WHERE save_id = ? AND status = 'Completed' ORDER BY season_id ASC, id ASC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament).collect())
    }

    /// 获取赛季所有赛事
    pub async fn get_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<Tournament>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournaments WHERE save_id = ? AND season_id = ? ORDER BY id ASC"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament).collect())
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
        "AnnualAwards" => SeasonPhase::AnnualAwards,
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
        calculated_market_value: row.try_get::<i64, _>("calculated_market_value").ok().map(|v| v as u64).unwrap_or(0),
        contract_end_season: row.get::<Option<i64>, _>("contract_end_season").map(|v| v as u32),
        join_season: row.get::<i64, _>("join_season") as u32,
        retire_season: row.get::<Option<i64>, _>("retire_season").map(|v| v as u32),
        is_starter: row.get("is_starter"),
        loyalty: row.try_get::<i64, _>("loyalty").ok().map(|v| v as u8).unwrap_or(50),
        satisfaction: row.try_get::<i64, _>("satisfaction").ok().map(|v| v as u8).unwrap_or(50),
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
        // NULL 表示队伍待定，转换为 0
        home_team_id: row.get::<Option<i64>, _>("home_team_id").unwrap_or(0) as u64,
        away_team_id: row.get::<Option<i64>, _>("away_team_id").unwrap_or(0) as u64,
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
    // 处理 Some(Position) 格式和纯 Position 格式
    let s = s.trim_start_matches("Some(").trim_end_matches(")");
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
        "Scheduled" | "SCHEDULED" => MatchStatus::Scheduled,
        "InProgress" | "IN_PROGRESS" => MatchStatus::InProgress,
        "Completed" | "COMPLETED" => MatchStatus::Completed,
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
        "PLAYOFFS_FMVP" => HonorType::PlayoffsFmvp,
        "PLAYER_CHAMPION" => HonorType::PlayerChampion,
        "PLAYER_RUNNER_UP" => HonorType::PlayerRunnerUp,
        "PLAYER_THIRD" => HonorType::PlayerThird,
        "PLAYER_FOURTH" => HonorType::PlayerFourth,
        "ANNUAL_MVP" => HonorType::AnnualMvp,
        "ANNUAL_TOP20" => HonorType::AnnualTop20,
        "ANNUAL_BEST_TOP" => HonorType::AnnualBestTop,
        "ANNUAL_BEST_JUNGLE" => HonorType::AnnualBestJungle,
        "ANNUAL_BEST_MID" => HonorType::AnnualBestMid,
        "ANNUAL_BEST_ADC" => HonorType::AnnualBestAdc,
        "ANNUAL_BEST_SUPPORT" => HonorType::AnnualBestSupport,
        "ANNUAL_ROOKIE" => HonorType::AnnualRookie,
        _ => HonorType::TeamChampion,
    }
}

fn honor_type_to_db_string(honor_type: &HonorType) -> &'static str {
    match honor_type {
        HonorType::TeamChampion => "TEAM_CHAMPION",
        HonorType::TeamRunnerUp => "TEAM_RUNNER_UP",
        HonorType::TeamThird => "TEAM_THIRD",
        HonorType::TeamFourth => "TEAM_FOURTH",
        HonorType::RegularSeasonFirst => "REGULAR_SEASON_FIRST",
        HonorType::TournamentMvp => "TOURNAMENT_MVP",
        HonorType::FinalsMvp => "FINALS_MVP",
        HonorType::RegularSeasonMvp => "REGULAR_SEASON_MVP",
        HonorType::PlayoffsFmvp => "PLAYOFFS_FMVP",
        HonorType::PlayerChampion => "PLAYER_CHAMPION",
        HonorType::PlayerRunnerUp => "PLAYER_RUNNER_UP",
        HonorType::PlayerThird => "PLAYER_THIRD",
        HonorType::PlayerFourth => "PLAYER_FOURTH",
        HonorType::AnnualMvp => "ANNUAL_MVP",
        HonorType::AnnualTop20 => "ANNUAL_TOP20",
        HonorType::AnnualBestTop => "ANNUAL_BEST_TOP",
        HonorType::AnnualBestJungle => "ANNUAL_BEST_JUNGLE",
        HonorType::AnnualBestMid => "ANNUAL_BEST_MID",
        HonorType::AnnualBestAdc => "ANNUAL_BEST_ADC",
        HonorType::AnnualBestSupport => "ANNUAL_BEST_SUPPORT",
        HonorType::AnnualRookie => "ANNUAL_ROOKIE",
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
        tournament_id: row.get::<Option<i64>, _>("tournament_id").map(|v| v as u64),
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
        .bind(honor_type_to_db_string(&honor.honor_type))
        .bind(honor.season_id as i64)
        .bind(honor.tournament_id.map(|v| v as i64))
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

    /// 获取战队所有荣誉（仅战队荣誉，不含选手荣誉）
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<Vec<Honor>, DatabaseError> {
        // 只返回战队专属荣誉（player_id IS NULL），不包含选手的冠军成员荣誉
        let rows = sqlx::query(
            "SELECT * FROM honors WHERE save_id = ? AND team_id = ? AND player_id IS NULL ORDER BY created_at DESC"
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

    /// 获取所有战队名次记录（冠军、亚军、季军、殿军）
    pub async fn get_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM honors
            WHERE save_id = ? AND honor_type IN ('TEAM_CHAMPION', 'TEAM_RUNNER_UP', 'TEAM_THIRD', 'TEAM_FOURTH')
            ORDER BY season_id DESC, tournament_id, honor_type
            "#
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

    /// 获取国际赛事冠军列表
    /// 国际赛事类型: Msi, WorldChampionship, MadridMasters, ShanghaiMasters 等
    pub async fn get_international_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<Honor>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM honors
            WHERE save_id = ? AND honor_type = 'TEAM_CHAMPION'
            AND tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters', 'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental')
            ORDER BY season_id ASC, tournament_type
            "#
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_honor).collect())
    }

    /// 获取选手荣誉排行榜（按冠军数+MVP数排序）
    pub async fn get_player_honor_rankings(
        pool: &Pool<Sqlite>,
        save_id: &str,
        limit: i32,
    ) -> Result<Vec<(u64, String, u32, u32, u32)>, DatabaseError> {
        // 返回: (player_id, player_name, champion_count, mvp_count, international_champion_count)
        let rows = sqlx::query(
            r#"
            SELECT
                player_id,
                player_name,
                SUM(CASE WHEN honor_type = 'PLAYER_CHAMPION' THEN 1 ELSE 0 END) as champion_count,
                SUM(CASE WHEN honor_type IN ('TOURNAMENT_MVP', 'FINALS_MVP', 'REGULAR_SEASON_MVP', 'PLAYOFFS_MVP') THEN 1 ELSE 0 END) as mvp_count,
                SUM(CASE WHEN honor_type = 'PLAYER_CHAMPION' AND tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters', 'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental') THEN 1 ELSE 0 END) as intl_count
            FROM honors
            WHERE save_id = ? AND player_id IS NOT NULL
            GROUP BY player_id, player_name
            ORDER BY champion_count DESC, mvp_count DESC, intl_count DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| {
            (
                row.get::<i64, _>("player_id") as u64,
                row.get::<String, _>("player_name"),
                row.get::<i64, _>("champion_count") as u32,
                row.get::<i64, _>("mvp_count") as u32,
                row.get::<i64, _>("intl_count") as u32,
            )
        }).collect())
    }

    /// 获取战队荣誉排行榜（按冠军数排序）
    pub async fn get_team_honor_rankings(
        pool: &Pool<Sqlite>,
        save_id: &str,
        limit: i32,
    ) -> Result<Vec<(u64, String, u32, u32, u32)>, DatabaseError> {
        // 返回: (team_id, team_name, champion_count, international_champion_count, runner_up_count)
        let rows = sqlx::query(
            r#"
            SELECT
                team_id,
                team_name,
                SUM(CASE WHEN honor_type = 'TEAM_CHAMPION' THEN 1 ELSE 0 END) as champion_count,
                SUM(CASE WHEN honor_type = 'TEAM_CHAMPION' AND tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters', 'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental') THEN 1 ELSE 0 END) as intl_count,
                SUM(CASE WHEN honor_type = 'TEAM_RUNNER_UP' THEN 1 ELSE 0 END) as runner_up_count
            FROM honors
            WHERE save_id = ? AND team_id IS NOT NULL AND honor_type IN ('TEAM_CHAMPION', 'TEAM_RUNNER_UP')
            GROUP BY team_id, team_name
            ORDER BY champion_count DESC, intl_count DESC, runner_up_count DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| {
            (
                row.get::<i64, _>("team_id") as u64,
                row.get::<String, _>("team_name"),
                row.get::<i64, _>("champion_count") as u32,
                row.get::<i64, _>("intl_count") as u32,
                row.get::<i64, _>("runner_up_count") as u32,
            )
        }).collect())
    }

    /// 统计战队亚军数量
    pub async fn count_team_runner_ups(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM honors WHERE save_id = ? AND team_id = ? AND honor_type = 'TEAM_RUNNER_UP'"
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
    }

    /// 统计战队季军数量
    pub async fn count_team_thirds(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM honors WHERE save_id = ? AND team_id = ? AND honor_type = 'TEAM_THIRD'"
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
    }

    /// 统计选手国际赛事冠军数量
    pub async fn count_player_international_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM honors
            WHERE save_id = ? AND player_id = ? AND honor_type = 'PLAYER_CHAMPION'
            AND tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters', 'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental')
            "#
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
    }

    /// 统计战队国际赛事冠军数量
    pub async fn count_team_international_champions(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM honors
            WHERE save_id = ? AND team_id = ? AND honor_type = 'TEAM_CHAMPION'
            AND tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters', 'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental')
            "#
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.get::<i64, _>("count") as u32)
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

/// 选手赛季统计仓库
pub struct PlayerStatsRepository;

impl PlayerStatsRepository {
    /// 获取或创建选手赛季统计
    pub async fn get_or_create(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        player_name: &str,
        season_id: i64,
        team_id: Option<i64>,
        region_id: Option<&str>,
        position: &str,
    ) -> Result<PlayerSeasonStatistics, DatabaseError> {
        // 尝试获取现有记录
        let existing = sqlx::query(
            r#"
            SELECT * FROM player_season_stats
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(save_id)
        .bind(player_id)
        .bind(season_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        if let Some(row) = existing {
            return Ok(row_to_player_stats(&row));
        }

        // 创建新记录
        let result = sqlx::query(
            r#"
            INSERT INTO player_season_stats
            (save_id, player_id, player_name, season_id, team_id, region_id, position,
             matches_played, games_played, total_impact, avg_impact, avg_performance,
             best_performance, worst_performance, consistency_score,
             international_titles, regional_titles, champion_bonus, yearly_top_score)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, 0.0, 0.0, 0.0, 0.0, 100.0, 100.0, 0, 0, 0.0, 0.0)
            "#
        )
        .bind(save_id)
        .bind(player_id)
        .bind(player_name)
        .bind(season_id)
        .bind(team_id)
        .bind(region_id)
        .bind(position)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(PlayerSeasonStatistics {
            id: Some(result.last_insert_rowid()),
            save_id: save_id.to_string(),
            player_id,
            player_name: player_name.to_string(),
            season_id,
            team_id,
            region_id: region_id.map(|s| s.to_string()),
            position: position.to_string(),
            matches_played: 0,
            games_played: 0,
            total_impact: 0.0,
            avg_impact: 0.0,
            avg_performance: 0.0,
            best_performance: 0.0,
            worst_performance: 100.0,
            consistency_score: 100.0,
            international_titles: 0,
            regional_titles: 0,
            champion_bonus: 0.0,
            yearly_top_score: 0.0,
        })
    }

    /// 更新选手赛季统计
    pub async fn update(
        pool: &Pool<Sqlite>,
        stats: &PlayerSeasonStatistics,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            UPDATE player_season_stats SET
                team_id = ?,
                region_id = ?,
                matches_played = ?,
                games_played = ?,
                total_impact = ?,
                avg_impact = ?,
                avg_performance = ?,
                best_performance = ?,
                worst_performance = ?,
                consistency_score = ?,
                international_titles = ?,
                regional_titles = ?,
                champion_bonus = ?,
                yearly_top_score = ?,
                updated_at = datetime('now')
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(stats.team_id)
        .bind(&stats.region_id)
        .bind(stats.matches_played)
        .bind(stats.games_played)
        .bind(stats.total_impact)
        .bind(stats.avg_impact)
        .bind(stats.avg_performance)
        .bind(stats.best_performance)
        .bind(stats.worst_performance)
        .bind(stats.consistency_score)
        .bind(stats.international_titles)
        .bind(stats.regional_titles)
        .bind(stats.champion_bonus)
        .bind(stats.yearly_top_score)
        .bind(&stats.save_id)
        .bind(stats.player_id)
        .bind(stats.season_id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 获取赛季排行榜（按年度Top得分排序）
    /// 注意：games_played 从 game_player_performances 表实时计算，确保数据准确
    pub async fn get_season_ranking(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        limit: i32,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        // 先获取基本排行榜数据
        let rows = sqlx::query(
            r#"
            SELECT pss.*,
                   COALESCE(gpp_count.real_games_played, 0) as real_games_played
            FROM player_season_stats pss
            LEFT JOIN (
                SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments t ON m.tournament_id = t.id
                WHERE gpp.save_id = ? AND t.season_id = ?
                GROUP BY gpp.save_id, gpp.player_id
            ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
            WHERE pss.save_id = ? AND pss.season_id = ?
              AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
            ORDER BY pss.yearly_top_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(save_id)
        .bind(season_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 转换并使用真实的 games_played 值
        Ok(rows.iter().map(|row| {
            let mut stats = row_to_player_stats(row);
            // 使用从 game_player_performances 计算的真实场次数
            let real_games: i64 = row.try_get("real_games_played").unwrap_or(0);
            if real_games > 0 {
                stats.games_played = real_games as i32;
            }
            stats
        }).collect())
    }

    /// 获取分位置排行榜
    /// 注意：games_played 从 game_player_performances 表实时计算，确保数据准确
    pub async fn get_position_ranking(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        position: &str,
        limit: i32,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT pss.*,
                   COALESCE(gpp_count.real_games_played, 0) as real_games_played
            FROM player_season_stats pss
            LEFT JOIN (
                SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments t ON m.tournament_id = t.id
                WHERE gpp.save_id = ? AND t.season_id = ?
                GROUP BY gpp.save_id, gpp.player_id
            ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
            WHERE pss.save_id = ? AND pss.season_id = ? AND pss.position = ?
              AND (pss.games_played > 0 OR COALESCE(gpp_count.real_games_played, 0) > 0)
            ORDER BY pss.yearly_top_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(save_id)
        .bind(season_id)
        .bind(position)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 转换并使用真实的 games_played 值
        Ok(rows.iter().map(|row| {
            let mut stats = row_to_player_stats(row);
            let real_games: i64 = row.try_get("real_games_played").unwrap_or(0);
            if real_games > 0 {
                stats.games_played = real_games as i32;
            }
            stats
        }).collect())
    }

    /// 获取队伍所有选手统计
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        team_id: i64,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM player_season_stats
            WHERE save_id = ? AND season_id = ? AND team_id = ?
            ORDER BY yearly_top_score DESC
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(team_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_stats).collect())
    }

    /// 获取选手的赛季统计
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        season_id: Option<i64>,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        // 使用与 get_season_ranking 相同的逻辑，从 game_player_performances 计算真实场次
        let rows = if let Some(sid) = season_id {
            sqlx::query(
                r#"
                SELECT pss.*,
                       COALESCE(gpp_count.real_games_played, 0) as real_games_played
                FROM player_season_stats pss
                LEFT JOIN (
                    SELECT gpp.save_id, gpp.player_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                    FROM game_player_performances gpp
                    JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                    JOIN matches m ON mg.match_id = m.id
                    JOIN tournaments t ON m.tournament_id = t.id
                    WHERE gpp.save_id = ? AND gpp.player_id = ? AND t.season_id = ?
                    GROUP BY gpp.save_id, gpp.player_id
                ) gpp_count ON pss.save_id = gpp_count.save_id AND pss.player_id = gpp_count.player_id
                WHERE pss.save_id = ? AND pss.player_id = ? AND pss.season_id = ?
                "#
            )
            .bind(save_id)
            .bind(player_id)
            .bind(sid)
            .bind(save_id)
            .bind(player_id)
            .bind(sid)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT pss.*,
                       COALESCE(gpp_count.real_games_played, 0) as real_games_played
                FROM player_season_stats pss
                LEFT JOIN (
                    SELECT gpp.save_id, gpp.player_id, t.season_id, COUNT(DISTINCT gpp.game_id) as real_games_played
                    FROM game_player_performances gpp
                    JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                    JOIN matches m ON mg.match_id = m.id
                    JOIN tournaments t ON m.tournament_id = t.id
                    WHERE gpp.save_id = ? AND gpp.player_id = ?
                    GROUP BY gpp.save_id, gpp.player_id, t.season_id
                ) gpp_count ON pss.save_id = gpp_count.save_id
                           AND pss.player_id = gpp_count.player_id
                           AND pss.season_id = gpp_count.season_id
                WHERE pss.save_id = ? AND pss.player_id = ?
                ORDER BY pss.season_id DESC
                "#
            )
            .bind(save_id)
            .bind(player_id)
            .bind(save_id)
            .bind(player_id)
            .fetch_all(pool)
            .await
        }
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 使用真实场次覆盖
        Ok(rows.iter().map(|row| {
            let mut stats = row_to_player_stats(row);
            let real_games: i64 = row.try_get("real_games_played").unwrap_or(0);
            if real_games > 0 {
                stats.games_played = real_games as i32;
            }
            stats
        }).collect())
    }

    /// 获取赛季所有选手统计（用于批量重新计算）
    pub async fn get_all_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<Vec<PlayerSeasonStatistics>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM player_season_stats
            WHERE save_id = ? AND season_id = ?
            "#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_stats).collect())
    }

    /// 清除赛季统计数据
    pub async fn clear_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "DELETE FROM player_season_stats WHERE save_id = ? AND season_id = ?"
        )
        .bind(save_id)
        .bind(season_id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 将数据库行转换为 PlayerSeasonStatistics
fn row_to_player_stats(row: &sqlx::sqlite::SqliteRow) -> PlayerSeasonStatistics {
    PlayerSeasonStatistics {
        id: Some(row.get::<i64, _>("id")),
        save_id: row.get("save_id"),
        player_id: row.get("player_id"),
        player_name: row.get("player_name"),
        season_id: row.get("season_id"),
        team_id: row.get("team_id"),
        region_id: row.get("region_id"),
        position: row.get("position"),
        matches_played: row.get("matches_played"),
        games_played: row.get("games_played"),
        total_impact: row.get("total_impact"),
        avg_impact: row.get("avg_impact"),
        avg_performance: row.get("avg_performance"),
        best_performance: row.get("best_performance"),
        worst_performance: row.get("worst_performance"),
        consistency_score: row.get("consistency_score"),
        international_titles: row.get("international_titles"),
        regional_titles: row.get("regional_titles"),
        champion_bonus: row.get("champion_bonus"),
        yearly_top_score: row.get("yearly_top_score"),
    }
}

// ==================== 赛事结果仓库 ====================

/// 赛事结果仓库
pub struct TournamentResultRepository;

impl TournamentResultRepository {
    /// 创建赛事结果
    pub async fn create(
        pool: &Pool<Sqlite>,
        result: &TournamentResult,
    ) -> Result<u64, DatabaseError> {
        let row = sqlx::query(
            r#"
            INSERT INTO tournament_results (
                save_id, season_id, tournament_id, tournament_type, tournament_name,
                champion_team_id, champion_team_name, runner_up_team_id, runner_up_team_name,
                third_team_id, third_team_name, fourth_team_id, fourth_team_name,
                final_match_id, final_score, total_matches, total_games
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&result.save_id)
        .bind(result.season_id as i64)
        .bind(result.tournament_id as i64)
        .bind(&result.tournament_type)
        .bind(&result.tournament_name)
        .bind(result.champion_team_id as i64)
        .bind(&result.champion_team_name)
        .bind(result.runner_up_team_id as i64)
        .bind(&result.runner_up_team_name)
        .bind(result.third_team_id.map(|v| v as i64))
        .bind(&result.third_team_name)
        .bind(result.fourth_team_id.map(|v| v as i64))
        .bind(&result.fourth_team_name)
        .bind(result.final_match_id.map(|v| v as i64))
        .bind(&result.final_score)
        .bind(result.total_matches.map(|v| v as i64))
        .bind(result.total_games.map(|v| v as i64))
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.last_insert_rowid() as u64)
    }

    /// 获取赛事结果
    pub async fn get_by_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Option<TournamentResult>, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM tournament_results WHERE save_id = ? AND tournament_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| row_to_tournament_result(&r)))
    }

    /// 获取赛季所有赛事结果
    pub async fn get_by_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<TournamentResult>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournament_results WHERE save_id = ? AND season_id = ? ORDER BY created_at"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament_result).collect())
    }

    /// 获取队伍的冠军记录
    pub async fn get_team_championships(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<Vec<TournamentResult>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournament_results WHERE save_id = ? AND champion_team_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament_result).collect())
    }

    /// 按赛事类型获取结果
    pub async fn get_by_tournament_type(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_type: &str,
    ) -> Result<Vec<TournamentResult>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournament_results WHERE save_id = ? AND tournament_type = ? ORDER BY season_id DESC"
        )
        .bind(save_id)
        .bind(tournament_type)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament_result).collect())
    }

    /// 获取所有赛事结果
    pub async fn get_all(
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<Vec<TournamentResult>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM tournament_results WHERE save_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_tournament_result).collect())
    }
}

/// 将数据库行转换为 TournamentResult
fn row_to_tournament_result(row: &sqlx::sqlite::SqliteRow) -> TournamentResult {
    TournamentResult {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        tournament_type: row.get("tournament_type"),
        tournament_name: row.get("tournament_name"),
        champion_team_id: row.get::<i64, _>("champion_team_id") as u64,
        champion_team_name: row.get("champion_team_name"),
        runner_up_team_id: row.get::<i64, _>("runner_up_team_id") as u64,
        runner_up_team_name: row.get("runner_up_team_name"),
        third_team_id: row.get::<Option<i64>, _>("third_team_id").map(|v| v as u64),
        third_team_name: row.get("third_team_name"),
        fourth_team_id: row.get::<Option<i64>, _>("fourth_team_id").map(|v| v as u64),
        fourth_team_name: row.get("fourth_team_name"),
        final_match_id: row.get::<Option<i64>, _>("final_match_id").map(|v| v as u64),
        final_score: row.get("final_score"),
        total_matches: row.get::<Option<i64>, _>("total_matches").map(|v| v as u32),
        total_games: row.get::<Option<i64>, _>("total_games").map(|v| v as u32),
        created_at: row.get("created_at"),
    }
}

// ==================== 选手赛事统计仓库 ====================

/// 选手赛事统计仓库
pub struct PlayerTournamentStatsRepository;

impl PlayerTournamentStatsRepository {
    /// 创建或更新选手赛事统计
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        stats: &PlayerTournamentStats,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO player_tournament_stats (
                save_id, season_id, tournament_id, tournament_type, player_id, player_name,
                team_id, team_name, position, games_played, games_won, total_impact,
                avg_impact, max_impact, avg_performance, best_performance, game_mvp_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(save_id, tournament_id, player_id) DO UPDATE SET
                games_played = excluded.games_played,
                games_won = excluded.games_won,
                total_impact = excluded.total_impact,
                avg_impact = excluded.avg_impact,
                max_impact = excluded.max_impact,
                avg_performance = excluded.avg_performance,
                best_performance = excluded.best_performance,
                game_mvp_count = excluded.game_mvp_count,
                updated_at = datetime('now')
            "#,
        )
        .bind(&stats.save_id)
        .bind(stats.season_id as i64)
        .bind(stats.tournament_id as i64)
        .bind(&stats.tournament_type)
        .bind(stats.player_id as i64)
        .bind(&stats.player_name)
        .bind(stats.team_id as i64)
        .bind(&stats.team_name)
        .bind(&stats.position)
        .bind(stats.games_played as i64)
        .bind(stats.games_won as i64)
        .bind(stats.total_impact)
        .bind(stats.avg_impact)
        .bind(stats.max_impact)
        .bind(stats.avg_performance)
        .bind(stats.best_performance)
        .bind(stats.game_mvp_count as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 批量创建/更新
    pub async fn upsert_batch(
        pool: &Pool<Sqlite>,
        stats_list: &[PlayerTournamentStats],
    ) -> Result<(), DatabaseError> {
        for stats in stats_list {
            Self::upsert(pool, stats).await?;
        }
        Ok(())
    }

    /// 获取选手在赛事中的统计
    pub async fn get_by_player_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        player_id: u64,
    ) -> Result<Option<PlayerTournamentStats>, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ? AND player_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(player_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| row_to_player_tournament_stats(&r)))
    }

    /// 获取赛事的所有选手统计（用于MVP计算）
    pub async fn get_by_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ? ORDER BY avg_impact DESC"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取赛事MVP候选（按MVP得分排序）
    pub async fn get_mvp_candidates(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        limit: i32,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        // MVP 计算: MVP次数 * 10 + 平均影响力 (MVP次数优先，影响力作为次要排序依据)
        // 降低最低比赛场数要求到1场，以支持季后赛等比赛场数较少的赛事
        let rows = sqlx::query(
            r#"
            SELECT *,
                   (game_mvp_count * 10.0 + avg_impact) as mvp_score
            FROM player_tournament_stats
            WHERE save_id = ? AND tournament_id = ? AND games_played >= 1
            ORDER BY mvp_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取选手在所有赛事中的统计
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND player_id = ? ORDER BY season_id DESC, tournament_id DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取队伍在赛事中的所有选手统计
    pub async fn get_by_team_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        team_id: u64,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ? AND team_id = ? ORDER BY avg_impact DESC"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 获取指定队伍的赛事MVP候选（按MVP得分排序）
    /// 用于国际赛事（MSI、马德里大师赛等）从冠军队伍中选择MVP
    pub async fn get_mvp_candidates_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
        team_id: u64,
        limit: i32,
    ) -> Result<Vec<PlayerTournamentStats>, DatabaseError> {
        // MVP 计算: MVP次数 * 10 + 平均影响力 (MVP次数优先，影响力作为次要排序依据)
        let rows = sqlx::query(
            r#"
            SELECT *,
                   (game_mvp_count * 10.0 + avg_impact) as mvp_score
            FROM player_tournament_stats
            WHERE save_id = ? AND tournament_id = ? AND team_id = ? AND games_played >= 1
            ORDER BY mvp_score DESC
            LIMIT ?
            "#
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .bind(team_id as i64)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_tournament_stats).collect())
    }

    /// 删除赛事的所有选手统计
    pub async fn delete_by_tournament(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "DELETE FROM player_tournament_stats WHERE save_id = ? AND tournament_id = ?"
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 将数据库行转换为 PlayerTournamentStats
fn row_to_player_tournament_stats(row: &sqlx::sqlite::SqliteRow) -> PlayerTournamentStats {
    PlayerTournamentStats {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        tournament_id: row.get::<i64, _>("tournament_id") as u64,
        tournament_type: row.get("tournament_type"),
        player_id: row.get::<i64, _>("player_id") as u64,
        player_name: row.get("player_name"),
        team_id: row.get::<i64, _>("team_id") as u64,
        team_name: row.get("team_name"),
        position: row.get("position"),
        games_played: row.get::<i64, _>("games_played") as u32,
        games_won: row.get::<i64, _>("games_won") as u32,
        total_impact: row.get("total_impact"),
        avg_impact: row.get("avg_impact"),
        max_impact: row.get("max_impact"),
        avg_performance: row.get("avg_performance"),
        best_performance: row.get("best_performance"),
        game_mvp_count: row.get::<i64, _>("game_mvp_count") as u32,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

// ========================================
// 年度积分仓库
// ========================================

/// 年度积分仓库
pub struct PointsRepository;

impl PointsRepository {
    /// 添加积分明细记录（带去重检查，防止同一队伍在同一赛事中重复获得积分）
    /// 返回 (record_id, is_new_record) - 如果是新记录返回true，已存在则返回false
    pub async fn add_points_detail(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        team_id: u64,
        tournament_id: u64,
        points: u32,
        final_rank: Option<u32>,
    ) -> Result<(u64, bool), DatabaseError> {
        // 先检查是否已存在该队伍在该赛事的积分记录
        let existing: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT id FROM annual_points_detail
            WHERE save_id = ? AND season_id = ? AND team_id = ? AND tournament_id = ?
            "#,
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(team_id as i64)
        .bind(tournament_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 如果已存在，跳过插入，返回已有记录的ID和false标志
        if let Some((existing_id,)) = existing {
            log::debug!("积分记录已存在，跳过: team_id={}, tournament_id={}", team_id, tournament_id);
            return Ok((existing_id as u64, false));
        }

        let result = sqlx::query(
            r#"
            INSERT INTO annual_points_detail (save_id, season_id, team_id, tournament_id, points, final_rank)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(team_id as i64)
        .bind(tournament_id as i64)
        .bind(points as i64)
        .bind(final_rank.map(|r| r as i64))
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok((result.last_insert_rowid() as u64, true))
    }

    /// 批量添加积分明细（自动去重）
    pub async fn batch_add_points(
        pool: &Pool<Sqlite>,
        details: &[AnnualPointsDetail],
    ) -> Result<(), DatabaseError> {
        for detail in details {
            let _ = Self::add_points_detail(
                pool,
                &detail.save_id,
                detail.season_id,
                detail.team_id,
                detail.tournament_id,
                detail.points,
                detail.final_rank,
            )
            .await?;
        }
        Ok(())
    }

    /// 获取队伍在某赛季的所有积分明细
    pub async fn get_team_season_points(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        team_id: u64,
    ) -> Result<Vec<AnnualPointsDetail>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT apd.*, t.name as tournament_name, t.tournament_type
            FROM annual_points_detail apd
            LEFT JOIN tournaments t ON apd.tournament_id = t.id
            WHERE apd.save_id = ? AND apd.season_id = ? AND apd.team_id = ?
            ORDER BY apd.id ASC
            "#,
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| AnnualPointsDetail {
                id: row.get::<i64, _>("id") as u64,
                save_id: row.get("save_id"),
                season_id: row.get::<i64, _>("season_id") as u64,
                team_id: row.get::<i64, _>("team_id") as u64,
                tournament_id: row.get::<i64, _>("tournament_id") as u64,
                tournament_name: row.get("tournament_name"),
                tournament_type: row.get("tournament_type"),
                points: row.get::<i64, _>("points") as u32,
                final_rank: row.get::<Option<i64>, _>("final_rank").map(|r| r as u32),
            })
            .collect())
    }

    /// 获取赛季年度积分排名（汇总所有队伍的积分）
    pub async fn get_season_rankings(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<TeamAnnualPoints>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT
                t.id as team_id,
                t.name as team_name,
                t.short_name as team_short_name,
                t.region_id,
                COALESCE(r.name, 'N/A') as region_code,
                COALESCE(SUM(apd.points), 0) as total_points,
                COUNT(DISTINCT apd.tournament_id) as tournaments_count
            FROM teams t
            LEFT JOIN regions r ON t.region_id = r.id
            LEFT JOIN annual_points_detail apd ON apd.team_id = t.id
              AND apd.save_id = ? AND apd.season_id = ?
            WHERE t.save_id = ?
            GROUP BY t.id
            ORDER BY total_points DESC
            "#,
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows
            .iter()
            .enumerate()
            .map(|(idx, row)| TeamAnnualPoints {
                rank: (idx + 1) as u32,
                team_id: row.get::<i64, _>("team_id") as u64,
                team_name: row.get("team_name"),
                team_short_name: row.get("team_short_name"),
                region_id: row.get::<i64, _>("region_id") as u64,
                region_code: row.get("region_code"),
                total_points: row.get::<i64, _>("total_points") as u32,
                tournaments_count: row.get::<i64, _>("tournaments_count") as u32,
            })
            .collect())
    }

    /// 获取赛事的积分明细
    pub async fn get_tournament_points(
        pool: &Pool<Sqlite>,
        save_id: &str,
        tournament_id: u64,
    ) -> Result<Vec<AnnualPointsDetail>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT apd.*, t.name as tournament_name, t.tournament_type
            FROM annual_points_detail apd
            LEFT JOIN tournaments t ON apd.tournament_id = t.id
            WHERE apd.save_id = ? AND apd.tournament_id = ?
            ORDER BY apd.points DESC
            "#,
        )
        .bind(save_id)
        .bind(tournament_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| AnnualPointsDetail {
                id: row.get::<i64, _>("id") as u64,
                save_id: row.get("save_id"),
                season_id: row.get::<i64, _>("season_id") as u64,
                team_id: row.get::<i64, _>("team_id") as u64,
                tournament_id: row.get::<i64, _>("tournament_id") as u64,
                tournament_name: row.get("tournament_name"),
                tournament_type: row.get("tournament_type"),
                points: row.get::<i64, _>("points") as u32,
                final_rank: row.get::<Option<i64>, _>("final_rank").map(|r| r as u32),
            })
            .collect())
    }

    /// 删除赛季的所有积分明细（新赛季开始时调用）
    pub async fn clear_season_points(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "DELETE FROM annual_points_detail WHERE save_id = ? AND season_id = ?"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

/// 队伍年度积分（用于排名展示）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamAnnualPoints {
    pub rank: u32,
    pub team_id: u64,
    pub team_name: String,
    pub team_short_name: Option<String>,
    pub region_id: u64,
    pub region_code: String,
    pub total_points: u32,
    pub tournaments_count: u32,
}

// ==================== 比赛详情仓库 ====================

use crate::models::match_game_detail::{
    MatchGameDetail, GamePlayerPerformance, MatchFullDetails,
    GameDetailWithPerformances, SaveMatchDetailsInput
};

/// 比赛详情仓库
pub struct MatchGameDetailRepository;

impl MatchGameDetailRepository {
    /// 保存比赛详情（包含所有局和选手表现）
    pub async fn save_match_details(
        pool: &Pool<Sqlite>,
        save_id: &str,
        input: &SaveMatchDetailsInput,
    ) -> Result<(), DatabaseError> {
        for game_input in &input.games {
            // 生成游戏ID
            let game_id = format!("{}_{}", input.match_id, game_input.game_number);

            // 保存单局数据
            sqlx::query(
                r#"
                INSERT INTO match_games (
                    id, save_id, match_id, game_number, winner_team_id, loser_team_id,
                    duration_minutes, mvp_player_id, key_player_id
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    winner_team_id = excluded.winner_team_id,
                    loser_team_id = excluded.loser_team_id,
                    duration_minutes = excluded.duration_minutes,
                    mvp_player_id = excluded.mvp_player_id,
                    key_player_id = excluded.key_player_id
                "#
            )
            .bind(&game_id)
            .bind(save_id)
            .bind(input.match_id)
            .bind(game_input.game_number)
            .bind(game_input.winner_team_id)
            .bind(game_input.loser_team_id)
            .bind(game_input.duration_minutes)
            .bind(game_input.mvp_player_id)
            .bind(game_input.key_player_id)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

            // 保存选手表现
            for perf in &game_input.performances {
                let perf_id = format!("{}_{}_{}", game_id, perf.player_id, perf.position);

                sqlx::query(
                    r#"
                    INSERT INTO game_player_performances (
                        id, save_id, game_id, player_id, player_name, team_id, team_name, position,
                        base_ability, condition_bonus, stability_noise, actual_ability,
                        impact_score, mvp_score, is_mvp, is_key_player,
                        kills, deaths, assists, cs, gold, damage_dealt, damage_taken, vision_score,
                        traits_json, activated_traits_json
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET
                        player_name = excluded.player_name,
                        team_name = excluded.team_name,
                        base_ability = excluded.base_ability,
                        condition_bonus = excluded.condition_bonus,
                        stability_noise = excluded.stability_noise,
                        actual_ability = excluded.actual_ability,
                        impact_score = excluded.impact_score,
                        mvp_score = excluded.mvp_score,
                        is_mvp = excluded.is_mvp,
                        is_key_player = excluded.is_key_player,
                        kills = excluded.kills,
                        deaths = excluded.deaths,
                        assists = excluded.assists,
                        cs = excluded.cs,
                        gold = excluded.gold,
                        damage_dealt = excluded.damage_dealt,
                        damage_taken = excluded.damage_taken,
                        vision_score = excluded.vision_score,
                        traits_json = excluded.traits_json,
                        activated_traits_json = excluded.activated_traits_json
                    "#
                )
                .bind(&perf_id)
                .bind(save_id)
                .bind(&game_id)
                .bind(perf.player_id)
                .bind(&perf.player_name)
                .bind(perf.team_id)
                .bind(&perf.team_name)
                .bind(&perf.position)
                .bind(perf.base_ability)
                .bind(perf.condition_bonus)
                .bind(perf.stability_noise)
                .bind(perf.actual_ability)
                .bind(perf.impact_score)
                .bind(perf.mvp_score)
                .bind(if perf.is_mvp { 1 } else { 0 })
                .bind(if perf.is_key_player { 1 } else { 0 })
                .bind(perf.kills)
                .bind(perf.deaths)
                .bind(perf.assists)
                .bind(perf.cs)
                .bind(perf.gold)
                .bind(perf.damage_dealt)
                .bind(perf.damage_taken)
                .bind(perf.vision_score)
                .bind(&perf.traits_json)
                .bind(&perf.activated_traits_json)
                .execute(pool)
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 获取比赛的所有局详情
    pub async fn get_match_games(
        pool: &Pool<Sqlite>,
        save_id: &str,
        match_id: i64,
    ) -> Result<Vec<MatchGameDetail>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM match_games WHERE save_id = ? AND match_id = ? ORDER BY game_number"
        )
        .bind(save_id)
        .bind(match_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_match_game_detail).collect())
    }

    /// 获取某局的选手表现
    pub async fn get_game_performances(
        pool: &Pool<Sqlite>,
        save_id: &str,
        game_id: &str,
    ) -> Result<Vec<GamePlayerPerformance>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM game_player_performances WHERE save_id = ? AND game_id = ? ORDER BY position"
        )
        .bind(save_id)
        .bind(game_id)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_game_player_performance).collect())
    }

    /// 获取完整比赛详情（包含所有局和选手表现）
    pub async fn get_match_full_details(
        pool: &Pool<Sqlite>,
        save_id: &str,
        match_id: i64,
    ) -> Result<Option<MatchFullDetails>, DatabaseError> {
        let games = Self::get_match_games(pool, save_id, match_id).await?;

        if games.is_empty() {
            return Ok(None);
        }

        let mut game_details = Vec::new();
        for game in games {
            let performances = Self::get_game_performances(pool, save_id, &game.id).await?;
            game_details.push(GameDetailWithPerformances {
                game,
                performances,
            });
        }

        Ok(Some(MatchFullDetails {
            match_id,
            games: game_details,
        }))
    }

    /// 删除比赛的所有详情数据
    pub async fn delete_match_details(
        pool: &Pool<Sqlite>,
        save_id: &str,
        match_id: i64,
    ) -> Result<(), DatabaseError> {
        // 先删除选手表现（有外键约束）
        sqlx::query(
            r#"
            DELETE FROM game_player_performances
            WHERE save_id = ? AND game_id IN (
                SELECT id FROM match_games WHERE save_id = ? AND match_id = ?
            )
            "#
        )
        .bind(save_id)
        .bind(save_id)
        .bind(match_id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // 再删除比赛局数据
        sqlx::query(
            "DELETE FROM match_games WHERE save_id = ? AND match_id = ?"
        )
        .bind(save_id)
        .bind(match_id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// 获取选手的影响力历史记录
    pub async fn get_player_impact_history(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        season_id: Option<i64>,
    ) -> Result<Vec<f64>, DatabaseError> {
        // 通过 game_id 关联 match_games 表，再关联 matches 表获取 tournament_id
        // 然后根据 tournament 的 season_id 过滤
        let query = if let Some(season) = season_id {
            sqlx::query_scalar::<_, f64>(
                r#"
                SELECT gpp.impact_score
                FROM game_player_performances gpp
                JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
                JOIN matches m ON mg.match_id = m.id
                JOIN tournaments t ON m.tournament_id = t.id
                WHERE gpp.save_id = ? AND gpp.player_id = ? AND t.season_id = ?
                ORDER BY gpp.created_at ASC
                "#
            )
            .bind(save_id)
            .bind(player_id)
            .bind(season)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_scalar::<_, f64>(
                r#"
                SELECT impact_score
                FROM game_player_performances
                WHERE save_id = ? AND player_id = ?
                ORDER BY created_at ASC
                "#
            )
            .bind(save_id)
            .bind(player_id)
            .fetch_all(pool)
            .await
        };

        query.map_err(|e| DatabaseError::Query(e.to_string()))
    }
}

/// 将数据库行转换为 MatchGameDetail
fn row_to_match_game_detail(row: &sqlx::sqlite::SqliteRow) -> MatchGameDetail {
    MatchGameDetail {
        id: row.get("id"),
        save_id: row.get("save_id"),
        match_id: row.get("match_id"),
        game_number: row.get("game_number"),
        winner_team_id: row.get("winner_team_id"),
        loser_team_id: row.get("loser_team_id"),
        duration_minutes: row.get("duration_minutes"),
        mvp_player_id: row.get("mvp_player_id"),
        key_player_id: row.get("key_player_id"),
        created_at: row.get("created_at"),
    }
}

/// 将数据库行转换为 GamePlayerPerformance
fn row_to_game_player_performance(row: &sqlx::sqlite::SqliteRow) -> GamePlayerPerformance {
    GamePlayerPerformance {
        id: row.get("id"),
        save_id: row.get("save_id"),
        game_id: row.get("game_id"),
        player_id: row.get("player_id"),
        player_name: row.get("player_name"),
        team_id: row.get("team_id"),
        team_name: row.get("team_name"),
        position: row.get("position"),
        base_ability: row.get("base_ability"),
        condition_bonus: row.get("condition_bonus"),
        stability_noise: row.get("stability_noise"),
        actual_ability: row.get("actual_ability"),
        impact_score: row.get("impact_score"),
        mvp_score: row.get("mvp_score"),
        is_mvp: row.get::<i32, _>("is_mvp") != 0,
        is_key_player: row.get::<i32, _>("is_key_player") != 0,
        kills: row.get("kills"),
        deaths: row.get("deaths"),
        assists: row.get("assists"),
        cs: row.get("cs"),
        gold: row.get("gold"),
        damage_dealt: row.get("damage_dealt"),
        damage_taken: row.get("damage_taken"),
        vision_score: row.get("vision_score"),
        traits_json: row.get("traits_json"),
        activated_traits_json: row.get("activated_traits_json"),
        created_at: row.get("created_at"),
    }
}

// ==================== 选手状态仓库 ====================

/// 选手赛季状态仓库
pub struct PlayerStatusRepository;

impl PlayerStatusRepository {
    /// 创建或更新选手赛季状态
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        status: &PlayerSeasonStatus,
    ) -> Result<u64, DatabaseError> {
        let reasons_json = serde_json::to_string(&status.departure_reasons)
            .unwrap_or_else(|_| "[]".to_string());

        let result = sqlx::query(
            r#"
            INSERT INTO player_season_status (
                save_id, season_id, player_id, satisfaction, wants_to_leave,
                departure_reasons, games_as_starter, total_games, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(save_id, season_id, player_id) DO UPDATE SET
                satisfaction = excluded.satisfaction,
                wants_to_leave = excluded.wants_to_leave,
                departure_reasons = excluded.departure_reasons,
                games_as_starter = excluded.games_as_starter,
                total_games = excluded.total_games,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&status.save_id)
        .bind(status.season_id as i64)
        .bind(status.player_id as i64)
        .bind(status.satisfaction as i32)
        .bind(status.wants_to_leave)
        .bind(&reasons_json)
        .bind(status.games_as_starter as i32)
        .bind(status.total_games as i32)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取选手赛季状态
    pub async fn get(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        player_id: u64,
    ) -> Result<Option<PlayerSeasonStatus>, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM player_season_status WHERE save_id = ? AND season_id = ? AND player_id = ?"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(player_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| row_to_player_season_status(&r)))
    }

    /// 获取球队所有选手的赛季状态
    pub async fn get_by_team(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        team_id: u64,
    ) -> Result<Vec<PlayerSeasonStatus>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT pss.* FROM player_season_status pss
            INNER JOIN players p ON pss.player_id = p.id
            WHERE pss.save_id = ? AND pss.season_id = ? AND p.team_id = ?
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(team_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_season_status).collect())
    }

    /// 获取所有想离队的选手
    pub async fn get_departure_candidates(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<PlayerSeasonStatus>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM player_season_status WHERE save_id = ? AND season_id = ? AND wants_to_leave = TRUE"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_player_season_status).collect())
    }

    /// 批量创建或更新选手赛季状态
    pub async fn batch_upsert(
        pool: &Pool<Sqlite>,
        statuses: &[PlayerSeasonStatus],
    ) -> Result<u32, DatabaseError> {
        let mut count = 0;
        for status in statuses {
            Self::upsert(pool, status).await?;
            count += 1;
        }
        Ok(count)
    }
}

/// 球队赛季表现仓库
pub struct TeamSeasonPerformanceRepository;

impl TeamSeasonPerformanceRepository {
    /// 创建或更新球队赛季表现
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        perf: &TeamSeasonPerformance,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO team_season_performance (
                save_id, season_id, team_id, final_rank, made_playoffs,
                playoff_result, international_result, consecutive_no_playoffs
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(save_id, season_id, team_id) DO UPDATE SET
                final_rank = excluded.final_rank,
                made_playoffs = excluded.made_playoffs,
                playoff_result = excluded.playoff_result,
                international_result = excluded.international_result,
                consecutive_no_playoffs = excluded.consecutive_no_playoffs
            "#,
        )
        .bind(&perf.save_id)
        .bind(perf.season_id as i64)
        .bind(perf.team_id as i64)
        .bind(perf.final_rank.map(|r| r as i32))
        .bind(perf.made_playoffs)
        .bind(&perf.playoff_result)
        .bind(&perf.international_result)
        .bind(perf.consecutive_no_playoffs as i32)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取球队赛季表现
    pub async fn get(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        team_id: u64,
    ) -> Result<Option<TeamSeasonPerformance>, DatabaseError> {
        let row = sqlx::query(
            "SELECT * FROM team_season_performance WHERE save_id = ? AND season_id = ? AND team_id = ?"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(team_id as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| row_to_team_season_performance(&r)))
    }

    /// 获取上赛季球队表现（用于计算连续未进季后赛）
    pub async fn get_previous_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        team_id: u64,
    ) -> Result<Option<TeamSeasonPerformance>, DatabaseError> {
        if season_id == 0 {
            return Ok(None);
        }
        Self::get(pool, save_id, season_id - 1, team_id).await
    }

    /// 获取所有球队的赛季表现
    pub async fn get_all_for_season(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
    ) -> Result<Vec<TeamSeasonPerformance>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM team_season_performance WHERE save_id = ? AND season_id = ?"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_team_season_performance).collect())
    }
}

/// 忠诚度变化记录仓库
pub struct LoyaltyChangeRepository;

impl LoyaltyChangeRepository {
    /// 创建忠诚度变化记录
    pub async fn create(
        pool: &Pool<Sqlite>,
        change: &LoyaltyChange,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO loyalty_changes (save_id, season_id, player_id, change_amount, reason)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&change.save_id)
        .bind(change.season_id as i64)
        .bind(change.player_id as i64)
        .bind(change.change_amount)
        .bind(&change.reason)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// 获取选手的忠诚度变化历史
    pub async fn get_by_player(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: u64,
    ) -> Result<Vec<LoyaltyChange>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT * FROM loyalty_changes WHERE save_id = ? AND player_id = ? ORDER BY created_at DESC"
        )
        .bind(save_id)
        .bind(player_id as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_loyalty_change).collect())
    }
}

// ==================== 辅助转换函数 ====================

/// 将数据库行转换为 PlayerSeasonStatus
fn row_to_player_season_status(row: &sqlx::sqlite::SqliteRow) -> PlayerSeasonStatus {
    let reasons_json: String = row.get("departure_reasons");
    let departure_reasons: Vec<DepartureReason> = serde_json::from_str(&reasons_json)
        .unwrap_or_default();

    PlayerSeasonStatus {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        player_id: row.get::<i64, _>("player_id") as u64,
        satisfaction: row.get::<i32, _>("satisfaction") as u8,
        wants_to_leave: row.get("wants_to_leave"),
        departure_reasons,
        games_as_starter: row.get::<i32, _>("games_as_starter") as u32,
        total_games: row.get::<i32, _>("total_games") as u32,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// 将数据库行转换为 TeamSeasonPerformance
fn row_to_team_season_performance(row: &sqlx::sqlite::SqliteRow) -> TeamSeasonPerformance {
    TeamSeasonPerformance {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        team_id: row.get::<i64, _>("team_id") as u64,
        final_rank: row.get::<Option<i32>, _>("final_rank").map(|r| r as u32),
        made_playoffs: row.get("made_playoffs"),
        playoff_result: row.get("playoff_result"),
        international_result: row.get("international_result"),
        consecutive_no_playoffs: row.get::<i32, _>("consecutive_no_playoffs") as u32,
        created_at: row.get("created_at"),
    }
}

/// 将数据库行转换为 LoyaltyChange
fn row_to_loyalty_change(row: &sqlx::sqlite::SqliteRow) -> LoyaltyChange {
    LoyaltyChange {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        player_id: row.get::<i64, _>("player_id") as u64,
        change_amount: row.get("change_amount"),
        reason: row.get("reason"),
        created_at: row.get("created_at"),
    }
}

// ==================== LLM 任务日志仓库 ====================

/// LLM 任务日志仓库
pub struct LLMTaskLogRepository;

impl LLMTaskLogRepository {
    /// 插入或更新任务
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        task: &LLMTaskLog,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO llm_task_log
            (save_id, season_id, task_type, entity_id, entity_type, status, attempt_count,
             max_attempts, error_msg, last_error_at, created_at, updated_at, completed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(save_id, season_id, task_type, entity_id)
            DO UPDATE SET
                status = excluded.status,
                attempt_count = excluded.attempt_count,
                error_msg = excluded.error_msg,
                last_error_at = excluded.last_error_at,
                updated_at = excluded.updated_at,
                completed_at = excluded.completed_at
            RETURNING id
            "#,
        )
        .bind(&task.save_id)
        .bind(task.season_id as i64)
        .bind(task.task_type.as_str())
        .bind(task.entity_id as i64)
        .bind(&task.entity_type)
        .bind(task.status.as_str())
        .bind(task.attempt_count as i64)
        .bind(task.max_attempts as i64)
        .bind(&task.error_msg)
        .bind(&task.last_error_at)
        .bind(&task.created_at)
        .bind(&task.updated_at)
        .bind(&task.completed_at)
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.get::<i64, _>(0) as u64)
    }

    /// 查询失败的任务
    pub async fn get_failed_tasks(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        task_type: Option<TaskType>,
    ) -> Result<Vec<LLMTaskLog>, DatabaseError> {
        let mut query = "SELECT * FROM llm_task_log WHERE save_id = ? AND season_id = ? AND status = 'failed'".to_string();

        if task_type.is_some() {
            query.push_str(" AND task_type = ?");
        }

        let mut sql = sqlx::query(&query)
            .bind(save_id)
            .bind(season_id as i64);

        if let Some(tt) = task_type {
            sql = sql.bind(tt.as_str());
        }

        let rows = sql
            .fetch_all(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(row_to_llm_task_log).collect())
    }

    /// 查询成功的任务 ID
    pub async fn get_success_ids(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        task_type: TaskType,
    ) -> Result<Vec<u64>, DatabaseError> {
        let rows = sqlx::query(
            "SELECT entity_id FROM llm_task_log WHERE save_id = ? AND season_id = ? AND task_type = ? AND status = 'success'"
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(task_type.as_str())
        .fetch_all(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get::<i64, _>(0) as u64).collect())
    }

    /// 获取任务统计
    pub async fn get_task_stats(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: u64,
        task_type: TaskType,
    ) -> Result<TaskStats, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END) as running,
                SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed
            FROM llm_task_log
            WHERE save_id = ? AND season_id = ? AND task_type = ?
            "#
        )
        .bind(save_id)
        .bind(season_id as i64)
        .bind(task_type.as_str())
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        if let Some(row) = row {
            Ok(TaskStats {
                total: row.get::<i64, _>(0) as u32,
                pending: row.get::<i64, _>(1) as u32,
                running: row.get::<i64, _>(2) as u32,
                success: row.get::<i64, _>(3) as u32,
                failed: row.get::<i64, _>(4) as u32,
            })
        } else {
            Ok(TaskStats::new())
        }
    }

    /// 清理旧任务（可选，防止表过大）
    pub async fn clean_old_tasks(
        pool: &Pool<Sqlite>,
        days_to_keep: i64,
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(
            "DELETE FROM llm_task_log WHERE datetime(created_at) < datetime('now', ?)"
        )
        .bind(format!("-{} days", days_to_keep))
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

/// 将数据库行转换为 LLMTaskLog
fn row_to_llm_task_log(row: &sqlx::sqlite::SqliteRow) -> LLMTaskLog {
    let task_type_str: String = row.get("task_type");
    let status_str: String = row.get("status");

    LLMTaskLog {
        id: row.get::<i64, _>("id") as u64,
        save_id: row.get("save_id"),
        season_id: row.get::<i64, _>("season_id") as u64,
        task_type: TaskType::from_str(&task_type_str).unwrap_or(TaskType::Intention),
        entity_id: row.get::<i64, _>("entity_id") as u64,
        entity_type: row.get("entity_type"),
        status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
        attempt_count: row.get::<i64, _>("attempt_count") as u32,
        max_attempts: row.get::<i64, _>("max_attempts") as u32,
        error_msg: row.get("error_msg"),
        last_error_at: row.get("last_error_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        completed_at: row.get("completed_at"),
    }
}
