<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useScan } from '@/composables/useScan'
import { formatBytes } from '@/utils/format'

const route = useRoute()
const router = useRouter()
const { scanning, progress, result, error, scan } = useScan()

const selectedPaths = ref<string[]>([])
const destPath = computed(() => route.query.dest as string)

function selectAll() {
  if (result.value) {
    selectedPaths.value = result.value.files.map((f) => f.path)
  }
}

onMounted(async () => {
  const source = route.query.source as string

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
  return formatBytes(result.value.total_size)
})

const newFiles = computed(() => result.value?.files.filter((f) => f.status === 'new') ?? [])
const backedUpFiles = computed(
  () => result.value?.files.filter((f) => f.status === 'backed_up') ?? []
)
const changedFiles = computed(() => result.value?.files.filter((f) => f.status === 'changed') ?? [])

const selectedSize = computed(() => {
  if (!result.value || selectedPaths.value.length === 0) return '0 B'
  const bytes = result.value.files
    .filter((f) => selectedPaths.value.includes(f.path))
    .reduce((acc, f) => acc + f.file_size, 0)
  return formatBytes(bytes)
})

function goBack() {
  router.push({ name: 'home' })
}

function startBackup() {
  if (!result.value || !destPath.value) return

  const selected = result.value.files.filter((f) => selectedPaths.value.includes(f.path))

  router.push({
    name: 'backup',
    query: {
      source: route.query.source as string,
      dest: destPath.value,
      files: JSON.stringify(selected)
    }
  })
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
        <StatsCard :title="$t('preview.new_files')" :value="newFiles.length" type="success" />
        <StatsCard
          :title="$t('preview.backed_up_files')"
          :value="backedUpFiles.length"
          type="info"
        />
        <StatsCard
          :title="$t('preview.changed_files')"
          :value="changedFiles.length"
          type="warning"
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
          <el-button type="primary" :disabled="selectedPaths.length === 0" @click="startBackup">
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
