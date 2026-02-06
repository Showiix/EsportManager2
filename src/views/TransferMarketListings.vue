<template>
  <div class="transfer-market-listings">
    <!-- 页面标题 -->
    <div class="page-header">
      <div class="header-content">
        <h1>转会挂牌市场</h1>
        <p>查看当前转会窗口中被挂牌出售的选手和自由球员</p>
      </div>
      <div class="header-stats">
        <div class="stat-item">
          <span class="stat-value">{{ activeCount }}</span>
          <span class="stat-label">在售选手</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ soldCount }}</span>
          <span class="stat-label">已售出</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ freeAgentCount }}</span>
          <span class="stat-label">自由球员</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ formatValue(totalDealAmount) }}</span>
          <span class="stat-label">总成交额</span>
        </div>
      </div>
    </div>

    <!-- 窗口状态提示 -->
    <el-alert
      v-if="marketData && !marketData.window_id"
      title="当前赛季暂无转会窗口"
      description="转会窗口尚未开启，仅显示自由球员列表。进入转会期后将显示挂牌选手。"
      type="info"
      show-icon
      :closable="false"
      style="margin-bottom: 16px"
    />
    <el-alert
      v-else-if="marketData && marketData.window_status"
      :title="`转会窗口状态: ${getWindowStatusLabel(marketData.window_status)} | 当前轮次: 第${marketData.current_round}轮 | S${marketData.season_id}赛季`"
      :type="getWindowStatusType(marketData.window_status)"
      show-icon
      :closable="false"
      style="margin-bottom: 16px"
    />

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
          <label>状态</label>
          <el-select v-model="filters.status" placeholder="全部状态" clearable style="width: 140px">
            <el-option label="全部" value="" />
            <el-option label="在售" value="ACTIVE" />
            <el-option label="已售出" value="SOLD" />
            <el-option label="自由球员" value="FREE" />
          </el-select>
        </div>
        <el-button type="primary" :icon="Refresh" @click="loadMarketData" :loading="loading">
          刷新数据
        </el-button>
      </div>
    </el-card>

    <!-- 数据表格 -->
    <el-card class="table-card">
      <el-table
        :data="paginatedList"
        v-loading="loading"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'ability', order: 'descending' }"
        @sort-change="handleSortChange"
        max-height="calc(100vh - 280px)"
      >
        <!-- 选手 -->
        <el-table-column label="选手" width="200" fixed>
          <template #default="{ row }">
            <div class="player-info">
              <div class="player-avatar" :class="getPositionClass(row.position)">
                {{ row.position || '?' }}
              </div>
              <div class="player-details">
                <span class="player-name">{{ row.player_name }}</span>
                <span class="player-team">
                  <el-tag v-if="row.listed_by_team_name && row.listing_status !== 'FREE'" size="small" :type="getRegionTagType(row.listed_by_region_code)">
                    {{ row.listed_by_team_name }}
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

        <!-- 能力 -->
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

        <!-- 潜力 -->
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

        <!-- 身价 -->
        <el-table-column prop="calculated_market_value" label="身价" width="120" sortable align="right">
          <template #default="{ row }">
            <span class="market-value-text">{{ formatValue(row.calculated_market_value) }}</span>
          </template>
        </el-table-column>

        <!-- 所属战队 -->
        <el-table-column label="所属战队" width="140" align="center">
          <template #default="{ row }">
            <template v-if="row.listing_status === 'FREE'">
              <span class="text-gray">--</span>
            </template>
            <template v-else>
              <el-tag size="small" :type="getRegionTagType(row.listed_by_region_code)">
                {{ row.listed_by_team_name }}
              </el-tag>
            </template>
          </template>
        </el-table-column>

        <!-- 挂牌价 -->
        <el-table-column prop="listing_price" label="挂牌价" width="120" sortable align="right">
          <template #default="{ row }">
            <span v-if="row.listing_price != null">{{ formatValue(row.listing_price) }}</span>
            <span v-else class="text-gray">--</span>
          </template>
        </el-table-column>

        <!-- 最低接受价 -->
        <el-table-column prop="min_accept_price" label="最低接受价" width="120" sortable align="right">
          <template #default="{ row }">
            <span v-if="row.min_accept_price != null">{{ formatValue(row.min_accept_price) }}</span>
            <span v-else class="text-gray">--</span>
          </template>
        </el-table-column>

        <!-- 状态 -->
        <el-table-column label="状态" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getListingStatusTagType(row.listing_status)" size="small">
              {{ getListingStatusLabel(row.listing_status) }}
            </el-tag>
          </template>
        </el-table-column>

        <!-- 买家 -->
        <el-table-column label="买家" width="140" align="center">
          <template #default="{ row }">
            <template v-if="row.listing_status === 'SOLD' && row.sold_to_team_name">
              <el-tag size="small" :type="getRegionTagType(row.sold_to_region_code)">
                {{ row.sold_to_team_name }}
              </el-tag>
            </template>
            <span v-else class="text-gray">--</span>
          </template>
        </el-table-column>

        <!-- 成交价 -->
        <el-table-column prop="actual_price" label="成交价" width="120" sortable align="right">
          <template #default="{ row }">
            <span v-if="row.actual_price != null" class="deal-price">{{ formatValue(row.actual_price) }}</span>
            <span v-else class="text-gray">--</span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[20, 50, 100, 200]"
          :total="filteredList.length"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, Refresh } from '@element-plus/icons-vue'
