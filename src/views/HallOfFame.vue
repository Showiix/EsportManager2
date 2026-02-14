<template>
  <div class="hall-of-fame-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <div class="back-link" @click="$router.push('/honors')">
          <el-icon><ArrowLeft /></el-icon>
          返回荣誉殿堂
        </div>
        <h1>名人堂</h1>
        <p>铭刻传奇选手的不朽荣耀</p>
      </div>
      
      <!-- 统计栏 -->
      <div class="stats-bar">
        <div class="stat-item">
          <span class="stat-value gold">{{ legendCount }}</span>
          <span class="stat-label">传奇选手</span>
        </div>
        <div class="stat-divider"></div>
        <div class="stat-item">
          <span class="stat-value silver">{{ hallOfFameCount }}</span>
          <span class="stat-label">名人堂成员</span>
        </div>
        <div class="stat-divider"></div>
        <div class="stat-item">
          <span class="stat-value">{{ maxScore }}</span>
          <span class="stat-label">最高评分</span>
        </div>
      </div>
    </div>

    <!-- 筛选和排序 -->
    <div class="filter-section">
      <div class="filter-group">
        <span class="filter-label">等级：</span>
        <el-radio-group v-model="tierFilter" size="small">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="Legend">传奇 (Legend)</el-radio-button>
          <el-radio-button value="HallOfFame">名人堂 (Hall of Fame)</el-radio-button>
        </el-radio-group>
      </div>
      
      <div class="filter-group">
        <span class="filter-label">位置：</span>
        <el-radio-group v-model="positionFilter" size="small">
          <el-radio-button value="all">全部</el-radio-button>
          <el-radio-button value="Top">上路</el-radio-button>
          <el-radio-button value="Jungle">打野</el-radio-button>
          <el-radio-button value="Mid">中路</el-radio-button>
          <el-radio-button value="ADC">ADC</el-radio-button>
          <el-radio-button value="Support">辅助</el-radio-button>
        </el-radio-group>
      </div>
      
      <div class="filter-group ml-auto">
        <span class="filter-label">排序：</span>
        <el-select v-model="sortBy" size="small" style="width: 120px">
          <el-option label="按总分" value="score" />
          <el-option label="按入选赛季" value="season" />
          <el-option label="按巅峰能力" value="ability" />
        </el-select>
      </div>
    </div>

    <!-- 内容区域 -->
    <div v-loading="loading" class="content-area">
      <el-empty v-if="filteredList.length === 0" description="暂无名人堂选手，选手退役后会自动评选">
        <template #image>
          <div class="empty-icon"><el-icon :size="48"><Trophy /></el-icon></div>
        </template>
      </el-empty>
      
      <div v-else class="hall-grid">
        <div 
          v-for="player in filteredList" 
          :key="player.id" 
          class="hall-card"
          :class="player.tier"
          @click="toggleExpand(player.id)"
        >
          <div class="card-main">
            <!-- 左侧：基本信息 -->
            <div class="player-info">
              <div class="player-header">
                <div class="player-name">{{ player.player_name }}</div>
                <div class="tier-badge" :class="player.tier">
                  {{ player.tier === 'Legend' ? '传奇' : '名人堂' }}
                </div>
              </div>
              
              <div class="meta-row">
                <el-tag size="small" effect="plain" class="position-tag">
                  {{ positionMap[player.position] || player.position }}
                </el-tag>
                <el-tag 
                  v-if="player.region_id" 
                  size="small" 
                  :type="getRegionType(player.region_id)"
                  effect="dark"
                >
                  {{ getRegionName(player.region_id) }}
                </el-tag>
                <span class="season-text">S{{ player.induction_season }} 入选</span>
              </div>
            </div>
            
            <!-- 右侧：数据 -->
            <div class="player-stats">
              <div class="score-box">
                <div class="score-val">{{ player.total_score }}</div>
                <div class="score-label">荣誉总分</div>
              </div>
            </div>
          </div>
          
          <!-- 详细数据行 -->
          <div class="stats-row">
            <div class="stat-mini">
              <span class="label">巅峰能力</span>
              <span class="val">{{ player.peak_ability || '-' }}</span>
            </div>
            <div class="stat-mini">
              <span class="label">职业生涯</span>
              <span class="val">{{ player.career_seasons || '-' }} 赛季</span>
            </div>
          </div>
          
          <!-- 展开区域：详细荣誉 -->
          <div v-if="expandedId === player.id" class="honors-detail">
            <div class="honors-grid">
              <div 
                v-for="(honor, idx) in parseHonors(player.honors_json)" 
                :key="idx"
                class="honor-item"
              >
                <div class="honor-count">{{ honor.count }}x</div>
                <div class="honor-name">{{ translateHonorCategory(honor.category) }}</div>
              </div>
            </div>
          </div>
          
          <!-- 展开提示 -->
          <div class="expand-hint">
            <el-icon :class="{ 'is-expanded': expandedId === player.id }"><ArrowDown /></el-icon>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Trophy, ArrowLeft, ArrowDown } from '@element-plus/icons-vue'
