import { invokeCommand } from './client'

// ========================================
// 年度颁奖 API
// ========================================

/** 五维评分维度 */
export interface ScoreDimensions {
  impact_norm: number
  performance_norm: number
  stability_norm: number
  appearance_norm: number
  honor_norm: number
  big_stage_norm: number
}

/** 选手评语 */
export interface PlayerCommentary {
  description: string
  tags: string[]
}

/** 年度最佳阵容选手信息 */
export interface AllProPlayer {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  yearly_score: number
  avg_impact: number
  games_played: number
  tier: number
  commentary: PlayerCommentary
}

/** 选手单赛事表现明细 */
export interface TournamentDetail {
  tournament_name: string
  tournament_type: string
  games_played: number
  avg_impact: number
  avg_performance: number
  best_performance: number
  mvp_count: number
  weight: number
}

/** 年度Top20选手信息 */
export interface Top20Player {
  rank: number
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  yearly_score: number
  avg_impact: number
  games_played: number
  age: number
  avg_performance: number
  best_performance: number
  consistency_score: number
  champion_bonus: number
  international_titles: number
  regional_titles: number
  dimensions: ScoreDimensions
  commentary: PlayerCommentary
  tournament_details: TournamentDetail[]
  big_stage_score: number
  has_international: boolean
}

/** 特别奖选手信息 */
export interface SpecialAwardPlayer {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  age: number
  score: number
  games_played: number
  commentary: PlayerCommentary
}

/** 年度最佳新秀信息 */
export interface RookiePlayer {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  age: number
  yearly_score: number
  avg_impact: number
  games_played: number
  dimensions: ScoreDimensions
  commentary: PlayerCommentary
}

/** 年度颁奖数据 */
export interface AnnualAwardsData {
  season_id: number
  top20: Top20Player[]
  all_pro_1st: AllProPlayer[]
  all_pro_2nd: AllProPlayer[]
  all_pro_3rd: AllProPlayer[]
  most_consistent: SpecialAwardPlayer | null
  most_dominant: SpecialAwardPlayer | null
  rookie_of_the_year: RookiePlayer | null
  already_awarded: boolean
}

export const awardsApi = {
  /** 获取年度颁奖数据 */
  getAnnualAwardsData: (seasonId?: number) =>
    invokeCommand<AnnualAwardsData>('get_annual_awards_data', { seasonId }),
}
