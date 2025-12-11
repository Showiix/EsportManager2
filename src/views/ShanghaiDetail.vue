<template>
  <div class="shanghai-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><Trophy /></el-icon>
          上海大师赛
        </h1>
        <p class="page-description">
          12支队伍(各赛区夏季赛冠亚季军)参赛,双败淘汰赛制,决出世界最强战队
        </p>
      </div>
      <div class="header-actions">
        <el-button @click="refreshData" :icon="Refresh">刷新数据</el-button>
      </div>
    </div>

    <!-- 上海大师赛状态卡片 -->
    <div v-if="currentBracket" class="shanghai-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>{{ currentBracket.seasonYear }} 上海大师赛</h2>
          <el-tag :type="getStatusType(currentBracket.status)" size="large">
            {{ getStatusText(currentBracket.status) }}
          </el-tag>
        </div>
        <div class="status-actions">
          <el-button
            v-if="currentBracket.status !== 'completed'"
            type="warning"
            @click="batchSimulate"
            :loading="batchSimulating"
            :icon="Promotion"
          >
            {{ batchSimulating ? `模拟中 (${simulationProgress}%)` : '一键模拟全部' }}
          </el-button>
        </div>
      </div>

      <!-- 参赛队伍分组 -->
      <div class="teams-groups">
        <div class="team-group legendary">
          <h3><el-icon><Star /></el-icon> 传奇组 (夏季赛冠军)</h3>
          <div class="team-list">
            <div
              v-for="team in mockBracket.legendaryGroup"
              :key="team.teamId"
              class="team-item"
            >
              <span class="team-name">{{ team.teamName }}</span>
              <el-tag size="small">{{ team.regionName }}</el-tag>
            </div>
          </div>
        </div>

        <div class="team-group challenger">
          <h3><el-icon><Medal /></el-icon> 挑战者组 (夏季赛亚军)</h3>
          <div class="team-list">
            <div
              v-for="team in mockBracket.challengerGroup"
              :key="team.teamId"
              class="team-item"
            >
              <span class="team-name">{{ team.teamName }}</span>
              <el-tag size="small">{{ team.regionName }}</el-tag>
            </div>
          </div>
        </div>

        <div class="team-group qualifier">
          <h3><el-icon><Flag /></el-icon> 资格赛组 (夏季赛季军)</h3>
          <div class="team-list">
            <div
              v-for="team in mockBracket.qualifierGroup"
              :key="team.teamId"
              class="team-item"
            >
              <span class="team-name">{{ team.teamName }}</span>
              <el-tag size="small">{{ team.regionName }}</el-tag>
            </div>
          </div>
        </div>
      </div>

      <!-- 对阵图 -->
      <div class="bracket-section">
        <MSIBracketView
          v-if="currentBracket"
          :bracket="currentBracket"
          @simulate-match="simulateMatch"
          @view-match="viewMatchDetails"
        />
      </div>

      <!-- 最终排名 -->
      <div v-if="currentBracket.status === 'completed'" class="final-standings">
        <h3>最终排名与积分</h3>
        <div class="standings-grid">
          <div class="standing-item champion">
            <div class="rank-badge">🏆 冠军</div>
            <div class="team-name">{{ currentBracket.champion?.teamName }}</div>
            <div class="points">+{{ currentBracket.pointsDistribution.champion }}分</div>
          </div>

          <div class="standing-item runner-up">
            <div class="rank-badge">🥈 亚军</div>
            <div class="team-name">{{ currentBracket.runnerUp?.teamName }}</div>
            <div class="points">+{{ currentBracket.pointsDistribution.runnerUp }}分</div>
          </div>

          <div class="standing-item third">
            <div class="rank-badge">🥉 季军</div>
            <div class="team-name">{{ currentBracket.thirdPlace?.teamName }}</div>
            <div class="points">+{{ currentBracket.pointsDistribution.thirdPlace }}分</div>
          </div>

          <div class="standing-item fourth">
            <div class="rank-badge">4️⃣ 殿军</div>
            <div class="team-name">{{ currentBracket.fourthPlace?.teamName }}</div>
            <div class="points">+{{ currentBracket.pointsDistribution.fourthPlace }}分</div>
          </div>
        </div>

        <!-- 败者组第二轮 (5-6名) -->
        <div v-if="currentBracket.loserRound2?.length > 0" class="loser-standings">
          <h4>败者组第二轮 (5-6名)</h4>
          <div class="loser-grid">
            <div
              v-for="(team, index) in currentBracket.loserRound2"
              :key="team.teamId"
              class="loser-item loser-r2"
            >
              <div class="rank-badge"><span class="rank-number">{{ 5 + index }}</span></div>
              <div class="team-name">{{ team.teamName }}</div>
              <div class="points">+{{ currentBracket.pointsDistribution.loserRound2 }}分</div>
            </div>
          </div>
        </div>

        <!-- 败者组第一轮 (7-8名) -->
        <div v-if="currentBracket.loserRound1?.length > 0" class="loser-standings">
          <h4>败者组第一轮 (7-8名)</h4>
          <div class="loser-grid">
            <div
              v-for="(team, index) in currentBracket.loserRound1"
              :key="team.teamId"
              class="loser-item loser-r1"
            >
              <div class="rank-badge"><span class="rank-number">{{ 7 + index }}</span></div>
              <div class="team-name">{{ team.teamName }}</div>
              <div class="points">+{{ currentBracket.pointsDistribution.loserRound1 }}分</div>
            </div>
          </div>
        </div>

        <!-- 完成后的操作区 -->
        <div class="completed-actions">
          <el-alert
            title="上海大师赛已完成！"
            type="success"
            :closable="false"
            show-icon
            class="completion-alert"
          >
            <template #default>
              <p>恭喜 <strong>{{ currentBracket.champion?.teamName }}</strong> 获得上海大师赛冠军！</p>
              <p>现在可以继续进行S世界赛了。</p>
            </template>
          </el-alert>

          <div class="action-buttons">
            <el-button
              type="primary"
              size="large"
              @click="goToWorlds"
              :icon="ArrowRight"
            >
              前往S世界赛
            </el-button>
          </div>
        </div>
      </div>
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
import { ref, computed, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Trophy,
  Refresh,
  Promotion,
  Star,
  Medal,
  Flag,
  ArrowRight
} from '@element-plus/icons-vue'
import MSIBracketView from '@/components/msi/MSIBracketView.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { PowerEngine } from '@/engines/PowerEngine'
import type { MatchDetail } from '@/types/matchDetail'
import type { Player, PlayerPosition } from '@/types/player'

