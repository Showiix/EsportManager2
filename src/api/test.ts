import { invokeCommand } from './client'

// ========================================
// Test Functions
// ========================================

export interface TestMatchResult {
  home_score: number
  away_score: number
  winner: string
  games: {
    game_number: number
    home_performance: number
    away_performance: number
    winner: string
  }[]
  win_probability: number
}

export const testApi = {
  simulateTestMatch: (homePower: number, awayPower: number, format: string) =>
    invokeCommand<TestMatchResult>('simulate_test_match', {
      homePower,
      awayPower,
      format
    }),
}
