<template>
  <div class="player-detail">
    <!-- 返回按钮 -->
    <div class="back-nav">
      <el-button text @click="goBack">
        <el-icon><ArrowLeft /></el-icon>
        返回上一页
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

    <!-- 雷达图 + 选手状态 并列行 -->
    <!-- 图表折叠区域 -->
    <div class="collapse-toolbar" v-if="playerStats">
      <el-button size="small" text @click="toggleAllCharts">
        <el-icon v-if="activeCharts.length > 0"><FolderOpened /></el-icon>
        <el-icon v-else><Folder /></el-icon>
        {{ activeCharts.length > 0 ? '全部收起' : '全部展开' }}
      </el-button>
    </div>
    <el-collapse v-model="activeCharts" class="charts-collapse" v-if="playerStats">
      <el-collapse-item title="能力画像 & 选手状态" name="radar-status">
        <div class="radar-status-row">
          <!-- 能力雷达图 -->
          <el-card class="radar-card">
            <template #header>
              <span class="card-title">能力画像</span>
            </template>
            <div class="chart-container small">
              <v-chart v-if="radarChartOption.series" class="chart" :option="radarChartOption" autoresize />
              <el-empty v-else description="暂无数据" :image-size="80" />
            </div>
            <div class="radar-labels" v-if="radarDimLabels.length > 0">
              <span class="radar-label-item" v-for="dim in radarDimLabels" :key="dim.name">
                {{ dim.name }}: <b>{{ dim.value }}</b>
              </span>
            </div>
          </el-card>

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
        </div>
      </el-collapse-item>

      <el-collapse-item title="影响力走势" name="impact-trend">
        <div class="collapse-chart-wrapper">
          <div class="chart-header-inline">
            <el-segmented v-model="impactRange" :options="impactRangeOptions" size="small" />
            <div class="chart-legend" v-if="performanceData.length > 0">
              <span class="legend-item">
                <span class="legend-dot avg"></span>
                平均影响力 {{ (playerStats.avgImpact || 0).toFixed(1) }}
              </span>
            </div>
          </div>
          <div class="chart-container">
            <v-chart v-if="performanceData.length > 0" class="chart" :option="chartOption" autoresize />
            <el-empty v-else description="暂无比赛数据" :image-size="100" />
          </div>
        </div>
      </el-collapse-item>

      <el-collapse-item title="数据分析" name="data-analysis">
        <div class="charts-row-2col">
          <!-- 身价走势 -->
          <el-card class="market-value-card">
            <template #header>
              <span class="card-title">身价走势</span>
            </template>
            <div class="chart-container small">
              <v-chart v-if="marketValueHistory.length > 0 && marketValueChartOption.series" class="chart" :option="marketValueChartOption" autoresize />
              <el-empty v-else description="暂无身价数据" :image-size="80" />
            </div>
          </el-card>

          <!-- 影响力分布 -->
          <el-card class="distribution-card" v-if="performanceData.length > 0">
            <template #header>
              <span class="card-title">影响力分布</span>
            </template>
            <div class="chart-container small">
              <v-chart class="chart" :option="distributionChartOption" autoresize />
            </div>
          </el-card>

          <!-- 稳定性仪表盘 -->
          <el-card class="gauge-card" v-if="performanceData.length > 0">
            <template #header>
              <span class="card-title">稳定性评分</span>
            </template>
            <div class="chart-container small">
              <v-chart class="chart" :option="gaugeChartOption" autoresize />
            </div>
          </el-card>

          <!-- 表现统计 -->
          <el-card class="summary-card" v-if="performanceData.length > 0">
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
      </el-collapse-item>

      <el-collapse-item title="同位置对比 & 赛事表现" name="position-tournament" v-if="positionCompareOption.series || tournamentHistory.length > 0">
        <div class="charts-row-2col">
          <el-card class="position-compare-card" v-if="positionCompareOption.series">
            <template #header>
              <span class="card-title">同位置对比</span>
            </template>
            <div class="chart-container medium">
              <v-chart class="chart" :option="positionCompareOption" autoresize />
            </div>
          </el-card>

          <el-card class="tournament-card" v-if="tournamentHistory.length > 0">
            <template #header>
              <span class="card-title">赛事表现</span>
            </template>
            <div class="chart-container medium">
              <v-chart class="chart" :option="tournamentChartOption" autoresize />
            </div>
          </el-card>
        </div>

        <!-- 赛事表现明细卡片 -->
        <div class="tournament-breakdown" v-if="tournamentHistory.length > 0">
          <div class="breakdown-header">
            <span class="breakdown-title">各赛事表现明细</span>
            <span class="big-stage-badge" :class="bigStageScore >= 0 ? 'positive' : 'negative'" v-if="hasInternational">
              大赛影响力 {{ bigStageScore >= 0 ? '+' : '' }}{{ bigStageScore.toFixed(1) }}
            </span>
            <span class="big-stage-badge no-intl" v-else>未参加国际赛</span>
          </div>
          <div class="breakdown-grid">
            <div class="tournament-detail-item" v-for="td in tournamentHistory" :key="td.tournament_type">
              <div class="td-header">
                <span class="td-name">{{ tournamentTypeNames[td.tournament_type] || td.tournament_type }}</span>
                <span class="td-weight" :class="getTournamentWeightClass(td.tournament_type)">×{{ getTournamentWeight(td.tournament_type).toFixed(1) }}</span>
              </div>
              <div class="td-stats">
                <div class="td-stat">
                  <span class="td-label">场次</span>
                  <span class="td-value">{{ td.games_played }}</span>
                </div>
                <div class="td-stat">
                  <span class="td-label">影响力</span>
                  <span class="td-value" :class="{ 'positive-val': td.avg_impact > 0, 'negative-val': td.avg_impact < 0 }">
                    {{ td.avg_impact > 0 ? '+' : '' }}{{ td.avg_impact.toFixed(1) }}
                  </span>
                </div>
                <div class="td-stat">
                  <span class="td-label">发挥</span>
                  <span class="td-value">{{ td.avg_performance.toFixed(1) }}</span>
                </div>
                <div class="td-stat">
                  <span class="td-label">巅峰</span>
                  <span class="td-value">{{ td.max_impact.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>

      <el-collapse-item title="职业生涯能力走势" name="career-ability" v-if="seasonHistory.length > 1">
        <div class="chart-container">
          <v-chart class="chart" :option="careerAbilityOption" autoresize />
        </div>
      </el-collapse-item>

      <el-collapse-item title="年度Top排名走势" name="yearly-top" v-if="yearlyTopHistory.length > 0">
        <div class="chart-container">
          <v-chart class="chart" :option="yearlyTopRankOption" autoresize />
        </div>
      </el-collapse-item>
    </el-collapse>

    <!-- 无数据提示 -->
    <el-empty v-if="!playerStats" description="暂无该选手数据" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, VideoCamera, TrendCharts, Aim, Star, Top, Bottom, StarFilled, Promotion, User, Folder, FolderOpened } from '@element-plus/icons-vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart, GaugeChart, RadarChart, BoxplotChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  MarkLineComponent,
  LegendComponent,
  RadarComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { teamApi, statsApi, playerApi } from '@/api/tauri'
import type { PlayerPosition, PlayerSeasonStats } from '@/types/player'
import type { PlayerFullDetail, MarketValueChange, PlayerSeasonHistoryEntry, PlayerTournamentHistoryItem, PlayerYearlyTopItem, PlayerRankingItem } from '@/api/tauri'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'

const logger = createLogger('DataCenterPlayerDetail')

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  GaugeChart,
  RadarChart,
  BoxplotChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  MarkLineComponent,
  LegendComponent,
  RadarComponent
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
const impactHistory = ref<number[]>([])
const playerFullDetail = ref<PlayerFullDetail | null>(null)
const marketValueHistory = ref<MarketValueChange[]>([])
const positionRankings = ref<PlayerRankingItem[]>([])
const tournamentHistory = ref<PlayerTournamentHistoryItem[]>([])
const seasonHistory = ref<PlayerSeasonHistoryEntry[]>([])
const yearlyTopHistory = ref<PlayerYearlyTopItem[]>([])
const impactRange = ref('全部')
const activeCharts = ref<string[]>([])
const allChartNames = ['radar-status', 'impact-trend', 'data-analysis', 'position-tournament', 'career-ability', 'yearly-top']
const toggleAllCharts = () => {
  activeCharts.value = activeCharts.value.length > 0 ? [] : [...allChartNames]
}
const impactRangeOptions = [
  { label: '近10场', value: '近10场' },
  { label: '近20场', value: '近20场' },
  { label: '近50场', value: '近50场' },
  { label: '全部', value: '全部' },
]

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

  // 获取身价变化历史
  try {
    const history = await statsApi.getPlayerMarketValueChanges(playerIdNum)
    marketValueHistory.value = history || []
    logger.debug('[DataCenterPlayerDetail] Market value history:', history?.length || 0, '条记录')
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取身价变化历史失败:', error)
    marketValueHistory.value = []
  }

  // 获取同位置排行数据
  try {
    const seasonNum = Number(selectedSeason.value.replace('S', ''))
    const position = playerStatsData.value?.position
    if (position) {
      const rankings = await statsApi.getPositionRanking(seasonNum, position, 10)
      positionRankings.value = rankings || []
    }
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取位置排行失败:', error)
    positionRankings.value = []
  }

  // 获取赛事表现历史
  try {
    const seasonNum = Number(selectedSeason.value.replace('S', ''))
    const history = await statsApi.getPlayerTournamentHistory(playerIdNum, seasonNum)
    tournamentHistory.value = history || []
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取赛事表现历史失败:', error)
    tournamentHistory.value = []
  }

  // 获取职业生涯历史
  try {
    const history = await statsApi.getPlayerSeasonHistory(playerIdNum)
    seasonHistory.value = history || []
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取赛季历史失败:', error)
    seasonHistory.value = []
  }

  // 获取年度Top排名历史
  try {
    const history = await statsApi.getPlayerYearlyTopHistory(playerIdNum)
    yearlyTopHistory.value = history || []
  } catch (error) {
    logger.error('[DataCenterPlayerDetail] 获取年度Top历史失败:', error)
    yearlyTopHistory.value = []
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

const INTL_TYPES = new Set([
  'Msi', 'MadridMasters', 'ClaudeIntercontinental', 'WorldChampionship',
  'ShanghaiMasters', 'IcpIntercontinental', 'SuperIntercontinental'
])

const bigStageScore = computed(() => {
  const history = tournamentHistory.value
  if (!history || history.length === 0) return 0
  let intlImpactSum = 0
  let intlGames = 0
  for (const t of history) {
    if (INTL_TYPES.has(t.tournament_type)) {
      intlImpactSum += t.avg_impact * t.games_played
      intlGames += t.games_played
    }
  }
  if (intlGames <= 0) return 0
  const rawScore = intlImpactSum / intlGames
  const confidence = Math.min(1.0, intlGames / 70)
  return rawScore * confidence
})

const hasInternational = computed(() => {
  return tournamentHistory.value.some(t => INTL_TYPES.has(t.tournament_type))
})

// 赛事权重映射（和 awards_commands.rs tournament_type_weight 一致）
const tournamentWeights: Record<string, number> = {
  'WorldChampionship': 1.5,
  'SuperIntercontinental': 1.4,
  'Msi': 1.3,
  'ClaudeIntercontinental': 1.2,
  'IcpIntercontinental': 1.2,
  'MadridMasters': 1.1,
  'ShanghaiMasters': 1.1,
  'SpringPlayoffs': 1.05,
  'SummerPlayoffs': 1.05,
}

// 赛事类型中文名（复用 tournamentChartOption 里的映射）
const tournamentTypeNames: Record<string, string> = {
  'SpringRegular': '春季常规赛',
  'SpringPlayoffs': '春季季后赛',
  'Msi': 'MSI季中赛',
  'MadridMasters': '马德里大师赛',
  'SummerRegular': '夏季常规赛',
  'SummerPlayoffs': '夏季季后赛',
  'ClaudeIntercontinental': 'Claude洲际赛',
  'WorldChampionship': '世界赛',
  'ShanghaiMasters': '上海大师赛',
  'IcpIntercontinental': 'ICP洲际赛',
  'SuperIntercontinental': 'Super洲际赛',
}

const getTournamentWeight = (type: string): number => {
  return tournamentWeights[type] ?? 0.9
}

const getTournamentWeightClass = (type: string): string => {
  const w = getTournamentWeight(type)
  if (w >= 1.3) return 'weight-high'
  if (w >= 1.0) return 'weight-mid'
  return 'weight-low'
}

// ECharts 配置
const chartOption = computed(() => {
  const allData = performanceData.value
  const rangeMap: Record<string, number> = { '近10场': 10, '近20场': 20, '近50场': 50 }
  const limit = rangeMap[impactRange.value]
  const data = limit ? allData.slice(-limit) : allData
  const offset = limit ? Math.max(0, allData.length - limit) : 0
  const avg = playerStats.value?.avgImpact || 0

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const value = params[0]?.value || 0
        const gameNum = params[0]?.dataIndex + 1 + offset
        return `第 ${gameNum} 场<br/>影响力: <b>${value}</b>`
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
      data: data.map((_, i) => i + 1 + offset),
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

// 能力雷达图配置
const radarChartOption = computed(() => {
  const stats = playerStats.value
  if (!stats) return {}

  const impactNorm = Math.min(100, Math.max(0, (stats.avgImpact + 5) * 5))
  const consistencyNorm = Math.min(100, Math.max(0, stats.consistencyScore || 0))
  const gamesNorm = Math.min(100, Math.max(0, (stats.gamesPlayed || 0) * 0.83))
  const championNorm = Math.min(100, Math.max(0, (stats.championBonus || 0) * 6.67))
  const performanceNorm = Math.min(100, Math.max(0, ((stats.avgPerformance || 50) - 50) * 2))
  const bigStageNorm = hasInternational.value
    ? Math.min(100, Math.max(0, (bigStageScore.value + 5) * 5))
    : 0

  return {
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
        color: '#6b7280',
        fontSize: 12
      },
      splitLine: {
        lineStyle: { color: '#e5e7eb' }
      },
      splitArea: {
        areaStyle: {
          color: ['rgba(59, 130, 246, 0.02)', 'rgba(59, 130, 246, 0.05)']
        }
      },
      axisLine: {
        lineStyle: { color: '#e5e7eb' }
      }
    },
    series: [{
      type: 'radar',
      data: [{
        value: [impactNorm, performanceNorm, bigStageNorm, consistencyNorm, gamesNorm, championNorm],
        areaStyle: {
          color: 'rgba(59, 130, 246, 0.2)'
        },
        lineStyle: {
          color: '#3b82f6',
          width: 2
        },
        itemStyle: {
          color: '#3b82f6'
        }
      }]
    }]
  }
})

// 雷达图维度实际数值标签
const radarDimLabels = computed(() => {
  const stats = playerStats.value
  if (!stats) return []
  const bsLabel = hasInternational.value
    ? `${bigStageScore.value >= 0 ? '+' : ''}${bigStageScore.value.toFixed(1)}`
    : 'N/A'
  return [
    { name: '影响力', value: `${stats.avgImpact >= 0 ? '+' : ''}${(stats.avgImpact || 0).toFixed(1)}` },
    { name: '发挥', value: `${(stats.avgPerformance || 0).toFixed(1)}` },
    { name: '大赛', value: bsLabel },
    { name: '稳定性', value: `${(stats.consistencyScore || 0).toFixed(0)}` },
    { name: '出场', value: `${stats.gamesPlayed || 0}场` },
    { name: '荣誉', value: `${(stats.championBonus || 0).toFixed(1)}` }
  ]
})

// 身价走势图配置
const marketValueChartOption = computed(() => {
  const history = marketValueHistory.value
  if (!history || history.length === 0) return {}

  // 按赛季排序，取每赛季最后一条记录的 new_value
  const seasonMap = new Map<number, number>()
  history.forEach(h => {
    seasonMap.set(h.season_id, h.new_value)
  })
  const seasons = Array.from(seasonMap.keys()).sort((a, b) => a - b)
  const values = seasons.map(s => seasonMap.get(s)! / 10000) // 转为万元

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const value = params[0]?.value || 0
        return `S${params[0]?.name}<br/>身价: <b>${value.toFixed(0)}万元</b>`
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
      data: seasons.map(s => `${s}`),
      axisLabel: {
        color: '#9ca3af',
        formatter: (value: string) => `S${value}`
      },
      axisLine: {
        lineStyle: { color: '#e5e7eb' }
      }
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: '#9ca3af',
        formatter: (value: number) => `${value}万`
      },
      splitLine: {
        lineStyle: { color: '#f3f4f6' }
      }
    },
    series: [{
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 8,
      data: values,
      lineStyle: {
        width: 3,
        color: '#f59e0b'
      },
      itemStyle: {
        color: '#f59e0b',
        borderWidth: 2,
        borderColor: '#fff'
      },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(245, 158, 11, 0.3)' },
            { offset: 1, color: 'rgba(245, 158, 11, 0.05)' }
          ]
        }
      }
    }]
  }
})

