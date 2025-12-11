<template>
  <div class="icp-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <el-button text @click="goBack">
            <el-icon><ArrowLeft /></el-icon>
            返回赛事列表
          </el-button>
          <h1 class="page-title">
            <el-icon><Flag /></el-icon>
            ICP 四赛区洲际对抗赛 (Intercontinental Championship)
          </h1>
          <p class="page-description">
            16支队伍（各赛区夏季赛前4名），按种子分组BO3单循环，决出最强赛区
          </p>
        </div>
      </div>
      <div class="header-actions">
        <el-button
          v-if="icpTournament.status === 'group_stage' && !isGroupStageComplete"
          type="primary"
          @click="batchSimulateGroupStage"
          :loading="simulatingGroupStage"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingGroupStage ? `模拟中 (${simulationProgress}%)` : '模拟种子组赛' }}
        </el-button>
        <el-button
          v-if="icpTournament.status === 'region_battle'"
          type="warning"
          @click="batchSimulateRegionBattle"
          :loading="simulatingRegionBattle"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingRegionBattle ? `模拟中 (${simulationProgress}%)` : '模拟赛区对决' }}
        </el-button>
      </div>
    </div>

    <!-- ICP赛事状态卡片 -->
    <div class="icp-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>ICP 四赛区洲际对抗赛</h2>
          <el-tag :type="getStatusType(icpTournament.status)" size="large">
            {{ getStatusText(icpTournament.status) }}
          </el-tag>
        </div>
      </div>

      <!-- 参赛队伍统计 -->
      <div class="teams-stats">
        <el-statistic title="参赛队伍总数" :value="16" />
        <el-statistic title="参赛赛区" :value="4" suffix="个" />
        <el-statistic title="种子组数量" :value="4" suffix="组" />
        <el-statistic title="每赛区队伍" :value="4" suffix="支" />
      </div>

      <!-- 赛区徽章统计 -->
      <div class="region-badges-section" v-if="icpTournament.status !== 'not_started'">
        <h3>赛区徽章统计</h3>
        <div class="region-badges-grid">
          <div
            v-for="region in sortedRegionStats"
            :key="region.region"
            class="region-badge-card"
            :class="{ champion: region.ranking === 1 }"
          >
            <div class="region-flag" :class="region.region.toLowerCase()">
              {{ getRegionFlag(region.region) }}
            </div>
            <div class="region-name">{{ region.regionName }}</div>
            <div class="badge-count">
              <span class="badge-icon">🏅</span>
              <span class="badge-number">{{ region.totalBadges }}</span>
            </div>
            <div v-if="region.ranking" class="region-rank">
              第{{ region.ranking }}名
            </div>
          </div>
        </div>
      </div>

      <!-- 种子组赛阶段 -->
      <el-card v-if="icpTournament.status !== 'not_started'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🎯 种子组赛阶段</span>
            <el-tag v-if="isGroupStageComplete" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <!-- 种子组积分榜 -->
        <div class="seed-groups">
          <el-tabs v-model="activeSeedGroup" type="card">
            <el-tab-pane
              v-for="group in icpTournament.seedGroups"
              :key="group.groupName"
              :label="`${getSeedGroupLabel(group.groupName)}组`"
              :name="group.groupName"
            >
              <ICPSeedGroupStanding
                :group="group"
                @simulate-match="handleSimulateMatch"
                @view-match="viewMatchDetails"
              />
            </el-tab-pane>
          </el-tabs>
        </div>

        <!-- 生成赛区对决按钮 -->
        <div v-if="isGroupStageComplete && icpTournament.status === 'group_stage'" class="generate-region-battle-section">
          <el-alert
            title="种子组赛已完成！"
            description="所有种子组比赛已完成，各组前2名获得徽章。现在可以进入赛区对决阶段。"
            type="success"
            :closable="false"
            show-icon
            class="mb-4"
          />
          <el-button
            type="primary"
            size="large"
            @click="handleGenerateRegionBattle"
            :loading="generatingRegionBattle"
          >
            <el-icon><Flag /></el-icon>
            进入赛区对决
          </el-button>
        </div>
      </el-card>

      <!-- 赛区对决阶段 -->
      <el-card v-if="icpTournament.status === 'region_battle' || icpTournament.status === 'completed'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🏆 赛区对决阶段</span>
            <el-tag v-if="icpTournament.status === 'completed'" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <!-- 赛区对决对阵 -->
        <div class="region-battle-section">
          <!-- 半决赛（如果需要） -->
          <div v-if="icpTournament.semifinal" class="battle-stage">
            <h4>🥊 半决赛</h4>
            <ICPRegionBattleCard
              :battle="icpTournament.semifinal"
              @simulate-match="handleSimulateRegionMatch"
              @view-match="viewMatchDetails"
            />
          </div>

          <!-- 决赛 -->
          <div v-if="icpTournament.final" class="battle-stage final">
            <h4>🏆 决赛</h4>
            <ICPRegionBattleCard
              :battle="icpTournament.final"
              @simulate-match="handleSimulateRegionMatch"
              @view-match="viewMatchDetails"
            />
          </div>
        </div>
      </el-card>

      <!-- 最终排名 -->
      <div v-if="icpTournament.status === 'completed'" class="final-standings">
        <h3>赛区最终排名与积分</h3>
        <div class="standings-grid">
          <div class="standing-item champion">
            <div class="rank-badge">🏆 最强赛区</div>
            <div class="region-flag large" :class="icpTournament.champion?.region.toLowerCase()">
              {{ getRegionFlag(icpTournament.champion?.region || '') }}
            </div>
            <div class="region-name">{{ icpTournament.champion?.regionName }}</div>
            <div class="points-detail">
              <div>参赛队伍: +12分</div>
              <div>未参赛队伍: +6分</div>
            </div>
          </div>

          <div class="standing-item runner-up">
            <div class="rank-badge">🥈 第二名</div>
            <div class="region-flag large" :class="icpTournament.runnerUp?.region.toLowerCase()">
              {{ getRegionFlag(icpTournament.runnerUp?.region || '') }}
            </div>
            <div class="region-name">{{ icpTournament.runnerUp?.regionName }}</div>
            <div class="points-detail">
              <div>参赛队伍: +8分</div>
              <div>未参赛队伍: +4分</div>
            </div>
          </div>

          <div class="standing-item third">
            <div class="rank-badge">🥉 第三名</div>
            <div class="region-flag large" :class="icpTournament.thirdPlace?.region.toLowerCase()">
              {{ getRegionFlag(icpTournament.thirdPlace?.region || '') }}
            </div>
            <div class="region-name">{{ icpTournament.thirdPlace?.regionName }}</div>
            <div class="points-detail">
              <div>参赛队伍: +6分</div>
              <div>未参赛队伍: +3分</div>
            </div>
          </div>

          <div class="standing-item fourth">
            <div class="rank-badge">4️⃣ 第四名</div>
            <div class="region-flag large" :class="icpTournament.fourthPlace?.region.toLowerCase()">
              {{ getRegionFlag(icpTournament.fourthPlace?.region || '') }}
            </div>
            <div class="region-name">{{ icpTournament.fourthPlace?.regionName }}</div>
            <div class="points-detail">
              <div>参赛队伍: +4分</div>
              <div>未参赛队伍: +2分</div>
            </div>
          </div>
        </div>

        <!-- ICP完成后的操作区 -->
        <div class="icp-completed-actions">
          <el-alert
            title="ICP洲际对抗赛已完成！"
            type="success"
            :closable="false"
            show-icon
            class="completion-alert"
          >
            <template #default>
              <p>恭喜 <strong>{{ icpTournament.champion?.regionName }}</strong> 成为本届最强赛区！</p>
            </template>
          </el-alert>
        </div>
      </div>
    </div>

    <!-- 比赛详情弹窗 -->
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
  Flag,
  ArrowLeft,
  DArrowRight
} from '@element-plus/icons-vue'
import ICPSeedGroupStanding from '@/components/icp/ICPSeedGroupStanding.vue'
import ICPRegionBattleCard from '@/components/icp/ICPRegionBattleCard.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { PowerEngine } from '@/engines/PowerEngine'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { internationalApi, matchApi, queryApi } from '@/api/tauri'
import type { BracketInfo, MatchBracketInfo, GroupStandingInfo, DetailedMatchResult, DetailedGameResult, PlayerGameStats } from '@/api/tauri'
import type { Player, PlayerPosition } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import type { ICPTournament, ICPSeedGroup, ICPMatch, ICPRegionStats, ICPRegionMatch, ICPGroupStanding } from '@/types/icp'

