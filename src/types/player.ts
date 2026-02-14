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

  // BP加成
  bpModifier?: number          // BP加成百分比（来自英雄熟练度和版本强势）

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
  yearlyTopScore: number
  bigStageScore?: number
  hasInternational?: boolean
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
  | 'finals_killer'    // 决赛杀手：决赛中能力爆发
  | 'regular_king'     // 常规赛之王：常规赛出色，季后赛略下滑
  | 'win_streak'       // 连胜狂魔：连胜时越打越强
  | 'explosive'        // 爆发型：stability -15，上限 +5
  | 'consistent'       // 稳定型：stability +10，上限 -3
  | 'streaky'          // 时好时坏：状态大起大落
  | 'big_game'         // 大场面选手：重要比赛发挥出色
  | 'choker'           // 关键掉链子：关键比赛掉链子
  | 'comeback_king'    // 逆风王：落后时 condition +3
  | 'tilter'           // 顺风浪：领先 -2，落后 -3
  | 'mental_fortress'  // 心态大师：momentum 效果减半
  | 'fragile'          // 玻璃心：输了 momentum -2
  | 'gambler'          // 赌徒：发挥极端
  | 'pressure_player'  // 抗压选手：被逼到绝境时爆发
  | 'complacent'       // 安于现状：领先时容易松懈
  | 'ironman'          // 铁人：无疲劳惩罚
  | 'volatile'         // 状态敏感：波动 ×1.5
  | 'endurance'        // 持久战型：长系列赛不疲劳
  | 'sprinter'         // 短跑型：短赛制爆发力强
  | 'night_owl'        // 夜猫子：赛季后半程状态更好
  | 'peak_form'        // 巅峰状态：巅峰期波动极小
  | 'rising_star'      // 新星：首赛季 ability +3
  | 'veteran'          // 老将风范：30岁后 stability +15
  | 'team_leader'      // 团队核心：队友 condition +1
  | 'lone_wolf'        // 独狼：单打独斗强，配合弱
  | 'supportive'       // 辅助型领袖：让队友变得更好
  | 'troublemaker'     // 刺头：实力出众但影响氛围
  | 'mentor'           // 导师：帮助年轻队友成长
  | 'late_blocker'     // 大器晚成：成长期和巅峰期延长
  | 'prodigy'          // 神童：年少成名，后期放缓
  | 'resilient'        // 抗衰老：衰退速度减半
  | 'glass_cannon'     // 易碎：巅峰更高但衰退更快
  | 'low_ceiling'      // 低天花板：潜力有限
  | 'limitless'        // 无限潜力：成长不设上限
  | 'battle_tested'    // 百战之躯：身经百战
  | 'peak_age'         // 黄金年龄：巅峰期状态优异
  | 'early_decline'    // 早衰：较早开始衰退
  | 'perfectionist'    // 完美主义者：效力同队久表现好
  | 'adaptable'        // 适应力强：新环境适应快
  | 'world_stage'      // 世界舞台：国际大赛爆发
  | 'group_stage_expert' // 小组赛专家
  | 'knockout_specialist' // 淘汰赛专家
  | 'cross_region'     // 跨赛区适应
  | 'tournament_horse' // 赛事铁马：多赛事不掉状态

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
  clutch: { type: 'clutch', name: '大赛型', description: '在季后赛和国际赛中状态更好', rarity: 4, isNegative: false },
  slow_starter: { type: 'slow_starter', name: '慢热型', description: '系列赛开局较慢，但后期渐入佳境', rarity: 2, isNegative: false },
  fast_starter: { type: 'fast_starter', name: '快枪手', description: '系列赛开局强势，但后期可能疲软', rarity: 2, isNegative: false },
  finals_killer: { type: 'finals_killer', name: '决赛杀手', description: '决赛中能力爆发', rarity: 5, isNegative: false },
  regular_king: { type: 'regular_king', name: '常规赛之王', description: '常规赛表现稳定出色，季后赛略有下滑', rarity: 3, isNegative: false },
  win_streak: { type: 'win_streak', name: '连胜狂魔', description: '连胜时越打越强', rarity: 3, isNegative: false },
  explosive: { type: 'explosive', name: '爆发型', description: '发挥波动大，但巅峰更高', rarity: 3, isNegative: false },
  consistent: { type: 'consistent', name: '稳定型', description: '发挥稳定，但上限略低', rarity: 2, isNegative: false },
  streaky: { type: 'streaky', name: '时好时坏', description: '状态大起大落，好坏交替', rarity: 1, isNegative: true },
  big_game: { type: 'big_game', name: '大场面选手', description: '重要比赛发挥出色', rarity: 4, isNegative: false },
  choker: { type: 'choker', name: '关键掉链子', description: '关键比赛掉链子', rarity: 1, isNegative: true },
  comeback_king: { type: 'comeback_king', name: '逆风王', description: '落后时愈战愈勇', rarity: 4, isNegative: false },
  tilter: { type: 'tilter', name: '顺风浪', description: '心态容易受比分影响', rarity: 1, isNegative: true },
  mental_fortress: { type: 'mental_fortress', name: '心态大师', description: '心态稳定，不受连胜连败影响', rarity: 4, isNegative: false },
  fragile: { type: 'fragile', name: '玻璃心', description: '输了比赛心态下滑更快', rarity: 1, isNegative: true },
  gambler: { type: 'gambler', name: '赌徒', description: '发挥极端，要么超神要么超鬼', rarity: 2, isNegative: true },
  pressure_player: { type: 'pressure_player', name: '抗压选手', description: '被逼到绝境时爆发', rarity: 3, isNegative: false },
  complacent: { type: 'complacent', name: '安于现状', description: '领先时容易松懈放水', rarity: 1, isNegative: true },
  ironman: { type: 'ironman', name: '铁人', description: '不受连续比赛疲劳影响', rarity: 3, isNegative: false },
  volatile: { type: 'volatile', name: '状态敏感', description: '状态波动比常人更大', rarity: 2, isNegative: true },
  endurance: { type: 'endurance', name: '持久战型', description: '长系列赛体力充沛，不会疲劳下滑', rarity: 3, isNegative: false },
  sprinter: { type: 'sprinter', name: '短跑型', description: '短赛制爆发力强，BO5后半段下降', rarity: 2, isNegative: false },
  night_owl: { type: 'night_owl', name: '夜猫子', description: '赛季后半程状态更好', rarity: 2, isNegative: false },
  peak_form: { type: 'peak_form', name: '巅峰状态', description: '巅峰期状态波动极小', rarity: 4, isNegative: false },
  rising_star: { type: 'rising_star', name: '新星', description: '新人赛季潜力爆发', rarity: 3, isNegative: false },
  veteran: { type: 'veteran', name: '老将风范', description: '老将经验丰富，发挥更稳', rarity: 3, isNegative: false },
  team_leader: { type: 'team_leader', name: '团队核心', description: '带动队友发挥', rarity: 5, isNegative: false },
  lone_wolf: { type: 'lone_wolf', name: '独狼', description: '单打独斗能力强，但不擅长配合', rarity: 3, isNegative: false },
  supportive: { type: 'supportive', name: '辅助型领袖', description: '让队友变得更好', rarity: 4, isNegative: false },
  troublemaker: { type: 'troublemaker', name: '刺头', description: '实力出众但影响队伍氛围', rarity: 1, isNegative: true },
  mentor: { type: 'mentor', name: '导师', description: '帮助年轻队友成长更快', rarity: 4, isNegative: false },
  late_blocker: { type: 'late_blocker', name: '大器晚成', description: '大器晚成，成长期和巅峰期延长2年', rarity: 4, isNegative: false },
  prodigy: { type: 'prodigy', name: '神童', description: '年少成名，但后期成长放缓', rarity: 4, isNegative: false },
  resilient: { type: 'resilient', name: '抗衰老', description: '身体素质出众，衰退速度减半', rarity: 4, isNegative: false },
  glass_cannon: { type: 'glass_cannon', name: '易碎', description: '巅峰更高但衰退更快', rarity: 2, isNegative: true },
  low_ceiling: { type: 'low_ceiling', name: '低天花板', description: '潜力有限，能力难以突破', rarity: 1, isNegative: true },
  limitless: { type: 'limitless', name: '无限潜力', description: '无限潜力，成长不设上限', rarity: 5, isNegative: false },
  battle_tested: { type: 'battle_tested', name: '百战之躯', description: '身经百战，经验丰富', rarity: 3, isNegative: false },
  peak_age: { type: 'peak_age', name: '黄金年龄', description: '巅峰期状态优异', rarity: 4, isNegative: false },
  early_decline: { type: 'early_decline', name: '早衰', description: '较早开始能力衰退', rarity: 1, isNegative: true },
  perfectionist: { type: 'perfectionist', name: '完美主义者', description: '效力同队久，表现越来越好', rarity: 3, isNegative: false },
  adaptable: { type: 'adaptable', name: '适应力强', description: '新环境适应快', rarity: 3, isNegative: false },
  world_stage: { type: 'world_stage', name: '世界舞台', description: '国际大赛中能力爆发', rarity: 5, isNegative: false },
  group_stage_expert: { type: 'group_stage_expert', name: '小组赛专家', description: '小组赛阶段表现出色', rarity: 3, isNegative: false },
  knockout_specialist: { type: 'knockout_specialist', name: '淘汰赛专家', description: '淘汰赛阶段能力爆发', rarity: 4, isNegative: false },
  cross_region: { type: 'cross_region', name: '跨赛区适应', description: '跨赛区转会后适应极快', rarity: 3, isNegative: false },
  tournament_horse: { type: 'tournament_horse', name: '赛事铁马', description: '多赛事连续作战不掉状态', rarity: 3, isNegative: false },
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
