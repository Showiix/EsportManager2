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
    <el-alert
      title="积分计算说明"
      type="info"
      :closable="false"
      show-icon
      class="notice-alert"
    >
      <template #default>
        年度积分排名包含：<strong>常规赛 + 季后赛 + MSI + 世界赛</strong>的积分。
        <span class="intercontinental-note">洲际赛作为荣誉赛事，积分仅作展示，不计入年度积分排名。</span>
      </template>
    </el-alert>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><UserFilled /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ currentData.length }}</div>
              <div class="stat-label">参赛队伍</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="28"><Trophy /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ topPoints }}</div>
              <div class="stat-label">最高积分</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="28"><TrendCharts /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ averagePower.toFixed(1) }}</div>
              <div class="stat-label">平均战力</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="28"><Medal /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">4</div>
              <div class="stat-label">赛区数量</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

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
            <el-select v-model="selectedSeason" placeholder="选择赛季">
              <el-option label="S1 赛季" value="S1" />
            </el-select>
          </div>

          <!-- 年度积分表格 -->
          <el-table :data="filteredAnnualRankings" stripe class="rankings-table">
            <el-table-column prop="rank" label="排名" width="100" align="center">
              <template #default="{ row }">
                <div class="rank-cell">
                  <el-tag
                    v-if="row.rank <= 3"
                    :type="getRankTagType(row.rank)"
                    size="large"
                    effect="dark"
                  >
                    <el-icon><Trophy /></el-icon>
                    {{ row.rank }}
                  </el-tag>
                  <span v-else class="rank-number">{{ row.rank }}</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="name" label="战队" min-width="200">
              <template #default="{ row }">
                <div class="team-cell">
                  <div class="team-avatar" :class="row.region.toLowerCase()">
                    {{ row.name.slice(0, 2) }}
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

            <el-table-column label="积分明细" min-width="500">
              <template #default="{ row }">
                <div class="points-breakdown">
                  <div class="points-item" v-if="row.spring > 0">
                    <span class="points-label">春季赛:</span>
                    <span class="points-value spring">{{ row.spring }}</span>
                  </div>
                  <div class="points-item" v-if="row.msi > 0">
                    <span class="points-label">MSI:</span>
                    <span class="points-value msi">{{ row.msi }}</span>
                  </div>
                  <div class="points-item" v-if="row.madrid > 0">
                    <span class="points-label">马德里:</span>
                    <span class="points-value madrid">{{ row.madrid }}</span>
                  </div>
                  <div class="points-item" v-if="row.summer > 0">
                    <span class="points-label">夏季赛:</span>
                    <span class="points-value summer">{{ row.summer }}</span>
                  </div>
                  <div class="points-item" v-if="row.worlds > 0">
                    <span class="points-label">世界赛:</span>
                    <span class="points-value worlds">{{ row.worlds }}</span>
                  </div>
                  <div class="points-item" v-if="row.shanghai > 0">
                    <span class="points-label">上海:</span>
                    <span class="points-value shanghai">{{ row.shanghai }}</span>
                  </div>
                  <span v-if="row.points === 0" class="no-points">暂无积分</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column label="操作" width="120" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" link size="small" @click="showPointsDetail(row)">
                  积分详情
                </el-button>
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
          <el-table :data="filteredPowerRankings" stripe class="rankings-table">
            <el-table-column prop="rank" label="排名" width="100" align="center">
              <template #default="{ row }">
                <div class="rank-cell">
                  <el-tag
                    v-if="row.rank <= 3"
                    :type="getRankTagType(row.rank)"
                    size="large"
                    effect="dark"
                  >
                    <el-icon><Trophy /></el-icon>
                    {{ row.rank }}
                  </el-tag>
                  <span v-else class="rank-number">{{ row.rank }}</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column prop="name" label="战队" min-width="200">
              <template #default="{ row }">
                <div class="team-cell">
                  <div class="team-avatar" :class="row.region.toLowerCase()">
                    {{ row.name.slice(0, 2) }}
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
            <div class="team-avatar large" :class="selectedTeam.region.toLowerCase()">
              {{ selectedTeam.name.slice(0, 2) }}
            </div>
            <div class="team-meta">
              <div class="team-name-large">{{ selectedTeam.name }}</div>
              <el-tag :type="getRegionTagType(selectedTeam.region)" effect="dark">
                {{ selectedTeam.region }}
              </el-tag>
            </div>
          </div>
        </div>

        <!-- 积分来源明细 -->
        <div class="points-sources">
          <h3>积分来源明细</h3>

          <!-- 春季赛 -->
          <div class="source-section" v-if="selectedTeamDetail.spring">
            <div class="source-header spring">
              <el-icon><Trophy /></el-icon>
              <span>春季赛</span>
              <span class="source-total">+{{ selectedTeamDetail.spring.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item" v-if="selectedTeamDetail.spring.regular > 0">
                <span class="item-name">常规赛排名奖励</span>
                <span class="item-desc">{{ selectedTeamDetail.spring.regularDesc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.spring.regular }}</span>
              </div>
              <div class="source-item" v-if="selectedTeamDetail.spring.playoffs > 0">
                <span class="item-name">季后赛成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.spring.playoffsDesc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.spring.playoffs }}</span>
              </div>
            </div>
          </div>

          <!-- MSI -->
          <div class="source-section" v-if="selectedTeamDetail.msi && selectedTeamDetail.msi.total > 0">
            <div class="source-header msi">
              <el-icon><Flag /></el-icon>
              <span>MSI 季中邀请赛</span>
              <span class="source-total">+{{ selectedTeamDetail.msi.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛事成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.msi.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.msi.total }}</span>
              </div>
            </div>
          </div>

          <!-- 马德里大师赛 -->
          <div class="source-section" v-if="selectedTeamDetail.madrid && selectedTeamDetail.madrid.total > 0">
            <div class="source-header madrid">
              <el-icon><Location /></el-icon>
              <span>马德里大师赛</span>
              <span class="source-total">+{{ selectedTeamDetail.madrid.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛事成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.madrid.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.madrid.total }}</span>
              </div>
            </div>
          </div>

          <!-- 夏季赛 -->
          <div class="source-section" v-if="selectedTeamDetail.summer">
            <div class="source-header summer">
              <el-icon><Sunny /></el-icon>
              <span>夏季赛</span>
              <span class="source-total">+{{ selectedTeamDetail.summer.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item" v-if="selectedTeamDetail.summer.regular > 0">
                <span class="item-name">常规赛排名奖励</span>
                <span class="item-desc">{{ selectedTeamDetail.summer.regularDesc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.summer.regular }}</span>
              </div>
              <div class="source-item" v-if="selectedTeamDetail.summer.playoffs > 0">
                <span class="item-name">季后赛成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.summer.playoffsDesc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.summer.playoffs }}</span>
              </div>
            </div>
          </div>

          <!-- Claude洲际赛 -->
          <div class="source-section" v-if="selectedTeamDetail.claude && selectedTeamDetail.claude.total > 0">
            <div class="source-header claude">
              <el-icon><Connection /></el-icon>
              <span>Claude 洲际赛</span>
              <span class="source-total">+{{ selectedTeamDetail.claude.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛事成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.claude.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.claude.total }}</span>
              </div>
            </div>
          </div>

          <!-- 世界赛 -->
          <div class="source-section" v-if="selectedTeamDetail.worlds && selectedTeamDetail.worlds.total > 0">
            <div class="source-header worlds">
              <el-icon><Trophy /></el-icon>
              <span>S 世界赛</span>
              <span class="source-total">+{{ selectedTeamDetail.worlds.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛事成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.worlds.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.worlds.total }}</span>
              </div>
            </div>
          </div>

          <!-- 上海大师赛 -->
          <div class="source-section" v-if="selectedTeamDetail.shanghai && selectedTeamDetail.shanghai.total > 0">
            <div class="source-header shanghai">
              <el-icon><OfficeBuilding /></el-icon>
              <span>上海大师赛</span>
              <span class="source-total">+{{ selectedTeamDetail.shanghai.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛事成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.shanghai.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.shanghai.total }}</span>
              </div>
            </div>
          </div>

          <!-- ICP洲际对抗赛 -->
          <div class="source-section" v-if="selectedTeamDetail.icp && selectedTeamDetail.icp.total > 0">
            <div class="source-header icp">
              <el-icon><Promotion /></el-icon>
              <span>ICP 洲际对抗赛</span>
              <span class="source-total">+{{ selectedTeamDetail.icp.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛区成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.icp.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.icp.total }}</span>
              </div>
            </div>
          </div>

          <!-- Super洲际邀请赛 -->
          <div class="source-section" v-if="selectedTeamDetail.super && selectedTeamDetail.super.total > 0">
            <div class="source-header super">
              <el-icon><Star /></el-icon>
              <span>Super 洲际邀请赛</span>
              <span class="source-total">+{{ selectedTeamDetail.super.total }} 分</span>
            </div>
            <div class="source-items">
              <div class="source-item">
                <span class="item-name">赛事成绩</span>
                <span class="item-desc">{{ selectedTeamDetail.super.desc }}</span>
                <span class="item-points">+{{ selectedTeamDetail.super.total }}</span>
              </div>
            </div>
          </div>

          <!-- 无积分提示 -->
          <el-empty v-if="selectedTeam.points === 0" description="暂无积分记录" />
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
                  <p>冠军35分、亚军30分、季军25分、殿军20分、第三阶段败者8分、第二阶段败者5分、Fighter组2分</p>
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
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  UserFilled,
  Trophy,
  TrendCharts,
  Medal,
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

