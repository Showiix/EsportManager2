<template>
  <div class="game-time-panel">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1>时间控制面板</h1>
      <p>统一管理游戏时间推进</p>
    </div>

    <!-- 加载状态 -->
    <el-skeleton v-if="isLoading && !timeState" :rows="10" animated />

    <!-- 主要内容 -->
    <div v-else-if="timeState" class="content-wrapper">
      <!-- 赛季进度卡片 -->
      <el-card class="season-progress-card" shadow="hover">
        <template #header>
          <div class="card-header">
            <span>第 {{ currentSeason }} 赛季进度</span>
            <el-tag type="primary">{{ seasonProgress.toFixed(0) }}%</el-tag>
          </div>
        </template>

        <!-- 进度条 -->
        <el-progress
          :percentage="seasonProgress"
          :stroke-width="12"
          :show-text="false"
          status="success"
        />

        <!-- 阶段时间线 -->
        <div class="phase-timeline">
          <div class="phase-items">
            <div
              v-for="(phase, index) in allPhases"
              :key="phase.phase"
              class="phase-item"
              :class="getPhaseItemClass(phase.status)"
            >
              <div class="phase-dot">
                <span v-if="phase.status === 'completed'">✓</span>
                <span v-else>{{ index + 1 }}</span>
              </div>
              <div class="phase-name">{{ getShortPhaseName(phase.display_name) }}</div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 当前阶段 + 操作按钮 -->
      <el-row :gutter="20" class="main-row">
        <!-- 当前阶段状态卡片 -->
        <el-col :span="12">
          <el-card shadow="hover">
            <template #header>
              <div class="card-header">
                <span>当前阶段</span>
              </div>
            </template>

            <el-descriptions :column="1" border>
              <el-descriptions-item label="阶段名称">
                <el-tag size="large" effect="dark" type="primary">{{ phaseDisplayName }}</el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="赛季">
                第 {{ currentSeason }} 赛季
              </el-descriptions-item>
              <el-descriptions-item label="状态">
                <el-tag :type="getStatusTagType(phaseStatus)">
                  {{ getStatusText(phaseStatus) }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item v-if="totalMatches > 0" label="比赛进度">
                <div class="match-progress">
                  <el-progress
                    :percentage="phaseProgress"
                    :status="phaseProgress >= 100 ? 'success' : ''"
                  />
                  <span class="progress-text">{{ completedMatches }} / {{ totalMatches }}</span>
                </div>
              </el-descriptions-item>
              <el-descriptions-item v-if="nextPhase" label="下一阶段">
                {{ nextPhase }}
              </el-descriptions-item>
            </el-descriptions>
          </el-card>
        </el-col>

        <!-- 操作按钮区 -->
        <el-col :span="12">
          <el-card shadow="hover">
            <template #header>
              <div class="card-header">
                <span>快速操作</span>
              </div>
            </template>

            <div class="action-buttons">
              <el-row :gutter="12">
                <el-col :span="12">
                  <el-button
                    type="primary"
                    size="large"
                    :icon="Promotion"
                    :disabled="!canDoAction('INITIALIZE_PHASE') || isLoading"
                    @click="handleInitPhase"
                    class="action-btn"
                  >
                    初始化阶段
                  </el-button>
                </el-col>
                <el-col :span="12">
                  <el-button
                    type="success"
                    size="large"
                    :icon="CircleCheck"
                    :disabled="!canAdvance || isLoading"
                    @click="handleCompleteAndAdvance"
                    class="action-btn"
                  >
                    完成并推进
                  </el-button>
                </el-col>
              </el-row>

              <!-- 赛季总结操作 -->
              <el-divider v-if="isSeasonEnd" />
              <el-row v-if="isSeasonEnd" :gutter="12">
                <el-col :span="24">
                  <el-button
                    type="success"
                    size="large"
                    :icon="RefreshRight"
                    :disabled="isLoading"
                    @click="handleStartNewSeason"
                    class="action-btn"
                  >
                    确认进入新赛季
                  </el-button>
                </el-col>
              </el-row>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 最近比赛结果 -->
      <el-card v-if="lastSimulatedMatch" shadow="hover" class="match-result-card">
        <template #header>
          <div class="card-header">
            <el-icon><Trophy /></el-icon>
            <span>最近比赛结果</span>
          </div>
        </template>

        <div class="match-result">
          <div class="team home-team">
            <span class="team-name">{{ lastSimulatedMatch.home_team_name }}</span>
            <span
              class="score"
              :class="{ winner: lastSimulatedMatch.home_score > lastSimulatedMatch.away_score }"
            >
              {{ lastSimulatedMatch.home_score }}
            </span>
          </div>
          <div class="vs">VS</div>
          <div class="team away-team">
            <span
              class="score"
              :class="{ winner: lastSimulatedMatch.away_score > lastSimulatedMatch.home_score }"
            >
              {{ lastSimulatedMatch.away_score }}
            </span>
            <span class="team-name">{{ lastSimulatedMatch.away_team_name }}</span>
          </div>
        </div>

        <el-divider />

        <div class="match-info">
          <span>{{ lastSimulatedMatch.tournament_name }}</span>
          <el-tag size="small">剩余 {{ lastSimulatedMatch.remaining_matches }} 场比赛</el-tag>
        </div>

        <el-alert
          v-if="lastSimulatedMatch.phase_completed"
          title="当前阶段所有比赛已完成！"
          type="success"
          show-icon
          :closable="false"
          class="phase-complete-alert"
        />
      </el-card>

      <!-- 年度颁奖典礼特殊卡片 -->
      <el-card v-if="isAnnualAwardsPhase" shadow="hover" class="awards-ceremony-card">
        <template #header>
          <div class="card-header">
            <el-icon><Trophy /></el-icon>
            <span>年度颁奖典礼</span>
            <el-tag type="warning" effect="dark">特别活动</el-tag>
          </div>
        </template>

        <div class="awards-ceremony-content">
          <div class="ceremony-icon"><el-icon :size="48" color="#f59e0b"><Trophy /></el-icon></div>
          <div class="ceremony-info">
            <h3>第 {{ currentSeason }} 赛季年度颁奖典礼</h3>
            <p>本赛季的精彩已落幕，是时候表彰那些在赛场上闪耀的选手们了！</p>
            <div class="awards-list">
              <div class="award-item">
                <span class="award-icon">👑</span>
                <span class="award-name">年度MVP</span>
              </div>
              <div class="award-item">
                <el-icon class="award-icon" color="#f59e0b"><StarFilled /></el-icon>
                <span class="award-name">年度Top20选手</span>
              </div>
              <div class="award-item">
                <el-icon class="award-icon" color="#3b82f6"><Medal /></el-icon>
                <span class="award-name">各位置最佳选手</span>
              </div>
              <div class="award-item">
                <span class="award-icon">🌱</span>
                <span class="award-name">年度最佳新秀</span>
              </div>
            </div>
          </div>
          <div class="ceremony-action">
            <router-link to="/annual-awards">
              <el-button type="primary" size="large" :icon="Trophy">
                进入颁奖典礼
              </el-button>
            </router-link>
          </div>
        </div>

        <el-alert
          title="点击「完成并推进」将颁发所有年度荣誉并进入转会期"
          type="info"
          show-icon
          :closable="false"
          class="ceremony-alert"
        />
      </el-card>

      <!-- 赛事进度详情 -->
      <el-card v-if="tournaments.length > 0" shadow="hover">
        <template #header>
          <div class="card-header">
            <span>赛事进度详情</span>
          </div>
        </template>

        <el-table :data="tournaments" stripe>
          <el-table-column prop="tournament_name" label="赛事名称" />
          <el-table-column prop="region" label="地区" width="100">
            <template #default="{ row }">
              {{ row.region || '-' }}
            </template>
          </el-table-column>
          <el-table-column label="进度" width="200">
            <template #default="{ row }">
              <el-progress
                :percentage="getTournamentProgressPercent(row)"
                :status="row.completed_matches >= row.total_matches ? 'success' : ''"
              />
            </template>
          </el-table-column>
          <el-table-column label="比赛数" width="120">
            <template #default="{ row }">
              {{ row.completed_matches }} / {{ row.total_matches }}
            </template>
          </el-table-column>
          <el-table-column prop="status" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="getTournamentStatusType(row.status)" size="small">
                {{ getTournamentStatusText(row.status) }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <!-- 最近颁发的荣誉 -->
      <el-card v-if="recentHonors.length > 0" shadow="hover">
        <template #header>
          <div class="card-header">
            <el-icon><Medal /></el-icon>
            <span>最近颁发的荣誉</span>
          </div>
        </template>

        <el-row :gutter="12">
          <el-col v-for="(honor, index) in recentHonors" :key="index" :span="8">
            <el-card shadow="never" class="honor-card">
              <div class="honor-icon">{{ getHonorIcon(honor.honor_type) }}</div>
              <div class="honor-info">
                <div class="honor-recipient">{{ honor.recipient_name }}</div>
                <div class="honor-type">{{ formatHonorType(honor.honor_type) }}</div>
                <div class="honor-tournament">{{ honor.tournament_name }}</div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-card>
    </div>

    <!-- 无数据状态 -->
    <el-empty v-else description="无法加载时间状态">
      <el-button type="primary" @click="fetchTimeState">重新加载</el-button>
    </el-empty>

    <!-- 加载遮罩 -->
    <div v-if="isLoading && timeState" class="loading-overlay" v-loading="true" element-loading-text="处理中..." />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useTimeStore } from '@/stores/useTimeStore'
import type { TimeAction } from '@/api/tauri'
import { formatHonorType } from '@/api/tauri'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Promotion,
  CircleCheck,
  RefreshRight,
  Trophy,
  Medal,
  StarFilled,
} from '@element-plus/icons-vue'

