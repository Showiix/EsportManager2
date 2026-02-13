use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};

use crate::engines::market_value::MarketValueEngine;
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
                      p.team_id, p.age, p.potential, p.tag, p.calculated_market_value, t.name as team_name
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

            // 续约谈判逻辑
            let loyalty_bonus = loyalty as f64 / 100.0;
            let satisfaction_bonus = satisfaction as f64 / 100.0;

            let market_value = if calculated_market_value > 0 {
                calculated_market_value as u64
            } else {
                MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", "MID")
            };
            let expected_salary = MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64;

            let team_rank = cache.get_composite_rank(team_id);
            let team_rank_bonus: f64 = match team_rank {
                1..=3 => 0.15,
                4..=7 => 0.08,
                8..=10 => 0.0,
                11..=14 => -0.08,
                _ => -0.12,
            };

            let salary_ratio = if expected_salary > 0 { salary as f64 / expected_salary as f64 } else { 1.0 };
            let salary_competitiveness: f64 = if salary_ratio >= 1.0 {
                0.10
            } else if salary_ratio >= 0.85 {
                0.0
            } else if salary_ratio >= 0.7 {
                -0.10
            } else {
                -0.20
            };

            let renewal_chance = (loyalty_bonus * 0.3 + satisfaction_bonus * 0.3 + 0.15
                + team_rank_bonus + salary_competitiveness).clamp(0.05, 0.95);

            let mut renewed = false;

            for _negotiation_round in 0..self.config.negotiation_max_rounds {
                let roll: f64 = rng.gen();
                if roll < renewal_chance {
                    // 续约成功
                    // 续约合同年限：基于年龄 + 随机浮动，范围 1-4 年
                    let base_years: i64 = if age <= 22 { 3 } else if age <= 25 { 2 } else if age <= 28 { 2 } else { 1 };
                    let random_adj: i64 = if rng.gen::<f64>() < 0.4 { 1 } else if rng.gen::<f64>() < 0.3 { -1 } else { 0 };
                    let new_contract_years = (base_years + random_adj).clamp(1, 4);
                    // 续约薪资博弈：选手筹码越强，要价越高
                    let player_leverage: f64 = {
                        let ability_factor = if ability >= 70 { 0.10 } else if ability >= 60 { 0.0 } else { -0.05 };
                        let loyalty_factor = if loyalty >= 80 { -0.05 } else if loyalty <= 40 { 0.08 } else { 0.0 };
                        let satisfaction_factor = if satisfaction >= 80 { -0.03 } else if satisfaction <= 40 { 0.05 } else { 0.0 };
                        let age_factor = if age <= 24 { 0.05 } else if age >= 30 { -0.05 } else { 0.0 };
                        (1.0_f64 + ability_factor + loyalty_factor + satisfaction_factor + age_factor).clamp(0.85, 1.15)
                    };
                    let new_salary = (expected_salary as f64 * player_leverage) as i64;

                    sqlx::query(
                        "UPDATE players SET salary = ?, contract_end_season = ?, loyalty = MIN(loyalty + 5, 100) WHERE id = ?"
                    )
                    .bind(new_salary)
                    .bind(season_id + new_contract_years)
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("续约更新失败: {}", e))?;

                    // 续约成功后清理R2可能生成的挂牌记录
                    sqlx::query("UPDATE player_listings SET status = 'CANCELLED' WHERE player_id = ? AND window_id = ? AND status = 'ACTIVE'")
                        .bind(player_id).bind(window_id).execute(pool).await.ok();

                    // 记录合同历史
                    Self::insert_contract(pool, save_id, player_id, team_id, "RENEWAL", new_salary * new_contract_years, new_contract_years, season_id, 0, 0).await?;

                    let event = self.record_event(
                        pool, window_id, 3,
                        TransferEventType::ContractRenewal,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id, &game_id, ability,
                        Some(team_id), Some(&team_name),
                        Some(team_id), Some(&team_name),
                        0, new_salary, new_contract_years,
                        &format!("[合同到期] 续约{}年，年薪{}万", new_contract_years, new_salary / 10000),
                    ).await?;
                    events.push(event);
                    renewed = true;
                    break;
                }
            }

            if !renewed {
                // 续约失败，成为自由球员
                cache.renewal_failed_pairs.insert((player_id, team_id));
                sqlx::query(
                    "UPDATE players SET team_id = NULL, satisfaction = MAX(satisfaction - 10, 0) WHERE id = ?"
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
                    &format!("[合同到期] 续约谈判失败，{}成为自由球员", game_id),
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
