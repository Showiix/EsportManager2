import { invokeCommand } from './client'

// ========================================
// Time Progression System API
// ========================================

/** 阶段状态 */
export type PhaseStatus = 'NOT_INITIALIZED' | 'IN_PROGRESS' | 'COMPLETED'

/** 可用的时间操作 */
export type TimeAction =
  | 'INITIALIZE_PHASE'
  | 'SIMULATE_NEXT_MATCH'
  | 'SIMULATE_ALL_MATCHES'
  | 'COMPLETE_AND_ADVANCE'
  | 'FAST_FORWARD_PHASE'
  | 'FAST_FORWARD_TO_SUMMER'
  | 'FAST_FORWARD_TO_WORLDS'
  | 'FAST_FORWARD_TO_SEASON_END'
  | 'START_TRANSFER_WINDOW'
  | 'EXECUTE_TRANSFER_ROUND'
  | 'START_DRAFT'
  | 'START_NEW_SEASON'

/** 赛事进度 */
export interface TournamentProgress {
  tournament_id: number
  tournament_name: string
  region: string | null
  total_matches: number
  completed_matches: number
  status: string
}

/** 阶段进度 */
export interface PhaseProgress {
  tournaments: TournamentProgress[]
  total_matches: number
  completed_matches: number
  percentage: number
}

/** 阶段信息 */
export interface PhaseInfo {
  phase: string
  display_name: string
  status: string // "completed" | "current" | "upcoming"
  index: number
}

/** 赛季进度 */
export interface SeasonProgress {
  phases: PhaseInfo[]
  current_phase_index: number
  total_phases: number
  percentage: number
}

/** 荣誉信息 */
export interface HonorInfo {
  honor_type: string
  recipient_name: string
  tournament_name: string
}

/** 游戏时间状态 - 统一的时间状态返回结构 */
export interface GameTimeState {
  save_id: string
  current_season: number
  current_phase: string
  phase_display_name: string
  phase_status: PhaseStatus
  phase_progress: PhaseProgress
  season_progress: SeasonProgress
  available_actions: TimeAction[]
  can_advance: boolean
  next_phase: string | null
}

/** 完成并推进结果 */
export interface CompleteAndAdvanceResult {
  success: boolean
  completed_phase: string
  new_phase: string | null
  honors_awarded: HonorInfo[]
  message: string
  new_time_state: GameTimeState
}

/** 快进结果 */
export interface FastForwardResult {
  success: boolean
  start_phase: string
  end_phase: string
  phases_advanced: number
  matches_simulated: number
  message: string
  skipped_phases?: string[]
}

/** 新赛季初始化结果 */
export interface NewSeasonResult {
  new_season: number
  starters_confirmed: number
  message: string
}

/** 单场模拟结果 */
export interface SimulateNextResult {
  match_id: number
  tournament_name: string
  home_team_name: string
  away_team_name: string
  home_score: number
  away_score: number
  winner_name: string
  remaining_matches: number
  phase_completed: boolean
}

/** 修复赛事状态结果 */
export interface FixTournamentStatusResult {
  fixed_count: number
  fixed_tournaments: string[]
  message: string
}

export const timeApi = {
  /** 获取完整的游戏时间状态 */
  getTimeState: () =>
    invokeCommand<GameTimeState>('get_time_state'),

  /** 初始化当前阶段（创建赛事） */
  initPhase: () =>
    invokeCommand<string>('time_init_phase'),

  /** 完成当前阶段并推进到下一阶段 */
  completeAndAdvance: () =>
    invokeCommand<CompleteAndAdvanceResult>('complete_and_advance'),

  /** 快进到指定目标 */
  fastForwardTo: (target: string) =>
    invokeCommand<FastForwardResult>('fast_forward_to', { target }),

  /** 模拟所有当前阶段的比赛 */
  simulateAll: () =>
    invokeCommand<number>('time_simulate_all'),

  /** 模拟下一场比赛 */
  simulateNext: () =>
    invokeCommand<SimulateNextResult>('time_simulate_next'),

  /** 开始新赛季 */
  startNewSeason: () =>
    invokeCommand<NewSeasonResult>('time_start_new_season'),

  /** 修复赛事状态 - 将已完成的赛事状态更新为 Completed */
  fixTournamentStatus: () =>
    invokeCommand<FixTournamentStatusResult>('fix_tournament_status'),
}
