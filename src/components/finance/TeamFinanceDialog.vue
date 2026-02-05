<template>
  <el-dialog
    :model-value="modelValue"
    :title="team ? `${team.team_name} - 财务详情` : '财务详情'"
    width="800px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="$emit('close')"
  >
    <template v-if="team">
      <!-- 财务概览 -->
      <div class="finance-overview">
        <div class="overview-item">
          <div class="overview-label">当前余额</div>
          <div class="overview-value" :class="{ negative: team.balance < 0 }">
            {{ formatMoney(team.balance) }}
          </div>
        </div>
        <div class="overview-item">
          <div class="overview-label">财务状态</div>
          <div class="overview-value">
            <el-tag :type="getStatusTagType(team.financial_status)" effect="dark" size="large">
              {{ getStatusInfo(team.financial_status).icon }} {{ getStatusInfo(team.financial_status).label }}
            </el-tag>
          </div>
        </div>
        <div class="overview-item">
          <div class="overview-label">转会预算</div>
          <div class="overview-value budget">{{ formatMoney(team.transfer_budget) }}</div>
        </div>
      </div>

      <!-- 收支明细 -->
      <el-row :gutter="20" class="detail-row">
        <el-col :span="12">
          <div class="detail-card income">
            <div class="detail-header">
              <el-icon><Top /></el-icon>
              <span>收入明细</span>
            </div>
            <div class="detail-content">
              <div class="detail-item">
                <span class="item-label">联赛分成</span>
                <span class="item-value">{{ formatMoney(report?.breakdown?.league_share || 0) }}</span>
              </div>
              <div class="detail-item">
                <span class="item-label">赛事奖金</span>
                <span class="item-value">{{ formatMoney(report?.breakdown?.prize_money || 0) }}</span>
              </div>
              <div class="detail-item">
                <span class="item-label">赞助收入</span>
                <span class="item-value">{{ formatMoney(report?.breakdown?.transfers_in || 0) }}</span>
              </div>
              <div class="detail-item" v-if="(report?.breakdown?.transfers_in || 0) > 0">
                <span class="item-label">转会收入</span>
                <span class="item-value">{{ formatMoney(report?.breakdown?.transfers_in || 0) }}</span>
              </div>
              <el-divider />
              <div class="detail-total">
                <span class="total-label">总收入</span>
                <span class="total-value income">+{{ formatMoney(team.total_income) }}</span>
              </div>
            </div>
          </div>
        </el-col>
        <el-col :span="12">
          <div class="detail-card expense">
            <div class="detail-header">
              <el-icon><Bottom /></el-icon>
              <span>支出明细</span>
            </div>
            <div class="detail-content">
              <div class="detail-item">
                <span class="item-label">选手薪资</span>
                <span class="item-value">{{ formatMoney(team.total_salary) }}</span>
              </div>
              <div class="detail-item">
                <span class="item-label">运营成本</span>
                <span class="item-value">{{ formatMoney(report?.breakdown?.other || 0) }}</span>
              </div>
              <div class="detail-item" v-if="(report?.breakdown?.transfers_out || 0) > 0">
                <span class="item-label">转会支出</span>
                <span class="item-value">{{ formatMoney(Math.abs(report?.breakdown?.transfers_out || 0)) }}</span>
              </div>
              <el-divider />
              <div class="detail-total">
                <span class="total-label">总支出</span>
                <span class="total-value expense">-{{ formatMoney(team.total_expense) }}</span>
              </div>
            </div>
          </div>
        </el-col>
      </el-row>

      <!-- 预测信息 -->
      <div class="forecast-section">
        <div class="forecast-title">
          <el-icon><TrendCharts /></el-icon>
          <span>财务预测</span>
        </div>
        <div class="forecast-content">
          <div class="forecast-item">
            <span class="forecast-label">预计赛季利润</span>
            <span class="forecast-value" :class="{ positive: team.projected_season_profit >= 0, negative: team.projected_season_profit < 0 }">
              {{ team.projected_season_profit >= 0 ? '+' : '' }}{{ formatMoney(team.projected_season_profit) }}
            </span>
          </div>
          <div class="forecast-item">
            <span class="forecast-label">最大可签薪资</span>
            <span class="forecast-value">{{ formatMoney(team.max_new_salary) }}</span>
          </div>
          <div class="forecast-item" v-if="team.is_crisis">
            <el-alert type="warning" :closable="false" show-icon>
              <template #title>
                <strong>财务危机警告</strong>
              </template>
              该队伍正处于财务危机状态，建议尽快调整财务策略。
            </el-alert>
          </div>
        </div>
      </div>

      <!-- 赛事奖金明细 -->
      <div class="prize-details-section" v-if="prizeDetails.length > 0">
        <div class="section-header">
          <div class="section-title prize">
            <el-icon><Trophy /></el-icon>
            <span>赛事奖金明细</span>
          </div>
          <div class="prize-total">
            累计奖金：<span class="money-income">{{ formatMoney(totalPrizeMoney) }}</span>
          </div>
        </div>
        <el-table :data="prizeDetails" stripe size="small" max-height="300">
          <el-table-column prop="season_id" label="赛季" width="70" align="center">
            <template #default="{ row }">S{{ row.season_id }}</template>
          </el-table-column>
          <el-table-column prop="tournament_name" label="赛事" min-width="150" />
          <el-table-column prop="tournament_type" label="类型" width="80" align="center">
            <template #default="{ row }">
              <el-tag :type="row.tournament_type === 'international' ? 'warning' : ''" size="small">
                {{ row.tournament_type === 'international' ? '国际赛' : '赛区赛' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="position" label="名次" width="80" align="center">
            <template #default="{ row }">
              {{ formatPosition(row.position) }}
            </template>
          </el-table-column>
          <el-table-column prop="amount" label="奖金" width="100" align="right">
            <template #default="{ row }">
              <span class="money-income">+{{ formatMoney(row.amount) }}</span>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 交易记录 -->
      <div class="transactions-section" v-if="transactions.length > 0">
        <div class="transactions-title">
          <el-icon><List /></el-icon>
          <span>近期交易记录</span>
        </div>
        <el-table :data="transactions" stripe max-height="250">
          <el-table-column prop="season_id" label="赛季" width="80" align="center">
            <template #default="{ row }">S{{ row.season_id }}</template>
          </el-table-column>
          <el-table-column prop="transaction_type" label="类型" width="120">
            <template #default="{ row }">
              <el-tag :type="getTransactionTagType(row.transaction_type)" size="small">
                {{ getTransactionLabel(row.transaction_type) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="amount" label="金额" width="120" align="right">
            <template #default="{ row }">
              <span :class="row.amount >= 0 ? 'money-income' : 'money-expense'">
                {{ row.amount >= 0 ? '+' : '' }}{{ formatMoney(row.amount) }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="description" label="说明" />
        </el-table>
      </div>
    </template>

    <template #footer>
      <el-button @click="$emit('close')">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  Top,
  Bottom,
  TrendCharts,
  List,
  Trophy,
} from '@element-plus/icons-vue'
import { useFinanceStore, type FinancialStatus } from '@/stores/useFinanceStore'
import type { TeamFinanceSummary, FinanceTransaction, SeasonFinanceReport, TournamentPrizeDetail } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TeamFinanceDialog')

const props = defineProps<{
  modelValue: boolean
  team: TeamFinanceSummary | null
}>()

defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'close'): void
}>()

const financeStore = useFinanceStore()

const transactions = ref<FinanceTransaction[]>([])
const report = ref<SeasonFinanceReport | null>(null)
const prizeDetails = ref<TournamentPrizeDetail[]>([])

// 计算累计奖金总额
const totalPrizeMoney = computed(() => {
  return prizeDetails.value.reduce((sum, item) => sum + item.amount, 0)
})

// 监听team变化，加载详细数据
watch(() => props.team, async (newTeam) => {
  if (newTeam) {
    try {
      // 加载交易记录
      const txns = await financeStore.fetchTeamTransactions(newTeam.team_id)
      transactions.value = txns.slice(0, 10) // 只显示最近10条

      // 加载赛季报告
      report.value = await financeStore.fetchSeasonReport(newTeam.team_id)

      // 加载赛事奖金明细
      prizeDetails.value = await financeStore.fetchTeamPrizeDetails(newTeam.team_id)
    } catch (e) {
      logger.error('加载财务详情失败', { teamId: newTeam.team_id, error: e })
    }
  }
}, { immediate: true })

// 格式化金额
function formatMoney(amount: number): string {
  return financeStore.formatMoney(amount)
}

// 获取状态信息
function getStatusInfo(status: FinancialStatus) {
  return financeStore.getStatusInfo(status)
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

// 获取交易类型标签
function getTransactionTagType(type: string): string {
  if (type.includes('Income') || type.includes('Bonus') || type.includes('Share')) {
    return 'success'
  }
  if (type.includes('Expense') || type.includes('Salary')) {
    return 'danger'
  }
  return 'info'
}

// 获取交易类型标签文字
function getTransactionLabel(type: string): string {
  const labels: Record<string, string> = {
    LeagueShare: '联赛分成',
    PlayoffBonus: '季后赛奖金',
    InternationalBonus: '国际赛奖金',
    Sponsorship: '赞助收入',
    TransferIncome: '转会收入',
    TransferExpense: '转会支出',
    Salary: '薪资支出',
    OperatingCost: '运营成本',
  }
  return labels[type] || type
}

// 格式化赛事名次
function formatPosition(position: string): string {
  const positionMap: Record<string, string> = {
    'CHAMPION': '冠军',
    'RUNNER_UP': '亚军',
    'THIRD': '季军',
    'FOURTH': '殿军',
    'QUARTER_FINAL': '八强',
    'GROUP_STAGE': '小组赛',
    '5TH_8TH': '5-8名',
    'SEMI_LOSER': '四强',
    'R1_LOSER': '首轮',
    'LOSERS_R2': '败者组',
    'LOSERS_R1': '败者组',
    'PREP_LOSER': '预选赛',
    'PROMOTION_LOSER': '晋级赛',
    'FIGHTER_OUT': '斗士出局',
  }
  return positionMap[position] || position
}
</script>

<style scoped>
/* 财务概览 */
.finance-overview {
  display: flex;
  justify-content: space-around;
  padding: 20px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e7ed 100%);
  border-radius: 12px;
  margin-bottom: 20px;
}

.overview-item {
  text-align: center;
}

.overview-label {
  font-size: 13px;
  color: #909399;
  margin-bottom: 8px;
}

.overview-value {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
}

.overview-value.negative {
  color: #ef4444;
}

.overview-value.budget {
  color: #3b82f6;
}

/* 收支明细卡片 */
.detail-row {
  margin-bottom: 20px;
}

.detail-card {
  background: #fafafa;
  border-radius: 12px;
  padding: 16px;
  height: 100%;
}

.detail-card.income {
  border-left: 4px solid #10b981;
}

.detail-card.expense {
  border-left: 4px solid #ef4444;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 16px;
}

.detail-card.income .detail-header {
  color: #10b981;
}

.detail-card.expense .detail-header {
  color: #ef4444;
}

.detail-content {
  padding: 0 8px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.item-label {
  font-size: 14px;
  color: #606266;
}

.item-value {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.detail-total {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.total-label {
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.total-value {
  font-size: 18px;
  font-weight: 700;
}

.total-value.income {
  color: #10b981;
}

.total-value.expense {
  color: #ef4444;
}

/* 预测区域 */
.forecast-section {
  background: #f0f9ff;
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 20px;
}

.forecast-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #3b82f6;
  margin-bottom: 16px;
}

.forecast-content {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
}

.forecast-item {
  flex: 1;
  min-width: 200px;
}

.forecast-label {
  display: block;
  font-size: 13px;
  color: #909399;
  margin-bottom: 4px;
}

.forecast-value {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.forecast-value.positive {
  color: #10b981;
}

.forecast-value.negative {
  color: #ef4444;
}

/* 交易记录 */
.transactions-section {
  margin-top: 20px;
}

.transactions-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 12px;
}

/* 金额样式 */
.money-income {
  color: #10b981;
  font-weight: 500;
}

.money-expense {
  color: #ef4444;
  font-weight: 500;
}

/* 赛事奖金明细 */
.prize-details-section {
  background: #fffbeb;
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.section-title.prize {
  color: #d97706;
}

.prize-total {
  font-size: 14px;
  color: #606266;
}

.prize-total .money-income {
  font-size: 16px;
  font-weight: 600;
}
</style>
