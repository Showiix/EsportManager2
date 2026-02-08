# 选手系统

## 概述

选手系统管理游戏中所有选手的属性、成长、衰退和退役。选手是比赛模拟的核心，其能力值直接影响队伍战力和比赛结果。

## 选手基础模型

```rust
pub struct Player {
    pub id: u64,
    pub game_id: String,              // 游戏ID（显示名称）
    pub real_name: Option<String>,    // 真实姓名
    pub nationality: Option<String>,  // 国籍
    pub age: u8,                      // 年龄 (16-35)
    pub ability: u8,                  // 能力值 (0-100)
    pub potential: u8,                // 潜力值 (0-100)
    pub stability: u8,               // 稳定性 (0-100)
    pub tag: PlayerTag,              // 标签
    pub status: PlayerStatus,        // 状态 (Active/Retired)
    pub position: Option<Position>,  // 位置
    pub team_id: Option<u64>,        // 所属战队
    pub salary: u64,                 // 薪资（单位：元）
    pub market_value: u64,           // 基础身价（单位：元）
    pub calculated_market_value: u64,// 计算后身价（含荣誉和赛区系数）
    pub contract_end_season: Option<u32>, // 合同到期赛季
    pub join_season: u32,            // 加入赛季
    pub retire_season: Option<u32>,  // 退役赛季
    pub is_starter: bool,            // 是否首发
    pub loyalty: u8,                 // 忠诚度 (0-100)
    pub satisfaction: u8,            // 满意度 (0-100)
}
```

## 选手属性

### 核心属性

| 属性 | 范围 | 说明 |
|------|------|------|
| `ability` | 0-100 | 当前能力值，直接影响比赛发挥 |
| `potential` | 0-100 | 潜力上限，成长不会超过此值 |
| `stability` | 0-100 | 稳定性，影响发挥波动 |
| `loyalty` | 0-100 | 忠诚度，影响转会意愿 |

### 位置

```rust
pub enum Position {
    Top,     // 上单
    Jug,     // 打野
    Mid,     // 中单
    Adc,     // 射手
    Sup,     // 辅助
}
```

### 选手标签

```rust
pub enum PlayerTag {
    Ordinary,  // 平庸 - 每赛季+1能力
    Normal,    // 一般 - 每赛季+2能力
    Genius,    // 天才 - 每赛季+3能力
}
```

| 标签 | 每赛季成长 | 身价系数 |
|------|-----------|---------|
| Genius | +3 | 1.2 |
| Normal | +2 | 1.0 |
| Ordinary | +1 | 0.9 |

### 忠诚度类型

```rust
pub enum LoyaltyType {
    Devoted,      // 忠心耿耿 (80-100)
    Loyal,        // 忠诚 (60-79)
    Neutral,      // 中立 (40-59)
    Opportunist,  // 机会主义 (20-39)
    Mercenary,    // 雇佣兵 (0-19)
}
```

## 选手生命周期

### 成长阶段

每赛季结算时，按以下流程计算能力变化：

```
成长 = f(标签随机基础, 突破/停滞事件, 年龄系数, 特性修正, 赛季表现)
```

#### ① 随机基础成长

| 标签 | 成长范围 | 期望值 |
|------|---------|--------|
| Genius | 2~4 | 3.0 |
| Normal | 1~3 | 2.0 |
| Ordinary | 0~2 | 1.0 |

#### ② 突破/停滞事件（互斥，10%总事件率）

- 5% 概率"突破赛季"：基础成长 +1
- 5% 概率"停滞赛季"：基础成长 = 0

#### ③ 年龄系数（平滑渐变）

| 年龄 | 系数 | 说明 |
|------|------|------|
| 16-24 | 1.0 | 全速成长 |
| 25-26 | 0.7 | 开始放缓 |
| 27-28 | 0.4 | 明显放缓 |
| 29-30 | 0.15 | 几乎停止 |

成长值 = `probabilistic_round(基础成长 × 年龄系数 × 特性修正)`

`probabilistic_round`: 2.7 → 70%概率得3，30%概率得2

#### ④ 表现加成（基于 player_season_stats）

| 条件 | 效果 | 说明 |
|------|------|------|
| games≥20 且 avg_perf > ability+5 | +1 成长 | 超常发挥 |
| games≥20 且 avg_perf > ability | 50%概率 +1 | 突破成长 |
| games==0 | 成长÷2 | 缺乏实战 |
| games>0 且 avg_perf < ability-5 | -1 成长 | 表现低迷 |

