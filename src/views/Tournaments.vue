<template>
  <div class="tournaments-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>赛事管理</h1>
        <p>{{ currentSeason }} 赛季赛事概览</p>
      </div>
      <div class="header-actions">
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
      <el-col :span="8" v-for="tournament in tournaments" :key="tournament.id">
        <el-card class="tournament-card" :class="getTournamentStatus(tournament)">
          <!-- 赛事头部 -->
          <div class="tournament-header" :class="tournament.region_id ? 'league' : 'international'">
            <div class="tournament-badge">
              {{ tournament.region_id ? '联赛' : '国际赛' }}
            </div>
            <div class="tournament-icon">
              {{ tournament.region_id ? '🏆' : '🌍' }}
            </div>
          </div>

          <!-- 赛事内容 -->
          <div class="tournament-content">
            <div class="tournament-title-row">
              <h3 class="tournament-name">{{ tournament.name }}</h3>
              <el-tag :type="getStatusTagType(tournament)" size="default">
                {{ getStatusText(tournament) }}
              </el-tag>
            </div>

            <p class="tournament-description">{{ tournament.tournament_type }}</p>

            <div class="tournament-info">
              <div class="info-item">
                <el-icon><UserFilled /></el-icon>
                <span>{{ tournament.champion_team_name ?? '待定' }}</span>
              </div>
              <div class="info-item">
                <el-icon><VideoPlay /></el-icon>
                <span>第 {{ tournament.season_id }} 赛季</span>
              </div>
            </div>

            <!-- 进度条 -->
            <div class="tournament-progress" v-if="getTournamentStatus(tournament) === 'active'">
              <div class="progress-label">
                <span>比赛进度</span>
                <span>{{ getProgress(tournament) }}%</span>
              </div>
              <el-progress
                :percentage="getProgress(tournament)"
                :stroke-width="8"
                :show-text="false"
                :color="'#67c23a'"
              />
            </div>

            <!-- 操作按钮 -->
            <div class="tournament-actions">
              <el-button
                v-if="getTournamentStatus(tournament) === 'active'"
                type="success"
                @click="continueTournament(tournament)"
              >
                <el-icon><VideoPlay /></el-icon>
                继续比赛
              </el-button>
              <el-button
                v-else-if="getTournamentStatus(tournament) === 'upcoming'"
                type="primary"
                disabled
              >
                <el-icon><Clock /></el-icon>
                等待开始
              </el-button>
              <el-button
                v-else
                type="info"
                @click="viewResults(tournament)"
              >
                <el-icon><View /></el-icon>
                查看结果
              </el-button>
              <el-button @click="viewDetails(tournament)">
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
          v-for="tournament in tournaments"
          :key="tournament.id"
          :type="getTimelineType(tournament)"
          :hollow="getTournamentStatus(tournament) === 'upcoming'"
          placement="top"
        >
          <div class="timeline-content">
            <div class="timeline-title">
              <span class="timeline-name">{{ tournament.name }}</span>
              <el-tag
                :type="getStatusTagType(tournament)"
                size="small"
              >
                {{ getStatusText(tournament) }}
              </el-tag>
            </div>
            <p class="timeline-desc">{{ tournament.tournament_type }}</p>
          </div>
        </el-timeline-item>
      </el-timeline>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  Trophy,
  VideoPlay,
  Clock,
  CircleCheck,
  UserFilled,
  View,
  Refresh,
} from '@element-plus/icons-vue'
import { useTournamentStoreTauri } from '@/stores/useTournamentStoreTauri'
import { useGameStore } from '@/stores/useGameStore'

const router = useRouter()
const tournamentStore = useTournamentStoreTauri()
const gameStore = useGameStore()

// 从 store 获取响应式数据
const { tournaments, isLoading } = storeToRefs(tournamentStore)
const { currentSeason, gameState } = storeToRefs(gameStore)

// 初始化加载数据
onMounted(async () => {
  if (gameState.value?.current_season) {
    await tournamentStore.loadSeasonTournaments(gameState.value.current_season)
  }
})

// 刷新赛事列表
const refreshTournaments = async () => {
  if (gameState.value?.current_season) {
    await tournamentStore.loadSeasonTournaments(gameState.value.current_season)
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

// 方法
const getStatusTagType = (tournament: any) => {
  const status = getTournamentStatus(tournament)
  switch (status) {
    case 'active': return 'success'
    case 'upcoming': return 'info'
    case 'completed': return 'primary'
    default: return 'info'
  }
}

const getStatusText = (tournament: any) => {
  const status = getTournamentStatus(tournament)
  switch (status) {
    case 'active': return '进行中'
    case 'upcoming': return '未开始'
    case 'completed': return '已完成'
    default: return '未知'
  }
}

const getTimelineType = (tournament: any) => {
  const status = getTournamentStatus(tournament)
  switch (status) {
    case 'active': return 'success'
    case 'completed': return 'primary'
    default: return 'info'
  }
}

const continueTournament = async (tournament: any) => {
  await tournamentStore.selectTournament(tournament.id)
  navigateToDetail(tournament)
}

const viewResults = async (tournament: any) => {
  await tournamentStore.selectTournament(tournament.id)
  navigateToDetail(tournament)
}

const viewDetails = async (tournament: any) => {
  await tournamentStore.selectTournament(tournament.id)
  navigateToDetail(tournament)
}

const navigateToDetail = (tournament: any) => {
  // 根据赛事类型跳转到不同的详情页
  // 后端返回 PascalCase 格式如 SpringRegular，直接比较原始值
  const type = tournament.tournament_type || ''

  // 联赛 - 根据类型跳转
  if (type === 'SpringRegular' || type === 'SpringPlayoffs') {
    router.push(`/tournaments/spring/${tournament.id}`)
  } else if (type === 'SummerRegular' || type === 'SummerPlayoffs') {
    router.push(`/tournaments/summer/${tournament.id}`)
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

.tournament-content { padding: 20px; }
.tournament-title-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.tournament-name { font-size: 18px; font-weight: 700; color: #303133; margin: 0; }
.tournament-description { font-size: 14px; color: #909399; margin: 0 0 16px 0; line-height: 1.5; }

.tournament-info { display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 16px; }
.info-item { display: flex; align-items: center; gap: 6px; font-size: 13px; color: #606266; }
.info-item .el-icon { color: #909399; }

.tournament-progress { margin-bottom: 16px; }
.progress-label { display: flex; justify-content: space-between; font-size: 13px; color: #606266; margin-bottom: 8px; }

.tournament-actions { display: flex; gap: 8px; }
.tournament-actions .el-button { flex: 1; }

/* 时间线 */
.timeline-card { margin-top: 20px; border-radius: 12px; }
.timeline-header { display: flex; justify-content: space-between; align-items: center; }
.timeline-header h2 { font-size: 18px; font-weight: 600; color: #303133; margin: 0; }
.timeline-content { padding: 12px 16px; background: #f5f7fa; border-radius: 8px; }
.timeline-title { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.timeline-name { font-weight: 600; color: #303133; }
.timeline-desc { font-size: 13px; color: #909399; margin: 0; }
</style>