const router = useRouter()
const route = useRoute()
const gameStore = useGameStore()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 后端数据状态
const tournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)
const groupStandings = ref<GroupStandingInfo[]>([])
const loading = ref(false)
const teamMap = ref<Map<number, { name: string; regionCode: string }>>(new Map())

// 响应式状态
const generatingRegionBattle = ref(false)
const simulatingGroupStage = ref(false)
const simulatingRegionBattle = ref(false)
const simulationProgress = ref(0)
const activeSeedGroup = ref('A')

// 生成模拟的种子组数据
const generateSeedGroupData = (): ICPSeedGroup[] => {
  // 各赛区夏季赛前4名
  const lplTeams = ['JDG', 'BLG', 'TES', 'WBG']
  const lckTeams = ['T1', 'GEN', 'DK', 'KT']
  const lecTeams = ['G2', 'FNC', 'MAD', 'BDS']
  const lcsTeams = ['C9', 'TL', '100T', 'FLY']

  const regions = ['LPL', 'LCK', 'LEC', 'LCS']
  const allTeams = [lplTeams, lckTeams, lecTeams, lcsTeams]

  const seedGroups: ICPSeedGroup[] = []
  const groupNames: ('A' | 'B' | 'C' | 'D')[] = ['A', 'B', 'C', 'D']

  // A组 = 一号种子，B组 = 二号种子...
  groupNames.forEach((groupName, seedIndex) => {
    const teams: { id: string; name: string; region: string }[] = []

    // 从每个赛区取对应种子位的队伍
    regions.forEach((region, regionIndex) => {
      teams.push({
        id: `${region.toLowerCase()}-seed${seedIndex + 1}`,
        name: allTeams[regionIndex][seedIndex],
        region: region
      })
    })

    // 生成积分榜
    const standings: ICPGroupStanding[] = teams.map((team, i) => ({
      teamId: team.id,
      teamName: team.name,
      region: team.region,
      seed: seedIndex + 1,
      position: i + 1,
      matchesPlayed: 0,
      wins: 0,
      losses: 0,
      points: 0,
      roundsWon: 0,
      roundsLost: 0,
      roundDifferential: 0,
      hasBadge: false
    }))

    // 生成组内BO3单循环比赛（6场比赛）
    const matches: ICPMatch[] = []
    let matchId = 1

    for (let i = 0; i < teams.length; i++) {
      for (let j = i + 1; j < teams.length; j++) {
        matches.push({
          id: `${groupName}-${matchId++}`,
          teamAId: teams[i].id,
          teamAName: teams[i].name,
          teamARegion: teams[i].region,
          teamBId: teams[j].id,
          teamBName: teams[j].name,
          teamBRegion: teams[j].region,
          scoreA: 0,
          scoreB: 0,
          winnerId: null,
          status: 'scheduled',
          bestOf: 3,
          stage: 'group',
          groupName: groupName,
          roundNumber: matchId - 1
        })
      }
    }

    seedGroups.push({
      groupName,
      seedNumber: seedIndex + 1,
      standings,
      matches,
      isComplete: false
    })
  })

  return seedGroups
}

