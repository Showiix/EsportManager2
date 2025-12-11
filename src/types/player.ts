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
