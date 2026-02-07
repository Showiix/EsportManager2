# 项目目录结构

## 完整目录结构

```
EsportManager2-Backend/
│
├── src/                          # 前端源码 (Vue 3 + TypeScript)
│   ├── api/                      # API 客户端
│   │   └── tauri.ts             # Tauri 命令调用封装
│   │
│   ├── components/               # Vue 组件
│   │   ├── common/              # 通用组件
│   │   │   ├── ErrorGuide.vue   # 错误引导弹窗
│   │   │   └── SeasonSelector.vue # 赛季选择器
│   │   ├── layout/              # 布局组件
│   │   │   ├── Header.vue       # 顶部导航
│   │   │   ├── MainLayout.vue   # 主布局
│   │   │   └── Sidebar.vue      # 侧边栏菜单
│   │   ├── match/               # 比赛相关组件
│   │   │   ├── GameDetailView.vue
│   │   │   └── MatchDetailDialog.vue
│   │   ├── msi/                 # MSI 赛事组件
│   │   │   ├── MSIBracketView.vue
│   │   │   └── MSIMatchCard.vue
│   │   ├── worlds/              # 世界赛组件
│   │   │   ├── KnockoutBracket.vue
│   │   │   ├── SwissRoundBracket.vue
│   │   │   ├── WorldsKnockoutBracket.vue
│   │   │   ├── WorldsMatchCard.vue
│   │   │   └── WorldsSwissRound.vue
│   │   ├── icp/                 # ICP 洲际赛组件
│   │   │   ├── ICPRegionBattleCard.vue
│   │   │   └── ICPSeedGroupStanding.vue
│   │   ├── super/               # Super 洲际赛组件
│   │   │   ├── SuperGroupStanding.vue
│   │   │   ├── SuperKnockoutBracket.vue
│   │   │   └── SuperMatchCard.vue
│   │   ├── clauch/              # Claude 洲际赛组件
│   │   │   ├── ClauchGroupStanding.vue
│   │   │   ├── ClauchKnockoutBracket.vue
│   │   │   └── ClauchMatchCard.vue
│   │   ├── transfer/            # 转会相关组件
│   │   │   ├── PlayerContractDialog.vue
│   │   │   └── PlayerStrategyDialog.vue
│   │   ├── finance/             # 财务组件
│   │   │   └── TeamFinanceDialog.vue
│   │   ├── player/              # 选手组件
│   │   │   └── PlayerEditDialog.vue
│   │   ├── settings/            # 设置组件
│   │   │   └── GameGuide.vue
│   │   └── dev/                 # 开发工具组件
│   │       └── LogMonitor.vue
│   │
│   ├── engines/                  # 前端计算引擎
│   │   ├── index.ts             # 模块导出
│   │   ├── PlayerEngine.ts      # 选手能力计算
│   │   └── PowerEngine.ts       # 队伍战力计算
│   │
│   ├── stores/                   # Pinia 状态管理
│   │   ├── tauri.ts             # Tauri API 封装
│   │   ├── useGameStore.ts      # 游戏全局状态
│   │   ├── useTimeStore.ts      # 时间推进状态
│   │   ├── useSeasonStore.ts    # 赛季状态
│   │   ├── usePlayerStore.ts    # 选手状态
│   │   ├── useTeamStore.ts      # 战队状态
│   │   ├── useTeamStoreTauri.ts # 战队 Tauri 集成
│   │   ├── useMatchDetailStore.ts # 比赛详情状态
│   │   ├── useScheduleStore.ts  # 赛程状态
│   │   ├── useRankingStore.ts   # 排名状态
│   │   ├── usePointsStore.ts    # 年度积分状态
│   │   ├── useHonorHallStore.ts # 荣誉殿堂状态
│   │   ├── useFinanceStore.ts   # 财务状态
│   │   ├── useRegionStore.ts    # 赛区状态
│   │   ├── useSettingsStore.ts  # 设置状态
│   │   ├── usePerformanceStore.ts # 性能监控状态
│   │   ├── useEventStore.ts     # 事件状态
│   │   ├── useAIStrategyStore.ts # AI 策略状态
│   │   ├── useAutoTournamentStore.ts # 自动赛事状态
│   │   ├── useTransferStoreTauri.ts  # 转会 Tauri 集成
│   │   ├── useTransferWindowStore.ts # 转会窗口状态
│   │   ├── useDraftStoreTauri.ts     # 选秀状态
│   │   ├── useDraftAuctionStore.ts   # 选秀权拍卖状态
│   │   ├── useTournamentStoreTauri.ts # 赛事 Tauri 集成
│   │   ├── usePlayoffStore.ts   # 季后赛状态
│   │   ├── useMSIStore.ts       # MSI 赛事状态
│   │   ├── useClauchStore.ts    # Claude 洲际赛状态
│   │   ├── useSuperStore.ts     # Super 洲际赛状态
│   │   └── useWorldsStore.ts    # 世界赛状态
│   │
│   ├── views/                    # 页面视图
│   │   ├── Dashboard.vue        # 仪表盘
│   │   ├── GameTimePanel.vue    # 时间控制面板
│   │   ├── Settings.vue         # 设置页面
│   │   ├── DevTools.vue         # 开发工具
│   │   ├── PerformanceMonitor.vue # 性能监控
│   │   │
│   │   ├── Teams.vue            # 战队列表
│   │   ├── TeamDetail.vue       # 战队详情
│   │   ├── TeamEdit.vue         # 战队编辑
│   │   ├── TeamGMConfig.vue     # GM 配置
│   │   ├── TeamEvaluationCenter.vue # 战队评估中心
│   │   │
│   │   ├── Players.vue          # 选手列表
│   │   ├── PlayerDetail.vue     # 选手详情
│   │   ├── PlayerStatistics.vue # 选手统计
│   │   ├── PlayerMarket.vue     # 选手市场
│   │   ├── PlayerEvaluationCenter.vue # 选手评估中心
│   │   │
│   │   ├── Tournaments.vue      # 赛事管理
│   │   ├── TournamentDetail.vue # 赛事详情
│   │   ├── SpringDetail.vue     # 春季赛详情
│   │   ├── SpringPlayoffsDetail.vue # 春季季后赛详情
│   │   ├── SummerDetail.vue     # 夏季赛详情
│   │   ├── SummerPlayoffsDetail.vue # 夏季季后赛详情
│   │   ├── MSIDetail.vue        # MSI 详情
│   │   ├── MadridDetail.vue     # 马德里大师赛详情
│   │   ├── ClauchDetail.vue     # Claude 洲际赛详情
│   │   ├── ICPDetail.vue        # ICP 洲际赛详情
│   │   ├── WorldsDetail.vue     # 世界赛详情
│   │   ├── ShanghaiDetail.vue   # 上海大师赛详情
│   │   ├── SuperDetail.vue      # Super 洲际赛详情
│   │   │
│   │   ├── Rankings.vue         # 积分排名
│   │   ├── Honors.vue           # 荣誉记录
│   │   ├── HonorHall.vue        # 荣誉殿堂
│   │   ├── TeamHonorRankings.vue  # 战队荣誉排行
│   │   ├── PlayerHonorRankings.vue # 选手荣誉排行
│   │   ├── InternationalHall.vue  # 国际赛殿堂
│   │   │
│   │   ├── DataCenter.vue       # 数据中心
│   │   ├── DataCenterPlayerDetail.vue # 数据中心选手详情
│   │   ├── AnnualAwards.vue     # 年度颁奖
│   │   ├── AnnualTop.vue        # 年度评选
│   │   │
│   │   ├── Finance.vue          # 财务页面
│   │   │
│   │   ├── Transfer.vue         # 转会系统
│   │   ├── TransferWindow.vue   # 转会窗口
│   │   ├── TransferBidAnalysis.vue   # 竞价分析
│   │   ├── TransferBroadcast.vue     # 转会动态
│   │   ├── TransferMarketListings.vue # 转会市场挂牌
│   │   ├── TransferReport.vue   # 转会报告
│   │   │
│   │   ├── Draft.vue            # 选秀
│   │   ├── DraftPool.vue        # 选秀池
│   │   ├── DraftRegion.vue      # 赛区选秀
│   │   └── DraftAuction.vue     # 选秀权拍卖
│   │
│   ├── types/                    # TypeScript 类型定义
│   │   ├── index.ts             # 主类型定义
│   │   ├── player.ts            # 选手类型
│   │   ├── matchDetail.ts       # 比赛详情类型
│   │   ├── clauch.ts            # Claude 洲际赛类型
│   │   ├── icp.ts               # ICP 洲际赛类型
│   │   └── super.ts             # Super 洲际赛类型
│   │
│   └── router/                   # Vue Router 路由
│       └── index.ts
│
├── src-tauri/                    # Tauri + Rust 后端
│   ├── src/
│   │   ├── main.rs              # 应用入口
│   │   ├── lib.rs               # 库入口
│   │   │
│   │   ├── commands/            # Tauri Commands (API接口层)
│   │   │   ├── mod.rs
│   │   │   ├── game_commands.rs         # 核心游戏命令
│   │   │   ├── save_commands.rs         # 存档管理
│   │   │   ├── time_commands.rs         # 时间推进
│   │   │   ├── match_commands.rs        # 比赛模拟
│   │   │   ├── match_detail_commands.rs # 比赛详情查询
│   │   │   ├── player_commands.rs       # 选手管理 (已废弃，合并到 query)
│   │   │   ├── team_commands.rs         # 战队管理
│   │   │   ├── honor_commands.rs        # 荣誉系统
│   │   │   ├── finance_commands.rs      # 财政系统
│   │   │   ├── transfer_commands.rs     # 转会系统
│   │   │   ├── draft_commands.rs        # 选秀系统
│   │   │   ├── draft_auction_commands.rs # 选秀权拍卖
│   │   │   ├── points_commands.rs       # 年度积分
│   │   │   ├── stats_commands.rs        # 数据统计
│   │   │   ├── query_commands.rs        # 通用查询
│   │   │   ├── international_commands.rs # 国际赛事
│   │   │   ├── event_commands.rs        # 事件系统
│   │   │   ├── awards_commands.rs       # 颁奖典礼
│   │   │   ├── dev_commands.rs          # 开发调试工具
│   │   │   ├── log_commands.rs          # 日志系统
│   │   │   └── perf_commands.rs         # 性能监控
│   │   │
│   │   ├── engines/             # 核心计算引擎
│   │   │   ├── mod.rs
│   │   │   ├── match_simulation.rs   # 比赛模拟引擎
│   │   │   ├── transfer.rs           # 转会引擎
│   │   │   ├── financial.rs          # 财政引擎
│   │   │   ├── points_calculation.rs # 积分计算引擎
│   │   │   ├── honor.rs              # 荣誉引擎
│   │   │   ├── draft.rs              # 选秀引擎
│   │   │   ├── draft_auction.rs      # 选秀权拍卖引擎
│   │   │   ├── market_value.rs       # 身价计算引擎
│   │   │   ├── season_progress.rs    # 赛季进度引擎
│   │   │   ├── power_engine.rs       # 队伍战力引擎
│   │   │   ├── traits.rs             # 选手特性系统
│   │   │   ├── condition.rs          # 选手状态/体力系统
│   │   │   ├── satisfaction.rs       # 选手满意度引擎
│   │   │   ├── player_decision.rs    # AI 选手决策
│   │   │   ├── player_performance.rs # 选手表现评估
│   │   │   └── event.rs              # 事件引擎
│   │   │
│   │   ├── models/              # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── player.rs             # 选手模型
│   │   │   ├── player_stats.rs       # 选手统计
│   │   │   ├── player_status.rs      # 选手状态
│   │   │   ├── team.rs               # 战队模型
│   │   │   ├── tournament.rs         # 赛事模型
│   │   │   ├── tournament_result.rs  # 赛事结果
│   │   │   ├── match_record.rs       # 比赛记录
│   │   │   ├── match_game_detail.rs  # 比赛详情
│   │   │   ├── honor.rs              # 荣誉模型
│   │   │   ├── season.rs             # 赛季模型
│   │   │   ├── save.rs               # 存档模型
│   │   │   ├── standings.rs          # 积分榜模型
│   │   │   ├── draft.rs              # 选秀模型
│   │   │   ├── draft_auction.rs      # 选秀权拍卖模型
│   │   │   ├── transfer.rs           # 转会事件模型
│   │   │   ├── financial.rs          # 财务交易模型
│   │   │   ├── event.rs              # 事件模型
│   │   │   ├── game_time.rs          # 游戏时间模型
│   │   │   └── llm_task_log.rs       # LLM 任务日志
│   │   │
│   │   ├── services/            # 业务服务层
│   │   │   ├── mod.rs
│   │   │   ├── game_flow.rs          # 游戏流程核心服务
│   │   │   ├── league_service.rs     # 联赛服务
│   │   │   ├── honor_service.rs      # 荣誉服务
│   │   │   ├── tournament_service.rs # 赛事服务
│   │   │   ├── init_service.rs       # 游戏初始化服务
│   │   │   ├── player_data.rs        # 选手初始数据
│   │   │   ├── draft_pool_data.rs    # 选秀池初始数据
│   │   │   ├── draft_ai_service.rs   # 选秀 AI 决策
│   │   │   ├── logging_service.rs    # 日志服务
│   │   │   └── perf_service.rs       # 性能监控服务
│   │   │
│   │   └── db/                  # 数据库操作
│   │       ├── mod.rs
│   │       ├── connection.rs    # 数据库连接与迁移
│   │       ├── migrations.rs    # 迁移工具
│   │       └── repository.rs    # 数据仓库（所有表操作）
│   │
│   ├── Cargo.toml               # Rust 依赖配置
│   └── tauri.conf.json          # Tauri 配置
│
├── docs/                         # 项目文档
│   ├── README.md                # 文档索引
│   ├── CONTRIBUTING.md          # 文档编写指南
│   ├── 01-overview/             # 项目概览
│   ├── 02-game-design/          # 游戏设计
│   ├── 03-core-systems/         # 核心系统
│   ├── 04-technical/            # 技术文档
│   ├── 05-ai/                   # AI 系统
│   ├── archive/                 # 归档文档
│   ├── debug/                   # 调试记录
│   └── develop/                 # 开发计划
│
├── .claude/                      # Claude Code 配置
│   └── skills/                  # 技能文档
│
├── debug/                        # Bug 修复记录
│   └── README.md
│
├── logs/                         # 日志与更新记录
│
├── package.json                 # Node.js 依赖
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
└── CLAUDE.md                   # Claude Code 项目指南
```