// 生成赛区统计数据
const generateRegionStats = (): ICPRegionStats[] => {
  const lplTeams = ['JDG', 'BLG', 'TES', 'WBG']
  const lckTeams = ['T1', 'GEN', 'DK', 'KT']
  const lecTeams = ['G2', 'FNC', 'MAD', 'BDS']
  const lcsTeams = ['C9', 'TL', '100T', 'FLY']

  return [
    {
      region: 'LPL',
      regionName: '中国赛区 (LPL)',
      totalBadges: 0,
      teams: lplTeams.map((name, i) => ({
        id: `lpl-seed${i + 1}`,
        name,
        region: 'LPL',
        seed: i + 1,
        badges: 0
      }))
    },
    {
      region: 'LCK',
      regionName: '韩国赛区 (LCK)',
      totalBadges: 0,
      teams: lckTeams.map((name, i) => ({
        id: `lck-seed${i + 1}`,
        name,
        region: 'LCK',
        seed: i + 1,
        badges: 0
      }))
    },
    {
      region: 'LEC',
      regionName: '欧洲赛区 (LEC)',
      totalBadges: 0,
      teams: lecTeams.map((name, i) => ({
        id: `lec-seed${i + 1}`,
        name,
        region: 'LEC',
        seed: i + 1,
        badges: 0
      }))
    },
    {
      region: 'LCS',
      regionName: '北美赛区 (LCS)',
      totalBadges: 0,
      teams: lcsTeams.map((name, i) => ({
        id: `lcs-seed${i + 1}`,
        name,
        region: 'LCS',
        seed: i + 1,
        badges: 0
      }))
    }
  ]
}

// ICP赛事数据
const icpTournament = reactive<ICPTournament>({
  id: '1',
  seasonYear: 2024,
  status: 'group_stage',
  seedGroups: generateSeedGroupData(),
  regionStats: generateRegionStats(),
  semifinal: undefined,
  final: undefined,
  champion: undefined,
  runnerUp: undefined,
  thirdPlace: undefined,
  fourthPlace: undefined
})

// 计算属性
const isGroupStageComplete = computed(() => {
  return icpTournament.seedGroups.every(group => {
    return group.matches.every(match => match.status === 'completed')
  })
})

const sortedRegionStats = computed(() => {
  return [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
})

// 方法
const goBack = () => {
  router.push('/tournaments')
}

const getStatusType = (status: string) => {
  const typeMap: Record<string, any> = {
    'not_started': 'info',
    'group_stage': 'warning',
    'region_battle': 'warning',
    'completed': 'success'
  }
  return typeMap[status] || 'info'
}

const getStatusText = (status: string) => {
  const textMap: Record<string, string> = {
    'not_started': '未开始',
    'group_stage': '种子组赛进行中',
    'region_battle': '赛区对决进行中',
    'completed': '已完成'
  }
  return textMap[status] || status
}

const getSeedGroupLabel = (groupName: string) => {
  const labelMap: Record<string, string> = {
    'A': '一号种子',
    'B': '二号种子',
    'C': '三号种子',
    'D': '四号种子'
  }
  return labelMap[groupName] || groupName
}

const getRegionFlag = (region: string) => {
  const flagMap: Record<string, string> = {
    'LPL': '🇨🇳',
    'LCK': '🇰🇷',
    'LEC': '🇪🇺',
    'LCS': '🇺🇸'
  }
  return flagMap[region] || '🏳️'
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
 */
const viewMatchDetails = (match: ICPMatch) => {
  if (match.status === 'completed') {
    const detail = matchDetailStore.getMatchDetail(match.id)
    if (detail) {
      currentMatchDetail.value = detail
      showMatchDetailDialog.value = true
      return
    }
  }
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
 * 转换后端比赛结果到前端 MatchDetail 格式
 */
const convertToMatchDetail = (result: DetailedMatchResult, matchId: string): MatchDetail => {
  const games = result.games.map((game: DetailedGameResult, idx: number) => ({
    gameNumber: idx + 1,
    winnerId: game.winner_id.toString(),
    duration: game.duration,
    teamAPlayers: game.team_a_players.map((p: PlayerGameStats) => ({
      playerId: p.player_id.toString(),
      playerName: p.player_name,
      position: p.position as PlayerPosition,
      baseAbility: p.base_ability,
      actualAbility: p.actual_ability,
      impactScore: p.impact_score,
      condition: p.condition,
      stability: p.stability,
      stabilityNoise: p.stability_noise,
      kills: p.kills,
      deaths: p.deaths,
      assists: p.assists,
      gold: p.gold,
      damage: p.damage,
      cs: p.cs,
      visionScore: p.vision_score
    })),
    teamBPlayers: game.team_b_players.map((p: PlayerGameStats) => ({
      playerId: p.player_id.toString(),
      playerName: p.player_name,
      position: p.position as PlayerPosition,
      baseAbility: p.base_ability,
      actualAbility: p.actual_ability,
      impactScore: p.impact_score,
      condition: p.condition,
      stability: p.stability,
      stabilityNoise: p.stability_noise,
      kills: p.kills,
      deaths: p.deaths,
      assists: p.assists,
      gold: p.gold,
      damage: p.damage,
      cs: p.cs,
      visionScore: p.vision_score
    })),
    teamAPerformance: game.team_a_performance,
    teamBPerformance: game.team_b_performance,
    performanceDiff: game.performance_diff,
    gameNoise: game.game_noise,
    mvpPlayerId: game.mvp_player_id?.toString(),
    mvpPlayerName: game.mvp_player_name,
    mvpTeamId: game.mvp_team_id?.toString()
  }))

  return {
    matchId,
    teamAId: result.team_a_id.toString(),
    teamAName: result.team_a_name,
    teamBId: result.team_b_id.toString(),
    teamBName: result.team_b_name,
    bestOf: result.best_of,
    finalScoreA: result.final_score_a,
    finalScoreB: result.final_score_b,
    winnerId: result.winner_id.toString(),
    games,
    tournamentType: 'icp',
    seasonId: String(icpTournament.seasonYear)
  }
}

/**
 * 模拟单场比赛
 */
const handleSimulateMatch = async (match: ICPMatch) => {
  // 尝试使用后端 API
  if (tournamentId.value && bracketData.value) {
    try {
      // 找到后端对应的 matchId
      const backendMatchId = findBackendMatchId(match)

      if (backendMatchId) {
        const result = await matchApi.simulateMatchDetailed(backendMatchId)

        if (result) {
          // 转换为前端格式
          const matchDetail = convertToMatchDetail(result, match.id)

          // 更新比赛状态
          match.scoreA = result.final_score_a
          match.scoreB = result.final_score_b
          match.winnerId = result.winner_id.toString()
          match.status = 'completed'
          match.completedAt = new Date()

          // 保存比赛详情
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
                String(icpTournament.seasonYear),
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
                String(icpTournament.seasonYear),
                'INTL'
              )
            })
          })

          // 推进对阵
          await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)

          // 更新积分榜
          updateGroupStandings(match)

          ElMessage.success(`比赛完成: ${match.teamAName} ${result.final_score_a} - ${result.final_score_b} ${match.teamBName}`)

          // 检查该组是否完成
          checkGroupCompletion()
          return
        }
      }
    } catch (error) {
      console.warn('后端 API 模拟失败，使用本地引擎:', error)
    }
  }

  // 后备: 使用 PowerEngine 本地模拟
  const teamAId = String(match.teamAId || '')
  const teamBId = String(match.teamBId || '')
  const teamAPlayers = generateTeamPlayers(teamAId, match.teamAName || '', match.teamARegion || '')
  const teamBPlayers = generateTeamPlayers(teamBId, match.teamBName || '', match.teamBRegion || '')

  const matchDetail = PowerEngine.simulateMatch(
    teamAId,
    match.teamAName || '',
    teamAPlayers,
    teamBId,
    match.teamBName || '',
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
  matchDetail.tournamentType = 'icp'
  matchDetail.seasonId = String(icpTournament.seasonYear)
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
        String(icpTournament.seasonYear),
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
        String(icpTournament.seasonYear),
        'INTL'
      )
    })
  })

  // 更新积分榜
  updateGroupStandings(match)

  ElMessage.success(`比赛完成: ${match.teamAName} ${matchDetail.finalScoreA} - ${matchDetail.finalScoreB} ${match.teamBName}`)

  // 检查该组是否完成
  checkGroupCompletion()
}

