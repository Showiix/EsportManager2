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
        <div class="season-selector-wrapper" v-if="!isAwardsPhase || ceremonyComplete">
          <SeasonSelector v-model="selectedSeason" />
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <!-- 空状态：非颁奖阶段且无数据 -->
    <div v-else-if="!awardsData && !isAwardsPhase" class="empty-state">
      <div class="empty-icon"></div>
      <h2>颁奖典礼尚未开始</h2>
      <p>请推进赛季至年度颁奖阶段</p>
    </div>

    <!-- 颁奖阶段：开始按钮 -->
    <div v-else-if="isAwardsPhase && !ceremonyStarted && !awardsData" class="start-ceremony">
      <div class="start-icon"></div>
      <h2>年度颁奖典礼已就绪</h2>
      <p>点击下方按钮开始颁奖</p>
      <el-button type="warning" size="large" :loading="starting" @click="startCeremony">
        开始颁奖典礼
      </el-button>
    </div>

    <!-- 典礼内容 -->
    <template v-else-if="awardsData">
      <!-- 典礼阶段1: Top20 逐个揭晓 -->
      <section
        v-if="currentSection === 'top20' || ceremonyComplete"
        class="awards-section top20-section"
      >
        <div class="section-header">
          <h2>年度Top20选手</h2>
          <span class="section-subtitle">ANNUAL TOP 20</span>
        </div>

        <div class="top20-list">
          <TransitionGroup name="reveal">
            <div
              v-for="player in visibleTop20"
              :key="player.player_id"
              class="top20-card"
              :class="{
                'is-mvp': player.rank === 1,
                'is-top3': player.rank <= 3,
                'is-top10': player.rank <= 10,
              }"
              @click="goToPlayer(player.player_id)"
            >
              <div class="rank-badge" :class="getRankClass(player.rank)">
                {{ player.rank }}
              </div>
              <div class="player-info">
                <div class="player-name">{{ player.player_name }}</div>
                <div class="player-meta">
                  <el-tag :type="getPositionTagType(player.position)" size="small">
                    {{ getPositionName(player.position) }}
                  </el-tag>
                  <span class="team-name">{{ player.team_name }}</span>
                  <span class="age-tag" v-if="player.age <= 20">{{ player.age }}岁</span>
                </div>
              </div>
              <div class="player-dims">
                <div class="dim-mini" v-for="dim in getDimBars(player)" :key="dim.label">
                  <div class="dim-mini-bar"><div class="dim-mini-fill" :style="{ width: dim.value + '%', background: dim.color }"></div></div>
                </div>
              </div>
              <div class="player-score">
                <span class="score-value">{{ player.yearly_score.toFixed(1) }}</span>
                <span class="score-label">得分</span>
              </div>
              <div class="player-tags" v-if="player.commentary.tags.length > 0">
                <span class="tag-item" v-for="tag in player.commentary.tags.slice(0, 2)" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </TransitionGroup>
        </div>

        <!-- 揭晓按钮 -->
        <div class="reveal-action" v-if="currentSection === 'top20' && !ceremonyComplete">
          <el-button
            v-if="revealedCount < 20"
            type="warning"
            size="large"
            @click="revealNext"
          >
            揭晓 #{{ 20 - revealedCount }} {{ revealedCount >= 19 ? '— 年度MVP' : '' }}
          </el-button>
          <el-button
            v-if="revealedCount > 0 && revealedCount < 20"
            type="default"
            @click="revealAllTop20"
          >
            全部揭晓
          </el-button>
          <el-button
            v-if="revealedCount >= 20"
            type="primary"
            size="large"
            @click="nextSection"
          >
            继续 — 最佳阵容
          </el-button>
        </div>
      </section>

      <!-- 典礼阶段2: 最佳阵容 -->
      <section
        v-if="(currentSection === 'allpro' || sectionPassed('allpro')) && (revealedTier > 0 || ceremonyComplete)"
        class="awards-section allpro-section"
      >
        <div class="section-header">
          <h2>年度最佳阵容</h2>
          <span class="section-subtitle">ALL-PRO TEAMS</span>
        </div>

        <div class="allpro-tiers-reveal">
          <!-- 三阵 -->
          <Transition name="tier-reveal">
            <div class="tier-block" v-if="revealedTier >= 1 || ceremonyComplete">
              <div class="tier-label bronze">三阵</div>
              <div class="tier-players">
                <div
                  class="tier-player-card"
                  v-for="p in sortByPosition(awardsData.all_pro_3rd)"
                  :key="p.player_id"
                  @click="goToPlayer(p.player_id)"
                >
                  <span class="tp-pos">{{ getPositionName(p.position) }}</span>
                  <span class="tp-name">{{ p.player_name }}</span>
                  <span class="tp-team">{{ p.team_name }}</span>
                  <span class="tp-score">{{ p.yearly_score.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </Transition>

          <!-- 二阵 -->
          <Transition name="tier-reveal">
            <div class="tier-block" v-if="revealedTier >= 2 || ceremonyComplete">
              <div class="tier-label silver">二阵</div>
              <div class="tier-players">
                <div
                  class="tier-player-card"
                  v-for="p in sortByPosition(awardsData.all_pro_2nd)"
                  :key="p.player_id"
                  @click="goToPlayer(p.player_id)"
                >
                  <span class="tp-pos">{{ getPositionName(p.position) }}</span>
                  <span class="tp-name">{{ p.player_name }}</span>
                  <span class="tp-team">{{ p.team_name }}</span>
                  <span class="tp-score">{{ p.yearly_score.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </Transition>

          <!-- 一阵 -->
          <Transition name="tier-reveal">
            <div class="tier-block" v-if="revealedTier >= 3 || ceremonyComplete">
              <div class="tier-label gold">一阵</div>
              <div class="tier-players">
                <div
                  class="tier-player-card highlight"
                  v-for="p in sortByPosition(awardsData.all_pro_1st)"
                  :key="p.player_id"
                  @click="goToPlayer(p.player_id)"
                >
                  <span class="tp-pos">{{ getPositionName(p.position) }}</span>
                  <span class="tp-name">{{ p.player_name }}</span>
                  <span class="tp-team">{{ p.team_name }}</span>
                  <span class="tp-score">{{ p.yearly_score.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </Transition>
        </div>

        <!-- 揭晓按钮 -->
        <div class="reveal-action" v-if="currentSection === 'allpro' && !ceremonyComplete">
          <el-button
            v-if="revealedTier < 3"
            type="warning"
            size="large"
            @click="revealNextTier"
          >
            揭晓{{ revealedTier === 0 ? '三阵' : revealedTier === 1 ? '二阵' : '一阵' }}
          </el-button>
          <el-button
            v-if="revealedTier >= 3"
            type="primary"
            size="large"
            @click="nextSection"
          >
            继续 — 特别奖项
          </el-button>
        </div>
      </section>

      <!-- 典礼阶段3: 特别奖项 -->
      <section
        v-if="(currentSection === 'special' || sectionPassed('special')) && (revealedSpecial > 0 || ceremonyComplete)"
        class="awards-section special-section"
      >
        <div class="section-header">
          <h2>特别奖项</h2>
          <span class="section-subtitle">SPECIAL AWARDS</span>
        </div>

        <div class="special-cards">
          <!-- 最稳定选手 -->
          <Transition name="special-reveal">
            <div
              v-if="(revealedSpecial >= 1 || ceremonyComplete) && awardsData.most_consistent"
              class="special-card stable"
              @click="goToPlayer(awardsData.most_consistent.player_id)"
            >
              <div class="special-badge">最稳定选手</div>
              <div class="special-name">{{ awardsData.most_consistent.player_name }}</div>
              <div class="special-meta">
                {{ awardsData.most_consistent.team_name }} · {{ getPositionName(awardsData.most_consistent.position) }}
              </div>
              <div class="special-desc">{{ awardsData.most_consistent.commentary.description }}</div>
              <div class="special-tags">
                <span class="stag" v-for="tag in awardsData.most_consistent.commentary.tags" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </Transition>

          <!-- 最佳新秀 -->
          <Transition name="special-reveal">
            <div
              v-if="(revealedSpecial >= 2 || ceremonyComplete) && awardsData.rookie_of_the_year"
              class="special-card rookie"
              @click="goToPlayer(awardsData.rookie_of_the_year.player_id)"
            >
              <div class="special-badge">最佳新秀</div>
              <div class="special-name">{{ awardsData.rookie_of_the_year.player_name }}</div>
              <div class="special-meta">
                {{ awardsData.rookie_of_the_year.team_name }} · {{ getPositionName(awardsData.rookie_of_the_year.position) }} · {{ awardsData.rookie_of_the_year.age }}岁
              </div>
              <div class="special-desc">{{ awardsData.rookie_of_the_year.commentary.description }}</div>
              <div class="special-tags">
                <span class="stag" v-for="tag in awardsData.rookie_of_the_year.commentary.tags" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </Transition>

          <!-- 最具统治力 -->
          <Transition name="special-reveal">
            <div
              v-if="(revealedSpecial >= 3 || ceremonyComplete) && awardsData.most_dominant"
              class="special-card dominant"
              @click="goToPlayer(awardsData.most_dominant.player_id)"
            >
              <div class="special-badge">最具统治力</div>
              <div class="special-name">{{ awardsData.most_dominant.player_name }}</div>
              <div class="special-meta">
                {{ awardsData.most_dominant.team_name }} · {{ getPositionName(awardsData.most_dominant.position) }}
              </div>
              <div class="special-desc">{{ awardsData.most_dominant.commentary.description }}</div>
              <div class="special-tags">
                <span class="stag" v-for="tag in awardsData.most_dominant.commentary.tags" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </Transition>
        </div>

        <!-- 揭晓按钮 -->
        <div class="reveal-action" v-if="currentSection === 'special' && !ceremonyComplete">
          <el-button
            v-if="revealedSpecial < 3"
            type="warning"
            size="large"
            @click="revealNextSpecial"
          >
            揭晓{{ revealedSpecial === 0 ? '最稳定选手' : revealedSpecial === 1 ? '最佳新秀' : '最具统治力' }}
          </el-button>
          <el-button
            v-if="revealedSpecial >= 3"
            type="primary"
            size="large"
            @click="nextSection"
          >
            继续 — 年度MVP揭晓
          </el-button>
        </div>
      </section>

      <!-- 典礼阶段4: MVP 终极揭晓 -->
      <section
        v-if="currentSection === 'mvp' || ceremonyComplete"
        class="awards-section mvp-section"
      >
        <div class="section-header">
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
            <div class="mvp-commentary" v-if="mvpPlayer.commentary.description">
              <p>{{ mvpPlayer.commentary.description }}</p>
              <div class="mvp-tags">
                <span class="mtag" v-for="tag in mvpPlayer.commentary.tags" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="reveal-action" v-if="currentSection === 'mvp' && !ceremonyComplete">
          <el-button type="success" size="large" @click="completeCeremony">
            典礼完成
          </el-button>
        </div>
      </section>

      <!-- 已完成提示 -->
      <el-alert
        v-if="ceremonyComplete"
        title="本赛季年度颁奖典礼已完成"
        type="success"
        show-icon
        :closable="false"
        class="awarded-alert"
      />

      <!-- 返回 -->
      <div class="action-footer">
        <el-button type="primary" size="large" @click="goBack">
          返回时间控制面板
        </el-button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSeasonStore } from '@/stores/useSeasonStore'
import { useTimeStore } from '@/stores/useTimeStore'
import { awardsApi } from '@/api/tauri'
import type { AnnualAwardsData, Top20Player, AllProPlayer } from '@/api/tauri'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'
import SeasonSelector from '@/components/common/SeasonSelector.vue'

const logger = createLogger('AnnualAwards')

const router = useRouter()
const seasonStore = useSeasonStore()
const timeStore = useTimeStore()

const loading = ref(false)
const starting = ref(false)
const awardsData = ref<AnnualAwardsData | null>(null)
const selectedSeason = ref(seasonStore.currentSeason)

// 典礼状态
const ceremonyStarted = ref(false)
const ceremonyComplete = ref(false)
const revealedCount = ref(0)
const currentSection = ref<'top20' | 'allpro' | 'special' | 'mvp' | 'complete'>('top20')
const revealedTier = ref(0)
const revealedSpecial = ref(0)

// 是否在颁奖阶段
const isAwardsPhase = computed(() => timeStore.isAnnualAwardsPhase)

// MVP 选手
const mvpPlayer = computed(() => awardsData.value?.top20[0] || null)

// 可见的 Top20（从 #20 往前揭晓）
const visibleTop20 = computed(() => {
  if (ceremonyComplete.value) return awardsData.value?.top20 ?? []
  const all = awardsData.value?.top20 ?? []
  // 显示从 #20 开始的 revealedCount 个
  return all.slice(all.length - revealedCount.value).reverse()
    .sort((a, b) => a.rank - b.rank)
    // 实际上按 rank 从小到大展示已揭晓的
    .concat() // 展示顺序：rank 小在前
})

// 按位置排序
const sortByPosition = (players: AllProPlayer[]) => {
  const order: Record<string, number> = { TOP: 0, JUG: 1, MID: 2, ADC: 3, SUP: 4 }
  return [...players].sort((a, b) => (order[a.position] ?? 99) - (order[b.position] ?? 99))
}

// 判断某个阶段是否已过
const sectionPassed = (section: string) => {
  const order = ['top20', 'allpro', 'special', 'mvp', 'complete']
  const currentIdx = order.indexOf(currentSection.value)
  const targetIdx = order.indexOf(section)
  return ceremonyComplete.value || currentIdx > targetIdx
}

// 五维条
const getDimBars = (player: Top20Player) => [
  { label: '影响', value: player.dimensions.impact_norm, color: '#3b82f6' },
  { label: '发挥', value: player.dimensions.performance_norm, color: '#10b981' },
  { label: '稳定', value: player.dimensions.stability_norm, color: '#8b5cf6' },
  { label: '出场', value: player.dimensions.appearance_norm, color: '#f59e0b' },
  { label: '荣誉', value: player.dimensions.honor_norm, color: '#ef4444' },
]

// 获取颁奖数据
const fetchAwardsData = async (seasonId?: number) => {
  loading.value = true
  try {
    awardsData.value = await awardsApi.getAnnualAwardsData(seasonId)
    if (awardsData.value?.already_awarded) {
      ceremonyComplete.value = true
      ceremonyStarted.value = true
    }
  } catch (error) {
    logger.error('获取颁奖数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 开始颁奖典礼
const startCeremony = async () => {
  starting.value = true
  try {
    // 触发后端颁奖
    await timeStore.completeAndAdvance()
    // 拉取颁奖数据
    await fetchAwardsData(selectedSeason.value)
    ceremonyStarted.value = true
    ceremonyComplete.value = false
    currentSection.value = 'top20'
    revealedCount.value = 0
    revealedTier.value = 0
    revealedSpecial.value = 0
  } catch (error) {
    logger.error('开始颁奖失败:', error)
  } finally {
    starting.value = false
  }
}

// Top20 揭晓下一位
const revealNext = () => {
  if (revealedCount.value < 20) {
    revealedCount.value++
  }
}

// 全部揭晓 Top20
const revealAllTop20 = () => {
  revealedCount.value = 20
}

// 揭晓下一阵
const revealNextTier = () => {
  if (revealedTier.value < 3) {
    revealedTier.value++
  }
}

// 揭晓下一个特别奖
const revealNextSpecial = () => {
  if (revealedSpecial.value < 3) {
    revealedSpecial.value++
  }
}

// 下一个阶段
const nextSection = () => {
  const order: Array<typeof currentSection.value> = ['top20', 'allpro', 'special', 'mvp', 'complete']
  const idx = order.indexOf(currentSection.value)
  if (idx < order.length - 1) {
    currentSection.value = order[idx + 1]
  }
}

// 完成典礼
const completeCeremony = () => {
  ceremonyComplete.value = true
  currentSection.value = 'complete'
}

// 方法
const goToPlayer = (playerId: number) => {
  router.push(`/data-center/player/${playerId}?season=S${selectedSeason.value}`)
}

const goBack = () => {
  router.push('/time')
}

const getPositionName = (position: string): string => {
  return POSITION_NAMES[position as PlayerPosition] || position
}

const getPositionTagType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger', JUG: 'warning', MID: 'primary', ADC: 'success', SUP: 'info'
  }
  return types[position] || 'info'
}

const getRankClass = (rank: number) => {
  if (rank === 1) return 'gold'
  if (rank === 2) return 'silver'
  if (rank === 3) return 'bronze'
  return ''
}

const getConfettiStyle = (index: number) => {
  const colors = ['#ffd700', '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ffeaa7', '#dfe6e9']
  return {
    left: `${Math.random() * 100}%`,
    animationDelay: `${Math.random() * 3}s`,
    backgroundColor: colors[index % colors.length]
  }
}

onMounted(async () => {
  await timeStore.fetchTimeState()
  if (isAwardsPhase.value) {
    // 如果已经颁发过，直接拉取数据
    await fetchAwardsData(selectedSeason.value)
  } else {
    // 非颁奖阶段，尝试拉取历史数据
    await fetchAwardsData(selectedSeason.value)
  }
})

watch(selectedSeason, (newSeason) => {
  ceremonyComplete.value = false
  ceremonyStarted.value = false
  revealedCount.value = 0
  revealedTier.value = 0
  revealedSpecial.value = 0
  currentSection.value = 'top20'
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

// 加载
.loading-container {
  padding: 40px;
  background: rgba(255, 255, 255, 0.05);
  margin: 20px;
  border-radius: 16px;
}

// 空状态
.empty-state, .start-ceremony {
  padding: 80px 40px;
  text-align: center;

  .empty-icon, .start-icon {
    width: 80px; height: 80px; border-radius: 50%;
    margin: 0 auto 24px;
  }
  .empty-icon { background: rgba(255, 255, 255, 0.1); }
  .start-icon { background: linear-gradient(135deg, #ffd700, #ff8c00); }

  h2 { font-size: 24px; color: white; margin: 0 0 12px; }
  p { font-size: 16px; color: rgba(255, 255, 255, 0.6); margin: 0 0 24px; }
}

// 通用section
.awards-section {
  padding: 40px;
  margin: 20px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  backdrop-filter: blur(10px);

  .section-header {
    text-align: center;
    margin-bottom: 32px;

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

// Top20 列表
.top20-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.top20-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 20px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s;

  &:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: translateX(8px);
  }

  &.is-mvp {
    background: linear-gradient(135deg, rgba(255, 215, 0, 0.2) 0%, rgba(255, 165, 0, 0.1) 100%);
    border: 1px solid rgba(255, 215, 0, 0.3);
    padding: 18px 20px;
  }
  &.is-top3:not(.is-mvp) {
    background: rgba(192, 192, 192, 0.1);
    border: 1px solid rgba(192, 192, 192, 0.2);
  }
  &.is-top10:not(.is-top3) {
    background: rgba(102, 126, 234, 0.1);
  }

  .rank-badge {
    width: 40px; height: 40px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%;
    font-weight: 700; font-size: 16px;
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.8);
    flex-shrink: 0;

    &.gold { background: linear-gradient(135deg, #ffd700, #ff8c00); color: #1a1a2e; }
    &.silver { background: linear-gradient(135deg, #c0c0c0, #a8a8a8); color: #1a1a2e; }
    &.bronze { background: linear-gradient(135deg, #cd7f32, #b8860b); color: #1a1a2e; }
  }

  .player-info {
    flex: 1; min-width: 0;
    .player-name {
      font-size: 16px; font-weight: 600; color: white;
      white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    }
    .player-meta {
      display: flex; align-items: center; gap: 8px; margin-top: 4px;
      .team-name { font-size: 12px; color: rgba(255, 255, 255, 0.6); }
      .age-tag { font-size: 11px; color: #2ed573; }
    }
  }

  .player-dims {
    display: flex; gap: 3px; width: 120px; flex-shrink: 0;
    .dim-mini {
      flex: 1;
      .dim-mini-bar {
        height: 24px; background: rgba(255,255,255,0.1); border-radius: 3px; overflow: hidden;
        .dim-mini-fill { height: 100%; border-radius: 3px; }
      }
    }
  }

  .player-score {
    text-align: right; flex-shrink: 0; width: 60px;
    .score-value { display: block; font-size: 20px; font-weight: 700; color: #ffd700; }
    .score-label { font-size: 10px; color: rgba(255, 255, 255, 0.5); }
  }

  .player-tags {
    display: flex; gap: 4px; flex-shrink: 0;
    .tag-item {
      padding: 2px 8px; border-radius: 10px; font-size: 11px;
      background: rgba(255, 255, 255, 0.1); color: rgba(255, 255, 255, 0.7);
    }
  }
}

// 揭晓动画
.reveal-enter-active {
  transition: all 0.5s ease-out;
}
.reveal-enter-from {
  opacity: 0;
  transform: translateY(30px);
}

// 揭晓按钮
.reveal-action {
  text-align: center;
  margin-top: 32px;
  display: flex;
  justify-content: center;
  gap: 12px;
}

// 最佳阵容
.allpro-tiers-reveal {
  display: flex; flex-direction: column; gap: 24px;

  .tier-block {
    .tier-label {
      font-size: 16px; font-weight: 700; padding: 10px 20px;
      border-radius: 10px; margin-bottom: 12px; color: white; text-align: center;
      &.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); }
      &.silver { background: linear-gradient(135deg, #94a3b8, #64748b); }
      &.bronze { background: linear-gradient(135deg, #cd7f32, #a0522d); }
    }

    .tier-players {
      display: flex; flex-wrap: wrap; gap: 8px; justify-content: center;

      .tier-player-card {
        display: flex; align-items: center; gap: 12px;
        padding: 12px 20px; border-radius: 10px;
        background: rgba(255, 255, 255, 0.05);
        cursor: pointer; transition: all 0.2s;
        &:hover { background: rgba(255, 255, 255, 0.1); }
        &.highlight { border: 1px solid rgba(255, 215, 0, 0.3); }

        .tp-pos { font-size: 12px; color: rgba(255, 255, 255, 0.5); width: 40px; }
        .tp-name { font-weight: 600; color: white; flex: 1; min-width: 80px; }
        .tp-team { font-size: 13px; color: rgba(255, 255, 255, 0.6); }
        .tp-score { font-weight: 700; color: #ffd700; }
      }
    }
  }
}

.tier-reveal-enter-active { transition: all 0.6s ease-out; }
.tier-reveal-enter-from { opacity: 0; transform: translateY(40px); }

// 特别奖项
.special-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
}

.special-card {
  padding: 24px; border-radius: 16px; cursor: pointer;
  transition: all 0.3s;

  &:hover { transform: translateY(-4px); }
  &.stable {
    background: linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(109, 40, 217, 0.1));
    border: 1px solid rgba(139, 92, 246, 0.3);
  }
  &.rookie {
    background: linear-gradient(135deg, rgba(46, 213, 115, 0.2), rgba(39, 174, 96, 0.1));
    border: 1px solid rgba(46, 213, 115, 0.3);
  }
  &.dominant {
    background: linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(220, 38, 38, 0.1));
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .special-badge {
    font-size: 12px; font-weight: 700;
    color: rgba(255, 255, 255, 0.6);
    margin-bottom: 8px; letter-spacing: 2px;
  }
  .special-name {
    font-size: 24px; font-weight: 800; color: white; margin-bottom: 8px;
  }
  .special-meta {
    font-size: 14px; color: rgba(255, 255, 255, 0.6); margin-bottom: 12px;
  }
  .special-desc {
    font-size: 14px; color: rgba(255, 255, 255, 0.8); line-height: 1.6; margin-bottom: 12px;
  }
  .special-tags {
    display: flex; gap: 6px; flex-wrap: wrap;
    .stag {
      padding: 3px 10px; border-radius: 12px; font-size: 11px;
      background: rgba(255, 255, 255, 0.1); color: rgba(255, 255, 255, 0.7);
    }
  }
}

.special-reveal-enter-active { transition: all 0.6s ease-out; }
.special-reveal-enter-from { opacity: 0; transform: scale(0.9) translateY(20px); }

// MVP section
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
    top: -100px; left: 50%; transform: translateX(-50%);
    width: 200px; height: 200px;
    background: radial-gradient(circle, rgba(255, 215, 0, 0.3) 0%, transparent 70%);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.5; transform: translateX(-50%) scale(1); }
    50% { opacity: 1; transform: translateX(-50%) scale(1.2); }
  }

  .mvp-badge {
    position: absolute;
    top: 20px; right: 20px;
    padding: 8px 20px;
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: #1a1a2e;
    font-weight: 800; font-size: 14px;
    border-radius: 20px;
  }

  .mvp-content {
    position: relative; z-index: 1; text-align: center;
  }
  .mvp-name {
    font-size: 36px; font-weight: 800; color: #ffd700; margin-bottom: 16px;
  }
  .mvp-meta {
    display: flex; justify-content: center; align-items: center; gap: 12px; margin-bottom: 24px;
    .mvp-team { color: rgba(255, 255, 255, 0.8); font-size: 16px; }
  }
  .mvp-stats {
    display: flex; justify-content: center; gap: 40px; margin-bottom: 24px;
    .stat-item {
      text-align: center;
      .stat-value { display: block; font-size: 28px; font-weight: 700; color: white; }
      .stat-label { font-size: 12px; color: rgba(255, 255, 255, 0.6); }
    }
  }
  .mvp-commentary {
    padding: 16px; background: rgba(255, 255, 255, 0.05); border-radius: 12px;
    p { color: rgba(255, 255, 255, 0.8); font-size: 15px; line-height: 1.6; margin: 0 0 12px; }
    .mvp-tags {
      display: flex; gap: 6px; justify-content: center;
      .mtag {
        padding: 4px 12px; border-radius: 12px; font-size: 12px;
        background: rgba(255, 215, 0, 0.2); color: #ffd700;
      }
    }
  }
}

// 已颁发提示
.awarded-alert { margin: 20px; }

// 底部
.action-footer {
  padding: 40px;
  text-align: center;
}
</style>
