import { describe, it, expect } from 'vitest'
import { PowerEngine } from '../PowerEngine'
import type { Player, PlayerPerformance } from '@/types/player'

describe('PowerEngine', () => {
  // ==================== calculateTeamPower ====================

  describe('calculateTeamPower', () => {
    it('should return average of actualAbility values', () => {
      const performances: PlayerPerformance[] = [
        makePerformance('1', 70),
        makePerformance('2', 80),
        makePerformance('3', 60),
      ]
      const power = PowerEngine.calculateTeamPower(performances)
      expect(power).toBeCloseTo(70, 1)
    })

    it('should return 0 for empty array', () => {
      expect(PowerEngine.calculateTeamPower([])).toBe(0)
    })

    it('should handle single player', () => {
      const performances = [makePerformance('1', 85)]
      expect(PowerEngine.calculateTeamPower(performances)).toBe(85)
    })
  })

  // ==================== calculateImpactScores ====================

  describe('calculateImpactScores', () => {
    it('should set positive impact for above-average players', () => {
      const performances = [
        makePerformance('1', 80),
        makePerformance('2', 70),
        makePerformance('3', 60),
      ]
      const teamAvg = 70
      PowerEngine.calculateImpactScores(performances, teamAvg)

      expect(performances[0].impactScore).toBeGreaterThan(0)
      expect(performances[1].impactScore).toBe(0)
      expect(performances[2].impactScore).toBeLessThan(0)
    })

    it('should round impact scores to 1 decimal', () => {
      const performances = [makePerformance('1', 73.33)]
      PowerEngine.calculateImpactScores(performances, 70)
      // (73.33 - 70) = 3.33, rounded to 3.3
      expect(performances[0].impactScore).toBe(3.3)
    })
  })

  // ==================== simulateMatch ====================

  describe('simulateMatch', () => {
    const teamAPlayers = makeTeamPlayers('A', 75)
    const teamBPlayers = makeTeamPlayers('B', 75)

    it('should return valid BO3 result (2-3 total games)', () => {
      const result = PowerEngine.simulateMatch(
        'A', 'Team A', teamAPlayers,
        'B', 'Team B', teamBPlayers,
        3
      )

      const totalGames = result.finalScoreA + result.finalScoreB
      expect(totalGames).toBeGreaterThanOrEqual(2)
      expect(totalGames).toBeLessThanOrEqual(3)
      expect(result.games.length).toBe(totalGames)

      // Winner should have 2 wins
      if (result.winnerId === 'A') {
        expect(result.finalScoreA).toBe(2)
      } else {
        expect(result.finalScoreB).toBe(2)
      }
    })

    it('should return valid BO5 result (3-5 total games)', () => {
      const result = PowerEngine.simulateMatch(
        'A', 'Team A', teamAPlayers,
        'B', 'Team B', teamBPlayers,
        5
      )

      const totalGames = result.finalScoreA + result.finalScoreB
      expect(totalGames).toBeGreaterThanOrEqual(3)
      expect(totalGames).toBeLessThanOrEqual(5)

      // Winner should have 3 wins
      if (result.winnerId === 'A') {
        expect(result.finalScoreA).toBe(3)
      } else {
        expect(result.finalScoreB).toBe(3)
      }
    })

    it('should return valid BO1 result', () => {
      const result = PowerEngine.simulateMatch(
        'A', 'Team A', teamAPlayers,
        'B', 'Team B', teamBPlayers,
        1
      )

      expect(result.games.length).toBe(1)
      expect(result.finalScoreA + result.finalScoreB).toBe(1)
    })

    it('should identify MVP across all games', () => {
      const result = PowerEngine.simulateMatch(
        'A', 'Team A', teamAPlayers,
        'B', 'Team B', teamBPlayers,
        3
      )

      // MVP should be defined
      expect(result.mvpPlayerId).toBeDefined()
      expect(result.mvpPlayerName).toBeDefined()
    })

    it('should have stronger team winning more often over many matches', () => {
      const strongPlayers = makeTeamPlayers('A', 85)
      const weakPlayers = makeTeamPlayers('B', 65)
      let strongWins = 0
      const n = 200

      for (let i = 0; i < n; i++) {
        const result = PowerEngine.simulateMatch(
          'A', 'Strong', strongPlayers,
          'B', 'Weak', weakPlayers,
          3
        )
        if (result.winnerId === 'A') strongWins++
      }

      // With 20-point difference, strong team should win >> 50%
      expect(strongWins / n).toBeGreaterThan(0.65)
    })
  })
})

// ==================== Helpers ====================

function makePerformance(id: string, actualAbility: number): PlayerPerformance {
  return {
    playerId: id,
    playerName: `Player${id}`,
    position: 'MID',
    teamId: '1',
    baseAbility: actualAbility,
    conditionBonus: 0,
    stabilityNoise: 0,
    actualAbility,
    impactScore: 0,
  }
}

function makeTeamPlayers(prefix: string, ability: number): Player[] {
  const positions: Array<'TOP' | 'JUG' | 'MID' | 'ADC' | 'SUP'> = ['TOP', 'JUG', 'MID', 'ADC', 'SUP']
  return positions.map((pos, i) => ({
    id: `${prefix}${i}`,
    gameId: `${prefix}_${pos}`,
    name: `${prefix} ${pos}`,
    teamId: prefix,
    position: pos,
    regionId: '1',
    ability,
    potential: ability + 5,
    stability: 70,
    condition: 0,
    age: 24,
    tag: 'NORMAL' as const,
  }))
}
