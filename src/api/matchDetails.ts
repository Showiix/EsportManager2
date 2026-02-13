import { invokeCommand } from './client'

// ========================================
// 比赛详情持久化 API
// ========================================

/** 单局比赛详情 */
export interface MatchGameDetail {
  id: string
  save_id: string
  match_id: number
  game_number: number
  winner_team_id: number
  loser_team_id: number
  duration_minutes: number | null
  mvp_player_id: number | null
  key_player_id: number | null
  home_power: number | null
  away_power: number | null
  home_meta_power: number | null
  away_meta_power: number | null
  created_at: string | null
}

/** 选手单局表现 */
export interface GamePlayerPerformance {
  id: string
  save_id: string
  game_id: string
  player_id: number
  player_name: string           // 选手名称（快照）
  team_id: number
  team_name: string             // 队伍名称（快照）
  position: string
  base_ability: number          // 基础能力值
  condition_bonus: number       // 状态加成
  stability_noise: number       // 稳定性波动
  actual_ability: number        // 实际发挥值
  impact_score: number          // 影响力得分
  mvp_score: number             // MVP 得分
  is_mvp: boolean
  is_key_player: boolean
  // 详细战斗数据
  kills: number | null
  deaths: number | null
  assists: number | null
  cs: number | null
  gold: number | null
  damage_dealt: number | null
  damage_taken: number | null
  vision_score: number | null
  // 特性系统
  traits_json: string | null              // 选手拥有的特性 (JSON数组)
  activated_traits_json: string | null    // 本局激活的特性效果 (JSON数组)
  created_at: string | null
}

/** 单局详情（包含选手表现） */
export interface GameDetailWithPerformances {
  game: MatchGameDetail
  performances: GamePlayerPerformance[]
}

/** 完整比赛详情 */
export interface MatchFullDetails {
  match_id: number
  games: GameDetailWithPerformances[]
}

/** 保存选手表现输入 */
export interface SavePerformanceInput {
  player_id: number
  player_name: string           // 选手名称（快照）
  team_id: number
  team_name: string             // 队伍名称（快照）
  position: string
  base_ability: number          // 基础能力值
  condition_bonus: number       // 状态加成
  stability_noise: number       // 稳定性波动
  actual_ability: number        // 实际发挥值
  impact_score: number          // 影响力得分
  mvp_score: number             // MVP 得分
  is_mvp: boolean
  is_key_player: boolean
  // 详细战斗数据
  kills: number | null
  deaths: number | null
  assists: number | null
  cs: number | null
  gold: number | null
  damage_dealt: number | null
  damage_taken: number | null
  vision_score: number | null
  // 特性系统
  traits_json: string | null              // 选手拥有的特性 (JSON数组)
  activated_traits_json: string | null    // 本局激活的特性效果 (JSON数组)
}

/** 保存单局输入 */
export interface SaveGameInput {
  game_number: number
  winner_team_id: number
  loser_team_id: number
  duration_minutes: number | null
  mvp_player_id: number | null
  key_player_id: number | null
  home_power: number | null
  away_power: number | null
  home_meta_power: number | null
  away_meta_power: number | null
  performances: SavePerformanceInput[]
}

/** 保存比赛详情输入 */
export interface SaveMatchDetailsInput {
  match_id: number
  games: SaveGameInput[]
}

export const matchDetailsApi = {
  /** 保存比赛详情 */
  saveMatchDetails: (saveId: string, input: SaveMatchDetailsInput) =>
    invokeCommand<void>('save_match_details', { saveId, input }),

  /** 获取比赛详情 */
  getMatchDetails: (saveId: string, matchId: number) =>
    invokeCommand<MatchFullDetails | null>('get_match_details', { saveId, matchId }),

  /** 删除比赛详情 */
  deleteMatchDetails: (saveId: string, matchId: number) =>
    invokeCommand<void>('delete_match_details', { saveId, matchId }),
}
