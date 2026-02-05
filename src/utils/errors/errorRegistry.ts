/**
 * 错误码注册表
 *
 * 定义所有已知错误码及其用户友好信息。
 */

export type ErrorSeverity = 'low' | 'medium' | 'high' | 'critical'

export interface ErrorInfo {
  code: string
  title: string
  description: string
  suggestion: string
  docUrl?: string
  severity: ErrorSeverity
}

/**
 * 错误注册表
 */
export const ERROR_REGISTRY: Record<string, ErrorInfo> = {
  // ============================================================================
  // 转会系统错误 (TR)
  // ============================================================================
  'E-TR-B-001': {
    code: 'E-TR-B-001',
    title: '预算不足',
    description: '球队当前余额不足以完成此次转会',
    suggestion: '请先出售其他选手或等待赛季奖金发放后再尝试',
    docUrl: '/help/transfer#budget',
    severity: 'medium',
  },
  'E-TR-B-002': {
    code: 'E-TR-B-002',
    title: '选手不可转会',
    description: '该选手当前状态不允许转会',
    suggestion: '选手可能在合同保护期内，或已被其他球队锁定',
    docUrl: '/help/transfer#availability',
    severity: 'low',
  },
  'E-TR-B-003': {
    code: 'E-TR-B-003',
    title: '合同冲突',
    description: '选手的当前合同与新合同存在冲突',
    suggestion: '请先解除选手的现有合同',
    severity: 'medium',
  },
  'E-TR-B-004': {
    code: 'E-TR-B-004',
    title: '薪资上限超出',
    description: '签约该选手后将超出球队薪资上限',
    suggestion: '请先降低其他选手薪资或出售高薪选手',
    docUrl: '/help/finance#salary-cap',
    severity: 'medium',
  },
  'E-TR-B-005': {
    code: 'E-TR-B-005',
    title: '转会窗口已关闭',
    description: '当前不在转会窗口期',
    suggestion: '请等待下一个转会窗口期再进行转会操作',
    severity: 'low',
  },
  'E-TR-B-006': {
    code: 'E-TR-B-006',
    title: '阵容已满',
    description: '球队阵容人数已达上限',
    suggestion: '请先出售或解约部分选手',
    severity: 'medium',
  },
  'E-TR-B-007': {
    code: 'E-TR-B-007',
    title: '选手拒绝',
    description: '选手拒绝了您的报价',
    suggestion: '尝试提高薪资待遇，或选择对球队更感兴趣的选手',
    severity: 'low',
  },
  'E-TR-V-001': {
    code: 'E-TR-V-001',
    title: '无效报价',
    description: '报价参数无效',
    suggestion: '请检查报价金额和合同年限是否合理',
    severity: 'low',
  },

  // ============================================================================
  // 比赛系统错误 (MT)
  // ============================================================================
  'E-MT-D-001': {
    code: 'E-MT-D-001',
    title: '比赛不存在',
    description: '找不到指定的比赛',
    suggestion: '请刷新页面后重试',
    severity: 'medium',
  },
  'E-MT-B-001': {
    code: 'E-MT-B-001',
    title: '比赛已进行',
    description: '该比赛已经完成，无法重复模拟',
    suggestion: '请选择未进行的比赛',
    severity: 'low',
  },
  'E-MT-B-002': {
    code: 'E-MT-B-002',
    title: '队伍不完整',
    description: '参赛队伍阵容不满足比赛要求',
    suggestion: '请确保每支队伍都有5名首发选手',
    severity: 'high',
  },
  'E-MT-B-003': {
    code: 'E-MT-B-003',
    title: '赛事未就绪',
    description: '赛事尚未完成初始化',
    suggestion: '请先初始化当前阶段',
    severity: 'medium',
  },

  // ============================================================================
  // 时间系统错误 (TM)
  // ============================================================================
  'E-TM-B-001': {
    code: 'E-TM-B-001',
    title: '阶段未完成',
    description: '当前阶段尚未完成，无法推进到下一阶段',
    suggestion: '请先完成当前阶段的所有比赛或任务',
    severity: 'medium',
  },
  'E-TM-B-002': {
    code: 'E-TM-B-002',
    title: '无效阶段切换',
    description: '无法从当前阶段切换到目标阶段',
    suggestion: '请按照正确的阶段顺序进行',
    severity: 'medium',
  },
  'E-TM-D-001': {
    code: 'E-TM-D-001',
    title: '存档不存在',
    description: '找不到指定的存档',
    suggestion: '请选择一个有效的存档或创建新存档',
    severity: 'high',
  },

  // ============================================================================
  // 数据库错误 (DB)
  // ============================================================================
  'E-DB-S-001': {
    code: 'E-DB-S-001',
    title: '数据库连接失败',
    description: '无法连接到游戏数据库',
    suggestion: '请尝试重启游戏。如问题持续，请检查游戏文件完整性',
    docUrl: '/help/troubleshoot#database',
    severity: 'critical',
  },
  'E-DB-S-002': {
    code: 'E-DB-S-002',
    title: '数据库查询失败',
    description: '执行数据库查询时发生错误',
    suggestion: '请尝试重试操作。如问题持续，请联系技术支持',
    severity: 'high',
  },
  'E-DB-D-001': {
    code: 'E-DB-D-001',
    title: '存档数据损坏',
    description: '存档文件可能已损坏',
    suggestion: '请尝试加载备份存档，或联系技术支持',
    docUrl: '/help/troubleshoot#save-corruption',
    severity: 'critical',
  },

  // ============================================================================
  // 系统错误 (SY)
  // ============================================================================
  'E-SY-S-001': {
    code: 'E-SY-S-001',
    title: '未知错误',
    description: '发生了未预期的错误',
    suggestion: '请查看日志文件获取更多信息，或联系技术支持',
    docUrl: '/help/troubleshoot#unknown',
    severity: 'high',
  },
  'E-SY-S-002': {
    code: 'E-SY-S-002',
    title: '配置错误',
    description: '系统配置不正确',
    suggestion: '请检查配置文件或重新安装游戏',
    severity: 'high',
  },
  'E-SY-S-003': {
    code: 'E-SY-S-003',
    title: '初始化失败',
    description: '系统初始化失败',
    suggestion: '请尝试重启游戏',
    severity: 'critical',
  },
  'E-SY-V-001': {
    code: 'E-SY-V-001',
    title: '无效参数',
    description: '提供的参数无效',
    suggestion: '请检查输入是否正确',
    severity: 'low',
  },
}

/**
 * 根据错误码获取错误信息
 */
export function getErrorInfo(code: string): ErrorInfo | undefined {
  return ERROR_REGISTRY[code]
}

/**
 * 检查是否为已知错误
 */
export function isKnownError(code: string): boolean {
  return code in ERROR_REGISTRY
}

/**
 * 获取错误严重程度颜色
 */
export function getSeverityColor(severity: ErrorSeverity): string {
  switch (severity) {
    case 'low':
      return '#909399'
    case 'medium':
      return '#E6A23C'
    case 'high':
      return '#F56C6C'
    case 'critical':
      return '#FF0000'
    default:
      return '#909399'
  }
}
