---
name: player-contract-center
description: 选手合同中心页面的开发与维护。用于查看、筛选、排序所有选手的合同信息、战队归属、身价、满意度等数据。当需要修改选手合同中心页面功能或样式时使用此技能。
---

# 选手合同中心 (Player Contract Center)

## Overview

选手合同中心是一个专业的数据展示页面，用于显示所有选手的合同状态、战队信息、能力值、身价等关键数据。页面支持多条件筛选、排序和分页功能。

## 架构

### 前端组件

**主页面文件**: `src/views/PlayerMarket.vue`

核心功能:
- 多条件筛选（状态、位置、赛区、战队、合同状态）
- 表格排序（能力值、潜力、年龄、薪资、满意度、身价）
- 分页显示（默认每页15条）
- 身价格式化（K/M/B单位）
- 进度条可视化（能力值、潜力、满意度、忠诚度）

### 后端命令

**命令文件**: `src-tauri/src/commands/transfer_commands.rs`

命令: `get_player_market_list`

```rust
#[tauri::command]
pub async fn get_player_market_list(
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<PlayerContractInfo>>, String>
```

### 数据模型

**模型文件**: `src-tauri/src/models/transfer.rs`

```rust
pub struct PlayerContractInfo {
    pub player_id: i64,
    pub player_name: String,
    pub position: Option<String>,
    pub age: i64,
    pub ability: i64,
    pub potential: i64,
    pub team_id: Option<i64>,
    pub team_name: Option<String>,
    pub region_code: Option<String>,
    pub salary: i64,
    pub contract_end_season: Option<i64>,
    pub join_season: Option<i64>,
    pub base_market_value: i64,
    pub calculated_market_value: i64,
    pub satisfaction: i64,
    pub loyalty: i64,
    pub is_starter: bool,
    pub status: String,
}
```

### API 接口

**接口文件**: `src/api/tauri.ts`

```typescript
export interface PlayerMarketInfo {
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  team_id: number | null
  team_name: string | null
  region_code: string | null
  salary: number
  contract_end_season: number | null
  join_season: number | null
  base_market_value: number
  calculated_market_value: number
  satisfaction: number
  loyalty: number
  is_starter: boolean
  status: string
}

export function getPlayerMarketList(): Promise<CommandResult<PlayerMarketInfo[]>>
```

## 筛选器配置

### 状态筛选
- `ACTIVE` - 现役
- `FREE_AGENT` - 自由球员
- `RETIRED` - 退役 (默认隐藏)

### 位置筛选
- `TOP`, `JUG`, `MID`, `BOT`, `SUP`

### 合同状态筛选
- 即将到期（1赛季内）
- 长期合同（3赛季以上）
- 自由球员

## 性能优化要点

1. **真分页渲染**: 使用 `paginatedPlayers` computed 属性，只渲染当前页数据
2. **线性进度条**: 使用 `el-progress` 线性模式替代圆形进度条，减少渲染开销
3. **固定列z-index**: 固定列需要设置 `z-index: 2` 防止滚动时内容溢出

```css
.player-table :deep(.el-table__fixed),
.player-table :deep(.el-table__fixed-right) {
  z-index: 2;
}

.player-table :deep(.el-table__fixed-body-wrapper) td,
.player-table :deep(.el-table__fixed-right .el-table__fixed-body-wrapper) td {
  overflow: hidden;
}
```

## 常见修改任务

### 添加新筛选条件
1. 在 `filters` 响应式对象中添加新字段
2. 在筛选器区域添加对应的 UI 组件
3. 在 `filteredPlayers` computed 中添加筛选逻辑

### 添加新表格列
1. 确保后端 SQL 查询包含该字段
2. 更新 `PlayerContractInfo` (Rust) 和 `PlayerMarketInfo` (TypeScript) 接口
3. 在表格中添加 `el-table-column`

### 修改表格高度
调整 `.player-table` 的 `max-height` 值:
```css
.player-table :deep(.el-table) {
  max-height: calc(100vh - 280px);
}
```

## 相关文件

| 文件 | 用途 |
|------|------|
| `src/views/PlayerMarket.vue` | 前端页面组件 |
| `src-tauri/src/commands/transfer_commands.rs` | 后端命令实现 |
| `src-tauri/src/models/transfer.rs` | 数据模型定义 |
| `src/api/tauri.ts` | 前端API接口 |
| `src-tauri/src/lib.rs` | 命令注册 |
