<template>
  <el-card class="player-champion-pool-card">
    <template #header>
      <div class="card-header">
        <span>英雄池</span>
        <el-tag type="info" size="small">{{ champions.length }} 个英雄</el-tag>
      </div>
    </template>

    <div v-if="loading" class="loading-state">
      <el-skeleton :rows="3" animated />
    </div>

    <div v-else-if="champions.length === 0" class="empty-state">
      <el-empty description="暂无英雄数据" :image-size="60" />
    </div>

    <div v-else class="champion-list">
      <div v-for="tier in ['SS', 'S', 'A']" :key="tier" class="tier-group">
        <div v-if="hasTier(tier)" class="tier-row">
          <div class="tier-label">
            <el-tag :type="getTierTagType(tier)" effect="dark" size="small">{{ tier }}</el-tag>
          </div>
          <div class="tier-champions">
            <div 
              v-for="champ in getChampionsByTier(tier)" 
              :key="champ.champion_id"
              class="champion-item"
            >
              <div class="champion-info">
                <span class="champion-name">{{ champ.name_cn }}</span>
                <span class="champion-stats">{{ champ.games_won }} / {{ champ.games_played }}</span>
              </div>
              <div class="win-rate-bar">
                <div 
                  class="win-rate-fill" 
                  :style="{ width: `${getWinRate(champ)}%`, backgroundColor: getWinRateColor(getWinRate(champ)) }"
                ></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { getPlayerChampionMastery, saveApi } from '@/api/tauri'
import type { PlayerMasteryInfo } from '@/api/tauri'

const props = defineProps<{
  playerId: number | string
}>()

const loading = ref(true)
const champions = ref<PlayerMasteryInfo[]>([])

const loadData = async () => {
  if (!props.playerId) return
  
  loading.value = true
  try {
    const saveId = await saveApi.getCurrentSaveId()
    if (saveId) {
      const pid = typeof props.playerId === 'string' ? parseInt(props.playerId) : props.playerId
      if (!isNaN(pid)) {
        champions.value = await getPlayerChampionMastery(saveId, pid)
      }
    }
  } catch (error) {
    console.error('Failed to load champion pool:', error)
  } finally {
    loading.value = false
  }
}

const hasTier = (tier: string) => {
  return champions.value.some(c => c.mastery_tier === tier)
}

const getChampionsByTier = (tier: string) => {
  return champions.value.filter(c => c.mastery_tier === tier)
}

const getTierTagType = (tier: string) => {
  switch (tier) {
    case 'SS': return 'danger'
    case 'S': return 'warning'
    case 'A': return 'primary'
    default: return 'info'
  }
}

const getWinRate = (champ: PlayerMasteryInfo) => {
  if (champ.games_played === 0) return 0
  return Math.round((champ.games_won / champ.games_played) * 100)
}

const getWinRateColor = (rate: number) => {
  if (rate >= 60) return '#67c23a' // success
  if (rate >= 50) return '#409eff' // primary
  if (rate >= 40) return '#e6a23c' // warning
  return '#f56c6c' // danger
}

watch(() => props.playerId, () => {
  loadData()
})

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.player-champion-pool-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.loading-state, .empty-state {
  padding: 40px 0;
  display: flex;
  justify-content: center;
}

.champion-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 8px 0;
}

.tier-group {
  display: flex;
  flex-direction: column;
}

.tier-row {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.tier-label {
  width: 40px;
  flex-shrink: 0;
  padding-top: 4px;
}

.tier-champions {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 10px;
}

.champion-item {
  background-color: #f5f7fa;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border: 1px solid #ebeef5;
  transition: all 0.2s;
}

.champion-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.05);
  border-color: #dcdfe6;
}

.champion-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.champion-name {
  font-weight: 700;
  color: #303133;
}

.champion-stats {
  color: #909399;
  font-size: 11px;
  font-family: monospace;
}

.win-rate-container {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.win-rate-text {
  font-size: 10px;
  color: #909399;
  text-align: right;
  margin-bottom: -2px;
}

.win-rate-bar {
  height: 4px;
  background-color: #e4e7ed;
  border-radius: 2px;
  overflow: hidden;
  width: 100%;
}

.win-rate-fill {
  height: 100%;
}
</style>
