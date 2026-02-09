import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { matchApi } from '@/api/tauri'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import type { MatchDetail } from '@/types/matchDetail'
import type { PlayerPosition } from '@/types/player'
import { createLogger } from '@/utils/logger'

const logger = createLogger('useBatchSimulation')

// --- 公共工具函数 ---

/** 解析位置格式（后端可能返回 "Some(Adc)" 格式） */
export const parsePosition = (pos: string | null | undefined): string => {
  if (!pos) return 'MID'
  const someMatch = pos.match(/Some\((\w+)\)/)
  if (someMatch) return someMatch[1]
  return pos
}

/** 将位置转换为标准 PlayerPosition 格式 */
export const normalizePosition = (pos: string): PlayerPosition => {
  const posMap: Record<string, PlayerPosition> = {
    'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP',
    'top': 'TOP', 'jungle': 'JUG', 'mid': 'MID', 'adc': 'ADC', 'support': 'SUP',
    'TOP': 'TOP', 'JUG': 'JUG', 'MID': 'MID', 'ADC': 'ADC', 'SUP': 'SUP',
    'Jug': 'JUG', 'Sup': 'SUP',
  }
  return posMap[pos] || 'MID'
}

/** 转换选手表现数据 */
export const convertPlayerPerformance = (p: any, teamId: string) => ({
  playerId: String(p.player_id),
  playerName: p.player_name,
  position: normalizePosition(parsePosition(p.position)),
  teamId: teamId,
  baseAbility: p.base_ability,
  conditionBonus: p.condition_bonus,
  stabilityNoise: p.stability_noise,
  actualAbility: p.actual_ability,
  impactScore: p.impact_score,
  traits: p.traits,
  activatedTraits: p.activated_traits?.map((t: any) => ({
    type: t.trait_type,
    name: t.name,
    effect: t.effect,
    value: t.value,
    isPositive: t.is_positive
  }))
})

/** 计算队伍战力（选手实际发挥能力平均值） */
export const calcTeamPower = (players: any[]): number => {
  if (!players || players.length === 0) return 0
  const sum = players.reduce((acc: number, p: any) => acc + (p.actual_ability || p.base_ability || 0), 0)
  return sum / players.length
}

/** 从模拟结果构建 MatchDetail 对象 */
export function buildMatchDetail(opts: {
  matchId: string | number
  tournamentType: string
  seasonId: string
  teamAId: string
  teamAName: string
  teamBId: string
  teamBName: string
  bestOf: number
  result: any
}): MatchDetail {
  const { matchId, tournamentType, seasonId, teamAId, teamAName, teamBId, teamBName, bestOf, result } = opts
  return {
    matchId,
    tournamentType,
    seasonId,
    teamAId,
    teamAName,
    teamBId,
    teamBName,
    bestOf,
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    winnerName: result.winner_id === result.home_team_id ? teamAName : teamBName,
    mvpPlayerId: result.match_mvp ? String(result.match_mvp.player_id) : undefined,
    mvpPlayerName: result.match_mvp?.player_name,
    mvpTeamId: result.match_mvp ? String(result.match_mvp.team_id) : undefined,
    mvpTotalImpact: result.match_mvp?.mvp_score,
    games: result.games.map((game: any) => {
      const teamAPower = calcTeamPower(game.home_players)
      const teamBPower = calcTeamPower(game.away_players)
      return {
        gameNumber: game.game_number,
        teamAId,
        teamAName,
        teamAPower,
        teamAPerformance: teamAPower,
        teamAMetaPower: game.home_performance,
        teamAPlayers: game.home_players.map((p: any) => convertPlayerPerformance(p, teamAId)),
        teamBId,
        teamBName,
        teamBPower,
        teamBPerformance: teamBPower,
        teamBMetaPower: game.away_performance,
        teamBPlayers: game.away_players.map((p: any) => convertPlayerPerformance(p, teamBId)),
        winnerId: String(game.winner_id),
        winnerName: game.winner_id === result.home_team_id ? teamAName : teamBName,
        powerDifference: teamAPower - teamBPower,
        performanceDifference: teamAPower - teamBPower,
        metaPowerDifference: game.home_performance - game.away_performance,
        isUpset: (teamAPower > teamBPower && game.winner_id !== result.home_team_id) ||
                 (teamBPower > teamAPower && game.winner_id === result.home_team_id)
      }
    })
  }
}

/** 记录比赛中所有选手的表现到 playerStore */
export function recordMatchPerformances(
  matchDetail: MatchDetail,
  seasonId: string,
  _competitionType: string,
  playerStore: ReturnType<typeof usePlayerStore>
) {
  matchDetail.games.forEach(game => {
    const recordPlayers = (players: any[]) => {
      players.forEach(perf => {
        playerStore.recordPerformance(
          perf.playerId,
          perf.playerName,
          perf.teamId,
          perf.position,
          perf.impactScore,
          perf.actualAbility,
          seasonId
        )
      })
    }
    recordPlayers(game.teamAPlayers)
    recordPlayers(game.teamBPlayers)
  })
}

// --- 批量模拟的比赛描述 ---

