# 测试指南

## 概述

项目采用前后端双重测试体系：Rust 后端使用内置测试框架 + tokio，Vue 前端使用 Vitest + happy-dom。CI 流水线自动执行所有测试。

## 测试命令

```bash
# Rust 后端测试
cargo test --manifest-path src-tauri/Cargo.toml

# 前端测试
npm test                    # 单次运行
npm run test:watch          # 监视模式
npm run test:coverage       # 覆盖率报告

# 全量检查（CI 等效）
cargo check --manifest-path src-tauri/Cargo.toml && \
cargo test --manifest-path src-tauri/Cargo.toml && \
npx vue-tsc --noEmit && \
npx vitest run
```

## Rust 后端测试

### 技术栈

| 依赖 | 用途 |
|------|------|
| `#[test]` | 同步单元测试 |
| `#[tokio::test]` | 异步单元测试（数据库层） |
| `rand::SeedableRng` | 可复现的随机数测试 |
| `sqlx::SqlitePool` | 内存 SQLite 数据库测试 |

### 测试分布

| 模块 | 位置 | 测试内容 |
|------|------|----------|
| 比赛模拟 | `engines/match_simulation.rs` | BO1/BO3/BO5、胜率、特性效果 |
| 转会系统 | `engines/transfer.rs` | 概率取整、稳定性评分、策略决策、意愿计算、匹配度、缓存操作 |
| 数据库层 | `db/repository.rs` | Save CRUD、Team 创建/查询、Player 创建/查询 |
| 选手模型 | `models/player.rs` | PlayerTag、Position、RegionCode、LoyaltyType、忠诚度方法 |
| 战队模型 | `models/team.rs` | 胜率计算、FinancialStatus 分级/边界值 |
| 选手状态 | `engines/condition.rs` | 状态计算、动量系统 |
| 选秀系统 | `engines/draft.rs` | 选秀年判定、球员生成、选秀执行 |
| 荣誉系统 | `engines/honor.rs` | MVP 计算、荣誉统计 |
| 财政系统 | `engines/financial.rs` | 奖金、赞助、转会预算 |
| 积分系统 | `engines/points_calculation.rs` | 积分配置、排名 |
| 满意度 | `engines/satisfaction.rs` | 离队阈值、忠诚度变化 |
| 特性系统 | `engines/traits.rs` | 修正值、组合效果 |
| 身价系统 | `engines/market_value.rs` | 荣誉系数、衰减 |

### 编写数据库测试

