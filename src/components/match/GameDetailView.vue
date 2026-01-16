<template>
  <div class="game-detail-view">
    <!-- 局数标题 -->
    <div class="game-header">
      <div class="game-title">
        <el-tag :type="game.isUpset ? 'warning' : 'info'" size="small">
          第{{ game.gameNumber }}局
        </el-tag>
        <el-tag v-if="game.isUpset" type="danger" size="small" effect="dark">
          爆冷
        </el-tag>
      </div>
      <div class="game-winner">
        <span class="winner-label">获胜方:</span>
        <span class="winner-name">{{ game.winnerName }}</span>
      </div>
    </div>

    <!-- 战力对比 -->
    <div class="power-comparison">
      <div class="comparison-header">
        <span class="team-label">{{ game.teamAName }}</span>
        <span class="vs-label">战力对比</span>
        <span class="team-label">{{ game.teamBName }}</span>
      </div>

      <div class="power-bars">
        <div class="power-value team-a">{{ formatPower(game.teamAPower) }}</div>
        <div class="progress-container">
          <div
            class="progress-bar team-a"
            :style="{ width: teamAPowerPercent + '%' }"
            :class="{ winner: game.winnerId === game.teamAId }"
          ></div>
          <div
            class="progress-bar team-b"
            :style="{ width: teamBPowerPercent + '%' }"
            :class="{ winner: game.winnerId === game.teamBId }"
          ></div>
        </div>
        <div class="power-value team-b">{{ formatPower(game.teamBPower) }}</div>
      </div>

      <div class="power-diff" :class="powerDiffClass">
        战力差: {{ formatDiff(game.powerDifference) }}
      </div>
    </div>

    <!-- 发挥值对比 -->
    <div class="performance-comparison">
      <div class="comparison-header">
        <span class="team-label">{{ game.teamAName }}</span>
        <span class="vs-label">发挥值</span>
        <span class="team-label">{{ game.teamBName }}</span>
      </div>

      <div class="perf-bars">
        <div class="perf-value team-a">{{ formatPower(game.teamAPerformance) }}</div>
        <div class="progress-container">
          <div
            class="progress-bar team-a"
            :style="{ width: teamAPerfPercent + '%' }"
            :class="{ winner: game.winnerId === game.teamAId }"
          ></div>
          <div
            class="progress-bar team-b"
            :style="{ width: teamBPerfPercent + '%' }"
            :class="{ winner: game.winnerId === game.teamBId }"
          ></div>
        </div>
        <div class="perf-value team-b">{{ formatPower(game.teamBPerformance) }}</div>
      </div>

      <div class="perf-diff" :class="perfDiffClass">
        发挥差: {{ formatDiff(game.performanceDifference) }}
      </div>
    </div>

    <!-- 选手表现表格 -->
    <div class="players-table">
      <div class="table-header">
        <span class="col-position">位置</span>
        <span class="col-name">选手</span>
        <span class="col-traits">特性</span>
        <span class="col-base">基础</span>
        <span class="col-condition">状态</span>
        <span class="col-noise">波动</span>
        <span class="col-actual">发挥</span>
        <span class="col-impact">影响</span>
      </div>

      <!-- A队选手 -->
      <div class="team-section">
        <div class="team-section-header">
          <span>{{ game.teamAName }}</span>
          <span class="team-power">战力: {{ formatPower(game.teamAPower) }}</span>
        </div>
        <div
          v-for="player in game.teamAPlayers"
          :key="player.playerId"
          class="player-row"
          :class="{ 'high-impact': player.impactScore > 3, 'low-impact': player.impactScore < -3 }"
        >
          <span class="col-position">{{ getPositionName(player.position) }}</span>
          <span class="col-name">{{ player.playerName }}</span>
          <span class="col-traits">
            <template v-if="player.activatedTraits && player.activatedTraits.length > 0">
              <el-tooltip
                v-for="trait in player.activatedTraits"
                :key="trait.type"
                :content="`${trait.name}: ${trait.effect}`"
                placement="top"
              >
                <el-tag
                  :type="trait.isPositive ? 'success' : 'danger'"
                  size="small"
                  class="trait-tag"
                >
                  {{ trait.name }}
                </el-tag>
              </el-tooltip>
            </template>
            <template v-else-if="player.traits && player.traits.length > 0">
              <el-tooltip
                v-for="trait in player.traits"
                :key="trait"
                :content="`${getTraitName(trait)}: 未激活 - ${getTraitDescription(trait)}`"
                placement="top"
              >
                <el-tag type="info" size="small" class="trait-tag inactive">
                  {{ getTraitName(trait) }}
                </el-tag>
              </el-tooltip>
            </template>
            <span v-else class="no-trait">-</span>
          </span>
          <span class="col-base">{{ player.baseAbility }}</span>
          <span class="col-condition" :class="getConditionClass(player.conditionBonus)">
            {{ formatBonus(player.conditionBonus) }}
          </span>
          <span class="col-noise" :class="getNoiseClass(player.stabilityNoise)">
            {{ formatBonus(player.stabilityNoise) }}
          </span>
          <span class="col-actual">{{ player.actualAbility }}</span>
          <span class="col-impact" :class="getImpactClass(player.impactScore)">
            {{ formatImpact(player.impactScore) }}
          </span>
        </div>
      </div>

      <!-- B队选手 -->
      <div class="team-section">
        <div class="team-section-header">
          <span>{{ game.teamBName }}</span>
          <span class="team-power">战力: {{ formatPower(game.teamBPower) }}</span>
        </div>
        <div
          v-for="player in game.teamBPlayers"
          :key="player.playerId"
          class="player-row"
          :class="{ 'high-impact': player.impactScore > 3, 'low-impact': player.impactScore < -3 }"
        >
          <span class="col-position">{{ getPositionName(player.position) }}</span>
          <span class="col-name">{{ player.playerName }}</span>
          <span class="col-traits">
            <template v-if="player.activatedTraits && player.activatedTraits.length > 0">
              <el-tooltip
                v-for="trait in player.activatedTraits"
                :key="trait.type"
                :content="`${trait.name}: ${trait.effect}`"
                placement="top"
              >
                <el-tag
                  :type="trait.isPositive ? 'success' : 'danger'"
                  size="small"
                  class="trait-tag"
                >
                  {{ trait.name }}
                </el-tag>
              </el-tooltip>
            </template>
            <template v-else-if="player.traits && player.traits.length > 0">
              <el-tooltip
                v-for="trait in player.traits"
                :key="trait"
                :content="`${getTraitName(trait)}: 未激活 - ${getTraitDescription(trait)}`"
                placement="top"
              >
                <el-tag type="info" size="small" class="trait-tag inactive">
                  {{ getTraitName(trait) }}
                </el-tag>
              </el-tooltip>
            </template>
            <span v-else class="no-trait">-</span>
          </span>
          <span class="col-base">{{ player.baseAbility }}</span>
          <span class="col-condition" :class="getConditionClass(player.conditionBonus)">
            {{ formatBonus(player.conditionBonus) }}
          </span>
          <span class="col-noise" :class="getNoiseClass(player.stabilityNoise)">
            {{ formatBonus(player.stabilityNoise) }}
          </span>
          <span class="col-actual">{{ player.actualAbility }}</span>
          <span class="col-impact" :class="getImpactClass(player.impactScore)">
            {{ formatImpact(player.impactScore) }}
          </span>
        </div>
      </div>
    </div>

    <!-- 图例说明 -->
    <div class="legend">
      <div class="legend-item">
        <span class="legend-color positive"></span>
        <span>正向影响</span>
      </div>
      <div class="legend-item">
        <span class="legend-color negative"></span>
        <span>负向影响</span>
      </div>
      <div class="legend-item">
        <span class="legend-color high-impact"></span>
        <span>关键发挥 (|影响| > 3)</span>
      </div>
      <div class="legend-item">
        <el-tag type="success" size="small">特性</el-tag>
        <span>激活特性</span>
      </div>
      <div class="legend-item">
        <el-tag type="info" size="small" class="inactive">特性</el-tag>
        <span>未激活特性</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { GameDetail } from '@/types/matchDetail'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES, getTraitDescription, getTraitName } from '@/types/player'

