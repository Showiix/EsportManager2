<template>
  <div class="super-management">
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
            Super洲际年度邀请赛 (Intercontinental Super Cup)
          </h1>
          <p class="page-description">
            全球年度积分前16强战队，四阶段BO5淘汰赛，角逐年度最强荣耀
          </p>
        </div>
      </div>
      <div class="header-actions">
        <el-button
          v-if="superBracket.status === 'not_started'"
          type="primary"
          @click="handleStartTournament"
          :loading="starting"
        >
          <el-icon><VideoPlay /></el-icon>
          开始Super洲际赛
        </el-button>
        <el-button
          v-if="superBracket.status === 'fighter_stage' && !isFighterStageComplete"
          type="primary"
          @click="batchSimulateFighterStage"
          :loading="simulatingFighter"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingFighter ? `模拟中 (${simulationProgress}%)` : '模拟Fighter组预选赛' }}
        </el-button>
        <el-button
          v-if="canGenerateChallenger"
          type="success"
          @click="handleGenerateChallengerStage"
          :loading="generatingChallenger"
        >
          <el-icon><Plus /></el-icon>
          生成第二阶段
        </el-button>
        <el-button
          v-if="superBracket.status === 'challenger_stage' && !isChallengerStageComplete"
          type="primary"
          @click="batchSimulateChallengerStage"
          :loading="simulatingChallenger"
        >
          <el-icon><DArrowRight /></el-icon>
          模拟挑战者组
        </el-button>
        <el-button
          v-if="canGenerateChampionPrep"
          type="success"
          @click="handleGenerateChampionPrepStage"
          :loading="generatingChampionPrep"
        >
          <el-icon><Plus /></el-icon>
          生成第三阶段
        </el-button>
        <el-button
          v-if="superBracket.status === 'champion_prep_stage' && !isChampionPrepComplete"
          type="primary"
          @click="batchSimulateChampionPrepStage"
          :loading="simulatingChampionPrep"
        >
          <el-icon><DArrowRight /></el-icon>
          模拟冠军预备战
        </el-button>
        <el-button
          v-if="canGenerateFinalStage"
          type="success"
          @click="handleGenerateFinalStage"
          :loading="generatingFinal"
        >
          <el-icon><Plus /></el-icon>
          生成终极冠军赛
        </el-button>
        <el-button
          v-if="superBracket.status === 'final_stage'"
          type="warning"
          @click="batchSimulateFinalStage"
          :loading="simulatingFinal"
        >
          <el-icon><DArrowRight /></el-icon>
          模拟终极冠军赛
        </el-button>
      </div>
    </div>

    <!-- 阶段未到提示 -->
    <el-alert
      v-if="phaseNotReached"
      title="赛事尚未开始"
      type="warning"
      :closable="false"
      show-icon
      class="phase-warning-alert"
    >
      <template #default>
        <div class="phase-warning-content">
          <p>当前赛季阶段：<strong>{{ currentPhaseDisplay }}</strong></p>
          <p>Super洲际年度邀请赛需要在 <strong>ICP洲际对抗赛</strong> 结束后才会开始。</p>
          <p>请先完成之前的赛事阶段，然后在时间控制面板推进到Super洲际赛阶段。</p>
        </div>
      </template>
    </el-alert>

    <!-- Super洲际赛状态卡片 -->
    <div class="super-status-card">
      <div class="status-header">
        <div class="status-info">
          <h2>Super洲际年度邀请赛</h2>
          <el-tag :type="getStatusType(superBracket.status)" size="large">
            {{ getStatusText(superBracket.status) }}
          </el-tag>
        </div>
      </div>

      <!-- 参赛队伍统计 -->
      <div class="teams-stats">
        <el-statistic title="参赛队伍总数" :value="16" />
        <el-statistic title="传奇组 (1-4名)" :value="4" suffix="支" />
        <el-statistic title="挑战者组 (5-8名)" :value="4" suffix="支" />
        <el-statistic title="Fighter组 (9-16名)" :value="8" suffix="支" />
      </div>

      <!-- 参赛队伍分组 -->
      <div v-if="superBracket.status !== 'not_started'" class="teams-groups">
        <div class="team-group legendary">
          <h3><el-icon><Star /></el-icon> 传奇组 (年度积分 1-4名)</h3>
          <div class="team-list">
            <div
              v-for="team in superBracket.qualifiedTeams.legendGroup"
              :key="team.teamId"
              class="team-item"
            >
              <span class="team-rank">#{{ team.globalRank }}</span>
              <span class="team-name">{{ team.teamName }}</span>
              <el-tag size="small">{{ team.regionName }}</el-tag>
              <span class="team-points">{{ team.annualPoints }}分</span>
            </div>
          </div>
        </div>

        <div class="team-group challenger">
          <h3><el-icon><Medal /></el-icon> 挑战者组 (年度积分 5-8名)</h3>
          <div class="team-list">
            <div
              v-for="team in superBracket.qualifiedTeams.challengerGroup"
              :key="team.teamId"
              class="team-item"
            >
              <span class="team-rank">#{{ team.globalRank }}</span>
              <span class="team-name">{{ team.teamName }}</span>
              <el-tag size="small">{{ team.regionName }}</el-tag>
              <span class="team-points">{{ team.annualPoints }}分</span>
            </div>
          </div>
        </div>

        <div class="team-group fighter">
          <h3><el-icon><Flag /></el-icon> Fighter组 (年度积分 9-16名)</h3>
          <div class="team-list">
            <div
              v-for="team in superBracket.qualifiedTeams.fighterGroup"
              :key="team.teamId"
              class="team-item"
            >
              <span class="team-rank">#{{ team.globalRank }}</span>
              <span class="team-name">{{ team.teamName }}</span>
              <el-tag size="small">{{ team.regionName }}</el-tag>
              <span class="team-points">{{ team.annualPoints }}分</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 第一阶段：Fighter组预选赛 -->
      <el-card v-if="superBracket.status !== 'not_started'" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🥊 第一阶段：Fighter组预选赛</span>
            <el-tag v-if="isFighterStageComplete" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <p class="stage-description">
          8支Fighter组战队（第9-16名）随机分为A、B两组，组内BO3单循环，每组第1名晋级第二阶段
        </p>

        <div class="fighter-groups">
          <el-tabs v-model="activeFighterGroup" type="card">
            <el-tab-pane
              v-for="group in superBracket.fighterGroups"
              :key="group.groupName"
              :label="`${group.groupName}组`"
              :name="group.groupName"
            >
              <SuperGroupStanding
                :group="group"
                @simulate-match="handleSimulateMatch"
                @view-match="viewMatchDetails"
              />
            </el-tab-pane>
          </el-tabs>
        </div>
      </el-card>

      <!-- 第二阶段：挑战者组 -->
      <el-card v-if="superBracket.challengerStage" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>⚔️ 第二阶段：挑战者组定位赛与晋级赛</span>
            <el-tag v-if="isChallengerStageComplete" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <SuperKnockoutBracket
          stage="challenger"
          :challenger-stage="superBracket.challengerStage"
          @simulate-match="handleSimulateMatch"
          @view-match="viewMatchDetails"
        />
      </el-card>

      <!-- 第三阶段：冠军赛预备战 -->
      <el-card v-if="superBracket.championPrepStage" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🎯 第三阶段：冠军赛预备战</span>
            <el-tag v-if="isChampionPrepComplete" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <SuperKnockoutBracket
          stage="champion_prep"
          :champion-prep-stage="superBracket.championPrepStage"
          @simulate-match="handleSimulateMatch"
          @view-match="viewMatchDetails"
        />
      </el-card>

      <!-- 第四阶段：终极冠军赛 -->
      <el-card v-if="superBracket.finalStage" class="stage-card">
        <template #header>
          <div class="card-header">
            <span>🏆 第四阶段：终极冠军赛</span>
            <el-tag v-if="superBracket.status === 'completed'" type="success">已完成</el-tag>
            <el-tag v-else type="warning">进行中</el-tag>
          </div>
        </template>

        <SuperKnockoutBracket
          stage="final"
          :final-stage="superBracket.finalStage"
          @simulate-match="handleSimulateMatch"
          @view-match="viewMatchDetails"
        />
      </el-card>

      <!-- 最终排名 -->
      <div v-if="superBracket.status === 'completed'" class="final-standings">
        <h3>最终排名与积分</h3>
        <div class="standings-grid">
          <div class="standing-item champion">
            <div class="rank-badge">🏆 冠军</div>
            <div class="team-name">{{ superBracket.champion?.teamName }}</div>
            <div class="region-name">{{ superBracket.champion?.regionName }}</div>
            <div class="points">+35分</div>
          </div>

          <div class="standing-item runner-up">
            <div class="rank-badge">🥈 亚军</div>
            <div class="team-name">{{ superBracket.runnerUp?.teamName }}</div>
            <div class="region-name">{{ superBracket.runnerUp?.regionName }}</div>
            <div class="points">+30分</div>
          </div>

          <div class="standing-item third">
            <div class="rank-badge">🥉 季军</div>
            <div class="team-name">{{ superBracket.thirdPlace?.teamName }}</div>
            <div class="region-name">{{ superBracket.thirdPlace?.regionName }}</div>
            <div class="points">+25分</div>
          </div>

          <div class="standing-item fourth">
            <div class="rank-badge">4️⃣ 殿军</div>
            <div class="team-name">{{ superBracket.fourthPlace?.teamName }}</div>
            <div class="region-name">{{ superBracket.fourthPlace?.regionName }}</div>
            <div class="points">+20分</div>
          </div>
        </div>

        <!-- Super洲际赛完成后的操作区 -->
        <div class="super-completed-actions">
          <el-alert
            title="Super洲际年度邀请赛已完成！"
            type="success"
            :closable="false"
            show-icon
            class="completion-alert"
          >
            <template #default>
              <p>恭喜 <strong>{{ superBracket.champion?.teamName }}</strong> 获得Super洲际年度邀请赛冠军，成为本赛季最强战队！</p>
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
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Trophy,
  ArrowLeft,
  DArrowRight,
  Plus,
  VideoPlay,
  Star,
  Medal,
  Flag
} from '@element-plus/icons-vue'
import SuperGroupStanding from '@/components/super/SuperGroupStanding.vue'
import SuperKnockoutBracket from '@/components/super/SuperKnockoutBracket.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, financeApi, pointsApi } from '@/api/tauri'
import type { BracketInfo, MatchBracketInfo, GroupStandingInfo, DetailedMatchResult, DetailedGameResult, PlayerGameStats } from '@/api/tauri'
import type { MatchDetail } from '@/types/matchDetail'
import type { PlayerPosition, TraitType } from '@/types/player'
import type {
  SuperMatch,
  SuperBracket,
} from '@/types/super'

