<template>
  <div class="icp-region-battle-card">
    <!-- 赛区对决头部 -->
    <div class="battle-header">
      <div class="region-side region-a">
        <div class="region-flag" :class="battle.regionA.toLowerCase()">
          {{ battle.regionA }}
        </div>
        <div class="region-name">{{ battle.regionAName }}</div>
        <div class="region-wins">{{ battle.regionAWins }} 胜</div>
      </div>

      <div class="battle-vs">
        <div class="vs-text">VS</div>
        <div v-if="battle.status === 'completed'" class="battle-result">
          <span v-if="battle.winnerId === battle.regionA" class="winner-badge">
            {{ battle.regionAName }} 胜出!
          </span>
          <span v-else class="winner-badge">
            {{ battle.regionBName }} 胜出!
          </span>
        </div>
      </div>

      <div class="region-side region-b">
        <div class="region-flag" :class="battle.regionB.toLowerCase()">
          {{ battle.regionB }}
        </div>
        <div class="region-name">{{ battle.regionBName }}</div>
        <div class="region-wins">{{ battle.regionBWins }} 胜</div>
      </div>
    </div>

    <!-- 四场BO5对决列表 -->
    <div class="battles-list">
      <div
        v-for="(match, index) in battle.matches"
        :key="match.id"
        class="battle-match"
        :class="{ completed: match.status === 'completed' }"
      >
        <div class="seed-label">{{ getSeedLabel(index + 1) }}对决</div>

        <div class="match-content">
          <div class="team team-a" :class="{ winner: match.winnerId === match.teamAId }">
            <span class="team-name">{{ match.teamAName }}</span>
            <span class="region-tag" :class="match.teamARegion?.toLowerCase()">
              {{ match.teamARegion }}
            </span>
          </div>

          <div class="match-score">
            <template v-if="match.status === 'completed'">
              <span class="score">{{ match.scoreA }} - {{ match.scoreB }}</span>
              <button class="btn btn-text" @click="$emit('view-match', match)">
                详情
              </button>
            </template>
            <template v-else>
              <button
                class="btn btn-primary"
                @click="$emit('simulate-match', battle, match)"
              >
                模拟
              </button>
            </template>
          </div>

          <div class="team team-b" :class="{ winner: match.winnerId === match.teamBId }">
            <span class="team-name">{{ match.teamBName }}</span>
            <span class="region-tag" :class="match.teamBRegion?.toLowerCase()">
              {{ match.teamBRegion }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 总比分进度条 -->
    <div class="battle-progress">
      <div class="progress-bar">
        <div
          class="progress-a"
          :style="{ width: `${(battle.regionAWins / 4) * 100}%` }"
        ></div>
        <div
          class="progress-b"
          :style="{ width: `${(battle.regionBWins / 4) * 100}%` }"
        ></div>
      </div>
      <div class="progress-labels">
        <span class="label-a">{{ battle.regionA }}: {{ battle.regionAWins }}/4</span>
        <span class="label-b">{{ battle.regionB }}: {{ battle.regionBWins }}/4</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ICPRegionMatch, ICPMatch } from '@/types/icp'

interface Props {
  battle: ICPRegionMatch
}

defineProps<Props>()

defineEmits<{
  (e: 'simulate-match', battle: ICPRegionMatch, match: ICPMatch): void
  (e: 'view-match', match: ICPMatch): void
}>()

const getRegionFlag = (region: string) => {
  const flagMap: Record<string, string> = {
    'LPL': '🇨🇳',
    'LCK': '🇰🇷',
    'LEC': '🇪🇺',
    'LCS': '🇺🇸'
  }
  return flagMap[region] || '🏳️'
}

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

<style scoped>
.icp-region-battle-card {
  background: #ffffff;
  border-radius: 10px;
  padding: 24px;
  border: 1px solid #e2e8f0;
}

.battle-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid #e2e8f0;
}

.region-side {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.region-flag {
  font-size: 20px;
  font-weight: 700;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: #f1f5f9;
  color: #64748b;
  border: 1px solid #e2e8f0;
}

.region-flag.lpl {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}

.region-flag.lck {
  background: #eff6ff;
  color: #2563eb;
  border-color: #bfdbfe;
}

.region-flag.lec {
  background: #f0fdf4;
  color: #16a34a;
  border-color: #bbf7d0;
}

.region-flag.lcs {
  background: #fefce8;
  color: #ca8a04;
  border-color: #fde68a;
}

.region-name {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
}

.region-wins {
  font-size: 20px;
  font-weight: 700;
  padding: 4px 16px;
  border-radius: 20px;
}

.region-a .region-wins {
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
}

.region-b .region-wins {
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}

.battle-vs {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 0 24px;
}

.vs-text {
  font-size: 24px;
  font-weight: 900;
  color: #94a3b8;
}

.winner-badge {
  display: inline-block;
  padding: 6px 16px;
  background: #fefce8;
  color: #ca8a04;
  border: 1px solid #fde68a;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 700;
}

.battles-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.battle-match {
  padding: 16px;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.battle-match.completed {
  background: #f0fdf4;
  border-color: #bbf7d0;
}

.seed-label {
  text-align: center;
  font-size: 12px;
  font-weight: 600;
  color: #6366f1;
  margin-bottom: 12px;
  padding: 4px 12px;
  background: #eef2ff;
  border: 1px solid #e0e7ff;
  border-radius: 4px;
  display: inline-block;
  margin-left: 50%;
  transform: translateX(-50%);
}

.match-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.match-content .team {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.match-content .team .team-name {
  font-weight: 600;
  font-size: 14px;
  color: #0f172a;
}

.match-content .team.winner .team-name {
  color: #22c55e;
  font-weight: 700;
}

.match-content .team.team-b {
  justify-content: flex-end;
}

.match-score {
  padding: 0 20px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.match-score .score {
  font-size: 18px;
  font-weight: 700;
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

.battle-progress .progress-bar {
  height: 12px;
  background: #e2e8f0;
  border-radius: 6px;
  display: flex;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress-a {
  height: 100%;
  background: #dc2626;
  transition: width 0.3s ease;
}

.progress-b {
  height: 100%;
  background: #2563eb;
  transition: width 0.3s ease;
  margin-left: auto;
}

.progress-labels {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
}

.label-a {
  color: #dc2626;
}

.label-b {
  color: #2563eb;
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
