/**
 * 选手引擎 - 计算每位选手每局的实际发挥
 *
 * 算法（策划案规定）：
 * - 稳定性标准差：σ = (100 - stability) / 10
 * - 实际能力 = 平均能力 + 状态加成 + 高斯噪声(0, σ)
 * - 钳位：[ability - 15, ability + 10]
 */

import type { Player, PlayerPerformance } from '@/types/player'
import { createLogger } from '@/utils/logger'

const logger = createLogger('PlayerEngine')

export class PlayerEngine {
  /**
   * Box-Muller 变换生成标准正态分布随机数
   * 返回值：均值为0，标准差为1的正态分布随机数
   */
  static gaussianRandom(): number {
    let u = 0
    let v = 0
    // 确保 u 和 v 不为0（log(0)未定义）
    while (u === 0) u = Math.random()
    while (v === 0) v = Math.random()
    return Math.sqrt(-2.0 * Math.log(u)) * Math.cos(2.0 * Math.PI * v)
  }

  /**
   * 计算稳定性对应的标准差
   * 稳定性越高，标准差越小，波动越小
   * @param stability 稳定性值 (0-100)
   * @returns 标准差 σ
   */
  static calculateStabilitySigma(stability: number): number {
    // σ = (100 - stability) / 10
    // stability = 100 → σ = 0（完全稳定）
    // stability = 60  → σ = 4（波动大）
    // stability = 0   → σ = 10（极不稳定）
    return (100 - Math.max(0, Math.min(100, stability))) / 10
  }

  /**
   * 计算单个选手在一局比赛中的实际发挥
   * @param player 选手数据
   * @returns 选手发挥数据
   */
  static calculatePerformance(player: Player): PlayerPerformance {
    // 1. 计算稳定性标准差
    const sigma = this.calculateStabilitySigma(player.stability)

    // 2. 生成高斯噪声
    const noise = this.gaussianRandom() * sigma

    // 3. 计算原始实际能力
    const rawAbility = player.ability + player.condition + noise

    // 4. 钳位到合理范围 [ability - 15, ability + 10]
    const minAbility = Math.max(0, player.ability - 15)
    const maxAbility = Math.min(100, player.ability + 10)
    const actualAbility = Math.max(minAbility, Math.min(maxAbility, rawAbility))

    return {
      playerId: player.id,
      playerName: player.gameId,
      position: player.position,
      teamId: player.teamId,
      baseAbility: player.ability,
      conditionBonus: player.condition,
      stabilityNoise: Math.round(noise * 100) / 100, // 保留2位小数
      actualAbility: Math.round(actualAbility * 10) / 10, // 保留1位小数
      impactScore: 0 // 稍后由 PowerEngine 计算
    }
  }

  /**
   * 批量计算队伍选手的发挥
   * @param players 选手数组（应该是5人）
   * @returns 选手发挥数组
   */
  static calculateTeamPerformances(players: Player[]): PlayerPerformance[] {
    if (players.length === 0) {
      logger.warn('没有选手数据')
      return []
    }
    if (players.length !== 5) {
      logger.warn('选手数量异常', { expected: 5, actual: players.length })
    }
    return players.map((player) => this.calculatePerformance(player))
  }

  /**
   * 根据年龄获取稳定性基础值
   * 策划案规定：
   * - 18-24岁（新星）：稳定性 60-75
   * - 25-29岁（巅峰）：稳定性 75-85
   * - 30-36岁（老将）：稳定性 85-95
   */
  static getBaseStabilityByAge(age: number): number {
    if (age < 18) return 55
    if (age <= 24) return 60 + Math.floor((age - 18) * 2.5) // 60-75
    if (age <= 29) return 75 + Math.floor((age - 25) * 2) // 75-85
    if (age <= 36) return 85 + Math.floor((age - 30) * 1.5) // 85-95
    return 95
  }

  /**
   * 根据年龄获取状态加成潜力
   * 策划案规定：
   * - 年轻选手（≤24岁）：+5 到 +8（"天神下凡"）
   * - 中生代（25-29岁）：±3以内
   * - 老将（≥30岁）：不超过+2，很少为负
   */
  static getConditionRange(age: number): { min: number; max: number } {
    if (age <= 24) return { min: -5, max: 8 }
    if (age <= 29) return { min: -3, max: 3 }
    return { min: 0, max: 2 }
  }

  /**
   * 随机生成选手当前状态
   */
  static generateRandomCondition(age: number): number {
    const range = this.getConditionRange(age)
    return Math.floor(Math.random() * (range.max - range.min + 1)) + range.min
  }
}
