<template>
  <div class="app-layout">
    <!-- 顶部导航 -->
    <el-header class="app-header">
      <div class="header-content">
        <div class="logo">
          <div class="logo-icon">
            <span>🎮</span>
          </div>
          <div class="logo-text">
            <h1>电竞比赛模拟器 2</h1>
            <span class="version">EsportManager v2.0</span>
          </div>
        </div>

        <div class="header-center">
          <div class="season-info">
            <el-tag type="primary" effect="dark" size="large">{{ currentSeason }}</el-tag>
            <el-tag type="info" size="large">{{ currentPhase }}</el-tag>
          </div>
        </div>

        <div class="header-actions">
          <el-button :icon="FolderOpened" @click="$router.push('/settings')">
            存档
          </el-button>
          <el-button :icon="Setting" circle @click="$router.push('/settings')" />
        </div>
      </div>
    </el-header>

    <!-- 主要内容区域 -->
    <el-container class="main-container">
      <!-- 侧边导航 -->
      <el-aside width="260px" class="app-sidebar">
        <el-menu
          :default-active="activeMenu"
          class="sidebar-menu"
          unique-opened
          @select="handleMenuSelect"
        >
          <el-menu-item index="/" :disabled="isMenuDisabled('/')">
            <el-icon><House /></el-icon>
            <span>首页仪表板</span>
          </el-menu-item>

          <el-menu-item index="/time" :disabled="isMenuDisabled('/time')">
            <el-icon><Clock /></el-icon>
            <span>时间控制</span>
          </el-menu-item>

          <el-sub-menu index="teams" :disabled="isMenuDisabled('/teams')">
            <template #title>
              <el-icon><UserFilled /></el-icon>
              <span>战队管理</span>
            </template>
            <el-menu-item index="/teams" :disabled="isMenuDisabled('/teams')">战队列表</el-menu-item>
            <el-menu-item index="/players" :disabled="isMenuDisabled('/players')">选手中心</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="competitions" :disabled="isMenuDisabled('/tournaments')">
            <template #title>
              <el-icon><Trophy /></el-icon>
              <span>赛事管理</span>
            </template>
            <el-menu-item index="/tournaments" :disabled="isMenuDisabled('/tournaments')">赛事总览</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="draft" :disabled="isMenuDisabled('/draft')">
            <template #title>
              <el-icon><Stamp /></el-icon>
              <span>选秀系统</span>
            </template>
            <el-menu-item index="/draft" :disabled="isMenuDisabled('/draft')">选秀总览</el-menu-item>
            <el-menu-item index="/draft/lpl" :disabled="isMenuDisabled('/draft/lpl')">🇨🇳 LPL 选秀</el-menu-item>
            <el-menu-item index="/draft/lck" :disabled="isMenuDisabled('/draft/lck')">🇰🇷 LCK 选秀</el-menu-item>
            <el-menu-item index="/draft/lec" :disabled="isMenuDisabled('/draft/lec')">🇪🇺 LEC 选秀</el-menu-item>
            <el-menu-item index="/draft/lcs" :disabled="isMenuDisabled('/draft/lcs')">🇺🇸 LCS 选秀</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="transfer-menu" :disabled="isMenuDisabled('/transfer')">
            <template #title>
              <el-icon><Sort /></el-icon>
              <span>转会市场</span>
            </template>
            <el-menu-item index="/transfer" :disabled="isMenuDisabled('/transfer')">📊 转会总览</el-menu-item>
            <el-menu-item index="/transfer/gm-config" :disabled="isMenuDisabled('/transfer/gm-config')">🤖 GM性格配置</el-menu-item>
            <el-menu-item index="/transfer/player-market" :disabled="isMenuDisabled('/transfer/player-market')">👤 选手合同中心</el-menu-item>
            <el-menu-item index="/transfer/market-listings" :disabled="isMenuDisabled('/transfer/market-listings')">📋 转会挂牌市场</el-menu-item>
            <el-menu-item index="/transfer/team-evaluation" :disabled="isMenuDisabled('/transfer/team-evaluation')">📋 战队评估中心</el-menu-item>
            <el-menu-item index="/transfer/player-evaluation" :disabled="isMenuDisabled('/transfer/player-evaluation')">👥 选手评估中心</el-menu-item>
          </el-sub-menu>

          <el-menu-item index="/rankings" :disabled="isMenuDisabled('/rankings')">
            <el-icon><Medal /></el-icon>
            <span>积分排名</span>
          </el-menu-item>

          <el-menu-item index="/finance" :disabled="isMenuDisabled('/finance')">
            <el-icon><Wallet /></el-icon>
            <span>财政中心</span>
          </el-menu-item>

          <el-menu-item index="/data-center" :disabled="isMenuDisabled('/data-center')">
            <el-icon><DataLine /></el-icon>
            <span>数据中心</span>
          </el-menu-item>

          <el-menu-item index="/annual-top" :disabled="isMenuDisabled('/annual-top')">
            <el-icon><Star /></el-icon>
            <span>IM年度评选</span>
          </el-menu-item>

          <el-menu-item index="/annual-awards" :disabled="isMenuDisabled('/annual-awards')">
            <el-icon><GoldMedal /></el-icon>
            <span>年度颁奖典礼</span>
          </el-menu-item>

          <el-menu-item index="/honors" :disabled="isMenuDisabled('/honors')">
            <el-icon><Trophy /></el-icon>
            <span>荣誉殿堂</span>
          </el-menu-item>

          <el-menu-item index="/performance">
            <el-icon><Odometer /></el-icon>
            <span>性能监测</span>
          </el-menu-item>

          <el-menu-item index="/settings">
            <el-icon><Setting /></el-icon>
            <span>系统设置</span>
          </el-menu-item>
        </el-menu>
      </el-aside>

      <!-- 主内容区 -->
      <el-main class="app-main">
        <div class="main-content">
          <!-- 面包屑导航 -->
          <el-breadcrumb class="breadcrumb" separator="/">
            <el-breadcrumb-item
              v-for="item in breadcrumbs"
              :key="item.path"
              :to="item.path"
            >
              {{ item.title }}
            </el-breadcrumb-item>
          </el-breadcrumb>

          <!-- 页面内容 -->
          <div class="page-content">
            <slot />
          </div>
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  House,
  UserFilled,
  Trophy,
  Stamp,
  Sort,
  Medal,
  Setting,
  FolderOpened,
  DataLine,
  Star,
  Clock,
  Wallet,
  GoldMedal,
} from '@element-plus/icons-vue'
import { useGameStore } from '@/stores/useGameStore'

