import apiClient from './client'
import type {
  Team,
  Region,
  Season,
  Competition,
  Match,
  ApiResponse,
  PaginatedResponse,
  QueryOptions,
  CreateTeamForm,
  UpdateTeamForm,
  CreateCompetitionForm,
  RegionalStandingsResponse,
  AnnualRankingsResponse,
  SeasonInfo,
  SeasonHonorsResponse,
  CreateHonorRecordForm,
  UpdateRankingForm,
  PlayoffBracket,
  GeneratePlayoffRequest,
  SimulatePlayoffMatchRequest,
  SimulatePlayoffMatchResponse,
  MSIBracket,
  GenerateMSIRequest,
  SimulateMSIMatchRequest,
  SimulateMSIMatchResponse,
  MSIEligibilityResponse,
  MSIQualification,
  WorldsBracket,
  GenerateWorldsRequest,
  SimulateWorldsMatchRequest,
  SimulateWorldsMatchResponse,
  WorldsEligibilityResponse,
  WorldsQualification,
  SuperBracket,
  SuperQualification,
  GenerateSuperRequest,
  SimulateSuperMatchRequest,
  SimulateSuperMatchResponse,
  SuperEligibilityResponse,
  ClauchBracket,
  ClauchQualification,
  GenerateClauchRequest,
  SimulateClauchMatchRequest,
  SimulateClauchMatchResponse,
  ClauchEligibilityResponse
} from '@/types'

