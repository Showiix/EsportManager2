import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Match, MatchResult, Team, Competition } from '@/types'
import { useRankingStore } from './useRankingStore'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('ScheduleStore')

export interface TeamScore {
  teamId: string
  teamName: string
  regionId: string
  points: number
  matches: number
  wins: number
  losses: number
  winRate: number
}

export interface SimulationResult {
  matches: MatchResult[]
  updatedScoreboard: TeamScore[]
  roundComplete: boolean
}

export const useScheduleStore = defineStore('schedule', () => {
  // 依赖store
  const rankingStore = useRankingStore()

  // 状态
  const currentRound = ref<number>(1)
  const totalRounds = ref<number>(18) // 默认18轮常规赛
  const matches = ref<Match[]>([])
  const isSimulating = ref<boolean>(false)
  const scoreboards = ref<TeamScore[]>([])
  const currentCompetition = ref<Competition | null>(null)
  const loading = ref<boolean>(false)
  const teamRegionCache = ref<Map<string, string>>(new Map()) // 缓存队伍赛区映射

  // 计算属性
  const currentRoundMatches = computed(() => {
    return matches.value.filter(match => (match.roundNumber || match.round) === currentRound.value)
  })

  const regionSchedules = computed(() => {
    const regions = {
      'LPL': { id: 'lpl', name: 'LPL', matches: [] as Match[], scoreboard: [] as TeamScore[] },
      'LCK': { id: 'lck', name: 'LCK', matches: [] as Match[], scoreboard: [] as TeamScore[] },
      'LEC': { id: 'lec', name: 'LEC', matches: [] as Match[], scoreboard: [] as TeamScore[] },
      'LCS': { id: 'lcs', name: 'LCS', matches: [] as Match[], scoreboard: [] as TeamScore[] }
    }

    // 调试：查看前5场比赛的 regionId
    logger.debug('前5场比赛的regionId', matches.value.slice(0, 5).map(m => ({
      id: m.id,
      regionId: m.regionId,
      teamAId: m.teamAId,
      teamAName: m.teamAName
    })))

    matches.value.forEach(match => {
      const regionKey = (match.regionId?.toUpperCase() || 'LPL') as keyof typeof regions
      if (regions[regionKey]) {
        regions[regionKey].matches.push(match)
      }
    })

    // 调试：查看前5个积分榜的 regionId
    logger.debug('前5个积分榜的regionId', scoreboards.value.slice(0, 5).map(s => ({
      teamId: s.teamId,
      teamName: s.teamName,
      regionId: s.regionId
    })))

    scoreboards.value.forEach(score => {
      const regionKey = (score.regionId?.toUpperCase() || 'LPL') as keyof typeof regions
      if (regions[regionKey]) {
        regions[regionKey].scoreboard.push(score)
      }
    })

    Object.values(regions).forEach(region => {
      region.scoreboard.sort((a, b) => b.points - a.points || b.winRate - a.winRate)
    })

    // 调试：输出每个赛区的数量
    logger.debug('各赛区比赛和积分榜数量', {
      LPL: { matches: regions.LPL.matches.length, scoreboard: regions.LPL.scoreboard.length },
      LCK: { matches: regions.LCK.matches.length, scoreboard: regions.LCK.scoreboard.length },
      LEC: { matches: regions.LEC.matches.length, scoreboard: regions.LEC.scoreboard.length },
      LCS: { matches: regions.LCS.matches.length, scoreboard: regions.LCS.scoreboard.length }
    })

    return regions
  })

  const currentRoundRegionMatches = computed(() => {
    const regionMatches = {
      'LPL': [] as Match[],
      'LCK': [] as Match[],
      'LEC': [] as Match[],
      'LCS': [] as Match[]
    }

    regionSchedules.value.LPL.matches.filter(m => (m.roundNumber || m.round) === currentRound.value).forEach(m => regionMatches.LPL.push(m))
    regionSchedules.value.LCK.matches.filter(m => (m.roundNumber || m.round) === currentRound.value).forEach(m => regionMatches.LCK.push(m))
    regionSchedules.value.LEC.matches.filter(m => (m.roundNumber || m.round) === currentRound.value).forEach(m => regionMatches.LEC.push(m))
    regionSchedules.value.LCS.matches.filter(m => (m.roundNumber || m.round) === currentRound.value).forEach(m => regionMatches.LCS.push(m))

    return regionMatches
  })

  const isLastRound = computed(() => {
    return currentRound.value >= totalRounds.value
  })

  const canProceedToNextRound = computed(() => {
    return currentRoundMatches.value.every(match => match.status === 'completed')
  })

  const completedMatches = computed(() => {
    return matches.value.filter(match => match.status === 'completed').length
  })

  const totalMatches = computed(() => {
    return matches.value.length
  })

  const progress = computed(() => {
    if (totalMatches.value === 0) return 0
    return Math.round((completedMatches.value / totalMatches.value) * 100)
  })

  // 动作
  // 加载队伍数据并缓存 - 返回队伍赛区映射
  async function loadTeamRegionMapping(): Promise<Map<string, string>> {
    // 如果已经有缓存，直接返回
    if (teamRegionCache.value.size > 0) {
      logger.debug('使用缓存的队伍赛区映射')
      return teamRegionCache.value
    }

    try {
      const { teamApi } = await import('@/api')
      // 重要：加载所有队伍（limit=100确保获取全部40支队伍）
      const teamsResponse = await teamApi.getTeams({ limit: 100 })
      if (teamsResponse.data) {
        const teamsData = Array.isArray(teamsResponse.data) ? teamsResponse.data : []
        const mapping = new Map<string, string>()

        logger.debug('正在加载队伍赛区信息', { count: teamsData.length })

        // regionId 到赛区代码的映射
        const regionIdToCode: Record<number, string> = {
          1: 'LPL',  // 中国大陆赛区
          2: 'LCK',  // 韩国赛区
          3: 'LEC',  // 欧洲赛区
          4: 'LCS'   // 北美赛区
        }

        teamsData.forEach((team: any) => {
          const teamId = String(team.id)
          // 后端返回的是 regionId (数字 1/2/3/4)，需要转换为赛区代码
          const regionCode = team.region_code
            ? team.region_code.toUpperCase()
            : (regionIdToCode[team.regionId] || 'LPL')
          mapping.set(teamId, regionCode)
        })

        teamRegionCache.value = mapping
        logger.debug('队伍赛区映射加载完成', { count: mapping.size })

        // 按赛区统计队伍数量
        const regionCount: Record<string, number> = {}
        mapping.forEach((region) => {
          regionCount[region] = (regionCount[region] || 0) + 1
        })
        logger.debug('各赛区队伍数量', regionCount)

        return mapping
      }
    } catch (error) {
      logger.warn('无法加载队伍数据，将使用默认赛区', { error })
    }

    return new Map()
  }

  async function loadSchedule(competitionId: string): Promise<void> {
    loading.value = true
    try {
      // 清空现有数据
      matches.value = []
      scoreboards.value = []
      currentRound.value = 1
      totalRounds.value = 18

      // 第三阶段：调用后端API获取赛程数据
      const { competitionApi } = await import('@/api')

      // 获取赛事信息
      const competitionResponse = await competitionApi.getCompetition(competitionId)
      if (competitionResponse.data) {
        currentCompetition.value = competitionResponse.data as any
      }

      // 获取当前轮次信息
      try {
        const roundResponse = await competitionApi.getCurrentRound(competitionId)
        logger.debug('获取当前轮次响应', { data: roundResponse.data })

        if (roundResponse.data) {
          // 如果比赛已完成且当前轮次超过总轮次，则显示最后一轮
          const apiCurrentRound = roundResponse.data.currentRound
          const apiTotalRounds = roundResponse.data.totalRounds

          // 如果后端返回了 totalRounds，使用它；否则使用默认值 18
          totalRounds.value = apiTotalRounds !== undefined ? apiTotalRounds : 18
          logger.debug('设置 totalRounds', { value: totalRounds.value })

          // 如果比赛已完成，显示最后一轮；否则显示当前轮次
          if (apiTotalRounds !== undefined && apiCurrentRound > apiTotalRounds) {
            currentRound.value = apiTotalRounds
          } else {
            currentRound.value = apiCurrentRound
          }
          logger.debug('设置 currentRound', { value: currentRound.value })
        }
      } catch (error) {
        logger.warn('无法获取当前轮次信息，使用默认值', { error })
        currentRound.value = 1
        totalRounds.value = 18
      }

      // 获取比赛数据
      const matchesResponse = await competitionApi.getCompetitionMatches(competitionId)
      if (matchesResponse.data) {
        // 处理嵌套的响应格式: { data: { data: [...], meta: {...} } }
        let matchesData: any[] = []
        if (Array.isArray(matchesResponse.data)) {
          matchesData = matchesResponse.data
        } else if ((matchesResponse.data as any).data && Array.isArray((matchesResponse.data as any).data)) {
          matchesData = (matchesResponse.data as any).data
        }
        
        logger.debug('比赛数据加载完成', { count: matchesData.length })

        // 加载队伍赛区映射（使用缓存）
        const teamRegionMap = await loadTeamRegionMapping()

        // 为每场比赛添加 regionId
        matches.value = matchesData.map((match: any) => {
          const teamAId = String(match.teamAId || match.homeTeamId || '')
          const regionId = teamRegionMap.get(teamAId) || 'LPL'  // 默认为LPL

          return {
            ...match,
            regionId,
            // 确保teamAId和teamBId为字符串类型
            teamAId: String(match.teamAId),
            teamBId: String(match.teamBId)
          }
        })
        logger.debug('比赛加载完成', { count: matches.value.length })

        // 打印赛区分布用于调试
        const regionDistribution: Record<string, number> = {}
        matches.value.forEach(m => {
          const region = m.regionId || 'unknown'
          regionDistribution[region] = (regionDistribution[region] || 0) + 1
        })
        logger.debug('比赛赛区分布', regionDistribution)
      }

      // 更新积分榜
      await updateScoreboard()
    } catch (error) {
      handleError(error, {
        component: 'ScheduleStore',
        userAction: '加载赛程',
        silent: true
      })

      // 回退到模拟数据
      await mockLoadSchedule(competitionId)
    } finally {
      loading.value = false
    }
  }

  async function proceedToNextRound(): Promise<void> {
    if (!canProceedToNextRound.value || isLastRound.value) return

    currentRound.value += 1

    // 在进入下一轮时更新常规赛积分榜
    if (currentCompetition.value) {
      try {
        const competitionType = getCompetitionType(currentCompetition.value)
        if (competitionType && ['spring', 'summer'].includes(competitionType)) {
          // Region code到ID的映射
          const regionCodeToId: Record<string, string> = {
            'lpl': '1', 'lck': '2', 'lec': '3', 'lcs': '4'
          }
          
          // 更新所有赛区的积分榜
          const regionCodes = [...new Set(scoreboards.value.map(s => s.regionId))]

          for (const regionCode of regionCodes) {
            const regionId = regionCodeToId[regionCode.toLowerCase()]
            if (regionId) {
              await rankingStore.updateRegionalStandings(
                regionId,
                currentCompetition.value.seasonId.toString(),
                competitionType as 'spring' | 'summer'
              )
            }
          }
        }
      } catch (error) {
        logger.error('更新积分榜失败', { error })
      }
    }

    // TODO: 调用API更新当前轮次
    // await competitionApi.updateCurrentRound(currentCompetition.value?.id, currentRound.value)
  }

  async function simulateCurrentRound(): Promise<SimulationResult> {
    if (isSimulating.value) {
      throw new Error('Simulation already in progress')
    }

    isSimulating.value = true

    try {
      const simulationEngine = new MatchSimulationEngine()
      const scoreCalculator = new ScoreCalculator()

      const roundMatches = currentRoundMatches.value
      const results: MatchResult[] = []

      // 模拟每场比赛
      for (const match of roundMatches) {
        const homeTeamId = match.homeTeamId || match.teamAId?.toString() || ''
        const awayTeamId = match.awayTeamId || match.teamBId?.toString() || ''
        const homeTeam = await getTeamById(homeTeamId)
        const awayTeam = await getTeamById(awayTeamId)

        if (homeTeam && awayTeam) {
          const result = simulationEngine.calculateMatchResult(homeTeam, awayTeam)
          results.push({
            ...result,
            homeScore: result.teamAScore,
            awayScore: result.teamBScore,
            homePoints: result.winnerId === homeTeam.id ?
              (result.teamAScore === 2 && result.teamBScore === 0 ? 3 : 2) :
              (result.teamBScore === 1 ? 1 : 0),
            awayPoints: result.winnerId === awayTeam.id ?
              (result.teamBScore === 2 && result.teamAScore === 0 ? 3 : 2) :
              (result.teamAScore === 1 ? 1 : 0),
            winner: result.winnerId
          })

          // 更新比赛状态
          const matchIndex = matches.value.findIndex(m => m.id === match.id)
          if (matchIndex !== -1 && matches.value[matchIndex]) {
            matches.value[matchIndex].result = results[results.length - 1]
            matches.value[matchIndex].status = 'completed' as any
            matches.value[matchIndex].scoreA = result.teamAScore
            matches.value[matchIndex].scoreB = result.teamBScore
            matches.value[matchIndex].winnerId = result.winnerId
          }
        }
      }

      // 更新积分榜
      const updatedScoreboard = scoreCalculator.updateScoreboard(results, scoreboards.value, roundMatches)
      scoreboards.value = updatedScoreboard

      // 如果是常规赛，实时更新积分榜
      if (currentCompetition.value) {
        try {
          const competitionType = getCompetitionType(currentCompetition.value)
          if (competitionType && ['spring', 'summer'].includes(competitionType)) {
            // Region code到ID的映射
            const regionCodeToId: Record<string, string> = {
              'lpl': '1', 'lck': '2', 'lec': '3', 'lcs': '4'
            }
            
            // 更新所有赛区的积分榜
            const regionCodes = [...new Set(scoreboards.value.map(s => s.regionId))]

            for (const regionCode of regionCodes) {
              const regionId = regionCodeToId[regionCode.toLowerCase()]
              if (regionId) {
                await rankingStore.updateRegionalStandings(
                  regionId,
                  currentCompetition.value.seasonId.toString(),
                  competitionType as 'spring' | 'summer'
                )
              }
            }
          }
        } catch (error) {
          logger.error('模拟后更新积分榜失败', { error })
        }
      }

      return {
        matches: results,
        updatedScoreboard,
        roundComplete: roundMatches.every(m => m.status === 'completed')
      }
    } finally {
      isSimulating.value = false
    }
  }

  // 模拟当前轮次的所有赛区比赛
  async function simulateRegionCurrentRound(_regionKey?: string): Promise<SimulationResult> {
    if (isSimulating.value) {
      throw new Error('Simulation already in progress')
    }

    isSimulating.value = true

    try {
      // 调用后端API模拟整轮比赛
      if (currentCompetition.value?.id) {
        const { competitionApi } = await import('@/api')
        const response = await competitionApi.simulateRound(currentCompetition.value.id.toString())

        if (response.data) {
          logger.debug('后端模拟轮次返回数据', {
            matchesSimulated: response.data.matchesSimulated
          })

          // 更新所有赛区的比赛结果（不只是当前赛区）
          let updatedCount = 0
          response.data.results.forEach((result: any) => {
            const matchIndex = matches.value.findIndex(m => m.id === result.matchId)
            if (matchIndex !== -1 && matches.value[matchIndex]) {
              const existingMatch = matches.value[matchIndex]

              // 根据比分判断获胜者ID
              let winnerId = ''
              const homeScore = result.homeScore || 0
              const awayScore = result.awayScore || 0

              if (homeScore > awayScore) {
                winnerId = String(existingMatch.teamAId)
              } else if (awayScore > homeScore) {
                winnerId = String(existingMatch.teamBId)
              }

              // 更新比赛数据
              matches.value[matchIndex] = {
                ...existingMatch,
                status: 'completed' as any,
                scoreA: homeScore,
                scoreB: awayScore,
                winnerId: winnerId,
              }

              updatedCount++
            }
          })

          logger.debug('成功更新比赛结果', { count: updatedCount })

          // 更新积分榜
          await updateScoreboard()

          // 如果是常规赛，实时更新所有赛区的积分榜
          const competitionType = getCompetitionType(currentCompetition.value)
          if (competitionType && ['spring', 'summer'].includes(competitionType)) {
            try {
              // 获取所有已更新的赛区
              const updatedRegions = new Set<string>()
              response.data.results.forEach((result: any) => {
                const match = matches.value.find(m => m.id === result.matchId)
                if (match && match.regionId) {
                  updatedRegions.add(match.regionId.toLowerCase())
                }
              })

              // Region code到ID的映射
              const regionCodeToId: Record<string, string> = {
                'lpl': '1',
                'lck': '2',
                'lec': '3',
                'lcs': '4'
              }

              // 更新所有涉及的赛区的积分榜
              for (const regionCode of updatedRegions) {
                try {
                  const regionId = regionCodeToId[regionCode.toLowerCase()]
                  if (!regionId) {
                    logger.warn('未知的赛区代码', { regionCode })
                    continue
                  }

                  await rankingStore.updateRegionalStandings(
                    regionId, // 传递数字ID
                    String(currentCompetition.value.seasonId),
                    competitionType as 'spring' | 'summer'
                  )
                  logger.debug('赛区积分榜更新完成', { region: regionCode.toUpperCase() })
                } catch (error) {
                  logger.warn('赛区积分榜更新失败', { region: regionCode.toUpperCase(), error })
                  // 继续更新其他赛区
                }
              }
            } catch (error) {
              logger.warn('更新积分榜失败', { error })
            }
          }

          logger.info('轮次模拟完成', { round: currentRound.value })

          return {
            matches: response.data.results.map((r: any) => ({
              teamAScore: r.homeScore || 0,
              teamBScore: r.awayScore || 0,
              winnerId: r.winner || '',
              homeScore: r.homeScore || 0,
              awayScore: r.awayScore || 0,
              homePoints: 0, // 这些由 updateScoreboard 计算
              awayPoints: 0,
              winner: r.winner || ''
            })),
            updatedScoreboard: scoreboards.value,
            roundComplete: response.data.isRoundComplete
          }
        }
      }

      // 如果没有后端数据，抛出错误
      throw new Error('无法连接到后端服务')
    } catch (error) {
      handleError(error, {
        component: 'ScheduleStore',
        userAction: '模拟轮次'
      })
      throw error
    } finally {
      isSimulating.value = false
    }
  }

  // 模拟单场比赛（已禁用）
  /*
  async function simulateSingleMatch(matchId: string): Promise<void> {
    if (isSimulating.value) {
      throw new Error('Simulation already in progress')
    }

    isSimulating.value = true

    try {
      // 调用后端API模拟比赛
      const { matchApi } = await import('@/api')
      const response = await matchApi.simulateMatch(matchId)

      if (response.data) {
        console.log(`🎮 单场比赛模拟返回数据:`, response.data)

        // 更新本地比赛数据
        const matchIndex = matches.value.findIndex(m => m.id === matchId)
        if (matchIndex !== -1) {
          const existingMatch = matches.value[matchIndex]

          // 根据比分判断获胜者ID（和 simulateRegionCurrentRound 一样的逻辑）
          let winnerId = ''
          const homeScore = response.data.scoreA || 0
          const awayScore = response.data.scoreB || 0

          if (homeScore > awayScore) {
            winnerId = String(existingMatch.teamAId)
          } else if (awayScore > homeScore) {
            winnerId = String(existingMatch.teamBId)
          }

          // 更新比赛数据，保留前端添加的字段
          matches.value[matchIndex] = {
            ...existingMatch,
            status: 'completed' as any,
            scoreA: homeScore,
            scoreB: awayScore,
            winnerId: winnerId,
          }

          console.log(`✅ 单场比赛更新后:`, {
            id: matches.value[matchIndex].id,
            scoreA: matches.value[matchIndex].scoreA,
            scoreB: matches.value[matchIndex].scoreB,
            winnerId: matches.value[matchIndex].winnerId
          })
        }

        // 更新积分榜
        await updateScoreboard()

        console.log(`✅ 单场比赛模拟完成: ${matchId}, 比分 ${homeScore}:${awayScore}`)
      }
    } finally {
      isSimulating.value = false
    }
  }
  */

  // 单场模拟功能已禁用，始终抛出错误
  async function simulateSingleMatch(_matchId: string): Promise<void> {
    throw new Error('单场模拟功能已禁用，请使用"模拟第X轮（所有赛区）"按钮')
  }

  async function updateScoreboard(): Promise<void> {
    // 使用缓存的队伍赛区映射
    const teamRegionMap = await loadTeamRegionMapping()

    // 从比赛数据中提取队伍信息并初始化积分榜
    const teamsMap = new Map<string, TeamScore>()

    // 从比赛数据中提取所有队伍
    matches.value.forEach(match => {
      const teamAId = String(match.teamAId || match.homeTeamId || '')
      const teamBId = String(match.teamBId || match.awayTeamId || '')

      // 初始化 teamA
      if (teamAId && !teamsMap.has(teamAId)) {
        const regionId = teamRegionMap.get(teamAId) || 'LPL'
        teamsMap.set(teamAId, {
          teamId: teamAId,
          // 使用简写名称 (teamAShort) 而不是完整名称
          teamName: (match as any).teamAShort || match.teamAName || `Team ${teamAId}`,
          regionId: regionId,
          points: 0,
          matches: 0,
          wins: 0,
          losses: 0,
          winRate: 0
        })
      }

      // 初始化 teamB
      if (teamBId && !teamsMap.has(teamBId)) {
        const regionId = teamRegionMap.get(teamBId) || 'LPL'
        teamsMap.set(teamBId, {
          teamId: teamBId,
          // 使用简写名称 (teamBShort) 而不是完整名称
          teamName: (match as any).teamBShort || match.teamBName || `Team ${teamBId}`,
          regionId: regionId,
          points: 0,
          matches: 0,
          wins: 0,
          losses: 0,
          winRate: 0
        })
      }
    })

    // 计算积分 - 只处理已完成的比赛，使用 Map 提升性能
    const completedMatches = matches.value.filter(m => m.status === 'completed')

    for (const match of completedMatches) {
      const teamAId = String(match.teamAId || match.homeTeamId || '')
      const teamBId = String(match.teamBId || match.awayTeamId || '')
      const scoreA = match.scoreA || 0
      const scoreB = match.scoreB || 0
      const winnerId = String(match.winnerId || '')

      const teamA = teamsMap.get(teamAId)
      const teamB = teamsMap.get(teamBId)

      if (teamA && teamB && teamAId && teamBId) {
        // 更新比赛场次
        teamA.matches += 1
        teamB.matches += 1

        // 更新胜负
        if (winnerId === teamAId) {
          teamA.wins += 1
          teamB.losses += 1
        } else if (winnerId === teamBId) {
          teamB.wins += 1
          teamA.losses += 1
        }

        // 更新积分（根据BO3规则：2-0得3分，2-1得2分，1-2得1分，0-2得0分）
        if (scoreA === 2 && scoreB === 0) {
          teamA.points += 3
        } else if (scoreA === 2 && scoreB === 1) {
          teamA.points += 2
          teamB.points += 1
        } else if (scoreA === 1 && scoreB === 2) {
          teamA.points += 1
          teamB.points += 2
        } else if (scoreA === 0 && scoreB === 2) {
          teamB.points += 3
        }

        // 更新胜率
        teamA.winRate = teamA.matches > 0 ? teamA.wins / teamA.matches : 0
        teamB.winRate = teamB.matches > 0 ? teamB.wins / teamB.matches : 0
      }
    }

    // 转换为数组并按积分排序
    scoreboards.value = Array.from(teamsMap.values()).sort(
      (a, b) => b.points - a.points || b.winRate - a.winRate
    )

    logger.debug('积分榜更新完成', { teamsCount: scoreboards.value.length })
  }

  function resetSchedule(): void {
    currentRound.value = 1
    matches.value = []
    scoreboards.value = []
    currentCompetition.value = null
    isSimulating.value = false
  }

  // 辅助函数
  function getCompetitionType(competition: Competition): string | null {
    return competition.type || null
  }

  // 季后赛或国际赛事结束时更新年度积分排名
  async function updateAnnualRankingsOnCompetitionEnd(competition: Competition): Promise<void> {
    try {
      // 如果是季后赛、MSI、世界赛更新年度积分排名
      // 注意：洲际赛不计入年度积分，所以不需要触发年度积分更新
      if (['spring', 'summer', 'msi', 'worlds'].includes(competition.type)) {
        await rankingStore.updateSeasonRankings(competition.seasonId.toString())
      }

      // 洲际赛结束时只更新荣誉殿堂，不更新年度积分排名
      if (competition.type === 'intercontinental') {
        logger.debug('洲际赛结束，更新荣誉记录但不影响年度积分排名')
      }
    } catch (error) {
      logger.error('更新年度积分排名失败', { error })
    }
  }

  // 手动触发积分榜更新
  async function refreshAllRankings(): Promise<void> {
    if (!currentCompetition.value) return

    try {
      await rankingStore.refreshAllRankings(currentCompetition.value.seasonId.toString())
    } catch (error) {
      logger.error('刷新所有排名失败', { error })
      throw error
    }
  }

  // 完成常规赛并生成季后赛
  async function completeRegularSeason(regionId?: string): Promise<void> {
    if (!currentCompetition.value) {
      throw new Error('当前没有赛事')
    }

    try {
      logger.info('开始完成常规赛流程')

      // 1. 调用完成赛事API
      const { competitionApi, playoffApi } = await import('@/api')

      logger.debug('调用完成赛事API', { competitionId: currentCompetition.value.id })
      await competitionApi.finishCompetition(String(currentCompetition.value.id))
      logger.debug('赛事状态已更新为完成')

      // 2. 如果指定了赛区，只为该赛区生成季后赛；否则为所有赛区生成季后赛
      const regions = regionId ? [regionId.toUpperCase()] : ['LPL', 'LCK', 'LEC', 'LCS']

      for (const region of regions) {
        try {
          logger.debug('检查赛区是否可以生成季后赛', { region })

          // 检查是否可以生成季后赛
          const eligibility = await playoffApi.checkPlayoffEligibility(
            String(currentCompetition.value.id),
            region
          )

          if (eligibility.data?.eligible) {
            logger.debug('赛区满足生成季后赛条件，开始生成', { region })

            // 生成季后赛
            const playoffResult = await playoffApi.generatePlayoff({
              competitionId: String(currentCompetition.value.id),
              seasonId: String(currentCompetition.value.seasonId),
              regionId: region,
              competitionType: currentCompetition.value.type as 'spring' | 'summer'
            })

            if (playoffResult.data) {
              logger.info('赛区季后赛生成成功', { region })
            }
          } else {
            logger.warn('赛区不满足生成季后赛条件', { region, reason: eligibility.data?.reason })
          }
        } catch (error) {
          logger.error('赛区季后赛生成失败', { region, error })
          // 继续处理其他赛区
        }
      }

      logger.info('常规赛完成流程执行完毕')
    } catch (error) {
      handleError(error, {
        component: 'ScheduleStore',
        userAction: '完成常规赛'
      })
      throw error
    }
  }

  async function getTeamById(teamId: string): Promise<Team | null> {
    const teams = await getAllTeams()
    return teams.find(team => team.id === teamId) || {
      id: teamId,
      name: `Team ${teamId}`,
      regionId: 'lpl',
      strength: Math.floor(Math.random() * 100)
    }
  }

  async function getAllTeams(): Promise<Team[]> {
    const regions = ['lpl', 'lck', 'lec', 'lcs']
    const teams: Team[] = []

    for (let i = 0; i < 40; i++) {
      teams.push({
        id: `team-${i}`,
        name: `${regions[i % 4].toUpperCase()} Team ${Math.floor(i / 4) + 1}`,
        regionId: regions[i % 4],
        strength: Math.floor(Math.random() * 100)
      })
    }

    return teams
  }

  async function mockLoadSchedule(competitionId: string): Promise<void> {
    const regions = ['lpl', 'lck', 'lec', 'lcs']
    const mockMatches: Match[] = []

    for (let round = 1; round <= 18; round++) {
      for (let i = 0; i < 20; i++) {
        mockMatches.push({
          id: `match-${round}-${i}`,
          competitionId,
          homeTeamId: `team-${i * 2}`,
          awayTeamId: `team-${i * 2 + 1}`,
          round,
          roundNumber: round,  // 添加 roundNumber 字段
          stage: 'regular_season',
          regionId: regions[Math.floor(i / 5) % 4],
          status: round < currentRound.value ? 'completed' as any : 'scheduled' as any
        })
      }
    }

    matches.value = mockMatches

    const mockScoreboard: TeamScore[] = []
    for (let i = 0; i < 40; i++) {
      mockScoreboard.push({
        teamId: `team-${i}`,
        teamName: `${regions[i % 4].toUpperCase()} Team ${Math.floor(i / 4) + 1}`,
        regionId: regions[i % 4],
        points: Math.floor(Math.random() * 30),
        matches: currentRound.value - 1,
        wins: Math.floor(Math.random() * (currentRound.value - 1)),
        losses: 0,
        winRate: 0
      })
    }

    scoreboards.value = mockScoreboard.sort((a, b) => b.points - a.points)
  }

  return {
    // 状态
    currentRound,
    totalRounds,
    matches,
    isSimulating,
    scoreboards,
    currentCompetition,
    loading,

    // 计算属性
    currentRoundMatches,
    regionSchedules,
    currentRoundRegionMatches,
    isLastRound,
    canProceedToNextRound,
    completedMatches,
    totalMatches,
    progress,

    // 动作
    loadSchedule,
    proceedToNextRound,
    simulateCurrentRound,
    simulateRegionCurrentRound,
    simulateSingleMatch,
    updateScoreboard,
    resetSchedule,
    updateAnnualRankingsOnCompetitionEnd,
    refreshAllRankings,
    completeRegularSeason
  }
})

