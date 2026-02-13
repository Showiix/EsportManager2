# 比赛模拟系统

## 概述

比赛模拟系统是电竞经理游戏的核心算法，基于选手真实属性，通过多层正态分布模型模拟比赛结果。既保证强队的优势，又引入合理的不确定性，使比赛结果更加真实有趣。

## 核心理念

- **选手为本**: 比赛结果由选手的真实属性决定，而非简单的队伍战力值
- **双层随机**: 选手发挥层 + 胜负判定层，两次正态分布叠加
- **可控波动**: 通过稳定性属性控制选手发挥的波动幅度
- **爆冷可能**: 即使弱队也有机会战胜强队，但概率符合实际

## 算法架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        比赛模拟流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐    ┌──────────────┐                          │
│   │   选手A属性   │    │   选手B属性   │  ... (共10名选手)        │
│   │ ability: 85  │    │ ability: 78  │                          │
│   │ stability:70 │    │ stability:60 │                          │
│   │ condition: 3 │    │ condition:-2 │                          │
│   └──────┬───────┘    └──────┬───────┘                          │
│          │                   │                                   │
│          ▼                   ▼                                   │
│   ┌──────────────────────────────────────┐                      │
│   │     第一层：选手发挥值计算（正态分布）   │                      │
│   │                                       │                      │
│   │  actual = ability + condition + ε     │                      │
│   │  ε ~ N(0, σ), σ = (100-stability)/10  │                      │
│   └──────────────────┬───────────────────┘                      │
│                      │                                           │
│                      ▼                                           │
│   ┌──────────────────────────────────────┐                      │
│   │     第二层：队伍发挥战力计算            │                      │
│   │                                       │                      │
│   │  team_perf = Σ(actual_i) / 5          │                      │
│   └──────────────────┬───────────────────┘                      │
│                      │                                           │
│                      ▼                                           │
│   ┌──────────────────────────────────────┐                      │
│   │     第三层：胜负判定（正态分布）        │                      │
│   │                                       │                      │
│   │  diff = home_perf - away_perf         │                      │
│   │  final = diff + ε, ε ~ N(0, 3)        │                      │
│   │  winner = final > 0 ? home : away     │                      │
│   └──────────────────────────────────────┘                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 详细算法

### 第一层：选手发挥值计算

每个选手在每局比赛中的实际发挥值：

```
actual_ability = ability + condition + stability_noise
```

#### 参数说明

| 参数 | 说明 | 范围 |
|------|------|------|
| `ability` | 选手基础能力值 | 0-100 |
| `condition` | 当前状态加成 | -10 ~ +10 |
| `stability_noise` | 稳定性波动（高斯噪声） | 动态计算 |

#### 稳定性波动计算

```
σ = (100 - stability) / 10
stability_noise ~ N(0, σ)
```

| stability | σ (标准差) | 波动范围 (约95%概率) | 说明 |
|-----------|-----------|---------------------|------|
| 90 | 1.0 | ±2 | 非常稳定 |
| 80 | 2.0 | ±4 | 稳定型选手 |
| 70 | 3.0 | ±6 | 普通选手 |
| 60 | 4.0 | ±8 | 较不稳定 |
| 50 | 5.0 | ±10 | 高波动选手 |

#### 发挥值钳位

为防止极端情况，发挥值被限制在合理范围内：

```
min_ability = max(ability - 15, 0)
max_ability = min(ability + 10, 100)
actual_ability = clamp(raw_ability, min_ability, max_ability)
```

### 第二层：队伍发挥战力计算

队伍本局的发挥战力为所有首发选手发挥值的平均：

```
team_performance = (Σ actual_ability_i) / 5
```

### 第三层：胜负判定

基于双方队伍发挥战力，通过正态分布增加局内不确定性：

```
performance_diff = home_perf - away_perf
game_noise ~ N(0, 3)
final_diff = performance_diff + game_noise

winner = final_diff > 0 ? home_team : away_team
```

#### 理论胜率

| 发挥战力差 | 强队胜率 | 说明 |
|-----------|---------|------|
| 0 | 50.0% | 势均力敌 |
| 1 | 63.1% | 微弱优势 |
| 2 | 74.8% | 明显优势 |
| 3 | 84.1% | 较大优势 |
| 5 | 95.2% | 压倒性优势 |
| 7 | 99.0% | 几乎必胜 |

## 比赛格式

```rust
pub enum MatchFormat {
    Bo1,  // 一局定胜负
    Bo3,  // 三局两胜
    Bo5,  // 五局三胜
}
```

## 影响力分数

每个选手的影响力分数反映其发挥相对于队伍平均水平的偏差：

```
impact_score = actual_ability - team_average
```

- **正值**: 选手发挥超出队伍平均，是本局的 Carry
- **负值**: 选手发挥低于队伍平均，拖累了队伍

## 数据结构

### MatchResult

