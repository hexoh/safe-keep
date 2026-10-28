<script setup lang="ts">
defineProps<{
  title: string
  hint?: string
}>()
</script>

<template>
  <div class="empty-state">
    <span class="empty-title">{{ title }}</span>
    <span v-if="hint" class="empty-hint">{{ hint }}</span>
    <div v-if="$slots.default" class="empty-actions">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  width: 100%;
  max-width: 520px;
  padding: 8px 0;
  margin: 0 auto;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  justify-content: center;
  text-align: center;
}

.empty-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text);
}

.empty-hint {
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-sec);
}

.empty-actions {
  display: flex;
  margin-top: 4px;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
}
</style>
