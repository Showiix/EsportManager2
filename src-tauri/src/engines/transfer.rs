//! 转会系统引擎
//!
//! 实现完整的8轮转会流程

use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};

use crate::models::transfer::*;
use crate::models::player::Position;
use crate::models::team::FinancialStatus;

/// 转会引擎
pub struct TransferEngine {
    config: TransferConfig,
}

impl Default for TransferEngine {
    fn default() -> Self {
        Self {
            config: TransferConfig::default(),
        }
    }
}

impl TransferEngine {
    pub fn new() -> Self {
        Self::default()
    }

    // ============================================
    // 主流程
    // ============================================

    /// 开始转会期
    pub async fn start_transfer_window(
        &self,
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<TransferWindowResponse, String> {
        // 创建转会期记录
        let result = sqlx::query(
            "INSERT INTO transfer_windows (save_id, season_id, status, current_round) VALUES (?, ?, 'IN_PROGRESS', 0)"
        )
        .bind(save_id)
        .bind(season_id)
        .execute(pool)
        .await
        .map_err(|e| format!("创建转会期失败: {}", e))?;

        let window_id = result.last_insert_rowid();

        // 初始化所有球队的AI性格配置（如果不存在）
        self.init_team_personalities(pool, save_id).await?;

        Ok(TransferWindowResponse {
            window_id,
            current_round: 0,
            status: "IN_PROGRESS".to_string(),
            season_id,
        })
    }

    /// 执行单轮转会
    pub async fn execute_round(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        round: i64,
    ) -> Result<RoundResult, String> {
        let window = self.get_window(pool, window_id).await?;
        let save_id = &window.save_id;

        let result = match round {
            1 => self.execute_season_settlement(pool, window_id, save_id, window.season_id).await?,
            2 => self.execute_renewal_negotiations(pool, window_id, save_id, window.season_id).await?,
            3 => self.execute_bidirectional_evaluation(pool, window_id, save_id, window.season_id).await?,
            4 => self.execute_free_agent_bidding(pool, window_id, save_id, window.season_id).await?,
            5 => self.execute_contracted_player_transfer(pool, window_id, save_id, window.season_id).await?,
            6 => self.execute_financial_adjustment(pool, window_id, save_id, window.season_id).await?,
            7 => self.execute_final_remedy(pool, window_id, save_id, window.season_id).await?,
            8 => self.execute_draft_pick_auction_round(pool, window_id, save_id, window.season_id).await?,
            _ => return Err(format!("无效轮次: {}", round)),
        };

        // 更新转会期轮次
        let new_status = if round >= self.config.max_rounds { "COMPLETED" } else { "IN_PROGRESS" };
        sqlx::query("UPDATE transfer_windows SET current_round = ?, status = ? WHERE id = ?")
            .bind(round)
            .bind(new_status)
            .execute(pool)
            .await
            .map_err(|e| format!("更新转会期状态失败: {}", e))?;

        if round >= self.config.max_rounds {
            sqlx::query("UPDATE transfer_windows SET completed_at = datetime('now') WHERE id = ?")
                .bind(window_id)
                .execute(pool)
                .await
                .map_err(|e| format!("更新转会期完成时间失败: {}", e))?;
        }

        Ok(result)
    }

    /// 快进模式
    pub async fn fast_forward(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        from_round: i64,
    ) -> Result<FastForwardResponse, String> {
        let mut rounds = Vec::new();
        let mut total_events = 0i64;

        for round in from_round..=self.config.max_rounds {
            let result = self.execute_round(pool, window_id, round).await?;
            total_events += result.events.len() as i64;
            rounds.push(result);
        }

        Ok(FastForwardResponse {
            completed_rounds: rounds.len() as i64,
            total_events,
            rounds,
        })
    }

    // ============================================
    // 第1轮：赛季结算
    // ============================================

    async fn execute_season_settlement(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        _season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // 获取所有活跃选手
        let players = self.get_active_players(pool, save_id).await?;

        for player in &players {
            let player_id: i64 = player.get("id");
            let age: i64 = player.get("age");
            let ability: i64 = player.get("ability");
            let potential: i64 = player.get("potential");
            let tag: String = player.get("tag");
            let game_id: String = player.get("game_id");
            let team_id: Option<i64> = player.try_get("team_id").ok();

            let new_age = age + 1;

            // 能力增长
            let growth = match tag.as_str() {
                "GENIUS" => 3i64,
                "NORMAL" => 2,
                _ => 1, // ORDINARY
            };

            let new_ability = if new_age <= 30 && ability < potential {
                (ability + growth).min(potential).min(100)
            } else if new_age > 30 {
                // 30岁以上开始衰退
                (ability - 1).max(50)
            } else {
                ability
            };

            // 更新选手
            sqlx::query(
                "UPDATE players SET age = ?, ability = ? WHERE id = ? AND save_id = ?"
            )
            .bind(new_age)
            .bind(new_ability)
            .bind(player_id)
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新选手年龄/能力失败: {}", e))?;

            // 退役检查
            if new_age >= 35 && new_ability < 65 {
                sqlx::query(
                    "UPDATE players SET status = 'RETIRED', team_id = NULL WHERE id = ? AND save_id = ?"
                )
                .bind(player_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("更新退役状态失败: {}", e))?;

                let from_team_name = if let Some(tid) = team_id {
                    self.get_team_name(pool, tid).await.unwrap_or_default()
                } else {
                    String::new()
                };

                let event = self.record_event(
                    pool, window_id, 1,
                    TransferEventType::PlayerRetirement,
                    EventLevel::from_ability_and_fee(new_ability as u8, 0),
                    player_id, &game_id, new_ability as i64,
                    team_id, if from_team_name.is_empty() { None } else { Some(&from_team_name) },
                    None, None,
                    0, 0, 0,
                    &format!("{}岁退役，能力值{}", new_age, new_ability),
                ).await?;
                events.push(event);
            }

            // 合同年限减少（通过 contract_end_season 管理）
        }

        // 更新所有球队战力
        self.recalculate_team_powers(pool, save_id).await?;

        Ok(RoundResult {
            round: 1,
            round_name: "赛季结算".to_string(),
            events,
            summary: "已完成赛季结算：选手年龄+1、能力值更新、退役处理".to_string(),
        })
    }

    // ============================================
    // 第2轮：续约谈判
    // ============================================

    async fn execute_renewal_negotiations(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // 获取合同即将到期的选手（contract_end_season = 当前赛季）
        let expiring_players: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT p.id, p.game_id, p.ability, p.salary, p.loyalty, p.satisfaction,
                      p.team_id, p.age, p.potential, p.tag, t.name as team_name
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

            // 续约谈判逻辑
            let loyalty_bonus = loyalty as f64 / 100.0;
            let satisfaction_bonus = satisfaction as f64 / 100.0;
            let renewal_chance = (loyalty_bonus * 0.4 + satisfaction_bonus * 0.4 + 0.2).min(1.0);

            // 最多3轮谈判
            let mut renewed = false;
            let expected_salary = self.calculate_expected_salary(ability as u8, age as u8);

            for _negotiation_round in 0..self.config.negotiation_max_rounds {
                let roll: f64 = rng.gen();
                if roll < renewal_chance {
                    // 续约成功
                    let new_contract_years = if age <= 24 { 3 } else if age <= 28 { 2 } else { 1 };
                    let new_salary = (expected_salary as f64 * 0.95) as i64; // 续约有小折扣

                    sqlx::query(
                        "UPDATE players SET salary = ?, contract_end_season = ?, loyalty = MIN(loyalty + 5, 100) WHERE id = ?"
                    )
                    .bind(new_salary)
                    .bind(season_id + new_contract_years)
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("续约更新失败: {}", e))?;

                    let event = self.record_event(
                        pool, window_id, 2,
                        TransferEventType::ContractRenewal,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id, &game_id, ability,
                        Some(team_id), Some(&team_name),
                        Some(team_id), Some(&team_name),
                        0, new_salary, new_contract_years,
                        &format!("续约{}年，年薪{}万", new_contract_years, new_salary / 10000),
                    ).await?;
                    events.push(event);
                    renewed = true;
                    break;
                }
            }

            if !renewed {
                // 续约失败，成为自由球员
                sqlx::query(
                    "UPDATE players SET team_id = NULL, satisfaction = MAX(satisfaction - 10, 0) WHERE id = ?"
                )
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("释放球员失败: {}", e))?;

                let event = self.record_event(
                    pool, window_id, 2,
                    TransferEventType::ContractTermination,
                    EventLevel::from_ability_and_fee(ability as u8, 0),
                    player_id, &game_id, ability,
                    Some(team_id), Some(&team_name),
                    None, None,
                    0, salary, 0,
                    &format!("续约谈判失败，{}成为自由球员", game_id),
                ).await?;
                events.push(event);
            }
        }

        Ok(RoundResult {
            round: 2,
            round_name: "续约谈判".to_string(),
            events,
            summary: "已完成续约谈判：处理到期合同选手".to_string(),
        })
    }

    // ============================================
    // 第3轮：双向评估
    // ============================================

    async fn execute_bidirectional_evaluation(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        _season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // 获取所有球队
        let teams = self.get_all_teams(pool, save_id).await?;

        for team in &teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");
            let balance: i64 = team.get("balance");

            // 获取球队阵容
            let roster = self.get_team_roster(pool, save_id, team_id).await?;
            let roster_count = roster.len() as i32;

            // 分析需求
            let analysis = self.analyze_team_needs(&roster, team_id, &team_name, balance);

            // 生成挂牌名单（高薪低能、年龄过大、位置过剩）
            for player in &roster {
                let player_id: i64 = player.get("id");
                let game_id: String = player.get("game_id");
                let ability: i64 = player.get("ability");
                let salary: i64 = player.get("salary");
                let age: i64 = player.get("age");

                // 性价比检查
                let value_ratio = if salary > 0 { ability as f64 / (salary as f64 / 10000.0) } else { 100.0 };
                let should_list = (value_ratio < 0.04 && salary > 200_0000)  // 高薪低能
                    || (age >= 32 && ability < 75)  // 年龄过大
                    || (roster_count > 7 && ability < analysis.power_rating as i64 - 10);  // 实力差距大

                if should_list {
                    let listing_price = self.calculate_market_value_simple(ability as u8, age as u8);

                    sqlx::query(
                        "INSERT INTO player_listings (player_id, window_id, listed_by_team_id, listing_price, min_accept_price, status) VALUES (?, ?, ?, ?, ?, 'ACTIVE')"
                    )
                    .bind(player_id)
                    .bind(window_id)
                    .bind(team_id)
                    .bind(listing_price)
                    .bind((listing_price as f64 * 0.8) as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("创建挂牌失败: {}", e))?;

                    let event = self.record_event(
                        pool, window_id, 3,
                        TransferEventType::PlayerListed,
                        EventLevel::from_ability_and_fee(ability as u8, 0),
                        player_id, &game_id, ability,
                        Some(team_id), Some(&team_name),
                        None, None,
                        listing_price, salary, 0,
                        &format!("{}被{}挂牌，标价{}万", game_id, team_name, listing_price / 10000),
                    ).await?;
                    events.push(event);
                }
            }
        }

        Ok(RoundResult {
            round: 3,
            round_name: "双向评估".to_string(),
            events,
            summary: "已完成双向评估：生成挂牌名单".to_string(),
        })
    }

    // ============================================
    // 第4轮：自由球员争夺
    // ============================================

    async fn execute_free_agent_bidding(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // 获取所有自由球员
        let free_agents: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT id, game_id, ability, salary, age, position, loyalty, potential, tag
               FROM players
               WHERE save_id = ? AND status = 'Active' AND team_id IS NULL
               ORDER BY ability DESC"#
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询自由球员失败: {}", e))?;

        // 获取所有球队及其需求
        let teams = self.get_all_teams(pool, save_id).await?;
        let mut team_transfer_counts: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();

        for free_agent in &free_agents {
            let player_id: i64 = free_agent.get("id");
            let game_id: String = free_agent.get("game_id");
            let ability: i64 = free_agent.get("ability");
            let age: i64 = free_agent.get("age");
            let position: String = free_agent.get("position");
            let loyalty: i64 = free_agent.get("loyalty");

            let expected_salary = self.calculate_expected_salary(ability as u8, age as u8);

            // 收集所有球队的报价
            let mut offers: Vec<TransferOffer> = Vec::new();

            for team in &teams {
                let team_id: i64 = team.get("id");
                let balance: i64 = team.get("balance");

                // 检查转会次数限制
                let count = team_transfer_counts.get(&team_id).copied().unwrap_or(0);
                if count >= self.config.max_transfers_per_round {
                    continue;
                }

                // 检查财务状况
                let fin_status = FinancialStatus::from_balance(balance);
                if !fin_status.can_buy() {
                    continue;
                }

                // 获取球队阵容
                let roster = self.get_team_roster(pool, save_id, team_id).await?;
                let roster_count = roster.len();

                if roster_count >= 10 {
                    continue; // 阵容已满
                }

                // 检查位置需求
                let pos_count = roster.iter()
                    .filter(|r| {
                        let p: String = r.get("position");
                        p == position
                    })
                    .count();

                if pos_count >= 2 {
                    continue; // 该位置已有2人
                }

                // 获取AI性格
                let personality = self.get_team_personality_config(pool, team_id).await;
                let weights = personality.as_ref()
                    .map(|p| p.get_weights())
                    .unwrap_or_default();

                // 计算匹配度和报价
                let match_score = self.calculate_match_score(
                    ability as u8, age as u8, &position, &weights, balance,
                );

                if match_score < 40.0 {
                    continue;
                }

                let salary_multiplier = if weights.star_chasing > 0.7 { 1.1 } else if weights.bargain_hunting > 0.7 { 0.85 } else { 1.0 };
                let offered_salary = (expected_salary as f64 * salary_multiplier) as i64;
                let contract_years = if age <= 24 && weights.long_term_focus > 0.5 { 3 } else if age <= 28 { 2 } else { 1 };

                offers.push(TransferOffer {
                    team_id,
                    player_id,
                    offered_salary,
                    contract_years,
                    transfer_fee: 0,
                    signing_bonus: offered_salary / 4,
                    match_score,
                    priority: match_score,
                });
            }

            if offers.is_empty() {
                continue;
            }

            // 按匹配度排序，选择最佳报价
            offers.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));

            // 球员选择报价（考虑薪资和球队实力）
            let mut best_offer: Option<&TransferOffer> = None;
            for offer in &offers {
                let willingness = self.calculate_willingness(
                    ability as u8, loyalty as u8, age as u8,
                    offer.offered_salary, expected_salary,
                    &mut rng,
                );
                if willingness >= 40.0 {
                    best_offer = Some(offer);
                    break;
                }
            }

            if let Some(offer) = best_offer {
                let to_team_id = offer.team_id;
                let to_team_name = self.get_team_name(pool, to_team_id).await.unwrap_or_default();

                // 执行签约
                sqlx::query(
                    "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = 50, satisfaction = 60 WHERE id = ?"
                )
                .bind(to_team_id)
                .bind(offer.offered_salary)
                .bind(season_id + offer.contract_years)
                .bind(player_id)
                .execute(pool)
                .await
                .map_err(|e| format!("签约失败: {}", e))?;

                // 扣除球队资金（签约奖金）
                sqlx::query("UPDATE teams SET balance = balance - ? WHERE id = ?")
                    .bind(offer.signing_bonus)
                    .bind(to_team_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("扣除资金失败: {}", e))?;

                *team_transfer_counts.entry(to_team_id).or_insert(0) += 1;

                let event = self.record_event(
                    pool, window_id, 4,
                    TransferEventType::FreeAgentSigning,
                    EventLevel::from_ability_and_fee(ability as u8, 0),
                    player_id, &game_id, ability,
                    None, None,
                    Some(to_team_id), Some(&to_team_name),
                    0, offer.offered_salary, offer.contract_years,
                    &format!("{}以自由球员身份加入{}，年薪{}万，合同{}年",
                             game_id, to_team_name, offer.offered_salary / 10000, offer.contract_years),
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

    async fn execute_contracted_player_transfer(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // 获取所有挂牌且尚未售出的选手
        let listings: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT pl.id as listing_id, pl.player_id, pl.listed_by_team_id, pl.listing_price, pl.min_accept_price,
                      p.game_id, p.ability, p.age, p.position, p.salary, p.loyalty,
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

        let teams = self.get_all_teams(pool, save_id).await?;

        for listing in &listings {
            let listing_id: i64 = listing.get("listing_id");
            let player_id: i64 = listing.get("player_id");
            let from_team_id: i64 = listing.get("listed_by_team_id");
            let listing_price: i64 = listing.get("listing_price");
            let min_price: i64 = listing.get("min_accept_price");
            let game_id: String = listing.get("game_id");
            let ability: i64 = listing.get("ability");
            let age: i64 = listing.get("age");
            let position: String = listing.get("position");
            let salary: i64 = listing.get("salary");
            let loyalty: i64 = listing.get("loyalty");
            let from_team_name: String = listing.get("from_team_name");

            let mut best_bid: Option<(i64, String, i64, i64, i64)> = None; // (team_id, team_name, bid_price, offered_salary, contract_years)

            for team in &teams {
                let team_id: i64 = team.get("id");
                if team_id == from_team_id {
                    continue;
                }

                let balance: i64 = team.get("balance");
                if balance < min_price {
                    continue;
                }

                let fin_status = FinancialStatus::from_balance(balance);
                if !fin_status.can_buy() {
                    continue;
                }

                // 检查位置需求
                let roster = self.get_team_roster(pool, save_id, team_id).await?;
                let pos_count = roster.iter()
                    .filter(|r| {
                        let p: String = r.get("position");
                        p == position
                    })
                    .count();

                if pos_count >= 2 || roster.len() >= 10 {
                    continue;
                }

                let personality = self.get_team_personality_config(pool, team_id).await;
                let weights = personality.as_ref()
                    .map(|p| p.get_weights())
                    .unwrap_or_default();

                let match_score = self.calculate_match_score(
                    ability as u8, age as u8, &position, &weights, balance,
                );

                if match_score < 50.0 {
                    continue;
                }

                // 出价
                let bid_price = (listing_price as f64 * (0.9 + rng.gen::<f64>() * 0.2)) as i64;
                if bid_price < min_price || bid_price > balance {
                    continue;
                }

                let team_name: String = team.get("name");
                let expected_salary = self.calculate_expected_salary(ability as u8, age as u8);
                let contract_years = if age <= 24 { 3 } else if age <= 28 { 2 } else { 1 };

                if best_bid.is_none() || bid_price > best_bid.as_ref().unwrap().2 {
                    best_bid = Some((team_id, team_name, bid_price, expected_salary, contract_years));
                }
            }

            if let Some((to_team_id, to_team_name, bid_price, new_salary, contract_years)) = best_bid {
                // 球员意愿检查
                let willingness = self.calculate_willingness(
                    ability as u8, loyalty as u8, age as u8,
                    new_salary, salary, &mut rng,
                );

                if willingness < 40.0 {
                    continue; // 球员拒绝
                }

                // 执行转会
                sqlx::query(
                    "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = 50, satisfaction = 55 WHERE id = ?"
                )
                .bind(to_team_id)
                .bind(new_salary)
                .bind(season_id + contract_years)
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

                let event = self.record_event(
                    pool, window_id, 5,
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
            round: 5,
            round_name: "有合同选手挖角".to_string(),
            events,
            summary: "已完成有合同选手交易".to_string(),
        })
    }

    // ============================================
    // 第6轮：财政调整
    // ============================================

    async fn execute_financial_adjustment(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        _season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // 查找财务困难球队
        let teams: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT id, name, balance FROM teams WHERE save_id = ? AND balance < 1000000"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询财务困难球队失败: {}", e))?;

        for team in &teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            // 找出最高薪的非核心选手
            let expensive_players: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
                r#"SELECT id, game_id, ability, salary, age
                   FROM players
                   WHERE save_id = ? AND team_id = ? AND status = 'Active'
                   ORDER BY salary DESC
                   LIMIT 2"#
            )
            .bind(save_id)
            .bind(team_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询高薪球员失败: {}", e))?;

            for player in &expensive_players {
                let player_id: i64 = player.get("id");
                let game_id: String = player.get("game_id");
                let ability: i64 = player.get("ability");
                let salary: i64 = player.get("salary");
                let age: i64 = player.get("age");

                // 检查是否已挂牌
                let already_listed: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM player_listings WHERE player_id = ? AND window_id = ? AND status = 'ACTIVE'"
                )
                .bind(player_id)
                .bind(window_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("检查挂牌状态失败: {}", e))?;

                if already_listed.is_some() {
                    continue;
                }

                // 挂牌出售
                let listing_price = self.calculate_market_value_simple(ability as u8, age as u8);
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

        Ok(RoundResult {
            round: 6,
            round_name: "财政调整".to_string(),
            events,
            summary: "已完成财政调整：财务困难球队处理".to_string(),
        })
    }

    // ============================================
    // 第7轮：收尾补救
    // ============================================

    async fn execute_final_remedy(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // 检查所有球队阵容完整性
        let teams = self.get_all_teams(pool, save_id).await?;

        for team in &teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            let roster = self.get_team_roster(pool, save_id, team_id).await?;
            let roster_count = roster.len();

            if roster_count >= 5 {
                continue;
            }

            // 需要紧急签约
            let positions_needed = self.find_missing_positions(&roster);

            for position in positions_needed {
                let pos_str = match position {
                    Position::Top => "TOP",
                    Position::Jug => "JUG",
                    Position::Mid => "MID",
                    Position::Adc => "ADC",
                    Position::Sup => "SUP",
                };

                // 找最佳可用自由球员
                let candidate: Option<sqlx::sqlite::SqliteRow> = sqlx::query(
                    r#"SELECT id, game_id, ability, age
                       FROM players
                       WHERE save_id = ? AND status = 'Active' AND team_id IS NULL AND position = ?
                       ORDER BY ability DESC
                       LIMIT 1"#
                )
                .bind(save_id)
                .bind(pos_str)
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("查找紧急签约候选失败: {}", e))?;

                if let Some(player) = candidate {
                    let player_id: i64 = player.get("id");
                    let game_id: String = player.get("game_id");
                    let ability: i64 = player.get("ability");
                    let age: i64 = player.get("age");

                    let salary = self.calculate_expected_salary(ability as u8, age as u8);
                    let contract_years = 1i64;

                    sqlx::query(
                        "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = 40, satisfaction = 50 WHERE id = ?"
                    )
                    .bind(team_id)
                    .bind(salary)
                    .bind(season_id + contract_years)
                    .bind(player_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("紧急签约失败: {}", e))?;

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

        // 更新所有球队战力
        self.recalculate_team_powers(pool, save_id).await?;

        Ok(RoundResult {
            round: 7,
            round_name: "收尾补救".to_string(),
            events,
            summary: "已完成收尾补救：确保阵容完整性".to_string(),
        })
    }

    // ============================================
    // 第8轮：选秀权拍卖
    // ============================================

    async fn execute_draft_pick_auction_round(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        _save_id: &str,
        _season_id: i64,
    ) -> Result<RoundResult, String> {
        // 选秀权拍卖在 Draft 阶段单独处理
        // 这里只是标记该轮完成
        let event = self.record_event(
            pool, window_id, 8,
            TransferEventType::DraftPickAuction,
            EventLevel::C,
            0, "选秀权拍卖", 0,
            None, None, None, None,
            0, 0, 0,
            "选秀权拍卖将在选秀阶段进行",
        ).await?;

        Ok(RoundResult {
            round: 8,
            round_name: "选秀权拍卖".to_string(),
            events: vec![event],
            summary: "选秀权拍卖阶段完成".to_string(),
        })
    }

    // ============================================
    // 声望引擎
    // ============================================

    pub async fn calculate_team_reputation(
        &self,
        pool: &Pool<Sqlite>,
        team_id: i64,
        save_id: &str,
        current_season: i64,
    ) -> Result<TeamReputation, String> {
        // 历史声望：基于累计荣誉
        let historical_honors: Vec<(String,)> = sqlx::query_as(
            "SELECT honor_type FROM honors WHERE team_id = ? AND save_id = ?"
        )
        .bind(team_id)
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询历史荣誉失败: {}", e))?;

        let mut historical: i64 = 0;
        for (honor_type,) in &historical_honors {
            historical += match honor_type.as_str() {
                "TeamChampion" => 20,
                "TeamRunnerUp" => 10,
                "TeamThird" => 5,
                "TeamFourth" => 3,
                _ => 0,
            };
        }
        historical = historical.min(100);

        // 近期声望：最近3个赛季积分
        let recent_points: Option<(i64,)> = sqlx::query_as(
            r#"SELECT COALESCE(SUM(points), 0)
               FROM annual_points_detail
               WHERE team_id = ? AND save_id = ? AND season_id > ? AND season_id <= ?"#
        )
        .bind(team_id)
        .bind(save_id)
        .bind(current_season - 3)
        .bind(current_season)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询近期积分失败: {}", e))?;

        let recent = recent_points
            .map(|(pts,)| (pts as f64 / 200.0 * 100.0).min(100.0) as i64)
            .unwrap_or(30);

        // 国际声望
        let intl_count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*)
               FROM honors
               WHERE team_id = ? AND save_id = ?
               AND (tournament_name LIKE '%世界赛%'
                    OR tournament_name LIKE '%MSI%'
                    OR tournament_name LIKE '%洲际%'
                    OR tournament_name LIKE '%Worlds%'
                    OR tournament_name LIKE '%Masters%')"#
        )
        .bind(team_id)
        .bind(save_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("查询国际荣誉失败: {}", e))?;

        let international = (intl_count.0 * 15).min(100);

        let overall = (historical as f64 * 0.3 + recent as f64 * 0.4 + international as f64 * 0.3) as i64;

        Ok(TeamReputation {
            team_id,
            overall: overall.min(100),
            historical,
            recent,
            international,
        })
    }

    // ============================================
    // 转会报告
    // ============================================

    pub async fn generate_transfer_report(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
    ) -> Result<TransferReport, String> {
        let window = self.get_window(pool, window_id).await?;

        let all_events = self.get_events(pool, window_id, None, None).await?;

        let mut events_by_type: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let mut events_by_level: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let mut total_transfer_fee = 0i64;
        let mut team_stats: std::collections::HashMap<i64, (String, i64, i64, i64, i64)> = std::collections::HashMap::new();

        for event in &all_events {
            *events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            *events_by_level.entry(event.level.clone()).or_insert(0) += 1;
            total_transfer_fee += event.transfer_fee;

            // 统计球队转入/转出
            if let Some(from_id) = event.from_team_id {
                let entry = team_stats.entry(from_id).or_insert_with(|| {
                    (event.from_team_name.clone().unwrap_or_default(), 0, 0, 0, 0)
                });
                entry.2 += 1; // players_out
                entry.4 += event.transfer_fee; // money_earned
            }
            if let Some(to_id) = event.to_team_id {
                let entry = team_stats.entry(to_id).or_insert_with(|| {
                    (event.to_team_name.clone().unwrap_or_default(), 0, 0, 0, 0)
                });
                entry.1 += 1; // players_in
                entry.3 += event.transfer_fee; // money_spent
            }
        }

        let team_summaries: Vec<TeamTransferSummary> = team_stats
            .into_iter()
            .map(|(team_id, (name, players_in, players_out, spent, earned))| {
                TeamTransferSummary {
                    team_id,
                    team_name: name,
                    players_in,
                    players_out,
                    money_spent: spent,
                    money_earned: earned,
                    net_spend: spent - earned,
                }
            })
            .collect();

        let mut top_events: Vec<TransferEvent> = all_events
            .iter()
            .filter(|e| e.level == "S" || e.level == "A")
            .cloned()
            .collect();
        top_events.sort_by(|a, b| b.transfer_fee.cmp(&a.transfer_fee));
        top_events.truncate(10);

        Ok(TransferReport {
            window_id,
            season_id: window.season_id,
            total_events: all_events.len() as i64,
            total_transfer_fee,
            events_by_type,
            events_by_level,
            team_summaries,
            top_events,
        })
    }

    // ============================================
    // 辅助方法
    // ============================================

    async fn get_window(&self, pool: &Pool<Sqlite>, window_id: i64) -> Result<TransferWindow, String> {
        let row: sqlx::sqlite::SqliteRow = sqlx::query(
            "SELECT id, save_id, season_id, status, current_round, started_at, completed_at FROM transfer_windows WHERE id = ?"
        )
        .bind(window_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("获取转会期失败: {}", e))?;

        let status_str: String = row.get("status");
        Ok(TransferWindow {
            id: row.get("id"),
            save_id: row.get("save_id"),
            season_id: row.get("season_id"),
            status: TransferWindowStatus::from_str(&status_str),
            current_round: row.get("current_round"),
            started_at: row.get("started_at"),
            completed_at: row.try_get("completed_at").ok(),
        })
    }

    async fn get_active_players(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<Vec<sqlx::sqlite::SqliteRow>, String> {
        sqlx::query(
            "SELECT * FROM players WHERE save_id = ? AND status = 'Active'"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询活跃选手失败: {}", e))
    }

    async fn get_all_teams(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<Vec<sqlx::sqlite::SqliteRow>, String> {
        sqlx::query("SELECT * FROM teams WHERE save_id = ?")
            .bind(save_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询球队失败: {}", e))
    }

    async fn get_team_roster(&self, pool: &Pool<Sqlite>, save_id: &str, team_id: i64) -> Result<Vec<sqlx::sqlite::SqliteRow>, String> {
        sqlx::query(
            "SELECT * FROM players WHERE save_id = ? AND team_id = ? AND status = 'Active'"
        )
        .bind(save_id)
        .bind(team_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询球队阵容失败: {}", e))
    }

    async fn get_team_name(&self, pool: &Pool<Sqlite>, team_id: i64) -> Result<String, String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT name FROM teams WHERE id = ?"
        )
        .bind(team_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("获取球队名称失败: {}", e))?;
        Ok(row.map(|(n,)| n).unwrap_or_default())
    }

    async fn get_team_personality_config(
        &self,
        pool: &Pool<Sqlite>,
        team_id: i64,
    ) -> Option<TeamPersonalityConfig> {
        let row: Option<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT * FROM team_personality_configs WHERE team_id = ?"
        )
        .bind(team_id)
        .fetch_optional(pool)
        .await
        .ok()?;

        row.map(|r| TeamPersonalityConfig {
            id: r.get("id"),
            team_id: r.get("team_id"),
            save_id: r.get("save_id"),
            personality: r.get("personality"),
            short_term_focus: r.get("short_term_focus"),
            long_term_focus: r.get("long_term_focus"),
            risk_tolerance: r.get("risk_tolerance"),
            youth_preference: r.get("youth_preference"),
            star_chasing: r.get("star_chasing"),
            bargain_hunting: r.get("bargain_hunting"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn init_team_personalities(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<(), String> {
        sqlx::query(
            r#"INSERT OR IGNORE INTO team_personality_configs (team_id, save_id, personality, updated_at)
               SELECT id, save_id, 'BALANCED', datetime('now')
               FROM teams WHERE save_id = ?"#
        )
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| format!("初始化球队性格失败: {}", e))?;
        Ok(())
    }

    async fn recalculate_team_powers(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<(), String> {
        let teams = self.get_all_teams(pool, save_id).await?;
        for team in &teams {
            let team_id: i64 = team.get("id");
            let result: Option<(f64,)> = sqlx::query_as(
                "SELECT AVG(ability) FROM players WHERE save_id = ? AND team_id = ? AND status = 'Active' AND is_starter = 1"
            )
            .bind(save_id)
            .bind(team_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("计算球队战力失败: {}", e))?;

            let power = result.map(|(avg,)| avg).unwrap_or(60.0);

            sqlx::query("UPDATE teams SET power_rating = ? WHERE id = ?")
                .bind(power)
                .bind(team_id)
                .execute(pool)
                .await
                .map_err(|e| format!("更新球队战力失败: {}", e))?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_event(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        round: i64,
        event_type: TransferEventType,
        level: EventLevel,
        player_id: i64,
        player_name: &str,
        player_ability: i64,
        from_team_id: Option<i64>,
        from_team_name: Option<&str>,
        to_team_id: Option<i64>,
        to_team_name: Option<&str>,
        transfer_fee: i64,
        salary: i64,
        contract_years: i64,
        reason: &str,
    ) -> Result<TransferEvent, String> {
        let result = sqlx::query(
            r#"INSERT INTO transfer_events
               (window_id, round, event_type, level, player_id, player_name, player_ability,
                from_team_id, from_team_name, to_team_id, to_team_name,
                transfer_fee, salary, contract_years, reason)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(window_id)
        .bind(round)
        .bind(event_type.as_str())
        .bind(level.as_str())
        .bind(player_id)
        .bind(player_name)
        .bind(player_ability)
        .bind(from_team_id)
        .bind(from_team_name)
        .bind(to_team_id)
        .bind(to_team_name)
        .bind(transfer_fee)
        .bind(salary)
        .bind(contract_years)
        .bind(reason)
        .execute(pool)
        .await
        .map_err(|e| format!("记录转会事件失败: {}", e))?;

        Ok(TransferEvent {
            id: result.last_insert_rowid(),
            window_id,
            round,
            event_type: event_type.as_str().to_string(),
            level: level.as_str().to_string(),
            player_id,
            player_name: player_name.to_string(),
            player_ability,
            from_team_id,
            from_team_name: from_team_name.map(String::from),
            to_team_id,
            to_team_name: to_team_name.map(String::from),
            transfer_fee,
            salary,
            contract_years,
            reason: Some(reason.to_string()),
            created_at: String::new(),
        })
    }

    pub async fn get_events(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        round: Option<i64>,
        level: Option<&str>,
    ) -> Result<Vec<TransferEvent>, String> {
        let mut query = String::from(
            r#"SELECT id, window_id, round, event_type, level, player_id, player_name, player_ability,
                      from_team_id, from_team_name, to_team_id, to_team_name,
                      transfer_fee, salary, contract_years, reason, created_at
               FROM transfer_events WHERE window_id = ?"#
        );

        if round.is_some() {
            query.push_str(" AND round = ?");
        }
        if level.is_some() {
            query.push_str(" AND level = ?");
        }
        query.push_str(" ORDER BY created_at ASC");

        let mut q = sqlx::query(&query).bind(window_id);
        if let Some(r) = round {
            q = q.bind(r);
        }
        if let Some(l) = level {
            q = q.bind(l);
        }

        let rows: Vec<sqlx::sqlite::SqliteRow> = q
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询转会事件失败: {}", e))?;

        Ok(rows.iter().map(|row| TransferEvent {
            id: row.get("id"),
            window_id: row.get("window_id"),
            round: row.get("round"),
            event_type: row.get("event_type"),
            level: row.get("level"),
            player_id: row.get("player_id"),
            player_name: row.get("player_name"),
            player_ability: row.get("player_ability"),
            from_team_id: row.try_get("from_team_id").ok(),
            from_team_name: row.try_get("from_team_name").ok(),
            to_team_id: row.try_get("to_team_id").ok(),
            to_team_name: row.try_get("to_team_name").ok(),
            transfer_fee: row.get("transfer_fee"),
            salary: row.get("salary"),
            contract_years: row.get("contract_years"),
            reason: row.try_get("reason").ok(),
            created_at: row.get("created_at"),
        }).collect())
    }

    // ============================================
    // 计算方法
    // ============================================

    /// 计算期望薪资（万元 -> 元）
    fn calculate_expected_salary(&self, ability: u8, age: u8) -> i64 {
        let base = ability as i64 * 8; // 万元/年

        let ability_coeff = match ability {
            90..=100 => 1.5,
            85..=89 => 1.3,
            80..=84 => 1.1,
            75..=79 => 1.0,
            _ => 0.8,
        };

        let age_coeff = match age {
            18..=24 => 0.8,
            25..=29 => 1.0,
            30..=34 => 0.9,
            _ => 0.7,
        };

        (base as f64 * ability_coeff * age_coeff * 10000.0) as i64
    }

    /// 计算简易身价（万元 -> 元）
    fn calculate_market_value_simple(&self, ability: u8, age: u8) -> i64 {
        let base_multiplier = match ability {
            95..=100 => 50i64,
            90..=94 => 35,
            85..=89 => 20,
            80..=84 => 12,
            75..=79 => 7,
            70..=74 => 4,
            60..=69 => 2,
            _ => 1,
        };

        let base_value = ability as i64 * base_multiplier;

        let age_factor = match age {
            17..=19 => 1.5,
            20..=22 => 1.3,
            23..=25 => 1.0,
            26..=27 => 0.85,
            28..=29 => 0.7,
            _ => 0.5,
        };

        (base_value as f64 * age_factor * 10000.0) as i64
    }

    /// 计算匹配度（0-100）
    fn calculate_match_score(
        &self,
        ability: u8,
        age: u8,
        _position: &str,
        weights: &AIDecisionWeights,
        balance: i64,
    ) -> f64 {
        // 能力匹配
        let ability_score = match ability {
            90..=100 => 100.0,
            85..=89 => 90.0,
            80..=84 => 80.0,
            75..=79 => 70.0,
            70..=74 => 60.0,
            _ => 40.0,
        };

        // 年龄匹配（根据性格偏好）
        let age_score = if weights.youth_preference > 0.7 {
            match age {
                17..=22 => 100.0,
                23..=25 => 80.0,
                26..=28 => 60.0,
                _ => 40.0,
            }
        } else if weights.short_term_focus > 0.7 {
            match age {
                24..=28 => 100.0,
                22..=30 => 80.0,
                _ => 60.0,
            }
        } else {
            match age {
                20..=28 => 100.0,
                18..=30 => 80.0,
                _ => 60.0,
            }
        };

        // 财务匹配
        let fin_status = FinancialStatus::from_balance(balance);
        let finance_score = match fin_status {
            FinancialStatus::Wealthy => 100.0,
            FinancialStatus::Healthy => 80.0,
            FinancialStatus::Tight => 60.0,
            _ => 30.0,
        };

        ability_score * 0.4 * weights.short_term_focus
            + age_score * 0.3 * weights.youth_preference.max(weights.short_term_focus)
            + finance_score * 0.3
    }

    /// 计算球员转会意愿（0-100）
    fn calculate_willingness(
        &self,
        _ability: u8,
        loyalty: u8,
        _age: u8,
        offered_salary: i64,
        current_salary: i64,
        rng: &mut impl Rng,
    ) -> f64 {
        // 薪资满意度
        let salary_ratio = if current_salary > 0 {
            offered_salary as f64 / current_salary as f64
        } else {
            1.5
        };
        let salary_score = if salary_ratio >= 1.2 { 100.0 }
            else if salary_ratio >= 1.0 { 80.0 }
            else if salary_ratio >= 0.8 { 60.0 }
            else if salary_ratio >= 0.6 { 40.0 }
            else { 20.0 };

        // 忠诚度影响（高忠诚度降低转会意愿）
        let loyalty_impact = (100.0 - loyalty as f64) * 0.5;

        // 随机波动
        let random_factor: f64 = rng.gen_range(-5.0..5.0);

        let base = salary_score * 0.4 + loyalty_impact * 0.3 + 50.0 * 0.3;
        (base + random_factor).clamp(0.0, 100.0)
    }

    /// 分析球队需求
    fn analyze_team_needs(
        &self,
        roster: &[sqlx::sqlite::SqliteRow],
        team_id: i64,
        team_name: &str,
        balance: i64,
    ) -> TeamNeedsAnalysis {
        let mut position_counts: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();
        let mut total_age = 0f64;
        let mut total_ability = 0f64;
        let roster_count = roster.len() as i32;

        for player in roster {
            let position: String = player.get("position");
            let ability: i64 = player.get("ability");
            let age: i64 = player.get("age");

            let entry = position_counts.entry(position).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += ability as f64;
            total_age += age as f64;
            total_ability += ability as f64;
        }

        let avg_age = if roster_count > 0 { total_age / roster_count as f64 } else { 25.0 };
        let power_rating = if roster_count > 0 { total_ability / roster_count as f64 } else { 60.0 };

        let positions = ["TOP", "JUG", "MID", "ADC", "SUP"];
        let mut position_needs = std::collections::HashMap::new();

        for pos in &positions {
            let (count, total_ab) = position_counts.get(*pos).copied().unwrap_or((0, 0.0));
            let avg_ab = if count > 0 { total_ab / count as f64 } else { 0.0 };
            let gap = 1 - count; // 每个位置至少需要1人

            let priority = if count == 0 {
                NeedPriority::Critical
            } else if avg_ab < 70.0 {
                NeedPriority::High
            } else if avg_ab < 80.0 {
                NeedPriority::Medium
            } else {
                NeedPriority::Low
            };

            let pos_enum = match *pos {
                "TOP" => Position::Top,
                "JUG" => Position::Jug,
                "MID" => Position::Mid,
                "ADC" => Position::Adc,
                _ => Position::Sup,
            };

            position_needs.insert(pos.to_string(), PositionNeed {
                position: pos_enum,
                current_count: count,
                target_count: 1,
                gap,
                current_avg_ability: avg_ab,
                required_ability: 70,
                priority,
            });
        }

        let fin_status = FinancialStatus::from_balance(balance);

        TeamNeedsAnalysis {
            team_id,
            team_name: team_name.to_string(),
            position_needs,
            financial_status: format!("{:?}", fin_status),
            avg_age,
            roster_count,
            power_rating,
            priority_score: 50.0,
            listing_candidates: Vec::new(),
            target_positions: Vec::new(),
        }
    }

    /// 找出缺少的位置
    fn find_missing_positions(&self, roster: &[sqlx::sqlite::SqliteRow]) -> Vec<Position> {
        let mut has_position = [false; 5]; // Top, Jug, Mid, Adc, Sup

        for player in roster {
            let pos: String = player.get("position");
            match pos.as_str() {
                "TOP" => has_position[0] = true,
                "JUG" => has_position[1] = true,
                "MID" => has_position[2] = true,
                "ADC" => has_position[3] = true,
                "SUP" => has_position[4] = true,
                _ => {}
            }
        }

        let mut missing = Vec::new();
        let all_positions = [Position::Top, Position::Jug, Position::Mid, Position::Adc, Position::Sup];

        for (i, has) in has_position.iter().enumerate() {
            if !has {
                missing.push(all_positions[i]);
            }
        }

        missing
    }
}
