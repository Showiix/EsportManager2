import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  MSIBracket,
  MSIMatch,
  GenerateMSIRequest,
  SimulateMSIMatchRequest
} from '@/types'

export const useMSIStore = defineStore('msi', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)

  // MSI对阵数据(按赛季ID存储)
  const msiBrackets = ref<Map<string, MSIBracket>>(new Map())

  // 当前选中的MSI
  const currentBracket = ref<MSIBracket | null>(null)

  // 计算属性
  const currentQualifiedTeams = computed(() => {
    return currentBracket.value?.qualifiedTeams || []
  })

  const legendaryGroup = computed(() => {
    return currentBracket.value?.legendaryGroup || []
  })

  const challengerGroup = computed(() => {
    return currentBracket.value?.challengerGroup || []
  })

  const qualifierGroup = computed(() => {
    return currentBracket.value?.qualifierGroup || []
  })

  const currentRounds = computed(() => {
    return currentBracket.value?.rounds || []
  })

  const isMSIComplete = computed(() => {
    return currentBracket.value?.status === 'completed'
  })

  const champion = computed(() => {
    return currentBracket.value?.champion || null
  })

  const runnerUp = computed(() => {
    return currentBracket.value?.runnerUp || null
  })

  const thirdPlace = computed(() => {
    return currentBracket.value?.thirdPlace || null
  })

  // 动作

  /**
   * 获取MSI对阵信息
   */
  async function fetchMSIBracket(seasonId: string) {
    loading.value = true
    error.value = null

    try {
      const { msiApi } = await import('@/api')
      const response = await msiApi.getMSIBracket(seasonId)

      if (response.data) {
        msiBrackets.value.set(seasonId, response.data)
        currentBracket.value = response.data
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '获取MSI信息失败'
      console.error('Failed to fetch MSI bracket:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 根据赛季代码获取MSI信息（如"S1"）
   */
  async function fetchMSIBracketBySeason(seasonCode: string) {
    // 从eventStore获取实际的season_id
    const { useEventStore } = await import('./useEventStore')
    const eventStore = useEventStore()
    
    const season = eventStore.seasons.find(s => s.seasonCode === seasonCode)
    if (!season) {
      throw new Error(`未找到赛季 ${seasonCode}`)
    }
    
    const seasonId = String(season.id)
    console.log(`[MSIStore] fetchMSIBracketBySeason: ${seasonCode} -> ID: ${seasonId}`)
    return await fetchMSIBracket(seasonId)
  }

  /**
   * 生成MSI对阵
   */
  async function generateMSI(request: GenerateMSIRequest) {
    loading.value = true
    error.value = null

    try {
      const { msiApi } = await import('@/api')
      const response = await msiApi.generateMSI(request)

      if (response.data) {
        msiBrackets.value.set(request.seasonId, response.data)
        currentBracket.value = response.data
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '生成MSI对阵失败'
      console.error('Failed to generate MSI:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 检查是否可以生成MSI
   */
  async function checkMSIEligibility(seasonId: string) {
    try {
      const { msiApi } = await import('@/api')
      const response = await msiApi.checkMSIEligibility(seasonId)
      return response.data
    } catch (err: any) {
      error.value = err.message || '检查MSI资格失败'
      console.error('Failed to check MSI eligibility:', err)
      throw err
    }
  }

  /**
   * 获取有资格参加MSI的队伍
   */
  async function getQualifiedTeams(seasonId: string) {
    try {
      const { msiApi } = await import('@/api')
      const response = await msiApi.getQualifiedTeams(seasonId)
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取晋级队伍失败'
      console.error('Failed to get qualified teams:', err)
      throw err
    }
  }

  /**
   * 模拟MSI单场比赛(BO5)
   */
  async function simulateMSIMatch(request: SimulateMSIMatchRequest) {
    loading.value = true
    error.value = null

    try {
      const { msiApi } = await import('@/api')
      const response = await msiApi.simulateMSIMatch(request)

      if (response.data) {
        // 更新当前对阵信息
        if (currentBracket.value) {
          // 更新比赛结果
          updateMatchInBracket(currentBracket.value, response.data.match)

          // 如果MSI完成,更新最终排名
          if (response.data.isMSIComplete && response.data.finalStandings) {
            currentBracket.value.champion = response.data.finalStandings.champion
            currentBracket.value.runnerUp = response.data.finalStandings.runnerUp
            currentBracket.value.thirdPlace = response.data.finalStandings.thirdPlace
            currentBracket.value.fourthPlace = response.data.finalStandings.fourthPlace
            currentBracket.value.loserRound2 = response.data.finalStandings.loserRound2
            currentBracket.value.loserRound1 = response.data.finalStandings.loserRound1
            currentBracket.value.status = 'completed'
          }
        }

        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      console.error('Failed to simulate MSI match:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新对阵中的比赛信息
   */
  function updateMatchInBracket(bracket: MSIBracket, match: MSIMatch) {
    bracket.rounds.forEach(round => {
      const index = round.matches.findIndex(m => m.id === match.id)
      if (index !== -1) {
        round.matches[index] = match
      }
    })
  }

  /**
   * 设置当前MSI
   */
  function setCurrentBracket(bracket: MSIBracket | null) {
    currentBracket.value = bracket
  }

  /**
   * 根据赛季ID获取MSI对阵
   */
  function getBracketBySeasonId(seasonId: string): MSIBracket | undefined {
    return msiBrackets.value.get(seasonId)
  }

  /**
   * 清空所有数据
   */
  function clearAll() {
    msiBrackets.value.clear()
    currentBracket.value = null
    error.value = null
  }

  /**
   * 重置错误
   */
  function resetError() {
    error.value = null
  }

  return {
    // 状态
    loading,
    error,
    msiBrackets,
    currentBracket,

    // 计算属性
    currentQualifiedTeams,
    legendaryGroup,
    challengerGroup,
    qualifierGroup,
    currentRounds,
    isMSIComplete,
    champion,
    runnerUp,
    thirdPlace,

    // 动作
    fetchMSIBracket,
    fetchMSIBracketBySeason,
    generateMSI,
    checkMSIEligibility,
    getQualifiedTeams,
    simulateMSIMatch,
    setCurrentBracket,
    getBracketBySeasonId,
    clearAll,
    resetError
  }
})
