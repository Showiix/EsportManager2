<template>
  <div class="madrid-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <el-button text @click="goBack">
            <el-icon><ArrowLeft /></el-icon>
            返回赛事列表
          </el-button>
          <h1 class="page-title">
            <el-icon><Trophy /></el-icon>
            马德里大师赛 (Madrid Masters)
          </h1>
          <p class="page-description">
            32支队伍（各赛区春季赛常规赛前8名），8个小组BO3单循环，东西半区BO5淘汰赛
          </p>
        </div>
      </div>
      <div class="header-actions">
        <el-button
          v-if="madridBracket.status === 'group_stage' && !isGroupStageComplete"
          type="primary"
          @click="batchSimulateGroupStage"
          :loading="simulatingGroupStage"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingGroupStage ? `模拟中 (${simulationProgress}%)` : '模拟小组赛' }}
        </el-button>
        <el-button
          v-if="madridBracket.status === 'knockout_stage'"
          type="warning"
          @click="batchSimulateKnockout"
          :loading="simulatingKnockout"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingKnockout ? `模拟中 (${simulationProgress}%)` : '模拟淘汰赛' }}
        </el-button>
      </div>
    </div>

    <!-- 阶段未到提示 -->
    <el-alert
      v-if="phaseNotReached"
      title="赛事尚未开始"
      type="warning"
      :closable="false"
      show-icon
      class="phase-warning-alert"
    >
      <template #default>
        <div class="phase-warning-content">
          <p>当前赛季阶段：<strong>{{ currentPhaseDisplay }}</strong></p>
          <p>马德里大师赛需要在 <strong>MSI季中赛</strong> 结束后才会开始。</p>
          <p>请先完成之前的赛事阶段，然后在时间控制面板推进到马德里大师赛阶段。</p>
        </div>
      </template>
    </el-alert>

    <!-- 马德里大师赛状态卡片 -->
    <div class="madrid-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>马德里大师赛</h2>
          <el-tag :type="getStatusType(madridBracket.status)" size="large">
            {{ getStatusText(madridBracket.status) }}
          </el-tag>
        </div>
      </div>

      <!-- 参赛队伍统计 -->
      <div class="teams-stats">
        <el-statistic title="参赛队伍总数" :value="32" />
        <el-statistic title="小组数量" :value="8" suffix="组" />
        <el-statistic title="东半区队伍" :value="16" />
        <el-statistic title="西半区队伍" :value="16" />
      </div>

      <!-- 小组赛阶段 -->
      <el-card v-if="madridBracket.status !== 'not_started'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>📊 小组赛阶段</span>
            <el-tag v-if="isGroupStageComplete" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <!-- 小组赛积分榜 -->
        <div class="group-standings">
          <el-tabs v-model="activeGroup" type="card">
            <el-tab-pane
              v-for="group in madridBracket.groups"
              :key="group.groupName"
              :label="`${group.groupName}组`"
              :name="group.groupName"
            >
              <ClauchGroupStanding
                :group="group"
                @simulate-match="handleSimulateMatch"
                @view-detail="viewMatchDetails"
              />
            </el-tab-pane>
          </el-tabs>
        </div>

        <!-- 生成淘汰赛按钮 -->
        <div v-if="isGroupStageComplete && madridBracket.status === 'group_stage'" class="generate-knockout-section">
          <el-alert
            title="小组赛已完成！"
            description="所有小组赛比赛已完成，各小组前2名已晋级。现在可以生成淘汰赛对阵。"
            type="success"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <el-button
            type="primary"
            size="large"
            @click="handleGenerateKnockout"
            :loading="generatingKnockout"
          >
            <el-icon><Plus /></el-icon>
            生成淘汰赛对阵
          </el-button>
        </div>
      </el-card>

      <!-- 淘汰赛阶段 -->
      <el-card v-if="madridBracket.status === 'knockout_stage' || madridBracket.status === 'completed'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🏅 淘汰赛阶段</span>
            <el-tag v-if="madridBracket.status === 'completed'" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <!-- 淘汰赛对阵图 -->
        <div class="knockout-brackets">
          <div class="bracket-section">
            <h3>东半区</h3>
            <ClauchKnockoutBracket
              v-if="madridBracket.knockoutEast"
              :knockout="madridBracket.knockoutEast"
              bracket="east"
              @simulate-match="handleSimulateMatch"
              @view-detail="viewMatchDetails"
            />
          </div>

          <div class="bracket-section">
            <h3>西半区</h3>
            <ClauchKnockoutBracket
              v-if="madridBracket.knockoutWest"
              :knockout="madridBracket.knockoutWest"
              bracket="west"
              @simulate-match="handleSimulateMatch"
              @view-detail="viewMatchDetails"
            />
          </div>
        </div>

        <!-- 决赛区域 -->
        <div v-if="showFinals" class="finals-section">
          <h3>🏆 决赛阶段</h3>
          <div class="finals-matches">
            <!-- 季军赛 -->
            <div v-if="madridBracket.thirdPlaceMatch" class="final-match third-place">
              <h4>🥉 季军赛</h4>
              <ClauchMatchCard
                :match="madridBracket.thirdPlaceMatch"
                @simulate="handleSimulateMatch"
                @view-detail="viewMatchDetails"
              />
            </div>

            <!-- 总决赛 -->
            <div v-if="madridBracket.grandFinal" class="final-match grand-final">
              <h4>🏆 总决赛</h4>
              <ClauchMatchCard
                :match="madridBracket.grandFinal"
                @simulate="handleSimulateMatch"
                @view-detail="viewMatchDetails"
              />
            </div>
          </div>
        </div>
      </el-card>

      <!-- 最终排名 -->
      <div v-if="madridBracket.status === 'completed'" class="final-standings">
        <h3>最终排名与积分</h3>
        <div class="standings-grid">
          <div class="standing-item champion">
            <div class="rank-badge">🏆 冠军</div>
            <div class="team-name">{{ madridBracket.champion?.teamName }}</div>
            <div class="region-name">{{ madridBracket.champion?.regionName }}</div>
            <div class="points">+20分</div>
          </div>

          <div class="standing-item runner-up">
            <div class="rank-badge">🥈 亚军</div>
            <div class="team-name">{{ madridBracket.runnerUp?.teamName }}</div>
            <div class="region-name">{{ madridBracket.runnerUp?.regionName }}</div>
            <div class="points">+16分</div>
          </div>

          <div class="standing-item third">
            <div class="rank-badge">🥉 季军</div>
            <div class="team-name">{{ madridBracket.thirdPlace?.teamName }}</div>
            <div class="region-name">{{ madridBracket.thirdPlace?.regionName }}</div>
            <div class="points">+12分</div>
          </div>

          <div class="standing-item fourth">
            <div class="rank-badge">4️⃣ 殿军</div>
            <div class="team-name">{{ madridBracket.fourthPlace?.teamName }}</div>
            <div class="region-name">{{ madridBracket.fourthPlace?.regionName }}</div>
            <div class="points">+8分</div>
          </div>
        </div>

        <!-- 马德里大师赛完成后的操作区 -->
        <div class="madrid-completed-actions">
          <el-alert
            title="马德里大师赛已完成！"
            type="success"
            :closable="false"
            show-icon
            class="completion-alert"
          >
            <template #default>
              <p>恭喜 <strong>{{ madridBracket.champion?.teamName }}</strong> 获得马德里大师赛冠军！</p>
            </template>
          </el-alert>
        </div>
      </div>
    </div>

    <!-- PowerEngine 比赛详情弹窗 -->
    <MatchDetailDialog
      v-if="currentMatchDetail"
      :visible="showMatchDetailDialog"
      :match-detail="currentMatchDetail"
      @close="handleCloseMatchDetail"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Trophy,
  ArrowLeft,
  DArrowRight,
  Plus
} from '@element-plus/icons-vue'
import ClauchGroupStanding from '@/components/clauch/ClauchGroupStanding.vue'
import ClauchKnockoutBracket from '@/components/clauch/ClauchKnockoutBracket.vue'
import ClauchMatchCard from '@/components/clauch/ClauchMatchCard.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi } from '@/api/tauri'
import type { BracketInfo, MatchBracketInfo, GroupStandingInfo, DetailedGameResult, PlayerGameStats } from '@/api/tauri'
import type { ClauchMatch, ClauchGroup, ClauchGroupStanding as ClauchGroupStandingType, ClauchKnockoutBracket as ClauchKnockoutBracketType } from '@/types/clauch'
import type { PlayerPosition } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import { createLogger } from '@/utils/logger'

