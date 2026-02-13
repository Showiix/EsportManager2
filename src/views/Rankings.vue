<template>
  <div class="rankings-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>积分排名</h1>
        <p>全球战队积分与战力排名</p>
      </div>
    </div>

    <!-- 积分说明 -->
    <div class="notice-bar">
      年度积分来源：<strong>联赛季后赛、MSI、马德里大师赛、Claude洲际赛、世界赛、上海大师赛、ICP洲际赛</strong>。Super洲际赛不发放积分，它是对全年积分的最终奖励。
    </div>

    <!-- 统计概览 -->
    <div class="stats-bar">
      <div class="stat-item">
        <span class="stat-value">{{ currentData.length }}</span>
        <span class="stat-label">参赛队伍</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value highlight">{{ topPoints }}</span>
        <span class="stat-label">最高积分</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ averagePower.toFixed(1) }}</span>
        <span class="stat-label">平均战力</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">4</span>
        <span class="stat-label">赛区</span>
      </div>
    </div>

    <!-- 标签切换 -->
    <el-card class="rankings-card">
      <el-tabs v-model="activeTab" type="card">
        <el-tab-pane label="年度积分榜" name="annual">
          <!-- 筛选控件 -->
          <div class="filter-row">
            <el-select v-model="selectedRegion" placeholder="选择赛区" clearable>
              <el-option label="全部赛区" value="" />
              <el-option label="LPL 中国赛区" value="LPL" />
              <el-option label="LCK 韩国赛区" value="LCK" />
              <el-option label="LEC 欧洲赛区" value="LEC" />
              <el-option label="LCS 北美赛区" value="LCS" />
            </el-select>
            <SeasonSelector v-model="selectedSeason" />
          </div>

          <!-- 年度积分表格 -->
          <el-table :data="filteredAnnualRankings" class="rankings-table">
            <el-table-column prop="rank" label="排名" width="80" align="center">
              <template #default="{ row }">
                <div class="rank-cell">
                  <div v-if="row.rank <= 3" :class="['rank-medal', 'rank-' + row.rank]">
                    {{ row.rank }}
                  </div>
                  <span v-else class="rank-number">{{ row.rank }}</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="name" label="战队" min-width="200">
              <template #default="{ row }">
                <div class="team-cell">
                  <div class="team-avatar" :class="row.region.toLowerCase()">
                    {{ row.short }}
                  </div>
                  <div class="team-info">
                    <div class="team-name">{{ row.name }}</div>
                    <el-tag :type="getRegionTagType(row.region)" size="small">
                      {{ row.region }}
                    </el-tag>
                  </div>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="points" label="总积分" width="120" align="center">
              <template #default="{ row }">
                <span class="total-points">{{ row.points }}</span>
              </template>
            </el-table-column>

            <el-table-column label="积分明细" min-width="360">
              <template #default="{ row }">
                <div class="points-breakdown-tags" v-if="row.pointsBreakdown">
                  <!-- 最多显示4个标签 -->
                  <template v-for="(item, index) in getVisibleTags(row.pointsBreakdown)" :key="item.key">
                    <span :class="['point-tag', item.key]" v-if="index < 4">
                      {{ item.label }} +{{ item.points }}
                    </span>
                  </template>
                  <!-- 超过4个显示更多按钮 -->
                  <el-popover
                    v-if="getVisibleTags(row.pointsBreakdown).length > 4"
                    placement="bottom"
                    :width="200"
                    trigger="hover"
                  >
                    <template #reference>
                      <span class="point-tag more">+{{ getVisibleTags(row.pointsBreakdown).length - 4 }}</span>
                    </template>
                    <div class="more-tags-popover">
                      <template v-for="(item, index) in getVisibleTags(row.pointsBreakdown)" :key="item.key">
                        <span :class="['point-tag', item.key]" v-if="index >= 4">
                          {{ item.label }} +{{ item.points }}
                        </span>
                      </template>
                    </div>
                  </el-popover>
                </div>
                <span v-else class="no-points">暂无积分</span>
              </template>
            </el-table-column>

            <el-table-column label="" width="90" fixed="right">
              <template #default="{ row }">
                <button class="detail-btn" @click="showPointsDetail(row)">详情</button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <el-tab-pane label="战力排名榜" name="power">
          <!-- 筛选控件 -->
          <div class="filter-row">
            <el-select v-model="selectedRegion" placeholder="选择赛区" clearable>
              <el-option label="全部赛区" value="" />
              <el-option label="LPL 中国赛区" value="LPL" />
              <el-option label="LCK 韩国赛区" value="LCK" />
              <el-option label="LEC 欧洲赛区" value="LEC" />
              <el-option label="LCS 北美赛区" value="LCS" />
            </el-select>
          </div>

          <!-- 战力排名表格 -->
          <el-table :data="filteredPowerRankings" class="rankings-table">
            <el-table-column prop="rank" label="排名" width="80" align="center">
              <template #default="{ row }">
                <div class="rank-cell">
                  <div v-if="row.rank <= 3" :class="['rank-medal', 'rank-' + row.rank]">
                    {{ row.rank }}
                  </div>
                  <span v-else class="rank-number">{{ row.rank }}</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="name" label="战队" min-width="200">
              <template #default="{ row }">
                <div class="team-cell">
                  <div class="team-avatar" :class="row.region.toLowerCase()">
                    {{ row.short }}
                  </div>
                  <div class="team-info">
                    <div class="team-name">{{ row.name }}</div>
                    <el-tag :type="getRegionTagType(row.region)" size="small">
                      {{ row.region }}
                    </el-tag>
                  </div>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="power" label="战力值" width="200">
              <template #default="{ row }">
                <div class="power-cell">
                  <el-progress
                    :percentage="row.power"
                    :color="getPowerColor(row.power)"
                    :stroke-width="12"
                  />
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="change" label="变化" width="120" align="center">
              <template #default="{ row }">
                <span v-if="row.change > 0" class="change-up">
                  <el-icon><Top /></el-icon> {{ row.change }}
                </span>
                <span v-else-if="row.change < 0" class="change-down">
                  <el-icon><Bottom /></el-icon> {{ Math.abs(row.change) }}
                </span>
                <span v-else class="change-none">-</span>
              </template>
            </el-table-column>

            <el-table-column label="操作" width="100" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" link size="small" @click="viewTeam(row)">
                  详情
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 积分详情弹窗 -->
    <el-dialog
      v-model="pointsDetailVisible"
      :title="`${selectedTeam?.name} - 积分详情`"
      width="700px"
      class="points-detail-dialog"
    >
      <div class="points-detail-content" v-if="selectedTeam">
        <!-- 总积分概览 -->
        <div class="total-overview">
          <div class="total-points-display">
            <span class="points-number">{{ selectedTeam.points }}</span>
            <span class="points-unit">总积分</span>
          </div>
          <div class="team-badge">
            <div class="team-avatar xlarge" :class="selectedTeam.region.toLowerCase()">
              {{ selectedTeam.short }}
            </div>
            <div class="team-meta">
              <div class="team-name-large">{{ selectedTeam.name }}</div>
              <el-tag :type="getRegionTagType(selectedTeam.region)" effect="dark">
                {{ selectedTeam.region }}
              </el-tag>
            </div>
          </div>
        </div>

        <!-- 积分来源明细 - 卡片式布局 -->
        <div class="points-sources">
          <h3>积分来源明细</h3>

          <!-- 卡片网格 -->
          <div class="points-cards-grid">
            <!-- 春季赛 -->
            <div class="points-card spring" v-if="selectedTeamDetail.spring">
              <div class="card-icon">
                <el-icon :size="24"><Trophy /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">春季赛</div>
                <div class="card-points">+{{ selectedTeamDetail.spring.total }}</div>
                <div class="card-desc" v-if="selectedTeamDetail.spring.playoffs > 0">
                  季后赛 {{ selectedTeamDetail.spring.playoffsDesc }}
                </div>
                <div class="card-desc" v-if="selectedTeamDetail.spring.regular > 0">
                  常规赛 {{ selectedTeamDetail.spring.regularDesc }}
                </div>
              </div>
            </div>

            <!-- MSI -->
            <div class="points-card msi" v-if="selectedTeamDetail.msi && selectedTeamDetail.msi.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><Flag /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">MSI</div>
                <div class="card-points">+{{ selectedTeamDetail.msi.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.msi.desc }}</div>
              </div>
            </div>

            <!-- 马德里大师赛 -->
            <div class="points-card madrid" v-if="selectedTeamDetail.madrid && selectedTeamDetail.madrid.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><Location /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">马德里大师赛</div>
                <div class="card-points">+{{ selectedTeamDetail.madrid.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.madrid.desc }}</div>
              </div>
            </div>

            <!-- 夏季赛 -->
            <div class="points-card summer" v-if="selectedTeamDetail.summer">
              <div class="card-icon">
                <el-icon :size="24"><Sunny /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">夏季赛</div>
                <div class="card-points">+{{ selectedTeamDetail.summer.total }}</div>
                <div class="card-desc" v-if="selectedTeamDetail.summer.playoffs > 0">
                  季后赛 {{ selectedTeamDetail.summer.playoffsDesc }}
                </div>
                <div class="card-desc" v-if="selectedTeamDetail.summer.regular > 0">
                  常规赛 {{ selectedTeamDetail.summer.regularDesc }}
                </div>
              </div>
            </div>

            <!-- Claude洲际赛 -->
            <div class="points-card claude" v-if="selectedTeamDetail.claude && selectedTeamDetail.claude.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><Connection /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">Claude洲际赛</div>
                <div class="card-points">+{{ selectedTeamDetail.claude.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.claude.desc }}</div>
              </div>
            </div>

            <!-- 世界赛 -->
            <div class="points-card worlds" v-if="selectedTeamDetail.worlds && selectedTeamDetail.worlds.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><Trophy /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">S 世界赛</div>
                <div class="card-points">+{{ selectedTeamDetail.worlds.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.worlds.desc }}</div>
              </div>
            </div>

            <!-- 上海大师赛 -->
            <div class="points-card shanghai" v-if="selectedTeamDetail.shanghai && selectedTeamDetail.shanghai.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><OfficeBuilding /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">上海大师赛</div>
                <div class="card-points">+{{ selectedTeamDetail.shanghai.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.shanghai.desc }}</div>
              </div>
            </div>

            <!-- ICP洲际对抗赛 -->
            <div class="points-card icp" v-if="selectedTeamDetail.icp && selectedTeamDetail.icp.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><Promotion /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">ICP洲际赛</div>
                <div class="card-points">+{{ selectedTeamDetail.icp.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.icp.desc }}</div>
              </div>
            </div>

            <!-- Super洲际邀请赛 -->
            <div class="points-card super" v-if="selectedTeamDetail.super && selectedTeamDetail.super.total > 0">
              <div class="card-icon">
                <el-icon :size="24"><Star /></el-icon>
              </div>
              <div class="card-content">
                <div class="card-title">Super邀请赛</div>
                <div class="card-points">+{{ selectedTeamDetail.super.total }}</div>
                <div class="card-desc">{{ selectedTeamDetail.super.desc }}</div>
              </div>
            </div>
          </div>

          <!-- 无积分提示 -->
          <el-empty v-if="teamPointsDetails.length === 0" description="暂无积分记录">
            <template #default>
              <p style="color: #909399; font-size: 12px; margin: 0;">
                请完成赛事后查看积分明细
              </p>
            </template>
          </el-empty>
        </div>

        <!-- 积分规则说明 -->
        <div class="points-rules">
          <el-collapse>
            <el-collapse-item title="积分规则说明" name="rules">
              <div class="rules-content">
                <div class="rule-section">
                  <h4>联赛（春/夏季赛）</h4>
                  <p>季后赛：冠军12分、亚军10分、季军8分、第4名6分、5-8名各3分</p>
                </div>
                <div class="rule-section">
                  <h4>MSI / 上海大师赛</h4>
                  <p>冠军20分、亚军16分、季军12分、殿军8分、败者组第二轮6分、败者组第一轮4分</p>
                </div>
                <div class="rule-section">
                  <h4>马德里大师赛 / Claude洲际赛</h4>
                  <p>冠军20分、亚军16分、季军12分、殿军8分、半决赛败者6分、四分之一决赛败者4分、首轮败者2分</p>
                </div>
                <div class="rule-section">
                  <h4>S 世界赛</h4>
                  <p>冠军20分、亚军16分、季军12分、殿军8分、淘汰赛首轮6分、小组赛4分</p>
                </div>
                <div class="rule-section">
                  <h4>ICP 洲际对抗赛</h4>
                  <p>最强赛区参赛12分/未参赛6分、第二名参赛8分/未参赛4分、第三名参赛6分/未参赛3分、第四名参赛4分/未参赛2分</p>
                </div>
                <div class="rule-section">
                  <h4>Super 洲际邀请赛</h4>
                  <p>Super赛是年度积分的最终奖励，年度积分Top16的队伍获得参赛资格，<strong>不颁发积分</strong></p>
                </div>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </div>

      <template #footer>
        <el-button @click="pointsDetailVisible = false">关闭</el-button>
        <el-button type="primary" @click="goToTeamDetail">查看战队详情</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  Trophy,
  Top,
  Bottom,
  Flag,
  Location,
  Sunny,
  Connection,
  OfficeBuilding,
  Promotion,
  Star,
} from '@element-plus/icons-vue'
import { teamApi, queryApi, pointsApi } from '@/api/tauri'
import SeasonSelector from '@/components/common/SeasonSelector.vue'
import type { Team, Region, TeamAnnualPoints, AnnualPointsDetail } from '@/api/tauri'
import { createLogger } from '@/utils/logger'
import { useTimeStore } from '@/stores/useTimeStore'