const timeStore = useTimeStore()
const {
  timeState,
  isLoading,
  recentHonors,
  lastSimulatedMatch,
  currentSeason,
  phaseDisplayName,
  phaseStatus,
  phaseProgress,
  seasonProgress,
  canAdvance,
  nextPhase,
  availableActions,
  tournaments,
  completedMatches,
  totalMatches,
  allPhases,
  isSeasonEnd,
} = storeToRefs(timeStore)

const { fetchTimeState } = timeStore

// 判断是否是年度颁奖典礼阶段
const isAnnualAwardsPhase = computed(() => {
  return phaseDisplayName.value === '年度颁奖典礼'
})

// 判断是否可以执行某个操作
const canDoAction = (action: TimeAction) => {
  return availableActions.value.includes(action)
}

// 获取阶段项样式类
const getPhaseItemClass = (status: string) => {
  return {
    'is-completed': status === 'completed',
    'is-current': status === 'current',
    'is-upcoming': status === 'upcoming',
  }
}

// 获取状态标签类型
const getStatusTagType = (status: string) => {
  switch (status) {
    case 'COMPLETED': return 'success'
    case 'IN_PROGRESS': return 'primary'
    default: return 'warning'
  }
}

// 获取状态文本
const getStatusText = (status: string) => {
  switch (status) {
    case 'NOT_INITIALIZED': return '未初始化'
    case 'IN_PROGRESS': return '进行中'
    case 'COMPLETED': return '已完成'
    default: return status
  }
}

