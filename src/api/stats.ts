import { invokeCommand } from './client'

// ========================================
// Player Stats (Data Center)
// ========================================

export interface PlayerSeasonStatistics {
  id: number | null
  save_id: string
  player_id: number
  player_name: string
  season_id: number
  team_id: number | null
  region_id: string | null
  position: string
  matches_played: number
  games_played: number
  total_impact: number
  avg_impact: number
  avg_performance: number
  best_performance: number
  worst_performance: number
  consistency_score: number
  international_titles: number
  regional_titles: number
  champion_bonus: number
  yearly_top_score: number
}

export interface PlayerRankingItem {
  player_id: number
  player_name: string
  team_id: number | null
  position: string
  region_id: string | null
  games_played: number
  avg_impact: number
  avg_performance: number
  consistency_score: number
  champion_bonus: number
  yearly_top_score: number
  big_stage_score: number
  has_international: boolean
}

export interface PlayerTournamentStats {
  id: number
  save_id: string
  season_id: number
  tournament_id: number
  tournament_type: string
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  games_played: number
  games_won: number
  total_impact: number
  avg_impact: number
  max_impact: number
  avg_performance: number
  best_performance: number
  game_mvp_count: number
  created_at?: string
  updated_at?: string
}

export interface RecordPerformanceParams {
  player_id: number
  player_name: string
  team_id: number
  position: string
  impact_score: number
  actual_ability: number
  season_id: number
  region_id?: string
}

export interface PlayerContractRecord {
  season: string
  event_type: string
  team_name: string
  salary: number
  contract_years: number
  transfer_fee: number
  reason: string | null
}

export interface PlayerSeasonHistoryEntry {
  season: string
  team_name: string
  ability: number
  potential: number
}

export interface PlayerTournamentHistoryItem {
  tournament_type: string
  season_id: number
  games_played: number
  avg_impact: number
  max_impact: number
  avg_performance: number
}

export interface PlayerYearlyTopItem {
  season: string
  yearly_top_score: number
  rank: number
  total_players: number
}

// Market value change record
export interface MarketValueChange {
  id: number
  season_id: number
  player_id: number
  player_name: string
  old_value: number
  new_value: number
  change_amount: number
  change_percent: number
  reason: string
  created_at: string
}

export const statsApi = {
  // Record a single player performance
  recordPerformance: (params: RecordPerformanceParams) =>
    invokeCommand<PlayerSeasonStatistics>('record_player_performance', { params }),

  // Batch record player performances
  batchRecordPerformance: (performances: RecordPerformanceParams[]) =>
    invokeCommand<number>('batch_record_player_performance', { performances }),

  // Record championship (for all players in a team)
  recordChampionship: (teamId: number, isInternational: boolean, seasonId: number) =>
    invokeCommand<number>('record_championship', { teamId, isInternational, seasonId }),

  // Get season impact ranking
  getSeasonImpactRanking: (seasonId: number, limit?: number) =>
    invokeCommand<PlayerRankingItem[]>('get_season_impact_ranking', { seasonId, limit }),

  // Get position ranking
  getPositionRanking: (seasonId: number, position: string, limit?: number) =>
    invokeCommand<PlayerRankingItem[]>('get_position_ranking', { seasonId, position, limit }),

  // Get player stats
  getPlayerStats: (playerId: number, seasonId?: number) =>
    invokeCommand<PlayerSeasonStatistics[]>('get_player_stats', { playerId, seasonId }),

  // Get team player stats
  getTeamPlayerStats: (teamId: number, seasonId: number) =>
    invokeCommand<PlayerSeasonStatistics[]>('get_team_player_stats', { teamId, seasonId }),

  // Clear season stats
  clearSeasonStats: (seasonId: number) =>
    invokeCommand<boolean>('clear_season_stats', { seasonId }),

  // Get player impact history (real data from match performances)
  getPlayerImpactHistory: (playerId: number, seasonId?: number) =>
    invokeCommand<number[]>('get_player_impact_history', { playerId, seasonId }),

  // Get tournament MVP ranking (sorted by MVP count)
  getTournamentMvpRanking: (tournamentId: number, limit?: number) =>
    invokeCommand<PlayerTournamentStats[]>('get_tournament_mvp_ranking', { tournamentId, limit }),

  // Recalculate yearly scores for all players (use new formula: 50% impact + 50% champion bonus)
  recalculateYearlyScores: (seasonId: number) =>
    invokeCommand<number>('recalculate_yearly_scores', { seasonId }),

  // Get player market value changes history
  getPlayerMarketValueChanges: (playerId: number) =>
    invokeCommand<MarketValueChange[]>('get_player_market_value_changes', { playerId }),

  getPlayerSeasonHistory: (playerId: number) =>
    invokeCommand<PlayerSeasonHistoryEntry[]>('get_player_season_history', { playerId }),

  getPlayerContractHistory: (playerId: number) =>
    invokeCommand<PlayerContractRecord[]>('get_player_contract_history', { playerId }),

  getPlayerTournamentHistory: (playerId: number, seasonId?: number) =>
    invokeCommand<PlayerTournamentHistoryItem[]>('get_player_tournament_history', { playerId, seasonId }),

  getPlayerYearlyTopHistory: (playerId: number) =>
    invokeCommand<PlayerYearlyTopItem[]>('get_player_yearly_top_history', { playerId }),
}
