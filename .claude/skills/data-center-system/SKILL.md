---
name: data-center-system
description: 电竞经理游戏的数据中心系统。管理选手赛季统计、影响力排行、赛事表现记录、身价变化追踪。当需要修改统计计算、排行榜逻辑、年度评分时使用此技能。
---

# 数据中心系统 (Data Center System)

## Overview

数据中心系统负责收集、存储和分析选手在各项赛事中的表现数据。它为荣誉系统、选手系统、转会系统提供数据支撑，是MVP计算、年度评选、身价评估的核心依据。

## 核心组件

### PlayerStatsRepository

**文件**: `src-tauri/src/db/repository/stats_repository.rs`

选手统计数据仓库，提供增删改查操作。

**核心方法**:
- `get_or_create(...)` - 获取或创建选手赛季统计
- `update(stats)` - 更新统计记录
- `get_season_ranking(...)` - 获取赛季影响力排行
- `get_position_ranking(...)` - 获取分位置排行
- `get_by_player(...)` - 获取选手历史统计
- `get_by_team(...)` - 获取队伍选手统计

### PlayerTournamentStatsRepository

**文件**: `src-tauri/src/db/repository/tournament_stats_repository.rs`

选手赛事统计仓库，记录单项赛事中的表现。

**核心方法**:
- `upsert(stats)` - 插入或更新赛事统计
- `get_by_player_tournament(...)` - 获取选手特定赛事统计
- `get_mvp_candidates(...)` - 获取MVP候选（按影响力排序）

## 数据结构

### PlayerSeasonStatistics (选手赛季统计)

**文件**: `src-tauri/src/models/stats.rs`

```rust
pub struct PlayerSeasonStatistics {
    pub id: u64,
    pub save_id: String,
    pub season_id: i64,
    pub player_id: i64,
    pub player_name: String,
    pub team_id: Option<i64>,
    pub region_id: Option<String>,
    pub position: String,

    // 比赛数据
    pub games_played: i32,          // 出场局数
    pub total_impact: f64,          // 总影响力
    pub avg_impact: f64,            // 平均影响力
    pub max_impact: f64,            // 最高单场影响力
    pub total_performance: f64,     // 总表现值
    pub avg_performance: f64,       // 平均表现
    pub best_performance: f64,      // 最佳表现

    // 荣誉加成
    pub regional_titles: i32,       // 赛区冠军数
    pub international_titles: i32,  // 国际赛冠军数
    pub champion_bonus: f64,        // 冠军加成分

    // 综合评分
    pub yearly_top_score: f64,      // 年度Top得分
}
```

**核心方法**:
```rust
impl PlayerSeasonStatistics {
    // 记录单场比赛表现
    pub fn record_game(&mut self, impact: f64, performance: f64);

    // 记录冠军
    pub fn record_championship(&mut self, is_international: bool);
}
```

### PlayerTournamentStats (选手赛事统计)

**文件**: `src-tauri/src/models/tournament_result.rs`

```rust
pub struct PlayerTournamentStats {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_type: String,
    pub player_id: u64,
    pub player_name: String,
    pub team_id: u64,
    pub team_name: String,
    pub position: String,

    pub games_played: u32,      // 出场局数
    pub games_won: u32,         // 胜场
    pub total_impact: f64,      // 总影响力
    pub avg_impact: f64,        // 平均影响力
    pub max_impact: f64,        // 最高影响力
    pub avg_performance: f64,   // 平均表现
    pub best_performance: f64,  // 最佳表现
    pub game_mvp_count: u32,    // 单场MVP次数
}
```

### PlayerRankingItem (排行榜项)

```rust
pub struct PlayerRankingItem {
    pub rank: u32,
    pub player_id: i64,
    pub player_name: String,
    pub team_id: Option<i64>,
    pub position: String,
    pub games_played: i32,
    pub avg_impact: f64,
    pub yearly_top_score: f64,
}
```

### MarketValueChange (身价变化)

