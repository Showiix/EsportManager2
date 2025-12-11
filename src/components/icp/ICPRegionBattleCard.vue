<template>
  <div class="icp-region-battle-card">
    <!-- 赛区对决头部 -->
    <div class="battle-header">
      <div class="region-side region-a">
        <div class="region-flag" :class="battle.regionA.toLowerCase()">
          {{ getRegionFlag(battle.regionA) }}
        </div>
        <div class="region-name">{{ battle.regionAName }}</div>
        <div class="region-wins">{{ battle.regionAWins }} 胜</div>
      </div>

      <div class="battle-vs">
        <div class="vs-text">VS</div>
        <div v-if="battle.status === 'completed'" class="battle-result">
          <span v-if="battle.winnerId === battle.regionA" class="winner-badge">
            {{ battle.regionAName }} 胜出!
          </span>
          <span v-else class="winner-badge">
            {{ battle.regionBName }} 胜出!
          </span>
        </div>
      </div>

      <div class="region-side region-b">
        <div class="region-flag" :class="battle.regionB.toLowerCase()">
          {{ getRegionFlag(battle.regionB) }}
        </div>
        <div class="region-name">{{ battle.regionBName }}</div>
        <div class="region-wins">{{ battle.regionBWins }} 胜</div>
      </div>
    </div>

    <!-- 四场BO5对决列表 -->
    <div class="battles-list">
      <div
        v-for="(match, index) in battle.matches"
        :key="match.id"
        class="battle-match"
        :class="{ completed: match.status === 'completed' }"
      >
        <div class="seed-label">{{ getSeedLabel(index + 1) }}对决</div>

        <div class="match-content">
          <div class="team team-a" :class="{ winner: match.winnerId === match.teamAId }">
            <span class="team-name">{{ match.teamAName }}</span>
            <el-tag :type="getRegionTagType(match.teamARegion)" size="small">
              {{ match.teamARegion }}
            </el-tag>
          </div>

          <div class="match-score">
            <template v-if="match.status === 'completed'">
              <span class="score">{{ match.scoreA }} - {{ match.scoreB }}</span>
              <el-button type="info" size="small" text @click="$emit('view-match', match)">
                详情
              </el-button>
            </template>
            <template v-else>
              <el-button
                type="primary"
                size="small"
                @click="$emit('simulate-match', battle, match)"
              >
                模拟
              </el-button>
            </template>
          </div>

          <div class="team team-b" :class="{ winner: match.winnerId === match.teamBId }">
            <span class="team-name">{{ match.teamBName }}</span>
            <el-tag :type="getRegionTagType(match.teamBRegion)" size="small">
              {{ match.teamBRegion }}
            </el-tag>
          </div>
        </div>
      </div>
    </div>

    <!-- 总比分进度条 -->
    <div class="battle-progress">
      <div class="progress-bar">
        <div
          class="progress-a"
          :style="{ width: `${(battle.regionAWins / 4) * 100}%` }"
        ></div>
        <div
          class="progress-b"
          :style="{ width: `${(battle.regionBWins / 4) * 100}%` }"
        ></div>
      </div>
      <div class="progress-labels">
        <span class="label-a">{{ battle.regionA }}: {{ battle.regionAWins }}/4</span>
        <span class="label-b">{{ battle.regionB }}: {{ battle.regionBWins }}/4</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ICPRegionMatch, ICPMatch } from '@/types/icp'

interface Props {
  battle: ICPRegionMatch
}

defineProps<Props>()

defineEmits<{
  (e: 'simulate-match', battle: ICPRegionMatch, match: ICPMatch): void
  (e: 'view-match', match: ICPMatch): void
}>()

const getRegionFlag = (region: string) => {
  const flagMap: Record<string, string> = {
    'LPL': '🇨🇳',
    'LCK': '🇰🇷',
    'LEC': '🇪🇺',
    'LCS': '🇺🇸'
  }
  return flagMap[region] || '🏳️'
}

const getSeedLabel = (seed: number) => {
  const labels: Record<number, string> = {
    1: '一号种子',
    2: '二号种子',
    3: '三号种子',
    4: '四号种子'
  }
  return labels[seed] || `${seed}号种子`
}

const getRegionTagType = (region?: string) => {
  const typeMap: Record<string, any> = {
    'LPL': 'danger',
    'LCK': 'primary',
    'LEC': 'success',
    'LCS': 'warning'
  }
  return typeMap[region || ''] || 'info'
}
</script>

<style scoped lang="scss">
.icp-region-battle-card {
  background: white;
  border-radius: 12px;
  padding: 24px;
  border: 2px solid #e5e7eb;

  .battle-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    padding-bottom: 20px;
    border-bottom: 2px solid #f3f4f6;

    .region-side {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 8px;
      flex: 1;

      .region-flag {
        font-size: 48px;
      }

      .region-name {
        font-size: 16px;
        font-weight: 700;
        color: #1f2937;
      }

      .region-wins {
        font-size: 20px;
        font-weight: 700;
        padding: 4px 16px;
        border-radius: 20px;
      }

      &.region-a .region-wins {
        background: linear-gradient(135deg, #fee2e2 0%, #fecaca 100%);
        color: #dc2626;
      }

      &.region-b .region-wins {
        background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
        color: #2563eb;
      }
    }

    .battle-vs {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 8px;
      padding: 0 24px;

      .vs-text {
        font-size: 24px;
        font-weight: 900;
        color: #9ca3af;
      }

      .battle-result {
        .winner-badge {
          padding: 6px 16px;
          background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
          color: #92400e;
          border-radius: 20px;
          font-size: 14px;
          font-weight: 700;
        }
      }
    }
  }

  .battles-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 24px;

    .battle-match {
      padding: 16px;
      background: #f9fafb;
      border-radius: 8px;
      border: 1px solid #e5e7eb;

      &.completed {
        background: #f0fdf4;
        border-color: #86efac;
      }

      .seed-label {
        text-align: center;
        font-size: 12px;
        font-weight: 600;
        color: #f59e0b;
        margin-bottom: 12px;
        padding: 4px 12px;
        background: #fffbeb;
        border-radius: 4px;
        display: inline-block;
        margin-left: 50%;
        transform: translateX(-50%);
      }

      .match-content {
        display: flex;
        align-items: center;
        justify-content: space-between;

        .team {
          display: flex;
          align-items: center;
          gap: 8px;
          flex: 1;

          .team-name {
            font-weight: 600;
            font-size: 14px;
            color: #374151;
          }

          &.winner .team-name {
            color: #10b981;
            font-weight: 700;
          }

          &.team-b {
            justify-content: flex-end;
          }
        }

        .match-score {
          padding: 0 20px;

          .score {
            font-size: 18px;
            font-weight: 700;
            color: #1f2937;
          }
        }
      }
    }
  }

  .battle-progress {
    .progress-bar {
      height: 12px;
      background: #e5e7eb;
      border-radius: 6px;
      display: flex;
      overflow: hidden;
      margin-bottom: 8px;

      .progress-a {
        height: 100%;
        background: linear-gradient(90deg, #ef4444 0%, #dc2626 100%);
        transition: width 0.3s ease;
      }

      .progress-b {
        height: 100%;
        background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
        transition: width 0.3s ease;
        margin-left: auto;
      }
    }

    .progress-labels {
      display: flex;
      justify-content: space-between;
      font-size: 12px;
      font-weight: 600;

      .label-a {
        color: #dc2626;
      }

      .label-b {
        color: #2563eb;
      }
    }
  }
}
</style>
