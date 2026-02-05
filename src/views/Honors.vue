<template>
  <div class="honors-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>荣誉殿堂</h1>
        <p>记录所有赛事的冠军与荣誉</p>
      </div>
      <div class="header-actions">
        <el-select v-model="selectedSeason" placeholder="选择赛季">
          <el-option label="全部赛季" value="all" />
          <el-option label="S1 赛季" value="S1" />
          <el-option label="S2 赛季" value="S2" />
        </el-select>
      </div>
    </div>

    <!-- 荣誉殿堂导航入口 -->
    <div class="hall-navigation">
      <el-card class="nav-card international" @click="$router.push('/international-hall')">
        <div class="nav-content">
          <div class="nav-icon">🏆</div>
          <div class="nav-info">
            <h3>国际荣誉殿堂</h3>
            <p>记录所有国际赛事的冠军荣耀</p>
          </div>
          <div class="nav-arrow">→</div>
        </div>
      </el-card>
      <div class="sub-nav">
        <el-card class="sub-nav-card" @click="$router.push('/player-honor-rankings')">
          <div class="sub-nav-content">
            <span class="sub-icon">👤</span>
            <span>选手荣誉榜</span>
          </div>
        </el-card>
        <el-card class="sub-nav-card" @click="$router.push('/team-honor-rankings')">
          <div class="sub-nav-content">
            <span class="sub-icon">🏠</span>
            <span>战队荣誉榜</span>
          </div>
        </el-card>
      </div>
    </div>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon gold">
              <el-icon :size="28"><Trophy /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ totalTournaments }}</div>
              <div class="stat-label">已完成赛事</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="28"><Flag /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ internationalCount }}</div>
              <div class="stat-label">国际赛事</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><Medal /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ leagueCount }}</div>
              <div class="stat-label">联赛赛事</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="28"><Star /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ mvpCount }}</div>
              <div class="stat-label">MVP</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 赛事卡片展示 -->
    <el-card class="honors-card">
      <template #header>
        <div class="card-header">
          <h2>赛事荣誉</h2>
          <div class="header-filters">
            <el-radio-group v-model="tournamentFilter" size="default">
              <el-radio-button value="all">全部</el-radio-button>
              <el-radio-button value="international">国际赛</el-radio-button>
              <el-radio-button value="league">联赛</el-radio-button>
            </el-radio-group>
          </div>
        </div>
      </template>

      <el-empty v-if="filteredTournaments.length === 0" description="暂无赛事记录">
        <template #image>
          <div class="empty-icon">🏆</div>
        </template>
        <template #description>
          <p>完成赛事后，荣誉将被记录在此</p>
        </template>
      </el-empty>

      <div v-else class="tournament-grid">
        <div
          v-for="tournament in filteredTournaments"
          :key="tournament.id"
          class="tournament-card"
          :class="tournament.type"
          @click="openTournamentDetail(tournament)"
        >
          <!-- 赛事卡片头部 -->
          <div class="tournament-header" :class="tournament.type">
            <div class="tournament-badge">
              {{ tournament.type === 'international' ? '国际赛' : '联赛' }}
            </div>
            <div class="tournament-trophy">🏆</div>
          </div>

          <!-- 赛事信息 -->
          <div class="tournament-content">
            <h3 class="tournament-name">{{ tournament.name }}</h3>
            <p class="tournament-season">{{ tournament.season }}</p>

            <!-- 冠军信息 -->
            <div class="champion-info">
              <div class="champion-label">{{ tournament.isIcp ? '最强赛区' : '冠军' }}</div>
              <div class="champion-team">
                <div class="team-avatar" :class="tournament.champion.region.toLowerCase()">
                  {{ tournament.champion.short }}
                </div>
                <div class="team-details">
                  <div class="team-name">{{ tournament.champion.name }}</div>
                  <div class="team-meta">
                    <el-tag :type="getRegionTagType(tournament.champion.region)" size="small">
                      {{ tournament.champion.region }}
                    </el-tag>
                    <span v-if="tournament.champion.teamCount" class="team-count">
                      {{ tournament.champion.teamCount }}支队伍
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 点击提示 -->
            <div class="click-hint">
              <el-icon><ArrowRight /></el-icon>
              <span>点击查看详情</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 按战队统计 -->
    <el-card class="team-stats-card">
      <template #header>
        <div class="card-header">
          <h2>战队荣誉榜</h2>
        </div>
      </template>

      <el-table :data="teamHonors" stripe class="honors-table">
        <el-table-column prop="rank" label="排名" width="80" align="center">
          <template #default="{ $index }">
            <div class="rank-badge" :class="getRankClass($index + 1)">
              {{ $index + 1 }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="team" label="战队" min-width="200">
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

        <el-table-column label="冠军" width="100" align="center">
          <template #default="{ row }">
            <span class="trophy-count gold">{{ row.champions }}</span>
          </template>
        </el-table-column>

        <el-table-column label="亚军" width="100" align="center">
          <template #default="{ row }">
            <span class="trophy-count silver">{{ row.runnerUps }}</span>
          </template>
        </el-table-column>

        <el-table-column label="四强" width="100" align="center">
          <template #default="{ row }">
            <span class="trophy-count bronze">{{ row.topFour }}</span>
          </template>
        </el-table-column>

        <el-table-column label="总积分" width="120" align="center">
          <template #default="{ row }">
            <span class="total-points">{{ row.points }}</span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 赛事详情弹窗 -->
    <el-dialog
      v-model="showDetailDialog"
      :title="selectedTournament?.name"
      width="600px"
      class="tournament-detail-dialog"
    >
      <div v-if="selectedTournament" class="tournament-detail">
        <!-- 赛事信息头部 -->
        <div class="detail-header" :class="selectedTournament.type">
          <div class="detail-badge">
            {{ selectedTournament.type === 'international' ? '国际赛' : '联赛' }}
          </div>
          <div class="detail-season">{{ selectedTournament.season }}</div>
        </div>

        <!-- 前四名展示 -->
        <div class="placements">
          <!-- 冠军/最强赛区 -->
          <div class="placement-item champion">
            <div class="placement-rank">
              <span class="rank-icon">🏆</span>
              <span class="rank-text">{{ selectedTournament.isIcp ? '最强赛区' : '冠军' }}</span>
            </div>
            <div class="placement-team">
              <div class="team-avatar large" :class="selectedTournament.champion.region.toLowerCase()">
                {{ selectedTournament.champion.short }}
              </div>
              <div class="team-details">
                <div class="team-name">{{ selectedTournament.champion.name }}</div>
                <div class="team-meta-row">
                  <el-tag :type="getRegionTagType(selectedTournament.champion.region)" size="small">
                    {{ selectedTournament.champion.region }}
                  </el-tag>
                  <span v-if="selectedTournament.champion.teamCount" class="team-count-badge">
                    {{ selectedTournament.champion.teamCount }}支队伍
                  </span>
                </div>
              </div>
            </div>
            <div v-if="selectedTournament.mvp" class="mvp-info">
              <el-icon><Star /></el-icon>
              <span>MVP: {{ selectedTournament.mvp }}</span>
            </div>
          </div>

          <!-- 亚军/第二赛区 -->
          <div class="placement-item runner-up">
            <div class="placement-rank">
              <span class="rank-icon">🥈</span>
              <span class="rank-text">{{ selectedTournament.isIcp ? '第二赛区' : '亚军' }}</span>
            </div>
            <div class="placement-team">
              <div class="team-avatar" :class="selectedTournament.runnerUp.region.toLowerCase()">
                {{ selectedTournament.runnerUp.short }}
              </div>
              <div class="team-details">
                <div class="team-name">{{ selectedTournament.runnerUp.name }}</div>
                <div class="team-meta-row">
                  <el-tag :type="getRegionTagType(selectedTournament.runnerUp.region)" size="small">
                    {{ selectedTournament.runnerUp.region }}
                  </el-tag>
                  <span v-if="selectedTournament.runnerUp.teamCount" class="team-count-badge">
                    {{ selectedTournament.runnerUp.teamCount }}支队伍
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 季军/第三赛区 -->
          <div class="placement-item third">
            <div class="placement-rank">
              <span class="rank-icon">🥉</span>
              <span class="rank-text">{{ selectedTournament.isIcp ? '第三赛区' : '季军' }}</span>
            </div>
            <div class="placement-team">
              <div class="team-avatar" :class="selectedTournament.third.region.toLowerCase()">
                {{ selectedTournament.third.short }}
              </div>
              <div class="team-details">
                <div class="team-name">{{ selectedTournament.third.name }}</div>
                <div class="team-meta-row">
                  <el-tag :type="getRegionTagType(selectedTournament.third.region)" size="small">
                    {{ selectedTournament.third.region }}
                  </el-tag>
                  <span v-if="selectedTournament.third.teamCount" class="team-count-badge">
                    {{ selectedTournament.third.teamCount }}支队伍
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 殿军/第四赛区 -->
          <div class="placement-item fourth">
            <div class="placement-rank">
              <span class="rank-icon">4</span>
              <span class="rank-text">{{ selectedTournament.isIcp ? '第四赛区' : '殿军' }}</span>
            </div>
            <div class="placement-team">
              <div class="team-avatar" :class="selectedTournament.fourth.region.toLowerCase()">
                {{ selectedTournament.fourth.short }}
              </div>
              <div class="team-details">
                <div class="team-name">{{ selectedTournament.fourth.name }}</div>
                <div class="team-meta-row">
                  <el-tag :type="getRegionTagType(selectedTournament.fourth.region)" size="small">
                    {{ selectedTournament.fourth.region }}
                  </el-tag>
                  <span v-if="selectedTournament.fourth.teamCount" class="team-count-badge">
                    {{ selectedTournament.fourth.teamCount }}支队伍
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="showDetailDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Trophy, Medal, Star, Flag, ArrowRight } from '@element-plus/icons-vue'
import { honorApi, teamApi, queryApi } from '@/api/tauri'
import type { Team } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('Honors')

// 类型定义
interface TeamInfo {
  name: string
  short: string
  region: string
  isRegionBattle?: boolean  // 是否是赛区对抗赛（ICP）
  teamCount?: number        // 赛区队伍数量
}

interface Tournament {
  id: number
  name: string
  season: string
  type: 'international' | 'league'
  isIcp: boolean            // 是否是ICP赛区对抗赛
  champion: TeamInfo
  runnerUp: TeamInfo
  third: TeamInfo
  fourth: TeamInfo
  mvp?: string
}

interface TeamHonor {
  name: string
  short: string
  region: string
  champions: number
  runnerUps: number
  topFour: number
  points: number
}

// 状态
const selectedSeason = ref('all')
const tournamentFilter = ref('all')
const showDetailDialog = ref(false)
const selectedTournament = ref<Tournament | null>(null)
const loading = ref(false)

// 数据
const tournaments = ref<Tournament[]>([])
const teamHonors = ref<TeamHonor[]>([])

// 缓存的队伍和赛区数据
const teamsMap = ref<Map<number, Team>>(new Map())
const regionsMap = ref<Map<number, string>>(new Map())

// 加载数据
onMounted(async () => {
  loading.value = true
  try {
    // 并行加载所有队伍和赛区信息
    const [teams, regions] = await Promise.all([
      teamApi.getAllTeams(),
      queryApi.getAllRegions(),
    ])

    // 构建队伍ID到队伍信息的映射
    teams.forEach((team: Team) => {
      teamsMap.value.set(team.id, team)
    })

    // 构建赛区ID到赛区代码的映射
    regions.forEach((region: { id: number; code: string }) => {
      regionsMap.value.set(region.id, region.code)
    })

    // 加载荣誉数据
    await loadHonors()
  } catch (error) {
    logger.error('Failed to load data:', error)
  } finally {
    loading.value = false
  }
})

// 加载荣誉数据
async function loadHonors() {
  try {
    // 获取所有冠军和MVP记录
    const [champions, mvps] = await Promise.all([
      honorApi.getAllChampions(),
      honorApi.getAllMvps(),
    ])

    // 按赛事ID分组荣誉记录
    const tournamentHonorsMap = new Map<number, {
      champions: any[]      // 可能有多个冠军队伍（ICP赛区对抗赛）
      runnerUps: any[]
      thirds: any[]
      fourths: any[]
      mvp?: string
      tournamentName: string
      tournamentType: string
      seasonId: number
    }>()

    // 处理冠军、亚军、季军、殿军
    champions.forEach((honor: any) => {
      const tournamentId = honor.tournament_id
      if (!tournamentHonorsMap.has(tournamentId)) {
        tournamentHonorsMap.set(tournamentId, {
          champions: [],
          runnerUps: [],
          thirds: [],
          fourths: [],
          tournamentName: honor.tournament_name,
          tournamentType: honor.tournament_type,
          seasonId: honor.season_id,
        })
      }
      const entry = tournamentHonorsMap.get(tournamentId)!

      const teamInfo = getTeamInfo(honor.team_id, honor.team_name)

      if (honor.honor_type === 'TEAM_CHAMPION') {
        entry.champions.push(teamInfo)
      } else if (honor.honor_type === 'TEAM_RUNNER_UP') {
        entry.runnerUps.push(teamInfo)
      } else if (honor.honor_type === 'TEAM_THIRD') {
        entry.thirds.push(teamInfo)
      } else if (honor.honor_type === 'TEAM_FOURTH') {
        entry.fourths.push(teamInfo)
      }
    })

    // 处理MVP
    mvps.forEach((honor: any) => {
      const tournamentId = honor.tournament_id
      if (tournamentHonorsMap.has(tournamentId) && honor.honor_type === 'TOURNAMENT_MVP') {
        tournamentHonorsMap.get(tournamentId)!.mvp = honor.player_name
      }
    })

    // 转换为Tournament数组
    const tournamentList: Tournament[] = []
    tournamentHonorsMap.forEach((data, id) => {
      if (data.champions.length > 0) {
        const defaultTeam: TeamInfo = { name: '未知', short: '?', region: 'LPL' }
        const isIcp = isIcpType(data.tournamentType)

        // 对于ICP赛区对抗赛，聚合同赛区的队伍显示赛区名
        const getPlacementInfo = (teams: TeamInfo[]): TeamInfo => {
          if (teams.length === 0) return defaultTeam
          if (isIcp && teams.length > 1) {
            const region = teams[0].region
            return {
              name: `${region}赛区`,
              short: region,
              region: region,
              isRegionBattle: true,
              teamCount: teams.length,
            }
          }
          return teams[0]
        }

        tournamentList.push({
          id,
          name: data.tournamentName,
          season: `S${data.seasonId}`,
          type: isInternationalType(data.tournamentType) ? 'international' : 'league',
          isIcp,
          champion: getPlacementInfo(data.champions),
          runnerUp: getPlacementInfo(data.runnerUps),
          third: getPlacementInfo(data.thirds),
          fourth: getPlacementInfo(data.fourths),
          mvp: data.mvp,
        })
      }
    })

    tournaments.value = tournamentList

    // 计算战队荣誉统计
    const teamStatsMap = new Map<number, TeamHonor>()
    champions.forEach((honor: any) => {
      if (!honor.team_id) return

      if (!teamStatsMap.has(honor.team_id)) {
        const teamInfo = getTeamInfo(honor.team_id, honor.team_name)
        teamStatsMap.set(honor.team_id, {
          name: teamInfo.name,
          short: teamInfo.short,
          region: teamInfo.region,
          champions: 0,
          runnerUps: 0,
          topFour: 0,
          points: 0,
        })
      }

      const stats = teamStatsMap.get(honor.team_id)!
      if (honor.honor_type === 'TEAM_CHAMPION') {
        stats.champions++
        stats.points += 5
      } else if (honor.honor_type === 'TEAM_RUNNER_UP') {
        stats.runnerUps++
        stats.points += 3
      } else if (honor.honor_type === 'TEAM_THIRD' || honor.honor_type === 'TEAM_FOURTH') {
        stats.topFour++
        stats.points += 1
      }
    })

    // 排序并设置
    teamHonors.value = Array.from(teamStatsMap.values())
      .sort((a, b) => b.points - a.points)
  } catch (error) {
    logger.error('Failed to load honors:', error)
  }
}

// 获取队伍信息
function getTeamInfo(teamId: number | undefined, teamName: string | undefined): TeamInfo {
  if (teamId && teamsMap.value.has(teamId)) {
    const team = teamsMap.value.get(teamId)!
    const regionCode = regionsMap.value.get(team.region_id) || 'LPL'
    return {
      name: team.name,
      short: team.short_name || team.name.slice(0, 3),
      region: regionCode,
    }
  }
  return {
    name: teamName || '未知',
    short: teamName ? teamName.slice(0, 3) : '?',
    region: 'LPL',
  }
}

// 判断是否是国际赛事
// 注意：后端存储格式是 PascalCase（如 MadridMasters），需要处理多种格式
function isInternationalType(tournamentType: string): boolean {
  const type = tournamentType.toLowerCase()
  // 国际赛事类型列表（支持多种格式）
  const internationalTypes = [
    'msi',
    'worlds',
    'worldchampionship',
    'world_championship',
    'madridmasters',
    'madrid_masters',
    'shanghaimasters',
    'shanghai_masters',
    'super',
    'superintercontinental',
    'super_intercontinental',
    'claudeintercontinental',
    'claude_intercontinental',
    'icpintercontinental',
    'icp_intercontinental',
  ]
  return internationalTypes.includes(type)
}

// 判断是否是ICP赛区对抗赛（赛区vs赛区的特殊赛制）
function isIcpType(tournamentType: string): boolean {
  const type = tournamentType.toLowerCase()
  // 只有 IcpIntercontinental 是赛区对抗赛，其他洲际赛（Claude、Super）不是
  return type === 'icpintercontinental' || type === 'icp_intercontinental'
}

// 计算属性
const totalTournaments = computed(() => {
  if (selectedSeason.value === 'all') return tournaments.value.length
  return tournaments.value.filter(t => t.season === selectedSeason.value).length
})

const internationalCount = computed(() => {
  const filtered = selectedSeason.value === 'all'
    ? tournaments.value
    : tournaments.value.filter(t => t.season === selectedSeason.value)
  return filtered.filter(t => t.type === 'international').length
})

const leagueCount = computed(() => {
  const filtered = selectedSeason.value === 'all'
    ? tournaments.value
    : tournaments.value.filter(t => t.season === selectedSeason.value)
  return filtered.filter(t => t.type === 'league').length
})

const mvpCount = computed(() => {
  const filtered = selectedSeason.value === 'all'
    ? tournaments.value
    : tournaments.value.filter(t => t.season === selectedSeason.value)
  return filtered.filter(t => t.mvp).length
})

const filteredTournaments = computed(() => {
  let result = tournaments.value

  // 按赛季筛选
  if (selectedSeason.value !== 'all') {
    result = result.filter(t => t.season === selectedSeason.value)
  }

  // 按类型筛选
  if (tournamentFilter.value !== 'all') {
    result = result.filter(t => t.type === tournamentFilter.value)
  }

  return result
})

// 方法
const openTournamentDetail = (tournament: Tournament) => {
  selectedTournament.value = tournament
  showDetailDialog.value = true
}

const getRegionTagType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: '',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getRankClass = (rank: number) => {
  if (rank === 1) return 'gold'
  if (rank === 2) return 'silver'
  if (rank === 3) return 'bronze'
  return ''
}
</script>

<style scoped>
.honors-view {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: var(--text-tertiary, #909399);
  margin: 0;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); }
.stat-icon.purple { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.orange { background: linear-gradient(135deg, #f97316, #ea580c); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: var(--text-tertiary, #909399);
  margin-top: 4px;
}

/* 荣誉卡片 */
.honors-card,
.team-stats-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary, #303133);
}

.empty-icon {
  font-size: 64px;
}

/* 赛事卡片网格 */
.tournament-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

/* 赛事卡片 */
.tournament-card {
  background: white;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #ebeef5;
  cursor: pointer;
  transition: all 0.3s ease;
}

.tournament-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.1);
}

