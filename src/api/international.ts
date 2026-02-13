import { invokeCommand } from './client'
import type { TournamentInfo } from './query'

// ========================================
// International Tournaments
// ========================================

export interface BracketInfo {
  tournament_id: number
  tournament_name: string
  tournament_type: string
  stages: StageInfo[]
  matches: MatchBracketInfo[]
}

export interface StageInfo {
  name: string
  display_name: string
  order: number
  total_matches: number
  completed_matches: number
}

export interface MatchBracketInfo {
  match_id: number
  stage: string
  match_order: number
  format: string
  home_team: TeamBracketInfo | null
  away_team: TeamBracketInfo | null
  home_score: number
  away_score: number
  winner_id: number | null
  status: string
}

export interface TeamBracketInfo {
  id: number
  name: string
  short_name: string | null
  region_code: string
}

export interface SwissRoundStatus {
  current_round: number
  teams: SwissTeamStatus[]
  completed: boolean
  qualified_teams: number[]
  eliminated_teams: number[]
}

export interface SwissTeamStatus {
  team_id: number
  team_name: string
  wins: number
  losses: number
  is_qualified: boolean
  is_eliminated: boolean
}

export interface GroupStandingInfo {
  group_name: string
  teams: TeamGroupStats[]
}

export interface TeamGroupStats {
  team_id: number
  team_name: string
  region_code: string
  wins: number
  losses: number
  games_won: number
  games_lost: number
  points: number
}

export const internationalApi = {
  createMsiTournament: (
    legendaryTeamIds: number[],
    challengerTeamIds: number[],
    qualifierTeamIds: number[]
  ) => invokeCommand<number>('create_msi_tournament', {
    legendaryTeamIds,
    challengerTeamIds,
    qualifierTeamIds
  }),

  createWorldsTournament: (directTeamIds: number[], groupTeamIds: number[]) =>
    invokeCommand<number>('create_worlds_tournament', {
      directTeamIds,
      groupTeamIds
    }),

  createMastersTournament: (tournamentType: string, teamIds: number[]) =>
    invokeCommand<number>('create_masters_tournament', {
      tournamentType,
      teamIds
    }),

  createSuperTournament: (
    legendaryTeamIds: number[],
    challengerTeamIds: number[],
    fighterTeamIds: number[]
  ) => invokeCommand<number>('create_super_tournament', {
    legendaryTeamIds,
    challengerTeamIds,
    fighterTeamIds
  }),

  getTournamentBracket: (tournamentId: number) =>
    invokeCommand<BracketInfo>('get_tournament_bracket', { tournamentId }),

  advanceBracket: (tournamentId: number, completedMatchId: number, winnerId: number) =>
    invokeCommand<number[]>('advance_bracket', {
      tournamentId,
      completedMatchId,
      winnerId
    }),

  getSwissRoundStatus: (tournamentId: number) =>
    invokeCommand<SwissRoundStatus>('get_swiss_round_status', { tournamentId }),

  generateNextSwissRound: (tournamentId: number) =>
    invokeCommand<number[]>('generate_next_swiss_round', { tournamentId }),

  // 填充世界赛淘汰赛对阵（瑞士轮完成后调用）
  fillWorldsKnockoutBracket: (tournamentId: number, qualifiedTeamIds: number[]) =>
    invokeCommand<number[]>('fill_worlds_knockout_bracket', { tournamentId, qualifiedTeamIds }),

  // 清理重复赛事
  cleanupDuplicateTournaments: (tournamentType: string) =>
    invokeCommand<number>('cleanup_duplicate_tournaments', { tournamentType }),

  // ICP洲际对抗赛
  createIcpTournament: (regionTeams: number[][]) =>
    invokeCommand<number>('create_icp_tournament', { regionTeams }),

  // 小组赛积分榜
  getGroupStandings: (tournamentId: number) =>
    invokeCommand<GroupStandingInfo[]>('get_group_standings', { tournamentId }),

  // 生成淘汰赛对阵
  generateKnockoutBracket: (tournamentId: number) =>
    invokeCommand<number[]>('generate_knockout_bracket', { tournamentId }),

  // 生成Super赛事第三阶段（冠军预备战）
  generateChampionPrepStage: (tournamentId: number) =>
    invokeCommand<number[]>('generate_champion_prep_stage', { tournamentId }),

  // 生成Super赛事第四阶段（终极冠军赛）
  generateFinalStage: (tournamentId: number) =>
    invokeCommand<number[]>('generate_final_stage', { tournamentId }),

  // 完成赛事 - 处理荣誉殿堂和年度积分
  completeTournament: (tournamentId: number) =>
    invokeCommand<TournamentCompletionResult>('complete_tournament', { tournamentId }),

  // 获取MSI参赛队伍分组（基于春季季后赛结果）
  getMsiQualifiedTeams: (seasonId: number) =>
    invokeCommand<MsiTeamGroups>('get_msi_qualified_teams', { seasonId }),

  // 重新生成MSI对阵（当队伍就绪但比赛未生成时使用）
  regenerateMsiBracket: (tournamentId: number) =>
    invokeCommand<number>('regenerate_msi_bracket', { tournamentId }),

  // 根据类型获取赛事列表
  getTournamentsByType: (tournamentType: string, seasonId: number) =>
    invokeCommand<TournamentInfo[]>('get_tournaments_by_type', { tournamentType, seasonId }),

  // 获取上海大师赛参赛队伍分组（基于夏季季后赛结果）
  getShanghaiQualifiedTeams: (seasonId: number) =>
    invokeCommand<MsiTeamGroups>('get_shanghai_qualified_teams', { seasonId }),

  // 重新生成上海大师赛对阵（删除现有比赛并重新初始化）
  regenerateShanghairacket: (tournamentId: number) =>
    invokeCommand<number>('regenerate_shanghai_bracket', { tournamentId }),

  // 重新生成ICP洲际对抗赛对阵（删除现有比赛并重新初始化）
  regenerateIcpBracket: (tournamentId: number) =>
    invokeCommand<number>('regenerate_icp_bracket', { tournamentId }),
}

// MSI参赛队伍分组信息
export interface MsiTeamGroups {
  legendary: MsiTeamInfo[]
  challenger: MsiTeamInfo[]
  qualifier: MsiTeamInfo[]
}

// MSI队伍信息
export interface MsiTeamInfo {
  team_id: number
  team_name: string
  short_name: string
  region_id: number
  region_name: string
}

// 赛事完成结果
export interface TournamentCompletionResult {
  tournament_id: number
  tournament_name: string
  honors_awarded: HonorAwardedInfo[]
  points_awarded: PointsAwardedInfo[]
  message: string
}

// 颁发的荣誉信息
export interface HonorAwardedInfo {
  honor_type: string
  recipient_name: string
  recipient_type: string // "team" or "player"
}

// 颁发的积分信息
export interface PointsAwardedInfo {
  team_id: number
  team_name: string
  points: number
  position: string
}
