/**
 * Draft Store - 管理选秀系统
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  draftApi,
  type DraftPlayer,
  type DraftOrder,
  type DraftPick
} from '@/api/tauri'

export const useDraftStoreTauri = defineStore('draftTauri', () => {
  // ========================================
  // State
  // ========================================

  // 当前赛区ID
  const currentRegionId = ref<number | null>(null)

  // 选秀池选手
  const draftPool = ref<DraftPlayer[]>([])

  // 选秀顺序
  const draftOrder = ref<DraftOrder[]>([])

  // 已完成的选秀选择
  const draftPicks = ref<DraftPick[]>([])

  // 当前选秀轮次
  const currentPick = ref(1)

  // 加载状态
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 选秀是否已开始
  const isDraftStarted = ref(false)

  // ========================================
  // Computed
  // ========================================

  // 可用选手（未被选走的）
  const availablePlayers = computed(() => {
    const pickedPlayerIds = new Set(draftPicks.value.map(p => p.player.id))
    return draftPool.value.filter(player => !pickedPlayerIds.has(player.id))
  })

  // 按潜力排序的可用选手
  const playersByPotential = computed(() => {
    return [...availablePlayers.value].sort((a, b) => b.potential - a.potential)
  })

  // 按能力排序的可用选手
  const playersByAbility = computed(() => {
    return [...availablePlayers.value].sort((a, b) => b.ability - a.ability)
  })

  // 按位置分组的可用选手
  const playersByPosition = computed(() => {
    const grouped: Record<string, DraftPlayer[]> = {}
    availablePlayers.value.forEach(player => {
      if (!grouped[player.position]) {
        grouped[player.position] = []
      }
      grouped[player.position].push(player)
    })
    return grouped
  })

  // 当前选秀队伍
  const currentTeam = computed(() => {
    return draftOrder.value.find(o => o.pick_number === currentPick.value) ?? null
  })

  // 选秀是否完成
  const isDraftComplete = computed(() => {
    return currentPick.value > draftOrder.value.length
  })

  // 选秀进度
  const draftProgress = computed(() => ({
    current: currentPick.value,
    total: draftOrder.value.length,
    percentage: draftOrder.value.length > 0
      ? Math.round(((currentPick.value - 1) / draftOrder.value.length) * 100)
      : 0
  }))

  // ========================================
  // Actions
  // ========================================

  /**
   * 初始化选秀 - 生成选秀池
   */
  const generateDraftPool = async (regionId: number, poolSize = 30) => {
    isLoading.value = true
    error.value = null
    currentRegionId.value = regionId

    try {
      draftPool.value = await draftApi.generateDraftPool(regionId, poolSize)
      console.log(`Generated draft pool with ${draftPool.value.length} players`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to generate draft pool'
      console.error('Failed to generate draft pool:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 进行选秀抽签 - 确定选秀顺序
   */
  const runDraftLottery = async (regionId?: number) => {
    const targetRegion = regionId ?? currentRegionId.value
    if (!targetRegion) {
      throw new Error('No region selected')
    }

    isLoading.value = true
    error.value = null

    try {
      draftOrder.value = await draftApi.runDraftLottery(targetRegion)
      isDraftStarted.value = true
      currentPick.value = 1
      draftPicks.value = []
      console.log('Draft lottery completed, order:', draftOrder.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to run draft lottery'
      console.error('Failed to run draft lottery:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取选秀顺序
   */
  const loadDraftOrder = async (regionId?: number) => {
    const targetRegion = regionId ?? currentRegionId.value
    if (!targetRegion) {
      throw new Error('No region selected')
    }

    isLoading.value = true
    error.value = null

    try {
      draftOrder.value = await draftApi.getDraftOrder(targetRegion)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load draft order'
      console.error('Failed to load draft order:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取可用选手
   */
  const loadAvailablePlayers = async (regionId?: number) => {
    const targetRegion = regionId ?? currentRegionId.value
    if (!targetRegion) {
      throw new Error('No region selected')
    }

    isLoading.value = true
    error.value = null

    try {
      draftPool.value = await draftApi.getAvailableDraftPlayers(targetRegion)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load available players'
      console.error('Failed to load available players:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 进行选秀选择
   */
  const makePick = async (playerId: number) => {
    if (!currentRegionId.value || !currentTeam.value) {
      throw new Error('Draft not ready')
    }

    isLoading.value = true
    error.value = null

    try {
      const pick = await draftApi.makeDraftPick(
        currentRegionId.value,
        currentTeam.value.team_id,
        playerId
      )

      draftPicks.value.push(pick)
      currentPick.value++

      console.log(`Pick ${pick.pick_number}: ${pick.team_name} selected ${pick.player.name}`)
      return pick
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to make draft pick'
      console.error('Failed to make draft pick:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * AI自动完成选秀
   */
  const autoCompleteDraft = async (regionId?: number) => {
    const targetRegion = regionId ?? currentRegionId.value
    if (!targetRegion) {
      throw new Error('No region selected')
    }

    isLoading.value = true
    error.value = null

    try {
      const picks = await draftApi.aiAutoDraft(targetRegion)
      draftPicks.value = picks
      currentPick.value = picks.length + 1
      console.log(`AI completed draft with ${picks.length} picks`)
      return picks
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to auto complete draft'
      console.error('Failed to auto complete draft:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 重置选秀
   */
  const resetDraft = () => {
    draftPool.value = []
    draftOrder.value = []
    draftPicks.value = []
    currentPick.value = 1
    isDraftStarted.value = false
    error.value = null
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
    currentRegionId,
    draftPool,
    draftOrder,
    draftPicks,
    currentPick,
    isLoading,
    error,
    isDraftStarted,

    // Computed
    availablePlayers,
    playersByPotential,
    playersByAbility,
    playersByPosition,
    currentTeam,
    isDraftComplete,
    draftProgress,

    // Actions
    generateDraftPool,
    runDraftLottery,
    loadDraftOrder,
    loadAvailablePlayers,
    makePick,
    autoCompleteDraft,
    resetDraft,
    clearError,
  }
})
