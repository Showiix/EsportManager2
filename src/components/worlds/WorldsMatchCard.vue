<template>
  <div
    class="worlds-match-card"
    :class="[
      `status-${match.status}`,
      { 'is-completed': match.status === 'completed', 'is-final': isFinal, 'is-third-place': isThirdPlace }
    ]"
  >
    <!-- 比赛信息头部 -->
    <div class="match-header">
      <span class="status-badge" :class="getMatchTypeBadgeClass()">
        {{ getMatchTypeLabel() }}
      </span>
      <span v-if="match.bestOf" class="status-badge badge-info">
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
        @click="handleSimulate"
      >
        模拟比赛
      </button>
      <div v-else-if="match.status === 'completed'" class="completed-actions">
        <span class="status-badge badge-success">已完成</span>
        <button class="detail-btn" @click="handleViewDetail">
          查看详情
        </button>
      </div>
      <span v-else class="status-badge badge-info">
        待确定对阵
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { WorldsKnockoutMatch } from '@/types/index'

interface Props {
  match: WorldsKnockoutMatch
  isFinal?: boolean
  isThirdPlace?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isFinal: false,
  isThirdPlace: false
})

const emit = defineEmits<{
  (e: 'simulate', match: WorldsKnockoutMatch): void
  (e: 'view-detail', match: WorldsKnockoutMatch): void
}>()

/**
 * 是否可以模拟
 */
const canSimulate = computed(() => {
  return (
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
 * 获取比赛类型标签样式类
 */
const getMatchTypeBadgeClass = (): string => {
  if (props.isFinal) return 'badge-danger'
  if (props.isThirdPlace) return 'badge-warning'
  if (props.match.round === 'QUARTER_FINAL') return 'badge-primary'
  if (props.match.round === 'SEMI_FINAL') return 'badge-success'
  return 'badge-info'
}

/**
 * 获取比赛类型标签文字
 */
const getMatchTypeLabel = (): string => {
  const roundLabels: Record<string, string> = {
    'QUARTER_FINAL': '八强赛',
    'SEMI_FINAL': '半决赛',
    'THIRD_PLACE': '季军赛',
    'FINAL': '总决赛'
  }
  return roundLabels[props.match.round || ''] || '淘汰赛'
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
.worlds-match-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #6366f1;
  border-radius: 8px;
  padding: 12px;
  min-width: 180px;
}

.worlds-match-card.is-completed {
  border-left-color: #22c55e;
}

.worlds-match-card.is-final {
  border-left-color: #6366f1;
  background: #f8fafc;
}

.worlds-match-card.is-third-place {
  border-left-color: #94a3b8;
  background: #f8fafc;
}

.match-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}

.status-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
  line-height: 1.4;
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
  background: #fefce8;
  color: #ca8a04;
}

.badge-danger {
  background: #fef2f2;
  color: #ef4444;
}

.badge-info {
  background: #f1f5f9;
  color: #64748b;
}

.teams-container {
  margin: 8px 0;
}

.team-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  border-radius: 4px;
  margin: 4px 0;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
}

.team-row.winner {
  background: #f0fdf4;
  border-color: #22c55e;
}

.team-row.winner .team-name {
  color: #16a34a;
  font-weight: 700;
}

.team-row.winner .team-score {
  color: #16a34a;
}

.team-info {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}

.team-name {
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.team-score {
  font-size: 16px;
  font-weight: bold;
  color: #64748b;
  min-width: 24px;
  text-align: center;
}

.vs-divider {
  text-align: center;
  color: #94a3b8;
  font-size: 10px;
  font-weight: bold;
  margin: 2px 0;
}

.match-actions {
  display: flex;
  justify-content: center;
  margin-top: 8px;
}

.completed-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.simulate-btn {
  padding: 4px 14px;
  font-size: 12px;
  font-weight: 600;
  color: #ffffff;
  background: #6366f1;
  border: 1px solid #6366f1;
  border-radius: 6px;
  cursor: pointer;
}

.detail-btn {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  color: #64748b;
  background: transparent;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  cursor: pointer;
}

/* 状态样式 */
.status-scheduled {
  border-color: #e2e8f0;
}

.status-in_progress {
  border-color: #ca8a04;
  background: #fefce8;
}

.status-completed {
  border-left-color: #22c55e;
}
</style>
