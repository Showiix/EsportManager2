<template>
  <div class="dev-tools">
    <div class="page-header">
      <h1>开发工具</h1>
      <div class="status-badge" v-if="gameStatus">
        <el-tag type="info">S{{ gameStatus.current_season }}</el-tag>
        <el-tag>{{ formatPhase(gameStatus.current_phase) }}</el-tag>
        <el-tag :type="gameStatus.phase_completed ? 'success' : 'warning'">
          {{ gameStatus.phase_completed ? '已完成' : '进行中' }}
        </el-tag>
      </div>
    </div>

    <!-- 游戏状态概览 -->
    <el-card class="stats-card" v-if="gameStatus">
      <template #header>
        <div class="card-header">
          <el-icon><DataLine /></el-icon>
          <span>游戏状态概览</span>
          <el-button text size="small" @click="refreshStatus" :loading="refreshing">
            <el-icon><Refresh /></el-icon>
          </el-button>
        </div>
      </template>
      <div class="stats-grid">
        <div class="stat-item">
          <div class="stat-value">{{ gameStatus.team_count }}</div>
          <div class="stat-label">战队</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ gameStatus.player_count }}</div>
          <div class="stat-label">选手</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ gameStatus.tournament_count }}</div>
          <div class="stat-label">赛事</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ gameStatus.completed_matches }}/{{ gameStatus.total_matches }}</div>
          <div class="stat-label">比赛进度</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">{{ gameStatus.honor_count }}</div>
          <div class="stat-label">荣誉</div>
        </div>
      </div>
    </el-card>

    <!-- 功能分组 -->
    <div class="tools-grid">
      <!-- 荣誉系统 -->
      <el-card class="tool-card">
        <template #header>
          <div class="card-header">
            <el-icon><Trophy /></el-icon>
            <span>荣誉系统</span>
          </div>
        </template>
        <div class="button-group">
          <el-button @click="reassignHonors" :loading="loading.honors">
            重新颁发荣誉
          </el-button>
          <el-button @click="recalculateAnnualPoints" :loading="loading.points">
            重算年度积分
          </el-button>
          <el-button @click="recalculateYearlyScores" :loading="loading.scores">
            重算选手得分
          </el-button>
        </div>
      </el-card>

      <!-- 数据修复 -->
      <el-card class="tool-card">
        <template #header>
          <div class="card-header">
            <el-icon><DataAnalysis /></el-icon>
            <span>数据修复</span>
          </div>
        </template>
        <div class="button-group">
          <el-button @click="syncPlayerGamesPlayed" :loading="loading.sync">
            同步选手场次
          </el-button>
          <el-button @click="recalculateStandings" :loading="loading.standings">
            重算积分榜
          </el-button>
          <el-button @click="checkDataConsistency" :loading="loading.consistency">
            数据一致性检查
          </el-button>
          <el-button @click="recalculateMarketValues" :loading="loading.marketValues" type="primary">
            重算选手身价
          </el-button>
          <el-button @click="fixStarters" :loading="loading.starters" type="warning">
            修复首发阵容
          </el-button>
          <el-button @click="migrateLoyaltySatisfaction" :loading="loading.loyaltySatisfaction" type="success">
            重算忠诚度/满意度
          </el-button>
        </div>
      </el-card>

      <!-- 赛事管理 -->
      <el-card class="tool-card">
        <template #header>
          <div class="card-header">
            <el-icon><Calendar /></el-icon>
            <span>赛事管理</span>
          </div>
        </template>
        <div class="button-group">
          <el-button @click="initPhase" :loading="loading.initPhase">
            初始化当前阶段
          </el-button>
          <el-button @click="resetPhase" :loading="loading.resetPhase">
            重置阶段状态
          </el-button>
          <el-button @click="getPendingMatches" :loading="loading.matches">
            查看待模拟比赛
          </el-button>
        </div>
      </el-card>

      <!-- 快进系统 -->
      <el-card class="tool-card">
        <template #header>
          <div class="card-header">
            <el-icon><DArrowRight /></el-icon>
            <span>快进系统</span>
          </div>
        </template>
        <div class="button-group">
          <el-button @click="fastForwardTo('SUPER_INTERCONTINENTAL')" :loading="loading.fastForward" type="primary">
            快进到 Super
          </el-button>
          <el-button @click="fastForwardTo('ICP_INTERCONTINENTAL')" :loading="loading.fastForward">
            快进到 ICP
          </el-button>
          <el-button @click="fastForwardTo('WORLDS')" :loading="loading.fastForward">
            快进到 Worlds
          </el-button>
          <el-button @click="fastForwardTo('NEXT_PHASE')" :loading="loading.fastForward" type="success">
            快进到下一阶段
          </el-button>
        </div>
      </el-card>

      <!-- 财务系统 -->
      <el-card class="tool-card">
        <template #header>
          <div class="card-header">
            <el-icon><Money /></el-icon>
            <span>财务系统</span>
          </div>
        </template>
        <div class="button-group">
          <el-button @click="redistributePrizes" :loading="loading.prizes">
            重新发放奖金
          </el-button>
          <div class="input-group">
            <el-input-number v-model="grantAmount" :min="100000" :step="100000" size="small" />
            <el-button @click="grantFunds" :loading="loading.funds" size="small">
              发放资金
            </el-button>
          </div>
        </div>
      </el-card>

      <!-- 快速测试 -->
      <el-card class="tool-card danger-zone">
        <template #header>
          <div class="card-header">
            <el-icon><Warning /></el-icon>
            <span>危险操作</span>
          </div>
        </template>
        <div class="button-group">
          <el-popconfirm
            title="确定要重置存档吗？这将清除所有比赛数据！"
            @confirm="resetSave(false)"
          >
            <template #reference>
              <el-button type="danger" :loading="loading.reset">
                重置存档 (保留队伍)
              </el-button>
            </template>
          </el-popconfirm>
          <el-popconfirm
            title="确定要完全重置存档吗？这将清除所有数据！"
            @confirm="resetSave(true)"
          >
            <template #reference>
              <el-button type="danger" :loading="loading.reset">
                完全重置存档
              </el-button>
            </template>
          </el-popconfirm>
        </div>
      </el-card>
    </div>

    <!-- 执行日志 -->
    <el-card class="log-card">
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>执行日志</span>
          <el-button text size="small" @click="clearLogs">清空</el-button>
        </div>
      </template>
      <div class="log-container">
        <div
          v-for="(log, index) in logs"
          :key="index"
          :class="['log-item', log.type]"
        >
          <span class="log-time">{{ log.time }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
        <div v-if="logs.length === 0" class="log-empty">
          暂无日志
        </div>
      </div>
    </el-card>

    <!-- 一致性检查结果弹窗 -->
    <el-dialog v-model="consistencyDialogVisible" title="数据一致性检查结果" width="600px">
      <div v-if="consistencyResult">
        <div class="consistency-summary">
          <el-statistic title="总检查项" :value="consistencyResult.total_checks" />
          <el-statistic title="通过" :value="consistencyResult.passed" class="success" />
          <el-statistic title="失败" :value="consistencyResult.failed" class="danger" />
        </div>
        <el-divider />
        <div v-if="consistencyResult.issues.length > 0">
          <h4>发现的问题：</h4>
          <el-table :data="consistencyResult.issues" stripe>
            <el-table-column prop="category" label="类别" width="120" />
            <el-table-column prop="description" label="描述" />
            <el-table-column prop="severity" label="级别" width="80">
              <template #default="{ row }">
                <el-tag :type="row.severity === 'error' ? 'danger' : 'warning'" size="small">
                  {{ row.severity === 'error' ? '错误' : '警告' }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>
        <div v-else class="all-pass">
          <el-icon :size="48" color="#67C23A"><CircleCheckFilled /></el-icon>
          <p>所有检查项都通过了！</p>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import {
  Trophy, DataAnalysis, Calendar, Money, Warning, Document,
  DataLine, Refresh, CircleCheckFilled, DArrowRight
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { devApi, statsApi, timeApi } from '@/api/tauri'
import type { ConsistencyCheckResult, GameStatusSummary } from '@/api/tauri'

// 状态
const gameStatus = ref<GameStatusSummary | null>(null)
const refreshing = ref(false)
const grantAmount = ref(1000000)
const consistencyDialogVisible = ref(false)
const consistencyResult = ref<ConsistencyCheckResult | null>(null)

const loading = reactive({
  honors: false,
  points: false,
  scores: false,
  sync: false,
  standings: false,
  consistency: false,
  initPhase: false,
  resetPhase: false,
  matches: false,
  prizes: false,
  funds: false,
  reset: false,
  fastForward: false,
  marketValues: false,
  starters: false,
  loyaltySatisfaction: false,
})

interface LogEntry {
  time: string
  message: string
  type: 'success' | 'error' | 'info'
}

const logs = ref<LogEntry[]>([])

// 添加日志
function addLog(message: string, type: LogEntry['type'] = 'info') {
  const now = new Date()
  const time = now.toLocaleTimeString('zh-CN', { hour12: false })
  logs.value.unshift({ time, message, type })
  if (logs.value.length > 50) {
    logs.value.pop()
  }
}

function clearLogs() {
  logs.value = []
}

// 格式化阶段名称
function formatPhase(phase: string): string {
  const phaseMap: Record<string, string> = {
    'SpringRegular': '春季常规赛',
    'SpringPlayoffs': '春季季后赛',
    'Msi': 'MSI季中赛',
    'SummerRegular': '夏季常规赛',
    'SummerPlayoffs': '夏季季后赛',
    'Worlds': '世界赛',
    'Intercontinental': '洲际赛',
  }
  return phaseMap[phase] || phase
}

// 刷新状态
async function refreshStatus() {
  refreshing.value = true
  try {
    const result = await devApi.getGameStatus()
    if (result.success && result.data) {
      gameStatus.value = result.data
    }
  } catch (e) {
    console.error('获取状态失败:', e)
  } finally {
    refreshing.value = false
  }
}

// 荣誉系统
async function reassignHonors() {
  loading.honors = true
  try {
    const result = await devApi.reassignHonors()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.honors = false
    refreshStatus()
  }
}

async function recalculateAnnualPoints() {
  loading.points = true
  try {
    const result = await devApi.recalculateAnnualPoints()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.points = false
  }
}

async function recalculateYearlyScores() {
  loading.scores = true
  try {
    const seasonId = gameStatus.value?.current_season || 1
    const count = await statsApi.recalculateYearlyScores(seasonId)
    const message = `成功重新计算 ${count} 名选手的年度得分`
    addLog(message, 'success')
    ElMessage.success(message)
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.scores = false
  }
}

// 数据修复
async function syncPlayerGamesPlayed() {
  loading.sync = true
  try {
    const result = await devApi.syncPlayerGamesPlayed()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
      if (result.data && result.data.details.length > 0) {
        result.data.details.slice(0, 5).forEach(d => addLog(`  ${d}`, 'info'))
        if (result.data.details.length > 5) {
          addLog(`  ... 还有 ${result.data.details.length - 5} 条`, 'info')
        }
      }
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.sync = false
  }
}

async function recalculateStandings() {
  loading.standings = true
  try {
    const result = await devApi.recalculateStandings()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.standings = false
  }
}

async function checkDataConsistency() {
  loading.consistency = true
  try {
    const result = await devApi.checkDataConsistency()
    if (result.success && result.data) {
      consistencyResult.value = result.data
      consistencyDialogVisible.value = true
      addLog(result.message, result.data.failed > 0 ? 'error' : 'success')
    } else {
      addLog(result.error || '检查失败', 'error')
      ElMessage.error(result.error || '检查失败')
    }
  } catch (e: any) {
    addLog(e.message || '检查失败', 'error')
    ElMessage.error(e.message || '检查失败')
  } finally {
    loading.consistency = false
  }
}

async function recalculateMarketValues() {
  loading.marketValues = true
  try {
    const result = await devApi.recalculateMarketValues()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.marketValues = false
  }
}

async function fixStarters() {
  loading.starters = true
  try {
    const result = await devApi.fixStarters()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
      // 显示详细修复信息
      if (result.data && result.data.details.length > 0) {
        result.data.details.forEach(team => {
          addLog(`  ${team.team_name}: ${team.fixes.join(', ')}`, 'info')
        })
      }
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.starters = false
  }
}

async function migrateLoyaltySatisfaction() {
  loading.loyaltySatisfaction = true
  try {
    const result = await devApi.migrateLoyaltySatisfaction()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.loyaltySatisfaction = false
  }
}

// 赛事管理
async function initPhase() {
  loading.initPhase = true
  try {
    const message = await timeApi.initPhase()
    addLog(message, 'success')
    ElMessage.success(message)
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.initPhase = false
    refreshStatus()
  }
}

async function resetPhase() {
  loading.resetPhase = true
  try {
    const result = await devApi.resetPhase()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.resetPhase = false
    refreshStatus()
  }
}

async function fastForwardTo(target: string) {
  loading.fastForward = true
  try {
    const result = await timeApi.fastForwardTo(target)
    addLog(`快进到 ${target}: ${result.message}`, 'success')
    ElMessage.success(`快进成功: ${result.message}`)
  } catch (e: any) {
    addLog(e.message || '快进失败', 'error')
    ElMessage.error(e.message || '快进失败')
  } finally {
    loading.fastForward = false
    refreshStatus()
  }
}

async function getPendingMatches() {
  loading.matches = true
  try {
    const result = await devApi.simulateAllMatches()
    if (result.success) {
      addLog(result.message, 'info')
      ElMessage.info(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.matches = false
  }
}

// 财务系统
async function redistributePrizes() {
  loading.prizes = true
  try {
    const result = await devApi.redistributePrizes()
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.prizes = false
  }
}

async function grantFunds() {
  loading.funds = true
  try {
    const result = await devApi.grantFunds(grantAmount.value)
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.funds = false
  }
}

// 重置存档
async function resetSave(keepTeams: boolean) {
  loading.reset = true
  try {
    const result = await devApi.resetSave(keepTeams)
    if (result.success) {
      addLog(result.message, 'success')
      ElMessage.success(result.message)
    } else {
      addLog(result.error || '操作失败', 'error')
      ElMessage.error(result.error || '操作失败')
    }
  } catch (e: any) {
    addLog(e.message || '操作失败', 'error')
    ElMessage.error(e.message || '操作失败')
  } finally {
    loading.reset = false
    refreshStatus()
  }
}

onMounted(() => {
  refreshStatus()
  addLog('开发工具已加载', 'info')
})
</script>

<style scoped lang="scss">
.dev-tools {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;

  h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
  }

  .status-badge {
    display: flex;
    gap: 8px;
  }
}

.stats-card {
  margin-bottom: 20px;

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 20px;

    .stat-item {
      text-align: center;

      .stat-value {
        font-size: 24px;
        font-weight: 600;
        color: var(--el-color-primary);
      }

      .stat-label {
        font-size: 12px;
        color: #909399;
        margin-top: 4px;
      }
    }
  }
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}

.tool-card {
  .button-group {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;

    .el-button {
      flex: 1;
      min-width: 120px;
    }

    .input-group {
      display: flex;
      gap: 8px;
      width: 100%;

      .el-input-number {
        flex: 1;
      }
    }
  }

  &.danger-zone {
    :deep(.el-card__header) {
      background: linear-gradient(135deg, #fff5f5, #fff);
      color: #f56c6c;
    }
  }
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;

  .el-button {
    margin-left: auto;
  }
}

.log-card {
  .log-container {
    max-height: 300px;
    overflow-y: auto;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 12px;
    background: #fafafa;
    border-radius: 4px;
    padding: 12px;
  }

  .log-item {
    padding: 4px 0;
    border-bottom: 1px solid #eee;

    &:last-child {
      border-bottom: none;
    }

    .log-time {
      color: #909399;
      margin-right: 12px;
    }

    &.success .log-message {
      color: #67c23a;
    }

    &.error .log-message {
      color: #f56c6c;
    }

    &.info .log-message {
      color: #606266;
    }
  }

  .log-empty {
    text-align: center;
    color: #909399;
    padding: 20px;
  }
}

.consistency-summary {
  display: flex;
  justify-content: space-around;
  padding: 20px 0;

  :deep(.el-statistic) {
    text-align: center;

    &.success .el-statistic__number {
      color: #67c23a;
    }

    &.danger .el-statistic__number {
      color: #f56c6c;
    }
  }
}

.all-pass {
  text-align: center;
  padding: 40px;

  p {
    margin-top: 16px;
    color: #67c23a;
    font-size: 16px;
  }
}
</style>
