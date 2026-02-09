<template>
  <div class="super-group-standing">
    <div class="group-header">
      <span class="group-title">{{ groupName }}组</span>
      <span class="status-badge" :class="'status-' + getGroupStatusType()">
        {{ getGroupStatusText() }}
      </span>
    </div>

    <!-- 小组积分榜 -->
    <div class="standings-table">
      <el-table
        :data="sortedStandings"
        stripe
        border
        style="width: 100%"
        :row-class-name="getRowClassName"
      >
        <el-table-column
          prop="position"
          label="排名"
          width="70"
          align="center"
        >
          <template #default="{ row }">
            <span v-if="row.position === 1" class="rank-qualified">{{ row.position }}</span>
            <span v-else>{{ row.position }}</span>
          </template>
        </el-table-column>

        <el-table-column
          prop="teamName"
          label="战队"
          min-width="150"
        >
          <template #default="{ row }">
            <div class="team-cell">
              <span class="team-name" :class="{ qualified: row.qualified }">{{ row.teamName }}</span>
              <span v-if="row.qualified" class="qualified-mark">&#10003;</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column
          prop="regionName"
          label="赛区"
          width="80"
          align="center"
        >
          <template #default="{ row }">
            <span class="region-tag" :class="'region-' + (row.regionName || '').toLowerCase()">{{ row.regionName }}</span>
          </template>
        </el-table-column>

        <el-table-column
          prop="matchesPlayed"
          label="场次"
          width="70"
          align="center"
        />

        <el-table-column
          prop="wins"
          label="胜"
          width="60"
          align="center"
        >
          <template #default="{ row }">
            <span class="win-count">{{ row.wins }}</span>
          </template>
        </el-table-column>

        <el-table-column
          prop="losses"
          label="负"
          width="60"
          align="center"
        >
          <template #default="{ row }">
            <span class="loss-count">{{ row.losses }}</span>
          </template>
        </el-table-column>

        <el-table-column
          prop="points"
          label="积分"
          width="80"
          align="center"
        >
          <template #default="{ row }">
            <span class="points-value">{{ row.points }}</span>
          </template>
        </el-table-column>

        <el-table-column
          label="小局"
          width="100"
          align="center"
        >
          <template #default="{ row }">
            {{ row.roundsWon }}-{{ row.roundsLost }}
          </template>
        </el-table-column>

        <el-table-column
          prop="roundDifferential"
          label="净胜局"
          width="90"
          align="center"
        >
          <template #default="{ row }">
            <span
              :class="{
                'positive-diff': row.roundDifferential > 0,
                'negative-diff': row.roundDifferential < 0
              }"
            >
              {{ row.roundDifferential > 0 ? '+' : '' }}{{ row.roundDifferential }}
            </span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 积分规则说明 -->
      <div class="scoring-rules">
        <span>积分规则: 2:0胜=3分 | 2:1胜=2分 | 1:2负=1分 | 0:2负=0分 | 小组第1名晋级第二阶段</span>
      </div>
    </div>

    <!-- 赛程折叠区 -->
    <div class="matches-toggle" @click="showMatches = !showMatches">
      <span>{{ showMatches ? '收起赛程' : '展开赛程' }}</span>
      <span class="toggle-arrow" :class="{ expanded: showMatches }">&#9662;</span>
    </div>

    <!-- 小组赛程 -->
    <div v-show="showMatches" class="group-matches">
      <!-- 按轮次分组显示 -->
      <div
        v-for="round in groupedMatches"
        :key="round.roundNumber"
        class="round-section"
      >
        <div class="round-header">
          <h5>第{{ round.roundNumber }}轮</h5>
          <span class="round-badge" :class="'round-' + getRoundProgressType(round)">
            {{ getRoundProgressText(round) }}
          </span>
        </div>

        <div class="matches-grid">
          <SuperMatchCard
            v-for="match in round.matches"
            :key="match.id"
            :match="match"
            :simulating="simulatingMatchId === match.id"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewMatch"
          />
        </div>
      </div>
    </div>

    <!-- 批量操作 -->
    <div v-show="showMatches" class="group-actions">
      <button
        v-if="hasUncompletedMatches"
        class="batch-btn"
        :disabled="batchSimulating"
        @click="handleBatchSimulate"
      >
        {{ batchSimulating ? '模拟中...' : '一键模拟全部比赛' }}
      </button>
      <span v-else class="all-done-label">小组赛已全部完成</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { SuperMatch, SuperGroup, SuperGroupStanding } from '@/types/super'
import SuperMatchCard from './SuperMatchCard.vue'

interface Props {
  group: SuperGroup
  simulatingMatchId?: string | null
  batchSimulating?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  simulatingMatchId: null,
  batchSimulating: false
})

const showMatches = ref(false)

const emit = defineEmits<{
  (e: 'simulate-match', match: SuperMatch): void
  (e: 'view-match', match: SuperMatch): void
  (e: 'batch-simulate-group'): void
}>()

/**
 * 小组名称
 */
const groupName = computed(() => props.group.groupName)

/**
 * 排序后的积分榜
 */
const sortedStandings = computed(() => {
  return [...props.group.standings].sort((a, b) => {
    // 先按积分排序
    if (b.points !== a.points) return b.points - a.points
    // 积分相同按净胜局排序
    if (b.roundDifferential !== a.roundDifferential) {
      return b.roundDifferential - a.roundDifferential
    }
    // 净胜局相同按胜场排序
    return b.wins - a.wins
  })
})

