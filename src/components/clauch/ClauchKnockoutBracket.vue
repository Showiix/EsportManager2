<template>
  <div class="clauch-knockout-bracket">
    <div class="bracket-header">
      <h3 class="bracket-title">
        {{ bracketName }}
      </h3>
      <el-tag :type="getBracketStatusType()" size="large">
        {{ getBracketStatusText() }}
      </el-tag>
    </div>

    <div class="bracket-container">
      <!-- 第一轮 -->
      <div v-if="filteredRound1.length > 0" class="bracket-round">
        <div class="round-header">
          <h4>第一轮</h4>
          <el-tag :type="getRoundStatusType(filteredRound1)" size="small">
            {{ getRoundStatusText(filteredRound1) }}
          </el-tag>
        </div>
        <div class="matches-column">
          <ClauchMatchCard
            v-for="match in filteredRound1"
            :key="match.id"
            :match="match"
            :simulating="simulatingMatchId === match.id"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>

      <!-- 连接线 -->
      <div class="bracket-connector">
        <svg class="connector-svg">
          <line x1="0%" y1="12.5%" x2="50%" y2="25%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="0%" y1="37.5%" x2="50%" y2="25%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="50%" y1="25%" x2="100%" y2="25%" stroke="#e4e7ed" stroke-width="2" />

          <line x1="0%" y1="62.5%" x2="50%" y2="75%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="0%" y1="87.5%" x2="50%" y2="75%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="50%" y1="75%" x2="100%" y2="75%" stroke="#e4e7ed" stroke-width="2" />
        </svg>
      </div>

      <!-- 半决赛 -->
      <div v-if="filteredSemiFinals.length > 0" class="bracket-round">
        <div class="round-header">
          <h4>半决赛</h4>
          <el-tag :type="getRoundStatusType(filteredSemiFinals)" size="small">
            {{ getRoundStatusText(filteredSemiFinals) }}
          </el-tag>
        </div>
        <div class="matches-column semi-finals">
          <ClauchMatchCard
            v-for="match in filteredSemiFinals"
            :key="match.id"
            :match="match"
            :simulating="simulatingMatchId === match.id"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>

      <!-- 连接线 -->
      <div class="bracket-connector">
        <svg class="connector-svg">
          <line x1="0%" y1="25%" x2="50%" y2="50%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="0%" y1="75%" x2="50%" y2="50%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="50%" y1="50%" x2="100%" y2="50%" stroke="#e4e7ed" stroke-width="2" />
        </svg>
      </div>

      <!-- 决赛 -->
      <div v-if="filteredFinal.length > 0" class="bracket-round">
        <div class="round-header">
          <h4>决赛</h4>
          <el-tag :type="getRoundStatusType(filteredFinal)" size="small">
            {{ getRoundStatusText(filteredFinal) }}
          </el-tag>
        </div>
        <div class="matches-column final">
          <ClauchMatchCard
            v-for="match in filteredFinal"
            :key="match.id"
            :match="match"
            :simulating="simulatingMatchId === match.id"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>
    </div>

    <!-- 决赛胜者信息 -->
    <div v-if="finalWinner" class="winner-section">
      <el-tag type="success" size="large" effect="dark">
        {{ bracket === 'east' ? '东半区' : '西半区' }}冠军: {{ finalWinner.teamName }}
      </el-tag>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ClauchMatch, ClauchKnockoutBracket } from '@/types/clauch'
import ClauchMatchCard from './ClauchMatchCard.vue'

interface Props {
  knockout: ClauchKnockoutBracket
  bracket: 'east' | 'west'
  simulatingMatchId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  simulatingMatchId: null
})

const emit = defineEmits<{
  (e: 'simulate-match', match: ClauchMatch): void
  (e: 'view-detail', matchId: string | number): void
}>()

/**
 * 半区名称
 */
const bracketName = computed(() => {
  return props.bracket === 'east' ? '东半区对阵' : '西半区对阵'
})

/**
 * 过滤后的第一轮比赛（排除 undefined/null）
 * 注意：后端返回的字段名是 round1，不是 quarterMatches
 */
const filteredRound1 = computed(() => {
  // 兼容后端返回的 round1 字段
  const matches = (props.knockout.round1 || props.knockout.quarterMatches || []).filter((m): m is ClauchMatch => m != null)
  return matches
})

/**
 * 过滤后的半决赛比赛（排除 undefined/null）
 * 注意：后端返回的字段名是 semiFinals，不是 semiMatches
 */
