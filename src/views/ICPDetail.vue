<template>
  <div class="icp-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div>
        <button class="back-btn" @click="goBack">&larr; 返回赛事列表</button>
        <h1 class="page-title">ICP 四赛区洲际对抗赛 (Intercontinental Championship)</h1>
        <p class="page-desc">16支队伍（各赛区夏季赛前4名），按种子分组BO3单循环，决出最强赛区</p>
      </div>
      <div class="header-actions">
        <button
          v-if="icpTournament.status === 'group_stage' && !isGroupStageComplete"
          class="action-btn primary-btn"
          @click="batchSimulateGroupStage"
          :disabled="simulatingGroupStage"
        >
          {{ simulatingGroupStage ? `模拟中 (${groupSimProgress}%)` : '模拟种子组赛' }}
        </button>
        <button
          v-if="icpTournament.status === 'region_battle' || icpTournament.status === 'tiebreaker'"
          class="action-btn warning-btn"
          @click="batchSimulateRegionBattle"
          :disabled="simulatingRegionBattle"
        >
          {{ simulatingRegionBattle ? `模拟中 (${battleSimProgress}%)` : '模拟赛区对决' }}
        </button>
      </div>
    </div>

    <!-- 阶段未到提示 -->
    <el-alert
      v-if="phaseNotReached"
      title="赛事尚未开始"
      type="warning"
      :closable="false"
      show-icon
      class="phase-warning-alert"
    >
      <template #default>
        <div class="phase-warning-content">
          <p>当前赛季阶段：<strong>{{ currentPhaseDisplay }}</strong></p>
          <p>ICP洲际对抗赛需要在 <strong>世界冠军赛</strong> 结束后才会开始。</p>
          <p>请先完成之前的赛事阶段，然后在时间控制面板推进到ICP洲际对抗赛阶段。</p>
        </div>
      </template>
    </el-alert>

    <!-- ICP赛事状态卡片 -->
    <ICPStatusCard
      :season="viewingSeason"
      :status="icpTournament.status"
      :seed-teams-grouped="seedTeamsGrouped"
      :sorted-region-stats="sortedRegionStats"
    />

    <!-- 种子组赛阶段 -->
    <ICPSeedGroupTabs
      v-if="icpTournament.status !== 'not_started'"
      :seed-groups="icpTournament.seedGroups"
      :tournament-status="icpTournament.status"
      :generating-region-battle="generatingRegionBattle"
      v-model:active-group="activeSeedGroup"
      @simulate-match="handleSimulateMatch"
      @view-match="viewMatchDetails"
      @generate-region-battle="handleGenerateRegionBattle"
    />

    <!-- 赛区对决阶段 -->
    <ICPRegionBattleSection
      v-if="icpTournament.status === 'region_battle' || icpTournament.status === 'completed'"
      :status="icpTournament.status"
      :semifinal="icpTournament.semifinal"
      :final="icpTournament.final"
      @simulate-match="handleSimulateRegionMatch"
      @simulate-tiebreaker="handleSimulateTiebreaker"
      @view-match="viewMatchDetails"
    />

    <!-- 最终排名 -->
    <ICPFinalStandings
      :status="icpTournament.status"
      :standings="icpStandings"
      :champion-name="icpTournament.champion?.regionName || ''"
    />

    <!-- 比赛详情弹窗 -->
    <MatchDetailDialog
      v-if="currentMatchDetail"
      :visible="showMatchDetailDialog"
      :match-detail="currentMatchDetail"
      @close="handleCloseMatchDetail"
    />
  </div>
</template>

<script setup lang="ts">
import ICPStatusCard from '@/components/icp/ICPStatusCard.vue'
import ICPSeedGroupTabs from '@/components/icp/ICPSeedGroupTabs.vue'
import ICPRegionBattleSection from '@/components/icp/ICPRegionBattleSection.vue'
import ICPFinalStandings from '@/components/icp/ICPFinalStandings.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useICPTournament } from '@/composables/useICPTournament'

const {
  icpTournament,
  viewingSeason,
  phaseNotReached,
  currentPhaseDisplay,
  simulatingGroupStage,
  groupSimProgress,
  simulatingRegionBattle,
  battleSimProgress,
  isGroupStageComplete,
  seedTeamsGrouped,
  sortedRegionStats,
  activeSeedGroup,
  generatingRegionBattle,
  icpStandings,
  currentMatchDetail,
  showMatchDetailDialog,
  
  goBack,
  batchSimulateGroupStage,
  batchSimulateRegionBattle,
  handleSimulateMatch,
  viewMatchDetails,
  handleGenerateRegionBattle,
  handleSimulateRegionMatch,
  handleSimulateTiebreaker,
  handleCloseMatchDetail
} = useICPTournament()
</script>

<style scoped>
.icp-management {
  padding: 24px;
  background: #f8fafc;
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.back-btn {
  background: none;
  border: none;
  color: #6366f1;
  font-size: 13px;
  cursor: pointer;
  padding: 0;
  margin-bottom: 8px;
}

.page-title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.page-desc {
  margin: 4px 0 0 0;
  color: #64748b;
  font-size: 13px;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.action-btn {
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  border: none;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.primary-btn {
  background: #6366f1;
  color: #fff;
}

.warning-btn {
  background: #f59e0b;
  color: #fff;
}

.phase-warning-alert {
  margin-bottom: 24px;
}

.phase-warning-content p {
  margin: 4px 0;
  line-height: 1.6;
}

.phase-warning-content p strong {
  color: var(--el-color-warning);
}
</style>