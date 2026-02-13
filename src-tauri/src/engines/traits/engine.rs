//! 特性引擎核心逻辑

use super::modifiers::{TraitContext, TraitModifiers};
use super::types::TraitType;

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

            TraitType::Fragile => {}

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

            TraitType::Ironman => {}

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

            TraitType::TeamLeader => {}

            TraitType::LoneWolf => {
                mods.ability_mod = 2;
                mods.condition_mod = -1;
            }

            TraitType::Supportive => {}

            TraitType::Troublemaker => {
                mods.ability_mod = 1;
                mods.condition_mod = -2;
            }

            TraitType::Mentor => {}

            TraitType::Perfectionist => {}

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
        let modified_ability =
            (base_ability as i16 + modifiers.ability_mod as i16).clamp(1, 100) as u8;

        let modified_stability =
            (base_stability as i16 + modifiers.stability_mod as i16).clamp(30, 100) as u8;

        let modified_condition =
            (base_condition as i16 + modifiers.condition_mod as i16).clamp(-10, 10) as i8;

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

        let trait_count = match ability {
            68..=100 => 2 + rng.gen_range(0..2),
            61..=67 => 1 + rng.gen_range(0..2),
            54..=60 => rng.gen_range(0..2),
            _ => {
                if rng.gen::<f64>() < 0.3 {
                    1
                } else {
                    0
                }
            }
        };

        if trait_count == 0 {
            return traits;
        }

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

        for _ in 0..trait_count {
            if available.is_empty() {
                break;
            }

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

            Self::remove_conflicting_traits(&mut available, selected);
        }

        traits
    }

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

        let mut awakened_one = false;

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

    fn get_awakening_candidates(
        ability: u8,
        age: u8,
        games_played: i32,
        avg_performance: f64,
        existing: &[TraitType],
        seasons_in_team: i64,
    ) -> Vec<(TraitType, f64)> {
        let mut candidates = Vec::new();

        if ability >= 75 && games_played >= 40 && avg_performance > 0.8 {
            candidates.push((TraitType::Clutch, 0.06));
            candidates.push((TraitType::BigGame, 0.08));
        }

        if ability >= 80 && avg_performance > 1.5 {
            candidates.push((TraitType::FinalsKiller, 0.04));
        }

        if games_played >= 40 && avg_performance > 0.2 && avg_performance < 0.8 && ability >= 65 {
            candidates.push((TraitType::RegularKing, 0.08));
        }

        if avg_performance > 1.0 && games_played >= 35 && ability >= 70 {
            candidates.push((TraitType::WinStreak, 0.08));
        }

        if ability >= 72 && avg_performance > 0.8 {
            candidates.push((TraitType::Explosive, 0.05));
        }

        if games_played >= 40 && avg_performance > 0.0 && avg_performance < 0.5 && ability >= 65 {
            candidates.push((TraitType::Consistent, 0.08));
        }

        if ability >= 70 && avg_performance > 0.5 {
            candidates.push((TraitType::Gambler, 0.03));
        }

        if avg_performance > 0.8 && ability >= 72 && games_played >= 30 {
            candidates.push((TraitType::ComebackKing, 0.06));
            candidates.push((TraitType::PressurePlayer, 0.05));
        }

        if avg_performance < -0.8 && games_played >= 25 {
            candidates.push((TraitType::Tilter, 0.10));
            candidates.push((TraitType::Fragile, 0.06));
            candidates.push((TraitType::Choker, 0.06));
        }

        if games_played >= 50 && avg_performance > 0.0 {
            candidates.push((TraitType::Ironman, 0.06));
            candidates.push((TraitType::Endurance, 0.08));
        }
        if games_played >= 55 && avg_performance > 0.3 {
            candidates.push((TraitType::TournamentHorse, 0.05));
        }

        if ability >= 75 && avg_performance > 0.5 && games_played >= 35 {
            candidates.push((TraitType::MentalFortress, 0.05));
        }

        if age >= 29 && ability >= 68 {
            candidates.push((TraitType::Veteran, 0.10));
            candidates.push((TraitType::BattleTested, 0.08));
        }
        if age >= 30 && ability >= 70 && seasons_in_team >= 2 {
            candidates.push((TraitType::Mentor, 0.06));
        }

        if age <= 19 && ability >= 70 {
            candidates.push((TraitType::Prodigy, 0.06));
            candidates.push((TraitType::RisingStar, 0.08));
        }

        if seasons_in_team >= 4 && ability >= 70 {
            candidates.push((TraitType::TeamLeader, 0.05));
        }
        if seasons_in_team >= 3 && ability >= 65 && avg_performance > 0.0 {
            candidates.push((TraitType::Supportive, 0.06));
        }

        if age >= 24 && age <= 27 && ability >= 75 && avg_performance > 0.5 {
            candidates.push((TraitType::PeakAge, 0.08));
            candidates.push((TraitType::PeakForm, 0.04));
        }

        if age <= 21 && ability >= 72 && avg_performance > 0.8 {
            candidates.push((TraitType::Limitless, 0.04));
        }

        if age >= 25 && ability < 55 && avg_performance < -0.2 {
            candidates.push((TraitType::LowCeiling, 0.08));
        }
        if age >= 27 && avg_performance < -0.5 {
            candidates.push((TraitType::EarlyDecline, 0.06));
        }

        if seasons_in_team <= 1 && avg_performance > 0.8 && ability >= 68 {
            candidates.push((TraitType::Adaptable, 0.08));
        }

        if ability >= 75 && avg_performance > 0.8 && seasons_in_team <= 1 {
            candidates.push((TraitType::LoneWolf, 0.05));
        }
        if avg_performance < -0.5 && ability >= 68 {
            candidates.push((TraitType::Troublemaker, 0.05));
        }

        if age >= 30 && ability >= 70 {
            candidates.push((TraitType::Resilient, 0.06));
        }

        if age >= 26 && ability >= 72 && avg_performance > 0.8 {
            candidates.push((TraitType::LateBlocker, 0.05));
        }

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

        if seasons_in_team >= 4 && avg_performance > 0.5 && ability >= 70 {
            candidates.push((TraitType::Perfectionist, 0.06));
        }

        candidates.retain(|(t, _)| !existing.contains(t));

        candidates
    }

    fn get_decay_probability(
        trait_type: TraitType,
        ability: u8,
        age: u8,
        games_played: i32,
        avg_performance: f64,
    ) -> f64 {
        match trait_type {
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
            _ => 0.0,
        }
    }
}
