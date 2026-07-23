<script setup lang="ts">
const props = defineProps<{
  status: string
}>()

const badgeMap: Record<string, { label: string; color: string; bg: string }> = {
  new: { label: 'New', color: 'var(--new)', bg: 'var(--new-bg)' },
  backed: { label: 'Backed Up', color: 'var(--backed)', bg: 'var(--backed-bg)' },
  backed_up: { label: 'Backed Up', color: 'var(--backed)', bg: 'var(--backed-bg)' },
  modified: { label: 'Modified', color: 'var(--modified)', bg: 'var(--modified-bg)' },
  failed: { label: 'Failed', color: 'var(--failed)', bg: 'var(--failed-bg)' },
  success: { label: 'Success', color: 'var(--new)', bg: 'var(--new-bg)' },
  partial: { label: 'Partial', color: 'var(--modified)', bg: 'var(--modified-bg)' },
  error: { label: 'Error', color: 'var(--failed)', bg: 'var(--failed-bg)' },
  running: { label: 'Running', color: 'var(--accent)', bg: 'var(--accent-dim)' },
  paused: { label: 'Paused', color: 'var(--modified)', bg: 'var(--modified-bg)' },
  done: { label: 'Complete', color: 'var(--new)', bg: 'var(--new-bg)' },
  verified: { label: 'Verified', color: 'var(--new)', bg: 'var(--new-bg)' }
}

const badge = computed(() => {
  return (
    badgeMap[props.status] ?? {
      label: props.status,
      color: 'var(--text-sec)',
      bg: 'var(--surface)'
    }
  )
})
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
