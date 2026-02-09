<template>
  <div class="msi-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><Trophy /></el-icon>
          MSI季中邀请赛
        </h1>
        <p class="page-description">
          12支队伍(各赛区春季赛冠亚季军)参赛,双败淘汰赛制,决出世界最强战队
        </p>
      </div>
      <div class="header-actions">
        <el-button @click="refreshData" :icon="Refresh">刷新数据</el-button>
      </div>
    </div>

    <!-- MSI状态卡片 -->
    <div v-if="currentMSIBracket" class="msi-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>{{ currentMSIBracket.seasonYear }} MSI季中邀请赛</h2>
          <el-tag :type="getStatusType(currentMSIBracket.status)" size="large">
            {{ getStatusText(currentMSIBracket.status) }}
          </el-tag>
        </div>
        <div class="status-actions">
          <el-button
            v-if="hasRealTeamData && currentMSIBracket.status !== 'completed'"
            type="warning"
            @click="batchSimulateMSI"
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
          <h3><el-icon><Star /></el-icon> 传奇组 (春季赛冠军)</h3>
          <div class="team-list">
            <template v-if="hasRealTeamData && legendaryGroupTeams.length > 0">
              <div
                v-for="team in legendaryGroupTeams"
                :key="team.teamId"
                class="team-item"
              >
                <span class="team-name">{{ team.teamName }}</span>
                <el-tag size="small">{{ team.regionName }}</el-tag>
              </div>
            </template>
            <div v-else class="team-item pending">
              <span class="team-name">待春季赛结束后确定</span>
            </div>
          </div>
        </div>

        <div class="team-group challenger">
          <h3><el-icon><Medal /></el-icon> 挑战者组 (春季赛亚军)</h3>
          <div class="team-list">
            <template v-if="hasRealTeamData && challengerGroupTeams.length > 0">
              <div
                v-for="team in challengerGroupTeams"
                :key="team.teamId"
                class="team-item"
              >
                <span class="team-name">{{ team.teamName }}</span>
                <el-tag size="small">{{ team.regionName }}</el-tag>
              </div>
            </template>
            <div v-else class="team-item pending">
              <span class="team-name">待春季赛结束后确定</span>
            </div>
          </div>
        </div>

        <div class="team-group qualifier">
          <h3><el-icon><Flag /></el-icon> 资格赛组 (春季赛季军)</h3>
          <div class="team-list">
            <template v-if="hasRealTeamData && qualifierGroupTeams.length > 0">
              <div
                v-for="team in qualifierGroupTeams"
                :key="team.teamId"
                class="team-item"
              >
                <span class="team-name">{{ team.teamName }}</span>
                <el-tag size="small">{{ team.regionName }}</el-tag>
              </div>
            </template>
            <div v-else class="team-item pending">
              <span class="team-name">待春季赛结束后确定</span>
            </div>
          </div>
        </div>
      </div>

      <!-- MSI对阵图 -->
      <div class="bracket-section">
        <template v-if="hasRealTeamData">
          <MSIBracketView
            v-if="currentMSIBracket"
            :bracket="currentMSIBracket"
            @simulate-match="simulateMSIMatch"
            @view-match="viewMatchDetails"
          />
        </template>
        <div v-else class="bracket-placeholder">
          <el-empty description="赛事尚未开始">
            <template #image>
              <el-icon :size="64" color="#c0c4cc"><Trophy /></el-icon>
            </template>
            <p class="placeholder-text">请先完成春季赛季后赛，确定参赛队伍后才能进行MSI对阵</p>
          </el-empty>
        </div>
      </div>

      <!-- 最终排名 -->
      <TournamentCompletionSection
        v-if="currentMSIBracket.status === 'completed'"
        :standings="msiStandings"
        banner-title="MSI季中邀请赛已完成！"
        :banner-champion="currentMSIBracket.champion?.teamName || ''"
        banner-description="获得MSI冠军！"
      >
        <div v-if="currentMSIBracket.loserRound2?.length > 0" class="loser-standings">
          <h4>败者组第二轮 (5-6名)</h4>
          <div class="loser-grid">
            <div v-for="(team, index) in currentMSIBracket.loserRound2" :key="team.teamId" class="loser-item loser-r2">
              <div class="rank-badge"><span class="rank-number">{{ 5 + Number(index) }}</span></div>
              <div class="team-name">{{ team.teamName }}</div>
              <div class="points">+{{ currentMSIBracket.pointsDistribution.loserRound2 }}分</div>
            </div>
          </div>
        </div>
        <div v-if="currentMSIBracket.loserRound1?.length > 0" class="loser-standings">
          <h4>败者组第一轮 (7-8名)</h4>
          <div class="loser-grid">
            <div v-for="(team, index) in currentMSIBracket.loserRound1" :key="team.teamId" class="loser-item loser-r1">
              <div class="rank-badge"><span class="rank-number">{{ 7 + Number(index) }}</span></div>
              <div class="team-name">{{ team.teamName }}</div>
              <div class="points">+{{ currentMSIBracket.pointsDistribution.loserRound1 }}分</div>
            </div>
          </div>
        </div>
      </TournamentCompletionSection>
    </div>

    <!-- 比赛详情对话框 -->
    <el-dialog v-model="showMatchDetails" title="比赛详情" width="700px">
      <div v-if="selectedMatch" class="match-details-content">
        <!-- 比赛类型 -->
        <div class="match-type-badge">
          <el-tag :type="getMatchTypeBadgeType(selectedMatch.matchType)">
            {{ getMatchTypeName(selectedMatch.matchType) }}
          </el-tag>
          <el-tag type="info">BO{{ selectedMatch.bestOf }}</el-tag>
        </div>

        <!-- 对阵双方 -->
        <div class="teams-matchup">
          <div class="team-card">
            <div class="team-name">{{ getTeamName(getTeamAId(selectedMatch)) }}</div>
            <div class="team-badge">队伍A</div>
          </div>
          <div class="vs-divider">VS</div>
          <div class="team-card">
            <div class="team-name">{{ getTeamName(getTeamBId(selectedMatch)) }}</div>
            <div class="team-badge">队伍B</div>
          </div>
        </div>

        <!-- 比赛结果 -->
        <div v-if="selectedMatch.status === 'completed' && hasMatchResult(selectedMatch)" class="match-result">
          <div class="result-badge">
            <el-tag type="success" size="large">已完成</el-tag>
          </div>
          <div class="score-display">
            <span class="team-score">
              <span class="score-label">{{ getTeamName(getTeamAId(selectedMatch)) }}</span>
              <span
                class="score-value"
                :class="{ 'winner-score': isMatchWinner(selectedMatch, getTeamAId(selectedMatch)) }"
              >
                {{ getMatchScoreA(selectedMatch) }}
              </span>
            </span>
            <span class="score-separator">-</span>
            <span class="team-score">
              <span
                class="score-value"
                :class="{ 'winner-score': isMatchWinner(selectedMatch, getTeamBId(selectedMatch)) }"
              >
                {{ getMatchScoreB(selectedMatch) }}
              </span>
              <span class="score-label">{{ getTeamName(getTeamBId(selectedMatch)) }}</span>
            </span>
          </div>
        </div>
        <div v-else class="match-pending">
          <el-tag type="info">待模拟</el-tag>
        </div>

        <!-- 比赛时间 -->
        <div class="match-time">
          <span class="label">比赛时间:</span>
          <span class="value">{{ formatDate(selectedMatch.playedAt || selectedMatch.scheduledAt) }}</span>
        </div>

        <!-- 操作按钮 -->
        <div v-if="selectedMatch.status !== 'completed'" class="dialog-actions">
          <el-button
            type="primary"
            @click="simulateCurrentMatch"
            :loading="simulating"
          >
            模拟此场比赛
          </el-button>
        </div>
      </div>
    </el-dialog>

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
import { useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Trophy,
  Refresh,
  Promotion,
  Star,
  Medal,
  Flag
} from '@element-plus/icons-vue'
import MSIBracketView from '@/components/msi/MSIBracketView.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import TournamentCompletionSection from '@/components/common/TournamentCompletionSection.vue'
import type { StandingItem } from '@/types/tournament'
import { PowerEngine } from '@/engines/PowerEngine'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, queryApi, type BracketInfo, type MatchBracketInfo, type MsiTeamGroups } from '@/api/tauri'
import type { Player, PlayerPosition } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import { createLogger } from '@/utils/logger'
import { useBatchSimulation, buildMatchDetail, recordMatchPerformances } from '@/composables/useBatchSimulation'

