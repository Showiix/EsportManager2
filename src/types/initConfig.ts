export interface PlayerInitConfig {
  game_id: string
  real_name: string | null
  nationality: string
  position: string // "Top" | "Jug" | "Mid" | "Adc" | "Sup"
  age: number
  ability: number
  potential: number
  is_starter: boolean
}

export interface DraftPoolPlayerInitConfig {
  game_id: string
  real_name: string
  position: string   // "Top" | "Jungle" | "Mid" | "Bot" | "Support"
  ability: number
  potential: number
  stability: number
  age: number
  tag: string         // "Genius" | "Normal" | "Ordinary"
}

export interface TeamInitConfig {
  name: string
  short_name: string
  initial_balance: number // 单位: 元
  players: PlayerInitConfig[]
}

export interface RegionInitConfig {
  id: number
  name: string
  short_name: string
  teams: TeamInitConfig[]
  free_agents: PlayerInitConfig[]
  draft_pool: DraftPoolPlayerInitConfig[]
}

export interface GameInitConfig {
  regions: RegionInitConfig[]
}
