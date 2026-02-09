<template>
  <div class="tournament-completion-section">
    <div class="standings-header">
      <h3>{{ title }}</h3>
      <span v-if="subtitle" class="subtitle">{{ subtitle }}</span>
    </div>
    <div class="standings-grid">
      <div
        v-for="item in standings"
        :key="item.rank"
        class="standing-item"
        :class="rankClass(item.rank)"
      >
        <template v-if="variant === 'region' && item.regionFlag">
          <div class="rank-icon">{{ rankIcon(item.rank) }}</div>
          <div class="rank-label">{{ item.label }}</div>
          <div class="region-flag-large">{{ item.regionFlag }}</div>
          <div class="team-name">{{ item.name }}</div>
          <div v-if="item.pointsDetail?.length" class="points-detail">
            <div v-for="(line, i) in item.pointsDetail" :key="i">{{ line }}</div>
          </div>
          <div v-else class="points-badge">{{ item.points }}</div>
        </template>
        <template v-else>
          <div class="rank-icon">{{ rankIcon(item.rank) }}</div>
          <div class="rank-label">{{ item.label }}</div>
          <div class="team-name">{{ item.name }}</div>
          <div v-if="item.regionName" class="region-name">{{ item.regionName }}</div>
          <div class="points-badge">{{ item.points }}</div>
        </template>
      </div>
    </div>

    <slot />

    <TournamentCompletionBanner
      :title="bannerTitle"
      :champion="bannerChampion"
      :description="bannerDescription || ''"
    />
  </div>
</template>

<script setup lang="ts">
import TournamentCompletionBanner from './TournamentCompletionBanner.vue'
import type { StandingItem } from '@/types/tournament'

withDefaults(defineProps<{
  standings: StandingItem[]
  title?: string
  subtitle?: string
  bannerTitle: string
  bannerChampion: string
  bannerDescription?: string
  variant?: 'team' | 'region'
}>(), {
  title: '最终排名与积分',
  variant: 'team'
})

const rankClass = (rank: 1 | 2 | 3 | 4) => {
  const map: Record<number, string> = { 1: 'champion', 2: 'runner-up', 3: 'third', 4: 'fourth' }
  return map[rank]
}

const rankIcon = (rank: 1 | 2 | 3 | 4) => {
  const map: Record<number, string> = { 1: '\u{1F451}', 2: '\u{1F948}', 3: '\u{1F949}', 4: '4' }
  return map[rank]
}
</script>

<style scoped lang="scss">
.tournament-completion-section {
  margin-top: 32px;
  padding: 24px;
  background: linear-gradient(135deg, #fefce8 0%, #fef9c3 100%);
  border-radius: 16px;
  border: 2px solid #fbbf24;

  .standings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 2px dashed #fbbf24;

    h3 {
      margin: 0;
      font-size: 22px;
      font-weight: 700;
      color: #92400e;
    }

    .subtitle {
      font-size: 13px;
      color: #a16207;
      background: rgba(251, 191, 36, 0.3);
      padding: 4px 12px;
      border-radius: 20px;
      font-weight: 500;
    }
  }

  .standings-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-bottom: 24px;

    .standing-item {
      padding: 20px 16px;
      border-radius: 16px;
      text-align: center;
      border: 2px solid;
      background: white;
      transition: all 0.3s ease;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);

      &:hover {
        transform: translateY(-4px);
        box-shadow: 0 8px 20px rgba(0, 0, 0, 0.12);
      }

      .rank-icon {
        font-size: 36px;
        margin-bottom: 8px;
        line-height: 1;
      }

      .rank-label {
        font-size: 12px;
        font-weight: 600;
        color: #6b7280;
        text-transform: uppercase;
        letter-spacing: 1px;
        margin-bottom: 8px;
      }

      .team-name {
        font-size: 20px;
        font-weight: 800;
        margin-bottom: 12px;
        color: #1f2937;
      }

      .region-name {
        font-size: 14px;
        color: #6b7280;
        margin-bottom: 8px;
      }

      .region-flag-large {
        font-size: 48px;
        margin-bottom: 8px;
      }

      .points-badge {
        display: inline-block;
        font-size: 14px;
        font-weight: 700;
        padding: 6px 14px;
        border-radius: 20px;
        background: linear-gradient(135deg, #10b981, #059669);
        color: white;
        box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
      }

      .points-detail {
        font-size: 13px;
        font-weight: 600;
        color: #10b981;
        line-height: 1.8;
      }

      &.champion {
        border-color: #fbbf24;
        background: linear-gradient(135deg, #fffbeb, #fef3c7);
        position: relative;
        overflow: hidden;

        &::before {
          content: '';
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          height: 4px;
          background: linear-gradient(90deg, #fbbf24, #f59e0b, #fbbf24);
        }

        .rank-icon {
          font-size: 42px;
          filter: drop-shadow(0 2px 4px rgba(251, 191, 36, 0.5));
        }

        .rank-label {
          color: #92400e;
        }

        .team-name {
          color: #92400e;
        }

        .points-badge {
          background: linear-gradient(135deg, #f59e0b, #d97706);
          box-shadow: 0 2px 8px rgba(245, 158, 11, 0.4);
        }
      }

      &.runner-up {
        border-color: #9ca3af;
        background: linear-gradient(135deg, #f9fafb, #f3f4f6);

        .rank-icon {
          filter: drop-shadow(0 2px 4px rgba(156, 163, 175, 0.4));
        }

        .rank-label {
          color: #4b5563;
        }
      }

      &.third {
        border-color: #d97706;
        background: linear-gradient(135deg, #fffbeb, #fef3c7);

        .rank-icon {
          filter: drop-shadow(0 2px 4px rgba(217, 119, 6, 0.4));
        }

        .rank-label {
          color: #92400e;
        }
      }

      &.fourth {
        border-color: #60a5fa;
        background: linear-gradient(135deg, #eff6ff, #dbeafe);

        .rank-icon {
          font-weight: 900;
          font-size: 28px;
          color: #3b82f6;
          background: linear-gradient(135deg, #3b82f6, #2563eb);
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
        }

        .rank-label {
          color: #1d4ed8;
        }
      }
    }
  }
}

@media (max-width: 1200px) {
  .tournament-completion-section {
    padding: 16px;

    .standings-header {
      flex-direction: column;
      gap: 12px;
      text-align: center;
    }

    .standings-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;

      .standing-item {
        padding: 16px 12px;

        .rank-icon {
          font-size: 28px;
        }

        .team-name {
          font-size: 16px;
        }

        .points-badge {
          font-size: 12px;
          padding: 4px 10px;
        }
      }
    }
  }
}
</style>
