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
      console.log(`Loaded ${tournaments.value.length} tournaments for season ${seasonId}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournaments'
      console.error('Failed to load tournaments:', e)
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
      console.log(`Loaded ${tournaments.value.length} tournaments for region ${regionId}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournaments'
      console.error('Failed to load tournaments:', e)
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
      console.log(`Loaded ${tournaments.value.length} international tournaments`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournaments'
      console.error('Failed to load tournaments:', e)
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

      console.log(`Selected tournament: ${tournament.name}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tournament'
      console.error('Failed to load tournament:', e)
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
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load matches'
      console.error('Failed to load matches:', e)
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
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load standings'
      console.error('Failed to load standings:', e)
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
      }
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to simulate match'
      console.error('Failed to simulate match:', e)
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
      const results = await tournamentApi.simulateAllMatches(currentTournament.value.id)
      matches.value = results

      // 重新加载积分榜
      await loadStandings(currentTournament.value.id)

      console.log(`Simulated ${results.length} matches`)
      return results
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to simulate all matches'
      console.error('Failed to simulate all matches:', e)
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

      return lastMatchResult.value
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to simulate match'
      console.error('Failed to simulate match:', e)
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
      console.error('Failed to get prediction:', e)
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
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load bracket'
      console.error('Failed to load bracket:', e)
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
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load swiss status'
      console.error('Failed to load swiss status:', e)
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
      return newMatchIds
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to generate swiss round'
      console.error('Failed to generate swiss round:', e)
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
      return updatedMatchIds
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to advance bracket'
      console.error('Failed to advance bracket:', e)
      throw e
    } finally {
      isLoading.value = false
    }
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
    clearSelection,
    clearError,
  }
})
