<template>
  <el-card class="condition-card radar-card">
    <template #header>
      <div class="card-header">
        <h2>
          <el-icon class="header-icon"><DataAnalysis /></el-icon>
          能力分析
        </h2>
        <span v-if="playerStats" class="count-badge">{{ playerStats.games_played }} 场比赛</span>
      </div>
    </template>

    <!-- 五维雷达图 -->
    <div v-if="playerStats" class="radar-content">
      <div ref="radarChartRef" class="radar-chart"></div>

      <!-- 详细数值展示 -->
      <div class="radar-stats">
        <div class="stat-row">
          <span class="stat-label">
            <el-icon class="stat-icon impact"><Lightning /></el-icon>
            影响力
          </span>
          <div class="stat-bar-wrapper">
            <div class="stat-bar" :style="{ width: (radarData?.impact || 0) + '%' }"></div>
          </div>
          <span class="stat-value">{{ radarData?.impact || 0 }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">
            <el-icon class="stat-icon performance"><Aim /></el-icon>
            发挥
          </span>
          <div class="stat-bar-wrapper">
            <div class="stat-bar performance" :style="{ width: (radarData?.performance || 0) + '%' }"></div>
          </div>
          <span class="stat-value">{{ radarData?.performance || 0 }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">
            <span class="stat-icon peak">🌍</span>
            大赛
          </span>
          <div class="stat-bar-wrapper">
            <div class="stat-bar peak" :style="{ width: (radarData?.bigStage || 0) + '%' }"></div>
          </div>
          <span class="stat-value">{{ radarData?.bigStage || 0 }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">
            <span class="stat-icon consistency">🛡️</span>
            稳定性
          </span>
          <div class="stat-bar-wrapper">
            <div class="stat-bar consistency" :style="{ width: (radarData?.consistency || 0) + '%' }"></div>
          </div>
          <span class="stat-value">{{ radarData?.consistency || 0 }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">
            <span class="stat-icon consistency">📊</span>
            出场
          </span>
          <div class="stat-bar-wrapper">
            <div class="stat-bar consistency" :style="{ width: (radarData?.games || 0) + '%' }"></div>
          </div>
          <span class="stat-value">{{ radarData?.games || 0 }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">
            <el-icon class="stat-icon honor"><Trophy /></el-icon>
            荣誉
          </span>
          <div class="stat-bar-wrapper">
            <div class="stat-bar honor" :style="{ width: (radarData?.honor || 0) + '%' }"></div>
          </div>
          <span class="stat-value">{{ radarData?.honor || 0 }}</span>
        </div>
      </div>
    </div>

    <el-empty v-else description="暂无比赛数据" :image-size="60">
      <template #image>
        <el-icon class="empty-icon"><DataAnalysis /></el-icon>
      </template>
    </el-empty>
  </el-card>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick, PropType } from 'vue'
import { DataAnalysis, Lightning, Aim, Trophy } from '@element-plus/icons-vue'
import * as echarts from 'echarts'
import { PlayerSeasonStats } from '@/composables/usePlayerDetail'

const props = defineProps({
  playerStats: {
    type: Object as PropType<PlayerSeasonStats | null>,
    default: null
  },
  radarData: {
    type: Object as PropType<{
      impact: number
      performance: number
      bigStage: number
      consistency: number
      games: number
      honor: number
    } | null>,
    default: null
  },
  playerName: {
    type: String,
    default: ''
  }
})

// 雷达图相关
const radarChartRef = ref<HTMLDivElement | null>(null)
let radarChart: echarts.ECharts | null = null

// 初始化雷达图
const initRadarChart = () => {
  if (!radarChartRef.value || !props.playerStats || !props.radarData) return

  // 如果已有实例，先销毁
  if (radarChart) {
    radarChart.dispose()
  }

  radarChart = echarts.init(radarChartRef.value)

  const data = props.radarData
  if (!data) return

  const option: echarts.EChartsOption = {
    radar: {
      indicator: [
        { name: '影响力', max: 100 },
        { name: '发挥', max: 100 },
        { name: '大赛', max: 100 },
        { name: '稳定性', max: 100 },
        { name: '出场', max: 100 },
        { name: '荣誉', max: 100 }
      ],
      shape: 'polygon',
      splitNumber: 4,
      axisName: {
        color: '#333333',
        fontSize: 12,
        fontWeight: 500
      },
      splitLine: {
        lineStyle: {
          color: 'rgba(128, 128, 128, 0.2)'
        }
      },
      splitArea: {
        areaStyle: {
          color: ['rgba(59, 130, 246, 0.02)', 'rgba(59, 130, 246, 0.04)', 'rgba(59, 130, 246, 0.06)', 'rgba(59, 130, 246, 0.08)']
        }
      },
      axisLine: {
        lineStyle: {
          color: 'rgba(128, 128, 128, 0.3)'
        }
      }
    },
    series: [{
      type: 'radar',
      data: [{
        value: [data.impact, data.performance, data.bigStage, data.consistency, data.games, data.honor],
        name: props.playerName,
        areaStyle: {
          color: {
            type: 'radial',
            x: 0.5,
            y: 0.5,
            r: 0.5,
            colorStops: [
              { offset: 0, color: 'rgba(59, 130, 246, 0.1)' },
              { offset: 1, color: 'rgba(59, 130, 246, 0.4)' }
            ]
          }
        },
        lineStyle: {
          color: '#3b82f6',
          width: 2
        },
        itemStyle: {
          color: '#3b82f6',
          borderColor: '#fff',
          borderWidth: 2
        },
        symbol: 'circle',
        symbolSize: 8
      }]
    }]
  }

  radarChart.setOption(option)

  // 监听窗口大小变化
  window.addEventListener('resize', () => {
    radarChart?.resize()
  })
}

// 监听 radarData 变化，重新渲染雷达图
watch(() => props.radarData, async () => {
  await nextTick()
  initRadarChart()
}, { deep: true })

onMounted(() => {
  nextTick(() => {
    initRadarChart()
  })
})
</script>

<style scoped>
.condition-card {
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

.header-icon {
  font-size: 18px;
  margin-right: 4px;
}

.count-badge {
  font-size: 14px;
  color: var(--text-tertiary);
}

.radar-card {
  min-height: 400px;
}

.radar-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.radar-chart {
  width: 100%;
  height: 220px;
}

.radar-stats {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.stat-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-row .stat-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 80px;
}

.stat-icon {
  font-size: 14px;
}

.stat-bar-wrapper {
  flex: 1;
  height: 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow: hidden;
}

.stat-bar {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #60a5fa);
  border-radius: 4px;
  transition: width 0.5s ease;
}

.stat-bar.performance {
  background: linear-gradient(90deg, #22c55e, #4ade80);
}

.stat-bar.consistency {
  background: linear-gradient(90deg, #8b5cf6, #a78bfa);
}

.stat-bar.peak {
  background: linear-gradient(90deg, #f59e0b, #fbbf24);
}

.stat-bar.honor {
  background: linear-gradient(90deg, #ef4444, #f87171);
}

.stat-row .stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  min-width: 32px;
  text-align: right;
}

.empty-icon {
  font-size: 64px;
}
</style>