export interface BatchMatchInfo {
  matchId: number
  teamAId: string
  teamAName: string
  teamBId: string
  teamBName: string
  bestOf: number
  /** 可选的后端数据库 ID（用于双重保存） */
  backendMatchId?: number
  /** 可选的前端自定义 ID（用于 matchDetailStore 的 key） */
  frontendMatchId?: string | number
}

export interface BatchSimulateOptions {
  confirmMessage: string
  confirmTitle: string
  confirmType?: 'info' | 'warning'
  successMessage: string
  errorPrefix: string
  matches: BatchMatchInfo[]
  tournamentType: string
  seasonId: string
  competitionType?: string
  /** 跳过确认对话框，直接开始模拟 */
  skipConfirm?: boolean
  delayMs?: number
  /** 赛事 ID — 如果提供，使用后端批量命令一次模拟全部 */
  tournamentId?: number
  /** 每场比赛模拟后的回调（如 advanceBracket） */
  onMatchSimulated?: (matchId: number, result: any, matchDetail: MatchDetail) => Promise<void>
  /** 全部完成后的回调（如 loadTournamentData） */
  onComplete?: () => Promise<void>
}

// --- Composable ---

export function useBatchSimulation() {
  const simulationProgress = ref(0)
  const isSimulating = ref(false)
  const matchDetailStore = useMatchDetailStore()
  const playerStore = usePlayerStore()

  /**
   * 批量模拟比赛（通用流程）
   * 包含确认对话框 → 循环模拟 → 保存详情 → 记录表现 → 成功通知
   */
  async function batchSimulate(options: BatchSimulateOptions) {
    try {
      if (!options.skipConfirm) {
        await ElMessageBox.confirm(
          options.confirmMessage,
          options.confirmTitle,
          {
            confirmButtonText: '开始模拟',
            cancelButtonText: '取消',
            type: options.confirmType || 'info'
          }
        )
      }

      isSimulating.value = true
      simulationProgress.value = 0

      const { matches } = options
      if (matches.length === 0) {
        ElMessage.info('没有需要模拟的比赛')
        return
      }

      if (options.tournamentId) {
        // 快速路径：一次 IPC 调用批量模拟
        const batchResult = await matchApi.simulateAllMatchesDetailed(options.tournamentId)

        for (let i = 0; i < batchResult.results.length; i++) {
          const result = batchResult.results[i]
          const match = matches.find(m => m.matchId === result.match_id)
          if (!match) continue

          const saveId = match.frontendMatchId ?? String(match.matchId)
          const matchDetail = buildMatchDetail({
            matchId: saveId,
            tournamentType: options.tournamentType,
            seasonId: options.seasonId,
            teamAId: match.teamAId,
            teamAName: match.teamAName,
            teamBId: match.teamBId,
            teamBName: match.teamBName,
            bestOf: match.bestOf,
            result
          })

          await matchDetailStore.saveMatchDetail(saveId, matchDetail)

          if (match.backendMatchId) {
            const dbDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
            await matchDetailStore.saveMatchDetail(match.backendMatchId, dbDetail)
          }

          recordMatchPerformances(
            matchDetail,
            options.seasonId,
            options.competitionType || 'INTL',
            playerStore
          )

          if (options.onMatchSimulated) {
            await options.onMatchSimulated(match.matchId, result, matchDetail)
          }

          simulationProgress.value = Math.floor(((i + 1) / batchResult.results.length) * 100)
        }
      } else {
        // 原有逐场模拟路径
        for (let i = 0; i < matches.length; i++) {
          const match = matches[i]
          try {
            const result = await matchApi.simulateMatchDetailed(match.matchId)

            const saveId = match.frontendMatchId ?? String(match.matchId)
            const matchDetail = buildMatchDetail({
              matchId: saveId,
              tournamentType: options.tournamentType,
              seasonId: options.seasonId,
              teamAId: match.teamAId,
              teamAName: match.teamAName,
              teamBId: match.teamBId,
              teamBName: match.teamBName,
              bestOf: match.bestOf,
              result
            })

            await matchDetailStore.saveMatchDetail(saveId, matchDetail)

            if (match.backendMatchId) {
              const dbDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
              await matchDetailStore.saveMatchDetail(match.backendMatchId, dbDetail)
            }

            recordMatchPerformances(
              matchDetail,
              options.seasonId,
              options.competitionType || 'INTL',
              playerStore
            )

            if (options.onMatchSimulated) {
              await options.onMatchSimulated(match.matchId, result, matchDetail)
            }
          } catch (e) {
            logger.error(`模拟比赛 ${match.matchId} 失败:`, e)
          }

          simulationProgress.value = Math.floor(((i + 1) / matches.length) * 100)
          await new Promise(resolve => setTimeout(resolve, options.delayMs ?? 50))
        }
      }

      playerStore.saveToStorage()

      if (options.onComplete) {
        await options.onComplete()
      }

      ElMessage.success(options.successMessage)
    } catch (error: any) {
      if (error !== 'cancel') {
        logger.error(`${options.errorPrefix}:`, error)
        ElMessage.error(error.message || options.errorPrefix)
      }
    } finally {
      isSimulating.value = false
      simulationProgress.value = 0
    }
  }

  return {
    simulationProgress,
    isSimulating,
    batchSimulate,
  }
}
