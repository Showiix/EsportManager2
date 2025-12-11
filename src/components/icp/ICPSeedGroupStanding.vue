<template>
  <div class="icp-seed-group-standing">
    <!-- 种子组信息 -->
    <div class="group-info">
      <div class="group-title">
        <span class="seed-badge">{{ getSeedLabel(group.seedNumber) }}</span>
        <span class="group-name">{{ group.groupName }}组</span>
      </div>
      <el-tag v-if="group.isComplete" type="success" size="small">已完成</el-tag>
      <el-tag v-else type="warning" size="small">进行中</el-tag>
    </div>

    <!-- 积分榜 -->
    <el-table :data="group.standings" stripe class="standings-table">
      <el-table-column label="排名" width="70" align="center">
        <template #default="{ row }">
          <div class="rank-cell" :class="{ 'has-badge': row.hasBadge }">
            <span v-if="row.hasBadge" class="badge-icon">🏅</span>
            <span class="rank-number">{{ row.position }}</span>
          </div>
        </template>
      </el-table-column>

      <el-table-column label="队伍" min-width="150">
        <template #default="{ row }">
          <div class="team-cell">
            <span class="team-name">{{ row.teamName }}</span>
            <el-tag :type="getRegionTagType(row.region)" size="small">
              {{ row.region }}
            </el-tag>
          </div>
        </template>
      </el-table-column>

      <el-table-column label="场次" width="70" align="center" prop="matchesPlayed" />
      <el-table-column label="胜" width="60" align="center" prop="wins" />
      <el-table-column label="负" width="60" align="center" prop="losses" />
      <el-table-column label="积分" width="70" align="center">
        <template #default="{ row }">
          <span class="points-cell">{{ row.points }}</span>
        </template>
      </el-table-column>
      <el-table-column label="小局" width="80" align="center">
        <template #default="{ row }">
          <span>{{ row.roundsWon }}-{{ row.roundsLost }}</span>
        </template>
      </el-table-column>
      <el-table-column label="净胜" width="70" align="center">
        <template #default="{ row }">
          <span :class="{ 'positive': row.roundDifferential > 0, 'negative': row.roundDifferential < 0 }">
            {{ row.roundDifferential > 0 ? '+' : '' }}{{ row.roundDifferential }}
          </span>
        </template>
      </el-table-column>
    </el-table>

    <!-- 比赛列表 -->
    <div class="matches-section">
      <h4>比赛列表</h4>
      <div class="matches-grid">
        <div
          v-for="match in group.matches"
          :key="match.id"
          class="match-card"
          :class="{ completed: match.status === 'completed' }"
        >
          <div class="match-teams">
            <div class="team" :class="{ winner: match.winnerId === match.teamAId }">
              <span class="team-name">{{ match.teamAName }}</span>
              <el-tag :type="getRegionTagType(match.teamARegion)" size="small">
                {{ match.teamARegion }}
              </el-tag>
            </div>
            <div class="vs-score">
              <template v-if="match.status === 'completed'">
                <span class="score">{{ match.scoreA }} - {{ match.scoreB }}</span>
              </template>
              <template v-else>
                <span class="vs">VS</span>
              </template>
            </div>
            <div class="team" :class="{ winner: match.winnerId === match.teamBId }">
              <span class="team-name">{{ match.teamBName }}</span>
              <el-tag :type="getRegionTagType(match.teamBRegion)" size="small">
                {{ match.teamBRegion }}
              </el-tag>
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
              <el-tag type="success" size="small">已完成</el-tag>
              <el-button type="info" size="small" text @click="$emit('view-match', match)">
                查看详情
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ICPSeedGroup, ICPMatch } from '@/types/icp'

interface Props {
  group: ICPSeedGroup
}

defineProps<Props>()

defineEmits<{
  (e: 'simulate-match', match: ICPMatch): void
  (e: 'view-match', match: ICPMatch): void
}>()

const getSeedLabel = (seed: number) => {
  const labels: Record<number, string> = {
    1: '一号种子',
    2: '二号种子',
    3: '三号种子',
    4: '四号种子'
  }
  return labels[seed] || `${seed}号种子`
}

const getRegionTagType = (region?: string) => {
  const typeMap: Record<string, any> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning'
  }
  return typeMap[region || ''] || 'info'
}
</script>

<style scoped lang="scss">
.icp-seed-group-standing {
  .group-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #e5e7eb;

    .group-title {
      display: flex;
      align-items: center;
      gap: 12px;

      .seed-badge {
        padding: 4px 12px;
        background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
        color: white;
        border-radius: 20px;
        font-size: 12px;
        font-weight: 600;
      }

      .group-name {
        font-size: 16px;
        font-weight: 600;
        color: #1f2937;
      }
    }
  }

  .standings-table {
    margin-bottom: 24px;

    .rank-cell {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;

      &.has-badge {
        .rank-number {
          color: #f59e0b;
          font-weight: 700;
        }
      }

      .badge-icon {
        font-size: 14px;
      }

      .rank-number {
        font-weight: 600;
      }
    }

    .team-cell {
      display: flex;
      align-items: center;
      gap: 8px;

      .team-name {
        font-weight: 600;
        color: #1f2937;
      }
    }

    .points-cell {
      font-weight: 700;
      color: #10b981;
    }

    .positive {
      color: #10b981;
      font-weight: 600;
    }

    .negative {
      color: #ef4444;
      font-weight: 600;
    }
  }

  .matches-section {
    h4 {
      margin: 0 0 16px 0;
      font-size: 14px;
      font-weight: 600;
      color: #6b7280;
    }

    .matches-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;

      .match-card {
        padding: 16px;
        background: #f9fafb;
        border-radius: 8px;
        border: 1px solid #e5e7eb;

        &.completed {
          background: #f0fdf4;
          border-color: #86efac;
        }

        .match-teams {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 12px;

          .team {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 4px;
            flex: 1;

            .team-name {
              font-weight: 600;
              font-size: 14px;
              color: #374151;
            }

            &.winner .team-name {
              color: #10b981;
            }
          }

          .vs-score {
            padding: 0 16px;

            .vs {
              font-size: 12px;
              color: #9ca3af;
              font-weight: 600;
            }

            .score {
              font-size: 16px;
              font-weight: 700;
              color: #1f2937;
            }
          }
        }

        .match-actions {
          display: flex;
          justify-content: center;

          .completed-actions {
            display: flex;
            align-items: center;
            gap: 8px;
          }
        }
      }
    }
  }
}
</style>
