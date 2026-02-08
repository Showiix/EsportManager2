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

### 主页仪表盘（DataCenter.vue）

1. **概览统计卡片**（4 列）
   - 参赛选手数、平均影响力、最高得分、平均场次
   - 数据来源：从 rankings 数组聚合计算，无需新 API

2. **位置对比分析图**（ECharts 分组柱状图）
   - X 轴：上单 / 打野 / 中单 / 下路 / 辅助
   - 系列：平均影响力、平均得分
   - 布局：与 Top5 侧边栏左右并列（3:2）

3. **TOP 5 选手侧边栏**
   - 取 filteredRankings 前 5，带排名徽章
   - 点击跳转选手详情页

4. **选手排行榜**（已有）
   - 按影响力 / 得分 / 场次等排序
   - 位置筛选：TOP / JUG / MID / ADC / SUP
   - 赛季筛选、搜索、分页

### 选手详情页（DataCenterPlayerDetail.vue）

1. **选手信息卡片** — 姓名、位置、战队、排名
2. **数据统计卡片**（4 列）— 参与局数、平均影响力、稳定性、得分
3. **能力雷达图**（ECharts RadarChart）— 5 维度归一化到 0-100
   - 影响力：`(avgImpact + 10) × 3.33`（原始 -10~+20 → 0~100）
   - 稳定性：`consistencyScore`（已是 0-100）
   - 出场：`gamesPlayed / 2`（上限 200 → 100）
   - 冠军：`championBonus × 6.67`（上限 15 → 100）
   - 发挥：`(avgPerformance - 50) × 2`（原始 50-100 → 0~100）
   - 布局：与选手状态卡片左右并列（1:1）
4. **选手状态卡片** — 满意度进度条、忠诚度进度条、离队警告
5. **影响力走势图**（折线图）— 每局影响力趋势 + 平均线
6. **身价走势图**（折线图 + 面积填充）
   - 数据来源：`statsApi.getPlayerMarketValueChanges(playerId)`
   - X 轴：赛季（S1, S2...），Y 轴：身价（万元）
   - 无数据时显示空状态提示
7. **影响力分布图**（柱状图）— 按区间分组统计
8. **稳定性仪表盘**（Gauge）— 0-100 评分
9. **表现统计**（4 格）— 正向/负向/亮眼表现、正向率

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
| `src/views/DataCenter.vue` | 数据中心主页（仪表盘 + 排行榜） |
| `src/views/DataCenterPlayerDetail.vue` | 选手详情页（雷达图 + 走势图） |
| `src-tauri/src/models/player_stats.rs` | 统计数据模型 |
| `src-tauri/src/db/repository.rs` | 数据仓库（含统计数据操作） |
| `src-tauri/src/commands/stats_commands.rs` | 统计命令接口 |
