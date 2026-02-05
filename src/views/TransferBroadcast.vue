<template>
  <div class="transfer-broadcast">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>转会播报</h1>
        <p>{{ currentSeason }} 赛季 - 转会窗口进行中</p>
      </div>
      <div class="header-actions">
        <el-tag :type="statusTagType" size="large" effect="dark">
          {{ statusText }}
        </el-tag>
      </div>
    </div>

    <!-- 进度条 -->
    <el-card class="progress-card">
      <div class="progress-header">
        <span class="progress-title">转会进度</span>
        <span class="progress-text">第 {{ currentRound }} / 7 轮</span>
      </div>
      <el-progress
        :percentage="progressPercentage"
        :stroke-width="12"
        :format="() => ''"
        status="success"
      />
      <div class="round-labels">
        <span
          v-for="(name, index) in roundNames"
          :key="index"
          class="round-label"
          :class="{ active: index <= currentRound, current: index === currentRound }"
        >
          {{ name }}
        </span>
      </div>
    </el-card>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="24"><Switch /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ transferWindow?.total_transfers ?? 0 }}</div>
              <div class="stat-label">完成交易</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="24"><User /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ transferWindow?.free_agents_signed ?? 0 }}</div>
              <div class="stat-label">自由球员签约</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="24"><Money /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatFee(transferWindow?.total_fees ?? 0) }}</div>
              <div class="stat-label">总转会费</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon red">
              <el-icon :size="24"><Warning /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ transferWindow?.retirements ?? 0 }}</div>
              <div class="stat-label">退役选手</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <el-button
        v-if="!isCompleted"
        type="primary"
        size="large"
        :loading="isProcessing"
        @click="handleNextRound"
      >
        <el-icon><CaretRight /></el-icon>
        执行下一轮
      </el-button>
      <el-button
        v-if="!isCompleted"
        type="warning"
        size="large"
        :loading="isProcessing"
        @click="handleFastForward"
      >
        <el-icon><DArrowRight /></el-icon>
        快进完成
      </el-button>
      <el-button
        v-if="isCompleted"
        type="success"
        size="large"
        @click="handleComplete"
      >
        <el-icon><CircleCheck /></el-icon>
        确认完成，进入新赛季
      </el-button>
      <el-button @click="router.push('/transfer')">
        <el-icon><Back /></el-icon>
        返回预览
      </el-button>
    </div>

    <!-- 当前轮次摘要 -->
    <el-card v-if="currentRoundSummary" class="round-summary-card">
      <div class="round-summary-header">
        <div class="round-badge">第 {{ currentRoundSummary.round }} 轮</div>
        <div class="round-name">{{ currentRoundSummary.round_name }}</div>
      </div>
      <div class="round-summary-content">
        {{ currentRoundSummary.summary }}
      </div>
      <div class="round-stats">
        <div class="round-stat">
          <span class="stat-value">{{ currentRoundSummary.events_count }}</span>
          <span class="stat-label">事件数</span>
        </div>
        <div class="round-stat">
          <span class="stat-value">{{ currentRoundSummary.transfers_count }}</span>
          <span class="stat-label">交易数</span>
        </div>
        <div class="round-stat">
          <span class="stat-value">{{ formatFee(currentRoundSummary.total_fees) }}</span>
          <span class="stat-label">转会费</span>
        </div>
      </div>
    </el-card>

    <!-- 新闻列表 -->
    <div class="news-section">
      <div class="news-header">
        <h2>
          <el-icon><Bell /></el-icon>
          转会新闻
        </h2>
        <el-radio-group v-model="filterImportance" size="small">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="BREAKING">头条</el-radio-button>
          <el-radio-button value="MAJOR">重大</el-radio-button>
        </el-radio-group>
      </div>

      <el-empty v-if="filteredEvents.length === 0" description="暂无转会新闻，点击「执行下一轮」开始" />

      <transition-group name="news-list" tag="div" class="news-list">
        <div
          v-for="event in filteredEvents"
          :key="event.id || `${event.round}-${event.player_id}`"
          class="news-card"
          :class="[`importance-${event.importance.toLowerCase()}`, `type-${event.event_type.toLowerCase()}`]"
        >
          <!-- 重要性标签 -->
          <div class="news-importance" :class="event.importance.toLowerCase()">
            {{ getImportanceText(event.importance) }}
          </div>

          <!-- 事件类型图标 -->
          <div class="news-type-icon" :class="event.event_type.toLowerCase()">
            <el-icon v-if="event.event_type === 'RETIREMENT'"><Warning /></el-icon>
            <el-icon v-else-if="event.event_type === 'CONTRACT_EXPIRE' || event.event_type === 'CONTRACTEXPIRE'"><Document /></el-icon>
            <el-icon v-else-if="event.event_type === 'FREE_AGENT' || event.event_type === 'FREEAGENT'"><User /></el-icon>
            <el-icon v-else-if="event.event_type === 'TRANSFER_REQUEST' || event.event_type === 'TRANSFERREQUEST'"><Message /></el-icon>
            <el-icon v-else-if="event.event_type === 'LOYALTY_STAY' || event.event_type === 'LOYALTYSTAY'"><Star /></el-icon>
            <el-icon v-else-if="event.event_type === 'REBUILD_SALE' || event.event_type === 'REBUILDSALE'"><SoldOut /></el-icon>
            <el-icon v-else-if="event.event_type === 'CONTRACT_RENEWAL' || event.event_type === 'CONTRACTRENEWAL'"><DocumentChecked /></el-icon>
            <el-icon v-else-if="event.event_type === 'RENEWAL_FAILED' || event.event_type === 'RENEWALFAILED'"><DocumentRemove /></el-icon>
            <el-icon v-else-if="event.event_type === 'STAR_POACHED' || event.event_type === 'STARPOACHED'"><Trophy /></el-icon>
            <el-icon v-else><Switch /></el-icon>
          </div>

          <!-- 新闻内容 -->
          <div class="news-content">
            <div class="news-headline">{{ event.headline }}</div>
            <div class="news-description">{{ event.description }}</div>

            <!-- 选手信息 -->
            <div class="news-player-info">
              <div class="player-card-mini">
                <div class="player-avatar" :class="getPositionClass(event.position)">
                  {{ getPositionShort(event.position) }}
                </div>
                <div class="player-details">
                  <span class="player-name">{{ event.player_name }}</span>
                  <span class="player-meta">{{ event.age }}岁 · 能力 {{ event.ability }}</span>
                </div>
                <div class="player-ability" :class="getAbilityClass(event.ability)">
                  {{ event.ability }}
                </div>
              </div>

              <!-- 转会信息 -->
              <div v-if="event.transfer_fee > 0 || event.new_salary" class="transfer-details">
                <span v-if="event.transfer_fee > 0" class="transfer-fee">
                  <el-icon><Money /></el-icon>
                  转会费 {{ formatFee(event.transfer_fee) }}
                </span>
                <span v-if="event.new_salary" class="new-salary">
                  年薪 {{ formatFee(event.new_salary) }}
                </span>
                <span v-if="event.contract_years" class="contract-years">
                  {{ event.contract_years }}年合同
                </span>
              </div>

              <!-- 竞争队伍 -->
              <div v-if="event.was_bidding_war" class="bidding-war">
                <el-tag type="danger" size="small" effect="plain">
                  竞价战
                </el-tag>
              </div>
            </div>
          </div>

          <!-- 轮次标签 -->
          <div class="news-round">
            第{{ event.round }}轮
          </div>
        </div>
      </transition-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Switch,
  User,
  Money,
  Warning,
  CaretRight,
  DArrowRight,
  CircleCheck,
  Back,
  Bell,
  Document,
  Message,
  Star,
  SoldOut,
  DocumentChecked,
  DocumentRemove,
  Trophy,
} from '@element-plus/icons-vue'
import { useTransferStoreTauri } from '@/stores/useTransferStoreTauri'
import { useGameStore } from '@/stores/useGameStore'
import { transferApi, type TransferRoundInfo } from '@/api/tauri'
import { formatMoney } from '@/utils'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TransferBroadcast')

