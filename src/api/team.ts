import { invokeCommand } from './client'
import type { Player } from './player'

// ========================================
// Team Management
// ========================================

export interface Team {
  id: number
  region_id: number
  name: string
  short_name: string | null
  power_rating: number
  total_matches: number
  wins: number
  win_rate: number
  annual_points: number
  cross_year_points: number
  balance: number
}

export interface TeamRoster {
  team: Team
  starters: Player[]
  substitutes: Player[]
}

export const teamApi = {
  getTeamsByRegion: (regionId: number) =>
    invokeCommand<Team[]>('get_teams_by_region', { regionId }),

  getAllTeams: () =>
    invokeCommand<Team[]>('get_all_teams'),

  getAllPlayers: () =>
    invokeCommand<Player[]>('get_all_players'),

  getTeam: (teamId: number) =>
    invokeCommand<Team>('get_team', { teamId }),

  getTeamRoster: (teamId: number) =>
    invokeCommand<TeamRoster>('get_team_roster', { teamId }),

  getTeamStarters: (teamId: number) =>
    invokeCommand<Player[]>('get_team_starters', { teamId }),

  setStarter: (teamId: number, playerId: number, isStarter: boolean) =>
    invokeCommand<void>('set_starter', { teamId, playerId, isStarter }),

  updateTeam: (request: { team_id: number; name?: string; short_name?: string }) =>
    invokeCommand<Team>('update_team', { request }),
}
