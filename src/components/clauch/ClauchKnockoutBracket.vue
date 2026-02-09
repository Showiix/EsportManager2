<template>
  <div class="clauch-knockout-bracket">
    <div class="bracket-header">
      <h3 class="bracket-title">{{ bracketName }}</h3>
      <span class="status-badge" :class="getBracketStatusType()">
        {{ getBracketStatusText() }}
      </span>
    </div>

    <div class="bracket-container">
      <!-- 第一轮 -->
      <div v-if="filteredRound1.length > 0" class="bracket-round">
        <div class="round-header">
          <span class="round-name">第一轮</span>
          <span class="round-status" :class="getRoundStatusType(filteredRound1)">
            {{ getRoundStatusText(filteredRound1) }}
          </span>
        </div>
        <div class="matches-column">
          <div v-for="match in filteredRound1" :key="match.id" class="match-slot">
            <ClauchMatchCard
              :match="match"
              :simulating="simulatingMatchId === match.id"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewDetail"
            />
          </div>
        </div>
      </div>

      <!-- 连接线: 第一轮 → 半决赛 -->
      <div v-if="filteredRound1.length > 0 && filteredSemiFinals.length > 0" class="bracket-connector">
        <div class="connector-spacer"></div>
        <div class="connector-body">
          <div class="connector-group">
            <div class="cg-input"></div>
            <div class="cg-input"></div>
          </div>
          <div class="connector-group">
            <div class="cg-input"></div>
            <div class="cg-input"></div>
          </div>
        </div>
      </div>

      <!-- 半决赛 -->
      <div v-if="filteredSemiFinals.length > 0" class="bracket-round">
        <div class="round-header">
          <span class="round-name">半决赛</span>
          <span class="round-status" :class="getRoundStatusType(filteredSemiFinals)">
            {{ getRoundStatusText(filteredSemiFinals) }}
          </span>
        </div>
        <div class="matches-column">
          <div v-for="match in filteredSemiFinals" :key="match.id" class="match-slot">
            <ClauchMatchCard
              :match="match"
              :simulating="simulatingMatchId === match.id"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewDetail"
            />
          </div>
        </div>
      </div>

      <!-- 连接线: 半决赛 → 决赛 -->
      <div v-if="filteredSemiFinals.length > 0 && filteredFinal.length > 0" class="bracket-connector">
        <div class="connector-spacer"></div>
        <div class="connector-body">
          <div class="connector-group">
            <div class="cg-input"></div>
            <div class="cg-input"></div>
          </div>
        </div>
      </div>

      <!-- 决赛 -->
      <div v-if="filteredFinal.length > 0" class="bracket-round">
        <div class="round-header">
          <span class="round-name">决赛</span>
          <span class="round-status" :class="getRoundStatusType(filteredFinal)">
            {{ getRoundStatusText(filteredFinal) }}
          </span>
        </div>
        <div class="matches-column">
          <div v-for="match in filteredFinal" :key="match.id" class="match-slot">
            <ClauchMatchCard
              :match="match"
              :simulating="simulatingMatchId === match.id"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewDetail"
            />
          </div>
        </div>
      </div>
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
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e2e8f0;
}

.bracket-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
}

.status-badge {
  display: inline-block;
  padding: 2px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 10px;
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

.bracket-container {
  display: flex;
  align-items: stretch;
  overflow-x: auto;
  padding: 12px 0;
}

.bracket-round {
  min-width: 200px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.round-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 12px;
  padding: 6px 12px;
  background: #f8fafc;
  border-radius: 6px;
  height: 28px;
}

.round-name {
  font-size: 13px;
  font-weight: 600;
  color: #64748b;
}

.round-status {
  font-size: 11px;
  font-weight: 500;
  padding: 1px 8px;
  border-radius: 8px;
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

/* 比赛列 - 使用 flex-1 slot 确保对齐 */
.matches-column {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.match-slot {
  flex: 1;
  display: flex;
  align-items: center;
  padding: 6px 0;
}

.match-slot > :deep(*) {
  width: 100%;
}

/* 连接线列 */
.bracket-connector {
  width: 40px;
  min-width: 40px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

/* 顶部留白，对齐 round-header (28px height + 12px margin) */
.connector-spacer {
  height: 40px;
  flex-shrink: 0;
}

.connector-body {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* 每一对比赛的连接组 */
.connector-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
}

/* 每个输入槽位对应一场比赛 */
.cg-input {
  flex: 1;
  position: relative;
}

/* 从比赛卡片中心引出的水平线 → 到中间竖线 */
.cg-input::after {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  width: 50%;
  height: 2px;
  background: #cbd5e1;
  transform: translateY(-1px);
}

/* 竖线：连接两条输入水平线 */
.connector-group::before {
  content: '';
  position: absolute;
  left: 50%;
  top: 25%;
  bottom: 25%;
  width: 2px;
  background: #cbd5e1;
  transform: translateX(-1px);
}

/* 输出水平线：从中间到下一轮比赛 */
.connector-group::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 50%;
  right: 0;
  height: 2px;
  background: #cbd5e1;
  transform: translateY(-1px);
}
</style>
