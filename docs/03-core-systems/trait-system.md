# 选手特性系统

## 概述

选手特性系统为电竞经理游戏中的选手赋予个性化能力，通过在不同比赛情境下动态修正选手的核心属性（ability、stability、condition），使比赛结果更具多样性和策略深度。

## 核心理念

- **情境触发**: 特性不是全局生效，而是根据具体比赛情境（季后赛、比分、局数等）决定是否触发
- **属性修正**: 通过修正 ability/stability/condition 三个维度影响选手表现
- **解耦设计**: 特性系统独立于核心模拟引擎，通过 `TraitModifiers` 传递修正值
- **双路径覆盖**: 快速模拟（时间推进）和详细模拟（手动触发）两条路径均已集成

## 算法架构

```
┌─────────────────────────────────────────────────────────────┐
│                    特性修正流程（每局每位选手）                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌──────────────┐    ┌──────────────┐                      │
│   │  选手基础属性  │    │  比赛情境     │                      │
│   │ ability: 80  │    │ is_playoff   │                      │
│   │ stability:70 │    │ game_number  │                      │
│   │ condition: 2 │    │ score_diff   │                      │
│   │ traits: [..] │    │ age, season  │                      │
│   └──────┬───────┘    └──────┬───────┘                      │
│          │                   │                               │
│          ▼                   ▼                               │
│   ┌──────────────────────────────────────┐                  │
│   │  TraitEngine::calculate_combined_    │                  │
│   │  modifiers(traits, context)          │                  │
│   │  → ability_mod, stability_mod,       │                  │
│   │    condition_mod, ceiling_mod        │                  │
│   └──────────────────┬───────────────────┘                  │
│                      │                                       │
│                      ▼                                       │
│   ┌──────────────────────────────────────┐                  │
│   │  TraitEngine::apply_modifiers(...)   │                  │
│   │  → modified_ability     (1-100)      │                  │
│   │  → modified_stability   (30-100)     │                  │
│   │  → modified_condition   (-10~10)     │                  │
│   │  → ability_ceiling      (ability~100)│                  │
│   └──────────────────┬───────────────────┘                  │
│                      │                                       │
│                      ▼                                       │
│   ┌──────────────────────────────────────┐                  │
│   │  TeamLeader 加成                      │                  │
│   │  队内有 TeamLeader → 其余队友         │                  │
│   │  condition +1                         │                  │
│   └──────────────────┬───────────────────┘                  │
│                      │                                       │
│                      ▼                                       │
│   ┌──────────────────────────────────────┐                  │
│   │  稳定性噪声                           │                  │
│   │  σ = (100 - modified_stability) / 10 │                  │
│   │  noise ~ N(0, σ)                     │                  │
│   └──────────────────┬───────────────────┘                  │
│                      │                                       │
│                      ▼                                       │
│   ┌──────────────────────────────────────┐                  │
│   │  实际能力 = ability + condition +     │                  │
│   │  noise                               │                  │
│   │  clamp [ability-15, ability_ceiling]  │                  │
│   └──────────────────┬───────────────────┘                  │
│                      │                                       │
│                      ▼                                       │
│   ┌──────────────────────────────────────┐                  │
│   │  Meta 位置加权 → 队伍战力             │                  │
│   │  MetaEngine::calculate_team_power_   │                  │
│   │  weighted(players, meta_weights)     │                  │
│   └──────────────────────────────────────┘                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 14 种特性详解

### 大赛表现类

| 特性 | 中文名 | 稀有度 | 效果 | 触发条件 |
|------|--------|--------|------|----------|
| Clutch | 大赛型 | 4 | condition +3 | 季后赛 或 国际赛 |
| SlowStarter | 慢热型 | 2 | 第1局 condition -2，第3+局 +2 | BO3/BO5 系列赛 |
| FastStarter | 快枪手 | 2 | 第1局 condition +2，第3+局 -1 | BO3/BO5 系列赛 |

### 稳定性类

| 特性 | 中文名 | 稀有度 | 效果 | 触发条件 |
|------|--------|--------|------|----------|
| Explosive | 爆发型 | 3 | stability -15，能力上限 +5 | 始终 |
| Consistent | 稳定型 | 2 | stability +10，能力上限 -3 | 始终 |

### 心态类

| 特性 | 中文名 | 稀有度 | 效果 | 触发条件 |
|------|--------|--------|------|----------|
| ComebackKing | 逆风王 | 4 | condition +3 | 当前比分落后 |
| Tilter | 顺风浪 | 1 | 领先 condition -2，落后 -3 | 比分不平 |
| MentalFortress | 心态大师 | 4 | momentum 系数 ×0.5 | 始终 |
| Fragile | 玻璃心 | 1 | momentum -2（而非 -1） | 赛后更新 |

### 体能类

| 特性 | 中文名 | 稀有度 | 效果 | 触发条件 |
|------|--------|--------|------|----------|
| Ironman | 铁人 | 3 | 无疲劳惩罚 | 始终 |
| Volatile | 状态敏感 | 2 | stability -10 | 始终 |

### 特殊类

| 特性 | 中文名 | 稀有度 | 效果 | 触发条件 |
|------|--------|--------|------|----------|
| RisingStar | 新星 | 3 | ability +3 | 首个赛季 |
| Veteran | 老将风范 | 3 | stability +15 | 30岁以上 |
| TeamLeader | 团队核心 | 5 | 队友 condition +1 | 始终 |

## 互斥规则

以下特性不可同时存在于一名选手身上：

```
SlowStarter ↔ FastStarter    （不可能同时慢热和快枪）
Explosive   ↔ Consistent    （爆发和稳定矛盾）
ComebackKing↔ Tilter        （逆风王不可能顺风浪）
Tilter      ↔ MentalFortress（心态崩和心态稳矛盾）
MentalFortress↔ Fragile     （心态大师不可能玻璃心）
```

## 特性生成

### 数量规则

| 选手能力值 | 特性数量 |
|-----------|---------|
| 68-100 | 2-3 个 |
| 61-67 | 1-2 个 |
| 54-60 | 0-1 个 |
| <54 | 30% 概率 1 个 |

### 前置条件

- **RisingStar**: 仅年龄 ≤ 20 的选手可获得
- **Veteran**: 仅年龄 ≥ 28 的选手可获得
- **TeamLeader**: 仅能力 ≥ 65 的选手可获得

### 权重

生成时按稀有度加权：`weight = 1 / rarity`。稀有度 1 的 Tilter 最常见（权重 1.0），稀有度 5 的 TeamLeader 最稀有（权重 0.2）。

## 修正值钳位

综合修正值在应用前会被限制在合理范围内：

| 修正项 | 范围 |
|--------|------|
| ability_mod | [-10, 10] |
| stability_mod | [-20, 20] |
| condition_mod | [-5, 5] |
| ability_ceiling_mod | [-5, 10] |

## 双路径集成

### 快速模拟路径（时间推进）

```
game_flow.rs: simulate_all_phase_matches
  ├── load_team_players()        ← 批量加载首发选手 + 特性
  ├── MetaEngine::get_current_weights()
  ├── 构建 MatchSimContext
  └── match_engine.simulate_match_with_traits()
        └── calculate_trait_adjusted_power()  ← 每局每位选手
              ├── TraitEngine::calculate_combined_modifiers()
              ├── TraitEngine::apply_modifiers()
              ├── TeamLeader 加成
              ├── 稳定性噪声
              └── MetaEngine 位置加权
