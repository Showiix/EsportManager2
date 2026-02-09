/**
 * 比赛详情相关类型定义
 */

import type { PlayerPerformance, PlayerPosition } from './player'

// 队伍统计数据
export interface TeamStats {
  totalKills: number
  totalDeaths: number
  totalAssists: number
  totalGold: number
  averageGameDuration: number
  firstBloodRate: number
  firstTowerRate: number
  baronRate: number
  dragonRate: number
}

// 单局MVP信息
export interface GameMvpInfo {
  playerId: string
  playerName: string
  teamId: string
  position: PlayerPosition
  mvpScore: number
}

// 单局比赛详情（BO系列中的每一局）
export interface GameDetail {
  gameNumber: number           // 第几局（从1开始）

  // 队伍A数据
  teamAId: string
  teamAName: string
  teamAPower: number           // A队战力（基于选手实际能力）
  teamAPerformance: number     // A队发挥值（正态分布后）
  teamAMetaPower?: number      // A队Meta加权战力
  teamAPlayers: PlayerPerformance[]

  // 队伍B数据
  teamBId: string
  teamBName: string
  teamBPower: number           // B队战力
  teamBPerformance: number     // B队发挥值
  teamBMetaPower?: number      // B队Meta加权战力
  teamBPlayers: PlayerPerformance[]

  // 结果
  winnerId: string
  winnerName: string
  powerDifference: number      // 战力差值（A - B）
  performanceDifference: number // 发挥差值（A - B）
  metaPowerDifference?: number // Meta加权战力差值（A - B）

  // 是否爆冷（战力低的队伍赢了）
  isUpset: boolean

  // 可选字段
  duration?: number            // 游戏时长（分钟）
  mvp?: GameMvpInfo            // 单局MVP
}

// 完整比赛详情（包含所有局）
export interface MatchDetail {
  matchId: string | number
  seasonId?: string
  tournamentId?: string          // 赛事ID
  tournamentType?: string        // 赛事类型（clauch/madrid/worlds等）

  // 队伍信息
  teamAId: string
  teamAName: string
  teamBId: string
  teamBName: string

  // 比赛配置
  bestOf: number               // BO1/BO3/BO5

  // 每局详情
  games: GameDetail[]

  // 最终结果
  finalScoreA: number
  finalScoreB: number
  winnerId: string
  winnerName?: string

  // MVP分析
  mvpPlayerId?: string
  mvpPlayerName?: string
  mvpTeamId?: string
  mvpTotalImpact?: number      // MVP累计影响力
  matchMvp?: GameMvpInfo       // 全场MVP

  // 关键选手（决定胜负的关键人物）
  keyPlayer?: {
    playerId: string
    playerName: string
    teamId: string
    reason: '高发挥' | '低发挥'
    impactScore: number
    gameNumber: number         // 在哪一局发挥关键作用
  }

  // 时间信息
  playedAt?: Date | string
  createdAt?: Date | string
  completedAt?: Date | string  // 完成时间

  // 队伍统计数据
  teamAStats?: TeamStats | null
  teamBStats?: TeamStats | null
}

// 赛季统计汇总
export interface SeasonMatchSummary {
  seasonId: string
  totalMatches: number
  totalGames: number
  upsetCount: number           // 爆冷次数
  avgPowerDifference: number   // 平均战力差
  avgPerformanceDifference: number
}

// 比赛类型名称映射
export const MATCH_TYPE_NAMES: Record<string, string> = {
  'group': '小组赛',
  'knockout': '淘汰赛',
  'swiss': '瑞士轮',
  'quarter_final': '八强赛',
  'semi_final': '半决赛',
  'third_place': '季军赛',
  'grand_final': '总决赛',
  'winner_r1': '胜者组第一轮',
  'winner_final': '胜者组决赛',
  'loser_r1': '败者组第一轮',
  'loser_r2': '败者组第二轮',
  'loser_r3': '败者组第三轮',
  'loser_r4': '败者组第四轮',
  'loser_final': '败者组决赛'
}
