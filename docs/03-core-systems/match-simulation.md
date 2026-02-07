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
| `src-tauri/src/commands/match_commands.rs` | 比赛命令接口 |
| `src-tauri/src/services/league_service.rs` | 联赛服务 |
| `src/engines/PlayerEngine.ts` | 前端选手计算 |
| `src/engines/PowerEngine.ts` | 前端战力计算 |
