<template>
  <div class="transfer-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <div>
        <h1>转会中心</h1>
        <p>S1赛季 - 转会窗口期</p>
      </div>
      <div class="header-actions">
        <!-- 转会状态标签 -->
        <el-tag v-if="transferState === 'idle'" type="info" size="large" effect="dark">
          <el-icon><Clock /></el-icon>
          等待开始转会
        </el-tag>
        <el-tag v-else-if="transferState === 'processing'" type="warning" size="large" effect="dark">
          <el-icon><Loading /></el-icon>
          转会进行中...
        </el-tag>
        <el-tag v-else-if="transferState === 'completed'" type="success" size="large" effect="dark">
          <el-icon><Check /></el-icon>
          转会已完成
        </el-tag>

        <!-- 操作按钮 -->
        <el-button
          v-if="transferState === 'idle'"
          type="primary"
          @click="startTransfer"
        >
          <el-icon><VideoPlay /></el-icon>
          开始转会
        </el-button>
        <el-button
          v-else-if="transferState === 'completed'"
          type="success"
          @click="confirmTransfer"
        >
          <el-icon><CircleCheck /></el-icon>
          确认完成
        </el-button>
      </div>
    </div>

    <!-- 统计概览 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon blue">
              <el-icon :size="28"><User /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ freeAgents.length }}</div>
              <div class="stat-label">自由球员</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon green">
              <el-icon :size="28"><Goods /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ listings.length }}</div>
              <div class="stat-label">挂牌出售</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon orange">
              <el-icon :size="28"><Switch /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ recentTransfers.length }}</div>
              <div class="stat-label">完成交易</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon purple">
              <el-icon :size="28"><Money /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ totalFees }}万</div>
              <div class="stat-label">总转会费</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 主要内容区：左侧筛选 + 右侧列表 -->
    <div class="main-content">
      <!-- 左侧筛选栏 -->
      <el-card class="filter-sidebar">
        <template #header>
          <div class="filter-header">
            <el-icon><Filter /></el-icon>
            <span>筛选条件</span>
          </div>
        </template>

        <!-- 类型筛选 -->
        <div class="filter-section">
          <div class="filter-title">选手类型</div>
          <el-radio-group v-model="filterType" @change="applyFilters">
            <el-radio value="all">全部</el-radio>
            <el-radio value="free">自由球员</el-radio>
            <el-radio value="listing">挂牌出售</el-radio>
          </el-radio-group>
        </div>

        <!-- 位置筛选 -->
        <div class="filter-section">
          <div class="filter-title">位置筛选</div>
          <el-checkbox-group v-model="selectedPositions" @change="applyFilters">
            <el-checkbox value="TOP">
              <span class="position-tag top">TOP</span> 上单
            </el-checkbox>
            <el-checkbox value="JUG">
              <span class="position-tag jug">JUG</span> 打野
            </el-checkbox>
            <el-checkbox value="MID">
              <span class="position-tag mid">MID</span> 中单
            </el-checkbox>
            <el-checkbox value="ADC">
              <span class="position-tag adc">ADC</span> 下路
            </el-checkbox>
            <el-checkbox value="SUP">
              <span class="position-tag sup">SUP</span> 辅助
            </el-checkbox>
          </el-checkbox-group>
        </div>

        <!-- 赛区筛选 -->
        <div class="filter-section">
          <div class="filter-title">赛区筛选</div>
          <el-checkbox-group v-model="selectedRegions" @change="applyFilters">
            <el-checkbox value="LPL">
              <el-tag type="danger" size="small">LPL</el-tag>
            </el-checkbox>
            <el-checkbox value="LCK">
              <el-tag type="primary" size="small">LCK</el-tag>
            </el-checkbox>
            <el-checkbox value="LEC">
              <el-tag type="success" size="small">LEC</el-tag>
            </el-checkbox>
            <el-checkbox value="LCS">
              <el-tag type="warning" size="small">LCS</el-tag>
            </el-checkbox>
          </el-checkbox-group>
        </div>

        <!-- 能力值筛选 -->
        <div class="filter-section">
          <div class="filter-title">能力值范围</div>
          <el-slider
            v-model="abilityRange"
            range
            :min="50"
            :max="100"
            :marks="abilityMarks"
            @change="applyFilters"
          />
          <div class="range-display">
            {{ abilityRange[0] }} - {{ abilityRange[1] }}
          </div>
        </div>

        <!-- 年龄筛选 -->
        <div class="filter-section">
          <div class="filter-title">年龄范围</div>
          <el-slider
            v-model="ageRange"
            range
            :min="16"
            :max="32"
            @change="applyFilters"
          />
          <div class="range-display">
            {{ ageRange[0] }}岁 - {{ ageRange[1] }}岁
          </div>
        </div>

        <!-- 清空筛选 -->
        <el-button class="clear-filter-btn" @click="clearFilters">
          <el-icon><Refresh /></el-icon>
          清空筛选
        </el-button>
      </el-card>

      <!-- 右侧内容区 -->
      <div class="content-area">
        <!-- 自由球员市场 -->
        <el-card v-if="filterType === 'all' || filterType === 'free'" class="market-card">
          <template #header>
            <div class="market-header">
              <div class="market-title">
                <el-icon class="title-icon free"><UserFilled /></el-icon>
                <span>自由球员市场</span>
                <el-tag type="success" size="small">{{ filteredFreeAgents.length }}人</el-tag>
              </div>
              <el-input
                v-model="searchFree"
                placeholder="搜索自由球员"
                clearable
                style="width: 200px;"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </div>
          </template>

          <el-empty v-if="filteredFreeAgents.length === 0" description="暂无符合条件的自由球员" />

          <div v-else class="player-grid">
            <div v-for="player in filteredFreeAgents" :key="'free-' + player.id" class="player-card free-agent">
              <div class="card-badge free">自由球员</div>
              <div class="player-header">
                <div class="player-avatar" :class="player.position.toLowerCase()">
                  {{ player.position }}
                </div>
                <div class="player-info">
                  <div class="player-name">{{ player.gameId }}</div>
                  <div class="player-meta">
                    <span class="player-realname">{{ player.name }}</span>
                    <el-tag size="small" type="info">{{ player.nationality }}</el-tag>
                  </div>
                </div>
                <div class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
                  {{ player.ability }}
                </div>
              </div>

              <div class="player-stats">
                <div class="stat-item">
                  <span class="stat-label">年龄</span>
                  <span class="stat-value">{{ player.age }}岁</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">潜力</span>
                  <span class="stat-value potential">{{ player.potential }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">标签</span>
                  <el-tag :type="getTalentType(player.tag)" size="small">
                    {{ getTalentLabel(player.tag) }}
                  </el-tag>
                </div>
                <div class="stat-item">
                  <span class="stat-label">稳定性</span>
                  <span class="stat-value">{{ player.stability }}</span>
                </div>
              </div>

              <div class="player-contract">
                <div class="contract-item">
                  <el-icon><Wallet /></el-icon>
                  <span class="contract-label">期望年薪</span>
                  <span class="contract-value salary">{{ formatPrice(player.salaryDemand) }}/年</span>
                </div>
              </div>

              <div class="player-actions">
                <el-button type="primary" @click="viewPlayer(player)">
                  <el-icon><View /></el-icon>
                  查看详情
                </el-button>
              </div>
            </div>
          </div>
        </el-card>

        <!-- 挂牌出售 -->
        <el-card v-if="filterType === 'all' || filterType === 'listing'" class="market-card">
          <template #header>
            <div class="market-header">
              <div class="market-title">
                <el-icon class="title-icon listing"><PriceTag /></el-icon>
                <span>挂牌出售</span>
                <el-tag type="warning" size="small">{{ filteredListings.length }}人</el-tag>
              </div>
              <el-input
                v-model="searchListing"
                placeholder="搜索挂牌选手"
                clearable
                style="width: 200px;"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </div>
          </template>

          <el-empty v-if="filteredListings.length === 0" description="暂无符合条件的挂牌选手" />

          <div v-else class="player-grid">
            <div v-for="player in filteredListings" :key="'listing-' + player.id" class="player-card listing">
              <div class="card-badge listing">挂牌出售</div>
              <div class="player-header">
                <div class="player-avatar" :class="player.position.toLowerCase()">
                  {{ player.position }}
                </div>
                <div class="player-info">
                  <div class="player-name">{{ player.gameId }}</div>
                  <div class="player-meta">
                    <el-tag :type="getRegionTagType(player.region)" size="small">
                      {{ player.region }}
                    </el-tag>
                    <span class="team-name">{{ player.team }}</span>
                  </div>
                </div>
                <div class="ability-number" :style="{ color: getAbilityColor(player.ability) }">
                  {{ player.ability }}
                </div>
              </div>

              <div class="player-stats">
                <div class="stat-item">
                  <span class="stat-label">年龄</span>
                  <span class="stat-value">{{ player.age }}岁</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">潜力</span>
                  <span class="stat-value potential">{{ player.potential }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">标签</span>
                  <el-tag :type="getTalentType(player.tag)" size="small">
                    {{ getTalentLabel(player.tag) }}
                  </el-tag>
                </div>
                <div class="stat-item">
                  <span class="stat-label">稳定性</span>
                  <span class="stat-value">{{ player.stability }}</span>
                </div>
              </div>

              <div class="player-contract">
                <div class="contract-item">
                  <el-icon><Money /></el-icon>
                  <span class="contract-label">转会费</span>
                  <span class="contract-value price">{{ formatPrice(player.askingPrice) }}</span>
                </div>
                <div class="contract-item">
                  <el-icon><Wallet /></el-icon>
                  <span class="contract-label">年薪</span>
                  <span class="contract-value">{{ formatPrice(player.salary) }}/年</span>
                </div>
                <div class="contract-item">
                  <el-icon><Calendar /></el-icon>
                  <span class="contract-label">合同</span>
                  <span class="contract-value">剩余{{ player.contractYears }}年</span>
                </div>
              </div>

              <div class="player-actions">
                <el-button type="primary" @click="viewPlayer(player)">
                  <el-icon><View /></el-icon>
                  查看详情
                </el-button>
              </div>
            </div>
          </div>
        </el-card>
      </div>
    </div>

    <!-- 底部转会动态 -->
    <el-card class="transfer-timeline-card">
      <template #header>
        <div class="timeline-header">
          <div class="timeline-title">
            <el-icon class="title-icon timeline"><Bell /></el-icon>
            <span>转会记录</span>
            <el-tag v-if="recentTransfers.length > 0" type="primary" size="small">
              {{ recentTransfers.length }}条记录
            </el-tag>
          </div>
          <el-button
            v-if="recentTransfers.length > 0"
            type="primary"
            link
            @click="showTransferDetail = true"
          >
            查看详情
            <el-icon><ArrowRight /></el-icon>
          </el-button>
        </div>
      </template>

      <el-empty v-if="recentTransfers.length === 0" description="暂无转会记录，点击「开始转会」观看AI转会" />

      <div v-else class="transfer-timeline">
        <div v-for="(transfer, index) in recentTransfers.slice(0, 5)" :key="index" class="timeline-item">
          <div class="timeline-dot" :class="transfer.type"></div>
          <div class="timeline-content">
            <div class="transfer-info">
              <span class="player-name">{{ transfer.player }}</span>
              <el-tag size="small" type="info">{{ transfer.position }}</el-tag>
              <template v-if="transfer.type === 'purchase'">
                <el-tag :type="getRegionTagType(transfer.fromRegion)" size="small">
                  {{ transfer.from }}
                </el-tag>
                <el-icon class="arrow"><Right /></el-icon>
                <el-tag :type="getRegionTagType(transfer.toRegion)" size="small">
                  {{ transfer.to }}
                </el-tag>
                <span class="transfer-fee">转会费: {{ formatPrice(transfer.fee) }}</span>
              </template>
              <template v-else-if="transfer.type === 'free'">
                <span class="transfer-desc">合同到期 → 自由球员</span>
              </template>
              <template v-else-if="transfer.type === 'retire'">
                <span class="transfer-desc retire">宣布退役</span>
              </template>
              <template v-else-if="transfer.type === 'sign'">
                <span class="transfer-desc">签约</span>
                <el-tag :type="getRegionTagType(transfer.toRegion)" size="small">
                  {{ transfer.to }}
                </el-tag>
              </template>
            </div>
            <div class="transfer-time">{{ transfer.date }}</div>
          </div>
        </div>
        <div v-if="recentTransfers.length > 5" class="more-transfers">
          还有 {{ recentTransfers.length - 5 }} 条记录...
        </div>
      </div>
    </el-card>

    <!-- 转会详情弹窗 -->
    <el-dialog
      v-model="showTransferDetail"
      title="转会详情"
      width="800px"
      :close-on-click-modal="false"
    >
      <div class="transfer-detail-content">
        <div class="transfer-summary">
          <div class="summary-item">
            <div class="summary-value">{{ transferSummary.totalTransfers }}</div>
            <div class="summary-label">总交易数</div>
          </div>
          <div class="summary-item">
            <div class="summary-value">{{ formatPrice(transferSummary.totalFees) }}</div>
            <div class="summary-label">总转会费</div>
          </div>
          <div class="summary-item">
            <div class="summary-value">{{ transferSummary.freeAgentsSigned }}</div>
            <div class="summary-label">自由球员签约</div>
          </div>
          <div class="summary-item">
            <div class="summary-value">{{ transferSummary.retirements }}</div>
            <div class="summary-label">退役人数</div>
          </div>
        </div>

        <el-divider />

        <div class="transfer-list-detail">
          <div v-for="(transfer, index) in recentTransfers" :key="index" class="transfer-detail-item">
            <div class="transfer-index">{{ index + 1 }}</div>
            <div class="transfer-detail-main">
              <div class="transfer-player-info">
                <span class="player-name">{{ transfer.player }}</span>
                <el-tag size="small" type="info">{{ transfer.position }}</el-tag>
              </div>
              <div class="transfer-detail-desc">
                <template v-if="transfer.type === 'purchase'">
                  从
                  <el-tag :type="getRegionTagType(transfer.fromRegion)" size="small">{{ transfer.from }}</el-tag>
                  转会至
                  <el-tag :type="getRegionTagType(transfer.toRegion)" size="small">{{ transfer.to }}</el-tag>
                  <span class="transfer-fee">转会费: {{ formatPrice(transfer.fee) }}</span>
                </template>
                <template v-else-if="transfer.type === 'free'">
                  合同到期，成为自由球员
                </template>
                <template v-else-if="transfer.type === 'retire'">
                  <span class="retire-text">宣布退役</span>
                </template>
                <template v-else-if="transfer.type === 'sign'">
                  以自由球员身份签约
                  <el-tag :type="getRegionTagType(transfer.toRegion)" size="small">{{ transfer.to }}</el-tag>
                </template>
              </div>
            </div>
            <div class="transfer-detail-time">{{ transfer.date }}</div>
          </div>
        </div>
      </div>

      <template #footer>
        <el-button @click="showTransferDetail = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Clock,
  User,
  Goods,
  Switch,
  Money,
  Filter,
  Refresh,
  UserFilled,
  Search,
  Check,
  PriceTag,
  Wallet,
  Calendar,
  Bell,
  ArrowRight,
  Right,
  Loading,
  VideoPlay,
  CircleCheck,
  View,
} from '@element-plus/icons-vue'

const router = useRouter()

// 转会状态: idle(等待开始) | processing(进行中) | completed(已完成)
type TransferState = 'idle' | 'processing' | 'completed'
const transferState = ref<TransferState>('idle')
const showTransferDetail = ref(false)

// 筛选状态
const filterType = ref('all')
const selectedPositions = ref<string[]>([])
const selectedRegions = ref<string[]>([])
const abilityRange = ref([50, 100])
const ageRange = ref([16, 32])
const searchFree = ref('')
const searchListing = ref('')

// 能力值标记
const abilityMarks = {
  60: '60',
  70: '70',
  80: '80',
  90: '90',
}

// 模拟数据 - 自由球员
const freeAgents = ref([
  { id: 1, gameId: 'Rookie', name: '宋义进', nationality: '韩国', position: 'MID', ability: 78, potential: 80, age: 28, tag: 'NORMAL', stability: 85, salaryDemand: 3000000 },
  { id: 2, gameId: 'Karsa', name: '洪浩轩', nationality: '中国台湾', position: 'JUG', ability: 76, potential: 77, age: 27, tag: 'NORMAL', stability: 88, salaryDemand: 2500000 },
  { id: 3, gameId: 'Huni', name: '许盛勋', nationality: '韩国', position: 'TOP', ability: 74, potential: 75, age: 26, tag: 'ORDINARY', stability: 70, salaryDemand: 2000000 },
  { id: 4, gameId: 'Rekkles', name: 'Martin', nationality: '瑞典', position: 'ADC', ability: 75, potential: 76, age: 28, tag: 'NORMAL', stability: 90, salaryDemand: 2500000 },
  { id: 5, gameId: 'SwordArt', name: '胡硕杰', nationality: '中国台湾', position: 'SUP', ability: 72, potential: 73, age: 27, tag: 'ORDINARY', stability: 85, salaryDemand: 1800000 },
  { id: 6, gameId: 'Ning', name: '高振宁', nationality: '中国', position: 'JUG', ability: 70, potential: 72, age: 26, tag: 'ORDINARY', stability: 65, salaryDemand: 1500000 },
])

// 模拟数据 - 挂牌出售
const listings = ref([
  { id: 1, gameId: 'Zeus', name: '崔宇杰', team: 'T1', region: 'LCK', position: 'TOP', ability: 88, potential: 94, age: 22, tag: 'GENIUS', stability: 75, askingPrice: 8000000, salary: 1500000, contractYears: 2 },
  { id: 2, gameId: 'Canyon', name: '金建部', team: 'Gen.G', region: 'LCK', position: 'JUG', ability: 90, potential: 92, age: 23, tag: 'GENIUS', stability: 80, askingPrice: 10000000, salary: 2000000, contractYears: 1 },
  { id: 3, gameId: 'Knight', name: '卓定', team: 'TES', region: 'LPL', position: 'MID', ability: 92, potential: 95, age: 24, tag: 'GENIUS', stability: 78, askingPrice: 12000000, salary: 2500000, contractYears: 2 },
  { id: 4, gameId: 'Elk', name: '谢家辉', team: 'BLG', region: 'LPL', position: 'ADC', ability: 86, potential: 90, age: 22, tag: 'GENIUS', stability: 72, askingPrice: 7000000, salary: 1200000, contractYears: 1 },
  { id: 5, gameId: 'Meiko', name: '田野', team: 'EDG', region: 'LPL', position: 'SUP', ability: 85, potential: 86, age: 26, tag: 'NORMAL', stability: 90, askingPrice: 5000000, salary: 1000000, contractYears: 1 },
  { id: 6, gameId: 'Caps', name: 'Rasmus', team: 'G2', region: 'LEC', position: 'MID', ability: 84, potential: 88, age: 25, tag: 'NORMAL', stability: 75, askingPrice: 6000000, salary: 1500000, contractYears: 2 },
])

// 模拟数据 - 转会记录 (初始为空，转会开始后填充)
const recentTransfers = ref<Array<{
  player: string
  position: string
  from: string
  fromRegion: string
  to: string
  toRegion: string
  fee: number
  type: 'purchase' | 'free' | 'retire' | 'sign'
  date: string
}>>([])

// 转会摘要统计
const transferSummary = computed(() => {
  const purchases = recentTransfers.value.filter(t => t.type === 'purchase')
  const signs = recentTransfers.value.filter(t => t.type === 'sign')
  const retires = recentTransfers.value.filter(t => t.type === 'retire')

  return {
    totalTransfers: recentTransfers.value.length,
    totalFees: purchases.reduce((sum, t) => sum + t.fee, 0),
    freeAgentsSigned: signs.length,
    retirements: retires.length,
  }
})

// 计算属性
const totalFees = computed(() => {
  return Math.round(recentTransfers.value.filter(t => t.type === 'purchase').reduce((sum, t) => sum + t.fee, 0) / 10000)
})

const filteredFreeAgents = computed(() => {
  return freeAgents.value.filter(p => {
    if (selectedPositions.value.length && !selectedPositions.value.includes(p.position)) return false
    if (p.ability < abilityRange.value[0] || p.ability > abilityRange.value[1]) return false
    if (p.age < ageRange.value[0] || p.age > ageRange.value[1]) return false
    if (searchFree.value && !p.gameId.toLowerCase().includes(searchFree.value.toLowerCase())) return false
    return true
  })
})

const filteredListings = computed(() => {
  return listings.value.filter(p => {
    if (selectedPositions.value.length && !selectedPositions.value.includes(p.position)) return false
    if (selectedRegions.value.length && !selectedRegions.value.includes(p.region)) return false
    if (p.ability < abilityRange.value[0] || p.ability > abilityRange.value[1]) return false
    if (p.age < ageRange.value[0] || p.age > ageRange.value[1]) return false
    if (searchListing.value && !p.gameId.toLowerCase().includes(searchListing.value.toLowerCase())) return false
    return true
  })
})

// 方法
const applyFilters = () => {
  // 筛选会自动通过计算属性应用
}

const clearFilters = () => {
  filterType.value = 'all'
  selectedPositions.value = []
  selectedRegions.value = []
  abilityRange.value = [50, 100]
  ageRange.value = [16, 32]
  searchFree.value = ''
  searchListing.value = ''
}

// 开始转会 - AI自动执行转会
const startTransfer = async () => {
  await ElMessageBox.confirm(
    'AI战队将开始进行转会操作，你可以观看转会结果。是否开始？',
    '开始转会',
    {
      confirmButtonText: '开始',
      cancelButtonText: '取消',
      type: 'info',
    }
  )

  transferState.value = 'processing'
  ElMessage.info('转会进行中...')

  // 模拟AI转会过程 (实际应该调用后端API)
  await simulateTransfers()

  transferState.value = 'completed'
  showTransferDetail.value = true
  ElMessage.success('转会已完成！')
}

// 模拟AI转会
const simulateTransfers = async () => {
  // 模拟延迟
  await new Promise(resolve => setTimeout(resolve, 2000))

  // 生成模拟的转会记录
  const mockTransfers = [
    { player: 'Chovy', position: 'MID', from: 'Gen.G', fromRegion: 'LCK', to: 'T1', toRegion: 'LCK', fee: 15000000, type: 'purchase' as const, date: 'S1 转会窗口' },
    { player: 'Ruler', position: 'ADC', from: 'Gen.G', fromRegion: 'LCK', to: 'JDG', toRegion: 'LPL', fee: 12000000, type: 'purchase' as const, date: 'S1 转会窗口' },
    { player: 'Viper', position: 'ADC', from: 'EDG', fromRegion: 'LPL', to: 'HLE', toRegion: 'LCK', fee: 8000000, type: 'purchase' as const, date: 'S1 转会窗口' },
    { player: 'Rookie', position: 'MID', from: '', fromRegion: '', to: 'WBG', toRegion: 'LPL', fee: 0, type: 'sign' as const, date: 'S1 转会窗口' },
    { player: 'Karsa', position: 'JUG', from: '', fromRegion: '', to: 'LNG', toRegion: 'LPL', fee: 0, type: 'sign' as const, date: 'S1 转会窗口' },
    { player: 'Uzi', position: 'ADC', from: '', fromRegion: '', to: '', toRegion: '', fee: 0, type: 'retire' as const, date: 'S1 转会窗口' },
    { player: 'TheShy', position: 'TOP', from: 'WBG', fromRegion: 'LPL', to: '', toRegion: '', fee: 0, type: 'free' as const, date: 'S1 转会窗口' },
  ]

  recentTransfers.value = mockTransfers

  // 更新市场数据 - 移除已签约的自由球员
  const signedPlayers = mockTransfers.filter(t => t.type === 'sign').map(t => t.player)
  freeAgents.value = freeAgents.value.filter(p => !signedPlayers.includes(p.gameId))

  // 移除已转会的挂牌选手 (模拟)
  const purchasedPlayers = mockTransfers.filter(t => t.type === 'purchase').map(t => t.player)
  // 这里可以更新listings，但保持简单暂不处理
}

// 确认完成转会
const confirmTransfer = async () => {
  await ElMessageBox.confirm(
    '确认完成转会？队员转换将生效，进入下一阶段。',
    '确认完成',
    {
      confirmButtonText: '确认',
      cancelButtonText: '取消',
      type: 'success',
    }
  )

  ElMessage.success('转会期已结束，队员转换完成！')
  // 这里可以跳转到下一个阶段或重置状态
  // router.push('/season')
}

const viewPlayer = (player: any) => {
  router.push(`/players/${player.id}`)
}

// 辅助函数
const getRegionTagType = (region: string) => {
  const types: Record<string, string> = {
    LPL: 'danger',
    LCK: 'primary',
    LEC: 'success',
    LCS: 'warning',
  }
  return types[region] || 'info'
}

const getAbilityColor = (ability: number) => {
  if (ability >= 90) return '#ef4444'
  if (ability >= 80) return '#f59e0b'
  if (ability >= 70) return '#3b82f6'
  return '#22c55e'
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

const formatPrice = (price: number) => {
  if (price >= 10000000) {
    return `${(price / 10000000).toFixed(1)}千万`
  }
  return `${(price / 10000).toFixed(0)}万`
}
</script>

<style scoped>
.transfer-view {
  padding: 0;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 14px;
  color: var(--text-tertiary, #909399);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

/* 统计卡片 */
.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 12px;
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.blue { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.stat-icon.green { background: linear-gradient(135deg, #22c55e, #16a34a); }
.stat-icon.orange { background: linear-gradient(135deg, #f59e0b, #d97706); }
.stat-icon.purple { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.stat-info { flex: 1; }

.stat-number {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: var(--text-tertiary, #909399);
  margin-top: 4px;
}

/* 主内容区布局 */
.main-content {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
}

/* 左侧筛选栏 */
.filter-sidebar {
  width: 260px;
  flex-shrink: 0;
  border-radius: 12px;
  height: fit-content;
  position: sticky;
  top: 20px;
}

.filter-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.filter-section {
  margin-bottom: 24px;
}

.filter-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary, #606266);
  margin-bottom: 12px;
}

.filter-section :deep(.el-radio-group),
.filter-section :deep(.el-checkbox-group) {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.position-tag {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  color: white;
  margin-right: 4px;
}

.position-tag.top { background: #ef4444; }
.position-tag.jug { background: #22c55e; }
.position-tag.mid { background: #3b82f6; }
.position-tag.adc { background: #f59e0b; }
.position-tag.sup { background: #8b5cf6; }

.range-display {
  text-align: center;
  margin-top: 8px;
  font-size: 13px;
  color: var(--text-secondary, #606266);
  font-weight: 500;
}

.clear-filter-btn {
  width: 100%;
  margin-top: 8px;
}

/* 右侧内容区 */
.content-area {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 市场卡片 */
.market-card {
  border-radius: 12px;
}

.market-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.market-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.title-icon {
  font-size: 20px;
}

.title-icon.free { color: #22c55e; }
.title-icon.listing { color: #f59e0b; }
.title-icon.timeline { color: #3b82f6; }

/* 选手卡片网格 */
.player-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

/* 选手卡片 */
.player-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  border: 1px solid #ebeef5;
  position: relative;
  transition: all 0.3s ease;
}

.player-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.player-card.free-agent {
  border-top: 3px solid #22c55e;
}

.player-card.listing {
  border-top: 3px solid #f59e0b;
}

.card-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  color: white;
}

.card-badge.free { background: #22c55e; }
.card-badge.listing { background: #f59e0b; }

.player-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding-top: 20px;
}

.player-avatar {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 13px;
}

.player-avatar.top { background: linear-gradient(135deg, #ef4444, #dc2626); }
.player-avatar.jug { background: linear-gradient(135deg, #22c55e, #16a34a); }
.player-avatar.mid { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.player-avatar.adc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.player-avatar.sup { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.player-info {
  flex: 1;
}

.player-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin-bottom: 4px;
}

.player-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.player-realname {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
}

.team-name {
  font-size: 12px;
  color: var(--text-secondary, #606266);
}

/* 选手属性 */
.player-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  margin-bottom: 16px;
  padding: 12px;
  background: #f5f7fa;
  border-radius: 8px;
}

.ability-number {
  flex-shrink: 0;
  font-size: 24px;
  font-weight: 700;
  min-width: 45px;
  text-align: center;
}

.player-stats .stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.player-stats .stat-label {
  font-size: 11px;
  color: var(--text-tertiary, #909399);
}

.player-stats .stat-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.player-stats .stat-value.potential {
  color: #8b5cf6;
}

/* 合同信息 */
.player-contract {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid #ebeef5;
}

.contract-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary, #606266);
}

.contract-item .el-icon {
  color: var(--text-tertiary, #909399);
}

.contract-label {
  color: var(--text-tertiary, #909399);
}

.contract-value {
  font-weight: 600;
  color: var(--text-primary, #303133);
}

.contract-value.price {
  color: #f59e0b;
}

.contract-value.salary {
  color: #22c55e;
}

/* 操作按钮 */
.player-actions {
  display: flex;
  gap: 8px;
}

.player-actions .el-button {
  flex: 1;
}

/* 转会动态卡片 */
.transfer-timeline-card {
  border-radius: 12px;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.timeline-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #303133);
}

/* 时间线 */
.transfer-timeline {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.timeline-item {
  display: flex;
  gap: 16px;
  padding: 12px 16px;
  background: #f5f7fa;
  border-radius: 8px;
  align-items: center;
}

.timeline-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.timeline-dot.purchase { background: #3b82f6; }
.timeline-dot.free { background: #22c55e; }
.timeline-dot.retire { background: #909399; }
.timeline-dot.sign { background: #8b5cf6; }

.timeline-content {
  flex: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.transfer-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.transfer-info .player-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #303133);
  margin: 0;
}

.transfer-info .arrow {
  color: var(--text-tertiary, #909399);
}

.transfer-fee {
  font-size: 13px;
  font-weight: 600;
  color: #f59e0b;
  margin-left: 8px;
}

.transfer-desc {
  font-size: 13px;
  color: var(--text-secondary, #606266);
}

.transfer-desc.retire {
  color: #909399;
}

.transfer-time {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
  white-space: nowrap;
}

/* 更多转会提示 */
.more-transfers {
  text-align: center;
  padding: 12px;
  color: var(--text-tertiary, #909399);
  font-size: 13px;
}

/* 转会详情弹窗 */
.transfer-detail-content {
  max-height: 60vh;
  overflow-y: auto;
}

.transfer-summary {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 12px;
}

.summary-item {
  text-align: center;
}

.summary-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary, #303133);
  margin-bottom: 4px;
}

.summary-label {
  font-size: 13px;
  color: var(--text-tertiary, #909399);
}

.transfer-list-detail {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.transfer-detail-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.transfer-index {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: #e0e0e0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: #666;
  flex-shrink: 0;
}

.transfer-detail-main {
  flex: 1;
}

.transfer-player-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.transfer-player-info .player-name {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
}

.transfer-detail-desc {
  font-size: 13px;
  color: var(--text-secondary, #606266);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.transfer-detail-desc .transfer-fee {
  margin-left: 4px;
}

.retire-text {
  color: #909399;
}

.transfer-detail-time {
  font-size: 12px;
  color: var(--text-tertiary, #909399);
  white-space: nowrap;
}

/* 响应式 */
@media (max-width: 1200px) {
  .player-grid {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  }
}

@media (max-width: 992px) {
  .main-content {
    flex-direction: column;
  }

  .filter-sidebar {
    width: 100%;
    position: static;
  }

  .filter-section :deep(.el-radio-group),
  .filter-section :deep(.el-checkbox-group) {
    flex-direction: row;
    flex-wrap: wrap;
  }
}

@media (max-width: 768px) {
  .player-stats {
    grid-template-columns: repeat(2, 1fr);
  }

  .timeline-content {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
}
</style>
