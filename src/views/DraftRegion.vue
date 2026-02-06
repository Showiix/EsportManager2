<template>
  <div class="draft-region-view">
    <!-- 返回导航 -->
    <div class="back-nav">
      <button class="back-btn" @click="$router.push('/draft')">
        <el-icon><ArrowLeft /></el-icon>
        <span>返回选秀系统</span>
      </button>
    </div>

    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <div class="region-badge" :class="selectedRegion">
          {{ selectedRegion.toUpperCase() }}
        </div>
        <div class="header-info">
          <h1 class="page-title">{{ regionName }} 选秀</h1>
          <p class="page-desc">{{ currentSeason }} 赛季新秀选拔</p>
        </div>
      </div>
      <div class="header-right">
        <div class="status-badge" :class="getStatusClass()">
          <span class="status-dot"></span>
          <span>{{ getStatusLabel() }}</span>
        </div>
      </div>
    </div>

    <!-- 赛区切换标签 -->
    <div class="region-tabs">
      <button
        v-for="r in regionList"
        :key="r.code"
        class="region-tab"
        :class="{ active: selectedRegion === r.code }"
        @click="handleRegionChange(r.code)"
      >
        <span class="tab-badge" :class="r.code">{{ r.code.toUpperCase() }}</span>
        <span class="tab-name">{{ r.name }}</span>
      </button>
    </div>

    <!-- 进度步骤条 -->
    <div class="progress-steps">
      <div
        v-for="(step, index) in steps"
        :key="index"
        class="step-item"
        :class="{
          'is-active': currentStep === index,
          'is-completed': currentStep > index
        }"
      >
        <div class="step-indicator">
          <el-icon v-if="currentStep > index"><Check /></el-icon>
          <span v-else>{{ index + 1 }}</span>
        </div>
        <div class="step-content">
          <div class="step-title">{{ step.title }}</div>
          <div class="step-desc">{{ step.desc }}</div>
        </div>
        <div v-if="index < steps.length - 1" class="step-line"></div>
      </div>
    </div>

    <!-- 步骤0: 抽取选秀名单 -->
    <div v-if="currentStep === 0" class="content-section">
      <div class="section-header">
        <div class="header-title">
          <el-icon :size="22"><User /></el-icon>
          <h2>抽取选秀名单</h2>
        </div>
      </div>

      <div class="draw-roster-section">
        <div class="draw-info">
          <el-icon :size="48" color="#3b82f6"><Tickets /></el-icon>
          <h3>从选秀池抽取本届选秀名单</h3>
          <p>点击下方按钮，将从选秀池中按能力值排序抽取前14名新秀，组成本届选秀名单</p>
        </div>

        <!-- 非选秀阶段提示 -->
        <div v-if="!isDraftPhase" class="phase-warning">
          <el-icon :size="20" color="#f59e0b"><WarningFilled /></el-icon>
          <span>当前阶段: {{ gameStore.currentPhaseDisplay }}，需要等到选秀阶段才能抽取</span>
        </div>

        <el-button
          type="primary"
          size="large"
          @click="drawDraftRoster"
          :loading="isLoading"
          :disabled="!isDraftPhase"
        >
          <el-icon class="mr-2"><Aim /></el-icon>
          {{ isDraftPhase ? '抽取选秀名单' : '未到选秀阶段' }}
        </el-button>
      </div>
    </div>

    <!-- 步骤1: 选秀名单 -->
    <div v-if="currentStep === 1" class="content-section">
      <div class="section-header">
        <div class="header-title">
          <el-icon :size="22"><User /></el-icon>
          <h2>选秀名单</h2>
        </div>
        <div class="header-meta">
          <span class="meta-count">共 {{ draftPool.length }} 名新秀</span>
        </div>
      </div>

      <div class="prospects-grid">
        <div
          v-for="player in draftPool"
          :key="player.rank"
          class="prospect-card"
          :class="getProspectClass(player.rank)"
        >
          <div class="prospect-rank">
            <span class="rank-number">{{ player.rank }}</span>
            <span class="rank-label">{{ player.title }}</span>
          </div>
          <div class="prospect-info">
            <div class="prospect-name">{{ player.gameId }}</div>
            <div class="prospect-stats">
              <div class="stat-item">
                <span class="stat-value ability" :style="{ color: getAbilityColor(player.ability) }">
                  {{ player.ability }}
                </span>
                <span class="stat-label">能力</span>
              </div>
              <div class="stat-item">
                <span class="stat-value potential">{{ player.potential }}</span>
                <span class="stat-label">潜力</span>
              </div>
            </div>
          </div>
          <div class="prospect-tag" :class="player.tag.toLowerCase()">
            {{ player.tag === 'GENIUS' ? '天才' : '普通' }}
          </div>
        </div>
      </div>

      <div class="section-footer">
        <el-button @click="currentStep = 0">
          <el-icon><ArrowLeft /></el-icon>
          返回上一步
        </el-button>
        <el-button type="primary" size="large" @click="startLottery">
          开始选秀权抽签
          <el-icon class="ml-2"><ArrowRight /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 步骤2: 抽签 -->
    <div v-if="currentStep === 2" class="content-section">
      <div class="section-header">
        <div class="header-title">
          <el-icon :size="22"><Tickets /></el-icon>
          <h2>选秀权抽签</h2>
        </div>
        <div class="header-actions">
          <el-button @click="drawSinglePick" :disabled="!hasUndrawnTeams">
            <el-icon><Aim /></el-icon>
            单个抽签
          </el-button>
          <el-button type="warning" @click="drawAllPicks" :disabled="!hasUndrawnTeams">
            <el-icon><MagicStick /></el-icon>
            一键抽签
          </el-button>
        </div>
      </div>

      <div class="lottery-grid">
        <div
          v-for="(team, index) in lotteryResults"
          :key="team.teamId"
          class="lottery-card"
          :class="{ 'is-drawn': team.pickOrder !== null }"
        >
          <div class="lottery-rank">
            <span class="rank-label">常规赛</span>
            <span class="rank-value">第 {{ index + 1 }} 名</span>
          </div>
          <div class="lottery-team">
            <div class="team-avatar" :class="selectedRegion">
              {{ team.teamName.substring(0, 2) }}
            </div>
            <span class="team-name">{{ team.teamName }}</span>
          </div>
          <div class="lottery-result">
            <template v-if="team.pickOrder !== null">
              <div class="pick-badge" :class="getPickClass(team.pickOrder)">
                第 {{ team.pickOrder }} 顺位
              </div>
              <div class="pick-player">
                <span class="player-tag" :class="getPickLabelClass(team.pickOrder)">
                  {{ draftPool[team.pickOrder - 1]?.title }}
                </span>
                <span class="player-name">{{ draftPool[team.pickOrder - 1]?.gameId }}</span>
              </div>
            </template>
            <div v-else class="pick-waiting">
              <el-icon :size="28" color="#d1d5db"><QuestionFilled /></el-icon>
              <span>待抽签</span>
            </div>
          </div>
        </div>
      </div>

      <div class="section-footer">
        <el-button @click="currentStep = 1">
          <el-icon><ArrowLeft /></el-icon>
          返回上一步
        </el-button>
        <el-button type="primary" size="large" @click="proceedToAuction" :disabled="hasUndrawnTeams">
          进入选秀权拍卖
          <el-icon class="ml-2"><ArrowRight /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 步骤3: 选秀权拍卖 -->
    <div v-if="currentStep === 3" class="content-section">
      <div class="section-header">
        <div class="header-title">
          <el-icon :size="22"><Sell /></el-icon>
          <h2>选秀权拍卖</h2>
        </div>
      </div>

      <div class="auction-section">
        <div class="auction-info">
          <el-icon :size="48" color="#f59e0b"><Money /></el-icon>
          <h3>选秀权交易市场</h3>
          <p>AI球队将根据财务状况和阵容需求自动挂牌/竞拍选秀权</p>
          <p class="sub-info">成交后选秀顺位将自动转移，卖家收取扣除5%联盟佣金后的收益</p>
        </div>

        <div class="auction-actions">
          <el-button type="primary" size="large" @click="goToAuction">
            <el-icon class="mr-2"><Sell /></el-icon>
            进入拍卖大厅
          </el-button>
          <el-button size="large" @click="skipAuction">
            <el-icon class="mr-2"><Right /></el-icon>
            跳过拍卖，直接分配
          </el-button>
        </div>
      </div>

      <div class="section-footer">
        <el-button @click="currentStep = 2">
          <el-icon><ArrowLeft /></el-icon>
          返回上一步
        </el-button>
      </div>
    </div>

    <!-- 步骤4: 分配 -->
    <div v-if="currentStep === 4" class="content-section">
      <div class="section-header">
        <div class="header-title">
          <el-icon :size="22"><Connection /></el-icon>
          <h2>分配选手到队伍</h2>
        </div>
      </div>

      <div class="notice-card">
        <el-icon :size="20" color="#f59e0b"><WarningFilled /></el-icon>
        <div class="notice-text">
          <strong>确认分配</strong>
          <span>点击下方按钮将按照抽签顺位将选手分配到对应队伍</span>
        </div>
      </div>

      <div class="assignment-list">
        <div
          v-for="item in assignmentList"
          :key="`${item.pickOrder}-${item.teamId}`"
          class="assignment-row"
          :class="{ 'is-assigned': item.assigned }"
        >
          <div class="assignment-order">
            <div class="order-badge" :class="getPickClass(item.pickOrder)">
              {{ item.pickOrder }}
            </div>
          </div>
          <div class="assignment-player">
            <span class="player-tag" :class="getPickLabelClass(item.pickOrder || 0)">
              {{ draftPool[(item.pickOrder || 1) - 1]?.title }}
            </span>
            <div class="player-info">
              <span class="player-name">{{ draftPool[(item.pickOrder || 1) - 1]?.gameId }}</span>
              <span class="player-stats">
                能力 {{ draftPool[(item.pickOrder || 1) - 1]?.ability }} ·
                潜力 {{ draftPool[(item.pickOrder || 1) - 1]?.potential }}
              </span>
            </div>
          </div>
          <div class="assignment-arrow">
            <el-icon :size="20"><Right /></el-icon>
          </div>
          <div class="assignment-team">
            <div class="team-avatar large" :class="selectedRegion">
              {{ item.teamName.substring(0, 2) }}
            </div>
            <span class="team-name">{{ item.teamName }}</span>
          </div>
          <div class="assignment-status">
            <span v-if="item.assigned" class="status-done">
              <el-icon><Check /></el-icon>
              已分配
            </span>
            <span v-else class="status-pending">待分配</span>
          </div>
        </div>
      </div>

      <div class="section-footer">
        <el-button @click="currentStep = 3">
          <el-icon><ArrowLeft /></el-icon>
          返回上一步
        </el-button>
        <el-button type="primary" size="large" @click="assignPlayers" :disabled="isAssigned">
          {{ isAssigned ? '已完成分配' : '确认分配选手' }}
          <el-icon class="ml-2"><Check /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 步骤5: 完成 -->
    <div v-if="currentStep >= 5" class="content-section">
      <div class="completion-banner">
        <div class="completion-icon">
          <el-icon :size="48"><SuccessFilled /></el-icon>
        </div>
        <h2 class="completion-title">选秀大会圆满结束!</h2>
        <p class="completion-desc">所有新秀选手已成功加入队伍</p>
        <el-button type="primary" size="large" @click="completeDraft">
          完成并返回
        </el-button>
      </div>

      <div class="result-section">
        <h3 class="result-title">选秀结果摘要</h3>
        <div class="result-grid">
          <div
            v-for="item in assignmentList"
            :key="`result-${item.pickOrder}-${item.teamId}`"
            class="result-card"
            :class="getPickClass(item.pickOrder)"
          >
            <div class="result-order">{{ item.pickOrder }}</div>
            <div class="result-content">
              <div class="result-player">{{ draftPool[(item.pickOrder || 1) - 1]?.gameId }}</div>
              <div class="result-team">
                <el-icon><ArrowRight /></el-icon>
                {{ item.teamName }}
              </div>
            </div>
            <div class="result-ability">
              {{ draftPool[(item.pickOrder || 1) - 1]?.ability }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft,
  ArrowRight,
  User,
  Check,
  Tickets,
  Connection,
  Right,
  QuestionFilled,
  WarningFilled,
  SuccessFilled,
  Aim,
  MagicStick,
  Sell,
  Money,
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { draftApi, teamApi, queryApi } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { storeToRefs } from 'pinia'
import { createLogger } from '@/utils/logger'

const logger = createLogger('DraftRegion')

const route = useRoute()
const router = useRouter()
const gameStore = useGameStore()
const { currentSeason: gameSeason, currentPhase } = storeToRefs(gameStore)

// 是否处于选秀阶段
const isDraftPhase = computed(() => currentPhase.value === 'Draft')

// 状态
const selectedRegion = ref((route.params.region as string)?.toLowerCase() || 'lpl')
const currentSeason = computed(() => `S${gameSeason.value}`)
const currentStep = ref(0)
const isLoading = ref(false)
const currentRegionId = ref<number>(1)

// 步骤定义
const steps = [
  { title: '抽取选秀名单', desc: '从选秀池抽取14人' },
  { title: '查看选秀名单', desc: '浏览本届新秀' },
  { title: '选秀权抽签', desc: '决定选秀顺位' },
  { title: '选秀权拍卖', desc: '交易选秀权' },
  { title: '分配选手', desc: '选手加入队伍' },
  { title: '完成选秀', desc: '选秀结束' },
]

// 是否已抽取选秀名单
const hasDraftRoster = ref(false)

// 赛区列表
const regionList = [
  { code: 'lpl', name: '中国' },
  { code: 'lck', name: '韩国' },
  { code: 'lec', name: '欧洲' },
  { code: 'lcs', name: '北美' },
]

// 赛区名称映射
const regionNames: Record<string, string> = {
  lpl: 'LPL 中国赛区',
  lck: 'LCK 韩国赛区',
  lec: 'LEC 欧洲赛区',
  lcs: 'LCS 北美赛区',
}

const regionName = computed(() => regionNames[selectedRegion.value] || '')

// 选秀池数据
const draftPool = ref<any[]>([])

// 抽签结果
const lotteryResults = ref<any[]>([])

// 后端抽签结果缓存（用于单个揭示）
const backendLotteryCache = ref<any[]>([])

// 分配列表（position-centric，拍卖后的实际签位归属）
const assignmentList = ref<any[]>([])
// 获取赛区ID
const getRegionId = async (regionCode: string): Promise<number> => {
  try {
    const regions = await queryApi.getAllRegions()
    const region = regions.find(r => r.code.toLowerCase() === regionCode.toLowerCase())
    return region?.id ?? 1
  } catch (e) {
    logger.error('Failed to get region id:', e)
    return 1
  }
}

// 加载队伍数据
const loadTeams = async () => {
  isLoading.value = true
  try {
    const regionId = await getRegionId(selectedRegion.value)
    const teams = await teamApi.getTeamsByRegion(regionId)

    lotteryResults.value = teams.map(team => ({
      teamId: team.id,
      teamName: team.name,
      pickOrder: null as number | null,
      assigned: false,
    }))
  } catch (e) {
    logger.error('Failed to load teams:', e)
    ElMessage.error('加载队伍失败')
  } finally {
    isLoading.value = false
  }
}

// 初始化
onMounted(async () => {
  await loadTeams()
  await restoreDraftState()
})

// 恢复已有的选秀状态
const restoreDraftState = async () => {
  try {
    const regionId = await getRegionId(selectedRegion.value)
    currentRegionId.value = regionId

    const status = await draftApi.getDraftRegionStatus(regionId)

    if (status.status === 'not_started') {
      currentStep.value = 0
      return
    }

    // 有选秀名单 → 构建 draftPool（包含已选和未选的全部球员）
    draftPool.value = status.draft_players.map((p, idx) => ({
      rank: idx + 1,
      title: idx === 0 ? '状元' : idx === 1 ? '榜眼' : idx === 2 ? '探花' : `第${idx + 1}顺位`,
      gameId: p.game_id,
      ability: p.ability,
      potential: p.potential,
      tag: p.tag,
      position: p.position,
    }))
    hasDraftRoster.value = true

    if (status.status === 'roster_drawn') {
      currentStep.value = 1
      return
    }

    // lottery_done 或 completed → 恢复抽签结果和分配列表
    backendLotteryCache.value = status.draft_orders

    status.draft_orders.forEach((order) => {
      const matchId = order.original_team_id ?? order.team_id
      const team = lotteryResults.value.find(t => t.teamId === matchId)
      if (team) {
        team.pickOrder = order.draft_position
      }
    })

    const isCompleted = status.status === 'completed'
    assignmentList.value = [...status.draft_orders]
      .sort((a, b) => a.draft_position - b.draft_position)
      .map((order) => ({
        pickOrder: order.draft_position,
        teamId: order.team_id,
        teamName: order.team_name,
        assigned: isCompleted,
      }))

    if (isCompleted) {
      currentStep.value = 5
    } else {
      currentStep.value = 2
    }
  } catch (e) {
    logger.error('恢复选秀状态失败:', e)
  }
}

// 监听路由参数变化
watch(
  () => route.params.region,
  async (newRegion) => {
    if (newRegion && typeof newRegion === 'string') {
      selectedRegion.value = newRegion.toLowerCase()
      // 重置选秀状态并重新加载数据
      await resetDraftState()
    }
  }
)

// 重置选秀状态
const resetDraftState = async () => {
  currentStep.value = 0
  hasDraftRoster.value = false
  draftPool.value = []
  backendLotteryCache.value = []
  assignmentList.value = []
  // 重新加载队伍数据
  await loadTeams()
  await restoreDraftState()
}

// 计算属性
const hasUndrawnTeams = computed(() => {
  return lotteryResults.value.some(r => r.pickOrder === null)
})

const isAssigned = computed(() => {
  return assignmentList.value.length > 0 && assignmentList.value.every(r => r.assigned)
})


// 方法
const handleRegionChange = (region: string) => {
  router.push(`/draft/${region}`)
}

const getStatusClass = () => {
  if (currentStep.value >= 4) return 'completed'
  if (currentStep.value > 0) return 'in-progress'
  return 'pending'
}

const getStatusLabel = () => {
  if (currentStep.value >= 4) return '已完成'
  if (currentStep.value > 0) return '进行中'
  return '待开始'
}

const getProspectClass = (rank: number) => {
  if (rank === 1) return 'elite'
  if (rank <= 3) return 'excellent'
  if (rank <= 6) return 'good'
  return 'normal'
}

const getPickClass = (order: number | null) => {
  if (!order) return ''
  if (order === 1) return 'gold'
  if (order === 2) return 'silver'
  if (order === 3) return 'bronze'
  return ''
}

const getPickLabelClass = (order: number) => {
  if (order === 1) return 'elite'
  if (order <= 3) return 'excellent'
  return 'normal'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 70) return '#22c55e'
  if (ability >= 60) return '#f59e0b'
  return '#ef4444'
}

