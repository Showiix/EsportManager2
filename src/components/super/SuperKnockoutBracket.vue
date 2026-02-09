<template>
  <div class="super-knockout-bracket">
    <div class="bracket-header">
      <h3 class="bracket-title">
        {{ stageTitle }}
      </h3>
      <span class="status-badge" :class="'status-' + getBracketStatusType()">
        {{ getBracketStatusText() }}
      </span>
    </div>

    <!-- 第二阶段：挑战者组 -->
    <div v-if="stage === 'challenger'" class="challenger-stage">
      <div class="stage-section">
        <h4 class="section-title">定位赛</h4>
        <p class="section-desc">挑战者组（5-8名）两两对阵，胜者进入胜者组</p>
        <div class="matches-row">
          <SuperMatchCard
            v-for="match in challengerStage?.positioningMatches || []"
            :key="match.id"
            :match="match"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewMatch"
          />
        </div>
      </div>

      <div class="stage-connector">
        <span class="connector-arrow">&#9662;</span>
      </div>

      <div class="stage-section">
        <h4 class="section-title">晋级赛</h4>
        <p class="section-desc">Fighter组胜者 vs 定位赛败者，胜者进入败者组</p>
        <div class="matches-row">
          <SuperMatchCard
            v-for="match in challengerStage?.promotionMatches || []"
            :key="match.id"
            :match="match"
            @simulate="handleSimulateMatch"
            @view-detail="handleViewMatch"
          />
        </div>
      </div>
    </div>

    <!-- 第三阶段：冠军赛预备战 -->
    <div v-if="stage === 'champion_prep'" class="champion-prep-stage">
      <div class="bracket-container">
        <!-- 胜者组 -->
        <div class="bracket-column winners">
          <div class="column-header winners-header">
            <span>胜者组</span>
          </div>
          <div class="match-slot">
            <SuperMatchCard
              v-if="championPrepStage?.winnersMatch"
              :match="championPrepStage.winnersMatch"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewMatch"
            />
            <div v-else class="empty-match">待定</div>
          </div>
        </div>

        <!-- CSS连接线 -->
        <div class="bracket-connector connector-merge"></div>

        <!-- 败者组 -->
        <div class="bracket-column losers">
          <div class="column-header losers-header">
            <span>败者组</span>
          </div>
          <div class="match-slot">
            <SuperMatchCard
              v-if="championPrepStage?.losersMatch"
              :match="championPrepStage.losersMatch"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewMatch"
            />
            <div v-else class="empty-match">待定</div>
          </div>
        </div>

        <!-- CSS连接线 -->
        <div class="bracket-connector connector-straight"></div>

        <!-- 败者组决赛 -->
        <div class="bracket-column losers-final">
          <div class="column-header final-header">
            <span>败者组决赛</span>
          </div>
          <div class="match-slot">
            <SuperMatchCard
              v-if="championPrepStage?.losersFinal"
              :match="championPrepStage.losersFinal"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewMatch"
            />
            <div v-else class="empty-match">待定</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 第四阶段：终极冠军赛 -->
    <div v-if="stage === 'final'" class="final-stage">
      <div class="final-bracket-container">
        <!-- 首轮 -->
        <div class="final-round">
          <div class="round-header">
            <h4>首轮</h4>
            <p class="round-desc">传奇组3/4名 vs 第三阶段晋级者</p>
          </div>
          <div class="matches-column">
            <SuperMatchCard
              v-for="match in finalStage?.round1 || []"
              :key="match.id"
              :match="match"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewMatch"
            />
          </div>
        </div>

        <!-- CSS连接线 -->
        <div class="bracket-connector connector-merge"></div>

        <!-- 次轮 -->
        <div class="final-round">
          <div class="round-header">
            <h4>次轮</h4>
            <p class="round-desc">首轮胜者 vs 传奇组1/2名</p>
          </div>
          <div class="matches-column">
            <SuperMatchCard
              v-for="match in finalStage?.round2 || []"
              :key="match.id"
              :match="match"
              @simulate="handleSimulateMatch"
              @view-detail="handleViewMatch"
            />
          </div>
        </div>

        <!-- CSS连接线 -->
        <div class="bracket-connector connector-merge"></div>

        <!-- 决赛区 -->
        <div class="final-round finals">
          <div class="round-header">
            <h4>决赛阶段</h4>
          </div>
          <div class="matches-column finals-column">
            <!-- 季军赛 -->
            <div v-if="finalStage?.thirdPlaceMatch" class="final-match third-place">
              <h5>季军赛</h5>
              <SuperMatchCard
                :match="finalStage.thirdPlaceMatch"
                @simulate="handleSimulateMatch"
                @view-detail="handleViewMatch"
              />
            </div>
            <!-- 总决赛 -->
            <div v-if="finalStage?.grandFinal" class="final-match grand-final">
              <h5>总决赛</h5>
              <SuperMatchCard
                :match="finalStage.grandFinal"
                @simulate="handleSimulateMatch"
                @view-detail="handleViewMatch"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SuperMatch, ChallengerStage, ChampionPrepStage, FinalStage } from '@/types/super'