const logger = createLogger('MadridDetail')

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const timeStore = useTimeStore()

// 阶段检查
const MADRID_PHASE = 'MADRID_MASTERS'
const phaseNotReached = computed(() => {
  const currentPhase = timeStore.currentPhase
  // 如果已经有比赛数据，说明赛事已初始化，不显示警告
  if (madridBracket.groups.length > 0 || (bracketData.value?.matches?.length ?? 0) > 0) {
    return false
  }
  // 后端使用 SCREAMING_SNAKE_CASE 格式序列化阶段名称
  const phaseOrder = [
    'SPRING_REGULAR', 'SPRING_PLAYOFFS', 'MSI', 'MADRID_MASTERS',
    'SUMMER_REGULAR', 'SUMMER_PLAYOFFS', 'CLAUDE_INTERCONTINENTAL',
    'WORLD_CHAMPIONSHIP', 'SHANGHAI_MASTERS', 'ICP_INTERCONTINENTAL',
    'SUPER_INTERCONTINENTAL', 'TRANSFER_WINDOW', 'DRAFT', 'SEASON_END'
  ]
  const currentIndex = phaseOrder.indexOf(currentPhase)
  const targetIndex = phaseOrder.indexOf(MADRID_PHASE)
  // 只有当前阶段早于目标阶段时才显示警告
  return currentIndex >= 0 && currentIndex < targetIndex
})

const currentPhaseDisplay = computed(() => timeStore.phaseDisplayName)

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

const router = useRouter()
const route = useRoute()

// 响应式状态
const tournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)
const groupStandings = ref<GroupStandingInfo[]>([])
const loading = ref(false)
const generatingKnockout = ref(false)
const simulatingGroupStage = ref(false)
const simulatingKnockout = ref(false)
const simulationProgress = ref(0)
const activeGroup = ref('A')

// 马德里大师赛数据 - 从后端获取并转换
const madridBracket = reactive({
  id: '',
  seasonYear: 2024,
  status: 'not_started' as 'not_started' | 'group_stage' | 'knockout_stage' | 'completed',
  groups: [] as ClauchGroup[],
  knockoutEast: null as ClauchKnockoutBracketType | null,
  knockoutWest: null as ClauchKnockoutBracketType | null,
  thirdPlaceMatch: null as ClauchMatch | null,
  grandFinal: null as ClauchMatch | null,
  champion: null as { teamName: string; regionName: string } | null,
  runnerUp: null as { teamName: string; regionName: string } | null,
  thirdPlace: null as { teamName: string; regionName: string } | null,
  fourthPlace: null as { teamName: string; regionName: string } | null
})

// 初始化：从路由参数获取赛事ID或创建新赛事
onMounted(async () => {
  loading.value = true
  try {
    // 先刷新时间状态，确保阶段检查是最新的
    await timeStore.fetchTimeState()

    const idParam = route.params.id || route.query.tournamentId
    if (idParam) {
      tournamentId.value = Number(idParam)
      await loadTournamentData()
    } else {
      // 如果没有ID，显示创建按钮或自动创建
      ElMessage.info('请先创建马德里大师赛')
    }
  } catch (error) {
    logger.error('初始化失败:', error)
    ElMessage.error('加载赛事数据失败')
  } finally {
    loading.value = false
  }
})

// 从后端加载赛事数据
const loadTournamentData = async () => {
  if (!tournamentId.value) return

  try {
    // 获取对阵数据
    bracketData.value = await internationalApi.getTournamentBracket(tournamentId.value)

    // 获取小组赛积分榜
    groupStandings.value = await internationalApi.getGroupStandings(tournamentId.value)

    // 转换数据格式适配前端组件
    convertBracketToMadridFormat()
  } catch (error) {
    logger.error('加载赛事数据失败:', error)
    throw error
  }
}

// 将后端数据转换为前端组件需要的格式
const convertBracketToMadridFormat = () => {
  if (!bracketData.value) return

  madridBracket.id = String(bracketData.value.tournament_id)

  // 确定赛事状态
  const stages = bracketData.value.stages
  const hasGroupStage = stages.some(s => s.name.startsWith('GROUP_'))
  const grandFinalMatch = bracketData.value.matches.find(m => m.stage === 'GRAND_FINAL')

  // 辅助函数：检查比赛状态是否为已完成（兼容大小写）
  const isMatchCompleted = (status: string) => status === 'Completed' || status === 'COMPLETED' || status === 'completed'

  // 检查淘汰赛是否真正生成（有比赛且队伍不为空）
  const knockoutMatches = bracketData.value.matches.filter(m =>
    m.stage.startsWith('EAST_') || m.stage.startsWith('WEST_') ||
    m.stage === 'THIRD_PLACE' || m.stage === 'GRAND_FINAL'
  )
  const hasRealKnockout = knockoutMatches.length > 0 &&
    knockoutMatches.some(m => m.home_team?.id || m.away_team?.id)

  // 检查小组赛是否完成
  const groupMatches = bracketData.value.matches.filter(m => m.stage.startsWith('GROUP_'))
  const _allGroupComplete = groupMatches.length > 0 &&
    groupMatches.every(m => m.status && isMatchCompleted(m.status))

  if (grandFinalMatch?.status && isMatchCompleted(grandFinalMatch.status)) {
    madridBracket.status = 'completed'
  } else if (hasRealKnockout) {
    // 淘汰赛已生成且有队伍数据
    madridBracket.status = 'knockout_stage'
  } else if (hasGroupStage) {
    // 只有小组赛阶段，或淘汰赛还未生成
    madridBracket.status = 'group_stage'
  }

  // 转换小组赛数据
  madridBracket.groups = convertGroupsData()

  // 转换淘汰赛数据
  if (madridBracket.status === 'knockout_stage' || madridBracket.status === 'completed') {
    convertKnockoutData()
  }

  // 设置最终排名
  if (madridBracket.status === 'completed') {
    setFinalStandings()
  }
}