// 战队相关API
export const teamApi = {
  // 获取战队列表
  getTeams: (params?: QueryOptions): Promise<PaginatedResponse<Team>> => {
    return apiClient.get('/api/teams', { params })
  },

  // 获取战队详情
  getTeam: (id: string): Promise<ApiResponse<Team>> => {
    return apiClient.get(`/api/teams/${id}`)
  },

  // 创建战队
  createTeam: (data: CreateTeamForm): Promise<ApiResponse<Team>> => {
    return apiClient.post('/api/teams', data)
  },

  // 更新战队
  updateTeam: (id: string, data: UpdateTeamForm): Promise<ApiResponse<Team>> => {
    return apiClient.put(`/api/teams/${id}`, data)
  },

  // 删除战队
  deleteTeam: (id: string): Promise<ApiResponse<void>> => {
    return apiClient.delete(`/api/teams/${id}`)
  },

  // 获取战队统计
  getTeamStatistics: (id: string): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/teams/${id}/statistics`)
  },

  // 获取战队比赛历史
  getTeamMatches: (id: string, params?: QueryOptions): Promise<PaginatedResponse<any>> => {
    return apiClient.get(`/api/teams/${id}/matches`, { params })
  },
}

// 赛区相关API
export const regionApi = {
  // 获取赛区列表
  getRegions: (): Promise<ApiResponse<Region[]>> => {
    return apiClient.get('/api/regions')
  },

  // 获取赛区详情
  getRegion: (id: string): Promise<ApiResponse<Region>> => {
    return apiClient.get(`/api/regions/${id}`)
  },

  // 更新赛区
  updateRegion: (id: string, data: Partial<Region>): Promise<ApiResponse<Region>> => {
    return apiClient.put(`/api/regions/${id}`, data)
  },

  // 获取赛区下的战队
  getRegionTeams: (id: string): Promise<ApiResponse<Team[]>> => {
    return apiClient.get(`/api/regions/${id}/teams`)
  },
}

// 赛季相关API
export const seasonApi = {
  // 获取赛季列表
  getSeasons: (): Promise<ApiResponse<Season[]>> => {
    return apiClient.get('/api/seasons')
  },

  // 获取赛季详情
  getSeason: (id: string): Promise<ApiResponse<Season>> => {
    return apiClient.get(`/api/seasons/${id}`)
  },

  // 创建赛季
  createSeason: (data: Partial<Season>): Promise<ApiResponse<Season>> => {
    return apiClient.post('/api/seasons', data)
  },

  // 更新赛季
  updateSeason: (id: string, data: Partial<Season>): Promise<ApiResponse<Season>> => {
    return apiClient.put(`/api/seasons/${id}`, data)
  },

  // 获取赛季下的赛事
  getSeasonCompetitions: (id: string): Promise<ApiResponse<Competition[]>> => {
    return apiClient.get(`/api/seasons/${id}/competitions`)
  },

  // 结束赛季并创建新赛季
  endSeason: (id: string): Promise<ApiResponse<any>> => {
    return apiClient.post(`/api/seasons/${id}/end`)
  },
}

// 赛事相关API
export const competitionApi = {
  // 获取赛事列表
  getCompetitions: (params?: QueryOptions): Promise<PaginatedResponse<Competition>> => {
    return apiClient.get('/api/competitions', { params })
  },

  // 获取赛事详情
  getCompetition: (id: string): Promise<ApiResponse<Competition>> => {
    return apiClient.get(`/api/competitions/${id}`)
  },

  // 创建赛事
  createCompetition: (data: CreateCompetitionForm): Promise<ApiResponse<Competition>> => {
    return apiClient.post('/api/competitions', data)
  },

  // 更新赛事
  updateCompetition: (id: string, data: Partial<Competition>): Promise<ApiResponse<Competition>> => {
    return apiClient.put(`/api/competitions/${id}`, data)
  },

  // 删除赛事
  deleteCompetition: (id: string): Promise<ApiResponse<void>> => {
    return apiClient.delete(`/api/competitions/${id}`)
  },

  // 开始赛事
  startCompetition: (id: string): Promise<ApiResponse<Competition>> => {
    return apiClient.post(`/api/competitions/${id}/start`)
  },

  // 结束赛事
  finishCompetition: (id: string): Promise<ApiResponse<Competition>> => {
    return apiClient.post(`/api/competitions/${id}/finish`)
  },

  // 获取赛事比赛
  getCompetitionMatches: (id: string, params?: QueryOptions): Promise<PaginatedResponse<Match>> => {
    return apiClient.get(`/api/competitions/${id}/matches`, { params })
  },

  // 生成赛事对阵
  generateMatches: (id: string): Promise<ApiResponse<Match[]>> => {
    return apiClient.post(`/api/competitions/${id}/generate-matches`)
  },

  // 获取赛事积分榜
  getCompetitionStandings: (id: string): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/competitions/${id}/standings`)
  },

  // 生成赛程（第二阶段新增）
  generateSchedule: (id: string): Promise<ApiResponse<Match[]>> => {
    return apiClient.post(`/api/competitions/${id}/generate-schedule`)
  },

  // 获取当前轮次信息（第二阶段新增）
  getCurrentRound: (id: string): Promise<ApiResponse<{
    currentRound: number
    totalRounds: number
    currentStage: string
    competition: Competition
  }>> => {
    return apiClient.get(`/api/competitions/${id}/current-round`)
  },

  // 模拟整轮比赛（第二阶段新增）
  simulateRound: (id: string): Promise<ApiResponse<{
    competitionId: string
    currentRound: number
    matchesSimulated: number
    results: Array<{
      matchId: string
      homeTeam?: string
      awayTeam?: string
      homeTeamName?: string
      awayTeamName?: string
      homeScore?: number
      awayScore?: number
      result?: string
      winner: string
    }>
    standings?: any[]
    nextRound: number
    isRoundComplete: boolean
  }>> => {
    return apiClient.post(`/api/competitions/${id}/simulate-round`)
  },

  // 添加参赛队伍
  addTeamToCompetition: (competitionId: string, data: { teamId: string; seed?: number; groupName?: string }): Promise<ApiResponse<void>> => {
    return apiClient.post(`/api/competitions/${competitionId}/teams`, data)
  },

  // 移除参赛队伍
  removeTeamFromCompetition: (competitionId: string, teamId: string): Promise<ApiResponse<void>> => {
    return apiClient.delete(`/api/competitions/${competitionId}/teams/${teamId}`)
  },

  // 获取参赛队伍列表
  getCompetitionTeams: (competitionId: string): Promise<ApiResponse<any[]>> => {
    return apiClient.get(`/api/competitions/${competitionId}/teams`)
  },
}

// 比赛相关API
export const matchApi = {
  // 获取比赛列表
  getMatches: (params?: QueryOptions): Promise<PaginatedResponse<Match>> => {
    return apiClient.get('/api/matches', { params })
  },

  // 获取比赛详情
  getMatch: (id: string): Promise<ApiResponse<Match>> => {
    return apiClient.get(`/api/matches/${id}`)
  },

  // 更新比赛
  updateMatch: (id: string, data: Partial<Match>): Promise<ApiResponse<Match>> => {
    return apiClient.put(`/api/matches/${id}`, data)
  },

  // 开始比赛
  startMatch: (id: string): Promise<ApiResponse<Match>> => {
    return apiClient.post(`/api/matches/${id}/start`)
  },

  // 结束比赛
  finishMatch: (id: string, result: any): Promise<ApiResponse<Match>> => {
    return apiClient.post(`/api/matches/${id}/finish`, result)
  },

  // 模拟比赛
  simulateMatch: (id: string): Promise<ApiResponse<Match>> => {
    return apiClient.post(`/api/matches/${id}/simulate`)
  },
}

