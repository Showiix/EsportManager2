<template>
  <div class="transfer-system">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>转会系统</h1>
        <p>S{{ selectedSeason }} 赛季 - 休赛期转会</p>
      </div>
      <div class="header-actions">
        <SeasonSelector v-model="selectedSeason" width="140px" />
        <el-tag type="info" size="large" effect="dark">
          全球统一转会期
        </el-tag>
      </div>
    </div>

    <!-- 当前赛季：显示操作界面 -->
    <template v-if="!isViewingHistory">
      <!-- 非转会期警告 -->
      <el-alert
        v-if="!isTransferPhase"
        type="warning"
        show-icon
        :closable="false"
        class="phase-warning"
      >
        <template #title>
          当前阶段：{{ timeStore.phaseDisplayName }}，需要在时间推进面板推进到转会期阶段才能操作
        </template>
      </el-alert>

      <!-- 流程说明 -->
      <el-card class="intro-card">
        <div class="intro-content">
          <div class="intro-icon">
            <el-icon :size="48"><Opportunity /></el-icon>
          </div>
          <div class="intro-text">
            <h3>转会期流程</h3>
            <p>全球四大赛区（LPL、LCK、LEC、LCS）统一进行转会，共 8 个阶段：</p>
            <div class="round-flow">
              <div v-for="(name, round) in roundNames" :key="round" class="round-item">
                <span class="round-number">{{ round }}</span>
                <span class="round-name">{{ name }}</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 统一入口 -->
      <el-card class="action-card">
        <div class="action-content">
          <div class="action-info">
            <div class="region-badges">
              <span class="region-badge lpl">LPL</span>
              <span class="region-badge lck">LCK</span>
              <span class="region-badge lec">LEC</span>
              <span class="region-badge lcs">LCS</span>
            </div>
            <h3>全球转会期</h3>
            <p>管理全部 {{ totalTeams }} 支战队的 {{ totalPlayers }} 名选手</p>
          </div>

          <div class="action-status">
            <template v-if="!isTransferPhase">
              <el-tag type="info" size="large" class="status-tag">
                <span class="tag-content"><el-icon><Warning /></el-icon> 未到转会期</span>
              </el-tag>
              <el-button type="info" size="large" disabled>
                <el-icon><VideoPlay /></el-icon>
                未到转会期阶段
              </el-button>
            </template>
            <template v-else-if="transferInProgress">
              <el-tag type="success" size="large" class="status-tag">
                <span class="tag-content"><el-icon><VideoPlay /></el-icon> 转会期进行中 (第{{ transferStore.currentRound }}轮)</span>
              </el-tag>
              <el-button type="success" size="large" @click="continueTransfer">
                <el-icon><VideoPlay /></el-icon>
                继续转会期
              </el-button>
            </template>
            <template v-else-if="!gmConfigured">
              <el-tag type="warning" size="large" class="status-tag">
                <span class="tag-content"><el-icon><Warning /></el-icon> 需配置 {{ unconfiguredGMCount }} 队GM</span>
              </el-tag>
              <el-button type="warning" size="large" @click="goToGMConfig">
                <el-icon><Setting /></el-icon>
                配置GM性格
              </el-button>
            </template>
            <template v-else>
              <el-tag type="success" size="large" class="status-tag">
                <span class="tag-content"><el-icon><Check /></el-icon> GM配置完成</span>
              </el-tag>
              <el-button type="primary" size="large" @click="startTransfer">
                <el-icon><VideoPlay /></el-icon>
                开始转会期
              </el-button>
            </template>
          </div>
        </div>
      </el-card>

      <!-- 赛区统计 -->
      <div class="stats-section">
        <div class="section-header">
          <h2>
            <el-icon><Flag /></el-icon>
            赛区概览
          </h2>
        </div>

        <el-row :gutter="16">
          <el-col v-for="region in regions" :key="region.id" :span="6">
            <el-card class="region-stat-card">
              <div class="region-stat-header" :style="{ background: getRegionGradient(region.code) }">
                <span class="region-code">{{ region.code }}</span>
                <span class="region-name">{{ region.name }}</span>
              </div>
              <div class="region-stat-body">
                <div class="stat-item">
                  <span class="stat-value">{{ getRegionTeamCount(region.id) }}</span>
                  <span class="stat-label">战队</span>
                </div>
                <div class="stat-item">
                  <span class="stat-value">{{ getRegionPlayerCount(region.id) }}</span>
                  <span class="stat-label">选手</span>
                </div>
                <div class="stat-item">
                  <template v-if="getRegionGMStatus(region.id).allConfigured">
                    <el-icon class="stat-icon success"><Check /></el-icon>
                  </template>
                  <template v-else>
                    <span class="stat-value warning">{{ getRegionGMStatus(region.id).unconfigured }}</span>
                  </template>
                  <span class="stat-label">待配置</span>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </div>

      <!-- 转会须知 -->
      <el-card class="notice-card">
        <template #header>
          <div class="card-header">
            <el-icon><InfoFilled /></el-icon>
            <span>转会须知</span>
          </div>
        </template>
        <ul class="notice-list">
          <li>转会期开始前，必须为所有 AI 球队配置 GM 性格</li>
          <li>四个赛区统一进行转会，选手可跨赛区转会</li>
          <li>中国选手更倾向留在 LPL，韩国选手相对开放外出</li>
          <li>转会期间会自动处理合同续约、自由球员签约、球员挖角等事务</li>
          <li>转会完成后可查看详细的转会报告</li>
        </ul>
      </el-card>
    </template>

    <!-- 历史赛季：显示转会记录摘要 -->
    <template v-else>
      <el-card v-if="historyLoading" class="history-card">
        <el-skeleton :rows="4" animated />
      </el-card>

      <template v-else-if="historyWindow">
        <el-card class="history-card">
          <div class="history-summary">
            <div class="history-icon">
              <el-icon :size="48"><Opportunity /></el-icon>
            </div>
            <div class="history-info">
              <h3>S{{ selectedSeason }} 赛季转会记录</h3>
              <div class="history-stats">
                <div class="history-stat-item">
                  <span class="history-stat-value">{{ historyReport?.total_events ?? 0 }}</span>
                  <span class="history-stat-label">转会事件</span>
                </div>
                <div class="history-stat-item">
                  <span class="history-stat-value">{{ formatMoney(historyReport?.total_transfer_fee ?? 0) }}</span>
                  <span class="history-stat-label">总转会费</span>
                </div>
                <div class="history-stat-item">
                  <span class="history-stat-value">{{ historyWindow.current_round }}</span>
                  <span class="history-stat-label">转会轮次</span>
                </div>
                <div class="history-stat-item">
                  <el-tag :type="historyWindow.status === 'completed' ? 'success' : 'warning'" size="small">
                    {{ historyWindow.status === 'completed' ? '已完成' : '进行中' }}
                  </el-tag>
                  <span class="history-stat-label">状态</span>
                </div>
              </div>
            </div>
          </div>
        </el-card>

        <!-- 转会类型分布 -->
        <el-card v-if="historyReport && Object.keys(historyReport.events_by_type).length > 0" class="history-card">
          <template #header>
            <div class="card-header">
              <el-icon><Flag /></el-icon>
              <span>转会类型分布</span>
            </div>
          </template>
          <div class="type-distribution">
            <div v-for="(count, type) in historyReport.events_by_type" :key="type" class="type-item">
              <span class="type-name">{{ EVENT_TYPE_NAMES[type as string] ?? type }}</span>
              <div class="type-bar-wrapper">
                <div class="type-bar" :style="{ width: getHistoryTypePercentage(count) + '%', background: getHistoryTypeColor(type as string) }"></div>
              </div>
              <span class="type-count">{{ count }}</span>
            </div>
          </div>
        </el-card>

        <!-- 查看完整报告按钮 -->
        <div class="history-actions">
          <el-button type="primary" size="large" @click="goToReport(historyWindow.window_id)">
            <el-icon><Document /></el-icon>
            查看完整转会报告
          </el-button>
        </div>
      </template>

      <el-empty v-else description="该赛季没有转会记录" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  Opportunity,
  Flag,
  Check,
  Warning,
  Setting,
  VideoPlay,
  InfoFilled,
  Document,
} from '@element-plus/icons-vue'
import { useTransferWindowStore, ROUND_NAMES, EVENT_TYPE_NAMES } from '@/stores/useTransferWindowStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { queryApi, transferWindowApi } from '@/api/tauri'
import type { TransferWindowResponse, TransferReport } from '@/api/tauri'
import SeasonSelector from '@/components/common/SeasonSelector.vue'
import { formatMoney } from '@/utils/format'
import { createLogger } from '@/utils/logger'

