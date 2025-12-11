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
            <div class="info-row">
              <span class="info-label">身价</span>
              <span class="info-value success">{{ formatMoney(player.marketValue) }}</span>
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
          <div class="empty-icon">🏆</div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import {
  ArrowLeft,
  Document,
  TrendCharts,
  Trophy,
  Clock,
} from '@element-plus/icons-vue'

const route = useRoute()
const playerId = route.params.id

// 所有选手数据
const allPlayers = [
  // T1
  { id: 1, gameId: 'Faker', realName: '李相赫', nationality: '韩国', team: 'T1', region: 'LCK', position: 'MID', age: 28, ability: 95, potential: 96, stability: 85, tag: 'GENIUS', salary: 3500000, marketValue: 16740000, contractEnd: 'S4', joinSeason: 'S1' },
  { id: 3, gameId: 'Zeus', realName: '崔宇济', nationality: '韩国', team: 'T1', region: 'LCK', position: 'TOP', age: 21, ability: 88, potential: 94, stability: 78, tag: 'GENIUS', salary: 1500000, marketValue: 8500000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 5, gameId: 'Keria', realName: '柳民锡', nationality: '韩国', team: 'T1', region: 'LCK', position: 'SUP', age: 22, ability: 89, potential: 93, stability: 82, tag: 'GENIUS', salary: 1400000, marketValue: 7800000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 11, gameId: 'Gumayusi', realName: '李民赫', nationality: '韩国', team: 'T1', region: 'LCK', position: 'ADC', age: 22, ability: 88, potential: 93, stability: 76, tag: 'GENIUS', salary: 1300000, marketValue: 7200000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 12, gameId: 'Oner', realName: '文贤俊', nationality: '韩国', team: 'T1', region: 'LCK', position: 'JUG', age: 22, ability: 86, potential: 91, stability: 80, tag: 'NORMAL', salary: 1200000, marketValue: 6500000, contractEnd: 'S3', joinSeason: 'S1' },
  // Gen.G
  { id: 2, gameId: 'Chovy', realName: '郑智勋', nationality: '韩国', team: 'Gen.G', region: 'LCK', position: 'MID', age: 24, ability: 93, potential: 95, stability: 88, tag: 'GENIUS', salary: 2500000, marketValue: 12500000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 6, gameId: 'Canyon', realName: '金建富', nationality: '韩国', team: 'Gen.G', region: 'LCK', position: 'JUG', age: 23, ability: 91, potential: 92, stability: 85, tag: 'GENIUS', salary: 2000000, marketValue: 10000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 16, gameId: 'Peyz', realName: '金善旻', nationality: '韩国', team: 'Gen.G', region: 'LCK', position: 'ADC', age: 19, ability: 82, potential: 92, stability: 72, tag: 'GENIUS', salary: 800000, marketValue: 5000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 17, gameId: 'Doran', realName: '崔铉俊', nationality: '韩国', team: 'Gen.G', region: 'LCK', position: 'TOP', age: 24, ability: 84, potential: 86, stability: 83, tag: 'NORMAL', salary: 1000000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 18, gameId: 'Lehends', realName: '孙时宇', nationality: '韩国', team: 'Gen.G', region: 'LCK', position: 'SUP', age: 26, ability: 85, potential: 86, stability: 88, tag: 'NORMAL', salary: 1100000, marketValue: 5800000, contractEnd: 'S2', joinSeason: 'S1' },
  // JDG
  { id: 4, gameId: 'Ruler', realName: '朴宰赫', nationality: '韩国', team: 'JDG', region: 'LPL', position: 'ADC', age: 26, ability: 90, potential: 91, stability: 90, tag: 'GENIUS', salary: 2200000, marketValue: 11000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 14, gameId: 'Kanavi', realName: '徐镇赫', nationality: '韩国', team: 'JDG', region: 'LPL', position: 'JUG', age: 24, ability: 88, potential: 89, stability: 82, tag: 'NORMAL', salary: 1800000, marketValue: 9000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 9, gameId: '369', realName: '白家浩', nationality: '中国', team: 'JDG', region: 'LPL', position: 'TOP', age: 23, ability: 87, potential: 90, stability: 75, tag: 'GENIUS', salary: 1600000, marketValue: 8500000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 21, gameId: 'Yagao', realName: '曾奇', nationality: '中国', team: 'JDG', region: 'LPL', position: 'MID', age: 25, ability: 84, potential: 85, stability: 80, tag: 'NORMAL', salary: 1400000, marketValue: 7000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 22, gameId: 'Missing', realName: '刘明浩', nationality: '中国', team: 'JDG', region: 'LPL', position: 'SUP', age: 24, ability: 85, potential: 87, stability: 82, tag: 'NORMAL', salary: 1200000, marketValue: 6000000, contractEnd: 'S2', joinSeason: 'S1' },
  // BLG
  { id: 7, gameId: 'Knight', realName: '卓定', nationality: '中国', team: 'BLG', region: 'LPL', position: 'MID', age: 24, ability: 89, potential: 92, stability: 78, tag: 'GENIUS', salary: 2000000, marketValue: 10000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 13, gameId: 'Elk', realName: '马朝阳', nationality: '中国', team: 'BLG', region: 'LPL', position: 'ADC', age: 22, ability: 86, potential: 90, stability: 76, tag: 'GENIUS', salary: 1300000, marketValue: 7500000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 19, gameId: 'Bin', realName: '陈泽彬', nationality: '中国', team: 'BLG', region: 'LPL', position: 'TOP', age: 22, ability: 87, potential: 91, stability: 70, tag: 'GENIUS', salary: 1400000, marketValue: 8000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 20, gameId: 'ON', realName: '李载元', nationality: '韩国', team: 'BLG', region: 'LPL', position: 'SUP', age: 21, ability: 82, potential: 88, stability: 75, tag: 'NORMAL', salary: 900000, marketValue: 5000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 23, gameId: 'XUN', realName: '彭立勋', nationality: '中国', team: 'BLG', region: 'LPL', position: 'JUG', age: 21, ability: 85, potential: 88, stability: 73, tag: 'NORMAL', salary: 1000000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  // TES
  { id: 24, gameId: 'Wayward', realName: '陈闵', nationality: '中国', team: 'TES', region: 'LPL', position: 'TOP', age: 22, ability: 84, potential: 89, stability: 76, tag: 'NORMAL', salary: 1100000, marketValue: 6000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 25, gameId: 'Tian', realName: '高天亮', nationality: '中国', team: 'TES', region: 'LPL', position: 'JUG', age: 24, ability: 85, potential: 87, stability: 75, tag: 'NORMAL', salary: 1300000, marketValue: 6500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 26, gameId: 'Creme', realName: '季明锴', nationality: '中国', team: 'TES', region: 'LPL', position: 'MID', age: 20, ability: 83, potential: 90, stability: 72, tag: 'GENIUS', salary: 1000000, marketValue: 6000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 27, gameId: 'JackeyLove', realName: '喻文波', nationality: '中国', team: 'TES', region: 'LPL', position: 'ADC', age: 24, ability: 88, potential: 89, stability: 74, tag: 'GENIUS', salary: 2000000, marketValue: 9500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 28, gameId: 'Mark', realName: '张宝蓝', nationality: '中国', team: 'TES', region: 'LPL', position: 'SUP', age: 23, ability: 83, potential: 86, stability: 78, tag: 'NORMAL', salary: 900000, marketValue: 5000000, contractEnd: 'S2', joinSeason: 'S1' },
  // WBG
  { id: 29, gameId: 'TheShy', realName: '姜承录', nationality: '韩国', team: 'WBG', region: 'LPL', position: 'TOP', age: 25, ability: 86, potential: 87, stability: 68, tag: 'GENIUS', salary: 1800000, marketValue: 8000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 30, gameId: 'Weiwei', realName: '魏伟', nationality: '中国', team: 'WBG', region: 'LPL', position: 'JUG', age: 23, ability: 84, potential: 87, stability: 76, tag: 'NORMAL', salary: 1100000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 31, gameId: 'Xiaohu', realName: '李元浩', nationality: '中国', team: 'WBG', region: 'LPL', position: 'MID', age: 27, ability: 86, potential: 87, stability: 85, tag: 'NORMAL', salary: 1600000, marketValue: 7500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 32, gameId: 'Light', realName: '王光宇', nationality: '中国', team: 'WBG', region: 'LPL', position: 'ADC', age: 22, ability: 84, potential: 88, stability: 77, tag: 'NORMAL', salary: 1000000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 33, gameId: 'Crisp', realName: '刘浩', nationality: '中国', team: 'WBG', region: 'LPL', position: 'SUP', age: 26, ability: 85, potential: 86, stability: 83, tag: 'NORMAL', salary: 1200000, marketValue: 6000000, contractEnd: 'S2', joinSeason: 'S1' },
  // HLE
  { id: 34, gameId: 'Doran2', realName: '金东河', nationality: '韩国', team: 'HLE', region: 'LCK', position: 'TOP', age: 23, ability: 83, potential: 87, stability: 79, tag: 'NORMAL', salary: 900000, marketValue: 5000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 35, gameId: 'Peanut', realName: '韩王浩', nationality: '韩国', team: 'HLE', region: 'LCK', position: 'JUG', age: 26, ability: 85, potential: 86, stability: 82, tag: 'NORMAL', salary: 1200000, marketValue: 6000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 36, gameId: 'Zeka', realName: '金建权', nationality: '韩国', team: 'HLE', region: 'LCK', position: 'MID', age: 21, ability: 86, potential: 91, stability: 75, tag: 'GENIUS', salary: 1300000, marketValue: 7500000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 37, gameId: 'Viper', realName: '朴道贤', nationality: '韩国', team: 'HLE', region: 'LCK', position: 'ADC', age: 24, ability: 89, potential: 90, stability: 84, tag: 'GENIUS', salary: 1800000, marketValue: 9000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 38, gameId: 'Delight', realName: '柳焕中', nationality: '韩国', team: 'HLE', region: 'LCK', position: 'SUP', age: 21, ability: 82, potential: 88, stability: 76, tag: 'NORMAL', salary: 800000, marketValue: 4500000, contractEnd: 'S2', joinSeason: 'S1' },
  // DK
  { id: 39, gameId: 'Kingen', realName: '黄成勋', nationality: '韩国', team: 'DK', region: 'LCK', position: 'TOP', age: 24, ability: 84, potential: 86, stability: 80, tag: 'NORMAL', salary: 1000000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 40, gameId: 'Lucid', realName: '申东旭', nationality: '韩国', team: 'DK', region: 'LCK', position: 'JUG', age: 20, ability: 83, potential: 90, stability: 74, tag: 'GENIUS', salary: 900000, marketValue: 5500000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 41, gameId: 'ShowMaker', realName: '许秀', nationality: '韩国', team: 'DK', region: 'LCK', position: 'MID', age: 24, ability: 90, potential: 92, stability: 82, tag: 'GENIUS', salary: 2000000, marketValue: 10000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 42, gameId: 'Aiming', realName: '金河霖', nationality: '韩国', team: 'DK', region: 'LCK', position: 'ADC', age: 24, ability: 86, potential: 88, stability: 81, tag: 'NORMAL', salary: 1200000, marketValue: 6500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 43, gameId: 'Kellin', realName: '金炯奎', nationality: '韩国', team: 'DK', region: 'LCK', position: 'SUP', age: 24, ability: 82, potential: 85, stability: 80, tag: 'NORMAL', salary: 800000, marketValue: 4500000, contractEnd: 'S2', joinSeason: 'S1' },
  // G2
  { id: 44, gameId: 'BrokenBlade', realName: '塞尔坎·切利克', nationality: '德国', team: 'G2', region: 'LEC', position: 'TOP', age: 24, ability: 84, potential: 87, stability: 78, tag: 'NORMAL', salary: 1000000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 45, gameId: 'Yike', realName: '马丁·桑德伯格', nationality: '瑞典', team: 'G2', region: 'LEC', position: 'JUG', age: 22, ability: 82, potential: 88, stability: 75, tag: 'NORMAL', salary: 800000, marketValue: 4500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 46, gameId: 'Caps', realName: '拉斯穆斯·温特', nationality: '丹麦', team: 'G2', region: 'LEC', position: 'MID', age: 25, ability: 88, potential: 90, stability: 80, tag: 'GENIUS', salary: 1800000, marketValue: 9000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 47, gameId: 'Hans Sama', realName: '史蒂文·利文', nationality: '法国', team: 'G2', region: 'LEC', position: 'ADC', age: 24, ability: 85, potential: 87, stability: 79, tag: 'NORMAL', salary: 1100000, marketValue: 6000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 48, gameId: 'Mikyx', realName: '米哈埃尔·梅赫雷', nationality: '斯洛文尼亚', team: 'G2', region: 'LEC', position: 'SUP', age: 26, ability: 84, potential: 85, stability: 82, tag: 'NORMAL', salary: 1000000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  // FNC
  { id: 49, gameId: 'Oscarinin', realName: '奥斯卡·穆尼奥斯', nationality: '西班牙', team: 'FNC', region: 'LEC', position: 'TOP', age: 22, ability: 82, potential: 88, stability: 76, tag: 'NORMAL', salary: 800000, marketValue: 4500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 50, gameId: 'Razork', realName: '伊凡·迪亚兹', nationality: '西班牙', team: 'FNC', region: 'LEC', position: 'JUG', age: 24, ability: 83, potential: 86, stability: 78, tag: 'NORMAL', salary: 900000, marketValue: 5000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 51, gameId: 'Humanoid', realName: '马雷克·布拉泽克', nationality: '捷克', team: 'FNC', region: 'LEC', position: 'MID', age: 25, ability: 86, potential: 88, stability: 80, tag: 'NORMAL', salary: 1300000, marketValue: 7000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 52, gameId: 'Noah', realName: '尼科拉·奥斯曼', nationality: '挪威', team: 'FNC', region: 'LEC', position: 'ADC', age: 21, ability: 82, potential: 89, stability: 74, tag: 'GENIUS', salary: 800000, marketValue: 5000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 53, gameId: 'Jun', realName: '李俊燮', nationality: '韩国', team: 'FNC', region: 'LEC', position: 'SUP', age: 22, ability: 81, potential: 86, stability: 77, tag: 'NORMAL', salary: 700000, marketValue: 4000000, contractEnd: 'S2', joinSeason: 'S1' },
  // C9
  { id: 54, gameId: 'Thanatos', realName: '崔俊锡', nationality: '韩国', team: 'C9', region: 'LCS', position: 'TOP', age: 21, ability: 82, potential: 88, stability: 75, tag: 'NORMAL', salary: 800000, marketValue: 4500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 55, gameId: 'Blaber', realName: '罗伯特·黄', nationality: '美国', team: 'C9', region: 'LCS', position: 'JUG', age: 24, ability: 84, potential: 86, stability: 76, tag: 'NORMAL', salary: 1100000, marketValue: 6000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 56, gameId: 'Jojopyun', realName: '约瑟夫·黄', nationality: '加拿大', team: 'C9', region: 'LCS', position: 'MID', age: 20, ability: 83, potential: 90, stability: 73, tag: 'GENIUS', salary: 1000000, marketValue: 6000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 57, gameId: 'Berserker', realName: '金炯宇', nationality: '韩国', team: 'C9', region: 'LCS', position: 'ADC', age: 21, ability: 86, potential: 91, stability: 78, tag: 'GENIUS', salary: 1400000, marketValue: 8000000, contractEnd: 'S3', joinSeason: 'S1' },
  { id: 58, gameId: 'Vulcan', realName: '菲利普·拉贾诺维奇', nationality: '加拿大', team: 'C9', region: 'LCS', position: 'SUP', age: 25, ability: 83, potential: 85, stability: 82, tag: 'NORMAL', salary: 900000, marketValue: 5000000, contractEnd: 'S2', joinSeason: 'S1' },
  // TL
  { id: 59, gameId: 'Impact', realName: '郑然泳', nationality: '韩国', team: 'TL', region: 'LCS', position: 'TOP', age: 29, ability: 83, potential: 84, stability: 88, tag: 'NORMAL', salary: 1200000, marketValue: 5500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 60, gameId: 'UmTi', realName: '文艺俊', nationality: '韩国', team: 'TL', region: 'LCS', position: 'JUG', age: 24, ability: 82, potential: 86, stability: 79, tag: 'NORMAL', salary: 900000, marketValue: 4800000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 61, gameId: 'APA', realName: '凯恩·福尔曼', nationality: '美国', team: 'TL', region: 'LCS', position: 'MID', age: 21, ability: 82, potential: 88, stability: 74, tag: 'NORMAL', salary: 800000, marketValue: 4500000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 62, gameId: 'Yeon', realName: '李显俊', nationality: '韩国', team: 'TL', region: 'LCS', position: 'ADC', age: 22, ability: 83, potential: 88, stability: 76, tag: 'NORMAL', salary: 900000, marketValue: 5000000, contractEnd: 'S2', joinSeason: 'S1' },
  { id: 63, gameId: 'CoreJJ', realName: '赵勇仁', nationality: '韩国', team: 'TL', region: 'LCS', position: 'SUP', age: 29, ability: 85, potential: 86, stability: 90, tag: 'NORMAL', salary: 1500000, marketValue: 6500000, contractEnd: 'S2', joinSeason: 'S1' },
]

// 根据ID找到对应选手
const foundPlayer = allPlayers.find(p => p.id === Number(playerId))

// 选手数据
const player = ref(foundPlayer || {
  id: Number(playerId),
  gameId: '未知选手',
  realName: '未知',
  nationality: '未知',
  team: '未知',
  region: 'LPL',
  position: 'MID',
  age: 20,
  ability: 70,
  potential: 80,
  stability: 75,
  tag: 'NORMAL',
  salary: 500000,
  marketValue: 3000000,
  contractEnd: 'S2',
  joinSeason: 'S1',
})

// 荣誉记录 - 根据选手能力值生成
const generateHonors = () => {
  if (!foundPlayer) return []
  const honors = []
  if (foundPlayer.ability >= 90) {
    honors.push({ season: 'S1', tournament: `${foundPlayer.region} 春季赛`, position: '冠军' })
    honors.push({ season: 'S1', tournament: 'MSI 季中赛', position: '冠军' })
    honors.push({ season: 'S1', tournament: `${foundPlayer.region} 夏季赛`, position: '亚军' })
  } else if (foundPlayer.ability >= 85) {
    honors.push({ season: 'S1', tournament: `${foundPlayer.region} 春季赛`, position: '亚军' })
    honors.push({ season: 'S1', tournament: `${foundPlayer.region} 夏季赛`, position: '季军' })
  } else if (foundPlayer.ability >= 80) {
    honors.push({ season: 'S1', tournament: `${foundPlayer.region} 夏季赛`, position: '季军' })
  }
  return honors
}

const honors = ref(generateHonors())

// 赛季历史
const seasonHistory = ref([
  { season: 'S1', team: player.value.team, ability: player.value.ability, potential: player.value.potential },
])

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

// 辅助函数
const formatMoney = (value: number) => {
  if (value >= 10000000) {
    return `${(value / 10000000).toFixed(1)} 千万`
  }
  return `${(value / 10000).toFixed(0)} 万`
}

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

const getHonorColor = (position: string) => {
  const colors: Record<string, string> = {
    '冠军': '#fbbf24',
    '亚军': '#9ca3af',
    '季军': '#f97316',
  }
  return colors[position] || '#3b82f6'
}

const getHonorClass = (position: string) => {
  const classes: Record<string, string> = {
    '冠军': 'champion',
    '亚军': 'runner-up',
    '季军': 'third-place',
  }
  return classes[position] || ''
}

const getHonorEmoji = (position: string) => {
  const emojis: Record<string, string> = {
    '冠军': '🏆',
    '亚军': '🥈',
    '季军': '🥉',
  }
  return emojis[position] || '🏅'
}

const getHonorTagType = (position: string) => {
  const types: Record<string, string> = {
    '冠军': 'warning',
    '亚军': 'info',
    '季军': 'danger',
  }
  return types[position] || 'primary'
}
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
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 24px;
}

.player-avatar.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.player-avatar.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.player-avatar.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.player-avatar.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
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

.team-avatar.mini {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 10px;
}

.team-avatar.mini.lpl {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.team-avatar.mini.lck {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.team-avatar.mini.lec {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.team-avatar.mini.lcs {
  background: linear-gradient(135deg, #f59e0b, #d97706);
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
</style>
