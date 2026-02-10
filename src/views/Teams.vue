<template>
  <div class="team-list">
    <!-- 页面标题和操作 -->
    <div class="page-header">
      <div>
        <h1>战队管理</h1>
        <p>管理所有赛区的战队信息和属性</p>
      </div>
    </div>

    <!-- 筛选和搜索 -->
    <el-card class="filter-card">
      <el-row :gutter="16" align="middle">
        <el-col :span="6">
          <el-input
            v-model="searchQuery"
            placeholder="搜索战队名称"
            clearable
            :prefix-icon="Search"
          />
        </el-col>
        <el-col :span="4">
          <el-select
            v-model="selectedRegionId"
            placeholder="选择赛区"
            clearable
            @change="onRegionChange"
          >
            <el-option label="全部赛区" :value="0" />
            <el-option
              v-for="region in regions"
              :key="region.id"
              :label="region.name"
              :value="region.id"
            />
          </el-select>
        </el-col>
        <el-col :span="6">
          <div class="strength-filter">
            <span>战力值:</span>
            <el-slider
              v-model="strengthRange"
              range
              :min="0"
              :max="100"
            />
          </div>
        </el-col>
        <el-col :span="4">
          <el-select v-model="sortBy" placeholder="排序方式">
            <el-option label="战队名称" value="name" />
            <el-option label="战力值" value="strength" />
            <el-option label="胜率" value="winRate" />
          </el-select>
        </el-col>
        <el-col :span="4">
          <el-button type="primary" @click="applyFilters">应用筛选</el-button>
          <el-button @click="clearFilters">清空</el-button>
        </el-col>
      </el-row>
    </el-card>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><UserFilled /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ displayTeams.length }}</div>
              <div class="stat-label">战队总数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="28"><TrendCharts /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ averageStrength.toFixed(1) }}</div>
              <div class="stat-label">平均战力</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="28"><Trophy /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ (averageWinRate * 100).toFixed(1) }}%</div>
              <div class="stat-label">平均胜率</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="28"><VideoPlay /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ totalMatches }}</div>
              <div class="stat-label">总比赛场次</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 加载状态 -->
    <el-card v-if="isLoading" class="table-card">
      <el-skeleton :rows="10" animated />
    </el-card>

    <!-- 战队列表 -->
    <el-card v-else class="table-card">
      <el-table :data="paginatedTeams" stripe style="width: 100%">
        <el-table-column prop="name" label="战队名称" min-width="180">
          <template #default="{ row }">
            <div class="team-cell">
              <div class="team-avatar" :class="getRegionClass(row.region_id)">
                {{ row.short_name || row.name.substring(0, 2) }}
              </div>
              <div class="team-info">
                <div class="team-name">{{ row.name }}</div>
                <div class="team-region-text">{{ getRegionName(row.region_id) }}</div>
              </div>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="region_id" label="赛区" width="120">
          <template #default="{ row }">
            <el-tag :type="getRegionTagType(row.region_id)" size="default">
              {{ getRegionShortName(row.region_id) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="power_rating" label="战力值" width="180">
          <template #default="{ row }">
            <div class="strength-cell">
              <el-progress
                :percentage="row.power_rating"
                :color="getStrengthColor(row.power_rating)"
                :stroke-width="10"
              />
              <span class="strength-value">{{ row.power_rating.toFixed(2) }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="比赛数据" width="160">
          <template #default="{ row }">
            <div class="match-stats">
              <span>总场次: {{ row.total_matches }}</span>
              <div class="win-loss">
                <span class="wins">胜: {{ row.wins }}</span>
                <span class="losses">负: {{ row.total_matches - row.wins }}</span>
              </div>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="win_rate" label="胜率" width="100">
          <template #default="{ row }">
            <div class="win-rate" :class="getWinRateClass(row.win_rate)">
              {{ (row.win_rate * 100).toFixed(1) }}%
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="annual_points" label="年度积分" width="100">
          <template #default="{ row }">
            <span class="points">{{ row.annual_points }}</span>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="160" fixed="right">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button type="primary" size="small" :icon="View" @click="viewTeam(row)">
                详情
              </el-button>
              <el-button type="info" size="small" :icon="Edit" @click="editTeam(row)">
                编辑
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50]"
          :total="displayTeams.length"
          layout="total, sizes, prev, pager, next"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'Teams' })
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  Search,
  UserFilled,
  TrendCharts,
  Trophy,
  VideoPlay,
  View,
  Edit,
} from '@element-plus/icons-vue'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'

const router = useRouter()
const teamStore = useTeamStoreTauri()

// 从 store 获取响应式数据
const { regions, teams, isLoading } = storeToRefs(teamStore)

// 筛选条件
const searchQuery = ref('')
const selectedRegionId = ref(0)
const strengthRange = ref([0, 100])
const sortBy = ref('strength')
const currentPage = ref(1)
const pageSize = ref(20)

// 初始化加载数据
onMounted(async () => {
  await teamStore.loadRegions()
  // 加载所有赛区的队伍
  await teamStore.loadAllTeams()
})

// 监听赛区变化
const onRegionChange = async (regionId: number) => {
  if (regionId && regionId > 0) {
    await teamStore.selectRegion(regionId)
  } else {
    // 选择"全部赛区"时，重新加载所有队伍
    await teamStore.loadAllTeams()
  }
}

