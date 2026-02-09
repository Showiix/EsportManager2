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
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('TransferStore')

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

      const [expiring, retiring, market] = await logger.timed('加载转会预览', () =>
        Promise.all([
          eventApi.getExpiringContracts(currentSeason),
          eventApi.getRetiringCandidates(),
          transferApi.getTransferMarket(),
        ])
      )

      expiringContracts.value = expiring
      retiringCandidates.value = retiring
      listings.value = market

      logger.info('转会预览数据已加载', {
        expiring: expiring.length,
        retiring: retiring.length,
        listings: market.length
      })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load preview data'
      handleError(e, {
        component: 'TransferStore',
        userAction: '加载转会预览',
        canRetry: true,
        retryFn: loadPreviewData
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // ========================================
  // Actions - AI 转会窗口
  // ========================================

  /**
   * 开始转会窗口（如果已存在则继续）
   */
  const startTransferWindow = async () => {
    isLoading.value = true
    error.value = null

    try {
      logger.info('开始转会窗口')
      transferWindow.value = await transferApi.startTransferWindow()
      allTransferEvents.value = []
      currentRoundEvents.value = []
      logger.info('转会窗口已开始', { status: transferWindow.value?.status })
      return transferWindow.value
    } catch (e) {
      // 如果转会窗口已存在，尝试获取现有状态
      const errMsg = e instanceof Error ? e.message : String(e)
      if (errMsg.includes('already exists')) {
        logger.debug('转会窗口已存在，加载现有状态')
        transferWindow.value = await transferApi.getTransferWindowStatus()
        // 加载已有的事件
        const events = await transferApi.getTransferEvents()
        allTransferEvents.value = events
        return transferWindow.value
      }
      error.value = errMsg
      handleError(e, {
        component: 'TransferStore',
        userAction: '开始转会窗口'
      })
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
      const nextRound = (transferWindow.value?.current_round ?? 0) + 1
      logger.info('执行转会轮次', { round: nextRound })

      const roundInfo = await logger.timed(`执行第${nextRound}轮转会`, () =>
        transferApi.executeTransferRound()
      )
      currentRoundEvents.value = roundInfo.events
      allTransferEvents.value.push(...roundInfo.events)

      // 更新窗口状态
      transferWindow.value = await transferApi.getTransferWindowStatus()

      logger.info('转会轮次完成', {
        round: roundInfo.round,
        eventsCount: roundInfo.events.length,
        summary: roundInfo.summary
      })
      return roundInfo
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to execute round'
      handleError(e, {
        component: 'TransferStore',
        userAction: '执行转会轮次',
        canRetry: true,
        retryFn: executeNextRound
      })
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
      logger.info('快进完成所有转会')
      transferWindow.value = await logger.timed('快进转会', () =>
        transferApi.fastForwardTransfers()
      )
      allTransferEvents.value = await transferApi.getTransferEvents()
      logger.info('转会窗口已完成', {
        totalEvents: allTransferEvents.value.length
      })
      return transferWindow.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fast forward'
      handleError(e, {
        component: 'TransferStore',
        userAction: '快进转会'
      })
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
      logger.error('获取轮次事件失败', { round, error: e })
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
      logger.debug('转会市场已加载', { count: listings.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load transfer market'
      handleError(e, {
        component: 'TransferStore',
        userAction: '加载转会市场',
        silent: true
      })
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
      logger.debug('自由球员已加载', { count: freeAgents.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load free agents'
      handleError(e, {
        component: 'TransferStore',
        userAction: '加载自由球员',
        silent: true
      })
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
      logger.info('挂牌出售选手', { teamId, playerId, askingPrice })
      const listing = await transferApi.listPlayerForTransfer(teamId, playerId, askingPrice)
      listings.value.push(listing)
      logger.info('选手已挂牌', { playerName: listing.player_name, price: askingPrice })
      return listing
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to list player'
      handleError(e, {
        component: 'TransferStore',
        userAction: '挂牌选手'
      })
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
      logger.info('取消挂牌', { listingId })
      await transferApi.cancelTransferListing(listingId)
      listings.value = listings.value.filter(l => l.id !== listingId)
      logger.info('挂牌已取消', { listingId })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to cancel listing'
      handleError(e, {
        component: 'TransferStore',
        userAction: '取消挂牌'
      })
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
      logger.info('购买挂牌选手', { listingId, buyerTeamId, contractYears, salary })
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

      logger.info('选手购买成功', {
        playerName: record.player_name,
        fee: record.fee,
        toTeam: record.to_team_name
      })
      return record
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to buy player'
      handleError(e, {
        component: 'TransferStore',
        userAction: '购买选手'
      })
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
      logger.info('签约自由球员', { playerId, teamId, contractYears, salary })
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

      logger.info('自由球员签约成功', {
        playerName: record.player_name,
        toTeam: record.to_team_name
      })
      return record
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to sign free agent'
      handleError(e, {
        component: 'TransferStore',
        userAction: '签约自由球员'
      })
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
      logger.debug('转会历史已加载', { count: transferHistory.value.length, teamId })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load transfer history'
      handleError(e, {
        component: 'TransferStore',
        userAction: '加载转会历史',
        silent: true
      })
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
   * 清除所有状态（切换存档时调用）
   */
  const clearAll = () => {
    listings.value = []
    freeAgents.value = []
    transferHistory.value = []
    transferWindow.value = null
    currentRoundEvents.value = []
    allTransferEvents.value = []
    expiringContracts.value = []
    retiringCandidates.value = []
    isLoading.value = false
    error.value = null
    clearFilters()
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
    clearAll,
    clearFilters,
    clearError,
  }
})
