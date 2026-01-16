import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { aiTransferApi, type AIStrategyInfo } from '@/api/tauri'

// 进度事件类型
export interface StrategyGenerationProgress {
  current: number
  total: number
  team_name: string
  status: 'generating' | 'success' | 'failed' | 'completed'
}

export const useAIStrategyStore = defineStore('aiStrategy', () => {
  // 生成状态
  const isGenerating = ref(false)
  const generationProgress = ref<StrategyGenerationProgress | null>(null)
  const strategies = ref<AIStrategyInfo[]>([])
  const error = ref<string | null>(null)

  // 事件监听器
  let unlistenProgress: UnlistenFn | null = null

  // 开始生成 AI 策略
  const generateStrategies = async () => {
    if (isGenerating.value) {
      console.warn('已有生成任务在进行中')
      return strategies.value
    }

    isGenerating.value = true
    generationProgress.value = null
    error.value = null

    try {
      // 设置进度事件监听
      unlistenProgress = await listen<StrategyGenerationProgress>('strategy-generation-progress', (event) => {
        generationProgress.value = event.payload
        if (event.payload.status === 'completed') {
          // 延迟清除进度，让用户看到完成状态
          setTimeout(() => {
            generationProgress.value = null
          }, 1500)
        }
      })

      // 调用 API 生成策略
      strategies.value = await aiTransferApi.generateAIStrategies()
      return strategies.value
    } catch (e) {
      console.error('Failed to generate strategies:', e)
      error.value = e instanceof Error ? e.message : '生成策略失败'
      throw e
    } finally {
      isGenerating.value = false
      // 清理监听器
      if (unlistenProgress) {
        unlistenProgress()
        unlistenProgress = null
      }
    }
  }

  // 清理（组件卸载时不需要调用，因为 store 是全局的）
  const cleanup = () => {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
    }
  }

  // 重置状态
  const reset = () => {
    isGenerating.value = false
    generationProgress.value = null
    strategies.value = []
    error.value = null
    cleanup()
  }

  return {
    // 状态
    isGenerating,
    generationProgress,
    strategies,
    error,
    // 方法
    generateStrategies,
    cleanup,
    reset,
  }
})
