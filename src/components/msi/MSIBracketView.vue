<template>
  <div class="msi-bracket-view">
    <h3 class="bracket-title">{{ tournamentName }}对阵图 - 双败淘汰赛制</h3>

    <!-- Entry Rounds -->
    <div class="entry-rounds">
      <div class="entry-block">
        <div class="entry-header qualifier">资格赛 <span class="entry-hint">季军组 · BO5</span></div>
        <div class="entry-matches">
          <MSIMatchCard
            v-for="(match, index) in qualifierMatches"
            :key="match?.id || `qual-${index}`"
            :match="match"
            :teams="bracket.qualifiedTeams"
            match-label="资格赛"
            color-theme="emerald"
            @simulate="handleSimulate"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>
      <div class="entry-block">
        <div class="entry-header challenger">挑战者赛 <span class="entry-hint">亚军组 · BO5</span></div>
        <div class="entry-matches">
          <MSIMatchCard
            v-for="(match, index) in challengerMatches"
            :key="match?.id || `chal-${index}`"
            :match="match"
            :teams="bracket.qualifiedTeams"
            match-label="挑战者赛"
            color-theme="blue"
            @simulate="handleSimulate"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>
      <div class="entry-block">
        <div class="entry-header winners">胜者组R1 <span class="entry-hint">传奇组 · BO5</span></div>
        <div class="entry-matches">
          <MSIMatchCard
            v-for="(match, index) in winnerR1Matches"
            :key="match?.id || `wr1-${index}`"
            :match="match"
            :teams="bracket.qualifiedTeams"
            match-label="胜者组"
            color-theme="green"
            @simulate="handleSimulate"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>
    </div>

    <!-- Winners Bracket Section -->
    <div class="bracket-section winners-section">
      <div class="section-label winners">胜者组</div>
      <div class="bracket-container">
        <div class="bracket-round">
          <div class="round-header">胜者组R1</div>
          <div class="matches-column">
            <div v-for="(match, index) in winnerR1Matches" :key="match?.id || `wr1b-${index}`" class="match-card-wrapper">
              <MSIMatchCard
                :match="match"
                :teams="bracket.qualifiedTeams"
                match-label="胜者组"
                color-theme="green"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
        <div class="bracket-connector connector-merge winners-color"></div>
        <div class="bracket-round">
          <div class="round-header">胜者组决赛</div>
          <div class="matches-column final">
            <div class="match-card-wrapper">
              <MSIMatchCard
                v-if="winnerFinalMatch"
                :match="winnerFinalMatch"
                :teams="bracket.qualifiedTeams"
                match-label="胜者组决赛"
                color-theme="green"
                :highlight="true"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Losers Bracket Section -->
    <div class="bracket-section losers-section">
      <div class="section-label losers">败者组</div>
      <div class="bracket-container">
        <!-- LR1 -->
        <div class="bracket-round">
          <div class="round-header">R1 <span class="drop-hint">(资格赛胜者 vs 挑战者败者)</span></div>
          <div class="matches-column">
            <div v-for="(match, index) in loserR1Matches" :key="match?.id || `lr1-${index}`" class="match-card-wrapper">
              <MSIMatchCard
                :match="match"
                :teams="bracket.qualifiedTeams"
                match-label="败者组R1"
                color-theme="amber"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
        <div class="bracket-connector connector-parallel losers-color"></div>
        <!-- LR2 -->
        <div class="bracket-round">
          <div class="round-header">R2</div>
          <div class="matches-column">
            <div v-for="(match, index) in loserR2Matches" :key="match?.id || `lr2-${index}`" class="match-card-wrapper">
              <MSIMatchCard
                :match="match"
                :teams="bracket.qualifiedTeams"
                match-label="败者组R2"
                color-theme="amber"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
        <div class="bracket-connector connector-merge losers-color"></div>
        <!-- LR3 -->
        <div class="bracket-round">
          <div class="round-header">R3 <span class="drop-hint">(+胜者组R1败者)</span></div>
          <div class="matches-column">
            <div v-for="(match, index) in loserR3Matches" :key="match?.id || `lr3-${index}`" class="match-card-wrapper">
              <MSIMatchCard
                :match="match"
                :teams="bracket.qualifiedTeams"
                match-label="败者组R3"
                color-theme="amber"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
        <div class="bracket-connector connector-merge losers-color"></div>
        <!-- LR4 -->
        <div class="bracket-round">
          <div class="round-header">R4</div>
          <div class="matches-column final">
            <div class="match-card-wrapper">
              <MSIMatchCard
                v-if="loserR4Match"
                :match="loserR4Match"
                :teams="bracket.qualifiedTeams"
                match-label="败者组R4"
                color-theme="amber"
                :highlight="true"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
        <div class="bracket-connector connector-straight losers-color"></div>
        <!-- Loser Final -->
        <div class="bracket-round">
          <div class="round-header">败者组决赛 <span class="drop-hint">(+胜者组败者)</span></div>
          <div class="matches-column final">
            <div class="match-card-wrapper">
              <MSIMatchCard
                v-if="loserFinalMatch"
                :match="loserFinalMatch"
                :teams="bracket.qualifiedTeams"
                match-label="败者组决赛"
                color-theme="orange"
                :highlight="true"
                @simulate="handleSimulate"
                @view-detail="handleViewDetail"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Grand Final Section -->
    <div class="bracket-section finals-section">
      <div class="section-label finals">总决赛</div>
      <div class="grand-final-area">
        <div class="match-card-wrapper">
          <MSIMatchCard
            v-if="grandFinalMatch"
            :match="grandFinalMatch"
            :teams="bracket.qualifiedTeams"
            :match-label="`${tournamentName}总决赛`"
            color-theme="red"
            :is-final="true"
            @simulate="handleSimulate"
            @view-detail="handleViewDetail"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MSIMatchCard from './MSIMatchCard.vue'
