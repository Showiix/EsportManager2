---
name: financial-units
description: 电竞经理游戏的金额单位规范。全项目金额统一以「元」存储和传递，前端使用 formatMoney 系列函数格式化显示。当需要新增金额相关计算、修改身价/薪资/奖金/赞助公式、添加新的金额字段、或排查金额显示异常时使用此技能。
---

# 金额单位规范

## 核心规则

- **存储/传递单位**: 元（所有后端函数、数据库字段、API 返回值）
- **显示格式**: 前端 `formatMoney()` 自动转换（>=1亿显示亿，>=1万显示万，否则显示元）
- **禁止**: 函数返回万元。所有计算结果必须是元

## 后端金额计算函数清单

| 函数 | 文件 | 说明 |
|------|------|------|
| `calculate_initial_salary()` | `services/init_service.rs` | 初始化薪资，返回元 |
| `calculate_market_value()` | `services/init_service.rs` | 初始化身价，返回元 |
| `calculate_base_market_value()` | `models/player.rs` | 基础身价计算，返回元 |
| `calculate_full_market_value()` | `models/player.rs` | 含荣誉系数的完整身价，返回元 |
| `calculate_base_market_value()` | `services/game_flow/market_value.rs` | 赛季结算身价重算，返回元 |
| `calculate_market_value()` | `commands/dev_commands.rs` | 开发工具身价重算，返回元 |
| `calculate_sponsorship()` | `engines/financial.rs` | 赞助收入，返回元 |

## 财务阈值（元）

财务状态判定使用 `FinancialStatus::from_balance(balance)`（定义在 `models/team.rs`）：

```
> 10_000_000 → Wealthy
> 5_000_000  → Healthy
> 1_000_000  → Tight
>= 0         → Deficit
< 0          → Bankrupt
```

## 财务配置默认值（元）

定义在 `models/financial.rs` 的 `FinancialConfig::default()`：

| 字段 | 值（元） |
|------|---------|
| `league_share` | 5_000_000 |
| `operating_cost` | 1_000_000 |
| `salary_cap` | 15_000_000 |
| `individual_salary_cap` | 4_000_000 |

奖金配置（`ranking_bonus_config`、`playoff_bonus_config`、`international_bonus_config`）同样以元为单位。

## 数据库字段（元）

| 表 | 字段 | 单位 |
|----|------|------|
| `players` | `salary` | 元 |
| `players` | `market_value` | 元 |
| `players` | `calculated_market_value` | 元 |
| `player_transfer_strategies` | `expected_salary` | 元 |
| `player_transfer_strategies` | `expected_min_salary` | 元 |
| `financial_transactions` | `amount` | 元 |
| `teams` | `balance` | 元 |

## 前端格式化

所有金额显示统一使用 `src/utils/format.ts` 中的函数：

```typescript
formatMoney(amount)           // 通用：自动选单位（元/万/亿）
formatValue(value)            // 身价专用
formatSalary(salary)          // 薪资专用，带"/年"后缀
formatTransferFee(fee)        // 转会费专用
formatBudget(amount)          // 预算/余额，支持负数
```

- `formatMoneyFromWan()` 已标记 `@deprecated`，**禁止在新代码中使用**
- 输入单位始终为元，函数内部自动转换显示

## 新增金额字段检查清单

添加新的金额相关功能时：

1. 后端计算函数返回值单位必须是元
2. 数据库字段存储元
3. 前端使用 `formatMoney` 系列函数显示
4. 测试用例使用元（如 `10_000_000` 而非 `1000`）
5. 日志中如需显示万，用 `amount / 10000` 转换

## 数据库迁移模式

旧数据（万元）迁移到新数据（元）的模式参考 `db/connection.rs` 中的 `run_unify_money_to_yuan_migration`：

1. 创建 `schema_migrations` 表记录迁移状态
2. 检测旧数据：`MAX(salary) < 10000 AND MAX(salary) > 0`
3. 批量更新：`SET salary = salary * 10000`
4. 有条件更新：`SET calculated_market_value = calculated_market_value * 10000 WHERE calculated_market_value < 1000000`
5. 标记迁移完成