const router = useRouter()
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()

// Mock 上海大师赛数据
const mockBracket = reactive({
  id: '1',
  seasonId: '1',
  seasonYear: 2024,
  status: 'in_progress' as string,
  qualifiedTeams: [
    { teamId: '1', teamName: 'BLG', regionName: 'LPL', seed: 1 },
    { teamId: '2', teamName: 'GEN', regionName: 'LCK', seed: 1 },
    { teamId: '3', teamName: 'FNC', regionName: 'LEC', seed: 1 },
    { teamId: '4', teamName: 'TL', regionName: 'LCS', seed: 1 },
    { teamId: '5', teamName: 'TES', regionName: 'LPL', seed: 2 },
    { teamId: '6', teamName: 'T1', regionName: 'LCK', seed: 2 },
    { teamId: '7', teamName: 'G2', regionName: 'LEC', seed: 2 },
    { teamId: '8', teamName: 'C9', regionName: 'LCS', seed: 2 },
    { teamId: '9', teamName: 'JDG', regionName: 'LPL', seed: 3 },
    { teamId: '10', teamName: 'DK', regionName: 'LCK', seed: 3 },
    { teamId: '11', teamName: 'MAD', regionName: 'LEC', seed: 3 },
    { teamId: '12', teamName: '100T', regionName: 'LCS', seed: 3 },
  ],
  legendaryGroup: [
    { teamId: '1', teamName: 'BLG', regionName: 'LPL' },
    { teamId: '2', teamName: 'GEN', regionName: 'LCK' },
    { teamId: '3', teamName: 'FNC', regionName: 'LEC' },
    { teamId: '4', teamName: 'TL', regionName: 'LCS' },
  ],
  challengerGroup: [
    { teamId: '5', teamName: 'TES', regionName: 'LPL' },
    { teamId: '6', teamName: 'T1', regionName: 'LCK' },
    { teamId: '7', teamName: 'G2', regionName: 'LEC' },
    { teamId: '8', teamName: 'C9', regionName: 'LCS' },
  ],
  qualifierGroup: [
    { teamId: '9', teamName: 'JDG', regionName: 'LPL' },
    { teamId: '10', teamName: 'DK', regionName: 'LCK' },
    { teamId: '11', teamName: 'MAD', regionName: 'LEC' },
    { teamId: '12', teamName: '100T', regionName: 'LCS' },
  ],
  rounds: [
    // 预选赛阶段
    {
      roundNumber: 1,
      roundName: '预选赛',
      status: 'pending',
      matches: [
        { id: 'qual1', matchType: 'qualifier', teamAId: '9', teamBId: '12', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'qual2', matchType: 'qualifier', teamAId: '10', teamBId: '11', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'chal1', matchType: 'challenger', teamAId: '5', teamBId: '8', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'chal2', matchType: 'challenger', teamAId: '6', teamBId: '7', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
      ]
    },
    // 败者组
    {
      roundNumber: 2,
      roundName: '败者组',
      status: 'pending',
      matches: [
        { id: 'lr1_1', matchType: 'loser_r1', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lr1_2', matchType: 'loser_r1', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lr2_1', matchType: 'loser_r2', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lr2_2', matchType: 'loser_r2', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lr3_1', matchType: 'loser_r3', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lr3_2', matchType: 'loser_r3', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lr4', matchType: 'loser_r4', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'lf', matchType: 'loser_final', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
      ]
    },
    // 胜者组 (传奇组)
    {
      roundNumber: 3,
      roundName: '胜者组',
      status: 'pending',
      matches: [
        { id: 'wr1_1', matchType: 'winner_r1', teamAId: '1', teamBId: '4', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'wr1_2', matchType: 'winner_r1', teamAId: '2', teamBId: '3', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
        { id: 'wf', matchType: 'winner_final', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
      ]
    },
    // 总决赛
    {
      roundNumber: 4,
      roundName: '总决赛',
      status: 'pending',
      matches: [
        { id: 'gf', matchType: 'grand_final', teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0 },
      ]
    }
  ],
  pointsDistribution: {
    champion: 20,
    runnerUp: 16,
    thirdPlace: 12,
    fourthPlace: 8,
    loserRound2: 6,
    loserRound1: 4
  },
  champion: null as any,
  runnerUp: null as any,
  thirdPlace: null as any,
  fourthPlace: null as any,
  loserRound2: [] as any[],  // 败者组第二轮败者 (2队)
  loserRound1: [] as any[]   // 败者组第一轮败者 (2队)
})

// 响应式状态
const batchSimulating = ref(false)
const simulationProgress = ref(0)
const showMatchDetails = ref(false)
const selectedMatch = ref<any>(null)
const simulating = ref(false)

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 计算属性 - 使用 mock 数据
const currentBracket = computed(() => mockBracket)

/**
 * 刷新数据
 */
const refreshData = () => {
  ElMessage.success('数据已刷新')
}

/**
 * 生成队伍选手数据
 */
const generateTeamPlayers = (teamId: string, teamName: string): Player[] => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  return positions.map((pos, idx) => ({
    id: `${teamId}-${pos}`,
    gameId: `Player${idx + 1}`,
    name: `Player${idx + 1}`,
    teamId: teamId,
    teamName: teamName,
    position: pos,
    regionId: 'INTL',
    regionName: '国际赛',
    ability: 70 + Math.floor(Math.random() * 25),
    potential: 80 + Math.floor(Math.random() * 15),
    stability: 60 + Math.floor(Math.random() * 35),
    condition: Math.floor(Math.random() * 11) - 5,
    age: 18 + Math.floor(Math.random() * 10),
    tag: Math.random() > 0.7 ? 'GENIUS' : Math.random() > 0.4 ? 'NORMAL' : 'ORDINARY'
  } as Player))
}

/**
 * 模拟单场比赛（使用PowerEngine）
 */
const simulateMatch = (match: any) => {
  const teamAName = getTeamName(match.teamAId)
  const teamBName = getTeamName(match.teamBId)

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(match.teamAId, teamAName)
  const teamBPlayers = generateTeamPlayers(match.teamBId, teamBName)

  // 使用 PowerEngine 模拟比赛
  const matchDetail = PowerEngine.simulateMatch(
    match.teamAId,
    teamAName,
    teamAPlayers,
    match.teamBId,
    teamBName,
    teamBPlayers,
    match.bestOf || 5
  )

  // 更新比赛结果
  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'

  // 保存比赛详情
  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'shanghai'
  matchDetail.seasonId = String(mockBracket.seasonYear)
  matchDetailStore.saveMatchDetail(match.id, matchDetail)

  // 记录选手表现
  matchDetail.games.forEach(game => {
    game.teamAPlayers.forEach(perf => {
      playerStore.recordPerformance(
        perf.playerId,
        perf.playerName,
        match.teamAId,
        perf.position,
        perf.impactScore,
        perf.actualAbility,
        String(mockBracket.seasonYear),
        'INTL'
      )
    })
    game.teamBPlayers.forEach(perf => {
      playerStore.recordPerformance(
        perf.playerId,
        perf.playerName,
        match.teamBId,
        perf.position,
        perf.impactScore,
        perf.actualAbility,
        String(mockBracket.seasonYear),
        'INTL'
      )
    })
  })

  ElMessage.success(`比赛完成: ${matchDetail.finalScoreA} - ${matchDetail.finalScoreB}`)

  updateBracketProgression()
}

/**
 * 更新对阵晋级 - 双败赛制
 */
const updateBracketProgression = () => {
  const rounds = mockBracket.rounds
  const preliminary = rounds[0].matches
  const loserBracket = rounds[1].matches
  const winnerBracket = rounds[2].matches
  const finalRound = rounds[3].matches

  const getLoser = (match: any) => {
    if (!match.winnerId) return null
    return match.winnerId === match.teamAId ? match.teamBId : match.teamAId
  }

  const qual1 = preliminary.find((m: any) => m.id === 'qual1')
  const qual2 = preliminary.find((m: any) => m.id === 'qual2')
  const chal1 = preliminary.find((m: any) => m.id === 'chal1')
  const chal2 = preliminary.find((m: any) => m.id === 'chal2')

  const lr1_1 = loserBracket.find((m: any) => m.id === 'lr1_1')
  const lr1_2 = loserBracket.find((m: any) => m.id === 'lr1_2')

  if (qual1?.winnerId && chal1?.winnerId && lr1_1) {
    lr1_1.teamAId = qual1.winnerId
    lr1_1.teamBId = getLoser(chal1)
  }
  if (qual2?.winnerId && chal2?.winnerId && lr1_2) {
    lr1_2.teamAId = qual2.winnerId
    lr1_2.teamBId = getLoser(chal2)
  }

  const lr2_1 = loserBracket.find((m: any) => m.id === 'lr2_1')
  const lr2_2 = loserBracket.find((m: any) => m.id === 'lr2_2')

  if (chal1?.winnerId && lr1_1?.winnerId && lr2_1) {
    lr2_1.teamAId = chal1.winnerId
    lr2_1.teamBId = lr1_1.winnerId
  }
  if (chal2?.winnerId && lr1_2?.winnerId && lr2_2) {
    lr2_2.teamAId = chal2.winnerId
    lr2_2.teamBId = lr1_2.winnerId
  }

  const wr1_1 = winnerBracket.find((m: any) => m.id === 'wr1_1')
  const wr1_2 = winnerBracket.find((m: any) => m.id === 'wr1_2')
  const lr3_1 = loserBracket.find((m: any) => m.id === 'lr3_1')
  const lr3_2 = loserBracket.find((m: any) => m.id === 'lr3_2')

  if (lr2_1?.winnerId && wr1_1?.winnerId && lr3_1) {
    lr3_1.teamAId = lr2_1.winnerId
    lr3_1.teamBId = getLoser(wr1_1)
  }
  if (lr2_2?.winnerId && wr1_2?.winnerId && lr3_2) {
    lr3_2.teamAId = lr2_2.winnerId
    lr3_2.teamBId = getLoser(wr1_2)
  }

  const wf = winnerBracket.find((m: any) => m.id === 'wf')
  if (wr1_1?.winnerId && wr1_2?.winnerId && wf) {
    wf.teamAId = wr1_1.winnerId
    wf.teamBId = wr1_2.winnerId
  }

  const lr4 = loserBracket.find((m: any) => m.id === 'lr4')
  if (lr3_1?.winnerId && lr3_2?.winnerId && lr4) {
    lr4.teamAId = lr3_1.winnerId
    lr4.teamBId = lr3_2.winnerId
  }

  const lf = loserBracket.find((m: any) => m.id === 'lf')
  if (wf?.winnerId && lr4?.winnerId && lf) {
    lf.teamAId = getLoser(wf)
    lf.teamBId = lr4.winnerId
  }

  const gf = finalRound.find((m: any) => m.id === 'gf')
  if (wf?.winnerId && lf?.winnerId && gf) {
    gf.teamAId = wf.winnerId
    gf.teamBId = lf.winnerId
  }

  if (gf?.winnerId) {
    mockBracket.status = 'completed'
    const champion = mockBracket.qualifiedTeams.find(t => t.teamId === gf.winnerId)
    const runnerUp = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(gf))
    const thirdPlace = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(lf))
    const fourthPlace = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr4))

    // 败者组第二轮败者 (lr3_1, lr3_2的败者) - 6分
    const loserR2Teams: any[] = []
    if (lr3_1?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr3_1))
      if (loser) loserR2Teams.push(loser)
    }
    if (lr3_2?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr3_2))
      if (loser) loserR2Teams.push(loser)
    }

    // 败者组第一轮败者 (lr2_1, lr2_2的败者) - 4分
    const loserR1Teams: any[] = []
    if (lr2_1?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr2_1))
      if (loser) loserR1Teams.push(loser)
    }
    if (lr2_2?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr2_2))
      if (loser) loserR1Teams.push(loser)
    }

    mockBracket.champion = champion || null
    mockBracket.runnerUp = runnerUp || null
    mockBracket.thirdPlace = thirdPlace || null
    mockBracket.fourthPlace = fourthPlace || null
    mockBracket.loserRound2 = loserR2Teams
    mockBracket.loserRound1 = loserR1Teams

    showChampionCelebration(champion?.teamName || '')
  }
}

