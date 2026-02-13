import { ref, computed } from 'vue'
import { teamApi, playerApi, honorApi, statsApi, formatHonorType, type TraitInfo, type PlayerConditionInfo, type MarketValueChange, type PlayerContractRecord, type PlayerTournamentHistoryItem } from '@/api/tauri'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'
import { useTimeStore } from '@/stores/useTimeStore'
import { createLogger } from '@/utils/logger'

const logger = createLogger('usePlayerDetail')

// 选手赛季统计接口
export interface PlayerSeasonStats {
  avg_impact: number
  avg_performance: number
  best_performance: number
  worst_performance: number
  consistency_score: number
  champion_bonus: number
  games_played: number
  international_titles: number
  regional_titles: number
}

// 选手基础信息接口
export interface PlayerDetailInfo {
  id: string
  gameId: string
  realName: string
  nationality: string
  team: string
  region: string
  position: string
  age: number
  ability: number
  potential: number
  stability: number
  loyalty: number
  satisfaction: number
  tag: string
  salary: number
  marketValue: number
  calculatedMarketValue: number
  contractEnd: string
  joinSeason: string
}

// 荣誉接口
export interface PlayerHonor {
  season: string
  tournament: string
  position: string
}

// 赛季历史接口
export interface PlayerHistorySeason {
  season: string
  team_name?: string
  team?: string
  ability: number
  potential: number
}

// 位置简称映射
const positionShortMap: Record<string, string> = {
  'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP'
}

// 国际赛事类型
const INTL_TYPES = new Set([
  'Msi', 'MadridMasters', 'ClaudeIntercontinental', 'WorldChampionship',
  'ShanghaiMasters', 'IcpIntercontinental', 'SuperIntercontinental'
])

