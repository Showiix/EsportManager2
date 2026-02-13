# 版本更新系统（Meta System）

## 概述

版本更新系统是电竞模拟的核心特征之一。在真实电竞中，每个赛季的游戏版本不同，导致各位置的重要程度发生变化——有时是"中路为王"，有时是"下路统治"。本系统通过 20 种预定义的 Meta 版本和加权战力计算公式，将这一机制引入游戏。

## 核心理念

- **版本影响战力**：不同 Meta 下，相同阵容的战力不同。中路能力 95 的队伍在"中路为王"版本下比在"辅助时代"版本下更强
- **短板惩罚**：队伍中的弱点位置会额外拖累战力（drag 效应 > carry 效应），鼓励阵容均衡
- **赛季轮换**：每赛季随机切换 Meta，制造赛季间的差异性和重玩价值
- **向后兼容**：S1 固定均衡版本，加权公式在均衡版本下等价于简单平均

## 系统架构

```
┌─────────────────────────────────────┐
│         赛季推进（新赛季开始）          │
│    services/game_flow/              │
│    advance_to_new_season()          │
└──────────────┬──────────────────────┘
               │ 调用
               ▼
┌─────────────────────────────────────┐
│        Meta 引擎                     │
│    engines/meta_engine.rs           │
│                                     │
│  ┌──────────┐  ┌──────────────────┐ │
│  │ roll_new │  │ calculate_team   │ │
│  │ _meta()  │  │ _power_weighted()│ │
│  └────┬─────┘  └───────▲──────────┘ │
│       │                │            │
└───────┼────────────────┼────────────┘
        │                │
        ▼                │ 调用
┌──────────────┐  ┌──────┴─────────────┐
│ meta_versions│  │   比赛模拟          │
│    (DB)      │  │ match_commands.rs   │
└──────────────┘  │ game_flow/          │
                  └────────────────────┘
```

## 20 种 Meta 版本

每种 Meta 定义了 5 个位置的权重，权重之和 = 5.0。

| 编号 | ID | 名称 | TOP | JUG | MID | ADC | SUP | 特点 |
|------|-----|------|------|------|------|------|------|------|
| 1 | Balanced | 均衡版本 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 各位置等权 |
| 2 | MidKingdom | 中路为王 | 0.85 | 0.90 | 1.40 | 0.95 | 0.90 | 中路权重最高 |
| 3 | BotLaneDominance | 下路统治 | 0.80 | 0.90 | 0.90 | 1.35 | 1.05 | ADC+辅助强势 |
| 4 | TopLaneCarry | 上单Carry | 1.35 | 0.90 | 0.90 | 0.90 | 0.95 | 上单主导 |
| 5 | JungleTempo | 打野节奏 | 0.85 | 1.40 | 0.90 | 0.95 | 0.90 | 打野核心 |
| 6 | SupportEra | 辅助时代 | 0.90 | 0.90 | 0.90 | 0.90 | 1.40 | 辅助影响力最大 |
| 7 | DualCarry | 双C输出 | 0.80 | 0.80 | 1.20 | 1.30 | 0.90 | 中路+ADC双核 |
| 8 | SoloLaneMeta | 单人线版本 | 1.25 | 0.80 | 1.20 | 0.85 | 0.90 | 上中单人线强势 |
| 9 | TeamfightMeta | 团战版本 | 0.90 | 0.90 | 1.10 | 1.15 | 0.95 | 团战输出位重要 |
| 10 | EarlyGameAggro | 前期进攻 | 0.90 | 1.30 | 1.00 | 0.90 | 0.90 | 打野前期节奏 |
| 11 | LateGameScaling | 后期发育 | 0.85 | 0.85 | 1.05 | 1.30 | 0.95 | ADC后期 carry |
| 12 | SplitPushMeta | 分推版本 | 1.30 | 0.90 | 1.00 | 0.85 | 0.95 | 上单分推 |
| 13 | VisionControl | 视野控制 | 0.85 | 1.10 | 0.90 | 0.85 | 1.30 | 辅助+打野视野 |
| 14 | PickComposition | 抓单阵容 | 0.90 | 1.25 | 1.10 | 0.85 | 0.90 | 打野+中路抓人 |
| 15 | ProtectTheCarry | 保护输出 | 0.80 | 0.85 | 0.90 | 1.35 | 1.10 | 全队保 ADC |
| 16 | DiveComposition | 开团版本 | 1.10 | 1.15 | 0.95 | 0.85 | 0.95 | 前排开团 |
| 17 | SkirmishMeta | 小规模团战 | 0.95 | 1.20 | 1.15 | 0.80 | 0.90 | 中野小团 |
| 18 | ObjectiveControl | 资源控制 | 0.90 | 1.25 | 0.95 | 1.00 | 0.90 | 打野控资源 |
| 19 | MidJungleSynergy | 中野联动 | 0.80 | 1.20 | 1.25 | 0.85 | 0.90 | 中野双核联动 |
| 20 | TopJungleSynergy | 上野联动 | 1.20 | 1.20 | 0.85 | 0.85 | 0.90 | 上野双核联动 |

