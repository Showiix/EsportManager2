import { invokeCommand } from './client'

// ========================================
// Transfer Market
// ========================================

export interface TransferListing {
  id: number
  player_id: number
  player_name: string
  position: string
  ability: number
  potential: number
  seller_team_id: number
  seller_team_name: string
  asking_price: number
  listing_type: string
  status: string
  listed_at: string
}

export interface FreeAgent {
  id: number
  name: string
  position: string
  nationality: string
  age: number
  ability: number
  potential: number
  tag: string
  expected_salary: number
}

export interface TransferRecord {
  id: number
  player_id: number
  player_name: string
  from_team_id: number | null
  from_team_name: string | null
  to_team_id: number
  to_team_name: string
  transfer_type: string
  fee: number
  salary: number
  contract_years: number
  transferred_at: string
}

export const transferApi = {
  getTransferMarket: () =>
    invokeCommand<TransferListing[]>('get_transfer_market'),

  getFreeAgents: () =>
    invokeCommand<FreeAgent[]>('get_free_agents'),

  listPlayerForTransfer: (teamId: number, playerId: number, askingPrice: number) =>
    invokeCommand<TransferListing>('list_player_for_transfer', {
      teamId,
      playerId,
      askingPrice
    }),

  cancelTransferListing: (listingId: number) =>
    invokeCommand<void>('cancel_transfer_listing', { listingId }),

  buyListedPlayer: (listingId: number, buyerTeamId: number, contractYears: number, salary: number) =>
    invokeCommand<TransferRecord>('buy_listed_player', {
      listingId,
      buyerTeamId,
      contractYears,
      salary
    }),

  signFreeAgent: (playerId: number, teamId: number, contractYears: number, salary: number) =>
    invokeCommand<TransferRecord>('sign_free_agent', {
      playerId,
      teamId,
      contractYears,
      salary
    }),

  getTransferHistory: (teamId?: number) =>
    invokeCommand<TransferRecord[]>('get_transfer_history', { teamId }),

  // ========== AI 转会窗口 API ==========

  // 开始转会窗口
  startTransferWindow: () =>
    invokeCommand<TransferWindowInfo>('start_transfer_window'),

  // 执行下一轮转会
  executeTransferRound: () =>
    invokeCommand<TransferRoundInfo>('execute_transfer_round'),

  // 快进完成所有转会
  fastForwardTransfers: () =>
    invokeCommand<TransferWindowInfo>('fast_forward_transfers'),

  // 获取转会窗口状态
  getTransferWindowStatus: () =>
    invokeCommand<TransferWindowInfo>('get_transfer_window_status'),

  // 获取转会事件列表
  getTransferEvents: (round?: number) =>
    invokeCommand<TransferEventInfo[]>('get_transfer_events', { round }),

  // ========== 市场分析和选手市场 API ==========

  // 获取球队转会计划列表
  getTeamTransferPlans: () =>
    invokeCommand<TeamTransferPlanInfo[]>('get_team_transfer_plans'),

  // 获取选手市场列表
  getPlayerMarketList: () =>
    invokeCommand<PlayerMarketInfo[]>('get_player_market_list'),

  // 获取选手合同详情
  getPlayerContractDetail: (playerId: number) =>
    invokeCommand<PlayerContractDetail>('get_player_contract_detail', { playerId }),
}

// AI 转会窗口类型定义
export interface TransferWindowInfo {
  id: number
  season_id: number
  status: string  // 'PREPARING' | 'IN_PROGRESS' | 'COMPLETED'
  current_round: number
  total_rounds: number
  total_transfers: number
  total_fees: number
  free_agents_signed: number
  retirements: number
  contract_expires: number
  started_at: string | null
  completed_at: string | null
}

