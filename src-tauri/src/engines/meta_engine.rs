//! 版本更新引擎 (Meta Engine)
//!
//! 管理电竞赛季的版本（Meta）系统：
//! - 20 种不同的 Meta 类型，每种定义 5 个位置的权重
//! - 每赛季随机切换版本（S1 固定为 Balanced，不连续重复）
//! - 加权平均 + carry/drag 效应的队伍战力计算

use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use super::champion::Archetype;

/// Meta 类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetaType {
    Balanced,
    MidKingdom,
    BotLaneDominance,
    TopLaneCarry,
    JungleTempo,
    SupportEra,
    DualCarry,
    SoloLaneMeta,
    TeamfightMeta,
    EarlyGameAggro,
    LateGameScaling,
    SplitPushMeta,
    VisionControl,
    PickComposition,
    ProtectTheCarry,
    DiveComposition,
    SkirmishMeta,
    ObjectiveControl,
    MidJungleSynergy,
    TopJungleSynergy,
}

impl MetaType {
    /// 获取所有 Meta 类型
    pub fn all() -> &'static [MetaType] {
        &[
            MetaType::Balanced,
            MetaType::MidKingdom,
            MetaType::BotLaneDominance,
            MetaType::TopLaneCarry,
            MetaType::JungleTempo,
            MetaType::SupportEra,
            MetaType::DualCarry,
            MetaType::SoloLaneMeta,
            MetaType::TeamfightMeta,
            MetaType::EarlyGameAggro,
            MetaType::LateGameScaling,
            MetaType::SplitPushMeta,
            MetaType::VisionControl,
            MetaType::PickComposition,
            MetaType::ProtectTheCarry,
            MetaType::DiveComposition,
            MetaType::SkirmishMeta,
            MetaType::ObjectiveControl,
            MetaType::MidJungleSynergy,
            MetaType::TopJungleSynergy,
        ]
    }

    /// Meta ID 字符串
    pub fn id(&self) -> &'static str {
        match self {
            MetaType::Balanced => "Balanced",
            MetaType::MidKingdom => "MidKingdom",
            MetaType::BotLaneDominance => "BotLaneDominance",
            MetaType::TopLaneCarry => "TopLaneCarry",
            MetaType::JungleTempo => "JungleTempo",
            MetaType::SupportEra => "SupportEra",
            MetaType::DualCarry => "DualCarry",
            MetaType::SoloLaneMeta => "SoloLaneMeta",
            MetaType::TeamfightMeta => "TeamfightMeta",
            MetaType::EarlyGameAggro => "EarlyGameAggro",
            MetaType::LateGameScaling => "LateGameScaling",
            MetaType::SplitPushMeta => "SplitPushMeta",
            MetaType::VisionControl => "VisionControl",
            MetaType::PickComposition => "PickComposition",
            MetaType::ProtectTheCarry => "ProtectTheCarry",
            MetaType::DiveComposition => "DiveComposition",
            MetaType::SkirmishMeta => "SkirmishMeta",
            MetaType::ObjectiveControl => "ObjectiveControl",
            MetaType::MidJungleSynergy => "MidJungleSynergy",
            MetaType::TopJungleSynergy => "TopJungleSynergy",
        }
    }

    /// 中文名称
    pub fn display_name(&self) -> &'static str {
        match self {
            MetaType::Balanced => "均衡版本",
            MetaType::MidKingdom => "中路为王",
            MetaType::BotLaneDominance => "下路统治",
            MetaType::TopLaneCarry => "上单Carry",
            MetaType::JungleTempo => "打野节奏",
            MetaType::SupportEra => "辅助时代",
            MetaType::DualCarry => "双C输出",
            MetaType::SoloLaneMeta => "单人线版本",
            MetaType::TeamfightMeta => "团战版本",
            MetaType::EarlyGameAggro => "前期进攻",
            MetaType::LateGameScaling => "后期发育",
            MetaType::SplitPushMeta => "分推版本",
            MetaType::VisionControl => "视野控制",
            MetaType::PickComposition => "抓单阵容",
            MetaType::ProtectTheCarry => "保护输出",
            MetaType::DiveComposition => "开团版本",
            MetaType::SkirmishMeta => "小规模团战",
            MetaType::ObjectiveControl => "资源控制",
            MetaType::MidJungleSynergy => "中野联动",
            MetaType::TopJungleSynergy => "上野联动",
        }
    }

    /// 版本描述
    pub fn description(&self) -> &'static str {
        match self {
            MetaType::Balanced => "各位置均衡发展，没有明显强势位置",
            MetaType::MidKingdom => "中路选手主导比赛节奏，中路权重大幅提升",
            MetaType::BotLaneDominance => "下路组合统治比赛，ADC是核心输出",
            MetaType::TopLaneCarry => "上单选手可以独自Carry比赛",
            MetaType::JungleTempo => "打野主导前期节奏，高权重影响比赛走向",
            MetaType::SupportEra => "辅助选手的视野和团战发起至关重要",
            MetaType::DualCarry => "中路和ADC双核输出，双C联动版本",
            MetaType::SoloLaneMeta => "上单和中单等单人线选手更有影响力",
            MetaType::TeamfightMeta => "团战频发的版本，中路和ADC贡献突出",
            MetaType::EarlyGameAggro => "前期进攻节奏快，打野主导早期gank",
            MetaType::LateGameScaling => "后期阵容发育版本，ADC权重大幅提升",
            MetaType::SplitPushMeta => "分推战术盛行，上单承担分推重任",
            MetaType::VisionControl => "视野控制决定比赛，打野和辅助协同布控",
            MetaType::PickComposition => "抓单阵容盛行，打野和中路协同抓人",
            MetaType::ProtectTheCarry => "保护ADC输出的阵容成为主流",
            MetaType::DiveComposition => "开团阵容盛行，上单和打野联合开团",
            MetaType::SkirmishMeta => "小规模团战频繁，打野和中路主导小团",
            MetaType::ObjectiveControl => "资源控制至关重要，打野主导龙和峡谷先锋",
            MetaType::MidJungleSynergy => "中野联动版本，中路和打野的配合是关键",
            MetaType::TopJungleSynergy => "上野联动版本，上路和打野协同推进优势",
        }
    }

    /// 从字符串解析 MetaType
    pub fn from_id(id: &str) -> Option<MetaType> {
        MetaType::all().iter().find(|m| m.id() == id).copied()
    }

    pub fn favored_archetypes(&self) -> &'static [Archetype] {
        match self {
            MetaType::Balanced          => &[],
            MetaType::EarlyGameAggro    => &[Archetype::Aggressive],
            MetaType::DiveComposition   => &[Archetype::Aggressive],
            MetaType::SkirmishMeta      => &[Archetype::Aggressive],
            MetaType::PickComposition   => &[Archetype::Aggressive, Archetype::Utility],
            MetaType::LateGameScaling   => &[Archetype::Scaling],
            MetaType::ProtectTheCarry   => &[Archetype::Scaling, Archetype::Utility],
            MetaType::DualCarry         => &[Archetype::Scaling],
            MetaType::VisionControl     => &[Archetype::Utility],
            MetaType::SupportEra        => &[Archetype::Utility],
            MetaType::SplitPushMeta     => &[Archetype::Splitpush],
            MetaType::SoloLaneMeta      => &[Archetype::Splitpush],
            MetaType::TeamfightMeta     => &[Archetype::Teamfight],
            MetaType::ObjectiveControl  => &[Archetype::Teamfight, Archetype::Utility],
            MetaType::MidKingdom        => &[Archetype::Aggressive, Archetype::Scaling],
            MetaType::BotLaneDominance  => &[Archetype::Scaling, Archetype::Utility],
            MetaType::TopLaneCarry      => &[Archetype::Splitpush, Archetype::Aggressive],
            MetaType::JungleTempo       => &[Archetype::Aggressive],
            MetaType::MidJungleSynergy  => &[Archetype::Aggressive, Archetype::Teamfight],
            MetaType::TopJungleSynergy  => &[Archetype::Splitpush, Archetype::Aggressive],
        }
    }
}

