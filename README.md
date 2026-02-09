# EsportManager 2

一款电竞经理模拟游戏，基于 **Tauri 2 + Vue 3 + Rust** 构建。模拟完整的英雄联盟电竞联赛生态，涵盖四大赛区、多项国际赛事、选手成长与转会系统。

## 游戏特色

- **四大赛区** - LPL（中国）、LCK（韩国）、LEC（欧洲）、LCS（北美），共 56 支战队、280 名选手
- **完整赛季周期** - 15 个赛季阶段，从春季常规赛到赛季结算，覆盖联赛、国际赛事、转会期
- **7 项赛事** - 春/夏季联赛、MSI、马德里大师赛、Claude 洲际赛、世界赛（瑞士轮+淘汰赛）、上海大师赛、ICP 洲际赛、Super 洲际赛
- **正态分布比赛模拟** - 基于战力值与标准差 (sigma=6.0) 的概率胜负判定，支持 BO1/BO3/BO5 赛制
- **选手成长系统** - 能力值、潜力、稳定性、忠诚度等多维属性，30 岁前成长、30 岁后衰退、35 岁+低能力退役
- **14 种选手特性** - 影响比赛中的能力/稳定性/状态修正
- **8 轮制转会系统** - AI 性格驱动的续约、竞标、跨赛区转会，中国选手有更高赛区忠诚度
- **财政系统** - 赛事奖金、赞助收入、薪资支出、转会预算，世界赛冠军奖金 5000 万
- **年度积分** - 各赛事积分累计，决定 Super 洲际赛参赛资格（年度 Top16）
- **荣誉殿堂** - 战队/选手冠军记录、赛事 MVP、年度 Top20
- **Meta 版本系统** - 20 种 Meta 版本影响位置权重与加权战力
- **每 4 年选秀** - 新秀球员选秀与选秀权拍卖

## 赛季流程

```
春季常规赛 → 春季季后赛 → MSI → 马德里大师赛 →
夏季常规赛 → 夏季季后赛 → Claude洲际赛 → 世界赛 →
上海大师赛 → ICP洲际赛 → Super洲际赛 → 年度颁奖 →
转会期 → 选秀(每4年) → 赛季结束
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | Vue 3 + TypeScript |
| UI 组件 | Element Plus |
| 状态管理 | Pinia |
| 图表 | ECharts |
| 后端 | Rust |
| 数据库 | SQLite (sqlx) |
| 测试 | Vitest (前端) + cargo test (后端) |

## 项目结构

```
EsportManager2-Backend/
├── src/                          # Vue 前端
│   ├── api/                     # Tauri 命令调用封装
│   ├── components/              # Vue 组件 (14 个模块)
│   │   ├── common/              # 通用组件
│   │   ├── layout/              # 布局组件
│   │   ├── match/               # 比赛相关
│   │   ├── msi/                 # MSI 赛事
│   │   ├── worlds/              # 世界赛
│   │   ├── clauch/              # Claude 洲际赛
│   │   ├── icp/                 # ICP 洲际赛
│   │   ├── super/               # Super 洲际赛
│   │   ├── transfer/            # 转会系统
│   │   ├── finance/             # 财务系统
│   │   └── player/              # 选手系统
│   ├── engines/                 # 前端计算引擎
│   ├── stores/                  # Pinia Store (29 个)
│   ├── views/                   # 页面视图 (52 个)
│   ├── types/                   # TypeScript 类型定义
│   └── router/                  # Vue Router
│
├── src-tauri/src/               # Rust 后端
│   ├── commands/                # Tauri Commands (23 个模块)
│   ├── engines/                 # 核心引擎 (19 个)
│   │   ├── match_simulation.rs  # 比赛模拟
│   │   ├── transfer.rs          # 转会引擎
│   │   ├── financial.rs         # 财政引擎
│   │   ├── points_calculation.rs # 积分计算
│   │   ├── traits.rs            # 选手特性
│   │   ├── meta_engine.rs       # Meta 版本
│   │   ├── draft.rs             # 选秀系统
│   │   ├── honor.rs             # 荣誉系统
│   │   └── ...
│   ├── models/                  # 数据模型 (20 个)
│   ├── services/                # 业务服务层 (11 个)
│   └── db/                      # 数据库连接与仓库
│
└── docs/                        # 项目文档
```

## 环境要求

- **Node.js** >= 18
- **Rust** >= 1.77.2
- **Tauri CLI** 2.x

## 快速开始

```bash
# 安装前端依赖
npm install

# 启动开发环境 (前端 + Rust 后端)
npm run tauri dev

# 构建发布版本
npm run tauri build
```

## 开发命令

```bash
# 前端类型检查
npx vue-tsc --noEmit

# Rust 编译检查
cargo check --manifest-path src-tauri/Cargo.toml

# 运行后端测试 (170+ tests)
cargo test --manifest-path src-tauri/Cargo.toml

# 运行前端测试
npm test

# 前端测试 (watch 模式)
npm run test:watch

# 前端测试覆盖率
npm run test:coverage
```

## 比赛模拟算法

基于正态分布的胜负判定：

- 每局比赛，双方各从 `N(战力值, 6.0^2)` 采样一个发挥值
- 发挥值高者获胜

**战力差与胜率对照：**

| 战力差 | 强队胜率 |
|--------|---------|
| 0 | 50% |
| 5 | 60% |
| 10 | 69% |
| 20 | 82% |

## 许可证

MIT
