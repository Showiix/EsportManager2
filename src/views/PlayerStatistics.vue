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
          <el-option label="2024赛季" value="2024" />
          <el-option label="2023赛季" value="2023" />
        </el-select>
        <el-select v-model="selectedRegion" placeholder="全部赛区" style="width: 120px" clearable>
          <el-option label="LPL" value="LPL" />
          <el-option label="LCK" value="LCK" />
          <el-option label="LEC" value="LEC" />
          <el-option label="LCS" value="LCS" />
        </el-select>
        <el-button type="primary" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
        <el-button type="success" @click="generateMockData" :loading="generating">
          <el-icon><Plus /></el-icon>
          生成模拟数据
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

        <el-table-column prop="playerName" label="选手" min-width="150">
          <template #default="{ row }">
            <div class="player-cell">
              <span class="player-name">{{ row.playerName }}</span>
              <el-tag size="small" type="info">{{ getPositionName(row.position) }}</el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="teamId" label="所属队伍" width="120">
          <template #default="{ row }">
            {{ row.teamId?.split('-')[0] || '-' }}
          </template>
        </el-table-column>

        <el-table-column prop="regionId" label="赛区" width="80" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="row.regionId === 'LCK' ? 'danger' : 'primary'">
              {{ row.regionId || '-' }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="gamesPlayed" label="参与局数" width="100" align="center" />

        <el-table-column prop="avgImpact" label="平均影响力" width="120" align="center">
          <template #default="{ row }">
            <span :class="getImpactClass(row.avgImpact ?? 0)">
              {{ formatImpact(row.avgImpact ?? 0) }}
            </span>
          </template>
        </el-table-column>

        <el-table-column label="冠军次数" width="130" align="center">
          <template #default="{ row }">
            <div class="champion-count">
              <span v-if="row.internationalTitles" class="intl-title">
                <el-icon><Trophy /></el-icon>{{ row.internationalTitles }}
              </span>
              <span v-if="row.regionalTitles" class="regional-title">
                <el-icon><Medal /></el-icon>{{ row.regionalTitles }}
              </span>
              <span v-if="!row.internationalTitles && !row.regionalTitles" class="no-title">-</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="championBonus" label="冠军加成" width="100" align="center">
          <template #default="{ row }">
            <span v-if="row.championBonus" class="champion-bonus">+{{ row.championBonus }}</span>
            <span v-else class="no-bonus">0</span>
          </template>
        </el-table-column>

        <el-table-column prop="yearlyTopScore" label="年度Top得分" width="130" align="center">
          <template #default="{ row }">
            <span class="yearly-top-score" :class="getYearlyScoreClass(row.yearlyTopScore ?? row.avgImpact ?? 0)">
              {{ formatScore(row.yearlyTopScore ?? row.avgImpact ?? 0) }}
            </span>
          </template>
        </el-table-column>

        <el-table-column prop="consistencyScore" label="稳定性" width="100" align="center">
          <template #default="{ row }">
            <el-progress
              :percentage="row.consistencyScore ?? 0"
              :stroke-width="10"
              :show-text="false"
              :color="getConsistencyColor(row.consistencyScore ?? 0)"
            />
            <span class="consistency-value">{{ (row.consistencyScore ?? 0).toFixed(1) }}</span>
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
          <div class="top-name">{{ pos.topPlayer.playerName }}</div>
          <div class="top-impact">
            <span :class="getYearlyScoreClass(pos.topPlayer.yearlyTopScore ?? pos.topPlayer.avgImpact ?? 0)">
              {{ formatScore(pos.topPlayer.yearlyTopScore ?? pos.topPlayer.avgImpact ?? 0) }}
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
import { ref, computed, onMounted } from 'vue'
import { DataLine, Refresh, Plus, Trophy, Medal } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { usePlayerStore } from '@/stores/usePlayerStore'
import { useMatchDetailStore } from '@/stores/useMatchDetailStore'
import type { PlayerSeasonStats, PlayerPosition, Player } from '@/types/player'
import { POSITION_NAMES } from '@/types/player'
import { PowerEngine } from '@/engines'

const playerStore = usePlayerStore()
const matchDetailStore = useMatchDetailStore()

// 状态
const selectedSeason = ref('2024')
const selectedRegion = ref('')
const generating = ref(false)

// 计算属性 - 统计概览
const overviewStats = computed(() => {
  // 依赖 updateTrigger 确保响应式更新
  void playerStore.updateTrigger
  const matchCount = matchDetailStore.totalMatches

  const upsetInfo = matchDetailStore.getUpsetRate(selectedSeason.value)
  const rankings = playerStore.getSeasonImpactRanking(selectedSeason.value, 100)

  return {
    totalMatches: matchCount,
    totalGames: upsetInfo.total,
    upsetRate: upsetInfo.rate,
    playersTracked: rankings.length
  }
})

// 计算属性 - 过滤后的排行榜
const filteredRankings = computed(() => {
  // 依赖 updateTrigger 确保响应式更新
  void playerStore.updateTrigger

  const allRankings = playerStore.getSeasonImpactRanking(selectedSeason.value, 100)
  console.log('排行榜数据:', allRankings.length, '条')

  if (selectedRegion.value) {
    return allRankings.filter(r => r.regionId === selectedRegion.value)
  }

  return allRankings
})

// 计算属性 - 位置统计
const positionStats = computed(() => {
  // 依赖 updateTrigger 确保响应式更新
  void playerStore.updateTrigger

  const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  const tagTypes = ['danger', 'warning', 'primary', 'success', 'info'] as const

  return positions.map((pos, idx) => {
    const rankings = playerStore.getPositionRanking(pos, selectedSeason.value, 1)
    return {
      position: pos,
      positionName: POSITION_NAMES[pos],
      count: playerStore.getPositionRanking(pos, selectedSeason.value, 100).length,
      topPlayer: rankings[0] || null,
      tagType: tagTypes[idx]
    }
  })
})

// 方法
const refreshData = () => {
  playerStore.loadFromStorage()
  matchDetailStore.loadFromStorage()
}

// 生成模拟数据
const generateMockData = async () => {
  generating.value = true

  try {
    // 模拟队伍名称
    const teamNames = [
      { id: 'T1', name: 'T1', regionId: 'LCK' },
      { id: 'GEN', name: 'Gen.G', regionId: 'LCK' },
      { id: 'HLE', name: 'Hanwha Life', regionId: 'LCK' },
      { id: 'DK', name: 'Dplus KIA', regionId: 'LCK' },
      { id: 'BLG', name: 'Bilibili Gaming', regionId: 'LPL' },
      { id: 'TES', name: 'Top Esports', regionId: 'LPL' },
      { id: 'JDG', name: 'JD Gaming', regionId: 'LPL' },
      { id: 'WBG', name: 'Weibo Gaming', regionId: 'LPL' },
    ]

    const positions: PlayerPosition[] = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
    const playerNames: Record<PlayerPosition, string[]> = {
      TOP: ['Zeus', 'Kiin', 'Bin', '369', 'Breathe', 'Doran', 'Morgan', 'Rich'],
      JUG: ['Oner', 'Canyon', 'Tarzan', 'Kanavi', 'Peanut', 'Wei', 'Jiejie', 'Xun'],
      MID: ['Faker', 'Chovy', 'Zeka', 'Knight', 'Rookie', 'Scout', 'Yagao', 'Creme'],
      ADC: ['Gumayusi', 'Peyz', 'Viper', 'Elk', 'JackeyLove', 'Ruler', 'Light', 'Photic'],
      SUP: ['Keria', 'Lehends', 'Meiko', 'Ming', 'ON', 'Beryl', 'Missing', 'Crisp']
    }

    // 为每个队伍生成5名选手
    const generateTeamPlayers = (teamId: string, teamName: string): Player[] => {
      return positions.map((pos, idx) => {
        const names = playerNames[pos]
        const playerName = names[Math.floor(Math.random() * names.length)]
        return {
          id: `${teamId}-${pos}`,
          gameId: playerName,
          name: playerName,
          teamId: teamId,
          position: pos,
          regionId: teamId.startsWith('T') || teamId.startsWith('G') || teamId.startsWith('H') || teamId.startsWith('D') ? 'LCK' : 'LPL',
          ability: 70 + Math.floor(Math.random() * 25),
          potential: 80 + Math.floor(Math.random() * 15),
          stability: 60 + Math.floor(Math.random() * 35),
          condition: Math.floor(Math.random() * 11) - 5,
          age: 18 + Math.floor(Math.random() * 10),
          tag: Math.random() > 0.7 ? 'GENIUS' : Math.random() > 0.4 ? 'NORMAL' : 'ORDINARY'
        } as Player
      })
    }

    // 模拟10场比赛
    for (let i = 0; i < 10; i++) {
      // 随机选择两个队伍
      const shuffled = [...teamNames].sort(() => Math.random() - 0.5)
      const teamA = shuffled[0]
      const teamB = shuffled[1]

      const teamAPlayers = generateTeamPlayers(teamA.id, teamA.name)
      const teamBPlayers = generateTeamPlayers(teamB.id, teamB.name)

      // 模拟比赛
      const matchId = `mock-match-${Date.now()}-${i}`
      const matchDetail = PowerEngine.simulateMatch(
        teamA.id,
        teamA.name,
        teamAPlayers,
        teamB.id,
        teamB.name,
        teamBPlayers,
        Math.random() > 0.5 ? 3 : 5 // BO3 或 BO5
      )
      matchDetail.matchId = matchId

      // 保存比赛详情
      matchDetailStore.saveMatchDetail(matchId, matchDetail)

      // 记录选手表现
      matchDetail.games.forEach(game => {
        game.teamAPlayers.forEach(perf => {
          playerStore.recordPerformance(
            perf.playerId,
            perf.playerName,
            teamA.id,
            perf.position,
            perf.impactScore,
            perf.actualAbility,
            selectedSeason.value,
            teamA.regionId
          )
        })
        game.teamBPlayers.forEach(perf => {
          playerStore.recordPerformance(
            perf.playerId,
            perf.playerName,
            teamB.id,
            perf.position,
            perf.impactScore,
            perf.actualAbility,
            selectedSeason.value,
            teamB.regionId
          )
        })
      })

      // 添加延迟使UI更流畅
      await new Promise(resolve => setTimeout(resolve, 100))
    }

    ElMessage.success('已生成10场模拟比赛数据！')
    playerStore.saveToStorage()
    refreshData()
  } catch (error) {
    console.error('生成模拟数据失败:', error)
    ElMessage.error('生成模拟数据失败')
  } finally {
    generating.value = false
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
