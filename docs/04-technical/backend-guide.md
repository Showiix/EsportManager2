# 后端开发指南

## 概述

EsportManager 2 后端使用 **Rust + Tauri 2.0** 构建，采用分层架构设计，主要包含 Commands、Services、Engines、Models、DB 五层。

## 架构分层

```
┌─────────────────────────────────────────────────────────┐
│                    Commands 层                          │
│              (Tauri Command API 接口)                   │
├─────────────────────────────────────────────────────────┤
│                    Services 层                          │
│              (业务逻辑编排服务)                          │
├─────────────────────────────────────────────────────────┤
│                    Engines 层                           │
│              (核心计算引擎)                              │
├─────────────────────────────────────────────────────────┤
│                    Models 层                            │
│              (数据模型定义)                              │
├─────────────────────────────────────────────────────────┤
│                      DB 层                              │
│              (数据库操作)                                │
└─────────────────────────────────────────────────────────┘
```

## 目录结构

```
src-tauri/src/
├── main.rs                    # 应用入口
├── lib.rs                     # 库入口
├── commands/                  # Tauri Commands
│   ├── mod.rs                # 模块导出
│   ├── time_commands.rs      # 时间命令
│   ├── match_commands.rs     # 比赛命令
│   ├── team_commands.rs      # 战队命令
│   ├── transfer_commands.rs  # 转会命令
│   ├── draft_commands.rs     # 选秀命令
│   ├── finance_commands.rs   # 财务命令
│   ├── honor_commands.rs     # 荣誉命令
│   ├── stats_commands.rs     # 统计命令
│   └── ...
├── services/                  # 业务服务
│   ├── mod.rs
│   ├── game_flow.rs          # 游戏流程服务
│   ├── tournament_service.rs # 赛事服务
│   ├── honor_service.rs      # 荣誉服务
│   └── ...
├── engines/                   # 核心引擎
│   ├── mod.rs
│   ├── match_simulation.rs   # 比赛模拟引擎
│   ├── transfer/           # 转会引擎（模块目录）
│   ├── financial.rs          # 财政引擎
│   ├── draft.rs              # 选秀引擎
│   ├── honor.rs              # 荣誉引擎
│   ├── points_calculation.rs # 积分计算引擎
│   ├── market_value.rs       # 身价引擎
│   ├── player_decision.rs    # 选手决策引擎
│   ├── season_progress.rs    # 赛季进度引擎
│   └── ...
├── models/                    # 数据模型
│   ├── mod.rs
│   ├── player.rs             # 选手模型
│   ├── team.rs               # 战队模型
│   ├── tournament.rs         # 赛事模型
│   ├── season.rs             # 赛季模型
│   ├── honor.rs              # 荣誉模型
│   ├── transfer.rs           # 转会模型
│   └── ...
├── db/                        # 数据库操作
│   ├── mod.rs
│   ├── connection.rs         # 数据库连接
│   ├── migrations.rs         # 数据库迁移
│   └── repository/         # 数据仓库（模块目录）
└── errors/                    # 错误处理
    ├── mod.rs
    └── error_codes.rs        # 错误码定义
```

## Commands 层

### 定义规范

```rust
// src-tauri/src/commands/example_commands.rs

use tauri::State;
use sqlx::SqlitePool;

/// 命令函数，使用 tauri::command 宏标注
#[tauri::command]
pub async fn get_something(
    pool: State<'_, SqlitePool>,  // 数据库连接池
    save_id: String,               // 参数
    id: i64,
) -> Result<SomeModel, String> {
    // 调用 service 或直接操作数据库
    let result = some_service::get_by_id(&pool, &save_id, id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}
```

### 注册命令

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // 时间命令
            commands::time_commands::get_time_state,
            commands::time_commands::time_init_phase,
            // 比赛命令
            commands::match_commands::simulate_match,
            // ... 更多命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Engines 层

### 比赛模拟引擎

```rust
// src-tauri/src/engines/match_simulation.rs

pub struct MatchSimulationEngine {
    std_dev: f64,  // 标准差 σ = 6
}

impl MatchSimulationEngine {
    pub fn new() -> Self {
        Self { std_dev: 6.0 }
    }

    /// 模拟单局比赛
    pub fn simulate_game(
        &self,
        home_power: f64,
        away_power: f64,
        home_team_id: i64,
        away_team_id: i64,
    ) -> GameResult {
        // 使用正态分布生成发挥值
        let home_performance = self.gaussian_random(home_power);
        let away_performance = self.gaussian_random(away_power);

        let winner_id = if home_performance > away_performance {
            home_team_id
        } else {
            away_team_id
        };

        GameResult {
            home_performance,
            away_performance,
            winner_id,
        }
    }

    /// 模拟完整比赛 (BO3/BO5)
    pub fn simulate_match(&self, match_info: &Match) -> MatchResult {
        let wins_needed = match match_info.format {
            MatchFormat::Bo1 => 1,
            MatchFormat::Bo3 => 2,
            MatchFormat::Bo5 => 3,
        };

        // 循环模拟直到一方获胜
        // ...
    }
}
```