const logger = createLogger('Transfer')

interface Region {
  id: number
  code: string
  name: string
}

interface Team {
  id: number
  name: string
  region_id: number
}

const router = useRouter()
const transferStore = useTransferWindowStore()
const timeStore = useTimeStore()

// 是否在转会期阶段
const isTransferPhase = computed(() => timeStore.isInTransferWindow)

// 赛季选择
const selectedSeason = ref(0)
const isViewingHistory = computed(() => selectedSeason.value !== 0 && selectedSeason.value !== timeStore.currentSeasonFromTime)

// 历史数据
const historyWindow = ref<TransferWindowResponse | null>(null)
const historyReport = ref<TransferReport | null>(null)
const historyLoading = ref(false)

// 状态
const regions = ref<Region[]>([])
const teamsByRegion = ref<Map<number, Team[]>>(new Map())
const gmStatusByRegion = ref<Map<number, { total: number; configured: number }>>(new Map())
const isLoading = ref(false)

// 轮次名称
const roundNames = ROUND_NAMES

// 赛区颜色
const regionColors: Record<string, string> = {
  LPL: 'linear-gradient(135deg, #ef4444, #dc2626)',
  LCK: 'linear-gradient(135deg, #3b82f6, #2563eb)',
  LEC: 'linear-gradient(135deg, #22c55e, #16a34a)',
  LCS: 'linear-gradient(135deg, #f59e0b, #d97706)',
}

