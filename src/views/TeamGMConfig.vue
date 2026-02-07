<template>
  <div class="gm-config-view">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push('/transfer')">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回转会系统</span>
      </button>
    </div>

    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>AI GM 配置</h1>
        <p>{{ regionName }} - 配置每支球队的 AI 总经理性格</p>
      </div>
      <div class="header-actions">
        <el-tag :type="allConfigured ? 'success' : 'warning'" size="large" effect="dark">
          {{ configuredCount }}/{{ teams.length }} 已配置
        </el-tag>
      </div>
    </div>

    <!-- 配置说明 -->
    <el-card class="intro-card">
      <div class="intro-content">
        <div class="intro-icon">
          <el-icon :size="40"><UserFilled /></el-icon>
        </div>
        <div class="intro-text">
          <h3>AI 性格说明</h3>
          <p>每种性格会影响球队在转会期的决策偏好，包括续约谈判、球员招募、价格敏感度等。</p>
          <div class="personality-legend">
            <div
              v-for="(config, key) in PERSONALITY_CONFIG"
              :key="key"
              class="legend-item"
            >
              <span class="legend-dot" :style="{ background: config.color }"></span>
              <span class="legend-label">{{ config.label }}</span>
              <span class="legend-desc">{{ config.description }}</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 赛区选择 -->
    <div class="region-selector">
      <el-radio-group v-model="selectedRegion" @change="loadTeams">
        <el-radio-button
          v-for="region in regions"
          :key="region.code"
          :value="region.code.toLowerCase()"
        >
          {{ region.code }}
        </el-radio-button>
      </el-radio-group>
    </div>

    <!-- 球队列表 -->
    <div class="teams-grid">
      <div
        v-for="team in teams"
        :key="team.id"
        class="team-card"
        :class="{ configured: isTeamConfigured(team.id) }"
      >
        <div class="team-header">
          <div class="team-info">
            <h3>{{ team.name }}</h3>
            <p>{{ team.short_name }}</p>
          </div>
          <el-tag
            v-if="isTeamConfigured(team.id)"
            type="success"
            size="small"
          >
            <el-icon><Check /></el-icon>
            已配置
          </el-tag>
          <el-tag v-else type="info" size="small">未配置</el-tag>
        </div>

        <!-- 当前性格 -->
        <div class="current-personality">
          <span class="label">当前性格:</span>
          <template v-if="getTeamPersonality(team.id)">
            <span
              class="personality-badge"
              :style="{ background: getPersonalityColor(getTeamPersonality(team.id)!) }"
            >
              {{ getPersonalityLabel(getTeamPersonality(team.id)!) }}
            </span>
          </template>
          <span v-else class="no-config">未设置</span>
        </div>

        <!-- 性格选择 -->
        <div class="personality-selector">
          <el-select
            v-model="teamPersonalitySelections[team.id]"
            placeholder="选择性格"
            style="width: 100%"
          >
            <el-option
              v-for="(config, key) in PERSONALITY_CONFIG"
              :key="key"
              :label="config.label"
              :value="key"
            >
              <div class="personality-option">
                <span class="option-dot" :style="{ background: config.color }"></span>
                <span class="option-label">{{ config.label }}</span>
                <span class="option-desc">{{ config.description }}</span>
              </div>
            </el-option>
          </el-select>
        </div>

        <!-- 保存按钮 -->
        <el-button
          type="primary"
          size="small"
          :loading="savingTeamId === team.id"
          :disabled="!teamPersonalitySelections[team.id]"
          @click="saveTeamPersonality(team.id)"
          style="width: 100%"
        >
          {{ isTeamConfigured(team.id) ? '更新配置' : '保存配置' }}
        </el-button>
      </div>
    </div>

    <!-- 批量操作 -->
    <el-card class="batch-card">
      <div class="batch-content">
        <div class="batch-info">
          <h3>批量配置</h3>
          <p>为所有未配置的球队设置默认性格</p>
        </div>
        <div class="batch-actions">
          <el-select v-model="batchPersonality" placeholder="选择性格" style="width: 160px">
            <el-option
              v-for="(config, key) in PERSONALITY_CONFIG"
              :key="key"
              :label="config.label"
              :value="key"
            />
          </el-select>
          <el-button
            type="primary"
            :loading="isBatchSaving"
            :disabled="!batchPersonality"
            @click="batchConfigureAll"
          >
            应用到未配置球队
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 确认按钮 -->
    <div class="action-bar">
      <el-button
        v-if="allConfigured"
        type="success"
        size="large"
        @click="confirmAndReturn"
      >
        <el-icon><Check /></el-icon>
        确认配置完成，返回转会系统
      </el-button>
      <el-button v-else type="info" size="large" disabled>
        请先完成所有球队的配置
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  ArrowLeft,
  UserFilled,
  Check,
} from '@element-plus/icons-vue'
import { useTransferWindowStore, PERSONALITY_CONFIG } from '@/stores/useTransferWindowStore'
import { queryApi } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TeamGMConfig')

