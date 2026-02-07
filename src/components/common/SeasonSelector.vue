<template>
  <el-select
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    placeholder="选择赛季"
    :style="{ width: width || '120px' }"
  >
    <el-option
      v-if="showAll"
      label="全部赛季"
      :value="0"
    />
    <el-option
      v-for="s in seasonOptions"
      :key="s.value"
      :label="s.value === currentSeason ? `S${s.value} (当前)` : `S${s.value}`"
      :value="s.value"
    />
  </el-select>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useSeasonStore } from '@/stores/useSeasonStore'

defineProps<{
  modelValue: number
  showAll?: boolean
  width?: string
}>()

defineEmits<{
  'update:modelValue': [value: number]
}>()

const seasonStore = useSeasonStore()
const { seasonOptions, currentSeason } = storeToRefs(seasonStore)
</script>
