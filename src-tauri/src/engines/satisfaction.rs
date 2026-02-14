//! 选手满意度与忠诚度计算引擎
//!
//! 实现选手满意度和忠诚度的计算逻辑，包括：
//! - 赛季结算时计算满意度变化
//! - 计算忠诚度变化
//! - 判断离队意愿

use crate::engines::market_value::MarketValueEngine;
use crate::models::{
    DepartureReason, LoyaltyChange, LoyaltyChangeReason, Player, TeamSeasonPerformance,
};

/// 满意度计算引擎
pub struct SatisfactionEngine;

impl SatisfactionEngine {
    /// 赛季结算时计算满意度变化
    ///
    /// # 参数
    /// - `player`: 选手信息
    /// - `team_perf`: 球队赛季表现
    /// - `games_as_starter`: 作为首发的比赛场数
    /// - `total_games`: 总比赛场数
    /// - `market_value`: 当前身价（万元）
    ///
    /// # 返回
    /// 满意度变化值（可正可负）
    pub fn calculate_season_changes(
        player: &Player,
        team_perf: &TeamSeasonPerformance,
        games_played: u32,
        team_total_games: u32,
        contract_role: &str,
        market_value: u64,
    ) -> i32 {
        let mut change: i32 = 0;

        let play_rate = if team_total_games > 0 {
            games_played as f64 / team_total_games as f64
        } else {
            0.0
        };

        // expected_rate: Starter=0.75, Sub=0.20, Prospect=0.05
        let expected_rate: f64 = match contract_role {
            "Sub" => 0.20,
            "Prospect" => 0.05,
            _ => 0.75,
        };
        let rate_gap = expected_rate - play_rate;
        if rate_gap > 0.0 {
            // penalty = gap × 25 × (ability / 50)
            let ability_factor = player.ability as f64 / 50.0;
            change -= (rate_gap * 25.0 * ability_factor).round() as i32;
        } else if play_rate >= 0.8 {
            change += 8;
        } else if play_rate >= expected_rate {
            change += 3;
        }

        // 2. 球队战绩影响
        if let Some(rank) = team_perf.final_rank {
            if rank >= 8 && player.ability >= 58 {
                change -= 10;
            } else if rank >= 6 && player.ability >= 61 {
                change -= 6;
            } else if rank <= 2 {
                change += 10;
            } else if rank <= 4 {
                change += 8;
            }
        }

        if team_perf.consecutive_no_playoffs >= 2 {
            change -= 10;
        } else if team_perf.consecutive_no_playoffs >= 1 {
            change -= 3;
        }

        // 3. 薪资满意度
        let expected_salary =
            MarketValueEngine::estimate_salary(market_value, player.ability, player.age);
        let salary_ratio = if expected_salary > 0 {
            player.salary as f64 / expected_salary as f64
        } else {
            1.0
        };

        if salary_ratio < 0.5 {
            change -= 15;
        } else if salary_ratio < 0.6 {
            change -= 10;
        } else if salary_ratio < 0.8 {
            change -= 5;
        } else if salary_ratio > 1.2 {
            change += 5;
        }

        // 4. 夺冠加成
        if team_perf.is_champion() {
            change += 20;
        } else if team_perf.has_international_achievement() {
            change += 15;
        } else if team_perf.made_playoffs {
            change += 5;
        }

        // 5. 年龄因素
        if player.age >= 28 && team_perf.is_poor_performance() {
            change -= 6;
        }
        if player.age <= 24 && play_rate < 0.5 {
            change -= 3;
        }

        // 6. 满意度自然回归：每赛季向60靠拢，幅度为差值的10%，至少±1
        let current_sat = player.satisfaction as i64;
        let regression_target = 60i64;
        let diff = regression_target - current_sat;
        if diff != 0 {
            let regression = if diff > 0 {
                (diff as f64 * 0.1).ceil() as i32
            } else {
                (diff as f64 * 0.1).floor() as i32
            };
            change += regression;
        }

        change
    }

