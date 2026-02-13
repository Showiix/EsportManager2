<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="$emit('update:visible', $event)"
    title=""
    width="520px"
    class="market-value-dialog"
    :show-close="true"
  >
    <div class="mv-content">
      <!-- 顶部大卡片：当前身价 -->
      <div class="mv-hero">
        <div class="mv-hero-bg"></div>
        <div class="mv-hero-content">
          <div class="mv-label">计算身价</div>
          <div class="mv-amount">{{ formatMoney(marketValueFactors.calculatedValue) }}</div>
          <div class="mv-player">{{ player.gameId }} · {{ getPositionName(player.position) }}</div>
        </div>
      </div>

      <!-- 系数分解 - 紧凑列表 -->
      <div class="mv-factors">
        <div class="mv-section-title">身价构成</div>
        <div class="mv-factor-list">
          <div class="mv-factor-row">
            <span class="mv-factor-icon">💎</span>
            <span class="mv-factor-name">基础身价</span>
            <span class="mv-factor-detail">能力{{ player.ability }}</span>
            <span class="mv-factor-val primary">{{ formatMoney(marketValueFactors.baseValue) }}</span>
          </div>
          <div class="mv-factor-row">
            <span class="mv-factor-icon">📅</span>
            <span class="mv-factor-name">年龄</span>
            <span class="mv-factor-detail">{{ player.age }}岁</span>
            <span class="mv-factor-val" :class="marketValueFactors.ageFactor >= 1 ? 'positive' : 'negative'">
              × {{ marketValueFactors.ageFactor.toFixed(2) }}
            </span>
          </div>
          <div class="mv-factor-row">
            <el-icon class="mv-factor-icon"><TrendCharts /></el-icon>
            <span class="mv-factor-name">潜力</span>
            <span class="mv-factor-detail">差值{{ player.potential - player.ability }}</span>
            <span class="mv-factor-val" :class="marketValueFactors.potentialFactor >= 1 ? 'positive' : 'negative'">
              × {{ marketValueFactors.potentialFactor.toFixed(2) }}
            </span>
          </div>
          <div class="mv-factor-row">
            <el-icon class="mv-factor-icon"><StarFilled /></el-icon>
            <span class="mv-factor-name">天赋</span>
            <span class="mv-factor-detail">{{ getTalentLabel(player.tag) }}</span>
            <span class="mv-factor-val" :class="marketValueFactors.tagFactor >= 1 ? 'positive' : 'negative'">
              × {{ marketValueFactors.tagFactor.toFixed(2) }}
            </span>
          </div>
          <div class="mv-factor-row">
            <el-icon class="mv-factor-icon"><Monitor /></el-icon>
            <span class="mv-factor-name">位置</span>
            <span class="mv-factor-detail">{{ getPositionName(player.position) }}</span>
            <span class="mv-factor-val" :class="marketValueFactors.positionFactor >= 1 ? 'positive' : 'negative'">
              × {{ marketValueFactors.positionFactor.toFixed(2) }}
            </span>
          </div>
          <div class="mv-factor-row">
            <span class="mv-factor-icon">🌍</span>
            <span class="mv-factor-name">赛区</span>
            <span class="mv-factor-detail">{{ player.region }}</span>
            <span class="mv-factor-val" :class="marketValueFactors.regionFactor >= 1 ? 'positive' : 'negative'">
              × {{ marketValueFactors.regionFactor.toFixed(2) }}
            </span>
          </div>
          <div class="mv-factor-row highlight">
            <el-icon class="mv-factor-icon"><Trophy /></el-icon>
            <span class="mv-factor-name">荣誉</span>
            <span class="mv-factor-detail">{{ getHonorDescription(marketValueFactors.honorFactor, marketValueFactors.teamHonorCount, marketValueFactors.individualHonorCount) }}</span>
            <span class="mv-factor-val" :class="marketValueFactors.honorFactor > 1 ? 'positive' : ''">
              × {{ marketValueFactors.honorFactor.toFixed(2) }}
            </span>
          </div>
        </div>
      </div>

      <!-- 身价变化记录 -->
      <div class="mv-history">
        <div class="mv-section-title">变化记录</div>
        <div class="mv-history-list" v-if="marketValueChanges.length > 0">
          <div class="mv-history-item" v-for="change in marketValueChanges" :key="change.id">
            <div class="mv-history-left">
              <el-tag size="small" :type="change.change_amount > 0 ? 'success' : 'danger'" effect="dark">
                {{ change.reason }}
              </el-tag>
            </div>
            <div class="mv-history-right">
              <span class="mv-history-change" :class="change.change_amount > 0 ? 'up' : 'down'">
                {{ change.change_amount > 0 ? '↑' : '↓' }} {{ Math.abs(change.change_percent).toFixed(0) }}%
              </span>
            </div>
          </div>
        </div>
        <div class="mv-no-history" v-else>
          <span class="mv-no-icon">📭</span>
          <span>暂无变化记录</span>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, PropType } from 'vue'