```

### 详细模拟路径（手动触发）

```
match_commands.rs: simulate_game_with_players
  ├── 查询双方选手数据
  ├── 批量加载特性 (parse_trait_type → TraitType::from_str)
  ├── 构建 TraitContext
  ├── TraitEngine::calculate_combined_modifiers()
  ├── TraitEngine::apply_modifiers()
  └── 逐选手计算发挥值 + 记录影响力
```

## 计算示例

### 示例：Clutch 特性在季后赛 BO5

选手属性：ability=80, stability=70, condition=0, traits=[Clutch]

**第1局（比分 0:0，季后赛）**:
```
TraitContext: is_playoff=true, game_number=1, score_diff=0
Clutch触发: condition_mod = +3
修正后: ability=80, stability=70, condition=3
噪声: σ = (100-70)/10 = 3.0, noise ~ N(0, 3)
实际能力 ≈ 80 + 3 + noise = 83 ± 3
```

**对比无特性选手（相同属性）**:
```
修正后: ability=80, stability=70, condition=0
实际能力 ≈ 80 + 0 + noise = 80 ± 3
```

Clutch 特性在季后赛中提供约 3 点 condition 优势。

### 示例：ComebackKing 在落后局面

选手属性：ability=75, traits=[ComebackKing], 当前比分 0:1

```
TraitContext: score_diff=-1
ComebackKing触发: condition_mod = +3
实际能力 ≈ 75 + 3 + noise = 78 ± noise
```

## 快速模拟中暂不生效的特性

| 特性 | 原因 | 生效路径 |
|------|------|----------|
| MentalFortress | 需要跨场次 momentum 状态追踪 | 详细模拟 |
| Fragile | 需要赛后 momentum 更新逻辑 | 详细模拟 |
| Ironman | 需要跨场次疲劳计数 | 详细模拟 |

这些特性需要"局间动态"功能支持，计划在后续版本中为快速模拟路径实现。

## 数据库

### player_traits 表

```sql
CREATE TABLE player_traits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    trait_type TEXT NOT NULL,        -- TraitType 枚举的 snake_case 字符串
    acquired_season INTEGER,         -- 获得特性的赛季
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(save_id, player_id, trait_type)
);

CREATE INDEX idx_player_traits ON player_traits(player_id);
```

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/traits.rs` | 特性定义、修正计算、生成逻辑、互斥规则 |
| `src-tauri/src/engines/match_simulation.rs` | MatchPlayerInfo、MatchSimContext、simulate_match_with_traits |
| `src-tauri/src/services/game_flow.rs` | load_team_players、simulate_all_phase_matches |
| `src-tauri/src/commands/match_commands.rs` | 详细模拟路径的特性集成 |
| `src-tauri/src/engines/condition.rs` | MatchContext（详细模拟路径用） |