// 积分排名相关API
export const rankingApi = {
  // 获取赛区常规赛积分榜
  getRegionalStandings: (
    regionId: string,
    seasonId: string,
    type: 'spring' | 'summer'
  ): Promise<ApiResponse<RegionalStandingsResponse>> => {
    return apiClient.get('/api/rankings/regional', {
      params: { regionId, seasonId, type }
    })
  },

  // 获取年度积分排名
  getAnnualRankings: (seasonId: string): Promise<ApiResponse<AnnualRankingsResponse>> => {
    return apiClient.get('/api/rankings/annual', {
      params: { seasonId }
    })
  },

  // 更新赛区常规赛积分榜（第三阶段：从后端获取）
  updateRegionalStandings: (data: UpdateRankingForm): Promise<ApiResponse<{
    regionId: string
    regionName: string
    seasonId: string
    competitionType: 'spring' | 'summer'
    standings: Array<{
      teamId: string
      teamName: string
      matchesPlayed: number
      wins: number
      losses: number
      winRate: number
      regularSeasonPoints: number
      roundDifferential: number
      position: number
    }>
    lastUpdated: string
  }>> => {
    return apiClient.post('/api/rankings/regional/update', data)
  },

  // 更新年度积分排名
  updateAnnualRankings: (seasonId: string): Promise<ApiResponse<void>> => {
    return apiClient.post('/api/rankings/annual/update', { seasonId })
  },

  // 批量刷新所有排名
  refreshAllRankings: (seasonId: string): Promise<ApiResponse<void>> => {
    return apiClient.post('/api/rankings/refresh', { seasonId })
  },
}

// 荣誉殿堂相关API
export const honorHallApi = {
  // 获取可用赛季列表
  getAvailableSeasons: (): Promise<ApiResponse<SeasonInfo[]>> => {
    return apiClient.get('/api/honor-hall/seasons')
  },

  // 获取指定赛季的荣誉数据
  getSeasonHonors: (seasonId: string): Promise<ApiResponse<SeasonHonorsResponse>> => {
    return apiClient.get(`/api/honor-hall/seasons/${seasonId}/honors`)
  },

  // 创建单条荣誉记录
  createHonorRecord: (data: CreateHonorRecordForm): Promise<ApiResponse<any>> => {
    return apiClient.post('/api/honor-hall/records', data)
  },

  // 批量创建荣誉记录
  batchCreateHonorRecords: (records: CreateHonorRecordForm[]): Promise<ApiResponse<any>> => {
    return apiClient.post('/api/honor-hall/records/batch', { records })
  },
}

// 季后赛相关API
export const playoffApi = {
  // 生成季后赛对阵(常规赛结束后调用)
  generatePlayoff: (data: GeneratePlayoffRequest): Promise<ApiResponse<PlayoffBracket>> => {
    return apiClient.post('/api/playoffs/generate', data)
  },

  // 获取季后赛对阵信息
  getPlayoffBracket: (
    competitionId: string,
    regionId: string
  ): Promise<ApiResponse<PlayoffBracket>> => {
    return apiClient.get(`/api/playoffs/bracket`, {
      params: { competitionId, regionId }
    })
  },

  // 获取赛区所有季后赛
  getRegionPlayoffs: (
    regionId: string,
    seasonId: string
  ): Promise<ApiResponse<PlayoffBracket[]>> => {
    return apiClient.get(`/api/playoffs/region/${regionId}`, {
      params: { seasonId }
    })
  },

  // 模拟季后赛单场比赛(BO5)
  simulatePlayoffMatch: (
    data: SimulatePlayoffMatchRequest
  ): Promise<ApiResponse<SimulatePlayoffMatchResponse>> => {
    return apiClient.post('/api/playoffs/simulate-match', data)
  },

  // 获取季后赛资格队伍(常规赛前4名)
  getQualifiedTeams: (
    competitionId: string,
    regionId: string
  ): Promise<ApiResponse<any[]>> => {
    return apiClient.get(`/api/playoffs/qualified-teams`, {
      params: { competitionId, regionId }
    })
  },

  // 检查是否可以生成季后赛(常规赛是否结束)
  checkPlayoffEligibility: (
    competitionId: string,
    regionId: string
  ): Promise<ApiResponse<{
    eligible: boolean
    reason?: string
    qualifiedTeams?: any[]
  }>> => {
    return apiClient.get(`/api/playoffs/check-eligibility`, {
      params: { competitionId, regionId }
    })
  },
}