const startLottery = () => {
  currentStep.value = 2
}

// 进入拍卖步骤
const proceedToAuction = () => {
  currentStep.value = 3
}

// 前往拍卖大厅页面
const goToAuction = () => {
  router.push(`/draft/${selectedRegion.value}/auction`)
}

// 跳过拍卖，直接进入分配
const skipAuction = async () => {
  await buildAssignmentList()
  currentStep.value = 4
}

// 从 draft_orders 构建分配列表（使用拍卖后的实际拥有者）
const buildAssignmentList = async () => {
  try {
    const draftOrder = await draftApi.getDraftOrder(currentRegionId.value)
    if (draftOrder && draftOrder.length > 0) {
      backendLotteryCache.value = draftOrder
      assignmentList.value = draftOrder
        .sort((a, b) => a.draft_position - b.draft_position)
        .map((order) => ({
          pickOrder: order.draft_position,
          teamId: order.team_id,
          teamName: order.team_name,
          assigned: false,
        }))
    }
  } catch (e) {
    logger.error('构建分配列表失败:', e)
  }
}

// 抽取选秀名单
const drawDraftRoster = async () => {
  isLoading.value = true
  try {
    const regionId = await getRegionId(selectedRegion.value)
    currentRegionId.value = regionId

    // 调用后端 generateDraftPool，从 draft_pool 表随机抽取14人写入 draft_players
    const players = await draftApi.generateDraftPool(regionId, 14)

    if (!players || players.length === 0) {
      ElMessage.warning('选秀池为空，请先在选手池管理中导入或生成新秀数据')
      return
    }

    draftPool.value = players.map((p, index) => ({
      id: p.id,
      rank: index + 1,
      title: index === 0 ? '状元' : index === 1 ? '榜眼' : index === 2 ? '探花' : `第${index + 1}顺位`,
      gameId: p.game_id,
      ability: p.ability,
      potential: p.potential,
      tag: p.tag,
      position: p.position,
    }))

    hasDraftRoster.value = true
    currentStep.value = 1
    ElMessage.success('选秀名单抽取完成！')
  } catch (e: any) {
    logger.error('Failed to draw draft roster:', e)
    ElMessage.error(e?.message || '抽取选秀名单失败')
  } finally {
    isLoading.value = false
  }
}

