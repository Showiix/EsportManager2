/**
 * Game Store - 管理游戏存档和状态
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { saveApi, type SaveInfo, type GameState } from '@/api/tauri'

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
  const currentPhaseDisplay = computed(() => gameState.value?.phase_display ?? '春季赛常规赛')

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
      await saveApi.initDatabase()
      isInitialized.value = true
      console.log('Database initialized successfully')
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to initialize database'
      console.error('Failed to initialize database:', e)
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
      console.log(`Loaded ${saves.value.length} saves`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load saves'
      console.error('Failed to load saves:', e)
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
      const saveInfo = await saveApi.createSave(name)
      console.log(`Created save: ${saveInfo.id}`)

      // 重新加载存档列表
      await loadSaves()

      // 自动加载新创建的存档
      await loadSave(saveInfo.id)

      return saveInfo.id
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create save'
      console.error('Failed to create save:', e)
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
      currentSave.value = await saveApi.loadSave(saveId)
      console.log(`Loaded save: ${currentSave.value.name}`)

      // 获取游戏状态
      await refreshGameState()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load save'
      console.error('Failed to load save:', e)
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
      await saveApi.deleteSave(saveId)
      console.log(`Deleted save: ${saveId}`)

      // 如果删除的是当前存档，清除状态
      if (currentSave.value?.id === saveId) {
        currentSave.value = null
        gameState.value = null
      }

      // 重新加载存档列表
      await loadSaves()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete save'
      console.error('Failed to delete save:', e)
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
    } catch (e) {
      console.error('Failed to refresh game state:', e)
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
      gameState.value = await saveApi.advancePhase()
      console.log(`Advanced to phase: ${gameState.value.phase_display}`)
      return gameState.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to advance phase'
      console.error('Failed to advance phase:', e)
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
        await loadSave(saveId)
        return true
      }
      return false
    } catch (e) {
      console.error('Failed to check current save:', e)
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
