---
name: trait-system
description: 电竞经理游戏的选手特性系统。管理14种选手特性的定义、触发条件、修正计算、随机生成、互斥规则。特性影响比赛模拟中选手的ability/stability/condition。当需要修改特性效果、添加新特性、调整触发条件、修改生成规则时使用此技能。
---

# 选手特性系统 (Trait System)

## Overview

选手特性系统为每位选手赋予 0-3 个独特特性，在不同比赛情境下动态修正选手的 ability、stability、condition 属性。特性系统完全解耦于核心模拟引擎，通过 `TraitModifiers` 结构体传递修正值。

## 核心组件

### TraitType (14 种特性枚举)

**文件**: `src-tauri/src/engines/traits.rs`

```rust
pub enum TraitType {
    // 大赛表现类
    Clutch,         // 大赛型：季后赛/国际赛 condition +3
    SlowStarter,    // 慢热型：第1局 condition -2，第3+局 +2
    FastStarter,    // 快枪手：第1局 condition +2，第3+局 -1

    // 稳定性类
    Explosive,      // 爆发型：stability -15，能力上限 +5
    Consistent,     // 稳定型：stability +10，能力上限 -3

    // 心态类
    ComebackKing,   // 逆风王：落后时 condition +3
    Tilter,         // 顺风浪：领先 condition -2，落后 -3
    MentalFortress, // 心态大师：momentum ×0.5
    Fragile,        // 玻璃心：输了 momentum -2

    // 体能类
    Ironman,        // 铁人：无疲劳惩罚
    Volatile,       // 状态敏感：stability -10

    // 特殊类
    RisingStar,     // 新星：首个赛季 ability +3
    Veteran,        // 老将风范：30岁后 stability +15
    TeamLeader,     // 团队核心：队友 condition +1
}
```

### TraitEngine

**文件**: `src-tauri/src/engines/traits.rs`

```rust
pub struct TraitEngine;

impl TraitEngine {
    /// 计算单个特性的修正值
    pub fn calculate_trait_modifier(trait_type: TraitType, ctx: &TraitContext) -> TraitModifiers

    /// 计算多个特性的综合修正（含钳位）
    pub fn calculate_combined_modifiers(traits: &[TraitType], ctx: &TraitContext) -> TraitModifiers

    /// 应用修正到选手属性，返回 (ability, stability, condition, ability_ceiling)
    pub fn apply_modifiers(ability: u8, stability: u8, condition: i8, mods: &TraitModifiers) -> (u8, u8, i8, u8)

    /// 随机生成选手特性
    pub fn generate_random_traits(ability: u8, age: u8, rng: &mut impl Rng) -> Vec<TraitType>
}
```

### TraitContext (比赛情境)

```rust
pub struct TraitContext {
    pub tournament_type: String,  // 赛事类型
    pub is_playoff: bool,         // 季后赛
    pub is_international: bool,   // 国际赛
    pub game_number: u8,          // 当前第几局 (1-5)
    pub score_diff: i8,           // 比分差 (正=领先)
    pub age: u8,                  // 选手年龄
    pub is_first_season: bool,    // 首个赛季
    pub games_since_rest: u32,    // 连续比赛场次
}
```

### TraitModifiers (修正值)

```rust
pub struct TraitModifiers {
    pub ability_mod: i8,            // 能力修正 [-10, 10]
    pub stability_mod: i8,          // 稳定性修正 [-20, 20]
    pub condition_mod: i8,          // 状态修正 [-5, 5]
    pub momentum_multiplier: f64,   // momentum 系数 (默认 1.0)
    pub ability_ceiling_mod: i8,    // 能力上限修正 [-5, 10]
}
```

## 特性效果一览

| 特性 | 类型 | 稀有度 | 触发条件 | 效果 |
|------|------|--------|----------|------|
| Clutch | 正面 | 4 | 季后赛/国际赛 | condition +3 |
| SlowStarter | 双面 | 2 | BO系列赛 | 第1局 condition -2，第3+局 +2 |
| FastStarter | 双面 | 2 | BO系列赛 | 第1局 condition +2，第3+局 -1 |
| Explosive | 双面 | 3 | 始终 | stability -15，上限 +5 |
| Consistent | 双面 | 2 | 始终 | stability +10，上限 -3 |
| ComebackKing | 正面 | 4 | 比分落后 | condition +3 |
| Tilter | 负面 | 1 | 比分不平 | 领先 -2，落后 -3 |
| MentalFortress | 正面 | 4 | 始终 | momentum ×0.5 |
| Fragile | 负面 | 1 | 赛后 | momentum -2 |
| Ironman | 正面 | 3 | 始终 | 无疲劳惩罚 |
| Volatile | 负面 | 2 | 始终 | stability -10 |
| RisingStar | 正面 | 3 | 首赛季 | ability +3 |
| Veteran | 正面 | 3 | 30岁+ | stability +15 |
| TeamLeader | 正面 | 5 | 始终 | 队友 condition +1 |

## 互斥规则

以下特性不可共存：

