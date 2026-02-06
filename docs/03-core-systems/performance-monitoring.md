# 性能监测系统

## 概述

性能监测系统提供全栈性能数据收集与可视化功能，帮助开发者分析 IPC 调用耗时、路由导航性能，以及后端命令执行效率。

**核心特点**：
- 默认关闭，用户主动开启
- 数据存储在内存中，不持久化
- 自动淘汰旧数据（前端 5000 条 / 后端 5000 条上限）

## 核心概念

| 概念 | 说明 |
|------|------|
| **IPC 调用** | 前端通过 Tauri 调用后端命令的过程 |
| **导航计时** | Vue Router 路由切换耗时（包含渲染完成时间） |
| **慢请求** | 耗时超过 500ms 的请求 |
| **P95** | 95% 的请求耗时低于此值 |

## 数据结构

### 前端记录

```typescript
interface InvokeRecord {
  command: string      // 命令名称
  duration: number     // 耗时 (ms)
  success: boolean     // 是否成功
  error?: string       // 错误信息
  timestamp: number    // 时间戳
}

interface NavigationRecord {
  from: string         // 来源路由
  to: string           // 目标路由
  duration: number     // 耗时 (ms)
  timestamp: number    // 时间戳
}
```

### 后端记录

```rust
pub struct PerfRecord {
    pub command: String,
    pub duration_ms: f64,
    pub timestamp: String,    // ISO 8601
    pub success: bool,
    pub error: Option<String>,
    pub phases: Option<Vec<PerfPhase>>,
}

pub struct PerfPhase {
    pub name: String,
    pub duration_ms: f64,
}
```

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                        前端层                                 │
│  ┌──────────────┐    ┌─────────────────────────────────────┐ │
│  │ tauri.ts     │───▸│ usePerformanceStore                 │ │
│  │ (IPC 拦截)    │    │ ├─ invokeRecords[]                  │ │
│  └──────────────┘    │ ├─ navigationRecords[]              │ │
│                      │ ├─ frontendSummary (computed)       │ │
│  ┌──────────────┐    │ ├─ commandDistribution (computed)   │ │
│  │ router.ts    │───▸│ └─ timelineBuckets (computed)       │ │
│  │ (导航拦截)    │    └─────────────────────────────────────┘ │
│  └──────────────┘                      │                     │
│                                        ▼                     │
│                      ┌─────────────────────────────────────┐ │
│                      │ PerformanceMonitor.vue              │ │
│                      │ ├─ 统计卡片 (6个)                    │ │
│                      │ ├─ 命令耗时图表 (ECharts)            │ │
│                      │ ├─ 趋势图表 (ECharts)               │ │
│                      │ └─ 日志表格 (ElTable)               │ │
│                      └─────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼ Tauri IPC
┌─────────────────────────────────────────────────────────────┐
│                        后端层                                 │
│  ┌──────────────────────────────────────────────────────────┐│
│  │ AppState                                                 ││
│  │ └─ perf: PerfCollector                                   ││
│  │        ├─ enabled: Mutex<bool>                           ││
│  │        ├─ records: Mutex<Vec<PerfRecord>>               ││
│  │        └─ max_records: 5000                              ││
│  └──────────────────────────────────────────────────────────┘│
│                               │                              │
│  ┌──────────────────────────────────────────────────────────┐│
│  │ perf_commands.rs                                         ││
│  │ ├─ get_perf_records                                      ││
│  │ ├─ get_perf_summary                                      ││
│  │ ├─ toggle_perf_monitoring                                ││
│  │ └─ clear_perf_records                                    ││
│  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## 核心流程

### 前端 IPC 计时

```
用户操作 → invokeCommand() → 记录开始时间
                ↓
         执行 Tauri IPC
                ↓
         记录结束时间 → 计算耗时
                ↓
         perfStore.recordInvoke() → 存入 invokeRecords[]
```

### 路由导航计时

```
路由切换 → beforeEach 记录开始时间
                ↓
         afterEach 触发
                ↓
         双 requestAnimationFrame 等待渲染
                ↓
         记录结束时间 → 计算耗时
                ↓
         perfStore.recordNavigation() → 存入 navigationRecords[]
```

## 页面功能

