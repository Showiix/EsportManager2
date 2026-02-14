use crate::db::DatabaseError;
use crate::models::*;
use crate::models::match_game_detail::*;
use sqlx::{Pool, Sqlite};
use super::helpers::*;

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
                    duration_minutes, mvp_player_id, key_player_id,
                    home_power, away_power, home_meta_power, away_meta_power,
                    home_base_power, away_base_power,
                    home_synergy_bonus, away_synergy_bonus,
                    home_bp_bonus, away_bp_bonus,
                    home_version_bonus, away_version_bonus
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    winner_team_id = excluded.winner_team_id,
                    loser_team_id = excluded.loser_team_id,
                    duration_minutes = excluded.duration_minutes,
                    mvp_player_id = excluded.mvp_player_id,
                    key_player_id = excluded.key_player_id,
                    home_power = excluded.home_power,
                    away_power = excluded.away_power,
                    home_meta_power = excluded.home_meta_power,
                    away_meta_power = excluded.away_meta_power,
                    home_base_power = excluded.home_base_power,
                    away_base_power = excluded.away_base_power,
                    home_synergy_bonus = excluded.home_synergy_bonus,
                    away_synergy_bonus = excluded.away_synergy_bonus,
                    home_bp_bonus = excluded.home_bp_bonus,
                    away_bp_bonus = excluded.away_bp_bonus,
                    home_version_bonus = excluded.home_version_bonus,
                    away_version_bonus = excluded.away_version_bonus
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
            .bind(game_input.home_power)
            .bind(game_input.away_power)
            .bind(game_input.home_meta_power)
            .bind(game_input.away_meta_power)
            .bind(game_input.home_base_power)
            .bind(game_input.away_base_power)
            .bind(game_input.home_synergy_bonus)
            .bind(game_input.away_synergy_bonus)
            .bind(game_input.home_bp_bonus)
            .bind(game_input.away_bp_bonus)
            .bind(game_input.home_version_bonus)
            .bind(game_input.away_version_bonus)
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
