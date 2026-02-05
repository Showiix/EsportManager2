<template>
  <div class="tournaments-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>赛事管理</h1>
        <p>{{ currentSeason }} 赛季赛事概览</p>
      </div>
      <div class="header-actions">
        <el-button type="warning" @click="handleFixTournamentStatus" :loading="isFixing">
          <el-icon><Tools /></el-icon>
          修复状态
        </el-button>
        <el-button type="primary" @click="refreshTournaments" :loading="isLoading">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><Trophy /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ tournaments.length }}</div>
              <div class="stat-label">赛事总数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="28"><VideoPlay /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ activeTournaments }}</div>
              <div class="stat-label">进行中</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="28"><Clock /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ upcomingTournaments }}</div>
              <div class="stat-label">未开始</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="28"><CircleCheck /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ completedTournaments }}</div>
              <div class="stat-label">已完成</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 加载状态 -->
    <el-card v-if="isLoading" class="loading-card">
      <el-skeleton :rows="10" animated />
    </el-card>

    <!-- 赛事卡片网格 -->
    <el-row v-else :gutter="20">
      <el-col :span="8" v-for="group in groupedTournaments" :key="group.isLeague ? group.type : group.originalTournament?.id">
        <el-card class="tournament-card" :class="group.status">
          <!-- 联赛头部 -->
          <div v-if="group.isLeague" class="tournament-header league">
            <div class="tournament-badge">联赛</div>
            <div class="tournament-icon">{{ group.icon }}</div>
            <!-- 赛区标签 -->
            <div v-if="group.regions.length > 0" class="region-tags">
              <span v-for="region in group.regions" :key="region" class="region-tag">{{ region }}</span>
            </div>
          </div>
          <!-- 国际赛事头部 - 使用图片 -->
          <div v-else class="tournament-header international" :style="getTournamentHeaderStyle(group.originalTournament)">
            <div class="tournament-badge">国际赛</div>
          </div>

          <!-- 赛事内容 -->
          <div class="tournament-content">
            <div class="tournament-title-row">
              <h3 class="tournament-name">{{ group.name }}</h3>
              <el-tag :type="group.status === 'active' ? 'success' : group.status === 'completed' ? 'primary' : 'info'" size="default">
                {{ group.status === 'active' ? '进行中' : group.status === 'completed' ? '已完成' : '未开始' }}
              </el-tag>
            </div>

            <p class="tournament-description">
              {{ group.isLeague ? `四大赛区 ${group.tournaments.length} 场赛事` : group.originalTournament?.tournament_type || '' }}
            </p>

            <div class="tournament-info">
              <div class="info-item">
                <el-icon><Trophy /></el-icon>
                <span>S{{ gameState?.current_season }} 赛季</span>
              </div>
              <div class="info-item" v-if="group.regions.length > 0">
                <el-icon><UserFilled /></el-icon>
                <span>{{ group.regions.join(' / ') }}</span>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="tournament-actions">
              <el-button
                v-if="group.status === 'active'"
                type="success"
                @click="navigateToGroup(group)"
              >
                <el-icon><VideoPlay /></el-icon>
                继续比赛
              </el-button>
              <el-button
                v-else-if="group.status === 'upcoming'"
                type="primary"
                disabled
              >
                <el-icon><Clock /></el-icon>
                等待开始
              </el-button>
              <el-button
                v-else
                type="info"
                @click="navigateToGroup(group)"
              >
                <el-icon><View /></el-icon>
                查看结果
              </el-button>
              <el-button @click="navigateToGroup(group)">
                详情
              </el-button>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 空状态 -->
    <el-card v-if="!isLoading && tournaments.length === 0" class="empty-card">
      <el-empty description="暂无赛事数据，请先加载存档" />
    </el-card>

    <!-- 赛季时间线 -->
    <el-card v-if="tournaments.length > 0" class="timeline-card">
      <template #header>
        <div class="timeline-header">
          <h2>赛季时间线</h2>
          <el-tag type="primary" effect="dark">{{ currentSeason }}</el-tag>
        </div>
      </template>

      <el-timeline>
        <el-timeline-item
          v-for="phase in seasonTimeline"
          :key="phase.type"
          :type="phase.timelineType"
          :hollow="phase.status === 'upcoming'"
          placement="top"
        >
          <div class="timeline-content" :class="{ 'current-phase': phase.isCurrent }">
            <div class="timeline-title">
              <span class="timeline-name">{{ phase.name }}</span>
              <el-tag
                :type="phase.tagType"
                size="small"
              >
                {{ phase.statusText }}
              </el-tag>
            </div>
            <p class="timeline-desc">{{ phase.description }}</p>
          </div>
        </el-timeline-item>
      </el-timeline>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Trophy,
  VideoPlay,
  Clock,
  CircleCheck,
  UserFilled,
  View,
  Refresh,
  Tools,
} from '@element-plus/icons-vue'
import { useTournamentStoreTauri } from '@/stores/useTournamentStoreTauri'
import { useGameStore } from '@/stores/useGameStore'
import { queryApi, timeApi } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('Tournaments')

