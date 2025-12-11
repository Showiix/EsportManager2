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
            32支队伍（各赛区夏季赛常规赛前8名），8个小组BO3单循环，东西半区BO5淘汰赛
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
                @view-match="viewMatchDetails"
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
              @view-match="viewMatchDetails"
            />
          </div>

          <div class="bracket-section">
            <h3>西半区</h3>
            <ClauchKnockoutBracket
              v-if="madridBracket.knockoutWest"
              :knockout="madridBracket.knockoutWest"
              bracket="west"
              @simulate-match="handleSimulateMatch"
              @view-match="viewMatchDetails"
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
import { ref, computed, reactive } from 'vue'
import { useRouter } from 'vue-router'
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
import { PowerEngine } from '@/engines/PowerEngine'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import type { ClauchMatch, ClauchGroup, ClauchGroupStanding as ClauchGroupStandingType, ClauchKnockoutBracket as ClauchKnockoutBracketType } from '@/types/clauch'
import type { Player, PlayerPosition } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

const router = useRouter()

// 响应式状态
const generatingKnockout = ref(false)
const simulatingGroupStage = ref(false)
const simulatingKnockout = ref(false)
const simulationProgress = ref(0)
const activeGroup = ref('A')

// 生成模拟的小组数据
const generateGroupData = (): ClauchGroup[] => {
  const lplTeams = ['JDG', 'BLG', 'TES', 'WBG', 'LNG', 'EDG', 'FPX', 'RNG']
  const lckTeams = ['T1', 'GEN', 'DK', 'KT', 'HLE', 'DRX', 'NS', 'LSB']
  const lecTeams = ['G2', 'FNC', 'MAD', 'BDS', 'VIT', 'SK', 'XL', 'AST']
  const lcsTeams = ['C9', 'TL', '100T', 'FLY', 'DIG', 'GG', 'TSM', 'EG']

  const groups: ClauchGroup[] = []
  const groupNames = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H']

  groupNames.forEach((name, index) => {
    const teams = [
      { id: `lpl-${index}`, name: lplTeams[index], region: 'LPL' },
      { id: `lck-${index}`, name: lckTeams[index], region: 'LCK' },
      { id: `lec-${index}`, name: lecTeams[index], region: 'LEC' },
      { id: `lcs-${index}`, name: lcsTeams[index], region: 'LCS' }
    ]

    // 生成积分榜
    const standings: ClauchGroupStandingType[] = teams.map((team, i) => ({
      teamId: team.id,
      teamName: team.name,
      position: i + 1,
      matchesPlayed: 3,
      wins: 3 - i,
      losses: i,
      points: (3 - i) * 3,
      roundsWon: (3 - i) * 2,
      roundsLost: i * 2,
      roundDifferential: ((3 - i) - i) * 2,
      qualified: i < 2
    }))

    // 生成小组赛比赛
    const matches: ClauchMatch[] = []
    let matchId = 1

    // 第一轮
    matches.push({
      id: `${name}-${matchId++}`,
      teamAId: teams[0].id,
      teamAName: teams[0].name,
      teamBId: teams[3].id,
      teamBName: teams[3].name,
      scoreA: 2,
      scoreB: 0,
      winnerId: teams[0].id,
      status: 'completed',
      bestOf: 3,
      stage: 'group',
      groupName: name,
      roundNumber: 1
    })
    matches.push({
      id: `${name}-${matchId++}`,
      teamAId: teams[1].id,
      teamAName: teams[1].name,
      teamBId: teams[2].id,
      teamBName: teams[2].name,
      scoreA: 2,
      scoreB: 1,
      winnerId: teams[1].id,
      status: 'completed',
      bestOf: 3,
      stage: 'group',
      groupName: name,
      roundNumber: 1
    })

    // 第二轮
    matches.push({
      id: `${name}-${matchId++}`,
      teamAId: teams[0].id,
      teamAName: teams[0].name,
      teamBId: teams[1].id,
      teamBName: teams[1].name,
      scoreA: 2,
      scoreB: 1,
      winnerId: teams[0].id,
      status: 'completed',
      bestOf: 3,
      stage: 'group',
      groupName: name,
      roundNumber: 2
    })
    matches.push({
      id: `${name}-${matchId++}`,
      teamAId: teams[2].id,
      teamAName: teams[2].name,
      teamBId: teams[3].id,
      teamBName: teams[3].name,
      scoreA: 2,
      scoreB: 0,
      winnerId: teams[2].id,
      status: 'completed',
      bestOf: 3,
      stage: 'group',
      groupName: name,
      roundNumber: 2
    })

    // 第三轮 - 待模拟
    matches.push({
      id: `${name}-${matchId++}`,
      teamAId: teams[0].id,
      teamAName: teams[0].name,
      teamBId: teams[2].id,
      teamBName: teams[2].name,
      scoreA: 0,
      scoreB: 0,
      winnerId: null,
      status: 'scheduled',
      bestOf: 3,
      stage: 'group',
      groupName: name,
      roundNumber: 3
    })
    matches.push({
      id: `${name}-${matchId++}`,
      teamAId: teams[1].id,
      teamAName: teams[1].name,
      teamBId: teams[3].id,
      teamBName: teams[3].name,
      scoreA: 0,
      scoreB: 0,
      winnerId: null,
      status: 'scheduled',
      bestOf: 3,
      stage: 'group',
      groupName: name,
      roundNumber: 3
    })

    groups.push({
      groupName: name,
      standings,
      matches
    })
  })

  return groups
}

