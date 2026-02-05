/**
 * API 日志包装器
 *
 * 为 Tauri invoke 调用提供自动日志记录功能。
 */

import { invoke } from '@tauri-apps/api/core'
import { createLogger } from './Logger'

const logger = createLogger('API')

type InvokeArgs = Record<string, unknown>

/**
 * 带日志的 invoke 调用
 */
export async function invokeWithLog<T>(command: string, args?: InvokeArgs): Promise<T> {
  const start = performance.now()

  logger.debug(`调用: ${command}`, args ? { args: summarizeArgs(args) } : undefined)

  try {
    const result = await invoke<T>(command, args)
    const duration = Math.round(performance.now() - start)

    logger.debug(`响应: ${command}`, {
      duration_ms: duration,
      success: true,
    })

    return result
  } catch (error) {
    const duration = Math.round(performance.now() - start)

    logger.error(`失败: ${command}`, {
      duration_ms: duration,
      error: String(error),
      args: args ? summarizeArgs(args) : undefined,
    })

    throw error
  }
}

/**
 * 简化大参数，避免日志过长
 */
function summarizeArgs(args: InvokeArgs): InvokeArgs {
  const result: InvokeArgs = {}

  for (const [key, value] of Object.entries(args)) {
    if (Array.isArray(value)) {
      result[key] = `Array(${value.length})`
    } else if (typeof value === 'object' && value !== null) {
      const keys = Object.keys(value)
      if (keys.length > 5) {
        result[key] = `Object{${keys.slice(0, 5).join(',')},...}`
      } else {
        result[key] = `Object{${keys.join(',')}}`
      }
    } else if (typeof value === 'string' && value.length > 100) {
      result[key] = `${value.slice(0, 100)}...`
    } else {
      result[key] = value
    }
  }

  return result
}

/**
 * 批量调用包装器
 */
export async function invokeAllWithLog<T>(
  calls: Array<{ command: string; args?: InvokeArgs }>
): Promise<T[]> {
  const start = performance.now()

  logger.debug(`批量调用: ${calls.length} 个命令`, {
    commands: calls.map((c) => c.command),
  })

  try {
    const results = await Promise.all(calls.map((c) => invoke<T>(c.command, c.args)))

    const duration = Math.round(performance.now() - start)
    logger.info(`批量调用完成`, {
      count: calls.length,
      duration_ms: duration,
    })

    return results
  } catch (error) {
    const duration = Math.round(performance.now() - start)
    logger.error(`批量调用失败`, {
      duration_ms: duration,
      error: String(error),
    })
    throw error
  }
}
