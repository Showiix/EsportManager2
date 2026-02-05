# 选手系统

## 概述

选手系统管理游戏中所有选手的属性、成长、衰退和退役。选手是比赛模拟的核心，其能力值直接影响队伍战力和比赛结果。

## 选手基础模型

```rust
pub struct Player {
    pub id: u64,
    pub save_id: String,
    pub game_id: String,           // 游戏ID（显示名称）
    pub real_name: String,         // 真实姓名
    pub nationality: String,       // 国籍
    pub position: Position,        // 位置
    pub age: u8,                   // 年龄 (16-35)
    pub ability: u8,               // 能力值 (0-100)
    pub potential: u8,             // 潜力值 (0-100)
    pub stability: u8,             // 稳定性 (0-100)
    pub status: PlayerStatus,      // 状态
    pub tag: PlayerTag,            // 标签
    pub loyalty: u8,               // 忠诚度 (0-100)
    pub team_id: Option<u64>,      // 所属战队
    pub contract_years: u8,        // 合同剩余年限
    pub salary: u64,               // 薪资
    pub home_region_id: u64,       // 出生赛区
    pub region_loyalty: u8,        // 赛区偏好值
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
    Jungle,  // 打野
    Mid,     // 中单
    Adc,     // 射手
    Support, // 辅助
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

### 成长阶段 (16-29岁)

每赛季结束时：

```
if age < 30 && ability < potential:
    ability += tag.growth_per_season()
```

### 衰退阶段 (30岁+)

```
if age >= 30:
    ability -= 1  // 每赛季-1能力
```

### 退役判定

```
if age >= 35 && ability < 65:
    status = Retired
```

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
| `src-tauri/src/models/player.rs` | 选手数据模型 |
| `src-tauri/src/engines/market_value.rs` | 身价计算引擎 |
| `src-tauri/src/commands/player_commands.rs` | 选手命令接口 |
