---
name: financial-system
description: 电竞经理游戏的财政系统。管理球队财务、赛事奖金、赞助收入、薪资支出、转会预算。当需要修改财务规则、奖金配置、财务报告时使用此技能。
---

# 财政系统 (Financial System)

## Overview

财政系统管理游戏中所有球队的财务状况，包括赛事奖金分配、赞助收入计算、薪资支出管理、转会预算建议等。它是转会系统决策的重要参考依据。

## 核心组件

### FinancialEngine

**文件**: `src-tauri/src/engines/financial.rs`

财务引擎，管理所有财务相关计算和交易。

```rust
pub struct FinancialEngine {
    config: FinancialConfig,
}
```

**核心方法**:
- `calculate_prize_money(...)` - 计算赛事奖金
- `calculate_sponsorship(...)` - 计算赞助收入
- `calculate_league_share()` - 计算联赛分成
- `calculate_operating_cost()` - 计算运营成本
- `generate_season_report(...)` - 生成赛季财务报告
- `distribute_prize_money(...)` - 分配比赛奖金

## 财务配置

### FinancialConfig

```rust
pub struct FinancialConfig {
    pub base_operating_cost: u64,   // 基础运营成本: 300万/赛季
    pub prize_pools: HashMap<TournamentType, PrizePool>,
    pub league_revenue_share: u64,  // 联赛分成: 150万/赛季
    pub sponsorship_coefficient: f64, // 赞助系数: 2.0
}
```

## 奖金池配置

### 国际赛事奖金

| 赛事 | 总奖金 | 冠军 | 亚军 | 季军 | 殿军 |
|------|--------|------|------|------|------|
| **MSI** | 4000万 | 2000万 (50%) | 1000万 (25%) | 500万 (12.5%) | 200万 (5%) |
| **世界赛** | 1.2亿 | 5000万 (41.7%) | 2500万 (20.8%) | 1200万 (10%) | 1200万 (10%) |
| **Super洲际赛** | 1.5亿 | 6000万 (40%) | 3000万 (20%) | 1500万 (10%) | 1500万 (10%) |
| **马德里大师赛** | 2000万 | 800万 (40%) | 400万 (20%) | 200万 (10%) | 200万 (10%) |
| **Claude洲际赛** | 2000万 | 800万 (40%) | 400万 (20%) | 200万 (10%) | 200万 (10%) |
| **上海大师赛** | 2500万 | 1000万 (40%) | 500万 (20%) | 250万 (10%) | 250万 (10%) |
| **ICP洲际对抗赛** | 3000万 | 1200万 (40%) | 600万 (20%) | 300万 (10%) | 300万 (10%) |

### 赛区赛事奖金

| 赛事 | 总奖金 | 冠军 | 亚军 | 季军 | 殿军 | 5-8名 |
|------|--------|------|------|------|------|-------|
| **春季季后赛** | 200万 | 70万 (35%) | 50万 (25%) | 30万 (15%) | 20万 (10%) | 8万 (4%/队) |
| **夏季季后赛** | 200万 | 70万 (35%) | 50万 (25%) | 30万 (15%) | 20万 (10%) | 8万 (4%/队) |

## 数据结构

### TeamSeasonFinance (赛季财务)

```rust
pub struct TeamSeasonFinance {
    pub id: u64,
    pub team_id: u64,
    pub season_id: u64,
    pub opening_balance: i64,     // 期初余额
    pub closing_balance: i64,     // 期末余额
    pub total_income: u64,        // 总收入
    pub total_expense: u64,       // 总支出
    pub financial_status: FinancialStatus,
    pub salary_cap_used: u64,     // 薪资支出
}
```

### FinancialStatus (财务状态)

```rust
pub enum FinancialStatus {
    Wealthy,   // 富裕 (余额 > 1000万)
    Healthy,   // 健康 (500-1000万)
    Tight,     // 紧张 (100-500万)
    Deficit,   // 赤字 (0-100万)
    Bankrupt,  // 破产 (< 0)
}
```

