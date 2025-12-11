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
      <el-tag size="small" :type="getMatchTypeBadgeType()">
        {{ getMatchTypeLabel() }}
      </el-tag>
      <el-tag v-if="match.bestOf" size="small" type="info">
        BO{{ match.bestOf }}
      </el-tag>
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
      <el-button
        v-if="canSimulate"
        type="primary"
        size="small"
        :loading="simulating"
        @click="handleSimulate"
      >
        模拟比赛
      </el-button>
      <template v-else-if="match.status === 'completed'">
        <el-button
          type="info"
          size="small"
          plain
          @click="handleViewDetail"
        >
          查看详情
        </el-button>
      </template>
      <el-tag v-else type="info" size="small">
        待确定对阵
      </el-tag>
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
 * 获取比赛类型标签
 */
const getMatchTypeBadgeType = (): string => {
  if (props.match.stage === 'group') return 'primary'
  if (props.match.matchType === 'grand_final') return 'danger'
  if (props.match.matchType === 'third_place') return 'warning'
  return 'success'
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
  emit('view-detail', props.match.id)
}
</script>

<style scoped>
.clauch-match-card {
  background: white;
  border: 2px solid #d1d5db;
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.clauch-match-card:hover {
  border-color: #409eff;
  box-shadow: 0 4px 16px 0 rgba(64, 158, 255, 0.25);
  transform: translateY(-2px);
}

.clauch-match-card.is-completed {
  background: #f5f7fa;
}

.match-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.teams-container {
  margin: 12px 0;
}

.team-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-radius: 6px;
  margin: 6px 0;
  transition: all 0.2s;
  background: #f5f7fa;
  border: 1px solid #dcdfe6;
}

.team-row:hover {
  background: #e4e7ed;
  border-color: #b3b8c0;
}

.team-row.winner {
  background: linear-gradient(to right, #ecf5ff, #e1f3d8);
  border: 2px solid #67c23a;
  font-weight: bold;
}

.team-row.winner:hover {
  background: linear-gradient(to right, #d9ecff, #c0e6b7);
  border-color: #67c23a;
}

.team-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.team-name {
  font-size: 15px;
  font-weight: 500;
  color: #303133;
}

.team-row.winner .team-name {
  color: #67c23a;
}

.team-score {
  font-size: 20px;
  font-weight: bold;
  color: #606266;
  min-width: 30px;
  text-align: center;
}

.team-row.winner .team-score {
  color: #67c23a;
}

.vs-divider {
  text-align: center;
  color: #909399;
  font-size: 12px;
  font-weight: bold;
  margin: 4px 0;
}

.match-actions {
  display: flex;
  justify-content: center;
  margin-top: 12px;
}

.match-footer {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #e4e7ed;
  text-align: center;
}

.completed-time {
  font-size: 12px;
  color: #909399;
}

/* 状态样式 */
.status-scheduled {
  border-color: #e4e7ed;
}

.status-in_progress {
  border-color: #e6a23c;
  background: #fdf6ec;
}

.status-completed {
  border-color: #67c23a;
}
</style>
