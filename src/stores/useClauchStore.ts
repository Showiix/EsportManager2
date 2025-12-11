import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { clauchApi } from '@/api'
import type {
  ClauchBracket,
  GenerateClauchRequest,
  SimulateClauchMatchRequest,
} from '@/types'

export const useClauchStore = defineStore('clauch', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)

  // C洲际赛数据(按赛季ID存储)
  const clauchBrackets = ref<Map<string, ClauchBracket>>(new Map())

  // 当前选中的C洲际赛
  const currentBracket = ref<ClauchBracket | null>(null)

  // 计算属性
  const currentQualifiedTeams = computed(() => {
    return currentBracket.value?.qualifiedTeams || []
  })

  const currentGroups = computed(() => {
    return currentBracket.value?.groups || []
  })

  const currentGroupStandings = computed(() => {
    return currentBracket.value?.groupStandings || []
  })

  const eastBracket = computed(() => {
    return currentBracket.value?.knockoutEast
  })

  const westBracket = computed(() => {
    return currentBracket.value?.knockoutWest
  })

  const currentRounds = computed(() => {
    return currentBracket.value?.rounds || []
  })

  const isClauchComplete = computed(() => {
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

  const fourthPlace = computed(() => {
    return currentBracket.value?.fourthPlace || null
  })

  // 动作

  /**
   * 获取C洲际赛对阵信息
   */
  async function fetchClauchBracket(seasonId: string) {
    loading.value = true
    error.value = null

    try {
      const response = await clauchApi.getClauchBracket(seasonId)

      if (response.data) {
        clauchBrackets.value.set(seasonId, response.data)
        currentBracket.value = response.data
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '获取C洲际赛信息失败'
      console.error('Failed to fetch Clauch bracket:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 根据赛季代码获取C洲际赛信息（如"S1"）
   */
  async function fetchClauchBracketBySeason(seasonCode: string) {
    // 从eventStore获取实际的season_id
    const { useEventStore } = await import('./useEventStore')
    const eventStore = useEventStore()

    const season = eventStore.seasons.find(s => s.seasonCode === seasonCode)
    if (!season) {
      throw new Error(`未找到赛季 ${seasonCode}`)
    }

    const seasonId = String(season.id)
    console.log(`[ClauchStore] fetchClauchBracketBySeason: ${seasonCode} -> ID: ${seasonId}`)
    return await fetchClauchBracket(seasonId)
  }

  /**
   * 生成C洲际赛对阵
   */
  async function generateClauch(request: GenerateClauchRequest) {
    loading.value = true
    error.value = null

    try {
      const response = await clauchApi.generateClauch(request)

      if (response.data) {
        clauchBrackets.value.set(request.seasonId, response.data)
        currentBracket.value = response.data
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '生成C洲际赛对阵失败'
      console.error('Failed to generate Clauch:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 检查是否可以生成C洲际赛
   */
  async function checkClauchEligibility(seasonId: string) {
    try {
      const response = await clauchApi.checkClauchEligibility(seasonId)
      return response.data
    } catch (err: any) {
      error.value = err.message || '检查C洲际赛资格失败'
      console.error('Failed to check Clauch eligibility:', err)
      throw err
    }
  }

  /**
   * 获取有资格参加C洲际赛的队伍
   */
  async function getQualifiedTeams(seasonId: string) {
    try {
      const response = await clauchApi.getQualifiedTeams(seasonId)
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取晋级队伍失败'
      console.error('Failed to get qualified teams:', err)
      throw err
    }
  }

  /**
   * 模拟C洲际赛单场比赛
   */
  async function simulateClauchMatch(request: SimulateClauchMatchRequest) {
    loading.value = true
    error.value = null

    try {
      const response = await clauchApi.simulateClauchMatch(request)

      // 模拟成功后，重新获取完整数据以确保前后端同步
      if (response.data && currentBracket.value) {
        await fetchClauchBracket(currentBracket.value.seasonId)
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      console.error('Failed to simulate Clauch match:', err)
      throw err
    } finally {
      loading.value = false
    }
  }


  /**
   * 获取小组积分榜
   */
  async function getGroupStandings(clauchId: string, groupName?: string) {
    try {
      const response = await clauchApi.getGroupStandings(clauchId, groupName)
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取小组积分榜失败'
      console.error('Failed to get group standings:', err)
      throw err
    }
  }

  /**
   * 生成淘汰赛对阵
   */
  async function generateKnockout(clauchId: string) {
    loading.value = true
    error.value = null

    try {
      const response = await clauchApi.generateKnockout(clauchId)

      // 重新获取C洲际赛数据
      if (currentBracket.value) {
        await fetchClauchBracket(currentBracket.value.seasonId)
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成淘汰赛对阵失败'
      console.error('Failed to generate knockout:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置当前C洲际赛
   */
  function setCurrentBracket(bracket: ClauchBracket | null) {
    currentBracket.value = bracket
  }

  /**
   * 根据赛季ID获取C洲际赛对阵
   */
  function getBracketBySeasonId(seasonId: string): ClauchBracket | undefined {
    return clauchBrackets.value.get(seasonId)
  }

  /**
   * 清空所有数据
   */
  function clearAll() {
    clauchBrackets.value.clear()
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
    clauchBrackets,
    currentBracket,

    // 计算属性
    currentQualifiedTeams,
    currentGroups,
    currentGroupStandings,
    eastBracket,
    westBracket,
    currentRounds,
    isClauchComplete,
    champion,
    runnerUp,
    thirdPlace,
    fourthPlace,

    // 动作
    fetchClauchBracket,
    fetchClauchBracketBySeason,
    generateClauch,
    checkClauchEligibility,
    getQualifiedTeams,
    simulateClauchMatch,
    getGroupStandings,
    generateKnockout,
    setCurrentBracket,
    getBracketBySeasonId,
    clearAll,
    resetError
  }
})
