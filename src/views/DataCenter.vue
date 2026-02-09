<template>
  <div class="data-center">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><DataLine /></el-icon>
          数据中心
        </h1>
        <p class="page-description">
          查看选手比赛数据统计与发挥表现
        </p>
      </div>
      <div class="header-actions">
        <SeasonSelector v-model="selectedSeason" />
        <el-button type="primary" @click="refreshData" :loading="loading">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button type="warning" @click="syncData" :loading="loading">
          同步数据
        </el-button>
      </div>
    </div>

    <!-- 概览统计行 -->
    <div class="dashboard-stats" v-if="rankings.length > 0">
      <el-card class="stat-card">
        <div class="stat-icon players">
          <el-icon><User /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ dashboardStats.totalPlayers }}</div>
          <div class="stat-label">参赛选手</div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-icon impact">
          <el-icon><TrendCharts /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value" :class="dashboardStats.avgImpact >= 0 ? 'positive' : 'negative'">
            {{ dashboardStats.avgImpact >= 0 ? '+' : '' }}{{ dashboardStats.avgImpact.toFixed(1) }}
          </div>
          <div class="stat-label">平均影响力</div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-icon top-score">
          <el-icon><Star /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value highlight">{{ dashboardStats.topScore.toFixed(1) }}</div>
          <div class="stat-label">最高得分</div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-icon games">
          <el-icon><VideoCamera /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ dashboardStats.avgGames.toFixed(0) }}</div>
          <div class="stat-label">平均场次</div>
        </div>
      </el-card>
    </div>

    <!-- 位置对比图表 + Top5 侧边栏 -->
    <div class="charts-dashboard-row" v-if="rankings.length > 0">
      <el-card class="position-chart-card">
        <template #header>
          <span class="card-title">各位置对比分析</span>
        </template>
        <div class="chart-container">
          <v-chart class="chart" :option="positionComparisonOption" autoresize />
        </div>
      </el-card>

      <el-card class="top5-card">
        <template #header>
          <span class="card-title">TOP 5 选手</span>
        </template>
        <div class="top5-list">
          <div
            class="top5-item"
            v-for="(player, index) in top5Players"
            :key="player.playerId"
            @click="goToPlayerDetail(player)"
          >
            <div class="top5-rank" :class="getRankClass(index)">{{ index + 1 }}</div>
            <div class="top5-info">
              <div class="top5-name">{{ player.playerName }}</div>
              <div class="top5-meta">
                <el-tag :type="getPositionTagType(player.position)" size="small">
                  {{ getPositionName(player.position) }}
                </el-tag>
                <span class="top5-team">{{ getTeamName(player.teamId) }}</span>
              </div>
            </div>
            <div class="top5-score">{{ (player.yearlyTopScore || player.avgImpact || 0).toFixed(1) }}</div>
          </div>
          <el-empty v-if="top5Players.length === 0" description="暂无数据" :image-size="60" />
        </div>
      </el-card>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <div class="position-filters">
        <el-button
          v-for="pos in positionFilters"
          :key="pos.value"
          :type="selectedPosition === pos.value ? 'primary' : 'default'"
          @click="selectedPosition = pos.value"
          round
        >
          {{ pos.label }}
        </el-button>
      </div>
      <div class="sort-filters">
        <el-select v-model="sortField" placeholder="排序字段" style="width: 120px">
          <el-option label="得分" value="yearlyTopScore" />
          <el-option label="场次" value="gamesPlayed" />
          <el-option label="出场分" value="gamesBonus" />
          <el-option label="影响力" value="avgImpact" />
          <el-option label="冠军分" value="championBonus" />
        </el-select>
        <el-button-group>
          <el-button :type="sortOrder === 'desc' ? 'primary' : 'default'" @click="sortOrder = 'desc'">
            降序
          </el-button>
          <el-button :type="sortOrder === 'asc' ? 'primary' : 'default'" @click="sortOrder = 'asc'">
            升序
          </el-button>
        </el-button-group>
      </div>
      <div class="search-box">
        <el-input
          v-model="searchQuery"
          placeholder="搜索选手..."
          prefix-icon="Search"
          clearable
          style="width: 200px"
        />
      </div>
    </div>

    <!-- 四象限散点图 -->
    <div class="scatter-chart-section" v-if="rankings.length > 0">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header clickable" @click="scatterExpanded = !scatterExpanded">
            <div class="card-header-left">
              <el-icon class="collapse-arrow" :class="{ collapsed: !scatterExpanded }">
                <ArrowDown />
              </el-icon>
              <span class="card-title">选手四象限分析</span>
            </div>
            <div class="scatter-controls" v-show="scatterExpanded" @click.stop>
              <el-button-group size="small">
                <el-button :type="scatterColorMode === 'position' ? 'primary' : ''" @click="scatterColorMode = 'position'">按位置</el-button>
                <el-button :type="scatterColorMode === 'region' ? 'primary' : ''" @click="scatterColorMode = 'region'">按赛区</el-button>
              </el-button-group>
            </div>
          </div>
        </template>
        <el-collapse-transition>
          <div v-show="scatterExpanded">
            <div class="scatter-chart-wrapper">
              <v-chart class="scatter-chart" :option="scatterOption" autoresize @click="onScatterClick" />
            </div>
            <!-- 颜色图例 -->
            <div class="scatter-legend">
              <span
                v-for="item in scatterLegendItems"
                :key="item.label"
                class="legend-item"
              >
                <span class="legend-dot" :style="{ backgroundColor: item.color }"></span>
                {{ item.label }}
              </span>
            </div>
            <!-- 四象限标签 -->
            <div class="quadrant-legend">
              <span class="q q1">核心基石（高影响力+高稳定性）</span>
              <span class="q q2">波动明星（高影响力+低稳定性）</span>
              <span class="q q3">稳定绿叶（低影响力+高稳定性）</span>
              <span class="q q4">待培养（低影响力+低稳定性）</span>
            </div>
          </div>
        </el-collapse-transition>
      </el-card>
    </div>

    <!-- 排行榜表格 -->
    <el-card class="rankings-card">
      <template #header>
        <div class="card-header">
          <span class="card-title">选手数据排行榜</span>
          <el-tag type="info" size="small">
            共 {{ filteredRankings.length }} 名选手
          </el-tag>
        </div>
      </template>

      <el-table
        :data="paginatedRankings"
        stripe
        style="width: 100%"
        :row-class-name="getRankRowClass"
        @row-click="goToPlayerDetail"
        class="clickable-table"
        v-loading="loading"
      >
        <el-table-column label="排名" width="80" align="center">
          <template #default="{ $index }">
            <div class="rank-badge" :class="getRankClass(getRealRank($index) - 1)">
              {{ getRealRank($index) }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="playerName" label="选手" min-width="150">
          <template #default="{ row }">
            <div class="player-cell">
              <span class="player-name">{{ row.playerName }}</span>
              <el-icon class="arrow-icon"><ArrowRight /></el-icon>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="position" label="位置" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getPositionTagType(row.position)" size="small">
              {{ getPositionName(row.position) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="teamId" label="战队" width="120">
          <template #default="{ row }">
            {{ getTeamName(row.teamId) }}
          </template>
        </el-table-column>

        <el-table-column prop="gamesPlayed" label="场次" width="80" align="center" />

        <el-table-column label="出场分" width="80" align="center">
          <template #default="{ row }">
            <span class="games-bonus">{{ ((row.gamesPlayed || 0) / 10).toFixed(1) }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="avgImpact" label="影响力" width="100" align="center">
          <template #default="{ row }">
            <span :class="getImpactClass(row.avgImpact)">
              {{ formatImpact(row.avgImpact) }}
            </span>
          </template>
        </el-table-column>

        <el-table-column label="冠军分" width="80" align="center">
          <template #default="{ row }">
            <span class="champion-bonus">{{ (row.championBonus || 0).toFixed(1) }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="consistencyScore" label="稳定性" width="120" align="center">
          <template #default="{ row }">
            <div class="consistency-cell">
              <el-progress
                :percentage="row.consistencyScore || 0"
                :stroke-width="8"
                :show-text="false"
                :color="getConsistencyColor(row.consistencyScore)"
              />
              <span class="consistency-value">{{ (row.consistencyScore || 0).toFixed(0) }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="yearlyTopScore" label="得分" width="100" align="center">
          <template #default="{ row }">
            <span class="yearly-score" :class="getScoreClass(row.yearlyTopScore || row.avgImpact)">
              {{ (row.yearlyTopScore || row.avgImpact || 0).toFixed(1) }}
            </span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页器 -->
      <div class="pagination-container" v-if="filteredRankings.length > 0">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="filteredRankings.length"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>

      <el-empty v-if="filteredRankings.length === 0" description="暂无统计数据" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { DataLine, Refresh, ArrowRight, ArrowDown, User, VideoCamera, TrendCharts, Star } from '@element-plus/icons-vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart, ScatterChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
  MarkLineComponent,
  MarkAreaComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useSeasonStore } from '@/stores/useSeasonStore'
import { teamApi, devApi } from '@/api/tauri'
import SeasonSelector from '@/components/common/SeasonSelector.vue'
import { ElMessage } from 'element-plus'
import type { PlayerPosition, PlayerSeasonStats } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  BarChart,
  ScatterChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
  MarkLineComponent,
  MarkAreaComponent
])

const logger = createLogger('DataCenter')

const router = useRouter()
const playerStore = usePlayerStore()
const seasonStore = useSeasonStore()

// 本地战队映射表
const teamsMap = ref<Map<number, string>>(new Map())
// 战队→赛区映射表（用 Record 代替 Map，确保 Vue 响应式追踪）
const teamsRegionMap = ref<Record<string, number>>({})

// 状态
const selectedSeason = ref(seasonStore.currentSeason)
const selectedPosition = ref('')
const searchQuery = ref('')
const loading = ref(false)
const rankings = ref<PlayerSeasonStats[]>([])

// 散点图着色模式
const scatterColorMode = ref<'position' | 'region'>('position')
// 散点图展开/折叠
const scatterExpanded = ref(true)

// 排序状态
const sortField = ref('yearlyTopScore')
const sortOrder = ref<'asc' | 'desc'>('desc')

// 分页状态
const currentPage = ref(1)
const pageSize = ref(20)

// 位置筛选
const positionFilters = [
  { label: '全部', value: '' },
  { label: 'TOP', value: 'TOP' },
  { label: 'JUG', value: 'JUG' },
  { label: 'MID', value: 'MID' },
  { label: 'ADC', value: 'ADC' },
  { label: 'SUP', value: 'SUP' },
]

// 异步获取排行数据
const fetchRankings = async () => {
  loading.value = true
  try {
    const seasonId = String(selectedSeason.value)
    logger.debug('[DataCenter] fetchRankings 开始, seasonId:', seasonId)
    // 增大 limit 以显示所有有比赛记录的选手
    const result = await playerStore.getSeasonImpactRanking(seasonId, 500)
    logger.debug('[DataCenter] fetchRankings 结果:', result?.length || 0, '条数据')
    if (result && result.length > 0) {
      logger.debug('[DataCenter] 第一条数据:', JSON.stringify(result[0]))
    }
    rankings.value = result
    logger.debug('[DataCenter] rankings.value 已更新，当前长度:', rankings.value.length)
  } catch (error) {
    logger.error('获取排行数据失败:', error)
    rankings.value = []
  } finally {
    loading.value = false
  }
}

// 概览统计数据
const dashboardStats = computed(() => {
  const data = rankings.value
  if (!data || data.length === 0) {
    return { totalPlayers: 0, avgImpact: 0, topScore: 0, avgGames: 0 }
  }
  const totalPlayers = data.length
  const avgImpact = data.reduce((sum, r) => sum + (r.avgImpact || 0), 0) / totalPlayers
  const topScore = Math.max(...data.map(r => r.yearlyTopScore || r.avgImpact || 0))
  const avgGames = data.reduce((sum, r) => sum + (r.gamesPlayed || 0), 0) / totalPlayers
  return { totalPlayers, avgImpact, topScore, avgGames }
})

// 位置对比图表配置
const positionComparisonOption = computed(() => {
  const positions = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  const posNames = ['上单', '打野', '中单', '下路', '辅助']
  const data = rankings.value

  const avgImpacts = positions.map(pos => {
    const players = data.filter(r => r.position === pos)
    if (players.length === 0) return 0
    return +(players.reduce((sum, r) => sum + (r.avgImpact || 0), 0) / players.length).toFixed(1)
  })

  const avgScores = positions.map(pos => {
    const players = data.filter(r => r.position === pos)
    if (players.length === 0) return 0
    return +(players.reduce((sum, r) => sum + (r.yearlyTopScore || r.avgImpact || 0), 0) / players.length).toFixed(1)
  })

  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' }
    },
    legend: {
      data: ['平均影响力', '平均得分'],
      top: 0,
      textStyle: { color: '#6b7280', fontSize: 12 }
    },
    grid: {
      left: '3%',
      right: '5%',
      bottom: '3%',
      top: '36px',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: posNames,
      axisLabel: { color: '#6b7280' },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    yAxis: [
      {
        type: 'value',
        name: '影响力',
        nameTextStyle: { color: '#3b82f6', fontSize: 11 },
        axisLabel: { color: '#3b82f6', fontSize: 11 },
        splitLine: { lineStyle: { color: '#f3f4f6' } }
      },
      {
        type: 'value',
        name: '得分',
        nameTextStyle: { color: '#8b5cf6', fontSize: 11 },
        axisLabel: { color: '#8b5cf6', fontSize: 11 },
        splitLine: { show: false }
      }
    ],
    series: [
      {
        name: '平均影响力',
        type: 'bar',
        yAxisIndex: 0,
        data: avgImpacts,
        itemStyle: { color: '#3b82f6', borderRadius: [4, 4, 0, 0] },
        barGap: '20%'
      },
      {
        name: '平均得分',
        type: 'bar',
        yAxisIndex: 1,
        data: avgScores,
        itemStyle: { color: '#8b5cf6', borderRadius: [4, 4, 0, 0] }
      }
    ]
  }
})

// 散点图着色方案
const positionColorMap: Record<string, string> = {
  TOP: '#e6a23c',
  JUG: '#67c23a',
  MID: '#409eff',
  ADC: '#f56c6c',
  SUP: '#909399'
}

const regionColorMap: Record<number, string> = {
  1: '#e6393f',  // LPL
  2: '#1a56db',  // LCK
  3: '#059669',  // LEC
  4: '#8b5cf6'   // LCS
}

const regionNameMap: Record<number, string> = {
  1: 'LPL',
  2: 'LCK',
  3: 'LEC',
  4: 'LCS'
}

// 获取选手赛区ID
const getPlayerRegionId = (teamId: string | number | null): number => {
  if (!teamId) return 0
  return teamsRegionMap.value.get(Number(teamId)) || 0
}

// 散点图四象限配置
const scatterOption = computed(() => {
  const players = filteredRankings.value
  if (players.length === 0) return {}

  // 数据：[稳定性, 影响力, 选手名, 战队名, 位置, playerId, regionId]
  const data = players.map(p => [
    p.consistencyScore || 0,      // x: 稳定性 0-100
    p.avgImpact || 0,             // y: 影响力
    p.playerName,
    getTeamName(p.teamId),
    p.position,
    p.playerId,
    getPlayerRegionId(p.teamId)
  ])

  // 计算均值线
  const avgStability = data.reduce((s, d) => s + (d[0] as number), 0) / data.length
  const avgImpact = data.reduce((s, d) => s + (d[1] as number), 0) / data.length

  // 计算数据范围，动态设置坐标轴让数据居中
  const stabilities = data.map(d => d[0] as number)
  const impacts = data.map(d => d[1] as number)
  const minStability = Math.min(...stabilities)
  const maxStability = Math.max(...stabilities)
  const minImpact = Math.min(...impacts)
  const maxImpact = Math.max(...impacts)

  // 稳定性轴：数据范围两侧加 15% padding，限制在 [0, 100]
  const xRange = maxStability - minStability || 20
  const xPad = xRange * 0.15
  const xMin = Math.max(0, Math.floor((minStability - xPad) / 5) * 5)
  const xMax = Math.min(100, Math.ceil((maxStability + xPad) / 5) * 5)

  // 影响力轴：数据范围两侧加 20% padding
  const yRange = maxImpact - minImpact || 5
  const yPad = yRange * 0.2
  const yMin = Math.floor((minImpact - yPad) * 2) / 2
  const yMax = Math.ceil((maxImpact + yPad) * 2) / 2

  return {
    tooltip: {
      trigger: 'item' as const,
      formatter: (params: any) => {
        const [stability, impact, name, team, pos] = params.data
        const regionId = params.data[6] as number
        const regionName = regionNameMap[regionId] || '-'
        return `<b>${name}</b> (${POSITION_NAMES[pos as PlayerPosition] || pos})<br/>` +
          `战队: ${team}<br/>` +
          `赛区: ${regionName}<br/>` +
          `稳定性: ${(stability as number).toFixed(0)}<br/>` +
          `影响力: ${(impact as number) >= 0 ? '+' : ''}${(impact as number).toFixed(2)}`
      }
    },
    grid: {
      left: '5%',
      right: '5%',
      bottom: '8%',
      top: '8%',
      containLabel: true
    },
    xAxis: {
      name: '稳定性 →',
      nameLocation: 'end' as const,
      nameTextStyle: { color: '#6b7280', fontSize: 12 },
      min: xMin,
      max: xMax,
      axisLabel: { color: '#6b7280' },
      splitLine: { lineStyle: { color: '#f3f4f6' } }
    },
    yAxis: {
      name: '↑ 影响力',
      nameLocation: 'end' as const,
      nameTextStyle: { color: '#6b7280', fontSize: 12 },
      min: yMin,
      max: yMax,
      axisLabel: { color: '#6b7280' },
      splitLine: { lineStyle: { color: '#f3f4f6' } }
    },
    series: [{
      type: 'scatter',
      data,
      symbolSize: (val: any[]) => Math.max(10, Math.abs(val[1] as number) * 1.5 + 10),
      itemStyle: {
        color: (params: any) => {
          if (scatterColorMode.value === 'region') {
            const regionId = params.data[6] as number
            return regionColorMap[regionId] || '#909399'
          }
          return positionColorMap[params.data[4] as string] || '#909399'
        },
        opacity: 0.8
      },
      emphasis: {
        itemStyle: {
          opacity: 1,
          shadowBlur: 10,
          shadowColor: 'rgba(0, 0, 0, 0.2)'
        }
      },
      markLine: {
        silent: true,
        lineStyle: {
          color: '#d1d5db',
          type: 'dashed' as const
        },
        label: {
          color: '#9ca3af',
          fontSize: 11
        },
        data: [
          {
            xAxis: avgStability,
            label: { formatter: `平均稳定性: ${avgStability.toFixed(0)}`, position: 'insideEndTop' as const }
          },
          {
            yAxis: avgImpact,
            label: { formatter: `平均影响力: ${avgImpact.toFixed(1)}`, position: 'insideEndTop' as const }
          }
        ]
      },
      markArea: {
        silent: true,
        data: [
          // 右上：核心基石
          [{
            xAxis: avgStability,
            yAxis: avgImpact,
            itemStyle: { color: 'rgba(16, 185, 129, 0.04)' }
          }, {
            xAxis: 100,
            yAxis: 'max'
          }],
          // 左上：波动明星
          [{
            xAxis: 0,
            yAxis: avgImpact,
            itemStyle: { color: 'rgba(245, 158, 11, 0.04)' }
          }, {
            xAxis: avgStability,
            yAxis: 'max'
          }],
          // 右下：稳定绿叶
          [{
            xAxis: avgStability,
            yAxis: 'min',
            itemStyle: { color: 'rgba(59, 130, 246, 0.04)' }
          }, {
            xAxis: 100,
            yAxis: avgImpact
          }],
          // 左下：待培养
          [{
            xAxis: 0,
            yAxis: 'min',
            itemStyle: { color: 'rgba(156, 163, 175, 0.04)' }
          }, {
            xAxis: avgStability,
            yAxis: avgImpact
          }]
        ]
      }
    }]
  }
})

// 散点图颜色图例数据
const scatterLegendItems = computed(() => {
  if (scatterColorMode.value === 'position') {
    return [
      { label: 'TOP', color: positionColorMap.TOP },
      { label: 'JUG', color: positionColorMap.JUG },
      { label: 'MID', color: positionColorMap.MID },
      { label: 'ADC', color: positionColorMap.ADC },
      { label: 'SUP', color: positionColorMap.SUP }
    ]
  }
  return [
    { label: 'LPL', color: regionColorMap[1] },
    { label: 'LCK', color: regionColorMap[2] },
    { label: 'LEC', color: regionColorMap[3] },
    { label: 'LCS', color: regionColorMap[4] }
  ]
})

// 散点图点击事件
const onScatterClick = (params: any) => {
  if (params.data && params.data[5]) {
    router.push(`/data-center/player/${params.data[5]}?season=S${selectedSeason.value}`)
  }
}

// Top5 选手
const top5Players = computed(() => {
  return filteredRankings.value.slice(0, 5)
})

// 计算属性 - 过滤后的排行榜
const filteredRankings = computed(() => {
  let result = [...rankings.value]

  // 位置筛选
  if (selectedPosition.value) {
    result = result.filter(r => r.position === selectedPosition.value)
  }

  // 搜索筛选
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(r =>
      r.playerName?.toLowerCase().includes(query)
    )
  }

  // 排序
  result.sort((a, b) => {
    let aValue: number
    let bValue: number

    switch (sortField.value) {
      case 'gamesPlayed':
        aValue = a.gamesPlayed || 0
        bValue = b.gamesPlayed || 0
        break
      case 'gamesBonus':
        aValue = (a.gamesPlayed || 0) / 10
        bValue = (b.gamesPlayed || 0) / 10
        break
      case 'avgImpact':
        aValue = a.avgImpact || 0
        bValue = b.avgImpact || 0
        break
      case 'championBonus':
        aValue = a.championBonus || 0
        bValue = b.championBonus || 0
        break
      case 'yearlyTopScore':
      default:
        aValue = a.yearlyTopScore || a.avgImpact || 0
        bValue = b.yearlyTopScore || b.avgImpact || 0
        break
    }

    if (sortOrder.value === 'asc') {
      return aValue - bValue
    } else {
      return bValue - aValue
    }
  })

  return result
})

// 计算属性 - 分页后的排行榜
const paginatedRankings = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredRankings.value.slice(start, end)
})

