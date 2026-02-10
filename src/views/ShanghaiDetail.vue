<template>
  <div class="shanghai-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div>
        <h1>上海大师赛</h1>
        <p>12支队伍(各赛区夏季赛冠亚季军) · 双败淘汰赛制</p>
      </div>
      <div class="header-actions">
        <el-button @click="refreshData" :icon="Refresh" size="small">刷新</el-button>
        <button class="back-btn" @click="goBack">&larr; 返回赛事列表</button>
      </div>
    </div>

    <!-- 上海大师赛状态 -->
    <div v-if="currentBracket">
      <div class="filter-section">
        <div class="filter-row">
          <div class="filter-group">
            <label>S{{ viewingSeason }} 上海大师赛</label>
            <el-tag :type="getStatusType(currentBracket.status)" size="small">
              {{ getStatusText(currentBracket.status) }}
            </el-tag>
          </div>
          <div class="filter-group" v-if="hasRealTeamData && currentBracket.status !== 'completed'">
            <el-button
              type="warning"
              size="small"
              @click="batchSimulate"
              :loading="batchSimulating"
            >
              {{ batchSimulating ? `模拟中 (${simulationProgress}%)` : '一键模拟全部' }}
            </el-button>
          </div>
        </div>
      </div>

      <!-- 参赛队伍分组 -->
      <div class="teams-groups">
        <div class="team-group">
          <div class="group-header legendary">传奇组 <span class="group-hint">(夏季赛冠军)</span></div>
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
              <span class="team-name">待夏季赛结束后确定</span>
            </div>
          </div>
        </div>

        <div class="team-group">
          <div class="group-header challenger">挑战者组 <span class="group-hint">(夏季赛亚军)</span></div>
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
              <span class="team-name">待夏季赛结束后确定</span>
            </div>
          </div>
        </div>

        <div class="team-group">
          <div class="group-header qualifier">资格赛组 <span class="group-hint">(夏季赛季军)</span></div>
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
              <span class="team-name">待夏季赛结束后确定</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 对阵图 -->
      <div class="bracket-section">
        <template v-if="hasRealTeamData">
          <MSIBracketView
            v-if="currentBracket"
            :bracket="currentBracket"
            tournament-name="上海大师赛"
            @simulate-match="simulateMatch"
            @view-match="viewMatchDetails"
          />
        </template>
        <div v-else class="bracket-placeholder">
          <p>赛事尚未开始</p>
          <p class="placeholder-text">请先完成夏季赛季后赛，确定参赛队伍后才能进行上海大师赛对阵</p>
        </div>
      </div>

      <!-- 最终排名 -->
      <TournamentCompletionSection
        v-if="currentBracket.status === 'completed'"
        :standings="shanghaiStandings"
        banner-title="上海大师赛已完成！"
        :banner-champion="currentBracket.champion?.teamName || ''"
        banner-description="获得上海大师赛冠军！"
      >
        <div v-if="currentBracket.loserRound2?.length > 0" class="loser-standings">
          <h4>败者组第二轮 (5-6名)</h4>
          <div class="loser-grid">
            <div v-for="(team, index) in currentBracket.loserRound2" :key="team.teamId" class="loser-item loser-r2">
              <div class="rank-badge"><span class="rank-number">{{ 5 + Number(index) }}</span></div>
              <div class="team-name">{{ team.teamName }}</div>
              <div class="points">+{{ currentBracket.pointsDistribution.loserRound2 }}分</div>
            </div>
          </div>
        </div>
        <div v-if="currentBracket.loserRound1?.length > 0" class="loser-standings">
          <h4>败者组第一轮 (7-8名)</h4>
          <div class="loser-grid">
            <div v-for="(team, index) in currentBracket.loserRound1" :key="team.teamId" class="loser-item loser-r1">
              <div class="rank-badge"><span class="rank-number">{{ 7 + Number(index) }}</span></div>
              <div class="team-name">{{ team.teamName }}</div>
              <div class="points">+{{ currentBracket.pointsDistribution.loserRound1 }}分</div>
            </div>
          </div>
        </div>
      </TournamentCompletionSection>
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
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Refresh
} from '@element-plus/icons-vue'
import MSIBracketView from '@/components/msi/MSIBracketView.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import TournamentCompletionSection from '@/components/common/TournamentCompletionSection.vue'
import type { StandingItem } from '@/types/tournament'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, queryApi, statsApi, type BracketInfo, type RecordPerformanceParams, type MsiTeamGroups } from '@/api/tauri'
import { PowerEngine } from '@/engines/PowerEngine'
import type { MatchDetail } from '@/types/matchDetail'
import type { Player, PlayerPosition } from '@/types/player'
import { createLogger } from '@/utils/logger'
import { useBatchSimulation } from '@/composables/useBatchSimulation'

const logger = createLogger('ShanghaiDetail')

const route = useRoute()
const router = useRouter()

const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()
const timeStore = useTimeStore()