/// 位置权重（5 个位置的权重之和 = 5.0）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWeights {
    pub top: f64,
    pub jug: f64,
    pub mid: f64,
    pub adc: f64,
    pub sup: f64,
}

impl MetaWeights {
    /// 根据位置字符串获取对应权重
    pub fn weight_for_position(&self, position: &str) -> f64 {
        match position.to_uppercase().as_str() {
            "TOP" => self.top,
            "JUG" | "JUNGLE" => self.jug,
            "MID" => self.mid,
            "ADC" | "BOT" => self.adc,
            "SUP" | "SUPPORT" => self.sup,
            _ => 1.0, // 默认均衡权重
        }
    }

    /// 均衡权重
    pub fn balanced() -> Self {
        MetaWeights { top: 1.0, jug: 1.0, mid: 1.0, adc: 1.0, sup: 1.0 }
    }
}

/// 获取 MetaType 对应的权重配置
pub fn get_meta_weights(meta: MetaType) -> MetaWeights {
    match meta {
        MetaType::Balanced          => MetaWeights { top: 1.00, jug: 1.00, mid: 1.00, adc: 1.00, sup: 1.00 },
        MetaType::MidKingdom        => MetaWeights { top: 0.85, jug: 0.90, mid: 1.40, adc: 0.95, sup: 0.90 },
        MetaType::BotLaneDominance  => MetaWeights { top: 0.80, jug: 0.90, mid: 0.90, adc: 1.35, sup: 1.05 },
        MetaType::TopLaneCarry      => MetaWeights { top: 1.35, jug: 0.90, mid: 0.90, adc: 0.90, sup: 0.95 },
        MetaType::JungleTempo       => MetaWeights { top: 0.85, jug: 1.40, mid: 0.90, adc: 0.95, sup: 0.90 },
        MetaType::SupportEra        => MetaWeights { top: 0.90, jug: 0.90, mid: 0.90, adc: 0.90, sup: 1.40 },
        MetaType::DualCarry         => MetaWeights { top: 0.80, jug: 0.80, mid: 1.20, adc: 1.30, sup: 0.90 },
        MetaType::SoloLaneMeta      => MetaWeights { top: 1.25, jug: 0.80, mid: 1.20, adc: 0.85, sup: 0.90 },
        MetaType::TeamfightMeta     => MetaWeights { top: 0.90, jug: 0.90, mid: 1.10, adc: 1.15, sup: 0.95 },
        MetaType::EarlyGameAggro    => MetaWeights { top: 0.90, jug: 1.30, mid: 1.00, adc: 0.90, sup: 0.90 },
        MetaType::LateGameScaling   => MetaWeights { top: 0.85, jug: 0.85, mid: 1.05, adc: 1.30, sup: 0.95 },
        MetaType::SplitPushMeta     => MetaWeights { top: 1.30, jug: 0.90, mid: 1.00, adc: 0.85, sup: 0.95 },
        MetaType::VisionControl     => MetaWeights { top: 0.85, jug: 1.10, mid: 0.90, adc: 0.85, sup: 1.30 },
        MetaType::PickComposition   => MetaWeights { top: 0.90, jug: 1.25, mid: 1.10, adc: 0.85, sup: 0.90 },
        MetaType::ProtectTheCarry   => MetaWeights { top: 0.80, jug: 0.85, mid: 0.90, adc: 1.35, sup: 1.10 },
        MetaType::DiveComposition   => MetaWeights { top: 1.10, jug: 1.15, mid: 0.95, adc: 0.85, sup: 0.95 },
        MetaType::SkirmishMeta      => MetaWeights { top: 0.95, jug: 1.20, mid: 1.15, adc: 0.80, sup: 0.90 },
        MetaType::ObjectiveControl  => MetaWeights { top: 0.90, jug: 1.25, mid: 0.95, adc: 1.00, sup: 0.90 },
        MetaType::MidJungleSynergy  => MetaWeights { top: 0.80, jug: 1.20, mid: 1.25, adc: 0.85, sup: 0.90 },
        MetaType::TopJungleSynergy  => MetaWeights { top: 1.20, jug: 1.20, mid: 0.85, adc: 0.85, sup: 0.90 },
    }
}

