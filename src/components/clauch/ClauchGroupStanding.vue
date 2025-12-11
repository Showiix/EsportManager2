<template>
  <div class="clauch-group-standing">
    <div class="group-header">
      <h3 class="group-title">{{ groupName }}组</h3>
      <el-tag :type="getGroupStatusType()" size="large">
        {{ getGroupStatusText() }}
      </el-tag>
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
            <el-tag
              v-if="row.position <= 2"
              type="success"
              size="small"
            >
              {{ row.position }}
            </el-tag>
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
              <span class="team-name">{{ row.teamName }}</span>
              <el-icon v-if="row.qualified" class="qualified-icon" color="#67c23a">
                <Select />
              </el-icon>
            </div>
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
        <el-text size="small" type="info">
          积分规则: 2:0胜=3分 | 2:1胜=2分 | 1:2负=1分 | 0:2负=0分 | 小组前2名晋级淘汰赛
        </el-text>
      </div>
    </div>

    <!-- 小组赛程 -->
    <div class="group-matches">
      <h4 class="matches-title">小组赛程</h4>

      <!-- 按轮次分组显示 -->
      <div
        v-for="round in groupedMatches"
        :key="round.roundNumber"
        class="round-section"
      >
        <div class="round-header">
          <h5>第{{ round.roundNumber }}轮</h5>
          <el-tag
            :type="getRoundProgressType(round)"
            size="small"
          >
            {{ getRoundProgressText(round) }}
          </el-tag>
        </div>

        <div class="matches-grid">
          <ClauchMatchCard
            v-for="match in round.matches"
            :key="match.id"
            :match="match"
            :simulating="simulatingMatchId === match.id"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>
    </div>

    <!-- 批量操作 -->
    <div class="group-actions">
      <el-button
        v-if="hasUncompletedMatches"
        type="primary"
        :loading="batchSimulating"
        @click="handleBatchSimulate"
      >
        一键模拟全部比赛
      </el-button>
      <el-tag v-else type="success" size="large">
        小组赛已全部完成
      </el-tag>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Select } from '@element-plus/icons-vue'
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
 * 按轮次分组的比赛（过滤掉 undefined/null）
 */
const groupedMatches = computed(() => {
  const rounds: { [key: number]: ClauchMatch[] } = {}

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
const getRoundProgressType = (round: { matches: ClauchMatch[] }) => {
  const allCompleted = round.matches.every(m => m.status === 'completed')
  if (allCompleted) return 'success'

  const anyCompleted = round.matches.some(m => m.status === 'completed')
  if (anyCompleted) return 'warning'

  return 'info'
}

/**
 * 获取轮次进度文本
 */
const getRoundProgressText = (round: { matches: ClauchMatch[] }) => {
  const completed = round.matches.filter(m => m.status === 'completed').length
  const total = round.matches.length

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
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 2px solid #e4e7ed;
}

.group-title {
  margin: 0;
  font-size: 20px;
  font-weight: bold;
  color: #303133;
}

.standings-table {
  margin-bottom: 24px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-name {
  font-weight: 500;
}

.qualified-icon {
  font-size: 16px;
}

.win-count {
  color: #67c23a;
  font-weight: 500;
}

.loss-count {
  color: #f56c6c;
  font-weight: 500;
}

.points-value {
  font-size: 16px;
  font-weight: bold;
  color: #409eff;
}

.positive-diff {
  color: #67c23a;
  font-weight: 500;
}

.negative-diff {
  color: #f56c6c;
  font-weight: 500;
}

.scoring-rules {
  margin-top: 12px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 4px;
  text-align: center;
}

:deep(.qualified-row) {
  background-color: #f0f9ff !important;
}

.group-matches {
  margin-top: 24px;
}

.matches-title {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: bold;
  color: #303133;
}

.round-section {
  margin-bottom: 24px;
}

.round-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.round-header h5 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #606266;
}

.matches-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}

.group-actions {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
  display: flex;
  justify-content: center;
}
</style>
