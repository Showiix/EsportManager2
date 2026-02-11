<template>
  <div class="tournament-detail-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>{{ tournament.name }}</h1>
        <p>{{ getStatusText(tournament.status) }}</p>
      </div>
      <div class="header-actions">
        <el-button
          v-if="tournament.status === 'active'"
          size="small"
          @click="simulateAll"
          :loading="batchSimulating"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ batchSimulating ? `模拟中 (${simulationProgress}%)` : '一键模拟' }}
        </el-button>
        <button class="back-btn" @click="goBack">← 返回赛事列表</button>
      </div>
    </div>

    <!-- 模拟进度条 -->
    <el-progress
      v-if="batchSimulating"
      :percentage="simulationProgress"
      :stroke-width="6"
      :show-text="false"
      style="margin-bottom: 12px;"
    />

    <!-- 赛区选择器 -->
    <div v-if="tournament.type === 'league'" class="filter-section">
      <div class="filter-row">
        <div class="filter-group">
          <label>赛区</label>
          <el-radio-group v-model="selectedRegion" @change="handleRegionChange" size="small">
            <el-radio-button v-for="region in regions" :key="region.id" :value="region.id">
              {{ region.name }}
            </el-radio-button>
          </el-radio-group>
        </div>
        <el-tooltip content="刷新数据" placement="bottom">
          <el-button circle size="small" @click="refreshData" :loading="refreshing">
            <el-icon><Refresh /></el-icon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <!-- 统计栏 -->
    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ completedMatches }}/{{ totalMatches }}</span>
        <span class="stat-label">已完成比赛</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ standings.length }}</span>
        <span class="stat-label">参赛队伍</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">第{{ currentWeek }}周</span>
        <span class="stat-label">当前进度</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value highlight">{{ Math.round(progress) }}%</span>
        <span class="stat-label">赛程进度</span>
      </div>
    </div>

    <!-- 常规赛内容 -->
    <div class="content-layout">
      <!-- 左侧：积分榜 + MVP -->
      <div class="left-panel">
        <!-- 积分榜 -->
        <div class="table-section">
          <div class="section-header">
            <span class="section-title">积分榜</span>
            <span class="section-tag">{{ getRegionName(selectedRegion) }}</span>
          </div>
          <div class="standings-list">
            <div class="standings-head">
              <span class="s-rank">#</span>
              <span class="s-team">战队</span>
              <span class="s-num">胜</span>
              <span class="s-num">负</span>
              <span class="s-num">胜率</span>
              <span class="s-num">积分</span>
            </div>
            <div
              v-for="(team, idx) in standings"
              :key="team.id"
              class="standings-row"
            >
              <span class="s-rank">
                <span class="rank-num" :class="getRankClass(idx + 1)">{{ idx + 1 }}</span>
              </span>
              <span class="s-team">{{ team.short }}</span>
              <span class="s-num win">{{ team.wins }}</span>
              <span class="s-num loss">{{ team.losses }}</span>
              <span class="s-num rate">{{ getWinRate(team) }}%</span>
              <span class="s-num pts">{{ team.points }}</span>
            </div>
          </div>
          <div class="playoff-line">前8名晋级季后赛</div>
        </div>

        <!-- MVP 排行 -->
        <div class="table-section" v-if="mvpRanking.length > 0 || mvpLoading">
          <div class="section-header">
            <span class="section-title">MVP 排行</span>
            <span class="mvp-hint">次数 / 场均</span>
          </div>
          <div v-if="mvpLoading" style="padding: 16px;">
            <el-skeleton :rows="3" animated />
          </div>
          <div v-else class="mvp-list">
            <div v-for="(player, idx) in mvpRanking" :key="player.player_id" class="mvp-row">
              <span class="mvp-rank" :class="getMvpRankClass(idx + 1)">{{ idx + 1 }}</span>
              <div class="mvp-info">
                <span class="mvp-name">{{ player.player_name }}</span>
                <span class="mvp-meta">{{ teamMap.get(player.team_id)?.short_name || player.team_name }} · {{ player.position }}</span>
              </div>
              <span class="mvp-count">{{ player.game_mvp_count }}</span>
              <span class="mvp-impact">{{ player.avg_impact?.toFixed(1) || '0.0' }}</span>
            </div>
          </div>
          <el-empty v-if="mvpRanking.length === 0 && !mvpLoading" description="暂无MVP数据" :image-size="40" />
        </div>
      </div>

      <!-- 右侧：比赛列表 -->
      <div class="right-panel">
        <div class="table-section">
          <div class="section-header">
            <span class="section-title">比赛列表</span>
            <el-select v-model="matchFilter" size="small" style="width: 100px;">
              <el-option label="全部" value="all" />
              <el-option label="已完成" value="completed" />
              <el-option label="进行中" value="active" />
              <el-option label="未开始" value="upcoming" />
            </el-select>
          </div>
          <div class="matches-scroll">
            <div v-for="group in groupedMatches" :key="group.week" class="match-week-group">
              <div class="week-header">
                <span class="week-label">第{{ group.week }}周</span>
                <span class="week-count">{{ group.matches.length }}场</span>
              </div>
              <div
                v-for="match in group.matches"
                :key="match.id"
                class="match-row"
                :class="match.status"
              >
                <div class="match-teams">
                  <div class="team home" :class="{ winner: match.winnerId === match.homeTeamId }">
                    <span class="team-name">{{ match.homeTeam }}</span>
                    <span class="team-score" v-if="match.status === 'completed'">{{ match.homeScore }}</span>
                  </div>
                  <div class="vs">VS</div>
                  <div class="team away" :class="{ winner: match.winnerId === match.awayTeamId }">
                    <span class="team-score" v-if="match.status === 'completed'">{{ match.awayScore }}</span>
                    <span class="team-name">{{ match.awayTeam }}</span>
                  </div>
                </div>
                <div class="match-actions">
                  <template v-if="match.status === 'completed'">
                    <el-button size="small" type="primary" text @click="viewMatchDetails(`spring-${match.id}`)">
                      详情
                    </el-button>
                  </template>
                  <template v-else>
                    <el-button type="primary" size="small" @click="simulateSingleMatch(match)" :loading="match.simulating">
                      模拟
                    </el-button>
                  </template>
                </div>
              </div>
            </div>
            <el-empty v-if="filteredMatches.length === 0" description="暂无比赛数据" :image-size="40" />
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
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  VideoPlay,
  DArrowRight,
  Refresh,
} from '@element-plus/icons-vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, teamApi, tournamentApi, matchApi, statsApi, type Team, type PlayerTournamentStats } from '@/api/tauri'
import type { MatchDetail, GameDetail } from '@/types/matchDetail'
import { useBatchSimulation } from '@/composables/useBatchSimulation'
import { createLogger } from '@/utils/logger'