### FinancialTransaction (财务交易)

```rust
pub struct FinancialTransaction {
    pub id: u64,
    pub save_id: String,
    pub team_id: u64,
    pub season_id: u64,
    pub transaction_type: TransactionType,
    pub amount: i64,              // 正数收入，负数支出
    pub description: Option<String>,
    pub related_player_id: Option<u64>,
    pub related_tournament_id: Option<u64>,
}
```

### TransactionType (交易类型)

```rust
pub enum TransactionType {
    // 收入类型
    Sponsorship,          // 赞助收入
    LeagueShare,          // 联赛分成
    PlayoffBonus,         // 季后赛奖金
    InternationalBonus,   // 国际赛奖金
    TransferIn,           // 转会收入

    // 支出类型
    Salary,               // 薪资
    OperatingCost,        // 运营成本
    TransferOut,          // 转会支出
    Fine,                 // 罚款
}
```

### FinancialStatusSummary (财务摘要)

```rust
pub struct FinancialStatusSummary {
    pub team_id: u64,
    pub balance: i64,
    pub is_crisis: bool,
    pub transfer_budget: i64,
    pub max_new_salary: u64,
    pub projected_season_profit: i64,
}
```

## 收入计算

### 赞助收入

基于球队战力和胜率计算：

```rust
fn calculate_sponsorship(&self, team: &Team) -> u64 {
    let base = match team.power_rating as u32 {
        90..=100 => 200,  // 顶级队伍
        85..=89 => 150,
        80..=84 => 120,
        75..=79 => 90,
        70..=74 => 70,
        65..=69 => 50,
        _ => 30,
    };

    let win_rate_bonus = if team.win_rate > 0.7 {
        1.5
    } else if team.win_rate > 0.5 {
        1.2
    } else {
        1.0
    };

    (base as f64 * win_rate_bonus * 2.0) as u64  // ×赞助系数
}
```

### 联赛分成

固定金额：**150万/赛季**

### 奖金收入

```rust
fn calculate_prize_money(&self, tournament_type: TournamentType, position: &str) -> u64 {
    if let Some(pool) = self.config.prize_pools.get(&tournament_type) {
        if let Some(&percentage) = pool.distribution.get(position) {
            return (pool.total as f64 * percentage) as u64;
        }
    }
    0
}
```

## 支出计算

### 运营成本

固定金额：**300万/赛季**

### 薪资支出

```rust
fn calculate_salary_expense(players: &[(u64, u64)]) -> u64 {
    players.iter().map(|(_, salary)| salary).sum()
}
```

## 财务决策

### 财务危机判定

```rust
fn is_in_financial_crisis(&self, team: &Team) -> bool {
    // 余额低于运营成本50%视为财务危机
    team.balance < (300 * 10000 / 2)  // < 150万
}
```

### 转会预算建议

```rust
fn suggest_transfer_budget(&self, team: &Team) -> i64 {
    if self.is_in_financial_crisis(team) {
        0
    } else {
        (team.balance as f64 * 0.3) as i64  // 建议使用30%余额
    }
}
```

### 最大可承受薪资

