# 开发者工具系统增强 - 实现计划

## 1. 项目背景

### 1.1 核心痛点
- 游戏模拟出现问题时，需要删除整个数据库才能恢复
- 无法回退到之前的游戏状态
- 调试时需要频繁重新开始

### 1.2 解决方案
1. **时间回滚系统** - 支持回退到任意历史阶段
2. **数据快照系统** - 随时保存/恢复游戏状态
3. **增强 DevTools UI** - 更直观的操作界面

---

## 2. 现有系统分析

### 2.1 赛季阶段定义 (SeasonPhase)

**文件**: `src-tauri/src/models/season.rs`

```rust
pub enum SeasonPhase {
    SpringRegular,          // 1. 春季赛常规赛
    SpringPlayoffs,         // 2. 春季赛季后赛
    Msi,                    // 3. MSI季中赛
    MadridMasters,          // 4. 马德里大师赛
    SummerRegular,          // 5. 夏季赛常规赛
    SummerPlayoffs,         // 6. 夏季赛季后赛
    ClaudeIntercontinental, // 7. Claude洲际赛
    WorldChampionship,      // 8. S世界赛
    ShanghaiMasters,        // 9. 上海大师赛
    IcpIntercontinental,    // 10. ICP四赛区洲际对抗赛
    SuperIntercontinental,  // 11. Super洲际年度邀请赛
    AnnualAwards,           // 12. 年度颁奖典礼
    TransferWindow,         // 13. 转会期
    Draft,                  // 14. 选秀（每4年：S2, S6, S10...）
    SeasonEnd,              // 15. 赛季结束
}
```

### 2.2 需要清理的数据表

回滚时需要删除目标阶段之后的数据：

| 表名 | 说明 | 关联字段 |
|------|------|---------|
| `tournaments` | 赛事 | `season_id`, `tournament_type` |
| `matches` | 比赛 | `tournament_id` |
| `match_games` | 单局比赛 | `match_id` |
| `game_player_performances` | 选手表现 | `game_id` |
| `league_standings` | 积分榜 | `tournament_id` |
| `tournament_results` | 赛事结果 | `tournament_id` |
| `honors` | 荣誉记录 | `tournament_id`, `season_id` |
| `annual_points_detail` | 年度积分明细 | `season_id`, `tournament_id` |
| `player_season_stats` | 选手赛季统计 | `season_id` |
| `player_tournament_stats` | 选手赛事统计 | `tournament_id` |
| `financial_transactions` | 财务交易 | `season_id` |
| `transfer_events` | 转会事件 | `season_id` |

### 2.3 现有时间系统 API

**后端命令** (`src-tauri/src/commands/time_commands.rs`):
- `get_time_state` - 获取当前时间状态
- `time_init_phase` - 初始化阶段
- `complete_and_advance` - 完成并推进
- `fast_forward_to` - 快进到指定阶段

**前端接口** (`src/api/tauri.ts`):
```typescript
export const timeApi = {
  getTimeState: () => invokeCommand<GameTimeState>('get_time_state'),
  initPhase: () => invokeCommand<string>('time_init_phase'),
  completeAndAdvance: () => invokeCommand<CompleteAndAdvanceResult>('complete_and_advance'),
  fastForwardTo: (target: string) => invokeCommand<FastForwardResult>('fast_forward_to', { target }),
  // ...
}
```

### 2.4 现有开发者工具

**后端命令** (`src-tauri/src/commands/dev_commands.rs`):
- 荣誉系统: `dev_reassign_honors`, `dev_recalculate_annual_points`
- 数据修复: `dev_sync_player_games_played`, `dev_recalculate_standings`, `dev_check_data_consistency`
- 赛事管理: `dev_reset_phase`, `dev_simulate_all_matches`
- 财务系统: `dev_redistribute_prizes`, `dev_grant_funds`
- 快速测试: `dev_reset_save`, `dev_get_game_status`

---

## 3. 新增功能设计

### 3.1 时间回滚系统

#### 3.1.1 后端命令

**新增命令**: `dev_rollback_to_phase`

**文件**: `src-tauri/src/commands/dev_commands.rs`

```rust
/// 回滚到指定阶段
/// 删除目标阶段之后的所有数据，将游戏状态重置到该阶段开始
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_rollback_to_phase(
    state: State<'_, AppState>,
    target_phase: String,
    target_season: Option<u32>,
) -> Result<DevCommandResult<RollbackResult>, String>
```

