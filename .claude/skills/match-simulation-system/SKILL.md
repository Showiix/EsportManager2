---
name: match-simulation-system
description: 电竞经理游戏的比赛模拟系统。基于正态分布的战力值比赛结果模拟、BO系列赛逻辑、胜率计算。当需要修改比赛模拟算法、战力影响、赛制逻辑时使用此技能。
---

# 比赛模拟系统 (Match Simulation System)

## Overview

比赛模拟系统使用基于正态分布的概率模型来模拟电竞比赛结果。它支持BO1/BO3/BO5多种赛制，并能计算两队对阵的预测胜率。

## 核心组件

### MatchSimulationEngine

**文件**: `src-tauri/src/engines/match_simulation.rs`

比赛模拟引擎，基于正态分布实现胜负判定。

```rust
pub struct MatchSimulationEngine {
    std_dev: f64,  // 标准差 (控制发挥波动程度)，默认 6.0
}
```

**核心方法**:
- `simulate_game(...)` - 模拟单局比赛
- `simulate_match(...)` - 模拟BO系列赛
- `calculate_win_probability(...)` - 计算胜率

## 核心算法

### 正态分布模拟

每局比赛中，双方的实际表现基于战力值的正态分布采样：

```rust
fn simulate_game(&self, home_power: f64, away_power: f64) -> (f64, f64, u64) {
    // 双方分别从 N(power, std_dev²) 分布采样
    let home_performance = Normal::new(home_power, self.std_dev).sample(&mut rng);
    let away_performance = Normal::new(away_power, self.std_dev).sample(&mut rng);

    // 表现值高的一方获胜
    let winner_id = if home_performance > away_performance {
        home_team_id
    } else {
        away_team_id
    };

    (home_performance, away_performance, winner_id)
}
```

### 胜率计算

基于正态分布CDF计算理论胜率：

```rust
fn calculate_win_probability(&self, team_power: f64, opponent_power: f64) -> f64 {
    let diff = team_power - opponent_power;
    let z = diff / (self.std_dev * SQRT_2);
    0.5 * (1.0 + erf(z))  // 误差函数近似
}
```

## 数据结构

### MatchFormat (赛制)

```rust
pub enum MatchFormat {
    Bo1,  // 单局制
    Bo3,  // 三局两胜
    Bo5,  // 五局三胜
}

impl MatchFormat {
    pub fn wins_needed(&self) -> u8 {
        match self {
            Bo1 => 1,
            Bo3 => 2,
            Bo5 => 3,
        }
    }
}
```

### Match (比赛)

```rust
pub struct Match {
    pub id: u64,
    pub tournament_id: u64,
    pub stage: String,
    pub round: Option<u32>,
    pub match_order: Option<u32>,
    pub format: MatchFormat,
    pub home_team_id: u64,
    pub away_team_id: u64,
    pub home_score: u8,
    pub away_score: u8,
    pub winner_id: Option<u64>,
    pub status: MatchStatus,
}
```

### MatchGame (单局)

```rust
pub struct MatchGame {
    pub id: u64,
    pub match_id: u64,
    pub game_number: u8,
    pub home_power: f64,
    pub away_power: f64,
    pub home_performance: f64,  // 主队实际表现
    pub away_performance: f64,  // 客队实际表现
    pub winner_id: u64,
    pub duration_minutes: Option<u32>,  // 比赛时长 (30-50分钟)
}
```

### MatchResult (比赛结果)

```rust
pub struct MatchResult {
    pub match_info: Match,
    pub games: Vec<MatchGame>,
    pub winner_id: u64,
    pub home_score: u8,
    pub away_score: u8,
}
```

### MatchStatus (比赛状态)

```rust
pub enum MatchStatus {
    Pending,     // 待进行
    Completed,   // 已完成
    Cancelled,   // 已取消
}
```

## 战力值影响

### 胜率对照表 (std_dev = 6.0)

| 战力差 | 强队胜率 |
|--------|----------|
| 0 | 50% |
| 5 | ~60% |
| 10 | ~69% |
| 15 | ~76% |
| 20 | ~82% |
| 25 | ~87% |
| 30 | ~91% |