const logger = createLogger('MSIDetail')

const route = useRoute()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()
const timeStore = useTimeStore()

// 从 query 获取赛季（赛事管理页传入），否则使用当前赛季
const viewingSeason = computed(() => Number(route.query.season) || gameStore.gameState?.current_season || 1)

// 加载状态
const loading = ref(false)
const currentTournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)

// MSI参赛队伍分组（从春季季后赛结果获取）
const msiQualifiedTeams = ref<MsiTeamGroups | null>(null)

// 队伍ID到名称的映射
const teamMap = ref<Map<number, { name: string; regionCode: string }>>(new Map())

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// Mock MSI数据
const mockMSIBracket = reactive({
  id: '1',
  seasonId: '1',
  seasonYear: 2024,
  status: 'in_progress' as 'in_progress' | 'completed' | 'not_started',
  qualifiedTeams: [
    { teamId: '1', teamName: 'JDG', regionName: 'LPL', seed: 1 },
    { teamId: '2', teamName: 'T1', regionName: 'LCK', seed: 1 },
    { teamId: '3', teamName: 'G2', regionName: 'LEC', seed: 1 },
    { teamId: '4', teamName: 'C9', regionName: 'LCS', seed: 1 },
    { teamId: '5', teamName: 'BLG', regionName: 'LPL', seed: 2 },
    { teamId: '6', teamName: 'GEN', regionName: 'LCK', seed: 2 },
    { teamId: '7', teamName: 'FNC', regionName: 'LEC', seed: 2 },
    { teamId: '8', teamName: 'TL', regionName: 'LCS', seed: 2 },
    { teamId: '9', teamName: 'TES', regionName: 'LPL', seed: 3 },
    { teamId: '10', teamName: 'DK', regionName: 'LCK', seed: 3 },
    { teamId: '11', teamName: 'MAD', regionName: 'LEC', seed: 3 },
    { teamId: '12', teamName: '100T', regionName: 'LCS', seed: 3 },
  ],
  legendaryGroup: [
    { teamId: '1', teamName: 'JDG', regionName: 'LPL' },
    { teamId: '2', teamName: 'T1', regionName: 'LCK' },
    { teamId: '3', teamName: 'G2', regionName: 'LEC' },
    { teamId: '4', teamName: 'C9', regionName: 'LCS' },
  ],
  challengerGroup: [
    { teamId: '5', teamName: 'BLG', regionName: 'LPL' },
    { teamId: '6', teamName: 'GEN', regionName: 'LCK' },
    { teamId: '7', teamName: 'FNC', regionName: 'LEC' },
    { teamId: '8', teamName: 'TL', regionName: 'LCS' },
  ],
  qualifierGroup: [
    { teamId: '9', teamName: 'TES', regionName: 'LPL' },
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
        // 资格赛组 (季军组) - 4队两两BO5单淘汰
        { id: 'qual1', matchType: 'qualifier', match_order: 1, teamAId: '9', teamBId: '12', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },  // TES vs 100T
        { id: 'qual2', matchType: 'qualifier', match_order: 2, teamAId: '10', teamBId: '11', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null }, // DK vs MAD
        // 挑战者组 (亚军组) - 4队PK
        { id: 'chal1', matchType: 'challenger', match_order: 1, teamAId: '5', teamBId: '8', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },  // BLG vs TL
        { id: 'chal2', matchType: 'challenger', match_order: 2, teamAId: '6', teamBId: '7', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },  // GEN vs FNC
      ]
    },
    // 败者组
    {
      roundNumber: 2,
      roundName: '败者组',
      status: 'pending',
      matches: [
        // 败者组R1: 资格赛胜者 vs 挑战者败者
        { id: 'lr1_1', matchType: 'loser_r1', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr1_2', matchType: 'loser_r1', match_order: 2, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        // 败者组R2: 挑战者胜者 vs R1胜者
        { id: 'lr2_1', matchType: 'loser_r2', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr2_2', matchType: 'loser_r2', match_order: 2, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        // 败者组R3: R2胜者 vs 胜者组R1败者
        { id: 'lr3_1', matchType: 'loser_r3', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr3_2', matchType: 'loser_r3', match_order: 2, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        // 败者组R4: 2名R3胜者对决
        { id: 'lr4', matchType: 'loser_r4', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        // 败者组决赛: 胜者组R2败者 vs R4胜者
        { id: 'lf', matchType: 'loser_final', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
      ]
    },
    // 胜者组 (传奇组)
    {
      roundNumber: 3,
      roundName: '胜者组',
      status: 'pending',
      matches: [
        // 胜者组R1: 4传奇组对决
        { id: 'wr1_1', matchType: 'winner_r1', match_order: 1, teamAId: '1', teamBId: '4', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },  // JDG vs C9
        { id: 'wr1_2', matchType: 'winner_r1', match_order: 2, teamAId: '2', teamBId: '3', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },  // T1 vs G2
        // 胜者组决赛
        { id: 'wf', matchType: 'winner_final', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
      ]
    },
    // 总决赛
    {
      roundNumber: 4,
      roundName: '总决赛',
      status: 'pending',
      matches: [
        { id: 'gf', matchType: 'grand_final', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
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
  loserRound1: [] as any[],   // 败者组第一轮败者 (2队)
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString()
})

// 批量模拟 composable
const { simulationProgress, isSimulating: batchSimulating } = useBatchSimulation()

// 响应式状态
const showMatchDetails = ref(false)
const selectedMatch = ref<any>(null)
const simulating = ref(false)

// 计算属性 - 使用 mock 数据
const currentMSIBracket = computed(() => mockMSIBracket as any)

const msiStandings = computed<StandingItem[]>(() => [
  { rank: 1, label: '冠军', name: currentMSIBracket.value.champion?.teamName || '', points: `+${currentMSIBracket.value.pointsDistribution.champion}分` },
  { rank: 2, label: '亚军', name: currentMSIBracket.value.runnerUp?.teamName || '', points: `+${currentMSIBracket.value.pointsDistribution.runnerUp}分` },
  { rank: 3, label: '季军', name: currentMSIBracket.value.thirdPlace?.teamName || '', points: `+${currentMSIBracket.value.pointsDistribution.thirdPlace}分` },
  { rank: 4, label: '殿军', name: currentMSIBracket.value.fourthPlace?.teamName || '', points: `+${currentMSIBracket.value.pointsDistribution.fourthPlace}分` },
])

// 是否有真实队伍数据（从后端加载）
const hasRealTeamData = computed(() => {
  // 检查后端是否已经加载了真实的对阵数据
  return bracketData.value !== null && bracketData.value.matches.length > 0
})

// 从后端数据或 mock 数据获取分组队伍
const legendaryGroupTeams = computed(() => {
  // 优先使用从API获取的队伍分组数据
  if (msiQualifiedTeams.value && msiQualifiedTeams.value.legendary.length > 0) {
    return msiQualifiedTeams.value.legendary.map(team => ({
      teamId: String(team.team_id),
      teamName: team.short_name || team.team_name,
      regionName: team.region_name
    }))
  }
  // 其次使用从比赛数据解析的分组
  if (hasRealTeamData.value && mockMSIBracket.legendaryGroup.length > 0) {
    const firstTeam = mockMSIBracket.legendaryGroup[0]
    if (firstTeam && firstTeam.teamId !== '1') {
      return mockMSIBracket.legendaryGroup
    }
  }
  return []
})

const challengerGroupTeams = computed(() => {
  // 优先使用从API获取的队伍分组数据
  if (msiQualifiedTeams.value && msiQualifiedTeams.value.challenger.length > 0) {
    return msiQualifiedTeams.value.challenger.map(team => ({
      teamId: String(team.team_id),
      teamName: team.short_name || team.team_name,
      regionName: team.region_name
    }))
  }
  // 其次使用从比赛数据解析的分组
  if (hasRealTeamData.value && mockMSIBracket.challengerGroup.length > 0) {
    const firstTeam = mockMSIBracket.challengerGroup[0]
    if (firstTeam && firstTeam.teamId !== '5') {
      return mockMSIBracket.challengerGroup
    }
  }
  return []
})

const qualifierGroupTeams = computed(() => {
  // 优先使用从API获取的队伍分组数据
  if (msiQualifiedTeams.value && msiQualifiedTeams.value.qualifier.length > 0) {
    return msiQualifiedTeams.value.qualifier.map(team => ({
      teamId: String(team.team_id),
      teamName: team.short_name || team.team_name,
      regionName: team.region_name
    }))
  }
  // 其次使用从比赛数据解析的分组
  if (hasRealTeamData.value && mockMSIBracket.qualifierGroup.length > 0) {
    const firstTeam = mockMSIBracket.qualifierGroup[0]
    if (firstTeam && firstTeam.teamId !== '9') {
      return mockMSIBracket.qualifierGroup
    }
  }
  return []
})

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
 * 刷新数据
 */
const refreshData = async () => {
  await loadMSIData()
  ElMessage.success('数据已刷新')
}

/**
 * 加载MSI赛事数据
 */
const loadMSIData = async () => {
  loading.value = true
  try {
    const seasonId = viewingSeason.value

    // 同时获取MSI赛事列表和参赛队伍分组
    const [tournaments, qualifiedTeams] = await Promise.all([
      queryApi.getInternationalTournaments(seasonId),
      internationalApi.getMsiQualifiedTeams(seasonId)
    ])

    // 存储参赛队伍分组
    msiQualifiedTeams.value = qualifiedTeams
    logger.debug('[MSI] 参赛队伍分组:', qualifiedTeams)

    // 查找 MSI 赛事 - 优先选择有比赛的，否则选择最新的（id最大的）
    const msiTournaments = tournaments.filter(t => t.tournament_type === 'Msi')
    // 优先选择有比赛的MSI赛事
    let msiTournament = msiTournaments.find(t => t.match_count > 0)
    // 如果没有有比赛的，选择最新的（id最大的）
    if (!msiTournament && msiTournaments.length > 0) {
      msiTournament = msiTournaments.reduce((latest, t) => t.id > latest.id ? t : latest)
    }

    if (msiTournament) {
      currentTournamentId.value = msiTournament.id
      logger.debug('[MSI] 选择赛事:', msiTournament.id, msiTournament.name, 'match_count:', msiTournament.match_count)

      // 如果没有比赛但队伍已就绪，尝试重新生成对阵
      if (msiTournament.match_count === 0 &&
          qualifiedTeams.legendary.length === 4 &&
          qualifiedTeams.challenger.length === 4 &&
          qualifiedTeams.qualifier.length === 4) {
        logger.debug('[MSI] 队伍已就绪但无比赛，尝试重新生成对阵...')
        try {
          const matchCount = await internationalApi.regenerateMsiBracket(msiTournament.id)
          logger.debug('[MSI] 成功生成', matchCount, '场比赛')
          ElMessage.success(`已生成 ${matchCount} 场 MSI 比赛`)
        } catch (e) {
          logger.error('[MSI] 重新生成对阵失败:', e)
        }
      }

      // 加载对阵数据
      await loadBracketData()
    } else {
      // 如果没有 MSI 赛事，保持 mock 数据显示
      logger.debug('No MSI tournament found for season', seasonId)
    }
  } catch (error) {
    logger.error('Failed to load MSI data:', error)
  } finally {
    loading.value = false
  }
}

/**
 * 加载对阵图数据
 */
const loadBracketData = async () => {
  if (!currentTournamentId.value) return

  try {
    const bracket = await internationalApi.getTournamentBracket(currentTournamentId.value)
    bracketData.value = bracket
    logger.debug('[MSI] loadBracketData: 获取到', bracket.matches.length, '场比赛')

    // 构建队伍映射
    teamMap.value.clear()
    bracket.matches.forEach(match => {
      if (match.home_team) {
        teamMap.value.set(match.home_team.id, {
          name: match.home_team.short_name || match.home_team.name,
          regionCode: match.home_team.region_code
        })
      }
      if (match.away_team) {
        teamMap.value.set(match.away_team.id, {
          name: match.away_team.short_name || match.away_team.name,
          regionCode: match.away_team.region_code
        })
      }
    })
    logger.debug('[MSI] teamMap 队伍数量:', teamMap.value.size)

    // 更新 mockMSIBracket 的状态
    updateMSIBracketFromBackend(bracket)
  } catch (error) {
    logger.error('Failed to load bracket data:', error)
  }
}

/**
 * 从后端数据更新 MSI 对阵
 */
const updateMSIBracketFromBackend = (bracket: BracketInfo) => {
  // 更新赛事状态
  const allCompleted = bracket.matches.every(m => m.status === 'Completed' || m.status === 'COMPLETED')
  const anyStarted = bracket.matches.some(m => m.status === 'Completed' || m.status === 'COMPLETED')
  mockMSIBracket.status = allCompleted ? 'completed' : anyStarted ? 'in_progress' : 'not_started'

  // 构建队伍分组 - 先从 teamMap 构建完整的队伍列表
  const allTeams: any[] = []

  // 从 teamMap 添加所有队伍
  teamMap.value.forEach((teamInfo, teamId) => {
    allTeams.push({
      teamId: String(teamId),
      teamName: teamInfo.name,
      regionName: teamInfo.regionCode,
      seed: 1
    })
  })
  logger.debug('[MSI] 从 teamMap 添加队伍:', allTeams.length)

  // 从比赛数据中补充（以防 teamMap 没有包含所有队伍）
  bracket.matches.forEach(match => {
    if (match.home_team) {
      const team = {
        teamId: String(match.home_team.id),
        teamName: match.home_team.short_name || match.home_team.name,
        regionName: match.home_team.region_code,
        seed: 1
      }
      if (!allTeams.find(t => t.teamId === team.teamId)) {
        allTeams.push(team)
      }
    }
    if (match.away_team) {
      const team = {
        teamId: String(match.away_team.id),
        teamName: match.away_team.short_name || match.away_team.name,
        regionName: match.away_team.region_code,
        seed: 1
      }
      if (!allTeams.find(t => t.teamId === team.teamId)) {
        allTeams.push(team)
      }
    }
  })

  // 如果从API获取了队伍分组，也添加到 allTeams
  if (msiQualifiedTeams.value) {
    const addTeamIfNotExists = (team: any) => {
      const teamObj = {
        teamId: String(team.team_id),
        teamName: team.short_name || team.team_name,
        regionName: team.region_name,
        seed: 1
      }
      if (!allTeams.find(t => t.teamId === teamObj.teamId)) {
        allTeams.push(teamObj)
      }
    }
    msiQualifiedTeams.value.legendary.forEach(addTeamIfNotExists)
    msiQualifiedTeams.value.challenger.forEach(addTeamIfNotExists)
    msiQualifiedTeams.value.qualifier.forEach(addTeamIfNotExists)
  }

  logger.debug('[MSI] 合并后队伍总数:', allTeams.length)

  // 根据阶段分类队伍
  const legendaryTeams: any[] = []
  const challengerTeams: any[] = []
  const qualifierTeams: any[] = []

  bracket.matches.forEach(match => {
    const stage = match.stage.toUpperCase()

    // 传奇组：胜者组阶段
    if (stage.includes('WINNER') || stage.includes('LEGENDARY')) {
      if (match.home_team) {
        const team = { teamId: String(match.home_team.id), teamName: match.home_team.short_name || match.home_team.name, regionName: match.home_team.region_code }
        if (!legendaryTeams.find(t => t.teamId === team.teamId)) legendaryTeams.push(team)
      }
      if (match.away_team) {
        const team = { teamId: String(match.away_team.id), teamName: match.away_team.short_name || match.away_team.name, regionName: match.away_team.region_code }
        if (!legendaryTeams.find(t => t.teamId === team.teamId)) legendaryTeams.push(team)
      }
    } else if (stage.includes('CHALLENGER')) {
      if (match.home_team) {
        const team = { teamId: String(match.home_team.id), teamName: match.home_team.short_name || match.home_team.name, regionName: match.home_team.region_code }
        if (!challengerTeams.find(t => t.teamId === team.teamId)) challengerTeams.push(team)
      }
      if (match.away_team) {
        const team = { teamId: String(match.away_team.id), teamName: match.away_team.short_name || match.away_team.name, regionName: match.away_team.region_code }
        if (!challengerTeams.find(t => t.teamId === team.teamId)) challengerTeams.push(team)
      }
    } else if (stage.includes('QUALIFIER')) {
      if (match.home_team) {
        const team = { teamId: String(match.home_team.id), teamName: match.home_team.short_name || match.home_team.name, regionName: match.home_team.region_code }
        if (!qualifierTeams.find(t => t.teamId === team.teamId)) qualifierTeams.push(team)
      }
      if (match.away_team) {
        const team = { teamId: String(match.away_team.id), teamName: match.away_team.short_name || match.away_team.name, regionName: match.away_team.region_code }
        if (!qualifierTeams.find(t => t.teamId === team.teamId)) qualifierTeams.push(team)
      }
    }
  })

  logger.debug('[MSI] 分组结果: legendary=', legendaryTeams.length, 'challenger=', challengerTeams.length, 'qualifier=', qualifierTeams.length)

  // 更新 mockMSIBracket
  mockMSIBracket.qualifiedTeams = allTeams
  mockMSIBracket.legendaryGroup = legendaryTeams.length > 0 ? legendaryTeams : mockMSIBracket.legendaryGroup
  mockMSIBracket.challengerGroup = challengerTeams.length > 0 ? challengerTeams : mockMSIBracket.challengerGroup
  mockMSIBracket.qualifierGroup = qualifierTeams.length > 0 ? qualifierTeams : mockMSIBracket.qualifierGroup

  // 更新比赛数据
  updateMatchesFromBackend(bracket.matches)
}

/**
 * 将后端比赛数据映射到前端格式
 */
const updateMatchesFromBackend = (matches: MatchBracketInfo[]) => {
  // 阶段映射：后端阶段名 -> 前端 matchType
  const stageToMatchType: Record<string, string> = {
    // 后端实际使用的阶段名（大写带下划线）
    'QUALIFIER_R1': 'qualifier',
    'CHALLENGER_R1': 'challenger',
    'WINNERS_R1': 'winner_r1',
    'WINNERS_FINAL': 'winner_final',
    'LOSERS_R1': 'loser_r1',
    'LOSERS_R2': 'loser_r2',
    'LOSERS_R3': 'loser_r3',
    'LOSERS_R4': 'loser_r4',
    'LOSERS_FINAL': 'loser_final',
    'GRAND_FINAL': 'grand_final',
    // 兼容旧格式
    'Qualifier': 'qualifier',
    'Challenger': 'challenger',
    'WinnerR1': 'winner_r1',
    'WinnersFinal': 'winner_final',
    'LoserR1': 'loser_r1',
    'LoserR2': 'loser_r2',
    'LoserR3': 'loser_r3',
    'LoserR4': 'loser_r4',
    'LosersFinal': 'loser_final',
    'GrandFinal': 'grand_final'
  }

  matches.forEach(backendMatch => {
    const matchType = stageToMatchType[backendMatch.stage] || backendMatch.stage.toLowerCase()
    logger.debug('[MSI] 处理后端比赛:', backendMatch.stage, '-> matchType:', matchType, 'match_order:', backendMatch.match_order)

    // 在 rounds 中查找对应的比赛
    for (const round of mockMSIBracket.rounds) {
      // 优先按 matchType 和 match_order 匹配
      let frontendMatch = round.matches.find((m: any) =>
        m.matchType === matchType && m.match_order === backendMatch.match_order
      )
      // 如果没找到，尝试只按 matchType 匹配（兼容单场比赛的情况）
      if (!frontendMatch) {
        frontendMatch = round.matches.find((m: any) => m.matchType === matchType)
      }

      if (frontendMatch) {
        // 更新比赛数据
        (frontendMatch as any).backendMatchId = backendMatch.match_id
        if (backendMatch.home_team) {
          frontendMatch.teamAId = String(backendMatch.home_team.id)
        }
        if (backendMatch.away_team) {
          frontendMatch.teamBId = String(backendMatch.away_team.id)
        }
        frontendMatch.scoreA = backendMatch.home_score
        frontendMatch.scoreB = backendMatch.away_score
        frontendMatch.winnerId = backendMatch.winner_id ? String(backendMatch.winner_id) : null

        // 处理状态 - 后端可能返回大写 'COMPLETED' 或首字母大写 'Completed'
        const backendStatus = backendMatch.status.toUpperCase()
        frontendMatch.status = backendStatus === 'COMPLETED' ? 'completed' :
                              backendStatus === 'INPROGRESS' || backendStatus === 'IN_PROGRESS' ? 'active' : 'scheduled'

        logger.debug('[MSI] 更新比赛:', frontendMatch.id, 'status:', frontendMatch.status,
          'score:', frontendMatch.scoreA, '-', frontendMatch.scoreB)
        break
      }
    }
  })
}

/**
 * 模拟单场比赛
 */
const simulateMSIMatch = async (match: any) => {
  // 如果有后端 match ID，使用后端 API 模拟
  if (match.backendMatchId && currentTournamentId.value) {
    try {
      // 使用后端 API 模拟比赛
      const result = await matchApi.simulateMatchDetailed(match.backendMatchId)
      logger.debug('[MSI] 模拟结果:', result)

      // 更新比赛结果
      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = String(result.winner_id)
      match.status = 'completed'
      match.playedAt = new Date().toISOString()

      // 获取队伍名称
      const teamA = mockMSIBracket.qualifiedTeams.find(t => t.teamId === match.teamAId)
      const teamB = mockMSIBracket.qualifiedTeams.find(t => t.teamId === match.teamBId)
      const teamAName = teamA?.teamName || '队伍A'
      const teamBName = teamB?.teamName || '队伍B'

      // 转换后端结果为 MatchDetail 格式并保存
      const matchDetail = buildMatchDetail({
        matchId: match.id,
        tournamentType: 'msi',
        seasonId: String(mockMSIBracket.seasonYear),
        teamAId: String(match.teamAId || ''),
        teamAName,
        teamBId: String(match.teamBId || ''),
        teamBName,
        bestOf: match.bestOf || 5,
        result
      })
      await matchDetailStore.saveMatchDetail(match.id, matchDetail)
      logger.debug(`[MSI] 已保存比赛详情到本地: ${match.id}`)

      // 同时用数据库 ID 保存一份，确保能从数据库加载（与季后赛保持一致）
      if (match.backendMatchId) {
        const dbMatchDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
        await matchDetailStore.saveMatchDetail(match.backendMatchId, dbMatchDetail)
        logger.debug(`[MSI] 已保存比赛详情到数据库: backendMatchId=${match.backendMatchId}`)
      }

      // 记录选手表现到 playerStore
      recordMatchPerformances(matchDetail, String(mockMSIBracket.seasonYear), 'INTL', playerStore)
      playerStore.saveToStorage()

      // 调用后端推进对阵
      await internationalApi.advanceBracket(
        currentTournamentId.value,
        match.backendMatchId,
        result.winner_id
      )

      // 重新加载对阵数据
      await loadBracketData()

      ElMessage.success(`比赛完成: ${match.teamAId} ${result.home_score} - ${result.away_score} ${match.teamBId}`)

      // 保存详情数据，但不自动弹出（用户可点击查看）
      currentMatchDetail.value = matchDetail

      // 检查是否全部完成
      checkMSICompletion()
      return
    } catch (error) {
      logger.error('Backend simulation failed, falling back to local:', error)
      // 后端失败时使用本地 PowerEngine
    }
  }

  // 本地 PowerEngine 模拟 (作为后备方案)
  const teamA = mockMSIBracket.qualifiedTeams.find(t => t.teamId === match.teamAId)
  const teamB = mockMSIBracket.qualifiedTeams.find(t => t.teamId === match.teamBId)

  const teamAName = teamA?.teamName || '队伍A'
  const teamBName = teamB?.teamName || '队伍B'
  const teamARegion = teamA?.regionName || 'Unknown'
  const teamBRegion = teamB?.regionName || 'Unknown'

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(match.teamAId, teamAName, teamARegion)
  const teamBPlayers = generateTeamPlayers(match.teamBId, teamBName, teamBRegion)

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
  match.playedAt = new Date().toISOString()

  // 保存比赛详情
  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'msi'
  matchDetail.seasonId = String(mockMSIBracket.seasonYear)
  matchDetailStore.saveMatchDetail(match.id, matchDetail)

  // 同时用数据库 ID 保存一份（如果有的话）
  if (match.backendMatchId) {
    const dbMatchDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
    matchDetailStore.saveMatchDetail(match.backendMatchId, dbMatchDetail)
  }

  // 记录选手表现
  recordMatchPerformances(matchDetail, String(mockMSIBracket.seasonYear), 'INTL', playerStore)
  playerStore.saveToStorage()

  ElMessage.success(`比赛完成: ${matchDetail.finalScoreA} - ${matchDetail.finalScoreB}`)

  // 更新后续比赛的对阵
  updateBracketProgression()
}

/**
 * 更新对阵晋级 - 双败赛制
 */
const updateBracketProgression = () => {
  const rounds = mockMSIBracket.rounds
  const preliminary = rounds[0].matches  // 预选赛 (qual1, qual2, chal1, chal2)
  const loserBracket = rounds[1].matches // 败者组 (lr1_1, lr1_2, lr2_1, lr2_2, lr3_1, lr3_2, lr4, lf)
  const winnerBracket = rounds[2].matches // 胜者组 (wr1_1, wr1_2, wf)
  const finalRound = rounds[3].matches   // 总决赛 (gf)

  // 辅助函数：获取match的败者
  const getLoser = (match: any) => {
    if (!match.winnerId) return null
    return match.winnerId === match.teamAId ? match.teamBId : match.teamAId
  }

  // --- 预选赛结果更新败者组R1 ---
  // qual1胜者 vs chal1败者 → lr1_1
  // qual2胜者 vs chal2败者 → lr1_2
  const qual1 = preliminary.find((m: any) => m.id === 'qual1')
  const qual2 = preliminary.find((m: any) => m.id === 'qual2')
  const chal1 = preliminary.find((m: any) => m.id === 'chal1')
  const chal2 = preliminary.find((m: any) => m.id === 'chal2')

  const lr1_1 = loserBracket.find((m: any) => m.id === 'lr1_1')
  const lr1_2 = loserBracket.find((m: any) => m.id === 'lr1_2')

  if (qual1?.winnerId && chal1?.winnerId && lr1_1) {
    lr1_1.teamAId = qual1.winnerId  // 资格赛胜者
    lr1_1.teamBId = getLoser(chal1) // 挑战者败者
  }
  if (qual2?.winnerId && chal2?.winnerId && lr1_2) {
    lr1_2.teamAId = qual2.winnerId  // 资格赛胜者
    lr1_2.teamBId = getLoser(chal2) // 挑战者败者
  }

  // --- 败者组R2: 挑战者胜者 vs R1胜者 ---
  const lr2_1 = loserBracket.find((m: any) => m.id === 'lr2_1')
  const lr2_2 = loserBracket.find((m: any) => m.id === 'lr2_2')

  if (chal1?.winnerId && lr1_1?.winnerId && lr2_1) {
    lr2_1.teamAId = chal1.winnerId  // 挑战者胜者
    lr2_1.teamBId = lr1_1.winnerId  // R1胜者
  }
  if (chal2?.winnerId && lr1_2?.winnerId && lr2_2) {
    lr2_2.teamAId = chal2.winnerId  // 挑战者胜者
    lr2_2.teamBId = lr1_2.winnerId  // R1胜者
  }

  // --- 胜者组R1败者 掉入败者组R3 ---
  const wr1_1 = winnerBracket.find((m: any) => m.id === 'wr1_1')
  const wr1_2 = winnerBracket.find((m: any) => m.id === 'wr1_2')
  const lr3_1 = loserBracket.find((m: any) => m.id === 'lr3_1')
  const lr3_2 = loserBracket.find((m: any) => m.id === 'lr3_2')

  // 败者组R3: R2胜者 vs 胜者组R1败者
  if (lr2_1?.winnerId && wr1_1?.winnerId && lr3_1) {
    lr3_1.teamAId = lr2_1.winnerId   // R2胜者
    lr3_1.teamBId = getLoser(wr1_1)  // 胜者组R1败者
  }
  if (lr2_2?.winnerId && wr1_2?.winnerId && lr3_2) {
    lr3_2.teamAId = lr2_2.winnerId   // R2胜者
    lr3_2.teamBId = getLoser(wr1_2)  // 胜者组R1败者
  }

  // --- 胜者组决赛 ---
  const wf = winnerBracket.find((m: any) => m.id === 'wf')
  if (wr1_1?.winnerId && wr1_2?.winnerId && wf) {
    wf.teamAId = wr1_1.winnerId
    wf.teamBId = wr1_2.winnerId
  }

  // --- 败者组R4: 2名R3胜者对决 ---
  const lr4 = loserBracket.find((m: any) => m.id === 'lr4')
  if (lr3_1?.winnerId && lr3_2?.winnerId && lr4) {
    lr4.teamAId = lr3_1.winnerId
    lr4.teamBId = lr3_2.winnerId
  }

  // --- 败者组决赛: 胜者组决赛败者 vs R4胜者 ---
  const lf = loserBracket.find((m: any) => m.id === 'lf')
  if (wf?.winnerId && lr4?.winnerId && lf) {
    lf.teamAId = getLoser(wf)  // 胜者组决赛败者
    lf.teamBId = lr4.winnerId  // R4胜者
  }

  // --- 总决赛: 胜者组冠军 vs 败者组决赛胜者 ---
  const gf = finalRound.find((m: any) => m.id === 'gf')
  if (wf?.winnerId && lf?.winnerId && gf) {
    gf.teamAId = wf.winnerId  // 胜者组冠军
    gf.teamBId = lf.winnerId  // 败者组决赛胜者
  }

  // --- 检查是否全部完成 ---
  if (gf?.winnerId) {
    mockMSIBracket.status = 'completed'
    const champion = mockMSIBracket.qualifiedTeams.find(t => t.teamId === gf.winnerId)
    const runnerUp = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(gf))
    // 季军是败者组决赛的败者
    const thirdPlace = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lf))
    // 殿军是败者组R4的败者
    const fourthPlace = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr4))

    // 败者组第二轮败者 (lr3_1, lr3_2的败者) - 6分
    const loserR2Teams: any[] = []
    if (lr3_1?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr3_1))
      if (loser) loserR2Teams.push(loser)
    }
    if (lr3_2?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr3_2))
      if (loser) loserR2Teams.push(loser)
    }

    // 败者组第一轮败者 (lr2_1, lr2_2的败者) - 4分
    const loserR1Teams: any[] = []
    if (lr2_1?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr2_1))
      if (loser) loserR1Teams.push(loser)
    }
    if (lr2_2?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr2_2))
      if (loser) loserR1Teams.push(loser)
    }

    mockMSIBracket.champion = champion || null
    mockMSIBracket.runnerUp = runnerUp || null
    mockMSIBracket.thirdPlace = thirdPlace || null
    mockMSIBracket.fourthPlace = fourthPlace || null
    mockMSIBracket.loserRound2 = loserR2Teams
    mockMSIBracket.loserRound1 = loserR1Teams

    showChampionCelebration(champion?.teamName || '')
  }
}

