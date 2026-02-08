import { describe, it, expect } from 'vitest'
import { PlayerEngine } from '../PlayerEngine'
import type { Player } from '@/types/player'

describe('PlayerEngine', () => {
  // ==================== gaussianRandom ====================

  describe('gaussianRandom', () => {
    it('should have mean close to 0 over many samples', () => {
      const n = 10000
      let sum = 0
      for (let i = 0; i < n; i++) {
        sum += PlayerEngine.gaussianRandom()
      }
      const mean = sum / n
      expect(Math.abs(mean)).toBeLessThan(0.1)
    })

    it('should have standard deviation close to 1 over many samples', () => {
      const n = 10000
      const values: number[] = []
      for (let i = 0; i < n; i++) {
        values.push(PlayerEngine.gaussianRandom())
      }
      const mean = values.reduce((a, b) => a + b, 0) / n
      const variance = values.reduce((sum, v) => sum + (v - mean) ** 2, 0) / n
      const stdDev = Math.sqrt(variance)
      expect(stdDev).toBeGreaterThan(0.9)
      expect(stdDev).toBeLessThan(1.1)
    })
  })

  // ==================== calculateStabilitySigma ====================

  describe('calculateStabilitySigma', () => {
    it('should return 0 for stability=100', () => {
      expect(PlayerEngine.calculateStabilitySigma(100)).toBe(0)
    })

    it('should return 4 for stability=60', () => {
      expect(PlayerEngine.calculateStabilitySigma(60)).toBe(4)
    })

    it('should return 10 for stability=0', () => {
      expect(PlayerEngine.calculateStabilitySigma(0)).toBe(10)
    })

    it('should clamp negative stability to 0', () => {
      expect(PlayerEngine.calculateStabilitySigma(-10)).toBe(10)
    })

    it('should clamp stability above 100', () => {
      expect(PlayerEngine.calculateStabilitySigma(120)).toBe(0)
    })
  })

  // ==================== calculatePerformance ====================

  describe('calculatePerformance', () => {
    const mockPlayer: Player = {
      id: '1',
      gameId: 'Faker',
      name: 'Lee Sang-hyeok',
      teamId: '1',
      position: 'MID',
      regionId: '1',
      ability: 80,
      potential: 90,
      stability: 70,
      condition: 2,
      age: 27,
      tag: 'GENIUS',
    }

    it('should return actualAbility within clamped range', () => {
      for (let i = 0; i < 100; i++) {
        const result = PlayerEngine.calculatePerformance(mockPlayer)
        const min = Math.max(0, mockPlayer.ability - 15) // 65
        const max = Math.min(100, mockPlayer.ability + 10) // 90
        expect(result.actualAbility).toBeGreaterThanOrEqual(min)
        expect(result.actualAbility).toBeLessThanOrEqual(max)
      }
    })

    it('should include condition bonus in calculation', () => {
      // With very high stability (low noise), the result should be close to ability + condition
      const stablePlayer: Player = { ...mockPlayer, stability: 100, condition: 5 }
      const results: number[] = []
      for (let i = 0; i < 100; i++) {
        results.push(PlayerEngine.calculatePerformance(stablePlayer).actualAbility)
      }
      const mean = results.reduce((a, b) => a + b, 0) / results.length
      // With stability=100, sigma=0, so actualAbility should be exactly ability + condition = 85
      expect(mean).toBeCloseTo(85, 0)
    })

    it('should return correct player metadata', () => {
      const result = PlayerEngine.calculatePerformance(mockPlayer)
      expect(result.playerId).toBe('1')
      expect(result.playerName).toBe('Faker')
      expect(result.position).toBe('MID')
      expect(result.baseAbility).toBe(80)
      expect(result.conditionBonus).toBe(2)
    })
  })

  // ==================== getBaseStabilityByAge ====================

  describe('getBaseStabilityByAge', () => {
    it('should return lower stability for younger players', () => {
      const young = PlayerEngine.getBaseStabilityByAge(20)
      const prime = PlayerEngine.getBaseStabilityByAge(27)
      const veteran = PlayerEngine.getBaseStabilityByAge(33)

      expect(young).toBeLessThan(prime)
      expect(prime).toBeLessThan(veteran)
    })

    it('should return 55 for under 18', () => {
      expect(PlayerEngine.getBaseStabilityByAge(16)).toBe(55)
    })

    it('should follow age-based ranges', () => {
      // 18-24: 60-75
      expect(PlayerEngine.getBaseStabilityByAge(18)).toBeGreaterThanOrEqual(60)
      expect(PlayerEngine.getBaseStabilityByAge(24)).toBeLessThanOrEqual(75)
      // 25-29: 75-85
      expect(PlayerEngine.getBaseStabilityByAge(25)).toBeGreaterThanOrEqual(75)
      expect(PlayerEngine.getBaseStabilityByAge(29)).toBeLessThanOrEqual(85)
      // 30-36: 85-95
      expect(PlayerEngine.getBaseStabilityByAge(30)).toBeGreaterThanOrEqual(85)
      expect(PlayerEngine.getBaseStabilityByAge(36)).toBeLessThanOrEqual(94)
    })

    it('should return 95 for age > 36', () => {
      expect(PlayerEngine.getBaseStabilityByAge(40)).toBe(95)
    })
  })

  // ==================== getConditionRange ====================

  describe('getConditionRange', () => {
    it('should return wider range for young players', () => {
      const young = PlayerEngine.getConditionRange(20)
      expect(young.min).toBe(-5)
      expect(young.max).toBe(8)
    })

    it('should return moderate range for prime age', () => {
      const prime = PlayerEngine.getConditionRange(27)
      expect(prime.min).toBe(-3)
      expect(prime.max).toBe(3)
    })

    it('should return narrow range for veterans', () => {
      const vet = PlayerEngine.getConditionRange(32)
      expect(vet.min).toBe(0)
      expect(vet.max).toBe(2)
    })

    it('should have wider range for young vs veterans', () => {
      const young = PlayerEngine.getConditionRange(20)
      const vet = PlayerEngine.getConditionRange(32)
      const youngSpan = young.max - young.min
      const vetSpan = vet.max - vet.min
      expect(youngSpan).toBeGreaterThan(vetSpan)
    })
  })
})