```rust
pub struct MatchResult {
    pub match_info: Match,
    pub games: Vec<MatchGame>,
    pub winner_id: u64,
    pub home_score: u8,
    pub away_score: u8,
}
```

### MatchGame

```rust
pub struct MatchGame {
    pub id: u64,
    pub match_id: u64,
    pub game_number: u8,
    pub home_power: f64,        // 主队战力
    pub away_power: f64,        // 客队战力
    pub home_performance: f64,  // 主队发挥值
    pub away_performance: f64,  // 客队发挥值
    pub winner_id: u64,
    pub duration_minutes: Option<u32>,
}
```

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/match_simulation.rs` | 核心模拟引擎 |
| `src-tauri/src/engines/traits.rs` | 选手特性系统 |
| `src-tauri/src/commands/match_commands.rs` | 比赛命令接口 |
| `src-tauri/src/services/game_flow/match_simulation.rs` | 时间推进集成 |
| `src-tauri/src/services/league_service.rs` | 联赛服务 |
| `src/engines/PlayerEngine.ts` | 前端选手计算 |
| `src/engines/PowerEngine.ts` | 前端战力计算 |

## 特性系统集成

比赛模拟引擎支持两种模拟路径：

### 纯战力值路径（回退）

```rust
engine.simulate_match(match_id, tournament_id, stage, format,
    home_team_id, away_team_id, home_power, away_power)
```

仅使用两个 `f64` 战力值做正态分布采样，无特性修正。

### 特性感知路径（默认）

```rust
engine.simulate_match_with_traits(match_id, tournament_id, stage, format,
    home_team_id, away_team_id, home_players, away_players, sim_ctx, meta_weights)
```

每局比赛逐选手计算特性修正后再做正态分布采样。数据流：

```
选手基础属性 → 特性修正 → 稳定性噪声 → 实际能力 → Meta 位置加权 → 队伍战力 → 正态分布采样 → 胜负
```

时间推进（`game_flow.rs`）默认使用特性感知路径，无选手数据时自动回退。

详见 [特性系统文档](trait-system.md)。

## Condition 动态计算系统

### 概述

Condition 代表选手的近期状态/手感（-10 ~ +10），由 `ConditionEngine` 基于多因子动态计算，而非静态存储在 `players` 表中。快速模拟路径和详细模拟路径均使用动态 condition。

### 影响因子

| 因子 | 说明 | 权重/范围 |
|------|------|-----------|
| 状态周期 (form_cycle) | 正弦波模拟自然起伏，0-100 映射到 sin 曲线 | 幅度由年龄决定：年轻 6.0、巅峰 4.0、老将 2.0 |
| 动能 (momentum) | 连胜 +1、连败 -1，累积 -5 ~ +5 | 效果系数 0.8 |
| 信心 (confidence) | 基于上场实际发挥与 ability 的差值 | ±2.0 上限 |
| 比赛压力 | 大赛/决赛/决胜局/落后时的额外负面修正 | 最大 -3.5 |

### Condition 范围限制（按年龄）

| 年龄段 | 范围 | 说明 |
|--------|------|------|
| 16-24 岁 | -5 ~ +8 | 年轻选手波动大，上限高 |
| 25-29 岁 | -3 ~ +3 | 巅峰期稳定 |
| 30+ 岁 | 0 ~ +2 | 老将最稳定 |

### 快速模拟路径集成

快速模拟（`simulate_all_phase_matches()`）采用 **内存缓存 + 阶段结束批量落盘** 策略：

1. **阶段开始**：`load_team_players()` 一次性 LEFT JOIN `player_form_factors` 表，动态计算所有首发选手的 condition，构建 `HashMap<u64, PlayerFormFactors>` 内存缓存
2. **每场比赛后**：`update_form_factors_after_match()` 在内存中更新 form factors（momentum、form_cycle 等）并重算 condition
3. **阶段结束**：`flush_form_factors_to_db()` 一个事务批量 UPSERT 写回数据库

### 新赛季重置

`advance_to_new_season()` 中批量重置所有 form factors：
- momentum = 0
- last_performance = 0.0
- last_match_won = 0
- games_since_rest = 0
- form_cycle = 随机值 (0-100)

### 数据存储

```sql
-- player_form_factors 表
CREATE TABLE player_form_factors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    form_cycle REAL NOT NULL DEFAULT 50.0,
    momentum INTEGER NOT NULL DEFAULT 0,
    last_performance REAL NOT NULL DEFAULT 0.0,
    last_match_won INTEGER NOT NULL DEFAULT 0,
    games_since_rest INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(save_id, player_id)
);
```

### 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/condition.rs` | ConditionEngine 核心算法 |
| `src-tauri/src/services/game_flow/match_simulation.rs` | 快速模拟路径集成（load_team_players / flush_form_factors_to_db） |
| `src-tauri/src/commands/match_commands.rs` | 详细模拟路径 form factors 更新 |
| `src-tauri/src/commands/game_commands.rs` | 单场模拟路径 form factors 更新 |