/**
 * 批量模拟
 */
const batchSimulate = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的比赛,直到决出冠军。是否继续?',
      '批量模拟确认',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    batchSimulating.value = true
    simulationProgress.value = 0

    const allMatches = mockBracket.rounds.flatMap(r => r.matches)
    let completed = 0
    const total = allMatches.length

    for (const match of allMatches) {
      if (match.status !== 'completed' && match.teamAId && match.teamBId) {
        await new Promise(resolve => setTimeout(resolve, 300))
        simulateMatch(match)
        completed++
        simulationProgress.value = Math.round((completed / total) * 100)
      }
    }

    ElMessage.success('批量模拟完成!')
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('模拟失败')
    }
  } finally {
    batchSimulating.value = false
    simulationProgress.value = 0
  }
}

/**
 * 显示冠军庆祝动画
 */
const showChampionCelebration = (championName: string) => {
  ElMessageBox.alert(
    `恭喜 ${championName} 获得上海大师赛冠军！`,
    '🏆 上海大师赛冠军诞生! 🏆',
    {
      confirmButtonText: '太棒了!',
      customClass: 'champion-celebration-box',
      showClose: false,
      center: true
    }
  )
}

/**
 * 查看比赛详情
 */
const viewMatchDetails = (match: any) => {
  const detail = matchDetailStore.getMatchDetail(match.id)
  if (detail) {
    currentMatchDetail.value = detail
    showMatchDetailDialog.value = true
  } else {
    ElMessage.warning('暂无比赛详情数据，请先模拟比赛')
  }
}