// MSI季中赛相关API
export const msiApi = {
  // 生成MSI对阵(春季赛季后赛全部结束后调用)
  generateMSI: (data: GenerateMSIRequest): Promise<ApiResponse<MSIBracket>> => {
    return apiClient.post('/api/msi/generate', data)
  },

  // 获取MSI对阵信息
  getMSIBracket: (seasonId: string): Promise<ApiResponse<MSIBracket>> => {
    return apiClient.get(`/api/msi/bracket`, {
      params: { seasonId }
    })
  },

  // 模拟MSI单场比赛(BO5)
  simulateMSIMatch: (
    data: SimulateMSIMatchRequest
  ): Promise<ApiResponse<SimulateMSIMatchResponse>> => {
    return apiClient.post('/api/msi/simulate-match', data)
  },

  // 获取MSI资格队伍(各赛区春季赛前三名)
  getQualifiedTeams: (seasonId: string): Promise<ApiResponse<MSIQualification[]>> => {
    return apiClient.get(`/api/msi/qualified-teams`, {
      params: { seasonId }
    })
  },

  // 检查是否可以生成MSI(所有赛区春季赛季后赛是否结束)
  checkMSIEligibility: (seasonId: string): Promise<ApiResponse<MSIEligibilityResponse>> => {
    return apiClient.get(`/api/msi/check-eligibility`, {
      params: { seasonId }
    })
  },

  // 获取历史MSI数据
  getHistoricalMSI: (seasonId: string): Promise<ApiResponse<MSIBracket[]>> => {
    return apiClient.get(`/api/msi/historical`, {
      params: { seasonId }
    })
  },
}

// 世界赛相关API
export const worldsApi = {
  // 生成世界赛对阵(夏季赛季后赛全部结束后调用)
  generateWorlds: (data: GenerateWorldsRequest): Promise<ApiResponse<WorldsBracket>> => {
    return apiClient.post('/api/worlds/generate', data)
  },

  // 获取世界赛对阵信息
  getWorldsBracket: (seasonId: string): Promise<ApiResponse<WorldsBracket>> => {
    return apiClient.get(`/api/worlds/bracket`, {
      params: { seasonId }
    })
  },

  // 模拟世界赛单场比赛(小组赛BO1或淘汰赛BO5)
  simulateWorldsMatch: (
    data: SimulateWorldsMatchRequest
  ): Promise<ApiResponse<SimulateWorldsMatchResponse>> => {
    return apiClient.post('/api/worlds/simulate-match', data)
  },

  // 获取世界赛资格队伍(各赛区夏季赛前三名)
  getQualifiedTeams: (seasonId: string): Promise<ApiResponse<WorldsQualification[]>> => {
    return apiClient.get(`/api/worlds/qualified-teams`, {
      params: { seasonId }
    })
  },

  // 检查是否可以生成世界赛(所有赛区夏季赛季后赛是否结束)
  checkWorldsEligibility: (seasonId: string): Promise<ApiResponse<WorldsEligibilityResponse>> => {
    return apiClient.get(`/api/worlds/check-eligibility`, {
      params: { seasonId }
    })
  },

  // 获取历史世界赛数据
  getHistoricalWorlds: (seasonId: string): Promise<ApiResponse<WorldsBracket[]>> => {
    return apiClient.get(`/api/worlds/historical`, {
      params: { seasonId }
    })
  },

  // 获取小组赛瑞士轮积分榜
  getSwissStandings: (worldsId: string): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/worlds/${worldsId}/swiss-standings`)
  },


  // 更新世界赛状态
  updateWorldsStatus: (worldsId: string, status: string): Promise<ApiResponse<any>> => {
    return apiClient.put(`/api/worlds/${worldsId}/status`, { status })
  },

  // 生成小组赛下一轮对阵(瑞士轮配对)
  generateSwissRound: (worldsId: string): Promise<ApiResponse<any>> => {
    return apiClient.post(`/api/worlds/${worldsId}/generate-swiss-round`)
  },

  // 生成淘汰赛对阵
  generateKnockout: (worldsId: string): Promise<ApiResponse<any>> => {
    return apiClient.post(`/api/worlds/${worldsId}/generate-knockout`)
  },
}

// 积分管理相关API
export const pointsApi = {
  // 获取战队积分详情
  getTeamPointsBreakdown: (teamId: number, seasonYear: number): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/points/team/${teamId}/${seasonYear}`)
  },

  // 获取赛季积分排名
  getSeasonPointsRanking: (seasonYear: number): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/points/season/${seasonYear}`)
  },

  // 重新计算赛季积分
  recalculateSeasonPoints: (seasonYear: number): Promise<ApiResponse<any>> => {
    return apiClient.post(`/api/points/recalculate/${seasonYear}`)
  },

  // 获取战队积分历史
  getTeamPointsHistory: (teamId: number, seasonYear?: number): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/points/history/${teamId}`, {
      params: seasonYear ? { seasonYear } : undefined
    })
  },

  // 获取赛区积分排名
  getRegionPointsRanking: (regionId: number, seasonYear: number): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/points/region/${regionId}/${seasonYear}`)
  },

  // 获取两年积分总和排名（用于Super洲际赛）
  getTwoYearPointsRanking: (season1Year: number, season2Year: number): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/points/two-year/${season1Year}/${season2Year}`)
  },
}

