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
      console.log(`Loaded ${regions.value.length} regions`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load regions'
      console.error('Failed to load regions:', e)
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
      teams.value = detail.teams
      console.log(`Selected region: ${detail.region.name}, ${teams.value.length} teams`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load region'
      console.error('Failed to load region:', e)
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
      teams.value = await teamApi.getTeamsByRegion(regionId)
      console.log(`Loaded ${teams.value.length} teams for region ${regionId}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load teams'
      console.error('Failed to load teams:', e)
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
      console.log(`Selected team: ${team.name}, ${allPlayers.value.length} players`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load team'
      console.error('Failed to load team:', e)
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
      console.error('Failed to load roster:', e)
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
      console.error('Failed to load player:', e)
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
      console.error('Failed to update starter:', e)
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
      console.error('Failed to search teams:', e)
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
      console.error('Failed to search players:', e)
      return []
    }
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
    selectRegion,
    loadTeamsByRegion,
    selectTeam,
    loadRoster,
    selectPlayer,
    setStarter,
    searchTeams,
    searchPlayers,
    clearSelection,
    clearError,
  }
})
