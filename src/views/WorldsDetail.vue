<template>
  <div class="worlds-management">
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
            S 世界赛 (World Championship)
          </h1>
          <p class="page-description">
            12支队伍（各赛区夏季赛冠亚季军），冠军直通淘汰赛，亚季军打瑞士轮小组赛
          </p>
        </div>
      </div>
      <div class="header-actions">
        <el-button
          v-if="worldsBracket.status === 'group_stage' && !isGroupStageComplete"
          type="primary"
          @click="batchSimulateSwissRound"
          :loading="simulatingSwiss"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingSwiss ? `模拟中 (${simulationProgress}%)` : '模拟瑞士轮' }}
        </el-button>
        <el-button
          v-if="worldsBracket.status === 'knockout_stage'"
          type="warning"
          @click="batchSimulateKnockout"
          :loading="simulatingKnockout"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingKnockout ? `模拟中 (${simulationProgress}%)` : '模拟淘汰赛' }}
        </el-button>
      </div>
    </div>

    <!-- 世界赛状态卡片 -->
    <div class="worlds-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>S 世界赛</h2>
          <el-tag :type="getStatusType(worldsBracket.status)" size="large">
            {{ getStatusText(worldsBracket.status) }}
          </el-tag>
        </div>
      </div>

      <!-- 参赛队伍统计 -->
      <div class="teams-stats">
        <el-statistic title="参赛队伍总数" :value="12" />
        <el-statistic title="直通淘汰赛" :value="4" suffix="队" />
        <el-statistic title="瑞士轮小组赛" :value="8" suffix="队" />
        <el-statistic title="淘汰赛名额" :value="8" suffix="队" />
      </div>

      <!-- 参赛队伍分组 -->
      <el-card v-if="worldsBracket.status !== 'not_started'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🎯 参赛队伍分组</span>
          </div>
        </template>

        <div class="teams-groups">
          <!-- 传奇组（直通淘汰赛） -->
          <div class="team-group legendary">
            <h3>
              <el-icon><Star /></el-icon>
              传奇组（夏季赛冠军）
            </h3>
            <div class="team-group-desc">直接晋级淘汰赛，保留半区种子位</div>
            <div class="team-list">
              <div v-for="team in directTeams" :key="team.teamId" class="team-item">
                <span class="team-name">{{ team.teamName }}</span>
                <div class="team-badges">
                  <el-tag :type="getRegionTagType(team.regionId)" size="small">
                    {{ team.regionName }}
                  </el-tag>
                  <el-tag v-if="team.quarterSlot" size="small" type="warning">
                    种子{{ team.quarterSlot }}
                  </el-tag>
                </div>
              </div>
            </div>
          </div>

          <!-- 挑战者组（参加瑞士轮） -->
          <div class="team-group challenger">
            <h3>
              <el-icon><Medal /></el-icon>
              挑战者组（夏季赛亚军+季军）
            </h3>
            <div class="team-group-desc">参加瑞士轮小组赛，争夺4个淘汰赛席位</div>
            <div class="team-list">
              <div v-for="team in groupStageTeams" :key="team.teamId" class="team-item">
                <span class="team-name">{{ team.teamName }}</span>
                <div class="team-badges">
                  <el-tag :type="getRegionTagType(team.regionId)" size="small">
                    {{ team.regionName }}
                  </el-tag>
                  <el-tag size="small" type="info">
                    {{ team.seed === 2 ? '亚军' : '季军' }}
                  </el-tag>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 小组赛（瑞士轮）阶段 -->
      <el-card v-if="worldsBracket.status !== 'not_started'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🇨🇭 小组赛 - 瑞士轮</span>
            <el-tag v-if="isGroupStageComplete" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <!-- 瑞士轮规则说明 -->
        <el-alert
          title="瑞士轮规则"
          type="info"
          :closable="false"
          show-icon
          class="swiss-info"
        >
          <ul>
            <li>2胜晋级淘汰赛，2败淘汰，最多3轮</li>
            <li>相同战绩队伍配对，已对战过的不再相遇</li>
            <li>BO1单场淘汰制</li>
          </ul>
        </el-alert>

        <!-- 瑞士轮积分榜 -->
        <div class="swiss-standings" v-if="swissStandings.length > 0">
          <h4>当前积分榜（第{{ currentSwissRound }}轮后）</h4>
          <el-table :data="swissStandings" stripe class="standings-table">
            <el-table-column label="排名" width="70" align="center">
              <template #default="{ row }">
                <span class="rank-number">{{ row.position }}</span>
              </template>
            </el-table-column>
            <el-table-column label="队伍" min-width="150">
              <template #default="{ row }">
                <div class="team-cell">
                  <span class="team-name">{{ row.teamName }}</span>
                  <el-tag :type="getRegionTagType(row.regionId)" size="small">
                    {{ row.regionName }}
                  </el-tag>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="战绩" width="80" align="center">
              <template #default="{ row }">
                <span class="record">{{ row.wins }}-{{ row.losses }}</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="100" align="center">
              <template #default="{ row }">
                <el-tag v-if="row.wins >= 2" type="success" size="small">已晋级</el-tag>
                <el-tag v-else-if="row.losses >= 2" type="danger" size="small">已淘汰</el-tag>
                <el-tag v-else type="info" size="small">进行中</el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- 瑞士轮比赛列表 -->
        <div class="swiss-matches">
          <el-tabs v-model="activeSwissRound" type="card">
            <el-tab-pane
              v-for="round in [1, 2, 3]"
              :key="round"
              :label="`第${round}轮`"
              :name="String(round)"
            >
              <WorldsSwissRound
                :matches="getSwissRoundMatches(round)"
                :round="round"
                @simulate-match="handleSimulateSwissMatch"
                @view-match="viewMatchDetails"
              />
            </el-tab-pane>
          </el-tabs>
        </div>

        <!-- 生成淘汰赛按钮 -->
        <div v-if="isGroupStageComplete && worldsBracket.status === 'group_stage'" class="generate-knockout-section">
          <el-alert
            title="瑞士轮已完成！"
            description="4支队伍以2胜晋级，4支队伍以2败淘汰。现在可以生成淘汰赛对阵。"
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
      <el-card v-if="worldsBracket.status === 'knockout_stage' || worldsBracket.status === 'completed'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🏅 淘汰赛</span>
            <el-tag v-if="worldsBracket.status === 'completed'" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <!-- 淘汰赛规则说明 -->
        <el-alert
          v-if="worldsBracket.status !== 'completed'"
          title="淘汰赛规则"
          type="info"
          :closable="false"
          show-icon
          class="knockout-info"
        >
          <ul>
            <li>八强赛：4个半区种子保护，确保冠军队不会提前相遇</li>
            <li>半决赛：4强争夺决赛席位</li>
            <li>季军赛：半决赛败者争夺第三名</li>
            <li>决赛：冠军争夺战，全部BO5</li>
          </ul>
        </el-alert>

        <!-- 淘汰赛对阵图 -->
        <div class="knockout-brackets">
          <WorldsKnockoutBracket
            :knockout-matches="knockoutMatches"
            :third-place-match="thirdPlaceMatch"
            :grand-final="grandFinal"
            @simulate-match="handleSimulateKnockoutMatch"
            @view-match="viewMatchDetails"
          />
        </div>
      </el-card>

      <!-- 最终排名 -->
      <div v-if="worldsBracket.status === 'completed'" class="final-standings">
        <h3>最终排名与积分</h3>
        <div class="standings-grid">
          <div class="standing-item champion">
            <div class="rank-badge">🏆 冠军</div>
            <div class="team-name">{{ worldsBracket.champion?.teamName }}</div>
            <div class="region-name">{{ worldsBracket.champion?.regionName }}</div>
            <div class="points">+20分</div>
          </div>

          <div class="standing-item runner-up">
            <div class="rank-badge">🥈 亚军</div>
            <div class="team-name">{{ worldsBracket.runnerUp?.teamName }}</div>
            <div class="region-name">{{ worldsBracket.runnerUp?.regionName }}</div>
            <div class="points">+16分</div>
          </div>

          <div class="standing-item third">
            <div class="rank-badge">🥉 季军</div>
            <div class="team-name">{{ worldsBracket.thirdPlace?.teamName }}</div>
            <div class="region-name">{{ worldsBracket.thirdPlace?.regionName }}</div>
            <div class="points">+12分</div>
          </div>

          <div class="standing-item fourth">
            <div class="rank-badge">4️⃣ 殿军</div>
            <div class="team-name">{{ worldsBracket.fourthPlace?.teamName }}</div>
            <div class="region-name">{{ worldsBracket.fourthPlace?.regionName }}</div>
            <div class="points">+8分</div>
          </div>
        </div>

        <!-- 其他排名 -->
        <div class="other-rankings">
          <h4>八强止步（+6分）</h4>
          <div class="teams-list">
            <div
              v-for="(team, index) in worldsBracket.quarterFinalists"
              :key="index"
              class="team-chip"
            >
              {{ team?.teamName }} ({{ team?.regionName }})
            </div>
          </div>

          <h4>小组赛止步（+4分）</h4>
          <div class="teams-list">
            <div
              v-for="(team, index) in worldsBracket.groupStageEliminated"
              :key="index"
              class="team-chip eliminated"
            >
              {{ team?.teamName }} ({{ team?.regionName }})
            </div>
          </div>
        </div>

        <!-- 世界赛完成后的操作区 -->
        <div class="worlds-completed-actions">
          <el-alert
            title="世界赛已完成！"
            type="success"
            :closable="false"
            show-icon
            class="completion-alert"
          >
            <template #default>
              <p>恭喜 <strong>{{ worldsBracket.champion?.teamName }}</strong> 获得世界赛冠军，成为世界最强战队！</p>
            </template>
          </el-alert>
        </div>
      </div>
    </div>

    <!-- 比赛详情弹窗 -->
    <MatchDetailDialog
      v-model="showMatchDetailDialog"
      :match-detail="currentMatchDetail"
      @close="handleCloseMatchDetail"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Trophy,
  ArrowLeft,
  DArrowRight,
  Plus,
  Star,
  Medal
} from '@element-plus/icons-vue'
import WorldsSwissRound from '@/components/worlds/WorldsSwissRound.vue'
import WorldsKnockoutBracket from '@/components/worlds/WorldsKnockoutBracket.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { PowerEngine } from '@/engines/PowerEngine'
import type { MatchDetail } from '@/types/matchDetail'
import type { Player, PlayerPosition } from '@/types/player'
import type { WorldsQualification, SwissStandings, WorldsMatch, WorldsSwissMatch, WorldsKnockoutMatch } from '@/types/index'

