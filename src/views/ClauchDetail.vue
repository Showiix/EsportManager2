<template>
  <div class="clauch-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="goBack">&larr; 返回赛事列表</button>
        <h1 class="page-title">C洲际赛 (Clauch Intercontinental Cup)</h1>
        <p class="page-desc">32支队伍（各赛区夏季赛常规赛前8名），8个小组BO3单循环，东西半区BO5淘汰赛</p>
      </div>
      <div class="header-actions">
        <button
          v-if="clauchBracket.status === 'group_stage' && !isGroupStageComplete"
          class="action-btn primary-btn"
          @click="batchSimulateGroupStage"
          :disabled="simulatingGroupStage"
        >
          {{ simulatingGroupStage ? `模拟中 (${groupSimProgress}%)` : '模拟小组赛' }}
        </button>
        <button
          v-if="clauchBracket.status === 'knockout_stage'"
          class="action-btn warning-btn"
          @click="batchSimulateKnockout"
          :disabled="simulatingKnockout"
        >
          {{ simulatingKnockout ? `模拟中 (${simulationProgress}%)` : '模拟淘汰赛' }}
        </button>
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
          <p>C洲际赛需要在 <strong>夏季季后赛</strong> 结束后才会开始。</p>
          <p>请先完成之前的赛事阶段，然后在时间控制面板推进到C洲际赛阶段。</p>
        </div>
      </template>
    </el-alert>

    <!-- C洲际赛状态卡片 -->
    <div class="clauch-status-card">
      <div class="status-header">
        <h2 class="status-title">S{{ viewingSeason }} C洲际赛</h2>
        <span class="status-badge" :class="getStatusType(clauchBracket.status)">
          {{ getStatusText(clauchBracket.status) }}
        </span>
      </div>

      <!-- 参赛队伍统计 -->
      <div class="stats-bar">
        <div class="stat-item">
          <span class="stat-value">32</span>
          <span class="stat-label">参赛队伍总数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">8</span>
          <span class="stat-label">小组数量</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">16</span>
          <span class="stat-label">东半区队伍</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">16</span>
          <span class="stat-label">西半区队伍</span>
        </div>
      </div>

      <!-- 赛事数据异常警告 -->
      <el-alert
        v-if="clauchBracket.status !== 'not_started' && clauchBracket.groups.length === 0"
        title="赛事数据异常"
        type="error"
        :closable="false"
        show-icon
        class="data-error-alert"
      >
        <template #default>
          <div class="error-content">
            <p>赛事已创建但没有生成比赛数据。这通常是因为：</p>
            <ul>
              <li>夏季常规赛尚未完成，积分榜数据不完整</li>
              <li>无法从积分榜获取足够的队伍（需要32支队伍）</li>
            </ul>
            <p><strong>解决方案</strong>：请返回并确保夏季常规赛已经全部完成，然后重新推进到C洲际赛阶段。</p>
          </div>
        </template>
      </el-alert>

      <!-- 小组赛阶段 -->
      <div v-if="clauchBracket.status !== 'not_started'" class="table-section">
        <div class="section-header">
          <span class="section-title">小组赛阶段</span>
          <span v-if="isGroupStageComplete" class="status-badge success">已完成</span>
          <span v-else class="status-badge warning">进行中</span>
        </div>

        <!-- 小组赛积分榜 -->
        <div class="groups-grid">
          <ClauchGroupStanding
            v-for="group in clauchBracket.groups"
            :key="group.groupName"
            :group="group"
            @simulate-match="handleSimulateMatch"
            @view-detail="handleViewMatchDetail"
          />
        </div>

        <!-- 生成淘汰赛按钮 -->
        <div v-if="isGroupStageComplete && clauchBracket.status === 'group_stage'" class="generate-knockout-section">
          <el-alert
            title="小组赛已完成！"
            description="所有小组赛比赛已完成，各小组前2名已晋级。现在可以生成淘汰赛对阵。"
            type="success"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <button class="action-btn primary-btn" @click="handleGenerateKnockout" :disabled="generatingKnockout">
            {{ generatingKnockout ? '生成中...' : '生成淘汰赛对阵' }}
          </button>
        </div>
      </div>

      <!-- 淘汰赛阶段 -->
      <div v-if="clauchBracket.status === 'knockout_stage' || clauchBracket.status === 'completed'" class="table-section">
        <div class="section-header">
          <span class="section-title">淘汰赛阶段</span>
          <span v-if="clauchBracket.status === 'completed'" class="status-badge success">已完成</span>
          <span v-else class="status-badge warning">进行中</span>
        </div>

        <!-- 淘汰赛对阵图 -->
        <div class="knockout-content">
          <div class="knockout-brackets">
            <div class="bracket-half">
              <div class="section-label">东半区</div>
              <ClauchKnockoutBracket
                v-if="clauchBracket.knockoutEast"
                :knockout="clauchBracket.knockoutEast"
                bracket="east"
                @simulate-match="handleSimulateMatch"
                @view-detail="handleViewMatchDetail"
              />
            </div>
            <div class="bracket-half">
              <div class="section-label">西半区</div>
              <ClauchKnockoutBracket
                v-if="clauchBracket.knockoutWest"
                :knockout="clauchBracket.knockoutWest"
                bracket="west"
                @simulate-match="handleSimulateMatch"
                @view-detail="handleViewMatchDetail"
              />
            </div>
          </div>
          <div v-if="showFinals" class="finals-content">
            <div class="section-label finals">决赛阶段</div>
            <div class="finals-matches">
              <div v-if="clauchBracket.thirdPlaceMatch" class="final-match-block">
                <div class="match-label">季军赛</div>
                <ClauchMatchCard :match="clauchBracket.thirdPlaceMatch" @simulate="handleSimulateMatch" @view-detail="handleViewMatchDetail" />
              </div>
              <div v-if="clauchBracket.grandFinal" class="final-match-block">
                <div class="match-label">总决赛</div>
                <ClauchMatchCard :match="clauchBracket.grandFinal" @simulate="handleSimulateMatch" @view-detail="handleViewMatchDetail" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <TournamentCompletionSection
        v-if="clauchBracket.status === 'completed'"
        :standings="clauchStandings"
        banner-title="C洲际赛已完成！"
        :banner-champion="clauchBracket.champion?.teamName || ''"
        banner-description="获得C洲际赛冠军！"
      />
    </div>

    <!-- 比赛详情弹窗 -->
    <MatchDetailDialog
      :visible="showMatchDetailDialog"
      :match-detail="currentMatchDetail"
      @update:visible="showMatchDetailDialog = $event"
      @close="handleCloseMatchDetail"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
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
import type { MatchDetail } from '@/types/matchDetail'
import type { ClauchMatch, ClauchGroup, ClauchGroupStanding as ClauchGroupStandingType, ClauchKnockoutBracket as ClauchKnockoutBracketType } from '@/types/clauch'
import { useBatchSimulation, buildMatchDetail, recordMatchPerformances } from '@/composables/useBatchSimulation'
import { createLogger } from '@/utils/logger'

