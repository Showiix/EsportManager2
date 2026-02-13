---
name: meta-system
description: 电竞经理游戏的版本更新（Meta）系统。管理20种Meta版本、位置权重、加权战力计算、赛季版本轮换。当需要修改Meta配置、位置权重、战力计算公式、版本轮换规则时使用此技能。
---

# 版本更新系统 (Meta System)

## Overview

版本更新系统模拟电竞游戏中的"版本"（Meta）概念。每个赛季拥有不同的版本，版本决定了各位置（TOP/JUG/MID/ADC/SUP）对队伍战力的贡献权重。系统包含 20 种预定义 Meta，每赛季随机切换，直接影响队伍战力计算和比赛模拟结果。

## 核心组件

### MetaEngine

**文件**: `src-tauri/src/engines/meta_engine.rs`

Meta 引擎核心，负责版本轮换和加权战力计算。

```rust
pub struct MetaEngine;

impl MetaEngine {
    /// 为新赛季随机生成 Meta，写入数据库
    pub async fn roll_new_meta(pool: &SqlitePool, save_id: &str, season: i64) -> Result<(), String>

    /// 获取当前赛季的 Meta 权重
    pub async fn get_current_weights(pool: &SqlitePool, save_id: &str, season: i64) -> MetaWeights

    /// 使用 Meta 权重计算队伍战力（加权平均 + carry/drag 效应）
    pub fn calculate_team_power_weighted(performances: &[(f64, &str)], weights: &MetaWeights) -> f64
}
```

### MetaType (20 种版本枚举)

```rust
pub enum MetaType {
    Balanced,           // 均衡版本
    MidKingdom,         // 中路为王
    BotLaneDominance,   // 下路统治
    TopLaneCarry,       // 上单Carry
    JungleTempo,        // 打野节奏
    SupportEra,         // 辅助时代
    DualCarry,          // 双C输出
    SoloLaneMeta,       // 单人线版本
    TeamfightMeta,      // 团战版本
    EarlyGameAggro,     // 前期进攻
    LateGameScaling,    // 后期发育
    SplitPushMeta,      // 分推版本
    VisionControl,      // 视野控制
    PickComposition,    // 抓单阵容
    ProtectTheCarry,    // 保护输出
    DiveComposition,    // 开团版本
    SkirmishMeta,       // 小规模团战
    ObjectiveControl,   // 资源控制
    MidJungleSynergy,   // 中野联动
    TopJungleSynergy,   // 上野联动
}
```

### MetaWeights (位置权重)

```rust
pub struct MetaWeights {
    pub top: f64,
    pub jug: f64,
    pub mid: f64,
    pub adc: f64,
    pub sup: f64,
}
```

每种 Meta 的 5 个位置权重之和 = 5.0（归一化）。

## 核心算法

### 加权战力计算

替代原有的简单算术平均（5 名选手 ability / 5），引入三步计算：

```rust
// 第1步：加权均值
let weighted_avg = Σ(meta_weight[pos] × actual_ability) / 5.0;

// 第2步：carry/drag 效应
const CARRY_RATE: f64 = 0.3;  // 强于均值 → 正面贡献
const DRAG_RATE: f64 = 0.5;   // 弱于均值 → 负面拖累

let carry_drag = Σ(
    if ability > weighted_avg: (ability - weighted_avg) × CARRY_RATE
    if ability < weighted_avg: (ability - weighted_avg) × DRAG_RATE
);

// 第3步：最终战力
let team_power = weighted_avg + carry_drag;
```

**设计意图**:
- 在 Balanced 版本下（所有权重 = 1.0），加权均值 = 简单均值，向后兼容
- carry_drag 惩罚短板（DRAG_RATE > CARRY_RATE），鼓励阵容均衡
- Meta 权重放大/缩小特定位置的贡献，使版本对阵容策略产生影响

### 版本轮换规则

- S1 固定为 `Balanced`（均衡版本）
- S2 起每赛季随机抽取，不连续重复（不会连续两年相同版本）
- 权重冗余存储到数据库，即使后续调整 Meta 配置也不影响历史数据

## 权重配置表

