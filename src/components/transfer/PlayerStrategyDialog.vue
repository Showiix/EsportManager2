<template>
  <el-dialog
    v-model="visible"
    :title="`${playerName} - AI转会策略`"
    width="700px"
    :close-on-click-modal="false"
  >
    <div v-if="loading" class="loading-container">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>正在生成AI策略...</span>
    </div>

    <div v-else-if="strategy" class="strategy-content">
      <!-- 基本决策 -->
      <el-card class="decision-card" shadow="never">
        <div class="decision-header">
          <div class="decision-status" :class="strategy.wants_to_leave ? 'leave' : 'stay'">
            <el-icon v-if="strategy.wants_to_leave"><ArrowRight /></el-icon>
            <el-icon v-else><Check /></el-icon>
            <span>{{ strategy.wants_to_leave ? '想要离队' : '愿意留队' }}</span>
          </div>
          <div class="confidence">
            <span>置信度</span>
            <el-progress
              :percentage="strategy.decision_confidence"
              :stroke-width="8"
              :color="getConfidenceColor(strategy.decision_confidence)"
              style="width: 100px"
            />
          </div>
        </div>

        <!-- 离队原因 -->
        <div v-if="strategy.wants_to_leave && strategy.departure_reasons.length > 0" class="reasons">
          <div class="section-label">离队原因</div>
          <div class="reason-tags">
            <el-tag
              v-for="reason in strategy.departure_reasons"
              :key="reason"
              type="warning"
              effect="plain"
            >
              {{ reason }}
            </el-tag>
          </div>
        </div>

        <!-- AI分析 -->
        <div class="reasoning">
          <div class="section-label">AI分析</div>
          <div class="reasoning-text">{{ strategy.leave_reasoning }}</div>
        </div>

        <!-- 分析依据（可折叠） -->
        <el-collapse v-if="strategy.analysis_data || strategy.analysis_steps?.length > 0" class="analysis-collapse">
          <el-collapse-item title="查看分析过程" name="analysis">
            <!-- 分析数据快照 -->
            <div v-if="strategy.analysis_data" class="analysis-data">
              <div class="section-label">分析数据</div>
              <div class="data-grid">
                <div class="data-item">
                  <span class="data-label">选手</span>
                  <span class="data-value">{{ strategy.analysis_data.player_name }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">位置</span>
                  <span class="data-value">{{ strategy.analysis_data.position }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">年龄</span>
                  <span class="data-value">{{ strategy.analysis_data.age }}岁</span>
                </div>
                <div class="data-item">
                  <span class="data-label">能力</span>
                  <span class="data-value highlight">{{ strategy.analysis_data.ability }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">潜力</span>
                  <span class="data-value">{{ strategy.analysis_data.potential }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">满意度</span>
                  <span class="data-value" :class="getSatisfactionClass(strategy.analysis_data.satisfaction)">
                    {{ strategy.analysis_data.satisfaction }}
                  </span>
                </div>
                <div class="data-item">
                  <span class="data-label">忠诚度</span>
                  <span class="data-value" :class="getLoyaltyClass(strategy.analysis_data.loyalty)">
                    {{ strategy.analysis_data.loyalty }}
                  </span>
                </div>
                <div class="data-item">
                  <span class="data-label">忠诚类型</span>
                  <span class="data-value">{{ strategy.analysis_data.loyalty_type }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">首发</span>
                  <span class="data-value">{{ strategy.analysis_data.is_starter ? '是' : '否' }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">年薪</span>
                  <span class="data-value">{{ formatSalary(strategy.analysis_data.current_salary) }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">球队</span>
                  <span class="data-value">{{ strategy.analysis_data.team_name }}</span>
                </div>
                <div class="data-item">
                  <span class="data-label">离队阈值</span>
                  <span class="data-value warning">{{ strategy.analysis_data.departure_threshold }}</span>
                </div>
              </div>
            </div>

            <!-- 分析步骤 -->
            <div v-if="strategy.analysis_steps?.length > 0" class="analysis-steps">
              <div class="section-label">推理步骤</div>
              <el-timeline>
                <el-timeline-item
                  v-for="(step, index) in strategy.analysis_steps"
                  :key="index"
                  :type="getStepType(step.step_name)"
                  :hollow="true"
                >
                  <div class="step-card">
                    <div class="step-header">
                      <span class="step-name">{{ step.step_name }}</span>
                    </div>
                    <div class="step-content">
                      <div class="step-row">
                        <span class="step-label">数据：</span>
                        <span class="step-value">{{ step.data_used }}</span>
                      </div>
                      <div v-if="step.threshold" class="step-row">
                        <span class="step-label">规则：</span>
                        <span class="step-value threshold">{{ step.threshold }}</span>
                      </div>
                      <div class="step-row">
                        <span class="step-label">结论：</span>
                        <span class="step-value result">{{ step.result }}</span>
                      </div>
                      <div class="step-row">
                        <span class="step-label">影响：</span>
                        <span class="step-value impact">{{ step.impact }}</span>
                      </div>
                    </div>
                  </div>
                </el-timeline-item>
              </el-timeline>
            </div>
          </el-collapse-item>
        </el-collapse>
      </el-card>

      <!-- 偏好球队 -->
      <el-card v-if="strategy.wants_to_leave && strategy.preferred_teams.length > 0" class="teams-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>偏好球队 ({{ strategy.preferred_teams.length }}支)</span>
            <el-tag v-if="strategy.is_mock" size="small" type="info">规则生成</el-tag>
            <el-tag v-else size="small" type="success">AI生成</el-tag>
          </div>
        </template>

        <div class="teams-list">
          <div
            v-for="team in strategy.preferred_teams"
            :key="team.team_id"
            class="team-item"
          >
            <div class="team-rank">{{ team.priority }}</div>
            <div class="team-info">
              <div class="team-name">{{ team.team_name }}</div>
              <div class="team-reason">
                <el-tag size="small" :type="getReasonTagType(team.reason)">
                  {{ team.reason }}
                </el-tag>
                <span class="reason-detail">{{ team.reason_detail }}</span>
              </div>
            </div>
            <div class="team-score">
              <el-progress
                type="circle"
                :percentage="team.attractiveness_score"
                :width="40"
                :stroke-width="4"
              />
            </div>
          </div>
        </div>

        <div v-if="strategy.team_preference_reasoning" class="teams-reasoning">
          <div class="section-label">选队理由</div>
          <div class="reasoning-text">{{ strategy.team_preference_reasoning }}</div>
        </div>
      </el-card>

      <!-- 期望条件 -->
      <el-card v-if="strategy.wants_to_leave" class="expectations-card" shadow="never">
        <template #header>
          <span>期望条件</span>
        </template>

        <div class="expectations-grid">
          <div class="expectation-item">
            <div class="exp-label">期望薪资</div>
            <div class="exp-value salary">{{ formatSalary(strategy.expected_salary) }}/年</div>
          </div>
          <div class="expectation-item">
            <div class="exp-label">最低薪资</div>
            <div class="exp-value">{{ formatSalary(strategy.expected_min_salary) }}/年</div>
          </div>
          <div class="expectation-item">
            <div class="exp-label">期望年限</div>
            <div class="exp-value">{{ strategy.expected_years }}年</div>
          </div>
          <div class="expectation-item">
            <div class="exp-label">要求首发</div>
            <div class="exp-value">
              <el-tag :type="strategy.requires_starter ? 'danger' : 'info'" size="small">
                {{ strategy.requires_starter ? '是' : '否' }}
              </el-tag>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 生成信息 -->
      <div class="meta-info">
        <span>生成时间: {{ formatDate(strategy.generated_at) }}</span>
        <span v-if="strategy.is_mock" class="mock-tag">规则引擎生成</span>
        <span v-else class="ai-tag">LLM生成</span>
      </div>
    </div>

    <div v-else class="no-strategy">
      <el-empty description="暂无策略数据">
        <el-button type="primary" @click="generateStrategy" :loading="loading">
          生成AI策略
        </el-button>
      </el-empty>
    </div>

    <template #footer>
      <el-button @click="visible = false">关闭</el-button>
      <el-button type="primary" @click="generateStrategy" :loading="loading">
        {{ strategy ? '重新生成' : '生成策略' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Loading, ArrowRight, Check } from '@element-plus/icons-vue'
import { aiTransferApi, type PlayerTransferStrategy } from '@/api/tauri'
import { formatMoneyFromWan } from '@/utils'

const props = defineProps<{
  playerId: number | null
  playerName: string
}>()

const emit = defineEmits<{
  strategyGenerated: [playerId: number]
}>()

const visible = defineModel<boolean>('visible', { default: false })

const strategy = ref<PlayerTransferStrategy | null>(null)
const loading = ref(false)

// 监听 playerId 变化，重置并加载策略
watch(() => props.playerId, async (newId, oldId) => {
  // playerId 变化时先清空旧策略
  if (newId !== oldId) {
    strategy.value = null
  }
  if (newId && visible.value) {
    await loadStrategy()
  }
}, { immediate: true })

watch(visible, async (newVal) => {
  if (newVal && props.playerId) {
    await loadStrategy()
  } else if (!newVal) {
    // 关闭弹窗时清空策略，避免下次打开时闪现旧数据
    strategy.value = null
  }
})

const loadStrategy = async () => {
  if (!props.playerId) return

  loading.value = true
  try {
    const result = await aiTransferApi.getPlayerTransferStrategy(props.playerId)
    strategy.value = result
  } catch (e) {
    console.error('Failed to load strategy:', e)
  } finally {
    loading.value = false
  }
}

const generateStrategy = async () => {
  if (!props.playerId) return

  loading.value = true
  try {
    const result = await aiTransferApi.generatePlayerTransferStrategy(props.playerId)
    strategy.value = result
    emit('strategyGenerated', props.playerId)
    ElMessage.success('策略生成成功')
  } catch (e) {
    console.error('Failed to generate strategy:', e)
    ElMessage.error('策略生成失败')
  } finally {
    loading.value = false
  }
}

// 此处薪资输入是万，使用 formatMoneyFromWan
const formatSalary = (salary: number) => formatMoneyFromWan(salary)

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

const getConfidenceColor = (value: number) => {
  if (value >= 80) return '#67c23a'
  if (value >= 60) return '#e6a23c'
  return '#f56c6c'
}

const getReasonTagType = (reason: string) => {
  const types: Record<string, string> = {
    '争冠球队': 'danger',
    '首发机会': 'success',
    '高薪待遇': 'warning',
    '老东家': 'primary',
    '知名俱乐部': 'info',
  }
  return types[reason] || 'info'
}

// 分析步骤颜色
const getStepType = (stepName: string) => {
  if (stepName.includes('最终决策')) return 'success'
  if (stepName.includes('概率')) return 'warning'
  return 'primary'
}

// 满意度颜色类
const getSatisfactionClass = (value: number) => {
  if (value >= 70) return 'high'
  if (value >= 40) return 'mid'
  return 'low'
}

// 忠诚度颜色类
const getLoyaltyClass = (value: number) => {
  if (value >= 70) return 'high'
  if (value >= 40) return 'mid'
  return 'low'
}
</script>

<style scoped>
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 0;
  color: #909399;
  gap: 12px;
}

.loading-container .is-loading {
  font-size: 32px;
  color: #409eff;
}

.strategy-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.decision-card {
  background: linear-gradient(135deg, #f5f7fa 0%, #fff 100%);
}

.decision-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.decision-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 700;
  padding: 8px 16px;
  border-radius: 8px;
}

.decision-status.leave {
  background: #fef0f0;
  color: #f56c6c;
}

.decision-status.stay {
  background: #f0f9eb;
  color: #67c23a;
}

.confidence {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #606266;
  font-size: 13px;
}

.section-label {
  font-size: 12px;
  color: #909399;
  margin-bottom: 8px;
  font-weight: 500;
}

.reasons {
  margin-bottom: 16px;
}

.reason-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.reasoning-text {
  color: #606266;
  line-height: 1.6;
  font-size: 14px;
  background: #fafafa;
  padding: 12px;
  border-radius: 6px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.teams-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.team-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #fafafa;
  border-radius: 8px;
}

.team-rank {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(135deg, #409eff, #337ecc);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 14px;
  flex-shrink: 0;
}

.team-info {
  flex: 1;
}

.team-name {
  font-weight: 600;
  color: #303133;
  margin-bottom: 4px;
}

.team-reason {
  display: flex;
  align-items: center;
  gap: 8px;
}

.reason-detail {
  font-size: 12px;
  color: #909399;
}

.team-score {
  flex-shrink: 0;
}

.teams-reasoning {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #ebeef5;
}

.expectations-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.expectation-item {
  text-align: center;
}

.exp-label {
  font-size: 12px;
  color: #909399;
  margin-bottom: 8px;
}

.exp-value {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.exp-value.salary {
  color: #409eff;
}

.meta-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #909399;
  padding-top: 8px;
  border-top: 1px dashed #ebeef5;
}

.mock-tag {
  padding: 2px 8px;
  background: #f0f0f0;
  border-radius: 4px;
}

.ai-tag {
  padding: 2px 8px;
  background: #e1f3d8;
  color: #67c23a;
  border-radius: 4px;
}

.no-strategy {
  padding: 40px 0;
}

/* 分析过程样式 */
.analysis-collapse {
  margin-top: 16px;
  border: none;
}

.analysis-collapse :deep(.el-collapse-item__header) {
  font-size: 13px;
  color: #409eff;
  background: transparent;
  border: none;
  padding-left: 0;
}

.analysis-collapse :deep(.el-collapse-item__wrap) {
  border: none;
}

.analysis-collapse :deep(.el-collapse-item__content) {
  padding: 12px 0 0 0;
}

.analysis-data {
  margin-bottom: 16px;
}

.data-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  background: #f5f7fa;
  padding: 12px;
  border-radius: 8px;
}

.data-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.data-label {
  font-size: 11px;
  color: #909399;
}

.data-value {
  font-size: 13px;
  font-weight: 600;
  color: #303133;
}

.data-value.highlight {
  color: #409eff;
}

.data-value.warning {
  color: #e6a23c;
}

.data-value.high {
  color: #67c23a;
}

.data-value.mid {
  color: #e6a23c;
}

.data-value.low {
  color: #f56c6c;
}

.analysis-steps {
  margin-top: 12px;
}

.step-card {
  background: #fafafa;
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 8px;
}

.step-header {
  margin-bottom: 8px;
}

.step-name {
  font-weight: 600;
  font-size: 13px;
  color: #303133;
}

.step-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.step-row {
  display: flex;
  font-size: 12px;
  line-height: 1.5;
}

.step-label {
  color: #909399;
  flex-shrink: 0;
  width: 45px;
}

.step-value {
  color: #606266;
  flex: 1;
}

.step-value.threshold {
  color: #909399;
  font-style: italic;
}

.step-value.result {
  color: #409eff;
  font-weight: 500;
}

.step-value.impact {
  color: #e6a23c;
  font-weight: 500;
}
</style>