const logger = createLogger('Rankings')

const router = useRouter()
const timeStore = useTimeStore()

// 状态
const activeTab = ref('annual')
const selectedRegion = ref('')
const selectedSeason = ref(timeStore.currentSeasonFromTime)
const pointsDetailVisible = ref(false)
const selectedTeam = ref<any>(null)
const loading = ref(false)
const teamPointsDetails = ref<AnnualPointsDetail[]>([])

// 缓存所有队伍的积分明细（用于表格显示）
const allTeamPointsCache = ref<Map<number, any>>(new Map())

// 赛区映射
const regionsMap = ref<Map<number, string>>(new Map())

// 数据 - 年度积分榜
const annualRankings = ref<any[]>([])

// 数据 - 战力排名榜
const powerRankings = ref<any[]>([])

// 加载积分排名数据（可按赛季重新加载）
async function loadPointsRankings(seasonId: number) {
  const pointsRankings = await pointsApi.getRankings(seasonId)

  // 为有积分的队伍加载积分明细
  const teamsWithPoints = pointsRankings.filter((item: TeamAnnualPoints) => item.total_points > 0)
  const detailsPromises = teamsWithPoints.map((item: TeamAnnualPoints) =>
    pointsApi.getTeamPoints(item.team_id, seasonId).then(details => ({ teamId: item.team_id, details }))
  )
  const allDetails = await Promise.all(detailsPromises)

  // 解析每个队伍的积分明细并缓存
  allTeamPointsCache.value.clear()
  allDetails.forEach(({ teamId, details }) => {
    const parsed = parseTeamPointsDetails(details)
    allTeamPointsCache.value.set(teamId, parsed)
  })

  // 使用积分排名API数据，并附加解析后的积分明细
  annualRankings.value = pointsRankings.map((item: TeamAnnualPoints) => {
    const cached = allTeamPointsCache.value.get(item.team_id)
    return {
      id: item.team_id,
      name: item.team_name,
      short: item.team_short_name || item.team_name.slice(0, 3),
      region: item.region_code,
      points: item.total_points,
      rank: item.rank,
      tournamentsCount: item.tournaments_count,
      pointsBreakdown: cached || null,
    }
  })
}

