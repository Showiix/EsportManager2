// =================================================================
// 电竞赛事模拟系统 - 财政管理Store
// =================================================================

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { financeApi, type TeamFinanceSummary, type FinanceTransaction, type SeasonFinanceReport, type TournamentPrizeDetail } from '@/api/tauri'

// ========================================
// 类型定义
// ========================================

export type FinancialStatus = 'Wealthy' | 'Healthy' | 'Tight' | 'Deficit' | 'Bankrupt'

export interface FinanceStats {
  totalAssets: number
  totalIncome: number
  totalExpense: number
  netProfit: number
  wealthyCount: number
  healthyCount: number
  tightCount: number
  deficitCount: number
  bankruptCount: number
}

// ========================================
// Store定义
// ========================================

export const useFinanceStore = defineStore('finance', () => {
  // ========================================
  // 状态
  // ========================================

  const loading = ref(false)
  const error = ref<string | null>(null)

  // 所有战队财务摘要
  const teamsFinance = ref<TeamFinanceSummary[]>([])

  // 当前选中的战队财务详情
  const selectedTeamFinance = ref<TeamFinanceSummary | null>(null)

  // 战队交易记录
  const teamTransactions = ref<Map<number, FinanceTransaction[]>>(new Map())

  // 战队赛季财务报告
  const seasonReports = ref<Map<string, SeasonFinanceReport>>(new Map())

  // 战队赛事奖金明细
  const teamPrizeDetails = ref<Map<number, TournamentPrizeDetail[]>>(new Map())

  // 筛选条件
  const filterRegionId = ref<number | null>(null)
  const sortBy = ref<'balance' | 'income' | 'expense' | 'status'>('balance')
  const sortOrder = ref<'asc' | 'desc'>('desc')
  const searchQuery = ref('')

  // ========================================
  // 计算属性
  // ========================================

  const isLoading = computed(() => loading.value)
  const hasError = computed(() => error.value !== null)

  // 财务统计
  const stats = computed<FinanceStats>(() => {
    const teams = teamsFinance.value
    return {
      totalAssets: teams.reduce((sum, t) => sum + t.balance, 0),
      totalIncome: teams.reduce((sum, t) => sum + t.total_income, 0),
      totalExpense: teams.reduce((sum, t) => sum + t.total_expense, 0),
      netProfit: teams.reduce((sum, t) => sum + (t.total_income - t.total_expense), 0),
      wealthyCount: teams.filter(t => t.financial_status === 'Wealthy').length,
      healthyCount: teams.filter(t => t.financial_status === 'Healthy').length,
      tightCount: teams.filter(t => t.financial_status === 'Tight').length,
      deficitCount: teams.filter(t => t.financial_status === 'Deficit').length,
      bankruptCount: teams.filter(t => t.financial_status === 'Bankrupt').length,
    }
  })

  // 筛选和排序后的战队列表
  const filteredTeams = computed(() => {
    let result = [...teamsFinance.value]

    // 赛区筛选
    if (filterRegionId.value !== null) {
      result = result.filter(t => t.region_id === filterRegionId.value)
    }

    // 搜索筛选
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      result = result.filter(t =>
        t.team_name.toLowerCase().includes(query) ||
        (t.short_name && t.short_name.toLowerCase().includes(query))
      )
    }

    // 排序
    result.sort((a, b) => {
      let compareValue = 0
      switch (sortBy.value) {
        case 'balance':
          compareValue = a.balance - b.balance
          break
        case 'income':
          compareValue = a.total_income - b.total_income
          break
        case 'expense':
          compareValue = a.total_expense - b.total_expense
          break
        case 'status':
          const statusOrder = { 'Wealthy': 5, 'Healthy': 4, 'Tight': 3, 'Deficit': 2, 'Bankrupt': 1 }
          compareValue = statusOrder[a.financial_status] - statusOrder[b.financial_status]
          break
      }
      return sortOrder.value === 'desc' ? -compareValue : compareValue
    })

    return result
  })

  // 按赛区分组的战队
  const teamsByRegion = computed(() => {
    const map = new Map<string, TeamFinanceSummary[]>()
    for (const team of teamsFinance.value) {
      const code = team.region_code || 'Unknown'
      if (!map.has(code)) {
        map.set(code, [])
      }
      map.get(code)!.push(team)
    }
    return map
  })

  // ========================================
  // Actions
  // ========================================

  /**
   * 获取所有战队财务状况
   */
  async function fetchAllTeamsFinance(regionId?: number): Promise<void> {
    loading.value = true
    error.value = null

    try {
      const data = await financeApi.getAllTeamsFinance(regionId)
      teamsFinance.value = data
      console.log('✅ 战队财务数据加载成功', {
        teamsCount: teamsFinance.value.length,
        regionId
      })
    } catch (err: any) {
      error.value = err.message || '获取战队财务数据失败'
      console.error('❌ 获取战队财务数据失败', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取单个战队财务摘要
   */
  async function fetchTeamFinanceSummary(teamId: number): Promise<TeamFinanceSummary> {
    loading.value = true
    error.value = null

    try {
      const data = await financeApi.getTeamFinanceSummary(teamId)
      selectedTeamFinance.value = data
      console.log('✅ 战队财务摘要加载成功', {
        teamId,
        teamName: data.team_name
      })
      return data
    } catch (err: any) {
      error.value = err.message || '获取战队财务摘要失败'
      console.error('❌ 获取战队财务摘要失败', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取战队交易记录
   */
  async function fetchTeamTransactions(teamId: number, seasonId?: number): Promise<FinanceTransaction[]> {
    loading.value = true
    error.value = null

    try {
      const data = await financeApi.getTeamTransactions(teamId, seasonId)
      teamTransactions.value.set(teamId, data)
      console.log('✅ 战队交易记录加载成功', {
        teamId,
        transactionsCount: data.length
      })
      return data
    } catch (err: any) {
      error.value = err.message || '获取战队交易记录失败'
      console.error('❌ 获取战队交易记录失败', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取战队赛季财务报告
   */
  async function fetchSeasonReport(teamId: number, seasonId?: number): Promise<SeasonFinanceReport> {
    loading.value = true
    error.value = null

    try {
      const data = await financeApi.getSeasonFinanceReport(teamId, seasonId)
      const key = `${teamId}-${seasonId || 'current'}`
      seasonReports.value.set(key, data)
      console.log('✅ 赛季财务报告加载成功', {
        teamId,
        seasonId
      })
      return data
    } catch (err: any) {
      error.value = err.message || '获取赛季财务报告失败'
      console.error('❌ 获取赛季财务报告失败', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取战队赛事奖金明细
   */
  async function fetchTeamPrizeDetails(teamId: number, seasonId?: number): Promise<TournamentPrizeDetail[]> {
    loading.value = true
    error.value = null

    try {
      const data = await financeApi.getTeamPrizeDetails(teamId, seasonId)
      teamPrizeDetails.value.set(teamId, data)
      console.log('✅ 战队赛事奖金明细加载成功', {
        teamId,
        prizeCount: data.length
      })
      return data
    } catch (err: any) {
      error.value = err.message || '获取战队赛事奖金明细失败'
      console.error('❌ 获取战队赛事奖金明细失败', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取战队赛事奖金明细（从缓存）
   */
  function getTeamPrizeDetails(teamId: number): TournamentPrizeDetail[] | undefined {
    return teamPrizeDetails.value.get(teamId)
  }

  /**
   * 设置筛选条件
   */
  function setFilter(regionId: number | null): void {
    filterRegionId.value = regionId
  }

  /**
   * 设置排序
   */
  function setSort(by: typeof sortBy.value, order: typeof sortOrder.value): void {
    sortBy.value = by
    sortOrder.value = order
  }

  /**
   * 设置搜索关键词
   */
  function setSearchQuery(query: string): void {
    searchQuery.value = query
  }

  /**
   * 获取战队交易记录（从缓存）
   */
  function getTeamTransactions(teamId: number): FinanceTransaction[] | undefined {
    return teamTransactions.value.get(teamId)
  }

  /**
   * 获取赛季财务报告（从缓存）
   */
  function getSeasonReport(teamId: number, seasonId?: number): SeasonFinanceReport | undefined {
    const key = `${teamId}-${seasonId || 'current'}`
    return seasonReports.value.get(key)
  }

  /**
   * 获取财务状态的显示信息
   */
  function getStatusInfo(status: FinancialStatus): { label: string; color: string; icon: string } {
    switch (status) {
      case 'Wealthy':
        return { label: '富裕', color: '#10b981', icon: '🟢' }
      case 'Healthy':
        return { label: '健康', color: '#3b82f6', icon: '🔵' }
      case 'Tight':
        return { label: '紧张', color: '#f59e0b', icon: '🟡' }
      case 'Deficit':
        return { label: '赤字', color: '#f97316', icon: '🟠' }
      case 'Bankrupt':
        return { label: '破产', color: '#ef4444', icon: '🔴' }
      default:
        return { label: '未知', color: '#6b7280', icon: '⚪' }
    }
  }

  /**
   * 格式化金额（输入单位：元）
   * 数据库存储的是实际金额（元），需要转换为合适的显示单位
   */
  function formatMoney(amount: number): string {
    const absAmount = Math.abs(amount)
    const sign = amount < 0 ? '-' : ''

    if (absAmount >= 100000000) {
      // 1亿及以上
      return `${sign}${(absAmount / 100000000).toFixed(2)}亿`
    } else if (absAmount >= 10000000) {
      // 1千万及以上
      return `${sign}${(absAmount / 10000000).toFixed(1)}千万`
    } else if (absAmount >= 10000) {
      // 1万及以上
      return `${sign}${(absAmount / 10000).toFixed(0)}万`
    } else {
      // 1万以下
      return `${sign}${absAmount.toFixed(0)}元`
    }
  }

  /**
   * 将 region_code 转换为联赛名称
   */
  function getLeagueName(regionCode: string): string {
    const leagueMap: Record<string, string> = {
      'CN': 'LPL',
      'KR': 'LCK',
      'EU': 'LEC',
      'NA': 'LCS',
    }
    return leagueMap[regionCode] || regionCode
  }

  /**
   * 清空缓存
   */
  function clearCache(): void {
    teamsFinance.value = []
    selectedTeamFinance.value = null
    teamTransactions.value.clear()
    seasonReports.value.clear()
    error.value = null
    console.log('🗑️ 财政Store缓存已清空')
  }

  /**
   * 清空错误
   */
  function clearError(): void {
    error.value = null
  }

  return {
    // 状态
    loading,
    error,
    teamsFinance,
    selectedTeamFinance,
    teamTransactions,
    seasonReports,
    teamPrizeDetails,
    filterRegionId,
    sortBy,
    sortOrder,
    searchQuery,

    // 计算属性
    isLoading,
    hasError,
    stats,
    filteredTeams,
    teamsByRegion,

    // Actions
    fetchAllTeamsFinance,
    fetchTeamFinanceSummary,
    fetchTeamTransactions,
    fetchSeasonReport,
    fetchTeamPrizeDetails,
    setFilter,
    setSort,
    setSearchQuery,
    getTeamTransactions,
    getSeasonReport,
    getTeamPrizeDetails,
    getStatusInfo,
    formatMoney,
    getLeagueName,
    clearCache,
    clearError
  }
})