/**
 * 关闭比赛详情弹窗
 */
const handleCloseMatchDetail = () => {
  showMatchDetailDialog.value = false
  currentMatchDetail.value = null
}

/**
 * 模拟当前选中的比赛
 */
const simulateCurrentMatch = () => {
  if (!selectedMatch.value) return
  showMatchDetails.value = false
  simulateMatch(selectedMatch.value)
}

/**
 * 前往S世界赛
 */
const goToWorlds = () => {
  router.push('/tournaments/worlds')
}

/**
 * 获取队伍A的ID
 */
const getTeamAId = (match: any): string | null => {
  return match?.teamAId || match?.homeTeamId || null
}

/**
 * 获取队伍B的ID
 */
const getTeamBId = (match: any): string | null => {
  return match?.teamBId || match?.awayTeamId || null
}

/**
 * 获取比赛A队比分
 */
const getMatchScoreA = (match: any): number => {
  return match?.scoreA ?? 0
}

/**
 * 获取比赛B队比分
 */
const getMatchScoreB = (match: any): number => {
  return match?.scoreB ?? 0
}

/**
 * 检查比赛是否有结果
 */
const hasMatchResult = (match: any): boolean => {
  return match?.status === 'completed'
}

/**
 * 判断是否为比赛获胜方
 */
