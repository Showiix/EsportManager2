<template>
  <div class="spring-playoffs-management">
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
            {{ getRegionName(selectedRegion) }} 春季季后赛
          </h1>
          <p class="page-description">
            常规赛前8名队伍参加，双败淘汰制，胜者组1-4名，败者组5-8名
          </p>
        </div>
      </div>
      <div class="header-actions">
        <el-button
          v-if="regularSeasonCompleted && !playoffsCompleted"
          type="primary"
          size="large"
          @click="simulatePlayoffs"
          :loading="playoffsSimulating"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ playoffsSimulating ? '模拟中...' : '一键模拟季后赛' }}
        </el-button>
      </div>
    </div>

    <!-- 状态卡片 -->
    <div class="playoffs-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>春季季后赛</h2>
          <el-tag :type="playoffsCompleted ? 'success' : regularSeasonCompleted ? 'warning' : 'info'" size="large">
            {{ playoffsCompleted ? '已完成' : regularSeasonCompleted ? '进行中' : '等待常规赛结束' }}
          </el-tag>
        </div>
        <!-- 赛区选择器 -->
        <div class="region-selector">
          <el-radio-group v-model="selectedRegion" @change="handleRegionChange" size="large">
            <el-radio-button v-for="region in regions" :key="region.id" :value="region.id">
              {{ region.name }}
            </el-radio-button>
          </el-radio-group>
        </div>
      </div>

      <!-- 常规赛未完成提示 -->
      <el-alert
        v-if="!regularSeasonCompleted"
        title="常规赛尚未完成"
        description="请先完成春季赛常规赛，季后赛排名将根据常规赛积分自动确定。"
        type="warning"
        :closable="false"
        show-icon
        class="regular-season-alert"
      />

      <!-- 参赛队伍统计 -->
      <div class="teams-stats">
        <el-statistic title="参赛队伍" :value="8" />
        <el-statistic title="胜者组" :value="4" suffix="队" />
        <el-statistic title="败者组" :value="4" suffix="队" />
        <el-statistic title="赛制" value="BO5" />
      </div>

      <!-- 完整对阵图 -->
      <el-card class="bracket-card">
        <template #header>
          <div class="card-header">
            <span class="stage-title">
              <el-icon><Trophy /></el-icon>
              双败淘汰赛对阵图
            </span>
          </div>
        </template>

        <div class="full-bracket">
          <!-- 胜者组 -->
          <div class="bracket-section winners-section">
            <div class="section-header winners">
              <el-icon><Top /></el-icon>
              胜者组 (Winner's Bracket)
            </div>

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
                          type="info"
                          size="small"
                          plain
                          @click="viewMatchDetail(match)"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- SVG连接线: 第一轮 -> 胜者组决赛 -->
              <div class="bracket-connector winners-connector">
                <svg class="connector-svg" viewBox="0 0 100 400" preserveAspectRatio="none">
                  <!-- 上方比赛出线 -->
                  <line x1="0" y1="25%" x2="50" y2="25%" stroke="#22c55e" stroke-width="2" />
                  <line x1="50" y1="25%" x2="50" y2="50%" stroke="#22c55e" stroke-width="2" />
                  <!-- 下方比赛出线 -->
                  <line x1="0" y1="75%" x2="50" y2="75%" stroke="#22c55e" stroke-width="2" />
                  <line x1="50" y1="75%" x2="50" y2="50%" stroke="#22c55e" stroke-width="2" />
                  <!-- 汇合线 -->
                  <line x1="50" y1="50%" x2="100" y2="50%" stroke="#22c55e" stroke-width="2" />
                </svg>
              </div>

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
                          type="info"
                          size="small"
                          plain
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
            <div class="section-header losers">
              <el-icon><Bottom /></el-icon>
              败者组 (Loser's Bracket)
            </div>

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
                          type="info"
                          size="small"
                          plain
                          @click="viewMatchDetail(match)"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- SVG连接线: 败者组第一轮 -> 第二轮 -->
              <div class="bracket-connector losers-connector">
                <svg class="connector-svg" viewBox="0 0 100 400" preserveAspectRatio="none">
                  <line x1="0" y1="25%" x2="100" y2="25%" stroke="#f59e0b" stroke-width="2" />
                  <line x1="0" y1="75%" x2="100" y2="75%" stroke="#f59e0b" stroke-width="2" />
                </svg>
              </div>

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
                          type="info"
                          size="small"
                          plain
                          @click="viewMatchDetail(match)"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- SVG连接线: 败者组第二轮 -> 第三轮 -->
              <div class="bracket-connector losers-connector">
                <svg class="connector-svg" viewBox="0 0 100 400" preserveAspectRatio="none">
                  <!-- 上方比赛出线 -->
                  <line x1="0" y1="25%" x2="50" y2="25%" stroke="#f59e0b" stroke-width="2" />
                  <line x1="50" y1="25%" x2="50" y2="50%" stroke="#f59e0b" stroke-width="2" />
                  <!-- 下方比赛出线 -->
                  <line x1="0" y1="75%" x2="50" y2="75%" stroke="#f59e0b" stroke-width="2" />
                  <line x1="50" y1="75%" x2="50" y2="50%" stroke="#f59e0b" stroke-width="2" />
                  <!-- 汇合线 -->
                  <line x1="50" y1="50%" x2="100" y2="50%" stroke="#f59e0b" stroke-width="2" />
                </svg>
              </div>

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
                          type="info"
                          size="small"
                          plain
                          @click="viewMatchDetail(losersRounds[2].matches[0])"
                        >
                          详情
                        </el-button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- SVG连接线: 败者组第三轮 -> 败者组决赛 -->
              <div class="bracket-connector losers-connector single">
                <svg class="connector-svg" viewBox="0 0 100 400" preserveAspectRatio="none">
                  <line x1="0" y1="50%" x2="100" y2="50%" stroke="#f59e0b" stroke-width="2" />
                </svg>
              </div>

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
                          type="info"
                          size="small"
                          plain
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
            <div class="section-header finals">
              <el-icon><Trophy /></el-icon>
              总决赛 (Grand Final)
            </div>

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
                    @click="simulateSingleMatch(finalMatch, 'final')"
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
                <div class="champion-crown">👑</div>
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

      <!-- 最终排名 -->
      <div v-if="playoffsCompleted" class="final-standings">
        <div class="standings-header">
          <h3>🏆 最终排名</h3>
          <span class="subtitle">年度积分已发放</span>
        </div>
        <div class="standings-grid">
          <div class="standing-item champion">
            <div class="rank-icon">👑</div>
            <div class="rank-label">冠军</div>
            <div class="team-name">{{ champion?.name }}</div>
            <div class="points-badge">+12 分</div>
          </div>
          <div class="standing-item runner-up">
            <div class="rank-icon">🥈</div>
            <div class="rank-label">亚军</div>
            <div class="team-name">{{ runnerUp?.name }}</div>
            <div class="points-badge">+10 分</div>
          </div>
          <div class="standing-item third">
            <div class="rank-icon">🥉</div>
            <div class="rank-label">季军</div>
            <div class="team-name">{{ thirdPlace?.name }}</div>
            <div class="points-badge">+8 分</div>
          </div>
          <div class="standing-item fourth">
            <div class="rank-icon">4</div>
            <div class="rank-label">殿军</div>
            <div class="team-name">{{ fourthPlace?.name }}</div>
            <div class="points-badge">+6 分</div>
          </div>
        </div>

        <div class="completion-section">
          <el-alert
            title="春季季后赛已完成！"
            type="success"
            :closable="false"
            show-icon
          >
            <template #default>
              <p>恭喜 <strong>{{ champion?.name }}</strong> 获得 {{ getRegionName(selectedRegion) }} 春季赛冠军！</p>
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
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  DArrowRight,
  Trophy,
  Top,
  Bottom,
  VideoPlay,
} from '@element-plus/icons-vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, teamApi, tournamentApi, matchApi, financeApi, type Team, type TournamentMatch, type DetailedGameResult, type PlayerGameStats } from '@/api/tauri'
import type { PlayerPosition, TraitType } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import { createLogger } from '@/utils/logger'

const logger = createLogger('SpringPlayoffsDetail')

const router = useRouter()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()
const gameStore = useGameStore()

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 状态
const selectedRegion = ref(1)
const playoffsSimulating = ref(false)
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
  id: 'final',
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
  status: 'upcoming',
  matchDetailKey: ''
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
    const seasonId = gameStore.gameState?.current_season || 1
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    const springPlayoffs = tournaments.find(t => t.tournament_type === 'SpringPlayoffs')
    if (springPlayoffs) {
      currentTournamentId.value = springPlayoffs.id
    }
  } catch (error) {
    logger.error('Failed to load tournament:', error)
  }
}

