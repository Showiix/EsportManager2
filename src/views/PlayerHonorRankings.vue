<template>
  <div class="player-honor-rankings">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>选手荣誉榜</h1>
        <p>查看所有选手的荣誉记录与排名</p>
      </div>
      <button class="back-btn" @click="router.push('/honors')">← 返回荣誉殿堂</button>
    </div>

    <!-- 筛选和排序 -->
    <div class="filter-section">
      <div class="filter-row">
        <div class="filter-group">
          <label>排序</label>
          <el-radio-group v-model="sortBy" size="small">
            <el-radio-button value="champion">冠军数</el-radio-button>
            <el-radio-button value="mvp">MVP数</el-radio-button>
            <el-radio-button value="total">总荣誉</el-radio-button>
          </el-radio-group>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <!-- 空状态 -->
    <el-empty v-else-if="rankings.length === 0" description="暂无选手荣誉记录" />

    <!-- 排行榜列表 -->
    <div v-else class="rankings-list">
      <!-- 前三名大卡片 -->
      <div class="top-three">
        <div
          v-for="player in topThree"
          :key="player.player_id"
          class="top-card"
          :class="getRankClass(player.rank)"
          @click="goToDetail(player.player_id)"
        >
          <div class="rank-badge">
            {{ getRankBadge(player.rank) }}
          </div>
          <div class="player-info">
            <div class="player-name">{{ player.player_name }}</div>
            <div class="player-meta">
              <span v-if="player.team_name">{{ player.team_name }}</span>
              <span v-if="player.position"> · {{ player.position }}</span>
            </div>
          </div>
          <div class="honor-stats">
            <div class="stat-item">
              <span class="stat-value">{{ player.champion_count }}</span>
              <span class="stat-label"><el-icon><Trophy /></el-icon>冠军</span>
            </div>
            <div class="stat-item">
              <span class="stat-value">{{ player.mvp_count }}</span>
              <span class="stat-label"><el-icon><Medal /></el-icon>MVP</span>
            </div>
            <div class="stat-item">
              <span class="stat-value">{{ player.international_champion_count }}</span>
              <span class="stat-label"><el-icon><Promotion /></el-icon>国际</span>
            </div>
          </div>
          <div class="total-honors">
            总荣誉: {{ player.champion_count + player.mvp_count }}
          </div>
          <div class="view-detail">查看详情 →</div>
        </div>
      </div>

      <!-- 其他排名列表 -->
      <div class="table-section">
        <div class="table-list">
          <div
            v-for="player in restRankings"
            :key="player.player_id"
            class="ranking-row"
            @click="goToDetail(player.player_id)"
          >
            <div class="rank-num">#{{ player.rank }}</div>
            <div class="player-info">
              <span class="name">{{ player.player_name }}</span>
              <span class="meta">
                {{ player.team_name || '-' }} · {{ player.position || '-' }}
              </span>
            </div>
            <div class="stats">
              <span class="stat"><el-icon><Trophy /></el-icon>{{ player.champion_count }}</span>
              <span class="stat"><el-icon><Medal /></el-icon>{{ player.mvp_count }}</span>
              <span class="stat"><el-icon><Promotion /></el-icon>{{ player.international_champion_count }}</span>
            </div>
            <div class="total">总: {{ player.champion_count + player.mvp_count }}</div>
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
import { Trophy, Medal, Promotion } from '@element-plus/icons-vue'
import { tauriApi, type PlayerHonorRanking } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('PlayerHonorRankings')

const router = useRouter()
const loading = ref(true)
const rankings = ref<PlayerHonorRanking[]>([])
const sortBy = ref('champion')

// 加载排行榜
const loadRankings = async () => {
  loading.value = true
  try {
    const res = await tauriApi.honor.getPlayerHonorRankings(100)
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
    } else if (sortBy.value === 'mvp') {
      return b.mvp_count - a.mvp_count
    } else {
      return (b.champion_count + b.mvp_count) - (a.champion_count + a.mvp_count)
    }
  })
  // 更新排名
  rankings.value.forEach((p, idx) => {
    p.rank = idx + 1
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
const goToDetail = (playerId: number) => {
  router.push(`/players/${playerId}`)
}

onMounted(() => {
  loadRankings()
})
</script>

<style scoped>
.player-honor-rankings {
  padding: 0;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.page-header p {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
}

.filter-section {
  margin-bottom: 16px;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-group label {
  font-size: 13px;
  color: #64748b;
  font-weight: 500;
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
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
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

.rank-badge {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: 12px;
  color: #94a3b8;
}

.top-card.gold .rank-badge { color: #f59e0b; }
.top-card.silver .rank-badge { color: #6b7280; }
.top-card.bronze .rank-badge { color: #d97706; }

.top-card .player-info {
  margin-bottom: 16px;
}

.player-name {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 4px;
}

.player-meta {
  font-size: 12px;
  color: #94a3b8;
}

.honor-stats {
  display: flex;
  justify-content: space-around;
  margin-bottom: 12px;
}

.stat-item {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.stat-label {
  font-size: 11px;
  color: #94a3b8;
}

.total-honors {
  font-size: 13px;
  color: #f59e0b;
  font-weight: 600;
  margin-bottom: 8px;
}

.view-detail {
  font-size: 11px;
  color: #cbd5e1;
}

.table-section {
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
}

.ranking-row {
  padding: 12px 20px;
  border-bottom: 1px solid #f8fafc;
  display: flex;
  align-items: center;
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

.rank-num {
  width: 50px;
  font-size: 14px;
  font-weight: 600;
  color: #94a3b8;
}

.ranking-row .player-info {
  flex: 1;
}

.ranking-row .player-info .name {
  font-size: 14px;
  font-weight: 600;
  color: #0f172a;
  margin-right: 8px;
}

.ranking-row .player-info .meta {
  font-size: 12px;
  color: #94a3b8;
}

.ranking-row .stats {
  display: flex;
  gap: 12px;
  margin-right: 16px;
}

.ranking-row .stats .stat {
  font-size: 13px;
  color: #64748b;
}

.ranking-row .total {
  width: 80px;
  text-align: center;
  font-size: 13px;
  color: #f59e0b;
  font-weight: 600;
}

.ranking-row .action {
  width: 80px;
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
