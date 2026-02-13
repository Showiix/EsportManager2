import { invokeCommand } from './client'

// ========================================
// Transfer Window System API (新转会系统)
// ========================================

/** 转会期状态 */
export type TransferWindowStatus = 'PENDING' | 'IN_PROGRESS' | 'COMPLETED' | 'CANCELLED'

/** 转会事件类型 */
export type TransferEventType =
  | 'SEASON_SETTLEMENT'
  | 'CONTRACT_RENEWAL'
  | 'CONTRACT_TERMINATION'
  | 'FREE_AGENT_SIGNING'
  | 'TRANSFER_PURCHASE'
  | 'PLAYER_RETIREMENT'
  | 'PLAYER_LISTED'
  | 'EMERGENCY_SIGNING'
  | 'DRAFT_PICK_AUCTION'
  | 'FINANCIAL_ADJUSTMENT'

/** 事件等级 */
export type TransferEventLevel = 'S' | 'A' | 'B' | 'C'

/** AI球队性格类型 */
export type AITeamPersonality = 'AGGRESSIVE' | 'CONSERVATIVE' | 'BALANCED' | 'DEVELOPMENT' | 'WIN_NOW'

/** 转会期响应 */
export interface TransferWindowResponse {
  window_id: number
  current_round: number
  status: string
  season_id: number
}

/** 转会窗口关闭验证结果 */
export interface TransferWindowCloseValidation {
  is_valid: boolean
  window_id: number
  issues: TransferCloseIssue[]
  message: string
}

/** 转会窗口关闭问题 */
export interface TransferCloseIssue {
  team_id: number
  team_name: string
  issue_type: string
  detail: string
}

/** 转会事件 */
export interface TransferEvent {
  id: number
  window_id: number
  round: number
  event_type: string
  level: string
  player_id: number
  player_name: string
  player_ability: number
  from_team_id: number | null
  from_team_name: string | null
  to_team_id: number | null
  to_team_name: string | null
  transfer_fee: number
  salary: number
  contract_years: number
  reason: string | null
  created_at: string
}

/** 轮次结果 */
export interface RoundResult {
  round: number
  round_name: string
  events: TransferEvent[]
  summary: string
}

/** 轮次执行响应 */
export interface RoundExecutionResponse {
  round: number
  round_name: string
  events: TransferEvent[]
  event_count: number
  next_round: number | null
  summary: string
}

/** 快进响应 */
export interface FastForwardResponse {
  completed_rounds: number
  total_events: number
  rounds: RoundResult[]
}

/** 球队转会摘要 */
export interface TeamTransferSummary {
  team_id: number
  team_name: string
  players_in: number
  players_out: number
  money_spent: number
  money_earned: number
  net_spend: number
}

/** 转会报告 */
export interface TransferReport {
  window_id: number
  season_id: number
  total_events: number
  total_transfer_fee: number
  events_by_type: Record<string, number>
  events_by_level: Record<string, number>
  team_summaries: TeamTransferSummary[]
  top_events: TransferEvent[]
}

/** 球队性格配置 */
export interface TeamPersonalityConfig {
  id: number
  team_id: number
  save_id: string
  personality: string
  short_term_focus: number
  long_term_focus: number
  risk_tolerance: number
  youth_preference: number
  star_chasing: number
  bargain_hunting: number
  updated_at: string
}

/** 球队声望 */
export interface TeamReputation {
  team_id: number
  overall: number
  historical: number
  recent: number
  international: number
}

/** 更新性格请求 */
export interface UpdatePersonalityRequest {
  personality: string
  short_term_focus?: number
  long_term_focus?: number
  risk_tolerance?: number
  youth_preference?: number
  star_chasing?: number
  bargain_hunting?: number
}

// ========== 评估系统类型 ==========

