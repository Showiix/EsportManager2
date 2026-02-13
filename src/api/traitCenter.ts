import { invokeCommand } from './client'
import type { TraitInfo } from './player'

// ========================================
// Trait Center (特性中心)
// ========================================

export interface PlayerTraitEntry {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  region: string
  position: string
  ability: number
  age: number
  traits: TraitInfo[]
}

export interface TraitCatalogEntry {
  trait_type: string
  name: string
  description: string
  rarity: number
  is_negative: boolean
  category: string
  awakening_conditions: string
  decay_conditions: string
}

export interface TeamSynergyInfo {
  team_id: number
  team_name: string
  avg_tenure: number
  synergy_bonus: number
  players: PlayerSynergyDetail[]
}

export interface PlayerSynergyDetail {
  player_id: number
  player_name: string
  position: string
  join_season: number
  tenure: number
}

export const traitCenterApi = {
  getAllPlayerTraits: (region?: string) =>
    invokeCommand<PlayerTraitEntry[]>('get_all_player_traits', { region: region ?? null }),

  getTraitCatalog: () =>
    invokeCommand<TraitCatalogEntry[]>('get_trait_catalog'),

  getTeamSynergy: (teamId: number) =>
    invokeCommand<TeamSynergyInfo>('get_team_synergy', { teamId }),
}