const logger = createLogger('SpringDetail')

const route = useRoute()
const router = useRouter()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()

// 从 query 获取赛季（赛事管理页传入），否则使用当前赛季
const viewingSeason = computed(() => Number(route.query.season) || gameStore.gameState?.current_season || 1)

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 状态
const selectedRegion = ref(1) // 默认 LPL region_id = 1
const matchFilter = ref('all')

// 批量模拟 composable
const { simulationProgress, isSimulating: batchSimulating, batchSimulate } = useBatchSimulation()
const loading = ref(false)
const refreshing = ref(false)

// 当前赛事ID
const currentTournamentId = ref<number | null>(null)

// 赛事信息 (从后端加载)
const tournament = ref({
  id: route.params.id,
  name: '春季常规赛',
  type: 'league',
  status: 'active',
  description: '四大赛区春季常规赛',
})

// 赛区数据 (从后端加载)
const regions = ref<{ id: number; name: string }[]>([])

// 队伍ID到名称的映射 (从后端加载)
const teamMap = ref<Map<number, Team>>(new Map())

// 当前显示的积分榜数据 (从后端加载)
const standings = ref<any[]>([])

// 当前显示的比赛数据 (从后端加载)
const matches = ref<any[]>([])

// MVP 排行榜数据
const mvpRanking = ref<PlayerTournamentStats[]>([])
const mvpLoading = ref(false)