const router = useRouter()
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()

// 响应式状态
const generatingKnockout = ref(false)
const simulatingSwiss = ref(false)
const simulatingKnockout = ref(false)
const simulationProgress = ref(0)
const activeSwissRound = ref('1')
const currentSwissRound = ref(1)

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 生成模拟的参赛队伍数据
const generateTeamsData = () => {
  const lplTeams = [
    { name: 'JDG', seed: 1 },
    { name: 'BLG', seed: 2 },
    { name: 'TES', seed: 3 }
  ]
  const lckTeams = [
    { name: 'T1', seed: 1 },
    { name: 'GEN', seed: 2 },
    { name: 'DK', seed: 3 }
  ]
  const lecTeams = [
    { name: 'G2', seed: 1 },
    { name: 'FNC', seed: 2 },
    { name: 'MAD', seed: 3 }
  ]
  const lcsTeams = [
    { name: 'C9', seed: 1 },
    { name: 'TL', seed: 2 },
    { name: '100T', seed: 3 }
  ]

  const allTeams: WorldsQualification[] = []
  const regions = [
    { id: 'LPL', name: '中国赛区', teams: lplTeams },
    { id: 'LCK', name: '韩国赛区', teams: lckTeams },
    { id: 'LEC', name: '欧洲赛区', teams: lecTeams },
    { id: 'LCS', name: '北美赛区', teams: lcsTeams }
  ]

  let quarterSlot = 1
  regions.forEach((region) => {
    region.teams.forEach((team) => {
      const qualification: WorldsQualification = {
        teamId: `${region.id.toLowerCase()}-${team.name.toLowerCase()}`,
        teamName: team.name,
        regionId: region.id,
        regionName: region.name,
        seed: team.seed,
        summerPlayoffRank: team.seed,
        summerPlayoffPoints: (4 - team.seed) * 100,
        directToKnockout: team.seed === 1,
        quarterSlot: team.seed === 1 ? quarterSlot++ : undefined
      }
      allTeams.push(qualification)
    })
  })

  return allTeams
}

