# 选秀系统

## 概述

选秀系统为球队补充新秀，每4个赛季进行一次（S2, S6, S10...）。

## 选秀时间

```
选秀年公式: (season - 2) % 4 == 0

选秀年: S2, S6, S10, S14, S18...
非选秀年: S1, S3, S4, S5, S7, S8, S9...
```

## 选秀规则

### 选秀池

- 每赛区 14 名新秀
- 由管理员提前导入数据库
- 包含: 选手名称、能力值、潜力值、标签

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

## 选秀数据结构

### 选秀池

```rust
pub struct DraftPool {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub player_id: u64,
    pub rank: u32,              // 选秀排名 (1-14)
    pub is_selected: bool,
    pub selected_by: Option<u64>,
}
```

### 选秀顺位

```rust
pub struct DraftPick {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub team_id: u64,
    pub pick_order: u32,        // 选秀顺位
    pub selected_player_id: Option<u64>,
}
```

## 选秀流程

1. **展示选秀名单**
   - 选手名称、能力值、潜力值、标签

2. **选秀权抽签**
   - 根据排名加权随机
   - 可逐个抽签或一键完成

3. **选秀执行**
   - 按顺位依次选择选手
   - 选手加入对应球队

4. **选秀结束**
   - 更新球队阵容
   - 进入转会系统

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
| `get_draft_pool(save_id, region_id)` | 获取选秀池 |
| `import_draft_pool(players)` | 导入选秀名单 |
| `start_draft_lottery(region_id)` | 开始选秀抽签 |
| `execute_draft_pick(pick_id, player_id)` | 执行选秀 |
| `auto_complete_draft(region_id)` | 自动完成选秀 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `src-tauri/src/models/draft.rs` | 选秀数据模型 |
| `src-tauri/src/engines/draft.rs` | 选秀引擎 |
| `src-tauri/src/commands/draft_commands.rs` | 选秀命令接口 |
