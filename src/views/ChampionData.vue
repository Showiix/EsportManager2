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
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'pick_count', order: 'descending' }"
        >
          <el-table-column label="体系" width="160">
            <template #default="{ row }">
              <span class="comp-name-text">{{ compName(row.comp_type) }}</span>
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
        <el-table
          v-if="compViewMode === 'team' && filteredTeamCompUsage.length > 0"
          :data="filteredTeamCompUsage"
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'games', order: 'descending' }"
        >
          <el-table-column prop="team_name" label="战队" width="140" />
          <el-table-column label="体系" width="140">
            <template #default="{ row }">
              <span class="comp-name-text">{{ row.comp_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="games" label="使用次数" width="100" align="center" sortable />
          <el-table-column prop="wins" label="胜场" width="80" align="center" sortable />
          <el-table-column label="胜率" width="100" align="center" sortable :sort-method="sortTeamCompWinRate">
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
          <el-card v-for="comp in compMatchups" :key="comp.id" class="comp-card" shadow="hover">
            <template #header>
              <div class="comp-card-header">
                <div class="comp-title">
                  <span class="name">{{ comp.name }}</span>
                  <el-tag :type="getDifficultyType(comp.difficulty)" size="small" effect="plain">
                    难度: {{ comp.difficulty }}
                  </el-tag>
                </div>
                <div class="archetypes">
                  <el-tag v-for="arch in comp.archetypes" :key="arch" type="info" size="small" class="arch-tag">
                    {{ arch }}
                  </el-tag>
                </div>
              </div>
            </template>
            <div class="comp-relationships">
              <div class="rel-section condition-section">
                <span class="rel-label">触发条件:</span>
                <div class="condition-text">{{ COMP_CONDITIONS[comp.id] }}</div>
              </div>
              <div class="rel-section" v-if="comp.hardCounters.length">
                <span class="rel-label">克制:</span>
                <div class="tags">
                  <el-tag v-for="t in comp.hardCounters" :key="t" type="danger" size="small" effect="light">
                    {{ COMP_NAME_MAP[t] || t }}
                  </el-tag>
                </div>
              </div>
              <div class="rel-section" v-if="comp.hardCounteredBy.length">
                <span class="rel-label">被克制:</span>
                <div class="tags">
                  <el-tag v-for="t in comp.hardCounteredBy" :key="t" type="" size="small" effect="light">
                    {{ COMP_NAME_MAP[t] || t }}
                  </el-tag>
                </div>
              </div>
              <div class="rel-section" v-if="comp.softCounters.length">
                <span class="rel-label">小克:</span>
                <div class="tags">
                  <el-tag v-for="t in comp.softCounters" :key="t" type="warning" size="small" effect="plain">
                    {{ COMP_NAME_MAP[t] || t }}
                  </el-tag>
                </div>
              </div>
              <div class="rel-section" v-if="comp.softCounteredBy.length">
                <span class="rel-label">被小克:</span>
                <div class="tags">
                  <el-tag v-for="t in comp.softCounteredBy" :key="t" type="success" size="small" effect="plain">
                    {{ COMP_NAME_MAP[t] || t }}
                  </el-tag>
                </div>
              </div>
            </div>
          </el-card>
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
          stripe
          style="width: 100%"
          :default-sort="{ prop: 'games', order: 'descending' }"
        >
          <el-table-column prop="player_name" label="选手" width="130">
            <template #default="{ row }">
              <span class="player-name">{{ row.player_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="team_name" label="战队" width="120" />
          <el-table-column prop="position" label="位置" width="80" align="center">
            <template #default="{ row }">
              <el-tag :type="positionTagType(row.position)" size="small">
                {{ positionName(row.position.charAt(0).toUpperCase() + row.position.slice(1).toLowerCase()) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="champion_name" label="英雄" width="120" />
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

const getDifficultyType = (diff: string): '' | 'success' | 'warning' | 'danger' => {
  if (diff === '高') return 'danger'
  if (diff === '中') return 'warning'
  return 'success'
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
  padding: 24px;
  min-height: 100%;

  .page-header {
    margin-bottom: 24px;
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

.matchup-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
  
  .comp-card {
    .comp-card-header {
      display: flex;
      flex-direction: column;
      gap: 8px;
      
      .comp-title {
        display: flex;
        justify-content: space-between;
        align-items: center;
        
        .name {
          font-size: 16px;
          font-weight: 700;
          color: #1f2937;
        }
      }
      
      .archetypes {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
      }
    }
    
    .comp-relationships {
      display: flex;
      flex-direction: column;
      gap: 12px;
      
      .rel-section {
        .rel-label {
          display: block;
          font-size: 12px;
          color: #6b7280;
          margin-bottom: 4px;
        }
        
        .tags {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
        }

        &.condition-section {
          background: #f8f9fa;
          border-radius: 6px;
          padding: 8px 10px;

          .condition-text {
            font-size: 13px;
            color: #374151;
            font-family: monospace;
            line-height: 1.5;
          }
        }
      }
    }
  }
}

.player-name {
  font-weight: 600;
  color: #1f2937;
}

.score-text {
  font-family: monospace;
  font-weight: 600;
  color: #409eff;
}

.comp-name-text {
  font-weight: 600;
  color: #374151;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
