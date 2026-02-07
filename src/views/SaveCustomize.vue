<template>
  <div class="save-customize-view">
    <!-- 顶部栏 -->
    <div class="page-header">
      <div class="header-left">
        <el-button link @click="handleBack">
          <el-icon><ArrowLeft /></el-icon>
          返回设置
        </el-button>
        <span class="header-title">自定义存档: 「{{ saveName }}」</span>
      </div>
      <div class="header-actions">
        <el-button @click="handleReset">
          <el-icon><RefreshLeft /></el-icon>
          重置全部
        </el-button>
        <el-button type="primary" :loading="isCreating" @click="handleCreate">
          <el-icon><Check /></el-icon>
          确认创建
        </el-button>
      </div>
    </div>

    <!-- 加载状态 -->
    <el-skeleton v-if="isLoading" :rows="10" animated />

    <!-- 主体内容 -->
    <template v-else-if="config">
      <!-- 赛区 Tab -->
      <el-tabs v-model="activeRegion" type="border-card" class="region-tabs">
        <el-tab-pane
          v-for="region in config.regions"
          :key="region.id"
          :label="region.name"
          :name="String(region.id)"
        >
          <!-- 队伍折叠面板 -->
          <el-collapse v-model="expandedTeams" accordion>
            <el-collapse-item
              v-for="(team, teamIdx) in region.teams"
              :key="teamIdx"
              :name="`${region.id}-${teamIdx}`"
            >
              <template #title>
                <div class="team-collapse-title">
                  <span class="team-name">{{ team.short_name }}</span>
                  <span class="team-full-name">{{ team.name }}</span>
                  <el-tag size="small" type="info">{{ formatMoney(team.initial_balance) }}</el-tag>
                  <el-tag size="small">{{ team.players.length }} 名选手</el-tag>
                </div>
              </template>

              <!-- 队伍编辑区 -->
              <div class="team-edit-section">
                <el-form :inline="true" class="team-basic-form">
                  <el-form-item label="全称">
                    <el-input v-model="team.name" style="width: 200px" />
                  </el-form-item>
                  <el-form-item label="简称">
                    <el-input v-model="team.short_name" style="width: 100px" />
                  </el-form-item>
                  <el-form-item label="资金(万元)">
                    <el-input-number
                      :model-value="team.initial_balance / 10000"
                      @update:model-value="(v: number) => team.initial_balance = (v || 0) * 10000"
                      :min="1000"
                      :max="50000"
                      :step="100"
                      style="width: 160px"
                    />
                  </el-form-item>
                </el-form>

                <!-- 选手表格 -->
                <el-table :data="team.players" border stripe size="small" class="player-table">
                  <el-table-column label="ID" width="120">
                    <template #default="{ row }">
                      <el-input v-model="row.game_id" size="small" />
                    </template>
                  </el-table-column>
                  <el-table-column label="真名" width="120">
                    <template #default="{ row }">
                      <el-input v-model="row.real_name" size="small" placeholder="-" />
                    </template>
                  </el-table-column>
                  <el-table-column label="国籍" width="80">
                    <template #default="{ row }">
                      <el-input v-model="row.nationality" size="small" />
                    </template>
                  </el-table-column>
                  <el-table-column label="位置" width="100">
                    <template #default="{ row }">
                      <el-select v-model="row.position" size="small">
                        <el-option label="Top" value="Top" />
                        <el-option label="Jug" value="Jug" />
                        <el-option label="Mid" value="Mid" />
                        <el-option label="Adc" value="Adc" />
                        <el-option label="Sup" value="Sup" />
                      </el-select>
                    </template>
                  </el-table-column>
                  <el-table-column label="年龄" width="100">
                    <template #default="{ row }">
                      <el-input-number v-model="row.age" size="small" :min="16" :max="40" controls-position="right" />
                    </template>
                  </el-table-column>
                  <el-table-column label="能力" width="100">
                    <template #default="{ row }">
                      <el-input-number v-model="row.ability" size="small" :min="40" :max="80" controls-position="right" />
                    </template>
                  </el-table-column>
                  <el-table-column label="潜力" width="100">
                    <template #default="{ row }">
                      <el-input-number v-model="row.potential" size="small" :min="40" :max="80" controls-position="right" />
                    </template>
                  </el-table-column>
                  <el-table-column label="首发" width="70" align="center">
                    <template #default="{ row }">
                      <el-switch v-model="row.is_starter" size="small" />
                    </template>
                  </el-table-column>
                  <el-table-column label="操作" width="60" align="center">
                    <template #default="{ $index }">
                      <el-button
                        link
                        type="danger"
                        size="small"
                        :disabled="team.players.length <= 5"
                        @click="removePlayer(team, $index)"
                      >
                        <el-icon><Delete /></el-icon>
                      </el-button>
                    </template>
                  </el-table-column>
                </el-table>

                <div class="team-actions">
                  <el-button size="small" @click="addPlayer(team, region)">
                    <el-icon><Plus /></el-icon>
                    添加选手
                  </el-button>
                  <el-button size="small" @click="resetTeam(region, teamIdx)">
                    <el-icon><RefreshLeft /></el-icon>
                    重置此队
                  </el-button>
                </div>
              </div>
            </el-collapse-item>
          </el-collapse>
        </el-tab-pane>
      </el-tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter, onBeforeRouteLeave } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  RefreshLeft,
  Check,
  Delete,
  Plus,
} from '@element-plus/icons-vue'
import { saveApi } from '@/api/tauri'
import { useGameStore } from '@/stores/useGameStore'
import type { GameInitConfig, RegionInitConfig, TeamInitConfig } from '@/types/initConfig'

