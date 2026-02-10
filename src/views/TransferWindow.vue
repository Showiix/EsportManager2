<template>
  <div class="transfer-window">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push('/transfer')">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回转会系统</span>
      </button>
    </div>

    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>全球转会期</h1>
        <p>S{{ currentSeason }} 赛季 - 休赛期 · LPL / LCK / LEC / LCS</p>
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
        <span class="progress-title">转会进度</span>
        <span class="progress-text">第 {{ currentRound }} / {{ totalRounds }} 轮 - {{ currentRoundName }}</span>
      </div>
      <el-progress
        :percentage="progressPercentage"
        :stroke-width="12"
        :format="() => ''"
        :status="isWindowCompleted ? 'success' : undefined"
      />
      <div class="round-labels">
        <span
          v-for="round in 7"
          :key="round"
          class="round-label"
          :class="{ active: round <= currentRound, current: round === currentRound + 1 }"
        >
          {{ getRoundName(round) }}
        </span>
      </div>
    </el-card>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="24"><Document /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.total }}</div>
              <div class="stat-label">总事件</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="24"><UserFilled /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.transfers }}</div>
              <div class="stat-label">转入签约</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="24"><EditPen /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.renewals }}</div>
              <div class="stat-label">续约成功</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="24"><Close /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.terminations }}</div>
              <div class="stat-label">续约失败</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon gray">
              <el-icon :size="24"><Trophy /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.retirements }}</div>
              <div class="stat-label">退役</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon gold">
              <el-icon :size="24"><Money /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatAmount(stats.totalFees) }}</div>
              <div class="stat-label">转会费</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <el-button
        v-if="!isWindowStarted"
        type="primary"
        size="large"
        :loading="isLoading"
        @click="handleStartWindow"
      >
        <el-icon><VideoPlay /></el-icon>
        开始转会期
      </el-button>
      <el-button
        v-if="isWindowInProgress && !isAwaitingClose"
        type="primary"
        size="large"
        :loading="isLoading"
        @click="handleNextRound"
      >
        <el-icon><CaretRight /></el-icon>
        执行下一轮
      </el-button>
      <el-button
        v-if="isWindowInProgress && !isAwaitingClose"
        type="warning"
        size="large"
        :loading="isLoading"
        @click="handleFastForward"
      >
        <el-icon><DArrowRight /></el-icon>
        快进完成
      </el-button>
      <el-button
        v-if="isAwaitingClose"
        type="success"
        size="large"
        :loading="isLoading"
        @click="handleConfirmClose"
      >
        <el-icon><CircleCheck /></el-icon>
        确认关闭转会窗口
      </el-button>
      <el-button
        v-if="isWindowCompleted"
        type="success"
        size="large"
        @click="goToReport"
      >
        <el-icon><DataAnalysis /></el-icon>
        查看转会报告
      </el-button>
      <el-button
        v-if="currentRound >= 5 || isWindowCompleted"
        type="primary"
        size="large"
        @click="goToBidAnalysis"
      >
        <el-icon><TrendCharts /></el-icon>
        竞价分析
      </el-button>
      <el-button @click="$router.push('/transfer')">
        <el-icon><Back /></el-icon>
        返回
      </el-button>
    </div>

    <!-- 事件列表 -->
    <div class="events-section">
      <div class="section-header">
        <h2>
          <el-icon><Bell /></el-icon>
          转会动态
          <span class="event-count">({{ filteredEvents.length }} 条)</span>
        </h2>
        <div class="filter-group">
          <!-- 赛区筛选 -->
          <el-select
            v-model="filterRegion"
            placeholder="全部赛区"
            clearable
            size="small"
            style="width: 120px"
          >
            <el-option label="全部赛区" value="" />
            <el-option
              v-for="r in availableRegions"
              :key="r"
              :label="r"
              :value="r"
            />
          </el-select>

          <!-- 战队筛选 -->
          <el-select
            v-model="filterTeam"
            placeholder="全部战队"
            clearable
            filterable
            size="small"
            style="width: 140px"
          >
            <el-option label="全部战队" value="" />
            <el-option
              v-for="t in availableTeams"
              :key="t"
              :label="t"
              :value="t"
            />
          </el-select>

          <!-- 类型筛选 -->
          <el-radio-group v-model="filterType" size="small">
            <el-radio-button value="all">全部</el-radio-button>
            <el-radio-button value="settlement">能力变化</el-radio-button>
            <el-radio-button value="renewal">续约</el-radio-button>
            <el-radio-button value="transfer">转会签约</el-radio-button>
            <el-radio-button value="listed">挂牌/求转</el-radio-button>
            <el-radio-button value="retirement">退役</el-radio-button>
          </el-radio-group>
        </div>
      </div>

      <el-empty v-if="filteredEvents.length === 0" description="暂无转会动态，点击「开始转会期」开始" />

      <div class="events-list">
        <div v-for="group in groupedEvents" :key="group.round" class="round-group">
          <!-- 轮次分组 header -->
          <div class="round-group-header" @click="toggleRound(group.round)">
            <el-icon class="collapse-arrow" :class="{ expanded: isRoundExpanded(group.round) }">
              <ArrowRight />
            </el-icon>
            <span class="round-group-title">第{{ group.round }}轮 · {{ group.name }}</span>
            <span class="round-event-count">{{ group.events.length }} 条</span>
            <span class="round-group-summary">
              <span v-if="group.settlementCount" class="summary-tag settlement">{{ group.settlementCount }}结算</span>
              <span v-if="group.renewalCount" class="summary-tag renewal">{{ group.renewalCount }}续约</span>
              <span v-if="group.terminationCount" class="summary-tag termination">{{ group.terminationCount }}失败</span>
              <span v-if="group.transferCount" class="summary-tag transfer">{{ group.transferCount }}签约</span>
              <span v-if="group.listedCount" class="summary-tag listed">{{ group.listedCount }}挂牌</span>
              <span v-if="group.retirementCount" class="summary-tag retirement">{{ group.retirementCount }}退役</span>
            </span>
          </div>

          <!-- 轮次事件内容 -->
          <transition name="round-collapse">
            <div v-if="isRoundExpanded(group.round)" class="round-group-body">
              <template v-for="event in group.events" :key="event.id">
                <!-- S级：大卡片 -->
                <div
                  v-if="event.level === 'S'"
                  class="event-featured"
                  :class="`type-${event.event_type.toLowerCase()}`"
                >
                  <div class="featured-level">
                    {{ getLevelLabel(event.level) }}
                  </div>
                  <div class="event-type-icon" :class="event.event_type.toLowerCase()">
                    <el-icon v-if="event.event_type === 'CONTRACT_RENEWAL'"><Check /></el-icon>
                    <el-icon v-else-if="event.event_type === 'CONTRACT_TERMINATION'"><Close /></el-icon>
                    <el-icon v-else-if="event.event_type === 'FREE_AGENT_SIGNING'"><EditPen /></el-icon>
                    <el-icon v-else-if="event.event_type === 'TRANSFER_PURCHASE'"><Money /></el-icon>
                    <el-icon v-else-if="event.event_type === 'PLAYER_RETIREMENT'"><Trophy /></el-icon>
                    <el-icon v-else-if="event.event_type === 'PLAYER_LISTED'"><Sell /></el-icon>
                    <el-icon v-else-if="event.event_type === 'EMERGENCY_SIGNING'"><Lightning /></el-icon>
                    <el-icon v-else-if="event.event_type === 'SEASON_SETTLEMENT'"><Calendar /></el-icon>
                    <el-icon v-else><Bell /></el-icon>
                  </div>
                  <div class="event-content">
                    <div class="event-headline">{{ getEventHeadline(event) }}</div>
                    <div class="event-description">{{ getEventDescription(event) }}</div>
                    <div class="event-details">
                      <span v-if="event.transfer_fee > 0" class="detail-item fee">
                        <el-icon><Money /></el-icon>
                        转会费 {{ formatAmount(event.transfer_fee) }}
                      </span>
                      <span v-if="event.salary > 0" class="detail-item salary">
                        <el-icon><Wallet /></el-icon>
                        年薪 {{ formatAmount(event.salary) }}
                      </span>
                      <span v-if="event.contract_years > 0" class="detail-item contract">
                        <el-icon><Calendar /></el-icon>
                        {{ event.contract_years }}年合同
                      </span>
                    </div>
                  </div>
                </div>

                <!-- A/B/C级：紧凑行 -->
                <div
                  v-else
                  class="event-compact"
                  :class="[`level-${event.level.toLowerCase()}`, `type-${event.event_type.toLowerCase()}`]"
                >
                  <div class="compact-icon" :class="event.event_type.toLowerCase()">
                    <el-icon v-if="event.event_type === 'CONTRACT_RENEWAL'"><Check /></el-icon>
                    <el-icon v-else-if="event.event_type === 'CONTRACT_TERMINATION'"><Close /></el-icon>
                    <el-icon v-else-if="event.event_type === 'FREE_AGENT_SIGNING'"><EditPen /></el-icon>
                    <el-icon v-else-if="event.event_type === 'TRANSFER_PURCHASE'"><Money /></el-icon>
                    <el-icon v-else-if="event.event_type === 'PLAYER_RETIREMENT'"><Trophy /></el-icon>
                    <el-icon v-else-if="event.event_type === 'PLAYER_LISTED'"><Sell /></el-icon>
                    <el-icon v-else-if="event.event_type === 'EMERGENCY_SIGNING'"><Lightning /></el-icon>
                    <el-icon v-else-if="event.event_type === 'SEASON_SETTLEMENT'"><Calendar /></el-icon>
                    <el-icon v-else><Bell /></el-icon>
                  </div>
                  <span class="compact-level" :class="event.level.toLowerCase()">
                    {{ getLevelLabel(event.level) }}
                  </span>
                  <span class="compact-headline">{{ getEventHeadline(event) }}</span>
                  <span class="compact-badges">
                    <span v-if="event.reason && event.event_type === 'SEASON_SETTLEMENT'" class="badge settlement-detail">{{ event.reason }}</span>
                    <span v-if="event.transfer_fee > 0" class="badge fee">{{ formatAmount(event.transfer_fee) }}</span>
                    <span v-if="event.salary > 0" class="badge salary">{{ formatAmount(event.salary) }}/年</span>
                    <span v-if="event.contract_years > 0" class="badge contract">{{ event.contract_years }}年</span>
                  </span>
                </div>
              </template>
            </div>
          </transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  ArrowRight,
  VideoPlay,
  CaretRight,
  DArrowRight,
  DataAnalysis,
  Back,
  Bell,
  Document,
  UserFilled,
  EditPen,
  Close,
  Trophy,
  Money,
  Check,
  Sell,
  Calendar,
  Wallet,
  TrendCharts,
  CircleCheck,
} from '@element-plus/icons-vue'
import { useTransferWindowStore, ROUND_NAMES, EVENT_TYPE_NAMES, EVENT_LEVEL_CONFIG } from '@/stores/useTransferWindowStore'
import { useGameStore } from '@/stores/useGameStore'
import type { TransferEvent } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TransferWindow')

