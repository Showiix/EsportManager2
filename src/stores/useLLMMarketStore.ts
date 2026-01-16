import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  llmMarketApi,
  type MarketStateSummary,
  type TeamMarketSummary,
  type GenerationProgress,
  type NegotiationListInfo,
  type NegotiationDetailInfo,
  type DepartureCandidateInfo,
  type MarketEvent,
  type RoundExecutionResult,
  type MarketPhase,
  type RenewalProcessingResult,
} from '@/api/tauri'

// 实时进度事件类型
interface IntentionProgressEvent {
  current: number
  total: number
  player_name: string
  status: string
  wants_to_leave: boolean | null
}

interface StrategyProgressEvent {
  current: number
  total: number
  team_name: string
  status: string
}

interface RenewalProgressEvent {
  current: number
  total: number
  player_name: string
  team_name: string
  status: string
  renewal_successful: boolean | null
}

// 阶段显示名称映射（5个大阶段）
export const PHASE_NAMES: Record<MarketPhase, string> = {
  INTENTION_GENERATION: '选手意愿',
  STRATEGY_GENERATION: '战队策略',
  RENEWAL_PROCESSING: '续约处理',
  FREE_MARKET: '自由市场',
  TRANSFER_ROUNDS: '挖角转会',
  COMPLETED: '已完成',
}

// 阶段描述映射
export const PHASE_DESCRIPTIONS: Record<MarketPhase, string> = {
  INTENTION_GENERATION: 'AI 分析每位选手的离队意愿和期望条件',
  STRATEGY_GENERATION: 'AI 为每支球队生成转会策略',
  RENEWAL_PROCESSING: '处理愿意留队选手的续约谈判',
  FREE_MARKET: '多轮报价和谈判，球队签约自由球员',
  TRANSFER_ROUNDS: '向有合同的85+选手发起挖人，需支付转会费',
  COMPLETED: '转会窗口已关闭',
}