import SuperMatchCard from './SuperMatchCard.vue'

interface Props {
  stage: 'challenger' | 'champion_prep' | 'final'
  challengerStage?: ChallengerStage | null
  championPrepStage?: ChampionPrepStage | null
  finalStage?: FinalStage | null
}

const props = withDefaults(defineProps<Props>(), {
  challengerStage: null,
  championPrepStage: null,
  finalStage: null
})

const emit = defineEmits<{
  (e: 'simulate-match', match: SuperMatch): void
  (e: 'view-match', match: SuperMatch): void
}>()

/**
 * 阶段标题
 */
const stageTitle = computed(() => {
  const titles = {
    challenger: '第二阶段：挑战者组定位赛与晋级赛',
    champion_prep: '第三阶段：冠军赛预备战',
    final: '第四阶段：终极冠军赛'
  }
  return titles[props.stage]
})

/**
 * 获取所有比赛
 */
const allMatches = computed((): SuperMatch[] => {
  const matches: SuperMatch[] = []

  if (props.stage === 'challenger' && props.challengerStage) {
    matches.push(...(props.challengerStage.positioningMatches || []))
    matches.push(...(props.challengerStage.promotionMatches || []))
  }

  if (props.stage === 'champion_prep' && props.championPrepStage) {
    if (props.championPrepStage.winnersMatch) matches.push(props.championPrepStage.winnersMatch)
    if (props.championPrepStage.losersMatch) matches.push(props.championPrepStage.losersMatch)
    if (props.championPrepStage.losersFinal) matches.push(props.championPrepStage.losersFinal)
  }

  if (props.stage === 'final' && props.finalStage) {
    matches.push(...(props.finalStage.round1 || []))
    matches.push(...(props.finalStage.round2 || []))
    if (props.finalStage.thirdPlaceMatch) matches.push(props.finalStage.thirdPlaceMatch)
    if (props.finalStage.grandFinal) matches.push(props.finalStage.grandFinal)
  }

  return matches.filter(m => m != null)
})

/**
 * 获取状态类型
 */
const getBracketStatusType = () => {
  if (allMatches.value.length === 0) return 'info'

  const allCompleted = allMatches.value.every(m => m.status === 'completed')
  if (allCompleted) return 'success'

  const anyInProgress = allMatches.value.some(m => m.status === 'in_progress')
  if (anyInProgress) return 'warning'

  return 'info'
}

/**
 * 获取状态文本
 */
const getBracketStatusText = () => {
  if (allMatches.value.length === 0) return '待开始'

  const completed = allMatches.value.filter(m => m.status === 'completed').length
  const total = allMatches.value.length

  if (completed === total) return '已完成'
  return `${completed}/${total}`
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
</script>

<style scoped>
.super-knockout-bracket {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 24px;
}

.bracket-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e2e8f0;
}