/// Carry/Drag 效应常量
const CARRY_RATE: f64 = 0.3;
const DRAG_RATE: f64 = 0.5;

/// Meta 引擎
pub struct MetaEngine;

impl MetaEngine {
    /// 为新赛季生成 Meta 版本并写入数据库
    /// S1 固定为 Balanced，S2+ 随机抽取（不连续重复）
    pub async fn roll_new_meta(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<MetaType, String> {
        let meta = if season_id == 1 {
            MetaType::Balanced
        } else {
            // 查询上一赛季的 meta
            let prev_meta: Option<String> = sqlx::query_scalar(
                "SELECT meta_type FROM meta_versions WHERE save_id = ? AND season_id = ? LIMIT 1"
            )
            .bind(save_id)
            .bind(season_id - 1)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("查询上赛季版本失败: {}", e))?;

            let mut rng = rand::thread_rng();
            let all_metas = MetaType::all();
            loop {
                let idx = rng.gen_range(0..all_metas.len());
                let candidate = all_metas[idx];
                // 不连续重复
                if let Some(ref prev) = prev_meta {
                    if candidate.id() == prev.as_str() {
                        continue;
                    }
                }
                break candidate;
            }
        };

        // 获取权重
        let weights = get_meta_weights(meta);

        // 写入数据库
        sqlx::query(
            r#"
            INSERT INTO meta_versions (save_id, season_id, meta_type, meta_name, weight_top, weight_jug, weight_mid, weight_adc, weight_sup)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(save_id, season_id) DO UPDATE SET
                meta_type = excluded.meta_type,
                meta_name = excluded.meta_name,
                weight_top = excluded.weight_top,
                weight_jug = excluded.weight_jug,
                weight_mid = excluded.weight_mid,
                weight_adc = excluded.weight_adc,
                weight_sup = excluded.weight_sup
            "#,
        )
        .bind(save_id)
        .bind(season_id)
        .bind(meta.id())
        .bind(meta.display_name())
        .bind(weights.top)
        .bind(weights.jug)
        .bind(weights.mid)
        .bind(weights.adc)
        .bind(weights.sup)
        .execute(pool)
        .await
        .map_err(|e| format!("写入 Meta 版本失败: {}", e))?;

        log::info!("S{} 版本: {} ({})", season_id, meta.display_name(), meta.id());
        Ok(meta)
    }

