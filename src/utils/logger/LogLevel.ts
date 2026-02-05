/**
 * 日志级别定义
 */
export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
}

export const LogLevelNames: Record<LogLevel, string> = {
  [LogLevel.TRACE]: 'TRACE',
  [LogLevel.DEBUG]: 'DEBUG',
  [LogLevel.INFO]: 'INFO',
  [LogLevel.WARN]: 'WARN',
  [LogLevel.ERROR]: 'ERROR',
}

export const LogLevelColors: Record<LogLevel, string> = {
  [LogLevel.TRACE]: '#888888',
  [LogLevel.DEBUG]: '#00BFFF',
  [LogLevel.INFO]: '#00FF00',
  [LogLevel.WARN]: '#FFA500',
  [LogLevel.ERROR]: '#FF4444',
}

/**
 * 根据字符串获取日志级别
 */
export function parseLogLevel(level: string): LogLevel {
  switch (level.toUpperCase()) {
    case 'TRACE':
      return LogLevel.TRACE
    case 'DEBUG':
      return LogLevel.DEBUG
    case 'INFO':
      return LogLevel.INFO
    case 'WARN':
    case 'WARNING':
      return LogLevel.WARN
    case 'ERROR':
      return LogLevel.ERROR
    default:
      return LogLevel.INFO
  }
}
