<script setup lang="ts">
import { ref } from 'vue';

const currentSeason = ref('S1');
const currentPhase = ref('春季赛常规赛');
const isPlaying = ref(false);

const phases = [
  '春季赛常规赛', '春季赛季后赛', 'MSI季中赛', '马德里大师赛',
  '夏季赛常规赛', '夏季赛季后赛', 'Claude洲际赛', 'S世界赛',
  '上海大师赛', 'ICP洲际对抗赛', 'Super洲际邀请赛', '转会期', '选秀', '赛季结束'
];

const advancePhase = () => {
  // TODO: 实际推进阶段逻辑
  console.log('推进阶段');
};
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
          @click="advancePhase"
          class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2"
        >
          <span>▶</span>
          <span>推进阶段</span>
        </button>

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
