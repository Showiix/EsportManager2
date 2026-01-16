<template>
  <el-dialog
    :model-value="modelValue"
    title="编辑选手属性"
    width="500px"
    @update:model-value="$emit('update:modelValue', $event)"
    @close="handleClose"
  >
    <template v-if="player">
      <!-- 选手基本信息 -->
      <div class="player-header">
        <div class="player-avatar" :class="player.region?.toLowerCase()">
          {{ player.gameId?.substring(0, 2) || '??' }}
        </div>
        <div class="player-info">
          <div class="player-name">{{ player.gameId }}</div>
          <div class="player-meta">
            <el-tag :type="getPositionType(player.position)" size="small">
              {{ player.position }}
            </el-tag>
            <span class="team-name">{{ player.team }}</span>
          </div>
        </div>
      </div>

      <el-divider />

      <!-- 编辑表单 -->
      <el-form :model="form" label-width="100px" label-position="left">
        <!-- 能力值 -->
        <el-form-item label="能力值">
          <div class="slider-row">
            <el-slider
              v-model="form.ability"
              :min="1"
              :max="100"
              :show-tooltip="false"
              class="ability-slider"
            />
            <el-input-number
              v-model="form.ability"
              :min="1"
              :max="100"
              size="small"
              controls-position="right"
              class="value-input"
            />
          </div>
          <div class="attr-hint">
            <span :style="{ color: getAbilityColor(form.ability) }">
              {{ getAbilityLevel(form.ability) }}
            </span>
            - 直接影响比赛发挥和胜负概率
          </div>
        </el-form-item>

        <!-- 潜力值 -->
        <el-form-item label="潜力值">
          <div class="slider-row">
            <el-slider
              v-model="form.potential"
              :min="1"
              :max="100"
              :show-tooltip="false"
              class="potential-slider"
            />
            <el-input-number
              v-model="form.potential"
              :min="1"
              :max="100"
              size="small"
              controls-position="right"
              class="value-input"
            />
          </div>
          <div class="attr-hint">
            <span class="potential-text">成长上限</span>
            - 影响选手未来成长空间
          </div>
        </el-form-item>

        <!-- 稳定性 -->
        <el-form-item label="稳定性">
          <div class="slider-row">
            <el-slider
              v-model="form.stability"
              :min="1"
              :max="100"
              :show-tooltip="false"
              class="stability-slider"
            />
            <el-input-number
              v-model="form.stability"
              :min="1"
              :max="100"
              size="small"
              controls-position="right"
              class="value-input"
            />
          </div>
          <div class="attr-hint">
            <span class="stability-text">{{ getStabilityLevel(form.stability) }}</span>
            - 稳定性越高，发挥波动越小 (σ = {{ ((100 - form.stability) / 10).toFixed(1) }})
          </div>
        </el-form-item>

        <!-- 年龄 -->
        <el-form-item label="年龄">
          <div class="slider-row">
            <el-slider
              v-model="form.age"
              :min="16"
              :max="45"
              :show-tooltip="false"
              class="age-slider"
            />
            <el-input-number
              v-model="form.age"
              :min="16"
              :max="45"
              size="small"
              controls-position="right"
              class="value-input"
            />
          </div>
          <div class="attr-hint">
            <span class="age-text">{{ getAgeLevel(form.age) }}</span>
            - 影响状态范围和基础稳定性
          </div>
        </el-form-item>
      </el-form>

      <!-- 影响说明 -->
      <el-alert type="info" :closable="false" class="impact-alert">
        <template #title>
          <strong>修改说明</strong>
        </template>
        修改这些属性会直接影响比赛模拟结果。能力值和稳定性是核心参数，年龄影响状态波动范围。
      </el-alert>
    </template>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :loading="saving" @click="handleSave">
        保存修改
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { playerApi } from '@/api/tauri'

interface PlayerData {
  id: number
  gameId: string
  name: string
  team: string
  region: string
  position: string
  age: number
  ability: number
  potential: number
  stability?: number
}

