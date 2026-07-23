<script setup lang="ts">
defineProps<{
  label: string
  value: string | number
  sub?: string
  color?: string
}>()
</script>

<template>
  <DesignCard padding="14px 18px">
    <div class="stat-card">
      <DesignLabel :text="label" />
      <span class="stat-value" :style="color ? { color } : {}">{{ value }}</span>
      <span v-if="sub" class="stat-sub">{{ sub }}</span>
    </div>
  </DesignCard>
</template>

<style scoped>
.stat-card {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.stat-value {
  font-family: var(--mono);
  font-size: 28px;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: -0.03em;
  color: var(--text);
}

.stat-sub {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
