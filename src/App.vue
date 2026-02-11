<script setup lang="ts">
import { onMounted, ref } from 'vue'
import MainLayout from './components/layout/MainLayout.vue'
import LogMonitor from './components/dev/LogMonitor.vue'
import ErrorGuide from './components/common/ErrorGuide.vue'
import { loggerService, createLogger, LogLevel } from '@/utils/logger'

const logger = createLogger('App')
const isDev = import.meta.env.DEV

const cachedViews = ref(['Teams', 'Players', 'DataCenter', 'AnnualTop'])

// 初始化日志系统
onMounted(() => {
  // 开发环境配置
  if (import.meta.env.DEV) {
    loggerService.configure({
      minLevel: LogLevel.DEBUG,
      enableConsole: true,
      enableRemote: true,
      flushInterval: 3000,
    })
  } else {
    // 生产环境配置
    loggerService.configure({
      minLevel: LogLevel.INFO,
      enableConsole: false,
      enableRemote: true,
      flushInterval: 5000,
    })
  }

  logger.info('应用启动', {
    env: import.meta.env.MODE,
    version: import.meta.env.VITE_APP_VERSION || '1.0.0',
  })
})
</script>

<template>
  <MainLayout>
    <router-view v-slot="{ Component }">
      <keep-alive :include="cachedViews">
        <component :is="Component" />
      </keep-alive>
    </router-view>
  </MainLayout>

  <!-- 开发工具：日志监控台 (Ctrl+Shift+L 切换显示) -->
  <LogMonitor v-if="isDev" />

  <!-- 全局错误引导弹窗 -->
  <ErrorGuide />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
</style>