```rust
pub struct MarketValueChange {
    pub id: i64,
    pub season_id: i64,
    pub player_id: i64,
    pub player_name: String,
    pub old_value: i64,
    pub new_value: i64,
    pub change_amount: i64,
    pub change_percent: f64,
    pub reason: String,
    pub created_at: String,
}
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/stats_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `record_player_performance` | 记录单场选手表现 | `PlayerSeasonStatistics` |
| `batch_record_player_performance` | 批量记录表现 | `i32` (记录数) |
| `record_championship` | 记录冠军荣誉 | `i32` (更新数) |
| `get_season_impact_ranking` | 获取赛季影响力排行 | `Vec<PlayerRankingItem>` |
| `get_position_ranking` | 获取分位置排行 | `Vec<PlayerRankingItem>` |
| `get_player_stats` | 获取选手统计 | `Vec<PlayerSeasonStatistics>` |
| `get_team_player_stats` | 获取队伍选手统计 | `Vec<PlayerSeasonStatistics>` |
| `clear_season_stats` | 清除赛季统计 | `bool` |
| `get_player_impact_history` | 获取影响力历史 | `Vec<f64>` |
| `get_tournament_mvp_ranking` | 获取赛事MVP排行 | `Vec<PlayerTournamentStats>` |
| `recalculate_yearly_scores` | 重算年度Top得分 | `i32` (更新数) |
| `get_player_market_value_changes` | 获取身价变化记录 | `Vec<MarketValueChange>` |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 记录选手表现
export async function recordPlayerPerformance(params: RecordPerformanceParams): Promise<PlayerSeasonStatistics>

// 获取赛季排行
export async function getSeasonImpactRanking(seasonId: number, limit?: number): Promise<PlayerRankingItem[]>

// 获取分位置排行
export async function getPositionRanking(seasonId: number, position: string, limit?: number): Promise<PlayerRankingItem[]>

// 获取选手统计
export async function getPlayerStats(playerId: number, seasonId?: number): Promise<PlayerSeasonStatistics[]>

// 获取赛事MVP排行
export async function getTournamentMvpRanking(tournamentId: number, limit?: number): Promise<PlayerTournamentStats[]>

// 获取身价变化
export async function getPlayerMarketValueChanges(playerId: number): Promise<MarketValueChange[]>
```

## 年度Top得分计算公式

```rust
// 综合三要素计算年度得分
yearly_top_score = avg_impact * 0.4        // 影响力权重 40%
                 + (games_played / 10.0) * 0.3  // 出场加成 30%
                 + champion_bonus * 0.3;   // 冠军加成 30%

// 冠军加成计算
champion_bonus = international_titles * 3 + regional_titles;
```

## 系统集成

### 与时间推进系统
- 比赛模拟后调用 `record_player_performance` 记录表现
- 赛事结束后更新 `PlayerTournamentStats`

### 与荣誉系统
- MVP计算基于 `PlayerTournamentStats.total_impact`
- 年度颁奖基于 `yearly_top_score` 排名

### 与选手系统
- 统计数据影响选手成长曲线
- 表现数据影响身价计算

### 与转会系统
- `avg_impact` 影响选手吸引力评估
- 历史表现影响 AI 转会决策

## 使用示例

### 记录比赛表现
```typescript
await recordPlayerPerformance({
    player_id: 1,
    player_name: "Faker",
    team_id: 1,
    position: "MID",
    impact_score: 85.5,
    actual_ability: 92,
    season_id: 1
})
```

### 获取赛季排行榜
```typescript
const ranking = await getSeasonImpactRanking(currentSeason, 20)
ranking.forEach((item, idx) => {
    console.log(`#${idx + 1} ${item.player_name} - 影响力: ${item.avg_impact.toFixed(1)}`)
})
```

### 获取MVP候选
```typescript
const mvpCandidates = await getTournamentMvpRanking(tournamentId, 10)
const mvp = mvpCandidates[0]  // 影响力最高的选手
console.log(`赛事MVP: ${mvp.player_name} (${mvp.total_impact.toFixed(1)}影响力)`)
```

## 数据库表

### player_season_statistics

```sql
CREATE TABLE player_season_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    team_id INTEGER,
    region_id TEXT,
    position TEXT,
    games_played INTEGER DEFAULT 0,
    total_impact REAL DEFAULT 0,
    avg_impact REAL DEFAULT 0,
    max_impact REAL DEFAULT 0,
    total_performance REAL DEFAULT 0,
    avg_performance REAL DEFAULT 0,
    best_performance REAL DEFAULT 0,
    regional_titles INTEGER DEFAULT 0,
    international_titles INTEGER DEFAULT 0,
    champion_bonus REAL DEFAULT 0,
    yearly_top_score REAL DEFAULT 0,
    UNIQUE(save_id, season_id, player_id)
);
```

### player_tournament_stats

```sql
CREATE TABLE player_tournament_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    tournament_type TEXT NOT NULL,
    player_id INTEGER NOT NULL,
    player_name TEXT NOT NULL,
    team_id INTEGER NOT NULL,
    team_name TEXT NOT NULL,
    position TEXT NOT NULL,
    games_played INTEGER DEFAULT 0,
    games_won INTEGER DEFAULT 0,
    total_impact REAL DEFAULT 0,
    avg_impact REAL DEFAULT 0,
    max_impact REAL DEFAULT 0,
    avg_performance REAL DEFAULT 0,
    best_performance REAL DEFAULT 0,
    game_mvp_count INTEGER DEFAULT 0,
    UNIQUE(save_id, tournament_id, player_id)
);
```

## 注意事项

1. **数据一致性**: 记录表现后需同时更新 `player_season_statistics` 和 `player_tournament_stats`
2. **性能优化**: 批量记录使用 `batch_record_player_performance`
3. **年度得分**: 新赛季开始时调用 `recalculate_yearly_scores` 确保公式一致
4. **身价变化**: 变化记录在 `market_value_changes` 表中，支持历史追溯