const router = useRouter()
const transferStore = useTransferStoreTauri()
const gameStore = useGameStore()

// 从 store 获取响应式数据
const {
  transferWindow,
  allTransferEvents,
  isLoading: _isLoading,
} = storeToRefs(transferStore)
const { currentSeason } = storeToRefs(gameStore)

// 本地状态
const isProcessing = ref(false)
const currentRoundSummary = ref<TransferRoundInfo | null>(null)
const filterImportance = ref('all')

// 轮次名称（0-7轮，共8轮）
const roundNames = ['赛季结算', '合同到期', '意愿处理', '自由球员', '重建清洗', '财政清洗', '强队补强', '收尾']

// 计算属性
const currentRound = computed(() => transferWindow.value?.current_round ?? 0)
const isCompleted = computed(() => transferWindow.value?.status === 'COMPLETED')
const progressPercentage = computed(() => (currentRound.value / 7) * 100)

const statusTagType = computed(() => {
  if (isCompleted.value) return 'success'
  if (currentRound.value > 0) return 'warning'
  return 'info'
})

const statusText = computed(() => {
  if (isCompleted.value) return '转会完成'
  if (currentRound.value >= 0) return `进行中 - ${roundNames[currentRound.value]}`
  return '准备开始'
})

const filteredEvents = computed(() => {
  if (filterImportance.value === 'all') {
    return [...allTransferEvents.value].reverse()
  }
  return [...allTransferEvents.value]
    .filter(e => e.importance === filterImportance.value)
    .reverse()
})

