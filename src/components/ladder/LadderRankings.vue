<template>
  <div class="ladder-rankings-container">
    <div class="filter-section">
      <div class="filter-row">
        <div class="filter-left">
          <el-button-group>
            <el-button
              v-for="pos in positionFilters"
              :key="pos.value"
              :type="selectedPosition === pos.value ? 'primary' : 'default'"
              @click="selectedPosition = pos.value"
              round
              size="small"
            >{{ pos.label }}</el-button>
          </el-button-group>
        </div>
        <div class="filter-right">
          <el-select v-model="sortField" style="width: 120px" size="small">
            <el-option label="天梯分" value="rating" />
            <el-option label="胜率" value="win_rate" />
            <el-option label="MVP" value="mvp_count" />
            <el-option label="影响力" value="avg_influence" />
          </el-select>
          <el-button-group>
            <el-button :type="sortOrder === 'desc' ? 'primary' : 'default'" @click="sortOrder = 'desc'" size="small">
              <el-icon><ArrowDown /></el-icon>
            </el-button>
            <el-button :type="sortOrder === 'asc' ? 'primary' : 'default'" @click="sortOrder = 'asc'" size="small">
              <el-icon><ArrowUp /></el-icon>
            </el-button>
          </el-button-group>
          <el-input
            v-model="searchQuery"
            placeholder="搜索选手/队伍..."
            :prefix-icon="Search"
            clearable
            size="small"
            style="width: 180px"
          />
        </div>
      </div>
    </div>

    <div class="table-section">
      <el-table
        :data="paginatedRankings"
        v-loading="loading"
        style="width: 100%"
        :row-class-name="getRowClass"
        size="small"
        class="rankings-table"
        @row-click="toggleExpand"
      >
      <el-table-column label="排名" width="70" align="center">
        <template #default="{ row }">
          <div class="rank-badge" :class="getRankClass(row.rank)">{{ row.rank }}</div>
        </template>
      </el-table-column>

      <el-table-column label="选手" min-width="120">
        <template #default="{ row }">
          <div class="player-cell">
            <span class="player-name">{{ row.game_id }}</span>
            <el-icon class="expand-icon" :class="{ 'is-expanded': expandedPlayerId === row.player_id }"><ArrowDown /></el-icon>
          </div>
        </template>
      </el-table-column>

      <el-table-column label="队伍" width="110" align="center">
        <template #default="{ row }">
          <span class="team-name">{{ row.team_name || '-' }}</span>
        </template>
      </el-table-column>

      <el-table-column label="位置" width="75" align="center">
        <template #default="{ row }">
          <el-tag :type="posTagType(row.position)" size="small" effect="light">{{ row.position }}</el-tag>
        </template>
      </el-table-column>

      <el-table-column label="天梯分" width="130" align="center">
        <template #default="{ row }">
          <div class="mini-bar-cell">
            <div class="mini-bar">
              <div class="mini-fill" :class="ratingBarClass(row.rating)" :style="{ width: ratingBarWidth(row.rating) }"></div>
            </div>
            <span class="rating-val" :class="ratingTextClass(row.rating)">{{ row.rating }}</span>
          </div>
        </template>
      </el-table-column>

      <el-table-column label="战绩" width="90" align="center">
        <template #default="{ row }">
          <span class="record"><span class="win-num">{{ row.wins }}</span>胜<span class="loss-num">{{ row.losses }}</span>负</span>
        </template>
      </el-table-column>

      <el-table-column label="胜率" width="80" align="center">
        <template #default="{ row }">
          <span :class="winRateClass(row.win_rate)">{{ row.win_rate.toFixed(1) }}%</span>
        </template>
      </el-table-column>

      <el-table-column label="MVP" width="60" align="center">
        <template #default="{ row }">
          <span :class="{ 'mvp-gold': row.mvp_count > 0 }">{{ row.mvp_count }}</span>
        </template>
      </el-table-column>

      <el-table-column label="影响力" width="120" align="center">
        <template #default="{ row }">
          <div class="mini-bar-cell">
            <div class="mini-bar">
              <div class="mini-fill" :class="row.avg_influence >= 0 ? 'fill-green' : 'fill-red'" :style="{ width: influenceBarWidth(row.avg_influence) }"></div>
            </div>
            <span :class="row.avg_influence >= 0 ? 'text-green' : 'text-red'">{{ row.avg_influence > 0 ? '+' : '' }}{{ row.avg_influence.toFixed(2) }}</span>
          </div>
        </template>
      </el-table-column>

      <el-table-column label="最高分" width="80" align="center">
        <template #default="{ row }">
          <span class="max-rating">{{ row.max_rating }}</span>
        </template>
      </el-table-column>
    </el-table>

    <transition name="chart-slide">
      <div v-if="expandedPlayerId !== null" class="chart-expand-row" :key="expandedPlayerId">
        <div class="chart-header">
          <span class="chart-title">{{ expandedPlayerName }} 天梯分走势</span>
          <span v-if="chartData" class="chart-summary">
            <span class="text-green">最高 {{ chartMax }}</span>
            <span class="text-red">最低 {{ chartMin }}</span>
            <span>{{ chartData.length - 1 }} 场</span>
          </span>
        </div>
        <div v-if="chartLoading" class="chart-loading">
          <el-icon class="is-loading"><Loading /></el-icon>
        </div>
        <v-chart v-else-if="chartOption" :option="chartOption" :style="{ height: '200px', width: '100%' }" autoresize />
      </div>
    </transition>

    <div class="pagination-wrapper" v-if="filteredRankings.length > 0">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="filteredRankings.length"
        layout="total, sizes, prev, pager, next"
        size="small"
      />
    </div>
    </div>
    <div v-if="!loading && filteredRankings.length === 0" class="empty-state">
      <el-empty description="暂无数据" :image-size="60" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Search, ArrowUp, ArrowDown, Loading } from '@element-plus/icons-vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import { TooltipComponent, GridComponent, MarkLineComponent } from 'echarts/components'