import { transferWindowApi, type TransferMarketData } from '@/api/tauri'
import { formatValue } from '@/utils/format'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TransferMarketListings')

// 统一列表项类型
interface MarketListItem {
  player_id: number
  player_name: string
  position: string | null
  age: number
  ability: number
  potential: number
  calculated_market_value: number
  listing_status: string  // ACTIVE / SOLD / FREE
  // 挂牌相关
  listing_price: number | null
  min_accept_price: number | null
  actual_price: number | null
  listed_by_team_name: string | null
  listed_by_region_code: string | null
  sold_to_team_name: string | null
  sold_to_region_code: string | null
}

// 数据
const marketData = ref<TransferMarketData | null>(null)
const loading = ref(false)

// 筛选
const filters = reactive({
  search: '',
  region: '',
  position: '',
  status: '',
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

// 合并列表
const mergedList = computed<MarketListItem[]>(() => {
  if (!marketData.value) return []

  const items: MarketListItem[] = []

  // 添加挂牌选手
  for (const l of marketData.value.listings) {
    items.push({
      player_id: l.player_id,
      player_name: l.player_name,
      position: l.position,
      age: l.age,
      ability: l.ability,
      potential: l.potential,
      calculated_market_value: l.calculated_market_value,
      listing_status: l.listing_status,
      listing_price: l.listing_price,
      min_accept_price: l.min_accept_price,
      actual_price: l.actual_price,
      listed_by_team_name: l.listed_by_team_name,
      listed_by_region_code: l.listed_by_region_code,
      sold_to_team_name: l.sold_to_team_name ?? null,
      sold_to_region_code: l.sold_to_region_code ?? null,
    })
  }

  // 添加自由球员
  for (const fa of marketData.value.free_agents) {
    items.push({
      player_id: fa.player_id,
      player_name: fa.player_name,
      position: fa.position,
      age: fa.age,
      ability: fa.ability,
      potential: fa.potential,
      calculated_market_value: fa.calculated_market_value,
      listing_status: 'FREE',
      listing_price: null,
      min_accept_price: null,
      actual_price: null,
      listed_by_team_name: null,
      listed_by_region_code: null,
      sold_to_team_name: null,
      sold_to_region_code: null,
    })
  }

  return items
})

// 统计
const activeCount = computed(() =>
  mergedList.value.filter(i => i.listing_status === 'ACTIVE').length
)
const soldCount = computed(() =>
  mergedList.value.filter(i => i.listing_status === 'SOLD').length
)
const freeAgentCount = computed(() =>
  mergedList.value.filter(i => i.listing_status === 'FREE').length
)
const totalDealAmount = computed(() =>
  mergedList.value
    .filter(i => i.listing_status === 'SOLD' && i.actual_price != null)
    .reduce((sum, i) => sum + (i.actual_price ?? 0), 0)
)

// 筛选后列表
const filteredList = computed(() => {
  let result = [...mergedList.value]

  if (filters.search) {
    const search = filters.search.toLowerCase()
    result = result.filter(i =>
      i.player_name.toLowerCase().includes(search) ||
      (i.listed_by_team_name && i.listed_by_team_name.toLowerCase().includes(search))
    )
  }

  if (filters.region) {
    result = result.filter(i => {
      if (i.listing_status === 'FREE') return false
      return i.listed_by_region_code === filters.region
    })
  }

  if (filters.position) {
    result = result.filter(i => i.position === filters.position)
  }

  if (filters.status) {
    result = result.filter(i => i.listing_status === filters.status)
  }

  // 排序
  result.sort((a, b) => {
    const aVal = (a as unknown as Record<string, unknown>)[sortConfig.prop] as number ?? 0
    const bVal = (b as unknown as Record<string, unknown>)[sortConfig.prop] as number ?? 0
    return sortConfig.order === 'ascending' ? aVal - bVal : bVal - aVal
  })

  return result
})

// 分页数据
const paginatedList = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  return filteredList.value.slice(start, start + pagination.pageSize)
})

