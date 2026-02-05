/**
 * 日志系统模块
 *
 * @example
 * ```ts
 * import { createLogger, loggerService, LogLevel } from '@/utils/logger'
 *
 * // 配置日志服务
 * loggerService.configure({
 *   minLevel: LogLevel.DEBUG,
 *   enableConsole: true,
 *   enableRemote: true,
 * })
 *
 * // 创建模块 Logger
 * const logger = createLogger('MyStore')
 *
 * // 使用 Logger
 * logger.info('操作成功', { data: result })
 * logger.error('操作失败', { error: e })
 * logger.action('点击按钮', { buttonId: 'submit' })
 *
 * // 带计时的操作
 * const result = await logger.timed('加载数据', async () => {
 *   return await api.loadData()
 * })
 * ```
 */

export { LogLevel, LogLevelNames, LogLevelColors, parseLogLevel } from './LogLevel'

export { Logger, loggerService, createLogger, type LogEntry, type LogSubscriber, type LoggerConfig } from './Logger'

export { invokeWithLog, invokeAllWithLog } from './apiLogger'