    /// 获取当前赛季的 Meta 权重，如果没有则返回均衡权重
    pub async fn get_current_weights(
        pool: &Pool<Sqlite>,
        save_id: &str,
        season_id: i64,
    ) -> Result<MetaWeights, String> {
        let row = sqlx::query(
            "SELECT weight_top, weight_jug, weight_mid, weight_adc, weight_sup FROM meta_versions WHERE save_id = ? AND season_id = ?"
        )
        .bind(save_id)
        .bind(season_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询 Meta 权重失败: {}", e))?;

        match row {
            Some(r) => Ok(MetaWeights {
                top: r.get("weight_top"),
                jug: r.get("weight_jug"),
                mid: r.get("weight_mid"),
                adc: r.get("weight_adc"),
                sup: r.get("weight_sup"),
            }),
            None => Ok(MetaWeights::balanced()),
        }
    }

    /// 加权平均 + carry/drag 效应计算队伍战力
    ///
    /// 输入: 5 名选手的 (actual_ability, position) + meta 权重
    /// 输出: team_power
    pub fn calculate_team_power_weighted(
        players: &[(f64, &str)], // (actual_ability, position)
        weights: &MetaWeights,
    ) -> f64 {
        if players.is_empty() {
            return 0.0;
        }

        // 第1步：加权均值
        let weighted_sum: f64 = players.iter()
            .map(|(ability, pos)| weights.weight_for_position(pos) * ability)
            .sum();
        let weighted_avg = weighted_sum / 5.0;

        // 第2步：carry/drag 效应
        let carry_drag: f64 = players.iter().map(|(ability, _pos)| {
            if *ability > weighted_avg {
                (*ability - weighted_avg) * CARRY_RATE
            } else if *ability < weighted_avg {
                (*ability - weighted_avg) * DRAG_RATE
            } else {
                0.0
            }
        }).sum();

        // 第3步：最终战力
        weighted_avg + carry_drag
    }
}

// ===== 单元测试 =====
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_meta_weights_sum_to_5() {
        for meta in MetaType::all() {
            let w = get_meta_weights(*meta);
            let sum = w.top + w.jug + w.mid + w.adc + w.sup;
            assert!(
                (sum - 5.0).abs() < 0.001,
                "{:?} 权重之和 = {}, 期望 5.0",
                meta, sum
            );
        }
    }

    #[test]
    fn test_balanced_equals_simple_average() {
        // 均衡版本下，加权平均应等于简单平均
        let players = vec![
            (80.0, "TOP"),
            (70.0, "JUG"),
            (90.0, "MID"),
            (85.0, "ADC"),
            (65.0, "SUP"),
        ];
        let weights = MetaWeights::balanced();

        // 简单平均
        let simple_avg: f64 = players.iter().map(|(a, _)| a).sum::<f64>() / 5.0;

        // 加权平均（均衡版本下权重全 1.0）
        let weighted_avg = players.iter()
            .map(|(a, p)| weights.weight_for_position(p) * a)
            .sum::<f64>() / 5.0;

        assert!(
            (simple_avg - weighted_avg).abs() < 0.001,
            "均衡版本: 简单平均 {} ≠ 加权平均 {}", simple_avg, weighted_avg
        );
    }

