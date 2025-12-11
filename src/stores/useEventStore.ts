import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  type Season,
  type Competition,
  type CompetitionType,
  type CompetitionStatus,
  type CompetitionFormat,
} from '@/types'

export const useEventStore = defineStore('event', () => {
  // 状态
  const seasons = ref<Season[]>([])
  const competitions = ref<Competition[]>([])
  const currentSeason = ref<Season | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const activeSeasons = computed(() =>
    seasons.value.filter(season => season.status === 'active')
  )

  const currentSeasonCompetitions = computed(() =>
    competitions.value.filter(comp => comp.seasonId === currentSeason.value?.id)
  )

  const ongoingCompetitions = computed(() =>
    competitions.value.filter(comp => comp.status === 'ongoing')
  )

  const completedCompetitions = computed(() =>
    competitions.value.filter(comp => comp.status === 'completed')
  )

  // 赛事操作
  const actions = {
    // 获取赛季列表
    async fetchSeasons() {
      loading.value = true
      error.value = null
      try {
        // 从后端API获取真实数据
        const { seasonApi } = await import('@/api')
        const response = await seasonApi.getSeasons()

        console.log('[EventStore] 从API获取到seasons:', response.data)

        if (response.data && Array.isArray(response.data)) {
          seasons.value = response.data
        } else {
          console.warn('[EventStore] API返回的seasons数据格式异常:', response)
          seasons.value = []
        }

        // 设置当前赛季为第一个active状态的赛季，如果没有则使用第一个
        if (!currentSeason.value && seasons.value.length > 0) {
          const activeSeason = seasons.value.find(s => s.status === 'active')
          currentSeason.value = activeSeason || seasons.value[0] || null
        }
      } catch (err) {
        error.value = '获取赛季列表失败'
        console.error(err)
      } finally {
        loading.value = false
      }
    },

    // 获取赛事列表
    async fetchCompetitions(seasonId?: string | number) {
      loading.value = true
      error.value = null
      try {
        // 从后端API获取真实数据
        const { competitionApi } = await import('@/api')
        const response = seasonId 
          ? await competitionApi.getCompetitions({ seasonId: String(seasonId), limit: 1000 } as any)
          : await competitionApi.getCompetitions({ limit: 1000 } as any)

        console.log('[EventStore] 从API获取到competitions:', response.data)

        if (response.data && Array.isArray(response.data)) {
          competitions.value = response.data
        } else if (response.data && 'items' in response.data) {
          // 处理分页响应
          competitions.value = (response.data as any).items
        } else {
          console.warn('[EventStore] API返回的competitions数据格式异常:', response)
          competitions.value = []
        }
      } catch (err) {
        error.value = '获取赛事列表失败'
        console.error(err)
      } finally {
        loading.value = false
      }
    },

    // 创建新赛事
    async createCompetition(competitionData: {
      name: string
      type: CompetitionType
      format: CompetitionFormat
      seasonId: string | number
      teamIds: string[]
      regionId?: string
      stage?: string
    }) {
      loading.value = true
      error.value = null
      try {
        // 生成规范的赛事代码（匹配后端格式：S1-spring）
        const season = seasons.value.find(s => s.id === competitionData.seasonId)
        const seasonCode = season?.seasonCode || 'S1'
        const competitionCode = `${seasonCode}-${competitionData.type}`

        // Mock 创建
        const newCompetition = {
          id: Date.now(), // 模拟数据库自增ID
          competitionCode: competitionCode,
          name: competitionData.name,
          type: competitionData.type,
          format: competitionData.format,
          seasonId: competitionData.seasonId,
          regionId: competitionData.regionId || 'GLOBAL',
          stage: competitionData.stage || 'main',
          status: 'planned' as CompetitionStatus,
          teams: [],
          matches: []
        } as Competition
        competitions.value.push(newCompetition)
        return newCompetition
      } catch (err) {
        error.value = '创建赛事失败'
        console.error(err)
        throw err
      } finally {
        loading.value = false
      }
    },

    // 更新赛事
    async updateCompetition(id: string, updates: Partial<Competition>) {
      loading.value = true
      error.value = null
      try {
        const index = competitions.value.findIndex(comp => comp.id === id)
        if (index !== -1) {
          competitions.value[index] = { ...competitions.value[index], ...updates } as Competition
          return competitions.value[index]
        }
        throw new Error('赛事不存在')
      } catch (err) {
        error.value = '更新赛事失败'
        console.error(err)
        throw err
      } finally {
        loading.value = false
      }
    },

    // 删除赛事
    async deleteCompetition(id: string) {
      loading.value = true
      error.value = null
      try {
        const index = competitions.value.findIndex(comp => comp.id === id)
        if (index !== -1) {
          competitions.value.splice(index, 1)
        }
      } catch (err) {
        error.value = '删除赛事失败'
        console.error(err)
        throw err
      } finally {
        loading.value = false
      }
    },

    // 开始赛事
    async startCompetition(id: string) {
      return await this.updateCompetition(id, {
        status: 'ongoing' as CompetitionStatus
      })
    },

    // 结束赛事
    async finishCompetition(id: string) {
      return await this.updateCompetition(id, {
        status: 'completed' as CompetitionStatus
      })
    },

    // 设置当前赛季
    setCurrentSeason(season: Season) {
      currentSeason.value = season
    },

    // 清除错误
    clearError() {
      error.value = null
    },

    // 重置状态
    reset() {
      seasons.value = []
      competitions.value = []
      currentSeason.value = null
      loading.value = false
      error.value = null
    }
  }

  return {
    // 状态
    seasons,
    competitions,
    currentSeason,
    loading,
    error,

    // 计算属性
    activeSeasons,
    currentSeasonCompetitions,
    ongoingCompetitions,
    completedCompetitions,

    // 方法
    ...actions
  }
})