// Super洲际赛相关API - 使用 Tauri 命令
export const superApi = {
  // 生成Super对阵(两年世界赛全部结束后调用)
  generateSuper: async (data: GenerateSuperRequest): Promise<ApiResponse<SuperBracket>> => {
    try {
      const { internationalApi } = await import('./tauri')
      // 从 data 中提取队伍分组
      const tournamentId = await internationalApi.createSuperTournament(
        data.legendaryTeamIds || [],
        data.challengerTeamIds || [],
        data.fighterTeamIds || []
      )
      // 获取创建后的对阵信息
      const bracket = await internationalApi.getTournamentBracket(tournamentId)
      return { success: true, data: bracket as any }
    } catch (err: any) {
      return { success: false, error: err.message || '生成Super对阵失败' }
    }
  },

  // 获取Super对阵信息
  getSuperBracket: async (tournamentId: number): Promise<ApiResponse<SuperBracket>> => {
    try {
      const { internationalApi } = await import('./tauri')
      const bracket = await internationalApi.getTournamentBracket(tournamentId)
      return { success: true, data: bracket as any }
    } catch (err: any) {
      return { success: false, error: err.message || '获取Super对阵失败' }
    }
  },

  // 模拟Super单场比赛
  simulateSuperMatch: async (
    data: SimulateSuperMatchRequest
  ): Promise<ApiResponse<SimulateSuperMatchResponse>> => {
    try {
      const { matchApi, internationalApi } = await import('./tauri')
      // 使用通用的比赛模拟命令
      const result = await matchApi.simulateMatchDetailed(data.matchId)
      // 如果需要推进对阵
      if (data.tournamentId && result.winner_id) {
        await internationalApi.advanceBracket(data.tournamentId, data.matchId, result.winner_id)
      }
      return {
        success: true,
        data: {
          match: result as any,
          isSuperComplete: false,
          isStageComplete: false
        }
      }
    } catch (err: any) {
      return { success: false, error: err.message || '模拟比赛失败' }
    }
  },

  // 获取Super资格队伍(基于两年积分总和前16名)
  getQualifiedTeams: async (): Promise<ApiResponse<SuperQualification[]>> => {
    try {
      const { pointsApi } = await import('./tauri')
      const teams = await pointsApi.getSuperQualifiedTeams()
      return { success: true, data: teams as any }
    } catch (err: any) {
      return { success: false, error: err.message || '获取资格队伍失败' }
    }
  },

  // 检查是否可以生成Super(两年世界赛是否都结束)
  checkSuperEligibility: async (): Promise<ApiResponse<SuperEligibilityResponse>> => {
    try {
      const { pointsApi } = await import('./tauri')
      const teams = await pointsApi.getSuperQualifiedTeams()
      // 如果有16支队伍，说明可以生成Super
      const canGenerate = teams && teams.length >= 16
      return {
        success: true,
        data: {
          canGenerate,
          qualifiedTeamsCount: teams?.length || 0
        } as any
      }
    } catch (err: any) {
      return { success: false, error: err.message || '检查资格失败' }
    }
  },

  // 获取Fighter组积分榜
  getFighterStandings: async (tournamentId: number): Promise<ApiResponse<any>> => {
    try {
      const { internationalApi } = await import('./tauri')
      const standings = await internationalApi.getGroupStandings(tournamentId)
      return { success: true, data: standings }
    } catch (err: any) {
      return { success: false, error: err.message || '获取积分榜失败' }
    }
  },

  // 完成赛事（发放奖金和荣誉）
  completeTournament: async (tournamentId: number): Promise<ApiResponse<any>> => {
    try {
      const { internationalApi } = await import('./tauri')
      const result = await internationalApi.completeTournament(tournamentId)
      return { success: true, data: result }
    } catch (err: any) {
      return { success: false, error: err.message || '完成赛事失败' }
    }
  },
}