// 加载数据
onMounted(async () => {
  loading.value = true
  try {
    // 并行加载队伍、赛区和积分排名数据
    const [teams, regions] = await Promise.all([
      teamApi.getAllTeams(),
      queryApi.getAllRegions(),
    ])

    // 构建赛区ID到赛区代码的映射
    regions.forEach((region: Region) => {
      regionsMap.value.set(region.id, region.code)
    })

    // 加载积分排名
    await loadPointsRankings(selectedSeason.value)

    // 处理战力排名榜数据
    const powerData = teams
      .map((team: Team) => ({
        id: team.id,
        name: team.name,
        short: team.short_name || team.name.slice(0, 3),
        region: regionsMap.value.get(team.region_id) || 'LPL',
        power: Math.round(team.power_rating),
        change: 0, // 暂时没有历史数据对比
      }))
      .sort((a, b) => b.power - a.power)
      .map((team, index) => ({ ...team, rank: index + 1 }))

    powerRankings.value = powerData
  } catch (error) {
    logger.error('Failed to load rankings data:', error)
  } finally {
    loading.value = false
  }
})

// 赛季切换时重新加载积分数据
watch(selectedSeason, async (newSeason) => {
  loading.value = true
  try {
    await loadPointsRankings(newSeason)
  } catch (error) {
    logger.error('Failed to reload rankings for season:', newSeason, error)
  } finally {
    loading.value = false
  }
})

