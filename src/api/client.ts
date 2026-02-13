/**
 * Tauri IPC Client
 * Replaces axios HTTP calls with Tauri invoke commands
 */
import { invoke } from '@tauri-apps/api/core'
import { createLogger } from '@/utils/logger'
import { usePerformanceStoreRaw } from '@/stores/usePerformanceStore'

export const logger = createLogger('TauriAPI')

// Generic API response from Rust backend
export interface CommandResult<T> {
  success: boolean
  data: T | null
  error: string | null
}

// Helper to invoke Tauri commands with error handling
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const perfStore = usePerformanceStoreRaw()
  const startTime = perfStore.isMonitoring ? performance.now() : 0

  try {
    const result = await invoke<CommandResult<T>>(command, args)
    logger.debug('Tauri命令执行成功', { command, result: JSON.stringify(result) })

    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: result.success,
        error: result.success ? undefined : (result.error || undefined),
        timestamp: Date.now(),
      })
    }

    if (result.success) {
      return result.data as T
    }
    throw new Error(result.error || 'Unknown error')
  } catch (error) {
    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: false,
        error: error instanceof Error ? error.message : String(error),
        timestamp: Date.now(),
      })
    }
    logger.error('Tauri命令执行失败', { command, error })
    throw error
  }
}

// Helper that returns the full CommandResult (for cases where we need to check success)
export async function invokeCommandRaw<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<CommandResult<T>> {
  const perfStore = usePerformanceStoreRaw()
  const startTime = perfStore.isMonitoring ? performance.now() : 0

  try {
    const result = await invoke<CommandResult<T>>(command, args)

    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: result.success,
        error: result.success ? undefined : (result.error || undefined),
        timestamp: Date.now(),
      })
    }

    return result
  } catch (error) {
    if (perfStore.isMonitoring && startTime > 0) {
      const duration = Math.round(performance.now() - startTime)
      perfStore.recordInvoke({
        command,
        duration,
        success: false,
        error: error instanceof Error ? error.message : String(error),
        timestamp: Date.now(),
      })
    }
    logger.error('Tauri命令执行失败', { command, error })
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : String(error)
    }
  }
}