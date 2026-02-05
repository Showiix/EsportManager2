# 前端开发指南

## 概述

EsportManager 2 前端使用 **Vue 3 + TypeScript + Element Plus** 构建，采用 Composition API 和 Pinia 状态管理。

## 目录结构

```
src/
├── main.ts                    # 应用入口
├── App.vue                    # 根组件
├── api/                       # API 调用
│   ├── index.ts              # API 导出
│   ├── tauri.ts              # Tauri 命令封装
│   └── client.ts             # API 客户端工具
├── components/                # 通用组件
│   ├── match/                # 比赛相关组件
│   ├── player/               # 选手相关组件
│   ├── finance/              # 财务相关组件
│   └── ...
├── views/                     # 页面视图
│   ├── HomeView.vue          # 主页
│   ├── MatchView.vue         # 比赛页
│   ├── TeamView.vue          # 战队页
│   └── ...
├── stores/                    # Pinia 状态管理
│   ├── useTimeStore.ts       # 时间状态
│   ├── useTeamStore.ts       # 战队状态
│   ├── usePlayerStore.ts     # 选手状态
│   ├── useMatchDetailStore.ts# 比赛详情状态
│   └── ...
├── engines/                   # 前端计算引擎
│   ├── index.ts              # 引擎导出
│   ├── PlayerEngine.ts       # 选手能力计算
│   └── PowerEngine.ts        # 战队战力计算
├── types/                     # TypeScript 类型定义
│   ├── index.ts              # 类型导出
│   ├── player.ts             # 选手类型
│   └── ...
├── router/                    # 路由配置
│   └── index.ts
├── utils/                     # 工具函数
│   ├── format.ts             # 格式化工具
│   ├── logger/               # 日志系统
│   └── errors/               # 错误处理
└── data/                      # 静态数据
    └── playerData.ts         # 选手数据
```

## 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue | 3.x | 核心框架 |
| TypeScript | 5.x | 类型系统 |
| Pinia | 2.x | 状态管理 |
| Vue Router | 4.x | 路由管理 |
| Element Plus | 2.x | UI 组件库 |
| Tauri | 2.0 | 桌面端框架 |

## API 调用

### Tauri 命令封装

```typescript
// src/api/tauri.ts
import { invoke } from '@tauri-apps/api/core';

// 获取时间状态
export async function getTimeState(saveId: string): Promise<GameTimeState> {
  return await invoke('get_time_state', { saveId });
}

// 模拟比赛
export async function simulateMatch(matchId: number): Promise<MatchResult> {
  return await invoke('simulate_match', { matchId });
}

// 获取战队阵容
export async function getTeamRoster(teamId: number): Promise<Player[]> {
  return await invoke('get_team_roster', { teamId });
}
```

### 使用示例

```typescript
import { getTimeState, simulateMatch } from '@/api/tauri';

// 在组件中调用
const timeState = await getTimeState(saveId);
const result = await simulateMatch(matchId);
```

## 状态管理 (Pinia)

### Store 定义

```typescript
// src/stores/useTimeStore.ts
import { defineStore } from 'pinia';
import { getTimeState, completeAndAdvance } from '@/api/tauri';

export const useTimeStore = defineStore('time', {
  state: () => ({
    saveId: '',
    currentSeason: 1,
    currentPhase: 'SPRING_REGULAR',
    phaseStatus: 'NOT_INITIALIZED',
    canAdvance: false,
    loading: false,
  }),

  getters: {
    phaseDisplayName: (state) => {
      const names: Record<string, string> = {
        SPRING_REGULAR: '春季赛常规赛',
        SPRING_PLAYOFFS: '春季赛季后赛',
        MSI: 'MSI季中赛',
        // ...
      };
      return names[state.currentPhase] || state.currentPhase;
    },
  },

  actions: {
    async fetchTimeState() {
      this.loading = true;
      try {
        const state = await getTimeState(this.saveId);
        this.currentSeason = state.current_season;
        this.currentPhase = state.current_phase;
        this.phaseStatus = state.phase_status;
        this.canAdvance = state.can_advance;
      } finally {
        this.loading = false;
      }
    },

    async advancePhase() {
      const result = await completeAndAdvance(this.saveId);
      await this.fetchTimeState();
      return result;
    },
  },
});
```

### Store 使用

```vue
<script setup lang="ts">
import { useTimeStore } from '@/stores/useTimeStore';

const timeStore = useTimeStore();

// 获取状态
await timeStore.fetchTimeState();

// 使用 getter
console.log(timeStore.phaseDisplayName);

// 调用 action
await timeStore.advancePhase();
</script>

<template>
  <div>
    <p>当前阶段: {{ timeStore.phaseDisplayName }}</p>
    <button @click="timeStore.advancePhase" :disabled="!timeStore.canAdvance">
      推进
    </button>
  </div>
</template>
```

## 前端计算引擎

### 选手能力计算