interface Region {
  id: number
  code: string
  name: string
}

interface Team {
  id: number
  name: string
  short_name: string | null
  region_id: number
}

const route = useRoute()
const router = useRouter()
const transferStore = useTransferWindowStore()

// 状态
const regions = ref<Region[]>([])
const teams = ref<Team[]>([])
const selectedRegion = ref<string>('')
const teamPersonalitySelections = ref<Record<number, string>>({})
const savingTeamId = ref<number | null>(null)
const batchPersonality = ref<string>('')
const isBatchSaving = ref(false)

// 计算属性
const regionName = computed(() => {
  const region = regions.value.find(r => r.code.toLowerCase() === selectedRegion.value)
  return region?.name ?? selectedRegion.value.toUpperCase()
})

const configuredCount = computed(() => {
  return teams.value.filter(t => isTeamConfigured(t.id)).length
})

const allConfigured = computed(() => {
  return teams.value.length > 0 && configuredCount.value === teams.value.length
})

// 检查球队是否已配置
function isTeamConfigured(teamId: number): boolean {
  return transferStore.teamPersonalities.has(teamId)
}

// 获取球队性格
function getTeamPersonality(teamId: number): string | null {
  const config = transferStore.teamPersonalities.get(teamId)
  return config?.personality ?? null
}

// 获取性格标签
function getPersonalityLabel(personality: string): string {
  return PERSONALITY_CONFIG[personality]?.label ?? personality
}

// 获取性格颜色
function getPersonalityColor(personality: string): string {
  return PERSONALITY_CONFIG[personality]?.color ?? '#6b7280'
}

// 加载赛区
async function loadRegions() {
  try {
    const allRegions = await queryApi.getAllRegions()
    regions.value = allRegions.filter(r =>
      ['LPL', 'LCK', 'LEC', 'LCS'].includes(r.code.toUpperCase())
    )

    // 从 URL 参数获取初始赛区
    const regionParam = route.query.region as string
    if (regionParam && regions.value.some(r => r.code.toLowerCase() === regionParam.toLowerCase())) {
      selectedRegion.value = regionParam.toLowerCase()
    } else if (regions.value.length > 0) {
      selectedRegion.value = regions.value[0].code.toLowerCase()
    }
  } catch (e) {
    logger.error('Failed to load regions:', e)
    ElMessage.error('加载赛区失败')
  }
}

// 加载球队
async function loadTeams() {
  if (!selectedRegion.value) return

  try {
    const region = regions.value.find(r => r.code.toLowerCase() === selectedRegion.value)
    if (!region) return

    teams.value = await queryApi.getTeamsByRegion(region.id)

    // 加载已有配置
    for (const team of teams.value) {
      const config = await transferStore.loadTeamPersonality(team.id)
      if (config) {
        teamPersonalitySelections.value[team.id] = config.personality
      }
    }
  } catch (e) {
    logger.error('Failed to load teams:', e)
    ElMessage.error('加载球队失败')
  }
}

