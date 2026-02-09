<template>
  <div class="tournaments-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>赛事管理</h1>
        <div style="display: flex; align-items: center; gap: 12px;">
          <p>赛事概览</p>
          <SeasonSelector v-model="selectedSeason" />
        </div>
      </div>
      <div class="header-actions">
        <el-tooltip content="修复赛事状态" placement="bottom">
          <el-button circle size="small" @click="handleFixTournamentStatus" :loading="isFixing">
            <el-icon><Tools /></el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="刷新" placement="bottom">
          <el-button circle size="small" @click="refreshTournaments" :loading="isLoading">
            <el-icon><Refresh /></el-icon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <!-- 统计栏 -->
    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ tournaments.length }}</span>
        <span class="stat-label">赛事总数</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value highlight">{{ activeTournaments }}</span>
        <span class="stat-label">进行中</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ upcomingTournaments }}</span>
        <span class="stat-label">未开始</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ completedTournaments }}</span>
        <span class="stat-label">已完成</span>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <!-- 赛事卡片网格 -->
    <div v-else class="tournament-grid">
      <div
        v-for="group in groupedTournaments"
        :key="group.isLeague ? group.type : group.originalTournament?.id"
        class="tournament-card"
        :class="[group.status, group.isLeague ? 'league' : 'international']"
      >
        <div class="card-top">
          <span class="card-badge" :class="group.isLeague ? 'league' : 'international'">
            {{ group.isLeague ? '联赛' : '国际赛' }}
          </span>
          <el-tag
            :type="group.status === 'active' ? 'success' : group.status === 'completed' ? '' : 'info'"
            size="small"
          >
            {{ group.status === 'active' ? '进行中' : group.status === 'completed' ? '已完成' : '未开始' }}
          </el-tag>
        </div>

        <h3 class="tournament-name">{{ group.name }}</h3>
        <p class="tournament-desc">
          {{ group.isLeague ? `四大赛区 ${group.tournaments.length} 场赛事` : group.originalTournament?.tournament_type || '' }}
        </p>

        <div class="card-meta">
          <span class="meta-item">
            <el-icon><Trophy /></el-icon>
            S{{ selectedSeason }} 赛季
          </span>
          <span v-if="group.regions.length > 0" class="meta-item">
            <el-icon><UserFilled /></el-icon>
            {{ group.regions.join(' / ') }}
          </span>
        </div>

        <div v-if="group.regions.length > 0" class="region-tags">
          <span v-for="region in group.regions" :key="region" class="region-tag">{{ region }}</span>
        </div>

        <!-- 操作按钮 -->
        <div class="card-actions">
          <el-button
            v-if="group.status === 'active'"
            type="success"
            size="small"
            @click="navigateToGroup(group)"
          >
            <el-icon><VideoPlay /></el-icon>
            继续比赛
          </el-button>
          <el-button
            v-else-if="group.status === 'upcoming'"
            size="small"
            disabled
          >
            <el-icon><Clock /></el-icon>
            等待开始
          </el-button>
          <el-button
            v-else
            size="small"
            @click="navigateToGroup(group)"
          >
            <el-icon><View /></el-icon>
            查看结果
          </el-button>
          <button class="detail-btn" @click="navigateToGroup(group)">详情 →</button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <el-empty v-if="!isLoading && tournaments.length === 0" description="暂无赛事数据，请先加载存档" />

    <!-- 赛季时间线 -->
    <div v-if="tournaments.length > 0" class="table-section timeline-section">
      <div class="timeline-header">
        <h2>赛季时间线</h2>
        <el-tag type="primary" size="small">{{ currentSeason }}</el-tag>
      </div>

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
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Trophy,
  VideoPlay,
  Clock,
  UserFilled,
  View,
  Refresh,
  Tools,
} from '@element-plus/icons-vue'
import { useTournamentStoreTauri } from '@/stores/useTournamentStoreTauri'
import { useGameStore } from '@/stores/useGameStore'
import { useSeasonStore } from '@/stores/useSeasonStore'
import { queryApi, timeApi } from '@/api/tauri'
import { createLogger } from '@/utils/logger'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const logger = createLogger('Tournaments')

