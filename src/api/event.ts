import { invokeCommand } from './client'

// ========================================
// Event System
// ========================================

export interface SeasonSettlementPreview {
  retiring_players: RetiringPlayer[]
  expiring_contracts: ExpiringContract[]
  age_updates: AgeUpdate[]
}

export interface RetiringPlayer {
  player_id: number
  player_name: string
  team_id: number | null
  age: number
  ability: number
  reason: string
  reason_description: string
}

export interface ExpiringContract {
  player_id: number
  player_name: string
  team_id: number | null
  age: number
  ability: number
  contract_end_season: number | null
  salary: number
}

export interface AgeUpdate {
  player_id: number
  player_name: string
  old_age: number
  new_age: number
}

export interface SeasonEvent {
  id: number
  event_type: string
  player_id: number | null
  player_name: string | null
  team_id: number | null
  team_name: string | null
  description: string
  season_id: number
  created_at: string
}

export const eventApi = {
  previewSeasonSettlement: () =>
    invokeCommand<SeasonSettlementPreview>('preview_season_settlement'),

  executeSeasonSettlement: () =>
    invokeCommand<SeasonEvent[]>('execute_season_settlement'),

  getSeasonEvents: (seasonId: number) =>
    invokeCommand<SeasonEvent[]>('get_season_events', { seasonId }),

  getPlayerEvents: (playerId: number) =>
    invokeCommand<SeasonEvent[]>('get_player_events', { playerId }),

  getEventsByType: (eventType: string) =>
    invokeCommand<SeasonEvent[]>('get_events_by_type', { eventType }),

  updatePlayersAge: () =>
    invokeCommand<AgeUpdate[]>('update_players_age'),

  getRetiringCandidates: () =>
    invokeCommand<RetiringPlayer[]>('get_retiring_candidates'),

  getExpiringContracts: (currentSeason: number) =>
    invokeCommand<ExpiringContract[]>('get_expiring_contracts', { currentSeason }),
}
