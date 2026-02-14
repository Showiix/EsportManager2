use std::collections::HashMap;

use crate::db::*;
use crate::engines::bp_engine::{BpEngine, DraftResult, PlayerChampionPool};
use crate::engines::champion::{self, MasteryTier, VersionTier};
use crate::engines::meta_engine::MetaType;
use crate::engines::{
    ConditionEngine, MatchPlayerInfo, MatchSimContext, MatchSimulationEngine, MetaEngine,
    PlayerFormFactors, TraitType,
};
use crate::models::*;
use crate::models::transfer::AITeamPersonality;
use rand::rngs::StdRng;
use rand::SeedableRng;
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
        let (mut team_players, team_bench, mut form_factors_map, team_personalities, player_mastery_map) = self.load_team_players(pool, save_id, save.current_season as i64).await?;
        let meta_weights = MetaEngine::get_current_weights(pool, save_id, save.current_season as i64)
            .await
            .unwrap_or_else(|_| crate::engines::MetaWeights::balanced());
        let meta_type_row: Option<String> = sqlx::query_scalar(
            "SELECT meta_type FROM meta_versions WHERE save_id = ? AND season_id = ? LIMIT 1"
        )
        .bind(save_id)
        .bind(save.current_season as i64)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("加载Meta类型失败: {}", e))?;
        let meta_type = meta_type_row
            .and_then(|s| MetaType::from_id(&s))
            .unwrap_or(MetaType::Balanced);
        let version_tiers: HashMap<u8, VersionTier> = champion::calculate_version_tiers(meta_type)
            .into_iter()
            .collect();
        let mut bp_rng = StdRng::from_entropy();

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

                    let home_bench_players = team_bench.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let away_bench_players = team_bench.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let home_pers = team_personalities.get(&match_info.home_team_id).copied().unwrap_or(AITeamPersonality::Balanced);
                    let away_pers = team_personalities.get(&match_info.away_team_id).copied().unwrap_or(AITeamPersonality::Balanced);

                    let home_has_players = team_players
                        .get(&match_info.home_team_id)
                        .map(|players| !players.is_empty())
                        .unwrap_or(false);
                    let away_has_players = team_players
                        .get(&match_info.away_team_id)
                        .map(|players| !players.is_empty())
                        .unwrap_or(false);

                    let result = if home_has_players && away_has_players {
                        let (home_pools, away_pools) = {
                            let home_players = team_players
                                .get(&match_info.home_team_id)
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);
                            let away_players = team_players
                                .get(&match_info.away_team_id)
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);
                            (
                                Self::build_champion_pools(home_players, &player_mastery_map),
                                Self::build_champion_pools(away_players, &player_mastery_map),
                            )
                        };
                        let draft = BpEngine::run_draft(
                            &home_pools,
                            &away_pools,
                            &version_tiers,
                            meta_type,
                            &mut bp_rng,
                        );

                        Self::save_draft_result(pool, save_id, match_info.id, &draft).await?;

                        if let Some(players) = team_players.get_mut(&match_info.home_team_id) {
                            for p in players.iter_mut() {
                                p.bp_modifier = draft
                                    .home_bp_modifiers
                                    .get(&p.player_id)
                                    .copied()
                                    .unwrap_or(0.0);
                                p.champion_version_score = Self::calculate_champion_version_score(
                                    p.player_id, &player_mastery_map, &version_tiers,
                                );
                            }
                        }
                        if let Some(players) = team_players.get_mut(&match_info.away_team_id) {
                            for p in players.iter_mut() {
                                p.bp_modifier = draft
                                    .away_bp_modifiers
                                    .get(&p.player_id)
                                    .copied()
                                    .unwrap_or(0.0);
                                p.champion_version_score = Self::calculate_champion_version_score(
                                    p.player_id, &player_mastery_map, &version_tiers,
                                );
                            }
                        }

                        let home_players = team_players.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                        let away_players = team_players.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);

                        match_engine.simulate_match_with_traits(
                            match_info.id, match_info.tournament_id, &match_info.stage,
                            match_info.format.clone(), match_info.home_team_id, match_info.away_team_id,
                            home_players, away_players,
                            home_bench_players, away_bench_players,
                            &sim_ctx, &meta_weights,
                            &home_pers, &away_pers,
                            save.current_season as u32,
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

                    let played = match_engine.take_last_games_played();
                    Self::update_season_games_played(pool, save_id, played).await?;
                    let total_games_this_match = (result.home_score + result.away_score) as i64;
                    Self::update_season_games_total(pool, save_id, match_info.home_team_id, match_info.away_team_id, total_games_this_match).await?;

                    // 比赛后更新 form factors
                    let home_won = result.winner_id == match_info.home_team_id;
                    let home_avg = Self::calculate_avg_performance(&result, match_info.home_team_id);
                    let away_avg = Self::calculate_avg_performance(&result, match_info.away_team_id);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.home_team_id, home_won, home_avg);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.away_team_id, !home_won, away_avg);
                    Self::update_form_factors_bench_after_match(&team_bench, &mut form_factors_map, match_info.home_team_id);
                    Self::update_form_factors_bench_after_match(&team_bench, &mut form_factors_map, match_info.away_team_id);

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
                    let home_bench_players = team_bench.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let away_bench_players = team_bench.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    let home_pers = team_personalities.get(&match_info.home_team_id).copied().unwrap_or(AITeamPersonality::Balanced);
                    let away_pers = team_personalities.get(&match_info.away_team_id).copied().unwrap_or(AITeamPersonality::Balanced);

                    let home_has_players = team_players
                        .get(&match_info.home_team_id)
                        .map(|players| !players.is_empty())
                        .unwrap_or(false);
                    let away_has_players = team_players
                        .get(&match_info.away_team_id)
                        .map(|players| !players.is_empty())
                        .unwrap_or(false);

                    let result = if home_has_players && away_has_players {
                        let (home_pools, away_pools) = {
                            let home_players = team_players
                                .get(&match_info.home_team_id)
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);
                            let away_players = team_players
                                .get(&match_info.away_team_id)
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);
                            (
                                Self::build_champion_pools(home_players, &player_mastery_map),
                                Self::build_champion_pools(away_players, &player_mastery_map),
                            )
                        };
                        let draft = BpEngine::run_draft(
                            &home_pools,
                            &away_pools,
                            &version_tiers,
                            meta_type,
                            &mut bp_rng,
                        );

                        Self::save_draft_result(pool, save_id, match_info.id, &draft).await?;

                        if let Some(players) = team_players.get_mut(&match_info.home_team_id) {
                            for p in players.iter_mut() {
                                p.bp_modifier = draft
                                    .home_bp_modifiers
                                    .get(&p.player_id)
                                    .copied()
                                    .unwrap_or(0.0);
                                p.champion_version_score = Self::calculate_champion_version_score(
                                    p.player_id, &player_mastery_map, &version_tiers,
                                );
                            }
                        }
                        if let Some(players) = team_players.get_mut(&match_info.away_team_id) {
                            for p in players.iter_mut() {
                                p.bp_modifier = draft
                                    .away_bp_modifiers
                                    .get(&p.player_id)
                                    .copied()
                                    .unwrap_or(0.0);
                                p.champion_version_score = Self::calculate_champion_version_score(
                                    p.player_id, &player_mastery_map, &version_tiers,
                                );
                            }
                        }

                        let home_players = team_players.get(&match_info.home_team_id).map(|v| v.as_slice()).unwrap_or(&[]);
                        let away_players = team_players.get(&match_info.away_team_id).map(|v| v.as_slice()).unwrap_or(&[]);

                        match_engine.simulate_match_with_traits(
                            match_info.id, match_info.tournament_id, &match_info.stage,
                            match_info.format.clone(), match_info.home_team_id, match_info.away_team_id,
                            home_players, away_players,
                            home_bench_players, away_bench_players,
                            &sim_ctx, &meta_weights,
                            &home_pers, &away_pers,
                            save.current_season as u32,
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

                    let played = match_engine.take_last_games_played();
                    Self::update_season_games_played(pool, save_id, played).await?;
                    let total_games_this_match = (result.home_score + result.away_score) as i64;
                    Self::update_season_games_total(pool, save_id, match_info.home_team_id, match_info.away_team_id, total_games_this_match).await?;

                    // 比赛后更新 form factors
                    let home_won = result.winner_id == match_info.home_team_id;
                    let home_avg = Self::calculate_avg_performance(&result, match_info.home_team_id);
                    let away_avg = Self::calculate_avg_performance(&result, match_info.away_team_id);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.home_team_id, home_won, home_avg);
                    Self::update_form_factors_after_match(&mut team_players, &mut form_factors_map, match_info.away_team_id, !home_won, away_avg);
                    Self::update_form_factors_bench_after_match(&team_bench, &mut form_factors_map, match_info.home_team_id);
                    Self::update_form_factors_bench_after_match(&team_bench, &mut form_factors_map, match_info.away_team_id);

                    simulated_count += 1;
                }
            }

            // 阶段结束，批量写回 form factors
            Self::flush_form_factors_to_db(pool, save_id, &form_factors_map).await?;

            Ok(simulated_count)
        }
    }

    fn build_champion_pools(
        players: &[MatchPlayerInfo],
        player_mastery_map: &HashMap<u64, HashMap<u8, MasteryTier>>,
    ) -> Vec<PlayerChampionPool> {
        players
            .iter()
            .map(|p| {
                let position = match p.position.to_uppercase().as_str() {
                    "TOP" => crate::models::player::Position::Top,
                    "JUG" | "JUNGLE" => crate::models::player::Position::Jug,
                    "MID" => crate::models::player::Position::Mid,
                    "ADC" | "BOT" => crate::models::player::Position::Adc,
                    "SUP" | "SUPPORT" => crate::models::player::Position::Sup,
                    _ => crate::models::player::Position::Mid,
                };
                PlayerChampionPool {
                    player_id: p.player_id,
                    position,
                    ability: p.ability,
                    masteries: player_mastery_map
                        .get(&p.player_id)
                        .cloned()
                        .unwrap_or_default(),
                    traits: p.traits.clone(),
                }
            })
            .collect()
    }

    /// 计算选手的版本适配分：SS/S 英雄中最佳版本 Tier 的得分
    /// SS+T1=+3, S+T1=+2, SS+T2=+1, S+T2=0, T3=-2 (SS 免疫 T3 → 0)
    fn calculate_champion_version_score(
        player_id: u64,
        player_mastery_map: &HashMap<u64, HashMap<u8, MasteryTier>>,
        version_tiers: &HashMap<u8, VersionTier>,
    ) -> f64 {
        let Some(masteries) = player_mastery_map.get(&player_id) else {
            return 0.0;
        };

        let mut best_score = 0.0_f64;
        for (&champion_id, &tier) in masteries {
            if tier != MasteryTier::SS && tier != MasteryTier::S {
                continue;
            }
            let vt = version_tiers.get(&champion_id).copied().unwrap_or(VersionTier::T2);
            let score = match (tier, vt) {
                (MasteryTier::SS, VersionTier::T1) => 3.0,
                (MasteryTier::S, VersionTier::T1) => 2.0,
                (MasteryTier::SS, VersionTier::T2) => 1.0,
                (MasteryTier::S, VersionTier::T2) => 0.0,
                (MasteryTier::SS, VersionTier::T3) => 0.0,
                (MasteryTier::S, VersionTier::T3) => -2.0,
                _ => 0.0,
            };
            best_score = best_score.max(score);
        }
        best_score
    }

    /// 预加载所有队伍的首发选手数据（含特性+动态condition），用于特性感知模拟
    pub async fn load_team_players(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        current_season: i64,
    ) -> Result<(
        HashMap<u64, Vec<MatchPlayerInfo>>,
        HashMap<u64, Vec<MatchPlayerInfo>>,
        HashMap<u64, PlayerFormFactors>,
        HashMap<u64, AITeamPersonality>,
        HashMap<u64, HashMap<u8, MasteryTier>>,
    ), String> {
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.ability, p.stability, p.age, p.position, p.team_id, p.join_season,
                   p.potential, p.satisfaction, p.is_starter,
                   COALESCE(pff.form_cycle, 50.0) as form_cycle,
                   COALESCE(pff.momentum, 0) as momentum,
                   COALESCE(pff.last_performance, 0.0) as last_performance,
                   COALESCE(pff.last_match_won, 0) as last_match_won,
                   COALESCE(pff.games_since_rest, 0) as games_since_rest
            FROM players p
            LEFT JOIN player_form_factors pff ON p.id = pff.player_id AND pff.save_id = ?
            WHERE p.save_id = ? AND p.status = 'Active'
            "#,
        )
        .bind(save_id)
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("加载选手数据失败: {}", e))?;

        let player_ids: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>("id")).collect();

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

        let mut player_mastery_map: HashMap<u64, HashMap<u8, MasteryTier>> = HashMap::new();
        if !player_ids.is_empty() {
            let placeholders: String = player_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!(
                "SELECT player_id, champion_id, mastery_tier FROM player_champion_mastery WHERE save_id = ? AND player_id IN ({})",
                placeholders
            );
            let mut query = sqlx::query(&query_str).bind(save_id);
            for pid in &player_ids {
                query = query.bind(pid);
            }
            let mastery_rows = query
                .fetch_all(pool)
                .await
                .map_err(|e| format!("加载英雄熟练度失败: {}", e))?;

            for row in &mastery_rows {
                let pid = row.get::<i64, _>("player_id") as u64;
                let champion_id = row.get::<i64, _>("champion_id") as u8;
                let tier_str: String = row.get("mastery_tier");
                if let Some(tier) = MasteryTier::from_id(&tier_str) {
                    player_mastery_map
                        .entry(pid)
                        .or_default()
                        .insert(champion_id, tier);
                }
            }
        }

        let mut team_players: HashMap<u64, Vec<MatchPlayerInfo>> = HashMap::new();
        let mut team_bench: HashMap<u64, Vec<MatchPlayerInfo>> = HashMap::new();
        let mut form_factors_map: HashMap<u64, PlayerFormFactors> = HashMap::new();
        let mut team_join_seasons: HashMap<u64, i64> = HashMap::new();

        for row in &rows {
            let player_id = row.get::<i64, _>("id") as u64;
            let team_id = row.get::<i64, _>("team_id") as u64;
            let join_season: i64 = row.get("join_season");
            let ability = row.get::<i64, _>("ability") as u8;
            let age = row.get::<i64, _>("age") as u8;
            let is_starter = row.get::<i64, _>("is_starter") == 1;
            team_join_seasons.insert(player_id, join_season);

            let factors = PlayerFormFactors {
                player_id,
                form_cycle: row.get::<f64, _>("form_cycle"),
                momentum: row.get::<i64, _>("momentum") as i8,
                last_performance: row.get::<f64, _>("last_performance"),
                last_match_won: row.get::<i64, _>("last_match_won") != 0,
                games_since_rest: row.get::<i64, _>("games_since_rest") as u32,
            };

            let condition = ConditionEngine::calculate_condition(age, ability, &factors, None);

            form_factors_map.insert(player_id, factors.clone());

            let player_info = MatchPlayerInfo {
                player_id,
                ability,
                stability: row.get::<i64, _>("stability") as u8,
                condition,
                age,
                position: row.get::<String, _>("position"),
                traits: player_traits_map.get(&player_id).cloned().unwrap_or_default(),
                is_first_season: join_season == current_season,
                is_starter,
                join_season,
                potential: row.get::<Option<i64>, _>("potential").unwrap_or(ability as i64) as u8,
                satisfaction: row.get::<Option<i64>, _>("satisfaction").unwrap_or(60) as u8,
                form_factors: Some(factors),
                bp_modifier: 0.0,
                champion_version_score: 0.0,
            };

            if is_starter {
                team_players.entry(team_id).or_default().push(player_info);
            } else {
                team_bench.entry(team_id).or_default().push(player_info);
            }
        }

        // 协同加成（仅首发）
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

        // 加载队伍 personality
        let mut team_personalities: HashMap<u64, AITeamPersonality> = HashMap::new();
        let personality_rows = sqlx::query(
            "SELECT team_id, personality FROM team_personality_configs WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for row in &personality_rows {
            let team_id = row.get::<i64, _>("team_id") as u64;
            let personality_str: String = row.get("personality");
            team_personalities.insert(team_id, AITeamPersonality::from_str(&personality_str));
        }

        Ok((
            team_players,
            team_bench,
            form_factors_map,
            team_personalities,
            player_mastery_map,
        ))
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

    pub(crate) fn update_form_factors_bench_after_match(
        team_bench: &HashMap<u64, Vec<MatchPlayerInfo>>,
        form_factors_map: &mut HashMap<u64, PlayerFormFactors>,
        team_id: u64,
    ) {
        if let Some(bench) = team_bench.get(&team_id) {
            for player in bench.iter() {
                if let Some(factors) = form_factors_map.remove(&player.player_id) {
                    let updated = ConditionEngine::update_form_factors_bench(factors);
                    form_factors_map.insert(player.player_id, updated);
                }
            }
        }
    }

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

    async fn update_season_games_played(
        pool: &Pool<Sqlite>,
        save_id: &str,
        games_played: HashMap<u64, u8>,
    ) -> Result<(), String> {
        if games_played.is_empty() {
            return Ok(());
        }

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        for (player_id, count) in games_played {
            sqlx::query(
                "UPDATE players SET season_games_played = season_games_played + ? WHERE id = ? AND save_id = ?",
            )
            .bind(count as i64)
            .bind(player_id as i64)
            .bind(save_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn update_season_games_total(
        pool: &Pool<Sqlite>,
        save_id: &str,
        home_team_id: u64,
        away_team_id: u64,
        total_games: i64,
    ) -> Result<(), String> {
        if total_games <= 0 {
            return Ok(());
        }
        sqlx::query(
            "UPDATE players SET season_games_total = season_games_total + ? WHERE team_id = ? AND save_id = ? AND status = 'Active'"
        )
        .bind(total_games)
        .bind(home_team_id as i64)
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            "UPDATE players SET season_games_total = season_games_total + ? WHERE team_id = ? AND save_id = ? AND status = 'Active'"
        )
        .bind(total_games)
        .bind(away_team_id as i64)
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 将 draft 结果存入 game_draft_results 表
    async fn save_draft_result(
        pool: &Pool<Sqlite>,
        save_id: &str,
        match_id: u64,
        draft: &DraftResult,
    ) -> Result<(), String> {
        let bans_json = serde_json::to_string(&draft.bans).unwrap_or_default();
        let home_picks_json = serde_json::to_string(&draft.home_picks).unwrap_or_default();
        let away_picks_json = serde_json::to_string(&draft.away_picks).unwrap_or_default();
        let home_comp = draft.home_comp.as_ref().map(|c| format!("{:?}", c));
        let away_comp = draft.away_comp.as_ref().map(|c| format!("{:?}", c));

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO game_draft_results
                (save_id, match_id, game_number, bans_json, home_picks_json, away_picks_json, home_comp, away_comp)
            VALUES (?, ?, 1, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(save_id)
        .bind(match_id as i64)
        .bind(&bans_json)
        .bind(&home_picks_json)
        .bind(&away_picks_json)
        .bind(&home_comp)
        .bind(&away_comp)
        .execute(pool)
        .await
        .map_err(|e| format!("存储BP结果失败: {}", e))?;

        Ok(())
    }

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
