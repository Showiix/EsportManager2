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
      <div class="header-actions">
        <SeasonSelector v-model="selectedSeason" :show-all="true" />
      </div>
    </div>


    <el-tabs v-model="activeTab" type="border-card" class="data-tabs">
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

        <el-table :data="filteredChampions" class="data-table" style="width: 100%">
          <el-table-column prop="id" label="ID" width="80" align="center" />
          <el-table-column prop="name_cn" label="中文名" min-width="120">
            <template #default="{ row }">
              <span class="champion-name">{{ row.name_cn }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="name_en" label="英文名" min-width="140" />
          <el-table-column prop="position" label="位置" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="positionTagType(row.position)" size="small">
                {{ positionName(row.position) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="archetype_name" label="定位" width="120" align="center">
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
          class="data-table"
          style="width: 100%"
          :default-sort="{ prop: 'pick_count', order: 'descending' }"
        >
          <el-table-column prop="name_cn" label="英雄" min-width="140">
            <template #default="{ row }">
              <span class="champion-name">{{ row.name_cn }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="position" label="位置" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="positionTagType(row.position)" size="small">
                {{ positionName(row.position) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="pick_count" label="选用" width="100" align="center" sortable />
          <el-table-column prop="win_count" label="胜场" width="100" align="center" sortable />
          <el-table-column label="胜率" width="120" align="center" sortable :sort-method="sortByWinRate">
            <template #default="{ row }">
              <span :class="winRateClass(row)">
                {{ row.pick_count > 0 ? ((row.win_count / row.pick_count) * 100).toFixed(1) + '%' : '-' }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="ban_count" label="Ban" width="100" align="center" sortable />
        </el-table>
        <el-empty v-else description="暂无比赛数据" />
      </el-tab-pane>

      <el-tab-pane label="体系统计" name="comp">
        <div class="filter-bar">
          <el-radio-group v-model="compViewMode" size="small">
            <el-radio-button value="overall">总体</el-radio-button>
            <el-radio-button value="team">按战队</el-radio-button>
          </el-radio-group>
          <el-select v-if="compViewMode === 'team'" v-model="compTeamFilter" placeholder="选择战队" clearable style="width: 180px">
            <el-option v-for="t in teamCompTeams" :key="t" :label="t" :value="t" />
          </el-select>
        </div>
        <el-table
          v-if="compViewMode === 'overall' && compStats.length > 0"
          :data="compStats"
          class="data-table"
          style="width: 100%"
          :default-sort="{ prop: 'pick_count', order: 'descending' }"
        >
          <el-table-column label="体系" min-width="160">
            <template #default="{ row }">
              <span class="comp-name-text">{{ compName(row.comp_type) }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="pick_count" label="使用次数" width="120" align="center" sortable />
          <el-table-column prop="win_count" label="胜场" width="100" align="center" sortable />
          <el-table-column label="胜率" width="120" align="center" sortable :sort-method="sortCompByWinRate">
            <template #default="{ row }">
              <span :class="compWinRateClass(row)">
                {{ row.pick_count > 0 ? ((row.win_count / row.pick_count) * 100).toFixed(1) + '%' : '-' }}
              </span>
            </template>
          </el-table-column>
        </el-table>
        <el-table
          v-if="compViewMode === 'team' && filteredTeamCompUsage.length > 0"
          :data="filteredTeamCompUsage"
          class="data-table"
          style="width: 100%"
          :default-sort="{ prop: 'games', order: 'descending' }"
        >
          <el-table-column prop="team_name" label="战队" min-width="140" />
          <el-table-column label="体系" min-width="140">
            <template #default="{ row }">
              <span class="comp-name-text">{{ row.comp_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="games" label="使用次数" width="120" align="center" sortable />
          <el-table-column prop="wins" label="胜场" width="100" align="center" sortable />
          <el-table-column label="胜率" width="120" align="center" sortable :sort-method="sortTeamCompWinRate">
            <template #default="{ row }">
              <span :class="teamCompWinRateClass(row)">
                {{ row.games > 0 ? ((row.wins / row.games) * 100).toFixed(1) + '%' : '-' }}
              </span>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="(compViewMode === 'overall' && compStats.length === 0) || (compViewMode === 'team' && filteredTeamCompUsage.length === 0)" description="暂无比赛数据" />
      </el-tab-pane>

      <el-tab-pane label="体系图鉴" name="matchup">

        <div class="matchup-grid">
          <div v-for="comp in compMatchups" :key="comp.id" class="comp-card-wrap">
            <div class="comp-card-top">
              <div class="comp-card-name">{{ comp.name }}</div>
              <div class="comp-card-meta">
                <span class="comp-diff" :class="'diff-' + comp.difficulty">{{ comp.difficulty }}</span>
                <el-tag v-for="arch in comp.archetypes" :key="arch" size="small" effect="plain" class="arch-tag">
                  {{ arch }}
                </el-tag>
              </div>
            </div>
            <div class="comp-card-condition">
              <span class="cond-label">触发</span>
              <span class="cond-text">{{ COMP_CONDITIONS[comp.id] }}</span>
            </div>
            <div class="comp-card-rels">
              <div v-if="comp.hardCounters.length" class="rel-row">
                <span class="rel-icon rel-strong">&#9650;</span>
                <div class="rel-tags">
                  <span v-for="t in comp.hardCounters" :key="t" class="rel-tag rel-tag-strong">{{ COMP_NAME_MAP[t] || t }}</span>
                </div>
              </div>
              <div v-if="comp.hardCounteredBy.length" class="rel-row">
                <span class="rel-icon rel-weak">&#9660;</span>
                <div class="rel-tags">
                  <span v-for="t in comp.hardCounteredBy" :key="t" class="rel-tag rel-tag-weak">{{ COMP_NAME_MAP[t] || t }}</span>
                </div>
              </div>
              <div v-if="comp.softCounters.length" class="rel-row">
                <span class="rel-icon rel-soft-strong">&#9651;</span>
                <div class="rel-tags">
                  <span v-for="t in comp.softCounters" :key="t" class="rel-tag rel-tag-soft-strong">{{ COMP_NAME_MAP[t] || t }}</span>
                </div>
              </div>
              <div v-if="comp.softCounteredBy.length" class="rel-row">
                <span class="rel-icon rel-soft-weak">&#9661;</span>
                <div class="rel-tags">
                  <span v-for="t in comp.softCounteredBy" :key="t" class="rel-tag rel-tag-soft-weak">{{ COMP_NAME_MAP[t] || t }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <el-tab-pane label="选手英雄" name="player">
        <div class="filter-bar">
          <el-input v-model="playerSearch" placeholder="搜索选手" clearable style="width: 200px" />
          <el-select v-model="playerPosFilter" placeholder="位置" clearable style="width: 120px">
            <el-option label="全部" value="" />
            <el-option label="上单" value="TOP" />
            <el-option label="打野" value="JUG" />
            <el-option label="中路" value="MID" />
            <el-option label="ADC" value="ADC" />
            <el-option label="辅助" value="SUP" />
          </el-select>
        </div>
        <el-table
          v-if="filteredPlayerUsage.length > 0"
          :data="pagedPlayerUsage"
          class="data-table"
          style="width: 100%"
          :default-sort="{ prop: 'games', order: 'descending' }"
        >
          <el-table-column prop="player_name" label="选手" min-width="130">
            <template #default="{ row }">
              <span class="player-name">{{ row.player_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="team_name" label="战队" width="120" />
          <el-table-column prop="position" label="位置" width="100" align="center">
            <template #default="{ row }">
              <el-tag :type="positionTagType(row.position)" size="small">
                {{ positionName(row.position.charAt(0).toUpperCase() + row.position.slice(1).toLowerCase()) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="champion_name" label="英雄" min-width="120">
            <template #default="{ row }">
              <span class="champion-name">{{ row.champion_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="games" label="使用次数" width="100" align="center" sortable />
          <el-table-column prop="wins" label="胜场" width="80" align="center" sortable />
          <el-table-column label="胜率" width="100" align="center" sortable :sort-method="sortPlayerUsageWinRate">
            <template #default="{ row }">
              <span :class="playerUsageWinRateClass(row)">
                {{ row.games > 0 ? ((row.wins / row.games) * 100).toFixed(1) + '%' : '-' }}
              </span>
            </template>
          </el-table-column>
        </el-table>
        <div v-if="filteredPlayerUsage.length > 0" class="pagination-wrapper">
          <el-pagination
            v-model:current-page="playerPage"
            v-model:page-size="playerPageSize"
            :total="filteredPlayerUsage.length"
            layout="total, sizes, prev, pager, next"
            :page-sizes="[20, 50, 100]"
            background
          />
        </div>
        <el-empty v-if="filteredPlayerUsage.length === 0" description="暂无选手英雄使用数据" />
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Aim } from '@element-plus/icons-vue'
import { getChampionList, getChampionStats, getCompStats, getPlayerChampionUsage, getTeamCompUsage } from '@/api/tauri'
import type { ChampionInfo, ChampionStatInfo, CompStatInfo, PlayerChampionUsageItem, TeamCompUsageItem } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { ElMessage } from 'element-plus'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const gameStore = useGameStore()
const timeStore = useTimeStore()
const activeTab = ref('list')
const selectedSeason = ref(timeStore.currentSeasonFromTime || 1)
const positionFilter = ref('')
const archetypeFilter = ref('')
const playerSearch = ref('')
const playerPosFilter = ref('')
const playerPage = ref(1)
const playerPageSize = ref(20)
const compViewMode = ref<'overall' | 'team'>('overall')
const compTeamFilter = ref('')

const champions = ref<ChampionInfo[]>([])
const championStats = ref<ChampionStatInfo[]>([])
const compStats = ref<CompStatInfo[]>([])
const playerUsage = ref<PlayerChampionUsageItem[]>([])
const teamCompUsage = ref<TeamCompUsageItem[]>([])

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

const COMP_CORE_ARCHETYPES: Record<string, string[]> = {
  Rush: ['Aggressive'], PickOff: ['Aggressive'], AllIn: ['Aggressive'],
  MidJungle: ['Aggressive'], TopJungle: ['Aggressive'],
  Protect: ['Scaling'], Stall: ['Scaling'], LateGame: ['Scaling'], DualCarry: ['Scaling'],
  Fortress: ['Utility'], UtilityComp: ['Utility'], Control: ['Utility'],
  Splitpush: ['Splitpush'], SideLane: ['Splitpush'], TripleThreat: ['Splitpush'],
  Teamfight: ['Teamfight'], Dive: ['Teamfight'], Skirmish: ['Teamfight'],
  BotLane: ['Scaling', 'Utility'],
  Flex: ['Aggressive', 'Scaling', 'Utility', 'Splitpush', 'Teamfight'],
}

const COMP_DIFFICULTY: Record<string, string> = {
  Rush: '中', PickOff: '高', AllIn: '低', MidJungle: '中', TopJungle: '中',
  Protect: '低', Fortress: '低', UtilityComp: '中', Stall: '低', BotLane: '中',
  Teamfight: '低', Dive: '高', Skirmish: '中', DualCarry: '高', Flex: '高',
  Splitpush: '高', SideLane: '高', Control: '高', TripleThreat: '高', LateGame: '低',
}

const COMP_CONDITIONS: Record<string, string> = {
  Rush: '激进≥3 且 上单=激进 且 打野=激进',
  PickOff: '激进≥3 且 打野=激进',
  AllIn: '激进≥4',
  MidJungle: '中路=激进 且 打野=激进 且 功能≥1',
  TopJungle: '上单=激进 且 打野=激进 且 团战≥1',
  Protect: 'ADC=后期 且 辅助=功能 且 团战≥1',
  Fortress: '团战≥2 且 功能≥2 且 激进=0',
  UtilityComp: '功能≥3',
  Stall: '后期≥2 且 功能≥2 且 团战≥1',
  BotLane: 'ADC=后期 且 辅助=功能/团战 且 激进≥1',
  Teamfight: '团战≥3',
  Dive: '激进≥2 且 团战≥2 且 打野=激进/团战',
  Skirmish: '激进≥2 且 后期≥1 且 (打野=激进 或 中路=激进)',
  DualCarry: '中路=后期 且 ADC=后期 且 功能≥1',
  Flex: '每种定位至少1个（激进+后期+功能+分推+团战各≥1）',
  Splitpush: '分推≥2 且 (上单=分推 或 中路=分推)',
  SideLane: '上单=分推 且 ADC=后期 且 团战≥2',
  Control: '功能≥2 且 后期≥2',
  TripleThreat: '上单=分推/激进 且 中路=激进/分推 且 ADC=后期',
  LateGame: '后期≥3',
}

const HARD_COUNTERS: [string, string][] = [
  ['Rush', 'Control'], ['PickOff', 'Splitpush'], ['AllIn', 'LateGame'],
  ['MidJungle', 'TripleThreat'], ['TopJungle', 'SideLane'],
  ['Protect', 'PickOff'], ['Fortress', 'AllIn'],
  ['UtilityComp', 'TopJungle'], ['Stall', 'Rush'],
  ['BotLane', 'MidJungle'], ['Teamfight', 'Skirmish'],
  ['Dive', 'DualCarry'], ['Skirmish', 'Stall'],
  ['DualCarry', 'Fortress'], ['Flex', 'Teamfight'],
  ['Splitpush', 'Protect'], ['SideLane', 'UtilityComp'],
  ['Control', 'BotLane'], ['TripleThreat', 'Dive'],
  ['LateGame', 'Flex'],
]

const SOFT_COUNTERS: [string, string][] = [
  ['Rush', 'LateGame'], ['Rush', 'DualCarry'],
  ['PickOff', 'LateGame'], ['PickOff', 'Control'],
  ['AllIn', 'TripleThreat'], ['AllIn', 'Splitpush'],
  ['MidJungle', 'SideLane'], ['MidJungle', 'Control'],
  ['TopJungle', 'Splitpush'], ['TopJungle', 'TripleThreat'],
  ['Protect', 'Dive'], ['Protect', 'AllIn'],
  ['Fortress', 'PickOff'], ['Fortress', 'MidJungle'],
  ['UtilityComp', 'PickOff'], ['UtilityComp', 'Rush'],
  ['Stall', 'AllIn'], ['Stall', 'TopJungle'],
  ['BotLane', 'Skirmish'], ['BotLane', 'Dive'],
  ['Teamfight', 'Flex'], ['Teamfight', 'SideLane'],
  ['Dive', 'Stall'], ['Dive', 'Protect'],
  ['Skirmish', 'BotLane'], ['Skirmish', 'Fortress'],
  ['DualCarry', 'UtilityComp'], ['DualCarry', 'Flex'],
  ['Flex', 'Stall'], ['Flex', 'MidJungle'],
  ['Splitpush', 'BotLane'], ['Splitpush', 'UtilityComp'],
  ['SideLane', 'Fortress'], ['SideLane', 'Skirmish'],
  ['Control', 'Rush'], ['Control', 'DualCarry'],
  ['TripleThreat', 'Protect'], ['TripleThreat', 'Teamfight'],
  ['LateGame', 'TopJungle'], ['LateGame', 'Teamfight'],
]

interface CompMatchup {
  id: string
  name: string
  difficulty: string
  archetypes: string[]
  hardCounters: string[]
  hardCounteredBy: string[]
  softCounters: string[]
  softCounteredBy: string[]
}

const compMatchups = computed<CompMatchup[]>(() => {
  const list: CompMatchup[] = []
  for (const type in COMP_NAME_MAP) {
    const hardCounters = HARD_COUNTERS.filter(x => x[0] === type).map(x => x[1])
    const hardCounteredBy = HARD_COUNTERS.filter(x => x[1] === type).map(x => x[0])
    const softCounters = SOFT_COUNTERS.filter(x => x[0] === type).map(x => x[1])
    const softCounteredBy = SOFT_COUNTERS.filter(x => x[1] === type).map(x => x[0])
    
    list.push({
      id: type,
      name: COMP_NAME_MAP[type],
      difficulty: COMP_DIFFICULTY[type] || '中',
      archetypes: COMP_CORE_ARCHETYPES[type] || [],
      hardCounters,
      hardCounteredBy,
      softCounters,
      softCounteredBy
    })
  }
  return list
})

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

const filteredPlayerUsage = computed(() => {
  return playerUsage.value.filter(item => {
    if (playerSearch.value && !item.player_name.toLowerCase().includes(playerSearch.value.toLowerCase())) return false
    if (playerPosFilter.value && item.position.toUpperCase() !== playerPosFilter.value) return false
    return true
  })
})

const pagedPlayerUsage = computed(() => {
  const start = (playerPage.value - 1) * playerPageSize.value
  return filteredPlayerUsage.value.slice(start, start + playerPageSize.value)
})

watch([playerSearch, playerPosFilter], () => {
  playerPage.value = 1
})

watch(selectedSeason, () => {
  fetchData()
})

const sortPlayerUsageWinRate = (a: PlayerChampionUsageItem, b: PlayerChampionUsageItem) => {
  const rateA = a.games > 0 ? a.wins / a.games : 0
  const rateB = b.games > 0 ? b.wins / b.games : 0
  return rateA - rateB
}

const playerUsageWinRateClass = (row: PlayerChampionUsageItem) => {
  if (row.games === 0) return ''
  const rate = row.wins / row.games
  if (rate > 0.55) return 'rate-high'
  if (rate < 0.45) return 'rate-low'
  return ''
}

const teamCompTeams = computed(() => {
  const teams = new Set(teamCompUsage.value.map(item => item.team_name))
  return Array.from(teams).sort()
})

const filteredTeamCompUsage = computed(() => {
  if (!compTeamFilter.value) return teamCompUsage.value
  return teamCompUsage.value.filter(item => item.team_name === compTeamFilter.value)
})

const sortTeamCompWinRate = (a: TeamCompUsageItem, b: TeamCompUsageItem) => {
  const rateA = a.games > 0 ? a.wins / a.games : 0
  const rateB = b.games > 0 ? b.wins / b.games : 0
  return rateA - rateB
}

const teamCompWinRateClass = (row: TeamCompUsageItem) => {
  if (row.games === 0) return ''
  const rate = row.wins / row.games
  if (rate > 0.55) return 'rate-high'
  if (rate < 0.45) return 'rate-low'
  return ''
}

const fetchData = async () => {
  try {
    const list = await getChampionList()
    champions.value = list

    const saveId = gameStore.currentSave?.id
    if (saveId) {
      const seasonId = selectedSeason.value > 0 ? selectedSeason.value : undefined
      const [stats, comps, pUsage, tComp] = await Promise.all([
        getChampionStats(saveId, seasonId),
        getCompStats(saveId, seasonId),
        getPlayerChampionUsage(saveId, seasonId),
        getTeamCompUsage(saveId, seasonId),
      ])
      championStats.value = stats
      compStats.value = comps
      playerUsage.value = pUsage
      teamCompUsage.value = tComp
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
  padding: 0;
  min-height: 100vh;
  background-color: #fff;

  .page-header {
    margin-bottom: 20px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;

    .header-actions {
      display: flex;
      gap: 12px;
      align-items: center;
    }

    .header-content {
      .page-title {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 24px;
        font-weight: 700;
        margin: 0 0 4px 0;
        color: #0f172a;
        letter-spacing: -0.3px;

        .el-icon {
          color: #6366f1;
        }
      }

      .page-description {
        margin: 0;
        color: #94a3b8;
        font-size: 13px;
      }
    }
  }

  .data-tabs {
    border: none;
    box-shadow: none;
    
    :deep(.el-tabs__header) {
      background-color: transparent;
      border-bottom: 1px solid #e2e8f0;
      margin-bottom: 20px;
    }

    :deep(.el-tabs__content) {
      padding: 0;
    }
    
    :deep(.el-tabs__item) {
      font-weight: 500;
      color: #64748b;
      
      &.is-active {
        color: #6366f1;
        font-weight: 600;
      }
    }
  }

  .filter-bar {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
  }
}

.data-table {
  width: 100%;
  
  :deep(.el-table__inner-wrapper::before) {
    display: none;
  }
  
  :deep(.el-table__header) {
    th.el-table__cell {
      background-color: #ffffff;
      color: #94a3b8;
      font-weight: 600;
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      padding: 12px 0;
      border-bottom: 1px solid #f1f5f9;
    }
  }

  :deep(.el-table__row) {
    transition: background-color 0.2s;
    
    td {
      padding: 12px 0;
      border-bottom: 1px solid #f8fafc;
    }

    &:hover {
      background-color: #f0f9ff !important;
      
      td {
        background-color: #f0f9ff !important;
      }
    }
    
    &:last-child td {
      border-bottom: none;
    }
  }
}

.champion-name,
.player-name,
.comp-name-text {
  font-weight: 600;
  color: #1f2937;
}

.rate-high {
  color: #10b981;
  font-weight: 600;
}

.rate-low {
  color: #ef4444;
  font-weight: 600;
}

.matchup-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}

.comp-card-wrap {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  transition: border-color 0.2s, transform 0.2s;
  background: #fff;

  &:hover {
    border-color: #6366f1;
    transform: translateY(-2px);
  }
}

.comp-card-top {
  padding: 14px 16px 10px;
  border-bottom: 1px solid #f1f5f9;
  background: linear-gradient(135deg, #f8fafc 0%, #fff 100%);

  .comp-card-name {
    font-size: 17px;
    font-weight: 700;
    color: #0f172a;
    margin-bottom: 8px;
  }

  .comp-card-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .comp-diff {
    display: inline-block;
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 4px;
    letter-spacing: 0.5px;

    &.diff-低 { background: #dcfce7; color: #166534; }
    &.diff-中 { background: #fef3c7; color: #92400e; }
    &.diff-高 { background: #fee2e2; color: #991b1b; }
  }

  .arch-tag {
    font-size: 11px;
  }
}

.comp-card-condition {
  padding: 10px 16px;
  background: #f8fafc;
  display: flex;
  gap: 8px;
  align-items: baseline;

  .cond-label {
    font-size: 11px;
    font-weight: 700;
    color: #94a3b8;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .cond-text {
    font-size: 12px;
    color: #475569;
    font-family: 'SF Mono', 'Menlo', monospace;
    line-height: 1.6;
  }
}

.comp-card-rels {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rel-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.rel-icon {
  font-size: 10px;
  width: 16px;
  text-align: center;
  flex-shrink: 0;

  &.rel-strong { color: #ef4444; }
  &.rel-weak { color: #6366f1; }
  &.rel-soft-strong { color: #f59e0b; }
  &.rel-soft-weak { color: #10b981; }
}

.rel-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.rel-tag {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;

  &.rel-tag-strong {
    background: #fef2f2;
    color: #dc2626;
    border: 1px solid #fecaca;
  }

  &.rel-tag-weak {
    background: #eef2ff;
    color: #4f46e5;
    border: 1px solid #c7d2fe;
  }

  &.rel-tag-soft-strong {
    background: #fffbeb;
    color: #d97706;
    border: 1px solid #fde68a;
  }

  &.rel-tag-soft-weak {
    background: #ecfdf5;
    color: #059669;
    border: 1px solid #a7f3d0;
  }
}

.player-name {
  font-weight: 600;
  color: #1f2937;
}

.score-text {
  font-family: monospace;
  font-weight: 600;
  color: #6366f1;
}

.comp-name-text {
  font-weight: 600;
  color: #1f2937;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
