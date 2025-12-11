<template>
  <div class="swiss-round-bracket">
    <h3 class="bracket-title">瑞士轮对阵图 - 2胜晋级 / 2败淘汰</h3>

    <!-- 轮次展示 -->
    <div class="rounds-container">
      <!-- 第1轮 -->
      <div class="round-column">
        <div class="round-header">
          <h4>第1轮</h4>
          <el-tag type="info" size="small">随机对阵</el-tag>
        </div>
        <div class="matches-list">
          <div
            v-for="match in round1Matches"
            :key="match.id"
            class="match-card"
            :class="{ completed: match.winnerId }"
          >
            <div class="match-number">
              <el-tag size="small" type="primary">BO3 #{{ match.matchNumber }}</el-tag>
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
            <div v-if="!match.winnerId" class="match-actions">
              <el-button
                size="small"
                type="primary"
                @click="$emit('simulate-match', match)"
              >
                模拟比赛
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 第2轮 -->
      <div v-if="round2Matches.length > 0" class="round-column">
        <div class="round-header">
          <h4>第2轮</h4>
          <el-tag type="warning" size="small">同战绩对决</el-tag>
        </div>
        <div class="matches-list">
          <!-- 1-0组 -->
          <div v-if="round2WinnersMatches.length > 0" class="group-section">
            <div class="group-label winners-group">1-0组</div>
            <div
              v-for="match in round2WinnersMatches"
              :key="match.id"
              class="match-card winners"
              :class="{ completed: match.winnerId }"
            >
              <div class="match-number">
                <el-tag size="small" type="success">BO3 #{{ match.matchNumber }}</el-tag>
              </div>

              <div class="team-row" :class="{ winner: match.winnerId === match.teamAId }">
                <span class="team-name">{{ match.teamAName || '待定' }}</span>
                <span v-if="match.winnerId" class="team-score">{{ match.scoreA }}</span>
              </div>

              <div class="team-row" :class="{ winner: match.winnerId === match.teamBId }">
                <span class="team-name">{{ match.teamBName || '待定' }}</span>
                <span v-if="match.winnerId" class="team-score">{{ match.scoreB }}</span>
              </div>

              <div v-if="!match.winnerId" class="match-actions">
                <el-button size="small" type="primary" @click="$emit('simulate-match', match)">
                  模拟比赛
                </el-button>
              </div>
            </div>
          </div>

          <!-- 0-1组 -->
          <div v-if="round2LosersMatches.length > 0" class="group-section">
            <div class="group-label losers-group">0-1组</div>
            <div
              v-for="match in round2LosersMatches"
              :key="match.id"
              class="match-card losers"
              :class="{ completed: match.winnerId }"
            >
              <div class="match-number">
                <el-tag size="small" type="warning">BO3 #{{ match.matchNumber }}</el-tag>
              </div>

              <div class="team-row" :class="{ winner: match.winnerId === match.teamAId }">
                <span class="team-name">{{ match.teamAName || '待定' }}</span>
                <span v-if="match.winnerId" class="team-score">{{ match.scoreA }}</span>
              </div>

              <div class="team-row" :class="{ winner: match.winnerId === match.teamBId }">
                <span class="team-name">{{ match.teamBName || '待定' }}</span>
                <span v-if="match.winnerId" class="team-score">{{ match.scoreB }}</span>
              </div>

              <div v-if="!match.winnerId" class="match-actions">
                <el-button size="small" type="primary" @click="$emit('simulate-match', match)">
                  模拟比赛
                </el-button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 第3轮 -->
      <div v-if="round3Matches.length > 0" class="round-column">
        <div class="round-header">
          <h4>第3轮</h4>
          <el-tag type="danger" size="small">1-1决战</el-tag>
        </div>
        <div class="matches-list">
          <div class="group-section">
            <div class="group-label tied-group">1-1组（背水一战）</div>
            <div
              v-for="match in round3Matches"
              :key="match.id"
              class="match-card tied"
              :class="{ completed: match.winnerId }"
            >
              <div class="match-number">
                <el-tag size="small" type="danger">BO3 #{{ match.matchNumber }}</el-tag>
              </div>

              <div class="team-row" :class="{ winner: match.winnerId === match.teamAId }">
                <span class="team-name">{{ match.teamAName || '待定' }}</span>
                <span v-if="match.winnerId" class="team-score">{{ match.scoreA }}</span>
              </div>

              <div class="team-row" :class="{ winner: match.winnerId === match.teamBId }">
                <span class="team-name">{{ match.teamBName || '待定' }}</span>
                <span v-if="match.winnerId" class="team-score">{{ match.scoreB }}</span>
              </div>

              <div v-if="!match.winnerId" class="match-actions">
                <el-button size="small" type="primary" @click="$emit('simulate-match', match)">
                  模拟比赛
                </el-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface SwissMatch {
  id: number
  roundNumber: number
  matchNumber: number
  teamAId?: string
  teamBId?: string
  teamAName?: string
  teamBName?: string
  winnerId?: string
  scoreA: number
  scoreB: number
  status: string
}

