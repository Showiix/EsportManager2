use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;

use crate::engines::market_value::MarketValueEngine;
use crate::models::team::FinancialStatus;
use crate::models::transfer::*;

use super::cache::{CachedPlayer, TransferCache};
use super::utils::normalize_position;
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第4轮：自由球员争夺
    // ============================================

    pub(crate) async fn execute_free_agent_bidding(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // 获取所有自由球员（不在任何队伍中的选手，需从数据库查询，因为缓存只存有队伍的选手）
        let free_agents: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT id, game_id, ability, salary, age, position, loyalty, potential, tag,
                      home_region_id, region_loyalty, stability, calculated_market_value
               FROM players
               WHERE save_id = ? AND status = 'Active' AND team_id IS NULL
               ORDER BY ability DESC"#
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询自由球员失败: {}", e))?;

        // 使用缓存获取所有球队ID
        let team_ids: Vec<i64> = cache.team_names.keys().copied().collect();
        // 从数据库查询整个窗口期内每队已完成的转入数量（含R4本轮）
        let window_transfer_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT to_team_id, COUNT(*) as cnt FROM transfer_events
               WHERE window_id = ? AND to_team_id IS NOT NULL
               AND event_type IN ('FREE_AGENT_SIGNING', 'TRANSFER_PURCHASE', 'EMERGENCY_SIGNING')
               GROUP BY to_team_id"#
        )
        .bind(window_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        let mut team_transfer_counts: HashMap<i64, i64> = window_transfer_rows.iter()
            .map(|r| (r.get::<i64, _>("to_team_id"), r.get::<i64, _>("cnt")))
            .collect();

        for free_agent in &free_agents {
            let player_id: i64 = free_agent.get("id");
            let game_id: String = free_agent.get("game_id");
            let ability: i64 = free_agent.get("ability");
            let age: i64 = free_agent.get("age");
            let position: String = normalize_position(&free_agent.get::<String, _>("position"));
            let loyalty: i64 = free_agent.get("loyalty");
            let home_region_id: Option<i64> = free_agent.try_get("home_region_id").ok();
            let region_loyalty: i64 = free_agent.try_get("region_loyalty").unwrap_or(70);
            let potential: i64 = free_agent.try_get("potential").unwrap_or(0);
            let tag: String = free_agent.try_get("tag").unwrap_or_else(|_| "NORMAL".to_string());
            let stability: i64 = free_agent.try_get("stability").unwrap_or(60);
            let calculated_market_value: i64 = free_agent.try_get("calculated_market_value").unwrap_or(0);

            // 使用完整身价（含荣誉系数）计算期望薪资
            let market_value = if calculated_market_value > 0 {
                calculated_market_value as u64
            } else {
                MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, potential as u8, &tag, &position)
            };
            let expected_salary = MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64;

            // 收集所有球队的报价
            let mut offers: Vec<TransferOffer> = Vec::new();

            for &team_id in &team_ids {
                if cache.renewal_failed_pairs.contains(&(player_id, team_id)) {
                    continue;
                }
                let balance = cache.team_balances.get(&team_id).copied().unwrap_or(0);

                // 检查转会次数限制（基础2个，空缺位置额外放宽）
                let count = team_transfer_counts.get(&team_id).copied().unwrap_or(0);
                let roster = cache.get_roster(team_id);
                let positions = ["Top", "Jug", "Mid", "Adc", "Sup"];
                let vacant_positions = positions.iter()
                    .filter(|pos| !roster.iter().any(|p| p.position == **pos))
                    .count() as i64;
                let dynamic_limit = self.config.max_transfers_per_round + vacant_positions;
                if count >= dynamic_limit {
                    continue;
                }
                if count >= self.config.max_transfers_per_window {
                    continue;
                }

                let fin_status = FinancialStatus::from_balance(balance);
                if !fin_status.can_buy() {
                    continue;
                }

                let roster_count = roster.len();

                let over_threshold = roster_count as i64 - 8;
                if over_threshold >= 5 {
                    continue;
                }

                // 检查位置需求
                let pos_count = roster.iter()
                    .filter(|r| r.position.eq_ignore_ascii_case(&position))
                    .count();

                if pos_count >= 2 {
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
                let roster = cache.get_roster(team_id);
                let team_rank = cache.get_composite_rank(team_id);

                // 计算匹配度和报价
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

                let match_score_threshold = if roster_count < 7 {
                    35.0
                } else if roster_count <= 8 {
                    50.0
                } else {
                    60.0
                };

                if match_score < match_score_threshold {
                    continue;
                }

                let salary_multiplier = {
                    let base_mult = if weights.star_chasing > 0.7 { 1.15 }
                        else if weights.star_chasing > 0.4 { 1.05 }
                        else if weights.bargain_hunting > 0.7 { 0.82 }
                        else if weights.bargain_hunting > 0.4 { 0.90 }
                        else { 0.95 };
                    // 加入 ±8% 随机波动
                    let random_factor = 0.92 + rng.gen::<f64>() * 0.16;
                    base_mult * random_factor
                };
                let is_bench_signing = pos_count == 1;

                let offered_salary = {
                    let base = (expected_salary as f64 * salary_multiplier) as i64;
                    if is_bench_signing { (base as f64 * 0.85) as i64 } else { base }
                };
                let contract_years = {
                    let base: i64 = if age <= 22 { 3 } else if age <= 25 { 2 } else if age <= 28 { 2 } else { 1 };
                    let personality_adj: i64 = if weights.long_term_focus > 0.7 { 1 } else if weights.short_term_focus > 0.7 { -1 } else { 0 };
                    let random_adj: i64 = if rng.gen::<f64>() < 0.3 { 1 } else if rng.gen::<f64>() < 0.25 { -1 } else { 0 };
                    let max_years = if is_bench_signing { 2 } else { 4 };
                    (base + personality_adj + random_adj).clamp(1, max_years)
                };
                let target_region_id = cache.team_region_ids.get(&team_id).copied().flatten();

                let bonus_ratio = if weights.star_chasing > 0.7 { 0.35 }
                    else if weights.bargain_hunting > 0.7 { 0.15 }
                    else { 0.25 };

                offers.push(TransferOffer {
                    team_id,
                    player_id,
                    offered_salary,
                    contract_years,
                    transfer_fee: 0,
                    signing_bonus: (offered_salary as f64 * bonus_ratio) as i64,
                    match_score,
                    priority: match_score,
                    target_region_id,
                });
            }

            if offers.is_empty() {
                continue;
            }

            // 按匹配度排序
            offers.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));

            // 市场竞争效应：多个球队竞争时，选手提高薪资期望基准
            let offer_count = offers.len();
            let market_premium = if offer_count >= 2 {
                1.0 + ((offer_count as f64 - 1.0) * 0.05).min(0.25)
            } else {
                1.0
            };
            let adjusted_expected_salary = (expected_salary as f64 * market_premium) as i64;

            // 对所有 offers 计算 willingness，收集竞价数据
            struct BidRecord {
                offer_idx: usize,
                willingness: f64,
                team_name: String,
                target_region_id: Option<i64>,
            }
            let mut bid_records: Vec<BidRecord> = Vec::new();

            for (idx, offer) in offers.iter().enumerate() {
                let target_roster = cache.get_roster(offer.team_id);
                let target_team_rank = cache.get_composite_rank(offer.team_id);
                let target_team_reputation = cache.get_team_reputation(offer.team_id);
                let willingness = self.calculate_willingness(
                    ability as u8, loyalty as u8, age as u8,
                    offer.offered_salary, adjusted_expected_salary,
                    home_region_id, offer.target_region_id, region_loyalty,
                    target_team_rank, target_team_reputation,
                    &target_roster, &position,
                    cache.get_player_stats(player_id),
                    &mut rng,
                );
                let willingness = (willingness + 15.0).min(100.0);
                let team_name = cache.get_team_name(offer.team_id);
                bid_records.push(BidRecord {
                    offer_idx: idx,
                    willingness,
                    team_name,
                    target_region_id: offer.target_region_id,
                });
            }

            // 选出最佳报价：选手选择意愿最高的队伍（自由球员有选择权）
            // 按 willingness 降序排列，选手优先去最想去的队伍
            bid_records.sort_by(|a, b| b.willingness.partial_cmp(&a.willingness).unwrap_or(std::cmp::Ordering::Equal));
            let winner_idx = bid_records.iter()
                .find(|r| r.willingness >= 40.0)
                .map(|r| r.offer_idx);

            // 写入所有竞价记录
            for record in &bid_records {
                let offer = &offers[record.offer_idx];
                let is_winner = Some(record.offer_idx) == winner_idx;
                let reject_reason = if is_winner {
                    None
                } else if record.willingness < 40.0 {
                    Some("willingness_too_low")
                } else {
                    Some("outbid")
                };
                let _ = Self::insert_bid(
                    pool, window_id, 4,
                    player_id, &game_id, ability, age, &position,
                    None, None,
                    offer.team_id, &record.team_name, record.target_region_id,
                    offer.offered_salary, offer.contract_years, 0, offer.signing_bonus,
                    offer.match_score, record.willingness, is_winner, reject_reason,
                ).await;
            }

            // 执行签约（如果有赢家）
            let best_offer = winner_idx.map(|idx| &offers[idx]);

            if let Some(offer) = best_offer {
                let to_team_id = offer.team_id;
                let to_team_name = cache.get_team_name(to_team_id);
                let to_roster = cache.get_roster(to_team_id);
                let signing_as_bench = to_roster.iter().any(|p| p.position.eq_ignore_ascii_case(&position));
                let new_contract_role = if signing_as_bench { "Sub" } else { "Starter" };

                let base_loyalty: i64 = 55;
                let base_satisfaction: i64 = 65;
                let youth_bonus: i64 = if age <= 23 { 5 } else { 0 };
                let new_loyalty = (base_loyalty + youth_bonus).min(100);
                let new_satisfaction = (base_satisfaction + youth_bonus).min(100);

                sqlx::query(
                    "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = ?, satisfaction = ?, join_season = ?, contract_role = ?, is_starter = ? WHERE id = ?"
                )
                .bind(to_team_id)
                .bind(offer.offered_salary)
                .bind(season_id + offer.contract_years)
                .bind(new_loyalty)
                .bind(new_satisfaction)
                .bind(season_id)
                .bind(new_contract_role)
                .bind(!signing_as_bench)
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("签约失败: {}", e))?;

                // 扣除球队资金（签约奖金，不超过当前余额）
                let current_balance = cache.team_balances.get(&to_team_id).copied().unwrap_or(0);
                let actual_bonus = offer.signing_bonus.min(current_balance.max(0));
                if actual_bonus > 0 {
                    sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
                        .bind(actual_bonus)
                        .bind(to_team_id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("扣除资金失败: {}", e))?;
                }

                // 记录财务交易：签约奖金支出
                Self::record_financial_transaction(
                    pool, save_id, season_id, to_team_id,
                    "TransferOut",
                    -(actual_bonus),
                    &format!("自由球员签约: {}", game_id),
                    player_id,
                ).await?;

                // 更新缓存
                cache.update_balance(to_team_id, -actual_bonus);
                // 将自由球员添加到目标队伍缓存
                let new_player = CachedPlayer {
                    id: player_id,
                    game_id: game_id.clone(),
                    ability,
                    potential: free_agent.try_get("potential").unwrap_or(0),
                    age,
                    salary: offer.offered_salary,
                    loyalty: new_loyalty,
                    satisfaction: new_satisfaction,
                    position: position.clone(),
                    tag: free_agent.try_get("tag").unwrap_or_else(|_| "NORMAL".to_string()),
                    team_id: Some(to_team_id),
                    is_starter: !signing_as_bench,
                    home_region_id,
                    region_loyalty,
                    contract_end_season: Some(season_id + offer.contract_years),
                    status: "Active".to_string(),
                    stability: free_agent.try_get("stability").unwrap_or(60),
                    growth_accumulator: free_agent.try_get("growth_accumulator").unwrap_or(0.0),
                    contract_role: new_contract_role.to_string(),
                    season_games_played: 0,
                    season_games_total: 0,
                };
                cache.team_rosters.entry(to_team_id).or_default().push(new_player);

                *team_transfer_counts.entry(to_team_id).or_insert(0) += 1;

                // 记录合同历史
                Self::insert_contract(pool, save_id, player_id, to_team_id, "FREE_AGENT", offer.offered_salary * offer.contract_years, offer.contract_years, season_id, 0, offer.signing_bonus).await?;

                let event = self.record_event(
                    pool, window_id, 4,
                    TransferEventType::FreeAgentSigning,
                    EventLevel::from_ability_and_fee(ability as u8, 0),
                    player_id, &game_id, ability,
                    None, None,
                    Some(to_team_id), Some(&to_team_name),
                    0, offer.offered_salary, offer.contract_years,
                    &format!("{}以自由球员身份加入{}，年薪{}万，合同{}年 | {}岁{}位 潜力{}",
                             game_id, to_team_name, offer.offered_salary / 10000, offer.contract_years,
                             age, position, potential),
                ).await?;
                events.push(event);
            }
        }

        Ok(RoundResult {
            round: 4,
            round_name: "自由球员争夺".to_string(),
            events,
            summary: "已完成自由球员争夺".to_string(),
        })
    }

    // ============================================
    // 第5轮：有合同选手挖角
    // ============================================
}