数据库测试使用内存 SQLite，通过 `SCHEMA_SQL`（`pub(crate)`）创建表结构：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::SCHEMA_SQL;

    async fn setup_test_db() -> Pool<Sqlite> {
        use sqlx::sqlite::SqlitePoolOptions;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await.unwrap();

        for stmt in SCHEMA_SQL.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() { continue; }
            // 允许 INDEX 创建失败（部分表由迁移脚本创建）
            let result = sqlx::query(trimmed).execute(&pool).await;
            if let Err(e) = result {
                if !trimmed.to_uppercase().contains("CREATE INDEX") {
                    panic!("Schema error: {}", e);
                }
            }
        }

        // 执行必要的迁移列
        let migrations = [
            "ALTER TABLE teams ADD COLUMN brand_value REAL NOT NULL DEFAULT 50.0",
            "ALTER TABLE players ADD COLUMN loyalty INTEGER NOT NULL DEFAULT 50",
            // ... 按需添加
        ];
        for sql in &migrations {
            sqlx::query(sql).execute(&pool).await.ok();
        }
        pool
    }

    #[tokio::test]
    async fn test_example() {
        let pool = setup_test_db().await;
        // ... 测试逻辑
    }
}
```

### 编写引擎测试

对于含随机数的函数，使用固定种子 RNG 保证可复现：

```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn test_with_rng() {
    let mut rng = StdRng::seed_from_u64(42);
    let result = some_random_function(&mut rng);
    assert!(result > 0.0);
}
```

## Vue 前端测试

### 技术栈

| 依赖 | 用途 |
|------|------|
| `vitest` | 测试框架（兼容 Vite） |
| `happy-dom` | 浏览器环境模拟 |
| `@vue/test-utils` | Vue 组件测试工具 |
| `@pinia/testing` | Pinia Store 测试工具 |

### 配置

测试配置在 `vitest.config.ts`：
- 环境：`happy-dom`
- 全局 API：`globals: true`（无需手动 import `describe`/`it`/`expect`）
- 路径别名：`@` → `src/`
- Tauri Mock：`@tauri-apps/api/*` 自动重定向到 `src/__mocks__/`

### 测试分布

| 模块 | 位置 | 测试内容 |
|------|------|----------|
| 选手引擎 | `engines/__tests__/PlayerEngine.test.ts` | 高斯随机、稳定性σ、发挥钳位、年龄稳定性、状态范围 |
| 战力引擎 | `engines/__tests__/PowerEngine.test.ts` | 队伍战力均值、影响力分数、BO1/BO3/BO5 模拟、强队胜率 |
| 格式化工具 | `utils/__tests__/format.test.ts` | formatMoney 自动单位、强制单位、薪资/预算格式、parseMoney 反解析 |

### Tauri API Mock

前端测试无法访问 Tauri 运行时，所有 `@tauri-apps/api` 调用通过 mock 层拦截：

```typescript
// src/__mocks__/@tauri-apps/api/core.ts
export async function invoke<T>(command: string): Promise<T> {
  if (mockResponses.has(command)) return mockResponses.get(command) as T
  throw new Error(`No mock for command: ${command}`)
}

// 测试中设置 mock 响应
import { __setMockResponse } from '@tauri-apps/api/core'
__setMockResponse('get_time_state', { season: 1, phase: 'SpringRegular' })
```

### 编写新测试

测试文件放在被测模块的 `__tests__/` 子目录中：

```
src/engines/__tests__/PlayerEngine.test.ts   → 测试 src/engines/PlayerEngine.ts
src/utils/__tests__/format.test.ts           → 测试 src/utils/format.ts
src/stores/__tests__/gameStore.test.ts       → 测试 src/stores/gameStore.ts
```

基本结构：

```typescript
import { describe, it, expect } from 'vitest'
import { SomeModule } from '../SomeModule'

describe('SomeModule', () => {
  describe('methodName', () => {
    it('should do something specific', () => {
      const result = SomeModule.methodName(input)
      expect(result).toBe(expected)
    })
  })
})
```

## CI 集成

`.github/workflows/ci.yml` 在 push/PR 时自动运行：

| Job | 步骤 |
|-----|------|
| `rust-check` | `cargo check` → `cargo test` |
| `frontend-check` | `npm ci` → `vue-tsc --noEmit` → `vitest run` |

## 扩展测试的优先级建议

当前零测试覆盖、推荐优先补充的模块：

| 优先级 | 模块 | 理由 |
|--------|------|------|
| 高 | `commands/` (23个模块) | 命令层是前后端桥梁，参数校验容易出错 |
| 高 | `services/game_flow.rs` | 赛季推进是核心流程 |
| 中 | `engines/financial.rs` 补充 | 奖金/赞助计算影响游戏平衡 |
| 中 | Vue Store 测试 | 状态管理逻辑复杂 |
| 低 | Vue 组件测试 | UI 层变化频繁，ROI 较低 |

## 文件位置

| 文件 | 说明 |
|------|------|
| `vitest.config.ts` | 前端测试配置 |
| `src/__mocks__/@tauri-apps/api/` | Tauri API mock 层 |
| `src/**/__tests__/*.test.ts` | 前端测试文件 |
| `src-tauri/src/**` 中的 `#[cfg(test)]` | Rust 测试模块 |
| `.github/workflows/ci.yml` | CI 测试流水线 |
| `src-tauri/Cargo.toml` `[dev-dependencies]` | Rust 测试依赖 |