    /// 判断是否想离队
    ///
    /// # 参数
    /// - `satisfaction`: 当前满意度
    /// - `loyalty`: 当前忠诚度
    /// - `player`: 选手信息
    /// - `team_perf`: 球队赛季表现
    ///
    /// # 返回
    /// (是否想离队, 离队原因列表)
    pub fn check_departure_intent(
        satisfaction: u8,
        loyalty: u8,
        player: &Player,
        team_perf: &TeamSeasonPerformance,
        play_rate: f64,
    ) -> (bool, Vec<DepartureReason>) {
        let threshold = Player::departure_threshold_static(loyalty);
        let mut reasons = Vec::new();

        // 满意度高于阈值，不想离队
        if satisfaction > threshold {
            return (false, reasons);
        }

        // 收集离队原因
        // 1. 追求冠军：能力强但球队战绩差
        if player.ability >= 61 && team_perf.final_rank.unwrap_or(10) >= 6 {
            reasons.push(DepartureReason::SeekingChampionship);
        }

        // 2. 寻找机会：年轻且上场少
        if player.age <= 24 && play_rate < 0.3 {
            reasons.push(DepartureReason::SeekingOpportunity);
        }

        // 3. 球队战绩差
        if team_perf.consecutive_no_playoffs >= 2 || team_perf.is_poor_performance() {
            reasons.push(DepartureReason::TeamPerformance);
        }

        // 4. 薪资不满（通过满意度间接判断）
        if satisfaction < 35 {
            let market_value = player.calculate_market_value();
            let expected =
                MarketValueEngine::estimate_salary(market_value, player.ability, player.age);
            if player.salary < (expected as f64 * 0.7) as u64 {
                reasons.push(DepartureReason::SalaryDispute);
            }
        }

        // 5. 缺少上场时间
        if play_rate < 0.3 && player.ability >= 54 {
            reasons.push(DepartureReason::LackOfPlaytime);
        }

        // 判断是否想离队：有明确原因，或满意度极低
        let wants_to_leave = !reasons.is_empty() || satisfaction < 30;

        (wants_to_leave, reasons)
    }

    /// 初始化新选手的满意度
    pub fn initial_satisfaction(player: &Player, is_from_draft: bool) -> u8 {
        let mut base = 70u8;

        // 选秀新人初始满意度较高
        if is_from_draft {
            base += 10;
        }

        // 首发满意度较高
        if player.is_starter {
            base += 5;
        }

        base.min(100)
    }
}

/// 忠诚度计算引擎
pub struct LoyaltyEngine;

impl LoyaltyEngine {
    /// 计算赛季忠诚度变化
    ///
    /// # 参数
    /// - `player`: 选手信息
    /// - `team_perf`: 球队赛季表现
    /// - `seasons_with_team`: 在当前球队效力的赛季数
    /// - `is_from_draft`: 是否为选秀出身
    /// - `was_listed`: 是否在本赛季被挂牌出售过
    ///
    /// # 返回
    /// (忠诚度变化值, 变化原因列表)
    pub fn calculate_season_changes(
        player: &Player,
        team_perf: &TeamSeasonPerformance,
        seasons_with_team: u32,
        is_from_draft: bool,
        was_listed: bool,
    ) -> Vec<(i32, LoyaltyChangeReason)> {
        let mut changes = Vec::new();

        // 1. 每赛季自然增长
        changes.push((
            LoyaltyChangeReason::SeasonPassed.default_change(),
            LoyaltyChangeReason::SeasonPassed,
        ));

        // 2. 青训出身加成（首个赛季）
        if seasons_with_team == 1 && is_from_draft {
            changes.push((
                LoyaltyChangeReason::DraftOrigin.default_change(),
                LoyaltyChangeReason::DraftOrigin,
            ));
        }

        // 3. 球队战绩影响
        if team_perf.is_champion() {
            changes.push((
                LoyaltyChangeReason::TeamChampion.default_change(),
                LoyaltyChangeReason::TeamChampion,
            ));
        } else if team_perf.made_playoffs {
            changes.push((
                LoyaltyChangeReason::MadePlayoffs.default_change(),
                LoyaltyChangeReason::MadePlayoffs,
            ));
        } else if team_perf.is_poor_performance() {
            changes.push((
                LoyaltyChangeReason::PoorTeamPerformance.default_change(),
                LoyaltyChangeReason::PoorTeamPerformance,
            ));
        }

        // 4. 长期替补惩罚
        if !player.is_starter && seasons_with_team >= 2 {
            changes.push((
                LoyaltyChangeReason::LongTermBench.default_change(),
                LoyaltyChangeReason::LongTermBench,
            ));
        }

        // 5. 被挂牌出售
        if was_listed {
            changes.push((
                LoyaltyChangeReason::ListedForSale.default_change(),
                LoyaltyChangeReason::ListedForSale,
            ));
        }

        changes
    }