/**
 * 按轮次分组的比赛（过滤掉 undefined/null）
 */
const groupedMatches = computed(() => {
  const rounds: { [key: number]: SuperMatch[] } = {}

  // 先过滤掉 undefined/null 的比赛
  const validMatches = (props.group.matches || []).filter(m => m != null)

  validMatches.forEach(match => {
    const roundNumber = match.roundNumber || 1
    if (!rounds[roundNumber]) {
      rounds[roundNumber] = []
    }
    rounds[roundNumber].push(match)
  })

  return Object.entries(rounds)
    .map(([roundNumber, matches]) => ({
      roundNumber: parseInt(roundNumber),
      matches
    }))
    .sort((a, b) => a.roundNumber - b.roundNumber)
})

/**
 * 是否有未完成的比赛
 */
const hasUncompletedMatches = computed(() => {
  return props.group.matches.some(m => m.status !== 'completed')
})

/**
 * 获取小组状态类型
 */
const getGroupStatusType = () => {
  const allCompleted = props.group.matches.every(m => m.status === 'completed')
  if (allCompleted) return 'success'

  const anyInProgress = props.group.matches.some(m => m.status === 'in_progress')
  if (anyInProgress) return 'warning'

  return 'info'
}

/**
 * 获取小组状态文本
 */
const getGroupStatusText = () => {
  const completed = props.group.matches.filter(m => m.status === 'completed').length
  const total = props.group.matches.length

  if (completed === total) return '已完成'
  return `${completed}/${total}`
}

/**
 * 获取轮次进度类型
 */
const getRoundProgressType = (round: { matches: SuperMatch[] }) => {
  const allCompleted = round.matches.every(m => m.status === 'completed')
  if (allCompleted) return 'success'

  const anyCompleted = round.matches.some(m => m.status === 'completed')
  if (anyCompleted) return 'warning'

  return 'info'
}

/**
 * 获取轮次进度文本
 */
const getRoundProgressText = (round: { matches: SuperMatch[] }) => {
  const completed = round.matches.filter(m => m.status === 'completed').length
  const total = round.matches.length

  if (completed === total) return '已完成'
  return `${completed}/${total}`
}

/**
 * 获取表格行类名
 */
const getRowClassName = ({ row }: { row: SuperGroupStanding }) => {
  if (row.position === 1) return 'qualified-row'
  return ''
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
 * 处理模拟比赛
 */
const handleSimulateMatch = (match: SuperMatch) => {
  emit('simulate-match', match)
}

/**
 * 处理查看比赛详情
 */
const handleViewMatch = (match: SuperMatch) => {
  emit('view-match', match)
}

/**
 * 处理批量模拟
 */
const handleBatchSimulate = () => {
  emit('batch-simulate-group')
}
</script>

<style scoped>
.super-group-standing {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 20px;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e2e8f0;
}

.group-title {
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.status-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
}

.status-info {
  background: #f1f5f9;
  color: #64748b;
}

.status-success {
  background: #f0fdf4;
  color: #22c55e;
}

.status-warning {
  background: #fffbeb;
  color: #d97706;
}

.standings-table {
  margin-bottom: 16px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.team-name {
  font-weight: 500;
  color: #0f172a;
}

.team-name.qualified {
  color: #22c55e;
  font-weight: 600;
}

.qualified-mark {
  color: #22c55e;
  font-size: 14px;
  font-weight: bold;
}

.rank-qualified {
  color: #22c55e;
  font-weight: 600;
}

.region-tag {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 500;
  background: #f1f5f9;
  color: #64748b;
}

.region-lpl { background: #fef2f2; color: #ef4444; }
.region-lck { background: #eef2ff; color: #6366f1; }
.region-lec { background: #f0fdf4; color: #22c55e; }
.region-lcs { background: #fffbeb; color: #d97706; }

.win-count {
  color: #22c55e;
  font-weight: 500;
}

.loss-count {
  color: #ef4444;
  font-weight: 500;
}

.points-value {
  font-size: 15px;
  font-weight: bold;
  color: #6366f1;
}

.positive-diff {
  color: #22c55e;
  font-weight: 500;
}

.negative-diff {
  color: #ef4444;
  font-weight: 500;
}

.scoring-rules {
  margin-top: 10px;
  padding: 10px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  text-align: center;
  font-size: 12px;
  color: #64748b;
}

:deep(.qualified-row) {
  background-color: #f0fdf4 !important;
}

/* 赛程折叠区 */
.matches-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px;
  margin: 16px 0 0 0;
  cursor: pointer;
  color: #64748b;
  font-size: 13px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  background: #f8fafc;
}

.toggle-arrow {
  display: inline-block;
  transition: transform 0.2s;
  font-size: 12px;
}

.toggle-arrow.expanded {
  transform: rotate(180deg);
}

.group-matches {
  margin-top: 16px;
}

.round-section {
  margin-bottom: 20px;
}

.round-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.round-header h5 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.round-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 500;
}

.round-info {
  background: #f1f5f9;
  color: #64748b;
}

.round-success {
  background: #f0fdf4;
  color: #22c55e;
}

.round-warning {
  background: #fffbeb;
  color: #d97706;
}

.matches-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.group-actions {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: center;
}

.batch-btn {
  padding: 8px 20px;
  background: #6366f1;
  color: #ffffff;
  border: 1px solid #6366f1;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.batch-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.all-done-label {
  display: inline-block;
  padding: 6px 16px;
  background: #f0fdf4;
  color: #22c55e;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
}
</style>
