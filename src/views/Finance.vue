<template>
  <div class="finance-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>财政中心</h1>
        <p>查看所有战队的财务状况</p>
      </div>
      <el-button
        type="primary"
        :icon="Refresh"
        :loading="loading"
        @click="handleRefresh"
      >
        刷新数据
      </el-button>
    </div>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon gold">
              <el-icon :size="28"><Coin /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatMoney(stats.totalAssets) }}</div>
              <div class="stat-label">总资产</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="28"><Top /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatMoney(stats.totalIncome) }}</div>
              <div class="stat-label">本赛季收入</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon red">
              <el-icon :size="28"><Bottom /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatMoney(stats.totalExpense) }}</div>
              <div class="stat-label">本赛季支出</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" :class="stats.netProfit >= 0 ? 'blue' : 'orange'">
              <el-icon :size="28"><TrendCharts /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number" :class="{ negative: stats.netProfit < 0 }">
                {{ stats.netProfit >= 0 ? '+' : '' }}{{ formatMoney(stats.netProfit) }}
              </div>
              <div class="stat-label">净利润</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 财务状态分布 -->
    <el-card class="status-distribution-card">
      <div class="status-distribution">
        <div class="status-item wealthy">
          <span class="status-dot dot-wealthy"></span>
          <span class="status-label">富裕</span>
          <span class="status-count">{{ stats.wealthyCount }}</span>
        </div>
        <div class="status-item healthy">
          <span class="status-dot dot-healthy"></span>
          <span class="status-label">健康</span>
          <span class="status-count">{{ stats.healthyCount }}</span>
        </div>
        <div class="status-item tight">
          <span class="status-dot dot-tight"></span>
          <span class="status-label">紧张</span>
          <span class="status-count">{{ stats.tightCount }}</span>
        </div>
        <div class="status-item deficit">
          <span class="status-dot dot-deficit"></span>
          <span class="status-label">赤字</span>
          <span class="status-count">{{ stats.deficitCount }}</span>
        </div>
        <div class="status-item bankrupt">
          <span class="status-dot dot-bankrupt"></span>
          <span class="status-label">破产</span>
          <span class="status-count">{{ stats.bankruptCount }}</span>
        </div>
      </div>
    </el-card>

    <!-- 战队财务列表 -->
    <el-card class="finance-card">
      <!-- 筛选和搜索 -->
      <div class="filter-row">
        <div class="filter-left">
          <el-radio-group v-model="selectedRegion" @change="handleRegionChange">
            <el-radio-button value="">全部赛区</el-radio-button>
            <el-radio-button value="CN">LPL</el-radio-button>
            <el-radio-button value="KR">LCK</el-radio-button>
            <el-radio-button value="EU">LEC</el-radio-button>
            <el-radio-button value="NA">LCS</el-radio-button>
          </el-radio-group>
        </div>
        <div class="filter-right">
          <el-input
            v-model="searchQuery"
            placeholder="搜索战队..."
            clearable
            style="width: 200px"
            @input="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
          <el-select v-model="sortOption" placeholder="排序方式" style="width: 140px" @change="handleSortChange">
            <el-option label="按余额" value="balance" />
            <el-option label="按收入" value="income" />
            <el-option label="按支出" value="expense" />
            <el-option label="按状态" value="status" />
          </el-select>
        </div>
      </div>

      <!-- 加载状态 -->
      <el-skeleton v-if="loading" :rows="10" animated />

      <!-- 战队列表表格 -->
      <el-table
        v-else
        :data="filteredTeams"
        stripe
        class="finance-table"
        @row-click="handleRowClick"
      >
        <el-table-column type="index" label="#" width="60" align="center" />

        <el-table-column prop="team_name" label="战队" min-width="180">
          <template #default="{ row }">
            <div class="team-cell">
              <div class="team-avatar medium" :class="getLeagueName(row.region_code).toLowerCase()">
                {{ row.short_name || row.team_name.substring(0, 2) }}
              </div>
              <div class="team-info">
                <div class="team-name">{{ row.team_name }}</div>
                <el-tag :type="getRegionTagType(getLeagueName(row.region_code))" size="small">
                  {{ getLeagueName(row.region_code) }}
                </el-tag>
              </div>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="balance" label="余额" width="140" align="right" sortable>
          <template #default="{ row }">
            <span class="money-value" :class="{ negative: row.balance < 0 }">
              {{ formatMoney(row.balance) }}
            </span>
          </template>
        </el-table-column>

        <el-table-column prop="total_income" label="收入" width="120" align="right">
          <template #default="{ row }">
            <span class="money-income">+{{ formatMoney(row.total_income) }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="total_expense" label="支出" width="120" align="right">
          <template #default="{ row }">
            <span class="money-expense">-{{ formatMoney(row.total_expense) }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="financial_status" label="状态" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getStatusTagType(row.financial_status)" effect="dark">
              {{ getStatusLabel(row.financial_status) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="100" align="center">
          <template #default="{ row }">
            <el-button
              type="primary"
              size="small"
              round
              class="detail-btn"
              @click.stop="openDetail(row)"
            >
              <el-icon class="mr-1"><View /></el-icon>
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 详情弹窗 -->
    <TeamFinanceDialog
      v-model="showDetailDialog"
      :team="selectedTeam"
      @close="showDetailDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Coin,
  Top,
  Bottom,
  TrendCharts,
  Search,
  View,
  Refresh,
} from '@element-plus/icons-vue'
import { useFinanceStore, type FinancialStatus } from '@/stores/useFinanceStore'
import type { TeamFinanceSummary } from '@/api/tauri'
import TeamFinanceDialog from '@/components/finance/TeamFinanceDialog.vue'

const financeStore = useFinanceStore()
const { loading, filteredTeams, stats } = storeToRefs(financeStore)

// 筛选状态
const selectedRegion = ref<string>('')
const searchQuery = ref('')
const sortOption = ref('balance')

// 弹窗状态
const showDetailDialog = ref(false)
const selectedTeam = ref<TeamFinanceSummary | null>(null)

// 初始化
onMounted(async () => {
  try {
    await financeStore.fetchAllTeamsFinance()
  } catch (e: any) {
    ElMessage.error('加载财务数据失败: ' + e.message)
  }
})

// 格式化金额
function formatMoney(amount: number): string {
  return financeStore.formatMoney(amount)
}

// 获取联赛名称（CN -> LPL）
function getLeagueName(regionCode: string): string {
  return financeStore.getLeagueName(regionCode)
}

// 赛区筛选
function handleRegionChange(regionCode: string) {
  financeStore.setFilter(regionCode === '' ? null : regionCode)
}

// 搜索
function handleSearch(query: string) {
  financeStore.setSearchQuery(query)
}

// 排序
function handleSortChange(sortBy: string) {
  financeStore.setSort(sortBy as any, 'desc')
}

// 获取赛区标签类型
function getRegionTagType(region: string): string {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

// 获取状态标签类型
function getStatusTagType(status: FinancialStatus): string {
  const types: Record<string, string> = {
    Wealthy: 'success',
    Healthy: '',
    Tight: 'warning',
    Deficit: 'warning',
    Bankrupt: 'danger',
  }
  return types[status] || 'info'
}

// 获取状态标签
function getStatusLabel(status: FinancialStatus): string {
  const labels: Record<string, string> = {
    Wealthy: '富裕',
    Healthy: '健康',
    Tight: '紧张',
    Deficit: '赤字',
    Bankrupt: '破产',
  }
  return labels[status] || status
}

// 行点击
function handleRowClick(row: TeamFinanceSummary) {
  openDetail(row)
}

// 打开详情弹窗
function openDetail(team: TeamFinanceSummary) {
  selectedTeam.value = team
  showDetailDialog.value = true
}

// 手动刷新数据
async function handleRefresh() {
  try {
    await financeStore.fetchAllTeamsFinance()
    ElMessage.success('财务数据已刷新')
  } catch (e: any) {
    ElMessage.error('刷新失败: ' + e.message)
  }
}
</script>

<style scoped>
.finance-view {
  padding: 0;
}

/* 页面标题 */
.page-header {
  margin-bottom: 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-header h1 {
  font-size: 26px;
  font-weight: 800;
  color: var(--text-primary, #1d2129);
  margin: 0 0 6px 0;
  letter-spacing: -0.5px;
}

.page-header p {
  font-size: 14px;
  color: var(--text-tertiary, #86909c);
  margin: 0;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 14px;
  border: 1px solid rgba(0, 0, 0, 0.04);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
  transition: box-shadow 0.25s ease, transform 0.25s ease;
}

.stat-card:hover {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.06), 0 8px 24px rgba(0, 0, 0, 0.06);
  transform: translateY(-2px);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 4px 0;
}

.stat-icon {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.stat-icon.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3); }
.stat-icon.green { background: linear-gradient(135deg, #34d399, #10b981); box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3); }
.stat-icon.red { background: linear-gradient(135deg, #f87171, #ef4444); box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3); }
.stat-icon.blue { background: linear-gradient(135deg, #60a5fa, #3b82f6); box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3); }
.stat-icon.orange { background: linear-gradient(135deg, #fb923c, #f97316); box-shadow: 0 4px 12px rgba(249, 115, 22, 0.3); }
.stat-icon.purple { background: linear-gradient(135deg, #a78bfa, #8b5cf6); box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3); }

.stat-info {
  flex: 1;
  min-width: 0;
}

.stat-number {
  font-size: 22px;
  font-weight: 800;
  color: var(--text-primary, #1d2129);
  letter-spacing: -0.5px;
  line-height: 1.2;
}

.stat-number.negative {
  color: #ef4444;
}

.stat-label {
  font-size: 13px;
  color: var(--text-tertiary, #86909c);
  margin-top: 4px;
  font-weight: 500;
}

/* 状态分布 */
.status-distribution-card {
  margin-bottom: 20px;
  border-radius: 14px;
  border: 1px solid rgba(0, 0, 0, 0.04);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
}

.status-distribution {
  display: flex;
  justify-content: space-around;
  align-items: center;
  padding: 6px 0;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 20px;
  border-radius: 10px;
  background: #f7f8fa;
  transition: background 0.2s ease;
}

.status-item:hover {
  background: #f0f1f3;
}

.status-dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.04);
}

.status-dot.dot-wealthy { background-color: #10b981; box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.15); }
.status-dot.dot-healthy { background-color: #3b82f6; box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.15); }
.status-dot.dot-tight { background-color: #f59e0b; box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.15); }
.status-dot.dot-deficit { background-color: #f97316; box-shadow: 0 0 0 3px rgba(249, 115, 22, 0.15); }
.status-dot.dot-bankrupt { background-color: #ef4444; box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15); }

.status-label {
  font-size: 14px;
  color: #4e5969;
  font-weight: 500;
}

.status-count {
  font-size: 20px;
  font-weight: 800;
  color: #1d2129;
  min-width: 20px;
  text-align: center;
}

/* 财务卡片 */
.finance-card {
  border-radius: 14px;
  border: 1px solid rgba(0, 0, 0, 0.04);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
}

.filter-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 14px 16px;
  background: #f7f8fa;
  border-radius: 10px;
  flex-wrap: wrap;
  gap: 12px;
}

.filter-left {
  display: flex;
  gap: 12px;
}

.filter-right {
  display: flex;
  gap: 10px;
}

/* 表格 */
.finance-table {
  width: 100%;
}

.finance-table :deep(.el-table__header th) {
  font-weight: 600;
  color: #86909c;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  background: #fafbfc;
}

.finance-table :deep(.el-table__row) {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.finance-table :deep(.el-table__row td) {
  padding: 14px 0;
}

.finance-table :deep(.el-table__row:hover > td) {
  background-color: #f0f7ff !important;
}

/* 战队单元格 */
.team-cell {
  display: flex;
  align-items: center;
  gap: 12px;
}

.team-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 5px;
}

.team-name {
  font-size: 14px;
  font-weight: 600;
  color: #1d2129;
}

/* 金额样式 */
.money-value {
  font-size: 14px;
  font-weight: 700;
  color: #1d2129;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.2px;
}

.money-value.negative {
  color: #ef4444;
}

.money-income {
  color: #10b981;
  font-weight: 600;
  font-size: 14px;
  font-variant-numeric: tabular-nums;
}

.money-expense {
  color: #ef4444;
  font-weight: 600;
  font-size: 14px;
  font-variant-numeric: tabular-nums;
}

/* 详情按钮 */
.detail-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  font-size: 12px;
  font-weight: 600;
  padding: 6px 14px;
  transition: all 0.2s ease;
  box-shadow: 0 2px 6px rgba(102, 126, 234, 0.3);
}

.detail-btn:hover {
  background: linear-gradient(135deg, #5a6fd6 0%, #6a4190 100%);
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(102, 126, 234, 0.45);
}

.detail-btn .mr-1 {
  margin-right: 4px;
}

/* 响应式 */
@media (max-width: 768px) {
  .filter-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .filter-left,
  .filter-right {
    width: 100%;
  }

  .status-distribution {
    flex-wrap: wrap;
    gap: 8px;
  }
}
</style>
