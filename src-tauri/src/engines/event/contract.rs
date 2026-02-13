use crate::models::{ContractExpireDetail, Player, Team};
use rand::Rng;

use super::EventEngine;

impl EventEngine {
    /// 判断合同是否到期
    pub fn is_contract_expired(&self, player: &Player, current_season: u32) -> bool {
        match player.contract_end_season {
            Some(end_season) => current_season >= end_season,
            None => false,
        }
    }

    fn calculate_renewal_probability(&self, player: &Player, team: &Team) -> f64 {
        let mut probability = self.config.contract_renewal_base_probability;

        if player.ability >= 61 {
            probability += 0.15;
        } else if player.ability >= 54 {
            probability += 0.05;
        } else if player.ability < 47 {
            probability -= 0.2;
        }

        if player.age <= 24 {
            probability += 0.1;
        } else if player.age >= 30 {
            probability -= 0.15;
        }

        if player.potential > player.ability + 10 {
            probability += 0.1;
        }

        if team.balance < 200 {
            probability -= 0.2;
        }

        probability.clamp(0.1, 0.95)
    }

    fn calculate_new_contract(&self, player: &Player) -> (u32, u64) {
        let mut rng = rand::thread_rng();

        let years = match player.age {
            age if age <= 24 => rng.gen_range(2..=4),
            age if age <= 28 => rng.gen_range(2..=3),
            _ => rng.gen_range(1..=2),
        };

        let base_salary = match player.ability {
            68..=100 => rng.gen_range(100..=200),
            62..=67 => rng.gen_range(50..=100),
            55..=61 => rng.gen_range(25..=50),
            47..=54 => rng.gen_range(10..=25),
            _ => rng.gen_range(5..=10),
        };

        (years, base_salary)
    }

    /// 处理合同到期
    pub fn process_contract_expiration(
        &self,
        player: &Player,
        team: &Team,
        _current_season: u32,
    ) -> ContractExpireDetail {
        let mut rng = rand::thread_rng();

        let renewal_probability = self.calculate_renewal_probability(player, team);
        let renewed = rng.gen::<f64>() < renewal_probability;

        let (new_contract_years, new_salary) = if renewed {
            let (years, salary) = self.calculate_new_contract(player);
            (Some(years), Some(salary))
        } else {
            (None, None)
        };

        ContractExpireDetail {
            player_id: player.id,
            player_name: player.game_id.clone(),
            team_id: team.id,
            team_name: team.name.clone(),
            ability: player.ability,
            age: player.age,
            renewed,
            new_contract_years,
            new_salary,
        }
    }

    /// 应用合同续约到选手
    pub fn apply_contract_renewal(
        &self,
        player: &mut Player,
        detail: &ContractExpireDetail,
        current_season: u32,
    ) {
        if detail.renewed {
            if let Some(years) = detail.new_contract_years {
                player.contract_end_season = Some(current_season + years);
            }
            if let Some(salary) = detail.new_salary {
                player.salary = salary;
            }
        } else {
            player.team_id = None;
            player.is_starter = false;
            player.contract_end_season = None;
        }
    }
}