**返回数据结构**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub target_season: u32,
    pub target_phase: String,
    pub deleted_tournaments: u32,
    pub deleted_matches: u32,
    pub deleted_honors: u32,
    pub message: String,
}
```

**回滚逻辑**:
1. 解析目标阶段 (`target_phase` -> `SeasonPhase`)
2. 获取目标阶段对应的 `TournamentType` 列表
3. 删除目标阶段及之后阶段的赛事和关联数据
4. 重置 `saves` 表的 `current_phase` 和 `phase_completed`
5. 返回删除统计

#### 3.1.2 阶段与赛事类型映射

```rust
fn get_phases_after(phase: SeasonPhase) -> Vec<SeasonPhase> {
    // 返回 phase 之后的所有阶段（不包含 phase 本身）
}

fn phase_to_tournament_types(phase: SeasonPhase) -> Vec<&'static str> {
    match phase {
        SeasonPhase::SpringRegular => vec!["SpringRegular"],
        SeasonPhase::SpringPlayoffs => vec!["SpringPlayoffs"],
        SeasonPhase::Msi => vec!["Msi"],
        SeasonPhase::MadridMasters => vec!["MadridMasters"],
        // ... 其他映射
    }
}
```

#### 3.1.3 前端接口

**新增 API** (`src/api/tauri.ts`):
```typescript
export interface RollbackResult {
  target_season: number
  target_phase: string
  deleted_tournaments: number
  deleted_matches: number
  deleted_honors: number
  message: string
}

// devApi 中新增
rollbackToPhase: (targetPhase: string, targetSeason?: number) =>
  invokeCommand<DevCommandResult<RollbackResult>>('dev_rollback_to_phase', { targetPhase, targetSeason }),
```

---

### 3.2 数据快照系统

#### 3.2.1 后端命令

**新增命令**:

1. `dev_create_snapshot` - 创建快照
2. `dev_restore_snapshot` - 恢复快照
3. `dev_list_snapshots` - 列出快照
4. `dev_delete_snapshot` - 删除快照

**文件**: `src-tauri/src/commands/snapshot_commands.rs` (新建)

```rust
/// 快照信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,           // 唯一ID (时间戳)
    pub name: String,         // 用户命名
    pub created_at: String,   // 创建时间
    pub season: u32,          // 赛季
    pub phase: String,        // 阶段
    pub file_size: u64,       // 文件大小 (bytes)
    pub team_count: u32,      // 战队数
    pub player_count: u32,    // 选手数
}

/// 创建快照
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_create_snapshot(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    name: String,
) -> Result<DevCommandResult<SnapshotInfo>, String>

/// 恢复快照
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_restore_snapshot(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    snapshot_id: String,
) -> Result<DevCommandResult<()>, String>

/// 列出快照
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_list_snapshots(
    app_handle: tauri::AppHandle,
) -> Result<DevCommandResult<Vec<SnapshotInfo>>, String>

/// 删除快照
#[tauri::command(rename_all = "camelCase")]
pub async fn dev_delete_snapshot(
    app_handle: tauri::AppHandle,
    snapshot_id: String,
) -> Result<DevCommandResult<()>, String>
```

**快照存储**:
- 目录: `{app_data}/snapshots/`
- 文件名: `{timestamp}_{name}.db`
- 元数据: `{timestamp}_{name}.json`

#### 3.2.2 前端接口

```typescript
export interface SnapshotInfo {
  id: string
  name: string
  created_at: string
  season: number
  phase: string
  file_size: number
  team_count: number
  player_count: number
}

// devApi 中新增
createSnapshot: (name: string) =>
  invokeCommand<DevCommandResult<SnapshotInfo>>('dev_create_snapshot', { name }),

restoreSnapshot: (snapshotId: string) =>
  invokeCommand<DevCommandResult<void>>('dev_restore_snapshot', { snapshotId }),

listSnapshots: () =>
  invokeCommand<DevCommandResult<SnapshotInfo[]>>('dev_list_snapshots'),

deleteSnapshot: (snapshotId: string) =>
  invokeCommand<DevCommandResult<void>>('dev_delete_snapshot', { snapshotId }),
