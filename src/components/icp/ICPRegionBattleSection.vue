<template>
  <div class="table-section">
    <div class="section-header">
      <span class="section-title">赛区对决阶段</span>
      <span v-if="status === 'completed'" class="status-badge success">已完成</span>
      <span v-else class="status-badge warning">进行中</span>
    </div>

    <!-- 赛区对决对阵 -->
    <div class="region-battle-section">
      <!-- 半决赛（如果需要） -->
      <div v-if="semifinal" class="battle-stage">
        <h4>半决赛</h4>
        <ICPRegionBattleCard
          :battle="semifinal"
          @simulate-match="(battle: ICPRegionMatch, match: ICPMatch) => $emit('simulate-match', battle, match)"
          @view-match="(match: ICPMatch) => $emit('view-match', match)"
        />
        <!-- 半决赛加赛 - 进行中或已完成都显示 -->
        <div v-if="semifinal.status === 'tiebreaker' || semifinal.tiebreakerMatch?.status === 'completed'" class="tiebreaker-section">
          <el-alert
            v-if="semifinal.status === 'tiebreaker'"
            title="比分 2:2 平局！"
            description="需要进行一号种子加赛决出胜者"
            type="warning"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <el-alert
            v-else-if="semifinal.tiebreakerMatch?.status === 'completed'"
            title="加赛已完成"
            type="success"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <div class="tiebreaker-match" v-if="semifinal.tiebreakerMatch">
            <div class="tiebreaker-header">
              <span class="tiebreaker-title">一号种子加赛 (BO5)</span>
            </div>
            <div class="tiebreaker-teams">
              <div class="team-side">
                <span class="team-name">{{ semifinal.tiebreakerMatch.teamAName }}</span>
                <span class="region-label">{{ semifinal.tiebreakerMatch.teamARegion }}</span>
              </div>
              <div class="vs-section">
                <template v-if="semifinal.tiebreakerMatch.status === 'completed'">
                  <span class="score">{{ semifinal.tiebreakerMatch.scoreA }}</span>
                  <span class="vs">:</span>
                  <span class="score">{{ semifinal.tiebreakerMatch.scoreB }}</span>
                </template>
                <span v-else class="vs">VS</span>
              </div>
              <div class="team-side">
                <span class="region-label">{{ semifinal.tiebreakerMatch.teamBRegion }}</span>
                <span class="team-name">{{ semifinal.tiebreakerMatch.teamBName }}</span>
              </div>
            </div>
            <div class="tiebreaker-actions">
              <button
                v-if="semifinal.tiebreakerMatch.status !== 'completed'"
                class="action-btn warning-btn"
                @click="$emit('simulate-tiebreaker', semifinal)"
              >
                模拟加赛
              </button>
              <button
                v-else
                class="action-btn"
                @click="$emit('view-match', semifinal.tiebreakerMatch)"
              >
                查看详情
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 决赛 -->
      <div v-if="final" class="battle-stage final">
        <h4>决赛</h4>
        <ICPRegionBattleCard
          :battle="final"
          @simulate-match="(battle: ICPRegionMatch, match: ICPMatch) => $emit('simulate-match', battle, match)"
          @view-match="(match: ICPMatch) => $emit('view-match', match)"
        />
        <!-- 决赛加赛 - 进行中或已完成都显示 -->
        <div v-if="final.status === 'tiebreaker' || final.tiebreakerMatch?.status === 'completed'" class="tiebreaker-section">
          <el-alert
            v-if="final.status === 'tiebreaker'"
            title="比分 2:2 平局！"
            description="需要进行一号种子加赛决出最强赛区"
            type="warning"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <el-alert
            v-else-if="final.tiebreakerMatch?.status === 'completed'"
            title="加赛已完成"
            type="success"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <div class="tiebreaker-match" v-if="final.tiebreakerMatch">
            <div class="tiebreaker-header">
              <span class="tiebreaker-title">一号种子加赛 (BO5)</span>
            </div>
            <div class="tiebreaker-teams">
              <div class="team-side">
                <span class="team-name">{{ final.tiebreakerMatch.teamAName }}</span>
                <span class="region-label">{{ final.tiebreakerMatch.teamARegion }}</span>
              </div>
              <div class="vs-section">
                <template v-if="final.tiebreakerMatch.status === 'completed'">
                  <span class="score">{{ final.tiebreakerMatch.scoreA }}</span>
                  <span class="vs">:</span>
                  <span class="score">{{ final.tiebreakerMatch.scoreB }}</span>
                </template>
                <span v-else class="vs">VS</span>
              </div>
              <div class="team-side">
                <span class="region-label">{{ final.tiebreakerMatch.teamBRegion }}</span>
                <span class="team-name">{{ final.tiebreakerMatch.teamBName }}</span>
              </div>
            </div>
            <div class="tiebreaker-actions">
              <button
                v-if="final.tiebreakerMatch.status !== 'completed'"
                class="action-btn warning-btn"
                @click="$emit('simulate-tiebreaker', final)"
              >
                模拟加赛
              </button>
              <button
                v-else
                class="action-btn"
                @click="$emit('view-match', final.tiebreakerMatch)"
              >
                查看详情
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ICPRegionMatch, ICPMatch } from '@/types/icp'
import ICPRegionBattleCard from '@/components/icp/ICPRegionBattleCard.vue'

defineProps<{
  semifinal?: ICPRegionMatch
  final?: ICPRegionMatch
  status: string
}>()

defineEmits<{
  (e: 'simulate-match', battle: ICPRegionMatch, match: ICPMatch): void
  (e: 'simulate-tiebreaker', battle: ICPRegionMatch): void
  (e: 'view-match', match: ICPMatch): void
}>()
</script>

<style scoped>
.table-section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 18px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.region-battle-section {
  padding: 16px;
}

.battle-stage {
  margin-bottom: 24px;
  padding: 20px;
  background: #f8fafc;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
}

.battle-stage h4 {
  margin: 0 0 16px 0;
  font-size: 15px;
  font-weight: 600;
  text-align: center;
  color: #0f172a;
}

.battle-stage.final {
  background: #fffbeb;
  border-color: #f59e0b;
}

.tiebreaker-section {
  margin-top: 20px;
  padding: 16px;
  background: #fffbeb;
  border: 1px dashed #f59e0b;
  border-radius: 10px;
}

.tiebreaker-match {
  background: #fff;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e2e8f0;
}

.tiebreaker-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e2e8f0;
}

.tiebreaker-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
}

.tiebreaker-teams {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 24px;
  margin-bottom: 16px;
}

.team-side {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-side .team-name {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.vs-section {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: #f8fafc;
  border-radius: 8px;
}

.vs-section .score {
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.vs-section .vs {
  font-size: 14px;
  font-weight: 600;
  color: #94a3b8;
}

.tiebreaker-actions {
  display: flex;
  justify-content: center;
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

.action-btn {
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  border: none;
}

.warning-btn {
  background: #f59e0b;
  color: #fff;
}

.region-label {
  display: inline-block;
  padding: 1px 8px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 8px;
  background: #f1f5f9;
  color: #64748b;
}

.mb-4 {
  margin-bottom: 16px;
}
</style>