import { honorApi, type HallOfFameEntry } from '@/api/tauri'
import { createLogger } from '@/utils/logger'

const logger = createLogger('HallOfFame')

// 状态
const loading = ref(false)
const hallList = ref<HallOfFameEntry[]>([])
const tierFilter = ref('all')
const positionFilter = ref('all')
const sortBy = ref('score')
const expandedId = ref<number | null>(null)

// 映射常量
const positionMap: Record<string, string> = {
  'Top': '上路',
  'Jungle': '打野',
  'Mid': '中路',
  'ADC': 'ADC',
  'Support': '辅助',
}

const regionMap: Record<number, { name: string; type: string }> = {
  1: { name: 'LPL', type: 'danger' },
  2: { name: 'LCK', type: '' },
  3: { name: 'LEC', type: 'success' },
  4: { name: 'LCS', type: 'warning' },
}

const honorCategoryMap: Record<string, string> = {
  'super_champion': 'Super冠军',
  'international_champion': '国际赛冠军',
  'league_champion': '联赛冠军',
  'tournament_mvp': '赛事MVP',
  'annual_mvp': '年度MVP',
  'annual_all_pro': '年度最佳阵容',
  'annual_top20': '年度Top20',
  'long_career': '长青生涯',
  'high_peak': '巅峰实力',
}

// 计算属性
const legendCount = computed(() => hallList.value.filter(p => p.tier === 'Legend').length)
const hallOfFameCount = computed(() => hallList.value.filter(p => p.tier === 'HallOfFame').length)
const maxScore = computed(() => {
  if (hallList.value.length === 0) return 0
  return Math.max(...hallList.value.map(p => p.total_score))
})

const filteredList = computed(() => {
  let result = [...hallList.value]
  
  // 筛选等级
  if (tierFilter.value !== 'all') {
    result = result.filter(p => p.tier === tierFilter.value)
  }
  
  // 筛选位置
  if (positionFilter.value !== 'all') {
    result = result.filter(p => p.position === positionFilter.value)
  }
  
  // 排序
  result.sort((a, b) => {
    if (sortBy.value === 'score') {
      return b.total_score - a.total_score
    } else if (sortBy.value === 'season') {
      return b.induction_season - a.induction_season
    } else if (sortBy.value === 'ability') {
      return (b.peak_ability || 0) - (a.peak_ability || 0)
    }
    return 0
  })
  
  return result
})

// 生命周期
onMounted(async () => {
  await loadData()
})

// 方法
async function loadData() {
  loading.value = true
  try {
    const data = await honorApi.getHallOfFame()
    hallList.value = data
  } catch (error) {
    logger.error('Failed to load hall of fame data', error)
  } finally {
    loading.value = false
  }
}

function getRegionName(id: number | null): string {
  if (!id) return '未知'
  return regionMap[id]?.name || '未知'
}

function getRegionType(id: number | null): string {
  if (!id) return 'info'
  return regionMap[id]?.type || 'info'
}

function toggleExpand(id: number) {
  if (expandedId.value === id) {
    expandedId.value = null
  } else {
    expandedId.value = id
  }
}

interface HonorItem {
  category: string
  count: number
}

