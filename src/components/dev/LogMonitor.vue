<template>
  <Teleport to="body">
    <!-- 悬浮开关按钮（仅开发模式） -->
    <div
      v-if="!isVisible"
      class="log-toggle-btn"
      @click="isVisible = true"
      title="打开日志监控台 (Ctrl+Shift+L)"
    >
      📋
    </div>

    <div
      v-if="isVisible"
      class="log-monitor"
      :class="{ minimized: isMinimized }"
      :style="{
        width: isMinimized ? 'auto' : panelWidth + 'px',
        height: isMinimized ? 'auto' : panelHeight + 'px',
      }"
    >
      <!-- 标题栏 -->
      <div class="log-monitor-header">
        <span class="title">📋 日志监控台</span>
        <div class="actions">
          <button @click="toggleSearch" title="搜索">🔍</button>
          <button @click="isMinimized = !isMinimized" :title="isMinimized ? '展开' : '最小化'">
            {{ isMinimized ? '▢' : '−' }}
          </button>
          <button @click="close" title="关闭">×</button>
        </div>
      </div>

      <template v-if="!isMinimized">
        <!-- 搜索栏 -->
        <div v-if="showSearch" class="search-bar">
          <input v-model="searchQuery" placeholder="搜索日志..." @input="handleSearch" />
        </div>

        <!-- 筛选器 -->
        <div class="filters">
          <div class="level-filters">
            <label v-for="level in levels" :key="level">
              <input type="checkbox" v-model="selectedLevels" :value="level" />
              <span :class="'level-' + level.toLowerCase()">{{ level }}</span>
            </label>
          </div>
          <div class="module-filters">
            <select v-model="selectedModule">
              <option value="">全部模块</option>
              <option v-for="m in modules" :key="m" :value="m">{{ m }}</option>
            </select>
          </div>
        </div>

        <!-- 日志列表 -->
        <div class="log-list" ref="logListRef">
          <div
            v-for="(log, index) in filteredLogs"
            :key="index"
            class="log-entry"
            :class="'level-' + log.level.toLowerCase()"
            @click="toggleExpand(index)"
          >
            <span class="time">{{ formatTime(log.timestamp) }}</span>
            <span class="level">[{{ log.level }}]</span>
            <span class="module">[{{ log.module }}]</span>
            <span class="message">{{ log.message }}</span>
            <div v-if="expandedIndex === index && log.data" class="log-data">
              <pre>{{ JSON.stringify(log.data, null, 2) }}</pre>
            </div>
          </div>
          <div v-if="filteredLogs.length === 0" class="empty-state">暂无日志</div>
        </div>

        <!-- 底部工具栏 -->
        <div class="log-monitor-footer">
          <span class="stats">共 {{ logs.length }} 条 | 显示 {{ filteredLogs.length }} 条</span>
          <label class="pause-label">
            <input type="checkbox" v-model="isPaused" />
            暂停
          </label>
          <button @click="clearLogs" class="footer-btn">清空</button>
          <button @click="exportLogs" class="footer-btn">导出</button>
        </div>
      </template>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { loggerService, type LogEntry } from '@/utils/logger'

// 可见性状态
const isVisible = ref(false)
const isMinimized = ref(false)
const showSearch = ref(false)
const searchQuery = ref('')
const isPaused = ref(false)
const expandedIndex = ref<number | null>(null)

// 日志数据
const logs = ref<LogEntry[]>([])
const maxLogs = ref(500)

// 筛选条件
const levels = ['DEBUG', 'INFO', 'WARN', 'ERROR']
const selectedLevels = ref(['INFO', 'WARN', 'ERROR'])
const selectedModule = ref('')

// 面板尺寸
const panelWidth = ref(650)
const panelHeight = ref(420)

// 日志列表引用
const logListRef = ref<HTMLElement | null>(null)

// 收集所有出现过的模块
const modules = computed(() => {
  const moduleSet = new Set(logs.value.map((l) => l.module))
  return Array.from(moduleSet).sort()
})

// 过滤日志
const filteredLogs = computed(() => {
  return logs.value.filter((log) => {
    if (!selectedLevels.value.includes(log.level)) return false
    if (selectedModule.value && log.module !== selectedModule.value) return false
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      return (
        log.message.toLowerCase().includes(query) || log.module.toLowerCase().includes(query)
      )
    }
    return true
  })
})

// 订阅日志
let unsubscribe: (() => void) | null = null

onMounted(() => {
  unsubscribe = loggerService.subscribe((entry) => {
    if (!isPaused.value) {
      logs.value.push(entry)
      if (logs.value.length > maxLogs.value) {
        logs.value = logs.value.slice(-maxLogs.value)
      }
      // 自动滚动到底部
      nextTick(() => {
        if (logListRef.value) {
          logListRef.value.scrollTop = logListRef.value.scrollHeight
        }
      })
    }
  })

  // 快捷键 Ctrl+Shift+L 切换显示
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  unsubscribe?.()
  window.removeEventListener('keydown', handleKeydown)
})

function handleKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'l') {
    e.preventDefault()
    isVisible.value = !isVisible.value
  }
}

function formatTime(timestamp: string) {
  try {
    return new Date(timestamp).toLocaleTimeString()
  } catch {
    return timestamp
  }
}

function clearLogs() {
  logs.value = []
  expandedIndex.value = null
}

function exportLogs() {
  const content = logs.value
    .map((l) => {
      const dataStr = l.data ? ` | ${JSON.stringify(l.data)}` : ''
      return `[${l.timestamp}] [${l.level}] [${l.module}] ${l.message}${dataStr}`
    })
    .join('\n')

  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `logs-${new Date().toISOString().slice(0, 10)}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function close() {
  isVisible.value = false
}

function toggleSearch() {
  showSearch.value = !showSearch.value
  if (!showSearch.value) {
    searchQuery.value = ''
  }
}

function toggleExpand(index: number) {
  expandedIndex.value = expandedIndex.value === index ? null : index
}

function handleSearch() {
  // 搜索时重置展开状态
  expandedIndex.value = null
}

// 暴露方法供外部调用
defineExpose({
  show: () => {
    isVisible.value = true
  },
  hide: () => {
    isVisible.value = false
  },
  toggle: () => {
    isVisible.value = !isVisible.value
  },
  isVisible,
})
</script>

<style scoped>
.log-monitor {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: #1e1e1e;
  border: 1px solid #444;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 99999;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #ddd;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.log-monitor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
  user-select: none;
}

.title {
  font-weight: bold;
  font-size: 13px;
}

.actions {
  display: flex;
  gap: 4px;
}

.actions button {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 4px 8px;
  font-size: 14px;
  border-radius: 4px;
  transition: all 0.2s;
}

.actions button:hover {
  color: #fff;
  background: #3a3a3a;
}

.search-bar {
  padding: 8px 12px;
  border-bottom: 1px solid #333;
}

.search-bar input {
  width: 100%;
  padding: 6px 12px;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 4px;
  color: #ddd;
  font-size: 12px;
  outline: none;
}

.search-bar input:focus {
  border-color: #007acc;
}

.filters {
  padding: 8px 12px;
  border-bottom: 1px solid #333;
  display: flex;
  gap: 16px;
  align-items: center;
  flex-wrap: wrap;
}

.level-filters {
  display: flex;
  gap: 12px;
}

.level-filters label {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  font-size: 11px;
}

.level-filters input[type='checkbox'] {
  margin: 0;
  cursor: pointer;
}

.module-filters select {
  padding: 4px 8px;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 4px;
  color: #ddd;
  font-size: 11px;
  cursor: pointer;
  outline: none;
}

.level-trace {
  color: #888888;
}
.level-debug {
  color: #00bfff;
}
.level-info {
  color: #00ff00;
}
.level-warn {
  color: #ffa500;
}
.level-error {
  color: #ff4444;
}

.log-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  min-height: 150px;
}

.log-entry {
  padding: 4px 12px;
  cursor: pointer;
  transition: background 0.15s;
  border-left: 3px solid transparent;
}

.log-entry:hover {
  background: #2a2a2a;
}

.log-entry.level-error {
  border-left-color: #ff4444;
}

.log-entry.level-warn {
  border-left-color: #ffa500;
}

.log-entry .time {
  color: #666;
  margin-right: 8px;
}

.log-entry .level {
  font-weight: bold;
  margin-right: 4px;
}

.log-entry .module {
  color: #888;
  margin-right: 8px;
}

.log-entry .message {
  color: #ccc;
}

.log-data {
  margin-top: 8px;
  padding: 8px;
  background: #252525;
  border-radius: 4px;
  overflow-x: auto;
}

.log-data pre {
  margin: 0;
  color: #aaa;
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-all;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #666;
}

.log-monitor-footer {
  padding: 8px 12px;
  border-top: 1px solid #333;
  display: flex;
  align-items: center;
  gap: 12px;
  background: #2a2a2a;
}

.footer-btn {
  padding: 4px 12px;
  background: #3a3a3a;
  border: none;
  border-radius: 4px;
  color: #ddd;
  cursor: pointer;
  font-size: 11px;
  transition: background 0.2s;
}

.footer-btn:hover {
  background: #4a4a4a;
}

.stats {
  flex: 1;
  color: #888;
  font-size: 11px;
}

.pause-label {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #888;
  font-size: 11px;
  cursor: pointer;
}

.minimized {
  width: auto !important;
  height: auto !important;
}

.minimized .log-monitor-header {
  border-bottom: none;
  border-radius: 8px;
}

/* 悬浮开关按钮 */
.log-toggle-btn {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 40px;
  height: 40px;
  background: #1e1e1e;
  border: 1px solid #444;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  cursor: pointer;
  z-index: 99998;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  transition: all 0.2s;
  opacity: 0.6;
}

.log-toggle-btn:hover {
  opacity: 1;
  background: #2d2d2d;
  transform: scale(1.1);
}
</style>
