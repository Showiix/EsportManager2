<template>
  <div class="worlds-swiss-round">
    <div v-if="matches.length === 0" class="empty-state">
      <p class="empty-text">暂无比赛</p>
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
          <span v-if="match.status === 'completed'" class="status-badge badge-success">已完成</span>
          <span v-else class="status-badge badge-info">待进行</span>
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
          <button
            v-if="match.status === 'scheduled'"
            class="simulate-btn"
            @click="$emit('simulate-match', match)"
          >
            模拟比赛
          </button>
          <div v-else class="completed-actions">
            <div class="winner-info">
              <span class="winner-label">胜者:</span>
              <span class="winner-name">{{ match.winnerId === match.teamAId ? match.teamAName : match.teamBName }}</span>
            </div>
            <button
              class="detail-btn"
              @click="$emit('view-match', match)"
            >
              查看详情
            </button>
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

<style scoped>
.worlds-swiss-round {
  /* root container */
}

.empty-state {
  padding: 40px 0;
  text-align: center;
}

.empty-text {
  font-size: 14px;
  color: #94a3b8;
}

.matches-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.match-card {
  padding: 16px;
  background: #ffffff;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #6366f1;
}

.match-card.completed {
  border-left-color: #22c55e;
  background: #f8fafc;
}

.match-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.match-label {
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  padding: 2px 8px;
  background: #f1f5f9;
  border-radius: 4px;
}

.status-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
  line-height: 1.4;
}

.badge-success {
  background: #f0fdf4;
  color: #22c55e;
}

.badge-info {
  background: #f1f5f9;
  color: #64748b;
}

.match-teams {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.team {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.team-name {
  font-weight: 600;
  font-size: 15px;
  color: #0f172a;
}

.team-score {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
}

.team.winner .team-name {
  color: #16a34a;
}

.team.winner .team-score {
  color: #16a34a;
}

.team.team-b {
  justify-content: flex-end;
  text-align: right;
}

.vs-divider {
  padding: 0 16px;
}

.vs-text {
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
}

.score-divider {
  font-size: 20px;
  font-weight: 700;
  color: #64748b;
}

.match-actions {
  display: flex;
  justify-content: center;
  padding-top: 8px;
  border-top: 1px solid #e2e8f0;
}

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
}

.winner-label {
  font-size: 12px;
  color: #64748b;
}

.winner-name {
  font-size: 14px;
  font-weight: 600;
  color: #16a34a;
}

.simulate-btn {
  padding: 4px 14px;
  font-size: 12px;
  font-weight: 600;
  color: #ffffff;
  background: #6366f1;
  border: 1px solid #6366f1;
  border-radius: 6px;
  cursor: pointer;
}

.detail-btn {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  color: #64748b;
  background: transparent;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  cursor: pointer;
}
</style>
