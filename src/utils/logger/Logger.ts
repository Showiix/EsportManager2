/**
 * 日志系统核心模块
 *
 * 提供统一的日志记录、控制台输出和后端同步功能。
 */

import { invoke } from '@tauri-apps/api/core'
import { LogLevel, LogLevelNames, LogLevelColors } from './LogLevel'

/**
 * 日志条目
 */
export interface LogEntry {
  level: string
  module: string
  message: string
  data?: unknown
  timestamp: string
  userAction?: string
}

/**
 * 日志订阅回调
 */
export type LogSubscriber = (entry: LogEntry) => void

/**
 * Logger 配置
 */
export interface LoggerConfig {
  minLevel: LogLevel
  enableConsole: boolean
  enableRemote: boolean
  batchSize: number
  flushInterval: number
}

/**
 * 日志服务 (单例)
 */
class LoggerService {
  private static instance: LoggerService

  private buffer: LogEntry[] = []
  private subscribers: Set<LogSubscriber> = new Set()
  private flushTimer: ReturnType<typeof setInterval> | null = null
  private config: LoggerConfig = {
    minLevel: LogLevel.DEBUG,
    enableConsole: true,
    enableRemote: true,
    batchSize: 10,
    flushInterval: 5000,
  }

  private constructor() {
    this.startFlushTimer()
    this.setupGlobalErrorHandlers()
  }

  static getInstance(): LoggerService {
    if (!LoggerService.instance) {
      LoggerService.instance = new LoggerService()
    }
    return LoggerService.instance
  }

  /**
   * 配置日志服务
   */
  configure(options: Partial<LoggerConfig>) {
    if (options.minLevel !== undefined) this.config.minLevel = options.minLevel
    if (options.enableConsole !== undefined) this.config.enableConsole = options.enableConsole
    if (options.enableRemote !== undefined) this.config.enableRemote = options.enableRemote
    if (options.batchSize !== undefined) this.config.batchSize = options.batchSize
    if (options.flushInterval !== undefined) {
      this.config.flushInterval = options.flushInterval
      this.startFlushTimer()
    }
  }

  /**
   * 创建模块 Logger
   */
  createLogger(module: string): Logger {
    return new Logger(module, this)
  }

  /**
   * 订阅日志事件 (用于日志监控台)
   */
  subscribe(callback: LogSubscriber): () => void {
    this.subscribers.add(callback)
    return () => this.subscribers.delete(callback)
  }

  /**
   * 记录日志
   */
  log(entry: LogEntry, level: LogLevel) {
    if (level < this.config.minLevel) return

    // 通知订阅者
    this.subscribers.forEach((cb) => {
      try {
        cb(entry)
      } catch (e) {
        console.error('[Logger] 订阅者回调出错:', e)
      }
    })

    // 控制台输出
    if (this.config.enableConsole) {
      this.logToConsole(entry, level)
    }

    // 添加到缓冲区
    if (this.config.enableRemote) {
      this.buffer.push(entry)
      if (this.buffer.length >= this.config.batchSize) {
        this.flush()
      }
    }
  }

  /**
   * 输出到控制台
   */
  private logToConsole(entry: LogEntry, level: LogLevel) {
    const color = LogLevelColors[level]
    const levelName = LogLevelNames[level]
    const timestamp = new Date().toLocaleTimeString()

    const prefix = `%c[${timestamp}] [${levelName}] [${entry.module}]`
    const style = `color: ${color}; font-weight: bold;`

    const consoleMethod =
      level === LogLevel.ERROR
        ? console.error
        : level === LogLevel.WARN
          ? console.warn
          : level === LogLevel.DEBUG
            ? console.debug
            : console.log

    if (entry.data !== undefined) {
      consoleMethod(prefix, style, entry.message, entry.data)
    } else {
      consoleMethod(prefix, style, entry.message)
    }
  }

  /**
   * 刷新缓冲区到后端
   */
  async flush() {
    if (this.buffer.length === 0) return

    const entries = [...this.buffer]
    this.buffer = []

    try {
      await invoke('log_frontend_event', { entries })
    } catch (error) {
      // 失败时放回缓冲区（只保留最近的100条）
      this.buffer = [...entries, ...this.buffer].slice(-100)
      console.warn('[Logger] 发送日志到后端失败:', error)
    }
  }