// 添加 Lightning 图标替代
const Lightning = EditPen // 使用 EditPen 作为替代

const router = useRouter()
const transferStore = useTransferWindowStore()
const gameStore = useGameStore()

const { currentSeason } = storeToRefs(gameStore)
const {
  isLoading,
  windowInfo,
  events,
  isWindowStarted,
  isWindowInProgress,
  isWindowCompleted,
  isAwaitingClose,
  currentRound,
  totalRounds,
  progressPercentage,
  currentRoundName,
  stats,
} = storeToRefs(transferStore)

// 本地状态
const filterType = ref('all')
const filterRegion = ref('')
const filterTeam = ref('')

// 计算属性
const statusTagType = computed(() => {
  if (isWindowCompleted.value) return 'success'
  if (isWindowInProgress.value) return 'warning'
  return 'info'
})

const statusText = computed(() => {
  if (isWindowCompleted.value) return '转会完成'
  if (isWindowInProgress.value) return `进行中 - 第${currentRound.value}轮`
  return '准备开始'
})

// 从事件中提取可用的赛区列表（根据战队名称前缀判断）
const availableRegions = computed(() => {
  const regions = new Set<string>()
  for (const event of events.value) {
    const teamName = event.from_team_name || event.to_team_name
    if (teamName) {
      // 尝试从战队名称匹配赛区
      if (teamName.includes('Gaming') || teamName.includes('Esports') || teamName.includes('Top') ||
          teamName.includes('Weibo') || teamName.includes('JD') || teamName.includes('LNG') ||
          teamName.includes('RNG') || teamName.includes('FPX') || teamName.includes('EDG') ||
          teamName.includes('BLG') || teamName.includes('IG') || teamName.includes('NIP')) {
        regions.add('LPL')
      }
      if (teamName.includes('T1') || teamName.includes('Gen') || teamName.includes('HLE') ||
          teamName.includes('DRX') || teamName.includes('DK') || teamName.includes('KT') ||
          teamName.includes('Sandbox') || teamName.includes('Freecs') || teamName.includes('BRION') ||
          teamName.includes('Nongshim') || teamName.includes('FearX') || teamName.includes('Longzhu')) {
        regions.add('LCK')
      }
      if (teamName.includes('Fnatic') || teamName.includes('G2') || teamName.includes('MAD') ||
          teamName.includes('Heretics') || teamName.includes('Vitality') || teamName.includes('Excel') ||
          teamName.includes('Misfits') || teamName.includes('Astralis') || teamName.includes('SK')) {
        regions.add('LEC')
      }
      if (teamName.includes('100') || teamName.includes('Cloud9') || teamName.includes('Liquid') ||
          teamName.includes('TSM') || teamName.includes('CLG') || teamName.includes('Dignitas') ||
          teamName.includes('EG') || teamName.includes('NRG') || teamName.includes('Immortals')) {
        regions.add('LCS')
      }
    }
  }
  return Array.from(regions).sort()
})