const router = useRouter()
const gameStore = useGameStore()
const timeStore = useTimeStore()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()

// 阶段检查
const SUPER_PHASE = 'SUPER_INTERCONTINENTAL'
const phaseNotReached = computed(() => {
  const currentPhase = timeStore.currentPhase
  // 后端使用 SCREAMING_SNAKE_CASE 格式序列化阶段名称
  const phaseOrder = [
    'SPRING_REGULAR', 'SPRING_PLAYOFFS', 'MSI', 'MADRID_MASTERS',
    'SUMMER_REGULAR', 'SUMMER_PLAYOFFS', 'CLAUDE_INTERCONTINENTAL',
    'WORLD_CHAMPIONSHIP', 'SHANGHAI_MASTERS', 'ICP_INTERCONTINENTAL',
    'SUPER_INTERCONTINENTAL', 'TRANSFER_WINDOW', 'DRAFT', 'SEASON_END'
  ]
  const currentIndex = phaseOrder.indexOf(currentPhase)
  const targetIndex = phaseOrder.indexOf(SUPER_PHASE)
  return currentIndex < targetIndex
})

const currentPhaseDisplay = computed(() => timeStore.phaseDisplayName)

// 后端数据
const tournamentId = ref<number | null>(null)
const bracketData = ref<BracketInfo | null>(null)
const groupStandings = ref<GroupStandingInfo[]>([])

// 比赛详情弹窗状态
const showMatchDetailDialog = ref(false)
const currentMatchDetail = ref<MatchDetail | null>(null)

// 响应式状态
const starting = ref(false)
const generatingChallenger = ref(false)
const generatingChampionPrep = ref(false)
const generatingFinal = ref(false)
const simulatingFighter = ref(false)
const simulatingChallenger = ref(false)
const simulatingChampionPrep = ref(false)
const simulatingFinal = ref(false)
const simulationProgress = ref(0)
const activeFighterGroup = ref('A')

/**
 * 从后端加载赛事数据
 */
const loadTournamentData = async () => {
  if (!tournamentId.value) return

  try {
    // 获取对阵数据
    bracketData.value = await internationalApi.getTournamentBracket(tournamentId.value)

    // 获取小组积分榜
    groupStandings.value = await internationalApi.getGroupStandings(tournamentId.value)

    // 转换数据到前端格式
    convertBracketToSuperFormat()
  } catch (error) {
    console.error('加载赛事数据失败:', error)
  }
}

/**
 * 将后端数据转换为 Super 前端格式
 */
const convertBracketToSuperFormat = () => {
  if (!bracketData.value) return

  // 转换 Fighter 组小组赛数据
  convertFighterGroupsData()

  // 转换各阶段淘汰赛数据
  convertKnockoutData()

  // 更新状态
  updateTournamentStatus()
}

/**
 * 转换 Fighter 组小组赛数据
 */
const convertFighterGroupsData = () => {
  if (!bracketData.value) return

  // 获取 Fighter 组的比赛
  const fighterMatches = bracketData.value.matches.filter(m => m.stage.startsWith('FIGHTER_GROUP'))

  // 如果没有 Fighter 组比赛，直接返回
  if (fighterMatches.length === 0) return

  // 按组分类
  const groupAMatches = fighterMatches.filter(m => m.stage === 'FIGHTER_GROUP_A')
  const groupBMatches = fighterMatches.filter(m => m.stage === 'FIGHTER_GROUP_B')

  // 获取积分榜（如果有）
  const groupAStanding = groupStandings.value.find(g => g.group_name === 'FIGHTER_A')
  const groupBStanding = groupStandings.value.find(g => g.group_name === 'FIGHTER_B')

  // 从比赛数据中提取队伍信息，生成初始积分榜
  const extractTeamsFromMatches = (matches: typeof groupAMatches) => {
    const teamMap = new Map<number, { id: number; name: string; region: string }>()
    matches.forEach(m => {
      if (m.home_team) {
        teamMap.set(m.home_team.id, { id: m.home_team.id, name: m.home_team.name, region: m.home_team.region_code })
      }
      if (m.away_team) {
        teamMap.set(m.away_team.id, { id: m.away_team.id, name: m.away_team.name, region: m.away_team.region_code })
      }
    })
    return Array.from(teamMap.values())
  }

  // 生成初始积分榜（如果没有积分榜数据）
  const generateInitialStandings = (teams: { id: number; name: string; region: string }[]) => {
    return teams.map((t, i) => ({
      teamId: String(t.id),
      teamName: t.name,
      regionName: t.region,
      position: i + 1,
      matchesPlayed: 0,
      wins: 0,
      losses: 0,
      points: 0,
      roundsWon: 0,
      roundsLost: 0,
      roundDifferential: 0,
      qualified: false
    }))
  }

  superBracket.fighterGroups = [
    {
      groupName: 'A',
      standings: groupAStanding ? groupAStanding.teams.map((t, i) => ({
        teamId: String(t.team_id),
        teamName: t.team_name,
        regionName: t.region_code || '',
        position: i + 1,
        matchesPlayed: t.wins + t.losses,
        wins: t.wins,
        losses: t.losses,
        points: t.points,
        roundsWon: t.games_won,
        roundsLost: t.games_lost,
        roundDifferential: t.games_won - t.games_lost,
        qualified: i === 0
      })) : generateInitialStandings(extractTeamsFromMatches(groupAMatches)),
      matches: groupAMatches.map(m => convertMatchFormat(m))
    },
    {
      groupName: 'B',
      standings: groupBStanding ? groupBStanding.teams.map((t, i) => ({
        teamId: String(t.team_id),
        teamName: t.team_name,
        regionName: t.region_code || '',
        position: i + 1,
        matchesPlayed: t.wins + t.losses,
        wins: t.wins,
        losses: t.losses,
        points: t.points,
        roundsWon: t.games_won,
        roundsLost: t.games_lost,
        roundDifferential: t.games_won - t.games_lost,
        qualified: i === 0
      })) : generateInitialStandings(extractTeamsFromMatches(groupBMatches)),
      matches: groupBMatches.map(m => convertMatchFormat(m))
    }
  ]
}