// 从 query 获取赛季（赛事管理页传入），否则使用当前赛季
const viewingSeason = computed(() => Number(route.query.season) || gameStore.gameState?.current_season || 1)

// 后端数据状态
const loading = ref(false)
const currentTournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)
const teamMap = ref<Map<number, { name: string; regionCode: string }>>(new Map())
const shanghaiQualifiedTeams = ref<MsiTeamGroups | null>(null)

// Mock 上海大师赛数据
const mockBracket = reactive({
  id: '1',
  seasonId: '1',
  seasonYear: 2024,
  status: 'in_progress' as 'in_progress' | 'completed' | 'not_started',
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
        { id: 'qual1', matchType: 'qualifier', match_order: 1, teamAId: '9', teamBId: '12', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'qual2', matchType: 'qualifier', match_order: 2, teamAId: '10', teamBId: '11', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'chal1', matchType: 'challenger', match_order: 1, teamAId: '5', teamBId: '8', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'chal2', matchType: 'challenger', match_order: 2, teamAId: '6', teamBId: '7', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
      ]
    },
    // 败者组
    {
      roundNumber: 2,
      roundName: '败者组',
      status: 'pending',
      matches: [
        { id: 'lr1_1', matchType: 'loser_r1', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr1_2', matchType: 'loser_r1', match_order: 2, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr2_1', matchType: 'loser_r2', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr2_2', matchType: 'loser_r2', match_order: 2, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr3_1', matchType: 'loser_r3', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr3_2', matchType: 'loser_r3', match_order: 2, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lr4', matchType: 'loser_r4', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'lf', matchType: 'loser_final', match_order: 1, teamAId: null as string | null, teamBId: null as string | null, status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
      ]
    },
    // 胜者组 (传奇组)
    {
      roundNumber: 3,
      roundName: '胜者组',
      status: 'pending',
      matches: [
        { id: 'wr1_1', matchType: 'winner_r1', match_order: 1, teamAId: '1', teamBId: '4', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
        { id: 'wr1_2', matchType: 'winner_r1', match_order: 2, teamAId: '2', teamBId: '3', status: 'scheduled', bestOf: 5, winnerId: null as string | null, scoreA: 0, scoreB: 0, backendMatchId: null as number | null },
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

// 响应式状态
const { simulationProgress, isSimulating: batchSimulating } = useBatchSimulation()

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 计算属性 - 使用 mock 数据
const currentBracket = computed(() => mockBracket as any)

const shanghaiStandings = computed<StandingItem[]>(() => [
  { rank: 1, label: '冠军', name: currentBracket.value.champion?.teamName || '', points: `+${currentBracket.value.pointsDistribution.champion}分` },
  { rank: 2, label: '亚军', name: currentBracket.value.runnerUp?.teamName || '', points: `+${currentBracket.value.pointsDistribution.runnerUp}分` },
  { rank: 3, label: '季军', name: currentBracket.value.thirdPlace?.teamName || '', points: `+${currentBracket.value.pointsDistribution.thirdPlace}分` },
  { rank: 4, label: '殿军', name: currentBracket.value.fourthPlace?.teamName || '', points: `+${currentBracket.value.pointsDistribution.fourthPlace}分` },
])

// 是否有真实队伍数据（从后端加载）
const hasRealTeamData = computed(() => {
  // 优先检查是否有从 API 获取的参赛队伍数据
  if (shanghaiQualifiedTeams.value) {
    const { legendary, challenger, qualifier } = shanghaiQualifiedTeams.value
    if (legendary.length > 0 || challenger.length > 0 || qualifier.length > 0) {
      return true
    }
  }
  // 其次检查后端是否已经加载了真实的对阵数据
  return bracketData.value !== null && bracketData.value.matches.length > 0
})

// 从后端数据或 mock 数据获取分组队伍
const legendaryGroupTeams = computed(() => {
  // 优先使用从 API 获取的队伍数据
  if (shanghaiQualifiedTeams.value && shanghaiQualifiedTeams.value.legendary.length > 0) {
    return shanghaiQualifiedTeams.value.legendary.map(team => ({
      teamId: String(team.team_id),
      teamName: team.short_name || team.team_name,
      regionName: team.region_name
    }))
  }
  // 其次检查 mockBracket 中是否有真实数据
  if (hasRealTeamData.value && mockBracket.legendaryGroup.length > 0) {
    const firstTeam = mockBracket.legendaryGroup[0]
    if (firstTeam && firstTeam.teamId !== '1') {
      return mockBracket.legendaryGroup
    }
  }
  return []
})

const challengerGroupTeams = computed(() => {
  // 优先使用从 API 获取的队伍数据
  if (shanghaiQualifiedTeams.value && shanghaiQualifiedTeams.value.challenger.length > 0) {
    return shanghaiQualifiedTeams.value.challenger.map(team => ({
      teamId: String(team.team_id),
      teamName: team.short_name || team.team_name,
      regionName: team.region_name
    }))
  }
  // 其次检查 mockBracket 中是否有真实数据
  if (hasRealTeamData.value && mockBracket.challengerGroup.length > 0) {
    const firstTeam = mockBracket.challengerGroup[0]
    if (firstTeam && firstTeam.teamId !== '5') {
      return mockBracket.challengerGroup
    }
  }
  return []
})

const qualifierGroupTeams = computed(() => {
  // 优先使用从 API 获取的队伍数据
  if (shanghaiQualifiedTeams.value && shanghaiQualifiedTeams.value.qualifier.length > 0) {
    return shanghaiQualifiedTeams.value.qualifier.map(team => ({
      teamId: String(team.team_id),
      teamName: team.short_name || team.team_name,
      regionName: team.region_name
    }))
  }
  // 其次检查 mockBracket 中是否有真实数据
  if (hasRealTeamData.value && mockBracket.qualifierGroup.length > 0) {
    const firstTeam = mockBracket.qualifierGroup[0]
    if (firstTeam && firstTeam.teamId !== '9') {
      return mockBracket.qualifierGroup
    }
  }
  return []
})

const goBack = () => {
  router.push('/tournaments')
}

/**
 * 刷新数据（自动清理重复赛事）
 */
const refreshData = async () => {
  try {
    // 先清理重复的上海大师赛
    const deleted = await internationalApi.cleanupDuplicateTournaments('ShanghaiMasters')
    if (deleted > 0) {
      logger.debug(`[ShanghaiDetail] 清理了 ${deleted} 个重复赛事`)
      ElMessage.success(`已清理 ${deleted} 个重复赛事`)
    }
  } catch (error) {
    logger.error('[ShanghaiDetail] 清理重复赛事失败:', error)
  }
  await loadShanghaiData()
  ElMessage.success('数据已刷新')
}

/**
 * 加载上海大师赛数据
 */
const loadShanghaiData = async () => {
  loading.value = true
  try {
    const seasonId = viewingSeason.value

    // 并行获取赛事列表和参赛队伍
    const [tournaments, qualifiedTeams] = await Promise.all([
      queryApi.getInternationalTournaments(seasonId),
      internationalApi.getShanghaiQualifiedTeams(seasonId)
    ])

    // 保存参赛队伍数据
    shanghaiQualifiedTeams.value = qualifiedTeams
    logger.debug('[ShanghaiDetail] Qualified teams:', qualifiedTeams)

    // 查找上海大师赛赛事
    const shanghaiTournament = tournaments.find(t => t.tournament_type === 'ShanghaiMasters')

    if (shanghaiTournament) {
      currentTournamentId.value = shanghaiTournament.id
      mockBracket.seasonYear = seasonId
      // 加载对阵数据
      await loadBracketData()
    } else {
      logger.debug('No Shanghai Masters tournament found for season', seasonId)
    }
  } catch (error) {
    logger.error('Failed to load Shanghai Masters data:', error)
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

    // 构建队伍映射并更新 qualifiedTeams
    teamMap.value.clear()
    const allTeams: any[] = []

    // 首先从 API 获取的参赛队伍添加
    if (shanghaiQualifiedTeams.value) {
      const addTeamIfNotExists = (team: any) => {
        const teamObj = {
          teamId: String(team.team_id),
          teamName: team.short_name || team.team_name,
          regionName: team.region_name
        }
        if (!allTeams.find(t => t.teamId === teamObj.teamId)) {
          allTeams.push(teamObj)
        }
      }
      shanghaiQualifiedTeams.value.legendary.forEach(addTeamIfNotExists)
      shanghaiQualifiedTeams.value.challenger.forEach(addTeamIfNotExists)
      shanghaiQualifiedTeams.value.qualifier.forEach(addTeamIfNotExists)
    }

    // 从对阵数据中补充队伍信息
    bracket.matches.forEach(match => {
      if (match.home_team) {
        teamMap.value.set(match.home_team.id, {
          name: match.home_team.short_name || match.home_team.name,
          regionCode: match.home_team.region_code
        })
        const teamObj = {
          teamId: String(match.home_team.id),
          teamName: match.home_team.short_name || match.home_team.name,
          regionName: match.home_team.region_code
        }
        if (!allTeams.find(t => t.teamId === teamObj.teamId)) {
          allTeams.push(teamObj)
        }
      }
      if (match.away_team) {
        teamMap.value.set(match.away_team.id, {
          name: match.away_team.short_name || match.away_team.name,
          regionCode: match.away_team.region_code
        })
        const teamObj = {
          teamId: String(match.away_team.id),
          teamName: match.away_team.short_name || match.away_team.name,
          regionName: match.away_team.region_code
        }
        if (!allTeams.find(t => t.teamId === teamObj.teamId)) {
          allTeams.push(teamObj)
        }
      }
    })

    // 更新 mockBracket.qualifiedTeams
    if (allTeams.length > 0) {
      mockBracket.qualifiedTeams = allTeams
      logger.debug('[Shanghai] Updated qualifiedTeams:', allTeams.length, 'teams')
    }

    // 更新分组（从 API 数据）
    if (shanghaiQualifiedTeams.value) {
      mockBracket.legendaryGroup = shanghaiQualifiedTeams.value.legendary.map(t => ({
        teamId: String(t.team_id),
        teamName: t.short_name || t.team_name,
        regionName: t.region_name
      }))
      mockBracket.challengerGroup = shanghaiQualifiedTeams.value.challenger.map(t => ({
        teamId: String(t.team_id),
        teamName: t.short_name || t.team_name,
        regionName: t.region_name
      }))
      mockBracket.qualifierGroup = shanghaiQualifiedTeams.value.qualifier.map(t => ({
        teamId: String(t.team_id),
        teamName: t.short_name || t.team_name,
        regionName: t.region_name
      }))
    }

    // 更新 mockBracket 的比赛状态
    updateBracketFromBackend(bracket)
  } catch (error) {
    logger.error('Failed to load bracket data:', error)
  }
}

/**
 * 从后端数据更新对阵
 */
const updateBracketFromBackend = (bracket: BracketInfo) => {
  // 更新赛事状态
  const allCompleted = bracket.matches.every(m => m.status === 'Completed' || m.status === 'COMPLETED')
  const anyStarted = bracket.matches.some(m => m.status === 'Completed' || m.status === 'COMPLETED')
  mockBracket.status = allCompleted ? 'completed' : anyStarted ? 'in_progress' : 'not_started'

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

  bracket.matches.forEach(backendMatch => {
    const matchType = stageToMatchType[backendMatch.stage] || backendMatch.stage.toLowerCase()
    logger.debug('[Shanghai] 处理后端比赛:', backendMatch.stage, '-> matchType:', matchType, 'match_order:', backendMatch.match_order)

    // 在 rounds 中查找对应的比赛
    for (const round of mockBracket.rounds) {
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
        frontendMatch.backendMatchId = backendMatch.match_id
        if (backendMatch.home_team) {
          const teamId = backendMatch.home_team.id
          const teamIdStr = String(teamId)
          const teamName = backendMatch.home_team.short_name || backendMatch.home_team.name
          frontendMatch.teamAId = teamIdStr
          // 确保队伍在 teamMap 中
          teamMap.value.set(teamId, { name: teamName, regionCode: backendMatch.home_team.region_code })
          // 确保队伍在 qualifiedTeams 中
          if (!mockBracket.qualifiedTeams.find(t => String(t.teamId) === teamIdStr)) {
            mockBracket.qualifiedTeams.push({
              teamId: teamIdStr,
              teamName: teamName,
              regionName: backendMatch.home_team.region_code || '',
              seed: 0
            })
          }
        }
        if (backendMatch.away_team) {
          const teamId = backendMatch.away_team.id
          const teamIdStr = String(teamId)
          const teamName = backendMatch.away_team.short_name || backendMatch.away_team.name
          frontendMatch.teamBId = teamIdStr
          // 确保队伍在 teamMap 中
          teamMap.value.set(teamId, { name: teamName, regionCode: backendMatch.away_team.region_code })
          // 确保队伍在 qualifiedTeams 中
          if (!mockBracket.qualifiedTeams.find(t => String(t.teamId) === teamIdStr)) {
            mockBracket.qualifiedTeams.push({
              teamId: teamIdStr,
              teamName: teamName,
              regionName: backendMatch.away_team.region_code || '',
              seed: 0
            })
          }
        }
        frontendMatch.scoreA = backendMatch.home_score
        frontendMatch.scoreB = backendMatch.away_score
        frontendMatch.winnerId = backendMatch.winner_id ? String(backendMatch.winner_id) : null

        // 处理状态 - 后端可能返回大写 'COMPLETED' 或首字母大写 'Completed'
        const backendStatus = backendMatch.status.toUpperCase()
        frontendMatch.status = backendStatus === 'COMPLETED' ? 'completed' :
                              backendStatus === 'INPROGRESS' || backendStatus === 'IN_PROGRESS' ? 'active' : 'scheduled'

        logger.debug('[Shanghai] 更新比赛:', frontendMatch.id, 'status:', frontendMatch.status,
          'score:', frontendMatch.scoreA, '-', frontendMatch.scoreB)
        break
      }
    }
  })
}

/**
 * 将后端 DetailedMatchResult 转换为前端 MatchDetail 格式
 */
const convertBackendToMatchDetail = (result: any, match: any): MatchDetail => {
  // 使用字符串比较，确保类型匹配
  const teamAIdStr = String(match.teamAId)
  const teamBIdStr = String(match.teamBId)

  // 优先从 teamMap 获取队伍名称，其次从 qualifiedTeams
  const getTeamNameById = (teamId: string): string => {
    const numId = Number(teamId)
    if (!isNaN(numId)) {
      const teamFromMap = teamMap.value.get(numId)
      if (teamFromMap) return teamFromMap.name
    }
    const team = mockBracket.qualifiedTeams.find(t => String(t.teamId) === teamId)
    return team?.teamName || '队伍'
  }

  const teamAName = getTeamNameById(teamAIdStr) || result.home_team_name || '队伍A'
  const teamBName = getTeamNameById(teamBIdStr) || result.away_team_name || '队伍B'

  return {
    matchId: match.id,
    tournamentType: 'shanghai',
    seasonId: String(mockBracket.seasonYear),
    teamAId: match.teamAId,
    teamAName,
    teamBId: match.teamBId,
    teamBName,
    bestOf: match.bestOf || 5,
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    winnerName: result.winner_id === result.home_team_id ? teamAName : teamBName,
    games: result.games.map((game: any, index: number) => {
      // 计算队伍战力（选手实际发挥能力平均值）
      const homePlayers = game.home_players || []
      const awayPlayers = game.away_players || []
      const teamAPower = homePlayers.length > 0
        ? homePlayers.reduce((sum: number, p: any) => sum + (p.actual_ability || p.base_ability || 0), 0) / homePlayers.length
        : 0
      const teamBPower = awayPlayers.length > 0
        ? awayPlayers.reduce((sum: number, p: any) => sum + (p.actual_ability || p.base_ability || 0), 0) / awayPlayers.length
        : 0
      const powerDifference = teamAPower - teamBPower

      return {
        gameNumber: game.game_number || index + 1,
        winnerId: String(game.winner_id),
        winnerName: game.winner_id === result.home_team_id ? teamAName : teamBName,
        durationMinutes: game.duration_minutes || 30,
        teamAId: match.teamAId,
        teamAName,
        teamAPower,
        teamAPerformance: game.home_performance,
        teamAMetaPower: game.home_performance,
        teamBId: match.teamBId,
        teamBName,
        teamBPower,
        teamBPerformance: game.away_performance,
        teamBMetaPower: game.away_performance,
        powerDifference,
        performanceDifference: game.home_performance - game.away_performance,
        metaPowerDifference: game.home_performance - game.away_performance,
        isUpset: (powerDifference > 0 && game.winner_id !== result.home_team_id) ||
                 (powerDifference < 0 && game.winner_id === result.home_team_id),
        mvp: game.game_mvp ? {
          playerId: String(game.game_mvp.player_id),
          playerName: game.game_mvp.player_name,
          teamId: String(game.game_mvp.team_id),
          position: game.game_mvp.position,
          mvpScore: game.game_mvp.mvp_score
        } : null,
        teamAPlayers: (game.home_players || []).map((p: any) => ({
          playerId: String(p.player_id),
          playerName: p.player_name,
          position: p.position,
          baseAbility: p.base_ability,
          conditionBonus: p.condition_bonus,
          stabilityNoise: p.stability_noise,
          actualAbility: p.actual_ability,
          kills: p.kills,
          deaths: p.deaths,
          assists: p.assists,
          cs: p.cs,
          gold: p.gold,
          damageDealt: p.damage_dealt,
          damageTaken: p.damage_taken,
          visionScore: p.vision_score,
          mvpScore: p.mvp_score,
          impactScore: p.impact_score,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        })),
        teamBPlayers: (game.away_players || []).map((p: any) => ({
          playerId: String(p.player_id),
          playerName: p.player_name,
          position: p.position,
          baseAbility: p.base_ability,
          conditionBonus: p.condition_bonus,
          stabilityNoise: p.stability_noise,
          actualAbility: p.actual_ability,
          kills: p.kills,
          deaths: p.deaths,
          assists: p.assists,
          cs: p.cs,
          gold: p.gold,
          damageDealt: p.damage_dealt,
          damageTaken: p.damage_taken,
          visionScore: p.vision_score,
          mvpScore: p.mvp_score,
          impactScore: p.impact_score,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        })),
        keyEvents: (game.key_events || []).map((e: any) => ({
          timeMinutes: e.time_minutes,
          eventType: e.event_type,
          description: e.description,
          teamId: String(e.team_id)
        }))
      }
    }),
    matchMvp: result.match_mvp ? {
      playerId: String(result.match_mvp.player_id),
      playerName: result.match_mvp.player_name,
      teamId: String(result.match_mvp.team_id),
      position: result.match_mvp.position,
      mvpScore: result.match_mvp.mvp_score
    } : undefined,
    teamAStats: result.home_team_stats ? {
      totalKills: result.home_team_stats.total_kills,
      totalDeaths: result.home_team_stats.total_deaths,
      totalAssists: result.home_team_stats.total_assists,
      totalGold: result.home_team_stats.total_gold,
      averageGameDuration: result.home_team_stats.average_game_duration,
      firstBloodRate: result.home_team_stats.first_blood_rate,
      firstTowerRate: result.home_team_stats.first_tower_rate,
      baronRate: result.home_team_stats.baron_rate,
      dragonRate: result.home_team_stats.dragon_rate
    } : null,
    teamBStats: result.away_team_stats ? {
      totalKills: result.away_team_stats.total_kills,
      totalDeaths: result.away_team_stats.total_deaths,
      totalAssists: result.away_team_stats.total_assists,
      totalGold: result.away_team_stats.total_gold,
      averageGameDuration: result.away_team_stats.average_game_duration,
      firstBloodRate: result.away_team_stats.first_blood_rate,
      firstTowerRate: result.away_team_stats.first_tower_rate,
      baronRate: result.away_team_stats.baron_rate,
      dragonRate: result.away_team_stats.dragon_rate
    } : null,
    playedAt: new Date().toISOString()
  }
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
 * 模拟单场比赛
 */
const simulateMatch = async (match: any) => {
  // 如果有后端 match ID，使用后端 API 模拟
  if (match.backendMatchId && currentTournamentId.value) {
    try {
      // 使用后端 API 模拟比赛
      const result = await matchApi.simulateMatchDetailed(match.backendMatchId)

      // 更新比赛结果
      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = String(result.winner_id)
      match.status = 'completed'
      match.playedAt = new Date().toISOString()

      // 转换后端结果为 MatchDetail 格式并保存
      const matchDetail = convertBackendToMatchDetail(result, match)
      await matchDetailStore.saveMatchDetail(match.id, matchDetail)

      // 同时用数据库 ID 保存一份，确保能从数据库加载
      if (match.backendMatchId) {
        const dbMatchDetail = { ...matchDetail, matchId: String(match.backendMatchId) }
        await matchDetailStore.saveMatchDetail(match.backendMatchId, dbMatchDetail)
      }

      // 记录选手表现到数据中心系统
      await recordPlayerPerformancesFromBackend(result)

      // 调用后端推进对阵
      await internationalApi.advanceBracket(
        currentTournamentId.value,
        match.backendMatchId,
        result.winner_id
      )

      // 重新加载对阵数据
      await loadBracketData()

      // 获取队伍名称用于显示
      const homeTeamName = teamMap.value.get(result.home_team_id)?.name || '队伍A'
      const awayTeamName = teamMap.value.get(result.away_team_id)?.name || '队伍B'
      ElMessage.success(`比赛完成: ${homeTeamName} ${result.home_score} - ${result.away_score} ${awayTeamName}`)

      // 检查是否全部完成
      await checkShanghaiCompletion()
      return
    } catch (error) {
      logger.error('Backend simulation failed, falling back to local:', error)
      // 后端失败时使用本地 PowerEngine
    }
  }

  // 本地 PowerEngine 模拟 (作为后备方案)
  const teamA = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(match.teamAId))
  const teamB = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(match.teamBId))

  const teamAName = teamA?.teamName || '队伍A'
  const teamBName = teamB?.teamName || '队伍B'

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
  await matchDetailStore.saveMatchDetail(match.id, matchDetail)

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
 * 检查上海大师赛是否完成
 */
const checkShanghaiCompletion = async () => {
  const finalRound = mockBracket.rounds[3]
  const grandFinal = finalRound?.matches.find((m: any) => m.id === 'gf')

  if (grandFinal?.winnerId) {
    mockBracket.status = 'completed'

    // 获取胜负方
    const getLoser = (match: any) => {
      if (!match?.winnerId) return null
      return match.winnerId === match.teamAId ? match.teamBId : match.teamAId
    }

    // 获取败者组决赛
    const loserBracket = mockBracket.rounds[1].matches
    const lf = loserBracket.find((m: any) => m.id === 'lf')
    const lr4 = loserBracket.find((m: any) => m.id === 'lr4')
    const lr3_1 = loserBracket.find((m: any) => m.id === 'lr3_1')
    const lr3_2 = loserBracket.find((m: any) => m.id === 'lr3_2')
    const lr2_1 = loserBracket.find((m: any) => m.id === 'lr2_1')
    const lr2_2 = loserBracket.find((m: any) => m.id === 'lr2_2')

    // 设置最终排名
    const champion = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(grandFinal.winnerId))
    const runnerUp = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(grandFinal)))
    const thirdPlace = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lf)))
    const fourthPlace = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr4)))

    // 败者组排名
    const loserR2Teams: any[] = []
    if (lr3_1?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr3_1)))
      if (loser) loserR2Teams.push(loser)
    }
    if (lr3_2?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr3_2)))
      if (loser) loserR2Teams.push(loser)
    }

    const loserR1Teams: any[] = []
    if (lr2_1?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr2_1)))
      if (loser) loserR1Teams.push(loser)
    }
    if (lr2_2?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr2_2)))
      if (loser) loserR1Teams.push(loser)
    }

    mockBracket.champion = champion || null
    mockBracket.runnerUp = runnerUp || null
    mockBracket.thirdPlace = thirdPlace || null
    mockBracket.fourthPlace = fourthPlace || null
    mockBracket.loserRound2 = loserR2Teams
    mockBracket.loserRound1 = loserR1Teams

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
    logger.debug(`[Shanghai] ${result.message}`)

    // 显示荣誉颁发信息
    if (result.honors_awarded.length > 0) {
      logger.debug('[Shanghai] 颁发的荣誉:')
      result.honors_awarded.forEach(honor => {
        logger.debug(`  - ${honor.honor_type}: ${honor.recipient_name} (${honor.recipient_type})`)
      })
    }

    // 显示积分颁发信息
    if (result.points_awarded.length > 0) {
      logger.debug('[Shanghai] 颁发的年度积分:')
      result.points_awarded.forEach(points => {
        logger.debug(`  - ${points.team_name}: +${points.points}分 (${points.position})`)
      })

      // 显示积分颁发通知
      const topTeams = result.points_awarded.slice(0, 4)
      const pointsMessage = topTeams.map(p => `${p.team_name} +${p.points}分`).join(', ')
      ElMessage.info(`年度积分已更新: ${pointsMessage}`)
    }

  } catch (error) {
    logger.error('[Shanghai] 完成赛事处理失败:', error)
    // 即使失败也不阻止游戏继续，只记录日志
  }
}