/**
 * 批量模拟MSI - 使用 useBatchSimulation composable 的工具函数
 */
const batchSimulateMSI = async () => {
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

    const matchDetailStore = useMatchDetailStore()
    const playerStore = usePlayerStore()

    // 计算总比赛数用于进度
    const totalMatches = mockMSIBracket.rounds.flatMap(r => r.matches).filter((m: any) => m.status !== 'completed').length
    let completed = 0

    // While 循环：每轮从最新响应式数据获取可模拟比赛，直到没有为止
    const MAX_ITERATIONS = 50 // 防止无限循环
    let iterations = 0

    while (iterations < MAX_ITERATIONS) {
      iterations++
      // 从最新的 mockMSIBracket 中获取可模拟的比赛
      const available = mockMSIBracket.rounds.flatMap(r => r.matches).filter((m: any) =>
        m.status !== 'completed' && m.backendMatchId && m.teamAId && m.teamBId
      )

      if (available.length === 0) break

      for (const match of available) {
        try {
          const result = await matchApi.simulateMatchDetailed(match.backendMatchId!)

          // 保存比赛详情
          const teamAName = getTeamName(match.teamAId) || ''
          const teamBName = getTeamName(match.teamBId) || ''
          const matchDetail = buildMatchDetail({
            matchId: match.id,
            tournamentType: 'msi',
            seasonId: String(mockMSIBracket.seasonYear),
            teamAId: String(match.teamAId),
            teamAName,
            teamBId: String(match.teamBId),
            teamBName,
            bestOf: match.bestOf || 5,
            result
          })
          await matchDetailStore.saveMatchDetail(match.id, matchDetail)

          if (match.backendMatchId) {
            const dbDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
            await matchDetailStore.saveMatchDetail(match.backendMatchId, dbDetail)
          }

          recordMatchPerformances(matchDetail, String(mockMSIBracket.seasonYear), 'INTL', playerStore)

          // 推进对阵
          if (currentTournamentId.value) {
            await internationalApi.advanceBracket(currentTournamentId.value, match.backendMatchId!, result.winner_id)
          }

          // 重新加载数据，使下一轮比赛的队伍信息被填充
          await loadBracketData()

          completed++
          simulationProgress.value = Math.floor((completed / totalMatches) * 100)
        } catch (e) {
          logger.error(`[MSI] 模拟比赛 ${match.backendMatchId} 失败:`, e)
        }

        await new Promise(resolve => setTimeout(resolve, 100))
      }
    }

    playerStore.saveToStorage()
    await loadBracketData()
    await checkMSICompletion()

    ElMessage.success('MSI批量模拟完成!')
  } catch (error: any) {
    if (error !== 'cancel') {
      logger.error('MSI批量模拟失败:', error)
      ElMessage.error(error.message || 'MSI模拟失败')
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
    `恭喜 ${championName} 获得MSI冠军,成为世界最强战队!`,
    '🏆 MSI冠军诞生! 🏆',
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
const viewMatchDetails = async (match: any) => {
  // 如果是已完成的比赛，尝试从 store 获取详情
  if (match.status === 'completed') {
    // 先尝试用前端 ID 从内存获取
    let detail = matchDetailStore.getMatchDetail(match.id)
    if (detail) {
      currentMatchDetail.value = detail
      showMatchDetailDialog.value = true
      return
    }

    // 再尝试用数据库 ID 从内存获取
    if (match.backendMatchId) {
      detail = matchDetailStore.getMatchDetail(match.backendMatchId)
      if (detail) {
        currentMatchDetail.value = detail
        showMatchDetailDialog.value = true
        return
      }

      // 最后尝试从数据库加载
      detail = await matchDetailStore.loadMatchDetailFromDb(match.backendMatchId)
      if (detail) {
        currentMatchDetail.value = detail
        showMatchDetailDialog.value = true
        return
      }
    }
  }

  // 如果没有详情数据，显示原有的简单弹窗
  selectedMatch.value = match
  showMatchDetails.value = true
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
  simulateMSIMatch(selectedMatch.value)
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
  // 优先从 teamMap（后端数据）获取
  const numId = Number(teamId)
  if (!isNaN(numId)) {
    const teamFromMap = teamMap.value.get(numId)
    if (teamFromMap) {
      return teamFromMap.name
    }
  }
  // 回退到 qualifiedTeams
  const team = mockMSIBracket.qualifiedTeams.find(t => t.teamId === teamId)
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

/**
 * 检查 MSI 赛事是否完成
 */
const checkMSICompletion = async () => {
  const finalRound = mockMSIBracket.rounds[3]
  const grandFinal = finalRound?.matches.find((m: any) => m.id === 'gf')

  if (grandFinal?.winnerId) {
    mockMSIBracket.status = 'completed'

    // 获取胜负方
    const getLoser = (match: any) => {
      if (!match?.winnerId) return null
      return match.winnerId === match.teamAId ? match.teamBId : match.teamAId
    }

    // 获取败者组决赛
    const loserBracket = mockMSIBracket.rounds[1].matches
    const lf = loserBracket.find((m: any) => m.id === 'lf')
    const lr4 = loserBracket.find((m: any) => m.id === 'lr4')
    const lr3_1 = loserBracket.find((m: any) => m.id === 'lr3_1')
    const lr3_2 = loserBracket.find((m: any) => m.id === 'lr3_2')
    const lr2_1 = loserBracket.find((m: any) => m.id === 'lr2_1')
    const lr2_2 = loserBracket.find((m: any) => m.id === 'lr2_2')

    // 设置最终排名
    const champion = mockMSIBracket.qualifiedTeams.find(t => t.teamId === grandFinal.winnerId)
    const runnerUp = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(grandFinal))
    const thirdPlace = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lf))
    const fourthPlace = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr4))

    // 败者组排名
    const loserR2Teams: any[] = []
    if (lr3_1?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr3_1))
      if (loser) loserR2Teams.push(loser)
    }
    if (lr3_2?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr3_2))
      if (loser) loserR2Teams.push(loser)
    }

    const loserR1Teams: any[] = []
    if (lr2_1?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr2_1))
      if (loser) loserR1Teams.push(loser)
    }
    if (lr2_2?.winnerId) {
      const loser = mockMSIBracket.qualifiedTeams.find(t => t.teamId === getLoser(lr2_2))
      if (loser) loserR1Teams.push(loser)
    }

    mockMSIBracket.champion = champion || null
    mockMSIBracket.runnerUp = runnerUp || null
    mockMSIBracket.thirdPlace = thirdPlace || null
    mockMSIBracket.fourthPlace = fourthPlace || null
    mockMSIBracket.loserRound2 = loserR2Teams
    mockMSIBracket.loserRound1 = loserR1Teams

    // 调用后端 completeTournament 命令处理荣誉殿堂和年度积分
    if (currentTournamentId.value) {
      await processTournamentCompletion(currentTournamentId.value)
    }

    if (champion) {
      showChampionCelebration(champion.teamName)
    }

    // 刷新时间状态（用户可在全局控制面板推进阶段）
    await timeStore.fetchTimeState()
  }
}