/**
 * 转换淘汰赛数据
 */
const convertKnockoutData = () => {
  if (!bracketData.value) return

  const matches = bracketData.value.matches

  // 挑战者组阶段
  const positioningMatches = matches.filter(m => m.stage === 'CHALLENGER_POSITIONING')
  const promotionMatches = matches.filter(m => m.stage === 'CHALLENGER_PROMOTION')

  if (positioningMatches.length > 0 || promotionMatches.length > 0) {
    superBracket.challengerStage = {
      positioningMatches: positioningMatches.map(m => convertMatchFormat(m)),
      promotionMatches: promotionMatches.map(m => convertMatchFormat(m))
    }
  }

  // 冠军预备战阶段 (使用初始化时的阶段名)
  const winnersMatch = matches.find(m => m.stage === 'PREP_WINNERS')
  const losersMatch = matches.find(m => m.stage === 'PREP_LOSERS')
  const losersFinal = matches.find(m => m.stage === 'PREP_LOSERS_FINAL')

  // 只有当 PREP_WINNERS 比赛有完整的队伍配对时才设置 championPrepStage
  // 初始化时 PREP_WINNERS 的队伍 ID 都是 0，只有生成第三阶段后才会填充
  const hasChampionPrepReady = winnersMatch && winnersMatch.home_team?.id && winnersMatch.away_team?.id
  if (hasChampionPrepReady) {
    superBracket.championPrepStage = {
      winnersMatch: winnersMatch ? convertMatchFormat(winnersMatch) : undefined,
      losersMatch: losersMatch ? convertMatchFormat(losersMatch) : undefined,
      losersFinal: losersFinal ? convertMatchFormat(losersFinal) : undefined
    }
  }

  // 终极冠军赛阶段 (使用初始化时的阶段名)
  const round1Matches = matches.filter(m => m.stage === 'FINALS_R1')
  const round2Matches = matches.filter(m => m.stage === 'FINALS_R2')
  const thirdPlaceMatch = matches.find(m => m.stage === 'THIRD_PLACE')
  const grandFinal = matches.find(m => m.stage === 'GRAND_FINAL')

  // 只有当 FINALS_R1 比赛有完整的队伍配对时才设置 finalStage
  // 初始化时 FINALS_R1 的 away_team_id 是 0，只有生成第四阶段后才会填充
  const hasR1TeamsReady = round1Matches.length > 0 && round1Matches.every(m => m.home_team?.id && m.away_team?.id)
  if (hasR1TeamsReady) {
    superBracket.finalStage = {
      round1: round1Matches.map(m => convertMatchFormat(m)),
      round2: round2Matches.map(m => convertMatchFormat(m)),
      thirdPlaceMatch: thirdPlaceMatch ? convertMatchFormat(thirdPlaceMatch) : undefined,
      grandFinal: grandFinal ? convertMatchFormat(grandFinal) : undefined
    }
  }

  // 设置最终排名
  setFinalStandings()
}

/**
 * 转换比赛格式
 */
const convertMatchFormat = (m: MatchBracketInfo): SuperMatch => {
  // 从 stage 提取组名（例如 'FIGHTER_GROUP_A' -> 'A'）
  let groupName: string | undefined
  if (m.stage.startsWith('FIGHTER_GROUP_')) {
    groupName = m.stage.replace('FIGHTER_GROUP_', '')
  }

  // 根据 match_order 计算轮次（4队单循环，每轮2场比赛）
  // match_order: 2,3,4 -> 第1轮; 6,7 -> 第2轮; 10 -> 第3轮
  let roundNumber = 1
  if (m.match_order >= 6 && m.match_order < 10) {
    roundNumber = 2
  } else if (m.match_order >= 10) {
    roundNumber = 3
  }

  return {
    id: String(m.match_id),
    teamAId: m.home_team?.id ? String(m.home_team.id) : '',
    teamAName: m.home_team?.name || '待定',
    teamARegion: m.home_team?.region_code || '',
    teamBId: m.away_team?.id ? String(m.away_team.id) : '',
    teamBName: m.away_team?.name || '待定',
    teamBRegion: m.away_team?.region_code || '',
    scoreA: m.home_score,
    scoreB: m.away_score,
    winnerId: m.winner_id ? String(m.winner_id) : undefined,
    status: (m.status === 'Completed' || m.status === 'COMPLETED') ? 'completed' : 'scheduled',
    bestOf: m.format === 'BO5' ? 5 : 3,
    stage: m.stage.startsWith('FIGHTER_GROUP') ? 'fighter_group' : m.stage as any,
    groupName,
    roundNumber,
    matchType: m.stage as any
  }
}

/**
 * 更新赛事状态
 */
const updateTournamentStatus = () => {
  if (!bracketData.value) return

  const matches = bracketData.value.matches

  // 检查各阶段完成状态
  const fighterMatches = matches.filter(m => m.stage.startsWith('FIGHTER_GROUP'))
  const fighterComplete = fighterMatches.length > 0 && fighterMatches.every(m => m.status === 'Completed' || m.status === 'COMPLETED')

  const challengerMatches = matches.filter(m => m.stage.startsWith('CHALLENGER'))
  const challengerComplete = challengerMatches.length > 0 && challengerMatches.every(m => m.status === 'Completed' || m.status === 'COMPLETED')

  const championPrepMatches = matches.filter(m => m.stage.startsWith('PREP_'))
  const championPrepComplete = championPrepMatches.length > 0 && championPrepMatches.every(m => m.status === 'Completed' || m.status === 'COMPLETED')

  const grandFinal = matches.find(m => m.stage === 'GRAND_FINAL')
  const tournamentComplete = grandFinal?.status === 'Completed' || grandFinal?.status === 'COMPLETED'

  // 设置状态
  if (tournamentComplete) {
    superBracket.status = 'completed'
  } else if (championPrepComplete && superBracket.finalStage) {
    superBracket.status = 'final_stage'
  } else if (challengerComplete && superBracket.championPrepStage) {
    superBracket.status = 'champion_prep_stage'
  } else if (fighterComplete && superBracket.challengerStage) {
    superBracket.status = 'challenger_stage'
  } else if (fighterMatches.length > 0) {
    superBracket.status = 'fighter_stage'
  }
}

/**
 * 设置最终排名
 */
