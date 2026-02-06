---
name: player-system
description: 电竞经理游戏的选手系统。管理选手属性、成长曲线、忠诚度、满意度、身价计算。当需要修改选手属性、成长规则、身价公式、忠诚度机制时使用此技能。
---

# 选手系统 (Player System)

## Overview

选手系统管理游戏中所有选手的核心属性，包括能力值、潜力值、年龄成长、忠诚度、满意度以及身价计算。它是转会系统、比赛模拟系统的基础数据来源。

## 核心组件

### Player Model

**文件**: `src-tauri/src/models/player.rs`

选手数据模型，包含所有选手属性和计算方法。

## 数据结构

### Player (选手)

```rust
pub struct Player {
    pub id: u64,
    pub game_id: String,           // 游戏ID (如 "Faker")
    pub real_name: Option<String>, // 真名
    pub nationality: Option<String>,
    pub age: u8,                   // 年龄 (17-40)
    pub ability: u8,               // 当前能力 (0-100)
    pub potential: u8,             // 潜力上限 (0-100)
    pub stability: u8,             // 稳定性 (影响比赛发挥波动)
    pub tag: PlayerTag,            // 成长标签
    pub status: PlayerStatus,      // 状态
    pub position: Option<Position>,// 位置
    pub team_id: Option<u64>,      // 所属队伍
    pub salary: u64,               // 薪资 (元/赛季)
    pub market_value: u64,         // 基础身价
    pub calculated_market_value: u64, // 计算后身价
    pub contract_end_season: Option<u32>, // 合同到期赛季
    pub join_season: u32,          // 加入赛季
    pub retire_season: Option<u32>,// 退役赛季
    pub is_starter: bool,          // 是否首发
    pub loyalty: u8,               // 忠诚度 (0-100)
    pub satisfaction: u8,          // 满意度 (0-100)
}
```

### PlayerTag (成长标签)

```rust
pub enum PlayerTag {
    Ordinary,  // 平庸: +1能力/赛季, 身价×0.9
    Normal,    // 一般: +2能力/赛季, 身价×1.0
    Genius,    // 天才: +3能力/赛季, 身价×1.2
}
```

### Position (位置)

```rust
pub enum Position {
    Top,  // 上单: 身价×1.0
    Jug,  // 打野: 身价×1.1
    Mid,  // 中单: 身价×1.2 (核心C位)
    Adc,  // ADC: 身价×1.15
    Sup,  // 辅助: 身价×0.9
}
```

### LoyaltyType (忠诚度类型)

```rust
pub enum LoyaltyType {
    Devoted,      // 忠心耿耿 (80-100)
    Loyal,        // 忠诚 (60-79)
    Neutral,      // 中立 (40-59)
    Opportunist,  // 机会主义 (20-39)
    Mercenary,    // 雇佣兵 (0-19)
}
```

### PlayerStatus (选手状态)

```rust
pub enum PlayerStatus {
    Active,   // 在役
    Retired,  // 退役
}
```

## 身价计算公式

### 基础身价
```rust
base_value = ability × base_multiplier(ability)
           × age_factor(age)
           × potential_factor(ability, potential)
           × tag_factor
           × position_factor
```

### 完整身价 (含荣誉和赛区加成)
```rust
full_value = base_value × region_factor × honor_factor
```

### 身价系数表

#### 能力基础系数 (单位: 元)
| 能力范围 | 系数 | 示例身价 |
|----------|------|----------|
| 95-100 | 500000 | 4750万-5000万 |
| 90-94 | 350000 | 3150万-3290万 |
| 85-89 | 200000 | 1700万-1780万 |
| 80-84 | 120000 | 960万-1008万 |
| 75-79 | 70000 | 525万-553万 |
| 70-74 | 40000 | 280万-296万 |
| 60-69 | 20000 | 120万-138万 |
| <60 | 10000 | <60万 |

#### 年龄系数
| 年龄 | 系数 | 说明 |
|------|------|------|
| 17-19 | 1.5 | 超新星溢价 |
| 20-22 | 1.3 | 年轻潜力股 |
| 23-25 | 1.0 | 黄金年龄 |
| 26-27 | 0.85 | 巅峰末期 |
| 28-29 | 0.7 | 开始下滑 |
| 30+ | 0.5 | 老将 |

#### 赛区系数
| 赛区 | 系数 |
|------|------|
| LPL | 1.3 |
| LCK | 1.2 |
| LEC | 1.0 |
| LCS | 0.9 |
| Other | 0.8 |

## 忠诚度系统

### 离队意愿阈值
高忠诚度选手需要更低的满意度才会想离队：

