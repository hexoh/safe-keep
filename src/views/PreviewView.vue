<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useScan } from '@/composables/useScan'
// import type { FileEntry } from '@/types/file'

const route = useRoute()
const router = useRouter()
const { scanning, progress, result, error, scan } = useScan()

const selectedPaths = ref<string[]>([])

function selectAll() {
  if (result.value) {
    selectedPaths.value = result.value.files.map((f) => f.path)
  }
}

onMounted(async () => {
  const source = route.query.source as string
  // const dest = route.query.dest as string

  if (!source) {
    router.push({ name: 'home' })
    return
  }

  await scan({
    source_path: source,
    include_images: route.query.images !== '0',
    include_videos: route.query.videos !== '0'
  })

  if (result.value) {
    selectAll()
  }
})

const totalFiles = computed(() => result.value?.total_files ?? 0)
const totalSize = computed(() => {
  if (!result.value) return '0 B'
  const bytes = result.value.total_size
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1024 && unitIdx < units.length - 1) {
    size /= 1024
    unitIdx++
  }
  return `${size.toFixed(unitIdx > 0 ? 2 : 0)} ${units[unitIdx]}`
})

const selectedSize = computed(() => {
  if (!result.value || selectedPaths.value.length === 0) return '0 B'
  const bytes = result.value.files
    .filter((f) => selectedPaths.value.includes(f.path))
    .reduce((acc, f) => acc + f.file_size, 0)
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1024 && unitIdx < units.length - 1) {
    size /= 1024
    unitIdx++
  }
  return `${size.toFixed(unitIdx > 0 ? 2 : 0)} ${units[unitIdx]}`
})

function goBack() {
  router.push({ name: 'home' })
}
</script>

<template>
  <div class="preview-view">
    <h1>{{ $t('preview.title') }}</h1>

    <div v-if="scanning" class="scanning-state">
      <el-progress
        :percentage="progress.total ? Math.round((progress.scanned / progress.total) * 100) : 0"
      />
      <p>{{ $t('common.loading') }}... {{ progress.scanned }} / {{ progress.total ?? '?' }}</p>
    </div>

    <div v-else-if="error" class="error-state">
      <el-result status="error" :title="$t('common.error')" :sub-title="error" />
    </div>

    <template v-else-if="result">
      <div class="stats-row">
        <StatsCard :title="$t('preview.total_files')" :value="totalFiles" type="primary" />
        <StatsCard :title="$t('preview.total_size')" :value="totalSize" type="info" />
        <StatsCard
          :title="$t('preview.selected_count')"
          :value="selectedPaths.length"
          type="success"
        />
      </div>

      <el-card shadow="never" class="table-card">
        <FileTable v-model:selected="selectedPaths" :files="result.files" :loading="scanning" />
      </el-card>

      <div class="bottom-actions">
        <el-text>
          {{ $t('preview.selected_count') }}: {{ selectedPaths.length }} ({{ selectedSize }})
        </el-text>
        <div class="action-buttons">
          <el-button @click="goBack">{{ $t('preview.go_back') }}</el-button>
          <el-button type="primary" :disabled="selectedPaths.length === 0">
            {{ $t('preview.start_backup') }}
          </el-button>
        </div>
      </div>
    </template>

    <div v-else class="empty-state">
      <el-empty :description="$t('common.no_data')" />
    </div>
  </div>
</template>
