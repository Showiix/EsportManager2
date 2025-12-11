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
                {{ row.team.substring(0, 2) }}
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

        <el-table-column label="操作" width="120" fixed="right" align="center">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="viewPlayer(row)">
              查看详情
            </el-button>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  User,
  Check,
  Clock,
  TrendCharts,
  Search,
} from '@element-plus/icons-vue'

const router = useRouter()

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

// 统计数据
const stats = ref({
  total: 280,
  active: 250,
  retired: 30,
  avgAbility: 75,
})

// 模拟数据
const players = ref([
  { id: 1, gameId: 'Faker', name: '李相赫', team: 'T1', region: 'LCK', position: 'MID', age: 28, ability: 95, potential: 96, tag: 'GENIUS' },
  { id: 2, gameId: 'Chovy', name: '郑志勋', team: 'Gen.G', region: 'LCK', position: 'MID', age: 24, ability: 93, potential: 95, tag: 'GENIUS' },
  { id: 3, gameId: 'Zeus', name: '崔宇杰', team: 'T1', region: 'LCK', position: 'TOP', age: 21, ability: 88, potential: 94, tag: 'GENIUS' },
  { id: 4, gameId: 'Ruler', name: '朴载赫', team: 'JDG', region: 'LPL', position: 'ADC', age: 26, ability: 90, potential: 91, tag: 'NORMAL' },
  { id: 5, gameId: 'Keria', name: '柳民锡', team: 'T1', region: 'LCK', position: 'SUP', age: 22, ability: 89, potential: 93, tag: 'GENIUS' },
  { id: 6, gameId: 'Canyon', name: '金建部', team: 'Gen.G', region: 'LCK', position: 'JUG', age: 23, ability: 91, potential: 92, tag: 'GENIUS' },
  { id: 7, gameId: 'Knight', name: '卓定', team: 'BLG', region: 'LPL', position: 'MID', age: 24, ability: 89, potential: 92, tag: 'GENIUS' },
  { id: 8, gameId: 'Caps', name: 'Rasmus', team: 'G2', region: 'LEC', position: 'MID', age: 25, ability: 85, potential: 88, tag: 'NORMAL' },
  { id: 9, gameId: '369', name: '白家浩', team: 'JDG', region: 'LPL', position: 'TOP', age: 23, ability: 87, potential: 90, tag: 'GENIUS' },
  { id: 10, gameId: 'Meiko', name: '田野', team: 'EDG', region: 'LPL', position: 'SUP', age: 26, ability: 85, potential: 86, tag: 'NORMAL' },
  { id: 11, gameId: 'Gumayusi', name: '李民赫', team: 'T1', region: 'LCK', position: 'ADC', age: 22, ability: 88, potential: 93, tag: 'GENIUS' },
  { id: 12, gameId: 'Oner', name: '文炫俊', team: 'T1', region: 'LCK', position: 'JUG', age: 22, ability: 86, potential: 91, tag: 'GENIUS' },
  { id: 13, gameId: 'Elk', name: '谢家辉', team: 'BLG', region: 'LPL', position: 'ADC', age: 22, ability: 86, potential: 90, tag: 'GENIUS' },
  { id: 14, gameId: 'Kanavi', name: '徐进赫', team: 'JDG', region: 'LPL', position: 'JUG', age: 24, ability: 88, potential: 89, tag: 'NORMAL' },
  { id: 15, gameId: 'Viper', name: '朴道玹', team: 'HLE', region: 'LCK', position: 'ADC', age: 24, ability: 89, potential: 90, tag: 'NORMAL' },
  { id: 16, gameId: 'Peyz', name: '金书垣', team: 'Gen.G', region: 'LCK', position: 'ADC', age: 19, ability: 82, potential: 92, tag: 'GENIUS' },
  { id: 17, gameId: 'Doran', name: '崔铉晙', team: 'Gen.G', region: 'LCK', position: 'TOP', age: 24, ability: 84, potential: 86, tag: 'NORMAL' },
  { id: 18, gameId: 'Lehends', name: '孙时佑', team: 'Gen.G', region: 'LCK', position: 'SUP', age: 26, ability: 85, potential: 86, tag: 'NORMAL' },
  { id: 19, gameId: 'Bin', name: '陈泽彬', team: 'BLG', region: 'LPL', position: 'TOP', age: 22, ability: 87, potential: 91, tag: 'GENIUS' },
  { id: 20, gameId: 'ON', name: '刘世宇', team: 'BLG', region: 'LPL', position: 'SUP', age: 21, ability: 82, potential: 88, tag: 'NORMAL' },
])

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
</style>
