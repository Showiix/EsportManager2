<template>
  <div class="worlds-swiss-round">
    <div v-if="matches.length === 0" class="empty-state">
      <el-empty description="暂无比赛" />
    </div>

    <div v-else class="matches-grid">
      <div
        v-for="match in matches"
        :key="match.id"
        class="match-card"
        :class="{ completed: match.status === 'completed' }"
      >
        <div class="match-header">
          <span class="match-label">BO1</span>
          <el-tag v-if="match.status === 'completed'" type="success" size="small">已完成</el-tag>
          <el-tag v-else type="info" size="small">待进行</el-tag>
        </div>

        <div class="match-teams">
          <div class="team team-a" :class="{ winner: match.winnerId === match.teamAId }">
            <span class="team-name">{{ match.teamAName }}</span>
            <span v-if="match.status === 'completed'" class="team-score">{{ match.scoreA }}</span>
          </div>

          <div class="vs-divider">
            <span v-if="match.status === 'completed'" class="score-divider">-</span>
            <span v-else class="vs-text">VS</span>
          </div>

          <div class="team team-b" :class="{ winner: match.winnerId === match.teamBId }">
            <span class="team-name">{{ match.teamBName }}</span>
            <span v-if="match.status === 'completed'" class="team-score">{{ match.scoreB }}</span>
          </div>
        </div>

        <div class="match-actions">
          <el-button
            v-if="match.status === 'scheduled'"
            type="primary"
            size="small"
            @click="$emit('simulate-match', match)"
          >
            模拟比赛
          </el-button>
          <div v-else class="completed-actions">
            <div class="winner-info">
              <span class="winner-label">胜者:</span>
              <span class="winner-name">{{ match.winnerId === match.teamAId ? match.teamAName : match.teamBName }}</span>
            </div>
            <el-button
              type="info"
              size="small"
              text
              @click="$emit('view-match', match)"
            >
              查看详情
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { WorldsSwissMatch } from '@/types/index'

interface Props {
  matches: WorldsSwissMatch[]
  round: number
}

defineProps<Props>()

defineEmits<{
  (e: 'simulate-match', match: WorldsSwissMatch): void
  (e: 'view-match', match: WorldsSwissMatch): void
}>()
</script>

<style scoped lang="scss">
.worlds-swiss-round {
  .empty-state {
    padding: 40px 0;
  }

  .matches-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;

    .match-card {
      padding: 16px;
      background: #f9fafb;
      border-radius: 12px;
      border: 2px solid #e5e7eb;
      transition: all 0.3s ease;

      &:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
      }

      &.completed {
        background: #f0fdf4;
        border-color: #86efac;
      }

      .match-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 12px;

        .match-label {
          font-size: 12px;
          font-weight: 600;
          color: #6b7280;
          padding: 2px 8px;
          background: #e5e7eb;
          border-radius: 4px;
        }
      }

      .match-teams {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 12px;

        .team {
          display: flex;
          align-items: center;
          gap: 8px;
          flex: 1;

          .team-name {
            font-weight: 600;
            font-size: 15px;
            color: #374151;
          }

          .team-score {
            font-size: 20px;
            font-weight: 700;
            color: #1f2937;
          }

          &.winner {
            .team-name {
              color: #10b981;
            }
            .team-score {
              color: #10b981;
            }
          }

          &.team-b {
            justify-content: flex-end;
            text-align: right;
          }
        }

        .vs-divider {
          padding: 0 16px;

          .vs-text {
            font-size: 12px;
            font-weight: 600;
            color: #9ca3af;
          }

          .score-divider {
            font-size: 20px;
            font-weight: 700;
            color: #6b7280;
          }
        }
      }

      .match-actions {
        display: flex;
        justify-content: center;
        padding-top: 8px;
        border-top: 1px solid #e5e7eb;

        .completed-actions {
          display: flex;
          align-items: center;
          justify-content: space-between;
          width: 100%;
        }

        .winner-info {
          display: flex;
          align-items: center;
          gap: 6px;

          .winner-label {
            font-size: 12px;
            color: #6b7280;
          }

          .winner-name {
            font-size: 14px;
            font-weight: 600;
            color: #10b981;
          }
        }
      }
    }
  }
}
</style>
