# 年度积分系统

## 概述

年度积分系统累计各赛事积分，用于决定 Super 洲际赛参赛资格（年度 Top16）。

## 积分配置

### 联赛季后赛

| 名次 | 积分 |
|------|------|
| 冠军 | 12 |
| 亚军 | 10 |
| 季军 | 8 |
| 殿军 | 6 |
| 5-8名 | 3 |

### 国际赛事

| 名次 | MSI | 世界赛 | 马德里/Claude | 上海 |
|------|-----|-------|--------------|------|
| 冠军 | 20 | 20 | 20 | 20 |
| 亚军 | 16 | 16 | 16 | 16 |
| 季军 | 12 | 12 | 12 | 12 |
| 殿军 | 8 | 8 | 8 | 8 |
| 第5-6 | 6 | 6 | 6 | 6 |
| 第7-8 | 4 | 4 | 4 | 4 |
| 其他 | - | - | 2 | - |

### ICP 洲际对抗赛

按赛区排名分配:

| 赛区排名 | 参赛队积分 | 未参赛队积分 |
|---------|-----------|-------------|
| 第1名 | 12 | 6 |
| 第2名 | 8 | 4 |
| 第3名 | 6 | 3 |
| 第4名 | 4 | 2 |

### Super 洲际赛

| 名次 | 积分 |
|------|------|
| 冠军 | 35 |
| 亚军 | 30 |
| 季军 | 25 |
| 殿军 | 20 |
| 第三阶段淘汰 | 8 |
| 第二阶段淘汰 | 5 |
| 第一阶段淘汰 | 2 |

## 数据结构

### 年度积分明细

```rust
pub struct AnnualPointsDetail {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    pub tournament_id: u64,
    pub tournament_name: Option<String>,
    pub tournament_type: Option<String>,
    pub points: u32,              // 获得积分
    pub final_rank: Option<u32>,  // 最终排名
}
```

### 全球排名

```rust
pub struct GlobalRanking {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    pub total_points: u32,        // 总积分
    pub rank: Option<u32>,        // 排名
}
```

## 积分流程

```
赛事结束 → complete_phase() → award_tournament_points() → 更新 team.annual_points
                                                              ↓
                                                    保存到 annual_points_detail 表
```

## 赛季结算

赛季结束时，年度积分清零：

```rust
// 赛季结算
team.annual_points = 0
```

## API 接口

| 接口 | 描述 |
|------|------|
| `get_annual_points(save_id, season_id)` | 获取年度积分 |
| `get_global_rankings(save_id, season_id)` | 获取全球排名 |
| `get_team_points_detail(team_id, season_id)` | 获取战队积分明细 |
| `calculate_event_points(...)` | 计算赛事积分 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/engines/points_calculation.rs` | 积分计算引擎 |
| `src-tauri/src/commands/points_commands.rs` | 积分命令接口 |
