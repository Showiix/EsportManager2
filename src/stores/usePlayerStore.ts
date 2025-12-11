/**
 * 选手数据 Store
 * 管理选手数据、生成模拟数据、记录比赛表现
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Player, PlayerPosition, PlayerTalent, PlayerSeasonStats } from '@/types/player'
import { PlayerEngine } from '@/engines/PlayerEngine'

// 位置列表
const POSITIONS: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']

// 常见游戏ID前缀（用于生成模拟数据）
const GAME_ID_PREFIXES = [
  'Faker', 'Deft', 'Ruler', 'Gumayusi', 'Zeus',
  'Keria', 'Oner', 'Canyon', 'Chovy', 'Peyz',
  'Viper', 'Meiko', 'Scout', 'Bin', 'Breathe',
  'Knight', 'JackeyLove', 'Rookie', 'TheShy', 'Uzi',
  'Caps', 'Jankos', 'Perkz', 'Rekkles', 'Hylissang'
]

export const usePlayerStore = defineStore('player', () => {
  // 状态
  const players = ref<Player[]>([])
  const playerSeasonStats = ref<Map<string, PlayerSeasonStats>>(new Map())
  const loading = ref(false)
  const currentSeasonId = ref('2024')
  const updateTrigger = ref(0) // 用于触发响应式更新

  // 计算属性
  const playersByTeam = computed(() => {
    const grouped: Record<string, Player[]> = {}
    players.value.forEach(player => {
      if (!grouped[player.teamId]) {
        grouped[player.teamId] = []
      }
      grouped[player.teamId]!.push(player)
    })
    return grouped
  })

  const playersByRegion = computed(() => {
    const grouped: Record<string, Player[]> = {}
    players.value.forEach(player => {
      if (!grouped[player.regionId]) {
        grouped[player.regionId] = []
      }
      grouped[player.regionId]!.push(player)
    })
    return grouped
  })

  const totalPlayers = computed(() => players.value.length)

  // 获取队伍首发阵容（5人）
  const getTeamRoster = (teamId: string): Player[] => {
    const teamPlayers = playersByTeam.value[teamId] || []
    // 按位置排序，确保每个位置有一人
    const roster: Player[] = []
    for (const pos of POSITIONS) {
      const player = teamPlayers.find(p => p.position === pos)
      if (player) {
        roster.push(player)
      }
    }
    return roster
  }

  // 获取选手赛季统计
  const getPlayerSeasonStats = (playerId: string, seasonId?: string): PlayerSeasonStats | null => {
    const key = `${playerId}-${seasonId || currentSeasonId.value}`
    return playerSeasonStats.value.get(key) || null
  }

  // 计算年度Top得分（加权计算：平均影响力70% + 冠军加成30%）
  const calculateYearlyTopScore = (avgImpact: number, internationalTitles: number, regionalTitles: number): number => {
    const championBonus = internationalTitles * 3 + regionalTitles * 1
    const weightedScore = avgImpact * 0.7 + championBonus * 0.3
    return Math.round(weightedScore * 10) / 10
  }

  // 记录选手比赛表现
  const recordPerformance = (
    playerId: string,
    playerName: string,
    teamId: string,
    position: PlayerPosition,
    impactScore: number,
    performance: number,
    seasonId?: string,
    regionId?: string
  ) => {
    const key = `${playerId}-${seasonId || currentSeasonId.value}`
    const existing = playerSeasonStats.value.get(key)

    if (existing) {
      existing.gamesPlayed++
      existing.totalImpact += impactScore
      existing.avgImpact = Math.round((existing.totalImpact / existing.gamesPlayed) * 10) / 10
      existing.bestPerformance = Math.round(Math.max(existing.bestPerformance, performance) * 10) / 10
      existing.worstPerformance = Math.round(Math.min(existing.worstPerformance, performance) * 10) / 10
      // 更新稳定性评分（基于发挥波动）
      const variance = existing.bestPerformance - existing.worstPerformance
      existing.consistencyScore = Math.round(Math.max(0, 100 - variance * 2) * 10) / 10
      if (regionId) existing.regionId = regionId
      // 更新年度Top得分
      existing.yearlyTopScore = calculateYearlyTopScore(
        existing.avgImpact,
        existing.internationalTitles || 0,
        existing.regionalTitles || 0
      )
    } else {
      const initAvgImpact = Math.round(impactScore * 10) / 10
      playerSeasonStats.value.set(key, {
        playerId,
        playerName,
        seasonId: seasonId || currentSeasonId.value,
        teamId,
        regionId,
        position,
        matchesPlayed: 0,
        gamesPlayed: 1,
        totalImpact: initAvgImpact,
        avgImpact: initAvgImpact,
        avgPerformance: Math.round(performance * 10) / 10,
        bestPerformance: Math.round(performance * 10) / 10,
        worstPerformance: Math.round(performance * 10) / 10,
        consistencyScore: 100,
        internationalTitles: 0,
        regionalTitles: 0,
        championBonus: 0,
        yearlyTopScore: initAvgImpact
      })
    }
    // 触发响应式更新
    updateTrigger.value++
  }

  // 记录冠军荣誉
  const recordChampionship = (
    teamId: string,
    isInternational: boolean, // true: 国际赛冠军, false: 赛区冠军
    seasonId?: string
  ) => {
    const sid = seasonId || currentSeasonId.value
    const roster = getTeamRoster(teamId)

    roster.forEach(player => {
      const key = `${player.id}-${sid}`
      const stats = playerSeasonStats.value.get(key)
      if (stats) {
        if (isInternational) {
          stats.internationalTitles = (stats.internationalTitles || 0) + 1
        } else {
          stats.regionalTitles = (stats.regionalTitles || 0) + 1
        }
        // 重新计算冠军加成和年度Top得分
        stats.championBonus = (stats.internationalTitles || 0) * 3 + (stats.regionalTitles || 0) * 1
        stats.yearlyTopScore = calculateYearlyTopScore(
          stats.avgImpact,
          stats.internationalTitles || 0,
          stats.regionalTitles || 0
        )
      }
    })
    updateTrigger.value++
  }

  // 批量记录队伍表现
  const recordTeamPerformances = (
    performances: Array<{
      playerId: string
      playerName: string
      teamId: string
      position: PlayerPosition
      impactScore: number
      actualAbility: number
    }>,
    seasonId?: string
  ) => {
    performances.forEach(p => {
      recordPerformance(
        p.playerId,
        p.playerName,
        p.teamId,
        p.position,
        p.impactScore,
        p.actualAbility,
        seasonId
      )
    })
  }

  // 增加比赛场次计数
  const incrementMatchCount = (teamId: string, seasonId?: string) => {
    const roster = getTeamRoster(teamId)
    roster.forEach(player => {
      const key = `${player.id}-${seasonId || currentSeasonId.value}`
      const stats = playerSeasonStats.value.get(key)
      if (stats) {
        stats.matchesPlayed++
      }
    })
  }

  // 获取年度Top排行（按年度Top得分排序）
  const getSeasonImpactRanking = (seasonId?: string, limit = 20): PlayerSeasonStats[] => {
    const sid = seasonId || currentSeasonId.value
    const allStats: PlayerSeasonStats[] = []

    playerSeasonStats.value.forEach((stats, key) => {
      if (key.endsWith(`-${sid}`)) {
        allStats.push(stats)
      }
    })

    return allStats
      .filter(s => s.gamesPlayed >= 1) // 至少参与1局
      .sort((a, b) => (b.yearlyTopScore || b.avgImpact) - (a.yearlyTopScore || a.avgImpact))
      .slice(0, limit)
  }

  // 按位置获取排行
  const getPositionRanking = (
    position: PlayerPosition,
    seasonId?: string,
    limit = 10
  ): PlayerSeasonStats[] => {
    const sid = seasonId || currentSeasonId.value
    const allStats: PlayerSeasonStats[] = []

    playerSeasonStats.value.forEach((stats, key) => {
      if (key.endsWith(`-${sid}`) && stats.position === position) {
        allStats.push(stats)
      }
    })

    return allStats
      .filter(s => s.gamesPlayed >= 1)
      .sort((a, b) => (b.yearlyTopScore || b.avgImpact) - (a.yearlyTopScore || a.avgImpact))
      .slice(0, limit)
  }

  // 生成随机天赋标签
  const generateTalent = (): PlayerTalent => {
    const rand = Math.random()
    if (rand < 0.1) return 'GENIUS' // 10% 天才
    if (rand < 0.5) return 'NORMAL' // 40% 普通
    return 'ORDINARY' // 50% 平庸
  }

  // 生成单个模拟选手
  const generateMockPlayer = (
    teamId: string,
    teamName: string,
    regionId: string,
    regionName: string,
    position: PlayerPosition,
    index: number
  ): Player => {
    const age = Math.floor(Math.random() * 15) + 18 // 18-32岁
    const talent = generateTalent()

    // 根据天赋和年龄确定能力值范围
    let baseAbility: number
    if (talent === 'GENIUS') {
      baseAbility = Math.floor(Math.random() * 15) + 80 // 80-95
    } else if (talent === 'NORMAL') {
      baseAbility = Math.floor(Math.random() * 20) + 65 // 65-85
    } else {
      baseAbility = Math.floor(Math.random() * 20) + 50 // 50-70
    }

    // 年龄修正（巅峰期能力更高）
    if (age >= 22 && age <= 27) {
      baseAbility = Math.min(100, baseAbility + 5)
    } else if (age > 30) {
      baseAbility = Math.max(40, baseAbility - (age - 30) * 2)
    }

    const stability = PlayerEngine.getBaseStabilityByAge(age)
    const condition = PlayerEngine.generateRandomCondition(age)
    const potential = talent === 'GENIUS' ? Math.floor(Math.random() * 10) + 90
                    : talent === 'NORMAL' ? Math.floor(Math.random() * 20) + 70
                    : Math.floor(Math.random() * 20) + 50

    // 生成游戏ID
    const prefixIndex = (index * 7 + position.charCodeAt(0)) % GAME_ID_PREFIXES.length
    const gameId = `${GAME_ID_PREFIXES[prefixIndex]}${Math.floor(Math.random() * 100)}`

    return {
      id: `player-${teamId}-${position}`,
      gameId,
      name: `选手${index + 1}`,
      teamId,
      teamName,
      position,
      regionId,
      regionName,
      ability: baseAbility,
      potential,
      stability,
      condition,
      age,
      tag: talent
    }
  }

  // 为队伍生成完整阵容
  const generateTeamRoster = (
    teamId: string,
    teamName: string,
    regionId: string,
    regionName: string
  ): Player[] => {
    return POSITIONS.map((pos, idx) =>
      generateMockPlayer(teamId, teamName, regionId, regionName, pos, idx)
    )
  }

  // 批量生成所有队伍的选手
  const generateAllPlayers = (
    teams: Array<{ id: string; name: string; regionId: string; regionName?: string }>
  ) => {
    loading.value = true
    try {
      const allPlayers: Player[] = []

      teams.forEach(team => {
        const roster = generateTeamRoster(
          team.id,
          team.name,
          team.regionId,
          team.regionName || team.regionId
        )
        allPlayers.push(...roster)
      })

      players.value = allPlayers
      console.log(`生成了 ${allPlayers.length} 名选手数据`)
    } finally {
      loading.value = false
    }
  }

  // 设置选手数据（从外部导入）
  const setPlayers = (newPlayers: Player[]) => {
    players.value = newPlayers
  }

  // 更新选手状态
  const updatePlayerCondition = (playerId: string, newCondition: number) => {
    const player = players.value.find(p => p.id === playerId)
    if (player) {
      player.condition = Math.max(-10, Math.min(10, newCondition))
    }
  }

  // 随机刷新所有选手状态
  const refreshAllConditions = () => {
    players.value.forEach(player => {
      player.condition = PlayerEngine.generateRandomCondition(player.age)
    })
  }

  // 清除统计数据
  const clearSeasonStats = (seasonId?: string) => {
    if (seasonId) {
      const keysToDelete: string[] = []
      playerSeasonStats.value.forEach((_, key) => {
        if (key.endsWith(`-${seasonId}`)) {
          keysToDelete.push(key)
        }
      })
      keysToDelete.forEach(key => playerSeasonStats.value.delete(key))
    } else {
      playerSeasonStats.value.clear()
    }
  }

  // 从 localStorage 恢复数据
  const loadFromStorage = () => {
    try {
      const stored = localStorage.getItem('esport-player-stats')
      if (stored) {
        const data = JSON.parse(stored)
        if (data.playerSeasonStats) {
          playerSeasonStats.value = new Map(Object.entries(data.playerSeasonStats))
        }
      }
    } catch (error) {
      console.error('Failed to load player stats from storage:', error)
    }
  }

  // 保存到 localStorage
  const saveToStorage = () => {
    try {
      const data = {
        playerSeasonStats: Object.fromEntries(playerSeasonStats.value),
        lastUpdated: new Date().toISOString()
      }
      localStorage.setItem('esport-player-stats', JSON.stringify(data))
    } catch (error) {
      console.error('Failed to save player stats to storage:', error)
    }
  }

  return {
    // 状态
    players,
    playerSeasonStats,
    loading,
    currentSeasonId,
    updateTrigger,

    // 计算属性
    playersByTeam,
    playersByRegion,
    totalPlayers,

    // 方法
    getTeamRoster,
    getPlayerSeasonStats,
    recordPerformance,
    recordChampionship,
    recordTeamPerformances,
    incrementMatchCount,
    getSeasonImpactRanking,
    getPositionRanking,
    generateMockPlayer,
    generateTeamRoster,
    generateAllPlayers,
    setPlayers,
    updatePlayerCondition,
    refreshAllConditions,
    clearSeasonStats,
    loadFromStorage,
    saveToStorage
  }
})