/**
 * 查找后端对应的 matchId
 */
const findBackendMatchId = (match: ICPMatch): number | null => {
  if (!bracketData.value) return null

  // 在所有比赛中查找匹配的
  const allMatches = bracketData.value.matches || []

  for (const m of allMatches) {
    // 根据队伍名称匹配
    const teamAName = teamMap.value.get(m.team_a_id || 0)?.name
    const teamBName = teamMap.value.get(m.team_b_id || 0)?.name

    if ((teamAName === match.teamAName && teamBName === match.teamBName) ||
        (teamAName === match.teamBName && teamBName === match.teamAName)) {
      return m.id
    }
  }

  return null
}

/**
 * 更新积分榜
 */
const updateGroupStandings = (match: ICPMatch) => {
  const group = icpTournament.seedGroups.find(g => g.groupName === match.groupName)
  if (!group) return

  const teamA = group.standings.find(s => s.teamId === match.teamAId)
  const teamB = group.standings.find(s => s.teamId === match.teamBId)

  if (teamA && teamB && match.scoreA !== undefined && match.scoreB !== undefined) {
    // 更新比赛场次
    teamA.matchesPlayed++
    teamB.matchesPlayed++

    // 更新小局数
    teamA.roundsWon += match.scoreA
    teamA.roundsLost += match.scoreB
    teamB.roundsWon += match.scoreB
    teamB.roundsLost += match.scoreA

    // 更新净胜局
    teamA.roundDifferential = teamA.roundsWon - teamA.roundsLost
    teamB.roundDifferential = teamB.roundsWon - teamB.roundsLost

    // 更新胜负和积分
    if (match.scoreA > match.scoreB) {
      teamA.wins++
      teamB.losses++
      // 2:0 得3分，2:1 得2分
      teamA.points += match.scoreA === 2 && match.scoreB === 0 ? 3 : 2
      // 1:2 得1分，0:2 得0分
      teamB.points += match.scoreB === 1 ? 1 : 0
    } else {
      teamB.wins++
      teamA.losses++
      teamB.points += match.scoreB === 2 && match.scoreA === 0 ? 3 : 2
      teamA.points += match.scoreA === 1 ? 1 : 0
    }

    // 重新排序积分榜
    group.standings.sort((a, b) => {
      if (b.points !== a.points) return b.points - a.points
      if (b.roundDifferential !== a.roundDifferential) return b.roundDifferential - a.roundDifferential
      return b.roundsWon - a.roundsWon
    })

    // 更新排名
    group.standings.forEach((s, i) => {
      s.position = i + 1
      s.hasBadge = i < 2 // 前两名获得徽章
    })
  }
}

/**
 * 检查组别是否完成并更新赛区徽章
 */
