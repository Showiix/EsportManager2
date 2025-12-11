<template>
  <div class="player-detail">
    <!-- 返回按钮 -->
    <div class="back-nav">
      <el-button text @click="goBack">
        <el-icon><ArrowLeft /></el-icon>
        返回排行榜
      </el-button>
      <el-select v-model="selectedSeason" placeholder="选择赛季" style="width: 100px" size="small">
        <el-option v-for="s in seasons" :key="s.value" :label="s.label" :value="s.value" />
      </el-select>
    </div>

    <!-- 选手信息卡片 -->
    <div class="player-info-card" v-if="playerStats">
      <div class="player-header">
        <div class="player-basic">
          <h1 class="player-name">{{ playerStats.playerName }}</h1>
          <div class="player-meta">
            <el-tag :type="getPositionTagType(playerStats.position)" size="large">
              {{ getPositionName(playerStats.position) }}
            </el-tag>
            <span class="team-name">{{ getTeamName(playerStats.teamId) }}</span>
            <span class="region-name" v-if="playerStats.regionId">{{ playerStats.regionId }}</span>
          </div>
        </div>
        <div class="player-rank" v-if="playerRank">
          <span class="rank-label">排名</span>
          <span class="rank-value" :class="getRankClass(playerRank)">#{{ playerRank }}</span>
        </div>
      </div>
    </div>

    <!-- 数据统计卡片 -->
    <div class="stats-cards" v-if="playerStats">
      <el-card class="stat-card">
        <div class="stat-icon games">
          <el-icon><VideoCamera /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ playerStats.gamesPlayed || 0 }}</div>
          <div class="stat-label">参与局数</div>
        </div>
      </el-card>

      <el-card class="stat-card">
        <div class="stat-icon impact">
          <el-icon><TrendCharts /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value" :class="getImpactClass(playerStats.avgImpact)">
            {{ formatImpact(playerStats.avgImpact) }}
          </div>
          <div class="stat-label">平均影响力</div>
        </div>
      </el-card>

      <el-card class="stat-card">
        <div class="stat-icon consistency">
          <el-icon><Aim /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ (playerStats.consistencyScore || 0).toFixed(0) }}</div>
          <div class="stat-label">稳定性</div>
        </div>
      </el-card>

      <el-card class="stat-card">
        <div class="stat-icon score">
          <el-icon><Star /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value score-value">
            {{ (playerStats.yearlyTopScore || playerStats.avgImpact || 0).toFixed(1) }}
          </div>
          <div class="stat-label">得分</div>
        </div>
      </el-card>
    </div>

    <!-- 发挥波动图 -->
    <el-card class="chart-card" v-if="playerStats">
      <template #header>
        <div class="card-header">
          <span class="card-title">发挥波动</span>
          <div class="chart-legend">
            <span class="legend-item">
              <span class="legend-dot avg"></span>
              平均线 {{ (playerStats.avgImpact || 0).toFixed(1) }}
            </span>
          </div>
        </div>
      </template>
      <div class="chart-container">
        <v-chart class="chart" :option="chartOption" autoresize />
      </div>
    </el-card>

    <!-- 发挥区间 -->
    <el-card class="range-card" v-if="playerStats">
      <template #header>
        <span class="card-title">发挥区间</span>
      </template>
      <div class="range-stats">
        <div class="range-item best">
          <div class="range-label">最高发挥</div>
          <div class="range-value">{{ (playerStats.bestPerformance || 0).toFixed(1) }}</div>
        </div>
        <div class="range-item worst">
          <div class="range-label">最低发挥</div>
          <div class="range-value">{{ (playerStats.worstPerformance || 0).toFixed(1) }}</div>
        </div>
        <div class="range-item variance">
          <div class="range-label">波动范围</div>
          <div class="range-value">
            {{ ((playerStats.bestPerformance || 0) - (playerStats.worstPerformance || 0)).toFixed(1) }}
          </div>
        </div>
      </div>

      <!-- 发挥区间可视化 -->
      <div class="range-visualization">
        <div class="range-bar">
          <div
            class="range-fill"
            :style="{
              left: `${getBarPosition(playerStats.worstPerformance || 0)}%`,
              width: `${getBarWidth(playerStats.worstPerformance || 0, playerStats.bestPerformance || 0)}%`
            }"
          ></div>
          <div
            class="avg-marker"
            :style="{ left: `${getBarPosition(playerStats.avgImpact || 0)}%` }"
          >
            <span class="avg-tooltip">平均</span>
          </div>
        </div>
        <div class="range-labels">
          <span>60</span>
          <span>70</span>
          <span>80</span>
          <span>90</span>
          <span>100</span>
        </div>
      </div>
    </el-card>

    <!-- 无数据提示 -->
    <el-empty v-if="!playerStats" description="暂无该选手数据" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, VideoCamera, TrendCharts, Aim, Star } from '@element-plus/icons-vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  MarkLineComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  MarkLineComponent
])

