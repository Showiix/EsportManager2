<template>
  <div class="msi-bracket-view">
    <h3 class="bracket-title">MSI对阵图 - 双败淘汰赛制</h3>

    <!-- 整体流程图 -->
    <div class="bracket-flow">
      <!-- 第一层：预选赛 + 胜者组R1 -->
      <div class="flow-layer layer-1">
        <!-- 资格赛组 -->
        <div class="stage-block qualifier-block">
          <div class="block-header qualifier-header">
            <span class="header-title">资格赛组</span>
            <span class="header-subtitle">季军组 · BO5</span>
          </div>
          <div class="matches-row">
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
          <div class="flow-arrow down">
            <svg viewBox="0 0 24 40" class="arrow-svg">
              <path d="M12 0 L12 30 M6 24 L12 30 L18 24" stroke="currentColor" fill="none" stroke-width="2"/>
            </svg>
            <span class="arrow-label">2胜者</span>
          </div>
        </div>

        <!-- 挑战者组 -->
        <div class="stage-block challenger-block">
          <div class="block-header challenger-header">
            <span class="header-title">挑战者组</span>
            <span class="header-subtitle">亚军组 · BO5</span>
          </div>
          <div class="matches-row">
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
          <div class="flow-arrows-split">
            <div class="flow-arrow down-left">
              <svg viewBox="0 0 40 40" class="arrow-svg">
                <path d="M30 0 L30 20 Q30 30 20 30 L10 30 M16 24 L10 30 L16 36" stroke="currentColor" fill="none" stroke-width="2"/>
              </svg>
              <span class="arrow-label">2败者</span>
            </div>
            <div class="flow-arrow down-right">
              <svg viewBox="0 0 40 40" class="arrow-svg">
                <path d="M10 0 L10 20 Q10 30 20 30 L30 30 M24 24 L30 30 L24 36" stroke="currentColor" fill="none" stroke-width="2"/>
              </svg>
              <span class="arrow-label">2胜者</span>
            </div>
          </div>
        </div>

        <!-- 胜者组R1 -->
        <div class="stage-block winner-block">
          <div class="block-header winner-header">
            <span class="header-title">胜者组R1</span>
            <span class="header-subtitle">传奇组 · BO5</span>
          </div>
          <div class="matches-row">
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
          <div class="flow-arrows-split">
            <div class="flow-arrow down-left">
              <svg viewBox="0 0 40 40" class="arrow-svg loser-arrow">
                <path d="M30 0 L30 20 Q30 30 20 30 L10 30 M16 24 L10 30 L16 36" stroke="currentColor" fill="none" stroke-width="2"/>
              </svg>
              <span class="arrow-label loser-label">2败者</span>
            </div>
            <div class="flow-arrow down-right">
              <svg viewBox="0 0 40 40" class="arrow-svg winner-arrow">
                <path d="M10 0 L10 20 Q10 30 20 30 L30 30 M24 24 L30 30 L24 36" stroke="currentColor" fill="none" stroke-width="2"/>
              </svg>
              <span class="arrow-label winner-label">2胜者</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 第二层：败者组R1 + 败者组R2 + 胜者组决赛 -->
      <div class="flow-layer layer-2">
        <!-- 败者组R1 -->
        <div class="stage-block loser-r1-block">
          <div class="block-header loser-header">
            <span class="header-title">败者组R1</span>
            <span class="header-subtitle">资格赛胜者 vs 挑战者败者</span>
          </div>
          <div class="matches-row">
            <MSIMatchCard
              v-for="(match, index) in loserR1Matches"
              :key="match?.id || `lr1-${index}`"
              :match="match"
              :teams="bracket.qualifiedTeams"
              match-label="败者组R1"
              color-theme="amber"
              @simulate="handleSimulate"
              @view-detail="handleViewDetail"
            />
          </div>
          <div class="flow-arrow down">
            <svg viewBox="0 0 24 40" class="arrow-svg">
              <path d="M12 0 L12 30 M6 24 L12 30 L18 24" stroke="currentColor" fill="none" stroke-width="2"/>
            </svg>
            <span class="arrow-label">2胜者</span>
          </div>
        </div>

        <!-- 败者组R2 -->
        <div class="stage-block loser-r2-block">
          <div class="block-header loser-header">
            <span class="header-title">败者组R2</span>
            <span class="header-subtitle">挑战者胜者 vs R1胜者</span>
          </div>
          <div class="matches-row">
            <MSIMatchCard
              v-for="(match, index) in loserR2Matches"
              :key="match?.id || `lr2-${index}`"
              :match="match"
              :teams="bracket.qualifiedTeams"
              match-label="败者组R2"
              color-theme="amber"
              @simulate="handleSimulate"
              @view-detail="handleViewDetail"
            />
          </div>
          <div class="flow-arrow down">
            <svg viewBox="0 0 24 40" class="arrow-svg">
              <path d="M12 0 L12 30 M6 24 L12 30 L18 24" stroke="currentColor" fill="none" stroke-width="2"/>
            </svg>
            <span class="arrow-label">2胜者</span>
          </div>
        </div>

        <!-- 胜者组决赛 -->
        <div class="stage-block winner-final-block">
          <div class="block-header winner-final-header">
            <span class="header-title">胜者组决赛</span>
            <span class="header-subtitle">争夺总决赛席位</span>
          </div>
          <div class="matches-row single">
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
          <div class="flow-arrows-split">
            <div class="flow-arrow down-left">
              <svg viewBox="0 0 40 40" class="arrow-svg loser-arrow">
                <path d="M30 0 L30 20 Q30 30 20 30 L10 30 M16 24 L10 30 L16 36" stroke="currentColor" fill="none" stroke-width="2"/>
              </svg>
              <span class="arrow-label loser-label">败者</span>
            </div>
            <div class="flow-arrow down-right">
              <svg viewBox="0 0 40 40" class="arrow-svg winner-arrow">
                <path d="M10 0 L10 20 Q10 30 20 30 L30 30 M24 24 L30 30 L24 36" stroke="currentColor" fill="none" stroke-width="2"/>
              </svg>
              <span class="arrow-label winner-label">胜者进总决赛</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 第三层：败者组R3 + 败者组R4 -->
      <div class="flow-layer layer-3">
        <!-- 败者组R3 -->
        <div class="stage-block loser-r3-block">
          <div class="block-header loser-header">
            <span class="header-title">败者组R3</span>
            <span class="header-subtitle">R2胜者 vs 胜者组R1败者</span>
          </div>
          <div class="matches-row">
            <MSIMatchCard
              v-for="(match, index) in loserR3Matches"
              :key="match?.id || `lr3-${index}`"
              :match="match"
              :teams="bracket.qualifiedTeams"
              match-label="败者组R3"
              color-theme="amber"
              @simulate="handleSimulate"
              @view-detail="handleViewDetail"
            />
          </div>
          <div class="flow-arrow down">
            <svg viewBox="0 0 24 40" class="arrow-svg">
              <path d="M12 0 L12 30 M6 24 L12 30 L18 24" stroke="currentColor" fill="none" stroke-width="2"/>
            </svg>
            <span class="arrow-label">2胜者</span>
          </div>
        </div>

        <!-- 败者组R4 -->
        <div class="stage-block loser-r4-block">
          <div class="block-header loser-r4-header">
            <span class="header-title">败者组R4</span>
            <span class="header-subtitle">2名R3胜者对决</span>
          </div>
          <div class="matches-row single">
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
          <div class="flow-arrow down">
            <svg viewBox="0 0 24 40" class="arrow-svg">
              <path d="M12 0 L12 30 M6 24 L12 30 L18 24" stroke="currentColor" fill="none" stroke-width="2"/>
            </svg>
            <span class="arrow-label">胜者</span>
          </div>
        </div>

        <!-- 占位 - 保持对齐 -->
        <div class="stage-block placeholder-block"></div>
      </div>

      <!-- 第四层：败者组决赛 + 总决赛 -->
      <div class="flow-layer layer-4 finals-layer">
        <!-- 败者组决赛 -->
        <div class="stage-block loser-final-block">
          <div class="block-header loser-final-header">
            <span class="header-title">败者组决赛</span>
            <span class="header-subtitle">胜者组败者 vs R4胜者</span>
          </div>
          <div class="matches-row single">
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
          <div class="flow-arrow right">
            <svg viewBox="0 0 60 24" class="arrow-svg horizontal">
              <path d="M0 12 L50 12 M44 6 L50 12 L44 18" stroke="currentColor" fill="none" stroke-width="2"/>
            </svg>
            <span class="arrow-label">胜者进总决赛</span>
          </div>
        </div>

        <!-- 总决赛 -->
        <div class="stage-block grand-final-block">
          <div class="block-header grand-final-header">
            <span class="header-title">总决赛</span>
            <span class="header-subtitle">胜者组冠军 vs 败者组决赛胜者</span>
          </div>
          <div class="matches-row single">
            <MSIMatchCard
              v-if="grandFinalMatch"
              :match="grandFinalMatch"
              :teams="bracket.qualifiedTeams"
              match-label="MSI总决赛"
              color-theme="red"
              :is-final="true"
              @simulate="handleSimulate"
              @view-detail="handleViewDetail"
            />
          </div>
          <div v-if="grandFinalMatch?.status === 'completed'" class="champion-banner">
            <span class="champion-icon">冠</span>
            <span class="champion-text">MSI冠军: {{ getWinnerName(grandFinalMatch) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 赛制说明 -->
    <div class="bracket-legend">
      <div class="legend-item">
        <span class="legend-dot qualifier"></span>
        <span class="legend-text">资格赛组 (季军)</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot challenger"></span>
        <span class="legend-text">挑战者组 (亚军)</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot winner"></span>
        <span class="legend-text">胜者组 (冠军/传奇组)</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot loser"></span>
        <span class="legend-text">败者组 (攀登组)</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot final"></span>
        <span class="legend-text">决赛</span>
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
}

const props = defineProps<Props>()

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

const handleSimulate = (match: MSIMatch) => {
  emit('simulate-match', match)
}

const handleViewDetail = (match: MSIMatch) => {
  emit('view-match', match)
}

const getWinnerName = (match: any): string => {
  if (!match?.winnerId) return '待定'
  const team = props.bracket?.qualifiedTeams?.find(t => t.teamId?.toString() === match.winnerId.toString())
  return team?.teamName || '待定'
}
</script>

<style scoped lang="scss">
.msi-bracket-view {
  padding: 24px;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 16px;
  overflow-x: auto;

  .bracket-title {
    text-align: center;
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin-bottom: 32px;

    &::after {
      content: '';
      display: block;
      width: 80px;
      height: 4px;
      background: linear-gradient(90deg, #3b82f6, #8b5cf6);
      margin: 12px auto 0;
      border-radius: 2px;
    }
  }

  .bracket-flow {
    display: flex;
    flex-direction: column;
    gap: 24px;
    min-width: 1100px;
  }

  .flow-layer {
    display: flex;
    justify-content: center;
    gap: 32px;

    &.layer-1 {
      .stage-block {
        flex: 1;
        max-width: 320px;
      }
    }

    &.layer-2 {
      .stage-block {
        flex: 1;
        max-width: 300px;
      }
    }

    &.layer-3 {
      .stage-block {
        flex: 1;
        max-width: 280px;
      }
    }

    &.finals-layer {
      justify-content: center;
      gap: 48px;

      .stage-block {
        flex: none;
        width: 320px;
      }
    }
  }

  .stage-block {
    background: white;
    border-radius: 12px;
    padding: 16px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);

    &.placeholder-block {
      background: transparent;
      box-shadow: none;
    }
  }

  .block-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-radius: 8px;
    margin-bottom: 16px;

    .header-icon {
      font-size: 20px;
    }

    .header-title {
      font-size: 16px;
      font-weight: 700;
    }

    .header-subtitle {
      font-size: 12px;
      opacity: 0.8;
      width: 100%;
    }

    &.qualifier-header {
      background: linear-gradient(135deg, #d1fae5 0%, #a7f3d0 100%);
      color: #065f46;
    }

    &.challenger-header {
      background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
      color: #1e40af;
    }

    &.winner-header {
      background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
      color: #166534;
    }

    &.loser-header {
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      color: #92400e;
    }

    &.loser-r4-header {
      background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
      color: #9a3412;
    }

    &.winner-final-header {
      background: linear-gradient(135deg, #bbf7d0 0%, #86efac 100%);
      color: #14532d;
    }

    &.loser-final-header {
      background: linear-gradient(135deg, #fdba74 0%, #fb923c 100%);
      color: #7c2d12;
    }

    &.grand-final-header {
      background: linear-gradient(135deg, #fecaca 0%, #f87171 100%);
      color: #7f1d1d;
    }
  }

  .matches-row {
    display: flex;
    gap: 12px;
    justify-content: center;

    &.single {
      justify-content: center;
    }
  }

  .flow-arrow {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-top: 12px;

    .arrow-svg {
      width: 24px;
      height: 40px;
      color: #94a3b8;

      &.horizontal {
        width: 60px;
        height: 24px;
      }

      &.loser-arrow {
        color: #f59e0b;
      }

      &.winner-arrow {
        color: #22c55e;
      }
    }

    .arrow-label {
      font-size: 11px;
      color: #64748b;
      margin-top: 4px;

      &.loser-label {
        color: #d97706;
      }

      &.winner-label {
        color: #16a34a;
      }
    }

    &.right {
      flex-direction: row;
      margin-top: 0;
      margin-left: 12px;

      .arrow-label {
        margin-top: 0;
        margin-left: 8px;
      }
    }
  }

  .flow-arrows-split {
    display: flex;
    justify-content: space-around;
    margin-top: 12px;

    .flow-arrow {
      margin-top: 0;

      .arrow-svg {
        width: 40px;
        height: 40px;
      }
    }
  }

  .champion-banner {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 16px;
    margin-top: 16px;
    background: linear-gradient(135deg, #fef3c7 0%, #fde047 100%);
    border-radius: 8px;
    border: 2px solid #fbbf24;

    .champion-icon {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 40px;
      height: 40px;
      border-radius: 50%;
      background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
      color: white;
      font-size: 18px;
      font-weight: 700;
      animation: bounce 1s infinite;
    }

    .champion-text {
      font-size: 18px;
      font-weight: 700;
      color: #92400e;
    }
  }

  .bracket-legend {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 24px;
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid #e2e8f0;

    .legend-item {
      display: flex;
      align-items: center;
      gap: 8px;

      .legend-dot {
        width: 12px;
        height: 12px;
        border-radius: 50%;

        &.qualifier {
          background: linear-gradient(135deg, #10b981 0%, #059669 100%);
        }

        &.challenger {
          background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
        }

        &.winner {
          background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
        }

        &.loser {
          background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
        }

        &.final {
          background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
        }
      }

      .legend-text {
        font-size: 13px;
        color: #64748b;
      }
    }
  }
}

@keyframes bounce {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-5px);
  }
}
</style>