// 计算真实排名（考虑分页偏移）
const getRealRank = (index: number): number => {
  return (currentPage.value - 1) * pageSize.value + index + 1
}

// 分页变化处理
const handlePageChange = (page: number) => {
  currentPage.value = page
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
}

// 方法
const refreshData = async () => {
  // 加载战队数据（用于显示战队名称）
  try {
    if (teamsMap.value.size === 0 || teamsRegionMap.value.size === 0) {
      const teams = await teamApi.getAllTeams()
      const newTeamsMap = new Map<number, string>()
      const newRegionMap = new Map<number, number>()
      teams.forEach(t => {
        newTeamsMap.set(t.id, t.short_name || t.name)
        newRegionMap.set(t.id, t.region_id)
      })
      teamsMap.value = newTeamsMap
      teamsRegionMap.value = newRegionMap
      logger.debug('[DataCenter] 加载战队数据:', teamsMap.value.size, '支队伍')
    }
  } catch (e) {
    logger.warn('加载战队数据失败:', e)
  }
  playerStore.loadFromStorage()
  await fetchRankings()
}

// 同步数据库数据
const syncData = async () => {
  loading.value = true
  try {
    const seasonNum = selectedSeason.value
    logger.debug('[DataCenter] 开始同步数据, seasonNum:', seasonNum)
    const result = await devApi.syncPlayerGamesPlayed(seasonNum)
    logger.debug('[DataCenter] 同步结果:', result)

    // 处理返回结果
    if (result.success) {
      ElMessage.success(`数据同步成功: ${result.data?.updated_count || 0} 条记录已更新`)
    } else {
      ElMessage.error(`同步失败: ${result.error || result.message || '未知错误'}`)
      return
    }

    await fetchRankings()
  } catch (e: any) {
    logger.error('同步失败:', e)
    ElMessage.error(`数据同步失败: ${e.message || e}`)
  } finally {
    loading.value = false
  }
}

