use sqlx::{Pool, Row, Sqlite};

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
        _cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();

        // ============================================
        // 1. 给所有球队发放赛季薪资
        // ============================================
        let all_teams: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT id, name FROM teams WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询球队失败: {}", e))?;

        let mut salary_paid_count = 0i64;
        let mut total_salary_paid = 0i64;

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let _team_name: String = team.get("name");

            let already_paid: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM financial_transactions WHERE team_id = ? AND save_id = ? AND season_id = ? AND transaction_type = 'Salary'"
            )
            .bind(team_id)
            .bind(save_id)
            .bind(season_id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if already_paid > 0 {
                continue;
            }

            // 计算该队年薪总额（优先从合同表查 annual_salary）
            let team_annual_salary: i64 = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(pc.annual_salary), 0)
                   FROM player_contracts pc
                   JOIN players p ON pc.player_id = p.id
                   WHERE p.team_id = ? AND p.save_id = ? AND p.status = 'Active' AND pc.is_active = 1"#
            )
            .bind(team_id)
            .bind(save_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("计算球队年薪失败: {}", e))?;

            // fallback: 如果合同表查出为0但有活跃选手，回退到旧算法（用 join_season 估算合同总年数）
            let team_annual_salary = if team_annual_salary == 0 {
                let fallback: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(SUM(salary), 0) FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active'"
                )
                .bind(team_id)
                .bind(save_id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);
                fallback
            } else {
                team_annual_salary
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

        // ============================================
        // 1.5 奢侈税结算（阵容超过起征线的球队）
        // ============================================
        let mut luxury_tax_count = 0i64;
        let mut total_luxury_tax = 0i64;

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            let already_taxed: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM financial_transactions WHERE team_id = ? AND save_id = ? AND season_id = ? AND transaction_type = 'LuxuryTax'"
            )
            .bind(team_id)
            .bind(save_id)
            .bind(season_id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if already_taxed > 0 {
                continue;
            }

            let roster_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active'"
            )
            .bind(team_id)
            .bind(save_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询阵容人数失败: {}", e))?;

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
            
            let representative_id: i64 = sqlx::query_scalar(
                "SELECT id FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active' ORDER BY ability DESC LIMIT 1"
            )
            .bind(team_id)
            .bind(save_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询代表选手失败: {}", e))?;

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

        // ============================================
        // 1.6 解约超额选手（挂牌未售出 + 阵容仍超线 → 直接解约）
        // ============================================
        let mut release_count = 0i64;
        let mut total_release_fee = 0i64;

        for team in &all_teams {
            let team_id: i64 = team.get("id");
            let team_name: String = team.get("name");

            // 查询当前阵容人数
            let roster_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM players WHERE team_id = ? AND save_id = ? AND status = 'Active'"
            )
            .bind(team_id)
            .bind(save_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询阵容人数失败: {}", e))?;

            let over_count = roster_count - 8;
            if over_count <= 0 {
                continue;
            }

            // 获取团队余额
            let team_balance: i64 = sqlx::query_scalar(
                "SELECT balance FROM teams WHERE id = ? AND save_id = ?"
            )
            .bind(team_id)
            .bind(save_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询余额失败: {}", e))?;

            // 找出可解约的选手：非首发 + 按（能力值-潜力值培养价值）排序，最差的优先解约
            // 排除首发，按 ability ASC 排序（能力最低的优先解约）
            let release_candidates: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
                r#"SELECT id, game_id, ability, age, potential, tag, position, salary, calculated_market_value
                   FROM players
                   WHERE team_id = ? AND save_id = ? AND status = 'Active' AND is_starter = 0
                   ORDER BY ability ASC, age DESC
                   LIMIT ?"#
            )
            .bind(team_id)
            .bind(save_id)
            .bind(over_count)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询解约候选失败: {}", e))?;

            let mut current_balance = team_balance;
            for candidate in &release_candidates {
                let player_id: i64 = candidate.get("id");
                let game_id: String = candidate.get("game_id");
                let ability: i64 = candidate.get("ability");
                let age: i64 = candidate.get("age");
                let potential: i64 = candidate.get("potential");
                let tag: String = candidate.get("tag");
                let position: String = normalize_position(&candidate.get::<String, _>("position"));
                let calculated_market_value: i64 = candidate.try_get("calculated_market_value").unwrap_or(0);

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

                let event = self.record_event(
                    pool, window_id, 6,
                    TransferEventType::PlayerRelease,
                    EventLevel::from_ability_and_fee(ability as u8, release_fee),
                    player_id, &game_id, ability,
                    Some(team_id), Some(&team_name),
                    None, None,
                    release_fee, candidate.get::<i64, _>("salary"), 0,
                    &format!("{}解约{}以避免奢侈税，支付解约金{}万", team_name, game_id, release_fee / 10000),
                ).await?;
                events.push(event);

                log::info!("R6解约: {}解约{}，解约金{}万", team_name, game_id, release_fee / 10000);
                release_count += 1;
                total_release_fee += release_fee;
            }
        }

        // ============================================
        // 2. 查找财务困难球队，挂牌出售高薪选手
        // ============================================
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
                let listing_price = MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", "MID") as i64;
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
