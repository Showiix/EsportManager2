<template>
  <div class="performance-monitor">
    <!-- 页面标题 -->
    <div class="page-header">
      <div class="header-left">
        <h1>性能监测</h1>
        <el-tag :type="store.isMonitoring ? 'success' : 'info'" effect="dark" size="large">
          {{ store.isMonitoring ? '监测中' : '未启动' }}
        </el-tag>
      </div>
      <div class="header-actions">
        <el-button
          v-if="!store.isMonitoring"
          type="primary"
          :icon="VideoPlay"
          @click="store.startMonitoring"
        >
          开始监测
        </el-button>
        <el-button
          v-else
          type="danger"
          :icon="VideoPause"
          @click="store.stopMonitoring"
        >
          停止监测
        </el-button>
        <el-button :icon="Delete" @click="handleClear">清除数据</el-button>
        <el-button :icon="Download" @click="handleExport">导出 JSON</el-button>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-cards">
      <el-card v-for="card in summaryCards" :key="card.label" class="stat-card" shadow="hover">
        <div class="stat-icon" :class="card.colorClass">
          <el-icon :size="24"><component :is="card.icon" /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ card.value }}</div>
          <div class="stat-label">{{ card.label }}</div>
        </div>
      </el-card>
    </div>

    <!-- 图表区域 -->
    <div class="charts-row">
      <!-- 命令耗时 Top 10 -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <span class="chart-title">命令耗时 Top 10 (平均)</span>
        </template>
        <div class="chart-container">
          <v-chart
            v-if="store.commandDistribution.length > 0"
            class="chart"
            :option="barChartOption"
            autoresize
          />
          <el-empty v-else description="暂无数据，请开始监测" :image-size="80" />
        </div>
      </el-card>

      <!-- 时间线趋势 -->
      <el-card class="chart-card" shadow="hover">
        <template #header>
          <span class="chart-title">请求趋势 (10秒间隔)</span>
        </template>
        <div class="chart-container">
          <v-chart
            v-if="store.timelineBuckets.length > 0"
            class="chart"
            :option="lineChartOption"
            autoresize
          />
          <el-empty v-else description="暂无数据，请开始监测" :image-size="80" />
        </div>
      </el-card>
    </div>

    <!-- 操作日志表格 -->
    <el-card class="log-card" shadow="hover">
      <template #header>
        <div class="log-header">
          <span class="chart-title">操作日志 ({{ filteredLogEntries.length }} 条)</span>
          <div class="log-filters">
            <el-input
              v-model="searchText"
              placeholder="搜索命令名..."
              clearable
              size="small"
              style="width: 200px"
              :prefix-icon="Search"
            />
            <el-select v-model="typeFilter" size="small" clearable placeholder="类型" style="width: 120px">
              <el-option label="IPC 调用" value="invoke" />
              <el-option label="页面导航" value="navigation" />
            </el-select>
            <el-select v-model="statusFilter" size="small" clearable placeholder="状态" style="width: 120px">
              <el-option label="成功" value="success" />
              <el-option label="失败" value="error" />
              <el-option label="慢请求" value="slow" />
            </el-select>
          </div>
        </div>
      </template>

      <el-table
        :data="paginatedLogEntries"
        stripe
        max-height="500"
        style="width: 100%"
        :default-sort="{ prop: 'timestamp', order: 'descending' }"
      >
        <el-table-column prop="type" label="类型" width="100">
          <template #default="{ row }">
            <el-tag :type="row.type === 'invoke' ? 'primary' : 'success'" size="small">
              {{ row.type === 'invoke' ? 'IPC' : '导航' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="name" label="名称" min-width="220" show-overflow-tooltip />
        <el-table-column prop="duration" label="耗时" width="120" sortable>
          <template #default="{ row }">
            <span :class="getDurationClass(row.duration)">
              {{ row.duration }}ms
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="success" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.success ? 'success' : 'danger'" size="small">
              {{ row.success ? '成功' : '失败' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="timestamp" label="时间" width="140" sortable>
          <template #default="{ row }">
            {{ formatTime(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column prop="error" label="错误信息" min-width="200" show-overflow-tooltip />
      </el-table>

      <div class="pagination-wrapper" v-if="filteredLogEntries.length > pageSize">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="filteredLogEntries.length"
          layout="prev, pager, next, total"
          small
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePerformanceStore } from '@/stores/usePerformanceStore'
import {
  VideoPlay, VideoPause, Delete, Download, Search,
  Stopwatch, Timer, Warning, CircleClose, DataAnalysis, Connection,
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'

// ECharts
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart, LineChart } from 'echarts/charts'
import {
  TooltipComponent,
  GridComponent,
  LegendComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'

use([CanvasRenderer, BarChart, LineChart, TooltipComponent, GridComponent, LegendComponent])

const store = usePerformanceStore()

// ========================================
// 统计卡片
// ========================================

const summaryCards = computed(() => {
  const s = store.frontendSummary
  return [
    {
      label: '总请求数',
      value: s.totalRequests.toLocaleString(),
      icon: Connection,
      colorClass: 'color-primary',
    },
    {
      label: '平均耗时',
      value: `${s.avgDurationMs}ms`,
      icon: Timer,
      colorClass: 'color-success',
    },
    {
      label: 'P95 耗时',
      value: `${s.p95DurationMs}ms`,
      icon: Stopwatch,
      colorClass: 'color-warning',
    },
    {
      label: '最大耗时',
      value: `${s.maxDurationMs}ms`,
      icon: DataAnalysis,
      colorClass: 'color-info',
    },
    {
      label: '慢请求',
      value: s.slowRequests.toLocaleString(),
      icon: Warning,
      colorClass: s.slowRequests > 0 ? 'color-danger' : 'color-success',
    },
    {
      label: '错误率',
      value: `${s.errorRate}%`,
      icon: CircleClose,
      colorClass: s.errorCount > 0 ? 'color-danger' : 'color-success',
    },
  ]
})

// ========================================
// 图表配置
// ========================================

const barChartOption = computed(() => {
  const top10 = store.commandDistribution.slice(0, 10).reverse()
  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      formatter: (params: any) => {
        const item = params[0]
        const cmd = top10[item.dataIndex]
        return `<b>${cmd.command}</b><br/>
          平均: ${cmd.avgMs}ms<br/>
          最大: ${cmd.maxMs}ms<br/>
          调用次数: ${cmd.count}<br/>
          错误: ${cmd.errors}`
      }
    },
    grid: { left: '30%', right: '8%', top: '3%', bottom: '3%' },
    xAxis: {
      type: 'value',
      name: 'ms',
      axisLabel: { color: '#a0a0a0' },
    },
    yAxis: {
      type: 'category',
      data: top10.map(d => {
        const name = d.command.replace(/^(get_|set_|update_|delete_)/, '')
        return name.length > 25 ? name.slice(0, 22) + '...' : name
      }),
      axisLabel: { color: '#c0c0c0', fontSize: 11 },
    },
    series: [{
      type: 'bar',
      data: top10.map(d => d.avgMs),
      barMaxWidth: 20,
      itemStyle: {
        borderRadius: [0, 4, 4, 0],
        color: (params: any) => {
          const value = params.value
          if (value > 1000) return '#F56C6C'
          if (value > 500) return '#E6A23C'
          if (value > 200) return '#409EFF'
          return '#67C23A'
        }
      },
    }],
  }
})

const lineChartOption = computed(() => {
  const buckets = store.timelineBuckets
  return {
    tooltip: { trigger: 'axis' },
    legend: {
      data: ['请求数', '平均耗时(ms)'],
      textStyle: { color: '#c0c0c0' },
    },
    grid: { left: '10%', right: '10%', top: '15%', bottom: '10%' },
    xAxis: {
      type: 'category',
      data: buckets.map(b => b.time),
      axisLabel: { color: '#a0a0a0', fontSize: 10 },
    },
    yAxis: [
      {
        type: 'value',
        name: '请求数',
        position: 'left',
        axisLabel: { color: '#a0a0a0' },
        nameTextStyle: { color: '#a0a0a0' },
      },
      {
        type: 'value',
        name: 'ms',
        position: 'right',
        axisLabel: { color: '#a0a0a0' },
        nameTextStyle: { color: '#a0a0a0' },
      },
    ],
    series: [
      {
        name: '请求数',
        type: 'line',
        data: buckets.map(b => b.count),
        smooth: true,
        areaStyle: { opacity: 0.15 },
        lineStyle: { color: '#409EFF' },
        itemStyle: { color: '#409EFF' },
      },
      {
        name: '平均耗时(ms)',
        type: 'line',
        yAxisIndex: 1,
        data: buckets.map(b => b.avgMs),
        smooth: true,
        lineStyle: { color: '#E6A23C' },
        itemStyle: { color: '#E6A23C' },
      },
    ],
  }
})

// ========================================
// 日志表格
// ========================================

const searchText = ref('')
const typeFilter = ref('')
const statusFilter = ref('')
const currentPage = ref(1)
const pageSize = 100

const filteredLogEntries = computed(() => {
  let entries = store.logEntries
  if (searchText.value) {
    const keyword = searchText.value.toLowerCase()
    entries = entries.filter(e => e.name.toLowerCase().includes(keyword))
  }
  if (typeFilter.value) {
    entries = entries.filter(e => e.type === typeFilter.value)
  }
  if (statusFilter.value === 'success') {
    entries = entries.filter(e => e.success)
  } else if (statusFilter.value === 'error') {
    entries = entries.filter(e => !e.success)
  } else if (statusFilter.value === 'slow') {
    entries = entries.filter(e => e.duration > store.slowThresholdMs)
  }
  return entries
})

const paginatedLogEntries = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return filteredLogEntries.value.slice(start, start + pageSize)
})

function getDurationClass(duration: number): string {
  if (duration > 1000) return 'duration-danger'
  if (duration > 500) return 'duration-warning'
  if (duration > 200) return 'duration-info'
  return 'duration-success'
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 1,
  } as Intl.DateTimeFormatOptions)
}

// ========================================
// 操作
// ========================================

async function handleClear() {
  await ElMessageBox.confirm('确定清除所有监测数据？', '清除数据', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
  store.clearAllData()
  currentPage.value = 1
  ElMessage.success('数据已清除')
}

function handleExport() {
  const data = store.exportData()
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `perf-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`
  a.click()
  URL.revokeObjectURL(url)
  ElMessage.success('导出成功')
}
</script>

<style scoped lang="scss">
.performance-monitor {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;

  h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 600;
    color: #e0e0e0;
  }
}

.header-actions {
  display: flex;
  gap: 8px;
}

// 统计卡片
.stats-cards {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 16px;
}

.stat-card {
  :deep(.el-card__body) {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
  }
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;

  &.color-primary { background: rgba(64, 158, 255, 0.15); color: #409EFF; }
  &.color-success { background: rgba(103, 194, 58, 0.15); color: #67C23A; }
  &.color-warning { background: rgba(230, 162, 60, 0.15); color: #E6A23C; }
  &.color-danger { background: rgba(245, 108, 108, 0.15); color: #F56C6C; }
  &.color-info { background: rgba(144, 147, 153, 0.15); color: #909399; }
}

.stat-content {
  .stat-value {
    font-size: 24px;
    font-weight: 700;
    color: #e0e0e0;
    line-height: 1.2;
  }
  .stat-label {
    font-size: 13px;
    color: #909399;
    margin-top: 4px;
  }
}

// 图表区域
.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.chart-card {
  .chart-container {
    height: 300px;
  }
  .chart {
    width: 100%;
    height: 100%;
  }
}

.chart-title {
  font-weight: 600;
  color: #e0e0e0;
}

// 日志表格
.log-card {
  .log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .log-filters {
    display: flex;
    gap: 8px;
  }
}

.pagination-wrapper {
  display: flex;
  justify-content: center;
  padding-top: 16px;
}

// 耗时颜色
.duration-success { color: #67C23A; font-weight: 600; }
.duration-info { color: #409EFF; font-weight: 600; }
.duration-warning { color: #E6A23C; font-weight: 700; }
.duration-danger { color: #F56C6C; font-weight: 700; }

// 响应式
@media (max-width: 1400px) {
  .stats-cards {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 1000px) {
  .charts-row {
    grid-template-columns: 1fr;
  }
  .stats-cards {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