## 核心目录说明

### 前端 (src/)

| 目录 | 文件数 | 说明 |
|------|--------|------|
| `api/` | 1 | Tauri 命令的前端封装，所有后端调用通过此层 |
| `components/` | 28 | 可复用的 Vue 组件，按功能模块分类 |
| `engines/` | 3 | 前端计算逻辑，如选手发挥值计算 |
| `stores/` | 29 | Pinia 状态管理，管理全局状态 |
| `views/` | 48 | 页面级组件，对应路由 |
| `types/` | 6 | TypeScript 类型定义 |

### 后端 (src-tauri/src/)

| 目录 | 文件数 | 说明 |
|------|--------|------|
| `commands/` | 23 | Tauri Command 定义，前后端通信的接口层 |
| `engines/` | 17 | 核心业务逻辑和算法实现 |
| `models/` | 20 | 数据结构定义，与数据库表对应 |
| `services/` | 11 | 业务服务层，组织多个引擎协作 |
| `db/` | 4 | 数据库连接、迁移和仓库操作（统一在 repository.rs 中） |

### 文档 (docs/)

| 目录 | 说明 |
|------|------|
| `01-overview/` | 项目架构、技术栈、目录结构 |
| `02-game-design/` | 游戏策划、赛事设计、玩法规则 |
| `03-core-systems/` | 各子系统的详细设计文档 |
| `04-technical/` | 数据库、API、开发指南 |
| `05-ai/` | AI 决策系统设计 |
| `archive/` | 历史归档文档 |
| `debug/` | 调试与 Bug 修复记录 |

## 文件命名规范

### Rust 文件

- 使用 `snake_case` 命名
- 模块入口使用 `mod.rs`
- 示例: `match_simulation.rs`, `game_flow.rs`

### TypeScript/Vue 文件

- 组件使用 `PascalCase`: `GameTimePanel.vue`
- 工具/类型使用 `camelCase` 或 `PascalCase`: `PlayerEngine.ts`
- Store 使用 `use` 前缀: `useTimeStore.ts`

### 文档文件

- 使用 `kebab-case` 命名
- 示例: `time-system.md`, `match-simulation.md`