// 生成瑞士轮比赛数据
const generateSwissMatches = (groupTeams: WorldsQualification[]): WorldsSwissMatch[] => {
  const matches: WorldsSwissMatch[] = []

  // 第一轮：随机配对
  const shuffled = [...groupTeams].sort(() => Math.random() - 0.5)
  for (let i = 0; i < shuffled.length; i += 2) {
    matches.push({
      id: `swiss-r1-${i / 2 + 1}`,
      matchType: 'swiss_round',
      stage: 'group',
      bestOf: 1,
      swissRound: 1,
      roundNumber: 1,
      teamAId: shuffled[i].teamId,
      teamAName: shuffled[i].teamName,
      teamBId: shuffled[i + 1].teamId,
      teamBName: shuffled[i + 1].teamName,
      status: 'scheduled'
    })
  }

  return matches
}

// 生成积分榜数据
const generateSwissStandings = (groupTeams: WorldsQualification[]): SwissStandings[] => {
  return groupTeams.map((team, index) => ({
    teamId: team.teamId,
    teamName: team.teamName,
    regionId: team.regionId,
    regionName: team.regionName,
    wins: 0,
    losses: 0,
    record: '0-0',
    matchesPlayed: 0,
    status: 'active' as const,
    qualified: false,
    eliminated: false,
    position: index + 1,
    currentRound: 1
  }))
}