```

---

### 3.3 DevTools.vue UI 增强

#### 3.3.1 新增组件

1. **TimelineControl.vue** - 时间轴控制组件
   - 可视化15个阶段进度
   - 支持点击快进/回滚
   - 显示当前位置

2. **SnapshotManager.vue** - 快照管理组件
   - 创建快照（输入名称）
   - 快照列表（显示详情）
   - 恢复/删除操作

#### 3.3.2 布局重构

```
DevTools.vue
├── 页面头部 (当前状态)
├── Tab 切换
│   ├── 时间控制 (新增)
│   │   ├── TimelineControl.vue
│   │   └── 快进/回滚按钮
│   ├── 快照管理 (新增)
│   │   └── SnapshotManager.vue
│   ├── 数据修复 (现有)
│   ├── 财务工具 (现有)
│   └── 危险操作 (现有)
└── 执行日志
```

---

## 4. 实现步骤

### 第一阶段：后端核心功能

1. **[dev_commands.rs]** 添加 `dev_rollback_to_phase` 命令
   - 实现阶段映射函数
   - 实现数据删除逻辑
   - 实现状态重置逻辑

2. **[snapshot_commands.rs]** 新建快照命令模块
   - 实现 `dev_create_snapshot`
   - 实现 `dev_restore_snapshot`
   - 实现 `dev_list_snapshots`
   - 实现 `dev_delete_snapshot`

3. **[mod.rs]** 注册新命令

4. **[main.rs]** 添加命令到 invoke_handler

### 第二阶段：前端 API

5. **[tauri.ts]** 添加新接口
   - 添加类型定义
   - 添加 devApi 方法

### 第三阶段：UI 组件

6. **[TimelineControl.vue]** 新建时间轴组件
   - 实现阶段可视化
   - 实现交互逻辑

7. **[SnapshotManager.vue]** 新建快照管理组件
   - 实现创建/恢复/删除

8. **[DevTools.vue]** 重构布局
   - 添加 Tab 切换
   - 集成新组件

---

## 5. 数据结构对齐

### 5.1 阶段字符串映射

前端到后端的阶段字符串必须一致：

| 前端 (target) | 后端 (SeasonPhase) |
|---------------|-------------------|
| `SPRING_REGULAR` | `SpringRegular` |
| `SPRING_PLAYOFFS` | `SpringPlayoffs` |
| `MSI` | `Msi` |
| `MADRID_MASTERS` | `MadridMasters` |
| `SUMMER_REGULAR` | `SummerRegular` |
| `SUMMER_PLAYOFFS` | `SummerPlayoffs` |
| `CLAUDE_INTERCONTINENTAL` | `ClaudeIntercontinental` |
| `WORLD_CHAMPIONSHIP` | `WorldChampionship` |
| `SHANGHAI_MASTERS` | `ShanghaiMasters` |
| `ICP_INTERCONTINENTAL` | `IcpIntercontinental` |
| `SUPER_INTERCONTINENTAL` | `SuperIntercontinental` |
| `ANNUAL_AWARDS` | `AnnualAwards` |
| `TRANSFER_WINDOW` | `TransferWindow` |
| `DRAFT` | `Draft` |
| `SEASON_END` | `SeasonEnd` |

### 5.2 现有接口复用

| 功能 | 复用现有 |
|------|---------|
| 快进 | `timeApi.fastForwardTo()` |
| 阶段重置 | `devApi.resetPhase()` |
| 游戏状态 | `devApi.getGameStatus()` |
| 时间状态 | `timeApi.getTimeState()` |

---

## 6. 风险与注意事项

### 6.1 数据完整性
- 回滚前应自动创建快照（可选）
- 删除操作不可逆，需二次确认

### 6.2 数据库锁
- 创建/恢复快照时需要独占数据库
- 恢复快照后需要重新加载存档

### 6.3 文件系统
- 快照文件可能较大（10-50MB）
- 需要定期清理旧快照

---

## 7. 测试计划

### 7.1 单元测试
- 阶段映射函数正确性
- 数据删除逻辑正确性

### 7.2 集成测试
- 回滚后游戏状态正确
- 快照恢复后数据一致
- UI 交互正常

### 7.3 边界测试
- 回滚到当前阶段
- 回滚到第一个阶段
- 跨赛季回滚
- 空快照列表

---

## 8. 预估工作量

| 任务 | 预估 |
|------|------|
| 后端回滚命令 | ●●●○○ |
| 后端快照命令 | ●●●●○ |
| 前端 API | ●○○○○ |
| 时间轴组件 | ●●●○○ |
| 快照管理组件 | ●●○○○ |
| DevTools 重构 | ●●○○○ |
| 测试 | ●●○○○ |

---

**文档版本**: v1.0
**创建日期**: 2024-02-06
**作者**: Claude
