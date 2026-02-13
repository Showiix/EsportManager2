use rand::Rng;

use crate::models::transfer::*;

use super::cache::{CachedPlayer, CachedPlayerStats};
use super::TransferEngine;

impl TransferEngine {
    /// 计算战绩稳定性评分
    #[allow(overlapping_range_endpoints)]
    pub(crate) fn calculate_stability_score(&self, current_rank: i32, last_rank: i32) -> i32 {
        let change = current_rank - last_rank; // 正数=下滑

        match (last_rank, change) {
            // 卫冕冠军/亚军
            (1, -1..=1) => 100, // 冠军→冠亚军：极稳定
            (1, 2..=3) => 70,   // 冠军→3-4名：可接受
            (1, 4..) => 30,     // 冠军→5名开外：危机
            (2, -2..=1) => 95,  // 亚军维持：稳定

            // 上赛季前4
            (3..=4, ..=-1) => 95, // 进步：稳定
            (3..=4, 0..=2) => 85, // 维持：稳定
            (3..=4, 3..=5) => 55, // 下滑：警惕
            (3..=4, 6..) => 30,   // 大幅下滑：必须调整

            // 上赛季5-8名
            (5..=8, ..=-3) => 95,  // 大幅上升：稳定
            (5..=8, -2..=2) => 80, // 维持：稳定
            (5..=8, 3..) => 50,    // 下滑：考虑调整

            // 上赛季9-14名（中下游队伍）
            (9..=14, ..=-4) => 95,   // 大幅进步：稳定
            (9..=14, -3..=-1) => 85, // 进步：稳定
            (9..=14, 0..=2) => 75,   // 维持：基本稳定
            (9..=14, 3..) => 45,     // 下滑：考虑调整

            // 其他情况
            (_, ..=-3) => 90,   // 大幅进步
            (_, -2..=-1) => 80, // 进步
            (_, 0..=1) => 70,   // 维持
            (_, 2..) => 40,     // 下滑
        }
    }

    /// 决定战队策略
    pub(crate) fn determine_team_strategy(
        &self,
        stability_score: i32,
        current_rank: i32,
        roster_power: f64,
        roster_age_avg: f64,
    ) -> (String, String, String) {
        let (strategy, urgency, reason) = if stability_score >= 90 {
            (
                "DYNASTY",
                "NONE",
                format!("战绩稳定，排名{}，无需变动", current_rank),
            )
        } else if stability_score >= 70 {
            (
                "MAINTAIN",
                "LOW",
                format!("战绩尚可，排名{}，可小幅调整", current_rank),
            )
        } else if stability_score >= 40 {
            if roster_age_avg > 26.0 {
                (
                    "UPGRADE",
                    "MEDIUM",
                    format!(
                        "战绩下滑且阵容老化，平均年龄{:.1}岁，需要补强",
                        roster_age_avg
                    ),
                )
            } else {
                (
                    "UPGRADE",
                    "MEDIUM",
                    format!("战绩下滑，排名{}，需要补强", current_rank),
                )
            }
        } else {
            if roster_power < 75.0 {
                (
                    "REBUILD",
                    "HIGH",
                    format!("战绩大幅下滑，阵容战力{:.1}偏低，需要重建", roster_power),
                )
            } else {
                (
                    "REBUILD",
                    "HIGH",
                    format!("战绩大幅下滑，排名从前列跌落，需要大幅调整"),
                )
            }
        };

        (strategy.to_string(), urgency.to_string(), reason)
    }