// 获取简短阶段名
const getShortPhaseName = (name: string) => {
  const map: Record<string, string> = {
    '春季常规赛': '春季',
    '春季季后赛': '春季后',
    'MSI季中赛': 'MSI',
    '马德里大师赛': '马德里',
    '夏季常规赛': '夏季',
    '夏季季后赛': '夏季后',
    'Claude洲际赛': 'Claude',
    '世界赛': '世界赛',
    '上海大师赛': '上海',
    'ICP洲际对抗赛': 'ICP',
    'Super洲际邀请赛': 'Super',
    '年度颁奖典礼': '颁奖',
    '转会期': '转会',
    '选秀大会': '选秀',
    '赛季总结': '总结',
  }
  return map[name] || name.slice(0, 3)
}

// 赛事进度百分比
const getTournamentProgressPercent = (tournament: { completed_matches: number; total_matches: number }) => {
  return tournament.total_matches > 0
    ? Math.round((tournament.completed_matches / tournament.total_matches) * 100)
    : 0
}

// 赛事状态类型
const getTournamentStatusType = (status: string) => {
  switch (status) {
    case 'completed': return 'success'
    case 'in_progress': return 'primary'
    default: return 'info'
  }
}

// 赛事状态文本
const getTournamentStatusText = (status: string) => {
  switch (status) {
    case 'completed': return '已完成'
    case 'in_progress': return '进行中'
    case 'upcoming': return '未开始'
    default: return status
  }
}

