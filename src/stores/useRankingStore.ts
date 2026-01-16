import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useEventStore } from './useEventStore'
import { useTeamStore } from './useTeamStore'
import { useRegionStore } from './useRegionStore'
import { pointsApi } from '@/api'
import type {
  Team,
  Competition,
} from '@/types'

// 积分排名相关类型定义
export interface RegularSeasonStanding {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  matchesPlayed: number
  wins: number
  losses: number
  winRate: number
  regularSeasonPoints: number
  roundDifferential: number // 小场分差
  lastUpdated: string
  position: number
}

export interface RegionalStandings {
  regionId: string
  regionName: string
  seasonId: string
  competitionType: 'spring' | 'summer'
  standings: RegularSeasonStanding[]
  lastUpdated: string
}

export interface AnnualTeamRanking {
  teamId: string
  teamName: string
  regionId: string
  regionName: string
  totalPoints: number
  springPoints: number
  summerPoints: number
  playoffPoints: number
  msiPoints: number
  worldsPoints: number
  intercontinentalPoints: number // 洲际赛积分，仅用于荣誉展示，不计入总积分
  achievements: string[]
  position: number
  seasonId: string
}

export interface SeasonRankings {
  seasonId: string
  seasonYear: number
  annualRankings: AnnualTeamRanking[]
  regionalStandings: {
    spring: RegionalStandings[]
    summer: RegionalStandings[]
  }
  lastUpdated: string
}

// 积分计算规则
export interface PointsConfig {
  regular: {
    win: number // 常规赛胜利积分
    loss: number // 常规赛失败积分
  }
  playoffs: {
    champion: number // 赛区冠军
    runnerUp: number // 亚军
    semifinal: number // 半决赛
    quarterfinal: number // 四分之一决赛
  }
  international: {
    msi: {
      champion: number
      runnerUp: number
      semifinal: number
      groupStage: number
    }
    worlds: {
      champion: number
      runnerUp: number
      semifinal: number
      quarterfinal: number
      groupStage: number
    }
    // 洲际赛不参与年度积分计算，仅作为荣誉展示
    intercontinental: {
      champion: number // 仅用于荣誉展示，不计入年度积分
      runnerUp: number
      semifinals: number
      groupStage: number
    }
  }
}