// 模拟计算引擎
class MatchSimulationEngine {
  calculateMatchResult(teamA: Team, teamB: Team): { teamAScore: number, teamBScore: number, winnerId: string } {
    const powerDiff = teamA.strength - teamB.strength
    const randomFactor = (Math.random() - 0.5) * 20
    const adjustedDiff = powerDiff + randomFactor

    const winProbability = this.calculateWinProbability(adjustedDiff)
    const isTeamAWin = Math.random() < winProbability

    const score = this.generateScore(Math.abs(adjustedDiff))

    return {
      teamAScore: isTeamAWin ? score.winner : score.loser,
      teamBScore: isTeamAWin ? score.loser : score.winner,
      winnerId: isTeamAWin ? teamA.id : teamB.id
    }
  }

  private calculateWinProbability(powerDiff: number): number {
    return 1 / (1 + Math.exp(-powerDiff / 10))
  }

  private generateScore(powerGap: number): { winner: number, loser: number } {
    if (powerGap > 30) {
      return { winner: 2, loser: 0 }
    } else if (powerGap > 15) {
      return Math.random() > 0.7 ? { winner: 2, loser: 0 } : { winner: 2, loser: 1 }
    } else {
      const rand = Math.random()
      if (rand > 0.6) return { winner: 2, loser: 1 }
      else if (rand > 0.3) return { winner: 2, loser: 0 }
      else return { winner: 2, loser: 1 }
    }
  }
}

