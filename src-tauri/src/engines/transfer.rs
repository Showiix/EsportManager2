//! 转会系统引擎
//!
//! 实现完整的7轮转会流程

use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};
use std::collections::{HashMap, HashSet};

use crate::models::transfer::*;
use crate::models::player::Position;
use crate::models::team::FinancialStatus;
use crate::engines::market_value::MarketValueEngine;
use crate::engines::traits::TraitType;

/// 缓存的选手信息（避免反复查询 SqliteRow）
#[derive(Debug, Clone)]
pub struct CachedPlayer {
    pub id: i64,
    pub game_id: String,
    pub ability: i64,
    pub potential: i64,
    pub age: i64,
    pub salary: i64,
    pub loyalty: i64,
    pub satisfaction: i64,
    pub position: String,
    pub tag: String,
    pub team_id: Option<i64>,
    pub is_starter: bool,
    pub home_region_id: Option<i64>,
    pub region_loyalty: i64,
    pub contract_end_season: Option<i64>,
    pub status: String,
    pub stability: i64,
}

impl CachedPlayer {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Self {
        Self {
            id: row.get("id"),
            game_id: row.get("game_id"),
            ability: row.get("ability"),
            potential: row.try_get("potential").unwrap_or(0),
            age: row.get("age"),
            salary: row.try_get("salary").unwrap_or(0),
            loyalty: row.try_get("loyalty").unwrap_or(70),
            satisfaction: row.try_get("satisfaction").unwrap_or(70),
            position: row.try_get("position").unwrap_or_default(),
            tag: row.try_get("tag").unwrap_or_else(|_| "NORMAL".to_string()),
            team_id: row.try_get("team_id").ok(),
            is_starter: row.try_get("is_starter").unwrap_or(false),
            home_region_id: row.try_get("home_region_id").ok(),
            region_loyalty: row.try_get("region_loyalty").unwrap_or(70),
            contract_end_season: row.try_get("contract_end_season").ok(),
            status: row.try_get("status").unwrap_or_else(|_| "Active".to_string()),
            stability: row.try_get("stability").unwrap_or(60),
        }
    }
}

/// 转会期间的内存缓存，避免 N+1 查询
pub struct TransferCache {
    pub team_names: HashMap<i64, String>,
    pub team_balances: HashMap<i64, i64>,
    pub team_region_ids: HashMap<i64, Option<i64>>,
    pub team_rosters: HashMap<i64, Vec<CachedPlayer>>,
    pub team_personalities: HashMap<i64, TeamPersonalityConfig>,
    pub player_recent_honors: HashSet<i64>,
    pub team_annual_ranks: HashMap<i64, i32>,
    pub team_last_season_ranks: HashMap<i64, i32>,
    pub team_reputations: HashMap<i64, i64>,
}

impl TransferCache {
    /// 批量构建缓存（替代数百次单独查询）
    pub async fn build(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<Self, String> {
        // 1. 批量加载所有球队信息
        let team_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT * FROM teams WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("缓存: 查询球队失败: {}", e))?;

        let mut team_names = HashMap::new();
        let mut team_balances = HashMap::new();
        let mut team_region_ids = HashMap::new();

        for row in &team_rows {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            let balance: i64 = row.get("balance");
            let region_id: Option<i64> = row.try_get("region_id").ok();
            team_names.insert(id, name);
            team_balances.insert(id, balance);
            team_region_ids.insert(id, region_id);
        }

        // 2. 批量加载所有活跃选手，按 team_id 分组
        let player_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT * FROM players WHERE save_id = ? AND status = 'Active'"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("缓存: 查询选手失败: {}", e))?;

        let mut team_rosters: HashMap<i64, Vec<CachedPlayer>> = HashMap::new();
        for row in &player_rows {
            let player = CachedPlayer::from_row(row);
            if let Some(tid) = player.team_id {
                team_rosters.entry(tid).or_default().push(player);
            }
        }

        // 3. 批量加载所有球队AI性格配置
        let personality_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT * FROM team_personality_configs WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_personalities = HashMap::new();
        for r in &personality_rows {
            let config = TeamPersonalityConfig {
                id: r.get("id"),
                team_id: r.get("team_id"),
                save_id: r.get("save_id"),
                personality: r.try_get("personality").unwrap_or_else(|_| "BALANCED".to_string()),
                short_term_focus: r.try_get("short_term_focus").unwrap_or(0.5),
                long_term_focus: r.try_get("long_term_focus").unwrap_or(0.5),
                risk_tolerance: r.try_get("risk_tolerance").unwrap_or(0.5),
                youth_preference: r.try_get("youth_preference").unwrap_or(0.5),
                star_chasing: r.try_get("star_chasing").unwrap_or(0.5),
                bargain_hunting: r.try_get("bargain_hunting").unwrap_or(0.5),
                updated_at: r.try_get("updated_at").unwrap_or_default(),
            };
            team_personalities.insert(config.team_id, config);
        }

        // 4. 批量加载近2赛季有荣誉的选手ID
        let honor_rows: Vec<(i64,)> = sqlx::query_as(
            r#"SELECT DISTINCT player_id FROM honors
               WHERE save_id = ? AND player_id IS NOT NULL
               AND season_id >= ?
               AND honor_type IN ('CHAMPION', 'MVP', 'FINALS_MVP', 'YEARLY_MVP', 'YEARLY_TOP20')"#
        )
        .bind(save_id)
        .bind(season_id - 1)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let player_recent_honors: HashSet<i64> = honor_rows.into_iter().map(|(id,)| id).collect();

        // 5. 批量计算所有球队年度积分排名（窗口函数）
        let rank_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT id, RANK() OVER (ORDER BY annual_points DESC) as rank
               FROM teams WHERE save_id = ?"#
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_annual_ranks = HashMap::new();
        for r in &rank_rows {
            let id: i64 = r.get("id");
            let rank: i32 = r.try_get("rank").unwrap_or(99);
            team_annual_ranks.insert(id, rank);
        }

        // 5.5 上赛季排名（从夏季常规赛积分榜获取）
        let last_season_rank_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT ls.team_id, ls.rank
               FROM league_standings ls
               JOIN tournaments t ON ls.tournament_id = t.id
               WHERE t.save_id = ? AND t.season_id = ?
               AND (t.tournament_type = 'SummerRegular' OR t.tournament_type = 'SpringRegular')
               ORDER BY t.tournament_type DESC"#
        )
        .bind(save_id)
        .bind(season_id - 1)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_last_season_ranks: HashMap<i64, i32> = HashMap::new();
        for r in &last_season_rank_rows {
            let team_id: i64 = r.get("team_id");
            let rank: i32 = r.try_get("rank").unwrap_or(99);
            team_last_season_ranks.entry(team_id).or_insert(rank);
        }

        // 6. 批量计算球队声望（简化版：基于荣誉+近期积分）
        let mut team_reputations = HashMap::new();
        for &team_id in team_names.keys() {
            // 历史荣誉声望
            let honor_rows: Vec<(String,)> = sqlx::query_as(
                "SELECT honor_type FROM honors WHERE team_id = ? AND save_id = ?"
            )
            .bind(team_id)
            .bind(save_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let mut historical: i64 = 0;
            for (honor_type,) in &honor_rows {
                historical += match honor_type.as_str() {
                    "TeamChampion" => 20,
                    "TeamRunnerUp" => 10,
                    "TeamThird" => 5,
                    "TeamFourth" => 3,
                    _ => 0,
                };
            }
            historical = historical.min(100);

            // 近期积分声望
            let recent_points: Option<(i64,)> = sqlx::query_as(
                r#"SELECT COALESCE(SUM(points), 0)
                   FROM annual_points_detail
                   WHERE team_id = ? AND save_id = ? AND season_id > ? AND season_id <= ?"#
            )
            .bind(team_id)
            .bind(save_id)
            .bind(season_id - 3)
            .bind(season_id)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

            let recent = recent_points
                .map(|(pts,)| (pts as f64 / 200.0 * 100.0).min(100.0) as i64)
                .unwrap_or(30);

            let overall = (historical as f64 * 0.4 + recent as f64 * 0.6).min(100.0) as i64;
            team_reputations.insert(team_id, overall);
        }

        Ok(Self {
            team_names,
            team_balances,
            team_region_ids,
            team_rosters,
            team_personalities,
            player_recent_honors,
            team_annual_ranks,
            team_last_season_ranks,
            team_reputations,
        })
    }

