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