const goToPlayerDetail = (row: any) => {
  router.push(`/data-center/player/${row.playerId}?season=S${selectedSeason.value}`)
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
  if (value < -5) return 'impact-low'
  if (value < 0) return 'impact-negative'
  return ''
}

const getRankClass = (index: number): string => {
  if (index === 0) return 'rank-gold'
  if (index === 1) return 'rank-silver'
  if (index === 2) return 'rank-bronze'
  return ''
}

const getRankRowClass = ({ rowIndex }: { rowIndex: number }): string => {
  const realRank = getRealRank(rowIndex)
  if (realRank <= 3) return 'top-rank-row'
  return ''
}

const getConsistencyColor = (score: number | null | undefined): string => {
  if (score == null) return '#909399'
  if (score >= 80) return '#67c23a'
  if (score >= 60) return '#e6a23c'
  return '#f56c6c'
}

const getScoreClass = (score: number): string => {
  if (score > 15) return 'score-excellent'
  if (score > 10) return 'score-good'
  if (score > 5) return 'score-average'
  return 'score-normal'
}

// 生命周期
onMounted(() => {
  refreshData()
})

// 监听赛季变化
watch(selectedSeason, () => {
  currentPage.value = 1
  fetchRankings()
})

// 监听筛选条件变化，重置页码
watch([selectedPosition, searchQuery], () => {
  currentPage.value = 1
})
</script>

