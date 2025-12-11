<template>
  <div class="tournaments-view">
    <!-- 历史赛季提示 -->
    <el-alert
      v-if="isViewingHistory"
      type="warning"
      show-icon
      :closable="false"
      class="history-alert"
    >
      <template #title>
        <div class="history-alert-content">
          <span>您正在查看 {{ viewingSeasonId }} 赛季的历史数据</span>
          <el-button type="primary" size="small" @click="returnToActiveSeason">
            返回当前赛季 ({{ activeSeasonId }})
          </el-button>
        </div>
      </template>
    </el-alert>

    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>赛事管理</h1>
        <p>{{ viewingSeasonId }} 赛季赛事概览</p>
      </div>
      <div class="header-actions">
        <el-select
          v-model="selectedSeason"
          placeholder="选择赛季"
          @change="onSeasonChange"
        >
          <el-option
            v-for="season in availableSeasons"
            :key="season.id"
            :label="season.label"
            :value="season.id"
          />
        </el-select>
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

    <!-- 赛事卡片网格 -->
    <el-row :gutter="20">
      <el-col :span="8" v-for="tournament in tournaments" :key="tournament.id">
        <el-card class="tournament-card" :class="tournament.status">
          <!-- 赛事头部图片 -->
          <div class="tournament-header" :class="tournament.type">
            <div class="tournament-badge">
              {{ tournament.type === 'league' ? '联赛' : '国际赛' }}
            </div>
            <img
              v-if="tournament.poster"
              :src="tournament.poster"
              :alt="tournament.name"
              class="tournament-poster"
            />
            <div v-else class="tournament-icon">
              {{ tournament.type === 'league' ? '🏆' : '🌍' }}
            </div>
          </div>

          <!-- 赛事内容 -->
          <div class="tournament-content">
            <div class="tournament-title-row">
              <h3 class="tournament-name">{{ tournament.name }}</h3>
              <el-tag :type="getStatusTagType(tournament.status)" size="default">
                {{ getStatusText(tournament.status) }}
              </el-tag>
            </div>

            <p class="tournament-description">{{ tournament.description }}</p>

            <div class="tournament-info">
              <div class="info-item" v-if="tournament.teams">
                <el-icon><UserFilled /></el-icon>
                <span>{{ tournament.teams }} 支队伍</span>
              </div>
              <div class="info-item" v-if="tournament.matches">
                <el-icon><VideoPlay /></el-icon>
                <span>{{ tournament.matches }} 场比赛</span>
              </div>
              <div class="info-item" v-if="tournament.prize">
                <el-icon><Trophy /></el-icon>
                <span>{{ tournament.prize }}</span>
              </div>
            </div>

            <!-- 进度条 -->
            <div class="tournament-progress" v-if="tournament.status === 'active'">
              <div class="progress-label">
                <span>比赛进度</span>
                <span>{{ tournament.progress }}%</span>
              </div>
              <el-progress
                :percentage="tournament.progress"
                :stroke-width="8"
                :show-text="false"
                :color="'#67c23a'"
              />
            </div>

            <!-- 操作按钮 -->
            <div class="tournament-actions">
              <el-button
                v-if="tournament.status === 'active'"
                type="success"
                @click="continueTournament(tournament)"
              >
                <el-icon><VideoPlay /></el-icon>
                继续比赛
              </el-button>
              <el-button
                v-else-if="tournament.status === 'upcoming'"
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

    <!-- 赛季时间线 -->
    <el-card class="timeline-card">
      <template #header>
        <div class="timeline-header">
          <h2>赛季时间线</h2>
          <el-tag type="primary" effect="dark">{{ viewingSeasonId }}</el-tag>
        </div>
      </template>

      <el-timeline>
        <el-timeline-item
          v-for="tournament in tournaments"
          :key="tournament.id"
          :type="getTimelineType(tournament.status)"
          :hollow="tournament.status === 'upcoming'"
          :timestamp="tournament.time"
          placement="top"
        >
          <div class="timeline-content">
            <div class="timeline-title">
              <span class="timeline-name">{{ tournament.name }}</span>
              <el-tag
                :type="getStatusTagType(tournament.status)"
                size="small"
              >
                {{ getStatusText(tournament.status) }}
              </el-tag>
            </div>
            <p class="timeline-desc">{{ tournament.description }}</p>
          </div>
        </el-timeline-item>
      </el-timeline>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  Trophy,
  VideoPlay,
  Clock,
  CircleCheck,
  UserFilled,
  View,
} from '@element-plus/icons-vue'
import { useSeasonStore } from '@/stores/useSeasonStore'

