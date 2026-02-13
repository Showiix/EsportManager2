use crate::db::DatabaseError;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};
use super::helpers::*;
use serde_json;

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

    /// 统计战队MVP数量（该战队选手获得的MVP荣誉）
    pub async fn count_team_mvps(
        pool: &Pool<Sqlite>,
        save_id: &str,
        team_id: u64,
    ) -> Result<u32, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM honors
            WHERE save_id = ? AND team_id = ? AND honor_type IN ('TOURNAMENT_MVP', 'FINALS_MVP', 'REGULAR_SEASON_MVP', 'PLAYOFFS_FMVP')
            "#
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
            WHERE save_id = ? AND player_id = ? AND honor_type IN ('TOURNAMENT_MVP', 'FINALS_MVP', 'REGULAR_SEASON_MVP', 'PLAYOFFS_FMVP')
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

    /// 获取选手荣誉排行榜（按冠军数+MVP数排序，含选手当前信息）
    pub async fn get_player_honor_rankings(
        pool: &Pool<Sqlite>,
        save_id: &str,
        limit: i32,
    ) -> Result<Vec<(u64, String, u32, u32, u32, Option<u64>, Option<String>, Option<String>)>, DatabaseError> {
        // 返回: (player_id, player_name, champion_count, mvp_count, international_champion_count, team_id, team_name, position)
        let rows = sqlx::query(
            r#"
            SELECT
                h.player_id,
                h.player_name,
                SUM(CASE WHEN h.honor_type = 'PLAYER_CHAMPION' THEN 1 ELSE 0 END) as champion_count,
                SUM(CASE WHEN h.honor_type IN ('TOURNAMENT_MVP', 'FINALS_MVP', 'REGULAR_SEASON_MVP', 'PLAYOFFS_FMVP') THEN 1 ELSE 0 END) as mvp_count,
                SUM(CASE WHEN h.honor_type = 'PLAYER_CHAMPION' AND h.tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters', 'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental') THEN 1 ELSE 0 END) as intl_count,
                p.team_id,
                t.name as team_name,
                p.position
            FROM honors h
            LEFT JOIN players p ON h.player_id = p.id
            LEFT JOIN teams t ON p.team_id = t.id
            WHERE h.save_id = ? AND h.player_id IS NOT NULL
            GROUP BY h.player_id, h.player_name
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
                row.get::<Option<i64>, _>("team_id").map(|id| id as u64),
                row.get::<Option<String>, _>("team_name"),
                row.get::<Option<String>, _>("position"),
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
