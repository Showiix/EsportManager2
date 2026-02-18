import { invoke } from '@tauri-apps/api/core'

export interface LadderTournamentInfo {
  id: number
  save_id: string
  season: number
  event_type: string
  event_name: string
  edition: number
  total_rounds: number
  current_round: number
  status: string
}

export interface LadderRankingEntry {
  rank: number
  player_id: number
  player_name: string
  game_id: string
  position: string
  team_name: string | null
  rating: number
  games_played: number
  wins: number
  losses: number
  win_rate: number
  mvp_count: number
  avg_influence: number
  max_rating: number
}

export interface LadderPlayerInfo {
  player_id: number
  player_name: string
  game_id: string
  position: string
  team_name: string | null
  rating: number
}

export interface LadderMatchInfo {
  id: number
  round_number: number
  match_number: number
  blue_team: LadderPlayerInfo[]
  red_team: LadderPlayerInfo[]
  blue_avg_rating: number
  red_avg_rating: number
  winner_side: string | null
  mvp_player_name: string | null
}

export interface PlayerReward {
  player_id: number
  player_name: string
  rank: number
  rating: number
  ability_gain: number
  trait_unlocked: string | null
}

export interface LadderCompletionResult {
  total_players: number
  rewards_distributed: PlayerReward[]
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

async function invokeCmd<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const result = await invoke<ApiResponse<T>>(cmd, args)
  if (result.success && result.data !== undefined) {
    return result.data
  }
  throw new Error(result.error || `Command ${cmd} failed`)
}

export async function simulateLadderRound(
  tournamentId: number
): Promise<string> {
  return invokeCmd<string>('simulate_ladder_round', { tournamentId })
}

export async function getLadderRankings(
  tournamentId: number
): Promise<LadderRankingEntry[]> {
  return invokeCmd<LadderRankingEntry[]>('get_ladder_rankings', { tournamentId })
}

export async function getLadderMatches(
  tournamentId: number,
  roundNumber?: number
): Promise<LadderMatchInfo[]> {
  return invokeCmd<LadderMatchInfo[]>('get_ladder_matches', {
    tournamentId,
    roundNumber: roundNumber ?? null
  })
}

export async function completeLadderTournament(
  tournamentId: number
): Promise<LadderCompletionResult> {
  return invokeCmd<LadderCompletionResult>('complete_ladder_tournament', { tournamentId })
}

export interface LadderMatchDetail {
  id: number
  round_number: number
  match_number: number
  blue_team: LadderPlayerInfo[]
  red_team: LadderPlayerInfo[]
  blue_avg_rating: number
  red_avg_rating: number
  blue_power: number
  red_power: number
  winner_side: string | null
  mvp_player_id: number | null
  mvp_player_name: string | null
  game_duration: number | null
  performances: Record<string, number> | null
  draft_result_json: string | null
  rating_changes: Record<string, number> | null
}

export async function getLadderMatchDetail(
  matchId: number
): Promise<LadderMatchDetail> {
  return invokeCmd<LadderMatchDetail>('get_ladder_match_detail', { matchId })
}

export async function getLadderTournaments(
  seasonId: number
): Promise<LadderTournamentInfo[]> {
  try {
    return await invokeCmd<LadderTournamentInfo[]>('get_ladder_tournaments', { seasonId })
  } catch {
    return []
  }
}

export interface RatingHistoryPoint {
  round: number
  rating: number
  change: number
}

export async function getPlayerLadderRatingHistory(
  tournamentId: number,
  playerId: number
): Promise<RatingHistoryPoint[]> {
  return invokeCmd<RatingHistoryPoint[]>('get_player_ladder_rating_history', { tournamentId, playerId })
}