export const useLLMMarketStore = defineStore('llmMarket', () => {
  // ==================== 状态 ====================

  // 市场状态
  const marketState = ref<MarketStateSummary | null>(null)
  const teamStates = ref<TeamMarketSummary[]>([])

  // 谈判数据
  const negotiations = ref<NegotiationListInfo[]>([])
  const currentNegotiation = ref<NegotiationDetailInfo | null>(null)

  // 离队候选人
  const departureCandidates = ref<DepartureCandidateInfo[]>([])

  // 事件
  const events = ref<MarketEvent[]>([])

  // 加载状态
  const isLoading = ref(false)
  const isGeneratingIntentions = ref(false)
  const isGeneratingStrategies = ref(false)
  const isProcessingRenewals = ref(false)
  const isExecutingRound = ref(false)

  // 规则引擎模式（默认开启）
  const useRuleEngine = ref(true)

  // 进度
  const generationProgress = ref<GenerationProgress | null>(null)

  // 续约结果
  const renewalResult = ref<RenewalProcessingResult | null>(null)

  // 错误
  const error = ref<string | null>(null)

  // ==================== 计算属性 ====================

  const isMarketInitialized = computed(() => marketState.value !== null)

  const currentPhase = computed(() => marketState.value?.current_phase || 'INTENTION_GENERATION')

  const currentPhaseName = computed(() =>
    marketState.value?.phase_name || PHASE_NAMES[currentPhase.value]
  )

  const currentPhaseDescription = computed(() =>
    marketState.value?.phase_description || PHASE_DESCRIPTIONS[currentPhase.value]
  )

  const canGenerateIntentions = computed(() =>
    currentPhase.value === 'INTENTION_GENERATION'
  )

  const canGenerateStrategies = computed(() =>
    currentPhase.value === 'STRATEGY_GENERATION'
  )

  const canProcessRenewals = computed(() =>
    currentPhase.value === 'RENEWAL_PROCESSING'
  )

  // 自由市场和挖角阶段：都可以执行轮次
  const canExecuteRound = computed(() =>
    currentPhase.value === 'FREE_MARKET' ||
    currentPhase.value === 'TRANSFER_ROUNDS'
  )

  // 需要手动推进的阶段
  const canAdvancePhase = computed(() =>
    currentPhase.value === 'STRATEGY_GENERATION'  // 策略生成后推进到续约
  )

  const isMarketComplete = computed(() => marketState.value?.is_completed || false)

  // 重要事件（重要度 >= 4）
  const importantEvents = computed(() =>
    events.value.filter(e =>
      e.event_type === 'SIGNING_COMPLETED' ||
      e.event_type === 'TRADE_COMPLETED' ||
      e.event_type === 'OFFER_ACCEPTED' ||
      e.event_type === 'TRANSFER_REQUESTED'
    )
  )

  // ==================== 方法 ====================

  // 初始化市场
  const initMarket = async () => {
    if (isLoading.value) return

    isLoading.value = true
    error.value = null

    try {
      marketState.value = await llmMarketApi.initMarket()
      await loadTeamStates()
      return marketState.value
    } catch (e) {
      console.error('Failed to init market:', e)
      error.value = e instanceof Error ? e.message : '初始化市场失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 加载市场状态
  const loadMarketState = async () => {
    isLoading.value = true
    error.value = null

    try {
      marketState.value = await llmMarketApi.getMarketState()
      // 如果阶段已经过了续约处理（自由市场、挖角、完成），加载续约结果
      if (marketState.value &&
          (marketState.value.current_phase === 'FREE_MARKET' ||
           marketState.value.current_phase === 'TRANSFER_ROUNDS' ||
           marketState.value.current_phase === 'COMPLETED')) {
        await loadRenewalResults()
      }
      return marketState.value
    } catch (e) {
      console.error('Failed to load market state:', e)
      error.value = e instanceof Error ? e.message : '加载市场状态失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 加载球队状态
  const loadTeamStates = async () => {
    try {
      teamStates.value = await llmMarketApi.getAllTeamMarketStates()
    } catch (e) {
      console.error('Failed to load team states:', e)
    }
  }

  // 生成选手意愿
  const generateIntentions = async () => {
    if (isGeneratingIntentions.value) return

    isGeneratingIntentions.value = true
    error.value = null

    // 设置初始进度
    generationProgress.value = {
      task_type: 'player_intentions',
      current: 0,
      total: 0,
      percentage: 0,
      current_item: '准备中...',
      is_completed: false,
      errors: [],
    }

    // 监听实时进度事件
    let unlisten: UnlistenFn | null = null
    try {
      unlisten = await listen<IntentionProgressEvent>('intention-generation-progress', (event) => {
        const data = event.payload
        generationProgress.value = {
          task_type: 'player_intentions',
          current: data.current,
          total: data.total,
          percentage: data.total > 0 ? Math.round((data.current / data.total) * 100) : 0,
          current_item: data.player_name,
          is_completed: data.status === 'completed',
          errors: [],
        }
      })

      // 调用后端生成
      const progress = await llmMarketApi.generatePlayerIntentions()
      await loadMarketState()
      await loadDepartureCandidates()
      return progress
    } catch (e) {
      console.error('Failed to generate intentions:', e)
      error.value = e instanceof Error ? e.message : '生成选手意愿失败'
      throw e
    } finally {
      // 取消监听
      if (unlisten) {
        unlisten()
      }
      isGeneratingIntentions.value = false
      generationProgress.value = null
    }
  }

  // 生成球队策略
  const generateStrategies = async () => {
    if (isGeneratingStrategies.value) return

    isGeneratingStrategies.value = true
    error.value = null

    // 设置初始进度
    generationProgress.value = {
      task_type: 'team_strategies',
      current: 0,
      total: 0,
      percentage: 0,
      current_item: '准备中...',
      is_completed: false,
      errors: [],
    }

    // 监听实时进度事件
    let unlisten: UnlistenFn | null = null
    try {
      unlisten = await listen<StrategyProgressEvent>('strategy-generation-progress', (event) => {
        const data = event.payload
        generationProgress.value = {
          task_type: 'team_strategies',
          current: data.current,
          total: data.total,
          percentage: data.total > 0 ? Math.round((data.current / data.total) * 100) : 0,
          current_item: data.team_name,
          is_completed: data.status === 'completed',
          errors: [],
        }
      })

      // 根据模式选择执行引擎
      const progress = useRuleEngine.value
        ? await llmMarketApi.generateRuleBasedStrategies()  // 规则引擎（快速）
        : await llmMarketApi.generateTeamStrategies()       // LLM引擎（慢）
      generationProgress.value = progress
      await loadMarketState()
      await loadTeamStates()
      return progress
    } catch (e) {
      console.error('Failed to generate strategies:', e)
      error.value = e instanceof Error ? e.message : '生成球队策略失败'
      throw e
    } finally {
      // 取消监听
      if (unlisten) {
        unlisten()
      }
      isGeneratingStrategies.value = false
      generationProgress.value = null
    }
  }

  // 处理续约
  const processRenewals = async (): Promise<RenewalProcessingResult | undefined> => {
    if (isProcessingRenewals.value) return

    isProcessingRenewals.value = true
    error.value = null
    renewalResult.value = null

    // 设置初始进度
    generationProgress.value = {
      task_type: 'renewals',
      current: 0,
      total: 0,
      percentage: 0,
      current_item: '准备中...',
      is_completed: false,
      errors: [],
    }

    // 监听实时进度事件
    let unlisten: UnlistenFn | null = null
    try {
      unlisten = await listen<RenewalProgressEvent>('renewal-progress', (event) => {
        const data = event.payload
        generationProgress.value = {
          task_type: 'renewals',
          current: data.current,
          total: data.total,
          percentage: data.total > 0 ? Math.round((data.current / data.total) * 100) : 0,
          current_item: data.player_name ? `${data.player_name} (${data.team_name})` : data.team_name,
          is_completed: data.status === 'completed',
          errors: [],
        }
      })

      const result = await llmMarketApi.processRenewals()
      renewalResult.value = result
      await loadMarketState()
      await loadEvents()
      return result
    } catch (e) {
      console.error('Failed to process renewals:', e)
      error.value = e instanceof Error ? e.message : '处理续约失败'
      throw e
    } finally {
      // 取消监听
      if (unlisten) {
        unlisten()
      }
      isProcessingRenewals.value = false
      generationProgress.value = null
    }
  }

  // 执行一轮
  const executeRound = async (): Promise<RoundExecutionResult | undefined> => {
    if (isExecutingRound.value) return

    isExecutingRound.value = true
    error.value = null

    try {
      // 根据模式选择执行引擎
      const result = useRuleEngine.value
        ? await llmMarketApi.executeRuleBasedRound()  // 规则引擎（快速）
        : await llmMarketApi.executeRound()           // LLM引擎（慢）

      await loadMarketState()
      await loadNegotiations()
      await loadEvents()
      return result
    } catch (e) {
      console.error('Failed to execute round:', e)
      error.value = e instanceof Error ? e.message : '执行轮次失败'
      throw e
    } finally {
      isExecutingRound.value = false
    }
  }

  // 推进阶段
  const advancePhase = async () => {
    try {
      marketState.value = await llmMarketApi.advancePhase()
      return marketState.value
    } catch (e) {
      console.error('Failed to advance phase:', e)
      error.value = e instanceof Error ? e.message : '推进阶段失败'
      throw e
    }
  }

  // 加载谈判列表
  const loadNegotiations = async () => {
    try {
      negotiations.value = await llmMarketApi.getActiveNegotiations()
    } catch (e) {
      console.error('Failed to load negotiations:', e)
    }
  }

  // 加载谈判详情
  const loadNegotiationDetail = async (negotiationId: number) => {
    try {
      currentNegotiation.value = await llmMarketApi.getNegotiationDetail(negotiationId)
      return currentNegotiation.value
    } catch (e) {
      console.error('Failed to load negotiation detail:', e)
      throw e
    }
  }

  // 加载离队候选人
  const loadDepartureCandidates = async () => {
    try {
      departureCandidates.value = await llmMarketApi.getDepartureCandidates()
    } catch (e) {
      console.error('Failed to load departure candidates:', e)
    }
  }

  // 加载事件
  const loadEvents = async () => {
    try {
      events.value = await llmMarketApi.getMarketEvents()
    } catch (e) {
      console.error('Failed to load events:', e)
    }
  }

  // 加载续约结果（从数据库）
  const loadRenewalResults = async () => {
    try {
      const result = await llmMarketApi.getRenewalResults()
      if (result && result.total_processed > 0) {
        renewalResult.value = result
      }
    } catch (e) {
      console.error('Failed to load renewal results:', e)
    }
  }

  // 加载指定轮次事件
  const loadEventsForRound = async (round: number) => {
    try {
      return await llmMarketApi.getEventsForRound(round)
    } catch (e) {
      console.error('Failed to load events for round:', e)
      return []
    }
  }

  // 重置转会市场（清除数据库数据）
  const resetMarket = async () => {
    if (isLoading.value) return

    isLoading.value = true
    error.value = null

    try {
      await llmMarketApi.resetMarket()
      reset()
      return true
    } catch (e) {
      console.error('Failed to reset market:', e)
      error.value = e instanceof Error ? e.message : '重置市场失败'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // 重置状态（仅前端）
  const reset = () => {
    marketState.value = null
    teamStates.value = []
    negotiations.value = []
    currentNegotiation.value = null
    departureCandidates.value = []
    events.value = []
    isLoading.value = false
    isGeneratingIntentions.value = false
    isGeneratingStrategies.value = false
    isProcessingRenewals.value = false
    isExecutingRound.value = false
    generationProgress.value = null
    renewalResult.value = null
    error.value = null
  }

  return {
    // 状态
    marketState,
    teamStates,
    negotiations,
    currentNegotiation,
    departureCandidates,
    events,
    isLoading,
    isGeneratingIntentions,
    isGeneratingStrategies,
    isProcessingRenewals,
    isExecutingRound,
    useRuleEngine,  // 规则引擎模式开关
    generationProgress,
    renewalResult,
    error,

    // 计算属性
    isMarketInitialized,
    currentPhase,
    currentPhaseName,
    currentPhaseDescription,
    canGenerateIntentions,
    canGenerateStrategies,
    canProcessRenewals,
    canExecuteRound,
    canAdvancePhase,
    isMarketComplete,
    importantEvents,

    // 方法
    initMarket,
    loadMarketState,
    loadTeamStates,
    generateIntentions,
    generateStrategies,
    processRenewals,
    executeRound,
    advancePhase,
    loadNegotiations,
    loadNegotiationDetail,
    loadDepartureCandidates,
    loadEvents,
    loadEventsForRound,
    loadRenewalResults,
    resetMarket,
    reset,
  }
})