const props = defineProps<{
  modelValue: boolean
  player: PlayerData | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'close'): void
  (e: 'saved', player: PlayerData): void
}>()

const saving = ref(false)

const form = reactive({
  ability: 75,
  potential: 80,
  stability: 70,
  age: 22,
})

// 监听player变化，初始化表单
watch(() => props.player, (newPlayer) => {
  if (newPlayer) {
    form.ability = newPlayer.ability
    form.potential = newPlayer.potential
    form.stability = newPlayer.stability || 70
    form.age = newPlayer.age
  }
}, { immediate: true })

// 获取位置标签类型
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

// 获取能力值颜色
const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  if (ability >= 60) return '#22c55e'
  return '#6b7280'
}

// 获取能力等级描述
const getAbilityLevel = (ability: number) => {
  if (ability >= 90) return '世界顶级'
  if (ability >= 85) return '顶尖选手'
  if (ability >= 80) return '明星选手'
  if (ability >= 75) return '主力水平'
  if (ability >= 70) return '合格首发'
  if (ability >= 60) return '替补水平'
  return '新人'
}

// 获取稳定性等级描述
const getStabilityLevel = (stability: number) => {
  if (stability >= 90) return '极其稳定'
  if (stability >= 80) return '非常稳定'
  if (stability >= 70) return '较为稳定'
  if (stability >= 60) return '一般'
  return '不稳定'
}

// 获取年龄阶段描述
const getAgeLevel = (age: number) => {
  if (age <= 18) return '新秀期'
  if (age <= 22) return '成长期'
  if (age <= 26) return '巅峰期'
  if (age <= 30) return '稳定期'
  return '老将'
}

// 关闭弹窗
const handleClose = () => {
  emit('update:modelValue', false)
  emit('close')
}

// 保存修改
const handleSave = async () => {
  if (!props.player) return

  saving.value = true
  try {
    await playerApi.updatePlayer({
      player_id: props.player.id,
      ability: form.ability,
      potential: form.potential,
      stability: form.stability,
      age: form.age,
    })

    ElMessage.success(`选手 ${props.player.gameId} 属性已更新`)

    // 触发保存事件，传递更新后的数据
    emit('saved', {
      ...props.player,
      ability: form.ability,
      potential: form.potential,
      stability: form.stability,
      age: form.age,
    })

    handleClose()
  } catch (e: any) {
    ElMessage.error('保存失败: ' + e.message)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
/* 选手头部 */
.player-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.player-avatar {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 18px;
  background: linear-gradient(135deg, #667eea, #764ba2);
}

.player-avatar.lpl { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-avatar.lck { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-avatar.lec { background: linear-gradient(135deg, #22c55e, #16a34a); }
.player-avatar.lcs { background: linear-gradient(135deg, #f59e0b, #d97706); }

.player-info {
  flex: 1;
}

.player-name {
  font-size: 20px;
  font-weight: 700;
  color: #303133;
  margin-bottom: 4px;
}

.player-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.team-name {
  font-size: 14px;
  color: #909399;
}

/* 滑块行 */
.slider-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.slider-row .el-slider {
  flex: 1;
}

.value-input {
  width: 100px;
}

/* 滑块颜色 */
.ability-slider :deep(.el-slider__bar) {
  background: linear-gradient(90deg, #22c55e, #f59e0b, #ef4444);
}

.potential-slider :deep(.el-slider__bar) {
  background: linear-gradient(90deg, #8b5cf6, #a855f7);
}

.stability-slider :deep(.el-slider__bar) {
  background: linear-gradient(90deg, #3b82f6, #06b6d4);
}

.age-slider :deep(.el-slider__bar) {
  background: linear-gradient(90deg, #6b7280, #374151);
}

/* 属性提示 */
.attr-hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.attr-hint .potential-text {
  color: #8b5cf6;
  font-weight: 500;
}

.attr-hint .stability-text {
  color: #3b82f6;
  font-weight: 500;
}

.attr-hint .age-text {
  color: #6b7280;
  font-weight: 500;
}

/* 影响说明 */
.impact-alert {
  margin-top: 16px;
}
</style>
