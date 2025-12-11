//! 选手特性系统
//!
//! 特性影响选手在不同情境下的表现，通过修改 ability/stability/condition 实现
//! 完全解耦，不影响核心模拟引擎

use serde::{Deserialize, Serialize};
use super::condition::MatchContext;

/// 特性类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraitType {
    // === 大赛表现类 ===
    /// 大赛型：季后赛/国际赛 condition +3
    Clutch,
    /// 慢热型：第1局 condition -2，第3+局 +2
    SlowStarter,
    /// 快枪手：第1局 condition +2，第3+局 -1
    FastStarter,

    // === 稳定性类 ===
    /// 爆发型：stability -15，能力上限 +5
    Explosive,
    /// 稳定型：stability +10，能力上限 -3
    Consistent,

    // === 心态类 ===
    /// 逆风王：落后时 condition +3
    ComebackKing,
    /// 顺风浪：领先时 condition -2，落后时 -3
    Tilter,
    /// 心态大师：momentum 效果减半
    MentalFortress,
    /// 玻璃心：输了 momentum -2（而非-1）
    Fragile,

    // === 体能类 ===
    /// 铁人：无疲劳惩罚
    Ironman,
    /// 状态敏感：condition 波动 ×1.5
    Volatile,

    // === 特殊类 ===
    /// 新星：首个赛季 ability +3
    RisingStar,
    /// 老将风范：30岁后 stability +15
    Veteran,
    /// 团队核心：队友 condition +1（需要特殊处理）
    TeamLeader,
}

impl TraitType {
    /// 获取特性的中文名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TraitType::Clutch => "大赛型",
            TraitType::SlowStarter => "慢热型",
            TraitType::FastStarter => "快枪手",
            TraitType::Explosive => "爆发型",
            TraitType::Consistent => "稳定型",
            TraitType::ComebackKing => "逆风王",
            TraitType::Tilter => "顺风浪",
            TraitType::MentalFortress => "心态大师",
            TraitType::Fragile => "玻璃心",
            TraitType::Ironman => "铁人",
            TraitType::Volatile => "状态敏感",
            TraitType::RisingStar => "新星",
            TraitType::Veteran => "老将风范",
            TraitType::TeamLeader => "团队核心",
        }
    }

    /// 获取特性描述
    pub fn description(&self) -> &'static str {
        match self {
            TraitType::Clutch => "在季后赛和国际赛中状态更好",
            TraitType::SlowStarter => "系列赛开局较慢，但后期渐入佳境",
            TraitType::FastStarter => "系列赛开局强势，但后期可能疲软",
            TraitType::Explosive => "发挥波动大，但巅峰更高",
            TraitType::Consistent => "发挥稳定，但上限略低",
            TraitType::ComebackKing => "落后时愈战愈勇",
            TraitType::Tilter => "心态容易受比分影响",
            TraitType::MentalFortress => "心态稳定，不受连胜连败影响",
            TraitType::Fragile => "输了比赛心态下滑更快",
            TraitType::Ironman => "不受连续比赛疲劳影响",
            TraitType::Volatile => "状态波动比常人更大",
            TraitType::RisingStar => "新人赛季潜力爆发",
            TraitType::Veteran => "老将经验丰富，发挥更稳",
            TraitType::TeamLeader => "带动队友发挥",
        }
    }

    /// 获取特性稀有度 (1-5)
    pub fn rarity(&self) -> u8 {
        match self {
            TraitType::Clutch => 4,
            TraitType::SlowStarter => 2,
            TraitType::FastStarter => 2,
            TraitType::Explosive => 3,
            TraitType::Consistent => 2,
            TraitType::ComebackKing => 4,
            TraitType::Tilter => 1,  // 负面特性
            TraitType::MentalFortress => 4,
            TraitType::Fragile => 1,  // 负面特性
            TraitType::Ironman => 3,
            TraitType::Volatile => 2,
            TraitType::RisingStar => 3,
            TraitType::Veteran => 3,
            TraitType::TeamLeader => 5,
        }
    }

    /// 是否为负面特性
    pub fn is_negative(&self) -> bool {
        matches!(self, TraitType::Tilter | TraitType::Fragile | TraitType::Volatile)
    }
}