### 转会引擎

```rust
// src-tauri/src/engines/transfer/mod.rs

pub struct TransferEngine;

impl TransferEngine {
    /// 执行 8 轮转会
    pub async fn execute_transfer_window(
        pool: &SqlitePool,
        save_id: &str,
    ) -> Result<TransferWindowResult> {
        let mut results = Vec::new();

        // 第1轮: 赛季结算
        results.push(self.execute_round_1(pool, save_id).await?);

        // 第2轮: 续约谈判
        results.push(self.execute_round_2(pool, save_id).await?);

        // ... 其他轮次

        Ok(TransferWindowResult { rounds: results })
    }

    /// 计算转会意愿
    fn calculate_willingness(
        player: &Player,
        from_team: &Team,
        to_team: &Team,
    ) -> f64 {
        let mut willingness = 50.0;

        // 战力差异
        if to_team.power_rating > from_team.power_rating {
            willingness += (to_team.power_rating - from_team.power_rating) * 2.0;
        }

        // 赛区偏好
        if player.home_region_id != to_team.region_id {
            willingness *= (100.0 - player.region_loyalty as f64) / 100.0;
        }

        willingness.clamp(0.0, 100.0)
    }
}
```

## Models 层

### 模型定义规范

```rust
// src-tauri/src/models/player.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 选手位置枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum Position {
    #[sqlx(rename = "TOP")]
    Top,
    #[sqlx(rename = "JUG")]
    Jug,
    #[sqlx(rename = "MID")]
    Mid,
    #[sqlx(rename = "ADC")]
    Adc,
    #[sqlx(rename = "SUP")]
    Sup,
}

/// 选手模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: i64,
    pub save_id: String,
    pub team_id: Option<i64>,
    pub game_id: String,
    pub real_name: Option<String>,
    pub age: i32,
    pub ability: i32,
    pub potential: i32,
    pub stability: i32,
    pub position: Position,
    pub salary: i64,
    pub market_value: i64,
    pub is_starter: bool,
    // ...
}
```

### 枚举序列化

```rust
// 使用 serde rename_all 统一序列化格式
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeasonPhase {
    SpringRegular,
    SpringPlayoffs,
    // ...
}
```

## DB 层

### 数据库连接

```rust
// src-tauri/src/db/connection.rs

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn create_pool(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite:{}", db_path))
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // 执行数据库迁移
    migrations::run_all(pool).await
}
```

### 数据库迁移

```rust
// src-tauri/src/db/migrations.rs

pub async fn run_all(pool: &SqlitePool) -> Result<()> {
    // 检查并添加新列
    let columns = get_column_names(pool, "players").await?;

    if !columns.contains(&"region_loyalty".to_string()) {
        sqlx::query("ALTER TABLE players ADD COLUMN region_loyalty INTEGER DEFAULT 60")
            .execute(pool)
            .await?;
    }

    Ok(())
}
```

### Repository 模式

```rust
// src-tauri/src/db/repository/player.rs

pub struct PlayerRepository;

impl PlayerRepository {
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Player>> {
        let player = sqlx::query_as::<_, Player>(
            "SELECT * FROM players WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(player)
    }

    pub async fn find_by_team(pool: &SqlitePool, team_id: i64) -> Result<Vec<Player>> {
        let players = sqlx::query_as::<_, Player>(
            "SELECT * FROM players WHERE team_id = ? ORDER BY position"
        )
        .bind(team_id)
        .fetch_all(pool)
        .await?;

        Ok(players)
    }

    pub async fn update(pool: &SqlitePool, player: &Player) -> Result<()> {
        sqlx::query(
            "UPDATE players SET ability = ?, salary = ?, ... WHERE id = ?"
        )
        .bind(player.ability)
        .bind(player.salary)
        .bind(player.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
```

## 错误处理

### 自定义错误类型

```rust
// src-tauri/src/errors/mod.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Insufficient funds")]
    InsufficientFunds,
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}
```

## 开发常用命令

```bash
# 编译检查
cargo check --manifest-path src-tauri/Cargo.toml

# 运行测试
cargo test --manifest-path src-tauri/Cargo.toml

# 格式化代码
cargo fmt --manifest-path src-tauri/Cargo.toml

# 代码检查
cargo clippy --manifest-path src-tauri/Cargo.toml

# 启动开发模式
npm run tauri dev
```

## 添加新功能流程

1. **定义数据模型** (`models/`)
2. **添加数据库迁移** (`db/migrations.rs`)
3. **实现核心引擎** (`engines/`)
4. **编写业务服务** (`services/` - 可选)
5. **暴露 Tauri 命令** (`commands/`)
6. **在 main.rs 注册命令**
7. **添加前端 API 封装**

## 文件位置

| 文件 | 说明 |
|-----|------|
| `src-tauri/Cargo.toml` | Rust 依赖配置 |
| `src-tauri/tauri.conf.json` | Tauri 配置 |
| `src-tauri/src/main.rs` | 应用入口 |
| `src-tauri/src/lib.rs` | 库入口 |
