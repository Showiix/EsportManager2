import { invokeCommand } from './client'

// ========================================
// Player Management
// ========================================

export interface Player {
  id: number
  game_id: string
  real_name: string | null
  team_id: number | null
  position: string | null
  nationality: string | null
  age: number
  ability: number
  potential: number
  stability: number
  salary: number
  market_value: number
  calculated_market_value: number  // 计算后的身价（含荣誉和赛区系数）
  contract_end_season: number | null
  join_season: number | null
  status: string
  tag: string
  is_starter: boolean
  satisfaction: number  // 满意度 0-100
  loyalty: number  // 忠诚度 0-100
}

// 特性信息
export interface TraitInfo {
  trait_type: string
  name: string
  description: string
  rarity: number  // 1-5
  is_negative: boolean
}

// 状态因子信息（包含计算后的 condition）
export interface PlayerConditionInfo {
  player_id: number
  form_cycle: number
  momentum: number          // -5 ~ +5
  last_performance: number
  last_match_won: boolean
  games_since_rest: number
  condition: number         // -10 ~ +10
  condition_range: [number, number]  // 年龄对应的范围
}

// 选手完整详情
export interface PlayerFullDetail {
  player: Player
  traits: TraitInfo[]
  condition_info: PlayerConditionInfo
}

// 选手属性更新请求
export interface UpdatePlayerRequest {
  player_id: number
  ability?: number
  potential?: number
  stability?: number
  age?: number
}

export const playerApi = {
  getPlayer: (playerId: number) =>
    invokeCommand<Player>('get_player', { playerId }),

  // 获取选手特性列表
  getPlayerTraits: (playerId: number) =>
    invokeCommand<TraitInfo[]>('get_player_traits', { playerId }),

  // 获取选手状态因子和 condition
  getPlayerCondition: (playerId: number) =>
    invokeCommand<PlayerConditionInfo>('get_player_condition', { playerId }),

  // 获取选手完整详情（包含特性和状态）
  getPlayerFullDetail: (playerId: number) =>
    invokeCommand<PlayerFullDetail>('get_player_full_detail', { playerId }),

  // 更新选手属性（能力值、潜力值、稳定性、年龄）
  updatePlayer: (request: UpdatePlayerRequest) =>
    invokeCommand<Player>('update_player', { request }),
}
