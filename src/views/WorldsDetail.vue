<template>
  <div class="worlds-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <button class="back-btn" @click="goBack">&larr; 返回赛事列表</button>
          <h1 class="page-title">
            S 世界赛 (World Championship)
          </h1>
          <p class="page-description">
            12支队伍（各赛区夏季赛冠亚季军），冠军直通淘汰赛，亚季军打瑞士轮小组赛
          </p>
        </div>
      </div>
      <div class="header-actions">
        <button
          v-if="worldsBracket.status === 'group_stage' && !isGroupStageComplete"
          class="action-btn primary-btn"
          @click="batchSimulateSwissRound"
          :disabled="simulatingSwiss"
        >
          {{ simulatingSwiss ? `模拟中 (${swissSimProgress}%)` : '模拟瑞士轮' }}
        </button>
        <button
          v-if="worldsBracket.status === 'knockout_stage'"
          class="action-btn warning-btn"
          @click="batchSimulateKnockout"
          :disabled="simulatingKnockout"
        >
          {{ simulatingKnockout ? `模拟中 (${koSimProgress}%)` : '模拟淘汰赛' }}
        </button>
      </div>
    </div>

    <!-- 世界赛状态卡片 -->
    <div class="worlds-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>S{{ viewingSeason }} 世界赛</h2>
          <span class="status-badge" :class="getStatusType(worldsBracket.status)">
            {{ getStatusText(worldsBracket.status) }}
          </span>
        </div>
      </div>

      <!-- 参赛队伍统计 -->
      <div class="stats-bar">
        <div class="stat-item"><span class="stat-value">12</span><span class="stat-label">参赛队伍总数</span></div>
        <div class="stat-item"><span class="stat-value">4</span><span class="stat-label">直通淘汰赛</span></div>
        <div class="stat-item"><span class="stat-value">8</span><span class="stat-label">瑞士轮小组赛</span></div>
        <div class="stat-item"><span class="stat-value">8</span><span class="stat-label">淘汰赛名额</span></div>
      </div>

      <!-- 参赛队伍分组 -->
      <div v-if="worldsBracket.status !== 'not_started'" class="table-section">
        <div class="section-header">
          <h3 class="section-title">参赛队伍分组</h3>
        </div>

        <div class="teams-groups">
          <!-- 传奇组（直通淘汰赛） -->
          <div class="team-group legendary">
            <h3>
              传奇组（夏季赛冠军）
            </h3>
            <div class="team-group-desc">直接晋级淘汰赛，保留半区种子位</div>
            <div class="team-list">
              <div v-for="team in directTeams" :key="team.teamId" class="team-item">
                <span class="team-name">{{ team.teamName }}</span>
                <div class="team-badges">
                  <span class="status-badge" :class="getRegionTagType(team.regionId)">
                    {{ team.regionName }}
                  </span>
                  <span v-if="team.quarterSlot" class="status-badge warning">
                    种子{{ team.quarterSlot }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 挑战者组（参加瑞士轮） -->
          <div class="team-group challenger">
            <h3>
              挑战者组（夏季赛亚军+季军）
            </h3>
            <div class="team-group-desc">参加瑞士轮小组赛，争夺4个淘汰赛席位</div>
            <div class="team-list">
              <div v-for="team in groupStageTeams" :key="team.teamId" class="team-item">
                <span class="team-name">{{ team.teamName }}</span>
                <div class="team-badges">
                  <span class="status-badge" :class="getRegionTagType(team.regionId)">
                    {{ team.regionName }}
                  </span>
                  <span class="status-badge info">
                    {{ team.seed === 2 ? '亚军' : '季军' }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 小组赛（瑞士轮）阶段 -->
      <div v-if="worldsBracket.status !== 'not_started'" class="table-section">
        <div class="section-header">
          <h3 class="section-title">小组赛 - 瑞士轮</h3>
          <span v-if="isGroupStageComplete" class="status-badge success">已完成</span>
          <span v-else class="status-badge warning">进行中</span>
        </div>

        <!-- 瑞士轮规则说明 -->
        <div class="swiss-rules">
          <div class="swiss-rules-title">瑞士轮规则</div>
          <div class="swiss-rules-items">
            <span class="rule-item">2胜晋级淘汰赛，2败淘汰，最多3轮</span>
            <span class="rule-divider">|</span>
            <span class="rule-item">相同战绩队伍配对，已对战过的不再相遇</span>
            <span class="rule-divider">|</span>
            <span class="rule-item">BO3赛制</span>
          </div>
        </div>

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
                  <span class="status-badge" :class="getRegionTagType(row.regionId)">
                    {{ row.regionName }}
                  </span>
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
                <span v-if="row.wins >= 2" class="status-badge success">已晋级</span>
                <span v-else-if="row.losses >= 2" class="status-badge danger">已淘汰</span>
                <span v-else class="status-badge info">进行中</span>
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
          <div class="swiss-complete-banner">
            <span class="swiss-complete-icon">&#10003;</span>
            <div class="swiss-complete-body">
              <div class="swiss-complete-title">瑞士轮已完成！</div>
              <div class="swiss-complete-desc">4支队伍以2胜晋级，4支队伍以2败淘汰。现在可以生成淘汰赛对阵。</div>
            </div>
          </div>
          <button
            class="action-btn primary-btn"
            @click="handleGenerateKnockout"
            :disabled="generatingKnockout"
          >
            生成淘汰赛对阵
          </button>
        </div>
      </div>

      <!-- 淘汰赛阶段 -->
      <div v-if="worldsBracket.status === 'knockout_stage' || worldsBracket.status === 'completed'" class="table-section">
        <div class="section-header">
          <h3 class="section-title">淘汰赛</h3>
          <span v-if="worldsBracket.status === 'completed'" class="status-badge success">已完成</span>
          <span v-else class="status-badge warning">进行中</span>
        </div>

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
      </div>

      <!-- 最终排名 -->
      <TournamentCompletionSection
        v-if="worldsBracket.status === 'completed'"
        :standings="worldsStandings"
        banner-title="世界赛已完成！"
        :banner-champion="worldsBracket.champion?.teamName || ''"
        banner-description="获得世界赛冠军，成为世界最强战队！"
      >
        <div class="other-rankings">
          <h4>八强止步（+6分）</h4>
          <div class="teams-list">
            <div v-for="(team, index) in worldsBracket.quarterFinalists" :key="index" class="team-chip">
              {{ team?.teamName }} ({{ team?.regionName }})
            </div>
          </div>
          <h4>小组赛止步（+4分）</h4>
          <div class="teams-list">
            <div v-for="(team, index) in worldsBracket.groupStageEliminated" :key="index" class="team-chip eliminated">
              {{ team?.teamName }} ({{ team?.regionName }})
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
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import WorldsSwissRound from '@/components/worlds/WorldsSwissRound.vue'
import WorldsKnockoutBracket from '@/components/worlds/WorldsKnockoutBracket.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import TournamentCompletionSection from '@/components/common/TournamentCompletionSection.vue'
import type { StandingItem } from '@/types/tournament'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, queryApi, financeApi, statsApi, type BracketInfo, type RecordPerformanceParams } from '@/api/tauri'
import { PowerEngine } from '@/engines/PowerEngine'
import type { MatchDetail } from '@/types/matchDetail'
import type { Player, PlayerPosition } from '@/types/player'
import type { WorldsQualification, SwissStandings, WorldsSwissMatch, WorldsKnockoutMatch } from '@/types/index'
import { createLogger } from '@/utils/logger'
import { useBatchSimulation } from '@/composables/useBatchSimulation'

const logger = createLogger('WorldsDetail')

const router = useRouter()
const route = useRoute()
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

// 响应式状态
const generatingKnockout = ref(false)
const { simulationProgress: swissSimProgress, isSimulating: simulatingSwiss } = useBatchSimulation()
const { simulationProgress: koSimProgress, isSimulating: simulatingKnockout } = useBatchSimulation()
const activeSwissRound = ref('1')
const currentSwissRound = ref(1)

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

/**
 * 获取赛区显示名称
 */
const getRegionDisplayName = (regionCode: string): string => {
  const regionMap: Record<string, string> = {
    'LPL': '中国赛区',
    'LCK': '韩国赛区',
    'LEC': '欧洲赛区',
    'LCS': '北美赛区'
  }
  return regionMap[regionCode] || regionCode
}

/**
 * 生成队伍选手数据（后备方案，当后端模拟失败时使用）
 */
const generateTeamPlayers = (teamId: string, teamName: string, regionId: string): Player[] => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  return positions.map((position, index) => ({
    id: `${teamId}-${position}`,
    gameId: `${teamName}P${index + 1}`,
    name: `${teamName}选手${index + 1}`,
    teamId: teamId,
    teamName,
    position,
    regionId: regionId,
    ability: 70 + Math.floor(Math.random() * 20),
    potential: 80 + Math.floor(Math.random() * 15),
    stability: 70 + Math.floor(Math.random() * 20),
    condition: Math.floor(Math.random() * 10) - 5,
    age: 18 + Math.floor(Math.random() * 8),
    tag: 'NORMAL' as const,
    nationality: regionId === 'LPL' ? '中国' : regionId === 'LCK' ? '韩国' : regionId === 'LEC' ? '欧洲' : '北美'
  }))
}

