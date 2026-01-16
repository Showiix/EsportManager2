<template>
  <el-dialog
    v-model="dialogVisible"
    :title="dialogTitle"
    width="900px"
    :close-on-click-modal="false"
    class="match-detail-dialog"
    @close="handleClose"
  >
    <template v-if="matchDetail">
      <!-- 比赛概要 -->
      <div class="match-summary">
        <div class="summary-teams">
          <div class="team-block team-a" :class="{ winner: matchDetail.winnerId === matchDetail.teamAId }">
            <span class="team-name">{{ matchDetail.teamAName }}</span>
            <span class="team-score">{{ matchDetail.finalScoreA }}</span>
          </div>
          <div class="vs-block">
            <span class="vs-text">VS</span>
            <el-tag v-if="matchDetail.bestOf" size="small" type="info">
              BO{{ matchDetail.bestOf }}
            </el-tag>
          </div>
          <div class="team-block team-b" :class="{ winner: matchDetail.winnerId === matchDetail.teamBId }">
            <span class="team-score">{{ matchDetail.finalScoreB }}</span>
            <span class="team-name">{{ matchDetail.teamBName }}</span>
          </div>
        </div>

        <!-- MVP 展示 -->
        <div v-if="matchDetail.mvpPlayerId" class="mvp-section">
          <div class="mvp-badge">
            <span class="mvp-icon">MVP</span>
          </div>
          <div class="mvp-info">
            <span class="mvp-name">{{ matchDetail.mvpPlayerName }}</span>
            <span class="mvp-impact">
              累计影响力: <strong>{{ formatNumber(matchDetail.mvpTotalImpact) }}</strong>
            </span>
          </div>
        </div>

        <!-- 关键选手 -->
        <div v-if="matchDetail.keyPlayer" class="key-player-section">
          <el-tag
            :type="matchDetail.keyPlayer.reason === '高发挥' ? 'success' : 'danger'"
            size="small"
          >
            关键人物
          </el-tag>
          <span class="key-player-name">{{ matchDetail.keyPlayer.playerName }}</span>
          <span class="key-player-reason">
            第{{ matchDetail.keyPlayer.gameNumber }}局{{ matchDetail.keyPlayer.reason }}
            ({{ formatBonus(matchDetail.keyPlayer.impactScore) }})
          </span>
        </div>
      </div>

      <!-- 每局选项卡 -->
      <el-tabs v-model="activeTab" class="game-tabs">
        <el-tab-pane
          v-for="game in matchDetail.games"
          :key="game.gameNumber"
          :label="getTabLabel(game)"
          :name="String(game.gameNumber)"
        >
          <GameDetailView :game="game" />
        </el-tab-pane>
      </el-tabs>

      <!-- 比赛统计 -->
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
import type { MatchDetail, GameDetail } from '@/types/matchDetail'
import GameDetailView from './GameDetailView.vue'
import { ElMessage } from 'element-plus'

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

// 重置选项卡
watch(() => props.matchDetail, (newVal) => {
  if (newVal) {
    activeTab.value = '1'
  }
})

// 弹窗标题
const dialogTitle = computed(() => {
  if (!props.matchDetail) return '比赛详情'
  return `${props.matchDetail.teamAName} vs ${props.matchDetail.teamBName} 比赛详情`
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

// 获取选项卡标签
const getTabLabel = (game: GameDetail): string => {
  const winner = game.winnerId === game.teamAId ? game.teamAName : game.teamBName
  let label = `第${game.gameNumber}局 - ${winner}胜`
  if (game.isUpset) {
    label += ' (爆冷)'
  }
  return label
}

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
.match-detail-dialog :deep(.el-dialog__body) {
  padding: 20px;
  max-height: 70vh;
  overflow-y: auto;
}

/* 比赛概要 */
.match-summary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 20px;
  color: white;
}

.summary-teams {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 30px;
}

.team-block {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 24px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  transition: all 0.3s;
}

.team-block.winner {
  background: rgba(255, 255, 255, 0.25);
  box-shadow: 0 0 20px rgba(255, 255, 255, 0.3);
}

.team-block.team-b {
  flex-direction: row-reverse;
}

.team-name {
  font-size: 18px;
  font-weight: bold;
}

.team-score {
  font-size: 32px;
  font-weight: bold;
}

.team-block.winner .team-score {
  color: #ffd700;
  text-shadow: 0 0 10px rgba(255, 215, 0, 0.5);
}

.vs-block {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.vs-text {
  font-size: 14px;
  font-weight: bold;
  opacity: 0.8;
}

/* MVP 区域 */
.mvp-section {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
}

.mvp-badge {
  background: linear-gradient(45deg, #ffd700, #ffb347);
  padding: 6px 12px;
  border-radius: 20px;
}

.mvp-icon {
  font-weight: bold;
  color: #1a1a2e;
  font-size: 14px;
}

.mvp-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mvp-name {
  font-size: 16px;
  font-weight: bold;
}

.mvp-impact {
  font-size: 13px;
  opacity: 0.9;
}

.mvp-impact strong {
  color: #ffd700;
}

/* 关键选手 */
.key-player-section {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-top: 12px;
  font-size: 14px;
}

.key-player-name {
  font-weight: bold;
}

.key-player-reason {
  opacity: 0.9;
}

/* 局数选项卡 */
.game-tabs {
  margin-top: 20px;
}

.game-tabs :deep(.el-tabs__item) {
  font-size: 14px;
}

.game-tabs :deep(.el-tabs__content) {
  padding: 0;
}

/* 比赛统计 */
.match-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-top: 20px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: #909399;
}

.stat-value {
  font-size: 18px;
  font-weight: bold;
  color: #303133;
}

.stat-value.upset {
  color: #e6a23c;
}
</style>
