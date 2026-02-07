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
}

export interface GameInitConfig {
  regions: RegionInitConfig[]
}
