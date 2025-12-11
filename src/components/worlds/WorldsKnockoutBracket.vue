<template>
  <div class="worlds-knockout-bracket">
    <div class="bracket-container">
      <!-- 八强赛 -->
      <div class="bracket-round">
        <div class="round-header">
          <h4>八强赛</h4>
          <el-tag :type="getRoundStatusType(quarterFinalMatches)" size="small">
            {{ getRoundStatusText(quarterFinalMatches) }}
          </el-tag>
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
      <div class="bracket-connector">
        <svg class="connector-svg" viewBox="0 0 60 400" preserveAspectRatio="none">
          <line x1="0" y1="12.5%" x2="30" y2="25%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="0" y1="37.5%" x2="30" y2="25%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="30" y1="25%" x2="60" y2="25%" stroke="#e4e7ed" stroke-width="2" />

          <line x1="0" y1="62.5%" x2="30" y2="75%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="0" y1="87.5%" x2="30" y2="75%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="30" y1="75%" x2="60" y2="75%" stroke="#e4e7ed" stroke-width="2" />
        </svg>
      </div>

      <!-- 半决赛 -->
      <div class="bracket-round">
        <div class="round-header">
          <h4>半决赛</h4>
          <el-tag :type="getRoundStatusType(semiFinalMatches)" size="small">
            {{ getRoundStatusText(semiFinalMatches) }}
          </el-tag>
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
      <div class="bracket-connector">
        <svg class="connector-svg" viewBox="0 0 60 400" preserveAspectRatio="none">
          <line x1="0" y1="25%" x2="30" y2="50%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="0" y1="75%" x2="30" y2="50%" stroke="#e4e7ed" stroke-width="2" />
          <line x1="30" y1="50%" x2="60" y2="50%" stroke="#e4e7ed" stroke-width="2" />
        </svg>
      </div>

      <!-- 决赛区域 -->
      <div class="bracket-round finals-round">
        <div class="round-header final-header">
          <h4>🏆 决赛</h4>
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
        <h4>🥉 季军赛</h4>
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

const getRoundStatusType = (matches: WorldsKnockoutMatch[]) => {
  if (matches.length === 0) return 'info'
  const allCompleted = matches.every(m => m.status === 'completed')
  if (allCompleted) return 'success'
  const anyInProgress = matches.some(m => m.status === 'in_progress')
  if (anyInProgress) return 'warning'
  return 'info'
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

<style scoped lang="scss">
.worlds-knockout-bracket {
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
    background: #f5f7fa;
    border-radius: 6px;

    h4 {
      margin: 0;
      font-size: 14px;
      font-weight: 600;
      color: #606266;
    }

    &.final-header {
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);

      h4 {
        color: #92400e;
        font-size: 16px;
      }
    }
  }

  .matches-column {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    justify-content: space-around;

    &.quarter {
      gap: 8px;
    }

    &.semi {
      justify-content: space-around;
      padding: 40px 0;
    }

    &.final {
      justify-content: center;
      padding: 80px 0;
    }
  }

  .finals-round {
    min-width: 240px;
  }

  .bracket-connector {
    width: 60px;
    min-width: 60px;
    display: flex;
    align-items: center;
  }

  .connector-svg {
    width: 100%;
    height: 100%;
  }

  .third-place-section {
    margin-top: 32px;
    padding: 20px;
    background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
    border-radius: 12px;
    border: 2px solid #d97706;

    .third-place-header {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 16px;

      h4 {
        margin: 0;
        font-size: 18px;
        font-weight: 600;
        color: #92400e;
      }

      .third-place-desc {
        font-size: 13px;
        color: #b45309;
      }
    }

    .third-place-match {
      max-width: 300px;
    }
  }
}
</style>