// 加载赛区列表
const loadRegions = async () => {
  try {
    const regionList = await queryApi.getAllRegions()
    regions.value = regionList.map(r => ({ id: r.id, name: r.name }))
    if (regionList.length > 0) {
      selectedRegion.value = regionList[0].id
    }
  } catch (error) {
    logger.error('Failed to load regions:', error)
    ElMessage.error('加载赛区数据失败')
  }
}

// 加载队伍数据
const loadTeams = async (regionId: number) => {
  try {
    const teams = await teamApi.getTeamsByRegion(regionId)
    teamMap.value.clear()
    teams.forEach(team => teamMap.value.set(team.id, team))
  } catch (error) {
    logger.error('Failed to load teams:', error)
  }
}

// 加载当前赛区的春季赛赛事
const loadTournament = async (regionId: number) => {
  try {
    const seasonId = viewingSeason.value
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    // 查找春季常规赛 (后端存储格式为 PascalCase: SpringRegular)
    const springRegular = tournaments.find(t => t.tournament_type === 'SpringRegular')
    if (springRegular) {
      currentTournamentId.value = springRegular.id
      tournament.value = {
        id: springRegular.id.toString(),
        name: springRegular.name,
        type: 'league',
        status: springRegular.status === 'Completed' ? 'completed' : 'active',
        description: '春季常规赛与季后赛',
      }
    }
  } catch (error) {
    logger.error('Failed to load tournament:', error)
  }
}

// 加载比赛列表
const loadMatches = async () => {
  if (!currentTournamentId.value) return
  try {
    const matchList = await tournamentApi.getTournamentMatches(currentTournamentId.value)
    matches.value = matchList.map(m => ({
      id: m.id,
      week: m.round || 1,
      homeTeamId: m.home_team_id,
      homeTeam: m.home_team_name || teamMap.value.get(m.home_team_id || 0)?.short_name || '未知',
      awayTeamId: m.away_team_id,
      awayTeam: m.away_team_name || teamMap.value.get(m.away_team_id || 0)?.short_name || '未知',
      homeScore: m.home_score,
      awayScore: m.away_score,
      winnerId: m.winner_id,
      // 后端状态格式为 PascalCase: Completed, InProgress, Scheduled
      status: (m.status === 'Completed' || m.status === 'COMPLETED') ? 'completed' : m.status === 'InProgress' ? 'active' : 'upcoming',
      simulating: false,
    }))
  } catch (error) {
    logger.error('Failed to load matches:', error)
  }
}

// 加载积分榜
const loadStandings = async () => {
  if (!currentTournamentId.value) return
  try {
    const standingList = await tournamentApi.getStandings(currentTournamentId.value)
    standings.value = standingList.map(s => {
      const team = teamMap.value.get(s.team_id)
      return {
        id: s.team_id,
        name: team?.name || s.team_name,
        short: team?.short_name || s.team_name,
        region: regions.value.find(r => r.id === selectedRegion.value)?.name || 'LPL',
        wins: s.wins,
        losses: s.losses,
        points: s.points,
      }
    })
  } catch (error) {
    logger.error('Failed to load standings:', error)
  }
}

// 加载 MVP 排行榜
const loadMvpRanking = async () => {
  if (!currentTournamentId.value) return
  mvpLoading.value = true
  try {
    const ranking = await statsApi.getTournamentMvpRanking(currentTournamentId.value, 10)
    mvpRanking.value = ranking
    logger.debug('[SpringDetail] Loaded MVP ranking:', ranking.length, 'players')
  } catch (error) {
    logger.error('Failed to load MVP ranking:', error)
    mvpRanking.value = []
  } finally {
    mvpLoading.value = false
  }
}

// 加载赛区所有数据
const loadRegionData = async (regionId: number) => {
  loading.value = true
  try {
    await loadTeams(regionId)
    await loadTournament(regionId)
    await loadMatches()
    await loadStandings()
    await loadMvpRanking()
  } finally {
    loading.value = false
  }
}

// 刷新数据
const refreshData = async () => {
  refreshing.value = true
  try {
    await loadRegionData(selectedRegion.value)
    ElMessage.success('数据刷新成功')
  } catch (error) {
    logger.error('刷新数据失败:', error)
    ElMessage.error('刷新数据失败')
  } finally {
    refreshing.value = false
  }
}

