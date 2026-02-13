<template>
  <TournamentCompletionSection
    v-if="status === 'completed'"
    :standings="standings"
    title="赛区最终排名与积分"
    variant="region"
    banner-title="ICP洲际对抗赛已完成！"
    :banner-champion="championName"
    banner-description="成为本届最强赛区！"
  />
</template>

<script setup lang="ts">
import TournamentCompletionSection from '@/components/common/TournamentCompletionSection.vue'
import type { StandingItem } from '@/types/tournament'

defineProps<{
  status: string
  standings: StandingItem[]
  championName: string
}>()
</script>