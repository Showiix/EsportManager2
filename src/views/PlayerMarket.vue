<template>
  <div class="player-market-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>选手市场</h1>
        <p>查看所有选手的合同、身价、满意度和忠诚度信息</p>
      </div>
      <div class="header-actions">
        <el-button @click="loadPlayerList" :loading="isLoading">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button type="warning" @click="batchGenerateStrategies" :loading="isBatchGenerating">
          <el-icon><MagicStick /></el-icon>
          批量生成AI策略
        </el-button>
        <el-button type="primary" @click="showDepartureCandidates" :loading="isLoadingStrategies">
          <el-icon><TrendCharts /></el-icon>
          查看离队意向
        </el-button>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-select v-model="filters.position" placeholder="全部位置" clearable style="width: 120px">
        <el-option label="全部位置" value="" />
        <el-option label="TOP" value="TOP" />
        <el-option label="JUG" value="JUG" />
        <el-option label="MID" value="MID" />
        <el-option label="ADC" value="ADC" />
        <el-option label="SUP" value="SUP" />
      </el-select>
      <el-select v-model="filters.region" placeholder="全部赛区" clearable style="width: 120px">
        <el-option label="全部赛区" value="" />
        <el-option label="LPL" value="LPL" />
        <el-option label="LCK" value="LCK" />
        <el-option label="LEC" value="LEC" />
        <el-option label="LCS" value="LCS" />
      </el-select>
      <el-select v-model="filters.contractStatus" placeholder="合同状态" clearable style="width: 140px">
        <el-option label="全部状态" value="" />
        <el-option label="即将到期" value="expiring" />
        <el-option label="已到期" value="expired" />
        <el-option label="长约" value="long" />
      </el-select>
      <el-select v-model="filters.transferIntent" placeholder="转会意愿" clearable style="width: 130px">
        <el-option label="全部" value="" />
        <el-option label="想离队" value="leave" />
        <el-option label="愿意留" value="stay" />
      </el-select>
      <el-select v-model="filters.satisfaction" placeholder="满意度" clearable style="width: 130px">
        <el-option label="全部" value="" />
        <el-option label="不满 (<40)" value="low" />
        <el-option label="一般 (40-70)" value="mid" />
        <el-option label="满意 (>70)" value="high" />
      </el-select>
      <el-input
        v-model="filters.search"
        placeholder="搜索选手..."
        style="width: 180px"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <!-- 选手表格 -->
    <el-card class="player-table-card">
      <el-table
        :data="paginatedPlayers"
        v-loading="isLoading"
        stripe
        style="width: 100%"
        @row-click="handleRowClick"
        row-class-name="clickable-row"
      >
        <el-table-column prop="player_name" label="选手" width="150" fixed>
          <template #default="{ row }">
            <div class="player-cell">
              <div class="player-avatar" :class="row.position?.toLowerCase()">
                {{ row.position || '?' }}
              </div>
              <span class="player-name-text">{{ row.player_name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="team_name" label="战队" width="140">
          <template #default="{ row }">
            <div v-if="row.team_name" class="team-cell">
              <span>{{ row.team_name }}</span>
              <el-tag v-if="row.region_code" size="small" :type="getRegionTagType(row.region_code)">
                {{ formatRegion(row.region_code) }}
              </el-tag>
            </div>
            <span v-else class="no-team">自由身</span>
          </template>
        </el-table-column>
        <el-table-column prop="ability" label="能力" width="70" align="center" sortable>
          <template #default="{ row }">
            <span :style="{ color: getAbilityColor(row.ability), fontWeight: 700 }">
              {{ row.ability }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="身价" width="100" align="right" sortable :sort-method="sortByValue">
          <template #default="{ row }">
            <span class="value-text">{{ formatValue(row.calculated_market_value || row.base_market_value) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="salary" label="年薪" width="90" align="right" sortable>
          <template #default="{ row }">
            <span>{{ formatSalary(row.salary) }}/年</span>
          </template>
        </el-table-column>
        <el-table-column label="合同" width="90" align="center">
          <template #default="{ row }">
            <span :class="getContractClass(row.contract_end_season)">
              {{ row.contract_end_season ? `${row.contract_end_season}赛季` : '已到期' }}
            </span>
            <el-icon v-if="isExpiringSoon(row.contract_end_season)" class="expire-warning"><WarningFilled /></el-icon>
          </template>
        </el-table-column>
        <el-table-column label="满意度" width="100" align="center" sortable :sort-by="'satisfaction'">
          <template #default="{ row }">
            <el-progress
              :percentage="row.satisfaction"
              :stroke-width="8"
              :color="getSatisfactionColor(row.satisfaction)"
              :show-text="false"
              style="width: 50px; display: inline-block;"
            />
            <span :class="getSatisfactionClass(row.satisfaction)" style="margin-left: 4px;">
              {{ row.satisfaction }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="忠诚度" width="100" align="center" sortable :sort-by="'loyalty'">
          <template #default="{ row }">
            <el-progress
              :percentage="row.loyalty"
              :stroke-width="8"
              :color="getLoyaltyColor(row.loyalty)"
              :show-text="false"
              style="width: 50px; display: inline-block;"
            />
            <span :class="getLoyaltyClass(row.loyalty)" style="margin-left: 4px;">
              {{ row.loyalty }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="意愿" width="80" align="center">
          <template #default="{ row }">
            <el-tag
              :type="row.wants_to_leave ? 'danger' : 'success'"
              size="small"
              effect="plain"
            >
              {{ row.wants_to_leave ? '想走' : '想留' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="age" label="年龄" width="60" align="center" sortable />
        <el-table-column label="AI策略" width="100" align="center" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              :type="hasStrategy(row.player_id) ? 'success' : 'default'"
              @click.stop="openStrategyDialog(row)"
            >
              <el-icon><DataAnalysis /></el-icon>
              {{ hasStrategy(row.player_id) ? '查看' : '分析' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[20, 50, 100]"
          :total="filteredPlayers.length"
          layout="total, sizes, prev, pager, next"
          background
        />
      </div>
    </el-card>

    <!-- 选手合同详情弹窗 -->
    <PlayerContractDialog
      v-model:visible="contractDialogVisible"
      :player-id="selectedPlayerId"
    />

    <!-- 选手AI策略弹窗 -->
    <PlayerStrategyDialog
      v-model:visible="strategyDialogVisible"
      :player-id="strategyPlayerId"
      :player-name="strategyPlayerName"
      @strategy-generated="onStrategyGenerated"
    />

    <!-- 离队意向列表弹窗 -->
    <el-dialog
      v-model="departureCandidatesVisible"
      title="离队意向选手"
      width="900px"
    >
      <el-table :data="departureCandidates" v-loading="isLoadingStrategies" stripe>
        <el-table-column prop="player_name" label="选手" width="120">
          <template #default="{ row }">
            <div class="player-cell">
              <div class="player-avatar" :class="row.position?.toLowerCase()">
                {{ row.position || '?' }}
              </div>
              <span>{{ row.player_name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="team_name" label="战队" width="120" />
        <el-table-column prop="ability" label="能力" width="70" align="center" />
        <el-table-column label="离队原因" min-width="150">
          <template #default="{ row }">
            <div class="reason-tags">
              <el-tag
                v-for="reason in row.departure_reasons"
                :key="reason"
                size="small"
                type="warning"
              >
                {{ reason }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="偏好球队" width="150">
          <template #default="{ row }">
            <span v-if="row.preferred_teams_count > 0">
              {{ row.preferred_teams[0]?.team_name }}
              <span v-if="row.preferred_teams_count > 1" class="more-teams">
                +{{ row.preferred_teams_count - 1 }}
              </span>
            </span>
            <span v-else class="no-data">-</span>
          </template>
        </el-table-column>
        <el-table-column label="置信度" width="80" align="center">
          <template #default="{ row }">
            <el-progress
              :percentage="row.decision_confidence"
              :stroke-width="6"
              :show-text="false"
              style="width: 50px"
            />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="80" align="center">
          <template #default="{ row }">
            <el-button size="small" @click="viewStrategyFromList(row)">
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, Refresh, WarningFilled, DataAnalysis, TrendCharts, MagicStick } from '@element-plus/icons-vue'
import { transferApi, aiTransferApi, type PlayerMarketInfo, type PlayerTransferStrategyInfo } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import PlayerContractDialog from '@/components/transfer/PlayerContractDialog.vue'
import PlayerStrategyDialog from '@/components/transfer/PlayerStrategyDialog.vue'

const gameStore = useGameStore()

// 数据
const players = ref<PlayerMarketInfo[]>([])
const isLoading = ref(false)
const currentPage = ref(1)
const pageSize = ref(50)

// 弹窗状态
const contractDialogVisible = ref(false)
const selectedPlayerId = ref<number | null>(null)

// AI策略弹窗状态
const strategyDialogVisible = ref(false)
const strategyPlayerId = ref<number | null>(null)
const strategyPlayerName = ref('')

// 离队意向列表
const departureCandidatesVisible = ref(false)
const departureCandidates = ref<PlayerTransferStrategyInfo[]>([])
const isLoadingStrategies = ref(false)

// 批量生成状态
const isBatchGenerating = ref(false)
const batchProgress = ref({ current: 0, total: 0 })

// 已生成策略的选手ID集合
const playersWithStrategy = ref<Set<number>>(new Set())

// 筛选条件
const filters = reactive({
  position: '',
  region: '',
  contractStatus: '',
  transferIntent: '',
  satisfaction: '',
  search: '',
})

// 当前赛季
const currentSeason = computed(() => gameStore.currentSeason)

// 加载选手列表
const loadPlayerList = async () => {
  isLoading.value = true
  try {
    players.value = await transferApi.getPlayerMarketList()
    // 加载已生成策略的选手列表
    await loadExistingStrategies()
  } catch (e) {
    console.error('Failed to load player list:', e)
    ElMessage.error('加载选手列表失败')
  } finally {
    isLoading.value = false
  }
}

// 加载已生成策略的选手ID
const loadExistingStrategies = async () => {
  try {
    const strategies = await aiTransferApi.getAllPlayerStrategies()
    playersWithStrategy.value = new Set(strategies.map(s => s.player_id))
  } catch (e) {
    console.error('Failed to load existing strategies:', e)
  }
}

// 检查选手是否已有策略
const hasStrategy = (playerId: number) => {
  return playersWithStrategy.value.has(playerId)
}

// 过滤后的选手列表
const filteredPlayers = computed(() => {
  return players.value.filter(player => {
    // 位置筛选
    if (filters.position && player.position?.toUpperCase() !== filters.position) return false

    // 赛区筛选
    if (filters.region && player.region_code !== filters.region) return false

    // 合同状态筛选
    if (filters.contractStatus) {
      const season = currentSeason.value
      const endSeason = player.contract_end_season
      if (filters.contractStatus === 'expiring' && !(endSeason && endSeason <= season + 1)) return false
      if (filters.contractStatus === 'expired' && endSeason !== null) return false
      if (filters.contractStatus === 'long' && !(endSeason && endSeason > season + 1)) return false
    }

    // 转会意愿筛选
    if (filters.transferIntent === 'leave' && !player.wants_to_leave) return false
    if (filters.transferIntent === 'stay' && player.wants_to_leave) return false

    // 满意度筛选
    if (filters.satisfaction === 'low' && player.satisfaction >= 40) return false
    if (filters.satisfaction === 'mid' && (player.satisfaction < 40 || player.satisfaction > 70)) return false
    if (filters.satisfaction === 'high' && player.satisfaction <= 70) return false

    // 搜索
    if (filters.search && !player.player_name.toLowerCase().includes(filters.search.toLowerCase())) return false

    return true
  })
})

// 分页后的选手列表
const paginatedPlayers = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredPlayers.value.slice(start, end)
})

// 点击行打开详情弹窗
const handleRowClick = (row: PlayerMarketInfo) => {
  selectedPlayerId.value = row.player_id
  contractDialogVisible.value = true
}

// 辅助函数
const formatRegion = (region: string | null) => {
  const regionMap: Record<string, string> = {
    'CN': 'LPL',
    'KR': 'LCK',
    'EU': 'LEC',
    'NA': 'LCS',
    'LPL': 'LPL',
    'LCK': 'LCK',
    'LEC': 'LEC',
    'LCS': 'LCS',
  }
  return regionMap[region || ''] || region || ''
}

const getRegionTagType = (region: string | null) => {
  const formatted = formatRegion(region)
  const types: Record<string, string> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning',
  }
  return types[formatted] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const formatValue = (value: number) => {
  // 身价单位已经是万
  if (value >= 10000) return `${(value / 10000).toFixed(1)}亿`
  if (value >= 1) return `${Math.round(value)}万`
  return `${value}万`
}

const formatSalary = (salary: number) => {
  // 薪资单位已经是万
  if (salary >= 10000) return `${(salary / 10000).toFixed(1)}亿`
  return `${Math.round(salary)}万`
}

const sortByValue = (a: PlayerMarketInfo, b: PlayerMarketInfo) => {
  const va = a.calculated_market_value || a.base_market_value
  const vb = b.calculated_market_value || b.base_market_value
  return va - vb
}

const getContractClass = (endSeason: number | null) => {
  if (!endSeason) return 'contract-expired'
  const season = currentSeason.value
  if (endSeason <= season) return 'contract-expired'
  if (endSeason === season + 1) return 'contract-expiring'
  return ''
}

const isExpiringSoon = (endSeason: number | null) => {
  if (!endSeason) return true
  return endSeason <= currentSeason.value + 1
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

// 打开AI策略弹窗
const openStrategyDialog = (row: PlayerMarketInfo) => {
  strategyPlayerId.value = row.player_id
  strategyPlayerName.value = row.player_name
  strategyDialogVisible.value = true
}

// 策略生成完成回调
const onStrategyGenerated = (playerId: number) => {
  playersWithStrategy.value.add(playerId)
}

// 查看离队意向列表
const showDepartureCandidates = async () => {
  isLoadingStrategies.value = true
  departureCandidatesVisible.value = true
  try {
    departureCandidates.value = await aiTransferApi.getAllPlayerStrategies()
  } catch (e) {
    console.error('Failed to load departure candidates:', e)
    ElMessage.error('加载离队意向列表失败')
  } finally {
    isLoadingStrategies.value = false
  }
}

// 从离队列表查看详情
const viewStrategyFromList = (row: PlayerTransferStrategyInfo) => {
  strategyPlayerId.value = row.player_id
  strategyPlayerName.value = row.player_name
  strategyDialogVisible.value = true
}

// 批量生成AI策略
const batchGenerateStrategies = async () => {
  // 只为有球队的选手生成策略
  const playersWithTeam = players.value.filter(p => p.team_id !== null)

  if (playersWithTeam.length === 0) {
    ElMessage.warning('没有可生成策略的选手')
    return
  }

  try {
    await ElMessageBox.confirm(
      `将为 ${playersWithTeam.length} 名选手生成AI策略，这可能需要一些时间。是否继续？`,
      '批量生成确认',
      {
        confirmButtonText: '开始生成',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
  } catch {
    return // 用户取消
  }

  isBatchGenerating.value = true
  batchProgress.value = { current: 0, total: playersWithTeam.length }

  let successCount = 0
  let failCount = 0

  for (let i = 0; i < playersWithTeam.length; i++) {
    const player = playersWithTeam[i]
    batchProgress.value.current = i + 1

    try {
      await aiTransferApi.generatePlayerTransferStrategy(player.player_id)
      // 更新已生成策略的集合
      playersWithStrategy.value.add(player.player_id)
      successCount++
    } catch (e) {
      console.error(`Failed to generate strategy for ${player.player_name}:`, e)
      failCount++
    }

    // 每10个显示一次进度
    if ((i + 1) % 10 === 0) {
      ElMessage.info(`进度: ${i + 1}/${playersWithTeam.length}`)
    }
  }

  isBatchGenerating.value = false

  if (failCount === 0) {
    ElMessage.success(`成功为 ${successCount} 名选手生成策略`)
  } else {
    ElMessage.warning(`生成完成: 成功 ${successCount} 人, 失败 ${failCount} 人`)
  }
}

// 初始化加载
onMounted(() => {
  loadPlayerList()
})
</script>

<style scoped>
.player-market-view {
  padding: 0;
}

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

.filter-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.player-table-card {
  border-radius: 12px;
}

.player-table-card :deep(.clickable-row) {
  cursor: pointer;
}

.player-table-card :deep(.clickable-row:hover) {
  background-color: #ecf5ff !important;
}

.player-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.player-avatar {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 10px;
  flex-shrink: 0;
}

.player-avatar.top { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-avatar.jug { background: linear-gradient(135deg, #22c55e, #16a34a); }
.player-avatar.mid { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-avatar.adc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.player-avatar.sup { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.player-name-text {
  font-weight: 600;
  color: #303133;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.no-team {
  color: #909399;
  font-style: italic;
}

.value-text {
  font-weight: 600;
  color: #409eff;
}

.contract-expired {
  color: #f56c6c;
  font-weight: 600;
}

.contract-expiring {
  color: #e6a23c;
}

.expire-warning {
  color: #e6a23c;
  margin-left: 4px;
}

.stat-high { color: #67c23a; font-weight: 600; }
.stat-mid { color: #e6a23c; }
.stat-low { color: #f56c6c; font-weight: 600; }

.loyalty-high { color: #409eff; font-weight: 600; }
.loyalty-mid { color: #909399; }
.loyalty-low { color: #f56c6c; font-weight: 600; }

.pagination-container {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.reason-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.more-teams {
  font-size: 12px;
  color: #909399;
  margin-left: 4px;
}

.no-data {
  color: #c0c4cc;
  font-style: italic;
}
</style>
