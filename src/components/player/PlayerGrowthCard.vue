<template>
  <el-card class="growth-card">
    <template #header>
      <div class="card-header">
        <h2>
          <el-icon><TrendCharts /></el-icon>
          成长记录
        </h2>
      </div>
    </template>

    <div v-if="loading" class="loading-state">
      <el-skeleton :rows="4" animated />
    </div>

    <el-empty v-else-if="logs.length === 0" description="暂无成长记录（需经历转会期结算）" :image-size="60" />

    <el-table v-else :data="logs" stripe class="growth-table">
      <el-table-column prop="season_id" label="赛季" width="70" align="center">
        <template #default="{ row }">
          S{{ row.season_id }}
        </template>
      </el-table-column>
      <el-table-column prop="team_name" label="战队" width="120">
        <template #default="{ row }">
          {{ row.team_name || '自由球员' }}
        </template>
      </el-table-column>
      <el-table-column prop="age" label="年龄" width="60" align="center" />
      <el-table-column label="能力" width="110" align="center">
        <template #default="{ row }">
          <span>{{ row.old_ability }}</span>
          <span class="arrow">→</span>
          <span :class="abilityChangeClass(row)">{{ row.new_ability }}</span>
        </template>
      </el-table-column>
      <el-table-column label="变化" width="70" align="center">
        <template #default="{ row }">
          <el-tag :type="changeTagType(row.new_ability - row.old_ability)" size="small">
            {{ formatChange(row.new_ability - row.old_ability) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="潜力" width="110" align="center">
        <template #default="{ row }">
          <template v-if="row.new_potential !== row.old_potential">
            <span>{{ row.old_potential }}</span>
            <span class="arrow">→</span>
            <span :class="row.new_potential > row.old_potential ? 'val-up' : 'val-down'">{{ row.new_potential }}</span>
          </template>
          <span v-else class="text-gray">{{ row.old_potential }}</span>
        </template>
      </el-table-column>
      <el-table-column label="成长因子" min-width="240">
        <template #default="{ row }">
          <div class="factor-tags">
            <el-tag v-if="row.base_growth !== 0" size="small" effect="plain" :type="row.base_growth > 0 ? '' : 'danger'">
              基础{{ row.base_growth > 0 ? '+' : '' }}{{ row.base_growth.toFixed(1) }}
            </el-tag>
            <el-tag v-if="row.age_coeff !== 1.0" size="small" effect="plain" type="info">
              年龄×{{ row.age_coeff.toFixed(2) }}
            </el-tag>
            <el-tag v-if="row.playtime_coeff !== 1.0" size="small" effect="plain" :type="row.playtime_coeff >= 1.0 ? 'success' : 'warning'">
              上场×{{ row.playtime_coeff.toFixed(1) }}
            </el-tag>
            <el-tag v-if="row.mentor_coeff !== 1.0" size="small" effect="plain" type="success">
              导师×{{ row.mentor_coeff.toFixed(2) }}
            </el-tag>
            <el-tag v-if="row.synergy_coeff !== 1.0" size="small" effect="plain" type="success">
              协同×{{ row.synergy_coeff.toFixed(2) }}
            </el-tag>
            <el-tag v-if="row.facility_coeff !== 1.0" size="small" effect="plain" type="">
              设施×{{ row.facility_coeff.toFixed(2) }}
            </el-tag>
            <el-tag v-if="row.prodigy_mod !== 1.0" size="small" effect="plain" type="warning">
              神童×{{ row.prodigy_mod.toFixed(1) }}
            </el-tag>
            <el-tag v-if="row.perf_bonus !== 0" size="small" effect="plain" :type="row.perf_bonus > 0 ? 'success' : 'danger'">
              表现{{ row.perf_bonus > 0 ? '+' : '' }}{{ row.perf_bonus.toFixed(1) }}
            </el-tag>
            <el-tag v-if="Math.abs(row.fluctuation) >= 0.5" size="small" effect="plain" type="info">
              波动{{ row.fluctuation > 0 ? '+' : '' }}{{ row.fluctuation.toFixed(1) }}
            </el-tag>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="事件" width="120" align="center">
        <template #default="{ row }">
          <el-tag v-if="row.growth_event" :type="eventTagType(row.growth_event)" size="small" effect="dark">
            {{ row.growth_event }}
          </el-tag>
          <span v-else class="text-gray">-</span>
        </template>
      </el-table-column>
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { TrendCharts } from '@element-plus/icons-vue'
import { getPlayerGrowthLogs, saveApi } from '@/api/tauri'
import type { PlayerGrowthLogItem } from '@/api/tauri'

const props = defineProps<{
  playerId: number | string
}>()

const loading = ref(true)
const logs = ref<PlayerGrowthLogItem[]>([])

const loadData = async () => {
  if (!props.playerId) return
  loading.value = true
  try {
    const saveId = await saveApi.getCurrentSaveId()
    if (saveId) {
      const pid = typeof props.playerId === 'string' ? parseInt(props.playerId) : props.playerId
      if (!isNaN(pid)) {
        logs.value = await getPlayerGrowthLogs(saveId, pid)
      }
    }
  } catch (error) {
    console.error('Failed to load growth logs:', error)
  } finally {
    loading.value = false
  }
}

const formatChange = (diff: number) => {
  if (diff > 0) return `+${diff}`
  if (diff < 0) return `${diff}`
  return '0'
}

const changeTagType = (diff: number): '' | 'success' | 'warning' | 'danger' | 'info' => {
  if (diff >= 3) return 'success'
  if (diff > 0) return ''
  if (diff < -2) return 'danger'
  if (diff < 0) return 'warning'
  return 'info'
}

const abilityChangeClass = (row: PlayerGrowthLogItem) => {
  if (row.new_ability > row.old_ability) return 'val-up'
  if (row.new_ability < row.old_ability) return 'val-down'
  return ''
}

const eventTagType = (event: string): '' | 'success' | 'warning' | 'danger' | 'info' => {
  const map: Record<string, '' | 'success' | 'warning' | 'danger' | 'info'> = {
    '觉醒': 'success',
    '天赋觉醒': 'success',
    '二次巅峰': 'success',
    '突破赛季': '',
    '瓶颈期': 'warning',
    '停滞赛季': 'info',
    '心态崩盘': 'danger',
    '伤病': 'danger',
  }
  return map[event] || 'info'
}

onMounted(() => loadData())
watch(() => props.playerId, () => loadData())
</script>

<style scoped>
.growth-card {
  margin-top: 20px;
  margin-bottom: 20px;
}

.card-header h2 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #1f2937;
}

.growth-table {
  width: 100%;
}

.arrow {
  margin: 0 4px;
  color: #9ca3af;
}

.val-up {
  color: #67c23a;
  font-weight: 600;
}

.val-down {
  color: #f56c6c;
  font-weight: 600;
}

.text-gray {
  color: #9ca3af;
}

.factor-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.loading-state {
  padding: 20px;
}
</style>