// 导入海报图片
import msiPoster from '@/assets/posters/msi.png'
import madridPoster from '@/assets/posters/马德里.png'
import claudePoster from '@/assets/posters/claude.png'
import worldsPoster from '@/assets/posters/s世界赛.png'
import shanghaiPoster from '@/assets/posters/shanghai.png'

const router = useRouter()
const seasonStore = useSeasonStore()

// 从store获取响应式数据
const {
  viewingSeasonId,
  activeSeasonId,
  availableSeasons,
  isViewingHistory
} = storeToRefs(seasonStore)

// 本地选择的赛季（与store同步）
const selectedSeason = ref('S1')

// 赛季切换
const onSeasonChange = (seasonId: string) => {
  seasonStore.switchViewingSeason(seasonId)
}

const returnToActiveSeason = () => {
  seasonStore.returnToActiveSeason()
  selectedSeason.value = activeSeasonId.value
}

// 初始化
onMounted(() => {
  seasonStore.loadSeasons()
  selectedSeason.value = viewingSeasonId.value
})

// 监听store中的赛季变化
watch(viewingSeasonId, (newVal) => {
  selectedSeason.value = newVal
})

// 模拟数据
const tournaments = ref([
  {
    id: 1,
    name: '春季赛',
    type: 'league',
    status: 'active',
    description: '四大赛区春季常规赛与季后赛',
    teams: 56,
    matches: 224,
    prize: '冠军积分 300',
    progress: 45,
    time: '1月-4月',
    poster: null,
  },
  {
    id: 2,
    name: 'MSI 季中邀请赛',
    type: 'international',
    status: 'upcoming',
    description: '春季赛冠亚季军参加的国际赛事',
    teams: 12,
    matches: 48,
    prize: '冠军积分 200',
    progress: 0,
    time: '5月',
    poster: msiPoster,
  },
  {
    id: 3,
    name: '马德里大师赛',
    type: 'international',
    status: 'upcoming',
    description: '各赛区前8名参加的邀请赛',
    teams: 32,
    matches: 64,
    prize: '荣誉赛事',
    progress: 0,
    time: '5月',
    poster: madridPoster,
  },
  {
    id: 4,
    name: '夏季赛',
    type: 'league',
    status: 'upcoming',
    description: '四大赛区夏季常规赛与季后赛',
    teams: 56,
    matches: 224,
    prize: '冠军积分 300',
    progress: 0,
    time: '6月-9月',
    poster: null,
  },
  {
    id: 5,
    name: 'Claude 洲际赛',
    type: 'international',
    status: 'upcoming',
    description: 'LPL vs LCK vs LEC vs LCS 四赛区对抗',
    teams: 32,
    matches: 48,
    prize: '荣誉赛事',
    progress: 0,
    time: '7月',
    poster: claudePoster,
  },
  {
    id: 6,
    name: 'S 世界赛',
    type: 'international',
    status: 'upcoming',
    description: '全球最高荣誉的年度总决赛',
    teams: 16,
    matches: 64,
    prize: '冠军积分 500',
    progress: 0,
    time: '10月-11月',
    poster: worldsPoster,
  },
  {
    id: 7,
    name: '上海大师赛',
    type: 'international',
    status: 'upcoming',
    description: '夏季赛冠亚季军参加的邀请赛',
    teams: 12,
    matches: 32,
    prize: '荣誉赛事',
    progress: 0,
    time: '9月',
    poster: shanghaiPoster,
  },
  {
    id: 8,
    name: 'ICP 洲际对抗赛',
    type: 'international',
    status: 'upcoming',
    description: '四大赛区年度对抗表演赛',
    teams: 16,
    matches: 24,
    prize: '荣誉赛事',
    progress: 0,
    time: '8月',
    poster: null,
  },
  {
    id: 9,
    name: 'Super 洲际邀请赛',
    type: 'international',
    status: 'upcoming',
    description: '年度积分前16名参加的年终盛典',
    teams: 16,
    matches: 32,
    prize: '冠军积分 100',
    progress: 0,
    time: '12月',
    poster: null,
  },
])

// 计算属性
const activeTournaments = computed(() =>
  tournaments.value.filter(t => t.status === 'active').length
)

const upcomingTournaments = computed(() =>
  tournaments.value.filter(t => t.status === 'upcoming').length
)

const completedTournaments = computed(() =>
  tournaments.value.filter(t => t.status === 'completed').length
)

