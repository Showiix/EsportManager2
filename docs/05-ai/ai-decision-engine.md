# AI 决策系统

## 概述

EsportManager 2 的 AI 决策系统采用**规则驱动**的方式，模拟电竞俱乐部的决策行为。主要包含以下几个子系统：

1. **选手决策引擎** - 选手评估报价并做出选择
2. **选秀 AI 服务** - AI 球队选择新秀
3. **转会 AI 决策** - AI 球队制定转会策略
4. **财务 AI 决策** - 财务状态驱动的行为

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    TransferEngine (转会引擎)                 │
│                   8 轮转会流程调度                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ PlayerDecision│  │  DraftAI     │  │ TeamTransfer │
│   选手决策    │  │  选秀AI      │  │  Strategy    │
│   引擎       │  │  服务        │  │  球队策略    │
└──────────────┘  └──────────────┘  └──────────────┘
```

## 选手决策引擎

### 文件位置

`src-tauri/src/engines/player_decision.rs`

### 核心功能

选手根据自身策略评估所有收到的报价，选择最优的一个。

### 决策模型

```rust
pub struct PlayerDecisionResult {
    pub player_id: u64,
    pub player_name: String,
    pub chosen_offer_id: u64,
    pub chosen_team_id: u64,
    pub chosen_team_name: String,
    pub reasoning: String,  // 决策理由
}
```

### 报价评分算法

选手对每个报价进行综合评分（满分 100 分）：

| 评分维度 | 分值范围 | 权重说明 |
|---------|---------|---------|
| 薪资评分 | 0-40分 | 最重要因素 |
| 球队评分 | 0-30分 | 球队偏好 |
| 角色评分 | 0-20分 | 首发保障 |
| 其他评分 | 0-10分 | 其他因素 |

#### 薪资评分细则

```rust
let salary_score = if offer.salary >= expected * 1.2 {
    40  // 超出期望 20%
} else if offer.salary >= expected {
    35  // 满足期望
} else if offer.salary >= min_salary {
    25  // 勉强接受
} else {
    0   // 不接受
};
```

#### 球队偏好评分

```rust
let team_score = match team_priority {
    Some(1) => 30,     // 第1偏好球队
    Some(2) => 25,     // 第2偏好
    Some(3) => 20,     // 第3偏好
    Some(4..=5) => 15, // 4-5偏好
    _ => 10,           // 不在偏好列表
};
```

#### 首发保障评分

```rust
let role_score = if requires_starter {
    if guarantee_starter { 20 } else { 0 }  // 要求首发
} else {
    if guarantee_starter { 15 } else { 10 } // 不要求首发
};
```

### 决策流程

```
1. 收集所有报价
       │
       ▼
2. 过滤不满足最低要求的报价
   - 薪资 < 最低期望？排除
   - 要求首发但未保证？排除
       │
       ▼
3. 为每个报价计算综合评分
       │
       ▼
4. 选择评分最高的报价
       │
       ▼
