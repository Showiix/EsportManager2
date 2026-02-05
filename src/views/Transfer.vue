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
          选择赛区开始转会
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
          <p>每个赛区的转会期分为 8 个阶段，按顺序执行：</p>
          <div class="round-flow">
            <div v-for="(name, round) in roundNames" :key="round" class="round-item">
              <span class="round-number">{{ round }}</span>
              <span class="round-name">{{ name }}</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 赛区选择 -->
    <div class="regions-section">
      <div class="section-header">
        <h2>
          <el-icon><Flag /></el-icon>
          选择赛区
        </h2>
      </div>

      <div class="regions-grid">
        <div
          v-for="region in regions"
          :key="region.id"
          class="region-card"
          :class="{ disabled: !canStartTransfer(region) }"
          @click="handleRegionClick(region)"
        >
          <div class="region-header">
            <div class="region-logo" :style="{ background: getRegionGradient(region.code) }">
              {{ region.code }}
            </div>
            <div class="region-info">
              <h3>{{ region.name }}</h3>
              <p>{{ getRegionTeamCount(region.id) }} 支战队</p>
            </div>
          </div>

          <div class="region-status">
            <template v-if="getRegionGMStatus(region.id).allConfigured">
              <el-tag type="success" size="small" class="gm-status-tag">
                <span class="tag-content">
                  <el-icon><Check /></el-icon>
                  <span>GM配置完成</span>
                </span>
              </el-tag>
            </template>
            <template v-else>
              <el-tag type="warning" size="small" class="gm-status-tag">
                <span class="tag-content">
                  <el-icon><Warning /></el-icon>
                  <span>需配置 {{ getRegionGMStatus(region.id).unconfigured }} 队GM</span>
                </span>
              </el-tag>
            </template>
          </div>

          <div class="region-actions">
            <el-button
              v-if="!getRegionGMStatus(region.id).allConfigured"
              type="warning"
              size="small"
              @click.stop="goToGMConfig(region)"
            >
              <el-icon><Setting /></el-icon>
              配置GM
            </el-button>
            <el-button
              v-else
              type="primary"
              size="small"
              @click.stop="startRegionTransfer(region)"
            >
              <el-icon><VideoPlay /></el-icon>
              开始转会
            </el-button>
          </div>
        </div>
      </div>
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
        <li>每个赛区的转会期独立进行，可以逐轮执行或快进完成</li>
        <li>转会期间会自动处理合同续约、自由球员签约、球员挖角等事务</li>
        <li>转会完成后可查看详细的转会报告</li>
      </ul>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
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

// 获取赛区渐变色
function getRegionGradient(code: string): string {
  return regionColors[code.toUpperCase()] || 'linear-gradient(135deg, #6b7280, #4b5563)'
}

// 获取赛区球队数量
function getRegionTeamCount(regionId: number): number {
  return teamsByRegion.value.get(regionId)?.length ?? 0
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

// 是否可以开始转会
function canStartTransfer(region: Region): boolean {
  return getRegionGMStatus(region.id).allConfigured
}

// 点击赛区
function handleRegionClick(region: Region) {
  if (!canStartTransfer(region)) {
    ElMessage.warning('请先完成该赛区所有球队的GM配置')
    return
  }
  startRegionTransfer(region)
}

// 跳转到GM配置
function goToGMConfig(region: Region) {
  router.push({
    path: '/transfer/gm-config',
    query: { region: region.code.toLowerCase() }
  })
}

// 开始赛区转会
function startRegionTransfer(region: Region) {
  transferStore.setRegion(region.id, region.code)
  router.push(`/transfer/window/${region.code.toLowerCase()}`)
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
    console.error('Failed to load data:', e)
    ElMessage.error('加载数据失败')
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  loadData()
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

/* 赛区选择 */
.regions-section {
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

.regions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.region-card {
  background: white;
  border-radius: 16px;
  padding: 20px;
  border: 2px solid #e5e7eb;
  cursor: pointer;
  transition: all 0.3s ease;
}

.region-card:hover:not(.disabled) {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15);
  border-color: #3b82f6;
}

.region-card.disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.region-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.region-logo {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 18px;
  font-weight: 700;
  flex-shrink: 0;
}

.region-info h3 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 4px 0;
}

.region-info p {
  font-size: 13px;
  color: #909399;
  margin: 0;
}

.region-status {
  margin-bottom: 16px;
}

.region-status .el-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.region-status .tag-content {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.region-actions {
  display: flex;
  gap: 8px;
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
