/**
 * 全局错误处理器
 *
 * 提供统一的错误处理和用户提示功能。
 */

import { ref } from 'vue'
import { createLogger } from '@/utils/logger'
import { getErrorInfo, isKnownError, type ErrorInfo } from './errorRegistry'

const logger = createLogger('ErrorHandler')

/**
 * 全局错误
 */
export interface GlobalError {
  code: string
  message: string
  details?: string
  canRetry: boolean
  retryFn?: () => Promise<unknown>
}

/**
 * 错误处理选项
 */
export interface HandleErrorOptions {
  /** 是否可以重试 */
  canRetry?: boolean
  /** 重试函数 */
  retryFn?: () => Promise<unknown>
  /** 静默处理，不显示弹窗 */
  silent?: boolean
  /** 错误发生的组件 */
  component?: string
  /** 用户正在进行的操作 */
  userAction?: string
}

// 全局错误状态
export const globalError = ref<GlobalError | null>(null)
export const showErrorDialog = ref(false)

/**
 * 处理错误
 *
 * @param error - 错误对象或字符串
 * @param options - 处理选项
 * @returns 格式化后的错误对象
 */
export function handleError(error: unknown, options?: HandleErrorOptions): GlobalError {
  let appError: GlobalError

  // 解析错误对象
  if (typeof error === 'object' && error !== null && 'code' in error) {
    // 结构化错误
    const e = error as { code: string; message: string; details?: string }
    appError = {
      code: e.code,
      message: e.message,
      details: e.details,
      canRetry: options?.canRetry ?? false,
      retryFn: options?.retryFn,
    }
  } else if (error instanceof Error) {
    // 标准 Error 对象
    appError = {
      code: 'E-SY-S-001',
      message: error.message,
      details: error.stack,
      canRetry: options?.canRetry ?? false,
      retryFn: options?.retryFn,
    }
  } else {
    // 其他类型
    appError = {
      code: 'E-SY-S-001',
      message: String(error),
      canRetry: options?.canRetry ?? false,
      retryFn: options?.retryFn,
    }
  }

  // 记录日志
  logger.error(`[${appError.code}] ${appError.message}`, {
    details: appError.details,
    component: options?.component,
    userAction: options?.userAction,
  })

  // 显示错误弹窗
  if (!options?.silent) {
    globalError.value = appError
    showErrorDialog.value = true
  }

  return appError
}

/**
 * 清除当前错误
 */
export function clearError() {
  globalError.value = null
  showErrorDialog.value = false
}

/**
 * 重试上次失败的操作
 */
export async function retryLastAction(): Promise<void> {
  if (globalError.value?.retryFn) {
    const retryFn = globalError.value.retryFn
    clearError()

    try {
      await retryFn()
    } catch (error) {
      handleError(error, {
        canRetry: true,
        retryFn,
      })
    }
  }
}

/**
 * 创建带错误处理的异步函数包装器
 */
export function withErrorHandling<T extends (...args: unknown[]) => Promise<unknown>>(
  fn: T,
  options?: Omit<HandleErrorOptions, 'retryFn'>
): T {
  return (async (...args: Parameters<T>) => {
    try {
      return await fn(...args)
    } catch (error) {
      handleError(error, {
        ...options,
        canRetry: options?.canRetry ?? true,
        retryFn: () => fn(...args) as Promise<void>,
      })
      throw error
    }
  }) as T
}

/**
 * 获取当前错误的详细信息
 */
export function getCurrentErrorInfo(): ErrorInfo | undefined {
  if (globalError.value?.code) {
    return getErrorInfo(globalError.value.code)
  }
  return undefined
}

/**
 * 检查当前错误是否为已知错误
 */
export function isCurrentErrorKnown(): boolean {
  if (globalError.value?.code) {
    return isKnownError(globalError.value.code)
  }
  return false
}
