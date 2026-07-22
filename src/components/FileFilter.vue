<script setup lang="ts">
const filter = defineModel<{
  includeImages: boolean
  includeVideos: boolean
  minSize: number
  maxSize: number
}>('filter', { required: true })
</script>

<template>
  <div class="file-filter">
    <div class="filter-row">
      <el-checkbox v-model="filter.includeImages" size="small">
        {{ $t('home.filter_images') }}
      </el-checkbox>
      <el-checkbox v-model="filter.includeVideos" size="small">
        {{ $t('home.filter_videos') }}
      </el-checkbox>
    </div>
    <div class="filter-row size-range-row">
      <span class="size-label">{{ $t('home.filter_min_size') }}:</span>
      <el-input-number
        v-model="filter.minSize"
        :min="0"
        :step="1024"
        size="small"
        controls-position="right"
      />
      <span class="size-label">{{ $t('home.filter_max_size') }}:</span>
      <el-input-number
        v-model="filter.maxSize"
        :min="0"
        :step="1024"
        size="small"
        controls-position="right"
      />
      <span class="size-unit">bytes</span>
    </div>
  </div>
</template>

<style scoped>
.file-filter {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.size-range-row {
  flex-wrap: wrap;
}

.size-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.size-unit {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}
</style>
