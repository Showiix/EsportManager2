<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="showErrorDialog" class="error-overlay" @click.self="handleOverlayClick">
        <div class="error-dialog" :class="severityClass">
          <!-- 错误头部 -->
          <div class="error-header">
            <div class="error-icon">
              <el-icon v-if="severity === 'critical'" color="#ff0000" :size="28"><CircleCloseFilled /></el-icon>
              <el-icon v-else-if="severity === 'high'" color="#f56c6c" :size="28"><WarningFilled /></el-icon>
              <el-icon v-else-if="severity === 'medium'" color="#e6a23c" :size="28"><Warning /></el-icon>
              <el-icon v-else color="#909399" :size="28"><InfoFilled /></el-icon>
            </div>
            <div class="error-title-section">
              <h3 class="error-title">{{ errorTitle }}</h3>
              <span class="error-code" @click="copyErrorCode" title="点击复制">
                {{ errorCode }}
              </span>
            </div>
            <button class="close-btn" @click="handleClose" title="关闭">×</button>
          </div>

          <!-- 错误内容 -->
          <div class="error-content">
            <p class="error-message">{{ errorMessage }}</p>

            <!-- 错误描述 -->
            <div v-if="errorInfo?.description" class="error-description">
              <p>{{ errorInfo.description }}</p>
            </div>

            <!-- 建议操作 -->
            <div v-if="errorInfo?.suggestion" class="error-suggestion">
              <div class="suggestion-header">
                <el-icon class="suggestion-icon" color="#409eff"><Opportunity /></el-icon>
                <span>建议操作</span>
              </div>
              <p>{{ errorInfo.suggestion }}</p>
            </div>

            <!-- 详细信息（可展开） -->
            <div v-if="errorDetails" class="error-details-section">
              <button class="toggle-details" @click="showDetails = !showDetails">
                {{ showDetails ? '收起详情' : '查看详情' }}
                <span class="arrow" :class="{ expanded: showDetails }">▼</span>
              </button>
              <Transition name="slide">
                <div v-if="showDetails" class="error-details">
                  <pre>{{ errorDetails }}</pre>
                </div>
              </Transition>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="error-actions">
            <button v-if="errorInfo?.docUrl" class="action-btn help-btn" @click="openHelp">
              <el-icon><Reading /></el-icon> 查看帮助
            </button>
            <button class="action-btn copy-btn" @click="copyAllInfo">
              <el-icon><CopyDocument /></el-icon> 复制信息
            </button>
            <button
              v-if="canRetry"
              class="action-btn retry-btn"
              :disabled="isRetrying"
              @click="handleRetry"
            >
              <span v-if="isRetrying" class="loading-spinner"></span>
              {{ isRetrying ? '重试中...' : '' }}
              <el-icon v-if="!isRetrying"><RefreshRight /></el-icon>
              {{ isRetrying ? '' : ' 重试' }}
            </button>
            <button class="action-btn confirm-btn" @click="handleClose">
              我知道了
            </button>
          </div>

          <!-- 底部提示 -->
          <div class="error-footer">
            <span class="severity-badge" :class="severity">
              {{ severityText }}
            </span>
            <span class="footer-hint">
              如问题持续，请联系技术支持并提供错误代码
            </span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  CircleCloseFilled,
  WarningFilled,
  Warning,
  InfoFilled,
  Opportunity,
  Reading,
  CopyDocument,
  RefreshRight,
} from '@element-plus/icons-vue'
import {
  globalError,
  showErrorDialog,
  clearError,
  retryLastAction,
  getCurrentErrorInfo,
} from '@/utils/errors'
import type { ErrorSeverity } from '@/utils/errors'
import { createLogger } from '@/utils/logger'

const logger = createLogger('ErrorGuide')

// 状态
const showDetails = ref(false)
const isRetrying = ref(false)
const copySuccess = ref(false)

// 计算属性
const errorCode = computed(() => globalError.value?.code || 'UNKNOWN')
const errorMessage = computed(() => globalError.value?.message || '发生未知错误')
const errorDetails = computed(() => globalError.value?.details)
const canRetry = computed(() => globalError.value?.canRetry && globalError.value?.retryFn)
const errorInfo = computed(() => getCurrentErrorInfo())

const errorTitle = computed(() => {
  return errorInfo.value?.title || '操作失败'
})

const severity = computed<ErrorSeverity>(() => {
  return errorInfo.value?.severity || 'medium'
})

const severityClass = computed(() => `severity-${severity.value}`)

const severityText = computed(() => {
  const map: Record<ErrorSeverity, string> = {
    low: '轻微',
    medium: '中等',
    high: '严重',
    critical: '严重',
  }
  return map[severity.value]
})

// 监听对话框显示状态
watch(showErrorDialog, (newVal) => {
  if (newVal) {
    showDetails.value = false
    isRetrying.value = false
  }
})

// 方法
function handleClose() {
  clearError()
}

function handleOverlayClick() {
  // 仅在非严重错误时允许点击遮罩关闭
  if (severity.value !== 'critical') {
    handleClose()
  }
}

async function handleRetry() {
  if (isRetrying.value) return

  isRetrying.value = true
  try {
    await retryLastAction()
  } finally {
    isRetrying.value = false
  }
}

function openHelp() {
  if (errorInfo.value?.docUrl) {
    // 在游戏内打开帮助页面或跳转到文档
    logger.debug('打开帮助文档', { url: errorInfo.value.docUrl })
    // TODO: 实现帮助页面跳转
  }
}