const checkRegularSeasonStatus = async (regionId: number) => {
  try {
    const seasonId = gameStore.gameState?.current_season || 1
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    const springRegular = tournaments.find(t => t.tournament_type === 'SpringRegular')

    if (springRegular) {
      // 检查常规赛是否完成 - 获取赛程并检查是否所有比赛都已完成
      const schedule = await tournamentApi.getSchedule(springRegular.id)
      // 状态比较不区分大小写
      const allMatchesCompleted = schedule.every(match =>
        match.status.toLowerCase() === 'completed'
      )
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
    const seasonId = gameStore.gameState?.current_season || 1
    const tournaments = await queryApi.getRegionTournaments(regionId, seasonId)
    const springRegular = tournaments.find(t => t.tournament_type === 'SpringRegular')
    if (springRegular) {
      const standingList = await tournamentApi.getStandings(springRegular.id)
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

  // 辅助函数：根据 stage 和 match_order 找到数据库比赛
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
    id: 'final',
    matchDetailKey: 'final',
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

  // 检查是否已完成
  if ((gfMatch?.status === 'Completed' || gfMatch?.status === 'COMPLETED') && gfMatch.winner_id) {
    playoffsCompleted.value = true
    champion.value = { name: getShortName(gfMatch.winner_id) }
    const loserId = gfMatch.winner_id === gfMatch.home_team_id ? gfMatch.away_team_id : gfMatch.home_team_id
    runnerUp.value = { name: getShortName(loserId) }

    // 从败者组决赛获取季军（败者组决赛的败者）
    if (lfMatch && (lfMatch.status === 'Completed' || lfMatch.status === 'COMPLETED') && lfMatch.winner_id) {
      const lfLoserId = lfMatch.winner_id === lfMatch.home_team_id ? lfMatch.away_team_id : lfMatch.home_team_id
      thirdPlace.value = { name: getShortName(lfLoserId) }
    }

    // 从败者组第三轮获取殿军（败者组第三轮的败者）
    if (l5Match && (l5Match.status === 'Completed' || l5Match.status === 'COMPLETED') && l5Match.winner_id) {
      const l5LoserId = l5Match.winner_id === l5Match.home_team_id ? l5Match.away_team_id : l5Match.home_team_id
      fourthPlace.value = { name: getShortName(l5LoserId) }
    }
  }
}

// 方法
const goBack = () => router.push('/tournaments')
const getRegionName = (regionId: number) => regions.value.find(r => r.id === regionId)?.name || 'LPL'

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
  const key = match.matchDetailKey || match.id
  let detail = matchDetailStore.getMatchDetail(`spring-playoffs-${key}`)

  // 如果本地没有，尝试从数据库加载
  if (!detail && match.dbMatchId) {
    logger.debug(`本地未找到详情，尝试从数据库加载: dbMatchId=${match.dbMatchId}`)
    detail = await matchDetailStore.loadMatchDetailFromDb(match.dbMatchId)
  }

  if (detail) {
    currentMatchDetail.value = detail
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
  const team = standings.value.find(t => t.id === teamId)
  return team?.short || '待定'
}

/**
 * 解析位置字符串，处理 Some(Adc) 格式转换为 ADC
 */
const parsePosition = (pos: string): PlayerPosition => {
  if (!pos) return 'MID' as PlayerPosition
  const p = pos.trim()
  // 处理 Some(xxx) 格式
  if (p.startsWith('Some(') && p.endsWith(')')) {
    const inner = p.slice(5, -1)
    return inner.toUpperCase() as PlayerPosition
  }
  // 直接返回大写
  return p.toUpperCase() as PlayerPosition
}

/**
 * 将后端选手表现数据转换为前端格式
 */
const convertPlayerPerformance = (p: PlayerGameStats, teamId: string) => ({
  playerId: String(p.player_id),
  playerName: p.player_name,
  position: parsePosition(p.position),
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

/**
 * 模拟单场比赛的核心函数 - 使用后端 API
 * @param match 前端对阵数据
 * @param dbMatchId 数据库中的比赛ID
 * @param matchIdPrefix 用于存储比赛详情的前缀
 */
const doSimulateMatch = async (match: any, dbMatchId: number, matchIdPrefix: string): Promise<number> => {
  const regionName = getRegionName(selectedRegion.value)

  // 调用后端比赛模拟 API
  const result = await matchApi.simulateMatchDetailed(dbMatchId)

  // 更新比赛状态
  match.scoreA = result.home_score
  match.scoreB = result.away_score
  match.winnerId = result.winner_id
  match.status = 'completed'

  // 将后端结果转换为前端 MatchDetail 格式
  const matchDetail: MatchDetail = {
    matchId: `spring-playoffs-${matchIdPrefix}`,
    tournamentType: 'spring-playoffs',
    seasonId: String(gameStore.currentSeason),
    teamAId: String(result.home_team_id),
    teamAName: match.teamA,
    teamBId: String(result.away_team_id),
    teamBName: match.teamB,
    bestOf: 5,
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    winnerName: result.winner_id === result.home_team_id ? match.teamA : match.teamB,
    mvpPlayerId: result.match_mvp ? String(result.match_mvp.player_id) : undefined,
    mvpPlayerName: result.match_mvp?.player_name,
    mvpTeamId: result.match_mvp ? String(result.match_mvp.team_id) : undefined,
    mvpTotalImpact: result.match_mvp?.mvp_score,
    games: result.games.map((game: DetailedGameResult) => {
      // 计算队伍战力（选手实际发挥能力平均值）- 每局不同
      const teamAPower = game.home_players.length > 0
        ? Math.round(game.home_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.home_players.length)
        : 0
      const teamBPower = game.away_players.length > 0
        ? Math.round(game.away_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.away_players.length)
        : 0
      const powerDifference = teamAPower - teamBPower

      return {
        gameNumber: game.game_number,
        teamAId: String(result.home_team_id),
        teamAName: match.teamA,
        teamAPower,
        teamAPerformance: game.home_performance,
        teamAPlayers: game.home_players.map(p => convertPlayerPerformance(p, String(result.home_team_id))),
        teamBId: String(result.away_team_id),
        teamBName: match.teamB,
        teamBPower,
        teamBPerformance: game.away_performance,
        teamBPlayers: game.away_players.map(p => convertPlayerPerformance(p, String(result.away_team_id))),
        winnerId: String(game.winner_id),
        winnerName: game.winner_id === result.home_team_id ? match.teamA : match.teamB,
        powerDifference,
        performanceDifference: game.home_performance - game.away_performance,
        isUpset: powerDifference > 0 && game.winner_id !== result.home_team_id ||
                 powerDifference < 0 && game.winner_id === result.home_team_id,
        duration: game.duration_minutes,
        mvp: game.game_mvp ? {
          playerId: String(game.game_mvp.player_id),
          playerName: game.game_mvp.player_name,
          teamId: String(game.game_mvp.team_id),
          position: parsePosition(game.game_mvp.position),
          mvpScore: game.game_mvp.mvp_score
        } : undefined
      }
    })
  }

  // 保存比赛详情到 Store（用本地 key）
  await matchDetailStore.saveMatchDetail(matchDetail.matchId, matchDetail)

  // 同时用数据库 ID 保存一份，确保能从数据库加载
  // 创建一个带有正确 dbMatchId 的副本用于数据库存储
  const dbMatchDetail = { ...matchDetail, matchId: dbMatchId }
  await matchDetailStore.saveMatchDetail(dbMatchId, dbMatchDetail)

  // 记录选手表现到统计
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
  else if (matchId === 'final') {
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
      `恭喜 ${winnerName} 获得 ${getRegionName(selectedRegion.value)} 春季赛冠军！\n\n` +
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

    await updateBracketAfterMatch(match.id, winnerId, loserId!)

    // 刷新数据以获取后端更新的对阵信息
    await updatePlayoffsData()

    ElMessage.success(`比赛完成: ${match.teamA} ${match.scoreA} - ${match.scoreB} ${match.teamB}`)
  } catch (error) {
    logger.error('模拟比赛失败:', error)
    ElMessage.error('模拟比赛失败')
  } finally {
    simulatingMatchId.value = null
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

  // 按顺序模拟所有比赛
  const matchOrder = [
    { match: winnersRounds.value[0].matches[0], id: 'w1-1' },
    { match: winnersRounds.value[0].matches[1], id: 'w1-2' },
    { match: losersRounds.value[0].matches[0], id: 'l1-1' },
    { match: losersRounds.value[0].matches[1], id: 'l1-2' },
  ]

  for (const { match, id } of matchOrder) {
    if (match.status !== 'completed' && canSimulate(match)) {
      await simulateSingleMatch(match, id)
      await new Promise(resolve => setTimeout(resolve, 200))
    }
  }

  // 胜者组决赛 + 败者组第二轮
  if (winnersRounds.value[1].matches[0].status !== 'completed' && canSimulate(winnersRounds.value[1].matches[0])) {
    await simulateSingleMatch(winnersRounds.value[1].matches[0], 'wf')
    await new Promise(resolve => setTimeout(resolve, 200))
  }

  for (let i = 0; i < losersRounds.value[1].matches.length; i++) {
    const match = losersRounds.value[1].matches[i]
    if (match.status !== 'completed' && canSimulate(match)) {
      await simulateSingleMatch(match, `l2-${i+1}`)
      await new Promise(resolve => setTimeout(resolve, 200))
    }
  }

  // 败者组第三轮
  if (losersRounds.value[2].matches[0].status !== 'completed' && canSimulate(losersRounds.value[2].matches[0])) {
    await simulateSingleMatch(losersRounds.value[2].matches[0], 'l3')
    await new Promise(resolve => setTimeout(resolve, 200))
  }

  // 败者组决赛
  if (losersRounds.value[3].matches[0].status !== 'completed' && canSimulate(losersRounds.value[3].matches[0])) {
    await simulateSingleMatch(losersRounds.value[3].matches[0], 'lf')
    await new Promise(resolve => setTimeout(resolve, 200))
  }

  // 总决赛
  if (finalMatch.value.status !== 'completed' && canSimulate(finalMatch.value)) {
    await simulateSingleMatch(finalMatch.value, 'final')
  }

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

<style scoped lang="scss">
.spring-playoffs-management {
  padding: 24px;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content .header-left {
      display: flex;
      flex-direction: column;
      gap: 8px;

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
  }

  .playoffs-status-card {
    background: white;
    border-radius: 16px;
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
        }
      }
    }

    .regular-season-alert {
      margin-bottom: 24px;
    }

    .teams-stats {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 20px;
      margin-bottom: 24px;
      padding: 20px;
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      border-radius: 12px;
    }
  }

  .bracket-card {
    border-radius: 12px;

    .card-header .stage-title {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 18px;
      font-weight: 600;
    }
  }

  .full-bracket {
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .bracket-section {
    border-radius: 12px;
    padding: 20px;

    &.winners-section {
      background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
    }

    &.losers-section {
      background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
    }

    &.finals-section {
      background: linear-gradient(135deg, #fdf4ff 0%, #fae8ff 100%);
      border: 2px solid #a855f7;
    }

    .section-header {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
      margin-bottom: 20px;
      padding-bottom: 12px;
      border-bottom: 2px solid;

      &.winners {
        color: #16a34a;
        border-color: #22c55e;
      }

      &.losers {
        color: #d97706;
        border-color: #f59e0b;
      }

      &.finals {
        color: #9333ea;
        border-color: #a855f7;
      }
    }
  }

  .bracket-container {
    display: flex;
    align-items: center;
    gap: 0;
    overflow-x: auto;
    padding: 20px 0;
  }

  .bracket-round {
    min-width: 220px;
    flex-shrink: 0;

    .round-header {
      text-align: center;
      font-size: 13px;
      font-weight: 600;
      color: #6b7280;
      margin-bottom: 16px;
      padding: 6px 12px;
      background: rgba(255, 255, 255, 0.8);
      border-radius: 6px;

      .drop-hint {
        font-size: 11px;
        color: #9ca3af;
        font-weight: normal;
      }
    }
  }

  .matches-column {
    display: flex;
    flex-direction: column;
    gap: 24px;
    min-height: 220px;
    justify-content: space-around;

    &.final {
      justify-content: center;
      min-height: 100px;
    }
  }

  .match-card-wrapper {
    display: flex;
    align-items: center;
  }

  .match-card {
    background: white;
    border-radius: 10px;
    padding: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    border-left: 4px solid #d1d5db;
    min-width: 200px;
    transition: all 0.2s;

    &:hover {
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }

    &.winners {
      border-left-color: #22c55e;
    }

    &.losers {
      border-left-color: #f59e0b;
    }

    &.completed {
      border-left-color: #3b82f6;
    }

    &.final-match {
      border-left-width: 6px;
    }

    .drop-in-indicator {
      font-size: 10px;
      color: #9ca3af;
      margin-bottom: 4px;
      padding: 2px 6px;
      background: #f3f4f6;
      border-radius: 4px;
      display: inline-block;
    }

    .match-teams {
      .match-team {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 8px;
        border-radius: 4px;
        transition: background 0.2s;

        &.winner {
          background: linear-gradient(90deg, #dcfce7, transparent);
        }

        .seed {
          font-size: 11px;
          color: #9ca3af;
          min-width: 20px;
        }

        .name {
          flex: 1;
          font-weight: 600;
          font-size: 14px;
        }

        .score {
          font-size: 16px;
          font-weight: 700;
          min-width: 20px;
          text-align: center;
        }
      }
    }

    .match-actions {
      margin-top: 8px;
      display: flex;
      gap: 8px;
      justify-content: flex-end;
    }
  }

  // SVG连接线
  .bracket-connector {
    width: 100px;
    min-width: 100px;
    flex-shrink: 0;
    display: flex;
    align-items: center;

    &.winners-connector {
      height: 220px;
    }

    &.losers-connector {
      height: 220px;

      &.single {
        height: 100px;
      }
    }

    .connector-svg {
      width: 100%;
      height: 100%;
    }
  }

  // 总决赛区域
  .grand-final-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    padding: 20px;
  }

  .final-match-card {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 40px;
    padding: 32px 48px;
    background: white;
    border-radius: 16px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    border: 3px solid #e5e7eb;
    position: relative;

    &.completed {
      border-color: #22c55e;
    }

    .final-team {
      text-align: center;
      padding: 20px 28px;
      background: #f9fafb;
      border-radius: 12px;
      min-width: 140px;
      transition: all 0.3s;

      &.champion {
        background: linear-gradient(135deg, #fbbf24, #f59e0b);
        color: white;
        transform: scale(1.1);
        box-shadow: 0 8px 24px rgba(245, 158, 11, 0.4);
      }

      .team-label {
        font-size: 11px;
        color: #6b7280;
        margin-bottom: 6px;
      }

      &.champion .team-label {
        color: rgba(255, 255, 255, 0.8);
      }

      .team-name {
        font-size: 22px;
        font-weight: 700;
        margin-bottom: 8px;
      }

      .team-score {
        font-size: 40px;
        font-weight: 900;
      }
    }

    .vs-badge {
      text-align: center;

      span {
        display: block;
        font-size: 28px;
        font-weight: 900;
        color: #a855f7;
      }

      small {
        font-size: 12px;
        color: #9ca3af;
      }
    }

    .final-actions {
      position: absolute;
      bottom: -50px;
    }
  }

  .champion-display {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px 40px;
    background: linear-gradient(135deg, #fbbf24, #f59e0b);
    border-radius: 12px;
    box-shadow: 0 4px 15px rgba(245, 158, 11, 0.3);
    margin-top: 20px;

    .champion-crown {
      font-size: 32px;
      animation: bounce 1s infinite;
    }

    .champion-trophy {
      font-size: 48px;
    }

    .champion-info {
      .champion-label {
        font-size: 14px;
        color: rgba(255, 255, 255, 0.8);
      }

      .champion-name {
        font-size: 28px;
        font-weight: 900;
        color: white;
      }
    }
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-5px); }
  }

  // 最终排名
  .final-standings {
    margin-top: 32px;
    padding: 24px;
    background: linear-gradient(135deg, #fefce8 0%, #fef9c3 100%);
    border-radius: 16px;
    border: 2px solid #fbbf24;

    .standings-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 24px;
      padding-bottom: 16px;
      border-bottom: 2px dashed #fbbf24;

      h3 {
        margin: 0;
        font-size: 22px;
        font-weight: 700;
        color: #92400e;
      }

      .subtitle {
        font-size: 13px;
        color: #a16207;
        background: rgba(251, 191, 36, 0.3);
        padding: 4px 12px;
        border-radius: 20px;
        font-weight: 500;
      }
    }

    .standings-grid {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 16px;
      margin-bottom: 24px;

      .standing-item {
        padding: 20px 16px;
        border-radius: 16px;
        text-align: center;
        border: 2px solid;
        background: white;
        transition: all 0.3s ease;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);

        &:hover {
          transform: translateY(-4px);
          box-shadow: 0 8px 20px rgba(0, 0, 0, 0.12);
        }

        .rank-icon {
          font-size: 36px;
          margin-bottom: 8px;
          line-height: 1;
        }

        .rank-label {
          font-size: 12px;
          font-weight: 600;
          color: #6b7280;
          text-transform: uppercase;
          letter-spacing: 1px;
          margin-bottom: 8px;
        }

        .team-name {
          font-size: 20px;
          font-weight: 800;
          margin-bottom: 12px;
          color: #1f2937;
        }

        .points-badge {
          display: inline-block;
          font-size: 14px;
          font-weight: 700;
          padding: 6px 14px;
          border-radius: 20px;
          background: linear-gradient(135deg, #10b981, #059669);
          color: white;
          box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
        }

        &.champion {
          border-color: #fbbf24;
          background: linear-gradient(135deg, #fffbeb, #fef3c7);
          position: relative;
          overflow: hidden;

          &::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: linear-gradient(90deg, #fbbf24, #f59e0b, #fbbf24);
          }

          .rank-icon {
            font-size: 42px;
            filter: drop-shadow(0 2px 4px rgba(251, 191, 36, 0.5));
          }

          .rank-label {
            color: #92400e;
          }

          .team-name {
            color: #92400e;
          }

          .points-badge {
            background: linear-gradient(135deg, #f59e0b, #d97706);
            box-shadow: 0 2px 8px rgba(245, 158, 11, 0.4);
          }
        }

        &.runner-up {
          border-color: #9ca3af;
          background: linear-gradient(135deg, #f9fafb, #f3f4f6);

          .rank-icon {
            filter: drop-shadow(0 2px 4px rgba(156, 163, 175, 0.4));
          }

          .rank-label {
            color: #4b5563;
          }
        }

        &.third {
          border-color: #d97706;
          background: linear-gradient(135deg, #fffbeb, #fef3c7);

          .rank-icon {
            filter: drop-shadow(0 2px 4px rgba(217, 119, 6, 0.4));
          }

          .rank-label {
            color: #92400e;
          }
        }

        &.fourth {
          border-color: #60a5fa;
          background: linear-gradient(135deg, #eff6ff, #dbeafe);

          .rank-icon {
            font-weight: 900;
            font-size: 28px;
            color: #3b82f6;
            background: linear-gradient(135deg, #3b82f6, #2563eb);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
          }

          .rank-label {
            color: #1d4ed8;
          }
        }
      }
    }

    .completion-section {
      :deep(.el-alert) {
        border-radius: 12px;
        background: white;
        border: 1px solid #22c55e;

        p {
          margin: 8px 0 0 0;

          strong {
            color: #f59e0b;
            font-weight: 700;
          }
        }
      }
    }
  }
}

@media (max-width: 1200px) {
  .spring-playoffs-management {
    .bracket-container {
      flex-direction: column;
      align-items: stretch;
    }

    .bracket-connector {
      display: none;
    }

    .bracket-round {
      width: 100%;
    }

    .matches-column {
      min-height: auto;
    }

    .final-match-card {
      flex-direction: column;
      gap: 20px;
    }

    .final-standings {
      padding: 16px;

      .standings-header {
        flex-direction: column;
        gap: 12px;
        text-align: center;
      }

      .standings-grid {
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;

        .standing-item {
          padding: 16px 12px;

          .rank-icon {
            font-size: 28px;
          }

          .team-name {
            font-size: 16px;
          }

          .points-badge {
            font-size: 12px;
            padding: 4px 10px;
          }
        }
      }
    }
  }
}
</style>
