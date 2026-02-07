<template>
  <div class="international-hall">
    <!-- 页面头部 -->
    <div class="hall-header">
      <div class="header-content">
        <h1>国 际 荣 誉 殿 堂</h1>
        <p class="subtitle">INTERNATIONAL HALL OF FAME</p>
        <p class="slogan">"记录每一个属于冠军的荣耀时刻"</p>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-radio-group v-model="selectedType" size="default">
        <el-radio-button value="all">全部</el-radio-button>
        <el-radio-button value="WorldChampionship">世界赛</el-radio-button>
        <el-radio-button value="Msi">MSI</el-radio-button>
        <el-radio-button value="MadridMasters">马德里大师赛</el-radio-button>
        <el-radio-button value="ShanghaiMasters">上海大师赛</el-radio-button>
        <el-radio-button value="other">其他国际赛</el-radio-button>
      </el-radio-group>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="8" animated />
    </div>

    <!-- 空状态 -->
    <el-empty v-else-if="filteredChampions.length === 0" description="暂无国际赛事冠军记录">
      <template #image>
        <div class="empty-icon"><el-icon :size="48"><Trophy /></el-icon></div>
      </template>
    </el-empty>

    <!-- 旗帜墙内容 -->
    <div v-else class="hall-content">
      <!-- 按赛事类型分组显示 -->
      <div v-for="(group, type) in groupedChampions" :key="type" class="tournament-section">
        <div class="section-header">
          <span class="section-icon">{{ getTournamentIcon(type) }}</span>
          <span class="section-title">{{ getTournamentDisplayName(type) }}</span>
        </div>
        <div class="section-divider"></div>

        <div class="champions-grid">
          <div
            v-for="champion in group"
            :key="champion.tournament_id"
            class="champion-card"
            :class="getTournamentClass(champion.tournament_type)"
            @click="toggleExpand(champion.tournament_id)"
          >
            <!-- 卡片收起状态 -->
            <div v-if="expandedId !== champion.tournament_id" class="card-collapsed">
              <div class="trophy-icon"><el-icon :size="24"><Trophy /></el-icon></div>
              <div class="team-name">{{ champion.champion_team_name }}</div>
              <div class="tournament-info">
                <span class="season">S{{ champion.season_id }}</span>
                <span class="tournament-short">{{ getTournamentShortName(champion.tournament_type) }}</span>
              </div>
              <div class="result">冠军</div>
              <div v-if="champion.final_score" class="final-score">
                {{ champion.final_score }}
              </div>
              <div class="expand-hint">点击查看详情</div>
            </div>

            <!-- 卡片展开状态 -->
            <div v-else class="card-expanded">
              <div class="expanded-header">
                <div class="trophy-icon large"><el-icon :size="48"><Trophy /></el-icon></div>
                <div class="team-name large">{{ champion.champion_team_name }}</div>
                <div class="tournament-full">
                  S{{ champion.season_id }} {{ champion.tournament_name }}
                </div>
                <div class="tournament-en">{{ getTournamentEnglishName(champion.tournament_type) }}</div>
                <div v-if="champion.final_score" class="final-info">
                  决赛: {{ champion.final_score }} vs {{ championDetails[champion.tournament_id]?.runner_up_team_name || '...' }}
                </div>
              </div>

              <!-- 加载详情 -->
              <div v-if="loadingDetail" class="detail-loading">
                <el-skeleton :rows="3" animated />
              </div>

              <div v-else-if="championDetails[champion.tournament_id]" class="expanded-content">
                <!-- 冠军阵容 -->
                <div class="roster-section">
                  <div class="roster-title">🥇 冠军阵容</div>
                  <div class="roster-grid">
                    <div
                      v-for="player in championDetails[champion.tournament_id].champion_roster"
                      :key="player.player_id"
                      class="roster-player"
                    >
                      <div class="player-name">{{ player.player_name }}</div>
                      <div class="player-position">{{ player.position }}</div>
                    </div>
                  </div>
                </div>

                <!-- 亚季殿军 -->
                <div class="rankings-section">
                  <div class="ranking-item silver">
                    🥈 亚军: {{ championDetails[champion.tournament_id].runner_up_team_name }}
                  </div>
                  <div v-if="championDetails[champion.tournament_id].third_team_name" class="ranking-item bronze">
                    🥉 季军: {{ championDetails[champion.tournament_id].third_team_name }}
                  </div>
                  <div v-if="championDetails[champion.tournament_id].fourth_team_name" class="ranking-item fourth">
                    4️⃣ 殿军: {{ championDetails[champion.tournament_id].fourth_team_name }}
                  </div>
                </div>
              </div>

              <div class="collapse-hint" @click.stop="expandedId = null">点击收起</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Trophy } from '@element-plus/icons-vue'