/// 特性修正结果
#[derive(Debug, Clone, Default)]
pub struct TraitModifiers {
    /// 能力值修正
    pub ability_mod: i8,
    /// 稳定性修正
    pub stability_mod: i8,
    /// 状态修正
    pub condition_mod: i8,
    /// momentum 系数 (默认 1.0)
    pub momentum_multiplier: f64,
    /// 能力上限修正
    pub ability_ceiling_mod: i8,
}

impl TraitModifiers {
    pub fn new() -> Self {
        Self {
            ability_mod: 0,
            stability_mod: 0,
            condition_mod: 0,
            momentum_multiplier: 1.0,
            ability_ceiling_mod: 0,
        }
    }

    /// 合并多个特性修正
    pub fn merge(&mut self, other: &TraitModifiers) {
        self.ability_mod += other.ability_mod;
        self.stability_mod += other.stability_mod;
        self.condition_mod += other.condition_mod;
        self.momentum_multiplier *= other.momentum_multiplier;
        self.ability_ceiling_mod += other.ability_ceiling_mod;
    }
}

/// 比赛情境（用于特性触发判断）
#[derive(Debug, Clone, Default)]
pub struct TraitContext {
    /// 赛事类型
    pub tournament_type: String,
    /// 是否季后赛
    pub is_playoff: bool,
    /// 是否国际赛
    pub is_international: bool,
    /// 当前第几局 (1-5)
    pub game_number: u8,
    /// 当前比分差 (正数表示领先)
    pub score_diff: i8,
    /// 选手年龄
    pub age: u8,
    /// 是否首个赛季
    pub is_first_season: bool,
    /// 连续比赛场次
    pub games_since_rest: u32,
}

impl TraitContext {
    /// 从 MatchContext 转换
    pub fn from_match_context(ctx: &MatchContext, age: u8, is_first_season: bool, games_since_rest: u32) -> Self {
        let is_international = matches!(
            ctx.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "clauch"
        );
        let is_playoff = ctx.round == "playoff" ||
                         ctx.round == "quarter" ||
                         ctx.round == "semi" ||
                         ctx.round == "final";

        Self {
            tournament_type: ctx.tournament_type.clone(),
            is_playoff,
            is_international,
            game_number: ctx.game_number,
            score_diff: ctx.score_diff,
            age,
            is_first_season,
            games_since_rest,
        }
    }
}

/// 特性引擎
pub struct TraitEngine;

impl TraitEngine {
    /// 计算单个特性的修正值
    pub fn calculate_trait_modifier(trait_type: TraitType, ctx: &TraitContext) -> TraitModifiers {
        let mut mods = TraitModifiers::new();

        match trait_type {
            TraitType::Clutch => {
                if ctx.is_playoff || ctx.is_international {
                    mods.condition_mod = 3;
                }
            }

            TraitType::SlowStarter => {
                match ctx.game_number {
                    1 => mods.condition_mod = -2,
                    3..=5 => mods.condition_mod = 2,
                    _ => {}
                }
            }

            TraitType::FastStarter => {
                match ctx.game_number {
                    1 => mods.condition_mod = 2,
                    3..=5 => mods.condition_mod = -1,
                    _ => {}
                }
            }

            TraitType::Explosive => {
                mods.stability_mod = -15;
                mods.ability_ceiling_mod = 5;
            }

            TraitType::Consistent => {
                mods.stability_mod = 10;
                mods.ability_ceiling_mod = -3;
            }

            TraitType::ComebackKing => {
                if ctx.score_diff < 0 {
                    mods.condition_mod = 3;
                }
            }

            TraitType::Tilter => {
                if ctx.score_diff > 0 {
                    mods.condition_mod = -2;
                } else if ctx.score_diff < 0 {
                    mods.condition_mod = -3;
                }
            }

            TraitType::MentalFortress => {
                mods.momentum_multiplier = 0.5;
            }

            TraitType::Fragile => {
                // 这个特性在 update_form_factors 时处理
                // 这里不做处理
            }

            TraitType::Ironman => {
                // 疲劳惩罚在其他地方处理，这里标记
                // 实际效果：games_since_rest 不影响 condition
            }

            TraitType::Volatile => {
                // 状态波动 ×1.5，通过修改 stability 实现
                mods.stability_mod = -10;
            }

            TraitType::RisingStar => {
                if ctx.is_first_season {
                    mods.ability_mod = 3;
                }
            }

            TraitType::Veteran => {
                if ctx.age >= 30 {
                    mods.stability_mod = 15;
                }
            }

            TraitType::TeamLeader => {
                // 队友加成需要特殊处理，这里不做
            }
        }

        mods
    }

