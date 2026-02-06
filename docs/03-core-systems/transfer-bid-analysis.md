# 转会竞价分析系统

## 概述

竞价分析系统记录 R4（自由球员竞标）和 R5（有合同选手挖角）中所有球队的竞价数据，包括出价金额、匹配度、意愿度，并通过独立页面展示完整竞价过程。

## 背景

原有 R4/R5 逻辑只保留最终签约结果，无法看到有哪些球队出价、出了多少、选手对各球队的意愿分数。本系统解决三个问题：

1. 持久化所有竞价数据（不只保留赢家）
2. 对所有竞标方计算 willingness（而非找到第一个通过即停止）
3. 前端独立页面展示完整竞价过程

## 核心概念

| 概念 | 说明 |
|------|------|
| `match_score` | 球队对选手的匹配度（0-100），由 AI 性格权重计算 |
| `willingness` | 选手对报价的意愿度（0-100），>= 40 才会接受 |
| `reject_reason` | 被拒原因：`willingness_too_low`（意愿不足）/ `outbid`（被更高者抢先） |
| `is_winner` | 是否为最终中标方 |

## 数据结构

### transfer_bids 表

```sql
CREATE TABLE transfer_bids (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    window_id INTEGER NOT NULL,       -- 转会窗口 ID
    round INTEGER NOT NULL,           -- 4（R4）或 5（R5）
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    player_ability INTEGER NOT NULL,
    player_age INTEGER NOT NULL,
    player_position TEXT,
    from_team_id INTEGER,             -- R4=NULL（自由球员），R5=卖方 ID
    from_team_name TEXT,
    bid_team_id INTEGER NOT NULL,     -- 竞标球队 ID
    bid_team_name TEXT NOT NULL,
    bid_team_region_id INTEGER,
    offered_salary INTEGER NOT NULL,  -- 报价年薪（元）
    contract_years INTEGER NOT NULL,
    transfer_fee INTEGER NOT NULL,    -- R4=0，R5=转会费出价
    signing_bonus INTEGER NOT NULL,
    match_score REAL NOT NULL,        -- 球队匹配度
    willingness REAL NOT NULL,        -- 选手意愿度
    is_winner INTEGER NOT NULL,       -- 是否中标
    reject_reason TEXT,               -- 被拒原因
    created_at TEXT NOT NULL
);
```

索引：`window_id`、`(window_id, round)`、`player_id`。

### Rust 模型

```rust
pub struct TransferBid { ... }           // 单条竞价记录
pub struct PlayerBidAnalysis { ... }     // 单个选手的所有竞价 + 结果
pub struct BidOverview { ... }           // 汇总统计 + 所有选手分析列表
```

## 核心流程

### R4 自由球员竞标

```
收集所有球队的报价 (offers)
    ↓
按 match_score 降序排列
    ↓
对 **所有** offers 计算 willingness
    ↓
选出 best_offer（match_score 最高且 willingness >= 40）
    ↓
标记每个 offer 的结果：
  - winner:              reject_reason = NULL
  - willingness < 40:    reject_reason = "willingness_too_low"
  - 被更高者抢先:         reject_reason = "outbid"
    ↓
全部写入 transfer_bids 表
    ↓
执行签约（如有赢家）
```

### R5 有合同选手挖角

```
收集所有通过筛选的竞标（出价 >= 最低接受价）
    ↓
按 bid_price 降序排列
    ↓
对 **所有** 竞标计算 willingness
    ↓
遍历排序后的竞标，第一个 willingness >= 40 的中标
  （允许次高出价中标 — 最高出价被拒后由次高出价者接替）
    ↓
标记结果并全部写入 transfer_bids 表
    ↓
执行转会（如有赢家）
```

**R5 行为变更**：原来最高出价被拒即整个交易流产，现在允许回退到次高出价者。

### 意愿度计算公式