| 特性A | 特性B |
|-------|-------|
| SlowStarter | FastStarter |
| Explosive | Consistent |
| ComebackKing | Tilter |
| Tilter | MentalFortress |
| MentalFortress | Fragile |

## 特性生成规则

根据选手能力值决定特性数量：

| 能力值范围 | 特性数量 |
|-----------|---------|
| 68-100 | 2-3 个 |
| 61-67 | 1-2 个 |
| 54-60 | 0-1 个 |
| <54 | 30% 概率 1 个 |

年龄相关特性的前置条件：
- RisingStar: 年龄 ≤ 20
- Veteran: 年龄 ≥ 28
- TeamLeader: 能力 ≥ 65

稀有度影响生成权重：`weight = 1 / rarity`，稀有度越低越常见。

## 快速模拟路径集成

### MatchPlayerInfo (比赛用选手信息)

**文件**: `src-tauri/src/engines/match_simulation.rs`

```rust
pub struct MatchPlayerInfo {
    pub ability: u8,
    pub stability: u8,
    pub condition: i8,
    pub age: u8,
    pub position: String,
    pub traits: Vec<TraitType>,
    pub is_first_season: bool,
}
```

### MatchSimContext (比赛情境)

```rust
pub struct MatchSimContext {
    pub is_playoff: bool,
    pub is_international: bool,
    pub tournament_type: String,
}
```

### simulate_match_with_traits

快速模拟路径的特性感知入口：

```rust
pub fn simulate_match_with_traits(
    &self,
    match_id: u64, tournament_id: u64, stage: &str,
    format: MatchFormat,
    home_team_id: u64, away_team_id: u64,
    home_players: &[MatchPlayerInfo],
    away_players: &[MatchPlayerInfo],
    sim_ctx: &MatchSimContext,
    meta_weights: &MetaWeights,
) -> MatchResult
```

每局比赛流程：
1. 构建 `TraitContext`（game_number、score_diff、is_playoff 等）
2. 对每位选手：特性修正 → 稳定性噪声 → 钳位
3. TeamLeader 队友加成
4. `MetaEngine::calculate_team_power_weighted` 计算队伍战力
5. 正态分布采样决定胜负

### calculate_trait_adjusted_power

```rust
fn calculate_trait_adjusted_power(
    &self,
    players: &[MatchPlayerInfo],
    game_number: u8,
    score_diff: i8,
    sim_ctx: &MatchSimContext,
    meta_weights: &MetaWeights,
    team_leader_bonus: bool,
) -> f64
```

逐选手计算流程：
```
基础属性 → TraitEngine::calculate_combined_modifiers
         → TraitEngine::apply_modifiers
         → TeamLeader 队友 condition +1
         → N(0, (100 - stability) / 10) 噪声
         → clamp [ability-15, ability_ceiling]
         → MetaEngine 位置加权 → team_power
```

## game_flow.rs 集成

`simulate_all_phase_matches` 方法的改造：

1. 调用 `load_team_players()` 预加载所有首发选手及特性
2. 调用 `MetaEngine::get_current_weights()` 获取 Meta 权重
3. 根据 `SeasonPhase` 构建 `MatchSimContext`
4. 有选手数据时调用 `simulate_match_with_traits`
5. 无选手数据时回退到 `league_service.simulate_match`

## 快速模拟中暂不生效的特性

| 特性 | 原因 |
|------|------|
| MentalFortress | 影响 momentum，需跨场次状态追踪 |
| Fragile | 影响赛后 momentum 更新 |
| Ironman | 影响疲劳，需跨场次疲劳计数 |

这些特性在详细模拟路径（`match_commands.rs`）中已生效。

## 数据库

### player_traits 表

```sql
CREATE TABLE player_traits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    trait_type TEXT NOT NULL,
    acquired_season INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(save_id, player_id, trait_type)
);
```

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/traits.rs` | 特性定义、修正计算、生成逻辑 |
| `src-tauri/src/engines/match_simulation.rs` | 快速模拟路径集成（MatchPlayerInfo、simulate_match_with_traits） |
| `src-tauri/src/services/game_flow.rs` | 时间推进集成（load_team_players、simulate_all_phase_matches） |
| `src-tauri/src/commands/match_commands.rs` | 详细模拟路径集成 |
| `src-tauri/src/engines/condition.rs` | 状态/体力系统（MatchContext） |

## 系统集成

### 与比赛模拟系统
- 特性修正 ability/stability/condition 后传入正态分布采样
- 快速模拟和详细模拟两条路径均已集成

### 与 Meta 版本系统
- 特性修正在选手个体层面，Meta 权重在位置层面，两者串联
- `calculate_trait_adjusted_power` 最终调用 `MetaEngine::calculate_team_power_weighted`

### 与选手系统
- 特性在选手创建时随机生成，存入 `player_traits` 表
- 特性数量和类型受 ability、age 影响

### 与时间推进系统
- `game_flow.rs` 的 `simulate_all_phase_matches` 预加载选手特性并传入模拟