    /// 计算多个特性的综合修正
    pub fn calculate_combined_modifiers(traits: &[TraitType], ctx: &TraitContext) -> TraitModifiers {
        let mut combined = TraitModifiers::new();

        for trait_type in traits {
            let trait_mod = Self::calculate_trait_modifier(*trait_type, ctx);
            combined.merge(&trait_mod);
        }

        // 限制修正值范围
        combined.ability_mod = combined.ability_mod.clamp(-10, 10);
        combined.stability_mod = combined.stability_mod.clamp(-20, 20);
        combined.condition_mod = combined.condition_mod.clamp(-5, 5);
        combined.ability_ceiling_mod = combined.ability_ceiling_mod.clamp(-5, 10);

        combined
    }

    /// 应用特性修正到选手属性
    pub fn apply_modifiers(
        base_ability: u8,
        base_stability: u8,
        base_condition: i8,
        modifiers: &TraitModifiers,
    ) -> (u8, u8, i8, u8) {
        // 应用能力修正
        let modified_ability = (base_ability as i16 + modifiers.ability_mod as i16)
            .clamp(1, 100) as u8;

        // 应用稳定性修正
        let modified_stability = (base_stability as i16 + modifiers.stability_mod as i16)
            .clamp(30, 100) as u8;

        // 应用状态修正
        let modified_condition = (base_condition as i16 + modifiers.condition_mod as i16)
            .clamp(-10, 10) as i8;

        // 计算能力上限
        let ability_ceiling = (modified_ability as i16 + 10 + modifiers.ability_ceiling_mod as i16)
            .clamp(modified_ability as i16, 100) as u8;

        (modified_ability, modified_stability, modified_condition, ability_ceiling)
    }

    /// 随机生成选手特性
    pub fn generate_random_traits(ability: u8, age: u8, rng: &mut impl rand::Rng) -> Vec<TraitType> {
        let mut traits = Vec::new();

        // 根据能力值决定特性数量
        let trait_count = match ability {
            90..=100 => 2 + rng.gen_range(0..2),  // 顶级选手 2-3 个特性
            80..=89 => 1 + rng.gen_range(0..2),   // 优秀选手 1-2 个特性
            70..=79 => rng.gen_range(0..2),       // 合格选手 0-1 个特性
            _ => if rng.gen::<f64>() < 0.3 { 1 } else { 0 },  // 低能力 30% 概率 1 个
        };

        if trait_count == 0 {
            return traits;
        }

        // 可用特性池（排除互斥特性）
        let mut available: Vec<TraitType> = vec![
            TraitType::Clutch,
            TraitType::SlowStarter,
            TraitType::FastStarter,
            TraitType::Explosive,
            TraitType::Consistent,
            TraitType::ComebackKing,
            TraitType::Tilter,
            TraitType::MentalFortress,
            TraitType::Fragile,
            TraitType::Ironman,
            TraitType::Volatile,
        ];

        // 年龄相关特性
        if age <= 20 {
            available.push(TraitType::RisingStar);
        }
        if age >= 28 {
            available.push(TraitType::Veteran);
        }
        if ability >= 85 {
            available.push(TraitType::TeamLeader);
        }

        // 按稀有度加权随机选择
        for _ in 0..trait_count {
            if available.is_empty() {
                break;
            }

            // 计算权重（稀有度越低越常见）
            let weights: Vec<f64> = available.iter()
                .map(|t| 1.0 / t.rarity() as f64)
                .collect();
            let total_weight: f64 = weights.iter().sum();

            let mut roll = rng.gen::<f64>() * total_weight;
            let mut selected_idx = 0;

            for (i, w) in weights.iter().enumerate() {
                roll -= w;
                if roll <= 0.0 {
                    selected_idx = i;
                    break;
                }
            }

            let selected = available.remove(selected_idx);
            traits.push(selected);

            // 移除互斥特性
            Self::remove_conflicting_traits(&mut available, selected);
        }

        traits
    }

