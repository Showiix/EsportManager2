<template>
  <div class="team-edit-view">
    <!-- 返回按钮 -->
    <div class="back-link">
      <el-button text @click="goBack">
        <el-icon><ArrowLeft /></el-icon>
        返回战队详情
      </el-button>
    </div>

    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>编辑战队</h1>
        <p>修改战队基本信息和配置</p>
      </div>
    </div>

    <!-- 编辑表单 -->
    <el-row :gutter="20">
      <el-col :span="16">
        <el-card class="form-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon><Setting /></el-icon>
                基本信息
              </h2>
            </div>
          </template>

          <el-form
            ref="formRef"
            :model="form"
            :rules="rules"
            label-width="100px"
            label-position="top"
            class="edit-form"
          >
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="战队名称" prop="name">
                  <el-input
                    v-model="form.name"
                    placeholder="请输入战队名称"
                    maxlength="50"
                    show-word-limit
                  >
                    <template #prefix>
                      <el-icon><OfficeBuilding /></el-icon>
                    </template>
                  </el-input>
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="所属赛区" prop="region">
                  <el-select v-model="form.region" placeholder="请选择赛区" style="width: 100%">
                    <el-option
                      v-for="region in regionOptions"
                      :key="region.value"
                      :label="region.label"
                      :value="region.value"
                    >
                      <span class="region-option">
                        <span class="region-flag">{{ region.flag }}</span>
                        <span>{{ region.label }}</span>
                      </span>
                    </el-option>
                  </el-select>
                </el-form-item>
              </el-col>
            </el-row>

            <el-form-item label="战力值" prop="power">
              <div class="power-slider">
                <el-slider
                  v-model="form.power"
                  :min="0"
                  :max="100"
                  :step="0.1"
                  :marks="powerMarks"
                  show-stops
                />
                <div class="power-display">
                  <div class="power-number" :style="{ color: getPowerColor(form.power) }">
                    {{ form.power.toFixed(1) }}
                  </div>
                  <div class="power-level">{{ getPowerLevel(form.power) }}</div>
                </div>
              </div>
            </el-form-item>

            <el-divider />

            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="资金余额">
                  <el-input-number
                    v-model="form.balance"
                    :min="0"
                    :max="999999999"
                    :step="100000"
                    controls-position="right"
                    style="width: 100%"
                  />
                  <div class="form-hint">当前: {{ formatMoney(form.balance) }}</div>
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="年度积分">
                  <el-input-number
                    v-model="form.points"
                    :min="0"
                    :max="9999"
                    :step="10"
                    controls-position="right"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
            </el-row>

            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="胜场">
                  <el-input-number
                    v-model="form.wins"
                    :min="0"
                    :max="999"
                    controls-position="right"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="负场">
                  <el-input-number
                    v-model="form.losses"
                    :min="0"
                    :max="999"
                    controls-position="right"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
            </el-row>
          </el-form>

          <div class="form-actions">
            <el-button @click="resetForm">
              <el-icon><Refresh /></el-icon>
              重置
            </el-button>
            <el-button type="primary" @click="handleSubmit" :loading="submitting">
              <el-icon><Check /></el-icon>
              保存修改
            </el-button>
          </div>
        </el-card>
      </el-col>

      <!-- 右侧预览 -->
      <el-col :span="8">
        <el-card class="preview-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon><View /></el-icon>
                预览
              </h2>
            </div>
          </template>

          <div class="preview-content">
            <div class="preview-avatar" :class="form.region.toLowerCase()">
              {{ form.name.substring(0, 2) || '??' }}
            </div>

            <h3 class="preview-name">{{ form.name || '战队名称' }}</h3>

            <el-tag :type="getRegionType(form.region)" size="large" effect="dark">
              {{ form.region || '赛区' }}
            </el-tag>

            <div class="preview-stats">
              <div class="preview-stat">
                <span class="stat-value" :style="{ color: getPowerColor(form.power) }">
                  {{ form.power.toFixed(1) }}
                </span>
                <span class="stat-label">战力</span>
              </div>
              <div class="preview-stat">
                <span class="stat-value gold">{{ form.points }}</span>
                <span class="stat-label">积分</span>
              </div>
              <div class="preview-stat">
                <span class="stat-value green">{{ formatMoney(form.balance) }}</span>
                <span class="stat-label">资金</span>
              </div>
            </div>

            <div class="preview-record">
              <span class="record-wins">{{ form.wins }}胜</span>
              <span class="record-divider">-</span>
              <span class="record-losses">{{ form.losses }}负</span>
            </div>

            <div class="preview-winrate">
              胜率: {{ winRate }}%
            </div>
          </div>
        </el-card>

        <!-- 危险操作 -->
        <el-card class="danger-card">
          <template #header>
            <div class="card-header danger">
              <h2>
                <el-icon><Warning /></el-icon>
                危险操作
              </h2>
            </div>
          </template>

          <div class="danger-content">
            <p class="danger-text">以下操作不可撤销，请谨慎操作</p>

            <el-button type="danger" plain @click="handleDissolve">
              <el-icon><Delete /></el-icon>
              解散战队
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import {
  ArrowLeft,
  Setting,
  OfficeBuilding,
  Refresh,
  Check,
  View,
  Warning,
  Delete,
} from '@element-plus/icons-vue'

const route = useRoute()
const router = useRouter()
const teamId = route.params.id

// 表单引用
const formRef = ref<FormInstance>()
const submitting = ref(false)

