import { invokeCommand } from './client'

// ========================================
// 开发工具 API (Development Tools)
// ========================================

/** 数据一致性检查结果 */
export interface ConsistencyCheckResult {
  total_checks: number
  passed: number
  failed: number
  issues: ConsistencyIssue[]
}

export interface ConsistencyIssue {
  category: string
  description: string
  severity: 'warning' | 'error'
}

/** 同步结果 */
export interface SyncResult {
  updated_count: number
  details: string[]
}

/** 游戏状态摘要 */
export interface GameStatusSummary {
  current_season: number
  current_phase: string
  phase_completed: boolean
  team_count: number
  player_count: number
  tournament_count: number
  total_matches: number
  completed_matches: number
  scheduled_matches: number
  honor_count: number
}

/** 开发命令结果 */
export interface DevCommandResult<T> {
  success: boolean
  data: T | null
  message: string
  error: string | null
}

export interface IncompleteMatchInfo {
  match_id: number
  stage: string
  status: string
  tournament_name: string
  tournament_type: string
  home_team: string | null
  away_team: string | null
}

export interface FixStartersResult {
  teams_fixed: number
  players_fixed: number
  details: TeamFixInfo[]
}

export interface TeamFixInfo {
  team_name: string
  fixes: string[]
}

export const devApi = {
  // 荣誉系统
  /** 重新颁发荣誉 */
  reassignHonors: (seasonId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_reassign_honors', { seasonId }),

  /** 重新计算年度积分 */
  recalculateAnnualPoints: (seasonId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_recalculate_annual_points', { seasonId }),

  // 数据修复
  /** 同步选手场次统计 */
  syncPlayerGamesPlayed: (seasonId?: number) =>
    invokeCommand<DevCommandResult<SyncResult>>('dev_sync_player_games_played', { seasonId }),

  /** 重新计算积分榜 */
  recalculateStandings: (tournamentId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_recalculate_standings', { tournamentId }),

  /** 数据一致性检查 */
  checkDataConsistency: () =>
    invokeCommand<DevCommandResult<ConsistencyCheckResult>>('dev_check_data_consistency', {}),

  // 赛事管理
  /** 重置阶段状态 */
  resetPhase: () =>
    invokeCommand<DevCommandResult<void>>('dev_reset_phase', {}),

  /** 获取待模拟比赛数量 */
  simulateAllMatches: (tournamentId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_simulate_all_matches', { tournamentId }),

  // 财务系统
  /** 重新发放赛事奖金 */
  redistributePrizes: (seasonId?: number) =>
    invokeCommand<DevCommandResult<number>>('dev_redistribute_prizes', { seasonId }),

  /** 给所有战队发放测试资金 */
  grantFunds: (amount: number) =>
    invokeCommand<DevCommandResult<number>>('dev_grant_funds', { amount }),

  // 快速测试
  /** 重置存档 */
  resetSave: (keepTeams: boolean) =>
    invokeCommand<DevCommandResult<void>>('dev_reset_save', { keepTeams }),

  /** 获取游戏状态摘要 */
  getGameStatus: () =>
    invokeCommand<DevCommandResult<GameStatusSummary>>('dev_get_game_status', {}),

  /** 检查未完成的比赛 */
  checkIncompleteMatches: (tournamentType?: string) =>
    invokeCommand<DevCommandResult<IncompleteMatchInfo[]>>('dev_check_incomplete_matches', { tournamentType }),

  /** 强制完成比赛 */
  forceCompleteMatch: (matchId: number) =>
    invokeCommand<DevCommandResult<void>>('dev_force_complete_match', { matchId }),

  /** 迁移选手忠诚度和满意度（根据选手属性重新计算） */
  migrateLoyaltySatisfaction: () =>
    invokeCommand<DevCommandResult<number>>('dev_migrate_loyalty_satisfaction'),

  /** 重新计算所有选手身价（使用新公式） */
  recalculateMarketValues: () =>
    invokeCommand<DevCommandResult<number>>('dev_recalculate_market_values'),

  /** 自动修复队伍首发阵容 */
  fixStarters: () =>
    invokeCommand<DevCommandResult<FixStartersResult>>('dev_fix_starters'),
}
