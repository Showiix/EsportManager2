/**
 * 选手相关类型定义
 */

// 选手位置
export type PlayerPosition = 'TOP' | 'JUG' | 'MID' | 'ADC' | 'SUP'

// 选手天赋标签
export type PlayerTalent = 'GENIUS' | 'NORMAL' | 'ORDINARY'

// 选手基础接口
export interface Player {
  id: string
  gameId: string               // 游戏ID（如 "Faker"）
  name: string                 // 真实姓名
  teamId: string               // 所属队伍ID
  teamName?: string            // 队伍名称
  position: PlayerPosition     // 位置
  regionId: string             // 赛区ID
  regionName?: string          // 赛区名称

  // 核心能力值
  ability: number              // 平均能力值 (0-100)
  potential: number            // 潜力值 (0-100)
  stability: number            // 稳定性 (0-100)，决定波动幅度

  // 状态
  condition: number            // 当前状态 (-10 ~ +10)
  age: number                  // 年龄
  tag: PlayerTalent            // 天赋标签

  // 可选字段
  nationality?: string         // 国籍
  salary?: number              // 年薪
  marketValue?: number         // 身价
  contractEnd?: string         // 合同到期赛季
}

// 激活的特性效果
export interface ActivatedTrait {
  type: TraitType
  name: string
  effect: string              // 效果描述，如 "+3 状态"
  value: number               // 效果数值
  isPositive: boolean         // 是否正面效果
}

// 选手单场发挥数据
export interface PlayerPerformance {
  playerId: string
  playerName: string
  position: PlayerPosition
  teamId: string

  // 能力计算过程
  baseAbility: number          // 基础能力（ability）
  conditionBonus: number       // 状态加成
  stabilityNoise: number       // 稳定性波动（高斯噪声）
  actualAbility: number        // 实际能力 = base + condition + noise（钳位后）

  // 统计分析
  impactScore: number          // 影响力分数 = actualAbility - teamAverage
  mvpScore?: number            // MVP 得分

  // 详细战斗数据（可选）
  kills?: number
  deaths?: number
  assists?: number
  cs?: number
  gold?: number
  damageDealt?: number
  damageTaken?: number
  visionScore?: number

  // 特性系统
  traits?: TraitType[]             // 选手拥有的特性
  activatedTraits?: ActivatedTrait[] // 本局激活的特性效果
}

// 选手赛季统计
export interface PlayerSeasonStats {
  playerId: string
  playerName: string
  seasonId: string
  teamId: string
  regionId?: string              // 赛区ID
  position: PlayerPosition

  // 比赛统计
  matchesPlayed: number        // 参与比赛场数
  gamesPlayed: number          // 参与小局数

  // 影响力统计
  totalImpact: number          // 累计影响力
  avgImpact: number            // 平均影响力

  // 发挥统计
  avgPerformance: number       // 平均发挥
  bestPerformance: number      // 最高单局发挥
  worstPerformance: number     // 最低单局发挥

  // 稳定性评分
  consistencyScore: number     // 稳定性评分（基于发挥波动）

  // 冠军加成
  internationalTitles: number  // 国际赛冠军次数 (MSI/Worlds)
  regionalTitles: number       // 赛区冠军次数
  championBonus: number        // 冠军加成分数 = 国际赛*3 + 赛区*1

  // 年度Top得分
  yearlyTopScore: number       // 年度Top得分 = 影响力×70% + 冠军加成×30%
}

// 位置名称映射
export const POSITION_NAMES: Record<PlayerPosition, string> = {
  TOP: '上单',
  JUG: '打野',
  MID: '中单',
  ADC: '下路',
  SUP: '辅助'
}

// 天赋标签名称映射
export const TALENT_NAMES: Record<PlayerTalent, string> = {
  GENIUS: '天才',
  NORMAL: '一般',
  ORDINARY: '平庸'
}

// 天赋成长速度（每赛季能力值增长）
export const TALENT_GROWTH: Record<PlayerTalent, number> = {
  GENIUS: 3,
  NORMAL: 2,
  ORDINARY: 1
}

// 选手状态因子（用于动态计算 condition）
export interface PlayerFormFactors {
  playerId: string
  formCycle: number              // 状态周期位置 (0-100)，用于计算正弦波
  momentum: number               // 动能 (-5 ~ +5)，连胜+1，连败-1
  lastPerformance: number        // 上场实际发挥值
  lastMatchWon: boolean          // 上场是否获胜
  gamesSinceRest: number         // 连续比赛场次（用于疲劳计算）
}

// 年龄对应的 condition 范围
export const CONDITION_RANGE_BY_AGE: Record<string, [number, number]> = {
  'young': [-5, 8],    // ≤24岁：高波动，高上限
  'prime': [-3, 3],    // 25-29岁：稳定期
  'veteran': [0, 2]    // ≥30岁：老将，稳定但上限低
}

// 获取年龄段
export function getAgeGroup(age: number): 'young' | 'prime' | 'veteran' {
  if (age <= 24) return 'young'
  if (age <= 29) return 'prime'
  return 'veteran'
}

// ==================== 特性系统 ====================

