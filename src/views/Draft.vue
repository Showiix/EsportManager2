<template>
  <div class="draft-view">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-info">
        <h1 class="page-title">选秀大会</h1>
        <p class="page-desc">每年进行一次选秀，为各赛区注入新鲜血液</p>
      </div>
      <div class="header-actions">
        <SeasonSelector v-model="selectedSeason" width="140px" />
        <el-button v-if="!isViewingHistory" type="primary" @click="goToPool">
          <el-icon><FolderOpened /></el-icon>
          选手池管理
        </el-button>
      </div>
    </div>

    <!-- 当前赛季：显示操作界面 -->
    <template v-if="!isViewingHistory">
      <!-- 数据概览 -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon blue">
            <el-icon :size="24"><User /></el-icon>
          </div>
          <div class="stat-body">
            <div class="stat-value">{{ totalProspects }}</div>
            <div class="stat-label">新秀总数</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon gold">
            <el-icon :size="24"><Star /></el-icon>
          </div>
          <div class="stat-body">
            <div class="stat-value">{{ eliteProspects }}</div>
            <div class="stat-label">状元候选</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon green">
            <el-icon :size="24"><Check /></el-icon>
          </div>
          <div class="stat-body">
            <div class="stat-value">{{ draftedCount }}</div>
            <div class="stat-label">已选中</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon purple">
            <el-icon :size="24"><Clock /></el-icon>
          </div>
          <div class="stat-body">
            <div class="stat-value">{{ remainingPicks }}</div>
            <div class="stat-label">剩余选秀</div>
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <el-card v-if="isLoading" class="section-block">
        <el-skeleton :rows="5" animated />
      </el-card>

      <!-- 赛区选择 -->
      <div v-else class="section-block">
        <div class="section-header">
          <h2 class="section-title">选择赛区进行选秀</h2>
        </div>

        <div class="regions-grid">
          <div
            v-for="region in regionsWithStatus"
            :key="region.id"
            class="region-card"
            :class="{ 'is-completed': region.completed }"
            @click="goToDraft(region)"
          >
            <div class="region-badge" :class="region.code?.toLowerCase()">
              {{ region.code }}
            </div>
            <div class="region-info">
              <h3 class="region-name">{{ region.name }}</h3>
              <div class="region-meta">
                <span class="meta-item">
                  <span class="meta-value">{{ region.teamCount }}</span>
                  <span class="meta-label">队伍</span>
                </span>
                <span class="meta-divider"></span>
                <span class="meta-item">
                  <span class="meta-value">{{ region.prospectCount }}</span>
                  <span class="meta-label">新秀</span>
                </span>
              </div>
            </div>
            <div class="region-status">
              <span v-if="region.completed" class="status-tag completed">
                <el-icon><Check /></el-icon>
                已完成
              </span>
              <span v-else class="status-tag pending">
                待选秀
              </span>
            </div>
            <div class="region-arrow">
              <el-icon><ArrowRight /></el-icon>
            </div>
          </div>
        </div>
      </div>

      <!-- 选秀规则 -->
      <div class="rules-section">
        <div class="rules-column">
          <div class="rules-card">
            <div class="rules-header">
              <div class="rules-icon">
                <el-icon :size="20"><Document /></el-icon>
              </div>
              <h3>选秀规则</h3>
            </div>
            <ul class="rules-list">
              <li>
                <span class="rule-bullet"></span>
                <span>选秀每年进行一次，在转会期结束后举行</span>
              </li>
              <li>
                <span class="rule-bullet"></span>
                <span>每个赛区14支队伍参与选秀，选秀池有14名新秀</span>
              </li>
              <li>
                <span class="rule-bullet"></span>
                <span>选秀顺位基于夏季赛常规赛排名：排名越靠后，获得高顺位的概率越高（但不绝对）</span>
              </li>
              <li>
                <span class="rule-bullet"></span>
                <span>新秀名单需提前导入选手池，确定状元、榜眼、探花等顺位</span>
              </li>
            </ul>
          </div>
        </div>

        <div class="rules-column">
          <div class="rules-card">
            <div class="rules-header">
              <div class="rules-icon gold">
                <el-icon :size="20"><Trophy /></el-icon>
              </div>
              <h3>选秀流程</h3>
            </div>
            <div class="flow-steps">
              <div class="flow-step">
                <div class="step-num">1</div>
                <div class="step-info">
                  <span class="step-title">导入选手池</span>
                  <span class="step-desc">管理员预先导入14名新秀数据</span>
                </div>
              </div>
              <div class="flow-step">
                <div class="step-num">2</div>
                <div class="step-info">
                  <span class="step-title">查看新秀名单</span>
                  <span class="step-desc">展示选手能力值、潜力值、标签</span>
                </div>
              </div>
              <div class="flow-step">
                <div class="step-num">3</div>
                <div class="step-info">
                  <span class="step-title">选秀权抽签</span>
                  <span class="step-desc">根据排名概率决定选秀顺位</span>
                </div>
              </div>
              <div class="flow-step">
                <div class="step-num">4</div>
                <div class="step-info">
                  <span class="step-title">分配选手</span>
                  <span class="step-desc">新秀加入对应战队</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 历史赛季：显示选秀记录 -->
    <template v-else>
      <el-card v-if="historyLoading" class="section-block">
        <el-skeleton :rows="5" animated />
      </el-card>

      <template v-else>
        <!-- 各赛区选秀结果 -->
        <div class="section-block" v-for="region in regionsWithHistoryStatus" :key="region.id">
          <div class="section-header">
            <h2 class="section-title">
              <span class="region-badge-inline" :class="region.code?.toLowerCase()">{{ region.code }}</span>
              {{ region.name }} 选秀结果
            </h2>
            <el-tag v-if="region.status === 'completed'" type="success" size="small">已完成</el-tag>
            <el-tag v-else-if="region.status === 'not_started'" type="info" size="small">未进行</el-tag>
            <el-tag v-else type="warning" size="small">{{ region.status }}</el-tag>
          </div>

          <!-- 选秀结果表格 -->
          <el-table
            v-if="region.draftResults.length > 0"
            :data="region.draftResults"
            stripe
            size="small"
            style="width: 100%"
          >
            <el-table-column prop="pick_number" label="顺位" width="70" align="center">
              <template #default="{ row }">
                <span class="pick-number" :class="{ 'top-pick': row.pick_number <= 3 }">
                  #{{ row.pick_number }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="team_name" label="战队" width="150" />
            <el-table-column prop="player_name" label="选手" width="150" />
            <el-table-column prop="position" label="位置" width="80" align="center" />
            <el-table-column prop="ability" label="能力" width="80" align="center">
              <template #default="{ row }">
                <span :class="getAbilityClass(row.ability)">{{ row.ability }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="potential" label="潜力" width="80" align="center">
              <template #default="{ row }">
                <span :class="getAbilityClass(row.potential)">{{ row.potential }}</span>
              </template>
            </el-table-column>
          </el-table>

          <el-empty v-else description="该赛区没有选秀数据" :image-size="60" />
        </div>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  User,
  Star,
  Check,
  Clock,
  ArrowRight,
  Document,
  Trophy,
  FolderOpened,
} from '@element-plus/icons-vue'
import { useDraftStoreTauri } from '@/stores/useDraftStoreTauri'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'
import { useGameStore } from '@/stores/useGameStore'
import { useSeasonStore } from '@/stores/useSeasonStore'
import { draftApi } from '@/api/tauri'
import type { DraftRegionStatus } from '@/api/tauri'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const router = useRouter()
const draftStore = useDraftStoreTauri()
const teamStore = useTeamStoreTauri()
const gameStore = useGameStore()
const seasonStore = useSeasonStore()

