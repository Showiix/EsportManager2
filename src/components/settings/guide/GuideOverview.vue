<template>
  <div class="guide-content">
    <div class="overview-intro">
      <p class="guide-desc">
        <strong>电竞经理模拟器 2</strong> 是一款基于 <el-tag size="small">Tauri</el-tag> + <el-tag size="small">Vue3</el-tag> + <el-tag size="small">Rust</el-tag> 的桌面端电竞赛事模拟游戏。
      </p>
      <p class="guide-desc">
        玩家管理 <strong>4 个赛区</strong>（LPL/LCK/LEC/LCS）共 <strong>56 支战队</strong>，体验从春季赛到年度 Super 洲际邀请赛的完整赛事周期。
      </p>
    </div>

    <div class="section">
      <h4><el-icon><Collection /></el-icon> 核心系统</h4>
      <div class="system-grid">
        <div class="system-item">
          <div class="system-icon time"><el-icon><Clock /></el-icon></div>
          <div class="system-info">
            <div class="system-name">时间推进系统</div>
            <div class="system-desc">控制赛季流转，14个阶段循环</div>
          </div>
        </div>
        <div class="system-item">
          <div class="system-icon match"><el-icon><VideoPlay /></el-icon></div>
          <div class="system-info">
            <div class="system-name">比赛模拟引擎</div>
            <div class="system-desc">三层正态分布模型计算胜负</div>
          </div>
        </div>
        <div class="system-item">
          <div class="system-icon honor"><el-icon><Medal /></el-icon></div>
          <div class="system-info">
            <div class="system-name">荣誉系统</div>
            <div class="system-desc">赛事结束自动颁发荣誉</div>
          </div>
        </div>
        <div class="system-item">
          <div class="system-icon points"><el-icon><DataAnalysis /></el-icon></div>
          <div class="system-info">
            <div class="system-name">年度积分系统</div>
            <div class="system-desc">决定Super赛参赛资格</div>
          </div>
        </div>
        <div class="system-item">
          <div class="system-icon data"><el-icon><TrendCharts /></el-icon></div>
          <div class="system-info">
            <div class="system-name">数据中心系统</div>
            <div class="system-desc">选手表现数据与MVP评选</div>
          </div>
        </div>
        <div class="system-item">
          <div class="system-icon transfer"><el-icon><Switch /></el-icon></div>
          <div class="system-info">
            <div class="system-name">转会系统</div>
            <div class="system-desc">选手合同与AI转会决策</div>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h4><el-icon><Aim /></el-icon> 游戏目标</h4>
      <div class="goals-list">
        <div class="goal-item">
          <el-icon class="goal-icon"><Trophy /></el-icon>
          <span>带领你的战队赢得 <strong>S 世界赛冠军</strong></span>
        </div>
        <div class="goal-item">
          <el-icon class="goal-icon"><StarFilled /></el-icon>
          <span>累积年度积分，参加 <strong>Super 洲际邀请赛</strong></span>
        </div>
        <div class="goal-item">
          <el-icon class="goal-icon"><Medal /></el-icon>
          <span>培养明星选手，建立 <strong>荣誉殿堂</strong></span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Collection,
  Clock,
  VideoPlay,
  Medal,
  DataAnalysis,
  TrendCharts,
  Switch,
  Aim,
  Trophy,
  StarFilled
} from '@element-plus/icons-vue'
</script>

<style scoped lang="scss">
.guide-content {
  padding: 8px 0;

  .guide-desc {
    color: #6b7280;
    margin-bottom: 16px;
    font-size: 14px;
  }

  .section {
    margin-bottom: 24px;

    h4 {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 15px;
      font-weight: 600;
      color: #374151;
      margin: 0 0 12px 0;
      padding-bottom: 8px;
      border-bottom: 1px solid #e5e7eb;
    }
  }

  // 游戏概述 - 系统网格
  .system-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    .system-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px;
      background: #f9fafb;
      border-radius: 10px;
      border: 1px solid #e5e7eb;
      transition: all 0.2s ease;

      &:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
      }

      .system-icon {
        width: 42px;
        height: 42px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 20px;
        color: white;

        &.time { background: linear-gradient(135deg, #f59e0b, #d97706); }
        &.match { background: linear-gradient(135deg, #3b82f6, #2563eb); }
        &.honor { background: linear-gradient(135deg, #eab308, #ca8a04); }
        &.points { background: linear-gradient(135deg, #22c55e, #16a34a); }
        &.data { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }
        &.transfer { background: linear-gradient(135deg, #06b6d4, #0891b2); }
      }

      .system-info {
        flex: 1;

        .system-name {
          font-size: 14px;
          font-weight: 600;
          color: #374151;
        }

        .system-desc {
          font-size: 12px;
          color: #9ca3af;
          margin-top: 2px;
        }
      }
    }
  }

  // 游戏目标
  .goals-list {
    display: flex;
    flex-direction: column;
    gap: 10px;

    .goal-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 14px 16px;
      background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
      border-radius: 10px;
      font-size: 14px;
      color: #92400e;

      .goal-icon {
        font-size: 20px;
        color: #d97706;
      }
    }
  }
}

@media (max-width: 768px) {
  .guide-content {
    .system-grid {
      grid-template-columns: 1fr;
    }
  }
}
</style>