export function usePlayerDetail(playerId: string) {
  const teamStore = useTeamStoreTauri()
  const timeStore = useTimeStore()

  // 状态定义
  const player = ref<PlayerDetailInfo>({
    id: playerId,
    gameId: '加载中...',
    realName: '加载中...',
    nationality: '未知',
    team: '未知',
    region: 'LPL',
    position: 'MID',
    age: 20,
    ability: 70,
    potential: 80,
    stability: 75,
    loyalty: 50,
    satisfaction: 50,
    tag: 'NORMAL',
    salary: 500000,
    marketValue: 3000000,
    calculatedMarketValue: 0,
    contractEnd: 'S2',
    joinSeason: 'S1',
  })

  const honors = ref<PlayerHonor[]>([])
  const contractHistory = ref<PlayerContractRecord[]>([])
  const traits = ref<TraitInfo[]>([])
  const conditionInfo = ref<PlayerConditionInfo | null>(null)
  const playerStats = ref<PlayerSeasonStats | null>(null)
  const tournamentHistory = ref<PlayerTournamentHistoryItem[]>([])
  const marketValueChanges = ref<MarketValueChange[]>([])
  const seasonHistory = ref<PlayerHistorySeason[]>([])
  const allTraits = ref<any[]>([
    { type: 'clutch', name: '大赛型', description: '在季后赛和国际赛中状态更好', rarity: 4, isNegative: false, icon: '🎯' },
    { type: 'slowstarter', name: '慢热型', description: '系列赛开局较慢，但后期渐入佳境', rarity: 2, isNegative: false, icon: '🐢' },
    { type: 'faststarter', name: '快枪手', description: '系列赛开局强势，但后期可能疲软', rarity: 2, isNegative: false, icon: '⚡' },
    { type: 'explosive', name: '爆发型', description: '发挥波动大，但巅峰更高', rarity: 3, isNegative: false, icon: '💥' },
    { type: 'consistent', name: '稳定型', description: '发挥稳定，但上限略低', rarity: 2, isNegative: false, icon: '🛡️' },
    { type: 'comebackking', name: '逆风王', description: '落后时愈战愈勇', rarity: 4, isNegative: false, icon: '👑' },
    { type: 'tilter', name: '顺风浪', description: '心态容易受比分影响', rarity: 1, isNegative: true, icon: '😰' },
    { type: 'mentalfortress', name: '心态大师', description: '心态稳定，不受连胜连败影响', rarity: 4, isNegative: false, icon: '🧠' },
    { type: 'fragile', name: '玻璃心', description: '输了比赛心态下滑更快', rarity: 1, isNegative: true, icon: '💔' },
    { type: 'ironman', name: '铁人', description: '不受连续比赛疲劳影响', rarity: 3, isNegative: false, icon: '💪' },
    { type: 'volatile', name: '状态敏感', description: '状态波动比常人更大', rarity: 2, isNegative: true, icon: '🎲' },
    { type: 'risingstar', name: '新星', description: '新人赛季潜力爆发', rarity: 3, isNegative: false, icon: '⭐' },
    { type: 'veteran', name: '老将风范', description: '老将经验丰富，发挥更稳', rarity: 3, isNegative: false, icon: '🎖️' },
    { type: 'teamleader', name: '团队核心', description: '带动队友发挥', rarity: 5, isNegative: false, icon: '🏅' },
  ])

  // 计算属性
  const careerYears = computed(() => {
    const joinMatch = player.value.joinSeason.match(/S(\d+)/)
    if (joinMatch) {
      const joinYear = parseInt(joinMatch[1])
      const currentYear = timeStore.currentSeason || 1
      return Math.max(1, currentYear - joinYear + 1)
    }
    return 1
  })

  const championCount = computed(() => {
    return honors.value.filter(h => h.position === '冠军').length
  })

  const displayMarketValue = computed(() => {
    return player.value.calculatedMarketValue > 0
      ? player.value.calculatedMarketValue
      : player.value.marketValue
  })

  const bigStageScore = computed(() => {
    const history = tournamentHistory.value
    if (!history || history.length === 0) return 0
    let sum = 0, games = 0
    for (const t of history) {
      if (INTL_TYPES.has(t.tournament_type)) {
        sum += t.avg_impact * t.games_played
        games += t.games_played
      }
    }
    if (games <= 0) return 0
    const rawScore = sum / games
    const confidence = Math.min(1.0, games / 70)
    return rawScore * confidence
  })

  const hasInternational = computed(() => {
    return tournamentHistory.value.some(t => INTL_TYPES.has(t.tournament_type))
  })

  const computeRadarData = computed(() => {
    if (!playerStats.value) return null

    const stats = playerStats.value

    const impactScore = Math.min(100, Math.max(0, (stats.avg_impact + 5) * 5))
    const performanceScore = Math.min(100, Math.max(0, (stats.avg_performance - 50) * 2))
    const consistencyScore = stats.consistency_score
    const honorScore = Math.min(100, stats.champion_bonus * 6.67)
    const gamesScore = Math.min(100, Math.max(0, stats.games_played * 0.83))
    const bigStageNorm = hasInternational.value
      ? Math.min(100, Math.max(0, (bigStageScore.value + 5) * 5))
      : 0

    return {
      impact: Math.round(impactScore),
      performance: Math.round(performanceScore),
      bigStage: Math.round(bigStageNorm),
      consistency: Math.round(consistencyScore),
      games: Math.round(gamesScore),
      honor: Math.round(honorScore)
    }
  })

  // 辅助函数
  const getRegionNationality = (regionCode: string) => {
    const nationalities: Record<string, string> = {
      'LPL': '中国',
      'LCK': '韩国',
      'LEC': '欧洲',
      'LCS': '北美',
    }
    return nationalities[regionCode] || '未知'
  }

  // 加载数据方法
  const loadPlayer = async () => {
    try {
      const numericId = parseInt(playerId)

      if (!isNaN(numericId)) {
        const foundPlayer = await playerApi.getPlayer(numericId)

        if (foundPlayer) {
          await teamStore.loadRegions()

          let teamName = '未知'
          let regionCode = 'LPL'

          if (foundPlayer.team_id) {
            try {
              const team = await teamApi.getTeam(foundPlayer.team_id)
              teamName = team.name
              const region = teamStore.regions.find(r => r.id === team.region_id)
              regionCode = region?.code || 'LPL'
            } catch (e) {
              logger.error('Failed to get team info:', e)
            }
          }

          const tag = foundPlayer.potential >= 90 || foundPlayer.ability >= 85 ? 'GENIUS'
            : foundPlayer.potential >= 75 || foundPlayer.ability >= 70 ? 'NORMAL'
            : 'ORDINARY'

          const marketValue = foundPlayer.ability * 100000 + foundPlayer.potential * 50000
          const salary = Math.round(marketValue * 0.15)
          const position = positionShortMap[foundPlayer.position || ''] || foundPlayer.position || 'MID'

          player.value = {
            id: playerId,
            gameId: foundPlayer.game_id,
            realName: foundPlayer.real_name || foundPlayer.game_id,
            nationality: getRegionNationality(regionCode),
            team: teamName,
            region: regionCode,
            position: position,
            age: foundPlayer.age,
            ability: foundPlayer.ability,
            potential: foundPlayer.potential,
            stability: foundPlayer.stability || Math.round(70 + (30 - foundPlayer.age) * 0.5 + Math.random() * 10),
            loyalty: foundPlayer.loyalty ?? 50,
            satisfaction: foundPlayer.satisfaction ?? 50,
            tag: tag,
            salary: foundPlayer.salary || salary,
            marketValue: foundPlayer.market_value || marketValue,
            calculatedMarketValue: foundPlayer.calculated_market_value || 0,
            contractEnd: foundPlayer.contract_end_season ? `S${foundPlayer.contract_end_season}` : 'S3',
            joinSeason: foundPlayer.join_season ? `S${foundPlayer.join_season}` : 'S1',
          }

          // Initial fallback for season history if not loaded yet
          if (seasonHistory.value.length === 0) {
            seasonHistory.value = [{
              season: 'S1',
              team_name: teamName,
              ability: foundPlayer.ability,
              potential: foundPlayer.potential
            }]
          }
        }
      }
    } catch (error) {
      logger.error('Failed to load player:', error)
    }
  }

  const loadHonors = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        const playerHonors = await honorApi.getPlayerHonors(numericId)
        honors.value = playerHonors.map(h => ({
          season: `S${h.season_id}`,
          tournament: h.tournament_name,
          position: formatHonorType(h.honor_type)
        }))
      }
    } catch (e) {
      logger.error('Failed to load player honors:', e)
      honors.value = []
    }
  }

  const loadContractHistory = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        contractHistory.value = await statsApi.getPlayerContractHistory(numericId)
      }
    } catch (e) {
      logger.error('Failed to load contract history:', e)
      contractHistory.value = []
    }
  }

  const loadTraitsAndCondition = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        const [traitsData, conditionData] = await Promise.all([
          playerApi.getPlayerTraits(numericId),
          playerApi.getPlayerCondition(numericId)
        ])
        traits.value = traitsData || []
        conditionInfo.value = conditionData
      }
    } catch (e) {
      logger.error('Failed to load traits/condition:', e)
      traits.value = []
      conditionInfo.value = null
    }
  }

  const loadStats = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        const statsResult = await statsApi.getPlayerStats(numericId)
        if (statsResult && statsResult.length > 0) {
          playerStats.value = statsResult[statsResult.length - 1]
        }
      }
    } catch (e) {
      logger.error('Failed to load player stats:', e)
      playerStats.value = null
    }
  }

  const loadTournamentHistory = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        const currentSeason = timeStore.currentSeason || 1
        const history = await statsApi.getPlayerTournamentHistory(numericId, currentSeason)
        tournamentHistory.value = history || []
      }
    } catch (e) {
      logger.error('Failed to load tournament history:', e)
      tournamentHistory.value = []
    }
  }

  const loadSeasonHistory = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        const history = await statsApi.getPlayerSeasonHistory(numericId)
        if (history && history.length > 0) {
          seasonHistory.value = history
        }
      }
    } catch (e) {
      logger.error('Failed to load season history:', e)
    }
  }

  const loadMarketValueChanges = async () => {
    try {
      const numericId = parseInt(playerId)
      if (!isNaN(numericId)) {
        const result = await statsApi.getPlayerMarketValueChanges(numericId)
        if (result) {
          marketValueChanges.value = result
        }
      }
    } catch (e) {
      logger.error('Failed to load market value changes:', e)
    }
  }

  const initData = async () => {
    await Promise.all([
      loadPlayer(),
      loadHonors(),
      loadContractHistory(),
      loadTraitsAndCondition(),
      loadStats(),
      loadTournamentHistory(),
      loadSeasonHistory()
    ])
  }

  return {
    player,
    honors,
    contractHistory,
    traits,
    conditionInfo,
    playerStats,
    tournamentHistory,
    marketValueChanges,
    seasonHistory,
    allTraits,
    
    careerYears,
    championCount,
    displayMarketValue,
    computeRadarData,
    hasInternational,
    
    loadPlayer,
    loadHonors,
    loadContractHistory,
    loadTraitsAndCondition,
    loadStats,
    loadTournamentHistory,
    loadSeasonHistory,
    loadMarketValueChanges,
    initData
  }
}
