import { invokeCommand } from './client'

// ========================================
// Draft System
// ========================================

export interface DraftPlayer {
  id: number
  game_id: string
  position: string
  nationality: string | null
  age: number
  ability: number
  potential: number
  tag: string
  draft_rank: number
  is_picked: boolean
}

export interface DraftOrder {
  team_id: number
  team_name: string
  original_team_id: number | null
  summer_rank: number
  draft_position: number
  lottery_result: string | null
}

export interface DraftPick {
  pick_number: number
  team_id: number
  team_name: string
  player: DraftPlayer
}

export interface DraftResultItem {
  pick_number: number
  team_id: number
  team_name: string
  player_id: number
  player_name: string
  position: string
  ability: number
  potential: number
}

export interface DraftRegionStatus {
  status: 'not_started' | 'roster_drawn' | 'lottery_done' | 'completed'
  draft_players: DraftPlayer[]
  draft_results: DraftResultItem[]
  draft_orders: DraftOrder[]
  total_players: number
  picked_count: number
}

export interface DraftPoolPlayer {
  id: number
  game_id: string
  real_name: string | null
  nationality: string | null
  age: number
  ability: number
  potential: number
  position: string
  tag: string
  status: string
}

export interface NewDraftPoolPlayer {
  game_id: string
  real_name?: string | null
  nationality?: string | null
  age: number
  ability: number
  potential: number
  position: string
  tag: string
}

export interface UpdateDraftPoolPlayer {
  gameId: string
  ability: number
  potential: number
  position: string
  tag: string
}

export const draftApi = {
  generateDraftPool: (regionId: number, poolSize?: number) =>
    invokeCommand<DraftPlayer[]>('generate_draft_pool', {
      regionId,
      poolSize: poolSize ?? 14
    }),

  runDraftLottery: (regionId: number) =>
    invokeCommand<DraftOrder[]>('run_draft_lottery', { regionId }),

  getDraftOrder: (regionId: number) =>
    invokeCommand<DraftOrder[]>('get_draft_order', { regionId }),

  getAvailableDraftPlayers: (regionId: number) =>
    invokeCommand<DraftPlayer[]>('get_available_draft_players', { regionId }),

  makeDraftPick: (regionId: number, teamId: number, playerId: number) =>
    invokeCommand<DraftPick>('make_draft_pick', { regionId, teamId, playerId }),

  aiAutoDraft: (regionId: number) =>
    invokeCommand<DraftPick[]>('ai_auto_draft', { regionId }),

  // 选手池管理
  getDraftPoolPlayers: (regionId: number) =>
    invokeCommand<DraftPoolPlayer[]>('get_draft_pool_players', { regionId }),

  addDraftPoolPlayers: (regionId: number, players: NewDraftPoolPlayer[]) =>
    invokeCommand<number>('add_draft_pool_players', { regionId, players }),

  updateDraftPoolPlayer: (playerId: number, data: UpdateDraftPoolPlayer) =>
    invokeCommand<void>('update_draft_pool_player', { playerId, ...data }),

  deleteDraftPoolPlayers: (regionId: number, playerIds?: number[]) =>
    invokeCommand<number>('delete_draft_pool_players', { regionId, playerIds }),

  generateRookies: (regionId: number, count?: number, seed?: number, abilityMin?: number, abilityMax?: number) =>
    invokeCommand<DraftPoolPlayer[]>('generate_rookies', { regionId, count, seed, abilityMin, abilityMax }),

  // 选秀状态查询
  getDraftRegionStatus: (regionId: number, seasonId?: number) =>
    invokeCommand<DraftRegionStatus>('get_draft_region_status', { regionId, seasonId }),
}