// 加载数据
const loadMarketData = async () => {
  loading.value = true
  try {
    marketData.value = await transferWindowApi.getTransferMarketListings()
  } catch (error) {
    logger.error('Failed to load market listings:', error)
    ElMessage.error(`加载挂牌市场数据失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 排序变化
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

// 辅助方法
const getPositionClass = (position: string | null) =>
  position ? `position-${position.toLowerCase()}` : ''

const getRegionTagType = (region: string | null) => {
  const types: Record<string, string> = { LPL: 'danger', LCK: 'primary', LEC: 'success', LCS: 'warning' }
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

const getListingStatusTagType = (status: string) => {
  const types: Record<string, string> = { ACTIVE: 'success', SOLD: 'danger', FREE: 'warning' }
  return (types[status] || 'info') as 'success' | 'danger' | 'warning' | 'info'
}

const getListingStatusLabel = (status: string) => {
  const labels: Record<string, string> = { ACTIVE: '在售', SOLD: '已售出', FREE: '自由球员' }
  return labels[status] || status
}

const getWindowStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    PENDING: '准备中',
    IN_PROGRESS: '进行中',
    COMPLETED: '已结束',
    CANCELLED: '已取消',
  }
  return labels[status] || status
}

const getWindowStatusType = (status: string) => {
  const types: Record<string, string> = {
    PENDING: 'info',
    IN_PROGRESS: 'success',
    COMPLETED: 'warning',
    CANCELLED: 'danger',
  }
  return (types[status] || 'info') as 'info' | 'success' | 'warning' | 'danger'
}

onMounted(() => {
  loadMarketData()
})
</script>

<style scoped>
.transfer-market-listings {
  padding: 0;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
  padding: 24px;
  background: linear-gradient(135deg, #1e3a5f 0%, #2d5a7b 100%);
  border-radius: 12px;
  color: white;
}

.header-content h1 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px 0;
}

.header-content p {
  font-size: 14px;
  opacity: 0.8;
  margin: 0;
}

.header-stats {
  display: flex;
  gap: 32px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
}

.stat-label {
  font-size: 12px;
  opacity: 0.8;
  margin-top: 4px;
}

/* 筛选卡片 */
.filter-card {
  margin-bottom: 16px;
  border-radius: 12px;
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
  color: #909399;
  font-weight: 500;
}

/* 表格卡片 */
.table-card {
  border-radius: 12px;
}

.table-card :deep(.el-table__fixed),
.table-card :deep(.el-table__fixed-right) {
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
.ability-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.ability-value, .potential-value {
  font-weight: 700;
  font-size: 14px;
}

.ability-elite { color: #e6a23c; }
.ability-high { color: #409eff; }
.ability-medium { color: #67c23a; }
.ability-low { color: #909399; }

/* 身价 */
.market-value-text {
  font-weight: 600;
  color: #409eff;
}

/* 成交价 */
.deal-price {
  font-weight: 600;
  color: #f56c6c;
}

/* 灰色文本 */
.text-gray {
  color: #c0c4cc;
}

/* 分页 */
.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
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
}
</style>
