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
          v-if="tournament.status === 'active' && currentPhase === 'regular'"
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
        <span class="selector-label">选择赛区:</span>
        <el-radio-group v-model="selectedRegion" @change="handleRegionChange">
          <el-radio-button v-for="region in regions" :key="region.id" :value="region.id">
            {{ region.name }}
          </el-radio-button>
        </el-radio-group>
      </div>
    </el-card>

    <!-- 阶段切换 -->
    <el-card class="phase-card">
      <el-tabs v-model="currentPhase" @tab-change="handlePhaseChange">
        <el-tab-pane label="常规赛" name="regular">
          <template #label>
            <span class="tab-label">
              <el-icon><List /></el-icon>
              常规赛
            </span>
          </template>
        </el-tab-pane>
        <el-tab-pane label="季后赛" name="playoffs" :disabled="!playoffsStarted">
          <template #label>
            <span class="tab-label">
              <el-icon><Trophy /></el-icon>
              季后赛
              <el-tag v-if="!playoffsStarted" size="small" type="info">未开始</el-tag>
            </span>
          </template>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 常规赛内容 -->
    <div v-if="currentPhase === 'regular'" class="regular-season-content">
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

    <!-- 季后赛内容 -->
    <div v-else-if="currentPhase === 'playoffs'" class="playoffs-content">
      <el-card class="bracket-card">
        <template #header>
          <div class="card-header">
            <h3>
              <el-icon><Trophy /></el-icon>
              季后赛对阵图
            </h3>
            <div class="header-actions">
              <el-button
                v-if="!playoffsCompleted"
                type="warning"
                size="small"
                @click="simulatePlayoffs"
                :loading="playoffsSimulating"
              >
                <el-icon><DArrowRight /></el-icon>
                一键模拟季后赛
              </el-button>
            </div>
          </div>
        </template>

        <!-- 双败淘汰赛对阵图 -->
        <div class="bracket-wrapper">
          <!-- 胜者组 -->
          <div class="bracket-section winners-section">
            <h4 class="section-title winners-title">
              <el-icon><Top /></el-icon>
              胜者组
            </h4>
            <div class="winners-bracket">
              <!-- 胜者组第一轮 -->
              <div class="bracket-column">
                <div class="round-label">第一轮</div>
                <div class="matches-column">
                  <div
                    v-for="match in winnersRounds[0].matches"
                    :key="match.id"
                    class="bracket-match winners"
                    :class="{ completed: match.status === 'completed' }"
                  >
                    <div class="match-team" :class="{ winner: match.winnerId === match.teamAId }">
                      <span class="seed">#{{ match.seedA }}</span>
                      <span class="name">{{ match.teamA || '待定' }}</span>
                      <span class="score" v-if="match.status === 'completed'">{{ match.scoreA }}</span>
                    </div>
                    <div class="match-team" :class="{ winner: match.winnerId === match.teamBId }">
                      <span class="seed">#{{ match.seedB }}</span>
                      <span class="name">{{ match.teamB || '待定' }}</span>
                      <span class="score" v-if="match.status === 'completed'">{{ match.scoreB }}</span>
                    </div>
                    <div class="connector-right"></div>
                  </div>
                </div>
              </div>

              <!-- 胜者组决赛 -->
              <div class="bracket-column">
                <div class="round-label">决赛</div>
                <div class="matches-column centered">
                  <div
                    class="bracket-match winners final"
                    :class="{ completed: winnersRounds[1].matches[0].status === 'completed' }"
                  >
                    <div class="connector-left"></div>
                    <div class="match-team" :class="{ winner: winnersRounds[1].matches[0].winnerId === winnersRounds[1].matches[0].teamAId }">
                      <span class="name">{{ winnersRounds[1].matches[0].teamA || '待定' }}</span>
                      <span class="score" v-if="winnersRounds[1].matches[0].status === 'completed'">{{ winnersRounds[1].matches[0].scoreA }}</span>
                    </div>
                    <div class="match-team" :class="{ winner: winnersRounds[1].matches[0].winnerId === winnersRounds[1].matches[0].teamBId }">
                      <span class="name">{{ winnersRounds[1].matches[0].teamB || '待定' }}</span>
                      <span class="score" v-if="winnersRounds[1].matches[0].status === 'completed'">{{ winnersRounds[1].matches[0].scoreB }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 败者组 -->
          <div class="bracket-section losers-section">
            <h4 class="section-title losers-title">
              <el-icon><Bottom /></el-icon>
              败者组
            </h4>
            <div class="losers-bracket">
              <!-- 败者组第一轮 (5v8, 6v7) -->
              <div class="bracket-column">
                <div class="round-label">第一轮</div>
                <div class="matches-column">
                  <div
                    v-for="match in losersRounds[0].matches"
                    :key="match.id"
                    class="bracket-match losers"
                    :class="{ completed: match.status === 'completed' }"
                  >
                    <div class="match-team" :class="{ winner: match.winnerId === match.teamAId }">
                      <span class="seed" v-if="'seedA' in match">#{{ match.seedA }}</span>
                      <span class="name">{{ match.teamA || '待定' }}</span>
                      <span class="score" v-if="match.status === 'completed'">{{ match.scoreA }}</span>
                    </div>
                    <div class="match-team" :class="{ winner: match.winnerId === match.teamBId }">
                      <span class="seed" v-if="'seedB' in match">#{{ match.seedB }}</span>
                      <span class="name">{{ match.teamB || '待定' }}</span>
                      <span class="score" v-if="match.status === 'completed'">{{ match.scoreB }}</span>
                    </div>
                    <div class="connector-right"></div>
                  </div>
                </div>
              </div>

              <!-- 败者组第二轮 -->
              <div class="bracket-column">
                <div class="round-label">第二轮</div>
                <div class="matches-column">
                  <div
                    v-for="match in losersRounds[1].matches"
                    :key="match.id"
                    class="bracket-match losers"
                    :class="{ completed: match.status === 'completed' }"
                  >
                    <div class="connector-left"></div>
                    <div class="match-team" :class="{ winner: match.winnerId === match.teamAId }">
                      <span class="name">{{ match.teamA || '待定' }}</span>
                      <span class="score" v-if="match.status === 'completed'">{{ match.scoreA }}</span>
                    </div>
                    <div class="match-team" :class="{ winner: match.winnerId === match.teamBId }">
                      <span class="name">{{ match.teamB || '待定' }}</span>
                      <span class="score" v-if="match.status === 'completed'">{{ match.scoreB }}</span>
                    </div>
                    <div class="connector-right"></div>
                  </div>
                </div>
              </div>

              <!-- 败者组第三轮 -->
              <div class="bracket-column">
                <div class="round-label">第三轮</div>
                <div class="matches-column centered">
                  <div
                    class="bracket-match losers"
                    :class="{ completed: losersRounds[2].matches[0].status === 'completed' }"
                  >
                    <div class="connector-left"></div>
                    <div class="match-team" :class="{ winner: losersRounds[2].matches[0].winnerId === losersRounds[2].matches[0].teamAId }">
                      <span class="name">{{ losersRounds[2].matches[0].teamA || '待定' }}</span>
                      <span class="score" v-if="losersRounds[2].matches[0].status === 'completed'">{{ losersRounds[2].matches[0].scoreA }}</span>
                    </div>
                    <div class="match-team" :class="{ winner: losersRounds[2].matches[0].winnerId === losersRounds[2].matches[0].teamBId }">
                      <span class="name">{{ losersRounds[2].matches[0].teamB || '待定' }}</span>
                      <span class="score" v-if="losersRounds[2].matches[0].status === 'completed'">{{ losersRounds[2].matches[0].scoreB }}</span>
                    </div>
                    <div class="connector-right"></div>
                  </div>
                </div>
              </div>

              <!-- 败者组决赛 -->
              <div class="bracket-column">
                <div class="round-label">败者组决赛</div>
                <div class="matches-column centered">
                  <div
                    class="bracket-match losers final"
                    :class="{ completed: losersRounds[3].matches[0].status === 'completed' }"
                  >
                    <div class="connector-left"></div>
                    <div class="match-team" :class="{ winner: losersRounds[3].matches[0].winnerId === losersRounds[3].matches[0].teamAId }">
                      <span class="name">{{ losersRounds[3].matches[0].teamA || '待定' }}</span>
                      <span class="score" v-if="losersRounds[3].matches[0].status === 'completed'">{{ losersRounds[3].matches[0].scoreA }}</span>
                    </div>
                    <div class="match-team" :class="{ winner: losersRounds[3].matches[0].winnerId === losersRounds[3].matches[0].teamBId }">
                      <span class="name">{{ losersRounds[3].matches[0].teamB || '待定' }}</span>
                      <span class="score" v-if="losersRounds[3].matches[0].status === 'completed'">{{ losersRounds[3].matches[0].scoreB }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 总决赛 -->
          <div class="bracket-section finals-section">
            <h4 class="section-title finals-title">
              <el-icon><Trophy /></el-icon>
              总决赛
            </h4>
            <div class="grand-final">
              <div class="final-match-card" :class="{ completed: finalMatch.status === 'completed' }">
                <div class="final-team" :class="{ champion: finalMatch.winnerId === finalMatch.teamAId }">
                  <div class="team-source">胜者组冠军</div>
                  <div class="team-name">{{ finalMatch.teamA || '待定' }}</div>
                  <div class="team-score" v-if="finalMatch.status === 'completed'">{{ finalMatch.scoreA }}</div>
                </div>
                <div class="vs-badge">VS</div>
                <div class="final-team" :class="{ champion: finalMatch.winnerId === finalMatch.teamBId }">
                  <div class="team-source">败者组冠军</div>
                  <div class="team-name">{{ finalMatch.teamB || '待定' }}</div>
                  <div class="team-score" v-if="finalMatch.status === 'completed'">{{ finalMatch.scoreB }}</div>
                </div>
              </div>

              <!-- 冠军展示 -->
              <div v-if="champion" class="champion-display">
                <div class="champion-trophy">🏆</div>
                <div class="champion-info">
                  <div class="champion-label">冠军</div>
                  <div class="champion-name">{{ champion.name }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-card>
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
  Trophy,
  Medal,
  UserFilled,
  Calendar,
  TrendCharts,
  Top,
  Bottom,
} from '@element-plus/icons-vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { PowerEngine } from '@/engines/PowerEngine'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, teamApi, tournamentApi, matchApi, type Team } from '@/api/tauri'
import type { Player, PlayerPosition } from '@/types/player'
import type { MatchDetail, GameDetail } from '@/types/matchDetail'

const route = useRoute()
const router = useRouter()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 状态
const currentPhase = ref<'regular' | 'playoffs'>('regular')
const selectedRegion = ref(1) // 默认 LPL region_id = 1
const matchFilter = ref('all')
const simulating = ref(false)
const batchSimulating = ref(false)
const playoffsSimulating = ref(false)
const loading = ref(false)

// 当前赛事ID
const currentTournamentId = ref<number | null>(null)

// 赛事信息 (从后端加载)
const tournament = ref({
  id: route.params.id,
  name: '春季赛',
  type: 'league',
  status: 'active',
  description: '四大赛区春季常规赛与季后赛',
})

// 赛区数据 (从后端加载)
const regions = ref<{ id: number; name: string }[]>([])

// 队伍ID到名称的映射 (从后端加载)
const teamMap = ref<Map<number, Team>>(new Map())

// 当前显示的积分榜数据 (从后端加载)
const standings = ref<any[]>([])

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

// 加载当前赛区的春季赛赛事
const loadTournament = async (regionId: number) => {
  try {
    const seasonId = gameStore.gameState?.current_season || 1
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    // 查找春季常规赛 (后端存储格式为 PascalCase: SpringRegular)
    const springRegular = tournaments.find(t => t.tournament_type === 'SpringRegular')
    if (springRegular) {
      currentTournamentId.value = springRegular.id
      tournament.value = {
        id: springRegular.id.toString(),
        name: springRegular.name,
        type: 'league',
        status: springRegular.status === 'InProgress' ? 'active' : springRegular.status.toLowerCase(),
        description: '春季常规赛与季后赛',
      }
    }
  } catch (error) {
    console.error('Failed to load tournament:', error)
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
      status: m.status === 'Completed' ? 'completed' : m.status === 'InProgress' ? 'active' : 'upcoming',
      simulating: false,
    }))
  } catch (error) {
    console.error('Failed to load matches:', error)
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
    console.error('Failed to load standings:', error)
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
  } finally {
    loading.value = false
  }
}

