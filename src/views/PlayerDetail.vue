<template>
  <div class="player-detail-view">
    <!-- 返回按钮 -->
    <div class="back-link">
      <el-button text @click="$router.back()">
        <el-icon><ArrowLeft /></el-icon>
        返回上一页
      </el-button>
      <el-button v-if="$route.params.id" text @click="navigateToDataCenter" style="color: #409eff;">
        <el-icon><DataAnalysis /></el-icon>
        数据中心
      </el-button>
    </div>

    <!-- 选手头部信息 -->
    <player-profile-card 
      :player="player" 
      :traits="traits" 
      :condition-info="conditionInfo" 
    />

    <!-- 详细信息区 -->
    <el-row :gutter="20" class="detail-row">
      <!-- 合同信息 -->
      <el-col :span="12">
        <player-contract-card 
          :player="player"
          :contract-history="contractHistory"
          :display-market-value="displayMarketValue"
          @open-market-value="showMarketValueDialog = true"
        />
      </el-col>

      <!-- 职业生涯 -->
      <el-col :span="12">
        <player-career-card
          :player="player"
          :career-years="careerYears"
          :champion-count="championCount"
          :honors-count="honors.length"
        />
      </el-col>
    </el-row>

    <!-- 天赋说明 -->
    <el-alert
      :title="getTalentDescription(player.tag)"
      :type="getTalentAlertType(player.tag)"
      :closable="false"
      show-icon
      class="talent-alert"
    />

    <!-- 特性与状态区 -->
    <el-row :gutter="20" class="traits-condition-row">
      <!-- 选手特性 -->
      <el-col :span="12">
        <player-traits-card
          :traits="traits"
          :all-traits="allTraits"
        />
      </el-col>

      <!-- 选手状态 - 六维雷达图 -->
      <el-col :span="12">
        <player-radar-card
          :player-stats="playerStats"
          :radar-data="computeRadarData"
          :player-name="player.gameId"
        />
      </el-col>
    </el-row>

    <!-- 荣誉记录 -->
    <player-honors-card :honors="honors" />

    <!-- 英雄池 -->
    <div v-if="championMasteries.length > 0" class="champion-pool-card">
      <div class="card-header">
        <span class="card-title">英雄池</span>
        <span class="card-sub">熟练英雄 {{ championMasteries.length }} 个</span>
      </div>
      <div class="mastery-sections">
        <div v-if="ssChampions.length > 0" class="mastery-section">
          <div class="mastery-tier-label tier-ss">SS 级 · 信仰英雄</div>
          <div class="champion-grid">
            <div v-for="c in ssChampions" :key="c.champion_id" class="champion-item">
              <span class="champion-name">{{ c.name_cn }}</span>
              <span class="champion-pos">{{ c.position }}</span>
              <span class="champion-stats">{{ c.games_played }}场 / {{ c.games_won }}胜</span>
            </div>
          </div>
        </div>
        <div v-if="sChampions.length > 0" class="mastery-section">
          <div class="mastery-tier-label tier-s">S 级 · 擅长英雄</div>
          <div class="champion-grid">
            <div v-for="c in sChampions" :key="c.champion_id" class="champion-item">
              <span class="champion-name">{{ c.name_cn }}</span>
              <span class="champion-pos">{{ c.position }}</span>
              <span class="champion-stats">{{ c.games_played }}场 / {{ c.games_won }}胜</span>
            </div>
          </div>
        </div>
        <div v-if="aChampions.length > 0" class="mastery-section">
          <div class="mastery-tier-label tier-a">A 级 · 可用英雄</div>
          <div class="champion-grid">
            <div v-for="c in aChampions" :key="c.champion_id" class="champion-item">
              <span class="champion-name">{{ c.name_cn }}</span>
              <span class="champion-pos">{{ c.position }}</span>
              <span class="champion-stats">{{ c.games_played }}场 / {{ c.games_won }}胜</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 赛季历史 -->
    <player-season-history 
      :season-history="seasonHistory"
      :player-region="player.region"
    />

    <!-- 身价详情弹窗 -->
    <player-market-value-dialog
      v-model:visible="showMarketValueDialog"
      :player="player"
      :market-value-changes="marketValueChanges"
      :honors="honors"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue"
import { useRoute, useRouter } from "vue-router"
import {
  ArrowLeft,
  DataAnalysis
} from "@element-plus/icons-vue"