/**
 * 生成队伍选手数据
 */
const generateTeamPlayers = (teamId: string, teamName: string, regionId: string): Player[] => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  return positions.map((pos, idx) => ({
    id: `${teamId}-${pos}`,
    gameId: `Player${idx + 1}`,
    name: `Player${idx + 1}`,
    teamId: teamId,
    teamName: teamName,
    position: pos,
    regionId: regionId,
    regionName: getRegionName(regionId),
    ability: 70 + Math.floor(Math.random() * 25),
    potential: 80 + Math.floor(Math.random() * 15),
    stability: 60 + Math.floor(Math.random() * 35),
    condition: Math.floor(Math.random() * 11) - 5,
    age: 18 + Math.floor(Math.random() * 10),
    tag: Math.random() > 0.7 ? 'GENIUS' : Math.random() > 0.4 ? 'NORMAL' : 'ORDINARY'
  } as Player))
}

/**
 * 获取赛区名称
 */
const getRegionName = (regionId: string): string => {
  const regionMap: Record<string, string> = {
    'LPL': '中国赛区',
    'LCK': '韩国赛区',
    'LEC': '欧洲赛区',
    'LCS': '北美赛区'
  }
  return regionMap[regionId] || regionId
}

// 世界赛数据
const allTeams = generateTeamsData()
const directTeams = computed(() => allTeams.filter(t => t.directToKnockout))
const groupStageTeams = computed(() => allTeams.filter(t => !t.directToKnockout))

const worldsBracket = reactive({
  id: '1',
  seasonId: 'S1',
  seasonYear: 2024,
  status: 'group_stage' as 'not_started' | 'group_stage' | 'knockout_stage' | 'completed',
  qualifiedTeams: allTeams,
  directTeams: directTeams.value,
  groupStageTeams: groupStageTeams.value,
  swissMatches: generateSwissMatches(groupStageTeams.value),
  swissStandings: generateSwissStandings(groupStageTeams.value),
  knockoutMatches: [] as WorldsKnockoutMatch[],
  champion: null as WorldsQualification | null,
  runnerUp: null as WorldsQualification | null,
  thirdPlace: null as WorldsQualification | null,
  fourthPlace: null as WorldsQualification | null,
  quarterFinalists: [] as WorldsQualification[],
  groupStageEliminated: [] as WorldsQualification[],
  pointsDistribution: {
    champion: 20,
    runnerUp: 16,
    thirdPlace: 12,
    fourthPlace: 8,
    quarterFinalist: 6,
    groupStageEliminated: 4
  }
})

// 计算属性
const swissStandings = computed(() => worldsBracket.swissStandings)

const isGroupStageComplete = computed(() => {
  const qualified = swissStandings.value.filter(s => s.wins >= 2).length
  const eliminated = swissStandings.value.filter(s => s.losses >= 2).length
  return qualified >= 4 && eliminated >= 4
})

const knockoutMatches = computed(() =>
  worldsBracket.knockoutMatches.filter(m =>
    m.round === 'QUARTER_FINAL' || m.round === 'SEMI_FINAL'
  )
)

const thirdPlaceMatch = computed(() =>
  worldsBracket.knockoutMatches.find(m => m.round === 'THIRD_PLACE')
)

const grandFinal = computed(() =>
  worldsBracket.knockoutMatches.find(m => m.round === 'FINAL')
)

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

const getRegionTagType = (regionId?: string) => {
  const typeMap: Record<string, any> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning'
  }
  return typeMap[regionId || ''] || 'info'
}

const getSwissRoundMatches = (round: number): WorldsSwissMatch[] => {
  return worldsBracket.swissMatches.filter(m => m.swissRound === round)
}

/**
 * 模拟瑞士轮单场比赛（使用PowerEngine）
 */
const handleSimulateSwissMatch = async (match: WorldsSwissMatch) => {
  // 获取队伍赛区信息
  const teamAInfo = allTeams.find(t => t.teamId === match.teamAId)
  const teamBInfo = allTeams.find(t => t.teamId === match.teamBId)
  const regionAId = teamAInfo?.regionId || 'INTL'
  const regionBId = teamBInfo?.regionId || 'INTL'

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(match.teamAId, match.teamAName, regionAId)
  const teamBPlayers = generateTeamPlayers(match.teamBId, match.teamBName, regionBId)

  // 使用 PowerEngine 模拟比赛 (BO1)
  const matchDetail = PowerEngine.simulateMatch(
    match.teamAId,
    match.teamAName,
    teamAPlayers,
    match.teamBId,
    match.teamBName,
    teamBPlayers,
    1 // BO1
  )

  // 更新比赛结果
  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'

  // 保存比赛详情
  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'worlds'
  matchDetail.seasonId = String(worldsBracket.seasonYear)
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
        String(worldsBracket.seasonYear),
        regionAId
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
        String(worldsBracket.seasonYear),
        regionBId
      )
    })
  })

  // 更新积分榜
  updateSwissStandings(match)

  ElMessage.success(`比赛完成: ${match.teamAName} ${match.scoreA} - ${match.scoreB} ${match.teamBName}`)

  // 检查当前轮是否完成，生成下一轮
  checkSwissRoundCompletion()
}