const logger = createLogger('ClauchDetail')

const router = useRouter()
const route = useRoute()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const timeStore = useTimeStore()
const gameStore = useGameStore()

// 从 query 获取赛季（赛事管理页传入），否则使用当前赛季
const viewingSeason = computed(() => Number(route.query.season) || gameStore.gameState?.current_season || 1)

// 阶段检查
const CLAUCH_PHASE = 'ClaudeIntercontinental'
const phaseNotReached = computed(() => {
  const currentPhase = timeStore.currentPhase
  // 如果已经有比赛数据，说明赛事已初始化，不显示警告
  if (clauchBracket.groups.length > 0 || (bracketData.value?.matches?.length ?? 0) > 0) {
    return false
  }
  const phaseOrder = [
    'SpringRegular', 'SpringPlayoffs', 'Msi', 'MadridMasters',
    'SummerRegular', 'SummerPlayoffs', 'ClaudeIntercontinental',
    'WorldChampionship', 'ShanghaiMasters', 'IcpIntercontinental',
    'SuperIntercontinental', 'TransferWindow', 'Draft', 'SeasonEnd'
  ]
  const currentIndex = phaseOrder.indexOf(currentPhase)
  const targetIndex = phaseOrder.indexOf(CLAUCH_PHASE)
  // 只有当前阶段早于目标阶段时才显示警告
  return currentIndex >= 0 && currentIndex < targetIndex
})

