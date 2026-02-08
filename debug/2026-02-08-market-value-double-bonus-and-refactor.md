# 身价系统双重加成 Bug + 全面重构

**日期**: 2026-02-08
**Commit**: `a34bbe7`
**涉及文件**:
- `engines/market_value.rs` — 唯一权威引擎
- `services/game_flow.rs` — 删除重复函数 + 修复双重加成
- `engines/transfer.rs` — 删除 3 个重复函数
- `commands/dev_commands.rs` — 删除 3 个重复函数
- `services/init_service.rs` — 删除 1 个重复函数
- `engines/satisfaction.rs` — 删除本地薪资公式
- `models/player.rs` — 委托引擎
- `commands/team_commands.rs` — parse_honor_category 参数适配

## 问题描述

1. **双重加成 Bug**：年度颁奖先用 `update_player_market_value` 乘 `market_value` 列（如 MVP +50%），再用 `recalculate_all_market_values` 基于荣誉重算 `calculated_market_value`，导致荣誉加成被应用两次
2. **身价公式 5 处重复**：game_flow / transfer / dev_commands / init_service / player.rs 各写一套，且有不一致
3. **薪资公式 3 套不一致**：同一选手（72能力/22岁）三套公式差 3.8 倍
4. **能力/年龄系数阶梯断层**：72 和 100 能力值系数相同（都是 25）

## 修复方案

- 删除 `update_player_market_value` 及年度颁奖中所有 `record_bonus` / `player_max_bonus` 代码，由 `recalculate_all_market_values` 统一通过荣誉系数处理
- `recalculate_player_market_value_full` 改为同时更新 `market_value`（基础）和 `calculated_market_value`（完整）
- 所有重复函数删除，委托 `MarketValueEngine`
- 能力/年龄系数改为分段线性插值

## 影响范围

所有选手身价数值会因公式变化（连续插值 vs 阶梯）产生变动，年度颁奖后的身价不再有双重膨胀。
