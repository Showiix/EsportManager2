<template>
  <el-dialog
    v-model="dialogVisible"
    :title="`${playerDetail?.player_name || '选手'} 的合同详情`"
    width="800px"
    destroy-on-close
  >
    <div v-loading="isLoading" class="contract-dialog-content">
      <template v-if="playerDetail">
        <!-- 顶部信息 -->
        <div class="top-section">
          <!-- 选手基本信息 -->
          <div class="player-header">
            <div class="player-avatar-large" :class="playerDetail.position?.toLowerCase()">
              {{ playerDetail.position || '?' }}
            </div>
            <div class="player-main-info">
              <h2>{{ playerDetail.player_name }}</h2>
              <div class="player-tags">
                <el-tag v-if="playerDetail.team_name" :type="getRegionTagType(playerDetail.region_code)">
                  {{ playerDetail.team_name }}
                </el-tag>
                <el-tag v-else type="info">自由身</el-tag>
                <span class="age-text">{{ playerDetail.age }}岁</span>
              </div>
              <div class="ability-row">
                <span class="label">能力</span>
                <span :style="{ color: getAbilityColor(playerDetail.ability) }" class="ability-value">{{ playerDetail.ability }}</span>
                <span class="label">潜力</span>
                <span class="ability-value">{{ playerDetail.potential }}</span>
                <span class="label">稳定性</span>
                <span class="ability-value">{{ playerDetail.stability }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 信息卡片网格 -->
        <div class="info-grid">
          <!-- 合同信息 -->
          <div class="info-card">
            <div class="card-header">
              <el-icon><Document /></el-icon>
              <span>合同信息</span>
            </div>
            <div class="card-body">
              <div class="info-row">
                <span class="label">当前年薪</span>
                <span class="value">{{ formatSalary(playerDetail.salary) }}/年</span>
              </div>
              <div class="info-row">
                <span class="label">合同到期</span>
                <span :class="getContractClass(playerDetail.contract_end_season)" class="value">
                  {{ playerDetail.contract_end_season ? `${playerDetail.contract_end_season}赛季` : '已到期' }}
                </span>
              </div>
              <div class="info-row">
                <span class="label">加入赛季</span>
                <span class="value">{{ playerDetail.join_season }}赛季</span>
              </div>
              <div class="info-row">
                <span class="label">效力年数</span>
                <span class="value">{{ playerDetail.years_in_team }}年</span>
              </div>
            </div>
          </div>

          <!-- 身价信息 -->
          <div class="info-card">
            <div class="card-header">
              <el-icon><Money /></el-icon>
              <span>身价信息</span>
            </div>
            <div class="card-body">
              <div class="info-row">
                <span class="label">基础身价</span>
                <span class="value">{{ formatValue(playerDetail.base_market_value) }}</span>
              </div>
              <div class="info-row">
                <span class="label">荣誉系数</span>
                <span class="value highlight">×{{ playerDetail.honor_factor.toFixed(2) }}</span>
              </div>
              <div class="info-row">
                <span class="label">赛区系数</span>
                <span class="value">×{{ playerDetail.region_factor.toFixed(2) }} ({{ playerDetail.region_code || '未知' }})</span>
              </div>
              <div class="info-row total">
                <span class="label">完整身价</span>
                <span class="value primary">{{ formatValue(playerDetail.calculated_market_value) }}</span>
              </div>
            </div>
          </div>

          <!-- 满意度信息 -->
          <div class="info-card">
            <div class="card-header">
              <el-icon><Sunny /></el-icon>
              <span>满意度</span>
            </div>
            <div class="card-body">
              <div class="progress-section">
                <el-progress
                  :percentage="playerDetail.satisfaction"
                  :stroke-width="12"
                  :color="getSatisfactionColor(playerDetail.satisfaction)"
                />
                <span :class="getSatisfactionClass(playerDetail.satisfaction)" class="progress-value">
                  {{ playerDetail.satisfaction }}
                </span>
              </div>
              <div class="status-text">
                状态: {{ getSatisfactionStatus(playerDetail.satisfaction) }}
              </div>
            </div>
          </div>

          <!-- 忠诚度信息 -->
          <div class="info-card">
            <div class="card-header">
              <el-icon><Medal /></el-icon>
              <span>忠诚度</span>
            </div>
            <div class="card-body">
              <div class="progress-section">
                <el-progress
                  :percentage="playerDetail.loyalty"
                  :stroke-width="12"
                  :color="getLoyaltyColor(playerDetail.loyalty)"
                />
                <span :class="getLoyaltyClass(playerDetail.loyalty)" class="progress-value">
                  {{ playerDetail.loyalty }}
                </span>
              </div>
              <div class="info-row">
                <span class="label">类型</span>
                <span class="value">{{ playerDetail.loyalty_type }}</span>
              </div>
              <div class="info-row">
                <span class="label">离队阈值</span>
                <span class="value">满意度 < {{ playerDetail.departure_threshold }}</span>
              </div>
              <div class="info-row">
                <span class="label">转会溢价</span>
                <span class="value">×{{ playerDetail.loyalty_price_factor.toFixed(2) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 转会意愿 -->
        <div v-if="playerDetail.wants_to_leave" class="departure-warning">
          <el-icon><Warning /></el-icon>
          <div class="warning-content">
            <strong>该选手有离队意愿</strong>
            <div class="reasons">
              原因: {{ playerDetail.departure_reasons.join('、') || '未知' }}
            </div>
          </div>
        </div>

        <!-- 身价变化趋势 -->
        <div v-if="playerDetail.market_value_history.length > 0" class="history-section">
          <div class="section-header">
            <el-icon><TrendCharts /></el-icon>
            <span>身价变化记录</span>
          </div>
          <el-table :data="playerDetail.market_value_history" stripe size="small" max-height="200">
            <el-table-column prop="season_id" label="赛季" width="80" />
            <el-table-column label="变化" width="180">
              <template #default="{ row }">
                <span>{{ formatValue(row.old_value) }} → {{ formatValue(row.new_value) }}</span>
              </template>
            </el-table-column>
            <el-table-column label="幅度" width="100">
              <template #default="{ row }">
                <span :class="row.change_amount >= 0 ? 'positive' : 'negative'">
                  {{ row.change_amount >= 0 ? '+' : '' }}{{ row.change_percent.toFixed(1) }}%
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="reason" label="原因" />
          </el-table>
        </div>

        <!-- 忠诚度变化记录 -->
        <div v-if="playerDetail.loyalty_changes.length > 0" class="history-section">
          <div class="section-header">
            <el-icon><Switch /></el-icon>
            <span>忠诚度变化记录</span>
          </div>
          <el-timeline>
            <el-timeline-item
              v-for="change in playerDetail.loyalty_changes.slice(0, 5)"
              :key="change.season_id"
              :type="change.change_amount >= 0 ? 'success' : 'danger'"
              :hollow="true"
            >
              <div class="timeline-content">
                <span class="season">S{{ change.season_id }}</span>
                <span :class="change.change_amount >= 0 ? 'positive' : 'negative'">
                  {{ change.change_amount >= 0 ? '+' : '' }}{{ change.change_amount }}
                </span>
                <span class="reason">{{ change.reason }}</span>
              </div>
            </el-timeline-item>
          </el-timeline>
        </div>
      </template>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">关闭</el-button>
      <el-button type="primary" @click="viewFullDetail">查看完整详情</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Document, Money, Sunny, Medal, Warning, TrendCharts, Switch } from '@element-plus/icons-vue'
import { transferApi, type PlayerContractDetail } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { formatMoneyFromWan as formatValue, formatMoneyFromWan as formatSalary } from '@/utils'
import { createLogger } from '@/utils/logger'

const logger = createLogger('PlayerContractDialog')

const router = useRouter()
const gameStore = useGameStore()

const props = defineProps<{
  visible: boolean
  playerId: number | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const dialogVisible = ref(false)
const playerDetail = ref<PlayerContractDetail | null>(null)
const isLoading = ref(false)

// 双向绑定 visible
watch(() => props.visible, (val) => {
  dialogVisible.value = val
})

watch(dialogVisible, (val) => {
  emit('update:visible', val)
})

// 监听 playerId 变化，加载数据
watch(() => props.playerId, async (newId) => {
  if (newId) {
    await loadPlayerDetail(newId)
  } else {
    playerDetail.value = null
  }
}, { immediate: true })

// 加载选手详情
const loadPlayerDetail = async (playerId: number) => {
  isLoading.value = true
  try {
    playerDetail.value = await transferApi.getPlayerContractDetail(playerId)
  } catch (e) {
    logger.error('加载选手详情失败', { playerId, error: e })
    ElMessage.error('加载选手详情失败')
  } finally {
    isLoading.value = false
  }
}

// 查看完整详情
const viewFullDetail = () => {
  if (props.playerId) {
    router.push(`/players/${props.playerId}`)
  }
}

// 辅助函数
const getRegionTagType = (region: string | null) => {
  const types: Record<string, string> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning',
  }
  return types[region || ''] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

// formatValue 和 formatSalary 从 @/utils 导入

const getContractClass = (endSeason: number | null) => {
  if (!endSeason) return 'contract-expired'
  const season = gameStore.currentSeason
  if (endSeason <= season) return 'contract-expired'
  if (endSeason === season + 1) return 'contract-expiring'
  return ''
}

const getSatisfactionColor = (value: number) => {
  if (value >= 70) return '#67c23a'
  if (value >= 40) return '#e6a23c'
  return '#f56c6c'
}

const getSatisfactionClass = (value: number) => {
  if (value >= 70) return 'stat-high'
  if (value >= 40) return 'stat-mid'
  return 'stat-low'
}

const getSatisfactionStatus = (value: number) => {
  if (value >= 80) return '非常满意'
  if (value >= 60) return '满意'
  if (value >= 40) return '一般'
  if (value >= 20) return '不满'
  return '非常不满'
}

const getLoyaltyColor = (value: number) => {
  if (value >= 70) return '#409eff'
  if (value >= 40) return '#909399'
  return '#f56c6c'
}

const getLoyaltyClass = (value: number) => {
  if (value >= 70) return 'loyalty-high'
  if (value >= 40) return 'loyalty-mid'
  return 'loyalty-low'
}
</script>

<style scoped>
.contract-dialog-content {
  min-height: 300px;
}

.top-section {
  margin-bottom: 20px;
}

.player-header {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 16px;
  background: linear-gradient(135deg, #f5f7fa, #e4e7ed);
  border-radius: 12px;
}

.player-avatar-large {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 18px;
  flex-shrink: 0;
}

.player-avatar-large.top { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-avatar-large.jug { background: linear-gradient(135deg, #22c55e, #16a34a); }
.player-avatar-large.mid { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-avatar-large.adc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.player-avatar-large.sup { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.player-main-info h2 {
  margin: 0 0 8px 0;
  font-size: 22px;
  color: #303133;
}

.player-tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.age-text {
  color: #909399;
  font-size: 14px;
}

.ability-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.ability-row .label {
  font-size: 13px;
  color: #909399;
}

.ability-row .ability-value {
  font-size: 16px;
  font-weight: 700;
}

/* 信息卡片网格 */
.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.info-card {
  background: #fafafa;
  border: 1px solid #ebeef5;
  border-radius: 10px;
  overflow: hidden;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #f5f7fa;
  font-weight: 600;
  color: #303133;
  border-bottom: 1px solid #ebeef5;
}

.card-body {
  padding: 12px 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.info-row .label {
  font-size: 13px;
  color: #909399;
}

.info-row .value {
  font-size: 14px;
  color: #303133;
  font-weight: 500;
}

.info-row .value.highlight {
  color: #e6a23c;
  font-weight: 700;
}

.info-row .value.primary {
  color: #409eff;
  font-weight: 700;
  font-size: 16px;
}

.info-row.total {
  border-top: 1px dashed #ebeef5;
  margin-top: 8px;
  padding-top: 12px;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.progress-section .el-progress {
  flex: 1;
}

.progress-value {
  font-size: 18px;
  font-weight: 700;
  min-width: 36px;
}

.status-text {
  font-size: 13px;
  color: #606266;
}

.contract-expired {
  color: #f56c6c !important;
  font-weight: 600;
}

.contract-expiring {
  color: #e6a23c !important;
}

.stat-high { color: #67c23a; }
.stat-mid { color: #e6a23c; }
.stat-low { color: #f56c6c; }

.loyalty-high { color: #409eff; }
.loyalty-mid { color: #909399; }
.loyalty-low { color: #f56c6c; }

/* 离队警告 */
.departure-warning {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: linear-gradient(135deg, #fef0f0, #fde2e2);
  border: 1px solid #fab6b6;
  border-radius: 10px;
  margin-bottom: 20px;
}

.departure-warning .el-icon {
  font-size: 24px;
  color: #f56c6c;
}

.warning-content strong {
  color: #f56c6c;
  display: block;
  margin-bottom: 4px;
}

.warning-content .reasons {
  font-size: 13px;
  color: #909399;
}

/* 历史记录 */
.history-section {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #ebeef5;
}

.timeline-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.timeline-content .season {
  font-weight: 600;
  color: #606266;
}

.positive {
  color: #67c23a;
  font-weight: 600;
}

.negative {
  color: #f56c6c;
  font-weight: 600;
}

.timeline-content .reason {
  color: #909399;
  font-size: 13px;
}
</style>
