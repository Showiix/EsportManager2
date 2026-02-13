import { invokeCommand } from './client'

// ========================================
// AI Transfer System (GM 配置和 AI 策略)
// ========================================

/** GM 人格类型 */
export type GMPersonalityType =
  | 'CHAMPIONSHIP'
  | 'YOUTH_DEVELOPMENT'
  | 'BALANCED'
  | 'SPECULATOR'
  | 'REBUILDING'
  | 'CUSTOM'

/** 出售策略激进度 */
export type SellAggressivenessType =
  | 'CONSERVATIVE'
  | 'NORMAL'
  | 'AGGRESSIVE'

/** GM 人格类型信息 */
export interface PersonalityTypeInfo {
  value: string
  name: string
  description: string
  icon: string
}

/** 球队 GM 配置信息 */
export interface TeamGMProfileInfo {
  team_id: number
  team_name: string
  team_short_name: string | null
  region_id: number
  region_name: string
  personality: GMPersonalityType
  personality_name: string
  personality_description: string
  custom_prompt: string | null
  risk_tolerance: number
  // 转会配置
  budget_ratio: number
  sell_aggressiveness: SellAggressivenessType
  preferred_age_min: number
  preferred_age_max: number
  min_ability_threshold: number
  price_premium_max: number
  position_priorities: Record<string, number>
  // 选秀配置
  draft_pick_sell_threshold: number
  draft_pick_bid_aggressiveness: number
  draft_preference_ability_weight: number
  draft_young_bias: number
}

/** 转会目标 */
export interface TransferTarget {
  player_id: number
  player_name: string
  position: string
  ability: number
  potential: number
  age: number
  current_team_id: number | null
  current_team_name: string | null
  market_value: number
  max_offer: number
  priority: number
  reasoning: string
}

/** 出售候选 */
export interface SellCandidate {
  player_id: number
  player_name: string
  position: string
  ability: number
  age: number
  salary: number
  market_value: number
  min_price: number
  urgency: number
  reasoning: string
}

/** 预算分配 */
export interface BudgetAllocation {
  total_budget: number
  transfer_spend: number
  salary_spend: number
  reserve: number
}

/** AI 策略信息（用于展示） */
export interface AIStrategyInfo {
  team_id: number
  team_name: string
  season_id: number
  overall_strategy: string
  strategy_description: string
  targets_count: number
  sell_count: number
  priority_positions: string[]
  budget: BudgetAllocation  // 与后端匹配
  reasoning: string
  is_mock: boolean
  generated_at: string
}

/** AI 转会策略（完整） */
export interface AITransferStrategy {
  id: number
  team_id: number
  save_id: string
  season_id: number
  overall_strategy: string
  strategy_description: string
  targets: TransferTarget[]
  willing_to_sell: SellCandidate[]
  priority_positions: string[]
  budget_allocation: BudgetAllocation
  reasoning: string
  analysis_steps?: TeamAnalysisStep[]
  is_mock: boolean
  generated_at: string
}

/** 战队分析步骤 */
export interface TeamAnalysisStep {
  step_name: string
  data_used: string
  threshold: string
  result: string
  impact: string
}

// ========== 选手转会策略类型 ==========

/** 偏好球队信息 */
export interface PreferredTeamInfo {
  team_id: number
  team_name: string
  priority: number
  reason: string
  reason_detail: string
  attractiveness_score: number
}

/** AI分析数据快照 */
export interface AnalysisDataSnapshot {
  player_name: string
  position: string
  age: number
  ability: number
  potential: number
  satisfaction: number
  loyalty: number
  is_starter: boolean
  current_salary: number  // 万/年
  contract_end_season: number | null
  team_name: string
  team_avg_ability: number
  loyalty_type: string       // 忠心耿耿/忠诚/中立/机会主义/雇佣兵
  departure_threshold: number // 离队阈值
}

/** AI分析步骤 */
export interface AnalysisStep {
  step_name: string      // 步骤名称，如"满意度分析"
  data_used: string      // 使用的数据，如"满意度: 35"
  threshold?: string     // 阈值说明，如"离队阈值: 50"
  result: string         // 结论，如"低于阈值，判断想离队"
  impact?: string        // 影响，如"基础离队概率 +70%"
}