export interface TransferEventInfo {
  id: number
  round: number
  event_type: string  // 'FREE_AGENT' | 'PURCHASE' | 'RETIREMENT' | 'CONTRACT_EXPIRE' | 'TRANSFER_REQUEST' | 'CONTRACT_RENEWAL' | 'RENEWAL_FAILED' | 'STAR_POACHED' | 'LOYALTY_STAY' | 'REBUILD_SALE'
  status: string
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  market_value: number
  from_team_id: number | null
  from_team_name: string | null
  to_team_id: number | null
  to_team_name: string | null
  transfer_fee: number
  new_salary: number | null
  contract_years: number | null
  contract_type: string
  price_ratio: number | null
  headline: string
  description: string
  importance: string  // 'BREAKING' | 'MAJOR' | 'NORMAL' | 'MINOR'
  competing_teams: number[]
  was_bidding_war: boolean
  created_at: string | null
}

export interface TransferRoundInfo {
  round: number
  round_name: string
  events_count: number
  transfers_count: number
  total_fees: number
  summary: string
  events: TransferEventInfo[]
}

// 选手满意度/忠诚度状态信息
export interface PlayerStatusInfo {
  player_id: number
  player_name: string
  satisfaction: number  // 0-100
  satisfaction_trend: number  // 相比上赛季的变化
  loyalty: number  // 0-100
  loyalty_type: string  // '忠心耿耿' | '忠诚' | '中立' | '机会主义' | '雇佣兵'
  wants_to_leave: boolean
  departure_reasons: string[]
}

// 想离队选手信息 (用于旧版选手市场)
export interface PlayerStatusDepartureInfo {
  player: PlayerStatusInfo
  team_id: number
  team_name: string
  market_value: number
  primary_reason: string
}

// ========================================
// 市场分析和选手市场类型定义
// ========================================

// 球队转会计划信息
export interface TeamTransferPlanInfo {
  team_id: number
  team_name: string
  region_code: string
  // 财务
  balance: number
  financial_status: string  // 'Wealthy' | 'Healthy' | 'Struggling' | 'Bankrupt'
  transfer_budget: number
  salary_space: number
  current_total_salary: number
  // 阵容
  roster_count: number
  avg_ability: number
  avg_age: number
  // 位置需求 (0-100)
  position_needs: Record<string, number>
  // 策略
  strategy: string  // 'AggressiveBuy' | 'Passive' | 'MustSell' | 'ForceClear' | 'FullRebuild' | 'StarHunting'
  ambition: string  // 'Championship' | 'Playoff' | 'Rebuild'
  // 标记
  must_sign: boolean
  must_clear: boolean
}

// 选手市场信息
export interface PlayerMarketInfo {
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  // 战队信息
  team_id: number | null
  team_name: string | null
  team_short_name: string | null
  region_code: string | null
  // 合同信息
  salary: number
  contract_end_season: number | null
  join_season: number | null
  // 身价信息
  base_market_value: number
  calculated_market_value: number
  // 状态信息
  satisfaction: number
  loyalty: number
  is_starter: boolean
  status: string
}

// 忠诚度变化记录
export interface LoyaltyChangeInfo {
  season_id: number
  change_amount: number
  reason: string
}

// 身价变化记录
export interface MarketValueChangeInfo {
  season_id: number
  old_value: number
  new_value: number
  change_amount: number
  change_percent: number
  reason: string
}

// 选手合同详情
export interface PlayerContractDetail {
  // 基础信息
  player_id: number
  player_name: string
  position: string
  age: number
  ability: number
  potential: number
  stability: number
  // 战队
  team_id: number | null
  team_name: string | null
  region_code: string | null
  // 合同
  salary: number
  contract_end_season: number | null
  join_season: number
  years_in_team: number
  // 身价详情
  base_market_value: number
  honor_factor: number
  region_factor: number
  calculated_market_value: number
  // 满意度详情
  satisfaction: number
  // 忠诚度详情
  loyalty: number
  loyalty_type: string
  departure_threshold: number
  loyalty_price_factor: number
  wants_to_leave: boolean
  departure_reasons: string[]
  // 历史
  market_value_history: MarketValueChangeInfo[]
  loyalty_changes: LoyaltyChangeInfo[]
}
