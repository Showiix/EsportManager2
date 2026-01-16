/**
 * Time Store - 管理游戏时间推进系统
 * 统一控制游戏的赛季、阶段、比赛进度
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  timeApi,
  type GameTimeState,
  type CompleteAndAdvanceResult,
  type FastForwardResult,
  type SeasonSettlementResult,
  type SimulateNextResult,
  type PhaseStatus,
  type TimeAction
} from '@/api/tauri'

export const useTimeStore = defineStore('time', () => {
  // ========================================
  // State
  // ========================================

  // 完整的时间状态
  const timeState = ref<GameTimeState | null>(null)

  // 加载状态
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 最后一场模拟的比赛结果
  const lastSimulatedMatch = ref<SimulateNextResult | null>(null)

  // 操作结果消息
  const lastMessage = ref<string | null>(null)

  // 最近颁发的荣誉
  const recentHonors = ref<CompleteAndAdvanceResult['honors_awarded']>([])

  // ========================================
  // Computed
  // ========================================

  // 当前赛季
  const currentSeason = computed(() => timeState.value?.current_season ?? 1)

  // 当前阶段
  const currentPhase = computed(() => timeState.value?.current_phase ?? 'SpringRegular')

  // 当前阶段显示名称
  const phaseDisplayName = computed(() => timeState.value?.phase_display_name ?? '加载中...')

  // 阶段状态
  const phaseStatus = computed<PhaseStatus>(() => timeState.value?.phase_status ?? 'NOT_INITIALIZED')

  // 阶段进度百分比
  const phaseProgress = computed(() => timeState.value?.phase_progress.percentage ?? 0)

  // 赛季进度百分比
  const seasonProgress = computed(() => timeState.value?.season_progress.percentage ?? 0)

  // 是否可以推进到下一阶段
  const canAdvance = computed(() => timeState.value?.can_advance ?? false)

  // 下一阶段名称
  const nextPhase = computed(() => timeState.value?.next_phase ?? null)

  // 可用操作
  const availableActions = computed<TimeAction[]>(() => timeState.value?.available_actions ?? [])

  // 当前阶段的赛事列表
  const tournaments = computed(() => timeState.value?.phase_progress.tournaments ?? [])

  // 已完成比赛数
  const completedMatches = computed(() => timeState.value?.phase_progress.completed_matches ?? 0)

  // 总比赛数
  const totalMatches = computed(() => timeState.value?.phase_progress.total_matches ?? 0)

  // 所有阶段信息
  const allPhases = computed(() => timeState.value?.season_progress.phases ?? [])

  // 当前阶段索引
  const currentPhaseIndex = computed(() => timeState.value?.season_progress.current_phase_index ?? 0)

  // 是否在赛事阶段
  const isInTournamentPhase = computed(() => {
    const phase = currentPhase.value
    return !['TransferWindow', 'Draft', 'SeasonEnd'].includes(phase)
  })

  // 是否在转会期
  const isInTransferWindow = computed(() => currentPhase.value === 'TransferWindow')

  // 是否在选秀期
  const isInDraft = computed(() => currentPhase.value === 'Draft')

  // 是否在赛季结束阶段
  const isSeasonEnd = computed(() => currentPhase.value === 'SeasonEnd')

  // ========================================
  // Actions
  // ========================================

  /**
   * 获取/刷新时间状态
   */
  const fetchTimeState = async () => {
    isLoading.value = true
    error.value = null

    try {
      timeState.value = await timeApi.getTimeState()
      console.log('Time state loaded:', timeState.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '获取时间状态失败'
      console.error('Failed to fetch time state:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 初始化当前阶段（创建赛事）
   */
  const initPhase = async () => {
    isLoading.value = true
    error.value = null

    try {
      const message = await timeApi.initPhase()
      lastMessage.value = message
      console.log('Phase initialized:', message)

      // 刷新状态
      await fetchTimeState()
      return message
    } catch (e) {
      error.value = e instanceof Error ? e.message : '初始化阶段失败'
      console.error('Failed to init phase:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 完成当前阶段并推进
   */
  const completeAndAdvance = async (): Promise<CompleteAndAdvanceResult> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await timeApi.completeAndAdvance()
      lastMessage.value = result.message
      recentHonors.value = result.honors_awarded

      // 更新状态
      timeState.value = result.new_time_state

      console.log('Phase completed and advanced:', result)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : '完成阶段失败'
      console.error('Failed to complete and advance:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 快进到指定目标
   */
  const fastForwardTo = async (target: string): Promise<FastForwardResult> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await timeApi.fastForwardTo(target)
      lastMessage.value = result.message

      // 刷新状态
      await fetchTimeState()

      console.log('Fast forwarded:', result)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : '快进失败'
      console.error('Failed to fast forward:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 模拟所有当前阶段的比赛
   */
  const simulateAll = async (): Promise<number> => {
    isLoading.value = true
    error.value = null

    try {
      const count = await timeApi.simulateAll()
      lastMessage.value = `已模拟 ${count} 场比赛`

      // 刷新状态
      await fetchTimeState()

      console.log('Simulated matches:', count)
      return count
    } catch (e) {
      error.value = e instanceof Error ? e.message : '模拟比赛失败'
      console.error('Failed to simulate all:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 模拟下一场比赛
   */
  const simulateNext = async (): Promise<SimulateNextResult> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await timeApi.simulateNext()
      lastSimulatedMatch.value = result
      lastMessage.value = `${result.home_team_name} ${result.home_score}:${result.away_score} ${result.away_team_name}`

      // 刷新状态
      await fetchTimeState()

      console.log('Simulated match:', result)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : '模拟比赛失败'
      console.error('Failed to simulate next:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 执行赛季结算
   */
  const executeSeasonSettlement = async (): Promise<SeasonSettlementResult> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await timeApi.seasonSettlement()
      lastMessage.value = `赛季 ${result.season} 结算完成`

      // 刷新状态
      await fetchTimeState()

      console.log('Season settlement:', result)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : '赛季结算失败'
      console.error('Failed to execute season settlement:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 开始新赛季
   */
  const startNewSeason = async (): Promise<number> => {
    isLoading.value = true
    error.value = null

    try {
      const newSeason = await timeApi.startNewSeason()
      lastMessage.value = `已进入第 ${newSeason} 赛季`

      // 刷新状态
      await fetchTimeState()

      console.log('New season started:', newSeason)
      return newSeason
    } catch (e) {
      error.value = e instanceof Error ? e.message : '开始新赛季失败'
      console.error('Failed to start new season:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 快进到下一阶段
   */
  const advanceToNextPhase = () => fastForwardTo('NEXT_PHASE')

  /**
   * 快进到夏季赛
   */
  const fastForwardToSummer = () => fastForwardTo('SUMMER')

  /**
   * 快进到世界赛
   */
  const fastForwardToWorlds = () => fastForwardTo('WORLDS')

  /**
   * 快进到赛季结束
   */
  const fastForwardToSeasonEnd = () => fastForwardTo('SEASON_END')

  /**
   * 清除错误
   */
  const clearError = () => {
    error.value = null
  }

  /**
   * 清除消息
   */
  const clearMessage = () => {
    lastMessage.value = null
  }

  // ========================================
  // Return
  // ========================================

  return {
    // State
    timeState,
    isLoading,
    error,
    lastMessage,
    recentHonors,
    lastSimulatedMatch,

    // Computed
    currentSeason,
    currentPhase,
    phaseDisplayName,
    phaseStatus,
    phaseProgress,
    seasonProgress,
    canAdvance,
    nextPhase,
    availableActions,
    tournaments,
    completedMatches,
    totalMatches,
    allPhases,
    currentPhaseIndex,
    isInTournamentPhase,
    isInTransferWindow,
    isInDraft,
    isSeasonEnd,

    // Actions
    fetchTimeState,
    initPhase,
    completeAndAdvance,
    fastForwardTo,
    simulateAll,
    simulateNext,
    executeSeasonSettlement,
    startNewSeason,
    advanceToNextPhase,
    fastForwardToSummer,
    fastForwardToWorlds,
    fastForwardToSeasonEnd,
    clearError,
    clearMessage,
  }
})