const route = useRoute()
const router = useRouter()
const gameStore = useGameStore()

const currentSeason = ref('S1')
const currentPhase = ref('春季赛常规赛')

// 当前激活的菜单项
const activeMenu = computed(() => route.path)

// 检查菜单项是否应该禁用
const isMenuDisabled = (path: string) => {
  if (path === '/settings') return false
  return !gameStore.hasSaveLoaded
}

// 处理菜单选择
const handleMenuSelect = (index: string) => {
  if (isMenuDisabled(index)) {
    ElMessage.warning('请先在设置页面创建或加载存档')
    return
  }
  router.push(index)
}

// 面包屑导航
const breadcrumbs = computed(() => {
  const pathSegments = route.path.split('/').filter(Boolean)
  const crumbs = [{ path: '/', title: '首页' }]

  const menuMap: Record<string, string> = {
    time: '时间控制',
    teams: '战队管理',
    players: '选手中心',
    tournaments: '赛事管理',
    draft: '选秀系统',
    transfer: '转会市场',
    'market-listings': '转会挂牌市场',
    rankings: '积分排名',
    finance: '财政中心',
    'data-center': '数据中心',
    'annual-top': 'IM年度评选',
    'annual-awards': '年度颁奖典礼',
    honors: '荣誉殿堂',
    performance: '性能监测',
    settings: '系统设置',
    lpl: 'LPL 中国赛区',
    lck: 'LCK 韩国赛区',
    lec: 'LEC 欧洲赛区',
    lcs: 'LCS 北美赛区',
  }

  let currentPath = ''
  for (const segment of pathSegments) {
    currentPath += `/${segment}`
    if (menuMap[segment]) {
      crumbs.push({
        path: currentPath,
        title: menuMap[segment],
      })
    }
  }

  return crumbs
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 0;
  height: 64px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 24px;
  height: 100%;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  width: 40px;
  height: 40px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
}

.logo-text h1 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: white;
}

.logo-text .version {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
}

.header-center {
  display: flex;
  align-items: center;
}

.season-info {
  display: flex;
  gap: 8px;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.main-container {
  flex: 1;
  overflow: hidden;
}

.app-sidebar {
  background: linear-gradient(180deg, #f8f9fa 0%, #ffffff 100%);
  border-right: 1px solid #e4e7ed;
  overflow-y: auto;
}

.sidebar-menu {
  border-right: none;
  background: transparent;
}

.sidebar-menu :deep(.el-menu-item),
.sidebar-menu :deep(.el-sub-menu__title) {
  height: 50px;
  line-height: 50px;
  margin: 4px 8px;
  border-radius: 8px;
}

.sidebar-menu :deep(.el-menu-item:hover),
.sidebar-menu :deep(.el-sub-menu__title:hover) {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.app-main {
  padding: 0;
  background: #f5f7fa;
  overflow-y: auto;
}

.main-content {
  padding: 20px;
  min-height: 100%;
}

.breadcrumb {
  margin-bottom: 16px;
  padding: 12px 16px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.page-content {
  background: white;
  border-radius: 12px;
  min-height: calc(100vh - 180px);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  padding: 24px;
}
</style>