import { TrendCharts, StarFilled, Monitor, Trophy } from '@element-plus/icons-vue'
import { formatMoney } from '@/utils'
import { MarketValueChange } from '@/api/tauri'
import { PlayerDetailInfo, PlayerHonor } from '@/composables/usePlayerDetail'

const props = defineProps({
  visible: {
    type: Boolean,
    required: true
  },
  player: {
    type: Object as PropType<PlayerDetailInfo>,
    required: true
  },
  marketValueChanges: {
    type: Array as PropType<MarketValueChange[]>,
    default: () => []
  },
  honors: {
    type: Array as PropType<PlayerHonor[]>,
    default: () => []
  }
})

defineEmits(['update:visible'])

// Helper functions (extracted from original component)
const getPositionName = (position: string) => {
  const names: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: '下路',
    SUP: '辅助',
  }
  return names[position] || position
}

const getTalentLabel = (tag: string) => {
  const labels: Record<string, string> = {
    GENIUS: '天才',
    NORMAL: '普通',
    ORDINARY: '平庸',
  }
  return labels[tag] || tag
}

const getAbilityMultiplier = (ability: number): number => {
  if (ability >= 95) return 50
  if (ability >= 90) return 35
  if (ability >= 85) return 20
  if (ability >= 80) return 12
  if (ability >= 75) return 7
  if (ability >= 70) return 4
  if (ability >= 60) return 2
  return 1
}

const getHonorDescription = (factor: number, teamCount?: number, individualCount?: number): string => {
  const total = (teamCount || 0) + (individualCount || 0)
  if (total === 0) return '无荣誉加成'

  const parts: string[] = []
  if (teamCount && teamCount > 0) parts.push(`${teamCount}项团队`)
  if (individualCount && individualCount > 0) parts.push(`${individualCount}项个人`)

  if (factor >= 3.0) return `传奇(${parts.join('+')})`
  if (factor >= 2.0) return `顶级(${parts.join('+')})`
  if (factor >= 1.5) return `优秀(${parts.join('+')})`
  if (factor > 1.0) return parts.join('+')
  return '无荣誉加成'
}

// Market value calculation logic
const marketValueFactors = computed(() => {
  const ability = props.player.ability
  const age = props.player.age
  const potential = props.player.potential
  const tag = props.player.tag
  const position = props.player.position
  const region = props.player.region

  // 基础身价计算
  const multiplier = getAbilityMultiplier(ability)
  const baseValue = ability * multiplier * 10000

  // 年龄系数
  let ageFactor = 1.0
  if (age <= 19) ageFactor = 1.5
  else if (age <= 22) ageFactor = 1.3
  else if (age <= 25) ageFactor = 1.0
  else if (age <= 27) ageFactor = 0.85
  else if (age <= 29) ageFactor = 0.7
  else ageFactor = 0.5

  // 潜力系数
  const potentialDiff = potential - ability
  let potentialFactor = 1.0
  if (potentialDiff > 10) potentialFactor = 1.25
  else if (potentialDiff >= 5) potentialFactor = 1.1

  // 标签系数
  const tagFactors: Record<string, number> = { GENIUS: 1.2, NORMAL: 1.0, ORDINARY: 0.9 }
  const tagFactor = tagFactors[tag] || 1.0

  // 位置系数
  const positionFactors: Record<string, number> = { MID: 1.2, ADC: 1.15, JUG: 1.1, TOP: 1.0, SUP: 0.9 }
  const positionFactor = positionFactors[position] || 1.0

  // 赛区系数
  const regionFactors: Record<string, number> = { LPL: 1.3, LCK: 1.2, LEC: 1.0, LCS: 0.9 }
  const regionFactor = regionFactors[region] || 0.8

  // 荣誉系数（全面计算各类荣誉）
  let honorPoints = 0
  let teamHonorCount = 0
  let individualHonorCount = 0

  props.honors.forEach(h => {
    const pos = h.position

    // 团队荣誉
    if (pos === '冠军' || pos === '冠军成员') {
      honorPoints += 0.30
      teamHonorCount++
    } else if (pos === '亚军' || pos === '亚军成员') {
      honorPoints += 0.15
      teamHonorCount++
    } else if (pos === '季军' || pos === '季军成员') {
      honorPoints += 0.10
      teamHonorCount++
    } else if (pos === '殿军' || pos === '殿军成员') {
      honorPoints += 0.05
      teamHonorCount++
    }
    // 个人MVP荣誉
    else if (pos === '赛事MVP' || pos === '决赛MVP' || pos === '季后赛FMVP' || pos === '年度MVP') {
      honorPoints += 0.25
      individualHonorCount++
    } else if (pos === '常规赛MVP') {
      honorPoints += 0.20
      individualHonorCount++
    }
    // 年度Top20（从tournament_name提取排名）
    else if (pos.includes('年度Top') || h.tournament.includes('年度Top')) {
      const match = (pos + h.tournament).match(/年度Top(\d+)/)
      if (match) {
        const rank = parseInt(match[1])
        if (rank <= 5) honorPoints += 0.20
        else if (rank <= 10) honorPoints += 0.15
        else honorPoints += 0.10
        individualHonorCount++
      }
    }
    // 年度最佳位置
    else if (pos.includes('年度最佳') && !pos.includes('新秀')) {
      honorPoints += 0.15
      individualHonorCount++
    }
    // 年度最佳新秀
    else if (pos.includes('年度最佳新秀') || pos === '年度新秀') {
      honorPoints += 0.10
      individualHonorCount++
    }
    // 常规赛第一
    else if (pos === '常规赛第一') {
      honorPoints += 0.08
      teamHonorCount++
    }
  })

  const honorFactor = 1.0 + honorPoints

  // 计算最终身价 = 基础身价 × 所有因子
  const calculatedValue = baseValue * ageFactor * potentialFactor * tagFactor * positionFactor * regionFactor * Math.min(honorFactor, 4.0)

  return {
    baseValue,
    ageFactor,
    potentialFactor,
    tagFactor,
    positionFactor,
    regionFactor,
    honorFactor: Math.min(honorFactor, 4.0), // 提高上限到4.0
    teamHonorCount,
    individualHonorCount,
    calculatedValue: Math.round(calculatedValue), // 最终计算的身价
  }
})
</script>

