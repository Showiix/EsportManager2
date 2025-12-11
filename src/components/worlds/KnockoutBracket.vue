<template>
  <div class="knockout-bracket">
    <h3 class="bracket-title">淘汰赛对阵图 - BO5单败淘汰</h3>

    <!-- 轮次展示 -->
    <div class="rounds-container">
      <!-- 八强赛 -->
      <div v-if="quarterFinalMatches.length > 0" class="round-column">
        <div class="round-header">
          <h4>八强赛</h4>
          <el-tag type="danger" size="small">BO5</el-tag>
        </div>
        <div class="matches-list">
          <div
            v-for="match in quarterFinalMatches"
            :key="match.id"
            class="match-card"
            :class="{ completed: match.winnerId }"
          >
            <div class="match-number">
              <el-tag size="small" type="danger">
                <span v-if="match.teamAQuarterSlot && match.teamBQuarterSlot">
                  QF #{{ match.matchNumber }} (半区{{ match.teamAQuarterSlot }}vs{{ match.teamBQuarterSlot }})
                </span>
                <span v-else>
                  QF #{{ match.matchNumber }}
                </span>
              </el-tag>
            </div>

            <!-- 队伍A -->
            <div class="team-row" :class="{ winner: match.winnerId === match.teamAId }">
              <span class="team-name">{{ match.teamAName || '待定' }}</span>
              <span v-if="match.winnerId" class="team-score">
                {{ match.scoreA }}
              </span>
            </div>

            <!-- 队伍B -->
            <div class="team-row" :class="{ winner: match.winnerId === match.teamBId }">
              <span class="team-name">{{ match.teamBName || '待定' }}</span>
              <span v-if="match.winnerId" class="team-score">
                {{ match.scoreB }}
              </span>
            </div>

            <!-- 操作按钮 -->
            <div v-if="!match.winnerId && match.teamAId && match.teamBId" class="match-actions">
              <el-button size="small" type="primary" @click="$emit('simulate-match', match)">
                模拟比赛
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 半决赛 -->
      <div v-if="semiFinalMatches.length > 0" class="round-column">
        <div class="round-header">
          <h4>半决赛</h4>
          <el-tag type="warning" size="small">BO5</el-tag>
        </div>
        <div class="matches-list">
          <div
            v-for="match in semiFinalMatches"
            :key="match.id"
            class="match-card"
            :class="{ completed: match.winnerId }"
          >
            <div class="match-number">
              <el-tag size="small" type="warning">SF #{{ match.matchNumber }}</el-tag>
            </div>

            <div class="team-row" :class="{ winner: match.winnerId === match.teamAId }">
              <span class="team-name">{{ match.teamAName || '待定' }}</span>
              <span v-if="match.winnerId" class="team-score">{{ match.scoreA }}</span>
            </div>

            <div class="team-row" :class="{ winner: match.winnerId === match.teamBId }">
              <span class="team-name">{{ match.teamBName || '待定' }}</span>
              <span v-if="match.winnerId" class="team-score">{{ match.scoreB }}</span>
            </div>

            <div v-if="!match.winnerId && match.teamAId && match.teamBId" class="match-actions">
              <el-button size="small" type="primary" @click="$emit('simulate-match', match)">
                模拟比赛
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 决赛和季军赛 -->
      <div class="round-column finals-column">
        <div class="round-header">
          <h4>决赛阶段</h4>
          <el-tag type="success" size="small">BO5</el-tag>
        </div>
        <div class="matches-list">
          <!-- 季军赛 -->
          <div v-if="thirdPlaceMatch" class="match-card third-place" :class="{ completed: thirdPlaceMatch.winnerId }">
            <div class="match-number">
              <el-tag size="small" type="info">季军赛</el-tag>
            </div>

            <div class="team-row" :class="{ winner: thirdPlaceMatch.winnerId === thirdPlaceMatch.teamAId }">
              <span class="team-name">{{ thirdPlaceMatch.teamAName || '待定' }}</span>
              <span v-if="thirdPlaceMatch.winnerId" class="team-score">{{ thirdPlaceMatch.scoreA }}</span>
            </div>

            <div class="team-row" :class="{ winner: thirdPlaceMatch.winnerId === thirdPlaceMatch.teamBId }">
              <span class="team-name">{{ thirdPlaceMatch.teamBName || '待定' }}</span>
              <span v-if="thirdPlaceMatch.winnerId" class="team-score">{{ thirdPlaceMatch.scoreB }}</span>
            </div>

            <div v-if="!thirdPlaceMatch.winnerId && thirdPlaceMatch.teamAId && thirdPlaceMatch.teamBId" class="match-actions">
              <el-button size="small" type="primary" @click="$emit('simulate-match', thirdPlaceMatch)">
                模拟比赛
              </el-button>
            </div>
          </div>

          <!-- 决赛 -->
          <div v-if="finalMatch" class="match-card final" :class="{ completed: finalMatch.winnerId }">
            <div class="match-number">
              <el-tag size="small" type="success">🏆 总决赛</el-tag>
            </div>

            <div class="team-row" :class="{ winner: finalMatch.winnerId === finalMatch.teamAId }">
              <span class="team-name">{{ finalMatch.teamAName || '待定' }}</span>
              <span v-if="finalMatch.winnerId" class="team-score">{{ finalMatch.scoreA }}</span>
            </div>

            <div class="team-row" :class="{ winner: finalMatch.winnerId === finalMatch.teamBId }">
              <span class="team-name">{{ finalMatch.teamBName || '待定' }}</span>
              <span v-if="finalMatch.winnerId" class="team-score">{{ finalMatch.scoreB }}</span>
            </div>

            <div v-if="!finalMatch.winnerId && finalMatch.teamAId && finalMatch.teamBId" class="match-actions">
              <el-button size="small" type="primary" @click="$emit('simulate-match', finalMatch)">
                模拟比赛
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface KnockoutMatch {
  id: number | string
  round: string
  matchNumber?: number
  teamAId?: string
  teamBId?: string
  teamAName?: string
  teamBName?: string
  teamAQuarterSlot?: number
  teamBQuarterSlot?: number
  winnerId?: string
  scoreA?: number
  scoreB?: number
  status?: string
}

