---
name: performance-monitoring-system
description: 全栈性能监测系统。提供 IPC 命令计时、路由导航计时、后端命令执行统计、实时图表展示和日志查看。当需要分析性能瓶颈、监测命令耗时、查看请求统计或调试慢操作时使用此技能。
---

# 性能监测系统 (Performance Monitoring System)

## Overview

性能监测系统提供全栈性能数据收集与可视化，包括前端 IPC 调用计时、路由导航计时、后端命令执行统计，以及实时图表和日志查看功能。

**核心特点**：
- 默认关闭，用户主动开启
- 数据存储在内存中，不持久化
- 前端 5000 条 / 后端 5000 条记录上限，自动淘汰旧数据

## 架构

```
[Rust 命令执行] → [PerfCollector (内存)] → [Tauri 命令暴露]
                                                    ↓
[前端 invokeCommand 计时] → [usePerformanceStore] → [PerformanceMonitor.vue]
[Vue Router 导航计时]   ↗                            (卡片 + 图表 + 表格)
```

## 前端

### Pinia Store

**文件**: `src/stores/usePerformanceStore.ts`

```typescript
import { usePerformanceStore } from '@/stores/usePerformanceStore'

const perfStore = usePerformanceStore()

// 开始/停止监测
perfStore.startMonitoring()
perfStore.stopMonitoring()

// 清除数据
perfStore.clearAllData()

// 导出数据
perfStore.exportData()

// 计算属性
perfStore.frontendSummary  // { total, avgDuration, p95, maxDuration, slowCount, errorRate }
perfStore.logEntries       // 合并 invoke + navigation 记录
perfStore.commandDistribution  // 命令分组统计
perfStore.timelineBuckets     // 10秒粒度时间线数据
```

### 在 API 层外部使用

**文件**: `src/api/tauri.ts`

```typescript
import { usePerformanceStoreRaw } from '@/stores/usePerformanceStore'

const perfStore = usePerformanceStoreRaw()  // 单例，可在 Vue setup 外使用
if (perfStore.isMonitoring) {
  perfStore.recordInvoke({ command, duration, success, error, timestamp })
}
```

### 监测页面

**文件**: `src/views/PerformanceMonitor.vue`

**路由**: `/performance`

**功能**:
- 6 个统计卡片：总请求数、平均耗时、P95、最大耗时、慢请求数、错误率
- 命令耗时 Top 10 柱状图（颜色按耗时分级）
- 请求趋势折线图（双 Y 轴：请求数 + 平均耗时）
- 操作日志表格（支持搜索、筛选、排序、分页）

## 后端 (Rust)

### 性能收集器

**文件**: `src-tauri/src/services/perf_service.rs`

```rust
pub struct PerfCollector {
    pub enabled: Mutex<bool>,            // 默认 false
    pub records: Mutex<Vec<PerfRecord>>, // 上限 5000
    pub max_records: usize,
}

pub struct PerfRecord {
    pub command: String,
    pub duration_ms: f64,
    pub timestamp: String,    // ISO 8601
    pub success: bool,
    pub error: Option<String>,
    pub phases: Option<Vec<PerfPhase>>,  // 大操作子阶段
}
```

**方法**:
- `is_enabled()` - 检查是否启用
- `set_enabled(bool)` - 设置启用状态
- `record(PerfRecord)` - 记录一条性能数据
- `get_records(limit, filter)` - 获取记录
- `clear()` - 清空记录
- `summary()` - 获取统计摘要

### Tauri 命令

**文件**: `src-tauri/src/commands/perf_commands.rs`

| 命令 | 参数 | 返回值 |
|------|------|--------|
| `get_perf_records` | `limit: u32, command_filter: Option<String>` | `Vec<PerfRecord>` |
| `get_perf_summary` | - | `PerfSummary` |
| `toggle_perf_monitoring` | `enabled: bool` | `bool` |
| `clear_perf_records` | - | `bool` |

### AppState 集成

**文件**: `src-tauri/src/commands/save_commands.rs`

```rust
pub struct AppState {
    pub db: Arc<RwLock<Option<DatabaseManager>>>,
    pub current_save_id: Arc<RwLock<Option<String>>>,
    pub perf: PerfCollector,  // 性能收集器
}
```

## 添加后端命令计时

在需要监测的命令中添加：

```rust
use crate::services::perf_service::PerfRecord;

#[tauri::command]
pub async fn some_command(state: State<'_, AppState>) -> Result<...> {
    let start = std::time::Instant::now();
    let mut success = true;
    let mut error_msg = None;

    let result = // ... 执行逻辑 ...

    if state.perf.is_enabled() {
        state.perf.record(PerfRecord {
            command: "some_command".to_string(),
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            success,
            error: error_msg,
            phases: None,
        });
    }

    result
}
```

## 与日志系统的关系

| 系统 | 存储方式 | 用途 |
|------|---------|------|
| **日志系统** | 文件持久化 | 应用运行日志、错误追踪 |
| **性能监测** | 内存临时存储 | 实时性能分析、用户主动查看 |

如需将性能数据写入日志，在 `perf_service.rs` 的 `record()` 中添加：

```rust
tracing::debug!(command = %record.command, duration_ms = record.duration_ms, "性能记录");
```

## 文件位置

| 文件 | 说明 |
|------|------|
| `src/stores/usePerformanceStore.ts` | 前端性能数据 Store |
| `src/views/PerformanceMonitor.vue` | 性能监测页面 |
| `src/api/tauri.ts` | IPC 调用计时拦截 |
| `src/router/index.ts` | 路由导航计时 |
| `src-tauri/src/services/perf_service.rs` | 后端性能收集器 |
| `src-tauri/src/commands/perf_commands.rs` | 后端性能命令 |
