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

    <!-- 选手状态卡片 -->
    <el-card class="status-card" v-if="playerFullDetail">
      <template #header>
        <div class="status-header">
          <span class="status-title">
            <el-icon><User /></el-icon>
            选手状态
          </span>
        </div>
      </template>

      <div class="status-content">
        <!-- 满意度 -->
        <div class="status-item">
          <div class="status-label">
            <span class="label-text">满意度</span>
            <span class="status-value" :class="getSatisfactionClass(playerFullDetail.player.satisfaction)">
              {{ playerFullDetail.player.satisfaction }}
            </span>
          </div>
          <el-progress
            :percentage="playerFullDetail.player.satisfaction"
            :stroke-width="8"
            :color="getSatisfactionColor(playerFullDetail.player.satisfaction)"
            :show-text="false"
          />
          <div class="status-desc">{{ getSatisfactionDesc(playerFullDetail.player.satisfaction) }}</div>
        </div>

        <!-- 忠诚度 -->
        <div class="status-item">
          <div class="status-label">
            <span class="label-text">忠诚度</span>
            <span class="status-value">{{ playerFullDetail.player.loyalty }}</span>
            <el-tag
              size="small"
              :type="getLoyaltyTagType(playerFullDetail.player.loyalty)"
              effect="plain"
            >
              {{ getLoyaltyTypeName(playerFullDetail.player.loyalty) }}
            </el-tag>
          </div>
          <el-progress
            :percentage="playerFullDetail.player.loyalty"
            :stroke-width="8"
            :color="getLoyaltyColor(playerFullDetail.player.loyalty)"
            :show-text="false"
          />
          <div class="status-desc">{{ getLoyaltyDesc(playerFullDetail.player.loyalty) }}</div>
        </div>

        <!-- 离队意愿警告 -->
        <el-alert
          v-if="playerFullDetail.player.satisfaction < 40"
          type="warning"
          :title="playerFullDetail.player.satisfaction < 30 ? '该选手可能想要离队' : '该选手满意度较低'"
          :description="'满意度过低可能导致选手在转会窗口主动申请离队'"
          :closable="false"
          show-icon
          class="departure-alert"
        />
      </div>
    </el-card>

    <!-- 影响力走势图 -->
    <el-card class="chart-card" v-if="playerStats">
      <template #header>
        <div class="card-header">
          <span class="card-title">影响力走势</span>
          <div class="chart-legend" v-if="performanceData.length > 0">
            <span class="legend-item">
              <span class="legend-dot avg"></span>
              平均影响力 {{ (playerStats.avgImpact || 0).toFixed(1) }}
            </span>
          </div>
        </div>
      </template>
      <div class="chart-container">
        <v-chart v-if="performanceData.length > 0" class="chart" :option="chartOption" autoresize />
        <el-empty v-else description="暂无比赛数据" :image-size="100" />
      </div>
    </el-card>

    <!-- 数据分析图表 -->
    <div class="charts-row" v-if="playerStats && performanceData.length > 0">
      <!-- 影响力分布 -->
      <el-card class="distribution-card">
        <template #header>
          <span class="card-title">影响力分布</span>
        </template>
        <div class="chart-container small">
          <v-chart class="chart" :option="distributionChartOption" autoresize />
        </div>
      </el-card>

      <!-- 稳定性仪表盘 -->
      <el-card class="gauge-card">
        <template #header>
          <span class="card-title">稳定性评分</span>
        </template>
        <div class="chart-container small">
          <v-chart class="chart" :option="gaugeChartOption" autoresize />
        </div>
      </el-card>

      <!-- 表现统计 -->
      <el-card class="summary-card">
        <template #header>
          <span class="card-title">表现统计</span>
        </template>
        <div class="summary-stats">
          <div class="summary-item">
            <div class="summary-icon positive">
              <el-icon><Top /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ positiveGames }}</div>
              <div class="summary-label">正向表现</div>
            </div>
          </div>
          <div class="summary-item">
            <div class="summary-icon negative">
              <el-icon><Bottom /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ negativeGames }}</div>
              <div class="summary-label">负向表现</div>
            </div>
          </div>
          <div class="summary-item">
            <div class="summary-icon highlight">
              <el-icon><StarFilled /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ highlightGames }}</div>
              <div class="summary-label">亮眼表现 (10+)</div>
            </div>
          </div>
          <div class="summary-item">
            <div class="summary-icon rate">
              <el-icon><Promotion /></el-icon>
            </div>
            <div class="summary-content">
              <div class="summary-value">{{ positiveRate }}%</div>
              <div class="summary-label">正向率</div>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 无数据提示 -->
    <el-empty v-if="!playerStats" description="暂无该选手数据" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, VideoCamera, TrendCharts, Aim, Star, Top, Bottom, StarFilled, Promotion, User } from '@element-plus/icons-vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart, GaugeChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  MarkLineComponent,
  LegendComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { teamApi, statsApi, playerApi } from '@/api/tauri'