| 编号 | ID | 中文名称 | TOP | JUG | MID | ADC | SUP |
|------|-----|---------|------|------|------|------|------|
| 1 | Balanced | 均衡版本 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 2 | MidKingdom | 中路为王 | 0.85 | 0.90 | 1.40 | 0.95 | 0.90 |
| 3 | BotLaneDominance | 下路统治 | 0.80 | 0.90 | 0.90 | 1.35 | 1.05 |
| 4 | TopLaneCarry | 上单Carry | 1.35 | 0.90 | 0.90 | 0.90 | 0.95 |
| 5 | JungleTempo | 打野节奏 | 0.85 | 1.40 | 0.90 | 0.95 | 0.90 |
| 6 | SupportEra | 辅助时代 | 0.90 | 0.90 | 0.90 | 0.90 | 1.40 |
| 7 | DualCarry | 双C输出 | 0.80 | 0.80 | 1.20 | 1.30 | 0.90 |
| 8 | SoloLaneMeta | 单人线版本 | 1.25 | 0.80 | 1.20 | 0.85 | 0.90 |
| 9 | TeamfightMeta | 团战版本 | 0.90 | 0.90 | 1.10 | 1.15 | 0.95 |
| 10 | EarlyGameAggro | 前期进攻 | 0.90 | 1.30 | 1.00 | 0.90 | 0.90 |
| 11 | LateGameScaling | 后期发育 | 0.85 | 0.85 | 1.05 | 1.30 | 0.95 |
| 12 | SplitPushMeta | 分推版本 | 1.30 | 0.90 | 1.00 | 0.85 | 0.95 |
| 13 | VisionControl | 视野控制 | 0.85 | 1.10 | 0.90 | 0.85 | 1.30 |
| 14 | PickComposition | 抓单阵容 | 0.90 | 1.25 | 1.10 | 0.85 | 0.90 |
| 15 | ProtectTheCarry | 保护输出 | 0.80 | 0.85 | 0.90 | 1.35 | 1.10 |
| 16 | DiveComposition | 开团版本 | 1.10 | 1.15 | 0.95 | 0.85 | 0.95 |
| 17 | SkirmishMeta | 小规模团战 | 0.95 | 1.20 | 1.15 | 0.80 | 0.90 |
| 18 | ObjectiveControl | 资源控制 | 0.90 | 1.25 | 0.95 | 1.00 | 0.90 |
| 19 | MidJungleSynergy | 中野联动 | 0.80 | 1.20 | 1.25 | 0.85 | 0.90 |
| 20 | TopJungleSynergy | 上野联动 | 1.20 | 1.20 | 0.85 | 0.85 | 0.90 |

## 数据库设计

```sql
CREATE TABLE meta_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    meta_type TEXT NOT NULL,
    meta_name TEXT NOT NULL,
    weight_top REAL NOT NULL,
    weight_jug REAL NOT NULL,
    weight_mid REAL NOT NULL,
    weight_adc REAL NOT NULL,
    weight_sup REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_meta_versions_save_season ON meta_versions(save_id, season_id);
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/meta_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `get_current_meta(save_id)` | 获取当前赛季版本信息 | `MetaInfo` |
| `get_meta_history(save_id)` | 获取历史版本列表 | `Vec<MetaHistoryEntry>` |
| `get_all_meta_types()` | 获取全部 20 种 Meta 配置 | `Vec<MetaTypeInfo>` |
| `get_meta_detail(meta_type)` | 获取指定 Meta 详细信息 | `MetaTypeInfo` |

## 数据模型

**文件**: `src-tauri/src/models/meta.rs`

```rust
pub struct MetaInfo {
    pub season_id: i64,
    pub meta_type: String,
    pub meta_name: String,
    pub description: String,
    pub weights: MetaWeightsInfo,
}

pub struct MetaHistoryEntry {
    pub season_id: i64,
    pub meta_type: String,
    pub meta_name: String,
    pub weight_top: f64,
    pub weight_jug: f64,
    pub weight_mid: f64,
    pub weight_adc: f64,
    pub weight_sup: f64,
}

pub struct MetaTypeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weights: MetaWeightsInfo,
}
```

## 系统集成

### 与时间推进系统
- `advance_to_new_season()` 在重置年度积分后、确认首发前，调用 `MetaEngine::roll_new_meta()` 生成新赛季版本
- **文件**: `src-tauri/src/services/game_flow/match_simulation.rs`

### 与比赛模拟系统
- `simulate_match_detailed()` 在模拟前获取当前 Meta 权重
- `simulate_game_with_players()` 和 `generate_team_stats()` 使用加权计算替代简单平均
- **文件**: `src-tauri/src/commands/match_commands.rs`

### 与战力计算
- `recalculate_team_powers()` 查询首发选手 + 位置，调用 `calculate_team_power_weighted()` 计算加权战力
- **文件**: `src-tauri/src/services/game_flow/match_simulation.rs`

## 使用示例

### 获取 Meta 权重
```rust
use crate::engines::meta_engine::{MetaType, get_meta_weights};

let weights = get_meta_weights(MetaType::MidKingdom);
// weights.mid = 1.40 (中路权重最高)
```

### 计算加权战力
```rust
use crate::engines::MetaEngine;

let performances = vec![
    (80.0, "Top"),
    (75.0, "Jungle"),
    (95.0, "Mid"),
    (85.0, "Adc"),
    (70.0, "Support"),
];
let weights = get_meta_weights(MetaType::MidKingdom);
let power = MetaEngine::calculate_team_power_weighted(&performances, &weights);
// 中路为王版本下，95 能力的中单贡献更大
```

### 生成新赛季 Meta
```rust
MetaEngine::roll_new_meta(&pool, &save_id, new_season).await?;
```

## 注意事项

1. **权重归一化**: 每种 Meta 的 5 个权重之和必须 = 5.0，否则会破坏战力数值平衡
2. **S1 固定 Balanced**: 第一赛季不 roll，保持均衡版本，确保初始体验一致
3. **不连续重复**: 版本轮换保证连续两个赛季不会出现相同 Meta
4. **向后兼容**: Balanced 版本下加权计算等价于简单平均，不影响原有平衡
5. **历史数据独立**: 权重冗余存入数据库，即使修改代码中的 Meta 配置也不影响已有存档的历史记录
6. **carry/drag 不对称**: DRAG_RATE (0.5) > CARRY_RATE (0.3)，短板对队伍的拖累大于强点的提升