// 转换小组赛数据
const convertGroupsData = (): ClauchGroup[] => {
  if (!bracketData.value) return []

  const groups: ClauchGroup[] = []
  const groupNames = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H']

  for (const groupName of groupNames) {
    const stageName = `GROUP_${groupName}`
    const groupMatches = bracketData.value.matches.filter(m => m.stage === stageName)

    if (groupMatches.length === 0) continue

    // 从积分榜获取队伍信息
    const standingInfo = groupStandings.value.find(g => g.group_name === groupName)

    // 转换比赛格式
    const matches: ClauchMatch[] = groupMatches.map(m => ({
      id: String(m.match_id),
      teamAId: String(m.home_team?.id || ''),
      teamAName: m.home_team?.name || '待定',
      teamBId: String(m.away_team?.id || ''),
      teamBName: m.away_team?.name || '待定',
      scoreA: m.home_score,
      scoreB: m.away_score,
      winnerId: m.winner_id ? String(m.winner_id) : null,
      status: (m.status === 'Completed' || m.status === 'COMPLETED' || m.status === 'completed') ? 'completed' : 'scheduled',
      bestOf: m.format === 'Bo3' ? 3 : m.format === 'Bo5' ? 5 : 1,
      stage: 'group',
      groupName: groupName,
      roundNumber: m.match_order
    }))

    // 转换积分榜格式
    const standings: ClauchGroupStandingType[] = standingInfo?.teams.map((team, idx) => ({
      teamId: String(team.team_id),
      teamName: team.team_name,
      position: idx + 1,
      matchesPlayed: team.wins + team.losses,
      wins: team.wins,
      losses: team.losses,
      points: team.points,
      roundsWon: team.games_won,
      roundsLost: team.games_lost,
      roundDifferential: team.games_won - team.games_lost,
      qualified: idx < 2
    })) || []

    groups.push({
      groupName,
      standings,
      matches
    })
  }

  return groups
}

// 转换淘汰赛数据
const convertKnockoutData = () => {
  if (!bracketData.value) return

  const matches = bracketData.value.matches

  // 东半区
  const eastR1 = matches.filter(m => m.stage === 'EAST_R1').sort((a, b) => a.match_order - b.match_order)
  const eastSemi = matches.filter(m => m.stage === 'EAST_SEMI').sort((a, b) => a.match_order - b.match_order)
  const eastFinal = matches.filter(m => m.stage === 'EAST_FINAL')

  if (eastR1.length > 0) {
    madridBracket.knockoutEast = {
      round1: eastR1.map(m => convertMatchFormat(m, 'east_quarter')),
      semiFinals: eastSemi.map(m => convertMatchFormat(m, 'east_semi')),
      final: eastFinal.map(m => convertMatchFormat(m, 'east_final'))
    }
  }

  // 西半区
  const westR1 = matches.filter(m => m.stage === 'WEST_R1').sort((a, b) => a.match_order - b.match_order)
  const westSemi = matches.filter(m => m.stage === 'WEST_SEMI').sort((a, b) => a.match_order - b.match_order)
  const westFinal = matches.filter(m => m.stage === 'WEST_FINAL')

  if (westR1.length > 0) {
    madridBracket.knockoutWest = {
      round1: westR1.map(m => convertMatchFormat(m, 'west_quarter')),
      semiFinals: westSemi.map(m => convertMatchFormat(m, 'west_semi')),
      final: westFinal.map(m => convertMatchFormat(m, 'west_final'))
    }
  }

  // 季军赛
  const thirdPlace = matches.find(m => m.stage === 'THIRD_PLACE')
  if (thirdPlace) {
    madridBracket.thirdPlaceMatch = convertMatchFormat(thirdPlace, 'third_place')
  }

  // 总决赛
  const grandFinal = matches.find(m => m.stage === 'GRAND_FINAL')
  if (grandFinal) {
    madridBracket.grandFinal = convertMatchFormat(grandFinal, 'grand_final')
  }
}

// 转换比赛格式
const convertMatchFormat = (m: MatchBracketInfo, matchType: string): ClauchMatch => ({
  id: String(m.match_id),
  teamAId: String(m.home_team?.id || ''),
  teamAName: m.home_team?.name || '待定',
  teamBId: String(m.away_team?.id || ''),
  teamBName: m.away_team?.name || '待定',
  scoreA: m.home_score,
  scoreB: m.away_score,
  winnerId: m.winner_id ? String(m.winner_id) : null,
  status: (m.status === 'Completed' || m.status === 'COMPLETED' || m.status === 'completed') ? 'completed' : 'scheduled',
  bestOf: m.format === 'Bo3' ? 3 : m.format === 'Bo5' ? 5 : 1,
  matchType
})

// 设置最终排名
const setFinalStandings = () => {
  const gf = madridBracket.grandFinal
  const tp = madridBracket.thirdPlaceMatch

  if (gf && gf.winnerId) {
    if (gf.winnerId === gf.teamAId) {
      madridBracket.champion = { teamName: gf.teamAName || '', regionName: '' }
      madridBracket.runnerUp = { teamName: gf.teamBName || '', regionName: '' }
    } else {
      madridBracket.champion = { teamName: gf.teamBName || '', regionName: '' }
      madridBracket.runnerUp = { teamName: gf.teamAName || '', regionName: '' }
    }
  }

  if (tp && tp.winnerId) {
    if (tp.winnerId === tp.teamAId) {
      madridBracket.thirdPlace = { teamName: tp.teamAName || '', regionName: '' }
      madridBracket.fourthPlace = { teamName: tp.teamBName || '', regionName: '' }
    } else {
      madridBracket.thirdPlace = { teamName: tp.teamBName || '', regionName: '' }
      madridBracket.fourthPlace = { teamName: tp.teamAName || '', regionName: '' }
    }
  }
}

