//! 选手状态 (Condition) 动态计算引擎
//!
//! Condition 代表选手的近期状态/手感，范围 -10 ~ +10
//! 影响因素：
//! - 状态周期 (form_cycle): 模拟自然状态起伏的正弦波
//! - 动能 (momentum): 连胜/连败的累积效应
//! - 信心 (confidence): 基于上场发挥的心理因素
//! - 比赛压力: 大赛/决赛的紧张因素

use serde::{Deserialize, Serialize};

/// 选手状态因子 (存储在数据库中)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerFormFactors {
    pub player_id: u64,
    /// 状态周期位置 (0-100)，用于计算正弦波
    pub form_cycle: f64,
    /// 动能 (-5 ~ +5)，连胜+1，连败-1
    pub momentum: i8,
    /// 上场实际发挥值
    pub last_performance: f64,
    /// 上场是否获胜
    pub last_match_won: bool,
    /// 连续比赛场次（用于疲劳计算，可选）
    pub games_since_rest: u32,
}

impl Default for PlayerFormFactors {
    fn default() -> Self {
        Self {
            player_id: 0,
            form_cycle: 50.0, // 初始在中间位置
            momentum: 0,
            last_performance: 0.0,
            last_match_won: false,
            games_since_rest: 0,
        }
    }
}

/// 比赛情境 (用于计算情境修正)
#[derive(Debug, Clone, Default)]
pub struct MatchContext {
    /// 赛事类型: league, playoff, msi, worlds, masters
    pub tournament_type: String,
    /// 轮次: group, quarter, semi, final
    pub round: String,
    /// 第几局 (1-5)
    pub game_number: u8,
    /// 是否决胜局
    pub is_decider: bool,
    /// 当前比分差 (正数表示领先)
    pub score_diff: i8,
}

impl MatchContext {
    /// 判断是否高压场景
    pub fn is_high_pressure(&self) -> bool {
        matches!(
            self.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "claude"
        ) || self.round == "final"
            || self.is_decider
    }
}

/// Condition 计算引擎
pub struct ConditionEngine;

impl ConditionEngine {
    /// 计算选手的 condition 值
    ///
    /// # 参数
    /// - `age`: 选手年龄
    /// - `ability`: 选手基础能力值
    /// - `factors`: 状态因子
    /// - `context`: 比赛情境 (可选)
    ///
    /// # 返回
    /// condition 值，范围受年龄限制
    pub fn calculate_condition(
        age: u8,
        ability: u8,
        factors: &PlayerFormFactors,
        context: Option<&MatchContext>,
    ) -> i8 {
        // 1. 状态周期波动 (核心不可预测来源)
        //    正弦波，amplitude 由年龄决定
        let amplitude = Self::get_amplitude_by_age(age);
        let cycle_bonus = (factors.form_cycle * std::f64::consts::PI / 50.0).sin() * amplitude;

        // 2. 动能加成
        //    momentum 范围 -5 ~ +5，效果系数 0.8
        let momentum_bonus = factors.momentum as f64 * 0.8;

        // 3. 信心因子 (基于上场发挥)
        let confidence_bonus = if factors.last_performance > 0.0 {
            let diff = factors.last_performance - ability as f64;
            (diff * 0.3).clamp(-2.0, 2.0)
        } else {
            0.0
        };

        // 4. 比赛压力修正
        let pressure_penalty = if let Some(ctx) = context {
            Self::calculate_pressure_penalty(ctx)
        } else {
            0.0
        };

        // 5. 合并计算
        let raw_condition = cycle_bonus + momentum_bonus + confidence_bonus + pressure_penalty;

        // 6. 年龄范围限制
        let (min, max) = Self::get_condition_range_by_age(age);
        raw_condition.round().clamp(min as f64, max as f64) as i8
    }

    /// 根据年龄获取状态波动幅度
    fn get_amplitude_by_age(age: u8) -> f64 {
        match age {
            16..=24 => 6.0, // 年轻人波动大
            25..=29 => 4.0, // 巅峰期较稳定
            _ => 2.0,       // 老将最稳定
        }
    }

    /// 根据年龄获取 condition 范围
    /// 策划案规定：
    /// - 年轻选手 (≤24岁)：-5 ~ +8
    /// - 中生代 (25-29岁)：-3 ~ +3
    /// - 老将 (≥30岁)：0 ~ +2
    pub fn get_condition_range_by_age(age: u8) -> (i8, i8) {
        match age {
            16..=24 => (-5, 8),
            25..=29 => (-3, 3),
            _ => (0, 2),
        }
    }

    /// 计算比赛压力惩罚
    fn calculate_pressure_penalty(ctx: &MatchContext) -> f64 {
        let mut penalty = 0.0;

        // 大赛压力
        if matches!(
            ctx.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "claude"
        ) {
            penalty -= 1.5;
        }

        // 决赛额外压力
        if ctx.round == "final" {
            penalty -= 1.0;
        }

        // 决胜局压力
        if ctx.is_decider {
            penalty -= 0.5;
        }

        // 落后时额外压力
        if ctx.score_diff < 0 {
            penalty -= 0.5;
        }

        penalty
    }

