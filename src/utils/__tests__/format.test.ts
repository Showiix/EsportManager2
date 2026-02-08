import { describe, it, expect } from 'vitest'
import { formatMoney, formatSalary, formatValue, formatBudget, parseMoney } from '../format'

describe('formatMoney', () => {
  // ==================== Auto unit selection ====================

  describe('auto unit selection', () => {
    it('should format >= 1亿 with 亿 unit', () => {
      expect(formatMoney(150000000)).toBe('1.50亿')
      expect(formatMoney(100000000)).toBe('1.00亿')
      expect(formatMoney(500000000)).toBe('5.00亿')
    })

    it('should format >= 1万 with 万 unit', () => {
      expect(formatMoney(35000000)).toBe('3500万')
      expect(formatMoney(1200000)).toBe('120万')
      expect(formatMoney(10000)).toBe('1万')
    })

    it('should format < 1万 with 元 unit', () => {
      expect(formatMoney(8000)).toBe('8000元')
      expect(formatMoney(500)).toBe('500元')
      expect(formatMoney(0)).toBe('0元')
    })

    it('should handle negative amounts', () => {
      expect(formatMoney(-50000000)).toBe('-5000万')
      expect(formatMoney(-150000000)).toBe('-1.50亿')
      expect(formatMoney(-8000)).toBe('-8000元')
    })
  })

  // ==================== Options ====================

  describe('options', () => {
    it('should hide unit when showUnit=false', () => {
      expect(formatMoney(150000000, { showUnit: false })).toBe('1.50')
    })

    it('should override decimals', () => {
      expect(formatMoney(150000000, { decimals: 0 })).toBe('2亿')
      expect(formatMoney(35000000, { decimals: 2 })).toBe('3500.00万')
    })

    it('should use 千万 unit when useQianWan=true', () => {
      expect(formatMoney(35000000, { useQianWan: true })).toBe('3.5千万')
      expect(formatMoney(15000000, { useQianWan: true })).toBe('1.5千万')
    })

    it('should force specific units', () => {
      expect(formatMoney(50000000, { forceUnit: 'yi' })).toBe('0.50亿')
      expect(formatMoney(50000000, { forceUnit: 'wan' })).toBe('5000万')
      expect(formatMoney(50000000, { forceUnit: 'yuan' })).toBe('50000000元')
    })
  })
})

// ==================== formatSalary ====================

describe('formatSalary', () => {
  it('should append /年 suffix by default', () => {
    expect(formatSalary(8000000)).toBe('800万/年')
  })

  it('should omit /年 when showPerYear=false', () => {
    expect(formatSalary(8000000, false)).toBe('800万')
  })
})

// ==================== formatValue ====================

describe('formatValue', () => {
  it('should not use 千万 unit', () => {
    expect(formatValue(35000000)).toBe('3500万')
    expect(formatValue(150000000)).toBe('1.50亿')
  })
})

// ==================== formatBudget ====================

describe('formatBudget', () => {
  it('should support negative values', () => {
    expect(formatBudget(-30000000)).toBe('-3000万')
    expect(formatBudget(500000000)).toBe('5.00亿')
  })
})

// ==================== parseMoney ====================

describe('parseMoney', () => {
  it('should parse 亿 back to yuan', () => {
    expect(parseMoney('1.50亿')).toBe(150000000)
  })

  it('should parse 千万 back to yuan', () => {
    expect(parseMoney('3.5千万')).toBe(35000000)
  })

  it('should parse 万 back to yuan', () => {
    expect(parseMoney('3500万')).toBe(35000000)
    expect(parseMoney('120万')).toBe(1200000)
  })

  it('should parse plain yuan (no unit multiplier)', () => {
    expect(parseMoney('8000元')).toBe(8000)
    expect(parseMoney('8000')).toBe(8000)
  })

  it('should handle negative values', () => {
    expect(parseMoney('-5000万')).toBe(-50000000)
    expect(parseMoney('-1.50亿')).toBe(-150000000)
  })

  it('should return 0 for empty or invalid input', () => {
    expect(parseMoney('')).toBe(0)
    expect(parseMoney('abc')).toBe(0)
  })
})