// 初始化
onMounted(async () => {
  // 检查是否有转会窗口
  await transferStore.getWindowStatus()
  if (!transferWindow.value) {
    ElMessage.warning('没有活动的转会窗口，请先开始转会')
    router.push('/transfer')
    return
  }

  // 加载已有的转会事件
  try {
    const events = await transferApi.getTransferEvents()
    if (events.length > 0) {
      // 直接更新 store 中的事件列表
      allTransferEvents.value.splice(0, allTransferEvents.value.length, ...events)
    }
  } catch (e) {
    logger.error('Failed to load transfer events:', e)
  }
})

// 执行下一轮
const handleNextRound = async () => {
  isProcessing.value = true
  try {
    const roundInfo = await transferStore.executeNextRound()
    currentRoundSummary.value = roundInfo
    ElMessage.success(`第 ${roundInfo.round} 轮完成：${roundInfo.summary}`)
  } catch (e) {
    logger.error('Failed to execute round:', e)
    ElMessage.error('执行失败')
  } finally {
    isProcessing.value = false
  }
}

// 快进完成
const handleFastForward = async () => {
  try {
    await ElMessageBox.confirm(
      '将快速完成所有剩余转会轮次。是否继续？',
      '快进完成',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    isProcessing.value = true
    await transferStore.fastForwardAll()
    ElMessage.success('转会窗口已完成！')
  } catch (e) {
    if (e !== 'cancel') {
      logger.error('Failed to fast forward:', e)
      ElMessage.error('快进失败')
    }
  } finally {
    isProcessing.value = false
  }
}

// 完成转会
const handleComplete = async () => {
  try {
    await ElMessageBox.confirm(
      '确认完成转会窗口？转会结果将生效，进入新赛季。',
      '确认完成',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'success',
      }
    )

    ElMessage.success('转会已完成！即将进入新赛季...')
    // TODO: 调用进入新赛季的 API
    router.push('/')
  } catch (e) {
    // 用户取消
  }
}

// 辅助函数
const getImportanceText = (importance: string) => {
  const map: Record<string, string> = {
    'BREAKING': '头条',
    'MAJOR': '重大',
    'NORMAL': '普通',
    'MINOR': '次要',
  }
  return map[importance] || importance
}

const getPositionShort = (position: string | null) => {
  if (!position) return '?'
  const posMap: Record<string, string> = {
    'TOP': 'TOP',
    'JUG': 'JUG',
    'JUNGLE': 'JUG',
    'MID': 'MID',
    'ADC': 'ADC',
    'BOT': 'ADC',
    'SUP': 'SUP',
    'SUPPORT': 'SUP',
  }
  return posMap[position.toUpperCase()] || position
}

const getPositionClass = (position: string | null) => {
  if (!position) return 'unknown'
  const pos = position.toLowerCase()
  if (pos === 'top') return 'top'
  if (pos === 'jug' || pos === 'jungle') return 'jungle'
  if (pos === 'mid') return 'mid'
  if (pos === 'adc' || pos === 'bot') return 'adc'
  if (pos === 'sup' || pos === 'support') return 'support'
  return 'unknown'
}

const getAbilityClass = (ability: number) => {
  if (ability >= 90) return 'legendary'
  if (ability >= 80) return 'elite'
  if (ability >= 70) return 'good'
  return 'normal'
}

// fee 的单位是元，使用统一格式化工具
const formatFee = (fee: number) => formatMoney(fee)
</script>