// 保存单个球队配置
async function saveTeamPersonality(teamId: number) {
  const personality = teamPersonalitySelections.value[teamId]
  if (!personality) return

  savingTeamId.value = teamId

  try {
    const success = await transferStore.updateTeamPersonality(teamId, {
      personality,
    })

    if (success) {
      ElMessage.success('配置已保存')
    } else {
      ElMessage.error('保存失败')
    }
  } catch (e) {
    logger.error('Failed to save personality:', e)
    ElMessage.error('保存失败')
  } finally {
    savingTeamId.value = null
  }
}

// 批量配置所有未配置球队
async function batchConfigureAll() {
  if (!batchPersonality.value) return

  isBatchSaving.value = true

  try {
    const unconfiguredTeams = teams.value.filter(t => !isTeamConfigured(t.id))
    let successCount = 0

    for (const team of unconfiguredTeams) {
      const success = await transferStore.updateTeamPersonality(team.id, {
        personality: batchPersonality.value,
      })
      if (success) {
        teamPersonalitySelections.value[team.id] = batchPersonality.value
        successCount++
      }
    }

    ElMessage.success(`已为 ${successCount} 支球队配置性格`)
  } catch (e) {
    logger.error('Failed to batch configure:', e)
    ElMessage.error('批量配置失败')
  } finally {
    isBatchSaving.value = false
  }
}

// 确认并返回
function confirmAndReturn() {
  transferStore.confirmGMConfig()
  ElMessage.success('配置已确认')
  router.push('/transfer')
}

// 监听赛区变化
watch(selectedRegion, () => {
  loadTeams()
})

onMounted(async () => {
  await loadRegions()
  await loadTeams()
})
</script>

<style scoped>
.gm-config-view {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
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

/* 介绍卡片 */
.intro-card {
  margin-bottom: 24px;
  border-radius: 12px;
}

.intro-content {
  display: flex;
  gap: 20px;
}

.intro-icon {
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.intro-text {
  flex: 1;
}

.intro-text h3 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 8px 0;
}

.intro-text p {
  font-size: 14px;
  color: #606266;
  margin: 0 0 16px 0;
}

.personality-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.legend-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

.legend-label {
  font-size: 13px;
  font-weight: 600;
  color: #303133;
}

.legend-desc {
  font-size: 12px;
  color: #909399;
}

/* 赛区选择 */
.region-selector {
  margin-bottom: 20px;
}

/* 球队网格 */
.teams-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.team-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  border: 2px solid #e5e7eb;
  transition: all 0.3s ease;
}

.team-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.team-card.configured {
  border-color: #22c55e;
  background: linear-gradient(135deg, #f0fdf4, #ffffff);
}

.team-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.team-info h3 {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 4px 0;
}

.team-info p {
  font-size: 12px;
  color: #909399;
  margin: 0;
}

.current-personality {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.current-personality .label {
  font-size: 13px;
  color: #909399;
}

.personality-badge {
  padding: 4px 10px;
  border-radius: 12px;
  color: white;
  font-size: 12px;
  font-weight: 600;
}

.no-config {
  font-size: 13px;
  color: #c0c4cc;
}

.personality-selector {
  margin-bottom: 12px;
}

.personality-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.option-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.option-label {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.option-desc {
  font-size: 12px;
  color: #909399;
  margin-left: auto;
}

/* 批量配置 */
.batch-card {
  border-radius: 12px;
  margin-bottom: 24px;
}

.batch-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.batch-info h3 {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 4px 0;
}

.batch-info p {
  font-size: 13px;
  color: #909399;
  margin: 0;
}

.batch-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

/* 操作按钮 */
.action-bar {
  display: flex;
  justify-content: center;
}
</style>
