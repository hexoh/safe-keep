<script setup lang="ts">
defineProps<{
  padding?: string
}>()
</script>

<template>
  <div class="design-card" :style="padding ? { padding } : {}">
    <slot />
  </div>
</template>

<style scoped>
.design-card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 10px;
}
</style>