// 计算属性
const isGroupStageComplete = computed(() => {
  // 如果没有小组或没有比赛，则认为未完成
  if (madridBracket.groups.length === 0) return false
  const hasMatches = madridBracket.groups.some(group => group.matches.length > 0)
  if (!hasMatches) return false
  return madridBracket.groups.every(group => {
    if (group.matches.length === 0) return false
    return group.matches.every(match => match.status === 'completed')
  })
})

const showFinals = computed(() => {
  return madridBracket.thirdPlaceMatch || madridBracket.grandFinal
})

// 方法
const goBack = () => {
  router.push('/tournaments')
}

const getStatusType = (status: string) => {
  const typeMap: Record<string, any> = {
    'not_started': 'info',
    'group_stage': 'warning',
    'knockout_stage': 'warning',
    'completed': 'success'
  }
  return typeMap[status] || 'info'
}

const getStatusText = (status: string) => {
  const textMap: Record<string, string> = {
    'not_started': '未开始',
    'group_stage': '小组赛进行中',
    'knockout_stage': '淘汰赛进行中',
    'completed': '已完成'
  }
  return textMap[status] || status
}

/**
 * 获取队伍名称
 */
const getTeamNameById = (teamId: string | number | null): string => {
  if (!teamId) return '待定'
  const numId = typeof teamId === 'string' ? parseInt(teamId) : teamId
  if (isNaN(numId)) return '待定'

  // 从 bracketData.matches 中查找队伍
  if (bracketData.value?.matches) {
    for (const match of bracketData.value.matches) {
      if (match.home_team?.id === numId) {
        return match.home_team.short_name || match.home_team.name || '待定'
      }
      if (match.away_team?.id === numId) {
        return match.away_team.short_name || match.away_team.name || '待定'
      }
    }
  }

  // 从 madridBracket.groups 中查找
  for (const group of madridBracket.groups) {
    for (const standing of group.standings) {
      if (String(standing.teamId) === String(teamId)) {
        return standing.teamName || '待定'
      }
    }
  }

  return '待定'
}

/**
 * 修正比赛详情中空的或不正确的队名和MVP
 */
const fixMatchDetailTeamNames = (detail: MatchDetail, matchOrId: ClauchMatch | string | number): MatchDetail => {
  const fixedDetail = { ...detail }
  const match = typeof matchOrId === 'object' ? matchOrId : null

  // 修正比赛级别的队名
  if (!fixedDetail.teamAName || fixedDetail.teamAName === '待定' || fixedDetail.teamAName === '') {
    fixedDetail.teamAName = getTeamNameById(fixedDetail.teamAId) || match?.teamAName || '队伍A'
  }
  if (!fixedDetail.teamBName || fixedDetail.teamBName === '待定' || fixedDetail.teamBName === '') {
    fixedDetail.teamBName = getTeamNameById(fixedDetail.teamBId) || match?.teamBName || '队伍B'
  }

  // 修正胜者名称
  if (fixedDetail.winnerId) {
    const winnerName = getTeamNameById(fixedDetail.winnerId)
    if (winnerName && winnerName !== '待定') {
      fixedDetail.winnerName = winnerName
    }
  }

  // 修正每局比赛的队名
  if (fixedDetail.games) {
    fixedDetail.games = fixedDetail.games.map(game => {
      const fixedGame = { ...game }
      if (!fixedGame.teamAName || fixedGame.teamAName === '待定' || fixedGame.teamAName === '') {
        fixedGame.teamAName = getTeamNameById(fixedGame.teamAId) || fixedDetail.teamAName
      }
      if (!fixedGame.teamBName || fixedGame.teamBName === '待定' || fixedGame.teamBName === '') {
        fixedGame.teamBName = getTeamNameById(fixedGame.teamBId) || fixedDetail.teamBName
      }
      if (fixedGame.winnerId) {
        const winnerName = getTeamNameById(fixedGame.winnerId)
        if (winnerName && winnerName !== '待定') {
          fixedGame.winnerName = winnerName
        }
      }
      return fixedGame
    })
  }

  // 修正 MVP：确保 MVP 来自胜者队伍，如果不是则重新计算
  if (fixedDetail.games && fixedDetail.games.length > 0) {
    const needRecalcMvp = !fixedDetail.mvpPlayerId || fixedDetail.mvpTeamId !== fixedDetail.winnerId

    if (needRecalcMvp) {
      const winnerId = fixedDetail.winnerId
      const isHomeWinner = fixedDetail.winnerId === fixedDetail.teamAId

      // 收集胜方队伍所有选手的累计影响力
      const winnerPlayerStats = new Map<string, { name: string, totalImpact: number, gameCount: number }>()

      fixedDetail.games.forEach(game => {
        // 只收集胜方队伍的选手数据
        const winnerPlayers = isHomeWinner ? game.teamAPlayers : game.teamBPlayers

        if (winnerPlayers) {
          winnerPlayers.forEach(p => {
            const existing = winnerPlayerStats.get(p.playerId)
            if (existing) {
              existing.totalImpact += p.impactScore || 0
              existing.gameCount++
            } else {
              winnerPlayerStats.set(p.playerId, {
                name: p.playerName,
                totalImpact: p.impactScore || 0,
                gameCount: 1
              })
            }
          })
        }
      })

      // 找出平均影响力最高的选手作为 MVP
      let maxAvgImpact = -Infinity
      winnerPlayerStats.forEach((stats, playerId) => {
        const avgImpact = stats.totalImpact / stats.gameCount
        if (avgImpact > maxAvgImpact) {
          maxAvgImpact = avgImpact
          fixedDetail.mvpPlayerId = playerId
          fixedDetail.mvpPlayerName = stats.name
          fixedDetail.mvpTeamId = winnerId
          fixedDetail.mvpTotalImpact = stats.totalImpact
        }
      })
    }
  }

  return fixedDetail
}

/**
 * 查看比赛详情
 * 支持传入 ClauchMatch 对象或者 matchId
 */
const viewMatchDetails = async (matchOrId: ClauchMatch | string | number) => {
  // 获取 matchId
  const matchId = typeof matchOrId === 'object' ? matchOrId.id : matchOrId

  // 先尝试从内存获取详情
  let detail = matchDetailStore.getMatchDetail(matchId)
  if (detail) {
    currentMatchDetail.value = fixMatchDetailTeamNames(detail, matchOrId)
    showMatchDetailDialog.value = true
    return
  }

  // 如果内存中没有，尝试从数据库加载
  detail = await matchDetailStore.loadMatchDetailFromDb(matchId)
  if (detail) {
    currentMatchDetail.value = fixMatchDetailTeamNames(detail, matchOrId)
    showMatchDetailDialog.value = true
    return
  }

  // 如果没有详情数据，显示消息
  ElMessage.info('该比赛暂无详细数据')
}

/**
 * 关闭比赛详情弹窗
 */
