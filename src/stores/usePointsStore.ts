// =================================================================
// 电竞赛事模拟系统 - 积分管理Store
// =================================================================

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { pointsApi } from '@/api';
import { createLogger } from '@/utils/logger';
import { handleError } from '@/utils/errors';

const logger = createLogger('PointsStore');

// ========================================
// 类型定义
// ========================================

export interface TeamPointsBreakdown {
  teamId: number;
  teamName: string;
  seasonYear: number;
  totalPoints: number;
  springPoints: number;
  summerPoints: number;
  springPlayoffPoints: number;
  summerPlayoffPoints: number;
  playoffPoints: number;
  msiPoints: number;
  worldsPoints: number;
  intercontinentalPoints: number; // C洲际赛积分
  pointDetails: Array<{
    pointType: string;
    points: number;
    earnedAt: string;
    description: string;
  }>;
}

export interface SeasonPointsRanking {
  teamId: number;
  teamName: string;
  regionId: number;
  regionName: string;
  totalPoints: number;
  springPoints: number;
  summerPoints: number;
  playoffPoints: number;
  msiPoints: number;
  worldsPoints: number;
  intercontinentalPoints: number; // C洲际赛积分
  rank: number;
}

export interface PointsHistory {
  id: number;
  teamId: number;
  teamName: string;
  seasonYear: number;
  pointType: string;
  points: number;
  earnedAt: string;
  description: string;
  competitionName: string | null;
}

// ========================================
// Store定义
// ========================================