// 辅助函数：根据 draft order 匹配 lotteryResults 中的原始队伍
// 拍卖后 team_id 变成买家，需要用 original_team_id 匹配
const findTeamByOrder = (order: { team_id: number; original_team_id?: number | null }) => {
  const matchId = order.original_team_id ?? order.team_id
  return lotteryResults.value.find(t => t.teamId === matchId)
}

// 执行后端抽签（一次性生成所有结果，缓存起来）
const runBackendLottery = async () => {
  isLoading.value = true
  try {
    const draftOrder = await draftApi.runDraftLottery(currentRegionId.value)
    backendLotteryCache.value = draftOrder
    // 一键抽签：直接全部显示
    draftOrder.forEach((order) => {
      const team = findTeamByOrder(order)
      if (team) {
        team.pickOrder = order.draft_position
      }
    })

    ElMessage.success('抽签完成!')
  } catch (e) {
    logger.error('Failed to run draft lottery:', e)
    ElMessage.error('抽签失败')
  } finally {
    isLoading.value = false
  }
}

const drawSinglePick = async () => {
  // 如果还没有后端抽签结果，先调后端生成
  if (backendLotteryCache.value.length === 0) {
    isLoading.value = true
    try {
      const draftOrder = await draftApi.runDraftLottery(currentRegionId.value)
      backendLotteryCache.value = draftOrder
    } catch (e) {
      logger.error('Failed to run draft lottery:', e)
      ElMessage.error('抽签失败')
      isLoading.value = false
      return
    } finally {
      isLoading.value = false
    }
  }

  // 找到下一个未揭示的队伍（按 draft_position 从大到小逐个揭示，即末位先揭示）
  const revealedTeamIds = new Set(
    lotteryResults.value.filter(t => t.pickOrder !== null).map(t => t.teamId)
  )
  // 按 draft_position 降序排列，逐个揭示
  const unrevealed = backendLotteryCache.value
    .filter(o => {
      const matchId = o.original_team_id ?? o.team_id
      return !revealedTeamIds.has(matchId)
    })
    .sort((a, b) => b.draft_position - a.draft_position)

  if (unrevealed.length > 0) {
    const next = unrevealed[0]
    const team = findTeamByOrder(next)
    if (team) {
      team.pickOrder = next.draft_position
    }
  }
}