import type { MSIBracket, MSIMatch } from '@/types'

interface Props {
  bracket: MSIBracket
  tournamentName?: string  // 赛事名称，默认为 "MSI"
}

const props = withDefaults(defineProps<Props>(), {
  tournamentName: 'MSI'
})

const emit = defineEmits<{
  (e: 'simulate-match', match: MSIMatch): void
  (e: 'view-match', match: MSIMatch): void
}>()

// 获取所有比赛
const allMatches = computed(() => {
  if (!props.bracket?.rounds) return []
  return props.bracket.rounds.flatMap(r => r.matches || [])
})

// 预选赛 - 资格赛组 (季军组2场)
const qualifierMatches = computed(() => {
  return allMatches.value.filter(m => m.matchType === 'qualifier').slice(0, 2)
})

// 预选赛 - 挑战者组 (亚军组2场)
const challengerMatches = computed(() => {
  return allMatches.value.filter(m => m.matchType === 'challenger').slice(0, 2)
})

// 败者组第一轮 (2场)
const loserR1Matches = computed(() => {
  return allMatches.value.filter(m => m.matchType === 'loser_r1').slice(0, 2)
})

// 败者组第二轮 (2场)
const loserR2Matches = computed(() => {
  return allMatches.value.filter(m => m.matchType === 'loser_r2').slice(0, 2)
})

// 败者组第三轮 (2场)
const loserR3Matches = computed(() => {
  return allMatches.value.filter(m => m.matchType === 'loser_r3').slice(0, 2)
})

// 败者组第四轮 (1场)
const loserR4Match = computed(() => {
  return allMatches.value.find(m => m.matchType === 'loser_r4')
})

// 败者组决赛 (1场)
const loserFinalMatch = computed(() => {
  return allMatches.value.find(m => m.matchType === 'loser_final')
})

// 胜者组第一轮 (2场)
const winnerR1Matches = computed(() => {
  return allMatches.value.filter(m => m.matchType === 'winner_r1').slice(0, 2)
})

// 胜者组决赛 (1场)
const winnerFinalMatch = computed(() => {
  return allMatches.value.find(m => m.matchType === 'winner_final')
})

// 总决赛 (1场)
const grandFinalMatch = computed(() => {
  return allMatches.value.find(m => m.matchType === 'grand_final')
})

const handleSimulate = (match: any) => {
  emit('simulate-match', match as MSIMatch)
}

const handleViewDetail = (match: any) => {
  emit('view-match', match as MSIMatch)
}

</script>

<style scoped>
.msi-bracket-view {
  padding: 0;
}

.bracket-title {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f1f5f9;
}

/* Entry rounds - 3 columns */
.entry-rounds {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
}

.entry-block {
  flex: 1;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
}

.entry-header {
  padding: 8px 12px;
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid #f1f5f9;
}

.entry-header.qualifier {
  color: #059669;
  background: #f0fdf4;
}

.entry-header.challenger {
  color: #2563eb;
  background: #eff6ff;
}

.entry-header.winners {
  color: #16a34a;
  background: #f0fdf4;
}

.entry-hint {
  font-size: 11px;
  color: #94a3b8;
  font-weight: 400;
  margin-left: 6px;
}

.entry-matches {
  display: flex;
  gap: 8px;
  padding: 12px;
}

/* Bracket sections */
.bracket-section {
  border-radius: 10px;
  padding: 16px;
  border: 1px solid #e2e8f0;
  margin-bottom: 16px;
}

.bracket-section.winners-section {
  border-left: 3px solid #22c55e;
}

.bracket-section.losers-section {
  border-left: 3px solid #f59e0b;
}

.bracket-section.finals-section {
  border-left: 3px solid #6366f1;
}

.section-label {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f1f5f9;
}

.section-label.winners {
  color: #16a34a;
}

.section-label.losers {
  color: #d97706;
}

.section-label.finals {
  color: #6366f1;
}

.bracket-container {
  display: flex;
  align-items: stretch;
  gap: 0;
  overflow-x: auto;
}

.bracket-round {
  min-width: 180px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.round-header {
  text-align: center;
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  margin-bottom: 12px;
  padding: 4px 10px;
  background: #f8fafc;
  border-radius: 4px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.drop-hint {
  font-size: 10px;
  color: #94a3b8;
  font-weight: 400;
  margin-left: 4px;
}

.matches-column {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  justify-content: space-around;
  min-height: 200px;
}

.matches-column.final {
  justify-content: center;
  min-height: 100px;
}

.match-card-wrapper {
  display: flex;
  align-items: center;
}

/* CSS connectors */
.bracket-connector {
  width: 50px;
  min-width: 50px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.bracket-connector::before {
  content: '';
  height: 44px;
  flex-shrink: 0;
}

.bracket-connector::after {
  content: '';
  flex: 1;
}

.winners-color {
  --line-color: #22c55e;
}

.losers-color {
  --line-color: #f59e0b;
}

.connector-merge::after {
  background:
    linear-gradient(var(--line-color), var(--line-color)) 0 25% / 50% 2px no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) 0 75% / 50% 2px no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) calc(50% - 1px) 50% / 2px 50% no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) 100% 50% / 50% 2px no-repeat;
}

.connector-parallel::after {
  background:
    linear-gradient(var(--line-color), var(--line-color)) 0 25% / 100% 2px no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) 0 75% / 100% 2px no-repeat;
}

.connector-straight::after {
  background:
    linear-gradient(var(--line-color), var(--line-color)) 0 50% / 100% 2px no-repeat;
}

/* Grand final */
.grand-final-area {
  display: flex;
  justify-content: center;
  padding: 16px;
}
</style>