export const usePointsStore = defineStore('points', () => {
  // ========================================
  // 状态
  // ========================================

  const loading = ref(false);
  const error = ref<string | null>(null);

  // 赛季积分排名
  const seasonRankings = ref<SeasonPointsRanking[]>([]);
  
  // 赛区积分排名
  const regionRankings = ref<Map<string, SeasonPointsRanking[]>>(new Map());
  
  // 战队积分详情
  const teamPointsBreakdown = ref<Map<string, TeamPointsBreakdown>>(new Map());
  
  // 战队积分历史
  const teamPointsHistory = ref<Map<string, PointsHistory[]>>(new Map());

  // ========================================
  // 计算属性
  // ========================================

  /**
   * 是否正在加载
   */
  const isLoading = computed(() => loading.value);

  /**
   * 是否有错误
   */
  const hasError = computed(() => error.value !== null);

  // ========================================
  // Actions
  // ========================================

  /**
   * 获取赛季积分排名
   */
  async function fetchSeasonRanking(seasonYear: number): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      const response = await pointsApi.getSeasonPointsRanking(seasonYear);
      
      if (response.success) {
        seasonRankings.value = response.data;
        logger.info('赛季积分排名加载成功', {
          seasonYear,
          teamsCount: seasonRankings.value.length
        });
      } else {
        throw new Error(response.message || '获取赛季积分排名失败');
      }
    } catch (err: any) {
      error.value = err.message || '获取赛季积分排名失败';
      handleError(err, {
        component: 'PointsStore',
        userAction: '获取赛季积分排名',
        silent: true
      });
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取赛区积分排名
   */
  async function fetchRegionRanking(regionId: number, seasonYear: number): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      const response = await pointsApi.getRegionPointsRanking(regionId, seasonYear);
      
      if (response.success) {
        const key = `${regionId}-${seasonYear}`;
        regionRankings.value.set(key, response.data);
        logger.info('赛区积分排名加载成功', {
          regionId,
          seasonYear,
          teamsCount: response.data.length
        });
      } else {
        throw new Error(response.message || '获取赛区积分排名失败');
      }
    } catch (err: any) {
      error.value = err.message || '获取赛区积分排名失败';
      handleError(err, {
        component: 'PointsStore',
        userAction: '获取赛区积分排名',
        silent: true
      });
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取战队积分详情
   */
  async function fetchTeamPointsBreakdown(teamId: number, seasonYear: number): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      const response = await pointsApi.getTeamPointsBreakdown(teamId, seasonYear);
      
      if (response.success) {
        const key = `${teamId}-${seasonYear}`;
        teamPointsBreakdown.value.set(key, response.data);
        logger.info('战队积分详情加载成功', {
          teamId,
          seasonYear,
          totalPoints: response.data.totalPoints
        });
      } else {
        throw new Error(response.message || '获取战队积分详情失败');
      }
    } catch (err: any) {
      error.value = err.message || '获取战队积分详情失败';
      handleError(err, {
        component: 'PointsStore',
        userAction: '获取战队积分详情',
        silent: true
      });
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取战队积分历史
   */
  async function fetchTeamPointsHistory(teamId: number, seasonYear?: number): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      const response = await pointsApi.getTeamPointsHistory(teamId, seasonYear);
      
      if (response.success) {
        const key = seasonYear ? `${teamId}-${seasonYear}` : `${teamId}-all`;
        teamPointsHistory.value.set(key, response.data);
        logger.info('战队积分历史加载成功', {
          teamId,
          seasonYear: seasonYear || '全部',
          recordsCount: response.data.length
        });
      } else {
        throw new Error(response.message || '获取战队积分历史失败');
      }
    } catch (err: any) {
      error.value = err.message || '获取战队积分历史失败';
      handleError(err, {
        component: 'PointsStore',
        userAction: '获取战队积分历史',
        silent: true
      });
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 重新计算赛季积分
   */
  async function recalculateSeasonPoints(seasonYear: number): Promise<any> {
    loading.value = true;
    error.value = null;

    try {
      const response = await pointsApi.recalculateSeasonPoints(seasonYear);
      
      if (response.success) {
        logger.info('赛季积分重新计算成功', {
          seasonYear,
          result: response.data
        });

        // 重新加载赛季排名
        await fetchSeasonRanking(seasonYear);

        return response.data;
      } else {
        throw new Error(response.message || '重新计算赛季积分失败');
      }
    } catch (err: any) {
      error.value = err.message || '重新计算赛季积分失败';
      handleError(err, {
        component: 'PointsStore',
        userAction: '重新计算赛季积分'
      });
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取战队积分详情（从缓存或API）
   */
  function getTeamPointsBreakdown(teamId: number, seasonYear: number): TeamPointsBreakdown | undefined {
    const key = `${teamId}-${seasonYear}`;
    return teamPointsBreakdown.value.get(key);
  }

  /**
   * 获取赛区积分排名（从缓存或API）
   */
  function getRegionRanking(regionId: number, seasonYear: number): SeasonPointsRanking[] | undefined {
    const key = `${regionId}-${seasonYear}`;
    return regionRankings.value.get(key);
  }

  /**
   * 获取战队积分历史（从缓存）
   */
  function getTeamHistory(teamId: number, seasonYear?: number): PointsHistory[] | undefined {
    const key = seasonYear ? `${teamId}-${seasonYear}` : `${teamId}-all`;
    return teamPointsHistory.value.get(key);
  }

  /**
   * 清空缓存
   */
  function clearCache(): void {
    seasonRankings.value = [];
    regionRankings.value.clear();
    teamPointsBreakdown.value.clear();
    teamPointsHistory.value.clear();
    error.value = null;
    logger.info('积分Store缓存已清空');
  }

  /**
   * 清空错误
   */
  function clearError(): void {
    error.value = null;
  }

  return {
    // 状态
    loading,
    error,
    seasonRankings,
    regionRankings,
    teamPointsBreakdown,
    teamPointsHistory,
    
    // 计算属性
    isLoading,
    hasError,
    
    // Actions
    fetchSeasonRanking,
    fetchRegionRanking,
    fetchTeamPointsBreakdown,
    fetchTeamPointsHistory,
    recalculateSeasonPoints,
    getTeamPointsBreakdown,
    getRegionRanking,
    getTeamHistory,
    clearCache,
    clearError
  };
});

