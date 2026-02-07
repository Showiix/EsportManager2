# 数据中心系统

## 概述

数据中心系统收集和分析选手比赛表现数据，用于排行榜、MVP评选和年度Top评定。

## 选手赛季统计

```rust
pub struct PlayerSeasonStatistics {
    pub id: Option<i64>,
    pub save_id: String,
    pub player_id: i64,
    pub player_name: String,
    pub season_id: i64,
    pub team_id: Option<i64>,
    pub position: String,

    // 比赛统计
    pub matches_played: i32,
    pub games_played: i32,

    // 影响力统计
    pub total_impact: f64,
    pub avg_impact: f64,

    // 发挥统计
    pub avg_performance: f64,
    pub best_performance: f64,
    pub worst_performance: f64,

    // 稳定性评分
    pub consistency_score: f64,

    // 冠军加成
    pub international_titles: i32,
    pub regional_titles: i32,
    pub champion_bonus: f64,

    // 年度Top得分
    pub yearly_top_score: f64,
}
```

## 影响力计算

### 单局影响力

```
单局影响力 = 选手实际发挥 - 队伍平均战力
```

- **正值**: 超常发挥，带动队伍
- **负值**: 表现低迷，拖累队伍
- **> 5**: 极度超神表现
- **< -5**: 严重拉胯表现

### 年度平均影响力

```
年度平均影响力 = 累计影响力 / 参与局数
```

### 年度Top得分

```
yearly_top_score = 平均影响力 × 0.7 + 冠军分 × 0.3
```

## 稳定性评分

基于选手发挥的标准差计算：

```
consistency_score = 100 - (performance_std_dev × 2)
```

评分范围 0-100，越高越稳定。

## 数据中心功能

1. **选手排行榜**
   - 按影响力排序
   - 按稳定性排序
   - 按冠军分排序

2. **位置筛选**
   - TOP / JUG / MID / ADC / SUP

3. **赛季筛选**
   - 查看不同赛季数据

4. **选手详情**
   - 个人表现趋势
   - 荣誉记录

## 数据记录流程

```
比赛模拟 → DetailedMatchResult → recordPerformance() → PlayerSeasonStats
                                                            ↓
                                                    影响力排名 / MVP 评选
```

## API 接口

| 接口 | 描述 |
|------|------|
| `record_player_performance(params)` | 记录选手表现 |
| `batch_record_player_performance(performances)` | 批量记录表现 |
| `get_player_season_stats(player_id, season_id)` | 获取选手赛季统计 |
| `get_player_rankings(season_id, position)` | 获取选手排行 |
| `sync_player_stats()` | 同步选手统计数据 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/models/player_stats.rs` | 统计数据模型 |
| `src-tauri/src/db/repository.rs` | 数据仓库（含统计数据操作） |
| `src-tauri/src/commands/stats_commands.rs` | 统计命令接口 |
