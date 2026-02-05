---
name: logging-system
description: 电竞经理游戏的日志与错误处理系统。管理前后端日志记录、错误码系统、日志监控台、错误引导弹窗。当需要添加日志、调试问题、修改错误处理、查看日志文件或修改日志/错误相关功能时使用此技能。
---

# 日志与错误处理系统 (Logging System)

## Overview

日志系统提供前后端统一的日志记录、结构化错误码、实时日志监控和用户友好的错误引导功能。

## 后端 (Rust)

### 日志服务

**文件**: `src-tauri/src/services/logging_service.rs`

```rust
use tracing::{info, warn, error, debug};

// 初始化日志系统（在 lib.rs 中调用）
pub fn init_logging(app_handle: &tauri::AppHandle) -> Result<(), String>

// 清理旧日志文件
pub fn cleanup_old_logs(log_dir: &Path, days: u64) -> io::Result<()>
```

**日志文件**:
- `logs/app.log` - 主应用日志（滚动，按天）
- `logs/error.log` - 仅错误日志
- `logs/frontend.log` - 前端同步日志

### 日志命令

**文件**: `src-tauri/src/commands/log_commands.rs`

| 命令 | 功能 |
|------|------|
| `log_frontend_event` | 接收前端批量日志 |
| `log_frontend_error` | 接收前端错误 |
| `get_log_files` | 获取日志文件列表 |
| `read_log_file` | 读取日志文件内容 |
| `cleanup_logs` | 清理旧日志 |

### 错误码系统

**文件**: `src-tauri/src/errors/error_codes.rs`

**错误码格式**: `E-[模块]-[类型]-[序号]`

**模块代码**:
| 代码 | 模块 |
|------|------|
| TM | 时间系统 |
| MT | 比赛系统 |
| TR | 转会系统 |
| PL | 选手系统 |
| FN | 财政系统 |
| DR | 选秀系统 |
| HN | 荣誉系统 |
| DB | 数据库 |
| SY | 系统 |

**类型代码**:
| 代码 | 含义 |
|------|------|
| V | 验证错误 |
| B | 业务逻辑错误 |
| D | 数据错误 |
| S | 系统错误 |
| N | 网络错误 |

**使用示例**:
```rust
use crate::errors::{AppError, transfer};

// 创建错误
let error = transfer::budget_insufficient();
// 返回: E-TR-B-001 预算不足

// 带详情的错误
let error = AppError::new("E-TR-B-001", "预算不足")
    .with_details("当前余额: 100万, 需要: 500万")
    .with_suggestion("请先出售其他选手");
```

## 前端 (Vue/TypeScript)

### Logger 模块

**文件**: `src/utils/logger/`

```typescript
import { createLogger } from '@/utils/logger'

const logger = createLogger('MyStore')

// 日志方法
logger.debug('调试信息', { data })
logger.info('一般信息', { userId })
logger.warn('警告信息')
logger.error('错误信息', { error })

// 用户操作日志
logger.action('点击按钮', { buttonId: 'submit' })

// 带计时的操作
const result = await logger.timed('加载数据', async () => {
  return await api.loadData()
})
```

### 错误处理模块

**文件**: `src/utils/errors/`

```typescript
import { handleError } from '@/utils/errors'

try {
  await someOperation()
} catch (error) {
  handleError(error, {
    canRetry: true,
    retryFn: () => someOperation(),
    component: 'TransferStore',
    userAction: '执行转会',
  })
}
```

### 日志监控台

**文件**: `src/components/dev/LogMonitor.vue`

**快捷键**: `Ctrl+Shift+L` 切换显示（仅开发环境）

### 错误引导弹窗

**文件**: `src/components/common/ErrorGuide.vue`

显示错误码、描述、建议操作、重试按钮。

## 添加新错误码

### 后端

在 `src-tauri/src/errors/error_codes.rs` 添加模块错误函数。

### 前端

在 `src/utils/errors/errorRegistry.ts` 添加:

```typescript
'E-XX-B-001': {
  code: 'E-XX-B-001',
  title: '错误标题',
  description: '详细描述',
  suggestion: '建议操作',
  severity: 'medium',  // low/medium/high/critical
},
```

## 日志文件位置

- macOS: `~/Library/Application Support/com.esportmanager2.app/logs/`
- Windows: `%APPDATA%/com.esportmanager2.app/logs/`
