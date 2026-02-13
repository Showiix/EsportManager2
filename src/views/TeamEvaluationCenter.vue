<template>
  <div class="evaluation-center">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>战队评估中心</h1>
        <div class="header-sub-row">
          <p>查看各战队的赛季评估、策略分析与阵容需求</p>
          <SeasonSelector v-model="selectedSeason" @update:model-value="onSeasonChange" width="140px" />
        </div>
      </div>
    </div>

    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ evaluations.length }}</span>
        <span class="stat-label">战队总数</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ dynastyCount }}</span>
        <span class="stat-label">王朝战队</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ rebuildCount }}</span>
        <span class="stat-label">重建战队</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ avgStability.toFixed(0) }}</span>
        <span class="stat-label">平均稳定性</span>
      </div>
    </div>

    <!-- 筛选区域 -->
    <div class="filter-section">
      <div class="filter-row">
        <div class="filter-group">
          <label>搜索战队</label>
          <el-input
            v-model="filters.search"
            placeholder="输入战队名称..."
            :prefix-icon="Search"
            clearable
            style="width: 200px"
          />
        </div>
        <div class="filter-group">
          <label>赛区</label>
          <el-select v-model="filters.region" placeholder="全部赛区" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="LPL" value="LPL" />
            <el-option label="LCK" value="LCK" />
            <el-option label="LEC" value="LEC" />
            <el-option label="LCS" value="LCS" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>战略类型</label>
          <el-select v-model="filters.strategy" placeholder="全部类型" clearable style="width: 140px">
            <el-option label="全部" value="" />
            <el-option label="王朝维持" value="DYNASTY" />
            <el-option label="稳定发展" value="MAINTAIN" />
            <el-option label="阵容补强" value="UPGRADE" />
            <el-option label="全面重建" value="REBUILD" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>紧迫程度</label>
          <el-select v-model="filters.urgency" placeholder="全部" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="无需调整" value="NONE" />
            <el-option label="低优先级" value="LOW" />
            <el-option label="中等优先" value="MEDIUM" />
            <el-option label="紧急调整" value="HIGH" />
          </el-select>
        </div>
        <el-button type="primary" :icon="Refresh" @click="loadEvaluations" :loading="loading">
          刷新数据
        </el-button>
        <el-button type="danger" :icon="Delete" @click="handleClearData" :loading="clearing" plain>
          清除评估数据
        </el-button>
      </div>
    </div>

    <!-- 空状态提示 -->
    <div v-if="!loading && evaluations.length === 0" class="empty-section">
      <el-empty description="还未开始转会期">
        <template #image>
          <el-icon :size="80" color="#c0c4cc"><Calendar /></el-icon>
        </template>
        <p class="empty-hint">战队评估数据将在转会期开始后生成</p>
      </el-empty>
    </div>

    <!-- 数据表格 -->
    <div v-else class="table-section">
      <el-table
        :data="paginatedEvaluations"
        v-loading="loading"
        style="width: 100%"
        :default-sort="{ prop: 'stability_score', order: 'descending' }"
        @row-click="handleRowClick"
        max-height="calc(100vh - 320px)"
      >
        <!-- 战队信息 -->
        <el-table-column label="战队" width="180" fixed>
          <template #default="{ row }">
            <div class="team-info">
              <el-tag size="small" :type="getRegionTagType(row.region_code)">
                {{ normalizeRegionCode(row.region_code) }}
              </el-tag>
              <span class="team-name">{{ row.team_short_name }}</span>
            </div>
          </template>
        </el-table-column>

        <!-- 排名变化 -->
        <el-table-column label="排名变化" width="120" align="center">
          <template #default="{ row }">
            <div class="rank-change">
              <span class="rank-number">{{ row.current_rank }}</span>
              <span :class="getRankChangeClass(row.current_rank, row.last_rank)">
                {{ getRankChangeText(row.current_rank, row.last_rank) }}
              </span>
            </div>
          </template>
        </el-table-column>

        <!-- 春季赛排名 -->
        <el-table-column prop="spring_rank" label="春季赛" width="90" sortable align="center">
          <template #default="{ row }">
            <span v-if="row.spring_rank" class="season-rank">{{ row.spring_rank }}</span>
            <span v-else class="rank-na">-</span>
          </template>
        </el-table-column>

        <!-- 夏季赛排名 -->
        <el-table-column prop="summer_rank" label="夏季赛" width="90" sortable align="center">
          <template #default="{ row }">
            <span v-if="row.summer_rank" class="season-rank">{{ row.summer_rank }}</span>
            <span v-else class="rank-na">-</span>
          </template>
        </el-table-column>

        <!-- 稳定性评分 -->
        <el-table-column prop="stability_score" label="稳定性" width="120" sortable align="center">
          <template #default="{ row }">
            <div class="stability-display">
              <span class="stability-value" :class="getStabilityClass(row.stability_score)">
                {{ row.stability_score }}
              </span>
              <el-progress
                :percentage="row.stability_score"
                :stroke-width="4"
                :show-text="false"
                :color="getStabilityColor(row.stability_score)"
              />
            </div>
          </template>
        </el-table-column>

        <!-- 战略类型 -->
        <el-table-column prop="strategy" label="战略" width="120" align="center">
          <template #default="{ row }">
            <el-tag :type="getStrategyTagType(row.strategy)" effect="dark">
              {{ getStrategyLabel(row.strategy) }}
            </el-tag>
          </template>
        </el-table-column>

        <!-- 紧迫程度 -->
        <el-table-column prop="urgency_level" label="紧迫度" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getUrgencyTagType(row.urgency_level)" size="small">
              {{ getUrgencyLabel(row.urgency_level) }}
            </el-tag>
          </template>
        </el-table-column>

        <!-- 阵容实力 -->
        <el-table-column prop="roster_power" label="阵容实力" width="100" sortable align="center">
          <template #default="{ row }">
            <span class="power-value">{{ row.roster_power.toFixed(1) }}</span>
          </template>
        </el-table-column>

        <!-- 阵容规模 -->
        <el-table-column prop="roster_count" label="人数" width="80" align="center">
          <template #default="{ row }">
            <span>{{ row.roster_count }}</span>
          </template>
        </el-table-column>

        <!-- 平均年龄 -->
        <el-table-column prop="avg_age" label="平均年龄" width="100" sortable align="center">
          <template #default="{ row }">
            <span :class="getAgeClass(row.avg_age)">{{ row.avg_age.toFixed(1) }}岁</span>
          </template>
        </el-table-column>

        <!-- 平均能力 -->
        <el-table-column prop="avg_ability" label="平均能力" width="100" sortable align="center">
          <template #default="{ row }">
            <span class="ability-value" :class="getAbilityClass(row.avg_ability)">
              {{ row.avg_ability.toFixed(1) }}
            </span>
          </template>
        </el-table-column>

        <!-- 剩余预算 -->
        <el-table-column prop="budget_remaining" label="剩余预算" width="120" sortable align="center">
          <template #default="{ row }">
            <span :class="getBudgetClass(row.budget_remaining)">
              {{ formatMoney(row.budget_remaining) }}
            </span>
          </template>
        </el-table-column>

        <!-- 操作 -->
        <el-table-column label="操作" width="100" fixed="right" align="center">
          <template #default="{ row }">
            <el-button type="primary" link @click.stop="showDetail(row)">
              查看详情
            </el-button>
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

    <!-- 战队详情弹窗 -->
    <el-dialog
      v-model="detailDialogVisible"
      width="800px"
      :close-on-click-modal="true"
      class="team-detail-modal"
    >
      <template #header>
        <div class="dialog-header">
          <div class="team-title">
            <el-tag :type="getRegionTagType(selectedTeam?.region_code || '')" size="large">
              {{ normalizeRegionCode(selectedTeam?.region_code || '') }}
            </el-tag>
            <h3>{{ selectedTeam?.team_short_name }}</h3>
          </div>
          <div class="team-badges">
            <el-tag :type="getStrategyTagType(selectedTeam?.strategy || '')" effect="dark" size="large">
              {{ getStrategyLabel(selectedTeam?.strategy || '') }}
            </el-tag>
            <el-tag :type="getUrgencyTagType(selectedTeam?.urgency_level || '')" size="large">
              {{ getUrgencyLabel(selectedTeam?.urgency_level || '') }}
            </el-tag>
          </div>
        </div>
      </template>

      <div v-if="selectedTeam" class="team-detail-content">
        <!-- 核心指标卡片 -->
        <div class="metrics-row">
          <div class="metric-card">
            <div class="metric-icon rank">
              <span class="rank-value">{{ selectedTeam.current_rank }}</span>
            </div>
            <div class="metric-info">
              <span class="metric-label">当前排名</span>
              <span class="metric-change" :class="getRankChangeClass(selectedTeam.current_rank, selectedTeam.last_rank)">
                {{ getRankChangeText(selectedTeam.current_rank, selectedTeam.last_rank) }} 上赛季第{{ selectedTeam.last_rank }}名
              </span>
              <span class="metric-sub" v-if="selectedTeam.spring_rank || selectedTeam.summer_rank">
                <template v-if="selectedTeam.spring_rank">春{{ selectedTeam.spring_rank }}</template>
                <template v-if="selectedTeam.spring_rank && selectedTeam.summer_rank"> · </template>
                <template v-if="selectedTeam.summer_rank">夏{{ selectedTeam.summer_rank }}</template>
              </span>
            </div>
          </div>
          <div class="metric-card">
            <div class="metric-icon stability" :class="getStabilityClass(selectedTeam.stability_score)">
              {{ selectedTeam.stability_score }}
            </div>
            <div class="metric-info">
              <span class="metric-label">稳定性评分</span>
              <el-progress
                :percentage="selectedTeam.stability_score"
                :stroke-width="6"
                :show-text="false"
                :color="getStabilityColor(selectedTeam.stability_score)"
              />
            </div>
          </div>
          <div class="metric-card">
            <div class="metric-icon power">
              {{ selectedTeam.roster_power.toFixed(1) }}
            </div>
            <div class="metric-info">
              <span class="metric-label">阵容实力</span>
              <span class="metric-sub">{{ selectedTeam.roster_count }}人 · {{ selectedTeam.avg_age.toFixed(1) }}岁</span>
            </div>
          </div>
          <div class="metric-card">
            <div class="metric-icon budget" :class="getBudgetClass(selectedTeam.budget_remaining)">
              {{ formatMoney(selectedTeam.budget_remaining) }}
            </div>
            <div class="metric-info">
              <span class="metric-label">剩余预算</span>
              <span class="metric-sub">可用于转会</span>
            </div>
          </div>
        </div>

        <!-- 战略说明 -->
        <div class="strategy-section" v-if="selectedTeam.evaluation_reason">
          <div class="strategy-icon" :class="selectedTeam.strategy.toLowerCase()">
            <el-icon :size="20"><TrendCharts /></el-icon>
          </div>
          <div class="strategy-text">
            <span class="strategy-title">{{ getStrategyDescription(selectedTeam.strategy) }}</span>
            <span class="strategy-reason">{{ selectedTeam.evaluation_reason }}</span>
          </div>
        </div>

        <!-- 位置需求 -->
        <div class="section-card" v-if="positionNeeds.length > 0">
          <div class="section-header">
            <h4>位置需求分析</h4>
            <span class="section-badge">{{ positionNeeds.length }}个位置</span>
          </div>
          <div class="position-grid">
            <div
              v-for="need in positionNeeds"
              :key="need.position"
              class="position-card"
              :class="need.need_level.toLowerCase()"
            >
              <div class="position-header">
                <el-tag :type="getPositionTagType(need.position)" effect="dark">
                  {{ getPositionLabel(need.position) }}
                </el-tag>
                <el-tag :type="getNeedLevelTagType(need.need_level)" size="small">
                  {{ getNeedLevelLabel(need.need_level) }}
                </el-tag>
              </div>
              <div class="position-body">
                <div v-if="need.current_starter_name" class="current-starter">
                  <span class="starter-name">{{ need.current_starter_name }}</span>
                  <span class="starter-stats">{{ need.current_starter_ability }} · {{ need.current_starter_age }}岁</span>
                </div>
                <div v-else class="no-starter">
                  <span>暂无首发</span>
                </div>
                <div class="position-reason" v-if="need.reason">{{ need.reason }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 选手评估 -->
        <div class="section-card" v-if="playerEvaluations.length > 0">
          <div class="section-header">
            <h4>选手去留评估</h4>
            <span class="section-badge">{{ playerEvaluations.filter(p => p.wants_to_leave).length }}人想离开</span>
          </div>
          <el-table :data="playerEvaluations" size="small" stripe max-height="300">
            <el-table-column prop="player_name" label="选手" width="100" />
            <el-table-column prop="position" label="位置" width="70" align="center">
              <template #default="{ row }">
                <el-tag :type="getPositionTagType(row.position)" size="small">
                  {{ row.position }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="ability" label="能力" width="60" align="center" />
            <el-table-column prop="stay_score" label="留队意愿" width="100" align="center">
              <template #default="{ row }">
                <el-progress
                  :percentage="Math.min(row.stay_score, 100)"
                  :stroke-width="8"
                  :show-text="false"
                  :color="getStayScoreColor(row.stay_score)"
                  style="width: 60px; display: inline-block;"
                />
                <span :class="getStayScoreClass(row.stay_score)" style="margin-left: 4px;">
                  {{ row.stay_score.toFixed(0) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="wants_to_leave" label="状态" width="90" align="center">
              <template #default="{ row }">
                <el-tag v-if="row.wants_to_leave" type="danger" size="small" effect="dark">想离开</el-tag>
                <el-tag v-else type="success" size="small">愿留队</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="leave_reason" label="原因">
              <template #default="{ row }">
                <span class="reason-text">{{ row.leave_reason || '-' }}</span>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>

      <template #footer>
        <el-button @click="detailDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, Refresh, Calendar, TrendCharts, Delete } from '@element-plus/icons-vue'
import { ElMessageBox } from 'element-plus'
import {
  transferWindowApi,
  type TeamSeasonEvaluationInfo,
  type PositionNeedInfo,
  type PlayerStayEvaluationInfo,
} from '@/api/tauri'
import { formatBudget } from '@/utils'
import { useTimeStore } from '@/stores/useTimeStore'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

// 状态
const loading = ref(false)
const clearing = ref(false)
const evaluations = ref<TeamSeasonEvaluationInfo[]>([])
const positionNeeds = ref<PositionNeedInfo[]>([])
const playerEvaluations = ref<PlayerStayEvaluationInfo[]>([])
const detailDialogVisible = ref(false)
const selectedTeam = ref<TeamSeasonEvaluationInfo | null>(null)
const currentPage = ref(1)
const pageSize = ref(20)
const timeStore = useTimeStore()
const selectedSeason = ref<number>(0)

// 筛选条件
const filters = reactive({
  search: '',
  region: '',
  strategy: '',
  urgency: '',
})

// 计算属性
const dynastyCount = computed(() =>
  evaluations.value.filter(e => e.strategy === 'DYNASTY').length
)

const rebuildCount = computed(() =>
  evaluations.value.filter(e => e.strategy === 'REBUILD').length
)

const avgStability = computed(() => {
  if (evaluations.value.length === 0) return 0
  const sum = evaluations.value.reduce((acc, e) => acc + e.stability_score, 0)
  return sum / evaluations.value.length
})

const filteredEvaluations = computed(() => {
  return evaluations.value.filter(e => {
    if (filters.search && !e.team_name.toLowerCase().includes(filters.search.toLowerCase())) {
      return false
    }
    if (filters.region && e.region_code !== filters.region) {
      return false
    }
    if (filters.strategy && e.strategy !== filters.strategy) {
      return false
    }
    if (filters.urgency && e.urgency_level !== filters.urgency) {
      return false
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
watch([() => filters.search, () => filters.region, () => filters.strategy, () => filters.urgency], () => {
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
    evaluations.value = await transferWindowApi.getTeamEvaluations(
      selectedSeason.value || undefined
    )
  } catch (error) {
    ElMessage.error('加载战队评估数据失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

function onSeasonChange() {
  loadEvaluations()
}

async function handleClearData() {
  try {
    await ElMessageBox.confirm(
      '确定要清除当前赛季的评估数据吗？清除后可重新执行转会期生成新的评估。',
      '确认清除',
      {
        confirmButtonText: '确认清除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    clearing.value = true
    const count = await transferWindowApi.clearEvaluationData()
    ElMessage.success(`已清除 ${count} 条评估数据`)
    evaluations.value = []
    positionNeeds.value = []
    playerEvaluations.value = []
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('清除评估数据失败')
      console.error(error)
    }
  } finally {
    clearing.value = false
  }
}

function handleRowClick(row: TeamSeasonEvaluationInfo) {
  showDetail(row)
}

async function showDetail(team: TeamSeasonEvaluationInfo) {
  selectedTeam.value = team
  detailDialogVisible.value = true

  // 加载位置需求
  try {
    positionNeeds.value = await transferWindowApi.getTeamPositionNeeds(team.team_id)
  } catch (error) {
    console.error('加载位置需求失败', error)
    positionNeeds.value = []
  }

  // 加载选手评估
  try {
    playerEvaluations.value = await transferWindowApi.getPlayerStayEvaluations(
      team.team_id,
      selectedSeason.value || undefined
    )
  } catch (error) {
    console.error('加载选手评估失败', error)
    playerEvaluations.value = []
  }
}

function formatMoney(value: number): string {
  return formatBudget(value)
}

// 样式辅助函数
function getRegionTagType(region: string): string {
  const types: Record<string, string> = {
    LPL: 'danger',
    CN: 'danger',
    LCK: 'primary',
    KR: 'primary',
    LEC: 'success',
    EU: 'success',
    LCS: 'warning',
    NA: 'warning',
  }
  return types[region] || 'info'
}

function normalizeRegionCode(region: string): string {
  const mapping: Record<string, string> = {
    CN: 'LPL',
    KR: 'LCK',
    EU: 'LEC',
    NA: 'LCS',
  }
  return mapping[region] || region
}

function getRankChangeClass(current: number, last: number): string {
  if (current < last) return 'rank-up'
  if (current > last) return 'rank-down'
  return 'rank-same'
}

function getRankChangeText(current: number, last: number): string {
  const diff = last - current
  if (diff > 0) return `↑${diff}`
  if (diff < 0) return `↓${Math.abs(diff)}`
  return '→'
}

function getStabilityClass(score: number): string {
  if (score >= 80) return 'stability-high'
  if (score >= 50) return 'stability-medium'
  return 'stability-low'
}

function getStabilityColor(score: number): string {
  if (score >= 80) return '#67c23a'
  if (score >= 50) return '#e6a23c'
  return '#f56c6c'
}

function getStrategyTagType(strategy: string): string {
  const types: Record<string, string> = {
    DYNASTY: 'success',
    MAINTAIN: 'primary',
    UPGRADE: 'warning',
    REBUILD: 'danger',
  }
  return types[strategy] || 'info'
}

function getStrategyLabel(strategy: string): string {
  const labels: Record<string, string> = {
    DYNASTY: '王朝维持',
    MAINTAIN: '稳定发展',
    UPGRADE: '阵容补强',
    REBUILD: '全面重建',
  }
  return labels[strategy] || strategy
}

function getStrategyDescription(strategy: string): string {
  const descriptions: Record<string, string> = {
    DYNASTY: '战队处于巅峰状态，战绩稳定优异，无需大规模调整，专注于维持现有阵容和化学反应。',
    MAINTAIN: '战队表现尚可，可进行小幅调整以保持竞争力，不宜大动干戈。',
    UPGRADE: '战队需要补强，可能存在薄弱位置或关键选手状态下滑，需要引进新血液。',
    REBUILD: '战队需要全面重建，可能因战绩大幅下滑、阵容老化等原因需要进行大规模调整。',
  }
  return descriptions[strategy] || ''
}

function getUrgencyTagType(urgency: string): string {
  const types: Record<string, string> = {
    NONE: 'success',
    LOW: 'info',
    MEDIUM: 'warning',
    HIGH: 'danger',
  }
  return types[urgency] || 'info'
}

function getUrgencyLabel(urgency: string): string {
  const labels: Record<string, string> = {
    NONE: '无需调整',
    LOW: '低',
    MEDIUM: '中等',
    HIGH: '紧急',
  }
  return labels[urgency] || urgency
}

function getAgeClass(age: number): string {
  if (age <= 22) return 'age-young'
  if (age <= 26) return 'age-prime'
  return 'age-old'
}

function getAbilityClass(ability: number): string {
  if (ability >= 85) return 'ability-elite'
  if (ability >= 75) return 'ability-good'
  if (ability >= 65) return 'ability-average'
  return 'ability-low'
}

function getBudgetClass(budget: number): string {
  if (budget >= 5000) return 'budget-high'
  if (budget >= 2000) return 'budget-medium'
  return 'budget-low'
}

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

function getNeedLevelTagType(level: string): string {
  const types: Record<string, string> = {
    CRITICAL: 'danger',
    HIGH: 'warning',
    MEDIUM: 'info',
    LOW: 'success',
  }
  return types[level] || 'info'
}

function getNeedLevelLabel(level: string): string {
  const labels: Record<string, string> = {
    CRITICAL: '紧急需求',
    HIGH: '高需求',
    MEDIUM: '中等需求',
    LOW: '低需求',
  }
  return labels[level] || level
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

// 初始化
onMounted(async () => {
  selectedSeason.value = timeStore.currentSeasonFromTime
  loadEvaluations()
})
</script>

<style scoped>
/* ===== Layout ===== */
.evaluation-center {
  padding: 0;
  min-height: 100vh;
}

/* ===== Page Header ===== */
.page-header {
  margin-bottom: 16px;
}

.page-header h1 {
  margin: 0 0 6px 0;
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.page-header p {
  margin: 0;
  font-size: 13px;
  color: #64748b;
}

.header-sub-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* ===== Stats Bar ===== */
.stats-bar {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 14px 24px;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
}

.stat-label {
  font-size: 12px;
  color: #94a3b8;
}

.stat-divider {
  width: 1px;
  height: 24px;
  background: #e2e8f0;
}

/* ===== Filter Section ===== */
.filter-section {
  margin-bottom: 16px;
}

.filter-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.filter-group label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 500;
  white-space: nowrap;
}

/* ===== Table Section ===== */
.table-section {
  margin-bottom: 20px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  background: #fff;
}

.table-section :deep(.el-table) {
  --el-table-border-color: #f1f5f9;
  --el-table-row-hover-bg-color: #f8fafc;
}

.table-section :deep(.el-table th.el-table__cell) {
  background: transparent;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  padding: 10px 0;
  border-bottom: 1px solid #e2e8f0;
}

.table-section :deep(.el-table td.el-table__cell) {
  padding: 8px 0;
  color: #0f172a;
  border-bottom: 1px solid #f1f5f9;
}

.table-section :deep(.el-table__body tr:hover > td.el-table__cell) {
  background: #f8fafc;
}

.table-section :deep(.el-table--enable-row-hover .el-table__body tr:hover > td) {
  background-color: #f8fafc;
}

/* ===== Team Info ===== */
.team-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-name {
  font-weight: 500;
  color: #0f172a;
}

/* ===== Rank ===== */
.rank-change {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 6px;
}

.rank-number {
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.rank-up {
  color: #22c55e;
  font-size: 12px;
}

.rank-down {
  color: #ef4444;
  font-size: 12px;
}

.rank-same {
  color: #94a3b8;
  font-size: 12px;
}

.season-rank {
  font-weight: 600;
  color: #0f172a;
}

.rank-na {
  color: #cbd5e1;
}

/* ===== Stability ===== */
.stability-display {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stability-value {
  font-weight: 600;
}

.stability-high {
  color: #22c55e;
}

.stability-medium {
  color: #f59e0b;
}

.stability-low {
  color: #ef4444;
}

/* ===== Power ===== */
.power-value {
  font-weight: 500;
  color: #3b82f6;
}

/* ===== Age ===== */
.age-young {
  color: #22c55e;
}

.age-prime {
  color: #3b82f6;
}

.age-old {
  color: #f59e0b;
}

/* ===== Ability ===== */
.ability-elite {
  color: #ef4444;
  font-weight: 600;
}

.ability-good {
  color: #f59e0b;
}

.ability-average {
  color: #94a3b8;
}

.ability-low {
  color: #cbd5e1;
}

/* ===== Budget ===== */
.budget-high {
  color: #22c55e;
}

.budget-medium {
  color: #f59e0b;
}

.budget-low {
  color: #ef4444;
}

/* ===== Empty Section ===== */
.empty-section {
  text-align: center;
  padding: 60px 0;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #fff;
  margin-bottom: 20px;
}

.empty-hint {
  color: #94a3b8;
  font-size: 14px;
  margin-top: 12px;
}

/* ===== Pagination ===== */
.pagination-wrapper {
  display: flex;
  justify-content: center;
  margin-top: 16px;
  padding: 16px;
}

/* ===== Stay Score ===== */
.stay-high {
  color: #22c55e;
  font-weight: 500;
}

.stay-medium {
  color: #f59e0b;
  font-weight: 500;
}

.stay-low {
  color: #ef4444;
  font-weight: 500;
}

.reason-text {
  font-size: 12px;
  color: #64748b;
}

.empty-text {
  color: #cbd5e1;
  font-style: italic;
}

.gap-negative {
  color: #ef4444;
}

.gap-positive {
  color: #22c55e;
}

/* ===== Dialog ===== */
.team-detail-modal :deep(.el-dialog__header) {
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
  margin-right: 0;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.team-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.team-title h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.team-badges {
  display: flex;
  gap: 8px;
}

.team-detail-content {
  max-height: 65vh;
  overflow-y: auto;
  padding-right: 8px;
}

/* ===== Metrics Row (Dialog) ===== */
.metrics-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 20px;
  background: #0f172a;
  border-radius: 10px;
  padding: 16px;
}

.metric-card {
  background: rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  gap: 12px;
  align-items: center;
}

.metric-icon {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 700;
  color: white;
  background: #3b82f6;
  flex-shrink: 0;
}

.metric-icon.rank {
  background: #6366f1;
}

.metric-icon.stability {
  background: #22c55e;
}

.metric-icon.stability.stability-medium {
  background: #f59e0b;
}

.metric-icon.stability.stability-low {
  background: #ef4444;
}

.metric-icon.power {
  background: #3b82f6;
}

.metric-icon.budget {
  background: #22c55e;
  font-size: 12px;
}

.metric-icon.budget.budget-medium {
  background: #f59e0b;
}

.metric-icon.budget.budget-low {
  background: #ef4444;
}

.rank-value {
  font-size: 20px;
}

.metric-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.metric-label {
  font-size: 12px;
  color: #94a3b8;
}

.metric-change {
  font-size: 11px;
  color: #64748b;
}

.metric-sub {
  font-size: 11px;
  color: #64748b;
}

/* ===== Strategy Section (Dialog) ===== */
.strategy-section {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: #f8fafc;
  border-radius: 10px;
  margin-bottom: 20px;
  align-items: flex-start;
}

.strategy-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.strategy-icon.dynasty {
  background: #22c55e;
}

.strategy-icon.maintain {
  background: #3b82f6;
}

.strategy-icon.upgrade {
  background: #f59e0b;
}

.strategy-icon.rebuild {
  background: #ef4444;
}

.strategy-text {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.strategy-title {
  font-size: 14px;
  color: #0f172a;
  font-weight: 500;
}

.strategy-reason {
  font-size: 13px;
  color: #64748b;
  line-height: 1.5;
}

/* ===== Section Card (Dialog) ===== */
.section-card {
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h4 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #0f172a;
}

.section-badge {
  font-size: 12px;
  color: #94a3b8;
  background: #f1f5f9;
  padding: 2px 8px;
  border-radius: 10px;
}

/* ===== Position Grid (Dialog) ===== */
.position-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 10px;
}

.position-card {
  background: #f8fafc;
  border-radius: 8px;
  padding: 10px;
  border-left: 3px solid #cbd5e1;
}

.position-card.critical {
  border-left-color: #ef4444;
  background: #f8fafc;
}

.position-card.high {
  border-left-color: #f59e0b;
  background: #f8fafc;
}

.position-card.medium {
  border-left-color: #3b82f6;
  background: #f8fafc;
}

.position-card.low {
  border-left-color: #22c55e;
  background: #f8fafc;
}

.position-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.position-body {
  font-size: 12px;
}

.current-starter {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.starter-name {
  color: #0f172a;
  font-weight: 500;
}

.starter-stats {
  color: #94a3b8;
  font-size: 11px;
}

.no-starter {
  color: #cbd5e1;
  font-style: italic;
}

.position-reason {
  margin-top: 6px;
  color: #64748b;
  font-size: 11px;
  line-height: 1.4;
}
</style>
