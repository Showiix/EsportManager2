<template>
  <div class="meta-version">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><Refresh /></el-icon>
          版本历史
        </h1>
        <p class="page-description">
          查看当前和历史赛季的 Meta 版本信息与位置权重
        </p>
      </div>
      <div class="header-actions">
        <SeasonSelector v-model="selectedSeason" />
      </div>
    </div>

    <!-- 版本英雄 Tier List -->
    <el-card class="tier-list-card" v-if="tierList.length > 0">
      <template #header>
        <div class="card-header">
          <span class="card-title">版本英雄强度</span>
          <span class="meta-type-label">{{ selectedMetaType }}</span>
        </div>
      </template>
      
      <div class="tier-groups">
        <!-- T1 版本之子 -->
        <div class="tier-group tier-t1" v-if="t1Champions.length > 0">
          <div class="tier-label">
            <span class="tier-badge t1">T1</span>
            <span class="tier-name">版本之子</span>
            <span class="tier-count">{{ t1Champions.length }}个</span>
          </div>
          <div class="champion-grid">
            <div v-for="c in t1Champions" :key="c.id" class="champion-card t1">
              <div class="champion-name">{{ c.name_cn }}</div>
              <div class="champion-tags">
                <el-tag :type="positionTagType(c.position)" size="small">{{ positionName(c.position) }}</el-tag>
                <el-tag type="info" size="small" effect="plain">{{ c.archetype_name }}</el-tag>
              </div>
            </div>
          </div>
        </div>
        
        <!-- T2 强势 -->
        <div class="tier-group tier-t2" v-if="t2Champions.length > 0">
          <div class="tier-label">
            <span class="tier-badge t2">T2</span>
            <span class="tier-name">强势</span>
            <span class="tier-count">{{ t2Champions.length }}个</span>
          </div>
          <div class="champion-grid">
            <div v-for="c in t2Champions" :key="c.id" class="champion-card t2">
              <div class="champion-name">{{ c.name_cn }}</div>
              <div class="champion-tags">
                <el-tag :type="positionTagType(c.position)" size="small">{{ positionName(c.position) }}</el-tag>
                <el-tag type="info" size="small" effect="plain">{{ c.archetype_name }}</el-tag>
              </div>
            </div>
          </div>
        </div>
        
        <!-- T3 标准 -->
        <div class="tier-group tier-t3" v-if="t3Champions.length > 0">
          <div class="tier-label">
            <span class="tier-badge t3">T3</span>
            <span class="tier-name">标准</span>
            <span class="tier-count">{{ t3Champions.length }}个</span>
          </div>
          <div class="champion-grid">
            <div v-for="c in t3Champions" :key="c.id" class="champion-card t3">
              <div class="champion-name">{{ c.name_cn }}</div>
              <div class="champion-tags">
                <el-tag :type="positionTagType(c.position)" size="small">{{ positionName(c.position) }}</el-tag>
                <el-tag type="info" size="small" effect="plain">{{ c.archetype_name }}</el-tag>
              </div>
            </div>
          </div>
        </div>
        
        <!-- T4 弱势 -->
        <div class="tier-group tier-t4" v-if="t4Champions.length > 0">
          <div class="tier-label">
            <span class="tier-badge t4">T4</span>
            <span class="tier-name">弱势</span>
            <span class="tier-count">{{ t4Champions.length }}个</span>
          </div>
          <div class="champion-grid">
            <div v-for="c in t4Champions" :key="c.id" class="champion-card t4">
              <div class="champion-name">{{ c.name_cn }}</div>
              <div class="champion-tags">
                <el-tag :type="positionTagType(c.position)" size="small">{{ positionName(c.position) }}</el-tag>
                <el-tag type="info" size="small" effect="plain">{{ c.archetype_name }}</el-tag>
              </div>
            </div>
          </div>
        </div>
        
        <!-- T5 版本弃子 -->
        <div class="tier-group tier-t5" v-if="t5Champions.length > 0">
          <div class="tier-label">
            <span class="tier-badge t5">T5</span>
            <span class="tier-name">版本弃子</span>
            <span class="tier-count">{{ t5Champions.length }}个</span>
          </div>
          <div class="champion-grid">
            <div v-for="c in t5Champions" :key="c.id" class="champion-card t5">
              <div class="champion-name">{{ c.name_cn }}</div>
              <div class="champion-tags">
                <el-tag :type="positionTagType(c.position)" size="small">{{ positionName(c.position) }}</el-tag>
                <el-tag type="info" size="small" effect="plain">{{ c.archetype_name }}</el-tag>
              </div>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 当前版本卡片 -->
    <el-card class="current-meta-card" v-loading="loading">
      <template #header>
        <div class="card-header">
          <span class="card-title">当前版本</span>
          <el-tag type="primary" size="large">S{{ currentMeta?.season_id }}</el-tag>
        </div>
      </template>

      <div class="current-meta-content" v-if="currentMeta">
        <div class="meta-info">
          <h2 class="meta-name">{{ currentMeta.meta_name }}</h2>
          <p class="meta-description">{{ currentMeta.description }}</p>
          <el-tag class="meta-type-tag">{{ currentMeta.meta_type }}</el-tag>
        </div>
        <div class="weights-chart">
          <div class="weight-bars">
            <div v-for="pos in positionList" :key="pos.key" class="weight-bar-item">
              <span class="position-label">{{ pos.label }}</span>
              <div class="bar-container">
                <div
                  class="bar-fill"
                  :style="{ width: getBarWidth(currentMeta.weights[pos.key as keyof MetaWeightsInfo]), backgroundColor: pos.color }"
                ></div>
              </div>
              <span class="weight-value">{{ currentMeta.weights[pos.key as keyof MetaWeightsInfo].toFixed(2) }}</span>
            </div>
          </div>
        </div>
      </div>

      <el-empty v-else description="暂无版本数据" />
    </el-card>

    <!-- 历史版本表格 -->
    <el-card class="history-card">
      <template #header>
        <div class="card-header">
          <span class="card-title">版本历史</span>
          <el-tag type="info" size="small">
            共 {{ history.length }} 个赛季
          </el-tag>
        </div>
      </template>

      <el-table :data="history" stripe style="width: 100%">
        <el-table-column prop="season_id" label="赛季" width="80" align="center">
          <template #default="{ row }">
            <el-tag :type="row.season_id === currentMeta?.season_id ? 'primary' : 'info'" size="small">
              S{{ row.season_id }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="meta_name" label="版本名称" width="140" />
        <el-table-column prop="meta_type" label="类型ID" width="160">
          <template #default="{ row }">
            <span class="mono-text">{{ row.meta_type }}</span>
          </template>
        </el-table-column>
        <el-table-column label="上单" width="90" align="center">
          <template #default="{ row }">
            <span :class="getWeightClass(row.weight_top)">{{ row.weight_top.toFixed(2) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="打野" width="90" align="center">
          <template #default="{ row }">
            <span :class="getWeightClass(row.weight_jug)">{{ row.weight_jug.toFixed(2) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="中路" width="90" align="center">
          <template #default="{ row }">
            <span :class="getWeightClass(row.weight_mid)">{{ row.weight_mid.toFixed(2) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="ADC" width="90" align="center">
          <template #default="{ row }">
            <span :class="getWeightClass(row.weight_adc)">{{ row.weight_adc.toFixed(2) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="辅助" width="90" align="center">
          <template #default="{ row }">
            <span :class="getWeightClass(row.weight_sup)">{{ row.weight_sup.toFixed(2) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="增强体系" width="220">
          <template #default="{ row }">
            <div class="comp-tags">
              <el-tag
                v-for="comp in getFavoredComps(row.meta_type)"
                :key="comp"
                type="success"
                size="small"
                effect="plain"
              >
                {{ COMP_NAME_MAP[comp] || comp }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="削弱体系" width="220">
          <template #default="{ row }">
            <div class="comp-tags">
              <el-tag
                v-for="comp in getNerfedComps(row.meta_type)"
                :key="comp"
                type="info"
                size="small"
                effect="plain"
                class="nerfed-tag"
              >
                {{ COMP_NAME_MAP[comp] || comp }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { getCurrentMeta, getMetaHistory, getChampionList } from '@/api/tauri'
import type { MetaInfo, MetaHistoryEntry, MetaWeightsInfo, ChampionInfo } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { useTimeStore } from '@/stores/useTimeStore'
import SeasonSelector from '@/components/common/SeasonSelector.vue'
import { ElMessage } from 'element-plus'

const gameStore = useGameStore()
const timeStore = useTimeStore()
const loading = ref(false)
const currentMeta = ref<MetaInfo | null>(null)
const history = ref<MetaHistoryEntry[]>([])
const selectedSeason = ref(timeStore.currentSeasonFromTime || 1)
const champions = ref<ChampionInfo[]>([])

// Meta type -> favored archetypes mapping
const META_FAVORED_ARCHETYPES: Record<string, string[]> = {
  Balanced: [],
  EarlyGameAggro: ['Aggressive'],
  DiveComposition: ['Aggressive'],
  SkirmishMeta: ['Aggressive'],
  PickComposition: ['Aggressive', 'Utility'],
  LateGameScaling: ['Scaling'],
  ProtectTheCarry: ['Scaling', 'Utility'],
  DualCarry: ['Scaling'],
  VisionControl: ['Utility'],
  SupportEra: ['Utility'],
  SplitPushMeta: ['Splitpush'],
  SoloLaneMeta: ['Splitpush'],
  TeamfightMeta: ['Teamfight'],
  ObjectiveControl: ['Teamfight', 'Utility'],
  MidKingdom: ['Aggressive', 'Scaling'],
  BotLaneDominance: ['Scaling', 'Utility'],
  TopLaneCarry: ['Splitpush', 'Aggressive'],
  JungleTempo: ['Aggressive'],
  MidJungleSynergy: ['Aggressive', 'Teamfight'],
  TopJungleSynergy: ['Splitpush', 'Aggressive'],
}

// Meta type -> disfavored archetypes mapping (mirrors backend disfavored_archetypes)
const META_DISFAVORED_ARCHETYPES: Record<string, string[]> = {
  Balanced: [],
  EarlyGameAggro: ['Scaling'],
  DiveComposition: ['Scaling'],
  SkirmishMeta: ['Scaling'],
  PickComposition: ['Scaling'],
  JungleTempo: ['Scaling'],
  LateGameScaling: ['Aggressive'],
  ProtectTheCarry: ['Aggressive'],
  DualCarry: ['Aggressive'],
  BotLaneDominance: ['Aggressive'],
  SplitPushMeta: ['Teamfight'],
  SoloLaneMeta: ['Teamfight'],
  TeamfightMeta: ['Splitpush'],
  ObjectiveControl: ['Splitpush'],
  VisionControl: [],
  SupportEra: [],
  MidKingdom: ['Splitpush'],
  TopLaneCarry: ['Scaling', 'Teamfight'],
  TopJungleSynergy: ['Scaling', 'Teamfight'],
  MidJungleSynergy: ['Scaling', 'Splitpush'],
}

// Position key mapping for champion position -> weight lookup
const POSITION_WEIGHT_KEY: Record<string, string> = {
  Top: 'top', Jug: 'jug', Mid: 'mid', Adc: 'adc', Sup: 'sup',
}

const selectedMetaType = computed(() => {
  const entry = history.value.find(h => h.season_id === selectedSeason.value)
  return entry?.meta_type || 'Balanced'
})

// Get position weights for the selected season's meta
const selectedMetaWeights = computed(() => {
  const entry = history.value.find(h => h.season_id === selectedSeason.value)
  if (!entry) return { top: 1, jug: 1, mid: 1, adc: 1, sup: 1 }
  return {
    top: entry.weight_top,
    jug: entry.weight_jug,
    mid: entry.weight_mid,
    adc: entry.weight_adc,
    sup: entry.weight_sup,
  }
})

type TierLevel = 'T1' | 'T2' | 'T3' | 'T4' | 'T5'

interface TierChampion extends ChampionInfo {
  tier: TierLevel
}

const tierList = computed<TierChampion[]>(() => {
  const metaType = selectedMetaType.value
  const favoredArchetypes = META_FAVORED_ARCHETYPES[metaType] || []
  const disfavoredArchetypes = META_DISFAVORED_ARCHETYPES[metaType] || []
  const weights = selectedMetaWeights.value
  
  return champions.value.map(c => {
    const archLower = c.archetype.toLowerCase()
    const posKey = POSITION_WEIGHT_KEY[c.position] || 'mid'
    const posWeight = weights[posKey as keyof typeof weights] ?? 1.0
    
    let tier: TierLevel
    if (favoredArchetypes.length === 0) {
      tier = 'T3' // Balanced meta: all standard
    } else if (favoredArchetypes.some(f => f.toLowerCase() === archLower)) {
      // Favored archetype: T1 if high-weight position, T2 otherwise
      tier = posWeight >= 1.1 ? 'T1' : 'T2'
    } else if (disfavoredArchetypes.some(f => f.toLowerCase() === archLower)) {
      // Disfavored archetype: T5 if low-weight position, T4 otherwise
      tier = posWeight <= 0.85 ? 'T5' : 'T4'
    } else {
      tier = 'T3' // Neutral
    }
    return { ...c, tier }
  })
})

const t1Champions = computed(() => tierList.value.filter(c => c.tier === 'T1'))
const t2Champions = computed(() => tierList.value.filter(c => c.tier === 'T2'))
const t3Champions = computed(() => tierList.value.filter(c => c.tier === 'T3'))
const t4Champions = computed(() => tierList.value.filter(c => c.tier === 'T4'))
const t5Champions = computed(() => tierList.value.filter(c => c.tier === 'T5'))

watch(selectedSeason, () => {
  // 更新 currentMeta 为选中赛季的数据
  const entry = history.value.find(h => h.season_id === selectedSeason.value)
  if (entry) {
    currentMeta.value = {
      season_id: entry.season_id,
      meta_type: entry.meta_type,
      meta_name: entry.meta_name,
      description: '', // MetaHistoryEntry 没有 description
      weights: {
        top: entry.weight_top,
        jug: entry.weight_jug,
        mid: entry.weight_mid,
        adc: entry.weight_adc,
        sup: entry.weight_sup,
      }
    }
  }
})

// Comp type -> core archetypes mapping
const COMP_CORE_ARCHETYPES: Record<string, string[]> = {
  Rush: ['Aggressive'], PickOff: ['Aggressive'], AllIn: ['Aggressive'],
  MidJungle: ['Aggressive'], TopJungle: ['Aggressive'],
  Protect: ['Scaling'], Stall: ['Scaling'], LateGame: ['Scaling'], DualCarry: ['Scaling'],
  Fortress: ['Utility'], UtilityComp: ['Utility'], Control: ['Utility'],
  Splitpush: ['Splitpush'], SideLane: ['Splitpush'], TripleThreat: ['Splitpush'],
  Teamfight: ['Teamfight'], Dive: ['Teamfight'], Skirmish: ['Teamfight'],
  BotLane: ['Scaling', 'Utility'],
  Flex: ['Aggressive', 'Scaling', 'Utility', 'Splitpush', 'Teamfight'],
}

const COMP_NAME_MAP: Record<string, string> = {
  Rush: '速推', PickOff: '抓单', AllIn: '莽夫', MidJungle: '中野联动', TopJungle: '上野联动',
  Protect: '保C', Fortress: '铁桶阵', UtilityComp: '功能流', Stall: '龟缩', BotLane: '下路统治',
  Teamfight: '团战', Dive: '开团', Skirmish: '小规模团战', DualCarry: '双C', Flex: '全能',
  Splitpush: '分推', SideLane: '4-1分带', Control: '运营', TripleThreat: '三线施压', LateGame: '后期发育',
}

const getFavoredComps = (metaType: string): string[] => {
  if (metaType === 'Balanced') return []
  const favoredArchetypes = META_FAVORED_ARCHETYPES[metaType] || []
  if (favoredArchetypes.length === 0) return []

  const favoredComps: string[] = []
  
  for (const [comp, archetypes] of Object.entries(COMP_CORE_ARCHETYPES)) {
    // Flex is special - always favored if meta has ANY favored archetype
    if (comp === 'Flex') {
      favoredComps.push(comp)
      continue
    }
    
    // Check if comp has ANY of the favored archetypes
    const isFavored = archetypes.some(a => favoredArchetypes.includes(a))
    if (isFavored) {
      favoredComps.push(comp)
    }
  }
  
  return favoredComps
}

const getNerfedComps = (metaType: string): string[] => {
  if (metaType === 'Balanced') return []
  const favoredArchetypes = META_FAVORED_ARCHETYPES[metaType] || []
  
  // If meta has no specific favored archetypes (unlikely unless Balanced), no nerfs
  if (favoredArchetypes.length === 0) return []

  const nerfedComps: string[] = []
  
  for (const [comp, archetypes] of Object.entries(COMP_CORE_ARCHETYPES)) {
    // Flex is never nerfed
    if (comp === 'Flex') continue
    
    // Comp is nerfed if NONE of its archetypes are favored
    const isFavored = archetypes.some(a => favoredArchetypes.includes(a))
    if (!isFavored) {
      nerfedComps.push(comp)
    }
  }
  
  return nerfedComps
}

const positionList = [
  { key: 'top', label: '上单', color: '#f56c6c' },
  { key: 'jug', label: '打野', color: '#e6a23c' },
  { key: 'mid', label: '中路', color: '#409eff' },
  { key: 'adc', label: 'ADC', color: '#67c23a' },
  { key: 'sup', label: '辅助', color: '#909399' },
]

const getBarWidth = (weight: number) => {
  // 权重范围 0.80 ~ 1.40，映射到 30% ~ 100%
  const min = 0.7
  const max = 1.5
  const pct = ((weight - min) / (max - min)) * 100
  return Math.max(10, Math.min(100, pct)) + '%'
}

const getWeightClass = (weight: number) => {
  if (weight >= 1.2) return 'weight-high'
  if (weight > 1.0) return 'weight-above'
  if (weight === 1.0) return 'weight-normal'
  if (weight >= 0.9) return 'weight-below'
  return 'weight-low'
}

const POSITION_NAME_MAP: Record<string, string> = {
  Top: '上单', Jug: '打野', Mid: '中路', Adc: 'ADC', Sup: '辅助',
}
const positionName = (pos: string) => POSITION_NAME_MAP[pos] || pos
const positionTagType = (pos: string): '' | 'success' | 'warning' | 'info' | 'danger' => {
  const map: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = {
    Top: 'danger', Jug: 'warning', Mid: '', Adc: 'success', Sup: 'info',
  }
  return map[pos] || 'info'
}

const fetchData = async () => {
  const saveId = gameStore.currentSave?.id
  if (!saveId) return

  loading.value = true
  try {
    const [meta, hist, list] = await Promise.all([
      getCurrentMeta(saveId),
      getMetaHistory(saveId),
      getChampionList(),
    ])
    currentMeta.value = meta
    history.value = hist
    champions.value = list
    
    // 如果有当前meta，设置选中赛季
    if (meta) {
      selectedSeason.value = meta.season_id
    }
  } catch (e: any) {
    ElMessage.error(`加载版本数据失败: ${e.message || e}`)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchData()
})
</script>

<style scoped lang="scss">
.meta-version {
  padding: 24px;
  min-height: 100%;

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;

    .header-content {
      .page-title {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 28px;
        font-weight: 700;
        margin: 0;
        color: #1f2937;

        .el-icon {
          color: #409eff;
        }
      }

      .page-description {
        margin: 8px 0 0 0;
        color: #6b7280;
        font-size: 14px;
      }
    }
  }

  .tier-list-card {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .card-title {
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }

      .meta-type-label {
        font-family: monospace;
        font-size: 13px;
        color: #6b7280;
      }
    }

    .tier-groups {
      display: flex;
      flex-direction: column;
      gap: 24px;
    }

    .tier-group {
      .tier-label {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-bottom: 12px;
        padding-bottom: 8px;
        border-bottom: 2px solid #f0f0f0;

        .tier-badge {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 24px;
          border-radius: 4px;
          font-size: 13px;
          font-weight: 700;
          color: white;

          &.t1 { background: linear-gradient(135deg, #f59e0b, #d97706); }
          &.t2 { background: linear-gradient(135deg, #3b82f6, #2563eb); }
          &.t3 { background: linear-gradient(135deg, #6b7280, #4b5563); }
          &.t4 { background: linear-gradient(135deg, #9ca3af, #d1d5db); color: #6b7280; }
          &.t5 { background: linear-gradient(135deg, #e5e7eb, #f3f4f6); color: #9ca3af; }
        }

        .tier-name {
          font-size: 15px;
          font-weight: 600;
          color: #374151;
        }

        .tier-count {
          font-size: 12px;
          color: #9ca3af;
        }
      }

      .champion-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 8px;
      }

      .champion-card {
        padding: 10px 12px;
        border-radius: 8px;
        border: 1px solid #e5e7eb;
        transition: all 0.2s;

        &:hover {
          transform: translateY(-1px);
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
        }

        &.t1 {
          background: linear-gradient(135deg, #fffbeb, #fef3c7);
          border-color: #fbbf24;

          .champion-name { color: #92400e; font-weight: 700; }
        }

        &.t2 {
          background: linear-gradient(135deg, #eff6ff, #dbeafe);
          border-color: #93c5fd;

          .champion-name { color: #1e40af; font-weight: 600; }
        }

        &.t3 {
          background: #f9fafb;
          border-color: #e5e7eb;

          .champion-name { color: #374151; font-weight: 600; }
        }

        &.t4 {
          background: #f3f4f6;
          border-color: #e5e7eb;
          opacity: 0.75;

          .champion-name { color: #9ca3af; font-weight: 500; }
        }

        &.t5 {
          background: #f9fafb;
          border-color: #f3f4f6;
          opacity: 0.55;

          .champion-name { color: #d1d5db; font-weight: 500; }
        }

        .champion-name {
          font-size: 14px;
          margin-bottom: 6px;
        }

        .champion-tags {
          display: flex;
          gap: 4px;
          flex-wrap: wrap;
        }
      }
    }
  }

  .current-meta-card {
    margin-bottom: 24px;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .card-title {
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }
    }

    .current-meta-content {
      display: flex;
      gap: 40px;
      align-items: flex-start;

      .meta-info {
        flex: 0 0 280px;

        .meta-name {
          font-size: 24px;
          font-weight: 700;
          color: #1f2937;
          margin: 0 0 8px 0;
        }

        .meta-description {
          color: #6b7280;
          font-size: 14px;
          line-height: 1.6;
          margin: 0 0 12px 0;
        }

        .meta-type-tag {
          font-family: monospace;
        }
      }

      .weights-chart {
        flex: 1;

        .weight-bars {
          display: flex;
          flex-direction: column;
          gap: 12px;

          .weight-bar-item {
            display: flex;
            align-items: center;
            gap: 12px;

            .position-label {
              flex: 0 0 40px;
              font-size: 14px;
              font-weight: 600;
              color: #374151;
              text-align: right;
            }

            .bar-container {
              flex: 1;
              height: 24px;
              background: #f3f4f6;
              border-radius: 12px;
              overflow: hidden;

              .bar-fill {
                height: 100%;
                border-radius: 12px;
                transition: width 0.6s ease;
              }
            }

            .weight-value {
              flex: 0 0 40px;
              font-size: 14px;
              font-weight: 600;
              color: #374151;
              font-family: monospace;
            }
          }
        }
      }
    }
  }

  .history-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .card-title {
        font-size: 18px;
        font-weight: 600;
        color: #1f2937;
      }
    }

    .mono-text {
      font-family: monospace;
      font-size: 13px;
      color: #6b7280;
    }

    .comp-tags {
      display: flex;
      flex-wrap: wrap;
      gap: 4px;
      
      .nerfed-tag {
        color: #909399;
        border-color: #e9e9eb;
        background-color: #f4f4f5;
      }
    }
  }
}

.weight-high {
  color: #f56c6c;
  font-weight: 700;
}

.weight-above {
  color: #e6a23c;
  font-weight: 600;
}

.weight-normal {
  color: #909399;
}

.weight-below {
  color: #67c23a;
  font-weight: 600;
}

.weight-low {
  color: #409eff;
  font-weight: 700;
}
</style>