/**
 * 加载世界赛数据
 */
const loadWorldsData = async () => {
  loading.value = true
  try {
    const seasonId = viewingSeason.value
    // 获取国际赛事列表
    const tournaments = await queryApi.getInternationalTournaments(seasonId)
    // 查找世界赛赛事
    const worldsTournament = tournaments.find(t => t.tournament_type === 'WorldChampionship')

    if (worldsTournament) {
      currentTournamentId.value = worldsTournament.id
      worldsBracket.seasonYear = seasonId
      // 加载对阵数据
      await loadBracketData()
    } else {
      logger.debug('No Worlds tournament found for season', seasonId)
    }
  } catch (error) {
    logger.error('Failed to load Worlds data:', error)
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

    // 更新对阵数据
    updateWorldsBracketFromBackend(bracket)
  } catch (error) {
    logger.error('Failed to load bracket data:', error)
  }
}

/**
 * 从后端数据更新世界赛对阵 - 完全重写，从后端数据创建所有数据
 */
const updateWorldsBracketFromBackend = (bracket: BracketInfo) => {
  logger.debug('[WorldsDetail] updateWorldsBracketFromBackend called, matches:', bracket.matches.length)

  // 辅助函数：检查比赛状态是否为已完成（兼容大小写）
  const isMatchCompleted = (status: string) => status === 'Completed' || status === 'COMPLETED' || status === 'completed'

  worldsBracket.id = String(bracket.tournament_id)

  // 收集所有参赛队伍
  const teamsMap = new Map<number, WorldsQualification>()
  bracket.matches.forEach(match => {
    if (match.home_team) {
      const regionCode = match.home_team.region_code || 'INTL'
      teamsMap.set(match.home_team.id, {
        teamId: String(match.home_team.id),
        teamName: match.home_team.short_name || match.home_team.name,
        regionId: regionCode,
        regionName: getRegionDisplayName(regionCode),
        seed: 1,
        summerPlayoffRank: 1,
        summerPlayoffPoints: 0,
        directToKnockout: false,
        quarterSlot: undefined
      })
    }
    if (match.away_team) {
      const regionCode = match.away_team.region_code || 'INTL'
      teamsMap.set(match.away_team.id, {
        teamId: String(match.away_team.id),
        teamName: match.away_team.short_name || match.away_team.name,
        regionId: regionCode,
        regionName: getRegionDisplayName(regionCode),
        seed: 1,
        summerPlayoffRank: 1,
        summerPlayoffPoints: 0,
        directToKnockout: false,
        quarterSlot: undefined
      })
    }
  })

  // 更新全局队伍列表
  allTeams.value = Array.from(teamsMap.values())
  worldsBracket.qualifiedTeams = allTeams.value

  // 判断赛事状态
  const swissMatches = bracket.matches.filter(m => m.stage.includes('SWISS') || m.stage.includes('Swiss'))
  const knockoutMatches = bracket.matches.filter(m =>
    m.stage.includes('QUARTER') || m.stage.includes('SEMI') ||
    m.stage.includes('FINAL') || m.stage.includes('THIRD') ||
    m.stage.includes('Quarter') || m.stage.includes('Semi') ||
    m.stage.includes('Final') || m.stage.includes('Third')
  )

  logger.debug('[WorldsDetail] Swiss matches:', swissMatches.length, 'Knockout matches:', knockoutMatches.length)

  const allSwissComplete = swissMatches.length > 0 && swissMatches.every(m => m.status && isMatchCompleted(m.status))
  const hasKnockoutTeams = knockoutMatches.some(m => m.home_team !== null && m.away_team !== null)
  const grandFinalMatch = knockoutMatches.find(m => m.stage === 'GRAND_FINAL' || m.stage === 'FINAL')

  if (grandFinalMatch && grandFinalMatch.status && isMatchCompleted(grandFinalMatch.status)) {
    worldsBracket.status = 'completed'
  } else if (hasKnockoutTeams && allSwissComplete) {
    worldsBracket.status = 'knockout_stage'
  } else if (bracket.matches.length > 0) {
    worldsBracket.status = 'group_stage'
  } else {
    worldsBracket.status = 'not_started'
  }

  logger.debug('[WorldsDetail] Status determined:', worldsBracket.status)

  // 转换瑞士轮比赛
  const newSwissMatches: WorldsSwissMatch[] = swissMatches.map(m => {
    // 从 stage 名称提取轮次，如 "SWISS_R1" -> 1
    const roundMatch = m.stage.match(/SWISS_R(\d+)/i)
    const swissRound = roundMatch ? parseInt(roundMatch[1]) : 1

    return {
      id: `swiss-${m.match_id}`,
      backendMatchId: m.match_id,
      competitionId: worldsBracket.id,
      matchType: 'swiss_round',
      stage: 'group',
      bestOf: m.format === 'Bo3' ? 3 : m.format === 'Bo5' ? 5 : 3, // 瑞士轮默认BO3
      swissRound: swissRound,
      roundNumber: swissRound,
      teamAId: m.home_team ? String(m.home_team.id) : '',
      teamAName: m.home_team ? (m.home_team.short_name || m.home_team.name) : '待定',
      teamBId: m.away_team ? String(m.away_team.id) : '',
      teamBName: m.away_team ? (m.away_team.short_name || m.away_team.name) : '待定',
      scoreA: m.home_score,
      scoreB: m.away_score,
      winnerId: m.winner_id ? String(m.winner_id) : undefined,
      status: isMatchCompleted(m.status || '') ? 'completed' : 'scheduled'
    }
  })
  worldsBracket.swissMatches = newSwissMatches

  // 更新当前瑞士轮轮次
  if (newSwissMatches.length > 0) {
    currentSwissRound.value = Math.max(...newSwissMatches.map(m => m.swissRound || 1))
  }

  // 生成瑞士轮积分榜
  const swissTeamIds = new Set<string>()
  newSwissMatches.forEach(m => {
    if (m.teamAId) swissTeamIds.add(m.teamAId)
    if (m.teamBId) swissTeamIds.add(m.teamBId)
  })

  const newSwissStandings: SwissStandings[] = []
  swissTeamIds.forEach(teamId => {
    const teamIdNum = parseInt(teamId)
    if (isNaN(teamIdNum)) return
    const team = teamsMap.get(teamIdNum)
    if (!team) return

    let wins = 0
    let losses = 0
    newSwissMatches.filter(m => m.status === 'completed').forEach(m => {
      if (m.teamAId === teamId) {
        if (m.winnerId === teamId) wins++
        else losses++
      } else if (m.teamBId === teamId) {
        if (m.winnerId === teamId) wins++
        else losses++
      }
    })

    newSwissStandings.push({
      teamId: teamId,
      teamName: team.teamName,
      regionId: team.regionId,
      regionName: team.regionName,
      wins,
      losses,
      record: `${wins}-${losses}`,
      matchesPlayed: wins + losses,
      status: wins >= 2 ? 'qualified' : losses >= 2 ? 'eliminated' : 'active',
      qualified: wins >= 2,
      eliminated: losses >= 2,
      position: 0,
      currentRound: currentSwissRound.value
    })
  })

  // 排序积分榜
  newSwissStandings.sort((a, b) => {
    if (b.wins !== a.wins) return b.wins - a.wins
    if (a.losses !== b.losses) return a.losses - b.losses
    return 0
  })
  newSwissStandings.forEach((s, i) => s.position = i + 1)
  worldsBracket.swissStandings = newSwissStandings

  // 更新直通队伍和小组赛队伍
  // 直通队伍是淘汰赛中有但瑞士轮中没有的队伍
  const knockoutTeamIds = new Set<string>()
  knockoutMatches.forEach(m => {
    if (m.home_team) knockoutTeamIds.add(String(m.home_team.id))
    if (m.away_team) knockoutTeamIds.add(String(m.away_team.id))
  })

  const directTeamIds = new Set<string>()
  knockoutTeamIds.forEach(id => {
    if (!swissTeamIds.has(id)) {
      directTeamIds.add(id)
    }
  })

  // 更新队伍的 directToKnockout 标记
  allTeams.value.forEach(team => {
    team.directToKnockout = directTeamIds.has(team.teamId)
  })

  worldsBracket.directTeams = allTeams.value.filter(t => t.directToKnockout)
  worldsBracket.groupStageTeams = allTeams.value.filter(t => !t.directToKnockout)

  // 转换淘汰赛比赛
  const roundMapping: Record<string, string> = {
    'QUARTER_FINALS': 'QUARTER_FINAL',
    'QUARTER_FINAL': 'QUARTER_FINAL',
    'SEMI_FINALS': 'SEMI_FINAL',
    'SEMI_FINAL': 'SEMI_FINAL',
    'THIRD_PLACE': 'THIRD_PLACE',
    'FINAL': 'FINAL',
    'GRAND_FINAL': 'FINAL'
  }

  const newKnockoutMatches: WorldsKnockoutMatch[] = knockoutMatches.map(m => ({
    id: `knockout-${m.match_id}`,
    backendMatchId: m.match_id,
    competitionId: worldsBracket.id,
    matchType: m.stage.includes('THIRD') ? 'third_place' :
               m.stage.includes('FINAL') ? 'grand_final' :
               m.stage.includes('SEMI') ? 'semi_final' : 'quarter_final',
    stage: 'knockout',
    bestOf: m.format === 'Bo5' ? 5 : m.format === 'Bo3' ? 3 : 5,
    round: roundMapping[m.stage] || m.stage,
    teamAId: m.home_team ? String(m.home_team.id) : '',
    teamAName: m.home_team ? (m.home_team.short_name || m.home_team.name) : '待定',
    teamBId: m.away_team ? String(m.away_team.id) : '',
    teamBName: m.away_team ? (m.away_team.short_name || m.away_team.name) : '待定',
    scoreA: m.home_score,
    scoreB: m.away_score,
    winnerId: m.winner_id ? String(m.winner_id) : undefined,
    status: isMatchCompleted(m.status || '') ? 'completed' : 'scheduled'
  }))
  worldsBracket.knockoutMatches = newKnockoutMatches

  // 设置最终排名
  if (worldsBracket.status === 'completed') {
    const finalMatch = newKnockoutMatches.find(m => m.round === 'FINAL')
    const thirdMatch = newKnockoutMatches.find(m => m.round === 'THIRD_PLACE')

    if (finalMatch && finalMatch.winnerId) {
      worldsBracket.champion = allTeams.value.find(t => t.teamId === finalMatch.winnerId) || null
      const loserId = finalMatch.winnerId === finalMatch.teamAId ? finalMatch.teamBId : finalMatch.teamAId
      worldsBracket.runnerUp = allTeams.value.find(t => t.teamId === loserId) || null
    }

    if (thirdMatch && thirdMatch.winnerId) {
      worldsBracket.thirdPlace = allTeams.value.find(t => t.teamId === thirdMatch.winnerId) || null
      const loserId = thirdMatch.winnerId === thirdMatch.teamAId ? thirdMatch.teamBId : thirdMatch.teamAId
      worldsBracket.fourthPlace = allTeams.value.find(t => t.teamId === loserId) || null
    }

    // 计算八强止步队伍（八强赛败者，排除进入四强的队伍）
    const semifinalistIds = new Set<string>()
    newKnockoutMatches.filter(m => m.round === 'SEMI_FINAL').forEach(m => {
      if (m.teamAId) semifinalistIds.add(String(m.teamAId))
      if (m.teamBId) semifinalistIds.add(String(m.teamBId))
    })

    worldsBracket.quarterFinalists = []
    newKnockoutMatches.filter(m => m.round === 'QUARTER_FINAL' && m.status === 'completed').forEach(m => {
      const loserId = m.winnerId === m.teamAId ? m.teamBId : m.teamAId
      if (loserId && !semifinalistIds.has(String(loserId))) {
        const loser = allTeams.value.find(t => t.teamId === String(loserId))
        if (loser) worldsBracket.quarterFinalists.push(loser)
      }
    })

    // 计算小组赛止步队伍（瑞士轮被淘汰的队伍）
    worldsBracket.groupStageEliminated = newSwissStandings
      .filter(s => s.eliminated)
      .map(s => allTeams.value.find(t => t.teamId === s.teamId))
      .filter((t): t is WorldsQualification => t !== undefined)
  }

  logger.debug('[WorldsDetail] Update complete. Swiss matches:', worldsBracket.swissMatches.length,
    'Knockout matches:', worldsBracket.knockoutMatches.length,
    'Swiss standings:', worldsBracket.swissStandings.length)
}