const currentPhaseDisplay = computed(() => timeStore.phaseDisplayName)

// 后端数据状态
const tournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)
const groupStandings = ref<GroupStandingInfo[]>([])
const loading = ref(false)

// 响应式状态
const generatingKnockout = ref(false)
const simulatingKnockout = ref(false)
const simulationProgress = ref(0)

const { simulationProgress: groupSimProgress, isSimulating: simulatingGroupStage, batchSimulate } = useBatchSimulation()

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// C洲际赛数据 - 从后端获取并转换
const clauchBracket = reactive({
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

// 计算属性
const isGroupStageComplete = computed(() => {
  return clauchBracket.groups.every(group => {
    return group.matches.every(match => match.status === 'completed')
  })
})

const showFinals = computed(() => {
  return clauchBracket.thirdPlaceMatch || clauchBracket.grandFinal
})

const clauchStandings = computed<StandingItem[]>(() => [
  { rank: 1, label: '冠军', name: clauchBracket.champion?.teamName || '', regionName: clauchBracket.champion?.regionName, points: '+20分' },
  { rank: 2, label: '亚军', name: clauchBracket.runnerUp?.teamName || '', regionName: clauchBracket.runnerUp?.regionName, points: '+16分' },
  { rank: 3, label: '季军', name: clauchBracket.thirdPlace?.teamName || '', regionName: clauchBracket.thirdPlace?.regionName, points: '+12分' },
  { rank: 4, label: '殿军', name: clauchBracket.fourthPlace?.teamName || '', regionName: clauchBracket.fourthPlace?.regionName, points: '+8分' },
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

    const matchDetail = buildMatchDetail({
      matchId: match.backendMatchId || match.id,
      tournamentType: 'clauch',
      seasonId: String(clauchBracket.seasonYear),
      teamAId: String(match.teamAId || ''),
      teamAName: match.teamAName || '',
      teamBId: String(match.teamBId || ''),
      teamBName: match.teamBName || '',
      bestOf: match.bestOf || 3,
      result
    })
    matchDetailStore.saveMatchDetail(match.id, matchDetail)

    recordMatchPerformances(matchDetail, String(clauchBracket.seasonYear), 'INTL', playerStore)
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
    if (clauchBracket.status === 'knockout_stage') {
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
  if (clauchBracket.grandFinal?.status === 'completed') {
    // 设置最终排名
    const grandFinal = clauchBracket.grandFinal
    const thirdPlaceMatch = clauchBracket.thirdPlaceMatch

    if (grandFinal.winnerId === grandFinal.teamAId) {
      clauchBracket.champion = { teamName: grandFinal.teamAName || '', regionName: 'LPL' }
      clauchBracket.runnerUp = { teamName: grandFinal.teamBName || '', regionName: 'LCK' }
    } else {
      clauchBracket.champion = { teamName: grandFinal.teamBName || '', regionName: 'LCK' }
      clauchBracket.runnerUp = { teamName: grandFinal.teamAName || '', regionName: 'LPL' }
    }

    if (thirdPlaceMatch && thirdPlaceMatch.winnerId === thirdPlaceMatch.teamAId) {
      clauchBracket.thirdPlace = { teamName: thirdPlaceMatch.teamAName || '', regionName: 'LEC' }
      clauchBracket.fourthPlace = { teamName: thirdPlaceMatch.teamBName || '', regionName: 'LCS' }
    } else if (thirdPlaceMatch) {
      clauchBracket.thirdPlace = { teamName: thirdPlaceMatch.teamBName || '', regionName: 'LCS' }
      clauchBracket.fourthPlace = { teamName: thirdPlaceMatch.teamAName || '', regionName: 'LEC' }
    }

    clauchBracket.status = 'completed'
    showChampionCelebration(clauchBracket.champion?.teamName || '')
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
  const groupMatches = clauchBracket.groups.flatMap(g => g.matches)
  const uncompleted = groupMatches.filter(m => m.status !== 'completed')

  logger.debug('[batchSimulateGroupStage] uncompleted:', uncompleted.length)

  await batchSimulate({
    confirmMessage: '将自动模拟所有未完成的小组赛比赛。是否继续?',
    confirmTitle: '模拟小组赛确认',
    confirmType: 'info',
    successMessage: '小组赛模拟完成！现在可以生成淘汰赛对阵。',
    errorPrefix: '小组赛模拟失败',
    tournamentType: 'clauch',
    seasonId: String(clauchBracket.seasonYear),
    competitionType: 'INTL',
    delayMs: 50,
    matches: uncompleted.map(m => ({
      matchId: Number(m.id),
      teamAId: String(m.teamAId || ''),
      teamAName: m.teamAName || '',
      teamBId: String(m.teamBId || ''),
      teamBName: m.teamBName || '',
      bestOf: m.bestOf || 3,
      frontendMatchId: m.id
    })),
    onComplete: async () => {
      await loadTournamentData()
    }
  })
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
            tournamentType: 'clauch',
            seasonId: String(clauchBracket.seasonYear),
            teamAId, teamAName, teamBId, teamBName, bestOf,
            result
          })
          await matchDetailStore.saveMatchDetail(String(match.match_id), matchDetail)

          recordMatchPerformances(matchDetail, String(clauchBracket.seasonYear), 'INTL', playerStore)

          if (tournamentId.value) {
            await internationalApi.advanceBracket(tournamentId.value, match.match_id, result.winner_id)
          }
        } catch (e) {
          logger.error(`模拟比赛 ${match.match_id} 失败:`, e)
        }

        await new Promise(resolve => setTimeout(resolve, 100))
      }

      await loadTournamentData()
    }

    playerStore.saveToStorage()

    clauchBracket.status = 'completed'
    ElMessage.success('淘汰赛模拟完成！')

    if (clauchBracket.champion) {
      showChampionCelebration(clauchBracket.champion.teamName)
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
 * 处理查看比赛详情
 */
const handleViewMatchDetail = async (matchId: string | number) => {
  // 先尝试从内存获取
  let detail = matchDetailStore.getMatchDetail(matchId)
  if (detail) {
    currentMatchDetail.value = detail
    showMatchDetailDialog.value = true
    return
  }

  // 如果内存中没有，尝试从数据库加载
  detail = await matchDetailStore.loadMatchDetailFromDb(matchId)
  if (detail) {
    currentMatchDetail.value = detail
    showMatchDetailDialog.value = true
    return
  }

  ElMessage.warning('暂无比赛详情数据')
}

/**
 * 关闭比赛详情弹窗
 */
const handleCloseMatchDetail = () => {
  showMatchDetailDialog.value = false
  currentMatchDetail.value = null
}

/**
 * 显示冠军庆祝动画
 */
const showChampionCelebration = (championName: string) => {
  ElMessageBox.alert(
    `恭喜 ${championName} 获得C洲际赛冠军，成为全球最强战队!`,
    '🏆 C洲际赛冠军诞生! 🏆',
    {
      confirmButtonText: '太棒了!',
      customClass: 'champion-celebration-box',
      showClose: false,
      center: true
    }
  )
}

/**
 * 从后端加载赛事数据
 */
const loadTournamentData = async () => {
  if (!tournamentId.value) return

  try {
    // 获取对阵数据
    bracketData.value = await internationalApi.getTournamentBracket(tournamentId.value)
    logger.debug('[ClauchDetail] 后端返回 bracketData:', bracketData.value)
    logger.debug('[ClauchDetail] 比赛数量:', bracketData.value?.matches?.length || 0)
    logger.debug('[ClauchDetail] 阶段数量:', bracketData.value?.stages?.length || 0)

    // 获取小组赛积分榜
    groupStandings.value = await internationalApi.getGroupStandings(tournamentId.value)
    logger.debug('[ClauchDetail] 小组积分榜:', groupStandings.value)

    // 转换数据格式适配前端组件
    convertBracketToClauchFormat()

    logger.debug('[ClauchDetail] 转换后 clauchBracket.status:', clauchBracket.status)
    logger.debug('[ClauchDetail] 转换后 clauchBracket.groups:', clauchBracket.groups)
    logger.debug('[ClauchDetail] isGroupStageComplete:', isGroupStageComplete.value)
  } catch (error) {
    logger.error('加载赛事数据失败:', error)
    throw error
  }
}

/**
 * 将后端数据转换为前端组件需要的格式
 */
const convertBracketToClauchFormat = () => {
  if (!bracketData.value) return

  clauchBracket.id = String(bracketData.value.tournament_id)

  // 确定赛事状态
  const stages = bracketData.value.stages
  const hasGroupStage = stages.some(s => s.name.startsWith('GROUP_'))
  const hasKnockout = stages.some(s => s.name.startsWith('EAST_') || s.name.startsWith('WEST_'))
  const grandFinalMatch = bracketData.value.matches.find(m => m.stage === 'GRAND_FINAL')

  // 辅助函数：检查比赛状态是否为已完成（兼容大小写）
  const isMatchCompleted = (status: string) => status === 'Completed' || status === 'COMPLETED'

  if (grandFinalMatch?.status && isMatchCompleted(grandFinalMatch.status)) {
    clauchBracket.status = 'completed'
  } else if (hasKnockout && bracketData.value.matches.some(m => m.stage.startsWith('EAST_') || m.stage.startsWith('WEST_'))) {
    const groupMatches = bracketData.value.matches.filter(m => m.stage.startsWith('GROUP_'))
    const allGroupComplete = groupMatches.every(m => m.status && isMatchCompleted(m.status))

    // 检查淘汰赛比赛是否已经分配队伍
    // 如果淘汰赛比赛存在但队伍为 null，说明还需要点击"生成淘汰赛"按钮
    const knockoutMatches = bracketData.value.matches.filter(m =>
      m.stage.startsWith('EAST_') || m.stage.startsWith('WEST_') ||
      m.stage === 'THIRD_PLACE' || m.stage === 'GRAND_FINAL'
    )
    const knockoutHasTeams = knockoutMatches.some(m => m.home_team !== null && m.away_team !== null)

    logger.debug('[convertBracketToClauchFormat] allGroupComplete:', allGroupComplete)
    logger.debug('[convertBracketToClauchFormat] knockoutHasTeams:', knockoutHasTeams)
    logger.debug('[convertBracketToClauchFormat] knockoutMatches sample:', knockoutMatches.slice(0, 2))

    // 只有当淘汰赛比赛已经分配了队伍时，才进入 knockout_stage
    if (allGroupComplete && knockoutHasTeams) {
      clauchBracket.status = 'knockout_stage'
    } else {
      clauchBracket.status = 'group_stage'
    }
  } else if (hasGroupStage) {
    clauchBracket.status = 'group_stage'
  }

  // 转换小组赛数据
  clauchBracket.groups = convertGroupsData()

  // 转换淘汰赛数据
  if (clauchBracket.status === 'knockout_stage' || clauchBracket.status === 'completed') {
    convertKnockoutData()
  }

  // 设置最终排名
  if (clauchBracket.status === 'completed') {
    setFinalStandings()
  }
}

/**
 * 转换小组赛数据
 */
const convertGroupsData = (): ClauchGroup[] => {
  if (!bracketData.value) return clauchBracket.groups

  const groups: ClauchGroup[] = []
  const groupNames = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H']

  for (const groupName of groupNames) {
    const stageName = `GROUP_${groupName}`
    const groupMatches = bracketData.value.matches.filter(m => m.stage === stageName)

    if (groupMatches.length === 0) continue

    const standingInfo = groupStandings.value.find(g => g.group_name === groupName)

    const matches: ClauchMatch[] = groupMatches.map(m => ({
      id: String(m.match_id),
      teamAId: String(m.home_team?.id || ''),
      teamAName: m.home_team?.name || '待定',
      teamBId: String(m.away_team?.id || ''),
      teamBName: m.away_team?.name || '待定',
      scoreA: m.home_score,
      scoreB: m.away_score,
      winnerId: m.winner_id ? String(m.winner_id) : null,
      status: (m.status === 'Completed' || m.status === 'COMPLETED') ? 'completed' : 'scheduled',
      bestOf: m.format === 'Bo3' ? 3 : m.format === 'Bo5' ? 5 : 1,
      stage: 'group',
      groupName: groupName,
      roundNumber: m.match_order
    }))

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

  return groups.length > 0 ? groups : clauchBracket.groups
}

/**
 * 转换淘汰赛数据
 */
const convertKnockoutData = () => {
  if (!bracketData.value) return

  const matches = bracketData.value.matches

  const convertMatchFormat = (m: MatchBracketInfo, matchType: string): ClauchMatch => ({
    id: String(m.match_id),
    teamAId: String(m.home_team?.id || ''),
    teamAName: m.home_team?.name || '待定',
    teamBId: String(m.away_team?.id || ''),
    teamBName: m.away_team?.name || '待定',
    scoreA: m.home_score,
    scoreB: m.away_score,
    winnerId: m.winner_id ? String(m.winner_id) : null,
    status: (m.status === 'Completed' || m.status === 'COMPLETED') ? 'completed' : 'scheduled',
    bestOf: m.format === 'Bo3' ? 3 : m.format === 'Bo5' ? 5 : 1,
    matchType
  })

  // 东半区
  const eastR1 = matches.filter(m => m.stage === 'EAST_R1').sort((a, b) => a.match_order - b.match_order)
  const eastSemi = matches.filter(m => m.stage === 'EAST_SEMI').sort((a, b) => a.match_order - b.match_order)
  const eastFinal = matches.filter(m => m.stage === 'EAST_FINAL')

  if (eastR1.length > 0) {
    clauchBracket.knockoutEast = {
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
    clauchBracket.knockoutWest = {
      round1: westR1.map(m => convertMatchFormat(m, 'west_quarter')),
      semiFinals: westSemi.map(m => convertMatchFormat(m, 'west_semi')),
      final: westFinal.map(m => convertMatchFormat(m, 'west_final'))
    }
  }

  // 季军赛
  const thirdPlace = matches.find(m => m.stage === 'THIRD_PLACE')
  if (thirdPlace) {
    clauchBracket.thirdPlaceMatch = convertMatchFormat(thirdPlace, 'third_place')
  }

  // 总决赛
  const grandFinal = matches.find(m => m.stage === 'GRAND_FINAL')
  if (grandFinal) {
    clauchBracket.grandFinal = convertMatchFormat(grandFinal, 'grand_final')
  }
}

/**
 * 设置最终排名
 */
const setFinalStandings = () => {
  const gf = clauchBracket.grandFinal
  const tp = clauchBracket.thirdPlaceMatch

  if (gf && gf.winnerId) {
    if (gf.winnerId === gf.teamAId) {
      clauchBracket.champion = { teamName: gf.teamAName || '', regionName: '' }
      clauchBracket.runnerUp = { teamName: gf.teamBName || '', regionName: '' }
    } else {
      clauchBracket.champion = { teamName: gf.teamBName || '', regionName: '' }
      clauchBracket.runnerUp = { teamName: gf.teamAName || '', regionName: '' }
    }
  }

  if (tp && tp.winnerId) {
    if (tp.winnerId === tp.teamAId) {
      clauchBracket.thirdPlace = { teamName: tp.teamAName || '', regionName: '' }
      clauchBracket.fourthPlace = { teamName: tp.teamBName || '', regionName: '' }
    } else {
      clauchBracket.thirdPlace = { teamName: tp.teamBName || '', regionName: '' }
      clauchBracket.fourthPlace = { teamName: tp.teamAName || '', regionName: '' }
    }
  }
}

// 初始化：从路由参数获取赛事ID或按类型查找
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
      // 如果没有ID，尝试按类型查找赛事
      const currentSave = gameStore.currentSave
      if (currentSave) {
        const seasonId = viewingSeason.value
        // 获取 Claude 洲际赛 (类型为 'Clauch')
        const tournaments = await internationalApi.getTournamentsByType('Clauch', seasonId)
        if (tournaments && tournaments.length > 0) {
          tournamentId.value = tournaments[0].id
          await loadTournamentData()
        } else {
          logger.warn('未找到 Clauch 赛事')
        }
      } else {
        logger.warn('未找到当前存档')
      }
    }
  } catch (error) {
    logger.error('初始化失败:', error)
    // 如果后端加载失败，继续使用 mock 数据
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.clauch-management {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.back-btn {
  background: none;
  border: none;
  color: #6366f1;
  font-size: 13px;
  cursor: pointer;
  padding: 0;
  text-align: left;
}

.back-btn:hover {
  color: #4f46e5;
}

.page-title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.page-desc {
  margin: 0;
  color: #64748b;
  font-size: 13px;
}

.header-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}

.action-btn {
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  border: none;
  transition: background 0.15s;
}

.primary-btn {
  background: #6366f1;
  color: #ffffff;
}

.primary-btn:hover {
  background: #4f46e5;
}

.primary-btn:disabled {
  background: #c7d2fe;
  cursor: not-allowed;
}

.warning-btn {
  background: #f59e0b;
  color: #ffffff;
}

.warning-btn:hover {
  background: #d97706;
}

.warning-btn:disabled {
  background: #fde68a;
  cursor: not-allowed;
}

.phase-warning-alert {
  margin-bottom: 24px;
}

.data-error-alert {
  margin-bottom: 24px;
}

.clauch-status-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 24px;
}

.status-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding-bottom: 14px;
  border-bottom: 1px solid #f1f5f9;
}

.status-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.status-badge {
  display: inline-block;
  padding: 2px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 10px;
  background: #f1f5f9;
  color: #64748b;
}

.status-badge.success {
  background: #f0fdf4;
  color: #16a34a;
}

.status-badge.warning {
  background: #fffbeb;
  color: #d97706;
}

.status-badge.info {
  background: #f1f5f9;
  color: #64748b;
}

.stats-bar {
  display: flex;
  gap: 0;
  margin-bottom: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
}

.stat-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 14px 12px;
  border-right: 1px solid #e2e8f0;
}

.stat-item:last-child {
  border-right: none;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: #0f172a;
}

.stat-label {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 2px;
}

.table-section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 18px;
  background: #f8fafc;
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

.generate-knockout-section {
  padding: 16px;
  text-align: center;
}

.mb-4 {
  margin-bottom: 16px;
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
