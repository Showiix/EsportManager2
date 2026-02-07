import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { useGameStore } from './useGameStore'

export const useSeasonStore = defineStore('season', () => {
  const gameStore = useGameStore()

  // 当前活跃赛季（来自后端）
  const currentSeason = computed(() => gameStore.currentSeason || 1)

  // 各页面正在查看的赛季（默认当前赛季）
  const viewingSeason = ref(1)

  // 赛季选项列表
  const seasonOptions = computed(() => {
    const list = []
    for (let i = 1; i <= currentSeason.value; i++) {
      list.push({ label: `S${i}`, value: i })
    }
    return list
  })

  // 是否在查看历史赛季
  const isViewingHistory = computed(() => viewingSeason.value !== currentSeason.value)

  // 切换查看的赛季
  const switchSeason = (season: number) => {
    viewingSeason.value = season
  }

  // 重置为当前赛季
  const resetToCurrentSeason = () => {
    viewingSeason.value = currentSeason.value
  }

  // 当后端赛季更新时，自动跟随
  watch(currentSeason, (newSeason) => {
    viewingSeason.value = newSeason
  })

  return {
    currentSeason,
    viewingSeason,
    seasonOptions,
    isViewingHistory,
    switchSeason,
    resetToCurrentSeason,
  }
})
