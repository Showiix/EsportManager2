import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  GenerateSuperRequest,
  SimulateSuperMatchRequest,
} from '@/types'
import type { BracketInfo, TeamAnnualPoints } from '@/api/tauri'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('SuperStore')

export const useSuperStore = defineStore('super', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Super对阵数据(按tournament_id存储)
  const superBrackets = ref<Map<number, BracketInfo>>(new Map())

  // 当前赛事ID
  const currentTournamentId = ref<number | null>(null)

  // 资格队伍（Top16）
  const qualifiedTeams = ref<TeamAnnualPoints[]>([])

  // 当前对阵信息
  const currentBracket = ref<BracketInfo | null>(null)

  // 计算属性

  /**
   * 当前参赛队伍（从资格队伍获取）
   */
  const currentQualifiedTeams = computed(() => {
    return qualifiedTeams.value || []
  })

  /**
   * 传奇组队伍（第1-4名）
   */
  const legendaryGroup = computed(() => {
    return qualifiedTeams.value.slice(0, 4)
  })

  /**
   * 挑战者组队伍（第5-8名）
   */
  const challengerGroup = computed(() => {
    return qualifiedTeams.value.slice(4, 8)
  })

  /**
   * Fighter组队伍（第9-16名）
   */
  const fighterGroup = computed(() => {
    return qualifiedTeams.value.slice(8, 16)
  })

  /**
   * Fighter A组
   */
  const fighterGroupA = computed(() => {
    return qualifiedTeams.value.slice(8, 12)
  })

  /**
   * Fighter B组
   */
  const fighterGroupB = computed(() => {
    return qualifiedTeams.value.slice(12, 16)
  })

  /**
   * 所有比赛
   */
  const currentMatches = computed(() => {
    return currentBracket.value?.matches || []
  })

  /**
   * 所有阶段
   */
  const currentStages = computed(() => {
    return currentBracket.value?.stages || []
  })

  /**
   * 当前阶段状态
   */
  const currentStage = computed(() => {
    // 根据比赛状态判断当前阶段
    const stages = currentBracket.value?.stages || []
    const inProgressStage = stages.find(s => s.completed_matches < s.total_matches)
    return inProgressStage?.name || 'completed'
  })

  /**
   * 是否完成
   */
  const isSuperComplete = computed(() => {
    const stages = currentBracket.value?.stages || []
    return stages.every(s => s.completed_matches >= s.total_matches)
  })

  /**
   * 冠军（从决赛获取）
   */
  const champion = computed(() => {
    const matches = currentBracket.value?.matches || []
    const grandFinal = matches.find(m => m.stage === 'GRAND_FINAL')
    if (grandFinal?.winner_id) {
      return grandFinal.home_team?.id === grandFinal.winner_id
        ? grandFinal.home_team
        : grandFinal.away_team
    }
    return null
  })

  /**
   * 亚军
   */
  const runnerUp = computed(() => {
    const matches = currentBracket.value?.matches || []
    const grandFinal = matches.find(m => m.stage === 'GRAND_FINAL')
    if (grandFinal?.winner_id) {
      return grandFinal.home_team?.id === grandFinal.winner_id
        ? grandFinal.away_team
        : grandFinal.home_team
    }
    return null
  })

  /**
   * 季军（从季军赛获取）
   */
  const thirdPlace = computed(() => {
    const matches = currentBracket.value?.matches || []
    const thirdPlaceMatch = matches.find(m => m.stage === 'THIRD_PLACE')
    if (thirdPlaceMatch?.winner_id) {
      return thirdPlaceMatch.home_team?.id === thirdPlaceMatch.winner_id
        ? thirdPlaceMatch.home_team
        : thirdPlaceMatch.away_team
    }
    return null
  })

  /**
   * 第四名
   */
  const fourthPlace = computed(() => {
    const matches = currentBracket.value?.matches || []
    const thirdPlaceMatch = matches.find(m => m.stage === 'THIRD_PLACE')
    if (thirdPlaceMatch?.winner_id) {
      return thirdPlaceMatch.home_team?.id === thirdPlaceMatch.winner_id
        ? thirdPlaceMatch.away_team
        : thirdPlaceMatch.home_team
    }
    return null
  })

  // 动作

  /**
   * 获取Super对阵信息（通过tournament_id）
   */
  async function fetchSuperBracket(tournamentId: number) {
    loading.value = true
    error.value = null

    try {
      logger.debug('获取Super赛事数据', { tournamentId })
      const { superApi } = await import('@/api')
      const response = await superApi.getSuperBracket(tournamentId)

      if (response.data) {
        // 使用 unknown 作为中间类型进行转换
        const bracket = response.data as unknown as BracketInfo
        superBrackets.value.set(tournamentId, bracket)
        currentBracket.value = bracket
        currentTournamentId.value = tournamentId
        logger.info('Super赛事数据加载成功')
        return response.data
      }
    } catch (err: any) {
      // 如果是404，表示还没有生成Super赛事
      if (err.response?.status === 404 || err.message?.includes('404')) {
        logger.debug('Super赛事尚未生成')
        currentBracket.value = null
      } else {
        error.value = err.message || '获取Super信息失败'
        handleError(err, {
          component: 'SuperStore',
          userAction: '获取Super赛事',
          silent: true
        })
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 生成Super对阵
   */
  async function generateSuper(request: GenerateSuperRequest) {
    loading.value = true
    error.value = null

    try {
      logger.info('开始生成Super赛事')
      const { superApi } = await import('@/api')
      const response = await superApi.generateSuper(request)

      if (response.data) {
        // 使用 unknown 作为中间类型进行转换
        const bracket = response.data as unknown as BracketInfo
        const tournamentId = bracket.tournament_id
        superBrackets.value.set(tournamentId, bracket)
        currentBracket.value = bracket
        currentTournamentId.value = tournamentId
        logger.info('Super赛事生成成功', { tournamentId })
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '生成Super对阵失败'
      handleError(err, {
        component: 'SuperStore',
        userAction: '生成Super赛事',
        canRetry: true,
        retryFn: () => generateSuper(request)
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 检查是否可以生成Super
   */
  async function checkSuperEligibility() {
    try {
      logger.debug('检查Super资格')
      const { superApi } = await import('@/api')
      const response = await superApi.checkSuperEligibility()
      logger.debug('Super资格检查结果', { result: response.data })
      return response.data
    } catch (err: any) {
      error.value = err.message || '检查Super资格失败'
      handleError(err, {
        component: 'SuperStore',
        userAction: '检查Super资格',
        silent: true
      })
      throw err
    }
  }

  /**
   * 获取有资格参加Super的队伍（基于年度积分前16名）
   */
  async function fetchQualifiedTeams() {
    try {
      logger.debug('获取晋级队伍')
      const { superApi } = await import('@/api')
      const response = await superApi.getQualifiedTeams()
      if (response.data) {
        // 使用 unknown 作为中间类型进行转换
        qualifiedTeams.value = response.data as unknown as TeamAnnualPoints[]
        logger.debug('晋级队伍数量', { count: qualifiedTeams.value.length })
      }
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取晋级队伍失败'
      handleError(err, {
        component: 'SuperStore',
        userAction: '获取晋级队伍',
        silent: true
      })
      throw err
    }
  }

  /**
   * 模拟Super单场比赛
   */
  async function simulateSuperMatch(request: SimulateSuperMatchRequest) {
    loading.value = true
    error.value = null

    try {
      logger.debug('模拟比赛', { matchId: request.matchId })
      const { superApi } = await import('@/api')
      const response = await superApi.simulateSuperMatch(request)

      if (response.data) {
        logger.info('比赛模拟成功', { result: response.data })

        // 刷新对阵数据
        if (currentTournamentId.value) {
          await fetchSuperBracket(currentTournamentId.value)
        }

        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      handleError(err, {
        component: 'SuperStore',
        userAction: '模拟Super比赛',
        canRetry: true,
        retryFn: () => simulateSuperMatch(request)
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取Fighter组积分榜
   */
  async function fetchFighterStandings(tournamentId: number) {
    try {
      logger.debug('获取Fighter组积分榜', { tournamentId })
      const { superApi } = await import('@/api')
      const response = await superApi.getFighterStandings(tournamentId)
      logger.debug('Fighter组积分榜', { standings: response.data })
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取积分榜失败'
      handleError(err, {
        component: 'SuperStore',
        userAction: '获取Fighter积分榜',
        silent: true
      })
      throw err
    }
  }

  /**
   * 完成赛事（发放奖金和荣誉）
   */
  async function completeTournament(tournamentId: number) {
    loading.value = true
    error.value = null

    try {
      logger.info('完成赛事', { tournamentId })
      const { superApi } = await import('@/api')
      const response = await superApi.completeTournament(tournamentId)
      logger.info('赛事完成成功', { result: response.data })
      return response.data
    } catch (err: any) {
      error.value = err.message || '完成赛事失败'
      handleError(err, {
        component: 'SuperStore',
        userAction: '完成Super赛事'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置当前Super对阵
   */
  function setCurrentBracket(bracket: BracketInfo | null) {
    currentBracket.value = bracket
    if (bracket) {
      currentTournamentId.value = bracket.tournament_id
    }
  }

  /**
   * 根据tournament_id获取Super对阵
   */
  function getBracketById(tournamentId: number): BracketInfo | undefined {
    return superBrackets.value.get(tournamentId)
  }

  /**
   * 清空所有数据
   */
  function clearAll() {
    superBrackets.value.clear()
    currentBracket.value = null
    currentTournamentId.value = null
    qualifiedTeams.value = []
    error.value = null
  }

  /**
   * 重置错误
   */
  function resetError() {
    error.value = null
  }

  /**
   * 获取阶段名称（中文）
   */
  function getStageName(stage: string): string {
    const stageNames: Record<string, string> = {
      'not_started': '未开始',
      'FIGHTER_GROUP_A': 'Fighter A组',
      'FIGHTER_GROUP_B': 'Fighter B组',
      'CHALLENGER_STAGE': '挑战者组阶段',
      'PREPARATION_STAGE': '冠军赛预备战',
      'CHAMPIONSHIP_STAGE': '终极冠军赛',
      'GRAND_FINAL': '总决赛',
      'THIRD_PLACE': '季军赛',
      'completed': '已完成'
    }
    return stageNames[stage] || stage
  }

  return {
    // 状态
    loading,
    error,
    superBrackets,
    currentBracket,
    currentTournamentId,
    qualifiedTeams,

    // 计算属性
    currentQualifiedTeams,
    legendaryGroup,
    challengerGroup,
    fighterGroup,
    fighterGroupA,
    fighterGroupB,
    currentMatches,
    currentStages,
    currentStage,
    isSuperComplete,
    champion,
    runnerUp,
    thirdPlace,
    fourthPlace,

    // 动作
    fetchSuperBracket,
    generateSuper,
    checkSuperEligibility,
    fetchQualifiedTeams,
    simulateSuperMatch,
    fetchFighterStandings,
    completeTournament,
    setCurrentBracket,
    getBracketById,
    clearAll,
    resetError,
    getStageName
  }
})

