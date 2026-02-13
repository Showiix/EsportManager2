use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

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
            INSERT INTO teams (save_id, region_id, name, short_name, power_rating, balance, brand_value)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(team.region_id as i64)
        .bind(&team.name)
        .bind(&team.short_name)
        .bind(team.power_rating)
        .bind(team.balance as i64)
        .bind(team.brand_value)
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
                balance = ?,
                brand_value = ?
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
        .bind(team.brand_value)
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
