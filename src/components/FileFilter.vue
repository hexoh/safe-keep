<script setup lang="ts">
const filter = defineModel<{
  includeImages: boolean
  includeVideos: boolean
  minSize: number
  maxSize: number
}>('filter', { required: true })

// const sizeUnitOptions = ['B', 'KB', 'MB', 'GB']
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
  </div>
</template>