const handleCloseMatchDetail = () => {
  showMatchDetailDialog.value = false
  currentMatchDetail.value = null
}

/**
 * 模拟单场比赛 - 使用后端 API
 */
const handleSimulateMatch = async (match: ClauchMatch) => {
  try {
    const matchId = Number(match.id)

    // 调用后端比赛模拟 API
    const result = await matchApi.simulateMatchDetailed(matchId)

    // 更新比赛状态
    match.scoreA = result.home_score
    match.scoreB = result.away_score
    match.winnerId = String(result.winner_id)
    match.status = 'completed'
    match.completedAt = new Date()

    // 解析位置格式（后端可能返回 "Some(Adc)" 格式）
    const parsePosition = (pos: string | null | undefined): string => {
      if (!pos) return 'MID'
      const someMatch = pos.match(/Some\((\w+)\)/)
      if (someMatch) return someMatch[1]
      return pos
    }

    // 将位置转换为标准格式
    const normalizePosition = (pos: string): PlayerPosition => {
      const posMap: Record<string, PlayerPosition> = {
        'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP',
        'top': 'TOP', 'jungle': 'JUG', 'mid': 'MID', 'adc': 'ADC', 'support': 'SUP',
        'TOP': 'TOP', 'JUG': 'JUG', 'MID': 'MID', 'ADC': 'ADC', 'SUP': 'SUP',
        'Jug': 'JUG', 'Sup': 'SUP',  // 后端 Rust 枚举格式
      }
      return posMap[pos] || 'MID'
    }

    // 将后端结果转换为前端 MatchDetail 格式
    const convertPlayerPerformance = (p: PlayerGameStats, teamId: string) => ({
      playerId: String(p.player_id),
      playerName: p.player_name,
      position: normalizePosition(parsePosition(p.position)),
      teamId: teamId,
      baseAbility: p.base_ability,
      conditionBonus: p.condition_bonus,
      stabilityNoise: p.stability_noise,
      actualAbility: p.actual_ability,
      impactScore: p.impact_score,
      traits: p.traits as any[],
      activatedTraits: p.activated_traits?.map(t => ({
        type: t.trait_type as any,
        name: t.name,
        effect: t.effect,
        value: t.value,
        isPositive: t.is_positive
      }))
    })

    // 计算队伍战力（选手实际发挥能力平均值）
    const calcTeamPower = (players: PlayerGameStats[]) => {
      if (!players || players.length === 0) return 0
      const sum = players.reduce((acc: number, p) => acc + (p.actual_ability || p.base_ability || 0), 0)
      return sum / players.length
    }

    // 保存比赛详情到 Store (用于展示)
    const matchDetail: MatchDetail = {
      matchId: match.id,
      tournamentType: 'madrid',
      seasonId: String(madridBracket.seasonYear),
      teamAId: String(match.teamAId || ''),
      teamAName: match.teamAName || '',
      teamBId: String(match.teamBId || ''),
      teamBName: match.teamBName || '',
      bestOf: match.bestOf || 3,
      finalScoreA: result.home_score,
      finalScoreB: result.away_score,
      winnerId: String(result.winner_id),
      winnerName: result.winner_id === result.home_team_id ? (match.teamAName || '') : (match.teamBName || ''),
      mvpPlayerId: result.match_mvp ? String(result.match_mvp.player_id) : undefined,
      mvpPlayerName: result.match_mvp?.player_name,
      mvpTeamId: result.match_mvp ? String(result.match_mvp.team_id) : undefined,
      mvpTotalImpact: result.match_mvp?.mvp_score,
      games: result.games.map((game: DetailedGameResult) => {
        const teamAPower = calcTeamPower(game.home_players)
        const teamBPower = calcTeamPower(game.away_players)
        return {
          gameNumber: game.game_number,
          teamAId: String(match.teamAId || ''),
          teamAName: match.teamAName || '',
          teamAPower,
          teamAPerformance: game.home_performance,
          teamAPlayers: game.home_players.map(p => convertPlayerPerformance(p, String(match.teamAId || ''))),
          teamBId: String(match.teamBId || ''),
          teamBName: match.teamBName || '',
          teamBPower,
          teamBPerformance: game.away_performance,
          teamBPlayers: game.away_players.map(p => convertPlayerPerformance(p, String(match.teamBId || ''))),
          winnerId: String(game.winner_id),
          winnerName: game.winner_id === result.home_team_id ? (match.teamAName || '') : (match.teamBName || ''),
          powerDifference: teamAPower - teamBPower,
          performanceDifference: game.home_performance - game.away_performance,
          isUpset: (teamAPower > teamBPower && game.winner_id !== result.home_team_id) ||
                   (teamBPower > teamAPower && game.winner_id === result.home_team_id)
        }
      })
    }
    await matchDetailStore.saveMatchDetail(match.id, matchDetail)

    // 同时用数据库 ID 保存一份，确保能从数据库加载
    if (match.backendMatchId) {
      const dbMatchDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
      await matchDetailStore.saveMatchDetail(match.backendMatchId, dbMatchDetail)
    }

    // 记录选手表现到统计（国际赛事使用 INTL 标识）
    matchDetail.games.forEach(game => {
      game.teamAPlayers.forEach(perf => {
        playerStore.recordPerformance(
          perf.playerId,
          perf.playerName,
          perf.teamId,
          perf.position,
          perf.impactScore,
          perf.actualAbility,
          String(madridBracket.seasonYear),
          'INTL'
        )
      })
      game.teamBPlayers.forEach(perf => {
        playerStore.recordPerformance(
          perf.playerId,
          perf.playerName,
          perf.teamId,
          perf.position,
          perf.impactScore,
          perf.actualAbility,
          String(madridBracket.seasonYear),
          'INTL'
        )
      })
    })
    playerStore.saveToStorage()

    ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

    // 更新淘汰赛对阵（如果需要）
    if (tournamentId.value && result.winner_id) {
      try {
        await internationalApi.advanceBracket(tournamentId.value, matchId, result.winner_id)
      } catch (e) {
        // 可能不是淘汰赛阶段，忽略
      }
    }

    // 刷新数据
    await loadTournamentData()

    // 检查是否完成
    if (madridBracket.status === 'knockout_stage') {
      checkKnockoutCompletion()
    }
  } catch (error) {
    logger.error('模拟比赛失败:', error)
    ElMessage.error('模拟比赛失败')
  }
}

/**
 * 检查淘汰赛是否完成
 */
