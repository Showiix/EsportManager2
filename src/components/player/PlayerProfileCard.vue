<template>
  <el-card class="profile-card">
    <div class="profile-content">
      <div class="avatar-section">
        <div class="player-avatar" :class="player.region.toLowerCase()">
          {{ player.position }}
        </div>
        <el-tag :type="getTalentType(player.tag)" size="large" effect="dark" class="talent-tag">
          {{ getTalentLabel(player.tag) }}
        </el-tag>
      </div>

      <div class="info-section">
        <div class="player-header">
          <h1 class="player-name">{{ player.gameId }}</h1>
          <div class="player-tags">
            <el-tag :type="getPositionType(player.position)" size="default">
              {{ getPositionName(player.position) }}
            </el-tag>
            <el-tag :type="getRegionType(player.region)" size="default">
              {{ player.region }}
            </el-tag>
            <el-tag type="success" size="default">在役</el-tag>
          </div>
        </div>
        <p class="player-real-name">{{ player.realName }} · {{ player.nationality }}</p>
        <div class="player-team">
          <div class="team-avatar mini" :class="player.region.toLowerCase()">
            {{ player.team.substring(0, 2) }}
          </div>
          <span>{{ player.team }}</span>
        </div>
      </div>

      <div class="stats-section">
        <div class="stat-number-display">
          <span class="stat-value" :style="{ color: getAbilityColor(player.ability) }">{{ player.ability }}</span>
          <span class="stat-label">能力</span>
        </div>
        <div class="stat-number-display">
          <span class="stat-value" style="color: #8b5cf6;">{{ player.potential }}</span>
          <span class="stat-label">潜力</span>
        </div>
        <div class="stat-number-display">
          <span class="stat-value" style="color: #22c55e;">{{ player.stability }}</span>
          <span class="stat-label">稳定</span>
        </div>
        <div class="stat-number-display">
          <span class="stat-value" :style="{ color: getLoyaltyColor(player.loyalty) }">{{ player.loyalty }}</span>
          <span class="stat-label">忠诚</span>
        </div>
        <div class="stat-number-display">
          <span class="stat-value" :style="{ color: getSatisfactionColor(player.satisfaction) }">{{ player.satisfaction }}</span>
          <span class="stat-label">满意</span>
        </div>
        <div class="stat-text">
          <div class="age-display">
            <span class="age-value">{{ player.age }}</span>
            <span class="age-label">岁</span>
          </div>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { PropType } from 'vue'
import { TraitInfo, PlayerConditionInfo } from '@/api/tauri'
import { PlayerDetailInfo } from '@/composables/usePlayerDetail'

defineProps({
  player: {
    type: Object as PropType<PlayerDetailInfo>,
    required: true
  },
  traits: {
    type: Array as PropType<TraitInfo[]>,
    default: () => []
  },
  conditionInfo: {
    type: Object as PropType<PlayerConditionInfo | null>,
    default: null
  }
})

// Helper functions extracted from original component
const getTalentType = (tag: string) => {
  const types: Record<string, string> = {
    GENIUS: 'warning',
    NORMAL: 'primary',
    ORDINARY: 'info',
  }
  return types[tag] || 'info'
}

const getTalentLabel = (tag: string) => {
  const labels: Record<string, string> = {
    GENIUS: '天才',
    NORMAL: '普通',
    ORDINARY: '平庸',
  }
  return labels[tag] || tag
}

const getPositionType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'success',
    MID: 'primary',
    ADC: 'warning',
    SUP: 'info',
  }
  return types[position] || 'info'
}

const getPositionName = (position: string) => {
  const names: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: '下路',
    SUP: '辅助',
  }
  return names[position] || position
}

const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getLoyaltyColor = (loyalty: number) => {
  if (loyalty >= 70) return '#22c55e'
  if (loyalty >= 50) return '#3b82f6'
  if (loyalty >= 35) return '#f59e0b'
  return '#ef4444'
}

const getSatisfactionColor = (satisfaction: number) => {
  if (satisfaction >= 65) return '#22c55e'
  if (satisfaction >= 50) return '#3b82f6'
  if (satisfaction >= 40) return '#f59e0b'
  return '#ef4444'
}
</script>

<style scoped>
.profile-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.profile-content {
  display: flex;
  align-items: flex-start;
  gap: 32px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.player-avatar {
  width: 120px;
  height: 120px;
  border-radius: 16px;
  font-size: 24px;
}

.talent-tag {
  font-size: 14px;
}

.info-section {
  flex: 1;
}

.player-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 8px;
}

.player-name {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.player-tags {
  display: flex;
  gap: 8px;
}

.player-real-name {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

.player-team {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  color: var(--text-primary);
}

.stats-section {
  display: flex;
  align-items: center;
  gap: 24px;
}

.stat-number-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stat-number-display .stat-value {
  font-size: 32px;
  font-weight: 700;
  line-height: 1;
}

.stat-number-display .stat-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.stat-text {
  text-align: center;
}

.age-display {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.age-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.age-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-top: 4px;
}
</style>
