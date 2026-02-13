<template>
  <div class="guide-content">
    <p class="guide-desc">游戏时间按 S1、S2、S3... 推进，每个赛季包含以下阶段：</p>

    <!-- 赛季流程图 -->
    <div class="season-flow">
      <div class="flow-container">
        <div class="flow-item spring">
          <div class="flow-icon">
            <el-icon><Sunny /></el-icon>
          </div>
          <div class="flow-label">春季联赛</div>
          <div class="flow-sub">常规赛+季后赛</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item msi">
          <div class="flow-icon">
            <el-icon><Trophy /></el-icon>
          </div>
          <div class="flow-label">MSI季中赛</div>
          <div class="flow-sub">12队双败淘汰</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item madrid">
          <div class="flow-icon">
            <el-icon><Flag /></el-icon>
          </div>
          <div class="flow-label">马德里大师赛</div>
          <div class="flow-sub">32队小组+淘汰</div>
        </div>
      </div>

      <div class="flow-container">
        <div class="flow-item summer">
          <div class="flow-icon">
            <el-icon><Sunny /></el-icon>
          </div>
          <div class="flow-label">夏季联赛</div>
          <div class="flow-sub">常规赛+季后赛</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item clauch">
          <div class="flow-icon">
            <el-icon><Promotion /></el-icon>
          </div>
          <div class="flow-label">C洲际赛</div>
          <div class="flow-sub">32队小组+淘汰</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item worlds">
          <div class="flow-icon">
            <el-icon><Trophy /></el-icon>
          </div>
          <div class="flow-label">S世界赛</div>
          <div class="flow-sub">瑞士轮+淘汰赛</div>
        </div>
      </div>

      <div class="flow-container">
        <div class="flow-item shanghai">
          <div class="flow-icon">
            <el-icon><Star /></el-icon>
          </div>
          <div class="flow-label">上海大师赛</div>
          <div class="flow-sub">12队双败淘汰</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item icp">
          <div class="flow-icon">
            <el-icon><Link /></el-icon>
          </div>
          <div class="flow-label">洲际对抗赛</div>
          <div class="flow-sub">4赛区对抗</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item super">
          <div class="flow-icon">
            <el-icon><StarFilled /></el-icon>
          </div>
          <div class="flow-label">Super洲际杯</div>
          <div class="flow-sub">年度前16队</div>
        </div>
      </div>

      <div class="flow-container last">
        <div class="flow-item transfer">
          <div class="flow-icon">
            <el-icon><Switch /></el-icon>
          </div>
          <div class="flow-label">转会期</div>
          <div class="flow-sub">买卖选手</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item draft">
          <div class="flow-icon">
            <el-icon><User /></el-icon>
          </div>
          <div class="flow-label">选秀</div>
          <div class="flow-sub">每4赛季一次</div>
        </div>
        <div class="flow-arrow">
          <el-icon><ArrowRight /></el-icon>
        </div>
        <div class="flow-item end">
          <div class="flow-icon">
            <el-icon><CircleCheck /></el-icon>
          </div>
          <div class="flow-label">赛季结束</div>
          <div class="flow-sub">进入下赛季</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Sunny,
  Trophy,
  Flag,
  Promotion,
  Star,
  Link,
  StarFilled,
  Switch,
  User,
  CircleCheck,
  ArrowRight
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

  // 赛季流程图
  .season-flow {
    .flow-container {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 12px;
      margin-bottom: 16px;
      flex-wrap: wrap;

      &.last {
        margin-bottom: 0;
      }
    }

    .flow-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 12px 16px;
      border-radius: 12px;
      min-width: 100px;

      .flow-icon {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 20px;
        color: white;
        margin-bottom: 8px;
      }

      .flow-label {
        font-size: 13px;
        font-weight: 600;
        color: #374151;
      }

      .flow-sub {
        font-size: 11px;
        color: #9ca3af;
        margin-top: 2px;
      }

      &.spring, &.summer {
        background: #fef3c7;
        .flow-icon { background: #f59e0b; }
      }
      &.msi, &.worlds {
        background: #dbeafe;
        .flow-icon { background: #3b82f6; }
      }
      &.madrid, &.clauch {
        background: #f3e8ff;
        .flow-icon { background: #8b5cf6; }
      }
      &.shanghai {
        background: #fce7f3;
        .flow-icon { background: #ec4899; }
      }
      &.icp {
        background: #cffafe;
        .flow-icon { background: #06b6d4; }
      }
      &.super {
        background: #fef9c3;
        .flow-icon { background: #eab308; }
      }
      &.transfer {
        background: #d1fae5;
        .flow-icon { background: #10b981; }
      }
      &.draft {
        background: #e0e7ff;
        .flow-icon { background: #6366f1; }
      }
      &.end {
        background: #f3f4f6;
        .flow-icon { background: #6b7280; }
      }
    }

    .flow-arrow {
      color: #d1d5db;
      font-size: 20px;
    }
  }
}
</style>
