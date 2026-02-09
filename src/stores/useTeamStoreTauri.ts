/**
 * Team Store (Tauri版本) - 管理队伍和选手数据
 * 使用 Tauri IPC 与 Rust 后端通信
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  teamApi,
  playerApi,
  queryApi,
  type Team,
  type Player,
  type TeamRoster,
  type Region
} from '@/api/tauri'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('TeamStore')

export const useTeamStoreTauri = defineStore('teamTauri', () => {
  // ========================================
  // State
  // ========================================

  // 所有赛区
  const regions = ref<Region[]>([])

  // 当前选中的赛区
  const selectedRegion = ref<Region | null>(null)

  // 当前赛区的队伍列表
  const teams = ref<Team[]>([])

  // 当前选中的队伍
  const selectedTeam = ref<Team | null>(null)

  // 当前队伍阵容
  const roster = ref<TeamRoster | null>(null)

  // 当前选中的选手
  const selectedPlayer = ref<Player | null>(null)

  // 加载状态
  const isLoading = ref(false)

  // 错误信息
  const error = ref<string | null>(null)

  // 搜索关键词
  const searchQuery = ref('')

  // ========================================
  // Computed
  // ========================================

  // 按战力排序的队伍
  const teamsByPower = computed(() => {
    return [...teams.value].sort((a, b) => b.power_rating - a.power_rating)
  })

  // 按胜率排序的队伍
  const teamsByWinRate = computed(() => {
    return [...teams.value].sort((a, b) => b.win_rate - a.win_rate)
  })

  // 首发阵容
  const starters = computed(() => roster.value?.starters ?? [])

  // 替补阵容
  const substitutes = computed(() => roster.value?.substitutes ?? [])

  // 全部选手
  const allPlayers = computed(() => [
    ...(roster.value?.starters ?? []),
    ...(roster.value?.substitutes ?? [])
  ])

  // 过滤后的队伍（搜索）
  const filteredTeams = computed(() => {
    if (!searchQuery.value) return teams.value
    const query = searchQuery.value.toLowerCase()
    return teams.value.filter(team =>
      team.name.toLowerCase().includes(query) ||
      (team.short_name?.toLowerCase().includes(query) ?? false)
    )
  })

  // ========================================
  // Actions
  // ========================================

  /**
   * 加载所有赛区
   */
  const loadRegions = async () => {
    isLoading.value = true
    error.value = null

    try {
      regions.value = await queryApi.getAllRegions()
      logger.debug('加载赛区', { count: regions.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load regions'
      handleError(e, {
        component: 'TeamStore',
        userAction: '加载赛区',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载所有队伍（所有赛区）
   */
  const loadAllTeams = async () => {
    isLoading.value = true
    error.value = null

    try {
      const rawTeams = await teamApi.getAllTeams()
      // 格式化战力值为两位小数
      teams.value = rawTeams.map(team => ({
        ...team,
        power_rating: Math.round(team.power_rating * 100) / 100
      }))
      logger.debug('加载所有队伍', { count: teams.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load all teams'
      handleError(e, {
        component: 'TeamStore',
        userAction: '加载所有队伍',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 选择赛区并加载队伍
   */
  const selectRegion = async (regionId: number) => {
    isLoading.value = true
    error.value = null

    try {
      // 获取赛区详情（包含队伍）
      const detail = await queryApi.getRegionDetail(regionId)
      selectedRegion.value = detail.region
      // 格式化战力值为两位小数
      teams.value = detail.teams.map(team => ({
        ...team,
        power_rating: Math.round(team.power_rating * 100) / 100
      }))
      logger.debug('选择赛区', { region: detail.region.name, teamCount: teams.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load region'
      handleError(e, {
        component: 'TeamStore',
        userAction: '选择赛区',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 按赛区ID加载队伍
   */
  const loadTeamsByRegion = async (regionId: number) => {
    isLoading.value = true
    error.value = null

    try {
      const rawTeams = await teamApi.getTeamsByRegion(regionId)
      // 格式化战力值为两位小数
      teams.value = rawTeams.map(team => ({
        ...team,
        power_rating: Math.round(team.power_rating * 100) / 100
      }))
      logger.debug('按赛区加载队伍', { regionId, count: teams.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load teams'
      handleError(e, {
        component: 'TeamStore',
        userAction: '按赛区加载队伍',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 选择队伍并加载阵容
   */
  const selectTeam = async (teamId: number) => {
    isLoading.value = true
    error.value = null

    try {
      // 并行获取队伍详情和阵容
      const [team, teamRoster] = await Promise.all([
        teamApi.getTeam(teamId),
        teamApi.getTeamRoster(teamId)
      ])

      selectedTeam.value = team
      roster.value = teamRoster
      logger.debug('选择队伍', { team: team.name, playerCount: allPlayers.value.length })
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load team'
      handleError(e, {
        component: 'TeamStore',
        userAction: '选择队伍',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载队伍阵容
   */
  const loadRoster = async (teamId: number) => {
    isLoading.value = true
    error.value = null

    try {
      roster.value = await teamApi.getTeamRoster(teamId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load roster'
      handleError(e, {
        component: 'TeamStore',
        userAction: '加载阵容',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 选择选手
   */
  const selectPlayer = async (playerId: number) => {
    isLoading.value = true
    error.value = null

    try {
      selectedPlayer.value = await playerApi.getPlayer(playerId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load player'
      handleError(e, {
        component: 'TeamStore',
        userAction: '选择选手',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 设置首发/替补
   */
  const setStarter = async (teamId: number, playerId: number, isStarter: boolean) => {
    isLoading.value = true
    error.value = null

    try {
      await teamApi.setStarter(teamId, playerId, isStarter)
      // 重新加载阵容
      await loadRoster(teamId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update starter'
      handleError(e, {
        component: 'TeamStore',
        userAction: '设置首发'
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 搜索队伍
   */
  const searchTeams = async (query: string) => {
    if (!query.trim()) {
      // 如果没有搜索词，显示当前赛区的队伍
      return
    }

    isLoading.value = true
    error.value = null

    try {
      teams.value = await queryApi.searchTeams(query)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to search teams'
      handleError(e, {
        component: 'TeamStore',
        userAction: '搜索队伍',
        silent: true
      })
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 搜索选手
   */
  const searchPlayers = async (query: string) => {
    if (!query.trim()) {
      return []
    }

    try {
      return await queryApi.searchPlayers(query)
    } catch (e) {
      logger.error('搜索选手失败', { query, error: e })
      return []
    }
  }

  /**
   * 清除所有状态（切换存档时调用）
   */
  const clearAll = () => {
    regions.value = []
    selectedRegion.value = null
    teams.value = []
    selectedTeam.value = null
    roster.value = null
    selectedPlayer.value = null
    isLoading.value = false
    error.value = null
    searchQuery.value = ''
  }

  /**
   * 清除选中状态
   */
  const clearSelection = () => {
    selectedTeam.value = null
    selectedPlayer.value = null
    roster.value = null
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
    regions,
    selectedRegion,
    teams,
    selectedTeam,
    roster,
    selectedPlayer,
    isLoading,
    error,
    searchQuery,

    // Computed
    teamsByPower,
    teamsByWinRate,
    starters,
    substitutes,
    allPlayers,
    filteredTeams,

    // Actions
    loadRegions,
    loadAllTeams,
    selectRegion,
    loadTeamsByRegion,
    selectTeam,
    loadRoster,
    selectPlayer,
    setStarter,
    searchTeams,
    searchPlayers,
    clearAll,
    clearSelection,
    clearError,
  }
})
