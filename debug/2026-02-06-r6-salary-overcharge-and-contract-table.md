# R6 年薪扣错导致球队大面积破产

**日期**: 2026-02-06
**Commit**: `c1f662c`
**涉及文件**:
- `src-tauri/src/models/transfer.rs`
- `src-tauri/src/db/connection.rs`
- `src-tauri/src/engines/transfer.rs`
- `src-tauri/src/commands/draft_commands.rs`

## 问题描述

R6 财政调整直接按 `SUM(salary)` 扣薪，但 `salary` 存的是**总合同薪资**而非年薪，导致球队被扣数倍于实际的金额而大面积破产。同时 R6 破产后挂牌选手只剩 R7 一轮，来不及交易。

## 原因分析

1. `players.salary` 存储的是合同总薪资，但 R6 当作年薪直接扣除
2. `players` 表没有存储合同总年数，无法正确计算年薪
3. R6 破产挂牌后，R7 只做紧急补人不处理挂牌交易

## 修复方案

1. **新建 `player_contracts` 表**：记录每次签约事件，存储 `total_salary`、`annual_salary`、`contract_years` 等字段，迁移时回填初始数据（年薪 = 总薪资 / 合同总年数）
2. **R6 年薪改查合同表**：优先从 `player_contracts.annual_salary` 取值，fallback 用 `salary / (contract_end_season - join_season)` 估算
3. **各签约点写入合同记录**：R3 续约、R4 自由球员、R5 转会、R7 紧急签约、选秀签约均调用 `insert_contract`
4. **R7 复用 R5 竞价逻辑**：`execute_contracted_player_transfer` 新增 `round` 参数，R7 开头以 `round=7` 调用，处理 R6 破产挂牌选手

## 影响范围

- 转会系统 R3-R7 全部轮次
- 选秀系统
- 财政系统（年薪扣除逻辑）
- 数据库新增 `player_contracts` 表