/**
 * 从后端模拟结果记录选手表现到数据中心系统
 */
const recordPlayerPerformancesFromBackend = async (result: any) => {
  const seasonId = viewingSeason.value
  const performances: RecordPerformanceParams[] = []

  // 遍历每局比赛的选手表现
  for (const game of result.games) {
    // 主队选手
    for (const player of (game.home_players || [])) {
      performances.push({
        player_id: player.player_id,
        player_name: player.player_name,
        team_id: result.home_team_id,
        position: player.position,
        impact_score: player.impact_score || 0,
        actual_ability: player.actual_ability || 0,
        season_id: Number(seasonId) || 1,
        region_id: 'INTL' // 国际赛事标记
      })
    }

    // 客队选手
    for (const player of (game.away_players || [])) {
      performances.push({
        player_id: player.player_id,
        player_name: player.player_name,
        team_id: result.away_team_id,
        position: player.position,
        impact_score: player.impact_score || 0,
        actual_ability: player.actual_ability || 0,
        season_id: Number(seasonId) || 1,
        region_id: 'INTL' // 国际赛事标记
      })
    }
  }

  // 批量记录到数据库
  if (performances.length > 0) {
    try {
      const count = await statsApi.batchRecordPerformance(performances)
      logger.debug(`[Shanghai] 已记录 ${count} 条选手表现数据`)
    } catch (error) {
      logger.error('[Shanghai] 记录选手表现失败:', error)
    }
  }
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
    const champion = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(gf.winnerId))
    const runnerUp = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(gf)))
    const thirdPlace = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lf)))
    const fourthPlace = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr4)))

    // 败者组第二轮败者 (lr3_1, lr3_2的败者) - 6分
    const loserR2Teams: any[] = []
    if (lr3_1?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr3_1)))
      if (loser) loserR2Teams.push(loser)
    }
    if (lr3_2?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr3_2)))
      if (loser) loserR2Teams.push(loser)
    }

    // 败者组第一轮败者 (lr2_1, lr2_2的败者) - 4分
    const loserR1Teams: any[] = []
    if (lr2_1?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr2_1)))
      if (loser) loserR1Teams.push(loser)
    }
    if (lr2_2?.winnerId) {
      const loser = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(getLoser(lr2_2)))
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

    // 计算总比赛数用于进度
    const totalMatches = mockBracket.rounds.flatMap(r => r.matches).filter(m => m.status !== 'completed').length
    let completed = 0

    // While 循环：每轮从最新响应式数据获取可模拟比赛，直到没有为止
    const MAX_ITERATIONS = 50
    let iterations = 0

    while (iterations < MAX_ITERATIONS) {
      iterations++
      // 从最新的 mockBracket 中获取可模拟的比赛（有队伍且未完成）
      const available = mockBracket.rounds.flatMap(r => r.matches).filter(
        m => m.status !== 'completed' && m.teamAId && m.teamBId
      )

      if (available.length === 0) break

      for (const match of available) {
        await simulateMatch(match)
        completed++
        simulationProgress.value = Math.round((completed / totalMatches) * 100)
        await new Promise(resolve => setTimeout(resolve, 200))
      }
    }

    await checkShanghaiCompletion()
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
const viewMatchDetails = async (match: any) => {
  // 优先使用后端数据库ID，其次使用前端本地ID
  const matchId = match.backendMatchId || match.id

  // 先尝试从内存获取
  let detail = matchDetailStore.getMatchDetail(matchId)
  if (!detail) {
    // 也尝试用前端ID查找（兼容旧数据）
    detail = matchDetailStore.getMatchDetail(match.id)
  }
  if (detail) {
    currentMatchDetail.value = fixMatchDetailTeamNames(detail, match)
    showMatchDetailDialog.value = true
    return
  }

  // 如果内存中没有，尝试从数据库加载（使用后端ID）
  if (match.backendMatchId) {
    detail = await matchDetailStore.loadMatchDetailFromDb(match.backendMatchId)
    if (detail) {
      currentMatchDetail.value = fixMatchDetailTeamNames(detail, match)
      showMatchDetailDialog.value = true
      return
    }
  }

  ElMessage.warning('暂无比赛详情数据，请先模拟比赛')
}

