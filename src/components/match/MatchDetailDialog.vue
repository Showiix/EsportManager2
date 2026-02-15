<template>
  <el-dialog
    v-model="dialogVisible"
    :show-header="false"
    width="1080px"
    :close-on-click-modal="false"
    class="match-detail-dialog"
    @close="handleClose"
  >
    <template v-if="matchDetail">
      <!-- 记分牌头部 -->
      <div class="scoreboard">
        <div class="scoreboard-main">
          <div class="scoreboard-team team-a" :class="{ winner: matchDetail.winnerId === matchDetail.teamAId }">
            <span class="team-name">{{ matchDetail.teamAName }}</span>
          </div>
          <div class="scoreboard-center">
            <div class="score-line">
              <span class="team-score" :class="{ winner: matchDetail.winnerId === matchDetail.teamAId }">{{ matchDetail.finalScoreA }}</span>
              <span class="vs-text">VS</span>
              <span class="team-score" :class="{ winner: matchDetail.winnerId === matchDetail.teamBId }">{{ matchDetail.finalScoreB }}</span>
            </div>
            <span class="bo-label">BO{{ matchDetail.bestOf }}</span>
          </div>
          <div class="scoreboard-team team-b" :class="{ winner: matchDetail.winnerId === matchDetail.teamBId }">
            <span class="team-name">{{ matchDetail.teamBName }}</span>
          </div>
        </div>

        <!-- 局数胜负时间线 -->
        <div v-if="gameTimeline.length > 1" class="game-timeline">
          <span
            v-for="(g, i) in gameTimeline"
            :key="i"
            class="timeline-dot"
            :class="g.side"
            :title="`第${g.gameNumber}局 ${g.winnerName}胜`"
          ></span>
        </div>

        <!-- MVP + 关键人物 + 协同值 -->
        <div class="scoreboard-meta">
          <div v-if="matchDetail.mvpPlayerId" class="mvp-card-dark">
            <span class="mvp-trophy-dark">MVP</span>
            <span class="mvp-name-dark">{{ matchDetail.mvpPlayerName }}</span>
            <span class="mvp-detail-dark">累计影响力: {{ formatNumber(matchDetail.mvpTotalImpact) }}</span>
          </div>
          <div v-if="matchDetail.keyPlayer" class="key-card-dark" :class="matchDetail.keyPlayer.reason === '高发挥' ? 'key-positive' : 'key-negative'">
            <span class="key-trophy-dark">关键人物</span>
            <span class="mvp-name-dark">{{ matchDetail.keyPlayer.playerName }}</span>
            <span class="mvp-detail-dark">
              第{{ matchDetail.keyPlayer.gameNumber }}局{{ matchDetail.keyPlayer.reason }}
              ({{ formatBonus(matchDetail.keyPlayer.impactScore) }})
            </span>
          </div>
        </div>

        <!-- 协同值展示 -->
        <div v-if="synergyA || synergyB" class="synergy-section">
          <div class="synergy-title-bar">
            <span class="synergy-label">团队协同</span>
          </div>
          <div class="synergy-compare">
            <div class="synergy-team">
              <span class="synergy-team-name">{{ matchDetail.teamAName }}</span>
              <div class="synergy-value-group">
                <span class="synergy-value" :class="getSynergyClass(synergyA?.synergy_bonus)">
                  +{{ (synergyA?.synergy_bonus ?? 0).toFixed(2) }}
                </span>
                <span class="synergy-sub">协同加成</span>
              </div>
              <span class="synergy-tenure">平均效力 {{ (synergyA?.avg_tenure ?? 0).toFixed(1) }} 赛季</span>
            </div>
            <div class="synergy-vs">VS</div>
            <div class="synergy-team">
              <span class="synergy-team-name">{{ matchDetail.teamBName }}</span>
              <div class="synergy-value-group">
                <span class="synergy-value" :class="getSynergyClass(synergyB?.synergy_bonus)">
                  +{{ (synergyB?.synergy_bonus ?? 0).toFixed(2) }}
                </span>
                <span class="synergy-sub">协同加成</span>
              </div>
              <span class="synergy-tenure">平均效力 {{ (synergyB?.avg_tenure ?? 0).toFixed(1) }} 赛季</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 局数选择器 -->
      <div class="game-selector">
        <button
          v-for="game in matchDetail.games"
          :key="game.gameNumber"
          class="game-card"
          :class="{ active: activeTab === String(game.gameNumber) }"
          @click="activeTab = String(game.gameNumber)"
        >
          <span class="game-card-num">G{{ game.gameNumber }}</span>
          <span class="game-card-winner">{{ game.winnerId === game.teamAId ? game.teamAName : game.teamBName }}胜</span>
          <span v-if="game.isUpset" class="game-card-upset">爆冷</span>
        </button>
      </div>

      <!-- 阵容信息（可折叠） -->
      <div v-if="sortedLineup" class="breakdown-panel lineup-panel">
        <button class="breakdown-toggle" @click="lineupOpen = !lineupOpen">
          <span class="toggle-arrow" :class="{ open: lineupOpen }">&#9654;</span>
          <span>阵容</span>
        </button>
        <div v-if="lineupOpen" class="breakdown-content">
          <div class="lineup-header-row">
            <span class="lineup-team-label">{{ matchDetail.teamAName }}</span>
            <span class="lineup-pos-header"></span>
            <span class="lineup-team-label">{{ matchDetail.teamBName }}</span>
          </div>
          <div class="lineup-matchup-list">
            <div v-for="(home, idx) in sortedLineup.home" :key="home.player_id" class="lineup-matchup-row">
              <span class="lineup-name home">{{ home.player_name }}</span>
              <span class="lineup-pos-badge">{{ home.position }}</span>
              <span class="lineup-name away">{{ sortedLineup.away[idx]?.player_name || '-' }}</span>
            </div>
          </div>
          <div v-if="sortedLineup.substitutions.length > 0" class="lineup-subs">
            <div class="lineup-subs-title">换人</div>
            <div v-for="s in sortedLineup.substitutions" :key="`sub-${s.player_id}-${s.game_number}`" class="lineup-sub-item">
              <span class="sub-arrow">↔</span>
              <span class="sub-in">{{ s.player_name }}</span>
              <span class="sub-pos">({{ s.position }})</span>
              <span v-if="s.replaced_player_name" class="sub-out">替换 {{ s.replaced_player_name }}</span>
              <span v-if="s.substitution_reason" class="sub-reason">· {{ s.substitution_reason }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 当前局详情 -->
      <GameDetailView v-if="currentGame" :game="currentGame" :match-id="matchDetail?.matchId" :key="`${matchDetail?.matchId}-${currentGame.gameNumber}-${activeTab}`" />

      <!-- 底部统计栏 -->
      <div class="match-stats">
        <div class="stat-item">
          <span class="stat-label">总局数</span>
          <span class="stat-value">{{ matchDetail.games.length }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">爆冷局数</span>
          <span class="stat-value upset">{{ upsetCount }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">平均战力差</span>
          <span class="stat-value">{{ avgPowerDiff }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">比赛时间</span>
          <span class="stat-value">{{ formatDate(matchDetail.playedAt) }}</span>
        </div>
      </div>
    </template>

    <template v-else>
      <el-empty description="暂无比赛详情数据" />
    </template>

    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
      <el-button v-if="matchDetail" type="primary" @click="handleExport">
        导出数据
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { MatchDetail } from '@/types/matchDetail'
import GameDetailView from './GameDetailView.vue'
import { ElMessage } from 'element-plus'
import { traitCenterApi, matchApi, type TeamSynergyInfo, type MatchLineupsResult, type MatchGameLineup } from '@/api/tauri'

interface Props {
  visible: boolean
  matchDetail: MatchDetail | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

const activeTab = ref('1')
const lineupOpen = ref(true)
const bpOpen = ref(true)

// 协同值数据
const synergyA = ref<TeamSynergyInfo | null>(null)
const synergyB = ref<TeamSynergyInfo | null>(null)

// 阵容数据
const lineups = ref<MatchLineupsResult | null>(null)

const currentLineup = computed<MatchGameLineup | undefined>(() => {
  if (!lineups.value) return undefined
  return lineups.value.games.find(g => String(g.game_number) === activeTab.value)
})

const LINEUP_POS_ORDER: Record<string, number> = { Top: 0, Jug: 1, Mid: 2, Adc: 3, Sup: 4, TOP: 0, JUG: 1, MID: 2, ADC: 3, SUP: 4 }

const sortedLineup = computed(() => {
  if (!currentLineup.value) return null
  const sortFn = (a: any, b: any) => (LINEUP_POS_ORDER[a.position] ?? 99) - (LINEUP_POS_ORDER[b.position] ?? 99)

  const mergeSubstitutions = (players: typeof currentLineup.value.home_players, subs: typeof currentLineup.value.substitutions, teamId: number) => {
    const result = [...players]
    for (const sub of subs) {
      if (sub.team_id !== teamId) continue
      result.push(sub)
    }
    return result
  }

  const homeTeamId = currentLineup.value.home_players[0]?.team_id
  const awayTeamId = currentLineup.value.away_players[0]?.team_id

  return {
    home: mergeSubstitutions(currentLineup.value.home_players, currentLineup.value.substitutions, homeTeamId).sort(sortFn),
    away: mergeSubstitutions(currentLineup.value.away_players, currentLineup.value.substitutions, awayTeamId).sort(sortFn),
    substitutions: currentLineup.value.substitutions,
  }
})

// 加载协同值
const loadSynergy = async () => {
  if (!props.matchDetail) return
  try {
    const [a, b] = await Promise.all([
      traitCenterApi.getTeamSynergy(Number(props.matchDetail.teamAId)),
      traitCenterApi.getTeamSynergy(Number(props.matchDetail.teamBId)),
    ])
    synergyA.value = a
    synergyB.value = b
  } catch {
    // 协同值加载失败不影响主流程
    synergyA.value = null
    synergyB.value = null
  }

  try {
    const numericId = typeof props.matchDetail.matchId === 'number'
      ? props.matchDetail.matchId
      : parseInt(String(props.matchDetail.matchId).replace(/\D/g, ''))
    if (!isNaN(numericId) && numericId > 0) {
      lineups.value = await matchApi.getMatchLineups(numericId)
    }
  } catch {
    lineups.value = null
  }
}

// 重置选项卡
watch(() => props.matchDetail, (newVal) => {
  if (newVal) {
    activeTab.value = '1'
    loadSynergy()
  } else {
    synergyA.value = null
    synergyB.value = null
    lineups.value = null
  }
}, { immediate: true })

watch(() => props.visible, (newVal) => {
  if (newVal && props.matchDetail) {
    activeTab.value = '1'
    lineupOpen.value = true
    bpOpen.value = true
    loadSynergy()
  }
})

// 当前选中的局
const currentGame = computed(() => {
  if (!props.matchDetail) return null
  return props.matchDetail.games.find(g => String(g.gameNumber) === activeTab.value) || props.matchDetail.games[0]
})

// 局数胜负时间线
const gameTimeline = computed(() => {
  if (!props.matchDetail) return []
  return props.matchDetail.games.map(g => ({
    gameNumber: g.gameNumber,
    winnerName: g.winnerName,
    side: g.winnerId === props.matchDetail!.teamAId ? 'team-a' : 'team-b'
  }))
})

// 爆冷局数
const upsetCount = computed(() => {
  if (!props.matchDetail) return 0
  return props.matchDetail.games.filter(g => g.isUpset).length
})

// 平均战力差
const avgPowerDiff = computed(() => {
  if (!props.matchDetail || props.matchDetail.games.length === 0) return '0'
  const totalDiff = props.matchDetail.games.reduce((sum, g) => sum + Math.abs(g.powerDifference), 0)
  return (totalDiff / props.matchDetail.games.length).toFixed(1)
})

// 格式化数字
const formatNumber = (value: number | undefined): string => {
  if (value === undefined) return '-'
  const rounded = Number(value.toFixed(2))
  return rounded > 0 ? `+${rounded}` : String(rounded)
}

// 格式化加成
const formatBonus = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  const rounded = Number(value.toFixed(2))
  return rounded > 0 ? `+${rounded}` : String(rounded)
}

// 协同值等级样式
const getSynergyClass = (bonus: number | undefined): string => {
  if (!bonus) return 'synergy-low'
  if (bonus >= 1.5) return 'synergy-high'
  if (bonus >= 0.8) return 'synergy-mid'
  return 'synergy-low'
}

// 格式化日期
const formatDate = (date: Date | string | undefined): string => {
  if (!date) return '-'
  const d = typeof date === 'string' ? new Date(date) : date
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 关闭弹窗
const handleClose = () => {
  dialogVisible.value = false
  emit('close')
}

// 导出数据
const handleExport = () => {
  if (!props.matchDetail) return

  try {
    const dataStr = JSON.stringify(props.matchDetail, null, 2)
    const blob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `match-${props.matchDetail.matchId}-detail.json`
    link.click()
    URL.revokeObjectURL(url)
    ElMessage.success('导出成功')
  } catch (error) {
    ElMessage.error('导出失败')
  }
}
</script>

<style scoped>
/* 弹窗基础 */
.match-detail-dialog :deep(.el-dialog) {
  border-radius: 16px;
  overflow: hidden;
}

.match-detail-dialog :deep(.el-dialog__body) {
  padding: 0;
  max-height: 80vh;
  overflow-y: auto;
}

.match-detail-dialog :deep(.el-dialog__footer) {
  border-top: 1px solid #f0f1f3;
  padding: 16px 24px;
}

/* 记分牌 */
.scoreboard {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 28px 32px 20px;
  color: white;
}

.scoreboard-main {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 24px;
}

.scoreboard-team {
  display: flex;
  align-items: center;
  min-width: 180px;
  justify-content: center;
}

.scoreboard-team.team-a {
  justify-content: flex-end;
}

.scoreboard-team.team-b {
  justify-content: flex-start;
}

.team-name {
  font-size: 17px;
  font-weight: 700;
  letter-spacing: -0.2px;
  opacity: 0.9;
}

.scoreboard-team.winner .team-name {
  opacity: 1;
}

.team-score {
  font-size: 48px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  opacity: 0.85;
}

.team-score.winner {
  color: #fbbf24;
  text-shadow: 0 0 20px rgba(251, 191, 36, 0.5);
  opacity: 1;
}

.scoreboard-center {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.score-line {
  display: flex;
  align-items: center;
  gap: 16px;
}

.vs-text {
  font-size: 13px;
  font-weight: 700;
  opacity: 0.5;
  letter-spacing: 2px;
}

.bo-label {
  font-size: 11px;
  font-weight: 600;
  opacity: 0.4;
  letter-spacing: 1px;
}

/* 局数时间线 */
.game-timeline {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-top: 16px;
}

.timeline-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  transition: transform 0.2s;
  cursor: default;
}

.timeline-dot.team-a {
  background: #60a5fa;
  box-shadow: 0 0 6px rgba(96, 165, 250, 0.5);
}

.timeline-dot.team-b {
  background: #fbbf24;
  box-shadow: 0 0 6px rgba(251, 191, 36, 0.5);
}

/* MVP / 关键人物 */
.scoreboard-meta {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.12);
  flex-wrap: wrap;
}

.mvp-card-dark {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  border-radius: 10px;
  border: 1px solid rgba(251, 191, 36, 0.35);
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.15) 0%, rgba(245, 158, 11, 0.08) 100%);
}

.mvp-trophy-dark {
  font-size: 10px;
  font-weight: 800;
  color: #92400e;
  background: linear-gradient(135deg, #fde68a, #fbbf24);
  padding: 2px 10px;
  border-radius: 20px;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.mvp-name-dark {
  font-weight: 800;
  font-size: 14px;
  color: white;
}

.mvp-detail-dark {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
}

.key-card-dark {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.06);
}

.key-card-dark.key-positive {
  border-color: rgba(16, 185, 129, 0.35);
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(16, 185, 129, 0.06) 100%);
}

.key-card-dark.key-negative {
  border-color: rgba(239, 68, 68, 0.35);
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15) 0%, rgba(239, 68, 68, 0.06) 100%);
}

