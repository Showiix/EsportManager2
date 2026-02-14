<template>
  <div class="summer-playoffs-management">
    <!-- 页面头部 -->
    <div class="page-header">
      <div>
        <h1>{{ getRegionName(selectedRegion) }} 夏季季后赛</h1>
        <p>常规赛前8名队伍 · 双败淘汰制</p>
      </div>
      <div class="header-actions">
        <el-button
          v-if="regularSeasonCompleted && !playoffsCompleted"
          type="primary"
          size="small"
          @click="simulatePlayoffs"
          :loading="playoffsSimulating"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ playoffsSimulating ? `模拟中 (${playoffsProgress}%)` : '一键模拟季后赛' }}
        </el-button>
        <button class="back-btn" @click="goBack">← 返回赛事列表</button>
      </div>
    </div>

    <!-- 模拟进度条 -->
    <el-progress
      v-if="playoffsSimulating"
      :percentage="playoffsProgress"
      :stroke-width="6"
      :show-text="false"
      style="margin-bottom: 12px;"
    />

    <!-- 赛区选择器 -->
    <div class="filter-section">
      <div class="filter-row">
        <div class="filter-group">
          <label>赛区</label>
          <el-radio-group v-model="selectedRegion" @change="handleRegionChange" size="small">
            <el-radio-button v-for="region in regions" :key="region.id" :value="region.id">
              {{ region.name }}
            </el-radio-button>
          </el-radio-group>
        </div>
        <el-tag :type="playoffsCompleted ? 'success' : regularSeasonCompleted ? 'warning' : 'info'" size="small">
          {{ playoffsCompleted ? '已完成' : regularSeasonCompleted ? '进行中' : '等待常规赛' }}
        </el-tag>
      </div>
    </div>

    <!-- 常规赛未完成提示 -->
    <el-alert
      v-if="!regularSeasonCompleted"
      title="常规赛尚未完成"
      description="请先完成夏季赛常规赛，季后赛排名将根据常规赛积分自动确定。"
      type="warning"
      :closable="false"
      show-icon
      style="margin-bottom: 16px;"
    />

    <!-- 统计栏 -->
    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">8</span>
        <span class="stat-label">参赛队伍</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">4</span>
        <span class="stat-label">胜者组</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">4</span>
        <span class="stat-label">败者组</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">BO5</span>
        <span class="stat-label">赛制</span>
      </div>
    </div>

    <!-- 完整对阵图 -->
    <div class="table-section">
      <div class="section-header">
        <span class="section-title">双败淘汰赛对阵图</span>
      </div>

        <div class="full-bracket">
          <!-- 胜者组 -->
          <div class="bracket-section winners-section">
            <div class="section-label winners">胜者组</div>

            <div class="bracket-container">
              <!-- 胜者组第一轮 -->
              <div class="bracket-round">
                <div class="round-header">第一轮</div>
                <div class="matches-column">
                  <div
                    v-for="(match, idx) in winnersRounds[0].matches"
                    :key="match.id"
                    class="match-card-wrapper"
                  >
                    <div class="match-card" :class="{ completed: match.status === 'completed', winners: true }">
                      <div class="match-teams">
                        <div class="match-team" :class="{ winner: match.winnerId === match.teamAId }">
                          <span class="seed">#{{ match.seedA }}</span>
                          <span class="name">{{ regularSeasonCompleted ? (match.teamA || '待定') : '待定' }}</span>
                          <span class="score">{{ match.status === 'completed' ? match.scoreA : '-' }}</span>
                        </div>
                        <div class="match-team" :class="{ winner: match.winnerId === match.teamBId }">
                          <span class="seed">#{{ match.seedB }}</span>
                          <span class="name">{{ regularSeasonCompleted ? (match.teamB || '待定') : '待定' }}</span>
                          <span class="score">{{ match.status === 'completed' ? match.scoreB : '-' }}</span>
                        </div>
                      </div>
                      <div class="match-actions">
                        <el-button
                          v-if="match.status !== 'completed' && canSimulate(match)"
                          type="primary"
                          size="small"
                          @click="simulateSingleMatch(match, `w1-${idx+1}`)"
                          :loading="simulatingMatchId === match.id"
                        >
                          <el-icon><VideoPlay /></el-icon>
                          模拟
                        </el-button>
                        <el-button
                          v-if="match.status === 'completed'"
                          type="primary"
                          size="small"
                          text
                          @click="viewMatchDetail(match)"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 连接线: 第一轮 -> 胜者组决赛 -->
              <div class="bracket-connector connector-merge winners-color"></div>

              <!-- 胜者组决赛 -->
              <div class="bracket-round">
                <div class="round-header">胜者组决赛</div>
                <div class="matches-column final">
                  <div class="match-card-wrapper">
                    <div class="match-card final-match" :class="{ completed: winnersRounds[1].matches[0].status === 'completed', winners: true }">
                      <div class="match-teams">
                        <div class="match-team" :class="{ winner: winnersRounds[1].matches[0].winnerId === winnersRounds[1].matches[0].teamAId }">
                          <span class="name">{{ winnersRounds[1].matches[0].teamA || '待定' }}</span>
                          <span class="score">{{ winnersRounds[1].matches[0].status === 'completed' ? winnersRounds[1].matches[0].scoreA : '-' }}</span>
                        </div>
                        <div class="match-team" :class="{ winner: winnersRounds[1].matches[0].winnerId === winnersRounds[1].matches[0].teamBId }">
                          <span class="name">{{ winnersRounds[1].matches[0].teamB || '待定' }}</span>
                          <span class="score">{{ winnersRounds[1].matches[0].status === 'completed' ? winnersRounds[1].matches[0].scoreB : '-' }}</span>
                        </div>
                      </div>
                      <div class="match-actions">
                        <el-button
                          v-if="winnersRounds[1].matches[0].status !== 'completed' && canSimulate(winnersRounds[1].matches[0])"
                          type="primary"
                          size="small"
                          @click="simulateSingleMatch(winnersRounds[1].matches[0], 'wf')"
                          :loading="simulatingMatchId === winnersRounds[1].matches[0].id"
                        >
                          <el-icon><VideoPlay /></el-icon>
                          模拟
                        </el-button>
                        <el-button
                          v-if="winnersRounds[1].matches[0].status === 'completed'"
                          type="primary"
                          size="small"
                          text
                          @click="viewMatchDetail(winnersRounds[1].matches[0])"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 败者组 -->
          <div class="bracket-section losers-section">
            <div class="section-label losers">败者组</div>

            <div class="bracket-container">
              <!-- 败者组第一轮 -->
              <div class="bracket-round">
                <div class="round-header">第一轮</div>
                <div class="matches-column">
                  <div
                    v-for="(match, idx) in losersRounds[0].matches"
                    :key="match.id"
                    class="match-card-wrapper"
                  >
                    <div class="match-card" :class="{ completed: match.status === 'completed', losers: true }">
                      <div class="match-teams">
                        <div class="match-team" :class="{ winner: match.winnerId === match.teamAId }">
                          <span class="seed">#{{ match.seedA }}</span>
                          <span class="name">{{ regularSeasonCompleted ? (match.teamA || '待定') : '待定' }}</span>
                          <span class="score">{{ match.status === 'completed' ? match.scoreA : '-' }}</span>
                        </div>
                        <div class="match-team" :class="{ winner: match.winnerId === match.teamBId }">
                          <span class="seed">#{{ match.seedB }}</span>
                          <span class="name">{{ regularSeasonCompleted ? (match.teamB || '待定') : '待定' }}</span>
                          <span class="score">{{ match.status === 'completed' ? match.scoreB : '-' }}</span>
                        </div>
                      </div>
                      <div class="match-actions">
                        <el-button
                          v-if="match.status !== 'completed' && canSimulate(match)"
                          type="warning"
                          size="small"
                          @click="simulateSingleMatch(match, `l1-${idx+1}`)"
                          :loading="simulatingMatchId === match.id"
                        >
                          <el-icon><VideoPlay /></el-icon>
                          模拟
                        </el-button>
                        <el-button
                          v-if="match.status === 'completed'"
                          type="primary"
                          size="small"
                          text
                          @click="viewMatchDetail(match)"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 连接线: 败者组第一轮 -> 第二轮 -->
              <div class="bracket-connector connector-parallel losers-color"></div>

              <!-- 败者组第二轮 -->
              <div class="bracket-round">
                <div class="round-header">第二轮 <span class="drop-hint">(胜者组败者加入)</span></div>
                <div class="matches-column">
                  <div
                    v-for="(match, idx) in losersRounds[1].matches"
                    :key="match.id"
                    class="match-card-wrapper"
                  >
                    <div class="match-card" :class="{ completed: match.status === 'completed', losers: true }">
                      <div class="drop-in-indicator">
                        <span>↓ 胜者组败者</span>
                      </div>
                      <div class="match-teams">
                        <div class="match-team" :class="{ winner: match.winnerId === match.teamAId }">
                          <span class="name">{{ match.teamA || '待定' }}</span>
                          <span class="score">{{ match.status === 'completed' ? match.scoreA : '-' }}</span>
                        </div>
                        <div class="match-team" :class="{ winner: match.winnerId === match.teamBId }">
                          <span class="name">{{ match.teamB || '待定' }}</span>
                          <span class="score">{{ match.status === 'completed' ? match.scoreB : '-' }}</span>
                        </div>
                      </div>
                      <div class="match-actions">
                        <el-button
                          v-if="match.status !== 'completed' && canSimulate(match)"
                          type="warning"
                          size="small"
                          @click="simulateSingleMatch(match, `l2-${idx+1}`)"
                          :loading="simulatingMatchId === match.id"
                        >
                          <el-icon><VideoPlay /></el-icon>
                          模拟
                        </el-button>
                        <el-button
                          v-if="match.status === 'completed'"
                          type="primary"
                          size="small"
                          text
                          @click="viewMatchDetail(match)"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 连接线: 败者组第二轮 -> 第三轮 -->
              <div class="bracket-connector connector-merge losers-color"></div>

              <!-- 败者组第三轮 -->
              <div class="bracket-round">
                <div class="round-header">第三轮</div>
                <div class="matches-column final">
                  <div class="match-card-wrapper">
                    <div class="match-card" :class="{ completed: losersRounds[2].matches[0].status === 'completed', losers: true }">
                      <div class="match-teams">
                        <div class="match-team" :class="{ winner: losersRounds[2].matches[0].winnerId === losersRounds[2].matches[0].teamAId }">
                          <span class="name">{{ losersRounds[2].matches[0].teamA || '待定' }}</span>
                          <span class="score">{{ losersRounds[2].matches[0].status === 'completed' ? losersRounds[2].matches[0].scoreA : '-' }}</span>
                        </div>
                        <div class="match-team" :class="{ winner: losersRounds[2].matches[0].winnerId === losersRounds[2].matches[0].teamBId }">
                          <span class="name">{{ losersRounds[2].matches[0].teamB || '待定' }}</span>
                          <span class="score">{{ losersRounds[2].matches[0].status === 'completed' ? losersRounds[2].matches[0].scoreB : '-' }}</span>
                        </div>
                      </div>
                      <div class="match-actions">
                        <el-button
                          v-if="losersRounds[2].matches[0].status !== 'completed' && canSimulate(losersRounds[2].matches[0])"
                          type="warning"
                          size="small"
                          @click="simulateSingleMatch(losersRounds[2].matches[0], 'l3')"
                          :loading="simulatingMatchId === losersRounds[2].matches[0].id"
                        >
                          <el-icon><VideoPlay /></el-icon>
                          模拟
                        </el-button>
                        <el-button
                          v-if="losersRounds[2].matches[0].status === 'completed'"
                          type="primary"
                          size="small"
                          text
                          @click="viewMatchDetail(losersRounds[2].matches[0])"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 连接线: 败者组第三轮 -> 败者组决赛 -->
              <div class="bracket-connector connector-straight losers-color"></div>

              <!-- 败者组决赛 -->
              <div class="bracket-round">
                <div class="round-header">败者组决赛 <span class="drop-hint">(胜者组亚军加入)</span></div>
                <div class="matches-column final">
                  <div class="match-card-wrapper">
                    <div class="match-card final-match" :class="{ completed: losersRounds[3].matches[0].status === 'completed', losers: true }">
                      <div class="drop-in-indicator">
                        <span>↓ 胜者组亚军</span>
                      </div>
                      <div class="match-teams">
                        <div class="match-team" :class="{ winner: losersRounds[3].matches[0].winnerId === losersRounds[3].matches[0].teamAId }">
                          <span class="name">{{ losersRounds[3].matches[0].teamA || '待定' }}</span>
                          <span class="score">{{ losersRounds[3].matches[0].status === 'completed' ? losersRounds[3].matches[0].scoreA : '-' }}</span>
                        </div>
                        <div class="match-team" :class="{ winner: losersRounds[3].matches[0].winnerId === losersRounds[3].matches[0].teamBId }">
                          <span class="name">{{ losersRounds[3].matches[0].teamB || '待定' }}</span>
                          <span class="score">{{ losersRounds[3].matches[0].status === 'completed' ? losersRounds[3].matches[0].scoreB : '-' }}</span>
                        </div>
                      </div>
                      <div class="match-actions">
                        <el-button
                          v-if="losersRounds[3].matches[0].status !== 'completed' && canSimulate(losersRounds[3].matches[0])"
                          type="warning"
                          size="small"
                          @click="simulateSingleMatch(losersRounds[3].matches[0], 'lf')"
                          :loading="simulatingMatchId === losersRounds[3].matches[0].id"
                        >
                          <el-icon><VideoPlay /></el-icon>
                          模拟
                        </el-button>
                        <el-button
                          v-if="losersRounds[3].matches[0].status === 'completed'"
                          type="primary"
                          size="small"
                          text
                          @click="viewMatchDetail(losersRounds[3].matches[0])"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 总决赛 -->
          <div class="bracket-section finals-section">
            <div class="section-label finals">总决赛</div>

            <div class="grand-final-area">
              <div class="final-match-card" :class="{ completed: finalMatch.status === 'completed' }">
                <div class="final-team" :class="{ champion: finalMatch.winnerId === finalMatch.teamAId }">
                  <div class="team-label">胜者组冠军</div>
                  <div class="team-name">{{ finalMatch.teamA || '待定' }}</div>
                  <div class="team-score" v-if="finalMatch.status === 'completed'">{{ finalMatch.scoreA }}</div>
                </div>

                <div class="vs-badge">
                  <span>VS</span>
                  <small>BO5</small>
                </div>

                <div class="final-team" :class="{ champion: finalMatch.winnerId === finalMatch.teamBId }">
                  <div class="team-label">败者组冠军</div>
                  <div class="team-name">{{ finalMatch.teamB || '待定' }}</div>
                  <div class="team-score" v-if="finalMatch.status === 'completed'">{{ finalMatch.scoreB }}</div>
                </div>

                <div class="final-actions">
                  <el-button
                    v-if="finalMatch.status !== 'completed' && canSimulate(finalMatch)"
                    type="success"
                    size="large"
                    @click="simulateSingleMatch(finalMatch, 'gf')"
                    :loading="simulatingMatchId === finalMatch.id"
                  >
                    <el-icon><VideoPlay /></el-icon>
                    模拟总决赛
                  </el-button>
                  <el-button
                    v-if="finalMatch.status === 'completed'"
                    type="info"
                    size="large"
                    @click="viewMatchDetail(finalMatch)"
                  >
                    查看详情
                  </el-button>
                </div>
              </div>

              <!-- 冠军展示 -->
              <div v-if="champion" class="champion-display">
                <el-icon :size="32" color="#f59e0b"><Trophy /></el-icon>
                <div class="champion-info">
                  <div class="champion-label">冠军</div>
                  <div class="champion-name">{{ champion.name }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <TournamentCompletionSection
        v-if="playoffsCompleted"
        :standings="summerStandings"
        subtitle="年度积分已发放"
        banner-title="夏季季后赛已完成！"
        :banner-champion="champion?.name || ''"
        :banner-description="`获得 ${getRegionName(selectedRegion)} 夏季赛冠军！`"
      />

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
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  DArrowRight,
  Trophy,
  VideoPlay,
} from '@element-plus/icons-vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import TournamentCompletionSection from '@/components/common/TournamentCompletionSection.vue'
import type { StandingItem } from '@/types/tournament'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, teamApi, tournamentApi, matchApi, financeApi, type Team, type TournamentMatch, type DetailedGameResult, type PlayerGameStats } from '@/api/tauri'
import type { PlayerPosition, TraitType } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import { createLogger } from '@/utils/logger'