/**
 * 将后端 DetailedMatchResult 转换为前端 MatchDetail 格式
 */
const convertBackendToMatchDetail = (result: any, match: any): MatchDetail => {
  const teamAInfo = allTeams.value.find(t => t.teamId === match.teamAId)
  const teamBInfo = allTeams.value.find(t => t.teamId === match.teamBId)

  const teamAName = teamAInfo?.teamName || match.teamAName || '队伍A'
  const teamBName = teamBInfo?.teamName || match.teamBName || '队伍B'

  return {
    matchId: match.id,
    tournamentType: 'worlds',
    seasonId: String(worldsBracket.seasonYear),
    teamAId: match.teamAId,
    teamAName,
    teamBId: match.teamBId,
    teamBName,
    bestOf: match.bestOf || 1,
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    winnerName: result.winner_id === result.home_team_id ? teamAName : teamBName,
    games: result.games.map((game: any, index: number) => {
      const homePlayers = game.home_players || []
      const awayPlayers = game.away_players || []
      const teamAPower = homePlayers.length > 0
        ? homePlayers.reduce((sum: number, p: any) => sum + (p.actual_ability || p.base_ability || 70), 0) / homePlayers.length
        : 70
      const teamBPower = awayPlayers.length > 0
        ? awayPlayers.reduce((sum: number, p: any) => sum + (p.actual_ability || p.base_ability || 70), 0) / awayPlayers.length
        : 70
      return {
        gameNumber: game.game_number || index + 1,
        teamAId: match.teamAId,
        teamAName,
        teamAPower,
        teamAPerformance: game.home_performance || 70,
        teamAMetaPower: game.home_performance || undefined,
        teamAPlayers: homePlayers.map((p: any) => ({
          playerId: String(p.player_id),
          playerName: p.player_name,
          position: p.position,
          teamId: match.teamAId,
          baseAbility: p.base_ability || 70,
          conditionBonus: p.condition_bonus || 0,
          stabilityNoise: p.stability_noise || 0,
          actualAbility: p.actual_ability || 70,
          impactScore: p.impact_score || 0,
          mvpScore: p.mvp_score,
          kills: p.kills,
          deaths: p.deaths,
          assists: p.assists,
          cs: p.cs,
          gold: p.gold,
          damageDealt: p.damage_dealt,
          damageTaken: p.damage_taken,
          visionScore: p.vision_score,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        })),
        teamBId: match.teamBId,
        teamBName,
        teamBPower,
        teamBPerformance: game.away_performance || 70,
        teamBMetaPower: game.away_performance || undefined,
        teamBPlayers: (game.away_players || []).map((p: any) => ({
          playerId: String(p.player_id),
          playerName: p.player_name,
          position: p.position,
          teamId: match.teamBId,
          baseAbility: p.base_ability || 70,
          conditionBonus: p.condition_bonus || 0,
          stabilityNoise: p.stability_noise || 0,
          actualAbility: p.actual_ability || 70,
          impactScore: p.impact_score || 0,
          mvpScore: p.mvp_score,
          kills: p.kills,
          deaths: p.deaths,
          assists: p.assists,
          cs: p.cs,
          gold: p.gold,
          damageDealt: p.damage_dealt,
          damageTaken: p.damage_taken,
          visionScore: p.vision_score,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        })),
        winnerId: String(game.winner_id),
        winnerName: game.winner_id === result.home_team_id ? teamAName : teamBName,
        powerDifference: teamAPower - teamBPower,
        performanceDifference: (game.home_performance || 0) - (game.away_performance || 0),
        metaPowerDifference: (game.home_performance || 0) - (game.away_performance || 0),
        isUpset: (teamAPower < teamBPower && game.winner_id === result.home_team_id) ||
                 (teamAPower > teamBPower && game.winner_id === result.away_team_id)
      }
    }),
    mvpPlayerId: result.match_mvp ? String(result.match_mvp.player_id) : undefined,
    mvpPlayerName: result.match_mvp?.player_name,
    mvpTeamId: result.match_mvp ? String(result.match_mvp.team_id) : undefined,
    mvpTotalImpact: result.match_mvp?.mvp_score,
    playedAt: new Date().toISOString()
  }
}

