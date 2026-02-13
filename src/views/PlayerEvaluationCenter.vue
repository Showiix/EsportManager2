<template>
  <div class="evaluation-center">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>选手评估中心</h1>
        <div class="header-sub-row">
          <p>查看各选手的去留意愿、薪资评估与合同状态</p>
          <SeasonSelector v-model="selectedSeason" @update:model-value="onSeasonChange" width="140px" />
        </div>
      </div>
    </div>

    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ evaluations.length }}</span>
        <span class="stat-label">选手总数</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ wantToLeaveCount }}</span>
        <span class="stat-label">想离开</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ wantToStayCount }}</span>
        <span class="stat-label">愿留队</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ avgStayScore.toFixed(0) }}</span>
        <span class="stat-label">平均留队意愿</span>
      </div>
    </div>

    <!-- 筛选区域 -->
    <div class="filter-section">
      <div class="filter-row">
        <div class="filter-group">
          <label>搜索选手</label>
          <el-input
            v-model="filters.search"
            placeholder="输入选手名称..."
            :prefix-icon="Search"
            clearable
            style="width: 200px"
          />
        </div>
        <div class="filter-group">
          <label>位置</label>
          <el-select v-model="filters.position" placeholder="全部位置" clearable style="width: 100px">
            <el-option label="全部" value="" />
            <el-option label="上单" value="TOP" />
            <el-option label="打野" value="JUG" />
            <el-option label="中单" value="MID" />
            <el-option label="ADC" value="ADC" />
            <el-option label="辅助" value="SUP" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>赛区</label>
          <el-select v-model="filters.region" placeholder="全部赛区" clearable style="width: 100px">
            <el-option label="全部" value="" />
            <el-option label="LPL" value="LPL" />
            <el-option label="LCK" value="LCK" />
            <el-option label="LEC" value="LEC" />
            <el-option label="LCS" value="LCS" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>意愿</label>
          <el-select v-model="filters.wantToLeave" placeholder="全部" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="想离开" value="leave" />
            <el-option label="愿留队" value="stay" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>能力范围</label>
          <el-select v-model="filters.abilityRange" placeholder="全部" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="90+" value="90+" />
            <el-option label="80-89" value="80-89" />
            <el-option label="70-79" value="70-79" />
            <el-option label="<70" value="<70" />
          </el-select>
        </div>
        <el-button type="primary" :icon="Refresh" @click="loadEvaluations" :loading="loading">
          刷新数据
        </el-button>
      </div>
    </div>

    <!-- 空状态提示 -->
    <div v-if="!loading && evaluations.length === 0" class="empty-section">
      <el-empty description="还未开始转会期">
        <template #image>
          <el-icon :size="80" color="#c0c4cc"><Calendar /></el-icon>
        </template>
        <p class="empty-hint">选手评估数据将在转会期开始后生成</p>
      </el-empty>
    </div>

    <!-- 数据表格 -->
    <div v-else class="table-section">
      <el-table
        :data="paginatedEvaluations"
        v-loading="loading"
        style="width: 100%"
        :default-sort="{ prop: 'ability', order: 'descending' }"
        max-height="calc(100vh - 380px)"
      >
        <!-- 选手信息 -->
        <el-table-column label="选手" width="140" fixed>
          <template #default="{ row }">
            <div class="player-info">
              <el-tag :type="getPositionTagType(row.position)" size="small" effect="dark">
                {{ getPositionLabel(row.position) }}
              </el-tag>
              <span class="player-name">{{ row.player_name }}</span>
            </div>
          </template>
        </el-table-column>

        <!-- 战队 -->
        <el-table-column prop="team_name" label="战队" width="140" align="center">
          <template #default="{ row }">
            <div class="team-cell">
              <el-tag size="small" :type="getRegionTagType(row.region_code)">
                {{ normalizeRegionCode(row.region_code) }}
              </el-tag>
              <span class="team-name">{{ row.team_short_name || row.team_name }}</span>
            </div>
          </template>
        </el-table-column>

        <!-- 能力 -->
        <el-table-column prop="ability" label="能力" width="80" sortable align="center">
          <template #default="{ row }">
            <span :class="getAbilityClass(row.ability)">{{ row.ability }}</span>
          </template>
        </el-table-column>

        <!-- 年龄 -->
        <el-table-column prop="age" label="年龄" width="70" sortable align="center">
          <template #default="{ row }">
            <span :class="getAgeClass(row.age)">{{ row.age }}岁</span>
          </template>
        </el-table-column>

        <!-- 留队意愿 -->
        <el-table-column prop="stay_score" label="留队意愿" width="130" sortable align="center">
          <template #default="{ row }">
            <div class="stay-score-display">
              <el-progress
                :percentage="Math.min(row.stay_score, 100)"
                :stroke-width="8"
                :show-text="false"
                :color="getStayScoreColor(row.stay_score)"
                style="width: 60px; display: inline-block;"
              />
              <span :class="getStayScoreClass(row.stay_score)" style="margin-left: 6px;">
                {{ row.stay_score.toFixed(0) }}
              </span>
            </div>
          </template>
        </el-table-column>

        <!-- 状态 -->
        <el-table-column prop="wants_to_leave" label="状态" width="90" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.wants_to_leave" type="danger" size="small" effect="dark">想离开</el-tag>
            <el-tag v-else type="success" size="small">愿留队</el-tag>
          </template>
        </el-table-column>

        <!-- 当前薪资 -->
        <el-table-column prop="salary" label="薪资" width="120" sortable align="center">
          <template #default="{ row }">
            <span class="salary-value">{{ formatSalary(row.salary, false) }}</span>
          </template>
        </el-table-column>

        <!-- 满意度 -->
        <el-table-column prop="satisfaction" label="满意度" width="100" sortable align="center">
          <template #default="{ row }">
            <span :class="getSatisfactionClass(row.satisfaction)">{{ row.satisfaction }}</span>
          </template>
        </el-table-column>

        <!-- 忠诚度 -->
        <el-table-column prop="loyalty" label="忠诚度" width="100" sortable align="center">
          <template #default="{ row }">
            <span :class="getLoyaltyClass(row.loyalty)">{{ row.loyalty }}</span>
          </template>
        </el-table-column>

        <!-- 离队原因 -->
        <el-table-column prop="leave_reason" label="原因" min-width="200">
          <template #default="{ row }">
            <span class="reason-text">{{ row.leave_reason || '-' }}</span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div v-if="filteredEvaluations.length > pageSize" class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :page-sizes="[20, 50, 100]"
          :total="filteredEvaluations.length"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, Refresh, Calendar } from '@element-plus/icons-vue'
