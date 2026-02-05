/**
 * 选手数据 Store
 * 管理选手数据、生成模拟数据、记录比赛表现
 * 已迁移到 SQLite 数据库存储
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Player, PlayerPosition, PlayerTalent, PlayerSeasonStats } from '@/types/player'
import { PlayerEngine } from '@/engines/PlayerEngine'
import { getTeamPlayersConfig, type PlayerConfig } from '@/data/playerData'
import { statsApi, type RecordPerformanceParams } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('PlayerStore')

// 位置列表
const POSITIONS: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']

export const usePlayerStore = defineStore('player', () => {
  // 状态
  const players = ref<Player[]>([])
  const playerSeasonStats = ref<Map<string, PlayerSeasonStats>>(new Map())
  const loading = ref(false)
  const currentSeasonId = ref('1') // 与游戏系统保持一致，使用数字格式
  const updateTrigger = ref(0) // 用于触发响应式更新

  // 待保存的表现数据队列（用于批量保存）
  const pendingPerformances = ref<RecordPerformanceParams[]>([])
  const saveTimeout = ref<ReturnType<typeof setTimeout> | null>(null)

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

  // 获取选手赛季统计（从本地缓存获取，后台会从数据库同步）
  const getPlayerSeasonStats = (playerId: string, seasonId?: string): PlayerSeasonStats | null => {
    const key = `${playerId}-${seasonId || currentSeasonId.value}`
    return playerSeasonStats.value.get(key) || null
  }

  // 计算年度Top得分（综合三要素：影响力40% + 出场30% + 冠军30%）
  const calculateYearlyTopScore = (avgImpact: number, gamesPlayed: number, internationalTitles: number, regionalTitles: number): number => {
    const championBonus = internationalTitles * 3 + regionalTitles * 1
    const gamesBonus = gamesPlayed / 10
    const weightedScore = avgImpact * 0.4 + gamesBonus * 0.3 + championBonus * 0.3
    return Math.round(weightedScore * 10) / 10
  }

  // 记录选手比赛表现（同时更新本地缓存和数据库）
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
    const sid = seasonId || currentSeasonId.value
    const key = `${playerId}-${sid}`

    // 1. 更新本地缓存（即时响应）
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
        existing.gamesPlayed,
        existing.internationalTitles || 0,
        existing.regionalTitles || 0
      )
    } else {
      const initAvgImpact = Math.round(impactScore * 10) / 10
      playerSeasonStats.value.set(key, {
        playerId,
        playerName,
        seasonId: sid,
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

    // 2. 添加到待保存队列（异步批量保存到数据库）
    const parsedPlayerId = Number(playerId.replace('player-', '').split('-')[0]) || 0
    logger.debug('记录选手表现', { playerId: parsedPlayerId, playerName, teamId, impactScore })

    pendingPerformances.value.push({
      player_id: parsedPlayerId,
      player_name: playerName,
      team_id: Number(teamId) || 0,
      position: position,
      impact_score: impactScore,
      actual_ability: performance,
      season_id: Number(sid) || 1,
      region_id: regionId
    })

    // 设置延迟批量保存
    scheduleBatchSave()
  }

  // 延迟批量保存到数据库
  const scheduleBatchSave = () => {
    if (saveTimeout.value) {
      clearTimeout(saveTimeout.value)
    }
    saveTimeout.value = setTimeout(async () => {
      if (pendingPerformances.value.length > 0) {
        const toSave = [...pendingPerformances.value]
        pendingPerformances.value = []
        logger.debug('批量保存选手表现', { count: toSave.length })
        try {
          const result = await statsApi.batchRecordPerformance(toSave)
          logger.debug('批量保存成功', { result })
        } catch (error) {
          logger.error('批量保存选手表现失败', { error })
          // 保存失败时，重新加入队列
          pendingPerformances.value.push(...toSave)
        }
      }
    }, 500) // 500ms 后批量保存
  }

  // 立即保存所有待保存的数据
  const flushPendingPerformances = async () => {
    if (saveTimeout.value) {
      clearTimeout(saveTimeout.value)
      saveTimeout.value = null
    }
    if (pendingPerformances.value.length > 0) {
      const toSave = [...pendingPerformances.value]
      pendingPerformances.value = []
      try {
        await statsApi.batchRecordPerformance(toSave)
        logger.info('立即保存选手表现', { count: toSave.length })
      } catch (error) {
        logger.error('立即保存选手表现失败', { error })
      }
    }
  }

  // 记录冠军荣誉（同时更新本地缓存和数据库）
  const recordChampionship = async (
    teamId: string,
    isInternational: boolean, // true: 国际赛冠军, false: 赛区冠军
    seasonId?: string
  ) => {
    const sid = seasonId || currentSeasonId.value
    const roster = getTeamRoster(teamId)

    // 1. 更新本地缓存
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
          stats.gamesPlayed,
          stats.internationalTitles || 0,
          stats.regionalTitles || 0
        )
      }
    })
    updateTrigger.value++

    // 2. 保存到数据库
    try {
      await statsApi.recordChampionship(Number(teamId) || 0, isInternational, Number(sid) || 1)
      logger.info('记录冠军荣誉', { teamId, isInternational, season: sid })
    } catch (error) {
      logger.error('保存冠军荣誉失败', { error })
    }
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

  // 获取年度Top排行（优先从数据库获取，本地缓存作为后备）
  const getSeasonImpactRanking = async (seasonId?: string, limit = 20): Promise<PlayerSeasonStats[]> => {
    const sid = seasonId || currentSeasonId.value
    // 统一转换为数字格式用于本地缓存查询（本地缓存 key 格式是 playerId-1）
    const numericSid = sid.startsWith('S') ? sid.substring(1) : sid

    try {
      // 尝试从数据库获取
      const ranking = await statsApi.getSeasonImpactRanking(Number(numericSid) || 1, limit)

      // 如果数据库返回空，尝试本地缓存
      if (ranking.length === 0) {
        logger.debug('数据库返回空，使用本地缓存', { seasonId: numericSid })
        return getSeasonImpactRankingLocal(numericSid, limit)
      }

      // 转换为前端格式
      return ranking.map(r => ({
        playerId: String(r.player_id),
        playerName: r.player_name,
        seasonId: numericSid,
        teamId: String(r.team_id || ''),
        regionId: r.region_id || undefined,
        position: r.position as PlayerPosition,
        matchesPlayed: 0,
        gamesPlayed: r.games_played,
        totalImpact: r.avg_impact * r.games_played,
        avgImpact: r.avg_impact,
        avgPerformance: r.avg_performance,
        bestPerformance: 0,
        worstPerformance: 100,
        consistencyScore: r.consistency_score,
        internationalTitles: 0,
        regionalTitles: 0,
        championBonus: r.champion_bonus,
        yearlyTopScore: r.yearly_top_score
      }))
    } catch (error) {
      logger.warn('从数据库获取排行失败，使用本地缓存', { error })
      // 使用本地缓存作为后备
      return getSeasonImpactRankingLocal(numericSid, limit)
    }
  }

  // 本地缓存获取排行（后备方法）
  const getSeasonImpactRankingLocal = (seasonId: string, limit = 20): PlayerSeasonStats[] => {
    const allStats: PlayerSeasonStats[] = []

    playerSeasonStats.value.forEach((stats, key) => {
      if (key.endsWith(`-${seasonId}`)) {
        allStats.push(stats)
      }
    })

    return allStats
      .filter(s => s.gamesPlayed >= 1) // 至少参与1局
      .sort((a, b) => (b.yearlyTopScore || b.avgImpact) - (a.yearlyTopScore || a.avgImpact))
      .slice(0, limit)
  }

  // 按位置获取排行（优先从数据库获取）
  const getPositionRanking = async (
    position: PlayerPosition,
    seasonId?: string,
    limit = 10
  ): Promise<PlayerSeasonStats[]> => {
    const sid = seasonId || currentSeasonId.value
    // 统一转换为数字格式用于本地缓存查询
    const numericSid = sid.startsWith('S') ? sid.substring(1) : sid

    try {
      // 尝试从数据库获取
      const ranking = await statsApi.getPositionRanking(Number(numericSid) || 1, position, limit)

      // 如果数据库返回空，尝试本地缓存
      if (ranking.length === 0) {
        return getPositionRankingLocal(position, numericSid, limit)
      }

      // 转换为前端格式
      return ranking.map(r => ({
        playerId: String(r.player_id),
        playerName: r.player_name,
        seasonId: numericSid,
        teamId: String(r.team_id || ''),
        regionId: r.region_id || undefined,
        position: r.position as PlayerPosition,
        matchesPlayed: 0,
        gamesPlayed: r.games_played,
        totalImpact: r.avg_impact * r.games_played,
        avgImpact: r.avg_impact,
        avgPerformance: r.avg_performance,
        bestPerformance: 0,
        worstPerformance: 100,
        consistencyScore: r.consistency_score,
        internationalTitles: 0,
        regionalTitles: 0,
        championBonus: r.champion_bonus,
        yearlyTopScore: r.yearly_top_score
      }))
    } catch (error) {
      logger.warn('从数据库获取位置排行失败，使用本地缓存', { error })
      // 使用本地缓存作为后备
      return getPositionRankingLocal(position, numericSid, limit)
    }
  }

  // 本地缓存获取位置排行（后备方法）
  const getPositionRankingLocal = (position: PlayerPosition, seasonId: string, limit = 10): PlayerSeasonStats[] => {
    const allStats: PlayerSeasonStats[] = []

    playerSeasonStats.value.forEach((stats, key) => {
      if (key.endsWith(`-${seasonId}`) && stats.position === position) {
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

  // 从配置创建选手对象
  const createPlayerFromConfig = (
    config: PlayerConfig,
    teamId: string,
    teamName: string,
    regionId: string,
    regionName: string
  ): Player => {
    const stability = PlayerEngine.getBaseStabilityByAge(config.age)
    const condition = PlayerEngine.generateRandomCondition(config.age)

    return {
      id: `player-${teamId}-${config.position}`,
      gameId: config.gameId,
      name: config.name,
      teamId,
      teamName,
      position: config.position,
      regionId,
      regionName,
      ability: config.ability,
      potential: config.potential,
      stability,
      condition,
      age: config.age,
      tag: config.tag
    }
  }

  // 生成单个模拟选手（备用，当没有真实数据时使用）
  const generateMockPlayer = (
    teamId: string,
    teamName: string,
    regionId: string,
    regionName: string,
    position: PlayerPosition,
    _index: number
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

    // 生成游戏ID - 使用队伍简称+位置
    const teamShort = teamName.split(' ').map(w => w[0]).join('').toUpperCase()
    const gameId = `${teamShort}_${position}`

    return {
      id: `player-${teamId}-${position}`,
      gameId,
      name: `${teamShort} ${position}`,
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
    // 尝试从真实数据配置中获取
    const teamConfig = getTeamPlayersConfig(teamId)

    if (teamConfig && teamConfig.players.length === 5) {
      // 使用真实选手数据
      return teamConfig.players.map(playerConfig =>
        createPlayerFromConfig(playerConfig, teamId, teamName, regionId, regionName)
      )
    }

    // 没有真实数据时，使用备用生成方法
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
      logger.info('生成选手数据', { count: allPlayers.length })
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

  // 清除统计数据（同时清除本地缓存和数据库）
  const clearSeasonStats = async (seasonId?: string) => {
    if (seasonId) {
      const keysToDelete: string[] = []
      playerSeasonStats.value.forEach((_, key) => {
        if (key.endsWith(`-${seasonId}`)) {
          keysToDelete.push(key)
        }
      })
      keysToDelete.forEach(key => playerSeasonStats.value.delete(key))

      // 清除数据库
      try {
        await statsApi.clearSeasonStats(Number(seasonId) || 1)
        logger.info('清除赛季数据', { seasonId })
      } catch (error) {
        logger.error('清除数据库赛季数据失败', { error })
      }
    } else {
      playerSeasonStats.value.clear()
    }
  }

  // 清空所有数据
  const clearAll = () => {
    players.value = []
    playerSeasonStats.value.clear()
    localStorage.removeItem('esport-player-stats')
    updateTrigger.value++
  }

  // 从 localStorage 恢复数据（保留向后兼容）
  const loadFromStorage = () => {
    try {
      const stored = localStorage.getItem('esport-player-stats')
      if (stored) {
        const data = JSON.parse(stored)
        if (data.playerSeasonStats) {
          playerSeasonStats.value = new Map(Object.entries(data.playerSeasonStats))
          logger.debug('从 localStorage 恢复选手数据')
        }
      }
    } catch (error) {
      logger.error('从 storage 加载选手数据失败', { error })
    }
  }

  // 保存到 localStorage（同时触发异步保存到数据库）
  const saveToStorage = () => {
    try {
      // 同时触发批量保存到数据库（主要存储方式）
      flushPendingPerformances()

      // 尝试保存到 localStorage（快速本地备份，但可能因配额不足失败）
      try {
        const data = {
          playerSeasonStats: Object.fromEntries(playerSeasonStats.value),
          lastUpdated: new Date().toISOString()
        }
        localStorage.setItem('esport-player-stats', JSON.stringify(data))
      } catch (storageError: any) {
        // localStorage 配额不足时，清理旧数据
        if (storageError.name === 'QuotaExceededError') {
          logger.warn('localStorage 配额不足，清理旧数据')
          localStorage.removeItem('esport-player-stats')
          localStorage.removeItem('esport-match-details')
        }
      }
    } catch (error) {
      logger.error('保存选手数据失败', { error })
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
    getSeasonImpactRankingLocal,
    getPositionRanking,
    getPositionRankingLocal,
    generateMockPlayer,
    generateTeamRoster,
    generateAllPlayers,
    setPlayers,
    updatePlayerCondition,
    refreshAllConditions,
    clearSeasonStats,
    clearAll,
    loadFromStorage,
    saveToStorage,
    flushPendingPerformances
  }
})
