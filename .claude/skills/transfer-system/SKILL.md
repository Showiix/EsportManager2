---
name: transfer-system
description: 电竞经理游戏的转会系统。8轮制转会流程、AI球队性格、选手评估、自由球员竞标、转会谈判、跨赛区转会偏好值。当需要修改转会规则、AI决策逻辑、转会流程、跨赛区意愿度时使用此技能。
---

# 转会系统 (Transfer System)

## Overview

转会系统实现了完整的8轮转会流程，包括赛季结算、续约谈判、自由球员竞标、合同选手转会、财务调整等环节。系统通过 AI 球队性格配置实现差异化的转会决策。

## 核心组件

### TransferEngine

**文件**: `src-tauri/src/engines/transfer/`

转会引擎，实现8轮转会流程。

```rust
pub struct TransferEngine {
    config: TransferConfig,
}
```

**核心方法**:
- `start_transfer_window(...)` - 开始转会期
- `execute_round(...)` - 执行单轮转会
- `fast_forward(...)` - 快进模式

## 8轮转会流程

| 轮次 | 名称 | 功能 |
|------|------|------|
| 1 | 赛季结算 | 年龄增长、能力成长/衰退、退役处理 |
| 2 | 双向评估 | AI评估是否需要引援/清洗 |
| 3 | 续约谈判 | 到期合同续约 |
| 4 | 自由球员竞标 | 自由球员签约 |
| 5 | 合同选手转会 | 有合同选手的买卖 |
| 6 | 财务调整 | 财务困难球队减员 |
| 7 | 最终补救 | 阵容不足球队补充 |
| 8 | 选秀权拍卖 | 选秀权交易（选秀年） |

## 数据结构

### TransferConfig (配置)

```rust
pub struct TransferConfig {
    pub max_rounds: i64,           // 最大轮次 (8)
    pub min_roster_size: i64,      // 最小阵容 (5)
    pub max_roster_size: i64,      // 最大阵容 (8)
    pub contract_duration: i64,    // 默认合同年限 (3)
    pub salary_cap_ratio: f64,     // 薪资占比上限 (0.6)
}
```

### TransferWindowResponse (转会期状态)

```rust
pub struct TransferWindowResponse {
    pub window_id: i64,
    pub season_id: i64,
    pub status: String,          // "IN_PROGRESS" | "COMPLETED"
    pub current_round: i64,
}
```

### RoundResult (轮次结果)

```rust
pub struct RoundResult {
    pub round: i64,
    pub round_name: String,
    pub events: Vec<TransferEvent>,
    pub summary: String,
}
```

### TransferEvent (转会事件)

```rust
pub struct TransferEvent {
    pub id: i64,
    pub window_id: i64,
    pub round: i64,
    pub event_type: String,
    pub player_id: Option<i64>,
    pub player_name: Option<String>,
    pub from_team_id: Option<i64>,
    pub from_team_name: Option<String>,
    pub to_team_id: Option<i64>,
    pub to_team_name: Option<String>,
    pub transfer_fee: Option<i64>,
    pub salary: Option<i64>,
    pub contract_years: Option<i64>,
    pub level: String,           // "MAJOR" | "MINOR" | "INFO"
    pub description: String,
}
```

### AITeamPersonality (AI球队性格)

```rust
pub enum AITeamPersonality {
    Aggressive,   // 激进型 - 高风险高回报
    Balanced,     // 平衡型 - 稳健经营
    Conservative, // 保守型 - 稳定优先
    Youth,        // 青训型 - 培养新人
    StarHunter,   // 追星型 - 追逐明星
    Bargain,      // 淘金型 - 寻找性价比
}

impl AITeamPersonality {
    pub fn default_weights(&self) -> PersonalityWeights {
        match self {
            Aggressive => PersonalityWeights {
                short_term_focus: 0.8,
                risk_tolerance: 0.9,
                star_chasing: 0.7,
                ..
            },
            Youth => PersonalityWeights {
                youth_preference: 0.9,
                long_term_focus: 0.8,
                bargain_hunting: 0.6,
                ..
            },
            // ...
        }
    }
}
```

### TeamPersonalityConfig (球队性格配置)

```rust
pub struct TeamPersonalityConfig {
    pub id: i64,
    pub team_id: i64,
    pub save_id: String,
    pub personality: String,
    pub short_term_focus: f64,   // 短期目标权重
    pub long_term_focus: f64,    // 长期目标权重
    pub risk_tolerance: f64,     // 风险容忍度
    pub youth_preference: f64,   // 年轻球员偏好
    pub star_chasing: f64,       // 明星追逐度
    pub bargain_hunting: f64,    // 性价比导向
}
```

