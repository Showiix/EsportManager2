import axios from 'axios'

// 创建axios实例
const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器
apiClient.interceptors.request.use(
  (config) => {
    // 添加认证token等
    const token = localStorage.getItem('auth_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 数据字段转换函数 - 将后端蛇形命名转换为前端驼峰命名
function transformKeys(obj: any): any {
  if (Array.isArray(obj)) {
    return obj.map(item => transformKeys(item))
  }

  if (obj !== null && typeof obj === 'object') {
    const transformed: any = {}

    // 特殊字段映射
    const fieldMapping: Record<string, string> = {
      'power_rating': 'strength',
      'region_id': 'regionId',
      'short_name': 'shortName',
      'founded_date': 'foundedDate',
      'logo_url': 'logoUrl',
      'is_active': 'isActive',
      'total_matches': 'totalMatches',
      'total_wins': 'wins',
      'total_losses': 'losses',
      'net_round_difference': 'netRoundDifference',
      'created_at': 'createdAt',
      'updated_at': 'updatedAt',
      'region_name': 'regionName',
      'region_code': 'regionCode',
      'competition_id': 'competitionId',
      'home_team_id': 'homeTeamId',
      'away_team_id': 'awayTeamId',
      'scheduled_at': 'scheduledAt',
      'played_at': 'playedAt',
      'season_id': 'seasonId',
      'season_code': 'seasonCode',
      'season_name': 'seasonName',
      'season_year': 'seasonYear',
      'display_name': 'displayName',
      'competition_code': 'competitionCode',
      'new_id': 'newId',
      'new_season_id': 'newSeasonId',
      'max_teams': 'maxTeams',
      'start_date': 'startDate',
      'end_date': 'endDate',
      'scoring_rules': 'scoringRules',
      'team_id': 'teamId'
    }

    for (const key in obj) {
      const newKey = fieldMapping[key] || key
      transformed[newKey] = transformKeys(obj[key])
    }

    // 添加statistics字段（如果有相关数据）
    if (transformed.totalMatches !== undefined) {
      transformed.statistics = {
        totalMatches: transformed.totalMatches || 0,
        wins: transformed.wins || 0,
        losses: transformed.losses || 0,
        winRate: transformed.totalMatches > 0 ? (transformed.wins || 0) / transformed.totalMatches : 0,
        totalPoints: transformed.totalPoints || 0,
        seasonPoints: transformed.seasonPoints || 0,
        intercontinentalPoints: transformed.intercontinentalPoints || 0
      }
    }

    return transformed
  }

  return obj
}

// 响应拦截器
apiClient.interceptors.response.use(
  (response) => {
    // 转换数据字段
    if (response.data) {
      const transformed = {
        ...response.data,
        data: transformKeys(response.data.data)
      }
      return transformed
    }
    return response.data
  },
  (error) => {
    console.error('API Error:', error)
    return Promise.reject(error)
  }
)

export default apiClient