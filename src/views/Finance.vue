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
    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ formatMoney(stats.totalAssets) }}</span>
        <span class="stat-label">总资产</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value income">+{{ formatMoney(stats.totalIncome) }}</span>
        <span class="stat-label">收入</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value expense">-{{ formatMoney(stats.totalExpense) }}</span>
        <span class="stat-label">支出</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value" :class="stats.netProfit >= 0 ? 'income' : 'expense'">
          {{ stats.netProfit >= 0 ? '+' : '' }}{{ formatMoney(stats.netProfit) }}
        </span>
        <span class="stat-label">净利润</span>
      </div>
    </div>

    <!-- 财务状态分布 -->
    <div class="status-bar">
      <div class="status-item">
        <span class="status-dot wealthy"></span>
        <span class="status-name">富裕</span>
        <span class="status-count">{{ stats.wealthyCount }}</span>
      </div>
      <div class="status-item">
        <span class="status-dot healthy"></span>
        <span class="status-name">健康</span>
        <span class="status-count">{{ stats.healthyCount }}</span>
      </div>
      <div class="status-item">
        <span class="status-dot tight"></span>
        <span class="status-name">紧张</span>
        <span class="status-count">{{ stats.tightCount }}</span>
      </div>
      <div class="status-item">
        <span class="status-dot deficit"></span>
        <span class="status-name">赤字</span>
        <span class="status-count">{{ stats.deficitCount }}</span>
      </div>
      <div class="status-item">
        <span class="status-dot bankrupt"></span>
        <span class="status-name">破产</span>
        <span class="status-count">{{ stats.bankruptCount }}</span>
      </div>
    </div>

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

        <el-table-column label="" width="80" align="center">
          <template #default="{ row }">
            <button class="detail-btn" @click.stop="openDetail(row)">详情</button>
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
  Search,
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

/* ====== 页面标题 ====== */
.page-header {
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
  letter-spacing: -0.3px;
}

.page-header p {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
}

/* ====== 统计条 ====== */
.stats-bar {
  display: flex;
  align-items: center;
  padding: 14px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  margin-bottom: 12px;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  flex: 1;
  justify-content: center;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  font-variant-numeric: tabular-nums;
}

.stat-value.income {
  color: #10b981;
}

.stat-value.expense {
  color: #ef4444;
}

.stat-label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 500;
}

.stat-divider {
  width: 1px;
  height: 24px;
  background: #e2e8f0;
  flex-shrink: 0;
}

/* ====== 状态分布 ====== */
.status-bar {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 10px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  margin-bottom: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  justify-content: center;
  padding: 4px 0;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.wealthy { background: #10b981; }
.status-dot.healthy { background: #3b82f6; }
.status-dot.tight { background: #f59e0b; }
.status-dot.deficit { background: #f97316; }
.status-dot.bankrupt { background: #ef4444; }

.status-name {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
}

.status-count {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  font-variant-numeric: tabular-nums;
}

/* ====== 财务主卡片 ====== */
.finance-card {
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  box-shadow: none;
}

/* ====== 筛选栏 ====== */
.filter-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 10px;
}

.filter-left {
  display: flex;
  gap: 10px;
}

.filter-right {
  display: flex;
  gap: 8px;
}

/* ====== 表格 ====== */
.finance-table {
  width: 100%;
}

.finance-table :deep(.el-table__header th) {
  font-weight: 600;
  color: #94a3b8;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: transparent;
  border-bottom: 1px solid #f1f5f9;
  padding: 10px 0;
}

.finance-table :deep(.el-table__body tr) {
  cursor: pointer;
  transition: background-color 0.15s;
}

.finance-table :deep(.el-table__body tr td) {
  padding: 12px 0;
  border-bottom: 1px solid #f8fafc;
}

.finance-table :deep(.el-table__body tr:hover > td) {
  background-color: #f8fafc !important;
}

.finance-table :deep(.el-table__body tr:last-child td) {
  border-bottom: none;
}

.finance-table :deep(.el-table__body tr.el-table__row--striped td) {
  background: transparent;
}

/* ====== 战队单元格 ====== */
.team-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.team-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 3px;
}

.team-name {
  font-size: 13px;
  font-weight: 600;
  color: #0f172a;
}

/* ====== 金额样式 ====== */
.money-value {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
  font-variant-numeric: tabular-nums;
}

.money-value.negative {
  color: #ef4444;
}

.money-income {
  color: #10b981;
  font-weight: 600;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
}

.money-expense {
  color: #ef4444;
  font-weight: 600;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
}

/* ====== 详情按钮 ====== */
.detail-btn {
  padding: 5px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #ffffff;
  color: #475569;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.detail-btn:hover {
  border-color: #6366f1;
  color: #6366f1;
  background: #f5f3ff;
}

/* ====== 响应式 ====== */
@media (max-width: 768px) {
  .filter-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .filter-left,
  .filter-right {
    width: 100%;
  }

  .stats-bar,
  .status-bar {
    flex-wrap: wrap;
    gap: 8px;
  }

  .stat-divider {
    display: none;
  }
}
</style>
