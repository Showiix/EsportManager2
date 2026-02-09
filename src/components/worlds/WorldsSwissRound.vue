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
          <span class="match-label">BO3</span>
          <span v-if="match.status === 'completed'" class="status-badge badge-success">已完成</span>
          <span v-else class="status-badge badge-info">待进行</span>
        </div>

        <div class="match-teams">
          <div class="team team-a" :class="{ winner: match.winnerId === match.teamAId }">
            <span class="team-name">{{ match.teamAName }}</span>
          </div>

          <div class="score-center">
            <template v-if="match.status === 'completed'">
              <span class="team-score" :class="{ 'score-winner': match.winnerId === match.teamAId }">{{ match.scoreA }}</span>
              <span class="score-divider">:</span>
              <span class="team-score" :class="{ 'score-winner': match.winnerId === match.teamBId }">{{ match.scoreB }}</span>
            </template>
            <span v-else class="vs-text">VS</span>
          </div>

          <div class="team team-b" :class="{ winner: match.winnerId === match.teamBId }">
            <span class="team-name">{{ match.teamBName }}</span>
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
  padding: 30px 0;
  text-align: center;
}

.empty-text {
  font-size: 13px;
  color: #94a3b8;
}

.matches-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.match-card {
  padding: 12px 14px;
  background: #ffffff;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #6366f1;
}

.match-card.completed {
  border-left-color: #22c55e;
  background: #fafffe;
}

.match-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.match-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  padding: 1px 8px;
  background: #f1f5f9;
  border-radius: 8px;
}

.status-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 500;
  padding: 1px 8px;
  border-radius: 8px;
  line-height: 1.4;
}

.badge-success {
  background: #f0fdf4;
  color: #16a34a;
}

.badge-info {
  background: #f1f5f9;
  color: #64748b;
}

.match-teams {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.team {
  flex: 1;
}

.team-name {
  font-weight: 500;
  font-size: 13px;
  color: #0f172a;
}

.team.winner .team-name {
  color: #16a34a;
  font-weight: 600;
}

.team.team-b {
  text-align: right;
}

.score-center {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
}

.team-score {
  font-size: 16px;
  font-weight: 700;
  color: #94a3b8;
  min-width: 16px;
  text-align: center;
}

.team-score.score-winner {
  color: #16a34a;
}

.vs-text {
  font-size: 11px;
  font-weight: 600;
  color: #cbd5e1;
}

.score-divider {
  font-size: 14px;
  font-weight: 600;
  color: #cbd5e1;
}

.match-actions {
  display: flex;
  justify-content: center;
  padding-top: 8px;
  border-top: 1px solid #f1f5f9;
}

.completed-actions {
  display: flex;
  align-items: center;
  justify-content: center;
}

.simulate-btn {
  padding: 4px 14px;
  font-size: 12px;
  font-weight: 500;
  color: #ffffff;
  background: #6366f1;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.simulate-btn:hover {
  background: #4f46e5;
}

.detail-btn {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  color: #64748b;
  background: #f1f5f9;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.detail-btn:hover {
  background: #e2e8f0;
}
</style>
