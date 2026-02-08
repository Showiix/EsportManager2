# 荣誉系统

## 概述

荣誉系统记录战队和选手获得的各类荣誉，在赛事结束时自动颁发。荣誉影响选手身价和球队声望。

## 荣誉类型

### 战队荣誉

| 类型 | 说明 |
|------|------|
| TeamChampion | 赛事冠军 |
| TeamRunnerUp | 赛事亚军 |
| TeamThird | 赛事季军 |
| TeamFourth | 赛事殿军 |
| RegularSeasonFirst | 常规赛第一 |

### 选手荣誉

| 类型 | 说明 |
|------|------|
| PlayerChampion | 冠军队成员 |
| PlayerRunnerUp | 亚军队成员 |
| PlayerThird | 季军队成员 |
| PlayerFourth | 殿军队成员 |
| TournamentMvp | 赛事MVP |
| FinalsMvp | 决赛MVP |
| RegularSeasonMvp | 常规赛MVP |
| PlayoffsFmvp | 季后赛FMVP |

### 年度荣誉

| 类型 | 说明 |
|------|------|
| AnnualMvp | 年度MVP（年度IM第一） |
| AnnualTop20 | 年度Top20 |
| AnnualAllPro1st | 年度最佳阵容一阵 |
| AnnualAllPro2nd | 年度最佳阵容二阵 |
| AnnualAllPro3rd | 年度最佳阵容三阵 |
| AnnualMostConsistent | 年度最稳定选手 |
| AnnualMostDominant | 年度最具统治力选手 |
| AnnualRookie | 年度最佳新秀 |

## 荣誉数据结构

```rust
pub struct Honor {
    pub id: u64,
    pub save_id: String,
    pub honor_type: HonorType,
    pub season_id: u64,
    pub tournament_id: u64,
    pub tournament_name: String,
    pub team_id: Option<u64>,      // 战队荣誉时有值
    pub team_name: Option<String>,
    pub player_id: Option<u64>,    // 选手荣誉时有值
    pub player_name: Option<String>,
}
```

## 荣誉颁发时机

| 阶段 | 颁发的荣誉 |
|------|-----------|
| 常规赛结束 | 常规赛第一、常规赛MVP |
| 季后赛结束 | 冠亚季殿军、选手冠军、季后赛MVP |
| 国际赛结束 | 冠亚季殿军、选手冠军、赛事MVP |
| 年度颁奖典礼 | 年度MVP、年度Top20、最佳阵容、最稳定、最具统治力、最佳新秀 |

## 荣誉对身价影响

| 荣誉类型 | 身价加成 | 持续时间 |
|---------|---------|---------|
| 世界赛冠军 | +80% | 永久 |
| 世界赛亚军 | +40% | 永久 |
| 世界赛四强 | +20% | 永久 |
| MSI冠军 | +50% | 永久 |
| MSI亚军 | +25% | 永久 |
| 联赛冠军 | +30% | 3赛季 |
| 联赛MVP | +20% | 2赛季 |

### 年度颁奖身价加成（取最高，不叠加）

同一选手获得多个年度奖项时，只取最高加成，不累加：

| 奖项 | 身价加成 |
|------|---------|
| 年度MVP | +50% |
| Top 2-5 | +30% |
| Top 6-10 | +20% |
| Top 11-20 | +10% |
| 最佳阵容一阵 | +25% |
| 最佳阵容二阵 | +15% |
| 最佳阵容三阵 | +10% |
| 最具统治力 | +20% |
| 最稳定选手 | +15% |
| 最佳新秀 | +30% |

实现方式：使用 `HashMap<player_id, (max_bonus, reason)>` 跟踪每位选手的最高加成。

## 年度颁奖典礼系统

### 奖项体系（24 个个人奖项）

| 档次 | 奖项 | 人数 |
|------|------|------|
| 至高荣誉 | 年度MVP（Top20 #1） | 1 |
| 年度Top20 | 1-20 排名 | 20 |
| 最佳阵容一阵 | 各位置 #1 | 5 |
| 最佳阵容二阵 | 各位置 #2 | 5 |
| 最佳阵容三阵 | 各位置 #3 | 5 |
| 特别奖 | 最稳定选手（consistency_score 最高，≥30 场） | 1 |
| 特别奖 | 最佳新秀（yearly_top_score，age ≤ 20，≥10 场） | 1 |
| 特别奖 | 最具统治力选手（dominance_score 最高，≥20 场） | 1 |

### 颁奖数据结构

```typescript
interface AnnualAwardsData {
  season_id: number
  top20: Top20Player[]        // 含五维分数 + 评语
  all_pro_1st: AllProPlayer[] // 一阵（5人）
  all_pro_2nd: AllProPlayer[] // 二阵（5人）
  all_pro_3rd: AllProPlayer[] // 三阵（5人）
  most_consistent: SpecialAwardPlayer | null
  most_dominant: SpecialAwardPlayer | null
  rookie_of_the_year: RookiePlayer | null
  already_awarded: boolean
}

interface ScoreDimensions {
  impact_norm: number       // 影响力归一化 0-100
  performance_norm: number  // 发挥归一化 0-100
  stability_norm: number    // 稳定性归一化 0-100
  appearance_norm: number   // 出场归一化 0-100
  honor_norm: number        // 荣誉归一化 0-100
}

interface PlayerCommentary {
  description: string       // 评语描述
  tags: string[]            // 标签（如"稳定核心"、"冠军基因"）
}
```

