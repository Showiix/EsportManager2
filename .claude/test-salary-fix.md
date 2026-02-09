# 薪资计算修复验证

## 问题描述

转会系统在计算期望薪资时只使用基础身价，导致有荣誉的选手实际薪资远低于满意度系统的期望值，触发"薪资被严重低估"惩罚。

## 修复内容

### 第2轮：续约谈判 (line 852-895)

**修改前**:
```rust
// 查询时未包含 calculated_market_value
SELECT p.id, p.game_id, p.ability, p.salary, p.loyalty, p.satisfaction,
       p.team_id, p.age, p.potential, p.tag, t.name as team_name

// 只用基础身价
let expected_salary = MarketValueEngine::estimate_salary(
    MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", "MID"),
    ability as u8, age as u8
) as i64;
```

**修改后**:
```rust
// 查询时包含 calculated_market_value
SELECT p.id, p.game_id, p.ability, p.salary, p.loyalty, p.satisfaction,
       p.team_id, p.age, p.potential, p.tag, p.calculated_market_value, t.name as team_name

// 优先使用完整身价
let calculated_market_value: i64 = player.try_get("calculated_market_value").unwrap_or(0);
let market_value = if calculated_market_value > 0 {
    calculated_market_value as u64
} else {
    MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, ability as u8, "NORMAL", "MID")
};
let expected_salary = MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64;
```

### 第4轮：自由球员竞标 (line 1669-1698)

**修改前**:
```rust
// 查询时未包含 calculated_market_value
SELECT id, game_id, ability, salary, age, position, loyalty, potential, tag,
       home_region_id, region_loyalty, stability

// 只用基础身价
let expected_salary = MarketValueEngine::estimate_salary(
    MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, potential as u8, &tag, &position),
    ability as u8, age as u8
) as i64;
```

**修改后**:
```rust
// 查询时包含 calculated_market_value
SELECT id, game_id, ability, salary, age, position, loyalty, potential, tag,
       home_region_id, region_loyalty, stability, calculated_market_value

// 优先使用完整身价
let calculated_market_value: i64 = free_agent.try_get("calculated_market_value").unwrap_or(0);
let market_value = if calculated_market_value > 0 {
    calculated_market_value as u64
} else {
    MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, potential as u8, &tag, &position)
};
let expected_salary = MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64;
```

### 第5轮：合同选手转会 (line 1982-2079)

**修改前**:
```rust
// 查询时未包含 calculated_market_value
SELECT pl.id as listing_id, pl.player_id, pl.listed_by_team_id, pl.listing_price, pl.min_accept_price,
       p.game_id, p.ability, p.age, p.position, p.salary, p.loyalty,
       p.home_region_id, p.region_loyalty, p.potential, p.tag, p.stability,
       t.name as from_team_name

// 使用转会标价计算薪资（错误！）
let base_salary = MarketValueEngine::estimate_salary(listing_price as u64, ability as u8, age as u8) as i64;
```

**修改后**:
```rust
// 查询时包含 calculated_market_value
SELECT pl.id as listing_id, pl.player_id, pl.listed_by_team_id, pl.listing_price, pl.min_accept_price,
       p.game_id, p.ability, p.age, p.position, p.salary, p.loyalty,
       p.home_region_id, p.region_loyalty, p.potential, p.tag, p.stability, p.calculated_market_value,
       t.name as from_team_name

// 使用完整身价计算薪资
let calculated_market_value: i64 = listing.try_get("calculated_market_value").unwrap_or(0);
let market_value = if calculated_market_value > 0 {
    calculated_market_value as u64
} else {
    MarketValueEngine::calculate_base_market_value(ability as u8, age as u8, potential as u8, &tag, &position)
};
let base_salary = MarketValueEngine::estimate_salary(market_value, ability as u8, age as u8) as i64;
```

## 效果对比

假设一个世界赛冠军选手（荣誉系数 2.3）：

| 项目 | 修复前 | 修复后 |
|------|--------|--------|
| 基础身价 | 1000万 | 1000万 |
| 完整身价 | 2300万（未使用） | 2300万 |
| 期望薪资（转会） | 108万/年 | 248万/年 |
| 期望薪资（满意度） | 276万/年 | 276万/年 |
| 薪资比例 | 39% → -20满意度 | 90% → 无惩罚 |

## 测试方法

1. 开始新存档，推进到第一个转会期
2. 查看有世界赛冠军或其他荣誉的选手转会后的薪资
3. 检查满意度系统是否还会出现大量"薪资被严重低估"事件

## 注意事项

- `calculated_market_value` 需要在赛季结算时更新（通过 `dev_recalculate_market_values` 或赛季推进时自动计算）
- 如果 `calculated_market_value = 0`，会回退到基础身价计算
- 旧存档需要运行一次身价重算才能生效
