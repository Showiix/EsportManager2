# 系统架构与调用链

## 技术架构概览

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              前端层 (Vue3 + TypeScript)                          │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  ┌─────────────────┐│
│  │    Views       │  │   Components   │  │  Pinia Stores  │  │   API Layer     ││
│  │  (页面组件)     │──│  (UI 组件)     │──│  (状态管理)     │──│  (tauri.ts)     ││
│  └────────────────┘  └────────────────┘  └────────────────┘  └────────┬────────┘│
└──────────────────────────────────────────────────────────────────────┼──────────┘
                                                                       │ Tauri IPC
┌──────────────────────────────────────────────────────────────────────┼──────────┐
│                              后端层 (Rust + Tauri)                    │          │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  ┌───────▼────────┐│
│  │   Engines      │──│   Services     │──│  Repositories  │──│   Commands     ││
│  │  (计算引擎)     │  │   (业务服务)    │  │   (数据仓库)    │  │   (命令接口)    ││
│  └────────────────┘  └────────────────┘  └───────┬────────┘  └────────────────┘│
│                                                   │                             │
│                                           ┌───────▼────────┐                    │
│                                           │    SQLite      │                    │
│                                           │   数据库        │                    │
│                                           └────────────────┘                    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 核心系统关系图

```
┌─────────────────────────────────────────────────────────────┐
│                        时间推进引擎                          │
│              (SeasonProgressEngine/GameFlowService)         │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   比赛引擎    │ │   转会系统    │ │   选秀系统    │
│(MatchSimEng) │ │(TransferEng) │ │ (DraftEng)   │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   选手系统    │ │   财政系统    │ │   荣誉系统    │
│  (Player)    │ │ (Financial)  │ │  (Honor)     │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       ├────────┬───────┴────────┬───────┘
       ▼        ▼                ▼
┌──────────┐ ┌──────────┐ ┌──────────────┐
│ 身价系统  │ │ 积分系统  │ │ 选秀权拍卖   │
│(MarketVal)│ │ (Points) │ │(DraftAuction)│
└──────────┘ └──────────┘ └──────────────┘

辅助系统:
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   特性系统    │ │  状态/体力    │ │   满意度     │ │   事件引擎    │
│  (Traits)    │ │ (Condition)  │ │(Satisfaction)│ │   (Event)    │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   战力引擎    │ │  选手决策     │ │  表现评估     │
│(PowerEngine) │ │(PlayerDecis) │ │(PlayerPerf)  │
└──────────────┘ └──────────────┘ └──────────────┘
```

## 系统交互关系

```
                              ┌───────────────────┐
                              │   时间推进系统     │
                              │  (GameFlowService)│
                              └─────────┬─────────┘
                                        │
              ┌─────────────────────────┼─────────────────────────┐
              │                         │                         │
              ▼                         ▼                         ▼
    ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
    │   比赛模拟引擎   │       │    荣誉系统      │       │   年度积分系统   │
    │ (LeagueService) │       │ (HonorService)  │       │ (PointsEngine)  │
    └────────┬────────┘       └────────┬────────┘       └────────┬────────┘
             │                         │                         │
             ▼                         ▼                         ▼
    ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
    │   数据中心系统   │       │    荣誉殿堂      │       │    积分排名      │
    │ (PlayerStats)   │       │   (HonorHall)   │       │   (Rankings)    │
    └─────────────────┘       └─────────────────┘       └─────────────────┘
             │                                                   │
             ▼                                                   ▼
    ┌─────────────────┐                                 ┌─────────────────┐
    │    选手评级      │                                 │  Super 资格赛    │
    │  MVP 评选       │                                 │   (Top 16)      │
    └─────────────────┘                                 └─────────────────┘
```

## 核心调用链

### 1. 游戏启动流程

```
App.vue mounted
    │
    ▼
saveApi.getSaves() ───────────────────────► get_saves (Tauri Command)
    │
    ▼
显示存档列表 / 创建新存档
    │
    ▼
saveApi.loadSave(saveId) ─────────────────► load_save
    │
    ▼
timeStore.fetchTimeState() ───────────────► get_time_state
    │
    ▼
渲染主界面，显示当前阶段
```

### 2. 比赛模拟流程

