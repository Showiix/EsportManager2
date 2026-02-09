import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Region, Team } from '@/types'
import { regionApi, mockData } from '@/api'
import { useTeamStore } from './useTeamStore'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('RegionStore')

export const useRegionStore = defineStore('region', () => {
  // 状态
  const regions = ref<Region[]>([])
  const loading = ref(false)
  const selectedRegion = ref<Region | null>(null)

  // 计算属性
  const regionOptions = computed(() =>
    regions.value.map(region => ({
      label: region.name,
      value: region.id
    }))
  )

  const regionStatistics = computed(() => {
    // 获取 teamStore 来访问战队数据
    const teamStore = useTeamStore()
    const teamsByRegion = teamStore.teamsByRegion
    
    const stats = regions.value.map(region => {
      const regionTeams = teamsByRegion[region.id] || []
      const teamCount = regionTeams.length
      
      const stat = {
        regionId: region.id,
        regionName: region.name,
        totalTeams: teamCount,
        averageStrength: teamCount > 0 
          ? regionTeams.reduce((sum, team) => sum + team.strength, 0) / teamCount 
          : 0,
        averageWinRate: teamCount > 0 
          ? regionTeams.reduce((sum, team) => sum + (team.statistics?.winRate || 0), 0) / teamCount 
          : 0,
        totalMatches: regionTeams.reduce((sum, team) => sum + (team.statistics?.totalMatches || 0), 0),
        totalPoints: regionTeams.reduce((sum, team) => sum + (team.statistics?.totalPoints || 0), 0),
      }
      
      logger.debug('赛区统计', { region: region.name, regionId: region.id, teamCount, averageStrength: stat.averageStrength.toFixed(1) })
      return stat
    })
    
    return stats
  })

  // 方法
  const fetchRegions = async () => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 300))
        regions.value = mockData.regions.map(region => ({
          ...region,
          teams: mockData.teams.filter(team => team.regionId === region.id)
        }))
        return
      }

      // 调用真实API
      const response = await regionApi.getRegions()
      // 确保 id 为字符串类型
      regions.value = (response.data || []).map((region: any) => ({
        ...region,
        id: String(region.id)
      }))
      logger.info('加载赛区完成', { regions: regions.value.map(r => ({ id: r.id, name: r.name })) })
    } catch (error) {
      handleError(error, {
        component: 'RegionStore',
        userAction: '加载赛区列表'
      })
      throw error
    } finally {
      loading.value = false
    }
  }

  const fetchRegion = async (id: string) => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 200))
        const region = mockData.regions.find(r => r.id === id)
        if (region) {
          selectedRegion.value = {
            ...region,
            teams: mockData.teams.filter(team => team.regionId === id)
          }
        }
        return selectedRegion.value
      }

      // 调用真实API
      const response = await regionApi.getRegion(id)
      selectedRegion.value = response.data ?? null
      return selectedRegion.value
    } catch (error) {
      handleError(error, {
        component: 'RegionStore',
        userAction: '获取赛区详情',
        silent: true
      })
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateRegion = async (id: string, updates: Partial<Region>) => {
    loading.value = true
    try {
      // 根据环境变量决定是否使用mock数据
      if (import.meta.env.VITE_USE_MOCK === 'true') {
        await new Promise(resolve => setTimeout(resolve, 300))
        const index = regions.value.findIndex(region => region.id === id)
        if (index !== -1) {
          regions.value[index] = { ...regions.value[index], ...updates } as Region
          if (selectedRegion.value?.id === id) {
            selectedRegion.value = regions.value[index] || null
          }
          return regions.value[index]
        }
        throw new Error('Region not found')
      }

      // 调用真实API
      const response = await regionApi.updateRegion(id, updates)
      const updatedRegion = response.data

      const index = regions.value.findIndex(region => region.id === id)
      if (index !== -1 && updatedRegion) {
        regions.value[index] = updatedRegion
      }
      if (selectedRegion.value?.id === id) {
        selectedRegion.value = updatedRegion ?? null
      }
      return updatedRegion
    } catch (error) {
      handleError(error, {
        component: 'RegionStore',
        userAction: '更新赛区'
      })
      throw error
    } finally {
      loading.value = false
    }
  }

  const getRegionById = (id: string) => {
    return regions.value.find(region => region.id === id)
  }

  const getRegionTeams = (regionId: string) => {
    const region = regions.value.find(r => r.id === regionId)
    return region?.teams || []
  }

  const clearSelectedRegion = () => {
    selectedRegion.value = null
  }

  /**
   * 清除所有状态（切换存档时调用）
   */
  const clearAll = () => {
    regions.value = []
    loading.value = false
    selectedRegion.value = null
  }

  // 更新赛区中的战队（当战队发生变化时）
  const updateRegionTeams = (regionId: string, teams: Team[]) => {
    const region = regions.value.find(r => r.id === regionId)
    if (region) {
      region.teams = teams
    }
  }

  return {
    // 状态
    regions,
    loading,
    selectedRegion,

    // 计算属性
    regionOptions,
    regionStatistics,

    // 方法
    fetchRegions,
    fetchRegion,
    updateRegion,
    getRegionById,
    getRegionTeams,
    clearAll,
    clearSelectedRegion,
    updateRegionTeams,
  }
})