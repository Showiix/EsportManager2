# 转会系统未写入财务交易记录

**日期**: 2026-02-06
**Commit**: `eea4f74`
**涉及文件**:
- `src-tauri/src/engines/transfer.rs`
- `src-tauri/src/commands/finance_commands.rs`

## 问题描述

转会引擎在执行转会时直接修改 `teams.balance`，但从未写入 `financial_transactions` 表。导致：
1. 财政中心页面看不到任何转会相关的收支明细
2. 赛季财务报告中 `TransferIncome`/`TransferExpense` 查询永远返回 0

## 原因分析

两个问题叠加：
1. `transfer.rs` 中 R4（自由球员签约）和 R5（有合同挖角）的资金变动只更新了 `teams.balance`，未插入 `financial_transactions` 记录
2. `finance_commands.rs` 中查询使用了错误的类型字符串 `TransferIncome`/`TransferExpense`，而实际枚举值为 `TransferIn`/`TransferOut`

## 修复方案

1. **transfer.rs**: 新增 `record_financial_transaction` 辅助方法，在 3 处资金变动后补充 INSERT：
   - R4 自由球员签约：买方签约奖金支出 (`TransferOut`)
   - R5 有合同挖角：买方转会费支出 (`TransferOut`) + 卖方转会费收入 (`TransferIn`)
2. **finance_commands.rs**: 修正查询类型字符串 `TransferIncome` → `TransferIn`，`TransferExpense` → `TransferOut`

## 影响范围

- 财政中心页面的转会收支明细展示
- 赛季财务报告中的 `transfer_net` 计算