const checkGroupCompletion = () => {
  icpTournament.seedGroups.forEach(group => {
    const isComplete = group.matches.every(m => m.status === 'completed')
    group.isComplete = isComplete

    if (isComplete) {
      // 更新赛区徽章统计
      group.standings.forEach(standing => {
        if (standing.hasBadge) {
          const region = icpTournament.regionStats.find(r => r.region === standing.region)
          if (region) {
            const team = region.teams.find(t => t.id === standing.teamId)
            if (team && team.badges === 0) {
              team.badges = 1
              region.totalBadges++
            }
          }
        }
      })
    }
  })
}

/**
 * 生成赛区对决
 */
const handleGenerateRegionBattle = async () => {
  generatingRegionBattle.value = true

  try {
    await new Promise(resolve => setTimeout(resolve, 500))

    // 根据徽章数量排序赛区
    const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)

    // 分配排名
    sortedRegions.forEach((region, index) => {
      region.ranking = index + 1
    })

    // 根据徽章排名决定对决方式
    const canDistinguishTop2 = sortedRegions[0].totalBadges > sortedRegions[1].totalBadges
    const canDistinguishTop3 = sortedRegions[1].totalBadges > sortedRegions[2].totalBadges

    if (canDistinguishTop2) {
      // 可以分出前两名，直接进行决赛
      icpTournament.final = createRegionBattle(
        sortedRegions[0],
        sortedRegions[1],
        'final'
      )
    } else if (canDistinguishTop3) {
      // 可以排出前三名，后两名先半决赛
      icpTournament.semifinal = createRegionBattle(
        sortedRegions[1],
        sortedRegions[2],
        'semifinal'
      )
      // 决赛待定（半决赛胜者 vs 第一名）
    } else {
      // 徽章数量无法明确区分，默认第一第二进行决赛
      icpTournament.final = createRegionBattle(
        sortedRegions[0],
        sortedRegions[1],
        'final'
      )
    }

    icpTournament.status = 'region_battle'
    ElMessage.success('赛区对决生成成功！')
  } finally {
    generatingRegionBattle.value = false
  }
}

/**
 * 创建赛区对决
 */
const createRegionBattle = (
  regionA: ICPRegionStats,
  regionB: ICPRegionStats,
  stage: 'semifinal' | 'final'
): ICPRegionMatch => {
  // 创建四场BO5对决（一号种子vs一号种子...）
  const matches: ICPMatch[] = []

  for (let seed = 1; seed <= 4; seed++) {
    const teamA = regionA.teams.find(t => t.seed === seed)
    const teamB = regionB.teams.find(t => t.seed === seed)

    if (teamA && teamB) {
      matches.push({
        id: `${stage}-seed${seed}`,
        teamAId: teamA.id,
        teamAName: teamA.name,
        teamARegion: regionA.region,
        teamBId: teamB.id,
        teamBName: teamB.name,
        teamBRegion: regionB.region,
        scoreA: 0,
        scoreB: 0,
        winnerId: null,
        status: 'scheduled',
        bestOf: 5,
        stage: stage
      })
    }
  }

  return {
    id: `${stage}-${regionA.region}-vs-${regionB.region}`,
    regionA: regionA.region,
    regionB: regionB.region,
    regionAName: regionA.regionName,
    regionBName: regionB.regionName,
    matches,
    regionAWins: 0,
    regionBWins: 0,
    winnerId: null,
    status: 'scheduled',
    stage
  }
}

/**
 * 模拟赛区对决中的单场比赛
 */
const handleSimulateRegionMatch = async (battle: ICPRegionMatch, match: ICPMatch) => {
  // 尝试使用后端 API
  if (tournamentId.value && bracketData.value) {
    try {
      const backendMatchId = findBackendMatchId(match)

      if (backendMatchId) {
        const result = await matchApi.simulateMatchDetailed(backendMatchId)

        if (result) {
          const matchDetail = convertToMatchDetail(result, match.id)

          match.scoreA = result.final_score_a
          match.scoreB = result.final_score_b
          match.winnerId = result.winner_id.toString()
          match.status = 'completed'
          match.completedAt = new Date()

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
                String(icpTournament.seasonYear),
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
                String(icpTournament.seasonYear),
                'INTL'
              )
            })
          })

          // 推进对阵
          await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)

          // 更新赛区胜场
          if (match.teamARegion === battle.regionA) {
            if (result.final_score_a > result.final_score_b) battle.regionAWins++
            else battle.regionBWins++
          } else {
            if (result.final_score_a > result.final_score_b) battle.regionBWins++
            else battle.regionAWins++
          }

          ElMessage.success(`比赛完成: ${match.teamAName} ${result.final_score_a} - ${result.final_score_b} ${match.teamBName}`)

          checkRegionBattleCompletion(battle)
          return
        }
      }
    } catch (error) {
      console.warn('后端 API 模拟失败，使用本地引擎:', error)
    }
  }

  // 后备: 使用 PowerEngine 本地模拟 (BO5)
  const teamAId = String(match.teamAId || '')
  const teamBId = String(match.teamBId || '')
  const teamAPlayers = generateTeamPlayers(teamAId, match.teamAName || '', match.teamARegion || '')
  const teamBPlayers = generateTeamPlayers(teamBId, match.teamBName || '', match.teamBRegion || '')

  const matchDetail = PowerEngine.simulateMatch(
    teamAId,
    match.teamAName || '',
    teamAPlayers,
    teamBId,
    match.teamBName || '',
    teamBPlayers,
    match.bestOf || 5
  )

  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'
  match.completedAt = new Date()

  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'icp'
  matchDetail.seasonId = String(icpTournament.seasonYear)
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
        String(icpTournament.seasonYear),
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
        String(icpTournament.seasonYear),
        'INTL'
      )
    })
  })

  // 更新赛区胜场
  if (match.teamARegion === battle.regionA) {
    if (matchDetail.finalScoreA > matchDetail.finalScoreB) battle.regionAWins++
    else battle.regionBWins++
  } else {
    if (matchDetail.finalScoreA > matchDetail.finalScoreB) battle.regionBWins++
    else battle.regionAWins++
  }

  ElMessage.success(`比赛完成: ${match.teamAName} ${matchDetail.finalScoreA} - ${matchDetail.finalScoreB} ${match.teamBName}`)

  checkRegionBattleCompletion(battle)
}