// 计算属性
const totalTeams = computed(() => {
  let count = 0
  teamsByRegion.value.forEach(teams => {
    count += teams.length
  })
  return count
})

const totalPlayers = computed(() => {
  // 估算：平均每队7名选手
  return totalTeams.value * 7
})

const gmConfigured = computed(() => {
  let allConfigured = true
  gmStatusByRegion.value.forEach(status => {
    if (status.configured < status.total) {
      allConfigured = false
    }
  })
  return allConfigured
})

const transferInProgress = computed(() => {
  return transferStore.isWindowStarted && !transferStore.isWindowCompleted
})

const unconfiguredGMCount = computed(() => {
  let count = 0
  gmStatusByRegion.value.forEach(status => {
    count += status.total - status.configured
  })
  return count
})

// 获取赛区渐变色
function getRegionGradient(code: string): string {
  return regionColors[code.toUpperCase()] || 'linear-gradient(135deg, #6b7280, #4b5563)'
}

// 获取赛区球队数量
function getRegionTeamCount(regionId: number): number {
  return teamsByRegion.value.get(regionId)?.length ?? 0
}

// 获取赛区选手数量（估算）
function getRegionPlayerCount(regionId: number): number {
  return getRegionTeamCount(regionId) * 7
}

// 获取赛区GM配置状态
function getRegionGMStatus(regionId: number) {
  const status = gmStatusByRegion.value.get(regionId)
  if (!status) {
    return { allConfigured: false, unconfigured: 0 }
  }
  return {
    allConfigured: status.configured === status.total,
    unconfigured: status.total - status.configured,
  }
}

// 跳转到GM配置
function goToGMConfig() {
  router.push('/transfer/gm-config')
}

// 开始转会期
function startTransfer() {
  transferStore.clearState()
  router.push('/transfer/window')
}

// 继续转会期（不清除状态）
function continueTransfer() {
  router.push('/transfer/window')
}

// 查看转会报告
function goToReport(windowId: number) {
  router.push(`/transfer/report/${windowId}`)
}

// 历史类型分布百分比
function getHistoryTypePercentage(count: number): number {
  if (!historyReport.value) return 0
  const max = Math.max(...Object.values(historyReport.value.events_by_type))
  return max > 0 ? (count / max) * 100 : 0
}

// 历史类型颜色
function getHistoryTypeColor(type: string): string {
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
    PLAYER_REQUEST_TRANSFER: '#e11d48',
  }
  return colors[type] ?? '#9ca3af'
}

// 加载历史赛季数据
async function loadHistoryData(season: number) {
  historyLoading.value = true
  historyWindow.value = null
  historyReport.value = null

  try {
    const window = await transferWindowApi.getTransferWindowBySeason(season)
    historyWindow.value = window
    if (window) {
      const report = await transferWindowApi.getTransferReport(window.window_id)
      historyReport.value = report
    }
  } catch (e) {
    logger.error('Failed to load history data:', e)
  } finally {
    historyLoading.value = false
  }
}

// 监听赛季切换
watch(selectedSeason, (val) => {
  if (val !== 0 && val !== timeStore.currentSeasonFromTime) {
    loadHistoryData(val)
  }
})

// 加载数据
async function loadData() {
  isLoading.value = true

  try {
    // 加载所有赛区
    const allRegions = await queryApi.getAllRegions()
    regions.value = allRegions.filter(r => ['LPL', 'LCK', 'LEC', 'LCS'].includes(r.code.toUpperCase()))

    // 加载每个赛区的球队
    for (const region of regions.value) {
      const teams = await queryApi.getTeamsByRegion(region.id)
      teamsByRegion.value.set(region.id, teams)

      // 加载GM配置状态
      let configuredCount = 0
      for (const team of teams) {
        const config = await transferStore.loadTeamPersonality(team.id)
        if (config) {
          configuredCount++
        }
      }
      gmStatusByRegion.value.set(region.id, {
        total: teams.length,
        configured: configuredCount,
      })
    }
  } catch (e) {
    logger.error('Failed to load data:', e)
    ElMessage.error('加载数据失败')
  } finally {
    isLoading.value = false
  }
}