// 计算属性
const totalMatches = computed(() => matches.value.length)
const completedMatches = computed(() => matches.value.filter(m => m.status === 'completed').length)
const currentWeek = computed(() => {
  const activeMatch = matches.value.find(m => m.status === 'active')
  return activeMatch?.week || 1
})
const progress = computed(() => (completedMatches.value / totalMatches.value) * 100)

const filteredMatches = computed(() => {
  if (matchFilter.value === 'all') return matches.value
  return matches.value.filter(m => m.status === matchFilter.value)
})

// 按周分组
const groupedMatches = computed(() => {
  const groups: { week: number; matches: typeof filteredMatches.value }[] = []
  let currentWeek = -1
  for (const match of filteredMatches.value) {
    if (match.week !== currentWeek) {
      currentWeek = match.week
      groups.push({ week: currentWeek, matches: [] })
    }
    groups[groups.length - 1].matches.push(match)
  }
  return groups
})

// 方法
const goBack = () => {
  router.push('/tournaments')
}

/**
 * 查看比赛详情
 */
const viewMatchDetails = async (matchId: string) => {
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
  ElMessage.info('该比赛暂无详细数据')
}

/**
 * 关闭比赛详情弹窗
 */
const handleCloseMatchDetail = () => {
  showMatchDetailDialog.value = false
  currentMatchDetail.value = null
}

const handleRegionChange = async (regionId: number) => {
  // 加载新赛区数据
  await loadRegionData(regionId)

  const regionName = regions.value.find(r => r.id === regionId)?.name || '未知'
  ElMessage.success(`已切换到 ${regionName} 赛区`)
}

const getRegionName = (regionId: number) => {
  return regions.value.find(r => r.id === regionId)?.name || 'LPL'
}

const getWinRate = (team: any) => {
  const total = team.wins + team.losses
  if (total === 0) return 0
  return Math.round((team.wins / total) * 100)
}


const getStatusText = (status: string) => {
  switch (status) {
    case 'active': return '进行中'
    case 'upcoming': return '未开始'
    case 'completed': return '已完成'
    default: return '未知'
  }
}

const getRankClass = (rank: number) => {
  if (rank === 1) return 'gold'
  if (rank === 2) return 'silver'
  if (rank === 3) return 'bronze'
  if (rank <= 4) return 'playoffs'
  return ''
}

const getMvpRankClass = (rank: number) => {
  if (rank === 1) return 'mvp-gold'
  if (rank === 2) return 'mvp-silver'
  if (rank === 3) return 'mvp-bronze'
  return ''
}

/**
 * 模拟单场比赛（点击比赛列表中的模拟按钮）
 */
const simulateSingleMatch = async (match: any) => {
  match.simulating = true

  try {
    // 使用后端 API 模拟比赛
    const result = await matchApi.simulateMatchDetailed(match.id)

    // 更新本地比赛数据
    match.homeScore = result.home_score
    match.awayScore = result.away_score
    match.winnerId = result.winner_id
    match.status = 'completed'

    // 转换后端结果为 MatchDetail 格式并保存到 store
    const matchDetail = convertToMatchDetail(result, match)
    logger.debug(`[SpringDetail] matchDetail.games.length = ${matchDetail.games.length}`)
    matchDetail.games.forEach((g, idx) => {
      logger.debug(`[SpringDetail] game[${idx}]: teamAPlayers=${g.teamAPlayers.length}, teamBPlayers=${g.teamBPlayers.length}`)
    })
    await matchDetailStore.saveMatchDetail(`spring-${match.id}`, matchDetail)

    // 记录选手表现到统计
    const regionName = getRegionName(selectedRegion.value)
    logger.debug(`[SpringDetail] 开始记录选手表现, regionName=${regionName}`)
    matchDetail.games.forEach(game => {
      game.teamAPlayers.forEach(perf => {
        playerStore.recordPerformance(
          perf.playerId,
          perf.playerName,
          perf.teamId,
          perf.position,
          perf.impactScore,
          perf.actualAbility,
          String(gameStore.currentSeason),
          regionName
        )
      })
      game.teamBPlayers.forEach(perf => {
        playerStore.recordPerformance(
          perf.playerId,
          perf.playerName,
          perf.teamId,
          perf.position,
          perf.impactScore,
          perf.actualAbility,
          String(gameStore.currentSeason),
          regionName
        )
      })
    })
    playerStore.saveToStorage()

    // 重新加载比赛列表和积分榜
    await loadMatches()
    await updateStandings()

    // 检查常规赛是否全部完成
    const allCompleted = matches.value.every(m => m.status === 'completed')
    if (allCompleted) {
      ElMessage.success('常规赛全部完成！请前往赛事管理页面进入季后赛')
    } else {
      ElMessage.success(`比赛结束: ${match.homeTeam} ${result.home_score} - ${result.away_score} ${match.awayTeam}`)
    }
  } catch (error) {
    logger.error('Failed to simulate match:', error)
    ElMessage.error('模拟比赛失败')
  } finally {
    match.simulating = false
  }
}