// 马德里大师赛数据
const madridBracket = reactive({
  id: '1',
  seasonYear: 2024,
  status: 'group_stage' as 'not_started' | 'group_stage' | 'knockout_stage' | 'completed',
  groups: generateGroupData(),
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
  return madridBracket.groups.every(group => {
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
 * 生成队伍选手数据
 */
const generateTeamPlayers = (teamId: string, teamName: string, regionName: string = 'Unknown'): Player[] => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  return positions.map((pos, idx) => ({
    id: `${teamId}-${pos}`,
    gameId: `Player${idx + 1}`,
    name: `Player${idx + 1}`,
    teamId: teamId,
    teamName: teamName,
    position: pos,
    regionId: regionName,
    regionName: regionName,
    ability: 70 + Math.floor(Math.random() * 25),
    potential: 80 + Math.floor(Math.random() * 15),
    stability: 60 + Math.floor(Math.random() * 35),
    condition: Math.floor(Math.random() * 11) - 5,
    age: 18 + Math.floor(Math.random() * 10),
    tag: Math.random() > 0.7 ? 'GENIUS' : Math.random() > 0.4 ? 'NORMAL' : 'ORDINARY'
  } as Player))
}

/**
 * 查看比赛详情
 * 支持传入 ClauchMatch 对象或者 matchId
 */
const viewMatchDetails = (matchOrId: ClauchMatch | string | number) => {
  // 获取 matchId
  const matchId = typeof matchOrId === 'object' ? matchOrId.id : matchOrId

  // 尝试从 store 获取详情
  const detail = matchDetailStore.getMatchDetail(matchId)
  if (detail) {
    currentMatchDetail.value = detail
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
 * 模拟单场比赛
 */
const handleSimulateMatch = async (match: ClauchMatch) => {
  // 获取队伍区域信息
  const getTeamRegion = (teamId: string): string => {
    if (teamId.startsWith('lpl')) return 'LPL'
    if (teamId.startsWith('lck')) return 'LCK'
    if (teamId.startsWith('lec')) return 'LEC'
    if (teamId.startsWith('lcs')) return 'LCS'
    return 'Unknown'
  }

  const teamAId = String(match.teamAId || '')
  const teamBId = String(match.teamBId || '')
  const teamARegion = getTeamRegion(teamAId)
  const teamBRegion = getTeamRegion(teamBId)

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(teamAId, match.teamAName || '队伍A', teamARegion)
  const teamBPlayers = generateTeamPlayers(teamBId, match.teamBName || '队伍B', teamBRegion)

  // 使用 PowerEngine 模拟比赛
  const matchDetail = PowerEngine.simulateMatch(
    teamAId,
    match.teamAName || '队伍A',
    teamAPlayers,
    teamBId,
    match.teamBName || '队伍B',
    teamBPlayers,
    match.bestOf || 3
  )

  // 更新比赛状态
  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'
  match.completedAt = new Date()

  // 保存比赛详情
  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'madrid'
  matchDetail.seasonId = String(madridBracket.seasonYear)
  matchDetailStore.saveMatchDetail(match.id, matchDetail)

  // 记录选手表现
  matchDetail.games.forEach(game => {
    game.teamAPlayers.forEach(perf => {
      playerStore.recordPerformance(
        perf.playerId,
        perf.playerName,
        String(match.teamAId),
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
        String(match.teamBId),
        perf.position,
        perf.impactScore,
        perf.actualAbility,
        String(madridBracket.seasonYear),
        'INTL'
      )
    })
  })

  ElMessage.success(`比赛完成: ${match.teamAName} ${matchDetail.finalScoreA} - ${matchDetail.finalScoreB} ${match.teamBName}`)

  // 检查是否所有比赛都完成了
  if (madridBracket.status === 'knockout_stage') {
    checkKnockoutCompletion()
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
 * 生成淘汰赛对阵
 */
const handleGenerateKnockout = async () => {
  generatingKnockout.value = true

  try {
    await new Promise(resolve => setTimeout(resolve, 500))

    // 获取各小组前两名
    const eastTeams: { id: string; name: string }[] = []
    const westTeams: { id: string; name: string }[] = []

    madridBracket.groups.forEach((group, index) => {
      const sortedStandings = [...group.standings].sort((a, b) => b.points - a.points)
      const top2 = sortedStandings.slice(0, 2)

      if (index < 4) {
        // A-D组进入东半区
        top2.forEach(team => {
          eastTeams.push({ id: String(team.teamId), name: team.teamName })
        })
      } else {
        // E-H组进入西半区
        top2.forEach(team => {
          westTeams.push({ id: String(team.teamId), name: team.teamName })
        })
      }
    })

    // 生成东半区对阵
    madridBracket.knockoutEast = {
      round1: [
        { id: 'e-q1', teamAId: eastTeams[0].id, teamAName: eastTeams[0].name, teamBId: eastTeams[7].id, teamBName: eastTeams[7].name, status: 'scheduled', bestOf: 5, matchType: 'east_quarter' },
        { id: 'e-q2', teamAId: eastTeams[1].id, teamAName: eastTeams[1].name, teamBId: eastTeams[6].id, teamBName: eastTeams[6].name, status: 'scheduled', bestOf: 5, matchType: 'east_quarter' },
        { id: 'e-q3', teamAId: eastTeams[2].id, teamAName: eastTeams[2].name, teamBId: eastTeams[5].id, teamBName: eastTeams[5].name, status: 'scheduled', bestOf: 5, matchType: 'east_quarter' },
        { id: 'e-q4', teamAId: eastTeams[3].id, teamAName: eastTeams[3].name, teamBId: eastTeams[4].id, teamBName: eastTeams[4].name, status: 'scheduled', bestOf: 5, matchType: 'east_quarter' }
      ],
      semiFinals: [
        { id: 'e-s1', teamAId: '', teamAName: '待定', teamBId: '', teamBName: '待定', status: 'scheduled', bestOf: 5, matchType: 'east_semi' },
        { id: 'e-s2', teamAId: '', teamAName: '待定', teamBId: '', teamBName: '待定', status: 'scheduled', bestOf: 5, matchType: 'east_semi' }
      ],
      final: [
        { id: 'e-f', teamAId: '', teamAName: '待定', teamBId: '', teamBName: '待定', status: 'scheduled', bestOf: 5, matchType: 'east_final' }
      ]
    }

    // 生成西半区对阵
    madridBracket.knockoutWest = {
      round1: [
        { id: 'w-q1', teamAId: westTeams[0].id, teamAName: westTeams[0].name, teamBId: westTeams[7].id, teamBName: westTeams[7].name, status: 'scheduled', bestOf: 5, matchType: 'west_quarter' },
        { id: 'w-q2', teamAId: westTeams[1].id, teamAName: westTeams[1].name, teamBId: westTeams[6].id, teamBName: westTeams[6].name, status: 'scheduled', bestOf: 5, matchType: 'west_quarter' },
        { id: 'w-q3', teamAId: westTeams[2].id, teamAName: westTeams[2].name, teamBId: westTeams[5].id, teamBName: westTeams[5].name, status: 'scheduled', bestOf: 5, matchType: 'west_quarter' },
        { id: 'w-q4', teamAId: westTeams[3].id, teamAName: westTeams[3].name, teamBId: westTeams[4].id, teamBName: westTeams[4].name, status: 'scheduled', bestOf: 5, matchType: 'west_quarter' }
      ],
      semiFinals: [
        { id: 'w-s1', teamAId: '', teamAName: '待定', teamBId: '', teamBName: '待定', status: 'scheduled', bestOf: 5, matchType: 'west_semi' },
        { id: 'w-s2', teamAId: '', teamAName: '待定', teamBId: '', teamBName: '待定', status: 'scheduled', bestOf: 5, matchType: 'west_semi' }
      ],
      final: [
        { id: 'w-f', teamAId: '', teamAName: '待定', teamBId: '', teamBName: '待定', status: 'scheduled', bestOf: 5, matchType: 'west_final' }
      ]
    }

    // 生成季军赛和总决赛
    madridBracket.thirdPlaceMatch = {
      id: 'third',
      teamAId: '',
      teamAName: '待定',
      teamBId: '',
      teamBName: '待定',
      status: 'scheduled',
      bestOf: 5,
      matchType: 'third_place'
    }

    madridBracket.grandFinal = {
      id: 'final',
      teamAId: '',
      teamAName: '待定',
      teamBId: '',
      teamBName: '待定',
      status: 'scheduled',
      bestOf: 5,
      matchType: 'grand_final'
    }

    madridBracket.status = 'knockout_stage'
    ElMessage.success('淘汰赛对阵生成成功!')
  } finally {
    generatingKnockout.value = false
  }
}

/**
 * 批量模拟小组赛
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

    const groupMatches = madridBracket.groups.flatMap(g => g.matches)
    const uncompletedGroupMatches = groupMatches.filter(m => m.status !== 'completed')

    for (let i = 0; i < uncompletedGroupMatches.length; i++) {
      const match = uncompletedGroupMatches[i]
      await simulateMatch(match)
      simulationProgress.value = Math.floor(((i + 1) / uncompletedGroupMatches.length) * 100)
      await new Promise(resolve => setTimeout(resolve, 100))
    }

    ElMessage.success('小组赛模拟完成！现在可以生成淘汰赛对阵。')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('小组赛模拟失败:', error)
      ElMessage.error(error.message || '小组赛模拟失败')
    }
  } finally {
    simulatingGroupStage.value = false
    simulationProgress.value = 0
  }
}

/**
 * 批量模拟淘汰赛
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

    await simulateKnockoutStage()

    madridBracket.status = 'completed'
    ElMessage.success('淘汰赛模拟完成！')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('淘汰赛模拟失败:', error)
      ElMessage.error(error.message || '淘汰赛模拟失败')
    }
  } finally {
    simulatingKnockout.value = false
    simulationProgress.value = 0
  }
}

/**
 * 模拟单场比赛（内部方法）
 */
const simulateMatch = async (match: ClauchMatch) => {
  // 获取队伍区域信息
  const getTeamRegion = (teamId: string): string => {
    if (teamId.startsWith('lpl')) return 'LPL'
    if (teamId.startsWith('lck')) return 'LCK'
    if (teamId.startsWith('lec')) return 'LEC'
    if (teamId.startsWith('lcs')) return 'LCS'
    return 'Unknown'
  }

  const teamAId = String(match.teamAId || '')
  const teamBId = String(match.teamBId || '')
  const teamARegion = getTeamRegion(teamAId)
  const teamBRegion = getTeamRegion(teamBId)

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(teamAId, match.teamAName || '队伍A', teamARegion)
  const teamBPlayers = generateTeamPlayers(teamBId, match.teamBName || '队伍B', teamBRegion)

  // 使用 PowerEngine 模拟比赛
  const matchDetail = PowerEngine.simulateMatch(
    teamAId,
    match.teamAName || '队伍A',
    teamAPlayers,
    teamBId,
    match.teamBName || '队伍B',
    teamBPlayers,
    match.bestOf || 3
  )

  // 更新比赛状态
  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'
  match.completedAt = new Date()

  // 保存比赛详情
  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'madrid'
  matchDetail.seasonId = String(madridBracket.seasonYear)
  matchDetailStore.saveMatchDetail(match.id, matchDetail)

  // 记录选手表现
  matchDetail.games.forEach(game => {
    game.teamAPlayers.forEach(perf => {
      playerStore.recordPerformance(
        perf.playerId,
        perf.playerName,
        String(match.teamAId),
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
        String(match.teamBId),
        perf.position,
        perf.impactScore,
        perf.actualAbility,
        String(madridBracket.seasonYear),
        'INTL'
      )
    })
  })
}

/**
 * 模拟淘汰赛阶段
 */
const simulateKnockoutStage = async () => {
  if (!madridBracket.knockoutEast || !madridBracket.knockoutWest) return

  // 模拟东半区
  await simulateBracket(madridBracket.knockoutEast)

  // 模拟西半区
  await simulateBracket(madridBracket.knockoutWest)

  // 设置季军赛和总决赛的对阵
  const eastFinal = madridBracket.knockoutEast.final?.[0]
  const westFinal = madridBracket.knockoutWest.final?.[0]

  if (eastFinal && westFinal && madridBracket.thirdPlaceMatch && madridBracket.grandFinal) {
    // 设置总决赛对阵（两个半区的冠军）
    madridBracket.grandFinal.teamAId = eastFinal.winnerId || ''
    madridBracket.grandFinal.teamAName = eastFinal.winnerId === eastFinal.teamAId ? eastFinal.teamAName : eastFinal.teamBName
    madridBracket.grandFinal.teamBId = westFinal.winnerId || ''
    madridBracket.grandFinal.teamBName = westFinal.winnerId === westFinal.teamAId ? westFinal.teamAName : westFinal.teamBName

    // 设置季军赛对阵（两个半区的亚军）
    madridBracket.thirdPlaceMatch.teamAId = eastFinal.winnerId === eastFinal.teamAId ? (eastFinal.teamBId || '') : (eastFinal.teamAId || '')
    madridBracket.thirdPlaceMatch.teamAName = eastFinal.winnerId === eastFinal.teamAId ? (eastFinal.teamBName || '待定') : (eastFinal.teamAName || '待定')
    madridBracket.thirdPlaceMatch.teamBId = westFinal.winnerId === westFinal.teamAId ? (westFinal.teamBId || '') : (westFinal.teamAId || '')
    madridBracket.thirdPlaceMatch.teamBName = westFinal.winnerId === westFinal.teamAId ? (westFinal.teamBName || '待定') : (westFinal.teamAName || '待定')

    // 模拟季军赛
    await simulateMatch(madridBracket.thirdPlaceMatch)
    await new Promise(resolve => setTimeout(resolve, 200))

    // 模拟总决赛
    await simulateMatch(madridBracket.grandFinal)

    // 设置最终排名
    const gf = madridBracket.grandFinal
    const tp = madridBracket.thirdPlaceMatch

    if (gf.winnerId === gf.teamAId) {
      madridBracket.champion = { teamName: gf.teamAName || '', regionName: 'LPL' }
      madridBracket.runnerUp = { teamName: gf.teamBName || '', regionName: 'LCK' }
    } else {
      madridBracket.champion = { teamName: gf.teamBName || '', regionName: 'LCK' }
      madridBracket.runnerUp = { teamName: gf.teamAName || '', regionName: 'LPL' }
    }

    if (tp.winnerId === tp.teamAId) {
      madridBracket.thirdPlace = { teamName: tp.teamAName || '', regionName: 'LEC' }
      madridBracket.fourthPlace = { teamName: tp.teamBName || '', regionName: 'LCS' }
    } else {
      madridBracket.thirdPlace = { teamName: tp.teamBName || '', regionName: 'LCS' }
      madridBracket.fourthPlace = { teamName: tp.teamAName || '', regionName: 'LEC' }
    }

    showChampionCelebration(madridBracket.champion?.teamName || '')
  }
}

/**
 * 模拟一个半区的淘汰赛
 */
const simulateBracket = async (bracket: ClauchKnockoutBracketType) => {
  // 第一轮
  if (bracket.round1) {
    for (const match of bracket.round1) {
      if (match.status !== 'completed') {
        await simulateMatch(match)
        await new Promise(resolve => setTimeout(resolve, 150))
      }
    }
  }

  // 更新半决赛对阵
  if (bracket.semiFinals && bracket.round1) {
    bracket.semiFinals[0].teamAId = bracket.round1[0].winnerId || ''
    bracket.semiFinals[0].teamAName = bracket.round1[0].winnerId === bracket.round1[0].teamAId ? bracket.round1[0].teamAName : bracket.round1[0].teamBName
    bracket.semiFinals[0].teamBId = bracket.round1[1].winnerId || ''
    bracket.semiFinals[0].teamBName = bracket.round1[1].winnerId === bracket.round1[1].teamAId ? bracket.round1[1].teamAName : bracket.round1[1].teamBName

    bracket.semiFinals[1].teamAId = bracket.round1[2].winnerId || ''
    bracket.semiFinals[1].teamAName = bracket.round1[2].winnerId === bracket.round1[2].teamAId ? bracket.round1[2].teamAName : bracket.round1[2].teamBName
    bracket.semiFinals[1].teamBId = bracket.round1[3].winnerId || ''
    bracket.semiFinals[1].teamBName = bracket.round1[3].winnerId === bracket.round1[3].teamAId ? bracket.round1[3].teamAName : bracket.round1[3].teamBName

    // 模拟半决赛
    for (const match of bracket.semiFinals) {
      if (match.status !== 'completed') {
        await simulateMatch(match)
        await new Promise(resolve => setTimeout(resolve, 150))
      }
    }
  }

  // 更新决赛对阵
  if (bracket.final && bracket.semiFinals) {
    bracket.final[0].teamAId = bracket.semiFinals[0].winnerId || ''
    bracket.final[0].teamAName = bracket.semiFinals[0].winnerId === bracket.semiFinals[0].teamAId ? bracket.semiFinals[0].teamAName : bracket.semiFinals[0].teamBName
    bracket.final[0].teamBId = bracket.semiFinals[1].winnerId || ''
    bracket.final[0].teamBName = bracket.semiFinals[1].winnerId === bracket.semiFinals[1].teamAId ? bracket.semiFinals[1].teamAName : bracket.semiFinals[1].teamBName

    // 模拟决赛
    if (bracket.final[0].status !== 'completed') {
      await simulateMatch(bracket.final[0])
      await new Promise(resolve => setTimeout(resolve, 150))
    }
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