onMounted(async () => {
  selectedSeason.value = timeStore.currentSeasonFromTime
  // 先加载时间状态，确保 phaseDisplayName 和 isInTransferWindow 正确
  await timeStore.fetchTimeState()
  await loadData()
  // 初始化转会期状态（检查是否有进行中的转会期）
  await transferStore.initTransferWindow()
})
</script>

<style scoped>
.transfer-system {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
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

/* 流程介绍卡片 */
.intro-card {
  margin-bottom: 24px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.intro-card :deep(.el-card__body) {
  padding: 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.intro-content {
  display: flex;
  gap: 24px;
  align-items: flex-start;
}

.intro-icon {
  width: 80px;
  height: 80px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.intro-text {
  flex: 1;
  color: white;
}

.intro-text h3 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.intro-text p {
  font-size: 14px;
  opacity: 0.9;
  margin: 0 0 16px 0;
}

.round-flow {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.round-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  font-size: 13px;
}

.round-number {
  width: 20px;
  height: 20px;
  background: white;
  color: #764ba2;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.round-name {
  opacity: 0.95;
}

/* 操作入口卡片 */
.action-card {
  margin-bottom: 24px;
  border-radius: 12px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.action-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
  flex-wrap: nowrap;
}

.action-info {
  flex: 1;
}

.region-badges {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.region-badge {
  padding: 4px 12px;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 600;
  color: white;
}

.region-badge.lpl { background: linear-gradient(135deg, #ef4444, #dc2626); }
.region-badge.lck { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.region-badge.lec { background: linear-gradient(135deg, #22c55e, #16a34a); }
.region-badge.lcs { background: linear-gradient(135deg, #f59e0b, #d97706); }

.action-info h3 {
  font-size: 22px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.action-info p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

.action-status {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  white-space: nowrap;
}

.status-tag {
  display: inline-flex;
  align-items: center;
  padding: 8px 12px;
  font-size: 13px;
}

.status-tag .tag-content {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.status-tag .el-icon {
  font-size: 14px;
  flex-shrink: 0;
}

/* 赛区统计 */
.stats-section {
  margin-bottom: 24px;
}

.section-header {
  margin-bottom: 16px;
}

.section-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.region-stat-card {
  border-radius: 12px;
  overflow: hidden;
}

.region-stat-card :deep(.el-card__body) {
  padding: 0;
}

.region-stat-header {
  padding: 16px;
  color: white;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.region-code {
  font-size: 20px;
  font-weight: 700;
}

.region-name {
  font-size: 13px;
  opacity: 0.9;
}

.region-stat-body {
  padding: 16px;
  display: flex;
  justify-content: space-around;
}

.stat-item {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 20px;
  font-weight: 700;
  color: #303133;
}

.stat-value.warning {
  color: #f59e0b;
}

.stat-label {
  display: block;
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.stat-icon {
  font-size: 20px;
}

.stat-icon.success {
  color: #22c55e;
}

/* 须知卡片 */
.notice-card {
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.notice-list {
  margin: 0;
  padding-left: 20px;
  color: #606266;
  line-height: 2;
}

.notice-list li {
  font-size: 14px;
}

/* 历史模式 */
.history-card {
  margin-bottom: 24px;
  border-radius: 12px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.history-summary {
  display: flex;
  gap: 24px;
  align-items: flex-start;
}

.history-icon {
  width: 80px;
  height: 80px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.history-info {
  flex: 1;
}

.history-info h3 {
  font-size: 20px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 16px 0;
}

.history-stats {
  display: flex;
  gap: 32px;
  flex-wrap: wrap;
}

.history-stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.history-stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #303133;
}

.history-stat-label {
  font-size: 12px;
  color: #909399;
}

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

.type-name {
  width: 80px;
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 500;
  color: #606266;
  text-align: right;
}

.type-bar-wrapper {
  flex: 1;
  height: 24px;
  background: #f3f4f6;
  border-radius: 6px;
  overflow: hidden;
}

.type-bar {
  height: 100%;
  border-radius: 6px;
  min-width: 4px;
  transition: width 0.3s ease;
}

.type-count {
  width: 36px;
  flex-shrink: 0;
  font-size: 14px;
  font-weight: 700;
  color: #303133;
  text-align: right;
}

.phase-warning {
  margin-bottom: 20px;
  border-radius: 8px;
}

.history-actions {
  text-align: center;
  margin-top: 24px;
}
</style>