// 从事件中提取可用的战队列表
const availableTeams = computed(() => {
  const teams = new Set<string>()
  for (const event of events.value) {
    if (event.from_team_name) teams.add(event.from_team_name)
    if (event.to_team_name) teams.add(event.to_team_name)
  }
  return Array.from(teams).filter(t => t && t !== '自由球员').sort()
})

const filteredEvents = computed(() => {
  let result = [...events.value].reverse()

  // 按事件类型分类筛选
  if (filterType.value !== 'all') {
    const typeGroups: Record<string, string[]> = {
      settlement: ['SEASON_SETTLEMENT'],
      renewal: ['CONTRACT_RENEWAL', 'CONTRACT_TERMINATION'],
      transfer: ['FREE_AGENT_SIGNING', 'TRANSFER_PURCHASE', 'EMERGENCY_SIGNING'],
      listed: ['PLAYER_LISTED', 'PLAYER_REQUEST_TRANSFER', 'PLAYER_RELEASE'],
      retirement: ['PLAYER_RETIREMENT'],
    }
    const allowedTypes = typeGroups[filterType.value] ?? []
    result = result.filter(e => allowedTypes.includes(e.event_type))
  }

  // 按战队筛选
  if (filterTeam.value) {
    result = result.filter(e =>
      e.from_team_name === filterTeam.value || e.to_team_name === filterTeam.value
    )
  }

  // 按赛区筛选（通过战队名称匹配）
  if (filterRegion.value) {
    result = result.filter(e => {
      const teamName = e.from_team_name || e.to_team_name || ''
      switch (filterRegion.value) {
        case 'LPL':
          return teamName.includes('Gaming') || teamName.includes('Esports') || teamName.includes('Top') ||
                 teamName.includes('Weibo') || teamName.includes('JD') || teamName.includes('LNG') ||
                 teamName.includes('RNG') || teamName.includes('FPX') || teamName.includes('EDG') ||
                 teamName.includes('BLG') || teamName.includes('IG') || teamName.includes('NIP') ||
                 teamName.includes('TT') || teamName.includes('AL') || teamName.includes('UP') ||
                 teamName.includes('Mercury')
        case 'LCK':
          return teamName.includes('T1') || teamName.includes('Gen') || teamName.includes('HLE') ||
                 teamName.includes('DRX') || teamName.includes('DK') || teamName.includes('KT') ||
                 teamName.includes('Sandbox') || teamName.includes('Freecs') || teamName.includes('BRION') ||
                 teamName.includes('Nongshim') || teamName.includes('FearX') || teamName.includes('Longzhu') ||
                 teamName.includes('BNK') || teamName.includes('Afreeca')
        case 'LEC':
          return teamName.includes('Fnatic') || teamName.includes('G2') || teamName.includes('MAD') ||
                 teamName.includes('Heretics') || teamName.includes('Vitality') || teamName.includes('Excel') ||
                 teamName.includes('Misfits') || teamName.includes('Astralis') || teamName.includes('SK') ||
                 teamName.includes('Whales') || teamName.includes('Falcons') || teamName.includes('Wolf') ||
                 teamName.includes('Nike') || teamName.includes('AmBear')
        case 'LCS':
          return teamName.includes('100') || teamName.includes('Cloud9') || teamName.includes('Liquid') ||
                 teamName.includes('TSM') || teamName.includes('CLG') || teamName.includes('Dignitas') ||
                 teamName.includes('EG') || teamName.includes('NRG') || teamName.includes('Immortals') ||
                 teamName.includes('Frost') || teamName.includes('Shopify') || teamName.includes('Logic')
        default:
          return true
      }
    })
  }

  // 重置分页
  return result
})

