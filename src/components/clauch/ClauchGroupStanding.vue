<template>
  <div class="clauch-group-standing">
    <div class="group-header">
      <span class="group-title">{{ groupName }}组</span>
      <span class="status-badge" :class="getGroupStatusType()">
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
        <el-table-column prop="position" label="#" width="45" align="center">
          <template #default="{ row }">
            <span :class="{ 'rank-qualified': row.position <= 2 }">{{ row.position }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="teamName" label="战队" min-width="120">
          <template #default="{ row }">
            <span class="team-name" :class="{ qualified: row.qualified }">{{ row.teamName }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="wins" label="胜" width="45" align="center">
          <template #default="{ row }">
            <span class="win-count">{{ row.wins }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="losses" label="负" width="45" align="center">
          <template #default="{ row }">
            <span class="loss-count">{{ row.losses }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="points" label="分" width="45" align="center">
          <template #default="{ row }">
            <span class="points-value">{{ row.points }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="roundDifferential" label="净局" width="50" align="center">
          <template #default="{ row }">
            <span :class="{ 'positive-diff': row.roundDifferential > 0, 'negative-diff': row.roundDifferential < 0 }">
              {{ row.roundDifferential > 0 ? '+' : '' }}{{ row.roundDifferential }}
            </span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 积分规则说明 -->
      <div class="scoring-rules">
        <span>2:0胜=3分 | 2:1胜=2分 | 1:2负=1分 | 0:2负=0分</span>
      </div>
    </div>

    <!-- 赛程折叠区 -->
    <div class="matches-toggle" @click="showMatches = !showMatches">
      <span>{{ showMatches ? '收起赛程' : '展开赛程' }}</span>
      <span class="toggle-arrow" :class="{ expanded: showMatches }">&#9662;</span>
    </div>

    <div v-show="showMatches" class="group-matches">
      <div class="matches-grid">
        <ClauchMatchCard
          v-for="match in allMatches"
          :key="match.id"
          :match="match"
          :simulating="simulatingMatchId === match.id"
          @simulate="handleSimulateMatch"
          @view-detail="handleViewDetail"
        />
      </div>
    </div>

    <div v-show="showMatches" class="group-actions">
      <button v-if="hasUncompletedMatches" class="batch-btn" :disabled="batchSimulating" @click="handleBatchSimulate">
        {{ batchSimulating ? '模拟中...' : '一键模拟全部' }}
      </button>
      <span v-else class="all-done-label">已全部完成</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ClauchMatch, ClauchGroup, ClauchGroupStanding } from '@/types/clauch'
import ClauchMatchCard from './ClauchMatchCard.vue'

interface Props {
  group: ClauchGroup
  simulatingMatchId?: string | null
  batchSimulating?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  simulatingMatchId: null,
  batchSimulating: false
})

const showMatches = ref(false)

const emit = defineEmits<{
  (e: 'simulate-match', match: ClauchMatch): void
  (e: 'batch-simulate-group'): void
  (e: 'view-detail', matchId: string | number): void
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
 * 所有比赛（平铺，过滤掉 undefined/null）
 */
const allMatches = computed(() => {
  return (props.group.matches || []).filter((m): m is ClauchMatch => m != null)
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
 * 获取表格行类名
 */
const getRowClassName = ({ row }: { row: ClauchGroupStanding }) => {
  if (row.position <= 2) return 'qualified-row'
  return ''
}

/**
 * 处理模拟比赛
 */
const handleSimulateMatch = (match: ClauchMatch) => {
  emit('simulate-match', match)
}

/**
 * 处理查看详情
 */
const handleViewDetail = (matchId: string | number) => {
  emit('view-detail', matchId)
}

/**
 * 处理批量模拟
 */
const handleBatchSimulate = () => {
  emit('batch-simulate-group')
}
</script>

<style scoped>
.clauch-group-standing {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 12px;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.group-title {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.status-badge {
  display: inline-block;
  padding: 1px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 8px;
  background: #f1f5f9;
  color: #64748b;
}

.status-badge.success {
  background: #f0fdf4;
  color: #16a34a;
}

.status-badge.warning {
  background: #fffbeb;
  color: #d97706;
}

.status-badge.info {
  background: #f1f5f9;
  color: #64748b;
}

.standings-table {
  margin-bottom: 8px;
}

.team-name {
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.team-name.qualified {
  color: #16a34a;
}

.rank-qualified {
  color: #16a34a;
  font-weight: 600;
}

.win-count {
  color: #16a34a;
  font-weight: 500;
}

.loss-count {
  color: #ef4444;
  font-weight: 500;
}

.points-value {
  font-weight: 600;
  color: #6366f1;
}

.positive-diff {
  color: #16a34a;
  font-weight: 500;
}

.negative-diff {
  color: #ef4444;
  font-weight: 500;
}

.scoring-rules {
  padding: 6px 10px;
  background: #f8fafc;
  border-radius: 4px;
  text-align: center;
  font-size: 11px;
  color: #94a3b8;
}

:deep(.qualified-row) {
  background-color: #f0fdf4 !important;
}

.matches-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 8px;
  margin-top: 8px;
  cursor: pointer;
  font-size: 12px;
  color: #6366f1;
  border-top: 1px solid #f1f5f9;
  user-select: none;
}

.matches-toggle:hover {
  color: #4f46e5;
}

.toggle-arrow {
  font-size: 10px;
  transition: transform 0.2s;
}

.toggle-arrow.expanded {
  transform: rotate(180deg);
}

.group-matches {
  margin-top: 8px;
}

.round-section {
  margin-bottom: 12px;
}

.round-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.round-name {
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
}

.round-status {
  font-size: 10px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: 6px;
  background: #f1f5f9;
  color: #64748b;
}

.round-status.success {
  background: #f0fdf4;
  color: #16a34a;
}

.round-status.warning {
  background: #fffbeb;
  color: #d97706;
}

.round-status.info {
  background: #f1f5f9;
  color: #64748b;
}

.matches-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.group-actions {
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px solid #f1f5f9;
  display: flex;
  justify-content: center;
}

.batch-btn {
  padding: 4px 14px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  border: none;
  background: #6366f1;
  color: #ffffff;
}

.batch-btn:hover {
  background: #4f46e5;
}

.batch-btn:disabled {
  background: #c7d2fe;
  cursor: not-allowed;
}

.all-done-label {
  font-size: 12px;
  color: #16a34a;
  font-weight: 500;
}
</style>