### TeamReputation (球队声望)

```rust
pub struct TeamReputation {
    pub team_id: i64,
    pub reputation_score: f64,    // 声望分 (0-100)
    pub reputation_tier: String,  // 档次
    pub recent_titles: i64,       // 近期冠军数
    pub historical_titles: i64,   // 历史冠军数
    pub average_ability: f64,     // 平均能力值
    pub financial_health: f64,    // 财务健康度
}
```

## 跨赛区转会偏好值系统

### 概述

选手拥有赛区偏好值，影响跨赛区转会时的意愿度。中国选手倾向留在 LPL，韩国选手相对开放外出。

### 数据库字段 (players 表)

| 字段 | 类型 | 说明 |
|------|------|------|
| `home_region_id` | INTEGER | 选手出生赛区（首次加入的赛区 ID） |
| `region_loyalty` | INTEGER | 本赛区偏好值 0-100，越高越不愿意离开 |

### 默认值规则

| 赛区 | region_id | 默认 region_loyalty | 范围 |
|------|-----------|---------------------|------|
| LPL | 1 | 82 | 75-90 |
| LCK | 2 | 65 | 55-75 |
| LEC | 3 | 55 | 45-65 |
| LCS | 4 | 50 | 40-60 |

### 意愿度计算

**文件**: `src-tauri/src/engines/transfer/scoring.rs` - `calculate_willingness` 函数

```rust
fn calculate_willingness(
    &self,
    _ability: u8,
    loyalty: u8,
    _age: u8,
    offered_salary: i64,
    current_salary: i64,
    home_region_id: Option<i64>,   // 选手出生赛区
    target_region_id: Option<i64>, // 目标球队赛区
    region_loyalty: i64,           // 赛区偏好值
    rng: &mut impl Rng,
) -> f64 {
    // ... 薪资、忠诚度计算 ...
    let base_willingness = (base + random_factor).clamp(0.0, 100.0);

    // 跨赛区惩罚
    let cross_region_factor = match (home_region_id, target_region_id) {
        (Some(home), Some(target)) if home != target => {
            // 跨赛区转会，意愿度乘以 (100 - region_loyalty) / 100
            (100.0 - region_loyalty as f64) / 100.0
        }
        _ => 1.0  // 本赛区无惩罚
    };

    base_willingness * cross_region_factor
}
```

### 示例计算

| 选手赛区 | region_loyalty | 跨区因子 | 效果 |
|----------|----------------|----------|------|
| LPL | 80 | 0.20 | 意愿度 ×0.2，大幅降低 |
| LCK | 65 | 0.35 | 意愿度 ×0.35，中度降低 |
| LEC | 55 | 0.45 | 意愿度 ×0.45，轻度降低 |
| LCS | 50 | 0.50 | 意愿度 ×0.5，轻度降低 |

### TransferOffer 结构

```rust
pub struct TransferOffer {
    pub team_id: i64,
    pub player_id: i64,
    pub offered_salary: i64,
    pub contract_years: i64,
    pub transfer_fee: i64,
    pub signing_bonus: i64,
    pub match_score: f64,
    pub priority: f64,
    pub target_region_id: Option<i64>,  // 目标球队赛区ID（用于跨赛区转会意愿计算）
}
```

### 新选手初始化

**文件**: `src-tauri/src/commands/draft_commands.rs` - `make_draft_pick` 函数

选手通过选秀加入球队时：
- `home_region_id` = 选秀赛区 ID
- `region_loyalty` = 根据赛区生成随机值

```rust
let region_loyalty: i64 = match region_id {
    1 => 75 + (rand::random::<u8>() % 16) as i64,  // LPL: 75-90
    2 => 55 + (rand::random::<u8>() % 21) as i64,  // LCK: 55-75
    3 => 45 + (rand::random::<u8>() % 21) as i64,  // LEC: 45-65
    4 => 40 + (rand::random::<u8>() % 21) as i64,  // LCS: 40-60
    _ => 60,
};
```

### 数据库迁移

**文件**: `src-tauri/src/db/connection.rs` - `run_migrations` 函数

