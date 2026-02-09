/**
 * Tournament Store - 管理赛事和比赛
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  tournamentApi,
  queryApi,
  internationalApi,
  matchApi,
  type TournamentMatch,
  type Standing,
  type TournamentInfo,
  type BracketInfo,
  type SwissRoundStatus,
  type DetailedMatchResult,
  type MatchPrediction
} from '@/api/tauri'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('TournamentStore')

export const useTournamentStoreTauri = defineStore('tournamentTauri', () => {
  // ========================================
  // State
  // ========================================

  // 当前赛事
  const currentTournament = ref<TournamentInfo | null>(null)

  // 赛事列表
  const tournaments = ref<TournamentInfo[]>([])

  // 当前赛事的比赛列表
  const matches = ref<TournamentMatch[]>([])

  // 积分榜
  const standings = ref<Standing[]>([])

  // 对阵图（淘汰赛）
  const bracket = ref<BracketInfo | null>(null)

  // 瑞士轮状态
  const swissStatus = ref<SwissRoundStatus | null>(null)

  // 最近的比赛结果
  const lastMatchResult = ref<DetailedMatchResult | null>(null)

  // 比赛预测
  const matchPrediction = ref<MatchPrediction | null>(null)

  // 加载状态
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // ========================================
  // Computed
  // ========================================

  // 已完成的比赛
  const completedMatches = computed(() =>
    matches.value.filter(m => m.status === 'Completed')
  )

  // 待进行的比赛
  const pendingMatches = computed(() =>
    matches.value.filter(m => m.status === 'Scheduled')
  )

  // 下一场比赛
  const nextMatch = computed(() =>
    pendingMatches.value[0] ?? null
  )

  // 赛事进度
  const tournamentProgress = computed(() => ({
    completed: completedMatches.value.length,
    total: matches.value.length,
    percentage: matches.value.length > 0
      ? Math.round((completedMatches.value.length / matches.value.length) * 100)
      : 0
  }))

  // 是否是国际赛事
  const isInternational = computed(() =>
    currentTournament.value?.region_id === null
  )

  // ========================================
  // Actions
  // ========================================

  /**
   * 加载赛季赛事
   */
  const loadSeasonTournaments = async (seasonId: number) => {
    isLoading.value = true
    error.value = null

    try {
      tournaments.value = await queryApi.getSeasonTournaments(seasonId)
      logger.debug('赛季赛事已加载', { seasonId, count: tournaments.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournaments'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载赛季赛事',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载赛区赛事
   */
  const loadRegionTournaments = async (regionId: number, seasonId?: number) => {
    isLoading.value = true
    error.value = null

    try {
      tournaments.value = await queryApi.getRegionTournaments(regionId, seasonId)
      logger.debug('赛区赛事已加载', { regionId, seasonId, count: tournaments.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournaments'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载赛区赛事',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载国际赛事
   */
  const loadInternationalTournaments = async (seasonId?: number) => {
    isLoading.value = true
    error.value = null

    try {
      tournaments.value = await queryApi.getInternationalTournaments(seasonId)
      logger.debug('国际赛事已加载', { seasonId, count: tournaments.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournaments'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载国际赛事',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 选择赛事
   */
  const selectTournament = async (tournamentId: number) => {
    isLoading.value = true
    error.value = null

    try {
      // 并行加载赛事详情、比赛和积分榜
      const [tournament, tournamentMatches, tournamentStandings] = await Promise.all([
        queryApi.getTournamentDetail(tournamentId),
        tournamentApi.getTournamentMatches(tournamentId),
        tournamentApi.getStandings(tournamentId)
      ])

      currentTournament.value = tournament
      matches.value = tournamentMatches
      standings.value = tournamentStandings

      logger.info('赛事已选择', { tournamentId, name: tournament.name })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournament'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '选择赛事',
        canRetry: true,
        retryFn: () => selectTournament(tournamentId)
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载比赛列表
   */
  const loadMatches = async (tournamentId: number) => {
    isLoading.value = true
    error.value = null

    try {
      matches.value = await tournamentApi.getTournamentMatches(tournamentId)
      logger.debug('比赛列表已加载', { tournamentId, count: matches.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load matches'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载比赛列表',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载积分榜
   */
  const loadStandings = async (tournamentId: number) => {
    isLoading.value = true
    error.value = null

    try {
      standings.value = await tournamentApi.getStandings(tournamentId)
      logger.debug('积分榜已加载', { tournamentId, teamsCount: standings.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load standings'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载积分榜',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 模拟下一场比赛
   */
  const simulateNext = async () => {
    if (!currentTournament.value) {
      throw new Error('No tournament selected')
    }

    isLoading.value = true
    error.value = null

    try {
      const result = await tournamentApi.simulateNextMatch(currentTournament.value.id)
      if (result) {
        // 更新比赛列表中的对应比赛
        const index = matches.value.findIndex(m => m.id === result.id)
        if (index !== -1) {
          matches.value[index] = result
        }

        // 重新加载积分榜
        await loadStandings(currentTournament.value.id)

        logger.debug('比赛已模拟', {
          matchId: result.id,
          homeTeam: result.home_team_name,
          awayTeam: result.away_team_name
        })
      }
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to simulate match'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '模拟下一场比赛',
        canRetry: true,
        retryFn: simulateNext
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 模拟所有比赛
   */
  const simulateAll = async () => {
    if (!currentTournament.value) {
      throw new Error('No tournament selected')
    }

    isLoading.value = true
    error.value = null

    try {
      const results = await logger.timed('模拟所有比赛', () =>
        tournamentApi.simulateAllMatches(currentTournament.value!.id)
      )
      matches.value = results

      // 重新加载积分榜
      await loadStandings(currentTournament.value.id)

      logger.info('所有比赛已模拟', { matchCount: results.length })
      return results
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to simulate all matches'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '模拟所有比赛',
        canRetry: true,
        retryFn: simulateAll
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 模拟比赛（带详细数据）
   */
  const simulateMatchDetailed = async (matchId: number) => {
    isLoading.value = true
    error.value = null

    try {
      lastMatchResult.value = await matchApi.simulateMatchDetailed(matchId)

      // 更新比赛列表
      if (currentTournament.value) {
        await loadMatches(currentTournament.value.id)
        await loadStandings(currentTournament.value.id)
      }

      logger.debug('详细比赛已模拟', { matchId })
      return lastMatchResult.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to simulate match'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '模拟详细比赛',
        canRetry: true,
        retryFn: () => simulateMatchDetailed(matchId)
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取比赛预测
   */
  const getMatchPrediction = async (homeTeamId: number, awayTeamId: number) => {
    isLoading.value = true
    error.value = null

    try {
      matchPrediction.value = await matchApi.getMatchPrediction(homeTeamId, awayTeamId)
      return matchPrediction.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to get prediction'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '获取比赛预测',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载对阵图（淘汰赛）
   */
  const loadBracket = async (tournamentId: number) => {
    isLoading.value = true
    error.value = null

    try {
      bracket.value = await internationalApi.getTournamentBracket(tournamentId)
      logger.debug('对阵图已加载', { tournamentId })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load bracket'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载对阵图',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载瑞士轮状态
   */
  const loadSwissStatus = async (tournamentId: number) => {
    isLoading.value = true
    error.value = null

    try {
      swissStatus.value = await internationalApi.getSwissRoundStatus(tournamentId)
      logger.debug('瑞士轮状态已加载', { tournamentId })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load swiss status'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '加载瑞士轮状态',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 生成下一轮瑞士轮
   */
  const generateNextSwissRound = async (tournamentId: number) => {
    isLoading.value = true
    error.value = null

    try {
      const newMatchIds = await internationalApi.generateNextSwissRound(tournamentId)
      // 重新加载比赛和瑞士轮状态
      await Promise.all([
        loadMatches(tournamentId),
        loadSwissStatus(tournamentId)
      ])
      logger.info('瑞士轮已生成', { tournamentId, matchCount: newMatchIds.length })
      return newMatchIds
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to generate swiss round'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '生成瑞士轮',
        canRetry: true,
        retryFn: () => generateNextSwissRound(tournamentId)
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 推进淘汰赛对阵
   */
  const advanceBracket = async (
    tournamentId: number,
    completedMatchId: number,
    winnerId: number
  ) => {
    isLoading.value = true
    error.value = null

    try {
      const updatedMatchIds = await internationalApi.advanceBracket(
        tournamentId,
        completedMatchId,
        winnerId
      )
      // 重新加载对阵图
      await loadBracket(tournamentId)
      logger.info('对阵图已推进', { tournamentId, completedMatchId, winnerId })
      return updatedMatchIds
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to advance bracket'
      handleError(e, {
        component: 'TournamentStore',
        userAction: '推进淘汰赛对阵',
        canRetry: true,
        retryFn: () => advanceBracket(tournamentId, completedMatchId, winnerId)
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 清除所有状态（切换存档时调用）
   */
  const clearAll = () => {
    currentTournament.value = null
    tournaments.value = []
    matches.value = []
    standings.value = []
    bracket.value = null
    swissStatus.value = null
    lastMatchResult.value = null
    matchPrediction.value = null
    isLoading.value = false
    error.value = null
  }

  /**
   * 清除选中状态
   */
  const clearSelection = () => {
    currentTournament.value = null
    matches.value = []
    standings.value = []
    bracket.value = null
    swissStatus.value = null
    lastMatchResult.value = null
    matchPrediction.value = null
  }

  /**
   * 清除错误
   */
  const clearError = () => {
    error.value = null
  }

  // ========================================
  // Return
  // ========================================

  return {
    // State
    currentTournament,
    tournaments,
    matches,
    standings,
    bracket,
    swissStatus,
    lastMatchResult,
    matchPrediction,
    isLoading,
    error,

    // Computed
    completedMatches,
    pendingMatches,
    nextMatch,
    tournamentProgress,
    isInternational,

    // Actions
    loadSeasonTournaments,
    loadRegionTournaments,
    loadInternationalTournaments,
    selectTournament,
    loadMatches,
    loadStandings,
    simulateNext,
    simulateAll,
    simulateMatchDetailed,
    getMatchPrediction,
    loadBracket,
    loadSwissStatus,
    generateNextSwissRound,
    advanceBracket,
    clearAll,
    clearSelection,
    clearError,
  }
})
