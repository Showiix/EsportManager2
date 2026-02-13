<template>
  <div class="table-section">
    <div class="section-header">
      <span class="section-title">种子组赛阶段</span>
      <span v-if="isGroupStageComplete" class="status-badge success">已完成</span>
      <span v-else class="status-badge warning">进行中</span>
    </div>

    <!-- 种子组积分榜 -->
    <div class="seed-groups">
      <el-tabs v-model="activeTab" type="card">
        <el-tab-pane
          v-for="group in seedGroups"
          :key="group.groupName"
          :label="`${getSeedGroupLabel(group.groupName)}组`"
          :name="group.groupName"
        >
          <ICPSeedGroupStanding
            :group="group"
            @simulate-match="$emit('simulate-match', $event)"
            @view-match="$emit('view-match', $event)"
          />
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 生成赛区对决按钮 -->
    <div v-if="isGroupStageComplete && tournamentStatus === 'group_stage'" class="generate-region-battle-section">
      <el-alert
        title="种子组赛已完成！"
        description="所有种子组比赛已完成，各组前2名获得徽章。现在可以进入赛区对决阶段。"
        type="success"
        :closable="false"
        show-icon
        class="mb-4"
      />
      <button
        class="action-btn primary-btn"
        @click="$emit('generate-region-battle')"
        :disabled="generatingRegionBattle"
      >
        进入赛区对决
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ICPSeedGroup, ICPMatch } from '@/types/icp'
import ICPSeedGroupStanding from '@/components/icp/ICPSeedGroupStanding.vue'

const props = defineProps<{
  seedGroups: ICPSeedGroup[]
  tournamentStatus: string
  generatingRegionBattle: boolean
  activeGroup?: string
}>()

const emit = defineEmits<{
  (e: 'update:activeGroup', value: string): void
  (e: 'simulate-match', match: ICPMatch): void
  (e: 'view-match', match: ICPMatch): void
  (e: 'generate-region-battle'): void
}>()

const activeTab = computed({
  get: () => props.activeGroup || 'A',
  set: (val) => emit('update:activeGroup', val)
})

const isGroupStageComplete = computed(() => {
  return props.seedGroups.every(group => {
    return group.matches.every(match => match.status === 'completed')
  })
})

const getSeedGroupLabel = (groupName: string) => {
  const labelMap: Record<string, string> = {
    'A': '一号种子',
    'B': '二号种子',
    'C': '三号种子',
    'D': '四号种子'
  }
  return labelMap[groupName] || groupName
}
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

.seed-groups {
  padding: 16px;
}

.generate-region-battle-section {
  margin-top: 24px;
  padding: 16px;
  text-align: center;
}

.generate-region-battle-section .action-btn {
  margin-top: 16px;
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

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.primary-btn {
  background: #6366f1;
  color: #fff;
}

.mb-4 {
  margin-bottom: 16px;
}
</style>