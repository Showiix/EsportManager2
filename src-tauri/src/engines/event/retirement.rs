use crate::models::{Player, PlayerRetirementDetail, PlayerStatus, RetirementReason, Team};
use rand::Rng;

use super::EventEngine;

impl EventEngine {
    /// 判断选手是否应该退役
    pub fn should_retire(&self, player: &Player) -> Option<RetirementReason> {
        if player.status == PlayerStatus::Retired {
            return None;
        }

        if player.age >= self.config.retirement_age {
            return Some(RetirementReason::Age);
        }

        if player.ability < self.config.low_ability_threshold {
            return Some(RetirementReason::LowAbility);
        }

        if player.age >= self.config.age_ability_age_threshold
            && player.ability < self.config.age_ability_ability_threshold
        {
            let mut rng = rand::thread_rng();
            let retire_probability = 0.3 + (player.age - 30) as f64 * 0.1;
            if rng.gen::<f64>() < retire_probability {
                return Some(RetirementReason::AgeAndAbility);
            }
        }

        None
    }

    /// 处理选手退役
    pub fn process_retirement(
        &self,
        player: &Player,
        team: Option<&Team>,
        current_season: u32,
    ) -> Option<PlayerRetirementDetail> {
        let reason = self.should_retire(player)?;

        Some(PlayerRetirementDetail {
            player_id: player.id,
            player_name: player.game_id.clone(),
            team_id: player.team_id,
            team_name: team.map(|t| t.name.clone()),
            final_ability: player.ability,
            age: player.age,
            career_seasons: current_season.saturating_sub(player.join_season),
            reason,
        })
    }

    /// 应用退役事件到选手
    pub fn apply_retirement(&self, player: &mut Player, current_season: u32) {
        player.status = PlayerStatus::Retired;
        player.retire_season = Some(current_season);
        player.team_id = None;
        player.is_starter = false;
    }
}