import type { PlayerPosition, PlayerSeasonStats } from '@/types/player'
import type { PlayerFullDetail } from '@/api/tauri'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'

const logger = createLogger('DataCenterPlayerDetail')

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  GaugeChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  MarkLineComponent,
  LegendComponent
])

const route = useRoute()
const router = useRouter()
const playerStore = usePlayerStore()
const gameStore = useGameStore()

// 本地战队映射表
const teamsMap = ref<Map<number, string>>(new Map())

// 状态
const selectedSeason = ref('S1')
const playerId = computed(() => route.params.playerId as string)
const playerRankValue = ref<number | null>(null)
const playerStatsData = ref<PlayerSeasonStats | null>(null)
const loading = ref(false)
const impactHistory = ref<number[]>([])  // 真实影响力历史数据
const playerFullDetail = ref<PlayerFullDetail | null>(null)  // 选手完整详情（含满意度/忠诚度）

// 赛季列表
const seasons = computed(() => {
  const currentSeason = gameStore.currentSeason || 1
  const list = []
  for (let i = 1; i <= currentSeason; i++) {
    list.push({ label: `S${i}`, value: `S${i}` })
  }
  return list
})

// 获取选手统计数据（从数据库获取）
const playerStats = computed(() => playerStatsData.value)

// 异步获取选手数据
const fetchPlayerStats = async () => {
  loading.value = true
  const playerIdNum = Number(playerId.value)

  try {
    // 从数据库获取排行榜数据，然后找到当前选手
    const rankings = await playerStore.getSeasonImpactRanking(selectedSeason.value, 500)
    const found = rankings.find(r => Number(r.playerId) === playerIdNum)
    playerStatsData.value = found || null

    // 同时更新排名
    if (found) {
      const index = rankings.findIndex(r => Number(r.playerId) === playerIdNum)
      playerRankValue.value = index >= 0 ? index + 1 : null
    } else {
      playerRankValue.value = null
    }
  } catch (error) {
    logger.error('获取选手统计数据失败:', error)
    playerStatsData.value = null
    playerRankValue.value = null
  }

  // 单独获取影响力历史数据（不影响主数据显示）
  try {
    const seasonNum = Number(selectedSeason.value.replace('S', ''))
    logger.debug('[DataCenterPlayerDetail] Fetching impact history for player:', playerIdNum, 'season:', seasonNum)
    const history = await statsApi.getPlayerImpactHistory(playerIdNum, seasonNum)
    logger.debug('[DataCenterPlayerDetail] Impact history result:', history)
    impactHistory.value = history || []
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取影响力历史数据失败:', error)
    impactHistory.value = []
  }

  // 获取选手完整详情（含满意度/忠诚度）
  try {
    const detail = await playerApi.getPlayerFullDetail(playerIdNum)
    playerFullDetail.value = detail
    logger.debug('[DataCenterPlayerDetail] Player full detail:', detail)
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取选手完整详情失败:', error)
    playerFullDetail.value = null
  }

  loading.value = false
}

// 获取选手排名
const playerRank = computed(() => playerRankValue.value)