const isMatchWinner = (match: any, teamId: string | null): boolean => {
  if (!teamId || !match) return false
  return match.winnerId?.toString() === teamId.toString()
}

/**
 * 获取队伍名称
 */
const getTeamName = (teamId: string | null): string => {
  if (!teamId) return '待定'
  const team = mockBracket.qualifiedTeams.find(t => t.teamId === teamId)
  return team?.teamName || '待定'
}

/**
 * 获取状态类型
 */
const getStatusType = (status: string) => {
  const typeMap: Record<string, any> = {
    'not_started': 'info',
    'in_progress': 'warning',
    'completed': 'success'
  }
  return typeMap[status] || 'info'
}

/**
 * 获取状态文本
 */
const getStatusText = (status: string) => {
  const textMap: Record<string, string> = {
    'not_started': '未开始',
    'in_progress': '进行中',
    'completed': '已完成'
  }
  return textMap[status] || status
}

/**
 * 获取比赛类型名称
 */
const getMatchTypeName = (matchType: string): string => {
  const typeMap: Record<string, string> = {
    'quarter_final': '八强赛',
    'semi_final': '半决赛',
    'third_place': '季军赛',
    'grand_final': '总决赛'
  }
  return typeMap[matchType] || matchType
}

/**
 * 获取比赛类型标签颜色
 */