.key-trophy-dark {
  font-size: 10px;
  font-weight: 800;
  padding: 2px 10px;
  border-radius: 20px;
  letter-spacing: 0.5px;
  white-space: nowrap;
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.key-positive .key-trophy-dark {
  background: rgba(16, 185, 129, 0.4);
}

.key-negative .key-trophy-dark {
  background: rgba(239, 68, 68, 0.4);
}

/* 协同值展示 */
.synergy-section {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.12);
}

.synergy-title-bar {
  text-align: center;
  margin-bottom: 10px;
}

.synergy-label {
  font-size: 11px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 2px;
  text-transform: uppercase;
}

.synergy-compare {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 24px;
}

.synergy-team {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  min-width: 140px;
}

.synergy-team-name {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
}

.synergy-value-group {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.synergy-value {
  font-size: 22px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}

.synergy-value.synergy-high {
  color: #fbbf24;
  text-shadow: 0 0 12px rgba(251, 191, 36, 0.4);
}

.synergy-value.synergy-mid {
  color: #60a5fa;
}

.synergy-value.synergy-low {
  color: rgba(255, 255, 255, 0.6);
}

.synergy-sub {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.4);
  font-weight: 500;
}

.synergy-tenure {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
}

.synergy-vs {
  font-size: 11px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.25);
  letter-spacing: 2px;
}

