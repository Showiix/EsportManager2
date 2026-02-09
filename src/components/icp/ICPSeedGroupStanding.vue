<template>
  <div class="icp-seed-group-standing">
    <!-- 种子组信息 -->
    <div class="group-info">
      <div class="group-title">
        <span class="seed-badge">{{ getSeedLabel(group.seedNumber) }}</span>
        <span class="group-name">{{ group.groupName }}组</span>
      </div>
      <span v-if="group.isComplete" class="status-tag status-success">已完成</span>
      <span v-else class="status-tag status-warning">进行中</span>
    </div>

    <!-- 积分榜 -->
    <el-table :data="group.standings" stripe class="standings-table">
      <el-table-column label="排名" width="70" align="center">
        <template #default="{ row }">
          <div class="rank-cell" :class="{ 'has-badge': row.hasBadge }">
            <span class="rank-number">{{ row.position }}</span>
          </div>
        </template>
      </el-table-column>

      <el-table-column label="队伍" min-width="150">
        <template #default="{ row }">
          <div class="team-cell">
            <span class="team-name">{{ row.teamName }}</span>
            <span class="region-tag" :class="row.region?.toLowerCase()">
              {{ row.region }}
            </span>
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
              <span class="region-tag" :class="match.teamARegion?.toLowerCase()">
                {{ match.teamARegion }}
              </span>
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
              <span class="region-tag" :class="match.teamBRegion?.toLowerCase()">
                {{ match.teamBRegion }}
              </span>
            </div>
          </div>

          <div class="match-actions">
            <button
              v-if="match.status === 'scheduled'"
              class="btn btn-primary"
              @click="$emit('simulate-match', match)"
            >
              模拟比赛
            </button>
            <div v-else class="completed-actions">
              <span class="status-tag status-success">已完成</span>
              <button class="btn btn-text" @click="$emit('view-match', match)">
                查看详情
              </button>
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
</script>

<style scoped>
.icp-seed-group-standing {
  background: #f8fafc;
  border-radius: 10px;
}

.group-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e2e8f0;
}

.group-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.seed-badge {
  padding: 4px 12px;
  background: #f1f5f9;
  color: #64748b;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid #e2e8f0;
}

.group-name {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
}

.status-tag {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  line-height: 1.6;
}

.status-success {
  background: #f0fdf4;
  color: #22c55e;
  border: 1px solid #bbf7d0;
}

.status-warning {
  background: #fefce8;
  color: #ca8a04;
  border: 1px solid #fde68a;
}

.standings-table {
  margin-bottom: 24px;
}

.rank-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.rank-cell.has-badge .rank-number {
  color: #6366f1;
  font-weight: 700;
}

.rank-number {
  font-weight: 600;
  color: #0f172a;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-cell .team-name {
  font-weight: 600;
  color: #0f172a;
}

.region-tag {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  background: #f1f5f9;
  color: #64748b;
  border: 1px solid #e2e8f0;
}

.region-tag.lpl {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}

.region-tag.lck {
  background: #eff6ff;
  color: #2563eb;
  border-color: #bfdbfe;
}

.region-tag.lec {
  background: #f0fdf4;
  color: #16a34a;
  border-color: #bbf7d0;
}

.region-tag.lcs {
  background: #fefce8;
  color: #ca8a04;
  border-color: #fde68a;
}

.points-cell {
  font-weight: 700;
  color: #22c55e;
}

.positive {
  color: #22c55e;
  font-weight: 600;
}

.negative {
  color: #ef4444;
  font-weight: 600;
}

.matches-section h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #64748b;
}

.matches-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.match-card {
  padding: 16px;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.match-card.completed {
  background: #f0fdf4;
  border-color: #bbf7d0;
}

.match-teams {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.match-teams .team {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex: 1;
}

.match-teams .team .team-name {
  font-weight: 600;
  font-size: 14px;
  color: #0f172a;
}

.match-teams .team.winner .team-name {
  color: #22c55e;
}

.vs-score {
  padding: 0 16px;
}

.vs-score .vs {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 600;
}

.vs-score .score {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
}

.match-actions {
  display: flex;
  justify-content: center;
}

.completed-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: opacity 0.2s;
}

.btn:hover {
  opacity: 0.85;
}

.btn-primary {
  background: #6366f1;
  color: #ffffff;
}

.btn-text {
  background: transparent;
  color: #64748b;
  padding: 6px 8px;
}

.btn-text:hover {
  color: #0f172a;
}
</style>
