<template>
  <div class="transfer-system">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>转会系统</h1>
        <p>S{{ currentSeason }} 赛季 - 休赛期转会</p>
      </div>
      <div class="header-actions">
        <el-tag type="info" size="large" effect="dark">
          全球统一转会期
        </el-tag>
      </div>
    </div>

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
          <template v-if="transferInProgress">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Opportunity,
  Flag,
  Check,
  Warning,
  Setting,
  VideoPlay,
  InfoFilled,
} from '@element-plus/icons-vue'
import { useTransferWindowStore, ROUND_NAMES } from '@/stores/useTransferWindowStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi } from '@/api/tauri'
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
const gameStore = useGameStore()

const { currentSeason } = storeToRefs(gameStore)

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
  await loadData()
  // 初始化转会期状态（检查是否有进行中的转会期）
  await transferStore.initTransferWindow()
})
</script>

<style scoped>
.transfer-system {
  padding: 0;
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

/* 流程介绍卡片 */
.intro-card {
  margin-bottom: 24px;
  border-radius: 16px;
  overflow: hidden;
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
  border-radius: 16px;
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
</style>
