<template>
  <div class="players-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>选手中心</h1>
        <p>查看所有职业选手信息和职业生涯数据</p>
      </div>
    </div>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><User /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.total }}</div>
              <div class="stat-label">总选手数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="28"><Check /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.active }}</div>
              <div class="stat-label">在役选手</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon silver">
              <el-icon :size="28"><Clock /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.retired }}</div>
              <div class="stat-label">退役选手</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="28"><TrendCharts /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ stats.avgAbility }}</div>
              <div class="stat-label">平均能力值</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 筛选和搜索 -->
    <el-card class="filter-card">
      <el-row :gutter="16" align="middle">
        <el-col :span="6">
          <el-input
            v-model="searchQuery"
            placeholder="搜索选手名称或游戏ID"
            clearable
            @clear="handleSearch"
            @keyup.enter="handleSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </el-col>
        <el-col :span="4">
          <el-select v-model="selectedRegion" placeholder="选择赛区" clearable @change="applyFilters">
            <el-option label="全部赛区" value="" />
            <el-option label="LPL" value="LPL" />
            <el-option label="LCK" value="LCK" />
            <el-option label="LEC" value="LEC" />
            <el-option label="LCS" value="LCS" />
          </el-select>
        </el-col>
        <el-col :span="4">
          <el-select v-model="selectedPosition" placeholder="选择位置" clearable @change="applyFilters">
            <el-option label="全部位置" value="" />
            <el-option label="上单 TOP" value="TOP" />
            <el-option label="打野 JUG" value="JUG" />
            <el-option label="中单 MID" value="MID" />
            <el-option label="下路 ADC" value="ADC" />
            <el-option label="辅助 SUP" value="SUP" />
          </el-select>
        </el-col>
        <el-col :span="4">
          <el-select v-model="selectedTalent" placeholder="选择天赋" clearable @change="applyFilters">
            <el-option label="全部天赋" value="" />
            <el-option label="天才" value="GENIUS" />
            <el-option label="普通" value="NORMAL" />
            <el-option label="平庸" value="ORDINARY" />
          </el-select>
        </el-col>
        <el-col :span="6">
          <div class="filter-actions">
            <el-button type="primary" @click="handleSearch">
              <el-icon><Search /></el-icon>
              搜索
            </el-button>
            <el-button @click="clearFilters">清空筛选</el-button>
          </div>
        </el-col>
      </el-row>
    </el-card>

    <!-- 选手列表 -->
    <el-card class="list-card">
      <template #header>
        <div class="card-header">
          <h2>选手列表</h2>
          <span class="count-badge">共 {{ filteredPlayers.length }} 名选手</span>
        </div>
      </template>

      <el-table :data="paginatedPlayers" stripe class="players-table" @sort-change="handleSortChange">
        <el-table-column prop="gameId" label="游戏ID" width="150" sortable>
          <template #default="{ row }">
            <router-link :to="`/players/${row.id}`" class="player-link">
              {{ row.gameId }}
            </router-link>
          </template>
        </el-table-column>

        <el-table-column prop="name" label="真实姓名" width="120" />

        <el-table-column prop="position" label="位置" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getPositionType(row.position)" size="default">
              {{ row.position }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="team" label="所属战队" width="150">
          <template #default="{ row }">
            <div class="team-cell">
              <div class="team-avatar mini" :class="row.region.toLowerCase()">
                {{ row.teamShort }}
              </div>
              <span>{{ row.team }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="region" label="赛区" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getRegionType(row.region)" size="small">
              {{ row.region }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="age" label="年龄" width="80" sortable align="center">
          <template #default="{ row }">
            {{ row.age }}岁
          </template>
        </el-table-column>

        <el-table-column prop="ability" label="能力值" width="120" sortable align="center">
          <template #default="{ row }">
            <span class="ability-number" :style="{ color: getAbilityColor(row.ability) }">
              {{ row.ability }}
            </span>
          </template>
        </el-table-column>

        <el-table-column prop="potential" label="潜力值" width="100" sortable align="center">
          <template #default="{ row }">
            <span class="potential-value">{{ row.potential }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="tag" label="天赋" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="getTalentType(row.tag)" size="small">
              {{ getTalentLabel(row.tag) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="loyalty" label="忠诚度" width="100" sortable align="center">
          <template #default="{ row }">
            <span class="loyalty-value" :style="{ color: getLoyaltyColor(row.loyalty) }">
              {{ row.loyalty }}
            </span>
          </template>
        </el-table-column>

        <el-table-column prop="satisfaction" label="满意度" width="100" sortable align="center">
          <template #default="{ row }">
            <span class="satisfaction-value" :style="{ color: getSatisfactionColor(row.satisfaction) }">
              {{ row.satisfaction }}
            </span>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="180" fixed="right" align="center">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button class="action-btn view-btn" size="small" @click="viewPlayer(row)">
                <el-icon><View /></el-icon>
                详情
              </el-button>
              <el-button class="action-btn edit-btn" size="small" @click="editPlayer(row)">
                <el-icon><Edit /></el-icon>
                编辑
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="filteredPlayers.length"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 编辑弹窗 -->
    <PlayerEditDialog
      v-model="showEditDialog"
      :player="editingPlayer"
      @close="showEditDialog = false"
      @saved="handlePlayerSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  User,
  Check,
  Clock,
  TrendCharts,
  Search,
  View,
  Edit,
} from '@element-plus/icons-vue'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'
import { teamApi, type Player } from '@/api/tauri'
import PlayerEditDialog from '@/components/player/PlayerEditDialog.vue'
import { createLogger } from '@/utils/logger'

const logger = createLogger('Players')

const router = useRouter()
const teamStore = useTeamStoreTauri()
const { regions, teams } = storeToRefs(teamStore)

// 原始选手数据
const rawPlayers = ref<Player[]>([])

// 筛选状态
const searchQuery = ref('')
const selectedRegion = ref('')
const selectedPosition = ref('')
const selectedTalent = ref('')

// 分页配置
const pagination = ref({
  page: 1,
  pageSize: 20,
})

// 排序配置
const sortConfig = ref({
  prop: '',
  order: '' as 'ascending' | 'descending' | '',
})

// 编辑弹窗状态
const showEditDialog = ref(false)
const editingPlayer = ref<any>(null)

// 初始化加载
onMounted(async () => {
  logger.debug('Players.vue onMounted started')
  try {
    // 先加载赛区信息（用于获取赛区代码）
    await teamStore.loadRegions()
    logger.debug('Regions loaded')

    // 获取所有队伍（用于获取队伍名称）
    const allTeams = await teamApi.getAllTeams()
    logger.debug(`Got ${allTeams.length} teams`)

    // 创建队伍映射
    const teamMap = new Map<number, { name: string; short_name: string; region_id: number }>()
    for (const team of allTeams) {
      teamMap.set(team.id, {
        name: team.name,
        short_name: team.short_name || team.name.slice(0, 3),
        region_id: team.region_id
      })
    }

    // 一次性获取所有选手
    const allPlayers = await teamApi.getAllPlayers()
    logger.debug(`Got ${allPlayers.length} players from API`)

    // 添加队伍信息
    rawPlayers.value = allPlayers.map(p => {
      const teamInfo = p.team_id ? teamMap.get(p.team_id) : null
      return {
        ...p,
        team_name: teamInfo?.name ?? '自由球员',
        team_short: teamInfo?.short_name ?? 'FA',
        region_code: teamInfo ? getRegionCode(teamInfo.region_id) : 'FA',
      }
    }) as any

    logger.debug(`Loaded ${rawPlayers.value.length} players total`)
  } catch (e) {
    logger.error('Failed to load players:', e)
  }
})

// 获取赛区代码
const getRegionCode = (regionId: number) => {
  const region = regions.value.find(r => r.id === regionId)
  return region?.code ?? 'LPL'
}

// 获取战队名称
const getTeamName = (teamId: number) => {
  const team = teams.value.find(t => t.id === teamId)
  return team?.name ?? '未知'
}

// 计算天赋标签
const getTalentTag = (ability: number, potential: number) => {
  if (potential >= 90 || ability >= 85) return 'GENIUS'
  if (potential >= 75 || ability >= 70) return 'NORMAL'
  return 'ORDINARY'
}

// 位置简称转换
const getPositionShort = (position: string) => {
  const shorts: Record<string, string> = {
    Top: 'TOP', Jungle: 'JUG', Mid: 'MID', Adc: 'ADC', Support: 'SUP'
  }
  return shorts[position] || position
}

// 统计数据
const stats = computed(() => {
  const total = rawPlayers.value.length || 280
  const avgAbility = rawPlayers.value.length > 0
    ? Math.round(rawPlayers.value.reduce((sum, p) => sum + p.ability, 0) / rawPlayers.value.length)
    : 75
  return {
    total,
    active: total,
    retired: 0,
    avgAbility,
  }
})

// 映射为显示格式
const players = computed(() => {
  return rawPlayers.value.map(p => ({
    id: p.id,
    gameId: p.game_id,
    name: p.real_name || p.game_id,
    team: (p as any).team_name || (p.team_id ? getTeamName(p.team_id) : '未知'),
    teamShort: (p as any).team_short || ((p as any).team_name || '').slice(0, 3),
    region: (p as any).region_code || 'LPL',
    position: getPositionShort(p.position || ''),
    age: p.age,
    ability: p.ability,
    potential: p.potential,
    tag: getTalentTag(p.ability, p.potential),
    loyalty: p.loyalty ?? 50,
    satisfaction: p.satisfaction ?? 50,
  }))
})

// 计算属性
const filteredPlayers = computed(() => {
  let result = players.value

  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(p =>
      p.gameId.toLowerCase().includes(query) ||
      p.name.toLowerCase().includes(query)
    )
  }

  // 赛区过滤
  if (selectedRegion.value) {
    result = result.filter(p => p.region === selectedRegion.value)
  }

  // 位置过滤
  if (selectedPosition.value) {
    result = result.filter(p => p.position === selectedPosition.value)
  }

  // 天赋过滤
  if (selectedTalent.value) {
    result = result.filter(p => p.tag === selectedTalent.value)
  }

  // 排序
  if (sortConfig.value.prop && sortConfig.value.order) {
    const prop = sortConfig.value.prop as keyof typeof result[0]
    const multiplier = sortConfig.value.order === 'ascending' ? 1 : -1
    result = [...result].sort((a, b) => {
      const aVal = a[prop]
      const bVal = b[prop]
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return (aVal - bVal) * multiplier
      }
      return String(aVal).localeCompare(String(bVal)) * multiplier
    })
  }

  return result
})

const paginatedPlayers = computed(() => {
  const start = (pagination.value.page - 1) * pagination.value.pageSize
  const end = start + pagination.value.pageSize
  return filteredPlayers.value.slice(start, end)
})

// 方法
const handleSearch = () => {
  pagination.value.page = 1
}

const applyFilters = () => {
  pagination.value.page = 1
}

const clearFilters = () => {
  searchQuery.value = ''
  selectedRegion.value = ''
  selectedPosition.value = ''
  selectedTalent.value = ''
  pagination.value.page = 1
}

const handleSortChange = ({ prop, order }: { prop: string; order: 'ascending' | 'descending' | null }) => {
  sortConfig.value.prop = prop
  sortConfig.value.order = order || ''
}

const handleSizeChange = (size: number) => {
  pagination.value.pageSize = size
  pagination.value.page = 1
}

const handlePageChange = (page: number) => {
  pagination.value.page = page
}

const viewPlayer = (player: any) => {
  router.push(`/players/${player.id}`)
}

// 编辑选手
const editPlayer = (player: any) => {
  editingPlayer.value = player
  showEditDialog.value = true
}

// 选手属性保存后更新列表
const handlePlayerSaved = (updatedPlayer: any) => {
  // 找到并更新 rawPlayers 中的选手数据
  const index = rawPlayers.value.findIndex(p => p.id === updatedPlayer.id)
  if (index !== -1) {
    rawPlayers.value[index] = {
      ...rawPlayers.value[index],
      ability: updatedPlayer.ability,
      potential: updatedPlayer.potential,
      stability: updatedPlayer.stability,
      age: updatedPlayer.age,
    }
  }
}

// 辅助函数
const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getPositionType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'success',
    MID: 'primary',
    ADC: 'warning',
    SUP: 'info',
  }
  return types[position] || 'info'
}

const getTalentType = (tag: string) => {
  const types: Record<string, string> = {
    GENIUS: 'warning',
    NORMAL: 'primary',
    ORDINARY: 'info',
  }
  return types[tag] || 'info'
}

const getTalentLabel = (tag: string) => {
  const labels: Record<string, string> = {
    GENIUS: '天才',
    NORMAL: '普通',
    ORDINARY: '平庸',
  }
  return labels[tag] || tag
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getLoyaltyColor = (loyalty: number) => {
  if (loyalty >= 70) return '#22c55e'  // 高忠诚度 - 绿色
  if (loyalty >= 50) return '#3b82f6'  // 中等 - 蓝色
  if (loyalty >= 35) return '#f59e0b'  // 较低 - 橙色
  return '#ef4444'  // 低忠诚度 - 红色
}

const getSatisfactionColor = (satisfaction: number) => {
  if (satisfaction >= 65) return '#22c55e'  // 高满意度 - 绿色
  if (satisfaction >= 50) return '#3b82f6'  // 中等 - 蓝色
  if (satisfaction >= 40) return '#f59e0b'  // 较低 - 橙色
  return '#ef4444'  // 低满意度 - 红色
}
</script>

<style scoped>
.players-view {
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
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: var(--text-tertiary);
  margin: 0;
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

.stat-icon.blue {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.stat-icon.green {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.stat-icon.silver {
  background: linear-gradient(135deg, #9ca3af, #6b7280);
}

.stat-icon.purple {
  background: linear-gradient(135deg, #8b5cf6, #7c3aed);
}

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* 筛选卡片 */
.filter-card {
  margin-bottom: 20px;
  border-radius: 12px;
}

.filter-card :deep(.el-card__body) {
  padding: 16px 20px;
}

.filter-card :deep(.el-select),
.filter-card :deep(.el-input) {
  width: 100%;
}

.filter-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

/* 列表卡片 */
.list-card {
  border-radius: 12px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.count-badge {
  font-size: 14px;
  color: var(--text-tertiary);
}

/* 选手链接 */
.player-link {
  color: var(--primary-color);
  font-weight: 600;
  text-decoration: none;
}

.player-link:hover {
  color: var(--primary-dark);
  text-decoration: underline;
}

/* 队伍单元格 */
.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-avatar.mini {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 10px;
}

.team-avatar.mini.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.team-avatar.mini.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.team-avatar.mini.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.team-avatar.mini.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

/* 能力值数字 */
.ability-number {
  font-size: 18px;
  font-weight: 700;
}

/* 潜力值 */
.potential-value {
  color: #8b5cf6;
  font-weight: 600;
}

/* 忠诚度 */
.loyalty-value {
  font-weight: 600;
}

/* 满意度 */
.satisfaction-value {
  font-weight: 600;
}

/* 分页 */
.pagination-wrapper {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border-light);
}

/* 表格样式 */
.players-table {
  border-radius: 8px;
}

/* 操作按钮 */
.action-buttons {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.action-btn {
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.action-btn :deep(.el-icon) {
  margin-right: 4px;
}

.view-btn {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  border: none;
  color: white;
}

.view-btn:hover {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.4);
}

.edit-btn {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  border: none;
  color: white;
}

.edit-btn:hover {
  background: linear-gradient(135deg, #d97706, #b45309);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(245, 158, 11, 0.4);
}
</style>