// 解析队伍积分明细（用于表格显示）
function parseTeamPointsDetails(details: AnnualPointsDetail[]) {
  const result: any = {
    spring: null,
    msi: null,
    madrid: null,
    summer: null,
    claude: null,
    worlds: null,
    shanghai: null,
    icp: null,
    super: null,
  }

  details.forEach(detail => {
    const tournamentType = detail.tournament_type || ''

    if (tournamentType.includes('SpringPlayoffs') || tournamentType.includes('SpringRegular')) {
      if (!result.spring) result.spring = { total: 0 }
      result.spring.total += detail.points
    } else if (tournamentType.includes('SummerPlayoffs') || tournamentType.includes('SummerRegular')) {
      if (!result.summer) result.summer = { total: 0 }
      result.summer.total += detail.points
    } else if (tournamentType.includes('Msi')) {
      result.msi = { total: detail.points }
    } else if (tournamentType.includes('MadridMasters')) {
      result.madrid = { total: detail.points }
    } else if (tournamentType.includes('ClaudeIntercontinental')) {
      result.claude = { total: detail.points }
    } else if (tournamentType.includes('WorldChampionship')) {
      result.worlds = { total: detail.points }
    } else if (tournamentType.includes('ShanghaiMasters')) {
      result.shanghai = { total: detail.points }
    } else if (tournamentType.includes('IcpIntercontinental')) {
      result.icp = { total: detail.points }
    } else if (tournamentType.includes('Super')) {
      result.super = { total: detail.points }
    }
  })

  return result
}

