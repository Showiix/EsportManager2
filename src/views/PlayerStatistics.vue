<template>
  <div class="player-statistics">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><DataLine /></el-icon>
          选手年度统计
        </h1>
        <p class="page-description">
          统计所有比赛中选手的影响力表现，分析年度最佳选手
        </p>
      </div>
      <div class="header-actions">
        <el-select v-model="selectedSeason" placeholder="选择赛季" style="width: 140px">
          <el-option label="S1赛季" value="S1" />
          <el-option label="S2赛季" value="S2" />
          <el-option label="S3赛季" value="S3" />
        </el-select>
        <el-select v-model="selectedRegion" placeholder="全部赛区" style="width: 120px" clearable>
          <el-option label="LPL" value="LPL" />
          <el-option label="LCK" value="LCK" />
          <el-option label="LEC" value="LEC" />
          <el-option label="LCS" value="LCS" />
        </el-select>
        <el-button type="primary" @click="refreshData" :loading="isLoading">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button type="warning" @click="syncData" :loading="isLoading">
          同步数据
        </el-button>
        <el-button type="danger" @click="clearAllData">
          <el-icon><Delete /></el-icon>
          清空数据
        </el-button>
      </div>
    </div>

    <!-- 统计概览 -->
    <div class="stats-overview">
      <el-card class="overview-card">
        <div class="stat-item">
          <div class="stat-value">{{ overviewStats.totalMatches }}</div>
          <div class="stat-label">总比赛场数</div>
        </div>
      </el-card>
      <el-card class="overview-card">
        <div class="stat-item">
          <div class="stat-value">{{ overviewStats.totalGames }}</div>
          <div class="stat-label">总小局数</div>
        </div>
      </el-card>
      <el-card class="overview-card">
        <div class="stat-item">
          <div class="stat-value">{{ overviewStats.upsetRate }}%</div>
          <div class="stat-label">爆冷率</div>
        </div>
      </el-card>
      <el-card class="overview-card">
        <div class="stat-item">
          <div class="stat-value">{{ overviewStats.playersTracked }}</div>
          <div class="stat-label">追踪选手数</div>
        </div>
      </el-card>
    </div>

    <!-- 年度 Top 排行榜 -->
    <el-card class="rankings-card">
      <template #header>
        <div class="card-header">
          <span>年度 Top 排行榜</span>
          <el-tag type="warning" size="small">
            年度Top = 影响力×70% + 冠军×30%
          </el-tag>
        </div>
      </template>

      <el-table
        :data="filteredRankings"
        stripe
        border
        style="width: 100%"
        :row-class-name="getRankRowClass"
      >
        <el-table-column label="排名" width="80" align="center">
          <template #default="{ $index }">
            <div class="rank-badge" :class="getRankClass($index)">
              {{ $index + 1 }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="player_name" label="选手" min-width="150">
          <template #default="{ row }">
            <div class="player-cell">
              <span class="player-name">{{ row.player_name }}</span>
              <el-tag size="small" type="info">{{ getPositionName(row.position) }}</el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="team_id" label="所属队伍" width="120">
          <template #default="{ row }">
            {{ row.team_id || '-' }}
          </template>
        </el-table-column>

        <el-table-column prop="region_id" label="赛区" width="80" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="row.region_id === 'LCK' ? 'danger' : 'primary'">
              {{ row.region_id || '-' }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="games_played" label="参与局数" width="100" align="center" />

        <el-table-column prop="avg_impact" label="平均影响力" width="120" align="center">
          <template #default="{ row }">
            <span :class="getImpactClass(row.avg_impact ?? 0)">
              {{ formatImpact(row.avg_impact ?? 0) }}
            </span>
          </template>
        </el-table-column>

        <el-table-column prop="champion_bonus" label="冠军加成" width="100" align="center">
          <template #default="{ row }">
            <span v-if="row.champion_bonus" class="champion-bonus">+{{ row.champion_bonus }}</span>
            <span v-else class="no-bonus">0</span>
          </template>
        </el-table-column>

        <el-table-column prop="yearly_top_score" label="年度Top得分" width="130" align="center">
          <template #default="{ row }">
            <span class="yearly-top-score" :class="getYearlyScoreClass(row.yearly_top_score ?? row.avg_impact ?? 0)">
              {{ formatScore(row.yearly_top_score ?? row.avg_impact ?? 0) }}
            </span>
          </template>
        </el-table-column>

        <el-table-column prop="consistency_score" label="稳定性" width="100" align="center">
          <template #default="{ row }">
            <el-progress
              :percentage="row.consistency_score ?? 0"
              :stroke-width="10"
              :show-text="false"
              :color="getConsistencyColor(row.consistency_score ?? 0)"
            />
            <span class="consistency-value">{{ (row.consistency_score ?? 0).toFixed(1) }}</span>
          </template>
        </el-table-column>
      </el-table>

      <el-empty v-if="filteredRankings.length === 0" description="暂无统计数据，请先进行比赛模拟" />
    </el-card>

    <!-- 位置分布图 -->
    <div class="position-stats">
      <el-card
        v-for="pos in positionStats"
        :key="pos.position"
        class="position-card"
      >
        <template #header>
          <div class="position-header">
            <span class="position-name">{{ pos.positionName }}</span>
            <el-tag :type="pos.tagType" size="small">{{ pos.count }}人</el-tag>
          </div>
        </template>
        <div class="top-player" v-if="pos.topPlayer">
          <div class="top-label">MVP</div>
          <div class="top-name">{{ pos.topPlayer.player_name }}</div>
          <div class="top-impact">
            <span :class="getYearlyScoreClass(pos.topPlayer.yearly_top_score ?? pos.topPlayer.avg_impact ?? 0)">
              {{ formatScore(pos.topPlayer.yearly_top_score ?? pos.topPlayer.avg_impact ?? 0) }}
            </span>
            <span class="impact-label">年度Top得分</span>
          </div>
        </div>
        <el-empty v-else description="暂无数据" :image-size="60" />
      </el-card>
    </div>

    <!-- 算法说明 -->
    <el-card class="algorithm-card">
      <template #header>
        <div class="algorithm-header">
          <span class="algorithm-title">Impact 年度 Top 算法说明</span>
          <el-tag type="info" size="small">v1.0</el-tag>
        </div>
      </template>
      <div class="algorithm-content">
        <div class="algorithm-section">
          <h4>1. 选手实际发挥计算</h4>
          <div class="formula">
            <code>稳定性标准差 σ = (100 - stability) / 10</code>
            <code>高斯噪声 noise = gaussianRandom() × σ</code>
            <code>实际能力 = ability + condition + noise</code>
            <code>钳位范围: [ability - 15, ability + 10]</code>
          </div>
          <p class="description">每局比赛中，选手的实际发挥会根据其稳定性产生波动。稳定性越高，波动越小。</p>
        </div>

        <div class="algorithm-section">
          <h4>2. 影响力分数 (Impact Score)</h4>
          <div class="formula">
            <code>队伍平均发挥 = Σ(5名选手实际能力) / 5</code>
            <code>影响力分数 = 个人实际发挥 - 队伍平均发挥</code>
          </div>
          <p class="description">影响力分数衡量选手相对于队伍平均水平的贡献。正值表示超越队伍平均，负值表示低于平均。</p>
        </div>

        <div class="algorithm-section">
          <h4>3. 冠军加成因素</h4>
          <div class="formula">
            <code>国际赛冠军 (MSI/Worlds): +3 分</code>
            <code>赛区冠军 (LPL/LCK/LEC/LCS): +1 分</code>
            <code>冠军加成 = 国际赛冠军数 × 3 + 赛区冠军数 × 1</code>
          </div>
          <p class="description">冠军荣誉是选手综合实力的重要体现，在年度Top计算中占30%权重。</p>
        </div>

        <div class="algorithm-section">
          <h4>4. 年度Top得分计算（加权）</h4>
          <div class="formula">
            <code>冠军加成 = 国际赛冠军 × 10 + 赛区冠军 × 5</code>
            <code>年度Top得分 = 平均影响力 × 70% + 冠军加成 × 30%</code>
            <code>稳定性评分 = 100 - (最佳发挥 - 最差发挥) × 2</code>
          </div>
          <p class="description">年度排名基于加权得分排序。平均影响力占70%权重，冠军因子占30%权重。</p>
        </div>

        <div class="algorithm-section">
          <h4>5. 胜负判定</h4>
          <div class="formula">
            <code>队伍发挥值 = 正态分布(队伍战力, σ=6)</code>
            <code>发挥值更高的队伍获胜该局</code>
          </div>
          <p class="description">即使战力较低的队伍也有机会爆冷获胜，这模拟了真实比赛中的不确定性。</p>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { DataLine, Refresh, Delete } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import type { PlayerPosition } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { statsApi, devApi, type PlayerRankingItem } from '@/api/tauri'

const playerStore = usePlayerStore()
const matchDetailStore = useMatchDetailStore()

// 状态
const selectedSeason = ref('S1')
const selectedRegion = ref('')
const isLoading = ref(false)

// 数据库数据 - 使用 ref 存储从数据库获取的数据
const rankingsData = ref<PlayerRankingItem[]>([])
const positionRankings = ref<Record<PlayerPosition, PlayerRankingItem[]>>({
  TOP: [],
  JUG: [],
  MID: [],
  ADC: [],
  SUP: []
})

// 从数据库加载排行榜数据
const loadRankingsFromDB = async () => {
  isLoading.value = true
  try {
    // 将 S1 转换为数字 1
    const seasonNum = parseInt(selectedSeason.value.replace('S', '')) || 1

    // 并行加载所有数据
    const [rankings, topRankings, jugRankings, midRankings, adcRankings, supRankings] = await Promise.all([
      statsApi.getSeasonImpactRanking(seasonNum, 100),
      statsApi.getPositionRanking(seasonNum, 'TOP', 100),
      statsApi.getPositionRanking(seasonNum, 'JUG', 100),
      statsApi.getPositionRanking(seasonNum, 'MID', 100),
      statsApi.getPositionRanking(seasonNum, 'ADC', 100),
      statsApi.getPositionRanking(seasonNum, 'SUP', 100)
    ])

    rankingsData.value = rankings
    positionRankings.value = {
      TOP: topRankings,
      JUG: jugRankings,
      MID: midRankings,
      ADC: adcRankings,
      SUP: supRankings
    }

    console.log('从数据库加载排行榜数据:', rankings.length, '条')
  } catch (e) {
    console.error('加载排行榜数据失败:', e)
    ElMessage.error('加载数据失败')
  } finally {
    isLoading.value = false
  }
}

// 计算属性 - 统计概览
const overviewStats = computed(() => {
  const upsetInfo = matchDetailStore.getUpsetRate(selectedSeason.value)

  return {
    totalMatches: matchDetailStore.totalMatches,
    totalGames: upsetInfo.total,
    upsetRate: upsetInfo.rate,
    playersTracked: rankingsData.value.length
  }
})

// 计算属性 - 过滤后的排行榜
const filteredRankings = computed(() => {
  console.log('排行榜数据:', rankingsData.value.length, '条')

  if (selectedRegion.value) {
    return rankingsData.value.filter(r => r.region_id === selectedRegion.value)
  }

  return rankingsData.value
})

// 计算属性 - 位置统计
const positionStats = computed(() => {
  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  const tagTypes = ['danger', 'warning', 'primary', 'success', 'info'] as const

  return positions.map((pos, idx) => {
    const rankings = positionRankings.value[pos]
    return {
      position: pos,
      positionName: POSITION_NAMES[pos],
      count: rankings.length,
      topPlayer: rankings[0] || null,
      tagType: tagTypes[idx]
    }
  })
})

// 监听赛季变化，重新加载数据
watch(selectedSeason, () => {
  loadRankingsFromDB()
})

// 同步数据库数据
const syncData = async () => {
  isLoading.value = true
  try {
    const seasonNum = parseInt(selectedSeason.value.replace('S', '')) || 1
    const result = await devApi.syncPlayerGamesPlayed(seasonNum)
    if (result.success) {
      ElMessage.success(`数据同步成功: ${result.data?.updated_count || 0} 条记录已更新`)
      await loadRankingsFromDB()
    } else {
      ElMessage.error(`同步失败: ${result.error}`)
    }
  } catch (e) {
    console.error('同步失败:', e)
    ElMessage.error('数据同步失败')
  } finally {
    isLoading.value = false
  }
}

// 方法
const refreshData = async () => {
  await loadRankingsFromDB()
  playerStore.loadFromStorage()
  matchDetailStore.loadFromStorage()
}

// 清空所有数据
const clearAllData = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有比赛统计数据吗？此操作不可恢复。',
      '清空数据',
      {
        confirmButtonText: '确定清空',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    matchDetailStore.clearAll()
    playerStore.clearAll()
    ElMessage.success('数据已清空')
  } catch (e) {
    // 用户取消
  }
}

const getPositionName = (position: PlayerPosition): string => {
  return POSITION_NAMES[position] || position
}

const formatImpact = (value: number | null | undefined): string => {
  if (value == null) return '0.0'
  if (value > 0) return `+${value.toFixed(1)}`
  return value.toFixed(1)
}

const getImpactClass = (value: number | null | undefined): string => {
  if (value == null) return ''
  if (value > 5) return 'very-positive'
  if (value > 0) return 'positive'
  if (value < -5) return 'very-negative'
  if (value < 0) return 'negative'
  return ''
}

const getRankClass = (index: number): string => {
  if (index === 0) return 'rank-gold'
  if (index === 1) return 'rank-silver'
  if (index === 2) return 'rank-bronze'
  return ''
}

const getRankRowClass = ({ rowIndex }: { rowIndex: number }): string => {
  if (rowIndex < 3) return 'top-rank-row'
  return ''
}

const getConsistencyColor = (score: number | null | undefined): string => {
  if (score == null) return '#909399'
  if (score >= 80) return '#67c23a'
  if (score >= 60) return '#e6a23c'
  return '#f56c6c'
}

const getYearlyScoreClass = (score: number): string => {
  if (score > 15) return 'score-excellent'
  if (score > 10) return 'score-good'
  if (score > 5) return 'score-average'
  return 'score-normal'
}

const formatScore = (value: number | null | undefined): string => {
  if (value == null) return '0.0'
  return value.toFixed(1)
}

// 生命周期
onMounted(() => {
  refreshData()
})
</script>

<style scoped lang="scss">
.player-statistics {
  padding: 24px;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content {
      .page-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 28px;
        font-weight: 700;
        margin: 0;
        color: #1f2937;
      }

      .page-description {
        margin: 8px 0 0 0;
        color: #6b7280;
        font-size: 14px;
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  .stats-overview {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;

    .overview-card {
      .stat-item {
        text-align: center;

        .stat-value {
          font-size: 32px;
          font-weight: 700;
          color: #409eff;
        }

        .stat-label {
          font-size: 14px;
          color: #909399;
          margin-top: 4px;
        }
      }
    }
  }

  .rankings-card {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
  }

  .player-cell {
    display: flex;
    align-items: center;
    gap: 8px;

    .player-name {
      font-weight: 500;
    }
  }

  .rank-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    font-weight: bold;
    background: #f5f7fa;
    color: #606266;

    &.rank-gold {
      background: linear-gradient(135deg, #ffd700, #ffb347);
      color: #1a1a2e;
    }

    &.rank-silver {
      background: linear-gradient(135deg, #c0c0c0, #a8a8a8);
      color: #1a1a2e;
    }

    &.rank-bronze {
      background: linear-gradient(135deg, #cd7f32, #b87333);
      color: white;
    }
  }

  .positive {
    color: #67c23a;
    font-weight: 500;
  }

  .negative {
    color: #f56c6c;
    font-weight: 500;
  }

  .very-positive {
    color: #67c23a;
    font-weight: bold;
    font-size: 16px;
  }

  .very-negative {
    color: #f56c6c;
    font-weight: bold;
  }

  .best-perf {
    color: #409eff;
    font-weight: 500;
  }

  .consistency-value {
    display: block;
    font-size: 12px;
    color: #909399;
    margin-top: 4px;
  }

  .champion-count {
    display: flex;
    justify-content: center;
    gap: 8px;
    align-items: center;

    .intl-title {
      display: inline-flex;
      align-items: center;
      gap: 2px;
      color: #ffd700;
      font-weight: bold;
      font-size: 14px;
    }

    .regional-title {
      display: inline-flex;
      align-items: center;
      gap: 2px;
      color: #c0c0c0;
      font-weight: 500;
      font-size: 14px;
    }

    .no-title {
      color: #c0c0c0;
    }
  }

  .champion-bonus {
    color: #f59e0b;
    font-weight: bold;
    font-size: 15px;
  }

  .no-bonus {
    color: #c0c0c0;
  }

  .yearly-top-score {
    font-weight: bold;
    font-size: 16px;

    &.score-excellent {
      color: #ffd700;
      text-shadow: 0 0 8px rgba(255, 215, 0, 0.5);
    }

    &.score-good {
      color: #67c23a;
    }

    &.score-average {
      color: #409eff;
    }

    &.score-normal {
      color: #606266;
    }
  }

  :deep(.top-rank-row) {
    background-color: #f0f9ff !important;
  }

  .position-stats {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 16px;

    .position-card {
      .position-header {
        display: flex;
        justify-content: space-between;
        align-items: center;

        .position-name {
          font-weight: 600;
        }
      }

      .top-player {
        text-align: center;

        .top-label {
          font-size: 12px;
          color: #f59e0b;
          font-weight: bold;
          margin-bottom: 8px;
        }

        .top-name {
          font-size: 18px;
          font-weight: 600;
          color: #303133;
          margin-bottom: 8px;
        }

        .top-impact {
          display: flex;
          flex-direction: column;
          gap: 4px;

          .impact-label {
            font-size: 12px;
            color: #909399;
          }
        }
      }
    }
  }

  .algorithm-card {
    margin-top: 24px;

    .algorithm-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .algorithm-title {
        font-weight: 600;
        font-size: 16px;
        color: #303133;
      }
    }

    .algorithm-content {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 24px;

      .algorithm-section {
        h4 {
          margin: 0 0 12px 0;
          font-size: 14px;
          color: #409eff;
          font-weight: 600;
        }

        .formula {
          background: #f5f7fa;
          border-radius: 8px;
          padding: 12px;
          margin-bottom: 8px;

          code {
            display: block;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 13px;
            color: #606266;
            line-height: 1.8;

            &::before {
              content: '› ';
              color: #409eff;
            }
          }
        }

        .description {
          margin: 0;
          font-size: 13px;
          color: #909399;
          line-height: 1.6;
        }
      }
    }
  }
}

@media (max-width: 1200px) {
  .player-statistics {
    .stats-overview {
      grid-template-columns: repeat(2, 1fr);
    }

    .position-stats {
      grid-template-columns: repeat(3, 1fr);
    }

    .algorithm-card .algorithm-content {
      grid-template-columns: 1fr;
    }
  }
}

@media (max-width: 768px) {
  .player-statistics {
    .page-header {
      flex-direction: column;
      gap: 16px;
    }

    .stats-overview {
      grid-template-columns: 1fr;
    }

    .position-stats {
      grid-template-columns: 1fr;
    }
  }
}
</style>
