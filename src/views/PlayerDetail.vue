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
    <player-champion-pool-card :player-id="Number(playerId)" />

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
import { ref, onMounted, watch } from "vue"
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
import PlayerChampionPoolCard from "@/components/player/PlayerChampionPoolCard.vue"

// Import Composable
import { usePlayerDetail } from "@/composables/usePlayerDetail"

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
  font-size: 14px;
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
</style>