/** 挂牌选手完整信息 */
export interface TransferMarketListingInfo {
  listing_id: number
  window_id: number
  listing_price: number | null
  min_accept_price: number | null
  listing_status: string
  listed_at: string
  sold_at: string | null
  actual_price: number | null
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  calculated_market_value: number
  listed_by_team_id: number
  listed_by_team_name: string
  listed_by_region_code: string | null
  sold_to_team_id: number | null
  sold_to_team_name: string | null
  sold_to_region_code: string | null
}

/** 自由球员信息 */
export interface FreeAgentMarketInfo {
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  calculated_market_value: number
  salary: number
}

/** 转会挂牌市场综合数据 */
export interface TransferMarketData {
  listings: TransferMarketListingInfo[]
  free_agents: FreeAgentMarketInfo[]
  window_status: string | null
  window_id: number | null
  current_round: number | null
  season_id: number
}

// ========== 竞价分析类型 ==========

/** 单条竞价记录 */
export interface TransferBid {
  id: number
  window_id: number
  round: number
  player_id: number
  player_name: string
  player_ability: number
  player_age: number
  player_position: string | null
  from_team_id: number | null
  from_team_name: string | null
  bid_team_id: number
  bid_team_name: string
  bid_team_region_id: number | null
  offered_salary: number
  contract_years: number
  transfer_fee: number
  signing_bonus: number
  match_score: number
  willingness: number
  is_winner: boolean
  reject_reason: string | null
}

/** 单个选手的竞价分析 */
export interface PlayerBidAnalysis {
  player_id: number
  player_name: string
  player_ability: number
  player_age: number
  player_position: string | null
  from_team_id: number | null
  from_team_name: string | null
  round: number
  total_bids: number
  bids: TransferBid[]
  winner_team_name: string | null
  outcome: string
}

/** 解约结果 */
export interface ReleasePlayerResult {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  release_fee: number
  remaining_balance: number
}

/** 竞价总览 */
export interface BidOverview {
  window_id: number
  round: number | null
  total_players: number
  total_bids: number
  successful_signings: number
  failed_signings: number
  avg_bids_per_player: number
  player_analyses: PlayerBidAnalysis[]
}

// ========== 评估系统类型 ==========

/** 战队赛季评估信息 */
export interface TeamSeasonEvaluationInfo {
  evaluation_id: number
  team_id: number
  team_name: string
  team_short_name: string
  region_code: string
  season_id: number
  current_rank: number
  last_rank: number
  spring_rank: number | null
  summer_rank: number | null
  stability_score: number
  strategy: string
  urgency_level: string
  roster_power: number
  roster_count: number
  avg_age: number
  avg_ability: number
  budget_remaining: number
  evaluation_reason: string
  created_at: string
}

/** 位置需求信息 */
export interface PositionNeedInfo {
  position: string
  current_starter_name: string | null
  current_starter_ability: number | null
  current_starter_age: number | null
  need_level: string
  min_ability_target: number | null
  reason: string | null
}

/** 选手挂牌评估信息 */
export interface PlayerListingEvaluationInfo {
  player_id: number
  player_name: string
  position: string
  age: number
  ability: number
  team_id: number
  team_name: string
  should_list: boolean
  list_reason: string
  is_protected: boolean
  protect_reason: string
  estimated_value: number
}

/** 选手留队评估信息 */
export interface PlayerStayEvaluationInfo {
  player_id: number
  player_name: string
  position: string
  age: number
  ability: number
  team_id: number
  team_name: string
  team_short_name: string | null
  region_code: string
  stay_score: number
  wants_to_leave: boolean
  leave_reason: string
  salary: number
  satisfaction: number
  loyalty: number
}