import VChart from 'vue-echarts'
import type { LadderRankingEntry, RatingHistoryPoint } from '@/api/ladder'
import { getPlayerLadderRatingHistory } from '@/api/ladder'

use([CanvasRenderer, LineChart, TooltipComponent, GridComponent, MarkLineComponent])

const props = defineProps<{ rankings: LadderRankingEntry[]; loading: boolean; tournamentId: number }>()

const selectedPosition = ref('')
const searchQuery = ref('')
const sortField = ref('rating')
const sortOrder = ref<'asc' | 'desc'>('desc')
const currentPage = ref(1)
const pageSize = ref(20)

const expandedPlayerId = ref<number | null>(null)
const expandedPlayerName = ref('')
const chartLoading = ref(false)
const chartData = ref<RatingHistoryPoint[] | null>(null)
const historyCache = new Map<number, RatingHistoryPoint[]>()

const positionFilters = [
  { label: '全部', value: '' },
  { label: 'Top', value: 'Top' },
  { label: 'Jug', value: 'Jug' },
  { label: 'Mid', value: 'Mid' },
  { label: 'Adc', value: 'Adc' },
  { label: 'Sup', value: 'Sup' },
]

const filteredRankings = computed(() => {
  let r = [...props.rankings]
  if (selectedPosition.value) r = r.filter(x => x.position === selectedPosition.value)
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    r = r.filter(x => x.game_id.toLowerCase().includes(q) || (x.team_name && x.team_name.toLowerCase().includes(q)))
  }
  r.sort((a, b) => {
    const av = a[sortField.value as keyof LadderRankingEntry] as number
    const bv = b[sortField.value as keyof LadderRankingEntry] as number
    return sortOrder.value === 'desc' ? (bv ?? 0) - (av ?? 0) : (av ?? 0) - (bv ?? 0)
  })
  return r
})

const paginatedRankings = computed(() => {
  const s = (currentPage.value - 1) * pageSize.value
  return filteredRankings.value.slice(s, s + pageSize.value)
})

watch([selectedPosition, searchQuery, sortField, sortOrder], () => { currentPage.value = 1 })

const chartMax = computed(() => chartData.value ? Math.max(...chartData.value.map(p => p.rating)) : 0)
const chartMin = computed(() => chartData.value ? Math.min(...chartData.value.map(p => p.rating)) : 0)

