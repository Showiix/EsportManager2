use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};
use std::collections::{HashMap, HashSet};

use crate::engines::market_value::MarketValueEngine;
use crate::engines::traits::{TraitEngine, TraitType};
use crate::models::transfer::*;

use super::cache::{CachedPlayer, TransferCache};
use super::TransferEngine;

impl TransferEngine {
// ============================================
    // 第1轮：赛季结算
    // ============================================

    pub(crate) async fn execute_season_settlement(
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
            let prev_accumulator = player.growth_accumulator;

            let (new_ability, new_accumulator) = if new_age <= growth_cap && ability < potential {
                // ========== 成长期（累积器模式） ==========

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

                // ⑤ 年龄衰减后成长（精确小数）
                let growth_after_age_f64 = base_growth as f64 * age_coeff * prodigy_mod;

                // ⑥ 表现加成 (D) — 基于赛季统计
                let perf_bonus: f64 = match stats_map.get(&player_id) {
                    Some(&(gp, avg_perf)) if gp >= 20 && avg_perf > ability as f64 + 5.0 => {
                        if perf_desc.is_empty() { perf_desc = "超常发挥".to_string(); }
                        else { perf_desc.push_str("+超常发挥"); }
                        1.0
                    }
                    Some(&(gp, avg_perf)) if gp >= 20 && avg_perf > ability as f64 => {
                        if rng.gen_bool(0.5) {
                            if perf_desc.is_empty() { perf_desc = "突破成长".to_string(); }
                            else { perf_desc.push_str("+突破成长"); }
                            1.0
                        } else { 0.0 }
                    }
                    Some(&(gp, _)) if gp == 0 => {
                        if perf_desc.is_empty() { perf_desc = "缺乏实战".to_string(); }
                        else { perf_desc.push_str("+缺乏实战"); }
                        // 缺乏实战：成长减半
                        -(growth_after_age_f64 / 2.0)
                    }
                    Some(&(gp, avg_perf)) if gp > 0 && avg_perf < (ability as f64) - 5.0 => {
                        if perf_desc.is_empty() { perf_desc = "表现低迷".to_string(); }
                        else { perf_desc.push_str("+表现低迷"); }
                        -1.0
                    }
                    _ => 0.0,
                };

                // ⑦ 累积器模式：小数精确累积，攒够整数才涨
                let raw_growth = (growth_after_age_f64 + perf_bonus).max(0.0);
                let accumulated = prev_accumulator + raw_growth;
                let integer_part = accumulated.trunc() as i64;
                let remainder = accumulated.fract();
                let new_val = (ability + integer_part).min(potential).min(100);
                (new_val, remainder)

            } else if new_age > growth_cap {
                // ========== 衰退期（累积器模式） ==========

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

                // ④ 累积器模式：负数累积，攒够才降
                let raw_decline = base_decline * tag_decay * trait_decay;
                // 衰退用负数累积（prev_accumulator 可能有正的残留，先抵消）
                let accumulated = prev_accumulator - raw_decline;
                let integer_part = accumulated.trunc() as i64; // 负数时 trunc 朝零取整，如 -1.3 → -1
                let remainder = accumulated.fract(); // -1.3 → -0.3
                // integer_part 为负数时表示衰退量（如 -1），ability + (-1) = 衰退1点
                let new_val = (ability + integer_part).max(50);
                (new_val, remainder)

            } else {
                // 已达潜力上限且在成长期，不成长也不衰退
                (ability, prev_accumulator)
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
                "UPDATE players SET age = ?, ability = ?, potential = ?, growth_accumulator = ? WHERE id = ? AND save_id = ?"
            )
            .bind(new_age)
            .bind(new_ability)
            .bind(new_potential)
            .bind(new_accumulator)
            .bind(player_id)
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新选手年龄/能力失败: {}", e))?;

            // 更新缓存
            cache.update_player_stats(player_id, team_id, new_age, new_ability, new_accumulator);

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

                sqlx::query("UPDATE player_contracts SET is_active = 0 WHERE save_id = ? AND player_id = ? AND is_active = 1")
                    .bind(save_id).bind(player_id).execute(pool).await.ok();

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

        // ============================================
        // 满意度 & 忠诚度赛季结算
        // ============================================

        // 查询本赛季各球队在常规赛的排名（取夏季常规赛 > 春季常规赛）
        let team_rank_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT ls.team_id, ls.rank, t.tournament_type
               FROM league_standings ls
               JOIN tournaments t ON ls.tournament_id = t.id
               WHERE t.save_id = ? AND t.season_id = ?
               AND (t.tournament_type = 'SummerRegular' OR t.tournament_type = 'SpringRegular')
               ORDER BY t.tournament_type DESC"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_final_ranks: HashMap<i64, i32> = HashMap::new();
        for r in &team_rank_rows {
            let tid: i64 = r.get("team_id");
            let rank: i32 = r.try_get("rank").unwrap_or(99);
            team_final_ranks.entry(tid).or_insert(rank); // SummerRegular 优先
        }

        // 查询本赛季各球队荣誉（判断是否夺冠、进季后赛）
        let team_honor_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT team_id, honor_type FROM honors
               WHERE save_id = ? AND season_id = ?
               AND team_id IS NOT NULL
               AND honor_type IN ('TeamChampion', 'TeamRunnerUp', 'TeamThird', 'TeamFourth')"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut team_is_champion: HashSet<i64> = HashSet::new();
        let mut team_made_playoffs: HashSet<i64> = HashSet::new();
        let mut team_has_international: HashSet<i64> = HashSet::new();
        for r in &team_honor_rows {
            let tid: i64 = r.get("team_id");
            let honor: String = r.get("honor_type");
            match honor.as_str() {
                "TeamChampion" => { team_is_champion.insert(tid); team_made_playoffs.insert(tid); }
                "TeamRunnerUp" | "TeamThird" | "TeamFourth" => { team_made_playoffs.insert(tid); }
                _ => {}
            }
        }

        // 查询本赛季国际赛荣誉
        let intl_honor_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            r#"SELECT DISTINCT team_id FROM honors
               WHERE save_id = ? AND season_id = ?
               AND team_id IS NOT NULL
               AND tournament_type IN ('Msi', 'WorldChampionship', 'MadridMasters',
                   'ShanghaiMasters', 'ClaudeIntercontinental', 'IcpIntercontinental', 'SuperIntercontinental')"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for r in &intl_honor_rows {
            let tid: i64 = r.get("team_id");
            team_has_international.insert(tid);
        }