<style scoped lang="scss">
.data-center {
  padding: 24px;
  min-height: 100%;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content {
      .page-title {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 28px;
        font-weight: 700;
        margin: 0;
        color: #1f2937;

        .el-icon {
          color: #409eff;
        }
      }

      .page-description {
        margin: 8px 0 0 0;
        color: #6b7280;
        font-size: 14px;
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  // 概览统计卡片
  .dashboard-stats {
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
        width: 48px;
        height: 48px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 22px;
        color: white;

        &.players { background: linear-gradient(135deg, #3b82f6, #1d4ed8); }
        &.impact { background: linear-gradient(135deg, #10b981, #059669); }
        &.top-score { background: linear-gradient(135deg, #f59e0b, #d97706); }
        &.games { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
      }

      .stat-content {
        .stat-value {
          font-size: 24px;
          font-weight: 700;
          color: #1f2937;

          &.positive { color: #059669; }
          &.negative { color: #ef4444; }
          &.highlight { color: #d97706; }
        }

        .stat-label {
          font-size: 13px;
          color: #6b7280;
          margin-top: 2px;
        }
      }
    }
  }

  // 图表仪表盘行
  .charts-dashboard-row {
    display: grid;
    grid-template-columns: 3fr 2fr;
    gap: 16px;
    margin-bottom: 24px;

    .card-title {
      font-size: 16px;
      font-weight: 600;
      color: #1f2937;
    }
  }

  .position-chart-card {
    .chart-container {
      height: 280px;

      .chart {
        width: 100%;
        height: 100%;
      }
    }
  }

  .top5-card {
    .top5-list {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    .top5-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 10px 12px;
      border-radius: 8px;
      cursor: pointer;
      transition: background-color 0.2s;

      &:hover {
        background: #f0f9ff;
      }

      .top5-rank {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 13px;
        background: #f3f4f6;
        color: #6b7280;
        flex-shrink: 0;

        &.rank-gold {
          background: linear-gradient(135deg, #ffd700, #ffb347);
          color: #1a1a2e;
        }

        &.rank-silver {
          background: linear-gradient(135deg, #c0c0c0, #a8a8a8);
          color: #1a1a2e;
        }

        &.rank-bronze {
          background: linear-gradient(135deg, #cd7f32, #b87333);
          color: white;
        }
      }

      .top5-info {
        flex: 1;
        min-width: 0;

        .top5-name {
          font-weight: 600;
          font-size: 14px;
          color: #1f2937;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .top5-meta {
          display: flex;
          align-items: center;
          gap: 6px;
          margin-top: 2px;

          .top5-team {
            font-size: 12px;
            color: #9ca3af;
          }
        }
      }

      .top5-score {
        font-size: 18px;
        font-weight: 700;
        color: #7c3aed;
        flex-shrink: 0;
      }
    }
  }

  .filter-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding: 16px 20px;
    background: #f8fafc;
    border-radius: 12px;

    .position-filters {
      display: flex;
      gap: 8px;
    }

    .sort-filters {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .search-box {
      display: flex;
      align-items: center;
    }
  }

  .rankings-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .card-title {
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }
    }

    .pagination-container {
      display: flex;
      justify-content: center;
      margin-top: 20px;
      padding-top: 16px;
      border-top: 1px solid #ebeef5;
    }
  }

  .clickable-table {
    :deep(.el-table__row) {
      cursor: pointer;
      transition: background-color 0.2s;

      &:hover {
        background-color: #f0f9ff !important;
      }
    }
  }

  .player-cell {
    display: flex;
    align-items: center;
    gap: 8px;

    .player-name {
      font-weight: 600;
      color: #1f2937;
    }

    .arrow-icon {
      color: #9ca3af;
      opacity: 0;
      transition: opacity 0.2s;
    }
  }

  .el-table__row:hover .arrow-icon {
    opacity: 1;
  }

  .rank-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    font-weight: bold;
    font-size: 14px;
    background: #f3f4f6;
    color: #6b7280;

    &.rank-gold {
      background: linear-gradient(135deg, #ffd700, #ffb347);
      color: #1a1a2e;
      box-shadow: 0 2px 8px rgba(255, 215, 0, 0.4);
    }

    &.rank-silver {
      background: linear-gradient(135deg, #c0c0c0, #a8a8a8);
      color: #1a1a2e;
      box-shadow: 0 2px 8px rgba(192, 192, 192, 0.4);
    }

    &.rank-bronze {
      background: linear-gradient(135deg, #cd7f32, #b87333);
      color: white;
      box-shadow: 0 2px 8px rgba(205, 127, 50, 0.4);
    }
  }

  .impact-high {
    color: #059669;
    font-weight: bold;
  }

  .impact-positive {
    color: #10b981;
    font-weight: 500;
  }

  .impact-negative {
    color: #f59e0b;
    font-weight: 500;
  }

  .impact-low {
    color: #ef4444;
    font-weight: bold;
  }

  .consistency-cell {
    display: flex;
    align-items: center;
    gap: 8px;

    .el-progress {
      flex: 1;
    }

    .consistency-value {
      font-size: 12px;
      font-weight: 500;
      color: #6b7280;
      min-width: 24px;
    }
  }

  .yearly-score {
    font-weight: bold;
    font-size: 16px;

    &.score-excellent {
      color: #fbbf24;
    }

    &.score-good {
      color: #10b981;
    }

    &.score-average {
      color: #3b82f6;
    }

    &.score-normal {
      color: #6b7280;
    }
  }

  // 散点图区域
  .scatter-chart-section {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      &.clickable {
        cursor: pointer;
        user-select: none;
      }

      .card-header-left {
        display: flex;
        align-items: center;
        gap: 8px;
      }

      .collapse-arrow {
        transition: transform 0.3s;
        color: #9ca3af;
        font-size: 16px;

        &.collapsed {
          transform: rotate(-90deg);
        }
      }

      .card-title {
        font-size: 16px;
        font-weight: 600;
        color: #1f2937;
      }
    }

    .scatter-chart-wrapper {
      height: 500px;

      .scatter-chart {
        width: 100%;
        height: 100%;
      }
    }

    .scatter-legend {
      display: flex;
      justify-content: center;
      gap: 20px;
      padding: 12px 0 8px;

      .legend-item {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: #4b5563;

        .legend-dot {
          width: 10px;
          height: 10px;
          border-radius: 50%;
          display: inline-block;
        }
      }
    }

    .quadrant-legend {
      display: flex;
      justify-content: center;
      flex-wrap: wrap;
      gap: 16px;
      padding: 8px 0 4px;
      border-top: 1px solid #f3f4f6;

      .q {
        font-size: 12px;
        padding: 4px 10px;
        border-radius: 4px;
        font-weight: 500;
      }

      .q1 {
        color: #059669;
        background: rgba(16, 185, 129, 0.08);
      }

      .q2 {
        color: #d97706;
        background: rgba(245, 158, 11, 0.08);
      }

      .q3 {
        color: #2563eb;
        background: rgba(59, 130, 246, 0.08);
      }

      .q4 {
        color: #6b7280;
        background: rgba(156, 163, 175, 0.08);
      }
    }
  }

  :deep(.top-rank-row) {
    background-color: #fefce8 !important;
  }
}

@media (max-width: 1024px) {
  .data-center {
    .dashboard-stats {
      grid-template-columns: repeat(2, 1fr);
    }

    .charts-dashboard-row {
      grid-template-columns: 1fr;
    }
  }
}

@media (max-width: 640px) {
  .data-center {
    .dashboard-stats {
      grid-template-columns: 1fr;
    }

    .filter-bar {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }
  }
}
</style>
