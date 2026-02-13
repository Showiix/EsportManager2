use crate::models::{Player, PlayerStatus, SeasonSettlementResult, Team};
use serde::{Deserialize, Serialize};

use super::EventEngine;

/// 事件引擎的辅助结构，用于批量处理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonEndProcessResult {
    /// 需要更新的选手ID和新能力值
    pub ability_updates: Vec<(u64, u8)>,
    /// 需要更新的选手ID和新年龄/稳定性
    pub age_updates: Vec<(u64, u8, u8)>,
    /// 需要标记退役的选手ID
    pub retirements: Vec<u64>,
    /// 合同续约信息: (选手ID, 是否续约, 新合同年限, 新薪资)
    pub contract_updates: Vec<(u64, bool, Option<u32>, Option<u64>)>,
    /// 成为自由球员的选手ID
    pub free_agents: Vec<u64>,
}

impl EventEngine {
    /// 执行完整的赛季结算
    pub fn process_season_settlement(
        &self,
        season_id: u64,
        players: &[Player],
        teams: &[Team],
        current_season: u32,
    ) -> SeasonSettlementResult {
        let mut growth_events = Vec::new();
        let mut decline_events = Vec::new();
        let mut retirement_events = Vec::new();
        let mut contract_expire_events = Vec::new();
        let rookie_events = Vec::new();

        let team_map: std::collections::HashMap<u64, &Team> =
            teams.iter().map(|t| (t.id, t)).collect();

        for player in players {
            if player.status == PlayerStatus::Retired {
                continue;
            }

            let team = player.team_id.and_then(|tid| team_map.get(&tid).copied());

            if let Some(detail) = self.process_retirement(player, team, current_season) {
                retirement_events.push(detail);
                continue;
            }

            if self.is_contract_expired(player, current_season) {
                if let Some(team) = team {
                    let detail = self.process_contract_expiration(player, team, current_season);
                    contract_expire_events.push(detail);
                }
            }

            if let Some(growth) = self.calculate_player_growth(player) {
                growth_events.push(growth);
            } else if let Some(decline) = self.calculate_player_decline(player) {
                decline_events.push(decline);
            }
        }

        SeasonSettlementResult {
            season_id,
            season_name: format!("S{}", season_id),
            growth_events,
            decline_events,
            retirement_events,
            contract_expire_events,
            rookie_events,
        }
    }

    /// 批量处理赛季结束事件，返回所有需要的数据库更新
    pub fn batch_process_season_end(
        &self,
        players: &[Player],
        teams: &[Team],
        current_season: u32,
    ) -> SeasonEndProcessResult {
        let mut result = SeasonEndProcessResult {
            ability_updates: Vec::new(),
            age_updates: Vec::new(),
            retirements: Vec::new(),
            contract_updates: Vec::new(),
            free_agents: Vec::new(),
        };

        let team_map: std::collections::HashMap<u64, &Team> =
            teams.iter().map(|t| (t.id, t)).collect();

        for player in players {
            if player.status == PlayerStatus::Retired {
                continue;
            }

            let team = player.team_id.and_then(|tid| team_map.get(&tid).copied());

            let aging = self.update_player_age(player);
            result
                .age_updates
                .push((player.id, aging.new_age, aging.new_stability));

            let aged_player = Player {
                age: aging.new_age,
                stability: aging.new_stability,
                ..player.clone()
            };

            if self.should_retire(&aged_player).is_some() {
                result.retirements.push(player.id);
                continue;
            }

            if self.is_contract_expired(player, current_season) {
                if let Some(team) = team {
                    let detail =
                        self.process_contract_expiration(&aged_player, team, current_season);
                    result.contract_updates.push((
                        player.id,
                        detail.renewed,
                        detail.new_contract_years,
                        detail.new_salary,
                    ));
                    if !detail.renewed {
                        result.free_agents.push(player.id);
                    }
                }
            }

            if let Some(growth) = self.calculate_player_growth(&aged_player) {
                result.ability_updates.push((player.id, growth.new_ability));
            } else if let Some(decline) = self.calculate_player_decline(&aged_player) {
                result
                    .ability_updates
                    .push((player.id, decline.new_ability));
            }
        }

        result
    }
}