/**
 * 将后端 DetailedMatchResult 转换为前端 MatchDetail 格式
 */
const convertToMatchDetail = (result: any, match: any): MatchDetail => {
  // 处理位置格式（后端可能返回 "Some(Adc)" 格式）
  const parsePosition = (pos: string | null | undefined): string => {
    if (!pos) return 'MID'
    // 处理 "Some(Adc)" 格式
    const someMatch = pos.match(/Some\((\w+)\)/)
    if (someMatch) {
      return someMatch[1]
    }
    return pos
  }

  // 将位置转换为标准格式
  const normalizePosition = (pos: string): string => {
    const posMap: Record<string, string> = {
      'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP',
      'top': 'TOP', 'jungle': 'JUG', 'mid': 'MID', 'adc': 'ADC', 'support': 'SUP',
      'Jug': 'JUG', 'Sup': 'SUP',  // 后端 Rust 枚举格式
    }
    return posMap[pos] || pos
  }

  const games: GameDetail[] = result.games.map((g: any) => {
    // 计算队伍平均发挥值（用于显示）
    const calcTeamAvgPerformance = (players: any[]) => {
      if (!players || players.length === 0) return 0
      const sum = players.reduce((acc: number, p: any) => acc + (p.actual_ability || 0), 0)
      return sum / players.length
    }

    const homeAvgPerf = calcTeamAvgPerformance(g.home_players)
    const awayAvgPerf = calcTeamAvgPerformance(g.away_players)

    return {
      gameNumber: g.game_number,
      teamAId: String(result.home_team_id),
      teamAName: result.home_team_name || match.homeTeam,
      teamAPower: homeAvgPerf,
      teamAPerformance: homeAvgPerf,
      teamAMetaPower: g.home_performance,
      teamAPlayers: (g.home_players || []).map((p: any) => {
        // 直接使用后端返回的真实数据
        return {
          playerId: String(p.player_id),
          playerName: p.player_name,
          teamId: String(result.home_team_id),
          position: normalizePosition(parsePosition(p.position)),
          baseAbility: p.base_ability || 70,
          actualAbility: p.actual_ability || 70,
          conditionBonus: p.condition_bonus || 0,
          stabilityNoise: p.stability_noise || 0,
          impactScore: p.impact_score || 0,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        }
      }),
      teamBId: String(result.away_team_id),
      teamBName: result.away_team_name || match.awayTeam,
      teamBPower: awayAvgPerf,
      teamBPerformance: awayAvgPerf,
      teamBMetaPower: g.away_performance,
      teamBPlayers: (g.away_players || []).map((p: any) => {
        // 直接使用后端返回的真实数据
        return {
          playerId: String(p.player_id),
          playerName: p.player_name,
          teamId: String(result.away_team_id),
          position: normalizePosition(parsePosition(p.position)),
          baseAbility: p.base_ability || 70,
          actualAbility: p.actual_ability || 70,
          conditionBonus: p.condition_bonus || 0,
          stabilityNoise: p.stability_noise || 0,
          impactScore: p.impact_score || 0,
          traits: p.traits,
          activatedTraits: p.activated_traits?.map((t: any) => ({
            type: t.trait_type,
            name: t.name,
            effect: t.effect,
            value: t.value,
            isPositive: t.is_positive
          }))
        }
      }),
      winnerId: String(g.winner_id),
      winnerName: g.winner_id === result.home_team_id ? (result.home_team_name || match.homeTeam) : (result.away_team_name || match.awayTeam),
      powerDifference: homeAvgPerf - awayAvgPerf,
      performanceDifference: homeAvgPerf - awayAvgPerf,
      metaPowerDifference: g.home_performance - g.away_performance,
      isUpset: false,
    }
  })

  return {
    matchId: `spring-${match.id}`,
    seasonId: String(viewingSeason.value),
    tournamentType: 'spring',
    teamAId: String(result.home_team_id),
    teamAName: result.home_team_name || match.homeTeam,
    teamBId: String(result.away_team_id),
    teamBName: result.away_team_name || match.awayTeam,
    bestOf: 3,
    games,
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    winnerName: result.winner_id === result.home_team_id ? (result.home_team_name || match.homeTeam) : (result.away_team_name || match.awayTeam),
    mvpPlayerId: result.match_mvp?.player_id ? String(result.match_mvp.player_id) : undefined,
    mvpPlayerName: result.match_mvp?.player_name,
    mvpTeamId: result.match_mvp?.team_id ? String(result.match_mvp.team_id) : undefined,
    mvpTotalImpact: result.match_mvp?.mvp_score,
    createdAt: new Date().toISOString(),
  }
}