    /// 比赛后更新状态因子
    ///
    /// # 参数
    /// - `factors`: 当前状态因子
    /// - `won`: 是否获胜
    /// - `performance`: 本场实际发挥值
    ///
    /// # 返回
    /// 更新后的状态因子
    pub fn update_form_factors(
        mut factors: PlayerFormFactors,
        won: bool,
        performance: f64,
    ) -> PlayerFormFactors {
        let cycle_step = 8.0 + rand::random::<f64>() * 7.0;
        factors.form_cycle = (factors.form_cycle + cycle_step) % 100.0;

        if won {
            factors.momentum = (factors.momentum + 1).min(5);
        } else {
            factors.momentum = (factors.momentum - 1).max(-5);
        }

        factors.last_performance = performance;
        factors.last_match_won = won;
        factors.games_since_rest += 1;

        factors
    }

    /// 替补（未出场）的 form_factors 更新
    /// form_cycle 慢推进 (+5~10)，momentum 向0衰减 (×0.8)，games_since_rest 重置为0
    pub fn update_form_factors_bench(mut factors: PlayerFormFactors) -> PlayerFormFactors {
        let cycle_step = 5.0 + rand::random::<f64>() * 5.0;
        factors.form_cycle = (factors.form_cycle + cycle_step) % 100.0;

        // momentum 向 0 衰减
        factors.momentum = (factors.momentum as f64 * 0.8).round() as i8;

        // 板凳休息，疲劳清零
        factors.games_since_rest = 0;

        factors
    }

    /// 重置状态因子 (新赛季开始时)
    pub fn reset_form_factors(player_id: u64) -> PlayerFormFactors {
        PlayerFormFactors {
            player_id,
            form_cycle: rand::random::<f64>() * 100.0, // 随机初始位置
            momentum: 0,
            last_performance: 0.0,
            last_match_won: false,
            games_since_rest: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_range_by_age() {
        assert_eq!(ConditionEngine::get_condition_range_by_age(18), (-5, 8));
        assert_eq!(ConditionEngine::get_condition_range_by_age(24), (-5, 8));
        assert_eq!(ConditionEngine::get_condition_range_by_age(25), (-3, 3));
        assert_eq!(ConditionEngine::get_condition_range_by_age(29), (-3, 3));
        assert_eq!(ConditionEngine::get_condition_range_by_age(30), (0, 2));
        assert_eq!(ConditionEngine::get_condition_range_by_age(35), (0, 2));
    }

    #[test]
    fn test_calculate_condition_default() {
        let factors = PlayerFormFactors::default();
        let condition = ConditionEngine::calculate_condition(25, 80, &factors, None);
        // 默认因子应该产生接近0的condition
        assert!(condition >= -3 && condition <= 3);
    }

    #[test]
    fn test_calculate_condition_with_momentum() {
        let mut factors = PlayerFormFactors::default();
        factors.momentum = 5; // 5连胜
        let condition = ConditionEngine::calculate_condition(25, 80, &factors, None);
        // 高动能应该产生正向condition
        assert!(condition > 0);
    }

    #[test]
    fn test_calculate_condition_young_player() {
        let mut factors = PlayerFormFactors::default();
        factors.form_cycle = 75.0; // 接近波峰
        let condition = ConditionEngine::calculate_condition(20, 80, &factors, None);
        // 年轻选手波动大，可能有高condition
        assert!(condition >= -5 && condition <= 8);
    }

    #[test]
    fn test_update_form_factors_win() {
        let factors = PlayerFormFactors::default();
        let updated = ConditionEngine::update_form_factors(factors, true, 85.0);
        assert_eq!(updated.momentum, 1);
        assert!(updated.last_match_won);
        assert_eq!(updated.last_performance, 85.0);
    }

    #[test]
    fn test_update_form_factors_lose() {
        let factors = PlayerFormFactors::default();
        let updated = ConditionEngine::update_form_factors(factors, false, 70.0);
        assert_eq!(updated.momentum, -1);
        assert!(!updated.last_match_won);
    }

    #[test]
    fn test_momentum_bounds() {
        let mut factors = PlayerFormFactors::default();
        factors.momentum = 5;
        let updated = ConditionEngine::update_form_factors(factors, true, 85.0);
        assert_eq!(updated.momentum, 5); // 不超过5

        let mut factors2 = PlayerFormFactors::default();
        factors2.momentum = -5;
        let updated2 = ConditionEngine::update_form_factors(factors2, false, 70.0);
        assert_eq!(updated2.momentum, -5); // 不低于-5
    }

    #[test]
    fn test_update_form_factors_bench() {
        let mut factors = PlayerFormFactors {
            player_id: 1,
            form_cycle: 30.0,
            momentum: 4,
            last_performance: 80.0,
            last_match_won: true,
            games_since_rest: 5,
        };
        let updated = ConditionEngine::update_form_factors_bench(factors.clone());
        // form_cycle advances by 5~10
        assert!(updated.form_cycle >= 35.0 || updated.form_cycle < 5.0); // handles wrap at 100
                                                                         // momentum decays toward 0: 4 * 0.8 = 3.2 → round to 3
        assert_eq!(updated.momentum, 3);
        // rest resets
        assert_eq!(updated.games_since_rest, 0);
        // last_performance and last_match_won unchanged
        assert_eq!(updated.last_performance, 80.0);
        assert!(updated.last_match_won);

        // negative momentum decay
        factors.momentum = -3;
        let updated2 = ConditionEngine::update_form_factors_bench(factors);
        // -3 * 0.8 = -2.4 → round to -2
        assert_eq!(updated2.momentum, -2);
    }
}
