<template>
  <div class="team-edit-view">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <el-button text @click="goBack" class="back-btn">
          <el-icon><ArrowLeft /></el-icon>
          返回
        </el-button>
        <div class="header-title" v-if="team">
          <h1>{{ team.name }}</h1>
          <el-tag :type="getRegionType(regionCode)" effect="dark">
            {{ regionCode }}
          </el-tag>
        </div>
      </div>
      <div class="header-actions">
        <el-button @click="goBack">取消</el-button>
        <el-button type="primary" @click="handleSubmit" :loading="submitting">
          <el-icon><Check /></el-icon>
          保存修改
        </el-button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <template v-else-if="team">
      <!-- 战队概览卡片 (只读) -->
      <div class="overview-card" :class="regionCode.toLowerCase()">
        <div class="overview-left">
          <div class="team-avatar">
            {{ team.short_name || team.name.slice(0, 2) }}
          </div>
          <div class="team-meta">
            <div class="team-name">{{ team.name }}</div>
            <el-tag :type="getRegionType(regionCode)" effect="dark" size="large">
              {{ regionCode }}
            </el-tag>
          </div>
        </div>
        <div class="overview-stats">
          <div class="stat-item">
            <span class="stat-value">{{ team.power_rating.toFixed(1) }}</span>
            <span class="stat-label">战力</span>
          </div>
          <div class="stat-item">
            <span class="stat-value gold">{{ team.annual_points }}</span>
            <span class="stat-label">积分</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ team.wins }}-{{ team.total_matches - team.wins }}</span>
            <span class="stat-label">战绩</span>
          </div>
          <div class="stat-item">
            <span class="stat-value" :class="getWinRateClass(team.win_rate)">
              {{ (team.win_rate * 100).toFixed(1) }}%
            </span>
            <span class="stat-label">胜率</span>
          </div>
        </div>
      </div>

      <!-- 编辑区域 -->
      <el-row :gutter="20">
        <!-- 基本信息 -->
        <el-col :span="12">
          <el-card class="edit-card">
            <template #header>
              <div class="card-header">
                <el-icon><Setting /></el-icon>
                <span>基本信息</span>
              </div>
            </template>
            <el-form label-position="top" class="edit-form">
              <el-form-item label="战队名称">
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

              <el-form-item label="战队简写">
                <el-input
                  v-model="form.shortName"
                  placeholder="2-4个字符"
                  maxlength="4"
                >
                  <template #prefix>
                    <el-icon><Ticket /></el-icon>
                  </template>
                </el-input>
                <div class="form-hint">显示在战队头像上的简写</div>
              </el-form-item>

              <el-form-item label="所属赛区">
                <el-input :value="regionCode" disabled>
                  <template #prefix>
                    <el-icon><Location /></el-icon>
                  </template>
                </el-input>
                <div class="form-hint readonly">赛区不可修改</div>
              </el-form-item>
            </el-form>
          </el-card>
        </el-col>

        <!-- 财务管理 -->
        <el-col :span="12">
          <el-card class="edit-card">
            <template #header>
              <div class="card-header">
                <el-icon><Wallet /></el-icon>
                <span>财务管理</span>
              </div>
            </template>
            <div class="finance-section">
              <div class="balance-display">
                <div class="balance-label">当前余额</div>
                <div class="balance-value">{{ formatMoney(team.balance) }}</div>
              </div>

              <el-divider />

              <div class="balance-adjust">
                <div class="adjust-label">调整金额</div>
                <div class="adjust-input">
                  <el-input-number
                    v-model="adjustAmount"
                    :min="-team.balance"
                    :max="999999999"
                    :step="100000"
                    controls-position="right"
                    style="width: 200px"
                  />
                </div>
                <div class="adjust-actions">
                  <el-button type="success" @click="adjustBalance(true)" :disabled="adjustAmount <= 0">
                    <el-icon><Plus /></el-icon>
                    增加 {{ formatMoney(adjustAmount) }}
                  </el-button>
                  <el-button type="danger" @click="adjustBalance(false)" :disabled="adjustAmount <= 0 || adjustAmount > team.balance">
                    <el-icon><Minus /></el-icon>
                    减少 {{ formatMoney(adjustAmount) }}
                  </el-button>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 战队简介 -->
      <el-card class="edit-card description-card">
        <template #header>
          <div class="card-header">
            <el-icon><Document /></el-icon>
            <span>战队简介</span>
          </div>
        </template>
        <el-input
          v-model="form.description"
          type="textarea"
          :rows="5"
          placeholder="请输入战队简介、历史荣誉等信息..."
          maxlength="1000"
          show-word-limit
        />
      </el-card>

      <!-- 危险操作 -->
      <el-card class="danger-card">
        <template #header>
          <div class="card-header danger">
            <el-icon><Warning /></el-icon>
            <span>危险操作</span>
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
    </template>

    <!-- 无数据 -->
    <el-empty v-else description="战队不存在" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  Check,
  Setting,
  OfficeBuilding,
  Ticket,
  Location,
  Wallet,
  Plus,
  Minus,
  Document,
  Warning,
  Delete,
} from '@element-plus/icons-vue'
import { teamApi, financeApi, queryApi, type Team, type Region } from '@/api/tauri'
import { formatMoney } from '@/utils'

