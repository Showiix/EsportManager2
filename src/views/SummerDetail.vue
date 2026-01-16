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
              <div class="stat-icon blue">
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
              <div class="stat-icon green">
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
              <div class="stat-icon orange">
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
              <div class="stat-icon purple">
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
                    <div class="team-avatar" :class="row.region?.toLowerCase()">
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
                      @click="viewMatchDetails(`summer-${match.id}`)"
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
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
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
import { useTimeStore } from '@/stores/useTimeStore'
import { queryApi, teamApi, tournamentApi, matchApi, statsApi, type Team, type PlayerTournamentStats } from '@/api/tauri'
import type { MatchDetail, GameDetail } from '@/types/matchDetail'

const route = useRoute()
const router = useRouter()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()
const timeStore = useTimeStore()
const { lastMessage: timeLastMessage, timeState } = storeToRefs(timeStore)

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
  name: '夏季赛',
  type: 'league',
  status: 'active',
  description: '四大赛区夏季常规赛与季后赛',
})

// 赛区数据 (从后端加载)
const regions = ref<{ id: number; name: string }[]>([])

// 队伍ID到名称的映射 (从后端加载)
const teamMap = ref<Map<number, Team>>(new Map())

// 当前显示的积分榜数据 (从后端加载)
const standings = ref<any[]>([])

// MVP 排行榜数据
const mvpRanking = ref<PlayerTournamentStats[]>([])
const mvpLoading = ref(false)

// 当前显示的比赛数据 (从后端加载)
const matches = ref<any[]>([])

// 加载赛区列表
const loadRegions = async () => {
  try {
    const regionList = await queryApi.getAllRegions()
    regions.value = regionList.map(r => ({ id: r.id, name: r.name }))
    if (regionList.length > 0) {
      selectedRegion.value = regionList[0].id
    }
  } catch (error) {
    console.error('Failed to load regions:', error)
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
    console.error('Failed to load teams:', error)
  }
}

// 加载当前赛区的夏季赛赛事
const loadTournament = async (regionId: number) => {
  try {
    const seasonId = gameStore.gameState?.current_season || 1
    console.log('[SummerDetail] loadTournament: regionId=', regionId, ', seasonId=', seasonId)
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    console.log('[SummerDetail] loadTournament: 获取到', tournaments.length, '个赛事')
    tournaments.forEach(t => console.log('[SummerDetail]   -', t.name, t.tournament_type, 'matches:', t.match_count))
    // 查找夏季常规赛 (后端存储格式为 PascalCase: SummerRegular)
    const summerRegular = tournaments.find(t => t.tournament_type === 'SummerRegular')
    if (summerRegular) {
      console.log('[SummerDetail] 找到夏季常规赛: id=', summerRegular.id, ', status=', summerRegular.status, ', matches=', summerRegular.match_count)
      currentTournamentId.value = summerRegular.id
      const statusLower = (summerRegular.status || '').toLowerCase()
      tournament.value = {
        id: summerRegular.id.toString(),
        name: summerRegular.name,
        type: 'league',
        status: (statusLower === 'inprogress' || statusLower === 'scheduled') ? 'active' :
                statusLower === 'completed' ? 'completed' : 'active',  // 默认为 active
        description: '夏季常规赛与季后赛',
      }
    } else {
      console.log('[SummerDetail] 未找到夏季常规赛!')
      // 没找到赛事也保持 active 状态，允许用户操作
      tournament.value.status = 'active'
    }
  } catch (error) {
    console.error('Failed to load tournament:', error)
  }
}

// 加载比赛列表
const loadMatches = async () => {
  if (!currentTournamentId.value) {
    console.log('[SummerDetail] loadMatches: 没有 currentTournamentId，跳过')
    return
  }
  try {
    console.log('[SummerDetail] loadMatches: tournamentId=', currentTournamentId.value)
    const matchList = await tournamentApi.getTournamentMatches(currentTournamentId.value)
    console.log('[SummerDetail] loadMatches: 获取到', matchList.length, '场比赛')
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
    console.error('Failed to load matches:', error)
  }
}

// 加载积分榜
const loadStandings = async () => {
  if (!currentTournamentId.value) {
    console.log('[SummerDetail] loadStandings: 没有 currentTournamentId，跳过')
    return
  }
  try {
    console.log('[SummerDetail] loadStandings: tournamentId=', currentTournamentId.value)
    const standingList = await tournamentApi.getStandings(currentTournamentId.value)
    console.log('[SummerDetail] loadStandings: 获取到', standingList.length, '条积分榜数据')
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
    console.error('Failed to load standings:', error)
  }
}