const route = useRoute()
const router = useRouter()
const gameStore = useGameStore()

const saveName = ref('')
const isLoading = ref(false)
const isCreating = ref(false)
const config = ref<GameInitConfig | null>(null)
const defaultConfig = ref<GameInitConfig | null>(null)
const activeRegion = ref('1')
const expandedTeams = ref<string>('')

// 格式化金额（元 → 万元）
const formatMoney = (value: number) => {
  return `${(value / 10000).toFixed(0)}万元`
}

// 深拷贝配置
const cloneConfig = (c: GameInitConfig): GameInitConfig => {
  return JSON.parse(JSON.stringify(c))
}

// 加载默认配置
onMounted(async () => {
  saveName.value = (route.query.name as string) || ''
  if (!saveName.value) {
    ElMessage.error('缺少存档名称')
    router.push('/settings')
    return
  }

  isLoading.value = true
  try {
    const data = await saveApi.getDefaultGameConfig()
    defaultConfig.value = data
    config.value = cloneConfig(data)
  } catch (e) {
    ElMessage.error(`加载默认配置失败: ${e}`)
    router.push('/settings')
  } finally {
    isLoading.value = false
  }
})

// 离开页面前确认
onBeforeRouteLeave(async (_to, _from, next) => {
  if (config.value && defaultConfig.value) {
    const changed = JSON.stringify(config.value) !== JSON.stringify(defaultConfig.value)
    if (changed) {
      try {
        await ElMessageBox.confirm(
          '页面上的修改尚未保存，确定要离开吗？',
          '确认离开',
          {
            confirmButtonText: '离开',
            cancelButtonText: '留在此页',
            type: 'warning',
          }
        )
        next()
      } catch {
        next(false)
      }
      return
    }
  }
  next()
})

// 返回设置页
const handleBack = () => {
  router.push('/settings')
}

// 重置全部
const handleReset = async () => {
  if (!defaultConfig.value) return
  try {
    await ElMessageBox.confirm('确定要重置所有修改吗？', '重置确认', {
      confirmButtonText: '重置',
      cancelButtonText: '取消',
      type: 'warning',
    })
    config.value = cloneConfig(defaultConfig.value)
    ElMessage.success('已重置为默认数据')
  } catch {}
}