interface Props {
  matches: KnockoutMatch[]
}

const props = defineProps<Props>()

defineEmits<{
  (e: 'simulate-match', match: KnockoutMatch): void
}>()

// 生成占位符比赛
function createPlaceholderMatch(round: string, matchNumber: number): KnockoutMatch {
  return {
    id: -(matchNumber + round.length), // 负数ID表示占位符
    round,
    matchNumber,
    teamAName: '待定',
    teamBName: '待定',
    status: 'pending'
  }
}

// 八强赛（4场）- 始终显示
const quarterFinalMatches = computed(() => {
  const realMatches = props.matches.filter(m => m.round === 'QUARTER_FINAL')
  if (realMatches.length > 0) {
    return realMatches
  }
  // 没有数据时显示占位符
  return [1, 2, 3, 4].map(i => createPlaceholderMatch('QUARTER_FINAL', i))
})

// 半决赛（2场）- 始终显示
const semiFinalMatches = computed(() => {
  const realMatches = props.matches.filter(m => m.round === 'SEMI_FINAL')
  if (realMatches.length > 0) {
    return realMatches
  }
  // 没有数据时显示占位符
  return [1, 2].map(i => createPlaceholderMatch('SEMI_FINAL', i))
})

// 季军赛（1场）- 始终显示
const thirdPlaceMatch = computed(() => {
  const realMatch = props.matches.find(m => m.round === 'THIRD_PLACE')
  if (realMatch) {
    return realMatch
  }
  // 没有数据时显示占位符
  return createPlaceholderMatch('THIRD_PLACE', 1)
})

// 决赛（1场）- 始终显示
const finalMatch = computed(() => {
  const realMatch = props.matches.find(m => m.round === 'FINAL')
  if (realMatch) {
    return realMatch
  }
  // 没有数据时显示占位符
  return createPlaceholderMatch('FINAL', 1)
})
</script>

<style scoped lang="scss">
.knockout-bracket {
  .bracket-title {
    margin: 0 0 24px 0;
    font-size: 20px;
    font-weight: 600;
    color: #1f2937;
    text-align: center;
  }

  .rounds-container {
    display: flex;
    gap: 24px;
    overflow-x: auto;
    padding: 16px;

    .round-column {
      min-width: 300px;
      flex-shrink: 0;

      &.finals-column {
        min-width: 320px;
      }

      .round-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 16px;
        padding: 12px;
        background: #f9fafb;
        border-radius: 8px;

        h4 {
          margin: 0;
          font-size: 16px;
          font-weight: 600;
          color: #374151;
        }
      }

      .matches-list {
        display: flex;
        flex-direction: column;
        gap: 20px;

        .match-card {
          background: white;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          padding: 12px;
          transition: all 0.3s;

          &:hover {
            border-color: #3b82f6;
            box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
            transform: translateY(-2px);
          }

          &.completed {
            background: #f9fafb;
          }

          &.third-place {
            border-left: 4px solid #f59e0b;
          }

          &.final {
            border-left: 4px solid #10b981;
            background: linear-gradient(135deg, #fef3c7 0%, #fff 100%);
          }

          .match-number {
            margin-bottom: 8px;
          }

          .team-row {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px;
            margin-bottom: 4px;
            border-radius: 6px;
            background: #f9fafb;
            border-left: 3px solid #d1d5db;
            transition: all 0.2s;

            .team-name {
              font-weight: 500;
              color: #374151;
              flex: 1;
            }

            .team-score {
              font-size: 18px;
              font-weight: 700;
              color: #6b7280;
              margin-left: 12px;
            }

            &.winner {
              background: linear-gradient(135deg, #d1fae5 0%, #ecfdf5 100%);
              border-left-color: #10b981;
              border-left-width: 4px;

              .team-name {
                color: #065f46;
                font-weight: 700;
              }

              .team-score {
                color: #10b981;
              }
            }
          }

          .match-actions {
            margin-top: 8px;
            text-align: center;

            .el-button {
              width: 100%;
            }
          }
        }
      }
    }
  }
}
</style>