const drawAllPicks = async () => {
  if (backendLotteryCache.value.length > 0) {
    // 已有缓存，直接全部揭示
    backendLotteryCache.value.forEach((order) => {
      const team = findTeamByOrder(order)
      if (team) {
        team.pickOrder = order.draft_position
      }
    })
    ElMessage.success('抽签完成!')
  } else {
    await runBackendLottery()
  }
}

// 分配选手到队伍 - 调用后端API
const assignPlayers = async () => {
  isLoading.value = true
  try {
    // 使用后端AI自动选秀完成分配
    const draftPicks = await draftApi.aiAutoDraft(currentRegionId.value)

    // 更新分配列表状态
    draftPicks.forEach((pick) => {
      const item = assignmentList.value.find(a => a.teamId === pick.team_id)
      if (item) {
        item.assigned = true
      }
    })

    currentStep.value = 5
    ElMessage.success('选手分配完成!')
  } catch (e) {
    logger.error('Failed to assign players:', e)
    ElMessage.error('分配选手失败')
  } finally {
    isLoading.value = false
  }
}

const completeDraft = () => {
  router.push('/draft')
}
</script>

<style scoped lang="scss">
.draft-region-view {
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

  &:hover {
    color: #3b82f6;
  }
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.region-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: 14px;
  font-size: 14px;
  font-weight: 700;
  color: white;

  &.lpl { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); }
  &.lck { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
  &.lec { background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); }
  &.lcs { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); }
}

