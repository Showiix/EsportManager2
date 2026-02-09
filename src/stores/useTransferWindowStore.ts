import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  transferWindowApi,
  queryApi,
  type TransferWindowResponse,
  type TransferEvent,
  type RoundExecutionResponse,
  type FastForwardResponse,
  type TransferReport,
  type TeamPersonalityConfig,
  type TeamReputation,
  type UpdatePersonalityRequest,
  type TransferWindowCloseValidation,
} from '@/api/tauri'
import { formatMoney } from '@/utils'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TransferWindowStore')

/** 轮次名称 */
export const ROUND_NAMES: Record<number, string> = {
  1: '赛季结算',
  2: '双向评估',
  3: '续约谈判',
  4: '自由球员',
  5: '有合同挖角',
  6: '财政调整',
  7: '收尾补救',
}

/** 事件类型名称 */
export const EVENT_TYPE_NAMES: Record<string, string> = {
  SEASON_SETTLEMENT: '赛季结算',
  CONTRACT_RENEWAL: '续约成功',
  CONTRACT_TERMINATION: '续约失败',
  FREE_AGENT_SIGNING: '自由签约',
  TRANSFER_PURCHASE: '转会买断',
  PLAYER_RETIREMENT: '光荣退役',
  PLAYER_LISTED: '挂牌出售',
  EMERGENCY_SIGNING: '紧急签约',
  DRAFT_PICK_AUCTION: '选秀权拍卖',
  FINANCIAL_ADJUSTMENT: '财政调整',
  PLAYER_REQUEST_TRANSFER: '球员求转',
  LOAN: '租借',
  LOAN_RETURN: '租借归还',
  PLAYER_RELEASE: '解约放人',
}

/** 事件等级配置 */
export const EVENT_LEVEL_CONFIG: Record<string, { label: string; color: string; tagType: string }> = {
  S: { label: '重磅', color: '#f59e0b', tagType: 'warning' },
  A: { label: '头条', color: '#8b5cf6', tagType: 'primary' },
  B: { label: '要闻', color: '#3b82f6', tagType: 'info' },
  C: { label: '普通', color: '#6b7280', tagType: 'default' },
}

/** AI性格配置 */
export const PERSONALITY_CONFIG: Record<string, { label: string; description: string; color: string }> = {
  AGGRESSIVE: { label: '激进型', description: '追求明星球员，愿意高价买入', color: '#ef4444' },
  CONSERVATIVE: { label: '保守型', description: '控制支出，偏好性价比球员', color: '#22c55e' },
  BALANCED: { label: '平衡型', description: '综合考虑各因素，稳健决策', color: '#3b82f6' },
  DEVELOPMENT: { label: '青训型', description: '注重年轻球员培养，长期发展', color: '#8b5cf6' },
  WIN_NOW: { label: '夺冠型', description: '不惜代价追求即战力，志在夺冠', color: '#f59e0b' },
}

