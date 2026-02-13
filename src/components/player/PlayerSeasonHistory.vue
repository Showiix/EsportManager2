<template>
  <el-card class="history-card">
    <template #header>
      <div class="card-header">
        <h2>
          <el-icon><Clock /></el-icon>
          赛季历史
        </h2>
      </div>
    </template>

    <el-table :data="seasonHistory" stripe class="history-table">
      <el-table-column prop="season" label="赛季" width="120" align="center" />
      <el-table-column label="所属战队" width="150">
        <template #default="{ row }">
          <div class="team-cell">
            <div class="team-avatar mini" :class="playerRegion.toLowerCase()">
              {{ (row.team_name || row.team || '').substring(0, 2) }}
            </div>
            <span>{{ row.team_name || row.team }}</span>
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="ability" label="能力值" width="120" align="center">
        <template #default="{ row }">
          <span class="ability-value" :style="{ color: getAbilityColor(row.ability) }">
            {{ row.ability }}
          </span>
        </template>
      </el-table-column>
      <el-table-column prop="potential" label="潜力值" width="120" align="center">
        <template #default="{ row }">
          <span class="potential-value">{{ row.potential }}</span>
        </template>
      </el-table-column>
      <el-table-column label="成长" width="100" align="center">
        <template #default="{ row, $index }">
          <template v-if="$index > 0">
            <el-tag
              :type="row.ability - seasonHistory[$index - 1].ability > 0 ? 'success' : row.ability - seasonHistory[$index - 1].ability < 0 ? 'danger' : 'info'"
              size="small"
            >
              {{ row.ability - seasonHistory[$index - 1].ability > 0 ? '+' : '' }}{{ row.ability - seasonHistory[$index - 1].ability }}
            </el-tag>
          </template>
          <span v-else class="text-gray">-</span>
        </template>
      </el-table-column>
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { PropType } from 'vue'
import { Clock } from '@element-plus/icons-vue'
import { PlayerHistorySeason } from '@/composables/usePlayerDetail'

defineProps({
  seasonHistory: {
    type: Array as PropType<PlayerHistorySeason[]>,
    default: () => []
  },
  playerRegion: {
    type: String,
    required: true
  }
})

// Helper function
const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}
</script>

<style scoped>
.history-card {
  border-radius: 12px;
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

.history-table {
  border-radius: 8px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-avatar.mini {
  width: 24px;
  height: 24px;
  font-size: 10px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  background-color: #909399; /* Fallback color */
}

/* Region-specific avatar background colors - assuming global styles or passed classes */
.team-avatar.lpl { background-color: #ef4444; }
.team-avatar.lck { background-color: #3b82f6; }
.team-avatar.lec { background-color: #10b981; }
.team-avatar.lcs { background-color: #f59e0b; }

.ability-value {
  font-weight: 700;
}

.potential-value {
  color: #8b5cf6;
  font-weight: 600;
}

.text-gray {
  color: var(--text-placeholder);
}
</style>