const checkKnockoutCompletion = () => {
  if (madridBracket.grandFinal?.status === 'completed') {
    // 设置最终排名
    const grandFinal = madridBracket.grandFinal
    const thirdPlaceMatch = madridBracket.thirdPlaceMatch

    if (grandFinal.winnerId === grandFinal.teamAId) {
      madridBracket.champion = { teamName: grandFinal.teamAName || '', regionName: 'LPL' }
      madridBracket.runnerUp = { teamName: grandFinal.teamBName || '', regionName: 'LCK' }
    } else {
      madridBracket.champion = { teamName: grandFinal.teamBName || '', regionName: 'LCK' }
      madridBracket.runnerUp = { teamName: grandFinal.teamAName || '', regionName: 'LPL' }
    }

    if (thirdPlaceMatch && thirdPlaceMatch.winnerId === thirdPlaceMatch.teamAId) {
      madridBracket.thirdPlace = { teamName: thirdPlaceMatch.teamAName || '', regionName: 'LEC' }
      madridBracket.fourthPlace = { teamName: thirdPlaceMatch.teamBName || '', regionName: 'LCS' }
    } else if (thirdPlaceMatch) {
      madridBracket.thirdPlace = { teamName: thirdPlaceMatch.teamBName || '', regionName: 'LCS' }
      madridBracket.fourthPlace = { teamName: thirdPlaceMatch.teamAName || '', regionName: 'LEC' }
    }

    madridBracket.status = 'completed'
    showChampionCelebration(madridBracket.champion?.teamName || '')
  }
}

/**
 * 生成淘汰赛对阵 - 使用后端 API
 */
const handleGenerateKnockout = async () => {
  if (!tournamentId.value) {
    ElMessage.error('赛事ID不存在')
    return
  }

  generatingKnockout.value = true

  try {
    // 调用后端生成淘汰赛对阵
    await internationalApi.generateKnockoutBracket(tournamentId.value)

    // 刷新数据
    await loadTournamentData()

    ElMessage.success('淘汰赛对阵生成成功!')
  } catch (error) {
    logger.error('生成淘汰赛对阵失败:', error)
    ElMessage.error('生成淘汰赛对阵失败')
  } finally {
    generatingKnockout.value = false
  }
}

/**
 * 批量模拟小组赛 - 使用后端 API
 */
const batchSimulateGroupStage = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的小组赛比赛。是否继续?',
      '模拟小组赛确认',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    simulatingGroupStage.value = true
    simulationProgress.value = 0

    // 获取所有未完成的小组赛比赛
    const groupMatches = madridBracket.groups.flatMap(g => g.matches)
    const uncompletedGroupMatches = groupMatches.filter(m => m.status !== 'completed')

    for (let i = 0; i < uncompletedGroupMatches.length; i++) {
      const match = uncompletedGroupMatches[i]
      const matchId = Number(match.id)

      try {
        // 调用后端模拟
        const result = await matchApi.simulateMatchDetailed(matchId)

        // 解析位置格式（后端可能返回 "Some(Adc)" 格式）
        const parsePosition = (pos: string | null | undefined): string => {
          if (!pos) return 'MID'
          const someMatch = pos.match(/Some\((\w+)\)/)
          if (someMatch) return someMatch[1]
          return pos
        }

        // 将位置转换为标准格式
        const normalizePosition = (pos: string): PlayerPosition => {
          const posMap: Record<string, PlayerPosition> = {
            'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP',
            'top': 'TOP', 'jungle': 'JUG', 'mid': 'MID', 'adc': 'ADC', 'support': 'SUP',
            'TOP': 'TOP', 'JUG': 'JUG', 'MID': 'MID', 'ADC': 'ADC', 'SUP': 'SUP',
            'Jug': 'JUG', 'Sup': 'SUP',  // 后端 Rust 枚举格式
          }
          return posMap[pos] || 'MID'
        }

        // 转换并保存比赛详情
        const convertPlayerPerformance = (p: any, teamId: string) => ({
          playerId: String(p.player_id),
          playerName: p.player_name,
          position: normalizePosition(parsePosition(p.position)),
          teamId: teamId,
          baseAbility: p.base_ability,
          conditionBonus: p.condition_bonus,
          stabilityNoise: p.stability_noise,
          actualAbility: p.actual_ability,
          impactScore: p.impact_score,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        })

        // 计算队伍战力（选手实际发挥能力平均值）
        const calcTeamPower = (players: any[]) => {
          if (!players || players.length === 0) return 0
          const sum = players.reduce((acc: number, p: any) => acc + (p.actual_ability || p.base_ability || 0), 0)
          return sum / players.length
        }

        const matchDetail: MatchDetail = {
          matchId: match.id,
          tournamentType: 'madrid',
          seasonId: String(madridBracket.seasonYear),
          teamAId: String(match.teamAId || ''),
          teamAName: match.teamAName || '',
          teamBId: String(match.teamBId || ''),
          teamBName: match.teamBName || '',
          bestOf: match.bestOf || 3,
          finalScoreA: result.home_score,
          finalScoreB: result.away_score,
          winnerId: String(result.winner_id),
          winnerName: result.winner_id === result.home_team_id ? (match.teamAName || '') : (match.teamBName || ''),
          mvpPlayerId: result.match_mvp ? String(result.match_mvp.player_id) : undefined,
          mvpPlayerName: result.match_mvp?.player_name,
          mvpTeamId: result.match_mvp ? String(result.match_mvp.team_id) : undefined,
          mvpTotalImpact: result.match_mvp?.mvp_score,
          games: result.games.map((game: any) => {
            const teamAPower = calcTeamPower(game.home_players)
            const teamBPower = calcTeamPower(game.away_players)
            return {
              gameNumber: game.game_number,
              teamAId: String(match.teamAId || ''),
              teamAName: match.teamAName || '',
              teamAPower,
              teamAPerformance: game.home_performance,
              teamAPlayers: game.home_players.map((p: any) => convertPlayerPerformance(p, String(match.teamAId || ''))),
              teamBId: String(match.teamBId || ''),
              teamBName: match.teamBName || '',
              teamBPower,
              teamBPerformance: game.away_performance,
              teamBPlayers: game.away_players.map((p: any) => convertPlayerPerformance(p, String(match.teamBId || ''))),
              winnerId: String(game.winner_id),
              winnerName: game.winner_id === result.home_team_id ? (match.teamAName || '') : (match.teamBName || ''),
              powerDifference: teamAPower - teamBPower,
              performanceDifference: game.home_performance - game.away_performance,
              isUpset: (teamAPower > teamBPower && game.winner_id !== result.home_team_id) ||
                       (teamBPower > teamAPower && game.winner_id === result.home_team_id)
            }
          })
        }
        await matchDetailStore.saveMatchDetail(match.id, matchDetail)

        // 同时用数据库 ID 保存一份
        if (match.backendMatchId) {
          const dbMatchDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
          await matchDetailStore.saveMatchDetail(match.backendMatchId, dbMatchDetail)
        }

        // 记录选手表现
        matchDetail.games.forEach(game => {
          game.teamAPlayers.forEach(perf => {
            playerStore.recordPerformance(
              perf.playerId,
              perf.playerName,
              perf.teamId,
              perf.position,
              perf.impactScore,
              perf.actualAbility,
              String(madridBracket.seasonYear),
              'INTL'
            )
          })
          game.teamBPlayers.forEach(perf => {
            playerStore.recordPerformance(
              perf.playerId,
              perf.playerName,
              perf.teamId,
              perf.position,
              perf.impactScore,
              perf.actualAbility,
              String(madridBracket.seasonYear),
              'INTL'
            )
          })
        })
      } catch (e) {
        logger.error(`模拟比赛 ${matchId} 失败:`, e)
      }

      simulationProgress.value = Math.floor(((i + 1) / uncompletedGroupMatches.length) * 100)
      await new Promise(resolve => setTimeout(resolve, 50))
    }

    playerStore.saveToStorage()

    // 刷新数据
    await loadTournamentData()

    ElMessage.success('小组赛模拟完成！现在可以生成淘汰赛对阵。')
  } catch (error: any) {
    if (error !== 'cancel') {
      logger.error('小组赛模拟失败:', error)
      ElMessage.error(error.message || '小组赛模拟失败')
    }
  } finally {
    simulatingGroupStage.value = false
    simulationProgress.value = 0
  }
}

