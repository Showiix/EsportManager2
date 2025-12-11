import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

// 游戏阶段定义
export type GamePhase =
  | 'spring_regular'      // 春季赛常规赛
  | 'spring_playoffs'     // 春季赛季后赛
  | 'msi'                 // MSI季中赛
  | 'madrid'              // 马德里大师赛
  | 'summer_regular'      // 夏季赛常规赛
  | 'summer_playoffs'     // 夏季赛季后赛
  | 'claude'              // Claude洲际赛
  | 'worlds'              // S世界赛
  | 'shanghai'            // 上海大师赛
  | 'icp'                 // ICP四赛区洲际对抗赛
  | 'super'               // Super洲际年度邀请赛
  | 'transfer'            // 转会期
  | 'draft'               // 选秀
  | 'season_end'          // 赛季结束

// 阶段信息
export interface PhaseInfo {
  id: GamePhase
  name: string
  description: string
  order: number
}

// 赛季数据接口
export interface SeasonData {
  id: string              // 如 'S1', 'S2'
  number: number          // 赛季数字，如 1, 2
  currentPhase: GamePhase
  completedPhases: GamePhase[]
  isActive: boolean       // 是否是当前正在进行的赛季
}

// 所有阶段定义
export const GAME_PHASES: PhaseInfo[] = [
  { id: 'spring_regular', name: '春季赛常规赛', description: '四大赛区春季常规赛', order: 1 },
  { id: 'spring_playoffs', name: '春季赛季后赛', description: '四大赛区春季季后赛', order: 2 },
  { id: 'msi', name: 'MSI 季中邀请赛', description: '春季赛冠亚季军参加的国际赛事', order: 3 },
  { id: 'madrid', name: '马德里大师赛', description: '各赛区前8名参加的邀请赛', order: 4 },
  { id: 'summer_regular', name: '夏季赛常规赛', description: '四大赛区夏季常规赛', order: 5 },
  { id: 'summer_playoffs', name: '夏季赛季后赛', description: '四大赛区夏季季后赛', order: 6 },
  { id: 'claude', name: 'Claude 洲际赛', description: 'LPL vs LCK vs LEC vs LCS 四赛区对抗', order: 7 },
  { id: 'worlds', name: 'S 世界赛', description: '全球最高荣誉的年度总决赛', order: 8 },
  { id: 'shanghai', name: '上海大师赛', description: '夏季赛冠亚季军参加的邀请赛', order: 9 },
  { id: 'icp', name: 'ICP 洲际对抗赛', description: '四大赛区年度对抗表演赛', order: 10 },
  { id: 'super', name: 'Super 洲际邀请赛', description: '年度积分前16名参加的年终盛典', order: 11 },
  { id: 'transfer', name: '转会期', description: '选手转会窗口期', order: 12 },
  { id: 'draft', name: '选秀', description: '新秀选拔', order: 13 },
  { id: 'season_end', name: '赛季结束', description: '赛季总结与数据归档', order: 14 },
]

const STORAGE_KEY = 'esport-manager-seasons'

