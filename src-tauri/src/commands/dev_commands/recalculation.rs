use sqlx::Row;
use tauri::State;
use crate::commands::save_commands::AppState;
use crate::db::{SaveRepository, TournamentRepository, MatchRepository};
use crate::engines::PointsCalculationEngine;
use crate::models::TournamentStatus;
use crate::engines::market_value::MarketValueEngine;
use crate::models::{PlayerTag, Position, PlayerSeasonStatistics};
use super::{DevCommandResult, RebuildStatsResult};

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_recalculate_annual_points(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    sqlx::query("DELETE FROM annual_points_detail WHERE save_id = ? AND season_id = ?")
        .bind(&save_id)
        .bind(season)
        .execute(&pool)
        .await
        .ok();

    sqlx::query("UPDATE teams SET annual_points = 0 WHERE save_id = ?")
        .bind(&save_id)
        .execute(&pool)
        .await
        .ok();

    let tournaments = TournamentRepository::get_by_season(&pool, &save_id, season as u64)
        .await
        .map_err(|e| e.to_string())?;

    let points_engine = PointsCalculationEngine::default();
    let mut updated_count = 0;

    for tournament in tournaments.iter().filter(|t| t.status == TournamentStatus::Completed) {
        let results = sqlx::query(
            "SELECT team_id, position FROM tournament_results WHERE save_id = ? AND tournament_id = ?"
        )
        .bind(&save_id)
        .bind(tournament.id as i64)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        for row in results {
            let team_id: i64 = row.get("team_id");
            let position: i64 = row.get("position");

            let position_str = match position {
                1 => "CHAMPION",
                2 => "RUNNER_UP",
                3 => "THIRD",
                4 => "FOURTH",
                5..=8 => "5TH_8TH",
                _ => continue,
            };

            let points = points_engine.get_points(tournament.tournament_type, position_str);

            if points > 0 {
                sqlx::query("UPDATE teams SET annual_points = annual_points + ? WHERE id = ?")
                    .bind(points as i64)
                    .bind(team_id)
                    .execute(&pool)
                    .await
                    .ok();

                sqlx::query(
                    "INSERT INTO annual_points_detail (save_id, season_id, team_id, tournament_id, points, position)
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(&save_id)
                .bind(season)
                .bind(team_id)
                .bind(tournament.id as i64)
                .bind(points as i64)
                .bind(position)
                .execute(&pool)
                .await
                .ok();

                updated_count += 1;
            }
        }
    }

    Ok(DevCommandResult::ok(
        updated_count,
        format!("成功重新计算年度积分，更新了 {} 条记录", updated_count),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_recalculate_standings(
    state: State<'_, AppState>,
    tournament_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let tournaments = if let Some(tid) = tournament_id {
        vec![TournamentRepository::get_by_id(&pool, tid as u64)
            .await
            .map_err(|e| e.to_string())?]
    } else {
        let save = SaveRepository::get_by_id(&pool, &save_id)
            .await
            .map_err(|e| e.to_string())?;
        TournamentRepository::get_by_season(&pool, &save_id, save.current_season as u64)
            .await
            .map_err(|e| e.to_string())?
    };

    let mut updated_count = 0;

    for tournament in &tournaments {
        if !format!("{:?}", tournament.tournament_type).contains("Regular") {
            continue;
        }

        let matches = MatchRepository::get_by_tournament(&pool, tournament.id)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "UPDATE league_standings SET matches_played = 0, wins = 0, losses = 0, points = 0, games_won = 0, games_lost = 0, game_diff = 0 WHERE tournament_id = ?"
        )
        .bind(tournament.id as i64)
        .execute(&pool)
        .await
        .ok();

        for m in matches.iter().filter(|m| m.status == crate::models::MatchStatus::Completed) {
            let home_score = m.home_score;
            let away_score = m.away_score;
            let home_won = home_score > away_score;

            sqlx::query(
                r#"
                UPDATE league_standings SET
                    matches_played = matches_played + 1,
                    wins = wins + ?,
                    losses = losses + ?,
                    points = points + ?,
                    games_won = games_won + ?,
                    games_lost = games_lost + ?,
                    game_diff = game_diff + ?
                WHERE tournament_id = ? AND team_id = ?
                "#
            )
            .bind(if home_won { 1 } else { 0 })
            .bind(if home_won { 0 } else { 1 })
            .bind(if home_won { 3 } else { 0 })
            .bind(home_score as i64)
            .bind(away_score as i64)
            .bind((home_score as i64) - (away_score as i64))
            .bind(tournament.id as i64)
            .bind(m.home_team_id as i64)
            .execute(&pool)
            .await
            .ok();

            sqlx::query(
                r#"
                UPDATE league_standings SET
                    matches_played = matches_played + 1,
                    wins = wins + ?,
                    losses = losses + ?,
                    points = points + ?,
                    games_won = games_won + ?,
                    games_lost = games_lost + ?,
                    game_diff = game_diff + ?
                WHERE tournament_id = ? AND team_id = ?
                "#
            )
            .bind(if home_won { 0 } else { 1 })
            .bind(if home_won { 1 } else { 0 })
            .bind(if home_won { 0 } else { 3 })
            .bind(away_score as i64)
            .bind(home_score as i64)
            .bind((away_score as i64) - (home_score as i64))
            .bind(tournament.id as i64)
            .bind(m.away_team_id as i64)
            .execute(&pool)
            .await
            .ok();
        }

        updated_count += 1;
    }

    Ok(DevCommandResult::ok(
        updated_count,
        format!("成功重新计算 {} 个赛事的积分榜", updated_count),
    ))
}

#[tauri::command]
pub async fn dev_recalculate_market_values(
    state: State<'_, AppState>,
) -> Result<DevCommandResult<u32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let rows = sqlx::query(
        r#"SELECT p.id, p.game_id, p.ability, p.potential, p.age, p.tag, p.position,
                  p.market_value, p.calculated_market_value,
                  r.short_name as region_code
           FROM players p
           LEFT JOIN teams t ON p.team_id = t.id
           LEFT JOIN regions r ON t.region_id = r.id
           WHERE p.save_id = ? AND p.status = 'Active'"#
    )
    .bind(&save_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch players: {}", e))?;

    let mut updated_count = 0u32;

    for row in rows {
        let id: i64 = row.get("id");
        let game_id: String = row.get("game_id");
        let ability: i64 = row.get("ability");
        let potential: i64 = row.get("potential");
        let age: i64 = row.get("age");
        let tag_str: String = row.get("tag");
        let position_str: String = row.get("position");
        let old_calculated: i64 = row.try_get("calculated_market_value").unwrap_or(0);
        let region_code: String = row.try_get::<Option<String>, _>("region_code")
            .ok()
            .flatten()
            .unwrap_or_else(|| "LPL".to_string());

        let tag = match tag_str.to_uppercase().as_str() {
            "GENIUS" => PlayerTag::Genius,
            "NORMAL" => PlayerTag::Normal,
            _ => PlayerTag::Ordinary,
        };

        let position = match position_str.to_uppercase().as_str() {
            "TOP" => Position::Top,
            "JUG" | "JUNGLE" => Position::Jug,
            "MID" => Position::Mid,
            "ADC" | "BOT" => Position::Adc,
            "SUP" | "SUPPORT" => Position::Sup,
            _ => Position::Mid,
        };

        let base_value = MarketValueEngine::calculate_base_market_value_enum(ability as u8, age as u8, potential as u8, &tag, &position);

        let honor_factor = {
            let honor_rows = sqlx::query(
                "SELECT honor_type, tournament_type, tournament_name, season_id FROM honors WHERE save_id = ? AND player_id = ?"
            )
            .bind(&save_id).bind(id)
            .fetch_all(&pool).await.unwrap_or_default();

            let honors: Vec<crate::engines::market_value::PlayerHonorRecord> = honor_rows.iter().filter_map(|row| {
                let ht: String = row.get("honor_type");
                let tt: String = row.try_get::<Option<String>, _>("tournament_type").ok().flatten().unwrap_or_default();
                let tn: String = row.try_get::<Option<String>, _>("tournament_name").ok().flatten().unwrap_or_default();
                let season: u32 = row.get::<i64, _>("season_id") as u32;
                MarketValueEngine::parse_honor_category(&ht, &tt, &tn)
                    .map(|cat| crate::engines::market_value::PlayerHonorRecord::new(cat, season, &tn))
            }).collect();

            let current_season = sqlx::query("SELECT current_season_id FROM saves WHERE id = ?")
                .bind(&save_id)
                .fetch_optional(&pool).await.ok().flatten()
                .map(|r| r.get::<i64, _>("current_season_id") as u32)
                .unwrap_or(1);

            MarketValueEngine::calculate_honor_factor(&honors, current_season)
        };

        let region_factor = MarketValueEngine::region_factor(&region_code);

        let new_value = ((base_value as f64) * honor_factor * region_factor) as i64;

        if new_value != old_calculated {
            sqlx::query("UPDATE players SET calculated_market_value = ? WHERE id = ?")
                .bind(new_value)
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| format!("Failed to update player {}: {}", game_id, e))?;

            log::debug!("{} 身价 {} -> {} 万 (荣誉×{:.2}, 赛区×{:.2})",
                game_id, old_calculated, new_value, honor_factor, region_factor);

            updated_count += 1;
        }
    }

    Ok(DevCommandResult::ok(
        updated_count,
        format!("成功更新 {} 名选手的计算身价", updated_count)
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_redistribute_prizes(
    state: State<'_, AppState>,
    season_id: Option<i64>,
) -> Result<DevCommandResult<i32>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let season = match season_id {
        Some(s) => s,
        None => {
            let save = SaveRepository::get_by_id(&pool, &save_id)
                .await
                .map_err(|e| e.to_string())?;
            save.current_season as i64
        }
    };

    let deleted: u64 = sqlx::query(
        "DELETE FROM financial_transactions WHERE save_id = ? AND season_id = ? AND (transaction_type = 'PlayoffBonus' OR transaction_type = 'InternationalBonus')"
    )
    .bind(&save_id)
    .bind(season)
    .execute(&pool)
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);

    let completed_tournaments: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tournaments WHERE save_id = ? AND season_id = ? AND status = 'Completed'"
    )
    .bind(&save_id)
    .bind(season)
    .fetch_one(&pool)
    .await
    .unwrap_or((0,));

    Ok(DevCommandResult::ok(
        completed_tournaments.0 as i32,
        format!("清除 {} 条旧记录，请通过前端重新完成赛事以发放奖金（共 {} 个已完成赛事）", deleted, completed_tournaments.0),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn dev_rebuild_player_season_stats(
    state: State<'_, AppState>,
    season_id: i64,
) -> Result<DevCommandResult<RebuildStatsResult>, String> {
    let guard = state.db.read().await;
    let db = match guard.as_ref() {
        Some(db) => db,
        None => return Ok(DevCommandResult::err("Database not initialized")),
    };

    let current_save = state.current_save_id.read().await;
    let save_id = match current_save.as_ref() {
        Some(id) => id.clone(),
        None => return Ok(DevCommandResult::err("No save loaded")),
    };

    let pool = match db.get_pool().await {
        Ok(p) => p,
        Err(e) => return Ok(DevCommandResult::err(format!("Failed to get pool: {}", e))),
    };

    let aggregated_stats: Vec<(i64, String, i64, String, String, i64, f64, f64, f64, f64)> = sqlx::query_as(
        r#"
        SELECT
            gpp.player_id,
            gpp.player_name,
            gpp.team_id,
            COALESCE(r.short_name, 'LPL') as region_code,
            gpp.position,
            COUNT(DISTINCT gpp.game_id) as games_played,
            SUM(gpp.impact_score) as total_impact,
            AVG(gpp.impact_score) as avg_impact,
            AVG(gpp.actual_ability) as avg_performance,
            MAX(gpp.actual_ability) as best_performance
        FROM game_player_performances gpp
        JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
        JOIN matches m ON mg.match_id = m.id
        JOIN tournaments t ON m.tournament_id = t.id
        LEFT JOIN teams tm ON gpp.team_id = tm.id
        LEFT JOIN regions r ON tm.region_id = r.id
        WHERE gpp.save_id = ? AND t.season_id = ?
        GROUP BY gpp.player_id, gpp.player_name, gpp.team_id, gpp.position
        "#
    )
    .bind(&save_id)
    .bind(season_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to aggregate stats: {}", e))?;

    if aggregated_stats.is_empty() {
        return Ok(DevCommandResult::ok(
            RebuildStatsResult {
                created_count: 0,
                updated_count: 0,
                players: vec![],
            },
            format!("S{} 没有找到比赛数据，无法重建统计", season_id),
        ));
    }

    let mut created_count = 0;
    let mut updated_count = 0;
    let mut players = Vec::new();

    for (player_id, player_name, team_id, region_code, position, games_played, total_impact, avg_impact, avg_performance, best_performance) in aggregated_stats {
        let extra_stats: Option<(f64, f64)> = sqlx::query_as(
            r#"
            SELECT
                MIN(gpp.actual_ability) as worst_performance,
                CASE
                    WHEN COUNT(*) > 1 THEN 100.0 - (
                        SQRT(SUM((gpp.actual_ability - sub.avg_perf) * (gpp.actual_ability - sub.avg_perf)) / (COUNT(*) - 1))
                    )
                    ELSE 100.0
                END as consistency_score
            FROM game_player_performances gpp
            JOIN match_games mg ON gpp.game_id = mg.id AND gpp.save_id = mg.save_id
            JOIN matches m ON mg.match_id = m.id
            JOIN tournaments t ON m.tournament_id = t.id
            CROSS JOIN (SELECT AVG(gpp2.actual_ability) as avg_perf
                        FROM game_player_performances gpp2
                        JOIN match_games mg2 ON gpp2.game_id = mg2.id AND gpp2.save_id = mg2.save_id
                        JOIN matches m2 ON mg2.match_id = m2.id
                        JOIN tournaments t2 ON m2.tournament_id = t2.id
                        WHERE gpp2.save_id = ? AND t2.season_id = ? AND gpp2.player_id = ?) sub
            WHERE gpp.save_id = ? AND t.season_id = ? AND gpp.player_id = ?
            "#
        )
        .bind(&save_id)
        .bind(season_id)
        .bind(player_id)
        .bind(&save_id)
        .bind(season_id)
        .bind(player_id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

        let (worst_performance, consistency_score) = extra_stats.unwrap_or((100.0, 100.0));
        let consistency_score = consistency_score.max(0.0).min(100.0);

        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM player_season_stats WHERE save_id = ? AND player_id = ? AND season_id = ?"
        )
        .bind(&save_id)
        .bind(player_id)
        .bind(season_id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

        let titles: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN tournament_type IN ('Worlds', 'MSI', 'Super', 'ICP', 'Claude', 'ShanghaiMasters', 'MadridMasters') AND honor_type = 'PLAYER_CHAMPION' THEN 1 ELSE 0 END), 0) as international,
                COALESCE(SUM(CASE WHEN tournament_type LIKE '%Playoff%' AND honor_type = 'PLAYER_CHAMPION' THEN 1 ELSE 0 END), 0) as regional
            FROM honors
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(&save_id)
        .bind(player_id)
        .bind(season_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0, 0));

        let placement_bonus: (f64, f64, f64, f64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN tournament_type IN ('Worlds', 'MSI', 'Super', 'ICP', 'Claude', 'ShanghaiMasters', 'MadridMasters') AND honor_type = 'PLAYER_RUNNER_UP' THEN 2.0 ELSE 0.0 END), 0.0),
                COALESCE(SUM(CASE WHEN tournament_type LIKE '%Playoff%' AND honor_type = 'PLAYER_RUNNER_UP' THEN 0.5 ELSE 0.0 END), 0.0),
                COALESCE(SUM(CASE WHEN tournament_type IN ('Worlds', 'MSI', 'Super', 'ICP', 'Claude', 'ShanghaiMasters', 'MadridMasters') AND honor_type = 'PLAYER_THIRD' THEN 1.0 ELSE 0.0 END), 0.0),
                COALESCE(SUM(CASE WHEN tournament_type LIKE '%Playoff%' AND honor_type = 'PLAYER_THIRD' THEN 0.25 ELSE 0.0 END), 0.0)
            FROM honors
            WHERE save_id = ? AND player_id = ? AND season_id = ?
            "#
        )
        .bind(&save_id)
        .bind(player_id)
        .bind(season_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((0.0, 0.0, 0.0, 0.0));

        let champion_bonus = (titles.0 * 3 + titles.1) as f64
            + placement_bonus.0 + placement_bonus.1 + placement_bonus.2 + placement_bonus.3;
        let yearly_top_score = PlayerSeasonStatistics::calculate_yearly_top_score(
            avg_impact,
            avg_performance,
            consistency_score,
            games_played as i32,
            champion_bonus,
        );
        let dominance_score = PlayerSeasonStatistics::calculate_dominance_score(
            best_performance,
            avg_impact,
            avg_performance,
        );

        if existing.is_some() {
            sqlx::query(
                r#"
                UPDATE player_season_stats SET
                    team_id = ?, region_id = ?, games_played = ?,
                    total_impact = ?, avg_impact = ?, avg_performance = ?,
                    best_performance = ?, worst_performance = ?, consistency_score = ?,
                    international_titles = ?, regional_titles = ?,
                    champion_bonus = ?, yearly_top_score = ?, dominance_score = ?,
                    updated_at = datetime('now')
                WHERE save_id = ? AND player_id = ? AND season_id = ?
                "#
            )
            .bind(team_id)
            .bind(&region_code)
            .bind(games_played)
            .bind(total_impact)
            .bind(avg_impact)
            .bind(avg_performance)
            .bind(best_performance)
            .bind(worst_performance)
            .bind(consistency_score)
            .bind(titles.0)
            .bind(titles.1)
            .bind(champion_bonus)
            .bind(yearly_top_score)
            .bind(dominance_score)
            .bind(&save_id)
            .bind(player_id)
            .bind(season_id)
            .execute(&pool)
            .await
            .ok();

            updated_count += 1;
        } else {
            sqlx::query(
                r#"
                INSERT INTO player_season_stats
                (save_id, player_id, player_name, season_id, team_id, region_id, position,
                 matches_played, games_played, total_impact, avg_impact, avg_performance,
                 best_performance, worst_performance, consistency_score,
                 international_titles, regional_titles, champion_bonus, yearly_top_score, dominance_score)
                VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&save_id)
            .bind(player_id)
            .bind(&player_name)
            .bind(season_id)
            .bind(team_id)
            .bind(&region_code)
            .bind(&position)
            .bind(games_played)
            .bind(total_impact)
            .bind(avg_impact)
            .bind(avg_performance)
            .bind(best_performance)
            .bind(worst_performance)
            .bind(consistency_score)
            .bind(titles.0)
            .bind(titles.1)
            .bind(champion_bonus)
            .bind(yearly_top_score)
            .bind(dominance_score)
            .execute(&pool)
            .await
            .ok();

            created_count += 1;
        }

        players.push(format!("{} (场次:{}, 得分:{:.1})", player_name, games_played, yearly_top_score));
    }

    Ok(DevCommandResult::ok(
        RebuildStatsResult {
            created_count,
            updated_count,
            players,
        },
        format!("S{} 统计数据重建完成：新建 {} 条，更新 {} 条", season_id, created_count, updated_count),
    ))
}
