/**
 * Transfer Store - 管理转会市场
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  transferApi,
  eventApi,
  type TransferListing,
  type FreeAgent,
  type TransferRecord,
  type TransferWindowInfo,
  type TransferEventInfo,
  type TransferRoundInfo,
  type ExpiringContract,
  type RetiringPlayer,
} from '@/api/tauri'
import { useGameStore } from './useGameStore'

export const useTransferStoreTauri = defineStore('transferTauri', () => {
  // ========================================
  // State
  // ========================================

  // 转会市场挂牌列表
  const listings = ref<TransferListing[]>([])

  // 自由球员列表
  const freeAgents = ref<FreeAgent[]>([])

  // 转会历史
  const transferHistory = ref<TransferRecord[]>([])

  // AI 转会窗口状态
  const transferWindow = ref<TransferWindowInfo | null>(null)

  // 当前轮次事件
  const currentRoundEvents = ref<TransferEventInfo[]>([])

  // 所有转会事件
  const allTransferEvents = ref<TransferEventInfo[]>([])

  // 合同到期选手
  const expiringContracts = ref<ExpiringContract[]>([])

  // 潜在退役选手
  const retiringCandidates = ref<RetiringPlayer[]>([])

  // 加载状态
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 筛选条件
  const filters = ref({
    position: '' as string,
    minAbility: 0,
    maxPrice: 0,
  })

  // ========================================
  // Computed
  // ========================================

  // 转会窗口是否已开始
  const isWindowStarted = computed(() => {
    return transferWindow.value !== null && transferWindow.value.status !== 'PREPARING'
  })

  // 转会窗口是否已完成
  const isWindowCompleted = computed(() => {
    return transferWindow.value?.status === 'COMPLETED'
  })

  // 当前轮次
  const currentRound = computed(() => transferWindow.value?.current_round ?? 0)

  // 重点关注选手（能力≥80的到期/挂牌选手）
  const highlightPlayers = computed(() => {
    const expiring = expiringContracts.value.filter(p => p.ability >= 80)
    const listed = listings.value.filter(p => p.ability >= 80)
    return { expiring, listed }
  })

  // 市场统计
  const marketStats = computed(() => ({
    totalListings: listings.value.length,
    totalFreeAgents: freeAgents.value.length,
    totalExpiring: expiringContracts.value.length,
    totalRetiring: retiringCandidates.value.length,
    avgListingPrice: listings.value.length > 0
      ? Math.round(listings.value.reduce((sum, l) => sum + l.asking_price, 0) / listings.value.length)
      : 0,
    avgFreeAgentSalary: freeAgents.value.length > 0
      ? Math.round(freeAgents.value.reduce((sum, a) => sum + a.expected_salary, 0) / freeAgents.value.length)
      : 0,
  }))

  // 过滤后的挂牌列表
  const filteredListings = computed(() => {
    return listings.value.filter(listing => {
      if (filters.value.position && listing.position !== filters.value.position) {
        return false
      }
      if (filters.value.minAbility && listing.ability < filters.value.minAbility) {
        return false
      }
      if (filters.value.maxPrice && listing.asking_price > filters.value.maxPrice) {
        return false
      }
      return true
    })
  })

  // 按能力排序的挂牌
  const listingsByAbility = computed(() => {
    return [...filteredListings.value].sort((a, b) => b.ability - a.ability)
  })

  // 按价格排序的挂牌
  const listingsByPrice = computed(() => {
    return [...filteredListings.value].sort((a, b) => a.asking_price - b.asking_price)
  })

  // 过滤后的自由球员
  const filteredFreeAgents = computed(() => {
    return freeAgents.value.filter(agent => {
      if (filters.value.position && agent.position !== filters.value.position) {
        return false
      }
      if (filters.value.minAbility && agent.ability < filters.value.minAbility) {
        return false
      }
      return true
    })
  })

  // 按位置分组的自由球员
  const freeAgentsByPosition = computed(() => {
    const grouped: Record<string, FreeAgent[]> = {}
    freeAgents.value.forEach(agent => {
      if (!grouped[agent.position]) {
        grouped[agent.position] = []
      }
      grouped[agent.position].push(agent)
    })
    return grouped
  })

  // ========================================
  // Actions - 预览数据加载
  // ========================================

  /**
   * 加载转会预览数据（合同到期、潜在退役、挂牌）
   */
  const loadPreviewData = async () => {
    isLoading.value = true
    error.value = null

    try {
      const gameStore = useGameStore()
      const currentSeason = gameStore.currentSeason

      const [expiring, retiring, market] = await Promise.all([
        eventApi.getExpiringContracts(currentSeason),
        eventApi.getRetiringCandidates(),
        transferApi.getTransferMarket(),
      ])

      expiringContracts.value = expiring
      retiringCandidates.value = retiring
      listings.value = market

      console.log(`Loaded preview: ${expiring.length} expiring, ${retiring.length} retiring, ${market.length} listings`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load preview data'
      console.error('Failed to load preview data:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // ========================================
  // Actions - AI 转会窗口
  // ========================================

  /**
   * 开始转会窗口
   */
  const startTransferWindow = async () => {
    isLoading.value = true
    error.value = null

    try {
      transferWindow.value = await transferApi.startTransferWindow()
      allTransferEvents.value = []
      currentRoundEvents.value = []
      console.log('Transfer window started:', transferWindow.value)
      return transferWindow.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to start transfer window'
      console.error('Failed to start transfer window:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 执行下一轮转会
   */
  const executeNextRound = async (): Promise<TransferRoundInfo> => {
    isLoading.value = true
    error.value = null

    try {
      const roundInfo = await transferApi.executeTransferRound()
      currentRoundEvents.value = roundInfo.events
      allTransferEvents.value.push(...roundInfo.events)

      // 更新窗口状态
      transferWindow.value = await transferApi.getTransferWindowStatus()

      console.log(`Round ${roundInfo.round} completed:`, roundInfo.summary)
      return roundInfo
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to execute round'
      console.error('Failed to execute round:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 快进完成所有转会
   */
  const fastForwardAll = async () => {
    isLoading.value = true
    error.value = null

    try {
      transferWindow.value = await transferApi.fastForwardTransfers()
      allTransferEvents.value = await transferApi.getTransferEvents()
      console.log('Transfer window completed:', transferWindow.value)
      return transferWindow.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fast forward'
      console.error('Failed to fast forward:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取转会窗口状态
   */
  const getWindowStatus = async () => {
    try {
      transferWindow.value = await transferApi.getTransferWindowStatus()
      return transferWindow.value
    } catch (e) {
      // 没有转会窗口时可能返回错误，这是正常的
      transferWindow.value = null
      return null
    }
  }

  /**
   * 获取指定轮次的事件
   */
  const getRoundEvents = async (round: number) => {
    try {
      return await transferApi.getTransferEvents(round)
    } catch (e) {
      console.error('Failed to get round events:', e)
      return []
    }
  }

  // ========================================
  // Actions - 原有功能
  // ========================================

  /**
   * 加载转会市场
   */
  const loadTransferMarket = async () => {
    isLoading.value = true
    error.value = null

    try {
      listings.value = await transferApi.getTransferMarket()
      console.log(`Loaded ${listings.value.length} transfer listings`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load transfer market'
      console.error('Failed to load transfer market:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载自由球员
   */
  const loadFreeAgents = async () => {
    isLoading.value = true
    error.value = null

    try {
      freeAgents.value = await transferApi.getFreeAgents()
      console.log(`Loaded ${freeAgents.value.length} free agents`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load free agents'
      console.error('Failed to load free agents:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载所有数据
   */
  const loadAll = async () => {
    await Promise.all([
      loadTransferMarket(),
      loadFreeAgents()
    ])
  }

  /**
   * 挂牌出售选手
   */
  const listPlayer = async (teamId: number, playerId: number, askingPrice: number) => {
    isLoading.value = true
    error.value = null

    try {
      const listing = await transferApi.listPlayerForTransfer(teamId, playerId, askingPrice)
      listings.value.push(listing)
      console.log(`Listed player ${listing.player_name} for ${askingPrice}`)
      return listing
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to list player'
      console.error('Failed to list player:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 取消挂牌
   */
  const cancelListing = async (listingId: number) => {
    isLoading.value = true
    error.value = null

    try {
      await transferApi.cancelTransferListing(listingId)
      listings.value = listings.value.filter(l => l.id !== listingId)
      console.log(`Cancelled listing ${listingId}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to cancel listing'
      console.error('Failed to cancel listing:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 购买挂牌选手
   */
  const buyPlayer = async (
    listingId: number,
    buyerTeamId: number,
    contractYears: number,
    salary: number
  ) => {
    isLoading.value = true
    error.value = null

    try {
      const record = await transferApi.buyListedPlayer(
        listingId,
        buyerTeamId,
        contractYears,
        salary
      )

      // 从列表中移除
      listings.value = listings.value.filter(l => l.id !== listingId)

      // 添加到历史
      transferHistory.value.unshift(record)

      console.log(`Bought player ${record.player_name} for ${record.fee}`)
      return record
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to buy player'
      console.error('Failed to buy player:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 签约自由球员
   */
  const signFreeAgent = async (
    playerId: number,
    teamId: number,
    contractYears: number,
    salary: number
  ) => {
    isLoading.value = true
    error.value = null

    try {
      const record = await transferApi.signFreeAgent(
        playerId,
        teamId,
        contractYears,
        salary
      )

      // 从自由球员列表移除
      freeAgents.value = freeAgents.value.filter(a => a.id !== playerId)

      // 添加到历史
      transferHistory.value.unshift(record)

      console.log(`Signed free agent ${record.player_name}`)
      return record
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to sign free agent'
      console.error('Failed to sign free agent:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载转会历史
   */
  const loadTransferHistory = async (teamId?: number) => {
    isLoading.value = true
    error.value = null

    try {
      transferHistory.value = await transferApi.getTransferHistory(teamId)
      console.log(`Loaded ${transferHistory.value.length} transfer records`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load transfer history'
      console.error('Failed to load transfer history:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 更新筛选条件
   */
  const updateFilters = (newFilters: Partial<typeof filters.value>) => {
    filters.value = { ...filters.value, ...newFilters }
  }

  /**
   * 清除筛选条件
   */
  const clearFilters = () => {
    filters.value = {
      position: '',
      minAbility: 0,
      maxPrice: 0,
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
    listings,
    freeAgents,
    transferHistory,
    transferWindow,
    currentRoundEvents,
    allTransferEvents,
    expiringContracts,
    retiringCandidates,
    isLoading,
    error,
    filters,

    // Computed
    isWindowStarted,
    isWindowCompleted,
    currentRound,
    highlightPlayers,
    marketStats,
    filteredListings,
    listingsByAbility,
    listingsByPrice,
    filteredFreeAgents,
    freeAgentsByPosition,

    // Actions - Preview
    loadPreviewData,

    // Actions - AI Transfer Window
    startTransferWindow,
    executeNextRound,
    fastForwardAll,
    getWindowStatus,
    getRoundEvents,

    // Actions - Original
    loadTransferMarket,
    loadFreeAgents,
    loadAll,
    listPlayer,
    cancelListing,
    buyPlayer,
    signFreeAgent,
    loadTransferHistory,
    updateFilters,
    clearFilters,
    clearError,
  }
})
