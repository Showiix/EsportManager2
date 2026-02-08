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
| 最佳阵容一队 | +25% | 2赛季 |
| 年度最佳新秀 | +20% | 3赛季 |

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

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/models/honor.rs` | 荣誉数据模型 |
| `src-tauri/src/engines/honor.rs` | 荣誉引擎 |
| `src-tauri/src/services/honor_service.rs` | 荣誉服务 |
| `src-tauri/src/commands/honor_commands.rs` | 荣誉命令接口 |