// 同位置对比雷达图
const positionCompareOption = computed(() => {
  const stats = playerStats.value
  if (!stats || positionRankings.value.length === 0) return {}

  const playerIdNum = Number(playerId.value)
  const top3 = positionRankings.value.slice(0, 3)
  const currentPlayer = positionRankings.value.find(r => r.player_id === playerIdNum)

  const players = [...top3]
  if (currentPlayer && !top3.find(p => p.player_id === playerIdNum)) {
    players.push(currentPlayer)
  }

  const colors = ['#3b82f6', '#22c55e', '#f59e0b', '#ef4444', '#8b5cf6']
  const indicators = [
    { name: '影响力', max: 100 },
    { name: '发挥', max: 100 },
    { name: '大赛', max: 100 },
    { name: '稳定性', max: 100 },
    { name: '出场', max: 100 },
    { name: '荣誉', max: 100 }
  ]

  const seriesData = players.map((p, i) => {
    const impactNorm = Math.min(100, Math.max(0, (p.avg_impact + 5) * 5))
    const consistencyNorm = Math.min(100, Math.max(0, p.consistency_score || 0))
    const gamesNorm = Math.min(100, Math.max(0, (p.games_played || 0) * 0.83))
    const championNorm = Math.min(100, Math.max(0, (p.champion_bonus || 0) * 6.67))
    const performanceNorm = Math.min(100, Math.max(0, ((p.avg_performance || 50) - 50) * 2))
    const bigStageNorm = p.has_international
      ? Math.min(100, Math.max(0, (p.big_stage_score + 5) * 5))
      : 0

    return {
      value: [impactNorm, performanceNorm, bigStageNorm, consistencyNorm, gamesNorm, championNorm],
      name: p.player_id === playerIdNum ? `${p.player_name} (本人)` : p.player_name,
      lineStyle: {
        width: p.player_id === playerIdNum ? 3 : 1.5,
        color: colors[i],
        type: p.player_id === playerIdNum ? 'solid' as const : 'dashed' as const
      },
      areaStyle: {
        color: p.player_id === playerIdNum ? `${colors[i]}33` : 'transparent'
      },
      itemStyle: { color: colors[i] }
    }
  })

  return {
    legend: {
      data: seriesData.map(d => d.name),
      bottom: 0,
      textStyle: { fontSize: 11, color: '#6b7280' }
    },
    radar: {
      indicator: indicators,
      shape: 'polygon' as const,
      splitNumber: 4,
      radius: '60%',
      axisName: { color: '#6b7280', fontSize: 11 },
      splitLine: { lineStyle: { color: '#e5e7eb' } },
      splitArea: { areaStyle: { color: ['rgba(59,130,246,0.02)', 'rgba(59,130,246,0.05)'] } },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    series: [{
      type: 'radar',
      data: seriesData
    }]
  }
})

// 赛事表现箱线图（用柱状图模拟：每赛事的 avg_impact + max_impact 范围）
const tournamentChartOption = computed(() => {
  const data = tournamentHistory.value
  if (!data || data.length === 0) return {}

  const typeNames: Record<string, string> = {
    'SpringRegular': '春季常规赛',
    'SpringPlayoffs': '春季季后赛',
    'Msi': 'MSI季中赛',
    'MadridMasters': '马德里大师赛',
    'SummerRegular': '夏季常规赛',
    'SummerPlayoffs': '夏季季后赛',
    'ClaudeIntercontinental': 'Claude洲际赛',
    'WorldChampionship': '世界赛',
    'ShanghaiMasters': '上海大师赛',
    'IcpIntercontinental': 'ICP洲际赛',
    'SuperIntercontinental': 'Super洲际赛',
  }

  const categories = data.map(d => typeNames[d.tournament_type] || d.tournament_type)
  const avgData = data.map(d => Math.round(d.avg_impact * 10) / 10)
  const maxData = data.map(d => Math.round(d.max_impact * 10) / 10)
  const gamesData = data.map(d => d.games_played)

  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: any) => {
        const idx = params[0]?.dataIndex ?? 0
        return `${categories[idx]}<br/>
          平均影响力: <b>${avgData[idx]}</b><br/>
          最高影响力: <b>${maxData[idx]}</b><br/>
          参与局数: <b>${gamesData[idx]}</b>`
      }
    },
    grid: { left: '3%', right: '4%', bottom: '15%', top: '10%', containLabel: true },
    xAxis: {
      type: 'category',
      data: categories,
      axisLabel: { color: '#6b7280', fontSize: 10, rotate: 30 },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    yAxis: {
      type: 'value',
      axisLabel: { color: '#9ca3af' },
      splitLine: { lineStyle: { color: '#f3f4f6' } }
    },
    series: [
      {
        name: '平均影响力',
        type: 'bar',
        data: avgData,
        barWidth: '35%',
        itemStyle: {
          borderRadius: [4, 4, 0, 0],
          color: {
            type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: '#60a5fa' },
              { offset: 1, color: '#3b82f6' }
            ]
          }
        }
      },
      {
        name: '最高影响力',
        type: 'bar',
        data: maxData,
        barWidth: '35%',
        itemStyle: {
          borderRadius: [4, 4, 0, 0],
          color: {
            type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: '#34d399' },
              { offset: 1, color: '#10b981' }
            ]
          }
        }
      }
    ]
  }
})