import { tauriApi, type InternationalChampionCard, type ChampionDetail } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('InternationalHall')

const loading = ref(true)
const loadingDetail = ref(false)
const champions = ref<InternationalChampionCard[]>([])
const championDetails = ref<Record<number, ChampionDetail>>({})
const selectedType = ref('all')
const expandedId = ref<number | null>(null)

// 获取国际赛事冠军
const loadChampions = async () => {
  loading.value = true
  try {
    const res = await tauriApi.honor.getInternationalChampions()
    champions.value = res || []
  } catch (error) {
    logger.error('Failed to load champions:', error)
    champions.value = []
  } finally {
    loading.value = false
  }
}

// 筛选冠军
const filteredChampions = computed(() => {
  if (selectedType.value === 'all') {
    return champions.value
  }
  if (selectedType.value === 'other') {
    const mainTypes = ['WorldChampionship', 'Msi', 'MadridMasters', 'ShanghaiMasters']
    return champions.value.filter(c => !mainTypes.includes(c.tournament_type))
  }
  return champions.value.filter(c => c.tournament_type === selectedType.value)
})

// 按赛事类型分组
const groupedChampions = computed(() => {
  const groups: Record<string, InternationalChampionCard[]> = {}
  for (const champion of filteredChampions.value) {
    const type = champion.tournament_type
    if (!groups[type]) {
      groups[type] = []
    }
    groups[type].push(champion)
  }
  // 按赛季排序（倒序，最新的在前）
  for (const type in groups) {
    groups[type].sort((a, b) => b.season_id - a.season_id)
  }
  return groups
})

// 展开/收起卡片
const toggleExpand = async (tournamentId: number) => {
  if (expandedId.value === tournamentId) {
    expandedId.value = null
    return
  }

  expandedId.value = tournamentId

  // 如果还没有加载详情，则加载
  if (!championDetails.value[tournamentId]) {
    loadingDetail.value = true
    try {
      const detail = await tauriApi.honor.getChampionDetail(tournamentId)
      championDetails.value[tournamentId] = detail
    } catch (error) {
      logger.error('Failed to load champion detail:', error)
    } finally {
      loadingDetail.value = false
    }
  }
}

// 获取赛事图标
const getTournamentIcon = (type: string): string => {
  const icons: Record<string, string> = {
    'WorldChampionship': '🌍',
    'Msi': '🌸',
    'MadridMasters': '🏔️',
    'ShanghaiMasters': '🌊',
    'ClaudeIntercontinental': '⚡',
    'IcpIntercontinental': '🌐',
    'SuperIntercontinental': '✨',
  }
  return icons[type] || '🏆'
}

// 获取赛事显示名称
const getTournamentDisplayName = (type: string): string => {
  const names: Record<string, string> = {
    'WorldChampionship': '世界赛 (World Championship)',
    'Msi': 'MSI (季中冠军赛)',
    'MadridMasters': '马德里大师赛 (Madrid Masters)',
    'ShanghaiMasters': '上海大师赛 (Shanghai Masters)',
    'ClaudeIntercontinental': 'Claude洲际赛',
    'IcpIntercontinental': 'ICP洲际赛',
    'SuperIntercontinental': 'Super洲际赛',
  }
  return names[type] || type
}

// 获取赛事简称
const getTournamentShortName = (type: string): string => {
  const names: Record<string, string> = {
    'WorldChampionship': '世界赛',
    'Msi': 'MSI',
    'MadridMasters': '马德里',
    'ShanghaiMasters': '上海',
    'ClaudeIntercontinental': 'Claude',
    'IcpIntercontinental': 'ICP',
    'SuperIntercontinental': 'Super',
  }
  return names[type] || type
}

// 获取赛事英文名
const getTournamentEnglishName = (type: string): string => {
  const names: Record<string, string> = {
    'WorldChampionship': 'World Championship',
    'Msi': 'Mid-Season Invitational',
    'MadridMasters': 'Madrid Masters',
    'ShanghaiMasters': 'Shanghai Masters',
    'ClaudeIntercontinental': 'Claude Intercontinental',
    'IcpIntercontinental': 'ICP Intercontinental',
    'SuperIntercontinental': 'Super Intercontinental',
  }
  return names[type] || type
}

// 获取赛事样式类
const getTournamentClass = (type: string): string => {
  const classes: Record<string, string> = {
    'WorldChampionship': 'worlds',
    'Msi': 'msi',
    'MadridMasters': 'madrid',
    'ShanghaiMasters': 'shanghai',
    'ClaudeIntercontinental': 'claude',
    'IcpIntercontinental': 'icp',
    'SuperIntercontinental': 'super',
  }
  return classes[type] || 'default'
}

onMounted(() => {
  loadChampions()
})
</script>

