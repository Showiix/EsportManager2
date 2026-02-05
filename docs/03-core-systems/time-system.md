# 时间推进系统

## 概述

时间推进系统是电竞经理游戏的核心系统，负责统一管理游戏内的时间流转。通过该系统，玩家可以控制赛季进度、推进阶段、模拟比赛，并在关键节点获得荣誉颁发等反馈。

## 系统架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           前端 (Vue3 + Pinia)                            │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐   │
│  │ GameTimePanel.vue│    │  useTimeStore.ts │    │   timeApi        │   │
│  │   (可视化页面)    │ ◄──│   (状态管理)      │ ◄──│   (API 调用)     │   │
│  └──────────────────┘    └──────────────────┘    └────────┬─────────┘   │
└───────────────────────────────────────────────────────────┼─────────────┘
                                                            │ Tauri IPC
┌───────────────────────────────────────────────────────────┼─────────────┐
│                           后端 (Rust + Tauri)              │             │
│  ┌──────────────────┐    ┌──────────────────┐    ┌────────▼─────────┐   │
│  │  GameFlowService │ ◄──│   game_flow.rs   │ ◄──│ time_commands.rs │   │
│  │   (核心服务)      │    │   (业务逻辑)      │    │   (命令接口)     │   │
│  └────────┬─────────┘    └──────────────────┘    └──────────────────┘   │
│           │                                                              │
│  ┌────────▼─────────┐    ┌──────────────────┐    ┌──────────────────┐   │
│  │   HonorService   │    │  LeagueService   │    │   EventEngine    │   │
│  │   (荣誉颁发)      │    │  (赛程生成)       │    │   (事件处理)     │   │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## 核心数据模型

### GameTimeState

统一的时间状态返回结构：

```typescript
interface GameTimeState {
  save_id: string              // 存档ID
  current_season: number       // 当前赛季 (1, 2, 3...)
  current_phase: string        // 当前阶段枚举值
  phase_display_name: string   // 当前阶段显示名称
  phase_status: PhaseStatus    // 阶段状态
  phase_progress: PhaseProgress // 阶段进度详情
  season_progress: SeasonProgress // 赛季进度详情
  available_actions: TimeAction[] // 当前可用操作
  can_advance: boolean         // 是否可以推进到下一阶段
  next_phase: string | null    // 下一阶段名称
}
```

### PhaseStatus

```typescript
type PhaseStatus =
  | 'NOT_INITIALIZED'  // 未初始化 - 阶段刚进入，赛事未创建
  | 'IN_PROGRESS'      // 进行中 - 赛事已创建，比赛进行中
  | 'COMPLETED'        // 已完成 - 所有比赛完成，可以推进
```

### TimeAction

根据当前阶段和状态，系统返回可执行的操作列表：

```typescript
type TimeAction =
  | 'INITIALIZE_PHASE'       // 初始化阶段（创建赛事）
  | 'SIMULATE_NEXT_MATCH'    // 模拟下一场比赛
  | 'SIMULATE_ALL_MATCHES'   // 模拟所有比赛
  | 'COMPLETE_AND_ADVANCE'   // 完成并推进到下一阶段
  | 'FAST_FORWARD_PHASE'     // 快进到下一阶段
  | 'FAST_FORWARD_TO_SUMMER' // 快进到夏季赛
  | 'FAST_FORWARD_TO_WORLDS' // 快进到世界赛
  | 'FAST_FORWARD_TO_SEASON_END' // 快进到赛季结束
  | 'START_TRANSFER_WINDOW'  // 开始转会窗口
  | 'EXECUTE_TRANSFER_ROUND' // 执行转会轮次
  | 'START_DRAFT'            // 开始选秀
  | 'EXECUTE_SEASON_SETTLEMENT' // 执行赛季结算
  | 'START_NEW_SEASON'       // 开始新赛季
```

## API 接口

### 后端命令 (Tauri Commands)

| 命令 | 说明 | 返回值 |
|------|------|--------|
| `get_time_state` | 获取完整的游戏时间状态 | `GameTimeState` |
| `time_init_phase` | 初始化当前阶段（创建赛事） | `string` |
| `complete_and_advance` | 完成当前阶段并推进 | `CompleteAndAdvanceResult` |
| `fast_forward_to` | 快进到指定目标 | `FastForwardResult` |
| `time_simulate_all` | 模拟当前阶段所有比赛 | `number` |
| `time_season_settlement` | 执行赛季结算 | `SeasonSettlementResult` |
| `time_start_new_season` | 开始新赛季 | `number` |

### 前端 API

```typescript
import { timeApi } from '@/api/tauri'

// 获取时间状态
const state = await timeApi.getTimeState()

// 初始化阶段
await timeApi.initPhase()

// 完成并推进
const result = await timeApi.completeAndAdvance()

// 快进到指定目标
await timeApi.fastForwardTo('SUMMER')  // NEXT_PHASE, SUMMER, WORLDS, SEASON_END

// 模拟所有比赛
const count = await timeApi.simulateAll()

// 赛季结算
const settlement = await timeApi.seasonSettlement()

// 开始新赛季
const newSeason = await timeApi.startNewSeason()
```

## 状态机流转

### 阶段内状态流转

```
┌──────────────────┐     初始化阶段      ┌──────────────────┐
│  NOT_INITIALIZED │ ─────────────────► │   IN_PROGRESS    │
│    (未初始化)     │                    │    (进行中)       │
└──────────────────┘                    └────────┬─────────┘
                                                 │
                                          模拟比赛...
                                                 │
                                                 ▼
                                        ┌──────────────────┐
                                        │    COMPLETED     │
                                        │    (已完成)       │
                                        └────────┬─────────┘
                                                 │
                                          完成并推进
                                                 │
                                                 ▼
                                        ┌──────────────────┐
                                        │  下一阶段开始     │
                                        │  NOT_INITIALIZED │
                                        └──────────────────┘
```

### 可用操作对照表

| 阶段类型 | 状态 | 可用操作 |
|----------|------|----------|
| 赛事阶段 | NOT_INITIALIZED | InitializePhase |
| 赛事阶段 | IN_PROGRESS | SimulateNextMatch, SimulateAllMatches |
| 赛事阶段 | COMPLETED | CompleteAndAdvance |
| 转会期 | - | StartTransferWindow, ExecuteTransferRound |
| 选秀期 | - | StartDraft |
| 赛季结束 | - | ExecuteSeasonSettlement, StartNewSeason |

## 快进功能

快进允许玩家跳过中间阶段：

| 目标 | 说明 |
|------|------|
| `NEXT_PHASE` | 快进到下一阶段 |
| `SUMMER` | 快进到夏季常规赛 |
| `WORLDS` | 快进到世界赛 |
| `SEASON_END` | 快进到赛季结束 |

## 文件位置

### 后端

| 文件 | 说明 |
|------|------|
| `models/game_time.rs` | 时间相关数据模型 |
| `services/game_flow.rs` | 游戏流程核心服务 |
| `commands/time_commands.rs` | Tauri 命令接口 |

### 前端

| 文件 | 说明 |
|------|------|
| `api/tauri.ts` | timeApi 定义 |
| `stores/useTimeStore.ts` | Pinia 状态管理 |
| `views/GameTimePanel.vue` | 时间控制面板页面 |
