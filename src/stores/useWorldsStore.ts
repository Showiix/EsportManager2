import { ref } from 'vue'
import { defineStore } from 'pinia'
import { worldsApi } from '@/api'

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
      console.log('开始获取世界赛数据，赛季:', season)
      const response = await worldsApi.getWorldsBracket(season)
      console.log('后端返回的完整响应:', response)

      if (response.data) {
        console.log('世界赛数据:', response.data)
        console.log('参赛队伍原始数据:', response.data.qualified_teams)

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
          console.log('✅ 从后端加载当前轮次:', currentSwissRound.value)
        } else {
          currentSwissRound.value = 0
        }

        // 更新参赛队伍数据
        // 优先使用playInTeams，如果没有则使用qualified_teams
        const teamsData = response.data.playInTeams || response.data.qualified_teams
        if (teamsData) {
          playInTeams.value = teamsData.map((team: any) => ({
            id: team.teamId,
            name: team.teamName,
            regionName: team.regionName,
            isDirect: team.directToKnockout,
            quarterSlot: team.quarterSlot
          }))
          console.log('解析后的队伍数据:', playInTeams.value)
        } else {
          console.warn('后端没有返回参赛队伍数据')
        }

        // 更新瑞士轮数据
        if (response.data.swissStandings || response.data.swiss_standings) {
          const standings = response.data.swissStandings || response.data.swiss_standings
          if (standings && Array.isArray(standings)) {
            swissStandings.value = standings.map((standing: any) => ({
              rank: 0, // 排名将在后续更新
              teamName: standing.teamName,
              teamId: standing.teamId,
              wins: standing.wins || 0,
              losses: standing.losses || 0,
              status: standing.status
            }))
            console.log('从后端加载的瑞士轮积分榜:', swissStandings.value)
          }
        }
        
        // 获取所有瑞士轮比赛数据
        if (response.data.swissMatches && Array.isArray(response.data.swissMatches)) {
          allSwissMatches.value = response.data.swissMatches
          console.log('✅ 从后端加载瑞士轮比赛:', allSwissMatches.value.length, '场')
        } else {
          allSwissMatches.value = []
        }

        // 更新淘汰赛数据
        if (response.data.knockoutMatches) {
          knockoutMatches.value = response.data.knockoutMatches
          console.log('✅ 从后端加载淘汰赛比赛:', knockoutMatches.value.length, '场')
        } else {
          knockoutMatches.value = []
        }
      } else {
        console.log('后端没有返回 data，设置为初始状态')
        // 如果没有数据，设置为初始状态
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
      console.error('获取世界赛数据时出错:', err)
      console.error('错误响应:', err.response)
      // 如果是404，表示该赛季还没有世界赛
      if (err.response?.status === 404 || err.message?.includes('404')) {
        console.log('404错误，该赛季尚未创建世界赛')
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
        console.error('Failed to fetch Worlds data:', err)
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
      // 后端会自动检测当前赛季
      const response = await worldsApi.generateWorlds({})

      if (response.data) {
        currentWorlds.value = {
          id: response.data.id as any,
          season: response.data.seasonId || 'S1',
          status: 'NOT_STARTED'
        }
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成世界赛失败'
      console.error('Failed to create Worlds:', err)
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

      console.log('入围赛抽签完成，队伍数据:', playInTeams.value)

      // 更新状态为已抽签
      if (currentWorlds.value.status === 'NOT_STARTED') {
        currentWorlds.value.status = 'PLAY_IN_DRAW'
      }
    } catch (err: any) {
      error.value = err.message || '入围赛抽签失败'
      console.error('Failed to conduct play-in draw:', err)
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
      console.log('开始小组赛，世界赛ID:', currentWorlds.value.id)
      
      // 先更新数据库中的状态
      await worldsApi.updateWorldsStatus(currentWorlds.value.id.toString(), 'group_stage')
      console.log('✅ 数据库状态已更新为 group_stage')
      
      // 更新本地状态为小组赛阶段
      currentWorlds.value.status = 'GROUP_STAGE'

      // 从后端获取瑞士轮积分榜（后端在创建世界赛时已初始化）
      const response = await worldsApi.getSwissStandings(currentWorlds.value.id.toString())
      console.log('瑞士轮积分榜响应:', response)
      
      if (response.data) {
        swissStandings.value = response.data.map((standing: any) => ({
          rank: 0, // 排名将在后续更新
          teamName: standing.teamName,
          teamId: standing.teamId,
          wins: standing.wins || 0,
          losses: standing.losses || 0
        }))
        console.log('解析后的瑞士轮积分榜:', swissStandings.value)
      }

      currentSwissRound.value = 0
      console.log('小组赛开始成功')
    } catch (err: any) {
      console.error('开始小组赛时出错:', err)
      console.error('错误详情:', err.response || err.message)
      error.value = err.response?.data?.error?.message || err.message || '开始小组赛失败'
      currentWorlds.value.status = 'PLAY_IN_DRAW' // 回滚状态
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

    console.log('🎮 [generateSwissRound] 开始生成瑞士轮对阵')
    console.log('🎮 [generateSwissRound] 当前世界赛ID:', currentWorlds.value.id)
    console.log('🎮 [generateSwissRound] 当前轮次:', currentSwissRound.value)

    loading.value = true
    error.value = null

    try {
      const response = await worldsApi.generateSwissRound(currentWorlds.value.id.toString())
      console.log('🎮 [generateSwissRound] 后端响应:', response)

      if (response.data) {
        currentSwissRound.value += 1
        const newMatches = response.data.matches || response.data || []
        console.log('🎮 [generateSwissRound] 新生成的比赛:', newMatches)
        console.log('🎮 [generateSwissRound] 比赛数量:', newMatches.length)
        
        currentSwissMatches.value = newMatches
        
        // 将新比赛添加到所有比赛列表中
        allSwissMatches.value.push(...newMatches)
        console.log('🎮 [generateSwissRound] 所有瑞士轮比赛数量:', allSwissMatches.value.length)
        console.log('🎮 [generateSwissRound] 所有瑞士轮比赛:', allSwissMatches.value)
        
        // 生成新一轮对阵后，更新积分榜
        await updateSwissStandings()
        console.log('🎮 [generateSwissRound] 积分榜更新完成')
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成瑞士轮对阵失败'
      console.error('❌ [generateSwissRound] 生成失败:', err)
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
        console.log('🎮 [simulateSwissMatch] 模拟成功，返回数据:', response.data)
        
        const updatedMatch = response.data.match
        console.log('🎮 [simulateSwissMatch] 更新的比赛:', updatedMatch)
        
        // 更新allSwissMatches中的比赛数据
        const matchIndex = allSwissMatches.value.findIndex(m => m.id == matchId)
        if (matchIndex !== -1 && updatedMatch) {
          allSwissMatches.value[matchIndex] = updatedMatch
          console.log('🎮 [simulateSwissMatch] 更新了allSwissMatches中的比赛')
        }

        // 更新当前轮次比赛数据
        const currentMatchIndex = currentSwissMatches.value.findIndex(m => m.id == matchId)
        if (currentMatchIndex !== -1 && updatedMatch) {
          currentSwissMatches.value[currentMatchIndex] = updatedMatch
          console.log('🎮 [simulateSwissMatch] 更新了currentSwissMatches中的比赛')
        }

        // 更新积分榜
        await updateSwissStandings()
        console.log('🎮 [simulateSwissMatch] 积分榜更新完成')
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      console.error('Failed to simulate Swiss match:', err)
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
      // TODO: 调用后端API设置比赛结果
      const matchIndex = currentSwissMatches.value.findIndex(m => m.id === matchId)
      if (matchIndex !== -1) {
        currentSwissMatches.value[matchIndex].winnerId = winnerId
      }

      // 更新积分榜
      await updateSwissStandings()
    } catch (err: any) {
      error.value = err.message || '设置比赛结果失败'
      console.error('Failed to set match winner:', err)
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
      console.error('Failed to update Swiss standings:', err)
    }
  }

  /**
   * 模拟淘汰赛比赛
   */
  async function simulateKnockoutMatch(matchId: number) {
    if (!currentWorlds.value?.id) {
      throw new Error('世界赛不存在')
    }

    console.log('🏆 [simulateKnockoutMatch] 开始模拟淘汰赛比赛')
    console.log('🏆 [simulateKnockoutMatch] 比赛ID:', matchId)

    loading.value = true
    error.value = null

    try {
      // 调用后端API模拟淘汰赛比赛
      const response = await worldsApi.simulateWorldsMatch({
        matchId: matchId.toString(),
        matchType: 'knockout'
      })
      console.log('🏆 [simulateKnockoutMatch] 后端响应:', response)

      if (response.data && response.data.match) {
        // 重新获取完整的世界赛数据，以更新所有相关比赛（包括推进到下一轮的队伍）
        await fetchWorldsBySeason(currentWorlds.value.season)
        console.log('🏆 [simulateKnockoutMatch] 已刷新完整淘汰赛对阵数据')
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '模拟淘汰赛比赛失败'
      console.error('❌ [simulateKnockoutMatch] 模拟失败:', err)
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

    console.log('🏆 [generateKnockoutBracket] 开始生成淘汰赛对阵')
    console.log('🏆 [generateKnockoutBracket] 当前世界赛ID:', currentWorlds.value.id)

    loading.value = true
    error.value = null

    try {
      const response = await worldsApi.generateKnockout(currentWorlds.value.id.toString())
      console.log('🏆 [generateKnockoutBracket] 后端响应:', response)

      if (response.data && response.data.matches) {
        knockoutMatches.value = response.data.matches
        console.log('🏆 [generateKnockoutBracket] 淘汰赛比赛数量:', knockoutMatches.value.length)
        
        // 更新状态为淘汰赛阶段
        currentWorlds.value.status = 'KNOCKOUT'
        console.log('🏆 [generateKnockoutBracket] 状态已更新为KNOCKOUT')
      }

      return response.data
    } catch (err: any) {
      error.value = err.message || '生成淘汰赛对阵失败'
      console.error('❌ [generateKnockoutBracket] 生成失败:', err)
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
      // TODO: 调用后端API设置比赛结果
      const matchIndex = knockoutMatches.value.findIndex(m => m.id === matchId)
      if (matchIndex !== -1) {
        knockoutMatches.value[matchIndex].winnerId = winnerId

        // 根据轮次推进队伍
        const match = knockoutMatches.value[matchIndex]

        if (match.round === 'QUARTER_FINAL') {
          // 生成半决赛
          generateSemiFinals()
        } else if (match.round === 'SEMI_FINAL') {
          // 生成决赛和季军赛
          generateFinals()
        } else if (match.round === 'FINAL' || match.round === 'THIRD_PLACE') {
          // 检查是否完成
          checkIfCompleted()
        }
      }
    } catch (err: any) {
      error.value = err.message || '设置比赛结果失败'
      console.error('Failed to set knockout match winner:', err)
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
    console.log('📊 [mapBackendStatus] 映射状态:', backendStatus, '->', statusMap[backendStatus])
    return statusMap[backendStatus] || 'NOT_STARTED'
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
    resetError
  }
})
















