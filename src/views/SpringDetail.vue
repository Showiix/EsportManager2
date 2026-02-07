<template>
  <div class="tournament-detail-view">
    <!-- 返回按钮和标题 -->
    <div class="page-header">
      <div class="header-left">
        <el-button text @click="goBack">
          <el-icon><ArrowLeft /></el-icon>
          返回赛事列表
        </el-button>
        <div class="title-section">
          <h1>{{ tournament.name }}</h1>
          <div class="title-tags">
            <el-tag :type="getStatusTagType(tournament.status)" size="large">
              {{ getStatusText(tournament.status) }}
            </el-tag>
            <el-tag type="info">{{ tournament.type === 'league' ? '联赛' : '国际赛' }}</el-tag>
          </div>
        </div>
      </div>
      <div class="header-actions">
        <el-button
          v-if="tournament.status === 'active'"
          type="primary"
          @click="simulateNextMatch"
          :loading="simulating"
        >
          <el-icon><VideoPlay /></el-icon>
          模拟下一场
        </el-button>
        <el-button
          v-if="tournament.status === 'active'"
          type="warning"
          @click="simulateAll"
          :loading="batchSimulating"
        >
          <el-icon><DArrowRight /></el-icon>
          一键模拟全部
        </el-button>
      </div>
    </div>

    <!-- 赛区选择器 (仅联赛显示) -->
    <el-card v-if="tournament.type === 'league'" class="region-selector-card">
      <div class="region-selector">
        <div class="selector-left">
          <span class="selector-label">选择赛区:</span>
          <el-radio-group v-model="selectedRegion" @change="handleRegionChange">
            <el-radio-button v-for="region in regions" :key="region.id" :value="region.id">
              {{ region.name }}
            </el-radio-button>
          </el-radio-group>
        </div>
        <el-button @click="refreshData" :icon="Refresh" :loading="refreshing">
          刷新数据
        </el-button>
      </div>
    </el-card>


    <!-- 常规赛内容 -->
    <div class="regular-season-content">
      <!-- 统计概览 -->
      <el-row :gutter="16" class="stats-row">
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-icon small blue">
                <el-icon :size="24"><VideoPlay /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ completedMatches }}/{{ totalMatches }}</div>
                <div class="stat-label">已完成比赛</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-icon small green">
                <el-icon :size="24"><UserFilled /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ standings.length }}</div>
                <div class="stat-label">参赛队伍</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-icon small orange">
                <el-icon :size="24"><Calendar /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">第 {{ currentWeek }} 周</div>
                <div class="stat-label">当前进度</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-icon small purple">
                <el-icon :size="24"><TrendCharts /></el-icon>
              </div>
              <div class="stat-info">
                <div class="stat-number">{{ Math.round(progress) }}%</div>
                <div class="stat-label">赛程进度</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <el-row :gutter="20">
        <!-- 左侧：积分榜 -->
        <el-col :span="10">
          <el-card class="standings-card">
            <template #header>
              <div class="card-header">
                <h3>
                  <el-icon><Medal /></el-icon>
                  积分榜
                </h3>
                <el-tag v-if="selectedRegion" type="primary">{{ getRegionName(selectedRegion) }}</el-tag>
              </div>
            </template>

            <el-table :data="standings" stripe class="standings-table">
              <el-table-column label="排名" width="70" align="center">
                <template #default="{ $index }">
                  <div class="rank-badge" :class="getRankClass($index + 1)">
                    {{ $index + 1 }}
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="战队" min-width="120">
                <template #default="{ row }">
                  <div class="team-cell">
                    <div class="team-avatar small" :class="row.region?.toLowerCase()">
                      {{ row.short }}
                    </div>
                    <span class="team-name">{{ row.short }}</span>
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="胜" width="60" align="center">
                <template #default="{ row }">
                  <span class="win-count">{{ row.wins }}</span>
                </template>
              </el-table-column>

              <el-table-column label="负" width="60" align="center">
                <template #default="{ row }">
                  <span class="loss-count">{{ row.losses }}</span>
                </template>
              </el-table-column>

              <el-table-column label="胜率" width="80" align="center">
                <template #default="{ row }">
                  <span class="win-rate">{{ getWinRate(row) }}%</span>
                </template>
              </el-table-column>

              <el-table-column label="积分" width="70" align="center">
                <template #default="{ row }">
                  <span class="points">{{ row.points }}</span>
                </template>
              </el-table-column>
            </el-table>

            <div class="playoffs-line">
              <el-divider>
                <el-tag type="success" size="small">前8名晋级季后赛</el-tag>
              </el-divider>
            </div>
          </el-card>

          <!-- MVP 排行榜 -->
          <el-card class="mvp-ranking-card">
            <template #header>
              <div class="card-header">
                <h3>
                  <el-icon><Star /></el-icon>
                  常规赛MVP排行榜
                </h3>
                <el-tag type="warning">MVP次数</el-tag>
              </div>
            </template>

            <el-table :data="mvpRanking" stripe class="mvp-table" v-loading="mvpLoading">
              <el-table-column label="排名" width="60" align="center">
                <template #default="{ $index }">
                  <div class="rank-badge" :class="getMvpRankClass($index + 1)">
                    {{ $index + 1 }}
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="选手" min-width="100">
                <template #default="{ row }">
                  <div class="player-cell">
                    <span class="player-name">{{ row.player_name }}</span>
                    <el-tag size="small" type="info">{{ row.position }}</el-tag>
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="战队" width="80" align="center">
                <template #default="{ row }">
                  <span class="team-name">{{ row.team_name }}</span>
                </template>
              </el-table-column>

              <el-table-column label="MVP次数" width="90" align="center">
                <template #default="{ row }">
                  <span class="mvp-count">{{ row.game_mvp_count }}</span>
                </template>
              </el-table-column>

              <el-table-column label="场均发挥" width="90" align="center">
                <template #default="{ row }">
                  <span class="avg-impact">{{ row.avg_impact?.toFixed(1) || '0.0' }}</span>
                </template>
              </el-table-column>
            </el-table>

            <el-empty v-if="mvpRanking.length === 0 && !mvpLoading" description="暂无MVP数据" />
          </el-card>
        </el-col>

        <!-- 右侧：比赛列表 -->
        <el-col :span="14">
          <el-card class="matches-card">
            <template #header>
              <div class="card-header">
                <h3>
                  <el-icon><List /></el-icon>
                  比赛列表
                </h3>
                <el-select v-model="matchFilter" placeholder="筛选状态" style="width: 120px;">
                  <el-option label="全部" value="all" />
                  <el-option label="已完成" value="completed" />
                  <el-option label="进行中" value="active" />
                  <el-option label="未开始" value="upcoming" />
                </el-select>
              </div>
            </template>

            <div class="matches-list">
              <div
                v-for="match in filteredMatches"
                :key="match.id"
                class="match-item"
                :class="match.status"
              >
                <div class="match-week">第{{ match.week }}周</div>
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
                    <el-tag type="success" size="small">已结束</el-tag>
                    <el-button
                      type="info"
                      size="small"
                      text
                      @click="viewMatchDetails(`spring-${match.id}`)"
                    >
                      查看详情
                    </el-button>
                  </template>
                  <template v-else>
                    <el-button
                      type="primary"
                      size="small"
                      @click="simulateSingleMatch(match)"
                      :loading="match.simulating"
                    >
                      模拟
                    </el-button>
                  </template>
                </div>
              </div>

              <el-empty v-if="filteredMatches.length === 0" description="暂无比赛数据" />
            </div>
          </el-card>
        </el-col>
      </el-row>
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
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  VideoPlay,
  DArrowRight,
  List,
  Medal,
  UserFilled,
  Calendar,
  TrendCharts,
  Star,
  Refresh,
} from '@element-plus/icons-vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, teamApi, tournamentApi, matchApi, statsApi, type Team, type PlayerTournamentStats } from '@/api/tauri'
import type { MatchDetail, GameDetail } from '@/types/matchDetail'
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
const simulating = ref(false)
const batchSimulating = ref(false)
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
        status: (springRegular.status === 'InProgress' || springRegular.status === 'Scheduled') ? 'active' :
                springRegular.status === 'Completed' ? 'completed' : 'upcoming',
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