// 获取可见的标签列表（按赛季顺序）
function getVisibleTags(breakdown: any) {
  if (!breakdown) return []

  const tagConfig = [
    { key: 'spring', label: '春季赛' },
    { key: 'msi', label: 'MSI' },
    { key: 'madrid', label: '马德里' },
    { key: 'summer', label: '夏季赛' },
    { key: 'claude', label: 'Claude' },
    { key: 'worlds', label: '世界赛' },
    { key: 'shanghai', label: '上海' },
    { key: 'icp', label: 'ICP' },
    { key: 'super', label: 'Super' },
  ]

  return tagConfig
    .filter(config => breakdown[config.key] && breakdown[config.key].total > 0)
    .map(config => ({
      key: config.key,
      label: config.label,
      points: breakdown[config.key].total,
    }))
}

// 计算属性
const filteredAnnualRankings = computed(() => {
  if (!selectedRegion.value) return annualRankings.value
  return annualRankings.value.filter(t => t.region === selectedRegion.value)
})

const filteredPowerRankings = computed(() => {
  if (!selectedRegion.value) return powerRankings.value
  return powerRankings.value.filter(t => t.region === selectedRegion.value)
})

const currentData = computed(() => {
  return activeTab.value === 'annual' ? filteredAnnualRankings.value : filteredPowerRankings.value
})

const topPoints = computed(() => {
  if (annualRankings.value.length === 0) return 0
  return Math.max(...annualRankings.value.map(t => t.points))
})

const averagePower = computed(() => {
  if (powerRankings.value.length === 0) return 0
  return powerRankings.value.reduce((sum, t) => sum + t.power, 0) / powerRankings.value.length
})

