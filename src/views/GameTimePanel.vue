<template>
  <div class="game-time-panel">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>时间控制面板</h1>
        <p>统一管理游戏时间推进</p>
      </div>
    </div>

    <!-- 加载状态 -->
    <el-skeleton v-if="isLoading && !timeState" :rows="10" animated />

    <!-- 主要内容 -->
    <div v-else-if="timeState" class="content-wrapper">
      <!-- 赛季进度 -->
      <div class="section-box season-progress-section">
        <div class="section-header">
          <span class="section-title">第 {{ currentSeason }} 赛季进度</span>
          <span class="progress-badge">{{ seasonProgress.toFixed(0) }}%</span>
        </div>

        <!-- 进度条 -->
        <el-progress
          :percentage="seasonProgress"
          :stroke-width="10"
          :show-text="false"
          color="#6366f1"
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
      </div>

      <!-- 当前阶段 + 操作按钮 -->
      <div class="two-col-row">
        <!-- 当前阶段状态 -->
        <div class="section-box">
          <div class="section-header">
            <span class="section-title">当前阶段</span>
          </div>

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
        </div>

        <!-- 操作按钮区 -->
        <div class="section-box">
          <div class="section-header">
            <span class="section-title">快速操作</span>
          </div>

          <div class="action-buttons">
            <div class="action-row">
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
            </div>

            <!-- 赛季总结操作 -->
            <template v-if="isSeasonEnd">
              <div class="action-divider"></div>
              <div class="action-row">
                <el-button
                  type="success"
                  size="large"
                  :icon="RefreshRight"
                  :disabled="isLoading"
                  @click="handleStartNewSeason"
                  class="action-btn full-width"
                >
                  确认进入新赛季
                </el-button>
              </div>
            </template>
          </div>
        </div>
      </div>

      <!-- 最近比赛结果 -->
      <div v-if="lastSimulatedMatch" class="section-box">
        <div class="section-header">
          <span class="section-title">
            <el-icon><Trophy /></el-icon>
            最近比赛结果
          </span>
        </div>

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

        <div class="section-divider"></div>

        <div class="match-info">
          <span>{{ lastSimulatedMatch.tournament_name }}</span>
          <span class="remaining-badge">剩余 {{ lastSimulatedMatch.remaining_matches }} 场比赛</span>
        </div>

        <div v-if="lastSimulatedMatch.phase_completed" class="phase-complete-notice">
          <span>✓ 当前阶段所有比赛已完成！</span>
        </div>
      </div>

      <!-- 年度颁奖典礼特殊卡片 -->
      <div v-if="isAnnualAwardsPhase" class="section-box awards-ceremony-section">
        <div class="section-header">
          <span class="section-title">
            <el-icon><Trophy /></el-icon>
            年度颁奖典礼
          </span>
          <span class="special-badge">特别活动</span>
        </div>

        <div class="awards-ceremony-content">
          <div class="ceremony-icon">🏆</div>
          <div class="ceremony-info">
            <h3>第 {{ currentSeason }} 赛季年度颁奖典礼</h3>
            <p>本赛季的精彩已落幕，是时候表彰那些在赛场上闪耀的选手们了！</p>
            <div class="awards-list">
              <div class="award-item">
                <span class="award-icon">👑</span>
                <span class="award-name">年度MVP + Top20选手</span>
              </div>
              <div class="award-item">
                <span class="award-icon">🏅</span>
                <span class="award-name">最佳阵容一/二/三阵</span>
              </div>
              <div class="award-item">
                <span class="award-icon">🌱</span>
                <span class="award-name">最佳新秀</span>
              </div>
              <div class="award-item">
                <span class="award-icon">⭐</span>
                <span class="award-name">最稳定 / 最具统治力</span>
              </div>
            </div>
          </div>
          <div class="ceremony-action">
            <router-link to="/annual-awards">
              <button class="ceremony-btn">进入颁奖典礼 →</button>
            </router-link>
          </div>
        </div>

        <div class="notice-bar ceremony-notice">
          点击「完成并推进」将颁发所有年度荣誉并进入转会期
        </div>
      </div>

      <!-- 赛事进度详情 -->
      <div v-if="tournaments.length > 0" class="section-box tournament-table">
        <div class="section-header">
          <span class="section-title">赛事进度详情</span>
        </div>

        <el-table :data="tournaments">
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
      </div>

      <!-- 最近颁发的荣誉 -->
      <div v-if="recentHonors.length > 0" class="section-box">
        <div class="section-header">
          <span class="section-title">
            <el-icon><Medal /></el-icon>
            最近颁发的荣誉
          </span>
        </div>

        <div class="honors-grid">
          <div v-for="(honor, index) in recentHonors" :key="index" class="honor-card">
            <div class="honor-icon">{{ getHonorIcon(honor.honor_type) }}</div>
            <div class="honor-info">
              <div class="honor-recipient">{{ honor.recipient_name }}</div>
              <div class="honor-type">{{ formatHonorType(honor.honor_type) }}</div>
              <div class="honor-tournament">{{ honor.tournament_name }}</div>
            </div>
          </div>
        </div>
      </div>
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
  return timeStore.isAnnualAwardsPhase
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
    '斗鱼巅峰赛': '斗鱼',
    '夏季常规赛': '夏季',
    '夏季季后赛': '夏季后',
    'Claude洲际赛': 'Claude',
    '世界赛': '世界赛',
    '抖音巅峰赛': '抖音',
    '上海大师赛': '上海',
    'ICP洲际对抗赛': 'ICP',
    'Super洲际邀请赛': 'Super',
    '虎牙巅峰赛': '虎牙',
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