/**
 * 检查赛区对决是否结束
 */
const checkRegionBattleCompletion = (battle: ICPRegionMatch) => {
  const allComplete = battle.matches.every(m => m.status === 'completed')

  if (allComplete) {
    // 判定胜者
    if (battle.regionAWins > battle.regionBWins) {
      battle.winnerId = battle.regionA
    } else if (battle.regionBWins > battle.regionAWins) {
      battle.winnerId = battle.regionB
    } else {
      // 平局，进行一号种子加赛（简化处理，随机决定）
      battle.winnerId = Math.random() > 0.5 ? battle.regionA : battle.regionB
      ElMessage.info('比分相同，一号种子加赛决出胜者！')
    }

    battle.status = 'completed'

    // 检查是否需要进行决赛或整体结束
    checkTournamentCompletion()
  }
}

/**
 * 检查比赛是否整体结束
 */
const checkTournamentCompletion = () => {
  // 如果有半决赛且已完成，设置决赛
  if (icpTournament.semifinal?.status === 'completed' && !icpTournament.final) {
    const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)
    const semifinalWinner = icpTournament.regionStats.find(r => r.region === icpTournament.semifinal?.winnerId)

    if (semifinalWinner) {
      icpTournament.final = createRegionBattle(
        sortedRegions[0], // 第一名赛区
        semifinalWinner,
        'final'
      )
    }
    return
  }

  // 如果决赛已完成
  if (icpTournament.final?.status === 'completed') {
    const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)

    // 设置最终排名
    const finalWinner = icpTournament.final.winnerId
    const finalLoser = finalWinner === icpTournament.final.regionA
      ? icpTournament.final.regionB
      : icpTournament.final.regionA

    icpTournament.champion = icpTournament.regionStats.find(r => r.region === finalWinner)
    icpTournament.runnerUp = icpTournament.regionStats.find(r => r.region === finalLoser)

    // 设置第三第四名
    const remainingRegions = sortedRegions.filter(
      r => r.region !== finalWinner && r.region !== finalLoser
    )
    icpTournament.thirdPlace = remainingRegions[0]
    icpTournament.fourthPlace = remainingRegions[1]

    icpTournament.status = 'completed'
    showChampionCelebration(icpTournament.champion?.regionName || '')
  }
}

/**
 * 批量模拟种子组赛
 */
const batchSimulateGroupStage = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的种子组比赛。是否继续?',
      '模拟种子组赛确认',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    simulatingGroupStage.value = true
    simulationProgress.value = 0

    const allMatches = icpTournament.seedGroups.flatMap(g => g.matches)
    const uncompletedMatches = allMatches.filter(m => m.status !== 'completed')

    for (let i = 0; i < uncompletedMatches.length; i++) {
      const match = uncompletedMatches[i]
      await simulateMatchInternal(match)
      simulationProgress.value = Math.floor(((i + 1) / uncompletedMatches.length) * 100)
      await new Promise(resolve => setTimeout(resolve, 80))
    }

    ElMessage.success('种子组赛模拟完成！现在可以进入赛区对决。')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('种子组赛模拟失败:', error)
      ElMessage.error(error.message || '种子组赛模拟失败')
    }
  } finally {
    simulatingGroupStage.value = false
    simulationProgress.value = 0
  }
}

/**
 * 批量模拟赛区对决
 */
const batchSimulateRegionBattle = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的赛区对决比赛，直到决出最强赛区。是否继续?',
      '模拟赛区对决确认',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    simulatingRegionBattle.value = true
    simulationProgress.value = 0

    // 模拟半决赛（如果有）
    if (icpTournament.semifinal && icpTournament.semifinal.status !== 'completed') {
      await simulateRegionBattleInternal(icpTournament.semifinal)
    }

    // 模拟决赛
    if (icpTournament.final && icpTournament.final.status !== 'completed') {
      await simulateRegionBattleInternal(icpTournament.final)
    }

    ElMessage.success('赛区对决模拟完成！')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('赛区对决模拟失败:', error)
      ElMessage.error(error.message || '赛区对决模拟失败')
    }
  } finally {
    simulatingRegionBattle.value = false
    simulationProgress.value = 0
  }
}

/**
 * 内部模拟单场比赛
 */
const simulateMatchInternal = async (match: ICPMatch) => {
  // 生成选手数据
  const teamAId = String(match.teamAId || '')
  const teamBId = String(match.teamBId || '')
  const teamAPlayers = generateTeamPlayers(teamAId, match.teamAName || '', match.teamARegion || '')
  const teamBPlayers = generateTeamPlayers(teamBId, match.teamBName || '', match.teamBRegion || '')

  // 使用 PowerEngine 模拟比赛
  const matchDetail = PowerEngine.simulateMatch(
    teamAId,
    match.teamAName || '',
    teamAPlayers,
    teamBId,
    match.teamBName || '',
    teamBPlayers,
    match.bestOf || 3
  )

  match.scoreA = matchDetail.finalScoreA
  match.scoreB = matchDetail.finalScoreB
  match.winnerId = matchDetail.winnerId
  match.status = 'completed'
  match.completedAt = new Date()

  // 保存比赛详情
  matchDetail.matchId = match.id
  matchDetail.tournamentType = 'icp'
  matchDetail.seasonId = String(icpTournament.seasonYear)
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
        String(icpTournament.seasonYear),
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
        String(icpTournament.seasonYear),
        'INTL'
      )
    })
  })

  updateGroupStandings(match)
  checkGroupCompletion()
}

