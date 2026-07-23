<script setup lang="ts">
defineProps<{
  value: number
  color?: string
  height?: number
}>()
</script>

<template>
  <div
    class="progress-track"
    :style="{ height: `${height ?? 6}px`, borderRadius: `${height ?? 6}px` }"
  >
    <div
      class="progress-fill"
      :style="{
        width: `${Math.min(100, Math.max(0, value))}%`,
        background: color ?? 'var(--accent)',
        borderRadius: `${height ?? 6}px`
      }"
    />
  </div>
</template>

<style scoped>
.progress-track {
  overflow: hidden;
  background: var(--border);
}

.progress-fill {
  height: 100%;
  transition: width 0.35s ease;
}
</style>