/* 局数选择器 */
.game-selector {
  display: flex;
  gap: 8px;
  padding: 16px 24px;
  background: #f7f8fa;
  border-bottom: 1px solid #f0f1f3;
}

.game-card {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1.5px solid #e5e7eb;
  border-radius: 10px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 12px;
  color: #86909c;
  outline: none;
}

.game-card:hover {
  border-color: #c0c4cc;
}

.game-card.active {
  border-color: #667eea;
  color: #1d2129;
  background: rgba(102, 126, 234, 0.04);
  box-shadow: 0 0 0 1px rgba(102, 126, 234, 0.15);
}

.game-card-num {
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}

.game-card-winner {
  font-weight: 600;
}

.game-card-upset {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  background: #fef3c7;
  color: #d97706;
  font-weight: 700;
}

/* 底部比赛统计 */
.match-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin: 0 24px 24px;
  padding: 18px;
  background: #f7f8fa;
  border-radius: 14px;
  border: 1px solid rgba(0, 0, 0, 0.04);
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
}

.stat-label {
  font-size: 11px;
  color: #86909c;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.stat-value {
  font-size: 20px;
  font-weight: 800;
  color: #1d2129;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.3px;
}

.stat-value.upset {
  color: #f59e0b;
}

/* 阵容区（可折叠面板） */
.lineup-panel {
  margin: 0 24px 16px;
}

