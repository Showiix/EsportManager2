/**
 * 战力引擎 - 基于5名选手计算队伍战力并模拟比赛
 *
 * 算法（策划案规定）：
 * - 队伍战力 = Σ(5名选手实际能力) / 5
 * - 发挥值 = 正态分布(队伍战力, σ=6)
 * - 发挥值高者获胜
 */

import type { Player, PlayerPerformance } from '@/types/player'
import type { GameDetail, MatchDetail } from '@/types/matchDetail'
import { PlayerEngine } from './PlayerEngine'

export class PowerEngine {
  // 队伍发挥的标准差（策划案规定）
  private static readonly PERFORMANCE_SIGMA = 6

  /**
   * Box-Muller 变换生成标准正态分布随机数
   */
  static gaussianRandom(): number {
    let u = 0
    let v = 0
    while (u === 0) u = Math.random()
    while (v === 0) v = Math.random()
    return Math.sqrt(-2.0 * Math.log(u)) * Math.cos(2.0 * Math.PI * v)
  }

  /**
   * 计算队伍战力（基于5名选手的实际发挥）
   * @param performances 选手发挥数组
   * @returns 队伍战力值
   */
  static calculateTeamPower(performances: PlayerPerformance[]): number {
    if (performances.length === 0) return 0
    const totalAbility = performances.reduce((sum, p) => sum + p.actualAbility, 0)
    return totalAbility / performances.length
  }

  /**
   * 计算队伍发挥值（正态分布）
   * @param teamPower 队伍战力
   * @returns 发挥值
   */
  static calculateTeamPerformanceValue(teamPower: number): number {
    const noise = this.gaussianRandom() * this.PERFORMANCE_SIGMA
    return teamPower + noise
  }

  /**
   * 计算选手影响力分数
   * 影响力 = 选手实际能力 - 队伍平均能力
   * @param performances 选手发挥数组
   * @param teamAverage 队伍平均能力
   */
  static calculateImpactScores(
    performances: PlayerPerformance[],
    teamAverage: number
  ): void {
    performances.forEach((p) => {
      p.impactScore = Math.round((p.actualAbility - teamAverage) * 10) / 10
    })
  }

  /**
   * 模拟单局比赛
   * @param teamAId A队ID
   * @param teamAName A队名称
   * @param teamAPlayers A队选手
   * @param teamBId B队ID
   * @param teamBName B队名称
   * @param teamBPlayers B队选手
   * @param gameNumber 第几局
   * @returns 单局比赛详情
   */
  static simulateGame(
    teamAId: string,
    teamAName: string,
    teamAPlayers: Player[],
    teamBId: string,
    teamBName: string,
    teamBPlayers: Player[],
    gameNumber: number
  ): GameDetail {
    // 1. 计算每位选手的实际发挥
    const teamAPerformances = PlayerEngine.calculateTeamPerformances(teamAPlayers)
    const teamBPerformances = PlayerEngine.calculateTeamPerformances(teamBPlayers)

    // 2. 计算队伍战力
    const teamAPower = this.calculateTeamPower(teamAPerformances)
    const teamBPower = this.calculateTeamPower(teamBPerformances)

    // 3. 计算影响力分数
    this.calculateImpactScores(teamAPerformances, teamAPower)
    this.calculateImpactScores(teamBPerformances, teamBPower)

    // 4. 计算队伍发挥值
    const teamAPerformanceValue = this.calculateTeamPerformanceValue(teamAPower)
    const teamBPerformanceValue = this.calculateTeamPerformanceValue(teamBPower)

    // 5. 决定胜负（发挥值高者获胜）
    const teamAWins = teamAPerformanceValue > teamBPerformanceValue
    const winnerId = teamAWins ? teamAId : teamBId
    const winnerName = teamAWins ? teamAName : teamBName

    // 6. 判断是否爆冷（战力低的队伍赢了）
    const isUpset =
      (teamAPower > teamBPower && !teamAWins) ||
      (teamBPower > teamAPower && teamAWins)

    return {
      gameNumber,
      teamAId,
      teamAName,
      teamAPower: Math.round(teamAPower * 10) / 10,
      teamAPerformance: Math.round(teamAPerformanceValue * 10) / 10,
      teamAPlayers: teamAPerformances,
      teamBId,
      teamBName,
      teamBPower: Math.round(teamBPower * 10) / 10,
      teamBPerformance: Math.round(teamBPerformanceValue * 10) / 10,
      teamBPlayers: teamBPerformances,
      winnerId,
      winnerName,
      powerDifference: Math.round((teamAPower - teamBPower) * 10) / 10,
      performanceDifference: Math.round((teamAPerformanceValue - teamBPerformanceValue) * 10) / 10,
      isUpset
    }
  }