function parseHonors(jsonStr: string): HonorItem[] {
  try {
    return JSON.parse(jsonStr)
  } catch (e) {
    return []
  }
}

function translateHonorCategory(category: string): string {
  return honorCategoryMap[category] || category
}
</script>

<style scoped>
.hall-of-fame-view {
  padding: 0;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.back-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  color: #64748b;
  cursor: pointer;
  margin-bottom: 8px;
  transition: color 0.2s;
}

.back-link:hover {
  color: #3b82f6;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.page-header p {
  font-size: 14px;
  color: #94a3b8;
  margin: 0;
}

/* 统计栏 */
.stats-bar {
  display: flex;
  align-items: center;
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 12px 20px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0 16px;
  min-width: 80px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  line-height: 1.2;
}

.stat-value.gold { color: #f59e0b; }
.stat-value.silver { color: #94a3b8; }

.stat-label {
  font-size: 12px;
  color: #64748b;
}

.stat-divider {
  width: 1px;
  height: 24px;
  background: #e2e8f0;
}

/* 筛选区 */
.filter-section {
  display: flex;
  align-items: center;
  gap: 24px;
  background: white;
  padding: 16px 20px;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-label {
  font-size: 13px;
  color: #64748b;
  font-weight: 500;
}

.ml-auto {
  margin-left: auto;
}

/* 列表区 */
.hall-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 20px;
}

.hall-card {
  background: white;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.hall-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.08);
}

/* 传奇样式 */
.hall-card.Legend {
  border-left: 3px solid #f59e0b;
}

.hall-card.Legend:hover {
  border-color: #f59e0b;
}

/* 名人堂样式 */
.hall-card.HallOfFame {
  border-left: 3px solid #94a3b8;
}

.hall-card.HallOfFame:hover {
  border-color: #94a3b8;
}

.card-main {
  padding: 16px 20px;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.player-info {
  flex: 1;
}

.player-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.player-name {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.tier-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 600;
  text-transform: uppercase;
}

.tier-badge.Legend {
  background: #fffbeb;
  color: #d97706;
  border: 1px solid #fcd34d;
}

.tier-badge.HallOfFame {
  background: #f1f5f9;
  color: #475569;
  border: 1px solid #cbd5e1;
}

.meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.season-text {
  font-size: 12px;
  color: #94a3b8;
  margin-left: 4px;
}

.score-box {
  text-align: center;
  background: #f8fafc;
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid #f1f5f9;
}

.score-val {
  font-size: 20px;
  font-weight: 700;
  color: #3b82f6;
  line-height: 1;
}

.score-label {
  font-size: 11px;
  color: #64748b;
  margin-top: 2px;
}

.stats-row {
  display: flex;
  padding: 10px 20px;
  background: #f8fafc;
  border-top: 1px solid #f1f5f9;
  border-bottom: 1px solid #f1f5f9;
}

.stat-mini {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.stat-mini .label {
  font-size: 11px;
  color: #94a3b8;
}

.stat-mini .val {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
}

/* 荣誉详情 */
.honors-detail {
  padding: 16px 20px;
  background: #fff;
  border-bottom: 1px solid #f1f5f9;
  animation: slideDown 0.2s ease-out;
}

.honors-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.honor-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}

.honor-count {
  font-weight: 700;
  color: #f59e0b;
  min-width: 20px;
}

.honor-name {
  color: #475569;
}

/* 展开提示箭头 */
.expand-hint {
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #cbd5e1;
  background: #fff;
  transition: background 0.2s;
}

.hall-card:hover .expand-hint {
  background: #f8fafc;
  color: #94a3b8;
}

.expand-hint .el-icon {
  transition: transform 0.3s;
}

.expand-hint .el-icon.is-expanded {
  transform: rotate(180deg);
}

.empty-icon {
  color: #e2e8f0;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-5px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 响应式 */
@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    gap: 16px;
  }
  
  .stats-bar {
    width: 100%;
    justify-content: space-around;
  }
  
  .filter-section {
    gap: 16px;
  }
  
  .ml-auto {
    margin-left: 0;
    width: 100%;
  }
  
  .hall-grid {
    grid-template-columns: 1fr;
  }
}
</style>