const route = useRoute()
const router = useRouter()
const playerStore = usePlayerStore()
const gameStore = useGameStore()

// 状态
const selectedSeason = ref('S1')
const playerId = computed(() => route.params.playerId as string)
const playerRankValue = ref<number | null>(null)

// 赛季列表
const seasons = computed(() => {
  const currentSeason = gameStore.currentSeason || 1
  const list = []
  for (let i = 1; i <= currentSeason; i++) {
    list.push({ label: `S${i}`, value: `S${i}` })
  }
  return list
})

// 获取选手统计数据
const playerStats = computed(() => {
  void playerStore.updateTrigger
  return playerStore.getPlayerSeasonStats(playerId.value, selectedSeason.value)
})

// 异步获取选手排名
const fetchPlayerRank = async () => {
  try {
    const rankings = await playerStore.getSeasonImpactRanking(selectedSeason.value, 100)
    const index = rankings.findIndex(r => r.playerId === playerId.value)
    playerRankValue.value = index >= 0 ? index + 1 : null
  } catch (error) {
    console.error('获取排名失败:', error)
    playerRankValue.value = null
  }
}

// 获取选手排名
const playerRank = computed(() => playerRankValue.value)

// 模拟发挥数据（因为当前没有存储每场比赛的具体数据）
const performanceData = computed(() => {
  if (!playerStats.value) return []

  const count = playerStats.value.gamesPlayed || 0
  const avg = playerStats.value.avgImpact || 0
  const best = playerStats.value.bestPerformance || avg + 5
  const worst = playerStats.value.worstPerformance || avg - 5

  // 生成模拟的发挥波动数据
  const data: number[] = []
  for (let i = 0; i < count; i++) {
    // 在最差和最好之间生成随机值，偏向平均值
    const variance = (best - worst) / 2
    const random = (Math.random() - 0.5) * 2 * variance
    const value = avg + random
    data.push(Math.round(value * 10) / 10)
  }

  // 确保包含最高和最低值
  if (data.length > 0) {
    data[Math.floor(Math.random() * data.length)] = best
    data[Math.floor(Math.random() * data.length)] = worst
  }

  return data
})

// ECharts 配置
const chartOption = computed(() => {
  const data = performanceData.value
  const avg = playerStats.value?.avgImpact || 0

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const value = params[0]?.value || 0
        return `第 ${params[0]?.dataIndex + 1} 场<br/>发挥值: <b>${value}</b>`
      }
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: '10%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: data.map((_, i) => i + 1),
      axisLabel: {
        color: '#9ca3af'
      },
      axisLine: {
        lineStyle: { color: '#e5e7eb' }
      }
    },
    yAxis: {
      type: 'value',
      min: 60,
      max: 100,
      axisLabel: {
        color: '#9ca3af'
      },
      splitLine: {
        lineStyle: { color: '#f3f4f6' }
      }
    },
    series: [
      {
        name: '发挥值',
        type: 'line',
        smooth: true,
        symbol: 'circle',
        symbolSize: 8,
        data: data,
        lineStyle: {
          width: 3,
          color: '#3b82f6'
        },
        itemStyle: {
          color: '#3b82f6',
          borderWidth: 2,
          borderColor: '#fff'
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(59, 130, 246, 0.3)' },
              { offset: 1, color: 'rgba(59, 130, 246, 0.05)' }
            ]
          }
        },
        markLine: {
          silent: true,
          symbol: 'none',
          lineStyle: {
            color: '#f59e0b',
            type: 'dashed',
            width: 2
          },
          data: [
            {
              yAxis: avg,
              label: {
                formatter: `平均 ${avg.toFixed(1)}`,
                position: 'end',
                color: '#f59e0b'
              }
            }
          ]
        }
      }
    ]
  }
})

// 方法
const goBack = () => {
  router.push('/data-center')
}

const getPositionName = (position: PlayerPosition): string => {
  return POSITION_NAMES[position] || position
}

const getPositionTagType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'warning',
    MID: 'primary',
    ADC: 'success',
    SUP: 'info'
  }
  return types[position] || 'info'
}

const getTeamName = (teamId: string | number | null): string => {
  if (!teamId) return '-'
  const idStr = String(teamId)
  return idStr.split('-')[0] || idStr
}

const formatImpact = (value: number | null | undefined): string => {
  if (value == null) return '0.0'
  if (value > 0) return `+${value.toFixed(1)}`
  return value.toFixed(1)
}

const getImpactClass = (value: number | null | undefined): string => {
  if (value == null) return ''
  if (value > 5) return 'impact-high'
  if (value > 0) return 'impact-positive'
  if (value < 0) return 'impact-negative'
  return ''
}

const getRankClass = (rank: number): string => {
  if (rank === 1) return 'rank-gold'
  if (rank === 2) return 'rank-silver'
  if (rank === 3) return 'rank-bronze'
  if (rank <= 10) return 'rank-top10'
  return ''
}

const getBarPosition = (value: number): number => {
  // 60-100 映射到 0-100%
  return ((value - 60) / 40) * 100
}

