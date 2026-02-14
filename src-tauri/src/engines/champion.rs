use crate::models::player::Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Archetype {
    Aggressive,
    Scaling,
    Utility,
    Splitpush,
    Teamfight,
}

impl Archetype {
    pub fn id(&self) -> &'static str {
        match self {
            Archetype::Aggressive => "aggressive",
            Archetype::Scaling => "scaling",
            Archetype::Utility => "utility",
            Archetype::Splitpush => "splitpush",
            Archetype::Teamfight => "teamfight",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Archetype::Aggressive => "激进",
            Archetype::Scaling => "后期",
            Archetype::Utility => "功能",
            Archetype::Splitpush => "分推",
            Archetype::Teamfight => "团战",
        }
    }

    pub fn all() -> &'static [Archetype] {
        &[
            Archetype::Aggressive,
            Archetype::Scaling,
            Archetype::Utility,
            Archetype::Splitpush,
            Archetype::Teamfight,
        ]
    }

    pub fn from_id(id: &str) -> Option<Archetype> {
        match id {
            "aggressive" => Some(Archetype::Aggressive),
            "scaling" => Some(Archetype::Scaling),
            "utility" => Some(Archetype::Utility),
            "splitpush" => Some(Archetype::Splitpush),
            "teamfight" => Some(Archetype::Teamfight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MasteryTier {
    SS,
    S,
    A,
    B,
}

impl MasteryTier {
    pub fn id(&self) -> &'static str {
        match self {
            MasteryTier::SS => "SS",
            MasteryTier::S => "S",
            MasteryTier::A => "A",
            MasteryTier::B => "B",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            MasteryTier::SS => "信仰",
            MasteryTier::S => "招牌",
            MasteryTier::A => "熟练",
            MasteryTier::B => "生疏",
        }
    }

    pub fn pick_score(&self) -> i8 {
        match self {
            MasteryTier::SS => 12,
            MasteryTier::S => 8,
            MasteryTier::A => 4,
            MasteryTier::B => 0,
        }
    }

    pub fn from_id(id: &str) -> Option<MasteryTier> {
        match id {
            "SS" => Some(MasteryTier::SS),
            "S" => Some(MasteryTier::S),
            "A" => Some(MasteryTier::A),
            "B" => Some(MasteryTier::B),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionTier {
    T1,
    T2,
    T3,
    T4,
    T5,
}

impl VersionTier {
    pub fn modifier(&self) -> i8 {
        match self {
            VersionTier::T1 => 5,
            VersionTier::T2 => 2,
            VersionTier::T3 => 0,
            VersionTier::T4 => -2,
            VersionTier::T5 => -4,
        }
    }

    pub fn ban_score(&self) -> i8 {
        match self {
            VersionTier::T1 => 6,
            VersionTier::T2 => 2,
            VersionTier::T3 => 0,
            VersionTier::T4 => -3,
            VersionTier::T5 => -6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Champion {
    pub id: u8,
    pub name_cn: &'static str,
    pub name_en: &'static str,
    pub position: Position,
    pub archetype: Archetype,
}

pub const CHAMPIONS: &[Champion] = &[
    // ===== TOP (10) =====
    Champion {
        id: 1,
        name_cn: "雷克顿",
        name_en: "Renekton",
        position: Position::Top,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 2,
        name_cn: "菲奥娜",
        name_en: "Fiora",
        position: Position::Top,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 3,
        name_cn: "贾克斯",
        name_en: "Jax",
        position: Position::Top,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 4,
        name_cn: "艾瑞莉娅",
        name_en: "Irelia",
        position: Position::Top,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 5,
        name_cn: "慎",
        name_en: "Shen",
        position: Position::Top,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 6,
        name_cn: "奥恩",
        name_en: "Ornn",
        position: Position::Top,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 7,
        name_cn: "剑魔",
        name_en: "Aatrox",
        position: Position::Top,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 8,
        name_cn: "格温",
        name_en: "Gwen",
        position: Position::Top,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 9,
        name_cn: "墨菲特",
        name_en: "Malphite",
        position: Position::Top,
        archetype: Archetype::Teamfight,
    },
    Champion {
        id: 10,
        name_cn: "莫德凯撒",
        name_en: "Mordekaiser",
        position: Position::Top,
        archetype: Archetype::Teamfight,
    },
    // ===== JUG (10) =====
    Champion {
        id: 11,
        name_cn: "李青",
        name_en: "Lee Sin",
        position: Position::Jug,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 12,
        name_cn: "雷恩加尔",
        name_en: "Rengar",
        position: Position::Jug,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 13,
        name_cn: "莉莉娅",
        name_en: "Lillia",
        position: Position::Jug,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 14,
        name_cn: "卡尔萨斯",
        name_en: "Karthus",
        position: Position::Jug,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 15,
        name_cn: "伊芙琳",
        name_en: "Evelynn",
        position: Position::Jug,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 16,
        name_cn: "赵信",
        name_en: "Xin Zhao",
        position: Position::Jug,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 17,
        name_cn: "奥拉夫",
        name_en: "Olaf",
        position: Position::Jug,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 18,
        name_cn: "特朗德尔",
        name_en: "Trundle",
        position: Position::Jug,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 19,
        name_cn: "阿木木",
        name_en: "Amumu",
        position: Position::Jug,
        archetype: Archetype::Teamfight,
    },
    Champion {
        id: 20,
        name_cn: "斯卡纳",
        name_en: "Skarner",
        position: Position::Jug,
        archetype: Archetype::Teamfight,
    },
    // ===== MID (10) =====
    Champion {
        id: 21,
        name_cn: "劫",
        name_en: "Zed",
        position: Position::Mid,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 22,
        name_cn: "泰隆",
        name_en: "Talon",
        position: Position::Mid,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 23,
        name_cn: "阿兹尔",
        name_en: "Azir",
        position: Position::Mid,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 24,
        name_cn: "维克托",
        name_en: "Viktor",
        position: Position::Mid,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 25,
        name_cn: "卡尔玛",
        name_en: "Karma",
        position: Position::Mid,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 26,
        name_cn: "加里奥",
        name_en: "Galio",
        position: Position::Mid,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 27,
        name_cn: "阿卡丽",
        name_en: "Akali",
        position: Position::Mid,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 28,
        name_cn: "杰斯",
        name_en: "Jayce",
        position: Position::Mid,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 29,
        name_cn: "奥莉安娜",
        name_en: "Orianna",
        position: Position::Mid,
        archetype: Archetype::Teamfight,
    },
    Champion {
        id: 30,
        name_cn: "卡萨丁",
        name_en: "Kassadin",
        position: Position::Mid,
        archetype: Archetype::Teamfight,
    },
    // ===== ADC (10) =====
    Champion {
        id: 31,
        name_cn: "德莱文",
        name_en: "Draven",
        position: Position::Adc,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 32,
        name_cn: "卢锡安",
        name_en: "Lucian",
        position: Position::Adc,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 33,
        name_cn: "薇恩",
        name_en: "Vayne",
        position: Position::Adc,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 34,
        name_cn: "凯莎",
        name_en: "Kai'Sa",
        position: Position::Adc,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 35,
        name_cn: "艾希",
        name_en: "Ashe",
        position: Position::Adc,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 36,
        name_cn: "赛娜",
        name_en: "Senna",
        position: Position::Adc,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 37,
        name_cn: "厄运小姐",
        name_en: "Miss Fortune",
        position: Position::Adc,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 38,
        name_cn: "泽丽",
        name_en: "Zeri",
        position: Position::Adc,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 39,
        name_cn: "克格莫",
        name_en: "Kog'Maw",
        position: Position::Adc,
        archetype: Archetype::Teamfight,
    },
    Champion {
        id: 40,
        name_cn: "金克斯",
        name_en: "Jinx",
        position: Position::Adc,
        archetype: Archetype::Teamfight,
    },
    // ===== SUP (10) =====
    Champion {
        id: 41,
        name_cn: "潘森",
        name_en: "Pantheon",
        position: Position::Sup,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 42,
        name_cn: "派克",
        name_en: "Pyke",
        position: Position::Sup,
        archetype: Archetype::Aggressive,
    },
    Champion {
        id: 43,
        name_cn: "塞拉菲妮",
        name_en: "Seraphine",
        position: Position::Sup,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 44,
        name_cn: "索拉卡",
        name_en: "Soraka",
        position: Position::Sup,
        archetype: Archetype::Scaling,
    },
    Champion {
        id: 45,
        name_cn: "璐璐",
        name_en: "Lulu",
        position: Position::Sup,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 46,
        name_cn: "娜美",
        name_en: "Nami",
        position: Position::Sup,
        archetype: Archetype::Utility,
    },
    Champion {
        id: 47,
        name_cn: "巴德",
        name_en: "Bard",
        position: Position::Sup,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 48,
        name_cn: "莫甘娜",
        name_en: "Morgana",
        position: Position::Sup,
        archetype: Archetype::Splitpush,
    },
    Champion {
        id: 49,
        name_cn: "蕾欧娜",
        name_en: "Leona",
        position: Position::Sup,
        archetype: Archetype::Teamfight,
    },
    Champion {
        id: 50,
        name_cn: "芮尔",
        name_en: "Rell",
        position: Position::Sup,
        archetype: Archetype::Teamfight,
    },
];

pub fn get_champion(id: u8) -> Option<&'static Champion> {
    CHAMPIONS.iter().find(|c| c.id == id)
}

pub fn get_champions_by_position(position: Position) -> Vec<&'static Champion> {
    CHAMPIONS
        .iter()
        .filter(|c| c.position == position)
        .collect()
}

pub fn get_champions_by_archetype(archetype: Archetype) -> Vec<&'static Champion> {
    CHAMPIONS
        .iter()
        .filter(|c| c.archetype == archetype)
        .collect()
}

pub fn get_champion_by_name_en(name: &str) -> Option<&'static Champion> {
    CHAMPIONS.iter().find(|c| c.name_en == name)
}

use super::meta_engine::{get_meta_weights, MetaType};

fn position_weight_id(position: Position) -> &'static str {
    match position {
        Position::Top => "top",
        Position::Jug => "jug",
        Position::Mid => "mid",
        Position::Adc => "adc",
        Position::Sup => "sup",
    }
}

/// 根据当前 Meta 计算每个英雄的版本 Tier
pub fn calculate_version_tiers(meta: MetaType) -> Vec<(u8, VersionTier)> {
    let favored = meta.favored_archetypes();
    let disfavored = meta.disfavored_archetypes();
    let weights = get_meta_weights(meta);

    CHAMPIONS
        .iter()
        .map(|c| {
            let tier = if favored.is_empty() {
                VersionTier::T3
            } else if favored.contains(&c.archetype) {
                let pos_weight = weights.weight_for_position(position_weight_id(c.position));
                if pos_weight >= 1.1 {
                    VersionTier::T1
                } else {
                    VersionTier::T2
                }
            } else if disfavored.contains(&c.archetype) {
                let pos_weight = weights.weight_for_position(position_weight_id(c.position));
                if pos_weight <= 0.85 {
                    VersionTier::T5
                } else {
                    VersionTier::T4
                }
            } else {
                VersionTier::T3
            };
            (c.id, tier)
        })
        .collect()
}

pub fn get_version_tier(champion_id: u8, meta: MetaType) -> VersionTier {
    let favored = meta.favored_archetypes();
    let disfavored = meta.disfavored_archetypes();
    let weights = get_meta_weights(meta);

    let champ = match get_champion(champion_id) {
        Some(c) => c,
        None => return VersionTier::T3,
    };

    if favored.is_empty() {
        VersionTier::T3
    } else if favored.contains(&champ.archetype) {
        let pos_weight = weights.weight_for_position(position_weight_id(champ.position));
        if pos_weight >= 1.1 {
            VersionTier::T1
        } else {
            VersionTier::T2
        }
    } else if disfavored.contains(&champ.archetype) {
        let pos_weight = weights.weight_for_position(position_weight_id(champ.position));
        if pos_weight <= 0.85 {
            VersionTier::T5
        } else {
            VersionTier::T4
        }
    } else {
        VersionTier::T3
    }
}

/// 赛季演变：根据使用次数决定熟练度升降
/// games_played: 该选手该英雄本赛季使用场次
/// 返回新的 MasteryTier（可能不变）
pub fn evolve_mastery(
    current: MasteryTier,
    games_played: u32,
    rng: &mut impl rand::Rng,
) -> MasteryTier {
    match current {
        MasteryTier::SS => {
            // SS 不降级
            MasteryTier::SS
        }
        MasteryTier::S => {
            if games_played == 0 {
                // 未使用：30% 降为 A
                if rng.gen::<f64>() < 0.30 {
                    MasteryTier::A
                } else {
                    MasteryTier::S
                }
            } else if games_played >= 15 {
                // 大量使用：20% 升为 SS
                if rng.gen::<f64>() < 0.20 {
                    MasteryTier::SS
                } else {
                    MasteryTier::S
                }
            } else {
                MasteryTier::S
            }
        }
        MasteryTier::A => {
            if games_played == 0 {
                // 未使用：40% 降为 B
                if rng.gen::<f64>() < 0.40 {
                    MasteryTier::B
                } else {
                    MasteryTier::A
                }
            } else if games_played >= 10 {
                // 常用：25% 升为 S
                if rng.gen::<f64>() < 0.25 {
                    MasteryTier::S
                } else {
                    MasteryTier::A
                }
            } else {
                MasteryTier::A
            }
        }
        MasteryTier::B => {
            if games_played >= 8 {
                // 频繁使用生疏英雄：30% 升为 A
                if rng.gen::<f64>() < 0.30 {
                    MasteryTier::A
                } else {
                    MasteryTier::B
                }
            } else {
                MasteryTier::B
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_champion_count() {
        assert_eq!(CHAMPIONS.len(), 50);
    }

    #[test]
    fn test_unique_ids() {
        let mut ids: Vec<u8> = CHAMPIONS.iter().map(|c| c.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 50);
    }

    #[test]
    fn test_position_distribution() {
        for pos in &[
            Position::Top,
            Position::Jug,
            Position::Mid,
            Position::Adc,
            Position::Sup,
        ] {
            let count = CHAMPIONS.iter().filter(|c| c.position == *pos).count();
            assert_eq!(
                count, 10,
                "Position {:?} should have 10 champions, got {}",
                pos, count
            );
        }
    }

    #[test]
    fn test_archetype_distribution() {
        for arch in Archetype::all() {
            let count = CHAMPIONS.iter().filter(|c| c.archetype == *arch).count();
            assert_eq!(
                count, 10,
                "Archetype {:?} should have 10 champions, got {}",
                arch, count
            );
        }
    }

    #[test]
    fn test_cross_distribution() {
        for pos in &[
            Position::Top,
            Position::Jug,
            Position::Mid,
            Position::Adc,
            Position::Sup,
        ] {
            for arch in Archetype::all() {
                let count = CHAMPIONS
                    .iter()
                    .filter(|c| c.position == *pos && c.archetype == *arch)
                    .count();
                assert_eq!(
                    count, 2,
                    "{:?}/{:?} should have 2, got {}",
                    pos, arch, count
                );
            }
        }
    }

    #[test]
    fn test_lookup_functions() {
        assert!(get_champion(1).is_some());
        assert_eq!(get_champion(1).unwrap().name_en, "Renekton");
        assert!(get_champion(0).is_none());
        assert!(get_champion(51).is_none());

        let tops = get_champions_by_position(Position::Top);
        assert_eq!(tops.len(), 10);

        let aggressives = get_champions_by_archetype(Archetype::Aggressive);
        assert_eq!(aggressives.len(), 10);

        assert!(get_champion_by_name_en("Zed").is_some());
        assert!(get_champion_by_name_en("FakeChamp").is_none());
    }
}
