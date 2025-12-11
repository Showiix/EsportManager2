<template>
  <div
    class="msi-match-card"
    :class="[
      `theme-${colorTheme}`,
      {
        'is-completed': match?.status === 'completed',
        'is-highlight': highlight,
        'is-final': isFinal
      }
    ]"
  >
    <!-- 比赛标签 -->
    <div class="match-label-bar">
      <span class="label-text">{{ matchLabel }}</span>
      <el-tag v-if="match?.bestOf" size="small" type="info">BO{{ match.bestOf }}</el-tag>
    </div>

    <!-- 队伍对阵 -->
    <div class="teams-section">
      <div
        class="team-row"
        :class="{ winner: isWinner(getTeamAId(match)) }"
      >
        <div class="team-info">
          <span class="team-name">{{ getTeamName(getTeamAId(match)) }}</span>
        </div>
        <div v-if="match?.status === 'completed'" class="team-score">
          {{ match.scoreA ?? 0 }}
        </div>
      </div>

      <div class="vs-divider">
        <span class="vs-text">VS</span>
      </div>

      <div
        class="team-row"
        :class="{ winner: isWinner(getTeamBId(match)) }"
      >
        <div class="team-info">
          <span class="team-name">{{ getTeamName(getTeamBId(match)) }}</span>
        </div>
        <div v-if="match?.status === 'completed'" class="team-score">
          {{ match.scoreB ?? 0 }}
        </div>
      </div>
    </div>

    <!-- 操作区域 -->
    <div class="action-section">
      <el-button
        v-if="canSimulate"
        :type="buttonType"
        size="small"
        @click="handleSimulate"
      >
        模拟比赛
      </el-button>
      <template v-else-if="match?.status === 'completed'">
        <el-tag type="success" size="small">已完成</el-tag>
        <el-button type="info" size="small" text @click="handleViewDetail">
          详情
        </el-button>
      </template>
      <el-tag v-else type="info" size="small">
        待确定对阵
      </el-tag>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Team {
  teamId: string
  teamName: string
  regionName?: string
}

interface Match {
  id?: string
  teamAId?: string | null
  teamBId?: string | null
  scoreA?: number
  scoreB?: number
  winnerId?: string | null
  status?: string
  bestOf?: number
  matchType?: string
}

interface Props {
  match?: Match
  teams?: Team[]
  matchLabel?: string
  colorTheme?: 'emerald' | 'blue' | 'green' | 'amber' | 'orange' | 'red'
  highlight?: boolean
  isFinal?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  matchLabel: '比赛',
  colorTheme: 'blue',
  highlight: false,
  isFinal: false
})

const emit = defineEmits<{
  (e: 'simulate', match: Match): void
  (e: 'view-detail', match: Match): void
}>()

const getTeamAId = (match?: Match): string | null => {
  return match?.teamAId || null
}

const getTeamBId = (match?: Match): string | null => {
  return match?.teamBId || null
}

const getTeamName = (teamId: string | null): string => {
  if (!teamId) return '待定'
  const team = props.teams?.find(t => t.teamId?.toString() === teamId.toString())
  return team?.teamName || '待定'
}

const isWinner = (teamId: string | null): boolean => {
  if (!teamId || !props.match?.winnerId) return false
  return props.match.winnerId.toString() === teamId.toString()
}

const canSimulate = computed(() => {
  if (!props.match) return false
  return (
    props.match.status !== 'completed' &&
    !!getTeamAId(props.match) &&
    !!getTeamBId(props.match)
  )
})

const buttonType = computed(() => {
  const typeMap: Record<string, any> = {
    'emerald': 'success',
    'blue': 'primary',
    'green': 'success',
    'amber': 'warning',
    'orange': 'warning',
    'red': 'danger'
  }
  return typeMap[props.colorTheme] || 'primary'
})

const handleSimulate = () => {
  if (props.match) {
    emit('simulate', props.match)
  }
}

const handleViewDetail = () => {
  if (props.match) {
    emit('view-detail', props.match)
  }
}
</script>

<style scoped lang="scss">
.msi-match-card {
  background: white;
  border: 2px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  min-width: 140px;
  transition: all 0.2s ease;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  &.is-completed {
    background: #f8fafc;
  }

  &.is-highlight {
    border-width: 3px;
  }

  &.is-final {
    border-width: 3px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  }

  // 颜色主题
  &.theme-emerald {
    border-color: #10b981;

    .match-label-bar {
      background: linear-gradient(135deg, #d1fae5 0%, #a7f3d0 100%);
      color: #065f46;
    }

    .team-row.winner {
      background: #d1fae5;
      .team-name, .team-score { color: #059669; }
    }
  }

  &.theme-blue {
    border-color: #3b82f6;

    .match-label-bar {
      background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
      color: #1e40af;
    }

    .team-row.winner {
      background: #dbeafe;
      .team-name, .team-score { color: #2563eb; }
    }
  }

  &.theme-green {
    border-color: #22c55e;

    .match-label-bar {
      background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
      color: #166534;
    }

    .team-row.winner {
      background: #dcfce7;
      .team-name, .team-score { color: #16a34a; }
    }
  }

  &.theme-amber {
    border-color: #f59e0b;

    .match-label-bar {
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      color: #92400e;
    }

    .team-row.winner {
      background: #fef3c7;
      .team-name, .team-score { color: #d97706; }
    }
  }

  &.theme-orange {
    border-color: #f97316;

    .match-label-bar {
      background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
      color: #9a3412;
    }

    .team-row.winner {
      background: #fed7aa;
      .team-name, .team-score { color: #ea580c; }
    }
  }

  &.theme-red {
    border-color: #ef4444;

    .match-label-bar {
      background: linear-gradient(135deg, #fecaca 0%, #fca5a5 100%);
      color: #991b1b;
    }

    .team-row.winner {
      background: #fee2e2;
      .team-name, .team-score { color: #dc2626; }
    }

    &.is-final {
      .match-label-bar {
        background: linear-gradient(135deg, #f87171 0%, #ef4444 100%);
        color: white;
      }
    }
  }

  .match-label-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 600;

    .label-text {
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
  }

  .teams-section {
    padding: 8px;
  }

  .team-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px;
    border-radius: 6px;
    margin: 4px 0;
    background: #f8fafc;
    transition: all 0.2s;

    &.winner {
      font-weight: 700;
    }

    .team-info {
      flex: 1;
    }

    .team-name {
      font-size: 13px;
      font-weight: 500;
      color: #334155;
    }

    .team-score {
      font-size: 16px;
      font-weight: 700;
      color: #475569;
      min-width: 20px;
      text-align: center;
    }
  }

  .vs-divider {
    text-align: center;
    padding: 2px 0;

    .vs-text {
      font-size: 10px;
      font-weight: 700;
      color: #94a3b8;
    }
  }

  .action-section {
    display: flex;
    justify-content: center;
    padding: 8px;
    background: #f8fafc;
    border-top: 1px solid #e2e8f0;
  }
}
</style>
