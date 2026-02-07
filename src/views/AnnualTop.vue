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
              评选标准: 影响力(40%) + 出场(30%) + 冠军(30%)
            </el-tag>
          </div>
          <div class="scoring-detail">
            <span>国际赛冠军 +3分</span>
            <span class="divider">|</span>
            <span>赛区冠军 +1分</span>
            <span class="divider">|</span>
            <span>每10场 +1分</span>
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
          <SeasonSelector v-model="selectedSeason" width="100px" />
        </div>
      </div>
    </div>

    <!-- 前三名展示 -->
    <div class="top-three" v-if="topThree.length > 0">
      <!-- 第二名 -->
      <div class="top-card silver" v-if="topThree[1]" @click="goToDetail(topThree[1])">
        <div class="rank-medal">🥈</div>
        <div class="player-name">{{ topThree[1].playerName }}</div>
        <div class="player-meta">
          <el-tag :type="getPositionTagType(topThree[1].position)" size="small">
            {{ getPositionName(topThree[1].position) }}
          </el-tag>
          <span>{{ getTeamName(topThree[1].teamId) }}</span>
        </div>
        <div class="score-breakdown">
          <div class="score-item">
            <span class="label">影响力</span>
            <span class="value">{{ (topThree[1].avgImpact || 0).toFixed(1) }}</span>
          </div>
          <div class="score-item">
            <span class="label">出场</span>
            <span class="value games">{{ topThree[1].gamesPlayed || 0 }}场</span>
          </div>
          <div class="score-item">
            <span class="label">冠军加成</span>
            <span class="value bonus">+{{ (topThree[1].championBonus || 0).toFixed(1) }}</span>
          </div>
        </div>
        <div class="total-score">
          <span class="label">年度得分</span>
          <span class="value">{{ (topThree[1].yearlyTopScore || topThree[1].avgImpact || 0).toFixed(1) }}</span>
        </div>
        <div class="champion-badges">
          <span v-if="topThree[1].internationalTitles" class="badge intl">
            🏆×{{ topThree[1].internationalTitles }}
          </span>
          <span v-if="topThree[1].regionalTitles" class="badge regional">
            🏅×{{ topThree[1].regionalTitles }}
          </span>
        </div>
      </div>

      <!-- 第一名 -->
      <div class="top-card gold" v-if="topThree[0]" @click="goToDetail(topThree[0])">
        <div class="crown">👑</div>
        <div class="rank-medal">🥇</div>
        <div class="mvp-label">年度最佳</div>
        <div class="player-name">{{ topThree[0].playerName }}</div>
        <div class="player-meta">
          <el-tag :type="getPositionTagType(topThree[0].position)" size="small">
            {{ getPositionName(topThree[0].position) }}
          </el-tag>
          <span>{{ getTeamName(topThree[0].teamId) }}</span>
          <span v-if="topThree[0].regionId">· {{ topThree[0].regionId }}</span>
        </div>
        <div class="score-breakdown">
          <div class="score-item">
            <span class="label">影响力</span>
            <span class="value">{{ (topThree[0].avgImpact || 0).toFixed(1) }}</span>
          </div>
          <div class="score-item">
            <span class="label">出场</span>
            <span class="value games">{{ topThree[0].gamesPlayed || 0 }}场</span>
          </div>
          <div class="score-item">
            <span class="label">冠军加成</span>
            <span class="value bonus">+{{ (topThree[0].championBonus || 0).toFixed(1) }}</span>
          </div>
        </div>
        <div class="total-score">
          <span class="label">年度得分</span>
          <span class="value">{{ (topThree[0].yearlyTopScore || topThree[0].avgImpact || 0).toFixed(1) }}</span>
        </div>
        <div class="champion-badges">
          <span v-if="topThree[0].internationalTitles" class="badge intl">
            🏆×{{ topThree[0].internationalTitles }}
          </span>
          <span v-if="topThree[0].regionalTitles" class="badge regional">
            🏅×{{ topThree[0].regionalTitles }}
          </span>
        </div>
      </div>

      <!-- 第三名 -->
      <div class="top-card bronze" v-if="topThree[2]" @click="goToDetail(topThree[2])">
        <div class="rank-medal">🥉</div>
        <div class="player-name">{{ topThree[2].playerName }}</div>
        <div class="player-meta">
          <el-tag :type="getPositionTagType(topThree[2].position)" size="small">
            {{ getPositionName(topThree[2].position) }}
          </el-tag>
          <span>{{ getTeamName(topThree[2].teamId) }}</span>
        </div>
        <div class="score-breakdown">
          <div class="score-item">
            <span class="label">影响力</span>
            <span class="value">{{ (topThree[2].avgImpact || 0).toFixed(1) }}</span>
          </div>
          <div class="score-item">
            <span class="label">出场</span>
            <span class="value games">{{ topThree[2].gamesPlayed || 0 }}场</span>
          </div>
          <div class="score-item">
            <span class="label">冠军加成</span>
            <span class="value bonus">+{{ (topThree[2].championBonus || 0).toFixed(1) }}</span>
          </div>
        </div>
        <div class="total-score">
          <span class="label">年度得分</span>
          <span class="value">{{ (topThree[2].yearlyTopScore || topThree[2].avgImpact || 0).toFixed(1) }}</span>
        </div>
        <div class="champion-badges">
          <span v-if="topThree[2].internationalTitles" class="badge intl">
            🏆×{{ topThree[2].internationalTitles }}
          </span>
          <span v-if="topThree[2].regionalTitles" class="badge regional">
            🏅×{{ topThree[2].regionalTitles }}
          </span>
        </div>
      </div>
    </div>

    <!-- 4-20名列表 -->
    <el-card class="ranking-list" v-if="restRankings.length > 0">
      <template #header>
        <span class="card-title">第 4-20 名</span>
      </template>
      <el-table :data="restRankings" stripe style="width: 100%" @row-click="goToDetail">
        <el-table-column label="#" width="60" align="center">
          <template #default="{ $index }">
            <span class="rank-number">{{ $index + 4 }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="playerName" label="选手" min-width="140">
          <template #default="{ row }">
            <span class="player-link">{{ row.playerName }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="position" label="位置" width="90" align="center">
          <template #default="{ row }">
            <el-tag :type="getPositionTagType(row.position)" size="small">
              {{ getPositionName(row.position) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="teamId" label="战队" width="100">
          <template #default="{ row }">
            {{ getTeamName(row.teamId) }}
          </template>
        </el-table-column>

        <el-table-column prop="avgImpact" label="影响力" width="90" align="center">
          <template #default="{ row }">
            {{ (row.avgImpact || 0).toFixed(1) }}
          </template>
        </el-table-column>

        <el-table-column prop="gamesPlayed" label="出场" width="80" align="center">
          <template #default="{ row }">
            <span class="games-value">{{ row.gamesPlayed || 0 }}场</span>
          </template>
        </el-table-column>

        <el-table-column label="冠军加成" width="90" align="center">
          <template #default="{ row }">
            <span class="bonus-value">+{{ (row.championBonus || 0).toFixed(1) }}</span>
          </template>
        </el-table-column>

        <el-table-column label="荣誉" width="100" align="center">
          <template #default="{ row }">
            <div class="honor-cell">
              <span v-if="row.internationalTitles">🏆×{{ row.internationalTitles }}</span>
              <span v-if="row.regionalTitles">🏅×{{ row.regionalTitles }}</span>
              <span v-if="!row.internationalTitles && !row.regionalTitles">-</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="yearlyTopScore" label="得分" width="90" align="center">
          <template #default="{ row }">
            <span class="score-value">
              {{ (row.yearlyTopScore || row.avgImpact || 0).toFixed(1) }}
            </span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 入选分布统计 -->
    <div class="distribution-stats" v-if="rankings.length > 0">
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
          <span class="card-title">赛区分布</span>
        </template>
        <div class="region-dist">
          <div
            v-for="region in regionDistribution"
            :key="region.region"
            class="region-item"
          >
            <span class="region-name">{{ region.region || '未知' }}</span>
            <div class="region-bar-container">
              <div
                class="region-bar"
                :style="{ width: `${region.percentage}%` }"
              ></div>
            </div>
            <span class="region-count">{{ region.count }}人</span>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 无数据提示 -->
    <el-empty v-if="rankings.length === 0" description="暂无评选数据，请先进行比赛模拟" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { teamApi, statsApi } from '@/api/tauri'
import SeasonSelector from '@/components/common/SeasonSelector.vue'
import type { PlayerPosition, PlayerSeasonStats } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { createLogger } from '@/utils/logger'

const logger = createLogger('AnnualTop')

const router = useRouter()
const playerStore = usePlayerStore()

// 本地战队映射表
const teamsMap = ref<Map<number, string>>(new Map())

// 状态
const selectedSeason = ref(1)
const rankings = ref<PlayerSeasonStats[]>([])
const loading = ref(false)
const recalculating = ref(false)

// 异步获取排行数据
const fetchRankings = async () => {
  loading.value = true
  try {
    // 先加载战队数据
    if (teamsMap.value.size === 0) {
      const teams = await teamApi.getAllTeams()
      teams.forEach(t => {
        teamsMap.value.set(t.id, t.short_name || t.name)
      })
    }
    rankings.value = await playerStore.getSeasonImpactRanking(String(selectedSeason.value), 20)
  } catch (error) {
    logger.error('获取排行数据失败:', error)
    rankings.value = []
  } finally {
    loading.value = false
  }
}

// 前三名
const topThree = computed(() => {
  return rankings.value.slice(0, 3)
})

// 4-20名
const restRankings = computed(() => {
  return rankings.value.slice(3, 20)
})

// 位置分布统计
const positionDistribution = computed(() => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  const total = rankings.value.length || 1

  return positions.map(pos => {
    const count = rankings.value.filter(r => r.position === pos).length
    return {
      position: pos,
      count,
      percentage: (count / total) * 100
    }
  })
})

// 赛区分布统计
const regionDistribution = computed(() => {
  const regionMap = new Map<string, number>()
  const total = rankings.value.length || 1

  rankings.value.forEach(r => {
    const region = r.regionId || '未知'
    regionMap.set(region, (regionMap.get(region) || 0) + 1)
  })

  return Array.from(regionMap.entries())
    .map(([region, count]) => ({
      region,
      count,
      percentage: (count / total) * 100
    }))
    .sort((a, b) => b.count - a.count)
})

// 方法
const goToDetail = (row: PlayerSeasonStats) => {
  router.push(`/data-center/player/${row.playerId}?season=S${selectedSeason.value}`)
}

const getPositionName = (position: PlayerPosition): string => {
  return POSITION_NAMES[position] || position
}

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

const getTeamName = (teamId: string | number | null): string => {
  if (!teamId) return '-'
  const numId = Number(teamId)
  return teamsMap.value.get(numId) || String(teamId)
}

// 初始化
onMounted(() => {
  playerStore.loadFromStorage()
  fetchRankings()
})

// 重新计算年度得分
const recalculateScores = async () => {
  recalculating.value = true
  try {
    const count = await statsApi.recalculateYearlyScores(selectedSeason.value)
    ElMessage.success(`已重新计算 ${count} 名选手的年度得分`)
    // 刷新排名数据
    await fetchRankings()
  } catch (error) {
    logger.error('重新计算失败:', error)
    ElMessage.error('重新计算失败')
  } finally {
    recalculating.value = false
  }
}

// 监听赛季变化
watch(selectedSeason, () => {
  fetchRankings()
})
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

        .scoring-rule {
          margin-bottom: 12px;
        }

        .scoring-detail {
          font-size: 14px;
          color: rgba(255, 255, 255, 0.7);

          .divider {
            margin: 0 12px;
            opacity: 0.5;
          }
        }
      }
    }
  }

  .top-three {
    display: flex;
    justify-content: center;
    align-items: flex-end;
    gap: 24px;
    margin-bottom: 32px;

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

      .rank-medal {
        font-size: 48px;
        margin-bottom: 12px;
      }

      .player-name {
        font-size: 20px;
        font-weight: 700;
        color: #1f2937;
        margin-bottom: 8px;
      }

      .player-meta {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 8px;
        font-size: 14px;
        color: #6b7280;
        margin-bottom: 16px;
      }

      .score-breakdown {
        display: flex;
        justify-content: center;
        gap: 24px;
        margin-bottom: 16px;

        .score-item {
          .label {
            display: block;
            font-size: 12px;
            color: #9ca3af;
          }
          .value {
            font-size: 18px;
            font-weight: 600;
            color: #1f2937;

            &.bonus {
              color: #f59e0b;
            }
          }
        }
      }

      .total-score {
        padding: 12px;
        background: #f3f4f6;
        border-radius: 12px;
        margin-bottom: 12px;

        .label {
          display: block;
          font-size: 12px;
          color: #6b7280;
        }
        .value {
          font-size: 28px;
          font-weight: 800;
          color: #7c3aed;
        }
      }

      .champion-badges {
        display: flex;
        justify-content: center;
        gap: 8px;

        .badge {
          padding: 4px 12px;
          border-radius: 20px;
          font-size: 14px;

          &.intl {
            background: #fef3c7;
            color: #92400e;
          }
          &.regional {
            background: #e0e7ff;
            color: #3730a3;
          }
        }
      }

      &.gold {
        width: 280px;
        box-shadow: 0 8px 24px rgba(251, 191, 36, 0.3);
        border: 2px solid #fbbf24;

        .crown {
          position: absolute;
          top: -30px;
          left: 50%;
          transform: translateX(-50%);
          font-size: 48px;
          animation: float 2s ease-in-out infinite;
        }

        .mvp-label {
          position: absolute;
          top: 12px;
          right: 12px;
          background: linear-gradient(135deg, #fbbf24, #f59e0b);
          color: white;
          padding: 4px 12px;
          border-radius: 20px;
          font-size: 12px;
          font-weight: 600;
        }
      }

      &.silver {
        width: 240px;
        box-shadow: 0 4px 16px rgba(192, 192, 192, 0.3);
        border: 2px solid #c0c0c0;
      }

      &.bronze {
        width: 240px;
        box-shadow: 0 4px 16px rgba(205, 127, 50, 0.3);
        border: 2px solid #cd7f32;
      }
    }
  }

  @keyframes float {
    0%, 100% { transform: translateX(-50%) translateY(0); }
    50% { transform: translateX(-50%) translateY(-10px); }
  }

  .ranking-list {
    margin-bottom: 24px;
    border-radius: 16px;

    .card-title {
      font-size: 18px;
      font-weight: 600;
      color: #1f2937;
    }

    .rank-number {
      font-weight: 600;
      color: #6b7280;
    }

    .player-link {
      font-weight: 600;
      color: #3b82f6;
      cursor: pointer;

      &:hover {
        text-decoration: underline;
      }
    }

    .bonus-value {
      color: #f59e0b;
      font-weight: 500;
    }

    .honor-cell {
      display: flex;
      gap: 4px;
      justify-content: center;
      font-size: 14px;
    }

    .score-value {
      font-weight: 700;
      color: #7c3aed;
      font-size: 16px;
    }

    :deep(.el-table__row) {
      cursor: pointer;

      &:hover {
        background-color: #f0f9ff !important;
      }
    }
  }

  .distribution-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;

    .dist-card {
      border-radius: 16px;

      .card-title {
        font-size: 16px;
        font-weight: 600;
        color: #1f2937;
      }
    }

    .position-dist {
      display: flex;
      justify-content: space-around;
      align-items: flex-end;
      height: 160px;
      padding-top: 20px;

      .dist-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;

        .dist-bar-container {
          height: 100px;
          width: 40px;
          background: #f3f4f6;
          border-radius: 8px;
          display: flex;
          align-items: flex-end;
          overflow: hidden;
        }

        .dist-bar {
          width: 100%;
          border-radius: 8px 8px 0 0;
          transition: height 0.3s;

          &.top { background: linear-gradient(180deg, #ef4444, #dc2626); }
          &.jug { background: linear-gradient(180deg, #f59e0b, #d97706); }
          &.mid { background: linear-gradient(180deg, #3b82f6, #2563eb); }
          &.adc { background: linear-gradient(180deg, #10b981, #059669); }
          &.sup { background: linear-gradient(180deg, #8b5cf6, #7c3aed); }
        }

        .dist-label {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
        }

        .dist-count {
          font-size: 12px;
          color: #6b7280;
        }
      }
    }

    .region-dist {
      .region-item {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 12px;

        &:last-child {
          margin-bottom: 0;
        }

        .region-name {
          width: 60px;
          font-size: 14px;
          font-weight: 500;
          color: #374151;
        }

        .region-bar-container {
          flex: 1;
          height: 24px;
          background: #f3f4f6;
          border-radius: 12px;
          overflow: hidden;
        }

        .region-bar {
          height: 100%;
          background: linear-gradient(90deg, #667eea, #764ba2);
          border-radius: 12px;
          transition: width 0.3s;
        }

        .region-count {
          width: 50px;
          font-size: 14px;
          color: #6b7280;
          text-align: right;
        }
      }
    }
  }
}

@media (max-width: 1024px) {
  .annual-top {
    .top-three {
      flex-direction: column;
      align-items: center;

      .top-card {
        width: 100% !important;
        max-width: 300px;
      }
    }

    .distribution-stats {
      grid-template-columns: 1fr;
    }
  }
}
</style>