import {
  transferWindowApi,
  type PlayerStayEvaluationInfo,
} from '@/api/tauri'
import { formatSalary } from '@/utils/format'
import { useTimeStore } from '@/stores/useTimeStore'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

// 状态
const loading = ref(false)
const evaluations = ref<PlayerStayEvaluationInfo[]>([])
const currentPage = ref(1)
const pageSize = ref(20)
const timeStore = useTimeStore()
const selectedSeason = ref<number>(0)

// 筛选条件
const filters = reactive({
  search: '',
  position: '',
  region: '',
  wantToLeave: '',
  abilityRange: '',
})

// 计算属性
const wantToLeaveCount = computed(() =>
  evaluations.value.filter(e => e.wants_to_leave).length
)

const wantToStayCount = computed(() =>
  evaluations.value.filter(e => !e.wants_to_leave).length
)

const avgStayScore = computed(() => {
  if (evaluations.value.length === 0) return 0
  const sum = evaluations.value.reduce((acc, e) => acc + e.stay_score, 0)
  return sum / evaluations.value.length
})

const filteredEvaluations = computed(() => {
  return evaluations.value.filter(e => {
    if (filters.search && !e.player_name.toLowerCase().includes(filters.search.toLowerCase())) {
      return false
    }
    if (filters.position && e.position !== filters.position) {
      return false
    }
    if (filters.region && normalizeRegionCode(e.region_code) !== filters.region) {
      return false
    }
    if (filters.wantToLeave === 'leave' && !e.wants_to_leave) {
      return false
    }
    if (filters.wantToLeave === 'stay' && e.wants_to_leave) {
      return false
    }
    if (filters.abilityRange) {
      switch (filters.abilityRange) {
        case '90+':
          if (e.ability < 90) return false
          break
        case '80-89':
          if (e.ability < 80 || e.ability >= 90) return false
          break
        case '70-79':
          if (e.ability < 70 || e.ability >= 80) return false
          break
        case '<70':
          if (e.ability >= 70) return false
          break
      }
    }
    return true
  })
})

// 分页后的数据
const paginatedEvaluations = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredEvaluations.value.slice(start, end)
})

// 筛选变化时重置分页
watch([() => filters.search, () => filters.position, () => filters.region, () => filters.wantToLeave, () => filters.abilityRange], () => {
  currentPage.value = 1
})

// 分页处理
function handleSizeChange(size: number) {
  pageSize.value = size
  currentPage.value = 1
}

function handlePageChange(page: number) {
  currentPage.value = page
}

