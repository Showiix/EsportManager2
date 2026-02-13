import { invokeCommand } from './client'

// ========================================
// Match Simulation
// ========================================

export interface DetailedMatchResult {
  match_id: number
  tournament_id: number
  home_team_id: number
  away_team_id: number
  home_score: number
  away_score: number
  winner_id: number
  games: DetailedGameResult[]
  match_mvp: MvpInfo | null
  home_team_stats: TeamMatchStats
  away_team_stats: TeamMatchStats
}

export interface MvpInfo {
  player_id: number
  player_name: string
  team_id: number
  position: string
  mvp_score: number
}

export interface DetailedGameResult {
  game_number: number
  winner_id: number
  duration_minutes: number
  home_performance: number
  away_performance: number
  game_mvp: MvpInfo
  home_players: PlayerGameStats[]
  away_players: PlayerGameStats[]
  key_events: GameEvent[]
}

export interface PlayerGameStats {
  player_id: number
  player_name: string
  position: string
  base_ability: number
  condition_bonus: number
  stability_noise: number
  actual_ability: number
  kills: number
  deaths: number
  assists: number
  cs: number
  gold: number
  damage_dealt: number
  damage_taken: number
  vision_score: number
  mvp_score: number
  impact_score: number
  // 特性系统
  traits: string[]                          // 选手拥有的特性列表
  activated_traits: ActivatedTraitInfo[]    // 本局激活的特性效果
}

// 激活的特性效果信息
export interface ActivatedTraitInfo {
  trait_type: string
  name: string           // 特性显示名称
  effect: string         // 效果描述
  value: number          // 效果数值
  is_positive: boolean   // 是否为正面效果
}

export interface GameEvent {
  time_minutes: number
  event_type: string
  description: string
  team_id: number
}

export interface TeamMatchStats {
  team_id: number
  total_kills: number
  total_deaths: number
  total_assists: number
  total_gold: number
  total_damage: number
}

export interface BatchDetailedResult {
  results: DetailedMatchResult[]
  total: number
  success: number
  failed: number
}

export interface PlayerSeasonStats {
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  position: string
  matches_played: number
  games_played: number
  wins: number
  losses: number
  total_kills: number
  total_deaths: number
  total_assists: number
  avg_kills: number
  avg_deaths: number
  avg_assists: number
  kda: number
  avg_cs: number
  avg_damage: number
  avg_gold: number
  mvp_count: number
}

export interface MatchPrediction {
  home_team_id: number
  home_team_name: string
  away_team_id: number
  away_team_name: string
  home_win_probability: number
  away_win_probability: number
  predicted_winner_id: number
  predicted_score: string
  key_factors: string[]
}

export const matchApi = {
  simulateMatchDetailed: (matchId: number) =>
    invokeCommand<DetailedMatchResult>('simulate_match_detailed', { matchId }),

  simulateAllMatchesDetailed: (tournamentId: number) =>
    invokeCommand<BatchDetailedResult>('simulate_all_matches_detailed', { tournamentId }),

  getPlayerSeasonStats: (playerId: number, seasonId?: number) =>
    invokeCommand<PlayerSeasonStats>('get_player_season_stats', { playerId, seasonId }),

  getMatchPrediction: (homeTeamId: number, awayTeamId: number) =>
    invokeCommand<MatchPrediction>('get_match_prediction', { homeTeamId, awayTeamId }),

  /** 更新比赛结果（用于本地模拟后同步数据库） */
  updateMatchResult: (matchId: number, homeScore: number, awayScore: number, winnerId: number) =>
    invokeCommand<boolean>('update_match_result', { matchId, homeScore, awayScore, winnerId }),

  /** 更新比赛队伍（用于填充淘汰赛待定队伍） */
  updateMatchTeams: (matchId: number, homeTeamId: number, awayTeamId: number) =>
    invokeCommand<boolean>('update_match_teams', { matchId, homeTeamId, awayTeamId }),

  /** 取消比赛（标记为 CANCELLED） */
  cancelMatch: (matchId: number) =>
    invokeCommand<boolean>('cancel_match', { matchId }),
}
