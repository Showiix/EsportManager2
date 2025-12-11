<template>
  <div class="transfer-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>转会中心</h1>
        <p>{{ currentSeason }} 赛季 - 休赛期转会预览</p>
      </div>
      <div class="header-actions">
        <el-button
          type="primary"
          size="large"
          :loading="isLoading"
          @click="handleStartTransfer"
        >
          <el-icon><VideoPlay /></el-icon>
          开始转会窗口
        </el-button>
      </div>
    </div>

    <!-- 概览统计 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="28"><Document /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ expiringContracts.length }}</div>
              <div class="stat-label">合同到期</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon red">
              <el-icon :size="28"><Warning /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ retiringCandidates.length }}</div>
              <div class="stat-label">潜在退役</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><Goods /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ listings.length }}</div>
              <div class="stat-label">挂牌出售</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card highlight-card">
          <div class="stat-content">
            <div class="stat-icon gold">
              <el-icon :size="28"><Star /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ highlightCount }}</div>
              <div class="stat-label">重点关注</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 重点关注区域 -->
    <el-card v-if="highlightCount > 0" class="section-card highlight-section">
      <template #header>
        <div class="section-header">
          <div class="section-title">
            <el-icon class="title-icon highlight"><Star /></el-icon>
            <span>重点关注</span>
            <el-tag type="warning" effect="dark" size="small">能力 ≥ 80</el-tag>
          </div>
          <span class="section-subtitle">高能力选手动态，值得关注！</span>
        </div>
      </template>

      <div class="highlight-grid">
        <!-- 高能力合同到期选手 -->
        <div v-for="player in highlightExpiring" :key="'exp-' + player.player_id" class="highlight-card-item expiring">
          <div class="highlight-badge">合同到期</div>
          <div class="highlight-content">
            <div class="player-avatar" :class="getPositionClass(player)">
              {{ getPositionShort(player) }}
            </div>
            <div class="player-info">
              <div class="player-name">{{ player.player_name }}</div>
              <div class="player-meta">
                <span class="age">{{ player.age }}岁</span>
              </div>
            </div>
            <div class="ability-badge" :class="getAbilityClass(player.ability)">
              {{ player.ability }}
            </div>
          </div>
          <div class="highlight-footer">
            <span class="salary">年薪 {{ formatSalary(player.salary) }}</span>
            <el-button size="small" type="primary" link @click="viewPlayer(player.player_id)">
              查看详情 <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>
        </div>

        <!-- 高能力挂牌选手 -->
        <div v-for="player in highlightListings" :key="'lst-' + player.id" class="highlight-card-item listing">
          <div class="highlight-badge listing">挂牌出售</div>
          <div class="highlight-content">
            <div class="player-avatar" :class="getListingPositionClass(player)">
              {{ getPositionShortFromString(player.position) }}
            </div>
            <div class="player-info">
              <div class="player-name">{{ player.player_name }}</div>
              <div class="player-meta">
                <span class="team">{{ player.seller_team_name }}</span>
              </div>
            </div>
            <div class="ability-badge" :class="getAbilityClass(player.ability)">
              {{ player.ability }}
            </div>
          </div>
          <div class="highlight-footer">
            <span class="price">转会费 {{ formatPrice(player.asking_price) }}</span>
            <el-button size="small" type="primary" link @click="viewPlayer(player.player_id)">
              查看详情 <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 主要内容区域 -->
    <el-row :gutter="20">
      <!-- 合同到期选手 -->
      <el-col :span="8">
        <el-card class="section-card">
          <template #header>
            <div class="section-header">
              <div class="section-title">
                <el-icon class="title-icon expiring"><Document /></el-icon>
                <span>合同到期</span>
                <el-tag type="warning" size="small">{{ expiringContracts.length }}人</el-tag>
              </div>
            </div>
          </template>

          <el-skeleton v-if="isLoading" :rows="6" animated />

          <el-empty v-else-if="expiringContracts.length === 0" description="暂无合同到期选手" :image-size="80" />

          <div v-else class="player-list">
            <div
              v-for="player in expiringContracts"
              :key="player.player_id"
              class="player-item"
              @click="viewPlayer(player.player_id)"
            >
              <div class="player-avatar small" :class="getPositionClass(player)">
                {{ getPositionShort(player) }}
              </div>
              <div class="player-info">
                <div class="player-name">{{ player.player_name }}</div>
                <div class="player-meta">
                  <span class="age">{{ player.age }}岁</span>
                  <span class="salary">{{ formatSalary(player.salary) }}/年</span>
                </div>
              </div>
              <div class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
                {{ player.ability }}
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 潜在退役选手 -->
      <el-col :span="8">
        <el-card class="section-card">
          <template #header>
            <div class="section-header">
              <div class="section-title">
                <el-icon class="title-icon retiring"><Warning /></el-icon>
                <span>潜在退役</span>
                <el-tag type="danger" size="small">{{ retiringCandidates.length }}人</el-tag>
              </div>
            </div>
          </template>

          <el-skeleton v-if="isLoading" :rows="6" animated />

          <el-empty v-else-if="retiringCandidates.length === 0" description="暂无潜在退役选手" :image-size="80" />

          <div v-else class="player-list">
            <div
              v-for="player in retiringCandidates"
              :key="player.player_id"
              class="player-item retiring"
              @click="viewPlayer(player.player_id)"
            >
              <div class="player-avatar small" :class="getRetiringPositionClass(player)">
                {{ getRetiringPositionShort(player) }}
              </div>
              <div class="player-info">
                <div class="player-name">{{ player.player_name }}</div>
                <div class="player-meta">
                  <span class="age warning">{{ player.age }}岁</span>
                  <el-tag size="small" type="danger" effect="plain">{{ player.reason_description }}</el-tag>
                </div>
              </div>
              <div class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
                {{ player.ability }}
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 挂牌出售选手 -->
      <el-col :span="8">
        <el-card class="section-card">
          <template #header>
            <div class="section-header">
              <div class="section-title">
                <el-icon class="title-icon listing"><Goods /></el-icon>
                <span>挂牌出售</span>
                <el-tag type="primary" size="small">{{ listings.length }}人</el-tag>
              </div>
            </div>
          </template>

          <el-skeleton v-if="isLoading" :rows="6" animated />

          <el-empty v-else-if="listings.length === 0" description="暂无挂牌选手" :image-size="80" />

          <div v-else class="player-list">
            <div
              v-for="player in listings"
              :key="player.id"
              class="player-item listing"
              @click="viewPlayer(player.player_id)"
            >
              <div class="player-avatar small" :class="getListingPositionClass(player)">
                {{ getPositionShortFromString(player.position) }}
              </div>
              <div class="player-info">
                <div class="player-name">{{ player.player_name }}</div>
                <div class="player-meta">
                  <span class="team">{{ player.seller_team_name }}</span>
                  <span class="price">{{ formatPrice(player.asking_price) }}</span>
                </div>
              </div>
              <div class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
                {{ player.ability }}
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 底部提示 -->
    <el-card class="info-card">
      <el-icon class="info-icon"><InfoFilled /></el-icon>
      <div class="info-content">
        <div class="info-title">关于转会窗口</div>
        <div class="info-desc">
          点击「开始转会窗口」后，AI 球队将自动进行转会操作。转会分为 5 个阶段：
          <strong>合同处理</strong> → <strong>自由球员争夺</strong> → <strong>财政清洗</strong> → <strong>强队补强</strong> → <strong>收尾补救</strong>。
          你可以观看每个阶段的转会新闻播报。
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  VideoPlay,
  Document,
  Warning,
  Goods,
  Star,
  ArrowRight,
  InfoFilled,
} from '@element-plus/icons-vue'
import { useTransferStoreTauri } from '@/stores/useTransferStoreTauri'
import { useGameStore } from '@/stores/useGameStore'