### 标准差调整

- **低标准差 (3.0)**: 强队优势更明显，冷门更少
- **高标准差 (10.0)**: 随机性更大，冷门更多

## Tauri 命令接口

**文件**: `src-tauri/src/commands/match_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `simulate_match_detailed` | 模拟比赛（详细结果） | `MatchResult` |
| `get_match_prediction` | 获取比赛预测 | `MatchPrediction` |
| `simulate_next_match` | 模拟下一场比赛 | `SimulateNextResult` |
| `simulate_all_matches` | 模拟所有比赛 | `u32` (数量) |
| `update_match_result` | 更新比赛结果 | `bool` |
| `cancel_match` | 取消比赛 | `bool` |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 模拟详细比赛
export async function simulateMatchDetailed(matchId: number): Promise<MatchResult>

// 获取比赛预测
export async function getMatchPrediction(homeTeamId: number, awayTeamId: number): Promise<MatchPrediction>

// 模拟下一场
export async function simulateNextMatch(): Promise<SimulateNextResult>

// 模拟所有
export async function simulateAllMatches(): Promise<number>
```

## 系统集成

### 与时间推进系统
- `time_simulate_next` / `time_simulate_all` 调用模拟引擎
- 阶段完成条件依赖比赛模拟结果

### 与数据中心系统
- 每局 `home_performance` / `away_performance` 记录为影响力
- 选手表现数据用于MVP计算

### 与选手系统
- 队伍战力值 = Meta 加权平均 + carry/drag 效应（详见 `meta-system` 技能）
- 在均衡版本（Balanced）下等价于简单平均 Σ(ability) / 5
- 选手 `stability` 通过 `traits.rs` / `condition.rs` 影响发挥波动

### 与版本系统 (Meta System)
- 每赛季的 Meta 决定各位置对战力的贡献权重
- `simulate_match_detailed()` 在模拟前获取当前 Meta 权重传入
- `generate_team_stats()` 使用 `MetaEngine::calculate_team_power_weighted()` 替代简单平均
- **文件**: `src-tauri/src/engines/meta_engine.rs`

### 与荣誉系统
- 模拟结束后触发冠军/MVP记录

## 使用示例

### 模拟单场BO5
```rust
let engine = MatchSimulationEngine::default();

let result = engine.simulate_match(
    match_id,
    tournament_id,
    "FINAL",
    MatchFormat::Bo5,
    t1_id, gen_id,
    92.0, 88.0,  // 战力值
);

println!("比分: {}:{}", result.home_score, result.away_score);
println!("胜者: {}", result.winner_id);
for game in &result.games {
    println!("第{}局: {:.1} vs {:.1} -> 胜者{}",
        game.game_number,
        game.home_performance,
        game.away_performance,
        game.winner_id);
}
```

### 计算胜率
```rust
let engine = MatchSimulationEngine::default();
let win_prob = engine.calculate_win_probability(92.0, 88.0);
println!("T1 胜率: {:.1}%", win_prob * 100.0);
// 输出: T1 胜率: 64.0%
```

### 自定义波动性
```rust
// 低波动 - 强队更稳定获胜
let stable_engine = MatchSimulationEngine::new(3.0);

// 高波动 - 更多冷门可能
let volatile_engine = MatchSimulationEngine::new(10.0);
```

## 比赛时长

每局比赛时长随机生成：
```rust
duration_minutes = 30 + rand::random::<u32>() % 20  // 30-50分钟
```

## 注意事项

1. **战力值来源**: 队伍战力 = Meta 加权平均 + carry/drag 效应（均衡版本下等价于简单平均）
2. **结果一致性**: `MatchResult` 中的 `match_info`、`games`、分数应保持一致
3. **随机性**: 每次模拟结果不同，无法复现
4. **表现值意义**: `performance` 值代表该局实际发挥，可用于数据中心记录
5. **Meta 权重**: 比赛模拟依赖当前赛季 Meta 权重，通过 `MetaEngine::get_current_weights()` 获取