    #[test]
    fn test_balanced_carry_drag_zero_for_equal_players() {
        // 5 名选手能力相同时，carry/drag 应为 0
        let players = vec![
            (75.0, "TOP"),
            (75.0, "JUG"),
            (75.0, "MID"),
            (75.0, "ADC"),
            (75.0, "SUP"),
        ];
        let weights = MetaWeights::balanced();
        let power = MetaEngine::calculate_team_power_weighted(&players, &weights);
        assert!(
            (power - 75.0).abs() < 0.001,
            "等能力选手战力应为 75.0, 实际 {}", power
        );
    }

    #[test]
    fn test_carry_drag_effect() {
        // 一个超强选手 + 一个超弱选手，drag 应大于 carry
        let players = vec![
            (90.0, "TOP"),
            (78.0, "JUG"),
            (78.0, "MID"),
            (78.0, "ADC"),
            (40.0, "SUP"), // 短板
        ];
        let weights = MetaWeights::balanced();
        let power = MetaEngine::calculate_team_power_weighted(&players, &weights);
        let simple_avg = (90.0 + 78.0 + 78.0 + 78.0 + 40.0) / 5.0; // 72.8

        // 由于 drag_rate > carry_rate，有短板时战力应低于简单平均
        assert!(
            power < simple_avg,
            "有短板时战力 {} 应低于简单平均 {}", power, simple_avg
        );
    }

    #[test]
    fn test_meta_weight_affects_weighted_avg() {
        // 中路为王版本下，强中单的加权均值应高于均衡版本的加权均值
        // （carry/drag 效应可能改变最终排序，但加权均值本身应体现 meta 的影响）
        let players = vec![
            (75.0, "TOP"),
            (75.0, "JUG"),
            (90.0, "MID"), // 强中单
            (75.0, "ADC"),
            (75.0, "SUP"),
        ];

        let balanced = MetaWeights::balanced();
        let mid_kingdom = get_meta_weights(MetaType::MidKingdom);

        // 计算加权均值（不含 carry/drag）
        let weighted_avg_balanced: f64 = players.iter()
            .map(|(a, p)| balanced.weight_for_position(p) * a)
            .sum::<f64>() / 5.0;
        let weighted_avg_mid_kingdom: f64 = players.iter()
            .map(|(a, p)| mid_kingdom.weight_for_position(p) * a)
            .sum::<f64>() / 5.0;

        assert!(
            weighted_avg_mid_kingdom > weighted_avg_balanced,
            "中路为王加权均值 {} 应高于均衡版本 {}",
            weighted_avg_mid_kingdom, weighted_avg_balanced
        );
    }

    #[test]
    fn test_meta_benefits_matching_team() {
        // 均衡队伍（所有人能力相同）在不同 meta 下战力应相同
        // 因为权重之和都是 5.0，且 carry_drag = 0
        let equal_players = vec![
            (80.0, "TOP"),
            (80.0, "JUG"),
            (80.0, "MID"),
            (80.0, "ADC"),
            (80.0, "SUP"),
        ];

        let balanced = MetaWeights::balanced();
        let mid_kingdom = get_meta_weights(MetaType::MidKingdom);

        let power_balanced = MetaEngine::calculate_team_power_weighted(&equal_players, &balanced);
        let power_mid_kingdom = MetaEngine::calculate_team_power_weighted(&equal_players, &mid_kingdom);

        assert!(
            (power_balanced - power_mid_kingdom).abs() < 0.001,
            "均衡队伍在不同 Meta 下战力应相同: {} vs {}",
            power_balanced, power_mid_kingdom
        );
    }

    #[test]
    fn test_20_meta_types() {
        assert_eq!(MetaType::all().len(), 20, "应有 20 种 Meta 类型");
    }

    #[test]
    fn test_meta_id_roundtrip() {
        for meta in MetaType::all() {
            let parsed = MetaType::from_id(meta.id());
            assert_eq!(parsed, Some(*meta), "{:?} ID 解析失败", meta);
        }
    }

    #[test]
    fn test_weight_for_position_variants() {
        let w = get_meta_weights(MetaType::MidKingdom);
        assert!((w.weight_for_position("MID") - 1.40).abs() < 0.001);
        assert!((w.weight_for_position("mid") - 1.40).abs() < 0.001);
        assert!((w.weight_for_position("TOP") - 0.85).abs() < 0.001);
        assert!((w.weight_for_position("JUNGLE") - 0.90).abs() < 0.001);
        assert!((w.weight_for_position("BOT") - 0.95).abs() < 0.001);
        assert!((w.weight_for_position("SUPPORT") - 0.90).abs() < 0.001);
    }
}