// Import Sub-components
import PlayerProfileCard from "@/components/player/PlayerProfileCard.vue"
import PlayerContractCard from "@/components/player/PlayerContractCard.vue"
import PlayerCareerCard from "@/components/player/PlayerCareerCard.vue"
import PlayerTraitsCard from "@/components/player/PlayerTraitsCard.vue"
import PlayerRadarCard from "@/components/player/PlayerRadarCard.vue"
import PlayerHonorsCard from "@/components/player/PlayerHonorsCard.vue"
import PlayerSeasonHistory from "@/components/player/PlayerSeasonHistory.vue"
import PlayerMarketValueDialog from "@/components/player/PlayerMarketValueDialog.vue"

// Import Composable
import { usePlayerDetail } from "@/composables/usePlayerDetail"
import { getPlayerChampionMastery, type PlayerMasteryInfo } from "@/api/tauri"
import { useGameStore } from "@/stores/useGameStore"

const route = useRoute()
const router = useRouter()
const playerId = route.params.id as string

const {
  player,
  honors,
  contractHistory,
  traits,
  conditionInfo,
  playerStats,
  marketValueChanges,
  seasonHistory,
  allTraits,
  
  careerYears,
  championCount,
  displayMarketValue,
  computeRadarData,
  
  loadMarketValueChanges,
  initData
} = usePlayerDetail(playerId)

// 身价弹窗状态
const showMarketValueDialog = ref(false)

// 英雄池数据
const gameStore = useGameStore()
const championMasteries = ref<PlayerMasteryInfo[]>([])

const ssChampions = computed(() => championMasteries.value.filter(c => c.mastery_tier === 'SS'))
const sChampions = computed(() => championMasteries.value.filter(c => c.mastery_tier === 'S'))
const aChampions = computed(() => championMasteries.value.filter(c => c.mastery_tier === 'A'))

const loadChampionMasteries = async () => {
  const saveId = gameStore.currentSave?.id
  if (!saveId || !playerId) return
  try {
    const result = await getPlayerChampionMastery(saveId, Number(playerId))
    championMasteries.value = result
  } catch {
    championMasteries.value = []
  }
}

// 监听身价弹窗打开
watch(showMarketValueDialog, (newVal) => {
  if (newVal) {
    loadMarketValueChanges()
  }
})

const navigateToDataCenter = () => {
  router.push(`/data-center/player/${route.params.id}`)
}

// Helper functions for alerts (keeping in main component as they are simple UI helpers)
const getTalentDescription = (tag: string) => {
  const desc: Record<string, string> = {
    GENIUS: "天才选手：每赛季能力值增长 +3，潜力上限更高",
    NORMAL: "普通选手：每赛季能力值增长 +2，稳定发挥",
    ORDINARY: "平庸选手：每赛季能力值增长 +1，成长较慢",
  }
  return desc[tag] || ""
}

const getTalentAlertType = (tag: string) => {
  const types: Record<string, "success" | "warning" | "info" | "error"> = {
    GENIUS: "warning",
    NORMAL: "info",
    ORDINARY: "info",
  }
  return types[tag] || "info"
}

onMounted(() => {
  initData()
  loadChampionMasteries()
})
</script>

<style scoped>
.player-detail-view {
  padding: 0;
}

.back-link {
  margin-bottom: 16px;
}

.back-link .el-button {
  color: var(--text-secondary);
  font-size: 14px;
}

.back-link .el-button:hover {
  color: var(--primary-color);
}

.detail-row {
  margin-bottom: 20px;
}

.talent-alert {
  margin-bottom: 20px;
  border-radius: 8px;
}

.traits-condition-row {
  margin-bottom: 20px;
}

.champion-pool-card {
  margin-bottom: 20px;
  background: white;
  border-radius: 12px;
  border: 1px solid #e5e7eb;
  overflow: hidden;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background: #f7f8fa;
  border-bottom: 1px solid #e5e7eb;
}

.card-title {
  font-size: 14px;
  font-weight: 700;
  color: #1d2129;
}

.card-sub {
  font-size: 12px;
  color: #86909c;
}

.mastery-sections {
  padding: 16px;
}

.mastery-section {
  margin-bottom: 16px;
}

.mastery-section:last-child {
  margin-bottom: 0;
}

.mastery-tier-label {
  font-size: 12px;
  font-weight: 700;
  margin-bottom: 8px;
}

.tier-ss { color: #dc2626; }
.tier-s { color: #d97706; }
.tier-a { color: #059669; }

.champion-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.champion-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: #f7f8fa;
  border-radius: 6px;
  font-size: 12px;
}

.champion-name {
  font-weight: 600;
  color: #1d2129;
}

.champion-pos {
  color: #667eea;
  font-weight: 500;
  font-size: 10px;
}

.champion-stats {
  color: #86909c;
  font-size: 11px;
}
</style>