export const useSeasonStore = defineStore('season', () => {
  // 所有赛季数据
  const seasons = ref<SeasonData[]>([])

  // 当前活跃赛季ID（正在进行的赛季）
  const activeSeasonId = ref<string>('S1')

  // 当前查看的赛季ID（用于浏览历史数据）
  const viewingSeasonId = ref<string>('S1')

  // 加载状态
  const isLoading = ref(false)

  // 计算属性：当前活跃赛季
  const activeSeason = computed(() =>
    seasons.value.find(s => s.id === activeSeasonId.value)
  )

  // 计算属性：当前查看的赛季
  const viewingSeason = computed(() =>
    seasons.value.find(s => s.id === viewingSeasonId.value)
  )

  // 计算属性：是否正在查看历史赛季
  const isViewingHistory = computed(() =>
    viewingSeasonId.value !== activeSeasonId.value
  )

  // 计算属性：所有可用赛季列表（用于选择器）
  const availableSeasons = computed(() =>
    seasons.value.map(s => ({
      id: s.id,
      number: s.number,
      isActive: s.isActive,
      label: s.isActive ? `${s.id} (当前)` : s.id
    }))
  )

  // 计算属性：当前阶段信息
  const currentPhaseInfo = computed(() => {
    const season = activeSeason.value
    if (!season) return null
    return GAME_PHASES.find(p => p.id === season.currentPhase)
  })

  // 初始化/加载数据
  const loadSeasons = async () => {
    isLoading.value = true
    try {
      const saved = localStorage.getItem(STORAGE_KEY)
      if (saved) {
        const data = JSON.parse(saved)
        seasons.value = data.seasons || []
        activeSeasonId.value = data.activeSeasonId || 'S1'
        viewingSeasonId.value = data.viewingSeasonId || activeSeasonId.value
      } else {
        // 初始化默认赛季
        initializeDefaultSeason()
      }
    } catch (err) {
      console.error('Failed to load seasons:', err)
      initializeDefaultSeason()
    } finally {
      isLoading.value = false
    }
  }

  // 初始化默认赛季
  const initializeDefaultSeason = () => {
    seasons.value = [{
      id: 'S1',
      number: 1,
      currentPhase: 'spring_regular',
      completedPhases: [],
      isActive: true
    }]
    activeSeasonId.value = 'S1'
    viewingSeasonId.value = 'S1'
    saveSeasons()
  }

  // 保存数据
  const saveSeasons = () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      seasons: seasons.value,
      activeSeasonId: activeSeasonId.value,
      viewingSeasonId: viewingSeasonId.value
    }))
  }

  // 切换查看的赛季（不影响游戏进度）
  const switchViewingSeason = (seasonId: string) => {
    const season = seasons.value.find(s => s.id === seasonId)
    if (season) {
      viewingSeasonId.value = seasonId
      saveSeasons()
    }
  }

  // 返回查看当前赛季
  const returnToActiveSeason = () => {
    viewingSeasonId.value = activeSeasonId.value
    saveSeasons()
  }

  // 推进到下一个阶段
  const advancePhase = () => {
    const season = activeSeason.value
    if (!season) return false

    const currentPhaseIndex = GAME_PHASES.findIndex(p => p.id === season.currentPhase)
    if (currentPhaseIndex === -1) return false

    // 将当前阶段加入已完成列表
    if (!season.completedPhases.includes(season.currentPhase)) {
      season.completedPhases.push(season.currentPhase)
    }

    // 如果是最后一个阶段，开始新赛季
    if (currentPhaseIndex >= GAME_PHASES.length - 1) {
      startNewSeason()
    } else {
      // 否则推进到下一阶段
      season.currentPhase = GAME_PHASES[currentPhaseIndex + 1].id
    }

    saveSeasons()
    return true
  }

  // 开始新赛季
  const startNewSeason = () => {
    const currentSeason = activeSeason.value
    if (currentSeason) {
      currentSeason.isActive = false
    }

    const newSeasonNumber = seasons.value.length + 1
    const newSeason: SeasonData = {
      id: `S${newSeasonNumber}`,
      number: newSeasonNumber,
      currentPhase: 'spring_regular',
      completedPhases: [],
      isActive: true
    }

    seasons.value.push(newSeason)
    activeSeasonId.value = newSeason.id
    viewingSeasonId.value = newSeason.id
    saveSeasons()
  }

  // 检查是否可以进行某个阶段的操作
  const canAccessPhase = (phase: GamePhase): boolean => {
    const season = activeSeason.value
    if (!season) return false

    // 如果是已完成的阶段，可以查看
    if (season.completedPhases.includes(phase)) return true

    // 如果是当前阶段，可以操作
    if (season.currentPhase === phase) return true

    return false
  }

  // 检查当前是否是某个阶段
  const isCurrentPhase = (phase: GamePhase): boolean => {
    const season = activeSeason.value
    return season?.currentPhase === phase
  }

  // 获取阶段状态
  const getPhaseStatus = (phase: GamePhase): 'completed' | 'active' | 'upcoming' => {
    const season = activeSeason.value
    if (!season) return 'upcoming'

    if (season.completedPhases.includes(phase)) return 'completed'
    if (season.currentPhase === phase) return 'active'
    return 'upcoming'
  }

  // 获取赛季的阶段完成情况
  const getSeasonProgress = (seasonId?: string): { completed: number; total: number; percentage: number } => {
    const season = seasonId
      ? seasons.value.find(s => s.id === seasonId)
      : activeSeason.value

    if (!season) {
      return { completed: 0, total: GAME_PHASES.length, percentage: 0 }
    }

    const completed = season.completedPhases.length
    const total = GAME_PHASES.length
    const percentage = Math.round((completed / total) * 100)

    return { completed, total, percentage }
  }

  return {
    // 状态
    seasons,
    activeSeasonId,
    viewingSeasonId,
    isLoading,

    // 计算属性
    activeSeason,
    viewingSeason,
    isViewingHistory,
    availableSeasons,
    currentPhaseInfo,

    // 方法
    loadSeasons,
    saveSeasons,
    switchViewingSeason,
    returnToActiveSeason,
    advancePhase,
    startNewSeason,
    canAccessPhase,
    isCurrentPhase,
    getPhaseStatus,
    getSeasonProgress
  }
})
