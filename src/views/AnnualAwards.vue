<template>
  <div class="annual-awards">
    <!-- 页面头部 -->
    <div class="ceremony-header">
      <div class="header-decoration">
        <div class="confetti" v-for="i in 20" :key="i" :style="getConfettiStyle(i)"></div>
      </div>
      <div class="header-content">
        <div class="season-badge">第 {{ selectedSeason }} 赛季</div>
        <h1 class="ceremony-title">年度颁奖典礼</h1>
        <p class="ceremony-subtitle">表彰本赛季最出色的选手们</p>
        <div class="season-selector-wrapper">
          <SeasonSelector v-model="selectedSeason" />
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <!-- 主要内容 -->
    <template v-else-if="awardsData">
      <!-- 年度MVP -->
      <section class="awards-section mvp-section">
        <div class="section-header">
          <span class="section-icon">👑</span>
          <h2>年度最有价值选手</h2>
          <span class="section-subtitle">ANNUAL MVP</span>
        </div>

        <div class="mvp-card" v-if="mvpPlayer" @click="goToPlayer(mvpPlayer.player_id)">
          <div class="mvp-spotlight"></div>
          <div class="mvp-badge">MVP</div>
          <div class="mvp-content">
            <div class="mvp-name">{{ mvpPlayer.player_name }}</div>
            <div class="mvp-meta">
              <el-tag :type="getPositionTagType(mvpPlayer.position)" effect="dark">
                {{ getPositionName(mvpPlayer.position) }}
              </el-tag>
              <span class="mvp-team">{{ mvpPlayer.team_name }}</span>
            </div>
            <div class="mvp-stats">
              <div class="stat-item">
                <span class="stat-value">{{ mvpPlayer.yearly_score.toFixed(1) }}</span>
                <span class="stat-label">年度得分</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ mvpPlayer.avg_impact.toFixed(1) }}</span>
                <span class="stat-label">场均影响力</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ mvpPlayer.games_played }}</span>
                <span class="stat-label">出场次数</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 年度Top20 -->
      <section class="awards-section top20-section">
        <div class="section-header">
          <span class="section-icon">🌟</span>
          <h2>年度Top20选手</h2>
          <span class="section-subtitle">ANNUAL TOP 20</span>
        </div>

        <div class="top20-grid">
          <div
            v-for="(player, index) in awardsData.top20"
            :key="player.player_id"
            class="top20-card"
            :class="{ 'is-mvp': index === 0, 'is-top3': index < 3, 'is-top10': index < 10 }"
            :style="{ '--delay': `${index * 0.05}s` }"
            @click="goToPlayer(player.player_id)"
          >
            <div class="rank-badge" :class="getRankClass(index)">
              {{ index === 0 ? '👑' : index + 1 }}
            </div>
            <div class="player-info">
              <div class="player-name">{{ player.player_name }}</div>
              <div class="player-meta">
                <el-tag :type="getPositionTagType(player.position)" size="small">
                  {{ getPositionName(player.position) }}
                </el-tag>
                <span class="team-name">{{ player.team_name }}</span>
              </div>
            </div>
            <div class="player-score">
              <span class="score-value">{{ player.yearly_score.toFixed(1) }}</span>
              <span class="score-label">得分</span>
            </div>
          </div>
        </div>
      </section>

      <!-- 年度最佳阵容 -->
      <section class="awards-section allpro-section">
        <div class="section-header">
          <span class="section-icon">🏅</span>
          <h2>年度最佳阵容</h2>
          <span class="section-subtitle">ALL-PRO TEAM</span>
        </div>

        <div class="allpro-container">
          <div
            v-for="player in sortedAllPro"
            :key="player.player_id"
            class="allpro-card"
            :class="player.position.toLowerCase()"
            @click="goToPlayer(player.player_id)"
          >
            <div class="position-icon">{{ getPositionIcon(player.position) }}</div>
            <div class="position-label">{{ getPositionName(player.position) }}</div>
            <div class="player-name">{{ player.player_name }}</div>
            <div class="team-name">{{ player.team_name }}</div>
            <div class="player-stats">
              <span class="score">{{ player.yearly_score.toFixed(1) }}</span>
              <span class="games">{{ player.games_played }}场</span>
            </div>
          </div>
        </div>
      </section>

      <!-- 年度最佳新秀 -->
      <section class="awards-section rookie-section" v-if="awardsData.rookie_of_the_year">
        <div class="section-header">
          <span class="section-icon">🌱</span>
          <h2>年度最佳新秀</h2>
          <span class="section-subtitle">ROOKIE OF THE YEAR</span>
        </div>

        <div class="rookie-card" @click="goToPlayer(awardsData.rookie_of_the_year!.player_id)">
          <div class="rookie-badge">ROOKIE</div>
          <div class="rookie-content">
            <div class="rookie-name">{{ awardsData.rookie_of_the_year.player_name }}</div>
            <div class="rookie-meta">
              <el-tag :type="getPositionTagType(awardsData.rookie_of_the_year.position)" effect="dark">
                {{ getPositionName(awardsData.rookie_of_the_year.position) }}
              </el-tag>
              <span class="rookie-team">{{ awardsData.rookie_of_the_year.team_name }}</span>
              <el-tag type="success" effect="plain">{{ awardsData.rookie_of_the_year.age }}岁</el-tag>
            </div>
            <div class="rookie-stats">
              <div class="stat-item">
                <span class="stat-value">{{ awardsData.rookie_of_the_year.yearly_score.toFixed(1) }}</span>
                <span class="stat-label">年度得分</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ awardsData.rookie_of_the_year.avg_impact.toFixed(1) }}</span>
                <span class="stat-label">场均影响力</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ awardsData.rookie_of_the_year.games_played }}</span>
                <span class="stat-label">出场次数</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 已颁发提示 -->
      <el-alert
        v-if="awardsData.already_awarded"
        title="本赛季年度荣誉已颁发"
        type="success"
        show-icon
        :closable="false"
        class="awarded-alert"
      />

      <!-- 返回按钮 -->
      <div class="action-footer">
        <el-button type="primary" size="large" @click="goBack">
          返回时间控制面板
        </el-button>
      </div>
    </template>

    <!-- 无数据 -->
    <el-empty v-else description="暂无颁奖数据">
      <el-button type="primary" @click="fetchAwardsData">重新加载</el-button>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSeasonStore } from '@/stores/useSeasonStore'