| 忠诚度 | 离队阈值 | 说明 |
|--------|----------|------|
| 90-100 | 20 | 满意度<20才想走 |
| 70-89 | 35 | 满意度<35才想走 |
| 50-69 | 50 | 满意度<50才想走 |
| 30-49 | 60 | 满意度<60就想走 |
| <30 | 70 | 满意度<70就想走 |

### 忠诚度影响

```rust
impl Player {
    // 拒绝挖角概率
    pub fn reject_poaching_chance(&self) -> f64 {
        match self.loyalty {
            90..=100 => 0.7,  // 70%拒绝
            70..=89 => 0.4,   // 40%拒绝
            50..=69 => 0.1,   // 10%拒绝
            _ => 0.0,
        }
    }

    // 转会费溢价
    pub fn loyalty_price_factor(&self) -> f64 {
        match self.loyalty {
            80..=100 => 1.3,  // 要求130%身价
            60..=79 => 1.15,  // 要求115%身价
            _ => 1.0,
        }
    }

    // 老东家偏好加成
    pub fn former_team_bonus(&self) -> f64 {
        match self.loyalty {
            80..=100 => 0.3,  // +30%吸引力
            60..=79 => 0.15,  // +15%
            _ => 0.0,
        }
    }
}
```

## 成长与衰退规则

### 能力成长

**代码位置**: `src-tauri/src/engines/transfer.rs` — `execute_season_settlement()` 函数

- **30岁前**: `new_ability = min(ability + tag_growth, potential, 100)`
- **30岁后**: `new_ability = max(ability - 1, 50)` (每赛季-1)

### 退役条件
```rust
if age >= 35 && ability < 65 {
    status = PlayerStatus::Retired;
}
```

### 稳定性计算
```rust
fn calculate_stability(age: u8) -> u8 {
    match age {
        16..=24 => 60 + (age - 16) * 2,  // 60-76
        25..=29 => 75 + (age - 25) * 2,  // 75-85
        30..=36 => 85 + (age - 30),      // 85-91
        _ => 70,
    }
}
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/team_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `get_player` | 获取选手详情 | `Player` |
| `get_all_players` | 获取所有选手 | `Vec<Player>` |
| `get_team_roster` | 获取队伍阵容 | `Vec<Player>` |
| `update_player` | 更新选手信息 | `bool` |
| `update_player_market_value` | 更新身价 | `bool` |
| `update_all_market_values` | 批量更新身价 | `i32` |
| `set_starter` | 设置首发 | `bool` |
| `get_player_traits` | 获取选手特性 | `PlayerTraits` |
| `get_player_condition` | 获取选手状态 | `PlayerCondition` |
| `get_player_full_detail` | 获取完整详情 | `PlayerFullDetail` |

## 系统集成

### 与比赛模拟系统
- `ability` 作为战力值基础
- `stability` 影响比赛发挥波动

### 与转会系统
- `loyalty` 影响转会意愿
- `satisfaction` 决定是否想离队
- `calculated_market_value` 决定转会费

### 与财政系统
- `salary` 影响队伍薪资支出
- 身价影响转会收支

### 与荣誉系统
- 冠军数影响 `honor_factor`
- 荣誉影响最终身价

## 使用示例

### 计算选手身价
```rust
let player = get_player(player_id)?;
let base_value = player.calculate_base_market_value();
let full_value = player.calculate_full_market_value("LPL", 1.5);
println!("基础身价: {}万, 完整身价: {}万", base_value, full_value);
```

### 检查离队意愿
```rust
if player.satisfaction < player.departure_threshold() {
    println!("{} 想要离队!", player.game_id);
}
```

### 更新忠诚度
```rust
// 续约成功增加忠诚度
player.update_loyalty(10);

// 被拒绝降低忠诚度
player.update_loyalty(-15);
```

## 注意事项

1. **身价更新时机**: 赛季结算时需调用 `update_all_market_values` 更新所有选手身价
2. **年龄增长**: 每个赛季开始时 `age + 1`
3. **合同到期**: `contract_end_season` 与当前赛季相同时，选手变为自由球员
4. **首发位置**: 每队每位置最多1名首发 (`is_starter = true`)
5. **金额单位**: 薪资和身价在后端统一以**元**存储和传递，前端使用 `formatMoney` 系列函数格式化显示（参见 `financial-units` 技能）
6. **标签字符串匹配必须大小写不敏感**: 数据库中 `tag` 字段存储为首字母大写形式（`"Normal"`, `"Genius"`, `"Ordinary"`），而非全大写。在 Rust 代码中对 tag 字符串做 `match` 时，**必须**先调用 `.to_uppercase()` 再匹配全大写常量，否则所有选手都会落入 `_ =>` 默认分支。已知正确写法：
   ```rust
   match tag.to_uppercase().as_str() {
       "GENIUS" => 3,
       "NORMAL" => 2,
       _ => 1, // ORDINARY
   }
   ```