.breakdown-panel {
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  overflow: hidden;
}

.breakdown-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 12px 16px;
  background: #f7f8fa;
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-weight: 700;
  color: #1d2129;
  text-align: left;
  transition: background 0.15s;
}

.breakdown-toggle:hover {
  background: #f0f1f3;
}

.toggle-arrow {
  font-size: 10px;
  color: #86909c;
  transition: transform 0.2s;
  display: inline-block;
}

.toggle-arrow.open {
  transform: rotate(90deg);
}

.breakdown-content {
  padding: 16px 20px;
  border-top: 1px solid #e5e7eb;
}

.lineup-header-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.lineup-header-row .lineup-team-label {
  flex: 1;
  font-size: 12px;
  font-weight: 700;
  color: #4e5969;
}

.lineup-header-row .lineup-team-label:first-child {
  text-align: right;
  padding-right: 12px;
}

.lineup-header-row .lineup-team-label:last-child {
  text-align: left;
  padding-left: 12px;
}

.lineup-pos-header {
  width: 40px;
}

.lineup-matchup-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.lineup-matchup-row {
  display: flex;
  align-items: center;
  padding: 4px 0;
  border-radius: 6px;
  background: white;
  border: 1px solid #e5e7eb;
  font-size: 12px;
}

.lineup-name {
  flex: 1;
  font-weight: 600;
  color: #1d2129;
  padding: 0 12px;
}

.lineup-name.home {
  text-align: right;
}

.lineup-name.away {
  text-align: left;
}

.lineup-pos-badge {
  width: 40px;
  text-align: center;
  font-weight: 800;
  font-size: 10px;
  color: #667eea;
  text-transform: uppercase;
  flex-shrink: 0;
}

.lineup-subs {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #e5e7eb;
}

.lineup-subs-title {
  font-size: 11px;
  font-weight: 700;
  color: #86909c;
  margin-bottom: 8px;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.lineup-sub-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  font-size: 12px;
  color: #4e5969;
}

.sub-arrow {
  color: #667eea;
  font-weight: 700;
}

.sub-in {
  font-weight: 700;
  color: #10b981;
}

.sub-pos {
  color: #86909c;
  font-size: 11px;
}

.sub-out {
  color: #ef4444;
}

.sub-reason {
  color: #86909c;
  font-size: 11px;
}
</style>