const filteredSemiFinals = computed(() => {
  // 兼容后端返回的 semiFinals 字段
  const matches = (props.knockout.semiFinals || props.knockout.semiMatches || []).filter((m): m is ClauchMatch => m != null)
  return matches
})

/**
 * 过滤后的决赛比赛（排除 undefined/null）
 * 注意：后端返回的字段名是 final (数组)，不是 finalMatch (对象)
 */
const filteredFinal = computed((): ClauchMatch[] => {
  // 兼容后端返回的 final 数组字段
  if (props.knockout.final && Array.isArray(props.knockout.final) && props.knockout.final.length > 0) {
    const firstMatch = props.knockout.final[0]
    return firstMatch ? [firstMatch] : []
  }
  // 兼容旧的 finalMatch 对象字段
  if (props.knockout.finalMatch) {
    return [props.knockout.finalMatch]
  }
  return []
})

/**
 * 获取所有比赛
 */
const allMatches = computed((): ClauchMatch[] => {
  return [
    ...filteredRound1.value,
    ...filteredSemiFinals.value,
    ...filteredFinal.value
  ]
})

/**
 * 获取半区状态类型
 */
const getBracketStatusType = () => {
  const allCompleted = allMatches.value.every(m => m.status === 'completed')
  if (allCompleted) return 'success'

  const anyInProgress = allMatches.value.some(m => m.status === 'in_progress')
  if (anyInProgress) return 'warning'

  return 'info'
}

/**
 * 获取半区状态文本
 */
const getBracketStatusText = () => {
  const completed = allMatches.value.filter(m => m.status === 'completed').length
  const total = allMatches.value.length

  if (completed === total) return '已完成'
  return `${completed}/${total}`
}

/**
 * 获取轮次状态类型
 */
const getRoundStatusType = (matches: (ClauchMatch | undefined)[]) => {
  const validMatches = matches.filter((m): m is ClauchMatch => m != null)
  if (validMatches.length === 0) return 'info'

  const allCompleted = validMatches.every(m => m.status === 'completed')
  if (allCompleted) return 'success'

  const anyInProgress = validMatches.some(m => m.status === 'in_progress')
  if (anyInProgress) return 'warning'

  return 'info'
}

/**
 * 获取轮次状态文本
 */
const getRoundStatusText = (matches: (ClauchMatch | undefined)[]) => {
  const validMatches = matches.filter((m): m is ClauchMatch => m != null)
  if (validMatches.length === 0) return '待定'

  const completed = validMatches.filter(m => m.status === 'completed').length
  const total = validMatches.length

  if (completed === total) return '已完成'
  return `${completed}/${total}`
}

/**
 * 决赛胜者
 */
const finalWinner = computed(() => {
  const finalMatch = props.knockout.finalMatch
  if (!finalMatch || finalMatch.status !== 'completed' || !finalMatch.winnerId) return null

  return {
    teamId: finalMatch.winnerId,
    teamName: finalMatch.winnerId === finalMatch.teamAId
      ? finalMatch.teamAName
      : finalMatch.teamBName
  }
})

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
</script>

<style scoped>
.clauch-knockout-bracket {
  background: transparent;
}

.bracket-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 2px solid #e4e7ed;
}

.bracket-title {
  margin: 0;
  font-size: 20px;
  font-weight: bold;
  color: #303133;
}

.bracket-container {
  display: flex;
  gap: 20px;
  min-width: max-content;
  padding: 20px 0;
}

.bracket-round {
  flex: 1;
  min-width: 300px;
  display: flex;
  flex-direction: column;
}

.round-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 16px;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 4px;
}

.round-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #606266;
}

.matches-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
  flex: 1;
  justify-content: space-around;
}

.matches-column.semi-finals {
  justify-content: space-around;
  padding: 50px 0;
}

.matches-column.final {
  justify-content: center;
  padding: 100px 0;
}

.bracket-connector {
  width: 100px;
  min-width: 100px;
  align-self: stretch;
  padding: 0 10px;
}

.connector-svg {
  width: 100%;
  height: 100%;
}

.winner-section {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 2px solid #e4e7ed;
  display: flex;
  justify-content: center;
  align-items: center;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .bracket-container {
    flex-direction: column;
  }

  .bracket-round {
    min-width: auto;
  }

  .bracket-connector {
    width: 100%;
    height: 60px;
    min-width: auto;
  }

  .connector-svg {
    transform: rotate(90deg);
  }

  .matches-column.semi-finals,
  .matches-column.final {
    padding: 0;
  }
}
</style>