// 荣誉图标
const getHonorIcon = (type: string) => {
  if (type.includes('冠军')) return '🥇'
  if (type.includes('亚军')) return '🥈'
  if (type.includes('季军')) return '🥉'
  if (type.includes('年度MVP')) return '👑'
  if (type.includes('年度Top20')) return '🌟'
  if (type.includes('年度最佳')) return '🏅'
  if (type.includes('年度新秀')) return '🌱'
  if (type.includes('MVP')) return '⭐'
  return '🏆'
}

// 操作处理函数
const handleInitPhase = async () => {
  try {
    await timeStore.initPhase()
    ElMessage.success('阶段初始化成功！')
  } catch (e) {
    ElMessage.error('初始化失败: ' + (e instanceof Error ? e.message : '未知错误'))
  }
}

const handleCompleteAndAdvance = async () => {
  try {
    const result = await timeStore.completeAndAdvance()
    // 显示成功弹窗
    const honorsText = result.honors_awarded.length > 0
      ? `\n\n颁发了 ${result.honors_awarded.length} 个荣誉`
      : ''
    ElMessageBox.alert(
      `${result.message}${honorsText}`,
      '推进成功',
      {
        confirmButtonText: '确定',
        type: 'success',
      }
    )
  } catch (e) {
    ElMessage.error('推进失败: ' + (e instanceof Error ? e.message : '未知错误'))
  }
}

const handleStartNewSeason = async () => {
  try {
    await ElMessageBox.confirm(
      '确认进入新赛季？系统将自动确认各队首发、更新战力并创建春季赛。',
      '确认进入新赛季',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'info',
      }
    )
    const result = await timeStore.startNewSeason()
    ElMessageBox.alert(
      result.message,
      '新赛季已开始',
      {
        confirmButtonText: '确定',
        type: 'success',
      }
    )
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') {
      ElMessage.error('开始新赛季失败: ' + (e instanceof Error ? e.message : '未知错误'))
    }
  }
}

// 自动刷新定时器
let refreshTimer: number | null = null