    /// 续约涨薪时的忠诚度变化
    pub fn on_salary_raise(old_salary: u64, new_salary: u64) -> Option<(i32, LoyaltyChangeReason)> {
        if new_salary > old_salary {
            let raise_ratio = (new_salary - old_salary) as f64 / old_salary as f64;
            if raise_ratio >= 0.2 {
                // 涨薪20%以上
                let bonus = if raise_ratio >= 0.5 { 10 } else { 5 };
                return Some((bonus, LoyaltyChangeReason::SalaryRaise));
            }
        }
        None
    }

    /// 续约谈崩时的忠诚度变化
    pub fn on_renewal_failed() -> (i32, LoyaltyChangeReason) {
        (
            LoyaltyChangeReason::RenewalFailed.default_change(),
            LoyaltyChangeReason::RenewalFailed,
        )
    }

    /// 成为首发时的忠诚度变化
    pub fn on_became_starter() -> (i32, LoyaltyChangeReason) {
        (
            LoyaltyChangeReason::BecameStarter.default_change(),
            LoyaltyChangeReason::BecameStarter,
        )
    }

    /// 初始化新选手的忠诚度
    pub fn initial_loyalty(is_from_draft: bool, team_reputation: f64) -> u8 {
        let mut base = 50u8;

        // 选秀出身有额外忠诚度
        if is_from_draft {
            base += 10;
        }

        // 根据球队声望调整（0-1.0）
        let reputation_bonus = (team_reputation * 10.0) as u8;
        base += reputation_bonus.min(10);

        base.min(100)
    }

    /// 应用忠诚度变化并生成记录
    pub fn apply_changes(
        player: &mut Player,
        changes: Vec<(i32, LoyaltyChangeReason)>,
        save_id: &str,
        season_id: u64,
    ) -> Vec<LoyaltyChange> {
        let mut records = Vec::new();

        for (change_amount, reason) in changes {
            player.update_loyalty(change_amount);

            records.push(LoyaltyChange {
                id: 0,
                save_id: save_id.to_string(),
                season_id,
                player_id: player.id,
                change_amount,
                reason: reason.display_name().to_string(),
                created_at: None,
            });
        }

        records
    }

    /// 计算总忠诚度变化
    pub fn sum_changes(changes: &[(i32, LoyaltyChangeReason)]) -> i32 {
        changes.iter().map(|(c, _)| *c).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_departure_threshold() {
        // 高忠诚度需要更低的满意度才会想离队
        assert_eq!(Player::departure_threshold_static(95), 20);
        assert_eq!(Player::departure_threshold_static(75), 35);
        assert_eq!(Player::departure_threshold_static(50), 50);
        assert_eq!(Player::departure_threshold_static(30), 60);
        assert_eq!(Player::departure_threshold_static(10), 70);
    }

    #[test]
    fn test_loyalty_changes() {
        let mut changes = Vec::new();

        // 赛季自然增长
        changes.push((3, LoyaltyChangeReason::SeasonPassed));

        // 青训加成
        changes.push((15, LoyaltyChangeReason::DraftOrigin));

        // 夺冠加成
        changes.push((8, LoyaltyChangeReason::TeamChampion));

        let total = LoyaltyEngine::sum_changes(&changes);
        assert_eq!(total, 26);
    }
}