        // 查询上赛季排名（连续未进季后赛的简化判断）
        let prev_no_playoff: HashSet<i64> = {
            let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
                r#"SELECT ls.team_id, ls.rank
                   FROM league_standings ls
                   JOIN tournaments t ON ls.tournament_id = t.id
                   WHERE t.save_id = ? AND t.season_id = ?
                   AND t.tournament_type = 'SummerRegular'"#
            )
            .bind(save_id)
            .bind(season_id - 1)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            rows.iter()
                .filter(|r| r.try_get::<i32, _>("rank").unwrap_or(99) >= 7)
                .map(|r| r.get::<i64, _>("team_id"))
                .collect()
        };

        // 查询选手加入赛季（用于忠诚度-在队年数）
        let join_season_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT id, join_season FROM players WHERE save_id = ? AND status = 'Active' AND team_id IS NOT NULL"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let join_season_map: HashMap<i64, i64> = join_season_rows.iter()
            .map(|r| {
                let pid: i64 = r.get("id");
                let js: i64 = r.try_get("join_season").unwrap_or(season_id);
                (pid, js)
            })
            .collect();

        // 批量更新满意度和忠诚度
        let active_players: Vec<CachedPlayer> = cache.team_rosters.values()
            .flat_map(|roster| roster.iter().cloned())
            .collect();

        for player in &active_players {
            let team_id = match player.team_id {
                Some(tid) => tid,
                None => continue,
            };

            let rank = team_final_ranks.get(&team_id).copied().unwrap_or(99);
            let is_champion = team_is_champion.contains(&team_id);
            let made_playoffs = team_made_playoffs.contains(&team_id);
            let has_international = team_has_international.contains(&team_id);
            let consecutive_no_playoffs = if rank >= 7 && prev_no_playoff.contains(&team_id) { 2u32 } else if rank >= 7 { 1 } else { 0 };

            let (gp, _avg_perf) = stats_map.get(&player.id).copied().unwrap_or((0, 0.0));
            let total_games = gp.max(0) as u32;
            let games_as_starter = if player.is_starter { total_games } else { 0u32 };
            let starter_ratio = if total_games > 0 { games_as_starter as f64 / total_games as f64 } else { 0.0 };

            // === 满意度变化 ===
            let mut sat_change: i32 = 0;

            // 上场时间
            if starter_ratio < 0.3 && player.ability >= 54 {
                sat_change -= 12;
            } else if starter_ratio < 0.5 && player.ability >= 47 {
                sat_change -= 6;
            } else if starter_ratio >= 0.8 {
                sat_change += 8;
            }

            // 球队战绩
            if rank >= 8 && player.ability >= 58 {
                sat_change -= 10;
            } else if rank >= 6 && player.ability >= 61 {
                sat_change -= 6;
            } else if rank <= 2 {
                sat_change += 10;
            } else if rank <= 4 {
                sat_change += 8;
            }

            // 连续未进季后赛
            if consecutive_no_playoffs >= 2 {
                sat_change -= 10;
            } else if consecutive_no_playoffs >= 1 {
                sat_change -= 3;
            }

            // 薪资满意度
            let market_value = MarketValueEngine::calculate_base_market_value(
                player.ability as u8, player.age as u8, player.ability as u8, "NORMAL", &player.position
            );
            let expected_salary = MarketValueEngine::estimate_salary(market_value, player.ability as u8, player.age as u8);
            let salary_ratio = if expected_salary > 0 { player.salary as f64 / expected_salary as f64 } else { 1.0 };

            if salary_ratio < 0.5 {
                sat_change -= 15;
            } else if salary_ratio < 0.6 {
                sat_change -= 10;
            } else if salary_ratio < 0.8 {
                sat_change -= 5;
            } else if salary_ratio > 1.2 {
                sat_change += 5;
            }

            // 夺冠/国际赛/季后赛加成
            if is_champion {
                sat_change += 20;
            } else if has_international {
                sat_change += 15;
            } else if made_playoffs {
                sat_change += 5;
            }

            // 年龄因素
            if player.age >= 28 && rank >= 8 {
                sat_change -= 6;
            }
            if player.age <= 24 && starter_ratio < 0.5 {
                sat_change -= 3;
            }

            // 满意度自然回归：每赛季向60靠拢，幅度为差值的10%，至少±1
            let current_sat = player.satisfaction;
            let regression_target = 60i64;
            let diff = regression_target - current_sat;
            if diff != 0 {
                let regression = if diff > 0 {
                    (diff as f64 * 0.1).ceil() as i32
                } else {
                    (diff as f64 * 0.1).floor() as i32
                };
                sat_change += regression;
            }

            // === 忠诚度变化 ===
            let mut loy_change: i32 = 3; // 每赛季自然增长（+2 → +3）

            let join_s = join_season_map.get(&player.id).copied().unwrap_or(season_id);
            let seasons_with_team = (season_id - join_s).max(0) as u32;

            // 青训出身（第一赛季加成）
            if seasons_with_team == 1 {
                // 简化：不知道是否选秀出身，跳过 DraftOrigin 加成
            }

            // 球队战绩
            if is_champion {
                loy_change += 8;
            } else if made_playoffs {
                loy_change += 5;
            } else if rank >= 8 {
                loy_change -= 3;
            }

            // 长期替补
            if !player.is_starter && seasons_with_team >= 2 {
                loy_change -= 5;
            }

            // 应用变化
            let new_satisfaction = (player.satisfaction + sat_change as i64).clamp(5, 100);
            let new_loyalty = (player.loyalty + loy_change as i64).clamp(5, 100);

            sqlx::query(
                "UPDATE players SET satisfaction = ?, loyalty = ? WHERE id = ? AND save_id = ?"
            )
            .bind(new_satisfaction)
            .bind(new_loyalty)
            .bind(player.id)
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新满意度/忠诚度失败: {}", e))?;

            // 同步更新缓存
            if let Some(roster) = cache.team_rosters.get_mut(&team_id) {
                if let Some(p) = roster.iter_mut().find(|p| p.id == player.id) {
                    p.satisfaction = new_satisfaction;
                    p.loyalty = new_loyalty;
                }
            }

            // 满意度变化≥10 或 忠诚度变化≥5 时生成事件
            let actual_sat_change = new_satisfaction - player.satisfaction;
            let actual_loy_change = new_loyalty - player.loyalty;
            if actual_sat_change.abs() >= 10 || actual_loy_change.abs() >= 5 {
                let from_team_name = cache.get_team_name(team_id);
                let mut parts = Vec::new();
                if actual_sat_change != 0 {
                    parts.push(format!("满意度 {} → {}（{}{}）",
                        player.satisfaction, new_satisfaction,
                        if actual_sat_change > 0 { "+" } else { "" }, actual_sat_change));
                }
                if actual_loy_change != 0 {
                    parts.push(format!("忠诚度 {} → {}（{}{}）",
                        player.loyalty, new_loyalty,
                        if actual_loy_change > 0 { "+" } else { "" }, actual_loy_change));
                }

                let level = if actual_sat_change.abs() >= 20 || actual_loy_change.abs() >= 10 {
                    EventLevel::A
                } else {
                    EventLevel::B
                };

                let event = self.record_event(
                    pool, window_id, 1,
                    TransferEventType::SeasonSettlement,
                    level,
                    player.id, &player.game_id, player.ability,
                    Some(team_id), Some(&from_team_name),
                    Some(team_id), Some(&from_team_name),
                    0, 0, 0,
                    &parts.join("，"),
                ).await?;
                events.push(event);
            }
        }

        log::info!("✅ 满意度/忠诚度赛季结算完成，更新了 {} 名选手", active_players.len());

        // === 特性觉醒/退化评估 ===
        let join_season_rows = sqlx::query(
            "SELECT id, join_season FROM players WHERE save_id = ? AND status = 'Active'"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询join_season失败: {}", e))?;

        let join_season_map: HashMap<i64, i64> = join_season_rows.iter()
            .map(|row| (row.get::<i64, _>("id"), row.get::<i64, _>("join_season")))
            .collect();

        let mut awakening_count = 0u32;
        let mut decay_count = 0u32;

        for player in &all_players {
            let player_traits = traits_map.get(&player.id).cloned().unwrap_or_default();
            let (gp, avg_perf) = stats_map.get(&player.id).copied().unwrap_or((0, 0.0));
            let join_s = join_season_map.get(&player.id).copied().unwrap_or(season_id);
            let seasons_in_team = (season_id - join_s).max(0);

            let (gained, lost) = TraitEngine::evaluate_trait_awakening(
                player.ability as u8,
                player.age as u8,
                gp,
                avg_perf,
                &player_traits,
                seasons_in_team,
                &mut rng,
            );

            for new_trait in &gained {
                let trait_name = format!("{:?}", new_trait);
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO player_traits (save_id, player_id, trait_type, acquired_season) VALUES (?, ?, ?, ?)"
                )
                .bind(save_id)
                .bind(player.id)
                .bind(&trait_name.to_lowercase())
                .bind(season_id)
                .execute(pool)
                .await;

                let team_name = player.team_id.and_then(|tid| {
                    cache.team_names.get(&tid).map(|s| s.as_str())
                });
                let event = self.record_event(
                    pool, window_id, 1,
                    TransferEventType::SeasonSettlement,
                    EventLevel::B,
                    player.id, &player.game_id, player.ability,
                    player.team_id, team_name,
                    player.team_id, team_name,
                    0, 0, 0,
                    &format!("特性觉醒：获得「{}」", new_trait.display_name()),
                ).await?;
                events.push(event);
                awakening_count += 1;
            }

            for lost_trait in &lost {
                let trait_name = format!("{:?}", lost_trait);
                let _ = sqlx::query(
                    "DELETE FROM player_traits WHERE save_id = ? AND player_id = ? AND trait_type = ?"
                )
                .bind(save_id)
                .bind(player.id)
                .bind(&trait_name.to_lowercase())
                .execute(pool)
                .await;

                let team_name = player.team_id.and_then(|tid| {
                    cache.team_names.get(&tid).map(|s| s.as_str())
                });
                let event = self.record_event(
                    pool, window_id, 1,
                    TransferEventType::SeasonSettlement,
                    EventLevel::B,
                    player.id, &player.game_id, player.ability,
                    player.team_id, team_name,
                    player.team_id, team_name,
                    0, 0, 0,
                    &format!("特性退化：失去「{}」", lost_trait.display_name()),
                ).await?;
                events.push(event);
                decay_count += 1;
            }
        }

        if awakening_count > 0 || decay_count > 0 {
            log::info!("✅ 特性觉醒/退化完成：{} 次觉醒，{} 次退化", awakening_count, decay_count);
        }

        // 更新所有球队战力
        self.recalculate_team_powers_optimized(pool, save_id).await?;

        Ok(RoundResult {
            round: 1,
            round_name: "赛季结算".to_string(),
            events,
            summary: "已完成赛季结算：选手年龄+1、能力值更新（含表现加成/年龄曲线/特性影响）、潜力微调、退役处理、满意度/忠诚度更新".to_string(),
        })
    }

    // ============================================
    // 第2轮：续约谈判
    // ============================================
}
