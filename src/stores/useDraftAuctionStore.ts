/**
 * Draft Auction Store - 管理选秀权拍卖
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  draftAuctionApi,
  type AuctionStatus,
  type AuctionEvent,
  type DraftPickPrice,
} from '@/api/tauri'
import { formatMoney } from '@/utils'

export const useDraftAuctionStore = defineStore('draftAuction', () => {
  // ========================================
  // State
  // ========================================

  // 拍卖状态
  const auctionStatus = ref<AuctionStatus | null>(null)

  // 拍卖事件列表
  const events = ref<AuctionEvent[]>([])

  // 签位价格配置
  const pickPrices = ref<DraftPickPrice[]>([])

  // 加载状态
  const isLoading = ref(false)

  // 是否正在自动播放
  const isAutoPlaying = ref(false)

  // 自动播放间隔 (ms)
  const autoPlayInterval = ref(2000)

  // 当前显示到的事件索引
  const displayedEventIndex = ref(0)

  // 错误信息
  const error = ref<string | null>(null)

  // ========================================
  // Computed
  // ========================================

  // 拍卖是否已开始
  const isAuctionStarted = computed(() => {
    return auctionStatus.value !== null && auctionStatus.value.status !== 'PREPARING'
  })

  // 拍卖是否已完成
  const isAuctionCompleted = computed(() => {
    return auctionStatus.value?.status === 'COMPLETED'
  })

  // 拍卖是否进行中
  const isAuctionInProgress = computed(() => {
    return auctionStatus.value?.status === 'IN_PROGRESS'
  })

  // 当前轮次
  const currentRound = computed(() => auctionStatus.value?.current_round ?? 0)

  // 总轮数
  const totalRounds = computed(() => auctionStatus.value?.total_rounds ?? 3)

  // 挂牌列表
  const listings = computed(() => auctionStatus.value?.listings ?? [])

  // 活跃挂牌
  const activeListings = computed(() =>
    listings.value.filter(l => l.status === 'ACTIVE')
  )

  // 已售出挂牌
  const soldListings = computed(() =>
    listings.value.filter(l => l.status === 'SOLD')
  )

  // 流拍挂牌
  const expiredListings = computed(() =>
    listings.value.filter(l => l.status === 'EXPIRED')
  )

  // 总成交额(万)
  const totalRevenue = computed(() =>
    Math.floor((auctionStatus.value?.total_revenue ?? 0) / 10000)
  )

  // 总佣金(万)
  const totalCommission = computed(() =>
    Math.floor((auctionStatus.value?.total_commission ?? 0) / 10000)
  )

  // 显示的事件列表
  const displayedEvents = computed(() =>
    events.value.slice(0, displayedEventIndex.value + 1)
  )

  // 是否有未显示的事件
  const hasMoreEvents = computed(() =>
    displayedEventIndex.value < events.value.length - 1
  )

  // 拍卖统计
  const auctionStats = computed(() => ({
    totalAuctions: auctionStatus.value?.total_auctions ?? 0,
    successfulAuctions: auctionStatus.value?.successful_auctions ?? 0,
    totalRevenue: totalRevenue.value,
    totalCommission: totalCommission.value,
    activeCount: activeListings.value.length,
    soldCount: soldListings.value.length,
    expiredCount: expiredListings.value.length,
  }))

  // ========================================
  // Actions
  // ========================================

  // 加载签位价格配置
  async function loadPickPrices() {
    try {
      const prices = await draftAuctionApi.getDraftPickPrices()
      pickPrices.value = prices
    } catch (e) {
      console.error('Failed to load pick prices:', e)
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  // 获取拍卖状态
  async function fetchAuctionStatus(regionId: number) {
    isLoading.value = true
    error.value = null

    try {
      const status = await draftAuctionApi.getStatus(regionId)
      auctionStatus.value = status

      if (status) {
        await fetchAuctionEvents(regionId)
      }
    } catch (e) {
      console.error('Failed to fetch auction status:', e)
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  // 获取拍卖事件
  async function fetchAuctionEvents(regionId: number) {
    try {
      const eventList = await draftAuctionApi.getEvents(regionId)
      events.value = eventList
      // 重置显示索引到最新
      displayedEventIndex.value = eventList.length - 1
    } catch (e) {
      console.error('Failed to fetch auction events:', e)
    }
  }

  // 开始拍卖
  async function startAuction(regionId: number) {
    isLoading.value = true
    error.value = null

    try {
      const status = await draftAuctionApi.startAuction(regionId)
      auctionStatus.value = status
      await fetchAuctionEvents(regionId)
      displayedEventIndex.value = 0 // 从头开始显示
      return status
    } catch (e) {
      console.error('Failed to start auction:', e)
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 执行一轮竞拍
  async function executeRound(regionId: number) {
    isLoading.value = true
    error.value = null

    try {
      const status = await draftAuctionApi.executeRound(regionId)
      auctionStatus.value = status
      await fetchAuctionEvents(regionId)
      return status
    } catch (e) {
      console.error('Failed to execute round:', e)
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 快进完成所有轮次
  async function fastForward(regionId: number) {
    isLoading.value = true
    error.value = null

    try {
      const status = await draftAuctionApi.fastForward(regionId)
      auctionStatus.value = status
      await fetchAuctionEvents(regionId)
      // 显示所有事件
      displayedEventIndex.value = events.value.length - 1
      return status
    } catch (e) {
      console.error('Failed to fast forward auction:', e)
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 完成拍卖
  async function finalizeAuction(regionId: number) {
    isLoading.value = true
    error.value = null

    try {
      await draftAuctionApi.finalizeAuction(regionId)
      await fetchAuctionStatus(regionId)
    } catch (e) {
      console.error('Failed to finalize auction:', e)
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 显示下一个事件
  function showNextEvent() {
    if (hasMoreEvents.value) {
      displayedEventIndex.value++
    }
  }

  // 开始自动播放
  function startAutoPlay() {
    isAutoPlaying.value = true
  }

  // 停止自动播放
  function stopAutoPlay() {
    isAutoPlaying.value = false
  }

  // 重置显示
  function resetDisplay() {
    displayedEventIndex.value = 0
  }

  // 跳转到最新
  function skipToLatest() {
    displayedEventIndex.value = events.value.length - 1
  }

  // 清除状态
  function clearState() {
    auctionStatus.value = null
    events.value = []
    displayedEventIndex.value = 0
    isAutoPlaying.value = false
    error.value = null
  }

  // ========================================
  // Helpers
  // ========================================

  // 获取指定签位的价格
  function getPriceForPosition(position: number): DraftPickPrice | undefined {
    return pickPrices.value.find(p => p.position === position)
  }

  // 格式化金额 - 使用统一工具函数
  function formatAmount(amount: number): string {
    return formatMoney(amount)
  }

  // 获取事件重要性颜色
  function getImportanceColor(importance: string): string {
    switch (importance) {
      case 'BREAKING':
        return 'var(--el-color-danger)'
      case 'MAJOR':
        return 'var(--el-color-warning)'
      case 'NORMAL':
        return 'var(--el-color-primary)'
      case 'MINOR':
        return 'var(--el-color-info)'
      default:
        return 'var(--el-color-info)'
    }
  }

  // 获取挂牌状态颜色
  function getListingStatusColor(status: string): string {
    switch (status) {
      case 'ACTIVE':
        return 'success'
      case 'SOLD':
        return 'primary'
      case 'EXPIRED':
        return 'info'
      case 'WITHDRAWN':
        return 'warning'
      default:
        return 'info'
    }
  }

  // 获取挂牌状态文本
  function getListingStatusText(status: string): string {
    switch (status) {
      case 'PENDING':
        return '待上架'
      case 'ACTIVE':
        return '拍卖中'
      case 'SOLD':
        return '已售出'
      case 'EXPIRED':
        return '已流拍'
      case 'WITHDRAWN':
        return '已撤回'
      default:
        return status
    }
  }

  return {
    // State
    auctionStatus,
    events,
    pickPrices,
    isLoading,
    isAutoPlaying,
    autoPlayInterval,
    displayedEventIndex,
    error,

    // Computed
    isAuctionStarted,
    isAuctionCompleted,
    isAuctionInProgress,
    currentRound,
    totalRounds,
    listings,
    activeListings,
    soldListings,
    expiredListings,
    totalRevenue,
    totalCommission,
    displayedEvents,
    hasMoreEvents,
    auctionStats,

    // Actions
    loadPickPrices,
    fetchAuctionStatus,
    fetchAuctionEvents,
    startAuction,
    executeRound,
    fastForward,
    finalizeAuction,
    showNextEvent,
    startAutoPlay,
    stopAutoPlay,
    resetDisplay,
    skipToLatest,
    clearState,

    // Helpers
    getPriceForPosition,
    formatAmount,
    getImportanceColor,
    getListingStatusColor,
    getListingStatusText,
  }
})
