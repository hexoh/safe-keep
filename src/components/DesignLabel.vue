<script setup lang="ts">
defineProps<{
  text?: string
}>()
</script>

<template>
  <span class="design-label">
    <slot>{{ text }}</slot>
  </span>
</template>

<style scoped>
.design-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  color: var(--text-sec);
  text-transform: uppercase;
}
</style>