### 评语生成系统

基于选手数据自动生成评语和标签，15+ 条件分支：

| 条件 | 评语 | 标签 |
|------|------|------|
| 高影响力 + 高稳定 | "全年表现如磐石般稳定" | 稳定核心 |
| 高影响力 + 低稳定 | "巅峰时刻无人可挡，低谷时也让人揪心" | 大心脏 |
| 冠军加成 ≥ 6 | "冠军荣耀加身" | 冠军基因 |
| age ≤ 20 + Top10 | "年仅X岁便跻身年度TopN" | 年少成名 |
| age ≥ 30 | "老将弥坚" | 老将风范 |
| games ≥ 100 | - | 铁人 |
| consistency > 85 | - | 稳如泰山 |
| best_performance > 90 | - | 超级carry |

### 前端页面

#### 年度评选数据分析页（AnnualTop.vue）

- **数据源**: `awardsApi.getAnnualAwardsData()`
- **Top3 领奖台**: 五维条形图 + 年度得分
- **雷达图对比**: ECharts RadarChart，Top3 选手五维叠加显示
- **Top20 完整排行**: 每列带迷你进度条（影响力/发挥/稳定性）+ 评语标签
- **最佳阵容三阵**: 金/银/铜配色，按位置展示
- **特别奖项**: 最稳定/最具统治力/最佳新秀
- **分布统计**: 位置分布柱状图 + 战队分布条形图

#### 年度颁奖典礼页（AnnualAwards.vue）

三种状态：
1. 非颁奖阶段 + 无数据 → 空状态
2. 有历史数据 → 静态查看模式（赛季选择器）
3. 颁奖阶段 → 实时典礼模式

典礼流程（4 阶段）：
```
[开始颁奖典礼] → 后端颁奖（completeAndAdvance）
    ↓
阶段1: Top20 逐个揭晓（#20→#1）
    ↓
阶段2: 最佳阵容（三阵→二阵→一阵）
    ↓
阶段3: 特别奖项（最稳定→最佳新秀→最具统治力）
    ↓
阶段4: MVP 终极揭晓 + 评语展示
```

## 荣誉殿堂

```rust
pub struct HonorHallData {
    pub champions: Vec<Honor>,  // 所有冠军
    pub mvps: Vec<Honor>,       // 所有MVP
    pub champions_by_type: HashMap<String, Vec<Honor>>, // 按赛事类型分类
}
```

## MVP 评选

MVP 基于选手表现数据评选：

```
mvp_score = 平均影响力 × 0.4 + 胜率贡献 × 0.3 + 关键表现 × 0.3
```

### 年度 Top20 评分（五维归一化加权）

```
yearly_top_score = 影响力(35%) + 发挥(20%) + 冠军(20%) + 稳定性(15%) + 出场(10%)
```

各维度归一化到 0-100：
- 影响力: `(avg_impact + 10) × 3.33`
- 发挥: `(avg_performance - 50) × 2`
- 冠军: `champion_bonus × 6.67`（国际赛冠军+3, 赛区冠军+1）
- 稳定性: `consistency_score`（已是 0-100）
- 出场: `games_played × 0.83`

### 统治力评分

```
dominance_score = 巅峰表现(35%) + 影响力(45%) + 发挥(20%)
```

各维度归一化到 0-100：
- 巅峰表现: `(best_performance - 60) × 2.5`
- 影响力: `(avg_impact + 5) × 5`
- 发挥: `(avg_performance - 50) × 2`

## API 接口

| 接口 | 描述 |
|------|------|
| `get_honor_hall(save_id)` | 获取荣誉殿堂数据 |
| `get_team_honors(team_id)` | 获取战队荣誉 |
| `get_player_honors(player_id)` | 获取选手荣誉 |
| `get_player_honor_rankings()` | 获取选手荣誉排行 |
| `get_team_honor_rankings()` | 获取战队荣誉排行 |
| `get_annual_awards_data(season_id)` | 获取年度颁奖数据（Top20 + 三阵 + 特别奖） |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/models/honor.rs` | 荣誉数据模型（HonorType 枚举） |
| `src-tauri/src/engines/honor.rs` | 荣誉引擎 |
| `src-tauri/src/services/honor_service.rs` | 荣誉服务 |
| `src-tauri/src/commands/honor_commands.rs` | 荣誉命令接口 |
| `src-tauri/src/commands/awards_commands.rs` | 年度颁奖命令（数据结构 + 查询 + 评语生成） |
| `src-tauri/src/services/game_flow.rs` | 游戏流程（颁奖逻辑 + 身价加成） |
| `src/views/AnnualTop.vue` | 年度评选数据分析页 |
| `src/views/AnnualAwards.vue` | 年度颁奖典礼页 |