## 战力计算公式

### 旧公式（已替代）

```
team_power = Σ(首发选手 ability) / 5
```

问题：所有位置贡献相同，无短板惩罚，无版本影响。

### 新公式

**第一步：加权均值**

```
weighted_avg = Σ(meta_weight[pos] × actual_ability) / 5.0
```

**第二步：carry/drag 效应**

```
CARRY_RATE = 0.3    // 强于均值 → 正面贡献（较小）
DRAG_RATE  = 0.5    // 弱于均值 → 负面拖累（较大）

carry_drag = Σ(
    if ability > weighted_avg → (ability - weighted_avg) × CARRY_RATE
    if ability < weighted_avg → (ability - weighted_avg) × DRAG_RATE
)
```

**第三步：最终战力**

```
team_power = weighted_avg + carry_drag
```

### 设计分析

- **DRAG_RATE > CARRY_RATE**：短板拖累 > 长板提升，鼓励阵容均衡
- **均衡版本下**：weighted_avg = simple_avg，carry_drag 效应仍然存在但权重不偏向任何位置
- **极端阵容**：一个 95 能力中单 + 四个 60 能力队友，在任何版本下都会被 carry_drag 严重惩罚

### 计算示例

队伍阵容：TOP=80, JUG=75, MID=95, ADC=85, SUP=70

**均衡版本 (Balanced)**：
- weighted_avg = (1.0×80 + 1.0×75 + 1.0×95 + 1.0×85 + 1.0×70) / 5 = 81.0
- carry_drag = (95-81)×0.3 + (85-81)×0.3 + (80-81)×0.5 + (75-81)×0.5 + (70-81)×0.5 = 4.2 + 1.2 - 0.5 - 3.0 - 5.5 = -3.6
- team_power = 81.0 + (-3.6) = 77.4

**中路为王 (MidKingdom)**：
- weighted_avg = (0.85×80 + 0.90×75 + 1.40×95 + 0.95×85 + 0.90×70) / 5 = 83.05
- carry_drag 计算基于 83.05 的偏差
- 中路 95 的超额贡献被放大，但其他位置偏低的拖累也存在

## 版本轮换规则

1. **S1 固定 Balanced**：第一赛季始终是均衡版本
2. **S2 起随机抽取**：从 20 种 Meta 中随机选择
3. **不连续重复**：连续两个赛季不能出现相同 Meta（如果随机到相同的则重新抽取）
4. **写入数据库**：每次 roll 将权重冗余存入 `meta_versions` 表

## 数据库表

```sql
CREATE TABLE meta_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    meta_type TEXT NOT NULL,        -- 'MidKingdom' 等
    meta_name TEXT NOT NULL,        -- '中路为王' 等
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

权重冗余存储的目的：即使后续修改代码中的 Meta 配置，历史存档中的赛季数据不受影响。

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/meta_engine.rs` | Meta 引擎核心（枚举、权重、roll、计算） |
| `src-tauri/src/models/meta.rs` | 数据模型（MetaInfo, MetaHistoryEntry, MetaTypeInfo） |
| `src-tauri/src/commands/meta_commands.rs` | 4 个 Tauri 命令 |
| `src-tauri/src/db/connection.rs` | 数据库迁移（meta_versions 表） |
| `src-tauri/src/services/game_flow/` | 时间系统集成点 |
| `src-tauri/src/commands/match_commands.rs` | 比赛模拟集成点 |

## Tauri 命令

| 命令 | 参数 | 功能 |
|------|------|------|
| `get_current_meta` | `save_id` | 获取当前赛季的 Meta 版本信息 |
| `get_meta_history` | `save_id` | 获取所有历史版本列表 |
| `get_all_meta_types` | 无 | 获取全部 20 种 Meta 的配置和权重 |
| `get_meta_detail` | `meta_type` | 获取指定 Meta 类型的详细信息 |
