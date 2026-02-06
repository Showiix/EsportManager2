/**
 * 统一金额格式化工具
 *
 * 游戏中所有金额（身价、薪资、转会费、奖金、预算等）都应使用此工具格式化
 * 数据库存储单位统一为「元」，显示时自动转换为合适的单位
 */

export interface FormatMoneyOptions {
  /** 是否显示单位（默认 true） */
  showUnit?: boolean
  /** 小数位数（默认：亿2位、千万1位、万0位） */
  decimals?: number
  /** 是否使用「千万」作为中间单位（默认 true） */
  useQianWan?: boolean
  /** 强制使用指定单位 */
  forceUnit?: 'yuan' | 'wan' | 'qianwan' | 'yi'
}

/**
 * 格式化金额（输入单位：元）
 *
 * @param amount - 金额（单位：元）
 * @param options - 格式化选项
 * @returns 格式化后的字符串
 *
 * @example
 * formatMoney(150000000)     // "1.50亿"
 * formatMoney(35000000)      // "3500万"  或 "3.5千万"（取决于 useQianWan）
 * formatMoney(1200000)       // "120万"
 * formatMoney(8000)          // "8000元"
 * formatMoney(-50000000)     // "-5000万"
 */
export function formatMoney(amount: number, options: FormatMoneyOptions = {}): string {
  const {
    showUnit = true,
    decimals,
    useQianWan = false,
    forceUnit,
  } = options

  const absAmount = Math.abs(amount)
  const sign = amount < 0 ? '-' : ''

  let value: number
  let unit: string

  if (forceUnit) {
    // 强制使用指定单位
    switch (forceUnit) {
      case 'yi':
        value = absAmount / 100000000
        unit = '亿'
        break
      case 'qianwan':
        value = absAmount / 10000000
        unit = '千万'
        break
      case 'wan':
        value = absAmount / 10000
        unit = '万'
        break
      default:
        value = absAmount
        unit = '元'
    }
  } else {
    // 自动选择合适的单位
    if (absAmount >= 100000000) {
      // >= 1亿
      value = absAmount / 100000000
      unit = '亿'
    } else if (useQianWan && absAmount >= 10000000) {
      // >= 1千万（可选）
      value = absAmount / 10000000
      unit = '千万'
    } else if (absAmount >= 10000) {
      // >= 1万
      value = absAmount / 10000
      unit = '万'
    } else {
      value = absAmount
      unit = '元'
    }
  }

  // 确定小数位数
  let finalDecimals: number
  if (decimals !== undefined) {
    finalDecimals = decimals
  } else {
    // 默认规则：亿2位、千万1位、万0位、元0位
    if (unit === '亿') {
      finalDecimals = 2
    } else if (unit === '千万') {
      finalDecimals = 1
    } else {
      finalDecimals = 0
    }
  }

  const formattedValue = value.toFixed(finalDecimals)

  return showUnit ? `${sign}${formattedValue}${unit}` : `${sign}${formattedValue}`
}

/**
 * @deprecated 使用 formatMoney() 代替。所有后端金额现已统一为元。
 * 格式化金额（输入单位：万元）
 *
 * 用于后端返回数据单位为「万元」的场景
 *
 * @param amountInWan - 金额（单位：万元）
 * @param options - 格式化选项
 * @returns 格式化后的字符串
 *
 * @example
 * formatMoneyFromWan(15000)  // "1.50亿"
 * formatMoneyFromWan(3500)   // "3500万"
 * formatMoneyFromWan(120)    // "120万"
 */
export function formatMoneyFromWan(amountInWan: number, options: FormatMoneyOptions = {}): string {
  // 转换为元后调用标准格式化函数
  return formatMoney(amountInWan * 10000, options)
}

/**
 * 格式化身价（输入单位：元）
 *
 * 身价专用，默认不使用「千万」单位，保持简洁
 *
 * @param value - 身价（单位：元）
 * @returns 格式化后的字符串
 *
 * @example
 * formatValue(150000000)  // "1.50亿"
 * formatValue(35000000)   // "3500万"
 * formatValue(1200000)    // "120万"
 */
export function formatValue(value: number): string {
  return formatMoney(value, { useQianWan: false })
}

/**
 * 格式化薪资（输入单位：元）
 *
 * 薪资专用，添加「/年」后缀
 *
 * @param salary - 年薪（单位：元）
 * @param showPerYear - 是否显示「/年」（默认 true）
 * @returns 格式化后的字符串
 *
 * @example
 * formatSalary(8000000)       // "800万/年"
 * formatSalary(8000000, false) // "800万"
 */
export function formatSalary(salary: number, showPerYear = true): string {
  const formatted = formatMoney(salary, { useQianWan: false })
  return showPerYear ? `${formatted}/年` : formatted
}

/**
 * 格式化转会费（输入单位：元）
 *
 * 转会费专用，可选使用「千万」单位以更精确显示大额转会
 *
 * @param fee - 转会费（单位：元）
 * @returns 格式化后的字符串
 *
 * @example
 * formatTransferFee(150000000)  // "1.50亿"
 * formatTransferFee(35000000)   // "3500万"
 */
export function formatTransferFee(fee: number): string {
  return formatMoney(fee, { useQianWan: false })
}

/**
 * 格式化预算/余额（输入单位：元）
 *
 * 财务相关金额，支持负数显示
 *
 * @param amount - 金额（单位：元）
 * @returns 格式化后的字符串
 *
 * @example
 * formatBudget(500000000)   // "5.00亿"
 * formatBudget(-30000000)   // "-3000万"
 */
export function formatBudget(amount: number): string {
  return formatMoney(amount, { useQianWan: false })
}

/**
 * 解析格式化的金额字符串，返回元
 *
 * @param formatted - 格式化后的金额字符串
 * @returns 金额（单位：元），解析失败返回 0
 *
 * @example
 * parseMoney("1.50亿")   // 150000000
 * parseMoney("3500万")   // 35000000
 * parseMoney("-800万")   // -8000000
 */
export function parseMoney(formatted: string): number {
  if (!formatted) return 0

  const isNegative = formatted.startsWith('-')
  const cleanStr = formatted.replace(/[^0-9.]/g, '')
  const value = parseFloat(cleanStr)

  if (isNaN(value)) return 0

  let multiplier = 1
  if (formatted.includes('亿')) {
    multiplier = 100000000
  } else if (formatted.includes('千万')) {
    multiplier = 10000000
  } else if (formatted.includes('万')) {
    multiplier = 10000
  }

  return (isNegative ? -1 : 1) * value * multiplier
}