const router = useRouter()
const tournamentStore = useTournamentStoreTauri()
const gameStore = useGameStore()

// 从 store 获取响应式数据
const { currentSeason, gameState } = storeToRefs(gameStore)

// 本地赛事列表（合并赛季赛事和国际赛事）
const tournaments = ref<any[]>([])
const isLoading = ref(false)
const isFixing = ref(false)

// 初始化加载数据
onMounted(async () => {
  await loadAllTournaments()
})

// 加载所有赛事（赛季 + 国际）
const loadAllTournaments = async () => {
  if (!gameState.value?.current_season) {
    logger.debug('No current season')
    return
  }

  isLoading.value = true
  const seasonId = gameState.value.current_season
  logger.debug('Loading tournaments for season:', seasonId)

  try {
    const [seasonTournaments, internationalTournaments] = await Promise.all([
      queryApi.getSeasonTournaments(seasonId),
      queryApi.getInternationalTournaments(seasonId)
    ])

    logger.debug('Season tournaments:', seasonTournaments)
    logger.debug('International tournaments:', internationalTournaments)

    // 合并并去重
    const allTournaments = [...seasonTournaments]
    for (const intl of internationalTournaments) {
      if (!allTournaments.some(t => t.id === intl.id)) {
        allTournaments.push(intl)
      }
    }
    tournaments.value = allTournaments
    logger.debug(`Loaded ${allTournaments.length} tournaments (${seasonTournaments.length} season + ${internationalTournaments.length} international)`)
  } catch (e) {
    logger.error('Failed to load tournaments:', e)
  } finally {
    isLoading.value = false
  }
}

// 刷新赛事列表
const refreshTournaments = async () => {
  await loadAllTournaments()
}

// 修复赛事状态
const handleFixTournamentStatus = async () => {
  isFixing.value = true
  try {
    const result = await timeApi.fixTournamentStatus()
    if (result.fixed_count > 0) {
      ElMessage.success(`${result.message}`)
      // 刷新赛事列表
      await loadAllTournaments()
    } else {
      ElMessage.info(result.message)
    }
  } catch (e) {
    logger.error('修复赛事状态失败:', e)
    ElMessage.error('修复赛事状态失败')
  } finally {
    isFixing.value = false
  }
}

// 计算属性
const activeTournaments = computed(() =>
  tournaments.value.filter(t => getTournamentStatus(t) === 'active').length
)

const upcomingTournaments = computed(() =>
  tournaments.value.filter(t => getTournamentStatus(t) === 'upcoming').length
)

const completedTournaments = computed(() =>
  tournaments.value.filter(t => getTournamentStatus(t) === 'completed').length
)

// 赛季阶段顺序（按时间推进引擎顺序）
const SEASON_PHASES = [
  { type: 'SpringRegular', name: '春季常规赛', description: '四大赛区春季常规赛' },
  { type: 'SpringPlayoffs', name: '春季季后赛', description: '四大赛区春季季后赛' },
  { type: 'Msi', name: 'MSI季中赛', description: '赛区冠军国际对抗' },
  { type: 'MadridMasters', name: '马德里大师赛', description: '国际邀请赛' },
  { type: 'SummerRegular', name: '夏季常规赛', description: '四大赛区夏季常规赛' },
  { type: 'SummerPlayoffs', name: '夏季季后赛', description: '四大赛区夏季季后赛' },
  { type: 'ClaudeIntercontinental', name: 'Claude洲际赛', description: '洲际对抗赛' },
  { type: 'WorldChampionship', name: 'S世界赛', description: '全球总决赛' },
  { type: 'ShanghaiMasters', name: '上海大师赛', description: '年终大师赛' },
  { type: 'IcpIntercontinental', name: 'ICP洲际对抗赛', description: '四赛区洲际对抗' },
  { type: 'SuperIntercontinental', name: 'Super洲际邀请赛', description: '年度邀请赛' },
]

