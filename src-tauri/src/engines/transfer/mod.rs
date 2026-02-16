//! 转会系统引擎
//!
//! 实现完整的7轮转会流程

mod cache;
mod round1_settlement;
mod round2_evaluation;
mod round3_renewal;
mod round4_free_agent;
mod round5_contracted;
mod round6_financial;
mod round7_remedy;
mod scoring;
mod utils;

#[cfg(test)]
mod tests;

pub use cache::*;
pub use utils::normalize_position;

use sqlx::{Pool, Row, Sqlite};

use crate::models::transfer::*;

pub struct TransferEngine {
    pub(crate) config: TransferConfig,
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

    pub(crate) async fn record_financial_transaction(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
        team_id: i64,
        transaction_type: &str,
        amount: i64,
        description: &str,
        related_player_id: i64,
    ) -> Result<(), String> {
        sqlx::query(
            r#"INSERT INTO financial_transactions (
                save_id, team_id, season_id, transaction_type, amount, description, related_player_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(save_id)
        .bind(team_id)
        .bind(season_id)
        .bind(transaction_type)
        .bind(amount)
        .bind(description)
        .bind(related_player_id)
        .execute(pool)
        .await
        .map_err(|e| format!("记录财务交易失败: {}", e))?;
        Ok(())
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
        let round_start = std::time::Instant::now();
        eprintln!("[转会] R{} 开始", round);

        let window = self.get_window(pool, window_id).await?;
        let save_id = &window.save_id;

        let mut cache = TransferCache::build(pool, save_id, window.season_id).await?;
        eprintln!("[转会] R{} cache构建完成 {:?}", round, round_start.elapsed());

        let result = match round {
            1 => self.execute_season_settlement(pool, window_id, save_id, window.season_id, &mut cache).await?,
            2 => self.execute_bidirectional_evaluation(pool, window_id, save_id, window.season_id, &mut cache).await?,
            3 => self.execute_renewal_negotiations(pool, window_id, save_id, window.season_id, &mut cache).await?,
            4 => self.execute_free_agent_bidding(pool, window_id, save_id, window.season_id, &mut cache).await?,
            5 => self.execute_contracted_player_transfer(pool, window_id, save_id, window.season_id, &mut cache, 5).await?,
            6 => self.execute_financial_adjustment(pool, window_id, save_id, window.season_id, &mut cache).await?,
            7 => self.execute_final_remedy(pool, window_id, save_id, window.season_id, &mut cache).await?,
            _ => return Err(format!("无效轮次: {}", round)),
        };

        if round >= 4 {
            Self::recalculate_starters_for_save(pool, save_id).await?;
        }

        sqlx::query("UPDATE transfer_windows SET current_round = ? WHERE id = ?")
            .bind(round)
            .bind(window_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新转会期轮次失败: {}", e))?;

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

// 转会窗口关闭验证
    // ============================================

    /// 验证并关闭转会窗口
    pub async fn validate_and_close_window(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        force: bool,
    ) -> Result<TransferWindowCloseValidation, String> {
        // 1. 验证 window 状态
        let window_row = sqlx::query(
            "SELECT id, save_id, season_id, status, current_round FROM transfer_windows WHERE id = ?"
        )
        .bind(window_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询转会窗口失败: {}", e))?;

        let window_row = match window_row {
            Some(r) => r,
            None => return Err("转会窗口不存在".to_string()),
        };

        let status: String = window_row.get("status");
        let current_round: i64 = window_row.get("current_round");
        let save_id: String = window_row.get("save_id");
        let season_id: i64 = window_row.get("season_id");

        if status == "COMPLETED" {
            return Ok(TransferWindowCloseValidation {
                is_valid: true,
                window_id,
                issues: vec![],
                message: "转会窗口已关闭".to_string(),
            });
        }

        if status != "IN_PROGRESS" {
            return Err("转会窗口状态不正确，只有进行中的窗口才能关闭".to_string());
        }

        if current_round < self.config.max_rounds {
            return Err(format!(
                "还有未完成的轮次（当前第{}轮，共{}轮），请先完成所有轮次",
                current_round, self.config.max_rounds
            ));
        }

        // 2. 检查所有球队阵容
        let mut issues: Vec<TransferCloseIssue> = Vec::new();

        let teams = sqlx::query(
            "SELECT id, name FROM teams WHERE save_id = ?"
        )
        .bind(&save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询球队失败: {}", e))?;

        for team_row in &teams {
            let team_id: i64 = team_row.get("id");
            let team_name: String = team_row.get("name");

            // 查询活跃选手数量
            let active_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM players WHERE team_id = ? AND status = 'Active'"
            )
            .bind(team_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询球队阵容失败: {}", e))?;

            if active_count < 5 {
                issues.push(TransferCloseIssue {
                    team_id,
                    team_name: team_name.clone(),
                    issue_type: "ROSTER_TOO_SMALL".to_string(),
                    detail: format!("{}只有{}名活跃选手，最少需要5名", team_name, active_count),
                });
            }

            if active_count > 15 {
                issues.push(TransferCloseIssue {
                    team_id,
                    team_name: team_name.clone(),
                    issue_type: "ROSTER_TOO_LARGE".to_string(),
                    detail: format!("{}有{}名活跃选手，最多允许15名", team_name, active_count),
                });
            }

            // 检查合同有效性
            let invalid_contracts: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM players WHERE team_id = ? AND status = 'Active' AND contract_end_season <= ?"
            )
            .bind(team_id)
            .bind(season_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询合同失败: {}", e))?;

            if invalid_contracts > 0 {
                issues.push(TransferCloseIssue {
                    team_id,
                    team_name: team_name.clone(),
                    issue_type: "INVALID_CONTRACT".to_string(),
                    detail: format!("{}有{}名选手合同已过期", team_name, invalid_contracts),
                });
            }
        }

        let is_valid = issues.is_empty();

        // 3. 如果通过验证或强制关闭，则标记 COMPLETED
        if is_valid || force {
            sqlx::query(
                "UPDATE transfer_windows SET status = 'COMPLETED', completed_at = datetime('now') WHERE id = ?"
            )
            .bind(window_id)
            .execute(pool)
            .await
            .map_err(|e| format!("关闭转会窗口失败: {}", e))?;

            sqlx::query(
                "UPDATE player_listings SET status = 'EXPIRED' WHERE window_id = ? AND status = 'ACTIVE'"
            )
            .bind(window_id)
            .execute(pool)
            .await
            .map_err(|e| format!("清理未售出挂牌失败: {}", e))?;

            let message = if is_valid {
                "转会窗口验证通过，已成功关闭".to_string()
            } else {
                format!("转会窗口已强制关闭，存在{}个问题", issues.len())
            };

            Ok(TransferWindowCloseValidation {
                is_valid,
                window_id,
                issues,
                message,
            })
        } else {
            Ok(TransferWindowCloseValidation {
                is_valid: false,
                window_id,
                issues,
                message: "转会窗口验证未通过，请处理以下问题或选择强制关闭".to_string(),
            })
        }
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

            let is_player_movement = matches!(
                event.event_type.as_str(),
                "CONTRACT_TERMINATION" | "FREE_AGENT_SIGNING" | "TRANSFER_PURCHASE"
                | "EMERGENCY_SIGNING" | "PLAYER_RETIREMENT" | "RETIREMENT" | "PLAYER_RELEASE"
            );

            if !is_player_movement {
                continue;
            }

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

    pub(crate) async fn recalculate_starters_for_save(pool: &Pool<Sqlite>, save_id: &str) -> Result<(), String> {
        sqlx::query("UPDATE players SET is_starter = 0 WHERE save_id = ? AND status = 'Active'")
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("清除首发失败: {}", e))?;

        sqlx::query(
            r#"UPDATE players SET is_starter = 1 WHERE id IN (
                SELECT id FROM (
                    SELECT id, ROW_NUMBER() OVER (
                        PARTITION BY team_id, UPPER(position)
                        ORDER BY ability DESC
                    ) as rn
                    FROM players
                    WHERE save_id = ? AND status = 'Active' AND team_id IS NOT NULL
                ) WHERE rn = 1
            )"#,
        )
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| format!("重算首发失败: {}", e))?;

        Ok(())
    }

    pub(crate) async fn get_window(&self, pool: &Pool<Sqlite>, window_id: i64) -> Result<TransferWindow, String> {
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

    pub(crate) async fn init_team_personalities(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<(), String> {
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

    /// 单条 SQL 更新所有球队战力（替代 N+1 循环查询）
    pub(crate) async fn recalculate_team_powers_optimized(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<(), String> {
        sqlx::query(
            r#"UPDATE teams SET power_rating = COALESCE(
                (SELECT AVG(ability) FROM players
                 WHERE players.save_id = teams.save_id
                 AND players.team_id = teams.id
                 AND players.status = 'Active'
                 AND players.is_starter = 1),
                60.0
            ) WHERE save_id = ?"#
        )
        .bind(save_id)
        .execute(pool)
        .await
        .map_err(|e| format!("批量更新球队战力失败: {}", e))?;
        Ok(())
    }

    /// 记录选手合同（签约/续约/转会时写入合同历史）
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn insert_contract(
        pool: &Pool<Sqlite>,
        save_id: &str,
        player_id: i64,
        team_id: i64,
        contract_type: &str,
        total_salary: i64,
        contract_years: i64,
        season_id: i64,
        transfer_fee: i64,
        signing_bonus: i64,
    ) -> Result<(), String> {
        // 1. 将该选手旧的活跃合同设为非活跃
        sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
            .bind(save_id).bind(player_id)
            .execute(pool).await.ok();

        // 2. 插入新合同
        let annual_salary = if contract_years > 0 { total_salary / contract_years } else { total_salary };
        sqlx::query(
            r#"INSERT INTO player_contracts (save_id, player_id, team_id, contract_type, total_salary, annual_salary, contract_years, start_season, end_season, transfer_fee, signing_bonus, is_active)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)"#
        )
        .bind(save_id).bind(player_id).bind(team_id)
        .bind(contract_type).bind(total_salary).bind(annual_salary).bind(contract_years)
        .bind(season_id).bind(season_id + contract_years)
        .bind(transfer_fee).bind(signing_bonus)
        .execute(pool)
        .await
        .map_err(|e| format!("记录合同失败: {}", e))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn record_event(
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
        // 从 transfer_windows 获取 save_id 和 season_id
        let window_row = sqlx::query(
            "SELECT save_id, season_id FROM transfer_windows WHERE id = ?"
        )
        .bind(window_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("查询转会窗口失败: {}", e))?;
        let save_id: String = window_row.get("save_id");
        let season_id: i64 = window_row.get("season_id");

        let result = sqlx::query(
            r#"INSERT INTO transfer_events
               (save_id, season_id, window_id, round, event_type, level, player_id, player_name, player_ability,
                from_team_id, from_team_name, to_team_id, to_team_name,
                transfer_fee, salary, contract_years, reason)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&save_id)
        .bind(season_id)
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

pub(crate) async fn insert_bid(
        pool: &Pool<Sqlite>,
        window_id: i64,
        round: i64,
        player_id: i64,
        player_name: &str,
        ability: i64,
        age: i64,
        position: &str,
        from_team_id: Option<i64>,
        from_team_name: Option<&str>,
        bid_team_id: i64,
        bid_team_name: &str,
        bid_team_region_id: Option<i64>,
        offered_salary: i64,
        contract_years: i64,
        transfer_fee: i64,
        signing_bonus: i64,
        match_score: f64,
        willingness: f64,
        is_winner: bool,
        reject_reason: Option<&str>,
    ) -> Result<(), String> {
        sqlx::query(
            r#"INSERT INTO transfer_bids
               (window_id, round, player_id, player_name, player_ability, player_age, player_position,
                from_team_id, from_team_name, bid_team_id, bid_team_name, bid_team_region_id,
                offered_salary, contract_years, transfer_fee, signing_bonus,
                match_score, willingness, is_winner, reject_reason)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(window_id)
        .bind(round)
        .bind(player_id)
        .bind(player_name)
        .bind(ability)
        .bind(age)
        .bind(position)
        .bind(from_team_id)
        .bind(from_team_name)
        .bind(bid_team_id)
        .bind(bid_team_name)
        .bind(bid_team_region_id)
        .bind(offered_salary)
        .bind(contract_years)
        .bind(transfer_fee)
        .bind(signing_bonus)
        .bind(match_score)
        .bind(willingness)
        .bind(is_winner as i32)
        .bind(reject_reason)
        .execute(pool)
        .await
        .map_err(|e| format!("写入竞价记录失败: {}", e))?;
        Ok(())
    }

}