```sql
-- 添加字段
ALTER TABLE players ADD COLUMN home_region_id INTEGER;
ALTER TABLE players ADD COLUMN region_loyalty INTEGER NOT NULL DEFAULT 70;

-- 初始化现有选手的 home_region_id（根据当前球队的赛区）
UPDATE players SET home_region_id = (
    SELECT t.region_id FROM teams t WHERE t.id = players.team_id
) WHERE team_id IS NOT NULL AND home_region_id IS NULL;

-- 根据赛区设置 region_loyalty
UPDATE players SET region_loyalty = CASE
    WHEN home_region_id = 1 THEN 75 + ABS(RANDOM() % 16)  -- LPL
    WHEN home_region_id = 2 THEN 55 + ABS(RANDOM() % 21)  -- LCK
    WHEN home_region_id = 3 THEN 45 + ABS(RANDOM() % 21)  -- LEC
    WHEN home_region_id = 4 THEN 40 + ABS(RANDOM() % 21)  -- LCS
    ELSE 60
END WHERE home_region_id IS NOT NULL;
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/transfer_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `start_transfer_window` | 开始转会期 | `TransferWindowResponse` |
| `execute_transfer_round` | 执行单轮 | `RoundExecutionResponse` |
| `fast_forward_transfer` | 快进转会 | `FastForwardResponse` |
| `get_transfer_events` | 获取事件 | `Vec<TransferEvent>` |
| `get_transfer_report` | 获取报告 | `TransferReport` |
| `get_transfer_window_status` | 获取状态 | `TransferWindowResponse` |
| `get_team_personality` | 获取球队性格 | `TeamPersonalityConfig` |
| `update_team_personality` | 更新球队性格 | `bool` |
| `get_team_reputation` | 获取球队声望 | `TeamReputation` |
| `get_player_market_list` | 获取选手列表 | `Vec<PlayerContractInfo>` |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 开始转会期
export async function startTransferWindow(): Promise<TransferWindowResponse>

// 执行单轮
export async function executeTransferRound(windowId: number, round: number): Promise<RoundExecutionResponse>

// 快进
export async function fastForwardTransfer(windowId: number, fromRound?: number): Promise<FastForwardResponse>

// 获取事件
export async function getTransferEvents(windowId: number, round?: number, level?: string): Promise<TransferEvent[]>

// 获取报告
export async function getTransferReport(windowId: number): Promise<TransferReport>

// 获取/更新球队性格
export async function getTeamPersonality(teamId: number): Promise<TeamPersonalityConfig>
export async function updateTeamPersonality(teamId: number, request: UpdatePersonalityRequest): Promise<boolean>

// 获取球队声望
export async function getTeamReputation(teamId: number): Promise<TeamReputation>
```

## 各轮次详解

### 第1轮: 赛季结算
```rust
// 年龄增长
new_age = age + 1;

// 能力成长 (30岁前)
if age <= 30 && ability < potential {
    new_ability = min(ability + tag_growth, potential, 100);
}

// 能力衰退 (30岁后)
if age > 30 {
    new_ability = max(ability - 1, 50);
}

// 退役检查
if age >= 35 && ability < 51 {
    status = "RETIRED";
}
```

### 第2轮: 双向评估
- AI 评估阵容强度和位置需求
- 决定引援优先级和预算分配
- 决定是否挂牌出售

### 第3轮: 续约谈判
- 合同到期选手 (`contract_end_season == current_season`)
- AI 根据选手价值决定是否续约
- 续约失败变为自由球员

### 第4轮: 自由球员竞标
- 无合同选手可被任意球队签约
- 多队竞争时出价最高者获胜
- 球队声望影响选手选择
- **位置筛选**：pos_count >= 2 不报价；pos_count == 1 仅能力升级或青训新人（age<=23 & potential>=70）才报价
- **match_score 门槛**：match_score < 50 不报价
- **ability_score 分段**：90+ → 100, 80-89 → 90, 75-79 → 80, 70-74 → 70, 65-69 → 60, 60-64 → 50, 55-59 → 35, 50-54 → 20, <50 → 10
- **need_score 分段**：空位 → 100, 1人 → 40, 2人 → 15, ≥3人 → 5

### 第5轮: 合同选手转会
- 有合同选手需要支付转会费
- 忠诚度影响转会费溢价
- 选手可能拒绝转会

### 第6轮: 财务调整
- 财务困难球队被迫出售选手
- 降薪或清洗高薪低能选手

### 第7轮: 最终补救
- 阵容不足5人的球队强制补充
- 从自由球员池或青训签入

### 第8轮: 选秀权拍卖
- 仅在选秀年激活 (S2, S6, S10...)
- 可交易选秀顺位

#### 卖签决策（4因素相乘模型）
```
sell_prob = 财务动机 × 签位留存 × 阵容系数 × 球队实力系数
```