```
salary_score = 基于 offered_salary / current_salary 的分段评分
loyalty_impact = (100 - loyalty) * 0.5
base = salary_score * 0.4 + loyalty_impact * 0.3 + 50 * 0.3 + random(-5, 5)

跨赛区惩罚:
  cross_region_factor = (100 - region_loyalty) / 100  （跨赛区时）
  cross_region_factor = 1.0                           （本赛区时）

willingness = base * cross_region_factor
```

意愿度阈值：**>= 40** 才接受签约。

## API 接口

| 接口 | 参数 | 描述 |
|------|------|------|
| `get_transfer_bids_overview` | `window_id`, `round?` | 获取竞价总览（按选手分组） |
| `get_player_bids` | `window_id`, `player_id` | 获取单个选手的所有竞价记录 |

### get_transfer_bids_overview

- `round = None`：返回 R4 + R5 全部数据
- `round = Some(4)`：仅返回 R4 自由球员竞价
- `round = Some(5)`：仅返回 R5 有合同挖角竞价

返回 `BidOverview`：

| 字段 | 说明 |
|------|------|
| `total_players` | 涉及选手数 |
| `total_bids` | 总出价数 |
| `successful_signings` | 成功签约数 |
| `failed_signings` | 竞价失败数 |
| `avg_bids_per_player` | 平均每位选手的竞标数 |
| `player_analyses` | 每位选手的竞价详情列表 |

## 前端页面

### 路由

```
/transfer/bid-analysis?windowId=xxx&seasonId=xxx
```

### 入口

TransferWindow.vue 中，R4/R5 完成后（`currentRound >= 5` 或转会期完成）显示"竞价分析"按钮。

### 页面结构

```
┌─────────────────────────────────────────────────┐
│ 竞价分析中心 · S{season} 赛季                     │
├──────┬──────┬──────┬──────┐                      │
│涉及选手│总出价数│成功签约│竞价失败│  统计卡片         │
├──────┴──────┴──────┴──────┘                      │
│ [全部/R4/R5] [搜索] [位置] [结果]  筛选栏          │
├─────────────────────────────────────────────────┤
│ 选手列表（可展开行）                               │
│ | 选手 | 位置 | 能力 | 年龄 | 来源 | 竞标数 | 结果 │
│ └─展开: 该选手所有竞价方详情                       │
│   | 排名 | 球队 | 薪资 | 合同 | 转会费              │
│   | 匹配度 | 意愿度(进度条) | 结果标签              │
└─────────────────────────────────────────────────┘
```

### 意愿度可视化

- `willingness >= 40`：绿色进度条 + 绿色数值
- `willingness < 40`：红色进度条 + 红色数值

### 结果标签

| 标签 | 颜色 | 含义 |
|------|------|------|
| 签约成功 | 绿 | `is_winner = true` |
| 意愿不足 | 红 | `reject_reason = "willingness_too_low"` |
| 被抢先 | 灰 | `reject_reason = "outbid"` |

## 向后兼容

已有存档的 `transfer_bids` 表在迁移时自动创建为空表。竞价分析页面会显示"暂无竞价数据，从下个转会期开始记录"。

## 性能评估

- R4：56 支球队 x ~30 个自由球员 = ~1680 次 willingness 计算（纯算术运算）
- R5：挂牌选手通常 10-20 个，每个 3-10 个竞标方
- 每转会期写入量：~500-2000 条，SQLite 无压力

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/db/connection.rs` | `transfer_bids` 建表迁移 |
| `src-tauri/src/models/transfer.rs` | `TransferBid`、`PlayerBidAnalysis`、`BidOverview` 结构体 |
| `src-tauri/src/engines/transfer.rs` | R4/R5 竞价数据收集 + `insert_bid` 方法 |
| `src-tauri/src/commands/transfer_commands.rs` | `get_transfer_bids_overview`、`get_player_bids` 命令 |
| `src/api/tauri.ts` | 前端 TS 类型 + API 函数 |
| `src/views/TransferBidAnalysis.vue` | 竞价分析页面 |
| `src/views/TransferWindow.vue` | 竞价分析入口按钮 |
| `src/router/index.ts` | `/transfer/bid-analysis` 路由 |
