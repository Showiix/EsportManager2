<template>
  <div class="player-detail-view">
    <!-- 返回按钮 -->
    <div class="back-link">
      <el-button text @click="$router.push('/players')">
        <el-icon><ArrowLeft /></el-icon>
        返回选手列表
      </el-button>
    </div>

    <!-- 选手头部信息 -->
    <el-card class="profile-card">
      <div class="profile-content">
        <!-- 头像区域 -->
        <div class="avatar-section">
          <div class="player-avatar" :class="player.region.toLowerCase()">
            {{ player.position }}
          </div>
          <el-tag :type="getTalentType(player.tag)" size="large" effect="dark" class="talent-tag">
            {{ getTalentLabel(player.tag) }}
          </el-tag>
        </div>

        <!-- 基本信息 -->
        <div class="info-section">
          <div class="player-header">
            <h1 class="player-name">{{ player.gameId }}</h1>
            <div class="player-tags">
              <el-tag :type="getPositionType(player.position)" size="default">
                {{ getPositionName(player.position) }}
              </el-tag>
              <el-tag :type="getRegionType(player.region)" size="default">
                {{ player.region }}
              </el-tag>
              <el-tag type="success" size="default">在役</el-tag>
            </div>
          </div>
          <p class="player-real-name">{{ player.realName }} · {{ player.nationality }}</p>
          <div class="player-team">
            <div class="team-avatar mini" :class="player.region.toLowerCase()">
              {{ player.team.substring(0, 2) }}
            </div>
            <span>{{ player.team }}</span>
          </div>
        </div>

        <!-- 能力值展示 -->
        <div class="stats-section">
          <div class="stat-number-display">
            <span class="stat-value" :style="{ color: getAbilityColor(player.ability) }">{{ player.ability }}</span>
            <span class="stat-label">能力</span>
          </div>
          <div class="stat-number-display">
            <span class="stat-value" style="color: #8b5cf6;">{{ player.potential }}</span>
            <span class="stat-label">潜力</span>
          </div>
          <div class="stat-number-display">
            <span class="stat-value" style="color: #22c55e;">{{ player.stability }}</span>
            <span class="stat-label">稳定</span>
          </div>
          <div class="stat-number-display">
            <span class="stat-value" :style="{ color: getLoyaltyColor(player.loyalty) }">{{ player.loyalty }}</span>
            <span class="stat-label">忠诚</span>
          </div>
          <div class="stat-number-display">
            <span class="stat-value" :style="{ color: getSatisfactionColor(player.satisfaction) }">{{ player.satisfaction }}</span>
            <span class="stat-label">满意</span>
          </div>
          <div class="stat-text">
            <div class="age-display">
              <span class="age-value">{{ player.age }}</span>
              <span class="age-label">岁</span>
            </div>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 详细信息区 -->
    <el-row :gutter="20" class="detail-row">
      <!-- 合同信息 -->
      <el-col :span="12">
        <el-card class="detail-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon><Document /></el-icon>
                合同信息
              </h2>
            </div>
          </template>
          <div class="info-list">
            <div class="info-row">
              <span class="info-label">所属战队</span>
              <span class="info-value">{{ player.team }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">所属赛区</span>
              <el-tag :type="getRegionType(player.region)" size="small">
                {{ player.region }}
              </el-tag>
            </div>
            <div class="info-row">
              <span class="info-label">合同到期</span>
              <span class="info-value highlight">{{ player.contractEnd }} 赛季</span>
            </div>
            <div class="info-row">
              <span class="info-label">年薪</span>
              <span class="info-value money">{{ formatMoney(player.salary) }}</span>
            </div>
            <div class="info-row clickable" @click="openMarketValueDetail">
              <span class="info-label">身价</span>
              <span class="info-value success">
                {{ formatMoney(displayMarketValue) }}
                <el-icon class="click-icon"><ArrowRight /></el-icon>
              </span>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 职业生涯 -->
      <el-col :span="12">
        <el-card class="detail-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon><TrendCharts /></el-icon>
                职业生涯
              </h2>
            </div>
          </template>
          <div class="info-list">
            <div class="info-row">
              <span class="info-label">加入赛季</span>
              <span class="info-value">{{ player.joinSeason }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">职业年数</span>
              <span class="info-value">{{ careerYears }} 年</span>
            </div>
            <div class="info-row">
              <span class="info-label">冠军数</span>
              <span class="info-value gold">{{ championCount }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">总荣誉</span>
              <span class="info-value">{{ honors.length }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">成长空间</span>
              <span class="info-value purple">+{{ player.potential - player.ability }}</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 天赋说明 -->
    <el-alert
      :title="getTalentDescription(player.tag)"
      :type="getTalentAlertType(player.tag)"
      :closable="false"
      show-icon
      class="talent-alert"
    />

    <!-- 特性与状态区 -->
    <el-row :gutter="20" class="traits-condition-row">
      <!-- 选手特性 -->
      <el-col :span="12">
        <el-card class="traits-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon class="header-icon"><Lightning /></el-icon>
                选手特性
              </h2>
              <div class="header-actions">
                <el-button size="small" text @click="showTraitsGuide = true">
                  <el-icon><InfoFilled /></el-icon>
                  特性图鉴
                </el-button>
                <span class="count-badge">{{ traits.length }} 项特性</span>
              </div>
            </div>
          </template>

          <el-empty v-if="traits.length === 0" description="暂无特性" :image-size="60">
            <template #image>
              <el-icon class="empty-icon"><Aim /></el-icon>
            </template>
          </el-empty>

          <div v-else class="traits-grid">
            <div
              v-for="trait in traits"
              :key="trait.trait_type"
              class="trait-item clickable"
              :class="[`rarity-${trait.rarity}`, { 'negative': trait.is_negative }]"
              @click="openTraitDetail(trait)"
            >
              <div class="trait-header">
                <span class="trait-name">{{ trait.name }}</span>
                <span class="trait-rarity">{{ '★'.repeat(trait.rarity) }}</span>
              </div>
              <div class="trait-description">{{ trait.description }}</div>
              <div class="trait-click-hint">点击查看详情</div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 选手状态 - 五维雷达图 -->
      <el-col :span="12">
        <el-card class="condition-card radar-card">
          <template #header>
            <div class="card-header">
              <h2>
                <el-icon class="header-icon"><DataAnalysis /></el-icon>
                能力分析
              </h2>
              <span v-if="playerStats" class="count-badge">{{ playerStats.games_played }} 场比赛</span>
            </div>
          </template>

          <!-- 五维雷达图 -->
          <div v-if="playerStats" class="radar-content">
            <div ref="radarChartRef" class="radar-chart"></div>

            <!-- 详细数值展示 -->
            <div class="radar-stats">
              <div class="stat-row">
                <span class="stat-label">
                  <el-icon class="stat-icon impact"><Lightning /></el-icon>
                  影响力
                </span>
                <div class="stat-bar-wrapper">
                  <div class="stat-bar" :style="{ width: (computeRadarData?.impact || 0) + '%' }"></div>
                </div>
                <span class="stat-value">{{ computeRadarData?.impact || 0 }}</span>
              </div>
              <div class="stat-row">
                <span class="stat-label">
                  <el-icon class="stat-icon performance"><Aim /></el-icon>
                  发挥
                </span>
                <div class="stat-bar-wrapper">
                  <div class="stat-bar performance" :style="{ width: (computeRadarData?.performance || 0) + '%' }"></div>
                </div>
                <span class="stat-value">{{ computeRadarData?.performance || 0 }}</span>
              </div>
              <div class="stat-row">
                <span class="stat-label">
                  <span class="stat-icon consistency">🛡️</span>
                  稳定性
                </span>
                <div class="stat-bar-wrapper">
                  <div class="stat-bar consistency" :style="{ width: (computeRadarData?.consistency || 0) + '%' }"></div>
                </div>
                <span class="stat-value">{{ computeRadarData?.consistency || 0 }}</span>
              </div>
              <div class="stat-row">
                <span class="stat-label">
                  <span class="stat-icon peak">🔥</span>
                  巅峰
                </span>
                <div class="stat-bar-wrapper">
                  <div class="stat-bar peak" :style="{ width: (computeRadarData?.peak || 0) + '%' }"></div>
                </div>
                <span class="stat-value">{{ computeRadarData?.peak || 0 }}</span>
              </div>
              <div class="stat-row">
                <span class="stat-label">
                  <el-icon class="stat-icon honor"><Trophy /></el-icon>
                  荣誉
                </span>
                <div class="stat-bar-wrapper">
                  <div class="stat-bar honor" :style="{ width: (computeRadarData?.honor || 0) + '%' }"></div>
                </div>
                <span class="stat-value">{{ computeRadarData?.honor || 0 }}</span>
              </div>
            </div>
          </div>

          <el-empty v-else description="暂无比赛数据" :image-size="60">
            <template #image>
              <el-icon class="empty-icon"><DataAnalysis /></el-icon>
            </template>
          </el-empty>
        </el-card>
      </el-col>
    </el-row>

    <!-- 荣誉记录 -->
    <el-card class="honors-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><Trophy /></el-icon>
            荣誉记录
          </h2>
          <span class="count-badge">共 {{ honors.length }} 项荣誉</span>
        </div>
      </template>

      <el-empty v-if="honors.length === 0" description="暂无荣誉记录">
        <template #image>
          <el-icon class="empty-icon"><Trophy /></el-icon>
        </template>
      </el-empty>

      <el-timeline v-else>
        <el-timeline-item
          v-for="honor in honors"
          :key="`${honor.season}-${honor.tournament}`"
          :timestamp="honor.season"
          placement="top"
          :color="getHonorColor(honor.position)"
          size="large"
        >
          <el-card class="honor-card" :class="getHonorClass(honor.position)" shadow="hover">
            <div class="honor-content">
              <div class="honor-icon">
                {{ getHonorEmoji(honor.position) }}
              </div>
              <div class="honor-info">
                <div class="honor-title">{{ honor.tournament }}</div>
                <el-tag :type="getHonorTagType(honor.position)" size="default" effect="dark">
                  {{ honor.position }}
                </el-tag>
              </div>
            </div>
          </el-card>
        </el-timeline-item>
      </el-timeline>
    </el-card>

    <!-- 赛季历史 -->
    <el-card class="history-card">
      <template #header>
        <div class="card-header">
          <h2>
            <el-icon><Clock /></el-icon>
            赛季历史
          </h2>
        </div>
      </template>

      <el-table :data="seasonHistory" stripe class="history-table">
        <el-table-column prop="season" label="赛季" width="120" align="center" />
        <el-table-column prop="team" label="所属战队" width="150">
          <template #default="{ row }">
            <div class="team-cell">
              <div class="team-avatar mini" :class="player.region.toLowerCase()">
                {{ row.team.substring(0, 2) }}
              </div>
              <span>{{ row.team }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="ability" label="能力值" width="120" align="center">
          <template #default="{ row }">
            <span class="ability-value" :style="{ color: getAbilityColor(row.ability) }">
              {{ row.ability }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="potential" label="潜力值" width="120" align="center">
          <template #default="{ row }">
            <span class="potential-value">{{ row.potential }}</span>
          </template>
        </el-table-column>
        <el-table-column label="成长" width="100" align="center">
          <template #default="{ row, $index }">
            <el-tag v-if="$index > 0" type="success" size="small">
              +{{ row.ability - seasonHistory[$index - 1].ability }}
            </el-tag>
            <span v-else class="text-gray">-</span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 特性详情弹窗 -->
    <el-dialog
      v-model="showTraitDialog"
      :title="''"
      width="520px"
      class="trait-detail-dialog"
      :show-close="false"
    >
      <div v-if="selectedTrait" class="trait-detail-content">
        <!-- 特性卡片头部 -->
        <div class="trait-card-header" :class="[`rarity-${selectedTrait.rarity}`, { 'negative': selectedTrait.is_negative }]">
          <div class="trait-card-close" @click="showTraitDialog = false">×</div>
          <div class="trait-card-icon">{{ getTraitIcon(selectedTrait.trait_type) }}</div>
          <div class="trait-card-name">{{ selectedTrait.name }}</div>
          <div class="trait-card-stars">
            <span v-for="n in 5" :key="n" :class="n <= selectedTrait.rarity ? 'star-filled' : 'star-empty'">★</span>
          </div>
          <div class="trait-card-type">
            <span v-if="selectedTrait.is_negative" class="type-negative">负面特性</span>
            <span v-else class="type-positive">正面特性</span>
          </div>
        </div>

        <!-- 特性描述 -->
        <div class="trait-info-card">
          <div class="info-card-title">
            <span class="icon">📝</span>
            <span>特性描述</span>
          </div>
          <div class="info-card-content description">{{ selectedTrait.description }}</div>
        </div>

        <!-- 效果详情 -->
        <div class="trait-info-card">
          <div class="info-card-title">
            <span class="icon">⚡</span>
            <span>效果详情</span>
          </div>
          <div class="effects-table">
            <div v-for="(effect, index) in getTraitEffects(selectedTrait.trait_type)" :key="index" class="effect-row">
              <div class="effect-label">{{ effect.condition }}</div>
              <div class="effect-val" :class="{ 'val-positive': effect.positive, 'val-negative': !effect.positive }">
                {{ effect.value }}
              </div>
            </div>
          </div>
        </div>

        <!-- 触发条件 & 作用机制 -->
        <div class="trait-info-grid">
          <div class="trait-info-card small">
            <div class="info-card-title">
              <span class="icon">🎯</span>
              <span>触发条件</span>
            </div>
            <div class="info-card-content">{{ getTraitTrigger(selectedTrait.trait_type) }}</div>
          </div>
          <div class="trait-info-card small">
            <div class="info-card-title">
              <span class="icon">⚙️</span>
              <span>作用机制</span>
            </div>
            <div class="info-card-content">{{ getTraitMechanism(selectedTrait.trait_type) }}</div>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 身价详情弹窗 -->
    <el-dialog
      v-model="showMarketValueDialog"
      title=""
      width="520px"
      class="market-value-dialog"
      :show-close="true"
    >
      <div class="mv-content">
        <!-- 顶部大卡片：当前身价 -->
        <div class="mv-hero">
          <div class="mv-hero-bg"></div>
          <div class="mv-hero-content">
            <div class="mv-label">计算身价</div>
            <div class="mv-amount">{{ formatMoney(marketValueFactors.calculatedValue) }}</div>
            <div class="mv-player">{{ player.gameId }} · {{ getPositionName(player.position) }}</div>
          </div>
        </div>

        <!-- 系数分解 - 紧凑列表 -->
        <div class="mv-factors">
          <div class="mv-section-title">身价构成</div>
          <div class="mv-factor-list">
            <div class="mv-factor-row">
              <span class="mv-factor-icon">💎</span>
              <span class="mv-factor-name">基础身价</span>
              <span class="mv-factor-detail">能力{{ player.ability }}</span>
              <span class="mv-factor-val primary">{{ formatMoney(marketValueFactors.baseValue) }}</span>
            </div>
            <div class="mv-factor-row">
              <span class="mv-factor-icon">📅</span>
              <span class="mv-factor-name">年龄</span>
              <span class="mv-factor-detail">{{ player.age }}岁</span>
              <span class="mv-factor-val" :class="marketValueFactors.ageFactor >= 1 ? 'positive' : 'negative'">
                × {{ marketValueFactors.ageFactor.toFixed(2) }}
              </span>
            </div>
            <div class="mv-factor-row">
              <el-icon class="mv-factor-icon"><TrendCharts /></el-icon>
              <span class="mv-factor-name">潜力</span>
              <span class="mv-factor-detail">差值{{ player.potential - player.ability }}</span>
              <span class="mv-factor-val" :class="marketValueFactors.potentialFactor >= 1 ? 'positive' : 'negative'">
                × {{ marketValueFactors.potentialFactor.toFixed(2) }}
              </span>
            </div>
            <div class="mv-factor-row">
              <el-icon class="mv-factor-icon"><StarFilled /></el-icon>
              <span class="mv-factor-name">天赋</span>
              <span class="mv-factor-detail">{{ getTalentLabel(player.tag) }}</span>
              <span class="mv-factor-val" :class="marketValueFactors.tagFactor >= 1 ? 'positive' : 'negative'">
                × {{ marketValueFactors.tagFactor.toFixed(2) }}
              </span>
            </div>
            <div class="mv-factor-row">
              <el-icon class="mv-factor-icon"><Monitor /></el-icon>
              <span class="mv-factor-name">位置</span>
              <span class="mv-factor-detail">{{ getPositionName(player.position) }}</span>
              <span class="mv-factor-val" :class="marketValueFactors.positionFactor >= 1 ? 'positive' : 'negative'">
                × {{ marketValueFactors.positionFactor.toFixed(2) }}
              </span>
            </div>
            <div class="mv-factor-row">
              <span class="mv-factor-icon">🌍</span>
              <span class="mv-factor-name">赛区</span>
              <span class="mv-factor-detail">{{ player.region }}</span>
              <span class="mv-factor-val" :class="marketValueFactors.regionFactor >= 1 ? 'positive' : 'negative'">
                × {{ marketValueFactors.regionFactor.toFixed(2) }}
              </span>
            </div>
            <div class="mv-factor-row highlight">
              <el-icon class="mv-factor-icon"><Trophy /></el-icon>
              <span class="mv-factor-name">荣誉</span>
              <span class="mv-factor-detail">{{ getHonorDescription(marketValueFactors.honorFactor, marketValueFactors.teamHonorCount, marketValueFactors.individualHonorCount) }}</span>
              <span class="mv-factor-val" :class="marketValueFactors.honorFactor > 1 ? 'positive' : ''">
                × {{ marketValueFactors.honorFactor.toFixed(2) }}
              </span>
            </div>
          </div>
        </div>

        <!-- 身价变化记录 -->
        <div class="mv-history">
          <div class="mv-section-title">变化记录</div>
          <div class="mv-history-list" v-if="marketValueChanges.length > 0">
            <div class="mv-history-item" v-for="change in marketValueChanges" :key="change.id">
              <div class="mv-history-left">
                <el-tag size="small" :type="change.change_amount > 0 ? 'success' : 'danger'" effect="dark">
                  {{ change.reason }}
                </el-tag>
              </div>
              <div class="mv-history-right">
                <span class="mv-history-change" :class="change.change_amount > 0 ? 'up' : 'down'">
                  {{ change.change_amount > 0 ? '↑' : '↓' }} {{ Math.abs(change.change_percent).toFixed(0) }}%
                </span>
              </div>
            </div>
          </div>
          <div class="mv-no-history" v-else>
            <span class="mv-no-icon">📭</span>
            <span>暂无变化记录</span>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 特性图鉴弹窗 -->
    <el-dialog
      v-model="showTraitsGuide"
      title="特性图鉴"
      width="700px"
      class="traits-guide-dialog"
    >
      <div class="traits-guide-content">
        <!-- 稀有度说明 -->
        <div class="rarity-legend">
          <span class="legend-title">稀有度说明：</span>
          <span class="legend-item rarity-1">★ 普通</span>
          <span class="legend-item rarity-2">★★ 稀有</span>
          <span class="legend-item rarity-3">★★★ 精良</span>
          <span class="legend-item rarity-4">★★★★ 史诗</span>
          <span class="legend-item rarity-5">★★★★★ 传说</span>
        </div>

        <!-- 特性分类 -->
        <div class="traits-category">
          <div class="category-title">正面特性</div>
          <div class="traits-grid-guide">
            <div
              v-for="trait in allTraits.filter(t => !t.isNegative)"
              :key="trait.type"
              class="trait-guide-item"
              :class="`rarity-${trait.rarity}`"
            >
              <div class="trait-guide-header">
                <span class="trait-guide-icon">{{ trait.icon }}</span>
                <span class="trait-guide-name">{{ trait.name }}</span>
                <span class="trait-guide-stars">{{ '★'.repeat(trait.rarity) }}</span>
              </div>
              <div class="trait-guide-desc">{{ trait.description }}</div>
              <div class="trait-guide-effect">
                <span v-for="(effect, idx) in getTraitEffects(trait.type).slice(0, 2)" :key="idx" class="effect-tag" :class="{ positive: effect.positive, negative: !effect.positive }">
                  {{ effect.value }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div class="traits-category negative">
          <div class="category-title">负面特性</div>
          <div class="traits-grid-guide">
            <div
              v-for="trait in allTraits.filter(t => t.isNegative)"
              :key="trait.type"
              class="trait-guide-item negative"
              :class="`rarity-${trait.rarity}`"
            >
              <div class="trait-guide-header">
                <span class="trait-guide-icon">{{ trait.icon }}</span>
                <span class="trait-guide-name">{{ trait.name }}</span>
                <span class="trait-guide-stars">{{ '★'.repeat(trait.rarity) }}</span>
              </div>
              <div class="trait-guide-desc">{{ trait.description }}</div>
              <div class="trait-guide-effect">
                <span v-for="(effect, idx) in getTraitEffects(trait.type).slice(0, 2)" :key="idx" class="effect-tag" :class="{ positive: effect.positive, negative: !effect.positive }">
                  {{ effect.value }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="showTraitsGuide = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import {
  ArrowLeft,
  ArrowRight,
  Document,
  TrendCharts,
  Trophy,
  Clock,
  InfoFilled,
  Lightning,
  DataAnalysis,
  Aim,
  StarFilled,
  Monitor,
} from '@element-plus/icons-vue'
import { teamApi, playerApi, honorApi, statsApi, formatHonorType, type TraitInfo, type PlayerConditionInfo, type MarketValueChange } from '@/api/tauri'
import { formatMoney } from '@/utils'
import { useTeamStoreTauri } from '@/stores/useTeamStoreTauri'
import * as echarts from 'echarts'
import { createLogger } from '@/utils/logger'

const logger = createLogger('PlayerDetail')

const route = useRoute()
const playerId = route.params.id as string
const teamStore = useTeamStoreTauri()

// 选手数据
const player = ref({
  id: playerId,
  gameId: '加载中...',
  realName: '加载中...',
  nationality: '未知',
  team: '未知',
  region: 'LPL',
  position: 'MID',
  age: 20,
  ability: 70,
  potential: 80,
  stability: 75,
  loyalty: 50,
  satisfaction: 50,
  tag: 'NORMAL',
  salary: 500000,
  marketValue: 3000000,
  calculatedMarketValue: 0,  // 计算后的身价（含荣誉和赛区系数）
  contractEnd: 'S2',
  joinSeason: 'S1',
})

// 荣誉记录
const honors = ref<Array<{season: string, tournament: string, position: string}>>([])

// 选手特性
const traits = ref<TraitInfo[]>([])

// 选手状态因子
const conditionInfo = ref<PlayerConditionInfo | null>(null)

// 选手赛季统计（用于五维图）
interface PlayerSeasonStats {
  avg_impact: number
  avg_performance: number
  best_performance: number
  worst_performance: number
  consistency_score: number
  champion_bonus: number
  games_played: number
  international_titles: number
  regional_titles: number
}
const playerStats = ref<PlayerSeasonStats | null>(null)

// 雷达图相关
const radarChartRef = ref<HTMLDivElement | null>(null)
let radarChart: echarts.ECharts | null = null

// 特性弹窗状态
const showTraitDialog = ref(false)
const selectedTrait = ref<TraitInfo | null>(null)

// 特性图鉴弹窗
const showTraitsGuide = ref(false)

// 所有特性数据
const allTraits = [
  { type: 'clutch', name: '大赛型', description: '在季后赛和国际赛中状态更好', rarity: 4, isNegative: false, icon: '🎯' },
  { type: 'slowstarter', name: '慢热型', description: '系列赛开局较慢，但后期渐入佳境', rarity: 2, isNegative: false, icon: '🐢' },
  { type: 'faststarter', name: '快枪手', description: '系列赛开局强势，但后期可能疲软', rarity: 2, isNegative: false, icon: '⚡' },
  { type: 'explosive', name: '爆发型', description: '发挥波动大，但巅峰更高', rarity: 3, isNegative: false, icon: '💥' },
  { type: 'consistent', name: '稳定型', description: '发挥稳定，但上限略低', rarity: 2, isNegative: false, icon: '🛡️' },
  { type: 'comebackking', name: '逆风王', description: '落后时愈战愈勇', rarity: 4, isNegative: false, icon: '👑' },
  { type: 'tilter', name: '顺风浪', description: '心态容易受比分影响', rarity: 1, isNegative: true, icon: '😰' },
  { type: 'mentalfortress', name: '心态大师', description: '心态稳定，不受连胜连败影响', rarity: 4, isNegative: false, icon: '🧠' },
  { type: 'fragile', name: '玻璃心', description: '输了比赛心态下滑更快', rarity: 1, isNegative: true, icon: '💔' },
  { type: 'ironman', name: '铁人', description: '不受连续比赛疲劳影响', rarity: 3, isNegative: false, icon: '💪' },
  { type: 'volatile', name: '状态敏感', description: '状态波动比常人更大', rarity: 2, isNegative: true, icon: '🎲' },
  { type: 'risingstar', name: '新星', description: '新人赛季潜力爆发', rarity: 3, isNegative: false, icon: '⭐' },
  { type: 'veteran', name: '老将风范', description: '老将经验丰富，发挥更稳', rarity: 3, isNegative: false, icon: '🎖️' },
  { type: 'teamleader', name: '团队核心', description: '带动队友发挥', rarity: 5, isNegative: false, icon: '🏅' },
]

// 身价弹窗状态
const showMarketValueDialog = ref(false)
const marketValueChanges = ref<MarketValueChange[]>([])

// 加载身价变化记录
const loadMarketValueChanges = async () => {
  try {
    const numericId = parseInt(playerId)
    if (!isNaN(numericId)) {
      const result = await statsApi.getPlayerMarketValueChanges(numericId)
      if (result) {
        marketValueChanges.value = result
      }
    }
  } catch (e) {
    logger.error('Failed to load market value changes:', e)
  }
}

// 监听身价弹窗打开
watch(showMarketValueDialog, (newVal) => {
  if (newVal) {
    loadMarketValueChanges()
  }
})

// 赛季历史
const seasonHistory = ref<Array<{season: string, team: string, ability: number, potential: number}>>([])

// 位置简称映射
const positionShortMap: Record<string, string> = {
  'Top': 'TOP', 'Jungle': 'JUG', 'Mid': 'MID', 'Adc': 'ADC', 'Support': 'SUP'
}

// 加载选手数据
onMounted(async () => {
  try {
    // 尝试将 playerId 转换为数字（后端返回的是数字ID）
    const numericId = parseInt(playerId)

    if (!isNaN(numericId)) {
      // 使用数字ID直接从API获取选手
      const foundPlayer = await playerApi.getPlayer(numericId)

      if (foundPlayer) {
        // 加载赛区信息获取赛区代码
        await teamStore.loadRegions()

        // 获取队伍信息
        let teamName = '未知'
        let regionCode = 'LPL'

        if (foundPlayer.team_id) {
          try {
            const team = await teamApi.getTeam(foundPlayer.team_id)
            teamName = team.name
            const region = teamStore.regions.find(r => r.id === team.region_id)
            regionCode = region?.code || 'LPL'
          } catch (e) {
            logger.error('Failed to get team info:', e)
          }
        }

        // 计算天赋标签
        const tag = foundPlayer.potential >= 90 || foundPlayer.ability >= 85 ? 'GENIUS'
          : foundPlayer.potential >= 75 || foundPlayer.ability >= 70 ? 'NORMAL'
          : 'ORDINARY'

        // 计算身价和工资
        const marketValue = foundPlayer.ability * 100000 + foundPlayer.potential * 50000
        const salary = Math.round(marketValue * 0.15)

        // 位置转换
        const position = positionShortMap[foundPlayer.position || ''] || foundPlayer.position || 'MID'

        player.value = {
          id: playerId,
          gameId: foundPlayer.game_id,
          realName: foundPlayer.real_name || foundPlayer.game_id,
          nationality: getRegionNationality(regionCode),
          team: teamName,
          region: regionCode,
          position: position,
          age: foundPlayer.age,
          ability: foundPlayer.ability,
          potential: foundPlayer.potential,
          stability: foundPlayer.stability || Math.round(70 + (30 - foundPlayer.age) * 0.5 + Math.random() * 10),
          loyalty: foundPlayer.loyalty ?? 50,
          satisfaction: foundPlayer.satisfaction ?? 50,
          tag: tag,
          salary: foundPlayer.salary || salary,
          marketValue: foundPlayer.market_value || marketValue,
          calculatedMarketValue: foundPlayer.calculated_market_value || 0,  // 计算后的身价
          contractEnd: foundPlayer.contract_end_season ? `S${foundPlayer.contract_end_season}` : 'S3',
          joinSeason: 'S1',
        }

        // 加载选手荣誉
        try {
          const playerHonors = await honorApi.getPlayerHonors(numericId)
          honors.value = playerHonors.map(h => ({
            season: `S${h.season_id}`,
            tournament: h.tournament_name,
            position: formatHonorType(h.honor_type)
          }))
        } catch (e) {
          logger.error('Failed to load player honors:', e)
          honors.value = []
        }

        // 加载选手特性和状态
        try {
          const [traitsData, conditionData] = await Promise.all([
            playerApi.getPlayerTraits(numericId),
            playerApi.getPlayerCondition(numericId)
          ])
          traits.value = traitsData || []
          conditionInfo.value = conditionData
        } catch (e) {
          logger.error('Failed to load traits/condition:', e)
          traits.value = []
          conditionInfo.value = null
        }

        // 加载选手赛季统计数据（用于五维图）
        try {
          const statsResult = await statsApi.getPlayerStats(numericId)
          if (statsResult && statsResult.length > 0) {
            // 取最新赛季的统计
            playerStats.value = statsResult[statsResult.length - 1]
            // 初始化雷达图
            await nextTick()
            initRadarChart()
          }
        } catch (e) {
          logger.error('Failed to load player stats:', e)
          playerStats.value = null
        }

        // 生成赛季历史
        seasonHistory.value = [{
          season: 'S1',
          team: teamName,
          ability: foundPlayer.ability,
          potential: foundPlayer.potential
        }]
      }
    }
  } catch (error) {
    logger.error('Failed to load player:', error)
  }
})

// 根据赛区获取国籍
const getRegionNationality = (regionCode: string) => {
  const nationalities: Record<string, string> = {
    'LPL': '中国',
    'LCK': '韩国',
    'LEC': '欧洲',
    'LCS': '北美',
  }
  return nationalities[regionCode] || '未知'
}

// 计算属性
const careerYears = computed(() => {
  const joinMatch = player.value.joinSeason.match(/S(\d+)/)
  if (joinMatch) {
    const joinYear = parseInt(joinMatch[1])
    const currentYear = 1 // 当前 S1
    return Math.max(1, currentYear - joinYear + 1)
  }
  return 1
})

const championCount = computed(() => {
  return honors.value.filter(h => h.position === '冠军').length
})

// 显示的身价：优先使用计算后的身价，如果为0则使用基础身价
const displayMarketValue = computed(() => {
  return player.value.calculatedMarketValue > 0
    ? player.value.calculatedMarketValue
    : player.value.marketValue
})

// formatMoney 从 @/utils 导入

const getRegionType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getPositionType = (position: string) => {
  const types: Record<string, string> = {
    TOP: 'danger',
    JUG: 'success',
    MID: 'primary',
    ADC: 'warning',
    SUP: 'info',
  }
  return types[position] || 'info'
}

const getPositionName = (position: string) => {
  const names: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: '下路',
    SUP: '辅助',
  }
  return names[position] || position
}

const getTalentType = (tag: string) => {
  const types: Record<string, string> = {
    GENIUS: 'warning',
    NORMAL: 'primary',
    ORDINARY: 'info',
  }
  return types[tag] || 'info'
}

const getTalentLabel = (tag: string) => {
  const labels: Record<string, string> = {
    GENIUS: '天才',
    NORMAL: '普通',
    ORDINARY: '平庸',
  }
  return labels[tag] || tag
}

const getTalentDescription = (tag: string) => {
  const desc: Record<string, string> = {
    GENIUS: '天才选手：每赛季能力值增长 +3，潜力上限更高',
    NORMAL: '普通选手：每赛季能力值增长 +2，稳定发挥',
    ORDINARY: '平庸选手：每赛季能力值增长 +1，成长较慢',
  }
  return desc[tag] || ''
}

const getTalentAlertType = (tag: string) => {
  const types: Record<string, string> = {
    GENIUS: 'warning',
    NORMAL: 'info',
    ORDINARY: 'info',
  }
  return types[tag] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
}

const getLoyaltyColor = (loyalty: number) => {
  if (loyalty >= 70) return '#22c55e'  // 高忠诚度 - 绿色
  if (loyalty >= 50) return '#3b82f6'  // 中等 - 蓝色
  if (loyalty >= 35) return '#f59e0b'  // 较低 - 橙色
  return '#ef4444'  // 低忠诚度 - 红色
}

const getSatisfactionColor = (satisfaction: number) => {
  if (satisfaction >= 65) return '#22c55e'  // 高满意度 - 绿色
  if (satisfaction >= 50) return '#3b82f6'  // 中等 - 蓝色
  if (satisfaction >= 40) return '#f59e0b'  // 较低 - 橙色
  return '#ef4444'  // 低满意度 - 红色
}

const getHonorColor = (position: string) => {
  const colors: Record<string, string> = {
    '冠军': '#fbbf24',
    '冠军成员': '#fbbf24',
    '亚军': '#9ca3af',
    '亚军成员': '#9ca3af',
    '季军': '#f97316',
    '季军成员': '#f97316',
    '殿军': '#3b82f6',
    '殿军成员': '#3b82f6',
    '赛事MVP': '#ef4444',
    '决赛MVP': '#ef4444',
    '常规赛MVP': '#ef4444',
    '季后赛FMVP': '#ef4444',
  }
  return colors[position] || '#3b82f6'
}

const getHonorClass = (position: string) => {
  const classes: Record<string, string> = {
    '冠军': 'champion',
    '冠军成员': 'champion',
    '亚军': 'runner-up',
    '亚军成员': 'runner-up',
    '季军': 'third-place',
    '季军成员': 'third-place',
    '殿军': 'fourth-place',
    '殿军成员': 'fourth-place',
  }
  return classes[position] || ''
}

const getHonorEmoji = (position: string) => {
  const emojis: Record<string, string> = {
    '冠军': '🏆',
    '冠军成员': '🏆',
    '亚军': '🥈',
    '亚军成员': '🥈',
    '季军': '🥉',
    '季军成员': '🥉',
    '殿军': '4️⃣',
    '殿军成员': '4️⃣',
    '赛事MVP': '⭐',
    '决赛MVP': '⭐',
    '常规赛MVP': '⭐',
    '季后赛FMVP': '⭐',
  }
  return emojis[position] || '🏅'
}

const getHonorTagType = (position: string) => {
  const types: Record<string, string> = {
    '冠军': 'warning',
    '冠军成员': 'warning',
    '亚军': '',           // 默认银色
    '亚军成员': '',
    '季军': 'success',    // 绿色/铜色
    '季军成员': 'success',
    '殿军': 'info',       // 蓝色
    '殿军成员': 'info',
    '赛事MVP': 'danger',  // 红色
    '决赛MVP': 'danger',
    '常规赛MVP': 'danger',
    '季后赛FMVP': 'danger',
    '常规赛第一': 'primary',
  }
  return types[position] || 'primary'
}

// ==================== 特性详情相关 ====================

// 打开特性详情弹窗
const openTraitDetail = (trait: TraitInfo) => {
  selectedTrait.value = trait
  showTraitDialog.value = true
}

// 获取特性图标
const getTraitIcon = (traitType: string): string => {
  const icons: Record<string, string> = {
    'clutch': '🎯',
    'slowstarter': '🐢',
    'slow_starter': '🐢',
    'faststarter': '⚡',
    'fast_starter': '⚡',
    'explosive': '💥',
    'consistent': '🛡️',
    'comebackking': '👑',
    'comeback_king': '👑',
    'tilter': '😰',
    'mentalfortress': '🧠',
    'mental_fortress': '🧠',
    'fragile': '💔',
    'ironman': '💪',
    'volatile': '🎲',
    'risingstar': '⭐',
    'rising_star': '⭐',
    'veteran': '🎖️',
    'teamleader': '🏅',
    'team_leader': '🏅',
  }
  return icons[traitType] || '❓'
}

// 获取特性效果详情
const getTraitEffects = (traitType: string): Array<{ condition: string; value: string; positive: boolean }> => {
  const effects: Record<string, Array<{ condition: string; value: string; positive: boolean }>> = {
    'clutch': [
      { condition: '季后赛中', value: '状态 +3', positive: true },
      { condition: '国际赛中', value: '状态 +3', positive: true },
    ],
    'slowstarter': [
      { condition: '系列赛第1局', value: '状态 -2', positive: false },
      { condition: '系列赛第3局起', value: '状态 +2', positive: true },
    ],
    'slow_starter': [
      { condition: '系列赛第1局', value: '状态 -2', positive: false },
      { condition: '系列赛第3局起', value: '状态 +2', positive: true },
    ],
    'faststarter': [
      { condition: '系列赛第1局', value: '状态 +2', positive: true },
      { condition: '系列赛第3局起', value: '状态 -1', positive: false },
    ],
    'fast_starter': [
      { condition: '系列赛第1局', value: '状态 +2', positive: true },
      { condition: '系列赛第3局起', value: '状态 -1', positive: false },
    ],
    'explosive': [
      { condition: '永久生效', value: '稳定性 -15', positive: false },
      { condition: '永久生效', value: '能力上限 +5', positive: true },
    ],
    'consistent': [
      { condition: '永久生效', value: '稳定性 +10', positive: true },
      { condition: '永久生效', value: '能力上限 -3', positive: false },
    ],
    'comebackking': [
      { condition: '比分落后时', value: '状态 +3', positive: true },
    ],
    'comeback_king': [
      { condition: '比分落后时', value: '状态 +3', positive: true },
    ],
    'tilter': [
      { condition: '比分领先时', value: '状态 -2', positive: false },
      { condition: '比分落后时', value: '状态 -3', positive: false },
    ],
    'mentalfortress': [
      { condition: '永久生效', value: '动能效果 ×0.5', positive: true },
    ],
    'mental_fortress': [
      { condition: '永久生效', value: '动能效果 ×0.5', positive: true },
    ],
    'fragile': [
      { condition: '输掉比赛后', value: '动能 -2（而非-1）', positive: false },
    ],
    'ironman': [
      { condition: '永久生效', value: '无疲劳惩罚', positive: true },
    ],
    'volatile': [
      { condition: '永久生效', value: '稳定性 -10', positive: false },
    ],
    'risingstar': [
      { condition: '首个职业赛季', value: '能力值 +3', positive: true },
    ],
    'rising_star': [
      { condition: '首个职业赛季', value: '能力值 +3', positive: true },
    ],
    'veteran': [
      { condition: '30岁后', value: '稳定性 +15', positive: true },
    ],
    'teamleader': [
      { condition: '永久生效', value: '队友状态 +1', positive: true },
    ],
    'team_leader': [
      { condition: '永久生效', value: '队友状态 +1', positive: true },
    ],
  }
  return effects[traitType] || []
}

// 获取特性触发条件
const getTraitTrigger = (traitType: string): string => {
  const triggers: Record<string, string> = {
    'clutch': '当比赛为季后赛或国际赛事（MSI、世界赛、大师赛等）时自动触发',
    'slowstarter': '根据当前系列赛局数自动判断，BO3/BO5 中第1局和第3局以后效果不同',
    'slow_starter': '根据当前系列赛局数自动判断，BO3/BO5 中第1局和第3局以后效果不同',
    'faststarter': '根据当前系列赛局数自动判断，BO3/BO5 中第1局和第3局以后效果不同',
    'fast_starter': '根据当前系列赛局数自动判断，BO3/BO5 中第1局和第3局以后效果不同',
    'explosive': '无条件永久生效，比赛计算时自动应用属性修正',
    'consistent': '无条件永久生效，比赛计算时自动应用属性修正',
    'comebackking': '当己方在系列赛中比分落后时触发（如 0-1、1-2）',
    'comeback_king': '当己方在系列赛中比分落后时触发（如 0-1、1-2）',
    'tilter': '根据当前系列赛比分判断，领先和落后都会受影响',
    'mentalfortress': '永久生效，连胜连败带来的动能变化减半',
    'mental_fortress': '永久生效，连胜连败带来的动能变化减半',
    'fragile': '每次输掉比赛后，动能下降幅度加倍',
    'ironman': '永久生效，连续多场比赛不会产生疲劳惩罚',
    'volatile': '永久生效，稳定性降低导致发挥波动增大',
    'risingstar': '仅在选手的第一个职业赛季生效',
    'rising_star': '仅在选手的第一个职业赛季生效',
    'veteran': '选手年龄达到30岁后自动生效',
    'teamleader': '永久生效，为同队其他4名选手提供状态加成',
    'team_leader': '永久生效，为同队其他4名选手提供状态加成',
  }
  return triggers[traitType] || '未知触发条件'
}

// 获取特性作用机制
const getTraitMechanism = (traitType: string): string => {
  const mechanisms: Record<string, string> = {
    'clutch': '状态值(condition)直接影响实际发挥值计算：实际发挥 = 能力值 + 状态值 + 高斯噪声',
    'slowstarter': '通过修改状态值影响发挥，慢热型选手适合打长系列赛',
    'slow_starter': '通过修改状态值影响发挥，慢热型选手适合打长系列赛',
    'faststarter': '通过修改状态值影响发挥，快枪手适合抢先手优势',
    'fast_starter': '通过修改状态值影响发挥，快枪手适合抢先手优势',
    'explosive': '稳定性影响高斯噪声的标准差(σ)，能力上限限制最高发挥值',
    'consistent': '更高的稳定性意味着更小的波动，但巅峰发挥受限',
    'comebackking': '心理素质过硬，逆境中反而能激发潜力',
    'comeback_king': '心理素质过硬，逆境中反而能激发潜力',
    'tilter': '心态不稳定，无论领先还是落后都会影响发挥',
    'mentalfortress': '动能(momentum)来自连胜连败，该特性减少心态波动',
    'mental_fortress': '动能(momentum)来自连胜连败，该特性减少心态波动',
    'fragile': '输掉比赛后心态影响更大，需要连胜来恢复状态',
    'ironman': '正常选手连续比赛会有疲劳惩罚，铁人特性免疫此效果',
    'volatile': '实际效果等同于降低稳定性，发挥更不可预测',
    'risingstar': '新人赛季额外的能力加成，模拟新人爆发现象',
    'rising_star': '新人赛季额外的能力加成，模拟新人爆发现象',
    'veteran': '老将经验带来的稳定性提升，发挥更加可靠',
    'teamleader': '领袖气质感染队友，提升整体团队表现',
    'team_leader': '领袖气质感染队友，提升整体团队表现',
  }
  return mechanisms[traitType] || '未知作用机制'
}

// ==================== 身价详情相关 ====================

// 打开身价详情弹窗
const openMarketValueDetail = () => {
  showMarketValueDialog.value = true
}

// 身价计算系数
const marketValueFactors = computed(() => {
  const ability = player.value.ability
  const age = player.value.age
  const potential = player.value.potential
  const tag = player.value.tag
  const position = player.value.position
  const region = player.value.region

  // 基础身价计算
  const multiplier = getAbilityMultiplier(ability)
  const baseValue = ability * multiplier * 10000

  // 年龄系数
  let ageFactor = 1.0
  if (age <= 19) ageFactor = 1.5
  else if (age <= 22) ageFactor = 1.3
  else if (age <= 25) ageFactor = 1.0
  else if (age <= 27) ageFactor = 0.85
  else if (age <= 29) ageFactor = 0.7
  else ageFactor = 0.5

  // 潜力系数
  const potentialDiff = potential - ability
  let potentialFactor = 1.0
  if (potentialDiff > 10) potentialFactor = 1.25
  else if (potentialDiff >= 5) potentialFactor = 1.1

  // 标签系数
  const tagFactors: Record<string, number> = { GENIUS: 1.2, NORMAL: 1.0, ORDINARY: 0.9 }
  const tagFactor = tagFactors[tag] || 1.0

  // 位置系数
  const positionFactors: Record<string, number> = { MID: 1.2, ADC: 1.15, JUG: 1.1, TOP: 1.0, SUP: 0.9 }
  const positionFactor = positionFactors[position] || 1.0

  // 赛区系数
  const regionFactors: Record<string, number> = { LPL: 1.3, LCK: 1.2, LEC: 1.0, LCS: 0.9 }
  const regionFactor = regionFactors[region] || 0.8

  // 荣誉系数（全面计算各类荣誉）
  let honorPoints = 0
  let teamHonorCount = 0
  let individualHonorCount = 0

  honors.value.forEach(h => {
    const pos = h.position

    // 团队荣誉
    if (pos === '冠军' || pos === '冠军成员') {
      honorPoints += 0.30
      teamHonorCount++
    } else if (pos === '亚军' || pos === '亚军成员') {
      honorPoints += 0.15
      teamHonorCount++
    } else if (pos === '季军' || pos === '季军成员') {
      honorPoints += 0.10
      teamHonorCount++
    } else if (pos === '殿军' || pos === '殿军成员') {
      honorPoints += 0.05
      teamHonorCount++
    }
    // 个人MVP荣誉
    else if (pos === '赛事MVP' || pos === '决赛MVP' || pos === '季后赛FMVP' || pos === '年度MVP') {
      honorPoints += 0.25
      individualHonorCount++
    } else if (pos === '常规赛MVP') {
      honorPoints += 0.20
      individualHonorCount++
    }
    // 年度Top20（从tournament_name提取排名）
    else if (pos.includes('年度Top') || h.tournament.includes('年度Top')) {
      const match = (pos + h.tournament).match(/年度Top(\d+)/)
      if (match) {
        const rank = parseInt(match[1])
        if (rank <= 5) honorPoints += 0.20
        else if (rank <= 10) honorPoints += 0.15
        else honorPoints += 0.10
        individualHonorCount++
      }
    }
    // 年度最佳位置
    else if (pos.includes('年度最佳') && !pos.includes('新秀')) {
      honorPoints += 0.15
      individualHonorCount++
    }
    // 年度最佳新秀
    else if (pos.includes('年度最佳新秀') || pos === '年度新秀') {
      honorPoints += 0.10
      individualHonorCount++
    }
    // 常规赛第一
    else if (pos === '常规赛第一') {
      honorPoints += 0.08
      teamHonorCount++
    }
  })

  const honorFactor = 1.0 + honorPoints

  // 计算最终身价 = 基础身价 × 所有因子
  const calculatedValue = baseValue * ageFactor * potentialFactor * tagFactor * positionFactor * regionFactor * Math.min(honorFactor, 4.0)

  return {
    baseValue,
    ageFactor,
    potentialFactor,
    tagFactor,
    positionFactor,
    regionFactor,
    honorFactor: Math.min(honorFactor, 4.0), // 提高上限到4.0
    teamHonorCount,
    individualHonorCount,
    calculatedValue: Math.round(calculatedValue), // 最终计算的身价
  }
})

// 获取能力值对应的倍率
const getAbilityMultiplier = (ability: number): number => {
  if (ability >= 95) return 50
  if (ability >= 90) return 35
  if (ability >= 85) return 20
  if (ability >= 80) return 12
  if (ability >= 75) return 7
  if (ability >= 70) return 4
  if (ability >= 60) return 2
  return 1
}

// 获取荣誉描述
const getHonorDescription = (factor: number, teamCount?: number, individualCount?: number): string => {
  const total = (teamCount || 0) + (individualCount || 0)
  if (total === 0) return '无荣誉加成'

  const parts: string[] = []
  if (teamCount && teamCount > 0) parts.push(`${teamCount}项团队`)
  if (individualCount && individualCount > 0) parts.push(`${individualCount}项个人`)

  if (factor >= 3.0) return `传奇(${parts.join('+')})`
  if (factor >= 2.0) return `顶级(${parts.join('+')})`
  if (factor >= 1.5) return `优秀(${parts.join('+')})`
  if (factor > 1.0) return parts.join('+')
  return '无荣誉加成'
}

// ==================== 五维雷达图 ====================

// 计算五维数据（将原始数据转换为0-100的分数）
const computeRadarData = computed(() => {
  if (!playerStats.value) return null

  const stats = playerStats.value

  // 影响力：avg_impact 通常在 -10 到 20 之间，转换为 0-100
  const impactScore = Math.min(100, Math.max(0, (stats.avg_impact + 10) * 3.33))

  // 发挥：avg_performance 通常在 60-100 之间
  const performanceScore = Math.min(100, Math.max(0, (stats.avg_performance - 50) * 2))

  // 稳定性：consistency_score 本身就是 0-100
  const consistencyScore = stats.consistency_score

  // 巅峰：best_performance 通常在 70-110
  const peakScore = Math.min(100, Math.max(0, (stats.best_performance - 60) * 2.5))

  // 荣誉：champion_bonus 国际赛*3+赛区冠军，最高算15分满
  const honorScore = Math.min(100, stats.champion_bonus * 6.67)

  return {
    impact: Math.round(impactScore),
    performance: Math.round(performanceScore),
    consistency: Math.round(consistencyScore),
    peak: Math.round(peakScore),
    honor: Math.round(honorScore)
  }
})

// 初始化雷达图
const initRadarChart = () => {
  if (!radarChartRef.value || !playerStats.value) return

  // 如果已有实例，先销毁
  if (radarChart) {
    radarChart.dispose()
  }

  radarChart = echarts.init(radarChartRef.value)

  const data = computeRadarData.value
  if (!data) return

  const option: echarts.EChartsOption = {
    radar: {
      indicator: [
        { name: '影响力', max: 100 },
        { name: '发挥', max: 100 },
        { name: '稳定性', max: 100 },
        { name: '巅峰', max: 100 },
        { name: '荣誉', max: 100 }
      ],
      shape: 'polygon',
      splitNumber: 4,
      axisName: {
        color: '#333333',
        fontSize: 12,
        fontWeight: 500
      },
      splitLine: {
        lineStyle: {
          color: 'rgba(128, 128, 128, 0.2)'
        }
      },
      splitArea: {
        areaStyle: {
          color: ['rgba(59, 130, 246, 0.02)', 'rgba(59, 130, 246, 0.04)', 'rgba(59, 130, 246, 0.06)', 'rgba(59, 130, 246, 0.08)']
        }
      },
      axisLine: {
        lineStyle: {
          color: 'rgba(128, 128, 128, 0.3)'
        }
      }
    },
    series: [{
      type: 'radar',
      data: [{
        value: [data.impact, data.performance, data.consistency, data.peak, data.honor],
        name: player.value.gameId,
        areaStyle: {
          color: {
            type: 'radial',
            x: 0.5,
            y: 0.5,
            r: 0.5,
            colorStops: [
              { offset: 0, color: 'rgba(59, 130, 246, 0.1)' },
              { offset: 1, color: 'rgba(59, 130, 246, 0.4)' }
            ]
          }
        },
        lineStyle: {
          color: '#3b82f6',
          width: 2
        },
        itemStyle: {
          color: '#3b82f6',
          borderColor: '#fff',
          borderWidth: 2
        },
        symbol: 'circle',
        symbolSize: 8
      }]
    }]
  }

  radarChart.setOption(option)

  // 监听窗口大小变化
  window.addEventListener('resize', () => {
    radarChart?.resize()
  })
}

// 监听 playerStats 变化，重新渲染雷达图
watch(playerStats, async () => {
  await nextTick()
  initRadarChart()
})
</script>

<style scoped>
.player-detail-view {
  padding: 0;
}

.back-link {
  margin-bottom: 16px;
}

.back-link .el-button {
  color: var(--text-secondary);
  font-size: 14px;
}

.back-link .el-button:hover {
  color: var(--primary-color);
}

/* 选手资料卡片 */
.profile-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.profile-content {
  display: flex;
  align-items: flex-start;
  gap: 32px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.player-avatar {
  width: 120px;
  height: 120px;
  border-radius: 16px;
  font-size: 24px;
}

.talent-tag {
  font-size: 14px;
}

.info-section {
  flex: 1;
}

.player-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 8px;
}

.player-name {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.player-tags {
  display: flex;
  gap: 8px;
}

.player-real-name {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

.player-team {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  color: var(--text-primary);
}

.stats-section {
  display: flex;
  align-items: center;
  gap: 24px;
}

.stat-number-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stat-number-display .stat-value {
  font-size: 32px;
  font-weight: 700;
  line-height: 1;
}

.stat-number-display .stat-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.stat-text {
  text-align: center;
}

.age-display {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.age-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.age-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* 详情卡片 */
.detail-row {
  margin-bottom: 20px;
}

.detail-card {
  border-radius: 12px;
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.count-badge {
  font-size: 14px;
  color: var(--text-tertiary);
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-light);
}

.info-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.info-label {
  font-size: 14px;
  color: var(--text-tertiary);
}

.info-value {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
}

.info-value.highlight {
  color: var(--primary-color);
}

.info-value.money {
  color: #f59e0b;
}

.info-value.success {
  color: #22c55e;
}

.info-value.gold {
  color: #fbbf24;
  font-weight: 700;
}

.info-value.purple {
  color: #8b5cf6;
}

/* 天赋说明 */
.talent-alert {
  margin-bottom: 20px;
  border-radius: 8px;
}

/* 荣誉卡片 */
.honors-card {
  border-radius: 12px;
  margin-bottom: 20px;
}

.empty-icon {
  font-size: 64px;
}

.honor-card {
  margin-bottom: 0;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.honor-card:hover {
  transform: translateX(4px);
}

.honor-card.champion {
  border-left: 4px solid #fbbf24;
  background: linear-gradient(135deg, #fffbeb 0%, #ffffff 100%);
}

.honor-card.runner-up {
  border-left: 4px solid #9ca3af;
}

.honor-card.third-place {
  border-left: 4px solid #f97316;
}

.honor-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.honor-icon {
  font-size: 32px;
}

.honor-info {
  flex: 1;
}

.honor-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

/* 赛季历史 */
.history-card {
  border-radius: 12px;
}

.history-table {
  border-radius: 8px;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ability-value {
  font-weight: 700;
}

.potential-value {
  color: #8b5cf6;
  font-weight: 600;
}

.text-gray {
  color: var(--text-placeholder);
}

:deep(.el-timeline-item__timestamp) {
  font-weight: 600;
  font-size: 14px;
  color: var(--primary-color);
}

/* 特性与状态区 */
.traits-condition-row {
  margin-bottom: 20px;
}

.traits-card,
.condition-card {
  border-radius: 12px;
  height: 100%;
}

.header-icon {
  font-size: 18px;
  margin-right: 4px;
}

/* 特性网格 */
.traits-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.trait-item {
  padding: 12px;
  border-radius: 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-light);
  transition: all 0.3s ease;
}

.trait-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

/* 特性稀有度样式 */
.trait-item.rarity-1 {
  border-left: 3px solid #9ca3af;
}

.trait-item.rarity-2 {
  border-left: 3px solid #22c55e;
}

.trait-item.rarity-3 {
  border-left: 3px solid #3b82f6;
}

.trait-item.rarity-4 {
  border-left: 3px solid #8b5cf6;
}

.trait-item.rarity-5 {
  border-left: 3px solid #f59e0b;
  background: linear-gradient(135deg, #fffbeb 0%, var(--bg-secondary) 100%);
}

.trait-item.negative {
  border-left-color: #ef4444;
  background: linear-gradient(135deg, #fef2f2 0%, var(--bg-secondary) 100%);
}

.trait-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.trait-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.trait-rarity {
  font-size: 12px;
  color: #f59e0b;
}

.trait-description {
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.4;
}

/* 状态面板 */
.condition-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.condition-display {
  text-align: center;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.condition-value {
  font-size: 48px;
  font-weight: 700;
  line-height: 1;
}

.condition-value.excellent {
  color: #22c55e;
}

.condition-value.good {
  color: #3b82f6;
}

.condition-value.normal {
  color: var(--text-primary);
}

.condition-value.poor {
  color: #f59e0b;
}

.condition-value.terrible {
  color: #ef4444;
}

.condition-label {
  font-size: 14px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.condition-range {
  font-size: 12px;
  color: var(--text-placeholder);
  margin-top: 8px;
}

/* 状态因子 */
.condition-factors {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.factor-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.factor-label {
  font-size: 13px;
  color: var(--text-tertiary);
}

.factor-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.factor-value.hot {
  color: #ef4444;
}

.factor-value.warming {
  color: #f59e0b;
}

.factor-value.neutral {
  color: var(--text-secondary);
}

.factor-value.cooling {
  color: #3b82f6;
}

.factor-value.cold {
  color: #6366f1;
}

/* ==================== 五维雷达图样式 ==================== */
.radar-card {
  min-height: 400px;
}

.radar-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.radar-chart {
  width: 100%;
  height: 220px;
}

.radar-stats {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 12px;
}

.stat-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-row .stat-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 80px;
}

.stat-icon {
  font-size: 14px;
}

.stat-bar-wrapper {
  flex: 1;
  height: 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow: hidden;
}

.stat-bar {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #60a5fa);
  border-radius: 4px;
  transition: width 0.5s ease;
}

.stat-bar.performance {
  background: linear-gradient(90deg, #22c55e, #4ade80);
}

.stat-bar.consistency {
  background: linear-gradient(90deg, #8b5cf6, #a78bfa);
}

.stat-bar.peak {
  background: linear-gradient(90deg, #f59e0b, #fbbf24);
}

.stat-bar.honor {
  background: linear-gradient(90deg, #ef4444, #f87171);
}

.stat-row .stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  min-width: 32px;
  text-align: right;
}

/* 可点击样式 */
.clickable {
  cursor: pointer;
}

.info-row.clickable:hover {
  background: var(--bg-secondary);
  margin: 0 -12px;
  padding: 12px 12px;
  border-radius: 8px;
}

.info-row .click-icon {
  font-size: 12px;
  margin-left: 4px;
  opacity: 0.5;
}

.info-row.clickable:hover .click-icon {
  opacity: 1;
}

/* 特性点击提示 */
.trait-item.clickable {
  cursor: pointer;
}

.trait-click-hint {
  font-size: 11px;
  color: var(--text-placeholder);
  margin-top: 8px;
  text-align: right;
  opacity: 0;
  transition: opacity 0.2s;
}

.trait-item:hover .trait-click-hint {
  opacity: 1;
}

/* ==================== 特性详情弹窗 ==================== */
.trait-detail-dialog :deep(.el-dialog__header) {
  display: none;
}

.trait-detail-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.trait-detail-content {
  padding: 0;
}

/* 特性卡片头部 */
.trait-card-header {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px 24px;
  border-radius: 12px 12px 0 0;
  text-align: center;
  background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
}

.trait-card-header.rarity-1 {
  background: linear-gradient(180deg, #6b7280 0%, #374151 100%);
}
.trait-card-header.rarity-2 {
  background: linear-gradient(180deg, #22c55e 0%, #16a34a 100%);
}
.trait-card-header.rarity-3 {
  background: linear-gradient(180deg, #3b82f6 0%, #2563eb 100%);
}
.trait-card-header.rarity-4 {
  background: linear-gradient(180deg, #8b5cf6 0%, #7c3aed 100%);
}
.trait-card-header.rarity-5 {
  background: linear-gradient(180deg, #f59e0b 0%, #d97706 100%);
}
.trait-card-header.negative {
  background: linear-gradient(180deg, #ef4444 0%, #dc2626 100%);
}

.trait-card-close {
  position: absolute;
  top: 12px;
  right: 16px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  border-radius: 50%;
  transition: all 0.2s;
}
.trait-card-close:hover {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.trait-card-icon {
  font-size: 56px;
  line-height: 1;
  margin-bottom: 12px;
  filter: drop-shadow(0 4px 8px rgba(0,0,0,0.3));
}

.trait-card-name {
  font-size: 26px;
  font-weight: 700;
  color: white;
  text-shadow: 0 2px 4px rgba(0,0,0,0.3);
  margin-bottom: 8px;
}

.trait-card-stars {
  font-size: 18px;
  margin-bottom: 12px;
  letter-spacing: 2px;
}
.trait-card-stars .star-filled {
  color: #fde047;
  text-shadow: 0 0 8px rgba(253, 224, 71, 0.6);
}
.trait-card-stars .star-empty {
  color: rgba(255, 255, 255, 0.3);
}

.trait-card-type {
  font-size: 13px;
  font-weight: 500;
}
.trait-card-type .type-positive {
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.2);
  padding: 4px 12px;
  border-radius: 12px;
}
.trait-card-type .type-negative {
  color: white;
  background: rgba(0, 0, 0, 0.2);
  padding: 4px 12px;
  border-radius: 12px;
}

/* 信息卡片 */
.trait-info-card {
  margin: 16px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
  border: 1px solid var(--border-light);
}

.trait-info-card.small {
  margin: 0;
}

.info-card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-light);
}

.info-card-title .icon {
  font-size: 16px;
}

.info-card-content {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.6;
}

.info-card-content.description {
  font-size: 15px;
  line-height: 1.8;
}

/* 效果表格 */
.effects-table {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.effect-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.effect-label {
  font-size: 14px;
  color: var(--text-secondary);
}

.effect-val {
  font-size: 15px;
  font-weight: 700;
}

.effect-val.val-positive {
  color: #22c55e;
}

.effect-val.val-negative {
  color: #ef4444;
}

/* 信息网格 */
.trait-info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin: 16px;
}

/* ==================== 身价详情弹窗 ==================== */
.market-value-content {
  padding: 0;
}

.market-value-header {
  text-align: center;
  padding: 24px;
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  border-radius: 12px;
  margin-bottom: 24px;
}

.current-value {
  font-size: 36px;
  font-weight: 700;
  color: white;
  line-height: 1;
}

.value-label {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
  margin-top: 8px;
}

.formula-section,
.factors-section {
  margin-bottom: 24px;
}

.formula-section .section-title,
.factors-section .section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.formula-display {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  font-family: monospace;
}

.factors-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.factor-item {
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-light);
}

.factor-item.highlight {
  background: linear-gradient(135deg, #fef3c7 0%, var(--bg-secondary) 100%);
  border-color: #f59e0b;
}

.factor-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 4px;
}

.factor-value {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.factor-detail {
  font-size: 11px;
  color: var(--text-placeholder);
}

/* 身价变化记录 */
.changes-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border-light);
}

.changes-section .section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.changes-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 200px;
  overflow-y: auto;
}

.change-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-light);
}

.change-reason {
  flex: 0 0 auto;
}

.change-values {
  flex: 1;
  text-align: center;
  font-size: 13px;
}

.change-values .old-value {
  color: var(--text-tertiary);
}

.change-values .arrow {
  margin: 0 8px;
  color: var(--text-placeholder);
}

.change-values .new-value {
  color: var(--text-primary);
  font-weight: 600;
}

.change-percent {
  font-size: 14px;
  font-weight: 700;
  min-width: 60px;
  text-align: right;
}

.change-percent.positive {
  color: #10b981;
}

.change-percent.negative {
  color: #ef4444;
}

.no-changes {
  padding: 20px;
  text-align: center;
  color: var(--text-placeholder);
  font-size: 13px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

/* ==================== 特性图鉴弹窗 ==================== */
.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-actions .el-button {
  color: var(--text-secondary);
}

.header-actions .el-button:hover {
  color: var(--primary-color);
}

.traits-guide-content {
  max-height: 60vh;
  overflow-y: auto;
}

.rarity-legend {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 20px;
}

.legend-title {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.legend-item {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}

.legend-item.rarity-1 {
  color: #6b7280;
  background: rgba(107, 114, 128, 0.1);
}

.legend-item.rarity-2 {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
}

.legend-item.rarity-3 {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.legend-item.rarity-4 {
  color: #8b5cf6;
  background: rgba(139, 92, 246, 0.1);
}

.legend-item.rarity-5 {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
}

.traits-category {
  margin-bottom: 24px;
}

.traits-category.negative {
  margin-bottom: 0;
}

.category-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
  padding-left: 8px;
  border-left: 3px solid #22c55e;
}

.traits-category.negative .category-title {
  border-left-color: #ef4444;
}

.traits-grid-guide {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.trait-guide-item {
  padding: 12px;
  border-radius: 10px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-light);
  transition: all 0.2s ease;
}

.trait-guide-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.trait-guide-item.rarity-1 {
  border-left: 3px solid #6b7280;
}

.trait-guide-item.rarity-2 {
  border-left: 3px solid #22c55e;
}

.trait-guide-item.rarity-3 {
  border-left: 3px solid #3b82f6;
}

.trait-guide-item.rarity-4 {
  border-left: 3px solid #8b5cf6;
}

.trait-guide-item.rarity-5 {
  border-left: 3px solid #f59e0b;
  background: linear-gradient(135deg, #fffbeb 0%, var(--bg-secondary) 100%);
}

.trait-guide-item.negative {
  border-left-color: #ef4444;
  background: linear-gradient(135deg, #fef2f2 0%, var(--bg-secondary) 100%);
}

.trait-guide-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.trait-guide-icon {
  font-size: 20px;
}

.trait-guide-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.trait-guide-stars {
  font-size: 11px;
  color: #f59e0b;
}

.trait-guide-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
  line-height: 1.4;
}

.trait-guide-effect {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.effect-tag {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 500;
}

.effect-tag.positive {
  color: #16a34a;
  background: rgba(34, 197, 94, 0.1);
}

.effect-tag.negative {
  color: #dc2626;
  background: rgba(239, 68, 68, 0.1);
}

/* ==================== 新版身价详情弹窗 ==================== */
.market-value-dialog :deep(.el-dialog__header) {
  display: none;
}

.market-value-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.mv-content {
  padding: 0;
}

/* 顶部大卡片 */
.mv-hero {
  position: relative;
  padding: 32px 24px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  border-radius: 12px 12px 0 0;
  text-align: center;
  overflow: hidden;
}

.mv-hero-bg {
  position: absolute;
  top: -50%;
  right: -20%;
  width: 200px;
  height: 200px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 50%;
}

.mv-hero-content {
  position: relative;
  z-index: 1;
}

.mv-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  margin-bottom: 8px;
}

.mv-amount {
  font-size: 42px;
  font-weight: 800;
  color: white;
  line-height: 1;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  margin-bottom: 8px;
}

.mv-player {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

/* 系数分解区 */
.mv-factors {
  padding: 20px;
}

.mv-section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.mv-section-title::before {
  content: '';
  width: 3px;
  height: 14px;
  background: var(--primary-color);
  border-radius: 2px;
}

.mv-factor-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mv-factor-row {
  display: flex;
  align-items: center;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  transition: all 0.2s ease;
}

.mv-factor-row:hover {
  background: var(--bg-tertiary);
}

.mv-factor-row.highlight {
  background: linear-gradient(135deg, #fef3c7 0%, var(--bg-secondary) 100%);
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.mv-factor-icon {
  font-size: 18px;
  width: 28px;
  flex-shrink: 0;
}

.mv-factor-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  min-width: 60px;
}

.mv-factor-detail {
  flex: 1;
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: right;
  margin-right: 12px;
}

.mv-factor-val {
  font-size: 14px;
  font-weight: 700;
  min-width: 55px;
  text-align: right;
}

.mv-factor-val.primary {
  color: var(--primary-color);
}

.mv-factor-val.positive {
  color: #10b981;
}

.mv-factor-val.negative {
  color: #ef4444;
}

/* 变化记录区 */
.mv-history {
  padding: 0 20px 20px;
}

.mv-history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 180px;
  overflow-y: auto;
}

.mv-history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-light);
}

.mv-history-left {
  flex: 1;
}

.mv-history-right {
  flex-shrink: 0;
}

.mv-history-change {
  font-size: 15px;
  font-weight: 700;
}

.mv-history-change.up {
  color: #10b981;
}

.mv-history-change.down {
  color: #ef4444;
}

.mv-no-history {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 24px;
  background: var(--bg-secondary);
  border-radius: 8px;
  color: var(--text-placeholder);
  font-size: 13px;
}

.mv-no-icon {
  font-size: 20px;
}
</style>
