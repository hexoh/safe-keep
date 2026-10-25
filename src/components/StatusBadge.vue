<script setup lang="ts">
const props = defineProps<{
  status: string
}>()

const badgeMap: Record<string, { color: string; bg: string }> = {
  new: { color: 'var(--new)', bg: 'var(--new-bg)' },
  backed: { color: 'var(--backed)', bg: 'var(--backed-bg)' },
  backed_up: { color: 'var(--backed)', bg: 'var(--backed-bg)' },
  modified: { color: 'var(--modified)', bg: 'var(--modified-bg)' },
  changed: { color: 'var(--modified)', bg: 'var(--modified-bg)' },
  failed: { color: 'var(--failed)', bg: 'var(--failed-bg)' },
  success: { color: 'var(--new)', bg: 'var(--new-bg)' },
  partial: { color: 'var(--modified)', bg: 'var(--modified-bg)' },
  error: { color: 'var(--failed)', bg: 'var(--failed-bg)' },
  running: { color: 'var(--accent)', bg: 'var(--accent-dim)' },
  paused: { color: 'var(--modified)', bg: 'var(--modified-bg)' },
  completed: { color: 'var(--new)', bg: 'var(--new-bg)' },
  cancelled: { color: 'var(--modified)', bg: 'var(--modified-bg)' },
  done: { color: 'var(--new)', bg: 'var(--new-bg)' },
  verified: { color: 'var(--new)', bg: 'var(--new-bg)' }
}

const badge = computed(
  () => badgeMap[props.status] ?? { color: 'var(--text-sec)', bg: 'var(--surface)' }
)
</script>

<template>
  <span class="status-badge" :style="{ color: badge.color, background: badge.bg }">
    <span class="dot" :style="{ background: badge.color }" />
    {{ $t(`status.${status}`) }}
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
  align-items: center;
  gap: 5px;
}

.dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