// 世界赛数据 - 初始化为空，从后端加载
const allTeams = ref<WorldsQualification[]>([])
const directTeams = computed(() => allTeams.value.filter(t => t.directToKnockout))
const groupStageTeams = computed(() => allTeams.value.filter(t => !t.directToKnockout))

const worldsBracket = reactive({
  id: '',
  seasonId: 'S1',
  seasonYear: 2024,
  status: 'not_started' as 'not_started' | 'group_stage' | 'knockout_stage' | 'completed',
  qualifiedTeams: [] as WorldsQualification[],
  directTeams: [] as WorldsQualification[],
  groupStageTeams: [] as WorldsQualification[],
  swissMatches: [] as WorldsSwissMatch[],
  swissStandings: [] as SwissStandings[],
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

const worldsStandings = computed<StandingItem[]>(() => [
  { rank: 1, label: '冠军', name: worldsBracket.champion?.teamName || '', regionName: worldsBracket.champion?.regionName, points: '+20分' },
  { rank: 2, label: '亚军', name: worldsBracket.runnerUp?.teamName || '', regionName: worldsBracket.runnerUp?.regionName, points: '+16分' },
  { rank: 3, label: '季军', name: worldsBracket.thirdPlace?.teamName || '', regionName: worldsBracket.thirdPlace?.regionName, points: '+12分' },
  { rank: 4, label: '殿军', name: worldsBracket.fourthPlace?.teamName || '', regionName: worldsBracket.fourthPlace?.regionName, points: '+8分' },
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
 * 模拟瑞士轮单场比赛（优先使用后端 API）
 */
const handleSimulateSwissMatch = async (match: WorldsSwissMatch) => {
  // 如果有后端 match ID，使用后端 API 模拟
  if ((match as any).backendMatchId && currentTournamentId.value) {
    try {
      const result = await matchApi.simulateMatchDetailed((match as any).backendMatchId)

      // 更新比赛结果
      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = String(result.winner_id)
      match.status = 'completed'

      // 转换后端结果为 MatchDetail 格式并保存
      const matchDetail = convertBackendToMatchDetail(result, match)
      matchDetailStore.saveMatchDetail(match.id, matchDetail)

      // 记录选手表现到数据中心
      await recordPlayerPerformancesFromBackend(result)

      // 调用后端推进对阵
      await internationalApi.advanceBracket(
        currentTournamentId.value,
        (match as any).backendMatchId,
        result.winner_id
      )

      // 重新加载对阵数据（会自动重新计算积分榜，无需再调用 updateSwissStandings）
      await loadBracketData()

      ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

      // 检查当前轮是否完成，生成下一轮
      checkSwissRoundCompletion()
      return
    } catch (error) {
      logger.error('Backend simulation failed, falling back to local:', error)
      // 后端失败时使用本地 PowerEngine
    }
  }

  // 本地 PowerEngine 模拟 (作为后备方案)
  // 获取队伍赛区信息
  const teamAInfo = allTeams.value.find(t => t.teamId === match.teamAId)
  const teamBInfo = allTeams.value.find(t => t.teamId === match.teamBId)
  const regionAId = teamAInfo?.regionId || 'INTL'
  const regionBId = teamBInfo?.regionId || 'INTL'

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(match.teamAId, match.teamAName, regionAId)
  const teamBPlayers = generateTeamPlayers(match.teamBId, match.teamBName, regionBId)

  // 使用 PowerEngine 模拟比赛 (BO3)
  const matchDetail = PowerEngine.simulateMatch(
    match.teamAId,
    match.teamAName,
    teamAPlayers,
    match.teamBId,
    match.teamBName,
    teamBPlayers,
    3 // BO3
  )

  // 更新比赛结果
  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'

  // 同步本地模拟结果到数据库
  if ((match as any).backendMatchId) {
    try {
      await matchApi.updateMatchResult(
        (match as any).backendMatchId,
        matchDetail.finalScoreA,
        matchDetail.finalScoreB,
        parseInt(matchDetail.winnerId)
      )
      logger.debug('[WorldsDetail] Swiss match local simulation synced to database')
    } catch (syncError) {
      logger.error('Failed to sync Swiss match to database:', syncError)
    }
  }

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
        String(match.teamAId || ''),
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
        String(match.teamBId || ''),
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
const checkSwissRoundCompletion = async () => {
  const currentRoundMatches = getSwissRoundMatches(currentSwissRound.value)
  const allComplete = currentRoundMatches.every(m => m.status === 'completed')

  if (allComplete && currentSwissRound.value < 3 && !isGroupStageComplete.value) {
    // 生成下一轮对阵
    await generateNextSwissRound()
  }
}

/**
 * 生成下一轮瑞士轮对阵（调用后端 API）
 */
const generateNextSwissRound = async () => {
  if (!currentTournamentId.value) {
    logger.error('No tournament ID')
    return
  }

  try {
    // 调用后端 API 生成下一轮比赛
    const newMatchIds = await internationalApi.generateNextSwissRound(currentTournamentId.value)
    logger.debug('[WorldsDetail] Generated next Swiss round, new match IDs:', newMatchIds)

    // 重新加载对阵数据
    await loadBracketData()

    const nextRound = currentSwissRound.value + 1
    currentSwissRound.value = nextRound
    activeSwissRound.value = String(nextRound)

    ElMessage.success(`已生成瑞士轮第 ${nextRound} 轮对阵`)
  } catch (error) {
    logger.error('Failed to generate next Swiss round:', error)
    ElMessage.error('生成下一轮对阵失败')
  }
}

/**
 * 生成淘汰赛对阵（调用后端 API）
 */
const handleGenerateKnockout = async () => {
  if (!currentTournamentId.value) {
    ElMessage.error('赛事ID不存在')
    return
  }

  generatingKnockout.value = true

  try {
    // 获取晋级的4支队伍ID
    const qualifiedTeamIds = worldsBracket.swissStandings
      .filter(s => s.qualified)
      .map(s => parseInt(s.teamId))

    if (qualifiedTeamIds.length !== 4) {
      ElMessage.error(`需要4支晋级队伍，当前只有 ${qualifiedTeamIds.length} 支`)
      return
    }

    // 调用后端 API 填充淘汰赛对阵
    await internationalApi.fillWorldsKnockoutBracket(currentTournamentId.value, qualifiedTeamIds)

    // 重新加载对阵数据
    await loadBracketData()

    ElMessage.success('淘汰赛对阵生成成功!')
  } catch (error) {
    logger.error('Failed to generate knockout bracket:', error)
    ElMessage.error('生成淘汰赛对阵失败')
  } finally {
    generatingKnockout.value = false
  }
}

/**
 * 模拟淘汰赛单场比赛（优先使用后端 API）
 */
const handleSimulateKnockoutMatch = async (match: WorldsKnockoutMatch) => {
  // 如果有后端 match ID，使用后端 API 模拟
  if ((match as any).backendMatchId && currentTournamentId.value) {
    try {
      const result = await matchApi.simulateMatchDetailed((match as any).backendMatchId)

      // 更新比赛结果
      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = String(result.winner_id)
      match.status = 'completed'

      // 转换后端结果为 MatchDetail 格式并保存
      const matchDetail = convertBackendToMatchDetail(result, match)
      matchDetailStore.saveMatchDetail(match.id, matchDetail)

      // 记录选手表现到数据中心
      await recordPlayerPerformancesFromBackend(result)

      // 调用后端推进对阵
      await internationalApi.advanceBracket(
        currentTournamentId.value,
        (match as any).backendMatchId,
        result.winner_id
      )

      // 重新加载对阵数据（后端已通过 advanceBracket 更新后续对阵）
      await loadBracketData()

      ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

      // 注意：不再调用 updateKnockoutBracket(match)
      // 因为后端的 advanceBracket 已经处理了对阵推进，loadBracketData 已刷新所有数据

      // 检查是否完成
      await checkKnockoutCompletion()
      return
    } catch (error) {
      logger.error('Backend simulation failed, falling back to local:', error)
      // 后端失败时使用本地 PowerEngine
    }
  }

  // 本地 PowerEngine 模拟 (作为后备方案)
  // 获取队伍赛区信息
  const teamAInfo = allTeams.value.find(t => t.teamId === match.teamAId)
  const teamBInfo = allTeams.value.find(t => t.teamId === match.teamBId)
  const regionAId = teamAInfo?.regionId || 'INTL'
  const regionBId = teamBInfo?.regionId || 'INTL'

  // 生成选手数据
  const teamAPlayers = generateTeamPlayers(String(match.teamAId || ''), match.teamAName || '', regionAId)
  const teamBPlayers = generateTeamPlayers(String(match.teamBId || ''), match.teamBName || '', regionBId)

  // 使用 PowerEngine 模拟比赛 (BO5)
  const matchDetail = PowerEngine.simulateMatch(
    String(match.teamAId || ''),
    match.teamAName || '',
    teamAPlayers,
    String(match.teamBId || ''),
    match.teamBName || '',
    teamBPlayers,
    match.bestOf || 5
  )

  // 更新比赛结果
  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'

  // 同步本地模拟结果到数据库
  if ((match as any).backendMatchId) {
    try {
      await matchApi.updateMatchResult(
        (match as any).backendMatchId,
        matchDetail.finalScoreA,
        matchDetail.finalScoreB,
        parseInt(matchDetail.winnerId)
      )
      logger.debug('[WorldsDetail] Knockout match local simulation synced to database')
    } catch (syncError) {
      logger.error('Failed to sync knockout match to database:', syncError)
    }
  }

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
        String(match.teamAId || ''),
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
        String(match.teamBId || ''),
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
  await checkKnockoutCompletion()
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
const checkKnockoutCompletion = async () => {
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

    // 调用后端 completeTournament 命令处理荣誉殿堂、年度积分和财政系统
    if (currentTournamentId.value) {
      await processTournamentCompletion(currentTournamentId.value)
    }

    showChampionCelebration(worldsBracket.champion?.teamName || '')

    // 刷新时间状态（用户可在全局控制面板推进阶段）
    await timeStore.fetchTimeState()
  }
}

/**
 * 批量模拟瑞士轮 - 使用完整模拟引擎
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
    swissSimProgress.value = 0

    let totalMatches = 0
    let completedMatches = 0

    // 先统计总比赛数
    for (let round = 1; round <= 3; round++) {
      totalMatches += getSwissRoundMatches(round).filter(m => m.status !== 'completed').length
    }

    while (!isGroupStageComplete.value && currentSwissRound.value <= 3) {
      const currentMatches = getSwissRoundMatches(currentSwissRound.value)
      const uncompletedMatches = currentMatches.filter(m => m.status !== 'completed')

      for (const match of uncompletedMatches) {
        // 使用完整的模拟引擎（与单场模拟相同的逻辑）
        await handleSimulateSwissMatch(match)
        completedMatches++
        swissSimProgress.value = Math.floor((completedMatches / Math.max(totalMatches, 1)) * 100)
        // 添加短暂延迟，让UI有时间更新
        await new Promise(resolve => setTimeout(resolve, 100))
      }

      // 检查是否需要生成下一轮
      checkSwissRoundCompletion()

      // 更新总比赛数（可能有新生成的比赛）
      const newMatches = getSwissRoundMatches(currentSwissRound.value).filter(m => m.status !== 'completed').length
      if (newMatches > 0) {
        totalMatches += newMatches
      }
    }

    ElMessage.success('瑞士轮模拟完成！')
  } catch (error: any) {
    if (error !== 'cancel') {
      logger.error('瑞士轮模拟失败:', error)
      ElMessage.error('瑞士轮模拟失败')
    }
  } finally {
    simulatingSwiss.value = false
    swissSimProgress.value = 0
  }
}

/**
 * 批量模拟淘汰赛 - 使用完整模拟引擎
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
    koSimProgress.value = 0

    const stages = ['QUARTER_FINAL', 'SEMI_FINAL', 'THIRD_PLACE', 'FINAL']
    let totalMatches = 0
    let completedMatches = 0

    // 统计总比赛数
    for (const stage of stages) {
      totalMatches += worldsBracket.knockoutMatches.filter(
        m => m.round === stage && m.status !== 'completed'
      ).length
    }

    for (const stage of stages) {
      // 每个阶段可能需要等待前一阶段完成才能确定队伍
      const stageMatches = worldsBracket.knockoutMatches.filter(m => m.round === stage)

      for (const match of stageMatches) {
        // 检查队伍是否已确定（可能需要等待前一阶段结果）
        if (match.status !== 'completed' && match.teamAId && match.teamBId) {
          // 使用完整的模拟引擎（与单场模拟相同的逻辑）
          await handleSimulateKnockoutMatch(match)
          completedMatches++
          koSimProgress.value = Math.floor((completedMatches / Math.max(totalMatches, 1)) * 100)
          // 添加短暂延迟，让UI有时间更新
          await new Promise(resolve => setTimeout(resolve, 150))
        }
      }
    }

    await checkKnockoutCompletion()
    ElMessage.success('淘汰赛模拟完成！')
  } catch (error: any) {
    if (error !== 'cancel') {
      logger.error('淘汰赛模拟失败:', error)
      ElMessage.error('淘汰赛模拟失败')
    }
  } finally {
    simulatingKnockout.value = false
    koSimProgress.value = 0
  }
}

/**
 * 查看比赛详情
 */
const viewMatchDetails = async (match: any) => {
  // 先尝试从内存获取
  let detail = matchDetailStore.getMatchDetail(match.id)
  if (detail) {
    currentMatchDetail.value = detail
    showMatchDetailDialog.value = true
    return
  }
  // 如果内存中没有，尝试从数据库加载
  detail = await matchDetailStore.loadMatchDetailFromDb(match.id)
  if (detail) {
    currentMatchDetail.value = detail
    showMatchDetailDialog.value = true
    return
  }
  ElMessage.warning('暂无比赛详情数据，请先模拟比赛')
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

/**
 * 处理赛事完成 - 荣誉殿堂 + 年度积分 + 财政系统
 */
const processTournamentCompletion = async (tournamentId: number) => {
  try {
    // 调用后端 completeTournament 命令处理荣誉殿堂和年度积分
    const result = await internationalApi.completeTournament(tournamentId)
    logger.debug(`[Worlds] ${result.message}`)

    // 输出荣誉信息
    if (result.honors_awarded.length > 0) {
      logger.debug('[Worlds] 颁发的荣誉:')
      result.honors_awarded.forEach(honor => {
        logger.debug(`  - ${honor.honor_type}: ${honor.recipient_name} (${honor.recipient_type})`)
      })
    }

    // 输出年度积分信息
    if (result.points_awarded.length > 0) {
      logger.debug('[Worlds] 颁发的年度积分:')
      result.points_awarded.forEach(points => {
        logger.debug(`  - ${points.team_name}: +${points.points}分 (${points.position})`)
      })
      // 显示前4名的积分变化
      const topTeams = result.points_awarded.slice(0, 4)
      const pointsMessage = topTeams.map(p => `${p.team_name} +${p.points}分`).join(', ')
      ElMessage.info(`年度积分已更新: ${pointsMessage}`)
    }

    // 分发赛事奖金
    try {
      await financeApi.distributeTournamentPrizes(tournamentId)
      logger.debug('[Worlds] 赛事奖金已分发')
      ElMessage.success('赛事奖金已分发给各参赛队伍')
    } catch (financeError) {
      logger.error('[Worlds] 奖金分发失败:', financeError)
    }
  } catch (error) {
    logger.error('[Worlds] 完成赛事处理失败:', error)
  }
}

/**
 * 从后端比赛结果记录选手表现数据到数据中心
 */
const recordPlayerPerformancesFromBackend = async (result: any) => {
  const seasonId = viewingSeason.value
  const performances: RecordPerformanceParams[] = []

  // 遍历所有比赛，收集选手表现数据
  for (const game of result.games) {
    // 处理主队选手
    for (const player of (game.home_players || [])) {
      performances.push({
        player_id: player.player_id,
        player_name: player.player_name,
        team_id: result.home_team_id,
        position: player.position,
        impact_score: player.impact_score || 0,
        actual_ability: player.actual_ability || 0,
        season_id: Number(seasonId) || 1,
        region_id: 'INTL'
      })
    }
    // 处理客队选手
    for (const player of (game.away_players || [])) {
      performances.push({
        player_id: player.player_id,
        player_name: player.player_name,
        team_id: result.away_team_id,
        position: player.position,
        impact_score: player.impact_score || 0,
        actual_ability: player.actual_ability || 0,
        season_id: Number(seasonId) || 1,
        region_id: 'INTL'
      })
    }
  }

  // 批量记录到数据中心
  if (performances.length > 0) {
    try {
      const count = await statsApi.batchRecordPerformance(performances)
      logger.debug(`[Worlds] 已记录 ${count} 条选手表现数据`)
    } catch (error) {
      logger.error('[Worlds] 记录选手表现失败:', error)
    }
  }
}

// 页面加载时初始化数据
onMounted(() => {
  loadWorldsData()
})
</script>

<style scoped>
.worlds-management { padding: 24px; }

.worlds-management .page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 24px; }
.worlds-management .header-left { display: flex; flex-direction: column; gap: 8px; align-items: flex-start; }
.worlds-management .page-title { display: flex; align-items: center; gap: 8px; font-size: 28px; font-weight: 700; margin: 0; color: #0f172a; }
.worlds-management .page-description { margin: 0; color: #64748b; font-size: 14px; }
.worlds-management .header-actions { display: flex; align-items: center; gap: 12px; }

.worlds-management .worlds-status-card { background: #ffffff; border-radius: 12px; padding: 24px; border: 1px solid #e2e8f0; }

.worlds-management .status-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid #e2e8f0; }
.worlds-management .status-header .status-info { display: flex; align-items: center; gap: 16px; }
.worlds-management .status-header .status-info h2 { margin: 0; font-size: 20px; font-weight: 600; color: #0f172a; }

.worlds-management .teams-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin-bottom: 32px; padding: 20px; background: #f8fafc; border-radius: 12px; border: 1px solid #e2e8f0; }

.worlds-management .stage-card { margin-bottom: 24px; }
.worlds-management .stage-card .card-header { display: flex; justify-content: space-between; align-items: center; }

.worlds-management .teams-groups { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin: 12px 0; }

.worlds-management .team-group { padding: 12px; border-radius: 8px; border: 1px solid #e2e8f0; }
.worlds-management .team-group h3 { display: flex; align-items: center; gap: 6px; margin: 0 0 4px 0; font-size: 14px; font-weight: 600; }
.worlds-management .team-group .team-group-desc { font-size: 11px; margin-bottom: 8px; color: #94a3b8; }

.worlds-management .team-list { display: flex; flex-direction: column; gap: 4px; }
.worlds-management .team-item { display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: #ffffff; border-radius: 6px; }
.worlds-management .team-item .team-name { font-weight: 500; font-size: 13px; color: #0f172a; }
.worlds-management .team-badges { display: flex; gap: 4px; align-items: center; }

.worlds-management .team-group.legendary { border-color: #fcd34d; background: #fffef5; }
.worlds-management .team-group.legendary h3 { color: #d97706; }
.worlds-management .team-group.legendary .team-group-desc { color: #92400e; }

.worlds-management .team-group.challenger { border-color: #c7d2fe; background: #f8f9ff; }
.worlds-management .team-group.challenger h3 { color: #6366f1; }
.worlds-management .team-group.challenger .team-group-desc { color: #6366f1; }

.worlds-management .swiss-rules { display: flex; align-items: center; gap: 10px; padding: 10px 14px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; margin-bottom: 16px; }
.worlds-management .swiss-rules-title { font-size: 13px; font-weight: 600; color: #0f172a; white-space: nowrap; }
.worlds-management .swiss-rules-items { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; font-size: 12px; color: #64748b; }
.worlds-management .rule-divider { color: #cbd5e1; }

.worlds-management .swiss-standings { margin: 16px 0; }
.worlds-management .swiss-standings h4 { margin: 0 0 10px 0; font-size: 14px; font-weight: 600; color: #0f172a; }
.worlds-management .standings-table .team-cell { display: flex; align-items: center; gap: 8px; }
.worlds-management .standings-table .team-cell .team-name { font-weight: 600; color: #0f172a; }
.worlds-management .standings-table .rank-number { font-weight: 600; }
.worlds-management .standings-table .record { font-weight: 600; color: #0f172a; }

.worlds-management .swiss-matches { margin-top: 16px; }

.worlds-management .generate-knockout-section { margin-top: 20px; text-align: center; }
.worlds-management .swiss-complete-banner { display: flex; align-items: flex-start; gap: 10px; padding: 12px 16px; background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 8px; margin-bottom: 12px; text-align: left; }
.worlds-management .swiss-complete-icon { flex-shrink: 0; width: 22px; height: 22px; line-height: 22px; text-align: center; font-size: 13px; font-weight: 700; color: #ffffff; background: #22c55e; border-radius: 50%; }
.worlds-management .swiss-complete-title { font-size: 14px; font-weight: 600; color: #166534; margin-bottom: 2px; }
.worlds-management .swiss-complete-desc { font-size: 12px; color: #16a34a; line-height: 1.5; }

.worlds-management .knockout-info { margin-bottom: 20px; }
.worlds-management .knockout-info ul { margin: 10px 0 0 0; padding-left: 20px; }
.worlds-management .knockout-info li { margin: 5px 0; }

.worlds-management .knockout-brackets { margin-top: 24px; }

.worlds-management .other-rankings { margin-bottom: 24px; }
.worlds-management .other-rankings h4 { font-size: 16px; font-weight: 600; color: #0f172a; margin: 20px 0 12px 0; }
.worlds-management .teams-list { display: flex; flex-wrap: wrap; gap: 12px; }
.worlds-management .team-chip { padding: 8px 16px; background: #f1f5f9; border: 1px solid #e2e8f0; border-radius: 20px; font-size: 14px; color: #0f172a; }
.worlds-management .team-chip.eliminated { opacity: 0.7; }

.worlds-management .mb-4 { margin-bottom: 16px; }

/* Design system utility classes */
.back-btn { display: inline-flex; align-items: center; gap: 6px; padding: 6px 14px; font-size: 13px; font-weight: 500; color: #64748b; background: #f1f5f9; border: 1px solid #e2e8f0; border-radius: 6px; cursor: pointer; }
.back-btn:hover { background: #e2e8f0; }
.action-btn { padding: 6px 16px; font-size: 13px; font-weight: 500; border-radius: 6px; cursor: pointer; border: 1px solid #e2e8f0; background: #ffffff; color: #0f172a; }
.action-btn:hover { background: #f8fafc; }
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.primary-btn { background: #6366f1; color: #ffffff; border-color: #6366f1; }
.primary-btn:hover { background: #4f46e5; }
.warning-btn { background: #f59e0b; color: #ffffff; border-color: #f59e0b; }
.warning-btn:hover { background: #d97706; }
.success-btn { background: #22c55e; color: #ffffff; border-color: #22c55e; }
.success-btn:hover { background: #16a34a; }
.status-badge { display: inline-block; padding: 2px 8px; font-size: 12px; font-weight: 500; border-radius: 8px; background: #f1f5f9; color: #64748b; }
.status-badge.success { background: #f0fdf4; color: #16a34a; }
.status-badge.warning { background: #fffbeb; color: #d97706; }
.status-badge.danger { background: #fef2f2; color: #ef4444; }
.status-badge.info { background: #f1f5f9; color: #64748b; }
.stats-bar { display: flex; gap: 24px; padding: 16px 20px; background: #f8fafc; border-radius: 8px; border: 1px solid #e2e8f0; }
.stat-item { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.stat-value { font-size: 20px; font-weight: 700; color: #0f172a; }
.stat-label { font-size: 12px; color: #94a3b8; }
.table-section { background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; margin-bottom: 16px; }
.section-header { display: flex; justify-content: space-between; align-items: center; padding: 14px 16px; border-bottom: 1px solid #e2e8f0; }
.section-title { margin: 0; font-size: 15px; font-weight: 600; color: #0f172a; }
.section-content { padding: 16px; }
</style>
