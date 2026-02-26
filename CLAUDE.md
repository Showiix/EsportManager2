# CLAUDE.md - EsportManager 2 项目指南

## 项目概述

EsportManager 2 是一款电竞经理模拟游戏，采用 **Tauri + Vue 3 + Rust** 技术栈构建。游戏模拟了完整的电竞联赛生态，包括四大赛区（LPL、LCK、LEC、LCS）、15个赛季阶段、选手成长系统、转会市场等核心玩法。

### 核心特性

- **4大赛区**: LPL（中国）、LCK（韩国）、LEC（欧洲）、LCS（北美），共56支战队、280名选手
- **15个赛季阶段**: 从春季赛到赛季结束的完整年度周期
- **正态分布比赛模拟**: 基于战力值和标准差的概率胜负判定
- **完整选手生命周期**: 成长、巅峰、衰退、退役
- **8轮制转会系统**: AI性格驱动的自动转会决策
- **跨赛区转会偏好**: 中国选手更倾向留在LPL

## 目录结构

```
EsportManager2-Backend/
├── src/                          # 前端源码 (Vue 3 + TypeScript)
│   ├── api/tauri.ts             # Tauri 命令调用封装
│   ├── components/              # Vue 组件 (28个，含 common/layout/match/msi/worlds/icp/super/clauch/transfer/finance/player/settings/dev)
│   ├── engines/                 # 前端计算引擎 (PlayerEngine, PowerEngine)
│   ├── stores/                  # Pinia 状态管理 (16个 Store，已清理旧 axios 版)
│   ├── views/                   # 页面视图 (48个)
│   ├── types/                   # TypeScript 类型定义 (6个)
│   └── router/                  # Vue Router 路由
│
├── src-tauri/src/               # Rust 后端源码
│   ├── commands/                # Tauri Commands (23个命令模块)
│   ├── engines/                 # 核心计算引擎 (17个引擎)
│   │   ├── match_simulation.rs # 比赛模拟引擎
│   │   ├── transfer/           # 转会引擎（已拆分为模块目录）
│   │   │   ├── mod.rs          # TransferEngine 定义 + 主流程
│   │   │   ├── cache.rs        # TransferCache 内存缓存
│   │   │   ├── round1_settlement.rs ~ round7_remedy.rs  # 7轮转会逻辑
│   │   │   ├── scoring.rs      # 匹配度/意愿度/策略算法
│   │   │   └── tests.rs        # 25个单元测试
│   │   ├── financial.rs        # 财政引擎
│   │   ├── points_calculation.rs # 积分计算引擎
│   │   ├── traits.rs           # 选手特性系统
│   │   ├── condition.rs        # 选手状态/体力
│   │   ├── satisfaction.rs     # 满意度引擎
│   │   ├── draft_auction.rs    # 选秀权拍卖
│   │   └── ...
│   ├── models/                  # 数据模型 (20个)
│   ├── services/                # 业务服务层 (11个)
│   │   ├── game_flow/          # 游戏流程核心（已拆分为模块目录）
│   │   │   ├── mod.rs          # GameFlowService 定义 + 公开类型
│   │   │   ├── phase_init.rs   # 阶段初始化（15个赛季阶段）
│   │   │   ├── phase_complete.rs # 阶段完成（荣誉/统计/推进）
│   │   │   ├── tournament_init.rs # 赛事初始化（常规赛/季后赛/大师赛）
│   │   │   ├── tournament_complete.rs # 赛事完成（排名/积分/奖金）
│   │   │   ├── time_system.rs  # 时间推进系统
│   │   │   ├── match_simulation.rs # 比赛模拟（含特性系统）
│   │   │   ├── annual_awards.rs # 年度颁奖
│   │   │   ├── market_value.rs # 身价与品牌价值
│   │   │   └── season_management.rs # 赛季管理
│   │   ├── league_service.rs   # 联赛服务
│   │   ├── honor_service.rs    # 荣誉服务
│   │   ├── init_service.rs     # 初始化服务
│   │   ├── player_data.rs      # 选手初始数据
│   │   └── ...
│   └── db/                      # 数据库操作 (SQLite)
│       ├── connection.rs       # 连接与迁移
│       └── repository/         # 数据仓库（已拆分为模块目录）
│           ├── mod.rs          # 统一 re-export
│           ├── helpers.rs      # 27个共享 row_to_*/parse_* 函数
│           ├── save.rs         # SaveRepository
│           ├── team.rs         # TeamRepository
│           ├── player.rs       # PlayerRepository + PlayerStatsRepository
│           ├── match_repo.rs   # MatchRepository + MatchGameDetailRepository
│           ├── tournament.rs   # TournamentRepository + TournamentResultRepository
│           ├── honor.rs        # HonorRepository
│           ├── points.rs       # PointsRepository
│           └── ...             # 其余 Repository 各自独立文件
│
├── docs/                        # 项目文档
│   └── architecture-refactor.md # 架构重构记录（详细拆分说明）
└── .claude/skills/              # Claude Code 技能文档
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| UI 组件库 | Element Plus |
| 状态管理 | Pinia |
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| 数据库 | SQLite (sqlx) |
| 前后端通信 | Tauri Commands |

## 核心系统

### 1. 时间推进系统 (Time System)

**文件**: `src-tauri/src/services/game_flow/` (模块目录), `src-tauri/src/engines/season_progress.rs`

管理游戏的15个赛季阶段流转：

```
春季赛常规赛 → 春季赛季后赛 → MSI → 马德里大师赛 →
夏季赛常规赛 → 夏季赛季后赛 → Claude洲际赛 → 世界赛 →
上海大师赛 → ICP洲际赛 → Super洲际赛 → 年度颁奖 →
转会期 → 选秀(每4年) → 赛季结束
```

**主要命令**:
- `get_time_state` - 获取游戏时间状态
- `time_init_phase` - 初始化当前阶段
- `complete_and_advance` - 完成并推进
- `fast_forward_to` - 快进到指定阶段

### 2. 比赛模拟系统 (Match Simulation)

**文件**: `src-tauri/src/engines/match_simulation.rs`

基于正态分布的胜负判定：
- 标准差 σ = 6.0
- 每局从 N(战力值, σ²) 采样发挥值
- 发挥值高者获胜

**胜率对照** (战力差 → 强队胜率):
- 0 → 50%, 5 → 60%, 10 → 69%, 20 → 82%

### 3. 选手系统 (Player System)

**文件**: `src-tauri/src/models/player.rs`

选手核心属性：
- `ability` (0-100): 当前能力值
- `potential` (0-100): 潜力上限
- `stability` (0-100): 稳定性
- `loyalty` (0-100): 忠诚度
- `tag`: 成长标签 (Ordinary/Normal/Genius)

**成长规则**:
- 30岁前: 每赛季 +1/+2/+3 能力（根据tag）
- 30岁后: 每赛季 -1 能力
- 35岁+且能力<65: 退役

### 4. 转会系统 (Transfer System)

**文件**: `src-tauri/src/engines/transfer/` (模块目录)

7轮转会流程：
1. 赛季结算（年龄/能力更新） → `round1_settlement.rs`
2. 双向评估 → `round2_evaluation.rs`
3. 续约谈判 → `round3_renewal.rs`
4. 自由球员竞标 → `round4_free_agent.rs`
5. 合同选手转会 → `round5_contracted.rs`
6. 财务调整 → `round6_financial.rs`
7. 最终补救 → `round7_remedy.rs`

**跨赛区偏好值**:
- `home_region_id`: 选手出生赛区
- `region_loyalty`: 赛区偏好值（LPL 75-90, LCK 55-75, LEC 45-65, LCS 40-60）
- 跨赛区转会意愿度 = 基础意愿 × (100 - region_loyalty) / 100

### 5. 财政系统 (Financial System)

**文件**: `src-tauri/src/engines/financial.rs`

收入来源：赛事奖金、赞助收入、联赛分成、转会收入
支出项目：选手薪资、运营成本、转会支出

**主要奖金池**:
- 世界赛: 1.2亿（冠军5000万）
- Super洲际赛: 1.5亿（冠军6000万）
- MSI: 4000万（冠军2000万）

### 6. 荣誉系统 (Honor System)

**文件**: `src-tauri/src/engines/honor.rs`

荣誉类型：
- 战队荣誉: 冠军/亚军/季军/殿军
- 选手荣誉: 冠军成员、赛事MVP、决赛MVP
- 年度荣誉: 年度MVP、年度Top20、最佳位置

### 7. 年度积分系统 (Annual Points)

**文件**: `src-tauri/src/engines/points_calculation.rs`

积分用于决定 Super 洲际赛参赛资格（年度 Top16）。

积分配置示例：
- 世界赛冠军: 20分, 亚军: 16分
- 联赛季后赛冠军: 12分

### 8. 数据中心系统 (Data Center)

**文件**: `src-tauri/src/db/repository/` (模块目录)

记录选手赛季表现：
- 影响力统计（单局影响力 = 选手发挥 - 队伍平均） → `player_tournament_stats.rs`
- 稳定性评分 → `player.rs` (PlayerStatsRepository)
- 年度Top得分 → `player.rs` (PlayerStatsRepository)

## 数据库设计

主要表：
- `saves`: 存档
- `teams`: 战队
- `players`: 选手
- `tournaments`: 赛事
- `matches`: 比赛
- `match_games`: 比赛局数详情
- `standings`: 积分榜
- `honors`: 荣誉记录
- `transfer_events`: 转会事件
- `transfer_windows`: 转会窗口
- `financial_transactions`: 财务交易
- `draft_pool`: 选秀池（持久化）
- `draft_players`: 当届选秀名单
- `draft_results`: 选秀结果
- `annual_points_detail`: 年度积分明细
- `player_season_stats`: 选手赛季统计

## 开发命令

```bash
# 启动开发环境
npm run tauri dev

