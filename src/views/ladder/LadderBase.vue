<template>
  <div class="ladder-view">
    <div class="page-header">
      <div>
        <h1>{{ eventName }}</h1>
        <p>
          <template v-if="tournamentInfo">
            第 {{ tournamentInfo.edition }} 届 · 轮次 {{ tournamentInfo.current_round }} / {{ tournamentInfo.total_rounds }}
          </template>
          <template v-else>加载中...</template>
        </p>
      </div>
      <div class="header-actions">
        <SeasonSelector v-model="selectedSeason" />
        <el-tag v-if="tournamentInfo" :type="getStatusType(tournamentInfo.status)" size="large">
          {{ getStatusText(tournamentInfo.status) }}
        </el-tag>
        <el-button type="primary" @click="handleSimulateRound" :loading="simulating" :disabled="!canSimulate">
          模拟下一轮
        </el-button>
        <el-button
          v-if="canComplete"
          type="success"
          @click="handleCompleteTournament"
          :loading="completing"
        >
          完成天梯赛并结算奖励
        </el-button>
        <button class="detail-btn" @click="loadData">刷新数据</button>
      </div>
    </div>

    <div class="stats-bar" v-if="tournamentInfo">
      <div class="stat-item">
        <span class="stat-value">{{ rankings.length }}</span>
        <span class="stat-label">参赛选手</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value highlight">{{ tournamentInfo.current_round }}</span>
        <span class="stat-label">当前轮次</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ matches.length }}</span>
        <span class="stat-label">总场次</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ tournamentInfo.total_rounds }}</span>
        <span class="stat-label">总轮次</span>
      </div>
    </div>

    <el-tabs v-model="activeTab" class="ladder-tabs">
      <el-tab-pane label="天梯榜单" name="rankings">
        <LadderRankings :rankings="rankings" :loading="loading" :tournament-id="tournamentInfo?.id ?? 0" />
      </el-tab-pane>
      <el-tab-pane label="赛程赛果" name="schedule">
        <LadderSchedule :matches="matches" :total-rounds="tournamentInfo?.total_rounds || 12" :loading="loading" />
      </el-tab-pane>
      <el-tab-pane label="数据统计" name="stats">
        <LadderStats :rankings="rankings" :loading="loading" />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import {
  simulateLadderRound,
  getLadderRankings,
  getLadderMatches,
  completeLadderTournament,
  getLadderTournaments,
  type LadderTournamentInfo,
  type LadderRankingEntry,
  type LadderMatchInfo
} from '@/api/ladder'
import LadderRankings from '@/components/ladder/LadderRankings.vue'
import LadderSchedule from '@/components/ladder/LadderSchedule.vue'
import LadderStats from '@/components/ladder/LadderStats.vue'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const props = defineProps<{
  eventType: string
  eventName: string
}>()

const gameStore = useGameStore()
const timeStore = useTimeStore()

const activeTab = ref('rankings')
const loading = ref(false)
const simulating = ref(false)
const completing = ref(false)
const selectedSeason = ref(timeStore.currentSeasonFromTime || timeStore.currentSeason || gameStore.currentSeason || 1)
const tournamentInfo = ref<LadderTournamentInfo | null>(null)
const rankings = ref<LadderRankingEntry[]>([])
const matches = ref<LadderMatchInfo[]>([])

const canSimulate = computed(() => {
  if (!tournamentInfo.value) return false
  return tournamentInfo.value.current_round < tournamentInfo.value.total_rounds && tournamentInfo.value.status !== 'completed'
})

const canComplete = computed(() => {
  if (!tournamentInfo.value) return false
  return tournamentInfo.value.current_round >= tournamentInfo.value.total_rounds && tournamentInfo.value.status !== 'completed'
})

const getStatusType = (status: string) => {
  switch (status) {
    case 'pending': return 'info'
    case 'ongoing': return 'warning'
    case 'completed': return 'success'
    default: return 'info'
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'pending': return '未开始'
    case 'ongoing': return '进行中'
    case 'completed': return '已完成'
    default: return status
  }
}

const fetchTournament = async () => {
  const season = selectedSeason.value
  try {
    loading.value = true
    const ladders = await getLadderTournaments(season)
    const found = ladders.find(l => l.event_type === props.eventType)
    if (found) {
      tournamentInfo.value = found
      await loadData()
    } else {
      ElMessage.warning('当前赛季暂无此天梯赛，请先通过时间推进创建')
    }
  } catch (error) {
    console.error('加载天梯赛失败:', error)
    ElMessage.error('加载天梯赛失败')
  } finally {
    loading.value = false
  }
}

