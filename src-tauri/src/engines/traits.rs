//! 选手特性系统
//!
//! 特性影响选手在不同情境下的表现，通过修改 ability/stability/condition 实现
//! 完全解耦，不影响核心模拟引擎

use super::condition::MatchContext;
use serde::{Deserialize, Serialize};

/// 特性类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraitType {
    // === 大赛表现类 ===
    Clutch,
    SlowStarter,
    FastStarter,
    FinalsKiller,
    RegularKing,
    WinStreak,

    // === 心态类 ===
    ComebackKing,
    Tilter,
    MentalFortress,
    Fragile,
    Gambler,
    PressurePlayer,
    Complacent,

    // === 稳定性类 ===
    Explosive,
    Consistent,
    Streaky,
    BigGame,
    Choker,

    // === 体能类 ===
    Ironman,
    Volatile,
    Endurance,
    Sprinter,
    NightOwl,
    PeakForm,

    // === 队伍互动类 ===
    TeamLeader,
    LoneWolf,
    Supportive,
    Troublemaker,
    Mentor,

    // === 成长/衰退类 ===
    LateBlocker,
    Prodigy,
    Resilient,
    GlassCannon,
    LowCeiling,
    Limitless,
    BattleTested,
    PeakAge,
    EarlyDecline,

    // === 特殊类 ===
    RisingStar,
    Veteran,
    Perfectionist,
    Adaptable,

    // === 国际赛类 ===
    WorldStage,
    GroupStageExpert,
    KnockoutSpecialist,
    CrossRegion,
    TournamentHorse,
}