```typescript
// src/engines/PlayerEngine.ts
export class PlayerEngine {
  /**
   * 计算选手单局实际发挥
   */
  static calculatePerformance(player: Player): PlayerPerformance {
    // 基于稳定性的标准差
    const sigma = (100 - player.stability) / 10;

    // 生成高斯噪声
    const noise = this.gaussianRandom() * sigma;

    // 原始能力 = 基础能力 + 状态加成 + 噪声
    const rawAbility = player.ability + player.condition + noise;

    // 钳位到合理范围 [ability-15, ability+10]
    const minAbility = Math.max(0, player.ability - 15);
    const maxAbility = Math.min(100, player.ability + 10);
    const actualAbility = Math.max(minAbility, Math.min(maxAbility, rawAbility));

    return {
      playerId: player.id,
      actualAbility,
      impactScore: 0,
    };
  }

  /**
   * Box-Muller 变换生成标准正态分布
   */
  static gaussianRandom(): number {
    let u = 0, v = 0;
    while (u === 0) u = Math.random();
    while (v === 0) v = Math.random();
    return Math.sqrt(-2.0 * Math.log(u)) * Math.cos(2.0 * Math.PI * v);
  }
}
```

### 战力计算

```typescript
// src/engines/PowerEngine.ts
export class PowerEngine {
  /**
   * 计算队伍战力 = 5名选手实际能力的平均值
   */
  static calculateTeamPower(performances: PlayerPerformance[]): number {
    if (performances.length === 0) return 0;

    const total = performances.reduce((sum, p) => sum + p.actualAbility, 0);
    return total / performances.length;
  }

  /**
   * 计算队伍基础战力 (用于展示)
   */
  static calculateBasePower(players: Player[]): number {
    const starters = players.filter(p => p.isStarter);
    if (starters.length === 0) return 0;

    const total = starters.reduce((sum, p) => sum + p.ability, 0);
    return total / starters.length;
  }
}
```

## 类型定义

```typescript
// src/types/player.ts
export interface Player {
  id: number;
  saveId: string;
  teamId: number | null;
  gameId: string;
  realName: string | null;
  age: number;
  ability: number;
  potential: number;
  stability: number;
  position: Position;
  salary: number;
  marketValue: number;
  isStarter: boolean;
}

export type Position = 'TOP' | 'JUG' | 'MID' | 'ADC' | 'SUP';

export interface PlayerPerformance {
  playerId: number;
  actualAbility: number;
  impactScore: number;
}
```

## 路由配置

```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
  },
  {
    path: '/match/:id',
    name: 'match',
    component: () => import('@/views/MatchView.vue'),
  },
  {
    path: '/team/:id',
    name: 'team',
    component: () => import('@/views/TeamView.vue'),
  },
  // ...
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
```

## 日志系统

```typescript
// src/utils/logger/Logger.ts
import { LogLevel } from './LogLevel';

export class Logger {
  private static level: LogLevel = LogLevel.INFO;

  static debug(message: string, ...args: any[]) {
    if (this.level <= LogLevel.DEBUG) {
      console.debug(`[DEBUG] ${message}`, ...args);
    }
  }

  static info(message: string, ...args: any[]) {
    if (this.level <= LogLevel.INFO) {
      console.info(`[INFO] ${message}`, ...args);
    }
  }

  static warn(message: string, ...args: any[]) {
    if (this.level <= LogLevel.WARN) {
      console.warn(`[WARN] ${message}`, ...args);
    }
  }

  static error(message: string, ...args: any[]) {
    if (this.level <= LogLevel.ERROR) {
      console.error(`[ERROR] ${message}`, ...args);
    }
  }
}
```

## 错误处理

```typescript
// src/utils/errors/globalErrorHandler.ts
import { Logger } from '../logger/Logger';

export function setupGlobalErrorHandler() {
  window.addEventListener('error', (event) => {
    Logger.error('Uncaught error:', event.error);
  });

  window.addEventListener('unhandledrejection', (event) => {
    Logger.error('Unhandled promise rejection:', event.reason);
  });
}
```

## 格式化工具

```typescript
// src/utils/format.ts

/**
 * 格式化金额 (元 -> 万元)
 */
export function formatMoney(yuan: number): string {
  const wan = yuan / 10000;
  return `${wan.toFixed(1)}万`;
}

/**
 * 格式化百分比
 */
export function formatPercent(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

/**
 * 格式化选手位置
 */
export function formatPosition(position: string): string {
  const names: Record<string, string> = {
    TOP: '上单',
    JUG: '打野',
    MID: '中单',
    ADC: 'ADC',
    SUP: '辅助',
  };
  return names[position] || position;
}
```

## 开发命令

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 类型检查
npx vue-tsc --noEmit

# 代码检查
npm run lint

# 启动 Tauri 开发模式
npm run tauri dev
```

## 添加新页面流程

1. **创建页面组件** (`views/XxxView.vue`)
2. **添加路由配置** (`router/index.ts`)
3. **创建 Store** (`stores/useXxxStore.ts`) - 如需要
4. **添加 API 封装** (`api/tauri.ts`) - 如需要
5. **定义类型** (`types/xxx.ts`) - 如需要

## 文件位置

| 文件 | 说明 |
|-----|------|
| `vite.config.ts` | Vite 配置 |
| `tsconfig.json` | TypeScript 配置 |
| `package.json` | 项目依赖 |
| `src/main.ts` | 应用入口 |
