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
            <li>BO3赛制</li>
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
      :visible="showMatchDetailDialog"
      :match-detail="currentMatchDetail"
      @update:visible="showMatchDetailDialog = $event"
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
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, queryApi, financeApi, statsApi, type BracketInfo, type RecordPerformanceParams } from '@/api/tauri'
import { PowerEngine } from '@/engines/PowerEngine'
import type { MatchDetail } from '@/types/matchDetail'
import type { Player, PlayerPosition } from '@/types/player'
import type { WorldsQualification, SwissStandings, WorldsSwissMatch, WorldsKnockoutMatch } from '@/types/index'
import { createLogger } from '@/utils/logger'

const logger = createLogger('WorldsDetail')

const router = useRouter()
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()
const timeStore = useTimeStore()

// 后端数据状态
const loading = ref(false)
const currentTournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)
const teamMap = ref<Map<number, { name: string; regionCode: string }>>(new Map())

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
    const seasonId = gameStore.gameState?.current_season || 1
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
      const teamAPower = game.home_performance || 70
      const teamBPower = game.away_performance || 70
      return {
        gameNumber: game.game_number || index + 1,
        teamAId: match.teamAId,
        teamAName,
        teamAPower,
        teamAPerformance: game.home_performance || 70,
        teamAPlayers: (game.home_players || []).map((p: any) => ({
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
    simulationProgress.value = 0

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
        simulationProgress.value = Math.floor((completedMatches / Math.max(totalMatches, 1)) * 100)
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
    simulationProgress.value = 0
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
    simulationProgress.value = 0

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
          simulationProgress.value = Math.floor((completedMatches / Math.max(totalMatches, 1)) * 100)
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
    simulationProgress.value = 0
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
  const seasonId = gameStore.gameState?.current_season || worldsBracket.seasonYear
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