// 职业生涯能力走势折线图
const careerAbilityOption = computed(() => {
  const data = seasonHistory.value
  if (!data || data.length === 0) return {}

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const idx = params[0]?.dataIndex ?? 0
        const entry = data[idx]
        return `${entry.season}<br/>
          战队: <b>${entry.team_name}</b><br/>
          能力: <b>${entry.ability}</b><br/>
          潜力: <b>${entry.potential}</b>`
      }
    },
    legend: {
      data: ['能力', '潜力'],
      bottom: 0,
      textStyle: { fontSize: 11, color: '#6b7280' }
    },
    grid: { left: '3%', right: '4%', bottom: '12%', top: '10%', containLabel: true },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: data.map(d => d.season),
      axisLabel: { color: '#9ca3af' },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    yAxis: {
      type: 'value',
      min: (value: { min: number }) => Math.max(0, Math.floor(value.min / 10) * 10 - 5),
      max: 100,
      axisLabel: { color: '#9ca3af' },
      splitLine: { lineStyle: { color: '#f3f4f6' } }
    },
    series: [
      {
        name: '能力',
        type: 'line',
        smooth: true,
        symbol: 'circle',
        symbolSize: 8,
        data: data.map(d => d.ability),
        lineStyle: { width: 3, color: '#3b82f6' },
        itemStyle: { color: '#3b82f6', borderWidth: 2, borderColor: '#fff' },
        areaStyle: {
          color: {
            type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(59,130,246,0.2)' },
              { offset: 1, color: 'rgba(59,130,246,0.02)' }
            ]
          }
        }
      },
      {
        name: '潜力',
        type: 'line',
        smooth: true,
        symbol: 'diamond',
        symbolSize: 6,
        data: data.map(d => d.potential),
        lineStyle: { width: 2, color: '#8b5cf6', type: 'dashed' },
        itemStyle: { color: '#8b5cf6', borderWidth: 2, borderColor: '#fff' }
      }
    ]
  }
})