/**
 * 内部模拟赛区对决
 */
const simulateRegionBattleInternal = async (battle: ICPRegionMatch) => {
  for (const match of battle.matches) {
    if (match.status !== 'completed') {
      // 生成选手数据
      const teamAId = String(match.teamAId || '')
      const teamBId = String(match.teamBId || '')
      const teamAPlayers = generateTeamPlayers(teamAId, match.teamAName || '', match.teamARegion || '')
      const teamBPlayers = generateTeamPlayers(teamBId, match.teamBName || '', match.teamBRegion || '')

      // 使用 PowerEngine 模拟比赛 (BO5)
      const matchDetail = PowerEngine.simulateMatch(
        teamAId,
        match.teamAName || '',
        teamAPlayers,
        teamBId,
        match.teamBName || '',
        teamBPlayers,
        match.bestOf || 5
      )

      match.scoreA = matchDetail.finalScoreA
      match.scoreB = matchDetail.finalScoreB
      match.winnerId = matchDetail.winnerId
      match.status = 'completed'
      match.completedAt = new Date()

      // 保存比赛详情
      matchDetail.matchId = match.id
      matchDetail.tournamentType = 'icp'
      matchDetail.seasonId = String(icpTournament.seasonYear)
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
            String(icpTournament.seasonYear),
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
            String(icpTournament.seasonYear),
            'INTL'
          )
        })
      })

      // 更新赛区胜场
      if (match.teamARegion === battle.regionA) {
        if (matchDetail.finalScoreA > matchDetail.finalScoreB) battle.regionAWins++
        else battle.regionBWins++
      } else {
        if (matchDetail.finalScoreA > matchDetail.finalScoreB) battle.regionBWins++
        else battle.regionAWins++
      }

      await new Promise(resolve => setTimeout(resolve, 100))
    }
  }

  // 判定胜者
  if (battle.regionAWins > battle.regionBWins) {
    battle.winnerId = battle.regionA
  } else if (battle.regionBWins > battle.regionAWins) {
    battle.winnerId = battle.regionB
  } else {
    battle.winnerId = Math.random() > 0.5 ? battle.regionA : battle.regionB
  }

  battle.status = 'completed'
  checkTournamentCompletion()
}

/**
 * 显示冠军庆祝动画
 */
const showChampionCelebration = (championName: string) => {
  ElMessageBox.alert(
    `恭喜 ${championName} 成为ICP洲际对抗赛最强赛区！`,
    '🏆 最强赛区诞生! 🏆',
    {
      confirmButtonText: '太棒了!',
      customClass: 'champion-celebration-box',
      showClose: false,
      center: true
    }
  )
}

/**
 * 加载ICP赛事数据
 */
const loadICPData = async () => {
  loading.value = true

  try {
    // 获取当前存档和赛季
    const currentSave = gameStore.currentSave
    if (!currentSave) {
      console.warn('未找到当前存档')
      return
    }

    const seasonId = currentSave.currentSeason || 1

    // 获取ICP赛事ID (类型为 'Icp')
    const tournaments = await internationalApi.getTournamentsByType('Icp', seasonId)
    if (tournaments && tournaments.length > 0) {
      tournamentId.value = tournaments[0].id
    }

    if (!tournamentId.value) {
      console.warn('未找到ICP赛事')
      return
    }

    // 加载队伍映射
    const teams = await queryApi.getTeams()
    if (teams) {
      teamMap.value.clear()
      teams.forEach((team: any) => {
        teamMap.value.set(team.id, {
          name: team.name,
          regionCode: team.region_code || team.regionCode || ''
        })
      })
    }

    // 获取对阵图数据
    const bracket = await internationalApi.getTournamentBracket(tournamentId.value)
    if (bracket) {
      bracketData.value = bracket
      convertBracketToICPFormat(bracket)
    }

    // 获取小组积分榜
    const standings = await internationalApi.getGroupStandings(tournamentId.value)
    if (standings) {
      groupStandings.value = standings
      updateICPStandingsFromBackend(standings)
    }

  } catch (error) {
    console.error('加载ICP数据失败:', error)
  } finally {
    loading.value = false
  }
}

/**
 * 转换后端对阵数据到ICP格式
 */
const convertBracketToICPFormat = (bracket: BracketInfo) => {
  if (!bracket.matches) return

  // 按阶段分类比赛
  const groupMatches = bracket.matches.filter(m => m.stage === 'group' || m.stage === 'Group')
  const knockoutMatches = bracket.matches.filter(m => m.stage !== 'group' && m.stage !== 'Group')

  // 更新种子组比赛数据
  groupMatches.forEach(match => {
    // 找到对应的前端比赛
    for (const group of icpTournament.seedGroups) {
      const frontendMatch = group.matches.find(m => {
        const teamAName = teamMap.value.get(match.team_a_id || 0)?.name
        const teamBName = teamMap.value.get(match.team_b_id || 0)?.name
        return (m.teamAName === teamAName && m.teamBName === teamBName) ||
               (m.teamAName === teamBName && m.teamBName === teamAName)
      })

      if (frontendMatch && match.winner_id) {
        frontendMatch.status = 'completed'
        frontendMatch.scoreA = match.score_a || 0
        frontendMatch.scoreB = match.score_b || 0
        frontendMatch.winnerId = match.winner_id.toString()
      }
    }
  })

  // 处理淘汰赛阶段
  if (knockoutMatches.length > 0) {
    // 如果有淘汰赛比赛，说明已进入赛区对决阶段
    icpTournament.status = 'region_battle'
  }
}

