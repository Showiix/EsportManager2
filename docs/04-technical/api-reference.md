# API 接口参考

## 概述

EsportManager 2 使用 **Tauri Commands** 作为前后端通信机制。所有 API 接口通过 `invoke` 方法调用 Rust 后端命令。

## 调用方式

### 前端调用

```typescript
// src/api/tauri.ts
import { invoke } from '@tauri-apps/api/core';

// 示例：获取时间状态
const timeState = await invoke('get_time_state', { saveId: 'xxx' });

// 示例：模拟比赛
const result = await invoke('simulate_match', { matchId: 123 });
```

### Rust 命令定义

```rust
// src-tauri/src/commands/xxx_commands.rs
#[tauri::command]
pub async fn get_time_state(
    pool: State<'_, SqlitePool>,
    save_id: String
) -> Result<GameTimeState, String> {
    // 实现逻辑
}
```

## 命令分类

### 1. 存档管理 (save_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `create_save` | `name: String` | `Save` | 创建新存档 |
| `get_saves` | - | `Vec<Save>` | 获取所有存档 |
| `load_save` | `save_id: String` | `Save` | 加载存档 |
| `delete_save` | `save_id: String` | `bool` | 删除存档 |

### 2. 时间推进 (time_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_time_state` | `save_id` | `GameTimeState` | 获取时间状态 |
| `time_init_phase` | `save_id` | `PhaseInitResult` | 初始化当前阶段 |
| `complete_and_advance` | `save_id` | `AdvanceResult` | 完成并推进 |
| `fast_forward_to` | `save_id, target` | `FastForwardResult` | 快进到指定阶段 |

### 3. 比赛相关 (match_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_tournament_matches` | `tournament_id` | `Vec<Match>` | 获取赛事比赛列表 |
| `simulate_match` | `match_id` | `MatchResult` | 模拟单场比赛 |
| `simulate_all_matches` | `tournament_id` | `Vec<MatchResult>` | 模拟所有比赛 |
| `get_match_result` | `match_id` | `MatchResult` | 获取比赛结果 |

### 4. 比赛详情 (match_detail_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_match_detail` | `match_id` | `MatchDetail` | 获取比赛详情 |
| `get_game_details` | `match_id` | `Vec<GameDetail>` | 获取小局详情 |
| `get_player_stats` | `game_id` | `Vec<PlayerStats>` | 获取选手表现 |

### 5. 战队管理 (team_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_team` | `team_id` | `Team` | 获取战队信息 |
| `get_teams_by_region` | `region_id` | `Vec<Team>` | 获取赛区战队 |
| `get_team_roster` | `team_id` | `Vec<Player>` | 获取战队阵容 |
| `set_starter` | `player_id, is_starter` | `bool` | 设置首发 |
| `update_team_power` | `team_id` | `Team` | 更新战队战力 |

### 6. 选手管理 (game_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_player` | `player_id` | `Player` | 获取选手信息 |
| `get_players_by_team` | `team_id` | `Vec<Player>` | 获取队伍选手 |
| `get_free_agents` | `save_id` | `Vec<Player>` | 获取自由球员 |
| `update_player_condition` | `player_id` | `Player` | 更新选手状态 |

### 7. 赛事管理 (event_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_tournament` | `tournament_id` | `Tournament` | 获取赛事信息 |
| `get_standings` | `tournament_id` | `Vec<Standing>` | 获取积分榜 |
| `get_tournament_bracket` | `tournament_id` | `Bracket` | 获取淘汰赛对阵 |

### 8. 国际赛事 (international_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_msi_state` | `save_id, season_id` | `MSIState` | 获取 MSI 状态 |
| `get_worlds_state` | `save_id, season_id` | `WorldsState` | 获取世界赛状态 |
| `get_super_state` | `save_id, season_id` | `SuperState` | 获取 Super 赛状态 |

### 9. 转会系统 (transfer_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_transfer_window_state` | `save_id` | `TransferState` | 获取转会窗口状态 |
| `execute_transfer_round` | `save_id, round` | `RoundResult` | 执行转会轮次 |
| `get_transfer_market` | `save_id` | `Vec<Listing>` | 获取转会市场 |
| `make_transfer_offer` | `params` | `OfferResult` | 发起转会报价 |
| `sign_free_agent` | `player_id, team_id` | `bool` | 签约自由球员 |

### 10. 选秀系统 (draft_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_draft_pool` | `save_id, region_id` | `Vec<Prospect>` | 获取选秀池 |
| `get_draft_order` | `save_id, region_id` | `Vec<DraftPick>` | 获取选秀顺位 |
| `execute_draft_pick` | `pick_id, prospect_id` | `DraftResult` | 执行选秀 |
| `auto_complete_draft` | `save_id` | `Vec<DraftResult>` | 自动完成选秀 |

