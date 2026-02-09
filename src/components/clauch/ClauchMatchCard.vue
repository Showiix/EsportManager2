<template>
  <div
    class="clauch-match-card"
    :class="[
      `status-${match.status}`,
      { 'is-completed': match.status === 'completed' }
    ]"
  >
    <!-- 比赛信息头部 -->
    <div class="match-header">
      <span class="match-type-badge">{{ getMatchTypeLabel() }}</span>
      <span v-if="match.bestOf" class="bo-badge">BO{{ match.bestOf }}</span>
    </div>

    <!-- 队伍对阵 -->
    <div class="teams-container">
      <div
        class="team-row"
        :class="{ winner: isWinner(match.teamAId) }"
      >
        <span class="team-name">{{ match.teamAName || '待定' }}</span>
        <span v-if="match.status === 'completed'" class="team-score">{{ match.scoreA }}</span>
      </div>

      <div class="vs-divider">VS</div>

      <div
        class="team-row"
        :class="{ winner: isWinner(match.teamBId) }"
      >
        <span class="team-name">{{ match.teamBName || '待定' }}</span>
        <span v-if="match.status === 'completed'" class="team-score">{{ match.scoreB }}</span>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="match-actions">
      <button
        v-if="canSimulate"
        class="action-btn simulate-btn"
        :disabled="simulating"
        @click="handleSimulate"
      >
        {{ simulating ? '模拟中...' : '模拟比赛' }}
      </button>
      <button
        v-else-if="match.status === 'completed'"
        class="action-btn detail-btn"
        @click="handleViewDetail"
      >
        查看详情
      </button>
      <span v-else class="pending-label">待确定对阵</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ClauchMatch } from '@/types/clauch'

interface Props {
  match: ClauchMatch
  simulating?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  simulating: false
})

const emit = defineEmits<{
  (e: 'simulate', match: ClauchMatch): void
  (e: 'view-detail', matchId: string | number): void
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
 * 获取比赛类型标签文字
 */
const getMatchTypeLabel = (): string => {
  if (props.match.stage === 'group') {
    return `${props.match.groupName}组 第${props.match.roundNumber || 1}轮`
  }

  const roundLabels: Record<string, string> = {
    'east_quarter': '东半区第一轮',
    'west_quarter': '西半区第一轮',
    'east_semi': '东半区半决赛',
    'west_semi': '西半区半决赛',
    'east_final': '东半区决赛',
    'west_final': '西半区决赛',
    'third_place': '季军赛',
    'grand_final': '总决赛'
  }

  return roundLabels[props.match.matchType || ''] || '淘汰赛'
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
  emit('view-detail', props.match.id)
}
</script>

<style scoped>
.clauch-match-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #6366f1;
  border-radius: 8px;
  padding: 12px 14px;
}

.clauch-match-card.is-completed {
  border-left-color: #22c55e;
  background: #fafffe;
}

.clauch-match-card.status-scheduled {
  border-left-color: #94a3b8;
}

.clauch-match-card.status-in_progress {
  border-left-color: #f59e0b;
}

.match-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}

.match-type-badge {
  font-size: 11px;
  font-weight: 500;
  color: #64748b;
  background: #f1f5f9;
  padding: 1px 8px;
  border-radius: 8px;
}

.bo-badge {
  font-size: 11px;
  font-weight: 500;
  color: #94a3b8;
  background: #f8fafc;
  padding: 1px 6px;
  border-radius: 6px;
}

.teams-container {
  margin: 8px 0;
}

.team-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  border-radius: 4px;
  margin: 4px 0;
  background: #f8fafc;
}

.team-row.winner {
  background: #f0fdf4;
}

.team-name {
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.team-row.winner .team-name {
  color: #16a34a;
  font-weight: 600;
}

.team-score {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  min-width: 20px;
  text-align: center;
}

.team-row.winner .team-score {
  color: #16a34a;
}

.vs-divider {
  text-align: center;
  color: #cbd5e1;
  font-size: 10px;
  font-weight: 600;
  margin: 2px 0;
}

.match-actions {
  display: flex;
  justify-content: center;
  margin-top: 8px;
}

.action-btn {
  padding: 4px 14px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  border: none;
  transition: background 0.15s;
}

.simulate-btn {
  background: #6366f1;
  color: #ffffff;
}

.simulate-btn:hover {
  background: #4f46e5;
}

.simulate-btn:disabled {
  background: #c7d2fe;
  cursor: not-allowed;
}

.detail-btn {
  background: #f1f5f9;
  color: #64748b;
}

.detail-btn:hover {
  background: #e2e8f0;
}

.pending-label {
  font-size: 11px;
  color: #94a3b8;
}
</style>
