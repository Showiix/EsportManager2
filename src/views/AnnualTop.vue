<template>
  <div class="annual-top">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-banner">
        <div class="banner-content">
          <h1 class="banner-title">IM 年度选手评选</h1>
          <p class="banner-subtitle">S{{ selectedSeason }} 赛季 年度最佳选手 TOP 20</p>
          <div class="scoring-rule">
            <el-tag type="warning" effect="dark" size="large">
              评选标准: 影响力(40%) + 发挥(18%) + 大赛表现(15%) + 稳定性(12%) + 出场(10%) + 荣誉(5%)
            </el-tag>
          </div>
          <div class="scoring-detail">
            <span>国际赛冠军 +3 | 亚军 +2 | 季军 +1</span>
            <span class="divider">|</span>
            <span>赛区冠军 +1 | 亚军 +0.5 | 季军 +0.25</span>
            <span class="divider">|</span>
            <span>赛事权重: 世界赛1.5x | Super1.4x | MSI1.3x</span>
          </div>
        </div>
        <div class="header-actions">
          <el-button
            type="primary"
            :icon="Refresh"
            :loading="recalculating"
            @click="recalculateScores"
          >
            刷新排名
          </el-button>
          <SeasonSelector v-model="selectedSeason" width="140px" />
        </div>
      </div>
    </div>

    <!-- 前三名 + 雷达图 -->
    <div class="top-section" v-if="top20.length > 0">
      <!-- 前三名展示 -->
      <div class="top-three">
        <!-- 第二名 -->
        <div class="top-card silver" v-if="top20[1]" @click="goToDetail(top20[1])">
          <div class="rank-badge">2</div>
          <div class="player-name">{{ top20[1].player_name }}</div>
          <div class="player-meta">
            <el-tag :type="getPositionTagType(top20[1].position)" size="small">
              {{ getPositionName(top20[1].position) }}
            </el-tag>
            <span>{{ top20[1].team_name }}</span>
          </div>
          <div class="dim-bars">
            <div class="dim-row" v-for="dim in getDimBars(top20[1])" :key="dim.label">
              <span class="dim-label">{{ dim.label }}</span>
              <div class="dim-bar-bg"><div class="dim-bar-fill" :style="{ width: dim.value + '%', background: dim.color }"></div></div>
              <span class="dim-val">{{ dim.value.toFixed(0) }}</span>
            </div>
          </div>
          <div class="total-score">
            <span class="label">年度得分</span>
            <span class="value">{{ top20[1].yearly_score.toFixed(1) }}</span>
          </div>
        </div>

        <!-- 第一名 -->
        <div class="top-card gold" v-if="top20[0]" @click="goToDetail(top20[0])">
          <div class="crown-icon"></div>
          <div class="rank-badge mvp">MVP</div>
          <div class="player-name">{{ top20[0].player_name }}</div>
          <div class="player-meta">
            <el-tag :type="getPositionTagType(top20[0].position)" size="small">
              {{ getPositionName(top20[0].position) }}
            </el-tag>
            <span>{{ top20[0].team_name }}</span>
          </div>
          <div class="dim-bars">
            <div class="dim-row" v-for="dim in getDimBars(top20[0])" :key="dim.label">
              <span class="dim-label">{{ dim.label }}</span>
              <div class="dim-bar-bg"><div class="dim-bar-fill" :style="{ width: dim.value + '%', background: dim.color }"></div></div>
              <span class="dim-val">{{ dim.value.toFixed(0) }}</span>
            </div>
          </div>
          <div class="total-score">
            <span class="label">年度得分</span>
            <span class="value">{{ top20[0].yearly_score.toFixed(1) }}</span>
          </div>
        </div>

        <!-- 第三名 -->
        <div class="top-card bronze" v-if="top20[2]" @click="goToDetail(top20[2])">
          <div class="rank-badge">3</div>
          <div class="player-name">{{ top20[2].player_name }}</div>
          <div class="player-meta">
            <el-tag :type="getPositionTagType(top20[2].position)" size="small">
              {{ getPositionName(top20[2].position) }}
            </el-tag>
            <span>{{ top20[2].team_name }}</span>
          </div>
          <div class="dim-bars">
            <div class="dim-row" v-for="dim in getDimBars(top20[2])" :key="dim.label">
              <span class="dim-label">{{ dim.label }}</span>
              <div class="dim-bar-bg"><div class="dim-bar-fill" :style="{ width: dim.value + '%', background: dim.color }"></div></div>
              <span class="dim-val">{{ dim.value.toFixed(0) }}</span>
            </div>
          </div>
          <div class="total-score">
            <span class="label">年度得分</span>
            <span class="value">{{ top20[2].yearly_score.toFixed(1) }}</span>
          </div>
        </div>
      </div>

      <!-- 雷达图对比 -->
      <el-card class="radar-card" v-if="top20.length >= 3">
        <template #header>
          <span class="card-title">Top3 六维对比</span>
        </template>
        <div class="chart-container">
          <v-chart class="chart" :option="radarOption" autoresize />
        </div>
      </el-card>
    </div>

    <!-- Top20 完整排行表 -->
    <el-card class="ranking-list" v-if="top20.length > 0">
      <template #header>
        <span class="card-title">年度 Top 20 完整排行</span>
        <span class="card-hint">点击行展开赛事明细 · 双击跳转选手详情</span>
      </template>
      <el-table ref="tableRef" :data="top20" stripe style="width: 100%" :row-key="(row: Top20Player) => row.player_id" @row-click="toggleExpand" @row-dblclick="goToDetail">
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="tournament-breakdown" v-if="row.tournament_details && row.tournament_details.length > 0">
              <div class="breakdown-header">
                <span class="breakdown-title">各赛事表现明细</span>
                <span class="big-stage-badge" :class="row.big_stage_score >= 0 ? 'positive' : 'negative'">
                  大赛影响力 {{ row.big_stage_score >= 0 ? '+' : '' }}{{ row.big_stage_score.toFixed(1) }}
                </span>
              </div>
              <div class="breakdown-grid">
                <div class="tournament-item" v-for="td in row.tournament_details" :key="td.tournament_type">
                  <div class="td-header">
                    <span class="td-name">{{ td.tournament_name }}</span>
                    <span class="td-weight" :class="getWeightClass(td.weight)">×{{ td.weight.toFixed(1) }}</span>
                  </div>
                  <div class="td-stats">
                    <div class="td-stat">
                      <span class="td-label">场次</span>
                      <span class="td-value">{{ td.games_played }}</span>
                    </div>
                    <div class="td-stat">
                      <span class="td-label">影响力</span>
                      <span class="td-value" :class="{ 'positive-val': td.avg_impact > 0, 'negative-val': td.avg_impact < 0 }">
                        {{ td.avg_impact > 0 ? '+' : '' }}{{ td.avg_impact.toFixed(1) }}
                      </span>
                    </div>
                    <div class="td-stat">
                      <span class="td-label">发挥</span>
                      <span class="td-value">{{ td.avg_performance.toFixed(1) }}</span>
                    </div>
                    <div class="td-stat">
                      <span class="td-label">巅峰</span>
                      <span class="td-value">{{ td.best_performance.toFixed(1) }}</span>
                    </div>
                    <div class="td-stat" v-if="td.mvp_count > 0">
                      <span class="td-label">MVP</span>
                      <span class="td-value mvp-val">{{ td.mvp_count }}次</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div v-else class="no-breakdown">暂无赛事明细数据</div>
          </template>
        </el-table-column>

        <el-table-column label="#" width="45" align="center">
          <template #default="{ row }">
            <span class="rank-number" :class="{ 'top-rank': row.rank <= 3 }">{{ row.rank }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="player_name" label="选手" min-width="100">
          <template #default="{ row }">
            <span class="player-link">{{ row.player_name }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="position" label="位置" width="65" align="center">
          <template #default="{ row }">
            <el-tag :type="getPositionTagType(row.position)" size="small">
              {{ getPositionName(row.position) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="team_name" label="战队" width="80" />

        <el-table-column label="影响力" width="85" align="center">
          <template #default="{ row }">
            <div class="mini-bar-cell">
              <div class="mini-bar"><div class="mini-fill impact" :style="{ width: row.dimensions.impact_norm + '%' }"></div></div>
              <span>{{ row.avg_impact.toFixed(1) }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="发挥" width="80" align="center">
          <template #default="{ row }">
            <div class="mini-bar-cell">
              <div class="mini-bar"><div class="mini-fill perf" :style="{ width: row.dimensions.performance_norm + '%' }"></div></div>
              <span>{{ row.avg_performance.toFixed(1) }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="大赛" width="80" align="center">
          <template #default="{ row }">
            <div class="mini-bar-cell">
              <div class="mini-bar"><div class="mini-fill big-stage" :style="{ width: row.dimensions.big_stage_norm + '%' }"></div></div>
              <span :class="{ 'positive-val': row.big_stage_score > 0, 'negative-val': row.big_stage_score < 0 }">
                {{ row.big_stage_score > 0 ? '+' : '' }}{{ row.big_stage_score.toFixed(1) }}
              </span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="稳定" width="70" align="center">
          <template #default="{ row }">
            <div class="mini-bar-cell">
              <div class="mini-bar"><div class="mini-fill stab" :style="{ width: row.dimensions.stability_norm + '%' }"></div></div>
              <span>{{ row.consistency_score.toFixed(0) }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="出场" width="55" align="center">
          <template #default="{ row }">
            <span class="games-value">{{ row.games_played }}</span>
          </template>
        </el-table-column>

        <el-table-column label="荣誉" width="60" align="center">
          <template #default="{ row }">
            <span class="bonus-value" v-if="row.champion_bonus > 0">+{{ row.champion_bonus.toFixed(0) }}</span>
            <span v-else>-</span>
          </template>
        </el-table-column>

        <el-table-column prop="yearly_score" label="总分" width="70" align="center" sortable>
          <template #default="{ row }">
            <span class="score-value">{{ row.yearly_score.toFixed(1) }}</span>
          </template>
        </el-table-column>

        <el-table-column label="标签" min-width="120">
           <template #default="{ row }">
            <div class="tags-cell">
              <el-tag
                v-for="tag in row.commentary.tags.slice(0, 3)"
                :key="tag"
                size="small"
                effect="plain"
                round
                :type="tag === '未参加国际赛' ? 'warning' : ''"
              >{{ tag }}</el-tag>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 最佳阵容 + 特别奖项 -->
    <div class="bottom-section" v-if="awardsData">
      <!-- 最佳阵容 -->
      <el-card class="allpro-card">
        <template #header>
          <span class="card-title">最佳阵容</span>
        </template>
        <div class="allpro-tiers">
          <div class="tier-section" v-for="tier in allProTiers" :key="tier.label">
            <div class="tier-header" :class="tier.class">{{ tier.label }}</div>
            <div class="tier-players">
              <div
                class="allpro-player"
                v-for="p in tier.players"
                :key="p.player_id"
                @click="goToPlayerDetail(p.player_id)"
              >
                <el-tag :type="getPositionTagType(p.position)" size="small">
                  {{ getPositionName(p.position) }}
                </el-tag>
                <span class="ap-name">{{ p.player_name }}</span>
                <span class="ap-team">{{ p.team_name }}</span>
                <span class="ap-score">{{ p.yearly_score.toFixed(1) }}</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 特别奖项 -->
      <el-card class="special-card">
        <template #header>
          <span class="card-title">特别奖项</span>
        </template>
        <div class="special-awards">
          <!-- 最稳定选手 -->
          <div class="special-item" v-if="awardsData.most_consistent" @click="goToPlayerDetail(awardsData.most_consistent.player_id)">
            <div class="special-icon stable"></div>
            <div class="special-label">最稳定选手</div>
            <div class="special-name">{{ awardsData.most_consistent.player_name }}</div>
            <div class="special-meta">{{ awardsData.most_consistent.team_name }} · {{ getPositionName(awardsData.most_consistent.position) }}</div>
            <div class="special-desc">{{ awardsData.most_consistent.commentary.description }}</div>
            <div class="special-tags">
              <el-tag v-for="tag in awardsData.most_consistent.commentary.tags" :key="tag" size="small" effect="plain" round>{{ tag }}</el-tag>
            </div>
          </div>

          <!-- 最具统治力 -->
          <div class="special-item" v-if="awardsData.most_dominant" @click="goToPlayerDetail(awardsData.most_dominant.player_id)">
            <div class="special-icon dominant"></div>
            <div class="special-label">最具统治力</div>
            <div class="special-name">{{ awardsData.most_dominant.player_name }}</div>
            <div class="special-meta">{{ awardsData.most_dominant.team_name }} · {{ getPositionName(awardsData.most_dominant.position) }}</div>
            <div class="special-desc">{{ awardsData.most_dominant.commentary.description }}</div>
            <div class="special-tags">
              <el-tag v-for="tag in awardsData.most_dominant.commentary.tags" :key="tag" size="small" effect="plain" round>{{ tag }}</el-tag>
            </div>
          </div>

          <!-- 最佳新秀 -->
          <div class="special-item" v-if="awardsData.rookie_of_the_year" @click="goToPlayerDetail(awardsData.rookie_of_the_year.player_id)">
            <div class="special-icon rookie"></div>
            <div class="special-label">最佳新秀</div>
            <div class="special-name">{{ awardsData.rookie_of_the_year.player_name }}</div>
            <div class="special-meta">{{ awardsData.rookie_of_the_year.team_name }} · {{ getPositionName(awardsData.rookie_of_the_year.position) }} · {{ awardsData.rookie_of_the_year.age }}岁</div>
            <div class="special-desc">{{ awardsData.rookie_of_the_year.commentary.description }}</div>
            <div class="special-tags">
              <el-tag v-for="tag in awardsData.rookie_of_the_year.commentary.tags" :key="tag" size="small" effect="plain" round>{{ tag }}</el-tag>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 入选分布统计 -->
    <div class="distribution-stats" v-if="top20.length > 0">
      <el-card class="dist-card">
        <template #header>
          <span class="card-title">位置分布</span>
        </template>
        <div class="position-dist">
          <div
            v-for="pos in positionDistribution"
            :key="pos.position"
            class="dist-item"
          >
            <div class="dist-bar-container">
              <div
                class="dist-bar"
                :style="{ height: `${pos.percentage}%` }"
                :class="pos.position.toLowerCase()"
              ></div>
            </div>
            <div class="dist-label">{{ pos.position }}</div>
            <div class="dist-count">{{ pos.count }}人</div>
          </div>
        </div>
      </el-card>

      <el-card class="dist-card">
        <template #header>
          <span class="card-title">战队分布</span>
        </template>
        <div class="region-dist">
          <div
            v-for="item in teamDistribution"
            :key="item.name"
            class="region-item"
          >
            <span class="region-name">{{ item.name }}</span>
            <div class="region-bar-container">
              <div
                class="region-bar"
                :style="{ width: `${item.percentage}%` }"
              ></div>
            </div>
            <span class="region-count">{{ item.count }}人</span>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 无数据提示 -->
    <el-empty v-if="!loading && top20.length === 0" description="暂无评选数据，请先进行比赛模拟" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useSeasonStore } from '@/stores/useSeasonStore'
import { statsApi, awardsApi } from '@/api/tauri'
import type { AnnualAwardsData, Top20Player } from '@/api/tauri'
import SeasonSelector from '@/components/common/SeasonSelector.vue'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'

// ECharts
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { RadarChart } from 'echarts/charts'
import {
  TooltipComponent,
  LegendComponent,
  RadarComponent
} from 'echarts/components'
import VChart from 'vue-echarts'

use([CanvasRenderer, RadarChart, TooltipComponent, LegendComponent, RadarComponent])

const logger = createLogger('AnnualTop')

const router = useRouter()
const seasonStore = useSeasonStore()

const selectedSeason = ref(seasonStore.currentSeason)
const awardsData = ref<AnnualAwardsData | null>(null)
const loading = ref(false)
const recalculating = ref(false)

// Top20 数据
const top20 = computed(() => awardsData.value?.top20 ?? [])

// 获取颁奖数据
const fetchData = async () => {
  loading.value = true
  try {
    awardsData.value = await awardsApi.getAnnualAwardsData(selectedSeason.value)
  } catch (error) {
    logger.error('获取颁奖数据失败:', error)
    awardsData.value = null
  } finally {
    loading.value = false
  }
}

// 五维条形数据
const getDimBars = (player: Top20Player) => [
  { label: '影响', value: player.dimensions.impact_norm, color: '#3b82f6' },
  { label: '发挥', value: player.dimensions.performance_norm, color: '#10b981' },
  { label: '大赛', value: player.dimensions.big_stage_norm, color: '#ec4899' },
  { label: '稳定', value: player.dimensions.stability_norm, color: '#8b5cf6' },
  { label: '出场', value: player.dimensions.appearance_norm, color: '#f59e0b' },
  { label: '荣誉', value: player.dimensions.honor_norm, color: '#ef4444' },
]

const radarOption = computed(() => {
  if (top20.value.length < 3) return {}

  const colors = ['#fbbf24', '#94a3b8', '#cd7f32']
  const names = ['影响力', '发挥', '大赛表现', '稳定性', '出场', '荣誉']

  return {
    tooltip: {},
    legend: {
      data: top20.value.slice(0, 3).map(p => p.player_name),
      bottom: 0,
      textStyle: { color: '#6b7280', fontSize: 12 }
    },
    radar: {
      indicator: names.map(n => ({ name: n, max: 100 })),
      shape: 'polygon',
      splitNumber: 4,
      axisName: { color: '#6b7280', fontSize: 12 },
      splitLine: { lineStyle: { color: '#e5e7eb' } },
      splitArea: { areaStyle: { color: ['rgba(59,130,246,0.02)', 'rgba(59,130,246,0.05)'] } },
      axisLine: { lineStyle: { color: '#e5e7eb' } }
    },
    series: [{
      type: 'radar',
      data: top20.value.slice(0, 3).map((p, i) => ({
        name: p.player_name,
        value: [
          p.dimensions.impact_norm,
          p.dimensions.performance_norm,
          p.dimensions.big_stage_norm,
          p.dimensions.stability_norm,
          p.dimensions.appearance_norm,
          p.dimensions.honor_norm,
        ],
        areaStyle: { color: colors[i] + '33' },
        lineStyle: { color: colors[i], width: 2 },
        itemStyle: { color: colors[i] }
      }))
    }]
  }
})

// 三阵数据
const allProTiers = computed(() => {
  if (!awardsData.value) return []
  return [
    { label: '一阵', class: 'tier-gold', players: awardsData.value.all_pro_1st },
    { label: '二阵', class: 'tier-silver', players: awardsData.value.all_pro_2nd },
    { label: '三阵', class: 'tier-bronze', players: awardsData.value.all_pro_3rd },
  ]
})

// 位置分布
const positionDistribution = computed(() => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  const total = top20.value.length || 1
  return positions.map(pos => ({
    position: pos,
    count: top20.value.filter(r => r.position === pos).length,
    percentage: (top20.value.filter(r => r.position === pos).length / total) * 100
  }))
})

// 战队分布
const teamDistribution = computed(() => {
  const map = new Map<string, number>()
  const total = top20.value.length || 1
  top20.value.forEach(r => {
    map.set(r.team_name, (map.get(r.team_name) || 0) + 1)
  })
  return Array.from(map.entries())
    .map(([name, count]) => ({ name, count, percentage: (count / total) * 100 }))
    .sort((a, b) => b.count - a.count)
})

// 方法
const tableRef = ref()

const toggleExpand = (row: Top20Player) => {
  tableRef.value?.toggleRowExpansion(row)
}

const getWeightClass = (weight: number) => {
  if (weight >= 1.4) return 'weight-high'
  if (weight >= 1.1) return 'weight-mid'
  return 'weight-low'
}

const goToDetail = (row: Top20Player) => {
  router.push(`/data-center/player/${row.player_id}?season=S${selectedSeason.value}`)
}

const goToPlayerDetail = (playerId: number) => {
  router.push(`/data-center/player/${playerId}?season=S${selectedSeason.value}`)
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

const recalculateScores = async () => {
  recalculating.value = true
  try {
    const count = await statsApi.recalculateYearlyScores(selectedSeason.value)
    ElMessage.success(`已重新计算 ${count} 名选手的年度得分`)
    await fetchData()
  } catch (error) {
    logger.error('重新计算失败:', error)
    ElMessage.error('重新计算失败')
  } finally {
    recalculating.value = false
  }
}

onMounted(() => { fetchData() })
watch(selectedSeason, () => { fetchData() })
</script>

<style scoped lang="scss">
.annual-top {
  padding: 24px;
  min-height: 100%;

  .page-header {
    margin-bottom: 32px;

    .header-banner {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      padding: 40px;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      border-radius: 16px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);

      .header-actions {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 12px;
        flex-shrink: 0;
      }

      .banner-content {
        .banner-title {
          font-size: 32px;
          font-weight: 700;
          color: white;
          margin: 0 0 8px 0;
        }
        .banner-subtitle {
          font-size: 16px;
          color: rgba(255, 255, 255, 0.9);
          margin: 0 0 16px 0;
        }
        .scoring-rule { margin-bottom: 12px; }
        .scoring-detail {
          font-size: 14px;
          color: rgba(255, 255, 255, 0.7);
          .divider { margin: 0 12px; opacity: 0.5; }
        }
      }
    }
  }

  // Top3 + 雷达图
  .top-section {
    margin-bottom: 32px;

    .top-three {
      display: flex;
      justify-content: center;
      align-items: flex-end;
      gap: 24px;
      margin-bottom: 24px;

      .top-card {
        background: white;
        border-radius: 16px;
        padding: 24px;
        text-align: center;
        cursor: pointer;
        transition: transform 0.3s, box-shadow 0.3s;
        position: relative;
        box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);

        &:hover {
          transform: translateY(-8px);
          box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
        }

        .rank-badge {
          width: 40px; height: 40px; line-height: 40px;
          border-radius: 50%;
          margin: 0 auto 12px;
          font-weight: 700; font-size: 18px; color: white;
          background: #94a3b8;

          &.mvp {
            width: auto; padding: 0 16px;
            border-radius: 20px;
            background: linear-gradient(135deg, #fbbf24, #f59e0b);
            font-size: 14px;
          }
        }

        .player-name {
          font-size: 20px; font-weight: 700; color: #1f2937; margin-bottom: 8px;
        }
        .player-meta {
          display: flex; justify-content: center; align-items: center;
          gap: 8px; font-size: 14px; color: #6b7280; margin-bottom: 16px;
        }

        .dim-bars {
          text-align: left; margin-bottom: 16px;
          .dim-row {
            display: flex; align-items: center; gap: 6px; margin-bottom: 4px;
            .dim-label { width: 32px; font-size: 11px; color: #9ca3af; }
            .dim-bar-bg {
              flex: 1; height: 6px; background: #f3f4f6; border-radius: 3px; overflow: hidden;
              .dim-bar-fill { height: 100%; border-radius: 3px; transition: width 0.3s; }
            }
            .dim-val { width: 28px; font-size: 11px; color: #6b7280; text-align: right; }
          }
        }

        .total-score {
          padding: 12px; background: #f3f4f6; border-radius: 12px;
          .label { display: block; font-size: 12px; color: #6b7280; }
          .value { font-size: 28px; font-weight: 800; color: #7c3aed; }
        }

        &.gold {
          width: 280px;
          box-shadow: 0 8px 24px rgba(251, 191, 36, 0.3);
          border: 2px solid #fbbf24;
          .crown-icon {
            position: absolute; top: -20px; left: 50%; transform: translateX(-50%);
            width: 40px; height: 40px;
            background: linear-gradient(135deg, #fbbf24, #f59e0b);
            clip-path: polygon(50% 0%, 65% 35%, 100% 35%, 75% 55%, 85% 90%, 50% 70%, 15% 90%, 25% 55%, 0% 35%, 35% 35%);
            animation: float 2s ease-in-out infinite;
          }
          .rank-badge { background: linear-gradient(135deg, #fbbf24, #f59e0b); }
        }
        &.silver {
          width: 240px;
          box-shadow: 0 4px 16px rgba(192, 192, 192, 0.3);
          border: 2px solid #c0c0c0;
          .rank-badge { background: #94a3b8; }
        }
        &.bronze {
          width: 240px;
          box-shadow: 0 4px 16px rgba(205, 127, 50, 0.3);
          border: 2px solid #cd7f32;
          .rank-badge { background: #cd7f32; }
        }
      }
    }

    .radar-card {
      border-radius: 16px;
      .card-title { font-size: 16px; font-weight: 600; color: #1f2937; }
      .chart-container { height: 320px; }
      .chart { width: 100%; height: 100%; }
    }
  }

  @keyframes float {
    0%, 100% { transform: translateX(-50%) translateY(0); }
    50% { transform: translateX(-50%) translateY(-8px); }
  }

  // Top20 表格
  .ranking-list {
    margin-bottom: 24px;
    border-radius: 16px;

    .card-title { font-size: 18px; font-weight: 600; color: #1f2937; }
    .card-hint { font-size: 12px; color: #9ca3af; margin-left: 12px; }

    .rank-number {
      font-weight: 600; color: #6b7280;
      &.top-rank { color: #f59e0b; font-size: 16px; }
    }
    .player-link {
      font-weight: 600; color: #3b82f6; cursor: pointer;
      &:hover { text-decoration: underline; }
    }
    .games-value { color: #6b7280; }
    .bonus-value { color: #f59e0b; font-weight: 500; }
    .score-value { font-weight: 700; color: #7c3aed; font-size: 16px; }
    .positive-val { color: #10b981; font-weight: 600; }
    .negative-val { color: #ef4444; font-weight: 600; }

    .mini-bar-cell {
      display: flex; align-items: center; gap: 4px;
      span { font-size: 12px; color: #6b7280; min-width: 32px; text-align: right; }
      .mini-bar {
        flex: 1; height: 4px; background: #f3f4f6; border-radius: 2px; overflow: hidden; min-width: 30px;
        .mini-fill {
          height: 100%; border-radius: 2px;
          &.impact { background: #3b82f6; }
          &.perf { background: #10b981; }
          &.big-stage { background: #ec4899; }
          &.stab { background: #8b5cf6; }
        }
      }
    }

    .tags-cell {
      display: flex; gap: 4px; flex-wrap: wrap;
    }

    :deep(.el-table__row) {
      cursor: pointer;
      &:hover { background-color: #f0f9ff !important; }
    }

    .tournament-breakdown {
      padding: 16px 24px;
      background: #f8fafc;

      .breakdown-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 12px;
        padding-bottom: 8px;
        border-bottom: 1px solid #e5e7eb;
      }

      .breakdown-title {
        font-size: 14px;
        font-weight: 600;
        color: #374151;
      }

      .big-stage-badge {
        font-size: 12px;
        font-weight: 700;
        padding: 3px 10px;
        border-radius: 6px;
        &.positive { background: #d1fae5; color: #059669; }
        &.negative { background: #fee2e2; color: #dc2626; }
      }

      .breakdown-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: 10px;
      }

      .tournament-item {
        padding: 12px;
        background: white;
        border-radius: 8px;
        border: 1px solid #e5e7eb;

        .td-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;

          .td-name { font-size: 13px; font-weight: 600; color: #1f2937; }
          .td-weight {
            font-size: 11px; font-weight: 700; padding: 2px 6px; border-radius: 4px;
            &.weight-high { background: #fef3c7; color: #d97706; }
            &.weight-mid { background: #e0e7ff; color: #4f46e5; }
            &.weight-low { background: #f3f4f6; color: #6b7280; }
          }
        }

        .td-stats {
          display: flex; flex-wrap: wrap; gap: 10px;
          .td-stat {
            display: flex; flex-direction: column;
            .td-label { font-size: 11px; color: #9ca3af; }
            .td-value {
              font-size: 13px; font-weight: 600; color: #374151;
              &.positive-val { color: #10b981; }
              &.negative-val { color: #ef4444; }
              &.mvp-val { color: #f59e0b; }
            }
          }
        }
      }
    }

    .no-breakdown {
      padding: 16px; text-align: center; color: #9ca3af; font-size: 13px;
    }
  }

  // 最佳阵容 + 特别奖
  .bottom-section {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 24px;
    margin-bottom: 24px;

    .card-title { font-size: 16px; font-weight: 600; color: #1f2937; }

    .allpro-card {
      border-radius: 16px;

      .allpro-tiers {
        display: flex; flex-direction: column; gap: 16px;

        .tier-section {
          .tier-header {
            font-size: 14px; font-weight: 700; padding: 8px 16px;
            border-radius: 8px; margin-bottom: 8px; color: white;
            &.tier-gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); }
            &.tier-silver { background: linear-gradient(135deg, #94a3b8, #64748b); }
            &.tier-bronze { background: linear-gradient(135deg, #cd7f32, #a0522d); }
          }
          .tier-players {
            display: flex; flex-direction: column; gap: 4px;
            .allpro-player {
              display: flex; align-items: center; gap: 8px;
              padding: 8px 12px; border-radius: 8px; cursor: pointer;
              &:hover { background: #f9fafb; }
              .ap-name { font-weight: 600; color: #1f2937; flex: 1; }
              .ap-team { font-size: 13px; color: #6b7280; }
              .ap-score { font-weight: 700; color: #7c3aed; font-size: 14px; }
            }
          }
        }
      }
    }

    .special-card {
      border-radius: 16px;

      .special-awards {
        display: flex; flex-direction: column; gap: 16px;

        .special-item {
          padding: 16px; border-radius: 12px;
          background: #f9fafb; cursor: pointer;
          transition: background 0.2s;
          &:hover { background: #f0f9ff; }

          .special-icon {
            width: 36px; height: 36px; border-radius: 50%;
            margin-bottom: 8px;
            &.stable { background: linear-gradient(135deg, #8b5cf6, #6d28d9); }
            &.dominant { background: linear-gradient(135deg, #ef4444, #dc2626); }
            &.rookie { background: linear-gradient(135deg, #10b981, #059669); }
          }
          .special-label { font-size: 12px; color: #9ca3af; font-weight: 600; margin-bottom: 4px; }
          .special-name { font-size: 18px; font-weight: 700; color: #1f2937; margin-bottom: 4px; }
          .special-meta { font-size: 13px; color: #6b7280; margin-bottom: 8px; }
          .special-desc { font-size: 13px; color: #374151; line-height: 1.5; margin-bottom: 8px; }
          .special-tags { display: flex; gap: 4px; flex-wrap: wrap; }
        }
      }
    }
  }

  // 分布统计
  .distribution-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;

    .dist-card {
      border-radius: 16px;
      .card-title { font-size: 16px; font-weight: 600; color: #1f2937; }
    }

    .position-dist {
      display: flex;
      justify-content: space-around;
      align-items: flex-end;
      height: 160px;
      padding-top: 20px;

      .dist-item {
        display: flex; flex-direction: column; align-items: center; gap: 8px;
        .dist-bar-container {
          height: 100px; width: 40px; background: #f3f4f6;
          border-radius: 8px; display: flex; align-items: flex-end; overflow: hidden;
        }
        .dist-bar {
          width: 100%; border-radius: 8px 8px 0 0; transition: height 0.3s;
          &.top { background: linear-gradient(180deg, #ef4444, #dc2626); }
          &.jug { background: linear-gradient(180deg, #f59e0b, #d97706); }
          &.mid { background: linear-gradient(180deg, #3b82f6, #2563eb); }
          &.adc { background: linear-gradient(180deg, #10b981, #059669); }
          &.sup { background: linear-gradient(180deg, #8b5cf6, #7c3aed); }
        }
        .dist-label { font-size: 14px; font-weight: 600; color: #374151; }
        .dist-count { font-size: 12px; color: #6b7280; }
      }
    }

    .region-dist {
      .region-item {
        display: flex; align-items: center; gap: 12px; margin-bottom: 12px;
        &:last-child { margin-bottom: 0; }
        .region-name { width: 80px; font-size: 14px; font-weight: 500; color: #374151; }
        .region-bar-container {
          flex: 1; height: 24px; background: #f3f4f6; border-radius: 12px; overflow: hidden;
        }
        .region-bar {
          height: 100%; background: linear-gradient(90deg, #667eea, #764ba2);
          border-radius: 12px; transition: width 0.3s;
        }
        .region-count { width: 50px; font-size: 14px; color: #6b7280; text-align: right; }
      }
    }
  }
}

@media (max-width: 1024px) {
  .annual-top {
    .top-section .top-three {
      flex-direction: column; align-items: center;
      .top-card { width: 100% !important; max-width: 300px; }
    }
    .bottom-section { grid-template-columns: 1fr; }
    .distribution-stats { grid-template-columns: 1fr; }
  }
}
</style>
