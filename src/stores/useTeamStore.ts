import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Team, QueryOptions } from '@/types'
import { teamApi, mockData } from '@/api'

export const useTeamStore = defineStore('team', () => {
  // 状态
  const teams = ref<Team[]>([])
  const loading = ref(false)
  const selectedTeam = ref<Team | null>(null)
  const filters = ref<QueryOptions>({
    page: 1,
    limit: 100, // 增加到100以获取所有战队
    search: '',
    sortBy: 'name',
    sortOrder: 'asc',
    filters: {}
  })

  // 计算属性
  const filteredTeams = computed(() => {
    let result = [...teams.value]

    // 搜索过滤
    if (filters.value.search) {
      const searchTerm = filters.value.search.toLowerCase()
      result = result.filter(team =>
        team.name.toLowerCase().includes(searchTerm)
      )
    }

    // 赛区过滤
    if (filters.value.filters?.regionId) {
      result = result.filter(team =>
        team.regionId === filters.value.filters?.regionId
      )
    }

    // 战力值范围过滤
    if (filters.value.filters?.minStrength !== undefined) {
      result = result.filter(team =>
        team.strength >= (filters.value.filters?.minStrength || 0)
      )
    }
    if (filters.value.filters?.maxStrength !== undefined) {
      result = result.filter(team =>
        team.strength <= (filters.value.filters?.maxStrength || 100)
      )
    }

    // 胜率范围过滤
    if (filters.value.filters?.minWinRate) {
      result = result.filter(team =>
        (team.statistics?.winRate || 0) >= (filters.value.filters?.minWinRate || 0)
      )
    }

    // 排序
    if (filters.value.sortBy) {
      result.sort((a, b) => {
        let aValue: any, bValue: any

        switch (filters.value.sortBy) {
          case 'name':
            aValue = a.name
            bValue = b.name
            break
          case 'strength':
            aValue = a.strength
            bValue = b.strength
            break
          case 'winRate':
            aValue = a.statistics?.winRate || 0
            bValue = b.statistics?.winRate || 0
            break
          case 'totalPoints':
            aValue = a.statistics?.totalPoints || 0
            bValue = b.statistics?.totalPoints || 0
            break
          default:
            return 0
        }

        if (filters.value.sortOrder === 'desc') {
          return aValue < bValue ? 1 : -1
        }
        return aValue > bValue ? 1 : -1
      })
    }

    return result
  })

  const teamsByRegion = computed(() => {
    const grouped: Record<string, Team[]> = {}
    teams.value.forEach(team => {
      if (!grouped[team.regionId]) {
        grouped[team.regionId] = []
      }
      grouped[team.regionId]!.push(team)
    })
    return grouped
  })

  const totalTeams = computed(() => teams.value.length)

  const teamStatsSummary = computed(() => {
    return {
      totalTeams: teams.value.length,
      averageStrength: teams.value.reduce((sum, team) => sum + team.strength, 0) / teams.value.length || 0,
      averageWinRate: teams.value.reduce((sum, team) => sum + (team.statistics?.winRate || 0), 0) / teams.value.length || 0,
      totalMatches: teams.value.reduce((sum, team) => sum + (team.statistics?.totalMatches || 0), 0),
    }
  })

  // 方法
  const fetchTeams = async (options?: QueryOptions) => {
    loading.value = true
    try {
      if (options) {
        filters.value = { ...filters.value, ...options }
      }

      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        // 模拟API延迟
        await new Promise(resolve => setTimeout(resolve, 500))
        teams.value = mockData.teams
        return
      }

      // 调用真实API
      const response = await teamApi.getTeams(filters.value)
      const backendTeams = response.data || []
      
      // 获取当前赛季积分数据
      let pointsData: any[] = []
      try {
        const pointsResponse = await fetch('http://localhost:8000/api/points/season/2024')
        const pointsResult = await pointsResponse.json()
        pointsData = pointsResult.data || []
      } catch (error) {
        console.warn('Failed to fetch points data:', error)
      }

      // 创建积分映射表（以teamId为key）
      const pointsMap = new Map()
      pointsData.forEach((p: any) => {
        pointsMap.set(String(p.teamId), p.totalPoints || 0)
      })

      console.log('Teams from API:', backendTeams.slice(0, 2))
      console.log('Points map size:', pointsMap.size)

      // apiClient的拦截器已经将字段转换为驼峰命名
      // 所以直接使用转换后的字段名
      teams.value = backendTeams.map((team: any) => {
        const teamId = String(team.id)
        const totalPoints = pointsMap.get(teamId) || 0
        
        console.log(`Team ${team.name}: regionId=${team.regionId}, strength=${team.strength}, totalPoints=${totalPoints}`)
        
        return {
          id: teamId,
          name: team.name,
          shortName: team.shortName || team.name,
          regionId: String(team.regionId), // apiClient已经转换为regionId
          strength: team.strength || 0, // apiClient已经转换为strength
          createdAt: team.createdAt,
          statistics: {
            totalMatches: team.statistics?.totalMatches || 0,
            wins: team.statistics?.wins || 0,
            losses: team.statistics?.losses || 0,
            winRate: team.statistics?.winRate || 0,
            totalPoints: totalPoints, // 从积分系统获取
            seasonPoints: 0,
            intercontinentalPoints: 0
          }
        }
      })
      console.log(`加载了 ${teams.value.length} 支队伍，积分数据: ${pointsData.length} 条`)
    } catch (error) {
      console.error('Failed to fetch teams:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const fetchTeam = async (id: string) => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 300))
        selectedTeam.value = mockData.teams.find(team => team.id === id) || null
        return selectedTeam.value
      }

      // 调用真实API
      const response = await teamApi.getTeam(id)
      selectedTeam.value = response.data
      return selectedTeam.value
    } catch (error) {
      console.error('Failed to fetch team:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const createTeam = async (teamData: Omit<Team, 'id' | 'statistics'>) => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 500))
        const newTeam: Team = {
          ...teamData,
          id: String(Date.now()),
          statistics: {
            totalMatches: 0,
            wins: 0,
            losses: 0,
            winRate: 0,
            totalPoints: 0,
            seasonPoints: 0,
            intercontinentalPoints: 0,
          }
        }
        teams.value.push(newTeam)
        return newTeam
      }

      // 调用真实API
      const response = await teamApi.createTeam(teamData)
      const newTeam = response.data
      teams.value.push(newTeam)
      return newTeam
    } catch (error) {
      console.error('Failed to create team:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateTeam = async (id: string, updates: Partial<Team>) => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 300))
        const index = teams.value.findIndex(team => team.id === id)
        if (index !== -1) {
          teams.value[index] = { ...teams.value[index], ...updates } as Team
          if (selectedTeam.value?.id === id) {
            selectedTeam.value = teams.value[index] || null
          }
          return teams.value[index]
        }
        throw new Error('Team not found')
      }

      // 调用真实API
      const response = await teamApi.updateTeam(id, updates)
      const updatedTeam = response.data

      const index = teams.value.findIndex(team => team.id === id)
      if (index !== -1) {
        teams.value[index] = updatedTeam
      }
      if (selectedTeam.value?.id === id) {
        selectedTeam.value = updatedTeam
      }
      return updatedTeam
    } catch (error) {
      console.error('Failed to update team:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const deleteTeam = async (id: string) => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 300))
        const index = teams.value.findIndex(team => team.id === id)
        if (index !== -1) {
          teams.value.splice(index, 1)
          if (selectedTeam.value?.id === id) {
            selectedTeam.value = null
          }
          return
        }
        throw new Error('Team not found')
      }

      // 调用真实API
      await teamApi.deleteTeam(id)
      const index = teams.value.findIndex(team => team.id === id)
      if (index !== -1) {
        teams.value.splice(index, 1)
      }
      if (selectedTeam.value?.id === id) {
        selectedTeam.value = null
      }
    } catch (error) {
      console.error('Failed to delete team:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateFilters = (newFilters: Partial<QueryOptions>) => {
    filters.value = {
      ...filters.value,
      ...newFilters,
      filters: {
        ...filters.value.filters,
        ...newFilters.filters
      }
    }
  }

  const clearFilters = () => {
    filters.value = {
      page: 1,
      limit: 100, // 增加到100以获取所有战队
      search: '',
      sortBy: 'name',
      sortOrder: 'asc',
      filters: {}
    }
  }

  const getTeamsByRegion = (regionId: string) => {
    return teams.value.filter(team => team.regionId === regionId)
  }

  const clearSelectedTeam = () => {
    selectedTeam.value = null
  }

  return {
    // 状态
    teams,
    loading,
    selectedTeam,
    filters,

    // 计算属性
    filteredTeams,
    teamsByRegion,
    totalTeams,
    teamStatsSummary,

    // 方法
    fetchTeams,
    fetchTeam,
    createTeam,
    updateTeam,
    deleteTeam,
    updateFilters,
    clearFilters,
    getTeamsByRegion,
    clearSelectedTeam,
  }
})