/**
 * 从后端更新积分榜
 */
const updateICPStandingsFromBackend = (standings: GroupStandingInfo[]) => {
  // 按组分类
  const groupedStandings: Record<string, GroupStandingInfo[]> = {}
  standings.forEach(s => {
    const groupName = s.group_name || 'A'
    if (!groupedStandings[groupName]) {
      groupedStandings[groupName] = []
    }
    groupedStandings[groupName].push(s)
  })

  // 更新各种子组积分榜
  Object.entries(groupedStandings).forEach(([groupName, groupStandings]) => {
    const group = icpTournament.seedGroups.find(g => g.groupName === groupName)
    if (!group) return

    groupStandings.forEach(backendStanding => {
      const teamName = teamMap.value.get(backendStanding.team_id)?.name
      const frontendStanding = group.standings.find(s => s.teamName === teamName)

      if (frontendStanding) {
        frontendStanding.matchesPlayed = backendStanding.matches_played
        frontendStanding.wins = backendStanding.wins
        frontendStanding.losses = backendStanding.losses
        frontendStanding.points = backendStanding.points
        frontendStanding.roundsWon = backendStanding.rounds_won || 0
        frontendStanding.roundsLost = backendStanding.rounds_lost || 0
        frontendStanding.roundDifferential = backendStanding.round_diff || 0
        frontendStanding.position = backendStanding.position
        frontendStanding.hasBadge = backendStanding.position <= 2
      }
    })

    // 重新排序
    group.standings.sort((a, b) => {
      if (b.points !== a.points) return b.points - a.points
      if (b.roundDifferential !== a.roundDifferential) return b.roundDifferential - a.roundDifferential
      return b.roundsWon - a.roundsWon
    })

    // 检查组是否完成
    group.isComplete = group.matches.every(m => m.status === 'completed')
  })

  // 更新赛区徽章统计
  icpTournament.seedGroups.forEach(group => {
    if (group.isComplete) {
      group.standings.forEach(standing => {
        if (standing.hasBadge) {
          const region = icpTournament.regionStats.find(r => r.region === standing.region)
          if (region) {
            const team = region.teams.find(t => t.id === standing.teamId)
            if (team && team.badges === 0) {
              team.badges = 1
              region.totalBadges++
            }
          }
        }
      })
    }
  })
}

/**
 * 检查ICP赛事完成状态
 */
const checkICPCompletion = () => {
  // 检查所有种子组是否完成
  const allGroupsComplete = icpTournament.seedGroups.every(g => g.isComplete)

  if (allGroupsComplete && icpTournament.status === 'group_stage') {
    // 种子组赛已完成，可以进入赛区对决
    checkGroupCompletion()
  }
}

// 生命周期钩子
onMounted(() => {
  loadICPData()
})
</script>

<style scoped lang="scss">
.icp-management {
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

  .icp-status-card {
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

    .region-badges-section {
      margin-bottom: 24px;

      h3 {
        margin: 0 0 16px 0;
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }

      .region-badges-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 16px;

        .region-badge-card {
          padding: 20px;
          background: white;
          border: 2px solid #e5e7eb;
          border-radius: 12px;
          text-align: center;
          transition: all 0.3s ease;

          &:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          }

          &.champion {
            border-color: #f59e0b;
            background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
          }

          .region-flag {
            font-size: 32px;
            margin-bottom: 8px;

            &.lpl { background: linear-gradient(135deg, #ef4444, #dc2626); -webkit-background-clip: text; }
            &.lck { background: linear-gradient(135deg, #3b82f6, #1d4ed8); -webkit-background-clip: text; }
            &.lec { background: linear-gradient(135deg, #22c55e, #16a34a); -webkit-background-clip: text; }
            &.lcs { background: linear-gradient(135deg, #8b5cf6, #7c3aed); -webkit-background-clip: text; }
          }

          .region-name {
            font-size: 14px;
            font-weight: 600;
            color: #374151;
            margin-bottom: 12px;
          }

          .badge-count {
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;

            .badge-icon {
              font-size: 20px;
            }

            .badge-number {
              font-size: 24px;
              font-weight: 700;
              color: #f59e0b;
            }
          }

          .region-rank {
            margin-top: 8px;
            font-size: 12px;
            color: #6b7280;
          }
        }
      }
    }

    .stage-card {
      margin-bottom: 24px;

      .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
      }
    }

    .seed-groups {
      margin-top: 16px;
    }

    .generate-region-battle-section {
      margin-top: 24px;
      text-align: center;

      .el-button {
        margin-top: 16px;
      }
    }

    .region-battle-section {
      margin-top: 24px;

      .battle-stage {
        margin-bottom: 24px;
        padding: 20px;
        background: #f9fafb;
        border-radius: 12px;

        h4 {
          margin: 0 0 16px 0;
          font-size: 18px;
          font-weight: 600;
          text-align: center;
          color: #1f2937;
        }

        &.final {
          background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
          border: 2px solid #f59e0b;
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
          border-radius: 12px;
          text-align: center;
          border: 2px solid;

          .rank-badge {
            font-size: 18px;
            font-weight: 700;
            margin-bottom: 12px;
          }

          .region-flag.large {
            font-size: 48px;
            margin-bottom: 8px;
          }

          .region-name {
            font-size: 14px;
            font-weight: 600;
            color: #374151;
            margin-bottom: 12px;
          }

          .points-detail {
            font-size: 12px;
            color: #10b981;
            line-height: 1.6;
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

      .icp-completed-actions {
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
