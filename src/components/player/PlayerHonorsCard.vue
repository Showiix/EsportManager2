<template>
  <el-card class="honors-card">
    <template #header>
      <div class="card-header clickable" @click="toggleExpanded">
        <h2>
          <el-icon><Trophy /></el-icon>
          荣誉记录
        </h2>
        <div class="header-right">
          <span class="count-badge">共 {{ honors.length }} 项荣誉</span>
          <el-icon class="collapse-arrow" :class="{ expanded: isExpanded }"><ArrowDown /></el-icon>
        </div>
      </div>
    </template>

    <template v-if="isExpanded">
      <el-empty v-if="honors.length === 0" description="暂无荣誉记录">
        <template #image>
          <el-icon class="empty-icon"><Trophy /></el-icon>
        </template>
      </el-empty>

      <el-timeline v-else>
        <el-timeline-item
          v-for="honor in honors"
          :key="`${honor.season}-${honor.tournament}`"
          :timestamp="honor.season"
          placement="top"
          :color="getHonorColor(honor.position)"
          size="large"
        >
          <el-card class="honor-card" :class="getHonorClass(honor.position)" shadow="hover">
            <div class="honor-content">
              <div class="honor-icon">
                {{ getHonorEmoji(honor.position) }}
              </div>
              <div class="honor-info">
                <div class="honor-title">{{ honor.tournament }}</div>
                <el-tag :type="getHonorTagType(honor.position)" size="default" effect="dark">
                  {{ honor.position }}
                </el-tag>
              </div>
            </div>
          </el-card>
        </el-timeline-item>
      </el-timeline>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { ref, PropType } from 'vue'
import { Trophy, ArrowDown } from '@element-plus/icons-vue'
import { PlayerHonor } from '@/composables/usePlayerDetail'

defineProps({
  honors: {
    type: Array as PropType<PlayerHonor[]>,
    default: () => []
  }
})

const isExpanded = ref(false)

const toggleExpanded = () => {
  isExpanded.value = !isExpanded.value
}

// Helper functions for honor styling
const getHonorColor = (position: string) => {
  const colors: Record<string, string> = {
    '冠军': '#fbbf24',
    '冠军成员': '#fbbf24',
    '亚军': '#9ca3af',
    '亚军成员': '#9ca3af',
    '季军': '#f97316',
    '季军成员': '#f97316',
    '殿军': '#3b82f6',
    '殿军成员': '#3b82f6',
    '赛事MVP': '#ef4444',
    '决赛MVP': '#ef4444',
    '常规赛MVP': '#ef4444',
    '季后赛FMVP': '#ef4444',
  }
  return colors[position] || '#3b82f6'
}

const getHonorClass = (position: string) => {
  const classes: Record<string, string> = {
    '冠军': 'champion',
    '冠军成员': 'champion',
    '亚军': 'runner-up',
    '亚军成员': 'runner-up',
    '季军': 'third-place',
    '季军成员': 'third-place',
    '殿军': 'fourth-place',
    '殿军成员': 'fourth-place',
  }
  return classes[position] || ''
}

const getHonorEmoji = (position: string) => {
  const emojis: Record<string, string> = {
    '冠军': '🏆',
    '冠军成员': '🏆',
    '亚军': '🥈',
    '亚军成员': '🥈',
    '季军': '🥉',
    '季军成员': '🥉',
    '殿军': '4️⃣',
    '殿军成员': '4️⃣',
    '赛事MVP': '⭐',
    '决赛MVP': '⭐',
    '常规赛MVP': '⭐',
    '季后赛FMVP': '⭐',
  }
  return emojis[position] || '🏅'
}

const getHonorTagType = (position: string) => {
  const types: Record<string, string> = {
    '冠军': 'warning',
    '冠军成员': 'warning',
    '亚军': '',           // 默认银色
    '亚军成员': '',
    '季军': 'success',    // 绿色/铜色
    '季军成员': 'success',
    '殿军': 'info',       // 蓝色
    '殿军成员': 'info',
    '赛事MVP': 'danger',  // 红色
    '决赛MVP': 'danger',
    '常规赛MVP': 'danger',
    '季后赛FMVP': 'danger',
    '常规赛第一': 'primary',
  }
  return types[position] || 'primary'
}
</script>

<style scoped>
.honors-card {
  border-radius: 12px;
  margin-bottom: 20px;
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

.clickable {
  cursor: pointer;
  user-select: none;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.count-badge {
  font-size: 14px;
  color: var(--text-tertiary);
}

.collapse-arrow {
  transition: transform 0.3s ease;
  transform: rotate(-90deg);
  color: #909399;
}

.collapse-arrow.expanded {
  transform: rotate(0deg);
}

.empty-icon {
  font-size: 64px;
}

.honor-card {
  margin-bottom: 0;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.honor-card:hover {
  transform: translateX(4px);
}

.honor-card.champion {
  border-left: 4px solid #fbbf24;
  background: linear-gradient(135deg, #fffbeb 0%, #ffffff 100%);
}

.honor-card.runner-up {
  border-left: 4px solid #9ca3af;
}

.honor-card.third-place {
  border-left: 4px solid #f97316;
}

.honor-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.honor-icon {
  font-size: 32px;
}

.honor-info {
  flex: 1;
}

.honor-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

:deep(.el-timeline-item__timestamp) {
  font-weight: 600;
  font-size: 14px;
  color: var(--primary-color);
}
</style>
