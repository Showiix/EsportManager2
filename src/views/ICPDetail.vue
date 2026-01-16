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
          v-if="icpTournament.status === 'region_battle' || icpTournament.status === 'tiebreaker'"
          type="warning"
          @click="batchSimulateRegionBattle"
          :loading="simulatingRegionBattle"
        >
          <el-icon><DArrowRight /></el-icon>
          {{ simulatingRegionBattle ? `模拟中 (${simulationProgress}%)` : '模拟赛区对决' }}
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
          <p>ICP洲际对抗赛需要在 <strong>世界冠军赛</strong> 结束后才会开始。</p>
          <p>请先完成之前的赛事阶段，然后在时间控制面板推进到ICP洲际对抗赛阶段。</p>
        </div>
      </template>
    </el-alert>

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

      <!-- 参赛队伍种子分组展示 -->
      <div class="seed-teams-display" v-if="icpTournament.status !== 'not_started'">
        <h3 class="section-title">
          <el-icon><User /></el-icon>
          参赛队伍种子分组
        </h3>
        <div class="seed-groups-grid">
          <!-- 一号种子 -->
          <div class="seed-group-card seed-1">
            <div class="seed-header">
              <span class="seed-icon">🥇</span>
              <span class="seed-title">一号种子</span>
              <el-tag type="danger" size="small">各赛区冠军</el-tag>
            </div>
            <div class="seed-team-list">
              <template v-if="seedTeamsGrouped[1]?.length > 0">
                <div
                  v-for="team in seedTeamsGrouped[1]"
                  :key="team.teamId"
                  class="seed-team-item"
                >
                  <span class="team-name">{{ team.teamName }}</span>
                  <el-tag :type="getRegionTagType(team.region)" size="small">
                    {{ team.region }}
                  </el-tag>
                </div>
              </template>
              <div v-else class="seed-team-item pending">
                <span class="team-name">待夏季赛结束后确定</span>
              </div>
            </div>
          </div>

          <!-- 二号种子 -->
          <div class="seed-group-card seed-2">
            <div class="seed-header">
              <span class="seed-icon">🥈</span>
              <span class="seed-title">二号种子</span>
              <el-tag type="warning" size="small">各赛区亚军</el-tag>
            </div>
            <div class="seed-team-list">
              <template v-if="seedTeamsGrouped[2]?.length > 0">
                <div
                  v-for="team in seedTeamsGrouped[2]"
                  :key="team.teamId"
                  class="seed-team-item"
                >
                  <span class="team-name">{{ team.teamName }}</span>
                  <el-tag :type="getRegionTagType(team.region)" size="small">
                    {{ team.region }}
                  </el-tag>
                </div>
              </template>
              <div v-else class="seed-team-item pending">
                <span class="team-name">待夏季赛结束后确定</span>
              </div>
            </div>
          </div>

          <!-- 三号种子 -->
          <div class="seed-group-card seed-3">
            <div class="seed-header">
              <span class="seed-icon">🥉</span>
              <span class="seed-title">三号种子</span>
              <el-tag type="success" size="small">各赛区季军</el-tag>
            </div>
            <div class="seed-team-list">
              <template v-if="seedTeamsGrouped[3]?.length > 0">
                <div
                  v-for="team in seedTeamsGrouped[3]"
                  :key="team.teamId"
                  class="seed-team-item"
                >
                  <span class="team-name">{{ team.teamName }}</span>
                  <el-tag :type="getRegionTagType(team.region)" size="small">
                    {{ team.region }}
                  </el-tag>
                </div>
              </template>
              <div v-else class="seed-team-item pending">
                <span class="team-name">待夏季赛结束后确定</span>
              </div>
            </div>
          </div>

          <!-- 四号种子 -->
          <div class="seed-group-card seed-4">
            <div class="seed-header">
              <span class="seed-icon">4️⃣</span>
              <span class="seed-title">四号种子</span>
              <el-tag type="info" size="small">各赛区殿军</el-tag>
            </div>
            <div class="seed-team-list">
              <template v-if="seedTeamsGrouped[4]?.length > 0">
                <div
                  v-for="team in seedTeamsGrouped[4]"
                  :key="team.teamId"
                  class="seed-team-item"
                >
                  <span class="team-name">{{ team.teamName }}</span>
                  <el-tag :type="getRegionTagType(team.region)" size="small">
                    {{ team.region }}
                  </el-tag>
                </div>
              </template>
              <div v-else class="seed-team-item pending">
                <span class="team-name">待夏季赛结束后确定</span>
              </div>
            </div>
          </div>
        </div>
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
            <!-- 半决赛加赛 - 进行中或已完成都显示 -->
            <div v-if="icpTournament.semifinal.status === 'tiebreaker' || icpTournament.semifinal.tiebreakerMatch?.status === 'completed'" class="tiebreaker-section">
              <el-alert
                v-if="icpTournament.semifinal.status === 'tiebreaker'"
                title="比分 2:2 平局！"
                description="需要进行一号种子加赛决出胜者"
                type="warning"
                :closable="false"
                show-icon
                class="mb-4"
              />
              <el-alert
                v-else-if="icpTournament.semifinal.tiebreakerMatch?.status === 'completed'"
                title="加赛已完成"
                type="success"
                :closable="false"
                show-icon
                class="mb-4"
              />
              <div class="tiebreaker-match" v-if="icpTournament.semifinal.tiebreakerMatch">
                <div class="tiebreaker-header">
                  <span class="tiebreaker-icon">⚔️</span>
                  <span class="tiebreaker-title">一号种子加赛 (BO5)</span>
                </div>
                <div class="tiebreaker-teams">
                  <div class="team-side">
                    <span class="team-name">{{ icpTournament.semifinal.tiebreakerMatch.teamAName }}</span>
                    <el-tag :type="getRegionTagType(icpTournament.semifinal.tiebreakerMatch.teamARegion)" size="small">
                      {{ icpTournament.semifinal.tiebreakerMatch.teamARegion }}
                    </el-tag>
                  </div>
                  <div class="vs-section">
                    <template v-if="icpTournament.semifinal.tiebreakerMatch.status === 'completed'">
                      <span class="score">{{ icpTournament.semifinal.tiebreakerMatch.scoreA }}</span>
                      <span class="vs">:</span>
                      <span class="score">{{ icpTournament.semifinal.tiebreakerMatch.scoreB }}</span>
                    </template>
                    <span v-else class="vs">VS</span>
                  </div>
                  <div class="team-side">
                    <el-tag :type="getRegionTagType(icpTournament.semifinal.tiebreakerMatch.teamBRegion)" size="small">
                      {{ icpTournament.semifinal.tiebreakerMatch.teamBRegion }}
                    </el-tag>
                    <span class="team-name">{{ icpTournament.semifinal.tiebreakerMatch.teamBName }}</span>
                  </div>
                </div>
                <div class="tiebreaker-actions">
                  <el-button
                    v-if="icpTournament.semifinal.tiebreakerMatch.status !== 'completed'"
                    type="danger"
                    @click="handleSimulateTiebreaker(icpTournament.semifinal)"
                  >
                    <el-icon><DArrowRight /></el-icon>
                    模拟加赛
                  </el-button>
                  <el-button
                    v-else
                    type="info"
                    @click="viewMatchDetails(icpTournament.semifinal.tiebreakerMatch)"
                  >
                    查看详情
                  </el-button>
                </div>
              </div>
            </div>
          </div>

          <!-- 决赛 -->
          <div v-if="icpTournament.final" class="battle-stage final">
            <h4>🏆 决赛</h4>
            <ICPRegionBattleCard
              :battle="icpTournament.final"
              @simulate-match="handleSimulateRegionMatch"
              @view-match="viewMatchDetails"
            />
            <!-- 决赛加赛 - 进行中或已完成都显示 -->
            <div v-if="icpTournament.final.status === 'tiebreaker' || icpTournament.final.tiebreakerMatch?.status === 'completed'" class="tiebreaker-section">
              <el-alert
                v-if="icpTournament.final.status === 'tiebreaker'"
                title="比分 2:2 平局！"
                description="需要进行一号种子加赛决出最强赛区"
                type="warning"
                :closable="false"
                show-icon
                class="mb-4"
              />
              <el-alert
                v-else-if="icpTournament.final.tiebreakerMatch?.status === 'completed'"
                title="加赛已完成"
                type="success"
                :closable="false"
                show-icon
                class="mb-4"
              />
              <div class="tiebreaker-match" v-if="icpTournament.final.tiebreakerMatch">
                <div class="tiebreaker-header">
                  <span class="tiebreaker-icon">⚔️</span>
                  <span class="tiebreaker-title">一号种子加赛 (BO5)</span>
                </div>
                <div class="tiebreaker-teams">
                  <div class="team-side">
                    <span class="team-name">{{ icpTournament.final.tiebreakerMatch.teamAName }}</span>
                    <el-tag :type="getRegionTagType(icpTournament.final.tiebreakerMatch.teamARegion)" size="small">
                      {{ icpTournament.final.tiebreakerMatch.teamARegion }}
                    </el-tag>
                  </div>
                  <div class="vs-section">
                    <template v-if="icpTournament.final.tiebreakerMatch.status === 'completed'">
                      <span class="score">{{ icpTournament.final.tiebreakerMatch.scoreA }}</span>
                      <span class="vs">:</span>
                      <span class="score">{{ icpTournament.final.tiebreakerMatch.scoreB }}</span>
                    </template>
                    <span v-else class="vs">VS</span>
                  </div>
                  <div class="team-side">
                    <el-tag :type="getRegionTagType(icpTournament.final.tiebreakerMatch.teamBRegion)" size="small">
                      {{ icpTournament.final.tiebreakerMatch.teamBRegion }}
                    </el-tag>
                    <span class="team-name">{{ icpTournament.final.tiebreakerMatch.teamBName }}</span>
                  </div>
                </div>
                <div class="tiebreaker-actions">
                  <el-button
                    v-if="icpTournament.final.tiebreakerMatch.status !== 'completed'"
                    type="danger"
                    @click="handleSimulateTiebreaker(icpTournament.final)"
                  >
                    <el-icon><DArrowRight /></el-icon>
                    模拟加赛
                  </el-button>
                  <el-button
                    v-else
                    type="info"
                    @click="viewMatchDetails(icpTournament.final.tiebreakerMatch)"
                  >
                    查看详情
                  </el-button>
                </div>
              </div>
            </div>
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
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Flag,
  ArrowLeft,
  DArrowRight,
  User
} from '@element-plus/icons-vue'
import ICPSeedGroupStanding from '@/components/icp/ICPSeedGroupStanding.vue'
import ICPRegionBattleCard from '@/components/icp/ICPRegionBattleCard.vue'
import MatchDetailDialog from '@/components/match/MatchDetailDialog.vue'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { internationalApi, matchApi, teamApi, financeApi } from '@/api/tauri'
import type { BracketInfo, GroupStandingInfo, DetailedGameResult, PlayerGameStats, DetailedMatchResult } from '@/api/tauri'
import type { PlayerPosition } from '@/types/player'
import type { MatchDetail } from '@/types/matchDetail'
import type { ICPTournament, ICPSeedGroup, ICPMatch, ICPRegionStats, ICPRegionMatch, ICPGroupStanding } from '@/types/icp'