/**
 * 调用后端完成赛事处理 - 处理荣誉殿堂和年度积分
 */
const processTournamentCompletion = async (tournamentId: number) => {
  try {
    const result = await internationalApi.completeTournament(tournamentId)

    // 打印结果信息
    logger.debug(`[MSI] ${result.message}`)

    // 显示荣誉颁发信息
    if (result.honors_awarded.length > 0) {
      logger.debug('[MSI] 颁发的荣誉:')
      result.honors_awarded.forEach(honor => {
        logger.debug(`  - ${honor.honor_type}: ${honor.recipient_name} (${honor.recipient_type})`)
      })
    }

    // 显示积分颁发信息
    if (result.points_awarded.length > 0) {
      logger.debug('[MSI] 颁发的年度积分:')
      result.points_awarded.forEach(points => {
        logger.debug(`  - ${points.team_name}: +${points.points}分 (${points.position})`)
      })

      // 显示积分颁发通知
      const topTeams = result.points_awarded.slice(0, 4)
      const pointsMessage = topTeams.map(p => `${p.team_name} +${p.points}分`).join(', ')
      ElMessage.info(`年度积分已更新: ${pointsMessage}`)
    }

  } catch (error) {
    logger.error('[MSI] 完成赛事处理失败:', error)
    // 即使失败也不阻止游戏继续，只记录日志
  }
}

// 页面加载时初始化数据
onMounted(() => {
  loadMSIData()
})
</script>

<style scoped lang="scss">
.msi-management {
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

  .msi-status-card {
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

            &.pending {
              background: #f3f4f6;
              justify-content: center;

              .team-name {
                color: #9ca3af;
                font-style: italic;
              }
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

      .bracket-placeholder {
        padding: 60px 20px;
        background: #f9fafb;
        border-radius: 12px;
        border: 2px dashed #e5e7eb;

        .placeholder-text {
          margin-top: 16px;
          color: #6b7280;
          font-size: 14px;
        }
      }
    }

    .loser-standings {
      margin-top: 24px;
      margin-bottom: 16px;

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

  // 比赛详情对话框样式
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
