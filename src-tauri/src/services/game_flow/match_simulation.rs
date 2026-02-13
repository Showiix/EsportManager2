use std::collections::HashMap;

use crate::db::*;
use crate::engines::{
    ConditionEngine, MatchPlayerInfo, MatchSimContext, MatchSimulationEngine, MetaEngine,
    PlayerFormFactors, TraitType,
};
use crate::models::*;
use sqlx::{Pool, Row, Sqlite};

use super::GameFlowService;

impl GameFlowService {
    /// 模拟当前阶段的所有比赛
    pub(crate) async fn simulate_all_phase_matches(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        phase: SeasonPhase,
    ) -> Result<u32, String> {
        let tournament_type = phase.to_tournament_type();

        if tournament_type.is_none() {
            return Ok(0);
        }

        let t_type = tournament_type.unwrap();
        let save = SaveRepository::get_by_id(pool, save_id)
            .await
            .map_err(|e| e.to_string())?;

        let tournaments = TournamentRepository::get_by_season_and_type(
            pool, save_id, save.current_season as u64, &format!("{:?}", t_type)
        ).await.map_err(|e| e.to_string())?;

        // 检查是否是季后赛阶段
        let is_playoff = matches!(phase, SeasonPhase::SpringPlayoffs | SeasonPhase::SummerPlayoffs);

        // === 特性系统：预加载选手数据 + form factors ===
        let (mut team_players, mut form_factors_map) = self.load_team_players(pool, save_id, save.current_season as i64).await?;
        let meta_weights = MetaEngine::get_current_weights(pool, save_id, save.current_season as i64)
            .await
            .unwrap_or_else(|_| crate::engines::MetaWeights::balanced());

        // 构建比赛情境
        let is_international = matches!(
            phase,
            SeasonPhase::Msi | SeasonPhase::MadridMasters |
            SeasonPhase::ClaudeIntercontinental | SeasonPhase::WorldChampionship |
            SeasonPhase::ShanghaiMasters | SeasonPhase::IcpIntercontinental |
            SeasonPhase::SuperIntercontinental
        );
        let tournament_type_str = format!("{:?}", t_type).to_lowercase();
        let sim_ctx = MatchSimContext {
            is_playoff,
            is_international,
            tournament_type: tournament_type_str,
        };
        let match_engine = MatchSimulationEngine::default();

        if is_playoff {
            // 季后赛：逐场模拟以确保正确生成后续对阵
            let mut simulated_count = 0u32;

            loop {
                let mut found_pending = false;

                for tournament in &tournaments {
                    let pending = MatchRepository::get_pending(pool, save_id, tournament.id)
                        .await
                        .map_err(|e| e.to_string())?;

                    if pending.is_empty() {
                        continue;
                    }

                    found_pending = true;
                    let match_info = &pending[0];

                    // 特性感知模拟
                    let home_players = team_players.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let away_players = team_players.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);

                    let result = if !home_players.is_empty() && !away_players.is_empty() {
                        match_engine.simulate_match_with_traits(
                            match_info.id, match_info.tournament_id, &match_info.stage,
                            match_info.format.clone(), match_info.home_team_id, match_info.away_team_id,
                            home_players, away_players, &sim_ctx, &meta_weights,
                        )
                    } else {
                        let home_team = TeamRepository::get_by_id(pool, match_info.home_team_id)
                            .await.map_err(|e| e.to_string())?;
                        let away_team = TeamRepository::get_by_id(pool, match_info.away_team_id)
                            .await.map_err(|e| e.to_string())?;
                        self.league_service.simulate_match(
                            match_info, home_team.power_rating, away_team.power_rating,
                        )
                    };

                    // 更新比赛结果
                    MatchRepository::update_result(
                        pool,
                        match_info.id,
                        result.home_score as u32,
                        result.away_score as u32,
                        result.winner_id,
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                    // 比赛后更新 form factors
                    let home_won = result.winner_id == match_info.home_team_id;
                    let home_avg = Self::calculate_avg_performance(&result, match_info.home_team_id);
                    let away_avg = Self::calculate_avg_performance(&result, match_info.away_team_id);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.home_team_id, home_won, home_avg);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.away_team_id, !home_won, away_avg);

                    simulated_count += 1;

                    // 检查并生成下一轮对阵
                    let all_matches = MatchRepository::get_by_tournament(pool, tournament.id)
                        .await
                        .map_err(|e| e.to_string())?;

                    let new_matches =
                        self.league_service.advance_playoff_bracket(tournament.id, &all_matches);

                    if !new_matches.is_empty() {
                        log::debug!(
                            "[simulate_all_phase_matches] 季后赛生成 {} 场新比赛",
                            new_matches.len()
                        );
                        MatchRepository::create_batch(pool, save_id, &new_matches)
                            .await
                            .map_err(|e| e.to_string())?;
                    }

                    break; // 每次只模拟一场，然后重新检查
                }

                if !found_pending {
                    break;
                }
            }

