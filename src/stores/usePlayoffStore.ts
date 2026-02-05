import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  PlayoffBracket,
  PlayoffMatch,
  GeneratePlayoffRequest,
  SimulatePlayoffMatchRequest
} from '@/types'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('PlayoffStore')

export const usePlayoffStore = defineStore('playoff', () => {
  // 状态
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 各赛区的季后赛对阵
  const playoffBrackets = ref<Map<string, PlayoffBracket>>(new Map())

  // 当前选中的季后赛
  const currentBracket = ref<PlayoffBracket | null>(null)

  // 计算属性
  const currentQualifiedTeams = computed(() => {
    return currentBracket.value?.qualifiedTeams || []
  })

  const currentRounds = computed(() => {
    return currentBracket.value?.rounds || []
  })

  const isPlayoffComplete = computed(() => {
    return currentBracket.value?.status === 'completed'
  })

  const champion = computed(() => {
    return currentBracket.value?.champion || null
  })

  const runnerUp = computed(() => {
    return currentBracket.value?.runnerUp || null
  })

  // 动作

  /**
   * 获取季后赛对阵信息
   */
  async function fetchPlayoffBracket(competitionId: string, regionId: string) {
    loading.value = true
    error.value = null

    try {
      const { playoffApi } = await import('@/api')
      const response = await playoffApi.getPlayoffBracket(competitionId, regionId)

      if (response.data) {
        // 重要：key 需要包含 competitionType 来区分春季赛和夏季赛
        const key = `${competitionId}-${regionId}-${response.data.competitionType}`
        playoffBrackets.value.set(key, response.data)
        currentBracket.value = response.data
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '获取季后赛信息失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '获取季后赛信息',
        silent: true
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取赛区所有季后赛
   */
  async function fetchRegionPlayoffs(regionId: string, seasonId: string) {
    loading.value = true
    error.value = null

    try {
      const { playoffApi } = await import('@/api')
      logger.debug('调用 API: getRegionPlayoffs', { regionId, seasonId })
      const response = await playoffApi.getRegionPlayoffs(regionId, seasonId)

      logger.debug('API 响应', { response })

      if (response.data) {
        logger.debug('收到季后赛对阵', { count: response.data.length })

        // 调试：显示当前 Map 中的所有 key
        logger.debug('当前 Map 中的所有 key', { keys: Array.from(playoffBrackets.value.keys()) })

        // 存储所有季后赛对阵
        response.data.forEach(bracket => {
          // 重要：key 需要包含 competitionType 来区分春季赛和夏季赛
          const key = `${bracket.competitionId}-${bracket.regionId}-${bracket.competitionType}`
          logger.debug('存储季后赛对阵', {
            key,
            competitionId: bracket.competitionId,
            competitionType: bracket.competitionType,
            regionId: bracket.regionId,
            regionName: bracket.regionName,
            seasonId: bracket.seasonId,
            status: bracket.status,
            qualifiedTeamsCount: bracket.qualifiedTeams?.length || 0
          })
          playoffBrackets.value.set(key, bracket)
        })

        // 调试：显示更新后 Map 中的所有 key 和 competitionType
        logger.debug('更新后 Map 中的数据')
        playoffBrackets.value.forEach((bracket, key) => {
          logger.debug('Map entry', {
            key,
            competitionType: bracket.competitionType,
            region: bracket.regionName,
            qualifiedTeams: bracket.qualifiedTeams?.length || 0
          })
        })

        return response.data
      } else {
        logger.debug('response.data 为空或未定义')
        return []
      }
    } catch (err: any) {
      error.value = err.message || '获取赛区季后赛失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '获取赛区季后赛',
        silent: true
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 生成季后赛对阵
   */
  async function generatePlayoff(request: GeneratePlayoffRequest) {
    loading.value = true
    error.value = null

    try {
      const { playoffApi } = await import('@/api')
      const response = await playoffApi.generatePlayoff(request)

      if (response.data) {
        // 重要：key 需要包含 competitionType 来区分春季赛和夏季赛
        const key = `${request.competitionId}-${request.regionId}-${request.competitionType}`
        playoffBrackets.value.set(key, response.data)
        currentBracket.value = response.data
        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '生成季后赛失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '生成季后赛',
        canRetry: true,
        retryFn: () => generatePlayoff(request)
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 检查是否可以生成季后赛
   */
  async function checkPlayoffEligibility(competitionId: string, regionId: string) {
    try {
      const { playoffApi } = await import('@/api')
      const response = await playoffApi.checkPlayoffEligibility(competitionId, regionId)
      return response.data
    } catch (err: any) {
      error.value = err.message || '检查季后赛资格失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '检查季后赛资格',
        silent: true
      })
      throw err
    }
  }

  /**
   * 获取有资格进入季后赛的队伍(常规赛前4名)
   */
  async function getQualifiedTeams(competitionId: string, regionId: string) {
    try {
      const { playoffApi } = await import('@/api')
      const response = await playoffApi.getQualifiedTeams(competitionId, regionId)
      return response.data
    } catch (err: any) {
      error.value = err.message || '获取晋级队伍失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '获取晋级队伍',
        silent: true
      })
      throw err
    }
  }

  /**
   * 模拟季后赛单场比赛(BO5)
   */
  async function simulatePlayoffMatch(request: SimulatePlayoffMatchRequest) {
    loading.value = true
    error.value = null

    try {
      const { playoffApi } = await import('@/api')
      const response = await playoffApi.simulatePlayoffMatch(request)

      if (response.data) {
        // 更新当前对阵信息
        if (currentBracket.value) {
          // 更新比赛结果
          updateMatchInBracket(currentBracket.value, response.data.match)

          // 如果季后赛完成,更新最终排名
          if (response.data.isPlayoffComplete && response.data.finalStandings) {
            currentBracket.value.champion = response.data.finalStandings.champion
            currentBracket.value.runnerUp = response.data.finalStandings.runnerUp
            currentBracket.value.thirdPlace = response.data.finalStandings.thirdPlace
            currentBracket.value.fourthPlace = response.data.finalStandings.fourthPlace
            currentBracket.value.status = 'completed'
          }
        }

        return response.data
      }
    } catch (err: any) {
      error.value = err.message || '模拟比赛失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '模拟季后赛比赛',
        canRetry: true,
        retryFn: () => simulatePlayoffMatch(request)
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 批量模拟整个季后赛(一键模拟)
   * @param bracket 要模拟的季后赛对阵
   * @param onProgress 进度回调函数
   */
  async function batchSimulatePlayoff(
    bracket: PlayoffBracket,
    onProgress?: (progress: number) => void
  ) {
    loading.value = true
    error.value = null

    try {
      const { playoffApi } = await import('@/api')
      let isComplete = false
      let simulatedCount = 0
      const maxIterations = 10 // 最多模拟10场比赛

      while (!isComplete && simulatedCount < maxIterations) {
        // 重新获取最新的季后赛数据
        const freshBracketResponse = await playoffApi.getPlayoffBracket(
          bracket.competitionId,
          bracket.regionId
        )

        if (!freshBracketResponse.data) {
          throw new Error('无法获取最新的季后赛数据')
        }

        const freshBracket = freshBracketResponse.data

        // 检查是否已完成
        if (freshBracket.status === 'completed') {
          isComplete = true
          // 更新到store中，重要：key 需要包含 competitionType
          const key = `${bracket.competitionId}-${bracket.regionId}-${bracket.competitionType}`
          playoffBrackets.value.set(key, freshBracket)
          break
        }

        // 找到第一个未完成的比赛
        let nextMatch: PlayoffMatch | null = null
        for (const round of freshBracket.rounds) {
          if (round.status === 'completed') continue

          for (const match of round.matches) {
            if (match.status !== 'completed') {
              nextMatch = match
              break
            }
          }
          if (nextMatch) break
        }

        if (!nextMatch) {
          // 没有找到未完成的比赛
          isComplete = true
          break
        }

        // 模拟这场比赛
        const result = await playoffApi.simulatePlayoffMatch({
          matchId: String(nextMatch.id),
          competitionId: String(nextMatch.competitionId)
        })

        simulatedCount++
        const progress = Math.min(Math.round((simulatedCount / 4) * 100), 95)
        if (onProgress) {
          onProgress(progress)
        }

        // 短暂延迟
        await new Promise(resolve => setTimeout(resolve, 300))

        // 检查是否完成
        if (result.data?.isPlayoffComplete) {
          isComplete = true
          if (onProgress) {
            onProgress(100)
          }

          // 更新到store中，重要：key 需要包含 competitionType
          const key = `${bracket.competitionId}-${bracket.regionId}-${bracket.competitionType}`
          const finalBracketResponse = await playoffApi.getPlayoffBracket(
            bracket.competitionId,
            bracket.regionId
          )
          if (finalBracketResponse.data) {
            playoffBrackets.value.set(key, finalBracketResponse.data)
          }

          return result.data
        }
      }

      if (!isComplete) {
        throw new Error('已达到最大模拟次数')
      }

      return null
    } catch (err: any) {
      error.value = err.message || '批量模拟失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '批量模拟季后赛',
        canRetry: true,
        retryFn: () => batchSimulatePlayoff(bracket, onProgress)
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新对阵中的比赛信息
   */
  function updateMatchInBracket(bracket: PlayoffBracket, match: PlayoffMatch) {
    bracket.rounds.forEach(round => {
      const index = round.matches.findIndex(m => m.id === match.id)
      if (index !== -1) {
        round.matches[index] = match
      }
    })
  }

  /**
   * 设置当前季后赛
   */
  function setCurrentBracket(bracket: PlayoffBracket | null) {
    currentBracket.value = bracket
  }

  /**
   * 根据key获取季后赛对阵
   */
  function getBracketByKey(competitionId: string, regionId: string, competitionType: 'spring' | 'summer'): PlayoffBracket | undefined {
    // 重要：key 需要包含 competitionType 来区分春季赛和夏季赛
    const key = `${competitionId}-${regionId}-${competitionType}`
    return playoffBrackets.value.get(key)
  }

  /**
   * 清空所有数据
   */
  function clearAll() {
    playoffBrackets.value.clear()
    currentBracket.value = null
    error.value = null
  }

  /**
   * 重置错误
   */
  function resetError() {
    error.value = null
  }

  /**
   * 完成季后赛（更新赛事状态）
   * @param bracket 要完成的季后赛对阵
   */
  async function completePlayoff(bracket: PlayoffBracket) {
    loading.value = true
    error.value = null

    try {
      logger.info('开始完成季后赛流程', { bracket })

      // 检查季后赛是否已完成
      if (bracket.status !== 'completed') {
        throw new Error('季后赛尚未完成，请先完成所有比赛')
      }

      // 后端会根据competition的type自动判断是季后赛还是常规赛
      // 这里我们需要获取季后赛对应的competition ID
      // 注意：bracket.competitionId 实际上是常规赛的ID，需要找到对应的季后赛competition
      // 但根据后端设计，季后赛是基于常规赛competitionId的，所以这里直接使用bracket.competitionId

      logger.debug('完成季后赛', { competitionId: bracket.competitionId, regionId: bracket.regionId })

      // 这里暂时不调用finishCompetition，因为季后赛的完成状态已经在模拟最后一场比赛时设置了
      // 如果后端有专门的完成季后赛API，可以在这里调用

      logger.info('季后赛已标记为完成')

      // 更新store中的状态，重要：key 需要包含 competitionType
      const key = `${bracket.competitionId}-${bracket.regionId}-${bracket.competitionType}`
      const updatedBracket = { ...bracket, status: 'completed' as any }
      playoffBrackets.value.set(key, updatedBracket)

      if (currentBracket.value?.id === bracket.id) {
        currentBracket.value = updatedBracket
      }

      logger.info('季后赛完成流程执行完毕')

      return updatedBracket
    } catch (err: any) {
      error.value = err.message || '完成季后赛失败'
      handleError(err, {
        component: 'PlayoffStore',
        userAction: '完成季后赛'
      })
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    // 状态
    loading,
    error,
    playoffBrackets,
    currentBracket,

    // 计算属性
    currentQualifiedTeams,
    currentRounds,
    isPlayoffComplete,
    champion,
    runnerUp,

    // 动作
    fetchPlayoffBracket,
    fetchRegionPlayoffs,
    generatePlayoff,
    checkPlayoffEligibility,
    getQualifiedTeams,
    simulatePlayoffMatch,
    batchSimulatePlayoff,
    setCurrentBracket,
    getBracketByKey,
    clearAll,
    resetError,
    completePlayoff
  }
})