const router = useRouter()
const tournamentStore = useTournamentStoreTauri()
const gameStore = useGameStore()
const seasonStore = useSeasonStore()

// 从 store 获取响应式数据
const { currentSeason, gameState } = storeToRefs(gameStore)

// 赛季选择
const selectedSeason = ref(0)

// 本地赛事列表（合并赛季赛事和国际赛事）
const tournaments = ref<any[]>([])
const isLoading = ref(false)
const isFixing = ref(false)

// 初始化加载数据
onMounted(async () => {
  selectedSeason.value = seasonStore.currentSeason
  await loadAllTournaments()
  // 自动检测并修复赛事状态
  try {
    const result = await timeApi.fixTournamentStatus()
    if (result.fixed_count > 0) {
      ElMessage.success(result.message)
      await loadAllTournaments()
    }
  } catch (e) {
    logger.error('Auto fix tournament status failed:', e)
  }
})

// 监听赛季切换
watch(selectedSeason, () => {
  loadAllTournaments()
})

// 加载所有赛事（赛季 + 国际）
const loadAllTournaments = async () => {
  if (!selectedSeason.value) {
    logger.debug('No season selected')
    return
  }

  isLoading.value = true
  const seasonId = selectedSeason.value
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
  'SpringRegular': { name: '春季常规赛', icon: '', order: 1 },
  'SpringPlayoffs': { name: '春季季后赛', icon: '', order: 2 },
  'SummerRegular': { name: '夏季常规赛', icon: '', order: 4 },
  'SummerPlayoffs': { name: '夏季季后赛', icon: '', order: 5 },
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
  const seasonQuery = { season: String(selectedSeason.value) }

  // 联赛 - 根据类型跳转
  if (type === 'SpringRegular') {
    router.push({ path: `/tournaments/spring/${tournament.id}`, query: seasonQuery })
  } else if (type === 'SpringPlayoffs') {
    router.push({ path: `/tournaments/spring-playoffs/${tournament.id}`, query: seasonQuery })
  } else if (type === 'SummerRegular') {
    router.push({ path: `/tournaments/summer/${tournament.id}`, query: seasonQuery })
  } else if (type === 'SummerPlayoffs') {
    router.push({ path: `/tournaments/summer-playoffs/${tournament.id}`, query: seasonQuery })
  } else if (type === 'Msi') {
    router.push({ path: '/tournaments/msi', query: seasonQuery })
  } else if (type === 'WorldChampionship') {
    router.push({ path: '/tournaments/worlds', query: seasonQuery })
  } else if (type === 'ShanghaiMasters') {
    router.push({ path: '/tournaments/shanghai', query: seasonQuery })
  } else if (type === 'MadridMasters') {
    router.push({ path: `/tournaments/madrid/${tournament.id}`, query: seasonQuery })
  } else if (type === 'ClaudeIntercontinental') {
    router.push({ path: `/tournaments/clauch/${tournament.id}`, query: seasonQuery })
  } else if (type === 'IcpIntercontinental') {
    router.push({ path: `/tournaments/icp/${tournament.id}`, query: seasonQuery })
  } else if (type === 'SuperIntercontinental') {
    router.push({ path: `/tournaments/super/${tournament.id}`, query: seasonQuery })
  } else {
    // 默认跳转
    router.push({ path: `/tournaments/${tournament.id}`, query: seasonQuery })
  }
}

