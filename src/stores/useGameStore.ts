/**
 * Game Store - 管理游戏存档和状态
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { saveApi, type SaveInfo, type GameState } from '@/api/tauri'
import { usePlayerStore } from './usePlayerStore'
import { useMatchDetailStore } from './useMatchDetailStore'
import { useTransferWindowStore } from './useTransferWindowStore'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('GameStore')

export const useGameStore = defineStore('game', () => {
  // ========================================
  // State
  // ========================================

  // 所有存档列表
  const saves = ref<SaveInfo[]>([])

  // 当前加载的存档
  const currentSave = ref<SaveInfo | null>(null)

  // 当前游戏状态
  const gameState = ref<GameState | null>(null)

  // 加载状态
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 数据库是否已初始化
  const isInitialized = ref(false)

  // ========================================
  // Computed
  // ========================================

  // 是否有存档加载
  const hasSaveLoaded = computed(() => currentSave.value !== null)

  // 当前赛季
  const currentSeason = computed(() => gameState.value?.current_season ?? 1)

  // 当前阶段
  const currentPhase = computed(() => gameState.value?.current_phase ?? 'SpringRegular')

  // 阶段显示名称
  const currentPhaseDisplay = computed(() => gameState.value?.phase_name ?? '春季赛常规赛')

  // ========================================
  // Actions
  // ========================================

  /**
   * 初始化数据库
   */
  const initDatabase = async () => {
    if (isInitialized.value) return

    isLoading.value = true
    error.value = null

    try {
      await logger.timed('初始化数据库', () => saveApi.initDatabase())
      isInitialized.value = true
      logger.info('数据库初始化成功')
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to initialize database'
      handleError(e, {
        component: 'GameStore',
        userAction: '初始化数据库',
        canRetry: true,
        retryFn: initDatabase
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除数据库（开发调试用）
   */
  const deleteDatabase = async () => {
    isLoading.value = true
    error.value = null

    try {
      logger.warn('删除数据库')
      await saveApi.deleteDatabase()
      // 清除所有状态
      isInitialized.value = false
      saves.value = []
      currentSave.value = null
      gameState.value = null
      logger.info('数据库已删除')
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete database'
      handleError(e, {
        component: 'GameStore',
        userAction: '删除数据库'
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载所有存档列表
   */
  const loadSaves = async () => {
    isLoading.value = true
    error.value = null

    try {
      saves.value = await saveApi.getSaves()
      // 如果能成功加载存档列表，说明数据库已初始化
      isInitialized.value = true
      logger.debug('存档列表已加载', { count: saves.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load saves'
      handleError(e, {
        component: 'GameStore',
        userAction: '加载存档列表',
        silent: true
      })
      // 保持 isInitialized 为 false，表示数据库未初始化
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 创建新存档
   */
  const createSave = async (name: string) => {
    isLoading.value = true
    error.value = null

    try {
      logger.info('创建新存档', { name })
      const saveInfo = await logger.timed('创建存档', () => saveApi.createSave(name))
      logger.info('存档创建成功', { saveId: saveInfo.id, name })

      // 重新加载存档列表
      await loadSaves()

      // 自动加载新创建的存档
      await loadSave(saveInfo.id)

      return saveInfo.id
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create save'
      handleError(e, {
        component: 'GameStore',
        userAction: '创建存档',
        canRetry: true,
        retryFn: () => createSave(name)
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载存档
   */
  const loadSave = async (saveId: string) => {
    isLoading.value = true
    error.value = null

    try {
      logger.info('加载存档', { saveId })
      // 清除旧存档的缓存数据
      const playerStore = usePlayerStore()
      const matchDetailStore = useMatchDetailStore()
      const transferWindowStore = useTransferWindowStore()
      playerStore.clearAll()
      matchDetailStore.clearAll()
      transferWindowStore.clearState()
      logger.debug('已清除旧存档缓存')

      currentSave.value = await logger.timed('加载存档', () => saveApi.loadSave(saveId))
      logger.info('存档加载成功', { name: currentSave.value.name })

      // 获取游戏状态
      await refreshGameState()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load save'
      handleError(e, {
        component: 'GameStore',
        userAction: '加载存档',
        canRetry: true,
        retryFn: () => loadSave(saveId)
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除存档
   */
  const deleteSave = async (saveId: string) => {
    isLoading.value = true
    error.value = null

    try {
      logger.info('删除存档', { saveId })
      await saveApi.deleteSave(saveId)
      logger.info('存档已删除', { saveId })

      // 如果删除的是当前存档，清除状态
      if (currentSave.value?.id === saveId) {
        currentSave.value = null
        gameState.value = null
      }

      // 重新加载存档列表
      await loadSaves()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete save'
      handleError(e, {
        component: 'GameStore',
        userAction: '删除存档'
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 刷新游戏状态
   */
  const refreshGameState = async () => {
    if (!currentSave.value) return

    try {
      gameState.value = await saveApi.getGameState()
      logger.debug('游戏状态已刷新', {
        season: gameState.value?.current_season,
        phase: gameState.value?.current_phase
      })
    } catch (e) {
      logger.error('刷新游戏状态失败', { error: e })
      // 不抛出错误，允许继续
    }
  }

  /**
   * 推进游戏阶段
   */
  const advancePhase = async () => {
    if (!currentSave.value) {
      throw new Error('No save loaded')
    }

    isLoading.value = true
    error.value = null

    try {
      logger.info('推进游戏阶段')
      gameState.value = await saveApi.advancePhase()
      logger.info('阶段推进成功', { phase: gameState.value.phase_name })
      return gameState.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to advance phase'
      handleError(e, {
        component: 'GameStore',
        userAction: '推进阶段',
        canRetry: true,
        retryFn: advancePhase
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 检查是否有已保存的存档ID
   */
  const checkCurrentSave = async () => {
    try {
      const saveId = await saveApi.getCurrentSaveId()
      if (saveId) {
        logger.debug('发现已保存的存档', { saveId })
        await loadSave(saveId)
        return true
      }
      return false
    } catch (e) {
      logger.error('检查当前存档失败', { error: e })
      return false
    }
  }

  /**
   * 清除错误
   */
  const clearError = () => {
    error.value = null
  }

  // ========================================
  // Return
  // ========================================

  return {
    // State
    saves,
    currentSave,
    gameState,
    isLoading,
    error,
    isInitialized,

    // Computed
    hasSaveLoaded,
    currentSeason,
    currentPhase,
    currentPhaseDisplay,

    // Actions
    initDatabase,
    deleteDatabase,
    loadSaves,
    createSave,
    loadSave,
    deleteSave,
    refreshGameState,
    advancePhase,
    checkCurrentSave,
    clearError,
  }
})