<style scoped>
/* Market Value Dialog Styles */
.market-value-dialog :deep(.el-dialog__header) {
  display: none;
}

.market-value-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.mv-content {
  padding: 0;
}

/* Hero Card */
.mv-hero {
  position: relative;
  padding: 32px 24px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  border-radius: 12px 12px 0 0;
  text-align: center;
  overflow: hidden;
}

.mv-hero-bg {
  position: absolute;
  top: -50%;
  right: -20%;
  width: 200px;
  height: 200px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
}

.mv-hero-content {
  position: relative;
  z-index: 1;
}

.mv-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  margin-bottom: 8px;
}

.mv-amount {
  font-size: 42px;
  font-weight: 800;
  color: white;
  line-height: 1;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  margin-bottom: 8px;
}

.mv-player {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

/* Factors Section */
.mv-factors {
  padding: 20px;
}

.mv-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.mv-section-title::before {
  content: '';
  width: 3px;
  height: 14px;
  background: var(--primary-color);
  border-radius: 2px;
}

.mv-factor-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mv-factor-row {
  display: flex;
  align-items: center;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  transition: all 0.2s ease;
}

.mv-factor-row:hover {
  background: var(--bg-tertiary);
}

.mv-factor-row.highlight {
  background: linear-gradient(135deg, #fef3c7 0%, var(--bg-secondary) 100%);
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.mv-factor-icon {
  font-size: 18px;
  width: 28px;
  flex-shrink: 0;
}

.mv-factor-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  min-width: 60px;
}

.mv-factor-detail {
  flex: 1;
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: right;
  margin-right: 12px;
}

.mv-factor-val {
  font-size: 14px;
  font-weight: 700;
  min-width: 55px;
  text-align: right;
}

.mv-factor-val.primary {
  color: var(--primary-color);
}

.mv-factor-val.positive {
  color: #10b981;
}

.mv-factor-val.negative {
  color: #ef4444;
}

/* History Section */
.mv-history {
  padding: 0 20px 20px;
}

.mv-history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 180px;
  overflow-y: auto;
}

.mv-history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-light);
}

.mv-history-left {
  flex: 1;
}

.mv-history-right {
  flex-shrink: 0;
}

.mv-history-change {
  font-size: 15px;
  font-weight: 700;
}

.mv-history-change.up {
  color: #10b981;
}

.mv-history-change.down {
  color: #ef4444;
}

.mv-no-history {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 24px;
  background: var(--bg-secondary);
  border-radius: 8px;
  color: var(--text-placeholder);
  font-size: 13px;
}

.mv-no-icon {
  font-size: 20px;
}
</style>
