use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Sqlite};

use crate::engines::market_value::MarketValueEngine;
use crate::models::transfer::*;

use super::cache::{CachedPlayer, CachedPlayerStats, TransferCache};
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第2轮：双向评估（战队评估选手 + 选手评估战队）
    // ============================================

    pub(crate) async fn execute_bidirectional_evaluation(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // 使用缓存获取所有球队ID
        let team_ids: Vec<i64> = cache.team_names.keys().copied().collect();

        for team_id in &team_ids {
            let team_id = *team_id;
            let team_name = cache.get_team_name(team_id);
            let balance = cache.team_balances.get(&team_id).copied().unwrap_or(0);

            // 1. 执行战队评估（使用缓存）
            let team_eval = self.evaluate_team_cached(pool, save_id, window_id, team_id, &team_name, balance, season_id, cache).await?;

            // 2. 获取球队阵容并评估每个选手
            let roster = cache.get_roster(team_id);

            for player in &roster {
                // 3. 执行选手评估（使用缓存）
                let mut player_eval = self.evaluate_player_cached(
                    pool, save_id, window_id, player.id, &player.game_id,
                    team_id, &team_name, &team_eval,
                    player.ability, player.age, player.salary, player.satisfaction, player.loyalty, &player.position,
                    &roster, season_id, cache
                ).await?;

                // 3.5 涨薪谈判：薪资差距大的选手先向球队提涨薪要求
                let estimated_salary = MarketValueEngine::estimate_salary(
                    MarketValueEngine::calculate_base_market_value(player.ability as u8, player.age as u8, player.ability as u8, "NORMAL", &player.position),
                    player.ability as u8, player.age as u8,
                ) as i64;
                let salary_ratio = if estimated_salary > 0 { player.salary as f64 / estimated_salary as f64 } else { 1.0 };

                // 涨薪门槛：薪资低于市场估值 + 能力不能远低于队伍首发平均
                let starter_avg_ability = {
                    let starters: Vec<i64> = roster.iter()
                        .filter(|p| p.is_starter)
                        .map(|p| p.ability)
                        .collect();
                    if starters.is_empty() {
                        roster.iter().map(|p| p.ability).sum::<i64>() / roster.len().max(1) as i64
                    } else {
                        starters.iter().sum::<i64>() / starters.len() as i64
                    }
                };
                let ability_qualifies = player.ability >= starter_avg_ability - 5;

                if salary_ratio < 0.85 && player.salary > 0 && ability_qualifies {
                    let raise_target = (estimated_salary as f64 * 0.90) as i64;
                    let raise_amount = raise_target - player.salary;

                    // 球队决策：余额充足 + 选手能力高 → 同意
                    let team_can_afford = balance > raise_amount * 3; // 至少能承担3年涨幅
                    let player_is_valuable = player.ability >= 60;
                    let roster_is_thin = cache.get_roster(team_id).iter()
                        .filter(|p| p.position == player.position)
                        .count() <= 1;

                    let agree_prob: f64 = if team_can_afford && player_is_valuable {
                        0.85
                    } else if team_can_afford && roster_is_thin {
                        0.75
                    } else if team_can_afford {
                        0.50
                    } else if player_is_valuable {
                        0.30
                    } else {
                        0.15
                    };

                    let mut rng = rand::rngs::StdRng::from_entropy();
                    if rng.gen::<f64>() < agree_prob {
                        let new_contract_years: i64 = if player.age <= 24 { 3 } else if player.age <= 28 { 2 } else { 1 };
                        let new_end_season = season_id + new_contract_years;

                        sqlx::query("UPDATE players SET salary = ?, contract_end_season = ? WHERE id = ? AND save_id = ?")
                            .bind(raise_target)
                            .bind(new_end_season)
                            .bind(player.id)
                            .bind(save_id)
                            .execute(pool)
                            .await
                            .map_err(|e| format!("更新薪资失败: {}", e))?;

                        let new_satisfaction = (player.satisfaction + 8).min(100);
                        sqlx::query("UPDATE players SET satisfaction = ? WHERE id = ? AND save_id = ?")
                            .bind(new_satisfaction)
                            .bind(player.id)
                            .bind(save_id)
                            .execute(pool)
                            .await
                            .map_err(|e| format!("更新满意度失败: {}", e))?;

                        // 旧合同失效，插入新合同
                        sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
                            .bind(save_id).bind(player.id).execute(pool).await.ok();
                        Self::insert_contract(pool, save_id, player.id, team_id, "SALARY_RAISE", raise_target * new_contract_years, new_contract_years, season_id, 0, 0).await?;

                        cache.update_player_salary(player.id, team_id, raise_target);

                        player_eval.stay_score = (player_eval.stay_score + 20.0).clamp(0.0, 100.0);
                        if player_eval.wants_to_leave && player_eval.stay_score >= 40.0 {
                            player_eval.wants_to_leave = false;
                            player_eval.leave_reason = String::new();
                        }

                        let event = self.record_event(
                            pool, window_id, 2,
                            TransferEventType::ContractRenewal,
                            EventLevel::C,
                            player.id, &player.game_id, player.ability,
                            Some(team_id), Some(&team_name),
                            Some(team_id), Some(&team_name),
                            0, raise_target, new_contract_years,
                            &format!("[主动涨薪] {}向{}提出涨薪要求，球队同意并续约{}年，年薪调整至{}万", player.game_id, team_name, new_contract_years, raise_target / 10000),
                        ).await?;
                        events.push(event);
                    } else {
                        // 涨薪被拒
                        let new_satisfaction = (player.satisfaction as i64 - 12).max(0) as i64;
                        sqlx::query("UPDATE players SET satisfaction = ? WHERE id = ? AND save_id = ?")
                            .bind(new_satisfaction)
                            .bind(player.id)
                            .bind(save_id)
                            .execute(pool)
                            .await
                            .map_err(|e| format!("更新满意度失败: {}", e))?;

                        // 被拒后更想走
                        player_eval.stay_score = (player_eval.stay_score - 15.0).clamp(0.0, 100.0);
                        if player_eval.stay_score < 40.0 {
                            player_eval.wants_to_leave = true;
                            if player_eval.leave_reason.is_empty() {
                                player_eval.leave_reason = "涨薪要求被拒绝".to_string();
                            }
                        }

                        let event = self.record_event(
                            pool, window_id, 2,
                            TransferEventType::ContractRenewal,
                            EventLevel::C,
                            player.id, &player.game_id, player.ability,
                            Some(team_id), Some(&team_name),
                            Some(team_id), Some(&team_name),
                            0, player.salary, 0,
                            &format!("[主动涨薪] {}向{}提出涨薪要求（期望年薪{}万），但球队拒绝", player.game_id, team_name, raise_target / 10000),
                        ).await?;
                        events.push(event);
                    }
                }

                // 4. 根据评估结果决定是否挂牌
                if player_eval.should_list {
                    let listing_price = MarketValueEngine::calculate_base_market_value(player.ability as u8, player.age as u8, player.potential as u8, &player.tag, &player.position) as i64;

                    sqlx::query(
                        "INSERT INTO player_listings (player_id, window_id, listed_by_team_id, listing_price, min_accept_price, status) VALUES (?, ?, ?, ?, ?, 'ACTIVE')"
                    )
                    .bind(player.id)
                    .bind(window_id)
                    .bind(team_id)
                    .bind(listing_price)
                    .bind((listing_price as f64 * 0.8) as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("创建挂牌失败: {}", e))?;

                    let event = self.record_event(
                        pool, window_id, 2,
                        TransferEventType::PlayerListed,
                        EventLevel::from_ability_and_fee(player.ability as u8, 0),
                        player.id, &player.game_id, player.ability,
                        Some(team_id), Some(&team_name),
                        None, None,
                        listing_price, player.salary, 0,
                        &format!("{}被{}挂牌，{}，标价{}万", player.game_id, team_name, player_eval.list_reason, listing_price / 10000),
                    ).await?;
                    events.push(event);
                }

                // 5. 如果选手想离开但战队不想放人，根据忠诚度+满意度决定是否强制挂牌
                if player_eval.wants_to_leave && !player_eval.should_list {
                    // 综合分 = (忠诚度 + 满意度) / 2，越低越容易强制挂牌
                    let combined = (player.loyalty + player.satisfaction) as f64 / 2.0;
                    // 强制挂牌概率: combined<30 → 90%, 30-50 → 60%, 50-70 → 30%, 70-90 → 10%, >90 → 0%
                    let force_list_prob = if combined < 30.0 {
                        0.90
                    } else if combined < 50.0 {
                        0.60
                    } else if combined < 70.0 {
                        0.30
                    } else if combined < 90.0 {
                        0.10
                    } else {
                        0.0
                    };

                    let mut rng = rand::rngs::StdRng::from_entropy();
                    let roll: f64 = rng.gen();

                    if roll < force_list_prob {
                        // 强制挂牌：选手坚持要走，战队被迫同意
                        let listing_price = MarketValueEngine::calculate_base_market_value(player.ability as u8, player.age as u8, player.potential as u8, &player.tag, &player.position) as i64;

                        sqlx::query(
                            "INSERT INTO player_listings (player_id, window_id, listed_by_team_id, listing_price, min_accept_price, status) VALUES (?, ?, ?, ?, ?, 'ACTIVE')"
                        )
                        .bind(player.id)
                        .bind(window_id)
                        .bind(team_id)
                        .bind(listing_price)
                        .bind((listing_price as f64 * 0.7) as i64)  // 被迫挂牌，最低接受价更低(70%)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("创建强制挂牌失败: {}", e))?;

                        let event = self.record_event(
                            pool, window_id, 2,
                            TransferEventType::PlayerListed,
                            EventLevel::from_ability_and_fee(player.ability as u8, 0),
                            player.id, &player.game_id, player.ability,
                            Some(team_id), Some(&team_name),
                            None, None,
                            listing_price, player.salary, 0,
                            &format!("{}坚持要求离队，{}被迫同意挂牌，标价{}万", player.game_id, team_name, listing_price / 10000),
                        ).await?;
                        events.push(event);
                    } else {
                        // 战队拒绝放人
                        let event = self.record_event(
                            pool, window_id, 2,
                            TransferEventType::PlayerRequestTransfer,
                            EventLevel::from_ability_and_fee(player.ability as u8, 0),
                            player.id, &player.game_id, player.ability,
                            Some(team_id), Some(&team_name),
                            None, None,
                            0, player.salary, 0,
                            &format!("{}向{}提出转会申请，原因：{}，但战队拒绝放人", player.game_id, team_name, player_eval.leave_reason),
                        ).await?;
                        events.push(event);
                    }
                }
            }
        }

        Ok(RoundResult {
            round: 2,
            round_name: "双向评估".to_string(),
            events,
            summary: "已完成双向评估：战队和选手互相评估，生成挂牌和转会申请".to_string(),
        })
    }

