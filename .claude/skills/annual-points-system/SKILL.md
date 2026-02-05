---
name: annual-points-system
description: 电竞经理游戏的年度积分系统。管理各队伍通过赛事获得的年度积分、排名计算、Super洲际赛资格判定。当需要修改积分规则、排名逻辑时使用此技能。
---

# 年度积分系统 (Annual Points System)

## Overview

年度积分系统负责计算和管理各队伍在一个赛季内通过参加各项赛事所获得的年度积分。积分用于决定 Super 洲际年度邀请赛的参赛资格（年度积分 Top16）。

## 核心组件

### PointsCalculationEngine

**文件**: `src-tauri/src/engines/points_calculation.rs`

积分计算引擎，存储积分配置并提供计算方法。

```rust
pub struct PointsCalculationEngine {
    config: HashMap<(TournamentType, String), u32>,  // (赛事类型, 名次) -> 积分
}
```

**核心方法**:
- `get_points(tournament_type, position)` - 获取指定赛事和名次的积分
- `calculate_event_points(...)` - 计算单场赛事积分
- `calculate_annual_rankings(...)` - 计算年度积分排名
- `get_global_top16(rankings)` - 获取 Top16 队伍
- `assign_super_cup_groups(top16)` - 分配 Super 赛分组

## 数据结构

### AnnualPointsDetail (积分明细)

**文件**: `src-tauri/src/models/mod.rs`

```rust
pub struct AnnualPointsDetail {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    pub tournament_id: u64,
    pub tournament_name: Option<String>,
    pub tournament_type: Option<String>,
    pub points: u32,
    pub final_rank: Option<u32>,
}
```

### TeamAnnualPoints (队伍年度积分)

```rust
pub struct TeamAnnualPoints {
    pub team_id: u64,
    pub team_name: String,
    pub region_code: String,
    pub total_points: u32,
    pub rank: u32,
}
```

### GlobalRanking (全球排名)

```rust
pub struct GlobalRanking {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub team_id: u64,
    pub global_rank: u32,
    pub total_points: u32,
}
```

## 积分配置表

### 联赛季后赛 (SpringPlayoffs / SummerPlayoffs)

| 名次 | 积分 |
|------|------|
| CHAMPION | 12 |
| RUNNER_UP | 10 |
| THIRD | 8 |
| FOURTH | 6 |
| 5TH_8TH | 3 |

### MSI 季中赛

| 名次 | 积分 |
|------|------|
| CHAMPION | 20 |
| RUNNER_UP | 16 |
| THIRD | 12 |
| FOURTH | 8 |
| LOSERS_R2 | 6 |
| LOSERS_R1 | 4 |

### 马德里大师赛 / Claude洲际赛 / 上海大师赛

| 名次 | 积分 |
|------|------|
| CHAMPION | 20 |
| RUNNER_UP | 16 |
| THIRD | 12 |
| FOURTH | 8 |
| SEMI_LOSER / LOSERS_R2 | 6 |
| QUARTER_LOSER / LOSERS_R1 | 4 |
| R1_LOSER | 2 |

### 世界赛

| 名次 | 积分 |
|------|------|
| CHAMPION | 20 |
| RUNNER_UP | 16 |
| THIRD | 12 |
| FOURTH | 8 |
| QUARTER_FINAL | 6 |
| GROUP_STAGE | 4 |

### ICP 四赛区洲际对抗赛

| 赛区排名 | 参赛队伍 | 未参赛队伍 |
|----------|----------|------------|
| 第一赛区 | 12 | 6 |
| 第二赛区 | 8 | 4 |
| 第三赛区 | 6 | 3 |
| 第四赛区 | 4 | 2 |

### Super 洲际年度邀请赛

**不颁发积分！** Super 赛是年度积分的终点奖励。

## Tauri 命令接口

**文件**: `src-tauri/src/commands/points_commands.rs`

| 命令 | 功能 | 返回类型 |
|------|------|----------|
| `get_annual_points_ranking` | 获取年度积分排名 | `Vec<TeamAnnualPoints>` |
| `get_team_points_detail` | 获取队伍积分明细 | `Vec<AnnualPointsDetail>` |
| `get_tournament_points` | 获取赛事积分发放记录 | `Vec<AnnualPointsDetail>` |
| `get_super_qualified_teams` | 获取 Super 资格队伍 (Top16) | `Vec<TeamAnnualPoints>` |

## 前端 API

**文件**: `src/api/tauri.ts`

```typescript
// 获取年度积分排名
export async function getAnnualPointsRanking(): Promise<TeamAnnualPoints[]>

// 获取队伍积分明细
export async function getTeamPointsDetail(teamId: number): Promise<AnnualPointsDetail[]>

// 获取赛事积分记录
export async function getTournamentPoints(tournamentId: number): Promise<AnnualPointsDetail[]>

// 获取 Super 赛资格队伍
export async function getSuperQualifiedTeams(): Promise<TeamAnnualPoints[]>
```

## 数据库表

### annual_points_detail

```sql
CREATE TABLE annual_points_detail (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    points INTEGER NOT NULL,
    final_rank INTEGER,
    FOREIGN KEY (team_id) REFERENCES teams(id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id)
);
```

### team_annual_points (视图或聚合表)

用于快速查询队伍年度总积分和排名。

## 系统集成

### 与时间推进系统
- 赛事完成后自动计算并保存积分
- 赛季结算时最终确定年度排名

### 与荣誉系统
- 积分可作为年度最佳队伍的参考指标

### 与国际赛事系统
- `get_super_qualified_teams` 用于确定 Super 赛参赛名单
- ICP 赛区排名影响积分分配

## Super 赛分组规则

根据年度积分 Top16 分组:

| 组别 | 排名范围 | 队伍数 |
|------|----------|--------|
| 传奇组 (Legendary) | 1-4 | 4 |
| 挑战者组 (Challenger) | 5-8 | 4 |
| Fighter 组 | 9-16 | 8 |

```rust
pub fn assign_super_cup_groups(&self, top16: &[&GlobalRanking])
    -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    // 返回 (legendary, challenger, fighter) 的队伍 ID 列表
}
```

## 使用示例

### 获取年度积分排名
```typescript
const rankings = await getAnnualPointsRanking()
rankings.forEach((team, idx) => {
    console.log(`#${idx + 1} ${team.team_name}: ${team.total_points}分`)
})
```

### 查看队伍积分来源
```typescript
const details = await getTeamPointsDetail(teamId)
details.forEach(d => {
    console.log(`${d.tournament_name}: ${d.points}分 (第${d.final_rank}名)`)
})
```

### 获取 Super 赛名单
```typescript
const qualified = await getSuperQualifiedTeams()
console.log(`Super赛参赛队伍: ${qualified.map(t => t.team_name).join(', ')}`)
```

## 注意事项

1. **Super 赛不计积分**: Super 赛是年度积分的奖励，不产生新积分
2. **ICP 特殊规则**: 积分根据赛区排名而非个人名次分配
3. **积分持久化**: 每场赛事结束后需调用 `PointsRepository.save_points_details` 保存
4. **排名更新时机**: 建议在每场赛事完成后重新计算排名