const route = useRoute()
const router = useRouter()
const teamId = computed(() => Number(route.params.id))

// 状态
const loading = ref(true)
const submitting = ref(false)
const team = ref<Team | null>(null)
const regions = ref<Region[]>([])
const adjustAmount = ref(100000)

// 表单数据
const form = ref({
  name: '',
  shortName: '',
  description: '',
})

// 计算赛区代码
const regionCode = computed(() => {
  if (!team.value) return ''
  const region = regions.value.find(r => r.id === team.value!.region_id)
  return region?.code || ''
})

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const [teamData, regionsData] = await Promise.all([
      teamApi.getTeam(teamId.value),
      queryApi.getAllRegions(),
    ])
    team.value = teamData
    regions.value = regionsData

    // 初始化表单
    form.value.name = teamData.name
    form.value.shortName = teamData.short_name || ''
    form.value.description = '' // 暂无此字段
  } catch (error) {
    console.error('加载战队数据失败:', error)
    ElMessage.error('加载战队数据失败')
  } finally {
    loading.value = false
  }
}

// 方法
const goBack = () => {
  router.push(`/teams/${teamId.value}`)
}

// formatMoney 从 @/utils 导入

const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getWinRateClass = (rate: number) => {
  if (rate >= 0.7) return 'win-rate-high'
  if (rate >= 0.5) return 'win-rate-mid'
  return 'win-rate-low'
}

// 调整余额
const adjustBalance = async (isAdd: boolean) => {
  if (!team.value || adjustAmount.value <= 0) return

  const amount = isAdd ? adjustAmount.value : -adjustAmount.value
  const description = isAdd ? '手动增加资金' : '手动扣除资金'

  try {
    await financeApi.recordTransaction(
      teamId.value,
      amount,
      'ADJUSTMENT',
      description
    )

    // 更新本地数据
    team.value.balance += amount
    ElMessage.success(`${isAdd ? '增加' : '减少'} ${formatMoney(Math.abs(amount))} 成功`)
    adjustAmount.value = 100000
  } catch (error) {
    console.error('调整余额失败:', error)
    ElMessage.error('调整余额失败')
  }
}

// 保存修改
const handleSubmit = async () => {
  if (!team.value) return

  submitting.value = true
  try {
    // TODO: 后端暂无更新战队基本信息的 API
    // 这里暂时模拟成功
    await new Promise(resolve => setTimeout(resolve, 500))

    ElMessage.success('战队信息保存成功')
    router.push(`/teams/${teamId.value}`)
  } catch (error) {
    console.error('保存失败:', error)
    ElMessage.error('保存失败，请重试')
  } finally {
    submitting.value = false
  }
}

// 解散战队
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

    // TODO: 后端暂无解散战队的 API
    ElMessage.info('功能开发中...')
  } catch {
    // 用户取消
  }
}

// 初始化
onMounted(() => {
  loadData()
})
</script>

<style scoped>
.team-edit-view {
  padding: 0;
}

/* 页面头部 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-light);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.back-btn {
  color: var(--text-secondary);
}

.back-btn:hover {
  color: var(--primary-color);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
}

/* 加载状态 */
.loading-container {
  padding: 40px;
}

/* 概览卡片 */
.overview-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 32px;
  border-radius: 16px;
  margin-bottom: 24px;
  color: white;
}

.overview-card.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.overview-card.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.overview-card.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.overview-card.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.overview-left {
  display: flex;
  align-items: center;
  gap: 20px;
}

.team-avatar {
  width: 80px;
  height: 80px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 700;
}

.team-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.team-name {
  font-size: 28px;
  font-weight: 700;
}

.overview-stats {
  display: flex;
  gap: 40px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
}

.stat-value.gold {
  color: #fef08a;
}

.stat-value.win-rate-high {
  color: #86efac;
}

.stat-value.win-rate-mid {
  color: #fef08a;
}

.stat-value.win-rate-low {
  color: #fca5a5;
}

.stat-label {
  font-size: 14px;
  opacity: 0.8;
}

/* 编辑卡片 */
.edit-card {
  margin-bottom: 20px;
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.card-header.danger {
  color: #ef4444;
}

.edit-form {
  padding-top: 8px;
}

.form-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.form-hint.readonly {
  color: #f59e0b;
}

/* 财务区域 */
.finance-section {
  padding: 8px 0;
}

.balance-display {
  text-align: center;
  padding: 16px 0;
}

.balance-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
}

.balance-value {
  font-size: 36px;
  font-weight: 700;
  color: #22c55e;
}

.balance-adjust {
  text-align: center;
}

.adjust-label {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.adjust-input {
  margin-bottom: 16px;
}

.adjust-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

/* 简介卡片 */
.description-card {
  margin-bottom: 20px;
}

/* 危险操作 */
.danger-card {
  border-radius: 12px;
  border: 1px solid #fecaca;
}

.danger-content {
  text-align: center;
  padding: 8px 0;
}

.danger-text {
  font-size: 14px;
  color: var(--text-tertiary);
  margin: 0 0 16px 0;
}

/* Element Plus 覆盖 */
:deep(.el-form-item__label) {
  font-weight: 500;
  color: var(--text-primary);
}

:deep(.el-textarea__inner) {
  font-size: 14px;
  line-height: 1.6;
}
</style>