```rust
fn max_affordable_salary(&self, team: &Team, current_salary_total: u64) -> u64 {
    let projected_income = sponsorship + league_share;
    let max_salary_budget = (projected_income as f64 * 0.6) as u64;  // 不超过收入60%

    if max_salary_budget > current_salary_total {
        max_salary_budget - current_salary_total
    } else {
        0
    }
}
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/finance_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `get_team_finance_summary` | 获取球队财务摘要 | `TeamFinanceSummary` |
| `get_all_teams_finance` | 获取所有球队财务 | `Vec<TeamFinanceSummary>` |
| `get_team_transactions` | 获取交易记录 | `Vec<FinancialTransaction>` |
| `record_transaction` | 记录交易 | `bool` |
| `get_season_finance_report` | 获取赛季报告 | `TeamSeasonFinance` |
| `pay_team_salaries` | 支付薪资 | `bool` |
| `distribute_league_share` | 发放联赛分成 | `bool` |
| `get_prize_pool_info` | 获取奖金池信息 | `PrizePool` |
| `distribute_tournament_prizes` | 发放赛事奖金 | `i32` |
| `get_team_prize_details` | 获取奖金明细 | `Vec<PrizeDetail>` |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 获取球队财务摘要
export async function getTeamFinanceSummary(teamId: number): Promise<TeamFinanceSummary>

// 获取所有球队财务
export async function getAllTeamsFinance(): Promise<TeamFinanceSummary[]>

// 获取交易记录
export async function getTeamTransactions(teamId: number, seasonId?: number): Promise<FinancialTransaction[]>

// 获取赛季财务报告
export async function getSeasonFinanceReport(teamId: number, seasonId: number): Promise<TeamSeasonFinance>

// 获取奖金池信息
export async function getPrizePoolInfo(tournamentType: string): Promise<PrizePool>

// 发放赛事奖金
export async function distributeTournamentPrizes(tournamentId: number): Promise<number>
```

## 系统集成

### 与时间推进系统
- 赛季结算时发放联赛分成
- 赛事完成时发放奖金

### 与转会系统
- 转会预算影响引援能力
- 财务危机触发强制出售
- 薪资上限限制签约

### 与选手系统
- 选手薪资计入支出
- 转会费影响收支

### 与荣誉系统
- 冠军奖金影响财务

## 使用示例

### 计算赛事奖金
```rust
let engine = FinancialEngine::new();

let champion_prize = engine.calculate_prize_money(TournamentType::WorldChampionship, "CHAMPION");
println!("世界赛冠军奖金: {}万元", champion_prize / 10000);
// 输出: 世界赛冠军奖金: 5000万元
```

### 获取财务状态
```rust
let status = engine.get_financial_status(&team, current_salary);

if status.is_crisis {
    println!("警告: 财务危机!");
} else {
    println!("转会预算: {}万", status.transfer_budget / 10000);
    println!("可新增薪资: {}万", status.max_new_salary / 10000);
}
```

### 生成赛季报告
```rust
let report = engine.generate_season_report(
    &team,
    season_id,
    salary_expense,
    prize_money,
    transfer_income,
);

println!("期初余额: {}万", report.opening_balance / 10000);
println!("总收入: {}万", report.total_income / 10000);
println!("总支出: {}万", report.total_expense / 10000);
println!("期末余额: {}万", report.closing_balance / 10000);
println!("财务状态: {:?}", report.financial_status);
```

### 分配奖金
```typescript
// 发放世界赛奖金
const count = await distributeTournamentPrizes(worldsTournamentId)
console.log(`已向${count}支队伍发放奖金`)
```

## 数据库表

### financial_transactions

```sql
CREATE TABLE financial_transactions (
    id INTEGER PRIMARY KEY,
    save_id TEXT NOT NULL,
    team_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    transaction_type TEXT NOT NULL,
    amount INTEGER NOT NULL,
    description TEXT,
    related_player_id INTEGER,
    related_tournament_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### team_season_finance

```sql
CREATE TABLE team_season_finance (
    id INTEGER PRIMARY KEY,
    team_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    opening_balance INTEGER,
    closing_balance INTEGER,
    total_income INTEGER,
    total_expense INTEGER,
    financial_status TEXT,
    salary_cap_used INTEGER,
    UNIQUE(team_id, season_id)
);
```

## 注意事项

1. **金额单位**: 内部计算使用**元**，显示时转换为**万元**
2. **赛季结算**: 需在赛季末调用 `generate_season_report` 生成报告
3. **交易记录**: 每笔收支都应调用 `record_transaction` 记录
4. **财务危机**: 余额低于150万触发危机，转会系统会强制出售选手
5. **薪资上限**: 薪资不应超过预计收入的60%
