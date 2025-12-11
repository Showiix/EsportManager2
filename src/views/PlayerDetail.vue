<template>
  <div class="player-detail-view">
    <!-- 返回按钮 -->
    <div class="back-link">
      <el-button text @click="$router.push('/players')">
        <el-icon><ArrowLeft /></el-icon>
        返回选手列表
      </el-button>
    </div>

    <!-- 选手头部信息 -->
    <el-card class="profile-card">
      <div class="profile-content">
        <!-- 头像区域 -->
        <div class="avatar-section">
          <div class="player-avatar" :class="player.region.toLowerCase()">
            {{ player.position }}
          </div>
          <el-tag :type="getTalentType(player.tag)" size="large" effect="dark" class="talent-tag">
            {{ getTalentLabel(player.tag) }}
          </el-tag>
        </div>

        <!-- 基本信息 -->
        <div class="info-section">
          <div class="player-header">
            <h1 class="player-name">{{ player.gameId }}</h1>
            <div class="player-tags">
              <el-tag :type="getPositionType(player.position)" size="default">
                {{ getPositionName(player.position) }}
              </el-tag>
              <el-tag :type="getRegionType(player.region)" size="default">
                {{ player.region }}
              </el-tag>
              <el-tag type="success" size="default">在役</el-tag>
            </div>
          </div>
          <p class="player-real-name">{{ player.realName }} · {{ player.nationality }}</p>
          <div class="player-team">
            <div class="team-avatar mini" :class="player.region.toLowerCase()">
              {{ player.team.substring(0, 2) }}
            </div>
            <span>{{ player.team }}</span>
          </div>
        </div>

        <!-- 能力值展示 -->
        <div class="stats-section">
          <div class="stat-number-display">
            <span class="stat-value" :style="{ color: getAbilityColor(player.ability) }">{{ player.ability }}</span>
            <span class="stat-label">能力</span>
          </div>
          <div class="stat-number-display">
            <span class="stat-value" style="color: #8b5cf6;">{{ player.potential }}</span>
            <span class="stat-label">潜力</span>
          </div>
          <div class="stat-number-display">
            <span class="stat-value" style="color: #22c55e;">{{ player.stability }}</span>
            <span class="stat-label">稳定</span>
          </div>
          <div class="stat-text">
            <div class="age-display">
              <span class="age-value">{{ player.age }}</span>
              <span class="age-label">岁</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 详细信息区 -->
    <el-row :gutter="20" class="detail-row">
      <!-- 合同信息 -->
      <el-col :span="12">
        <el-card class="detail-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon><Document /></el-icon>
                合同信息
              </h2>
            </div>
          </template>
          <div class="info-list">
            <div class="info-row">
              <span class="info-label">所属战队</span>
              <span class="info-value">{{ player.team }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">所属赛区</span>
              <el-tag :type="getRegionType(player.region)" size="small">
                {{ player.region }}
              </el-tag>
            </div>
            <div class="info-row">
              <span class="info-label">合同到期</span>
              <span class="info-value highlight">{{ player.contractEnd }} 赛季</span>
            </div>
            <div class="info-row">
              <span class="info-label">年薪</span>
              <span class="info-value money">{{ formatMoney(player.salary) }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">身价</span>
              <span class="info-value success">{{ formatMoney(player.marketValue) }}</span>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 职业生涯 -->
      <el-col :span="12">
        <el-card class="detail-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon><TrendCharts /></el-icon>
                职业生涯
              </h2>
            </div>
          </template>
          <div class="info-list">
            <div class="info-row">
              <span class="info-label">加入赛季</span>
              <span class="info-value">{{ player.joinSeason }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">职业年数</span>
              <span class="info-value">{{ careerYears }} 年</span>
            </div>
            <div class="info-row">
              <span class="info-label">冠军数</span>
              <span class="info-value gold">{{ championCount }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">总荣誉</span>
              <span class="info-value">{{ honors.length }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">成长空间</span>
              <span class="info-value purple">+{{ player.potential - player.ability }}</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 天赋说明 -->
    <el-alert
      :title="getTalentDescription(player.tag)"
      :type="getTalentAlertType(player.tag)"
      :closable="false"
      show-icon
      class="talent-alert"
    />

    <!-- 特性与状态区 -->
    <el-row :gutter="20" class="traits-condition-row">
      <!-- 选手特性 -->
      <el-col :span="12">
        <el-card class="traits-card">
          <template #header>
            <div class="card-header">
              <h2>
                <span class="header-icon">⚡</span>
                选手特性
              </h2>
              <span class="count-badge">{{ traits.length }} 项特性</span>
            </div>
          </template>

          <el-empty v-if="traits.length === 0" description="暂无特性" :image-size="60">
            <template #image>
              <div class="empty-icon">🎯</div>
            </template>
          </el-empty>

          <div v-else class="traits-grid">
            <div
              v-for="trait in traits"
              :key="trait.trait_type"
              class="trait-item"
              :class="[`rarity-${trait.rarity}`, { 'negative': trait.is_negative }]"
            >
              <div class="trait-header">
                <span class="trait-name">{{ trait.name }}</span>
                <span class="trait-rarity">{{ '★'.repeat(trait.rarity) }}</span>
              </div>
              <div class="trait-description">{{ trait.description }}</div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 选手状态 -->
      <el-col :span="12">
        <el-card class="condition-card">
          <template #header>
            <div class="card-header">
              <h2>
                <span class="header-icon">📊</span>
                当前状态
              </h2>
            </div>
          </template>

          <div v-if="conditionInfo" class="condition-content">
            <!-- Condition 值展示 -->
            <div class="condition-display">
              <div class="condition-value" :class="getConditionClass(conditionInfo.condition)">
                {{ conditionInfo.condition > 0 ? '+' : '' }}{{ conditionInfo.condition }}
              </div>
              <div class="condition-label">状态值</div>
              <div class="condition-range">
                范围: {{ conditionInfo.condition_range[0] }} ~ +{{ conditionInfo.condition_range[1] }}
              </div>
            </div>

            <!-- 状态因子详情 -->
            <div class="condition-factors">
              <div class="factor-item">
                <span class="factor-label">动能</span>
                <span class="factor-value" :class="getMomentumClass(conditionInfo.momentum)">
                  {{ conditionInfo.momentum > 0 ? '+' : '' }}{{ conditionInfo.momentum }}
                </span>
              </div>
              <div class="factor-item">
                <span class="factor-label">状态周期</span>
                <el-progress
                  :percentage="conditionInfo.form_cycle"
                  :stroke-width="8"
                  :show-text="false"
                  color="#3b82f6"
                />
              </div>
              <div class="factor-item">
                <span class="factor-label">上场发挥</span>
                <span class="factor-value">
                  {{ conditionInfo.last_performance > 0 ? conditionInfo.last_performance.toFixed(1) : '-' }}
                </span>
              </div>
              <div class="factor-item">
                <span class="factor-label">上场结果</span>
                <el-tag :type="conditionInfo.last_match_won ? 'success' : 'danger'" size="small">
                  {{ conditionInfo.last_match_won ? '胜' : '负' }}
                </el-tag>
              </div>
              <div class="factor-item">
                <span class="factor-label">连续比赛</span>
                <span class="factor-value">{{ conditionInfo.games_since_rest }} 场</span>
              </div>
            </div>
          </div>

          <el-empty v-else description="暂无状态数据" :image-size="60">
            <template #image>
              <div class="empty-icon">📊</div>
            </template>
          </el-empty>
        </el-card>
      </el-col>
    </el-row>

    <!-- 荣誉记录 -->
    <el-card class="honors-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><Trophy /></el-icon>
            荣誉记录
          </h2>
          <span class="count-badge">共 {{ honors.length }} 项荣誉</span>
        </div>
      </template>

      <el-empty v-if="honors.length === 0" description="暂无荣誉记录">
        <template #image>
          <div class="empty-icon">🏆</div>
        </template>
      </el-empty>

      <el-timeline v-else>
        <el-timeline-item
          v-for="honor in honors"
          :key="`${honor.season}-${honor.tournament}`"
          :timestamp="honor.season"
          placement="top"
          :color="getHonorColor(honor.position)"
          size="large"
        >
          <el-card class="honor-card" :class="getHonorClass(honor.position)" shadow="hover">
            <div class="honor-content">
              <div class="honor-icon">
                {{ getHonorEmoji(honor.position) }}
              </div>
              <div class="honor-info">
                <div class="honor-title">{{ honor.tournament }}</div>
                <el-tag :type="getHonorTagType(honor.position)" size="default" effect="dark">
                  {{ honor.position }}
                </el-tag>
              </div>
            </div>
          </el-card>
        </el-timeline-item>
      </el-timeline>
    </el-card>

    <!-- 赛季历史 -->
    <el-card class="history-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><Clock /></el-icon>
            赛季历史
          </h2>
        </div>
      </template>

      <el-table :data="seasonHistory" stripe class="history-table">
        <el-table-column prop="season" label="赛季" width="120" align="center" />
        <el-table-column prop="team" label="所属战队" width="150">
          <template #default="{ row }">
            <div class="team-cell">
              <div class="team-avatar mini" :class="player.region.toLowerCase()">
                {{ row.team.substring(0, 2) }}
              </div>
              <span>{{ row.team }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="ability" label="能力值" width="120" align="center">
          <template #default="{ row }">
            <span class="ability-value" :style="{ color: getAbilityColor(row.ability) }">
              {{ row.ability }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="potential" label="潜力值" width="120" align="center">
          <template #default="{ row }">
            <span class="potential-value">{{ row.potential }}</span>
          </template>
        </el-table-column>
        <el-table-column label="成长" width="100" align="center">
          <template #default="{ row, $index }">
            <el-tag v-if="$index > 0" type="success" size="small">
              +{{ row.ability - seasonHistory[$index - 1].ability }}
            </el-tag>
            <span v-else class="text-gray">-</span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import {
  ArrowLeft,
  Document,
  TrendCharts,
  Trophy,
  Clock,
} from '@element-plus/icons-vue'
import { teamApi, playerApi, type TraitInfo, type PlayerConditionInfo } from '@/api/tauri'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'

const route = useRoute()
const playerId = route.params.id as string
const teamStore = useTeamStoreTauri()

// 选手数据
const player = ref({
  id: playerId,
  gameId: '加载中...',
  realName: '加载中...',
  nationality: '未知',
  team: '未知',
  region: 'LPL',
  position: 'MID',
  age: 20,
  ability: 70,
  potential: 80,
  stability: 75,
  tag: 'NORMAL',
  salary: 500000,
  marketValue: 3000000,
  contractEnd: 'S2',
  joinSeason: 'S1',
})

// 荣誉记录
const honors = ref<Array<{season: string, tournament: string, position: string}>>([])

// 选手特性
const traits = ref<TraitInfo[]>([])

// 选手状态因子
const conditionInfo = ref<PlayerConditionInfo | null>(null)

// 赛季历史
const seasonHistory = ref<Array<{season: string, team: string, ability: number, potential: number}>>([])

// 位置简称映射
const positionShortMap: Record<string, string> = {
  'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP'
}

// 加载选手数据
onMounted(async () => {
  try {
    // 尝试将 playerId 转换为数字（后端返回的是数字ID）
    const numericId = parseInt(playerId)

    if (!isNaN(numericId)) {
      // 使用数字ID直接从API获取选手
      const foundPlayer = await playerApi.getPlayer(numericId)

      if (foundPlayer) {
        // 加载赛区信息获取赛区代码
        await teamStore.loadRegions()

        // 获取队伍信息
        let teamName = '未知'
        let regionCode = 'LPL'

        if (foundPlayer.team_id) {
          try {
            const team = await teamApi.getTeam(foundPlayer.team_id)
            teamName = team.name
            const region = teamStore.regions.find(r => r.id === team.region_id)
            regionCode = region?.code || 'LPL'
          } catch (e) {
            console.error('Failed to get team info:', e)
          }
        }

        // 计算天赋标签
        const tag = foundPlayer.potential >= 90 || foundPlayer.ability >= 85 ? 'GENIUS'
          : foundPlayer.potential >= 75 || foundPlayer.ability >= 70 ? 'NORMAL'
          : 'ORDINARY'

        // 计算身价和工资
        const marketValue = foundPlayer.ability * 100000 + foundPlayer.potential * 50000
        const salary = Math.round(marketValue * 0.15)

        // 位置转换
        const position = positionShortMap[foundPlayer.position || ''] || foundPlayer.position || 'MID'

        player.value = {
          id: playerId,
          gameId: foundPlayer.game_id,
          realName: foundPlayer.real_name || foundPlayer.game_id,
          nationality: getRegionNationality(regionCode),
          team: teamName,
          region: regionCode,
          position: position,
          age: foundPlayer.age,
          ability: foundPlayer.ability,
          potential: foundPlayer.potential,
          stability: foundPlayer.stability || Math.round(70 + (30 - foundPlayer.age) * 0.5 + Math.random() * 10),
          tag: tag,
          salary: foundPlayer.salary || salary,
          marketValue: foundPlayer.market_value || marketValue,
          contractEnd: foundPlayer.contract_end_season ? `S${foundPlayer.contract_end_season}` : 'S3',
          joinSeason: 'S1',
        }

        // 荣誉记录初始为空（实际数据由后端获取）
        honors.value = []

        // 加载选手特性和状态
        try {
          const [traitsData, conditionData] = await Promise.all([
            playerApi.getPlayerTraits(numericId),
            playerApi.getPlayerCondition(numericId)
          ])
          traits.value = traitsData || []
          conditionInfo.value = conditionData
        } catch (e) {
          console.error('Failed to load traits/condition:', e)
          traits.value = []
          conditionInfo.value = null
        }

        // 生成赛季历史
        seasonHistory.value = [{
          season: 'S1',
          team: teamName,
          ability: foundPlayer.ability,
          potential: foundPlayer.potential
        }]
      }
    }
  } catch (error) {
    console.error('Failed to load player:', error)
  }
})

// 根据赛区获取国籍
const getRegionNationality = (regionCode: string) => {
  const nationalities: Record<string, string> = {
    'LPL': '中国',
    'LCK': '韩国',
    'LEC': '欧洲',
    'LCS': '北美',
  }
  return nationalities[regionCode] || '未知'
}

// 计算属性
const careerYears = computed(() => {
  const joinMatch = player.value.joinSeason.match(/S(\d+)/)
  if (joinMatch) {
    const joinYear = parseInt(joinMatch[1])
    const currentYear = 1 // 当前 S1
    return Math.max(1, currentYear - joinYear + 1)
  }
  return 1
})

const championCount = computed(() => {
  return honors.value.filter(h => h.position === '冠军').length
})

// 辅助函数
const formatMoney = (value: number) => {
  if (value >= 10000000) {
    return `${(value / 10000000).toFixed(1)} 千万`
  }
  return `${(value / 10000).toFixed(0)} 万`
}

const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getPositionType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'success',
    MID: 'primary',
    ADC: 'warning',
    SUP: 'info',
  }
  return types[position] || 'info'
}

const getPositionName = (position: string) => {
  const names: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: '下路',
    SUP: '辅助',
  }
  return names[position] || position
}

const getTalentType = (tag: string) => {
  const types: Record<string, string> = {
    GENIUS: 'warning',
    NORMAL: 'primary',
    ORDINARY: 'info',
  }
  return types[tag] || 'info'
}

const getTalentLabel = (tag: string) => {
  const labels: Record<string, string> = {
    GENIUS: '天才',
    NORMAL: '普通',
    ORDINARY: '平庸',
  }
  return labels[tag] || tag
}

const getTalentDescription = (tag: string) => {
  const desc: Record<string, string> = {
    GENIUS: '天才选手：每赛季能力值增长 +3，潜力上限更高',
    NORMAL: '普通选手：每赛季能力值增长 +2，稳定发挥',
    ORDINARY: '平庸选手：每赛季能力值增长 +1，成长较慢',
  }
  return desc[tag] || ''
}

const getTalentAlertType = (tag: string) => {
  const types: Record<string, string> = {
    GENIUS: 'warning',
    NORMAL: 'info',
    ORDINARY: 'info',
  }
  return types[tag] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getHonorColor = (position: string) => {
  const colors: Record<string, string> = {
    '冠军': '#fbbf24',
    '亚军': '#9ca3af',
    '季军': '#f97316',
  }
  return colors[position] || '#3b82f6'
}

const getHonorClass = (position: string) => {
  const classes: Record<string, string> = {
    '冠军': 'champion',
    '亚军': 'runner-up',
    '季军': 'third-place',
  }
  return classes[position] || ''
}

const getHonorEmoji = (position: string) => {
  const emojis: Record<string, string> = {
    '冠军': '🏆',
    '亚军': '🥈',
    '季军': '🥉',
  }
  return emojis[position] || '🏅'
}

const getHonorTagType = (position: string) => {
  const types: Record<string, string> = {
    '冠军': 'warning',
    '亚军': 'info',
    '季军': 'danger',
  }
  return types[position] || 'primary'
}

// 状态值样式
const getConditionClass = (condition: number) => {
  if (condition >= 5) return 'excellent'
  if (condition >= 2) return 'good'
  if (condition >= 0) return 'normal'
  if (condition >= -3) return 'poor'
  return 'terrible'
}

// 动能值样式
const getMomentumClass = (momentum: number) => {
  if (momentum >= 3) return 'hot'
  if (momentum >= 1) return 'warming'
  if (momentum <= -3) return 'cold'
  if (momentum <= -1) return 'cooling'
  return 'neutral'
}
</script>

<style scoped>
.player-detail-view {
  padding: 0;
}

.back-link {
  margin-bottom: 16px;
}

.back-link .el-button {
  color: var(--text-secondary);
  font-size: 14px;
}

.back-link .el-button:hover {
  color: var(--primary-color);
}

/* 选手资料卡片 */
.profile-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.profile-content {
  display: flex;
  align-items: flex-start;
  gap: 32px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.player-avatar {
  width: 120px;
  height: 120px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 24px;
}

.player-avatar.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.player-avatar.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.player-avatar.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.player-avatar.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.talent-tag {
  font-size: 14px;
}

.info-section {
  flex: 1;
}

.player-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 8px;
}

.player-name {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.player-tags {
  display: flex;
  gap: 8px;
}

.player-real-name {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

.player-team {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  color: var(--text-primary);
}

.team-avatar.mini {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 10px;
}

.team-avatar.mini.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.team-avatar.mini.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.team-avatar.mini.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.team-avatar.mini.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.stats-section {
  display: flex;
  align-items: center;
  gap: 24px;
}

.stat-number-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stat-number-display .stat-value {
  font-size: 32px;
  font-weight: 700;
  line-height: 1;
}

.stat-number-display .stat-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.stat-text {
  text-align: center;
}

.age-display {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.age-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.age-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* 详情卡片 */
.detail-row {
  margin-bottom: 20px;
}

.detail-card {
  border-radius: 12px;
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.count-badge {
  font-size: 14px;
  color: var(--text-tertiary);
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-light);
}

.info-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.info-label {
  font-size: 14px;
  color: var(--text-tertiary);
}

.info-value {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.info-value.highlight {
  color: var(--primary-color);
}

.info-value.money {
  color: #f59e0b;
}

.info-value.success {
  color: #22c55e;
}

.info-value.gold {
  color: #fbbf24;
  font-weight: 700;
}

.info-value.purple {
  color: #8b5cf6;
}

/* 天赋说明 */
.talent-alert {
  margin-bottom: 20px;
  border-radius: 8px;
}

/* 荣誉卡片 */
.honors-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.empty-icon {
  font-size: 64px;
}

.honor-card {
  margin-bottom: 0;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.honor-card:hover {
  transform: translateX(4px);
}

.honor-card.champion {
  border-left: 4px solid #fbbf24;
  background: linear-gradient(135deg, #fffbeb 0%, #ffffff 100%);
}

.honor-card.runner-up {
  border-left: 4px solid #9ca3af;
}

.honor-card.third-place {
  border-left: 4px solid #f97316;
}

.honor-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.honor-icon {
  font-size: 32px;
}

.honor-info {
  flex: 1;
}

.honor-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

/* 赛季历史 */
.history-card {
  border-radius: 12px;
}

.history-table {
  border-radius: 8px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ability-value {
  font-weight: 700;
}

.potential-value {
  color: #8b5cf6;
  font-weight: 600;
}

.text-gray {
  color: var(--text-placeholder);
}

:deep(.el-timeline-item__timestamp) {
  font-weight: 600;
  font-size: 14px;
  color: var(--primary-color);
}

/* 特性与状态区 */
.traits-condition-row {
  margin-bottom: 20px;
}

.traits-card,
.condition-card {
  border-radius: 12px;
  height: 100%;
}

.header-icon {
  font-size: 18px;
  margin-right: 4px;
}

/* 特性网格 */
.traits-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.trait-item {
  padding: 12px;
  border-radius: 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-light);
  transition: all 0.3s ease;
}

.trait-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

/* 特性稀有度样式 */
.trait-item.rarity-1 {
  border-left: 3px solid #9ca3af;
}

.trait-item.rarity-2 {
  border-left: 3px solid #22c55e;
}

.trait-item.rarity-3 {
  border-left: 3px solid #3b82f6;
}

.trait-item.rarity-4 {
  border-left: 3px solid #8b5cf6;
}

.trait-item.rarity-5 {
  border-left: 3px solid #f59e0b;
  background: linear-gradient(135deg, #fffbeb 0%, var(--bg-secondary) 100%);
}

.trait-item.negative {
  border-left-color: #ef4444;
  background: linear-gradient(135deg, #fef2f2 0%, var(--bg-secondary) 100%);
}

.trait-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.trait-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.trait-rarity {
  font-size: 12px;
  color: #f59e0b;
}

.trait-description {
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.4;
}

/* 状态面板 */
.condition-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.condition-display {
  text-align: center;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.condition-value {
  font-size: 48px;
  font-weight: 700;
  line-height: 1;
}

.condition-value.excellent {
  color: #22c55e;
}

.condition-value.good {
  color: #3b82f6;
}

.condition-value.normal {
  color: var(--text-primary);
}

.condition-value.poor {
  color: #f59e0b;
}

.condition-value.terrible {
  color: #ef4444;
}

.condition-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.condition-range {
  font-size: 12px;
  color: var(--text-placeholder);
  margin-top: 8px;
}

/* 状态因子 */
.condition-factors {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.factor-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.factor-label {
  font-size: 13px;
  color: var(--text-tertiary);
}

.factor-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.factor-value.hot {
  color: #ef4444;
}

.factor-value.warming {
  color: #f59e0b;
}

.factor-value.neutral {
  color: var(--text-secondary);
}

.factor-value.cooling {
  color: #3b82f6;
}

.factor-value.cold {
  color: #6366f1;
}
</style>