<style scoped lang="scss">
.international-hall {
  padding: 20px;
  background: linear-gradient(180deg, #1a1a2e 0%, #16213e 100%);
  min-height: 100vh;
}

.hall-header {
  text-align: center;
  padding: 40px 0;
  margin-bottom: 30px;

  .header-content {
    h1 {
      font-size: 36px;
      font-weight: bold;
      color: #ffd700;
      letter-spacing: 8px;
      text-shadow: 0 0 20px rgba(255, 215, 0, 0.5);
      margin-bottom: 10px;
    }

    .subtitle {
      font-size: 16px;
      color: #b8860b;
      letter-spacing: 4px;
      margin-bottom: 15px;
    }

    .slogan {
      font-size: 14px;
      color: #888;
      font-style: italic;
    }
  }
}

.filter-bar {
  display: flex;
  justify-content: center;
  margin-bottom: 30px;

  :deep(.el-radio-button__inner) {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    color: #ccc;

    &:hover {
      color: #ffd700;
    }
  }

  :deep(.el-radio-button__original-radio:checked + .el-radio-button__inner) {
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    border-color: #ffd700;
    color: #1a1a2e;
  }
}

.loading-container, .empty-icon {
  text-align: center;
  padding: 60px;
  font-size: 48px;
}

.hall-content {
  max-width: 1400px;
  margin: 0 auto;
}

.tournament-section {
  margin-bottom: 50px;

  .section-header {
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 15px;

    .section-icon {
      font-size: 28px;
    }

    .section-title {
      font-size: 22px;
      font-weight: bold;
      color: #fff;
    }
  }

  .section-divider {
    height: 2px;
    background: linear-gradient(90deg, #ffd700, transparent);
    margin-bottom: 25px;
  }
}

.champions-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
}

.champion-card {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  border: 2px solid transparent;

  &:hover {
    transform: translateY(-5px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }

  // 赛事类型边框颜色
  &.worlds { border-color: #ffd700; }
  &.msi { border-color: #9b59b6; }
  &.madrid { border-color: #e74c3c; }
  &.shanghai { border-color: #3498db; }
  &.claude { border-color: #e67e22; }
  &.icp { border-color: #2ecc71; }
  &.super { border-image: linear-gradient(45deg, #ff6b6b, #ffd93d, #6bcb77, #4d96ff, #9b59b6) 1; }
}

.card-collapsed {
  width: 160px;
  padding: 20px;
  text-align: center;

  .trophy-icon {
    font-size: 36px;
    margin-bottom: 12px;
  }

  .team-name {
    font-size: 18px;
    font-weight: bold;
    color: #fff;
    margin-bottom: 10px;
  }

  .tournament-info {
    font-size: 14px;
    color: #aaa;
    margin-bottom: 8px;

    .season {
      color: #ffd700;
      margin-right: 5px;
    }
  }

  .result {
    font-size: 16px;
    color: #ffd700;
    font-weight: bold;
    margin-bottom: 8px;
  }

  .final-score {
    font-size: 13px;
    color: #888;
  }

  .expand-hint {
    font-size: 12px;
    color: #666;
    margin-top: 12px;
  }
}

.card-expanded {
  width: 400px;
  padding: 25px;

  .expanded-header {
    text-align: center;
    padding-bottom: 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    margin-bottom: 20px;

    .trophy-icon.large {
      font-size: 48px;
      margin-bottom: 15px;
    }

    .team-name.large {
      font-size: 24px;
      font-weight: bold;
      color: #fff;
      margin-bottom: 10px;
    }

    .tournament-full {
      font-size: 16px;
      color: #ffd700;
      margin-bottom: 5px;
    }

    .tournament-en {
      font-size: 12px;
      color: #888;
      margin-bottom: 10px;
    }

    .final-info {
      font-size: 14px;
      color: #aaa;
    }
  }

  .roster-section {
    margin-bottom: 20px;

    .roster-title {
      font-size: 14px;
      color: #ffd700;
      margin-bottom: 12px;
    }

    .roster-grid {
      display: flex;
      justify-content: space-between;
      gap: 10px;

      .roster-player {
        flex: 1;
        text-align: center;
        padding: 10px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 8px;

        .player-name {
          font-size: 14px;
          color: #fff;
          font-weight: 500;
          margin-bottom: 4px;
        }

        .player-position {
          font-size: 12px;
          color: #888;
        }
      }
    }
  }

  .rankings-section {
    padding: 15px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;

    .ranking-item {
      padding: 8px 0;
      font-size: 14px;

      &.silver { color: #c0c0c0; }
      &.bronze { color: #cd7f32; }
      &.fourth { color: #888; }
    }
  }

  .collapse-hint {
    text-align: center;
    font-size: 12px;
    color: #666;
    margin-top: 15px;
    cursor: pointer;

    &:hover {
      color: #ffd700;
    }
  }
}

.detail-loading {
  padding: 20px;
}
</style>