// 按轮次分组
interface EventGroup {
  round: number
  name: string
  events: TransferEvent[]
  settlementCount: number
  renewalCount: number
  terminationCount: number
  transferCount: number
  listedCount: number
  retirementCount: number
}

const groupedEvents = computed<EventGroup[]>(() => {
  const groups = new Map<number, TransferEvent[]>()
  for (const event of filteredEvents.value) {
    if (!groups.has(event.round)) groups.set(event.round, [])
    groups.get(event.round)!.push(event)
  }
  return Array.from(groups.entries())
    .sort((a, b) => b[0] - a[0])
    .map(([round, evts]) => ({
      round,
      name: getRoundName(round),
      events: evts,
      settlementCount: evts.filter(e => e.event_type === 'SEASON_SETTLEMENT').length,
      renewalCount: evts.filter(e => e.event_type === 'CONTRACT_RENEWAL').length,
      terminationCount: evts.filter(e => e.event_type === 'CONTRACT_TERMINATION').length,
      transferCount: evts.filter(e => e.event_type === 'FREE_AGENT_SIGNING' || e.event_type === 'TRANSFER_PURCHASE' || e.event_type === 'EMERGENCY_SIGNING').length,
      listedCount: evts.filter(e => e.event_type === 'PLAYER_LISTED' || e.event_type === 'PLAYER_REQUEST_TRANSFER' || e.event_type === 'PLAYER_RELEASE').length,
      retirementCount: evts.filter(e => e.event_type === 'PLAYER_RETIREMENT').length,
    }))
})