const logger = createLogger('SummerPlayoffsDetail')

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
const selectedRegion = ref(1)
const playoffsSimulating = ref(false)
const playoffsProgress = ref(0)
const simulatingMatchId = ref<string | null>(null)
const loading = ref(false)

// 常规赛完成状态
const regularSeasonCompleted = ref(false)

// 当前赛事ID
const currentTournamentId = ref<number | null>(null)

// 赛区数据
const regions = ref<{ id: number; name: string }[]>([])

// 队伍ID到名称的映射
const teamMap = ref<Map<number, Team>>(new Map())

// 积分榜数据
const standings = ref<any[]>([])

// 季后赛比赛数据（从数据库加载）
const playoffsMatches = ref<TournamentMatch[]>([])

// 季后赛数据
const playoffsCompleted = ref(false)
const champion = ref<{ name: string } | null>(null)
const runnerUp = ref<{ name: string } | null>(null)
const thirdPlace = ref<{ name: string } | null>(null)
const fourthPlace = ref<{ name: string } | null>(null)

// 胜者组轮次
const winnersRounds = ref([
  {
    name: '胜者组第一轮',
    matches: [
      { id: 'w1', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: 1 as number | null, teamBId: null as number | null, teamB: '', seedB: 4 as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
      { id: 'w2', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: 2 as number | null, teamBId: null as number | null, teamB: '', seedB: 3 as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
    ]
  },
  {
    name: '胜者组决赛',
    matches: [
      { id: 'w3', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: null as number | null, teamBId: null as number | null, teamB: '', seedB: null as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
    ]
  }
])

// 败者组轮次
const losersRounds = ref([
  {
    name: '败者组第一轮',
    matches: [
      { id: 'l1', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: 5 as number | null, teamBId: null as number | null, teamB: '', seedB: 8 as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
      { id: 'l2', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: 6 as number | null, teamBId: null as number | null, teamB: '', seedB: 7 as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
    ]
  },
  {
    name: '败者组第二轮',
    matches: [
      { id: 'l3', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: null as number | null, teamBId: null as number | null, teamB: '', seedB: null as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
      { id: 'l4', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: null as number | null, teamBId: null as number | null, teamB: '', seedB: null as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
    ]
  },
  {
    name: '败者组第三轮',
    matches: [
      { id: 'l5', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: null as number | null, teamBId: null as number | null, teamB: '', seedB: null as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
    ]
  },
  {
    name: '败者组决赛',
    matches: [
      { id: 'l6', dbMatchId: null as number | null, teamAId: null as number | null, teamA: '', seedA: null as number | null, teamBId: null as number | null, teamB: '', seedB: null as number | null, scoreA: 0, scoreB: 0, winnerId: null as number | null, status: 'upcoming', matchDetailKey: '' },
    ]
  }
])

// 总决赛
const finalMatch = ref({
  id: 'gf',
  matchDetailKey: 'gf',
  dbMatchId: null as number | null,
  teamAId: null as number | null,
  teamA: '',
  seedA: null as number | null,
  teamBId: null as number | null,
  teamB: '',
  seedB: null as number | null,
  scoreA: 0,
  scoreB: 0,
  winnerId: null as number | null,
  status: 'upcoming'
})

// 加载函数
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

const loadTeams = async (regionId: number) => {
  try {
    const teams = await teamApi.getTeamsByRegion(regionId)
    teamMap.value.clear()
    teams.forEach(team => teamMap.value.set(team.id, team))
  } catch (error) {
    logger.error('Failed to load teams:', error)
  }
}

const loadTournament = async (regionId: number) => {
  try {
    const seasonId = viewingSeason.value
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    const summerPlayoffs = tournaments.find(t => t.tournament_type === 'SummerPlayoffs')
    if (summerPlayoffs) {
      currentTournamentId.value = summerPlayoffs.id
    }
  } catch (error) {
    logger.error('Failed to load tournament:', error)
  }
}

const checkRegularSeasonStatus = async (regionId: number) => {
  try {
    const seasonId = viewingSeason.value
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    const summerRegular = tournaments.find(t => t.tournament_type === 'SummerRegular')

    if (summerRegular) {
      // 检查常规赛是否完成 - 获取赛程并检查是否所有比赛都已完成
      const matches = await tournamentApi.getTournamentMatches(summerRegular.id)
      const allMatchesCompleted = matches.every((match: TournamentMatch) => match.status === 'Completed' || match.status === 'COMPLETED')
      regularSeasonCompleted.value = allMatchesCompleted
    } else {
      regularSeasonCompleted.value = false
    }
  } catch (error) {
    logger.error('Failed to check regular season status:', error)
    regularSeasonCompleted.value = false
  }
}

const loadStandings = async (regionId: number) => {
  try {
    const seasonId = viewingSeason.value
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    const summerRegular = tournaments.find(t => t.tournament_type === 'SummerRegular')
    if (summerRegular) {
      const standingList = await tournamentApi.getStandings(summerRegular.id)
      standings.value = standingList.map(s => {
        const team = teamMap.value.get(s.team_id)
        return {
          id: s.team_id,
          name: team?.name || s.team_name,
          short: team?.short_name || s.team_name,
          wins: s.wins,
          losses: s.losses,
          points: s.points,
        }
      })

      // 只有常规赛完成时才更新季后赛数据
      if (regularSeasonCompleted.value) {
        await updatePlayoffsData()
      }
    }
  } catch (error) {
    logger.error('Failed to load standings:', error)
  }
}

const loadRegionData = async (regionId: number) => {
  loading.value = true
  try {
    await loadTeams(regionId)
    await loadTournament(regionId)
    await checkRegularSeasonStatus(regionId)
    await loadStandings(regionId)
  } finally {
    loading.value = false
  }
}

const updatePlayoffsData = async () => {
  if (standings.value.length < 8) return
  if (!regularSeasonCompleted.value) return
  if (!currentTournamentId.value) return

  const sortedTeams = [...standings.value].sort((a, b) => b.points - a.points)

  playoffsCompleted.value = false
  champion.value = null
  runnerUp.value = null
  thirdPlace.value = null
  fourthPlace.value = null

  // 从数据库加载季后赛比赛
  try {
    playoffsMatches.value = await tournamentApi.getTournamentMatches(currentTournamentId.value)
  } catch (error) {
    logger.error('Failed to load playoffs matches:', error)
    playoffsMatches.value = []
  }

  // 辅助函数：根据阶段和顺序查找数据库比赛
  const findDbMatch = (stage: string, matchOrder?: number): TournamentMatch | undefined => {
    return playoffsMatches.value.find(m =>
      m.stage === stage && (matchOrder === undefined || m.match_order === matchOrder)
    )
  }

  // 辅助函数：获取队伍短名
  const getShortName = (teamId: number | null | undefined): string => {
    if (!teamId) return '待定'
    const team = teamMap.value.get(teamId)
    return team?.short_name || sortedTeams.find(t => t.id === teamId)?.short || '待定'
  }

  // 初始化胜者组第一轮 (排名 1v4, 2v3)
  const w1Match = findDbMatch('WINNERS_R1', 1)
  const w2Match = findDbMatch('WINNERS_R1', 2)

  winnersRounds.value[0].matches = [
    {
      id: 'w1',
      matchDetailKey: 'w1-1',
      dbMatchId: w1Match?.id || null,
      teamAId: w1Match?.home_team_id || sortedTeams[0].id,
      teamA: getShortName(w1Match?.home_team_id) || sortedTeams[0].short,
      seedA: 1,
      teamBId: w1Match?.away_team_id || sortedTeams[3].id,
      teamB: getShortName(w1Match?.away_team_id) || sortedTeams[3].short,
      seedB: 4,
      scoreA: w1Match?.home_score || 0,
      scoreB: w1Match?.away_score || 0,
      winnerId: w1Match?.winner_id || null,
      status: (w1Match?.status === 'Completed' || w1Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    },
    {
      id: 'w2',
      matchDetailKey: 'w1-2',
      dbMatchId: w2Match?.id || null,
      teamAId: w2Match?.home_team_id || sortedTeams[1].id,
      teamA: getShortName(w2Match?.home_team_id) || sortedTeams[1].short,
      seedA: 2,
      teamBId: w2Match?.away_team_id || sortedTeams[2].id,
      teamB: getShortName(w2Match?.away_team_id) || sortedTeams[2].short,
      seedB: 3,
      scoreA: w2Match?.home_score || 0,
      scoreB: w2Match?.away_score || 0,
      winnerId: w2Match?.winner_id || null,
      status: (w2Match?.status === 'Completed' || w2Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    }
  ]

  // 胜者组决赛
  const wfMatch = findDbMatch('WINNERS_FINAL')
  winnersRounds.value[1].matches = [
    {
      id: 'w3',
      matchDetailKey: 'wf',
      dbMatchId: wfMatch?.id || null,
      teamAId: wfMatch?.home_team_id || null,
      teamA: getShortName(wfMatch?.home_team_id),
      seedA: null,
      teamBId: wfMatch?.away_team_id || null,
      teamB: getShortName(wfMatch?.away_team_id),
      seedB: null,
      scoreA: wfMatch?.home_score || 0,
      scoreB: wfMatch?.away_score || 0,
      winnerId: wfMatch?.winner_id || null,
      status: (wfMatch?.status === 'Completed' || wfMatch?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    }
  ]

  // 败者组第一轮 (排名 5v8, 6v7)
  const l1Match = findDbMatch('LOSERS_R1', 1)
  const l2Match = findDbMatch('LOSERS_R1', 2)
  losersRounds.value[0].matches = [
    {
      id: 'l1',
      matchDetailKey: 'l1-1',
      dbMatchId: l1Match?.id || null,
      teamAId: l1Match?.home_team_id || sortedTeams[4].id,
      teamA: getShortName(l1Match?.home_team_id) || sortedTeams[4].short,
      seedA: 5,
      teamBId: l1Match?.away_team_id || sortedTeams[7]?.id || null,
      teamB: getShortName(l1Match?.away_team_id) || sortedTeams[7]?.short || '待定',
      seedB: 8,
      scoreA: l1Match?.home_score || 0,
      scoreB: l1Match?.away_score || 0,
      winnerId: l1Match?.winner_id || null,
      status: (l1Match?.status === 'Completed' || l1Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    },
    {
      id: 'l2',
      matchDetailKey: 'l1-2',
      dbMatchId: l2Match?.id || null,
      teamAId: l2Match?.home_team_id || sortedTeams[5].id,
      teamA: getShortName(l2Match?.home_team_id) || sortedTeams[5].short,
      seedA: 6,
      teamBId: l2Match?.away_team_id || sortedTeams[6]?.id || null,
      teamB: getShortName(l2Match?.away_team_id) || sortedTeams[6]?.short || '待定',
      seedB: 7,
      scoreA: l2Match?.home_score || 0,
      scoreB: l2Match?.away_score || 0,
      winnerId: l2Match?.winner_id || null,
      status: (l2Match?.status === 'Completed' || l2Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    }
  ]

  // 败者组第二轮
  const l3Match = findDbMatch('LOSERS_R2', 1)
  const l4Match = findDbMatch('LOSERS_R2', 2)
  losersRounds.value[1].matches = [
    {
      id: 'l3',
      matchDetailKey: 'l2-1',
      dbMatchId: l3Match?.id || null,
      teamAId: l3Match?.home_team_id || null,
      teamA: getShortName(l3Match?.home_team_id),
      seedA: null,
      teamBId: l3Match?.away_team_id || null,
      teamB: getShortName(l3Match?.away_team_id),
      seedB: null,
      scoreA: l3Match?.home_score || 0,
      scoreB: l3Match?.away_score || 0,
      winnerId: l3Match?.winner_id || null,
      status: (l3Match?.status === 'Completed' || l3Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    },
    {
      id: 'l4',
      matchDetailKey: 'l2-2',
      dbMatchId: l4Match?.id || null,
      teamAId: l4Match?.home_team_id || null,
      teamA: getShortName(l4Match?.home_team_id),
      seedA: null,
      teamBId: l4Match?.away_team_id || null,
      teamB: getShortName(l4Match?.away_team_id),
      seedB: null,
      scoreA: l4Match?.home_score || 0,
      scoreB: l4Match?.away_score || 0,
      winnerId: l4Match?.winner_id || null,
      status: (l4Match?.status === 'Completed' || l4Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    }
  ]

  // 败者组第三轮
  const l5Match = findDbMatch('LOSERS_R3')
  losersRounds.value[2].matches = [
    {
      id: 'l5',
      matchDetailKey: 'l3',
      dbMatchId: l5Match?.id || null,
      teamAId: l5Match?.home_team_id || null,
      teamA: getShortName(l5Match?.home_team_id),
      seedA: null,
      teamBId: l5Match?.away_team_id || null,
      teamB: getShortName(l5Match?.away_team_id),
      seedB: null,
      scoreA: l5Match?.home_score || 0,
      scoreB: l5Match?.away_score || 0,
      winnerId: l5Match?.winner_id || null,
      status: (l5Match?.status === 'Completed' || l5Match?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    }
  ]

  // 败者组决赛
  const lfMatch = findDbMatch('LOSERS_FINAL')
  losersRounds.value[3].matches = [
    {
      id: 'l6',
      matchDetailKey: 'lf',
      dbMatchId: lfMatch?.id || null,
      teamAId: lfMatch?.home_team_id || null,
      teamA: getShortName(lfMatch?.home_team_id),
      seedA: null,
      teamBId: lfMatch?.away_team_id || null,
      teamB: getShortName(lfMatch?.away_team_id),
      seedB: null,
      scoreA: lfMatch?.home_score || 0,
      scoreB: lfMatch?.away_score || 0,
      winnerId: lfMatch?.winner_id || null,
      status: (lfMatch?.status === 'Completed' || lfMatch?.status === 'COMPLETED') ? 'completed' : 'upcoming'
    }
  ]

  // 总决赛
  const gfMatch = findDbMatch('GRAND_FINAL')
  finalMatch.value = {
    id: 'gf',
    matchDetailKey: 'gf',
    dbMatchId: gfMatch?.id || null,
    teamAId: gfMatch?.home_team_id || null,
    teamA: getShortName(gfMatch?.home_team_id),
    seedA: null,
    teamBId: gfMatch?.away_team_id || null,
    teamB: getShortName(gfMatch?.away_team_id),
    seedB: null,
    scoreA: gfMatch?.home_score || 0,
    scoreB: gfMatch?.away_score || 0,
    winnerId: gfMatch?.winner_id || null,
    status: (gfMatch?.status === 'Completed' || gfMatch?.status === 'COMPLETED') ? 'completed' : 'upcoming'
  }

  // 检查是否完成
  if (finalMatch.value.status === 'completed' && finalMatch.value.winnerId) {
    playoffsCompleted.value = true
    const winnerName = getTeamName(finalMatch.value.winnerId)
    const loserName = finalMatch.value.winnerId === finalMatch.value.teamAId
      ? getTeamName(finalMatch.value.teamBId)
      : getTeamName(finalMatch.value.teamAId)
    champion.value = { name: winnerName }
    runnerUp.value = { name: loserName }
  }
}

// 方法
const goBack = () => router.push('/tournaments')
const getRegionName = (regionId: number) => regions.value.find(r => r.id === regionId)?.name || 'LPL'

const summerStandings = computed<StandingItem[]>(() => [
  { rank: 1, label: '冠军', name: champion.value?.name || '', points: '+12 分' },
  { rank: 2, label: '亚军', name: runnerUp.value?.name || '', points: '+10 分' },
  { rank: 3, label: '季军', name: thirdPlace.value?.name || '', points: '+8 分' },
  { rank: 4, label: '殿军', name: fourthPlace.value?.name || '', points: '+6 分' },
])

const handleRegionChange = async (regionId: number) => {
  playoffsCompleted.value = false
  champion.value = null
  runnerUp.value = null
  thirdPlace.value = null
  fourthPlace.value = null
  await loadRegionData(regionId)
  ElMessage.success(`已切换到 ${getRegionName(regionId)} 赛区`)
}

// 检查比赛是否可以模拟
const canSimulate = (match: any) => {
  return regularSeasonCompleted.value && match.teamAId && match.teamBId
}

const viewMatchDetail = async (match: any) => {
  if (match.status !== 'completed') return

  // 首先尝试从本地缓存获取
  const key = match.dbMatchId || match.matchDetailKey || match.id
  let detail = matchDetailStore.getMatchDetail(`summer-playoffs-${key}`)

  // 如果本地没有，尝试从数据库加载
  if (!detail && match.dbMatchId) {
    logger.debug(`本地未找到详情，尝试从数据库加载: dbMatchId=${match.dbMatchId}`)
    detail = await matchDetailStore.loadMatchDetailFromDb(match.dbMatchId)
  }

  if (detail) {
    // 后处理：修正空的或不正确的队名
    const fixedDetail = { ...detail }

    // 修正比赛级别的队名
    if (!fixedDetail.teamAName || fixedDetail.teamAName === '待定' || fixedDetail.teamAName === '') {
      const teamAId = parseInt(fixedDetail.teamAId)
      if (!isNaN(teamAId)) {
        fixedDetail.teamAName = getTeamName(teamAId)
      }
    }
    if (!fixedDetail.teamBName || fixedDetail.teamBName === '待定' || fixedDetail.teamBName === '') {
      const teamBId = parseInt(fixedDetail.teamBId)
      if (!isNaN(teamBId)) {
        fixedDetail.teamBName = getTeamName(teamBId)
      }
    }

    // 修正胜者名称
    if (fixedDetail.winnerId) {
      const winnerId = parseInt(fixedDetail.winnerId)
      if (!isNaN(winnerId)) {
        fixedDetail.winnerName = getTeamName(winnerId)
      }
    }

    // 修正每局比赛的队名
    if (fixedDetail.games) {
      fixedDetail.games = fixedDetail.games.map(game => {
        const fixedGame = { ...game }
        if (!fixedGame.teamAName || fixedGame.teamAName === '待定' || fixedGame.teamAName === '') {
          const teamAId = parseInt(fixedGame.teamAId)
          if (!isNaN(teamAId)) {
            fixedGame.teamAName = getTeamName(teamAId)
          }
        }
        if (!fixedGame.teamBName || fixedGame.teamBName === '待定' || fixedGame.teamBName === '') {
          const teamBId = parseInt(fixedGame.teamBId)
          if (!isNaN(teamBId)) {
            fixedGame.teamBName = getTeamName(teamBId)
          }
        }
        if (fixedGame.winnerId) {
          const winnerId = parseInt(fixedGame.winnerId)
          if (!isNaN(winnerId)) {
            fixedGame.winnerName = getTeamName(winnerId)
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

    currentMatchDetail.value = fixedDetail
    showMatchDetailDialog.value = true
  } else {
    ElMessage.warning('比赛详情未找到，可能需要重新模拟')
  }
}

const handleCloseMatchDetail = () => {
  showMatchDetailDialog.value = false
  currentMatchDetail.value = null
}

const getTeamName = (teamId: number | null) => {
  if (!teamId) return '待定'
  // 首先从 standings 查找（常规赛积分榜）
  const team = standings.value.find(t => t.id === teamId)
  if (team?.short) return team.short
  if (team?.name) return team.name
  // 然后从 teamMap 查找（所有队伍）
  const teamFromMap = teamMap.value.get(teamId)
  if (teamFromMap?.short_name) return teamFromMap.short_name
  if (teamFromMap?.name) return teamFromMap.name
  return '待定'
}

// 转换选手表现数据格式
const convertPlayerPerformance = (p: PlayerGameStats, teamId: string) => ({
  playerId: String(p.player_id),
  playerName: p.player_name,
  position: p.position as PlayerPosition,
  teamId: teamId,
  baseAbility: p.base_ability,
  conditionBonus: p.condition_bonus,
  stabilityNoise: p.stability_noise,
  actualAbility: p.actual_ability,
  impactScore: p.impact_score,
  traits: p.traits as any[],
  activatedTraits: p.activated_traits?.map(t => ({
    type: t.trait_type as TraitType,
    name: t.name,
    effect: t.effect,
    value: t.value,
    isPositive: t.is_positive
  }))
})

// 模拟单场比赛的核心函数 - 使用后端API
const doSimulateMatch = async (match: any, dbMatchId: number, _matchIdPrefix: string): Promise<number> => {
  const regionName = getRegionName(selectedRegion.value)

  // 调用后端API模拟比赛
  const result = await matchApi.simulateMatchDetailed(dbMatchId)

  // 更新比赛状态
  match.scoreA = result.home_score
  match.scoreB = result.away_score
  match.winnerId = result.winner_id
  match.status = 'completed'

  // 使用后端返回的队伍ID查找正确的队伍名称（更可靠）
  const teamAName = getTeamName(result.home_team_id) || match.teamA || '队伍A'
  const teamBName = getTeamName(result.away_team_id) || match.teamB || '队伍B'

  // 构建游戏详情
  const games = result.games.map((game: DetailedGameResult) => {
    // 计算队伍战力（选手实际发挥能力平均值）- 每局不同
    const teamAPower = game.home_players.length > 0
      ? game.home_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.home_players.length
      : 0
    const teamBPower = game.away_players.length > 0
      ? game.away_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.away_players.length
      : 0
    const powerDifference = teamAPower - teamBPower

    return {
      gameNumber: game.game_number,
      teamAId: String(result.home_team_id),
      teamAName: teamAName,
      teamAPower,
      teamAPerformance: game.home_performance,
      teamAMetaPower: game.home_performance,
      teamAPlayers: game.home_players.map(p => convertPlayerPerformance(p, String(result.home_team_id))),
      teamBId: String(result.away_team_id),
      teamBName: teamBName,
      teamBPower,
      teamBPerformance: game.away_performance,
      teamBMetaPower: game.away_performance,
      teamBPlayers: game.away_players.map(p => convertPlayerPerformance(p, String(result.away_team_id))),
      winnerId: String(game.winner_id),
      winnerName: game.winner_id === result.home_team_id ? teamAName : teamBName,
      powerDifference,
      performanceDifference: game.home_performance - game.away_performance,
      metaPowerDifference: game.home_performance - game.away_performance,
      isUpset: powerDifference > 0 && game.winner_id !== result.home_team_id ||
               powerDifference < 0 && game.winner_id === result.home_team_id,
      teamABasePower: game.home_base_power ?? undefined,
      teamBBasePower: game.away_base_power ?? undefined,
      teamASynergyBonus: game.home_synergy_bonus ?? undefined,
      teamBSynergyBonus: game.away_synergy_bonus ?? undefined,
      teamABpBonus: game.home_bp_bonus ?? undefined,
      teamBBpBonus: game.away_bp_bonus ?? undefined,
      teamAVersionBonus: game.home_version_bonus ?? undefined,
      teamBVersionBonus: game.away_version_bonus ?? undefined,
      duration: game.duration_minutes,
      mvp: game.game_mvp ? {
        playerId: String(game.game_mvp.player_id),
        playerName: game.game_mvp.player_name,
        teamId: String(game.game_mvp.team_id),
        position: game.game_mvp.position as PlayerPosition,
        mvpScore: game.game_mvp.mvp_score
      } : undefined
    }
  })

  // 计算 MVP（仅从胜者队伍中选择）
  let mvpPlayerId: string | undefined = result.match_mvp ? String(result.match_mvp.player_id) : undefined
  let mvpPlayerName: string | undefined = result.match_mvp?.player_name
  let mvpTeamId: string | undefined = result.match_mvp ? String(result.match_mvp.team_id) : undefined
  let mvpTotalImpact: number | undefined = result.match_mvp?.mvp_score

  // 如果后端没有返回 MVP，则在前端计算（只从胜者队伍中选择）
  if (!mvpPlayerId) {
    const winnerId = String(result.winner_id)
    const isHomeWinner = result.winner_id === result.home_team_id

    // 收集胜方队伍所有选手的累计影响力
    const winnerPlayerStats = new Map<string, { name: string, totalImpact: number, gameCount: number }>()

    games.forEach(game => {
      // 只收集胜方队伍的选手数据
      const winnerPlayers = isHomeWinner ? game.teamAPlayers : game.teamBPlayers

      winnerPlayers.forEach(p => {
        const existing = winnerPlayerStats.get(p.playerId)
        if (existing) {
          existing.totalImpact += p.impactScore
          existing.gameCount++
        } else {
          winnerPlayerStats.set(p.playerId, {
            name: p.playerName,
            totalImpact: p.impactScore,
            gameCount: 1
          })
        }
      })
    })

    // 找出平均影响力最高的选手作为 MVP
    let maxAvgImpact = -Infinity
    winnerPlayerStats.forEach((stats, playerId) => {
      const avgImpact = stats.totalImpact / stats.gameCount
      if (avgImpact > maxAvgImpact) {
        maxAvgImpact = avgImpact
        mvpPlayerId = playerId
        mvpPlayerName = stats.name
        mvpTeamId = winnerId
        mvpTotalImpact = stats.totalImpact
      }
    })
  }

  // 构建 MatchDetail 用于展示
  const matchDetail: MatchDetail = {
    matchId: `summer-playoffs-${dbMatchId}`,
    tournamentType: 'summer-playoffs',
    seasonId: String(gameStore.currentSeason),
    teamAId: String(result.home_team_id),
    teamAName: teamAName,
    teamBId: String(result.away_team_id),
    teamBName: teamBName,
    bestOf: 5,
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    winnerName: result.winner_id === result.home_team_id ? teamAName : teamBName,
    mvpPlayerId,
    mvpPlayerName,
    mvpTeamId,
    mvpTotalImpact,
    games
  }

  // 保存比赛详情到 Store（用本地 key）
  await matchDetailStore.saveMatchDetail(matchDetail.matchId, matchDetail)

  // 同时用数据库 ID 保存一份，确保能从数据库加载
  const dbMatchDetail = { ...matchDetail, matchId: dbMatchId }
  await matchDetailStore.saveMatchDetail(dbMatchId, dbMatchDetail)

  // 记录选手表现到统计
  matchDetail.games.forEach(game => {
    game.teamAPlayers.forEach(perf => {
      playerStore.recordPerformance(perf.playerId, perf.playerName, perf.teamId, perf.position, perf.impactScore, perf.actualAbility, String(gameStore.currentSeason), regionName)
    })
    game.teamBPlayers.forEach(perf => {
      playerStore.recordPerformance(perf.playerId, perf.playerName, perf.teamId, perf.position, perf.impactScore, perf.actualAbility, String(gameStore.currentSeason), regionName)
    })
  })
  playerStore.saveToStorage()

  return result.winner_id
}

// 更新后续比赛的队伍
const updateBracketAfterMatch = async (matchId: string, winnerId: number, loserId: number) => {
  const winnerName = getTeamName(winnerId)
  const loserName = getTeamName(loserId)

  // 胜者组第一轮 -> 胜者组决赛 + 败者组第二轮
  if (matchId === 'w1') {
    winnersRounds.value[1].matches[0].teamAId = winnerId
    winnersRounds.value[1].matches[0].teamA = winnerName
    losersRounds.value[1].matches[0].teamBId = loserId
    losersRounds.value[1].matches[0].teamB = loserName
  } else if (matchId === 'w2') {
    winnersRounds.value[1].matches[0].teamBId = winnerId
    winnersRounds.value[1].matches[0].teamB = winnerName
    losersRounds.value[1].matches[1].teamBId = loserId
    losersRounds.value[1].matches[1].teamB = loserName
  }
  // 胜者组决赛 -> 总决赛 + 败者组决赛
  else if (matchId === 'w3') {
    finalMatch.value.teamAId = winnerId
    finalMatch.value.teamA = winnerName
    losersRounds.value[3].matches[0].teamBId = loserId
    losersRounds.value[3].matches[0].teamB = loserName
  }
  // 败者组第一轮 -> 败者组第二轮
  else if (matchId === 'l1') {
    losersRounds.value[1].matches[0].teamAId = winnerId
    losersRounds.value[1].matches[0].teamA = winnerName
  } else if (matchId === 'l2') {
    losersRounds.value[1].matches[1].teamAId = winnerId
    losersRounds.value[1].matches[1].teamA = winnerName
  }
  // 败者组第二轮 -> 败者组第三轮
  else if (matchId === 'l3') {
    losersRounds.value[2].matches[0].teamAId = winnerId
    losersRounds.value[2].matches[0].teamA = winnerName
  } else if (matchId === 'l4') {
    losersRounds.value[2].matches[0].teamBId = winnerId
    losersRounds.value[2].matches[0].teamB = winnerName
  }
  // 败者组第三轮 -> 败者组决赛
  else if (matchId === 'l5') {
    losersRounds.value[3].matches[0].teamAId = winnerId
    losersRounds.value[3].matches[0].teamA = winnerName
    fourthPlace.value = { name: loserName }
  }
  // 败者组决赛 -> 总决赛
  else if (matchId === 'l6') {
    finalMatch.value.teamBId = winnerId
    finalMatch.value.teamB = winnerName
    thirdPlace.value = { name: loserName }
  }
  // 总决赛
  else if (matchId === 'gf' || matchId === 'final') {
    champion.value = { name: winnerName }
    runnerUp.value = { name: loserName }
    playoffsCompleted.value = true

    // 发放赛事奖金
    if (currentTournamentId.value) {
      try {
        await financeApi.distributeTournamentPrizes(currentTournamentId.value)
        logger.debug('季后赛奖金已发放')
      } catch (e) {
        logger.error('发放奖金失败:', e)
      }
    }

    ElMessageBox.alert(
      `恭喜 ${winnerName} 获得 ${getRegionName(selectedRegion.value)} 夏季赛冠军！\n\n` +
      `✅ 奖金已发放到各战队账户\n` +
      `💡 请在时间控制面板完成阶段推进，系统将自动颁发荣誉和年度积分`,
      '🏆 冠军诞生！',
      { confirmButtonText: '太棒了！', center: true }
    )
  }
}

// 单场模拟
const simulateSingleMatch = async (match: any, matchIdPrefix: string) => {
  if (!canSimulate(match)) {
    ElMessage.warning('该比赛的队伍尚未确定或常规赛未完成')
    return
  }

  if (!match.dbMatchId) {
    ElMessage.error('数据库比赛ID不存在，无法模拟')
    return
  }

  simulatingMatchId.value = match.id

  try {
    const winnerId = await doSimulateMatch(match, match.dbMatchId, matchIdPrefix)
    const loserId = winnerId === match.teamAId ? match.teamBId : match.teamAId

    await updateBracketAfterMatch(match.id, winnerId!, loserId!)
    await updatePlayoffsData()

    ElMessage.success(`比赛完成: ${match.teamA} ${match.scoreA} - ${match.scoreB} ${match.teamB}`)
  } catch (error) {
    logger.error('模拟比赛失败:', error)
    ElMessage.error('模拟比赛失败')
  } finally {
    simulatingMatchId.value = null
  }
}

// 从最新响应式数据中按阶段获取待模拟比赛
const getPhaseMatches = (phase: string): { match: any, id: string }[] => {
  switch (phase) {
    case 'first':
      return [
        { match: winnersRounds.value[0]?.matches[0], id: 'w1-1' },
        { match: winnersRounds.value[0]?.matches[1], id: 'w1-2' },
        { match: losersRounds.value[0]?.matches[0], id: 'l1-1' },
        { match: losersRounds.value[0]?.matches[1], id: 'l1-2' },
      ]
    case 'second':
      return [
        { match: winnersRounds.value[1]?.matches[0], id: 'wf' },
        ...(losersRounds.value[1]?.matches || []).map((m: any, i: number) => ({ match: m, id: `l2-${i+1}` })),
      ]
    case 'third':
      return [{ match: losersRounds.value[2]?.matches[0], id: 'l3' }]
    case 'fourth':
      return [{ match: losersRounds.value[3]?.matches[0], id: 'lf' }]
    case 'final':
      return [{ match: finalMatch.value, id: 'gf' }]
    default:
      return []
  }
}

// 一键模拟全部
const simulatePlayoffs = async () => {
  if (!regularSeasonCompleted.value) {
    ElMessage.warning('常规赛尚未完成，无法开始季后赛')
    return
  }

  await ElMessageBox.confirm('将自动模拟整个季后赛，是否继续？', '模拟季后赛', {
    confirmButtonText: '开始',
    cancelButtonText: '取消',
    type: 'warning'
  })

  playoffsSimulating.value = true
  playoffsProgress.value = 0

  const phases = ['first', 'second', 'third', 'fourth', 'final']
  const totalMatches = getPhaseMatches('first').length + getPhaseMatches('second').length + 1 + 1 + 1
  let completed = 0

  // 按阶段逐步模拟，每阶段从最新响应式数据读取比赛
  for (const phase of phases) {
    const matches = getPhaseMatches(phase)
    for (const { match, id } of matches) {
      if (match && match.status !== 'completed' && canSimulate(match)) {
        await simulateSingleMatch(match, id)
        completed++
        playoffsProgress.value = Math.floor((completed / totalMatches) * 100)
        await new Promise(resolve => setTimeout(resolve, 200))
      }
    }
  }

  playoffsProgress.value = 100
  playoffsSimulating.value = false
}

onMounted(async () => {
  matchDetailStore.loadFromStorage()
  await loadRegions()
  if (selectedRegion.value) {
    await loadRegionData(selectedRegion.value)
  }
})
</script>

<style scoped>
.summer-playoffs-management {
  padding: 0;
}

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

.stat-label {
  font-size: 11px;
  color: #94a3b8;
}

.stat-divider {
  width: 1px;
  height: 28px;
  background: #e2e8f0;
}

.table-section {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 16px;
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

.full-bracket {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 16px;
}

.bracket-section {
  border-radius: 10px;
  padding: 16px;
  border: 1px solid #e2e8f0;
}

.bracket-section.winners-section {
  border-left: 3px solid #22c55e;
}

.bracket-section.losers-section {
  border-left: 3px solid #f59e0b;
}

.bracket-section.finals-section {
  border-left: 3px solid #6366f1;
}

.section-label {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f1f5f9;
}

.section-label.winners { color: #16a34a; }
.section-label.losers { color: #d97706; }
.section-label.finals { color: #6366f1; }

.bracket-container {
  display: flex;
  align-items: stretch;
  gap: 0;
  overflow-x: auto;
}

.bracket-round {
  min-width: 210px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.round-header {
  text-align: center;
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  margin-bottom: 12px;
  padding: 4px 10px;
  background: #f8fafc;
  border-radius: 4px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.drop-hint {
  font-size: 10px;
  color: #94a3b8;
  font-weight: 400;
  margin-left: 4px;
}

.matches-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
  flex: 1;
  justify-content: space-around;
  min-height: 200px;
}

.matches-column.final {
  justify-content: center;
  min-height: 100px;
}

.match-card-wrapper {
  display: flex;
  align-items: center;
}

.match-card {
  background: #ffffff;
  border-radius: 8px;
  padding: 10px 12px;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #e2e8f0;
  min-width: 195px;
  transition: all 0.15s;
}

.match-card:hover {
  border-color: #cbd5e1;
}

.match-card.winners { border-left-color: #22c55e; }
.match-card.losers { border-left-color: #f59e0b; }
.match-card.completed { border-left-color: #6366f1; }
.match-card.final-match { border-left-width: 4px; }

.drop-in-indicator {
  font-size: 10px;
  color: #94a3b8;
  margin-bottom: 4px;
}

.match-card .match-teams .match-team {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 4px;
}

.match-card .match-teams .match-team.winner {
  background: #f0fdf4;
}

.match-card .match-teams .match-team .seed {
  font-size: 10px;
  color: #94a3b8;
  min-width: 18px;
}

.match-card .match-teams .match-team .name {
  flex: 1;
  font-weight: 500;
  font-size: 13px;
  color: #0f172a;
}

.match-card .match-teams .match-team .score {
  font-size: 15px;
  font-weight: 700;
  min-width: 18px;
  text-align: center;
  color: #0f172a;
}

.match-card .match-actions {
  margin-top: 6px;
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

/* CSS连接线 */
.bracket-connector {
  width: 60px;
  min-width: 60px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.bracket-connector::before {
  content: '';
  height: 44px;
  flex-shrink: 0;
}

.bracket-connector::after {
  content: '';
  flex: 1;
}

.winners-color { --line-color: #22c55e; }
.losers-color { --line-color: #f59e0b; }

.connector-merge::after {
  background:
    linear-gradient(var(--line-color), var(--line-color)) 0 25% / 50% 2px no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) 0 75% / 50% 2px no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) calc(50% - 1px) 50% / 2px 50% no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) 100% 50% / 50% 2px no-repeat;
}

.connector-parallel::after {
  background:
    linear-gradient(var(--line-color), var(--line-color)) 0 25% / 100% 2px no-repeat,
    linear-gradient(var(--line-color), var(--line-color)) 0 75% / 100% 2px no-repeat;
}

.connector-straight::after {
  background:
    linear-gradient(var(--line-color), var(--line-color)) 0 50% / 100% 2px no-repeat;
}

.grand-final-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 16px;
}

.final-match-card {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 32px;
  padding: 24px 36px;
  background: #ffffff;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
}

.final-match-card.completed {
  border-color: #6366f1;
}

.final-team {
  text-align: center;
  padding: 16px 24px;
  background: #f8fafc;
  border-radius: 10px;
  min-width: 120px;
  transition: all 0.2s;
}

.final-team.champion {
  background: #fef3c7;
  border: 1px solid #f59e0b;
}

.final-team .team-label {
  font-size: 11px;
  color: #94a3b8;
  margin-bottom: 4px;
}

.final-team.champion .team-label {
  color: #d97706;
}

.final-team .team-name {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 6px;
}

.final-team.champion .team-name {
  color: #92400e;
}

.final-team .team-score {
  font-size: 32px;
  font-weight: 900;
  color: #0f172a;
}

.vs-badge {
  text-align: center;
}

.vs-badge span {
  display: block;
  font-size: 20px;
  font-weight: 900;
  color: #6366f1;
}

.vs-badge small {
  font-size: 11px;
  color: #94a3b8;
}

.final-actions {
  display: flex;
  align-items: center;
  margin-left: 12px;
}

.champion-display {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 24px;
  background: #fef3c7;
  border: 1px solid #fcd34d;
  border-radius: 10px;
}

.champion-info .champion-label {
  font-size: 12px;
  color: #92400e;
}

.champion-info .champion-name {
  font-size: 20px;
  font-weight: 800;
  color: #78350f;
}
</style>