async function copyErrorCode() {
  try {
    await navigator.clipboard.writeText(errorCode.value)
    showCopyFeedback()
  } catch (e) {
    logger.error('复制错误码失败', { error: e })
  }
}

async function copyAllInfo() {
  const info = [
    `错误代码: ${errorCode.value}`,
    `错误标题: ${errorTitle.value}`,
    `错误信息: ${errorMessage.value}`,
    errorInfo.value?.description ? `描述: ${errorInfo.value.description}` : '',
    errorInfo.value?.suggestion ? `建议: ${errorInfo.value.suggestion}` : '',
    errorDetails.value ? `详情:\n${errorDetails.value}` : '',
    `时间: ${new Date().toISOString()}`,
  ]
    .filter(Boolean)
    .join('\n')

  try {
    await navigator.clipboard.writeText(info)
    showCopyFeedback()
  } catch (e) {
    logger.error('复制错误信息失败', { error: e })
  }
}

function showCopyFeedback() {
  copySuccess.value = true
  setTimeout(() => {
    copySuccess.value = false
  }, 2000)
}
</script>

<style scoped>
.error-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 99998;
  backdrop-filter: blur(2px);
}

.error-dialog {
  background: #1e1e1e;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  border: 1px solid #333;
}

/* 严重程度边框 */
.error-dialog.severity-low {
  border-left: 4px solid #909399;
}

.error-dialog.severity-medium {
  border-left: 4px solid #e6a23c;
}

.error-dialog.severity-high {
  border-left: 4px solid #f56c6c;
}

.error-dialog.severity-critical {
  border-left: 4px solid #ff0000;
  animation: pulse-border 2s infinite;
}

@keyframes pulse-border {
  0%,
  100% {
    box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(255, 0, 0, 0);
  }
}

.error-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: #2a2a2a;
  border-bottom: 1px solid #333;
}

.error-icon {
  display: flex;
  align-items: center;
  line-height: 1;
}

.error-title-section {
  flex: 1;
  min-width: 0;
}

.error-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #fff;
}

.error-code {
  display: inline-block;
  margin-top: 4px;
  padding: 2px 8px;
  background: #3a3a3a;
  border-radius: 4px;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: #888;
  cursor: pointer;
  transition: all 0.2s;
}

.error-code:hover {
  background: #4a4a4a;
  color: #aaa;
}

.close-btn {
  background: none;
  border: none;
  color: #666;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
  line-height: 1;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  color: #fff;
  background: #3a3a3a;
}

.error-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.error-message {
  margin: 0 0 16px;
  font-size: 14px;
  color: #ddd;
  line-height: 1.6;
}

.error-description {
  margin-bottom: 16px;
  padding: 12px;
  background: #252525;
  border-radius: 8px;
}

.error-description p {
  margin: 0;
  font-size: 13px;
  color: #aaa;
  line-height: 1.5;
}

.error-suggestion {
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(64, 158, 255, 0.1);
  border: 1px solid rgba(64, 158, 255, 0.2);
  border-radius: 8px;
}

.suggestion-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 600;
  color: #409eff;
}

.suggestion-icon {
  font-size: 16px;
}

.error-suggestion p {
  margin: 0;
  font-size: 13px;
  color: #ccc;
  line-height: 1.5;
}

.error-details-section {
  margin-top: 12px;
}

.toggle-details {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: #666;
  font-size: 12px;
  cursor: pointer;
  padding: 4px 0;
  transition: color 0.2s;
}

.toggle-details:hover {
  color: #aaa;
}

.toggle-details .arrow {
  font-size: 10px;
  transition: transform 0.2s;
}

.toggle-details .arrow.expanded {
  transform: rotate(180deg);
}

.error-details {
  margin-top: 8px;
  padding: 12px;
  background: #1a1a1a;
  border-radius: 6px;
  overflow-x: auto;
}

.error-details pre {
  margin: 0;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: #888;
  white-space: pre-wrap;
  word-break: break-all;
}

.error-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid #333;
  background: #252525;
}

.action-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
}

.help-btn {
  background: #3a3a3a;
  color: #ddd;
}

.help-btn:hover {
  background: #4a4a4a;
}

.copy-btn {
  background: #3a3a3a;
  color: #ddd;
}

.copy-btn:hover {
  background: #4a4a4a;
}

.retry-btn {
  background: #409eff;
  color: #fff;
}

.retry-btn:hover:not(:disabled) {
  background: #66b1ff;
}

.retry-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.confirm-btn {
  background: #67c23a;
  color: #fff;
  margin-left: auto;
}

.confirm-btn:hover {
  background: #85ce61;
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-footer {
  padding: 12px 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  background: #1a1a1a;
  border-top: 1px solid #2a2a2a;
}

.severity-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.severity-badge.low {
  background: rgba(144, 147, 153, 0.2);
  color: #909399;
}

.severity-badge.medium {
  background: rgba(230, 162, 60, 0.2);
  color: #e6a23c;
}

.severity-badge.high {
  background: rgba(245, 108, 108, 0.2);
  color: #f56c6c;
}

.severity-badge.critical {
  background: rgba(255, 0, 0, 0.2);
  color: #ff4444;
}

.footer-hint {
  font-size: 11px;
  color: #666;
}

/* 动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-active .error-dialog,
.fade-leave-active .error-dialog {
  transition: transform 0.2s ease;
}

.fade-enter-from .error-dialog,
.fade-leave-to .error-dialog {
  transform: scale(0.95);
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.2s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  max-height: 0;
}

.slide-enter-to,
.slide-leave-from {
  max-height: 200px;
}
</style>
