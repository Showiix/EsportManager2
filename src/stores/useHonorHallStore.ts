import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { honorHallApi } from '@/api'
import type {
  SeasonHonorsResponse,
  SeasonInfo
} from '@/types'
import { createLogger } from '@/utils/logger'
import { handleError } from '@/utils/errors'

const logger = createLogger('HonorHallStore')

export const useHonorHallStore = defineStore('honorHall', () => {
  // ===================================================================
  // 状态
  // ===================================================================
  
  const selectedSeasonId = ref<string>('')
  const currentHonorData = ref<SeasonHonorsResponse | null>(null)
  const loading = ref(false)
  const availableSeasons = ref<SeasonInfo[]>([])
  const error = ref<string | null>(null)

  // ===================================================================
  // 计算属性
  // ===================================================================

  const currentSeason = computed(() => {
    return availableSeasons.value.find(s => s.id === selectedSeasonId.value)
  })

  const hasData = computed(() => {
    return currentHonorData.value !== null
  })

  // ===================================================================
  // 方法
  // ===================================================================

  /**
   * 获取可用赛季列表
   */
  const fetchAvailableSeasons = async () => {
    try {
      loading.value = true
      error.value = null

      const response = await honorHallApi.getAvailableSeasons()

      if (response.success && response.data) {
        availableSeasons.value = response.data.sort((a, b) => b.year - a.year)

        // 设置默认选中的赛季（最新的已完成赛季或活跃赛季）
        if (availableSeasons.value.length > 0 && !selectedSeasonId.value) {
          const latestSeason = availableSeasons.value.find(s => s.status === 'completed')
                            || availableSeasons.value[0]
          if (latestSeason) {
            selectedSeasonId.value = latestSeason.id
          }
        }
      } else {
        error.value = response.message || '获取赛季列表失败'
      }
    } catch (err: any) {
      error.value = err.message || '获取赛季列表失败'
      handleError(err, {
        component: 'HonorHallStore',
        userAction: '获取赛季列表',
        silent: true
      })
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取赛季荣誉数据
   */
  const fetchSeasonHonorData = async (seasonId?: string) => {
    const targetSeasonId = seasonId || selectedSeasonId.value
    if (!targetSeasonId) {
      logger.warn('未提供赛季ID')
      return
    }

    try {
      loading.value = true
      error.value = null

      const response = await honorHallApi.getSeasonHonors(targetSeasonId)

      if (response.success && response.data) {
        currentHonorData.value = response.data
      } else {
        error.value = response.message || '获取荣誉数据失败'
        currentHonorData.value = null
      }
    } catch (err: any) {
      error.value = err.message || '获取荣誉数据失败'
      currentHonorData.value = null
      handleError(err, {
        component: 'HonorHallStore',
        userAction: '获取荣誉数据',
        silent: true
      })
    } finally {
      loading.value = false
    }
  }

  /**
   * 切换赛季
   */
  const switchSeason = async (seasonId: string) => {
    selectedSeasonId.value = seasonId
    await fetchSeasonHonorData(seasonId)
  }

  /**
   * 刷新数据
   */
  const refreshData = async () => {
    await Promise.all([
      fetchAvailableSeasons(),
      fetchSeasonHonorData()
    ])
  }

  /**
   * 清除错误信息
   */
  const clearError = () => {
    error.value = null
  }

  // ===================================================================
  // 返回值
  // ===================================================================

  return {
    // 状态
    selectedSeasonId,
    currentHonorData,
    loading,
    availableSeasons,
    error,

    // 计算属性
    currentSeason,
    hasData,

    // 方法
    fetchAvailableSeasons,
    fetchSeasonHonorData,
    switchSeason,
    refreshData,
    clearError
  }
})