.header-info {
  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 4px 0;
  }

  .page-desc {
    font-size: 14px;
    color: #6b7280;
    margin: 0;
  }
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 24px;
  font-size: 14px;
  font-weight: 600;

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  &.completed {
    background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
    color: #166534;
    .status-dot { background: #22c55e; }
  }

  &.in-progress {
    background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
    color: #92400e;
    .status-dot {
      background: #f59e0b;
      animation: pulse 2s infinite;
    }
  }

  &.pending {
    background: #f3f4f6;
    color: #6b7280;
    .status-dot { background: #9ca3af; }
  }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 赛区标签 */
.region-tabs {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  padding: 16px;
  background: white;
  border-radius: 14px;
  border: 1px solid #e5e7eb;
}

.region-tab {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  padding: 12px 16px;
  background: #f9fafb;
  border: 2px solid transparent;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: #f3f4f6;
  }

  &.active {
    background: white;
    border-color: #3b82f6;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.15);
  }

  .tab-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    font-size: 11px;
    font-weight: 700;
    color: white;

    &.lpl { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); }
    &.lck { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
    &.lec { background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); }
    &.lcs { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); }
  }

  .tab-name {
    font-size: 14px;
    font-weight: 600;
    color: #374151;
  }
}

/* 进度步骤 */
.progress-steps {
  display: flex;
  align-items: flex-start;
  padding: 24px;
  background: white;
  border-radius: 14px;
  border: 1px solid #e5e7eb;
  margin-bottom: 24px;
}

