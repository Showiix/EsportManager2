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

interface MatchBase {
  id?: string | number
  teamAId?: string | number | null
  teamBId?: string | number | null
  scoreA?: number
  scoreB?: number
  winnerId?: string | number | null
  status?: string
  bestOf?: number
  matchType?: string
  bracketType?: string
  competitionId?: string | number
}

interface Props {
  match?: MatchBase
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
  (e: 'simulate', match: MatchBase): void
  (e: 'view-detail', match: MatchBase): void
}>()

const getTeamAId = (match?: MatchBase): string | null => {
  return match?.teamAId?.toString() || null
}

const getTeamBId = (match?: MatchBase): string | null => {
  return match?.teamBId?.toString() || null
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

<style scoped>
.msi-match-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
  min-width: 150px;
  transition: all 0.15s;
}

.msi-match-card:hover {
  border-color: #cbd5e1;
}

.msi-match-card.is-completed {
  border-left-color: #6366f1;
}

.msi-match-card.is-highlight {
  border-left-width: 4px;
}

.msi-match-card.is-final {
  border-left-width: 4px;
}

/* Theme colors - border-left */
.msi-match-card.theme-emerald { border-left-color: #10b981; }
.msi-match-card.theme-blue { border-left-color: #3b82f6; }
.msi-match-card.theme-green { border-left-color: #22c55e; }
.msi-match-card.theme-amber { border-left-color: #f59e0b; }
.msi-match-card.theme-orange { border-left-color: #f97316; }
.msi-match-card.theme-red { border-left-color: #ef4444; }

.match-label-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 600;
  background: #f8fafc;
  color: #64748b;
}

/* Theme label colors */
.msi-match-card.theme-emerald .match-label-bar { background: #f0fdf4; color: #059669; }
.msi-match-card.theme-blue .match-label-bar { background: #eff6ff; color: #1e40af; }
.msi-match-card.theme-green .match-label-bar { background: #f0fdf4; color: #166534; }
.msi-match-card.theme-amber .match-label-bar { background: #fef3c7; color: #92400e; }
.msi-match-card.theme-orange .match-label-bar { background: #fff7ed; color: #9a3412; }
.msi-match-card.theme-red .match-label-bar { background: #fef2f2; color: #991b1b; }
.msi-match-card.theme-red.is-final .match-label-bar { background: #ef4444; color: #ffffff; }

.label-text {
  letter-spacing: 0.5px;
}

.teams-section {
  padding: 6px 8px;
}

.team-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 5px 8px;
  border-radius: 4px;
  margin: 2px 0;
  transition: all 0.15s;
}

.team-row.winner {
  background: #f0fdf4;
  font-weight: 700;
}

.team-info {
  flex: 1;
}

.team-name {
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.team-score {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  min-width: 18px;
  text-align: center;
}

.vs-divider {
  text-align: center;
  padding: 1px 0;
}

.vs-text {
  font-size: 9px;
  font-weight: 600;
  color: #cbd5e1;
}

.action-section {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
  padding: 6px 8px;
  border-top: 1px solid #f1f5f9;
}
</style>