interface Props {
  game: GameDetail
}

const props = defineProps<Props>()

// 计算战力百分比
const totalPower = computed(() => props.game.teamAPower + props.game.teamBPower)
const teamAPowerPercent = computed(() =>
  totalPower.value > 0 ? (props.game.teamAPower / totalPower.value) * 100 : 50
)
const teamBPowerPercent = computed(() =>
  totalPower.value > 0 ? (props.game.teamBPower / totalPower.value) * 100 : 50
)

// 计算发挥值百分比
const totalPerf = computed(() => props.game.teamAPerformance + props.game.teamBPerformance)
const teamAPerfPercent = computed(() =>
  totalPerf.value > 0 ? (props.game.teamAPerformance / totalPerf.value) * 100 : 50
)
const teamBPerfPercent = computed(() =>
  totalPerf.value > 0 ? (props.game.teamBPerformance / totalPerf.value) * 100 : 50
)

// 战力差样式
const powerDiffClass = computed(() => {
  if (props.game.powerDifference > 0) return 'positive'
  if (props.game.powerDifference < 0) return 'negative'
  return ''
})

// 发挥差样式
const perfDiffClass = computed(() => {
  if (props.game.performanceDifference > 0) return 'positive'
  if (props.game.performanceDifference < 0) return 'negative'
  return ''
})