// 导航到合并的赛事组详情
const navigateToGroup = async (group: TournamentGroup) => {
  const seasonQuery = { season: String(selectedSeason.value) }

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
      router.push({ path: `/tournaments/spring/${firstTournament.id}`, query: { grouped: 'true', ...seasonQuery } })
    } else if (type === 'SpringPlayoffs') {
      router.push({ path: `/tournaments/spring-playoffs/${firstTournament.id}`, query: { grouped: 'true', ...seasonQuery } })
    } else if (type === 'SummerRegular') {
      router.push({ path: `/tournaments/summer/${firstTournament.id}`, query: { grouped: 'true', ...seasonQuery } })
    } else if (type === 'SummerPlayoffs') {
      router.push({ path: `/tournaments/summer-playoffs/${firstTournament.id}`, query: { grouped: 'true', ...seasonQuery } })
    } else {
      router.push({ path: `/tournaments/${firstTournament.id}`, query: seasonQuery })
    }
  }
}

</script>

<style scoped>
.tournaments-view {
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
  color: #0f172a;
  margin: 0 0 4px 0;
  letter-spacing: -0.3px;
}

.page-header p {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 8px;
}

/* 统计栏 */
.stats-bar {
  display: flex;
  align-items: center;
  padding: 14px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  margin-bottom: 20px;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  flex: 1;
  justify-content: center;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  font-variant-numeric: tabular-nums;
}

.stat-value.highlight {
  color: #6366f1;
}

.stat-label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 500;
}

.stat-divider {
  width: 1px;
  height: 24px;
  background: #e2e8f0;
  flex-shrink: 0;
}

/* 加载 */
.loading-container {
  padding: 40px;
}

/* 赛事卡片网格 */
.tournament-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.tournament-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 20px;
  transition: all 0.2s ease;
}

.tournament-card:hover {
  border-color: #6366f1;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.08);
}

.tournament-card.active {
  border-left: 3px solid #10b981;
}

.tournament-card.upcoming {
  border-left: 3px solid #6366f1;
}

.tournament-card.completed {
  border-left: 3px solid #94a3b8;
}

/* 卡片顶部 */
.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.card-badge {
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.card-badge.league {
  background: rgba(99, 102, 241, 0.08);
  color: #6366f1;
}

.card-badge.international {
  background: rgba(139, 92, 246, 0.08);
  color: #8b5cf6;
}

/* 卡片内容 */
.tournament-name {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.tournament-desc {
  font-size: 13px;
  color: #94a3b8;
  margin: 0 0 12px 0;
}

.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 12px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #64748b;
}

.meta-item .el-icon {
  color: #94a3b8;
}

/* 赛区标签 */
.region-tags {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
}

.region-tag {
  padding: 2px 8px;
  background: #f1f5f9;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 500;
  color: #64748b;
}

/* 操作按钮 */
.card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid #f1f5f9;
}

.detail-btn {
  margin-left: auto;
  padding: 5px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #ffffff;
  color: #475569;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.detail-btn:hover {
  border-color: #6366f1;
  color: #6366f1;
  background: #f5f3ff;
}

/* 时间线 */
.table-section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 20px;
  background: #ffffff;
}

.timeline-section {
  margin-top: 4px;
  padding: 20px 24px;
}

.timeline-section :deep(.el-timeline) {
  padding-left: 0;
}

.timeline-section :deep(.el-timeline-item__tail) {
  left: 5px;
}

.timeline-section :deep(.el-timeline-item__node) {
  left: 0;
}

.timeline-section :deep(.el-timeline-item__wrapper) {
  padding-left: 24px;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.timeline-header h2 {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.timeline-content {
  padding: 10px 14px;
  background: #f8fafc;
  border-radius: 6px;
  border: 1px solid #f1f5f9;
  transition: all 0.2s ease;
}

.timeline-content.current-phase {
  background: #f0fdf4;
  border: 1px solid #10b981;
}

.timeline-content.current-phase .timeline-name {
  color: #10b981;
}

.timeline-title {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.timeline-name {
  font-weight: 600;
  font-size: 14px;
  color: #0f172a;
}

.timeline-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
}

/* 响应式 */
@media (max-width: 1200px) {
  .tournament-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .stats-bar {
    flex-wrap: wrap;
    gap: 8px;
  }

  .stat-divider {
    display: none;
  }
}

@media (max-width: 768px) {
  .tournament-grid {
    grid-template-columns: 1fr;
  }
}
</style>
