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
│   │   ├── match/               # 比赛相关组件
│   │   ├── player/              # 选手相关组件
│   │   ├── finance/             # 财务相关组件
│   │   ├── honor/               # 荣誉相关组件
│   │   ├── transfer/            # 转会相关组件
│   │   └── common/              # 通用组件
│   │
│   ├── engines/                  # 前端计算引擎
│   │   ├── PlayerEngine.ts      # 选手能力计算
│   │   └── PowerEngine.ts       # 队伍战力计算
│   │
│   ├── stores/                   # Pinia 状态管理
│   │   ├── useGameStore.ts      # 游戏全局状态
│   │   ├── useTimeStore.ts      # 时间推进状态
│   │   ├── useSeasonStore.ts    # 赛季状态
│   │   ├── usePlayerStore.ts    # 选手状态
│   │   ├── useTeamStore.ts      # 战队状态
│   │   └── useMatchDetailStore.ts # 比赛详情状态
│   │
│   ├── views/                    # 页面视图
│   │   ├── Dashboard.vue        # 仪表盘
│   │   ├── GameTimePanel.vue    # 时间控制面板
│   │   ├── Rankings.vue         # 积分排名
│   │   ├── HonorHall.vue        # 荣誉殿堂
│   │   └── ...
│   │
│   ├── types/                    # TypeScript 类型定义
│   │   ├── player.ts
│   │   ├── team.ts
│   │   ├── tournament.ts
│   │   └── ...
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
│   │   │   ├── time_commands.rs
│   │   │   ├── match_commands.rs
│   │   │   ├── player_commands.rs
│   │   │   ├── team_commands.rs
│   │   │   ├── honor_commands.rs
│   │   │   ├── finance_commands.rs
│   │   │   ├── transfer_commands.rs
│   │   │   ├── draft_commands.rs
│   │   │   ├── points_commands.rs
│   │   │   └── stats_commands.rs
│   │   │
│   │   ├── engines/             # 核心计算引擎
│   │   │   ├── mod.rs
│   │   │   ├── match_simulation.rs   # 比赛模拟引擎
│   │   │   ├── transfer.rs           # 转会引擎
│   │   │   ├── financial.rs          # 财政引擎
│   │   │   ├── points_calculation.rs # 积分计算引擎
│   │   │   ├── honor.rs              # 荣誉引擎
│   │   │   ├── draft.rs              # 选秀引擎
│   │   │   ├── market_value.rs       # 身价计算引擎
│   │   │   └── season_progress.rs    # 赛季进度引擎
│   │   │
│   │   ├── models/              # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── player.rs
│   │   │   ├── team.rs
│   │   │   ├── tournament.rs
│   │   │   ├── match_game.rs
│   │   │   ├── honor.rs
│   │   │   ├── season.rs
│   │   │   ├── draft.rs
│   │   │   └── player_stats.rs
│   │   │
│   │   ├── services/            # 业务服务层
│   │   │   ├── mod.rs
│   │   │   ├── game_flow.rs     # 游戏流程服务
│   │   │   ├── league.rs        # 联赛服务
│   │   │   └── honor.rs         # 荣誉服务
│   │   │
│   │   └── db/                  # 数据库操作
│   │       ├── mod.rs
│   │       ├── connection.rs    # 数据库连接与迁移
│   │       └── repository/      # 数据仓库
│   │           ├── mod.rs
│   │           ├── player_repository.rs
│   │           ├── team_repository.rs
│   │           ├── match_repository.rs
│   │           ├── honor_repository.rs
│   │           └── stats_repository.rs
│   │
│   ├── Cargo.toml               # Rust 依赖配置
│   └── tauri.conf.json          # Tauri 配置
│
├── docs/                         # 项目文档
│   ├── README.md                # 文档索引
│   ├── 01-overview/             # 项目概览
│   ├── 02-game-design/          # 游戏设计
│   ├── 03-core-systems/         # 核心系统
│   ├── 04-technical/            # 技术文档
│   ├── 05-ai/                   # AI 系统
│   └── archive/                 # 归档文档
│
├── .claude/                      # Claude Code 配置
│   └── skills/                  # 技能文档
│
├── package.json                 # Node.js 依赖
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
└── CLAUDE.md                   # Claude Code 项目指南
```

## 核心目录说明

### 前端 (src/)

| 目录 | 说明 |
|------|------|
| `api/` | Tauri 命令的前端封装，所有后端调用通过此层 |
| `components/` | 可复用的 Vue 组件，按功能模块分类 |
| `engines/` | 前端计算逻辑，如选手发挥值计算 |
| `stores/` | Pinia 状态管理，管理全局状态 |
| `views/` | 页面级组件，对应路由 |
| `types/` | TypeScript 类型定义 |

### 后端 (src-tauri/src/)

| 目录 | 说明 |
|------|------|
| `commands/` | Tauri Command 定义，是前后端通信的接口层 |
| `engines/` | 核心业务逻辑和算法实现 |
| `models/` | 数据结构定义，与数据库表对应 |
| `services/` | 业务服务层，组织多个引擎协作 |
| `db/` | 数据库连接、迁移和仓库操作 |

### 文档 (docs/)

| 目录 | 说明 |
|------|------|
| `01-overview/` | 项目架构、技术栈、目录结构 |
| `02-game-design/` | 游戏策划、赛事设计、玩法规则 |
| `03-core-systems/` | 各子系统的详细设计文档 |
| `04-technical/` | 数据库、API、开发指南 |
| `05-ai/` | AI 决策系统设计 |

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
