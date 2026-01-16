/**
 * 比赛详情 Store
 * 管理比赛详情数据，支持 localStorage 和数据库持久化
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { MatchDetail, SeasonMatchSummary, GameDetail } from '@/types/matchDetail'
import type { PlayerPerformance } from '@/types/player'
import { matchDetailsApi, type SaveMatchDetailsInput, type SaveGameInput, type SavePerformanceInput } from '@/api/tauri'
import { useGameStore } from './useGameStore'

const STORAGE_KEY = 'esport-match-details'

// 将 MatchDetail 转换为数据库保存格式
function convertToSaveInput(matchId: number, detail: MatchDetail): SaveMatchDetailsInput {
  const games: SaveGameInput[] = detail.games.map((game: GameDetail) => {
    const isTeamAWinner = game.winnerId === game.teamAId
    const winnerId = parseInt(isTeamAWinner ? game.teamAId : game.teamBId)
    const loserId = parseInt(isTeamAWinner ? game.teamBId : game.teamAId)

    // 找出 MVP - 影响力最高的选手
    const allPlayers = [...game.teamAPlayers, ...game.teamBPlayers]
    const mvpPlayer = allPlayers.reduce((max, p) => p.impactScore > max.impactScore ? p : max, allPlayers[0])

    // 关键选手 - 使用比赛级别的 keyPlayer 或默认使用 MVP
    const keyPlayerId = detail.keyPlayer?.gameNumber === game.gameNumber
      ? parseInt(detail.keyPlayer.playerId)
      : parseInt(mvpPlayer.playerId)

    const performances: SavePerformanceInput[] = allPlayers.map((perf: PlayerPerformance) => ({
      player_id: parseInt(perf.playerId),
      player_name: perf.playerName,
      team_id: parseInt(perf.teamId),
      team_name: '', // Will be filled from team data if needed
      position: perf.position,
      base_ability: perf.baseAbility,
      condition_bonus: perf.conditionBonus,
      stability_noise: perf.stabilityNoise,
      actual_ability: perf.actualAbility,
      impact_score: perf.impactScore,
      mvp_score: perf.mvpScore ?? 0,
      is_mvp: perf.playerId === mvpPlayer.playerId,
      is_key_player: perf.playerId === String(keyPlayerId),
      kills: perf.kills ?? null,
      deaths: perf.deaths ?? null,
      assists: perf.assists ?? null,
      cs: perf.cs ?? null,
      gold: perf.gold ?? null,
      damage_dealt: perf.damageDealt ?? null,
      damage_taken: perf.damageTaken ?? null,
      vision_score: perf.visionScore ?? null,
      traits_json: perf.traits ? JSON.stringify(perf.traits) : null,
      activated_traits_json: perf.activatedTraits ? JSON.stringify(perf.activatedTraits) : null
    }))

    return {
      game_number: game.gameNumber,
      winner_team_id: winnerId,
      loser_team_id: loserId,
      duration_minutes: null,
      mvp_player_id: parseInt(mvpPlayer.playerId),
      key_player_id: keyPlayerId,
      performances
    }
  })

  return {
    match_id: matchId,
    games
  }
}

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

  // 保存比赛详情（同时保存到 localStorage 和数据库）
  const saveMatchDetail = async (matchId: string | number, detail: MatchDetail) => {
    detail.matchId = matchId
    matchDetails.value.set(matchId, detail)
    saveToStorage()
    console.log(`保存比赛详情到本地: ${matchId}`)

    // 同时保存到数据库
    try {
      const gameStore = useGameStore()
      const saveId = gameStore.currentSave?.id
      if (saveId) {
        // 从 matchId 中提取数字 ID（如 "spring-123" -> 123）
        const numericMatchId = typeof matchId === 'number'
          ? matchId
          : parseInt(String(matchId).replace(/\D/g, ''))

        if (!isNaN(numericMatchId) && numericMatchId > 0) {
          const input = convertToSaveInput(numericMatchId, detail)
          await matchDetailsApi.saveMatchDetails(saveId, input)
          console.log(`保存比赛详情到数据库: matchId=${numericMatchId}`)
        }
      }
    } catch (error) {
      console.error('保存比赛详情到数据库失败:', error)
      // 不影响本地存储，仅记录错误
    }
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
