use crate::models::{
    Player, PlayerAgingResult, PlayerDeclineDetail, PlayerGrowthDetail, PlayerStatus, PlayerTag,
    Position, RookieGenerationDetail, Team,
};
use rand::Rng;

use super::EventEngine;

impl EventEngine {
    /// 计算选手能力成长
    /// 根据标签决定成长值: 天才+3, 普通+2, 平庸+1
    /// 成长上限为潜力值
    pub fn calculate_player_growth(&self, player: &Player) -> Option<PlayerGrowthDetail> {
        if player.age >= self.config.growth_stop_age {
            return None;
        }

        if player.ability >= player.potential {
            return None;
        }

        if player.status == PlayerStatus::Retired {
            return None;
        }

        let growth_amount = player.tag.growth_per_season();
        let new_ability = (player.ability + growth_amount).min(player.potential);

        if new_ability > player.ability {
            Some(PlayerGrowthDetail {
                player_id: player.id,
                player_name: player.game_id.clone(),
                old_ability: player.ability,
                new_ability,
                growth_amount: new_ability - player.ability,
                tag: format!("{:?}", player.tag),
                reason: format!(
                    "赛季结束能力成长 ({} 标签 +{})",
                    match player.tag {
                        PlayerTag::Genius => "天才",
                        PlayerTag::Normal => "普通",
                        PlayerTag::Ordinary => "平庸",
                    },
                    growth_amount
                ),
            })
        } else {
            None
        }
    }

    /// 计算选手能力衰退
    /// 30岁以上开始衰退，衰退量随年龄增加
    pub fn calculate_player_decline(&self, player: &Player) -> Option<PlayerDeclineDetail> {
        if player.age < self.config.decline_start_age {
            return None;
        }

        if player.status == PlayerStatus::Retired {
            return None;
        }

        let decline_amount = match player.age {
            30..=31 => 1,
            32..=33 => 2,
            34..=35 => 3,
            _ => 4,
        };

        let mut rng = rand::thread_rng();
        let random_factor: i8 = rng.gen_range(-1..=1);
        let final_decline = (decline_amount as i8 + random_factor).max(0) as u8;

        if final_decline == 0 {
            return None;
        }

        let new_ability = player.ability.saturating_sub(final_decline);

        Some(PlayerDeclineDetail {
            player_id: player.id,
            player_name: player.game_id.clone(),
            old_ability: player.ability,
            new_ability,
            decline_amount: player.ability - new_ability,
            age: player.age,
            reason: format!("年龄增长导致能力下滑 ({}岁)", player.age),
        })
    }

    /// 更新选手年龄和稳定性
    pub fn update_player_age(&self, player: &Player) -> PlayerAgingResult {
        let new_age = player.age + 1;
        let new_stability = Player::calculate_stability(new_age);

        PlayerAgingResult {
            player_id: player.id,
            player_name: player.game_id.clone(),
            old_age: player.age,
            new_age,
            old_stability: player.stability,
            new_stability,
        }
    }

    /// 生成新秀选手（使用 RookieGenerator 生成拟真信息）
    pub fn generate_rookie(
        &self,
        team: &Team,
        position: Position,
        _season_id: u32,
    ) -> RookieGenerationDetail {
        let mut generator = super::super::RookieGenerator::from_entropy();
        let existing_ids = std::collections::HashSet::new();
        let rookies = generator.generate_rookies(1, 1, &existing_ids);

        if let Some(r) = rookies.into_iter().next() {
            RookieGenerationDetail {
                player_id: 0,
                player_name: r.game_id,
                team_id: team.id,
                team_name: team.name.clone(),
                ability: r.ability,
                potential: r.potential,
                position: format!("{:?}", position),
                tag: r.tag,
            }
        } else {
            let mut rng = rand::thread_rng();
            let ability = rng.gen_range(61..=64);
            let potential = ability + rng.gen_range(2..=4);
            RookieGenerationDetail {
                player_id: 0,
                player_name: format!("Rookie{}", rng.gen_range(1..9999)),
                team_id: team.id,
                team_name: team.name.clone(),
                ability,
                potential,
                position: format!("{:?}", position),
                tag: "Normal".to_string(),
            }
        }
    }

    /// 应用成长事件到选手
    pub fn apply_growth(&self, player: &mut Player, growth: &PlayerGrowthDetail) {
        player.ability = growth.new_ability;
        player.market_value = player.calculate_market_value();
    }

    /// 应用衰退事件到选手
    pub fn apply_decline(&self, player: &mut Player, decline: &PlayerDeclineDetail) {
        player.ability = decline.new_ability;
        player.market_value = player.calculate_market_value();
    }

    /// 应用年龄更新到选手
    pub fn apply_aging(&self, player: &mut Player, aging: &PlayerAgingResult) {
        player.age = aging.new_age;
        player.stability = aging.new_stability;
    }
}
