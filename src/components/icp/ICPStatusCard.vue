<template>
  <div class="icp-status-card">
    <div class="status-header">
      <div class="status-info">
        <h2>S{{ season }} ICP 四赛区洲际对抗赛</h2>
        <span class="status-badge" :class="statusType">
          {{ statusText }}
        </span>
      </div>
    </div>

    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">16</span>
        <span class="stat-label">参赛队伍总数</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">4</span>
        <span class="stat-label">参赛赛区</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">4</span>
        <span class="stat-label">种子组数量</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">4</span>
        <span class="stat-label">每赛区队伍</span>
      </div>
    </div>

    <!-- 参赛队伍种子分组展示 -->
    <div class="seed-teams-display" v-if="status !== 'not_started'">
      <h3 class="section-title">参赛队伍种子分组</h3>
      <div class="seed-groups-grid">
        <div v-for="seed in 4" :key="seed" :class="['seed-group-card', `seed-${seed}`]">
          <div class="seed-header">
            <span class="seed-title">{{ seedLabels[seed] }}</span>
            <span :class="['status-badge', seedBadgeTypes[seed]]">{{ seedDescriptions[seed] }}</span>
          </div>
          <div class="seed-team-list">
            <template v-if="seedTeamsGrouped[seed]?.length > 0">
              <div
                v-for="team in seedTeamsGrouped[seed]"
                :key="team.teamId"
                class="seed-team-item"
              >
                <span class="team-name">{{ team.teamName }}</span>
                <span class="region-label">{{ team.region }}</span>
              </div>
            </template>
            <div v-else class="seed-team-item pending">
              <span class="team-name">待夏季赛结束后确定</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 赛区徽章统计 -->
    <div class="region-badges-section" v-if="status !== 'not_started'">
      <h3>赛区徽章统计</h3>
      <div class="region-badges-grid">
        <div
          v-for="region in sortedRegionStats"
          :key="region.region"
          class="region-badge-card"
          :class="{ champion: region.ranking === 1 }"
        >
          <div class="region-flag" :class="region.region.toLowerCase()">
            {{ getRegionFlag(region.region) }}
          </div>
          <div class="region-name">{{ region.regionName }}</div>
          <div class="badge-count">
            <span class="badge-number">{{ region.totalBadges }}</span>
          </div>
          <div v-if="region.ranking" class="region-rank">
            第{{ region.ranking }}名
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ICPRegionStats } from '@/types/icp'

const props = defineProps<{
  season: number
  status: string
  seedTeamsGrouped: Record<number, Array<{ teamId: string, teamName: string, region: string }>>
  sortedRegionStats: ICPRegionStats[]
}>()

const statusType = computed(() => {
  const typeMap: Record<string, any> = {
    'not_started': 'info',
    'group_stage': 'warning',
    'region_battle': 'warning',
    'completed': 'success'
  }
  return typeMap[props.status] || 'info'
})

const statusText = computed(() => {
  const textMap: Record<string, string> = {
    'not_started': '未开始',
    'group_stage': '种子组赛进行中',
    'region_battle': '赛区对决进行中',
    'completed': '已完成'
  }
  return textMap[props.status] || props.status
})

const seedLabels: Record<number, string> = {
  1: '一号种子',
  2: '二号种子',
  3: '三号种子',
  4: '四号种子'
}

const seedDescriptions: Record<number, string> = {
  1: '各赛区冠军',
  2: '各赛区亚军',
  3: '各赛区季军',
  4: '各赛区殿军'
}

const seedBadgeTypes: Record<number, string> = {
  1: 'warning',
  2: 'warning',
  3: 'success',
  4: 'info'
}

const getRegionFlag = (region: string) => {
  const flagMap: Record<string, string> = {
    'LPL': '🇨🇳',
    'LCK': '🇰🇷',
    'LEC': '🇪🇺',
    'LCS': '🇺🇸'
  }
  return flagMap[region] || '🏳️'
}
</script>

<style scoped>
.icp-status-card {
  background: #fff;
  border-radius: 10px;
  padding: 24px;
  border: 1px solid #e2e8f0;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e2e8f0;
}

.status-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-info h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.stats-bar {
  display: flex;
  gap: 0;
  margin-bottom: 20px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
}

.stat-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 14px;
  border-right: 1px solid #e2e8f0;
}

.stat-item:last-child {
  border-right: none;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.stat-label {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 2px;
}

.status-badge {
  display: inline-block;
  padding: 2px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 10px;
}

.status-badge.success { background: #f0fdf4; color: #16a34a; }
.status-badge.warning { background: #fffbeb; color: #d97706; }
.status-badge.info { background: #f1f5f9; color: #64748b; }

.region-label {
  display: inline-block;
  padding: 1px 8px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 8px;
  background: #f1f5f9;
  color: #64748b;
}

.seed-teams-display { margin-bottom: 24px; }
.seed-teams-display .section-title {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.seed-groups-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.seed-group-card {
  padding: 16px;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  background: #fff;
}

.seed-group-card.seed-1 { border-color: #f59e0b; }
.seed-group-card.seed-1 .seed-title { color: #b45309; }
.seed-group-card.seed-2 { border-color: #94a3b8; }
.seed-group-card.seed-2 .seed-title { color: #475569; }
.seed-group-card.seed-3 { border-color: #a78bfa; }
.seed-group-card.seed-3 .seed-title { color: #6d28d9; }
.seed-group-card.seed-4 { border-color: #64748b; }
.seed-group-card.seed-4 .seed-title { color: #374151; }

.seed-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e2e8f0;
}

.seed-title {
  font-size: 14px;
  font-weight: 600;
  flex: 1;
}

.seed-team-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.seed-team-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f8fafc;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
}

.seed-team-item .team-name {
  font-weight: 600;
  color: #0f172a;
  font-size: 13px;
}

.seed-team-item.pending .team-name {
  color: #94a3b8;
  font-style: italic;
  font-weight: normal;
}

.region-badges-section { margin-bottom: 24px; }
.region-badges-section h3 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.region-badges-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.region-badge-card {
  padding: 20px;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  text-align: center;
}

.region-badge-card.champion {
  border-color: #f59e0b;
  background: #fffbeb;
}

.region-flag {
  font-size: 28px;
  margin-bottom: 8px;
}

.region-name {
  font-size: 13px;
  font-weight: 600;
  color: #0f172a;
  margin-bottom: 12px;
}

.badge-count {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.badge-number {
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.region-rank {
  margin-top: 8px;
  font-size: 12px;
  color: #94a3b8;
}
</style>