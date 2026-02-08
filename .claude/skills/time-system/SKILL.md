---
name: time-system
description: 电竞经理游戏的时间推进系统。管理赛季阶段、比赛模拟、赛季结算等核心游戏流程。当需要修改游戏进度、阶段推进、比赛模拟逻辑时使用此技能。
---

# 时间推进系统 (Time System)

## Overview

时间推进系统是游戏的核心引擎，管理整个赛季的15个阶段流转、比赛模拟调度、以及赛季结算。它与比赛模拟系统、荣誉系统、财政系统紧密集成。

## 核心组件

### 1. SeasonProgressEngine

**文件**: `src-tauri/src/engines/season_progress.rs`

赛季进度引擎，管理阶段状态和可执行操作。

```rust
pub struct SeasonProgressEngine {
    current_season: u32,
    current_phase: SeasonPhase,
    phase_completed: bool,
}
```

**核心方法**:
- `advance_phase()` - 推进到下一阶段
- `get_available_actions()` - 获取当前阶段可执行的操作
- `validate_action()` - 验证操作是否允许
- `get_progress()` - 获取赛季进度 (已完成/总数)

### 2. GameFlowService

**文件**: `src-tauri/src/services/game_flow.rs`

游戏流程服务，协调各引擎完成阶段初始化和推进。

**核心方法**:
- `get_time_state()` - 获取完整游戏时间状态
- `initialize_phase()` - 初始化当前阶段（创建赛事）
- `complete_and_advance()` - 完成当前阶段并推进
- `complete_phase()` - 完成阶段并颁发荣誉（**含幂等保护**：已完成的阶段不会重复执行）
- `fast_forward_to()` - 快进到指定目标阶段
- `execute_season_settlement()` - 执行赛季结算
- `advance_to_new_season()` - 推进到新赛季

**初始化辅助方法** (由 `initialize_phase` 调用，避免春/夏季重复代码):
- `init_regional_regular_season(pool, save_id, season_id, tournament_type, season_label)` - 赛区常规赛通用初始化
- `init_regional_playoffs(pool, save_id, season_id, playoff_type, regular_type_str, season_label)` - 赛区季后赛通用初始化
- `init_32team_masters(pool, save_id, season_id, tournament_type, source_regular, tournament_name)` - 32队大师赛通用初始化

**工具方法**:
- `get_all_region_ids(pool, save_id)` - 获取所有赛区 ID
- `count_tournament_matches(pool, tournament_id)` - 统计赛事比赛数

## 数据结构

### SeasonPhase (赛季阶段)

**文件**: `src-tauri/src/models/season.rs`

```rust
pub enum SeasonPhase {
    SpringRegular,          // 春季赛常规赛
    SpringPlayoffs,         // 春季赛季后赛
    Msi,                    // MSI季中赛
    MadridMasters,          // 马德里大师赛
    SummerRegular,          // 夏季赛常规赛
    SummerPlayoffs,         // 夏季赛季后赛
    ClaudeIntercontinental, // Claude洲际赛
    WorldChampionship,      // S世界赛
    ShanghaiMasters,        // 上海大师赛
    IcpIntercontinental,    // ICP四赛区洲际对抗赛
    SuperIntercontinental,  // Super洲际年度邀请赛
    AnnualAwards,           // 年度颁奖典礼
    TransferWindow,         // 转会期
    Draft,                  // 选秀（每4年一次：S2, S6, S10...）
    SeasonEnd,              // 赛季结束
}
```

**SeasonPhase 实用方法** (集中映射，避免在各处重复 match):

| 方法 | 返回类型 | 说明 |
|------|----------|------|
| `to_tournament_type()` | `Option<TournamentType>` | 转换为赛事类型枚举 |
| `tournament_type_str()` | `Option<&'static str>` | 赛事类型字符串（用于 DB 查询） |
| `is_playoff()` | `bool` | 是否季后赛（SpringPlayoffs / SummerPlayoffs） |
| `is_international()` | `bool` | 是否国际赛 |
| `is_non_tournament()` | `bool` | 是否无赛事阶段（颁奖/转会/选秀/赛季结束） |
| `display_name()` | `&'static str` | 中文显示名（如 "春季常规赛"） |
| `order()` | `u8` | 阶段顺序号 (0-14) |
| `is_before(target)` | `bool` | 是否在目标阶段之前 |

> **注意**: 不要在 `game_flow.rs` 或 `time_commands.rs` 中重新编写阶段→赛事类型映射，统一使用 `SeasonPhase` 上的方法。

### GameTimeState (游戏时间状态)

**文件**: `src-tauri/src/models/game_time.rs`

```rust
pub struct GameTimeState {
    pub save_id: String,
    pub current_season: u32,
    pub current_phase: SeasonPhase,
    pub phase_display_name: String,
    pub phase_status: PhaseStatus,
    pub phase_progress: PhaseProgress,      // 当前阶段进度
    pub season_progress: SeasonProgress,    // 赛季整体进度
    pub available_actions: Vec<TimeAction>, // 可用操作
    pub can_advance: bool,                  // 是否可推进
    pub next_phase: Option<String>,         // 下一阶段
}
```

### GameAction (游戏操作)

