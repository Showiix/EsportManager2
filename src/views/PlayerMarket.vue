<template>
  <div class="contract-center">
    <!-- 页面标题 -->
    <div class="page-header">
      <div class="header-content">
        <h1>选手合同中心</h1>
        <p>管理和查看所有选手的合同、薪资与状态信息</p>
      </div>
      <div class="header-stats">
        <div class="stat-item">
          <span class="stat-value">{{ totalPlayers }}</span>
          <span class="stat-label">总选手数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ expiringContracts }}</span>
          <span class="stat-label">合同即将到期</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ freeAgents }}</span>
          <span class="stat-label">自由球员</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ formatSalary(totalSalary, false) }}</span>
          <span class="stat-label">总薪资支出</span>
        </div>
      </div>
    </div>

    <!-- 筛选区域 -->
    <el-card class="filter-card">
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
          <label>位置</label>
          <el-select v-model="filters.position" placeholder="全部位置" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="上单" value="TOP" />
            <el-option label="打野" value="JUG" />
            <el-option label="中单" value="MID" />
            <el-option label="ADC" value="ADC" />
            <el-option label="辅助" value="SUP" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>合同状态</label>
          <el-select v-model="filters.contractStatus" placeholder="全部状态" clearable style="width: 140px">
            <el-option label="全部" value="" />
            <el-option label="有效合同" value="active" />
            <el-option label="即将到期" value="expiring" />
            <el-option label="自由球员" value="free" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>首发状态</label>
          <el-select v-model="filters.starterStatus" placeholder="全部" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="首发" value="starter" />
            <el-option label="替补" value="bench" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>满意度</label>
          <el-select v-model="filters.satisfaction" placeholder="全部" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="满意 (≥70)" value="high" />
            <el-option label="一般 (40-69)" value="medium" />
            <el-option label="不满 (<40)" value="low" />
          </el-select>
        </div>
        <el-button type="primary" :icon="Refresh" @click="loadPlayers" :loading="loading">
          刷新数据
        </el-button>
      </div>
    </el-card>

    <!-- 数据表格 -->
    <el-card class="table-card">
      <el-table
        :data="paginatedPlayers"
        v-loading="loading"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'ability', order: 'descending' }"
        @sort-change="handleSortChange"
        max-height="calc(100vh - 280px)"
      >
        <!-- 选手基本信息 -->
        <el-table-column label="选手" width="200" fixed>
          <template #default="{ row }">
            <div class="player-info">
              <div class="player-avatar" :class="getPositionClass(row.position)">
                {{ row.position || '?' }}
              </div>
              <div class="player-details">
                <span class="player-name">{{ row.player_name }}</span>
                <span class="player-team">
                  <el-tag v-if="row.team_name" size="small" :type="getRegionTagType(row.region_code)">
                    {{ row.team_name }}
                  </el-tag>
                  <el-tag v-else size="small" type="info">自由球员</el-tag>
                </span>
              </div>
            </div>
          </template>
        </el-table-column>

        <!-- 年龄 -->
        <el-table-column prop="age" label="年龄" width="80" sortable align="center">
          <template #default="{ row }">
            <span :class="getAgeClass(row.age)">{{ row.age }}岁</span>
          </template>
        </el-table-column>

        <!-- 能力值 -->
        <el-table-column prop="ability" label="能力" width="100" sortable align="center">
          <template #default="{ row }">
            <div class="ability-display">
              <span class="ability-value" :class="getAbilityClass(row.ability)">{{ row.ability }}</span>
              <el-progress
                :percentage="row.ability"
                :stroke-width="4"
                :show-text="false"
                :color="getAbilityColor(row.ability)"
              />
            </div>
          </template>
        </el-table-column>

        <!-- 潜力值 -->
        <el-table-column prop="potential" label="潜力" width="100" sortable align="center">
          <template #default="{ row }">
            <div class="ability-display">
              <span class="potential-value">{{ row.potential }}</span>
              <el-progress
                :percentage="row.potential"
                :stroke-width="4"
                :show-text="false"
                color="#67c23a"
              />
            </div>
          </template>
        </el-table-column>

        <!-- 首发状态 -->
        <el-table-column label="首发" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="row.is_starter ? 'success' : 'info'" size="small">
              {{ row.is_starter ? '首发' : '替补' }}
            </el-tag>
          </template>
        </el-table-column>

        <!-- 合同信息 -->
        <el-table-column label="合同" width="160">
          <template #default="{ row }">
            <div class="contract-info">
              <div class="contract-salary">
                <span class="label">薪资:</span>
                <span class="value">{{ formatSalary(row.salary) }}</span>
              </div>
              <div class="contract-duration">
                <span class="label">到期:</span>
                <span class="value" :class="getContractClass(row.contract_end_season)">
                  {{ row.contract_end_season ? `第${row.contract_end_season}赛季` : '无合同' }}
                </span>
              </div>
            </div>
          </template>
        </el-table-column>

        <!-- 身价 -->
        <el-table-column prop="calculated_market_value" label="身价" width="140" sortable align="right">
          <template #default="{ row }">
            <div class="market-value">
              <span class="current-value">{{ formatValue(row.calculated_market_value) }}</span>
              <span v-if="row.calculated_market_value !== row.base_market_value" class="base-value">
                基础: {{ formatValue(row.base_market_value) }}
              </span>
            </div>
          </template>
        </el-table-column>

        <!-- 满意度 -->
        <el-table-column prop="satisfaction" label="满意度" width="100" sortable align="center">
          <template #default="{ row }">
            <div class="stat-cell">
              <span class="stat-value" :style="{ color: getSatisfactionColor(row.satisfaction) }">
                {{ row.satisfaction }}
              </span>
              <el-progress
                :percentage="row.satisfaction"
                :stroke-width="4"
                :show-text="false"
                :color="getSatisfactionColor(row.satisfaction)"
              />
            </div>
          </template>
        </el-table-column>

        <!-- 忠诚度 -->
        <el-table-column prop="loyalty" label="忠诚度" width="100" sortable align="center">
          <template #default="{ row }">
            <div class="stat-cell">
              <span class="stat-value" :style="{ color: getLoyaltyColor(row.loyalty) }">
                {{ row.loyalty }}
              </span>
              <el-progress
                :percentage="row.loyalty"
                :stroke-width="4"
                :show-text="false"
                :color="getLoyaltyColor(row.loyalty)"
              />
            </div>
          </template>
        </el-table-column>

        <!-- 状态 -->
        <el-table-column prop="status" label="状态" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getStatusTagType(row.status)" size="small">
              {{ getStatusLabel(row.status) }}
            </el-tag>
          </template>
        </el-table-column>

        <!-- 操作 -->
        <el-table-column label="操作" width="100" fixed="right" align="center">
          <template #default="{ row }">
            <el-button type="primary" link size="small" @click="showPlayerDetail(row)">
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[20, 50, 100, 200]"
          :total="filteredPlayers.length"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 选手详情弹窗 -->
    <el-dialog
      v-model="detailDialogVisible"
      :title="`选手详情 - ${selectedPlayer?.player_name || ''}`"
      width="600px"
      :close-on-click-modal="true"
    >
      <div v-if="selectedPlayer" class="player-detail-dialog">
        <div class="detail-section">
          <h4>基本信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">选手名称</span>
              <span class="value">{{ selectedPlayer.player_name }}</span>
            </div>
            <div class="detail-item">
              <span class="label">位置</span>
              <span class="value">{{ getPositionLabel(selectedPlayer.position) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">年龄</span>
              <span class="value">{{ selectedPlayer.age }}岁</span>
            </div>
            <div class="detail-item">
              <span class="label">能力值</span>
              <span class="value ability" :class="getAbilityClass(selectedPlayer.ability)">{{ selectedPlayer.ability }}</span>
            </div>
            <div class="detail-item">
              <span class="label">潜力值</span>
              <span class="value potential">{{ selectedPlayer.potential }}</span>
            </div>
            <div class="detail-item">
              <span class="label">首发状态</span>
              <span class="value">{{ selectedPlayer.is_starter ? '首发' : '替补' }}</span>
            </div>
          </div>
        </div>

        <div class="detail-section">
          <h4>战队信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">当前战队</span>
              <span class="value">{{ selectedPlayer.team_name || '自由球员' }}</span>
            </div>
            <div class="detail-item">
              <span class="label">赛区</span>
              <span class="value">{{ selectedPlayer.region_code || '-' }}</span>
            </div>
            <div class="detail-item">
              <span class="label">加入赛季</span>
              <span class="value">{{ selectedPlayer.join_season ? `第${selectedPlayer.join_season}赛季` : '-' }}</span>
            </div>
          </div>
        </div>

        <div class="detail-section">
          <h4>合同信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">年薪</span>
              <span class="value salary">{{ formatSalary(selectedPlayer.salary, false) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">合同到期</span>
              <span class="value" :class="getContractClass(selectedPlayer.contract_end_season)">
                {{ selectedPlayer.contract_end_season ? `第${selectedPlayer.contract_end_season}赛季` : '无合同' }}
              </span>
            </div>
            <div class="detail-item">
              <span class="label">基础身价</span>
              <span class="value">{{ formatValue(selectedPlayer.base_market_value) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">当前身价</span>
              <span class="value market-value">{{ formatValue(selectedPlayer.calculated_market_value) }}</span>
            </div>
          </div>
        </div>

        <div class="detail-section">
          <h4>状态信息</h4>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="label">满意度</span>
              <div class="value-with-progress">
                <span class="value">{{ selectedPlayer.satisfaction }}%</span>
                <el-progress
                  :percentage="selectedPlayer.satisfaction"
                  :stroke-width="6"
                  :color="getSatisfactionColor(selectedPlayer.satisfaction)"
                />
              </div>
            </div>
            <div class="detail-item">
              <span class="label">忠诚度</span>
              <div class="value-with-progress">
                <span class="value">{{ selectedPlayer.loyalty }}%</span>
                <el-progress
                  :percentage="selectedPlayer.loyalty"
                  :stroke-width="6"
                  :color="getLoyaltyColor(selectedPlayer.loyalty)"
                />
              </div>
            </div>
            <div class="detail-item">
              <span class="label">选手状态</span>
              <el-tag :type="getStatusTagType(selectedPlayer.status)">
                {{ getStatusLabel(selectedPlayer.status) }}
              </el-tag>
            </div>
          </div>
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
import { transferApi, type PlayerMarketInfo } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { formatValue, formatSalary } from '@/utils/format'
import { createLogger } from '@/utils/logger'

const logger = createLogger('PlayerMarket')

const gameStore = useGameStore()

// 数据
const players = ref<PlayerMarketInfo[]>([])
const loading = ref(false)
const detailDialogVisible = ref(false)
const selectedPlayer = ref<PlayerMarketInfo | null>(null)

// 筛选条件
const filters = reactive({
  search: '',
  region: '',
  position: '',
  contractStatus: '',
  starterStatus: '',
  satisfaction: '',
})

// 分页
const pagination = reactive({
  page: 1,
  pageSize: 50,
})

// 排序
const sortConfig = reactive({
  prop: 'ability',
  order: 'descending' as 'ascending' | 'descending',
})

// 计算属性
const totalPlayers = computed(() => players.value.length)

const expiringContracts = computed(() => {
  const currentSeason = gameStore.currentSave?.current_season || 1
  return players.value.filter(p =>
    p.contract_end_season && p.contract_end_season <= currentSeason + 1
  ).length
})

const freeAgents = computed(() =>
  players.value.filter(p => !p.team_id).length
)

const totalSalary = computed(() =>
  players.value.reduce((sum, p) => sum + (p.salary || 0), 0)
)

const filteredPlayers = computed(() => {
  let result = [...players.value]
  const currentSeason = gameStore.currentSave?.current_season || 1

  // 搜索
  if (filters.search) {
    const search = filters.search.toLowerCase()
    result = result.filter(p =>
      p.player_name.toLowerCase().includes(search) ||
      (p.team_name && p.team_name.toLowerCase().includes(search))
    )
  }

  // 赛区筛选
  if (filters.region) {
    result = result.filter(p => p.region_code === filters.region)
  }

  // 位置筛选
  if (filters.position) {
    result = result.filter(p => p.position === filters.position)
  }

  // 合同状态筛选
  if (filters.contractStatus) {
    switch (filters.contractStatus) {
      case 'active':
        result = result.filter(p => p.contract_end_season && p.contract_end_season > currentSeason + 1)
        break
      case 'expiring':
        result = result.filter(p =>
          p.contract_end_season &&
          p.contract_end_season <= currentSeason + 1 &&
          p.contract_end_season >= currentSeason
        )
        break
      case 'free':
        result = result.filter(p => !p.team_id)
        break
    }
  }

  // 首发状态筛选
  if (filters.starterStatus) {
    result = result.filter(p =>
      filters.starterStatus === 'starter' ? p.is_starter : !p.is_starter
    )
  }

  // 满意度筛选
  if (filters.satisfaction) {
    switch (filters.satisfaction) {
      case 'high':
        result = result.filter(p => p.satisfaction >= 70)
        break
      case 'medium':
        result = result.filter(p => p.satisfaction >= 40 && p.satisfaction < 70)
        break
      case 'low':
        result = result.filter(p => p.satisfaction < 40)
        break
    }
  }

  // 排序
  result.sort((a, b) => {
    const aVal = a[sortConfig.prop as keyof PlayerMarketInfo] as number
    const bVal = b[sortConfig.prop as keyof PlayerMarketInfo] as number
    return sortConfig.order === 'ascending' ? aVal - bVal : bVal - aVal
  })

  return result
})

// 分页后的数据（只渲染当前页）
const paginatedPlayers = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  const end = start + pagination.pageSize
  return filteredPlayers.value.slice(start, end)
})

// 方法
const loadPlayers = async () => {
  loading.value = true
  try {
    const result = await transferApi.getPlayerMarketList()
    players.value = result
  } catch (error) {
    logger.error('Failed to load player list:', error)
    ElMessage.error(`加载选手列表失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const showPlayerDetail = (player: PlayerMarketInfo) => {
  selectedPlayer.value = player
  detailDialogVisible.value = true
}

const handleSortChange = ({ prop, order }: { prop: string; order: 'ascending' | 'descending' | null }) => {
  sortConfig.prop = prop || 'ability'
  sortConfig.order = order || 'descending'
}

const handleSizeChange = (size: number) => {
  pagination.pageSize = size
  pagination.page = 1
}

const handlePageChange = (page: number) => {
  pagination.page = page
}

const getPositionLabel = (position: string | null) => {
  const labels: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: 'ADC',
    SUP: '辅助',
  }
  return position ? labels[position] || position : '-'
}

const getPositionClass = (position: string | null) => {
  return position ? `position-${position.toLowerCase()}` : ''
}

const getRegionTagType = (region: string | null) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return (region ? types[region] : 'info') as 'danger' | 'primary' | 'success' | 'warning' | 'info'
}

const getAgeClass = (age: number) => {
  if (age <= 20) return 'age-young'
  if (age >= 28) return 'age-old'
  return 'age-prime'
}

const getAbilityClass = (ability: number) => {
  if (ability >= 90) return 'ability-elite'
  if (ability >= 80) return 'ability-high'
  if (ability >= 70) return 'ability-medium'
  return 'ability-low'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#e6a23c'
  if (ability >= 80) return '#409eff'
  if (ability >= 70) return '#67c23a'
  return '#909399'
}

const getContractClass = (endSeason: number | null) => {
  if (!endSeason) return 'contract-none'
  const currentSeason = gameStore.currentSave?.current_season || 1
  if (endSeason <= currentSeason) return 'contract-expired'
  if (endSeason <= currentSeason + 1) return 'contract-expiring'
  return 'contract-active'
}

const getSatisfactionColor = (satisfaction: number) => {
  if (satisfaction >= 70) return '#67c23a'
  if (satisfaction >= 40) return '#e6a23c'
  return '#f56c6c'
}

const getLoyaltyColor = (loyalty: number) => {
  if (loyalty >= 70) return '#409eff'
  if (loyalty >= 40) return '#e6a23c'
  return '#f56c6c'
}

const getStatusTagType = (status: string) => {
  const types: Record<string, string> = {
    ACTIVE: 'success',
    INJURED: 'danger',
    SUSPENDED: 'warning',
    RETIRED: 'info',
  }
  return (types[status] || 'info') as 'success' | 'danger' | 'warning' | 'info'
}

const getStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    ACTIVE: '活跃',
    INJURED: '受伤',
    SUSPENDED: '停赛',
    RETIRED: '退役',
  }
  return labels[status] || status
}

// 生命周期
onMounted(() => {
  loadPlayers()
})
</script>

<style scoped>
.contract-center {
  padding: 20px;
  background: #f5f7fa;
  min-height: 100vh;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 24px;
  background: linear-gradient(135deg, #1e3a5f 0%, #2d5a7b 100%);
  border-radius: 12px;
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.header-content h1 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px 0;
}

.header-content p {
  font-size: 14px;
  opacity: 0.85;
  margin: 0;
}

.header-stats {
  display: flex;
  gap: 30px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
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

/* 筛选卡片 */
.filter-card {
  margin-bottom: 20px;
  border-radius: 12px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.filter-card :deep(.el-card__body) {
  padding: 12px 16px;
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
  color: #909399;
  font-weight: 500;
  white-space: nowrap;
}

/* 表格卡片 */
.table-card {
  border-radius: 12px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.table-card :deep(.el-table th.el-table__cell) {
  background: #f8f9fb;
  color: #606266;
  font-size: 13px;
  font-weight: 600;
  padding: 8px 0;
}

.table-card :deep(.el-table td.el-table__cell) {
  padding: 6px 0;
}

/* 修复固定列溢出问题 */
.table-card :deep(.el-table__fixed),
.table-card :deep(.el-table__fixed-right) {
  z-index: 10;
}

.table-card :deep(.el-table__fixed-right-patch) {
  z-index: 10;
}

.table-card :deep(.el-table .el-table__cell) {
  overflow: hidden;
}

/* 选手信息 */
.player-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.player-avatar {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  color: white;
  background: #909399;
}

.player-avatar.position-top { background: linear-gradient(135deg, #f56c6c, #e6a23c); }
.player-avatar.position-jug { background: linear-gradient(135deg, #67c23a, #85ce61); }
.player-avatar.position-mid { background: linear-gradient(135deg, #409eff, #66b1ff); }
.player-avatar.position-adc { background: linear-gradient(135deg, #e6a23c, #f7ba2a); }
.player-avatar.position-sup { background: linear-gradient(135deg, #909399, #b4b4b4); }

.player-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.player-name {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
}

.player-team {
  font-size: 12px;
}

/* 年龄样式 */
.age-young { color: #67c23a; font-weight: 600; }
.age-prime { color: #303133; }
.age-old { color: #f56c6c; }

/* 能力值样式 */
.ability-display, .stat-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-cell .stat-value {
  font-weight: 700;
  font-size: 14px;
}

.ability-value, .potential-value {
  font-weight: 700;
  font-size: 14px;
}

.ability-elite { color: #e6a23c; }
.ability-high { color: #409eff; }
.ability-medium { color: #67c23a; }
.ability-low { color: #909399; }

/* 合同信息 */
.contract-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.contract-salary, .contract-duration {
  display: flex;
  gap: 4px;
  font-size: 13px;
}

.contract-salary .label, .contract-duration .label {
  color: #909399;
}

.contract-salary .value {
  font-weight: 500;
  color: #67c23a;
}

.contract-active { color: #67c23a; }
.contract-expiring { color: #e6a23c; font-weight: 600; }
.contract-expired { color: #f56c6c; }
.contract-none { color: #909399; }

/* 身价 */
.market-value {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.current-value {
  font-weight: 600;
  color: #409eff;
}

.base-value {
  font-size: 11px;
  color: #909399;
}

/* 分页 */
.pagination-wrapper {
  margin-top: 16px;
  padding: 16px;
  display: flex;
  justify-content: center;
}

/* 详情弹窗 */
.player-detail-dialog {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.detail-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 12px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid #ebeef5;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
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
  font-weight: 500;
  color: #303133;
}

.detail-item .value.ability { color: #409eff; font-size: 18px; }
.detail-item .value.potential { color: #67c23a; font-size: 18px; }
.detail-item .value.salary { color: #67c23a; }
.detail-item .value.market-value { color: #409eff; }

.value-with-progress {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.value-with-progress .value {
  font-weight: 600;
}

/* 响应式 */
@media (max-width: 1200px) {
  .page-header {
    flex-direction: column;
    gap: 16px;
  }

  .header-stats {
    width: 100%;
    justify-content: space-around;
  }

  .filter-row {
    flex-direction: column;
    align-items: stretch;
  }

  .filter-group {
    width: 100%;
  }

  .detail-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