```
点击"模拟下一场"
    │
    ▼
time_simulate_next (Tauri Command)
    │
    ├─► MatchRepository::get_next_pending_match()
    │
    ├─► LeagueService::simulate_match()
    │       │
    │       ├─► 获取双方首发选手
    │       │
    │       ├─► simulate_game_with_players() [每局]
    │       │       │
    │       │       ├─► 第一层：计算选手发挥值
    │       │       │   actual = ability + condition + N(0,σ)
    │       │       │
    │       │       ├─► 第二层：计算队伍战力
    │       │       │   team_power = Σ(actual) / 5
    │       │       │
    │       │       └─► 第三层：判定胜负
    │       │           diff + N(0,3) > 0 ? home : away
    │       │
    │       └─► 更新比赛结果到数据库
    │
    └─► 返回 SimulateNextResult
```

### 3. 阶段完成与推进流程

```
所有比赛完成后，点击"完成并推进"
    │
    ▼
complete_and_advance (Tauri Command)
    │
    ▼
GameFlowService::complete_phase()
    │
    ├─► 颁发荣誉 (HonorService)
    │       ├─► 推断赛事结果（冠亚季殿军）
    │       ├─► 创建 TeamChampion, TeamRunnerUp 等荣誉
    │       ├─► 创建 PlayerChampion（冠军队选手）
    │       └─► 创建 TournamentMvp（赛事MVP）
    │
    ├─► 颁发积分 (PointsCalculationEngine)
    │       ├─► 根据名次获取积分配置
    │       ├─► 保存到 annual_points_detail 表
    │       └─► 更新 team.annual_points
    │
    ├─► 推进到下一阶段
    │       └─► 更新 save.current_phase
    │
    └─► 返回 CompleteAndAdvanceResult
            { honors_awarded, points_awarded, next_phase }
```

### 4. 赛季结算流程

```
Super 赛结束后，进入赛季结算阶段
    │
    ▼
GameFlowService::execute_season_settlement()
    │
    ├─► 选手年龄增长 (player.age += 1)
    ├─► 选手能力成长 (根据 talent 标签)
    ├─► 选手衰退 (年龄 > 28 开始)
    ├─► 退役判定 (年龄 >= 35 或 能力 < 50)
    └─► 合同处理 (contract_years -= 1)
    │
    ▼
GameFlowService::advance_to_new_season()
    │
    ├─► 清空年度积分 (team.annual_points = 0)
    ├─► 重置赛季状态 (current_season += 1)
    └─► 设置阶段 (current_phase = SpringRegular)
```

## 前端 Store 架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Pinia Stores (29个)                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  核心状态:                                                           │
│  useGameStore        useTimeStore         useSeasonStore            │
│  useSettingsStore    useRegionStore       useEventStore             │
│                                                                     │
│  数据状态:                                                           │
│  usePlayerStore      useTeamStore         useTeamStoreTauri         │
│  useMatchDetailStore useScheduleStore     useRankingStore           │
│  usePointsStore      useHonorHallStore    useFinanceStore           │
│                                                                     │
│  赛事状态:                                                           │
│  useTournamentStoreTauri   useAutoTournamentStore                   │
│  usePlayoffStore     useMSIStore          useClauchStore            │
│  useSuperStore       useWorldsStore                                 │
│                                                                     │
│  转会/选秀状态:                                                      │
│  useTransferStoreTauri     useTransferWindowStore                   │
│  useDraftStoreTauri        useDraftAuctionStore                     │
│                                                                     │
│  辅助状态:                                                           │
│  useAIStrategyStore  usePerformanceStore                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## API 层结构

```typescript
// src/api/tauri.ts
export const tauriApi = {
  app: appApi,           // 应用信息
  save: saveApi,         // 存档管理
  team: teamApi,         // 队伍管理
  player: playerApi,     // 选手管理
  tournament: tournamentApi,  // 赛事管理
  honor: honorApi,       // 荣誉查询
  draft: draftApi,       // 选秀系统
  draftAuction: draftAuctionApi, // 选秀权拍卖
  transfer: transferApi, // 转会系统
  finance: financeApi,   // 财政系统
  query: queryApi,       // 通用查询
  international: internationalApi,  // 国际赛事
  match: matchApi,       // 比赛模拟
  event: eventApi,       // 事件系统
  stats: statsApi,       // 数据统计
  time: timeApi,         // 时间推进
  points: pointsApi,     // 年度积分
  matchDetails: matchDetailsApi,  // 比赛详情
  perf: perfApi,         // 性能监控
  log: logApi,           // 日志系统
  awards: awardsApi,     // 颁奖系统
  dev: devApi,           // 开发工具
}
```
