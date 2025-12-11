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
 * 获取比赛类型标签
 */
const getMatchTypeBadgeType = (): string => {
  if (props.isFinal) return 'danger'
  if (props.isThirdPlace) return 'warning'
  if (props.match.round === 'QUARTER_FINAL') return 'primary'
  if (props.match.round === 'SEMI_FINAL') return 'success'
  return 'info'
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

<style scoped lang="scss">
.worlds-match-card {
  background: white;
  border: 2px solid #d1d5db;
  border-radius: 8px;
  padding: 12px;
  transition: all 0.3s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  min-width: 180px;

  &:hover {
    border-color: #409eff;
    box-shadow: 0 4px 12px rgba(64, 158, 255, 0.2);
    transform: translateY(-2px);
  }

  &.is-completed {
    background: #f5f7fa;
  }

  &.is-final {
    border-color: #f59e0b;
    background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);

    &:hover {
      border-color: #d97706;
      box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
    }
  }

  &.is-third-place {
    border-color: #d97706;
    background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
  }

  .match-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
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
    transition: all 0.2s;
    background: #f5f7fa;
    border: 1px solid #e5e7eb;

    &:hover {
      background: #e4e7ed;
    }

    &.winner {
      background: linear-gradient(to right, #ecf5ff, #e1f3d8);
      border: 2px solid #67c23a;

      .team-name {
        color: #67c23a;
        font-weight: 700;
      }

      .team-score {
        color: #67c23a;
      }
    }
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
    color: #303133;
  }

  .team-score {
    font-size: 16px;
    font-weight: bold;
    color: #606266;
    min-width: 24px;
    text-align: center;
  }

  .vs-divider {
    text-align: center;
    color: #909399;
    font-size: 10px;
    font-weight: bold;
    margin: 2px 0;
  }

  .match-actions {
    display: flex;
    justify-content: center;
    margin-top: 8px;

    .completed-actions {
      display: flex;
      align-items: center;
      gap: 8px;
    }
  }
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