// 积分计算器
class ScoreCalculator {
  updateScoreboard(results: MatchResult[], currentScoreboard: TeamScore[], matches: Match[]): TeamScore[] {
    const scoreMap = new Map(currentScoreboard.map(team => [team.teamId, { ...team }]))

    results.forEach((result, index) => {
      const match = matches[index]
      if (!match) return

      const homeTeamId = match.homeTeamId || match.teamAId?.toString() || ''
      const awayTeamId = match.awayTeamId || match.teamBId?.toString() || ''
      const homeTeam = scoreMap.get(homeTeamId)
      const awayTeam = scoreMap.get(awayTeamId)

      if (homeTeam && awayTeam && homeTeamId && awayTeamId) {
        homeTeam.points += result.homePoints
        homeTeam.matches += 1
        if (result.winner === homeTeamId) {
          homeTeam.wins += 1
        } else {
          homeTeam.losses += 1
        }
        homeTeam.winRate = homeTeam.matches > 0 ? homeTeam.wins / homeTeam.matches : 0

        awayTeam.points += result.awayPoints
        awayTeam.matches += 1
        if (result.winner === awayTeamId) {
          awayTeam.wins += 1
        } else {
          awayTeam.losses += 1
        }
        awayTeam.winRate = awayTeam.matches > 0 ? awayTeam.wins / awayTeam.matches : 0
      }
    })

    return Array.from(scoreMap.values()).sort((a, b) => b.points - a.points)
  }
}