<style scoped>
.transfer-broadcast {
  padding: 0;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

/* 进度卡片 */
.progress-card {
  margin-bottom: 20px;
  border-radius: 12px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-title {
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.progress-text {
  font-size: 14px;
  color: #606266;
}

.round-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 12px;
}

.round-label {
  font-size: 12px;
  color: #c0c4cc;
  text-align: center;
  flex: 1;
  transition: all 0.3s ease;
}

.round-label.active {
  color: #67c23a;
}

.round-label.current {
  color: #409eff;
  font-weight: 600;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
  transition: all 0.3s ease;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.green { background: linear-gradient(135deg, #22c55e, #16a34a); }
.stat-icon.orange { background: linear-gradient(135deg, #f59e0b, #d97706); }
.stat-icon.red { background: linear-gradient(135deg, #ef4444, #dc2626); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 13px;
  color: #909399;
  margin-top: 4px;
}

/* 操作按钮 */
.action-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

/* 轮次摘要卡片 */
.round-summary-card {
  margin-bottom: 20px;
  border-radius: 12px;
  border-left: 4px solid #409eff;
  background: linear-gradient(135deg, #ecf5ff, #f4f4f5);
}

.round-summary-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.round-badge {
  padding: 4px 12px;
  background: #409eff;
  color: white;
  font-size: 12px;
  font-weight: 600;
  border-radius: 4px;
}

.round-name {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.round-summary-content {
  font-size: 14px;
  color: #606266;
  margin-bottom: 16px;
  line-height: 1.6;
}

.round-stats {
  display: flex;
  gap: 24px;
}

.round-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.round-stat .stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #409eff;
}

.round-stat .stat-label {
  font-size: 12px;
  color: #909399;
}

/* 新闻区域 */
.news-section {
  margin-bottom: 20px;
}

.news-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.news-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

/* 新闻列表 */
.news-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.news-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  border: 1px solid #ebeef5;
  position: relative;
  display: flex;
  gap: 16px;
  transition: all 0.3s ease;
}

.news-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

/* 重要性样式 */
.news-card.importance-breaking {
  border-left: 4px solid #ef4444;
  background: linear-gradient(135deg, #fef2f2, #ffffff);
}

.news-card.importance-major {
  border-left: 4px solid #f59e0b;
  background: linear-gradient(135deg, #fffbeb, #ffffff);
}

.news-card.importance-normal {
  border-left: 4px solid #3b82f6;
}

.news-card.importance-minor {
  border-left: 4px solid #9ca3af;
  opacity: 0.85;
}

/* 重要性标签 */
.news-importance {
  position: absolute;
  top: 12px;
  right: 12px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  color: white;
}

.news-importance.breaking { background: #ef4444; }
.news-importance.major { background: #f59e0b; }
.news-importance.normal { background: #3b82f6; }
.news-importance.minor { background: #9ca3af; }

/* 事件类型图标 */
.news-type-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 24px;
  flex-shrink: 0;
}

.news-type-icon.retirement { background: linear-gradient(135deg, #ef4444, #dc2626); }
.news-type-icon.contract_expire,
.news-type-icon.contractexpire { background: linear-gradient(135deg, #f59e0b, #d97706); }
.news-type-icon.free_agent,
.news-type-icon.freeagent { background: linear-gradient(135deg, #22c55e, #16a34a); }
.news-type-icon.purchase { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.news-type-icon.transfer_request,
.news-type-icon.transferrequest { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.news-type-icon.loyalty_stay,
.news-type-icon.loyaltystay { background: linear-gradient(135deg, #06b6d4, #0891b2); }
.news-type-icon.rebuild_sale,
.news-type-icon.rebuildsale { background: linear-gradient(135deg, #ec4899, #db2777); }
.news-type-icon.contract_renewal,
.news-type-icon.contractrenewal { background: linear-gradient(135deg, #10b981, #059669); }
.news-type-icon.renewal_failed,
.news-type-icon.renewalfailed { background: linear-gradient(135deg, #f97316, #ea580c); }
.news-type-icon.star_poached,
.news-type-icon.starpoached { background: linear-gradient(135deg, #f59e0b, #d97706); }

/* 新闻内容 */
.news-content {
  flex: 1;
  min-width: 0;
}

.news-headline {
  font-size: 16px;
  font-weight: 700;
  color: #303133;
  margin-bottom: 8px;
  padding-right: 60px;
}

.news-description {
  font-size: 14px;
  color: #606266;
  margin-bottom: 12px;
  line-height: 1.5;
}

/* 选手信息 */
.news-player-info {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.player-card-mini {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.player-avatar {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 11px;
}

.player-avatar.top { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-avatar.jungle { background: linear-gradient(135deg, #22c55e, #16a34a); }
.player-avatar.mid { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-avatar.adc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.player-avatar.support { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.player-avatar.unknown { background: linear-gradient(135deg, #9ca3af, #6b7280); }

.player-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.player-name {
  font-size: 13px;
  font-weight: 600;
  color: #303133;
}

.player-meta {
  font-size: 11px;
  color: #909399;
}

.player-ability {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 700;
  color: white;
}

.player-ability.legendary { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-ability.elite { background: linear-gradient(135deg, #f59e0b, #d97706); }
.player-ability.good { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-ability.normal { background: linear-gradient(135deg, #22c55e, #16a34a); }

/* 转会详情 */
.transfer-details {
  display: flex;
  gap: 12px;
  align-items: center;
}

.transfer-fee {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  font-weight: 600;
  color: #f59e0b;
}

.new-salary {
  font-size: 13px;
  color: #22c55e;
}

.contract-years {
  font-size: 13px;
  color: #606266;
}

.bidding-war {
  margin-left: auto;
}

/* 轮次标签 */
.news-round {
  position: absolute;
  bottom: 12px;
  right: 12px;
  font-size: 11px;
  color: #909399;
}

/* 过渡动画 */
.news-list-enter-active,
.news-list-leave-active {
  transition: all 0.4s ease;
}

.news-list-enter-from {
  opacity: 0;
  transform: translateX(-30px);
}

.news-list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
