<template>
  <div class="draft-auction">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push(`/draft/${region}`)">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回选秀</span>
      </button>
    </div>

    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>选秀权拍卖</h1>
        <p>{{ currentSeason }} 赛季 - {{ regionName }}</p>
      </div>
      <div class="header-actions">
        <el-tag :type="statusTagType" size="large" effect="dark">
          {{ statusText }}
        </el-tag>
      </div>
    </div>

    <!-- 进度条 -->
    <el-card class="progress-card">
      <div class="progress-header">
        <span class="progress-title">拍卖进度</span>
        <span class="progress-text">第 {{ currentRound }} / {{ totalRounds }} 轮</span>
      </div>
      <el-progress
        :percentage="progressPercentage"
        :stroke-width="12"
        :format="() => ''"
        status="success"
      />
      <div class="round-labels">
        <span
          v-for="index in totalRounds"
          :key="index"
          class="round-label"
          :class="{ active: index <= currentRound, current: index === currentRound }"
        >
          第{{ index }}轮
        </span>
      </div>
    </el-card>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="5">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="24"><Tickets /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ auctionStats.totalAuctions }}</div>
              <div class="stat-label">挂牌数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="5">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="24"><Check /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ auctionStats.soldCount }}</div>
              <div class="stat-label">成交数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="5">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon red">
              <el-icon :size="24"><Close /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ auctionStats.expiredCount }}</div>
              <div class="stat-label">流拍数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="5">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="24"><Money /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatMoney(auctionStats.totalRevenue) }}</div>
              <div class="stat-label">总成交额</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="24"><Coin /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatMoney(auctionStats.totalCommission) }}</div>
              <div class="stat-label">联盟佣金</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <el-button
        v-if="!isAuctionStarted"
        type="primary"
        size="large"
        :loading="isLoading"
        @click="handleStartAuction"
      >
        <el-icon><VideoPlay /></el-icon>
        开始拍卖
      </el-button>
      <el-button
        v-if="isAuctionInProgress"
        type="primary"
        size="large"
        :loading="isLoading"
        @click="handleNextRound"
      >
        <el-icon><CaretRight /></el-icon>
        执行下一轮
      </el-button>
      <el-button
        v-if="isAuctionInProgress"
        type="warning"
        size="large"
        :loading="isLoading"
        @click="handleFastForward"
      >
        <el-icon><DArrowRight /></el-icon>
        快进完成
      </el-button>
      <el-button
        v-if="isAuctionCompleted && !isFinalized"
        type="success"
        size="large"
        :loading="isLoading"
        @click="handleFinalize"
      >
        <el-icon><CircleCheck /></el-icon>
        确认结果，继续选秀
      </el-button>
      <el-button @click="router.push(`/draft/${region}`)">
        <el-icon><Back /></el-icon>
        返回选秀
      </el-button>
    </div>

    <!-- 挂牌列表 -->
    <div v-if="listings.length > 0" class="listings-section">
      <div class="section-header">
        <h2>
          <el-icon><Goods /></el-icon>
          挂牌签位
        </h2>
        <div class="filter-group">
          <el-radio-group v-model="filterStatus" size="small">
            <el-radio-button value="all">全部</el-radio-button>
            <el-radio-button value="ACTIVE">拍卖中</el-radio-button>
            <el-radio-button value="SOLD">已成交</el-radio-button>
            <el-radio-button value="EXPIRED">已流拍</el-radio-button>
          </el-radio-group>
        </div>
      </div>

      <div class="listings-grid">
        <div
          v-for="listing in filteredListings"
          :key="listing.id"
          class="listing-card"
          :class="listing.status.toLowerCase()"
        >
          <div class="listing-header">
            <div class="pick-badge" :class="getPickClass(listing.draft_position)">
              第{{ listing.draft_position }}签
            </div>
            <el-tag
              :type="getListingStatusType(listing.status)"
              size="small"
              effect="dark"
            >
              {{ getListingStatusText(listing.status) }}
            </el-tag>
          </div>

          <div class="listing-content">
            <div class="seller-info">
              <span class="info-label">卖家</span>
              <span class="team-name">{{ listing.seller_team_name }}</span>
            </div>

            <div class="price-info">
              <div class="price-row">
                <span class="price-label">起拍价</span>
                <span class="price-value">{{ formatAmount(listing.starting_price) }}</span>
              </div>
              <div class="price-row current">
                <span class="price-label">当前价</span>
                <span class="price-value highlight">{{ formatAmount(listing.current_price) }}</span>
              </div>
            </div>

            <div v-if="listing.status === 'SOLD'" class="buyer-info">
              <span class="info-label">买家</span>
              <span class="team-name success">{{ listing.buyer_team_name }}</span>
              <span class="final-price">成交价: {{ formatAmount(listing.final_price || listing.current_price) }}</span>
            </div>
          </div>

          <div class="listing-footer">
            <span class="bid-round">竞价轮数: {{ listing.current_bid_round }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 求购信息 -->
    <div v-if="wantedRequests.length > 0" class="wanted-section">
      <div class="section-header">
        <h2>
          <el-icon><Goods /></el-icon>
          求购信息
          <el-tag size="small" type="warning" effect="plain" round>{{ wantedRequests.length }}</el-tag>
        </h2>
        <div class="filter-group">
          <el-radio-group v-model="filterWantedStatus" size="small">
            <el-radio-button value="all">全部</el-radio-button>
            <el-radio-button value="ACTIVE">求购中</el-radio-button>
            <el-radio-button value="FULFILLED">已成交</el-radio-button>
            <el-radio-button value="REJECTED">已拒绝</el-radio-button>
          </el-radio-group>
        </div>
      </div>

      <div class="wanted-grid">
        <div
          v-for="group in wantedGroups"
          :key="`${group.holderId}-${group.position}`"
          class="wanted-card"
          :class="group.bestStatus.toLowerCase()"
        >
          <!-- 卡片头部：签位 + 持有者 + 状态 -->
          <div class="wanted-header">
            <div class="wanted-header-left">
              <div class="pick-badge" :class="getPickClass(group.position)">
                第{{ group.position }}签
              </div>
              <span class="holder-name">{{ group.holderTeamName }} 持有</span>
            </div>
            <div class="wanted-header-right">
              <el-tag v-if="group.bidders.length >= 2" size="small" type="danger" effect="plain" round>
                {{ group.bidders.length }}方竞价
              </el-tag>
              <el-tag
                :type="getWantedStatusType(group.bestStatus)"
                size="small"
                effect="dark"
              >
                {{ getWantedStatusText(group.bestStatus) }}
              </el-tag>
            </div>
          </div>

          <!-- 成交摘要 -->
          <div v-if="group.winnerName" class="wanted-deal-summary">
            <el-icon><CircleCheck /></el-icon>
            <span>{{ group.winnerName }} 以 {{ formatAmount(group.finalPrice || 0) }} 竞得</span>
            <span v-if="group.bidders.length >= 2" class="compete-note">
              （击败{{ group.bidders.length - 1 }}支球队）
            </span>
          </div>

          <!-- 竞价者列表 -->
          <div class="bidder-list">
            <div
              v-for="(bid, i) in group.bidders"
              :key="bid.id"
              class="bidder-row"
              :class="{ winner: bid.status === 'FULFILLED', rejected: bid.status === 'REJECTED', expired: bid.status === 'EXPIRED' }"
            >
              <div class="bidder-rank">
                <span class="rank-num" :class="{ first: i === 0 }">{{ i + 1 }}</span>
              </div>
              <div class="bidder-info">
                <span class="bidder-name">{{ bid.buyer_team_name }}</span>
                <span class="bidder-reason">{{ bid.reason }}</span>
              </div>
              <div class="bidder-price">
                <span class="bid-amount">{{ formatAmount(bid.offer_price) }}</span>
                <span v-if="bid.final_price" class="bid-final">
                  成交 {{ formatAmount(bid.final_price) }}
                </span>
              </div>
              <div class="bidder-status">
                <el-tag
                  :type="getWantedStatusType(bid.status)"
                  size="small"
                  effect="light"
                >
                  {{ getWantedStatusText(bid.status) }}
                </el-tag>
                <span v-if="bid.response_reason" class="bid-response">{{ bid.response_reason }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 无挂牌提示 -->
    <el-empty
      v-if="listings.length === 0 && isAuctionStarted"
      description="本次拍卖没有签位挂牌"
    />

    <!-- 新闻区域 -->
    <div class="news-section">
      <div class="news-header">
        <h2>
          <el-icon><Bell /></el-icon>
          拍卖动态
        </h2>
        <el-radio-group v-model="filterImportance" size="small">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="BREAKING">头条</el-radio-button>
          <el-radio-button value="MAJOR">重要</el-radio-button>
        </el-radio-group>
      </div>

      <el-empty v-if="filteredEvents.length === 0" description="暂无拍卖动态，点击「开始拍卖」开始" />

      <div class="news-list">
        <div
          v-for="event in filteredEvents"
          :key="event.id"
          class="news-row"
          :class="[`importance-${event.importance.toLowerCase()}`, { expanded: isAuctionEventExpanded(event.id) }]"
          @click="toggleAuctionEvent(event.id)"
        >
          <div class="news-row-compact">
            <div class="news-type-icon" :class="event.event_type.toLowerCase()">
              <el-icon v-if="event.event_type === 'AUCTION_START'"><VideoPlay /></el-icon>
              <el-icon v-else-if="event.event_type === 'LISTING_CREATED'"><Tickets /></el-icon>
              <el-icon v-else-if="event.event_type === 'BID_PLACED'"><Money /></el-icon>
              <el-icon v-else-if="event.event_type === 'BID_RAISED'"><Top /></el-icon>
              <el-icon v-else-if="event.event_type === 'SOLD'"><CircleCheck /></el-icon>
              <el-icon v-else-if="event.event_type === 'EXPIRED'"><Clock /></el-icon>
              <el-icon v-else-if="event.event_type === 'AUCTION_END'"><Flag /></el-icon>
              <el-icon v-else><Bell /></el-icon>
            </div>
            <span class="news-level-tag" :class="event.importance.toLowerCase()">
              {{ getImportanceText(event.importance) }}
            </span>
            <span class="news-headline-text">{{ event.headline }}</span>
            <span class="news-badges">
              <span v-if="event.draft_position" class="badge pick">第{{ event.draft_position }}签</span>
              <span v-if="event.amount" class="badge amount">{{ formatAmount(event.amount) }}</span>
              <span v-if="event.team_name" class="badge team">{{ event.team_name }}</span>
            </span>
            <span class="news-round-tag">R{{ event.round }}</span>
            <el-icon class="expand-arrow" :class="{ opened: isAuctionEventExpanded(event.id) }">
              <ArrowRight />
            </el-icon>
          </div>
          <transition name="event-detail">
            <div v-if="isAuctionEventExpanded(event.id)" class="news-row-detail" @click.stop>
              <div class="detail-description">{{ event.description }}</div>
              <div v-if="event.amount" class="detail-tags">
                <span v-if="event.draft_position" class="detail-item">
                  <el-icon><Tickets /></el-icon>
                  第{{ event.draft_position }}签
                </span>
                <span class="detail-item amount">
                  <el-icon><Money /></el-icon>
                  {{ formatAmount(event.amount) }}
                </span>
                <span v-if="event.team_name" class="detail-item team">
                  {{ event.team_name }}
                </span>
              </div>
            </div>
          </transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  ArrowRight,
  Tickets,
  Check,
  Close,
  Money,
  Coin,
  CaretRight,
  DArrowRight,
  CircleCheck,
  Back,
  Bell,
  Goods,
  VideoPlay,
  Top,
  Clock,
  Flag,
} from '@element-plus/icons-vue'
import { useDraftAuctionStore } from '@/stores/useDraftAuctionStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, draftAuctionApi } from '@/api/tauri'
import type { WantedRequestInfo } from '@/api/tauri'
import { formatMoney } from '@/utils'
import { createLogger } from '@/utils/logger'

const logger = createLogger('DraftAuction')

const route = useRoute()
const router = useRouter()
const auctionStore = useDraftAuctionStore()
const gameStore = useGameStore()

// 从 store 获取响应式数据
const {
  events,
  isLoading,
  isAuctionStarted,
  isAuctionCompleted,
  isAuctionInProgress,
  currentRound,
  totalRounds,
  listings,
  auctionStats,
} = storeToRefs(auctionStore)

const { currentSeason: gameSeason } = storeToRefs(gameStore)

// 本地状态
const region = ref((route.params.region as string)?.toLowerCase() || 'lpl')
const regionId = ref<number>(1)
const filterStatus = ref('all')
const filterImportance = ref('all')
const isFinalized = ref(false)
const wantedRequests = ref<WantedRequestInfo[]>([])
const filterWantedStatus = ref('all')
const expandedAuctionEvents = ref(new Set<number>())

// 赛区名称映射
const regionNames: Record<string, string> = {
  lpl: 'LPL 中国赛区',
  lck: 'LCK 韩国赛区',
  lec: 'LEC 欧洲赛区',
  lcs: 'LCS 北美赛区',
}

// 计算属性
const currentSeason = computed(() => `S${gameSeason.value}`)
const regionName = computed(() => regionNames[region.value] || '')
const progressPercentage = computed(() => (currentRound.value / totalRounds.value) * 100)

const statusTagType = computed(() => {
  if (isAuctionCompleted.value) return 'success'
  if (isAuctionInProgress.value) return 'warning'
  return 'info'
})

const statusText = computed(() => {
  if (isAuctionCompleted.value) return '拍卖完成'
  if (isAuctionInProgress.value) return `进行中 - 第${currentRound.value}轮`
  return '准备开始'
})

const filteredListings = computed(() => {
  if (filterStatus.value === 'all') return listings.value
  return listings.value.filter(l => l.status === filterStatus.value)
})

const filteredEvents = computed(() => {
  const sorted = [...events.value].reverse()
  if (filterImportance.value === 'all') return sorted
  return sorted.filter(e => e.importance === filterImportance.value)
})

interface WantedGroup {
  holderTeamName: string
  holderId: number
  position: number
  bidders: WantedRequestInfo[]
  bestStatus: string
  winnerName: string | null
  finalPrice: number | null
}

const wantedGroups = computed<WantedGroup[]>(() => {
  const map = new Map<string, WantedRequestInfo[]>()
  const source = filterWantedStatus.value === 'all'
    ? wantedRequests.value
    : wantedRequests.value.filter(w => w.status === filterWantedStatus.value)

  for (const w of source) {
    const key = `${w.holder_team_id}-${w.target_position}`
    if (!map.has(key)) map.set(key, [])
    map.get(key)!.push(w)
  }

  const groups: WantedGroup[] = []
  for (const [, items] of map) {
    const sorted = [...items].sort((a, b) => b.offer_price - a.offer_price)
    const winner = sorted.find(w => w.status === 'FULFILLED')
    const bestStatus = winner ? 'FULFILLED'
      : sorted.some(w => w.status === 'ACTIVE') ? 'ACTIVE'
      : sorted.every(w => w.status === 'EXPIRED') ? 'EXPIRED'
      : 'REJECTED'

    groups.push({
      holderTeamName: sorted[0].holder_team_name,
      holderId: sorted[0].holder_team_id,
      position: sorted[0].target_position,
      bidders: sorted,
      bestStatus,
      winnerName: winner?.buyer_team_name ?? null,
      finalPrice: winner?.final_price ?? null,
    })
  }

  return groups.sort((a, b) => a.position - b.position)
})

// 获取赛区ID
const getRegionId = async (regionCode: string): Promise<number> => {
  try {
    const regions = await queryApi.getAllRegions()
    const r = regions.find(r => r.code.toLowerCase() === regionCode.toLowerCase())
    return r?.id ?? 1
  } catch (e) {
    logger.error('Failed to get region id:', e)
    return 1
  }
}

// 加载求购数据
const loadWantedRequests = async () => {
  try {
    wantedRequests.value = await draftAuctionApi.getWantedRequests(regionId.value)
  } catch (e) {
    logger.error('Failed to load wanted requests:', e)
  }
}

// 初始化
onMounted(async () => {
  regionId.value = await getRegionId(region.value)
  await auctionStore.loadPickPrices()
  await auctionStore.fetchAuctionStatus(regionId.value)
  await loadWantedRequests()
})

// 监听路由变化
watch(
  () => route.params.region,
  async (newRegion) => {
    if (newRegion && typeof newRegion === 'string') {
      region.value = newRegion.toLowerCase()
      regionId.value = await getRegionId(region.value)
      auctionStore.clearState()
      await auctionStore.fetchAuctionStatus(regionId.value)
      await loadWantedRequests()
    }
  }
)

// 开始拍卖
const handleStartAuction = async () => {
  try {
    await auctionStore.startAuction(regionId.value)
    await loadWantedRequests()
    ElMessage.success('拍卖已开始！')
  } catch (e) {
    logger.error('Failed to start auction:', e)
    ElMessage.error('开始拍卖失败')
  }
}

// 执行下一轮
const handleNextRound = async () => {
  try {
    await auctionStore.executeRound(regionId.value)
    await loadWantedRequests()
    ElMessage.success(`第 ${currentRound.value} 轮完成`)
  } catch (e) {
    logger.error('Failed to execute round:', e)
    ElMessage.error('执行失败')
  }
}

// 快进完成
const handleFastForward = async () => {
  try {
    await ElMessageBox.confirm(
      '将快速完成所有剩余拍卖轮次。是否继续？',
      '快进完成',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    await auctionStore.fastForward(regionId.value)
    await loadWantedRequests()
    ElMessage.success('拍卖已完成！')
  } catch (e) {
    if (e !== 'cancel') {
      logger.error('Failed to fast forward:', e)
      ElMessage.error('快进失败')
    }
  }
}

// 确认结果
const handleFinalize = async () => {
  try {
    await ElMessageBox.confirm(
      '确认拍卖结果？选秀顺位将更新，财务变动将生效。',
      '确认结果',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'success',
      }
    )

    await auctionStore.finalizeAuction(regionId.value)
    isFinalized.value = true
    ElMessage.success('拍卖结果已确认！即将返回选秀...')
    setTimeout(() => {
      router.push(`/draft/${region.value}`)
    }, 1500)
  } catch (e) {
    if (e !== 'cancel') {
      logger.error('Failed to finalize:', e)
      ElMessage.error('确认失败')
    }
  }
}

// 辅助函数
const formatAmount = (amount: number): string => {
  return auctionStore.formatAmount(amount)
}

const getPickClass = (position: number) => {
  if (position === 1) return 'gold'
  if (position === 2) return 'silver'
  if (position === 3) return 'bronze'
  return ''
}

const getListingStatusType = (status: string) => {
  switch (status) {
    case 'ACTIVE': return 'success'
    case 'SOLD': return 'primary'
    case 'EXPIRED': return 'info'
    case 'WITHDRAWN': return 'warning'
    default: return 'info'
  }
}

const getListingStatusText = (status: string) => {
  return auctionStore.getListingStatusText(status)
}

const getImportanceText = (importance: string) => {
  const map: Record<string, string> = {
    'BREAKING': '头条',
    'MAJOR': '重要',
    'NORMAL': '普通',
    'MINOR': '次要',
  }
  return map[importance] || importance
}

const getWantedStatusType = (status: string) => {
  switch (status) {
    case 'ACTIVE': return 'warning'
    case 'FULFILLED': return 'success'
    case 'REJECTED': return 'danger'
    case 'EXPIRED': return 'info'
    default: return 'info'
  }
}

const getWantedStatusText = (status: string) => {
  const map: Record<string, string> = {
    'ACTIVE': '求购中',
    'FULFILLED': '已成交',
    'REJECTED': '已拒绝',
    'EXPIRED': '已过期',
  }
  return map[status] || status
}

function toggleAuctionEvent(id: number) {
  const s = new Set(expandedAuctionEvents.value)
  if (s.has(id)) s.delete(id)
  else s.add(id)
  expandedAuctionEvents.value = s
}

function isAuctionEventExpanded(id: number): boolean {
  return expandedAuctionEvents.value.has(id)
}
</script>

<style scoped>
.draft-auction {
  padding: 0;
}

/* 返回导航 */
.back-nav {
  margin-bottom: 20px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  background: none;
  border: none;
  color: #6b7280;
  font-size: 14px;
  cursor: pointer;
  transition: color 0.2s;
}

.back-btn:hover {
  color: #3b82f6;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

/* 进度卡片 */
.progress-card {
  margin-bottom: 20px;
  border-radius: 12px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-title {
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.progress-text {
  font-size: 14px;
  color: #606266;
}

.round-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 12px;
}

.round-label {
  font-size: 12px;
  color: #c0c4cc;
  text-align: center;
  flex: 1;
  transition: all 0.3s ease;
}

.round-label.active {
  color: #67c23a;
}

.round-label.current {
  color: #409eff;
  font-weight: 600;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
  transition: all 0.3s ease;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.green { background: linear-gradient(135deg, #22c55e, #16a34a); }
.stat-icon.orange { background: linear-gradient(135deg, #f59e0b, #d97706); }
.stat-icon.red { background: linear-gradient(135deg, #ef4444, #dc2626); }
.stat-icon.purple { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 22px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 13px;
  color: #909399;
  margin-top: 4px;
}

/* 操作按钮 */
.action-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

/* 挂牌区域 */
.listings-section {
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.listings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.listing-card {
  background: white;
  border-radius: 12px;
  padding: 16px;
  border: 2px solid #e5e7eb;
  transition: all 0.3s ease;
}

.listing-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.listing-card.active {
  border-color: #67c23a;
  background: linear-gradient(135deg, #f0fdf4, #ffffff);
}

.listing-card.sold {
  border-color: #409eff;
  background: linear-gradient(135deg, #ecf5ff, #ffffff);
}

.listing-card.expired {
  border-color: #909399;
  background: #f5f7fa;
  opacity: 0.8;
}

.listing-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.pick-badge {
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  background: #dbeafe;
  color: #1e40af;
}

.pick-badge.gold {
  background: linear-gradient(135deg, #fef3c7, #fde68a);
  color: #92400e;
}

.pick-badge.silver {
  background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
  color: #374151;
}

.pick-badge.bronze {
  background: linear-gradient(135deg, #fed7aa, #fdba74);
  color: #9a3412;
}

.listing-content {
  margin-bottom: 12px;
}

.seller-info,
.buyer-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.info-label {
  font-size: 12px;
  color: #909399;
}

.team-name {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.team-name.success {
  color: #67c23a;
}

.final-price {
  margin-left: auto;
  font-size: 13px;
  font-weight: 600;
  color: #f59e0b;
}

.price-info {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
}

.price-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.price-row:last-child {
  margin-bottom: 0;
}

.price-row.current {
  padding-top: 8px;
  border-top: 1px dashed #dcdfe6;
}

.price-label {
  font-size: 12px;
  color: #909399;
}

.price-value {
  font-size: 14px;
  font-weight: 500;
  color: #606266;
}

.price-value.highlight {
  font-size: 16px;
  font-weight: 700;
  color: #f59e0b;
}

.listing-footer {
  display: flex;
  justify-content: flex-end;
}

.bid-round {
  font-size: 12px;
  color: #909399;
}

/* 求购区域 */
.wanted-section {
  margin-bottom: 24px;
}

.wanted-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.wanted-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  border: 2px solid #e5e7eb;
  transition: all 0.3s ease;
}

.wanted-card:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}

.wanted-card.active {
  border-color: #e6a23c;
  background: linear-gradient(135deg, #fdf6ec, #ffffff);
}

.wanted-card.fulfilled {
  border-color: #67c23a;
  background: linear-gradient(135deg, #f0fdf4, #ffffff);
}

.wanted-card.rejected {
  border-color: #f56c6c;
  background: linear-gradient(135deg, #fef0f0, #ffffff);
  opacity: 0.85;
}

.wanted-card.expired {
  border-color: #909399;
  background: #f5f7fa;
  opacity: 0.8;
}

.wanted-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.wanted-header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.holder-name {
  font-size: 14px;
  color: #606266;
}

.wanted-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.wanted-deal-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 14px;
  background: linear-gradient(135deg, #ecfdf5, #d1fae5);
  border-radius: 8px;
  margin-bottom: 14px;
  font-size: 14px;
  font-weight: 600;
  color: #065f46;
}

.compete-note {
  font-weight: 400;
  color: #6b7280;
  font-size: 13px;
}

.bidder-list {
  display: flex;
  flex-direction: column;
  gap: 0;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  overflow: hidden;
}

.bidder-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-bottom: 1px solid #f2f3f5;
  transition: background 0.2s;
}

.bidder-row:last-child {
  border-bottom: none;
}

.bidder-row:hover {
  background: #f9fafb;
}

.bidder-row.winner {
  background: #f0fdf4;
}

.bidder-row.rejected {
  opacity: 0.7;
}

.bidder-row.expired {
  opacity: 0.55;
}

.bidder-rank {
  flex-shrink: 0;
  width: 28px;
  text-align: center;
}

.rank-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 700;
  background: #f3f4f6;
  color: #6b7280;
}

.rank-num.first {
  background: linear-gradient(135deg, #fef3c7, #fde68a);
  color: #92400e;
}

.bidder-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bidder-name {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.bidder-reason {
  font-size: 12px;
  color: #909399;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bidder-price {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.bid-amount {
  font-size: 14px;
  font-weight: 700;
  color: #f59e0b;
}

.bid-final {
  font-size: 12px;
  font-weight: 600;
  color: #16a34a;
}

.bidder-status {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  min-width: 80px;
}

.bid-response {
  font-size: 11px;
  color: #909399;
  text-align: right;
}

/* 新闻区域 */
.news-section {
  margin-bottom: 20px;
}

.news-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.news-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

/* 新闻列表 */
.news-list {
  display: flex;
  flex-direction: column;
  gap: 0;
  border: 1px solid #ebeef5;
  border-radius: 10px;
  overflow: hidden;
  background: white;
}

.news-row {
  cursor: pointer;
  transition: background 0.15s;
}

.news-row:hover {
  background: #fafafa;
}

.news-row + .news-row {
  border-top: 1px solid #f2f3f5;
}

.news-row.importance-breaking {
  border-left: 4px solid #ef4444;
  background: linear-gradient(90deg, #fff5f5, white);
}

.news-row.importance-major {
  border-left: 4px solid #f59e0b;
  background: linear-gradient(90deg, #fffdf5, white);
}

.news-row.importance-normal {
  border-left: 3px solid #3b82f6;
}

.news-row.importance-minor {
  border-left: 3px solid #e5e7eb;
}

.news-row.expanded {
  background: #f9fafb;
}

.news-row-compact {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  min-height: 40px;
}

.news-type-icon {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 14px;
  flex-shrink: 0;
}

.news-type-icon.auction_start { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.news-type-icon.listing_created { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.news-type-icon.bid_placed { background: linear-gradient(135deg, #22c55e, #16a34a); }
.news-type-icon.bid_raised { background: linear-gradient(135deg, #f59e0b, #d97706); }
.news-type-icon.sold { background: linear-gradient(135deg, #10b981, #059669); }
.news-type-icon.expired { background: linear-gradient(135deg, #9ca3af, #6b7280); }
.news-type-icon.withdrawn { background: linear-gradient(135deg, #ef4444, #dc2626); }
.news-type-icon.auction_end { background: linear-gradient(135deg, #06b6d4, #0891b2); }
.news-type-icon.wanted_created { background: linear-gradient(135deg, #e6a23c, #d97706); }
.news-type-icon.wanted_accepted { background: linear-gradient(135deg, #10b981, #059669); }
.news-type-icon.wanted_rejected { background: linear-gradient(135deg, #ef4444, #dc2626); }

.news-level-tag {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 3px;
  color: white;
  flex-shrink: 0;
}

.news-level-tag.breaking { background: #ef4444; }
.news-level-tag.major { background: #f59e0b; }
.news-level-tag.normal { background: #3b82f6; }
.news-level-tag.minor { background: #9ca3af; }

.news-headline-text {
  font-size: 13px;
  font-weight: 500;
  color: #303133;
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.news-badges {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.news-badges .badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  white-space: nowrap;
}

.news-badges .badge.pick {
  background: #dbeafe;
  color: #1e40af;
}

.news-badges .badge.amount {
  background: #fef3c7;
  color: #d97706;
  font-weight: 600;
}

.news-badges .badge.team {
  background: #f0f9ff;
  color: #0284c7;
}

.news-round-tag {
  font-size: 11px;
  color: #909399;
  flex-shrink: 0;
}

.expand-arrow {
  font-size: 12px;
  color: #c0c4cc;
  transition: transform 0.25s ease;
  flex-shrink: 0;
}

.expand-arrow.opened {
  transform: rotate(90deg);
  color: #409eff;
}

.news-row-detail {
  padding: 0 16px 12px 52px;
}

.news-row-detail .detail-description {
  font-size: 13px;
  color: #606266;
  line-height: 1.5;
  margin-bottom: 8px;
}

.news-row-detail .detail-tags {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.news-row-detail .detail-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #606266;
}

.news-row-detail .detail-item.amount {
  font-weight: 600;
  color: #f59e0b;
}

.news-row-detail .detail-item.team {
  padding: 4px 10px;
  background: #f0f9ff;
  border-radius: 4px;
  color: #0284c7;
  font-weight: 500;
}

.event-detail-enter-active,
.event-detail-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}

.event-detail-enter-from,
.event-detail-leave-to {
  opacity: 0;
  max-height: 0;
}

.event-detail-enter-to,
.event-detail-leave-from {
  opacity: 1;
  max-height: 200px;
}
</style>
