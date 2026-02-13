#[cfg(test)]
mod tests {
    use super::super::engine::TraitEngine;
    use super::super::modifiers::TraitContext;
    use super::super::types::TraitType;

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
        let ctx1 = TraitContext {
            game_number: 1,
            ..Default::default()
        };
        let mods1 = TraitEngine::calculate_trait_modifier(TraitType::SlowStarter, &ctx1);
        assert_eq!(mods1.condition_mod, -2);

        let ctx3 = TraitContext {
            game_number: 3,
            ..Default::default()
        };
        let mods3 = TraitEngine::calculate_trait_modifier(TraitType::SlowStarter, &ctx3);
        assert_eq!(mods3.condition_mod, 2);
    }

    #[test]
    fn test_comeback_king_trait() {
        let ctx = TraitContext {
            score_diff: -1,
            ..Default::default()
        };
        let mods = TraitEngine::calculate_trait_modifier(TraitType::ComebackKing, &ctx);
        assert_eq!(mods.condition_mod, 3);

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
        assert_eq!(mods.condition_mod, 3);
        assert_eq!(mods.stability_mod, -15);
        assert_eq!(mods.ability_ceiling_mod, 5);
    }

    #[test]
    fn test_apply_modifiers() {
        let modifiers = super::super::modifiers::TraitModifiers {
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
        assert_eq!(ceiling, 98);
    }

    #[test]
    fn test_trait_rarity() {
        assert!(TraitType::TeamLeader.rarity() > TraitType::Tilter.rarity());
        assert!(TraitType::Clutch.rarity() > TraitType::SlowStarter.rarity());
    }
}