// 加载 MVP 排行榜
const loadMvpRanking = async () => {
  if (!currentTournamentId.value) return
  mvpLoading.value = true
  try {
    const ranking = await statsApi.getTournamentMvpRanking(currentTournamentId.value, 10)
    mvpRanking.value = ranking
    console.log('[SummerDetail] Loaded MVP ranking:', ranking.length, 'players')
  } catch (error) {
    console.error('Failed to load MVP ranking:', error)
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

// 手动刷新数据
const refreshData = async () => {
  refreshing.value = true
  try {
    await loadRegionData(selectedRegion.value)
    ElMessage.success('数据刷新成功')
  } catch (error) {
    console.error('刷新数据失败:', error)
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
    await matchDetailStore.saveMatchDetail(`summer-${match.id}`, matchDetail)

    // 记录选手表现到统计
    const regionName = getRegionName(selectedRegion.value)
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
    console.error('Failed to simulate match:', error)
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
    // 计算队伍平均发挥值
    const homeAvg = g.home_performance
    const awayAvg = g.away_performance

    return {
      gameNumber: g.game_number,
      teamAId: String(result.home_team_id),
      teamAName: result.home_team_name || match.homeTeam,
      teamAPower: g.home_performance,
      teamAPerformance: g.home_performance,
      teamAPlayers: (g.home_players || []).map((p: any) => {
        // 根据 MVP 分数模拟状态和波动
        const mvpScore = p.mvp_score || 0
        const conditionBonus = Math.round((mvpScore - 3) * 2 * 10) / 10  // 根据mvp分数推算状态
        const stabilityNoise = Math.round((Math.random() - 0.5) * 6 * 10) / 10  // 随机波动 -3 到 +3
        const baseAbility = Math.round(homeAvg - conditionBonus - stabilityNoise)

        return {
          playerId: String(p.player_id),
          playerName: p.player_name,
          teamId: String(result.home_team_id),
          position: normalizePosition(parsePosition(p.position)),
          baseAbility: Math.max(50, Math.min(100, baseAbility)),
          actualAbility: Math.round(homeAvg),
          conditionBonus: conditionBonus,
          stabilityNoise: stabilityNoise,
          impactScore: Math.round((mvpScore - 3) * 10) / 10,
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
      teamBPower: g.away_performance,
      teamBPerformance: g.away_performance,
      teamBPlayers: (g.away_players || []).map((p: any) => {
        // 根据 MVP 分数模拟状态和波动
        const mvpScore = p.mvp_score || 0
        const conditionBonus = Math.round((mvpScore - 3) * 2 * 10) / 10
        const stabilityNoise = Math.round((Math.random() - 0.5) * 6 * 10) / 10
        const baseAbility = Math.round(awayAvg - conditionBonus - stabilityNoise)

        return {
          playerId: String(p.player_id),
          playerName: p.player_name,
          teamId: String(result.away_team_id),
          position: normalizePosition(parsePosition(p.position)),
          baseAbility: Math.max(50, Math.min(100, baseAbility)),
          actualAbility: Math.round(awayAvg),
          conditionBonus: conditionBonus,
          stabilityNoise: stabilityNoise,
          impactScore: Math.round((mvpScore - 3) * 10) / 10,
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
      powerDifference: g.home_performance - g.away_performance,
      performanceDifference: g.home_performance - g.away_performance,
      isUpset: false,
    }
  })

  return {
    matchId: `summer-${match.id}`,
    seasonId: String(gameStore.gameState?.current_season || 1),
    tournamentType: 'summer',
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
    console.error('Failed to simulate next match:', error)
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

    ElMessage.success('常规赛模拟完成！请前往赛事管理页面进入季后赛')
  } catch (error) {
    console.error('Failed to simulate all matches:', error)
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
  // 先刷新时间状态，确保能获取最新的阶段信息
  await timeStore.fetchTimeState()

  // 加载赛区列表
  await loadRegions()
  // 加载默认赛区数据
  if (selectedRegion.value) {
    await loadRegionData(selectedRegion.value)
  }
})

// 监听时间状态变化，当阶段初始化后重新加载数据
watch(
  timeLastMessage,
  async (newVal, oldVal) => {
    console.log('[SummerDetail] watch lastMessage:', oldVal, '->', newVal)
    // 检测到初始化成功消息时刷新数据
    if (newVal && newVal.includes('成功创建') && selectedRegion.value) {
      console.log('[SummerDetail] 检测到阶段初始化成功，重新加载数据')
      // 等待一小段时间确保后端数据已持久化
      await new Promise(resolve => setTimeout(resolve, 100))
      await loadRegionData(selectedRegion.value)
    }
  }
)

// 监听赛事进度变化
watch(
  () => timeState.value?.phase_progress?.completed_matches,
  async (newVal, oldVal) => {
    // 当比赛完成数变化时，刷新积分榜
    if (newVal !== oldVal && selectedRegion.value && currentTournamentId.value) {
      await loadStandings()
    }
  }
)
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

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.green { background: linear-gradient(135deg, #22c55e, #16a34a); }
.stat-icon.orange { background: linear-gradient(135deg, #f97316, #ea580c); }
.stat-icon.purple { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

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

.team-avatar {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
  font-weight: 700;
}

.team-avatar.lpl { background: linear-gradient(135deg, #ef4444, #dc2626); }
.team-avatar.lck { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.team-avatar.lec { background: linear-gradient(135deg, #22c55e, #16a34a); }
.team-avatar.lcs { background: linear-gradient(135deg, #f59e0b, #d97706); }

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

/* 响应式 */
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
</style>