/**
 * 批量模拟淘汰赛 - 使用后端 API
 */
const batchSimulateKnockout = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的淘汰赛比赛，直到决出冠军。是否继续?',
      '模拟淘汰赛确认',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    simulatingKnockout.value = true
    simulationProgress.value = 0

    // 按阶段顺序模拟
    const stages = ['EAST_R1', 'WEST_R1', 'EAST_SEMI', 'WEST_SEMI', 'EAST_FINAL', 'WEST_FINAL', 'THIRD_PLACE', 'GRAND_FINAL']

    for (const stageName of stages) {
      // 获取当前阶段的比赛
      const stageMatches = bracketData.value?.matches.filter(m => m.stage === stageName && m.status !== 'Completed' && m.status !== 'COMPLETED' && m.status !== 'completed') || []

      for (const match of stageMatches) {
        try {
          const result = await matchApi.simulateMatchDetailed(match.match_id)

          // 解析位置格式（后端可能返回 "Some(Adc)" 格式）
          const parsePosition = (pos: string | null | undefined): string => {
            if (!pos) return 'MID'
            const someMatch = pos.match(/Some\((\w+)\)/)
            if (someMatch) return someMatch[1]
            return pos
          }

          // 将位置转换为标准格式
          const normalizePosition = (pos: string): PlayerPosition => {
            const posMap: Record<string, PlayerPosition> = {
              'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP',
              'top': 'TOP', 'jungle': 'JUG', 'mid': 'MID', 'adc': 'ADC', 'support': 'SUP',
              'TOP': 'TOP', 'JUG': 'JUG', 'MID': 'MID', 'ADC': 'ADC', 'SUP': 'SUP',
              'Jug': 'JUG', 'Sup': 'SUP',  // 后端 Rust 枚举格式
            }
            return posMap[pos] || 'MID'
          }

          // 转换并保存比赛详情
          const convertPlayerPerformance = (p: any, teamId: string) => ({
            playerId: String(p.player_id),
            playerName: p.player_name,
            position: normalizePosition(parsePosition(p.position)),
            teamId: teamId,
            baseAbility: p.base_ability,
            conditionBonus: p.condition_bonus,
            stabilityNoise: p.stability_noise,
            actualAbility: p.actual_ability,
            impactScore: p.impact_score,
            traits: p.traits,
            activatedTraits: p.activated_traits?.map((t: any) => ({
              type: t.trait_type,
              name: t.name,
              effect: t.effect,
              value: t.value,
              isPositive: t.is_positive
            }))
          })

          // 计算队伍战力（选手实际发挥能力平均值）
          const calcTeamPower = (players: any[]) => {
            if (!players || players.length === 0) return 0
            const sum = players.reduce((acc: number, p: any) => acc + (p.actual_ability || p.base_ability || 0), 0)
            return sum / players.length
          }

          const teamAId = String(match.home_team?.id || '')
          const teamAName = match.home_team?.name || ''
          const teamBId = String(match.away_team?.id || '')
          const teamBName = match.away_team?.name || ''
          const bestOf = match.format === 'Bo5' ? 5 : match.format === 'Bo3' ? 3 : 1

          const matchDetail: MatchDetail = {
            matchId: String(match.match_id),
            tournamentType: 'madrid',
            seasonId: String(madridBracket.seasonYear),
            teamAId,
            teamAName,
            teamBId,
            teamBName,
            bestOf,
            finalScoreA: result.home_score,
            finalScoreB: result.away_score,
            winnerId: String(result.winner_id),
            winnerName: result.winner_id === result.home_team_id ? teamAName : teamBName,
            mvpPlayerId: result.match_mvp ? String(result.match_mvp.player_id) : undefined,
            mvpPlayerName: result.match_mvp?.player_name,
            mvpTeamId: result.match_mvp ? String(result.match_mvp.team_id) : undefined,
            mvpTotalImpact: result.match_mvp?.mvp_score,
            games: result.games.map((game: any) => {
              const teamAPower = calcTeamPower(game.home_players)
              const teamBPower = calcTeamPower(game.away_players)
              return {
                gameNumber: game.game_number,
                teamAId,
                teamAName,
                teamAPower,
                teamAPerformance: game.home_performance,
                teamAPlayers: game.home_players.map((p: any) => convertPlayerPerformance(p, teamAId)),
                teamBId,
                teamBName,
                teamBPower,
                teamBPerformance: game.away_performance,
                teamBPlayers: game.away_players.map((p: any) => convertPlayerPerformance(p, teamBId)),
                winnerId: String(game.winner_id),
                winnerName: game.winner_id === result.home_team_id ? teamAName : teamBName,
                powerDifference: teamAPower - teamBPower,
                performanceDifference: game.home_performance - game.away_performance,
                isUpset: (teamAPower > teamBPower && game.winner_id !== result.home_team_id) ||
                         (teamBPower > teamAPower && game.winner_id === result.home_team_id)
              }
            })
          }
          await matchDetailStore.saveMatchDetail(String(match.match_id), matchDetail)

          // 记录选手表现
          matchDetail.games.forEach(game => {
            game.teamAPlayers.forEach(perf => {
              playerStore.recordPerformance(
                perf.playerId,
                perf.playerName,
                perf.teamId,
                perf.position,
                perf.impactScore,
                perf.actualAbility,
                String(madridBracket.seasonYear),
                'INTL'
              )
            })
            game.teamBPlayers.forEach(perf => {
              playerStore.recordPerformance(
                perf.playerId,
                perf.playerName,
                perf.teamId,
                perf.position,
                perf.impactScore,
                perf.actualAbility,
                String(madridBracket.seasonYear),
                'INTL'
              )
            })
          })

          // 更新淘汰赛对阵
          if (tournamentId.value) {
            await internationalApi.advanceBracket(tournamentId.value, match.match_id, result.winner_id)
          }
        } catch (e) {
          logger.error(`模拟比赛 ${match.match_id} 失败:`, e)
        }

        await new Promise(resolve => setTimeout(resolve, 100))
      }

      // 刷新数据以获取更新的对阵
      await loadTournamentData()
    }

    playerStore.saveToStorage()

    madridBracket.status = 'completed'
    ElMessage.success('淘汰赛模拟完成！')

    if (madridBracket.champion) {
      showChampionCelebration(madridBracket.champion.teamName)
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      logger.error('淘汰赛模拟失败:', error)
      ElMessage.error(error.message || '淘汰赛模拟失败')
    }
  } finally {
    simulatingKnockout.value = false
    simulationProgress.value = 0
  }
}

