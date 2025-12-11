import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Dashboard',
      component: () => import('@/views/Dashboard.vue'),
      meta: { title: '仪表盘' }
    },
    {
      path: '/tournaments',
      name: 'Tournaments',
      component: () => import('@/views/Tournaments.vue'),
      meta: { title: '赛事管理' }
    },
    {
      path: '/tournaments/:id',
      name: 'TournamentDetail',
      component: () => import('@/views/TournamentDetail.vue'),
      meta: { title: '赛事详情' }
    },
    {
      path: '/tournaments/summer/:id',
      name: 'SummerDetail',
      component: () => import('@/views/SummerDetail.vue'),
      meta: { title: '夏季赛' }
    },
    {
      path: '/tournaments/spring/:id',
      name: 'SpringDetail',
      component: () => import('@/views/SpringDetail.vue'),
      meta: { title: '春季赛' }
    },
    {
      path: '/tournaments/msi',
      name: 'MSIDetail',
      component: () => import('@/views/MSIDetail.vue'),
      meta: { title: 'MSI 季中邀请赛' }
    },
    {
      path: '/tournaments/worlds',
      name: 'WorldsDetail',
      component: () => import('@/views/WorldsDetail.vue'),
      meta: { title: 'S 世界赛' }
    },
    {
      path: '/tournaments/clauch/:id',
      name: 'ClauchDetail',
      component: () => import('@/views/ClauchDetail.vue'),
      meta: { title: '洲际赛' }
    },
    {
      path: '/tournaments/madrid/:id',
      name: 'MadridDetail',
      component: () => import('@/views/MadridDetail.vue'),
      meta: { title: '马德里大师赛' }
    },
    {
      path: '/tournaments/shanghai',
      name: 'ShanghaiDetail',
      component: () => import('@/views/ShanghaiDetail.vue'),
      meta: { title: '上海大师赛' }
    },
    {
      path: '/tournaments/icp/:id',
      name: 'ICPDetail',
      component: () => import('@/views/ICPDetail.vue'),
      meta: { title: 'ICP 洲际对抗赛' }
    },
    {
      path: '/tournaments/super/:id',
      name: 'SuperDetail',
      component: () => import('@/views/SuperDetail.vue'),
      meta: { title: 'Super 洲际年度邀请赛' }
    },
    {
      path: '/teams',
      name: 'Teams',
      component: () => import('@/views/Teams.vue'),
      meta: { title: '战队管理' }
    },
    {
      path: '/teams/:id',
      name: 'TeamDetail',
      component: () => import('@/views/TeamDetail.vue'),
      meta: { title: '战队详情' }
    },
    {
      path: '/teams/:id/edit',
      name: 'TeamEdit',
      component: () => import('@/views/TeamEdit.vue'),
      meta: { title: '编辑战队' }
    },
    {
      path: '/players',
      name: 'Players',
      component: () => import('@/views/Players.vue'),
      meta: { title: '选手中心' }
    },
    {
      path: '/players/:id',
      name: 'PlayerDetail',
      component: () => import('@/views/PlayerDetail.vue'),
      meta: { title: '选手详情' }
    },
    {
      path: '/draft',
      name: 'Draft',
      component: () => import('@/views/Draft.vue'),
      meta: { title: '选秀系统' }
    },
    {
      path: '/draft/pool',
      name: 'DraftPool',
      component: () => import('@/views/DraftPool.vue'),
      meta: { title: '选手池管理' }
    },
    {
      path: '/draft/:region',
      name: 'DraftRegion',
      component: () => import('@/views/DraftRegion.vue'),
      meta: { title: '赛区选秀' }
    },
    {
      path: '/transfer',
      name: 'Transfer',
      component: () => import('@/views/Transfer.vue'),
      meta: { title: '转会市场' }
    },
    {
      path: '/rankings',
      name: 'Rankings',
      component: () => import('@/views/Rankings.vue'),
      meta: { title: '积分排名' }
    },
    {
      path: '/player-statistics',
      name: 'PlayerStatistics',
      component: () => import('@/views/PlayerStatistics.vue'),
      meta: { title: '选手统计' }
    },
    {
      path: '/honors',
      name: 'Honors',
      component: () => import('@/views/Honors.vue'),
      meta: { title: '荣誉殿堂' }
    },
    {
      path: '/settings',
      name: 'Settings',
      component: () => import('@/views/Settings.vue'),
      meta: { title: '系统设置' }
    },
  ]
});

export default router;
