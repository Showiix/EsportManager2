import { ref } from 'vue'
import { defineStore } from 'pinia'
import { worldsApi } from '@/api'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('WorldsStore')

interface WorldsData {
  id?: number
  season: string
  status: 'NOT_STARTED' | 'PLAY_IN_DRAW' | 'GROUP_STAGE' | 'KNOCKOUT' | 'COMPLETED'
  teams?: any[]
  createdAt?: string
  champion?: any
  runnerUp?: any
  thirdPlace?: any
  fourthPlace?: any
  quarterFinalists?: any[]
  groupStageTeams?: any[]
  pointsDistribution?: {
    champion: number
    runnerUp: number
    thirdPlace: number
    fourthPlace: number
    quarterFinalist: number
    groupStage: number
  }
}

export const useWorldsStore = defineStore('worlds', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 多赛季世界赛数据存储（season_code -> WorldsData）
  const worldsBrackets = ref<Map<string, WorldsData>>(new Map())

  // 当前世界赛数据
  const currentWorlds = ref<WorldsData>({
    season: 'S1',
    status: 'NOT_STARTED'
  })

  // 入围赛队伍
  const playInTeams = ref<any[]>([])

  // 瑞士轮相关
  const swissStandings = ref<any[]>([])
  const currentSwissRound = ref(0)
  const currentSwissMatches = ref<any[]>([])
  const allSwissMatches = ref<any[]>([]) // 所有瑞士轮比赛（用于对阵图）

  // 淘汰赛相关
  const knockoutMatches = ref<any[]>([])

  // 最终排名
  const finalRankings = ref<any[]>([])

  // ========================================
  // 方法
  // ========================================

  /**
   * 根据赛季获取世界赛数据
   */
  async function fetchWorldsBySeason(season: string) {
    loading.value = true
    error.value = null

    try {
      logger.debug('开始获取世界赛数据', { season })
      const response = await worldsApi.getWorldsBracket(season)

      if (response.data) {
        logger.debug('世界赛数据加载成功', {
          season,
          status: response.data.status,
          teamsCount: response.data.qualified_teams?.length
        })

        const worldsData = {
          id: response.data.id as any,
          season: season,
          status: mapBackendStatus(response.data.status),
          champion: response.data.champion,
          runnerUp: response.data.runnerUp,
          thirdPlace: response.data.thirdPlace,
          fourthPlace: response.data.fourthPlace,
          quarterFinalists: response.data.quarterFinalists,
          groupStageTeams: response.data.groupStageTeams,
          pointsDistribution: response.data.pointsDistribution ? {
            champion: response.data.pointsDistribution.champion || 20,
            runnerUp: response.data.pointsDistribution.runnerUp || 16,
            thirdPlace: response.data.pointsDistribution.thirdPlace || 12,
            fourthPlace: response.data.pointsDistribution.fourthPlace || 8,
            quarterFinalist: response.data.pointsDistribution.quarterFinalist || 6,
            groupStage: (response.data.pointsDistribution as any).groupStage || (response.data.pointsDistribution as any).groupStageEliminated || 4
          } : undefined
        }

        currentWorlds.value = worldsData

        // 存入Map，供历史查看
        worldsBrackets.value.set(season, worldsData)

        // 读取当前瑞士轮轮次
        if (response.data.currentSwissRound !== undefined) {
          currentSwissRound.value = response.data.currentSwissRound
          logger.debug('加载当前轮次', { round: currentSwissRound.value })
        } else {
          currentSwissRound.value = 0
        }

        // 更新参赛队伍数据
        const teamsData = response.data.playInTeams || response.data.qualified_teams
        if (teamsData) {
          playInTeams.value = teamsData.map((team: any) => ({
            id: team.teamId,
            name: team.teamName,
            regionName: team.regionName,
            isDirect: team.directToKnockout,
            quarterSlot: team.quarterSlot
          }))
          logger.debug('解析队伍数据', { count: playInTeams.value.length })
        } else {
          logger.warn('后端没有返回参赛队伍数据')
        }

        // 更新瑞士轮数据
        if (response.data.swissStandings || response.data.swiss_standings) {
          const standings = response.data.swissStandings || response.data.swiss_standings
          if (standings && Array.isArray(standings)) {
            swissStandings.value = standings.map((standing: any) => ({
              rank: 0,
              teamName: standing.teamName,
              teamId: standing.teamId,
              wins: standing.wins || 0,
              losses: standing.losses || 0,
              status: standing.status
            }))
            logger.debug('加载瑞士轮积分榜', { count: swissStandings.value.length })
          }
        }

        // 获取所有瑞士轮比赛数据
        if (response.data.swissMatches && Array.isArray(response.data.swissMatches)) {
          allSwissMatches.value = response.data.swissMatches
          logger.debug('加载瑞士轮比赛', { count: allSwissMatches.value.length })
        } else {
          allSwissMatches.value = []
        }

        // 更新淘汰赛数据
        if (response.data.knockoutMatches) {
          knockoutMatches.value = response.data.knockoutMatches
          logger.debug('加载淘汰赛比赛', { count: knockoutMatches.value.length })
        } else {
          knockoutMatches.value = []
        }
      } else {
        logger.debug('后端没有返回数据，设置为初始状态')
        currentWorlds.value = {
          season: season,
          status: 'NOT_STARTED'
        }
        playInTeams.value = []
        swissStandings.value = []
        currentSwissMatches.value = []
        knockoutMatches.value = []
      }
    } catch (err: any) {
      // 如果是404，表示该赛季还没有世界赛
      if (err.response?.status === 404 || err.message?.includes('404')) {
        logger.debug('该赛季尚未创建世界赛', { season })
        currentWorlds.value = {
          season: season,
          status: 'NOT_STARTED'
        }
        playInTeams.value = []
        swissStandings.value = []
        currentSwissMatches.value = []
        knockoutMatches.value = []
      } else {
        error.value = err.message || '获取世界赛数据失败'
        handleError(err, {
          component: 'WorldsStore',
          userAction: '获取世界赛数据',
          silent: true
        })
      }
    } finally {
      loading.value = false
    }
  }

  /**
   * 创建世界赛
   */
  async function createWorlds() {
    loading.value = true
    error.value = null

    try {
      logger.info('创建世界赛')
      const response = await worldsApi.generateWorlds({})

      if (response.data) {
        currentWorlds.value = {
          id: response.data.id as any,
          season: response.data.seasonId || 'S1',
          status: 'NOT_STARTED'
        }
        logger.info('世界赛创建成功', { id: response.data.id })
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成世界赛失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '创建世界赛'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 进行入围赛抽签
   * 注意：实际上后端在生成世界赛时就已经完成了队伍分配
   * 这个方法只是重新获取数据并更新状态
   */
  async function conductPlayInDraw() {
    if (!currentWorlds.value?.id) {
      throw new Error('请先创建世界赛')
    }

    loading.value = true
    error.value = null

    try {
      // 重新获取世界赛数据，后端应该已经完成了队伍分配
      await fetchWorldsBySeason(currentWorlds.value.season)

      // 如果没有队伍数据，说明后端还没完成初始化
      if (playInTeams.value.length === 0) {
        throw new Error('世界赛队伍数据尚未生成，请稍后再试')
      }

      logger.info('入围赛抽签完成', { teamsCount: playInTeams.value.length })

      // 更新状态为已抽签
      if (currentWorlds.value.status === 'NOT_STARTED') {
        currentWorlds.value.status = 'PLAY_IN_DRAW'
      }
    } catch (err: any) {
      error.value = err.message || '入围赛抽签失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '入围赛抽签'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 开始小组赛
   */
  async function startGroupStage() {
    if (!currentWorlds.value?.id) {
      throw new Error('世界赛不存在')
    }

    loading.value = true
    error.value = null

    try {
      logger.info('开始小组赛', { worldsId: currentWorlds.value.id })

      // 先更新数据库中的状态
      await worldsApi.updateWorldsStatus(currentWorlds.value.id.toString(), 'group_stage')
      logger.debug('数据库状态已更新为 group_stage')

      // 更新本地状态为小组赛阶段
      currentWorlds.value.status = 'GROUP_STAGE'

      // 从后端获取瑞士轮积分榜
      const response = await worldsApi.getSwissStandings(currentWorlds.value.id.toString())

      if (response.data) {
        swissStandings.value = response.data.map((standing: any) => ({
          rank: 0,
          teamName: standing.teamName,
          teamId: standing.teamId,
          wins: standing.wins || 0,
          losses: standing.losses || 0
        }))
        logger.debug('瑞士轮积分榜加载成功', { count: swissStandings.value.length })
      }

      currentSwissRound.value = 0
      logger.info('小组赛开始成功')
    } catch (err: any) {
      error.value = err.response?.data?.error?.message || err.message || '开始小组赛失败'
      currentWorlds.value.status = 'PLAY_IN_DRAW' // 回滚状态
      handleError(err, {
        component: 'WorldsStore',
        userAction: '开始小组赛'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 生成瑞士轮下一轮对阵
   */
  async function generateSwissRound() {
    if (!currentWorlds.value?.id) {
      throw new Error('世界赛不存在')
    }

    logger.info('开始生成瑞士轮对阵', {
      worldsId: currentWorlds.value.id,
      currentRound: currentSwissRound.value
    })

    loading.value = true
    error.value = null

    try {
      const response = await worldsApi.generateSwissRound(currentWorlds.value.id.toString())

      if (response.data) {
        currentSwissRound.value += 1
        const newMatches = response.data.matches || response.data || []
        logger.debug('新生成的比赛', { count: newMatches.length })

        currentSwissMatches.value = newMatches

        // 将新比赛添加到所有比赛列表中
        allSwissMatches.value.push(...newMatches)
        logger.debug('所有瑞士轮比赛', { count: allSwissMatches.value.length })

        // 生成新一轮对阵后，更新积分榜
        await updateSwissStandings()
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成瑞士轮对阵失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '生成瑞士轮对阵'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 模拟瑞士轮比赛
   */
  async function simulateSwissMatch(matchId: number) {
    if (!currentWorlds.value?.id) {
      throw new Error('世界赛不存在')
    }

    loading.value = true
    error.value = null

    try {
      // 调用后端API模拟比赛
      const response = await worldsApi.simulateWorldsMatch({
        matchId: matchId.toString(),
        matchType: 'swiss'
      })

      if (response.data) {
        logger.debug('瑞士轮比赛模拟成功', { matchId })

        const updatedMatch = response.data.match

        // 更新allSwissMatches中的比赛数据
        const matchIndex = allSwissMatches.value.findIndex(m => m.id == matchId)
        if (matchIndex !== -1 && updatedMatch) {
          allSwissMatches.value[matchIndex] = updatedMatch
        }

        // 更新当前轮次比赛数据
        const currentMatchIndex = currentSwissMatches.value.findIndex(m => m.id == matchId)
        if (currentMatchIndex !== -1 && updatedMatch) {
          currentSwissMatches.value[currentMatchIndex] = updatedMatch
        }

        // 更新积分榜
        await updateSwissStandings()
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '模拟瑞士轮比赛'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置瑞士轮比赛获胜者
   */
  async function setSwissMatchWinner(matchId: number, winnerId: number) {
    loading.value = true
    error.value = null

    try {
      const matchIndex = currentSwissMatches.value.findIndex(m => m.id === matchId)
      if (matchIndex !== -1) {
        currentSwissMatches.value[matchIndex].winnerId = winnerId
      }

      // 更新积分榜
      await updateSwissStandings()
    } catch (err: any) {
      error.value = err.message || '设置比赛结果失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '设置瑞士轮比赛结果'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新瑞士轮积分榜
   */
  async function updateSwissStandings() {
    if (!currentWorlds.value?.id) return

    try {
      const response = await worldsApi.getSwissStandings(currentWorlds.value.id.toString())

      if (response.data) {
        swissStandings.value = response.data
      }
    } catch (err: any) {
      logger.error('更新瑞士轮积分榜失败', { error: err })
    }
  }

  /**
   * 模拟淘汰赛比赛
   */
  async function simulateKnockoutMatch(matchId: number) {
    if (!currentWorlds.value?.id) {
      throw new Error('世界赛不存在')
    }

    logger.info('开始模拟淘汰赛比赛', { matchId })

    loading.value = true
    error.value = null

    try {
      // 调用后端API模拟淘汰赛比赛
      const response = await worldsApi.simulateWorldsMatch({
        matchId: matchId.toString(),
        matchType: 'knockout'
      })

      if (response.data && response.data.match) {
        // 重新获取完整的世界赛数据
        await fetchWorldsBySeason(currentWorlds.value.season)
        logger.info('淘汰赛比赛模拟完成', { matchId })
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '模拟淘汰赛比赛失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '模拟淘汰赛比赛'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 生成淘汰赛对阵
   */
  async function generateKnockoutBracket() {
    if (!currentWorlds.value?.id) {
      throw new Error('世界赛不存在')
    }

    logger.info('开始生成淘汰赛对阵', { worldsId: currentWorlds.value.id })

    loading.value = true
    error.value = null

    try {
      const response = await worldsApi.generateKnockout(currentWorlds.value.id.toString())

      if (response.data && response.data.matches) {
        knockoutMatches.value = response.data.matches
        logger.info('淘汰赛对阵生成成功', { matchCount: knockoutMatches.value.length })

        // 更新状态为淘汰赛阶段
        currentWorlds.value.status = 'KNOCKOUT'
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成淘汰赛对阵失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '生成淘汰赛对阵'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置淘汰赛比赛获胜者
   */
  async function setKnockoutMatchWinner(matchId: number, winnerId: number) {
    loading.value = true
    error.value = null

    try {
      const matchIndex = knockoutMatches.value.findIndex(m => m.id === matchId)
      if (matchIndex !== -1) {
        knockoutMatches.value[matchIndex].winnerId = winnerId

        // 根据轮次推进队伍
        const match = knockoutMatches.value[matchIndex]

        if (match.round === 'QUARTER_FINAL') {
          generateSemiFinals()
        } else if (match.round === 'SEMI_FINAL') {
          generateFinals()
        } else if (match.round === 'FINAL' || match.round === 'THIRD_PLACE') {
          checkIfCompleted()
        }
      }
    } catch (err: any) {
      error.value = err.message || '设置比赛结果失败'
      handleError(err, {
        component: 'WorldsStore',
        userAction: '设置淘汰赛比赛结果'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 生成半决赛
   */
  function generateSemiFinals() {
    const quarterFinals = knockoutMatches.value.filter(m => m.round === 'QUARTER_FINAL')
    const allFinished = quarterFinals.every(m => m.winnerId)

    if (allFinished && !knockoutMatches.value.some(m => m.round === 'SEMI_FINAL')) {
      const winners = quarterFinals.map(m => ({
        id: m.winnerId,
        name: m.winnerId === m.team1Id ? m.team1Name : m.team2Name
      }))

      if (winners.length === 4 && winners.every(w => w.id && w.name)) {
        knockoutMatches.value.push(
          { id: 5, round: 'SEMI_FINAL', team1Id: winners[0]!.id, team1Name: winners[0]!.name, team2Id: winners[1]!.id, team2Name: winners[1]!.name },
          { id: 6, round: 'SEMI_FINAL', team1Id: winners[2]!.id, team1Name: winners[2]!.name, team2Id: winners[3]!.id, team2Name: winners[3]!.name }
        )
      }
    }
  }

  /**
   * 生成决赛和季军赛
   */
  function generateFinals() {
    const semiFinals = knockoutMatches.value.filter(m => m.round === 'SEMI_FINAL')
    const allFinished = semiFinals.every(m => m.winnerId)

    if (allFinished && !knockoutMatches.value.some(m => m.round === 'FINAL')) {
      const winners = semiFinals.map(m => ({
        id: m.winnerId,
        name: m.winnerId === m.team1Id ? m.team1Name : m.team2Name
      }))
      const losers = semiFinals.map(m => ({
        id: m.winnerId === m.team1Id ? m.team2Id : m.team1Id,
        name: m.winnerId === m.team1Id ? m.team2Name : m.team1Name
      }))

      if (winners.length === 2 && losers.length === 2 && 
          winners.every(w => w.id && w.name) && losers.every(l => l.id && l.name)) {
        knockoutMatches.value.push(
          { id: 7, round: 'THIRD_PLACE', team1Id: losers[0]!.id, team1Name: losers[0]!.name, team2Id: losers[1]!.id, team2Name: losers[1]!.name },
          { id: 8, round: 'FINAL', team1Id: winners[0]!.id, team1Name: winners[0]!.name, team2Id: winners[1]!.id, team2Name: winners[1]!.name }
        )
      }
    }
  }

  /**
   * 检查是否完成
   */
  function checkIfCompleted() {
    const finalMatch = knockoutMatches.value.find(m => m.round === 'FINAL')
    const thirdPlaceMatch = knockoutMatches.value.find(m => m.round === 'THIRD_PLACE')

    if (finalMatch?.winnerId && thirdPlaceMatch?.winnerId) {
      currentWorlds.value.status = 'COMPLETED'

      // 生成最终排名
      const champion = finalMatch.winnerId === finalMatch.team1Id ? finalMatch.team1Name : finalMatch.team2Name
      const runnerUp = finalMatch.winnerId === finalMatch.team1Id ? finalMatch.team2Name : finalMatch.team1Name
      const thirdPlace = thirdPlaceMatch.winnerId === thirdPlaceMatch.team1Id ? thirdPlaceMatch.team1Name : thirdPlaceMatch.team2Name
      const fourthPlace = thirdPlaceMatch.winnerId === thirdPlaceMatch.team1Id ? thirdPlaceMatch.team2Name : thirdPlaceMatch.team1Name

      finalRankings.value = [
        { rank: 1, teamName: champion, prize: '$500,000' },
        { rank: 2, teamName: runnerUp, prize: '$300,000' },
        { rank: 3, teamName: thirdPlace, prize: '$150,000' },
        { rank: 4, teamName: fourthPlace, prize: '$100,000' }
      ]
    }
  }

  /**
   * 映射后端状态到前端状态
   */
  function mapBackendStatus(backendStatus: any): WorldsData['status'] {
    const statusMap: Record<string, WorldsData['status']> = {
      'not_started': 'NOT_STARTED',
      'play_in_draw': 'PLAY_IN_DRAW',
      'group_stage': 'GROUP_STAGE',
      'knockout': 'KNOCKOUT',
      'knockout_stage': 'KNOCKOUT',
      'completed': 'COMPLETED'
    }
    logger.debug('映射状态', { from: backendStatus, to: statusMap[backendStatus] })
    return statusMap[backendStatus] || 'NOT_STARTED'
  }

  /**
   * 清除所有状态（切换存档时调用）
   */
  function clearAll() {
    loading.value = false
    error.value = null
    worldsBrackets.value = new Map()
    currentWorlds.value = { season: 'S1', status: 'NOT_STARTED' }
    playInTeams.value = []
    swissStandings.value = []
    currentSwissRound.value = 0
    currentSwissMatches.value = []
    allSwissMatches.value = []
    knockoutMatches.value = []
    finalRankings.value = []
  }

  /**
   * 重置错误
   */
  function resetError() {
    error.value = null
  }

  return {
    // 状态
    loading,
    error,
    currentWorlds,
    playInTeams,
    swissStandings,
    currentSwissRound,
    currentSwissMatches,
    allSwissMatches,
    knockoutMatches,
    finalRankings,
    worldsBrackets, // 多赛季数据存储

    // 方法
    fetchWorldsBySeason,
    createWorlds,
    conductPlayInDraw,
    startGroupStage,
    generateSwissRound,
    simulateSwissMatch,
    setSwissMatchWinner,
    updateSwissStandings,
    simulateKnockoutMatch,
    generateKnockoutBracket,
    setKnockoutMatchWinner,
    clearAll,
    resetError
  }
})
