// 季后赛数据
const playoffsStarted = ref(false)
const playoffsCompleted = ref(false)
const champion = ref<{ name: string } | null>(null)

// 胜者组轮次 (1-4名: 1 vs 4, 2 vs 3)
const winnersRounds = ref([
  {
    name: '胜者组第一轮',
    matches: [
      { id: 'w1', teamAId: 1, teamA: 'JDG', seedA: 1, teamBId: 4, teamB: 'WBG', seedB: 4, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
      { id: 'w2', teamAId: 2, teamA: 'BLG', seedA: 2, teamBId: 3, teamB: 'TES', seedB: 3, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    ]
  },
  {
    name: '胜者组决赛',
    matches: [
      { id: 'w3', teamAId: null, teamA: '', seedA: null, teamBId: null, teamB: '', seedB: null, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    ]
  }
])

// 败者组轮次 (5-8名: 5 vs 8, 6 vs 7，加上胜者组掉落的队伍)
const losersRounds = ref([
  {
    name: '败者组第一轮',
    desc: '5 vs 8, 6 vs 7',
    matches: [
      { id: 'l1', teamAId: 5, teamA: 'LNG', seedA: 5, teamBId: 8, teamB: 'RNG', seedB: 8, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
      { id: 'l2', teamAId: 6, teamA: 'EDG', seedA: 6, teamBId: 7, teamB: 'FPX', seedB: 7, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    ]
  },
  {
    name: '败者组第二轮',
    desc: '败者组R1胜者 vs 胜者组R1败者',
    matches: [
      { id: 'l3', teamAId: null, teamA: '', seedA: null, teamBId: null, teamB: '', seedB: null, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
      { id: 'l4', teamAId: null, teamA: '', seedA: null, teamBId: null, teamB: '', seedB: null, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    ]
  },
  {
    name: '败者组第三轮',
    desc: '败者组R2两个胜者对决',
    matches: [
      { id: 'l5', teamAId: null, teamA: '', teamBId: null, teamB: '', scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    ]
  },
  {
    name: '败者组决赛',
    desc: '败者组R3胜者 vs 胜者组决赛败者',
    matches: [
      { id: 'l6', teamAId: null, teamA: '', teamBId: null, teamB: '', scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    ]
  }
])

// 总决赛
const finalMatch = ref({
  id: 'final',
  teamAId: null,
  teamA: '',
  teamBId: null,
  teamB: '',
  scoreA: 0,
  scoreB: 0,
  winnerId: null,
  status: 'upcoming'
})

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
 * 生成队伍选手数据
 */
const generateTeamPlayers = (teamId: number, teamName: string, regionName: string = 'Unknown'): Player[] => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  return positions.map((pos, idx) => ({
    id: `${teamId}-${pos}`,
    gameId: `Player${idx + 1}`,
    name: `Player${idx + 1}`,
    teamId: String(teamId),
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
const viewMatchDetails = (matchId: string) => {
  const detail = matchDetailStore.getMatchDetail(matchId)
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
  // 重置季后赛状态
  playoffsStarted.value = false
  playoffsCompleted.value = false
  champion.value = null
  currentPhase.value = 'regular'

  // 加载新赛区数据
  await loadRegionData(regionId)

  // 根据赛区更新季后赛数据
  updatePlayoffsData()

  const regionName = regions.value.find(r => r.id === regionId)?.name || '未知'
  ElMessage.success(`已切换到 ${regionName} 赛区`)
}

// 更新季后赛对阵数据
const updatePlayoffsData = () => {
  if (standings.value.length < 8) return

  // 按积分排序
  const sortedTeams = [...standings.value].sort((a, b) => b.points - a.points)

  // 更新胜者组第一轮 (1 vs 4, 2 vs 3)
  winnersRounds.value[0].matches = [
    {
      id: 'w1',
      teamAId: sortedTeams[0].id,
      teamA: sortedTeams[0].short,
      seedA: 1,
      teamBId: sortedTeams[3].id,
      teamB: sortedTeams[3].short,
      seedB: 4,
      scoreA: 0,
      scoreB: 0,
      winnerId: null,
      status: 'upcoming'
    },
    {
      id: 'w2',
      teamAId: sortedTeams[1].id,
      teamA: sortedTeams[1].short,
      seedA: 2,
      teamBId: sortedTeams[2].id,
      teamB: sortedTeams[2].short,
      seedB: 3,
      scoreA: 0,
      scoreB: 0,
      winnerId: null,
      status: 'upcoming'
    }
  ]

  // 重置胜者组决赛
  winnersRounds.value[1].matches = [
    { id: 'w3', teamAId: null, teamA: '', seedA: null, teamBId: null, teamB: '', seedB: null, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' }
  ]

  // 更新败者组第一轮 (5 vs 8, 6 vs 7)
  losersRounds.value[0].matches = [
    {
      id: 'l1',
      teamAId: sortedTeams[4].id,
      teamA: sortedTeams[4].short,
      seedA: 5,
      teamBId: sortedTeams[7]?.id || null,
      teamB: sortedTeams[7]?.short || '待定',
      seedB: 8,
      scoreA: 0,
      scoreB: 0,
      winnerId: null,
      status: 'upcoming'
    },
    {
      id: 'l2',
      teamAId: sortedTeams[5].id,
      teamA: sortedTeams[5].short,
      seedA: 6,
      teamBId: sortedTeams[6]?.id || null,
      teamB: sortedTeams[6]?.short || '待定',
      seedB: 7,
      scoreA: 0,
      scoreB: 0,
      winnerId: null,
      status: 'upcoming'
    }
  ]

  // 重置败者组其他轮次
  losersRounds.value[1].matches = [
    { id: 'l3', teamAId: null, teamA: '', seedA: null, teamBId: null, teamB: '', seedB: null, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' },
    { id: 'l4', teamAId: null, teamA: '', seedA: null, teamBId: null, teamB: '', seedB: null, scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' }
  ]
  losersRounds.value[2].matches = [
    { id: 'l5', teamAId: null, teamA: '', teamBId: null, teamB: '', scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' }
  ]
  losersRounds.value[3].matches = [
    { id: 'l6', teamAId: null, teamA: '', teamBId: null, teamB: '', scoreA: 0, scoreB: 0, winnerId: null, status: 'upcoming' }
  ]

  // 重置总决赛
  finalMatch.value = {
    id: 'final',
    teamAId: null,
    teamA: '',
    teamBId: null,
    teamB: '',
    scoreA: 0,
    scoreB: 0,
    winnerId: null,
    status: 'upcoming'
  }
}

const handlePhaseChange = (phase: string) => {
  if (phase === 'playoffs' && !playoffsStarted.value) {
    // 检查常规赛是否结束
    const allCompleted = matches.value.every(m => m.status === 'completed')
    if (allCompleted) {
      playoffsStarted.value = true
    }
  }
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
    matchDetailStore.saveMatchDetail(`spring-${match.id}`, matchDetail)

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
          '2024',
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
          '2024',
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
      playoffsStarted.value = true
      ElMessage.success('常规赛全部完成！季后赛已开启')
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
    seasonId: String(gameStore.gameState?.current_season || 1),
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
    mvpPlayerId: result.mvp?.player_id ? String(result.mvp.player_id) : undefined,
    mvpPlayerName: result.mvp?.player_name,
    mvpTeamId: result.mvp?.team_id ? String(result.mvp.team_id) : undefined,
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

    // 开启季后赛
    playoffsStarted.value = true
    ElMessage.success('常规赛模拟完成！季后赛已开启')
  } catch (error) {
    console.error('Failed to simulate all matches:', error)
    ElMessage.error('模拟比赛失败')
  } finally {
    batchSimulating.value = false
  }
}

const simulatePlayoffs = async () => {
  await ElMessageBox.confirm('将自动模拟整个季后赛，是否继续？', '模拟季后赛', {
    confirmButtonText: '开始',
    cancelButtonText: '取消',
    type: 'warning'
  })

  playoffsSimulating.value = true

  // 辅助函数：使用PowerEngine模拟单场比赛
  const regionName = getRegionName(selectedRegion.value)
  const simulateMatch = (match: any, matchIdPrefix: string) => {
    const teamAPlayers = generateTeamPlayers(match.teamAId, match.teamA, regionName)
    const teamBPlayers = generateTeamPlayers(match.teamBId, match.teamB, regionName)

    const matchDetail = PowerEngine.simulateMatch(
      String(match.teamAId),
      match.teamA,
      teamAPlayers,
      String(match.teamBId),
      match.teamB,
      teamBPlayers,
      5 // BO5
    )

    match.scoreA = matchDetail.finalScoreA
    match.scoreB = matchDetail.finalScoreB
    match.winnerId = matchDetail.winnerId === String(match.teamAId) ? match.teamAId : match.teamBId
    match.status = 'completed'

    // 保存比赛详情
    matchDetail.matchId = `spring-playoffs-${matchIdPrefix}`
    matchDetail.tournamentType = 'spring-playoffs'
    matchDetail.seasonId = '2024'
    matchDetailStore.saveMatchDetail(matchDetail.matchId, matchDetail)

    // 记录选手表现
    matchDetail.games.forEach(game => {
      game.teamAPlayers.forEach(perf => {
        playerStore.recordPerformance(
        perf.playerId,
        perf.playerName,
        perf.teamId,
        perf.position,
        perf.impactScore,
        perf.actualAbility,
        '2024',
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
        '2024',
        regionName
      )
      })
    })

    return matchDetail.winnerId === String(match.teamAId)
  }

  // 获取队伍名称
  const getTeamName = (teamId: number | null) => {
    const team = standings.value.find(t => t.id === teamId)
    return team?.short || '待定'
  }

  await new Promise(resolve => setTimeout(resolve, 500))

  // ========== 第一阶段：胜者组第一轮 + 败者组第一轮 ==========
  // 胜者组第一轮：1 vs 4, 2 vs 3
  const w1Match1 = winnersRounds.value[0].matches[0] // 1 vs 4
  const w1Match2 = winnersRounds.value[0].matches[1] // 2 vs 3
  simulateMatch(w1Match1, 'w1-1')
  simulateMatch(w1Match2, 'w1-2')

  // 败者组第一轮：5 vs 8, 6 vs 7
  const l1Match1 = losersRounds.value[0].matches[0] // 5 vs 8
  const l1Match2 = losersRounds.value[0].matches[1] // 6 vs 7
  simulateMatch(l1Match1, 'l1-1')
  simulateMatch(l1Match2, 'l1-2')

  await new Promise(resolve => setTimeout(resolve, 500))

  // ========== 第二阶段：胜者组决赛 + 败者组第二轮 ==========
  // 胜者组决赛
  const wfMatch = winnersRounds.value[1].matches[0]
  wfMatch.teamAId = w1Match1.winnerId
  wfMatch.teamA = getTeamName(w1Match1.winnerId)
  wfMatch.teamBId = w1Match2.winnerId
  wfMatch.teamB = getTeamName(w1Match2.winnerId)
  simulateMatch(wfMatch, 'wf')

  // 败者组第二轮：败者组R1胜者 vs 胜者组R1败者
  const l2Match1 = losersRounds.value[1].matches[0]
  const l2Match2 = losersRounds.value[1].matches[1]

  // 败者组R1胜者 vs 胜者组R1败者1
  l2Match1.teamAId = l1Match1.winnerId
  l2Match1.teamA = getTeamName(l1Match1.winnerId)
  l2Match1.teamBId = w1Match1.winnerId === w1Match1.teamAId ? w1Match1.teamBId : w1Match1.teamAId
  l2Match1.teamB = getTeamName(l2Match1.teamBId)
  simulateMatch(l2Match1, 'l2-1')

  // 败者组R1胜者 vs 胜者组R1败者2
  l2Match2.teamAId = l1Match2.winnerId
  l2Match2.teamA = getTeamName(l1Match2.winnerId)
  l2Match2.teamBId = w1Match2.winnerId === w1Match2.teamAId ? w1Match2.teamBId : w1Match2.teamAId
  l2Match2.teamB = getTeamName(l2Match2.teamBId)
  simulateMatch(l2Match2, 'l2-2')

  await new Promise(resolve => setTimeout(resolve, 500))

  // ========== 第三阶段：败者组第三轮 ==========
  // 败者组R2两个胜者对决
  const l3Match = losersRounds.value[2].matches[0]
  l3Match.teamAId = l2Match1.winnerId
  l3Match.teamA = getTeamName(l2Match1.winnerId)
  l3Match.teamBId = l2Match2.winnerId
  l3Match.teamB = getTeamName(l2Match2.winnerId)
  simulateMatch(l3Match, 'l3')

  await new Promise(resolve => setTimeout(resolve, 500))

  // ========== 第四阶段：败者组决赛 ==========
  // 败者组R3胜者 vs 胜者组决赛败者
  const lfMatch = losersRounds.value[3].matches[0]
  lfMatch.teamAId = l3Match.winnerId
  lfMatch.teamA = getTeamName(l3Match.winnerId)
  lfMatch.teamBId = wfMatch.winnerId === wfMatch.teamAId ? wfMatch.teamBId : wfMatch.teamAId
  lfMatch.teamB = getTeamName(lfMatch.teamBId)
  simulateMatch(lfMatch, 'lf')

  await new Promise(resolve => setTimeout(resolve, 500))

  // ========== 总决赛 ==========
  finalMatch.value.teamAId = wfMatch.winnerId
  finalMatch.value.teamA = getTeamName(wfMatch.winnerId)
  finalMatch.value.teamBId = lfMatch.winnerId
  finalMatch.value.teamB = getTeamName(lfMatch.winnerId)
  simulateMatch(finalMatch.value, 'final')

  champion.value = { name: getTeamName(finalMatch.value.winnerId) }
  playoffsCompleted.value = true
  playoffsSimulating.value = false

  ElMessage.success(`🏆 恭喜 ${champion.value.name} 获得冠军！`)
}

const updateStandings = async () => {
  // 从后端重新加载积分榜
  await loadStandings()
}

onMounted(async () => {
  // 加载赛区列表
  await loadRegions()
  // 加载默认赛区数据
  if (selectedRegion.value) {
    await loadRegionData(selectedRegion.value)
    updatePlayoffsData()
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
