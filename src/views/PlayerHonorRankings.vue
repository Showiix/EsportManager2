<template>
  <div class="player-honor-rankings">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1><el-icon><User /></el-icon> 选手荣誉榜</h1>
      <p>PLAYER HONOR RANKINGS</p>
    </div>

    <!-- 筛选和排序 -->
    <div class="filter-bar">
      <div class="sort-options">
        <span class="label">排序:</span>
        <el-radio-group v-model="sortBy" size="small">
          <el-radio-button value="champion"><el-icon><Trophy /></el-icon> 冠军数</el-radio-button>
          <el-radio-button value="mvp"><el-icon><Medal /></el-icon> MVP数</el-radio-button>
          <el-radio-button value="total">总荣誉</el-radio-button>
        </el-radio-group>
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
              <span class="stat-label">🌍国际</span>
            </div>
          </div>
          <div class="total-honors">
            总荣誉: {{ player.champion_count + player.mvp_count }}
          </div>
          <div class="view-detail">查看详情 →</div>
        </div>
      </div>

      <!-- 其他排名列表 -->
      <el-card class="rankings-table-card">
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
              <span class="stat">🌍{{ player.international_champion_count }}</span>
            </div>
            <div class="total">总: {{ player.champion_count + player.mvp_count }}</div>
            <div class="action">
              <el-button size="small" type="primary" text>详情 →</el-button>
            </div>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Trophy, Medal, User } from '@element-plus/icons-vue'
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
  if (rank === 1) return '🥇 #1'
  if (rank === 2) return '🥈 #2'
  if (rank === 3) return '🥉 #3'
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

<style scoped lang="scss">
.player-honor-rankings {
  padding: 20px;
  background: #ffffff;
  min-height: 100vh;
}

.page-header {
  text-align: center;
  padding: 30px 0;
  margin-bottom: 20px;

  h1 {
    font-size: 28px;
    color: #303133;
    margin-bottom: 8px;
  }

  p {
    font-size: 14px;
    color: #909399;
    letter-spacing: 2px;
  }
}

.filter-bar {
  display: flex;
  justify-content: center;
  margin-bottom: 30px;

  .label {
    color: #606266;
    margin-right: 10px;
    line-height: 32px;
  }

  :deep(.el-radio-button__inner) {
    background: #f5f7fa;
    border-color: #dcdfe6;
    color: #606266;
  }

  :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
    background: linear-gradient(135deg, #f59e0b, #d97706);
    border-color: #f59e0b;
    color: #fff;
  }
}

.loading-container {
  padding: 40px;
}

.rankings-list {
  max-width: 1000px;
  margin: 0 auto;
}

.top-three {
  display: flex;
  justify-content: center;
  gap: 20px;
  margin-bottom: 30px;
}

.top-card {
  width: 280px;
  padding: 25px;
  background: #f5f7fa;
  border-radius: 16px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
  border: 2px solid transparent;

  &:hover {
    transform: translateY(-5px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
  }

  &.gold {
    border-color: #ffd700;
    background: linear-gradient(135deg, #fffbeb, #fef3c7);
  }

  &.silver {
    border-color: #c0c0c0;
    background: linear-gradient(135deg, #f9fafb, #f3f4f6);
  }

  &.bronze {
    border-color: #cd7f32;
    background: linear-gradient(135deg, #fff7ed, #fed7aa);
  }

  .rank-badge {
    font-size: 24px;
    margin-bottom: 15px;
  }

  .player-info {
    margin-bottom: 20px;

    .player-name {
      font-size: 20px;
      font-weight: bold;
      color: #303133;
      margin-bottom: 5px;
    }

    .player-meta {
      font-size: 13px;
      color: #909399;
    }
  }

  .honor-stats {
    display: flex;
    justify-content: space-around;
    margin-bottom: 15px;

    .stat-item {
      text-align: center;

      .stat-value {
        display: block;
        font-size: 22px;
        font-weight: bold;
        color: #303133;
      }

      .stat-label {
        font-size: 12px;
        color: #909399;
      }
    }
  }

  .total-honors {
    font-size: 14px;
    color: #d97706;
    font-weight: 600;
    margin-bottom: 10px;
  }

  .view-detail {
    font-size: 12px;
    color: #909399;
  }
}

.rankings-table-card {
  background: #ffffff;
  border: 1px solid #ebeef5;

  :deep(.el-card__body) {
    padding: 0;
  }
}

.table-list {
  .ranking-row {
    display: flex;
    align-items: center;
    padding: 15px 20px;
    border-bottom: 1px solid #ebeef5;
    cursor: pointer;
    transition: background 0.2s;

    &:hover {
      background: #f5f7fa;
    }

    &:last-child {
      border-bottom: none;
    }

    .rank-num {
      width: 60px;
      font-size: 16px;
      font-weight: bold;
      color: #909399;
    }

    .player-info {
      flex: 1;

      .name {
        font-size: 15px;
        font-weight: 500;
        color: #303133;
        margin-right: 10px;
      }

      .meta {
        font-size: 12px;
        color: #909399;
      }
    }

    .stats {
      display: flex;
      gap: 15px;
      margin-right: 20px;

      .stat {
        font-size: 14px;
        color: #606266;
      }
    }

    .total {
      width: 80px;
      text-align: center;
      font-size: 14px;
      color: #d97706;
      font-weight: 600;
    }

    .action {
      width: 80px;
      text-align: right;
    }
  }
}
</style>