| 因素 | 值域 | 说明 |
|------|------|------|
| 财务动机 | 0.05-0.70 | Bankrupt 0.70, Deficit 0.50, Tight 0.25, Healthy 0.10, Wealthy 0.05 |
| 签位留存 | 0.10-1.20 | 状元签 0.10, 2号签 0.15, 3号签 0.20, 4-5位 0.40, 6-8位 0.60, 9-10位 0.80, 11-12位 1.00, 13+ 1.20 |
| 阵容系数 | 0.10-1.50 | <5人 0.10, <7人 0.50, 7-8人 1.00, >=9人 1.50 |
| 球队实力 | 0.50-1.30 | 强队(>65)持高签(<=5) 1.30, 弱队(<55)持高签 0.50, 其他 1.00 |

概率上限 90%。

#### 竞拍决策（多因素加权）
- 预算按财务状态差异化: Wealthy 40%, Healthy 30%, Tight 15%, Deficit 5%, Bankrupt 不竞拍
- 签位价值 14 级梯度（状元签 100 → 13号签 25 → 14+ 20）
- 竞拍概率 = 签位价值/100 × 0.50 × 阵容需求 × 实力因素 × 财务信心 × 价格衰减 × 轮次系数
- 出价上限按财务状态差异化: Wealthy ×1.5, Healthy ×1.3, Tight ×1.15, 其他 ×1.05

## 系统集成

### 与时间推进系统
- `TransferWindow` 阶段触发转会流程
- 转会结束后推进到选秀/赛季结束

### 与选手系统
- 使用 `loyalty` / `satisfaction` 影响转会意愿
- 使用 `home_region_id` / `region_loyalty` 影响跨赛区转会意愿
- 更新合同、薪资、所属球队

### 与财政系统
- 转会费影响球队余额
- 薪资影响财务健康

### 与荣誉系统
- 历史荣誉影响球队声望

### 跨赛区转会
- 四个赛区（LPL/LCK/LEC/LCS）统一转会期
- 选手可跨赛区转会，但意愿度受 `region_loyalty` 影响
- 中国选手更倾向留在 LPL（region_loyalty 75-90）
- 韩国选手相对开放外出（region_loyalty 55-75）

## 使用示例

### 开始转会期
```typescript
const response = await startTransferWindow()
console.log(`转会期ID: ${response.window_id}`)
```

### 逐轮执行
```typescript
for (let round = 1; round <= 8; round++) {
    const result = await executeTransferRound(windowId, round)
    console.log(`第${round}轮: ${result.round_name}`)
    console.log(`事件数: ${result.events.length}`)
}
```

### 快进模式
```typescript
const result = await fastForwardTransfer(windowId, 1)
console.log(`完成${result.completed_rounds}轮`)
console.log(`总事件: ${result.total_events}`)
```

### 配置球队性格
```typescript
await updateTeamPersonality(teamId, {
    personality: "YOUTH",
    youth_preference: 0.9,
    long_term_focus: 0.8,
})
```

## 数据库表

### transfer_windows

```sql
CREATE TABLE transfer_windows (
    id INTEGER PRIMARY KEY,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    status TEXT DEFAULT 'IN_PROGRESS',
    current_round INTEGER DEFAULT 0,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);
```

### transfer_events

```sql
CREATE TABLE transfer_events (
    id INTEGER PRIMARY KEY,
    window_id INTEGER NOT NULL,
    round INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    player_id INTEGER,
    from_team_id INTEGER,
    to_team_id INTEGER,
    transfer_fee INTEGER,
    salary INTEGER,
    contract_years INTEGER,
    level TEXT DEFAULT 'MINOR',
    description TEXT
);
```

### team_personality_configs

```sql
CREATE TABLE team_personality_configs (
    id INTEGER PRIMARY KEY,
    team_id INTEGER UNIQUE,
    save_id TEXT,
    personality TEXT,
    short_term_focus REAL,
    long_term_focus REAL,
    risk_tolerance REAL,
    youth_preference REAL,
    star_chasing REAL,
    bargain_hunting REAL
);
```

## 注意事项

1. **同一赛季限制**: 每赛季只能有一个进行中的转会期
2. **轮次顺序**: 必须按1-8顺序执行，不能跳过
3. **球队性格初始化**: 首次开始转会期会为所有球队初始化性格配置
4. **财务限制**: 球队余额不足时无法完成高价转会
5. **选秀年判断**: `(season - 2) % 4 == 0` 为选秀年