const router = useRouter()
const gameStore = useGameStore()
const timeStore = useTimeStore()

// Stores
const matchDetailStore = useMatchDetailStore()
const playerStore = usePlayerStore()

// 阶段检查
const ICP_PHASE = 'ICP_INTERCONTINENTAL'
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
  const targetIndex = phaseOrder.indexOf(ICP_PHASE)
  return currentIndex < targetIndex
})

const currentPhaseDisplay = computed(() => timeStore.phaseDisplayName)

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

// ICP赛事数据 - 从后端加载
const icpTournament = reactive<ICPTournament>({
  id: '',
  seasonYear: 2024,
  status: 'not_started',
  seedGroups: [],
  regionStats: [],
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
 * 获取赛区标签类型
 */
const getRegionTagType = (region?: string) => {
  const typeMap: Record<string, any> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning'
  }
  return typeMap[region || ''] || 'info'
}

/**
 * 按种子号分组的队伍数据
 */
const seedTeamsGrouped = computed(() => {
  const grouped: Record<number, Array<{ teamId: string, teamName: string, region: string }>> = {
    1: [],
    2: [],
    3: [],
    4: []
  }

  // 从种子组数据中提取队伍
  icpTournament.seedGroups.forEach(group => {
    const seedNumber = group.seedNumber || (group.groupName.charCodeAt(0) - 'A'.charCodeAt(0) + 1)

    if (group.standings && group.standings.length > 0) {
      group.standings.forEach(team => {
        if (!grouped[seedNumber]) {
          grouped[seedNumber] = []
        }
        // 避免重复添加
        if (!grouped[seedNumber].some(t => t.teamId === String(team.teamId))) {
          grouped[seedNumber].push({
            teamId: String(team.teamId),
            teamName: team.teamName,
            region: team.region || ''
          })
        }
      })
    }
  })

  return grouped
})

/**
 * 查看比赛详情
 */
const viewMatchDetails = async (match: ICPMatch) => {
  if (match.status === 'completed') {
    // 优先使用 backendMatchId（数据库中的实际ID）
    const matchIdForLookup = match.backendMatchId || match.id

    // 先尝试从内存获取（同时检查两种ID）
    let detail = matchDetailStore.getMatchDetail(matchIdForLookup) || matchDetailStore.getMatchDetail(match.id)
    if (detail) {
      currentMatchDetail.value = detail
      showMatchDetailDialog.value = true
      return
    }
    // 如果内存中没有，尝试从数据库加载（使用后端ID）
    if (match.backendMatchId) {
      detail = await matchDetailStore.loadMatchDetailFromDb(match.backendMatchId)
      if (detail) {
        currentMatchDetail.value = detail
        showMatchDetailDialog.value = true
        return
      }
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
  // 从 teamMap 获取队伍名称
  const homeTeamId = result.home_team_id.toString()
  const awayTeamId = result.away_team_id.toString()
  const homeTeamName = teamMap.value.get(result.home_team_id)?.name || `Team ${result.home_team_id}`
  const awayTeamName = teamMap.value.get(result.away_team_id)?.name || `Team ${result.away_team_id}`

  const games = result.games.map((game: DetailedGameResult, idx: number) => {
    // 计算队伍战力（选手actual_ability的平均值 - 每局不同）
    const teamAPower = game.home_players.length > 0
      ? game.home_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.home_players.length
      : 0
    const teamBPower = game.away_players.length > 0
      ? game.away_players.reduce((sum, p) => sum + (p.actual_ability || p.base_ability), 0) / game.away_players.length
      : 0

    // 计算战力差和判断是否爆冷
    const powerDifference = teamAPower - teamBPower
    const winnerId = game.winner_id.toString()
    const winnerName = winnerId === homeTeamId ? homeTeamName : awayTeamName
    // 爆冷：战力低的队伍赢了
    const isUpset = (powerDifference > 0 && winnerId !== homeTeamId) ||
                    (powerDifference < 0 && winnerId !== awayTeamId)

    return {
      gameNumber: idx + 1,
      winnerId,
      winnerName,
      duration: game.duration_minutes,
      // 添加队伍ID和名称，用于保存到数据库
      teamAId: homeTeamId,
      teamAName: homeTeamName,
      teamBId: awayTeamId,
      teamBName: awayTeamName,
      // 添加队伍战力
      teamAPower,
      teamBPower,
      powerDifference,
      isUpset,
      teamAPlayers: game.home_players.map((p: PlayerGameStats) => ({
        playerId: p.player_id.toString(),
        playerName: p.player_name,
        teamId: homeTeamId, // 添加 teamId
        position: p.position as PlayerPosition,
        baseAbility: p.base_ability,
        actualAbility: p.actual_ability,
        impactScore: p.impact_score,
        conditionBonus: p.condition_bonus, // 修正字段名
        stability: 0,
        stabilityNoise: p.stability_noise,
        kills: p.kills,
        deaths: p.deaths,
        assists: p.assists,
        gold: p.gold,
        damageDealt: p.damage_dealt, // 修正字段名
        cs: p.cs,
        visionScore: p.vision_score,
        traits: p.traits as any[],
        activatedTraits: p.activated_traits?.map(t => ({
          type: t.trait_type as any,
          name: t.name,
          effect: t.effect,
          value: t.value,
          isPositive: t.is_positive
        }))
      })),
      teamBPlayers: game.away_players.map((p: PlayerGameStats) => ({
        playerId: p.player_id.toString(),
        playerName: p.player_name,
        teamId: awayTeamId, // 添加 teamId
        position: p.position as PlayerPosition,
        baseAbility: p.base_ability,
        actualAbility: p.actual_ability,
        impactScore: p.impact_score,
        conditionBonus: p.condition_bonus, // 修正字段名
        stability: 0,
        stabilityNoise: p.stability_noise,
        kills: p.kills,
        deaths: p.deaths,
        assists: p.assists,
        gold: p.gold,
        damageDealt: p.damage_dealt, // 修正字段名
        cs: p.cs,
        visionScore: p.vision_score,
        traits: p.traits as any[],
        activatedTraits: p.activated_traits?.map(t => ({
          type: t.trait_type as any,
          name: t.name,
          effect: t.effect,
          value: t.value,
          isPositive: t.is_positive
        }))
      })),
      teamAPerformance: game.home_performance,
      teamBPerformance: game.away_performance,
      performanceDifference: game.home_performance - game.away_performance,
      gameNoise: 0,
      mvpPlayerId: game.game_mvp?.player_id?.toString(),
      mvpPlayerName: game.game_mvp?.player_name,
      mvpTeamId: game.game_mvp?.team_id?.toString()
    }
  })

  return {
    matchId,
    teamAId: result.home_team_id.toString(),
    teamAName: homeTeamName,
    teamBId: result.away_team_id.toString(),
    teamBName: awayTeamName,
    bestOf: 3, // ICP 种子组赛为 BO3
    finalScoreA: result.home_score,
    finalScoreB: result.away_score,
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
  if (tournamentId.value) {
    try {
      // 优先使用已保存的 backendMatchId，否则尝试查找
      const backendMatchId = match.backendMatchId || findBackendMatchId(match)

      console.log('[handleSimulateMatch]', {
        matchId: match.id,
        backendMatchId,
        teamA: match.teamAName,
        teamB: match.teamBName
      })

      if (backendMatchId) {
        const result = await matchApi.simulateMatchDetailed(backendMatchId)

        if (result) {
          // 转换为前端格式（使用后端ID作为matchId以便后续加载）
          const matchDetail = convertToMatchDetail(result, String(backendMatchId))

          // 更新比赛状态
          match.backendMatchId = backendMatchId // 保存后端ID用于后续加载详情
          match.scoreA = result.home_score
          match.scoreB = result.away_score
          match.winnerId = result.winner_id.toString()
          match.status = 'completed'
          match.completedAt = new Date()

          // 保存比赛详情（使用后端ID）
          matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)

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

          ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

          // 检查该组是否完成
          checkGroupCompletion()
          return
        }
      }
    } catch (error) {
      console.warn('后端 API 模拟失败，使用本地引擎:', error)
    }
  }

  // 没有后端 matchId 时抛出错误
  ElMessage.error('无法找到后端比赛ID，请确保赛事数据已正确初始化')
}

/**
 * 查找后端对应的 matchId
 * @param match 前端比赛对象
 * @param stagePrefix 可选的阶段前缀，用于限定搜索范围（如 'ICP_FINAL', 'ICP_SEMI'）
 */
const findBackendMatchId = (match: ICPMatch, stagePrefix?: string): number | null => {
  if (!bracketData.value) {
    console.warn('[findBackendMatchId] bracketData is null')
    return null
  }

  // 在所有比赛中查找匹配的
  let allMatches = bracketData.value.matches || []

  // 如果指定了阶段前缀，只在该阶段的比赛中查找
  if (stagePrefix) {
    allMatches = allMatches.filter(m => m.stage?.startsWith(stagePrefix))
  }

  console.log('[findBackendMatchId] Looking for match:', {
    teamAName: match.teamAName,
    teamBName: match.teamBName,
    teamAId: match.teamAId,
    teamBId: match.teamBId,
    stage: match.stage,
    stagePrefix,
    totalBackendMatches: allMatches.length
  })

  // 方法1: 按队伍名称匹配（适用于所有比赛）
  for (const m of allMatches) {
    const homeTeamName = m.home_team?.short_name || m.home_team?.name || ''
    const awayTeamName = m.away_team?.short_name || m.away_team?.name || ''

    if ((homeTeamName === match.teamAName && awayTeamName === match.teamBName) ||
        (homeTeamName === match.teamBName && awayTeamName === match.teamAName)) {
      console.log('[findBackendMatchId] Found by name:', m.match_id)
      return m.match_id
    }
  }

  // 方法2: 按队伍 ID 匹配（适用于已填充队伍的淘汰赛比赛）
  if (match.teamAId && match.teamBId) {
    for (const m of allMatches) {
      const homeTeamId = m.home_team?.id
      const awayTeamId = m.away_team?.id

      if ((homeTeamId === Number(match.teamAId) && awayTeamId === Number(match.teamBId)) ||
          (homeTeamId === Number(match.teamBId) && awayTeamId === Number(match.teamAId))) {
        console.log('[findBackendMatchId] Found by ID:', m.match_id)
        return m.match_id
      }
    }
  }

  // 如果匹配失败，打印调试信息
  console.warn('[findBackendMatchId] No match found. Backend matches:')
  allMatches.slice(0, 10).forEach((m, idx) => {
    console.log(`  [${idx}] stage: ${m.stage}, home: ${m.home_team?.id}(${m.home_team?.short_name || m.home_team?.name}), away: ${m.away_team?.id}(${m.away_team?.short_name || m.away_team?.name})`)
  })

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
      if (b.wins !== a.wins) return b.wins - a.wins
      // 使用 teamId 作为最终 tiebreaker 确保稳定排序
      return parseInt(String(a.teamId)) - parseInt(String(b.teamId))
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
    // 关键：只有当第二名和第三名可以明确区分时，才能跳过半决赛直接决赛
    const canSkipSemifinal = sortedRegions[1].totalBadges > sortedRegions[2].totalBadges

    console.log('[ICP] 赛区排名:', sortedRegions.map(r => `${r.region}:${r.totalBadges}`).join(', '))
    console.log('[ICP] 是否可以跳过半决赛:', canSkipSemifinal)

    if (canSkipSemifinal) {
      // 第二名徽章数 > 第三名，可以明确区分前两名，直接进行决赛
      icpTournament.final = createRegionBattle(
        sortedRegions[0],
        sortedRegions[1],
        'final'
      )
      // 填充后端决赛比赛队伍
      await fillKnockoutMatchTeams('ICP_FINAL', icpTournament.final)
      // 标记半决赛比赛为已取消
      await cancelUnusedMatches('ICP_SEMI')
    } else {
      // 第二名和第三名徽章数相同，需要半决赛来决定谁进入决赛
      icpTournament.semifinal = createRegionBattle(
        sortedRegions[1],
        sortedRegions[2],
        'semifinal'
      )
      // 填充后端半决赛比赛队伍
      await fillKnockoutMatchTeams('ICP_SEMI', icpTournament.semifinal)
      // 决赛待定（半决赛胜者 vs 第一名）
    }

    icpTournament.status = 'region_battle'
    ElMessage.success('赛区对决生成成功！')
  } finally {
    generatingRegionBattle.value = false
  }
}

/**
 * 填充后端淘汰赛比赛的队伍
 */
const fillKnockoutMatchTeams = async (stagePrefix: string, regionBattle: ICPRegionMatch) => {
  if (!bracketData.value) return

  const allMatches = bracketData.value.matches || []

  // 遍历赛区对决的每场比赛（4 场，对应 4 个种子位置）
  for (let i = 0; i < regionBattle.matches.length; i++) {
    const match = regionBattle.matches[i]
    const seed = i + 1
    const targetStage = `${stagePrefix}_${seed}`

    // 找到后端对应的比赛
    const backendMatch = allMatches.find(m => m.stage === targetStage)

    if (backendMatch && match.teamAId && match.teamBId) {
      // 保存后端ID用于后续加载比赛详情
      match.backendMatchId = backendMatch.match_id

      try {
        await matchApi.updateMatchTeams(
          backendMatch.match_id,
          Number(match.teamAId),
          Number(match.teamBId)
        )
        console.log(`[fillKnockoutMatchTeams] Updated ${targetStage}: ${match.teamAName} vs ${match.teamBName}, backendMatchId=${backendMatch.match_id}`)
      } catch (error) {
        console.error(`[fillKnockoutMatchTeams] Failed to update ${targetStage}:`, error)
      }
    }
  }

  // 刷新 bracketData 以获取更新后的队伍信息
  if (tournamentId.value) {
    const bracket = await internationalApi.getTournamentBracket(tournamentId.value)
    if (bracket) {
      bracketData.value = bracket
    }
  }
}

/**
 * 取消不需要的比赛（标记为 CANCELLED）
 */
const cancelUnusedMatches = async (stagePrefix: string) => {
  if (!bracketData.value) return

  const allMatches = bracketData.value.matches || []

  // 找到所有匹配前缀的比赛
  const matchesToCancel = allMatches.filter(m =>
    m.stage?.startsWith(stagePrefix) && m.status?.toUpperCase() !== 'COMPLETED'
  )

  console.log(`[ICP] 取消 ${matchesToCancel.length} 场不需要的比赛 (${stagePrefix})`)

  for (const match of matchesToCancel) {
    try {
      await matchApi.cancelMatch(match.match_id)
      console.log(`[ICP] 已取消比赛: ${match.stage} (ID: ${match.match_id})`)
    } catch (error) {
      console.warn(`[ICP] 取消比赛失败: ${match.stage}`, error)
    }
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
  if (tournamentId.value) {
    try {
      // 根据赛区对决阶段确定stage前缀
      const stagePrefix = battle.stage === 'semifinal' ? 'ICP_SEMI' : 'ICP_FINAL'
      // 优先使用已保存的 backendMatchId，否则尝试查找（限定在对应阶段）
      const backendMatchId = match.backendMatchId || findBackendMatchId(match, stagePrefix)

      console.log('[handleSimulateRegionMatch]', {
        matchId: match.id,
        backendMatchId,
        stagePrefix,
        teamA: match.teamAName,
        teamB: match.teamBName
      })

      if (backendMatchId) {
        const result = await matchApi.simulateMatchDetailed(backendMatchId)

        if (result) {
          const matchDetail = convertToMatchDetail(result, String(backendMatchId))

          match.backendMatchId = backendMatchId // 保存后端ID用于后续加载详情
          match.scoreA = result.home_score
          match.scoreB = result.away_score
          match.winnerId = result.winner_id.toString()
          match.status = 'completed'
          match.completedAt = new Date()

          matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)

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
            if (result.home_score > result.away_score) battle.regionAWins++
            else battle.regionBWins++
          } else {
            if (result.home_score > result.away_score) battle.regionBWins++
            else battle.regionAWins++
          }

          ElMessage.success(`比赛完成: ${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

          checkRegionBattleCompletion(battle)
          return
        }
      }
    } catch (error) {
      console.warn('后端 API 模拟失败:', error)
    }
  }

  // 没有后端 matchId 时抛出错误
  ElMessage.error('无法找到后端比赛ID，请确保赛事数据已正确初始化')
}

/**
 * 检查赛区对决是否结束
 */
const checkRegionBattleCompletion = async (battle: ICPRegionMatch) => {
  // 检查是否有赛区已经赢得3场（BO4取胜条件）
  if (battle.regionAWins >= 3) {
    battle.winnerId = battle.regionA
    battle.status = 'completed'
    await cancelUnusedTiebreaker(battle)
    await checkTournamentCompletion()
    return
  }
  if (battle.regionBWins >= 3) {
    battle.winnerId = battle.regionB
    battle.status = 'completed'
    await cancelUnusedTiebreaker(battle)
    await checkTournamentCompletion()
    return
  }

  // 检查是否所有比赛都已完成
  const allComplete = battle.matches.every(m => m.status === 'completed')

  if (allComplete) {
    // 判定胜者
    if (battle.regionAWins > battle.regionBWins) {
      battle.winnerId = battle.regionA
      battle.status = 'completed'
      await cancelUnusedTiebreaker(battle)
      await checkTournamentCompletion()
    } else if (battle.regionBWins > battle.regionAWins) {
      battle.winnerId = battle.regionB
      battle.status = 'completed'
      await cancelUnusedTiebreaker(battle)
      await checkTournamentCompletion()
    } else {
      // 2:2 平局，需要进行一号种子加赛
      await setupTiebreakerMatch(battle)
    }
  }
}

/**
 * 取消不需要的加赛比赛
 */
const cancelUnusedTiebreaker = async (battle: ICPRegionMatch) => {
  if (!bracketData.value) return

  const tiebreakerStage = battle.stage === 'semifinal' ? 'ICP_SEMI_TIEBREAKER' : 'ICP_FINAL_TIEBREAKER'
  const tiebreakerMatch = bracketData.value.matches?.find(m => m.stage === tiebreakerStage)

  if (tiebreakerMatch && tiebreakerMatch.status !== 'Completed' && tiebreakerMatch.status !== 'COMPLETED') {
    try {
      await matchApi.cancelMatch(tiebreakerMatch.match_id)
      console.log(`[ICP] 已取消不需要的加赛: ${tiebreakerStage}`)
    } catch (error) {
      console.error('[ICP] 取消加赛失败:', error)
    }
  }
}

/**
 * 设置加赛（2:2平局时调用）
 */
const setupTiebreakerMatch = async (battle: ICPRegionMatch) => {
  // 找到一号种子的队伍（从第一场比赛获取）
  const seed1Match = battle.matches[0] // 第一场是一号种子对决

  if (!seed1Match) {
    console.error('无法找到一号种子比赛')
    return
  }

  // 确定加赛的阶段名
  const tiebreakerStage = battle.stage === 'semifinal' ? 'ICP_SEMI_TIEBREAKER' : 'ICP_FINAL_TIEBREAKER'

  // 创建加赛比赛对象
  battle.tiebreakerMatch = {
    id: `${battle.stage}-tiebreaker`,
    teamAId: seed1Match.teamAId,
    teamAName: seed1Match.teamAName,
    teamARegion: seed1Match.teamARegion,
    teamBId: seed1Match.teamBId,
    teamBName: seed1Match.teamBName,
    teamBRegion: seed1Match.teamBRegion,
    scoreA: 0,
    scoreB: 0,
    winnerId: null,
    status: 'scheduled',
    bestOf: 5,
    stage: battle.stage
  }

  // 找到后端加赛比赛并填充队伍
  if (bracketData.value) {
    const backendMatch = bracketData.value.matches?.find(m => m.stage === tiebreakerStage)
    if (backendMatch && seed1Match.teamAId && seed1Match.teamBId) {
      battle.tiebreakerMatch.backendMatchId = backendMatch.match_id

      try {
        await matchApi.updateMatchTeams(
          backendMatch.match_id,
          Number(seed1Match.teamAId),
          Number(seed1Match.teamBId)
        )
        console.log(`[setupTiebreakerMatch] 已设置加赛队伍: ${seed1Match.teamAName} vs ${seed1Match.teamBName}`)
      } catch (error) {
        console.error('[setupTiebreakerMatch] 设置加赛队伍失败:', error)
      }
    }
  }

  battle.status = 'tiebreaker'
  ElMessage.warning('比分 2:2 平局！一号种子需要进行加赛决出胜者。')
}

/**
 * 模拟加赛
 */
const handleSimulateTiebreaker = async (battle: ICPRegionMatch) => {
  if (!battle.tiebreakerMatch) {
    ElMessage.error('加赛比赛不存在')
    return
  }

  const match = battle.tiebreakerMatch
  const backendMatchId = match.backendMatchId

  if (!backendMatchId) {
    ElMessage.error('无法找到加赛后端比赛ID')
    return
  }

  try {
    const result = await matchApi.simulateMatchDetailed(backendMatchId)

    if (result) {
      const matchDetail = convertToMatchDetail(result, String(backendMatchId))

      match.backendMatchId = backendMatchId
      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = result.winner_id.toString()
      match.status = 'completed'
      match.completedAt = new Date()

      matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)

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
      if (tournamentId.value) {
        await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
      }

      // 根据加赛结果判定赛区胜者
      if (match.teamARegion === battle.regionA) {
        battle.winnerId = result.home_score > result.away_score ? battle.regionA : battle.regionB
      } else {
        battle.winnerId = result.home_score > result.away_score ? battle.regionB : battle.regionA
      }

      battle.status = 'completed'
      ElMessage.success(`加赛完成！${match.teamAName} ${result.home_score} - ${result.away_score} ${match.teamBName}`)

      await checkTournamentCompletion()
    }
  } catch (error) {
    console.error('模拟加赛失败:', error)
    ElMessage.error('模拟加赛失败')
  }
}

/**
 * 检查比赛是否整体结束
 */
const checkTournamentCompletion = async () => {
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
      // 填充后端决赛比赛队伍
      await fillKnockoutMatchTeams('ICP_FINAL', icpTournament.final)
      ElMessage.success('半决赛完成！决赛已生成')
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
    if (icpTournament.semifinal) {
      if (icpTournament.semifinal.status === 'tiebreaker' && icpTournament.semifinal.tiebreakerMatch) {
        // 如果处于加赛状态，直接模拟加赛
        await simulateTiebreakerInternal(icpTournament.semifinal)
      } else if (icpTournament.semifinal.status !== 'completed') {
        await simulateRegionBattleInternal(icpTournament.semifinal)
      }
    }

    // 模拟决赛
    if (icpTournament.final) {
      if (icpTournament.final.status === 'tiebreaker' && icpTournament.final.tiebreakerMatch) {
        // 如果处于加赛状态，直接模拟加赛
        await simulateTiebreakerInternal(icpTournament.final)
      } else if (icpTournament.final.status !== 'completed') {
        await simulateRegionBattleInternal(icpTournament.final)
      }
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
 * 内部模拟单场比赛 - 使用后端 API
 */
const simulateMatchInternal = async (match: ICPMatch) => {
  // 优先使用已保存的 backendMatchId，否则尝试查找
  const backendMatchId = match.backendMatchId || findBackendMatchId(match)

  if (!backendMatchId) {
    console.error('无法找到后端比赛ID:', match.id, match.teamAName, 'vs', match.teamBName)
    return
  }

  try {
    const result = await matchApi.simulateMatchDetailed(backendMatchId)

    if (result) {
      const matchDetail = convertToMatchDetail(result, String(backendMatchId))

      match.backendMatchId = backendMatchId // 保存后端ID用于后续加载详情
      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = result.winner_id.toString()
      match.status = 'completed'
      match.completedAt = new Date()

      matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)

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
      if (tournamentId.value) {
        await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
      }

      updateGroupStandings(match)
      checkGroupCompletion()
    }
  } catch (error) {
    console.error('模拟比赛失败:', error)
    ElMessage.error('模拟比赛失败')
  }
}

/**
 * 内部模拟赛区对决 - 使用后端 API
 */
const simulateRegionBattleInternal = async (battle: ICPRegionMatch) => {
  // 根据赛区对决阶段确定stage前缀
  const stagePrefix = battle.stage === 'semifinal' ? 'ICP_SEMI' : 'ICP_FINAL'

  for (const match of battle.matches) {
    // 检查是否已经有赛区赢得3场（BO4取胜条件）
    if (battle.regionAWins >= 3 || battle.regionBWins >= 3) {
      console.log(`[ICP] 赛区对决提前结束: ${battle.regionAName} ${battle.regionAWins} - ${battle.regionBWins} ${battle.regionBName}`)
      break
    }

    if (match.status !== 'completed') {
      // 优先使用已保存的 backendMatchId，否则尝试查找（限定在对应阶段）
      const backendMatchId = match.backendMatchId || findBackendMatchId(match, stagePrefix)

      if (!backendMatchId) {
        console.error('无法找到后端比赛ID:', match.id, match.teamAName, 'vs', match.teamBName)
        continue
      }

      try {
        const result = await matchApi.simulateMatchDetailed(backendMatchId)

        if (result) {
          const matchDetail = convertToMatchDetail(result, String(backendMatchId))

          match.backendMatchId = backendMatchId // 保存后端ID用于后续加载详情
          match.scoreA = result.home_score
          match.scoreB = result.away_score
          match.winnerId = result.winner_id.toString()
          match.status = 'completed'
          match.completedAt = new Date()

          matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)

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
          if (tournamentId.value) {
            await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
          }

          // 更新赛区胜场
          if (match.teamARegion === battle.regionA) {
            if (result.home_score > result.away_score) battle.regionAWins++
            else battle.regionBWins++
          } else {
            if (result.home_score > result.away_score) battle.regionBWins++
            else battle.regionAWins++
          }
        }
      } catch (error) {
        console.error('模拟比赛失败:', error)
      }

      await new Promise(resolve => setTimeout(resolve, 100))
    }
  }

  // 判定胜者（检查是否有赛区已赢得3场）
  if (battle.regionAWins >= 3 || battle.regionAWins > battle.regionBWins) {
    battle.winnerId = battle.regionA
    battle.status = 'completed'
    await cancelUnusedTiebreaker(battle)
    await checkTournamentCompletion()
  } else if (battle.regionBWins >= 3 || battle.regionBWins > battle.regionAWins) {
    battle.winnerId = battle.regionB
    battle.status = 'completed'
    await cancelUnusedTiebreaker(battle)
    await checkTournamentCompletion()
  } else {
    // 2:2 平局，需要进行加赛
    await setupTiebreakerMatch(battle)
    // 自动模拟加赛
    if (battle.tiebreakerMatch) {
      await simulateTiebreakerInternal(battle)
    }
  }
}

/**
 * 内部模拟加赛 - 用于批量模拟
 */
const simulateTiebreakerInternal = async (battle: ICPRegionMatch) => {
  if (!battle.tiebreakerMatch) return

  const match = battle.tiebreakerMatch
  const backendMatchId = match.backendMatchId

  if (!backendMatchId) {
    console.error('无法找到加赛后端比赛ID')
    return
  }

  try {
    const result = await matchApi.simulateMatchDetailed(backendMatchId)

    if (result) {
      const matchDetail = convertToMatchDetail(result, String(backendMatchId))

      match.scoreA = result.home_score
      match.scoreB = result.away_score
      match.winnerId = result.winner_id.toString()
      match.status = 'completed'
      match.completedAt = new Date()

      matchDetailStore.saveMatchDetail(backendMatchId, matchDetail)

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
      if (tournamentId.value) {
        await internationalApi.advanceBracket(tournamentId.value, backendMatchId, result.winner_id)
      }

      // 根据加赛结果判定赛区胜者
      if (match.teamARegion === battle.regionA) {
        battle.winnerId = result.home_score > result.away_score ? battle.regionA : battle.regionB
      } else {
        battle.winnerId = result.home_score > result.away_score ? battle.regionB : battle.regionA
      }

      battle.status = 'completed'
      await checkTournamentCompletion()
    }
  } catch (error) {
    console.error('模拟加赛失败:', error)
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
      console.log('ICP赛事奖金已发放')
    } catch (e) {
      console.error('发放奖金失败:', e)
    }
  }

  ElMessageBox.alert(
    `恭喜 ${championName} 成为ICP洲际对抗赛最强赛区！\n\n` +
    `✅ 奖金已发放到各战队账户\n` +
    `💡 请在时间控制面板完成阶段推进，系统将自动颁发荣誉和年度积分`,
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
    // 先刷新时间状态，确保阶段检查是最新的
    await timeStore.fetchTimeState()

    // 获取当前存档和赛季
    const currentSave = gameStore.currentSave
    if (!currentSave) {
      console.warn('未找到当前存档')
      return
    }

    const seasonId = currentSave.current_season || 1

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
    const teams = await teamApi.getAllTeams()
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
    }

    // 获取小组积分榜
    const standings = await internationalApi.getGroupStandings(tournamentId.value)
    if (standings) {
      groupStandings.value = standings
    }

    // 从后端数据初始化种子组（必须同时有对阵和积分榜数据）
    if (bracket && standings && standings.length > 0) {
      initializeSeedGroupsFromBackend(bracket, standings)

      // 从后端恢复赛区对决状态（半决赛/决赛）
      restoreRegionBattleFromBackend(bracket)

      // 如果决赛需要填充队伍（半决赛已完成但决赛未设置的情况）
      if (icpTournament.final && (icpTournament.final as any)._needsFillTeams) {
        delete (icpTournament.final as any)._needsFillTeams
        await fillKnockoutMatchTeams('ICP_FINAL', icpTournament.final)
      }
    } else {
      console.warn('[ICP] 数据不足，无法初始化种子组:', {
        hasBracket: !!bracket,
        standingsCount: standings?.length || 0
      })
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
const _convertBracketToICPFormat = (bracket: BracketInfo) => {
  if (!bracket.matches) return

  // 按阶段分类比赛
  const groupMatches = bracket.matches.filter(m => m.stage === 'group' || m.stage === 'Group')
  const knockoutMatches = bracket.matches.filter(m => m.stage !== 'group' && m.stage !== 'Group')

  // 更新种子组比赛数据
  groupMatches.forEach(match => {
    // 找到对应的前端比赛
    for (const group of icpTournament.seedGroups) {
      const frontendMatch = group.matches.find(m => {
        // 使用正确的字段名：home_team 和 away_team
        const homeTeamName = match.home_team?.name || match.home_team?.short_name || ''
        const awayTeamName = match.away_team?.name || match.away_team?.short_name || ''
        return (m.teamAName === homeTeamName && m.teamBName === awayTeamName) ||
               (m.teamAName === awayTeamName && m.teamBName === homeTeamName)
      })

      if (frontendMatch && match.winner_id) {
        frontendMatch.status = 'completed'
        frontendMatch.scoreA = match.home_score || 0
        frontendMatch.scoreB = match.away_score || 0
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
 * 从后端数据初始化种子组
 */
const initializeSeedGroupsFromBackend = (bracket: BracketInfo, standings: GroupStandingInfo[]) => {
  console.log('[ICP] initializeSeedGroupsFromBackend - standings:', standings)
  console.log('[ICP] initializeSeedGroupsFromBackend - bracket matches:', bracket.matches?.length)

  // 清空现有数据
  icpTournament.seedGroups = []
  icpTournament.regionStats = []

  // 首先从对阵数据构建队伍ID到赛区代码的映射
  const teamRegionMap = new Map<number, string>()
  if (bracket.matches) {
    bracket.matches.forEach(match => {
      if (match.home_team) {
        teamRegionMap.set(match.home_team.id, match.home_team.region_code || '')
      }
      if (match.away_team) {
        teamRegionMap.set(match.away_team.id, match.away_team.region_code || '')
      }
    })
  }
  console.log('[ICP] teamRegionMap size:', teamRegionMap.size)

  // 从积分榜构建种子组
  // GroupStandingInfo 包含 { group_name, teams: TeamGroupStats[] }
  const groupMap = new Map<string, { teams: any[], matches: any[] }>()

  // 先从积分榜获取队伍信息
  standings.forEach(groupStanding => {
    // 处理 group_name: "ICP_A" -> "A", "ICP_GROUP_A" -> "A", "A" -> "A"
    let groupName = groupStanding.group_name || 'A'
    groupName = groupName.replace('ICP_GROUP_', '').replace('ICP_', '').replace('GROUP_', '')

    if (!groupMap.has(groupName)) {
      groupMap.set(groupName, { teams: [], matches: [] })
    }

    // 遍历该组的所有队伍
    if (groupStanding.teams) {
      groupStanding.teams.forEach(teamStats => {
        // 从对阵数据获取赛区代码
        const regionCode = teamRegionMap.get(teamStats.team_id) || ''

        groupMap.get(groupName)!.teams.push({
          teamId: String(teamStats.team_id),
          teamName: teamStats.team_name,
          region: regionCode,
          wins: teamStats.wins || 0,
          losses: teamStats.losses || 0,
          points: teamStats.points || 0,
          gamesWon: teamStats.games_won || 0,
          gamesLost: teamStats.games_lost || 0,
          position: 0,
          hasBadge: false
        })
      })
    }
  })

  // 从对阵数据获取比赛信息 - 只处理种子组比赛
  if (bracket.matches) {
    bracket.matches.forEach(match => {
      const stage = match.stage || ''

      // 只处理种子组比赛 (ICP_GROUP_A, ICP_GROUP_B, ICP_GROUP_C, ICP_GROUP_D)
      if (!stage.startsWith('ICP_GROUP_')) {
        return // 跳过非种子组比赛（半决赛、决赛等）
      }

      // 解析 stage: "ICP_GROUP_A" -> "A"
      const groupName = stage.replace('ICP_GROUP_', '')

      // 只接受有效的组名 (A, B, C, D)
      if (!['A', 'B', 'C', 'D'].includes(groupName)) {
        return
      }

      if (!groupMap.has(groupName)) {
        groupMap.set(groupName, { teams: [], matches: [] })
      }

      const homeTeam = match.home_team
      const awayTeam = match.away_team

      if (homeTeam && awayTeam) {
        const matchStatus = (match.status || '').toUpperCase()
        const isCompleted = matchStatus === 'COMPLETED'

        groupMap.get(groupName)!.matches.push({
          id: String(match.match_id),
          backendMatchId: match.match_id, // 存储后端数字ID用于加载比赛详情
          groupName: groupName,
          teamAId: String(homeTeam.id),
          teamAName: homeTeam.short_name || homeTeam.name,
          teamARegion: homeTeam.region_code || '',
          teamBId: String(awayTeam.id),
          teamBName: awayTeam.short_name || awayTeam.name,
          teamBRegion: awayTeam.region_code || '',
          scoreA: match.home_score || 0,
          scoreB: match.away_score || 0,
          winnerId: match.winner_id ? String(match.winner_id) : null,
          status: isCompleted ? 'completed' : 'scheduled',
          bestOf: 3,
          stage: 'group'
        })
      }
    })
  }

  // 构建种子组数据
  const seedGroups: ICPSeedGroup[] = []
  const regionStatsMap = new Map<string, ICPRegionStats>()

  // 按组名排序 (A, B, C, D)
  const sortedGroupNames = Array.from(groupMap.keys()).sort()

  sortedGroupNames.forEach(groupName => {
    const groupData = groupMap.get(groupName)!
    const teams = groupData.teams
    const matches = groupData.matches

    // 排序队伍（按积分、净胜场、胜场、teamId确保稳定）
    teams.sort((a: any, b: any) => {
      if (b.points !== a.points) return b.points - a.points
      const aDiff = a.gamesWon - a.gamesLost
      const bDiff = b.gamesWon - b.gamesLost
      if (bDiff !== aDiff) return bDiff - aDiff
      if (b.wins !== a.wins) return b.wins - a.wins
      // 使用 teamId 作为最终 tiebreaker 确保稳定排序（与后端一致）
      return parseInt(String(a.teamId)) - parseInt(String(b.teamId))
    })

    // 设置排名和徽章
    teams.forEach((team: any, index: number) => {
      team.position = index + 1
      team.hasBadge = index < 2 // 前两名获得徽章
    })

    // 检查组是否完成
    const isComplete = matches.length > 0 && matches.every((m: any) => m.status === 'completed')

    // 计算种子号: A=1, B=2, C=3, D=4
    const seedNumber = groupName.charCodeAt(0) - 'A'.charCodeAt(0) + 1

    // 构建积分榜
    const standings: ICPGroupStanding[] = teams.map((team: any) => ({
      teamId: team.teamId,
      teamName: team.teamName,
      region: team.region,
      seed: seedNumber,
      matchesPlayed: team.wins + team.losses,
      wins: team.wins,
      losses: team.losses,
      points: team.points,
      roundsWon: team.gamesWon,
      roundsLost: team.gamesLost,
      roundDifferential: team.gamesWon - team.gamesLost,
      position: team.position,
      hasBadge: team.hasBadge
    }))

    seedGroups.push({
      groupName: groupName as 'A' | 'B' | 'C' | 'D',
      seedNumber,
      teams: teams.map((t: any) => ({
        id: t.teamId,
        name: t.teamName,
        region: t.region
      })),
      matches,
      standings,
      isComplete
    })

    // 收集赛区统计
    teams.forEach((team: any) => {
      const region = team.region
      if (!region) return

      if (!regionStatsMap.has(region)) {
        regionStatsMap.set(region, {
          region,
          regionName: getRegionDisplayName(region),
          teams: [],
          totalBadges: 0,
          ranking: 0
        })
      }

      const regionStats = regionStatsMap.get(region)!
      const seed = seedGroups.length // 当前组号就是种子号

      // 检查是否已添加该队伍
      if (!regionStats.teams.find(t => t.id === team.teamId)) {
        regionStats.teams.push({
          id: team.teamId,
          name: team.teamName,
          region: region,
          seed,
          badges: isComplete && team.hasBadge ? 1 : 0
        })

        if (isComplete && team.hasBadge) {
          regionStats.totalBadges++
        }
      }
    })
  })

  // 设置种子组
  icpTournament.seedGroups = seedGroups

  // 设置赛区统计
  icpTournament.regionStats = Array.from(regionStatsMap.values())

  // 更新状态
  if (seedGroups.length > 0) {
    const allComplete = seedGroups.every(g => g.isComplete)
    const anyStarted = seedGroups.some(g => g.matches.some(m => m.status === 'completed'))

    if (allComplete) {
      icpTournament.status = 'group_stage' // 种子组完成，可进入赛区对决
    } else if (anyStarted) {
      icpTournament.status = 'group_stage'
    } else {
      icpTournament.status = 'group_stage' // 有数据就是进行中
    }
  }

  // 设置默认选中的组
  if (seedGroups.length > 0) {
    activeSeedGroup.value = seedGroups[0].groupName
  }

  console.log('[ICP] 初始化完成 - seedGroups:', seedGroups.length, 'regionStats:', icpTournament.regionStats.length)
}

/**
 * 获取赛区显示名称
 */
const getRegionDisplayName = (regionCode: string): string => {
  const nameMap: Record<string, string> = {
    'LPL': 'LPL (中国)',
    'LCK': 'LCK (韩国)',
    'LEC': 'LEC (欧洲)',
    'LCS': 'LCS (北美)'
  }
  return nameMap[regionCode] || regionCode
}

/**
 * 从后端更新积分榜
 */
const _updateICPStandingsFromBackend = (standings: GroupStandingInfo[]) => {
  // 如果 seedGroups 为空，说明还没初始化，直接返回
  if (icpTournament.seedGroups.length === 0) {
    console.warn('[ICP] seedGroups 为空，跳过更新积分榜')
    return
  }

  // 遍历每个组的积分榜
  standings.forEach(groupStanding => {
    // 处理 group_name 格式
    let groupName = groupStanding.group_name || 'A'
    groupName = groupName.replace('ICP_GROUP_', '').replace('ICP_', '').replace('GROUP_', '')

    const group = icpTournament.seedGroups.find(g => g.groupName === groupName)
    if (!group) {
      console.warn('[ICP] 找不到组:', groupName)
      return
    }

    // 遍历该组的队伍统计
    if (groupStanding.teams) {
      groupStanding.teams.forEach(teamStats => {
        const frontendStanding = group.standings.find(s => s.teamId === String(teamStats.team_id))

        if (frontendStanding) {
          frontendStanding.matchesPlayed = (teamStats.wins || 0) + (teamStats.losses || 0)
          frontendStanding.wins = teamStats.wins || 0
          frontendStanding.losses = teamStats.losses || 0
          frontendStanding.points = teamStats.points || 0
          frontendStanding.roundsWon = teamStats.games_won || 0
          frontendStanding.roundsLost = teamStats.games_lost || 0
          frontendStanding.roundDifferential = (teamStats.games_won || 0) - (teamStats.games_lost || 0)
        }
      })
    }

    // 重新排序
    group.standings.sort((a, b) => {
      if (b.points !== a.points) return b.points - a.points
      if (b.roundDifferential !== a.roundDifferential) return b.roundDifferential - a.roundDifferential
      if (b.wins !== a.wins) return b.wins - a.wins
      // 使用 teamId 作为最终 tiebreaker 确保稳定排序
      return parseInt(String(a.teamId)) - parseInt(String(b.teamId))
    })

    // 更新排名和徽章
    group.standings.forEach((s, idx) => {
      s.position = idx + 1
      s.hasBadge = idx < 2
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
 * 从后端数据恢复赛区对决状态
 */
const restoreRegionBattleFromBackend = (bracket: BracketInfo) => {
  if (!bracket.matches) return

  console.log('[ICP] restoreRegionBattleFromBackend - 开始恢复赛区对决状态')

  // 根据徽章数量排序赛区（用于确定对阵双方）
  const sortedRegions = [...icpTournament.regionStats].sort((a, b) => b.totalBadges - a.totalBadges)

  // 分配排名
  sortedRegions.forEach((region, index) => {
    region.ranking = index + 1
  })

  // 检查半决赛比赛
  const semiMatches = bracket.matches.filter(m => m.stage?.startsWith('ICP_SEMI_') && !m.stage?.includes('TIEBREAKER'))
  const semiTiebreaker = bracket.matches.find(m => m.stage === 'ICP_SEMI_TIEBREAKER')

  // 检查决赛比赛
  const finalMatches = bracket.matches.filter(m => m.stage?.startsWith('ICP_FINAL_') && !m.stage?.includes('TIEBREAKER'))
  const finalTiebreaker = bracket.matches.find(m => m.stage === 'ICP_FINAL_TIEBREAKER')

  console.log('[ICP] 半决赛比赛数:', semiMatches.length, '决赛比赛数:', finalMatches.length)

  // 检查是否有已分配队伍的半决赛比赛
  const hasAssignedSemiMatches = semiMatches.some(m => m.home_team?.id && m.away_team?.id)
  const hasAssignedFinalMatches = finalMatches.some(m => m.home_team?.id && m.away_team?.id)

  if (!hasAssignedSemiMatches && !hasAssignedFinalMatches) {
    console.log('[ICP] 没有已分配的淘汰赛比赛，跳过恢复')
    return
  }

  // 恢复半决赛状态
  if (hasAssignedSemiMatches && semiMatches.length >= 4) {
    console.log('[ICP] 恢复半决赛状态')

    // 从比赛中推断对阵的两个赛区
    const firstMatch = semiMatches.find(m => m.home_team?.id && m.away_team?.id)
    if (firstMatch && firstMatch.home_team && firstMatch.away_team) {
      const regionA = firstMatch.home_team.region_code || ''
      const regionB = firstMatch.away_team.region_code || ''

      const regionStatsA = icpTournament.regionStats.find(r => r.region === regionA)
      const regionStatsB = icpTournament.regionStats.find(r => r.region === regionB)

      if (regionStatsA && regionStatsB) {
        // 创建半决赛对象
        const semifinalMatches: ICPMatch[] = []
        let regionAWins = 0
        let regionBWins = 0

        // 按种子号排序比赛
        const sortedSemiMatches = [...semiMatches].sort((a, b) => {
          const seedA = parseInt(a.stage?.replace('ICP_SEMI_', '') || '0')
          const seedB = parseInt(b.stage?.replace('ICP_SEMI_', '') || '0')
          return seedA - seedB
        })

        sortedSemiMatches.forEach((m, idx) => {
          if (!m.home_team || !m.away_team) return

          const matchStatus = (m.status || '').toUpperCase()
          const isCompleted = matchStatus === 'COMPLETED'

          semifinalMatches.push({
            id: `semifinal-seed${idx + 1}`,
            backendMatchId: m.match_id,
            teamAId: String(m.home_team.id),
            teamAName: m.home_team.short_name || m.home_team.name || '',
            teamARegion: m.home_team.region_code || '',
            teamBId: String(m.away_team.id),
            teamBName: m.away_team.short_name || m.away_team.name || '',
            teamBRegion: m.away_team.region_code || '',
            scoreA: m.home_score || 0,
            scoreB: m.away_score || 0,
            winnerId: m.winner_id ? String(m.winner_id) : null,
            status: isCompleted ? 'completed' : 'scheduled',
            bestOf: 5,
            stage: 'semifinal'
          })

          // 统计赛区胜场
          if (isCompleted && m.winner_id) {
            const winnerRegion = m.winner_id === m.home_team.id
              ? m.home_team.region_code
              : m.away_team.region_code
            if (winnerRegion === regionA) regionAWins++
            else if (winnerRegion === regionB) regionBWins++
          }
        })

        // 判断半决赛状态
        let semifinalStatus: 'scheduled' | 'in_progress' | 'completed' | 'tiebreaker' = 'scheduled'
        let semifinalWinner: string | null = null

        if (regionAWins >= 3) {
          semifinalStatus = 'completed'
          semifinalWinner = regionA
        } else if (regionBWins >= 3) {
          semifinalStatus = 'completed'
          semifinalWinner = regionB
        } else if (semifinalMatches.some(m => m.status === 'completed')) {
          // 检查是否4场都完成了（可能需要加赛）
          const allCompleted = semifinalMatches.every(m => m.status === 'completed')
          if (allCompleted) {
            if (regionAWins > regionBWins) {
              semifinalStatus = 'completed'
              semifinalWinner = regionA
            } else if (regionBWins > regionAWins) {
              semifinalStatus = 'completed'
              semifinalWinner = regionB
            } else {
              // 2:2 平局，检查加赛
              semifinalStatus = 'tiebreaker'
            }
          } else {
            semifinalStatus = 'in_progress'
          }
        }

        icpTournament.semifinal = {
          id: `semifinal-${regionA}-vs-${regionB}`,
          regionA,
          regionB,
          regionAName: regionStatsA.regionName,
          regionBName: regionStatsB.regionName,
          matches: semifinalMatches,
          regionAWins,
          regionBWins,
          winnerId: semifinalWinner,
          status: semifinalStatus,
          stage: 'semifinal'
        }

        // 恢复加赛状态
        if (semiTiebreaker && semiTiebreaker.home_team && semiTiebreaker.away_team) {
          const tbStatus = (semiTiebreaker.status || '').toUpperCase()
          const tbCompleted = tbStatus === 'COMPLETED'

          icpTournament.semifinal.tiebreakerMatch = {
            id: 'semifinal-tiebreaker',
            backendMatchId: semiTiebreaker.match_id,
            teamAId: String(semiTiebreaker.home_team.id),
            teamAName: semiTiebreaker.home_team.short_name || semiTiebreaker.home_team.name || '',
            teamARegion: semiTiebreaker.home_team.region_code || '',
            teamBId: String(semiTiebreaker.away_team.id),
            teamBName: semiTiebreaker.away_team.short_name || semiTiebreaker.away_team.name || '',
            teamBRegion: semiTiebreaker.away_team.region_code || '',
            scoreA: semiTiebreaker.home_score || 0,
            scoreB: semiTiebreaker.away_score || 0,
            winnerId: semiTiebreaker.winner_id ? String(semiTiebreaker.winner_id) : null,
            status: tbCompleted ? 'completed' : 'scheduled',
            bestOf: 5,
            stage: 'semifinal'
          }

          if (tbCompleted && semiTiebreaker.winner_id) {
            const tbWinnerRegion = semiTiebreaker.winner_id === semiTiebreaker.home_team.id
              ? semiTiebreaker.home_team.region_code
              : semiTiebreaker.away_team.region_code
            icpTournament.semifinal.winnerId = tbWinnerRegion || null
            icpTournament.semifinal.status = 'completed'
          }
        }

        console.log('[ICP] 半决赛恢复完成:', {
          regionA, regionB,
          regionAWins, regionBWins,
          status: icpTournament.semifinal.status,
          winner: icpTournament.semifinal.winnerId
        })
      }
    }
  }

  // 恢复决赛状态
  if (hasAssignedFinalMatches && finalMatches.length >= 4) {
    console.log('[ICP] 恢复决赛状态')

    // 从比赛中推断对阵的两个赛区
    const firstMatch = finalMatches.find(m => m.home_team?.id && m.away_team?.id)
    if (firstMatch && firstMatch.home_team && firstMatch.away_team) {
      const regionA = firstMatch.home_team.region_code || ''
      const regionB = firstMatch.away_team.region_code || ''

      const regionStatsA = icpTournament.regionStats.find(r => r.region === regionA)
      const regionStatsB = icpTournament.regionStats.find(r => r.region === regionB)

      if (regionStatsA && regionStatsB) {
        // 创建决赛对象
        const finalBattleMatches: ICPMatch[] = []
        let regionAWins = 0
        let regionBWins = 0

        // 按种子号排序比赛
        const sortedFinalMatches = [...finalMatches].sort((a, b) => {
          const seedA = parseInt(a.stage?.replace('ICP_FINAL_', '') || '0')
          const seedB = parseInt(b.stage?.replace('ICP_FINAL_', '') || '0')
          return seedA - seedB
        })

        sortedFinalMatches.forEach((m, idx) => {
          if (!m.home_team || !m.away_team) return

          const matchStatus = (m.status || '').toUpperCase()
          const isCompleted = matchStatus === 'COMPLETED'

          finalBattleMatches.push({
            id: `final-seed${idx + 1}`,
            backendMatchId: m.match_id,
            teamAId: String(m.home_team.id),
            teamAName: m.home_team.short_name || m.home_team.name || '',
            teamARegion: m.home_team.region_code || '',
            teamBId: String(m.away_team.id),
            teamBName: m.away_team.short_name || m.away_team.name || '',
            teamBRegion: m.away_team.region_code || '',
            scoreA: m.home_score || 0,
            scoreB: m.away_score || 0,
            winnerId: m.winner_id ? String(m.winner_id) : null,
            status: isCompleted ? 'completed' : 'scheduled',
            bestOf: 5,
            stage: 'final'
          })

          // 统计赛区胜场
          if (isCompleted && m.winner_id) {
            const winnerRegion = m.winner_id === m.home_team.id
              ? m.home_team.region_code
              : m.away_team.region_code
            if (winnerRegion === regionA) regionAWins++
            else if (winnerRegion === regionB) regionBWins++
          }
        })

        // 判断决赛状态
        let finalStatus: 'scheduled' | 'in_progress' | 'completed' | 'tiebreaker' = 'scheduled'
        let finalWinner: string | null = null

        if (regionAWins >= 3) {
          finalStatus = 'completed'
          finalWinner = regionA
        } else if (regionBWins >= 3) {
          finalStatus = 'completed'
          finalWinner = regionB
        } else if (finalBattleMatches.some(m => m.status === 'completed')) {
          const allCompleted = finalBattleMatches.every(m => m.status === 'completed')
          if (allCompleted) {
            if (regionAWins > regionBWins) {
              finalStatus = 'completed'
              finalWinner = regionA
            } else if (regionBWins > regionAWins) {
              finalStatus = 'completed'
              finalWinner = regionB
            } else {
              finalStatus = 'tiebreaker'
            }
          } else {
            finalStatus = 'in_progress'
          }
        }

        icpTournament.final = {
          id: `final-${regionA}-vs-${regionB}`,
          regionA,
          regionB,
          regionAName: regionStatsA.regionName,
          regionBName: regionStatsB.regionName,
          matches: finalBattleMatches,
          regionAWins,
          regionBWins,
          winnerId: finalWinner,
          status: finalStatus,
          stage: 'final'
        }

        // 恢复加赛状态
        if (finalTiebreaker && finalTiebreaker.home_team && finalTiebreaker.away_team) {
          const tbStatus = (finalTiebreaker.status || '').toUpperCase()
          const tbCompleted = tbStatus === 'COMPLETED'

          icpTournament.final.tiebreakerMatch = {
            id: 'final-tiebreaker',
            backendMatchId: finalTiebreaker.match_id,
            teamAId: String(finalTiebreaker.home_team.id),
            teamAName: finalTiebreaker.home_team.short_name || finalTiebreaker.home_team.name || '',
            teamARegion: finalTiebreaker.home_team.region_code || '',
            teamBId: String(finalTiebreaker.away_team.id),
            teamBName: finalTiebreaker.away_team.short_name || finalTiebreaker.away_team.name || '',
            teamBRegion: finalTiebreaker.away_team.region_code || '',
            scoreA: finalTiebreaker.home_score || 0,
            scoreB: finalTiebreaker.away_score || 0,
            winnerId: finalTiebreaker.winner_id ? String(finalTiebreaker.winner_id) : null,
            status: tbCompleted ? 'completed' : 'scheduled',
            bestOf: 5,
            stage: 'final'
          }

          if (tbCompleted && finalTiebreaker.winner_id) {
            const tbWinnerRegion = finalTiebreaker.winner_id === finalTiebreaker.home_team.id
              ? finalTiebreaker.home_team.region_code
              : finalTiebreaker.away_team.region_code
            icpTournament.final.winnerId = tbWinnerRegion || null
            icpTournament.final.status = 'completed'
          }
        }

        console.log('[ICP] 决赛恢复完成:', {
          regionA, regionB,
          regionAWins, regionBWins,
          status: icpTournament.final.status,
          winner: icpTournament.final.winnerId
        })
      }
    }
  }

  // 更新赛事状态
  if (icpTournament.semifinal || icpTournament.final) {
    icpTournament.status = 'region_battle'

    // 如果半决赛已完成但决赛还没有设置，需要创建决赛
    if (icpTournament.semifinal?.status === 'completed' && !icpTournament.final) {
      const semifinalWinner = icpTournament.regionStats.find(r => r.region === icpTournament.semifinal?.winnerId)

      if (semifinalWinner) {
        console.log('[ICP] 半决赛已完成，创建决赛对阵')
        icpTournament.final = createRegionBattle(
          sortedRegions[0], // 第一名赛区
          semifinalWinner,
          'final'
        )
        // 注意：fillKnockoutMatchTeams 是异步的，需要在之后调用
        // 这里标记需要填充决赛队伍
        icpTournament.final._needsFillTeams = true
      }
    }

    // 检查是否已完成
    if (icpTournament.final?.status === 'completed') {
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
      console.log('[ICP] 赛事已完成，冠军:', icpTournament.champion?.regionName)
    }
  }

  console.log('[ICP] 赛区对决状态恢复完成, status:', icpTournament.status)
}

/**
 * 检查ICP赛事完成状态
 */
const _checkICPCompletion = () => {
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

    // 种子队伍分组展示
    .seed-teams-display {
      margin-bottom: 32px;

      .section-title {
        display: flex;
        align-items: center;
        gap: 8px;
        margin: 0 0 20px 0;
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }

      .seed-groups-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 16px;

        .seed-group-card {
          padding: 16px;
          border-radius: 12px;
          border: 2px solid;
          background: white;
          transition: all 0.3s ease;

          &:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          }

          &.seed-1 {
            border-color: #f59e0b;
            background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);

            .seed-header {
              .seed-title { color: #b45309; }
            }
          }

          &.seed-2 {
            border-color: #94a3b8;
            background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);

            .seed-header {
              .seed-title { color: #475569; }
            }
          }

          &.seed-3 {
            border-color: #a78bfa;
            background: linear-gradient(135deg, #f5f3ff 0%, #ede9fe 100%);

            .seed-header {
              .seed-title { color: #6d28d9; }
            }
          }

          &.seed-4 {
            border-color: #6b7280;
            background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);

            .seed-header {
              .seed-title { color: #374151; }
            }
          }

          .seed-header {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 12px;
            padding-bottom: 8px;
            border-bottom: 1px solid rgba(0, 0, 0, 0.1);

            .seed-icon {
              font-size: 20px;
            }

            .seed-title {
              font-size: 16px;
              font-weight: 600;
              flex: 1;
            }
          }

          .seed-team-list {
            display: flex;
            flex-direction: column;
            gap: 8px;

            .seed-team-item {
              display: flex;
              justify-content: space-between;
              align-items: center;
              padding: 8px 12px;
              background: rgba(255, 255, 255, 0.7);
              border-radius: 6px;
              border: 1px solid rgba(0, 0, 0, 0.05);

              .team-name {
                font-weight: 600;
                color: #1f2937;
                font-size: 14px;
              }

              &.pending {
                .team-name {
                  color: #9ca3af;
                  font-style: italic;
                  font-weight: normal;
                }
              }
            }
          }
        }
      }
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

  // 加赛样式
  .tiebreaker-section {
    margin-top: 20px;
    padding: 16px;
    background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
    border: 2px dashed #f59e0b;
    border-radius: 12px;

    .tiebreaker-match {
      background: white;
      border-radius: 8px;
      padding: 16px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);

      .tiebreaker-header {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        margin-bottom: 16px;
        padding-bottom: 12px;
        border-bottom: 1px solid #e5e7eb;

        .tiebreaker-icon {
          font-size: 24px;
        }

        .tiebreaker-title {
          font-size: 18px;
          font-weight: 700;
          color: #b45309;
        }
      }

      .tiebreaker-teams {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 24px;
        margin-bottom: 16px;

        .team-side {
          display: flex;
          align-items: center;
          gap: 8px;

          .team-name {
            font-size: 16px;
            font-weight: 600;
            color: #1f2937;
          }
        }

        .vs-section {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 16px;
          background: #f3f4f6;
          border-radius: 8px;

          .score {
            font-size: 24px;
            font-weight: 700;
            color: #1f2937;
          }

          .vs {
            font-size: 16px;
            font-weight: 600;
            color: #6b7280;
          }
        }
      }

      .tiebreaker-actions {
        display: flex;
        justify-content: center;
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
