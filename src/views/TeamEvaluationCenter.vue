<template>
  <div class="evaluation-center">
    <!-- 页面标题 -->
    <div class="page-header">
      <div class="header-content">
        <h1>战队评估中心</h1>
        <p>查看各战队的赛季评估、策略分析与阵容需求</p>
      </div>
      <div class="header-stats">
        <div class="stat-item">
          <span class="stat-value">{{ evaluations.length }}</span>
          <span class="stat-label">战队总数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ dynastyCount }}</span>
          <span class="stat-label">王朝战队</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ rebuildCount }}</span>
          <span class="stat-label">重建战队</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ avgStability.toFixed(0) }}</span>
          <span class="stat-label">平均稳定性</span>
        </div>
      </div>
    </div>

    <!-- 筛选区域 -->
    <el-card class="filter-card">
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
      </div>
    </el-card>

    <!-- 数据表格 -->
    <el-card class="table-card">
      <el-table
        :data="filteredEvaluations"
        v-loading="loading"
        stripe
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
                {{ row.region_code }}
              </el-tag>
              <span class="team-name">{{ row.team_name }}</span>
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
    </el-card>

    <!-- 战队详情弹窗 -->
    <el-dialog
      v-model="detailDialogVisible"
      :title="`战队评估详情 - ${selectedTeam?.team_name || ''}`"
      width="700px"
      :close-on-click-modal="true"
    >
      <div v-if="selectedTeam" class="team-detail-dialog">
        <!-- 基本评估 -->
        <div class="detail-section">
          <h4>赛季评估</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">当前排名</span>
              <span class="value">第 {{ selectedTeam.current_rank }} 名</span>
            </div>
            <div class="detail-item">
              <span class="label">上赛季排名</span>
              <span class="value">第 {{ selectedTeam.last_rank }} 名</span>
            </div>
            <div class="detail-item">
              <span class="label">稳定性评分</span>
              <span class="value" :class="getStabilityClass(selectedTeam.stability_score)">
                {{ selectedTeam.stability_score }}
              </span>
            </div>
            <div class="detail-item">
              <span class="label">评估结论</span>
              <span class="value">{{ selectedTeam.evaluation_reason }}</span>
            </div>
          </div>
        </div>

        <!-- 战略决策 -->
        <div class="detail-section">
          <h4>战略决策</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">战略类型</span>
              <el-tag :type="getStrategyTagType(selectedTeam.strategy)" effect="dark" size="large">
                {{ getStrategyLabel(selectedTeam.strategy) }}
              </el-tag>
            </div>
            <div class="detail-item">
              <span class="label">调整紧迫度</span>
              <el-tag :type="getUrgencyTagType(selectedTeam.urgency_level)">
                {{ getUrgencyLabel(selectedTeam.urgency_level) }}
              </el-tag>
            </div>
          </div>
          <div class="strategy-description">
            <p>{{ getStrategyDescription(selectedTeam.strategy) }}</p>
          </div>
        </div>

        <!-- 阵容信息 -->
        <div class="detail-section">
          <h4>阵容信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">阵容实力</span>
              <span class="value power">{{ selectedTeam.roster_power.toFixed(1) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">阵容人数</span>
              <span class="value">{{ selectedTeam.roster_count }} 人</span>
            </div>
            <div class="detail-item">
              <span class="label">平均年龄</span>
              <span class="value" :class="getAgeClass(selectedTeam.avg_age)">
                {{ selectedTeam.avg_age.toFixed(1) }} 岁
              </span>
            </div>
            <div class="detail-item">
              <span class="label">平均能力</span>
              <span class="value" :class="getAbilityClass(selectedTeam.avg_ability)">
                {{ selectedTeam.avg_ability.toFixed(1) }}
              </span>
            </div>
            <div class="detail-item">
              <span class="label">剩余预算</span>
              <span class="value" :class="getBudgetClass(selectedTeam.budget_remaining)">
                {{ formatMoney(selectedTeam.budget_remaining) }}
              </span>
            </div>
          </div>
        </div>

        <!-- 位置需求 -->
        <div class="detail-section" v-if="positionNeeds.length > 0">
          <h4>位置需求</h4>
          <el-table :data="positionNeeds" size="small" stripe>
            <el-table-column prop="position" label="位置" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="getPositionTagType(row.position)" size="small">
                  {{ getPositionLabel(row.position) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="current_count" label="现有" width="60" align="center" />
            <el-table-column prop="target_count" label="目标" width="60" align="center" />
            <el-table-column prop="gap" label="缺口" width="60" align="center">
              <template #default="{ row }">
                <span :class="row.gap < 0 ? 'gap-negative' : 'gap-positive'">
                  {{ row.gap > 0 ? '+' : '' }}{{ row.gap }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="current_avg_ability" label="现有能力" width="90" align="center">
              <template #default="{ row }">
                {{ row.current_avg_ability.toFixed(1) }}
              </template>
            </el-table-column>
            <el-table-column prop="priority" label="优先级" align="center">
              <template #default="{ row }">
                <el-tag :type="getPriorityTagType(row.priority)" size="small">
                  {{ getPriorityLabel(row.priority) }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- 选手评估 -->
        <div class="detail-section" v-if="playerEvaluations.length > 0">
          <h4>选手去留评估</h4>
          <el-table :data="playerEvaluations" size="small" stripe max-height="250">
            <el-table-column prop="player_name" label="选手" width="100" />
            <el-table-column prop="position" label="位置" width="60" align="center">
              <template #default="{ row }">
                <el-tag :type="getPositionTagType(row.position)" size="small">
                  {{ row.position }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="ability" label="能力" width="60" align="center" />
            <el-table-column prop="stay_score" label="留队意愿" width="90" align="center">
              <template #default="{ row }">
                <span :class="getStayScoreClass(row.stay_score)">
                  {{ row.stay_score.toFixed(0) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="wants_to_leave" label="状态" width="100" align="center">
              <template #default="{ row }">
                <el-tag v-if="row.wants_to_leave" type="danger" size="small">想离开</el-tag>
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
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, Refresh } from '@element-plus/icons-vue'
import {
  transferWindowApi,
  type TeamSeasonEvaluationInfo,
  type PositionNeedInfo,
  type PlayerStayEvaluationInfo,
} from '@/api/tauri'
import { formatMoneyFromWan } from '@/utils'

// 状态
const loading = ref(false)
const evaluations = ref<TeamSeasonEvaluationInfo[]>([])
const positionNeeds = ref<PositionNeedInfo[]>([])
const playerEvaluations = ref<PlayerStayEvaluationInfo[]>([])
const detailDialogVisible = ref(false)
const selectedTeam = ref<TeamSeasonEvaluationInfo | null>(null)

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

// 方法
async function loadEvaluations() {
  loading.value = true
  try {
    evaluations.value = await transferWindowApi.getTeamEvaluations()
  } catch (error) {
    ElMessage.error('加载战队评估数据失败')
    console.error(error)
  } finally {
    loading.value = false
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
    playerEvaluations.value = await transferWindowApi.getPlayerStayEvaluations(team.team_id)
  } catch (error) {
    console.error('加载选手评估失败', error)
    playerEvaluations.value = []
  }
}

function formatMoney(value: number): string {
  return formatMoneyFromWan(value)
}

// 样式辅助函数
function getRegionTagType(region: string): string {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
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

function getPriorityTagType(priority: string): string {
  const types: Record<string, string> = {
    CRITICAL: 'danger',
    HIGH: 'warning',
    MEDIUM: 'info',
    LOW: 'success',
  }
  return types[priority] || 'info'
}

function getPriorityLabel(priority: string): string {
  const labels: Record<string, string> = {
    CRITICAL: '紧急',
    HIGH: '高',
    MEDIUM: '中',
    LOW: '低',
  }
  return labels[priority] || priority
}

function getStayScoreClass(score: number): string {
  if (score >= 70) return 'stay-high'
  if (score >= 40) return 'stay-medium'
  return 'stay-low'
}

// 初始化
onMounted(() => {
  loadEvaluations()
})
</script>

<style scoped>
.evaluation-center {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  color: white;
}

.header-content h1 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
}

.header-content p {
  margin: 0;
  opacity: 0.9;
  font-size: 14px;
}

.header-stats {
  display: flex;
  gap: 30px;
}

.stat-item {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 28px;
  font-weight: 700;
}

.stat-label {
  font-size: 12px;
  opacity: 0.85;
}

.filter-card {
  margin-bottom: 20px;
}

.filter-row {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: flex-end;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-group label {
  font-size: 12px;
  color: #606266;
}

.table-card {
  margin-bottom: 20px;
}

.team-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-name {
  font-weight: 500;
}

.rank-change {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.rank-number {
  font-size: 18px;
  font-weight: 600;
}

.rank-up {
  color: #67c23a;
  font-size: 12px;
}

.rank-down {
  color: #f56c6c;
  font-size: 12px;
}

.rank-same {
  color: #909399;
  font-size: 12px;
}

.stability-display {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stability-value {
  font-weight: 600;
}

.stability-high {
  color: #67c23a;
}

.stability-medium {
  color: #e6a23c;
}

.stability-low {
  color: #f56c6c;
}

.power-value {
  font-weight: 500;
  color: #409eff;
}

.age-young {
  color: #67c23a;
}

.age-prime {
  color: #409eff;
}

.age-old {
  color: #e6a23c;
}

.ability-elite {
  color: #f56c6c;
  font-weight: 600;
}

.ability-good {
  color: #e6a23c;
}

.ability-average {
  color: #909399;
}

.ability-low {
  color: #c0c4cc;
}

.budget-high {
  color: #67c23a;
}

.budget-medium {
  color: #e6a23c;
}

.budget-low {
  color: #f56c6c;
}

/* 弹窗样式 */
.team-detail-dialog {
  max-height: 70vh;
  overflow-y: auto;
}

.detail-section {
  margin-bottom: 24px;
}

.detail-section h4 {
  margin: 0 0 12px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid #ebeef5;
  color: #303133;
  font-size: 16px;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-item .label {
  font-size: 12px;
  color: #909399;
}

.detail-item .value {
  font-size: 14px;
  color: #303133;
  font-weight: 500;
}

.detail-item .value.power {
  color: #409eff;
  font-size: 18px;
}

.strategy-description {
  margin-top: 12px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 6px;
}

.strategy-description p {
  margin: 0;
  color: #606266;
  font-size: 14px;
  line-height: 1.6;
}

.gap-negative {
  color: #f56c6c;
}

.gap-positive {
  color: #67c23a;
}

.stay-high {
  color: #67c23a;
  font-weight: 500;
}

.stay-medium {
  color: #e6a23c;
  font-weight: 500;
}

.stay-low {
  color: #f56c6c;
  font-weight: 500;
}

.reason-text {
  font-size: 12px;
  color: #606266;
}
</style>
