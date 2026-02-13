import { invokeCommand } from './client'
import type { Team } from './team'
import type { Player } from './player'
import type { HonorRecord } from './honor'

// ========================================
// Query System
// ========================================

export interface Region {
  id: number
  code: string
  name: string
  team_count: number
}

export interface RegionDetail {
  region: Region
  teams: Team[]
  current_tournament: TournamentInfo | null
}

export interface TournamentInfo {
  id: number
  name: string
  tournament_type: string
  season_id: number
  region_id: number | null
  status: string
  champion_team_id?: number | null
  champion_team_name?: string | null
  match_count: number
  completed_matches: number
}

export interface SeasonOverview {
  season_id: number
  tournaments: TournamentInfo[]
  top_teams: Team[]
  recent_champions: HonorRecord[]
}

export interface SearchResult {
  teams: Team[]
  players: Player[]
}

export const queryApi = {
  getAllRegions: () =>
    invokeCommand<Region[]>('get_all_regions'),

  getRegionDetail: (regionId: number) =>
    invokeCommand<RegionDetail>('get_region_detail', { regionId }),

  getSeasonTournaments: (seasonId: number) =>
    invokeCommand<TournamentInfo[]>('get_season_tournaments', { seasonId }),

  getRegionTournaments: (regionId: number, seasonId?: number) =>
    invokeCommand<TournamentInfo[]>('get_region_tournaments', { regionId, seasonId }),

  getTournamentDetail: (tournamentId: number) =>
    invokeCommand<TournamentInfo>('get_tournament_detail', { tournamentId }),

  getInternationalTournaments: (seasonId?: number) =>
    invokeCommand<TournamentInfo[]>('get_international_tournaments', { seasonId }),

  getSeasonOverview: (seasonId: number) =>
    invokeCommand<SeasonOverview>('get_season_overview', { seasonId }),

  searchTeams: (query: string) =>
    invokeCommand<Team[]>('search_teams', { query }),

  searchPlayers: (query: string) =>
    invokeCommand<Player[]>('search_players', { query }),

  getTeamsByRegion: (regionId: number) =>
    invokeCommand<Team[]>('get_teams_by_region', { regionId }),
}
