/**
 * 性能监测 Store
 * 收集 Tauri IPC 调用耗时和路由导航耗时
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ========================================
// Types
// ========================================

export interface InvokeRecord {
  command: string
  duration: number       // ms
  success: boolean
  error?: string
  timestamp: number      // Date.now()
}

export interface NavigationRecord {
  from: string
  to: string
  routeName: string
  duration: number       // ms
  timestamp: number
}

export interface PerfLogEntry {
  id: number
  type: 'invoke' | 'navigation'
  name: string
  duration: number
  success: boolean
  error?: string
  timestamp: number
  details?: string
}

export interface PerfSummaryData {
  totalRequests: number
  avgDurationMs: number
  maxDurationMs: number
  p95DurationMs: number
  slowRequests: number
  errorCount: number
  errorRate: number
}

export interface CommandStats {
  command: string
  count: number
  avgMs: number
  totalMs: number
  maxMs: number
  errors: number
}

// ========================================
// Raw singleton for use outside Vue setup
// ========================================

interface PerfStoreRaw {
  isMonitoring: boolean
  recordInvoke: (record: InvokeRecord) => void
  recordNavigation: (record: NavigationRecord) => void
}

let _rawStore: PerfStoreRaw | null = null

export function usePerformanceStoreRaw(): PerfStoreRaw {
  if (!_rawStore) {
    return {
      isMonitoring: false,
      recordInvoke: () => {},
      recordNavigation: () => {},
    }
  }
  return _rawStore
}

// ========================================
// Store Definition
// ========================================

export const usePerformanceStore = defineStore('performance', () => {
  // State
  const isMonitoring = ref(false)
  const invokeRecords = ref<InvokeRecord[]>([])
  const navigationRecords = ref<NavigationRecord[]>([])
  const slowThresholdMs = ref(500)

  const MAX_INVOKE_RECORDS = 5000
  const MAX_NAV_RECORDS = 500

  // ========================================
  // Computed: Summary
  // ========================================

  const frontendSummary = computed<PerfSummaryData>(() => {
    const records = invokeRecords.value
    const total = records.length
    if (total === 0) {
      return {
        totalRequests: 0,
        avgDurationMs: 0,
        maxDurationMs: 0,
        p95DurationMs: 0,
        slowRequests: 0,
        errorCount: 0,
        errorRate: 0,
      }
    }
    const durations = records.map(r => r.duration).sort((a, b) => a - b)
    const sum = durations.reduce((a, b) => a + b, 0)
    const p95Index = Math.floor(durations.length * 0.95)
    const errorCount = records.filter(r => !r.success).length
    return {
      totalRequests: total,
      avgDurationMs: Math.round(sum / total),
      maxDurationMs: durations[durations.length - 1],
      p95DurationMs: durations[Math.min(p95Index, durations.length - 1)],
      slowRequests: records.filter(r => r.duration > slowThresholdMs.value).length,
      errorCount,
      errorRate: Math.round((errorCount / total) * 10000) / 100,  // 百分比保留2位
    }
  })

  // ========================================
  // Computed: Log entries (merged, newest first)
  // ========================================

  const logEntries = computed<PerfLogEntry[]>(() => {
    const invokeEntries: PerfLogEntry[] = invokeRecords.value.map((r, i) => ({
      id: i,
      type: 'invoke' as const,
      name: r.command,
      duration: r.duration,
      success: r.success,
      error: r.error,
      timestamp: r.timestamp,
    }))
    const navEntries: PerfLogEntry[] = navigationRecords.value.map((r, i) => ({
      id: 100000 + i,
      type: 'navigation' as const,
      name: `${r.from} → ${r.to}`,
      duration: r.duration,
      success: true,
      timestamp: r.timestamp,
      details: r.routeName,
    }))
    return [...invokeEntries, ...navEntries]
      .sort((a, b) => b.timestamp - a.timestamp)
  })

  // ========================================
  // Computed: Command distribution
  // ========================================

  const commandDistribution = computed<CommandStats[]>(() => {
    const map = new Map<string, { count: number; totalMs: number; maxMs: number; errors: number }>()
    for (const r of invokeRecords.value) {
      const existing = map.get(r.command) || { count: 0, totalMs: 0, maxMs: 0, errors: 0 }
      existing.count++
      existing.totalMs += r.duration
      if (r.duration > existing.maxMs) existing.maxMs = r.duration
      if (!r.success) existing.errors++
      map.set(r.command, existing)
    }
    return Array.from(map.entries())
      .map(([cmd, stats]) => ({
        command: cmd,
        count: stats.count,
        avgMs: Math.round(stats.totalMs / stats.count),
        totalMs: stats.totalMs,
        maxMs: stats.maxMs,
        errors: stats.errors,
      }))
      .sort((a, b) => b.avgMs - a.avgMs)
  })

  // ========================================
  // Computed: Timeline buckets (10s intervals)
  // ========================================

  const timelineBuckets = computed(() => {
    if (invokeRecords.value.length === 0) return []
    const records = invokeRecords.value
    const bucketSize = 10000 // 10 seconds
    const minTime = records[0]?.timestamp || 0
    const maxTime = records[records.length - 1]?.timestamp || 0
    const buckets: Array<{ time: string; count: number; avgMs: number; errors: number }> = []

    for (let t = minTime; t <= maxTime + bucketSize; t += bucketSize) {
      const inBucket = records.filter(r => r.timestamp >= t && r.timestamp < t + bucketSize)
      if (inBucket.length > 0) {
        const avgMs = inBucket.reduce((s, r) => s + r.duration, 0) / inBucket.length
        buckets.push({
          time: new Date(t).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
          count: inBucket.length,
          avgMs: Math.round(avgMs),
          errors: inBucket.filter(r => !r.success).length,
        })
      }
    }
    return buckets
  })

  // ========================================
  // Computed: Top 10 slowest
  // ========================================

  const slowestCommands = computed(() => {
    return [...invokeRecords.value]
      .sort((a, b) => b.duration - a.duration)
      .slice(0, 10)
  })

  // ========================================
  // Actions
  // ========================================

  function recordInvoke(record: InvokeRecord) {
    if (invokeRecords.value.length >= MAX_INVOKE_RECORDS) {
      invokeRecords.value.splice(0, MAX_INVOKE_RECORDS / 5)
    }
    invokeRecords.value.push(record)
  }

  function recordNavigation(record: NavigationRecord) {
    if (navigationRecords.value.length >= MAX_NAV_RECORDS) {
      navigationRecords.value.splice(0, 100)
    }
    navigationRecords.value.push(record)
  }

  async function startMonitoring() {
    isMonitoring.value = true
    try {
      await invoke('toggle_perf_monitoring', { enabled: true })
    } catch {
      // 后端命令尚未注册时忽略
    }
  }

  async function stopMonitoring() {
    isMonitoring.value = false
    try {
      await invoke('toggle_perf_monitoring', { enabled: false })
    } catch {}
  }

  async function clearAllData() {
    invokeRecords.value = []
    navigationRecords.value = []
    try {
      await invoke('clear_perf_records')
    } catch {}
  }

  function exportData(): string {
    return JSON.stringify({
      summary: frontendSummary.value,
      commandDistribution: commandDistribution.value,
      invokeRecords: invokeRecords.value,
      navigationRecords: navigationRecords.value,
      exportedAt: new Date().toISOString(),
    }, null, 2)
  }

  // Register raw singleton
  _rawStore = {
    get isMonitoring() { return isMonitoring.value },
    recordInvoke,
    recordNavigation,
  }

  return {
    // State
    isMonitoring,
    invokeRecords,
    navigationRecords,
    slowThresholdMs,
    // Computed
    frontendSummary,
    logEntries,
    commandDistribution,
    timelineBuckets,
    slowestCommands,
    // Actions
    recordInvoke,
    recordNavigation,
    startMonitoring,
    stopMonitoring,
    clearAllData,
    exportData,
  }
})
