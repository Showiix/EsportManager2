import { invokeCommand } from './client'

// ========================================
// Tournament & Match Management
// ========================================

export interface TournamentMatch {
  id: number
  tournament_id: number
  stage: string
  round: number | null
  match_order: number | null
  format: string
  home_team_id: number | null
  away_team_id: number | null
  home_team_name: string | null
  away_team_name: string | null
  home_score: number
  away_score: number
  winner_id: number | null
  status: string
}

export interface Standing {
  team_id: number
  team_name: string
  wins: number
  losses: number
  round_diff: number
  points: number
  rank: number
}

export const tournamentApi = {
  getTournamentMatches: (tournamentId: number) =>
    invokeCommand<TournamentMatch[]>('get_tournament_matches', { tournamentId }),

  getStandings: (tournamentId: number) =>
    invokeCommand<Standing[]>('get_standings', { tournamentId }),

  simulateNextMatch: (tournamentId: number) =>
    invokeCommand<TournamentMatch | null>('simulate_next_match', { tournamentId }),

  simulateAllMatches: (tournamentId: number) =>
    invokeCommand<TournamentMatch[]>('simulate_all_matches', { tournamentId }),

  // getSchedule is alias for getTournamentMatches
  getSchedule: (tournamentId: number) =>
    invokeCommand<TournamentMatch[]>('get_tournament_matches', { tournamentId }),
}