/**
 * 更新瑞士轮积分榜
 */
const updateSwissStandings = (match: WorldsSwissMatch) => {
  const teamA = worldsBracket.swissStandings.find(s => s.teamId === match.teamAId)
  const teamB = worldsBracket.swissStandings.find(s => s.teamId === match.teamBId)

  if (teamA && teamB) {
    teamA.matchesPlayed++
    teamB.matchesPlayed++

    if (match.winnerId === match.teamAId) {
      teamA.wins++
      teamB.losses++
    } else {
      teamB.wins++
      teamA.losses++
    }

    teamA.record = `${teamA.wins}-${teamA.losses}`
    teamB.record = `${teamB.wins}-${teamB.losses}`

    // 更新状态
    if (teamA.wins >= 2) {
      teamA.status = 'qualified'
      teamA.qualified = true
    } else if (teamA.losses >= 2) {
      teamA.status = 'eliminated'
      teamA.eliminated = true
    }

    if (teamB.wins >= 2) {
      teamB.status = 'qualified'
      teamB.qualified = true
    } else if (teamB.losses >= 2) {
      teamB.status = 'eliminated'
      teamB.eliminated = true
    }

    // 重新排序
    worldsBracket.swissStandings.sort((a, b) => {
      if (b.wins !== a.wins) return b.wins - a.wins
      if (a.losses !== b.losses) return a.losses - b.losses
      return 0
    })

    worldsBracket.swissStandings.forEach((s, i) => {
      s.position = i + 1
    })
  }
}

/**
 * 检查瑞士轮轮次是否完成
 */
const checkSwissRoundCompletion = () => {
  const currentRoundMatches = getSwissRoundMatches(currentSwissRound.value)
  const allComplete = currentRoundMatches.every(m => m.status === 'completed')

  if (allComplete && currentSwissRound.value < 3 && !isGroupStageComplete.value) {
    // 生成下一轮对阵
    generateNextSwissRound()
  }
}

/**
 * 生成下一轮瑞士轮对阵
 */
const generateNextSwissRound = () => {
  const nextRound = currentSwissRound.value + 1
  const activeTeams = worldsBracket.swissStandings.filter(
    s => s.status === 'active'
  )

  // 按战绩分组配对
  const grouped: Record<string, SwissStandings[]> = {}
  activeTeams.forEach(team => {
    const key = team.record
    if (!grouped[key]) grouped[key] = []
    grouped[key].push(team)
  })

  const newMatches: WorldsSwissMatch[] = []
  let matchNum = 1

  Object.values(grouped).forEach(teams => {
    const shuffled = [...teams].sort(() => Math.random() - 0.5)
    for (let i = 0; i < shuffled.length; i += 2) {
      if (shuffled[i + 1]) {
        newMatches.push({
          id: `swiss-r${nextRound}-${matchNum++}`,
          matchType: 'swiss_round',
          stage: 'group',
          bestOf: 1,
          swissRound: nextRound,
          roundNumber: nextRound,
          teamAId: shuffled[i].teamId,
          teamAName: shuffled[i].teamName,
          teamBId: shuffled[i + 1].teamId,
          teamBName: shuffled[i + 1].teamName,
          status: 'scheduled'
        })
      }
    }
  })

  worldsBracket.swissMatches.push(...newMatches)
  currentSwissRound.value = nextRound
  activeSwissRound.value = String(nextRound)
}

/**
 * 生成淘汰赛对阵
 */
