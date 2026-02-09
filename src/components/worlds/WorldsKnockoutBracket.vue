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
        <div class="matches-column quarter">
          <WorldsMatchCard
            v-for="match in quarterFinalMatches"
            :key="match.id"
            :match="match"
            @simulate="handleSimulate"
            @view-detail="handleViewMatch"
          />
        </div>
      </div>

      <!-- 连接线 八强->半决赛 -->
      <div class="bracket-connector connector-merge"></div>

      <!-- 半决赛 -->
      <div class="bracket-round">
        <div class="round-header">
          <h4>半决赛</h4>
          <span class="status-badge" :class="getRoundStatusClass(semiFinalMatches)">
            {{ getRoundStatusText(semiFinalMatches) }}
          </span>
        </div>
        <div class="matches-column semi">
          <WorldsMatchCard
            v-for="match in semiFinalMatches"
            :key="match.id"
            :match="match"
            @simulate="handleSimulate"
            @view-detail="handleViewMatch"
          />
        </div>
      </div>

      <!-- 连接线 半决赛->决赛 -->
      <div class="bracket-connector connector-merge"></div>

      <!-- 决赛区域 -->
      <div class="bracket-round finals-round">
        <div class="round-header final-header">
          <h4>决赛</h4>
        </div>
        <div class="matches-column final">
          <WorldsMatchCard
            v-if="grandFinal"
            :match="grandFinal"
            :is-final="true"
            @simulate="handleSimulate"
            @view-detail="handleViewMatch"
          />
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
  gap: 0;
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
  margin-bottom: 16px;
  padding: 8px 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
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

.badge-primary {
  background: #eef2ff;
  color: #6366f1;
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

.matches-column {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  justify-content: space-around;
}

.matches-column.quarter {
  gap: 8px;
}

.matches-column.semi {
  justify-content: space-around;
  padding: 40px 0;
}

.matches-column.final {
  justify-content: center;
  padding: 80px 0;
}

.finals-round {
  min-width: 240px;
}

.bracket-connector {
  width: 60px;
  min-width: 60px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.bracket-connector::before {
  content: '';
  height: 28px;
  flex-shrink: 0;
}

.bracket-connector::after {
  content: '';
  flex: 1;
}

.connector-merge::after {
  background:
    linear-gradient(#cbd5e1, #cbd5e1) 0 25% / 50% 2px no-repeat,
    linear-gradient(#cbd5e1, #cbd5e1) 0 75% / 50% 2px no-repeat,
    linear-gradient(#cbd5e1, #cbd5e1) calc(50% - 1px) 50% / 2px 50% no-repeat,
    linear-gradient(#cbd5e1, #cbd5e1) 100% 50% / 50% 2px no-repeat;
}

.third-place-section {
  margin-top: 32px;
  padding: 20px;
  background: #f8fafc;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #94a3b8;
}

.third-place-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.third-place-header h4 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
}

.third-place-desc {
  font-size: 13px;
  color: #64748b;
}

.third-place-match {
  max-width: 300px;
}
</style>