// 从 store 获取响应式数据
const { isLoading } = storeToRefs(draftStore)
const { regions } = storeToRefs(teamStore)

// 赛季选择
const selectedSeason = ref(0)
const isViewingHistory = computed(() => selectedSeason.value !== 0 && selectedSeason.value !== seasonStore.currentSeason)

// 各赛区选秀状态（当前赛季）
const regionStatuses = ref<Record<number, DraftRegionStatus>>({})

// 历史数据
const historyRegionStatuses = ref<Record<number, DraftRegionStatus>>({})
const historyLoading = ref(false)

// 初始化
onMounted(async () => {
  await gameStore.refreshGameState()
  selectedSeason.value = seasonStore.currentSeason
  await teamStore.loadRegions()
  await loadRegionStatuses()
})

// 加载当前赛季的赛区状态
async function loadRegionStatuses() {
  const statusPromises = regions.value.map(async (region) => {
    try {
      const status = await draftApi.getDraftRegionStatus(region.id)
      regionStatuses.value[region.id] = status
    } catch (e) {
      // 忽略单个赛区加载失败
    }
  })
  await Promise.all(statusPromises)
}

// 加载历史赛季的选秀数据
async function loadHistoryData(season: number) {
  historyLoading.value = true
  historyRegionStatuses.value = {}

  try {
    const statusPromises = regions.value.map(async (region) => {
      try {
        const status = await draftApi.getDraftRegionStatus(region.id, season)
        historyRegionStatuses.value[region.id] = status
      } catch (e) {
        // 忽略单个赛区加载失败
      }
    })
    await Promise.all(statusPromises)
  } finally {
    historyLoading.value = false
  }
}

