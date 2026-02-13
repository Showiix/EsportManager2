# 数据库设计

## 概述

EsportManager 2 使用 **SQLite** 作为本地数据库，通过 **sqlx** 进行异步数据库操作。数据库采用存档隔离设计，每个存档的数据通过 `save_id` 字段关联。

## ER 关系图

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  saves   │────<│ regions  │────<│  teams   │
└──────────┘     └──────────┘     └────┬─────┘
                                       │
                 ┌─────────────────────┼─────────────────────┐
                 │                     │                     │
           ┌─────┴─────┐         ┌─────┴─────┐         ┌─────┴─────┐
           │  players  │         │tournaments│         │team_finance│
           └─────┬─────┘         └─────┬─────┘         └───────────┘
                 │                     │
         ┌───────┴───────┐       ┌─────┴─────┐
         │               │       │           │
   ┌─────┴─────┐   ┌─────┴─────┐ │  matches  │
   │player_stats│   │  honors  │ └─────┬─────┘
   └───────────┘   └───────────┘       │
                                 ┌─────┴─────┐
                                 │match_games│
                                 └───────────┘
```

## 核心表结构

### 1. 存档管理

#### saves (存档表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | CHAR(36) | 存档 UUID，主键 |
| name | VARCHAR(100) | 存档名称 |
| current_season | INT | 当前赛季 (S1, S2...) |
| current_phase | ENUM | 当前阶段 |
| phase_completed | BOOLEAN | 当前阶段是否完成 |
| created_at | TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | 更新时间 |

### 2. 基础数据

#### regions (赛区表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| code | VARCHAR(10) | 赛区代码 (LPL/LCK/LEC/LCS) |
| name | VARCHAR(50) | 赛区名称 |
| full_name | VARCHAR(100) | 赛区全称 |

#### teams (战队表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| region_id | BIGINT | 所属赛区 |
| name | VARCHAR(100) | 战队名称 |
| short_name | VARCHAR(20) | 战队简称 |
| power_rating | DECIMAL(5,2) | 战力值 (0-100) |
| total_matches | INT | 总场次 |
| wins | INT | 胜场 |
| win_rate | DECIMAL(5,2) | 胜率 |
| annual_points | INT | 年度积分 |
| cross_year_points | INT | 跨年度积分 |
| balance | BIGINT | 账户余额(元) |

### 3. 选手系统

#### players (选手表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| team_id | BIGINT | 所属战队 (NULL=自由球员) |
| game_id | VARCHAR(50) | 游戏 ID |
| real_name | VARCHAR(100) | 真实姓名 |
| nationality | VARCHAR(50) | 国籍 |
| age | TINYINT | 年龄 (16-36) |
| ability | TINYINT | 能力值 (0-100) |
| potential | TINYINT | 潜力值上限 (0-100) |
| stability | TINYINT | 稳定性 (0-100) |
| tag | ENUM | 标签 (ORDINARY/NORMAL/GENIUS) |
| status | ENUM | 状态 (ACTIVE/RETIRED) |
| position | ENUM | 位置 (TOP/JUG/MID/ADC/SUP) |
| salary | BIGINT | 年薪(元) |
| market_value | BIGINT | 身价(元) |
| contract_end_season | INT | 合同到期赛季 |
| is_starter | BOOLEAN | 是否首发 |

#### player_season_history (选手赛季历史表)

记录每赛季的属性快照，用于历史数据追溯。

### 4. 赛事系统

#### seasons (赛季表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| season_number | INT | 赛季编号 |
| status | ENUM | 状态 (UPCOMING/IN_PROGRESS/COMPLETED) |
| started_at | TIMESTAMP | 开始时间 |
| ended_at | TIMESTAMP | 结束时间 |

#### tournaments (赛事表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| season_id | BIGINT | 所属赛季 |
| type | ENUM | 赛事类型 |
| name | VARCHAR(100) | 赛事名称 |
| region_id | BIGINT | 所属赛区 (国际赛为 NULL) |
| status | ENUM | 状态 |
| current_stage | VARCHAR(50) | 当前阶段 |
| current_round | INT | 当前轮次 |

#### tournament_participants (赛事参赛队伍表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| tournament_id | BIGINT | 赛事 ID |
| team_id | BIGINT | 战队 ID |
| seed | INT | 种子排名 |
| group_name | VARCHAR(10) | 分组 |
| final_rank | INT | 最终排名 |
| points_earned | INT | 获得积分 |

### 5. 比赛记录

#### matches (比赛表)

一场 BO 系列赛为一条记录。

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| tournament_id | BIGINT | 所属赛事 |
| stage | VARCHAR(50) | 阶段 |
| round | INT | 轮次 |
| format | ENUM | 赛制 (BO1/BO3/BO5) |
| home_team_id | BIGINT | 主队 |
| away_team_id | BIGINT | 客队 |
| home_score | TINYINT | 主队得分 |
| away_score | TINYINT | 客队得分 |
| winner_id | BIGINT | 获胜队伍 |
| status | ENUM | 状态 |

#### match_games (比赛小局表)

单局比赛详情。

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| match_id | BIGINT | 所属比赛 |
| game_number | TINYINT | 局数 (1-5) |
| home_power | DECIMAL | 主队战力值 |
| away_power | DECIMAL | 客队战力值 |
| home_performance | DECIMAL | 主队发挥值 |
| away_performance | DECIMAL | 客队发挥值 |
| winner_id | BIGINT | 获胜队伍 |

#### player_match_stats (选手比赛表现表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| match_game_id | BIGINT | 所属小局 |
| player_id | BIGINT | 选手 |
| team_id | BIGINT | 所属队伍 |
| base_ability | TINYINT | 基础能力 |
| form_bonus | TINYINT | 状态加成 |
| performance | DECIMAL | 实际发挥值 |
| contribution | DECIMAL | 贡献值 |

### 6. 积分系统

#### league_standings (联赛积分榜表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| tournament_id | BIGINT | 所属联赛 |
| team_id | BIGINT | 战队 |
| rank | INT | 排名 |
| matches_played | INT | 已比场次 |
| wins | INT | 胜场 |
| losses | INT | 负场 |
| points | INT | 积分 |
| games_won | INT | 小局胜场 |
| games_lost | INT | 小局负场 |
| game_diff | INT | 净胜小局 |

#### annual_points_detail (年度积分明细表)

记录每个赛事获得的积分明细。

#### global_rankings (全球积分排名表)

全球战队积分排名。

### 7. 荣誉系统

#### honors (荣誉表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| season_id | BIGINT | 赛季 |
| tournament_id | BIGINT | 赛事 |
| team_id | BIGINT | 获得战队 |
| honor_type | ENUM | 荣誉类型 |
| description | VARCHAR(255) | 描述 |

#### player_honors (选手荣誉关联表)

关联选手与荣誉的多对多关系。

### 8. 选秀系统

#### draft_pool (选秀池表)

存储选秀候选新秀。

#### draft_order (选秀顺位表)

存储各队伍的选秀顺位。

#### draft_results (选秀结果表)

存储选秀执行结果。

### 9. 转会系统

#### transfer_records (转会记录表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| season_id | BIGINT | 转会赛季 |
| player_id | BIGINT | 转会选手 |
| from_team_id | BIGINT | 原球队 |
| to_team_id | BIGINT | 新球队 |
| transfer_type | ENUM | 转会类型 |
| transfer_fee | BIGINT | 转会费 |
| new_salary | BIGINT | 新薪资 |
| contract_years | INT | 合同年限 |

#### transfer_listings (转会市场挂牌表)

球队挂牌出售的选手。

#### free_agents (自由球员市场表)

自由球员列表。

### 10. 财务系统

#### team_finances (球队财务表)

每赛季财务快照。

#### financial_transactions (财务明细表)

| 字段 | 类型 | 说明 |
|-----|------|------|
| id | BIGINT | 主键 |
| save_id | CHAR(36) | 所属存档 |
| team_id | BIGINT | 战队 |
| season_id | BIGINT | 赛季 |
| type | ENUM | 交易类型 |
| amount | BIGINT | 金额 (正=收入,负=支出) |
| description | VARCHAR(255) | 描述 |

### 11. 系统配置

#### points_config (积分配置表)

各赛事各名次的积分配置。

#### financial_config (财务配置表)

财务相关参数配置。

### 12. 历史交锋

#### head_to_head_stats (战队历史交锋统计表)

记录两队之间的历史交手数据。

## 索引设计

### 核心索引

```sql
-- 存档相关查询
CREATE INDEX idx_save_region ON teams(save_id, region_id);
CREATE INDEX idx_save_team ON players(save_id, team_id);
CREATE INDEX idx_save_type ON tournaments(save_id, type);

-- 比赛查询
CREATE INDEX idx_tournament_stage ON matches(tournament_id, stage);
CREATE INDEX idx_team_matches ON matches(home_team_id);
CREATE INDEX idx_team_matches_away ON matches(away_team_id);

-- 统计查询
CREATE INDEX idx_player_stats ON player_match_stats(player_id);
CREATE INDEX idx_power_rating ON teams(power_rating);
CREATE INDEX idx_annual_points ON teams(annual_points);
```

## 数据库迁移

迁移代码位于 `src-tauri/src/db/migrations.rs`，使用字段检测进行增量迁移：

```rust
// 添加新列示例
if !column_names.contains(&"new_column") {
    sqlx::query("ALTER TABLE table ADD COLUMN new_column TYPE")
        .execute(pool).await?;
}
```

## 文件位置

| 文件 | 说明 |
|-----|------|
| `src-tauri/src/db/connection.rs` | 数据库连接管理 |
| `src-tauri/src/db/migrations.rs` | 数据库迁移 |
| `src-tauri/src/db/repository/` | 数据仓库接口（模块目录） |
| `src-tauri/src/models/*.rs` | 数据模型定义 |
