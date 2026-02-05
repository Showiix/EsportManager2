---
name: honor-system
description: 电竞经理游戏的荣誉记录系统。管理战队和选手的冠军、MVP等荣誉记录、荣誉殿堂数据聚合。当需要修改荣誉类型、MVP计算逻辑、荣誉展示时使用此技能。
---

# 荣誉记录系统 (Honor System)

## Overview

荣誉记录系统负责记录和管理所有赛事中产生的荣誉，包括战队冠亚季殿军、选手冠军成员、赛事MVP、决赛MVP等。系统提供荣誉殿堂数据聚合和统计功能。

## 核心组件

### HonorEngine

**文件**: `src-tauri/src/engines/honor.rs`

荣誉引擎，统一管理所有荣誉的创建、计算和聚合。

```rust
pub struct HonorEngine;
```

**核心方法**:

#### 荣誉创建
- `create_team_champion(...)` - 记录战队冠军
- `create_team_runner_up(...)` - 记录战队亚军
- `create_team_third(...)` - 记录战队季军
- `create_team_fourth(...)` - 记录战队殿军
- `create_player_champion(...)` - 记录选手冠军（冠军队成员）
- `create_tournament_mvp(...)` - 记录赛事MVP
- `create_finals_mvp(...)` - 记录决赛MVP
- `create_regular_season_mvp(...)` - 记录常规赛MVP
- `create_playoffs_fmvp(...)` - 记录季后赛FMVP

#### MVP计算
- `calculate_tournament_mvp(performances)` - 计算赛事MVP（累计影响力最高）
- `calculate_finals_mvp(performances)` - 计算决赛MVP（胜方影响力最高）

#### 数据聚合
- `process_tournament_honors(...)` - 处理赛事结束时的所有荣誉
- `build_honor_hall(all_honors)` - 构建荣誉殿堂数据
- `count_team_champions(honors, team_id)` - 统计战队冠军数
- `count_player_champions(honors, player_id)` - 统计选手冠军数
- `count_player_mvps(honors, player_id)` - 统计选手MVP数

## 数据结构

### HonorType (荣誉类型)

**文件**: `src-tauri/src/models/honor.rs`

```rust
pub enum HonorType {
    // 战队荣誉
    TeamChampion,       // 战队冠军
    TeamRunnerUp,       // 战队亚军
    TeamThird,          // 战队季军
    TeamFourth,         // 战队殿军

    // 选手荣誉
    PlayerChampion,     // 选手冠军（冠军队成员）
    PlayerRunnerUp,     // 选手亚军
    PlayerThird,        // 选手季军
    PlayerFourth,       // 选手殿军

    // MVP荣誉
    TournamentMvp,      // 赛事MVP
    FinalsMvp,          // 决赛MVP
    RegularSeasonMvp,   // 常规赛MVP
    PlayoffsFmvp,       // 季后赛FMVP
}
```

### Honor (荣誉记录)

```rust
pub struct Honor {
    pub id: u64,
    pub save_id: String,
    pub honor_type: HonorType,
    pub season_id: u64,
    pub tournament_id: Option<u64>,
    pub tournament_name: String,
    pub tournament_type: String,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub player_id: Option<u64>,
    pub player_name: Option<String>,
    pub position: Option<String>,
    pub stats: Option<HonorStats>,  // MVP相关统计
    pub created_at: String,
}
```

### HonorStats (荣誉统计)

```rust
pub struct HonorStats {
    pub total_impact: f64,      // 总影响力
    pub mvp_count: u32,         // 单场MVP次数
    pub games_played: u32,      // 参与局数
    pub wins: u32,              // 胜场
    pub avg_performance: f64,   // 平均表现
}
```

### TournamentHonors (赛事荣誉集合)

```rust
pub struct TournamentHonors {
    pub team_champion: Option<Honor>,
    pub team_runner_up: Option<Honor>,
    pub team_third: Option<Honor>,
    pub team_fourth: Option<Honor>,
    pub player_champions: Vec<Honor>,  // 冠军队5名选手
    pub tournament_mvp: Option<Honor>,
    pub finals_mvp: Option<Honor>,
}
```

### HonorHallData (荣誉殿堂)

```rust
pub struct HonorHallData {
    pub champions: Vec<Honor>,                          // 所有冠军
    pub champions_by_type: HashMap<String, Vec<Honor>>, // 按赛事类型分组
    pub mvps: Vec<Honor>,                               // 所有MVP
}
```

## Tauri 命令接口