### 统计卡片

| 卡片 | 说明 |
|------|------|
| 总请求数 | IPC 调用总次数 |
| 平均耗时 | 所有请求的平均耗时 (ms) |
| P95 耗时 | 95% 请求的耗时上限 (ms) |
| 最大耗时 | 单次请求最大耗时 (ms) |
| 慢请求数 | 耗时 > 500ms 的请求数 |
| 错误率 | 失败请求占比 (%) |

### 图表

**命令耗时 Top 10**：
- 横向柱状图
- 颜色编码：>1000ms 红色，>500ms 黄色，其他绿色

**请求趋势**：
- 双 Y 轴折线图
- 左轴：10 秒内请求数
- 右轴：10 秒内平均耗时

### 日志表格

| 列 | 说明 |
|----|------|
| 类型 | IPC / 导航 |
| 名称 | 命令名或路由路径 |
| 耗时 | 毫秒数，可排序 |
| 状态 | 成功 / 失败 / 慢 |
| 时间 | 执行时间 |
| 错误 | 错误信息（如有） |

支持：搜索、类型筛选、状态筛选、分页

## API 接口

### 前端 Store

| 方法/属性 | 说明 |
|-----------|------|
| `startMonitoring()` | 开始监测 |
| `stopMonitoring()` | 停止监测 |
| `clearAllData()` | 清除所有数据 |
| `exportData()` | 导出 JSON 文件 |
| `frontendSummary` | 统计摘要 (computed) |
| `logEntries` | 合并日志列表 (computed) |
| `commandDistribution` | 命令分布统计 (computed) |

### 后端命令

| 命令 | 参数 | 返回值 |
|------|------|--------|
| `get_perf_records` | `limit`, `command_filter` | `Vec<PerfRecord>` |
| `get_perf_summary` | - | `PerfSummary` |
| `toggle_perf_monitoring` | `enabled: bool` | `bool` |
| `clear_perf_records` | - | `bool` |

## 与日志系统的区别

| 特性 | 性能监测 | 日志系统 |
|------|---------|---------|
| 存储方式 | 内存 | 文件持久化 |
| 默认状态 | 关闭 | 开启 |
| 数据上限 | 5000 条 | 按天滚动 |
| 主要用途 | 性能分析 | 错误追踪 |
| 目标用户 | 开发者调试 | 全场景 |

## 使用示例

### 手动为后端命令添加计时

```rust
#[tauri::command]
pub async fn my_slow_command(state: State<'_, AppState>) -> Result<...> {
    let start = std::time::Instant::now();

    // 执行业务逻辑
    let result = do_something().await;

    // 记录性能数据
    if state.perf.is_enabled() {
        state.perf.record(PerfRecord {
            command: "my_slow_command".to_string(),
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            success: result.is_ok(),
            error: result.as_ref().err().map(|e| e.to_string()),
            phases: None,
        });
    }

    result
}
```

### 记录操作子阶段

```rust
let mut phases = Vec::new();

let phase_start = std::time::Instant::now();
// 阶段1: 数据加载
load_data().await?;
phases.push(PerfPhase {
    name: "数据加载".to_string(),
    duration_ms: phase_start.elapsed().as_secs_f64() * 1000.0,
});

let phase_start = std::time::Instant::now();
// 阶段2: 计算处理
process_data()?;
phases.push(PerfPhase {
    name: "计算处理".to_string(),
    duration_ms: phase_start.elapsed().as_secs_f64() * 1000.0,
});

state.perf.record(PerfRecord {
    command: "complex_operation".to_string(),
    duration_ms: total_start.elapsed().as_secs_f64() * 1000.0,
    phases: Some(phases),
    // ...
});
```

## 文件位置

| 文件 | 说明 |
|------|------|
| `src/stores/usePerformanceStore.ts` | 前端 Pinia Store |
| `src/views/PerformanceMonitor.vue` | 监测页面 |
| `src/api/tauri.ts` | IPC 调用计时拦截 |
| `src/router/index.ts` | 路由导航计时 |
| `src-tauri/src/services/perf_service.rs` | 后端性能收集器 |
| `src-tauri/src/commands/perf_commands.rs` | 后端性能命令 |
