<script setup lang="ts">
import { ref } from 'vue';

const currentSeason = ref('S1');
const currentPhase = ref('春季赛常规赛');
</script>

<template>
  <header class="h-16 bg-gray-800 border-b border-gray-700 px-6 flex items-center justify-between">
    <!-- Logo -->
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
        <span class="text-xl">🎮</span>
      </div>
      <div>
        <h1 class="text-lg font-bold text-white">电竞比赛模拟器 2</h1>
        <p class="text-xs text-gray-400">EsportManager 2</p>
      </div>
    </div>

    <!-- 当前状态 -->
    <div class="flex items-center gap-4">
      <!-- 赛季信息 -->
      <div class="flex items-center gap-2">
        <div class="px-3 py-1.5 bg-blue-600 rounded-lg">
          <span class="text-sm font-bold text-white">{{ currentSeason }}</span>
        </div>
        <div class="px-3 py-1.5 bg-gray-700 rounded-lg max-w-[200px]">
          <span class="text-sm text-gray-200 truncate">{{ currentPhase }}</span>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center gap-2">
        <button
          class="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          title="快速保存"
        >
          💾
        </button>

        <button
          class="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
          title="设置"
        >
          ⚙️
        </button>
      </div>
    </div>
  </header>
</template>