    /// 获取球队名称
    pub fn get_team_name(&self, team_id: i64) -> String {
        self.team_names.get(&team_id).cloned().unwrap_or_default()
    }

    /// 获取球队阵容
    pub fn get_roster(&self, team_id: i64) -> Vec<CachedPlayer> {
        self.team_rosters.get(&team_id).cloned().unwrap_or_default()
    }

    /// 获取球队AI性格权重
    pub fn get_weights(&self, team_id: i64) -> AIDecisionWeights {
        self.team_personalities
            .get(&team_id)
            .map(|p| p.get_weights())
            .unwrap_or_default()
    }

    /// 检查选手是否有近期荣誉
    pub fn has_recent_honor(&self, player_id: i64) -> bool {
        self.player_recent_honors.contains(&player_id)
    }

    /// 获取球队年度排名
    pub fn get_team_rank(&self, team_id: i64) -> i32 {
        self.team_annual_ranks.get(&team_id).copied().unwrap_or(99)
    }

    pub fn get_team_last_rank(&self, team_id: i64) -> i32 {
        self.team_last_season_ranks.get(&team_id).copied()
            .unwrap_or_else(|| self.get_team_rank(team_id))
    }

    /// 获取球队综合声望（0-100）
    pub fn get_team_reputation(&self, team_id: i64) -> i64 {
        *self.team_reputations.get(&team_id).unwrap_or(&30)
    }

    /// 转会后更新缓存：将选手从旧队移到新队
    pub fn transfer_player(&mut self, player_id: i64, from_team_id: Option<i64>, to_team_id: Option<i64>, updates: Option<PlayerCacheUpdate>) {
        let mut player = None;

        // 从旧队移除
        if let Some(from_id) = from_team_id {
            if let Some(roster) = self.team_rosters.get_mut(&from_id) {
                if let Some(pos) = roster.iter().position(|p| p.id == player_id) {
                    player = Some(roster.remove(pos));
                }
            }
        }

        // 添加到新队
        if let Some(to_id) = to_team_id {
            if let Some(mut p) = player {
                p.team_id = Some(to_id);
                if let Some(upd) = updates {
                    if let Some(s) = upd.salary { p.salary = s; }
                    if let Some(l) = upd.loyalty { p.loyalty = l; }
                    if let Some(sat) = upd.satisfaction { p.satisfaction = sat; }
                    if let Some(ces) = upd.contract_end_season { p.contract_end_season = Some(ces); }
                }
                self.team_rosters.entry(to_id).or_default().push(p);
            }
        }
    }

    /// 更新球队余额缓存
    pub fn update_balance(&mut self, team_id: i64, delta: i64) {
        if let Some(balance) = self.team_balances.get_mut(&team_id) {
            *balance += delta;
        }
    }

    /// 将选手标记为退役（从阵容中移除）
    pub fn retire_player(&mut self, player_id: i64, team_id: Option<i64>) {
        if let Some(tid) = team_id {
            if let Some(roster) = self.team_rosters.get_mut(&tid) {
                roster.retain(|p| p.id != player_id);
            }
        }
    }

    /// 更新选手属性（年龄/能力）
    pub fn update_player_stats(&mut self, player_id: i64, team_id: Option<i64>, new_age: i64, new_ability: i64) {
        if let Some(tid) = team_id {
            if let Some(roster) = self.team_rosters.get_mut(&tid) {
                if let Some(p) = roster.iter_mut().find(|p| p.id == player_id) {
                    p.age = new_age;
                    p.ability = new_ability;
                }
            }
        }
    }

    /// 释放选手（从队伍移除，变为自由球员）
    pub fn release_player(&mut self, player_id: i64, team_id: i64) {
        if let Some(roster) = self.team_rosters.get_mut(&team_id) {
            roster.retain(|p| p.id != player_id);
        }
    }
}

/// 选手缓存更新参数
pub struct PlayerCacheUpdate {
    pub salary: Option<i64>,
    pub loyalty: Option<i64>,
    pub satisfaction: Option<i64>,
    pub contract_end_season: Option<i64>,
}