const handleSimulateRound = async () => {
  if (!tournamentInfo.value) return

  try {
    simulating.value = true
    const message = await simulateLadderRound(tournamentInfo.value.id)
    ElMessage.success(message)
    await fetchTournament()
  } catch (error: any) {
    console.error('模拟轮次失败:', error)
    ElMessage.error(error.message || '模拟失败')
  } finally {
    simulating.value = false
  }
}

const loadRankings = async () => {
  if (!tournamentInfo.value) return
  try {
    rankings.value = await getLadderRankings(tournamentInfo.value.id)
  } catch (error) {
    console.error('加载榜单失败:', error)
  }
}

const loadMatches = async () => {
  if (!tournamentInfo.value) return
  try {
    matches.value = await getLadderMatches(tournamentInfo.value.id)
  } catch (error) {
    console.error('加载赛程失败:', error)
  }
}

const handleCompleteTournament = async () => {
  if (!tournamentInfo.value) return

  try {
    await ElMessageBox.confirm(
      '确认完成天梯赛并结算奖励？此操作不可撤销。',
      '确认完成',
      { confirmButtonText: '确认', cancelButtonText: '取消', type: 'warning' }
    )

    completing.value = true
    const result = await completeLadderTournament(tournamentInfo.value.id)

    let message = `天梯赛已完成！共 ${result.total_players} 名选手参赛，${result.rewards_distributed.length} 名选手获得奖励。`
    
    if (result.rewards_distributed.length > 0) {
      const top3 = result.rewards_distributed.slice(0, 3)
      message += '\n\n前三名：\n'
      top3.forEach(reward => {
        message += `${reward.rank}. ${reward.player_name} (Rating: ${reward.rating})`
        if (reward.ability_gain > 0) message += ` +${reward.ability_gain} 能力`
        if (reward.trait_unlocked) message += ` 解锁特性: ${reward.trait_unlocked}`
        message += '\n'
      })
    }

    ElMessage.success({ message, duration: 5000, showClose: true })
    await fetchTournament()
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('完成天梯赛失败:', error)
      ElMessage.error(error.message || '完成失败')
    }
  } finally {
    completing.value = false
  }
}

const loadData = async () => {
  loading.value = true
  try {
    await Promise.all([loadRankings(), loadMatches()])
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await fetchTournament()
})

watch(selectedSeason, () => {
  tournamentInfo.value = null
  rankings.value = []
  matches.value = []
  fetchTournament()
})
</script>

<style scoped>
.ladder-view { padding: 0; }

.page-header {
  display: flex; justify-content: space-between; align-items: flex-start;
  margin-bottom: 20px;
}
.page-header h1 {
  font-size: 24px; font-weight: 700; color: #0f172a;
  margin: 0 0 4px 0; letter-spacing: -0.3px;
}
.page-header p { font-size: 13px; color: #94a3b8; margin: 0; }
.header-actions { display: flex; align-items: center; gap: 10px; }

.stats-bar {
  display: flex; align-items: center;
  padding: 14px 24px; background: #ffffff;
  border: 1px solid #e2e8f0; border-radius: 10px;
  margin-bottom: 16px;
}
.stat-item {
  display: flex; align-items: baseline; gap: 6px;
  flex: 1; justify-content: center;
}
.stat-value {
  font-size: 20px; font-weight: 700; color: #0f172a;
  font-variant-numeric: tabular-nums;
}
.stat-value.highlight { color: #6366f1; }
.stat-label { font-size: 12px; color: #94a3b8; font-weight: 500; }
.stat-divider { width: 1px; height: 24px; background: #e2e8f0; flex-shrink: 0; }

.detail-btn {
  padding: 5px 14px; border: 1px solid #e2e8f0; border-radius: 6px;
  background: #ffffff; color: #475569; font-size: 12px;
  font-weight: 500; cursor: pointer; transition: all 0.15s;
}
.detail-btn:hover { border-color: #6366f1; color: #6366f1; background: #f5f3ff; }

.ladder-tabs :deep(.el-tabs__header) { margin-bottom: 0; }
.ladder-tabs :deep(.el-tabs__content) { padding: 0; }
</style>