.step-item {
  display: flex;
  align-items: center;
  flex: 1;
  position: relative;

  &:last-child {
    flex: none;
  }

  .step-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #f3f4f6;
    color: #9ca3af;
    font-size: 14px;
    font-weight: 600;
    flex-shrink: 0;
    margin-right: 12px;
    transition: all 0.3s;
  }

  .step-content {
    flex-shrink: 0;
    margin-right: 16px;
  }

  .step-title {
    font-size: 14px;
    font-weight: 600;
    color: #6b7280;
    margin-bottom: 2px;
  }

  .step-desc {
    font-size: 12px;
    color: #9ca3af;
  }

  .step-line {
    flex: 1;
    height: 2px;
    background: #e5e7eb;
    margin: 0 8px;
  }

  &.is-active {
    .step-indicator {
      background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
      color: white;
      box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    }

    .step-title {
      color: #1f2937;
    }
  }

  &.is-completed {
    .step-indicator {
      background: #22c55e;
      color: white;
    }

    .step-title {
      color: #1f2937;
    }

    .step-line {
      background: #22c55e;
    }
  }
}

/* 内容区块 */
.content-section {
  background: white;
  border-radius: 16px;
  border: 1px solid #e5e7eb;
  padding: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  .header-title {
    display: flex;
    align-items: center;
    gap: 10px;
    color: #3b82f6;

    h2 {
      font-size: 18px;
      font-weight: 600;
      color: #1f2937;
      margin: 0;
    }
  }

  .header-meta {
    .meta-count {
      font-size: 14px;
      color: #6b7280;
      padding: 6px 14px;
      background: #f3f4f6;
      border-radius: 20px;
    }
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }
}

