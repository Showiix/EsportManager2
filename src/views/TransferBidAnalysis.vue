<template>
  <div class="bid-analysis">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>
          竞价分析中心
          <span class="formula-trigger" @click="showFormulaDialog = true" title="意愿度计算说明">
            <el-icon :size="16"><QuestionFilled /></el-icon>
          </span>
        </h1>
        <div class="header-sub-row">
          <p>S{{ selectedSeason || seasonId }} 赛季 · R4/R5 竞价过程透明化</p>
          <SeasonSelector v-model="selectedSeason" @update:model-value="onSeasonChange" width="140px" />
        </div>
      </div>
    </div>

    <div class="stats-bar" v-if="overview">
      <div class="stat-item">
        <span class="stat-value">{{ overview.total_players }}</span>
        <span class="stat-label">涉及选手</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ overview.total_bids }}</span>
        <span class="stat-label">总出价</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value income">{{ overview.successful_signings }}</span>
        <span class="stat-label">成功签约</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value expense">{{ overview.failed_signings }}</span>
        <span class="stat-label">竞价失败</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ overview.avg_bids_per_player.toFixed(1) }}</span>
        <span class="stat-label">平均竞标</span>
      </div>
    </div>

    <!-- 筛选区域 -->
    <div class="filter-section">
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
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && (!overview || overview.total_players === 0)" class="empty-section">
      <el-empty description="暂无竞价数据，从下个转会期开始记录" />
    </div>

    <!-- 选手竞价列表 -->
    <div v-if="overview && filteredAnalyses.length > 0" class="table-section">
      <el-table
        :data="paginatedData"
        v-loading="loading"
        row-key="playerKey"
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
    </div>

    <!-- 加载中 -->
    <div v-if="loading && !overview" class="loading-container">
      <el-icon class="is-loading" :size="32"><Loading /></el-icon>
      <span>加载竞价数据中...</span>
    </div>

    <!-- 意愿度公式弹窗 -->
    <el-dialog v-model="showFormulaDialog" width="620px" :show-close="false" append-to-body class="formula-dialog">
      <template #header>
        <div class="formula-dialog-header">
          <div class="formula-dialog-icon">
            <el-icon :size="20"><QuestionFilled /></el-icon>
          </div>
          <div>
            <h3>意愿度计算公式</h3>
            <p>选手是否接受报价的核心判定逻辑</p>
          </div>
        </div>
      </template>

      <div class="formula-content">
        <!-- Step 1 -->
        <div class="formula-card">
          <div class="formula-card-header">
            <span class="step-badge">1</span>
            <span class="step-title">薪资满意度</span>
            <span class="weight-badge blue">权重 40%</span>
          </div>
          <div class="formula-card-body">
            <p class="formula-hint">报价薪资 / 当前薪资 = 薪资比</p>
            <div class="score-bar-list">
              <div class="score-bar-item" v-for="item in salaryTiers" :key="item.label">
                <span class="score-bar-label">{{ item.label }}</span>
                <div class="score-bar-track">
                  <div class="score-bar-fill" :style="{ width: item.score + '%', background: item.color }" />
                </div>
                <span class="score-bar-value" :style="{ color: item.color }">{{ item.score }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2 -->
        <div class="formula-card">
          <div class="formula-card-header">
            <span class="step-badge">2</span>
            <span class="step-title">忠诚度影响</span>
            <span class="weight-badge purple">权重 30%</span>
          </div>
          <div class="formula-card-body">
            <p class="formula-hint">忠诚度越高，转会意愿越低</p>
            <div class="code-block">
              <span class="code-keyword">loyalty_impact</span> = (100 - loyalty) &times; 0.5
            </div>
            <div class="example-row">
              <div class="example-item">
                <span class="example-label">loyalty = 30</span>
                <span class="example-arrow">&rarr;</span>
                <span class="example-value good">35</span>
              </div>
              <div class="example-item">
                <span class="example-label">loyalty = 70</span>
                <span class="example-arrow">&rarr;</span>
                <span class="example-value mid">15</span>
              </div>
              <div class="example-item">
                <span class="example-label">loyalty = 90</span>
                <span class="example-arrow">&rarr;</span>
                <span class="example-value bad">5</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 3 -->
        <div class="formula-card">
          <div class="formula-card-header">
            <span class="step-badge">3</span>
            <span class="step-title">基础意愿合成</span>
          </div>
          <div class="formula-card-body">
            <div class="code-block">
              <span class="code-keyword">base</span> = salary_score &times; <span class="code-num">0.4</span>
              + loyalty_impact &times; <span class="code-num">0.3</span>
              + <span class="code-num">15</span>
              + random(<span class="code-num">-5</span>, <span class="code-num">5</span>)
            </div>
            <p class="formula-hint" style="margin-top: 8px">固定基础分 15 保证最低意愿底线，随机波动 &plusmn;5 模拟心态</p>
          </div>
        </div>

        <!-- Step 4 -->
        <div class="formula-card">
          <div class="formula-card-header">
            <span class="step-badge">4</span>
            <span class="step-title">跨赛区惩罚</span>
            <span class="weight-badge orange">核心机制</span>
          </div>
          <div class="formula-card-body">
            <div class="code-block">
              <div><span class="code-comment">// 本赛区转会</span></div>
              <div><span class="code-keyword">最终意愿</span> = base &times; <span class="code-num">1.0</span></div>
              <div style="margin-top: 4px"><span class="code-comment">// 跨赛区转会</span></div>
              <div><span class="code-keyword">最终意愿</span> = base &times; (100 - region_loyalty) / 100</div>
            </div>
            <div class="region-grid">
              <div class="region-item" v-for="r in regionData" :key="r.name">
                <div class="region-flag">{{ r.flag }}</div>
                <div class="region-name">{{ r.name }}</div>
                <div class="region-loyalty">loyalty {{ r.range }}</div>
                <div class="region-factor" :style="{ color: r.color }">{{ r.factor }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 5 -->
        <div class="formula-card threshold">
          <div class="formula-card-body" style="text-align: center; padding: 16px">
            <div class="threshold-text">
              意愿度 <span class="threshold-op">&ge;</span> <span class="threshold-num">40</span> <span class="threshold-arrow">&rarr;</span> 接受签约
            </div>
            <p class="formula-hint" style="margin-top: 6px">低于 40 的报价将被选手拒绝</p>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button type="primary" @click="showFormulaDialog = false">我知道了</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { transferWindowApi } from '@/api/tauri'
import type { BidOverview, PlayerBidAnalysis } from '@/api/tauri'
import { formatMoney } from '@/utils/format'
import { Search, Loading, QuestionFilled } from '@element-plus/icons-vue'
import { useSeasonStore } from '@/stores/useSeasonStore'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const route = useRoute()
const seasonStore = useSeasonStore()

const loading = ref(false)
const overview = ref<BidOverview | null>(null)
const activeTab = ref<number>(0)
const searchText = ref('')
const filterPosition = ref('')
const filterOutcome = ref('')
const windowId = ref<number>(0)
const seasonId = ref<number>(0)
const selectedSeason = ref<number>(0)
const showFormulaDialog = ref(false)

// 弹窗静态数据
const salaryTiers = [
  { label: '&ge; 1.2', score: 100, color: '#67c23a' },
  { label: '&ge; 1.0', score: 80, color: '#409eff' },
  { label: '&ge; 0.8', score: 60, color: '#e6a23c' },
  { label: '&ge; 0.6', score: 40, color: '#f56c6c' },
  { label: '< 0.6', score: 20, color: '#909399' },
]

const regionData = [
  { name: 'LPL', flag: '🇨🇳', range: '75~90', factor: '0.10 ~ 0.25', color: '#f56c6c' },
  { name: 'LCK', flag: '🇰🇷', range: '55~75', factor: '0.25 ~ 0.45', color: '#e6a23c' },
  { name: 'LEC', flag: '🇪🇺', range: '45~65', factor: '0.35 ~ 0.55', color: '#409eff' },
  { name: 'LCS', flag: '🇺🇸', range: '40~60', factor: '0.40 ~ 0.60', color: '#67c23a' },
]

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
    const aVal = (a as unknown as Record<string, unknown>)[sortConfig.prop] as number
    const bVal = (b as unknown as Record<string, unknown>)[sortConfig.prop] as number
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
  loading.value = true
  try {
    const round = activeTab.value === 0 ? undefined : activeTab.value
    overview.value = await transferWindowApi.getTransferBidsOverview(
      windowId.value || undefined, round, selectedSeason.value || undefined
    )
    pagination.page = 1
  } catch (e) {
    console.error('加载竞价数据失败', e)
  } finally {
    loading.value = false
  }
}

function onSeasonChange(val: number) {
  selectedSeason.value = val
  seasonId.value = val
  windowId.value = 0
  loadData()
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
  const map: Record<string, string> = { top: 'TOP', jug: 'JUG', mid: 'MID', adc: 'BOT', sup: 'SUP', bot: 'BOT' }
  return map[(pos || '').toLowerCase()] || (pos || '?')
}

function getPositionClass(pos: string | null) {
  const map: Record<string, string> = { top: 'position-top', jug: 'position-jug', mid: 'position-mid', adc: 'position-bot', sup: 'position-sup', bot: 'position-bot' }
  return map[(pos || '').toLowerCase()] || ''
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
  if (route.query.seasonId) {
    selectedSeason.value = Number(route.query.seasonId)
    seasonId.value = selectedSeason.value
  } else if (route.query.windowId) {
    windowId.value = Number(route.query.windowId)
  }

  if (!windowId.value && !selectedSeason.value) {
    try {
      const tw = await transferWindowApi.getCurrentTransferWindow()
      if (tw) {
        windowId.value = tw.window_id
        seasonId.value = tw.season_id
        selectedSeason.value = tw.season_id
      }
    } catch (e) {
      console.error('获取当前转会窗口失败', e)
    }
  }

  // 如果仍然没有选中赛季，用当前活跃赛季
  if (!selectedSeason.value) {
    selectedSeason.value = seasonStore.currentSeason
    seasonId.value = selectedSeason.value
  }

  await loadData()
})
</script>

