<template>
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
  </el-card>
</template>

<script setup lang="ts">
import { ref, PropType } from 'vue'
import { Lightning, InfoFilled, Aim } from '@element-plus/icons-vue'
import { TraitInfo } from '@/api/tauri'

const props = defineProps({
  traits: {
    type: Array as PropType<TraitInfo[]>,
    default: () => []
  },
  allTraits: {
    type: Array as PropType<any[]>,
    required: true
  }
})

const showTraitDialog = ref(false)
const selectedTrait = ref<TraitInfo | null>(null)
const showTraitsGuide = ref(false)

const openTraitDetail = (trait: TraitInfo) => {
  selectedTrait.value = trait
  showTraitDialog.value = true
}

// Helper functions for trait details (extracted from original)
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
</script>

<style scoped>
.traits-card {
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

.header-icon {
  font-size: 18px;
  margin-right: 4px;
}

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

.count-badge {
  font-size: 14px;
  color: var(--text-tertiary);
}

.empty-icon {
  font-size: 64px;
}

/* Traits Grid */
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

.clickable {
  cursor: pointer;
}

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

/* Trait Dialog Styles */
.trait-detail-dialog :deep(.el-dialog__header) {
  display: none;
}

.trait-detail-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.trait-detail-content {
  padding: 0;
}

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

.trait-info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin: 16px;
}

/* Guide Dialog Styles */
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
</style>
