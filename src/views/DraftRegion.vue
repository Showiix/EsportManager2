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

    <!-- 步骤1: 选秀池 -->
    <div v-if="currentStep === 0" class="content-section">
      <div class="section-header">
        <div class="header-title">
          <el-icon :size="22"><User /></el-icon>
          <h2>选秀池名单</h2>
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
        <el-button type="primary" size="large" @click="startLottery">
          开始选秀权抽签
          <el-icon class="ml-2"><ArrowRight /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 步骤2: 抽签 -->
    <div v-if="currentStep === 1" class="content-section">
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
            <template v-if="team.pickOrder">
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
        <el-button @click="currentStep = 0">
          <el-icon><ArrowLeft /></el-icon>
          返回上一步
        </el-button>
        <el-button type="primary" size="large" @click="proceedToAssignment" :disabled="hasUndrawnTeams">
          进入选手分配
          <el-icon class="ml-2"><ArrowRight /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 步骤3: 分配 -->
    <div v-if="currentStep === 2" class="content-section">
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
          v-for="team in sortedLotteryResults"
          :key="team.teamId"
          class="assignment-row"
          :class="{ 'is-assigned': team.assigned }"
        >
          <div class="assignment-order">
            <div class="order-badge" :class="getPickClass(team.pickOrder)">
              {{ team.pickOrder }}
            </div>
          </div>
          <div class="assignment-player">
            <span class="player-tag" :class="getPickLabelClass(team.pickOrder || 0)">
              {{ draftPool[(team.pickOrder || 1) - 1]?.title }}
            </span>
            <div class="player-info">
              <span class="player-name">{{ draftPool[(team.pickOrder || 1) - 1]?.gameId }}</span>
              <span class="player-stats">
                能力 {{ draftPool[(team.pickOrder || 1) - 1]?.ability }} ·
                潜力 {{ draftPool[(team.pickOrder || 1) - 1]?.potential }}
              </span>
            </div>
          </div>
          <div class="assignment-arrow">
            <el-icon :size="20"><Right /></el-icon>
          </div>
          <div class="assignment-team">
            <div class="team-avatar large" :class="selectedRegion">
              {{ team.teamName.substring(0, 2) }}
            </div>
            <span class="team-name">{{ team.teamName }}</span>
          </div>
          <div class="assignment-status">
            <span v-if="team.assigned" class="status-done">
              <el-icon><Check /></el-icon>
              已分配
            </span>
            <span v-else class="status-pending">待分配</span>
          </div>
        </div>
      </div>

      <div class="section-footer">
        <el-button @click="currentStep = 1">
          <el-icon><ArrowLeft /></el-icon>
          返回上一步
        </el-button>
        <el-button type="primary" size="large" @click="assignPlayers" :disabled="isAssigned">
          {{ isAssigned ? '已完成分配' : '确认分配选手' }}
          <el-icon class="ml-2"><Check /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 步骤4: 完成 -->
    <div v-if="currentStep >= 3" class="content-section">
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
            v-for="team in sortedLotteryResults"
            :key="team.teamId"
            class="result-card"
            :class="getPickClass(team.pickOrder)"
          >
            <div class="result-order">{{ team.pickOrder }}</div>
            <div class="result-content">
              <div class="result-player">{{ draftPool[(team.pickOrder || 1) - 1]?.gameId }}</div>
              <div class="result-team">
                <el-icon><ArrowRight /></el-icon>
                {{ team.teamName }}
              </div>
            </div>
            <div class="result-ability">
              {{ draftPool[(team.pickOrder || 1) - 1]?.ability }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
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
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

const route = useRoute()
const router = useRouter()

// 状态
const selectedRegion = ref((route.params.region as string)?.toLowerCase() || 'lpl')
const currentSeason = ref('S2')
const currentStep = ref(0)

// 步骤定义
const steps = [
  { title: '查看选秀池', desc: '浏览新秀名单' },
  { title: '选秀权抽签', desc: '决定选秀顺位' },
  { title: '分配选手', desc: '选手加入队伍' },
  { title: '完成选秀', desc: '选秀结束' },
]

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

// 选秀池数据 - 14名新秀
const draftPool = ref([
  { rank: 1, title: '状元', gameId: 'Rookie1', ability: 75, potential: 95, tag: 'GENIUS' },
  { rank: 2, title: '榜眼', gameId: 'Rookie2', ability: 72, potential: 92, tag: 'GENIUS' },
  { rank: 3, title: '探花', gameId: 'Rookie3', ability: 70, potential: 90, tag: 'GENIUS' },
  { rank: 4, title: '第4顺位', gameId: 'Rookie4', ability: 68, potential: 85, tag: 'NORMAL' },
  { rank: 5, title: '第5顺位', gameId: 'Rookie5', ability: 65, potential: 82, tag: 'NORMAL' },
  { rank: 6, title: '第6顺位', gameId: 'Rookie6', ability: 63, potential: 80, tag: 'NORMAL' },
  { rank: 7, title: '第7顺位', gameId: 'Rookie7', ability: 60, potential: 78, tag: 'NORMAL' },
  { rank: 8, title: '第8顺位', gameId: 'Rookie8', ability: 58, potential: 76, tag: 'NORMAL' },
  { rank: 9, title: '第9顺位', gameId: 'Rookie9', ability: 55, potential: 74, tag: 'NORMAL' },
  { rank: 10, title: '第10顺位', gameId: 'Rookie10', ability: 52, potential: 72, tag: 'NORMAL' },
  { rank: 11, title: '第11顺位', gameId: 'Rookie11', ability: 50, potential: 70, tag: 'NORMAL' },
  { rank: 12, title: '第12顺位', gameId: 'Rookie12', ability: 48, potential: 68, tag: 'NORMAL' },
  { rank: 13, title: '第13顺位', gameId: 'Rookie13', ability: 46, potential: 66, tag: 'NORMAL' },
  { rank: 14, title: '第14顺位', gameId: 'Rookie14', ability: 44, potential: 64, tag: 'NORMAL' },
])

// 抽签结果 - 14支队伍
const lotteryResults = ref([
  { teamId: 1, teamName: 'Team A', pickOrder: null as number | null, assigned: false },
  { teamId: 2, teamName: 'Team B', pickOrder: null as number | null, assigned: false },
  { teamId: 3, teamName: 'Team C', pickOrder: null as number | null, assigned: false },
  { teamId: 4, teamName: 'Team D', pickOrder: null as number | null, assigned: false },
  { teamId: 5, teamName: 'Team E', pickOrder: null as number | null, assigned: false },
  { teamId: 6, teamName: 'Team F', pickOrder: null as number | null, assigned: false },
  { teamId: 7, teamName: 'Team G', pickOrder: null as number | null, assigned: false },
  { teamId: 8, teamName: 'Team H', pickOrder: null as number | null, assigned: false },
  { teamId: 9, teamName: 'Team I', pickOrder: null as number | null, assigned: false },
  { teamId: 10, teamName: 'Team J', pickOrder: null as number | null, assigned: false },
  { teamId: 11, teamName: 'Team K', pickOrder: null as number | null, assigned: false },
  { teamId: 12, teamName: 'Team L', pickOrder: null as number | null, assigned: false },
  { teamId: 13, teamName: 'Team M', pickOrder: null as number | null, assigned: false },
  { teamId: 14, teamName: 'Team N', pickOrder: null as number | null, assigned: false },
])

// 监听路由参数变化
watch(
  () => route.params.region,
  (newRegion) => {
    if (newRegion && typeof newRegion === 'string') {
      selectedRegion.value = newRegion.toLowerCase()
      // 重置选秀状态
      resetDraftState()
    }
  }
)

// 重置选秀状态
const resetDraftState = () => {
  currentStep.value = 0
  // 重置抽签结果
  lotteryResults.value.forEach(team => {
    team.pickOrder = null
    team.assigned = false
  })
}

// 计算属性
const hasUndrawnTeams = computed(() => {
  return lotteryResults.value.some(r => r.pickOrder === null)
})

const isAssigned = computed(() => {
  return lotteryResults.value.every(r => r.assigned)
})

const sortedLotteryResults = computed(() => {
  return [...lotteryResults.value]
    .filter(r => r.pickOrder !== null)
    .sort((a, b) => (a.pickOrder || 0) - (b.pickOrder || 0))
})

// 方法
const handleRegionChange = (region: string) => {
  router.push(`/draft/${region}`)
}

const getStatusClass = () => {
  if (currentStep.value >= 3) return 'completed'
  if (currentStep.value > 0) return 'in-progress'
  return 'pending'
}

const getStatusLabel = () => {
  if (currentStep.value >= 3) return '已完成'
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
  currentStep.value = 1
}

const drawSinglePick = () => {
  const undrawnTeam = lotteryResults.value.find(r => r.pickOrder === null)
  if (!undrawnTeam) return

  const usedPicks = lotteryResults.value
    .filter(r => r.pickOrder !== null)
    .map(r => r.pickOrder)

  const availablePicks = Array.from({ length: 14 }, (_, i) => i + 1)
    .filter(p => !usedPicks.includes(p))

  const randomIndex = Math.floor(Math.random() * availablePicks.length)
  undrawnTeam.pickOrder = availablePicks[randomIndex]

  ElMessage.success(`${undrawnTeam.teamName} 抽中第 ${undrawnTeam.pickOrder} 顺位`)
}

const drawAllPicks = () => {
  const availablePicks = Array.from({ length: 14 }, (_, i) => i + 1)
  const shuffled = availablePicks.sort(() => Math.random() - 0.5)

  let pickIndex = 0
  lotteryResults.value.forEach(team => {
    if (team.pickOrder === null) {
      team.pickOrder = shuffled[pickIndex]
      pickIndex++
    }
  })

  ElMessage.success('一键抽签完成!')
}

const proceedToAssignment = () => {
  currentStep.value = 2
}

const assignPlayers = () => {
  lotteryResults.value.forEach(team => {
    team.assigned = true
  })
  currentStep.value = 3
  ElMessage.success('选手分配完成!')
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
  padding: 48px 24px;
  margin-bottom: 32px;

  .completion-icon {
    color: #22c55e;
    margin-bottom: 20px;
  }

  .completion-title {
    font-size: 24px;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 8px 0;
  }

  .completion-desc {
    font-size: 15px;
    color: #6b7280;
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
  grid-template-columns: repeat(7, 1fr);
  gap: 12px;
}

.result-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: #f9fafb;
  border-radius: 10px;

  &.gold {
    background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
  }

  &.silver {
    background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
  }

  &.bronze {
    background: linear-gradient(135deg, #fff7ed 0%, #fed7aa 100%);
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
    }
  }

  .result-ability {
    font-size: 16px;
    font-weight: 700;
    color: #22c55e;
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
</style>