#### ⑤ 最终计算

```
final_growth = max(0, 年龄衰减后成长 + 表现加成)
new_ability = min(ability + final_growth, potential, 100)
```

### 衰退阶段

成长上限年龄后开始衰退（默认31岁，LateBlocker特性延至33岁）：

| 等效年龄 | 基础衰退 | 说明 |
|---------|---------|------|
| 31 | 0.5/季 | 缓慢衰退 |
| 32-33 | 1.0/季 | 正常衰退 |
| 34-35 | 1.5/季 | 加速衰退 |
| 36+ | 2.0/季 | 快速衰退 |

**标签影响衰退速率：**

| 标签 | 衰退系数 | 说明 |
|------|---------|------|
| Genius | ×0.7 | 天才保持更久 |
| Normal | ×1.0 | 标准 |
| Ordinary | ×1.2 | 衰退更快 |

最终衰退 = `probabilistic_round(基础衰退 × 标签系数 × 特性系数)`，能力最低 50。

### 潜力微漂移

每赛季结算时小概率调整潜力值：

| 条件 | 概率 | 效果 |
|------|------|------|
| games≥30 且 avg_perf > ability+5 | 8% | potential +1 |
| games≥20 且 avg_perf < ability-5 且 age>28 | 12% | potential -1 |

潜力值范围：50-100。

### 退役判定（概率制）

| 条件 | 退役概率 |
|------|---------|
| age ≥ 37 | 100% |
| age ≥ 35 且 ability < 50 | 80% |
| age ≥ 35 且 ability < 60 | 50% |
| age ≥ 33 且 ability < 55 | 20% |

按最严格匹配，不叠加。

## 稳定性与年龄

| 年龄段 | 稳定性范围 | 特点 |
|--------|-----------|------|
| 16-24岁 | 60-75 | 波动大，上限高 |
| 25-29岁 | 75-85 | 最稳定，巅峰期 |
| 30岁+ | 85-95 | 极其稳定，难爆发 |

## 身价计算

```rust
pub fn calculate_market_value(player: &Player, honors: &[Honor]) -> u64 {
    let base_value = calculate_base_value(player);
    let ability_factor = calculate_ability_factor(player);
    let position_factor = player.position.market_value_factor();
    let region_factor = player.get_region_factor();
    let age_factor = calculate_age_factor(player.age);
    let tag_factor = player.tag.market_value_factor();
    let honor_factor = calculate_honor_factor(honors, player.age);

    (base_value as f64 * ability_factor * position_factor *
     region_factor * age_factor * tag_factor * honor_factor) as u64
}
```

### 各系数

| 系数类型 | 计算方式 | 范围 |
|---------|---------|------|
| 位置系数 | Mid: 1.2, Adc: 1.15, Jug: 1.1, Top: 1.0, Sup: 0.9 | 0.9-1.2 |
| 赛区系数 | LPL: 1.3, LCK: 1.2, LEC: 1.0, LCS: 0.9 | 0.9-1.3 |
| 年龄系数 | 20-24: 1.2, 25: 1.1, 26: 1.0, 每+1岁-0.1 | 0.5-1.2 |
| 标签系数 | Genius: 1.2, Normal: 1.0, Ordinary: 0.9 | 0.9-1.2 |

## 选手特性系统

特性影响选手在不同情境下的表现，通过修改 ability/stability/condition 实现。

### 特性类型（18种）

**大赛表现类:**

| 特性 | 英文名 | 效果 |
|------|--------|------|
| 大赛型 | `Clutch` | 季后赛/国际赛 condition +3 |
| 慢热型 | `SlowStarter` | 第1局 -2，第3+局 +2 |
| 快枪手 | `FastStarter` | 第1局 +2，第3+局 -1 |

**稳定性类:**

| 特性 | 英文名 | 效果 |
|------|--------|------|
| 爆发型 | `Explosive` | stability -15，能力上限 +5 |
| 稳定型 | `Consistent` | stability +10，能力上限 -3 |

**心态类:**

| 特性 | 英文名 | 效果 |
|------|--------|------|
| 逆风王 | `ComebackKing` | 落后时 condition +3 |
| 顺风浪 | `Tilter` | 领先 -2，落后 -3 [负面] |
| 心态大师 | `MentalFortress` | momentum 效果减半 |
| 玻璃心 | `Fragile` | 输了 momentum -2 [负面] |

**体能类:**

