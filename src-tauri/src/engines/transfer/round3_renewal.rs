use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;

use crate::engines::market_value::MarketValueEngine;
use crate::models::team::FinancialStatus;
use crate::models::transfer::*;

use super::cache::TransferCache;
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第2轮：续约谈判
    // ============================================

    pub(crate) async fn execute_renewal_negotiations(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // 获取合同即将到期的选手（contract_end_season = 当前赛季）
        let expiring_players: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT p.id, p.game_id, p.ability, p.salary, p.loyalty, p.satisfaction,
                      p.team_id, p.age, p.potential, p.tag, p.calculated_market_value, p.contract_role, t.name as team_name
               FROM players p
               LEFT JOIN teams t ON p.team_id = t.id
               WHERE p.save_id = ? AND p.status = 'Active'
               AND p.team_id IS NOT NULL
               AND p.contract_end_season IS NOT NULL
               AND p.contract_end_season <= ?"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询到期合同失败: {}", e))?;

        // R2 已写入球队赛季评估策略，这里缓存为 team_id -> strategy
        let strategy_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT team_id, strategy
               FROM team_season_evaluations
               WHERE save_id = ? AND window_id = ?
               ORDER BY id DESC"#,
        )
        .bind(save_id)
        .bind(window_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_strategies: HashMap<i64, String> = HashMap::new();
        for row in strategy_rows {
            let team_id: i64 = row.get("team_id");
            if team_strategies.contains_key(&team_id) {
                continue;
            }
            let strategy: String = row
                .try_get("strategy")
                .unwrap_or_else(|_| "MAINTAIN".to_string());
            team_strategies.insert(team_id, strategy);
        }

        for player in &expiring_players {
            let player_id: i64 = player.get("id");
            let game_id: String = player.get("game_id");
            let ability: i64 = player.get("ability");
            let salary: i64 = player.get("salary");
            let loyalty: i64 = player.get("loyalty");
            let satisfaction: i64 = player.get("satisfaction");
            let team_id: i64 = player.get("team_id");
            let team_name: String = player.get("team_name");
            let age: i64 = player.get("age");
            let calculated_market_value: i64 = player.try_get("calculated_market_value").unwrap_or(0);
            let contract_role: String = player.try_get("contract_role").unwrap_or_else(|_| "Starter".to_string());

            let roster = cache.get_roster(team_id);
            let cached_player = roster.iter().find(|p| p.id == player_id);
            let is_starter = cached_player
                .map(|p| p.is_starter)
                .unwrap_or(contract_role != "Sub");
            let player_position = cached_player.map(|p| p.position.clone());
            let team_rank = cache.get_composite_rank(team_id);
            let team_balance = cache.team_balances.get(&team_id).copied().unwrap_or(0);
            let team_strategy = team_strategies
                .get(&team_id)
                .map(String::as_str)
                .unwrap_or("MAINTAIN");

            let market_value = if calculated_market_value > 0 {
                calculated_market_value as u64
            } else {
                MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", "MID")
            };
            let expected_salary = (MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64).max(1);

            // Step 1: 退役判定（先于续约）
            let forced_retire = age >= 37
                || (age >= 35 && ability < 50)
                || (age >= 33 && ability < 45)
                || (age >= 30 && ability < 40);

            let mut retire_chance = 0.0_f64;
            if age >= 33 && ability < 55 {
                retire_chance = retire_chance.max((age - 32) as f64 * 0.15);
            }
            if age >= 30 && satisfaction < 30 {
                retire_chance = retire_chance.max(0.30);
            }
            retire_chance = retire_chance.clamp(0.0, 1.0);

            let should_retire =
                forced_retire || (retire_chance > 0.0 && rng.gen_range(0.0..1.0) < retire_chance);
            if should_retire {
                sqlx::query("UPDATE players SET status = 'Retired' WHERE id = ?")
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .ok();

                sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
                    .bind(save_id)
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .ok();

                cache.release_player(player_id, team_id);

                let event = self
                    .record_event(
                        pool,
                        window_id,
                        3,
                        TransferEventType::Retirement,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id,
                        &game_id,
                        ability,
                        Some(team_id),
                        Some(&team_name),
                        None,
                        None,
                        0,
                        0,
                        0,
                        &format!("[退役] {}宣布退役 | {}岁 能力{}", game_id, age, ability),
                    )
                    .await?;
                events.push(event);
                continue;
            }

            // Step 2: 球队决策 - 是否续约
            let has_stronger_same_position = if let Some(pos) = player_position.as_deref() {
                roster.iter().any(|p| {
                    p.id != player_id
                        && p.position.eq_ignore_ascii_case(pos)
                        && p.ability - ability >= 10
                })
            } else {
                false
            };

            let financial_status = FinancialStatus::from_balance(team_balance);
            let team_declines = (has_stronger_same_position && !is_starter)
                || (team_strategy.eq_ignore_ascii_case("REBUILD") && age >= 28)
                || (matches!(financial_status, FinancialStatus::Bankrupt | FinancialStatus::Deficit)
                    && salary as f64 > expected_salary as f64 * 0.8);

            if team_declines {
                cache.renewal_failed_pairs.insert((player_id, team_id));
                sqlx::query(
                    "UPDATE players SET team_id = NULL, satisfaction = MAX(satisfaction - 20, 0) WHERE id = ?",
                )
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("释放球员失败: {}", e))?;

                sqlx::query(
                    "UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1",
                )
                .bind(save_id)
                .bind(player_id)
                .execute(pool)
                .await
                .ok();

                sqlx::query("UPDATE player_listings SET status = 'CANCELLED' WHERE player_id = ? AND window_id = ? AND status = 'ACTIVE'")
                    .bind(player_id)
                    .bind(window_id)
                    .execute(pool)
                    .await
                    .ok();

                cache.release_player(player_id, team_id);

                let event = self
                    .record_event(
                        pool,
                        window_id,
                        3,
                        TransferEventType::ContractTermination,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id,
                        &game_id,
                        ability,
                        Some(team_id),
                        Some(&team_name),
                        None,
                        None,
                        0,
                        salary,
                        0,
                        &format!("[合同到期] 球队决定不续约{}", game_id),
                    )
                    .await?;
                events.push(event);
                continue;
            }

            // Step 3: 选手决策 - 是否续约
            let player_declines = satisfaction < 30
                || (loyalty < 20 && team_rank > 10)
                || (ability >= 80 && team_rank > 8);

            if player_declines {
                cache.renewal_failed_pairs.insert((player_id, team_id));
                sqlx::query(
                    "UPDATE players SET team_id = NULL, satisfaction = MAX(satisfaction - 20, 0) WHERE id = ?",
                )
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("释放球员失败: {}", e))?;

                sqlx::query(
                    "UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1",
                )
                .bind(save_id)
                .bind(player_id)
                .execute(pool)
                .await
                .ok();

                sqlx::query("UPDATE player_listings SET status = 'CANCELLED' WHERE player_id = ? AND window_id = ? AND status = 'ACTIVE'")
                    .bind(player_id)
                    .bind(window_id)
                    .execute(pool)
                    .await
                    .ok();

                cache.release_player(player_id, team_id);

                let event = self
                    .record_event(
                        pool,
                        window_id,
                        3,
                        TransferEventType::ContractTermination,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id,
                        &game_id,
                        ability,
                        Some(team_id),
                        Some(&team_name),
                        None,
                        None,
                        0,
                        salary,
                        0,
                        &format!("[合同到期] {}拒绝续约，寻求新机会", game_id),
                    )
                    .await?;
                events.push(event);
                continue;
            }

            // Step 4: 薪资博弈（双方都愿意续约）
            let player_leverage: f64 = {
                let ability_factor = if ability >= 80 {
                    0.15
                } else if ability >= 70 {
                    0.08
                } else if ability >= 60 {
                    0.0
                } else {
                    -0.05
                };
                let age_factor = if age <= 24 {
                    0.05
                } else if age >= 30 {
                    -0.08
                } else {
                    0.0
                };
                let loyalty_factor = if loyalty <= 40 {
                    0.10
                } else if loyalty >= 80 {
                    -0.05
                } else {
                    0.0
                };
                (1.0_f64 + ability_factor + age_factor + loyalty_factor)
                    .clamp(0.85_f64, 1.20_f64)
            };
            let player_expected_salary =
                ((expected_salary as f64 * player_leverage).round() as i64).max(1);

            let team_budget_factor: f64 = {
                let rank_factor = if team_rank <= 3 {
                    0.15
                } else if team_rank <= 7 {
                    0.05
                } else if team_rank <= 10 {
                    0.0
                } else {
                    -0.10
                };
                let balance_factor = if team_balance > 50_000_000 {
                    0.10
                } else if team_balance < 5_000_000 {
                    -0.15
                } else {
                    0.0
                };
                (1.0_f64 + rank_factor + balance_factor).clamp(0.75_f64, 1.25_f64)
            };

            let mut renewed = false;
            let mut final_team_offer = ((expected_salary as f64 * team_budget_factor * 0.85).round() as i64).max(1);

            for negotiation_round in 0..self.config.negotiation_max_rounds {
                let offer_multiplier = match negotiation_round {
                    0 => 0.85,
                    1 => 0.92,
                    _ => 1.0,
                };
                let team_offer =
                    ((expected_salary as f64 * team_budget_factor * offer_multiplier).round() as i64)
                        .max(1);
                final_team_offer = team_offer;

                if (team_offer as f64) >= (player_expected_salary as f64) * 0.90 {
                    renewed = true;
                    break;
                }
                if (team_offer as f64) < (player_expected_salary as f64) * 0.75 {
                    continue;
                }
            }

            if renewed {
                // 续约合同年限：沿用原逻辑
                let base_years: i64 = if age <= 22 { 3 } else if age <= 25 { 2 } else if age <= 28 { 2 } else { 1 };
                let random_adj: i64 = if rng.gen_range(0.0..1.0) < 0.4 {
                    1
                } else if rng.gen_range(0.0..1.0) < 0.3 {
                    -1
                } else {
                    0
                };
                let new_contract_years = (base_years + random_adj).clamp(1, 4);

                let mut new_salary = ((final_team_offer + player_expected_salary) / 2).max(1);
                if contract_role == "Sub" {
                    new_salary = ((new_salary as f64 * 0.85).round() as i64).max(1);
                }

                sqlx::query(
                    "UPDATE players SET salary = ?, contract_end_season = ?, loyalty = MIN(loyalty + 5, 100) WHERE id = ?",
                )
                .bind(new_salary)
                .bind(season_id + new_contract_years)
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("续约更新失败: {}", e))?;

                // 续约成功后清理R2可能生成的挂牌记录
                sqlx::query("UPDATE player_listings SET status = 'CANCELLED' WHERE player_id = ? AND window_id = ? AND status = 'ACTIVE'")
                    .bind(player_id)
                    .bind(window_id)
                    .execute(pool)
                    .await
                    .ok();

                // 记录合同历史
                Self::insert_contract(
                    pool,
                    save_id,
                    player_id,
                    team_id,
                    "RENEWAL",
                    new_salary * new_contract_years,
                    new_contract_years,
                    season_id,
                    0,
                    0,
                )
                .await?;

                let event = self
                    .record_event(
                        pool,
                        window_id,
                        3,
                        TransferEventType::ContractRenewal,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id,
                        &game_id,
                        ability,
                        Some(team_id),
                        Some(&team_name),
                        Some(team_id),
                        Some(&team_name),
                        0,
                        new_salary,
                        new_contract_years,
                        &format!(
                            "[合同到期] 续约{}年，年薪{}万 | {}岁{}",
                            new_contract_years,
                            new_salary / 10000,
                            age,
                            contract_role
                        ),
                    )
                    .await?;
                events.push(event);
            }

            if !renewed {
                // 续约失败，成为自由球员
                cache.renewal_failed_pairs.insert((player_id, team_id));
                sqlx::query(
                    "UPDATE players SET team_id = NULL, satisfaction = MAX(satisfaction - 20, 0) WHERE id = ?"
                )
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("释放球员失败: {}", e))?;

                // 旧合同失效
                sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
                    .bind(save_id).bind(player_id).execute(pool).await.ok();

                // 清理该选手的活跃挂牌记录（防止R5对已成为自由球员的选手进行有合同转会）
                sqlx::query("UPDATE player_listings SET status = 'CANCELLED' WHERE player_id = ? AND window_id = ? AND status = 'ACTIVE'")
                    .bind(player_id).bind(window_id).execute(pool).await.ok();

                // 更新缓存
                cache.release_player(player_id, team_id);

                let event = self.record_event(
                    pool, window_id, 3,
                    TransferEventType::ContractTermination,
                    EventLevel::from_ability_and_fee(ability as u8, 0),
                    player_id, &game_id, ability,
                    Some(team_id), Some(&team_name),
                    None, None,
                    0, salary, 0,
                    &format!("[合同到期] 续约谈判失败，{}成为自由球员 | {}岁 忠诚{}满意{}", game_id, age, loyalty, satisfaction),
                ).await?;
                events.push(event);
            }
        }

        Ok(RoundResult {
            round: 3,
            round_name: "续约谈判".to_string(),
            events,
            summary: "已完成续约谈判：处理到期合同选手".to_string(),
        })
    }

    // ============================================
    // 第2轮：双向评估（战队评估选手 + 选手评估战队）
    // ============================================
}