const getBarWidth = (min: number, max: number): number => {
  return ((max - min) / 40) * 100
}

// 初始化
onMounted(() => {
  // 从 URL 参数获取赛季
  const seasonParam = route.query.season as string
  if (seasonParam) {
    selectedSeason.value = seasonParam
  }
  playerStore.loadFromStorage()
  fetchPlayerRank()
})

// 监听赛季变化
watch(selectedSeason, () => {
  // 更新 URL
  router.replace({
    query: { ...route.query, season: selectedSeason.value }
  })
  fetchPlayerRank()
})
</script>

<style scoped lang="scss">
.player-detail {
  padding: 24px;
  min-height: 100%;

  .back-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }

  .player-info-card {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border-radius: 16px;
    padding: 32px;
    margin-bottom: 24px;
    color: white;

    .player-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
    }

    .player-basic {
      .player-name {
        font-size: 36px;
        font-weight: 700;
        margin: 0 0 12px 0;
      }

      .player-meta {
        display: flex;
        align-items: center;
        gap: 12px;

        .team-name,
        .region-name {
          font-size: 16px;
          opacity: 0.9;
        }
      }
    }

    .player-rank {
      text-align: center;
      background: rgba(255, 255, 255, 0.2);
      border-radius: 12px;
      padding: 16px 24px;

      .rank-label {
        display: block;
        font-size: 12px;
        opacity: 0.8;
        margin-bottom: 4px;
      }

      .rank-value {
        font-size: 32px;
        font-weight: 700;

        &.rank-gold { color: #ffd700; }
        &.rank-silver { color: #c0c0c0; }
        &.rank-bronze { color: #cd7f32; }
        &.rank-top10 { color: #a5f3fc; }
      }
    }
  }

  .stats-cards {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;

    .stat-card {
      :deep(.el-card__body) {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 20px;
      }

      .stat-icon {
        width: 56px;
        height: 56px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 24px;
        color: white;

        &.games { background: linear-gradient(135deg, #3b82f6, #1d4ed8); }
        &.impact { background: linear-gradient(135deg, #10b981, #059669); }
        &.consistency { background: linear-gradient(135deg, #f59e0b, #d97706); }
        &.score { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
      }

      .stat-content {
        .stat-value {
          font-size: 28px;
          font-weight: 700;
          color: #1f2937;

          &.impact-high { color: #059669; }
          &.impact-positive { color: #10b981; }
          &.impact-negative { color: #ef4444; }
          &.score-value { color: #7c3aed; }
        }

        .stat-label {
          font-size: 14px;
          color: #6b7280;
          margin-top: 4px;
        }
      }
    }
  }

  .chart-card {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .card-title {
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }

      .chart-legend {
        .legend-item {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 14px;
          color: #6b7280;

          .legend-dot {
            width: 12px;
            height: 3px;
            border-radius: 2px;

            &.avg { background: #f59e0b; }
          }
        }
      }
    }

    .chart-container {
      height: 300px;

      .chart {
        width: 100%;
        height: 100%;
      }
    }
  }

  .range-card {
    .card-title {
      font-size: 18px;
      font-weight: 600;
      color: #1f2937;
    }

    .range-stats {
      display: flex;
      gap: 48px;
      margin-bottom: 32px;

      .range-item {
        text-align: center;

        .range-label {
          font-size: 14px;
          color: #6b7280;
          margin-bottom: 8px;
        }

        .range-value {
          font-size: 32px;
          font-weight: 700;
        }

        &.best .range-value { color: #10b981; }
        &.worst .range-value { color: #ef4444; }
        &.variance .range-value { color: #6b7280; }
      }
    }

    .range-visualization {
      .range-bar {
        position: relative;
        height: 24px;
        background: #f3f4f6;
        border-radius: 12px;
        overflow: visible;

        .range-fill {
          position: absolute;
          top: 0;
          height: 100%;
          background: linear-gradient(90deg, #ef4444, #f59e0b, #10b981);
          border-radius: 12px;
        }

        .avg-marker {
          position: absolute;
          top: -8px;
          width: 4px;
          height: 40px;
          background: #f59e0b;
          border-radius: 2px;
          transform: translateX(-50%);

          .avg-tooltip {
            position: absolute;
            bottom: 100%;
            left: 50%;
            transform: translateX(-50%);
            background: #f59e0b;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
            white-space: nowrap;
            margin-bottom: 4px;
          }
        }
      }

      .range-labels {
        display: flex;
        justify-content: space-between;
        margin-top: 8px;
        font-size: 12px;
        color: #9ca3af;
      }
    }
  }
}

@media (max-width: 1024px) {
  .player-detail {
    .stats-cards {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

@media (max-width: 640px) {
  .player-detail {
    .stats-cards {
      grid-template-columns: 1fr;
    }

    .player-info-card {
      .player-header {
        flex-direction: column;
        gap: 20px;
      }

      .player-rank {
        align-self: flex-start;
      }
    }
  }
}
</style>