// 计算赛季时间线
const seasonTimeline = computed(() => {
  const currentPhase = gameState.value?.current_phase || 'SpringRegular'

  return SEASON_PHASES.map((phase, index) => {
    // 查找该阶段对应的赛事
    const phaseTournaments = tournaments.value.filter(t => t.tournament_type === phase.type)

    // 判断阶段状态
    let status: 'active' | 'upcoming' | 'completed' = 'upcoming'
    const currentPhaseIndex = SEASON_PHASES.findIndex(p => p.type === currentPhase)

    if (index < currentPhaseIndex) {
      status = 'completed'
    } else if (index === currentPhaseIndex) {
      status = 'active'
    } else {
      status = 'upcoming'
    }

    // 如果有对应赛事，根据赛事状态更精确判断
    if (phaseTournaments.length > 0) {
      const statuses = phaseTournaments.map(t => getTournamentStatus(t))
      if (statuses.every(s => s === 'completed')) {
        status = 'completed'
      } else if (statuses.some(s => s === 'active')) {
        status = 'active'
      }
    }

    return {
      type: phase.type,
      name: phase.name,
      description: phase.description,
      status,
      isCurrent: phase.type === currentPhase,
      timelineType: status === 'active' ? 'success' : status === 'completed' ? 'primary' : 'info',
      tagType: status === 'active' ? 'success' : status === 'completed' ? 'primary' : 'info',
      statusText: status === 'active' ? '进行中' : status === 'completed' ? '已完成' : '未开始',
    }
  })
})

// 需要合并的联赛类型
const leagueTypes = ['SpringRegular', 'SpringPlayoffs', 'SummerRegular', 'SummerPlayoffs']

// 联赛类型配置
const leagueTypeConfig: Record<string, { name: string, icon: string, order: number }> = {
  'SpringRegular': { name: '春季常规赛', icon: '🌸', order: 1 },
  'SpringPlayoffs': { name: '春季季后赛', icon: '🏆', order: 2 },
  'SummerRegular': { name: '夏季常规赛', icon: '☀️', order: 4 },
  'SummerPlayoffs': { name: '夏季季后赛', icon: '🏆', order: 5 },
}

// 赛事分组（只合并联赛，国际赛事保持原样）
interface TournamentGroup {
  type: string
  name: string
  icon: string
  isLeague: boolean
  order: number
  tournaments: any[]
  regions: string[]
  status: 'active' | 'upcoming' | 'completed'
  progress: number
  // 国际赛事使用原始数据
  originalTournament?: any
}

const groupedTournaments = computed<TournamentGroup[]>(() => {
  const leagueGroups: Record<string, TournamentGroup> = {}
  const internationalList: TournamentGroup[] = []

  logger.debug('Processing tournaments:', tournaments.value.length)

  for (const t of tournaments.value) {
    const type = t.tournament_type || 'Unknown'

    // 检查是否是需要合并的联赛类型
    if (leagueTypes.includes(type)) {
      const config = leagueTypeConfig[type]

      if (!leagueGroups[type]) {
        leagueGroups[type] = {
          type,
          name: config.name,
          icon: config.icon,
          isLeague: true,
          order: config.order,
          tournaments: [],
          regions: [],
          status: 'upcoming',
          progress: 0
        }
      }

      leagueGroups[type].tournaments.push(t)

      // 提取赛区名称
      const regionMatch = t.name?.match(/(LPL|LCK|LEC|LCS)/)
      if (regionMatch && !leagueGroups[type].regions.includes(regionMatch[1])) {
        leagueGroups[type].regions.push(regionMatch[1])
      }
    } else {
      // 国际赛事 - 保持原样单独显示
      logger.debug('International tournament:', t.name, t.tournament_type)
      internationalList.push({
        type,
        name: t.name, // 使用原始名称
        icon: '', // 国际赛事用图片，不需要emoji
        isLeague: false,
        order: 100, // 国际赛事排在后面
        tournaments: [t],
        regions: [],
        status: getTournamentStatus(t),
        progress: getProgress(t),
        originalTournament: t
      })
    }
  }

  // 计算联赛组的状态和进度
  for (const group of Object.values(leagueGroups)) {
    const statuses = group.tournaments.map(t => getTournamentStatus(t))
    if (statuses.some(s => s === 'active')) {
      group.status = 'active'
    } else if (statuses.every(s => s === 'completed')) {
      group.status = 'completed'
    } else {
      group.status = 'upcoming'
    }

    // 计算平均进度
    const progresses = group.tournaments.map(t => getProgress(t))
    group.progress = Math.round(progresses.reduce((a, b) => a + b, 0) / progresses.length)
  }

  // 合并并排序：联赛在前，国际赛事在后
  const allGroups = [...Object.values(leagueGroups), ...internationalList]
  logger.debug('Grouped result:', allGroups.length, 'groups (', Object.keys(leagueGroups).length, 'leagues +', internationalList.length, 'international)')
  logger.debug('All groups:', allGroups.map(g => ({ name: g.name, isLeague: g.isLeague, order: g.order })))
  return allGroups.sort((a, b) => a.order - b.order)
})