// 轮次折叠状态
const expandedRounds = ref(new Set<number>())

// 初始展开最新轮次
watch(groupedEvents, (groups) => {
  if (groups.length > 0 && expandedRounds.value.size === 0) {
    expandedRounds.value.add(groups[0].round)
  }
}, { immediate: true })

function toggleRound(round: number) {
  const newSet = new Set(expandedRounds.value)
  if (newSet.has(round)) {
    newSet.delete(round)
  } else {
    newSet.add(round)
  }
  expandedRounds.value = newSet
}

function isRoundExpanded(round: number): boolean {
  return expandedRounds.value.has(round)
}

watch([filterType, filterRegion, filterTeam], () => {
  expandedRounds.value = new Set<number>()
})

// 开始转会期
async function handleStartWindow() {
  try {
    await transferStore.startTransferWindow()
    ElMessage.success('转会期已开始！')
  } catch (e) {
    logger.error('Failed to start window:', e)
    ElMessage.error('开始转会期失败')
  }
}

// 执行下一轮
async function handleNextRound() {
  try {
    const result = await transferStore.executeRound()
    ElMessage.success(`${result.round_name} 完成，${result.event_count} 个事件`)
  } catch (e) {
    logger.error('Failed to execute round:', e)
    ElMessage.error('执行失败')
  }
}

