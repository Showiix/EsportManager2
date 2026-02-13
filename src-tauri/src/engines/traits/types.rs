//! 特性类型定义

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
