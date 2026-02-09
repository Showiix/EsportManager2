<template>
  <div class="tournament-completion-banner">
    <div class="banner-icon">
      <span class="trophy">🏆</span>
    </div>
    <div class="banner-content">
      <div class="banner-title">{{ title }}</div>
      <div class="banner-champion">
        恭喜 <span class="champion-name">{{ champion }}</span>
      </div>
      <div class="banner-description">{{ description }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  champion?: string
  description: string
}>()
</script>

<style scoped lang="scss">
.tournament-completion-banner {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 20px 24px;
  background: linear-gradient(135deg, #fffbeb 0%, #ffffff 100%);
  border: 1px solid #f59e0b40;
  border-radius: 12px;
  margin-top: 32px;

  .banner-icon {
    flex-shrink: 0;
    width: 52px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
    border-radius: 50%;

    .trophy {
      font-size: 24px;
      line-height: 1;
    }
  }

  .banner-content {
    flex: 1;
    min-width: 0;

    .banner-title {
      font-size: 16px;
      font-weight: 700;
      color: #92400e;
      margin-bottom: 4px;
    }

    .banner-champion {
      font-size: 14px;
      color: #374151;
      margin-bottom: 2px;

      .champion-name {
        color: #f59e0b;
        font-weight: 700;
        font-size: 16px;
      }
    }

    .banner-description {
      font-size: 13px;
      color: #6b7280;
      line-height: 1.5;
    }
  }
}
</style>