// 显示的队伍（经过筛选和排序）
const displayTeams = computed(() => {
  let result = [...teams.value]

  // 搜索筛选
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(t =>
      t.name.toLowerCase().includes(query) ||
      (t.short_name?.toLowerCase().includes(query) ?? false)
    )
  }

  // 战力值筛选
  result = result.filter(t =>
    t.power_rating >= strengthRange.value[0] &&
    t.power_rating <= strengthRange.value[1]
  )

  // 排序
  result.sort((a, b) => {
    if (sortBy.value === 'name') return a.name.localeCompare(b.name)
    if (sortBy.value === 'strength') return b.power_rating - a.power_rating
    if (sortBy.value === 'winRate') return b.win_rate - a.win_rate
    return 0
  })

  return result
})

// 分页后的队伍
const paginatedTeams = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return displayTeams.value.slice(start, end)
})

// 统计计算
const averageStrength = computed(() => {
  if (displayTeams.value.length === 0) return 0
  return displayTeams.value.reduce((sum, t) => sum + t.power_rating, 0) / displayTeams.value.length
})

const averageWinRate = computed(() => {
  if (displayTeams.value.length === 0) return 0
  return displayTeams.value.reduce((sum, t) => sum + t.win_rate, 0) / displayTeams.value.length
})

const totalMatches = computed(() =>
  displayTeams.value.reduce((sum, t) => sum + t.total_matches, 0)
)

// 筛选操作
const applyFilters = () => { currentPage.value = 1 }
const clearFilters = () => {
  searchQuery.value = ''
  selectedRegionId.value = 0
  strengthRange.value = [0, 100]
  sortBy.value = 'strength'
}

// 导航
const viewTeam = (team: any) => { router.push(`/teams/${team.id}`) }
const editTeam = (team: any) => { router.push(`/teams/${team.id}/edit`) }

// 辅助函数 - 赛区相关
const getRegionName = (regionId: number) => {
  const region = regions.value.find(r => r.id === regionId)
  return region?.name ?? '未知赛区'
}

const getRegionShortName = (regionId: number) => {
  const region = regions.value.find(r => r.id === regionId)
  return region?.code ?? '???'
}

const getRegionClass = (regionId: number) => {
  const region = regions.value.find(r => r.id === regionId)
  return region?.code?.toLowerCase() ?? 'unknown'
}

const getRegionTagType = (regionId: number) => {
  const region = regions.value.find(r => r.id === regionId)
  const code = region?.code ?? ''
  const types: Record<string, string> = { LPL: 'danger', LCK: 'primary', LEC: 'success', LCS: 'warning' }
  return types[code] || 'info'
}

// 辅助函数 - 样式
const getStrengthColor = (strength: number) => {
  if (strength >= 85) return '#67c23a'
  if (strength >= 75) return '#e6a23c'
  return '#f56c6c'
}

const getWinRateClass = (winRate: number) => {
  if (winRate >= 0.7) return 'high'
  if (winRate >= 0.5) return 'medium'
  return 'low'
}
</script>

<style scoped>
.team-list { padding: 0; }

.page-header { margin-bottom: 20px; }
.page-header h1 { font-size: 24px; font-weight: 700; color: #303133; margin: 0 0 8px 0; }
.page-header p { font-size: 14px; color: #909399; margin: 0; }

.filter-card { margin-bottom: 20px; }
.strength-filter { display: flex; align-items: center; gap: 12px; }
.strength-filter span { white-space: nowrap; color: #606266; font-size: 14px; }
.strength-filter .el-slider { flex: 1; }

.stats-row { margin-bottom: 20px; }
.stat-card { border-radius: 12px; }
.stat-content { display: flex; align-items: center; gap: 16px; padding: 8px 0; }
.stat-info { flex: 1; }
.stat-number { font-size: 28px; font-weight: 700; color: #303133; line-height: 1; }
.stat-label { font-size: 14px; color: #909399; margin-top: 4px; }

.table-card { border-radius: 12px; }
.team-cell { display: flex; align-items: center; gap: 12px; }
.team-avatar.unknown { background: linear-gradient(135deg, #95a5a6, #7f8c8d); }
.team-info { flex: 1; }
.team-name { font-weight: 600; color: #303133; }
.team-region-text { font-size: 12px; color: #909399; }

.strength-cell { display: flex; align-items: center; gap: 12px; }
.strength-cell .el-progress { flex: 1; }
.strength-value { font-weight: 600; min-width: 30px; text-align: right; }

.match-stats { font-size: 13px; color: #606266; }
.win-loss { display: flex; gap: 12px; margin-top: 4px; }
.wins { color: #67c23a; }
.losses { color: #f56c6c; }

.win-rate { font-size: 18px; font-weight: 700; text-align: center; }
.win-rate.high { color: #67c23a; }
.win-rate.medium { color: #e6a23c; }
.win-rate.low { color: #f56c6c; }

.points { font-weight: 600; color: #409eff; }

.action-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.pagination-wrapper { display: flex; justify-content: flex-end; margin-top: 20px; padding-top: 20px; border-top: 1px solid #ebeef5; }
</style>