/**
 * 修正比赛详情中空的或不正确的队名和MVP
 */
const fixMatchDetailTeamNames = (detail: MatchDetail, match: any): MatchDetail => {
  const fixedDetail = { ...detail }

  // 修正比赛级别的队名
  if (!fixedDetail.teamAName || fixedDetail.teamAName === '待定' || fixedDetail.teamAName === '') {
    fixedDetail.teamAName = getTeamName(fixedDetail.teamAId) || match?.teamAName || '队伍A'
  }
  if (!fixedDetail.teamBName || fixedDetail.teamBName === '待定' || fixedDetail.teamBName === '') {
    fixedDetail.teamBName = getTeamName(fixedDetail.teamBId) || match?.teamBName || '队伍B'
  }

  // 修正胜者名称
  if (fixedDetail.winnerId) {
    const winnerName = getTeamName(fixedDetail.winnerId)
    if (winnerName && winnerName !== '待定') {
      fixedDetail.winnerName = winnerName
    }
  }

  // 修正每局比赛的队名
  if (fixedDetail.games) {
    fixedDetail.games = fixedDetail.games.map(game => {
      const fixedGame = { ...game }
      if (!fixedGame.teamAName || fixedGame.teamAName === '待定' || fixedGame.teamAName === '') {
        fixedGame.teamAName = getTeamName(fixedGame.teamAId) || fixedDetail.teamAName
      }
      if (!fixedGame.teamBName || fixedGame.teamBName === '待定' || fixedGame.teamBName === '') {
        fixedGame.teamBName = getTeamName(fixedGame.teamBId) || fixedDetail.teamBName
      }
      if (fixedGame.winnerId) {
        const winnerName = getTeamName(fixedGame.winnerId)
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
 * 关闭比赛详情弹窗
 */
const handleCloseMatchDetail = () => {
  showMatchDetailDialog.value = false
  currentMatchDetail.value = null
}

/**
 * 获取队伍名称
 */
const getTeamName = (teamId: string | null): string => {
  if (!teamId) return '待定'
  const team = mockBracket.qualifiedTeams.find(t => String(t.teamId) === String(teamId))
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

// 页面加载时初始化数据
onMounted(() => {
  loadShanghaiData()
})
</script>

<style scoped>
.shanghai-management {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

/* Page header */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.page-header h1 {
  font-size: 22px;
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
  align-items: center;
  gap: 10px;
}

.back-btn {
  background: none;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 6px 14px;
  font-size: 13px;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover {
  color: #6366f1;
  border-color: #6366f1;
}

/* Filter section */
.filter-section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 14px 18px;
  margin-bottom: 20px;
  background: #fff;
}

.filter-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-group label {
  font-size: 15px;
  font-weight: 600;
  color: #0f172a;
}

/* Teams groups */
.teams-groups {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.team-group {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 14px;
  background: #fff;
}

.group-header {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 10px;
  padding-bottom: 8px;
  border-bottom: 2px solid #e2e8f0;
}

.group-header.legendary {
  color: #d97706;
  border-bottom-color: #f59e0b;
}

.group-header.challenger {
  color: #2563eb;
  border-bottom-color: #3b82f6;
}

.group-header.qualifier {
  color: #059669;
  border-bottom-color: #10b981;
}

.group-hint {
  font-weight: 400;
  color: #94a3b8;
  font-size: 12px;
}

.team-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.team-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  background: #f8fafc;
}

.team-item .team-name {
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.team-item.pending {
  justify-content: center;
  border-style: dashed;
}

.team-item.pending .team-name {
  color: #94a3b8;
  font-style: italic;
  font-weight: 400;
}

/* Bracket section */
.bracket-section {
  margin-bottom: 24px;
}

.bracket-placeholder {
  text-align: center;
  padding: 48px 20px;
  border: 1px dashed #e2e8f0;
  border-radius: 10px;
  background: #f8fafc;
}

.bracket-placeholder p {
  margin: 0 0 6px 0;
  color: #64748b;
  font-size: 14px;
}

.bracket-placeholder .placeholder-text {
  color: #94a3b8;
  font-size: 13px;
}

/* Loser standings */
.loser-standings {
  margin-top: 20px;
  margin-bottom: 12px;
}

.loser-standings h4 {
  margin: 0 0 10px 0;
  font-size: 14px;
  font-weight: 600;
  color: #64748b;
}

.loser-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.loser-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  background: #fff;
}

.loser-item .rank-badge .rank-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: #94a3b8;
  color: white;
  font-size: 12px;
  font-weight: 600;
}

.loser-item .team-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.loser-item .points {
  font-size: 13px;
  font-weight: 600;
  color: #6366f1;
}

.loser-item.loser-r2 {
  border-color: #c4b5fd;
  background: #f5f3ff;
}

.loser-item.loser-r1 {
  border-color: #fbcfe8;
  background: #fdf2f8;
}
</style>