// 监听赛季切换
watch(selectedSeason, (val) => {
  if (val !== 0 && val !== seasonStore.currentSeason) {
    loadHistoryData(val)
  }
})

// 计算属性 - 带状态的赛区列表（当前赛季）
const regionsWithStatus = computed(() => {
  return regions.value.map(region => ({
    ...region,
    completed: regionStatuses.value[region.id]?.status === 'completed',
    teamCount: 14,
    prospectCount: regionStatuses.value[region.id]?.total_players ?? 14,
  }))
})

// 计算属性 - 历史赛季的赛区列表
const regionsWithHistoryStatus = computed(() => {
  return regions.value.map(region => {
    const status = historyRegionStatuses.value[region.id]
    return {
      ...region,
      status: status?.status ?? 'not_started',
      draftResults: status?.draft_results ?? [],
    }
  })
})

// 统计数据
const totalProspects = computed(() => regionsWithStatus.value.length * 14)
const eliteProspects = computed(() => regionsWithStatus.value.length)
const draftedCount = computed(() => {
  return regionsWithStatus.value.filter(r => r.completed).length * 14
})
const remainingPicks = computed(() => totalProspects.value - draftedCount.value)

// 能力值颜色
function getAbilityClass(value: number): string {
  if (value >= 85) return 'ability-legendary'
  if (value >= 75) return 'ability-high'
  if (value >= 65) return 'ability-mid'
  return 'ability-low'
}

// 方法
const goToDraft = (region: any) => {
  router.push(`/draft/${region.code?.toLowerCase() ?? region.id}`)
}

const goToPool = () => {
  router.push('/draft/pool')
}
</script>

<style scoped lang="scss">
.draft-view {
  padding: 0;
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-info {
  .page-title {
    font-size: 28px;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 6px 0;
  }

  .page-desc {
    font-size: 14px;
    color: #6b7280;
    margin: 0;
  }
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 24px;
  font-size: 14px;
  font-weight: 600;

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  &.active {
    background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
    color: #166534;

    .status-dot {
      background: #22c55e;
      box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
      animation: pulse 2s infinite;
    }
  }

  &.inactive {
    background: #f3f4f6;
    color: #6b7280;

    .status-dot {
      background: #9ca3af;
    }
  }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* 通知横幅 */
.notice-banner {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
  border: 1px solid #bfdbfe;
  border-radius: 12px;
  margin-bottom: 24px;

  .notice-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: #3b82f6;
    border-radius: 10px;
    color: white;
  }

  .notice-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .notice-title {
    font-size: 15px;
    font-weight: 600;
    color: #1e40af;
  }

  .notice-desc {
    font-size: 13px;
    color: #3b82f6;
  }
}

/* 数据概览 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: white;
  border-radius: 16px;
  border: 1px solid #e5e7eb;
  transition: all 0.2s ease;

  &:hover {
    border-color: #d1d5db;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  }

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 52px;
    height: 52px;
    border-radius: 14px;
    color: white;

    &.blue {
      background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    }

    &.gold {
      background: linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%);
    }

    &.green {
      background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
    }

    &.purple {
      background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
    }
  }

  .stat-body {
    flex: 1;
  }

  .stat-value {
    font-size: 28px;
    font-weight: 700;
    color: #1f2937;
    line-height: 1;
  }

  .stat-label {
    font-size: 13px;
    color: #6b7280;
    margin-top: 4px;
  }
}

/* 区块 */
.section-block {
  background: white;
  border-radius: 16px;
  border: 1px solid #e5e7eb;
  padding: 24px;
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;

  .section-title {
    font-size: 18px;
    font-weight: 600;
    color: #1f2937;
    margin: 0;
  }
}

/* 赛区卡片网格 */
.regions-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.region-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: #f9fafb;
  border: 2px solid transparent;
  border-radius: 14px;
  cursor: pointer;
  transition: all 0.25s ease;

  &:hover {
    background: white;
    border-color: #3b82f6;
    box-shadow: 0 4px 16px rgba(59, 130, 246, 0.15);
    transform: translateY(-2px);

    .region-arrow {
      opacity: 1;
      transform: translateX(0);
    }
  }

  &.is-completed {
    background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
    border-color: #86efac;
  }
}