  /**
   * 模拟完整比赛（BO1/BO3/BO5）
   * @returns 比赛详情
   */
  static simulateMatch(
    teamAId: string,
    teamAName: string,
    teamAPlayers: Player[],
    teamBId: string,
    teamBName: string,
    teamBPlayers: Player[],
    bestOf: number
  ): MatchDetail {
    const winsNeeded = Math.ceil(bestOf / 2)
    const games: GameDetail[] = []
    let scoreA = 0
    let scoreB = 0
    let gameNumber = 0

    // 模拟直到一方获得足够胜利
    while (scoreA < winsNeeded && scoreB < winsNeeded) {
      gameNumber++
      const game = this.simulateGame(
        teamAId,
        teamAName,
        teamAPlayers,
        teamBId,
        teamBName,
        teamBPlayers,
        gameNumber
      )
      games.push(game)

      if (game.winnerId === teamAId) {
        scoreA++
      } else {
        scoreB++
      }
    }

    // 分析MVP和关键选手
    const analysis = this.analyzeMatch(games, teamAId)

    const winnerId = scoreA > scoreB ? teamAId : teamBId
    const winnerName = scoreA > scoreB ? teamAName : teamBName

    return {
      matchId: '', // 由调用方设置
      teamAId,
      teamAName,
      teamBId,
      teamBName,
      bestOf,
      games,
      finalScoreA: scoreA,
      finalScoreB: scoreB,
      winnerId,
      winnerName,
      mvpPlayerId: analysis.mvp?.playerId,
      mvpPlayerName: analysis.mvp?.playerName,
      mvpTeamId: analysis.mvp?.teamId,
      mvpTotalImpact: analysis.mvp?.totalImpact,
      keyPlayer: analysis.keyPlayer,
      playedAt: new Date()
    }
  }

  /**
   * 分析比赛，找出MVP和关键选手
   */
  private static analyzeMatch(
    games: GameDetail[],
    teamAId: string
  ): {
    mvp: { playerId: string; playerName: string; teamId: string; totalImpact: number } | null
    keyPlayer: {
      playerId: string
      playerName: string
      teamId: string
      reason: '高发挥' | '低发挥'
      impactScore: number
      gameNumber: number
    } | null
  } {
    // 统计每位选手的累计影响力
    const playerImpacts = new Map<
      string,
      {
        playerId: string
        playerName: string
        teamId: string
        totalImpact: number
        gameCount: number
      }
    >()

    games.forEach((game) => {
      const allPlayers = [...game.teamAPlayers, ...game.teamBPlayers]
      allPlayers.forEach((p) => {
        const existing = playerImpacts.get(p.playerId)
        if (existing) {
          existing.totalImpact += p.impactScore
          existing.gameCount++
        } else {
          playerImpacts.set(p.playerId, {
            playerId: p.playerId,
            playerName: p.playerName,
            teamId: p.teamId,
            totalImpact: p.impactScore,
            gameCount: 1
          })
        }
      })
    })

    // 找出影响力最高的选手作为MVP
    let mvp: { playerId: string; playerName: string; teamId: string; totalImpact: number } | null =
      null
    let maxImpact = -Infinity

    playerImpacts.forEach((p) => {
      if (p.totalImpact > maxImpact) {
        maxImpact = p.totalImpact
        mvp = {
          playerId: p.playerId,
          playerName: p.playerName,
          teamId: p.teamId,
          totalImpact: Math.round(p.totalImpact * 10) / 10
        }
      }
    })

    // 找出关键选手（最后一局影响最大的）
    const lastGame = games[games.length - 1]
    const lastGamePlayers = [...lastGame.teamAPlayers, ...lastGame.teamBPlayers]
    const keyPlayerData = lastGamePlayers.reduce((max, p) =>
      Math.abs(p.impactScore) > Math.abs(max.impactScore) ? p : max
    )

    const keyPlayer = {
      playerId: keyPlayerData.playerId,
      playerName: keyPlayerData.playerName,
      teamId: keyPlayerData.teamId,
      reason: (keyPlayerData.impactScore > 0 ? '高发挥' : '低发挥') as '高发挥' | '低发挥',
      impactScore: keyPlayerData.impactScore,
      gameNumber: lastGame.gameNumber
    }

    return { mvp, keyPlayer }
  }

  /**
   * 根据比赛详情生成简要比分
   * 用于和旧系统兼容
   */
  static getMatchScore(detail: MatchDetail): { scoreA: number; scoreB: number; winnerId: string } {
    return {
      scoreA: detail.finalScoreA,
      scoreB: detail.finalScoreB,
      winnerId: detail.winnerId
    }
  }
}