const setFinalStandings = () => {
  if (!superBracket.finalStage?.grandFinal || superBracket.finalStage.grandFinal.status !== 'completed') return

  const gf = superBracket.finalStage.grandFinal
  const tp = superBracket.finalStage.thirdPlaceMatch

  if (gf.winnerId === gf.teamAId) {
    superBracket.champion = { teamId: gf.teamAId!, teamName: gf.teamAName || '', regionName: gf.teamARegion || '', annualPoints: 0, globalRank: 0 }
    superBracket.runnerUp = { teamId: gf.teamBId!, teamName: gf.teamBName || '', regionName: gf.teamBRegion || '', annualPoints: 0, globalRank: 0 }
  } else {
    superBracket.champion = { teamId: gf.teamBId!, teamName: gf.teamBName || '', regionName: gf.teamBRegion || '', annualPoints: 0, globalRank: 0 }
    superBracket.runnerUp = { teamId: gf.teamAId!, teamName: gf.teamAName || '', regionName: gf.teamARegion || '', annualPoints: 0, globalRank: 0 }
  }

  if (tp && tp.winnerId) {
    if (tp.winnerId === tp.teamAId) {
      superBracket.thirdPlace = { teamId: tp.teamAId!, teamName: tp.teamAName || '', regionName: tp.teamARegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.fourthPlace = { teamId: tp.teamBId!, teamName: tp.teamBName || '', regionName: tp.teamBRegion || '', annualPoints: 0, globalRank: 0 }
    } else {
      superBracket.thirdPlace = { teamId: tp.teamBId!, teamName: tp.teamBName || '', regionName: tp.teamBRegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.fourthPlace = { teamId: tp.teamAId!, teamName: tp.teamAName || '', regionName: tp.teamARegion || '', annualPoints: 0, globalRank: 0 }
    }
  }
}

// Super洲际赛数据
const superBracket = reactive<SuperBracket>({
  id: '1',
  seasonYear: 2024,
  status: 'not_started',
  qualifiedTeams: {
    legendGroup: [],
    challengerGroup: [],
    fighterGroup: []
  },
  fighterGroups: [],
  challengerStage: null,
  championPrepStage: null,
  finalStage: null,
  champion: null,
  runnerUp: null,
  thirdPlace: null,
  fourthPlace: null
})

// 计算属性
const isFighterStageComplete = computed(() => {
  return superBracket.fighterGroups.every(group => {
    return group.matches.every(match => match.status === 'completed')
  })
})

const isChallengerStageComplete = computed(() => {
  if (!superBracket.challengerStage) return false
  const allMatches = [
    ...superBracket.challengerStage.positioningMatches,
    ...superBracket.challengerStage.promotionMatches
  ]
  return allMatches.every(m => m.status === 'completed')
})

const isChampionPrepComplete = computed(() => {
  if (!superBracket.championPrepStage) return false
  const matches = [
    superBracket.championPrepStage.winnersMatch,
    superBracket.championPrepStage.losersMatch,
    superBracket.championPrepStage.losersFinal
  ].filter(m => m != null) as SuperMatch[]
  return matches.every(m => m.status === 'completed')
})

const canGenerateChallenger = computed(() => {
  return superBracket.status === 'fighter_stage' &&
    isFighterStageComplete.value &&
    !superBracket.challengerStage
})

const canGenerateChampionPrep = computed(() => {
  return superBracket.status === 'challenger_stage' &&
    isChallengerStageComplete.value &&
    !superBracket.championPrepStage
})

const canGenerateFinalStage = computed(() => {
  return superBracket.status === 'champion_prep_stage' &&
    isChampionPrepComplete.value &&
    !superBracket.finalStage
})

// 方法
const goBack = () => {
  router.push('/tournaments')
}

const getStatusType = (status: string) => {
  const typeMap: Record<string, any> = {
    'not_started': 'info',
    'fighter_stage': 'warning',
    'challenger_stage': 'warning',
    'champion_prep_stage': 'warning',
    'final_stage': 'warning',
    'completed': 'success'
  }
  return typeMap[status] || 'info'
}

const getStatusText = (status: string) => {
  const textMap: Record<string, string> = {
    'not_started': '未开始',
    'fighter_stage': 'Fighter组预选赛',
    'challenger_stage': '挑战者组阶段',
    'champion_prep_stage': '冠军预备战阶段',
    'final_stage': '终极冠军赛进行中',
    'completed': '已完成'
  }
  return textMap[status] || status
}

/**
 * 查看比赛详情
 */
const viewMatchDetails = async (match: SuperMatch) => {
  if (match.status === 'completed') {
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
 * 开始Super洲际赛
 */
const handleStartTournament = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要开始Super洲际年度邀请赛吗？将根据年度积分排名确定参赛队伍。',
      '开始Super洲际赛',
      {
        confirmButtonText: '开始',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    starting.value = true

    // TODO: 从后端获取年度积分排名队伍
    // 目前使用示例数据，实际应从后端获取
    const legendaryTeamIds = [1, 2, 3, 4]      // 传奇组: 年度积分 1-4 名
    const challengerTeamIds = [5, 6, 7, 8]      // 挑战者组: 年度积分 5-8 名
    const fighterTeamIds = [9, 10, 11, 12, 13, 14, 15, 16]  // Fighter组: 年度积分 9-16 名

    // 调用后端创建赛事
    const id = await internationalApi.createSuperTournament(
      legendaryTeamIds,
      challengerTeamIds,
      fighterTeamIds
    )
    tournamentId.value = id

    // 加载赛事数据
    await loadTournamentData()

    superBracket.status = 'fighter_stage'
    ElMessage.success('Super洲际赛已开始！Fighter组预选赛抽签完成。')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('开始失败:', error)
      ElMessage.error(error.message || '开始失败')
    }
  } finally {
    starting.value = false
  }
}

/**
 * 模拟单场比赛
 */
const handleSimulateMatch = async (match: SuperMatch) => {
  try {
    const matchId = Number(match.id)
    if (!match.id || isNaN(matchId)) {
      ElMessage.error('比赛ID无效，请先生成该阶段的比赛')
      return
    }
    const result = await matchApi.simulateMatchDetailed(matchId)

    // 更新比赛状态
    match.scoreA = result.home_score
    match.scoreB = result.away_score
    match.winnerId = String(result.winner_id)
    match.status = 'completed'
    match.completedAt = new Date()

    // 转换选手表现数据
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

    // 构建比赛详情
    const matchDetail: MatchDetail = {
      matchId: match.id,
      tournamentId: String(tournamentId.value),
      tournamentType: 'super',
      seasonId: String(superBracket.seasonYear),
      teamAId: String(match.teamAId || ''),
      teamAName: match.teamAName || '',
      teamBId: String(match.teamBId || ''),
      teamBName: match.teamBName || '',
      finalScoreA: result.home_score,
      finalScoreB: result.away_score,
      winnerId: String(result.winner_id),
      bestOf: match.bestOf || 5,
      games: result.games.map((game: DetailedGameResult) => {
        const gameWinnerId = String(game.winner_id)
        const teamAId = String(match.teamAId || '')
        const teamBId = String(match.teamBId || '')
        const isTeamAWinner = gameWinnerId === teamAId
        return {
          gameNumber: game.game_number,
          teamAId,
          teamAName: match.teamAName || '',
          teamAPower: game.home_performance,
          teamAPerformance: game.home_performance,
          teamAPlayers: game.home_players.map(p => convertPlayerPerformance(p, teamAId)),
          teamBId,
          teamBName: match.teamBName || '',
          teamBPower: game.away_performance,
          teamBPerformance: game.away_performance,
          teamBPlayers: game.away_players.map(p => convertPlayerPerformance(p, teamBId)),
          winnerId: gameWinnerId,
          winnerName: isTeamAWinner ? (match.teamAName || '') : (match.teamBName || ''),
          powerDifference: game.home_performance - game.away_performance,
          performanceDifference: game.home_performance - game.away_performance,
          isUpset: false,
          duration: game.duration_minutes,
          mvp: {
            playerId: String(game.game_mvp.player_id),
            playerName: game.game_mvp.player_name,
            teamId: String(game.game_mvp.team_id),
            position: game.game_mvp.position as PlayerPosition,
            mvpScore: game.game_mvp.mvp_score
          }
        }
      }),
      matchMvp: result.match_mvp ? {
        playerId: String(result.match_mvp.player_id),
        playerName: result.match_mvp.player_name,
        teamId: String(result.match_mvp.team_id),
        position: result.match_mvp.position as PlayerPosition,
        mvpScore: result.match_mvp.mvp_score
      } : undefined,
      completedAt: new Date()
    }

    // 保存比赛详情
    matchDetailStore.saveMatchDetail(match.id, matchDetail)

    // 调用后端推进对阵 (如果是淘汰赛)
    if (tournamentId.value && result.winner_id) {
      try {
        await internationalApi.advanceBracket(tournamentId.value, matchId, result.winner_id)
      } catch (e) {
        // 可能不是淘汰赛，忽略错误
      }
    }

    // 重新加载数据
    await loadTournamentData()

    ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

    // 检查是否所有比赛都完成了
    if (superBracket.status === 'final_stage') {
      checkFinalCompletion()
    }
  } catch (error) {
    console.error('模拟比赛失败:', error)
    ElMessage.error('模拟比赛失败')
  }
}

/**
 * 批量模拟Fighter组预选赛
 */
const batchSimulateFighterStage = async () => {
  try {
    await ElMessageBox.confirm(
      '将自动模拟所有未完成的Fighter组预选赛比赛。是否继续?',
      '模拟Fighter组预选赛',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    simulatingFighter.value = true
    simulationProgress.value = 0

    // 从后端获取未完成的 Fighter 组比赛
    const fighterMatches = bracketData.value?.matches.filter(
      m => m.stage.startsWith('FIGHTER_GROUP') && m.status !== 'Completed'
    ) || []

    for (let i = 0; i < fighterMatches.length; i++) {
      const match = fighterMatches[i]
      const result = await matchApi.simulateMatchDetailed(match.match_id)

      // 保存比赛详情
      saveMatchDetailFromRaw(match, result)

      // 推进对阵
      if (tournamentId.value && result.winner_id) {
        try {
          await internationalApi.advanceBracket(tournamentId.value, match.match_id, result.winner_id)
        } catch (e) {
          // 忽略
        }
      }

      simulationProgress.value = Math.floor(((i + 1) / fighterMatches.length) * 100)
      await new Promise(resolve => setTimeout(resolve, 100))
    }

    // 重新加载数据
    await loadTournamentData()

    ElMessage.success('Fighter组预选赛模拟完成！现在可以生成第二阶段。')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('模拟失败:', error)
      ElMessage.error(error.message || '模拟失败')
    }
  } finally {
    simulatingFighter.value = false
    simulationProgress.value = 0
  }
}

/**
 * 生成第二阶段（挑战者组）
 */
const handleGenerateChallengerStage = async () => {
  console.log('[Super] 生成第二阶段, tournamentId:', tournamentId.value)

  if (!tournamentId.value) {
    ElMessage.error('赛事ID不存在，请刷新页面重试')
    return
  }

  generatingChallenger.value = true

  try {
    // 调用后端生成挑战者组阶段
    console.log('[Super] 调用 generateKnockoutBracket API...')
    await internationalApi.generateKnockoutBracket(tournamentId.value)
    console.log('[Super] API 调用成功')

    // 重新加载数据
    await loadTournamentData()

    superBracket.status = 'challenger_stage'
    ElMessage.success('第二阶段生成成功！')
  } catch (error: any) {
    console.error('生成第二阶段失败:', error)
    ElMessage.error(error?.message || '生成第二阶段失败')
  } finally {
    generatingChallenger.value = false
  }
}

/**
 * 批量模拟挑战者组阶段
 */
const batchSimulateChallengerStage = async () => {
  if (!tournamentId.value) return

  try {
    await ElMessageBox.confirm(
      '将自动模拟所有挑战者组比赛。是否继续?',
      '模拟挑战者组',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    simulatingChallenger.value = true

    // 获取挑战者组阶段的比赛
    const challengerMatches = bracketData.value?.matches.filter(
      m => m.stage.startsWith('CHALLENGER') && m.status !== 'Completed'
    ) || []

    for (const match of challengerMatches) {
      const result = await matchApi.simulateMatchDetailed(match.match_id)

      // 保存比赛详情
      saveMatchDetailFromRaw(match, result)

      // 推进对阵
      if (tournamentId.value && result.winner_id) {
        try {
          await internationalApi.advanceBracket(tournamentId.value, match.match_id, result.winner_id)
        } catch (e) {
          // 忽略
        }
      }

      await new Promise(resolve => setTimeout(resolve, 200))
    }

    // 重新加载数据
    await loadTournamentData()

    ElMessage.success('挑战者组阶段模拟完成！现在可以生成第三阶段。')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('模拟失败:', error)
      ElMessage.error(error.message || '模拟失败')
    }
  } finally {
    simulatingChallenger.value = false
  }
}

/**
 * 生成第三阶段（冠军预备战）
 */
const handleGenerateChampionPrepStage = async () => {
  if (!tournamentId.value) return

  generatingChampionPrep.value = true

  try {
    // 调用后端 API 生成第三阶段比赛
    const matchIds = await internationalApi.generateChampionPrepStage(tournamentId.value)
    console.log('[handleGenerateChampionPrepStage] 创建了比赛 IDs:', matchIds)

    // 重新加载赛事数据
    await loadTournamentData()

    ElMessage.success('第三阶段生成成功！')
  } catch (error) {
    console.error('生成第三阶段失败:', error)
    ElMessage.error(`生成第三阶段失败: ${error}`)
  } finally {
    generatingChampionPrep.value = false
  }
}

/**
 * 批量模拟冠军预备战
 */
const batchSimulateChampionPrepStage = async () => {
  if (!superBracket.championPrepStage) return

  try {
    await ElMessageBox.confirm(
      '将自动模拟冠军预备战阶段。是否继续?',
      '模拟冠军预备战',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    simulatingChampionPrep.value = true

    // 模拟胜者组对决
    if (superBracket.championPrepStage.winnersMatch?.status !== 'completed') {
      await simulateMatchInternal(superBracket.championPrepStage.winnersMatch!)
      await new Promise(resolve => setTimeout(resolve, 200))
    }

    // 模拟败者组对决
    if (superBracket.championPrepStage.losersMatch?.status !== 'completed') {
      await simulateMatchInternal(superBracket.championPrepStage.losersMatch!)
      await new Promise(resolve => setTimeout(resolve, 200))
    }

    // 更新败者组决赛对阵
    const wm = superBracket.championPrepStage.winnersMatch!
    const lm = superBracket.championPrepStage.losersMatch!

    const winnersLoser = wm.winnerId === wm.teamAId
      ? { id: wm.teamBId, name: wm.teamBName, region: wm.teamBRegion }
      : { id: wm.teamAId, name: wm.teamAName, region: wm.teamARegion }

    const losersWinner = lm.winnerId === lm.teamAId
      ? { id: lm.teamAId, name: lm.teamAName, region: lm.teamARegion }
      : { id: lm.teamBId, name: lm.teamBName, region: lm.teamBRegion }

    superBracket.championPrepStage.losersFinal!.teamAId = winnersLoser.id
    superBracket.championPrepStage.losersFinal!.teamAName = winnersLoser.name
    superBracket.championPrepStage.losersFinal!.teamARegion = winnersLoser.region
    superBracket.championPrepStage.losersFinal!.teamBId = losersWinner.id
    superBracket.championPrepStage.losersFinal!.teamBName = losersWinner.name
    superBracket.championPrepStage.losersFinal!.teamBRegion = losersWinner.region

    // 模拟败者组决赛
    await simulateMatchInternal(superBracket.championPrepStage.losersFinal!)

    // 重新加载赛事数据以同步后端状态
    await loadTournamentData()

    ElMessage.success('冠军预备战模拟完成！现在可以生成终极冠军赛。')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('模拟失败:', error)
      ElMessage.error(error.message || '模拟失败')
    }
  } finally {
    simulatingChampionPrep.value = false
  }
}

/**
 * 生成第四阶段（终极冠军赛）
 */
const handleGenerateFinalStage = async () => {
  if (!tournamentId.value) return

  generatingFinal.value = true

  try {
    // 调用后端 API 生成第四阶段比赛
    const matchIds = await internationalApi.generateFinalStage(tournamentId.value)
    console.log('[handleGenerateFinalStage] 创建了比赛 IDs:', matchIds)

    // 重新加载赛事数据
    await loadTournamentData()

    ElMessage.success('终极冠军赛生成成功！')
  } catch (error) {
    console.error('生成终极冠军赛失败:', error)
    ElMessage.error(`生成终极冠军赛失败: ${error}`)
  } finally {
    generatingFinal.value = false
  }
}

/**
 * 批量模拟终极冠军赛
 */
const batchSimulateFinalStage = async () => {
  if (!superBracket.finalStage) return

  try {
    await ElMessageBox.confirm(
      '将自动模拟终极冠军赛，直到决出冠军。是否继续?',
      '模拟终极冠军赛',
      {
        confirmButtonText: '开始模拟',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    simulatingFinal.value = true

    // 模拟首轮
    for (const match of superBracket.finalStage.round1) {
      if (match.status !== 'completed') {
        await simulateMatchInternal(match)
        await new Promise(resolve => setTimeout(resolve, 200))
      }
    }

    // 更新次轮对阵
    const r1m1 = superBracket.finalStage.round1[0]
    const r1m2 = superBracket.finalStage.round1[1]

    const r1w1 = r1m1.winnerId === r1m1.teamAId
      ? { id: r1m1.teamAId, name: r1m1.teamAName, region: r1m1.teamARegion }
      : { id: r1m1.teamBId, name: r1m1.teamBName, region: r1m1.teamBRegion }

    const r1w2 = r1m2.winnerId === r1m2.teamAId
      ? { id: r1m2.teamAId, name: r1m2.teamAName, region: r1m2.teamARegion }
      : { id: r1m2.teamBId, name: r1m2.teamBName, region: r1m2.teamBRegion }

    superBracket.finalStage.round2[0].teamAId = r1w1.id
    superBracket.finalStage.round2[0].teamAName = r1w1.name
    superBracket.finalStage.round2[0].teamARegion = r1w1.region

    superBracket.finalStage.round2[1].teamAId = r1w2.id
    superBracket.finalStage.round2[1].teamAName = r1w2.name
    superBracket.finalStage.round2[1].teamARegion = r1w2.region

    // 模拟次轮
    for (const match of superBracket.finalStage.round2) {
      if (match.status !== 'completed') {
        await simulateMatchInternal(match)
        await new Promise(resolve => setTimeout(resolve, 200))
      }
    }

    // 更新季军赛和总决赛对阵
    const r2m1 = superBracket.finalStage.round2[0]
    const r2m2 = superBracket.finalStage.round2[1]

    const r2w1 = r2m1.winnerId === r2m1.teamAId
      ? { id: r2m1.teamAId, name: r2m1.teamAName, region: r2m1.teamARegion }
      : { id: r2m1.teamBId, name: r2m1.teamBName, region: r2m1.teamBRegion }

    const r2l1 = r2m1.winnerId === r2m1.teamAId
      ? { id: r2m1.teamBId, name: r2m1.teamBName, region: r2m1.teamBRegion }
      : { id: r2m1.teamAId, name: r2m1.teamAName, region: r2m1.teamARegion }

    const r2w2 = r2m2.winnerId === r2m2.teamAId
      ? { id: r2m2.teamAId, name: r2m2.teamAName, region: r2m2.teamARegion }
      : { id: r2m2.teamBId, name: r2m2.teamBName, region: r2m2.teamBRegion }

    const r2l2 = r2m2.winnerId === r2m2.teamAId
      ? { id: r2m2.teamBId, name: r2m2.teamBName, region: r2m2.teamBRegion }
      : { id: r2m2.teamAId, name: r2m2.teamAName, region: r2m2.teamARegion }

    // 季军赛对阵
    superBracket.finalStage.thirdPlaceMatch!.teamAId = r2l1.id
    superBracket.finalStage.thirdPlaceMatch!.teamAName = r2l1.name
    superBracket.finalStage.thirdPlaceMatch!.teamARegion = r2l1.region
    superBracket.finalStage.thirdPlaceMatch!.teamBId = r2l2.id
    superBracket.finalStage.thirdPlaceMatch!.teamBName = r2l2.name
    superBracket.finalStage.thirdPlaceMatch!.teamBRegion = r2l2.region

    // 总决赛对阵
    superBracket.finalStage.grandFinal!.teamAId = r2w1.id
    superBracket.finalStage.grandFinal!.teamAName = r2w1.name
    superBracket.finalStage.grandFinal!.teamARegion = r2w1.region
    superBracket.finalStage.grandFinal!.teamBId = r2w2.id
    superBracket.finalStage.grandFinal!.teamBName = r2w2.name
    superBracket.finalStage.grandFinal!.teamBRegion = r2w2.region

    // 模拟季军赛
    await simulateMatchInternal(superBracket.finalStage.thirdPlaceMatch!)
    await new Promise(resolve => setTimeout(resolve, 200))

    // 模拟总决赛
    await simulateMatchInternal(superBracket.finalStage.grandFinal!)

    // 设置最终排名
    const gf = superBracket.finalStage.grandFinal!
    const tp = superBracket.finalStage.thirdPlaceMatch!

    if (gf.winnerId === gf.teamAId) {
      superBracket.champion = { teamId: gf.teamAId!, teamName: gf.teamAName || '', regionName: gf.teamARegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.runnerUp = { teamId: gf.teamBId!, teamName: gf.teamBName || '', regionName: gf.teamBRegion || '', annualPoints: 0, globalRank: 0 }
    } else {
      superBracket.champion = { teamId: gf.teamBId!, teamName: gf.teamBName || '', regionName: gf.teamBRegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.runnerUp = { teamId: gf.teamAId!, teamName: gf.teamAName || '', regionName: gf.teamARegion || '', annualPoints: 0, globalRank: 0 }
    }

    if (tp.winnerId === tp.teamAId) {
      superBracket.thirdPlace = { teamId: tp.teamAId!, teamName: tp.teamAName || '', regionName: tp.teamARegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.fourthPlace = { teamId: tp.teamBId!, teamName: tp.teamBName || '', regionName: tp.teamBRegion || '', annualPoints: 0, globalRank: 0 }
    } else {
      superBracket.thirdPlace = { teamId: tp.teamBId!, teamName: tp.teamBName || '', regionName: tp.teamBRegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.fourthPlace = { teamId: tp.teamAId!, teamName: tp.teamAName || '', regionName: tp.teamARegion || '', annualPoints: 0, globalRank: 0 }
    }

    superBracket.status = 'completed'

    // 重新加载赛事数据以同步后端状态
    await loadTournamentData()

    showChampionCelebration(superBracket.champion?.teamName || '')
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('模拟失败:', error)
      ElMessage.error(error.message || '模拟失败')
    }
  } finally {
    simulatingFinal.value = false
  }
}

/**
 * 模拟单场比赛（内部方法）- 使用后端 API
 */
const simulateMatchInternal = async (match: SuperMatch) => {
  // 调用 handleSimulateMatch 但不显示消息
  try {
    const matchId = Number(match.id)
    if (isNaN(matchId)) {
      console.error('无效的比赛ID:', match.id)
      return
    }

    const result = await matchApi.simulateMatchDetailed(matchId)

    // 更新比赛状态
    match.scoreA = result.home_score
    match.scoreB = result.away_score
    match.winnerId = String(result.winner_id)
    match.status = 'completed'
    match.completedAt = new Date()

    // 转换选手表现数据
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

    // 构建比赛详情
    const matchDetail: MatchDetail = {
      matchId: match.id,
      tournamentId: String(tournamentId.value),
      tournamentType: 'super',
      seasonId: String(superBracket.seasonYear),
      teamAId: String(match.teamAId || ''),
      teamAName: match.teamAName || '',
      teamBId: String(match.teamBId || ''),
      teamBName: match.teamBName || '',
      finalScoreA: result.home_score,
      finalScoreB: result.away_score,
      winnerId: String(result.winner_id),
      bestOf: match.bestOf || 5,
      games: result.games.map((game: DetailedGameResult) => {
        const gameWinnerId = String(game.winner_id)
        const teamAId = String(match.teamAId || '')
        const teamBId = String(match.teamBId || '')
        const isTeamAWinner = gameWinnerId === teamAId
        return {
          gameNumber: game.game_number,
          teamAId,
          teamAName: match.teamAName || '',
          teamAPower: game.home_performance,
          teamAPerformance: game.home_performance,
          teamAPlayers: game.home_players.map(p => convertPlayerPerformance(p, teamAId)),
          teamBId,
          teamBName: match.teamBName || '',
          teamBPower: game.away_performance,
          teamBPerformance: game.away_performance,
          teamBPlayers: game.away_players.map(p => convertPlayerPerformance(p, teamBId)),
          winnerId: gameWinnerId,
          winnerName: isTeamAWinner ? (match.teamAName || '') : (match.teamBName || ''),
          powerDifference: game.home_performance - game.away_performance,
          performanceDifference: game.home_performance - game.away_performance,
          isUpset: false,
          duration: game.duration_minutes,
          mvp: {
            playerId: String(game.game_mvp.player_id),
            playerName: game.game_mvp.player_name,
            teamId: String(game.game_mvp.team_id),
            position: game.game_mvp.position as PlayerPosition,
            mvpScore: game.game_mvp.mvp_score
          }
        }
      }),
      matchMvp: result.match_mvp ? {
        playerId: String(result.match_mvp.player_id),
        playerName: result.match_mvp.player_name,
        teamId: String(result.match_mvp.team_id),
        position: result.match_mvp.position as PlayerPosition,
        mvpScore: result.match_mvp.mvp_score
      } : undefined,
      completedAt: new Date()
    }

    // 保存比赛详情
    matchDetailStore.saveMatchDetail(match.id, matchDetail)

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
          String(superBracket.seasonYear),
          'INTL'
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
          String(superBracket.seasonYear),
          'INTL'
        )
      })
    })

    // 推进对阵
    if (tournamentId.value && result.winner_id) {
      try {
        await internationalApi.advanceBracket(tournamentId.value, matchId, result.winner_id)
      } catch (e) {
        // 可能不是淘汰赛，忽略错误
      }
    }
  } catch (error) {
    console.error('模拟比赛失败:', error)
    throw error
  }
}

/**
 * 保存比赛详情（从原始后端数据）
 * 用于批量模拟时保存详情
 */
const saveMatchDetailFromRaw = (match: MatchBracketInfo, result: DetailedMatchResult) => {
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

  const teamAId = match.home_team?.id ? String(match.home_team.id) : ''
  const teamBId = match.away_team?.id ? String(match.away_team.id) : ''

  const matchDetail: MatchDetail = {
    matchId: String(match.match_id),
    tournamentId: String(tournamentId.value),
    tournamentType: 'super',
    seasonId: String(superBracket.seasonYear),
    teamAId,
    teamAName: match.home_team?.name || '待定',
    teamBId,
    teamBName: match.away_team?.name || '待定',
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
    winnerId: String(result.winner_id),
    bestOf: match.format === 'BO5' ? 5 : 3,
    games: result.games.map((game: DetailedGameResult) => {
      const gameWinnerId = String(game.winner_id)
      const isTeamAWinner = gameWinnerId === teamAId
      return {
        gameNumber: game.game_number,
        teamAId,
        teamAName: match.home_team?.name || '待定',
        teamAPower: game.home_performance,
        teamAPerformance: game.home_performance,
        teamAPlayers: game.home_players.map(p => convertPlayerPerformance(p, teamAId)),
        teamBId,
        teamBName: match.away_team?.name || '待定',
        teamBPower: game.away_performance,
        teamBPerformance: game.away_performance,
        teamBPlayers: game.away_players.map(p => convertPlayerPerformance(p, teamBId)),
        winnerId: gameWinnerId,
        winnerName: isTeamAWinner ? (match.home_team?.name || '待定') : (match.away_team?.name || '待定'),
        powerDifference: game.home_performance - game.away_performance,
        performanceDifference: game.home_performance - game.away_performance,
        isUpset: false,
        duration: game.duration_minutes,
        mvp: {
          playerId: String(game.game_mvp.player_id),
          playerName: game.game_mvp.player_name,
          teamId: String(game.game_mvp.team_id),
          position: game.game_mvp.position as PlayerPosition,
          mvpScore: game.game_mvp.mvp_score
        }
      }
    }),
    matchMvp: result.match_mvp ? {
      playerId: String(result.match_mvp.player_id),
      playerName: result.match_mvp.player_name,
      teamId: String(result.match_mvp.team_id),
      position: result.match_mvp.position as PlayerPosition,
      mvpScore: result.match_mvp.mvp_score
    } : undefined,
    completedAt: new Date()
  }

  // 保存比赛详情
  matchDetailStore.saveMatchDetail(String(match.match_id), matchDetail)
}

/**
 * 检查终极冠军赛是否完成
 */
const checkFinalCompletion = () => {
  if (superBracket.finalStage?.grandFinal?.status === 'completed') {
    const gf = superBracket.finalStage.grandFinal
    const tp = superBracket.finalStage.thirdPlaceMatch

    if (gf.winnerId === gf.teamAId) {
      superBracket.champion = { teamId: gf.teamAId!, teamName: gf.teamAName || '', regionName: gf.teamARegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.runnerUp = { teamId: gf.teamBId!, teamName: gf.teamBName || '', regionName: gf.teamBRegion || '', annualPoints: 0, globalRank: 0 }
    } else {
      superBracket.champion = { teamId: gf.teamBId!, teamName: gf.teamBName || '', regionName: gf.teamBRegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.runnerUp = { teamId: gf.teamAId!, teamName: gf.teamAName || '', regionName: gf.teamARegion || '', annualPoints: 0, globalRank: 0 }
    }

    if (tp && tp.winnerId === tp.teamAId) {
      superBracket.thirdPlace = { teamId: tp.teamAId!, teamName: tp.teamAName || '', regionName: tp.teamARegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.fourthPlace = { teamId: tp.teamBId!, teamName: tp.teamBName || '', regionName: tp.teamBRegion || '', annualPoints: 0, globalRank: 0 }
    } else if (tp) {
      superBracket.thirdPlace = { teamId: tp.teamBId!, teamName: tp.teamBName || '', regionName: tp.teamBRegion || '', annualPoints: 0, globalRank: 0 }
      superBracket.fourthPlace = { teamId: tp.teamAId!, teamName: tp.teamAName || '', regionName: tp.teamARegion || '', annualPoints: 0, globalRank: 0 }
    }

    superBracket.status = 'completed'
    showChampionCelebration(superBracket.champion?.teamName || '')
  }
}

/**
 * 显示冠军庆祝动画
 */
const showChampionCelebration = async (championName: string) => {
  // 发放赛事奖金
  if (tournamentId.value) {
    try {
      await financeApi.distributeTournamentPrizes(tournamentId.value)
      console.log('Super赛事奖金已发放')
    } catch (e) {
      console.error('发放奖金失败:', e)
    }
  }

  ElMessageBox.alert(
    `恭喜 ${championName} 获得Super洲际年度邀请赛冠军，成为本赛季最强战队！\n\n` +
    `✅ 奖金已发放到各战队账户\n` +
    `💡 请在时间控制面板完成阶段推进，系统将自动颁发荣誉和年度积分`,
    '🏆 Super洲际赛冠军诞生! 🏆',
    {
      confirmButtonText: '太棒了!',
      customClass: 'champion-celebration-box',
      showClose: false,
      center: true
    }
  )
}

/**
 * 初始化Super赛事数据
 */
const initSuperData = async () => {
  try {
    // 先刷新时间状态，确保阶段检查是最新的
    await timeStore.fetchTimeState()

    // 获取当前存档和赛季
    const currentSave = gameStore.currentSave
    if (!currentSave) {
      console.warn('未找到当前存档')
      return
    }

    const seasonId = currentSave.current_season || 1

    // 加载参赛队伍数据（Top 16）
    try {
      const qualifiedTeams = await pointsApi.getSuperQualifiedTeams()
      if (qualifiedTeams && qualifiedTeams.length >= 16) {
        // 传奇组: 1-4名
        superBracket.qualifiedTeams.legendGroup = qualifiedTeams.slice(0, 4).map(t => ({
          teamId: String(t.team_id),
          teamName: t.team_name,
          regionName: t.region_code,
          annualPoints: t.total_points,
          globalRank: t.rank
        }))
        // 挑战者组: 5-8名
        superBracket.qualifiedTeams.challengerGroup = qualifiedTeams.slice(4, 8).map(t => ({
          teamId: String(t.team_id),
          teamName: t.team_name,
          regionName: t.region_code,
          annualPoints: t.total_points,
          globalRank: t.rank
        }))
        // Fighter组: 9-16名
        superBracket.qualifiedTeams.fighterGroup = qualifiedTeams.slice(8, 16).map(t => ({
          teamId: String(t.team_id),
          teamName: t.team_name,
          regionName: t.region_code,
          annualPoints: t.total_points,
          globalRank: t.rank
        }))
      }
    } catch (e) {
      console.warn('加载参赛队伍数据失败:', e)
    }

    // 获取 Super 赛事ID (类型为 'SuperIntercontinental')
    const tournaments = await internationalApi.getTournamentsByType('SuperIntercontinental', seasonId)
    if (tournaments && tournaments.length > 0) {
      tournamentId.value = tournaments[0].id

      // 加载赛事数据
      await loadTournamentData()
    } else {
      console.warn('未找到 Super 赛事')
    }
  } catch (error) {
    console.error('初始化 Super 数据失败:', error)
  }
}

// 生命周期钩子
onMounted(() => {
  initSuperData()
})
</script>

<style scoped lang="scss">
.super-management {
  padding: 24px;

  .phase-warning-alert {
    margin-bottom: 24px;

    .phase-warning-content {
      p {
        margin: 4px 0;
        line-height: 1.6;

        strong {
          color: var(--el-color-warning);
        }
      }
    }
  }

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
      flex-wrap: wrap;
    }
  }

  .super-status-card {
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
      background: linear-gradient(135deg, #f3e8ff 0%, #ddd6fe 100%);
      border-radius: 12px;
    }

    .teams-groups {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 20px;
      margin-bottom: 32px;

      .team-group {
        padding: 20px;
        border-radius: 12px;
        border: 2px solid;

        h3 {
          display: flex;
          align-items: center;
          gap: 8px;
          margin: 0 0 16px 0;
          font-size: 16px;
          font-weight: 600;
        }

        &.legendary {
          border-color: #f59e0b;
          background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);

          h3 {
            color: #92400e;
          }
        }

        &.challenger {
          border-color: #3b82f6;
          background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);

          h3 {
            color: #1e40af;
          }
        }

        &.fighter {
          border-color: #6b7280;
          background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);

          h3 {
            color: #374151;
          }
        }

        .team-list {
          display: flex;
          flex-direction: column;
          gap: 8px;

          .team-item {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 14px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
            transition: all 0.2s ease;

            &:hover {
              transform: translateX(4px);
              box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            }

            .team-rank {
              font-size: 14px;
              font-weight: 700;
              color: #8b5cf6;
              min-width: 32px;
            }

            .team-name {
              flex: 1;
              font-size: 15px;
              font-weight: 600;
              color: #1f2937;
            }

            .team-points {
              font-size: 13px;
              font-weight: 500;
              color: #10b981;
            }
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

      .stage-description {
        margin: 0 0 16px 0;
        color: #6b7280;
        font-size: 14px;
      }
    }

    .qualified-teams-card {
      .qualified-teams-container {
        display: flex;
        flex-direction: column;
        gap: 24px;
      }

      .team-group {
        .group-label {
          display: flex;
          align-items: center;
          gap: 12px;
          margin: 0 0 12px 0;

          .rank-range {
            font-size: 14px;
            color: #6b7280;
          }
        }

        .team-cards {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
          gap: 12px;
        }

        .qualified-team-card {
          padding: 16px;
          border-radius: 8px;
          text-align: center;
          border: 2px solid;

          .rank-badge {
            font-size: 14px;
            font-weight: 600;
            margin-bottom: 8px;
          }

          .team-name {
            font-size: 16px;
            font-weight: 600;
            margin-bottom: 8px;
            color: #1f2937;
          }

          .annual-points {
            margin-top: 8px;
            font-size: 14px;
            font-weight: 500;
            color: #10b981;
          }

          &.legend {
            border-color: #ef4444;
            background: linear-gradient(135deg, #fef2f2 0%, #fee2e2 100%);
          }

          &.challenger {
            border-color: #f59e0b;
            background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
          }

          &.fighter {
            border-color: #6b7280;
            background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
          }
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
          border-radius: 8px;
          text-align: center;
          border: 2px solid;

          .rank-badge {
            font-size: 18px;
            margin-bottom: 8px;
            white-space: nowrap;
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
            color: #8b5cf6;
          }

          &.champion {
            border-color: #8b5cf6;
            background: linear-gradient(135deg, #f3e8ff 0%, #ddd6fe 100%);
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

      .super-completed-actions {
        margin-top: 32px;
        text-align: center;

        .completion-alert {
          margin-bottom: 20px;
          border-radius: 8px;
          text-align: left;

          p {
            margin: 8px 0;
            font-size: 14px;
            line-height: 1.6;

            strong {
              color: #8b5cf6;
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
  background: linear-gradient(135deg, #f3e8ff 0%, #ddd6fe 100%);
  border: 3px solid #8b5cf6;

  .el-message-box__title {
    font-size: 28px;
    font-weight: 900;
    background: linear-gradient(135deg, #8b5cf6 0%, #6d28d9 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .el-message-box__content {
    font-size: 18px;
    color: #6b21a8;
  }

  .el-button--primary {
    background: linear-gradient(135deg, #8b5cf6 0%, #6d28d9 100%);
    border: none;

    &:hover {
      background: linear-gradient(135deg, #7c3aed 0%, #5b21b6 100%);
    }
  }
}
</style>
