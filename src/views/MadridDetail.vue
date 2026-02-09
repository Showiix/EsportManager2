<template>
  <div class="madrid-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div>
        <h1>马德里大师赛</h1>
        <p>32支队伍 · 8个小组BO3单循环 · 东西半区BO5淘汰赛</p>
      </div>
      <div class="header-actions">
        <el-button v-if="madridBracket.status === 'group_stage' && !isGroupStageComplete" type="primary" size="small" @click="batchSimulateGroupStage" :loading="simulatingGroupStage">
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingGroupStage ? `模拟中 (${groupSimProgress}%)` : '模拟小组赛' }}
        </el-button>
        <el-button v-if="madridBracket.status === 'knockout_stage'" type="primary" size="small" @click="batchSimulateKnockout" :loading="simulatingKnockout">
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingKnockout ? `模拟中 (${simulationProgress}%)` : '模拟淘汰赛' }}
        </el-button>
        <button class="back-btn" @click="goBack">&larr; 返回赛事列表</button>
      </div>
    </div>

    <!-- 阶段未到提示 -->
    <el-alert
      v-if="phaseNotReached"
      title="赛事尚未开始"
      type="warning"
      :closable="false"
      show-icon
      style="margin-bottom: 16px;"
    >
      <template #default>
        <div class="phase-warning-content">
          <p>当前赛季阶段：<strong>{{ currentPhaseDisplay }}</strong></p>
          <p>马德里大师赛需要在 <strong>MSI季中赛</strong> 结束后才会开始。</p>
          <p>请先完成之前的赛事阶段，然后在时间控制面板推进到马德里大师赛阶段。</p>
        </div>
      </template>
    </el-alert>

    <!-- 马德里大师赛状态 -->
    <div class="filter-section">
      <div class="filter-row">
        <span style="font-weight: 600; color: #0f172a;">S{{ viewingSeason }} 马德里大师赛</span>
        <el-tag :type="getStatusType(madridBracket.status)" size="small">{{ getStatusText(madridBracket.status) }}</el-tag>
      </div>
    </div>

    <!-- 参赛队伍统计 -->
    <div class="stats-bar">
      <div class="stat-item"><span class="stat-value">32</span><span class="stat-label">参赛队伍</span></div>
      <div class="stat-divider"></div>
      <div class="stat-item"><span class="stat-value">8</span><span class="stat-label">小组</span></div>
      <div class="stat-divider"></div>
      <div class="stat-item"><span class="stat-value">16</span><span class="stat-label">东半区</span></div>
      <div class="stat-divider"></div>
      <div class="stat-item"><span class="stat-value">16</span><span class="stat-label">西半区</span></div>
    </div>

    <!-- 小组赛阶段 -->
    <div v-if="madridBracket.status !== 'not_started'" class="table-section">
      <div class="section-header">
        <span class="section-title">小组赛阶段</span>
        <el-tag v-if="isGroupStageComplete" type="success" size="small">已完成</el-tag>
        <el-tag v-else type="warning" size="small">进行中</el-tag>
      </div>
      <div class="groups-grid">
        <ClauchGroupStanding
          v-for="group in madridBracket.groups"
          :key="group.groupName"
          :group="group"
          @simulate-match="handleSimulateMatch"
          @view-detail="viewMatchDetails"
        />
      </div>
      <!-- 生成淘汰赛按钮 -->
      <div v-if="isGroupStageComplete && madridBracket.status === 'group_stage'" style="padding: 16px; text-align: center;">
        <el-button type="primary" size="small" @click="handleGenerateKnockout" :loading="generatingKnockout">
          <el-icon><Plus /></el-icon> 生成淘汰赛对阵
        </el-button>
      </div>
    </div>

    <!-- 淘汰赛阶段 -->
    <div v-if="madridBracket.status === 'knockout_stage' || madridBracket.status === 'completed'" class="table-section">
      <div class="section-header">
        <span class="section-title">淘汰赛阶段</span>
        <el-tag v-if="madridBracket.status === 'completed'" type="success" size="small">已完成</el-tag>
        <el-tag v-else type="warning" size="small">进行中</el-tag>
      </div>
      <div class="knockout-content">
        <div class="knockout-brackets">
          <div class="bracket-half">
            <div class="section-label">东半区</div>
            <ClauchKnockoutBracket
              v-if="madridBracket.knockoutEast"
              :knockout="madridBracket.knockoutEast"
              bracket="east"
              @simulate-match="handleSimulateMatch"
              @view-detail="viewMatchDetails"
            />
          </div>
          <div class="bracket-half">
            <div class="section-label">西半区</div>
            <ClauchKnockoutBracket
              v-if="madridBracket.knockoutWest"
              :knockout="madridBracket.knockoutWest"
              bracket="west"
              @simulate-match="handleSimulateMatch"
              @view-detail="viewMatchDetails"
            />
          </div>
        </div>
        <!-- Finals section -->
        <div v-if="showFinals" class="finals-content">
          <div class="section-label finals">决赛阶段</div>
          <div class="finals-matches">
            <div v-if="madridBracket.thirdPlaceMatch" class="final-match-block">
              <div class="match-label">季军赛</div>
              <ClauchMatchCard :match="madridBracket.thirdPlaceMatch" @simulate="handleSimulateMatch" @view-detail="viewMatchDetails" />
            </div>
            <div v-if="madridBracket.grandFinal" class="final-match-block">
              <div class="match-label">总决赛</div>
              <ClauchMatchCard :match="madridBracket.grandFinal" @simulate="handleSimulateMatch" @view-detail="viewMatchDetails" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <TournamentCompletionSection
      v-if="madridBracket.status === 'completed'"
      :standings="madridStandings"
      banner-title="马德里大师赛已完成！"
      :banner-champion="madridBracket.champion?.teamName || ''"
      banner-description="获得马德里大师赛冠军！"
    />

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
  DArrowRight,
  Plus
} from '@element-plus/icons-vue'
import ClauchGroupStanding from '@/components/clauch/ClauchGroupStanding.vue'
import ClauchKnockoutBracket from '@/components/clauch/ClauchKnockoutBracket.vue'
import ClauchMatchCard from '@/components/clauch/ClauchMatchCard.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import TournamentCompletionSection from '@/components/common/TournamentCompletionSection.vue'
import type { StandingItem } from '@/types/tournament'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { useGameStore } from '@/stores/useGameStore'
import { internationalApi, matchApi } from '@/api/tauri'
import type { BracketInfo, MatchBracketInfo, GroupStandingInfo } from '@/api/tauri'
import type { ClauchMatch, ClauchGroup, ClauchGroupStanding as ClauchGroupStandingType, ClauchKnockoutBracket as ClauchKnockoutBracketType } from '@/types/clauch'
import type { MatchDetail } from '@/types/matchDetail'
import { createLogger } from '@/utils/logger'
import { useBatchSimulation, buildMatchDetail, recordMatchPerformances } from '@/composables/useBatchSimulation'

