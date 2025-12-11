<template>
  <div class="super-knockout-bracket">
    <div class="bracket-header">
      <h3 class="bracket-title">
        {{ stageTitle }}
      </h3>
      <el-tag :type="getBracketStatusType()" size="large">
        {{ getBracketStatusText() }}
      </el-tag>
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
        <el-icon><Bottom /></el-icon>
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

        <!-- 连接线 -->
        <div class="bracket-connector vertical">
          <svg class="connector-svg" viewBox="0 0 60 200" preserveAspectRatio="none">
            <path d="M 30 0 L 30 100 L 60 100" stroke="#e4e7ed" stroke-width="2" fill="none" />
            <path d="M 30 200 L 30 100" stroke="#e4e7ed" stroke-width="2" fill="none" />
          </svg>
        </div>

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

        <!-- 连接线 -->
        <div class="bracket-connector vertical">
          <svg class="connector-svg" viewBox="0 0 60 200" preserveAspectRatio="none">
            <path d="M 0 100 L 30 100 L 30 200" stroke="#e4e7ed" stroke-width="2" fill="none" />
          </svg>
        </div>

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

        <!-- 连接线 -->
        <div class="bracket-connector">
          <svg class="connector-svg" viewBox="0 0 100 300" preserveAspectRatio="none">
            <line x1="0" y1="25%" x2="50" y2="37.5%" stroke="#e4e7ed" stroke-width="2" />
            <line x1="0" y1="75%" x2="50" y2="62.5%" stroke="#e4e7ed" stroke-width="2" />
            <line x1="50" y1="37.5%" x2="100" y2="37.5%" stroke="#e4e7ed" stroke-width="2" />
            <line x1="50" y1="62.5%" x2="100" y2="62.5%" stroke="#e4e7ed" stroke-width="2" />
          </svg>
        </div>

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

        <!-- 连接线 -->
        <div class="bracket-connector">
          <svg class="connector-svg" viewBox="0 0 100 300" preserveAspectRatio="none">
            <line x1="0" y1="37.5%" x2="50" y2="50%" stroke="#e4e7ed" stroke-width="2" />
            <line x1="0" y1="62.5%" x2="50" y2="50%" stroke="#e4e7ed" stroke-width="2" />
            <line x1="50" y1="50%" x2="100" y2="50%" stroke="#e4e7ed" stroke-width="2" />
          </svg>
        </div>

        <!-- 决赛区 -->
        <div class="final-round finals">
          <div class="round-header">
            <h4>决赛阶段</h4>
          </div>
          <div class="matches-column finals-column">
            <!-- 季军赛 -->
            <div v-if="finalStage?.thirdPlaceMatch" class="final-match third-place">
              <h5>🥉 季军赛</h5>
              <SuperMatchCard
                :match="finalStage.thirdPlaceMatch"
                @simulate="handleSimulateMatch"
                @view-detail="handleViewMatch"
              />
            </div>
            <!-- 总决赛 -->
            <div v-if="finalStage?.grandFinal" class="final-match grand-final">
              <h5>🏆 总决赛</h5>
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
import { Bottom } from '@element-plus/icons-vue'
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
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
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

/* 第二阶段：挑战者组 */
.challenger-stage {
  padding: 20px;
}

.stage-section {
  margin-bottom: 24px;
}

.section-title {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.section-desc {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: #909399;
}

.matches-row {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.stage-connector {
  display: flex;
  justify-content: center;
  margin: 16px 0;
  font-size: 24px;
  color: #909399;
}

/* 第三阶段：冠军赛预备战 */
.champion-prep-stage {
  padding: 20px;
}

.bracket-container {
  display: flex;
  align-items: center;
  gap: 20px;
  overflow-x: auto;
  padding: 20px 0;
}

.bracket-column {
  flex: 1;
  min-width: 320px;
  display: flex;
  flex-direction: column;
}

.column-header {
  padding: 12px 16px;
  border-radius: 8px;
  text-align: center;
  font-weight: 600;
  margin-bottom: 16px;
}

.winners-header {
  background: linear-gradient(135deg, #fef3c7 0%, #fde047 100%);
  color: #92400e;
}

.losers-header {
  background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
  color: #1e40af;
}

.final-header {
  background: linear-gradient(135deg, #f3e8ff 0%, #ddd6fe 100%);
  color: #6b21a8;
}

.match-slot {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

.empty-match {
  padding: 40px;
  background: #f5f7fa;
  border: 2px dashed #e4e7ed;
  border-radius: 12px;
  color: #909399;
  text-align: center;
  width: 100%;
}

.bracket-connector.vertical {
  width: 60px;
  min-width: 60px;
  height: 200px;
}

.connector-svg {
  width: 100%;
  height: 100%;
}

/* 第四阶段：终极冠军赛 */
.final-stage {
  padding: 20px;
}

.final-bracket-container {
  display: flex;
  gap: 20px;
  overflow-x: auto;
  padding: 20px 0;
}

.final-round {
  flex: 1;
  min-width: 320px;
}

.round-header {
  margin-bottom: 16px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.round-header h4 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.round-desc {
  margin: 0;
  font-size: 12px;
  color: #909399;
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

.bracket-connector {
  width: 100px;
  min-width: 100px;
  display: flex;
  align-items: center;
}

.final-match {
  padding: 16px;
  border-radius: 12px;
}

.final-match h5 {
  margin: 0 0 12px 0;
  font-size: 16px;
  text-align: center;
}

.final-match.third-place {
  background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
  border: 2px solid #d97706;
}

.final-match.grand-final {
  background: linear-gradient(135deg, #fef3c7 0%, #fde047 100%);
  border: 2px solid #f59e0b;
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
    height: 60px;
    min-width: auto;
  }

  .bracket-connector.vertical {
    width: 100%;
    height: 60px;
  }

  .connector-svg {
    transform: rotate(90deg);
  }

  .matches-column {
    min-height: auto;
  }
}
</style>