// 获取赛事状态
const getTournamentStatus = (tournament: any): 'active' | 'upcoming' | 'completed' => {
  // TournamentInfo has status field with values like 'Scheduled', 'InProgress', 'Completed'
  const status = tournament.status?.toLowerCase() ?? ''
  if (status === 'completed' || tournament.champion_team_id) return 'completed'
  if (status === 'inprogress' || status === 'in_progress') return 'active'
  return 'upcoming'
}

// 获取进度 (simplified since we don't have match_count)
const getProgress = (tournament: any): number => {
  // Without match counts in TournamentInfo, we return 50% for active tournaments
  const status = getTournamentStatus(tournament)
  if (status === 'completed') return 100
  if (status === 'active') return 50
  return 0
}

const navigateToDetail = (tournament: any) => {
  // 根据赛事类型跳转到不同的详情页
  // 后端返回 PascalCase 格式如 SpringRegular，直接比较原始值
  const type = tournament.tournament_type || ''

  // 联赛 - 根据类型跳转
  if (type === 'SpringRegular') {
    router.push(`/tournaments/spring/${tournament.id}`)
  } else if (type === 'SpringPlayoffs') {
    router.push(`/tournaments/spring-playoffs/${tournament.id}`)
  } else if (type === 'SummerRegular') {
    router.push(`/tournaments/summer/${tournament.id}`)
  } else if (type === 'SummerPlayoffs') {
    router.push(`/tournaments/summer-playoffs/${tournament.id}`)
  } else if (type === 'Msi') {
    router.push('/tournaments/msi')
  } else if (type === 'WorldChampionship') {
    router.push('/tournaments/worlds')
  } else if (type === 'ShanghaiMasters') {
    router.push('/tournaments/shanghai')
  } else if (type === 'MadridMasters') {
    router.push(`/tournaments/madrid/${tournament.id}`)
  } else if (type === 'ClaudeIntercontinental') {
    router.push(`/tournaments/clauch/${tournament.id}`)
  } else if (type === 'IcpIntercontinental') {
    router.push(`/tournaments/icp/${tournament.id}`)
  } else if (type === 'SuperIntercontinental') {
    router.push(`/tournaments/super/${tournament.id}`)
  } else {
    // 默认跳转
    router.push(`/tournaments/${tournament.id}`)
  }
}

// 导航到合并的赛事组详情
const navigateToGroup = async (group: TournamentGroup) => {
  // 国际赛事直接跳转到原有页面
  if (!group.isLeague && group.originalTournament) {
    await tournamentStore.selectTournament(group.originalTournament.id)
    navigateToDetail(group.originalTournament)
    return
  }

  // 联赛：选择第一个赛事（通常是LPL）
  const firstTournament = group.tournaments[0]
  if (firstTournament) {
    await tournamentStore.selectTournament(firstTournament.id)

    // 根据类型跳转，传入 regionGroup 参数表示需要显示赛区选择
    const type = group.type
    if (type === 'SpringRegular') {
      router.push({ path: `/tournaments/spring/${firstTournament.id}`, query: { grouped: 'true' } })
    } else if (type === 'SpringPlayoffs') {
      router.push({ path: `/tournaments/spring-playoffs/${firstTournament.id}`, query: { grouped: 'true' } })
    } else if (type === 'SummerRegular') {
      router.push({ path: `/tournaments/summer/${firstTournament.id}`, query: { grouped: 'true' } })
    } else if (type === 'SummerPlayoffs') {
      router.push({ path: `/tournaments/summer-playoffs/${firstTournament.id}`, query: { grouped: 'true' } })
    } else {
      router.push(`/tournaments/${firstTournament.id}`)
    }
  }
}