const handleGenerateKnockout = async () => {
  generatingKnockout.value = true

  try {
    await new Promise(resolve => setTimeout(resolve, 500))

    // 获取晋级的4支队伍
    const qualifiedFromSwiss = worldsBracket.swissStandings
      .filter(s => s.qualified)
      .map(s => worldsBracket.groupStageTeams.find(t => t.teamId === s.teamId)!)

    // 记录小组赛淘汰队伍
    worldsBracket.groupStageEliminated = worldsBracket.swissStandings
      .filter(s => s.eliminated)
      .map(s => worldsBracket.groupStageTeams.find(t => t.teamId === s.teamId)!)

    // 生成八强赛对阵（4个冠军 vs 4个小组赛晋级）
    const quarterFinals: WorldsKnockoutMatch[] = []

    directTeams.value.forEach((champion, index) => {
      const opponent = qualifiedFromSwiss[index]
      quarterFinals.push({
        id: `qf-${index + 1}`,
        matchType: 'quarter_final',
        stage: 'knockout',
        bestOf: 5,
        round: 'QUARTER_FINAL',
        quarterSlot: champion.quarterSlot,
        teamAId: champion.teamId,
        teamAName: champion.teamName,
        teamBId: opponent.teamId,
        teamBName: opponent.teamName,
        status: 'scheduled'
      })
    })

    // 生成半决赛占位
    const semiFinals: WorldsKnockoutMatch[] = [
      {
        id: 'sf-1',
        matchType: 'semi_final',
        stage: 'knockout',
        bestOf: 5,
        round: 'SEMI_FINAL',
        teamAId: '',
        teamAName: '待定',
        teamBId: '',
        teamBName: '待定',
        status: 'scheduled'
      },
      {
        id: 'sf-2',
        matchType: 'semi_final',
        stage: 'knockout',
        bestOf: 5,
        round: 'SEMI_FINAL',
        teamAId: '',
        teamAName: '待定',
        teamBId: '',
        teamBName: '待定',
        status: 'scheduled'
      }
    ]

    // 生成季军赛
    const thirdPlace: WorldsKnockoutMatch = {
      id: 'third',
      matchType: 'third_place',
      stage: 'third_place',
      bestOf: 5,
      round: 'THIRD_PLACE',
      teamAId: '',
      teamAName: '待定',
      teamBId: '',
      teamBName: '待定',
      status: 'scheduled'
    }

    // 生成总决赛
    const final: WorldsKnockoutMatch = {
      id: 'final',
      matchType: 'grand_final',
      stage: 'knockout',
      bestOf: 5,
      round: 'FINAL',
      teamAId: '',
      teamAName: '待定',
      teamBId: '',
      teamBName: '待定',
      status: 'scheduled'
    }

    worldsBracket.knockoutMatches = [...quarterFinals, ...semiFinals, thirdPlace, final]
    worldsBracket.status = 'knockout_stage'
    ElMessage.success('淘汰赛对阵生成成功!')
  } finally {
    generatingKnockout.value = false
  }
}

/**
 * 模拟淘汰赛单场比赛（使用PowerEngine）
 */
const handleSimulateKnockoutMatch = async (match: WorldsKnockoutMatch) => {
  // 获取队伍赛区信息
  const teamAInfo = allTeams.find(t => t.teamId === match.teamAId)
  const teamBInfo = allTeams.find(t => t.teamId === match.teamBId)
  const regionAId = teamAInfo?.regionId || 'INTL'
  const regionBId = teamBInfo?.regionId || 'INTL'

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(match.teamAId, match.teamAName, regionAId)
  const teamBPlayers = generateTeamPlayers(match.teamBId, match.teamBName, regionBId)

  // 使用 PowerEngine 模拟比赛 (BO5)
  const matchDetail = PowerEngine.simulateMatch(
    match.teamAId,
    match.teamAName,
    teamAPlayers,
    match.teamBId,
    match.teamBName,
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
  matchDetail.tournamentType = 'worlds'
  matchDetail.seasonId = String(worldsBracket.seasonYear)
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
        String(worldsBracket.seasonYear),
        regionAId
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
        String(worldsBracket.seasonYear),
        regionBId
      )
    })
  })

  ElMessage.success(`比赛完成: ${match.teamAName} ${matchDetail.finalScoreA} - ${matchDetail.finalScoreB} ${match.teamBName}`)

  // 更新后续对阵
  updateKnockoutBracket(match)

  // 检查是否完成
  checkKnockoutCompletion()
}

/**
 * 更新淘汰赛对阵
 */
