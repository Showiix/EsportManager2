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
    </div>

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
import { ref, onMounted } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { getCurrentMeta, getMetaHistory } from '@/api/tauri'
import type { MetaInfo, MetaHistoryEntry, MetaWeightsInfo } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import { ElMessage } from 'element-plus'

const gameStore = useGameStore()
const loading = ref(false)
const currentMeta = ref<MetaInfo | null>(null)
const history = ref<MetaHistoryEntry[]>([])

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

const fetchData = async () => {
  const saveId = gameStore.currentSave?.id
  if (!saveId) return

  loading.value = true
  try {
    const [meta, hist] = await Promise.all([
      getCurrentMeta(saveId),
      getMetaHistory(saveId),
    ])
    currentMeta.value = meta
    history.value = hist
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