// 获取国际赛事头部样式（背景图片）
const getTournamentHeaderStyle = (tournament: any) => {
  if (!tournament) return {}

  const type = tournament.tournament_type || ''
  const imageMap: Record<string, string> = {
    'Msi': '/images/tournaments/msi.png',
    'WorldChampionship': '/images/tournaments/worlds.png',
    'ShanghaiMasters': '/images/tournaments/shanghai.png',
    'MadridMasters': '/images/tournaments/madrid.png',
    'ClaudeIntercontinental': '/images/tournaments/claude.png',
  }

  const imagePath = imageMap[type]
  if (imagePath) {
    return {
      backgroundImage: `url(${imagePath})`,
      backgroundSize: 'cover',
      backgroundPosition: 'center'
    }
  }

  return {}
}
</script>

<style scoped>
.tournaments-view { padding: 0; }

.page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 20px; }
.page-header h1 { font-size: 24px; font-weight: 700; color: #303133; margin: 0 0 8px 0; }
.page-header p { font-size: 14px; color: #909399; margin: 0; }
.header-actions { display: flex; gap: 12px; }

.stats-row { margin-bottom: 20px; }
.stat-card { border-radius: 12px; }
.stat-content { display: flex; align-items: center; gap: 16px; padding: 8px 0; }
.stat-icon { width: 56px; height: 56px; border-radius: 12px; display: flex; align-items: center; justify-content: center; color: white; }
.stat-icon.blue { background: linear-gradient(135deg, #667eea, #764ba2); }
.stat-icon.green { background: linear-gradient(135deg, #11998e, #38ef7d); }
.stat-icon.orange { background: linear-gradient(135deg, #f093fb, #f5576c); }
.stat-icon.purple { background: linear-gradient(135deg, #4facfe, #00f2fe); }
.stat-info { flex: 1; }
.stat-number { font-size: 28px; font-weight: 700; color: #303133; line-height: 1; }
.stat-label { font-size: 14px; color: #909399; margin-top: 4px; }

.loading-card, .empty-card { border-radius: 12px; margin-bottom: 20px; }

/* 赛事卡片 */
.tournament-card { margin-bottom: 20px; border-radius: 12px; overflow: hidden; transition: all 0.3s ease; }
.tournament-card:hover { transform: translateY(-4px); box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15); }
.tournament-card.active { border-left: 4px solid #67c23a; }
.tournament-card.upcoming { border-left: 4px solid #409eff; }
.tournament-card.completed { border-left: 4px solid #909399; }
.tournament-card :deep(.el-card__body) { padding: 0; }

.tournament-header { height: 140px; display: flex; align-items: center; justify-content: center; position: relative; overflow: hidden; }
.tournament-header.league { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }
.tournament-header.international { background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); }
.tournament-badge { position: absolute; top: 12px; left: 12px; padding: 4px 12px; background: rgba(255, 255, 255, 0.2); backdrop-filter: blur(4px); border-radius: 20px; color: white; font-size: 12px; font-weight: 500; z-index: 1; }
.tournament-icon { font-size: 48px; }
.region-tags { position: absolute; bottom: 12px; left: 12px; display: flex; gap: 6px; flex-wrap: wrap; }
.region-tag { padding: 2px 8px; background: rgba(255, 255, 255, 0.25); backdrop-filter: blur(4px); border-radius: 12px; color: white; font-size: 11px; font-weight: 500; }

.tournament-content { padding: 20px; }
.tournament-title-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.tournament-name { font-size: 18px; font-weight: 700; color: #303133; margin: 0; }
.tournament-description { font-size: 14px; color: #909399; margin: 0 0 16px 0; line-height: 1.5; }

.tournament-info { display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 16px; }
.info-item { display: flex; align-items: center; gap: 6px; font-size: 13px; color: #606266; }
.info-item .el-icon { color: #909399; }

.tournament-actions { display: flex; gap: 8px; }
.tournament-actions .el-button { flex: 1; }

/* 时间线 */
.timeline-card { margin-top: 20px; border-radius: 12px; }
.timeline-header { display: flex; justify-content: space-between; align-items: center; }
.timeline-header h2 { font-size: 18px; font-weight: 600; color: #303133; margin: 0; }
.timeline-content {
  padding: 12px 16px;
  background: #f5f7fa;
  border-radius: 8px;
  transition: all 0.3s ease;
}
.timeline-content.current-phase {
  background: linear-gradient(135deg, #e8f5e9, #c8e6c9);
  border: 2px solid #4caf50;
  box-shadow: 0 2px 8px rgba(76, 175, 80, 0.2);
}
.timeline-content.current-phase .timeline-name {
  color: #2e7d32;
}
.timeline-title { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.timeline-name { font-weight: 600; color: #303133; }
.timeline-desc { font-size: 13px; color: #909399; margin: 0; }
</style>