// 方法
async function loadEvaluations() {
  loading.value = true
  try {
    evaluations.value = await transferWindowApi.getPlayerStayEvaluations(
      undefined,
      selectedSeason.value || undefined
    )
  } catch (error) {
    ElMessage.error('加载选手评估数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

function onSeasonChange() {
  loadEvaluations()
}

function getRegionTagType(region: string): string {
  const types: Record<string, string> = {
    LPL: 'danger', CN: 'danger',
    LCK: 'primary', KR: 'primary',
    LEC: 'success', EU: 'success',
    LCS: 'warning', NA: 'warning',
  }
  return types[region] || 'info'
}

function normalizeRegionCode(region: string): string {
  const mapping: Record<string, string> = { CN: 'LPL', KR: 'LCK', EU: 'LEC', NA: 'LCS' }
  return mapping[region] || region
}

// 样式辅助函数
function getPositionLabel(position: string): string {
  const labels: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: 'ADC',
    SUP: '辅助',
  }
  return labels[position] || position
}

function getPositionTagType(position: string): string {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'success',
    MID: 'primary',
    ADC: 'warning',
    SUP: 'info',
  }
  return types[position] || 'info'
}

function getAbilityClass(ability: number): string {
  if (ability >= 90) return 'ability-elite'
  if (ability >= 80) return 'ability-good'
  if (ability >= 70) return 'ability-average'
  return 'ability-low'
}

function getAgeClass(age: number): string {
  if (age <= 22) return 'age-young'
  if (age <= 26) return 'age-prime'
  return 'age-old'
}

function getStayScoreClass(score: number): string {
  if (score >= 70) return 'stay-high'
  if (score >= 40) return 'stay-medium'
  return 'stay-low'
}

function getStayScoreColor(score: number): string {
  if (score >= 70) return '#67c23a'
  if (score >= 40) return '#e6a23c'
  return '#f56c6c'
}

function getSatisfactionClass(satisfaction: number): string {
  if (satisfaction >= 80) return 'satisfaction-high'
  if (satisfaction >= 60) return 'satisfaction-medium'
  return 'satisfaction-low'
}

function getLoyaltyClass(loyalty: number): string {
  if (loyalty >= 80) return 'loyalty-high'
  if (loyalty >= 60) return 'loyalty-medium'
  return 'loyalty-low'
}

// 初始化
onMounted(async () => {
  selectedSeason.value = timeStore.currentSeasonFromTime
  loadEvaluations()
})
</script>

<style scoped>
.evaluation-center { padding: 0; }

.page-header { margin-bottom: 20px; }
.page-header h1 { font-size: 24px; font-weight: 700; color: #0f172a; margin: 0 0 4px 0; }
.page-header p { font-size: 13px; color: #94a3b8; margin: 0; }
.header-sub-row { display: flex; align-items: center; gap: 12px; }

.stats-bar { display: flex; align-items: center; padding: 14px 24px; background: #fff; border: 1px solid #e2e8f0; border-radius: 10px; margin-bottom: 12px; }
.stat-item { display: flex; align-items: baseline; gap: 6px; flex: 1; justify-content: center; }
.stat-value { font-size: 20px; font-weight: 700; color: #0f172a; font-variant-numeric: tabular-nums; }
.stat-label { font-size: 12px; color: #94a3b8; font-weight: 500; }
.stat-divider { width: 1px; height: 24px; background: #e2e8f0; flex-shrink: 0; }

.filter-section { margin-bottom: 16px; }
.filter-row { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; }
.filter-group { display: flex; align-items: center; gap: 6px; }
.filter-group label { font-size: 12px; color: #94a3b8; font-weight: 500; white-space: nowrap; }

.empty-section { border: 1px solid #e2e8f0; border-radius: 10px; padding: 40px 0; text-align: center; margin-bottom: 16px; }
.empty-hint { color: #94a3b8; font-size: 13px; margin-top: 12px; }

.table-section { border: 1px solid #e2e8f0; border-radius: 10px; padding: 16px; margin-bottom: 16px; }
.table-section :deep(.el-table th.el-table__cell) { font-weight: 600; color: #94a3b8; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; background: transparent; border-bottom: 1px solid #f1f5f9; padding: 10px 0; }
.table-section :deep(.el-table__body tr) { transition: background-color 0.15s; }
.table-section :deep(.el-table__body tr td) { padding: 12px 0; border-bottom: 1px solid #f8fafc; }
.table-section :deep(.el-table__body tr:hover > td) { background-color: #f8fafc !important; }
.table-section :deep(.el-table__body tr:last-child td) { border-bottom: none; }

.player-info { display: flex; align-items: center; gap: 8px; }
.player-name { font-weight: 600; font-size: 13px; color: #0f172a; }
.team-cell { display: flex; align-items: center; justify-content: center; gap: 6px; }
.team-cell :deep(.el-tag) { min-width: 36px; text-align: center; }
.team-name { color: #64748b; font-size: 13px; }

.salary-value { color: #f59e0b; font-weight: 600; font-size: 13px; }
.ability-elite { color: #ef4444; font-weight: 700; }
.ability-good { color: #f59e0b; font-weight: 600; }
.ability-average { color: #3b82f6; }
.ability-low { color: #94a3b8; }
.age-young { color: #22c55e; }
.age-prime { color: #3b82f6; }
.age-old { color: #f59e0b; }
.stay-high { color: #22c55e; font-weight: 600; }
.stay-medium { color: #f59e0b; font-weight: 600; }
.stay-low { color: #ef4444; font-weight: 600; }
.satisfaction-high { color: #22c55e; }
.satisfaction-medium { color: #f59e0b; }
.satisfaction-low { color: #ef4444; }
.loyalty-high { color: #22c55e; }
.loyalty-medium { color: #f59e0b; }
.loyalty-low { color: #ef4444; }
.reason-text { font-size: 12px; color: #64748b; }
.stay-score-display { display: flex; align-items: center; }

.pagination-wrapper { display: flex; justify-content: center; margin-top: 16px; }

@media (max-width: 1200px) {
  .stats-bar { flex-wrap: wrap; gap: 8px; }
  .stat-divider { display: none; }
  .filter-row { flex-direction: column; align-items: stretch; }
}
</style>
