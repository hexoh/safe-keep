<script setup lang="ts">
defineProps<{
  speed: number
  remaining: number | null
  transferred: string
  total: string
}>()
</script>

<template>
  <div class="speed-indicator">
    <div class="speed-item">
      <span class="speed-label">{{ $t('backup.speed') }}</span>
      <span class="speed-value">{{ speed.toFixed(1) }} MB/s</span>
    </div>
    <div class="speed-item">
      <span class="speed-label">{{ $t('backup.remaining') }}</span>
      <span class="speed-value">
        {{ remaining != null ? `${Math.round(remaining)}s` : '--' }}
      </span>
    </div>
    <div class="speed-item">
      <span class="speed-label">{{ $t('backup.transferred') }}</span>
      <span class="speed-value">{{ transferred }} / {{ total }}</span>
    </div>
  </div>
</template>