// 获取位置名称
const getPositionName = (position: PlayerPosition): string => {
  return POSITION_NAMES[position] || position
}

// 格式化加成数值
const formatBonus = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  if (value > 0) return `+${value.toFixed(1)}`
  return value.toFixed(1)
}

// 格式化影响力分数 (保留两位小数)
const formatImpact = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  if (value > 0) return `+${value.toFixed(2)}`
  return value.toFixed(2)
}

// 格式化战力/发挥值 (保留两位小数)
const formatPower = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  return value.toFixed(2)
}

// 格式化差值
const formatDiff = (value: number | undefined): string => {
  if (value === undefined || value === null) return '-'
  if (value > 0) return `+${value.toFixed(2)} (A队优势)`
  if (value < 0) return `${value.toFixed(2)} (B队优势)`
  return '0 (势均力敌)'
}

// 状态加成样式
const getConditionClass = (value: number): string => {
  if (value > 3) return 'very-positive'
  if (value > 0) return 'positive'
  if (value < -3) return 'very-negative'
  if (value < 0) return 'negative'
  return ''
}

// 波动样式
const getNoiseClass = (value: number): string => {
  if (value > 2) return 'positive'
  if (value < -2) return 'negative'
  return ''
}

// 影响力样式
const getImpactClass = (value: number): string => {
  if (value > 5) return 'very-positive'
  if (value > 0) return 'positive'
  if (value < -5) return 'very-negative'
  if (value < 0) return 'negative'
  return ''
}

</script>

<style scoped>
.game-detail-view {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
}

.game-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e4e7ed;
}

.game-title {
  display: flex;
  gap: 8px;
  align-items: center;
}

.game-winner {
  font-size: 14px;
}

.winner-label {
  color: #909399;
}

.winner-name {
  color: #67c23a;
  font-weight: bold;
  margin-left: 8px;
}

/* 战力/发挥对比 */
.power-comparison,
.performance-comparison {
  margin-bottom: 20px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.comparison-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
  font-size: 14px;
}

.team-label {
  font-weight: bold;
  color: #303133;
}

.vs-label {
  color: #909399;
}

.power-bars,
.perf-bars {
  display: flex;
  align-items: center;
  gap: 12px;
}