<style scoped>
.bid-analysis { padding: 0; }
.page-header { margin-bottom: 20px; }
.page-header h1 { font-size: 24px; font-weight: 700; color: #0f172a; margin: 0 0 4px 0; display: inline-flex; align-items: center; }
.page-header p { font-size: 13px; color: #94a3b8; margin: 0; }
.header-sub-row { display: flex; align-items: center; gap: 12px; }

.stats-bar { display: flex; align-items: center; padding: 14px 24px; background: #fff; border: 1px solid #e2e8f0; border-radius: 10px; margin-bottom: 12px; }
.stat-item { display: flex; align-items: baseline; gap: 6px; flex: 1; justify-content: center; }
.stat-item .stat-value { font-size: 20px; font-weight: 700; color: #0f172a; font-variant-numeric: tabular-nums; }
.stat-item .stat-value.income { color: #10b981; }
.stat-item .stat-value.expense { color: #ef4444; }
.stat-item .stat-label { font-size: 12px; color: #94a3b8; font-weight: 500; }
.stat-divider { width: 1px; height: 24px; background: #e2e8f0; flex-shrink: 0; }

.filter-section { margin-bottom: 16px; }
.filter-row { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; }
.filter-group { display: flex; align-items: center; gap: 6px; }
.filter-group label { font-size: 12px; color: #94a3b8; font-weight: 500; white-space: nowrap; }

.empty-section { border: 1px solid #e2e8f0; border-radius: 10px; padding: 40px; text-align: center; }
.table-section { border: 1px solid #e2e8f0; border-radius: 10px; padding: 16px; }
.table-section :deep(.el-table th.el-table__cell) { font-weight: 600; color: #94a3b8; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; background: transparent; border-bottom: 1px solid #f1f5f9; padding: 10px 0; }
.table-section :deep(.el-table__body tr) { transition: background-color 0.15s; }
.table-section :deep(.el-table__body tr td) { padding: 12px 0; border-bottom: 1px solid #f8fafc; }
.table-section :deep(.el-table__body tr:hover > td) { background-color: #f8fafc !important; }
.table-section :deep(.el-table__fixed), .table-section :deep(.el-table__fixed-right) { z-index: 10; }
.table-section :deep(.el-table .el-table__cell) { overflow: hidden; }

.player-info { display: flex; align-items: center; gap: 10px; }
.player-avatar { width: 36px; height: 36px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700; color: white; background: #94a3b8; flex-shrink: 0; }
.player-avatar.position-top { background: #ef4444; }
.player-avatar.position-jug { background: #22c55e; }
.player-avatar.position-mid { background: #3b82f6; }
.player-avatar.position-bot { background: #f59e0b; }
.player-avatar.position-sup { background: #6b7280; }
.player-details { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.player-name { font-weight: 600; font-size: 13px; color: #0f172a; }
.player-source { font-size: 12px; }

.ability-display { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.ability-value { font-weight: 700; font-size: 14px; }
.ability-elite { color: #f59e0b; }
.ability-high { color: #3b82f6; }
.ability-medium { color: #22c55e; }
.ability-low { color: #94a3b8; }
.age-young { color: #22c55e; font-weight: 600; }
.age-prime { color: #0f172a; }
.age-old { color: #ef4444; }
.bid-count { font-weight: 600; font-size: 15px; }
.bid-count.bid-hot { color: #f59e0b; }
.outcome-cell { display: flex; align-items: center; }

.bid-detail-container { padding: 16px 24px; background: #f8fafc; }
.bid-detail-header { font-size: 13px; font-weight: 600; color: #64748b; margin-bottom: 10px; }
.money-value { color: #3b82f6; font-weight: 500; }
.score-value { font-weight: 700; font-size: 13px; }
.score-high { color: #22c55e; }
.score-mid { color: #f59e0b; }
.score-low { color: #ef4444; }
.willingness-cell { display: flex; align-items: center; gap: 8px; }
.willingness-value { font-size: 13px; font-weight: 700; min-width: 36px; text-align: right; }
.willingness-value.pass { color: #22c55e; }
.willingness-value.fail { color: #ef4444; }

.pagination-wrapper { margin-top: 16px; display: flex; justify-content: center; }
.loading-container { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 80px 0; color: #94a3b8; }

@media (max-width: 1200px) {
  .stats-bar { flex-wrap: wrap; gap: 8px; }
  .stat-divider { display: none; }
  .filter-row { flex-direction: column; align-items: stretch; }
}

/* ========== 公式触发器 ========== */
.formula-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  cursor: pointer;
  vertical-align: middle;
  margin-left: 8px;
  transition: all 0.2s;
}

.formula-trigger:hover {
  background: rgba(255, 255, 255, 0.4);
  transform: scale(1.1);
}

/* ========== 公式弹窗 ========== */
/* ========== 公式弹窗（穿透 el-dialog） ========== */
:deep(.formula-dialog .el-dialog__header) {
  padding: 0;
  margin: 0;
}

:deep(.formula-dialog .el-dialog__body) {
  padding: 20px 24px;
}

:deep(.formula-dialog .el-dialog) {
  border-radius: 12px;
  overflow: hidden;
}

.formula-dialog-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 20px 24px;
  background: linear-gradient(135deg, #2d3a4e, #3a5068);
  border-radius: 8px 8px 0 0;
  color: white;
}

.formula-dialog-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.formula-dialog-header h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 700;
}

.formula-dialog-header p {
  margin: 3px 0 0;
  font-size: 12px;
  opacity: 0.7;
}

.formula-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* 卡片 */
.formula-card {
  border: 1px solid #ebeef5;
  border-radius: 10px;
  overflow: hidden;
  transition: box-shadow 0.2s;
}

.formula-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.formula-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: #fafbfc;
  border-bottom: 1px solid #ebeef5;
}

.step-badge {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  background: #303133;
  color: white;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.step-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.weight-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
  margin-left: auto;
}

.weight-badge.blue {
  color: #409eff;
  background: rgba(64, 158, 255, 0.1);
}

.weight-badge.purple {
  color: #9467bd;
  background: rgba(148, 103, 189, 0.1);
}

.weight-badge.orange {
  color: #e6a23c;
  background: rgba(230, 162, 60, 0.1);
}

.formula-card-body {
  padding: 14px 16px;
}

.formula-hint {
  font-size: 12px;
  color: #909399;
  margin: 0 0 10px;
  line-height: 1.5;
}

/* 薪资进度条 */
.score-bar-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.score-bar-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.score-bar-label {
  font-size: 12px;
  color: #606266;
  min-width: 48px;
  text-align: right;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
}

.score-bar-track {
  flex: 1;
  height: 8px;
  background: #f0f2f5;
  border-radius: 4px;
  overflow: hidden;
}

.score-bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.4s ease;
}

.score-bar-value {
  font-size: 13px;
  font-weight: 700;
  min-width: 28px;
  text-align: right;
}

/* 代码块 */
.code-block {
  background: #1e1e2e;
  color: #cdd6f4;
  padding: 12px 16px;
  border-radius: 8px;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  font-size: 13px;
  line-height: 1.8;
}

.code-keyword { color: #89b4fa; font-weight: 600; }
.code-num { color: #fab387; }
.code-comment { color: #6c7086; font-style: italic; }

/* 忠诚度示例 */
.example-row {
  display: flex;
  gap: 12px;
  margin-top: 10px;
}

.example-item {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 8px;
  font-size: 12px;
}

.example-label {
  color: #909399;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
}

.example-arrow {
  color: #c0c4cc;
}

.example-value {
  font-weight: 700;
  font-size: 15px;
}

.example-value.good { color: #67c23a; }
.example-value.mid { color: #e6a23c; }
.example-value.bad { color: #f56c6c; }

/* 赛区网格 */
.region-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  margin-top: 12px;
}

.region-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 10px 6px;
  background: #f5f7fa;
  border-radius: 8px;
  text-align: center;
}

.region-flag {
  font-size: 20px;
  line-height: 1;
}

.region-name {
  font-size: 13px;
  font-weight: 700;
  color: #303133;
}

.region-loyalty {
  font-size: 11px;
  color: #909399;
}

.region-factor {
  font-size: 13px;
  font-weight: 700;
  margin-top: 2px;
}

/* 阈值卡片 */
.formula-card.threshold {
  border-color: #b3e19d;
  background: linear-gradient(135deg, rgba(103, 194, 58, 0.04), rgba(103, 194, 58, 0.1));
}

.threshold-text {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.threshold-op {
  color: #67c23a;
}

.threshold-num {
  font-size: 24px;
  font-weight: 800;
  color: #67c23a;
}

.threshold-arrow {
  color: #67c23a;
  margin: 0 4px;
}
</style>
