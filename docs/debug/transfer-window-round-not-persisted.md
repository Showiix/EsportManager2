# 转会期轮次未持久化 (current_round 始终为 0)

## 问题描述

执行转会轮次后，`transfer_windows.current_round` 始终为 0。重新加载存档后转会总览显示"第0轮"，需要重新开始。但 `transfer_events` 和 `player_listings` 数据正常写入。

## 根因

`src-tauri/src/engines/transfer/mod.rs` 第 365 行的 UPDATE 语句缺少 `.bind(window_id)`：

```rust
// 修复前 — 3 个占位符，只绑定了 2 个参数
sqlx::query("UPDATE transfer_windows SET current_round = ?, status = ? WHERE id = ?")
    .bind(round)
    .bind(new_status)
    // 缺少 .bind(window_id) ← 第三个 ? 未绑定，默认 NULL
    .execute(pool)
```

`WHERE id = NULL` 永远匹配不到任何行，UPDATE 执行成功但影响 0 行。sqlx 动态查询 (`query()`) 不会在编译期检查参数数量，运行时也不报错。

## 修复

```rust
// 修复后
sqlx::query("UPDATE transfer_windows SET current_round = ?, status = ? WHERE id = ?")
    .bind(round)
    .bind(new_status)
    .bind(window_id)  // ← 补上
    .execute(pool)
```

## 附带改进

| 文件 | 改动 |
|------|------|
| `transfer_commands.rs` | 新增 `get_current_transfer_window` 纯查询命令 |
| `useTransferWindowStore.ts` | `initTransferWindow` 改用纯查询，不再意外创建窗口 |
| `useGameStore.ts` | 加载存档时清除 transferWindowStore 状态 |

## 诊断方法

```sql
-- 查看窗口状态
SELECT id, current_round, status FROM transfer_windows;

-- 对比事件数据（如果事件有数据但 current_round=0，说明 UPDATE 未生效）
SELECT round, COUNT(*) FROM transfer_events WHERE window_id = ? GROUP BY round;
```
