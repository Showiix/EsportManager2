/**
 * 错误处理模块
 *
 * @example
 * ```ts
 * import { handleError, clearError, globalError, showErrorDialog } from '@/utils/errors'
 *
 * // 处理错误
 * try {
 *   await someOperation()
 * } catch (error) {
 *   handleError(error, {
 *     canRetry: true,
 *     retryFn: () => someOperation(),
 *     component: 'TransferStore',
 *     userAction: '执行转会',
 *   })
 * }
 *
 * // 在组件中使用
 * const errorInfo = getCurrentErrorInfo()
 * ```
 */

export {
  ERROR_REGISTRY,
  getErrorInfo,
  isKnownError,
  getSeverityColor,
  type ErrorInfo,
  type ErrorSeverity,
} from './errorRegistry'

export {
  globalError,
  showErrorDialog,
  handleError,
  clearError,
  retryLastAction,
  withErrorHandling,
  getCurrentErrorInfo,
  isCurrentErrorKnown,
  type GlobalError,
  type HandleErrorOptions,
} from './globalErrorHandler'
