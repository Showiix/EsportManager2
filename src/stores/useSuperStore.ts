import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  SuperBracket,
  SuperMatch,
  SuperStage,
  GenerateSuperRequest,
  SimulateSuperMatchRequest,
} from '@/types'

export const useSuperStore = defineStore('super', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Super对阵数据(按Super周期存储，如"S1-S2"、"S3-S4")
  const superBrackets = ref<Map<string, SuperBracket>>(new Map())

  // 当前选中的Super
  const currentBracket = ref<SuperBracket | null>(null)

  // 计算属性

  /**
   * 当前参赛队伍
   */
  const currentQualifiedTeams = computed(() => {
    return currentBracket.value?.qualifiedTeams || []
  })

  /**
   * 传奇组队伍（第1-4名）
   */
  const legendaryGroup = computed(() => {
    return currentBracket.value?.legendaryGroup || []
  })

  /**
   * 挑战者组队伍（第5-8名）
   */
  const challengerGroup = computed(() => {
    return currentBracket.value?.challengerGroup || []
  })

  /**
   * Fighter组队伍（第9-16名）
   */
  const fighterGroup = computed(() => {
    return currentBracket.value?.fighterGroup || []
  })

  /**
   * Fighter A组
   */
  const fighterGroupA = computed(() => {
    return currentBracket.value?.fighterGroupA || []
  })

  /**
   * Fighter B组
   */
  const fighterGroupB = computed(() => {
    return currentBracket.value?.fighterGroupB || []
  })

  /**
   * 所有轮次
   */
  const currentRounds = computed(() => {
    return currentBracket.value?.rounds || []
  })

  /**
   * Fighter组积分榜
   */
  const fighterStandings = computed(() => {
    return currentBracket.value?.fighterStandings || []
  })

  /**
   * 当前阶段
   */
  const currentStage = computed(() => {
    return currentBracket.value?.status || 'not_started'
  })

  /**
   * 是否完成
   */
  const isSuperComplete = computed(() => {
    return currentBracket.value?.status === 'completed'
  })

  /**
   * 冠军
   */
  const champion = computed(() => {
    return currentBracket.value?.champion || null
  })

  /**
   * 亚军
   */
  const runnerUp = computed(() => {
    return currentBracket.value?.runnerUp || null
  })

  /**
   * 季军
   */
  const thirdPlace = computed(() => {
    return currentBracket.value?.thirdPlace || null
  })

  /**
   * 第四名
   */
  const fourthPlace = computed(() => {
    return currentBracket.value?.fourthPlace || null
  })

  // 动作

  /**
   * 获取Super对阵信息
   */
  async function fetchSuperBracket(season1Code: string, season2Code: string) {
    loading.value = true
    error.value = null

    try {
      console.log(`[SuperStore] 获取Super赛事数据: ${season1Code}-${season2Code}`)
      const { superApi } = await import('@/api')
      const response = await superApi.getSuperBracket(season1Code, season2Code)

      if (response.data) {
        const key = `${season1Code}-${season2Code}`
        superBrackets.value.set(key, response.data)
        currentBracket.value = response.data
        console.log(`[SuperStore] ✅ Super赛事数据加载成功`)
        return response.data
      }
    } catch (err: any) {
      // 如果是404，表示还没有生成Super赛事
      if (err.response?.status === 404 || err.message?.includes('404')) {
        console.log('[SuperStore] Super赛事尚未生成')
        currentBracket.value = null
      } else {
        error.value = err.message || '获取Super信息失败'
        console.error('[SuperStore] ❌ 获取Super失败:', err)
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
      console.log(`[SuperStore] 开始生成Super赛事: ${request.season1Code}-${request.season2Code}`)
      const { superApi } = await import('@/api')
      const response = await superApi.generateSuper(request)

      if (response.data) {
        const key = `${request.season1Code}-${request.season2Code}`
        superBrackets.value.set(key, response.data)
        currentBracket.value = response.data
        console.log(`[SuperStore] ✅ Super赛事生成成功`)
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '生成Super对阵失败'
      console.error('[SuperStore] ❌ 生成Super失败:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 检查是否可以生成Super
   */
  async function checkSuperEligibility(season1Code: string, season2Code: string) {
    try {
      console.log(`[SuperStore] 检查Super资格: ${season1Code}-${season2Code}`)
      const { superApi } = await import('@/api')
      const response = await superApi.checkSuperEligibility(season1Code, season2Code)
      console.log(`[SuperStore] Super资格检查结果:`, response.data)
      return response.data
    } catch (err: any) {
      error.value = err.message || '检查Super资格失败'
      console.error('[SuperStore] ❌ 检查Super资格失败:', err)
      throw err
    }
  }

  /**
   * 获取有资格参加Super的队伍（基于两年积分）
   */
  async function getQualifiedTeams(season1Code: string, season2Code: string) {
    try {
      console.log(`[SuperStore] 获取晋级队伍: ${season1Code}-${season2Code}`)
      const { superApi } = await import('@/api')
      const response = await superApi.getQualifiedTeams(season1Code, season2Code)
      console.log(`[SuperStore] 晋级队伍数量:`, response.data?.length || 0)
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取晋级队伍失败'
      console.error('[SuperStore] ❌ 获取晋级队伍失败:', err)
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
      console.log(`[SuperStore] 模拟比赛: matchId=${request.matchId}`)
      const { superApi } = await import('@/api')
      const response = await superApi.simulateSuperMatch(request)

      if (response.data) {
        console.log(`[SuperStore] ✅ 比赛模拟成功`, response.data)
        
        // 更新当前对阵信息
        if (currentBracket.value) {
          // 更新比赛结果
          updateMatchInBracket(currentBracket.value, response.data.match)

          // 更新Fighter组积分榜（如果有）
          if (response.data.updatedStandings) {
            currentBracket.value.fighterStandings = response.data.updatedStandings
          }

          // 如果Super完成,更新最终排名
          if (response.data.isSuperComplete && response.data.finalStandings) {
            currentBracket.value.champion = response.data.finalStandings.champion
            currentBracket.value.runnerUp = response.data.finalStandings.runnerUp
            currentBracket.value.thirdPlace = response.data.finalStandings.thirdPlace
            currentBracket.value.fourthPlace = response.data.finalStandings.fourthPlace
            currentBracket.value.championshipRound2Eliminated = response.data.finalStandings.championshipRound2Eliminated
            currentBracket.value.championshipRound1Eliminated = response.data.finalStandings.championshipRound1Eliminated
            currentBracket.value.prepStageEliminated = response.data.finalStandings.prepStageEliminated
            currentBracket.value.advancementEliminated = response.data.finalStandings.advancementEliminated
            currentBracket.value.fighterEliminated = response.data.finalStandings.fighterEliminated
            currentBracket.value.status = 'completed'
          }

          // 如果阶段完成，更新状态
          if (response.data.isStageComplete) {
            console.log(`[SuperStore] 阶段完成，准备进入下一阶段`)
          }
        }

        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      console.error('[SuperStore] ❌ 模拟比赛失败:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 开始下一阶段
   */
  async function startNextStage(superId: string) {
    loading.value = true
    error.value = null

    try {
      console.log(`[SuperStore] 开始下一阶段: superId=${superId}`)
      const { superApi } = await import('@/api')
      const response = await superApi.startNextStage(superId)

      if (response.data) {
        // 刷新当前对阵数据
        if (currentBracket.value) {
          await fetchSuperBracket(currentBracket.value.season1Code, currentBracket.value.season2Code)
        }
        console.log(`[SuperStore] ✅ 进入下一阶段成功`)
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '开始下一阶段失败'
      console.error('[SuperStore] ❌ 开始下一阶段失败:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新对阵中的比赛信息
   */
  function updateMatchInBracket(bracket: SuperBracket, match: SuperMatch) {
    bracket.rounds.forEach(round => {
      const index = round.matches.findIndex(m => m.id === match.id)
      if (index !== -1) {
        round.matches[index] = match
      }
    })
  }

  /**
   * 设置当前Super
   */
  function setCurrentBracket(bracket: SuperBracket | null) {
    currentBracket.value = bracket
  }

  /**
   * 根据周期获取Super对阵
   */
  function getBracketByPeriod(season1Code: string, season2Code: string): SuperBracket | undefined {
    const key = `${season1Code}-${season2Code}`
    return superBrackets.value.get(key)
  }

  /**
   * 清空所有数据
   */
  function clearAll() {
    superBrackets.value.clear()
    currentBracket.value = null
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
  function getStageName(stage: SuperStage): string {
    const stageNames: Record<SuperStage, string> = {
      'not_started': '未开始',
      'fighter_group': 'Fighter组预选赛',
      'challenger_stage': '挑战者组阶段',
      'preparation_stage': '冠军赛预备战',
      'championship_stage': '终极冠军赛',
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

    // 计算属性
    currentQualifiedTeams,
    legendaryGroup,
    challengerGroup,
    fighterGroup,
    fighterGroupA,
    fighterGroupB,
    currentRounds,
    fighterStandings,
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
    getQualifiedTeams,
    simulateSuperMatch,
    startNextStage,
    setCurrentBracket,
    getBracketByPeriod,
    clearAll,
    resetError,
    getStageName
  }
})