.bracket-title {
  margin: 0;
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

/* 第二阶段：挑战者组 */
.challenger-stage {
  padding: 16px 0;
}

.stage-section {
  margin-bottom: 24px;
}

.section-title {
  margin: 0 0 6px 0;
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
}

.section-desc {
  margin: 0 0 16px 0;
  font-size: 13px;
  color: #64748b;
}

.matches-row {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.stage-connector {
  display: flex;
  justify-content: center;
  margin: 12px 0;
  color: #94a3b8;
  font-size: 18px;
}

.connector-arrow {
  display: inline-block;
}

/* 第三阶段：冠军赛预备战 */
.champion-prep-stage {
  padding: 16px 0;
}

.bracket-container {
  display: flex;
  align-items: center;
  gap: 0;
  overflow-x: auto;
  padding: 16px 0;
}

.bracket-column {
  flex: 1;
  min-width: 320px;
  display: flex;
  flex-direction: column;
}

.column-header {
  padding: 10px 16px;
  border-radius: 4px;
  text-align: center;
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 12px;
  border: 1px solid #e2e8f0;
  background: #f8fafc;
}

.winners-header {
  color: #16a34a;
  border-color: #bbf7d0;
  background: #f0fdf4;
}

.losers-header {
  color: #d97706;
  border-color: #fde68a;
  background: #fffbeb;
}

.final-header {
  color: #6366f1;
  border-color: #c7d2fe;
  background: #eef2ff;
}

.match-slot {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

.empty-match {
  padding: 40px;
  background: #f8fafc;
  border: 2px dashed #e2e8f0;
  border-radius: 6px;
  color: #94a3b8;
  text-align: center;
  width: 100%;
}

/* CSS连接线 */
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
    linear-gradient(#cbd5e1,#cbd5e1) 0 25%/50% 2px no-repeat,
    linear-gradient(#cbd5e1,#cbd5e1) 0 75%/50% 2px no-repeat,
    linear-gradient(#cbd5e1,#cbd5e1) calc(50% - 1px) 50%/2px 50% no-repeat,
    linear-gradient(#cbd5e1,#cbd5e1) 100% 50%/50% 2px no-repeat;
}

.connector-straight::after {
  background: linear-gradient(#cbd5e1,#cbd5e1) 0 50%/100% 2px no-repeat;
}

/* 第四阶段：终极冠军赛 */
.final-stage {
  padding: 16px 0;
}

.final-bracket-container {
  display: flex;
  gap: 0;
  overflow-x: auto;
  padding: 16px 0;
}

.final-round {
  flex: 1;
  min-width: 320px;
}

.round-header {
  margin-bottom: 12px;
  padding: 10px 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
}

.round-header h4 {
  margin: 0 0 2px 0;
  font-size: 15px;
  font-weight: 600;
  color: #0f172a;
}

.round-desc {
  margin: 0;
  font-size: 12px;
  color: #64748b;
}

.matches-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
  justify-content: space-around;
  min-height: 300px;
}

.finals-column {
  gap: 24px;
}

.final-match {
  padding: 16px;
  border-radius: 6px;
}

.final-match h5 {
  margin: 0 0 12px 0;
  font-size: 15px;
  text-align: center;
  font-weight: 600;
}

.final-match.third-place {
  background: #fffbeb;
  border: 1px solid #fde68a;
}

.final-match.third-place h5 {
  color: #d97706;
}

.final-match.grand-final {
  background: #eef2ff;
  border: 1px solid #c7d2fe;
}

.final-match.grand-final h5 {
  color: #6366f1;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .bracket-container,
  .final-bracket-container {
    flex-direction: column;
  }

  .bracket-column,
  .final-round {
    min-width: auto;
  }

  .bracket-connector {
    width: 100%;
    height: 40px;
    min-width: auto;
    flex-direction: row;
  }

  .bracket-connector::before {
    height: auto;
    width: 28px;
    flex-shrink: 0;
  }

  .bracket-connector::after {
    background: linear-gradient(#cbd5e1,#cbd5e1) 50% 0/2px 100% no-repeat;
  }

  .matches-column {
    min-height: auto;
  }
}
</style>