| 特性 | 英文名 | 效果 |
|------|--------|------|
| 铁人 | `Ironman` | 无疲劳惩罚 |
| 状态敏感 | `Volatile` | condition 波动×1.5 [负面] |

**特殊类:**

| 特性 | 英文名 | 效果 |
|------|--------|------|
| 新星 | `RisingStar` | 首赛季 ability +3 |
| 老将风范 | `Veteran` | 30岁后 stability +15 |
| 团队核心 | `TeamLeader` | 队友 condition +1 |

**成长类:**

| 特性 | 英文名 | 效果 |
|------|--------|------|
| 大器晚成 | `LateBlocker` | 成长/衰退年龄按 age-2 计算，延长2年巅峰期 |
| 神童 | `Prodigy` | 20岁前成长×1.5，25岁后成长×0.8 |
| 抗衰老 | `Resilient` | 衰退速率×0.5 |
| 易碎 | `GlassCannon` | 衰退速率×1.5，但能力上限+3 [负面] |

### 互斥规则

- SlowStarter 与 FastStarter
- Explosive 与 Consistent
- ComebackKing 与 Tilter
- MentalFortress 与 Fragile/Tilter
- LateBlocker 与 Prodigy
- Resilient 与 GlassCannon

## 选手满意度

满意度影响选手的转会意愿和续约态度。

| 满意度范围 | 状态 | 影响 |
|-----------|------|------|
| 80-100 | 非常满意 | 愿意降薪续约 |
| 60-79 | 满意 | 正常续约 |
| 40-59 | 一般 | 可能拒绝续约 |
| 20-39 | 不满 | 主动求转会 |
| 0-19 | 非常不满 | 强烈要求离队 |

## 赛区偏好

选手有出生赛区偏好，影响跨赛区转会意愿：

| 出生赛区 | 偏好值范围 |
|---------|-----------|
| LPL | 75-90 |
| LCK | 55-75 |
| LEC | 45-65 |
| LCS | 40-60 |

**跨赛区转会意愿**:

```
willingness = base_willingness × (100 - region_loyalty) / 100
```

## 自由选手

自由选手是不属于任何战队的选手（`team_id = NULL`），可在转会期被签约。

### 初始自由选手

游戏初始化时包含 **110 名**预定义自由选手（LPL 28 / LCK 28 / LEC 27 / LCS 27）：

| 赛区 | 人数 | 数据来源 |
|------|------|---------|
| LPL | 28 | `free_agent_data.rs` |
| LCK | 28 | `free_agent_data.rs` |
| LEC | 27 | `free_agent_data.rs` |
| LCS | 27 | `free_agent_data.rs` |

**属性范围**：

| 属性 | 范围 | 说明 |
|------|------|------|
| 能力值 | 34-62 | 整体低于战队选手 |
| 潜力值 | 58-64 | 部分年轻选手有成长空间 |
| 年龄 | 17-25 | 覆盖新人到老将 |
| 标签 | Ordinary/Normal | 无 Genius |

**初始化路径**：

```
快速创建 → init_service 直接创建战队选手（无自由选手）
自定义创建 → get_default_game_config 加载 free_agent_data → 用户可编辑 → init_service 创建
```

自定义创建时，自由选手特点：
- `team_id: None`（无所属战队）
- `is_starter: false`
- `contract_end_season: None`（无合同）
- 忠诚度、满意度、身价等根据属性自动计算

## API 接口

| 接口 | 描述 |
|------|------|
| `get_players(save_id)` | 获取所有选手 |
| `get_player_detail(player_id)` | 获取选手详情 |
| `update_player(player)` | 更新选手信息 |
| `get_player_honors(player_id)` | 获取选手荣誉 |
| `get_retired_players(save_id)` | 获取已退役选手 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/models/player.rs` | 选手数据模型（含 Position, PlayerTag, PlayerStatus, LoyaltyType） |
| `src-tauri/src/engines/market_value.rs` | 身价计算引擎 |
| `src-tauri/src/engines/traits.rs` | 选手特性系统（TraitType 枚举及效果计算） |
| `src-tauri/src/engines/condition.rs` | 选手状态/体力系统 |
| `src-tauri/src/engines/satisfaction.rs` | 选手满意度引擎 |
| `src-tauri/src/engines/player_decision.rs` | AI 选手决策 |
| `src-tauri/src/engines/player_performance.rs` | 选手表现评估 |
| `src-tauri/src/services/free_agent_data.rs` | 预定义自由选手数据（110人） |
| `src-tauri/src/commands/query_commands.rs` | 选手查询命令接口 |
