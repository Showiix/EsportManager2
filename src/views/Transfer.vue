<template>
  <div class="transfer-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>转会中心</h1>
        <p>{{ currentSeason }} 赛季 - 市场分析</p>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-select v-model="analysisFilters.region" placeholder="全部赛区" clearable style="width: 120px">
        <el-option label="全部赛区" value="" />
        <el-option label="LPL" value="LPL" />
        <el-option label="LCK" value="LCK" />
        <el-option label="LEC" value="LEC" />
        <el-option label="LCS" value="LCS" />
      </el-select>
      <el-select v-model="analysisFilters.strategy" placeholder="全部策略" clearable style="width: 140px">
        <el-option label="全部策略" value="" />
        <el-option label="积极买人" value="AggressiveBuy" />
        <el-option label="观望" value="Passive" />
        <el-option label="必须卖人" value="MustSell" />
        <el-option label="强制清洗" value="ForceClear" />
        <el-option label="全面重建" value="FullRebuild" />
        <el-option label="追逐巨星" value="StarHunting" />
      </el-select>
      <el-input
        v-model="analysisFilters.search"
        placeholder="搜索球队..."
        style="width: 200px"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      <el-button @click="loadTeamPlans" :loading="isLoadingPlans">
        <el-icon><Refresh /></el-icon>
        刷新数据
      </el-button>
    </div>

    <!-- 球队转会计划表格 -->
    <el-card class="analysis-table-card">
      <el-table
        :data="filteredTeamPlans"
        v-loading="isLoadingPlans"
        stripe
        style="width: 100%"
        max-height="600"
      >
        <el-table-column prop="team_name" label="球队" width="120" fixed>
          <template #default="{ row }">
            <div class="team-cell">
              <span class="team-name">{{ row.team_name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="赛区" width="70" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="getRegionTagType(row.region_code)">{{ formatRegion(row.region_code) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="strategy" label="策略" width="90">
          <template #default="{ row }">
            <el-tag :type="getStrategyTagType(row.strategy)" effect="dark" size="small">
              {{ getStrategyLabel(row.strategy) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="ambition" label="野心" width="80">
          <template #default="{ row }">
            <span :class="'ambition-' + row.ambition.toLowerCase()">{{ getAmbitionLabel(row.ambition) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="financial_status" label="财务" width="70">
          <template #default="{ row }">
            <el-tag :type="getFinancialTagType(row.financial_status)" size="small">
              {{ getFinancialLabel(row.financial_status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="transfer_budget" label="预算" width="90" align="right">
          <template #default="{ row }">
            <span class="budget-value">{{ formatBudget(row.transfer_budget) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="salary_space" label="薪资空间" width="100" align="right">
          <template #default="{ row }">
            <span :class="row.salary_space > 0 ? 'positive' : 'negative'">
              {{ formatBudget(row.salary_space) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="roster_count" label="人数" width="65" align="center">
          <template #default="{ row }">
            <span :class="{ 'roster-warning': row.roster_count < 5 || row.roster_count > 10 }">
              {{ row.roster_count }}/10
            </span>
          </template>
        </el-table-column>
        <el-table-column label="TOP" min-width="45" align="center">
          <template #default="{ row }">
            <span :class="getNeedClass(row.position_needs?.TOP)">{{ row.position_needs?.TOP || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column label="JUG" min-width="45" align="center">
          <template #default="{ row }">
            <span :class="getNeedClass(row.position_needs?.JUG)">{{ row.position_needs?.JUG || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column label="MID" min-width="45" align="center">
          <template #default="{ row }">
            <span :class="getNeedClass(row.position_needs?.MID)">{{ row.position_needs?.MID || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column label="ADC" min-width="45" align="center">
          <template #default="{ row }">
            <span :class="getNeedClass(row.position_needs?.ADC)">{{ row.position_needs?.ADC || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column label="SUP" min-width="45" align="center">
          <template #default="{ row }">
            <span :class="getNeedClass(row.position_needs?.SUP)">{{ row.position_needs?.SUP || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="avg_ability" label="均能力" width="75" align="center">
          <template #default="{ row }">
            <span :style="{ color: getAbilityColor(row.avg_ability) }">
              {{ row.avg_ability.toFixed(1) }}
            </span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 说明 -->
      <div class="table-legend">
        <span>位置需求: </span>
        <span class="legend-item need-urgent">100=急需</span>
        <span class="legend-item need-need">70=需要</span>
        <span class="legend-item need-consider">30=可考虑</span>
        <span class="legend-item need-none">0=不需要</span>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, reactive } from 'vue'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Search,
  Refresh,
} from '@element-plus/icons-vue'
import { useGameStore } from '@/stores/useGameStore'
import { transferApi, type TeamTransferPlanInfo } from '@/api/tauri'

const gameStore = useGameStore()

// 市场分析数据
const teamPlans = ref<TeamTransferPlanInfo[]>([])
const isLoadingPlans = ref(false)
const analysisFilters = reactive({
  region: '',
  strategy: '',
  search: '',
})

// 加载球队转会计划
const loadTeamPlans = async () => {
  isLoadingPlans.value = true
  try {
    teamPlans.value = await transferApi.getTeamTransferPlans()
  } catch (e) {
    console.error('Failed to load team plans:', e)
    ElMessage.error('加载球队转会计划失败')
  } finally {
    isLoadingPlans.value = false
  }
}

// 过滤后的球队计划
const filteredTeamPlans = computed(() => {
  return teamPlans.value.filter(plan => {
    // 筛选时需要转换 region_code
    const planRegion = formatRegion(plan.region_code)
    if (analysisFilters.region && planRegion !== analysisFilters.region) return false
    if (analysisFilters.strategy && plan.strategy !== analysisFilters.strategy) return false
    if (analysisFilters.search && !plan.team_name.toLowerCase().includes(analysisFilters.search.toLowerCase())) return false
    return true
  })
})

// 从 store 获取响应式数据
const { currentSeason } = storeToRefs(gameStore)

// 初始化加载数据
onMounted(async () => {
  try {
    await gameStore.refreshGameState()
    await loadTeamPlans()
  } catch (e) {
    console.error('Failed to load data:', e)
    ElMessage.error('加载数据失败')
  }
})

// ======== 辅助函数 ========

// 格式化赛区显示（CN -> LPL, KR -> LCK 等）
const formatRegion = (region: string) => {
  const regionMap: Record<string, string> = {
    'CN': 'LPL',
    'KR': 'LCK',
    'EU': 'LEC',
    'NA': 'LCS',
    'LPL': 'LPL',
    'LCK': 'LCK',
    'LEC': 'LEC',
    'LCS': 'LCS',
  }
  return regionMap[region] || region
}

const getRegionTagType = (region: string) => {
  const formatted = formatRegion(region)
  const types: Record<string, string> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning',
  }
  return types[formatted] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getStrategyTagType = (strategy: string) => {
  const types: Record<string, string> = {
    'AggressiveBuy': 'success',
    'Passive': 'info',
    'MustSell': 'warning',
    'ForceClear': 'danger',
    'FullRebuild': 'danger',
    'StarHunting': '',
  }
  return types[strategy] || 'info'
}

const getStrategyLabel = (strategy: string) => {
  const labels: Record<string, string> = {
    'AggressiveBuy': '积极买',
    'Passive': '观望',
    'MustSell': '必须卖',
    'ForceClear': '清洗',
    'FullRebuild': '重建',
    'StarHunting': '追星',
  }
  return labels[strategy] || strategy
}

const getAmbitionLabel = (ambition: string) => {
  const labels: Record<string, string> = {
    'Championship': '争冠',
    'Playoff': '季后赛',
    'Rebuild': '重建',
  }
  return labels[ambition] || ambition
}

const getFinancialTagType = (status: string) => {
  const types: Record<string, string> = {
    'Wealthy': 'success',
    'Healthy': 'primary',
    'Struggling': 'warning',
    'Bankrupt': 'danger',
  }
  return types[status] || 'info'
}

const getFinancialLabel = (status: string) => {
  const labels: Record<string, string> = {
    'Wealthy': '富裕',
    'Healthy': '健康',
    'Struggling': '紧张',
    'Bankrupt': '破产',
  }
  return labels[status] || status
}

const formatBudget = (value: number) => {
  // 后端已转换为万，直接使用
  if (value >= 10000) return `${(value / 10000).toFixed(1)}亿`
  if (value >= 1) return `${Math.round(value)}万`
  return `${value}万`
}

const getNeedClass = (need: number | undefined) => {
  if (need === undefined) return 'need-none'
  if (need >= 100) return 'need-urgent'
  if (need >= 70) return 'need-need'
  if (need >= 30) return 'need-consider'
  return 'need-none'
}
</script>

<style scoped>
.transfer-view {
  padding: 0;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

/* 筛选栏 */
.filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

/* 表格卡片 */
.analysis-table-card {
  border-radius: 12px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-name {
  font-weight: 600;
  color: #303133;
}

.budget-value {
  font-weight: 600;
  color: #409eff;
}

.positive {
  color: #67c23a;
  font-weight: 600;
}

.negative {
  color: #f56c6c;
  font-weight: 600;
}

.roster-warning {
  color: #f56c6c;
  font-weight: 600;
}

/* 野心等级样式 */
.ambition-championship {
  color: #e6a23c;
  font-weight: 700;
}

.ambition-playoff {
  color: #409eff;
  font-weight: 600;
}

.ambition-rebuild {
  color: #909399;
}

/* 位置需求样式 */
.need-urgent {
  color: #f56c6c;
  font-weight: 700;
  white-space: nowrap;
  display: inline-block;
}

.need-need {
  color: #e6a23c;
  font-weight: 600;
  white-space: nowrap;
  display: inline-block;
}

.need-consider {
  color: #409eff;
  white-space: nowrap;
  display: inline-block;
}

.need-none {
  color: #c0c4cc;
  white-space: nowrap;
  display: inline-block;
}

/* 表格列头不换行 */
.analysis-table-card :deep(.el-table__header th) {
  white-space: nowrap;
}

.analysis-table-card :deep(.el-table__cell) {
  padding: 8px 4px;
}

/* 图例 */
.table-legend {
  margin-top: 12px;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 6px;
  font-size: 12px;
  color: #606266;
}

.legend-item {
  margin-left: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}

.legend-item.need-urgent {
  background: #fef0f0;
  color: #f56c6c;
}

.legend-item.need-need {
  background: #fdf6ec;
  color: #e6a23c;
}

.legend-item.need-consider {
  background: #ecf5ff;
  color: #409eff;
}

.legend-item.need-none {
  background: #f5f7fa;
  color: #c0c4cc;
}
</style>