const chartOption = computed(() => {
  if (!chartData.value || chartData.value.length < 2) return null
  const data = chartData.value
  const ratings = data.map(p => p.rating)
  const minR = Math.min(...ratings)
  const maxR = Math.max(...ratings)
  const padding = Math.max(20, Math.round((maxR - minR) * 0.15))

  return {
    grid: { left: 50, right: 20, top: 16, bottom: 30 },
    tooltip: {
      trigger: 'axis' as const,
      formatter: (params: { dataIndex: number; value: number }[]) => {
        const p = params[0]
        const point = data[p.dataIndex]
        const label = point.round === 0 ? '初始' : `第 ${point.round} 轮`
        const changeStr = point.change > 0
          ? `<span style="color:#10b981">+${point.change}</span>`
          : point.change < 0
            ? `<span style="color:#ef4444">${point.change}</span>`
            : ''
        return `${label}<br/>天梯分: <b>${point.rating}</b> ${changeStr}`
      }
    },
    xAxis: {
      type: 'category' as const,
      data: data.map(p => p.round === 0 ? '初始' : `R${p.round}`),
      axisLabel: { fontSize: 10, color: '#94a3b8' },
      axisLine: { lineStyle: { color: '#e2e8f0' } },
      axisTick: { show: false },
    },
    yAxis: {
      type: 'value' as const,
      min: minR - padding,
      max: maxR + padding,
      axisLabel: { fontSize: 10, color: '#94a3b8' },
      splitLine: { lineStyle: { color: '#f1f5f9', type: 'dashed' as const } },
    },
    series: [{
      type: 'line' as const,
      data: ratings,
      smooth: true,
      symbol: 'circle',
      symbolSize: 4,
      lineStyle: { color: '#6366f1', width: 2 },
      itemStyle: { color: '#6366f1' },
      areaStyle: {
        color: {
          type: 'linear' as const,
          x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(99,102,241,0.15)' },
            { offset: 1, color: 'rgba(99,102,241,0.01)' },
          ]
        }
      },
      markLine: {
        silent: true,
        symbol: 'none',
        lineStyle: { color: '#94a3b8', type: 'dashed' as const, width: 1 },
        data: [{ yAxis: 1200, label: { formatter: '1200', fontSize: 9, color: '#94a3b8' } }]
      }
    }]
  }
})

async function toggleExpand(row: LadderRankingEntry) {
  if (expandedPlayerId.value === row.player_id) {
    expandedPlayerId.value = null
    return
  }
  expandedPlayerId.value = row.player_id
  expandedPlayerName.value = row.game_id

  if (historyCache.has(row.player_id)) {
    chartData.value = historyCache.get(row.player_id)!
    return
  }

  chartLoading.value = true
  try {
    const data = await getPlayerLadderRatingHistory(props.tournamentId, row.player_id)
    historyCache.set(row.player_id, data)
    chartData.value = data
  } catch {
    chartData.value = null
  } finally {
    chartLoading.value = false
  }
}

const getRankClass = (rank: number) => rank === 1 ? 'rank-gold' : rank === 2 ? 'rank-silver' : rank === 3 ? 'rank-bronze' : ''
const getRowClass = ({ row }: { row: LadderRankingEntry }) => {
  let cls = row.rank <= 3 ? 'top-row' : ''
  if (expandedPlayerId.value === row.player_id) cls += ' expanded-row'
  return cls
}
const posTagType = (p: string) => ({ Top: 'danger', Jug: 'warning', Mid: '', Adc: 'success', Sup: 'info' }[p] || '') as '' | 'success' | 'warning' | 'info' | 'danger'
const ratingBarWidth = (r: number) => Math.min(100, Math.max(0, (r - 800) / 8)) + '%'
const ratingBarClass = (r: number) => r > 1400 ? 'fill-blue-bright' : r < 1000 ? 'fill-red' : 'fill-blue'
const ratingTextClass = (r: number) => r > 1400 ? 'text-blue-bright' : r < 1000 ? 'text-red' : ''
const winRateClass = (r: number) => r >= 60 ? 'text-green' : r >= 50 ? 'text-orange' : 'text-red'
const influenceBarWidth = (v: number) => Math.min(100, Math.max(0, (v + 5) * 10)) + '%'
</script>

<style scoped>
.ladder-rankings-container { padding: 16px 0; }

.filter-section { margin-bottom: 16px; }
.filter-row {
  display: flex; flex-wrap: wrap; gap: 10px; align-items: center;
  justify-content: space-between;
}
.filter-left, .filter-right { display: flex; align-items: center; gap: 8px; }