// 特性类型
export type TraitType =
  | 'clutch'           // 大赛型：季后赛/国际赛 condition +3
  | 'slow_starter'     // 慢热型：第1局 -2，第3+局 +2
  | 'fast_starter'     // 快枪手：第1局 +2，第3+局 -1
  | 'explosive'        // 爆发型：stability -15，上限 +5
  | 'consistent'       // 稳定型：stability +10，上限 -3
  | 'comeback_king'    // 逆风王：落后时 condition +3
  | 'tilter'           // 顺风浪：领先 -2，落后 -3
  | 'mental_fortress'  // 心态大师：momentum 效果减半
  | 'fragile'          // 玻璃心：输了 momentum -2
  | 'ironman'          // 铁人：无疲劳惩罚
  | 'volatile'         // 状态敏感：波动 ×1.5
  | 'rising_star'      // 新星：首赛季 ability +3
  | 'veteran'          // 老将风范：30岁后 stability +15
  | 'team_leader'      // 团队核心：队友 condition +1

// 特性信息
export interface TraitInfo {
  type: TraitType
  name: string
  description: string
  rarity: number      // 1-5
  isNegative: boolean
}

// 特性配置表
export const TRAIT_CONFIG: Record<TraitType, TraitInfo> = {
  clutch: {
    type: 'clutch',
    name: '大赛型',
    description: '在季后赛和国际赛中状态更好',
    rarity: 4,
    isNegative: false
  },
  slow_starter: {
    type: 'slow_starter',
    name: '慢热型',
    description: '系列赛开局较慢，但后期渐入佳境',
    rarity: 2,
    isNegative: false
  },
  fast_starter: {
    type: 'fast_starter',
    name: '快枪手',
    description: '系列赛开局强势，但后期可能疲软',
    rarity: 2,
    isNegative: false
  },
  explosive: {
    type: 'explosive',
    name: '爆发型',
    description: '发挥波动大，但巅峰更高',
    rarity: 3,
    isNegative: false
  },
  consistent: {
    type: 'consistent',
    name: '稳定型',
    description: '发挥稳定，但上限略低',
    rarity: 2,
    isNegative: false
  },
  comeback_king: {
    type: 'comeback_king',
    name: '逆风王',
    description: '落后时愈战愈勇',
    rarity: 4,
    isNegative: false
  },
  tilter: {
    type: 'tilter',
    name: '顺风浪',
    description: '心态容易受比分影响',
    rarity: 1,
    isNegative: true
  },
  mental_fortress: {
    type: 'mental_fortress',
    name: '心态大师',
    description: '心态稳定，不受连胜连败影响',
    rarity: 4,
    isNegative: false
  },
  fragile: {
    type: 'fragile',
    name: '玻璃心',
    description: '输了比赛心态下滑更快',
    rarity: 1,
    isNegative: true
  },
  ironman: {
    type: 'ironman',
    name: '铁人',
    description: '不受连续比赛疲劳影响',
    rarity: 3,
    isNegative: false
  },
  volatile: {
    type: 'volatile',
    name: '状态敏感',
    description: '状态波动比常人更大',
    rarity: 2,
    isNegative: true
  },
  rising_star: {
    type: 'rising_star',
    name: '新星',
    description: '新人赛季潜力爆发',
    rarity: 3,
    isNegative: false
  },
  veteran: {
    type: 'veteran',
    name: '老将风范',
    description: '老将经验丰富，发挥更稳',
    rarity: 3,
    isNegative: false
  },
  team_leader: {
    type: 'team_leader',
    name: '团队核心',
    description: '带动队友发挥',
    rarity: 5,
    isNegative: false
  }
}

// 获取特性显示名称（兼容多种格式）
export function getTraitName(traitType: TraitType | string): string {
  // 先尝试直接匹配
  if (TRAIT_CONFIG[traitType as TraitType]?.name) {
    return TRAIT_CONFIG[traitType as TraitType].name
  }
  // 尝试转换为 snake_case 后匹配（处理 PascalCase 如 "Veteran" -> "veteran"）
  const snakeCase = String(traitType)
    .replace(/([A-Z])/g, '_$1')
    .toLowerCase()
    .replace(/^_/, '') as TraitType
  return TRAIT_CONFIG[snakeCase]?.name || traitType
}

// 获取特性描述（兼容多种格式）
export function getTraitDescription(traitType: TraitType | string): string {
  // 先尝试直接匹配
  if (TRAIT_CONFIG[traitType as TraitType]?.description) {
    return TRAIT_CONFIG[traitType as TraitType].description
  }
  // 尝试转换为 snake_case 后匹配
  const snakeCase = String(traitType)
    .replace(/([A-Z])/g, '_$1')
    .toLowerCase()
    .replace(/^_/, '') as TraitType
  return TRAIT_CONFIG[snakeCase]?.description || ''
}

// 根据稀有度获取颜色
export function getTraitRarityColor(traitType: TraitType): string {
  const rarity = TRAIT_CONFIG[traitType]?.rarity || 1
  switch (rarity) {
    case 5: return '#ff8c00'  // 传奇橙
    case 4: return '#a855f7'  // 史诗紫
    case 3: return '#3b82f6'  // 稀有蓝
    case 2: return '#22c55e'  // 普通绿
    default: return '#6b7280' // 灰色
  }
}