const router = useRouter()

// 状态
const activeTab = ref('annual')
const selectedRegion = ref('')
const selectedSeason = ref('S1')
const pointsDetailVisible = ref(false)
const selectedTeam = ref<any>(null)

// 模拟数据 - 年度积分榜
const annualRankings = ref([
  { rank: 1, name: 'T1', region: 'LCK', points: 920, spring: 300, msi: 200, madrid: 8, summer: 320, worlds: 100, shanghai: 16 },
  { rank: 2, name: 'Gen.G', region: 'LCK', points: 900, spring: 280, msi: 150, madrid: 16, summer: 300, worlds: 170, shanghai: 8 },
  { rank: 3, name: 'JD Gaming', region: 'LPL', points: 850, spring: 320, msi: 180, madrid: 20, summer: 280, worlds: 70, shanghai: 12 },
  { rank: 4, name: 'Bilibili Gaming', region: 'LPL', points: 780, spring: 280, msi: 0, madrid: 12, summer: 320, worlds: 180, shanghai: 20 },
  { rank: 5, name: 'Hanwha Life', region: 'LCK', points: 750, spring: 250, msi: 0, madrid: 6, summer: 280, worlds: 220, shanghai: 6 },
  { rank: 6, name: 'Top Esports', region: 'LPL', points: 720, spring: 260, msi: 0, madrid: 4, summer: 300, worlds: 160, shanghai: 4 },
  { rank: 7, name: 'Weibo Gaming', region: 'LPL', points: 680, spring: 220, msi: 120, madrid: 2, summer: 240, worlds: 100, shanghai: 0 },
  { rank: 8, name: 'G2 Esports', region: 'LEC', points: 650, spring: 300, msi: 100, madrid: 4, summer: 180, worlds: 70, shanghai: 0 },
  { rank: 9, name: 'Fnatic', region: 'LEC', points: 600, spring: 250, msi: 0, madrid: 2, summer: 280, worlds: 70, shanghai: 0 },
  { rank: 10, name: 'Cloud9', region: 'LCS', points: 520, spring: 280, msi: 0, madrid: 2, summer: 180, worlds: 60, shanghai: 0 },
  { rank: 11, name: 'Team Liquid', region: 'LCS', points: 480, spring: 220, msi: 80, madrid: 2, summer: 150, worlds: 30, shanghai: 0 },
  { rank: 12, name: 'FlyQuest', region: 'LCS', points: 450, spring: 180, msi: 0, madrid: 2, summer: 200, worlds: 70, shanghai: 0 },
])