    /// 移除与已选特性互斥的特性
    fn remove_conflicting_traits(available: &mut Vec<TraitType>, selected: TraitType) {
        let conflicts: Vec<TraitType> = match selected {
            TraitType::SlowStarter => vec![TraitType::FastStarter],
            TraitType::FastStarter => vec![TraitType::SlowStarter],
            TraitType::Explosive => vec![TraitType::Consistent],
            TraitType::Consistent => vec![TraitType::Explosive],
            TraitType::ComebackKing => vec![TraitType::Tilter],
            TraitType::Tilter => vec![TraitType::ComebackKing, TraitType::MentalFortress],
            TraitType::MentalFortress => vec![TraitType::Fragile, TraitType::Tilter],
            TraitType::Fragile => vec![TraitType::MentalFortress],
            _ => vec![],
        };

        available.retain(|t| !conflicts.contains(t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clutch_trait() {
        let ctx = TraitContext {
            is_playoff: true,
            is_international: false,
            ..Default::default()
        };

        let mods = TraitEngine::calculate_trait_modifier(TraitType::Clutch, &ctx);
        assert_eq!(mods.condition_mod, 3);
    }

    #[test]
    fn test_slow_starter_trait() {
        // 第1局
        let ctx1 = TraitContext { game_number: 1, ..Default::default() };
        let mods1 = TraitEngine::calculate_trait_modifier(TraitType::SlowStarter, &ctx1);
        assert_eq!(mods1.condition_mod, -2);

        // 第3局
        let ctx3 = TraitContext { game_number: 3, ..Default::default() };
        let mods3 = TraitEngine::calculate_trait_modifier(TraitType::SlowStarter, &ctx3);
        assert_eq!(mods3.condition_mod, 2);
    }

    #[test]
    fn test_comeback_king_trait() {
        // 落后时
        let ctx = TraitContext { score_diff: -1, ..Default::default() };
        let mods = TraitEngine::calculate_trait_modifier(TraitType::ComebackKing, &ctx);
        assert_eq!(mods.condition_mod, 3);

        // 领先时
        let ctx2 = TraitContext { score_diff: 1, ..Default::default() };
        let mods2 = TraitEngine::calculate_trait_modifier(TraitType::ComebackKing, &ctx2);
        assert_eq!(mods2.condition_mod, 0);
    }

    #[test]
    fn test_combined_modifiers() {
        let traits = vec![TraitType::Clutch, TraitType::Explosive];
        let ctx = TraitContext {
            is_playoff: true,
            ..Default::default()
        };

        let mods = TraitEngine::calculate_combined_modifiers(&traits, &ctx);
        assert_eq!(mods.condition_mod, 3);  // Clutch
        assert_eq!(mods.stability_mod, -15);  // Explosive
        assert_eq!(mods.ability_ceiling_mod, 5);  // Explosive
    }

    #[test]
    fn test_apply_modifiers() {
        let modifiers = TraitModifiers {
            ability_mod: 3,
            stability_mod: -15,
            condition_mod: 2,
            momentum_multiplier: 1.0,
            ability_ceiling_mod: 5,
        };

        let (ability, stability, condition, ceiling) =
            TraitEngine::apply_modifiers(80, 70, 0, &modifiers);

        assert_eq!(ability, 83);
        assert_eq!(stability, 55);
        assert_eq!(condition, 2);
        assert_eq!(ceiling, 98);  // 83 + 10 + 5 = 98
    }

    #[test]
    fn test_trait_rarity() {
        assert!(TraitType::TeamLeader.rarity() > TraitType::Tilter.rarity());
        assert!(TraitType::Clutch.rarity() > TraitType::SlowStarter.rarity());
    }
}
