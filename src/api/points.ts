import { invokeCommand } from './client'

// ========================================
// 年度积分 API
// ========================================

/** 队伍年度积分 */
export interface TeamAnnualPoints {
  rank: number
  team_id: number
  team_name: string
  team_short_name: string | null
  region_id: number
  region_code: string
  total_points: number
  tournaments_count: number
}

/** 积分明细 */
export interface AnnualPointsDetail {
  id: number
  save_id: string
  season_id: number
  team_id: number
  tournament_id: number
  tournament_name?: string
  tournament_type?: string
  points: number
  final_rank: number | null
}

export const pointsApi = {
  /** 获取年度积分排名 */
  getRankings: (seasonId?: number) =>
    invokeCommand<TeamAnnualPoints[]>('get_annual_points_ranking', seasonId != null ? { seasonId } : undefined),

  /** 获取队伍的积分明细 */
  getTeamPoints: (teamId: number, seasonId?: number) =>
    invokeCommand<AnnualPointsDetail[]>('get_team_points_detail', { teamId, ...(seasonId != null ? { seasonId } : {}) }),

  /** 获取赛事的积分发放记录 */
  getTournamentPoints: (tournamentId: number) =>
    invokeCommand<AnnualPointsDetail[]>('get_tournament_points', { tournamentId }),

  /** 获取Super资格队伍（Top16） */
  getSuperQualifiedTeams: () =>
    invokeCommand<TeamAnnualPoints[]>('get_super_qualified_teams'),
}
