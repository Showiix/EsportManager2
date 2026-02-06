# 转会窗口关闭机制

## 概述

转会窗口完成全部 7 轮后不再自动关闭，需要玩家手动确认关闭。关闭前系统会验证所有球队的阵容和合同状态，确保数据完整性。

## 关闭流程

```
R1-R7 执行完毕
    │
    ▼
前端显示"确认关闭转会窗口"按钮
    │
    ▼
调用 confirm_close_transfer_window(window_id, force=false)
    │
    ▼
后端执行验证检查
    │
    ├─ 全部通过 → 标记 COMPLETED → 返回 is_valid=true
    │
    └─ 存在问题 → 返回 is_valid=false + issues 列表
         │
         ├─ 用户选择"强制关闭" → 调用 force=true → 标记 COMPLETED
         │
         └─ 用户选择"返回修复" → 不做操作
```

## 验证检查项

### 阵容人数检查

| issue_type | 触发条件 | 说明 |
|-----------|----------|------|
| `ROSTER_TOO_SMALL` | 活跃选手 < 5 | 每队至少需要 5 名首发 |
| `ROSTER_TOO_LARGE` | 活跃选手 > 10 | 每队最多 10 名选手 |

查询逻辑：
```sql
SELECT COUNT(*) FROM players WHERE team_id = ? AND status = 'Active'
```

### 合同有效性检查

| issue_type | 触发条件 | 说明 |
|-----------|----------|------|
| `INVALID_CONTRACT` | contract_end_season <= 当前赛季 | 合同已过期的选手不应存在 |

查询逻辑：
```sql
SELECT COUNT(*) FROM players
WHERE team_id = ? AND status = 'Active' AND contract_end_season <= ?
```

## 数据结构

### TransferWindowCloseValidation

```rust
pub struct TransferWindowCloseValidation {
    pub is_valid: bool,        // 验证是否通过
    pub window_id: i64,        // 转会窗口 ID
    pub issues: Vec<TransferCloseIssue>,  // 问题列表
    pub message: String,       // 提示消息
}
```

### TransferCloseIssue

```rust
pub struct TransferCloseIssue {
    pub team_id: i64,          // 球队 ID
    pub team_name: String,     // 球队名称
    pub issue_type: String,    // 问题类型
    pub detail: String,        // 问题详情
}
```

## 前置条件

调用关闭验证前必须满足：

1. 转会窗口状态为 `IN_PROGRESS`
2. 当前轮次已达到 `max_rounds`（7）
3. 如窗口已为 `COMPLETED`，直接返回成功

不满足条件时返回错误，不执行验证。

## 与时间引擎的集成

### 阶段状态判断

`game_flow.rs` 的 `get_phase_progress` 根据数据库查询转会窗口实际状态：

```
无记录        → total=0, completed=0 → PhaseStatus::NotInitialized
IN_PROGRESS   → total=1, completed=0 → PhaseStatus::InProgress
COMPLETED     → total=1, completed=1 → PhaseStatus::Completed
```

### 可用操作

| PhaseStatus | 可用操作 |
|-------------|---------|
| NotInitialized | StartTransferWindow |
| InProgress | ExecuteTransferRound |
| Completed | CompleteAndAdvance |

### 推进保护

`complete_and_advance` 在转会期阶段会额外验证：

```rust
if current_phase == SeasonPhase::TransferWindow {
    // 查询 transfer_windows 状态
    // 非 COMPLETED 则拒绝推进
}
```

## API 接口

### 后端命令

```rust
#[tauri::command]
pub async fn confirm_close_transfer_window(
    state: State<'_, AppState>,
    window_id: i64,
    force: Option<bool>,  // 默认 false
) -> Result<CommandResult<TransferWindowCloseValidation>, String>
```

### 前端调用

```typescript
// 正常关闭（验证不通过则不关闭）
const result = await transferWindowApi.confirmCloseTransferWindow(windowId)

// 强制关闭（跳过验证问题）
const result = await transferWindowApi.confirmCloseTransferWindow(windowId, true)
```

### Store

```typescript
// computed：是否等待关闭确认
const isAwaitingClose = computed(() =>
  windowInfo.value?.status === 'IN_PROGRESS' && windowInfo.value?.current_round >= 7
)

// action：确认关闭
async function confirmCloseWindow(force: boolean): Promise<TransferWindowCloseValidation>
```

## 前端交互

### 按钮显示逻辑

| 条件 | 显示按钮 |
|------|---------|
| IN_PROGRESS 且 round < 7 | "执行下一轮" + "快进完成" |
| IN_PROGRESS 且 round >= 7 | "确认关闭转会窗口" |
| COMPLETED | 无操作按钮 |

### 验证失败弹窗

验证未通过时，使用 `ElMessageBox.confirm` 展示问题列表，提供两个选项：

- **确认按钮**："强制关闭" — 调用 `confirmCloseWindow(true)`
- **取消按钮**："返回修复" — 不做操作

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/transfer.rs` | `validate_and_close_window` 方法 |
| `src-tauri/src/models/transfer.rs` | 验证结构体定义 |
| `src-tauri/src/commands/transfer_commands.rs` | `confirm_close_transfer_window` 命令 |
| `src-tauri/src/services/game_flow.rs` | 阶段状态判断 + 推进验证 |
| `src/api/tauri.ts` | 前端 API + 类型定义 |
| `src/stores/useTransferWindowStore.ts` | Store action + computed |
| `src/views/TransferWindow.vue` | 关闭按钮 + 弹窗交互 |
