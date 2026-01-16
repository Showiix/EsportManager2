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
          <el-tag v-if="match.teamARegion" size="small" :type="getRegionTagType(match.teamARegion)" class="region-tag">
            {{ match.teamARegion }}
          </el-tag>
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
          <el-tag v-if="match.teamBRegion" size="small" :type="getRegionTagType(match.teamBRegion)" class="region-tag">
            {{ match.teamBRegion }}
          </el-tag>
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
      <div v-else-if="match.status === 'completed'" class="completed-actions">
        <el-tag type="success" size="small">已完成</el-tag>
        <el-button type="info" size="small" text @click="handleViewDetail">
          查看详情
        </el-button>
      </div>
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
  if (props.match.matchType === 'CHALLENGER_POSITIONING') return ''
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
  background: white;
  border: 2px solid #d1d5db;
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.super-match-card:hover {
  border-color: #8b5cf6;
  box-shadow: 0 4px 16px 0 rgba(139, 92, 246, 0.25);
  transform: translateY(-2px);
}

.super-match-card.is-completed {
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
  background: linear-gradient(to right, #f3e8ff, #ddd6fe);
  border: 2px solid #8b5cf6;
  font-weight: bold;
}

.team-row.winner:hover {
  background: linear-gradient(to right, #ede9fe, #c4b5fd);
  border-color: #7c3aed;
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

.region-tag {
  font-size: 10px;
}

.team-row.winner .team-name {
  color: #7c3aed;
}

.team-score {
  font-size: 20px;
  font-weight: bold;
  color: #606266;
  min-width: 30px;
  text-align: center;
}

.team-row.winner .team-score {
  color: #7c3aed;
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

.completed-actions {
  display: flex;
  align-items: center;
  gap: 8px;
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
  border-color: #8b5cf6;
}
</style>
