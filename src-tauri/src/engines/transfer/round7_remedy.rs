use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};

use crate::engines::market_value::MarketValueEngine;
use crate::models::player::Position;
use crate::models::transfer::*;

use super::cache::{CachedPlayer, TransferCache};
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第7轮：收尾补救
    // ============================================

    pub(crate) async fn execute_final_remedy(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // ============================================
        // 0. 复用R5逻辑：处理所有活跃挂牌选手（含R6破产挂牌）
        // ============================================
        let r5_repeat = self.execute_contracted_player_transfer(pool, window_id, save_id, season_id, cache, 7).await?;
        events.extend(r5_repeat.events);

        // ============================================
        // 1. 检查所有球队阵容完整性，紧急补人
        // ============================================
        // 使用缓存检查所有球队阵容完整性
        let team_ids: Vec<i64> = cache.team_names.keys().copied().collect();

        for &team_id in &team_ids {
            let team_name = cache.get_team_name(team_id);
            let roster = cache.get_roster(team_id);

            if roster.len() >= 13 {
                continue;
            }

            let mut has_position = [false; 5];
            for player in &roster {
                match player.position.to_uppercase().as_str() {
                    "TOP" => has_position[0] = true,
                    "JUG" => has_position[1] = true,
                    "MID" => has_position[2] = true,
                    "ADC" => has_position[3] = true,
                    "SUP" => has_position[4] = true,
                    _ => {}
                }
            }

            if has_position.iter().all(|&h| h) {
                continue; // 所有位置都有人，跳过
            }

            let all_positions = [
                (Position::Top, "TOP"),
                (Position::Jug, "JUG"),
                (Position::Mid, "MID"),
                (Position::Adc, "ADC"),
                (Position::Sup, "SUP"),
            ];

            let mut rng = rand::rngs::StdRng::from_entropy();
            let team_rank = cache.get_composite_rank(team_id);
            let team_reputation = cache.get_team_reputation(team_id);
            let team_region_id = cache.team_region_ids.get(&team_id).copied().flatten();

            for (i, (_, pos_str)) in all_positions.iter().enumerate() {
                if has_position[i] {
                    continue;
                }

                let candidates: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
                    r#"SELECT id, game_id, ability, age, potential, tag, loyalty,
                              home_region_id, region_loyalty, stability
                       FROM players
                       WHERE save_id = ? AND status = 'Active' AND team_id IS NULL AND UPPER(position) = UPPER(?)
                       ORDER BY ability DESC
                       LIMIT 10"#
                )
                .bind(save_id)
                .bind(*pos_str)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("查找紧急签约候选失败: {}", e))?;

                let mut best_candidate: Option<(i64, String, i64, i64, f64)> = None;
                let target_roster = cache.get_roster(team_id);

                for candidate in &candidates {
                    let c_id: i64 = candidate.get("id");
                    if cache.renewal_failed_pairs.contains(&(c_id, team_id)) {
                        continue;
                    }
                    let c_game_id: String = candidate.get("game_id");
                    let c_ability: i64 = candidate.get("ability");
                    let c_age: i64 = candidate.get("age");
                    let c_loyalty: i64 = candidate.try_get("loyalty").unwrap_or(50);
                    let c_home_region_id: Option<i64> = candidate.try_get("home_region_id").ok();
                    let c_region_loyalty: i64 = candidate.try_get("region_loyalty").unwrap_or(70);

                    let salary_est = MarketValueEngine::estimate_salary(
                        MarketValueEngine::calculate_base_market_value(c_ability as u8, c_age as u8, c_ability as u8, "NORMAL", pos_str),
                        c_ability as u8, c_age as u8,
                    ) as i64;

                    let willingness = self.calculate_willingness(
                        c_ability as u8, c_loyalty as u8, c_age as u8,
                        salary_est, salary_est,
                        c_home_region_id, team_region_id, c_region_loyalty,
                        team_rank, team_reputation,
                        &target_roster, pos_str,
                        cache.get_player_stats(c_id),
                        &mut rng,
                    );

                    if willingness >= 30.0 {
                        best_candidate = Some((c_id, c_game_id, c_ability, c_age, willingness));
                        break;
                    }

                    if best_candidate.is_none() {
                        best_candidate = Some((c_id, c_game_id, c_ability, c_age, willingness));
                    }
                }

                if let Some((player_id, game_id, ability, age, _willingness)) = best_candidate {
                    let salary = MarketValueEngine::estimate_salary(MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", pos_str), ability as u8, age as u8) as i64;
                    let contract_years: i64 = if age <= 25 && rng.gen::<f64>() < 0.4 { 2 } else { 1 };

                    sqlx::query(
                        "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = 40, satisfaction = 50, join_season = ? WHERE id = ?"
                    )
                    .bind(team_id)
                    .bind(salary)
                    .bind(season_id + contract_years)
                    .bind(season_id)
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("紧急签约失败: {}", e))?;

                    let signing_bonus = salary / 4;
                    let current_balance = cache.team_balances.get(&team_id).copied().unwrap_or(0);
                    let actual_bonus = signing_bonus.min(current_balance.max(0));
                    if actual_bonus > 0 {
                        sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
                            .bind(actual_bonus)
                            .bind(team_id)
                            .execute(pool)
                            .await
                            .map_err(|e| format!("紧急签约扣款失败: {}", e))?;
                        cache.update_balance(team_id, -actual_bonus);

                        Self::record_financial_transaction(
                            pool, save_id, season_id, team_id,
                            "TransferOut",
                            -(actual_bonus),
                            &format!("紧急签约: {}", game_id),
                            player_id,
                        ).await?;
                    }

                    Self::insert_contract(pool, save_id, player_id, team_id, "EMERGENCY", salary * contract_years, contract_years, season_id, 0, actual_bonus).await?;

                    let new_player = CachedPlayer {
                        id: player_id,
                        game_id: game_id.clone(),
                        ability,
                        potential: 0,
                        age,
                        salary,
                        loyalty: 40,
                        satisfaction: 50,
                        position: pos_str.to_string(),
                        tag: "NORMAL".to_string(),
                        team_id: Some(team_id),
                        is_starter: false,
                        home_region_id: None,
                        region_loyalty: 70,
                        contract_end_season: Some(season_id + contract_years),
                        status: "Active".to_string(),
                        stability: 60,
                        growth_accumulator: 0.0,
                    };
                    cache.team_rosters.entry(team_id).or_default().push(new_player);

                    let event = self.record_event(
                        pool, window_id, 7,
                        TransferEventType::EmergencySigning,
                        EventLevel::C,
                        player_id, &game_id, ability,
                        None, None,
                        Some(team_id), Some(&team_name),
                        0, salary, contract_years,
                        &format!("{}紧急签约{}补充阵容", team_name, game_id),
                    ).await?;
                    events.push(event);
                }
            }
        }

        // 更新所有球队战力（单条SQL优化）
        self.recalculate_team_powers_optimized(pool, save_id).await?;

        Ok(RoundResult {
            round: 7,
            round_name: "收尾补救".to_string(),
            events,
            summary: "已完成收尾补救：确保阵容完整性".to_string(),
        })
    }

    // ============================================
    // 转会窗口关闭验证
    // ============================================
}
