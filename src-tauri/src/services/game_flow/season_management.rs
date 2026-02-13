use crate::db::*;
use crate::engines::MetaEngine;
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::{GameFlowService, NewSeasonResult};

impl GameFlowService {
    /// 自动确认首发：为每支队伍的每个位置选能力最高的选手设为首发
    pub async fn auto_confirm_starters(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<u32, String> {
        // 清除所有首发标记
        sqlx::query("UPDATE players SET is_starter = 0 WHERE save_id = ? AND status = 'Active'")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("清除首发失败: {}", e))?;

        // 获取所有队伍
        let teams = TeamRepository::get_all(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let positions = ["Top", "Jug", "Mid", "Adc", "Sup"];
        let mut confirmed_count = 0u32;

        for team in &teams {
            let mut team_confirmed = 0u32;
            for pos in &positions {
                // 找到该队伍该位置能力最高的选手
                let result = sqlx::query(
                    r#"
                    SELECT id, game_id, ability, position FROM players
                    WHERE save_id = ? AND team_id = ? AND status = 'Active'
                      AND UPPER(position) = UPPER(?)
                    ORDER BY ability DESC
                    LIMIT 1
                    "#,
                )
                .bind(save_id)
                .bind(team.id as i64)
                .bind(pos)
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("查询最强选手失败: {}", e))?;

                if let Some(row) = result {
                    let player_id: i64 = row.get("id");
                    sqlx::query("UPDATE players SET is_starter = 1 WHERE id = ?")
                        .bind(player_id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("设置首发失败: {}", e))?;
                    confirmed_count += 1;
                    team_confirmed += 1;
                } else {
                    // 打印该队伍所有选手的位置信息，帮助排查
                    let all_players: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
                        "SELECT id, game_id, position, status, ability FROM players WHERE save_id = ? AND team_id = ? ORDER BY position"
                    )
                    .bind(save_id)
                    .bind(team.id as i64)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

                    let player_info: Vec<String> = all_players.iter().map(|p| {
                        format!("{}({}, {}, ability={})",
                            p.get::<String, _>("game_id"),
                            p.get::<String, _>("position"),
                            p.get::<String, _>("status"),
                            p.get::<i64, _>("ability"))
                    }).collect();

                    log::debug!("警告: 战队 {} (id={}) 缺少 {} 位置的选手! 该队所有选手: {:?}",
                        team.name, team.id, pos, player_info);
                }
            }
            if team_confirmed < 5 {
                log::debug!("战队 {} (id={}) 只确认了 {}/5 名首发!", team.name, team.id, team_confirmed);
            }
        }

        log::debug!("确认了 {} 名首发选手", confirmed_count);
        Ok(confirmed_count)
    }

    /// 重新计算所有队伍的战力值：取首发选手能力值的平均值
    pub async fn recalculate_team_powers(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<(), String> {
        let teams = TeamRepository::get_all(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        for team in &teams {
            let avg_ability: f64 = sqlx::query_scalar(
                r#"
                SELECT COALESCE(AVG(ability), 60.0) FROM players
                WHERE save_id = ? AND team_id = ? AND status = 'Active' AND is_starter = 1
                "#,
            )
            .bind(save_id)
            .bind(team.id as i64)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("计算队伍战力失败: {}", e))?;

            sqlx::query("UPDATE teams SET power_rating = ? WHERE id = ?")
                .bind(avg_ability)
                .bind(team.id as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("更新队伍战力失败: {}", e))?;
        }

        log::debug!("更新了 {} 支队伍的战力", teams.len());
        Ok(())
    }

    /// 推进到新赛季
    pub async fn advance_to_new_season(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
    ) -> Result<NewSeasonResult, String> {
        // 获取当前存档
        let mut save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        // 1. 更新赛季号和阶段
        save.current_season += 1;
        save.current_phase = SeasonPhase::SpringRegular;
        save.phase_completed = false;
        save.updated_at = chrono::Utc::now();

        SaveRepository::update(pool, &save)
            .await
            .map_err(|e| e.to_string())?;

        // 2. 批量重置年度积分
        sqlx::query("UPDATE teams SET annual_points = 0 WHERE save_id = ?")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("重置年度积分失败: {}", e))?;

        // 2.5 重置选手 form factors（新赛季状态重置）
        sqlx::query(
            r#"
            UPDATE player_form_factors
            SET momentum = 0,
                last_performance = 0.0,
                last_match_won = 0,
                games_since_rest = 0,
                form_cycle = (ABS(RANDOM()) % 10000) / 100.0,
                updated_at = datetime('now')
            WHERE save_id = ?
            "#,
        )
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| format!("重置 form factors 失败: {}", e))?;

        // 2.6 为新赛季生成 Meta 版本
        MetaEngine::roll_new_meta(pool, save_id, save.current_season as i64).await
            .map_err(|e| format!("生成 Meta 版本失败: {}", e))?;

        // 3. 自动确认首发
        let starters_confirmed = self.auto_confirm_starters(pool, save_id).await?;

        // 4. 更新战力
        self.recalculate_team_powers(pool, save_id).await?;

        // 5. 初始化春季赛（创建4个赛区的赛事、赛程、积分榜）
        self.initialize_phase(pool, save_id, save.current_season as u64, SeasonPhase::SpringRegular).await?;

        let message = format!(
            "已进入第 {} 赛季，确认了 {} 名首发选手，已更新战力并创建春季赛",
            save.current_season, starters_confirmed
        );

        Ok(NewSeasonResult {
            new_season: save.current_season,
            starters_confirmed,
            message,
        })
    }
}
