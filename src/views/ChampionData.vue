<template>
  <div class="champion-data">
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><Aim /></el-icon>
          英雄数据
        </h1>
        <p class="page-description">
          查看英雄列表、使用率、胜率和体系统计
        </p>
      </div>
    </div>

    <el-tabs v-model="activeTab" type="border-card">
      <el-tab-pane label="英雄列表" name="list">
        <div class="filter-bar">
          <el-select v-model="positionFilter" placeholder="位置" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="上单" value="Top" />
            <el-option label="打野" value="Jug" />
            <el-option label="中路" value="Mid" />
            <el-option label="ADC" value="Adc" />
            <el-option label="辅助" value="Sup" />
          </el-select>
          <el-select v-model="archetypeFilter" placeholder="定位" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="激进" value="aggressive" />
            <el-option label="后期" value="scaling" />
            <el-option label="功能" value="utility" />
            <el-option label="分推" value="splitpush" />
            <el-option label="团战" value="teamfight" />
          </el-select>
        </div>

        <el-table :data="filteredChampions" stripe style="width: 100%">
          <el-table-column prop="id" label="ID" width="60" align="center" />
          <el-table-column prop="name_cn" label="中文名" width="120" />
          <el-table-column prop="name_en" label="英文名" width="140" />
          <el-table-column prop="position" label="位置" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="positionTagType(row.position)" size="small">
                {{ positionName(row.position) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="archetype_name" label="定位" width="100" align="center">
            <template #default="{ row }">
              <el-tag type="info" size="small" effect="plain">{{ row.archetype_name }}</el-tag>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="使用统计" name="stats">
        <el-table
          v-if="championStats.length > 0"
          :data="championStats"
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'pick_count', order: 'descending' }"
        >
          <el-table-column prop="name_cn" label="英雄" width="120" />
          <el-table-column prop="position" label="位置" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="positionTagType(row.position)" size="small">
                {{ positionName(row.position) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="pick_count" label="选用" width="90" align="center" sortable />
          <el-table-column prop="win_count" label="胜场" width="90" align="center" sortable />
          <el-table-column label="胜率" width="100" align="center" sortable :sort-method="sortByWinRate">
            <template #default="{ row }">
              <span :class="winRateClass(row)">
                {{ row.pick_count > 0 ? ((row.win_count / row.pick_count) * 100).toFixed(1) + '%' : '-' }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="ban_count" label="Ban" width="90" align="center" sortable />
        </el-table>
        <el-empty v-else description="暂无比赛数据" />
      </el-tab-pane>

      <el-tab-pane label="体系统计" name="comp">
        <el-table
          v-if="compStats.length > 0"
          :data="compStats"
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'pick_count', order: 'descending' }"
        >
          <el-table-column label="体系" width="160">
            <template #default="{ row }">
              {{ compName(row.comp_type) }}
            </template>
          </el-table-column>
          <el-table-column prop="pick_count" label="使用次数" width="120" align="center" sortable />
          <el-table-column prop="win_count" label="胜场" width="100" align="center" sortable />
          <el-table-column label="胜率" width="100" align="center" sortable :sort-method="sortCompByWinRate">
            <template #default="{ row }">
              <span :class="compWinRateClass(row)">
                {{ row.pick_count > 0 ? ((row.win_count / row.pick_count) * 100).toFixed(1) + '%' : '-' }}
              </span>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-else description="暂无比赛数据" />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Aim } from '@element-plus/icons-vue'
import { getChampionList, getChampionStats, getCompStats } from '@/api/tauri'
import type { ChampionInfo, ChampionStatInfo, CompStatInfo } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { ElMessage } from 'element-plus'

const gameStore = useGameStore()
const activeTab = ref('list')
const positionFilter = ref('')
const archetypeFilter = ref('')

const champions = ref<ChampionInfo[]>([])
const championStats = ref<ChampionStatInfo[]>([])
const compStats = ref<CompStatInfo[]>([])

const filteredChampions = computed(() => {
  return champions.value.filter(c => {
    if (positionFilter.value && c.position !== positionFilter.value) return false
    if (archetypeFilter.value && c.archetype !== archetypeFilter.value) return false
    return true
  })
})

const COMP_NAME_MAP: Record<string, string> = {
  Rush: '速推', PickOff: '抓单', AllIn: '莽夫', MidJungle: '中野联动', TopJungle: '上野联动',
  Protect: '保C', Fortress: '铁桶阵', UtilityComp: '功能流', Stall: '龟缩', BotLane: '下路统治',
  Teamfight: '团战', Dive: '开团', Skirmish: '小规模团战', DualCarry: '双C', Flex: '全能',
  Splitpush: '分推', SideLane: '4-1分带', Control: '运营', TripleThreat: '三线施压', LateGame: '后期发育',
}

const POSITION_NAME_MAP: Record<string, string> = {
  Top: '上单', Jug: '打野', Mid: '中路', Adc: 'ADC', Sup: '辅助',
}

const compName = (type: string) => COMP_NAME_MAP[type] || type
const positionName = (pos: string) => POSITION_NAME_MAP[pos] || pos

const positionTagType = (pos: string): '' | 'success' | 'warning' | 'info' | 'danger' => {
  const map: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = {
    Top: 'danger', Jug: 'warning', Mid: '', Adc: 'success', Sup: 'info',
  }
  return map[pos] || 'info'
}

const winRateClass = (row: ChampionStatInfo) => {
  if (row.pick_count === 0) return ''
  const rate = row.win_count / row.pick_count
  if (rate > 0.55) return 'rate-high'
  if (rate < 0.45) return 'rate-low'
  return ''
}

const compWinRateClass = (row: CompStatInfo) => {
  if (row.pick_count === 0) return ''
  const rate = row.win_count / row.pick_count
  if (rate > 0.55) return 'rate-high'
  if (rate < 0.45) return 'rate-low'
  return ''
}

const sortByWinRate = (a: ChampionStatInfo, b: ChampionStatInfo) => {
  const rateA = a.pick_count > 0 ? a.win_count / a.pick_count : 0
  const rateB = b.pick_count > 0 ? b.win_count / b.pick_count : 0
  return rateA - rateB
}

const sortCompByWinRate = (a: CompStatInfo, b: CompStatInfo) => {
  const rateA = a.pick_count > 0 ? a.win_count / a.pick_count : 0
  const rateB = b.pick_count > 0 ? b.win_count / b.pick_count : 0
  return rateA - rateB
}

const fetchData = async () => {
  try {
    const list = await getChampionList()
    champions.value = list

    const saveId = gameStore.currentSave?.id
    if (saveId) {
      const [stats, comps] = await Promise.all([
        getChampionStats(saveId),
        getCompStats(saveId),
      ])
      championStats.value = stats
      compStats.value = comps
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(`加载英雄数据失败: ${msg}`)
  }
}

onMounted(() => {
  fetchData()
})
</script>

<style scoped lang="scss">
.champion-data {
  padding: 24px;
  min-height: 100%;

  .page-header {
    margin-bottom: 24px;

    .header-content {
      .page-title {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 28px;
        font-weight: 700;
        margin: 0;
        color: #1f2937;

        .el-icon {
          color: #409eff;
        }
      }

      .page-description {
        margin: 8px 0 0 0;
        color: #6b7280;
        font-size: 14px;
      }
    }
  }

  .filter-bar {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
  }
}

.rate-high {
  color: #67c23a;
  font-weight: 600;
}

.rate-low {
  color: #f56c6c;
  font-weight: 600;
}
</style>