```rust
pub enum GameAction {
    GenerateSpringSchedule,   // 生成春季赛赛程
    SimulateSpringMatch,      // 模拟春季赛比赛
    GeneratePlayoffs,         // 生成季后赛对阵
    SimulatePlayoffMatch,     // 模拟季后赛比赛
    GenerateMSI,              // 生成MSI对阵
    SimulateMSIMatch,         // 模拟MSI比赛
    // ... 其他赛事
    StartTransferWindow,      // 开始转会期
    ProcessTransfer,          // 执行转会
    EndTransferWindow,        // 结束转会期
    StartDraft,               // 开始选秀
    ProcessDraft,             // 执行选秀
    EndSeason,                // 结束赛季
    StartNewSeason,           // 开始新赛季
}
```

### FastForwardTarget (快进目标)

```rust
pub enum FastForwardTarget {
    NextPhase,              // 下一阶段
    ToPhase(SeasonPhase),   // 到指定阶段
    SeasonEnd,              // 赛季结束
}
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/time_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `get_time_state` | 获取完整游戏时间状态 | `GameTimeState` |
| `time_init_phase` | 初始化当前阶段（创建赛事） | `String` |
| `complete_and_advance` | 完成当前阶段并推进 | `CompleteAndAdvanceResult` |
| `fast_forward_to` | 快进到指定目标 | `FastForwardResult` |
| `time_simulate_all` | 模拟当前阶段所有比赛 | `u32` (模拟数量) |
| `time_simulate_next` | 模拟下一场比赛 | `SimulateNextResult` |
| `time_season_settlement` | 执行赛季结算 | `SeasonSettlementResult` |
| `time_start_new_season` | 推进到新赛季 | `u32` (新赛季号) |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 获取时间状态
export async function getTimeState(): Promise<GameTimeState>

// 初始化阶段
export async function timeInitPhase(): Promise<string>

// 完成并推进
export async function completeAndAdvance(): Promise<CompleteAndAdvanceResult>

// 快进到目标
export async function fastForwardTo(target: string): Promise<FastForwardResult>

// 模拟所有比赛
export async function timeSimulateAll(): Promise<number>

// 模拟下一场
export async function timeSimulateNext(): Promise<SimulateNextResult>

// 赛季结算
export async function timeSeasonSettlement(): Promise<SeasonSettlementResult>

// 开始新赛季
export async function timeStartNewSeason(): Promise<number>
```

### 快进目标字符串

| 目标 | 说明 |
|------|------|
| `NEXT_PHASE` | 下一阶段 |
| `SUMMER` / `SUMMER_REGULAR` | 夏季赛常规赛 |
| `WORLDS` / `WORLD_CHAMPIONSHIP` | 世界赛 |
| `SEASON_END` | 赛季结束 |
| `MSI` | MSI季中赛 |
| `SPRING_PLAYOFFS` | 春季季后赛 |
| `TRANSFER_WINDOW` | 转会期 |
| `DRAFT` | 选秀 |

## 系统集成

### 与比赛模拟系统
- `time_simulate_all` 和 `time_simulate_next` 调用 `LeagueService.simulate_match()`
- 季后赛自动生成下一轮对阵 (`advance_playoff_bracket`)
- 模拟后自动保存选手统计数据

### 与荣誉系统
- 赛事完成后自动记录冠军和MVP
- 赛季结算时生成年度荣誉

### 与财政系统
- 赛季结算时发放奖金
- 分赛区联盟分成

### 与数据中心系统
- 比赛模拟后更新 `player_tournament_stats`
- 影响力和表现数据用于MVP计算

### 与转会系统
- `TransferWindow` 阶段触发转会流程
- 转会期结束后推进到选秀

### 与选秀系统
- `Draft` 阶段仅在选秀年激活（S2, S6, S10...）
- `SeasonPhase::is_draft_year(season)` 判断是否选秀年

## 阶段流转规则

```
SpringRegular → SpringPlayoffs → Msi → MadridMasters
     ↓
SummerRegular → SummerPlayoffs → ClaudeIntercontinental
     ↓
WorldChampionship → ShanghaiMasters → IcpIntercontinental
     ↓
SuperIntercontinental → AnnualAwards → TransferWindow
     ↓
Draft (if is_draft_year) → SeasonEnd → [新赛季 SpringRegular]
```

## 使用示例

### 获取当前游戏状态
```typescript
const state = await getTimeState()
console.log(`S${state.current_season} - ${state.phase_display_name}`)
console.log(`进度: ${state.phase_progress.completed_matches}/${state.phase_progress.total_matches}`)
```

### 模拟并推进
```typescript
// 模拟所有当前阶段比赛
await timeSimulateAll()

// 完成并推进到下一阶段
const result = await completeAndAdvance()
console.log(`已推进到: ${result.new_phase}`)
```

### 快进到世界赛
```typescript
const result = await fastForwardTo('WORLDS')
console.log(`跳过了 ${result.skipped_phases.length} 个阶段`)
```

## 注意事项

1. **阶段初始化**: 进入新阶段前必须调用 `time_init_phase` 创建赛事
2. **季后赛特殊处理**: 使用逐场模拟确保正确生成对阵
3. **选秀年判断**: `is_draft_year` 规则为 `(season - 2) % 4 == 0`
4. **赛季结算顺序**: 必须在 `SeasonEnd` 阶段调用，包含退役、合同过期等处理
5. **幂等性保护**: `complete_phase` 会检查赛事是否已完成，重复调用安全返回
6. **映射方法统一**: 阶段→赛事类型映射统一使用 `SeasonPhase` 上的方法（`to_tournament_type()`, `tournament_type_str()`, `display_name()` 等），不要在业务代码中重复编写 match 映射
7. **初始化方法复用**: 春/夏季常规赛、季后赛、大师赛使用 `init_regional_regular_season`、`init_regional_playoffs`、`init_32team_masters` 通用方法，新增类似赛制时优先复用
