<template>
  <div class="bid-analysis">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push('/transfer/window')">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回转会期</span>
      </button>
    </div>

    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>竞价分析中心</h1>
        <p>S{{ seasonId }} 赛季 · 竞价过程透明化</p>
      </div>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="16" class="stats-row" v-if="overview">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="24"><User /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ overview.total_players }}</div>
              <div class="stat-label">涉及选手</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="24"><Tickets /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ overview.total_bids }}</div>
              <div class="stat-label">总出价数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="24"><Check /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ overview.successful_signings }}</div>
              <div class="stat-label">成功签约</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon red">
              <el-icon :size="24"><Close /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ overview.failed_signings }}</div>
              <div class="stat-label">竞价失败</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 筛选栏 -->
    <el-card class="filter-card">
      <div class="filter-row">
        <el-radio-group v-model="activeTab" @change="handleTabChange">
          <el-radio-button :value="0">全部</el-radio-button>
          <el-radio-button :value="4">R4 自由球员</el-radio-button>
          <el-radio-button :value="5">R5 合同挖角</el-radio-button>
        </el-radio-group>
        <el-input
          v-model="searchText"
          placeholder="搜索选手名称"
          clearable
          style="width: 200px; margin-left: 16px"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-select v-model="filterPosition" placeholder="位置" clearable style="width: 120px; margin-left: 12px">
          <el-option label="Top" value="Top" />
          <el-option label="Jungle" value="Jungle" />
          <el-option label="Mid" value="Mid" />
          <el-option label="Bot" value="Bot" />
          <el-option label="Support" value="Support" />
        </el-select>
        <el-select v-model="filterOutcome" placeholder="结果" clearable style="width: 120px; margin-left: 12px">
          <el-option label="签约成功" value="signed" />
          <el-option label="竞价失败" value="no_willing_team" />
        </el-select>
      </div>
    </el-card>

    <!-- 空状态 -->
    <el-card v-if="!loading && (!overview || overview.total_players === 0)" class="empty-card">
      <el-empty description="暂无竞价数据，从下个转会期开始记录" />
    </el-card>

    <!-- 选手竞价列表 -->
    <el-card v-if="overview && filteredAnalyses.length > 0" class="table-card">
      <el-table
        :data="filteredAnalyses"
        row-key="playerKey"
        stripe
        style="width: 100%"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="bid-detail-container">
              <el-table :data="row.bids" size="small" stripe>
                <el-table-column label="#" width="50">
                  <template #default="{ $index }">{{ $index + 1 }}</template>
                </el-table-column>
                <el-table-column prop="bid_team_name" label="球队" width="140" />
                <el-table-column label="薪资" width="100">
                  <template #default="{ row: bid }">{{ formatMoney(bid.offered_salary) }}</template>
                </el-table-column>
                <el-table-column label="合同" width="70">
                  <template #default="{ row: bid }">{{ bid.contract_years }}年</template>
                </el-table-column>
                <el-table-column label="转会费" width="100" v-if="activeTab !== 4">
                  <template #default="{ row: bid }">{{ bid.transfer_fee > 0 ? formatMoney(bid.transfer_fee) : '-' }}</template>
                </el-table-column>
                <el-table-column label="匹配度" width="90">
                  <template #default="{ row: bid }">
                    <span :style="{ color: bid.match_score >= 70 ? '#67c23a' : bid.match_score >= 50 ? '#e6a23c' : '#f56c6c' }">
                      {{ bid.match_score.toFixed(1) }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column label="意愿度" width="200">
                  <template #default="{ row: bid }">
                    <div class="willingness-cell">
                      <el-progress
                        :percentage="Math.min(bid.willingness, 100)"
                        :stroke-width="14"
                        :color="bid.willingness >= 40 ? '#67c23a' : '#f56c6c'"
                        :format="() => ''"
                        style="flex: 1"
                      />
                      <span class="willingness-value" :class="{ pass: bid.willingness >= 40, fail: bid.willingness < 40 }">
                        {{ bid.willingness.toFixed(1) }}
                      </span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="结果" width="120">
                  <template #default="{ row: bid }">
                    <el-tag v-if="bid.is_winner" type="success" size="small">签约成功</el-tag>
                    <el-tag v-else-if="bid.reject_reason === 'willingness_too_low'" type="danger" size="small">意愿不足</el-tag>
                    <el-tag v-else-if="bid.reject_reason === 'outbid'" type="info" size="small">被抢先</el-tag>
                    <el-tag v-else type="info" size="small">未中标</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="选手" width="140">
          <template #default="{ row }">
            <span class="player-name">{{ row.player_name }}</span>
          </template>
        </el-table-column>
        <el-table-column label="位置" width="80">
          <template #default="{ row }">
            <el-tag size="small" :type="positionTagType(row.player_position)">{{ row.player_position || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="player_ability" label="能力" width="70" sortable />
        <el-table-column prop="player_age" label="年龄" width="70" sortable />
        <el-table-column label="来源" width="120">
          <template #default="{ row }">
            <span v-if="row.from_team_name">{{ row.from_team_name }}</span>
            <el-tag v-else size="small" type="info">自由球员</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="轮次" width="80">
          <template #default="{ row }">
            <el-tag size="small" :type="row.round === 4 ? 'primary' : 'warning'">R{{ row.round }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="total_bids" label="竞标数" width="80" sortable />
        <el-table-column label="结果" width="140">
          <template #default="{ row }">
            <el-tag v-if="row.outcome === 'signed'" type="success">
              {{ row.winner_team_name }}
            </el-tag>
            <el-tag v-else type="danger">无球队满足意愿</el-tag>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 加载中 -->
    <div v-if="loading" class="loading-container">
      <el-icon class="is-loading" :size="32"><Loading /></el-icon>
      <span>加载竞价数据中...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { transferWindowApi } from '@/api/tauri'
import type { BidOverview, PlayerBidAnalysis } from '@/api/tauri'
import { formatMoney } from '@/utils/format'
import {
  ArrowLeft, User, Tickets, Check, Close, Search, Loading
} from '@element-plus/icons-vue'

const route = useRoute()

const loading = ref(false)
const overview = ref<BidOverview | null>(null)
const activeTab = ref<number>(0)
const searchText = ref('')
const filterPosition = ref('')
const filterOutcome = ref('')
const windowId = ref<number>(0)
const seasonId = ref<number>(0)

const filteredAnalyses = computed(() => {
  if (!overview.value) return []
  let list = overview.value.player_analyses
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
  return list.map(p => ({ ...p, playerKey: `${p.player_id}-${p.round}` }))
})

function positionTagType(pos: string | null) {
  const map: Record<string, string> = { Top: '', Jungle: 'success', Mid: 'warning', Bot: 'danger', Support: 'info' }
  return map[pos || ''] || 'info'
}

async function loadData() {
  if (!windowId.value) return
  loading.value = true
  try {
    const round = activeTab.value === 0 ? undefined : activeTab.value
    overview.value = await transferWindowApi.getTransferBidsOverview(windowId.value, round)
  } catch (e) {
    console.error('加载竞价数据失败', e)
  } finally {
    loading.value = false
  }
}

function handleTabChange() {
  loadData()
}

onMounted(async () => {
  // 从路由 query 或当前转会窗口获取 windowId
  if (route.query.windowId) {
    windowId.value = Number(route.query.windowId)
  }
  if (route.query.seasonId) {
    seasonId.value = Number(route.query.seasonId)
  }
  // 如果没有 windowId，尝试获取当前转会窗口
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
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.back-nav {
  margin-bottom: 16px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  color: var(--el-color-primary);
  cursor: pointer;
  font-size: 14px;
  padding: 4px 0;
}

.back-btn:hover {
  opacity: 0.8;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0 0 4px 0;
  font-size: 24px;
  font-weight: 600;
}

.page-header p {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.stats-row {
  margin-bottom: 16px;
}

.stat-card {
  height: 80px;
}

.stat-card :deep(.el-card__body) {
  padding: 16px;
  height: 100%;
  display: flex;
  align-items: center;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-icon.blue { background: rgba(64, 158, 255, 0.1); color: #409eff; }
.stat-icon.purple { background: rgba(148, 103, 189, 0.1); color: #9467bd; }
.stat-icon.green { background: rgba(103, 194, 58, 0.1); color: #67c23a; }
.stat-icon.red { background: rgba(245, 108, 108, 0.1); color: #f56c6c; }

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}

.filter-card {
  margin-bottom: 16px;
}

.filter-card :deep(.el-card__body) {
  padding: 12px 16px;
}

.filter-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.table-card {
  margin-bottom: 20px;
}

.table-card :deep(.el-card__body) {
  padding: 0;
}

.player-name {
  font-weight: 600;
}

.bid-detail-container {
  padding: 12px 20px;
  background: var(--el-fill-color-lighter);
}

.willingness-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.willingness-value {
  font-size: 13px;
  font-weight: 600;
  min-width: 36px;
  text-align: right;
}

.willingness-value.pass {
  color: #67c23a;
}

.willingness-value.fail {
  color: #f56c6c;
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--el-text-color-secondary);
}

.empty-card {
  margin-bottom: 20px;
}
</style>
