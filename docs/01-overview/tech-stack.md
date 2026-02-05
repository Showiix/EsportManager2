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
│   ├── time_commands.rs
│   ├── match_commands.rs
│   ├── player_commands.rs
│   └── ...
├── engines/                 # 核心计算引擎
│   ├── match_simulation.rs
│   ├── transfer.rs
│   ├── financial.rs
│   └── points_calculation.rs
├── models/                  # 数据模型
│   ├── player.rs
│   ├── team.rs
│   ├── tournament.rs
│   └── ...
├── services/                # 业务服务层
│   ├── game_flow.rs
│   ├── league.rs
│   └── honor.rs
└── db/                      # 数据库操作
    ├── connection.rs
    └── repository/
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