  /**
   * 记录前端错误到后端
   */
  async logError(
    component: string,
    message: string,
    stack?: string,
    userAction?: string,
    errorCode?: string
  ) {
    try {
      await invoke('log_frontend_error', {
        errorMessage: message,
        stackTrace: stack,
        component,
        userAction,
        errorCode,
      })
    } catch (error) {
      console.error('[Logger] 发送错误日志失败:', error)
    }
  }

  /**
   * 启动定时刷新
   */
  private startFlushTimer() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer)
    }
    this.flushTimer = setInterval(() => this.flush(), this.config.flushInterval)
  }

  /**
   * 设置全局错误处理
   */
  private setupGlobalErrorHandlers() {
    if (typeof window === 'undefined') return

    window.addEventListener('error', (event) => {
      this.logError('GlobalError', event.message, event.error?.stack)
    })

    window.addEventListener('unhandledrejection', (event) => {
      this.logError('UnhandledRejection', String(event.reason))
    })
  }

  /**
   * 获取当前配置
   */
  getConfig(): LoggerConfig {
    return { ...this.config }
  }

  /**
   * 销毁服务
   */
  destroy() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer)
      this.flushTimer = null
    }
    this.flush()
    this.subscribers.clear()
  }
}

/**
 * 模块 Logger
 */
export class Logger {
  constructor(
    private module: string,
    private service: LoggerService
  ) {}

  private createEntry(message: string, data?: unknown, userAction?: string): LogEntry {
    return {
      level: '',
      module: this.module,
      message,
      data,
      timestamp: new Date().toISOString(),
      userAction,
    }
  }

  trace(message: string, data?: unknown) {
    const entry = this.createEntry(message, data)
    entry.level = 'TRACE'
    this.service.log(entry, LogLevel.TRACE)
  }

  debug(message: string, data?: unknown) {
    const entry = this.createEntry(message, data)
    entry.level = 'DEBUG'
    this.service.log(entry, LogLevel.DEBUG)
  }

  info(message: string, data?: unknown) {
    const entry = this.createEntry(message, data)
    entry.level = 'INFO'
    this.service.log(entry, LogLevel.INFO)
  }

  warn(message: string, data?: unknown) {
    const entry = this.createEntry(message, data)
    entry.level = 'WARN'
    this.service.log(entry, LogLevel.WARN)
  }

  error(message: string, data?: unknown) {
    const entry = this.createEntry(message, data)
    entry.level = 'ERROR'
    this.service.log(entry, LogLevel.ERROR)
  }

  /**
   * 记录用户操作
   */
  action(actionName: string, details?: unknown) {
    const entry = this.createEntry(`用户操作: ${actionName}`, details, actionName)
    entry.level = 'INFO'
    this.service.log(entry, LogLevel.INFO)
  }

  /**
   * 带计时的异步操作
   */
  async timed<T>(operationName: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now()
    this.debug(`开始: ${operationName}`)

    try {
      const result = await fn()
      const duration = Math.round(performance.now() - start)
      this.info(`完成: ${operationName}`, { duration_ms: duration })
      return result
    } catch (error) {
      const duration = Math.round(performance.now() - start)
      this.error(`失败: ${operationName}`, {
        duration_ms: duration,
        error: String(error),
      })
      throw error
    }
  }

  /**
   * 计时同步操作
   */
  timedSync<T>(operationName: string, fn: () => T): T {
    const start = performance.now()
    this.debug(`开始: ${operationName}`)

    try {
      const result = fn()
      const duration = Math.round(performance.now() - start)
      this.info(`完成: ${operationName}`, { duration_ms: duration })
      return result
    } catch (error) {
      const duration = Math.round(performance.now() - start)
      this.error(`失败: ${operationName}`, {
        duration_ms: duration,
        error: String(error),
      })
      throw error
    }
  }
}

// 导出单例
export const loggerService = LoggerService.getInstance()

// 快捷创建 Logger
export function createLogger(module: string): Logger {
  return loggerService.createLogger(module)
}
