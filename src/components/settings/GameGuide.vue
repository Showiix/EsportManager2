<template>
  <div class="game-guide">
    <el-collapse v-model="activeNames" accordion>
      <!-- 游戏概述 -->
      <el-collapse-item name="overview">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #6366f1;"><Monitor /></el-icon>
            <span>游戏概述</span>
          </div>
        </template>
        <GuideOverview />
      </el-collapse-item>

      <!-- 赛季周期 -->
      <el-collapse-item name="season">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #f59e0b;"><Calendar /></el-icon>
            <span>赛季周期</span>
          </div>
        </template>
        <GuideSeason />
      </el-collapse-item>

      <!-- 联赛体系 -->
      <el-collapse-item name="league">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #3b82f6;"><Aim /></el-icon>
            <span>联赛体系</span>
          </div>
        </template>
        <GuideLeague />
      </el-collapse-item>

      <!-- 国际赛事 -->
      <el-collapse-item name="international">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #8b5cf6;"><Trophy /></el-icon>
            <span>国际赛事</span>
          </div>
        </template>
        <GuideInternational />
      </el-collapse-item>

      <!-- 积分体系 -->
      <el-collapse-item name="points">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #22c55e;"><DataAnalysis /></el-icon>
            <span>积分体系</span>
          </div>
        </template>
        <GuidePoints />
      </el-collapse-item>

      <!-- 荣誉系统 -->
      <el-collapse-item name="honor">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #eab308;"><Medal /></el-icon>
            <span>荣誉系统</span>
          </div>
        </template>
        <GuideHonor />
      </el-collapse-item>

      <!-- 选手体系 -->
      <el-collapse-item name="player">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #ec4899;"><User /></el-icon>
            <span>选手体系</span>
          </div>
        </template>
        <GuidePlayer />
      </el-collapse-item>

      <!-- 比赛模拟 -->
      <el-collapse-item name="simulation">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #f97316;"><Cpu /></el-icon>
            <span>比赛模拟</span>
          </div>
        </template>
        <GuideSimulation />
      </el-collapse-item>

      <!-- 数据中心 -->
      <el-collapse-item name="datacenter">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #8b5cf6;"><TrendCharts /></el-icon>
            <span>数据中心</span>
          </div>
        </template>
        <GuideDataCenter />
      </el-collapse-item>

      <!-- 转会系统 -->
      <el-collapse-item name="transfer">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #06b6d4;"><Switch /></el-icon>
            <span>转会系统</span>
          </div>
        </template>
        <GuideTransfer />
      </el-collapse-item>

      <!-- 财政系统 -->
      <el-collapse-item name="finance">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #10b981;"><Wallet /></el-icon>
            <span>财政系统</span>
          </div>
        </template>
        <GuideFinance />
      </el-collapse-item>

      <!-- 选秀系统 -->
      <el-collapse-item name="draft">
        <template #title>
          <div class="collapse-title">
            <el-icon class="title-icon" style="color: #a855f7;"><UserFilled /></el-icon>
            <span>选秀系统</span>
          </div>
        </template>
        <GuideDraft />
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  Monitor,
  Calendar,
  Aim,
  Trophy,
  DataAnalysis,
  Medal,
  User,
  Cpu,
  TrendCharts,
  Switch,
  Wallet,
  UserFilled
} from '@element-plus/icons-vue'

import GuideOverview from './guide/GuideOverview.vue'
import GuideSeason from './guide/GuideSeason.vue'
import GuideLeague from './guide/GuideLeague.vue'
import GuideInternational from './guide/GuideInternational.vue'
import GuidePoints from './guide/GuidePoints.vue'
import GuideHonor from './guide/GuideHonor.vue'
import GuidePlayer from './guide/GuidePlayer.vue'
import GuideSimulation from './guide/GuideSimulation.vue'
import GuideDataCenter from './guide/GuideDataCenter.vue'
import GuideTransfer from './guide/GuideTransfer.vue'
import GuideFinance from './guide/GuideFinance.vue'
import GuideDraft from './guide/GuideDraft.vue'

const activeNames = ref('overview')
</script>

<style scoped lang="scss">
.game-guide {
  .collapse-title {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 15px;
    font-weight: 600;

    .title-icon {
      font-size: 20px;
    }
  }
}
</style>