const updateKnockoutBracket = (match: WorldsKnockoutMatch) => {
  const winnerTeam = match.winnerId === match.teamAId
    ? { id: match.teamAId, name: match.teamAName }
    : { id: match.teamBId, name: match.teamBName }
  const loserTeam = match.winnerId === match.teamAId
    ? { id: match.teamBId, name: match.teamBName }
    : { id: match.teamAId, name: match.teamAName }

  if (match.round === 'QUARTER_FINAL') {
    // 记录八强淘汰
    const loser = worldsBracket.qualifiedTeams.find(t => t.teamId === loserTeam.id)
    if (loser) worldsBracket.quarterFinalists.push(loser)

    // 更新半决赛对阵
    const qfIndex = worldsBracket.knockoutMatches.filter(m => m.round === 'QUARTER_FINAL' && m.status === 'completed').length
    const sfIndex = Math.floor((qfIndex - 1) / 2)
    const semiFinal = worldsBracket.knockoutMatches.find(m => m.round === 'SEMI_FINAL' && m.id === `sf-${sfIndex + 1}`)

    if (semiFinal) {
      if (!semiFinal.teamAId) {
        semiFinal.teamAId = winnerTeam.id
        semiFinal.teamAName = winnerTeam.name
      } else {
        semiFinal.teamBId = winnerTeam.id
        semiFinal.teamBName = winnerTeam.name
      }
    }
  } else if (match.round === 'SEMI_FINAL') {
    // 更新总决赛和季军赛
    const final = worldsBracket.knockoutMatches.find(m => m.round === 'FINAL')
    const thirdPlace = worldsBracket.knockoutMatches.find(m => m.round === 'THIRD_PLACE')

    if (final) {
      if (!final.teamAId) {
        final.teamAId = winnerTeam.id
        final.teamAName = winnerTeam.name
      } else {
        final.teamBId = winnerTeam.id
        final.teamBName = winnerTeam.name
      }
    }

    if (thirdPlace) {
      if (!thirdPlace.teamAId) {
        thirdPlace.teamAId = loserTeam.id
        thirdPlace.teamAName = loserTeam.name
      } else {
        thirdPlace.teamBId = loserTeam.id
        thirdPlace.teamBName = loserTeam.name
      }
    }
  }
}

/**
 * 检查淘汰赛是否完成
 */
const checkKnockoutCompletion = () => {
  const final = worldsBracket.knockoutMatches.find(m => m.round === 'FINAL')
  const thirdPlace = worldsBracket.knockoutMatches.find(m => m.round === 'THIRD_PLACE')

  if (final?.status === 'completed' && thirdPlace?.status === 'completed') {
    // 设置最终排名
    worldsBracket.champion = worldsBracket.qualifiedTeams.find(
      t => t.teamId === final.winnerId
    ) || null
    worldsBracket.runnerUp = worldsBracket.qualifiedTeams.find(
      t => t.teamId === (final.winnerId === final.teamAId ? final.teamBId : final.teamAId)
    ) || null
    worldsBracket.thirdPlace = worldsBracket.qualifiedTeams.find(
      t => t.teamId === thirdPlace.winnerId
    ) || null
    worldsBracket.fourthPlace = worldsBracket.qualifiedTeams.find(
      t => t.teamId === (thirdPlace.winnerId === thirdPlace.teamAId ? thirdPlace.teamBId : thirdPlace.teamAId)
    ) || null

    worldsBracket.status = 'completed'
    showChampionCelebration(worldsBracket.champion?.teamName || '')
  }
}

/**
 * 批量模拟瑞士轮
 */
const batchSimulateSwissRound = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的瑞士轮比赛。是否继续?',
      '模拟瑞士轮确认',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    simulatingSwiss.value = true
    simulationProgress.value = 0

    while (!isGroupStageComplete.value && currentSwissRound.value <= 3) {
      const currentMatches = getSwissRoundMatches(currentSwissRound.value)
      const uncompletedMatches = currentMatches.filter(m => m.status !== 'completed')

      for (let i = 0; i < uncompletedMatches.length; i++) {
        const match = uncompletedMatches[i]
        const winnerId = Math.random() > 0.5 ? match.teamAId : match.teamBId
        match.winnerId = winnerId
        match.scoreA = winnerId === match.teamAId ? 1 : 0
        match.scoreB = winnerId === match.teamBId ? 1 : 0
        match.status = 'completed'
        updateSwissStandings(match)
        await new Promise(resolve => setTimeout(resolve, 80))
      }

      checkSwissRoundCompletion()
      simulationProgress.value = Math.floor((currentSwissRound.value / 3) * 100)
    }

    ElMessage.success('瑞士轮模拟完成！')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('瑞士轮模拟失败:', error)
      ElMessage.error('瑞士轮模拟失败')
    }
  } finally {
    simulatingSwiss.value = false
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

    const stages = ['QUARTER_FINAL', 'SEMI_FINAL', 'THIRD_PLACE', 'FINAL']

    for (let stageIdx = 0; stageIdx < stages.length; stageIdx++) {
      const stage = stages[stageIdx]
      const stageMatches = worldsBracket.knockoutMatches.filter(m => m.round === stage)

      for (const match of stageMatches) {
        if (match.status !== 'completed' && match.teamAId && match.teamBId) {
          const scoreA = Math.random() > 0.5 ? 3 : Math.floor(Math.random() * 3)
          const scoreB = scoreA === 3 ? Math.floor(Math.random() * 3) : 3
          match.scoreA = scoreA
          match.scoreB = scoreB
          match.winnerId = scoreA > scoreB ? match.teamAId : match.teamBId
          match.status = 'completed'
          updateKnockoutBracket(match)
          await new Promise(resolve => setTimeout(resolve, 150))
        }
      }

      simulationProgress.value = Math.floor(((stageIdx + 1) / stages.length) * 100)
    }

    checkKnockoutCompletion()
    ElMessage.success('淘汰赛模拟完成！')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('淘汰赛模拟失败:', error)
      ElMessage.error('淘汰赛模拟失败')
    }
  } finally {
    simulatingKnockout.value = false
    simulationProgress.value = 0
  }
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
 * 显示冠军庆祝动画
 */