/* 页面标题 */
.page-header { margin-bottom: 20px; }
.page-header h1 {
  font-size: 24px; font-weight: 700; color: #0f172a;
  margin: 0 0 4px 0; letter-spacing: -0.3px;
}
.page-header p { font-size: 13px; color: #94a3b8; margin: 0; }

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 通用区块 */
.section-box {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 16px;
  background: #ffffff;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: #0f172a;
  display: flex;
  align-items: center;
  gap: 6px;
}

.section-title .el-icon {
  font-size: 16px;
  color: #6366f1;
}

.section-divider {
  height: 1px;
  background: #f1f5f9;
  margin: 16px 0;
}

/* 进度徽章 */
.progress-badge {
  font-size: 13px;
  font-weight: 600;
  color: #6366f1;
  background: #eef2ff;
  padding: 2px 10px;
  border-radius: 6px;
}

/* 赛季进度 */
.season-progress-section .el-progress {
  margin-bottom: 16px;
}

/* 阶段时间线 */
.phase-timeline {
  margin-top: 16px;
}

.phase-items {
  display: flex;
  gap: 0;
}

.phase-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  padding: 8px 2px;
  border-radius: 6px;
  transition: background-color 0.15s;
}

.phase-item:hover {
  background: #f8fafc;
}

.phase-dot {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  margin-bottom: 6px;
  transition: all 0.15s;
}

.phase-item.is-completed .phase-dot {
  background: #10b981;
  color: white;
}

.phase-item.is-current .phase-dot {
  background: #6366f1;
  color: white;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
}

.phase-item.is-upcoming .phase-dot {
  background: #f1f5f9;
  color: #94a3b8;
}

.phase-name {
  font-size: 11px;
  color: #64748b;
  text-align: center;
  white-space: nowrap;
  line-height: 1.3;
}

.phase-item.is-current .phase-name {
  color: #6366f1;
  font-weight: 600;
}

.phase-item.is-completed .phase-name {
  color: #10b981;
}

/* 两列布局 */
.two-col-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

/* 描述列表样式 */
.section-box :deep(.el-descriptions) {
  --el-descriptions-table-border: 1px solid #f1f5f9;
}

.section-box :deep(.el-descriptions__label) {
  font-weight: 500;
  color: #94a3b8;
  font-size: 13px;
  background: #f8fafc;
  width: 100px;
}

.section-box :deep(.el-descriptions__content) {
  color: #0f172a;
  font-size: 13px;
}

/* 比赛进度 */
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
  color: #64748b;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
}

/* 操作按钮 */
.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.action-row {
  display: flex;
  gap: 12px;
}

.action-btn {
  flex: 1;
  height: 48px;
}

.action-btn.full-width {
  width: 100%;
}

.action-divider {
  height: 1px;
  background: #f1f5f9;
  margin: 4px 0;
}

/* 最近比赛结果 */
.match-result {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 40px;
  padding: 16px 0;
}

.match-result .team {
  display: flex;
  align-items: center;
  gap: 20px;
}