/**
 * 显示冠军庆祝动画
 */
const showChampionCelebration = (championName: string) => {
  ElMessageBox.alert(
    `恭喜 ${championName} 获得马德里大师赛冠军，成为全球最强战队!`,
    '🏆 马德里大师赛冠军诞生! 🏆',
    {
      confirmButtonText: '太棒了!',
      customClass: 'champion-celebration-box',
      showClose: false,
      center: true
    }
  )
}
</script>

<style scoped lang="scss">
.madrid-management {
  padding: 24px;

  .phase-warning-alert {
    margin-bottom: 24px;

    .phase-warning-content {
      p {
        margin: 4px 0;
        line-height: 1.6;

        strong {
          color: var(--el-color-warning);
        }
      }
    }
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content {
      .header-left {
        display: flex;
        flex-direction: column;
        gap: 8px;
      }

      .page-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 28px;
        font-weight: 700;
        margin: 0;
        color: #1f2937;
      }

      .page-description {
        margin: 0;
        color: #6b7280;
        font-size: 14px;
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  .madrid-status-card {
    background: white;
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);

    .status-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 24px;
      padding-bottom: 16px;
      border-bottom: 1px solid #e5e7eb;

      .status-info {
        display: flex;
        align-items: center;
        gap: 16px;

        h2 {
          margin: 0;
          font-size: 20px;
          font-weight: 600;
          color: #1f2937;
        }
      }
    }

    .teams-stats {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 20px;
      margin-bottom: 32px;
      padding: 20px;
      background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
      border-radius: 12px;
    }

    .stage-card {
      margin-bottom: 24px;

      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
      }
    }

    .generate-knockout-section {
      margin-top: 24px;
      text-align: center;

      .el-button {
        margin-top: 16px;
      }
    }

    .knockout-brackets {
      display: flex;
      flex-direction: column;
      gap: 32px;
      margin-top: 24px;

      .bracket-section {
        border: 2px solid #e5e7eb;
        border-radius: 12px;
        padding: 20px;
        background: white;

        h3 {
          margin: 0 0 16px 0;
          font-size: 18px;
          font-weight: 600;
          color: #1f2937;
          text-align: center;
        }

        overflow-x: auto;
        overflow-y: hidden;

        &::-webkit-scrollbar {
          height: 8px;
        }

        &::-webkit-scrollbar-track {
          background: #f3f4f6;
          border-radius: 4px;
        }

        &::-webkit-scrollbar-thumb {
          background: #d1d5db;
          border-radius: 4px;

          &:hover {
            background: #9ca3af;
          }
        }
      }
    }

    .finals-section {
      margin-top: 32px;
      padding: 24px;
      background: linear-gradient(135deg, #fef3c7 0%, #fde047 100%);
      border-radius: 12px;

      h3 {
        margin: 0 0 24px 0;
        font-size: 20px;
        font-weight: 700;
        text-align: center;
        color: #92400e;
      }

      .finals-matches {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 24px;

        .final-match {
          h4 {
            margin: 0 0 12px 0;
            font-size: 16px;
            font-weight: 600;
            text-align: center;
          }

          &.third-place {
            border: 2px solid #d97706;
            padding: 16px;
            border-radius: 8px;
            background: white;
          }

          &.grand-final {
            border: 2px solid #f59e0b;
            padding: 16px;
            border-radius: 8px;
            background: white;
          }
        }
      }
    }

    .final-standings {
      margin-top: 32px;

      h3 {
        margin: 0 0 16px 0;
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }

      .standings-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 16px;
        margin-bottom: 24px;

        .standing-item {
          padding: 20px;
          border-radius: 8px;
          text-align: center;
          border: 2px solid;

          .rank-badge {
            font-size: 18px;
            margin-bottom: 8px;
            white-space: nowrap;
          }

          .team-name {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 8px;
            color: #1f2937;
          }

          .region-name {
            font-size: 14px;
            color: #6b7280;
            margin-bottom: 8px;
          }

          .points {
            font-size: 16px;
            font-weight: 700;
            color: #10b981;
          }

          &.champion {
            border-color: #f59e0b;
            background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
          }

          &.runner-up {
            border-color: #9ca3af;
            background: linear-gradient(135deg, #f9fafb 0%, #e5e7eb 100%);
          }

          &.third {
            border-color: #d97706;
            background: linear-gradient(135deg, #fed7aa 0%, #fdba74 100%);
          }

          &.fourth {
            border-color: #60a5fa;
            background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
          }
        }
      }

      .madrid-completed-actions {
        margin-top: 32px;
        text-align: center;

        .completion-alert {
          margin-bottom: 20px;
          border-radius: 8px;
          text-align: left;

          p {
            margin: 8px 0;
            font-size: 14px;
            line-height: 1.6;

            strong {
              color: #f59e0b;
              font-weight: 700;
            }
          }
        }
      }
    }
  }

  .mb-4 {
    margin-bottom: 16px;
  }
}

// 冠军庆祝动画
@keyframes champion-bounce {
  0% {
    transform: scale(0.3) rotate(-10deg);
    opacity: 0;
  }
  50% {
    transform: scale(1.05) rotate(5deg);
  }
  100% {
    transform: scale(1) rotate(0deg);
    opacity: 1;
  }
}

:deep(.champion-celebration-box) {
  animation: champion-bounce 0.8s cubic-bezier(0.68, -0.55, 0.265, 1.55);
  background: linear-gradient(135deg, #fef3c7 0%, #fde047 100%);
  border: 3px solid #fbbf24;

  .el-message-box__title {
    font-size: 28px;
    font-weight: 900;
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .el-message-box__content {
    font-size: 18px;
    color: #92400e;
  }

  .el-button--primary {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    border: none;

    &:hover {
      background: linear-gradient(135deg, #d97706 0%, #b45309 100%);
    }
  }
}
</style>
