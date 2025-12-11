/**
 * 比赛详情 Store
 * 管理比赛详情数据，支持 localStorage 持久化
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { MatchDetail, GameDetail, SeasonMatchSummary } from '@/types/matchDetail'

const STORAGE_KEY = 'esport-match-details'

export const useMatchDetailStore = defineStore('matchDetail', () => {
  // 状态
  const matchDetails = ref<Map<string | number, MatchDetail>>(new Map())
  const currentMatchDetail = ref<MatchDetail | null>(null)
  const dialogVisible = ref(false)
  const loading = ref(false)

  // 计算属性
  const totalMatches = computed(() => matchDetails.value.size)

  const recentMatches = computed(() => {
    const all = Array.from(matchDetails.value.values())
    return all
      .sort((a, b) => {
        const dateA = new Date(a.playedAt || 0).getTime()
        const dateB = new Date(b.playedAt || 0).getTime()
        return dateB - dateA
      })
      .slice(0, 10)
  })

  // 获取赛季统计汇总
  const getSeasonSummary = (seasonId: string): SeasonMatchSummary => {
    const seasonMatches = Array.from(matchDetails.value.values())
      .filter(m => m.seasonId === seasonId)

    let totalGames = 0
    let upsetCount = 0
    let totalPowerDiff = 0
    let totalPerfDiff = 0

    seasonMatches.forEach(match => {
      match.games.forEach(game => {
        totalGames++
        if (game.isUpset) upsetCount++
        totalPowerDiff += Math.abs(game.powerDifference)
        totalPerfDiff += Math.abs(game.performanceDifference)
      })
    })

    return {
      seasonId,
      totalMatches: seasonMatches.length,
      totalGames,
      upsetCount,
      avgPowerDifference: totalGames > 0 ? Math.round((totalPowerDiff / totalGames) * 10) / 10 : 0,
      avgPerformanceDifference: totalGames > 0 ? Math.round((totalPerfDiff / totalGames) * 10) / 10 : 0
    }
  }

  // 保存比赛详情
  const saveMatchDetail = (matchId: string | number, detail: MatchDetail) => {
    detail.matchId = matchId
    matchDetails.value.set(matchId, detail)
    saveToStorage()
    console.log(`保存比赛详情: ${matchId}`)
  }

  // 获取比赛详情
  const getMatchDetail = (matchId: string | number): MatchDetail | null => {
    return matchDetails.value.get(matchId) || null
  }

  // 显示比赛详情弹窗
  const showMatchDetail = (matchId: string | number) => {
    const detail = getMatchDetail(matchId)
    if (detail) {
      currentMatchDetail.value = detail
      dialogVisible.value = true
    } else {
      console.warn(`比赛详情不存在: ${matchId}`)
    }
  }

  // 关闭弹窗
  const closeDialog = () => {
    dialogVisible.value = false
    currentMatchDetail.value = null
  }

  // 删除比赛详情
  const deleteMatchDetail = (matchId: string | number) => {
    matchDetails.value.delete(matchId)
    saveToStorage()
  }

  // 清空所有数据
  const clearAll = () => {
    matchDetails.value.clear()
    currentMatchDetail.value = null
    localStorage.removeItem(STORAGE_KEY)
  }

  // 清空赛季数据
  const clearSeason = (seasonId: string) => {
    const keysToDelete: (string | number)[] = []
    matchDetails.value.forEach((detail, key) => {
      if (detail.seasonId === seasonId) {
        keysToDelete.push(key)
      }
    })
    keysToDelete.forEach(key => matchDetails.value.delete(key))
    saveToStorage()
  }

  // 获取队伍比赛历史
  const getTeamMatches = (teamId: string, limit = 10): MatchDetail[] => {
    return Array.from(matchDetails.value.values())
      .filter(m => m.teamAId === teamId || m.teamBId === teamId)
      .sort((a, b) => {
        const dateA = new Date(a.playedAt || 0).getTime()
        const dateB = new Date(b.playedAt || 0).getTime()
        return dateB - dateA
      })
      .slice(0, limit)
  }

  // 获取队伍对战记录
  const getHeadToHead = (teamAId: string, teamBId: string): MatchDetail[] => {
    return Array.from(matchDetails.value.values())
      .filter(m =>
        (m.teamAId === teamAId && m.teamBId === teamBId) ||
        (m.teamAId === teamBId && m.teamBId === teamAId)
      )
      .sort((a, b) => {
        const dateA = new Date(a.playedAt || 0).getTime()
        const dateB = new Date(b.playedAt || 0).getTime()
        return dateB - dateA
      })
  }

  // 获取赛事比赛列表
  const getTournamentMatches = (tournamentType: string, seasonId?: string): MatchDetail[] => {
    return Array.from(matchDetails.value.values())
      .filter(m => {
        if (m.tournamentType !== tournamentType) return false
        if (seasonId && m.seasonId !== seasonId) return false
        return true
      })
      .sort((a, b) => {
        const dateA = new Date(a.playedAt || 0).getTime()
        const dateB = new Date(b.playedAt || 0).getTime()
        return dateA - dateB
      })
  }

  // 统计爆冷率
  const getUpsetRate = (seasonId?: string): { total: number; upsets: number; rate: number } => {
    let matches = Array.from(matchDetails.value.values())
    if (seasonId) {
      matches = matches.filter(m => m.seasonId === seasonId)
    }

    let totalGames = 0
    let upsets = 0

    matches.forEach(match => {
      match.games.forEach(game => {
        totalGames++
        if (game.isUpset) upsets++
      })
    })

    return {
      total: totalGames,
      upsets,
      rate: totalGames > 0 ? Math.round((upsets / totalGames) * 1000) / 10 : 0
    }
  }

  // 从 localStorage 加载数据
  const loadFromStorage = () => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const data = JSON.parse(stored)
        if (data.matchDetails) {
          matchDetails.value = new Map(Object.entries(data.matchDetails))
          console.log(`从存储加载了 ${matchDetails.value.size} 场比赛详情`)
        }
      }
    } catch (error) {
      console.error('Failed to load match details from storage:', error)
    }
  }

  // 保存到 localStorage
  const saveToStorage = () => {
    try {
      const data = {
        matchDetails: Object.fromEntries(matchDetails.value),
        lastUpdated: new Date().toISOString()
      }
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
    } catch (error) {
      console.error('Failed to save match details to storage:', error)
    }
  }

  // 导出数据（用于备份）
  const exportData = (): string => {
    const data = {
      matchDetails: Object.fromEntries(matchDetails.value),
      exportedAt: new Date().toISOString()
    }
    return JSON.stringify(data, null, 2)
  }

  // 导入数据（从备份恢复）
  const importData = (jsonString: string) => {
    try {
      const data = JSON.parse(jsonString)
      if (data.matchDetails) {
        matchDetails.value = new Map(Object.entries(data.matchDetails))
        saveToStorage()
        console.log(`导入了 ${matchDetails.value.size} 场比赛详情`)
      }
    } catch (error) {
      console.error('Failed to import match details:', error)
      throw new Error('导入数据格式错误')
    }
  }

  return {
    // 状态
    matchDetails,
    currentMatchDetail,
    dialogVisible,
    loading,

    // 计算属性
    totalMatches,
    recentMatches,

    // 方法
    getSeasonSummary,
    saveMatchDetail,
    getMatchDetail,
    showMatchDetail,
    closeDialog,
    deleteMatchDetail,
    clearAll,
    clearSeason,
    getTeamMatches,
    getHeadToHead,
    getTournamentMatches,
    getUpsetRate,
    loadFromStorage,
    saveToStorage,
    exportData,
    importData
  }
})
