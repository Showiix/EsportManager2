# 技术栈说明

## 整体架构

| 层级 | 技术 | 版本 |
|------|------|------|
| 前端框架 | Vue 3 + TypeScript | Vue 3.x |
| UI 组件库 | Element Plus | - |
| 状态管理 | Pinia | - |
| 桌面框架 | Tauri | 2.0 |
| 后端语言 | Rust | - |
| 数据库 | SQLite (sqlx) | - |
| 前后端通信 | Tauri Commands | IPC |

## 前端技术栈

### Vue 3 + TypeScript

- **Composition API**: 使用 `<script setup>` 语法
- **类型安全**: 完整的 TypeScript 类型定义
- **响应式系统**: `ref`, `reactive`, `computed`

### Element Plus

- 企业级 UI 组件库
- 主题定制支持
- 国际化支持

### Pinia

状态管理架构：

```typescript
// 示例 Store 结构
export const useTimeStore = defineStore('time', () => {
  const timeState = ref<GameTimeState | null>(null)
  const isLoading = ref(false)

  async function fetchTimeState() {
    isLoading.value = true
    timeState.value = await timeApi.getTimeState()
    isLoading.value = false
  }

  return { timeState, isLoading, fetchTimeState }
})
```

### 前端目录结构

```
src/
├── api/                      # API 客户端
│   └── tauri.ts             # Tauri 命令封装
├── components/               # Vue 组件
│   ├── match/               # 比赛相关
│   ├── player/              # 选手相关
│   └── finance/             # 财务相关
├── engines/                  # 前端计算引擎
│   ├── PlayerEngine.ts      # 选手能力计算
│   └── PowerEngine.ts       # 队伍战力计算
├── stores/                   # Pinia 状态管理
├── views/                    # 页面视图
├── types/                    # TypeScript 类型
└── router/                   # Vue Router
```

## 后端技术栈

### Rust

- **安全性**: 内存安全，无运行时开销
- **性能**: 接近 C/C++ 的执行效率
- **并发**: 安全的并发编程模型

### Tauri 2.0

桌面应用框架：

- 跨平台支持 (Windows, macOS, Linux)
- 原生系统 API 访问
- 安全的进程间通信 (IPC)
- 小巧的打包体积

### SQLite + sqlx

- **sqlx**: 编译时检查的 SQL 查询
- **异步**: 完整的异步数据库操作
- **迁移**: 内置数据库迁移支持

### 后端目录结构

```
src-tauri/src/
├── commands/                # Tauri Commands (API接口)
│   ├── game_commands.rs     # 核心游戏命令
│   ├── save_commands.rs     # 存档管理
│   ├── time_commands.rs     # 时间推进
│   ├── match_commands.rs    # 比赛模拟
│   ├── match_detail_commands.rs # 比赛详情
│   ├── team_commands.rs     # 战队管理
│   ├── honor_commands.rs    # 荣誉系统
│   ├── finance_commands.rs  # 财政系统
│   ├── transfer_commands.rs # 转会系统
│   ├── draft_commands.rs    # 选秀系统
│   ├── draft_auction_commands.rs # 选秀权拍卖
│   ├── points_commands.rs   # 年度积分
│   ├── stats_commands.rs    # 数据统计
│   ├── query_commands.rs    # 通用查询
│   ├── international_commands.rs # 国际赛事
│   ├── event_commands.rs    # 事件系统
│   ├── awards_commands.rs   # 颁奖典礼
│   ├── dev_commands.rs      # 开发调试
│   ├── log_commands.rs      # 日志系统
│   └── perf_commands.rs     # 性能监控
├── engines/                 # 核心计算引擎
│   ├── match_simulation.rs  # 比赛模拟
│   ├── transfer.rs          # 转会引擎
│   ├── financial.rs         # 财政引擎
│   ├── points_calculation.rs # 积分计算
│   ├── honor.rs             # 荣誉引擎
│   ├── draft.rs             # 选秀引擎
│   ├── draft_auction.rs     # 选秀权拍卖
│   ├── market_value.rs      # 身价计算
│   ├── season_progress.rs   # 赛季进度
│   ├── power_engine.rs      # 队伍战力
│   ├── traits.rs            # 选手特性
│   ├── condition.rs         # 选手状态
│   ├── satisfaction.rs      # 满意度引擎
│   ├── player_decision.rs   # AI 选手决策
│   ├── player_performance.rs # 表现评估
│   └── event.rs             # 事件引擎
├── models/                  # 数据模型
│   ├── player.rs            # 选手模型
│   ├── team.rs              # 战队模型
│   ├── tournament.rs        # 赛事模型
│   ├── transfer.rs          # 转会模型
│   ├── financial.rs         # 财务模型
│   └── ...                  # (共20个模型文件)
├── services/                # 业务服务层
│   ├── game_flow.rs         # 游戏流程核心
│   ├── league_service.rs    # 联赛服务
│   ├── honor_service.rs     # 荣誉服务
│   ├── tournament_service.rs # 赛事服务
│   ├── init_service.rs      # 初始化服务
│   ├── player_data.rs       # 选手初始数据
│   ├── draft_pool_data.rs   # 选秀池数据
│   └── ...                  # (共11个服务文件)
└── db/                      # 数据库操作
    ├── connection.rs         # 连接与迁移
    ├── migrations.rs         # 迁移工具
    └── repository.rs         # 数据仓库（统一）
```

## 前后端通信

### Tauri Command 示例

**后端定义**:

```rust
#[tauri::command]
pub async fn get_time_state(
    state: State<'_, AppState>,
    save_id: String,
) -> Result<GameTimeState, String> {
    let pool = &state.db_pool;
    let service = GameFlowService::new();
    service.get_time_state(pool, &save_id)
        .await
        .map_err(|e| e.to_string())
}
```

**前端调用**:

```typescript
// src/api/tauri.ts
export const timeApi = {
  async getTimeState(): Promise<GameTimeState> {
    return invoke('get_time_state', { saveId: getCurrentSaveId() })
  }
}
```

## 数据库

### SQLite 配置

- 单文件数据库，便于存档管理
- WAL 模式提升并发性能
- 完整的外键约束支持

### 数据库迁移

在 `src-tauri/src/db/connection.rs` 中管理迁移：

```rust
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // 检查并添加新列
    if !column_names.contains(&"new_column") {
        sqlx::query("ALTER TABLE table ADD COLUMN new_column TYPE")
            .execute(pool).await?;
    }
    Ok(())
}
```

## 开发工具

### 必要环境

- Node.js 18+
- Rust (rustup)
- 系统依赖 (参考 Tauri 文档)

### 开发命令

```bash
# 安装依赖
npm install

# 启动开发环境
npm run tauri dev

# 前端类型检查
npx vue-tsc --noEmit

# Rust 编译检查
cargo check --manifest-path src-tauri/Cargo.toml

# 格式化 Rust 代码
cargo fmt --manifest-path src-tauri/Cargo.toml

# 构建发布版本
npm run tauri build
```

### 推荐 IDE 配置

- **VS Code** + rust-analyzer + Vue - Official
- **RustRover** (JetBrains)
