import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useEventStore } from './useEventStore'
import { useTeamStore } from './useTeamStore'
import { useRegionStore } from './useRegionStore'
import type { Competition, Team, CompetitionType, CompetitionFormat } from '@/types'
import { createLogger } from '@/utils/logger'

const logger = createLogger('AutoTournamentStore')

interface AutoCreationRule {
  id: string
  name: string
  trigger: 'season_end' | 'playoffs_end' | 'competition_end'
  triggerCondition: {
    competitionType?: CompetitionType
    regionId?: string
    status: 'completed'
  }
  action: {
    createCompetition: {
      type: CompetitionType
      format: CompetitionFormat
      name: string
      teamSelection: 'top_teams' | 'playoff_winners' | 'regional_champions'
      teamCount?: number
      regionIds?: string[]
    }
  }
  enabled: boolean
}

export const useAutoTournamentStore = defineStore('autoTournament', () => {
  // 状态
  const rules = ref<AutoCreationRule[]>([])
  const isProcessing = ref(false)
  const processingLog = ref<string[]>([])

  // 依赖的其他store
  const eventStore = useEventStore()
  const teamStore = useTeamStore()
  const regionStore = useRegionStore()

  // 默认规则配置
  const initializeDefaultRules = () => {
    rules.value = [
      // 春季赛结束后创建春季季后赛
      {
        id: 'spring-to-playoffs',
        name: '春季赛 → 春季季后赛',
        trigger: 'competition_end',
        triggerCondition: {
          competitionType: 'spring',
          status: 'completed'
        },
        action: {
          createCompetition: {
            type: 'spring',
            format: 'playoffs',
            name: '春季季后赛',
            teamSelection: 'top_teams',
            teamCount: 8
          }
        },
        enabled: true
      },

      // 夏季赛结束后创建夏季季后赛
      {
        id: 'summer-to-playoffs',
        name: '夏季赛 → 夏季季后赛',
        trigger: 'competition_end',
        triggerCondition: {
          competitionType: 'summer',
          status: 'completed'
        },
        action: {
          createCompetition: {
            type: 'summer',
            format: 'playoffs',
            name: '夏季季后赛',
            teamSelection: 'top_teams',
            teamCount: 8
          }
        },
        enabled: true
      },

      // 春季季后赛结束后创建MSI
      {
        id: 'spring-playoffs-to-msi',
        name: '春季季后赛 → 季中冠军赛',
        trigger: 'playoffs_end',
        triggerCondition: {
          competitionType: 'spring',
          status: 'completed'
        },
        action: {
          createCompetition: {
            type: 'msi',
            format: 'swiss',
            name: '季中冠军赛',
            teamSelection: 'regional_champions',
            teamCount: 4,
            regionIds: ['1', '2', '3', '4'] // LPL, LCK, LEC, LCS
          }
        },
        enabled: true
      },

      // 夏季季后赛结束后创建全球总决赛
      {
        id: 'summer-playoffs-to-worlds',
        name: '夏季季后赛 → 全球总决赛',
        trigger: 'playoffs_end',
        triggerCondition: {
          competitionType: 'summer',
          status: 'completed'
        },
        action: {
          createCompetition: {
            type: 'worlds',
            format: 'swiss',
            name: '全球总决赛',
            teamSelection: 'regional_champions',
            teamCount: 16,
            regionIds: ['1', '2', '3', '4']
          }
        },
        enabled: true
      },

      // 全球总决赛结束后创建洲际超级杯
      {
        id: 'worlds-to-intercontinental',
        name: '全球总决赛 → 洲际超级杯',
        trigger: 'competition_end',
        triggerCondition: {
          competitionType: 'worlds',
          status: 'completed'
        },
        action: {
          createCompetition: {
            type: 'intercontinental',
            format: 'single_elimination',
            name: '洲际超级杯',
            teamSelection: 'regional_champions',
            teamCount: 4,
            regionIds: ['1', '2', '3', '4']
          }
        },
        enabled: true
      }
    ]
  }

  // 检查并触发自动创建
  const checkAutoCreation = async (completedCompetition: Competition) => {
    if (isProcessing.value) return

    isProcessing.value = true
    processingLog.value = []

    try {
      log(`检查赛事: ${completedCompetition.name} 的自动创建规则`)

      // 查找匹配的规则
      const matchingRules = rules.value.filter(rule => {
        if (!rule.enabled) return false

        const condition = rule.triggerCondition

        // 检查赛事类型匹配
        if (condition.competitionType && condition.competitionType !== completedCompetition.type) {
          return false
        }

        // 检查状态匹配
        if (condition.status !== completedCompetition.status) {
          return false
        }

        // 检查触发类型
        if (rule.trigger === 'playoffs_end' && completedCompetition.format !== 'playoffs') {
          return false
        }

        return true
      })

      if (matchingRules.length === 0) {
        log('未找到匹配的自动创建规则')
        return
      }

      // 执行匹配的规则
      for (const rule of matchingRules) {
        await executeRule(rule, completedCompetition)
      }

    } finally {
      isProcessing.value = false
    }
  }

  // 执行规则
  const executeRule = async (rule: AutoCreationRule, triggerCompetition: Competition) => {
    log(`执行规则: ${rule.name}`)

    const action = rule.action.createCompetition

    // 检查是否已存在相同类型的赛事
    const existingCompetition = eventStore.competitions.find(comp =>
      comp.type === action.type &&
      comp.format === action.format &&
      comp.status !== 'completed'
    )

    if (existingCompetition) {
      log(`跳过创建: ${action.name} 已存在`)
      return
    }

    // 选择参赛队伍
    const selectedTeams = await selectTeams(action, triggerCompetition)

    if (selectedTeams.length === 0) {
      log(`跳过创建: ${action.name} - 无可用参赛队伍`)
      return
    }

    // 创建新赛事
    const newCompetition = {
      name: action.name,
      type: action.type,
      format: action.format,
      seasonId: getCurrentSeasonId(),
      teamIds: selectedTeams.map(team => team.id)
    }

    try {
      await eventStore.createCompetition(newCompetition)
      log(`成功创建: ${action.name}，参赛队伍: ${selectedTeams.length}支`)
    } catch (error) {
      log(`创建失败: ${action.name} - ${error}`)
    }
  }

  // 选择参赛队伍
  const selectTeams = async (action: any, triggerCompetition: Competition): Promise<Team[]> => {
    const allTeams = teamStore.teams
    let selectedTeams: Team[] = []

    switch (action.teamSelection) {
      case 'top_teams':
        // 从每个赛区选择排名最高的队伍
        const regions = regionStore.regions
        for (const region of regions) {
          const regionTeams = allTeams
            .filter(team => team.regionId === region.id)
            .sort((a, b) => (b.statistics?.seasonPoints || 0) - (a.statistics?.seasonPoints || 0))
            .slice(0, Math.ceil((action.teamCount || 8) / regions.length))

          selectedTeams.push(...regionTeams)
        }
        break

      case 'playoff_winners':
        // 获取季后赛的前几名队伍
        const playoffWinners = getPlayoffWinners(triggerCompetition)
        selectedTeams = playoffWinners.slice(0, action.teamCount || 4)
        break

      case 'regional_champions':
        // 从指定赛区选择冠军队伍
        if (action.regionIds) {
          for (const regionId of action.regionIds) {
            const regionChampion = getRegionChampion(regionId)
            if (regionChampion) {
              selectedTeams.push(regionChampion)
            }
          }
        }
        break
    }

    return selectedTeams.slice(0, action.teamCount || selectedTeams.length)
  }

  // 获取季后赛获胜者
  const getPlayoffWinners = (playoffCompetition: Competition): Team[] => {
    // 简化逻辑：返回参赛队伍的前几名
    return playoffCompetition.teams
      .sort((a, b) => (b.statistics?.seasonPoints || 0) - (a.statistics?.seasonPoints || 0))
      .slice(0, 4)
  }

  // 获取赛区冠军
  const getRegionChampion = (regionId: string): Team | null => {
    // 查找该赛区最近完成的季后赛冠军
    const regionPlayoffs = eventStore.competitions.filter(comp =>
      comp.format === 'playoffs' &&
      comp.status === 'completed' &&
      comp.teams.some(team => team.regionId === regionId)
    )

    if (regionPlayoffs.length === 0) {
      // 如果没有季后赛，返回该赛区积分最高的队伍
      const regionTeams = teamStore.teams
        .filter(team => team.regionId === regionId)
        .sort((a, b) => (b.statistics?.seasonPoints || 0) - (a.statistics?.seasonPoints || 0))

      return regionTeams[0] || null
    }

    // 返回最新季后赛的冠军（简化为第一支队伍）
    const latestPlayoff = regionPlayoffs.sort((a, b) =>
      new Date(b.startDate || '').getTime() - new Date(a.startDate || '').getTime()
    )[0]

    return latestPlayoff.teams[0] || null
  }

  // 获取当前赛季ID
  const getCurrentSeasonId = (): string => {
    const currentSeason = eventStore.currentSeason
    return currentSeason?.id?.toString() || '1'
  }

  // 添加日志
  const log = (message: string) => {
    const timestamp = new Date().toLocaleTimeString()
    processingLog.value.push(`[${timestamp}] ${message}`)
    logger.debug(message)
  }

  // 手动触发检查
  const manualCheck = async () => {
    log('手动触发自动赛事检查')

    const completedCompetitions = eventStore.competitions.filter(comp =>
      comp.status === 'completed'
    )

    for (const competition of completedCompetitions) {
      await checkAutoCreation(competition)
    }
  }

  // 更新规则
  const updateRule = (ruleId: string, updates: Partial<AutoCreationRule>) => {
    const index = rules.value.findIndex(rule => rule.id === ruleId)
    if (index !== -1) {
      rules.value[index] = { ...rules.value[index], ...updates }
    }
  }

  // 添加自定义规则
  const addCustomRule = (rule: AutoCreationRule) => {
    rules.value.push(rule)
  }

  // 删除规则
  const removeRule = (ruleId: string) => {
    const index = rules.value.findIndex(rule => rule.id === ruleId)
    if (index !== -1) {
      rules.value.splice(index, 1)
    }
  }

  // 清空日志
  const clearLog = () => {
    processingLog.value = []
  }

  return {
    // 状态
    rules,
    isProcessing,
    processingLog,

    // 方法
    initializeDefaultRules,
    checkAutoCreation,
    manualCheck,
    updateRule,
    addCustomRule,
    removeRule,
    clearLog
  }
})