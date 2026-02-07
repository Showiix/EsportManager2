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
| TournamentMvp | 赛事MVP |
| FinalsMvp | 决赛MVP |
| RegularSeasonMvp | 常规赛MVP |
| PlayoffsFmvp | 季后赛FMVP |

### 年度荣誉

| 类型 | 说明 |
|------|------|
| AnnualMvp | 年度MVP |
| AnnualTop20 | 年度Top20 |
| AnnualBestTop | 年度最佳上单 |
| AnnualBestJungle | 年度最佳打野 |
| AnnualBestMid | 年度最佳中单 |
| AnnualBestAdc | 年度最佳ADC |
| AnnualBestSupport | 年度最佳辅助 |
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
| 年度颁奖典礼 | 年度MVP、年度Top20、最佳位置 |

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

### 年度 Top20 评分

```
yearly_top_score = 平均影响力 × 0.7 + 冠军分 × 0.3
```

冠军分计算:
- 世界赛冠军: +15
- MSI冠军: +10
- 联赛冠军: +8

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