/** 选手转会策略（完整） */
export interface PlayerTransferStrategy {
  id: number
  player_id: number
  save_id: string
  season_id: number
  wants_to_leave: boolean
  decision_confidence: number
  departure_reasons: string[]
  leave_reasoning: string
  preferred_teams: PreferredTeamInfo[]
  team_preference_reasoning: string
  expected_salary: number
  expected_min_salary: number
  expected_years: number
  requires_starter: boolean
  analysis_data: AnalysisDataSnapshot | null
  analysis_steps: AnalysisStep[]
  is_mock: boolean
  generated_at: string
}

/** 选手转会策略展示信息 */
export interface PlayerTransferStrategyInfo {
  player_id: number
  player_name: string
  position: string
  ability: number
  age: number
  team_id: number
  team_name: string
  wants_to_leave: boolean
  decision_confidence: number
  departure_reasons: string[]
  leave_reasoning: string
  preferred_teams_count: number
  preferred_teams: PreferredTeamInfo[]
  team_preference_reasoning: string
  expected_salary: number
  expected_min_salary: number
  expected_years: number
  requires_starter: boolean
  is_mock: boolean
  generated_at: string
}

export const aiTransferApi = {
  /** 获取所有 GM 人格类型 */
  getPersonalityTypes: () =>
    invokeCommand<PersonalityTypeInfo[]>('get_gm_personality_types'),

  /** 获取所有球队的 GM 配置 */
  getAllGMProfiles: () =>
    invokeCommand<TeamGMProfileInfo[]>('get_all_gm_profiles'),

  /** 获取单个球队的 GM 配置 */
  getTeamGMProfile: (teamId: number) =>
    invokeCommand<TeamGMProfileInfo>('get_team_gm_profile', { teamId }),

  /** 更新球队 GM 配置 */
  updateTeamGMProfile: (
    teamId: number,
    personality: string,
    customPrompt: string | null,
    riskTolerance: number,
    budgetRatio: number,
    sellAggressiveness: string,
    preferredAgeMin: number,
    preferredAgeMax: number,
    minAbilityThreshold: number,
    pricePremiumMax: number,
    positionPriorities: Record<string, number>,
    draftPickSellThreshold: number,
    draftPickBidAggressiveness: number,
    draftPreferenceAbilityWeight: number,
    draftYoungBias: number
  ) =>
    invokeCommand<void>('update_team_gm_profile', {
      teamId,
      personality,
      customPrompt,
      riskTolerance,
      budgetRatio,
      sellAggressiveness,
      preferredAgeMin,
      preferredAgeMax,
      minAbilityThreshold,
      pricePremiumMax,
      positionPriorities,
      draftPickSellThreshold,
      draftPickBidAggressiveness,
      draftPreferenceAbilityWeight,
      draftYoungBias,
    }),

  /** 批量更新 GM 配置 */
  batchUpdateGMProfiles: (
    profiles: [number, string, string | null, number, boolean][]
  ) =>
    invokeCommand<number>('batch_update_gm_profiles', { profiles }),

  /** 生成所有球队的 AI 策略 */
  generateAIStrategies: () =>
    invokeCommand<AIStrategyInfo[]>('generate_ai_strategies'),

  /** 获取单个球队的 AI 策略 */
  getTeamAIStrategy: (teamId: number) =>
    invokeCommand<AITransferStrategy>('get_team_ai_strategy', { teamId }),

  /** 初始化 AI 转会相关表 */
  initAITransferTables: () =>
    invokeCommand<void>('init_ai_transfer_tables'),

  // ========== 选手转会策略相关 ==========

  /** 为选手生成转会策略 */
  generatePlayerTransferStrategy: (playerId: number) =>
    invokeCommand<PlayerTransferStrategy>('generate_player_transfer_strategy', { playerId }),

  /** 获取选手的转会策略 */
  getPlayerTransferStrategy: (playerId: number) =>
    invokeCommand<PlayerTransferStrategy | null>('get_player_transfer_strategy', { playerId }),

  /** 获取所有想离队选手的策略列表 */
  getAllPlayerStrategies: () =>
    invokeCommand<PlayerTransferStrategyInfo[]>('get_all_player_strategies'),

  /** 初始化选手策略数据库表 */
  initPlayerStrategyTables: () =>
    invokeCommand<void>('init_player_strategy_tables'),
}