// 年度Top排名走势折线图
const yearlyTopRankOption = computed(() => {
  const data = yearlyTopHistory.value
  if (!data || data.length === 0) return {}

  const rankData = data.map(d => d.rank <= 20 ? d.rank : null)
  const hasOffChart = data.some(d => d.rank > 20)

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const idx = params[0]?.dataIndex ?? 0
        const entry = data[idx]
        const rankStr = entry.rank <= 20 ? `#${entry.rank}` : '20+（未上榜）'
        return `${entry.season}<br/>
          排名: <b>${rankStr}</b><br/>
          得分: <b>${entry.yearly_top_score.toFixed(1)}</b><br/>
          参与人数: ${entry.total_players}`
      }
    },
    grid: { left: '3%', right: '4%', bottom: '8%', top: '10%', containLabel: true },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: data.map(d => d.season),
      axisLabel: { color: '#9ca3af' },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    yAxis: {
      type: 'value',
      inverse: true,
      min: 1,
      max: hasOffChart ? 25 : 20,
      axisLabel: {
        color: '#9ca3af',
        formatter: (value: number) => value > 20 ? '20+' : `#${value}`
      },
      splitLine: { lineStyle: { color: '#f3f4f6' } }
    },
    series: [{
      type: 'line',
      smooth: true,
      symbol: 'circle',
      symbolSize: 10,
      connectNulls: false,
      data: rankData.map((r) => {
        if (r === null) {
          return {
            value: 22,
            itemStyle: { color: '#d1d5db', borderColor: '#9ca3af' },
            label: { show: true, formatter: '20+', color: '#9ca3af', fontSize: 10 }
          }
        }
        return {
          value: r,
          itemStyle: {
            color: r <= 3 ? '#f59e0b' : r <= 10 ? '#3b82f6' : '#6b7280',
            borderWidth: 2,
            borderColor: '#fff'
          },
          label: {
            show: true,
            formatter: `#${r}`,
            color: r <= 3 ? '#f59e0b' : r <= 10 ? '#3b82f6' : '#6b7280',
            fontSize: 11,
            fontWeight: 'bold',
            position: 'top'
          }
        }
      }),
      lineStyle: {
        width: 3,
        color: {
          type: 'linear', x: 0, y: 0, x2: 1, y2: 0,
          colorStops: [
            { offset: 0, color: '#f59e0b' },
            { offset: 1, color: '#3b82f6' }
          ]
        }
      },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(245,158,11,0.15)' },
            { offset: 1, color: 'rgba(59,130,246,0.05)' }
          ]
        }
      }
    }]
  }
})

