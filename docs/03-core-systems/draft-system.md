# 选秀系统

## 概述

选秀系统为球队补充新秀，每4个赛季进行一次（S2, S6, S10...）。选秀池持久化存储在 `draft_pool` 表中，每赛区50人，选秀时从池中随机抽取14人组成当届选秀名单。

## 选秀时间

```
选秀年公式: (season - 2) % 4 == 0

选秀年: S2, S6, S10, S14, S18...
非选秀年: S1, S3, S4, S5, S7, S8, S9...
```

## 核心架构

### 双表分离

```
draft_pool (50人/赛区，持久化大池子)
    ↓ ORDER BY RANDOM() LIMIT 14
draft_players (14人/赛区，当届选秀名单)
    ↓ make_draft_pick
draft_results (被队伍选中 → 创建 player)
```

| 表 | 用途 | 生命周期 |
|---|------|---------|
| `draft_pool` | 持久化选秀大池子 | 跨赛季存在，选手被选后标记 `drafted` |
| `draft_players` | 当届选秀名单（14人） | 每次选秀生成，选秀结束后保留记录 |

### 数据流

```
新建存档 → init_service 读 draft_pool_data.rs → 写入 draft_pool (200人)
选秀阶段 → generate_draft_pool → draft_pool 随机抽14人 → 写入 draft_players
选手被选 → make_draft_pick → draft_pool.status='drafted' + 创建 player
```

## draft_pool 表结构

```sql
CREATE TABLE draft_pool (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    region_id INTEGER NOT NULL,
    game_id TEXT NOT NULL,
    real_name TEXT,
    nationality TEXT,
    age INTEGER NOT NULL,
    ability INTEGER NOT NULL,
    potential INTEGER NOT NULL,
    position TEXT NOT NULL,
    tag TEXT NOT NULL DEFAULT 'Normal',
    status TEXT NOT NULL DEFAULT 'available',  -- available / drafted
    drafted_season INTEGER,
    drafted_by_team_id INTEGER,
    created_season INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);
```

**索引**:
- `idx_draft_pool_save_region(save_id, region_id)`
- `idx_draft_pool_status(save_id, region_id, status)`

## 选秀规则

### 选秀池初始化

- 新建存档时，从 `draft_pool_data.rs` 读取预定义数据
- 每赛区50人，共200人写入 `draft_pool` 表
- 初始状态均为 `available`

### 选秀名单生成

- 从 `draft_pool WHERE status='available'` 中随机抽取14人
- 按综合评分排序：`ability * 0.4 + potential * 0.6`
- 写入 `draft_players` 表，分配 draft_rank（1-14）

### 选秀顺序

基于夏季赛常规赛排名：

- 排名越靠后，获得高顺位的概率越高
- 使用加权随机算法

```rust
fn draft_lottery(team_rank: u32) -> f64 {
    let base_weights = [
        1.0,   // 第1名
        1.5,   // 第2名
        2.0,   // 第3名
        2.5,   // 第4名
        3.0,   // 第5名
        3.5,   // 第6名
        4.0,   // 第7名
        4.5,   // 第8名
        5.0,   // 第9名
        5.5,   // 第10名
        6.0,   // 第11名
        6.5,   // 第12名
        7.0,   // 第13名
        8.0,   // 第14名
    ];
    base_weights[team_rank as usize - 1]
}
```

## 选秀流程

1. **抽取选秀名单** — 从 `draft_pool` 随机抽14人，写入 `draft_players`
2. **选秀权抽签** — 根据排名加权随机分配顺位
3. **选秀执行** — 按顺位依次选择选手，`draft_pool.status` 更新为 `drafted`
4. **选秀结束** — 选手加入球队，进入转会系统

## 新秀合同

新秀被选中后自动签订 **2 年**合同：

| 属性 | 值 |
|------|-----|
| 合同年限 | 2 年（current_season + 2） |
| 新秀薪资 | 20 万元 |
| 初始身价 | 50 万元 |
| home_region_id | 选秀赛区 |

## 新秀属性范围

| 属性 | 范围 | 说明 |
|------|------|------|
| 年龄 | 16-19 | 新秀年龄 |
| 能力值 | 50-75 | 初始能力 |
| 潜力值 | 70-98 | 成长上限 |
| 稳定性 | 50-70 | 较不稳定 |

## API 接口

| 接口 | 描述 |
|------|------|
| `generate_draft_pool(region_id, pool_size?)` | 从 draft_pool 随机抽取并生成当届选秀名单 |
| `get_available_draft_players(region_id)` | 获取当届可选选手 |
| `run_draft_lottery(region_id)` | 选秀权抽签 |
| `make_draft_pick(region_id, player_id, team_id)` | 执行选秀（同步更新 draft_pool） |
| `ai_auto_draft(region_id)` | AI 自动完成选秀 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/commands/draft_commands.rs` | 选秀命令接口 |
| `src-tauri/src/engines/draft.rs` | 选秀引擎（抽签算法） |
| `src-tauri/src/services/draft_pool_data.rs` | 预定义选秀池数据（200人） |
| `src-tauri/src/services/init_service.rs` | 初始化写入 draft_pool |
| `src-tauri/src/db/connection.rs` | draft_pool 表迁移 |
| `src/views/DraftRegion.vue` | 选秀页面前端 |
| `src/api/tauri.ts` | 前端 API 封装 |
