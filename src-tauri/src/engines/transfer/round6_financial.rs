use std::collections::{HashMap, HashSet};

use sqlx::{Pool, Row, Sqlite};

use crate::engines::financial::FinancialEngine;
use crate::engines::market_value::MarketValueEngine;
use crate::models::transfer::*;

use super::cache::TransferCache;
use super::utils::normalize_position;
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第6轮：财政调整
    // ============================================

    pub(crate) async fn execute_financial_adjustment(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let r6_start = std::time::Instant::now();
        eprintln!("[R6] 开始执行");

        // 清理之前中断执行留下的 round6 残留数据，确保幂等
        let r6_types = vec!["Salary", "Sponsorship", "OperatingCost", "FacilityMaintenance", "LuxuryTax", "Penalty"];
        for tx_type in &r6_types {
            sqlx::query(
                "DELETE FROM financial_transactions WHERE save_id = ? AND season_id = ? AND transaction_type = ?"
            )
            .bind(save_id)
            .bind(season_id)
            .bind(tx_type)
            .execute(pool)
            .await
            .ok();
        }

        // 回滚 round6 的转会事件
        sqlx::query(
            "DELETE FROM transfer_events WHERE window_id = ? AND round = 6"
        )
        .bind(window_id)
        .execute(pool)
        .await
        .ok();

        eprintln!("[R6] 清理残留完成 {:?}", r6_start.elapsed());

        let balance_rows = sqlx::query(
            "SELECT id, balance FROM teams WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("重新加载余额失败: {}", e))?;
        for row in &balance_rows {
            let tid: i64 = row.get("id");
            let bal: i64 = row.get("balance");
            cache.team_balances.insert(tid, bal);
        }

        #[derive(Clone, Copy)]
        struct TeamDetail {
            balance: i64,
            power_rating: f64,
            win_rate: f64,
            brand_value: f64,
            training_facility: i64,
        }

        let all_teams: Vec<sqlx::sqlite::SqliteRow> =
            sqlx::query("SELECT id, name FROM teams WHERE save_id = ?")
                .bind(save_id)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("查询球队失败: {}", e))?;

        let existing_transaction_rows = sqlx::query(
            "SELECT team_id, transaction_type FROM financial_transactions WHERE save_id = ? AND season_id = ?",
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询已存在财务交易失败: {}", e))?;

        let existing_transactions: HashSet<(i64, String)> = existing_transaction_rows
            .iter()
            .map(|row| {
                (
                    row.get::<i64, _>("team_id"),
                    row.get::<String, _>("transaction_type"),
                )
            })
            .collect();

        let mut existing_transactions_by_type: HashMap<String, HashSet<i64>> = HashMap::new();
        for (team_id, tx_type) in &existing_transactions {
            existing_transactions_by_type
                .entry(tx_type.clone())
                .or_default()
                .insert(*team_id);
        }

        let has_existing_transaction = |team_id: i64, tx_type: &str| -> bool {
            existing_transactions_by_type
                .get(tx_type)
                .map(|team_ids| team_ids.contains(&team_id))
                .unwrap_or(false)
        };

        let annual_salary_rows = sqlx::query(
            r#"SELECT p.team_id, COALESCE(SUM(pc.annual_salary), 0) as annual_salary
               FROM player_contracts pc
               JOIN players p ON pc.player_id = p.id
               WHERE p.save_id = ? AND p.status = 'Active' AND pc.is_active = 1
               GROUP BY p.team_id"#,
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("批量查询合同年薪失败: {}", e))?;

        let mut team_annual_salaries: HashMap<i64, i64> = HashMap::new();
        for row in &annual_salary_rows {
            team_annual_salaries.insert(
                row.get::<i64, _>("team_id"),
                row.get::<i64, _>("annual_salary"),
            );
        }

        let fallback_salary_rows = sqlx::query(
            r#"SELECT team_id, COALESCE(SUM(salary), 0) as fallback_salary
               FROM players
               WHERE save_id = ? AND status = 'Active' AND team_id IS NOT NULL
               GROUP BY team_id"#,
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("批量查询fallback年薪失败: {}", e))?;

        let mut team_fallback_salaries: HashMap<i64, i64> = HashMap::new();
        for row in &fallback_salary_rows {
            team_fallback_salaries.insert(
                row.get::<i64, _>("team_id"),
                row.get::<i64, _>("fallback_salary"),
            );
        }

        let team_detail_rows = sqlx::query(
            "SELECT id, balance, power_rating, win_rate, brand_value, training_facility FROM teams WHERE save_id = ?",
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("批量查询球队详情失败: {}", e))?;

        let mut team_details: HashMap<i64, TeamDetail> = HashMap::new();
        for row in &team_detail_rows {
            let team_id: i64 = row.get("id");
            team_details.insert(
                team_id,
                TeamDetail {
                    balance: row.get("balance"),
                    power_rating: row.get("power_rating"),
                    win_rate: row.get("win_rate"),
                    brand_value: row.get("brand_value"),
                    training_facility: row.get("training_facility"),
                },
            );
        }

        let player_market_rows = sqlx::query(
            "SELECT id, calculated_market_value FROM players WHERE save_id = ? AND status = 'Active'",
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("批量查询选手身价失败: {}", e))?;

        let mut player_market_values: HashMap<i64, i64> = HashMap::new();
        for row in &player_market_rows {
            player_market_values.insert(
                row.get::<i64, _>("id"),
                row.try_get::<i64, _>("calculated_market_value").unwrap_or(0),
            );
        }

        let active_listing_rows =
            sqlx::query("SELECT player_id FROM player_listings WHERE window_id = ? AND status = 'ACTIVE'")
                .bind(window_id)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("批量查询挂牌状态失败: {}", e))?;

        let mut active_listing_player_ids: HashSet<i64> = active_listing_rows
            .iter()
            .map(|row| row.get::<i64, _>("player_id"))
            .collect();

        eprintln!("[R6] 批量预加载完成 {:?}", r6_start.elapsed());

        // ============================================
        // 1. 给所有球队发放赛季薪资
        // ============================================
        let mut salary_paid_count = 0i64;
        let mut total_salary_paid = 0i64;

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let _team_name: String = team.get("name");

            if has_existing_transaction(team_id, "Salary") {
                continue;
            }

            let contract_annual_salary = *team_annual_salaries.get(&team_id).unwrap_or(&0);

            // fallback: 如果合同表查出为0但有活跃选手，回退到旧算法（用 join_season 估算合同总年数）
            let team_annual_salary = if contract_annual_salary == 0 {
                *team_fallback_salaries.get(&team_id).unwrap_or(&0)
            } else {
                contract_annual_salary
            };

            if team_annual_salary <= 0 {
                continue;
            }

            // 从余额扣除年薪
            sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
                .bind(team_annual_salary)
                .bind(team_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("扣除薪资失败: {}", e))?;
            cache.update_balance(team_id, -team_annual_salary);

            // 记录财务交易
            sqlx::query(
                "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, 'Salary', ?, ?)"
            )
            .bind(save_id)
            .bind(team_id)
            .bind(season_id)
            .bind(-team_annual_salary)
            .bind(format!("S{}赛季薪资支出", season_id))
            .execute(pool)
            .await
            .map_err(|e| format!("记录薪资交易失败: {}", e))?;

            salary_paid_count += 1;
            total_salary_paid += team_annual_salary;
        }

        eprintln!("[R6] 薪资发放完成 {:?}, {}支队伍", r6_start.elapsed(), salary_paid_count);

        // ============================================
        // 1.5 赞助收入发放 + 运营成本扣除 + 训练设施维护费
        // ============================================
        let financial_engine = FinancialEngine::new();

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            if has_existing_transaction(team_id, "Sponsorship") {
                continue;
            }

            if let Some(team_detail) = team_details.get(&team_id).copied() {
                let balance = cache
                    .team_balances
                    .get(&team_id)
                    .copied()
                    .unwrap_or(team_detail.balance);

                let temp_team = crate::models::Team {
                    id: team_id as u64,
                    region_id: 0,
                    name: team_name.clone(),
                    short_name: None,
                    power_rating: team_detail.power_rating,
                    total_matches: 0,
                    wins: 0,
                    win_rate: team_detail.win_rate,
                    annual_points: 0,
                    cross_year_points: 0,
                    balance,
                    brand_value: team_detail.brand_value,
                };
                let sponsorship = financial_engine.calculate_sponsorship(&temp_team);
                if sponsorship > 0 {
                    if sqlx::query("UPDATE teams SET balance = balance + ? WHERE id = ? AND save_id = ?")
                        .bind(sponsorship as i64)
                        .bind(team_id)
                        .bind(save_id)
                        .execute(pool)
                        .await
                        .is_ok()
                    {
                        cache.update_balance(team_id, sponsorship as i64);
                    }

                    sqlx::query(
                        "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, ?, ?, ?)",
                    )
                    .bind(save_id)
                    .bind(team_id)
                    .bind(season_id)
                    .bind("Sponsorship")
                    .bind(sponsorship as i64)
                    .bind(format!("S{}赛季赞助收入", season_id))
                    .execute(pool)
                    .await
                    .ok();
                }

                let team_annual_salary = *team_annual_salaries.get(&team_id).unwrap_or(&0);
                let operating_cost = financial_engine.calculate_operating_cost(team_annual_salary as u64);
                if operating_cost > 0 {
                    if sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
                        .bind(operating_cost as i64)
                        .bind(team_id)
                        .bind(save_id)
                        .execute(pool)
                        .await
                        .is_ok()
                    {
                        cache.update_balance(team_id, -(operating_cost as i64));
                    }

                    sqlx::query(
                        "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, ?, ?, ?)",
                    )
                    .bind(save_id)
                    .bind(team_id)
                    .bind(season_id)
                    .bind("OperatingCost")
                    .bind(-(operating_cost as i64))
                    .bind(format!("S{}赛季运营成本", season_id))
                    .execute(pool)
                    .await
                    .ok();
                }
            }
        }

        eprintln!("[R6] 赞助+运营完成 {:?}", r6_start.elapsed());
        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            if has_existing_transaction(team_id, "FacilityMaintenance") {
                continue;
            }

            if let Some(team_detail) = team_details.get(&team_id).copied() {
                let mut facility_level = team_detail.training_facility;
                let balance = cache
                    .team_balances
                    .get(&team_id)
                    .copied()
                    .unwrap_or(team_detail.balance);

                if facility_level <= 0 {
                    continue;
                }

                let maintenance = FinancialEngine::calculate_facility_maintenance(facility_level as u32) as i64;

                if balance < maintenance {
                    // 余额不足维护 → 自动降级直到能付得起
                    let mut new_level = facility_level;
                    let mut cost = maintenance;
                    while new_level > 1 && balance < cost {
                        new_level -= 1;
                        cost = FinancialEngine::calculate_facility_maintenance(new_level as u32) as i64;
                    }

                    if new_level < facility_level {
                        log::info!(
                            "R6设施降级: {}设施{}级→{}级（余额{}万不足维护费{}万）",
                            team_name,
                            facility_level,
                            new_level,
                            balance / 10000,
                            maintenance / 10000
                        );

                        sqlx::query("UPDATE teams SET training_facility = ? WHERE id = ? AND save_id = ?")
                            .bind(new_level)
                            .bind(team_id)
                            .bind(save_id)
                            .execute(pool)
                            .await
                            .ok();

                        facility_level = new_level;
                        if let Some(detail) = team_details.get_mut(&team_id) {
                            detail.training_facility = new_level;
                        }
                    }
                }

                let actual_maintenance =
                    FinancialEngine::calculate_facility_maintenance(facility_level as u32) as i64;
                if actual_maintenance > 0 {
                    if sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
                        .bind(actual_maintenance)
                        .bind(team_id)
                        .bind(save_id)
                        .execute(pool)
                        .await
                        .is_ok()
                    {
                        cache.update_balance(team_id, -actual_maintenance);
                    }

                    sqlx::query(
                        "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, ?, ?, ?)",
                    )
                    .bind(save_id)
                    .bind(team_id)
                    .bind(season_id)
                    .bind("FacilityMaintenance")
                    .bind(-actual_maintenance)
                    .bind(format!(
                        "S{}训练设施维护费（{}级，{}万/赛季）",
                        season_id,
                        facility_level,
                        actual_maintenance / 10000
                    ))
                    .execute(pool)
                    .await
                    .ok();
                }
            }
        }

        eprintln!("[R6] 设施维护完成 {:?}", r6_start.elapsed());
        let mut luxury_tax_count = 0i64;
        let mut total_luxury_tax = 0i64;

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            if has_existing_transaction(team_id, "LuxuryTax") {
                continue;
            }

            let roster_count: i64 = cache
                .team_rosters
                .get(&team_id)
                .map(|roster| roster.len() as i64)
                .unwrap_or(0);

            let over_count = roster_count - self.config.luxury_tax_threshold;
            if over_count <= 0 {
                continue;
            }

            // 线性递增：每超出1人缴纳 luxury_tax_per_player
            let tax_amount = over_count * self.config.luxury_tax_per_player;

            // 扣除奢侈税
            sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
                .bind(tax_amount)
                .bind(team_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("扣除奢侈税失败: {}", e))?;
            cache.update_balance(team_id, -tax_amount);

            // 记录财务交易
            sqlx::query(
                "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, 'LuxuryTax', ?, ?)"
            )
            .bind(save_id)
            .bind(team_id)
            .bind(season_id)
            .bind(-tax_amount)
            .bind(format!("S{}奢侈税：阵容{}人，超出{}人，每人{}万", season_id, roster_count, over_count, self.config.luxury_tax_per_player / 10000))
            .execute(pool)
            .await
            .map_err(|e| format!("记录奢侈税交易失败: {}", e))?;

            log::info!("R6奢侈税: {}阵容{}人，超出{}人，缴税{}万", team_name, roster_count, over_count, tax_amount / 10000);

            let representative_id = cache
                .team_rosters
                .get(&team_id)
                .and_then(|roster| roster.iter().max_by_key(|p| p.ability).map(|p| p.id))
                .ok_or_else(|| "查询代表选手失败: 未找到活跃选手".to_string())?;

            let event = self.record_event(
                pool, window_id, 6,
                TransferEventType::FinancialAdjustment,
                EventLevel::B,
                representative_id, &team_name, 0,
                Some(team_id), Some(&team_name),
                None, None,
                tax_amount, 0, 0,
                &format!("{}缴纳奢侈税{}万（阵容{}人，超出{}人，每人{}万）", team_name, tax_amount / 10000, roster_count, over_count, self.config.luxury_tax_per_player / 10000),
            ).await?;
            events.push(event);

            luxury_tax_count += 1;
            total_luxury_tax += tax_amount;
        }

        eprintln!("[R6] 奢侈税完成 {:?}, {}支队伍", r6_start.elapsed(), luxury_tax_count);

        let mut release_count = 0i64;
        let mut total_release_fee = 0i64;

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            let roster_count: i64 = cache
                .team_rosters
                .get(&team_id)
                .map(|roster| roster.len() as i64)
                .unwrap_or(0);

            let over_count = roster_count - 8;
            if over_count <= 0 {
                continue;
            }

            let team_balance = cache
                .team_balances
                .get(&team_id)
                .copied()
                .or_else(|| team_details.get(&team_id).map(|d| d.balance))
                .unwrap_or(0);

            // 找出可解约的选手：非首发 + 按（能力值-潜力值培养价值）排序，最差的优先解约
            // 排除首发，按 ability ASC 排序（能力最低的优先解约）
            let mut release_candidates = cache
                .get_roster(team_id)
                .into_iter()
                .filter(|player| !player.is_starter)
                .collect::<Vec<_>>();
            release_candidates.sort_by(|a, b| a.ability.cmp(&b.ability).then_with(|| b.age.cmp(&a.age)));
            release_candidates.truncate(over_count as usize);

            let mut current_balance = team_balance;
            for candidate in &release_candidates {
                let player_id = candidate.id;
                let game_id = candidate.game_id.clone();
                let ability = candidate.ability;
                let age = candidate.age;
                let potential = candidate.potential;
                let tag = candidate.tag.clone();
                let position = normalize_position(&candidate.position);
                let calculated_market_value = player_market_values.get(&player_id).copied().unwrap_or(0);

                // 保护有培养价值的年轻选手（23岁以下 + 能力≥55）
                if age <= 23 && ability >= 55 {
                    continue;
                }

                // 计算解约金 = 身价50%
                let market_value = if calculated_market_value > 0 {
                    calculated_market_value
                } else {
                    MarketValueEngine::calculate_base_market_value(
                        ability as u8, age as u8, potential as u8, &tag, &position
                    ) as i64
                };
                let release_fee = market_value / 2;

                // 比较：解约金 vs 留着交的奢侈税（至少1个赛季）
                // 如果余额不够支付解约金，也跳过
                if release_fee > current_balance {
                    log::debug!("R6解约: {}解约金{}万超出{}余额{}万，跳过", game_id, release_fee / 10000, team_name, current_balance / 10000);
                    continue;
                }

                // 执行解约
                sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ? AND save_id = ?")
                    .bind(release_fee)
                .bind(team_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("扣除解约金失败: {}", e))?;

                current_balance -= release_fee;
                cache.update_balance(team_id, -release_fee);

                // 记录财务交易
                sqlx::query(
                    "INSERT INTO financial_transactions (save_id, team_id, season_id, transaction_type, amount, description) VALUES (?, ?, ?, 'Penalty', ?, ?)"
                )
                .bind(save_id)
                .bind(team_id)
                .bind(season_id)
                .bind(-release_fee)
                .bind(format!("解约{}，支付解约金{}万", game_id, release_fee / 10000))
                .execute(pool)
                .await
                .map_err(|e| format!("记录解约交易失败: {}", e))?;

                // 选手变为自由球员
                sqlx::query(
                    "UPDATE players SET team_id = NULL, is_starter = 0, satisfaction = MAX(satisfaction - 15, 0) WHERE id = ? AND save_id = ?"
                )
                .bind(player_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("释放选手失败: {}", e))?;

                // 合同失效
                sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
                    .bind(save_id)
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .ok();
                cache.release_player(player_id, team_id);

                let event = self.record_event(
                    pool, window_id, 6,
                    TransferEventType::PlayerRelease,
                    EventLevel::from_ability_and_fee(ability as u8, release_fee),
                    player_id, &game_id, ability,
                    Some(team_id), Some(&team_name),
                    None, None,
                    release_fee, candidate.salary, 0,
                    &format!("{}解约{}以避免奢侈税，支付解约金{}万", team_name, game_id, release_fee / 10000),
                ).await?;
                events.push(event);

                log::info!("R6解约: {}解约{}，解约金{}万", team_name, game_id, release_fee / 10000);
                release_count += 1;
                total_release_fee += release_fee;
            }
        }

        eprintln!("[R6] 解约超额完成 {:?}, {}人", r6_start.elapsed(), release_count);

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            let balance = cache
                .team_balances
                .get(&team_id)
                .copied()
                .or_else(|| team_details.get(&team_id).map(|d| d.balance))
                .unwrap_or(0);
            if balance >= 1_000_000 {
                continue;
            }

            // 找出最高薪的非核心选手
            let mut expensive_players = cache.get_roster(team_id);
            expensive_players.sort_by(|a, b| b.salary.cmp(&a.salary));
            expensive_players.truncate(2);

            for player in &expensive_players {
                let player_id = player.id;
                let game_id = player.game_id.clone();
                let ability = player.ability;
                let salary = player.salary;
                let age = player.age;

                // 检查是否已挂牌
                if active_listing_player_ids.contains(&player_id) {
                    continue;
                }

                // 挂牌出售
                let listing_price = MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", &player.position) as i64;
                let discount_price = (listing_price as f64 * 0.7) as i64; // 财务困难打折

                sqlx::query(
                    "INSERT INTO player_listings (player_id, window_id, listed_by_team_id, listing_price, min_accept_price, status) VALUES (?, ?, ?, ?, ?, 'ACTIVE')"
                )
                .bind(player_id)
                .bind(window_id)
                .bind(team_id)
                .bind(discount_price)
                .bind((discount_price as f64 * 0.6) as i64)
                .execute(pool)
                .await
                .map_err(|e| format!("财政调整挂牌失败: {}", e))?;
                active_listing_player_ids.insert(player_id);

                let event = self.record_event(
                    pool, window_id, 6,
                    TransferEventType::FinancialAdjustment,
                    EventLevel::from_ability_and_fee(ability as u8, 0),
                    player_id, &game_id, ability,
                    Some(team_id), Some(&team_name),
                    None, None,
                    discount_price, salary, 0,
                    &format!("{}因{}财务困难被折价挂牌", game_id, team_name),
                ).await?;
                events.push(event);
            }
        }

        eprintln!("[R6] 全部完成 {:?}", r6_start.elapsed());

        Ok(RoundResult {
            round: 6,
            round_name: "财政调整".to_string(),
            events,
            summary: format!(
                "已完成财政调整：{}支球队支付薪资共{}万{}{}，财务困难球队处理完成",
                salary_paid_count, total_salary_paid / 10000,
                if luxury_tax_count > 0 {
                    format!("，{}支球队缴纳奢侈税共{}万", luxury_tax_count, total_luxury_tax / 10000)
                } else {
                    String::new()
                },
                if release_count > 0 {
                    format!("，解约{}名超额选手共支付{}万", release_count, total_release_fee / 10000)
                } else {
                    String::new()
                }
            ),
        })
    }

    // ============================================
    // 第7轮：收尾补救
    // ============================================
}
