use sqlx::{Pool, Row, Sqlite};
use std::collections::{HashMap, HashSet};

use crate::models::transfer::*;

use super::utils::normalize_position;

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
    pub growth_accumulator: f64,
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
            position: normalize_position(&row.try_get::<String, _>("position").unwrap_or_default()),
            tag: row.try_get("tag").unwrap_or_else(|_| "NORMAL".to_string()),
            team_id: row.try_get("team_id").ok(),
            is_starter: row.try_get("is_starter").unwrap_or(false),
            home_region_id: row.try_get("home_region_id").ok(),
            region_loyalty: row.try_get("region_loyalty").unwrap_or(70),
            contract_end_season: row.try_get("contract_end_season").ok(),
            status: row.try_get("status").unwrap_or_else(|_| "Active".to_string()),
            stability: row.try_get("stability").unwrap_or(60),
            growth_accumulator: row.try_get("growth_accumulator").unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CachedPlayerStats {
    pub avg_impact: f64,
    pub avg_performance: f64,
    pub consistency_score: f64,
    pub dominance_score: f64,
    pub games_played: i32,
    // 国际赛表现（来自 player_tournament_stats）
    pub international_avg_impact: f64,
    pub international_games: i32,
    // MVP 次数（来自 player_tournament_stats，当赛季所有赛事合计）
    pub total_mvp_count: i32,
    // 选手状态动量（来自 player_form_factors）
    pub momentum: i32,
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
    /// 续约破裂黑名单：(player_id, team_id)，同窗口内原队不能再签回
    pub renewal_failed_pairs: HashSet<(i64, i64)>,
    pub team_spring_ranks: HashMap<i64, i32>,
    pub team_summer_ranks: HashMap<i64, i32>,
    pub player_season_stats: HashMap<i64, CachedPlayerStats>,
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

        // 5.6 当季春季赛/夏季赛排名
        let current_season_standings: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT ls.team_id, ls.rank, t.tournament_type
               FROM league_standings ls
               JOIN tournaments t ON ls.tournament_id = t.id
               WHERE t.save_id = ? AND t.season_id = ?
               AND t.tournament_type IN ('SpringRegular', 'SummerRegular')"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_spring_ranks: HashMap<i64, i32> = HashMap::new();
        let mut team_summer_ranks: HashMap<i64, i32> = HashMap::new();
        for r in &current_season_standings {
            let team_id: i64 = r.get("team_id");
            let rank: i32 = r.try_get("rank").unwrap_or(99);
            let t_type: String = r.get("tournament_type");
            match t_type.as_str() {
                "SpringRegular" => { team_spring_ranks.entry(team_id).or_insert(rank); },
                "SummerRegular" => { team_summer_ranks.entry(team_id).or_insert(rank); },
                _ => {}
            }
        }

        // 5.7 加载选手赛季统计数据（用于转会评估）
        let player_stats_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT player_id, avg_impact, avg_performance, consistency_score,
                      dominance_score, games_played
               FROM player_season_stats
               WHERE save_id = ? AND season_id = ?"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut player_season_stats: HashMap<i64, CachedPlayerStats> = HashMap::new();
        for r in &player_stats_rows {
            let pid: i64 = r.get("player_id");
            player_season_stats.insert(pid, CachedPlayerStats {
                avg_impact: r.try_get("avg_impact").unwrap_or(0.0),
                avg_performance: r.try_get("avg_performance").unwrap_or(0.0),
                consistency_score: r.try_get("consistency_score").unwrap_or(50.0),
                dominance_score: r.try_get("dominance_score").unwrap_or(0.0),
                games_played: r.try_get("games_played").unwrap_or(0),
                ..Default::default()
            });
        }

        // 5.8 从 player_tournament_stats 加载国际赛表现 + MVP 次数
        let intl_types = "('Msi','WorldChampionship','MadridMasters','ShanghaiMasters','ClaudeIntercontinental','IcpIntercontinental','SuperIntercontinental')";
        let intl_query = format!(
            r#"SELECT player_id,
                      AVG(avg_impact) as intl_avg_impact,
                      SUM(games_played) as intl_games
               FROM player_tournament_stats
               WHERE save_id = ? AND season_id = ?
               AND tournament_type IN {}
               GROUP BY player_id"#,
            intl_types
        );
        let intl_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(&intl_query)
            .bind(save_id)
            .bind(season_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        for r in &intl_rows {
            let pid: i64 = r.get("player_id");
            let entry = player_season_stats.entry(pid).or_default();
            entry.international_avg_impact = r.try_get("intl_avg_impact").unwrap_or(0.0);
            entry.international_games = r.try_get::<i64, _>("intl_games").unwrap_or(0) as i32;
        }

        let mvp_query = r#"SELECT player_id, SUM(game_mvp_count) as total_mvps
               FROM player_tournament_stats
               WHERE save_id = ? AND season_id = ?
               GROUP BY player_id"#;
        let mvp_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(mvp_query)
            .bind(save_id)
            .bind(season_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        for r in &mvp_rows {
            let pid: i64 = r.get("player_id");
            let entry = player_season_stats.entry(pid).or_default();
            entry.total_mvp_count = r.try_get::<i64, _>("total_mvps").unwrap_or(0) as i32;
        }

        // 5.9 从 player_form_factors 加载 momentum
        let momentum_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT player_id, momentum FROM player_form_factors WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for r in &momentum_rows {
            let pid: i64 = r.get("player_id");
            let entry = player_season_stats.entry(pid).or_default();
            entry.momentum = r.try_get("momentum").unwrap_or(0);
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
            renewal_failed_pairs: HashSet::new(),
            team_spring_ranks,
            team_summer_ranks,
            player_season_stats,
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

    /// 综合排名：春季赛(0.3) + 夏季赛(0.3) + 年度积分(0.4)，无数据的维度权重重分配
    pub fn get_composite_rank(&self, team_id: i64) -> i32 {
        let spring = self.team_spring_ranks.get(&team_id).copied();
        let summer = self.team_summer_ranks.get(&team_id).copied();
        let annual = self.team_annual_ranks.get(&team_id).copied();

        let mut total_weight = 0.0f64;
        let mut weighted_rank = 0.0f64;

        if let Some(r) = spring { total_weight += 0.3; weighted_rank += r as f64 * 0.3; }
        if let Some(r) = summer { total_weight += 0.3; weighted_rank += r as f64 * 0.3; }
        if let Some(r) = annual { total_weight += 0.4; weighted_rank += r as f64 * 0.4; }

        if total_weight > 0.0 {
            (weighted_rank / total_weight).round() as i32
        } else {
            99
        }
    }

    pub fn get_team_last_rank(&self, team_id: i64) -> i32 {
        self.team_last_season_ranks.get(&team_id).copied()
            .unwrap_or_else(|| self.get_team_rank(team_id))
    }

    /// 获取球队综合声望（0-100）
    pub fn get_team_reputation(&self, team_id: i64) -> i64 {
        *self.team_reputations.get(&team_id).unwrap_or(&30)
    }

    pub fn get_player_stats(&self, player_id: i64) -> Option<&CachedPlayerStats> {
        self.player_season_stats.get(&player_id)
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

    /// 更新选手属性（年龄/能力/累积器）
    pub fn update_player_stats(&mut self, player_id: i64, team_id: Option<i64>, new_age: i64, new_ability: i64, new_accumulator: f64) {
        if let Some(tid) = team_id {
            if let Some(roster) = self.team_rosters.get_mut(&tid) {
                if let Some(p) = roster.iter_mut().find(|p| p.id == player_id) {
                    p.age = new_age;
                    p.ability = new_ability;
                    p.growth_accumulator = new_accumulator;
                }
            }
        }
    }

    pub fn update_player_salary(&mut self, player_id: i64, team_id: i64, new_salary: i64) {
        if let Some(roster) = self.team_rosters.get_mut(&team_id) {
            if let Some(p) = roster.iter_mut().find(|p| p.id == player_id) {
                p.salary = new_salary;
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