const showChampionCelebration = (championName: string) => {
  ElMessageBox.alert(
    `恭喜 ${championName} 获得世界赛冠军，成为全球最强战队!`,
    '🏆 世界赛冠军诞生! 🏆',
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
.worlds-management {
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
      align-items: center;
      gap: 12px;
    }
  }

  .worlds-status-card {
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
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
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

    .teams-groups {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 16px;
      margin: 20px 0;

      .team-group {
        padding: 20px;
        border-radius: 12px;
        border: 2px solid;
        transition: all 0.3s ease;

        &:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        h3 {
          display: flex;
          align-items: center;
          gap: 8px;
          margin: 0 0 8px 0;
          font-size: 18px;
          font-weight: 700;
        }

        .team-group-desc {
          font-size: 13px;
          margin-bottom: 16px;
          opacity: 0.8;
        }

        .team-list {
          display: flex;
          flex-direction: column;
          gap: 10px;

          .team-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 16px;
            background: white;
            border-radius: 8px;
            transition: all 0.2s ease;

            &:hover {
              transform: translateX(4px);
              box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
            }

            .team-name {
              font-weight: 600;
              font-size: 15px;
              color: #374151;
            }

            .team-badges {
              display: flex;
              gap: 6px;
              align-items: center;
            }
          }
        }

        &.legendary {
          border-color: #f59e0b;
          background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);

          h3 {
            color: #d97706;
          }

          .team-group-desc {
            color: #92400e;
          }
        }

        &.challenger {
          border-color: #3b82f6;
          background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);

          h3 {
            color: #2563eb;
          }

          .team-group-desc {
            color: #1e40af;
          }
        }
      }
    }

    .swiss-info {
      margin-bottom: 20px;

      ul {
        margin: 10px 0 0 0;
        padding-left: 20px;
      }

      li {
        margin: 5px 0;
      }
    }

    .swiss-standings {
      margin: 20px 0;

      h4 {
        margin: 0 0 12px 0;
        font-size: 16px;
        font-weight: 600;
        color: #374151;
      }

      .standings-table {
        .team-cell {
          display: flex;
          align-items: center;
          gap: 8px;

          .team-name {
            font-weight: 600;
            color: #1f2937;
          }
        }

        .rank-number {
          font-weight: 600;
        }

        .record {
          font-weight: 600;
          color: #374151;
        }
      }
    }

    .swiss-matches {
      margin-top: 20px;
    }

    .generate-knockout-section {
      margin-top: 24px;
      text-align: center;

      .el-button {
        margin-top: 16px;
      }
    }

    .knockout-info {
      margin-bottom: 20px;

      ul {
        margin: 10px 0 0 0;
        padding-left: 20px;
      }

      li {
        margin: 5px 0;
      }
    }

    .knockout-brackets {
      margin-top: 24px;
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
          border-radius: 12px;
          text-align: center;
          border: 2px solid;
          transition: transform 0.2s;

          &:hover {
            transform: translateY(-4px);
          }

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

      .other-rankings {
        margin-bottom: 32px;

        h4 {
          font-size: 16px;
          font-weight: 600;
          color: #374151;
          margin: 20px 0 12px 0;
        }

        .teams-list {
          display: flex;
          flex-wrap: wrap;
          gap: 12px;

          .team-chip {
            padding: 8px 16px;
            background: linear-gradient(135deg, #f3f4f6 0%, #e5e7eb 100%);
            border: 1px solid #d1d5db;
            border-radius: 20px;
            font-size: 14px;
            color: #374151;
            transition: all 0.2s;

            &:hover {
              background: linear-gradient(135deg, #e5e7eb 0%, #d1d5db 100%);
              transform: translateY(-2px);
            }

            &.eliminated {
              opacity: 0.7;
            }
          }
        }
      }

      .worlds-completed-actions {
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
