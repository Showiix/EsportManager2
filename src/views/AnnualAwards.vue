<template>
  <div class="annual-awards">
    <!-- 页面头部 -->
    <div class="ceremony-header">
      <div class="header-content">
        <span class="season-badge">S{{ selectedSeason }}</span>
        <h1 class="ceremony-title">年度颁奖典礼</h1>
        <p class="ceremony-subtitle">表彰本赛季最出色的选手们</p>
        <!-- 赛季选择器：始终可见，方便查看历史数据 -->
        <div class="season-selector-wrapper">
          <SeasonSelector v-model="selectedSeason" />
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="8" animated />
    </div>

    <!-- 空状态：非颁奖阶段且无已颁发数据（仅当前赛季显示此提示） -->
    <div v-else-if="isCurrentSeason && !isAwardsPhase && !awardsData?.already_awarded" class="empty-state">
      <div class="empty-icon-wrap">
        <el-icon :size="48"><Trophy /></el-icon>
      </div>
      <h2>颁奖典礼尚未开始</h2>
      <p>请推进赛季至年度颁奖阶段</p>
    </div>

    <!-- 历史赛季无数据（仅在统计数据也不存在时才显示） -->
    <div v-else-if="!isCurrentSeason && !awardsData?.already_awarded && (!awardsData || awardsData.top20.length === 0)" class="empty-state">
      <div class="empty-icon-wrap">
        <el-icon :size="48"><Trophy /></el-icon>
      </div>
      <h2>暂无颁奖数据</h2>
      <p>S{{ selectedSeason }} 赛季尚未进行颁奖典礼</p>
    </div>

    <!-- 颁奖阶段：开始按钮（仅当前赛季） -->
    <div v-else-if="isCurrentSeason && isAwardsPhase && !ceremonyStarted && !awardsData?.already_awarded" class="start-state">
      <div class="start-icon-wrap">
        <el-icon :size="56"><Trophy /></el-icon>
      </div>
      <h2>年度颁奖典礼已就绪</h2>
      <p>点击下方按钮，开始颁奖</p>
      <el-button type="warning" size="large" :loading="starting" @click="startCeremony">
        开始颁奖典礼
      </el-button>
    </div>

    <!-- 典礼内容 -->
    <div v-else-if="awardsData && awardsData.top20.length > 0" class="ceremony-body">

      <!-- 阶段1: Top20 -->
      <section v-if="currentSection === 'top20' || sectionPassed('top20')" class="section">
        <div class="section-hd">
          <div class="section-tag">ANNUAL TOP 20</div>
          <h2>年度 Top20 选手</h2>
        </div>

        <!-- 表格 -->
        <div class="top20-table">
          <!-- 表头 -->
          <div class="top20-header">
            <div class="col-rank">#</div>
            <div class="col-player">选手</div>
            <div class="col-dim" v-for="dim in dimLabels" :key="dim.key">
              <span class="dim-dot" :style="{ background: dim.color }"></span>
              {{ dim.label }}
            </div>
            <div class="col-score">总分</div>
            <div class="col-tags">标签</div>
          </div>

          <!-- 数据行 -->
          <TransitionGroup name="reveal" tag="div" class="top20-body">
            <div
              v-for="player in visibleTop20"
              :key="player.player_id"
              class="top20-row"
              :class="{ 'is-mvp': player.rank === 1, 'is-top3': player.rank <= 3 && player.rank > 1 }"
              @click="goToPlayer(player.player_id)"
            >
              <div class="col-rank">
                <span class="rank-badge" :class="getRankClass(player.rank)">{{ player.rank }}</span>
              </div>
              <div class="col-player">
                <span class="name">{{ player.player_name }}</span>
                <span class="meta">
                  <el-tag :type="getPositionTagType(player.position)" size="small" effect="dark">{{ getPositionName(player.position) }}</el-tag>
                  <span class="team">{{ player.team_name }}</span>
                </span>
              </div>
              <div class="col-dim" v-for="dim in getDimBars(player)" :key="dim.label">
                <span class="dim-val">{{ dim.value.toFixed(0) }}</span>
                <div class="dim-bar">
                  <div class="dim-fill" :style="{ width: dim.value + '%', background: dim.color }"></div>
                </div>
              </div>
              <div class="col-score">{{ player.yearly_score.toFixed(1) }}</div>
              <div class="col-tags">
                <span v-for="tag in player.commentary.tags.slice(0, 2)" :key="tag" class="tag-chip">{{ tag }}</span>
              </div>
            </div>
          </TransitionGroup>
        </div>

        <div class="actions" v-if="currentSection === 'top20' && !ceremonyComplete">
          <el-button v-if="revealedCount < 20" type="warning" @click="revealNext">
            揭晓 #{{ 20 - revealedCount }}{{ revealedCount >= 19 ? ' — MVP' : '' }}
          </el-button>
          <el-button v-if="revealedCount > 0 && revealedCount < 20" @click="revealAllTop20">
            全部揭晓
          </el-button>
          <el-button v-if="revealedCount >= 20" type="primary" @click="nextSection">
            继续 — 最佳阵容
          </el-button>
        </div>
      </section>

      <!-- 阶段2: 最佳阵容 -->
      <section
        v-if="currentSection === 'allpro' || sectionPassed('allpro')"
        class="section"
      >
        <div class="section-hd">
          <div class="section-tag">ALL-PRO TEAMS</div>
          <h2>年度最佳阵容</h2>
        </div>

        <div class="allpro-container">
          <Transition name="tier-reveal">
            <div class="tier" v-if="revealedTier >= 1 || ceremonyComplete">
              <div class="tier-hd bronze">三阵</div>
              <div class="tier-grid">
                <div class="tier-card" v-for="p in sortByPosition(awardsData.all_pro_3rd)" :key="p.player_id" @click="goToPlayer(p.player_id)">
                  <el-tag :type="getPositionTagType(p.position)" size="small" effect="dark">{{ getPositionName(p.position) }}</el-tag>
                  <span class="tc-name">{{ p.player_name }}</span>
                  <span class="tc-team">{{ p.team_name }}</span>
                  <span class="tc-score">{{ p.yearly_score.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </Transition>

          <Transition name="tier-reveal">
            <div class="tier" v-if="revealedTier >= 2 || ceremonyComplete">
              <div class="tier-hd silver">二阵</div>
              <div class="tier-grid">
                <div class="tier-card" v-for="p in sortByPosition(awardsData.all_pro_2nd)" :key="p.player_id" @click="goToPlayer(p.player_id)">
                  <el-tag :type="getPositionTagType(p.position)" size="small" effect="dark">{{ getPositionName(p.position) }}</el-tag>
                  <span class="tc-name">{{ p.player_name }}</span>
                  <span class="tc-team">{{ p.team_name }}</span>
                  <span class="tc-score">{{ p.yearly_score.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </Transition>

          <Transition name="tier-reveal">
            <div class="tier" v-if="revealedTier >= 3 || ceremonyComplete">
              <div class="tier-hd gold">一阵</div>
              <div class="tier-grid">
                <div class="tier-card gold-border" v-for="p in sortByPosition(awardsData.all_pro_1st)" :key="p.player_id" @click="goToPlayer(p.player_id)">
                  <el-tag :type="getPositionTagType(p.position)" size="small" effect="dark">{{ getPositionName(p.position) }}</el-tag>
                  <span class="tc-name">{{ p.player_name }}</span>
                  <span class="tc-team">{{ p.team_name }}</span>
                  <span class="tc-score">{{ p.yearly_score.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </Transition>
        </div>

        <div class="actions" v-if="currentSection === 'allpro' && !ceremonyComplete">
          <el-button v-if="revealedTier < 3" type="warning" @click="revealNextTier">
            揭晓{{ revealedTier === 0 ? '三阵' : revealedTier === 1 ? '二阵' : '一阵' }}
          </el-button>
          <el-button v-if="revealedTier >= 3" type="primary" @click="nextSection">
            继续 — 特别奖项
          </el-button>
        </div>
      </section>

      <!-- 阶段3: 特别奖项 -->
      <section
        v-if="currentSection === 'special' || sectionPassed('special')"
        class="section"
      >
        <div class="section-hd">
          <div class="section-tag">SPECIAL AWARDS</div>
          <h2>特别奖项</h2>
        </div>

        <div class="special-grid">
          <Transition name="special-reveal">
            <div v-if="(revealedSpecial >= 1 || ceremonyComplete) && awardsData.most_consistent" class="sp-card" @click="goToPlayer(awardsData.most_consistent.player_id)">
              <div class="sp-icon stable-icon"><el-icon :size="24"><Aim /></el-icon></div>
              <div class="sp-label">最稳定选手</div>
              <div class="sp-name">{{ awardsData.most_consistent.player_name }}</div>
              <div class="sp-meta">{{ awardsData.most_consistent.team_name }} · {{ getPositionName(awardsData.most_consistent.position) }}</div>
              <p class="sp-desc">{{ awardsData.most_consistent.commentary.description }}</p>
              <div class="sp-tags"><span v-for="tag in awardsData.most_consistent.commentary.tags" :key="tag">{{ tag }}</span></div>
            </div>
          </Transition>

          <Transition name="special-reveal">
            <div v-if="(revealedSpecial >= 2 || ceremonyComplete) && awardsData.rookie_of_the_year" class="sp-card" @click="goToPlayer(awardsData.rookie_of_the_year.player_id)">
              <div class="sp-icon rookie-icon"><el-icon :size="24"><Star /></el-icon></div>
              <div class="sp-label">最佳新秀</div>
              <div class="sp-name">{{ awardsData.rookie_of_the_year.player_name }}</div>
              <div class="sp-meta">{{ awardsData.rookie_of_the_year.team_name }} · {{ getPositionName(awardsData.rookie_of_the_year.position) }} · {{ awardsData.rookie_of_the_year.age }}岁</div>
              <p class="sp-desc">{{ awardsData.rookie_of_the_year.commentary.description }}</p>
              <div class="sp-tags"><span v-for="tag in awardsData.rookie_of_the_year.commentary.tags" :key="tag">{{ tag }}</span></div>
            </div>
          </Transition>

          <Transition name="special-reveal">
            <div v-if="(revealedSpecial >= 3 || ceremonyComplete) && awardsData.most_dominant" class="sp-card" @click="goToPlayer(awardsData.most_dominant.player_id)">
              <div class="sp-icon dominant-icon"><el-icon :size="24"><MagicStick /></el-icon></div>
              <div class="sp-label">最具统治力</div>
              <div class="sp-name">{{ awardsData.most_dominant.player_name }}</div>
              <div class="sp-meta">{{ awardsData.most_dominant.team_name }} · {{ getPositionName(awardsData.most_dominant.position) }}</div>
              <p class="sp-desc">{{ awardsData.most_dominant.commentary.description }}</p>
              <div class="sp-tags"><span v-for="tag in awardsData.most_dominant.commentary.tags" :key="tag">{{ tag }}</span></div>
            </div>
          </Transition>
        </div>

        <div class="actions" v-if="currentSection === 'special' && !ceremonyComplete">
          <el-button v-if="revealedSpecial < 3" type="warning" @click="revealNextSpecial">
            揭晓{{ revealedSpecial === 0 ? '最稳定选手' : revealedSpecial === 1 ? '最佳新秀' : '最具统治力' }}
          </el-button>
          <el-button v-if="revealedSpecial >= 3" type="primary" @click="nextSection">
            继续 — 年度MVP
          </el-button>
        </div>
      </section>

      <!-- 阶段4: MVP -->
      <section v-if="currentSection === 'mvp' || sectionPassed('mvp')" class="section mvp-section">
        <div class="section-hd">
          <div class="section-tag mvp-tag">ANNUAL MVP</div>
          <h2>年度最有价值选手</h2>
        </div>

        <div class="mvp-card" v-if="mvpPlayer" @click="goToPlayer(mvpPlayer.player_id)">
          <div class="mvp-glow"></div>
          <div class="mvp-badge-float">MVP</div>
          <div class="mvp-body">
            <div class="mvp-name">{{ mvpPlayer.player_name }}</div>
            <div class="mvp-meta">
              <el-tag :type="getPositionTagType(mvpPlayer.position)" effect="dark" size="large">{{ getPositionName(mvpPlayer.position) }}</el-tag>
              <span class="mvp-team">{{ mvpPlayer.team_name }}</span>
            </div>
            <div class="mvp-numbers">
              <div><strong>{{ mvpPlayer.yearly_score.toFixed(1) }}</strong><span>年度得分</span></div>
              <div><strong>{{ mvpPlayer.avg_impact.toFixed(1) }}</strong><span>场均影响力</span></div>
              <div><strong>{{ mvpPlayer.games_played }}</strong><span>出场次数</span></div>
            </div>
            <div class="mvp-quote" v-if="mvpPlayer.commentary.description">
              <p>"{{ mvpPlayer.commentary.description }}"</p>
              <div class="mvp-tags">
                <span v-for="tag in mvpPlayer.commentary.tags" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="actions" v-if="currentSection === 'mvp' && !ceremonyComplete">
          <el-button type="success" @click="completeCeremony">典礼完成</el-button>
        </div>
      </section>

      <!-- 完成提示 -->
      <el-alert
        v-if="ceremonyComplete"
        title="本赛季年度颁奖典礼已完成"
        type="success"
        show-icon
        :closable="false"
        class="done-alert"
      />

      <!-- 底部 -->
      <div class="footer-actions">
        <el-button @click="goBack">返回时间面板</el-button>
      </div>
    </div>

    <!-- 有数据但 top20 为空 -->
    <div v-else-if="awardsData && awardsData.top20.length === 0 && !loading" class="empty-state">
      <div class="empty-icon-wrap">
        <el-icon :size="48"><Trophy /></el-icon>
      </div>
      <h2>暂无评选数据</h2>
      <p>当前赛季尚无足够的比赛数据，请先模拟更多比赛</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Trophy, Aim, Star, MagicStick } from '@element-plus/icons-vue'
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

const isAwardsPhase = computed(() => timeStore.isAnnualAwardsPhase)
const isCurrentSeason = computed(() => selectedSeason.value === seasonStore.currentSeason)
const mvpPlayer = computed(() => awardsData.value?.top20[0] || null)

const visibleTop20 = computed(() => {
  if (ceremonyComplete.value || sectionPassed('top20')) return awardsData.value?.top20 ?? []
  const all = awardsData.value?.top20 ?? []
  return all.slice(Math.max(0, all.length - revealedCount.value))
    .sort((a, b) => a.rank - b.rank)
})

const sortByPosition = (players: AllProPlayer[]) => {
  const order: Record<string, number> = { TOP: 0, JUG: 1, MID: 2, ADC: 3, SUP: 4 }
  return [...players].sort((a, b) => (order[a.position] ?? 99) - (order[b.position] ?? 99))
}

const sectionPassed = (section: string) => {
  const order = ['top20', 'allpro', 'special', 'mvp', 'complete']
  return ceremonyComplete.value || order.indexOf(currentSection.value) > order.indexOf(section)
}

const dimLabels = [
  { key: 'impact', label: '影响力', color: '#60a5fa' },
  { key: 'performance', label: '发挥', color: '#34d399' },
  { key: 'stability', label: '稳定', color: '#a78bfa' },
  { key: 'appearance', label: '出场', color: '#fbbf24' },
  { key: 'honor', label: '荣誉', color: '#f87171' },
]

const getDimBars = (player: Top20Player) => [
  { label: '影响力', value: player.dimensions.impact_norm, color: '#60a5fa' },
  { label: '发挥', value: player.dimensions.performance_norm, color: '#34d399' },
  { label: '稳定', value: player.dimensions.stability_norm, color: '#a78bfa' },
  { label: '出场', value: player.dimensions.appearance_norm, color: '#fbbf24' },
  { label: '荣誉', value: player.dimensions.honor_norm, color: '#f87171' },
]

// --- Actions ---

const fetchAwardsData = async (seasonId?: number) => {
  loading.value = true
  try {
    awardsData.value = await awardsApi.getAnnualAwardsData(seasonId)
    if (awardsData.value?.already_awarded) {
      ceremonyComplete.value = true
      ceremonyStarted.value = true
    } else if (!isCurrentSeason.value && awardsData.value && awardsData.value.top20.length > 0) {
      // 历史赛季：虽然颁奖典礼未正式完成，但有统计数据，直接展示
      ceremonyComplete.value = true
      ceremonyStarted.value = true
    }
  } catch (error) {
    logger.error('获取颁奖数据失败:', error)
  } finally {
    loading.value = false
  }
}

const resetCeremonyState = () => {
  ceremonyStarted.value = false
  ceremonyComplete.value = false
  revealedCount.value = 0
  revealedTier.value = 0
  revealedSpecial.value = 0
  currentSection.value = 'top20'
}

const startCeremony = async () => {
  starting.value = true
  try {
    await timeStore.completeAndAdvance()
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

const revealNext = () => { if (revealedCount.value < 20) revealedCount.value++ }
const revealAllTop20 = () => { revealedCount.value = 20 }
const revealNextTier = () => { if (revealedTier.value < 3) revealedTier.value++ }
const revealNextSpecial = () => { if (revealedSpecial.value < 3) revealedSpecial.value++ }

const nextSection = () => {
  const order: Array<typeof currentSection.value> = ['top20', 'allpro', 'special', 'mvp', 'complete']
  const idx = order.indexOf(currentSection.value)
  if (idx < order.length - 1) currentSection.value = order[idx + 1]
}

const completeCeremony = () => {
  ceremonyComplete.value = true
  currentSection.value = 'complete'
}

const goToPlayer = (id: number) => router.push(`/data-center/player/${id}?season=S${selectedSeason.value}`)
const goBack = () => router.push('/time')

const getPositionName = (pos: string) => POSITION_NAMES[pos as PlayerPosition] || pos
const getPositionTagType = (pos: string) => ({ TOP: 'danger', JUG: 'warning', MID: 'primary', ADC: 'success', SUP: 'info' }[pos] || 'info')
const getRankClass = (r: number) => r === 1 ? 'gold' : r === 2 ? 'silver' : r === 3 ? 'bronze' : ''

onMounted(async () => {
  await timeStore.fetchTimeState()
  await fetchAwardsData(selectedSeason.value)
})

watch(selectedSeason, (s) => {
  resetCeremonyState()
  fetchAwardsData(s)
})
</script>

<style scoped lang="scss">
/* ========== 全局 ========== */
.annual-awards {
  min-height: 100%;
  background: #0f1923;
  color: #c8d6e5;
}

/* ========== Header ========== */
.ceremony-header {
  padding: 48px 32px 32px;
  text-align: center;
  background: linear-gradient(180deg, #1a2940 0%, #0f1923 100%);
  border-bottom: 1px solid rgba(255, 215, 0, 0.08);

  .season-badge {
    display: inline-block;
    padding: 4px 16px;
    font-size: 13px;
    font-weight: 700;
    color: #fbbf24;
    border: 1px solid rgba(251, 191, 36, 0.4);
    border-radius: 20px;
    letter-spacing: 2px;
    margin-bottom: 12px;
  }

  .ceremony-title {
    font-size: 32px;
    font-weight: 800;
    color: #f1f5f9;
    margin: 0 0 8px;
    letter-spacing: 2px;
  }

  .ceremony-subtitle {
    font-size: 14px;
    color: #64748b;
    margin: 0;
  }

  .season-selector-wrapper {
    margin-top: 16px;
    display: flex;
    justify-content: center;
  }
}

/* ========== 空状态 / 开始 ========== */
.empty-state, .start-state {
  padding: 80px 32px;
  text-align: center;

  .empty-icon-wrap, .start-icon-wrap {
    width: 88px;
    height: 88px;
    margin: 0 auto 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: rgba(251, 191, 36, 0.08);
    color: #fbbf24;
  }

  h2 {
    font-size: 22px;
    font-weight: 700;
    color: #e2e8f0;
    margin: 0 0 8px;
  }

  p {
    font-size: 14px;
    color: #64748b;
    margin: 0 0 28px;
  }
}

/* ========== Loading ========== */
.loading-container {
  padding: 40px 32px;
}

/* ========== Ceremony Body ========== */
.ceremony-body {
  padding: 0 24px 40px;
}

/* ========== Section 通用 ========== */
.section {
  margin-top: 24px;
  padding: 28px;
  background: #151f2e;
  border: 1px solid #1e293b;
  border-radius: 12px;
}

.section-hd {
  text-align: center;
  margin-bottom: 24px;

  .section-tag {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 4px;
    color: #475569;
    margin-bottom: 4px;

    &.mvp-tag { color: #fbbf24; }
  }

  h2 {
    font-size: 20px;
    font-weight: 700;
    color: #f1f5f9;
    margin: 0;
  }
}

.actions {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-top: 24px;
}

/* ========== Top20 表格 (CSS Grid) ========== */
$grid-cols: 40px 1fr 72px 72px 72px 72px 72px 56px 120px;

.top20-table {
  overflow-x: auto;
}

.top20-header {
  display: grid;
  grid-template-columns: $grid-cols;
  gap: 0 8px;
  padding: 0 12px 10px;
  border-bottom: 1px solid #1e293b;
  margin-bottom: 4px;
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  align-items: center;

  .col-score { text-align: right; }

  .col-dim {
    display: flex;
    align-items: center;
    gap: 4px;

    .dim-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      flex-shrink: 0;
    }
  }
}

.top20-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.top20-row {
  display: grid;
  grid-template-columns: $grid-cols;
  gap: 0 8px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  align-items: center;
  transition: background 0.2s;

  &:hover { background: rgba(255, 255, 255, 0.04); }

  &.is-mvp {
    background: rgba(251, 191, 36, 0.06);
    border: 1px solid rgba(251, 191, 36, 0.15);
  }

  &.is-top3 {
    background: rgba(148, 163, 184, 0.06);
  }

  .col-rank {
    .rank-badge {
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 8px;
      font-weight: 800;
      font-size: 13px;
      color: #64748b;
      background: #1e293b;

      &.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); color: #1a1a2e; }
      &.silver { background: linear-gradient(135deg, #94a3b8, #64748b); color: #1a1a2e; }
      &.bronze { background: linear-gradient(135deg, #cd7f32, #b8860b); color: #1a1a2e; }
    }
  }

  .col-player {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;

    .name {
      font-size: 14px;
      font-weight: 600;
      color: #e2e8f0;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .meta {
      display: flex;
      align-items: center;
      gap: 6px;
      .team { font-size: 12px; color: #64748b; }
    }
  }

  .col-dim {
    display: flex;
    flex-direction: column;
    gap: 3px;

    .dim-val {
      font-size: 12px;
      font-weight: 700;
      color: #cbd5e1;
    }

    .dim-bar {
      height: 4px;
      background: #1e293b;
      border-radius: 2px;
      overflow: hidden;

      .dim-fill {
        height: 100%;
        border-radius: 2px;
        transition: width 0.3s ease;
      }
    }
  }

  .col-score {
    text-align: right;
    font-size: 17px;
    font-weight: 800;
    color: #fbbf24;
  }

  .col-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;

    .tag-chip {
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 11px;
      background: #1e293b;
      color: #94a3b8;
    }
  }
}

/* ========== 揭晓动画 ========== */
.reveal-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.reveal-enter-from { opacity: 0; transform: translateY(20px); }

/* ========== 最佳阵容 ========== */
.allpro-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.tier {
  .tier-hd {
    font-size: 14px;
    font-weight: 700;
    padding: 8px 16px;
    border-radius: 6px;
    text-align: center;
    color: white;
    margin-bottom: 10px;

    &.gold { background: linear-gradient(135deg, #b8860b, #daa520); }
    &.silver { background: linear-gradient(135deg, #64748b, #94a3b8); }
    &.bronze { background: linear-gradient(135deg, #8b5e3c, #cd7f32); }
  }

  .tier-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
  }

  .tier-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 14px 8px;
    border-radius: 8px;
    background: #1a2636;
    border: 1px solid #1e293b;
    cursor: pointer;
    transition: all 0.2s;

    &:hover { background: #1e2d40; border-color: #334155; }

    &.gold-border { border-color: rgba(251, 191, 36, 0.25); }

    .tc-name { font-size: 14px; font-weight: 700; color: #e2e8f0; text-align: center; }
    .tc-team { font-size: 12px; color: #64748b; }
    .tc-score { font-size: 16px; font-weight: 800; color: #fbbf24; }
  }
}

.tier-reveal-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.tier-reveal-enter-from { opacity: 0; transform: translateY(30px); }

/* ========== 特别奖项 ========== */
.special-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.sp-card {
  padding: 24px;
  border-radius: 10px;
  background: #1a2636;
  border: 1px solid #1e293b;
  cursor: pointer;
  transition: all 0.25s;

  &:hover { border-color: #334155; transform: translateY(-2px); }

  .sp-icon {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 10px;
    margin-bottom: 14px;

    &.stable-icon { background: rgba(139, 92, 246, 0.15); color: #a78bfa; }
    &.rookie-icon { background: rgba(52, 211, 153, 0.15); color: #34d399; }
    &.dominant-icon { background: rgba(248, 113, 113, 0.15); color: #f87171; }
  }

  .sp-label { font-size: 12px; font-weight: 600; color: #64748b; letter-spacing: 1px; margin-bottom: 6px; }
  .sp-name { font-size: 22px; font-weight: 800; color: #f1f5f9; margin-bottom: 4px; }
  .sp-meta { font-size: 13px; color: #64748b; margin-bottom: 14px; }
  .sp-desc { font-size: 13px; color: #94a3b8; line-height: 1.6; margin: 0 0 12px; }
  .sp-tags {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    span {
      padding: 2px 10px;
      border-radius: 4px;
      font-size: 11px;
      background: #1e293b;
      color: #94a3b8;
    }
  }
}

.special-reveal-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.special-reveal-enter-from { opacity: 0; transform: translateY(20px) scale(0.97); }

/* ========== MVP ========== */
.mvp-section {
  border-color: rgba(251, 191, 36, 0.12) !important;
  background: linear-gradient(180deg, #1a2233 0%, #151f2e 100%);
}

.mvp-card {
  position: relative;
  max-width: 560px;
  margin: 0 auto;
  padding: 36px;
  background: #111a27;
  border: 2px solid rgba(251, 191, 36, 0.3);
  border-radius: 16px;
  cursor: pointer;
  overflow: hidden;
  transition: transform 0.3s, box-shadow 0.3s;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 32px rgba(251, 191, 36, 0.12);
  }

  .mvp-glow {
    position: absolute;
    top: -80px;
    left: 50%;
    transform: translateX(-50%);
    width: 240px;
    height: 160px;
    background: radial-gradient(ellipse, rgba(251, 191, 36, 0.12) 0%, transparent 70%);
    pointer-events: none;
  }

  .mvp-badge-float {
    position: absolute;
    top: 16px;
    right: 16px;
    padding: 4px 16px;
    background: linear-gradient(135deg, #fbbf24, #f59e0b);
    color: #1a1a2e;
    font-weight: 800;
    font-size: 13px;
    border-radius: 6px;
    letter-spacing: 2px;
  }

  .mvp-body {
    position: relative;
    z-index: 1;
    text-align: center;
  }

  .mvp-name {
    font-size: 30px;
    font-weight: 800;
    color: #fbbf24;
    margin-bottom: 12px;
  }

  .mvp-meta {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 10px;
    margin-bottom: 24px;
    .mvp-team { color: #94a3b8; font-size: 15px; }
  }

  .mvp-numbers {
    display: flex;
    justify-content: center;
    gap: 36px;
    margin-bottom: 24px;

    div {
      text-align: center;
      strong { display: block; font-size: 24px; font-weight: 800; color: #f1f5f9; }
      span { font-size: 11px; color: #64748b; }
    }
  }

  .mvp-quote {
    padding: 16px;
    background: rgba(251, 191, 36, 0.04);
    border: 1px solid rgba(251, 191, 36, 0.1);
    border-radius: 8px;

    p {
      color: #cbd5e1;
      font-size: 14px;
      line-height: 1.7;
      margin: 0 0 10px;
      font-style: italic;
    }

    .mvp-tags {
      display: flex;
      gap: 6px;
      justify-content: center;
      span {
        padding: 3px 12px;
        border-radius: 4px;
        font-size: 11px;
        background: rgba(251, 191, 36, 0.1);
        color: #fbbf24;
      }
    }
  }
}

/* ========== Footer ========== */
.done-alert {
  margin: 24px 0 0;
  border-radius: 8px;
}

.footer-actions {
  display: flex;
  justify-content: center;
  gap: 12px;
  padding: 32px 0 16px;
}

/* ========== Responsive ========== */
@media (max-width: 900px) {
  .special-grid { grid-template-columns: 1fr; }
  .tier .tier-grid { grid-template-columns: repeat(3, 1fr); }
  .top20-header,
  .top20-row {
    grid-template-columns: 40px 1fr 56px;
  }
  .top20-header .col-dim,
  .top20-header .col-tags,
  .top20-row .col-dim,
  .top20-row .col-tags {
    display: none;
  }
}
</style>