// 模拟数据 - 战力排名榜
const powerRankings = ref([
  { rank: 1, name: 'T1', region: 'LCK', power: 90, change: 0 },
  { rank: 2, name: 'Gen.G', region: 'LCK', power: 89, change: 1 },
  { rank: 3, name: 'JD Gaming', region: 'LPL', power: 88, change: -1 },
  { rank: 4, name: 'Bilibili Gaming', region: 'LPL', power: 86, change: 2 },
  { rank: 5, name: 'Hanwha Life', region: 'LCK', power: 85, change: 0 },
  { rank: 6, name: 'Top Esports', region: 'LPL', power: 85, change: -2 },
  { rank: 7, name: 'Weibo Gaming', region: 'LPL', power: 84, change: 1 },
  { rank: 8, name: 'Dplus KIA', region: 'LCK', power: 83, change: -1 },
  { rank: 9, name: 'G2 Esports', region: 'LEC', power: 82, change: 0 },
  { rank: 10, name: 'Fnatic', region: 'LEC', power: 80, change: 2 },
  { rank: 11, name: 'MAD Lions', region: 'LEC', power: 78, change: -1 },
  { rank: 12, name: 'Cloud9', region: 'LCS', power: 76, change: 0 },
])

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

// 积分详情数据（模拟数据，实际应从store获取）
const pointsDetailData: Record<string, any> = {
  'T1': {
    spring: { total: 300, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    msi: { total: 200, desc: '冠军 (20分)' },
    madrid: { total: 8, desc: '殿军 (8分)' },
    summer: { total: 320, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    claude: { total: 8, desc: '殿军 (8分)' },
    worlds: { total: 100, desc: '四强 (8分)' },
    shanghai: { total: 16, desc: '亚军 (16分)' },
    icp: { total: 12, desc: '最强赛区参赛队伍 (12分)' },
    super: { total: 0, desc: '' },
  },
  'Gen.G': {
    spring: { total: 280, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    msi: { total: 150, desc: '季军 (12分)' },
    madrid: { total: 16, desc: '亚军 (16分)' },
    summer: { total: 300, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    claude: { total: 12, desc: '季军 (12分)' },
    worlds: { total: 170, desc: '亚军 (16分)' },
    shanghai: { total: 8, desc: '殿军 (8分)' },
    icp: { total: 12, desc: '最强赛区参赛队伍 (12分)' },
    super: { total: 0, desc: '' },
  },
  'JD Gaming': {
    spring: { total: 320, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    msi: { total: 180, desc: '亚军 (16分)' },
    madrid: { total: 20, desc: '冠军 (20分)' },
    summer: { total: 280, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    claude: { total: 16, desc: '亚军 (16分)' },
    worlds: { total: 70, desc: '八强 (6分)' },
    shanghai: { total: 12, desc: '季军 (12分)' },
    icp: { total: 8, desc: '第二名赛区参赛队伍 (8分)' },
    super: { total: 0, desc: '' },
  },
  'Bilibili Gaming': {
    spring: { total: 280, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    msi: { total: 0, desc: '' },
    madrid: { total: 12, desc: '季军 (12分)' },
    summer: { total: 320, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    claude: { total: 20, desc: '冠军 (20分)' },
    worlds: { total: 180, desc: '冠军 (20分)' },
    shanghai: { total: 20, desc: '冠军 (20分)' },
    icp: { total: 8, desc: '第二名赛区参赛队伍 (8分)' },
    super: { total: 0, desc: '' },
  },
  'Hanwha Life': {
    spring: { total: 250, regular: 0, regularDesc: '', playoffs: 8, playoffsDesc: '季军' },
    msi: { total: 0, desc: '' },
    madrid: { total: 6, desc: '半决赛败者 (6分)' },
    summer: { total: 280, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    claude: { total: 6, desc: '半决赛败者 (6分)' },
    worlds: { total: 220, desc: '季军 (12分)' },
    shanghai: { total: 6, desc: '败者组第二轮 (6分)' },
    icp: { total: 12, desc: '最强赛区参赛队伍 (12分)' },
    super: { total: 0, desc: '' },
  },
  'Top Esports': {
    spring: { total: 260, regular: 0, regularDesc: '', playoffs: 8, playoffsDesc: '季军' },
    msi: { total: 0, desc: '' },
    madrid: { total: 4, desc: '四分之一决赛败者 (4分)' },
    summer: { total: 300, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    claude: { total: 4, desc: '四分之一决赛败者 (4分)' },
    worlds: { total: 160, desc: '殿军 (8分)' },
    shanghai: { total: 4, desc: '败者组第一轮 (4分)' },
    icp: { total: 8, desc: '第二名赛区参赛队伍 (8分)' },
    super: { total: 0, desc: '' },
  },
  'Weibo Gaming': {
    spring: { total: 220, regular: 0, regularDesc: '', playoffs: 6, playoffsDesc: '第四名' },
    msi: { total: 120, desc: '殿军 (8分)' },
    madrid: { total: 2, desc: '首轮败者 (2分)' },
    summer: { total: 240, regular: 0, regularDesc: '', playoffs: 8, playoffsDesc: '季军' },
    claude: { total: 2, desc: '首轮败者 (2分)' },
    worlds: { total: 100, desc: '八强 (6分)' },
    shanghai: { total: 0, desc: '' },
    icp: { total: 8, desc: '第二名赛区参赛队伍 (8分)' },
    super: { total: 0, desc: '' },
  },
  'G2 Esports': {
    spring: { total: 300, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    msi: { total: 100, desc: '八强 (6分)' },
    madrid: { total: 4, desc: '四分之一决赛败者 (4分)' },
    summer: { total: 180, regular: 0, regularDesc: '', playoffs: 6, playoffsDesc: '第四名' },
    claude: { total: 0, desc: '' },
    worlds: { total: 70, desc: '小组赛 (4分)' },
    shanghai: { total: 0, desc: '' },
    icp: { total: 6, desc: '第三名赛区参赛队伍 (6分)' },
    super: { total: 0, desc: '' },
  },
  'Fnatic': {
    spring: { total: 250, regular: 0, regularDesc: '', playoffs: 10, playoffsDesc: '亚军' },
    msi: { total: 0, desc: '' },
    madrid: { total: 2, desc: '首轮败者 (2分)' },
    summer: { total: 280, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    claude: { total: 0, desc: '' },
    worlds: { total: 70, desc: '小组赛 (4分)' },
    shanghai: { total: 0, desc: '' },
    icp: { total: 6, desc: '第三名赛区参赛队伍 (6分)' },
    super: { total: 0, desc: '' },
  },
  'Cloud9': {
    spring: { total: 280, regular: 0, regularDesc: '', playoffs: 12, playoffsDesc: '冠军' },
    msi: { total: 0, desc: '' },
    madrid: { total: 2, desc: '首轮败者 (2分)' },
    summer: { total: 180, regular: 0, regularDesc: '', playoffs: 6, playoffsDesc: '第四名' },
    claude: { total: 0, desc: '' },
    worlds: { total: 60, desc: '小组赛 (4分)' },
    shanghai: { total: 0, desc: '' },
    icp: { total: 4, desc: '第四名赛区参赛队伍 (4分)' },
    super: { total: 0, desc: '' },
  },
  'Team Liquid': {
    spring: { total: 220, regular: 0, regularDesc: '', playoffs: 8, playoffsDesc: '季军' },
    msi: { total: 80, desc: '八强 (6分)' },
    madrid: { total: 2, desc: '首轮败者 (2分)' },
    summer: { total: 150, regular: 0, regularDesc: '', playoffs: 6, playoffsDesc: '第四名' },
    claude: { total: 0, desc: '' },
    worlds: { total: 30, desc: '入围赛淘汰 (2分)' },
    shanghai: { total: 0, desc: '' },
    icp: { total: 4, desc: '第四名赛区参赛队伍 (4分)' },
    super: { total: 0, desc: '' },
  },
  'FlyQuest': {
    spring: { total: 180, regular: 0, regularDesc: '', playoffs: 6, playoffsDesc: '第四名' },
    msi: { total: 0, desc: '' },
    madrid: { total: 2, desc: '首轮败者 (2分)' },
    summer: { total: 200, regular: 0, regularDesc: '', playoffs: 8, playoffsDesc: '季军' },
    claude: { total: 0, desc: '' },
    worlds: { total: 70, desc: '八强 (6分)' },
    shanghai: { total: 0, desc: '' },
    icp: { total: 4, desc: '第四名赛区参赛队伍 (4分)' },
    super: { total: 0, desc: '' },
  },
}

// 获取选中战队的详细积分数据
const selectedTeamDetail = computed(() => {
  if (!selectedTeam.value) return {}
  const teamName = selectedTeam.value.name
  return pointsDetailData[teamName] || {
    spring: { total: selectedTeam.value.spring || 0, regular: 0, regularDesc: '', playoffs: selectedTeam.value.spring || 0, playoffsDesc: '参赛' },
    msi: { total: selectedTeam.value.msi || 0, desc: selectedTeam.value.msi > 0 ? '参赛' : '' },
    summer: { total: selectedTeam.value.summer || 0, regular: 0, regularDesc: '', playoffs: selectedTeam.value.summer || 0, playoffsDesc: '参赛' },
    worlds: { total: selectedTeam.value.worlds || 0, desc: selectedTeam.value.worlds > 0 ? '参赛' : '' },
  }
})

// 方法
const showPointsDetail = (team: any) => {
  selectedTeam.value = team
  pointsDetailVisible.value = true
}

const goToTeamDetail = () => {
  if (selectedTeam.value) {
    router.push(`/teams/${selectedTeam.value.rank}`)
    pointsDetailVisible.value = false
  }
}

const viewTeam = (team: any) => {
  router.push(`/teams/${team.rank}`)
}

const getRankTagType = (rank: number) => {
  switch (rank) {
    case 1: return 'danger'
    case 2: return 'warning'
    case 3: return 'success'
    default: return 'info'
  }
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

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: #909399;
  margin: 0;
}

.notice-alert {
  margin-bottom: 20px;
  border-radius: 8px;
}

.intercontinental-note {
  color: #ff9800;
  font-weight: 500;
}

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

.stat-icon.blue {
  background: linear-gradient(135deg, #667eea, #764ba2);
}

.stat-icon.green {
  background: linear-gradient(135deg, #11998e, #38ef7d);
}

.stat-icon.orange {
  background: linear-gradient(135deg, #f093fb, #f5576c);
}

.stat-icon.purple {
  background: linear-gradient(135deg, #4facfe, #00f2fe);
}

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.rankings-card {
  border-radius: 12px;
}

.filter-row {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.filter-row .el-select {
  width: 180px;
}

.rankings-table {
  border-radius: 8px;
}

.rank-cell {
  display: flex;
  justify-content: center;
  align-items: center;
}

.rank-number {
  font-weight: 700;
  font-size: 18px;
  color: #606266;
}

.team-cell {
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

.team-avatar.lpl {
  background: linear-gradient(135deg, #ff4757, #ff6b81);
}

.team-avatar.lck {
  background: linear-gradient(135deg, #3742fa, #5352ed);
}

.team-avatar.lec {
  background: linear-gradient(135deg, #2ed573, #7bed9f);
}

.team-avatar.lcs {
  background: linear-gradient(135deg, #ffa502, #ff6348);
}

.team-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.team-name {
  font-weight: 600;
  color: #303133;
  font-size: 15px;
}

.total-points {
  font-size: 24px;
  font-weight: 700;
  color: #409eff;
}

.points-breakdown {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.points-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: #f0f2f5;
  border-radius: 4px;
  font-size: 13px;
}

.points-label {
  color: #909399;
}

.points-value {
  font-weight: 600;
}

.points-value.spring {
  color: #67c23a;
}

.points-value.summer {
  color: #e6a23c;
}

.points-value.msi {
  color: #f56c6c;
}

.points-value.worlds {
  color: #9c27b0;
}

.points-value.madrid {
  color: #e6a23c;
}

.points-value.shanghai {
  color: #00bcd4;
}

.no-points {
  color: #c0c4cc;
  font-style: italic;
}

.power-cell {
  padding-right: 20px;
}

.change-up {
  color: #67c23a;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.change-down {
  color: #f56c6c;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.change-none {
  color: #909399;
}

/* 表格样式优化 */
.rankings-table :deep(.el-table__row) {
  height: 72px;
}

.rankings-table :deep(.el-tag--large) {
  padding: 8px 12px;
  font-size: 14px;
}

.rankings-table :deep(.el-tag--large .el-icon) {
  margin-right: 4px;
}

/* 积分详情弹窗样式 */
.points-detail-content {
  padding: 0 8px;
}

.total-overview {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  margin-bottom: 24px;
}

.total-points-display {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.total-points-display .points-number {
  font-size: 48px;
  font-weight: 800;
  color: white;
  line-height: 1;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
}

.total-points-display .points-unit {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
  margin-top: 4px;
}

.team-badge {
  display: flex;
  align-items: center;
  gap: 16px;
}

.team-avatar.large {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.team-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.team-name-large {
  font-size: 20px;
  font-weight: 700;
  color: white;
}

.points-sources h3 {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 2px solid #f0f2f5;
}

.source-section {
  margin-bottom: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  overflow: hidden;
}

.source-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  font-weight: 600;
  color: white;
}

.source-header .el-icon {
  font-size: 18px;
}

.source-header .source-total {
  margin-left: auto;
  font-size: 16px;
}

.source-header.spring {
  background: linear-gradient(135deg, #67c23a, #95d475);
}

.source-header.msi {
  background: linear-gradient(135deg, #f56c6c, #fab6b6);
}

.source-header.madrid {
  background: linear-gradient(135deg, #e6a23c, #f3d19e);
}

.source-header.summer {
  background: linear-gradient(135deg, #ff9500, #ffb74d);
}

.source-header.claude {
  background: linear-gradient(135deg, #409eff, #79bbff);
}

.source-header.worlds {
  background: linear-gradient(135deg, #9c27b0, #ce93d8);
}

.source-header.shanghai {
  background: linear-gradient(135deg, #00bcd4, #4dd0e1);
}

.source-header.icp {
  background: linear-gradient(135deg, #ff5722, #ff8a65);
}

.source-header.super {
  background: linear-gradient(135deg, #ffc107, #ffecb3);
  color: #333;
}

.source-items {
  padding: 8px 0;
  background: #fafafa;
}

.source-item {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid #f0f2f5;
}

.source-item:last-child {
  border-bottom: none;
}

.source-item .item-name {
  font-weight: 500;
  color: #303133;
  min-width: 120px;
}

.source-item .item-desc {
  flex: 1;
  color: #909399;
  font-size: 13px;
}

.source-item .item-points {
  font-weight: 700;
  color: #67c23a;
  font-size: 15px;
}

.points-rules {
  margin-top: 20px;
}

.points-rules :deep(.el-collapse-item__header) {
  font-weight: 600;
  color: #606266;
}

.rules-content {
  padding: 8px 0;
}

.rule-section {
  margin-bottom: 12px;
}

.rule-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 4px 0;
}

.rule-section p {
  font-size: 13px;
  color: #909399;
  margin: 0;
  line-height: 1.6;
}

/* 弹窗样式覆盖 */
.points-detail-dialog :deep(.el-dialog__header) {
  border-bottom: 1px solid #f0f2f5;
  padding-bottom: 16px;
}

.points-detail-dialog :deep(.el-dialog__title) {
  font-weight: 700;
  font-size: 18px;
}

.points-detail-dialog :deep(.el-dialog__body) {
  max-height: 60vh;
  overflow-y: auto;
}
</style>