// 表单数据
const form = ref({
  name: 'T1',
  region: 'LCK',
  power: 85.5,
  balance: 12000000,
  points: 180,
  wins: 15,
  losses: 3,
})

// 原始数据（用于重置）
const originalForm = ref({ ...form.value })

// 赛区选项
const regionOptions = [
  { value: 'LPL', label: 'LPL 中国赛区', flag: '🇨🇳' },
  { value: 'LCK', label: 'LCK 韩国赛区', flag: '🇰🇷' },
  { value: 'LEC', label: 'LEC 欧洲赛区', flag: '🇪🇺' },
  { value: 'LCS', label: 'LCS 北美赛区', flag: '🇺🇸' },
]

// 战力标记
const powerMarks = {
  0: '0',
  25: '25',
  50: '50',
  75: '75',
  100: '100',
}

// 表单验证规则
const rules: FormRules = {
  name: [
    { required: true, message: '请输入战队名称', trigger: 'blur' },
    { min: 2, max: 50, message: '战队名称长度在 2 到 50 个字符', trigger: 'blur' },
  ],
  region: [
    { required: true, message: '请选择赛区', trigger: 'change' },
  ],
  power: [
    { required: true, message: '请设置战力值', trigger: 'blur' },
  ],
}

// 计算属性
const winRate = computed(() => {
  const total = form.value.wins + form.value.losses
  if (total === 0) return 0
  return ((form.value.wins / total) * 100).toFixed(1)
})

// 方法
const goBack = () => {
  router.push(`/teams/${teamId}`)
}

const formatMoney = (value: number) => {
  if (value >= 10000000) {
    return `${(value / 10000000).toFixed(1)}千万`
  }
  return `${(value / 10000).toFixed(0)}万`
}

const getPowerColor = (power: number) => {
  if (power >= 85) return '#ef4444'
  if (power >= 75) return '#f59e0b'
  if (power >= 65) return '#3b82f6'
  return '#22c55e'
}

const getPowerLevel = (power: number) => {
  if (power >= 90) return '传奇'
  if (power >= 80) return '史诗'
  if (power >= 70) return '稀有'
  if (power >= 60) return '普通'
  return '一般'
}

const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const resetForm = () => {
  form.value = { ...originalForm.value }
  formRef.value?.clearValidate()
  ElMessage.info('表单已重置')
}

const handleSubmit = async () => {
  if (!formRef.value) return

  try {
    const valid = await formRef.value.validate()
    if (!valid) return

    submitting.value = true

    // 模拟 API 调用
    await new Promise(resolve => setTimeout(resolve, 1000))

    ElMessage.success('战队信息保存成功')
    router.push(`/teams/${teamId}`)
  } catch (error) {
    console.error('保存失败:', error)
    ElMessage.error('保存失败，请重试')
  } finally {
    submitting.value = false
  }
}

const handleDissolve = async () => {
  try {
    await ElMessageBox.confirm(
      '解散战队将删除所有战队数据，此操作不可恢复。确定要解散战队吗？',
      '危险操作',
      {
        confirmButtonText: '确定解散',
        cancelButtonText: '取消',
        type: 'warning',
        confirmButtonClass: 'el-button--danger',
      }
    )

    // 模拟 API 调用
    ElMessage.success('战队已解散')
    router.push('/teams')
  } catch {
    // 用户取消
  }
}

// 初始化
onMounted(() => {
  // 这里会从 API 加载战队数据
  // 目前使用模拟数据
  originalForm.value = { ...form.value }
})
</script>

<style scoped>
.team-edit-view {
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

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: var(--text-tertiary);
  margin: 0;
}

/* 表单卡片 */
.form-card {
  border-radius: 12px;
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

.card-header.danger h2 {
  color: #ef4444;
}

.edit-form {
  margin-top: 8px;
}

.region-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.region-flag {
  font-size: 16px;
}

/* 战力滑块 */
.power-slider {
  display: flex;
  align-items: center;
  gap: 24px;
}

.power-slider .el-slider {
  flex: 1;
}

.power-display {
  flex-shrink: 0;
  text-align: center;
}

.power-number {
  font-size: 28px;
  font-weight: 700;
  line-height: 1;
}

.power-level {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.form-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border-light);
}

/* 预览卡片 */
.preview-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.preview-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.preview-avatar {
  width: 100px;
  height: 100px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 28px;
}

.preview-avatar.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.preview-avatar.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.preview-avatar.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.preview-avatar.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.preview-name {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.preview-stats {
  display: flex;
  gap: 24px;
}

.preview-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.preview-stat .stat-value {
  font-size: 20px;
  font-weight: 700;
}

.preview-stat .stat-value.gold {
  color: #fbbf24;
}

.preview-stat .stat-value.green {
  color: #22c55e;
}

.preview-stat .stat-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.preview-record {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
}

.record-wins {
  color: #22c55e;
}

.record-divider {
  color: var(--text-tertiary);
}

.record-losses {
  color: #ef4444;
}

.preview-winrate {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 危险操作卡片 */
.danger-card {
  border-radius: 12px;
  border: 1px solid #fecaca;
}

.danger-content {
  text-align: center;
}

.danger-text {
  font-size: 14px;
  color: var(--text-tertiary);
  margin: 0 0 16px 0;
}

/* Element Plus 覆盖 */
:deep(.el-slider__marks-text) {
  font-size: 12px;
}

:deep(.el-input-number) {
  width: 100%;
}

:deep(.el-form-item__label) {
  font-weight: 500;
  color: var(--text-primary);
}
</style>
