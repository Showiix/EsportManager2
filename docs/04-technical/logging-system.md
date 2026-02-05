# 日志系统

## 概述

EsportManager 2 使用双重日志系统：
- **后端**: Rust `tracing` 生态系统
- **前端**: 自定义 Logger 类

## 后端日志 (Rust)

### 技术栈

| 组件 | 用途 |
|------|------|
| `tracing` | 日志记录框架 |
| `tracing-subscriber` | 日志订阅器 |
| `tracing-appender` | 日志文件写入 |

### 配置选项

```rust
// src-tauri/src/services/logging_service.rs

pub struct LoggingConfig {
    /// 日志目录
    pub log_dir: PathBuf,
    /// 最大日志级别
    pub max_level: Level,
    /// 是否启用控制台输出
    pub enable_console: bool,
    /// 是否启用文件输出
    pub enable_file: bool,
    /// 是否使用 JSON 格式
    pub enable_json: bool,
    /// 应用名称
    pub app_name: String,
}
```

### 预设配置

```rust
impl LoggingConfig {
    /// 开发模式配置
    pub fn development() -> Self {
        Self {
            max_level: Level::DEBUG,
            enable_console: true,
            enable_file: true,
            enable_json: false, // 开发时更易读
            ..Default::default()
        }
    }

    /// 生产模式配置
    pub fn production() -> Self {
        Self {
            max_level: Level::INFO,
            enable_console: false,
            enable_file: true,
            enable_json: true,
            ..Default::default()
        }
    }
}
```

### 日志级别

| 级别 | 用途 |
|------|------|
| `TRACE` | 最详细的追踪信息 |
| `DEBUG` | 调试信息 |
| `INFO` | 一般信息 |
| `WARN` | 警告信息 |
| `ERROR` | 错误信息 |

### 使用示例

```rust
use tracing::{info, warn, error, debug, instrument};

// 基本日志
info!("游戏启动");
warn!("资金不足: {}", balance);
error!("数据库连接失败: {}", e);
debug!("计算战力: team_id={}, power={}", team_id, power);

// 带字段的结构化日志
info!(
    save_id = %save_id,
    season = current_season,
    phase = ?current_phase,
    "阶段推进完成"
);

// 函数追踪
#[instrument(skip(pool))]
async fn simulate_match(pool: &SqlitePool, match_id: i64) -> Result<MatchResult> {
    // 自动记录函数调用
    info!("开始模拟比赛");
    // ...
}
```

### 日志文件

日志文件按日期滚动存储：

```
logs/
├── esport_manager_2.2024-01-15.log
├── esport_manager_2.2024-01-16.log
└── esport_manager_2.2024-01-17.log
```

### JSON 格式日志

生产环境使用 JSON 格式，便于日志分析：

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "target": "esport_manager_2::engines::match_simulation",
  "message": "比赛模拟完成",
  "match_id": 123,
  "winner_id": 45
}
```

## 前端日志 (TypeScript)

### Logger 类

```typescript
// src/utils/logger/Logger.ts
import { LogLevel } from './LogLevel';

export class Logger {
  private static level: LogLevel = LogLevel.INFO;

  static setLevel(level: LogLevel) {
    this.level = level;
  }

  static debug(message: string, ...args: any[]) {
    if (this.level <= LogLevel.DEBUG) {
      console.debug(`[DEBUG] ${message}`, ...args);
    }
  }

  static info(message: string, ...args: any[]) {
    if (this.level <= LogLevel.INFO) {
      console.info(`[INFO] ${message}`, ...args);
    }
  }

  static warn(message: string, ...args: any[]) {
    if (this.level <= LogLevel.WARN) {
      console.warn(`[WARN] ${message}`, ...args);
    }
  }

  static error(message: string, ...args: any[]) {
    if (this.level <= LogLevel.ERROR) {
      console.error(`[ERROR] ${message}`, ...args);
    }
  }
}
```

### 日志级别枚举

```typescript
// src/utils/logger/LogLevel.ts
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}
```

### API 日志

```typescript
// src/utils/logger/apiLogger.ts
import { Logger } from './Logger';

export function logApiCall(command: string, params: any) {
  Logger.debug(`API Call: ${command}`, params);
}

export function logApiResponse(command: string, response: any) {
  Logger.debug(`API Response: ${command}`, response);
}

export function logApiError(command: string, error: any) {
  Logger.error(`API Error: ${command}`, error);
}
```

### 使用示例

```typescript
import { Logger } from '@/utils/logger';

// 基本日志
Logger.info('页面加载完成');
Logger.warn('数据格式不正确', data);
Logger.error('请求失败', error);
Logger.debug('调试信息', { id, value });

// 设置日志级别
Logger.setLevel(LogLevel.DEBUG);  // 开发环境
Logger.setLevel(LogLevel.WARN);   // 生产环境
```

## 日志命令 (Tauri)

### 获取日志

```typescript
// 获取后端日志
const logs = await invoke('get_logs', { level: 'INFO', limit: 100 });
```

### 清空日志

```typescript
// 清空日志文件
await invoke('clear_logs');
```

## 最佳实践

### 1. 日志内容规范

```rust
// 好的日志
info!(
    team_id = team_id,
    player_id = player_id,
    transfer_fee = fee,
    "转会完成"
);

// 避免的日志
info!("转会完成 {} {} {}", team_id, player_id, fee);
```

### 2. 敏感信息处理

```rust
// 不要记录敏感信息
// 错误
info!("用户登录: password={}", password);

// 正确
info!(user_id = user_id, "用户登录成功");
```

### 3. 日志级别选择

| 场景 | 级别 |
|------|------|
| 函数调用追踪 | DEBUG |
| 业务流程节点 | INFO |
| 可恢复的问题 | WARN |
| 需要关注的错误 | ERROR |

### 4. 性能考虑

```rust
// 使用懒求值避免不必要的计算
debug!("详细数据: {:?}", expensive_debug_info());

// 更好的方式
if tracing::enabled!(tracing::Level::DEBUG) {
    debug!("详细数据: {:?}", expensive_debug_info());
}
```

## 文件位置

| 文件 | 说明 |
|-----|------|
| `src-tauri/src/services/logging_service.rs` | 后端日志服务 |
| `src-tauri/src/commands/log_commands.rs` | 日志命令接口 |
| `src/utils/logger/Logger.ts` | 前端日志类 |
| `src/utils/logger/LogLevel.ts` | 日志级别枚举 |
| `src/utils/logger/apiLogger.ts` | API 日志工具 |
