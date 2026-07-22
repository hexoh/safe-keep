<script setup lang="ts">
defineProps<{
  percentage: number
  label?: string
  showText?: boolean
}>()
</script>

<template>
  <div class="progress-bar-wrapper">
    <div v-if="label" class="progress-label">{{ label }}</div>
    <el-progress
      :percentage="Math.round(percentage)"
      :show-text="showText !== false"
      :stroke-width="20"
      :format="() => `${Math.round(percentage)}%`"
    />
  </div>
</template>