.region-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: 14px;
  font-size: 14px;
  font-weight: 700;
  color: white;
  letter-spacing: 0.5px;

  &.lpl {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  }

  &.lck {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  }

  &.lec {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  }

  &.lcs {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  }
}

.region-info {
  flex: 1;

  .region-name {
    font-size: 16px;
    font-weight: 600;
    color: #1f2937;
    margin: 0 0 8px 0;
  }

  .region-meta {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .meta-item {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .meta-value {
    font-size: 16px;
    font-weight: 700;
    color: #374151;
  }

  .meta-label {
    font-size: 12px;
    color: #9ca3af;
  }

  .meta-divider {
    width: 1px;
    height: 12px;
    background: #d1d5db;
  }
}

.region-status {
  .status-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;

    &.completed {
      background: #dcfce7;
      color: #166534;
    }

    &.pending {
      background: #dbeafe;
      color: #1e40af;
    }

    &.locked {
      background: #f3f4f6;
      color: #6b7280;
    }
  }
}

.region-arrow {
  color: #3b82f6;
  opacity: 0;
  transform: translateX(-8px);
  transition: all 0.25s ease;
}

/* 规则区块 */
.rules-section {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.rules-card {
  background: white;
  border-radius: 16px;
  border: 1px solid #e5e7eb;
  padding: 24px;
  height: 100%;
}

.rules-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;

  .rules-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    border-radius: 10px;
    color: white;

    &.gold {
      background: linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%);
    }
  }

  h3 {
    font-size: 16px;
    font-weight: 600;
    color: #1f2937;
    margin: 0;
  }
}

.rules-list {
  list-style: none;
  padding: 0;
  margin: 0;

  li {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid #f3f4f6;
    font-size: 14px;
    color: #4b5563;
    line-height: 1.5;

    &:last-child {
      border-bottom: none;
      padding-bottom: 0;
    }

    &:first-child {
      padding-top: 0;
    }
  }

  .rule-bullet {
    width: 6px;
    height: 6px;
    background: #3b82f6;
    border-radius: 50%;
    margin-top: 7px;
    flex-shrink: 0;
  }
}

/* 评级列表 */
.flow-steps {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.flow-step {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: #f9fafb;
  border-radius: 10px;
  transition: all 0.2s ease;

  &:hover {
    background: #f3f4f6;
  }

  .step-num {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    color: white;
    font-size: 14px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .step-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .step-title {
    font-size: 14px;
    font-weight: 600;
    color: #1f2937;
  }

  .step-desc {
    font-size: 12px;
    color: #6b7280;
  }
}

/* 历史模式 */
.region-badge-inline {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 24px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  color: white;
  margin-right: 8px;

  &.lpl {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  }
  &.lck {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  }
  &.lec {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  }
  &.lcs {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  }
}

.pick-number {
  font-weight: 600;
  color: #6b7280;

  &.top-pick {
    color: #f59e0b;
    font-weight: 700;
  }
}

.ability-legendary {
  color: #ef4444;
  font-weight: 700;
}

.ability-high {
  color: #f59e0b;
  font-weight: 600;
}

.ability-mid {
  color: #3b82f6;
  font-weight: 500;
}

.ability-low {
  color: #6b7280;
}
</style>