/* 新秀卡片网格 */
.prospects-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 14px;
  margin-bottom: 24px;
}

.prospect-card {
  padding: 16px;
  background: #f9fafb;
  border: 2px solid transparent;
  border-radius: 12px;
  transition: all 0.2s;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }

  &.elite {
    background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
    border-color: #fbbf24;
  }

  &.excellent {
    background: linear-gradient(135deg, #f5f3ff 0%, #ede9fe 100%);
    border-color: #a78bfa;
  }

  .prospect-rank {
    text-align: center;
    margin-bottom: 12px;

    .rank-number {
      display: block;
      font-size: 28px;
      font-weight: 700;
      color: #1f2937;
      line-height: 1;
    }

    .rank-label {
      font-size: 12px;
      color: #6b7280;
    }
  }

  .prospect-info {
    text-align: center;

    .prospect-name {
      font-size: 15px;
      font-weight: 600;
      color: #1f2937;
      margin-bottom: 10px;
    }

    .prospect-stats {
      display: flex;
      justify-content: center;
      gap: 16px;
    }

    .stat-item {
      text-align: center;
    }

    .stat-value {
      display: block;
      font-size: 18px;
      font-weight: 700;
      line-height: 1;

      &.potential {
        color: #8b5cf6;
      }
    }

    .stat-label {
      font-size: 11px;
      color: #9ca3af;
    }
  }

  .prospect-tag {
    display: inline-block;
    width: 100%;
    text-align: center;
    padding: 6px 0;
    margin-top: 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;

    &.genius {
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      color: #92400e;
    }

    &.normal {
      background: #f3f4f6;
      color: #6b7280;
    }
  }
}

/* 抽签网格 */
.lottery-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.lottery-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: #f9fafb;
  border: 2px solid #e5e7eb;
  border-radius: 12px;
  transition: all 0.3s;

  &.is-drawn {
    background: white;
    border-color: #22c55e;
  }

  .lottery-rank {
    text-align: center;
    padding-right: 16px;
    border-right: 1px solid #e5e7eb;

    .rank-label {
      display: block;
      font-size: 11px;
      color: #9ca3af;
    }

    .rank-value {
      font-size: 13px;
      font-weight: 600;
      color: #374151;
    }
  }

  .lottery-team {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 120px;

    .team-avatar {
      width: 36px;
      height: 36px;
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 11px;
      font-weight: 700;
      color: white;

      &.lpl { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); }
      &.lck { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
      &.lec { background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); }
      &.lcs { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); }
    }

    .team-name {
      font-size: 14px;
      font-weight: 600;
      color: #1f2937;
    }
  }

  .lottery-result {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;

    .pick-badge {
      padding: 6px 14px;
      border-radius: 6px;
      font-size: 13px;
      font-weight: 600;
      background: #dbeafe;
      color: #1e40af;

      &.gold {
        background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
        color: #92400e;
      }

      &.silver {
        background: linear-gradient(135deg, #f3f4f6 0%, #e5e7eb 100%);
        color: #374151;
      }

      &.bronze {
        background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
        color: #9a3412;
      }
    }

    .pick-player {
      display: flex;
      align-items: center;
      gap: 8px;

      .player-tag {
        padding: 4px 10px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 600;

        &.elite {
          background: #fef3c7;
          color: #92400e;
        }

        &.excellent {
          background: #ede9fe;
          color: #6d28d9;
        }

        &.normal {
          background: #f3f4f6;
          color: #6b7280;
        }
      }

      .player-name {
        font-size: 14px;
        font-weight: 500;
        color: #374151;
      }
    }

    .pick-waiting {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 4px;

      span {
        font-size: 12px;
        color: #9ca3af;
      }
    }
  }
}

/* 通知卡片 */
.notice-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 20px;
  background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
  border: 1px solid #fde68a;
  border-radius: 10px;
  margin-bottom: 24px;

  .notice-text {
    display: flex;
    flex-direction: column;
    gap: 2px;

    strong {
      font-size: 14px;
      color: #92400e;
    }

    span {
      font-size: 13px;
      color: #b45309;
    }
  }
}