            // 阶段结束，批量写回 form factors
            Self::flush_form_factors_to_db(pool, save_id, &form_factors_map).await?;

            Ok(simulated_count)
        } else {
            // 非季后赛：批量模拟
            let mut simulated_count = 0u32;

            for tournament in &tournaments {
                let pending = MatchRepository::get_pending(pool, save_id, tournament.id)
                    .await
                    .map_err(|e| e.to_string())?;

                for match_info in &pending {
                    // 特性感知模拟
                    let home_players = team_players.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let away_players = team_players.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);

                    let result = if !home_players.is_empty() && !away_players.is_empty() {
                        match_engine.simulate_match_with_traits(
                            match_info.id, match_info.tournament_id, &match_info.stage,
                            match_info.format.clone(), match_info.home_team_id, match_info.away_team_id,
                            home_players, away_players, &sim_ctx, &meta_weights,
                        )
                    } else {
                        let home_team = TeamRepository::get_by_id(pool, match_info.home_team_id)
                            .await.map_err(|e| e.to_string())?;
                        let away_team = TeamRepository::get_by_id(pool, match_info.away_team_id)
                            .await.map_err(|e| e.to_string())?;
                        self.league_service.simulate_match(
                            match_info, home_team.power_rating, away_team.power_rating,
                        )
                    };

                    MatchRepository::update_result(
                        pool,
                        match_info.id,
                        result.home_score as u32,
                        result.away_score as u32,
                        result.winner_id,
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                    // 比赛后更新 form factors
                    let home_won = result.winner_id == match_info.home_team_id;
                    let home_avg = Self::calculate_avg_performance(&result, match_info.home_team_id);
                    let away_avg = Self::calculate_avg_performance(&result, match_info.away_team_id);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.home_team_id, home_won, home_avg);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.away_team_id, !home_won, away_avg);

                    simulated_count += 1;
                }
            }

            // 阶段结束，批量写回 form factors
            Self::flush_form_factors_to_db(pool, save_id, &form_factors_map).await?;

            Ok(simulated_count)
        }
    }

    /// 预加载所有队伍的首发选手数据（含特性+动态condition），用于特性感知模拟
    pub async fn load_team_players(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: i64,
    ) -> Result<(HashMap<u64, Vec<MatchPlayerInfo>>, HashMap<u64, PlayerFormFactors>), String> {
        // 查询所有在役首发选手，LEFT JOIN form factors
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.ability, p.stability, p.age, p.position, p.team_id, p.join_season,
                   COALESCE(pff.form_cycle, 50.0) as form_cycle,
                   COALESCE(pff.momentum, 0) as momentum,
                   COALESCE(pff.last_performance, 0.0) as last_performance,
                   COALESCE(pff.last_match_won, 0) as last_match_won,
                   COALESCE(pff.games_since_rest, 0) as games_since_rest
            FROM players p
            LEFT JOIN player_form_factors pff ON p.id = pff.player_id AND pff.save_id = ?
            WHERE p.save_id = ? AND p.status = 'Active' AND p.is_starter = 1
            "#,
        )
        .bind(save_id)
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("加载选手数据失败: {}", e))?;

        // 收集所有 player_id
        let player_ids: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>("id")).collect();

        // 批量查询特性
        let mut player_traits_map: HashMap<u64, Vec<TraitType>> = HashMap::new();
        if !player_ids.is_empty() {
            let placeholders: String = player_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!(
                "SELECT player_id, trait_type FROM player_traits WHERE save_id = ? AND player_id IN ({})",
                placeholders
            );
            let mut query = sqlx::query(&query_str).bind(save_id);
            for pid in &player_ids {
                query = query.bind(pid);
            }
            let trait_rows = query.fetch_all(pool).await.map_err(|e| format!("加载特性失败: {}", e))?;

            for row in &trait_rows {
                let pid = row.get::<i64, _>("player_id") as u64;
                let trait_str: String = row.get("trait_type");
                if let Some(tt) = TraitType::from_str(&trait_str) {
                    player_traits_map.entry(pid).or_default().push(tt);
                }
            }
        }

        // 按 team_id 分组，同时构建 form_factors_map
        let mut team_players: HashMap<u64, Vec<MatchPlayerInfo>> = HashMap::new();
        let mut form_factors_map: HashMap<u64, PlayerFormFactors> = HashMap::new();
        let mut team_join_seasons: HashMap<u64, i64> = HashMap::new();
        for row in &rows {
            let player_id = row.get::<i64, _>("id") as u64;
            let team_id = row.get::<i64, _>("team_id") as u64;
            let join_season: i64 = row.get("join_season");
            let ability = row.get::<i64, _>("ability") as u8;
            let age = row.get::<i64, _>("age") as u8;
            team_join_seasons.insert(player_id, join_season);

            // 构建 form factors
            let factors = PlayerFormFactors {
                player_id,
                form_cycle: row.get::<f64, _>("form_cycle"),
                momentum: row.get::<i64, _>("momentum") as i8,
                last_performance: row.get::<f64, _>("last_performance"),
                last_match_won: row.get::<i64, _>("last_match_won") != 0,
                games_since_rest: row.get::<i64, _>("games_since_rest") as u32,
            };

            // 动态计算 condition
            let condition = ConditionEngine::calculate_condition(age, ability, &factors, None);

            form_factors_map.insert(player_id, factors);

            let player_info = MatchPlayerInfo {
                player_id,
                ability,
                stability: row.get::<i64, _>("stability") as u8,
                condition,
                age,
                position: row.get::<String, _>("position"),
                traits: player_traits_map.get(&player_id).cloned().unwrap_or_default(),
                is_first_season: join_season == current_season,
            };

            team_players.entry(team_id).or_default().push(player_info);
        }

        // 化学反应/协同加成：同队时间越长 condition 越高
        for (_team_id, players) in team_players.iter_mut() {
            if players.is_empty() {
                continue;
            }
            let avg_tenure: f64 = players
                .iter()
                .map(|p| {
                    let tenure = (current_season - *team_join_seasons.get(&p.player_id).unwrap_or(&current_season)).max(0) + 1;
                    tenure as f64
                })
                .sum::<f64>()
                / players.len() as f64;

            let synergy_bonus = (avg_tenure * 0.4).min(2.0) as i8;
            if synergy_bonus > 0 {
                for player in players.iter_mut() {
                    let age = player.age;
                    let (min, max) = ConditionEngine::get_condition_range_by_age(age);
                    player.condition = (player.condition + synergy_bonus).clamp(min, max);
                }
            }
        }

        Ok((team_players, form_factors_map))
    }

    /// 比赛后更新内存中的 form factors 并重算 condition
    pub(crate) fn update_form_factors_after_match(
        team_players: &mut HashMap<u64, Vec<MatchPlayerInfo>>,
        form_factors_map: &mut HashMap<u64, PlayerFormFactors>,
        team_id: u64,
        won: bool,
        avg_performance: f64,
    ) {
        if let Some(players) = team_players.get_mut(&team_id) {
            for player in players.iter_mut() {
                if let Some(factors) = form_factors_map.remove(&player.player_id) {
                    let updated = ConditionEngine::update_form_factors(factors, won, avg_performance);
                    let new_condition = ConditionEngine::calculate_condition(
                        player.age, player.ability, &updated, None,
                    );
                    player.condition = new_condition;
                    form_factors_map.insert(player.player_id, updated);
                }
            }
        }
    }

    /// 批量将 form factors 写回数据库
    pub(crate) async fn flush_form_factors_to_db(
        pool: &Pool<Sqlite>,
        save_id: &str,
        form_factors_map: &HashMap<u64, PlayerFormFactors>,
    ) -> Result<(), String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        for (player_id, factors) in form_factors_map {
            sqlx::query(
                r#"
                INSERT INTO player_form_factors (save_id, player_id, form_cycle, momentum, last_performance, last_match_won, games_since_rest, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
                ON CONFLICT(save_id, player_id) DO UPDATE SET
                    form_cycle = excluded.form_cycle,
                    momentum = excluded.momentum,
                    last_performance = excluded.last_performance,
                    last_match_won = excluded.last_match_won,
                    games_since_rest = excluded.games_since_rest,
                    updated_at = datetime('now')
                "#,
            )
            .bind(save_id)
            .bind(*player_id as i64)
            .bind(factors.form_cycle)
            .bind(factors.momentum as i64)
            .bind(factors.last_performance)
            .bind(if factors.last_match_won { 1i64 } else { 0i64 })
            .bind(factors.games_since_rest as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 从 MatchResult 中计算某队的平均 performance
    pub(crate) fn calculate_avg_performance(result: &crate::models::MatchResult, team_id: u64) -> f64 {
        if result.games.is_empty() {
            return 0.0;
        }
        let total: f64 = result.games.iter().map(|g| {
            if g.winner_id == team_id {
                // 该队赢的局，取对应方的 performance
                if team_id == result.match_info.home_team_id {
                    g.home_performance
                } else {
                    g.away_performance
                }
            } else {
                if team_id == result.match_info.home_team_id {
                    g.home_performance
                } else {
                    g.away_performance
                }
            }
        }).sum();
        total / result.games.len() as f64
    }
}