**文件**: `src-tauri/src/commands/honor_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `get_honor_hall` | 获取荣誉殿堂数据 | `HonorHallData` |
| `get_team_honors` | 获取战队所有荣誉 | `Vec<Honor>` |
| `get_player_honors` | 获取选手所有荣誉 | `Vec<Honor>` |
| `get_season_honors` | 获取赛季所有荣誉 | `Vec<Honor>` |
| `get_tournament_honors` | 获取赛事所有荣誉 | `Vec<Honor>` |
| `get_team_champion_count` | 获取战队冠军数 | `u32` |
| `get_player_champion_count` | 获取选手冠军数 | `u32` |
| `get_player_mvp_count` | 获取选手MVP数 | `u32` |
| `get_team_honor_stats` | 获取战队荣誉统计 | `TeamHonorStats` |
| `get_player_honor_stats` | 获取选手荣誉统计 | `PlayerHonorStats` |
| `get_champions_by_type` | 按赛事类型获取冠军 | `Vec<Honor>` |
| `get_all_champions` | 获取所有冠军记录 | `Vec<Honor>` |
| `get_all_mvps` | 获取所有MVP记录 | `Vec<Honor>` |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 获取荣誉殿堂
export async function getHonorHall(): Promise<HonorHallData>

// 获取战队荣誉
export async function getTeamHonors(teamId: number): Promise<Honor[]>

// 获取选手荣誉
export async function getPlayerHonors(playerId: number): Promise<Honor[]>

// 获取战队冠军数
export async function getTeamChampionCount(teamId: number): Promise<number>

// 获取选手MVP数
export async function getPlayerMvpCount(playerId: number): Promise<number>
```

## MVP 计算规则

### 赛事MVP (TournamentMvp)
- 统计整个赛事中所有选手的表现
- 选取**累计影响力最高**的选手
- 不限制胜负方

```rust
pub fn calculate_tournament_mvp(
    &self,
    player_performances: &[(player_id, name, team_id, team_name, position, impact, is_winner, is_mvp)]
) -> Option<PlayerTournamentStats>
```

### 决赛MVP (FinalsMvp)
- 只统计决赛场次
- 只考虑**胜方队伍**的选手
- 选取胜方累计影响力最高的选手

```rust
pub fn calculate_finals_mvp(
    &self,
    finals_performances: &[(player_id, name, team_id, team_name, position, impact, is_winner)]
) -> Option<(player_id, name, team_id, team_name, position, HonorStats)>
```

## 系统集成

### 与时间推进系统
- 赛事完成时调用 `process_tournament_honors` 生成所有荣誉
- 赛季结算时汇总年度荣誉

### 与数据中心系统
- MVP计算依赖 `PlayerTournamentStats` 数据
- `total_impact`、`avg_performance` 等统计来自数据中心

### 与选手系统
- 荣誉影响选手声望和市场价值
- 冠军数/MVP数是选手履历的重要组成

### 与财政系统
- 荣誉可能触发额外奖金（如冠军奖金）

## 使用示例

### 处理赛事结束荣誉
```rust
let engine = HonorEngine::new();

let honors = engine.process_tournament_honors(
    &save_id,
    season_id,
    tournament_id,
    "S1 世界赛",
    "WorldChampionship",
    (t1_id, "T1"),           // 冠军
    (gen_id, "GEN"),         // 亚军
    Some((jdg_id, "JDG")),   // 季军
    Some((blg_id, "BLG")),   // 殿军
    &champion_players,        // 冠军队选手
    Some(&mvp_stats),         // 赛事MVP
    Some(finals_mvp_info),    // 决赛MVP
);

// 保存到数据库
HonorRepository::save_batch(&pool, &honors.to_vec()).await?;
```

### 查询荣誉殿堂
```typescript
const hall = await getHonorHall()

// 显示世界赛冠军
const worldsChampions = hall.champions_by_type['WorldChampionship']
worldsChampions.forEach(h => {
    console.log(`S${h.season_id} 世界冠军: ${h.team_name}`)
})

// 显示MVP排行
console.log(`MVP总数: ${hall.mvps.length}`)
```

### 统计选手荣誉
```typescript
const championCount = await getPlayerChampionCount(playerId)
const mvpCount = await getPlayerMvpCount(playerId)
console.log(`冠军: ${championCount}x | MVP: ${mvpCount}x`)
```

## 注意事项

1. **荣誉去重**: 同一赛事不应重复记录相同荣誉，需检查是否已存在
2. **选手冠军**: 只记录比赛时的首发/出场选手，替补不自动获得
3. **决赛MVP**: 必须来自胜方队伍
4. **荣誉持久化**: 赛事结束后立即保存，避免数据丢失
5. **荣誉殿堂缓存**: 可考虑缓存 `HonorHallData` 提升查询性能