.table-section {
  border: 1px solid #e2e8f0; border-radius: 10px; padding: 16px;
}

.rankings-table :deep(.el-table th.el-table__cell) {
  font-weight: 600; color: #94a3b8; font-size: 11px;
  text-transform: uppercase; letter-spacing: 0.5px;
  background: transparent; border-bottom: 1px solid #f1f5f9;
  padding: 10px 0;
}
.rankings-table :deep(.el-table__body tr) { transition: background-color 0.15s; cursor: pointer; }
.rankings-table :deep(.el-table__body tr td) {
  padding: 12px 0; border-bottom: 1px solid #f8fafc;
}
.rankings-table :deep(.el-table__body tr:hover > td) {
  background-color: #f8fafc !important;
}
.rankings-table :deep(.el-table__body tr:last-child td) { border-bottom: none; }
.rankings-table :deep(.top-row td) { background-color: #fefce8 !important; }
.rankings-table :deep(.expanded-row td) { background-color: #eef2ff !important; }

.player-cell { display: flex; align-items: center; gap: 6px; }
.expand-icon {
  font-size: 12px; color: #94a3b8; transition: transform 0.2s;
}
.expand-icon.is-expanded { transform: rotate(180deg); color: #6366f1; }

.rank-badge {
  width: 28px; height: 28px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-weight: 700; font-size: 12px; margin: 0 auto;
  background: #f1f5f9; color: #64748b;
}
.rank-gold { background: linear-gradient(135deg, #ffd700, #ffb347); color: #1a1a2e; }
.rank-silver { background: linear-gradient(135deg, #c0c0c0, #a8a8a8); color: #1a1a2e; }
.rank-bronze { background: linear-gradient(135deg, #cd7f32, #b87333); color: #fff; }

.player-name { font-weight: 600; color: #0f172a; font-size: 13px; }
.team-name { font-size: 12px; color: #64748b; }

.mini-bar-cell { display: flex; align-items: center; gap: 6px; }
.mini-bar { flex: 1; height: 4px; background: #f1f5f9; border-radius: 2px; overflow: hidden; min-width: 30px; }
.mini-fill { height: 100%; border-radius: 2px; transition: width 0.3s; }
.fill-blue { background: #6366f1; }
.fill-blue-bright { background: #4f46e5; }
.fill-red { background: #ef4444; }
.fill-green { background: #10b981; }

.rating-val { font-weight: 600; font-size: 13px; font-variant-numeric: tabular-nums; min-width: 36px; text-align: right; color: #0f172a; }
.text-blue-bright { color: #4f46e5; font-weight: 700; }
.text-green { color: #10b981; font-weight: 600; }
.text-orange { color: #f59e0b; font-weight: 600; }
.text-red { color: #ef4444; font-weight: 600; }

.record { font-size: 12px; color: #475569; }
.win-num { color: #10b981; font-weight: 600; }
.loss-num { color: #ef4444; font-weight: 600; }

.mvp-gold { color: #f59e0b; font-weight: 700; }
.max-rating { font-size: 12px; color: #94a3b8; font-variant-numeric: tabular-nums; }

.chart-expand-row {
  border: 1px solid #e0e7ff; border-radius: 8px;
  background: linear-gradient(180deg, #f8faff 0%, #fff 100%);
  padding: 12px 16px; margin: 8px 0;
}
.chart-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 8px;
}
.chart-title { font-size: 13px; font-weight: 600; color: #1e293b; }
.chart-summary { display: flex; gap: 12px; font-size: 11px; color: #64748b; }
.chart-loading {
  height: 200px; display: flex; align-items: center; justify-content: center;
  color: #94a3b8; font-size: 20px;
}

.chart-slide-enter-active { transition: all 0.25s ease-out; }
.chart-slide-leave-active { transition: all 0.15s ease-in; }
.chart-slide-enter-from { opacity: 0; max-height: 0; transform: translateY(-8px); }
.chart-slide-enter-to { opacity: 1; max-height: 300px; transform: translateY(0); }
.chart-slide-leave-from { opacity: 1; max-height: 300px; }
.chart-slide-leave-to { opacity: 0; max-height: 0; }

.pagination-wrapper { margin-top: 16px; display: flex; justify-content: center; }
.empty-state { padding: 40px 0; display: flex; justify-content: center; }
</style>
