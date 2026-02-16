<template>
  <el-card class="detail-card">
    <template #header>
      <div class="card-header">
        <h2>
          <el-icon><TrendCharts /></el-icon>
          职业生涯
        </h2>
      </div>
    </template>
    <div class="info-list">
      <div class="info-row">
        <span class="info-label">加入赛季</span>
        <span class="info-value">S{{ debutSeason }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">职业年数</span>
        <span class="info-value">{{ careerYears }} 年</span>
      </div>
      <div class="info-row">
        <span class="info-label">冠军数</span>
        <span class="info-value gold">{{ championCount }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">总荣誉</span>
        <span class="info-value">{{ honorsCount }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">成长空间</span>
        <span class="info-value purple">+{{ player.potential - player.ability }}</span>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { PropType } from 'vue'
import { TrendCharts } from '@element-plus/icons-vue'
import { PlayerDetailInfo } from '@/composables/usePlayerDetail'

defineProps({
  player: {
    type: Object as PropType<PlayerDetailInfo>,
    required: true
  },
  careerYears: {
    type: Number,
    required: true
  },
  debutSeason: {
    type: Number,
    required: true
  },
  championCount: {
    type: Number,
    required: true
  },
  honorsCount: {
    type: Number,
    required: true
  }
})
</script>

<style scoped>
.detail-card {
  border-radius: 12px;
  height: 100%;
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

.info-value.gold {
  color: #fbbf24;
  font-weight: 700;
}

.info-value.purple {
  color: #8b5cf6;
}
</style>