    /// 计算匹配度（0-100）
    pub(crate) fn calculate_match_score(
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
        let pos_players: Vec<&CachedPlayer> =
            roster.iter().filter(|p| p.position == position).collect();
        let pos_count = pos_players.len();
        let need_score = match pos_count {
            0 => 100.0, // 该位置空缺，急需
            1 => 40.0,  // 已有首发，仅轻度需求
            2 => 15.0,  // 饱和
            _ => 5.0,   // 超饱和
        };

        // 5. 提升度（0-100）：选手能力相对于球队该位置最强选手的提升
        let best_at_pos = pos_players.iter().map(|p| p.ability).max().unwrap_or(0);
        let upgrade_score = if pos_count == 0 {
            // 空位，能力直接映射
            (ability as f64).clamp(40.0, 100.0)
        } else {
            let diff = ability as i64 - best_at_pos;
            match diff {
                d if d >= 10 => 100.0, // 大幅提升
                d if d >= 5 => 85.0,   // 明显提升
                d if d >= 0 => 65.0,   // 略有提升或持平
                d if d >= -5 => 45.0,  // 略弱于现有
                _ => 25.0,             // 明显弱于现有
            }
        };

        // 6. 排名因子（弱队更渴望强援）
        let rank_factor = match team_rank {
            1..=3 => 0.9,   // 强队，选人更挑剔
            4..=7 => 1.0,   // 中游
            8..=10 => 1.05, // 中下游，更积极
            11..=14 => 1.1, // 弱队，急需补强
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
        let w_ability = 0.25 + 0.15 * weights.short_term_focus; // 0.25 ~ 0.40
        let w_age = 0.15 + 0.15 * weights.youth_preference.max(weights.short_term_focus); // 0.15 ~ 0.30
        let w_finance = 0.10 + 0.10 * weights.bargain_hunting; // 0.10 ~ 0.20
        let w_need = 0.20; // 固定 0.20
        let w_upgrade = 0.15 + 0.10 * weights.short_term_focus; // 0.15 ~ 0.25
                                                                // 潜力权重受AI性格影响：发展型球队更看重潜力
        let w_potential = 0.05 + 0.10 * weights.youth_preference; // 0.05 ~ 0.15
                                                                  // 稳定性权重受AI性格影响：保守型球队更看重稳定性
        let w_stability = 0.05 + 0.05 * (1.0 - weights.risk_tolerance); // 0.05 ~ 0.10
        let total_w =
            w_ability + w_age + w_finance + w_need + w_upgrade + w_potential + w_stability;

        // 归一化后加权求和，再乘以排名因子和成长标签乘数
        let raw = (ability_score * w_ability
            + age_score * w_age
            + finance_score * w_finance
            + need_score * w_need
            + upgrade_score * w_upgrade
            + potential_score * w_potential
            + stability_score * w_stability)
            / total_w;

        (raw * rank_factor * tag_multiplier).clamp(0.0, 100.0)
    }

    /// 计算球员转会意愿（0-100）
    /// 8 因素 + 年龄优先级权重系统
    pub(crate) fn calculate_willingness(
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
        player_stats: Option<&CachedPlayerStats>,
        rng: &mut impl Rng,
    ) -> f64 {
        // 1. 薪资满意度（20-100）
        let salary_ratio = if current_salary > 0 {
            offered_salary as f64 / current_salary as f64
        } else {
            1.5
        };
        let salary_score = if salary_ratio >= 1.2 {
            100.0
        } else if salary_ratio >= 1.0 {
            80.0
        } else if salary_ratio >= 0.8 {
            60.0
        } else if salary_ratio >= 0.6 {
            40.0
        } else {
            20.0
        };

        // 2. 球队竞争力（20-100）：基于目标球队排名
        let competitiveness_score = match target_team_rank {
            1..=3 => 100.0,
            4..=6 => 80.0,
            7..=10 => 60.0,
            11..=14 => 40.0,
            _ => 20.0,
        };

        // 3. 首发机会（30-100）：比较自己能力 vs 目标队该位置首发能力
        let best_at_pos = target_roster
            .iter()
            .filter(|p| p.position == position)
            .map(|p| p.ability)
            .max()
            .unwrap_or(0);
        let starting_chance_score = if best_at_pos == 0 {
            100.0 // 该位置空缺，必定首发
        } else {
            let diff = ability as i64 - best_at_pos;
            if diff >= 5 {
                100.0
            }
            // 明显更强
            else if diff >= 0 {
                85.0
            }
            // 略强或持平
            else if diff >= -5 {
                70.0
            }
            // 略弱，有竞争
            else {
                30.0
            } // 明显更弱
        };

        // 4. 球队声望（20-100）：基于 target_team_reputation 线性映射
        let reputation_score =
            (target_team_reputation as f64 / 100.0 * 80.0 + 20.0).clamp(20.0, 100.0);

        // 5. 队友质量（30-100）：目标队平均能力映射
        let avg_ability = if target_roster.is_empty() {
            50.0
        } else {
            target_roster.iter().map(|p| p.ability as f64).sum::<f64>() / target_roster.len() as f64
        };
        let teammate_quality_score = if avg_ability >= 70.0 {
            100.0
        } else if avg_ability >= 65.0 {
            80.0
        } else if avg_ability >= 60.0 {
            65.0
        } else {
            40.0
        };

        // 6. 忠诚影响（0-50）
        let loyalty_factor = (100.0 - loyalty as f64) * 0.5;

        // 7. 发展空间（30-100）：仅对年轻选手有效
        let development_score = if age <= 23 {
            // 检查目标队是否有高能力同位置老将可学习
            let has_mentor = target_roster
                .iter()
                .any(|p| p.position == position && p.age >= 26 && p.ability >= 70);
            let team_avg_high = avg_ability >= 65.0;
            if has_mentor && team_avg_high {
                100.0
            } else if has_mentor || team_avg_high {
                75.0
            } else {
                45.0
            }
        } else {
            50.0 // 非年轻选手，发展空间中性
        };

        // 8. 随机波动（-8 ~ +8）
        let random_noise: f64 = rng.gen_range(-8.0..8.0);

        // 年龄优先级权重系统
        let (w_salary, w_compete, w_start, w_reputation, w_teammate, w_develop) = match age {
            17..=21 => (0.10, 0.10, 0.25, 0.10, 0.10, 0.20), // 新秀期
            22..=25 => (0.15, 0.20, 0.20, 0.10, 0.10, 0.10), // 成长期
            26..=28 => (0.20, 0.30, 0.10, 0.15, 0.10, 0.00), // 巅峰期
            29..=31 => (0.35, 0.15, 0.10, 0.15, 0.10, 0.00), // 老将期
            _ => (0.40, 0.10, 0.10, 0.15, 0.10, 0.00),       // 退役前
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

        // 国际赛表现加成：大赛型选手更受强队吸引
        let intl_bonus = if let Some(stats) = player_stats {
            if stats.international_games >= 5 && stats.international_avg_impact > 1.0 {
                if target_team_rank <= 4 {
                    8.0
                } else if target_team_rank <= 8 {
                    4.0
                } else {
                    -3.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // momentum 加成：状态好的选手更自信（要价更高，对弱队更挑剔）
        let momentum_bonus = if let Some(stats) = player_stats {
            let m = stats.momentum;
            if m >= 3 {
                if target_team_rank <= 6 {
                    5.0
                } else {
                    -3.0
                }
            } else if m <= -3 {
                3.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let adjusted_willingness =
            (base_willingness + intl_bonus + momentum_bonus).clamp(0.0, 100.0);

        // 跨赛区惩罚
        let cross_region_factor = match (home_region_id, target_region_id) {
            (Some(home), Some(target)) if home != target => (100.0 - region_loyalty as f64) / 100.0,
            _ => 1.0,
        };

        (adjusted_willingness * cross_region_factor).clamp(0.0, 100.0)
    }
}