const router = useRouter()
const transferStore = useTransferStoreTauri()
const gameStore = useGameStore()

// 从 store 获取响应式数据
const {
  listings,
  expiringContracts,
  retiringCandidates,
  isLoading,
  highlightPlayers,
} = storeToRefs(transferStore)
const { currentSeason } = storeToRefs(gameStore)

// 高能力选手列表
const highlightExpiring = computed(() => highlightPlayers.value.expiring)
const highlightListings = computed(() => highlightPlayers.value.listed)
const highlightCount = computed(() => highlightExpiring.value.length + highlightListings.value.length)

// 初始化加载数据
onMounted(async () => {
  try {
    await transferStore.loadPreviewData()
  } catch (e) {
    console.error('Failed to load preview data:', e)
    ElMessage.error('加载预览数据失败')
  }
})

// 开始转会窗口
const handleStartTransfer = async () => {
  try {
    await ElMessageBox.confirm(
      '开始转会窗口后，AI 球队将自动进行转会操作。你可以观看转会新闻播报。是否开始？',
      '开始转会窗口',
      {
        confirmButtonText: '开始',
        cancelButtonText: '取消',
        type: 'info',
      }
    )

    // 开始转会窗口
    await transferStore.startTransferWindow()
    ElMessage.success('转会窗口已开启！')

    // 跳转到转会播报页面
    router.push('/transfer/broadcast')
  } catch (e) {
    if (e !== 'cancel') {
      console.error('Failed to start transfer window:', e)
      ElMessage.error('开启转会窗口失败')
    }
  }
}

// 查看选手详情
const viewPlayer = (playerId: number) => {
  router.push(`/players/${playerId}`)
}

// 辅助函数
const getPositionShort = (player: { player_id: number }) => {
  // 对于 ExpiringContract，没有 position 字段，需要从其他地方获取或显示默认值
  return '?'
}

const getPositionClass = (_player: { player_id: number }) => {
  return 'unknown'
}

const getRetiringPositionShort = (_player: { player_id: number }) => {
  return '?'
}

const getRetiringPositionClass = (_player: { player_id: number }) => {
  return 'unknown'
}

const getPositionShortFromString = (position: string) => {
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
  return posMap[position?.toUpperCase()] || position || '?'
}

