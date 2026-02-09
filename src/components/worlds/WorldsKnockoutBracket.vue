<template>
  <div class="worlds-knockout-bracket">
    <div class="bracket-container">
      <!-- 八强赛 -->
      <div class="bracket-round">
        <div class="round-header">
          <h4>八强赛</h4>
          <span class="status-badge" :class="getRoundStatusClass(quarterFinalMatches)">
            {{ getRoundStatusText(quarterFinalMatches) }}
          </span>
        </div>
        <div class="matches-column">
          <div v-for="match in quarterFinalMatches" :key="match.id" class="match-slot">
            <WorldsMatchCard
              :match="match"
              @simulate="handleSimulate"
              @view-detail="handleViewMatch"
            />
          </div>
        </div>
      </div>

      <!-- 连接线 八强->半决赛 -->
      <div class="bracket-connector">
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
      <div class="bracket-round">
        <div class="round-header">
          <h4>半决赛</h4>
          <span class="status-badge" :class="getRoundStatusClass(semiFinalMatches)">
            {{ getRoundStatusText(semiFinalMatches) }}
          </span>
        </div>
        <div class="matches-column">
          <div v-for="match in semiFinalMatches" :key="match.id" class="match-slot">
            <WorldsMatchCard
              :match="match"
              @simulate="handleSimulate"
              @view-detail="handleViewMatch"
            />
          </div>
        </div>
      </div>

      <!-- 连接线 半决赛->决赛 -->
      <div class="bracket-connector">
        <div class="connector-spacer"></div>
        <div class="connector-body">
          <div class="connector-group">
            <div class="cg-input"></div>
            <div class="cg-input"></div>
          </div>
        </div>
      </div>

      <!-- 决赛区域 -->
      <div class="bracket-round finals-round">
        <div class="round-header final-header">
          <h4>决赛</h4>
        </div>
        <div class="matches-column">
          <div v-if="grandFinal" class="match-slot">
            <WorldsMatchCard
              :match="grandFinal"
              :is-final="true"
              @simulate="handleSimulate"
              @view-detail="handleViewMatch"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 季军赛单独显示 -->
    <div v-if="thirdPlaceMatch" class="third-place-section">
      <div class="third-place-header">
        <h4>季军赛</h4>
        <span class="third-place-desc">半决赛败者争夺第三名</span>
      </div>
      <div class="third-place-match">
        <WorldsMatchCard
          :match="thirdPlaceMatch"
          :is-third-place="true"
          @simulate="handleSimulate"
          @view-detail="handleViewMatch"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import WorldsMatchCard from './WorldsMatchCard.vue'
import type { WorldsKnockoutMatch } from '@/types/index'

interface Props {
  knockoutMatches: WorldsKnockoutMatch[]
  thirdPlaceMatch?: WorldsKnockoutMatch
  grandFinal?: WorldsKnockoutMatch
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'simulate-match', match: WorldsKnockoutMatch): void
  (e: 'view-match', match: WorldsKnockoutMatch): void
}>()

const quarterFinalMatches = computed(() =>
  props.knockoutMatches.filter(m => m.round === 'QUARTER_FINAL')
)

const semiFinalMatches = computed(() =>
  props.knockoutMatches.filter(m => m.round === 'SEMI_FINAL')
)

const getRoundStatusClass = (matches: WorldsKnockoutMatch[]) => {
  if (matches.length === 0) return 'badge-info'
  const allCompleted = matches.every(m => m.status === 'completed')
  if (allCompleted) return 'badge-success'
  const anyInProgress = matches.some(m => m.status === 'in_progress')
  if (anyInProgress) return 'badge-warning'
  return 'badge-info'
}

const getRoundStatusText = (matches: WorldsKnockoutMatch[]) => {
  if (matches.length === 0) return '待定'
  const completed = matches.filter(m => m.status === 'completed').length
  const total = matches.length
  if (completed === total) return '已完成'
  return `${completed}/${total}`
}

const handleSimulate = (match: WorldsKnockoutMatch) => {
  emit('simulate-match', match)
}

const handleViewMatch = (match: WorldsKnockoutMatch) => {
  emit('view-match', match)
}
</script>

<style scoped>
.worlds-knockout-bracket {
  /* root container */
}

.bracket-container {
  display: flex;
  align-items: stretch;
  min-width: max-content;
  padding: 20px 0;
  overflow-x: auto;
}

.bracket-round {
  display: flex;
  flex-direction: column;
  min-width: 220px;
}

.round-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 12px;
  padding: 8px 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  height: 28px;
}

.round-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.round-header.final-header {
  background: #f8fafc;
  border-left: 3px solid #6366f1;
}

.round-header.final-header h4 {
  color: #6366f1;
  font-size: 16px;
}

.status-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
  line-height: 1.4;
}

.badge-success {
  background: #f0fdf4;
  color: #22c55e;
}

.badge-warning {
  background: #fefce8;
  color: #ca8a04;
}

.badge-info {
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

.finals-round {
  min-width: 240px;
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

/* 季军赛 */
.third-place-section {
  margin-top: 24px;
  padding: 16px;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #94a3b8;
}

.third-place-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.third-place-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.third-place-desc {
  font-size: 12px;
  color: #64748b;
}

.third-place-match {
  max-width: 300px;
}
</style>