export const useRankingStore = defineStore('ranking', () => {
  // 状态
  const regionalStandings = ref<Map<string, RegionalStandings>>(new Map())
  const seasonRankings = ref<Map<string, SeasonRankings>>(new Map())
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 依赖的其他store
  const eventStore = useEventStore()
  const teamStore = useTeamStore()
  const regionStore = useRegionStore()

  // 积分配置
  const pointsConfig = ref<PointsConfig>({
    regular: {
      win: 3,
      loss: 0
    },
    playoffs: {
      champion: 50,
      runnerUp: 35,
      semifinal: 25,
      quarterfinal: 15
    },
    international: {
      msi: {
        champion: 100,
        runnerUp: 80,
        semifinal: 60,
        groupStage: 20
      },
      worlds: {
        champion: 150,
        runnerUp: 120,
        semifinal: 90,
        quarterfinal: 60,
        groupStage: 30
      },
      intercontinental: {
        champion: 0, // 洲际赛不计入年度积分
        runnerUp: 0,
        semifinals: 0,
        groupStage: 0
      }
    }
  })

  // 计算属性
  const getCurrentSeasonRankings = computed(() => {
    const currentSeason = eventStore.currentSeason
    if (!currentSeason) return null
    return seasonRankings.value.get(String(currentSeason.id)) || null
  })

  const getRegionalStandingsByRegion = computed(() => {
    return (regionId: string, type: 'spring' | 'summer') => {
      const key = `${regionId}-${type}`
      return regionalStandings.value.get(key)
    }
  })

  // 获取赛区常规赛积分榜（第三阶段：从后端获取）
  const fetchRegionalStandings = async (
    regionId: string,
    seasonId: string,
    competitionType: 'spring' | 'summer'
  ) => {
    loading.value = true
    error.value = null

    try {
      console.log(`📊 获取积分榜: regionId=${regionId}, seasonId=${seasonId}, type=${competitionType}`)
      
      // 第三阶段：从后端获取积分榜数据
      const { rankingApi } = await import('@/api')
      const response = await rankingApi.getRegionalStandings(regionId, seasonId, competitionType)

      console.log('📊 后端返回数据:', response.data)

      if (response.data) {
        const standings: RegionalStandings = {
          regionId: response.data.regionId || regionId,
          regionName: response.data.regionName || '',
          seasonId,
          competitionType,
          standings: response.data.standings || [],
          lastUpdated: response.data.lastUpdated || new Date().toISOString()
        }

        const key = `${regionId}-${competitionType}`
        regionalStandings.value.set(key, standings)
        
        console.log(`✅ 积分榜已更新: key=${key}, 战队数=${standings.standings.length}`)
        console.log('战队列表:', standings.standings.map(s => s.teamName))

        return standings
      }

      // 如果后端返回空数据，回退到本地计算（兼容模式）
      await Promise.all([
        teamStore.fetchTeams(),
        regionStore.fetchRegions(),
        eventStore.fetchCompetitions(seasonId)
      ])

      const standings = await calculateRegionalStandings(regionId, seasonId, competitionType)
      const key = `${regionId}-${competitionType}`
      regionalStandings.value.set(key, standings)

      return standings
    } catch (err) {
      error.value = '获取赛区积分榜失败'
      console.error(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // 获取全年赛事积分排名
  const fetchSeasonRankings = async (seasonId: string) => {
    loading.value = true
    error.value = null

    try {
      // 获取赛季信息以获取年份
      const season = eventStore.seasons.find(s => s.id === seasonId)
      const seasonYear = season?.year || new Date().getFullYear()

      console.log(`🔍 获取赛季积分排名: seasonId=${seasonId}, year=${seasonYear}`)

      // 首先尝试使用新的积分系统API
      try {
        const response = await pointsApi.getSeasonPointsRanking(seasonYear)
        
        if (response.success && response.data.length > 0) {
          console.log('✅ 使用积分系统API获取数据成功', response.data.length)
          
          // 转换API数据为前端格式
          const annualRankings = response.data.map((item: any) => ({
            teamId: String(item.teamId),
            teamName: item.teamName,
            regionId: String(item.regionId),
            regionName: item.regionName,
            totalPoints: item.totalPoints,
            springPoints: item.springPoints,
            summerPoints: item.summerPoints,
            playoffPoints: item.playoffPoints,
            msiPoints: item.msiPoints,
            worldsPoints: item.worldsPoints,
            intercontinentalPoints: item.intercontinentalPoints || 0,
            position: parseInt(item.rank),
            achievements: [] // 可以后续添加成就数据
          }))

          const rankings: SeasonRankings = {
            seasonId,
            seasonYear,
            annualRankings,
            regionalStandings: {
              spring: [],
              summer: []
            },
            lastUpdated: new Date().toISOString()
          }

          seasonRankings.value.set(seasonId, rankings)
          console.log('📊 积分排名已更新到Store', annualRankings.length)
          
          return rankings
        }
      } catch (apiError) {
        console.warn('⚠️ 积分系统API调用失败，回退到本地计算', apiError)
      }

      // 如果API失败，回退到本地计算
      await Promise.all([
        teamStore.fetchTeams(),
        regionStore.fetchRegions(),
        eventStore.fetchCompetitions(seasonId)
      ])

      const rankings = await calculateSeasonRankings(seasonId)
      seasonRankings.value.set(seasonId, rankings)

      return rankings
    } catch (err) {
      error.value = '获取赛季积分排名失败'
      console.error(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // 计算赛区常规赛积分榜
  const calculateRegionalStandings = async (
    regionId: string,
    seasonId: string,
    competitionType: 'spring' | 'summer'
  ): Promise<RegionalStandings> => {
    const region = regionStore.regions.find(r => r.id === regionId)
    if (!region) throw new Error('赛区不存在')

    const regionTeams = teamStore.teams.filter(t => t.regionId === regionId)

    // 查找对应的常规赛
    const regularSeasonCompetition = eventStore.competitions.find(c =>
      c.seasonId === seasonId &&
      c.type === competitionType &&
      c.format === 'regular_season'
    )

    const standings: RegularSeasonStanding[] = []

    for (const team of regionTeams) {
      const teamStanding = await calculateTeamRegularSeasonStanding(
        team,
        regularSeasonCompetition
      )
      standings.push(teamStanding)
    }

    // 按积分排序，积分相同按胜场数排序，再相同按小场分差排序
    standings.sort((a, b) => {
      if (a.regularSeasonPoints !== b.regularSeasonPoints) {
        return b.regularSeasonPoints - a.regularSeasonPoints
      }
      if (a.wins !== b.wins) {
        return b.wins - a.wins
      }
      return b.roundDifferential - a.roundDifferential
    })

    // 设置排名
    standings.forEach((standing, index) => {
      standing.position = index + 1
    })

    return {
      regionId,
      regionName: region.name,
      seasonId,
      competitionType,
      standings,
      lastUpdated: new Date().toISOString()
    }
  }

  // 计算单个队伍的常规赛表现
  const calculateTeamRegularSeasonStanding = async (
    team: Team,
    competition?: Competition
  ): Promise<RegularSeasonStanding> => {
    let wins = 0
    let losses = 0
    let regularSeasonPoints = 0
    let roundDifferential = 0

    if (competition?.matches) {
      const teamMatches = competition.matches.filter(match =>
        match.homeTeamId === team.id || match.awayTeamId === team.id
      )

      for (const match of teamMatches) {
        if (match.result) {
          const isHome = match.homeTeamId === team.id
          const teamScore = isHome ? match.result.homeScore : match.result.awayScore
          const opponentScore = isHome ? match.result.awayScore : match.result.homeScore

          if (teamScore > opponentScore) {
            wins++
            regularSeasonPoints += pointsConfig.value.regular.win
          } else {
            losses++
            regularSeasonPoints += pointsConfig.value.regular.loss
          }

          roundDifferential += (teamScore - opponentScore)
        }
      }
    }

    const matchesPlayed = wins + losses
    const winRate = matchesPlayed > 0 ? (wins / matchesPlayed) * 100 : 0

    const region = regionStore.regions.find(r => r.id === team.regionId)

    return {
      teamId: team.id,
      teamName: team.name,
      regionId: team.regionId,
      regionName: region?.name || '',
      matchesPlayed,
      wins,
      losses,
      winRate: Math.round(winRate * 100) / 100,
      regularSeasonPoints,
      roundDifferential,
      lastUpdated: new Date().toISOString(),
      position: 0 // 将在排序后设置
    }
  }

  // 计算全年赛事积分排名
  const calculateSeasonRankings = async (seasonId: string): Promise<SeasonRankings> => {
    const season = eventStore.seasons.find(s => s.id === seasonId)
    if (!season) throw new Error('赛季不存在')

    const seasonCompetitions = eventStore.competitions.filter(c => c.seasonId === seasonId)
    const allTeams = teamStore.teams

    const annualRankings: AnnualTeamRanking[] = []

    for (const team of allTeams) {
      const ranking = await calculateTeamAnnualRanking(team, seasonCompetitions)
      annualRankings.push(ranking)
    }

    // 按总积分排序
    annualRankings.sort((a, b) => {
      if (a.totalPoints !== b.totalPoints) {
        return b.totalPoints - a.totalPoints
      }
      // 积分相同按成就数量排序
      return b.achievements.length - a.achievements.length
    })

    // 设置排名
    annualRankings.forEach((ranking, index) => {
      ranking.position = index + 1
    })

    // 计算春季和夏季常规赛积分榜
    const springStandings: RegionalStandings[] = []
    const summerStandings: RegionalStandings[] = []

    for (const region of regionStore.regions) {
      try {
        const springStanding = await calculateRegionalStandings(region.id, seasonId, 'spring')
        springStandings.push(springStanding)

        const summerStanding = await calculateRegionalStandings(region.id, seasonId, 'summer')
        summerStandings.push(summerStanding)
      } catch (error) {
        console.warn(`Failed to calculate standings for region ${region.id}:`, error)
      }
    }

    return {
      seasonId,
      seasonYear: season.year,
      annualRankings,
      regionalStandings: {
        spring: springStandings,
        summer: summerStandings
      },
      lastUpdated: new Date().toISOString()
    }
  }

  // 计算单个队伍的年度积分
  const calculateTeamAnnualRanking = async (
    team: Team,
    competitions: Competition[]
  ): Promise<AnnualTeamRanking> => {
    let springPoints = 0
    let summerPoints = 0
    let playoffPoints = 0
    let msiPoints = 0
    let worldsPoints = 0
    let intercontinentalPoints = 0
    const achievements: string[] = []

    for (const competition of competitions) {
      // 安全检查：确保 competition.teams 存在且是数组
      if (!competition.teams || !Array.isArray(competition.teams)) continue
      if (!competition.teams.some(t => t.id === team.id)) continue

      const teamPosition = getTeamPositionInCompetition(team, competition)

      switch (competition.type) {
        case 'spring':
          if (competition.format === 'regular_season') {
            springPoints += calculateRegularSeasonPoints(team, competition)
          } else if (competition.format === 'playoffs') {
            const points = calculatePlayoffPoints(teamPosition)
            playoffPoints += points
            if (teamPosition === 1) achievements.push(`${competition.name}冠军`)
            else if (teamPosition === 2) achievements.push(`${competition.name}亚军`)
          }
          break

        case 'summer':
          if (competition.format === 'regular_season') {
            summerPoints += calculateRegularSeasonPoints(team, competition)
          } else if (competition.format === 'playoffs') {
            const points = calculatePlayoffPoints(teamPosition)
            playoffPoints += points
            if (teamPosition === 1) achievements.push(`${competition.name}冠军`)
            else if (teamPosition === 2) achievements.push(`${competition.name}亚军`)
          }
          break

        case 'msi':
          msiPoints += calculateInternationalPoints(teamPosition, 'msi')
          if (teamPosition === 1) achievements.push('MSI冠军')
          else if (teamPosition === 2) achievements.push('MSI亚军')
          break

        case 'worlds':
          worldsPoints += calculateInternationalPoints(teamPosition, 'worlds')
          if (teamPosition === 1) achievements.push('世界冠军')
          else if (teamPosition === 2) achievements.push('世界亚军')
          break

        case 'intercontinental':
          // 洲际赛不计入年度积分，但记录成就和荣誉积分
          intercontinentalPoints += calculateInternationalPoints(teamPosition, 'intercontinental')
          if (teamPosition === 1) achievements.push('洲际冠军')
          // 注意：洲际赛积分不计入总积分
          break
      }
    }

    // 计算总积分：洲际赛不计入年度积分排名
    const totalPoints = springPoints + summerPoints + playoffPoints + msiPoints + worldsPoints
    const region = regionStore.regions.find(r => r.id === team.regionId)

    return {
      teamId: team.id,
      teamName: team.name,
      regionId: team.regionId,
      regionName: region?.name || '',
      totalPoints,
      springPoints,
      summerPoints,
      playoffPoints,
      msiPoints,
      worldsPoints,
      intercontinentalPoints,
      achievements,
      position: 0, // 将在排序后设置
      seasonId: String(competitions[0]?.seasonId || '')
    }
  }

  // 获取队伍在赛事中的排名（简化实现）
  const getTeamPositionInCompetition = (team: Team, competition: Competition): number => {
    // 简化实现：按战力排序来模拟最终排名
    const competitionTeams = competition.teams.slice().sort((a, b) => b.strength - a.strength)
    const position = competitionTeams.findIndex(t => t.id === team.id) + 1
    return position || competitionTeams.length
  }

  // 计算常规赛积分
  const calculateRegularSeasonPoints = (team: Team, competition: Competition): number => {
    // 简化实现：基于比赛结果计算
    if (!competition.matches) return 0

    let points = 0
    const teamMatches = competition.matches.filter(match =>
      match.homeTeamId === team.id || match.awayTeamId === team.id
    )

    for (const match of teamMatches) {
      if (match.result) {
        const isHome = match.homeTeamId === team.id
        const teamScore = isHome ? match.result.homeScore : match.result.awayScore
        const opponentScore = isHome ? match.result.awayScore : match.result.homeScore

        if (teamScore > opponentScore) {
          points += pointsConfig.value.regular.win
        } else {
          points += pointsConfig.value.regular.loss
        }
      }
    }

    return points
  }

  // 计算季后赛积分
  const calculatePlayoffPoints = (position: number): number => {
    const config = pointsConfig.value.playoffs

    switch (position) {
      case 1: return config.champion
      case 2: return config.runnerUp
      case 3:
      case 4: return config.semifinal
      case 5:
      case 6:
      case 7:
      case 8: return config.quarterfinal
      default: return 0
    }
  }

  // 计算国际赛事积分
  const calculateInternationalPoints = (
    position: number,
    competitionType: 'msi' | 'worlds' | 'intercontinental'
  ): number => {
    const config = pointsConfig.value.international[competitionType]

    switch (position) {
      case 1: return config.champion
      case 2: return config.runnerUp
      case 3:
      case 4:
        if (competitionType === 'intercontinental') {
          return (config as any).semifinals
        }
        return (config as any).semifinal
      case 5:
      case 6:
      case 7:
      case 8:
        if (competitionType === 'worlds') {
          return (config as any).quarterfinal
        }
        return config.groupStage
      default: return config.groupStage
    }
  }

  // 更新常规赛积分榜（在比赛结束后调用）（第三阶段：调用后端API）
  const updateRegionalStandings = async (
    regionId: string,
    seasonId: string,
    competitionType: 'spring' | 'summer'
  ) => {
    try {
      // 第三阶段：调用后端API更新积分榜
      const { rankingApi } = await import('@/api')
      const response = await rankingApi.updateRegionalStandings({
        regionId,
        seasonId,
        competitionType
      })

      if (response.data) {
        const standings: RegionalStandings = {
          regionId: response.data!.regionId,
          regionName: response.data!.regionName,
          seasonId: response.data!.seasonId,
          competitionType: response.data!.competitionType,
          standings: response.data!.standings.map(item => ({
            teamId: item.teamId,
            teamName: item.teamName,
            regionId: response.data!.regionId,
            regionName: response.data!.regionName,
            matchesPlayed: item.matchesPlayed,
            wins: item.wins,
            losses: item.losses,
            winRate: item.winRate,
            regularSeasonPoints: item.regularSeasonPoints,
            roundDifferential: item.roundDifferential,
            position: item.position,
            lastUpdated: response.data!.lastUpdated
          })),
          lastUpdated: response.data!.lastUpdated
        }

        const key = `${regionId}-${competitionType}`
        regionalStandings.value.set(key, standings)
        return standings
      }

      // 回退到本地计算
      const standings = await calculateRegionalStandings(regionId, seasonId, competitionType)
      const key = `${regionId}-${competitionType}`
      regionalStandings.value.set(key, standings)
      return standings
    } catch (error) {
      console.error('更新赛区积分榜失败:', error)
      throw error
    }
  }

  // 更新全年积分排名（在重要赛事结束后调用）
  const updateSeasonRankings = async (seasonId: string) => {
    try {
      const rankings = await calculateSeasonRankings(seasonId)
      seasonRankings.value.set(seasonId, rankings)
      return rankings
    } catch (error) {
      console.error('更新赛季积分排名失败:', error)
      throw error
    }
  }

  // 批量更新所有排名（在赛季结束或重要节点）
  const refreshAllRankings = async (seasonId: string) => {
    loading.value = true
    error.value = null

    try {
      // 更新所有赛区的春季和夏季积分榜
      const regionalPromises = regionStore.regions.flatMap(region => [
        updateRegionalStandings(region.id, seasonId, 'spring'),
        updateRegionalStandings(region.id, seasonId, 'summer')
      ])

      await Promise.all(regionalPromises)

      // 更新全年积分排名
      await updateSeasonRankings(seasonId)
    } catch (err) {
      error.value = '刷新排名数据失败'
      console.error(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    // 状态
    regionalStandings,
    seasonRankings,
    loading,
    error,
    pointsConfig,

    // 计算属性
    getCurrentSeasonRankings,
    getRegionalStandingsByRegion,

    // 方法
    fetchRegionalStandings,
    fetchSeasonRankings,
    updateRegionalStandings,
    updateSeasonRankings,
    refreshAllRankings
  }
})