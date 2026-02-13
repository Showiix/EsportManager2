<template>
  <el-card class="detail-card contract-card">
    <template #header>
      <div class="card-header">
        <h2>
          <el-icon><Document /></el-icon>
          合同信息
        </h2>
        <el-button
          v-if="contractHistory.length > 0"
          text
          size="small"
          @click="showContractHistory = !showContractHistory"
        >
          {{ showContractHistory ? '收起历史' : '合同历史' }}
          <el-icon class="collapse-arrow" :class="{ expanded: showContractHistory }"><ArrowDown /></el-icon>
        </el-button>
      </div>
    </template>
    <div class="info-list">
      <div class="info-row">
        <span class="info-label">所属战队</span>
        <span class="info-value">{{ player.team }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">所属赛区</span>
        <el-tag :type="getRegionType(player.region)" size="small">
          {{ player.region }}
        </el-tag>
      </div>
      <div class="info-row">
        <span class="info-label">合同到期</span>
        <span class="info-value highlight">{{ player.contractEnd }} 赛季</span>
      </div>
      <div class="info-row">
        <span class="info-label">年薪</span>
        <span class="info-value money">{{ formatMoney(player.salary) }}</span>
      </div>
      <div class="info-row clickable" @click="$emit('open-market-value')">
        <span class="info-label">身价</span>
        <span class="info-value success">
          {{ formatMoney(displayMarketValue) }}
          <el-icon class="click-icon"><ArrowRight /></el-icon>
        </span>
      </div>
    </div>

    <div v-if="showContractHistory" class="contract-history">
      <el-divider content-position="left">合同历史</el-divider>
      <el-timeline>
        <el-timeline-item
          v-for="(record, idx) in contractHistory"
          :key="idx"
          :timestamp="record.season"
          placement="top"
          :color="getContractEventColor(record.event_type)"
        >
          <div class="contract-event">
            <div class="contract-event-header">
              <el-tag size="small" :type="getContractEventTagType(record.event_type)">{{ record.event_type }}</el-tag>
              <span class="contract-team">{{ record.team_name }}</span>
            </div>
            <div class="contract-event-detail">
              <span v-if="record.salary > 0">年薪 {{ formatMoney(record.salary) }}</span>
              <span v-if="record.contract_years > 0">· {{ record.contract_years }}年</span>
              <span v-if="record.transfer_fee > 0">· 转会费 {{ formatMoney(record.transfer_fee) }}</span>
            </div>
            <div v-if="record.reason" class="contract-event-reason">{{ record.reason }}</div>
          </div>
        </el-timeline-item>
      </el-timeline>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, PropType } from 'vue'
import { Document, ArrowDown, ArrowRight } from '@element-plus/icons-vue'
import { formatMoney } from '@/utils'
import { PlayerContractRecord } from '@/api/tauri'
import { PlayerDetailInfo } from '@/composables/usePlayerDetail'

defineProps({
  player: {
    type: Object as PropType<PlayerDetailInfo>,
    required: true
  },
  contractHistory: {
    type: Array as PropType<PlayerContractRecord[]>,
    default: () => []
  },
  displayMarketValue: {
    type: Number,
    required: true
  }
})

defineEmits(['open-market-value'])

const showContractHistory = ref(false)

// Helper functions
const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getContractEventColor = (eventType: string) => {
  const colors: Record<string, string> = {
    '续约': '#67c23a',
    '自由签约': '#409eff',
    '转会加盟': '#e6a23c',
    '紧急签约': '#f56c6c',
    '赛季结算': '#909399',
  }
  return colors[eventType] || '#909399'
}

const getContractEventTagType = (eventType: string): '' | 'success' | 'warning' | 'danger' | 'info' => {
  const types: Record<string, '' | 'success' | 'warning' | 'danger' | 'info'> = {
    '续约': 'success',
    '自由签约': '',
    '转会加盟': 'warning',
    '紧急签约': 'danger',
    '赛季结算': 'info',
  }
  return types[eventType] || 'info'
}
</script>

<style scoped>
.detail-card {
  border-radius: 12px;
  height: 100%;
}

.contract-card {
  height: auto;
}

.contract-card .collapse-arrow {
  transition: transform 0.3s ease;
  transform: rotate(-90deg);
  margin-left: 4px;
}

.contract-card .collapse-arrow.expanded {
  transform: rotate(0deg);
}

.contract-history {
  margin-top: 8px;
}

.contract-history .contract-event .contract-event-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.contract-history .contract-event .contract-event-header .contract-team {
  font-weight: 500;
  color: #303133;
}

.contract-history .contract-event .contract-event-detail {
  font-size: 13px;
  color: #606266;
}

.contract-history .contract-event .contract-event-detail span + span {
  margin-left: 4px;
}

.contract-history .contract-event .contract-event-reason {
  font-size: 12px;
  color: #909399;
  margin-top: 2px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-light);
}

.info-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.info-label {
  font-size: 14px;
  color: var(--text-tertiary);
}

.info-value {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.info-value.highlight {
  color: var(--primary-color);
}

.info-value.money {
  color: #f59e0b;
}

.info-value.success {
  color: #22c55e;
}

.clickable {
  cursor: pointer;
}

.info-row.clickable:hover {
  background: var(--bg-secondary);
  margin: 0 -12px;
  padding: 12px 12px;
  border-radius: 8px;
}

.info-row .click-icon {
  font-size: 12px;
  margin-left: 4px;
  opacity: 0.5;
}

.info-row.clickable:hover .click-icon {
  opacity: 1;
}

:deep(.el-timeline-item__timestamp) {
  font-weight: 600;
  font-size: 14px;
  color: var(--primary-color);
}
</style>
