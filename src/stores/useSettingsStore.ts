import { defineStore } from 'pinia'
import { ref } from 'vue'
import { handleError } from '@/utils/errors'

export interface SystemSettings {
  // 基础设置
  basic: {
    theme: 'light' | 'dark' | 'auto'
    language: 'zh-CN' | 'en-US'
    layout: 'default' | 'compact' | 'wide'
    autoSave: boolean
    autoSaveInterval: number // 分钟
  }

  // 游戏设置
  game: {
    enableRealtimeUpdates: boolean
    showDetailedStats: boolean
    enableNotifications: boolean
  }

  // 数据管理
  data: {
    enableBackup: boolean
    backupInterval: number // 小时
    maxBackupCount: number
    compressionLevel: 'none' | 'low' | 'medium' | 'high'
  }
}

const defaultSettings: SystemSettings = {
  basic: {
    theme: 'light',
    language: 'zh-CN',
    layout: 'default',
    autoSave: true,
    autoSaveInterval: 5
  },
  game: {
    enableRealtimeUpdates: true,
    showDetailedStats: true,
    enableNotifications: true
  },
  data: {
    enableBackup: true,
    backupInterval: 24,
    maxBackupCount: 30,
    compressionLevel: 'medium'
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<SystemSettings>({ ...defaultSettings })
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // 从本地存储加载设置
  const loadSettings = async () => {
    try {
      isLoading.value = true
      error.value = null

      const savedSettings = localStorage.getItem('esport-manager-settings')
      if (savedSettings) {
        const parsedSettings = JSON.parse(savedSettings)
        // 合并默认设置和保存的设置，确保新增的配置项有默认值
        settings.value = mergeSettings(defaultSettings, parsedSettings)
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载设置失败'
      handleError(err, {
        component: 'SettingsStore',
        userAction: '加载设置',
        silent: true
      })
    } finally {
      isLoading.value = false
    }
  }

  // 保存设置到本地存储
  const saveSettings = async () => {
    try {
      isLoading.value = true
      error.value = null

      localStorage.setItem('esport-manager-settings', JSON.stringify(settings.value))

      // 触发设置变更事件
      window.dispatchEvent(new CustomEvent('settings-changed', {
        detail: settings.value
      }))
    } catch (err) {
      error.value = err instanceof Error ? err.message : '保存设置失败'
      handleError(err, {
        component: 'SettingsStore',
        userAction: '保存设置'
      })
      throw err
    } finally {
      isLoading.value = false
    }
  }

  // 更新特定设置
  const updateSettings = async <K extends keyof SystemSettings>(
    category: K,
    updates: Partial<SystemSettings[K]>
  ) => {
    settings.value[category] = { ...settings.value[category], ...updates }
    await saveSettings()
  }

  // 重置设置到默认值
  const resetSettings = async () => {
    settings.value = { ...defaultSettings }
    await saveSettings()
  }

  // 重置特定分类的设置
  const resetCategory = async <K extends keyof SystemSettings>(category: K) => {
    settings.value[category] = { ...defaultSettings[category] }
    await saveSettings()
  }

  // 导出设置
  const exportSettings = () => {
    const dataStr = JSON.stringify(settings.value, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)

    const link = document.createElement('a')
    link.href = url
    link.download = `esport-manager-settings-${new Date().toISOString().split('T')[0]}.json`
    link.click()

    URL.revokeObjectURL(url)
  }

  // 导入设置
  const importSettings = async (file: File) => {
    return new Promise<void>((resolve, reject) => {
      const reader = new FileReader()

      reader.onload = async (e) => {
        try {
          const content = e.target?.result as string
          const importedSettings = JSON.parse(content)

          // 验证导入的设置格式
          if (!validateSettings(importedSettings)) {
            throw new Error('导入的设置文件格式不正确')
          }

          settings.value = mergeSettings(defaultSettings, importedSettings)
          await saveSettings()
          resolve()
        } catch (err) {
          reject(err instanceof Error ? err : new Error('导入设置失败'))
        }
      }

      reader.onerror = () => reject(new Error('读取文件失败'))
      reader.readAsText(file)
    })
  }

  // 获取当前设置值
  const getSetting = <K extends keyof SystemSettings, T extends keyof SystemSettings[K]>(
    category: K,
    key: T
  ): SystemSettings[K][T] => {
    return settings.value[category][key]
  }

  return {
    settings,
    isLoading,
    error,
    loadSettings,
    saveSettings,
    updateSettings,
    resetSettings,
    resetCategory,
    exportSettings,
    importSettings,
    getSetting
  }
})

// 辅助函数：深度合并设置对象
function mergeSettings(
  defaultSettings: SystemSettings,
  savedSettings: Partial<SystemSettings>
): SystemSettings {
  const result = { ...defaultSettings }

  for (const key in savedSettings) {
    const category = key as keyof SystemSettings
    if (savedSettings[category] && typeof savedSettings[category] === 'object') {
      (result as any)[category] = { ...defaultSettings[category], ...savedSettings[category] }
    }
  }

  return result
}

// 辅助函数：验证设置格式
function validateSettings(settings: any): settings is Partial<SystemSettings> {
  if (!settings || typeof settings !== 'object') {
    return false
  }

  const validCategories = ['basic', 'game', 'data']

  for (const key in settings) {
    if (!validCategories.includes(key)) {
      return false
    }

    if (typeof settings[key] !== 'object') {
      return false
    }
  }

  return true
}