/// 战队评估（使用缓存版本）
    pub(crate) async fn evaluate_team_cached(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        window_id: i64,
        team_id: i64,
        team_name: &str,
        balance: i64,
        season_id: i64,
        cache: &TransferCache,
    ) -> Result<TeamEvaluation, String> {
        // 使用缓存获取阵容
        let roster = cache.get_roster(team_id);
        let roster_count = roster.len() as i32;

        // 计算阵容战力和平均年龄
        let mut total_ability: f64 = 0.0;
        let mut total_age: f64 = 0.0;
        let mut total_salary: i64 = 0;

        for player in &roster {
            total_ability += player.ability as f64;
            total_age += player.age as f64;
            total_salary += player.salary;
        }

        let roster_power = if roster_count > 0 { total_ability / roster_count as f64 } else { 0.0 };
        let roster_age_avg = if roster_count > 0 { total_age / roster_count as f64 } else { 0.0 };

        // 使用缓存获取排名
        let current_rank = cache.get_composite_rank(team_id);
        let last_rank = cache.get_team_last_rank(team_id);

        let rank_change = current_rank - last_rank;
        let rank_trend = if rank_change < -2 {
            "UP"
        } else if rank_change > 2 {
            "DOWN"
        } else {
            "STABLE"
        };

        let stability_score = self.calculate_stability_score(current_rank, last_rank);

        let (strategy, urgency_level, strategy_reason) = self.determine_team_strategy(
            stability_score, current_rank, roster_power, roster_age_avg
        );

        // 保存评估结果到数据库
        let result = sqlx::query(
            r#"INSERT INTO team_season_evaluations
            (save_id, window_id, team_id, team_name, season_id,
             current_rank, last_season_rank, rank_trend, rank_change,
             roster_power, roster_age_avg, roster_salary_total, budget_remaining, roster_count,
             stability_score, urgency_level, strategy, strategy_reason)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(save_id)
        .bind(window_id)
        .bind(team_id)
        .bind(team_name)
        .bind(season_id)
        .bind(current_rank)
        .bind(last_rank)
        .bind(rank_trend)
        .bind(rank_change)
        .bind(roster_power)
        .bind(roster_age_avg)
        .bind(total_salary)
        .bind(balance)
        .bind(roster_count)
        .bind(stability_score)
        .bind(&urgency_level)
        .bind(&strategy)
        .bind(&strategy_reason)
        .execute(pool)
        .await
        .map_err(|e| format!("保存战队评估失败: {}", e))?;

        let evaluation_id = result.last_insert_rowid();

        // 生成位置需求（使用缓存版本）
        self.generate_position_needs_cached(pool, evaluation_id, &roster, &strategy, roster_power, balance).await?;

        Ok(TeamEvaluation {
            evaluation_id,
            team_id,
            current_rank,
            last_rank,
            stability_score,
            strategy: strategy.clone(),
            urgency_level,
            roster_power,
        })
    }

    /// 生成位置需求（使用缓存版本）
    pub(crate) async fn generate_position_needs_cached(
        &self,
        pool: &Pool<Sqlite>,
        evaluation_id: i64,
        roster: &[CachedPlayer],
        strategy: &str,
        roster_power: f64,
        budget: i64,
    ) -> Result<(), String> {
        let positions = ["Top", "Jug", "Mid", "Adc", "Sup"];

        for pos in &positions {
            let starter = roster.iter()
                .filter(|p| p.position == *pos)
                .max_by_key(|p| p.ability);

            let (starter_id, starter_name, starter_ability, starter_age) = match starter {
                Some(p) => (Some(p.id), Some(p.game_id.clone()), Some(p.ability), Some(p.age)),
                None => (None, None, None, None),
            };

            let (need_level, min_ability_target, reason) = match strategy {
                "DYNASTY" => ("NONE", 0, "阵容稳定无需变动".to_string()),
                "MAINTAIN" => {
                    if let Some(ability) = starter_ability {
                        if ability < 58 {
                            ("OPTIONAL", (ability + 5) as i32, format!("{}位置能力{}偏低，可考虑补强", pos, ability))
                        } else {
                            ("NONE", 0, "当前首发足够".to_string())
                        }
                    } else {
                        ("CRITICAL", 58, format!("{}位置缺少首发", pos))
                    }
                },
                "UPGRADE" => {
                    if let Some(ability) = starter_ability {
                        if ability < roster_power as i64 - 5 {
                            ("IMPORTANT", (roster_power as i32 + 5).min(72), format!("{}位置能力{}低于队伍均值", pos, ability))
                        } else if ability < 61 {
                            ("OPTIONAL", 61, format!("{}位置能力{}，可考虑升级", pos, ability))
                        } else {
                            ("NONE", 0, "当前首发足够".to_string())
                        }
                    } else {
                        ("CRITICAL", 60, format!("{}位置缺少首发", pos))
                    }
                },
                "REBUILD" => {
                    if let Some(ability) = starter_ability {
                        if ability < 58 {
                            ("CRITICAL", 80, format!("重建期需要补强{}位置", pos))
                        } else {
                            ("OPTIONAL", (ability + 5) as i32, format!("可考虑升级{}位置", pos))
                        }
                    } else {
                        ("CRITICAL", 75, format!("{}位置缺少首发", pos))
                    }
                },
                _ => ("NONE", 0, "无需求".to_string()),
            };

            if need_level != "NONE" {
                let max_salary = (budget as f64 * 0.15) as i64;
                let prefer_young = strategy == "REBUILD" || starter_age.unwrap_or(25) > 27;

                sqlx::query(
                    r#"INSERT INTO team_position_needs
                    (evaluation_id, position, current_starter_id, current_starter_name,
                     current_starter_ability, current_starter_age,
                     need_level, min_ability_target, max_salary_budget, prefer_young, reason)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
                )
                .bind(evaluation_id)
                .bind(*pos)
                .bind(starter_id)
                .bind(&starter_name)
                .bind(starter_ability)
                .bind(starter_age)
                .bind(need_level)
                .bind(min_ability_target)
                .bind(max_salary)
                .bind(prefer_young as i32)
                .bind(&reason)
                .execute(pool)
                .await
                .map_err(|e| format!("保存位置需求失败: {}", e))?;
            }

            // 替补需求：如果该位置只有1人（无替补），且不是 DYNASTY 策略
            if strategy != "DYNASTY" {
                let pos_player_count = roster.iter().filter(|p| p.position == *pos).count();
                if pos_player_count == 1 {
                    let bench_min_ability = starter_ability.unwrap_or(50) as i32 - 15;
                    let bench_max_salary = (budget as f64 * 0.06) as i64;
                    sqlx::query(
                        r#"INSERT INTO team_position_needs
                        (evaluation_id, position, current_starter_id, current_starter_name,
                         current_starter_ability, current_starter_age,
                         need_level, min_ability_target, max_salary_budget, prefer_young, reason)
                        VALUES (?, ?, ?, ?, ?, ?, 'OPTIONAL', ?, ?, 1, ?)"#
                    )
                    .bind(evaluation_id)
                    .bind(format!("{}_BENCH", pos))
                    .bind(starter_id)
                    .bind(&starter_name)
                    .bind(starter_ability)
                    .bind(starter_age)
                    .bind(bench_min_ability)
                    .bind(bench_max_salary)
                    .bind(format!("{}位置缺少替补", pos))
                    .execute(pool)
                    .await
                    .map_err(|e| format!("保存替补需求失败: {}", e))?;
                }
            }
        }

        Ok(())
    }

    /// 选手评估（使用缓存版本）
    pub(crate) async fn evaluate_player_cached(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        window_id: i64,
        player_id: i64,
        player_name: &str,
        team_id: i64,
        team_name: &str,
        team_eval: &TeamEvaluation,
        ability: i64,
        age: i64,
        salary: i64,
        satisfaction: i64,
        loyalty: i64,
        position: &str,
        roster: &[CachedPlayer],
        _season_id: i64,
        cache: &TransferCache,
    ) -> Result<PlayerEvaluation, String> {
        let mut stay_score: f64 = 60.0;

        // 1. 战队排名评分
        let team_rank_score = match team_eval.current_rank {
            1..=4 => 10.0,
            5..=8 => 5.0,
            9..=11 => -2.0,
            12..=13 => -3.0,
            _ => -5.0,
        };
        stay_score += team_rank_score;

        // 2. 战绩趋势评分
        let rank_change = team_eval.last_rank - team_eval.current_rank;
        let team_trend_score = (rank_change as f64 * 3.0).clamp(-15.0, 15.0);
        stay_score += team_trend_score;

        // 3. 队友水平评分
        let teammate_avg: f64 = roster.iter()
            .filter(|p| p.id != player_id)
            .map(|p| p.ability as f64)
            .sum::<f64>() / (roster.len() - 1).max(1) as f64;

        let teammate_score = if ability > teammate_avg as i64 + 10 {
            -5.0
        } else if ability < teammate_avg as i64 - 5 {
            10.0
        } else {
            0.0
        };
        stay_score += teammate_score;

        // 4. 薪资评分
        let estimated_salary = MarketValueEngine::estimate_salary(MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", position), ability as u8, age as u8) as i64;
        let salary_ratio = if estimated_salary > 0 { salary as f64 / estimated_salary as f64 } else { 1.0 };
        let salary_score = if salary_ratio < 0.7 {
            -12.0
        } else if salary_ratio < 0.9 {
            -6.0
        } else if salary_ratio > 1.2 {
            10.0
        } else {
            0.0
        };
        stay_score += salary_score;

        // 5. 荣誉渴望
        let has_recent_honor = cache.has_recent_honor(player_id);
        let honor_score = if has_recent_honor {
            5.0
        } else if ability >= 70 && team_eval.current_rank > 8 {
            -5.0
        } else {
            0.0
        };
        stay_score += honor_score;

        // 6. 满意度评分
        let satisfaction_score = (satisfaction as f64 - 70.0) * 0.3;
        stay_score += satisfaction_score;

        // 7. 年龄因素
        if age >= 28 && team_eval.current_rank > 8 {
            stay_score -= 12.0;
        }

        // 8. 忠诚度加成
        stay_score += (loyalty as f64 - 70.0) * 0.5;

        // 9. 数据中心统计
        let performance_score = if let Some(stats) = cache.get_player_stats(player_id) {
            if stats.games_played >= 10 {
                let impact = stats.avg_impact;
                if impact > 3.0 && team_eval.current_rank > 8 {
                    -6.0
                } else if impact > 1.5 && team_eval.current_rank > 10 {
                    -3.0
                } else if impact < -2.0 {
                    3.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };
        stay_score += performance_score;

        // 10. 国际赛表现
        let intl_score = if let Some(stats) = cache.get_player_stats(player_id) {
            if stats.international_games >= 5 {
                let intl_impact = stats.international_avg_impact;
                if intl_impact > 2.0 && team_eval.current_rank > 8 {
                    -8.0
                } else if intl_impact > 1.0 && team_eval.current_rank > 10 {
                    -4.0
                } else if intl_impact < -1.0 && team_eval.current_rank <= 6 {
                    3.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };
        stay_score += intl_score;

        // 11. 状态动量
        let momentum_score = if let Some(stats) = cache.get_player_stats(player_id) {
            let m = stats.momentum;
            if m >= 3 {
                3.0
            } else if m <= -3 {
                -5.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        stay_score += momentum_score;

        // 12. 忠诚老将保护
        if loyalty >= 80 {
            stay_score += 10.0;
        }

        // 13. 青训归属感：年轻+忠诚度不低 = 自己培养的选手不轻易走
        if age <= 24 && loyalty >= 65 {
            stay_score += 8.0;
        }

        let stay_score = stay_score.clamp(0.0, 100.0);
        let wants_to_leave = stay_score < 40.0;

        let leave_reason = if wants_to_leave {
            let mut factors: Vec<(f64, &str)> = Vec::new();

            if salary_score <= -10.0 {
                factors.push((salary_score, "薪资被严重低估"));
            } else if salary_score < 0.0 {
                factors.push((salary_score, "对薪资待遇不满"));
            }

            if team_rank_score <= -4.0 {
                factors.push((team_rank_score, "战队战绩太差"));
            } else if team_rank_score < 0.0 {
                factors.push((team_rank_score, "战队缺乏竞争力"));
            }

            if team_trend_score <= -9.0 {
                factors.push((team_trend_score, "战队成绩大幅下滑"));
            } else if team_trend_score < -3.0 {
                factors.push((team_trend_score, "战队近期状态下滑"));
            }

            if teammate_score < 0.0 {
                factors.push((teammate_score, "队友水平跟不上"));
            }

            if honor_score < 0.0 {
                factors.push((honor_score, "渴望在强队证明自己"));
            }

            if satisfaction_score <= -6.0 {
                factors.push((satisfaction_score, "对球队管理非常不满"));
            } else if satisfaction_score < -2.0 {
                factors.push((satisfaction_score, "对球队现状不太满意"));
            }

            if age >= 28 && team_eval.current_rank > 8 {
                factors.push((-12.0, "想去强队冲击荣誉"));
            }

            let loyalty_contribution = (loyalty as f64 - 70.0) * 0.5;
            if loyalty_contribution <= -5.0 {
                factors.push((loyalty_contribution, "缺乏归属感"));
            }

            if performance_score <= -5.0 {
                factors.push((performance_score, "个人表现出色但队伍拖后腿"));
            } else if performance_score < 0.0 {
                factors.push((performance_score, "觉得自己能力在队伍中被浪费"));
            }

            if intl_score <= -6.0 {
                factors.push((intl_score, "国际赛表现出色，不甘于弱队"));
            } else if intl_score < 0.0 {
                factors.push((intl_score, "大赛经验丰富，渴望更高舞台"));
            }

            if momentum_score < 0.0 {
                factors.push((momentum_score, "近期状态低迷想换环境"));
            }

            factors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            factors.first().map_or("对现状不满意".to_string(), |(_, reason)| reason.to_string())
        } else {
            "".to_string()
        };

        // 战队评估选手是否应该挂牌（使用缓存版本）
        let player_stats = cache.get_player_stats(player_id);
        let (should_list, list_reason, protect_reason) = self.evaluate_player_for_listing_cached(
            ability, age, salary, team_eval, has_recent_honor, position, roster, player_stats
        );

        // 保存选手评估到数据库
        sqlx::query(
            r#"INSERT INTO player_season_evaluations
            (save_id, window_id, player_id, player_name, team_id, team_name,
             ability, age, salary, satisfaction, loyalty,
             team_rank_score, team_trend_score, teammate_score, salary_score, honor_score, satisfaction_score,
             stay_score, wants_to_leave, leave_reason,
             estimated_market_salary, salary_gap)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(save_id)
        .bind(window_id)
        .bind(player_id)
        .bind(player_name)
        .bind(team_id)
        .bind(team_name)
        .bind(ability)
        .bind(age)
        .bind(salary)
        .bind(satisfaction)
        .bind(loyalty)
        .bind(team_rank_score)
        .bind(team_trend_score)
        .bind(teammate_score)
        .bind(salary_score)
        .bind(honor_score)
        .bind(satisfaction_score)
        .bind(stay_score)
        .bind(wants_to_leave as i32)
        .bind(&leave_reason)
        .bind(estimated_salary)
        .bind(estimated_salary - salary)
        .execute(pool)
        .await
        .map_err(|e| format!("保存选手评估失败: {}", e))?;

        // 保存挂牌评估
        let team_eval_id = team_eval.evaluation_id;
        sqlx::query(
            r#"INSERT INTO team_listing_evaluations
            (evaluation_id, player_id, player_name, position,
             ability, age, salary, has_recent_honor, season_influence_rank,
             should_list, list_reason, protect_reason, suggested_price)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(team_eval_id)
        .bind(player_id)
        .bind(player_name)
        .bind(position)
        .bind(ability)
        .bind(age)
        .bind(salary)
        .bind(has_recent_honor as i32)
        .bind(0)
        .bind(should_list as i32)
        .bind(&list_reason)
        .bind(&protect_reason)
        .bind(MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", position) as i64)
        .execute(pool)
        .await
        .map_err(|e| format!("保存挂牌评估失败: {}", e))?;

        Ok(PlayerEvaluation {
            player_id,
            stay_score,
            wants_to_leave,
            leave_reason,
            should_list,
            list_reason,
        })
    }

    /// 评估选手是否应该被挂牌（使用缓存版本）
    pub(crate) fn evaluate_player_for_listing_cached(
        &self,
        ability: i64,
        age: i64,
        salary: i64,
        team_eval: &TeamEvaluation,
        has_recent_honor: bool,
        _position: &str,
        roster: &[CachedPlayer],
        player_stats: Option<&CachedPlayerStats>,
    ) -> (bool, String, String) {
        if has_recent_honor && ability >= 58 {
            return (false, "".to_string(), "近2赛季有荣誉".to_string());
        }

        if team_eval.strategy == "DYNASTY" {
            if ability < 60 || age >= 34 {
                return (true, "能力过低或年龄过大".to_string(), "".to_string());
            }
            return (false, "".to_string(), "战队处于王朝期".to_string());
        }

        let has_strong_performance = player_stats
            .map(|s| s.games_played >= 10 && s.avg_performance >= ability as f64 - 2.0 && s.avg_impact > 0.5)
            .unwrap_or(false);

        let has_mvp_presence = player_stats
            .map(|s| s.total_mvp_count >= 3)
            .unwrap_or(false);

        let has_intl_impact = player_stats
            .map(|s| s.international_games >= 5 && s.international_avg_impact > 1.0)
            .unwrap_or(false);

        if has_mvp_presence && ability >= 60 {
            return (false, "".to_string(), "赛季MVP表现突出".to_string());
        }

        if has_intl_impact && ability >= 62 {
            return (false, "".to_string(), "国际赛表现出色".to_string());
        }

        if ability >= team_eval.roster_power as i64 + 3 && ability >= 65 {
            return (false, "".to_string(), "核心选手".to_string());
        }

        if has_strong_performance && ability >= 60 {
            return (false, "".to_string(), "赛季表现优秀".to_string());
        }

        if team_eval.strategy == "MAINTAIN" {
            if ability < 54 || (age >= 32 && ability < 58) {
                return (true, "能力不足".to_string(), "".to_string());
            }
            return (false, "".to_string(), "阵容稳定".to_string());
        }

        let estimated_salary = MarketValueEngine::estimate_salary(
            MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", _position),
            ability as u8, age as u8,
        ) as i64;
        let overpaid_ratio = if estimated_salary > 0 { salary as f64 / estimated_salary as f64 } else { 1.0 };
        if overpaid_ratio > 1.8 && salary > 150_0000 && !has_strong_performance && !has_mvp_presence && !has_intl_impact {
            return (true, "高薪低能".to_string(), "".to_string());
        }

        if age >= 32 && ability < 65 {
            return (true, "年龄偏大且能力一般".to_string(), "".to_string());
        }

        let roster_count = roster.len() as i32;

        if roster_count >= 7 && ability < team_eval.roster_power as i64 - 5 {
            return (true, "能力低于队伍均值".to_string(), "".to_string());
        }

        if roster_count > 8 {
            let best_at_pos = roster.iter()
                .filter(|p| p.position == _position)
                .map(|p| p.ability)
                .max()
                .unwrap_or(0);
            if ability < best_at_pos {
                let has_youth_value = age <= 23 && ability >= 55;
                if !has_youth_value {
                    return (true, format!("阵容超额({}人)，非首发且无培养价值", roster_count), "".to_string());
                }
            }
        }

        if ability < 51 {
            return (true, "能力过低".to_string(), "".to_string());
        }

        let same_pos_players: Vec<&CachedPlayer> = roster.iter()
            .filter(|p| p.position == _position)
            .collect();
        if same_pos_players.len() >= 3 {
            let min_ability_at_pos = same_pos_players.iter().map(|p| p.ability).min().unwrap_or(0);
            if ability == min_ability_at_pos {
                return (true, format!("{}位置已有{}人，能力最低被挂牌", _position, same_pos_players.len()), "".to_string());
            }
        }

        let over_threshold = roster_count as i64 - self.config.luxury_tax_threshold;
        if over_threshold > 0 {
            let is_starter_level = roster.iter()
                .filter(|p| p.position == _position)
                .any(|p| p.ability <= ability && p.is_starter);

            if !is_starter_level {
                let has_youth_value = age <= 23 && ability >= 55;
                if !has_youth_value {
                    return (true, format!("阵容超额({}人)，非首发且无培养价值", roster_count), "".to_string());
                }
            }
        }

        (false, "".to_string(), "综合评估通过".to_string())
    }

    // ============================================
    // 第4轮：自由球员争夺
    // ============================================
}