.power-value,
.perf-value {
  font-size: 18px;
  font-weight: bold;
  min-width: 50px;
  text-align: center;
}

.power-value.team-a,
.perf-value.team-a {
  color: #409eff;
}

.power-value.team-b,
.perf-value.team-b {
  color: #e6a23c;
}

.progress-container {
  flex: 1;
  display: flex;
  height: 24px;
  background: #e4e7ed;
  border-radius: 12px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  transition: width 0.3s ease;
}

.progress-bar.team-a {
  background: linear-gradient(to right, #409eff, #66b1ff);
}

.progress-bar.team-b {
  background: linear-gradient(to left, #e6a23c, #f0c78a);
}

.progress-bar.winner {
  box-shadow: 0 0 8px rgba(103, 194, 58, 0.6);
}

.power-diff,
.perf-diff {
  text-align: center;
  margin-top: 8px;
  font-size: 13px;
  color: #606266;
}

.power-diff.positive,
.perf-diff.positive {
  color: #409eff;
}

.power-diff.negative,
.perf-diff.negative {
  color: #e6a23c;
}

/* 选手表格 */
.players-table {
  margin-top: 20px;
}

.table-header {
  display: grid;
  grid-template-columns: 60px 1fr 80px 60px 60px 60px 60px 60px;
  gap: 8px;
  padding: 12px 16px;
  background: #303133;
  color: white;
  border-radius: 8px 8px 0 0;
  font-size: 13px;
  font-weight: bold;
}

.team-section {
  border: 1px solid #e4e7ed;
  border-top: none;
}

.team-section:last-child {
  border-radius: 0 0 8px 8px;
}

.team-section-header {
  display: flex;
  justify-content: space-between;
  padding: 10px 16px;
  background: #f5f7fa;
  font-weight: bold;
  font-size: 14px;
  border-bottom: 1px solid #e4e7ed;
}

.team-power {
  color: #909399;
  font-weight: normal;
}

.player-row {
  display: grid;
  grid-template-columns: 60px 1fr 80px 60px 60px 60px 60px 60px;
  gap: 8px;
  padding: 10px 16px;
  font-size: 13px;
  border-bottom: 1px solid #ebeef5;
  transition: background 0.2s;
}

.player-row:last-child {
  border-bottom: none;
}

.player-row:hover {
  background: #f5f7fa;
}

.player-row.high-impact {
  background: linear-gradient(to right, rgba(103, 194, 58, 0.1), transparent);
}

.player-row.low-impact {
  background: linear-gradient(to right, rgba(245, 108, 108, 0.1), transparent);
}

.col-position {
  color: #909399;
}

.col-name {
  font-weight: 500;
}

.col-base,
.col-actual {
  text-align: center;
}

.col-condition,
.col-noise,
.col-impact {
  text-align: center;
  font-weight: 500;
}

.positive {
  color: #67c23a;
}

.negative {
  color: #f56c6c;
}

.very-positive {
  color: #67c23a;
  font-weight: bold;
}

.very-negative {
  color: #f56c6c;
  font-weight: bold;
}

/* 图例 */
.legend {
  display: flex;
  gap: 20px;
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #e4e7ed;
  font-size: 12px;
  color: #909399;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
}

.legend-color.positive {
  background: #67c23a;
}

.legend-color.negative {
  background: #f56c6c;
}

.legend-color.high-impact {
  background: linear-gradient(to right, rgba(103, 194, 58, 0.3), rgba(245, 108, 108, 0.3));
}

/* 特性列样式 */
.col-traits {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-wrap: wrap;
}

.trait-tag {
  font-size: 11px;
  padding: 2px 6px;
  height: auto;
  line-height: 1.2;
}

.trait-tag.inactive {
  opacity: 0.6;
  border-style: dashed;
}

.trait-icon {
  font-size: 14px;
  cursor: pointer;
  transition: transform 0.2s;
}

.trait-icon:hover {
  transform: scale(1.2);
}

.no-trait {
  color: #c0c4cc;
  font-size: 12px;
}
</style>
