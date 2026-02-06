<template>
  <div class="bid-analysis">
    <!-- 页面标题 -->
    <div class="page-header">
      <div class="header-content">
        <h1>竞价分析中心</h1>
        <p>S{{ seasonId }} 赛季 · R4/R5 竞价过程透明化</p>
      </div>
      <div class="header-stats" v-if="overview">
        <div class="stat-item">
          <span class="stat-value">{{ overview.total_players }}</span>
          <span class="stat-label">涉及选手</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ overview.total_bids }}</span>
          <span class="stat-label">总出价数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value success">{{ overview.successful_signings }}</span>
          <span class="stat-label">成功签约</span>
        </div>
        <div class="stat-item">
          <span class="stat-value danger">{{ overview.failed_signings }}</span>
          <span class="stat-label">竞价失败</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ overview.avg_bids_per_player.toFixed(1) }}</span>
          <span class="stat-label">平均竞标数</span>
        </div>
      </div>
    </div>

    <!-- 筛选区域 -->
    <el-card class="filter-card">
      <div class="filter-row">
        <div class="filter-group">
          <label>轮次</label>
          <el-radio-group v-model="activeTab" size="default" @change="handleTabChange">
            <el-radio-button :value="0">全部</el-radio-button>
            <el-radio-button :value="4">R4 自由球员</el-radio-button>
            <el-radio-button :value="5">R5 合同挖角</el-radio-button>
          </el-radio-group>
        </div>
        <div class="filter-group">
          <label>搜索选手</label>
          <el-input
            v-model="searchText"
            placeholder="输入选手名称..."
            :prefix-icon="Search"
            clearable
            style="width: 180px"
          />
        </div>
        <div class="filter-group">
          <label>位置</label>
          <el-select v-model="filterPosition" placeholder="全部位置" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="Top" value="Top" />
            <el-option label="Jungle" value="Jungle" />
            <el-option label="Mid" value="Mid" />
            <el-option label="Bot" value="Bot" />
            <el-option label="Support" value="Support" />
          </el-select>
        </div>
        <div class="filter-group">
          <label>结果</label>
          <el-select v-model="filterOutcome" placeholder="全部结果" clearable style="width: 130px">
            <el-option label="全部" value="" />
            <el-option label="签约成功" value="signed" />
            <el-option label="竞价失败" value="no_willing_team" />
          </el-select>
        </div>
      </div>
    </el-card>

    <!-- 空状态 -->
    <el-card v-if="!loading && (!overview || overview.total_players === 0)" class="empty-card">
      <el-empty description="暂无竞价数据，从下个转会期开始记录" />
    </el-card>

    <!-- 选手竞价列表 -->
    <el-card v-if="overview && filteredAnalyses.length > 0" class="table-card">
      <el-table
        :data="paginatedData"
        v-loading="loading"
        row-key="playerKey"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'player_ability', order: 'descending' }"
        @sort-change="handleSortChange"
        max-height="calc(100vh - 320px)"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="bid-detail-container">
              <div class="bid-detail-header">
                <span>{{ row.player_name }} 的竞价详情（共 {{ row.total_bids }} 个报价）</span>
              </div>
              <el-table :data="row.bids" size="small" stripe border>
                <el-table-column label="#" width="46" align="center">
                  <template #default="{ $index }">{{ $index + 1 }}</template>
                </el-table-column>
                <el-table-column prop="bid_team_name" label="竞标球队" width="140" />
                <el-table-column label="报价薪资" width="110" align="right">
                  <template #default="{ row: bid }">
                    <span class="money-value">{{ formatMoney(bid.offered_salary) }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="合同" width="65" align="center">
                  <template #default="{ row: bid }">{{ bid.contract_years }}年</template>
                </el-table-column>
                <el-table-column v-if="activeTab !== 4" label="转会费" width="110" align="right">
                  <template #default="{ row: bid }">
                    <span class="money-value">{{ bid.transfer_fee > 0 ? formatMoney(bid.transfer_fee) : '-' }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="匹配度" width="90" align="center">
                  <template #default="{ row: bid }">
                    <span class="score-value" :class="getScoreClass(bid.match_score)">
                      {{ bid.match_score.toFixed(1) }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column label="意愿度" min-width="200">
                  <template #default="{ row: bid }">
                    <div class="willingness-cell">
                      <el-progress
                        :percentage="Math.min(bid.willingness, 100)"
                        :stroke-width="12"
                        :color="getWillingnessColor(bid.willingness)"
                        :show-text="false"
                        style="flex: 1"
                      />
                      <span class="willingness-value" :class="bid.willingness >= 40 ? 'pass' : 'fail'">
                        {{ bid.willingness.toFixed(1) }}
                      </span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="结果" width="110" align="center">
                  <template #default="{ row: bid }">
                    <el-tag v-if="bid.is_winner" type="success" size="small" effect="dark">签约成功</el-tag>
                    <el-tag v-else-if="bid.reject_reason === 'willingness_too_low'" type="danger" size="small">意愿不足</el-tag>
                    <el-tag v-else-if="bid.reject_reason === 'outbid'" type="info" size="small">被抢先</el-tag>
                    <el-tag v-else type="info" size="small">未中标</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </template>
        </el-table-column>

        <!-- 选手信息 -->
        <el-table-column label="选手" width="180" fixed>
          <template #default="{ row }">
            <div class="player-info">
              <div class="player-avatar" :class="getPositionClass(row.player_position)">
                {{ getPositionShort(row.player_position) }}
              </div>
              <div class="player-details">
                <span class="player-name">{{ row.player_name }}</span>
                <span class="player-source">
                  <el-tag v-if="row.from_team_name" size="small" type="warning">{{ row.from_team_name }}</el-tag>
                  <el-tag v-else size="small" type="info">自由球员</el-tag>
                </span>
              </div>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="player_ability" label="能力" width="90" sortable="custom" align="center">
          <template #default="{ row }">
            <div class="ability-display">
              <span class="ability-value" :class="getAbilityClass(row.player_ability)">{{ row.player_ability }}</span>
              <el-progress
                :percentage="row.player_ability"
                :stroke-width="4"
                :show-text="false"
                :color="getAbilityColor(row.player_ability)"
              />
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="player_age" label="年龄" width="75" sortable="custom" align="center">
          <template #default="{ row }">
            <span :class="getAgeClass(row.player_age)">{{ row.player_age }}岁</span>
          </template>
        </el-table-column>

        <el-table-column label="轮次" width="80" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="row.round === 4 ? 'primary' : 'warning'" effect="plain">
              R{{ row.round }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="total_bids" label="竞标数" width="85" sortable="custom" align="center">
          <template #default="{ row }">
            <span class="bid-count" :class="{ 'bid-hot': row.total_bids >= 5 }">{{ row.total_bids }}</span>
          </template>
        </el-table-column>

        <el-table-column label="结果" min-width="160">
          <template #default="{ row }">
            <div class="outcome-cell">
              <el-tag v-if="row.outcome === 'signed'" type="success" effect="dark" size="default">
                {{ row.winner_team_name }}
              </el-tag>
              <el-tag v-else type="danger" effect="plain" size="default">
                无球队满足意愿
              </el-tag>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[20, 50, 100]"
          :total="filteredAnalyses.length"
          layout="total, sizes, prev, pager, next"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 加载中 -->
    <div v-if="loading && !overview" class="loading-container">
      <el-icon class="is-loading" :size="32"><Loading /></el-icon>
      <span>加载竞价数据中...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { transferWindowApi } from '@/api/tauri'
import type { BidOverview, PlayerBidAnalysis } from '@/api/tauri'
import { formatMoney } from '@/utils/format'
import { Search, Loading } from '@element-plus/icons-vue'

const route = useRoute()

const loading = ref(false)
const overview = ref<BidOverview | null>(null)
const activeTab = ref<number>(0)
const searchText = ref('')
const filterPosition = ref('')
const filterOutcome = ref('')
const windowId = ref<number>(0)
const seasonId = ref<number>(0)

// 分页
const pagination = reactive({
  page: 1,
  pageSize: 50,
})

// 排序
const sortConfig = reactive({
  prop: 'player_ability',
  order: 'descending' as 'ascending' | 'descending',
})

// 筛选后的数据
const filteredAnalyses = computed(() => {
  if (!overview.value) return []
  let list = overview.value.player_analyses as (PlayerBidAnalysis & { playerKey?: string })[]

  if (searchText.value) {
    const s = searchText.value.toLowerCase()
    list = list.filter(p => p.player_name.toLowerCase().includes(s))
  }
  if (filterPosition.value) {
    list = list.filter(p => p.player_position === filterPosition.value)
  }
  if (filterOutcome.value) {
    list = list.filter(p => p.outcome === filterOutcome.value)
  }

  // 排序
  const sorted = [...list]
  sorted.sort((a, b) => {
    const aVal = (a as Record<string, unknown>)[sortConfig.prop] as number
    const bVal = (b as Record<string, unknown>)[sortConfig.prop] as number
    return sortConfig.order === 'ascending' ? aVal - bVal : bVal - aVal
  })

  return sorted.map(p => ({ ...p, playerKey: `${p.player_id}-${p.round}` }))
})

// 分页后的数据
const paginatedData = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  const end = start + pagination.pageSize
  return filteredAnalyses.value.slice(start, end)
})

// 方法
async function loadData() {
  if (!windowId.value) return
  loading.value = true
  try {
    const round = activeTab.value === 0 ? undefined : activeTab.value
    overview.value = await transferWindowApi.getTransferBidsOverview(windowId.value, round)
    pagination.page = 1
  } catch (e) {
    console.error('加载竞价数据失败', e)
  } finally {
    loading.value = false
  }
}

function handleTabChange() {
  loadData()
}

function handleSortChange({ prop, order }: { prop: string; order: 'ascending' | 'descending' | null }) {
  sortConfig.prop = prop || 'player_ability'
  sortConfig.order = order || 'descending'
}

function handleSizeChange(size: number) {
  pagination.pageSize = size
  pagination.page = 1
}

function handlePageChange(page: number) {
  pagination.page = page
}

function getPositionShort(pos: string | null) {
  const map: Record<string, string> = { Top: 'TOP', Jungle: 'JUG', Mid: 'MID', Bot: 'BOT', Support: 'SUP' }
  return map[pos || ''] || '?'
}

function getPositionClass(pos: string | null) {
  const map: Record<string, string> = { Top: 'position-top', Jungle: 'position-jug', Mid: 'position-mid', Bot: 'position-bot', Support: 'position-sup' }
  return map[pos || ''] || ''
}

function getAbilityClass(ability: number) {
  if (ability >= 90) return 'ability-elite'
  if (ability >= 80) return 'ability-high'
  if (ability >= 70) return 'ability-medium'
  return 'ability-low'
}

function getAbilityColor(ability: number) {
  if (ability >= 90) return '#e6a23c'
  if (ability >= 80) return '#409eff'
  if (ability >= 70) return '#67c23a'
  return '#909399'
}

function getAgeClass(age: number) {
  if (age <= 22) return 'age-young'
  if (age >= 28) return 'age-old'
  return 'age-prime'
}

function getScoreClass(score: number) {
  if (score >= 70) return 'score-high'
  if (score >= 50) return 'score-mid'
  return 'score-low'
}

function getWillingnessColor(w: number) {
  if (w >= 60) return '#67c23a'
  if (w >= 40) return '#e6a23c'
  return '#f56c6c'
}

onMounted(async () => {
  if (route.query.windowId) {
    windowId.value = Number(route.query.windowId)
  }
  if (route.query.seasonId) {
    seasonId.value = Number(route.query.seasonId)
  }
  if (!windowId.value) {
    try {
      const tw = await transferWindowApi.getCurrentTransferWindow()
      if (tw) {
        windowId.value = tw.window_id
        seasonId.value = tw.season_id
      }
    } catch (e) {
      console.error('获取当前转会窗口失败', e)
    }
  }
  await loadData()
})
</script>

<style scoped>
.bid-analysis {
  padding: 0;
}

/* ========== 页面标题（渐变色） ========== */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
  padding: 24px;
  background: linear-gradient(135deg, #2d3a4e 0%, #3a5068 50%, #2d5a7b 100%);
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

.stat-item .stat-value {
  font-size: 24px;
  font-weight: 700;
}

.stat-item .stat-value.success {
  color: #95d475;
}

.stat-item .stat-value.danger {
  color: #f89898;
}

.stat-item .stat-label {
  font-size: 12px;
  opacity: 0.8;
  margin-top: 4px;
}

/* ========== 筛选卡片 ========== */
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

/* ========== 空状态 ========== */
.empty-card {
  border-radius: 12px;
}

/* ========== 表格卡片 ========== */
.table-card {
  border-radius: 12px;
}

/* 固定列溢出修复 */
.table-card :deep(.el-table__fixed),
.table-card :deep(.el-table__fixed-right) {
  z-index: 10;
}

.table-card :deep(.el-table .el-table__cell) {
  overflow: hidden;
}

/* ========== 选手信息列 ========== */
.player-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.player-avatar {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  color: white;
  background: #909399;
  flex-shrink: 0;
}

.player-avatar.position-top { background: linear-gradient(135deg, #f56c6c, #e6a23c); }
.player-avatar.position-jug { background: linear-gradient(135deg, #67c23a, #85ce61); }
.player-avatar.position-mid { background: linear-gradient(135deg, #409eff, #66b1ff); }
.player-avatar.position-bot { background: linear-gradient(135deg, #e6a23c, #f7ba2a); }
.player-avatar.position-sup { background: linear-gradient(135deg, #909399, #b4b4b4); }

.player-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.player-name {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
}

.player-source {
  font-size: 12px;
}

/* ========== 能力值 ========== */
.ability-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.ability-value {
  font-weight: 700;
  font-size: 14px;
}

.ability-elite { color: #e6a23c; }
.ability-high { color: #409eff; }
.ability-medium { color: #67c23a; }
.ability-low { color: #909399; }

/* ========== 年龄 ========== */
.age-young { color: #67c23a; font-weight: 600; }
.age-prime { color: #303133; }
.age-old { color: #f56c6c; }

/* ========== 竞标数 ========== */
.bid-count {
  font-weight: 600;
  font-size: 15px;
}

.bid-count.bid-hot {
  color: #e6a23c;
}

/* ========== 结果列 ========== */
.outcome-cell {
  display: flex;
  align-items: center;
}

/* ========== 展开行：竞价详情 ========== */
.bid-detail-container {
  padding: 16px 24px;
  background: #fafbfc;
}

.bid-detail-header {
  font-size: 13px;
  font-weight: 600;
  color: #606266;
  margin-bottom: 10px;
}

.money-value {
  color: #409eff;
  font-weight: 500;
}

.score-value {
  font-weight: 700;
  font-size: 13px;
}

.score-high { color: #67c23a; }
.score-mid { color: #e6a23c; }
.score-low { color: #f56c6c; }

.willingness-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.willingness-value {
  font-size: 13px;
  font-weight: 700;
  min-width: 36px;
  text-align: right;
}

.willingness-value.pass { color: #67c23a; }
.willingness-value.fail { color: #f56c6c; }

/* ========== 分页 ========== */
.pagination-wrapper {
  margin-top: 16px;
  padding: 0 16px 16px;
  display: flex;
  justify-content: flex-end;
}

/* ========== 加载中 ========== */
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 80px 0;
  color: var(--el-text-color-secondary);
}

/* ========== 响应式 ========== */
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
