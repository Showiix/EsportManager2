<template>
  <div class="team-honor-rankings">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>战队荣誉榜</h1>
        <p>查看所有战队的荣誉记录与排名</p>
      </div>
      <button class="back-btn" @click="router.push('/honors')">← 返回荣誉殿堂</button>
    </div>

    <!-- 筛选和排序 -->
    <div class="filter-section">
      <div class="filter-row">
        <div class="filter-group">
          <label>排序</label>
          <el-radio-group v-model="sortBy" size="small">
            <el-radio-button value="champion">总冠军</el-radio-button>
            <el-radio-button value="international">国际冠军</el-radio-button>
            <el-radio-button value="runner_up">亚军数</el-radio-button>
          </el-radio-group>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <!-- 空状态 -->
    <el-empty v-else-if="rankings.length === 0" description="暂无战队荣誉记录" />

    <!-- 排行榜列表 -->
    <div v-else class="rankings-list">
      <!-- 前三名大卡片 -->
      <div class="top-three">
        <div
          v-for="team in topThree"
          :key="team.team_id"
          class="top-card"
          :class="getRankClass(team.rank)"
          @click="goToDetail(team.team_id)"
        >
          <div class="rank-badge">
            {{ getRankBadge(team.rank) }}
          </div>
          <div class="team-info">
            <div class="team-name">{{ team.team_name }}</div>
          </div>
          <div class="honor-stats">
            <div class="stat-row">
              <div class="stat-item">
                <span class="stat-value">{{ team.champion_count }}</span>
                <span class="stat-label">冠军</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ team.international_champion_count }}</span>
                <span class="stat-label">国际</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ team.runner_up_count }}</span>
                <span class="stat-label">亚军</span>
              </div>
            </div>
          </div>
          <div class="view-detail">查看详情 →</div>
        </div>
      </div>

      <!-- 其他排名列表 -->
      <div class="table-section">
        <div class="table-list">
          <div
            v-for="team in restRankings"
            :key="team.team_id"
            class="ranking-row"
            @click="goToDetail(team.team_id)"
          >
            <div class="rank-num">#{{ team.rank }}</div>
            <div class="team-info">
              <span class="name">{{ team.team_name }}</span>
            </div>
            <div class="stats">
              <span class="stat">{{ team.champion_count }} 冠</span>
              <span class="stat">{{ team.international_champion_count }} 国际</span>
              <span class="stat">{{ team.runner_up_count }} 亚</span>
            </div>
            <div class="action">
              <el-button size="small" type="primary" text>详情 →</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'

import { tauriApi, type TeamHonorRanking } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TeamHonorRankings')

const router = useRouter()
const loading = ref(true)
const rankings = ref<TeamHonorRanking[]>([])
const sortBy = ref('champion')

// 加载排行榜
const loadRankings = async () => {
  loading.value = true
  try {
    const res = await tauriApi.honor.getTeamHonorRankings(100)
    rankings.value = res || []
    sortRankings()
  } catch (error) {
    logger.error('Failed to load rankings:', error)
    rankings.value = []
  } finally {
    loading.value = false
  }
}

// 排序
const sortRankings = () => {
  rankings.value.sort((a, b) => {
    if (sortBy.value === 'champion') {
      return b.champion_count - a.champion_count
    } else if (sortBy.value === 'international') {
      return b.international_champion_count - a.international_champion_count
    } else {
      return b.runner_up_count - a.runner_up_count
    }
  })
  // 更新排名
  rankings.value.forEach((t, idx) => {
    t.rank = idx + 1
  })
}

// 监听排序变化
watch(sortBy, () => {
  sortRankings()
})

// 前三名
const topThree = computed(() => rankings.value.slice(0, 3))

// 其他排名
const restRankings = computed(() => rankings.value.slice(3))

// 获取排名样式类
const getRankClass = (rank: number): string => {
  if (rank === 1) return 'gold'
  if (rank === 2) return 'silver'
  if (rank === 3) return 'bronze'
  return ''
}

// 获取排名徽章
const getRankBadge = (rank: number): string => {
  if (rank === 1) return '#1'
  if (rank === 2) return '#2'
  if (rank === 3) return '#3'
  return `#${rank}`
}

// 跳转到详情页
const goToDetail = (teamId: number) => {
  router.push(`/teams/${teamId}`)
}

onMounted(() => {
  loadRankings()
})
</script>

<style scoped>
.team-honor-rankings {
  padding: 0;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.page-header p {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.filter-section {
  margin-bottom: 16px;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 20px;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-group label {
  font-size: 13px;
  color: #64748b;
  white-space: nowrap;
}

.loading-container {
  padding: 40px;
}

.rankings-list {
}

.top-three {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
}

.top-card {
  flex: 1;
  padding: 20px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.top-card:hover {
  border-color: #6366f1;
  transform: translateY(-3px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.1);
}

.top-card.gold {
  border-left: 3px solid #f59e0b;
}

.top-card.silver {
  border-left: 3px solid #94a3b8;
}

.top-card.bronze {
  border-left: 3px solid #d97706;
}

.top-card .rank-badge {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 10px;
  color: #94a3b8;
}

.top-card.gold .rank-badge { color: #f59e0b; }
.top-card.silver .rank-badge { color: #6b7280; }
.top-card.bronze .rank-badge { color: #d97706; }

.top-card .team-info {
  margin-bottom: 16px;
}

.top-card .team-name {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
}

.top-card .honor-stats {
  margin-bottom: 12px;
}

.top-card .stat-row {
  display: flex;
  justify-content: space-around;
}

.top-card .stat-item {
  text-align: center;
}

.top-card .stat-value {
  display: block;
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.top-card .stat-label {
  font-size: 11px;
  color: #94a3b8;
}

.top-card .view-detail {
  font-size: 11px;
  color: #cbd5e1;
}

.table-section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
  background: #ffffff;
}

.ranking-row {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  border-bottom: 1px solid #f8fafc;
  cursor: pointer;
  transition: all 0.2s ease;
}

.ranking-row:hover {
  background: #f8fafc;
  padding-left: 24px;
}

.ranking-row:last-child {
  border-bottom: none;
}

.ranking-row .rank-num {
  width: 50px;
  font-size: 14px;
  font-weight: 600;
  color: #94a3b8;
}

.ranking-row .team-info {
  flex: 1;
}

.ranking-row .team-info .name {
  font-size: 14px;
  font-weight: 500;
  color: #0f172a;
}

.ranking-row .stats {
  display: flex;
  gap: 16px;
  margin-right: 16px;
}

.ranking-row .stats .stat {
  font-size: 13px;
  color: #64748b;
}

.ranking-row .action {
  width: 70px;
  text-align: right;
}

.back-btn {
  padding: 5px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #ffffff;
  color: #475569;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.back-btn:hover {
  border-color: #6366f1;
  color: #6366f1;
  background: #f5f3ff;
}
</style>