// 重置单队
const resetTeam = (region: RegionInitConfig, teamIdx: number) => {
  if (!defaultConfig.value) return
  const defaultRegion = defaultConfig.value.regions.find(r => r.id === region.id)
  if (defaultRegion && defaultRegion.teams[teamIdx]) {
    region.teams[teamIdx] = JSON.parse(JSON.stringify(defaultRegion.teams[teamIdx]))
    ElMessage.success('已重置此队')
  }
}

// 添加选手
const addPlayer = (team: TeamInitConfig, region: RegionInitConfig) => {
  const defaultNationality = region.short_name === 'CN' ? 'CN' : region.short_name === 'KR' ? 'KR' : region.short_name === 'EU' ? 'EU' : 'NA'
  team.players.push({
    game_id: `NewPlayer_${Date.now()}`,
    real_name: null,
    nationality: defaultNationality,
    position: 'Mid',
    age: 18,
    ability: 60,
    potential: 65,
    is_starter: false,
  })
}

// 删除选手
const removePlayer = (team: TeamInitConfig, index: number) => {
  if (team.players.length <= 5) {
    ElMessage.warning('每队至少需要5名选手')
    return
  }
  team.players.splice(index, 1)
}

// 验证配置
const validateConfig = (): string | null => {
  if (!config.value) return '配置数据未加载'

  const positions = ['Top', 'Jug', 'Mid', 'Adc', 'Sup']

  for (const region of config.value.regions) {
    for (const team of region.teams) {
      if (team.players.length < 5) {
        return `${region.name} - ${team.short_name}: 至少需要5名选手`
      }

      // 检查每个位置至少有1个首发
      for (const pos of positions) {
        const hasStarter = team.players.some(p => p.position === pos && p.is_starter)
        if (!hasStarter) {
          return `${region.name} - ${team.short_name}: ${pos} 位置缺少首发选手`
        }
      }

      // 检查每个位置最多1个首发
      for (const pos of positions) {
        const starterCount = team.players.filter(p => p.position === pos && p.is_starter).length
        if (starterCount > 1) {
          return `${region.name} - ${team.short_name}: ${pos} 位置有多个首发选手`
        }
      }

      // 检查数值范围
      for (const player of team.players) {
        if (player.ability < 40 || player.ability > 80) {
          return `${region.name} - ${team.short_name} - ${player.game_id}: 能力值需在40-80之间`
        }
        if (player.potential < 40 || player.potential > 80) {
          return `${region.name} - ${team.short_name} - ${player.game_id}: 潜力值需在40-80之间`
        }
        if (player.age < 16 || player.age > 40) {
          return `${region.name} - ${team.short_name} - ${player.game_id}: 年龄需在16-40之间`
        }
        if (!player.game_id.trim()) {
          return `${region.name} - ${team.short_name}: 存在空的选手ID`
        }
      }
    }
  }

  return null
}

// 创建存档
const handleCreate = async () => {
  if (!config.value) return

  const error = validateConfig()
  if (error) {
    ElMessage.error(error)
    return
  }

  isCreating.value = true
  try {
    await gameStore.createSaveWithConfig(saveName.value, config.value)
    // 创建成功后标记为无变更，避免离开提示
    defaultConfig.value = cloneConfig(config.value)
    ElMessage.success('自定义存档创建成功')
    router.push('/')
  } catch (e) {
    ElMessage.error(`创建存档失败: ${e}`)
  } finally {
    isCreating.value = false
  }
}
</script>

<style scoped>
.save-customize-view {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.header-actions {
  display: flex;
  gap: 10px;
}

.region-tabs {
  border-radius: 8px;
}

/* 队伍折叠标题 */
.team-collapse-title {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.team-name {
  font-weight: 700;
  font-size: 15px;
  color: var(--text-primary, #303133);
  min-width: 50px;
}

.team-full-name {
  font-size: 13px;
  color: var(--text-secondary, #606266);
  flex: 1;
}

/* 队伍编辑区 */
.team-edit-section {
  padding: 12px 0;
}

.team-basic-form {
  margin-bottom: 12px;
}

.player-table {
  margin-bottom: 12px;
}

.player-table :deep(.el-input-number) {
  width: 90px;
}

.team-actions {
  display: flex;
  gap: 8px;
}
</style>