// 积分详情数据（从API获取）
const selectedTeamDetail = computed(() => {
  if (!selectedTeam.value || teamPointsDetails.value.length === 0) {
    return {
      spring: null,
      msi: null,
      madrid: null,
      summer: null,
      claude: null,
      worlds: null,
      shanghai: null,
      icp: null,
      super: null,
    }
  }

  // 根据赛事类型分组积分明细
  const details = teamPointsDetails.value
  const result: any = {
    spring: null,
    msi: null,
    madrid: null,
    summer: null,
    claude: null,
    worlds: null,
    shanghai: null,
    icp: null,
    super: null,
  }

  details.forEach(detail => {
    const tournamentType = detail.tournament_type || ''
    const rankDesc = detail.final_rank ? `第${detail.final_rank}名` : ''

    if (tournamentType.includes('SpringPlayoffs') || tournamentType.includes('SpringRegular')) {
      if (!result.spring) {
        result.spring = { total: 0, regular: 0, regularDesc: '', playoffs: 0, playoffsDesc: '' }
      }
      if (tournamentType.includes('Playoffs')) {
        result.spring.playoffs += detail.points
        result.spring.playoffsDesc = `${rankDesc} +${detail.points}分`
      } else {
        result.spring.regular += detail.points
        result.spring.regularDesc = `${rankDesc} +${detail.points}分`
      }
      result.spring.total = result.spring.regular + result.spring.playoffs
    } else if (tournamentType.includes('SummerPlayoffs') || tournamentType.includes('SummerRegular')) {
      if (!result.summer) {
        result.summer = { total: 0, regular: 0, regularDesc: '', playoffs: 0, playoffsDesc: '' }
      }
      if (tournamentType.includes('Playoffs')) {
        result.summer.playoffs += detail.points
        result.summer.playoffsDesc = `${rankDesc} +${detail.points}分`
      } else {
        result.summer.regular += detail.points
        result.summer.regularDesc = `${rankDesc} +${detail.points}分`
      }
      result.summer.total = result.summer.regular + result.summer.playoffs
    } else if (tournamentType.includes('Msi')) {
      result.msi = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    } else if (tournamentType.includes('MadridMasters')) {
      result.madrid = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    } else if (tournamentType.includes('ClaudeIntercontinental')) {
      result.claude = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    } else if (tournamentType.includes('WorldChampionship')) {
      result.worlds = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    } else if (tournamentType.includes('ShanghaiMasters')) {
      result.shanghai = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    } else if (tournamentType.includes('IcpIntercontinental')) {
      result.icp = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    } else if (tournamentType.includes('Super')) {
      result.super = { total: detail.points, desc: `${rankDesc} +${detail.points}分` }
    }
  })

  return result
})

// 方法
const showPointsDetail = async (team: any) => {
  selectedTeam.value = team
  pointsDetailVisible.value = true

  // 加载队伍积分明细
  try {
    teamPointsDetails.value = await pointsApi.getTeamPoints(team.id, selectedSeason.value)
    logger.debug('[Rankings] Team points details for team', team.id, ':', teamPointsDetails.value)
    logger.debug('[Rankings] selectedTeamDetail computed:', selectedTeamDetail.value)
  } catch (error) {
    logger.error('Failed to load team points detail:', error)
    teamPointsDetails.value = []
  }
}

const goToTeamDetail = () => {
  if (selectedTeam.value) {
    router.push(`/teams/${selectedTeam.value.id}`)
    pointsDetailVisible.value = false
  }
}

const viewTeam = (team: any) => {
  router.push(`/teams/${team.id}`)
}

const getRegionTagType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getPowerColor = (power: number) => {
  if (power >= 85) return '#67c23a'
  if (power >= 75) return '#e6a23c'
  return '#f56c6c'
}
</script>

<style scoped>
.rankings-view {
  padding: 0;
}

/* ====== 页面标题 ====== */
.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
  letter-spacing: -0.3px;
}

.page-header p {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
}

/* ====== 积分说明条 ====== */
.notice-bar {
  padding: 10px 16px;
  background: #f8fafc;
  border-left: 3px solid #6366f1;
  border-radius: 0 8px 8px 0;
  font-size: 13px;
  color: #475569;
  margin-bottom: 16px;
  line-height: 1.6;
}

.notice-bar strong {
  color: #1e293b;
}

/* ====== 统计条 ====== */
.stats-bar {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 14px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  flex: 1;
  justify-content: center;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  font-variant-numeric: tabular-nums;
}

.stat-value.highlight {
  color: #6366f1;
}

.stat-label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 500;
}

.stat-divider {
  width: 1px;
  height: 24px;
  background: #e2e8f0;
  flex-shrink: 0;
}

/* ====== 排名主卡片 ====== */
.rankings-card {
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  box-shadow: none;
}

.rankings-card :deep(.el-card__body) {
  padding: 0;
}

.rankings-card :deep(.el-tabs__header) {
  margin: 0;
  padding: 0 20px;
  border-bottom: 1px solid #f1f5f9;
}

.rankings-card :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.rankings-card :deep(.el-tabs--card > .el-tabs__header .el-tabs__nav) {
  border: none;
}

.rankings-card :deep(.el-tabs--card > .el-tabs__header .el-tabs__item) {
  border: none;
  font-weight: 600;
  font-size: 13px;
  color: #94a3b8;
  padding: 0 16px;
  height: 44px;
  line-height: 44px;
  transition: color 0.2s;
}

.rankings-card :deep(.el-tabs--card > .el-tabs__header .el-tabs__item.is-active) {
  color: #0f172a;
  background: transparent;
  border-bottom: 2px solid #6366f1;
}