// 快进完成
async function handleFastForward() {
  try {
    await ElMessageBox.confirm(
      '将快速完成所有剩余转会轮次。是否继续？',
      '快进完成',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    const result = await transferStore.fastForward()
    ElMessage.success(`转会期完成！共 ${result.total_events} 个事件`)
  } catch (e) {
    if (e !== 'cancel') {
      logger.error('Failed to fast forward:', e)
      ElMessage.error('快进失败')
    }
  }
}

// 确认关闭转会窗口
async function handleConfirmClose() {
  try {
    const result = await transferStore.confirmCloseWindow(false)

    if (result.is_valid) {
      ElMessage.success('转会窗口验证通过，已成功关闭！')
    } else {
      // 显示问题列表，让用户选择强制关闭或返回修复
      const issueList = result.issues.map(
        (issue) => `${issue.team_name}: ${issue.detail}`
      ).join('\n')

      await ElMessageBox.confirm(
        `转会窗口验证未通过，存在以下问题：\n\n${issueList}\n\n是否强制关闭？`,
        '验证未通过',
        {
          confirmButtonText: '强制关闭',
          cancelButtonText: '返回修复',
          type: 'warning',
          dangerouslyUseHTMLString: false,
        }
      )

      // 用户选择强制关闭
      const forceResult = await transferStore.confirmCloseWindow(true)
      ElMessage.warning(forceResult.message)
    }
  } catch (e) {
    if (e !== 'cancel') {
      logger.error('Failed to close transfer window:', e)
      ElMessage.error('关闭转会窗口失败')
    }
  }
}

// 查看报告
function goToReport() {
  if (windowInfo.value) {
    router.push(`/transfer/report/${windowInfo.value.window_id}`)
  }
}

// 查看竞价分析
function goToBidAnalysis() {
  if (windowInfo.value) {
    router.push({
      path: '/transfer/bid-analysis',
      query: {
        windowId: String(windowInfo.value.window_id),
        seasonId: String(windowInfo.value.season_id),
      },
    })
  }
}

// 辅助函数
function getRoundName(round: number): string {
  return ROUND_NAMES[round] ?? `第${round}轮`
}

function formatAmount(amount: number): string {
  return transferStore.formatAmount(amount)
}

function getLevelLabel(level: string): string {
  return EVENT_LEVEL_CONFIG[level]?.label ?? level
}

function getEventTypeName(eventType: string): string {
  return EVENT_TYPE_NAMES[eventType] ?? eventType
}

function getEventHeadline(event: TransferEvent): string {
  const typeName = getEventTypeName(event.event_type)

  switch (event.event_type) {
    case 'CONTRACT_RENEWAL':
      return `${event.player_name} 续约成功`
    case 'CONTRACT_TERMINATION':
      return `${event.player_name} 续约失败，成为自由球员`
    case 'FREE_AGENT_SIGNING':
      return `${event.to_team_name} 签下自由球员 ${event.player_name}`
    case 'TRANSFER_PURCHASE':
      return `${event.to_team_name} 买断 ${event.player_name}`
    case 'PLAYER_RETIREMENT':
      return `${event.player_name} 宣布退役`
    case 'PLAYER_LISTED':
      return `${event.from_team_name} 将 ${event.player_name} 挂牌出售`
    case 'EMERGENCY_SIGNING':
      return `${event.to_team_name} 紧急签约 ${event.player_name}`
    case 'SEASON_SETTLEMENT':
      return `${event.player_name} 赛季数据结算`
    default:
      return `${event.player_name} - ${typeName}`
  }
}

function getEventDescription(event: TransferEvent): string {
  const ability = event.player_ability

  switch (event.event_type) {
    case 'CONTRACT_RENEWAL':
      return `${event.to_team_name || event.from_team_name} 与 ${event.player_name}(${ability}能力) 续约成功`
    case 'CONTRACT_TERMINATION':
      return `${event.from_team_name} 与 ${event.player_name}(${ability}能力) 续约谈判破裂`
    case 'FREE_AGENT_SIGNING':
      return `${event.player_name}(${ability}能力) 以自由身加盟 ${event.to_team_name}`
    case 'TRANSFER_PURCHASE':
      return `${event.player_name}(${ability}能力) 从 ${event.from_team_name} 转会至 ${event.to_team_name}`
    case 'PLAYER_RETIREMENT':
      return `${event.player_name}(${ability}能力) 结束职业生涯，感谢付出`
    case 'EMERGENCY_SIGNING':
      return `${event.to_team_name} 因阵容不足紧急签下 ${event.player_name}`
    default:
      return event.reason || ''
  }
}

// 页面加载时恢复转会期状态
onMounted(async () => {
  await transferStore.initTransferWindow()
})
</script>

<style scoped>
.transfer-window {
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
  font-size: 11px;
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
.stat-icon.purple { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.stat-icon.orange { background: linear-gradient(135deg, #f59e0b, #d97706); }
.stat-icon.gray { background: linear-gradient(135deg, #6b7280, #4b5563); }
.stat-icon.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 20px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

/* 操作按钮 */
.action-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

/* 事件区域 */
.events-section {
  margin-bottom: 20px;
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

/* 事件列表 */
.events-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* ===== 轮次分组 ===== */
.round-group {
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid #ebeef5;
  background: white;
}

.round-group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #f5f7fa;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  user-select: none;
  transition: background 0.2s;
}

.round-group-header:hover {
  background: #ebeef5;
}

.collapse-arrow {
  transition: transform 0.3s ease;
  font-size: 12px;
  color: #909399;
}

.collapse-arrow.expanded {
  transform: rotate(90deg);
}

.round-group-title {
  flex-shrink: 0;
}

.round-event-count {
  font-size: 12px;
  font-weight: 400;
  color: #909399;
}

.round-group-summary {
  display: flex;
  gap: 6px;
  margin-left: auto;
}

.summary-tag {
  font-size: 11px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: 3px;
}

.summary-tag.settlement {
  background: #e0f2fe;
  color: #0284c7;
}

.summary-tag.renewal {
  background: #dcfce7;
  color: #16a34a;
}

.summary-tag.termination {
  background: #fee2e2;
  color: #dc2626;
}

.summary-tag.transfer {
  background: #fef3c7;
  color: #d97706;
}

.summary-tag.listed {
  background: #f3e8ff;
  color: #7c3aed;
}

.summary-tag.retirement {
  background: #f3f4f6;
  color: #4b5563;
}

.round-group-body {
  display: flex;
  flex-direction: column;
}

/* 折叠过渡 */
.round-collapse-enter-active,
.round-collapse-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.round-collapse-enter-from,
.round-collapse-leave-to {
  opacity: 0;
  max-height: 0;
}

/* ===== S级大卡片 ===== */
.event-featured {
  background: linear-gradient(135deg, #fffbeb, #ffffff);
  border-left: 4px solid #f59e0b;
  padding: 12px 16px;
  display: flex;
  gap: 12px;
  position: relative;
  transition: background 0.2s;
}

.event-featured:hover {
  background: linear-gradient(135deg, #fef9c3, #fffbeb);
}

.featured-level {
  position: absolute;
  top: 8px;
  right: 12px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  color: white;
  background: #f59e0b;
}

.event-featured .event-type-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  font-size: 20px;
}

.event-featured .event-content {
  flex: 1;
  min-width: 0;
}

.event-featured .event-headline {
  font-size: 15px;
  font-weight: 700;
  color: #303133;
  margin-bottom: 4px;
  padding-right: 50px;
}

.event-featured .event-description {
  font-size: 13px;
  color: #606266;
  margin-bottom: 8px;
  line-height: 1.4;
}

.event-featured .event-details {
  display: flex;
  gap: 12px;
  align-items: center;
}

.event-featured .detail-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #606266;
}

.event-featured .detail-item.fee {
  font-weight: 600;
  color: #f59e0b;
}

.event-featured .detail-item.salary {
  color: #22c55e;
}

.event-featured .detail-item.contract {
  color: #3b82f6;
}

/* ===== A/B/C级紧凑行 ===== */
.event-compact {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: white;
  transition: background 0.15s;
  min-height: 40px;
}

.event-compact:hover {
  background: #fafafa;
}

.event-compact.level-a {
  border-left: 3px solid #8b5cf6;
}

.event-compact.level-b {
  border-left: 3px solid #3b82f6;
}

.event-compact.level-c {
  border-left: 3px solid #e5e7eb;
}

.compact-icon {
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

.compact-icon.contract_renewal { background: linear-gradient(135deg, #22c55e, #16a34a); }
.compact-icon.contract_termination { background: linear-gradient(135deg, #ef4444, #dc2626); }
.compact-icon.free_agent_signing { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.compact-icon.transfer_purchase { background: linear-gradient(135deg, #f59e0b, #d97706); }
.compact-icon.player_retirement { background: linear-gradient(135deg, #6b7280, #4b5563); }
.compact-icon.player_listed { background: linear-gradient(135deg, #f97316, #ea580c); }
.compact-icon.emergency_signing { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.compact-icon.season_settlement { background: linear-gradient(135deg, #06b6d4, #0891b2); }

.compact-level {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 3px;
  color: white;
  flex-shrink: 0;
}

.compact-level.a { background: #8b5cf6; }
.compact-level.b { background: #3b82f6; }
.compact-level.c { background: #9ca3af; }

.compact-headline {
  font-size: 13px;
  font-weight: 500;
  color: #303133;
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.compact-badges {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  white-space: nowrap;
}

.badge.fee {
  background: #fef3c7;
  color: #d97706;
  font-weight: 600;
}

.badge.salary {
  background: #dcfce7;
  color: #16a34a;
}

.badge.contract {
  background: #dbeafe;
  color: #2563eb;
}

.badge.settlement-detail {
  background: #f3f4f6;
  color: #4b5563;
  font-size: 11px;
  max-width: 280px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ===== 保留的事件类型图标样式（S级用） ===== */
.event-type-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.event-type-icon.contract_renewal { background: linear-gradient(135deg, #22c55e, #16a34a); }
.event-type-icon.contract_termination { background: linear-gradient(135deg, #ef4444, #dc2626); }
.event-type-icon.free_agent_signing { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.event-type-icon.transfer_purchase { background: linear-gradient(135deg, #f59e0b, #d97706); }
.event-type-icon.player_retirement { background: linear-gradient(135deg, #6b7280, #4b5563); }
.event-type-icon.player_listed { background: linear-gradient(135deg, #f97316, #ea580c); }
.event-type-icon.emergency_signing { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.event-type-icon.season_settlement { background: linear-gradient(135deg, #06b6d4, #0891b2); }

/* 事件数量 */
.event-count {
  font-size: 14px;
  font-weight: 400;
  color: #909399;
  margin-left: 8px;
}

/* 筛选组 */
.filter-group {
  display: flex;
  gap: 12px;
  align-items: center;
}
</style>