// C洲际赛相关API
export const clauchApi = {
  // 生成C洲际赛对阵(世界赛结束后调用)
  generateClauch: (data: GenerateClauchRequest): Promise<ApiResponse<ClauchBracket>> => {
    return apiClient.post('/api/clauch/generate', data)
  },

  // 获取C洲际赛对阵信息
  getClauchBracket: (seasonId: string): Promise<ApiResponse<ClauchBracket>> => {
    return apiClient.get(`/api/clauch/bracket`, {
      params: { seasonId }
    })
  },

  // 模拟C洲际赛单场比赛
  simulateClauchMatch: (
    data: SimulateClauchMatchRequest
  ): Promise<ApiResponse<SimulateClauchMatchResponse>> => {
    return apiClient.post('/api/clauch/simulate-match', data)
  },

  // 获取C洲际赛资格队伍(各赛区夏季赛常规赛前8名)
  getQualifiedTeams: (seasonId: string): Promise<ApiResponse<ClauchQualification[]>> => {
    return apiClient.get(`/api/clauch/qualified-teams`, {
      params: { seasonId }
    })
  },

  // 检查是否可以生成C洲际赛(世界赛是否结束)
  checkClauchEligibility: (seasonId: string): Promise<ApiResponse<ClauchEligibilityResponse>> => {
    return apiClient.get(`/api/clauch/check-eligibility`, {
      params: { seasonId }
    })
  },

  // 获取历史C洲际赛数据
  getHistoricalClauch: (): Promise<ApiResponse<ClauchBracket[]>> => {
    return apiClient.get(`/api/clauch/historical`)
  },

  // 获取小组赛积分榜
  getGroupStandings: (clauchId: string, groupName?: string): Promise<ApiResponse<any>> => {
    return apiClient.get(`/api/clauch/${clauchId}/group-standings`, {
      params: groupName ? { groupName } : undefined
    })
  },

  // 更新C洲际赛状态
  updateClauchStatus: (clauchId: string, status: string): Promise<ApiResponse<any>> => {
    return apiClient.put(`/api/clauch/${clauchId}/status`, { status })
  },

  // 生成淘汰赛对阵
  generateKnockout: (clauchId: string): Promise<ApiResponse<any>> => {
    return apiClient.post(`/api/clauch/${clauchId}/generate-knockout`)
  },
}