// 使用真实的影响力历史数据
const performanceData = computed(() => {
  // 如果有真实数据，直接使用
  if (impactHistory.value && impactHistory.value.length > 0) {
    return impactHistory.value.map(v => Math.round(v * 10) / 10)
  }
  // 没有真实数据时返回空数组
  return []
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
        return `第 ${params[0]?.dataIndex + 1} 场<br/>影响力: <b>${value}</b>`
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
      min: -30,
      max: 30,
      axisLabel: {
        color: '#9ca3af',
        formatter: (value: number) => value > 0 ? `+${value}` : `${value}`
      },
      splitLine: {
        lineStyle: { color: '#f3f4f6' }
      }
    },
    series: [
      {
        name: '影响力',
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
                formatter: `平均 ${avg >= 0 ? '+' : ''}${avg.toFixed(1)}`,
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

// 表现统计
const positiveGames = computed(() => performanceData.value.filter(v => v > 0).length)
const negativeGames = computed(() => performanceData.value.filter(v => v < 0).length)
const highlightGames = computed(() => performanceData.value.filter(v => v >= 10).length)
const positiveRate = computed(() => {
  const total = performanceData.value.length
  if (total === 0) return 0
  return Math.round((positiveGames.value / total) * 100)
})

// 影响力分布图配置
const distributionChartOption = computed(() => {
  const data = performanceData.value
  // 分组统计：<-10, -10~-5, -5~0, 0~5, 5~10, >10
  const ranges = [
    { label: '<-10', min: -Infinity, max: -10, color: '#ef4444' },
    { label: '-10~-5', min: -10, max: -5, color: '#f97316' },
    { label: '-5~0', min: -5, max: 0, color: '#fbbf24' },
    { label: '0~5', min: 0, max: 5, color: '#a3e635' },
    { label: '5~10', min: 5, max: 10, color: '#22c55e' },
    { label: '>10', min: 10, max: Infinity, color: '#10b981' }
  ]

  const counts = ranges.map(r => ({
    name: r.label,
    value: data.filter(v => v >= r.min && v < r.max).length,
    itemStyle: { color: r.color }
  }))

  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: any) => `${params[0].name}<br/>场次: <b>${params[0].value}</b>`
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '8%',
      top: '8%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: ranges.map(r => r.label),
      axisLabel: { color: '#6b7280', fontSize: 11 },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    yAxis: {
      type: 'value',
      axisLabel: { color: '#9ca3af' },
      splitLine: { lineStyle: { color: '#f3f4f6' } }
    },
    series: [{
      type: 'bar',
      data: counts,
      barWidth: '60%',
      itemStyle: { borderRadius: [4, 4, 0, 0] }
    }]
  }
})

// 稳定性仪表盘配置
const gaugeChartOption = computed(() => {
  const consistency = playerStats.value?.consistencyScore || 0
  // 稳定性越高越好，范围 0-100
  const normalizedValue = Math.min(100, Math.max(0, consistency))

  return {
    series: [{
      type: 'gauge',
      startAngle: 200,
      endAngle: -20,
      min: 0,
      max: 100,
      splitNumber: 5,
      itemStyle: {
        color: {
          type: 'linear',
          x: 0, y: 0, x2: 1, y2: 0,
          colorStops: [
            { offset: 0, color: '#ef4444' },
            { offset: 0.5, color: '#f59e0b' },
            { offset: 1, color: '#22c55e' }
          ]
        }
      },
      progress: {
        show: true,
        width: 20
      },
      pointer: { show: false },
      axisLine: {
        lineStyle: { width: 20, color: [[1, '#e5e7eb']] }
      },
      axisTick: { show: false },
      splitLine: { show: false },
      axisLabel: { show: false },
      title: {
        offsetCenter: [0, '20%'],
        fontSize: 14,
        color: '#6b7280'
      },
      detail: {
        offsetCenter: [0, '-10%'],
        fontSize: 32,
        fontWeight: 'bold',
        color: normalizedValue >= 70 ? '#22c55e' : normalizedValue >= 40 ? '#f59e0b' : '#ef4444',
        formatter: (value: number) => value.toFixed(0)
      },
      data: [{ value: normalizedValue, name: '稳定性' }]
    }]
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
  const numId = Number(teamId)
  return teamsMap.value.get(numId) || String(teamId)
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

// 满意度相关
const getSatisfactionColor = (satisfaction: number): string => {
  if (satisfaction >= 70) return '#22c55e'
  if (satisfaction >= 50) return '#f59e0b'
  if (satisfaction >= 30) return '#f97316'
  return '#ef4444'
}

const getSatisfactionClass = (satisfaction: number): string => {
  if (satisfaction >= 70) return 'satisfaction-high'
  if (satisfaction >= 50) return 'satisfaction-medium'
  if (satisfaction >= 30) return 'satisfaction-low'
  return 'satisfaction-danger'
}

const getSatisfactionDesc = (satisfaction: number): string => {
  if (satisfaction >= 80) return '非常满意，对球队忠心耿耿'
  if (satisfaction >= 60) return '比较满意，愿意继续效力'
  if (satisfaction >= 40) return '一般，可能考虑其他机会'
  if (satisfaction >= 30) return '不太满意，有离队倾向'
  return '非常不满，很可能申请转会'
}

// 忠诚度相关
const getLoyaltyColor = (loyalty: number): string => {
  if (loyalty >= 80) return '#8b5cf6'
  if (loyalty >= 60) return '#3b82f6'
  if (loyalty >= 40) return '#22c55e'
  if (loyalty >= 20) return '#f59e0b'
  return '#ef4444'
}

const getLoyaltyTypeName = (loyalty: number): string => {
  if (loyalty >= 80) return '忠心耿耿'
  if (loyalty >= 60) return '忠诚'
  if (loyalty >= 40) return '中立'
  if (loyalty >= 20) return '机会主义'
  return '雇佣兵'
}

const getLoyaltyTagType = (loyalty: number): string => {
  if (loyalty >= 80) return 'success'
  if (loyalty >= 60) return 'primary'
  if (loyalty >= 40) return 'info'
  if (loyalty >= 20) return 'warning'
  return 'danger'
}

const getLoyaltyDesc = (loyalty: number): string => {
  if (loyalty >= 80) return '对球队有深厚感情，很难被挖走'
  if (loyalty >= 60) return '对球队有归属感，不轻易离开'
  if (loyalty >= 40) return '职业态度，会考虑各种机会'
  if (loyalty >= 20) return '容易被高薪吸引，可能主动寻求转会'
  return '纯粹看重报酬，随时可能离开'
}

// 初始化
onMounted(async () => {
  // 从 URL 参数获取赛季（支持 S1 和 1 两种格式）
  const seasonParam = route.query.season as string
  if (seasonParam) {
    // 统一转换为 S1 格式
    selectedSeason.value = seasonParam.startsWith('S') ? seasonParam : `S${seasonParam}`
  }

  // 加载战队数据（用于显示战队名称）
  try {
    if (teamsMap.value.size === 0) {
      const teams = await teamApi.getAllTeams()
      teams.forEach(t => {
        teamsMap.value.set(t.id, t.short_name || t.name)
      })
    }
  } catch (e) {
    logger.warn('加载战队数据失败:', e)
  }

  // 加载选手数据
  await fetchPlayerStats()
})

// 监听赛季变化
watch(selectedSeason, async () => {
  // 更新 URL
  router.replace({
    query: { ...route.query, season: selectedSeason.value }
  })
  // 重新获取选手数据
  await fetchPlayerStats()
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

  // 选手状态卡片
  .status-card {
    margin-bottom: 24px;
    border-radius: 12px;

    :deep(.el-card__header) {
      padding: 16px 20px;
      background: linear-gradient(135deg, #f0f9ff, #e0f2fe);
      border-bottom: 1px solid #bae6fd;
    }

    .status-header {
      .status-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 16px;
        font-weight: 600;
        color: #0369a1;
      }
    }

    .status-content {
      padding: 8px 0;
    }

    .status-item {
      margin-bottom: 20px;

      &:last-child {
        margin-bottom: 0;
      }

      .status-label {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 8px;

        .label-text {
          font-size: 14px;
          font-weight: 500;
          color: #374151;
        }

        .status-value {
          font-size: 18px;
          font-weight: 700;

          &.satisfaction-high { color: #22c55e; }
          &.satisfaction-medium { color: #f59e0b; }
          &.satisfaction-low { color: #f97316; }
          &.satisfaction-danger { color: #ef4444; }
        }
      }

      .status-desc {
        margin-top: 6px;
        font-size: 12px;
        color: #6b7280;
      }
    }

    .departure-alert {
      margin-top: 16px;
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

  // 数据分析图表行
  .charts-row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 16px;
    margin-bottom: 24px;

    .card-title {
      font-size: 16px;
      font-weight: 600;
      color: #1f2937;
    }

    .chart-container.small {
      height: 200px;

      .chart {
        width: 100%;
        height: 100%;
      }
    }
  }

  .distribution-card,
  .gauge-card {
    :deep(.el-card__header) {
      padding: 12px 16px;
      border-bottom: 1px solid #f3f4f6;
    }

    :deep(.el-card__body) {
      padding: 16px;
    }
  }

  .summary-card {
    :deep(.el-card__header) {
      padding: 12px 16px;
      border-bottom: 1px solid #f3f4f6;
    }

    :deep(.el-card__body) {
      padding: 16px;
    }

    .summary-stats {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 16px;
    }

    .summary-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px;
      background: #f9fafb;
      border-radius: 8px;

      .summary-icon {
        width: 40px;
        height: 40px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 18px;
        color: white;

        &.positive { background: linear-gradient(135deg, #22c55e, #16a34a); }
        &.negative { background: linear-gradient(135deg, #ef4444, #dc2626); }
        &.highlight { background: linear-gradient(135deg, #f59e0b, #d97706); }
        &.rate { background: linear-gradient(135deg, #3b82f6, #2563eb); }
      }

      .summary-content {
        .summary-value {
          font-size: 20px;
          font-weight: 700;
          color: #1f2937;
        }

        .summary-label {
          font-size: 12px;
          color: #6b7280;
          margin-top: 2px;
        }
      }
    }
  }
}

@media (max-width: 1200px) {
  .player-detail {
    .charts-row {
      grid-template-columns: 1fr 1fr;

      .summary-card {
        grid-column: span 2;
      }
    }
  }
}

@media (max-width: 1024px) {
  .player-detail {
    .stats-cards {
      grid-template-columns: repeat(2, 1fr);
    }

    .charts-row {
      grid-template-columns: 1fr;

      .summary-card {
        grid-column: span 1;
      }
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