const simulateAll = async () => {
  if (!currentTournamentId.value) {
    ElMessage.error('赛事未加载')
    return
  }

  // 收集未完成的比赛
  const pendingMatches = matches.value.filter(m => m.status !== 'completed')
  if (pendingMatches.length === 0) {
    ElMessage.info('没有待模拟的比赛')
    return
  }

  await batchSimulate({
    confirmMessage: `将自动模拟剩余 ${pendingMatches.length} 场比赛，是否继续？`,
    confirmTitle: '一键模拟',
    confirmType: 'warning',
    successMessage: '常规赛模拟完成！请前往赛事管理页面进入季后赛',
    errorPrefix: '模拟比赛失败',
    tournamentType: 'spring',
    seasonId: String(viewingSeason.value),
    competitionType: 'LEAGUE',
    delayMs: 10,
    tournamentId: currentTournamentId.value ?? undefined,
    matches: pendingMatches.map(m => ({
      matchId: m.id,
      teamAId: String(m.homeTeamId),
      teamAName: m.homeTeam,
      teamBId: String(m.awayTeamId),
      teamBName: m.awayTeam,
      bestOf: 3,
      backendMatchId: m.id,
      frontendMatchId: `spring-${m.id}`,
    })),
    onComplete: async () => {
      await loadMatches()
      await updateStandings()
    }
  })
}

const updateStandings = async () => {
  // 从后端重新加载积分榜
  await loadStandings()
}

onMounted(async () => {
  // 从 localStorage 加载比赛详情数据
  matchDetailStore.loadFromStorage()

  // 加载赛区列表
  await loadRegions()
  // 加载默认赛区数据
  if (selectedRegion.value) {
    await loadRegionData(selectedRegion.value)
  }
})
</script>

<style scoped>
.tournament-detail-view {
  padding: 0;
}

/* 页面头部 */
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
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
  align-items: center;
  gap: 8px;
}

.back-btn {
  padding: 5px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #ffffff;
  color: #475569;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.back-btn:hover {
  border-color: #6366f1;
  color: #6366f1;
  background: #f5f3ff;
}

/* 筛选栏 */
.filter-section {
  margin-bottom: 16px;
}

.filter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-group label {
  font-size: 13px;
  color: #64748b;
  font-weight: 500;
}