const getStatusTagType = (status: string) => {
  switch (status) {
    case 'active': return 'success'
    case 'upcoming': return 'info'
    case 'completed': return 'primary'
    default: return 'info'
  }
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

const simulateNextMatch = async () => {
  if (!currentTournamentId.value) {
    ElMessage.error('赛事未加载')
    return
  }

  const nextMatch = matches.value.find(m => m.status === 'active' || m.status === 'upcoming')
  if (!nextMatch) {
    ElMessage.info('没有待模拟的比赛')
    return
  }

  simulating.value = true

  try {
    // 使用后端 API 模拟下一场比赛
    const result = await tournamentApi.simulateNextMatch(currentTournamentId.value)

    if (result) {
      // 重新加载比赛列表和积分榜
      await loadMatches()
      await updateStandings()

      ElMessage.success(`比赛结束: ${result.home_team_name} ${result.home_score} - ${result.away_score} ${result.away_team_name}`)
    } else {
      ElMessage.info('没有待模拟的比赛')
    }
  } catch (error) {
    logger.error('Failed to simulate next match:', error)
    ElMessage.error('模拟比赛失败')
  } finally {
    simulating.value = false
  }
}

const simulateAll = async () => {
  if (!currentTournamentId.value) {
    ElMessage.error('赛事未加载')
    return
  }

  await ElMessageBox.confirm('将自动模拟所有剩余比赛，是否继续？', '一键模拟', {
    confirmButtonText: '开始',
    cancelButtonText: '取消',
    type: 'warning'
  })

  batchSimulating.value = true

  try {
    // 使用后端 API 模拟所有比赛
    await tournamentApi.simulateAllMatches(currentTournamentId.value)

    // 重新加载比赛列表和积分榜
    await loadMatches()
    await updateStandings()

    // 开启季后赛
    ElMessage.success('常规赛模拟完成！请前往赛事管理页面进入季后赛')
  } catch (error) {
    logger.error('Failed to simulate all matches:', error)
    ElMessage.error('模拟比赛失败')
  } finally {
    batchSimulating.value = false
  }
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
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.title-section h1 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin: 0;
}

.title-tags {
  display: flex;
  gap: 8px;
}

.header-actions {
  display: flex;
  gap: 12px;
}

/* 赛区选择器 */
.region-selector-card {
  margin-bottom: 20px;
  border-radius: 12px;
}

.region-selector {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.selector-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.selector-label {
  font-weight: 600;
  color: var(--text-primary, #303133);
}

/* 阶段切换 */
.phase-card {
  margin-bottom: 20px;
  border-radius: 12px;
}

.phase-card :deep(.el-tabs__header) {
  margin: 0;
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-number {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #303133);
}

.stat-label {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
}

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary, #303133);
}

/* 积分榜 */
.standings-card {
  border-radius: 12px;
  height: fit-content;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.team-name {
  font-weight: 500;
}

.rank-badge {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 13px;
  background: #f0f2f5;
  color: #606266;
}

.rank-badge.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); color: white; }
.rank-badge.silver { background: linear-gradient(135deg, #9ca3af, #6b7280); color: white; }
.rank-badge.bronze { background: linear-gradient(135deg, #f97316, #ea580c); color: white; }
.rank-badge.playoffs { background: linear-gradient(135deg, #22c55e, #16a34a); color: white; }

.win-count { color: #22c55e; font-weight: 600; }
.loss-count { color: #ef4444; font-weight: 600; }
.win-rate { color: #3b82f6; font-weight: 600; }
.points { font-weight: 700; color: #8b5cf6; }

.playoffs-line {
  margin-top: 16px;
}

/* 比赛列表 */
.matches-card {
  border-radius: 12px;
}

.matches-list {
  max-height: 500px;
  overflow-y: auto;
}

.match-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 8px;
  background: #f5f7fa;
  transition: all 0.3s ease;
}

.match-item:hover {
  background: #ebeef5;
}

.match-item.completed {
  background: #f0fdf4;
}

.match-item.active {
  background: #fef3c7;
  border: 1px solid #f59e0b;
}

.match-week {
  width: 60px;
  font-size: 12px;
  color: var(--text-tertiary, #909399);
}

.match-teams {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}

.match-teams .team {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 100px;
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

.match-teams .team-score {
  font-weight: 700;
  font-size: 18px;
  color: var(--text-primary, #303133);
}

.match-teams .vs {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
  font-weight: 600;
}

.match-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 120px;
  justify-content: flex-end;
}

/* 季后赛对阵图 */
.bracket-card {
  border-radius: 12px;
}

.bracket-wrapper {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding: 20px 0;
}

.bracket-section {
  background: #f5f7fa;
  border-radius: 12px;
  padding: 20px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 20px 0;
  color: var(--text-primary, #303133);
}

.winners-title { color: #22c55e; }
.losers-title { color: #f59e0b; }
.finals-title { color: #8b5cf6; }

/* 对阵图横向布局 */
.winners-bracket,
.losers-bracket {
  display: flex;
  align-items: flex-start;
  gap: 60px;
  overflow-x: auto;
  padding: 10px 0;
}

.bracket-column {
  display: flex;
  flex-direction: column;
  min-width: 180px;
}

.round-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-tertiary, #909399);
  margin-bottom: 16px;
  text-align: center;
  padding: 4px 12px;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

.matches-column {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.matches-column.centered {
  justify-content: center;
  min-height: 100%;
}

/* 比赛卡片 */
.bracket-match {
  position: relative;
  background: white;
  border-radius: 8px;
  padding: 8px 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  border-left: 3px solid #e5e7eb;
  min-width: 160px;
}

.bracket-match.winners {
  border-left-color: #22c55e;
}

.bracket-match.losers {
  border-left-color: #f59e0b;
}

.bracket-match.completed {
  border-left-color: #3b82f6;
}

.bracket-match.final {
  border-left-width: 4px;
}

/* 队伍显示 */
.match-team {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  transition: background 0.2s;
}

.match-team.winner {
  background: #f0fdf4;
}

.match-team .seed {
  font-size: 11px;
  color: var(--text-tertiary, #909399);
  min-width: 20px;
}

.match-team .name {
  flex: 1;
  font-weight: 500;
  font-size: 14px;
}

.match-team .score {
  font-weight: 700;
  color: var(--text-primary, #303133);
  font-size: 14px;
}

/* 连接线 */
.connector-right {
  position: absolute;
  right: -30px;
  top: 50%;
  width: 30px;
  height: 2px;
  background: #d1d5db;
}

.connector-right::after {
  content: '';
  position: absolute;
  right: 0;
  top: -4px;
  border: 5px solid transparent;
  border-left-color: #d1d5db;
}

.connector-left {
  position: absolute;
  left: -30px;
  top: 50%;
  width: 30px;
  height: 2px;
  background: #d1d5db;
}

/* 胜者组特殊布局 - 让决赛居中 */
.winners-bracket .bracket-column:last-child .matches-column {
  margin-top: 50px;
}

/* 败者组特殊布局 - 逐渐居中 */
.losers-bracket .bracket-column:nth-child(3) .matches-column,
.losers-bracket .bracket-column:nth-child(4) .matches-column {
  margin-top: 50px;
}

/* 总决赛区域 */
.finals-section {
  background: linear-gradient(135deg, #fef3c7, #fde68a);
  border: 2px solid #f59e0b;
}

.grand-final {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}

.final-match-card {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 40px;
  padding: 24px 40px;
  background: white;
  border-radius: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.final-match-card.completed {
  border: 2px solid #22c55e;
}

.final-team {
  text-align: center;
  padding: 16px 24px;
  background: #f9fafb;
  border-radius: 12px;
  min-width: 140px;
  transition: all 0.3s;
}

.final-team.champion {
  background: linear-gradient(135deg, #fbbf24, #f59e0b);
  color: white;
  transform: scale(1.05);
  box-shadow: 0 4px 15px rgba(245, 158, 11, 0.4);
}

.final-team .team-source {
  font-size: 11px;
  color: #909399;
  margin-bottom: 4px;
}

.final-team.champion .team-source {
  color: rgba(255, 255, 255, 0.8);
}

.final-team .team-name {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 8px;
}

.final-team .team-score {
  font-size: 36px;
  font-weight: 900;
}

.vs-badge {
  font-size: 24px;
  font-weight: 900;
  color: #f59e0b;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* 冠军展示 */
.champion-display {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 20px 40px;
  background: linear-gradient(135deg, #fbbf24, #f59e0b);
  border-radius: 12px;
  box-shadow: 0 4px 15px rgba(245, 158, 11, 0.3);
}

.champion-trophy {
  font-size: 48px;
  animation: trophy-bounce 1s ease infinite;
}

@keyframes trophy-bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

.champion-info {
  text-align: left;
}

.champion-label {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
}

.champion-name {
  font-size: 28px;
  font-weight: 900;
  color: white;
}

/* MVP 排行榜 */
.mvp-ranking-card {
  border-radius: 12px;
  margin-top: 20px;
}

.mvp-table {
  width: 100%;
}

.player-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.player-name {
  font-weight: 500;
  color: var(--text-primary, #303133);
}

.mvp-count {
  font-weight: 700;
  font-size: 16px;
  color: #f59e0b;
}

.avg-impact {
  font-weight: 600;
  color: #3b82f6;
}

.rank-badge.mvp-gold {
  background: linear-gradient(135deg, #fbbf24, #f59e0b);
  color: white;
}

.rank-badge.mvp-silver {
  background: linear-gradient(135deg, #9ca3af, #6b7280);
  color: white;
}

.rank-badge.mvp-bronze {
  background: linear-gradient(135deg, #f97316, #ea580c);
  color: white;
}

/* 响应式 */
@media (max-width: 1200px) {
  .winners-bracket,
  .losers-bracket {
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .connector-right,
  .connector-left {
    display: none;
  }

  .winners-bracket .bracket-column:last-child .matches-column,
  .losers-bracket .bracket-column:nth-child(3) .matches-column,
  .losers-bracket .bracket-column:nth-child(4) .matches-column {
    margin-top: 0;
  }
}

@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    gap: 16px;
  }

  .title-section {
    flex-direction: column;
    align-items: flex-start;
  }

  .match-teams {
    flex-direction: column;
    gap: 8px;
  }

  .match-teams .team {
    justify-content: center !important;
  }

  .final-match-card {
    flex-direction: column;
    gap: 16px;
    padding: 20px;
  }
}
</style>