.rankings-card :deep(.el-tabs--card > .el-tabs__header .el-tabs__item:hover) {
  color: #475569;
}

.rankings-card :deep(.el-tabs__content) {
  padding: 16px 20px 20px;
}

/* ====== 筛选栏 ====== */
.filter-row {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
  align-items: center;
}

.filter-row .el-select {
  width: 160px;
}

/* ====== 表格 ====== */
.rankings-table {
  width: 100%;
}

.rankings-table :deep(.el-table__header th) {
  font-weight: 600;
  color: #94a3b8;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: transparent;
  border-bottom: 1px solid #f1f5f9;
  padding: 10px 0;
}

.rankings-table :deep(.el-table__body tr) {
  transition: background-color 0.15s;
}

.rankings-table :deep(.el-table__body tr td) {
  padding: 12px 0;
  border-bottom: 1px solid #f8fafc;
}

.rankings-table :deep(.el-table__body tr:hover > td) {
  background-color: #f8fafc !important;
}

.rankings-table :deep(.el-table__body tr:last-child td) {
  border-bottom: none;
}

/* 去掉默认 stripe 和 border */
.rankings-table :deep(.el-table__body tr.el-table__row--striped td) {
  background: transparent;
}

/* ====== 排名徽章 ====== */
.rank-cell {
  display: flex;
  justify-content: center;
  align-items: center;
}

.rank-medal {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  color: white;
}