/// 概率取整：2.7 → 70%概率得3，30%概率得2
fn probabilistic_round(value: f64, rng: &mut impl Rng) -> i64 {
    let floor = value.floor() as i64;
    let frac = value - value.floor();
    if frac > 0.0 && rng.gen::<f64>() < frac {
        floor + 1
    } else {
        floor
    }
}

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

    async fn record_financial_transaction(
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
        let window = self.get_window(pool, window_id).await?;
        let save_id = &window.save_id;

        // 构建缓存（替代每轮数百次单独查询）
        let mut cache = TransferCache::build(pool, save_id, window.season_id).await?;

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

        // 更新转会期轮次（不再自动标记 COMPLETED，需要手动确认关闭）
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

    // ============================================
    // 第1轮：赛季结算
    // ============================================

    async fn execute_season_settlement(
        &self,
        pool: &Pool<Sqlite>,
        window_id: i64,
        save_id: &str,
        season_id: i64,
        cache: &mut TransferCache,
    ) -> Result<RoundResult, String> {
        let mut events = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();

        // === 预加载：赛季统计数据（表现关联成长 + 潜力微漂移） ===
        let stats_rows = sqlx::query(
            "SELECT player_id, games_played, avg_performance \
             FROM player_season_stats WHERE save_id = ? AND season_id = ?"
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询赛季统计失败: {}", e))?;

        let stats_map: HashMap<i64, (i32, f64)> = stats_rows.iter()
            .map(|row| {
                let pid: i64 = row.get("player_id");
                let gp: i32 = row.get("games_played");
                let avg_perf: f64 = row.get("avg_performance");
                (pid, (gp, avg_perf))
            })
            .collect();

        // === 预加载：选手特性（成长类特性判定） ===
        let all_player_ids: Vec<i64> = cache.team_rosters.values()
            .flat_map(|roster| roster.iter().map(|p| p.id))
            .collect();

        let mut traits_map: HashMap<i64, Vec<TraitType>> = HashMap::new();
        if !all_player_ids.is_empty() {
            let placeholders: String = all_player_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!(
                "SELECT player_id, trait_type FROM player_traits WHERE save_id = ? AND player_id IN ({})",
                placeholders
            );
            let mut query = sqlx::query(&query_str).bind(save_id);
            for pid in &all_player_ids {
                query = query.bind(pid);
            }
            let trait_rows = query.fetch_all(pool).await
                .map_err(|e| format!("查询特性失败: {}", e))?;

            for row in &trait_rows {
                let pid: i64 = row.get("player_id");
                let trait_str: String = row.get("trait_type");
                if let Some(tt) = TraitType::from_str(&trait_str) {
                    traits_map.entry(pid).or_default().push(tt);
                }
            }
        }

        // === 遍历所有选手，执行成长/衰退/退役 ===
        let all_players: Vec<CachedPlayer> = cache.team_rosters.values()
            .flat_map(|roster| roster.iter().cloned())
            .collect();

        for player in &all_players {
            let player_id = player.id;
            let age = player.age;
            let ability = player.ability;
            let potential = player.potential;
            let tag = &player.tag;
            let game_id = &player.game_id;
            let team_id = player.team_id;

            let new_age = age + 1;

            // 获取选手特性
            let player_traits = traits_map.get(&player_id).cloned().unwrap_or_default();
            let has_late_blocker = player_traits.contains(&TraitType::LateBlocker);
            let has_prodigy = player_traits.contains(&TraitType::Prodigy);
            let has_resilient = player_traits.contains(&TraitType::Resilient);
            let has_glass_cannon = player_traits.contains(&TraitType::GlassCannon);

            // 成长上限年龄（LateBlocker 延长2年）
            let growth_cap: i64 = if has_late_blocker { 32 } else { 30 };
            // 用于年龄系数查表的等效年龄
            let effective_age = if has_late_blocker { new_age - 2 } else { new_age };

            let mut perf_desc = String::new();

            let new_ability = if new_age <= growth_cap && ability < potential {
                // ========== 成长期 ==========

                // ① 随机基础成长 (A)
                let base_growth: i64 = match tag.to_uppercase().as_str() {
                    "GENIUS" => rng.gen_range(2..=4),
                    "NORMAL" => rng.gen_range(1..=3),
                    _ => rng.gen_range(0..=2), // ORDINARY
                };

                // ② 突破/停滞事件 (A) — 互斥，10%总事件率
                let event_roll: f64 = rng.gen();
                let base_growth = if event_roll < 0.05 {
                    perf_desc = "突破赛季".to_string();
                    base_growth + 1
                } else if event_roll < 0.10 {
                    perf_desc = "停滞赛季".to_string();
                    0
                } else {
                    base_growth
                };

                // ③ 年龄系数 (B) — 平滑渐变替代30岁硬截断
                let age_coeff: f64 = match effective_age {
                    0..=24 => 1.0,
                    25..=26 => 0.7,
                    27..=28 => 0.4,
                    29..=30 => 0.15,
                    _ => 0.0, // LateBlocker 31-32 也用 effective_age-2
                };

                // ④ 神童特性修正 (F)
                let prodigy_mod = if has_prodigy {
                    if new_age <= 20 { 1.5 } else if new_age >= 25 { 0.8 } else { 1.0 }
                } else {
                    1.0
                };

                // ⑤ 年龄衰减后成长 = probabilistic_round(base × 系数)
                let growth_after_age = probabilistic_round(
                    base_growth as f64 * age_coeff * prodigy_mod, &mut rng
                );

                // ⑥ 表现加成 (D) — 基于赛季统计
                let perf_bonus = match stats_map.get(&player_id) {
                    Some(&(gp, avg_perf)) if gp >= 20 && avg_perf > ability as f64 + 5.0 => {
                        if perf_desc.is_empty() { perf_desc = "超常发挥".to_string(); }
                        else { perf_desc.push_str("+超常发挥"); }
                        1i64
                    }
                    Some(&(gp, avg_perf)) if gp >= 20 && avg_perf > ability as f64 => {
                        if rng.gen_bool(0.5) {
                            if perf_desc.is_empty() { perf_desc = "突破成长".to_string(); }
                            else { perf_desc.push_str("+突破成长"); }
                            1
                        } else { 0 }
                    }
                    Some(&(gp, _)) if gp == 0 => {
                        if perf_desc.is_empty() { perf_desc = "缺乏实战".to_string(); }
                        else { perf_desc.push_str("+缺乏实战"); }
                        // 缺乏实战：成长减半（向上取整保留最低1点成长机会）
                        -((growth_after_age + 1) / 2)
                    }
                    Some(&(gp, avg_perf)) if gp > 0 && avg_perf < (ability as f64) - 5.0 => {
                        if perf_desc.is_empty() { perf_desc = "表现低迷".to_string(); }
                        else { perf_desc.push_str("+表现低迷"); }
                        -1
                    }
                    _ => 0,
                };

                // ⑦ 最终成长值
                let final_growth = (growth_after_age + perf_bonus).max(0);
                (ability + final_growth).min(potential).min(100)

            } else if new_age > growth_cap {
                // ========== 衰退期 ==========

                // ① 基础衰退率 (B) — 渐进式衰退
                let base_decline: f64 = match effective_age {
                    0..=30 => 0.0,
                    31 => 0.5,
                    32..=33 => 1.0,
                    34..=35 => 1.5,
                    _ => 2.0, // 36+
                };

                // ② 标签系数 (C) — 天才衰退慢，平庸衰退快
                let tag_decay = match tag.to_uppercase().as_str() {
                    "GENIUS" => 0.7,
                    "ORDINARY" => 1.2,
                    _ => 1.0, // NORMAL
                };

                // ③ 特性修正 (F) — Resilient 减缓，GlassCannon 加速
                let trait_decay = if has_resilient { 0.5 }
                    else if has_glass_cannon { 1.5 }
                    else { 1.0 };

                // ④ 概率取整
                let final_decline = probabilistic_round(
                    base_decline * tag_decay * trait_decay, &mut rng
                );
                (ability - final_decline).max(50)

            } else {
                // 已达潜力上限且在成长期，不成长也不衰退
                ability
            };

            // ========== 潜力微漂移 (E) ==========
            let mut new_potential = potential;
            if let Some(&(gp, avg_perf)) = stats_map.get(&player_id) {
                if gp >= 30 && avg_perf > ability as f64 + 5.0 && rng.gen_bool(0.08) {
                    new_potential = (potential + 1).min(100);
                    if perf_desc.is_empty() { perf_desc = "潜力↑".to_string(); }
                    else { perf_desc.push_str("+潜力↑"); }
                } else if gp >= 20 && avg_perf < (ability as f64) - 5.0
                    && new_age > 28 && rng.gen_bool(0.12) {
                    new_potential = (potential - 1).max(50);
                    if perf_desc.is_empty() { perf_desc = "潜力↓".to_string(); }
                    else { perf_desc.push_str("+潜力↓"); }
                }
            }

            // ========== 更新数据库 ==========
            sqlx::query(
                "UPDATE players SET age = ?, ability = ?, potential = ? WHERE id = ? AND save_id = ?"
            )
            .bind(new_age)
            .bind(new_ability)
            .bind(new_potential)
            .bind(player_id)
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新选手年龄/能力失败: {}", e))?;

            // 更新缓存
            cache.update_player_stats(player_id, team_id, new_age, new_ability);

            // ========== 事件记录 ==========
            let ability_change = new_ability - ability;
            let change_desc = if ability_change > 0 {
                if !perf_desc.is_empty() {
                    format!("+{}({})", ability_change, perf_desc)
                } else {
                    format!("+{}", ability_change)
                }
            } else if ability_change < 0 {
                if !perf_desc.is_empty() {
                    format!("{}({})", ability_change, perf_desc)
                } else {
                    format!("{}", ability_change)
                }
            } else if !perf_desc.is_empty() {
                format!("不变({})", perf_desc)
            } else {
                "不变".to_string()
            };

            let potential_desc = if new_potential != potential {
                format!("，潜力 {} → {}", potential, new_potential)
            } else {
                String::new()
            };

            let from_team_name = if let Some(tid) = team_id {
                cache.get_team_name(tid)
            } else {
                "自由球员".to_string()
            };

            let level = if ability_change >= 3 {
                EventLevel::A
            } else if ability_change >= 2 {
                EventLevel::B
            } else {
                EventLevel::C
            };

            let event = self.record_event(
                pool, window_id, 1,
                TransferEventType::SeasonSettlement,
                level,
                player_id, game_id, new_ability,
                team_id, if from_team_name.is_empty() { None } else { Some(&from_team_name) },
                team_id, if from_team_name.is_empty() { None } else { Some(&from_team_name) },
                0, 0, 0,
                &format!("年龄 {} → {}，能力 {} → {} ({}){}", age, new_age, ability, new_ability, change_desc, potential_desc),
            ).await?;
            events.push(event);

            // ========== 退役判定 (G) — 概率制 ==========
            let retire_chance: f64 = if new_age >= 37 {
                1.0
            } else if new_age >= 35 && new_ability < 50 {
                0.8
            } else if new_age >= 35 && new_ability < 60 {
                0.5
            } else if new_age >= 33 && new_ability < 55 {
                0.2
            } else {
                0.0
            };

            if retire_chance > 0.0 && rng.gen::<f64>() < retire_chance {
                sqlx::query(
                    "UPDATE players SET status = 'RETIRED', team_id = NULL WHERE id = ? AND save_id = ?"
                )
                .bind(player_id)
                .bind(save_id)
                .execute(pool)
                .await
                .map_err(|e| format!("更新退役状态失败: {}", e))?;

                cache.retire_player(player_id, team_id);

                let retire_desc = if retire_chance < 1.0 {
                    format!("{}岁退役，能力值{}（退役概率{}%）", new_age, new_ability, (retire_chance * 100.0) as i32)
                } else {
                    format!("{}岁退役，能力值{}", new_age, new_ability)
                };

                let event = self.record_event(
                    pool, window_id, 1,
                    TransferEventType::PlayerRetirement,
                    EventLevel::from_ability_and_fee(new_ability as u8, 0),
                    player_id, game_id, new_ability as i64,
                    team_id, if from_team_name.is_empty() { None } else { Some(&from_team_name) },
                    None, None,
                    0, 0, 0,
                    &retire_desc,
                ).await?;
                events.push(event);
            }
        }

        // 更新所有球队战力
        self.recalculate_team_powers_optimized(pool, save_id).await?;

        Ok(RoundResult {
            round: 1,
            round_name: "赛季结算".to_string(),
            events,
            summary: "已完成赛季结算：选手年龄+1、能力值更新（含表现加成/年龄曲线/特性影响）、潜力微调、退役处理".to_string(),
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

            let team_rank = cache.get_team_rank(team_id);
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
                    &format!("续约谈判失败，{}成为自由球员", game_id),
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

    async fn execute_bidirectional_evaluation(
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
                let player_eval = self.evaluate_player_cached(
                    pool, save_id, window_id, player.id, &player.game_id,
                    team_id, &team_name, &team_eval,
                    player.ability, player.age, player.salary, player.satisfaction, player.loyalty, &player.position,
                    &roster, season_id, cache
                ).await?;

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

    /// 计算战绩稳定性评分
    #[allow(overlapping_range_endpoints)]
    fn calculate_stability_score(&self, current_rank: i32, last_rank: i32) -> i32 {
        let change = current_rank - last_rank;  // 正数=下滑

        match (last_rank, change) {
            // 卫冕冠军/亚军
            (1, -1..=1) => 100,      // 冠军→冠亚军：极稳定
            (1, 2..=3) => 70,        // 冠军→3-4名：可接受
            (1, 4..) => 30,          // 冠军→5名开外：危机
            (2, -2..=1) => 95,       // 亚军维持：稳定

            // 上赛季前4
            (3..=4, ..=-1) => 95,    // 进步：稳定
            (3..=4, 0..=2) => 85,    // 维持：稳定
            (3..=4, 3..=5) => 55,    // 下滑：警惕
            (3..=4, 6..) => 30,      // 大幅下滑：必须调整

            // 上赛季5-8名
            (5..=8, ..=-3) => 95,    // 大幅上升：稳定
            (5..=8, -2..=2) => 80,   // 维持：稳定
            (5..=8, 3..) => 50,      // 下滑：考虑调整

            // 上赛季9-14名（中下游队伍）
            (9..=14, ..=-4) => 95,   // 大幅进步：稳定
            (9..=14, -3..=-1) => 85, // 进步：稳定
            (9..=14, 0..=2) => 75,   // 维持：基本稳定
            (9..=14, 3..) => 45,     // 下滑：考虑调整

            // 其他情况
            (_, ..=-3) => 90,        // 大幅进步
            (_, -2..=-1) => 80,      // 进步
            (_, 0..=1) => 70,        // 维持
            (_, 2..) => 40,          // 下滑
        }
    }

    /// 决定战队策略
    fn determine_team_strategy(&self, stability_score: i32, current_rank: i32, roster_power: f64, roster_age_avg: f64)
        -> (String, String, String)
    {
        let (strategy, urgency, reason) = if stability_score >= 90 {
            ("DYNASTY", "NONE", format!("战绩稳定，排名{}，无需变动", current_rank))
        } else if stability_score >= 70 {
            ("MAINTAIN", "LOW", format!("战绩尚可，排名{}，可小幅调整", current_rank))
        } else if stability_score >= 40 {
            if roster_age_avg > 26.0 {
                ("UPGRADE", "MEDIUM", format!("战绩下滑且阵容老化，平均年龄{:.1}岁，需要补强", roster_age_avg))
            } else {
                ("UPGRADE", "MEDIUM", format!("战绩下滑，排名{}，需要补强", current_rank))
            }
        } else {
            if roster_power < 75.0 {
                ("REBUILD", "HIGH", format!("战绩大幅下滑，阵容战力{:.1}偏低，需要重建", roster_power))
            } else {
                ("REBUILD", "HIGH", format!("战绩大幅下滑，排名从前列跌落，需要大幅调整"))
            }
        };

        (strategy.to_string(), urgency.to_string(), reason)
    }

    /// 战队评估（使用缓存版本）
    async fn evaluate_team_cached(
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
        let current_rank = cache.get_team_rank(team_id);
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
    async fn generate_position_needs_cached(
        &self,
        pool: &Pool<Sqlite>,
        evaluation_id: i64,
        roster: &[CachedPlayer],
        strategy: &str,
        roster_power: f64,
        budget: i64,
    ) -> Result<(), String> {
        let positions = ["TOP", "JUG", "MID", "ADC", "SUP"];

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
        }

        Ok(())
    }

    /// 选手评估（使用缓存版本）
    async fn evaluate_player_cached(
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
        let mut stay_score: f64 = 50.0;

        // 1. 战队排名评分
        let team_rank_score = match team_eval.current_rank {
            1..=4 => 20.0,
            5..=8 => 10.0,
            9..=12 => -5.0,
            _ => -15.0,
        };
        stay_score += team_rank_score;

        // 2. 战绩趋势评分
        let rank_change = team_eval.last_rank - team_eval.current_rank;
        let team_trend_score = (rank_change as f64 * 3.0).clamp(-15.0, 15.0);
        stay_score += team_trend_score;

        // 3. 队友水平评分（使用缓存阵容）
        let teammate_avg: f64 = roster.iter()
            .filter(|p| p.id != player_id)
            .map(|p| p.ability as f64)
            .sum::<f64>() / (roster.len() - 1).max(1) as f64;

        let teammate_score = if ability > teammate_avg as i64 + 10 {
            -15.0
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
            -20.0
        } else if salary_ratio < 0.9 {
            -10.0
        } else if salary_ratio > 1.2 {
            15.0
        } else {
            0.0
        };
        stay_score += salary_score;

        // 5. 荣誉评分（使用缓存）
        let has_recent_honor = cache.has_recent_honor(player_id);
        let honor_score = if has_recent_honor {
            5.0
        } else if ability >= 61 && team_eval.current_rank > 8 {
            -10.0
        } else {
            0.0
        };
        stay_score += honor_score;

        // 6. 满意度评分
        let satisfaction_score = (satisfaction as f64 - 70.0) * 0.5;
        stay_score += satisfaction_score;

        // 7. 年龄因素
        if age >= 28 && team_eval.current_rank > 8 {
            stay_score -= 20.0;
        }

        // 8. 忠诚度加成
        stay_score += (loyalty as f64 - 70.0) * 0.3;

        let stay_score = stay_score.clamp(0.0, 100.0);
        let wants_to_leave = stay_score < 40.0;

        let leave_reason = if wants_to_leave {
            // 收集所有负面因素，按影响程度排序，取最严重的作为离队原因
            let mut factors: Vec<(f64, &str)> = Vec::new();

            // 薪资因素
            if salary_score <= -15.0 {
                factors.push((salary_score, "薪资被严重低估"));
            } else if salary_score < 0.0 {
                factors.push((salary_score, "对薪资待遇不满"));
            }

            // 战队排名因素
            if team_rank_score <= -10.0 {
                factors.push((team_rank_score, "战队战绩太差"));
            } else if team_rank_score < 0.0 {
                factors.push((team_rank_score, "战队缺乏竞争力"));
            }

            // 战绩趋势因素
            if team_trend_score <= -9.0 {
                factors.push((team_trend_score, "战队成绩大幅下滑"));
            } else if team_trend_score < -3.0 {
                factors.push((team_trend_score, "战队近期状态下滑"));
            }

            // 队友水平因素
            if teammate_score < 0.0 {
                factors.push((teammate_score, "队友水平跟不上"));
            }

            // 荣誉渴望因素
            if honor_score < 0.0 {
                factors.push((honor_score, "渴望在强队证明自己"));
            }

            // 满意度因素
            if satisfaction_score <= -10.0 {
                factors.push((satisfaction_score, "对球队管理非常不满"));
            } else if satisfaction_score < -3.0 {
                factors.push((satisfaction_score, "对球队现状不太满意"));
            }

            // 年龄+弱队因素
            if age >= 28 && team_eval.current_rank > 8 {
                factors.push((-20.0, "想去强队冲击荣誉"));
            }

            // 忠诚度因素
            let loyalty_contribution = (loyalty as f64 - 70.0) * 0.3;
            if loyalty_contribution <= -6.0 {
                factors.push((loyalty_contribution, "缺乏归属感"));
            }

            // 按负面程度排序（最负面的排最前）
            factors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            factors.first().map_or("对现状不满意".to_string(), |(_, reason)| reason.to_string())
        } else {
            "".to_string()
        };

        // 战队评估选手是否应该挂牌（使用缓存版本）
        let (should_list, list_reason, protect_reason) = self.evaluate_player_for_listing_cached(
            ability, age, salary, team_eval, has_recent_honor, position, roster
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
    fn evaluate_player_for_listing_cached(
        &self,
        ability: i64,
        age: i64,
        salary: i64,
        team_eval: &TeamEvaluation,
        has_recent_honor: bool,
        _position: &str,
        roster: &[CachedPlayer],
    ) -> (bool, String, String) {
        // 荣誉保护
        if has_recent_honor && ability >= 58 {
            return (false, "".to_string(), "近2赛季有荣誉".to_string());
        }

        // 王朝模式几乎不挂牌
        if team_eval.strategy == "DYNASTY" {
            if ability < 60 || age >= 34 {
                return (true, "能力过低或年龄过大".to_string(), "".to_string());
            }
            return (false, "".to_string(), "战队处于王朝期".to_string());
        }

        // 核心选手保护
        if ability > team_eval.roster_power as i64 + 5 && ability >= 61 {
            return (false, "".to_string(), "核心选手".to_string());
        }

        // 维持模式
        if team_eval.strategy == "MAINTAIN" {
            if ability < 54 || (age >= 32 && ability < 58) {
                return (true, "能力不足".to_string(), "".to_string());
            }
            return (false, "".to_string(), "阵容稳定".to_string());
        }

        // 补强/重建模式
        let roster_count = roster.len() as i32;

        let value_ratio = if salary > 0 { ability as f64 / (salary as f64 / 10000.0) } else { 100.0 };
        if value_ratio < 0.40 && salary > 100_0000 {
            return (true, "高薪低能".to_string(), "".to_string());
        }

        if age >= 30 && ability < 78 {
            return (true, "年龄偏大且能力一般".to_string(), "".to_string());
        }

        if roster_count >= 7 && ability < team_eval.roster_power as i64 - 5 {
            return (true, "能力低于队伍均值".to_string(), "".to_string());
        }

        if ability < 51 {
            return (true, "能力过低".to_string(), "".to_string());
        }

        // 同位置超过2人时，挂牌该位置能力最低的选手
        let same_pos_players: Vec<&CachedPlayer> = roster.iter()
            .filter(|p| p.position == _position)
            .collect();
        if same_pos_players.len() >= 3 {
            let min_ability_at_pos = same_pos_players.iter().map(|p| p.ability).min().unwrap_or(0);
            if ability == min_ability_at_pos {
                return (true, format!("{}位置已有{}人，能力最低被挂牌", _position, same_pos_players.len()), "".to_string());
            }
        }

        // 阵容超出奢侈税起征线时，更积极挂牌非首发且无培养价值的选手
        let over_threshold = roster_count as i64 - self.config.luxury_tax_threshold;
        if over_threshold > 0 {
            // 找到该选手在同位置的排名
            let is_starter_level = roster.iter()
                .filter(|p| p.position == _position)
                .any(|p| p.ability <= ability && p.is_starter);

            // 不是首发水平
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

    async fn execute_free_agent_bidding(
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
            let position: String = free_agent.get("position");
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
                let balance = cache.team_balances.get(&team_id).copied().unwrap_or(0);

                // 检查转会次数限制（基础2个，空缺位置额外放宽）
                let count = team_transfer_counts.get(&team_id).copied().unwrap_or(0);
                let roster = cache.get_roster(team_id);
                let positions = ["TOP", "JUG", "MID", "ADC", "SUP"];
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

                // 超过奢侈税起征线时，AI大幅降低签人意愿（但不硬性禁止）
                let over_threshold = roster_count as i64 - self.config.luxury_tax_threshold;
                if over_threshold >= 5 {
                    // 超出5人以上，几乎不会再签人
                    continue;
                }

                // 检查位置需求
                let pos_count = roster.iter()
                    .filter(|r| r.position == position)
                    .count();

                if pos_count >= 2 {
                    continue;
                }

                // pos_count == 1 时，只在实力升级或培养新人时才报价
                if pos_count == 1 {
                    let best_ability_at_pos = roster.iter()
                        .filter(|r| r.position == position)
                        .map(|r| r.ability)
                        .max()
                        .unwrap_or(0);
                    let is_upgrade = ability > best_ability_at_pos;
                    let is_youth_prospect = age <= 23 && potential >= 70;
                    if !is_upgrade && !is_youth_prospect {
                        continue;
                    }
                }

                // 使用缓存获取AI性格权重
                let weights = cache.get_weights(team_id);
                let roster = cache.get_roster(team_id);
                let team_rank = cache.get_team_rank(team_id);

                // 计算匹配度和报价
                let match_score = self.calculate_match_score(
                    ability as u8, age as u8, &position, &weights, balance,
                    &roster, team_rank,
                    potential as u8, stability as u8, &tag,
                );

                // 超出奢侈税起征线时，降低匹配分数
                let match_score = if over_threshold > 0 {
                    match_score * (1.0 - over_threshold as f64 * 0.15)
                } else {
                    match_score
                };

                if match_score < 50.0 {
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
                let offered_salary = (expected_salary as f64 * salary_multiplier) as i64;
                let contract_years = {
                    let base: i64 = if age <= 22 { 3 } else if age <= 25 { 2 } else if age <= 28 { 2 } else { 1 };
                    let personality_adj: i64 = if weights.long_term_focus > 0.7 { 1 } else if weights.short_term_focus > 0.7 { -1 } else { 0 };
                    let random_adj: i64 = if rng.gen::<f64>() < 0.3 { 1 } else if rng.gen::<f64>() < 0.25 { -1 } else { 0 };
                    (base + personality_adj + random_adj).clamp(1, 4)
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
            let market_premium = if offer_count >= 3 {
                1.0 + (offer_count as f64 - 2.0) * 0.05  // 每多一个竞争者+5%
            } else {
                1.0
            };
            // 调整薪资比较基准（选手期望更高）
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
                let target_team_rank = cache.get_team_rank(offer.team_id);
                let target_team_reputation = cache.get_team_reputation(offer.team_id);
                let willingness = self.calculate_willingness(
                    ability as u8, loyalty as u8, age as u8,
                    offer.offered_salary, adjusted_expected_salary,
                    home_region_id, offer.target_region_id, region_loyalty,
                    target_team_rank, target_team_reputation,
                    &target_roster, &position,
                    &mut rng,
                );
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
                    loyalty: 50,
                    satisfaction: 60,
                    position: position.clone(),
                    tag: free_agent.try_get("tag").unwrap_or_else(|_| "NORMAL".to_string()),
                    team_id: Some(to_team_id),
                    is_starter: false,
                    home_region_id,
                    region_loyalty,
                    contract_end_season: Some(season_id + offer.contract_years),
                    status: "Active".to_string(),
                    stability: free_agent.try_get("stability").unwrap_or(60),
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
            let position: String = listing.get("position");
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
                    .filter(|r| r.position == position)
                    .count();

                let over_threshold = roster.len() as i64 - self.config.luxury_tax_threshold;
                if pos_count >= 2 || over_threshold >= 5 {
                    continue;
                }

                if pos_count == 1 {
                    let best_ability_at_pos = roster.iter()
                        .filter(|r| r.position == position)
                        .map(|r| r.ability)
                        .max()
                        .unwrap_or(0);
                    let is_upgrade = ability > best_ability_at_pos;
                    let is_youth_prospect = age <= 23 && potential >= 70;
                    if !is_upgrade && !is_youth_prospect {
                        continue;
                    }
                }

                // 使用缓存获取AI性格权重
                let weights = cache.get_weights(team_id);
                let team_rank = cache.get_team_rank(team_id);

                let match_score = self.calculate_match_score(
                    ability as u8, age as u8, &position, &weights, balance,
                    &roster, team_rank,
                    potential as u8, stability as u8, &tag,
                );

                // 超出奢侈税起征线时，降低匹配分数
                let match_score = if over_threshold > 0 {
                    match_score * (1.0 - over_threshold as f64 * 0.15)
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

            for (idx, bid) in all_bids.iter().enumerate() {
                let target_roster = cache.get_roster(bid.0);
                let target_team_rank = cache.get_team_rank(bid.0);
                let target_team_reputation = cache.get_team_reputation(bid.0);
                let willingness = self.calculate_willingness(
                    ability as u8, loyalty as u8, age as u8,
                    bid.3, salary,
                    home_region_id, bid.5, region_loyalty,
                    target_team_rank, target_team_reputation,
                    &target_roster, &position,
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

    async fn execute_financial_adjustment(
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

            let over_count = roster_count - self.config.luxury_tax_threshold;
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
                let position: String = candidate.get("position");
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

    async fn execute_final_remedy(
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

            // 检查每个位置是否有人（不能只看总人数，可能有8人但缺某个位置）
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
            let team_rank = cache.get_team_rank(team_id);
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
                        "UPDATE players SET team_id = ?, salary = ?, contract_end_season = ?, loyalty = 40, satisfaction = 50 WHERE id = ?"
                    )
                    .bind(team_id)
                    .bind(salary)
                    .bind(season_id + contract_years)
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

            if active_count > 10 {
                issues.push(TransferCloseIssue {
                    team_id,
                    team_name: team_name.clone(),
                    issue_type: "ROSTER_TOO_LARGE".to_string(),
                    detail: format!("{}有{}名活跃选手，最多允许10名", team_name, active_count),
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
                | "EMERGENCY_SIGNING" | "PLAYER_RETIREMENT" | "PLAYER_RELEASE"
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

    /// 单条 SQL 更新所有球队战力（替代 N+1 循环查询）
    async fn recalculate_team_powers_optimized(&self, pool: &Pool<Sqlite>, save_id: &str) -> Result<(), String> {
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
    async fn insert_contract(
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

    /// 计算匹配度（0-100）
    fn calculate_match_score(
        &self,
        ability: u8,
        age: u8,
        position: &str,
        weights: &AIDecisionWeights,
        balance: i64,
        roster: &[CachedPlayer],
        team_rank: i32,
        potential: u8,
        stability: u8,
        tag: &str,
    ) -> f64 {
        // 1. 能力匹配（0-100）
        let ability_score = match ability {
            90..=100 => 100.0,
            80..=89 => 90.0,
            75..=79 => 80.0,
            70..=74 => 70.0,
            65..=69 => 60.0,
            60..=64 => 50.0,
            55..=59 => 35.0,
            50..=54 => 20.0,
            _ => 10.0,
        };

        // 2. 年龄匹配（0-100，根据性格偏好）
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

        // 3. 财务匹配（0-100，连续化：基于 balance 的对数映射）
        let finance_score = if balance <= 0 {
            0.0
        } else {
            // balance 单位是元，100万=1_000_000
            // ln(100万)≈13.8, ln(1000万)≈16.1, ln(5000万)≈17.7
            let log_balance = (balance as f64).ln();
            // 映射到 0-100：ln(100万)→30, ln(5000万)→100
            ((log_balance - 13.8) / (17.7 - 13.8) * 70.0 + 30.0).clamp(10.0, 100.0)
        };

        // 4. 位置需求度（0-100）
        let pos_players: Vec<&CachedPlayer> = roster.iter()
            .filter(|p| p.position == position)
            .collect();
        let pos_count = pos_players.len();
        let need_score = match pos_count {
            0 => 100.0,   // 该位置空缺，急需
            1 => 40.0,    // 已有首发，仅轻度需求
            2 => 15.0,    // 饱和
            _ => 5.0,     // 超饱和
        };

        // 5. 提升度（0-100）：选手能力相对于球队该位置最强选手的提升
        let best_at_pos = pos_players.iter()
            .map(|p| p.ability)
            .max()
            .unwrap_or(0);
        let upgrade_score = if pos_count == 0 {
            // 空位，能力直接映射
            (ability as f64).clamp(40.0, 100.0)
        } else {
            let diff = ability as i64 - best_at_pos;
            match diff {
                d if d >= 10 => 100.0,   // 大幅提升
                d if d >= 5 => 85.0,     // 明显提升
                d if d >= 0 => 65.0,     // 略有提升或持平
                d if d >= -5 => 45.0,    // 略弱于现有
                _ => 25.0,               // 明显弱于现有
            }
        };

        // 6. 排名因子（弱队更渴望强援）
        let rank_factor = match team_rank {
            1..=3 => 0.9,     // 强队，选人更挑剔
            4..=7 => 1.0,     // 中游
            8..=10 => 1.05,   // 中下游，更积极
            11..=14 => 1.1,   // 弱队，急需补强
            _ => 1.0,
        };

        // 7. 潜力因素（0-100）：23岁以下更看重潜力
        let potential_score = if age <= 23 {
            match potential {
                80..=100 => 100.0,
                70..=79 => 80.0,
                60..=69 => 60.0,
                _ => 40.0,
            }
        } else {
            match potential {
                80..=100 => 80.0,
                70..=79 => 65.0,
                _ => 50.0,
            }
        };

        // 8. 稳定性因素（0-100）
        let stability_score = match stability {
            80..=100 => 100.0,
            65..=79 => 80.0,
            50..=64 => 60.0,
            _ => 40.0,
        };

        // 9. 成长标签乘数
        let tag_multiplier = match tag {
            "GENIUS" | "Genius" => 1.08,
            "NORMAL" | "Normal" => 1.0,
            "ORDINARY" | "Ordinary" => 0.95,
            _ => 1.0,
        };

        // 根据 AI 性格动态调整各项权重比例
        let w_ability = 0.25 + 0.15 * weights.short_term_focus;      // 0.25 ~ 0.40
        let w_age = 0.15 + 0.15 * weights.youth_preference.max(weights.short_term_focus); // 0.15 ~ 0.30
        let w_finance = 0.10 + 0.10 * weights.bargain_hunting;       // 0.10 ~ 0.20
        let w_need = 0.20;                                            // 固定 0.20
        let w_upgrade = 0.15 + 0.10 * weights.short_term_focus;      // 0.15 ~ 0.25
        // 潜力权重受AI性格影响：发展型球队更看重潜力
        let w_potential = 0.05 + 0.10 * weights.youth_preference;     // 0.05 ~ 0.15
        // 稳定性权重受AI性格影响：保守型球队更看重稳定性
        let w_stability = 0.05 + 0.05 * (1.0 - weights.risk_tolerance); // 0.05 ~ 0.10
        let total_w = w_ability + w_age + w_finance + w_need + w_upgrade + w_potential + w_stability;

        // 归一化后加权求和，再乘以排名因子和成长标签乘数
        let raw = (ability_score * w_ability
            + age_score * w_age
            + finance_score * w_finance
            + need_score * w_need
            + upgrade_score * w_upgrade
            + potential_score * w_potential
            + stability_score * w_stability) / total_w;

        (raw * rank_factor * tag_multiplier).clamp(0.0, 100.0)
    }

    /// 写入一条竞价记录到 transfer_bids 表
    async fn insert_bid(
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

    /// 计算球员转会意愿（0-100）
    /// 8 因素 + 年龄优先级权重系统
    fn calculate_willingness(
        &self,
        ability: u8,
        loyalty: u8,
        age: u8,
        offered_salary: i64,
        current_salary: i64,
        home_region_id: Option<i64>,
        target_region_id: Option<i64>,
        region_loyalty: i64,
        target_team_rank: i32,
        target_team_reputation: i64,
        target_roster: &[CachedPlayer],
        position: &str,
        rng: &mut impl Rng,
    ) -> f64 {
        // 1. 薪资满意度（20-100）
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

        // 2. 球队竞争力（20-100）：基于目标球队排名
        let competitiveness_score = match target_team_rank {
            1..=3 => 100.0,
            4..=6 => 80.0,
            7..=10 => 60.0,
            11..=14 => 40.0,
            _ => 20.0,
        };

        // 3. 首发机会（30-100）：比较自己能力 vs 目标队该位置首发能力
        let best_at_pos = target_roster.iter()
            .filter(|p| p.position == position)
            .map(|p| p.ability)
            .max()
            .unwrap_or(0);
        let starting_chance_score = if best_at_pos == 0 {
            100.0  // 该位置空缺，必定首发
        } else {
            let diff = ability as i64 - best_at_pos;
            if diff >= 5 { 100.0 }       // 明显更强
            else if diff >= 0 { 85.0 }   // 略强或持平
            else if diff >= -5 { 70.0 }   // 略弱，有竞争
            else { 30.0 }                 // 明显更弱
        };

        // 4. 球队声望（20-100）：基于 target_team_reputation 线性映射
        let reputation_score = (target_team_reputation as f64 / 100.0 * 80.0 + 20.0).clamp(20.0, 100.0);

        // 5. 队友质量（30-100）：目标队平均能力映射
        let avg_ability = if target_roster.is_empty() {
            50.0
        } else {
            target_roster.iter().map(|p| p.ability as f64).sum::<f64>() / target_roster.len() as f64
        };
        let teammate_quality_score = if avg_ability >= 70.0 { 100.0 }
            else if avg_ability >= 65.0 { 80.0 }
            else if avg_ability >= 60.0 { 65.0 }
            else { 40.0 };

        // 6. 忠诚影响（0-50）
        let loyalty_factor = (100.0 - loyalty as f64) * 0.5;

        // 7. 发展空间（30-100）：仅对年轻选手有效
        let development_score = if age <= 23 {
            // 检查目标队是否有高能力同位置老将可学习
            let has_mentor = target_roster.iter().any(|p| {
                p.position == position && p.age >= 26 && p.ability >= 70
            });
            let team_avg_high = avg_ability >= 65.0;
            if has_mentor && team_avg_high { 100.0 }
            else if has_mentor || team_avg_high { 75.0 }
            else { 45.0 }
        } else {
            50.0  // 非年轻选手，发展空间中性
        };

        // 8. 随机波动（-8 ~ +8）
        let random_noise: f64 = rng.gen_range(-8.0..8.0);

        // 年龄优先级权重系统
        let (w_salary, w_compete, w_start, w_reputation, w_teammate, w_develop) = match age {
            17..=21 => (0.10, 0.10, 0.25, 0.10, 0.10, 0.20),  // 新秀期
            22..=25 => (0.15, 0.20, 0.20, 0.10, 0.10, 0.10),  // 成长期
            26..=28 => (0.20, 0.30, 0.10, 0.15, 0.10, 0.00),  // 巅峰期
            29..=31 => (0.35, 0.15, 0.10, 0.15, 0.10, 0.00),  // 老将期
            _       => (0.40, 0.10, 0.10, 0.15, 0.10, 0.00),  // 退役前
        };
        // 忠诚影响固定 0.15 权重
        let w_loyalty = 0.15;

        let weighted_score = salary_score * w_salary
            + competitiveness_score * w_compete
            + starting_chance_score * w_start
            + reputation_score * w_reputation
            + teammate_quality_score * w_teammate
            + development_score * w_develop
            + loyalty_factor * w_loyalty
            + random_noise;

        let base_willingness = weighted_score.clamp(0.0, 100.0);

        // 跨赛区惩罚
        let cross_region_factor = match (home_region_id, target_region_id) {
            (Some(home), Some(target)) if home != target => {
                (100.0 - region_loyalty as f64) / 100.0
            }
            _ => 1.0
        };

        (base_willingness * cross_region_factor).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    // ==================== Helper ====================

    fn seeded_rng(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }

    fn make_engine() -> TransferEngine {
        TransferEngine::new()
    }

    fn make_cached_player(id: i64, ability: i64, age: i64, position: &str) -> CachedPlayer {
        CachedPlayer {
            id,
            game_id: format!("Player{}", id),
            ability,
            potential: 80,
            age,
            salary: 5_000_000,
            loyalty: 50,
            satisfaction: 60,
            position: position.to_string(),
            tag: "NORMAL".to_string(),
            team_id: Some(1),
            is_starter: true,
            home_region_id: Some(1),
            region_loyalty: 70,
            contract_end_season: Some(3),
            status: "Active".to_string(),
            stability: 70,
        }
    }

    fn empty_cache() -> TransferCache {
        TransferCache {
            team_names: HashMap::new(),
            team_balances: HashMap::new(),
            team_region_ids: HashMap::new(),
            team_rosters: HashMap::new(),
            team_personalities: HashMap::new(),
            player_recent_honors: HashSet::new(),
            team_annual_ranks: HashMap::new(),
            team_last_season_ranks: HashMap::new(),
            team_reputations: HashMap::new(),
        }
    }

    // ==================== probabilistic_round ====================

    #[test]
    fn test_probabilistic_round_integer_returns_self() {
        let mut rng = seeded_rng(42);
        assert_eq!(probabilistic_round(3.0, &mut rng), 3);
        assert_eq!(probabilistic_round(0.0, &mut rng), 0);
        assert_eq!(probabilistic_round(-2.0, &mut rng), -2);
    }

    #[test]
    fn test_probabilistic_round_fractional_averages_correctly() {
        let mut rng = seeded_rng(42);
        let n = 10_000;
        let sum: i64 = (0..n).map(|_| probabilistic_round(2.7, &mut rng)).collect::<Vec<_>>().iter().sum();
        let mean = sum as f64 / n as f64;
        // Mean should be close to 2.7 (within 0.1)
        assert!((mean - 2.7).abs() < 0.1, "mean was {}", mean);
    }

    #[test]
    fn test_probabilistic_round_only_returns_floor_or_ceil() {
        let mut rng = seeded_rng(123);
        for _ in 0..1000 {
            let result = probabilistic_round(4.3, &mut rng);
            assert!(result == 4 || result == 5, "got {}", result);
        }
    }

    // ==================== calculate_stability_score ====================

    #[test]
    fn test_stability_score_champion_stays_champion() {
        let engine = make_engine();
        // 冠军→冠军: change = 0
        assert_eq!(engine.calculate_stability_score(1, 1), 100);
    }

    #[test]
    fn test_stability_score_champion_drops_badly() {
        let engine = make_engine();
        // 冠军→6名: change = 5 (≥4)
        assert_eq!(engine.calculate_stability_score(6, 1), 30);
    }

    #[test]
    fn test_stability_score_midtable_improves() {
        let engine = make_engine();
        // 8名→5名: change = -3 (大幅上升)
        assert_eq!(engine.calculate_stability_score(5, 8), 95);
    }

    #[test]
    fn test_stability_score_bottom_keeps_falling() {
        let engine = make_engine();
        // 10名→13名: change = 3 (下滑)
        assert_eq!(engine.calculate_stability_score(13, 10), 45);
    }

    #[test]
    fn test_stability_score_runner_up_maintains() {
        let engine = make_engine();
        // 亚军→冠军: change = -1
        assert_eq!(engine.calculate_stability_score(1, 2), 95);
    }

    // ==================== determine_team_strategy ====================

    #[test]
    fn test_strategy_dynasty_when_stable() {
        let engine = make_engine();
        let (strategy, urgency, _) = engine.determine_team_strategy(95, 1, 80.0, 24.0);
        assert_eq!(strategy, "DYNASTY");
        assert_eq!(urgency, "NONE");
    }

    #[test]
    fn test_strategy_upgrade_aging_roster() {
        let engine = make_engine();
        let (strategy, urgency, reason) = engine.determine_team_strategy(50, 8, 75.0, 27.5);
        assert_eq!(strategy, "UPGRADE");
        assert_eq!(urgency, "MEDIUM");
        assert!(reason.contains("老化"), "reason: {}", reason);
    }

    #[test]
    fn test_strategy_rebuild_when_catastrophic() {
        let engine = make_engine();
        let (strategy, urgency, _) = engine.determine_team_strategy(30, 14, 60.0, 25.0);
        assert_eq!(strategy, "REBUILD");
        assert_eq!(urgency, "HIGH");
    }

    #[test]
    fn test_strategy_maintain_when_decent() {
        let engine = make_engine();
        let (strategy, urgency, _) = engine.determine_team_strategy(75, 4, 78.0, 25.0);
        assert_eq!(strategy, "MAINTAIN");
        assert_eq!(urgency, "LOW");
    }

    // ==================== calculate_willingness ====================

    #[test]
    fn test_willingness_same_region_high_offer() {
        let engine = make_engine();
        let mut rng = seeded_rng(42);
        let roster = vec![
            make_cached_player(10, 70, 25, "MID"),
            make_cached_player(11, 68, 24, "TOP"),
        ];
        let w = engine.calculate_willingness(
            75,          // ability
            50,          // loyalty
            26,          // age (巅峰期)
            8_000_000,   // offered_salary (高于当前)
            5_000_000,   // current_salary
            Some(1),     // home_region_id
            Some(1),     // target_region_id (同赛区)
            70,          // region_loyalty
            2,           // target_team_rank (强队)
            80,          // target_team_reputation
            &roster,
            "JUG",
            &mut rng,
        );
        assert!(w > 60.0, "same region high offer should yield high willingness, got {}", w);
    }

    #[test]
    fn test_willingness_cross_region_penalty() {
        let engine = make_engine();
        let mut rng1 = seeded_rng(42);
        let mut rng2 = seeded_rng(42);
        let roster = vec![make_cached_player(10, 70, 25, "MID")];

        let same_region = engine.calculate_willingness(
            75, 50, 26, 7_000_000, 5_000_000,
            Some(1), Some(1), 85, 3, 70, &roster, "JUG", &mut rng1,
        );
        let cross_region = engine.calculate_willingness(
            75, 50, 26, 7_000_000, 5_000_000,
            Some(1), Some(2), 85, 3, 70, &roster, "JUG", &mut rng2,
        );
        assert!(
            cross_region < same_region,
            "cross-region ({}) should be less than same-region ({})",
            cross_region, same_region
        );
    }

    #[test]
    fn test_willingness_young_values_development() {
        let engine = make_engine();
        let mut rng1 = seeded_rng(42);
        let mut rng2 = seeded_rng(42);

        // Team with mentor (high-ability veteran at JUG) + strong avg ability
        // The young MID player benefits from development environment
        let roster_with_mentor = vec![
            make_cached_player(10, 78, 28, "JUG"),  // veteran mentor at different pos
            make_cached_player(11, 72, 25, "TOP"),
            make_cached_player(12, 70, 24, "ADC"),
            make_cached_player(13, 68, 23, "SUP"),
        ];
        // Weak team with no mentor, low avg ability
        let roster_no_mentor = vec![
            make_cached_player(10, 55, 20, "JUG"),
            make_cached_player(11, 52, 20, "TOP"),
            make_cached_player(12, 50, 19, "ADC"),
            make_cached_player(13, 48, 19, "SUP"),
        ];

        let w_mentor = engine.calculate_willingness(
            65, 50, 20, 4_000_000, 3_000_000,
            Some(1), Some(1), 50, 5, 60, &roster_with_mentor, "MID", &mut rng1,
        );
        let w_no_mentor = engine.calculate_willingness(
            65, 50, 20, 4_000_000, 3_000_000,
            Some(1), Some(1), 50, 12, 30, &roster_no_mentor, "MID", &mut rng2,
        );
        assert!(
            w_mentor > w_no_mentor,
            "young player should prefer team with mentor and better environment: {} vs {}",
            w_mentor, w_no_mentor
        );
    }

    // ==================== calculate_match_score ====================

    #[test]
    fn test_match_score_high_ability_strong_team() {
        let engine = make_engine();
        let weights = AIDecisionWeights::default();
        let roster = vec![make_cached_player(10, 72, 25, "TOP")];
        let score = engine.calculate_match_score(
            85, 25, "MID", &weights,
            50_000_000, &roster, 3, 85, 75, "NORMAL",
        );
        assert!(score > 60.0, "high ability should yield high match score, got {}", score);
    }

    #[test]
    fn test_match_score_low_ability() {
        let engine = make_engine();
        let weights = AIDecisionWeights::default();
        let roster = vec![make_cached_player(10, 72, 25, "MID")];
        let score = engine.calculate_match_score(
            45, 30, "MID", &weights,
            50_000_000, &roster, 3, 50, 60, "ORDINARY",
        );
        assert!(score < 50.0, "low ability should yield low match score, got {}", score);
    }

    #[test]
    fn test_match_score_empty_position_high_need() {
        let engine = make_engine();
        let weights = AIDecisionWeights::default();
        // No one at ADC position
        let roster = vec![make_cached_player(10, 72, 25, "MID")];
        let score = engine.calculate_match_score(
            70, 24, "ADC", &weights,
            30_000_000, &roster, 5, 75, 70, "NORMAL",
        );
        assert!(score > 55.0, "empty position should boost match score, got {}", score);
    }

    #[test]
    fn test_match_score_genius_tag_multiplier() {
        let engine = make_engine();
        let weights = AIDecisionWeights::default();
        let roster = vec![make_cached_player(10, 65, 25, "TOP")];

        let score_genius = engine.calculate_match_score(
            75, 22, "MID", &weights,
            30_000_000, &roster, 5, 80, 70, "GENIUS",
        );
        let score_ordinary = engine.calculate_match_score(
            75, 22, "MID", &weights,
            30_000_000, &roster, 5, 80, 70, "ORDINARY",
        );
        assert!(
            score_genius > score_ordinary,
            "genius ({}) should score higher than ordinary ({})",
            score_genius, score_ordinary
        );
    }

    // ==================== TransferCache ====================

    #[test]
    fn test_cache_transfer_player() {
        let mut cache = empty_cache();
        let player = make_cached_player(1, 75, 24, "MID");
        cache.team_rosters.insert(100, vec![player]);
        cache.team_rosters.insert(200, vec![]);

        cache.transfer_player(1, Some(100), Some(200), None);

        assert_eq!(cache.get_roster(100).len(), 0, "player should be removed from old team");
        assert_eq!(cache.get_roster(200).len(), 1, "player should be added to new team");
        assert_eq!(cache.get_roster(200)[0].team_id, Some(200));
    }

    #[test]
    fn test_cache_transfer_player_with_updates() {
        let mut cache = empty_cache();
        let player = make_cached_player(1, 75, 24, "MID");
        cache.team_rosters.insert(100, vec![player]);
        cache.team_rosters.insert(200, vec![]);

        cache.transfer_player(1, Some(100), Some(200), Some(PlayerCacheUpdate {
            salary: Some(10_000_000),
            loyalty: Some(40),
            satisfaction: Some(80),
            contract_end_season: Some(5),
        }));

        let roster = cache.get_roster(200);
        assert_eq!(roster[0].salary, 10_000_000);
        assert_eq!(roster[0].loyalty, 40);
        assert_eq!(roster[0].satisfaction, 80);
        assert_eq!(roster[0].contract_end_season, Some(5));
    }

    #[test]
    fn test_cache_update_balance() {
        let mut cache = empty_cache();
        cache.team_balances.insert(100, 50_000_000);

        cache.update_balance(100, -20_000_000);
        assert_eq!(*cache.team_balances.get(&100).unwrap(), 30_000_000);

        cache.update_balance(100, 10_000_000);
        assert_eq!(*cache.team_balances.get(&100).unwrap(), 40_000_000);
    }

    #[test]
    fn test_cache_retire_player() {
        let mut cache = empty_cache();
        let p1 = make_cached_player(1, 75, 34, "MID");
        let p2 = make_cached_player(2, 70, 25, "TOP");
        cache.team_rosters.insert(100, vec![p1, p2]);

        cache.retire_player(1, Some(100));

        let roster = cache.get_roster(100);
        assert_eq!(roster.len(), 1);
        assert_eq!(roster[0].id, 2);
    }

    #[test]
    fn test_cache_get_team_rank_default() {
        let cache = empty_cache();
        assert_eq!(cache.get_team_rank(999), 99);
    }

    #[test]
    fn test_cache_get_team_reputation_default() {
        let cache = empty_cache();
        assert_eq!(cache.get_team_reputation(999), 30);
    }
}
