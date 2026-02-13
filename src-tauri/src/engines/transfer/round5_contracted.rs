use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;

use crate::engines::market_value::MarketValueEngine;
use crate::models::team::FinancialStatus;
use crate::models::transfer::*;

use super::cache::{PlayerCacheUpdate, TransferCache};
use super::utils::normalize_position;
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第5轮：有合同选手挖角
    // ============================================

    pub(crate) async fn execute_contracted_player_transfer(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
        round: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // 获取所有挂牌且尚未售出的选手
        let listings: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT pl.id as listing_id, pl.player_id, pl.listed_by_team_id, pl.listing_price, pl.min_accept_price,
                      p.game_id, p.ability, p.age, p.position, p.salary, p.loyalty,
                      p.home_region_id, p.region_loyalty, p.potential, p.tag, p.stability, p.calculated_market_value,
                      t.name as from_team_name
               FROM player_listings pl
               JOIN players p ON pl.player_id = p.id
               JOIN teams t ON pl.listed_by_team_id = t.id
               WHERE pl.window_id = ? AND pl.status = 'ACTIVE'
               ORDER BY p.ability DESC"#
        )
        .bind(window_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询挂牌选手失败: {}", e))?;

        // 使用缓存获取所有球队ID
        let team_ids: Vec<i64> = cache.team_names.keys().copied().collect();

        let window_transfer_rows_r5: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT to_team_id, COUNT(*) as cnt FROM transfer_events
               WHERE window_id = ? AND to_team_id IS NOT NULL
               AND event_type IN ('FREE_AGENT_SIGNING', 'TRANSFER_PURCHASE', 'EMERGENCY_SIGNING')
               GROUP BY to_team_id"#
        )
        .bind(window_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        let team_window_counts: HashMap<i64, i64> = window_transfer_rows_r5.iter()
            .map(|r| (r.get::<i64, _>("to_team_id"), r.get::<i64, _>("cnt")))
            .collect();

        for listing in &listings {
            let listing_id: i64 = listing.get("listing_id");
            let player_id: i64 = listing.get("player_id");
            let from_team_id: i64 = listing.get("listed_by_team_id");
            let listing_price: i64 = listing.get("listing_price");
            let min_price: i64 = listing.get("min_accept_price");
            let game_id: String = listing.get("game_id");
            let ability: i64 = listing.get("ability");
            let age: i64 = listing.get("age");
            let position: String = normalize_position(&listing.get::<String, _>("position"));
            let salary: i64 = listing.get("salary");
            let loyalty: i64 = listing.get("loyalty");
            let from_team_name: String = listing.get("from_team_name");
            let home_region_id: Option<i64> = listing.try_get("home_region_id").ok();
            let region_loyalty: i64 = listing.try_get("region_loyalty").unwrap_or(70);
            let potential: i64 = listing.try_get("potential").unwrap_or(0);
            let tag: String = listing.try_get("tag").unwrap_or_else(|_| "NORMAL".to_string());
            let stability: i64 = listing.try_get("stability").unwrap_or(60);
            let calculated_market_value: i64 = listing.try_get("calculated_market_value").unwrap_or(0);

            let mut all_bids: Vec<(i64, String, i64, i64, i64, Option<i64>, f64)> = Vec::new();
            // (team_id, team_name, bid_price, expected_salary, contract_years, target_region_id, match_score)

            for &team_id in &team_ids {
                if team_id == from_team_id {
                    continue;
                }

                let window_count = team_window_counts.get(&team_id).copied().unwrap_or(0);
                if window_count >= self.config.max_transfers_per_window {
                    continue;
                }

                let balance = cache.team_balances.get(&team_id).copied().unwrap_or(0);
                if balance < min_price {
                    continue;
                }

                let fin_status = FinancialStatus::from_balance(balance);
                if !fin_status.can_buy() {
                    continue;
                }

                // 使用缓存检查位置需求
                let roster = cache.get_roster(team_id);
                let pos_count = roster.iter()
                    .filter(|r| r.position.eq_ignore_ascii_case(&position))
                    .count();

                let over_threshold = roster.len() as i64 - 8;
                if pos_count >= 2 || over_threshold >= 5 {
                    continue;
                }

                if pos_count == 1 {
                    let best_ability_at_pos = roster.iter()
                        .filter(|r| r.position.eq_ignore_ascii_case(&position))
                        .map(|r| r.ability)
                        .max()
                        .unwrap_or(0);
                    let is_upgrade = ability > best_ability_at_pos;
                    let is_youth_prospect = age <= 23 && potential >= 70 && potential > best_ability_at_pos;
                    if !is_upgrade && !is_youth_prospect {
                        continue;
                    }
                }

                // 使用缓存获取AI性格权重
                let weights = cache.get_weights(team_id);
                let team_rank = cache.get_composite_rank(team_id);

                let match_score = self.calculate_match_score(
                    ability as u8, age as u8, &position, &weights, balance,
                    &roster, team_rank,
                    potential as u8, stability as u8, &tag,
                );

                // 超出奢侈税起征线时，降低匹配分数
                let match_score = if over_threshold > 0 {
                    match_score * (1.0 - over_threshold as f64 * 0.25)
                } else {
                    match_score
                };

                if match_score < 50.0 {
                    continue;
                }

                // 出价
                let bid_price = (listing_price as f64 * (0.9 + rng.gen::<f64>() * 0.2)) as i64;
                if bid_price < min_price || bid_price > balance {
                    continue;
                }

                let team_name = cache.get_team_name(team_id);
                // 使用完整身价（含荣誉系数）计算期望薪资，而不是转会标价
                let market_value = if calculated_market_value > 0 {
                    calculated_market_value as u64
                } else {
                    MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, potential as u8, &tag, &position)
                };
                let base_salary = MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64;
                // 根据球队AI性格和随机波动调整报价薪资
                let salary_multiplier = {
                    let base_mult = if weights.star_chasing > 0.7 { 1.15 }
                        else if weights.star_chasing > 0.4 { 1.05 }
                        else if weights.bargain_hunting > 0.7 { 0.82 }
                        else if weights.bargain_hunting > 0.4 { 0.90 }
                        else { 0.95 };
                    let random_factor = 0.92 + rng.gen::<f64>() * 0.16;
                    base_mult * random_factor
                };
                let expected_salary = (base_salary as f64 * salary_multiplier) as i64;
                let contract_years = {
                    let base: i64 = if age <= 22 { 3 } else if age <= 25 { 2 } else if age <= 28 { 2 } else { 1 };
                    let personality_adj: i64 = if weights.long_term_focus > 0.7 { 1 } else if weights.short_term_focus > 0.7 { -1 } else { 0 };
                    let random_adj: i64 = if rng.gen::<f64>() < 0.3 { 1 } else if rng.gen::<f64>() < 0.25 { -1 } else { 0 };
                    (base + personality_adj + random_adj).clamp(1, 4)
                };
                let target_region_id = cache.team_region_ids.get(&team_id).copied().flatten();

                all_bids.push((team_id, team_name, bid_price, expected_salary, contract_years, target_region_id, match_score));
            }

            if all_bids.is_empty() {
                continue;
            }

            // 按出价金额降序排列
            all_bids.sort_by(|a, b| b.2.cmp(&a.2));

            // 竞价升温：多个球队竞标时推高出价
            if all_bids.len() >= 2 {
                let bid_premium = (1.0 + (all_bids.len() as f64 - 1.0) * 0.04).min(1.20);
                for bid in all_bids.iter_mut() {
                    bid.2 = (bid.2 as f64 * bid_premium) as i64;  // 推高转会费
                }
            }

            // 溢价后重新验证预算，剔除余额不足的竞标
            all_bids.retain(|bid| {
                let balance = cache.team_balances.get(&bid.0).copied().unwrap_or(0);
                if bid.2 > balance {
                    log::debug!("R5: {}出价{}超出余额{}，剔除", bid.1, bid.2, balance);
                    false
                } else {
                    true
                }
            });

            if all_bids.is_empty() {
                continue;
            }

            // 对所有竞标计算 willingness
            struct R5BidRecord {
                idx: usize,
                willingness: f64,
            }
            let mut bid_records: Vec<R5BidRecord> = Vec::new();

            let market_value_for_willingness = if calculated_market_value > 0 {
                calculated_market_value as u64
            } else {
                MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, potential as u8, &tag, &position)
            };
            let reference_salary = MarketValueEngine::estimate_salary(market_value_for_willingness, ability as u8, age as u8) as i64;
            let willingness_salary_base = reference_salary.max(salary);

            for (idx, bid) in all_bids.iter().enumerate() {
                let target_roster = cache.get_roster(bid.0);
                let target_team_rank = cache.get_composite_rank(bid.0);
                let target_team_reputation = cache.get_team_reputation(bid.0);
                let willingness = self.calculate_willingness(
                    ability as u8, loyalty as u8, age as u8,
                    bid.3, willingness_salary_base,
                    home_region_id, bid.5, region_loyalty,
                    target_team_rank, target_team_reputation,
                    &target_roster, &position,
                    cache.get_player_stats(player_id),
                    &mut rng,
                );
                bid_records.push(R5BidRecord { idx, willingness });
            }

            // 按 bid_price 降序遍历，第一个 willingness >= 40 的中标（允许次高出价中标）
            let winner_idx = bid_records.iter()
                .find(|r| r.willingness >= 40.0)
                .map(|r| r.idx);

            // 写入所有竞价记录
            for record in &bid_records {
                let bid = &all_bids[record.idx];
                let is_winner = Some(record.idx) == winner_idx;
                let reject_reason = if is_winner {
                    None
                } else if record.willingness < 40.0 {
                    Some("willingness_too_low")
                } else {
                    Some("outbid")
                };
                let _ = Self::insert_bid(
                    pool, window_id, round,
                    player_id, &game_id, ability, age, &position,
                    Some(from_team_id), Some(&from_team_name),
                    bid.0, &bid.1, bid.5,
                    bid.3, bid.4, bid.2, 0,
                    bid.6, record.willingness, is_winner, reject_reason,
                ).await;
            }

            if let Some(widx) = winner_idx {
                let (to_team_id, ref to_team_name, bid_price, new_salary, contract_years, _target_region_id, _match_score) = all_bids[widx];

                // 执行转会
                sqlx::query(
                    "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = 50, satisfaction = 55, join_season = ? WHERE id = ?"
                )
                .bind(to_team_id)
                .bind(new_salary)
                .bind(season_id + contract_years)
                .bind(season_id)
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("转会更新失败: {}", e))?;

                // 资金变动
                sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
                    .bind(bid_price)
                    .bind(to_team_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("买方扣款失败: {}", e))?;

                sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ?")
                    .bind(bid_price)
                    .bind(from_team_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("卖方收款失败: {}", e))?;

                // 记录财务交易：买方转会费支出
                Self::record_financial_transaction(
                    pool, save_id, season_id, to_team_id,
                    "TransferOut",
                    -(bid_price),
                    &format!("转会费支出: 买入{}", game_id),
                    player_id,
                ).await?;

                // 记录财务交易：卖方转会费收入
                Self::record_financial_transaction(
                    pool, save_id, season_id, from_team_id,
                    "TransferIn",
                    bid_price,
                    &format!("转会费收入: 卖出{}", game_id),
                    player_id,
                ).await?;

                // 更新缓存
                cache.transfer_player(player_id, Some(from_team_id), Some(to_team_id), Some(PlayerCacheUpdate {
                    salary: Some(new_salary),
                    loyalty: Some(50),
                    satisfaction: Some(55),
                    contract_end_season: Some(season_id + contract_years),
                }));
                cache.update_balance(to_team_id, -bid_price);
                cache.update_balance(from_team_id, bid_price);

                // 更新挂牌状态
                sqlx::query(
                    "UPDATE player_listings SET status = 'SOLD', sold_at = datetime('now'), sold_to_team_id = ?, actual_price = ? WHERE id = ?"
                )
                .bind(to_team_id)
                .bind(bid_price)
                .bind(listing_id)
                .execute(pool)
                .await
                .map_err(|e| format!("更新挂牌状态失败: {}", e))?;

                // 记录合同历史
                Self::insert_contract(pool, save_id, player_id, to_team_id, "TRANSFER", new_salary * contract_years, contract_years, season_id, bid_price, 0).await?;

                let event = self.record_event(
                    pool, window_id, round,
                    TransferEventType::TransferPurchase,
                    EventLevel::from_ability_and_fee(ability as u8, bid_price),
                    player_id, &game_id, ability,
                    Some(from_team_id), Some(&from_team_name),
                    Some(to_team_id), Some(&to_team_name),
                    bid_price, new_salary, contract_years,
                    &format!("{}从{}转会至{}，转会费{}万", game_id, from_team_name, to_team_name, bid_price / 10000),
                ).await?;
                events.push(event);
            }
        }

        Ok(RoundResult {
            round,
            round_name: "有合同选手挖角".to_string(),
            events,
            summary: "已完成有合同选手交易".to_string(),
        })
    }

    // ============================================
    // 第6轮：财政调整
    // ============================================
}