/** 转会系统 API */
export const transferWindowApi = {
  // 开始转会期
  startTransferWindow: () =>
    invokeCommand<TransferWindowResponse>('start_transfer_window'),

  // 执行单轮转会
  executeTransferRound: (windowId: number, round: number) =>
    invokeCommand<RoundExecutionResponse>('execute_transfer_round', { windowId, round }),

  // 快进转会期
  fastForwardTransfer: (windowId: number, fromRound?: number) =>
    invokeCommand<FastForwardResponse>('fast_forward_transfer', { windowId, fromRound }),

  // 获取转会事件
  getTransferEvents: (windowId: number, round?: number, level?: string) =>
    invokeCommand<TransferEvent[]>('get_transfer_events', { windowId, round, level }),

  // 获取转会报告
  getTransferReport: (windowId: number) =>
    invokeCommand<TransferReport>('get_transfer_report', { windowId }),

  // 获取转会期状态
  getTransferWindowStatus: (windowId: number) =>
    invokeCommand<TransferWindowResponse>('get_transfer_window_status', { windowId }),

  // 查询当前赛季的转会窗口（纯查询，不创建）
  getCurrentTransferWindow: () =>
    invokeCommand<TransferWindowResponse | null>('get_current_transfer_window'),

  // 查询指定赛季的转会窗口
  getTransferWindowBySeason: (seasonId: number) =>
    invokeCommand<TransferWindowResponse | null>('get_transfer_window_by_season', { seasonId }),

  // 获取球队AI性格
  getTeamPersonality: (teamId: number) =>
    invokeCommand<TeamPersonalityConfig | null>('get_team_personality', { teamId }),

  // 更新球队AI性格
  updateTeamPersonality: (teamId: number, request: UpdatePersonalityRequest) =>
    invokeCommand<boolean>('update_team_personality', { teamId, request }),

  // 获取球队声望
  getTeamReputation: (teamId: number) =>
    invokeCommand<TeamReputation>('get_team_reputation', { teamId }),

  // ========== 评估系统 ==========

  // 获取战队评估列表
  getTeamEvaluations: (seasonId?: number) =>
    invokeCommand<TeamSeasonEvaluationInfo[]>('get_team_evaluations', { seasonId }),

  // 获取战队位置需求
  getTeamPositionNeeds: (teamId: number, seasonId?: number) =>
    invokeCommand<PositionNeedInfo[]>('get_team_position_needs', { teamId, seasonId }),

  // 获取选手挂牌评估
  getPlayerListingEvaluations: (teamId?: number, seasonId?: number) =>
    invokeCommand<PlayerListingEvaluationInfo[]>('get_player_listing_evaluations', { teamId, seasonId }),

  // 获取选手留队评估
  getPlayerStayEvaluations: (teamId?: number, seasonId?: number) =>
    invokeCommand<PlayerStayEvaluationInfo[]>('get_player_stay_evaluations', { teamId, seasonId }),

  // 清除评估数据（用于重新生成）
  clearEvaluationData: (seasonId?: number) =>
    invokeCommand<number>('clear_evaluation_data', { seasonId }),

  // 获取转会挂牌市场数据
  getTransferMarketListings: () =>
    invokeCommand<TransferMarketData>('get_transfer_market_listings'),

  // ========== 转会窗口关闭 ==========

  // 确认关闭转会窗口（含验证）
  confirmCloseTransferWindow: (windowId: number, force?: boolean) =>
    invokeCommand<TransferWindowCloseValidation>('confirm_close_transfer_window', { windowId, force }),

  // ========== 竞价分析 ==========

  // 获取竞价总览
  getTransferBidsOverview: (windowId?: number, round?: number, seasonId?: number) =>
    invokeCommand<BidOverview>('get_transfer_bids_overview', { windowId, round, seasonId }),

  // 获取单个选手的竞价记录
  getPlayerBids: (windowId: number, playerId: number) =>
    invokeCommand<PlayerBidAnalysis>('get_player_bids', { windowId, playerId }),

  // ========== 解约 ==========

  // 解约选手（支付身价50%作为解约金）
  releasePlayer: (playerId: number) =>
    invokeCommand<ReleasePlayerResult>('release_player', { playerId }),
}