import { tauriApi } from '@/api/tauri'
import type { AnnualAwardsData } from '@/api/tauri'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const logger = createLogger('AnnualAwards')

const router = useRouter()
const seasonStore = useSeasonStore()

const loading = ref(false)
const awardsData = ref<AnnualAwardsData | null>(null)
const selectedSeason = ref(seasonStore.currentSeason)

// MVP选手 (Top20的第一名)
const mvpPlayer = computed(() => {
  return awardsData.value?.top20[0] || null
})

// 排序后的最佳阵容 (按位置排序: TOP, JUG, MID, ADC, SUP)
const sortedAllPro = computed(() => {
  if (!awardsData.value?.all_pro_team) return []
  const positionOrder: Record<string, number> = { TOP: 0, JUG: 1, MID: 2, ADC: 3, SUP: 4 }
  return [...awardsData.value.all_pro_team].sort((a, b) => {
    return (positionOrder[a.position] ?? 99) - (positionOrder[b.position] ?? 99)
  })
})

// 获取颁奖数据
const fetchAwardsData = async (seasonId?: number) => {
  loading.value = true
  try {
    awardsData.value = await tauriApi.awards.getAnnualAwardsData(seasonId)
  } catch (error) {
    logger.error('获取颁奖数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 跳转到选手详情
const goToPlayer = (playerId: number) => {
  router.push(`/data-center/player/${playerId}?season=S${selectedSeason.value}`)
}

// 返回
const goBack = () => {
  router.push('/time')
}

// 获取位置名称
const getPositionName = (position: string): string => {
  return POSITION_NAMES[position as PlayerPosition] || position
}

// 获取位置标签类型
const getPositionTagType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'warning',
    MID: 'primary',
    ADC: 'success',
    SUP: 'info'
  }
  return types[position] || 'info'
}

// 获取位置图标
const getPositionIcon = (position: string) => {
  const icons: Record<string, string> = {
    TOP: '🗡️',
    JUG: '🌲',
    MID: '⚡',
    ADC: '🏹',
    SUP: '🛡️'
  }
  return icons[position] || '👤'
}

// 获取排名样式类
const getRankClass = (index: number) => {
  if (index === 0) return 'gold'
  if (index === 1) return 'silver'
  if (index === 2) return 'bronze'
  return ''
}

// 彩带样式
const getConfettiStyle = (index: number) => {
  const colors = ['#ffd700', '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ffeaa7', '#dfe6e9']
  return {
    left: `${Math.random() * 100}%`,
    animationDelay: `${Math.random() * 3}s`,
    backgroundColor: colors[index % colors.length]
  }
}

onMounted(() => {
  fetchAwardsData(selectedSeason.value)
})

// 赛季切换时重新加载数据
watch(selectedSeason, (newSeason) => {
  fetchAwardsData(newSeason)
})
</script>

<style scoped lang="scss">
.annual-awards {
  min-height: 100vh;
  background: linear-gradient(180deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  padding: 0;
}

// 头部
.ceremony-header {
  position: relative;
  padding: 60px 40px;
  text-align: center;
  overflow: hidden;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.3) 0%, rgba(118, 75, 162, 0.3) 100%);

  .header-decoration {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .confetti {
    position: absolute;
    width: 10px;
    height: 10px;
    border-radius: 2px;
    animation: confetti-fall 5s linear infinite;
    opacity: 0.8;
  }

  @keyframes confetti-fall {
    0% { transform: translateY(-100px) rotate(0deg); opacity: 1; }
    100% { transform: translateY(100vh) rotate(720deg); opacity: 0; }
  }

  .header-content {
    position: relative;
    z-index: 1;
  }

  .season-badge {
    display: inline-block;
    padding: 8px 24px;
    background: rgba(255, 215, 0, 0.2);
    border: 2px solid #ffd700;
    border-radius: 30px;
    color: #ffd700;
    font-weight: 600;
    margin-bottom: 16px;
  }

  .ceremony-title {
    font-size: 48px;
    font-weight: 800;
    color: white;
    margin: 0 0 12px 0;
    text-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .ceremony-subtitle {
    font-size: 18px;
    color: rgba(255, 255, 255, 0.8);
    margin: 0;
  }

  .season-selector-wrapper {
    margin-top: 16px;
    display: flex;
    justify-content: center;
  }
}

// 加载状态
.loading-container {
  padding: 40px;
  background: rgba(255, 255, 255, 0.05);
  margin: 20px;
  border-radius: 16px;
}

// 通用section样式
.awards-section {
  padding: 40px;
  margin: 20px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  backdrop-filter: blur(10px);

  .section-header {
    text-align: center;
    margin-bottom: 32px;

    .section-icon {
      font-size: 48px;
      display: block;
      margin-bottom: 12px;
    }

    h2 {
      font-size: 28px;
      font-weight: 700;
      color: white;
      margin: 0 0 8px 0;
    }

    .section-subtitle {
      font-size: 14px;
      color: rgba(255, 255, 255, 0.5);
      letter-spacing: 4px;
    }
  }
}

// MVP卡片
.mvp-section {
  background: linear-gradient(135deg, rgba(255, 215, 0, 0.15) 0%, rgba(255, 165, 0, 0.1) 100%);
}

.mvp-card {
  position: relative;
  max-width: 600px;
  margin: 0 auto;
  padding: 40px;
  background: linear-gradient(135deg, #1a1a2e 0%, #2d2d44 100%);
  border: 3px solid #ffd700;
  border-radius: 20px;
  cursor: pointer;
  overflow: hidden;
  transition: transform 0.3s, box-shadow 0.3s;

  &:hover {
    transform: translateY(-8px);
    box-shadow: 0 20px 40px rgba(255, 215, 0, 0.3);
  }

  .mvp-spotlight {
    position: absolute;
    top: -100px;
    left: 50%;
    transform: translateX(-50%);
    width: 200px;
    height: 200px;
    background: radial-gradient(circle, rgba(255, 215, 0, 0.3) 0%, transparent 70%);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.5; transform: translateX(-50%) scale(1); }
    50% { opacity: 1; transform: translateX(-50%) scale(1.2); }
  }

  .mvp-badge {
    position: absolute;
    top: 20px;
    right: 20px;
    padding: 8px 20px;
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: #1a1a2e;
    font-weight: 800;
    font-size: 14px;
    border-radius: 20px;
  }

  .mvp-content {
    position: relative;
    z-index: 1;
    text-align: center;
  }

  .mvp-name {
    font-size: 36px;
    font-weight: 800;
    color: #ffd700;
    margin-bottom: 16px;
  }

  .mvp-meta {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 12px;
    margin-bottom: 24px;

    .mvp-team {
      color: rgba(255, 255, 255, 0.8);
      font-size: 16px;
    }
  }

  .mvp-stats {
    display: flex;
    justify-content: center;
    gap: 40px;

    .stat-item {
      text-align: center;

      .stat-value {
        display: block;
        font-size: 28px;
        font-weight: 700;
        color: white;
      }

      .stat-label {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.6);
      }
    }
  }
}

// Top20网格
.top20-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.top20-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s;
  animation: fade-in 0.5s ease-out backwards;
  animation-delay: var(--delay);

  &:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: translateX(8px);
  }

  &.is-mvp {
    background: linear-gradient(135deg, rgba(255, 215, 0, 0.2) 0%, rgba(255, 165, 0, 0.1) 100%);
    border: 1px solid rgba(255, 215, 0, 0.3);
  }

  &.is-top3:not(.is-mvp) {
    background: rgba(192, 192, 192, 0.1);
    border: 1px solid rgba(192, 192, 192, 0.2);
  }

  &.is-top10:not(.is-top3) {
    background: rgba(102, 126, 234, 0.1);
  }

  .rank-badge {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-weight: 700;
    font-size: 16px;
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.8);

    &.gold {
      background: linear-gradient(135deg, #ffd700, #ff8c00);
      color: #1a1a2e;
      font-size: 20px;
    }

    &.silver {
      background: linear-gradient(135deg, #c0c0c0, #a8a8a8);
      color: #1a1a2e;
    }

    &.bronze {
      background: linear-gradient(135deg, #cd7f32, #b8860b);
      color: #1a1a2e;
    }
  }

  .player-info {
    flex: 1;
    min-width: 0;

    .player-name {
      font-size: 16px;
      font-weight: 600;
      color: white;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .player-meta {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-top: 4px;

      .team-name {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.6);
      }
    }
  }

  .player-score {
    text-align: right;

    .score-value {
      display: block;
      font-size: 20px;
      font-weight: 700;
      color: #ffd700;
    }

    .score-label {
      font-size: 10px;
      color: rgba(255, 255, 255, 0.5);
    }
  }
}

@keyframes fade-in {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

// 最佳阵容
.allpro-container {
  display: flex;
  justify-content: center;
  gap: 20px;
  flex-wrap: wrap;
}

.allpro-card {
  width: 160px;
  padding: 24px 16px;
  text-align: center;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.3s;

  &:hover {
    transform: translateY(-8px);
  }

  &.top { border-top: 3px solid #ef4444; }
  &.jug { border-top: 3px solid #f59e0b; }
  &.mid { border-top: 3px solid #3b82f6; }
  &.adc { border-top: 3px solid #10b981; }
  &.sup { border-top: 3px solid #8b5cf6; }

  .position-icon {
    font-size: 32px;
    margin-bottom: 8px;
  }

  .position-label {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: 12px;
    letter-spacing: 2px;
  }

  .player-name {
    font-size: 16px;
    font-weight: 700;
    color: white;
    margin-bottom: 4px;
  }

  .team-name {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    margin-bottom: 12px;
  }

  .player-stats {
    display: flex;
    justify-content: center;
    gap: 12px;
    font-size: 12px;

    .score {
      color: #ffd700;
      font-weight: 600;
    }

    .games {
      color: rgba(255, 255, 255, 0.5);
    }
  }
}

// 新秀卡片
.rookie-section {
  background: linear-gradient(135deg, rgba(46, 213, 115, 0.15) 0%, rgba(39, 174, 96, 0.1) 100%);
}

.rookie-card {
  position: relative;
  max-width: 500px;
  margin: 0 auto;
  padding: 32px;
  background: linear-gradient(135deg, #1a1a2e 0%, #2d2d44 100%);
  border: 2px solid #2ed573;
  border-radius: 16px;
  cursor: pointer;
  overflow: hidden;
  transition: transform 0.3s, box-shadow 0.3s;

  &:hover {
    transform: translateY(-8px);
    box-shadow: 0 16px 32px rgba(46, 213, 115, 0.3);
  }

  .rookie-badge {
    position: absolute;
    top: 16px;
    right: 16px;
    padding: 6px 16px;
    background: linear-gradient(135deg, #2ed573, #27ae60);
    color: white;
    font-weight: 700;
    font-size: 12px;
    border-radius: 16px;
  }

  .rookie-content {
    text-align: center;
  }

  .rookie-name {
    font-size: 28px;
    font-weight: 800;
    color: #2ed573;
    margin-bottom: 12px;
  }

  .rookie-meta {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;

    .rookie-team {
      color: rgba(255, 255, 255, 0.8);
    }
  }

  .rookie-stats {
    display: flex;
    justify-content: center;
    gap: 32px;

    .stat-item {
      text-align: center;

      .stat-value {
        display: block;
        font-size: 24px;
        font-weight: 700;
        color: white;
      }

      .stat-label {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.6);
      }
    }
  }
}

// 已颁发提示
.awarded-alert {
  margin: 20px;
}

// 底部操作
.action-footer {
  padding: 40px;
  text-align: center;
}
</style>