### 11. 选秀权拍卖 (draft_auction_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_auction_state` | `save_id` | `AuctionState` | 获取拍卖状态 |
| `place_bid` | `save_id, amount` | `BidResult` | 出价 |
| `complete_auction` | `save_id` | `AuctionResult` | 完成拍卖 |

### 12. 财务系统 (finance_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_team_finance` | `team_id` | `TeamFinance` | 获取战队财务 |
| `get_finance_summary` | `team_id` | `FinanceSummary` | 获取财务摘要 |
| `get_transactions` | `team_id, season_id` | `Vec<Transaction>` | 获取交易记录 |
| `distribute_prize` | `tournament_id` | `bool` | 分发奖金 |

### 13. 荣誉系统 (honor_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_honor_hall` | `save_id` | `HonorHall` | 获取荣誉殿堂 |
| `get_team_honors` | `team_id` | `Vec<Honor>` | 获取战队荣誉 |
| `get_player_honors` | `player_id` | `Vec<Honor>` | 获取选手荣誉 |
| `award_tournament_honors` | `tournament_id` | `Vec<Honor>` | 颁发赛事荣誉 |

### 14. 积分系统 (points_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_annual_points` | `save_id, season_id` | `Vec<TeamPoints>` | 获取年度积分 |
| `get_global_rankings` | `save_id` | `Vec<Ranking>` | 获取全球排名 |
| `get_points_detail` | `team_id, season_id` | `Vec<PointsDetail>` | 获取积分明细 |

### 15. 统计系统 (stats_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_player_season_stats` | `player_id, season_id` | `PlayerStats` | 获取选手赛季统计 |
| `get_player_rankings` | `season_id, position` | `Vec<PlayerRank>` | 获取选手排行 |
| `record_performance` | `params` | `bool` | 记录选手表现 |

### 16. 年度颁奖 (awards_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_annual_awards` | `save_id, season_id` | `AnnualAwards` | 获取年度颁奖 |
| `calculate_mvp` | `save_id, season_id` | `Player` | 计算年度 MVP |
| `calculate_top20` | `save_id, season_id` | `Vec<Player>` | 计算年度 Top20 |

### 17. 查询工具 (query_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `search_players` | `query` | `Vec<Player>` | 搜索选手 |
| `search_teams` | `query` | `Vec<Team>` | 搜索战队 |

### 18. 开发工具 (dev_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `reset_database` | - | `bool` | 重置数据库 |
| `generate_test_data` | - | `bool` | 生成测试数据 |

### 19. 日志系统 (log_commands.rs)

| 命令 | 参数 | 返回值 | 描述 |
|------|------|-------|------|
| `get_logs` | `level, limit` | `Vec<LogEntry>` | 获取日志 |
| `clear_logs` | - | `bool` | 清空日志 |

## 错误处理

### 错误码定义

错误码定义在 `src-tauri/src/errors/error_codes.rs`：

```rust
pub enum ErrorCode {
    // 通用错误 1xxx
    Unknown = 1000,
    DatabaseError = 1001,
    NotFound = 1002,

    // 存档错误 2xxx
    SaveNotFound = 2001,
    SaveCorrupted = 2002,

    // 比赛错误 3xxx
    MatchNotFound = 3001,
    MatchAlreadyCompleted = 3002,

    // 转会错误 4xxx
    InsufficientFunds = 4001,
    PlayerNotAvailable = 4002,

    // ...
}
```

### 错误返回格式

```typescript
interface ApiError {
    code: number;
    message: string;
    details?: string;
}
```

## 前端 Store 对应

| Store | 主要调用的 Commands |
|-------|---------------------|
| `useTimeStore` | time_commands |
| `useTeamStore` | team_commands |
| `usePlayerStore` | game_commands |
| `useMatchDetailStore` | match_detail_commands |
| `useTournamentStore` | event_commands |
| `useTransferStore` | transfer_commands |
| `useDraftStore` | draft_commands |
| `useFinanceStore` | finance_commands |
| `useHonorStore` | honor_commands |
| `usePointsStore` | points_commands |

## 文件位置

| 文件 | 说明 |
|-----|------|
| `src-tauri/src/commands/mod.rs` | 命令模块入口 |
| `src-tauri/src/commands/*_commands.rs` | 各模块命令 |
| `src/api/tauri.ts` | 前端 API 封装 |
| `src/api/client.ts` | API 客户端工具 |
