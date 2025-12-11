<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

const route = useRoute();
const router = useRouter();
const expandedMenus = ref<string[]>(['选秀系统']);

interface MenuItem {
  name: string;
  path: string;
  icon: string;
  children?: MenuItem[];
}

const menuItems: MenuItem[] = [
  { name: '仪表盘', path: '/', icon: 'dashboard' },
  { name: '赛事管理', path: '/tournaments', icon: 'trophy' },
  { name: '战队管理', path: '/teams', icon: 'users' },
  { name: '选手中心', path: '/players', icon: 'gamepad' },
  {
    name: '选秀系统',
    path: '/draft',
    icon: 'clipboard',
    children: [
      { name: 'LPL 选秀', path: '/draft/lpl', icon: 'cn' },
      { name: 'LCK 选秀', path: '/draft/lck', icon: 'kr' },
      { name: 'LEC 选秀', path: '/draft/lec', icon: 'eu' },
      { name: 'LCS 选秀', path: '/draft/lcs', icon: 'us' },
    ]
  },
  { name: '转会市场', path: '/transfer', icon: 'exchange' },
  { name: '积分排名', path: '/rankings', icon: 'chart' },
  { name: '数据中心', path: '/data-center', icon: 'stats' },
  { name: 'IM年度评选', path: '/annual-top', icon: 'star' },
  { name: '荣誉殿堂', path: '/honors', icon: 'medal' },
  { name: '系统设置', path: '/settings', icon: 'settings' },
];

const isActive = (path: string) => {
  if (path === '/') {
    return route.path === '/';
  }
  return route.path.startsWith(path);
};

const navigate = (path: string) => {
  router.push(path);
};

const toggleMenu = (name: string) => {
  const index = expandedMenus.value.indexOf(name);
  if (index > -1) {
    expandedMenus.value.splice(index, 1);
  } else {
    expandedMenus.value.push(name);
  }
};

const isExpanded = (name: string) => expandedMenus.value.includes(name);

// SVG Icons
const icons: Record<string, string> = {
  dashboard: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z"/>',
  trophy: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"/>',
  users: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>',
  gamepad: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z"/>',
  clipboard: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"/>',
  exchange: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>',
  chart: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>',
  stats: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 8v8m-4-5v5m-4-2v2m-2 4h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/>',
  star: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"/>',
  medal: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3l7 7 7-7M5 3v4a7 7 0 007 7 7 7 0 007-7V3M12 14l-2 6h4l-2-6z"/>',
  settings: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>',
  chevron: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>',
};

const regionFlags: Record<string, string> = {
  cn: '🇨🇳',
  kr: '🇰🇷',
  eu: '🇪🇺',
  us: '🇺🇸',
};
</script>

<template>
  <aside class="w-72 glass-card min-h-[calc(100vh-72px)] flex flex-col border-r-0" style="border-radius: 0;">
    <!-- 导航菜单 -->
    <nav class="flex-1 p-4 overflow-y-auto">
      <div class="space-y-1.5">
        <template v-for="item in menuItems" :key="item.path">
          <!-- 无子菜单 -->
          <a
            v-if="!item.children"
            @click="navigate(item.path)"
            :class="[
              'group flex items-center gap-3 px-4 py-3 rounded-xl cursor-pointer transition-all duration-300',
              isActive(item.path)
                ? 'menu-active text-white'
                : 'text-gray-400 hover:text-white hover:bg-white/5'
            ]"
          >
            <svg
              class="w-5 h-5 transition-transform duration-300 group-hover:scale-110"
              :class="isActive(item.path) ? 'text-white' : 'text-gray-500 group-hover:text-cyan-400'"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              v-html="icons[item.icon]"
            />
            <span class="font-medium text-sm">{{ item.name }}</span>
            <div
              v-if="isActive(item.path)"
              class="ml-auto w-1.5 h-1.5 rounded-full bg-white animate-pulse"
            />
          </a>

          <!-- 有子菜单 -->
          <div v-else>
            <div
              @click="toggleMenu(item.name)"
              :class="[
                'group flex items-center gap-3 px-4 py-3 rounded-xl cursor-pointer transition-all duration-300',
                isActive(item.path)
                  ? 'bg-white/10 text-white'
                  : 'text-gray-400 hover:text-white hover:bg-white/5'
              ]"
            >
              <svg
                class="w-5 h-5 transition-transform duration-300 group-hover:scale-110"
                :class="isActive(item.path) ? 'text-cyan-400' : 'text-gray-500 group-hover:text-cyan-400'"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                v-html="icons[item.icon]"
              />
              <span class="font-medium text-sm flex-1">{{ item.name }}</span>
              <svg
                class="w-4 h-4 transition-transform duration-300"
                :class="isExpanded(item.name) ? 'rotate-180' : ''"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                v-html="icons.chevron"
              />
            </div>

            <transition
              enter-active-class="transition-all duration-300 ease-out"
              enter-from-class="opacity-0 -translate-y-2 max-h-0"
              enter-to-class="opacity-100 translate-y-0 max-h-48"
              leave-active-class="transition-all duration-200 ease-in"
              leave-from-class="opacity-100 translate-y-0 max-h-48"
              leave-to-class="opacity-0 -translate-y-2 max-h-0"
            >
              <div v-show="isExpanded(item.name)" class="mt-1 ml-4 pl-4 border-l border-white/10 space-y-1 overflow-hidden">
                <a
                  v-for="child in item.children"
                  :key="child.path"
                  @click="navigate(child.path)"
                  :class="[
                    'flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 text-sm',
                    isActive(child.path)
                      ? 'menu-active text-white'
                      : 'text-gray-500 hover:text-white hover:bg-white/5'
                  ]"
                >
                  <span class="text-base">{{ regionFlags[child.icon] }}</span>
                  <span>{{ child.name }}</span>
                </a>
              </div>
            </transition>
          </div>
        </template>
      </div>
    </nav>

    <!-- 底部信息 -->
    <div class="p-4 border-t border-white/5">
      <div class="glass-card rounded-xl p-4">
        <div class="flex items-center gap-3 mb-3">
          <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center">
            <span class="text-lg">🎮</span>
          </div>
          <div>
            <div class="text-sm font-semibold text-white">EsportManager 2</div>
            <div class="text-xs text-gray-500">v0.1.0 Beta</div>
          </div>
        </div>
        <div class="h-1 rounded-full bg-white/10 overflow-hidden">
          <div class="h-full w-1/3 rounded-full bg-gradient-to-r from-cyan-500 to-blue-600"></div>
        </div>
        <div class="text-xs text-gray-500 mt-2 text-center">开发进度 33%</div>
      </div>
    </div>
  </aside>
</template>