# 前端类型检查
npx vue-tsc --noEmit

# Rust 编译检查
cargo check --manifest-path src-tauri/Cargo.toml

# 运行测试
cargo test --manifest-path src-tauri/Cargo.toml   # Rust 后端 (170+ tests)
npm test                                           # Vue 前端 (vitest, 46 tests)
npm run test:watch                                 # 前端监视模式
npm run test:coverage                              # 前端覆盖率

# 构建发布版本
npm run tauri build
```

## Claude Code 技能

项目配置了以下技能（位于 `.claude/skills/`）：

| 技能 | 用途 |
|------|------|
| `time-system` | 时间推进系统修改 |
| `match-simulation-system` | 比赛模拟算法修改 |
| `player-system` | 选手属性/成长/身价修改 |
| `transfer-system` | 转会规则/AI决策修改 |
| `financial-system` | 财务规则/奖金配置修改 |
| `financial-units` | 金额单位规范（元/万元） |
| `honor-system` | 荣誉类型/MVP计算修改 |
| `annual-points-system` | 积分规则修改 |
| `data-center-system` | 统计计算/排行榜修改 |
| `performance-monitoring-system` | 性能监测/命令计时 |
| `logging-system` | 日志与错误处理 |
| `trait-system` | 选手特性定义/触发/生成规则 |
| `player-contract-center` | 选手合同中心页面 |
| `menu-manager` | 侧边栏菜单管理 |
| `commit-generator` | 生成规范的 commit message |
| `debug-doc-writer` | Bug 修复文档记录 |
| `doc-writer` | 项目文档编写 |
| `db-backup` | 数据库备份 |
| `skill-creator` | 创建新技能 |
| `page-style` | 页面视觉风格规范 |

## 常见开发任务

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands/` 中添加函数
2. 使用 `#[tauri::command]` 宏标注
3. 在 `main.rs` 中注册命令
4. 在 `src/api/tauri.ts` 中添加前端调用封装

### 添加数据库迁移

在 `src-tauri/src/db/connection.rs` 的 `run_migrations` 函数中添加：

```rust
if !column_names.contains(&"new_column") {
    sqlx::query("ALTER TABLE table ADD COLUMN new_column TYPE")
        .execute(pool).await?;
}
```

### 修改比赛模拟参数

编辑 `src-tauri/src/engines/match_simulation.rs`:
- 调整 `std_dev` 改变随机性
- 修改 `simulate_game` 改变单局逻辑

### 修改转会逻辑

编辑 `src-tauri/src/engines/transfer/` 目录:
- `scoring.rs` → `calculate_willingness`: 转会意愿计算, `calculate_match_score`: 匹配度计算
- `round1_settlement.rs` ~ `round7_remedy.rs` → 各轮次执行逻辑
- `cache.rs` → TransferCache 缓存结构

## 注意事项

1. **金额单位**: 内部使用**元**，显示时转换为**万元**
2. **首发限制**: 每队每位置最多1名首发
3. **合同到期**: `contract_end_season == current_season` 时变为自由球员
4. **跨赛区转会**: 受 `region_loyalty` 影响，LPL选手最不愿意外流