5. 生成决策结果和理由
```

## 选秀 AI 服务

### 文件位置

`src-tauri/src/services/draft_ai_service.rs`

### 核心功能

AI 球队根据阵容需求和新秀能力，选择最优新秀。

### 评分算法

```rust
pub fn calculate_player_score(
    player: &DraftPlayer,
    position_needs: &HashMap<String, u32>,
    roster: &[Player],
) -> f64 {
    // 1. 基础评分
    let score = ability * 0.6 + potential * 0.4;

    // 2. 年龄调整（18-22岁 +10%）
    if age >= 18 && age <= 22 {
        score *= 1.1;
    }

    // 3. 位置需求调整
    score *= 1.0 + (position_need / 100.0);

    // 4. 阵容深度调整
    if same_position_count >= 2 {
        score *= 0.7;  // 位置过剩，降低优先级
    } else if same_position_count == 0 {
        score *= 1.3;  // 位置空缺，提高优先级
    }

    // 5. 年轻球员偏好（≤21岁 +10%）
    if age <= 21 {
        score *= 1.1;
    }

    score
}
```

### 位置需求计算

```rust
pub fn calculate_position_needs(roster: &[Player]) -> HashMap<String, u32> {
    // 计算各位置的需求分数 (0-100)
    // 0 = 该位置已满员
    // 100 = 该位置急需补充
}
```

| 位置球员数 | 需求分数 |
|-----------|---------|
| 0 | 100 (急需) |
| 1 | 70 (需要) |
| 2 | 30 (充足) |
| 3+ | 0 (过剩) |

## 球队转会策略

### 策略类型

根据财务状态和战绩，球队采用不同的转会策略：

| 策略类型 | 条件 | 行为 |
|---------|------|------|
| Aggressive | 富裕 + 战绩差 | 积极引援，愿意溢价 |
| Balanced | 健康财务 | 正常参与市场 |
| Conservative | 财务紧张 | 优先性价比 |
| Survival | 赤字 | 只出不进 |

### 策略评估

```rust
pub struct TeamTransferStrategy {
    pub team_id: u64,
    pub strategy_type: StrategyType,
    pub max_transfer_budget: i64,     // 最大转会预算
    pub max_salary_offer: i64,        // 最大薪资报价
    pub target_positions: Vec<String>, // 目标位置
    pub priority_players: Vec<u64>,   // 优先目标选手
}
```

### 报价生成

```rust
// 生成报价金额
fn generate_offer_amount(
    player_value: i64,
    strategy: &TeamTransferStrategy,
    competition_level: f64,  // 竞争程度
) -> i64 {
    let base = player_value;

    let multiplier = match strategy.strategy_type {
        Aggressive => 1.2,    // 愿意溢价 20%
        Balanced => 1.0,      // 市场价
        Conservative => 0.9,  // 尝试低价
        Survival => 0.0,      // 不出价
    };

    (base as f64 * multiplier * (1.0 + competition_level * 0.1)) as i64
}
```

## 财务驱动决策

### 决策矩阵

| 财务状态 | 余额范围 | 转会行为 | 续约行为 |
|---------|---------|---------|---------|
| 富裕 | > 1000万 | 积极引援 | 主动加薪续约 |
| 健康 | 500-1000万 | 正常参与 | 正常续约 |
| 紧张 | 100-500万 | 谨慎引援 | 优先保留核心 |
| 赤字 | 0-100万 | 出售选手 | 不续约高薪 |
| 破产 | < 0 | 强制出售 | 强制释放 |

### 球员估值调整

根据财务状态调整挂牌价格：

```rust
fn calculate_listing_price(
    player_value: i64,
    financial_status: FinancialStatus,
) -> i64 {
    let multiplier = match financial_status {
        Wealthy => 1.2,    // 不急售，溢价
        Healthy => 1.0,    // 市场价
        Tight => 0.9,      // 略低于市场价
        Deficit => 0.7,    // 急售折扣
        Bankrupt => 0.5,   // 超低价清仓
    };

    (player_value as f64 * multiplier) as i64
}
```

## 选手满意度系统

### 文件位置

`src-tauri/src/engines/satisfaction.rs`

### 满意度因素

| 因素 | 权重 | 影响 |
|------|------|------|
| 薪资满意度 | 30% | 实际薪资 vs 期望薪资 |
| 上场时间 | 25% | 首发 vs 替补 |
| 球队成绩 | 20% | 战队排名和荣誉 |
| 合同年限 | 15% | 剩余合同时长 |
| 球队战力 | 10% | 队友实力 |

### 满意度影响

```
满意度 > 80: 愿意降薪续约
满意度 60-80: 正常续约
满意度 40-60: 可能寻求转会
满意度 < 40: 主动要求交易
```

## 赛区偏好系统

### 偏好值范围

| 赛区 | 本土选手偏好值范围 |
|------|------------------|
| LPL | 75-90 |
| LCK | 55-75 |
| LEC | 45-65 |
| LCS | 40-60 |

### 跨赛区转会意愿

```rust
// 跨赛区转会意愿度
let willingness = base_willingness * (100 - region_loyalty) / 100;

// 例如：LPL 选手 (region_loyalty = 85)
// 跨赛区意愿 = 50 * (100 - 85) / 100 = 7.5 (非常低)
```

## API 接口

| 接口 | 描述 |
|------|------|
| `evaluate_offers_and_choose` | 选手评估报价 |
| `select_best_draft_player` | AI 选秀决策 |
| `generate_team_strategy` | 生成球队转会策略 |
| `calculate_player_satisfaction` | 计算选手满意度 |

## 文件位置

| 文件 | 说明 |
|-----|------|
| `src-tauri/src/engines/player_decision.rs` | 选手决策引擎 |
| `src-tauri/src/services/draft_ai_service.rs` | 选秀 AI 服务 |
| `src-tauri/src/engines/transfer/` | 转会引擎（含 AI 策略） |
| `src-tauri/src/engines/satisfaction.rs` | 满意度系统 |