// Mock数据 - 开发阶段使用
export const mockData = {
  regions: [
    { id: '1', name: 'LPL', teams: [] },
    { id: '2', name: 'LCK', teams: [] },
    { id: '3', name: 'LEC', teams: [] },
    { id: '4', name: 'LCS', teams: [] },
  ] as Region[],

  teams: [
    // LPL 战队
    { id: '1', name: 'FunPlus Phoenix', regionId: '1', strength: 85, statistics: { totalMatches: 24, wins: 18, losses: 6, winRate: 0.75, totalPoints: 45, seasonPoints: 45, intercontinentalPoints: 0 } },
    { id: '2', name: 'Top Esports', regionId: '1', strength: 82, statistics: { totalMatches: 24, wins: 17, losses: 7, winRate: 0.71, totalPoints: 42, seasonPoints: 42, intercontinentalPoints: 0 } },
    { id: '3', name: 'JD Gaming', regionId: '1', strength: 80, statistics: { totalMatches: 24, wins: 16, losses: 8, winRate: 0.67, totalPoints: 39, seasonPoints: 39, intercontinentalPoints: 0 } },
    { id: '4', name: 'Edward Gaming', regionId: '1', strength: 78, statistics: { totalMatches: 24, wins: 15, losses: 9, winRate: 0.63, totalPoints: 36, seasonPoints: 36, intercontinentalPoints: 0 } },
    { id: '5', name: 'Royal Never Give Up', regionId: '1', strength: 76, statistics: { totalMatches: 24, wins: 14, losses: 10, winRate: 0.58, totalPoints: 33, seasonPoints: 33, intercontinentalPoints: 0 } },
    { id: '6', name: 'Bilibili Gaming', regionId: '1', strength: 72, statistics: { totalMatches: 24, wins: 12, losses: 12, winRate: 0.50, totalPoints: 27, seasonPoints: 27, intercontinentalPoints: 0 } },
    { id: '7', name: 'Weibo Gaming', regionId: '1', strength: 70, statistics: { totalMatches: 24, wins: 11, losses: 13, winRate: 0.46, totalPoints: 24, seasonPoints: 24, intercontinentalPoints: 0 } },
    { id: '8', name: 'LNG Esports', regionId: '1', strength: 68, statistics: { totalMatches: 24, wins: 10, losses: 14, winRate: 0.42, totalPoints: 21, seasonPoints: 21, intercontinentalPoints: 0 } },
    { id: '9', name: 'Invictus Gaming', regionId: '1', strength: 65, statistics: { totalMatches: 24, wins: 8, losses: 16, winRate: 0.33, totalPoints: 15, seasonPoints: 15, intercontinentalPoints: 0 } },
    { id: '10', name: 'ThunderTalk Gaming', regionId: '1', strength: 62, statistics: { totalMatches: 24, wins: 6, losses: 18, winRate: 0.25, totalPoints: 9, seasonPoints: 9, intercontinentalPoints: 0 } },

    // LCK 战队
    { id: '11', name: 'T1', regionId: '2', strength: 90, statistics: { totalMatches: 24, wins: 20, losses: 4, winRate: 0.83, totalPoints: 54, seasonPoints: 54, intercontinentalPoints: 0 } },
    { id: '12', name: 'DWG KIA', regionId: '2', strength: 88, statistics: { totalMatches: 24, wins: 19, losses: 5, winRate: 0.79, totalPoints: 51, seasonPoints: 51, intercontinentalPoints: 0 } },
    { id: '13', name: 'Gen.G', regionId: '2', strength: 85, statistics: { totalMatches: 24, wins: 18, losses: 6, winRate: 0.75, totalPoints: 48, seasonPoints: 48, intercontinentalPoints: 0 } },
    { id: '14', name: 'KT Rolster', regionId: '2', strength: 78, statistics: { totalMatches: 24, wins: 15, losses: 9, winRate: 0.63, totalPoints: 36, seasonPoints: 36, intercontinentalPoints: 0 } },
    { id: '15', name: 'Hanwha Life Esports', regionId: '2', strength: 75, statistics: { totalMatches: 24, wins: 13, losses: 11, winRate: 0.54, totalPoints: 30, seasonPoints: 30, intercontinentalPoints: 0 } },
    { id: '16', name: 'DRX', regionId: '2', strength: 72, statistics: { totalMatches: 24, wins: 12, losses: 12, winRate: 0.50, totalPoints: 27, seasonPoints: 27, intercontinentalPoints: 0 } },
    { id: '17', name: 'Liiv SANDBOX', regionId: '2', strength: 70, statistics: { totalMatches: 24, wins: 11, losses: 13, winRate: 0.46, totalPoints: 24, seasonPoints: 24, intercontinentalPoints: 0 } },
    { id: '18', name: 'Kwangdong Freecs', regionId: '2', strength: 67, statistics: { totalMatches: 24, wins: 9, losses: 15, winRate: 0.38, totalPoints: 18, seasonPoints: 18, intercontinentalPoints: 0 } },
    { id: '19', name: 'BRO', regionId: '2', strength: 64, statistics: { totalMatches: 24, wins: 7, losses: 17, winRate: 0.29, totalPoints: 12, seasonPoints: 12, intercontinentalPoints: 0 } },
    { id: '20', name: 'Nongshim RedForce', regionId: '2', strength: 60, statistics: { totalMatches: 24, wins: 5, losses: 19, winRate: 0.21, totalPoints: 6, seasonPoints: 6, intercontinentalPoints: 0 } },

    // LEC 战队
    { id: '21', name: 'G2 Esports', regionId: '3', strength: 83, statistics: { totalMatches: 24, wins: 17, losses: 7, winRate: 0.71, totalPoints: 42, seasonPoints: 42, intercontinentalPoints: 0 } },
    { id: '22', name: 'Fnatic', regionId: '3', strength: 80, statistics: { totalMatches: 24, wins: 16, losses: 8, winRate: 0.67, totalPoints: 39, seasonPoints: 39, intercontinentalPoints: 0 } },
    { id: '23', name: 'MAD Lions', regionId: '3', strength: 78, statistics: { totalMatches: 24, wins: 15, losses: 9, winRate: 0.63, totalPoints: 36, seasonPoints: 36, intercontinentalPoints: 0 } },
    { id: '24', name: 'Rogue', regionId: '3', strength: 75, statistics: { totalMatches: 24, wins: 13, losses: 11, winRate: 0.54, totalPoints: 30, seasonPoints: 30, intercontinentalPoints: 0 } },
    { id: '25', name: 'Misfits Gaming', regionId: '3', strength: 72, statistics: { totalMatches: 24, wins: 12, losses: 12, winRate: 0.50, totalPoints: 27, seasonPoints: 27, intercontinentalPoints: 0 } },
    { id: '26', name: 'Team Vitality', regionId: '3', strength: 70, statistics: { totalMatches: 24, wins: 11, losses: 13, winRate: 0.46, totalPoints: 24, seasonPoints: 24, intercontinentalPoints: 0 } },
    { id: '27', name: 'Excel Esports', regionId: '3', strength: 67, statistics: { totalMatches: 24, wins: 9, losses: 15, winRate: 0.38, totalPoints: 18, seasonPoints: 18, intercontinentalPoints: 0 } },
    { id: '28', name: 'SK Gaming', regionId: '3', strength: 65, statistics: { totalMatches: 24, wins: 8, losses: 16, winRate: 0.33, totalPoints: 15, seasonPoints: 15, intercontinentalPoints: 0 } },
    { id: '29', name: 'Team BDS', regionId: '3', strength: 62, statistics: { totalMatches: 24, wins: 6, losses: 18, winRate: 0.25, totalPoints: 9, seasonPoints: 9, intercontinentalPoints: 0 } },
    { id: '30', name: 'Astralis', regionId: '3', strength: 58, statistics: { totalMatches: 24, wins: 4, losses: 20, winRate: 0.17, totalPoints: 3, seasonPoints: 3, intercontinentalPoints: 0 } },

    // LCS 战队
    { id: '31', name: 'Cloud9', regionId: '4', strength: 78, statistics: { totalMatches: 24, wins: 15, losses: 9, winRate: 0.63, totalPoints: 36, seasonPoints: 36, intercontinentalPoints: 0 } },
    { id: '32', name: 'Team Liquid', regionId: '4', strength: 75, statistics: { totalMatches: 24, wins: 13, losses: 11, winRate: 0.54, totalPoints: 30, seasonPoints: 30, intercontinentalPoints: 0 } },
    { id: '33', name: '100 Thieves', regionId: '4', strength: 73, statistics: { totalMatches: 24, wins: 12, losses: 12, winRate: 0.50, totalPoints: 27, seasonPoints: 27, intercontinentalPoints: 0 } },
    { id: '34', name: 'TSM', regionId: '4', strength: 70, statistics: { totalMatches: 24, wins: 11, losses: 13, winRate: 0.46, totalPoints: 24, seasonPoints: 24, intercontinentalPoints: 0 } },
    { id: '35', name: 'Evil Geniuses', regionId: '4', strength: 68, statistics: { totalMatches: 24, wins: 10, losses: 14, winRate: 0.42, totalPoints: 21, seasonPoints: 21, intercontinentalPoints: 0 } },
    { id: '36', name: 'FlyQuest', regionId: '4', strength: 65, statistics: { totalMatches: 24, wins: 8, losses: 16, winRate: 0.33, totalPoints: 15, seasonPoints: 15, intercontinentalPoints: 0 } },
    { id: '37', name: 'Dignitas', regionId: '4', strength: 63, statistics: { totalMatches: 24, wins: 7, losses: 17, winRate: 0.29, totalPoints: 12, seasonPoints: 12, intercontinentalPoints: 0 } },
    { id: '38', name: 'Immortals', regionId: '4', strength: 60, statistics: { totalMatches: 24, wins: 5, losses: 19, winRate: 0.21, totalPoints: 6, seasonPoints: 6, intercontinentalPoints: 0 } },
    { id: '39', name: 'Golden Guardians', regionId: '4', strength: 58, statistics: { totalMatches: 24, wins: 4, losses: 20, winRate: 0.17, totalPoints: 3, seasonPoints: 3, intercontinentalPoints: 0 } },
    { id: '40', name: 'CLG', regionId: '4', strength: 55, statistics: { totalMatches: 24, wins: 3, losses: 21, winRate: 0.13, totalPoints: 0, seasonPoints: 0, intercontinentalPoints: 0 } },
  ] as Team[],
}