/* 分配列表 */
.assignment-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.assignment-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: #f9fafb;
  border: 2px solid #e5e7eb;
  border-radius: 12px;
  transition: all 0.2s;

  &.is-assigned {
    background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
    border-color: #86efac;
  }

  .assignment-order {
    .order-badge {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 36px;
      height: 36px;
      border-radius: 50%;
      font-size: 14px;
      font-weight: 700;
      background: #e5e7eb;
      color: #374151;

      &.gold {
        background: linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%);
        color: white;
      }

      &.silver {
        background: linear-gradient(135deg, #9ca3af 0%, #6b7280 100%);
        color: white;
      }

      &.bronze {
        background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
        color: white;
      }
    }
  }

  .assignment-player {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;

    .player-tag {
      padding: 4px 10px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: 600;

      &.elite {
        background: #fef3c7;
        color: #92400e;
      }

      &.excellent {
        background: #ede9fe;
        color: #6d28d9;
      }

      &.normal {
        background: #f3f4f6;
        color: #6b7280;
      }
    }

    .player-info {
      display: flex;
      flex-direction: column;

      .player-name {
        font-size: 15px;
        font-weight: 600;
        color: #1f2937;
      }

      .player-stats {
        font-size: 12px;
        color: #6b7280;
      }
    }
  }

  .assignment-arrow {
    color: #3b82f6;
  }

  .assignment-team {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 150px;

    .team-avatar {
      width: 40px;
      height: 40px;
      border-radius: 10px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 12px;
      font-weight: 700;
      color: white;

      &.lpl { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); }
      &.lck { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
      &.lec { background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); }
      &.lcs { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); }
    }

    .team-name {
      font-size: 15px;
      font-weight: 600;
      color: #1f2937;
    }
  }

  .assignment-status {
    .status-done {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 6px 12px;
      background: #dcfce7;
      color: #166534;
      border-radius: 6px;
      font-size: 13px;
      font-weight: 600;
    }

    .status-pending {
      padding: 6px 12px;
      background: #f3f4f6;
      color: #6b7280;
      border-radius: 6px;
      font-size: 13px;
      font-weight: 500;
    }
  }
}

/* 完成横幅 */
.completion-banner {
  text-align: center;
  padding: 40px 24px;
  margin-bottom: 32px;
  background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
  border-radius: 12px;
  border: 1px solid #bbf7d0;

  .completion-icon {
    color: #22c55e;
    margin-bottom: 16px;
  }

  .completion-title {
    font-size: 22px;
    font-weight: 700;
    color: #166534;
    margin: 0 0 8px 0;
  }

  .completion-desc {
    font-size: 14px;
    color: #4ade80;
    margin: 0 0 24px 0;
  }
}

/* 结果区块 */
.result-section {
  padding-top: 24px;
  border-top: 1px solid #e5e7eb;

  .result-title {
    font-size: 16px;
    font-weight: 600;
    color: #1f2937;
    margin: 0 0 16px 0;
  }
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.result-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: #f9fafb;
  border-radius: 10px;
  border: 1px solid #f3f4f6;

  &.gold {
    background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
    border-color: #fde68a;
  }

  &.silver {
    background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
    border-color: #e5e7eb;
  }

  &.bronze {
    background: linear-gradient(135deg, #fff7ed 0%, #fed7aa 100%);
    border-color: #fdba74;
  }

  .result-order {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: #e5e7eb;
    color: #374151;
    font-size: 13px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .result-content {
    flex: 1;
    min-width: 0;

    .result-player {
      font-size: 14px;
      font-weight: 600;
      color: #1f2937;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .result-team {
      display: flex;
      align-items: center;
      gap: 4px;
      font-size: 12px;
      color: #6b7280;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }

  .result-ability {
    font-size: 16px;
    font-weight: 700;
    color: #22c55e;
    flex-shrink: 0;
  }
}

/* 底部 */
.section-footer {
  display: flex;
  justify-content: space-between;
  padding-top: 24px;
  border-top: 1px solid #e5e7eb;
}

.ml-2 {
  margin-left: 8px;
}

.mr-2 {
  margin-right: 8px;
}

/* 抽取选秀名单区块 */
.draw-roster-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 24px;
  text-align: center;

  .draw-info {
    margin-bottom: 32px;

    h3 {
      font-size: 20px;
      font-weight: 600;
      color: #1f2937;
      margin: 20px 0 8px 0;
    }

    p {
      font-size: 14px;
      color: #6b7280;
      margin: 0;
      max-width: 400px;
    }
  }

  .phase-warning {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
    border: 1px solid #fde68a;
    border-radius: 8px;
    margin-bottom: 20px;
    font-size: 14px;
    color: #92400e;
  }
}

/* 拍卖区块 */
.auction-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 24px;
  text-align: center;

  .auction-info {
    margin-bottom: 32px;

    h3 {
      font-size: 20px;
      font-weight: 600;
      color: #1f2937;
      margin: 20px 0 8px 0;
    }

    p {
      font-size: 14px;
      color: #6b7280;
      margin: 0 0 8px 0;
      max-width: 500px;
    }

    .sub-info {
      font-size: 13px;
      color: #9ca3af;
    }
  }

  .auction-actions {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    justify-content: center;
  }
}
</style>