const logger = createLogger('MadridDetail')

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const timeStore = useTimeStore()
const gameStore = useGameStore()

const viewingSeason = computed(() => Number(route.query.season) || gameStore.gameState?.current_season || 1)

// 阶段检查
const MADRID_PHASE = 'MadridMasters'
const phaseNotReached = computed(() => {
  const currentPhase = timeStore.currentPhase
  // 如果已经有比赛数据，说明赛事已初始化，不显示警告
  if (madridBracket.groups.length > 0 || (bracketData.value?.matches?.length ?? 0) > 0) {
    return false
  }
  const phaseOrder = [
    'SpringRegular', 'SpringPlayoffs', 'Msi', 'MadridMasters',
    'SummerRegular', 'SummerPlayoffs', 'ClaudeIntercontinental',
    'WorldChampionship', 'ShanghaiMasters', 'IcpIntercontinental',
    'SuperIntercontinental', 'TransferWindow', 'Draft', 'SeasonEnd'
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
const simulatingKnockout = ref(false)
const simulationProgress = ref(0)

const { simulationProgress: groupSimProgress, isSimulating: simulatingGroupStage, batchSimulate } = useBatchSimulation()

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

const madridStandings = computed<StandingItem[]>(() => [
  { rank: 1, label: '冠军', name: madridBracket.champion?.teamName || '', regionName: madridBracket.champion?.regionName, points: '+20分' },
  { rank: 2, label: '亚军', name: madridBracket.runnerUp?.teamName || '', regionName: madridBracket.runnerUp?.regionName, points: '+16分' },
  { rank: 3, label: '季军', name: madridBracket.thirdPlace?.teamName || '', regionName: madridBracket.thirdPlace?.regionName, points: '+12分' },
  { rank: 4, label: '殿军', name: madridBracket.fourthPlace?.teamName || '', regionName: madridBracket.fourthPlace?.regionName, points: '+8分' },
])

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

    // 保存比赛详情到 Store (用于展示)
    const matchDetail = buildMatchDetail({
      matchId: match.id,
      tournamentType: 'madrid',
      seasonId: String(madridBracket.seasonYear),
      teamAId: String(match.teamAId || ''),
      teamAName: match.teamAName || '',
      teamBId: String(match.teamBId || ''),
      teamBName: match.teamBName || '',
      bestOf: match.bestOf || 3,
      result
    })
    await matchDetailStore.saveMatchDetail(match.id, matchDetail)

    // 同时用数据库 ID 保存一份，确保能从数据库加载
    if (match.backendMatchId) {
      const dbMatchDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
      await matchDetailStore.saveMatchDetail(match.backendMatchId, dbMatchDetail)
    }

    // 记录选手表现到统计（国际赛事使用 INTL 标识）
    recordMatchPerformances(matchDetail, String(madridBracket.seasonYear), 'INTL', playerStore)
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
 * 批量模拟小组赛 - 使用 useBatchSimulation composable
 */
const batchSimulateGroupStage = async () => {
  const groupMatches = madridBracket.groups.flatMap(g => g.matches)
  const uncompleted = groupMatches.filter(m => m.status !== 'completed')

  logger.debug('[batchSimulateGroupStage] uncompleted:', uncompleted.length)

  await batchSimulate({
    confirmMessage: '将自动模拟所有未完成的小组赛比赛。是否继续?',
    confirmTitle: '模拟小组赛确认',
    confirmType: 'info',
    successMessage: '小组赛模拟完成！现在可以生成淘汰赛对阵。',
    errorPrefix: '小组赛模拟失败',
    tournamentType: 'madrid',
    seasonId: String(madridBracket.seasonYear),
    competitionType: 'INTL',
    delayMs: 50,
    matches: uncompleted.map(m => ({
      matchId: Number(m.id),
      teamAId: String(m.teamAId || ''),
      teamAName: m.teamAName || '',
      teamBId: String(m.teamBId || ''),
      teamBName: m.teamBName || '',
      bestOf: m.bestOf || 3,
      frontendMatchId: m.id,
      backendMatchId: m.backendMatchId ? Number(m.backendMatchId) : undefined
    })),
    onComplete: async () => {
      await loadTournamentData()
    }
  })
}

/**
 * 批量模拟淘汰赛 - 使用 buildMatchDetail / recordMatchPerformances
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

          const teamAId = String(match.home_team?.id || '')
          const teamAName = match.home_team?.name || ''
          const teamBId = String(match.away_team?.id || '')
          const teamBName = match.away_team?.name || ''
          const bestOf = match.format === 'Bo5' ? 5 : match.format === 'Bo3' ? 3 : 1

          const matchDetail = buildMatchDetail({
            matchId: String(match.match_id),
            tournamentType: 'madrid',
            seasonId: String(madridBracket.seasonYear),
            teamAId, teamAName, teamBId, teamBName, bestOf,
            result
          })
          await matchDetailStore.saveMatchDetail(String(match.match_id), matchDetail)

          recordMatchPerformances(matchDetail, String(madridBracket.seasonYear), 'INTL', playerStore)

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

<style scoped>
.madrid-management {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.page-header h1 {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.page-header p {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.back-btn {
  background: none;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 5px 12px;
  font-size: 13px;
  color: #475569;
  cursor: pointer;
  transition: all 0.15s;
}

.back-btn:hover {
  border-color: #94a3b8;
  color: #0f172a;
}

.filter-section {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stats-bar {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 16px 20px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
}

.stat-label {
  font-size: 12px;
  color: #64748b;
}

.stat-divider {
  width: 1px;
  height: 32px;
  background: #e2e8f0;
}

.table-section {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e2e8f0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.groups-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  padding: 16px;
}

.knockout-content {
  padding: 16px;
}

.section-label {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 12px;
  color: #0f172a;
}

.section-label.finals {
  text-align: center;
  font-size: 14px;
}

.knockout-brackets {
  display: flex;
  gap: 16px;
}

.bracket-half {
  flex: 1;
  overflow-x: auto;
}

.finals-content {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #e2e8f0;
}

.finals-matches {
  display: flex;
  gap: 24px;
  justify-content: center;
}

.final-match-block {
  text-align: center;
}

.match-label {
  font-size: 13px;
  font-weight: 600;
  color: #64748b;
  margin-bottom: 8px;
}
</style>
