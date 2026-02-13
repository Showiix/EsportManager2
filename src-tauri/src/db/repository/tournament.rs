use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

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
