<template>
  <div class="home-view">
    <!-- 未加载存档提示 -->
    <div v-if="!hasSaveLoaded" class="no-save-overlay">
      <div class="no-save-content">
        <el-icon :size="64" class="no-save-icon"><FolderRemove /></el-icon>
        <h2>请先加载存档</h2>
        <p>在设置页面创建或加载一个存档以开始游戏</p>
        <el-button type="primary" size="large" @click="$router.push('/settings')">
          <el-icon><Setting /></el-icon>
          前往设置
        </el-button>
      </div>
    </div>

    <!-- 赛季状态栏 -->
    <div v-else class="season-status-bar">
      <div class="season-info">
        <div class="season-badge">
          <span class="season-label">赛季</span>
          <span class="season-value">S{{ currentSeason }}</span>
        </div>
        <div class="current-phase">
          <el-icon><Clock /></el-icon>
          <span>当前阶段：{{ currentPhaseDisplay }}</span>
        </div>
      </div>
      <div class="season-actions">
        <el-button type="primary" @click="handleAdvancePhase" :loading="isLoading">
          <el-icon><Right /></el-icon>
          推进阶段
        </el-button>
      </div>
    </div>

    <!-- 欢迎区域 -->
    <div class="welcome-section">
      <div class="welcome-content">
        <h1 class="welcome-title fade-in">电竞赛事管理系统</h1>
        <p class="welcome-subtitle fade-in-delay">Esports Tournament Management System v2.0</p>
      </div>
    </div>

    <!-- 赛区展示区域 -->
    <div class="regions-showcase">
      <div class="regions-grid">
        <div
          v-for="region in displayRegions"
          :key="region.code"
          class="region-card"
          :class="region.code.toLowerCase()"
          @click="goToRegion(region.code)"
        >
          <div class="region-decoration">
            <div class="decoration-circle"></div>
            <div class="decoration-circle"></div>
          </div>
          <div class="region-content">
            <div class="region-icon">
              <div class="icon-text">{{ region.code }}</div>
            </div>
            <h2 class="region-name">{{ region.name }}</h2>
            <p class="region-description">{{ region.fullName }}</p>
            <div class="region-stats">
              <div class="stat-item">
                <span class="stat-value">{{ region.teamCount }}</span>
                <span class="stat-label">战队</span>
              </div>
              <div class="stat-item">
                <span class="stat-value">{{ region.teamCount * 5 }}</span>
                <span class="stat-label">选手</span>
              </div>
            </div>
            <div class="region-badge-tag">
              <span class="badge-dot"></span>
              <span>{{ region.location }}</span>
            </div>
          </div>
          <div class="card-glow"></div>
        </div>
      </div>
    </div>

    <!-- 快捷操作区 -->
    <div class="quick-actions">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card class="action-card" shadow="hover" @click="$router.push('/tournaments')">
            <div class="action-icon tournaments">
              <el-icon :size="32"><Trophy /></el-icon>
            </div>
            <div class="action-info">
              <h3>赛事管理</h3>
              <p>查看和管理所有赛事</p>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="action-card" shadow="hover" @click="$router.push('/rankings')">
            <div class="action-icon rankings">
              <el-icon :size="32"><Medal /></el-icon>
            </div>
            <div class="action-info">
              <h3>积分排名</h3>
              <p>查看全球战队排名</p>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="action-card" shadow="hover" @click="$router.push('/transfer')">
            <div class="action-icon transfer">
              <el-icon :size="32"><Sort /></el-icon>
            </div>
            <div class="action-info">
              <h3>转会市场</h3>
              <p>选手交易与签约</p>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="action-card" shadow="hover" @click="$router.push('/draft')">
            <div class="action-icon draft">
              <el-icon :size="32"><Stamp /></el-icon>
            </div>
            <div class="action-info">
              <h3>选秀系统</h3>
              <p>新秀选拔与培养</p>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 装饰性背景元素 -->
    <div class="background-decoration">
      <div class="floating-circle circle-1"></div>
      <div class="floating-circle circle-2"></div>
      <div class="floating-circle circle-3"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { ElMessage } from 'element-plus'
import {
  Trophy, Medal, Sort, Stamp, Clock, Right,
  FolderRemove, Setting
} from '@element-plus/icons-vue'
import { useGameStore } from '@/stores/useGameStore'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'

const router = useRouter()
const gameStore = useGameStore()
const teamStore = useTeamStoreTauri()

// 从 store 获取响应式数据
const {
  hasSaveLoaded,
  currentSeason,
  currentPhaseDisplay,
  isLoading
} = storeToRefs(gameStore)

