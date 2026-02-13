//! 特性修正与情境

use super::super::condition::MatchContext;

/// 特性修正结果
#[derive(Debug, Clone, Default)]
pub struct TraitModifiers {
    /// 能力值修正
    pub ability_mod: i8,
    /// 稳定性修正
    pub stability_mod: i8,
    /// 状态修正
    pub condition_mod: i8,
    /// momentum 系数 (默认 1.0)
    pub momentum_multiplier: f64,
    /// 能力上限修正
    pub ability_ceiling_mod: i8,
}

impl TraitModifiers {
    pub fn new() -> Self {
        Self {
            ability_mod: 0,
            stability_mod: 0,
            condition_mod: 0,
            momentum_multiplier: 1.0,
            ability_ceiling_mod: 0,
        }
    }

    /// 合并多个特性修正
    pub fn merge(&mut self, other: &TraitModifiers) {
        self.ability_mod += other.ability_mod;
        self.stability_mod += other.stability_mod;
        self.condition_mod += other.condition_mod;
        self.momentum_multiplier *= other.momentum_multiplier;
        self.ability_ceiling_mod += other.ability_ceiling_mod;
    }
}

/// 比赛情境（用于特性触发判断）
#[derive(Debug, Clone, Default)]
pub struct TraitContext {
    /// 赛事类型
    pub tournament_type: String,
    /// 是否季后赛
    pub is_playoff: bool,
    /// 是否国际赛
    pub is_international: bool,
    /// 当前第几局 (1-5)
    pub game_number: u8,
    /// 当前比分差 (正数表示领先)
    pub score_diff: i8,
    /// 选手年龄
    pub age: u8,
    /// 是否首个赛季
    pub is_first_season: bool,
    /// 连续比赛场次
    pub games_since_rest: u32,
}

impl TraitContext {
    /// 从 MatchContext 转换
    pub fn from_match_context(
        ctx: &MatchContext,
        age: u8,
        is_first_season: bool,
        games_since_rest: u32,
    ) -> Self {
        let is_international = matches!(
            ctx.tournament_type.as_str(),
            "msi" | "worlds" | "masters" | "shanghai" | "clauch"
        );
        let is_playoff = ctx.round == "playoff"
            || ctx.round == "quarter"
            || ctx.round == "semi"
            || ctx.round == "final";

        Self {
            tournament_type: ctx.tournament_type.clone(),
            is_playoff,
            is_international,
            game_number: ctx.game_number,
            score_diff: ctx.score_diff,
            age,
            is_first_season,
            games_since_rest,
        }
    }
}
