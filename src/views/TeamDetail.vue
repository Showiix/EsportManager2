<template>
  <div class="team-detail-view">
    <!-- 返回按钮 -->
    <div class="back-link">
      <el-button text @click="$router.push('/teams')">
        <el-icon><ArrowLeft /></el-icon>
        返回战队列表
      </el-button>
    </div>

    <!-- 战队头部信息 -->
    <el-card class="profile-card">
      <div class="profile-content">
        <!-- 战队头像区域 -->
        <div class="avatar-section">
          <div class="team-avatar large" :class="team.region.toLowerCase()">
            {{ team.name.substring(0, 2) }}
          </div>
          <el-tag :type="getRegionType(team.region)" size="large" effect="dark" class="region-tag">
            {{ team.region }}
          </el-tag>
        </div>

        <!-- 基本信息 -->
        <div class="info-section">
          <div class="team-header">
            <h1 class="team-name">{{ team.name }}</h1>
            <div class="team-tags">
              <el-tag type="success" size="default">活跃</el-tag>
              <el-button type="primary" size="small" @click="goToEdit">
                <el-icon><Edit /></el-icon>
                编辑战队
              </el-button>
            </div>
          </div>
          <p class="team-region-name">{{ getRegionFullName(team.region) }}</p>
          <div class="team-record">
            <span class="record-item">
              <span class="record-label">战绩</span>
              <span class="record-value">{{ team.wins }}胜 {{ team.losses }}负</span>
            </span>
            <el-divider direction="vertical" />
            <span class="record-item">
              <span class="record-label">胜率</span>
              <span class="record-value" :class="getWinRateClass(winRate)">{{ winRate }}%</span>
            </span>
          </div>
        </div>

        <!-- 战力值展示 -->
        <div class="stats-section">
          <div class="stat-number-display">
            <span class="power-value" :style="{ color: getPowerColor(team.power) }">{{ team.power.toFixed(1) }}</span>
            <span class="power-label">战力</span>
          </div>
          <div class="stat-text-group">
            <div class="stat-text-item">
              <span class="stat-value gold">{{ team.points }}</span>
              <span class="stat-label">年度积分</span>
            </div>
            <div class="stat-text-item">
              <span class="stat-value money">{{ formatMoney(team.balance) }}</span>
              <span class="stat-label">资金余额</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="24"><Check /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ team.wins }}</div>
              <div class="stat-label">胜场</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon red">
              <el-icon :size="24"><Close /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ team.losses }}</div>
              <div class="stat-label">负场</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon gold">
              <el-icon :size="24"><Star /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ team.points }}</div>
              <div class="stat-label">年度积分</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="24"><Wallet /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ formatMoney(team.balance) }}</div>
              <div class="stat-label">资金余额</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 首发阵容 -->
    <el-card class="roster-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><User /></el-icon>
            首发阵容
          </h2>
          <span class="count-badge">共 {{ players.length }} 名选手</span>
        </div>
      </template>

      <div class="roster-grid">
        <div
          v-for="player in players"
          :key="player.id"
          class="player-card"
          @click="goToPlayer(player.id)"
        >
          <div class="position-badge" :class="getPositionClass(player.position)">
            {{ player.position }}
          </div>
          <div class="player-avatar" :class="team.region.toLowerCase()">
            {{ player.game_id.substring(0, 1) }}
          </div>
          <div class="player-name">{{ player.game_id }}</div>
          <div class="player-stats">
            <span class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
              {{ player.ability }}
            </span>
          </div>
          <div class="player-details">
            <span class="detail-item">
              <span class="detail-label">潜力</span>
              <span class="detail-value purple">{{ player.potential }}</span>
            </span>
            <span class="detail-item">
              <span class="detail-label">年龄</span>
              <span class="detail-value">{{ player.age }}岁</span>
            </span>
          </div>
          <div class="player-salary">
            {{ formatMoney(player.salary) }}/年
          </div>
        </div>
      </div>
    </el-card>

    <!-- 替补阵容 -->
    <el-card class="roster-card substitute-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><UserFilled /></el-icon>
            替补阵容
          </h2>
          <span class="count-badge">共 {{ substitutePlayers.length }} 名选手</span>
        </div>
      </template>

      <el-empty v-if="substitutePlayers.length === 0" description="暂无替补选手">
        <template #image>
          <div class="empty-icon">👥</div>
        </template>
      </el-empty>

      <div v-else class="roster-grid substitute-grid">
        <div
          v-for="player in substitutePlayers"
          :key="player.id"
          class="player-card substitute"
          @click="goToPlayer(player.id)"
        >
          <div class="position-badge" :class="getPositionClass(player.position)">
            {{ player.position }}
          </div>
          <div class="player-avatar" :class="team.region.toLowerCase()">
            {{ player.game_id.substring(0, 1) }}
          </div>
          <div class="player-name">{{ player.game_id }}</div>
          <div class="player-stats">
            <span class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
              {{ player.ability }}
            </span>
          </div>
          <div class="player-details">
            <span class="detail-item">
              <span class="detail-label">潜力</span>
              <span class="detail-value purple">{{ player.potential }}</span>
            </span>
            <span class="detail-item">
              <span class="detail-label">年龄</span>
              <span class="detail-value">{{ player.age }}岁</span>
            </span>
          </div>
          <div class="player-salary">
            {{ formatMoney(player.salary) }}/年
          </div>
        </div>
      </div>
    </el-card>

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
          :key="`${honor.season_id}-${honor.tournament_id}`"
          :timestamp="`S${honor.season_id}`"
          placement="top"
          :color="getHonorColor(honor.honor_type)"
          size="large"
        >
          <el-card class="honor-card" :class="getHonorClass(honor.honor_type)" shadow="hover">
            <div class="honor-content">
              <div class="honor-icon">
                {{ getHonorEmoji(honor.honor_type) }}
              </div>
              <div class="honor-info">
                <div class="honor-title">{{ honor.tournament_name }}</div>
                <el-tag :type="getHonorTagType(honor.honor_type)" size="default" effect="dark">
                  {{ formatHonorType(honor.honor_type) }}
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

      <el-empty v-if="seasonHistory.length === 0" description="暂无赛季历史记录">
        <template #image>
          <div class="empty-icon">📊</div>
        </template>
      </el-empty>

      <el-table v-else :data="seasonHistory" stripe class="history-table">
        <el-table-column prop="season" label="赛季" width="120" align="center" />
        <el-table-column prop="wins" label="胜场" width="100" align="center">
          <template #default="{ row }">
            <span class="text-green">{{ row.wins }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="losses" label="负场" width="100" align="center">
          <template #default="{ row }">
            <span class="text-red">{{ row.losses }}</span>
          </template>
        </el-table-column>
        <el-table-column label="胜率" width="120" align="center">
          <template #default="{ row }">
            <span :class="getWinRateClass(row.winRate)">{{ row.winRate }}%</span>
          </template>
        </el-table-column>
        <el-table-column prop="points" label="积分" width="100" align="center">
          <template #default="{ row }">
            <span class="text-gold">{{ row.points }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="rank" label="排名" width="120" align="center">
          <template #default="{ row }">
            <el-tag :type="getRankTagType(row.rank)" size="small">
              第 {{ row.rank }} 名
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="荣誉" min-width="200">
          <template #default="{ row }">
            <el-tag v-if="row.achievement" :type="getAchievementType(row.achievement)" size="small" effect="dark">
              {{ row.achievement }}
            </el-tag>
            <span v-else class="text-gray">-</span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 战队历史故事 -->
    <el-card class="story-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><Document /></el-icon>
            战队传奇
          </h2>
          <el-tag type="info" effect="plain">{{ team.name }} 的故事</el-tag>
        </div>
      </template>

      <div class="story-content">
        <!-- 创立篇章 -->
        <div class="story-chapter">
          <div class="chapter-header">
            <div class="chapter-icon founding">
              <el-icon><Flag /></el-icon>
            </div>
            <div class="chapter-title">
              <h3>创立篇章</h3>
              <span class="chapter-subtitle">{{ teamStory.founding.year }}</span>
            </div>
          </div>
          <div class="chapter-content">
            <p>{{ teamStory.founding.story }}</p>
          </div>
        </div>

        <!-- 崛起之路 -->
        <div class="story-chapter">
          <div class="chapter-header">
            <div class="chapter-icon rise">
              <el-icon><TrendCharts /></el-icon>
            </div>
            <div class="chapter-title">
              <h3>崛起之路</h3>
              <span class="chapter-subtitle">奋斗与成长</span>
            </div>
          </div>
          <div class="chapter-content">
            <p>{{ teamStory.rise.story }}</p>
          </div>
        </div>

        <!-- 辉煌时刻 -->
        <div class="story-chapter" v-if="teamStory.glory">
          <div class="chapter-header">
            <div class="chapter-icon glory">
              <el-icon><Trophy /></el-icon>
            </div>
            <div class="chapter-title">
              <h3>辉煌时刻</h3>
              <span class="chapter-subtitle">荣耀加冕</span>
            </div>
          </div>
          <div class="chapter-content">
            <p>{{ teamStory.glory.story }}</p>
          </div>
        </div>

        <!-- 传奇人物 -->
        <div class="story-chapter">
          <div class="chapter-header">
            <div class="chapter-icon legends">
              <el-icon><Star /></el-icon>
            </div>
            <div class="chapter-title">
              <h3>传奇人物</h3>
              <span class="chapter-subtitle">队史名宿</span>
            </div>
          </div>
          <div class="chapter-content">
            <p>{{ teamStory.legends.story }}</p>
            <div class="legend-players" v-if="teamStory.legends.players?.length">
              <div
                v-for="player in teamStory.legends.players"
                :key="player.name"
                class="legend-player-item"
              >
                <span class="legend-name">{{ player.name }}</span>
                <span class="legend-title">{{ player.title }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 战队文化 -->
        <div class="story-chapter">
          <div class="chapter-header">
            <div class="chapter-icon culture">
              <el-icon><Aim /></el-icon>
            </div>
            <div class="chapter-title">
              <h3>战队文化</h3>
              <span class="chapter-subtitle">精神传承</span>
            </div>
          </div>
          <div class="chapter-content">
            <p>{{ teamStory.culture.story }}</p>
            <div class="team-motto" v-if="teamStory.culture.motto">
              <el-icon><ChatDotSquare /></el-icon>
              <span>"{{ teamStory.culture.motto }}"</span>
            </div>
          </div>
        </div>

        <!-- 展望未来 -->
        <div class="story-chapter">
          <div class="chapter-header">
            <div class="chapter-icon future">
              <el-icon><Sunrise /></el-icon>
            </div>
            <div class="chapter-title">
              <h3>展望未来</h3>
              <span class="chapter-subtitle">新的征程</span>
            </div>
          </div>
          <div class="chapter-content">
            <p>{{ teamStory.future.story }}</p>
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import {
  ArrowLeft,
  Edit,
  Check,
  Close,
  Star,
  Wallet,
  User,
  UserFilled,
  Trophy,
  Clock,
  Document,
  Flag,
  TrendCharts,
  Aim,
  ChatDotSquare,
  Sunrise,
} from '@element-plus/icons-vue'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'
import { honorApi, formatHonorType, type HonorRecord } from '@/api/tauri'
import { formatMoney } from '@/utils'
import { createLogger } from '@/utils/logger'

const logger = createLogger('TeamDetail')

const route = useRoute()
const router = useRouter()
const teamStore = useTeamStoreTauri()
const teamId = Number(route.params.id)

// 从 store 获取响应式数据
const { selectedTeam, starters, substitutes, regions } = storeToRefs(teamStore)

// 荣誉记录
const honors = ref<HonorRecord[]>([])

// 加载数据
onMounted(async () => {
  await teamStore.loadRegions()
  await teamStore.selectTeam(teamId)

  // 加载荣誉
  try {
    honors.value = await honorApi.getTeamHonors(teamId)
  } catch (e) {
    logger.error('Failed to load team honors:', e)
  }
})

// 监听路由参数变化
watch(() => route.params.id, async (newId) => {
  if (newId) {
    await teamStore.selectTeam(Number(newId))
    try {
      honors.value = await honorApi.getTeamHonors(Number(newId))
    } catch (e) {
      logger.error('Failed to load team honors:', e)
    }
  }
})

// 计算战队数据
const team = computed(() => {
  if (!selectedTeam.value) {
    return {
      id: teamId,
      name: '加载中...',
      shortName: '...',
      region: 'LPL',
      region_id: 1,
      power: 0,
      balance: 0,
      wins: 0,
      losses: 0,
      points: 0,
    }
  }
  return {
    id: selectedTeam.value.id,
    name: selectedTeam.value.name,
    shortName: selectedTeam.value.short_name || selectedTeam.value.name.substring(0, 3),
    region: getRegionCode(selectedTeam.value.region_id),
    region_id: selectedTeam.value.region_id,
    power: selectedTeam.value.power_rating,
    balance: selectedTeam.value.balance,
    wins: selectedTeam.value.wins,
    losses: selectedTeam.value.total_matches - selectedTeam.value.wins,
    points: selectedTeam.value.annual_points,
  }
})

// 选手列表
const players = computed(() => {
  return starters.value.map(p => ({
    id: p.id,
    game_id: p.game_id,
    position: getPositionShort(p.position || ''),
    ability: p.ability,
    potential: p.potential,
    age: p.age,
    salary: p.salary,
  }))
})

// 替补选手列表
const substitutePlayers = computed(() => {
  return substitutes.value.map(p => ({
    id: p.id,
    game_id: p.game_id,
    position: getPositionShort(p.position || ''),
    ability: p.ability,
    potential: p.potential,
    age: p.age,
    salary: p.salary,
  }))
})

// 获取赛区代码
const getRegionCode = (regionId: number) => {
  const region = regions.value.find(r => r.id === regionId)
  return region?.code ?? 'LPL'
}

// 位置简称转换
const getPositionShort = (position: string) => {
  const shorts: Record<string, string> = {
    Top: 'TOP', Jungle: 'JUG', Mid: 'MID', Adc: 'ADC', Support: 'SUP'
  }
  return shorts[position] || position
}

// 赛季历史（实际数据由后端获取，初始为空）
const seasonHistory = ref<any[]>([])

// 所有战队的历史故事
const allTeamStories: Record<number, any> = {
  // JDG
  1: {
    founding: {
      year: '2017年',
      story: 'JD Gaming（京东电子竞技俱乐部）成立于2017年，由京东集团出资创建。作为LPL赛区的后起之秀，JDG从成立之初就展现出了强大的决心和雄厚的资金实力，致力于打造一支世界顶级的电竞战队。'
    },
    rise: {
      story: '在2019年，JDG开始崭露头角，凭借着稳定的发挥和精妙的运营，逐渐成为LPL赛区的强队之一。战队以严谨的训练体系和出色的团队配合著称，每一场比赛都展现出LPL战队的最高水准。'
    },
    glory: {
      story: 'JDG在2022年迎来了巅峰时刻，接连获得MSI冠军和LPL夏季赛冠军，成为LPL赛区最具统治力的战队之一。凭借着Ruler、Kanavi、369等世界级选手的出色发挥，JDG向世界证明了自己的实力。'
    },
    legends: {
      story: '在JDG的历史中，涌现出众多优秀选手，他们用汗水和热血书写着属于JDG的传奇。',
      players: [
        { name: 'Ruler', title: '世界级ADC · 团战之王' },
        { name: 'Kanavi', title: '顶尖打野 · 节奏大师' },
        { name: '369', title: '上单核心 · 团战先锋' },
        { name: 'Yagao', title: '中路支柱 · 稳定输出' },
        { name: 'Missing', title: '冠军辅助 · 视野掌控' },
      ]
    },
    culture: {
      story: 'JDG的队训是"坚持不懈，追求卓越"。战队强调团队协作与个人实力的完美结合，每一位选手都被要求在赛场上展现最好的自己。正是这种追求完美的精神，让JDG成为LPL赛区最受尊敬的战队之一。',
      motto: '京东电竞，势不可挡'
    },
    future: {
      story: '新赛季，JDG将继续以冠军为目标，凭借着豪华的阵容配置和成熟的战术体系，向着更高的荣誉发起冲击。JDG的故事，远未结束...'
    }
  },
  // BLG
  2: {
    founding: {
      year: '2017年',
      story: 'Bilibili Gaming成立于2017年，由哔哩哔哩公司出资创建。作为年轻人文化的代表，BLG从一开始就带着独特的青春气息，致力于打造一支充满活力的电竞战队。'
    },
    rise: {
      story: 'BLG在近年来快速崛起，通过引进Knight、Bin等顶级选手，战队实力大幅提升。凭借着激进的打法风格和年轻选手的无限潜力，BLG成为LPL赛区最具观赏性的战队之一。'
    },
    glory: {
      story: 'BLG在2023年取得了重大突破，成功进入世界赛四强，向世界展示了LPL新生代力量的崛起。战队以敢打敢拼的风格，赢得了无数粉丝的支持和喜爱。'
    },
    legends: {
      story: '在BLG的成长历程中，涌现出众多天才选手，他们代表着LPL的未来和希望。',
      players: [
        { name: 'Knight', title: '中路天才 · 操作大师' },
        { name: 'Bin', title: '上单新星 · 极限操作' },
        { name: 'Elk', title: 'ADC新秀 · 团战收割' },
        { name: 'XUN', title: '打野核心 · 进攻先锋' },
        { name: 'ON', title: '辅助支柱 · 团队粘合剂' },
      ]
    },
    culture: {
      story: 'BLG的文化核心是"年轻、热血、无畏"。战队鼓励选手展现个人特色，在赛场上尽情表演。这种自由奔放的风格，让BLG成为最受年轻观众喜爱的战队。',
      motto: '干杯，为了热爱！'
    },
    future: {
      story: '新赛季，BLG将继续以年轻的姿态向冠军发起冲击。凭借着天才选手的组合，BLG的未来充满无限可能...'
    }
  },
  // TES
  3: {
    founding: {
      year: '2017年',
      story: 'Top Esports（滔搏电子竞技俱乐部）成立于2017年，由滔搏运动出资创建。战队以"顶级"为名，立志成为电竞领域的顶尖存在。'
    },
    rise: {
      story: 'TES在2020年迎来巅峰，接连斩获LPL春季赛和夏季赛冠军，成为当年LPL赛区的绝对霸主。战队以强势的对线实力和出色的团战能力著称。'
    },
    glory: {
      story: 'TES在2020年创造了辉煌的战绩，包揽了LPL春夏两季冠军，并在世界赛上展现出顶级竞争力。战队的成功证明了LPL选手的世界级实力。'
    },
    legends: {
      story: '在TES的历史中，涌现出众多实力派选手，他们用精湛的技术和稳定的发挥书写着TES的辉煌。',
      players: [
        { name: 'JackeyLove', title: 'S8世界冠军 · ADC天花板' },
        { name: 'Knight', title: '中路核心 · 操作之神' },
        { name: '369', title: '上单支柱 · 团战先锋' },
        { name: 'Karsa', title: '打野大师 · 经验丰富' },
        { name: 'Meiko', title: '辅助核心 · 团队领袖' },
      ]
    },
    culture: {
      story: 'TES的核心文化是"追求卓越，永不满足"。战队强调每一个细节的完善，每一次训练的认真，正是这种严谨的态度，铸就了TES的强大。',
      motto: '顶峰相见！'
    },
    future: {
      story: '新赛季，TES将继续向着世界冠军的目标前进。凭借着丰富的经验和强大的阵容，TES依然是LPL赛区最具竞争力的战队之一...'
    }
  },
  // WBG
  4: {
    founding: {
      year: '2020年',
      story: 'Weibo Gaming成立于2020年，由新浪微博收购原SN战队改组而来。作为社交媒体巨头旗下的战队，WBG拥有庞大的粉丝基础和强大的资金支持。'
    },
    rise: {
      story: 'WBG在2022年完成华丽转身，凭借着TheShy、SofM等明星选手的加盟，战队实力大幅提升，一度成为LPL赛区最具话题性的战队之一。'
    },
    glory: {
      story: 'WBG在2022年取得了不俗的成绩，成功进入世界赛，向世界展示了中国电竞的多元化发展。战队以娱乐性十足的比赛风格，吸引了大量观众。'
    },
    legends: {
      story: '在WBG的历史中，汇聚了众多个性鲜明的选手，他们用独特的风格为战队增添了无限魅力。',
      players: [
        { name: 'TheShy', title: '上单传奇 · 极限操作王' },
        { name: 'SofM', title: '越南打野 · 独特风格' },
        { name: 'Angel', title: '中单核心 · 稳定发挥' },
        { name: 'huanfeng', title: 'ADC新星 · 逆风翻盘专家' },
        { name: 'SwordArt', title: '辅助老将 · 经验丰富' },
      ]
    },
    culture: {
      story: 'WBG的文化是"娱乐与竞技并重"。战队注重选手的个人品牌打造，同时也追求竞技上的突破。这种独特的定位让WBG在电竞圈独树一帜。',
      motto: '微博电竞，热爱无限！'
    },
    future: {
      story: '新赛季，WBG将继续以独特的风格征战LPL。凭借着明星选手的号召力和不断进步的团队配合，WBG的未来值得期待...'
    }
  },
  // T1
  5: {
    founding: {
      year: '2012年',
      story: 'T1（前身为SK Telecom T1）成立于2012年，由韩国电信巨头SK电讯出资创建。作为韩国最具影响力的电竞俱乐部之一，T1从创立之初便以打造世界顶级战队为目标。在那个电竞刚刚萌芽的年代，T1率先建立了完善的选手培养体系和职业化管理模式，为日后的辉煌奠定了坚实的基础。'
    },
    rise: {
      story: '2013年，T1迎来了一位改变战队命运的天才少年——Faker。这位年仅17岁的中单选手以其惊人的天赋和对游戏的深刻理解，迅速成为战队的绝对核心。在Faker的带领下，T1开始了他们的王朝之路。战队以严格的训练制度和精妙的战术配合著称，每一次比赛都展现出韩国电竞的最高水准。'
    },
    glory: {
      story: 'T1是英雄联盟历史上最成功的战队，创造了无数不可复制的传奇。三次世界赛冠军（S3、S5、S6）、两次MSI冠军、十一座LCK联赛冠军奖杯...每一座奖杯背后都凝聚着队员们无数个日夜的汗水与付出。2016年的世界赛三连冠更是缔造了电竞史上前无古人的伟业，T1的名字永远镌刻在英雄联盟的荣誉殿堂。'
    },
    legends: {
      story: '在T1的历史长河中，涌现出无数传奇人物。他们用青春和热血书写着属于自己的故事，也为战队的荣耀添砖加瓦。',
      players: [
        { name: 'Faker', title: '不灭的大魔王 · 史上最伟大的选手' },
        { name: 'Bang', title: '世界赛双冠ADC · 团战输出机器' },
        { name: 'Wolf', title: '冠军辅助 · Faker最默契的搭档' },
        { name: 'MaRin', title: 'S5世界赛FMVP · 上单霸主' },
        { name: 'Bengi', title: '三冠王打野 · 永远的丛林之王' },
      ]
    },
    culture: {
      story: 'T1的队训是"永不言弃，追求卓越"。在这里，每一位选手都被要求以最高标准要求自己，无论是训练还是比赛。战队倡导的不仅是胜利，更是对电竞精神的完美诠释。正是这种追求极致的文化，让T1能够在起伏中始终保持顶尖的竞争力。',
      motto: 'We Are T1. We Never Give Up.'
    },
    future: {
      story: '新赛季，T1迎来了新的阵容变化，Zeus、Oner、Faker、Gumayusi和Keria组成的全新五人组正在向着新的高峰发起冲击。虽然前方的道路充满挑战，但T1的旗帜永远飘扬在最高处。王朝或许会有低谷，但传奇永不落幕。T1的故事，还在继续书写...'
    }
  },
  // Gen.G
  6: {
    founding: {
      year: '2017年',
      story: 'Gen.G成立于2017年，前身为三星Galaxy战队。作为LCK赛区的老牌豪门，Gen.G继承了三星战队的优良传统，致力于打造一支以稳定和智慧著称的顶级战队。'
    },
    rise: {
      story: 'Gen.G在近年来逐渐崛起，特别是在引进Chovy和Canyon等顶级选手后，战队实力大幅提升。凭借着精密的运营和出色的团队配合，Gen.G成为LCK赛区最具统治力的战队之一。'
    },
    glory: {
      story: 'Gen.G在2022年达到巅峰，包揽LCK春夏两季冠军，并在MSI上展现出强大的竞争力。战队以稳定著称，每一场比赛都展现出极高的完成度。'
    },
    legends: {
      story: '在Gen.G的历史中，涌现出众多实力派选手，他们用精湛的技术和稳定的发挥书写着战队的辉煌。',
      players: [
        { name: 'Chovy', title: '中路大师 · 对线之神' },
        { name: 'Canyon', title: '打野天才 · 节奏掌控者' },
        { name: 'Ruler', title: '世界级ADC · 团战核心' },
        { name: 'Doran', title: '上单支柱 · 稳定发挥' },
        { name: 'Lehends', title: '辅助核心 · 视野专家' },
      ]
    },
    culture: {
      story: 'Gen.G的核心文化是"稳定、智慧、团结"。战队强调科学的训练方法和数据驱动的决策，每一次比赛都经过周密的准备。这种专业的态度，让Gen.G成为LCK赛区最难击败的对手。',
      motto: 'We Are Gen.G. We Fight As One.'
    },
    future: {
      story: '新赛季，Gen.G将继续以冠军为目标前进。凭借着最豪华的阵容配置和成熟的团队体系，Gen.G有望创造更多辉煌...'
    }
  },
  // HLE
  7: {
    founding: {
      year: '2017年',
      story: 'Hanwha Life Esports成立于2017年，由韩华生命保险出资创建。作为传统企业涉足电竞的代表，HLE展现了韩国电竞产业的多元化发展。'
    },
    rise: {
      story: 'HLE在近年来稳步成长，通过青训体系培养出众多优秀选手。战队以团队配合和稳定发挥著称，逐渐在LCK赛区站稳脚跟。'
    },
    glory: {
      story: 'HLE在2023年取得了历史性突破，成功打入世界赛，向世界展示了韩国中游战队的潜力和决心。'
    },
    legends: {
      story: '在HLE的成长过程中，涌现出众多潜力新星，他们代表着LCK的新生力量。',
      players: [
        { name: 'Zeka', title: '中单核心 · 新生代希望' },
        { name: 'Viper', title: 'ADC天才 · 团战收割' },
        { name: 'Delight', title: '辅助新星 · 潜力无限' },
        { name: 'DuDu', title: '上单支柱 · 稳定发挥' },
        { name: 'Peanut', title: '打野老将 · 经验丰富' },
      ]
    },
    culture: {
      story: 'HLE的文化核心是"坚持与成长"。战队重视年轻选手的培养，相信每一个人都有无限的潜力。这种耐心培育的理念，让HLE成为LCK赛区最具发展潜力的战队。',
      motto: '韩华生命，点亮梦想！'
    },
    future: {
      story: '新赛季，HLE将继续以培养新人、冲击冠军为目标。凭借着年轻的阵容和积极的态度，HLE的未来充满希望...'
    }
  },
  // DK
  8: {
    founding: {
      year: '2020年',
      story: 'Dplus KIA（原Damwon Gaming）成立于2017年，2020年更名为DK。战队从LCK次级联赛一步步崛起，成为韩国电竞的新王者。'
    },
    rise: {
      story: 'DK在2020年实现了惊人的崛起，从默默无闻的新军一跃成为世界冠军。战队以激进的打法和超强的执行力著称，改变了LCK赛区的打法风格。'
    },
    glory: {
      story: 'DK在2020年创造了历史，以统治级的表现夺得世界赛冠军。Canyon更是获得FMVP荣誉，展现了新一代韩国选手的绝对实力。'
    },
    legends: {
      story: '在DK的辉煌历程中，涌现出众多世界级选手，他们用实力证明了新生代的崛起。',
      players: [
        { name: 'Nuguri', title: '上单天才 · S10 FMVP' },
        { name: 'Canyon', title: '打野之神 · 节奏主宰' },
        { name: 'ShowMaker', title: '中单核心 · 操作巅峰' },
        { name: 'Ghost', title: 'ADC支柱 · 稳定输出' },
        { name: 'BeryL', title: '辅助核心 · 团战指挥' },
      ]
    },
    culture: {
      story: 'DK的核心文化是"无畏前行"。战队鼓励选手展现个人特色，在比赛中敢于尝试创新打法。这种勇于挑战的精神，让DK成为LCK赛区最具活力的战队。',
      motto: 'DK Fighting!'
    },
    future: {
      story: '新赛季，DK将继续以重返巅峰为目标。虽然阵容经历变化，但战队的精神永不改变，DK的传奇仍在延续...'
    }
  },
  // G2
  9: {
    founding: {
      year: '2014年',
      story: 'G2 Esports成立于2014年，由Ocelote创建。作为欧洲最成功的电竞俱乐部，G2从一开始就以其独特的风格和个性，在电竞圈独树一帜。'
    },
    rise: {
      story: 'G2在2016年进入LEC后迅速崛起，凭借着Perkz等天才选手的加盟，战队开始了欧洲王朝的统治。G2以其创新的打法和娱乐精神，成为全球最具人气的战队之一。'
    },
    glory: {
      story: 'G2在2019年达到巅峰，包揽LEC春夏两季冠军和MSI冠军，并进入世界赛决赛。战队以其多样化的战术和出色的团队配合，向世界证明了欧洲的实力。'
    },
    legends: {
      story: '在G2的历史中，涌现出众多欧洲顶级选手，他们用实力和个性为G2增添了无限魅力。',
      players: [
        { name: 'Caps', title: '中单天才 · 欧洲之光' },
        { name: 'Jankos', title: '打野老将 · 第一血之王' },
        { name: 'Perkz', title: '灵魂核心 · 多面手' },
        { name: 'Wunder', title: '上单大师 · WOW玩家' },
        { name: 'Mikyx', title: '辅助核心 · 团战发起者' },
      ]
    },
    culture: {
      story: 'G2的文化核心是"娱乐至上，追求卓越"。战队以其幽默风趣的社交媒体运营和独特的队伍氛围著称。在G2，赢比赛很重要，但快乐同样重要。',
      motto: 'We Are G2. We Are Army.'
    },
    future: {
      story: '新赛季，G2将继续以欧洲冠军和世界赛突破为目标。凭借着丰富的经验和强大的品牌影响力，G2依然是LEC赛区的标杆战队...'
    }
  },
  // FNC
  10: {
    founding: {
      year: '2004年',
      story: 'Fnatic成立于2004年，是全球历史最悠久的电竞俱乐部之一。作为欧洲电竞的开拓者，Fnatic见证并参与了整个电竞行业的发展历程。'
    },
    rise: {
      story: 'Fnatic在S1世界赛上夺冠，成为历史上第一个英雄联盟世界冠军。战队以其深厚的底蕴和青训能力著称，培养出无数欧洲顶级选手。'
    },
    glory: {
      story: 'Fnatic是欧洲最成功的战队之一，多次获得LEC冠军，并在2018年再次进入世界赛决赛。战队的橙黑色队服已经成为欧洲电竞的标志。'
    },
    legends: {
      story: '在Fnatic的漫长历史中，涌现出无数欧洲电竞传奇，他们定义了欧洲英雄联盟的风格。',
      players: [
        { name: 'xPeke', title: '传奇中单 · 后门之王' },
        { name: 'Rekkles', title: 'ADC大师 · 欧洲传奇' },
        { name: 'Huni', title: '上单天才 · 世界游历者' },
        { name: 'Broxah', title: '打野核心 · 稳定发挥' },
        { name: 'Caps', title: '中单天才 · 欧洲之光' },
      ]
    },
    culture: {
      story: 'Fnatic的文化核心是"传承与创新"。战队尊重历史，同时也不断追求突破。在Fnatic，每一位选手都肩负着延续传奇的责任。',
      motto: 'Always Fnatic!'
    },
    future: {
      story: '新赛季，Fnatic将继续以重回巅峰为目标。作为欧洲电竞的象征，Fnatic的故事远未结束...'
    }
  },
  // MAD
  11: {
    founding: {
      year: '2017年',
      story: 'MAD Lions成立于2017年，作为LEC赛区的新生力量，MAD以其年轻、激进的风格迅速崭露头角。'
    },
    rise: {
      story: 'MAD Lions在2021年迎来巅峰，连续获得LEC春夏两季冠军，成为欧洲新王。战队以其凶猛的进攻风格和无畏的比赛态度著称。'
    },
    glory: {
      story: 'MAD Lions在2021年创造了队史最佳战绩，不仅包揽LEC两季冠军，还在MSI和世界赛上有出色表现，向世界展示了欧洲新生代的实力。'
    },
    legends: {
      story: '在MAD Lions的崛起过程中，涌现出众多欧洲新星，他们代表着LEC的未来。',
      players: [
        { name: 'Elyoya', title: '打野新星 · 进攻先锋' },
        { name: 'Humanoid', title: '中单核心 · 稳定输出' },
        { name: 'Carzzy', title: 'ADC天才 · 团战收割' },
        { name: 'Armut', title: '上单支柱 · 武器大师' },
        { name: 'Kaiser', title: '辅助核心 · 团战发起' },
      ]
    },
    culture: {
      story: 'MAD Lions的文化核心是"狂野与激情"。战队鼓励选手在比赛中释放本能，展现最真实的自我。这种无畏的精神，让MAD成为最具观赏性的战队之一。',
      motto: 'MAD Lions, Roar!'
    },
    future: {
      story: '新赛季，MAD Lions将继续以冠军为目标。凭借着年轻的阵容和激进的风格，MAD的未来充满可能...'
    }
  },
  // C9
  12: {
    founding: {
      year: '2013年',
      story: 'Cloud9成立于2013年，是北美最具传奇色彩的电竞俱乐部。战队从成立之初就展现出强大的竞争力，成为LCS赛区的标杆。'
    },
    rise: {
      story: 'C9在首个赛季就以25胜3负的惊人战绩统治LCS。战队以其独特的团队文化和出色的战术创新著称，多次在世界赛上为北美争光。'
    },
    glory: {
      story: 'C9是北美唯一一支每年都打入世界赛的战队，并在2018年创造了北美战队的最佳战绩——世界赛四强。战队的蓝白色标志已经成为北美电竞的象征。'
    },
    legends: {
      story: '在C9的历史中，涌现出众多北美顶级选手，他们定义了北美英雄联盟的风格。',
      players: [
        { name: 'Sneaky', title: 'ADC传奇 · C9元老' },
        { name: 'Jensen', title: '中单大师 · 北美之光' },
        { name: 'Blaber', title: '打野核心 · 节奏掌控' },
        { name: 'Meteos', title: '打野先驱 · 战术创新者' },
        { name: 'Hai', title: '创始人 · 团队领袖' },
      ]
    },
    culture: {
      story: 'C9的文化核心是"团结、创新、快乐"。战队强调团队氛围的重要性，相信快乐的选手才能发挥最好的水平。这种独特的管理哲学，让C9成为北美最稳定的战队。',
      motto: 'Cloud9, Always On Top!'
    },
    future: {
      story: '新赛季，C9将继续以世界赛突破为目标。作为北美电竞的象征，C9的传奇故事仍在书写...'
    }
  },
  // TL
  13: {
    founding: {
      year: '2000年',
      story: 'Team Liquid成立于2000年，最初是一个星际争霸社区。战队后来发展成为全球最大的电竞俱乐部之一，旗下拥有多个游戏项目的顶级战队。'
    },
    rise: {
      story: 'TL在2018年实现突破，通过引进Doublelift等顶级选手，战队开始了对LCS的统治。连续四次获得LCS冠军，创造了北美电竞的历史。'
    },
    glory: {
      story: 'TL是北美近年来最成功的战队，不仅统治国内赛场，还在MSI等国际赛事上有出色表现，证明了北美战队的国际竞争力。'
    },
    legends: {
      story: '在TL的历史中，汇聚了众多北美顶级选手，他们用实力证明了TL的统治地位。',
      players: [
        { name: 'Doublelift', title: 'ADC传奇 · 北美第一AD' },
        { name: 'CoreJJ', title: '辅助大师 · 世界冠军' },
        { name: 'Impact', title: '上单老将 · 世界冠军' },
        { name: 'Jensen', title: '中单核心 · 稳定发挥' },
        { name: 'Xmithie', title: '打野支柱 · 团队核心' },
      ]
    },
    culture: {
      story: 'TL的文化核心是"专业与卓越"。战队拥有业界顶级的训练设施和专业的管理团队，每一个细节都追求完美。这种专业的态度，让TL成为北美最值得尊敬的俱乐部。',
      motto: 'Team Liquid, Liquid Strong!'
    },
    future: {
      story: '新赛季，TL将继续以国内冠军和国际赛事突破为目标。凭借着雄厚的资源和专业的管理，TL的未来充满期待...'
    }
  },
  // FLY
  14: {
    founding: {
      year: '2017年',
      story: 'FlyQuest成立于2017年，由NBA密尔沃基雄鹿队老板Wesley Edens出资创建。战队以环保为核心理念，致力于用电竞的力量推动可持续发展。'
    },
    rise: {
      story: 'FLY在近年来稳步成长，通过培养年轻选手和合理的战术安排，战队逐渐成为LCS赛区的竞争力量。'
    },
    glory: {
      story: 'FLY在2020年取得了队史最佳战绩，成功打入世界赛，向世界展示了北美中游战队的潜力和决心。'
    },
    legends: {
      story: '在FLY的成长过程中，涌现出众多潜力选手，他们代表着北美电竞的希望。',
      players: [
        { name: 'Wildturtle', title: 'ADC老将 · 经验丰富' },
        { name: 'PowerOfEvil', title: '中单核心 · 稳定发挥' },
        { name: 'Santorin', title: '打野支柱 · 团队核心' },
        { name: 'Licorice', title: '上单新星 · 潜力无限' },
        { name: 'Ignar', title: '辅助核心 · 创意打法' },
      ]
    },
    culture: {
      story: 'FLY的文化核心是"绿色电竞"。战队将环保理念融入电竞运营，每一场胜利都会进行环保捐款。这种独特的社会责任感，让FLY在电竞圈独树一帜。',
      motto: 'FlyQuest, Rising Green!'
    },
    future: {
      story: '新赛季，FLY将继续以环保和竞技双丰收为目标。凭借着独特的理念和稳定的发挥，FLY的故事充满正能量...'
    }
  },
}

// 默认故事（用于未定义的战队）
const defaultStory = {
  founding: {
    year: '2020年',
    story: '这是一支新兴的电竞战队，怀揣着冠军的梦想加入了职业赛场。战队的创建者们相信，通过努力训练和团队协作，任何梦想都可以实现。'
  },
  rise: {
    story: '战队正在稳步成长，通过不断的训练和比赛积累经验。每一位选手都在努力提升自己，为战队的未来打下坚实的基础。'
  },
  glory: {
    story: '虽然战队还在成长阶段，但每一场比赛都是向冠军迈进的一步。未来的荣誉，正在等待着这支充满潜力的队伍。'
  },
  legends: {
    story: '战队的选手们正在用汗水书写属于自己的传奇，每一个人都有成为明星的潜力。',
    players: []
  },
  culture: {
    story: '战队以"团结、努力、进步"为核心理念，相信只要坚持不懈，就一定能够取得成功。',
    motto: '永不放弃，追逐梦想！'
  },
  future: {
    story: '新赛季充满挑战，也充满机遇。战队将继续前进，为荣誉而战，为梦想而拼搏...'
  }
}

// 根据战队ID获取对应的故事
const teamStory = ref(allTeamStories[Number(teamId)] || defaultStory)

// 计算属性
const winRate = computed(() => {
  const total = team.value.wins + team.value.losses
  if (total === 0) return 0
  return ((team.value.wins / total) * 100).toFixed(1)
})

// 方法
const goToEdit = () => {
  router.push(`/teams/${teamId}/edit`)
}

const goToPlayer = (playerId: number) => {
  router.push(`/players/${playerId}`)
}

// formatMoney 从 @/utils 导入

const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getRegionFullName = (region: string) => {
  const names: Record<string, string> = {
    LPL: '中国赛区 · LPL',
    LCK: '韩国赛区 · LCK',
    LEC: '欧洲赛区 · LEC',
    LCS: '北美赛区 · LCS',
  }
  return names[region] || region
}

const getPowerColor = (power: number) => {
  if (power >= 85) return '#ef4444'
  if (power >= 75) return '#f59e0b'
  if (power >= 65) return '#3b82f6'
  return '#22c55e'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getWinRateClass = (rate: number | string) => {
  const numRate = typeof rate === 'string' ? parseFloat(rate) : rate
  if (numRate >= 70) return 'text-green'
  if (numRate >= 50) return 'text-blue'
  return 'text-red'
}

const getPositionClass = (position: string) => {
  const classes: Record<string, string> = {
    TOP: 'top',
    JUG: 'jug',
    MID: 'mid',
    ADC: 'adc',
    SUP: 'sup',
    Top: 'top',
    Jug: 'jug',
    Mid: 'mid',
    Adc: 'adc',
    Sup: 'sup',
  }
  return classes[position] || ''
}

const getHonorColor = (honorType: string) => {
  const colors: Record<string, string> = {
    // 英文类型
    'TEAM_CHAMPION': '#fbbf24',
    'TEAM_RUNNER_UP': '#9ca3af',
    'TEAM_THIRD': '#f97316',
    'TEAM_FOURTH': '#3b82f6',
    'PLAYER_CHAMPION': '#fbbf24',
    'PLAYER_RUNNER_UP': '#9ca3af',
    'PLAYER_THIRD': '#f97316',
    'PLAYER_FOURTH': '#3b82f6',
    'TOURNAMENT_MVP': '#ef4444',
    'FINALS_MVP': '#ef4444',
    'REGULAR_SEASON_MVP': '#ef4444',
    'PLAYOFFS_FMVP': '#ef4444',
    // 中文类型
    '冠军': '#fbbf24',
    '亚军': '#9ca3af',
    '季军': '#f97316',
    '殿军': '#3b82f6',
  }
  return colors[honorType] || '#3b82f6'
}

const getHonorClass = (honorType: string) => {
  const classes: Record<string, string> = {
    // 英文类型
    'TEAM_CHAMPION': 'champion',
    'TEAM_RUNNER_UP': 'runner-up',
    'TEAM_THIRD': 'third-place',
    'TEAM_FOURTH': 'fourth-place',
    'PLAYER_CHAMPION': 'champion',
    'PLAYER_RUNNER_UP': 'runner-up',
    'PLAYER_THIRD': 'third-place',
    'PLAYER_FOURTH': 'fourth-place',
    // 中文类型
    '冠军': 'champion',
    '亚军': 'runner-up',
    '季军': 'third-place',
    '殿军': 'fourth-place',
  }
  return classes[honorType] || ''
}

const getHonorEmoji = (honorType: string) => {
  const emojis: Record<string, string> = {
    // 英文类型
    'TEAM_CHAMPION': '🏆',
    'TEAM_RUNNER_UP': '🥈',
    'TEAM_THIRD': '🥉',
    'TEAM_FOURTH': '4️⃣',
    'PLAYER_CHAMPION': '🏆',
    'PLAYER_RUNNER_UP': '🥈',
    'PLAYER_THIRD': '🥉',
    'PLAYER_FOURTH': '4️⃣',
    'TOURNAMENT_MVP': '⭐',
    'FINALS_MVP': '⭐',
    'REGULAR_SEASON_MVP': '⭐',
    'PLAYOFFS_FMVP': '⭐',
    'REGULAR_SEASON_FIRST': '🥇',
    // 中文类型
    '冠军': '🏆',
    '亚军': '🥈',
    '季军': '🥉',
    '殿军': '4️⃣',
  }
  return emojis[honorType] || '🏅'
}

const getHonorTagType = (honorType: string) => {
  const types: Record<string, string> = {
    // 英文类型
    'TEAM_CHAMPION': 'warning',
    'TEAM_RUNNER_UP': '',        // 默认银色
    'TEAM_THIRD': 'success',     // 绿色
    'TEAM_FOURTH': 'info',       // 蓝色
    'PLAYER_CHAMPION': 'warning',
    'PLAYER_RUNNER_UP': '',
    'PLAYER_THIRD': 'success',
    'PLAYER_FOURTH': 'info',
    'TOURNAMENT_MVP': 'danger',
    'FINALS_MVP': 'danger',
    'REGULAR_SEASON_MVP': 'danger',
    'PLAYOFFS_FMVP': 'danger',
    'REGULAR_SEASON_FIRST': 'primary',
    // 中文类型
    '冠军': 'warning',
    '亚军': '',
    '季军': 'success',
    '殿军': 'info',
  }
  return types[honorType] || 'primary'
}

const getRankTagType = (rank: number) => {
  if (rank <= 3) return 'danger'
  if (rank <= 6) return 'warning'
  return 'info'
}

const getAchievementType = (achievement: string) => {
  if (achievement.includes('冠军')) return 'warning'
  if (achievement.includes('亚军')) return 'info'
  return 'primary'
}
</script>

<style scoped>
.team-detail-view {
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

/* 战队资料卡片 */
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

.team-avatar.large {
  width: 120px;
  height: 120px;
  border-radius: 16px;
  font-size: 32px;
}

.region-tag {
  font-size: 14px;
}

.info-section {
  flex: 1;
}

.team-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 8px;
}

.team-name {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.team-tags {
  display: flex;
  gap: 8px;
  align-items: center;
}

.team-region-name {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

.team-record {
  display: flex;
  align-items: center;
  gap: 16px;
}

.record-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.record-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.record-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
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

.power-value {
  font-size: 36px;
  font-weight: 700;
  line-height: 1;
}

.power-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.stat-text-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stat-text-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-text-item .stat-value {
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
}

.stat-text-item .stat-value.gold {
  color: #fbbf24;
}

.stat-text-item .stat-value.money {
  color: #22c55e;
}

.stat-text-item .stat-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* 统计卡片行 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.green {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.stat-icon.red {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.stat-icon.gold {
  background: linear-gradient(135deg, #fbbf24, #f59e0b);
}

.stat-icon.blue {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

/* 阵容卡片 */
.roster-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.substitute-card {
  background: linear-gradient(135deg, #fafafa 0%, #f5f5f5 100%);
}

.substitute-card .card-header h2 {
  color: #6b7280;
}

.substitute-grid {
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
}

.player-card.substitute {
  background: var(--bg-secondary);
  opacity: 0.9;
}

.player-card.substitute:hover {
  opacity: 1;
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

.roster-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
}

.player-card {
  background: var(--bg-tertiary);
  border-radius: 12px;
  padding: 16px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
}

.player-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.position-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  color: white;
  margin-bottom: 12px;
}

.position-badge.top { background: linear-gradient(135deg, #ff6b6b, #ee5a5a); }
.position-badge.jug { background: linear-gradient(135deg, #51cf66, #40c057); }
.position-badge.mid { background: linear-gradient(135deg, #5c9fff, #4c8fef); }
.position-badge.adc { background: linear-gradient(135deg, #ffd43b, #fcc419); }
.position-badge.sup { background: linear-gradient(135deg, #cc5de8, #be4bdb); }

.player-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  font-size: 24px;
  margin: 0 auto 8px;
}

.player-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.player-stats {
  display: flex;
  justify-content: center;
  margin-bottom: 8px;
}

.ability-number {
  font-size: 24px;
  font-weight: 700;
}

.player-details {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-bottom: 8px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.detail-label {
  font-size: 10px;
  color: var(--text-tertiary);
}

.detail-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.detail-value.purple {
  color: #8b5cf6;
}

.player-salary {
  font-size: 12px;
  color: var(--text-tertiary);
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

.text-green { color: #22c55e; font-weight: 600; }
.text-red { color: #ef4444; font-weight: 600; }
.text-blue { color: #3b82f6; font-weight: 600; }
.text-gold { color: #fbbf24; font-weight: 600; }
.text-gray { color: var(--text-placeholder); }

:deep(.el-timeline-item__timestamp) {
  font-weight: 600;
  font-size: 14px;
  color: var(--primary-color);
}

/* 战队故事卡片 */
.story-card {
  border-radius: 12px;
  margin-top: 20px;
}

.story-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.story-chapter {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 12px;
  padding: 24px;
  transition: all 0.3s ease;
}

.story-chapter:hover {
  transform: translateX(4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.chapter-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.chapter-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 20px;
}

.chapter-icon.founding {
  background: linear-gradient(135deg, #6366f1, #4f46e5);
}

.chapter-icon.rise {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.chapter-icon.glory {
  background: linear-gradient(135deg, #fbbf24, #f59e0b);
}

.chapter-icon.legends {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.chapter-icon.culture {
  background: linear-gradient(135deg, #8b5cf6, #7c3aed);
}

.chapter-icon.future {
  background: linear-gradient(135deg, #06b6d4, #0891b2);
}

.chapter-title h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.chapter-subtitle {
  font-size: 13px;
  color: var(--text-tertiary);
}

.chapter-content {
  padding-left: 64px;
}

.chapter-content p {
  font-size: 15px;
  line-height: 1.8;
  color: var(--text-secondary);
  margin: 0;
  text-align: justify;
}

/* 传奇人物列表 */
.legend-players {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.legend-player-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 16px;
  background: white;
  border-radius: 8px;
  border-left: 3px solid #ef4444;
  transition: all 0.3s ease;
}

.legend-player-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.legend-name {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
}

.legend-title {
  font-size: 12px;
  color: var(--text-tertiary);
}

/* 战队格言 */
.team-motto {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
  padding: 16px 20px;
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
  border-radius: 8px;
  color: white;
}

.team-motto .el-icon {
  font-size: 24px;
  opacity: 0.9;
}

.team-motto span {
  font-size: 16px;
  font-weight: 600;
  font-style: italic;
}
</style>