const { regions } = storeToRefs(teamStore)

// 赛区显示数据
const displayRegions = computed(() => {
  if (regions.value.length > 0) {
    return regions.value.map(r => ({
      code: r.code,
      name: getRegionName(r.code),
      fullName: getRegionFullName(r.code),
      location: getRegionLocation(r.code),
      teamCount: r.team_count
    }))
  }
  // 默认数据
  return [
    { code: 'LPL', name: '中国大陆职业联赛', fullName: 'League of Legends Pro League', location: '中国赛区', teamCount: 14 },
    { code: 'LCK', name: '韩国冠军联赛', fullName: 'League of Legends Champions Korea', location: '韩国赛区', teamCount: 14 },
    { code: 'LEC', name: '欧洲冠军联赛', fullName: 'League of Legends European Championship', location: '欧洲赛区', teamCount: 14 },
    { code: 'LCS', name: '北美冠军联赛', fullName: 'League of Legends Championship Series', location: '北美赛区', teamCount: 14 },
  ]
})

// 方法
const goToRegion = (regionCode: string) => {
  router.push(`/teams?region=${regionCode.toLowerCase()}`)
}

const handleAdvancePhase = async () => {
  try {
    await gameStore.advancePhase()
    ElMessage.success(`已推进到: ${currentPhaseDisplay.value}`)
  } catch (e) {
    ElMessage.error('推进阶段失败')
  }
}

const getRegionName = (code: string) => {
  const names: Record<string, string> = {
    LPL: '中国大陆职业联赛',
    LCK: '韩国冠军联赛',
    LEC: '欧洲冠军联赛',
    LCS: '北美冠军联赛'
  }
  return names[code] || code
}

const getRegionFullName = (code: string) => {
  const names: Record<string, string> = {
    LPL: 'League of Legends Pro League',
    LCK: 'League of Legends Champions Korea',
    LEC: 'League of Legends European Championship',
    LCS: 'League of Legends Championship Series'
  }
  return names[code] || ''
}

const getRegionLocation = (code: string) => {
  const locations: Record<string, string> = {
    LPL: '中国赛区',
    LCK: '韩国赛区',
    LEC: '欧洲赛区',
    LCS: '北美赛区'
  }
  return locations[code] || ''
}

// 初始化
onMounted(async () => {
  if (hasSaveLoaded.value) {
    await teamStore.loadRegions()
  }
})
</script>

<style scoped>
/* 未加载存档提示 */
.no-save-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.95);
  z-index: 100;
  border-radius: 12px;
}

.no-save-content {
  text-align: center;
  padding: 40px;
}

.no-save-icon {
  color: #909399;
  margin-bottom: 20px;
}

.no-save-content h2 {
  font-size: 24px;
  color: #303133;
  margin: 0 0 12px 0;
}

.no-save-content p {
  color: #909399;
  margin: 0 0 24px 0;
}

/* 赛季状态栏 */
.season-status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  margin-bottom: 20px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  position: relative;
  z-index: 10;
}

.season-info {
  display: flex;
  align-items: center;
  gap: 24px;
}

.season-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-radius: 20px;
  color: white;
}

.season-label {
  font-size: 12px;
  opacity: 0.9;
}

.season-value {
  font-size: 18px;
  font-weight: 700;
}

.current-phase {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  border-radius: 20px;
  color: white;
  font-size: 14px;
  font-weight: 500;
}

.current-phase .el-icon {
  font-size: 16px;
}

.season-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

/* 基础布局 */
.home-view {
  min-height: calc(100vh - 180px);
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  position: relative;
  overflow: hidden;
  padding: 40px 20px;
  margin: -24px;
  border-radius: 12px;
}

/* 欢迎区域 */
.welcome-section {
  text-align: center;
  padding: 40px 20px 60px;
  position: relative;
  z-index: 2;
}

.welcome-content {
  max-width: 800px;
  margin: 0 auto;
}

.welcome-title {
  font-size: 2.8rem;
  font-weight: 800;
  color: white;
  margin: 0 0 16px 0;
  text-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
  letter-spacing: 2px;
}

.welcome-subtitle {
  font-size: 1.1rem;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 300;
  letter-spacing: 3px;
  text-transform: uppercase;
}

/* 动画 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.fade-in {
  animation: fadeInUp 1s ease-out;
}

.fade-in-delay {
  animation: fadeInUp 1s ease-out 0.3s both;
}

/* 赛区展示 */
.regions-showcase {
  max-width: 1400px;
  margin: 0 auto;
  position: relative;
  z-index: 2;
}

