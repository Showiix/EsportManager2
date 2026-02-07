# EsportManager 2 文档中心

> 电竞经理模拟游戏 - 基于 Tauri + Vue 3 + Rust 技术栈

## 快速导航

| 分类 | 说明 | 适合人群 |
|------|------|----------|
| [项目概览](./01-overview/) | 架构设计、技术栈、目录结构 | 新成员入门 |
| [游戏设计](./02-game-design/) | 策划案、赛事体系、玩法规则 | 策划/产品 |
| [核心系统](./03-core-systems/) | 各子系统详细设计文档 | 开发人员 |
| [技术文档](./04-technical/) | 数据库、API、前后端指南 | 开发人员 |
| [AI 系统](./05-ai/) | AI 决策引擎设计 | 开发人员 |

> 📝 **编写新文档？** 请阅读 [文档编写指南](./CONTRIBUTING.md)

---

## 01-overview 项目概览

- [architecture.md](./01-overview/architecture.md) - 系统架构与调用链
- [tech-stack.md](./01-overview/tech-stack.md) - 技术栈说明
- [directory-structure.md](./01-overview/directory-structure.md) - 项目目录结构

## 02-game-design 游戏设计

- [game-concept.md](./02-game-design/game-concept.md) - 游戏概念与定位
- [season-phases.md](./02-game-design/season-phases.md) - 赛季阶段设计
- [regions-and-teams.md](./02-game-design/regions-and-teams.md) - 赛区与战队设计
- [tournaments.md](./02-game-design/tournaments.md) - 赛事体系设计

## 03-core-systems 核心系统

| 系统 | 文档 | 核心文件 |
|------|------|----------|
| 时间推进 | [time-system.md](./03-core-systems/time-system.md) | `services/game_flow.rs` |
| 比赛模拟 | [match-simulation.md](./03-core-systems/match-simulation.md) | `engines/match_simulation.rs` |
| 选手系统 | [player-system.md](./03-core-systems/player-system.md) | `models/player.rs` |
| 转会系统 | [transfer-system.md](./03-core-systems/transfer-system.md) | `engines/transfer.rs` |
| 转会窗口关闭 | [transfer-window-close.md](./03-core-systems/transfer-window-close.md) | `engines/transfer.rs` |
| 竞价分析 | [transfer-bid-analysis.md](./03-core-systems/transfer-bid-analysis.md) | `engines/transfer.rs` |
| 财政系统 | [financial-system.md](./03-core-systems/financial-system.md) | `engines/financial.rs` |
| 荣誉系统 | [honor-system.md](./03-core-systems/honor-system.md) | `engines/honor.rs` |
| 积分系统 | [points-system.md](./03-core-systems/points-system.md) | `engines/points_calculation.rs` |
| 数据中心 | [data-center.md](./03-core-systems/data-center.md) | `db/repository/stats_repository.rs` |
| 选秀系统 | [draft-system.md](./03-core-systems/draft-system.md) | `engines/draft.rs` |
| 性能监测 | [performance-monitoring.md](./03-core-systems/performance-monitoring.md) | `services/perf_service.rs` |

## 04-technical 技术文档

- [database-schema.md](./04-technical/database-schema.md) - 数据库设计
- [api-reference.md](./04-technical/api-reference.md) - Tauri Commands API 参考
- [frontend-guide.md](./04-technical/frontend-guide.md) - 前端开发指南
- [backend-guide.md](./04-technical/backend-guide.md) - 后端开发指南
- [logging-system.md](./04-technical/logging-system.md) - 日志系统

## 05-ai AI 系统

- [ai-decision-engine.md](./05-ai/ai-decision-engine.md) - AI 决策引擎

---

## 开发命令

```bash
# 启动开发环境
npm run tauri dev

# 前端类型检查
npx vue-tsc --noEmit

# Rust 编译检查
cargo check --manifest-path src-tauri/Cargo.toml

# 构建发布版本
npm run tauri build
```

## 核心特性

- **4大赛区**: LPL（中国）、LCK（韩国）、LEC（欧洲）、LCS（北美）
- **15个赛季阶段**: 春季赛 → MSI → 夏季赛 → 世界赛 → Super洲际赛 → 转会期
- **正态分布比赛模拟**: 基于战力值和标准差的概率胜负判定
- **选手生命周期**: 成长、巅峰、衰退、退役
- **8轮制转会系统**: AI性格驱动的自动转会决策，含选秀权拍卖

---

**文档版本**: 2.1
**最后更新**: 2026-02-07