export const useTransferWindowStore = defineStore('transferWindow', () => {
  // ============================================
  // State
  // ============================================

  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // 转会期状态
  const windowInfo = ref<TransferWindowResponse | null>(null)
  const events = ref<TransferEvent[]>([])
  const report = ref<TransferReport | null>(null)

  // 赛区信息
  const currentRegionId = ref<number>(0)
  const currentRegionCode = ref<string>('')

  // AI性格配置
  const teamPersonalities = ref<Map<number, TeamPersonalityConfig>>(new Map())
  const teamReputations = ref<Map<number, TeamReputation>>(new Map())
  const gmConfigConfirmed = ref(false)

  // ============================================
  // Getters
  // ============================================

  const isWindowStarted = computed(() =>
    windowInfo.value !== null && windowInfo.value.status !== 'PENDING'
  )

  const isWindowInProgress = computed(() =>
    windowInfo.value?.status === 'IN_PROGRESS'
  )

  const isWindowCompleted = computed(() =>
    windowInfo.value?.status === 'COMPLETED'
  )

  /** 是否等待确认关闭（所有轮次完成但未确认关闭） */
  const isAwaitingClose = computed(() =>
    windowInfo.value?.status === 'IN_PROGRESS' && windowInfo.value?.current_round >= 7
  )

  const currentRound = computed(() =>
    windowInfo.value?.current_round ?? 0
  )

  const totalRounds = computed(() => 7)

  const progressPercentage = computed(() =>
    (currentRound.value / totalRounds.value) * 100
  )

  const currentRoundName = computed(() =>
    ROUND_NAMES[currentRound.value] ?? ''
  )

  // 按等级分组的事件
  const eventsByLevel = computed(() => {
    const grouped: Record<string, TransferEvent[]> = { S: [], A: [], B: [], C: [] }
    for (const event of events.value) {
      if (grouped[event.level]) {
        grouped[event.level].push(event)
      }
    }
    return grouped
  })

  // 按轮次分组的事件
  const eventsByRound = computed(() => {
    const grouped: Record<number, TransferEvent[]> = {}
    for (const event of events.value) {
      if (!grouped[event.round]) {
        grouped[event.round] = []
      }
      grouped[event.round].push(event)
    }
    return grouped
  })

  // 统计信息
  const stats = computed(() => {
    const total = events.value.length
    const transfers = events.value.filter(e =>
      ['FREE_AGENT_SIGNING', 'TRANSFER_PURCHASE', 'EMERGENCY_SIGNING'].includes(e.event_type)
    ).length
    const renewals = events.value.filter(e => e.event_type === 'CONTRACT_RENEWAL').length
    const terminations = events.value.filter(e => e.event_type === 'CONTRACT_TERMINATION').length
    const retirements = events.value.filter(e => e.event_type === 'PLAYER_RETIREMENT').length
    const totalFees = events.value.reduce((sum, e) => sum + e.transfer_fee, 0)

    return {
      total,
      transfers,
      renewals,
      terminations,
      retirements,
      totalFees,
    }
  })

  // ============================================
  // Actions
  // ============================================

  /** 清除状态 */
  function clearState() {
    windowInfo.value = null
    events.value = []
    report.value = null
    error.value = null
  }

  /** 设置当前赛区 */
  function setRegion(regionId: number, regionCode: string) {
    currentRegionId.value = regionId
    currentRegionCode.value = regionCode
  }

  /** 初始化转会期（页面加载时恢复状态，纯查询不创建） */
  async function initTransferWindow() {
    // 如果已经有状态，不需要重新初始化
    if (windowInfo.value) {
      return windowInfo.value
    }

    isLoading.value = true
    error.value = null

    try {
      // 纯查询当前赛季是否有转会窗口（不会创建新的）
      const response = await transferWindowApi.getCurrentTransferWindow()

      if (!response) {
        // 当前赛季没有转会窗口
        return null
      }

      windowInfo.value = response

      // 如果转会期已经在进行中或已完成，加载已有的事件
      if (response.current_round > 0) {
        const existingEvents = await transferWindowApi.getTransferEvents(response.window_id)
        events.value = existingEvents
      }

      return response
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      // 初始化失败不抛出错误，只记录
      logger.warn('初始化转会期失败', { error: e })
      return null
    } finally {
      isLoading.value = false
    }
  }

  /** 开始转会期 */
  async function startTransferWindow() {
    isLoading.value = true
    error.value = null

    try {
      const response = await transferWindowApi.startTransferWindow()
      windowInfo.value = response
      events.value = []
      return response
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /** 执行单轮 */
  async function executeRound(): Promise<RoundExecutionResponse> {
    if (!windowInfo.value) {
      throw new Error('转会期未开始')
    }

    isLoading.value = true
    error.value = null

    try {
      const nextRound = currentRound.value + 1
      const response = await transferWindowApi.executeTransferRound(
        windowInfo.value.window_id,
        nextRound
      )

      // 更新状态
      windowInfo.value.current_round = nextRound

      // 添加新事件
      events.value = [...events.value, ...response.events]

      return response
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /** 快进完成 */
  async function fastForward(): Promise<FastForwardResponse> {
    if (!windowInfo.value) {
      throw new Error('转会期未开始')
    }

    isLoading.value = true
    error.value = null

    try {
      const fromRound = currentRound.value + 1
      const response = await transferWindowApi.fastForwardTransfer(
        windowInfo.value.window_id,
        fromRound
      )

      // 更新状态
      windowInfo.value.current_round = 7

      // 合并所有事件
      for (const round of response.rounds) {
        events.value = [...events.value, ...round.events]
      }

      return response
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /** 确认关闭转会窗口 */
  async function confirmCloseWindow(force: boolean = false): Promise<TransferWindowCloseValidation> {
    if (!windowInfo.value) {
      throw new Error('转会期未开始')
    }

    isLoading.value = true
    error.value = null

    try {
      const result = await transferWindowApi.confirmCloseTransferWindow(
        windowInfo.value.window_id,
        force
      )

      // 如果成功关闭，更新状态
      if (result.is_valid || force) {
        windowInfo.value.status = 'COMPLETED'
      }

      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /** 获取转会报告 */
  async function fetchReport(): Promise<TransferReport> {
    if (!windowInfo.value) {
      throw new Error('转会期未开始')
    }

    isLoading.value = true
    error.value = null

    try {
      const response = await transferWindowApi.getTransferReport(windowInfo.value.window_id)
      report.value = response
      return response
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /** 加载事件 */
  async function loadEvents(round?: number, level?: string) {
    if (!windowInfo.value) return

    isLoading.value = true
    try {
      const response = await transferWindowApi.getTransferEvents(
        windowInfo.value.window_id,
        round,
        level
      )
      events.value = response
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  /** 加载球队AI性格 */
  async function loadTeamPersonality(teamId: number): Promise<TeamPersonalityConfig | null> {
    try {
      const config = await transferWindowApi.getTeamPersonality(teamId)
      if (config) {
        teamPersonalities.value.set(teamId, config)
      }
      return config
    } catch (e) {
      logger.warn('加载球队AI性格失败', { teamId, error: e })
      return null
    }
  }

  /** 加载所有球队AI性格 */
  async function loadAllTeamPersonalities(regionId: number) {
    isLoading.value = true
    try {
      const teams = await queryApi.getTeamsByRegion(regionId)
      for (const team of teams) {
        await loadTeamPersonality(team.id)
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  /** 更新球队AI性格 */
  async function updateTeamPersonality(teamId: number, request: UpdatePersonalityRequest): Promise<boolean> {
    try {
      const success = await transferWindowApi.updateTeamPersonality(teamId, request)
      if (success) {
        await loadTeamPersonality(teamId)
      }
      return success
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  /** 加载球队声望 */
  async function loadTeamReputation(teamId: number): Promise<TeamReputation | null> {
    try {
      const reputation = await transferWindowApi.getTeamReputation(teamId)
      teamReputations.value.set(teamId, reputation)
      return reputation
    } catch (e) {
      logger.warn('加载球队声望失败', { teamId, error: e })
      return null
    }
  }

  /** 确认GM配置 */
  function confirmGMConfig() {
    gmConfigConfirmed.value = true
  }

  /** 重置GM配置确认 */
  function resetGMConfigConfirmation() {
    gmConfigConfirmed.value = false
  }

  // ============================================
  // Helpers
  // ============================================

  /** 格式化金额（输入单位：元）- 使用统一工具函数 */
  function formatAmount(amount: number): string {
    return formatMoney(amount)
  }

  /** 获取事件类型名称 */
  function getEventTypeName(eventType: string): string {
    return EVENT_TYPE_NAMES[eventType] ?? eventType
  }

  /** 获取事件等级配置 */
  function getEventLevelConfig(level: string) {
    return EVENT_LEVEL_CONFIG[level] ?? EVENT_LEVEL_CONFIG.C
  }

  /** 获取轮次名称 */
  function getRoundName(round: number): string {
    return ROUND_NAMES[round] ?? `第${round}轮`
  }

  /** 获取性格配置 */
  function getPersonalityConfig(personality: string) {
    return PERSONALITY_CONFIG[personality] ?? PERSONALITY_CONFIG.BALANCED
  }

  return {
    // State
    isLoading,
    error,
    windowInfo,
    events,
    report,
    currentRegionId,
    currentRegionCode,
    teamPersonalities,
    teamReputations,
    gmConfigConfirmed,

    // Getters
    isWindowStarted,
    isWindowInProgress,
    isWindowCompleted,
    isAwaitingClose,
    currentRound,
    totalRounds,
    progressPercentage,
    currentRoundName,
    eventsByLevel,
    eventsByRound,
    stats,

    // Actions
    clearState,
    setRegion,
    initTransferWindow,
    startTransferWindow,
    executeRound,
    confirmCloseWindow,
    fastForward,
    fetchReport,
    loadEvents,
    loadTeamPersonality,
    loadAllTeamPersonalities,
    updateTeamPersonality,
    loadTeamReputation,
    confirmGMConfig,
    resetGMConfigConfirmation,

    // Helpers
    formatAmount,
    getEventTypeName,
    getEventLevelConfig,
    getRoundName,
    getPersonalityConfig,
  }
})