/* 统计栏 */
.stats-bar {
  display: flex;
  align-items: center;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 14px 24px;
  margin-bottom: 16px;
  gap: 24px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.stat-value.highlight {
  color: #6366f1;
}

.stat-label {
  font-size: 11px;
  color: #94a3b8;
}

.stat-divider {
  width: 1px;
  height: 28px;
  background: #e2e8f0;
}

/* 内容布局 */
.content-layout {
  display: flex;
  gap: 16px;
}

.left-panel {
  width: 380px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.right-panel {
  flex: 1;
  min-width: 0;
}

/* 通用区块 */
.table-section {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #f1f5f9;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
}

.section-tag {
  font-size: 11px;
  color: #6366f1;
  font-weight: 500;
  padding: 2px 8px;
  background: #f5f3ff;
  border-radius: 4px;
}

.mvp-hint {
  font-size: 11px;
  color: #94a3b8;
  font-weight: 400;
}

/* 积分榜 */
.standings-head {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background: #f8fafc;
  font-size: 11px;
  color: #94a3b8;
  font-weight: 600;
}

.standings-row {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  border-bottom: 1px solid #f8fafc;
  transition: background 0.15s;
  font-size: 13px;
}

.standings-row:last-child {
  border-bottom: none;
}

.standings-row:hover {
  background: #f8fafc;
}

.s-rank { width: 40px; text-align: center; }
.s-team { flex: 1; font-weight: 500; color: #0f172a; }
.s-num { width: 50px; text-align: center; color: #64748b; }
.s-num.win { color: #22c55e; font-weight: 600; }
.s-num.loss { color: #ef4444; font-weight: 600; }
.s-num.rate { color: #3b82f6; font-weight: 500; }
.s-num.pts { color: #6366f1; font-weight: 700; }

.rank-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  font-size: 11px;
  font-weight: 700;
  background: #f1f5f9;
  color: #64748b;
}

.rank-num.gold { background: #fef3c7; color: #d97706; }
.rank-num.silver { background: #f1f5f9; color: #64748b; }
.rank-num.bronze { background: #ffedd5; color: #ea580c; }
.rank-num.playoffs { background: #dcfce7; color: #16a34a; }

.playoff-line {
  padding: 8px 16px;
  font-size: 11px;
  color: #94a3b8;
  text-align: center;
  border-top: 1px dashed #e2e8f0;
}

/* MVP 排行 */
.mvp-row {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  border-bottom: 1px solid #f8fafc;
  gap: 10px;
  font-size: 13px;
}

.mvp-row:last-child {
  border-bottom: none;
}

.mvp-rank {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  background: #f1f5f9;
  color: #64748b;
  flex-shrink: 0;
}

.mvp-rank.mvp-gold { background: #fef3c7; color: #d97706; }
.mvp-rank.mvp-silver { background: #f1f5f9; color: #64748b; }
.mvp-rank.mvp-bronze { background: #ffedd5; color: #ea580c; }

.mvp-info {
  flex: 1;
  min-width: 0;
}

.mvp-name {
  font-weight: 600;
  color: #0f172a;
}

.mvp-meta {
  font-size: 11px;
  color: #94a3b8;
  display: block;
}

.mvp-count {
  font-weight: 700;
  color: #f59e0b;
  font-size: 14px;
  min-width: 24px;
  text-align: center;
}

.mvp-impact {
  font-weight: 500;
  color: #64748b;
  font-size: 12px;
  min-width: 32px;
  text-align: right;
}

/* 比赛列表 */
.matches-scroll {
  max-height: 700px;
  overflow-y: auto;
}

.week-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  background: #f8fafc;
  position: sticky;
  top: 0;
  z-index: 1;
}

.week-label {
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
}

.week-count {
  font-size: 11px;
  color: #94a3b8;
}

.match-row {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid #f8fafc;
  transition: background 0.15s;
}

.match-row:hover {
  background: #f8fafc;
}

.match-row.active {
  background: #fffbeb;
  border-left: 2px solid #f59e0b;
}

.match-teams {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.match-teams .team {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 90px;
}

.match-teams .team.home {
  justify-content: flex-end;
}

.match-teams .team.away {
  justify-content: flex-start;
}

.match-teams .team.winner .team-name {
  font-weight: 700;
  color: #22c55e;
}

.match-teams .team-name {
  font-size: 13px;
  font-weight: 500;
  color: #0f172a;
}

.match-teams .team-score {
  font-weight: 700;
  font-size: 16px;
  color: #0f172a;
}

.match-teams .vs {
  font-size: 11px;
  color: #cbd5e1;
  font-weight: 600;
}

.match-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 70px;
  justify-content: flex-end;
}
</style>
