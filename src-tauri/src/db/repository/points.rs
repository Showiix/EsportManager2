use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

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
