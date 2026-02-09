<template>
  <div class="international-hall">
    <!-- 页面头部 -->
    <div class="page-header">
      <div>
        <h1>国际荣誉殿堂</h1>
        <p>记录每一个属于冠军的荣耀时刻</p>
      </div>
      <button class="back-btn" @click="router.push('/honors')">← 返回荣誉殿堂</button>
    </div>

    <!-- 筛选栏 -->
    <div class="filter-section">
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
          <el-icon class="section-icon"><Trophy /></el-icon>
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
                  <div class="roster-title"><el-icon><Trophy /></el-icon> 冠军阵容</div>
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
                    <el-icon><Medal /></el-icon> 亚军: {{ championDetails[champion.tournament_id].runner_up_team_name }}
                  </div>
                  <div v-if="championDetails[champion.tournament_id].third_team_name" class="ranking-item bronze">
                    <el-icon><Medal /></el-icon> 季军: {{ championDetails[champion.tournament_id].third_team_name }}
                  </div>
                  <div v-if="championDetails[champion.tournament_id].fourth_team_name" class="ranking-item fourth">
                    <el-icon><Medal /></el-icon> 殿军: {{ championDetails[champion.tournament_id].fourth_team_name }}
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
import { useRouter } from 'vue-router'
import { Trophy, Medal } from '@element-plus/icons-vue'
import { tauriApi, type InternationalChampionCard, type ChampionDetail } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('InternationalHall')

const router = useRouter()
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

<style scoped>
.international-hall {
  padding: 0;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
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

.filter-section {
  margin-bottom: 16px;
}

.loading-container {
  padding: 40px;
}

.empty-icon {
  color: #cbd5e1;
}

.hall-content {
  margin: 0;
}

/* 赛事分组 */
.tournament-section {
  margin-bottom: 32px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.section-icon {
  font-size: 18px;
  color: #94a3b8;
}

.section-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
}

.section-divider {
  height: 1px;
  background: #f1f5f9;
  margin-bottom: 16px;
}

/* 卡片网格 */
.champions-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.champion-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.champion-card:hover {
  border-color: #6366f1;
  transform: translateY(-3px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.1);
}

.champion-card.worlds {
  border-left: 3px solid #f59e0b;
}

.champion-card.msi {
  border-left: 3px solid #8b5cf6;
}

.champion-card.madrid {
  border-left: 3px solid #ef4444;
}

.champion-card.shanghai {
  border-left: 3px solid #3b82f6;
}

.champion-card.claude {
  border-left: 3px solid #f97316;
}

.champion-card.icp {
  border-left: 3px solid #22c55e;
}

.champion-card.super {
  border-left: 3px solid #6366f1;
}

/* 卡片收起状态 */
.card-collapsed {
  width: 150px;
  padding: 14px;
  text-align: center;
}

.card-collapsed .trophy-icon {
  margin-bottom: 8px;
  color: #94a3b8;
}

.card-collapsed .team-name {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 6px;
}

.card-collapsed .tournament-info {
  font-size: 12px;
  color: #94a3b8;
  margin-bottom: 4px;
}

.card-collapsed .tournament-info .season {
  color: #6366f1;
  font-weight: 600;
  margin-right: 4px;
}

.card-collapsed .result {
  font-size: 12px;
  color: #0f172a;
  font-weight: 600;
  margin-bottom: 4px;
}

.card-collapsed .final-score {
  font-size: 11px;
  color: #94a3b8;
}

.card-collapsed .expand-hint {
  font-size: 11px;
  color: #cbd5e1;
  margin-top: 8px;
}

/* 卡片展开状态 */
.card-expanded {
  width: 360px;
  padding: 20px;
  background: #ffffff;
}

.expanded-header {
  text-align: center;
  padding-bottom: 14px;
  border-bottom: 1px solid #f1f5f9;
  margin-bottom: 14px;
}

.expanded-header .trophy-icon.large {
  margin-bottom: 10px;
  color: #94a3b8;
}

.expanded-header .team-name.large {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
  margin-bottom: 6px;
}

.expanded-header .tournament-full {
  font-size: 13px;
  color: #6366f1;
  margin-bottom: 2px;
}

.expanded-header .tournament-en {
  font-size: 11px;
  color: #cbd5e1;
  margin-bottom: 6px;
}

.expanded-header .final-info {
  font-size: 13px;
  color: #64748b;
}

.detail-loading {
  padding: 16px;
}

/* 冠军阵容 */
.roster-section {
  margin-bottom: 14px;
}

.roster-title {
  font-size: 13px;
  color: #0f172a;
  font-weight: 600;
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.roster-title .el-icon {
  color: #f59e0b;
}

.roster-grid {
  display: flex;
  gap: 6px;
}

.roster-player {
  flex: 1;
  text-align: center;
  padding: 8px 4px;
  background: #f8fafc;
  border: 1px solid #f1f5f9;
  border-radius: 6px;
}

.player-name {
  font-size: 12px;
  color: #0f172a;
  font-weight: 500;
  margin-bottom: 2px;
}

.player-position {
  font-size: 11px;
  color: #94a3b8;
}

/* 名次区 */
.rankings-section {
  padding: 12px;
  background: #f8fafc;
  border: 1px solid #f1f5f9;
  border-radius: 6px;
}

.ranking-item {
  padding: 4px 0;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.ranking-item.silver {
  color: #64748b;
}

.ranking-item.bronze {
  color: #d97706;
}

.ranking-item.fourth {
  color: #94a3b8;
}

.collapse-hint {
  text-align: center;
  font-size: 12px;
  color: #cbd5e1;
  margin-top: 12px;
  cursor: pointer;
  transition: color 0.15s;
}

.collapse-hint:hover {
  color: #6366f1;
}

.back-btn {
  padding: 5px 14px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #ffffff;
  color: #475569;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.back-btn:hover {
  border-color: #6366f1;
  color: #6366f1;
  background: #f5f3ff;
}
</style>