.tournament-card.international {
  border-top: 3px solid #8b5cf6;
}

.tournament-card.league {
  border-top: 3px solid #3b82f6;
}

.tournament-header {
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.tournament-header.international {
  background: linear-gradient(135deg, #8b5cf6, #6d28d9);
}

.tournament-header.league {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.tournament-badge {
  position: absolute;
  top: 12px;
  left: 12px;
  padding: 4px 12px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  color: white;
  font-size: 12px;
  font-weight: 500;
}

.tournament-trophy {
  font-size: 40px;
}

.tournament-content {
  padding: 20px;
}

.tournament-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin: 0 0 4px 0;
}

.tournament-season {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
  margin: 0 0 16px 0;
}

.champion-info {
  margin-bottom: 16px;
}

.champion-label {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.champion-team {
  display: flex;
  align-items: center;
  gap: 12px;
}

.team-avatar {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 14px;
}

.team-avatar.large {
  width: 56px;
  height: 56px;
  font-size: 16px;
}

.team-avatar.lpl { background: linear-gradient(135deg, #ef4444, #dc2626); }
.team-avatar.lck { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.team-avatar.lec { background: linear-gradient(135deg, #22c55e, #16a34a); }
.team-avatar.lcs { background: linear-gradient(135deg, #f59e0b, #d97706); }

.team-details { flex: 1; }

.team-name {
  font-weight: 600;
  color: var(--text-primary, #303133);
  margin-bottom: 4px;
}

.team-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-meta-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-count {
  font-size: 12px;
  color: var(--text-secondary, #606266);
  font-weight: 500;
}

.team-count-badge {
  font-size: 11px;
  color: #8b5cf6;
  font-weight: 600;
  background: rgba(139, 92, 246, 0.1);
  padding: 2px 8px;
  border-radius: 10px;
}

.click-hint {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-tertiary, #909399);
  padding-top: 12px;
  border-top: 1px solid #f0f2f5;
}

/* 战队荣誉表格 */
.honors-table {
  border-radius: 8px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 12px;
}

.team-info { flex: 1; }

.rank-badge {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 14px;
  background: #f0f2f5;
  color: #606266;
}

.rank-badge.gold { background: linear-gradient(135deg, #fbbf24, #f59e0b); color: white; }
.rank-badge.silver { background: linear-gradient(135deg, #9ca3af, #6b7280); color: white; }
.rank-badge.bronze { background: linear-gradient(135deg, #f97316, #ea580c); color: white; }

.trophy-count {
  font-size: 18px;
  font-weight: 700;
}

.trophy-count.gold { color: #f59e0b; }
.trophy-count.silver { color: #6b7280; }
.trophy-count.bronze { color: #ea580c; }

.total-points {
  font-size: 20px;
  font-weight: 700;
  color: #3b82f6;
}

/* 赛事详情弹窗 */
.tournament-detail-dialog :deep(.el-dialog__header) {
  padding-bottom: 0;
}

.tournament-detail {
  padding: 0;
}

.detail-header {
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: white;
}

.detail-header.international {
  background: linear-gradient(135deg, #8b5cf6, #6d28d9);
}

.detail-header.league {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.detail-badge {
  padding: 4px 12px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  font-size: 12px;
}

.detail-season {
  font-size: 14px;
  opacity: 0.9;
}

/* 名次展示 */
.placements {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.placement-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border-radius: 12px;
  background: #f5f7fa;
}

.placement-item.champion {
  background: linear-gradient(135deg, #fef3c7, #fde68a);
  border: 2px solid #f59e0b;
}

.placement-item.runner-up {
  background: linear-gradient(135deg, #f3f4f6, #e5e7eb);
  border: 2px solid #9ca3af;
}

.placement-item.third {
  background: linear-gradient(135deg, #fed7aa, #fdba74);
  border: 2px solid #f97316;
}

.placement-item.fourth {
  background: #f5f7fa;
  border: 1px solid #ebeef5;
}

.placement-rank {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 60px;
}

.rank-icon {
  font-size: 28px;
  line-height: 1;
}

.placement-item.fourth .rank-icon {
  width: 32px;
  height: 32px;
  background: #e0e0e0;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 700;
  color: #666;
}

.rank-text {
  font-size: 12px;
  color: var(--text-secondary, #606266);
  margin-top: 4px;
}

.placement-team {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.mvp-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #f59e0b;
  font-weight: 500;
}

.mvp-info .el-icon {
  color: #f59e0b;
}

/* 荣誉殿堂导航入口 */
.hall-navigation {
  margin-bottom: 24px;
}

.nav-card {
  cursor: pointer;
  border-radius: 12px;
  border: 2px solid transparent;
  transition: all 0.3s ease;
  margin-bottom: 16px;
}

.nav-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.nav-card.international {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-color: #ffd700;
}

.nav-card.international:hover {
  border-color: #ffed4a;
  box-shadow: 0 8px 24px rgba(255, 215, 0, 0.2);
}

.nav-card :deep(.el-card__body) {
  padding: 20px 24px;
}

.nav-content {
  display: flex;
  align-items: center;
  gap: 20px;
}

.nav-icon {
  font-size: 40px;
  line-height: 1;
}

.nav-info {
  flex: 1;
}

.nav-info h3 {
  font-size: 18px;
  font-weight: 700;
  color: #ffd700;
  margin: 0 0 6px 0;
}

.nav-info p {
  font-size: 13px;
  color: #b8860b;
  margin: 0;
}

.nav-arrow {
  font-size: 24px;
  color: #ffd700;
  transition: transform 0.3s ease;
}

.nav-card:hover .nav-arrow {
  transform: translateX(4px);
}

.sub-nav {
  display: flex;
  gap: 16px;
}

.sub-nav-card {
  flex: 1;
  cursor: pointer;
  border-radius: 10px;
  border: 1px solid #ebeef5;
  transition: all 0.3s ease;
}

.sub-nav-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
  border-color: #667eea;
}

.sub-nav-card :deep(.el-card__body) {
  padding: 16px 20px;
}

.sub-nav-content {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary, #303133);
}

.sub-icon {
  font-size: 22px;
}

/* 响应式 */
@media (max-width: 992px) {
  .tournament-grid {
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  }
}

@media (max-width: 768px) {
  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .placement-item {
    flex-wrap: wrap;
  }

  .mvp-info {
    width: 100%;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px dashed #ddd;
  }
}
</style>
