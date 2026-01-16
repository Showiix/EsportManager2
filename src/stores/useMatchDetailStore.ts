/**
 * 比赛详情 Store
 * 管理比赛详情数据，支持 localStorage 和数据库持久化
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { MatchDetail, SeasonMatchSummary, GameDetail } from '@/types/matchDetail'
import type { PlayerPerformance, PlayerPosition, TraitType, ActivatedTrait } from '@/types/player'
import { matchDetailsApi, type SaveMatchDetailsInput, type SaveGameInput, type SavePerformanceInput, type MatchFullDetails, type GameDetailWithPerformances } from '@/api/tauri'
import { useGameStore } from './useGameStore'

const STORAGE_KEY = 'esport-match-details'

// 根据 saveId 生成存储 key
function getStorageKey(saveId: string | undefined): string {
  return saveId ? `${STORAGE_KEY}-${saveId}` : STORAGE_KEY
}

// 解析特性 JSON 字符串
function parseTraitsJson(json: string | null): TraitType[] | undefined {
  if (!json) return undefined
  try {
    return JSON.parse(json) as TraitType[]
  } catch {
    return undefined
  }
}

// 解析激活特性 JSON 字符串
function parseActivatedTraitsJson(json: string | null): ActivatedTrait[] | undefined {
  if (!json) return undefined
  try {
    return JSON.parse(json) as ActivatedTrait[]
  } catch {
    return undefined
  }
}

// 将数据库格式转换为 MatchDetail 格式（直接使用数据库中的完整数据）
function convertFromDbFormat(matchId: number, dbData: MatchFullDetails): MatchDetail {
  const games: GameDetail[] = dbData.games.map((gameData: GameDetailWithPerformances) => {
    const game = gameData.game
    const performances = gameData.performances

    // 分离两队的选手表现
    const teamAId = String(game.winner_team_id)
    const teamBId = String(game.loser_team_id)

    // 获取队伍名称（从第一个选手的 team_name 字段获取）
    const teamAPerf = performances.find(p => p.team_id === game.winner_team_id)
    const teamBPerf = performances.find(p => p.team_id === game.loser_team_id)
    const teamAName = teamAPerf?.team_name || ''
    const teamBName = teamBPerf?.team_name || ''

    const teamAPlayers: PlayerPerformance[] = performances
      .filter(p => p.team_id === game.winner_team_id)
      .map(p => ({
        playerId: String(p.player_id),
        playerName: p.player_name,           // 直接使用数据库中的名称
        teamId: String(p.team_id),
        position: p.position as PlayerPosition,
        baseAbility: p.base_ability,         // 直接使用数据库中的基础能力
        conditionBonus: p.condition_bonus,   // 直接使用数据库中的状态加成
        stabilityNoise: p.stability_noise,   // 直接使用数据库中的稳定性波动
        actualAbility: p.actual_ability,     // 直接使用数据库中的实际发挥
        impactScore: p.impact_score,
        mvpScore: p.mvp_score,
        // 详细战斗数据
        kills: p.kills ?? undefined,
        deaths: p.deaths ?? undefined,
        assists: p.assists ?? undefined,
        cs: p.cs ?? undefined,
        gold: p.gold ?? undefined,
        damageDealt: p.damage_dealt ?? undefined,
        damageTaken: p.damage_taken ?? undefined,
        visionScore: p.vision_score ?? undefined,
        // 特性系统
        traits: parseTraitsJson(p.traits_json),
        activatedTraits: parseActivatedTraitsJson(p.activated_traits_json)
      }))

    const teamBPlayers: PlayerPerformance[] = performances
      .filter(p => p.team_id === game.loser_team_id)
      .map(p => ({
        playerId: String(p.player_id),
        playerName: p.player_name,
        teamId: String(p.team_id),
        position: p.position as PlayerPosition,
        baseAbility: p.base_ability,
        conditionBonus: p.condition_bonus,
        stabilityNoise: p.stability_noise,
        actualAbility: p.actual_ability,
        impactScore: p.impact_score,
        mvpScore: p.mvp_score,
        kills: p.kills ?? undefined,
        deaths: p.deaths ?? undefined,
        assists: p.assists ?? undefined,
        cs: p.cs ?? undefined,
        gold: p.gold ?? undefined,
        damageDealt: p.damage_dealt ?? undefined,
        damageTaken: p.damage_taken ?? undefined,
        visionScore: p.vision_score ?? undefined,
        // 特性系统
        traits: parseTraitsJson(p.traits_json),
        activatedTraits: parseActivatedTraitsJson(p.activated_traits_json)
      }))

    // 计算队伍表现和战力
    const teamAPerfAvg = teamAPlayers.reduce((sum, p) => sum + p.impactScore, 0) / (teamAPlayers.length || 1)
    const teamBPerfAvg = teamBPlayers.reduce((sum, p) => sum + p.impactScore, 0) / (teamBPlayers.length || 1)
    const teamAPower = teamAPlayers.reduce((sum, p) => sum + p.baseAbility, 0) / (teamAPlayers.length || 1)
    const teamBPower = teamBPlayers.reduce((sum, p) => sum + p.baseAbility, 0) / (teamBPlayers.length || 1)

    return {
      gameNumber: game.game_number,
      teamAId,
      teamAName,
      teamAPower,
      teamAPerformance: teamAPerfAvg,
      teamAPlayers,
      teamBId,
      teamBName,
      teamBPower,
      teamBPerformance: teamBPerfAvg,
      teamBPlayers,
      winnerId: teamAId,
      winnerName: teamAName,
      powerDifference: teamAPower - teamBPower,
      performanceDifference: teamAPerfAvg - teamBPerfAvg,
      isUpset: teamBPower > teamAPower // 战力低的队伍获胜
    }
  })

  // 计算最终比分
  const teamIds = new Set<string>()
  const teamNames = new Map<string, string>()
  games.forEach(g => {
    teamIds.add(g.teamAId)
    teamIds.add(g.teamBId)
    if (g.teamAName) teamNames.set(g.teamAId, g.teamAName)
    if (g.teamBName) teamNames.set(g.teamBId, g.teamBName)
  })
  const [firstTeamId, secondTeamId] = Array.from(teamIds)

  let scoreA = 0
  let scoreB = 0
  games.forEach(g => {
    if (g.winnerId === firstTeamId) scoreA++
    else scoreB++
  })

  const winnerId = scoreA > scoreB ? firstTeamId : secondTeamId
  const winnerName = teamNames.get(winnerId) || ''

  // 计算比赛MVP（仅从胜方队伍中选择）
  let mvpPlayerId: string | undefined
  let mvpPlayerName: string | undefined
  let mvpTeamId: string | undefined
  let mvpTotalImpact: number | undefined

  // 收集胜方队伍所有选手的累计影响力
  const winnerPlayerStats = new Map<string, { name: string, totalImpact: number, gameCount: number }>()

  games.forEach(g => {
    // 获取本局胜方队伍的选手
    const winnerPlayers = g.winnerId === winnerId
      ? (g.teamAId === winnerId ? g.teamAPlayers : g.teamBPlayers)
      : []

    winnerPlayers.forEach(p => {
      const existing = winnerPlayerStats.get(p.playerId)
      if (existing) {
        existing.totalImpact += p.impactScore
        existing.gameCount++
      } else {
        winnerPlayerStats.set(p.playerId, {
          name: p.playerName,
          totalImpact: p.impactScore,
          gameCount: 1
        })
      }
    })
  })

  // 找出平均影响力最高的选手作为MVP
  let maxAvgImpact = -Infinity
  winnerPlayerStats.forEach((stats, playerId) => {
    const avgImpact = stats.totalImpact / stats.gameCount
    if (avgImpact > maxAvgImpact) {
      maxAvgImpact = avgImpact
      mvpPlayerId = playerId
      mvpPlayerName = stats.name
      mvpTeamId = winnerId
      mvpTotalImpact = stats.totalImpact
    }
  })

  return {
    matchId,
    teamAId: firstTeamId || '',
    teamAName: teamNames.get(firstTeamId) || '',
    teamBId: secondTeamId || '',
    teamBName: teamNames.get(secondTeamId) || '',
    bestOf: games.length > 3 ? 5 : (games.length > 1 ? 3 : 1),
    games,
    finalScoreA: scoreA,
    finalScoreB: scoreB,
    winnerId,
    winnerName,
    mvpPlayerId,
    mvpPlayerName,
    mvpTeamId,
    mvpTotalImpact
  }
}

// 将 MatchDetail 转换为数据库保存格式（包含完整的选手和队伍信息）
function convertToSaveInput(matchId: number, detail: MatchDetail): SaveMatchDetailsInput {
  const games: SaveGameInput[] = detail.games.map((game: GameDetail) => {
    const isTeamAWinner = game.winnerId === game.teamAId
    const winnerId = parseInt(isTeamAWinner ? game.teamAId : game.teamBId)
    const loserId = parseInt(isTeamAWinner ? game.teamBId : game.teamAId)

    // 找出 MVP - 仅从胜方队伍中选择影响力最高的选手
    const allPlayers = [...game.teamAPlayers, ...game.teamBPlayers]
    const winnerPlayers = isTeamAWinner ? game.teamAPlayers : game.teamBPlayers
    const mvpPlayer = winnerPlayers.length > 0
      ? winnerPlayers.reduce((max, p) => p.impactScore > max.impactScore ? p : max, winnerPlayers[0])
      : allPlayers[0]

    // 关键选手 - 使用比赛级别的 keyPlayer 或默认使用 MVP
    const keyPlayerId = detail.keyPlayer?.gameNumber === game.gameNumber
      ? parseInt(detail.keyPlayer.playerId)
      : parseInt(mvpPlayer.playerId)

    // 构建完整的选手表现数据，包含所有详细信息
    const performances: SavePerformanceInput[] = allPlayers.map((perf: PlayerPerformance) => ({
      player_id: parseInt(perf.playerId),
      player_name: perf.playerName,                    // 选手名称快照
      team_id: parseInt(perf.teamId),
      team_name: perf.teamId === game.teamAId ? game.teamAName : game.teamBName,  // 队伍名称快照
      position: perf.position,
      base_ability: perf.baseAbility,                  // 基础能力值
      condition_bonus: perf.conditionBonus,            // 状态加成
      stability_noise: perf.stabilityNoise,            // 稳定性波动
      actual_ability: perf.actualAbility,              // 实际发挥值
      impact_score: perf.impactScore,                  // 影响力得分
      mvp_score: perf.mvpScore || perf.impactScore,    // MVP 得分
      is_mvp: perf.playerId === mvpPlayer.playerId,
      is_key_player: perf.playerId === String(keyPlayerId),
      // 详细战斗数据
      kills: perf.kills ?? null,
      deaths: perf.deaths ?? null,
      assists: perf.assists ?? null,
      cs: perf.cs ?? null,
      gold: perf.gold ?? null,
      damage_dealt: perf.damageDealt ?? null,
      damage_taken: perf.damageTaken ?? null,
      vision_score: perf.visionScore ?? null,
      // 特性系统
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

  // 获取比赛详情（同步版本，只从内存获取）
  const getMatchDetail = (matchId: string | number): MatchDetail | null => {
    return matchDetails.value.get(matchId) || null
  }

  // 从数据库加载单个比赛详情
  const loadMatchDetailFromDb = async (matchId: string | number): Promise<MatchDetail | null> => {
    // 先检查内存中是否已有
    const cached = matchDetails.value.get(matchId)
    if (cached) {
      return cached
    }

    // 尝试从数据库加载
    try {
      const gameStore = useGameStore()
      const saveId = gameStore.currentSave?.id
      if (!saveId) {
        console.warn('无法从数据库加载比赛详情: 没有当前存档')
        return null
      }

      // 从 matchId 中提取数字 ID
      const numericMatchId = typeof matchId === 'number'
        ? matchId
        : parseInt(String(matchId).replace(/\D/g, ''))

      if (isNaN(numericMatchId) || numericMatchId <= 0) {
        console.warn('无效的 matchId:', matchId)
        return null
      }

      console.log(`从数据库加载比赛详情: matchId=${numericMatchId}, saveId=${saveId}`)
      const dbData = await matchDetailsApi.getMatchDetails(saveId, numericMatchId)

      if (dbData && dbData.games && dbData.games.length > 0) {
        const detail = convertFromDbFormat(numericMatchId, dbData)
        // 直接使用数据库中的完整数据，不再需要 enrichMatchDetail
        // 缓存到内存
        matchDetails.value.set(matchId, detail)
        console.log(`成功从数据库加载比赛详情: matchId=${numericMatchId}`)
        return detail
      }

      console.log(`数据库中没有找到比赛详情: matchId=${numericMatchId}`)
      return null
    } catch (error) {
      console.error('从数据库加载比赛详情失败:', error)
      return null
    }
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

  // 清空所有数据（当前存档）
  const clearAll = () => {
    const gameStore = useGameStore()
    const saveId = gameStore.currentSave?.id
    const storageKey = getStorageKey(saveId)

    matchDetails.value.clear()
    currentMatchDetail.value = null
    localStorage.removeItem(storageKey)
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

  // 从 localStorage 加载数据（按 saveId 区分）
  const loadFromStorage = () => {
    try {
      const gameStore = useGameStore()
      const saveId = gameStore.currentSave?.id
      const storageKey = getStorageKey(saveId)

      // 先清空内存，确保不会混用不同存档的数据
      matchDetails.value.clear()

      const stored = localStorage.getItem(storageKey)
      if (stored) {
        const data = JSON.parse(stored)
        if (data.matchDetails) {
          matchDetails.value = new Map(Object.entries(data.matchDetails))
          console.log(`从存储加载了 ${matchDetails.value.size} 场比赛详情 (saveId: ${saveId})`)
        }
      }
    } catch (error) {
      console.error('Failed to load match details from storage:', error)
    }
  }

  // 保存到 localStorage（按 saveId 区分）
  const saveToStorage = () => {
    try {
      const gameStore = useGameStore()
      const saveId = gameStore.currentSave?.id
      const storageKey = getStorageKey(saveId)

      const data = {
        matchDetails: Object.fromEntries(matchDetails.value),
        lastUpdated: new Date().toISOString()
      }
      localStorage.setItem(storageKey, JSON.stringify(data))
    } catch (error) {
      console.error('Failed to save match details to storage:', error)
    }
  }

  // 清空当前存档的缓存（用于切换存档时）
  const clearCache = () => {
    matchDetails.value.clear()
    currentMatchDetail.value = null
    console.log('已清空比赛详情缓存')
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
    loadMatchDetailFromDb,
    showMatchDetail,
    closeDialog,
    deleteMatchDetail,
    clearAll,
    clearCache,
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