// 方法
const getStatusTagType = (status: string) => {
  switch (status) {
    case 'active': return 'success'
    case 'upcoming': return 'info'
    case 'completed': return 'primary'
    default: return 'info'
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'active': return '进行中'
    case 'upcoming': return '未开始'
    case 'completed': return '已完成'
    default: return '未知'
  }
}

const getTimelineType = (status: string) => {
  switch (status) {
    case 'active': return 'success'
    case 'completed': return 'primary'
    default: return 'info'
  }
}

const continueTournament = (tournament: any) => {
  console.log('Continue tournament:', tournament.name)
  navigateToDetail(tournament)
}

const viewResults = (tournament: any) => {
  console.log('View results:', tournament.name)
  navigateToDetail(tournament)
}

const viewDetails = (tournament: any) => {
  console.log('View details:', tournament.name)
  navigateToDetail(tournament)
}

const navigateToDetail = (tournament: any) => {
  // 根据赛事类型跳转到不同的详情页
  switch (tournament.id) {
    case 1: // 春季赛
      router.push(`/tournaments/${tournament.id}`)
      break
    case 2: // MSI 季中邀请赛
      router.push('/tournaments/msi')
      break
    case 3: // 马德里大师赛
      router.push(`/tournaments/madrid/${tournament.id}`)
      break
    case 4: // 夏季赛
      router.push(`/tournaments/summer/${tournament.id}`)
      break
    case 5: // Claude 洲际赛
      router.push(`/tournaments/clauch/${tournament.id}`)
      break
    case 6: // S 世界赛
      router.push('/tournaments/worlds')
      break
    case 7: // 上海大师赛（与MSI相同赛制）
      router.push('/tournaments/shanghai')
      break
    case 8: // ICP 洲际对抗赛
      router.push(`/tournaments/icp/${tournament.id}`)
      break
    case 9: // Super 洲际年度邀请赛
      router.push(`/tournaments/super/${tournament.id}`)
      break
    default: // 其他赛事
      router.push(`/tournaments/${tournament.id}`)
  }
}
</script>

<style scoped>
.tournaments-view {
  padding: 0;
}

/* 历史赛季提示 */
.history-alert {
  margin-bottom: 16px;
  border-radius: 8px;
}

.history-alert-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
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

.header-actions {
  display: flex;
  gap: 12px;
}

.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
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
  background: linear-gradient(135deg, #667eea, #764ba2);
}

.stat-icon.green {
  background: linear-gradient(135deg, #11998e, #38ef7d);
}

.stat-icon.orange {
  background: linear-gradient(135deg, #f093fb, #f5576c);
}

.stat-icon.purple {
  background: linear-gradient(135deg, #4facfe, #00f2fe);
}

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

/* 赛事卡片 */
.tournament-card {
  margin-bottom: 20px;
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.tournament-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.15);
}

.tournament-card.active {
  border-left: 4px solid #67c23a;
}

.tournament-card.upcoming {
  border-left: 4px solid #409eff;
}

.tournament-card.completed {
  border-left: 4px solid #909399;
}

.tournament-card :deep(.el-card__body) {
  padding: 0;
}

.tournament-header {
  height: 140px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.tournament-header.league {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.tournament-header.international {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.tournament-poster {
  width: 100%;
  height: 100%;
  object-fit: cover;
  position: absolute;
  top: 0;
  left: 0;
}

.tournament-badge {
  position: absolute;
  top: 12px;
  left: 12px;
  padding: 4px 12px;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(4px);
  border-radius: 20px;
  color: white;
  font-size: 12px;
  font-weight: 500;
  z-index: 1;
}

.tournament-icon {
  font-size: 48px;
}

.tournament-content {
  padding: 20px;
}

.tournament-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.tournament-name {
  font-size: 18px;
  font-weight: 700;
  color: #303133;
  margin: 0;
}

.tournament-description {
  font-size: 14px;
  color: #909399;
  margin: 0 0 16px 0;
  line-height: 1.5;
}

.tournament-info {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #606266;
}

.info-item .el-icon {
  color: #909399;
}

.tournament-progress {
  margin-bottom: 16px;
}

.progress-label {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: #606266;
  margin-bottom: 8px;
}

.tournament-actions {
  display: flex;
  gap: 8px;
}

.tournament-actions .el-button {
  flex: 1;
}

/* 时间线 */
.timeline-card {
  margin-top: 20px;
  border-radius: 12px;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.timeline-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  margin: 0;
}

.timeline-content {
  padding: 12px 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.timeline-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.timeline-name {
  font-weight: 600;
  color: #303133;
}

.timeline-desc {
  font-size: 13px;
  color: #909399;
  margin: 0;
}
</style>
