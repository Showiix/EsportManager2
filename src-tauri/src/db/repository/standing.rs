use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

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