interface Props {
  matches: SwissMatch[]
  currentRound?: number
}

const props = withDefaults(defineProps<Props>(), {
  currentRound: 0
})

defineEmits<{
  (e: 'simulate-match', match: SwissMatch): void
}>()

// 生成占位符比赛
function createPlaceholderMatches(roundNumber: number, count: number): SwissMatch[] {
  return Array.from({ length: count }, (_, i) => ({
    id: -(roundNumber * 100 + i + 1), // 负数ID表示占位符
    roundNumber,
    matchNumber: i + 1,
    teamAName: '待定',
    teamBName: '待定',
    scoreA: 0,
    scoreB: 0,
    status: 'pending'
  }))
}

// 第1轮比赛（所有队伍随机对阵）- 始终显示（4场）
const round1Matches = computed(() => {
  const realMatches = props.matches.filter(m => m.roundNumber === 1)
  if (realMatches.length > 0) {
    return realMatches
  }
  // 如果没有真实数据，显示占位符
  return createPlaceholderMatches(1, 4)
})

// 第2轮比赛 - 根据进度显示（4场）
const round2Matches = computed(() => {
  const realMatches = props.matches.filter(m => m.roundNumber === 2)
  if (realMatches.length > 0) {
    return realMatches
  }
  // 如果第1轮还未开始，不显示第2轮
  if (props.currentRound < 1) {
    return []
  }
  // 如果第1轮已完成，显示占位符
  return createPlaceholderMatches(2, 4)
})

// 第2轮 - 1-0组（胜者组）
const round2WinnersMatches = computed(() => {
  return round2Matches.value.slice(0, 2) // 前2场是1-0组
})

// 第2轮 - 0-1组（败者组）
const round2LosersMatches = computed(() => {
  return round2Matches.value.slice(2, 4) // 后2场是0-1组
})

// 第3轮比赛（1-1组决战）- 根据进度显示（2场）
const round3Matches = computed(() => {
  const realMatches = props.matches.filter(m => m.roundNumber === 3)
  if (realMatches.length > 0) {
    return realMatches
  }
  // 如果第2轮还未开始，不显示第3轮
  if (props.currentRound < 2) {
    return []
  }
  // 如果第2轮已完成，显示占位符
  return createPlaceholderMatches(3, 2)
})
</script>

<style scoped lang="scss">
.swiss-round-bracket {
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

        .group-section {
          .group-label {
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 13px;
            font-weight: 600;
            margin-bottom: 12px;
            text-align: center;

            &.winners-group {
              background: linear-gradient(135deg, #d1fae5 0%, #a7f3d0 100%);
              color: #065f46;
              border: 2px solid #10b981;
            }

            &.losers-group {
              background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
              color: #92400e;
              border: 2px solid #f59e0b;
            }

            &.tied-group {
              background: linear-gradient(135deg, #fee2e2 0%, #fecaca 100%);
              color: #991b1b;
              border: 2px solid #ef4444;
            }
          }

          .match-card {
            margin-bottom: 12px;
          }
        }

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

          &.winners {
            border-left: 4px solid #10b981;
          }

          &.losers {
            border-left: 4px solid #f59e0b;
          }

          &.tied {
            border-left: 4px solid #ef4444;
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
          }
        }
      }
    }
  }
}
</style>