.match-result .team-name {
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.match-result .score {
  font-size: 36px;
  font-weight: 700;
  color: #cbd5e1;
  font-variant-numeric: tabular-nums;
}

.match-result .score.winner {
  color: #10b981;
}

.match-result .vs {
  font-size: 18px;
  font-weight: 600;
  color: #cbd5e1;
}

.match-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: #64748b;
  font-size: 13px;
}

.remaining-badge {
  font-size: 12px;
  color: #6366f1;
  background: #eef2ff;
  padding: 2px 10px;
  border-radius: 6px;
  font-weight: 500;
}

.phase-complete-notice {
  margin-top: 12px;
  padding: 10px 16px;
  background: #f0fdf4;
  border-left: 3px solid #10b981;
  border-radius: 0 6px 6px 0;
  font-size: 13px;
  color: #166534;
  font-weight: 500;
}

/* 年度颁奖典礼 */
.awards-ceremony-section {
  border-color: #6366f1;
  border-width: 1px 1px 1px 3px;
}

.special-badge {
  font-size: 11px;
  font-weight: 600;
  color: #f59e0b;
  background: #fffbeb;
  padding: 2px 10px;
  border-radius: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.awards-ceremony-content {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 8px 0 16px;
}

.ceremony-icon {
  font-size: 48px;
  flex-shrink: 0;
}

.ceremony-info {
  flex: 1;
}

.ceremony-info h3 {
  font-size: 18px;
  font-weight: 700;
  margin: 0 0 6px 0;
  color: #0f172a;
}

.ceremony-info p {
  margin: 0 0 14px 0;
  color: #64748b;
  font-size: 13px;
}

.awards-list {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.award-item {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 13px;
}

.award-icon {
  font-size: 14px;
}

.award-name {
  color: #475569;
  font-weight: 500;
}

.ceremony-action {
  flex-shrink: 0;
}

.ceremony-btn {
  padding: 10px 24px;
  border: 1px solid #6366f1;
  border-radius: 6px;
  background: #6366f1;
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.ceremony-btn:hover {
  background: #4f46e5;
  border-color: #4f46e5;
}

.notice-bar {
  padding: 10px 16px;
  background: #f8fafc;
  border-left: 3px solid #6366f1;
  border-radius: 0 8px 8px 0;
  font-size: 13px;
  color: #475569;
  line-height: 1.6;
}

.ceremony-notice {
  margin-top: 8px;
}

/* 赛事进度表格 */
.tournament-table :deep(.el-table) {
  --el-table-border-color: #f1f5f9;
  --el-table-header-bg-color: transparent;
  --el-table-row-hover-bg-color: #f8fafc;
}

.tournament-table :deep(.el-table th.el-table__cell) {
  font-weight: 600;
  color: #94a3b8;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: transparent;
  border-bottom: 1px solid #f1f5f9;
  padding: 10px 0;
}

.tournament-table :deep(.el-table__body tr) {
  transition: background-color 0.15s;
}

.tournament-table :deep(.el-table__body tr td) {
  padding: 12px 0;
  border-bottom: 1px solid #f8fafc;
}

.tournament-table :deep(.el-table__body tr:hover > td) {
  background-color: #f8fafc !important;
}

.tournament-table :deep(.el-table__body tr:last-child td) {
  border-bottom: none;
}

/* 荣誉网格 */
.honors-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.honor-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #f8fafc;
  border: 1px solid #f1f5f9;
  border-radius: 8px;
}

.honor-icon {
  font-size: 28px;
  flex-shrink: 0;
}

.honor-info {
  flex: 1;
  min-width: 0;
}

.honor-recipient {
  font-weight: 600;
  color: #0f172a;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.honor-type {
  font-size: 12px;
  color: #64748b;
}

.honor-tournament {
  font-size: 12px;
  color: #94a3b8;
}

/* 加载遮罩 */
.loading-overlay {
  position: fixed;
  inset: 0;
  background: rgba(255, 255, 255, 0.8);
  z-index: 1000;
}

/* 响应式 */
@media (max-width: 1200px) {
  .two-col-row {
    grid-template-columns: 1fr;
  }

  .honors-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .honors-grid {
    grid-template-columns: 1fr;
  }

  .awards-ceremony-content {
    flex-direction: column;
    text-align: center;
  }
}
</style>
