<template>
  <div
    class="super-match-card"
    :class="[
      `status-${match.status}`,
      { 'is-completed': match.status === 'completed' }
    ]"
  >
    <!-- 比赛信息头部 -->
    <div class="match-header">
      <span class="badge" :class="'badge-' + getMatchTypeBadgeType()">
        {{ getMatchTypeLabel() }}
      </span>
      <span v-if="match.bestOf" class="badge badge-info">
        BO{{ match.bestOf }}
      </span>
    </div>

    <!-- 队伍对阵 -->
    <div class="teams-container">
      <div
        class="team-row"
        :class="{ winner: isWinner(match.teamAId) }"
      >
        <div class="team-info">
          <span class="team-name">
            {{ match.teamAName || '待定' }}
          </span>
          <span v-if="match.teamARegion" class="region-tag" :class="'region-' + (match.teamARegion || '').toLowerCase()">
            {{ match.teamARegion }}
          </span>
        </div>
        <div v-if="match.status === 'completed'" class="team-score">
          {{ match.scoreA }}
        </div>
      </div>

      <div class="vs-divider">VS</div>

      <div
        class="team-row"
        :class="{ winner: isWinner(match.teamBId) }"
      >
        <div class="team-info">
          <span class="team-name">
            {{ match.teamBName || '待定' }}
          </span>
          <span v-if="match.teamBRegion" class="region-tag" :class="'region-' + (match.teamBRegion || '').toLowerCase()">
            {{ match.teamBRegion }}
          </span>
        </div>
        <div v-if="match.status === 'completed'" class="team-score">
          {{ match.scoreB }}
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="match-actions">
      <button
        v-if="canSimulate"
        class="simulate-btn"
        :disabled="simulating"
        @click="handleSimulate"
      >
        {{ simulating ? '模拟中...' : '模拟比赛' }}
      </button>
      <div v-else-if="match.status === 'completed'" class="completed-actions">
        <span class="badge badge-success">已完成</span>
        <button class="detail-btn" @click="handleViewDetail">
          查看详情
        </button>
      </div>
      <span v-else class="badge badge-info">
        待确定对阵
      </span>
    </div>

    <!-- 完成时间 -->
    <div v-if="match.status === 'completed' && match.completedAt" class="match-footer">
      <span class="completed-time">
        完成时间: {{ formatDate(match.completedAt) }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SuperMatch } from '@/types/super'

interface Props {
  match: SuperMatch
  simulating?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  simulating: false
})

const emit = defineEmits<{
  (e: 'simulate', match: SuperMatch): void
  (e: 'view-detail', match: SuperMatch): void
}>()

/**
 * 是否可以模拟
 */
const canSimulate = computed(() => {
  return (
    props.match.id &&
    props.match.status !== 'completed' &&
    props.match.teamAId &&
    props.match.teamBId
  )
})

/**
 * 判断是否为胜者
 */
const isWinner = (teamId: string | number | undefined): boolean => {
  if (!teamId || !props.match.winnerId) return false
  return String(teamId) === String(props.match.winnerId)
}

/**
 * 获取赛区标签颜色
 */
const getRegionTagType = (region?: string): string => {
  const typeMap: Record<string, string> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning'
  }
  return typeMap[region || ''] || 'info'
}

/**
 * 获取比赛类型标签
 */
const getMatchTypeBadgeType = (): string => {
  if (props.match.stage === 'fighter_group') return 'primary'
  if (props.match.matchType === 'GRAND_FINAL') return 'danger'
  if (props.match.matchType === 'THIRD_PLACE') return 'warning'
  if (props.match.matchType === 'CHALLENGER_POSITIONING') return 'default'
  if (props.match.matchType === 'CHALLENGER_PROMOTION') return 'info'
  return 'success'
}

/**
 * 获取比赛类型标签文字
 */
const getMatchTypeLabel = (): string => {
  if (props.match.stage === 'fighter_group') {
    return `Fighter组 ${props.match.groupName}组 第${props.match.roundNumber || 1}轮`
  }

  const roundLabels: Record<string, string> = {
    // 第二阶段
    'CHALLENGER_POSITIONING': '挑战者定位赛',
    'CHALLENGER_PROMOTION': '晋级赛',
    // 第三阶段
    'PREP_WINNERS': '胜者组对决',
    'PREP_LOSERS': '败者组对决',
    'PREP_LOSERS_FINAL': '败者组决赛',
    // 第四阶段
    'FINALS_R1': '终极赛首轮',
    'FINALS_R2': '终极赛次轮',
    'THIRD_PLACE': '季军赛',
    'GRAND_FINAL': '总决赛'
  }

  return roundLabels[props.match.matchType || ''] || '淘汰赛'
}

/**
 * 格式化日期
 */
const formatDate = (date: Date | string): string => {
  const d = typeof date === 'string' ? new Date(date) : date
  return d.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

/**
 * 处理模拟比赛
 */
const handleSimulate = () => {
  emit('simulate', props.match)
}

/**
 * 处理查看详情
 */
const handleViewDetail = () => {
  emit('view-detail', props.match)
}
</script>

<style scoped>
.super-match-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #6366f1;
  border-radius: 6px;
  padding: 16px;
}

.super-match-card.is-completed {
  border-left-color: #22c55e;
  background: #f8fafc;
}

.match-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  line-height: 1.6;
}

.badge-primary {
  background: #eef2ff;
  color: #6366f1;
}

.badge-success {
  background: #f0fdf4;
  color: #22c55e;
}

.badge-warning {
  background: #fffbeb;
  color: #d97706;
}

.badge-danger {
  background: #fef2f2;
  color: #ef4444;
}

.badge-info {
  background: #f1f5f9;
  color: #64748b;
}

.badge-default {
  background: #f8fafc;
  color: #94a3b8;
}

.teams-container {
  margin: 12px 0;
}

.team-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-radius: 4px;
  margin: 4px 0;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
}

.team-row.winner {
  background: #f0fdf4;
  border-color: #22c55e;
  font-weight: bold;
}

.team-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.team-name {
  font-size: 14px;
  font-weight: 500;
  color: #0f172a;
}

.region-tag {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 500;
  background: #f1f5f9;
  color: #64748b;
}

.region-lpl { background: #fef2f2; color: #ef4444; }
.region-lck { background: #eef2ff; color: #6366f1; }
.region-lec { background: #f0fdf4; color: #22c55e; }
.region-lcs { background: #fffbeb; color: #d97706; }

.team-row.winner .team-name {
  color: #16a34a;
}

.team-score {
  font-size: 18px;
  font-weight: bold;
  color: #0f172a;
  min-width: 28px;
  text-align: center;
}

.team-row.winner .team-score {
  color: #16a34a;
}

.vs-divider {
  text-align: center;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
  margin: 2px 0;
}

.match-actions {
  display: flex;
  justify-content: center;
  margin-top: 12px;
}

.simulate-btn {
  padding: 6px 16px;
  background: #6366f1;
  color: #ffffff;
  border: 1px solid #6366f1;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.simulate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.detail-btn {
  padding: 4px 12px;
  background: transparent;
  color: #64748b;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.completed-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.match-footer {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #e2e8f0;
  text-align: center;
}

.completed-time {
  font-size: 12px;
  color: #94a3b8;
}

/* 状态样式 */
.status-scheduled {
  border-color: #e2e8f0;
  border-left-color: #6366f1;
}

.status-in_progress {
  border-color: #d97706;
  border-left-color: #d97706;
  background: #fffbeb;
}

.status-completed {
  border-color: #e2e8f0;
  border-left-color: #22c55e;
}
</style>