onMounted(async () => {
  await fetchTimeState()

  // 每5秒自动刷新状态
  refreshTimer = window.setInterval(() => {
    if (!isLoading.value) {
      fetchTimeState()
    }
  }, 5000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped>
.game-time-panel {
  padding: 0;
}

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  color: #909399;
  margin: 0;
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.card-header .el-icon {
  font-size: 18px;
  color: #409eff;
}

.season-progress-card .el-progress {
  margin-bottom: 20px;
}

.phase-timeline {
  margin-top: 20px;
  overflow-x: auto;
  padding-bottom: 8px;
}

.phase-items {
  display: flex;
  gap: 4px;
  min-width: max-content;
}

.phase-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 56px;
  padding: 8px 4px;
  border-radius: 8px;
  transition: all 0.2s;
}

.phase-item:hover {
  background: rgba(64, 158, 255, 0.05);
}

.phase-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 6px;
  transition: all 0.2s;
}

.phase-item.is-completed .phase-dot {
  background: #67c23a;
  color: white;
}

.phase-item.is-current .phase-dot {
  background: #409eff;
  color: white;
  box-shadow: 0 0 0 4px rgba(64, 158, 255, 0.2);
}

.phase-item.is-upcoming .phase-dot {
  background: #e4e7ed;
  color: #909399;
}

.phase-name {
  font-size: 11px;
  color: #606266;
  text-align: center;
  white-space: nowrap;
  line-height: 1.3;
}

.phase-item.is-current .phase-name {
  color: #409eff;
  font-weight: 600;
}

.phase-item.is-completed .phase-name {
  color: #67c23a;
}

.main-row {
  margin-top: 0;
}

.match-progress {
  display: flex;
  align-items: center;
  gap: 12px;
}

.match-progress .el-progress {
  flex: 1;
}

.progress-text {
  white-space: nowrap;
  color: #606266;
}

.action-buttons .el-row {
  margin-bottom: 12px;
}

.action-buttons .el-row:last-child {
  margin-bottom: 0;
}

.action-btn {
  width: 100%;
  height: 50px;
}

.match-result-card .match-result {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 40px;
  padding: 20px 0;
}

.match-result .team {
  display: flex;
  align-items: center;
  gap: 20px;
}

.match-result .team-name {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.match-result .score {
  font-size: 36px;
  font-weight: 700;
  color: #909399;
}

.match-result .score.winner {
  color: #67c23a;
}

.match-result .vs {
  font-size: 20px;
  font-weight: 600;
  color: #c0c4cc;
}

.match-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: #909399;
}

.phase-complete-alert {
  margin-top: 16px;
}

.honor-card {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.honor-card :deep(.el-card__body) {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
}

.honor-icon {
  font-size: 32px;
}

.honor-info {
  flex: 1;
  min-width: 0;
}

.honor-recipient {
  font-weight: 600;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.honor-type {
  font-size: 12px;
  color: #606266;
}

.honor-tournament {
  font-size: 12px;
  color: #909399;
}

.loading-overlay {
  position: fixed;
  inset: 0;
  background: rgba(255, 255, 255, 0.8);
  z-index: 1000;
}

/* 年度颁奖典礼卡片样式 */
.awards-ceremony-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

.awards-ceremony-card :deep(.el-card__header) {
  background: rgba(255, 255, 255, 0.1);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.awards-ceremony-card .card-header {
  color: white;
}

.awards-ceremony-card .card-header .el-icon {
  color: #ffd700;
}

.awards-ceremony-content {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 20px 0;
}

.ceremony-icon {
  font-size: 64px;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

.ceremony-info {
  flex: 1;
  color: #1a1a2e;
}

.ceremony-info h3 {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a2e;
}

.ceremony-info p {
  margin: 0 0 16px 0;
  color: #2d2d44;
}

.awards-list {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.award-item {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.15);
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 13px;
}

.award-icon {
  font-size: 16px;
}

.award-name {
  color: #1a1a2e;
}

.ceremony-action {
  flex-shrink: 0;
}

.ceremony-action .el-button {
  background: rgba(255, 255, 255, 0.2);
  border: 2px solid white;
  color: white;
  font-weight: 600;
  padding: 20px 32px;
  font-size: 16px;
}

.ceremony-action .el-button:hover {
  background: rgba(255, 255, 255, 0.3);
}

.ceremony-alert {
  margin-top: 16px;
  background: rgba(255, 255, 255, 0.9);
}
</style>