const goBack = () => {
  router.back()
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

  .collapse-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 8px;
  }

  .charts-collapse {
    border: none;
    margin-bottom: 24px;

    :deep(.el-collapse-item__header) {
      font-size: 16px;
      font-weight: 600;
      color: #1f2937;
      background: #f9fafb;
      border-radius: 8px;
      padding: 12px 16px;
      margin-bottom: 4px;
      border: none;
      height: auto;
      line-height: 1.5;
    }

    :deep(.el-collapse-item__wrap) {
      border: none;
    }

    :deep(.el-collapse-item__content) {
      padding: 16px 0;
    }

    :deep(.el-collapse-item) {
      border: none;
      margin-bottom: 8px;
    }

    .chart-container {
      height: 300px;

      .chart {
        width: 100%;
        height: 100%;
      }
    }
  }

  .collapse-chart-wrapper {
    .chart-header-inline {
      display: flex;
      align-items: center;
      gap: 16px;
      margin-bottom: 12px;

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

  // 雷达图 + 选手状态 并列行
  .radar-status-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 24px;
  }

  // 雷达图卡片
  .radar-card {
    :deep(.el-card__header) {
      padding: 12px 16px;
      border-bottom: 1px solid #f3f4f6;
    }

    :deep(.el-card__body) {
      padding: 16px;
    }

    .card-title {
      font-size: 16px;
      font-weight: 600;
      color: #1f2937;
    }

    .chart-container.small {
      height: 240px;

      .chart {
        width: 100%;
        height: 100%;
      }
    }

    .radar-labels {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      justify-content: center;
      padding-top: 8px;
      border-top: 1px solid #f3f4f6;
      margin-top: 8px;

      .radar-label-item {
        font-size: 12px;
        color: #6b7280;
        background: #f9fafb;
        padding: 4px 10px;
        border-radius: 12px;

        b {
          color: #1f2937;
        }
      }
    }
  }

  // 选手状态卡片
  .status-card {
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

      .chart-header-right {
        display: flex;
        align-items: center;
        gap: 16px;
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

  // 数据分析图表行（2列布局）
  .charts-row-2col {
    display: grid;
    grid-template-columns: 1fr 1fr;
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

  .market-value-card,
  .distribution-card,
  .gauge-card,
  .position-compare-card,
  .tournament-card {
    :deep(.el-card__header) {
      padding: 12px 16px;
      border-bottom: 1px solid #f3f4f6;
    }

    :deep(.el-card__body) {
      padding: 16px;
    }
  }

  .chart-container.medium {
    height: 280px;

    .chart {
      width: 100%;
      height: 100%;
    }
  }

  // 赛事表现明细卡片
  .tournament-breakdown {
    margin-top: 16px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 12px;

    .breakdown-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 12px;

      .breakdown-title {
        font-size: 14px;
        font-weight: 600;
        color: #374151;
      }

      .big-stage-badge {
        font-size: 12px;
        font-weight: 700;
        padding: 3px 10px;
        border-radius: 6px;
        &.positive { background: #d1fae5; color: #059669; }
        &.negative { background: #fee2e2; color: #dc2626; }
        &.no-intl { background: #fef3c7; color: #d97706; }
      }
    }

    .breakdown-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
      gap: 10px;
    }

    .tournament-detail-item {
      padding: 12px;
      background: white;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      .td-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;

        .td-name { font-size: 13px; font-weight: 600; color: #1f2937; }
        .td-weight {
          font-size: 11px; font-weight: 700; padding: 2px 6px; border-radius: 4px;
          &.weight-high { background: #fef3c7; color: #d97706; }
          &.weight-mid { background: #e0e7ff; color: #4f46e5; }
          &.weight-low { background: #f3f4f6; color: #6b7280; }
        }
      }

      .td-stats {
        display: flex; flex-wrap: wrap; gap: 10px;
        .td-stat {
          display: flex; flex-direction: column;
          .td-label { font-size: 11px; color: #9ca3af; }
          .td-value {
            font-size: 13px; font-weight: 600; color: #374151;
            &.positive-val { color: #10b981; }
            &.negative-val { color: #ef4444; }
          }
        }
      }
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
    .radar-status-row {
      grid-template-columns: 1fr;
    }

    .charts-row-2col {
      grid-template-columns: 1fr 1fr;
    }
  }
}

@media (max-width: 1024px) {
  .player-detail {
    .stats-cards {
      grid-template-columns: repeat(2, 1fr);
    }

    .charts-row-2col {
      grid-template-columns: 1fr;
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
