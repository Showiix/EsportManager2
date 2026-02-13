use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

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