const getMatchTypeBadgeType = (matchType: string) => {
  if (matchType === 'grand_final') return 'danger'
  if (matchType === 'semi_final') return 'success'
  if (matchType === 'third_place') return 'warning'
  return 'info'
}

/**
 * 格式化日期
 */
const formatDate = (dateString: string | undefined): string => {
  if (!dateString) return '未知时间'
  return new Date(dateString).toLocaleString('zh-CN')
}
</script>

<style scoped lang="scss">
.shanghai-management {
  padding: 24px;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content {
      .page-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 28px;
        font-weight: 700;
        margin: 0 0 8px 0;
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

  .shanghai-status-card {
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

      .status-actions {
        display: flex;
        gap: 12px;
      }
    }

    .teams-groups {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 16px;
      margin-bottom: 32px;

      .team-group {
        padding: 16px;
        border-radius: 8px;
        border: 2px solid;

        h3 {
          display: flex;
          align-items: center;
          gap: 8px;
          margin: 0 0 12px 0;
          font-size: 16px;
          font-weight: 600;
        }

        .team-list {
          display: flex;
          flex-direction: column;
          gap: 8px;

          .team-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 12px;
            background: white;
            border-radius: 6px;

            .team-name {
              font-weight: 500;
              color: #374151;
            }
          }
        }

        &.legendary {
          border-color: #f59e0b;
          background: #fffbeb;

          h3 {
            color: #d97706;
          }
        }

        &.challenger {
          border-color: #3b82f6;
          background: #eff6ff;

          h3 {
            color: #2563eb;
          }
        }

        &.qualifier {
          border-color: #10b981;
          background: #f0fdf4;

          h3 {
            color: #059669;
          }
        }
      }
    }

    .bracket-section {
      margin-bottom: 32px;
    }

    .final-standings {
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
            font-size: 24px;
            margin-bottom: 8px;
          }

          .team-name {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 8px;
            color: #1f2937;
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

      .completed-actions {
        margin-top: 32px;

        .completion-alert {
          margin-bottom: 20px;
          border-radius: 8px;

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

        .action-buttons {
          display: flex;
          justify-content: center;
          gap: 16px;

          .el-button {
            min-width: 200px;
          }
        }
      }

      // 败者组排名样式
      .loser-standings {
        margin-top: 24px;

        h4 {
          margin: 0 0 12px 0;
          font-size: 16px;
          font-weight: 600;
          color: #6b7280;
        }

        .loser-grid {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: 12px;

          .loser-item {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 12px 16px;
            border-radius: 8px;
            border: 1px solid;

            .rank-badge {
              .rank-number {
                display: flex;
                align-items: center;
                justify-content: center;
                width: 28px;
                height: 28px;
                border-radius: 50%;
                background: #9ca3af;
                color: white;
                font-size: 14px;
                font-weight: 600;
              }
            }

            .team-name {
              flex: 1;
              font-size: 15px;
              font-weight: 500;
              color: #374151;
            }

            .points {
              font-size: 14px;
              font-weight: 600;
              color: #10b981;
            }

            &.loser-r2 {
              border-color: #a78bfa;
              background: #f5f3ff;
            }

            &.loser-r1 {
              border-color: #f9a8d4;
              background: #fdf2f8;
            }
          }
        }
      }
    }
  }

  .match-details-content {
    .match-type-badge {
      display: flex;
      gap: 8px;
      margin-bottom: 16px;
    }

    .teams-matchup {
      display: flex;
      align-items: center;
      gap: 24px;
      padding: 24px;
      background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
      border-radius: 12px;
      margin-bottom: 16px;

      .team-card {
        flex: 1;
        text-align: center;

        .team-name {
          font-size: 20px;
          font-weight: 600;
          color: #1f2937;
          margin-bottom: 8px;
        }

        .team-badge {
          font-size: 12px;
          color: #6b7280;
        }
      }

      .vs-divider {
        font-size: 18px;
        font-weight: 700;
        color: #9ca3af;
      }
    }

    .match-result,
    .match-pending {
      margin-bottom: 16px;
      text-align: center;

      .result-badge {
        margin-bottom: 12px;
      }

      .score-display {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 16px;
        font-size: 36px;
        font-weight: 700;

        .score-value {
          color: #6b7280;
          transition: all 0.3s;

          &.winner-score {
            color: #10b981;
            transform: scale(1.1);
          }
        }

        .score-separator {
          color: #d1d5db;
        }

        .score-label {
          font-size: 14px;
          color: #6b7280;
          margin: 0 8px;
        }
      }
    }

    .match-time {
      padding: 12px;
      background: #f9fafb;
      border-radius: 6px;
      margin-bottom: 16px;

      .label {
        color: #6b7280;
        margin-right: 8px;
      }

      .value {
        font-weight: 500;
        color: #374151;
      }
    }

    .dialog-actions {
      display: flex;
      justify-content: center;
      gap: 12px;
    }
  }
}

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

@keyframes trophy-shake {
  0% { transform: rotate(-5deg); }
  100% { transform: rotate(5deg); }
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
  }
  50% {
    transform: scale(1.05);
    box-shadow: 0 6px 20px rgba(245, 158, 11, 0.6);
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
    animation: trophy-shake 0.5s infinite alternate;
  }

  .el-message-box__content {
    font-size: 18px;
    color: #92400e;
  }

  .el-button--primary {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    border: none;
    animation: pulse 1.5s infinite;

    &:hover {
      background: linear-gradient(135deg, #d97706 0%, #b45309 100%);
    }
  }
}
</style>