impl TraitType {
    /// 获取特性的中文名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TraitType::Clutch => "大赛型",
            TraitType::SlowStarter => "慢热型",
            TraitType::FastStarter => "快枪手",
            TraitType::FinalsKiller => "决赛杀手",
            TraitType::RegularKing => "常规赛之王",
            TraitType::WinStreak => "连胜狂魔",
            TraitType::Explosive => "爆发型",
            TraitType::Consistent => "稳定型",
            TraitType::Streaky => "时好时坏",
            TraitType::BigGame => "大场面选手",
            TraitType::Choker => "关键掉链子",
            TraitType::ComebackKing => "逆风王",
            TraitType::Tilter => "顺风浪",
            TraitType::MentalFortress => "心态大师",
            TraitType::Fragile => "玻璃心",
            TraitType::Gambler => "赌徒",
            TraitType::PressurePlayer => "抗压选手",
            TraitType::Complacent => "安于现状",
            TraitType::Ironman => "铁人",
            TraitType::Volatile => "状态敏感",
            TraitType::Endurance => "持久战型",
            TraitType::Sprinter => "短跑型",
            TraitType::NightOwl => "夜猫子",
            TraitType::PeakForm => "巅峰状态",
            TraitType::RisingStar => "新星",
            TraitType::Veteran => "老将风范",
            TraitType::TeamLeader => "团队核心",
            TraitType::LoneWolf => "独狼",
            TraitType::Supportive => "辅助型领袖",
            TraitType::Troublemaker => "刺头",
            TraitType::Mentor => "导师",
            TraitType::LateBlocker => "大器晚成",
            TraitType::Prodigy => "神童",
            TraitType::Resilient => "抗衰老",
            TraitType::GlassCannon => "易碎",
            TraitType::LowCeiling => "低天花板",
            TraitType::Limitless => "无限潜力",
            TraitType::BattleTested => "百战之躯",
            TraitType::PeakAge => "黄金年龄",
            TraitType::EarlyDecline => "早衰",
            TraitType::Perfectionist => "完美主义者",
            TraitType::Adaptable => "适应力强",
            TraitType::WorldStage => "世界舞台",
            TraitType::GroupStageExpert => "小组赛专家",
            TraitType::KnockoutSpecialist => "淘汰赛专家",
            TraitType::CrossRegion => "跨赛区适应",
            TraitType::TournamentHorse => "赛事铁马",
        }
    }

    /// 获取特性描述
    pub fn description(&self) -> &'static str {
        match self {
            TraitType::Clutch => "在季后赛和国际赛中状态更好",
            TraitType::SlowStarter => "系列赛开局较慢，但后期渐入佳境",
            TraitType::FastStarter => "系列赛开局强势，但后期可能疲软",
            TraitType::FinalsKiller => "决赛中能力爆发",
            TraitType::RegularKing => "常规赛表现稳定出色，季后赛略有下滑",
            TraitType::WinStreak => "连胜时越打越强",
            TraitType::Explosive => "发挥波动大，但巅峰更高",
            TraitType::Consistent => "发挥稳定，但上限略低",
            TraitType::Streaky => "状态大起大落，好坏交替",
            TraitType::BigGame => "重要比赛发挥出色",
            TraitType::Choker => "关键比赛掉链子",
            TraitType::ComebackKing => "落后时愈战愈勇",
            TraitType::Tilter => "心态容易受比分影响",
            TraitType::MentalFortress => "心态稳定，不受连胜连败影响",
            TraitType::Fragile => "输了比赛心态下滑更快",
            TraitType::Gambler => "发挥极端，要么超神要么超鬼",
            TraitType::PressurePlayer => "被逼到绝境时爆发",
            TraitType::Complacent => "领先时容易松懈放水",
            TraitType::Ironman => "不受连续比赛疲劳影响",
            TraitType::Volatile => "状态波动比常人更大",
            TraitType::Endurance => "长系列赛体力充沛，不会疲劳下滑",
            TraitType::Sprinter => "短赛制爆发力强，BO5后半段下降",
            TraitType::NightOwl => "赛季后半程状态更好",
            TraitType::PeakForm => "巅峰期状态波动极小",
            TraitType::RisingStar => "新人赛季潜力爆发",
            TraitType::Veteran => "老将经验丰富，发挥更稳",
            TraitType::TeamLeader => "带动队友发挥",
            TraitType::LoneWolf => "单打独斗能力强，但不擅长配合",
            TraitType::Supportive => "让队友变得更好",
            TraitType::Troublemaker => "实力出众但影响队伍氛围",
            TraitType::Mentor => "帮助年轻队友成长更快",
            TraitType::LateBlocker => "大器晚成，成长期和巅峰期延长2年",
            TraitType::Prodigy => "年少成名，但后期成长放缓",
            TraitType::Resilient => "身体素质出众，衰退速度减半",
            TraitType::GlassCannon => "巅峰更高但衰退更快",
            TraitType::LowCeiling => "潜力有限，能力难以突破",
            TraitType::Limitless => "无限潜力，成长不设上限",
            TraitType::BattleTested => "比赛打得越多，表现越稳定",
            TraitType::PeakAge => "在黄金年龄段爆发力更强",
            TraitType::EarlyDecline => "比常人更早开始衰退",
            TraitType::Perfectionist => "队伍磨合好时表现加成，磨合差时惩罚",
            TraitType::Adaptable => "换队后适应速度快，首赛季无惩罚",
            TraitType::WorldStage => "世界赛超神发挥",
            TraitType::GroupStageExpert => "小组赛阶段表现稳定",
            TraitType::KnockoutSpecialist => "淘汰赛阶段能力爆发",
            TraitType::CrossRegion => "跨赛区打比赛不受影响",
            TraitType::TournamentHorse => "国际赛全程状态不衰减",
        }
    }

    /// 获取特性稀有度 (1-5)
    pub fn rarity(&self) -> u8 {
        match self {
            TraitType::Clutch => 4,
            TraitType::SlowStarter => 2,
            TraitType::FastStarter => 2,
            TraitType::FinalsKiller => 5,
            TraitType::RegularKing => 3,
            TraitType::WinStreak => 3,
            TraitType::Explosive => 3,
            TraitType::Consistent => 2,
            TraitType::Streaky => 1,
            TraitType::BigGame => 4,
            TraitType::Choker => 1,
            TraitType::ComebackKing => 4,
            TraitType::Tilter => 1,
            TraitType::MentalFortress => 4,
            TraitType::Fragile => 1,
            TraitType::Gambler => 2,
            TraitType::PressurePlayer => 4,
            TraitType::Complacent => 1,
            TraitType::Ironman => 3,
            TraitType::Volatile => 2,
            TraitType::Endurance => 3,
            TraitType::Sprinter => 2,
            TraitType::NightOwl => 2,
            TraitType::PeakForm => 5,
            TraitType::RisingStar => 3,
            TraitType::Veteran => 3,
            TraitType::TeamLeader => 5,
            TraitType::LoneWolf => 2,
            TraitType::Supportive => 3,
            TraitType::Troublemaker => 1,
            TraitType::Mentor => 4,
            TraitType::LateBlocker => 3,
            TraitType::Prodigy => 4,
            TraitType::Resilient => 4,
            TraitType::GlassCannon => 2,
            TraitType::LowCeiling => 1,
            TraitType::Limitless => 5,
            TraitType::BattleTested => 3,
            TraitType::PeakAge => 3,
            TraitType::EarlyDecline => 1,
            TraitType::Perfectionist => 3,
            TraitType::Adaptable => 3,
            TraitType::WorldStage => 5,
            TraitType::GroupStageExpert => 2,
            TraitType::KnockoutSpecialist => 4,
            TraitType::CrossRegion => 3,
            TraitType::TournamentHorse => 4,
        }
    }

    /// 是否为负面特性
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            TraitType::Tilter
                | TraitType::Fragile
                | TraitType::Volatile
                | TraitType::GlassCannon
                | TraitType::Choker
                | TraitType::Complacent
                | TraitType::Streaky
                | TraitType::Troublemaker
                | TraitType::LowCeiling
                | TraitType::EarlyDecline
        )
    }

    /// 从字符串解析特性类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "clutch" => Some(Self::Clutch),
            "slow_starter" | "slowstarter" => Some(Self::SlowStarter),
            "fast_starter" | "faststarter" => Some(Self::FastStarter),
            "finals_killer" | "finalskiller" => Some(Self::FinalsKiller),
            "regular_king" | "regularking" => Some(Self::RegularKing),
            "win_streak" | "winstreak" => Some(Self::WinStreak),
            "explosive" => Some(Self::Explosive),
            "consistent" => Some(Self::Consistent),
            "streaky" => Some(Self::Streaky),
            "big_game" | "biggame" => Some(Self::BigGame),
            "choker" => Some(Self::Choker),
            "comeback_king" | "comebackking" => Some(Self::ComebackKing),
            "tilter" => Some(Self::Tilter),
            "mental_fortress" | "mentalfortress" => Some(Self::MentalFortress),
            "fragile" => Some(Self::Fragile),
            "gambler" => Some(Self::Gambler),
            "pressure_player" | "pressureplayer" => Some(Self::PressurePlayer),
            "complacent" => Some(Self::Complacent),
            "ironman" => Some(Self::Ironman),
            "volatile" => Some(Self::Volatile),
            "endurance" => Some(Self::Endurance),
            "sprinter" => Some(Self::Sprinter),
            "night_owl" | "nightowl" => Some(Self::NightOwl),
            "peak_form" | "peakform" => Some(Self::PeakForm),
            "rising_star" | "risingstar" => Some(Self::RisingStar),
            "veteran" => Some(Self::Veteran),
            "team_leader" | "teamleader" => Some(Self::TeamLeader),
            "lone_wolf" | "lonewolf" => Some(Self::LoneWolf),
            "supportive" => Some(Self::Supportive),
            "troublemaker" => Some(Self::Troublemaker),
            "mentor" => Some(Self::Mentor),
            "late_blocker" | "lateblocker" => Some(Self::LateBlocker),
            "prodigy" => Some(Self::Prodigy),
            "resilient" => Some(Self::Resilient),
            "glass_cannon" | "glasscannon" => Some(Self::GlassCannon),
            "low_ceiling" | "lowceiling" => Some(Self::LowCeiling),
            "limitless" => Some(Self::Limitless),
            "battle_tested" | "battletested" => Some(Self::BattleTested),
            "peak_age" | "peakage" => Some(Self::PeakAge),
            "early_decline" | "earlydecline" => Some(Self::EarlyDecline),
            "perfectionist" => Some(Self::Perfectionist),
            "adaptable" => Some(Self::Adaptable),
            "world_stage" | "worldstage" => Some(Self::WorldStage),
            "group_stage_expert" | "groupstageexpert" => Some(Self::GroupStageExpert),
            "knockout_specialist" | "knockoutspecialist" => Some(Self::KnockoutSpecialist),
            "cross_region" | "crossregion" => Some(Self::CrossRegion),
            "tournament_horse" | "tournamenthorse" => Some(Self::TournamentHorse),
            _ => None,
        }
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
    pub fn from_match_context(
        ctx: &MatchContext,
        age: u8,
        is_first_season: bool,
        games_since_rest: u32,
    ) -> Self {
        let is_international = matches!(
            ctx.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "clauch"
        );
        let is_playoff = ctx.round == "playoff"
            || ctx.round == "quarter"
            || ctx.round == "semi"
            || ctx.round == "final";

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

            TraitType::SlowStarter => match ctx.game_number {
                1 => mods.condition_mod = -2,
                3..=5 => mods.condition_mod = 2,
                _ => {}
            },

            TraitType::FastStarter => match ctx.game_number {
                1 => mods.condition_mod = 2,
                3..=5 => mods.condition_mod = -1,
                _ => {}
            },

            TraitType::FinalsKiller => {
                if ctx.tournament_type.contains("final") || ctx.game_number >= 4 {
                    mods.ability_mod = 3;
                    mods.condition_mod = 2;
                }
            }

            TraitType::RegularKing => {
                if !ctx.is_playoff && !ctx.is_international {
                    mods.condition_mod = 2;
                    mods.stability_mod = 5;
                } else if ctx.is_playoff {
                    mods.condition_mod = -1;
                }
            }

            TraitType::WinStreak => {
                // momentum > 0 意味着连胜中，通过 condition 加成体现
                if ctx.score_diff > 0 {
                    mods.condition_mod = 2;
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

            TraitType::Streaky => {
                mods.stability_mod = -20;
            }

            TraitType::BigGame => {
                if ctx.is_playoff || ctx.is_international {
                    mods.condition_mod = 2;
                    mods.stability_mod = 5;
                }
            }

            TraitType::Choker => {
                if ctx.is_playoff || ctx.is_international {
                    mods.condition_mod = -3;
                    mods.stability_mod = -10;
                }
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
                // momentum 惩罚在 update_form_factors 中处理
            }

            TraitType::Gambler => {
                mods.stability_mod = -25;
                mods.ability_ceiling_mod = 8;
            }

            TraitType::PressurePlayer => {
                if ctx.score_diff < 0 && (ctx.is_playoff || ctx.is_international) {
                    mods.ability_mod = 2;
                    mods.condition_mod = 3;
                }
            }

            TraitType::Complacent => {
                if ctx.score_diff > 0 {
                    mods.condition_mod = -2;
                    mods.stability_mod = -5;
                }
            }

            TraitType::Ironman => {
                // 疲劳惩罚豁免，在其他地方处理
            }

            TraitType::Volatile => {
                mods.stability_mod = -10;
            }

            TraitType::Endurance => {
                if ctx.game_number >= 4 {
                    mods.condition_mod = 2;
                }
            }

            TraitType::Sprinter => match ctx.game_number {
                1..=2 => mods.condition_mod = 2,
                4..=5 => mods.condition_mod = -2,
                _ => {}
            },

            TraitType::NightOwl => {
                // 赛季后半程加成，在赛季结算中用 tournament_type 判断
                // 夏季赛及之后的赛事
                let late_season = matches!(
                    ctx.tournament_type.as_str(),
                    "summer_regular"
                        | "summer_playoff"
                        | "worlds"
                        | "shanghai"
                        | "clauch"
                        | "super"
                        | "icp"
                );
                if late_season {
                    mods.condition_mod = 2;
                }
            }

            TraitType::PeakForm => {
                if ctx.age >= 25 && ctx.age <= 29 {
                    mods.stability_mod = 15;
                }
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
                // 队友加成在 team 层面处理
            }

            TraitType::LoneWolf => {
                mods.ability_mod = 2;
                mods.condition_mod = -1;
            }

            TraitType::Supportive => {
                // 队友 condition +1，在 team 层面处理，自身无加成
            }

            TraitType::Troublemaker => {
                mods.ability_mod = 1;
                mods.condition_mod = -2;
            }

            TraitType::Mentor => {
                // 年轻队友成长加速，在赛季结算中处理
            }

            TraitType::Perfectionist => {
                // synergy 高时加成，synergy 低时惩罚 — 在 game_flow 化学反应层处理
            }

            TraitType::Adaptable => {
                if ctx.is_first_season {
                    mods.condition_mod = 2;
                }
            }

            TraitType::WorldStage => {
                if ctx.tournament_type == "worlds" {
                    mods.ability_mod = 3;
                    mods.condition_mod = 3;
                }
            }

            TraitType::GroupStageExpert => {
                if !ctx.is_playoff && ctx.is_international {
                    mods.condition_mod = 2;
                    mods.stability_mod = 5;
                }
            }

            TraitType::KnockoutSpecialist => {
                if ctx.is_playoff && ctx.is_international {
                    mods.condition_mod = 3;
                    mods.ability_mod = 2;
                }
            }

            TraitType::CrossRegion => {
                if ctx.is_international {
                    mods.condition_mod = 1;
                }
            }

            TraitType::TournamentHorse => {
                if ctx.is_international && ctx.games_since_rest > 5 {
                    mods.condition_mod = 2;
                }
            }

            // 成长类特性：在赛季结算时处理，比赛中不生效
            TraitType::LateBlocker
            | TraitType::Prodigy
            | TraitType::Resilient
            | TraitType::LowCeiling
            | TraitType::Limitless
            | TraitType::BattleTested
            | TraitType::PeakAge
            | TraitType::EarlyDecline => {}

            TraitType::GlassCannon => {
                mods.ability_ceiling_mod = 3;
            }
        }

        mods
    }
    /// 计算多个特性的综合修正
    pub fn calculate_combined_modifiers(
        traits: &[TraitType],
        ctx: &TraitContext,
    ) -> TraitModifiers {
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
        let modified_ability =
            (base_ability as i16 + modifiers.ability_mod as i16).clamp(1, 100) as u8;

        // 应用稳定性修正
        let modified_stability =
            (base_stability as i16 + modifiers.stability_mod as i16).clamp(30, 100) as u8;

        // 应用状态修正
        let modified_condition =
            (base_condition as i16 + modifiers.condition_mod as i16).clamp(-10, 10) as i8;

        // 计算能力上限
        let ability_ceiling = (modified_ability as i16 + 10 + modifiers.ability_ceiling_mod as i16)
            .clamp(modified_ability as i16, 100) as u8;

        (
            modified_ability,
            modified_stability,
            modified_condition,
            ability_ceiling,
        )
    }

    /// 随机生成选手特性
    pub fn generate_random_traits(
        ability: u8,
        age: u8,
        rng: &mut impl rand::Rng,
    ) -> Vec<TraitType> {
        let mut traits = Vec::new();

        // 根据能力值决定特性数量
        let trait_count = match ability {
            68..=100 => 2 + rng.gen_range(0..2), // 顶级选手 2-3 个特性
            61..=67 => 1 + rng.gen_range(0..2),  // 优秀选手 1-2 个特性
            54..=60 => rng.gen_range(0..2),      // 合格选手 0-1 个特性
            _ => {
                if rng.gen::<f64>() < 0.3 {
                    1
                } else {
                    0
                }
            } // 低能力 30% 概率 1 个
        };

        if trait_count == 0 {
            return traits;
        }

        // 可用特性池（排除互斥特性）
        let mut available: Vec<TraitType> = vec![
            TraitType::Clutch,
            TraitType::SlowStarter,
            TraitType::FastStarter,
            TraitType::WinStreak,
            TraitType::Explosive,
            TraitType::Consistent,
            TraitType::Streaky,
            TraitType::ComebackKing,
            TraitType::Tilter,
            TraitType::MentalFortress,
            TraitType::Fragile,
            TraitType::Gambler,
            TraitType::PressurePlayer,
            TraitType::Complacent,
            TraitType::Ironman,
            TraitType::Volatile,
            TraitType::Endurance,
            TraitType::Sprinter,
            TraitType::NightOwl,
            TraitType::LoneWolf,
            TraitType::Supportive,
            TraitType::Troublemaker,
            TraitType::Adaptable,
            TraitType::CrossRegion,
        ];

        // 年龄相关特性
        if age <= 20 {
            available.push(TraitType::RisingStar);
            available.push(TraitType::Prodigy);
        }
        if age >= 28 {
            available.push(TraitType::Veteran);
        }
        if ability >= 65 {
            available.push(TraitType::TeamLeader);
            available.push(TraitType::Mentor);
            available.push(TraitType::BigGame);
            available.push(TraitType::FinalsKiller);
        }
        if ability >= 70 {
            available.push(TraitType::WorldStage);
            available.push(TraitType::KnockoutSpecialist);
            available.push(TraitType::PeakForm);
            available.push(TraitType::Limitless);
        }
        // 成长类特性（所有年龄可获得）
        available.push(TraitType::LateBlocker);
        available.push(TraitType::Resilient);
        available.push(TraitType::GlassCannon);
        available.push(TraitType::LowCeiling);
        available.push(TraitType::BattleTested);
        available.push(TraitType::PeakAge);
        available.push(TraitType::EarlyDecline);
        available.push(TraitType::GroupStageExpert);
        available.push(TraitType::TournamentHorse);
        available.push(TraitType::RegularKing);
        available.push(TraitType::Perfectionist);
        available.push(TraitType::Choker);

        // 按稀有度加权随机选择
        for _ in 0..trait_count {
            if available.is_empty() {
                break;
            }

            // 计算权重（稀有度越低越常见）
            let weights: Vec<f64> = available.iter().map(|t| 1.0 / t.rarity() as f64).collect();
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
            TraitType::FinalsKiller => vec![TraitType::Choker],
            TraitType::RegularKing => vec![TraitType::Clutch, TraitType::BigGame],
            TraitType::Explosive => vec![TraitType::Consistent, TraitType::PeakForm],
            TraitType::Consistent => {
                vec![TraitType::Explosive, TraitType::Streaky, TraitType::Gambler]
            }
            TraitType::Streaky => vec![TraitType::Consistent, TraitType::PeakForm],
            TraitType::BigGame => vec![TraitType::Choker, TraitType::RegularKing],
            TraitType::Choker => vec![
                TraitType::BigGame,
                TraitType::Clutch,
                TraitType::FinalsKiller,
                TraitType::PressurePlayer,
            ],
            TraitType::ComebackKing => vec![TraitType::Tilter, TraitType::Complacent],
            TraitType::Tilter => vec![
                TraitType::ComebackKing,
                TraitType::MentalFortress,
                TraitType::PressurePlayer,
            ],
            TraitType::MentalFortress => vec![TraitType::Fragile, TraitType::Tilter],
            TraitType::Fragile => vec![TraitType::MentalFortress],
            TraitType::Gambler => vec![TraitType::Consistent, TraitType::PeakForm],
            TraitType::PressurePlayer => {
                vec![TraitType::Tilter, TraitType::Choker, TraitType::Complacent]
            }
            TraitType::Complacent => vec![TraitType::ComebackKing, TraitType::PressurePlayer],
            TraitType::Ironman => vec![TraitType::Sprinter],
            TraitType::Endurance => vec![TraitType::Sprinter],
            TraitType::Sprinter => vec![TraitType::Ironman, TraitType::Endurance],
            TraitType::PeakForm => {
                vec![TraitType::Explosive, TraitType::Streaky, TraitType::Gambler]
            }
            TraitType::TeamLeader => vec![TraitType::LoneWolf, TraitType::Troublemaker],
            TraitType::LoneWolf => vec![TraitType::TeamLeader, TraitType::Supportive],
            TraitType::Supportive => vec![TraitType::LoneWolf, TraitType::Troublemaker],
            TraitType::Troublemaker => vec![
                TraitType::TeamLeader,
                TraitType::Supportive,
                TraitType::Mentor,
            ],
            TraitType::Mentor => vec![TraitType::Troublemaker],
            TraitType::LateBlocker => vec![TraitType::Prodigy, TraitType::EarlyDecline],
            TraitType::Prodigy => vec![TraitType::LateBlocker],
            TraitType::Resilient => vec![TraitType::GlassCannon, TraitType::EarlyDecline],
            TraitType::GlassCannon => vec![TraitType::Resilient],
            TraitType::LowCeiling => vec![TraitType::Limitless],
            TraitType::Limitless => vec![TraitType::LowCeiling],
            TraitType::EarlyDecline => vec![TraitType::LateBlocker, TraitType::Resilient],
            TraitType::WorldStage => vec![TraitType::GroupStageExpert],
            TraitType::GroupStageExpert => {
                vec![TraitType::WorldStage, TraitType::KnockoutSpecialist]
            }
            TraitType::KnockoutSpecialist => vec![TraitType::GroupStageExpert],
            _ => vec![],
        };

        available.retain(|t| !conflicts.contains(t));
    }

    /// 赛季结算时评估特性觉醒与退化
    /// 返回 (新获得的特性, 失去的特性)
    pub fn evaluate_trait_awakening(
        ability: u8,
        age: u8,
        games_played: i32,
        avg_performance: f64,
        existing_traits: &[TraitType],
        seasons_in_team: i64,
        rng: &mut impl rand::Rng,
    ) -> (Vec<TraitType>, Vec<TraitType>) {
        let mut gained = Vec::new();
        let mut lost = Vec::new();

        // 全局觉醒门槛：只有 25% 的选手有机会觉醒新特性
        // 高能力(80+)选手略微提高到 35%，低能力(<55)选手降低到 15%
        let base_awakening_rate = if ability >= 80 {
            0.35
        } else if ability >= 70 {
            0.25
        } else if ability >= 55 {
            0.20
        } else {
            0.15
        };

        if rng.gen::<f64>() >= base_awakening_rate {
            // 未通过全局门槛，只检查退化，跳过觉醒
            for trait_type in existing_traits {
                let decay_prob = Self::get_decay_probability(
                    *trait_type,
                    ability,
                    age,
                    games_played,
                    avg_performance,
                );
                if decay_prob > 0.0 && rng.gen::<f64>() < decay_prob {
                    lost.push(*trait_type);
                    break;
                }
            }
            return (gained, lost);
        }

        // 每赛季最多觉醒1个，退化1个
        let mut awakened_one = false;

        // === 觉醒检查 ===
        let candidates: Vec<(TraitType, f64)> = Self::get_awakening_candidates(
            ability,
            age,
            games_played,
            avg_performance,
            existing_traits,
            seasons_in_team,
        );

        for (trait_type, prob) in &candidates {
            if awakened_one {
                break;
            }
            if existing_traits.contains(trait_type) {
                continue;
            }
            // 检查互斥
            let mut temp = existing_traits.to_vec();
            temp.extend(gained.iter());
            let has_conflict = Self::get_conflicts(*trait_type)
                .iter()
                .any(|c| temp.contains(c));
            if has_conflict {
                continue;
            }

            if rng.gen::<f64>() < *prob {
                gained.push(*trait_type);
                awakened_one = true;
            }
        }

        // === 退化检查 ===
        for trait_type in existing_traits {
            let decay_prob = Self::get_decay_probability(
                *trait_type,
                ability,
                age,
                games_played,
                avg_performance,
            );
            if decay_prob > 0.0 && rng.gen::<f64>() < decay_prob {
                lost.push(*trait_type);
                break; // 每赛季最多退化1个
            }
        }

        (gained, lost)
    }

    fn get_conflicts(trait_type: TraitType) -> Vec<TraitType> {
        match trait_type {
            TraitType::SlowStarter => vec![TraitType::FastStarter],
            TraitType::FastStarter => vec![TraitType::SlowStarter],
            TraitType::FinalsKiller => vec![TraitType::Choker],
            TraitType::RegularKing => vec![TraitType::Clutch, TraitType::BigGame],
            TraitType::Explosive => vec![TraitType::Consistent, TraitType::PeakForm],
            TraitType::Consistent => {
                vec![TraitType::Explosive, TraitType::Streaky, TraitType::Gambler]
            }
            TraitType::Streaky => vec![TraitType::Consistent, TraitType::PeakForm],
            TraitType::BigGame => vec![TraitType::Choker, TraitType::RegularKing],
            TraitType::Choker => vec![
                TraitType::BigGame,
                TraitType::Clutch,
                TraitType::FinalsKiller,
                TraitType::PressurePlayer,
            ],
            TraitType::ComebackKing => vec![TraitType::Tilter, TraitType::Complacent],
            TraitType::Tilter => vec![
                TraitType::ComebackKing,
                TraitType::MentalFortress,
                TraitType::PressurePlayer,
            ],
            TraitType::MentalFortress => vec![TraitType::Fragile, TraitType::Tilter],
            TraitType::Fragile => vec![TraitType::MentalFortress],
            TraitType::Gambler => vec![TraitType::Consistent, TraitType::PeakForm],
            TraitType::PressurePlayer => {
                vec![TraitType::Tilter, TraitType::Choker, TraitType::Complacent]
            }
            TraitType::Complacent => vec![TraitType::ComebackKing, TraitType::PressurePlayer],
            TraitType::Ironman => vec![TraitType::Sprinter],
            TraitType::Endurance => vec![TraitType::Sprinter],
            TraitType::Sprinter => vec![TraitType::Ironman, TraitType::Endurance],
            TraitType::PeakForm => {
                vec![TraitType::Explosive, TraitType::Streaky, TraitType::Gambler]
            }
            TraitType::TeamLeader => vec![TraitType::LoneWolf, TraitType::Troublemaker],
            TraitType::LoneWolf => vec![TraitType::TeamLeader, TraitType::Supportive],
            TraitType::Supportive => vec![TraitType::LoneWolf, TraitType::Troublemaker],
            TraitType::Troublemaker => vec![
                TraitType::TeamLeader,
                TraitType::Supportive,
                TraitType::Mentor,
            ],
            TraitType::Mentor => vec![TraitType::Troublemaker],
            TraitType::LateBlocker => vec![TraitType::Prodigy, TraitType::EarlyDecline],
            TraitType::Prodigy => vec![TraitType::LateBlocker],
            TraitType::Resilient => vec![TraitType::GlassCannon, TraitType::EarlyDecline],
            TraitType::GlassCannon => vec![TraitType::Resilient],
            TraitType::LowCeiling => vec![TraitType::Limitless],
            TraitType::Limitless => vec![TraitType::LowCeiling],
            TraitType::EarlyDecline => vec![TraitType::LateBlocker, TraitType::Resilient],
            TraitType::WorldStage => vec![TraitType::GroupStageExpert],
            TraitType::GroupStageExpert => {
                vec![TraitType::WorldStage, TraitType::KnockoutSpecialist]
            }
            TraitType::KnockoutSpecialist => vec![TraitType::GroupStageExpert],
            _ => vec![],
        }
    }

    /// 根据选手状态返回可觉醒特性及其概率
    fn get_awakening_candidates(
        ability: u8,
        age: u8,
        games_played: i32,
        avg_performance: f64,
        existing: &[TraitType],
        seasons_in_team: i64,
    ) -> Vec<(TraitType, f64)> {
        let mut candidates = Vec::new();

        // Clutch / BigGame: 高能力 + 大量高质量比赛
        if ability >= 75 && games_played >= 40 && avg_performance > 0.8 {
            candidates.push((TraitType::Clutch, 0.06));
            candidates.push((TraitType::BigGame, 0.08));
        }

        // FinalsKiller: 顶尖表现
        if ability >= 80 && avg_performance > 1.5 {
            candidates.push((TraitType::FinalsKiller, 0.04));
        }

        // RegularKing: 打满常规赛 + 稳定中等表现
        if games_played >= 40 && avg_performance > 0.2 && avg_performance < 0.8 && ability >= 65 {
            candidates.push((TraitType::RegularKing, 0.08));
        }

        // WinStreak: 持续高表现
        if avg_performance > 1.0 && games_played >= 35 && ability >= 70 {
            candidates.push((TraitType::WinStreak, 0.08));
        }

        // Explosive: 高能力 + 高表现
        if ability >= 72 && avg_performance > 0.8 {
            candidates.push((TraitType::Explosive, 0.05));
        }

        // Consistent: 大量比赛 + 表现稳定在中间段
        if games_played >= 40 && avg_performance > 0.0 && avg_performance < 0.5 && ability >= 65 {
            candidates.push((TraitType::Consistent, 0.08));
        }

        // Gambler: 高能力 + 极端表现波动
        if ability >= 70 && avg_performance > 0.5 {
            candidates.push((TraitType::Gambler, 0.03));
        }

        // ComebackKing / PressurePlayer: 高能力 + 显著高表现
        if avg_performance > 0.8 && ability >= 72 && games_played >= 30 {
            candidates.push((TraitType::ComebackKing, 0.06));
            candidates.push((TraitType::PressurePlayer, 0.05));
        }

        // 负面特性: 表现极差
        if avg_performance < -0.8 && games_played >= 25 {
            candidates.push((TraitType::Tilter, 0.10));
            candidates.push((TraitType::Fragile, 0.06));
            candidates.push((TraitType::Choker, 0.06));
        }

        // Ironman / Endurance / TournamentHorse: 超大量比赛 + 表现不差
        if games_played >= 50 && avg_performance > 0.0 {
            candidates.push((TraitType::Ironman, 0.06));
            candidates.push((TraitType::Endurance, 0.08));
        }
        if games_played >= 55 && avg_performance > 0.3 {
            candidates.push((TraitType::TournamentHorse, 0.05));
        }

        // MentalFortress: 高能力 + 稳定正面表现 + 大量比赛
        if ability >= 75 && avg_performance > 0.5 && games_played >= 35 {
            candidates.push((TraitType::MentalFortress, 0.05));
        }

        // 老将觉醒: 需要高能力支撑
        if age >= 29 && ability >= 68 {
            candidates.push((TraitType::Veteran, 0.10));
            candidates.push((TraitType::BattleTested, 0.08));
        }
        if age >= 30 && ability >= 70 && seasons_in_team >= 2 {
            candidates.push((TraitType::Mentor, 0.06));
        }

        // 年轻天才: 更严格
        if age <= 19 && ability >= 70 {
            candidates.push((TraitType::Prodigy, 0.06));
            candidates.push((TraitType::RisingStar, 0.08));
        }

        // TeamLeader / Supportive: 长期效力 + 高能力
        if seasons_in_team >= 4 && ability >= 70 {
            candidates.push((TraitType::TeamLeader, 0.05));
        }
        if seasons_in_team >= 3 && ability >= 65 && avg_performance > 0.0 {
            candidates.push((TraitType::Supportive, 0.06));
        }

        // PeakAge / PeakForm: 黄金年龄 + 高能力
        if age >= 24 && age <= 27 && ability >= 75 && avg_performance > 0.5 {
            candidates.push((TraitType::PeakAge, 0.08));
            candidates.push((TraitType::PeakForm, 0.04));
        }

        // Limitless: 非常年轻 + 高能力 + 优秀表现
        if age <= 21 && ability >= 72 && avg_performance > 0.8 {
            candidates.push((TraitType::Limitless, 0.04));
        }

        // LowCeiling / EarlyDecline: 负面成长
        if age >= 25 && ability < 55 && avg_performance < -0.2 {
            candidates.push((TraitType::LowCeiling, 0.08));
        }
        if age >= 27 && avg_performance < -0.5 {
            candidates.push((TraitType::EarlyDecline, 0.06));
        }

        // Adaptable: 新转会 + 高能力 + 优秀表现
        if seasons_in_team <= 1 && avg_performance > 0.8 && ability >= 68 {
            candidates.push((TraitType::Adaptable, 0.08));
        }

        // LoneWolf: 高能力 + 高表现 + 短期效力
        if ability >= 75 && avg_performance > 0.8 && seasons_in_team <= 1 {
            candidates.push((TraitType::LoneWolf, 0.05));
        }
        // Troublemaker: 表现很差 + 有能力
        if avg_performance < -0.5 && ability >= 68 {
            candidates.push((TraitType::Troublemaker, 0.05));
        }

        // Resilient: 老将 + 高能力
        if age >= 30 && ability >= 70 {
            candidates.push((TraitType::Resilient, 0.06));
        }

        // LateBlocker: 大器晚成
        if age >= 26 && ability >= 72 && avg_performance > 0.8 {
            candidates.push((TraitType::LateBlocker, 0.05));
        }

        // 国际赛类: 大量比赛 + 高表现
        if games_played >= 35 && avg_performance > 0.8 && ability >= 68 {
            candidates.push((TraitType::GroupStageExpert, 0.06));
        }
        if games_played >= 30 && avg_performance > 0.5 && seasons_in_team <= 2 {
            candidates.push((TraitType::CrossRegion, 0.05));
        }
        if ability >= 80 && avg_performance > 1.5 {
            candidates.push((TraitType::WorldStage, 0.03));
            candidates.push((TraitType::KnockoutSpecialist, 0.04));
        }

        // Perfectionist: 长期效力 + 高能力 + 稳定高表现
        if seasons_in_team >= 4 && avg_performance > 0.5 && ability >= 70 {
            candidates.push((TraitType::Perfectionist, 0.06));
        }

        candidates.retain(|(t, _)| !existing.contains(t));

        candidates
    }

    /// 特性退化概率
    fn get_decay_probability(
        trait_type: TraitType,
        ability: u8,
        age: u8,
        games_played: i32,
        avg_performance: f64,
    ) -> f64 {
        match trait_type {
            // 正面特性在表现差时有退化概率
            TraitType::Clutch | TraitType::BigGame | TraitType::FinalsKiller => {
                if avg_performance < -0.5 {
                    0.10
                } else {
                    0.0
                }
            }
            TraitType::MentalFortress => {
                if avg_performance < -0.8 {
                    0.08
                } else {
                    0.0
                }
            }
            TraitType::Consistent => {
                if avg_performance < -0.3 || avg_performance > 1.0 {
                    0.06
                } else {
                    0.0
                }
            }
            TraitType::TeamLeader | TraitType::Supportive => {
                if avg_performance < -0.5 {
                    0.08
                } else {
                    0.0
                }
            }
            TraitType::RisingStar => {
                // 过了新秀期自然退化
                if age >= 22 {
                    0.50
                } else {
                    0.0
                }
            }
            TraitType::PeakForm => {
                if age >= 30 || age < 24 {
                    0.15
                } else {
                    0.0
                }
            }
            TraitType::PeakAge => {
                if age >= 30 {
                    0.30
                } else {
                    0.0
                }
            }
            TraitType::Prodigy => {
                if age >= 25 {
                    0.20
                } else {
                    0.0
                }
            }
            // 负面特性在高表现时有退化概率（好事）
            TraitType::Tilter | TraitType::Fragile => {
                if avg_performance > 0.5 && games_played >= 25 {
                    0.12
                } else {
                    0.0
                }
            }
            TraitType::Choker => {
                if avg_performance > 0.8 {
                    0.10
                } else {
                    0.0
                }
            }
            TraitType::Complacent => {
                if avg_performance > 0.3 {
                    0.08
                } else {
                    0.0
                }
            }
            TraitType::Troublemaker => {
                if avg_performance > 0.5 {
                    0.08
                } else {
                    0.0
                }
            }
            TraitType::LowCeiling => {
                if ability >= 68 {
                    0.10
                } else {
                    0.0
                }
            }
            TraitType::EarlyDecline => {
                if ability >= 70 && age <= 27 {
                    0.08
                } else {
                    0.0
                }
            }
            TraitType::GlassCannon => {
                if age <= 26 && avg_performance > 0.3 {
                    0.06
                } else {
                    0.0
                }
            }
            TraitType::Streaky => {
                if avg_performance > 0.3 && games_played >= 30 {
                    0.08
                } else {
                    0.0
                }
            }
            // 其他特性不退化
            _ => 0.0,
        }
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
        let ctx1 = TraitContext {
            game_number: 1,
            ..Default::default()
        };
        let mods1 = TraitEngine::calculate_trait_modifier(TraitType::SlowStarter, &ctx1);
        assert_eq!(mods1.condition_mod, -2);

        // 第3局
        let ctx3 = TraitContext {
            game_number: 3,
            ..Default::default()
        };
        let mods3 = TraitEngine::calculate_trait_modifier(TraitType::SlowStarter, &ctx3);
        assert_eq!(mods3.condition_mod, 2);
    }

    #[test]
    fn test_comeback_king_trait() {
        // 落后时
        let ctx = TraitContext {
            score_diff: -1,
            ..Default::default()
        };
        let mods = TraitEngine::calculate_trait_modifier(TraitType::ComebackKing, &ctx);
        assert_eq!(mods.condition_mod, 3);

        // 领先时
        let ctx2 = TraitContext {
            score_diff: 1,
            ..Default::default()
        };
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
        assert_eq!(mods.condition_mod, 3); // Clutch
        assert_eq!(mods.stability_mod, -15); // Explosive
        assert_eq!(mods.ability_ceiling_mod, 5); // Explosive
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
        assert_eq!(ceiling, 98); // 83 + 10 + 5 = 98
    }

    #[test]
    fn test_trait_rarity() {
        assert!(TraitType::TeamLeader.rarity() > TraitType::Tilter.rarity());
        assert!(TraitType::Clutch.rarity() > TraitType::SlowStarter.rarity());
    }
}