.regions-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  padding: 0 20px;
}

.region-card {
  background: white;
  border-radius: 20px;
  padding: 32px 24px;
  position: relative;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
}

.region-card:hover {
  transform: translateY(-12px) scale(1.02);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.25);
}

.region-card:hover .card-glow {
  opacity: 1;
}

.card-glow {
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle, rgba(255, 255, 255, 0.2) 0%, transparent 70%);
  opacity: 0;
  transition: opacity 0.4s ease;
  pointer-events: none;
}

.region-content {
  position: relative;
  z-index: 2;
  text-align: center;
}

.region-icon {
  width: 100px;
  height: 100px;
  margin: 0 auto 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.4s ease;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.region-card:hover .region-icon {
  transform: scale(1.1) rotate(5deg);
}

.icon-text {
  font-size: 2rem;
  font-weight: 900;
  color: white;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  letter-spacing: 2px;
}

.region-name {
  font-size: 1.3rem;
  font-weight: 700;
  color: #2c3e50;
  margin: 0 0 8px 0;
}

.region-description {
  font-size: 0.8rem;
  color: #7f8c8d;
  margin: 0 0 16px 0;
}

.region-stats {
  display: flex;
  justify-content: center;
  gap: 24px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 800;
  color: #2c3e50;
}

.stat-label {
  font-size: 0.75rem;
  color: #95a5a6;
}

.region-badge-tag {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 20px;
  background: rgba(52, 152, 219, 0.1);
  color: #3498db;
  font-size: 0.85rem;
  font-weight: 600;
}

.badge-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #3498db;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.6; transform: scale(1.2); }
}

/* 各赛区颜色 */
.region-card.lpl .region-icon { background: linear-gradient(135deg, #ff4757 0%, #ff6b81 100%); }
.region-card.lpl .region-badge-tag { background: rgba(255, 71, 87, 0.1); color: #ff4757; }
.region-card.lpl .badge-dot { background: #ff4757; }

.region-card.lck .region-icon { background: linear-gradient(135deg, #3742fa 0%, #5352ed 100%); }
.region-card.lck .region-badge-tag { background: rgba(55, 66, 250, 0.1); color: #3742fa; }
.region-card.lck .badge-dot { background: #3742fa; }

.region-card.lec .region-icon { background: linear-gradient(135deg, #2ed573 0%, #7bed9f 100%); }
.region-card.lec .region-badge-tag { background: rgba(46, 213, 115, 0.1); color: #2ed573; }
.region-card.lec .badge-dot { background: #2ed573; }

.region-card.lcs .region-icon { background: linear-gradient(135deg, #ffa502 0%, #ff6348 100%); }
.region-card.lcs .region-badge-tag { background: rgba(255, 165, 2, 0.1); color: #ff6348; }
.region-card.lcs .badge-dot { background: #ff6348; }

/* 快捷操作 */
.quick-actions {
  max-width: 1400px;
  margin: 40px auto 0;
  padding: 0 20px;
  position: relative;
  z-index: 2;
}

.action-card {
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.95);
}

.action-card:hover {
  transform: translateY(-8px);
}

.action-card :deep(.el-card__body) {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
}

.action-icon {
  width: 60px;
  height: 60px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.action-icon.tournaments { background: linear-gradient(135deg, #667eea, #764ba2); }
.action-icon.rankings { background: linear-gradient(135deg, #f093fb, #f5576c); }
.action-icon.transfer { background: linear-gradient(135deg, #4facfe, #00f2fe); }
.action-icon.draft { background: linear-gradient(135deg, #43e97b, #38f9d7); }

.action-info h3 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
}

.action-info p {
  margin: 0;
  font-size: 12px;
  color: #7f8c8d;
}

/* 背景装饰 */
.background-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  overflow: hidden;
}

.floating-circle {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  animation: float 20s ease-in-out infinite;
}

.circle-1 { width: 300px; height: 300px; top: 10%; left: 5%; }
.circle-2 { width: 200px; height: 200px; top: 60%; right: 10%; animation-delay: 3s; }
.circle-3 { width: 150px; height: 150px; bottom: 10%; left: 50%; animation-delay: 6s; }

@keyframes float {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  50% { transform: translateY(-30px) rotate(180deg); }
}

/* 响应式 */
@media (max-width: 1400px) {
  .regions-grid { grid-template-columns: repeat(2, 1fr); }
}

@media (max-width: 768px) {
  .welcome-title { font-size: 2rem; }
  .regions-grid { grid-template-columns: 1fr; }
}
</style>
