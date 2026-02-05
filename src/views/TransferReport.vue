<template>
  <div class="transfer-report">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push('/transfer')">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回转会系统</span>
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="isLoading" class="loading-container">
      <el-icon class="is-loading" :size="48"><Loading /></el-icon>
      <p>加载转会报告中...</p>
    </div>

    <!-- 报告内容 -->
    <template v-else-if="report">
      <!-- 页面标题 -->
      <div class="page-header">
        <div>
          <h1>转会报告</h1>
          <p>S{{ report.season_id }} 赛季转会期总结</p>
        </div>
        <div class="header-actions">
          <el-tag type="success" size="large" effect="dark">
            已完成
          </el-tag>
        </div>
      </div>

      <!-- 总览统计 -->
      <el-row :gutter="16" class="stats-row">
        <el-col :span="6">
          <el-card class="stat-card highlight">
            <div class="stat-content">
              <div class="stat-icon blue">
                <el-icon :size="32"><Document /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ report.total_events }}</div>
                <div class="stat-label">总事件数</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card highlight">
            <div class="stat-content">
              <div class="stat-icon gold">
                <el-icon :size="32"><Money /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ formatAmount(report.total_transfer_fee) }}</div>
                <div class="stat-label">总转会费</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card highlight">
            <div class="stat-content">
              <div class="stat-icon green">
                <el-icon :size="32"><UserFilled /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ report.team_summaries.length }}</div>
                <div class="stat-label">参与球队</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card highlight">
            <div class="stat-content">
              <div class="stat-icon purple">
                <el-icon :size="32"><Star /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ report.top_events.length }}</div>
                <div class="stat-label">头条事件</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 事件类型分布 -->
      <el-row :gutter="16" class="charts-row">
        <el-col :span="12">
          <el-card class="chart-card">
            <template #header>
              <div class="card-header">
                <el-icon><PieChart /></el-icon>
                <span>事件类型分布</span>
              </div>
            </template>
            <div class="type-distribution">
              <div
                v-for="(count, type) in report.events_by_type"
                :key="type"
                class="type-item"
              >
                <div class="type-bar" :style="{ width: getTypePercentage(count) + '%', background: getTypeColor(type as string) }"></div>
                <span class="type-name">{{ getTypeName(type as string) }}</span>
                <span class="type-count">{{ count }}</span>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card class="chart-card">
            <template #header>
              <div class="card-header">
                <el-icon><Medal /></el-icon>
                <span>事件等级分布</span>
              </div>
            </template>
            <div class="level-distribution">
              <div
                v-for="(count, level) in report.events_by_level"
                :key="level"
                class="level-item"
              >
                <div class="level-badge" :class="(level as string).toLowerCase()">{{ level }}</div>
                <div class="level-info">
                  <span class="level-label">{{ getLevelLabel(level as string) }}</span>
                  <span class="level-count">{{ count }} 个事件</span>
                </div>
                <div class="level-bar-wrapper">
                  <div class="level-bar" :style="{ width: getLevelPercentage(count) + '%' }" :class="(level as string).toLowerCase()"></div>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 球队转会汇总 -->
      <el-card class="teams-card">
        <template #header>
          <div class="card-header">
            <el-icon><OfficeBuilding /></el-icon>
            <span>球队转会汇总</span>
          </div>
        </template>
        <el-table :data="sortedTeamSummaries" stripe style="width: 100%">
          <el-table-column prop="team_name" label="球队" width="180">
            <template #default="{ row }">
              <span class="team-name">{{ row.team_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="players_in" label="转入" width="100" align="center">
            <template #default="{ row }">
              <el-tag v-if="row.players_in > 0" type="success" size="small">+{{ row.players_in }}</el-tag>
              <span v-else class="zero">0</span>
            </template>
          </el-table-column>
          <el-table-column prop="players_out" label="转出" width="100" align="center">
            <template #default="{ row }">
              <el-tag v-if="row.players_out > 0" type="danger" size="small">-{{ row.players_out }}</el-tag>
              <span v-else class="zero">0</span>
            </template>
          </el-table-column>
          <el-table-column prop="money_spent" label="支出" width="150" align="right">
            <template #default="{ row }">
              <span class="money spent">{{ formatAmount(row.money_spent) }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="money_earned" label="收入" width="150" align="right">
            <template #default="{ row }">
              <span class="money earned">{{ formatAmount(row.money_earned) }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="net_spend" label="净支出" width="150" align="right">
            <template #default="{ row }">
              <span class="money" :class="row.net_spend > 0 ? 'spent' : 'earned'">
                {{ row.net_spend > 0 ? '-' : '+' }}{{ formatAmount(Math.abs(row.net_spend)) }}
              </span>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <!-- 头条事件 -->
      <el-card class="top-events-card">
        <template #header>
          <div class="card-header">
            <el-icon><Star /></el-icon>
            <span>头条事件</span>
          </div>
        </template>
        <div class="top-events-list">
          <div
            v-for="event in report.top_events"
            :key="event.id"
            class="top-event-item"
            :class="`level-${event.level.toLowerCase()}`"
          >
            <div class="event-level-badge" :class="event.level.toLowerCase()">
              {{ event.level }}
            </div>
            <div class="event-info">
              <div class="event-headline">{{ getEventHeadline(event) }}</div>
              <div class="event-meta">
                <span v-if="event.transfer_fee > 0" class="meta-item fee">
                  <el-icon><Money /></el-icon>
                  {{ formatAmount(event.transfer_fee) }}
                </span>
                <span v-if="event.salary > 0" class="meta-item">
                  <el-icon><Wallet /></el-icon>
                  年薪 {{ formatAmount(event.salary) }}
                </span>
                <span class="meta-item">
                  <el-icon><Calendar /></el-icon>
                  第{{ event.round }}轮
                </span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 操作按钮 -->
      <div class="action-bar">
        <el-button type="primary" size="large" @click="$router.push('/transfer')">
          <el-icon><Back /></el-icon>
          返回转会系统
        </el-button>
      </div>
    </template>

    <!-- 无数据 -->
    <el-empty v-else description="未找到转会报告" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import {
  ArrowLeft,
  Loading,
  Document,
  Money,
  UserFilled,
  Star,
  PieChart,
  Medal,
  OfficeBuilding,
  Back,
  Wallet,
  Calendar,
} from '@element-plus/icons-vue'
import { transferWindowApi, type TransferReport, type TransferEvent } from '@/api/tauri'
import { formatMoneyFromWan } from '@/utils'
import { EVENT_TYPE_NAMES, EVENT_LEVEL_CONFIG } from '@/stores/useTransferWindowStore'

const route = useRoute()

const isLoading = ref(true)
const report = ref<TransferReport | null>(null)

// 按净支出排序的球队汇总
const sortedTeamSummaries = computed(() => {
  if (!report.value) return []
  return [...report.value.team_summaries].sort((a, b) => b.net_spend - a.net_spend)
})

// 加载报告
async function loadReport() {
  const windowId = parseInt(route.params.windowId as string)
  if (isNaN(windowId)) {
    isLoading.value = false
    return
  }

  try {
    report.value = await transferWindowApi.getTransferReport(windowId)
  } catch (e) {
    console.error('Failed to load report:', e)
  } finally {
    isLoading.value = false
  }
}

// 格式化金额 - 使用统一工具（输入单位为万）
function formatAmount(amount: number): string {
  return formatMoneyFromWan(amount)
}

// 获取类型名称
function getTypeName(type: string): string {
  return EVENT_TYPE_NAMES[type] ?? type
}

// 获取类型颜色
function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    CONTRACT_RENEWAL: '#22c55e',
    CONTRACT_TERMINATION: '#ef4444',
    FREE_AGENT_SIGNING: '#3b82f6',
    TRANSFER_PURCHASE: '#f59e0b',
    PLAYER_RETIREMENT: '#6b7280',
    PLAYER_LISTED: '#f97316',
    EMERGENCY_SIGNING: '#8b5cf6',
    SEASON_SETTLEMENT: '#06b6d4',
    DRAFT_PICK_AUCTION: '#14b8a6',
    FINANCIAL_ADJUSTMENT: '#ec4899',
  }
  return colors[type] ?? '#9ca3af'
}

// 获取类型百分比
function getTypePercentage(count: number): number {
  if (!report.value) return 0
  const max = Math.max(...Object.values(report.value.events_by_type))
  return (count / max) * 100
}

// 获取等级标签
function getLevelLabel(level: string): string {
  return EVENT_LEVEL_CONFIG[level]?.label ?? level
}

// 获取等级百分比
function getLevelPercentage(count: number): number {
  if (!report.value) return 0
  const max = Math.max(...Object.values(report.value.events_by_level))
  return (count / max) * 100
}

// 获取事件标题
function getEventHeadline(event: TransferEvent): string {
  switch (event.event_type) {
    case 'CONTRACT_RENEWAL':
      return `${event.player_name} 续约 ${event.to_team_name || event.from_team_name}`
    case 'CONTRACT_TERMINATION':
      return `${event.player_name} 离开 ${event.from_team_name}`
    case 'FREE_AGENT_SIGNING':
      return `${event.to_team_name} 签下 ${event.player_name}`
    case 'TRANSFER_PURCHASE':
      return `${event.player_name} 从 ${event.from_team_name} 转会至 ${event.to_team_name}`
    case 'PLAYER_RETIREMENT':
      return `${event.player_name} 宣布退役`
    case 'EMERGENCY_SIGNING':
      return `${event.to_team_name} 紧急签约 ${event.player_name}`
    default:
      return `${event.player_name} - ${getTypeName(event.event_type)}`
  }
}

onMounted(() => {
  loadReport()
})
</script>

<style scoped>
.transfer-report {
  padding: 0;
}

/* 返回导航 */
.back-nav {
  margin-bottom: 20px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  background: none;
  border: none;
  color: #6b7280;
  font-size: 14px;
  cursor: pointer;
  transition: color 0.2s;
}

.back-btn:hover {
  color: #3b82f6;
}

/* 加载中 */
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  color: #909399;
}

.loading-container p {
  margin-top: 16px;
  font-size: 14px;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.page-header h1 {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 24px;
}

.stat-card.highlight {
  border-radius: 16px;
  border: none;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.stat-card .stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-card .stat-icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); }
.stat-icon.green { background: linear-gradient(135deg, #22c55e, #16a34a); }
.stat-icon.purple { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 13px;
  color: #909399;
  margin-top: 6px;
}

/* 图表卡片 */
.charts-row {
  margin-bottom: 24px;
}

.chart-card {
  border-radius: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

/* 类型分布 */
.type-distribution {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.type-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.type-bar {
  height: 24px;
  border-radius: 4px;
  min-width: 4px;
  transition: width 0.3s ease;
}

.type-name {
  flex: 1;
  font-size: 13px;
  color: #606266;
}

.type-count {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

/* 等级分布 */
.level-distribution {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.level-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.level-badge {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 14px;
  font-weight: 700;
  flex-shrink: 0;
}

.level-badge.s { background: #f59e0b; }
.level-badge.a { background: #8b5cf6; }
.level-badge.b { background: #3b82f6; }
.level-badge.c { background: #9ca3af; }

.level-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 80px;
}

.level-label {
  font-size: 13px;
  font-weight: 500;
  color: #303133;
}

.level-count {
  font-size: 12px;
  color: #909399;
}

.level-bar-wrapper {
  flex: 1;
  height: 8px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
}

.level-bar {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.level-bar.s { background: #f59e0b; }
.level-bar.a { background: #8b5cf6; }
.level-bar.b { background: #3b82f6; }
.level-bar.c { background: #9ca3af; }

/* 球队汇总表格 */
.teams-card {
  border-radius: 16px;
  margin-bottom: 24px;
}

.team-name {
  font-weight: 600;
  color: #303133;
}

.zero {
  color: #c0c4cc;
}

.money {
  font-weight: 500;
}

.money.spent {
  color: #ef4444;
}

.money.earned {
  color: #22c55e;
}

/* 头条事件 */
.top-events-card {
  border-radius: 16px;
  margin-bottom: 24px;
}

.top-events-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.top-event-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border-radius: 12px;
  background: #f9fafb;
  transition: all 0.2s ease;
}

.top-event-item:hover {
  background: #f3f4f6;
}

.top-event-item.level-s {
  background: linear-gradient(135deg, #fffbeb, #fef3c7);
}

.top-event-item.level-a {
  background: linear-gradient(135deg, #faf5ff, #f3e8ff);
}

.event-level-badge {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 16px;
  font-weight: 700;
  flex-shrink: 0;
}

.event-level-badge.s { background: #f59e0b; }
.event-level-badge.a { background: #8b5cf6; }
.event-level-badge.b { background: #3b82f6; }
.event-level-badge.c { background: #9ca3af; }

.event-info {
  flex: 1;
}

.event-headline {
  font-size: 15px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 6px;
}

.event-meta {
  display: flex;
  gap: 16px;
  align-items: center;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: #606266;
}

.meta-item.fee {
  font-weight: 600;
  color: #f59e0b;
}

/* 操作按钮 */
.action-bar {
  display: flex;
  justify-content: center;
  margin-top: 24px;
}
</style>
