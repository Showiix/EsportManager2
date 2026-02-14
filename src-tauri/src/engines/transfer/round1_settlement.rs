use rand::Rng;
use rand::SeedableRng;
use sqlx::{Pool, Row, Sqlite};
use std::collections::{HashMap, HashSet};

use crate::engines::champion::{self, MasteryTier};
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

        // === 预加载：上场时间系数（从 game_player_performances 聚合） ===
        // 1) 每个选手本赛季实际上场局数
        let player_games_rows = sqlx::query(
            r#"SELECT gpp.player_id, COUNT(DISTINCT gpp.game_id) as games_appeared
               FROM game_player_performances gpp
               JOIN match_games mg ON gpp.game_id = mg.id
               JOIN matches m ON mg.match_id = m.id
               JOIN tournaments t ON m.tournament_id = t.id
               WHERE gpp.save_id = ? AND t.season_id = ?
               GROUP BY gpp.player_id"#
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询选手上场局数失败: {}", e))?;

        let player_games_map: HashMap<i64, i64> = player_games_rows.iter()
            .map(|row| {
                let pid: i64 = row.get("player_id");
                let games: i64 = row.get("games_appeared");
                (pid, games)
            })
            .collect();

        // 2) 每个队伍本赛季总局数（参与的所有比赛局数）
        let team_games_rows = sqlx::query(
            r#"SELECT team_id, COUNT(DISTINCT game_id) as total_games FROM (
                   SELECT mg.winner_team_id as team_id, mg.id as game_id
                   FROM match_games mg
                   JOIN matches m ON mg.match_id = m.id
                   JOIN tournaments t ON m.tournament_id = t.id
                   WHERE mg.save_id = ? AND t.season_id = ?
                   UNION ALL
                   SELECT mg.loser_team_id as team_id, mg.id as game_id
                   FROM match_games mg
                   JOIN matches m ON mg.match_id = m.id
                   JOIN tournaments t ON m.tournament_id = t.id
                   WHERE mg.save_id = ? AND t.season_id = ?
               ) GROUP BY team_id"#
        )
        .bind(save_id)
        .bind(season_id)
        .bind(save_id)
        .bind(season_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询队伍总局数失败: {}", e))?;

        let team_total_games_map: HashMap<i64, i64> = team_games_rows.iter()
            .map(|row| {
                let tid: i64 = row.get("team_id");
                let total: i64 = row.get("total_games");
                (tid, total)
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

        // === 预加载：选手加入赛季（协同成长计算用） ===
        let join_season_rows_early: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT id, join_season FROM players WHERE save_id = ? AND status = 'Active' AND team_id IS NOT NULL"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let join_season_map_early: HashMap<i64, i64> = join_season_rows_early.iter()
            .map(|r| {
                let pid: i64 = r.get("id");
                let js: i64 = r.try_get("join_season").unwrap_or(season_id);
                (pid, js)
            })
            .collect();

        // === 预计算：每个队伍首发平均同队赛季数（协同成长） ===
        let mut team_avg_tenure: HashMap<i64, f64> = HashMap::new();
        for (&tid, roster) in &cache.team_rosters {
            let starters: Vec<&CachedPlayer> = roster.iter().filter(|p| p.is_starter).collect();
            if starters.len() >= 5 {
                let total_tenure: f64 = starters.iter()
                    .map(|p| {
                        let js = join_season_map_early.get(&p.id).copied().unwrap_or(season_id);
                        (season_id - js).max(0) as f64
                    })
                    .sum();
                team_avg_tenure.insert(tid, total_tenure / starters.len() as f64);
            }
        }

        // === 预加载：队伍训练设施等级 ===
        let facility_rows = sqlx::query(
            "SELECT id, training_facility FROM teams WHERE save_id = ?"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let team_facility_map: HashMap<i64, i64> = facility_rows.iter()
            .map(|r| (r.get::<i64, _>("id"), r.try_get::<i64, _>("training_facility").unwrap_or(1)))
            .collect();

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

            let mut log_base_growth: f64 = 0.0;
            let mut log_age_coeff: f64 = 1.0;
            let mut log_playtime_coeff: f64 = 1.0;
            let mut log_mentor_coeff: f64 = 1.0;
            let mut log_synergy_coeff: f64 = 1.0;
            let mut log_facility_coeff: f64 = 1.0;
            let mut log_prodigy_mod: f64 = 1.0;
            let mut log_perf_bonus: f64 = 0.0;
            let mut log_fluctuation: f64 = 0.0;

            let (new_ability, new_accumulator) = if new_age <= growth_cap && ability < potential {
                // ========== 成长期（累积器模式） ==========

                // ① 随机基础成长 (A)
                let base_growth: i64 = match tag.to_uppercase().as_str() {
                    "GENIUS" => rng.gen_range(2..=4),
                    "NORMAL" => rng.gen_range(1..=3),
                    _ => rng.gen_range(0..=2), // ORDINARY
                };

                // ② 突破/停滞事件 (A) — 互斥，10%总事件率
                let event_roll: f64 = rng.r#gen();
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

                // ⑤ 上场时间系数 — 基于实际出场局数/队伍总局数
                let starter_ratio_for_growth: f64 = if let Some(tid) = team_id {
                    let appeared = player_games_map.get(&player_id).copied().unwrap_or(0);
                    let total = team_total_games_map.get(&tid).copied().unwrap_or(0);
                    if total > 0 { appeared as f64 / total as f64 } else { 0.0 }
                } else {
                    0.0
                };
                let playtime_coeff: f64 = if starter_ratio_for_growth >= 0.8 {
                    1.2
                } else if starter_ratio_for_growth >= 0.3 {
                    1.0
                } else if starter_ratio_for_growth > 0.0 {
                    0.5
                } else {
                    0.3
                };

                // ⑥ 导师效应 — 年轻选手(≤22)受同位置导师/老将加成
                let mentor_coeff: f64 = if new_age <= 22 {
                    if let Some(tid) = team_id {
                        let teammates = cache.team_rosters.get(&tid);
                        let has_mentor_same_pos = teammates.map_or(false, |roster| {
                            roster.iter().any(|t| {
                                t.id != player_id
                                    && t.position == player.position
                                    && traits_map.get(&t.id).map_or(false, |tr| tr.contains(&TraitType::Mentor))
                            })
                        });
                        let has_veteran_same_pos = teammates.map_or(false, |roster| {
                            roster.iter().any(|t| {
                                t.id != player_id
                                    && t.position == player.position
                                    && t.ability >= 70
                                    && t.age >= 28
                                    && !traits_map.get(&t.id).map_or(false, |tr| tr.contains(&TraitType::Mentor))
                            })
                        });
                        if has_mentor_same_pos {
                            1.25
                        } else if has_veteran_same_pos {
                            1.10
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                // ⑦ 协同成长 — 首发平均同队≥3赛季 → +5%
                let synergy_coeff: f64 = if let Some(tid) = team_id {
                    if team_avg_tenure.get(&tid).copied().unwrap_or(0.0) >= 3.0 { 1.05 } else { 1.0 }
                } else {
                    1.0
                };

                // ⑧ 培训设施加成 — 等级1~10 → +5%~+50%
                let facility_coeff: f64 = if let Some(tid) = team_id {
                    let level = team_facility_map.get(&tid).copied().unwrap_or(1).clamp(1, 10);
                    1.0 + level as f64 * 0.05
                } else {
                    1.0
                };

                // ⑨ 环境系数后成长（精确小数）
                let growth_after_env_f64 = base_growth as f64 * age_coeff * prodigy_mod * playtime_coeff * mentor_coeff * synergy_coeff * facility_coeff;

                log_base_growth = base_growth as f64;
                log_age_coeff = age_coeff;
                log_playtime_coeff = playtime_coeff;
                log_mentor_coeff = mentor_coeff;
                log_synergy_coeff = synergy_coeff;
                log_facility_coeff = facility_coeff;
                log_prodigy_mod = prodigy_mod;

                // ⑩ 表现加成 (D) — 基于赛季统计
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
                    Some(&(gp, avg_perf)) if gp > 0 && avg_perf < (ability as f64) - 5.0 => {
                        if perf_desc.is_empty() { perf_desc = "表现低迷".to_string(); }
                        else { perf_desc.push_str("+表现低迷"); }
                        -1.0
                    }
                    _ => 0.0,
                };

                // ⑪ 随机波动 ±1
                let fluctuation: f64 = rng.gen_range(-1.0..=1.0);

                log_perf_bonus = perf_bonus;
                log_fluctuation = fluctuation;

                // ⑫ 累积器模式：小数精确累积，攒够整数才涨
                let raw_growth = (growth_after_env_f64 + perf_bonus + fluctuation).max(0.0);
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

                log_base_growth = -raw_decline;
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

            // ========== 成长事件系统 ==========
            // 每人每赛季最多1个事件，互斥判定
            let mut new_ability = new_ability;
            let mut new_potential = new_potential;
            let mut growth_event: Option<&str> = None;

            let event_roll: f64 = rng.r#gen();
            let mut threshold = 0.0;

            // 心态崩盘：满意度<30, 10%
            threshold += 0.10;
            if growth_event.is_none() && player.satisfaction < 30 && event_roll < threshold {
                let loss = rng.gen_range(1..=3) as i64;
                new_ability = (new_ability - loss).max(50);
                growth_event = Some("心态崩盘");
                if perf_desc.is_empty() { perf_desc = format!("心态崩盘(-{})", loss); }
                else { perf_desc.push_str(&format!("+心态崩盘(-{})", loss)); }
            }

            // 瓶颈：25-28岁, 8%
            threshold += 0.08;
            if growth_event.is_none() && new_age >= 25 && new_age <= 28 && event_roll < threshold {
                // 本赛季成长归零：回退到原能力
                new_ability = ability;
                growth_event = Some("瓶颈期");
                if perf_desc.is_empty() { perf_desc = "瓶颈期".to_string(); }
                else { perf_desc.push_str("+瓶颈期"); }
            }

            // 天赋觉醒：Genius + ≤22岁, 6%
            threshold += 0.06;
            if growth_event.is_none() && tag.to_uppercase() == "GENIUS" && new_age <= 22 && event_roll < threshold {
                let pot_gain = rng.gen_range(2..=3) as i64;
                new_potential = (new_potential + pot_gain).min(100);
                growth_event = Some("天赋觉醒");
                if perf_desc.is_empty() { perf_desc = format!("天赋觉醒(潜力+{})", pot_gain); }
                else { perf_desc.push_str(&format!("+天赋觉醒(潜力+{})", pot_gain)); }
            }

            // 觉醒：≤24岁, 5%
            threshold += 0.05;
            if growth_event.is_none() && new_age <= 24 && event_roll < threshold {
                let gain = rng.gen_range(3..=5) as i64;
                new_ability = (new_ability + gain).min(new_potential).min(100);
                growth_event = Some("觉醒");
                if perf_desc.is_empty() { perf_desc = format!("觉醒(+{})", gain); }
                else { perf_desc.push_str(&format!("+觉醒(+{})", gain)); }
            }

            // 二次巅峰：29-32岁, 5% — 衰退期回复少量能力
            threshold += 0.05;
            if growth_event.is_none() && new_age >= 29 && new_age <= 32 && event_roll < threshold {
                let gain = rng.gen_range(1..=2) as i64;
                new_ability = (new_ability + gain).min(new_potential).min(100);
                growth_event = Some("二次巅峰");
                if perf_desc.is_empty() { perf_desc = format!("二次巅峰(+{})", gain); }
                else { perf_desc.push_str(&format!("+二次巅峰(+{})", gain)); }
            }

            // 伤病：任何年龄, 3%
            threshold += 0.03;
            if growth_event.is_none() && event_roll < threshold {
                let loss = rng.gen_range(2..=4) as i64;
                new_ability = (new_ability - loss).max(50);
                growth_event = Some("伤病");
                if perf_desc.is_empty() { perf_desc = format!("伤病(-{})", loss); }
                else { perf_desc.push_str(&format!("+伤病(-{})", loss)); }
            }

            // 更新 growth_event_state
            let event_state = if let Some(evt) = growth_event {
                format!("{{\"season\":{},\"event\":\"{}\"}}", season_id, evt)
            } else {
                "{}".to_string()
            };

            // ========== 更新数据库 ==========
            sqlx::query(
                "UPDATE players SET age = ?, ability = ?, potential = ?, growth_accumulator = ?, growth_event_state = ? WHERE id = ? AND save_id = ?"
            )
            .bind(new_age)
            .bind(new_ability)
            .bind(new_potential)
            .bind(new_accumulator)
            .bind(&event_state)
            .bind(player_id)
            .bind(save_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新选手年龄/能力失败: {}", e))?;

            // 更新缓存
            cache.update_player_stats(player_id, team_id, new_age, new_ability, new_accumulator);

            let log_team_name = if let Some(tid) = team_id {
                cache.get_team_name(tid)
            } else {
                String::new()
            };
            let log_growth_event = growth_event.map(|s| s.to_string());
            let log_desc = format!("{} → {} ({}{})",
                ability, new_ability,
                if !perf_desc.is_empty() { &perf_desc } else { "正常" },
                if new_potential != potential { format!(", 潜力{}→{}", potential, new_potential) } else { String::new() }
            );
            sqlx::query(
                r#"INSERT INTO player_growth_logs
                    (save_id, season_id, player_id, player_name, team_name, age,
                     old_ability, new_ability, old_potential, new_potential,
                     base_growth, age_coeff, playtime_coeff, mentor_coeff, synergy_coeff,
                     facility_coeff, prodigy_mod, perf_bonus, fluctuation, growth_event, description)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
            )
            .bind(save_id)
            .bind(season_id)
            .bind(player_id)
            .bind(game_id)
            .bind(&log_team_name)
            .bind(new_age)
            .bind(ability)
            .bind(new_ability)
            .bind(potential)
            .bind(new_potential)
            .bind(log_base_growth)
            .bind(log_age_coeff)
            .bind(log_playtime_coeff)
            .bind(log_mentor_coeff)
            .bind(log_synergy_coeff)
            .bind(log_facility_coeff)
            .bind(log_prodigy_mod)
            .bind(log_perf_bonus)
            .bind(log_fluctuation)
            .bind(&log_growth_event)
            .bind(&log_desc)
            .execute(pool)
            .await
            .ok();

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

            if retire_chance > 0.0 && rng.r#gen::<f64>() < retire_chance {
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

                if let Ok(Some(hof_entry)) = crate::engines::honor::HonorEngine::evaluate_hall_of_fame(
                    pool, save_id, player_id, season_id,
                )
                .await
                {
                    let _ = sqlx::query(
                        "INSERT OR IGNORE INTO hall_of_fame (save_id, player_id, player_name, position, region_id, induction_season, total_score, tier, peak_ability, career_seasons, honors_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(save_id)
                    .bind(player_id)
                    .bind(&hof_entry.player_name)
                    .bind(&hof_entry.position)
                    .bind(hof_entry.region_id)
                    .bind(hof_entry.induction_season)
                    .bind(hof_entry.total_score)
                    .bind(&hof_entry.tier)
                    .bind(hof_entry.peak_ability)
                    .bind(hof_entry.career_seasons)
                    .bind(&hof_entry.honors_json)
                    .execute(pool)
                    .await;

                    let hof_desc = format!(
                        "入选{}！总积分{}，巅峰能力{}，职业生涯{}赛季",
                        if hof_entry.tier == "Legend" {
                            "传奇殿堂"
                        } else {
                            "名人堂"
                        },
                        hof_entry.total_score,
                        hof_entry.peak_ability,
                        hof_entry.career_seasons
                    );
                    let hof_event = self.record_event(
                        pool,
                        window_id,
                        1,
                        TransferEventType::PlayerRetirement,
                        EventLevel::A,
                        player_id,
                        game_id,
                        new_ability as i64,
                        team_id,
                        if from_team_name.is_empty() {
                            None
                        } else {
                            Some(&from_team_name)
                        },
                        None,
                        None,
                        0,
                        0,
                        0,
                        &hof_desc,
                    )
                    .await?;
                    events.push(hof_event);
                }

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

            let appeared = player_games_map.get(&player.id).copied().unwrap_or(0);
            let team_total = team_total_games_map.get(&team_id).copied().unwrap_or(0);
            let play_rate = if team_total > 0 { appeared as f64 / team_total as f64 } else { 0.0 };

            // === 满意度变化 ===
            let mut sat_change: i32 = 0;

            // 上场时间 — 基于 contract_role 的期望出场率动态计算
            let expected_rate: f64 = match player.contract_role.as_str() {
                "Sub" => 0.20,
                "Prospect" => 0.05,
                _ => 0.75, // Starter
            };
            let rate_gap = expected_rate - play_rate; // >0 表示出场不足
            if rate_gap > 0.0 {
                // 出场低于预期：penalty = gap × 25 × (ability/50)
                let ability_factor = player.ability as f64 / 50.0;
                sat_change -= (rate_gap * 25.0 * ability_factor).round() as i32;
            } else if play_rate >= 0.8 {
                sat_change += 8;
            } else if play_rate >= expected_rate {
                sat_change += 3;
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
            if player.age <= 24 && play_rate < 0.5 {
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

        let mastery_rows = sqlx::query(
            "SELECT pcm.player_id, pcm.champion_id, pcm.mastery_tier, pcm.games_played, pcm.games_won, p.game_id, p.team_id, p.ability
             FROM player_champion_mastery pcm
             JOIN players p ON pcm.player_id = p.id AND pcm.save_id = p.save_id
             WHERE pcm.save_id = ? AND p.status = 'Active'"
        )
        .bind(save_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("查询英雄熟练度失败: {}", e))?;

        let mut upgrade_count = 0u32;
        let mut downgrade_count = 0u32;

        for row in &mastery_rows {
            let player_id: i64 = row.get("player_id");
            let champion_id: i64 = row.get("champion_id");
            let tier_str: String = row.get("mastery_tier");
            let games_played: i64 = row.get("games_played");
            let games_won: i64 = row.get("games_won");
            let game_id: String = row.get("game_id");
            let team_id: Option<i64> = row.try_get::<i64, _>("team_id").ok();
            let ability: i64 = row.get("ability");

            let current = match MasteryTier::from_id(&tier_str) {
                Some(t) => t,
                None => continue,
            };

            let new_tier = champion::evolve_mastery(current, games_played as u32, games_won as u32, &mut rng);

            if new_tier != current {
                if new_tier == MasteryTier::B {
                    sqlx::query(
                        "DELETE FROM player_champion_mastery WHERE save_id = ? AND player_id = ? AND champion_id = ?"
                    )
                    .bind(save_id)
                    .bind(player_id)
                    .bind(champion_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("删除B级熟练度失败: {}", e))?;
                } else {
                    sqlx::query(
                        "UPDATE player_champion_mastery SET mastery_tier = ?, games_played = 0 WHERE save_id = ? AND player_id = ? AND champion_id = ?"
                    )
                    .bind(new_tier.id())
                    .bind(save_id)
                    .bind(player_id)
                    .bind(champion_id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("更新熟练度失败: {}", e))?;
                }

                let upgraded = new_tier.pick_score() > current.pick_score();
                if upgraded {
                    upgrade_count += 1;
                } else {
                    downgrade_count += 1;
                }

                let champion_name = champion::get_champion(champion_id as u8)
                    .map(|c| c.name_cn.to_string())
                    .unwrap_or_else(|| format!("英雄#{}", champion_id));

                let desc = if upgraded {
                    format!(
                        "{}的{}熟练度提升: {} → {}",
                        game_id,
                        champion_name,
                        current.display_name(),
                        new_tier.display_name()
                    )
                } else {
                    format!(
                        "{}的{}熟练度下降: {} → {}",
                        game_id,
                        champion_name,
                        current.display_name(),
                        new_tier.display_name()
                    )
                };

                let event = self.record_event(
                    pool, window_id, 1,
                    TransferEventType::SeasonSettlement,
                    EventLevel::C,
                    player_id, &game_id, ability,
                    team_id, team_id.and_then(|tid| cache.team_names.get(&tid).map(|s| s.as_str())),
                    team_id, team_id.and_then(|tid| cache.team_names.get(&tid).map(|s| s.as_str())),
                    0, 0, 0,
                    &desc,
                ).await?;
                events.push(event);
            } else {
                sqlx::query(
                    "UPDATE player_champion_mastery SET games_played = 0 WHERE save_id = ? AND player_id = ? AND champion_id = ?"
                )
                .bind(save_id)
                .bind(player_id)
                .bind(champion_id)
                .execute(pool)
                .await
                .map_err(|e| format!("重置使用次数失败: {}", e))?;
            }
        }

        eprintln!("[R1] 英雄池演变完成: {} 升级, {} 降级", upgrade_count, downgrade_count);

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
