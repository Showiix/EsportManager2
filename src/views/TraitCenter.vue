<template>
  <div class="trait-center">
    <div class="page-header">
      <div class="header-content">
        <h1 class="page-title">
          <el-icon><MagicStick /></el-icon>
          特性中心
        </h1>
        <p class="page-description">
          查看选手特性与特性图鉴，了解觉醒与退化条件
        </p>
      </div>
      <div class="header-actions">
        <el-button type="primary" @click="refreshData" :loading="loading">
          <el-icon><Refresh /></el-icon>
          刷新数据
        </el-button>
      </div>
    </div>

    <div class="dashboard-stats" v-if="!loading && allPlayers.length > 0">
      <el-card class="stat-card">
        <div class="stat-icon players">
          <el-icon><User /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ stats.totalPlayers }}</div>
          <div class="stat-label">参赛选手</div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-icon traits">
          <el-icon><MagicStick /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value highlight">{{ stats.totalTraits }}</div>
          <div class="stat-label">特性总数</div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-icon positive">
          <el-icon><Star /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value positive">{{ stats.playersWithTraits }}</div>
          <div class="stat-label">拥有特性</div>
        </div>
      </el-card>
      <el-card class="stat-card">
        <div class="stat-icon avg">
          <el-icon><TrendCharts /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ stats.avgTraits }}</div>
          <div class="stat-label">人均特性</div>
        </div>
      </el-card>
    </div>

    <el-tabs v-model="activeTab" class="main-tabs">
      <el-tab-pane label="选手特性" name="players">
        <div class="filter-bar">
          <el-select v-model="filterRegion" placeholder="全部赛区" clearable style="width: 140px">
            <el-option label="全部赛区" value="" />
            <el-option label="LPL" value="LPL" />
            <el-option label="LCK" value="LCK" />
            <el-option label="LEC" value="LEC" />
            <el-option label="LCS" value="LCS" />
          </el-select>
          <el-select v-model="filterPosition" placeholder="全部位置" clearable style="width: 140px">
            <el-option label="全部位置" value="" />
            <el-option label="上单" value="top" />
            <el-option label="打野" value="jungle" />
            <el-option label="中单" value="mid" />
            <el-option label="ADC" value="bot" />
            <el-option label="辅助" value="support" />
          </el-select>
          <el-select v-model="filterTraitType" placeholder="特性筛选" clearable style="width: 160px">
            <el-option label="全部特性" value="" />
            <el-option label="仅有特性" value="has" />
            <el-option label="无特性" value="none" />
          </el-select>
          <el-input
            v-model="searchText"
            placeholder="搜索选手名称"
            clearable
            style="width: 200px"
            :prefix-icon="Search"
          />
        </div>

        <el-table :data="paginatedPlayers" stripe style="width: 100%" max-height="600" v-loading="loading">
          <el-table-column label="选手" width="150" sortable :sort-by="(row: PlayerTraitEntry) => row.player_name">
            <template #default="{ row }">
              <span class="player-name">{{ row.player_name }}</span>
            </template>
          </el-table-column>
          <el-table-column label="战队" prop="team_name" width="140" />
          <el-table-column label="赛区" prop="region" width="80">
            <template #default="{ row }">
              <el-tag size="small" :type="getRegionTagType(row.region)">{{ row.region }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="位置" width="90" align="center">
            <template #default="{ row }">
              <el-tag :type="getPositionTagType(row.position)" size="small">
                {{ getPositionName(row.position) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="能力" prop="ability" width="80" sortable align="center" />
          <el-table-column label="年龄" prop="age" width="80" sortable align="center" />
          <el-table-column label="特性" min-width="300">
            <template #default="{ row }">
              <div class="trait-tags" v-if="row.traits.length > 0">
                <el-tooltip
                  v-for="trait in row.traits"
                  :key="trait.trait_type"
                  :content="trait.description"
                  placement="top"
                >
                  <span
                    class="trait-chip"
                    :class="trait.is_negative ? 'negative' : 'positive'"
                  >
                    <span class="trait-rarity">{{ '★'.repeat(trait.rarity) }}</span>
                    {{ trait.name }}
                  </span>
                </el-tooltip>
              </div>
              <span v-else class="no-trait">—</span>
            </template>
          </el-table-column>
        </el-table>

        <!-- 分页器 -->
        <div class="pagination-container" v-if="filteredPlayers.length > 0">
          <el-pagination
            v-model:current-page="currentPage"
            v-model:page-size="pageSize"
            :page-sizes="[20, 50, 100]"
            :total="filteredPlayers.length"
            layout="total, sizes, prev, pager, next, jumper"
            @size-change="handleSizeChange"
            @current-change="handlePageChange"
          />
        </div>
      </el-tab-pane>

      <el-tab-pane label="特性图鉴" name="catalog">
        <div class="filter-bar">
          <el-select v-model="catalogCategory" placeholder="全部分类" clearable style="width: 160px">
            <el-option label="全部分类" value="" />
            <el-option v-for="cat in categoryOptions" :key="cat.value" :label="cat.label" :value="cat.value" />
          </el-select>
          <el-select v-model="catalogRarity" placeholder="全部稀有度" clearable style="width: 140px">
            <el-option label="全部稀有度" value="" />
            <el-option label="★ 普通" :value="1" />
            <el-option label="★★ 稀有" :value="2" />
            <el-option label="★★★ 精良" :value="3" />
            <el-option label="★★★★ 史诗" :value="4" />
            <el-option label="★★★★★ 传说" :value="5" />
          </el-select>
          <el-switch v-model="showNegativeOnly" active-text="仅负面" inactive-text="" />
        </div>

        <div class="catalog-grid">
          <div
            v-for="(group, catKey) in groupedCatalog"
            :key="catKey"
            class="catalog-category"
          >
            <h3 class="category-title">
              <span class="category-icon">{{ getCategoryIcon(catKey) }}</span>
              {{ getCategoryName(catKey) }}
              <el-tag size="small" type="info">{{ group.length }}</el-tag>
            </h3>
            <div class="catalog-cards">
              <el-card
                v-for="entry in group"
                :key="entry.trait_type"
                :class="['trait-card', { negative: entry.is_negative }]"
                shadow="hover"
              >
                <div class="trait-card-header">
                  <span class="trait-card-name">{{ entry.name }}</span>
                  <span class="trait-card-rarity" :class="'rarity-' + entry.rarity">
                    {{ '★'.repeat(entry.rarity) }}
                  </span>
                </div>
                <p class="trait-card-desc">{{ entry.description }}</p>
                <div class="trait-card-conditions">
                  <div class="condition-row" v-if="entry.awakening_conditions !== '随机生成'">
                    <span class="condition-label awakening">觉醒</span>
                    <span class="condition-text">{{ entry.awakening_conditions }}</span>
                  </div>
                  <div class="condition-row" v-else>
                    <span class="condition-label random">随机</span>
                    <span class="condition-text">初始随机生成</span>
                  </div>
                  <div class="condition-row" v-if="entry.decay_conditions !== '无'">
                    <span class="condition-label decay">退化</span>
                    <span class="condition-text">{{ entry.decay_conditions }}</span>
                  </div>
                </div>
              </el-card>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { MagicStick, Refresh, User, Star, TrendCharts, Search } from '@element-plus/icons-vue'
import { traitCenterApi } from '@/api/tauri'
import type { PlayerTraitEntry, TraitCatalogEntry } from '@/api/tauri'

const loading = ref(false)
const activeTab = ref('players')
const allPlayers = ref<PlayerTraitEntry[]>([])
const catalog = ref<TraitCatalogEntry[]>([])

const filterRegion = ref('')
const filterPosition = ref('')
const filterTraitType = ref('')
const searchText = ref('')
const catalogCategory = ref('')
const catalogRarity = ref<number | ''>('')
const showNegativeOnly = ref(false)

// 分页状态
const currentPage = ref(1)
const pageSize = ref(20)

const categoryOptions = [
  { label: '大赛表现', value: 'big_game' },
  { label: '心态', value: 'mentality' },
  { label: '稳定性', value: 'stability' },
  { label: '体能', value: 'stamina' },
  { label: '队伍互动', value: 'team' },
  { label: '成长/衰退', value: 'growth' },
  { label: '特殊', value: 'special' },
  { label: '国际赛', value: 'international' },
]

const stats = computed(() => {
  const totalPlayers = allPlayers.value.length
  const playersWithTraits = allPlayers.value.filter(p => p.traits.length > 0).length
  const totalTraits = allPlayers.value.reduce((sum, p) => sum + p.traits.length, 0)
  const avgTraits = totalPlayers > 0 ? (totalTraits / totalPlayers).toFixed(1) : '0'
  return { totalPlayers, playersWithTraits, totalTraits, avgTraits }
})

const filteredPlayers = computed(() => {
  return allPlayers.value.filter(p => {
    if (filterRegion.value && p.region !== filterRegion.value) return false
    if (filterPosition.value && p.position !== filterPosition.value) return false
    if (filterTraitType.value === 'has' && p.traits.length === 0) return false
    if (filterTraitType.value === 'none' && p.traits.length > 0) return false
    if (searchText.value && !p.player_name.toLowerCase().includes(searchText.value.toLowerCase())) return false
    return true
  })
})

// 分页后的选手数据
const paginatedPlayers = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredPlayers.value.slice(start, end)
})

// 分页变化处理
const handlePageChange = (page: number) => {
  currentPage.value = page
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
}

// 筛选变化时重置页码
watch([filterRegion, filterPosition, filterTraitType, searchText], () => {
  currentPage.value = 1
})

const filteredCatalog = computed(() => {
  return catalog.value.filter(e => {
    if (catalogCategory.value && e.category !== catalogCategory.value) return false
    if (catalogRarity.value !== '' && e.rarity !== catalogRarity.value) return false
    if (showNegativeOnly.value && !e.is_negative) return false
    return true
  })
})

const groupedCatalog = computed(() => {
  const groups: Record<string, TraitCatalogEntry[]> = {}
  for (const entry of filteredCatalog.value) {
    if (!groups[entry.category]) groups[entry.category] = []
    groups[entry.category].push(entry)
  }
  return groups
})

function getPositionName(pos: string) {
  const map: Record<string, string> = { top: '上单', jungle: '打野', mid: '中单', bot: 'ADC', support: '辅助' }
  return map[pos] || pos
}

function getRegionTagType(region: string) {
  const map: Record<string, string> = { LPL: 'danger', LCK: 'primary', LEC: 'success', LCS: 'warning' }
  return (map[region] || 'info') as 'danger' | 'primary' | 'success' | 'warning' | 'info'
}

function getPositionTagType(position: string) {
  const map: Record<string, string> = { top: 'danger', jungle: 'warning', mid: 'primary', bot: 'success', support: 'info' }
  return (map[position] || 'info') as 'danger' | 'warning' | 'primary' | 'success' | 'info'
}

function getCategoryName(cat: string) {
  const map: Record<string, string> = {
    big_game: '大赛表现', mentality: '心态', stability: '稳定性',
    stamina: '体能', team: '队伍互动', growth: '成长/衰退',
    special: '特殊', international: '国际赛',
  }
  return map[cat] || cat
}

function getCategoryIcon(cat: string) {
  const map: Record<string, string> = {
    big_game: '🏆', mentality: '🧠', stability: '📊',
    stamina: '💪', team: '🤝', growth: '📈',
    special: '✨', international: '🌍',
  }
  return map[cat] || '📋'
}

async function refreshData() {
  loading.value = true
  try {
    const [players, catalogData] = await Promise.all([
      traitCenterApi.getAllPlayerTraits(filterRegion.value || undefined),
      traitCenterApi.getTraitCatalog(),
    ])
    allPlayers.value = players
    catalog.value = catalogData
  } catch (e) {
    console.error('Failed to load trait data:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => refreshData())
</script>

<style scoped lang="scss">
.trait-center {
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
          color: #8b5cf6;
        }
      }

      .page-description {
        margin: 8px 0 0 0;
        color: #6b7280;
        font-size: 14px;
      }
    }

    .header-actions {
      display: flex;
      gap: 12px;
    }
  }

  // 概览统计卡片
  .dashboard-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;

    .stat-card {
      :deep(.el-card__body) {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 20px;
      }

      .stat-icon {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 22px;
        color: white;

        &.players { background: linear-gradient(135deg, #667eea, #4f46e5); }
        &.traits { background: linear-gradient(135deg, #f59e0b, #d97706); }
        &.positive { background: linear-gradient(135deg, #22c55e, #16a34a); }
        &.avg { background: linear-gradient(135deg, #3b82f6, #1d4ed8); }
      }

      .stat-content {
        .stat-value {
          font-size: 24px;
          font-weight: 700;
          color: #1f2937;

          &.highlight { color: #d97706; }
          &.positive { color: #059669; }
        }

        .stat-label {
          font-size: 13px;
          color: #6b7280;
          margin-top: 2px;
        }
      }
    }
  }

  .main-tabs {
    margin-top: 8px;
  }

  .filter-bar {
    display: flex;
    gap: 12px;
    align-items: center;
    margin-bottom: 16px;
    flex-wrap: wrap;
    padding: 16px 20px;
    background: #f8fafc;
    border-radius: 12px;
  }

  .player-name {
    font-weight: 600;
    color: #1f2937;
  }

  .trait-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .trait-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 8px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: default;
    transition: transform 0.15s, box-shadow 0.15s;
    white-space: nowrap;

    &.positive {
      background: linear-gradient(135deg, #f0fdf4, #dcfce7);
      color: #15803d;
      border: 1px solid #bbf7d0;
    }

    &.negative {
      background: linear-gradient(135deg, #fef2f2, #fee2e2);
      color: #b91c1c;
      border: 1px solid #fecaca;
    }

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
    }

    .trait-rarity {
      font-size: 9px;
      color: #f59e0b;
    }
  }

  .no-trait {
    color: #c0c4cc;
  }

  // 分页容器
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid #ebeef5;
  }

  // 特性图鉴
  .catalog-grid {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .category-title {
    font-size: 18px;
    font-weight: 700;
    margin: 0 0 12px 0;
    display: flex;
    align-items: center;
    gap: 8px;
    color: #1f2937;

    .category-icon {
      font-size: 20px;
    }
  }

  .catalog-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 12px;
  }

  .trait-card {
    border-left: 4px solid #52c41a;
    transition: transform 0.2s;

    &:hover {
      transform: translateY(-2px);
    }

    &.negative {
      border-left-color: #f56c6c;
    }

    :deep(.el-card__body) {
      padding: 14px 16px;
    }

    .trait-card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 6px;
    }

    .trait-card-name {
      font-weight: 700;
      font-size: 15px;
      color: #1f2937;
    }

    .trait-card-rarity {
      font-size: 12px;
    }

    .trait-card-desc {
      font-size: 13px;
      color: #666;
      margin: 0 0 10px 0;
      line-height: 1.5;
    }

    .trait-card-conditions {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    .condition-row {
      display: flex;
      align-items: flex-start;
      gap: 8px;
      font-size: 12px;
    }

    .condition-label {
      flex-shrink: 0;
      padding: 1px 6px;
      border-radius: 4px;
      font-weight: 600;
      font-size: 11px;

      &.awakening {
        background: #e6f7ff;
        color: #1890ff;
      }

      &.decay {
        background: #fff1f0;
        color: #f56c6c;
      }

      &.random {
        background: #f6ffed;
        color: #52c41a;
      }
    }

    .condition-text {
      color: #888;
      line-height: 1.4;
    }
  }

  // 稀有度颜色
  .rarity-1 { color: #c0c4cc; }
  .rarity-2 { color: #52c41a; }
  .rarity-3 { color: #1890ff; }
  .rarity-4 { color: #722ed1; }
  .rarity-5 { color: #f5af19; }
}

@media (max-width: 1024px) {
  .trait-center {
    .dashboard-stats {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}
</style>