const getListingPositionClass = (player: { position: string }) => {
  const pos = player.position?.toLowerCase()
  if (pos === 'top') return 'top'
  if (pos === 'jug' || pos === 'jungle') return 'jungle'
  if (pos === 'mid') return 'mid'
  if (pos === 'adc' || pos === 'bot') return 'adc'
  if (pos === 'sup' || pos === 'support') return 'support'
  return 'unknown'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getAbilityClass = (ability: number) => {
  if (ability >= 90) return 'legendary'
  if (ability >= 80) return 'elite'
  if (ability >= 70) return 'good'
  return 'normal'
}

const formatSalary = (salary: number) => {
  if (salary >= 1000) return `${(salary / 100).toFixed(0)}万`
  return `${salary}万`
}

const formatPrice = (price: number) => {
  if (price >= 10000) return `${(price / 10000).toFixed(0)}亿`
  if (price >= 100) return `${(price / 100).toFixed(0)}万`
  return `${price}万`
}
</script>

<style scoped>
.transfer-view {
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

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.orange { background: linear-gradient(135deg, #f59e0b, #d97706); }
.stat-icon.red { background: linear-gradient(135deg, #ef4444, #dc2626); }
.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.highlight-card {
  border: 2px solid #fbbf24;
  background: linear-gradient(135deg, #fffbeb, #fef3c7);
}

/* 区块卡片 */
.section-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.title-icon {
  font-size: 20px;
}

.title-icon.expiring { color: #f59e0b; }
.title-icon.retiring { color: #ef4444; }
.title-icon.listing { color: #3b82f6; }
.title-icon.highlight { color: #fbbf24; }

.section-subtitle {
  font-size: 13px;
  color: #909399;
}

/* 重点关注区域 */
.highlight-section {
  border: 2px solid #fbbf24;
  background: linear-gradient(135deg, #fffbeb, #ffffff);
}

.highlight-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.highlight-card-item {
  background: white;
  border-radius: 12px;
  padding: 16px;
  border: 1px solid #ebeef5;
  position: relative;
  transition: all 0.3s ease;
}

.highlight-card-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.highlight-card-item.expiring {
  border-left: 4px solid #f59e0b;
}

.highlight-card-item.listing {
  border-left: 4px solid #3b82f6;
}

.highlight-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  color: white;
  background: #f59e0b;
}

.highlight-badge.listing {
  background: #3b82f6;
}

.highlight-content {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.highlight-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #f0f0f0;
}

.highlight-footer .salary,
.highlight-footer .price {
  font-size: 13px;
  font-weight: 600;
  color: #606266;
}

/* 选手列表 */
.player-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
}

.player-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.player-item:hover {
  background: #e8f4ff;
  transform: translateX(4px);
}

.player-item.retiring {
  background: #fef2f2;
}

.player-item.retiring:hover {
  background: #fee2e2;
}

.player-item.listing {
  background: #eff6ff;
}

.player-item.listing:hover {
  background: #dbeafe;
}

/* 选手头像 */
.player-avatar {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 12px;
  flex-shrink: 0;
}

.player-avatar.small {
  width: 36px;
  height: 36px;
  font-size: 11px;
}

.player-avatar.top { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-avatar.jungle { background: linear-gradient(135deg, #22c55e, #16a34a); }
.player-avatar.mid { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-avatar.adc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.player-avatar.support { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.player-avatar.unknown { background: linear-gradient(135deg, #9ca3af, #6b7280); }

/* 选手信息 */
.player-info {
  flex: 1;
  min-width: 0;
}

.player-name {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.player-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.player-meta .age {
  font-size: 12px;
  color: #909399;
}

.player-meta .age.warning {
  color: #ef4444;
  font-weight: 600;
}

.player-meta .salary,
.player-meta .price {
  font-size: 12px;
  color: #606266;
}

.player-meta .team {
  font-size: 12px;
  color: #606266;
}

/* 能力值 */
.ability-number {
  font-size: 18px;
  font-weight: 700;
  min-width: 36px;
  text-align: center;
}

.ability-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 16px;
  font-weight: 700;
  color: white;
}

.ability-badge.legendary { background: linear-gradient(135deg, #ef4444, #dc2626); }
.ability-badge.elite { background: linear-gradient(135deg, #f59e0b, #d97706); }
.ability-badge.good { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.ability-badge.normal { background: linear-gradient(135deg, #22c55e, #16a34a); }

/* 信息卡片 */
.info-card {
  border-radius: 12px;
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 20px;
  background: linear-gradient(135deg, #f0f9ff, #e0f2fe);
  border: 1px solid #bae6fd;
}

.info-icon {
  font-size: 24px;
  color: #0284c7;
  flex-shrink: 0;
}

.info-content {
  flex: 1;
}

.info-title {
  font-size: 15px;
  font-weight: 600;
  color: #0369a1;
  margin-bottom: 8px;
}

.info-desc {
  font-size: 13px;
  color: #0c4a6e;
  line-height: 1.6;
}

.info-desc strong {
  color: #0284c7;
}

/* 滚动条样式 */
.player-list::-webkit-scrollbar {
  width: 6px;
}

.player-list::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.player-list::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.player-list::-webkit-scrollbar-thumb:hover {
  background: #a1a1a1;
}
</style>