.rank-1 {
  background: linear-gradient(135deg, #fbbf24, #f59e0b);
  box-shadow: 0 2px 8px rgba(245, 158, 11, 0.35);
}

.rank-2 {
  background: linear-gradient(135deg, #a1a1aa, #71717a);
  box-shadow: 0 2px 8px rgba(113, 113, 122, 0.3);
}

.rank-3 {
  background: linear-gradient(135deg, #d97706, #b45309);
  box-shadow: 0 2px 8px rgba(180, 83, 9, 0.3);
}

.rank-number {
  font-weight: 600;
  font-size: 14px;
  color: #94a3b8;
  font-variant-numeric: tabular-nums;
}

/* ====== 战队单元格 ====== */
.team-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.team-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 3px;
}

.team-name {
  font-weight: 600;
  color: #0f172a;
  font-size: 13px;
  line-height: 1.3;
}

/* ====== 总积分 ====== */
.total-points {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
  font-variant-numeric: tabular-nums;
}

/* ====== 积分明细标签 ====== */
.points-breakdown-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.point-tag {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  letter-spacing: 0.1px;
}

.point-tag.spring {
  background: #ecfdf5;
  color: #059669;
}

.point-tag.msi {
  background: #fef2f2;
  color: #dc2626;
}

.point-tag.madrid {
  background: #fffbeb;
  color: #d97706;
}

.point-tag.summer {
  background: #fff7ed;
  color: #ea580c;
}

.point-tag.claude {
  background: #eff6ff;
  color: #2563eb;
}

.point-tag.worlds {
  background: #f5f3ff;
  color: #7c3aed;
}

.point-tag.shanghai {
  background: #ecfeff;
  color: #0891b2;
}

.point-tag.icp {
  background: #fff7ed;
  color: #c2410c;
}

.point-tag.super {
  background: #fefce8;
  color: #ca8a04;
}

.point-tag.more {
  background: #f1f5f9;
  color: #64748b;
  cursor: pointer;
}

.point-tag.more:hover {
  background: #e2e8f0;
}

.more-tags-popover {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.no-points {
  color: #cbd5e1;
  font-size: 12px;
}

/* ====== 战力进度条 ====== */
.power-cell {
  padding-right: 16px;
}

.power-cell :deep(.el-progress-bar__outer) {
  border-radius: 4px;
  background: #f1f5f9;
}

.power-cell :deep(.el-progress-bar__inner) {
  border-radius: 4px;
}

/* ====== 排名变化 ====== */
.change-up {
  color: #10b981;
  font-weight: 600;
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-variant-numeric: tabular-nums;
}

.change-down {
  color: #ef4444;
  font-weight: 600;
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-variant-numeric: tabular-nums;
}

.change-none {
  color: #cbd5e1;
  font-size: 13px;
}

/* ====== 详情按钮 ====== */
.detail-btn {
  padding: 5px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #ffffff;
  color: #475569;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.detail-btn:hover {
  border-color: #6366f1;
  color: #6366f1;
  background: #f5f3ff;
}

/* ====== 积分详情弹窗 ====== */
.points-detail-dialog :deep(.el-dialog) {
  border-radius: 12px;
  overflow: hidden;
}

.points-detail-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid #f1f5f9;
  padding: 16px 24px;
}

.points-detail-dialog :deep(.el-dialog__title) {
  font-weight: 700;
  font-size: 16px;
  color: #0f172a;
}

.points-detail-dialog :deep(.el-dialog__body) {
  max-height: 60vh;
  overflow-y: auto;
  padding: 20px 24px;
}

.points-detail-content {
  padding: 0;
}

/* 总积分概览横幅 */
.total-overview {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  background: #0f172a;
  border-radius: 10px;
  margin-bottom: 20px;
}

.total-points-display {
  display: flex;
  flex-direction: column;
}

.total-points-display .points-number {
  font-size: 42px;
  font-weight: 800;
  color: white;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.total-points-display .points-unit {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 4px;
  font-weight: 500;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.team-badge {
  display: flex;
  align-items: center;
  gap: 14px;
}

.team-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.team-name-large {
  font-size: 18px;
  font-weight: 700;
  color: white;
}

/* ====== 积分来源明细 ====== */
.points-sources h3 {
  font-size: 13px;
  font-weight: 600;
  color: #94a3b8;
  margin: 0 0 12px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.points-cards-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.points-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 14px;
  border-radius: 8px;
  background: #f8fafc;
  border: 1px solid #f1f5f9;
  transition: border-color 0.15s;
}

.points-card:hover {
  border-color: #e2e8f0;
}

.points-card .card-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.points-card .card-content {
  flex: 1;
  min-width: 0;
}

.points-card .card-title {
  font-size: 11px;
  font-weight: 600;
  color: #94a3b8;
  margin-bottom: 2px;
  letter-spacing: 0.3px;
}

.points-card .card-points {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 2px;
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
}

.points-card .card-desc {
  font-size: 11px;
  color: #94a3b8;
  line-height: 1.4;
}

/* 赛事卡片颜色 - 只给 icon 上色 */
.points-card.spring .card-icon { background: #10b981; }
.points-card.spring .card-points { color: #059669; }

.points-card.msi .card-icon { background: #ef4444; }
.points-card.msi .card-points { color: #dc2626; }

.points-card.madrid .card-icon { background: #f59e0b; }
.points-card.madrid .card-points { color: #d97706; }

.points-card.summer .card-icon { background: #f97316; }
.points-card.summer .card-points { color: #ea580c; }

.points-card.claude .card-icon { background: #3b82f6; }
.points-card.claude .card-points { color: #2563eb; }

.points-card.worlds .card-icon { background: #8b5cf6; }
.points-card.worlds .card-points { color: #7c3aed; }

.points-card.shanghai .card-icon { background: #06b6d4; }
.points-card.shanghai .card-points { color: #0891b2; }

.points-card.icp .card-icon { background: #ea580c; }
.points-card.icp .card-points { color: #c2410c; }

.points-card.super .card-icon { background: #eab308; }
.points-card.super .card-points { color: #ca8a04; }

/* ====== 积分规则 ====== */
.points-rules {
  margin-top: 16px;
  border-top: 1px solid #f1f5f9;
  padding-top: 12px;
}

.points-rules :deep(.el-collapse) {
  border: none;
}

.points-rules :deep(.el-collapse-item__header) {
  font-weight: 600;
  color: #64748b;
  font-size: 13px;
  border: none;
  height: 36px;
}

.points-rules :deep(.el-collapse-item__wrap) {
  border: none;
}

.rules-content {
  padding: 4px 0;
}

.rule-section {
  margin-bottom: 10px;
}

.rule-section:last-child {
  margin-bottom: 0;
}

.rule-section h4 {
  font-size: 12px;
  font-weight: 600;
  color: #334155;
  margin: 0 0 2px 0;
}

.rule-section p {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
  line-height: 1.6;
}

/* ====== 响应式 ====== */
@media (max-width: 768px) {
  .filter-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .stats-bar {
    flex-wrap: wrap;
    gap: 8px;
  }

  .stat-divider {
    display: none;
  }

  .points-cards-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 480px) {
  .points-cards-grid {
    grid-template-columns: 1fr;
  }
}
</style>
