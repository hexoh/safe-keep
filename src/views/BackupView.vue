<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useBackup } from '@/composables/useBackup'
import type { FileEntry } from '@/types/file'

const route = useRoute()
const router = useRouter()

const { running, paused, progress, result, logs, error, start, pause, resume, cancel } = useBackup()

function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIdx = 0
  while (size >= 1024 && unitIdx < units.length - 1) {
    size /= 1024
    unitIdx++
  }
  return `${size.toFixed(unitIdx > 0 ? 2 : 0)} ${units[unitIdx]}`
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${Math.round(secs)}s`
  const m = Math.floor(secs / 60)
  const s = Math.round(secs % 60)
  return `${m}m ${s}s`
}

const overallPercentage = computed(() => {
  if (!progress.value) return 0
  return progress.value.total_files > 0
    ? (progress.value.completed_files / progress.value.total_files) * 100
    : 0
})

const fileProgressPercent = computed(() => {
  if (!progress.value) return 0
  return progress.value.current_file_progress * 100
})

const transferredText = computed(() => {
  if (!progress.value) return '0 B'
  return formatBytes(progress.value.copied_bytes)
})

const totalText = computed(() => {
  if (!progress.value) return '0 B'
  return formatBytes(progress.value.total_bytes)
})

const completedText = computed(() => {
  if (!progress.value) return '0'
  return `${progress.value.completed_files} / ${progress.value.total_files}`
})

const logContainer = ref<HTMLElement | null>(null)

watch(logs, () => {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
})

onMounted(async () => {
  const source = route.query.source as string
  const dest = route.query.dest as string
  const filesJson = route.query.files as string

  if (!source || !dest || !filesJson) {
    router.push({ name: 'home' })
    return
  }

  let files: FileEntry[]
  try {
    files = JSON.parse(filesJson)
  } catch {
    router.push({ name: 'home' })
    return
  }

  await start({
    sourceRoot: source,
    destPath: dest,
    files: files.map((f) => ({
      source_path: f.path,
      relative_path: f.relative_path,
      file_size: f.file_size
    })),
    conflictStrategy: 'rename',
    concurrency: 4
  })
})

function goToHistory() {
  router.push({ name: 'history' })
}

function goBack() {
  router.push({ name: 'home' })
}
</script>

<template>
  <div class="backup-view">
    <h1>{{ $t('backup.title') }}</h1>

    <div v-if="error" class="error-state">
      <el-result status="error" :title="$t('common.error')" :sub-title="error">
        <template #extra>
          <el-button @click="goBack">{{ $t('common.back') }}</el-button>
        </template>
      </el-result>
    </div>

    <div v-else-if="result" class="complete-state">
      <el-result
        :status="result.failed > 0 ? 'warning' : 'success'"
        :title="$t('backup.completed')"
      >
        <template #sub-title>
          <div class="result-stats">
            <p>{{ $t('backup.success_count') }}: {{ result.succeeded }}</p>
            <p v-if="result.failed > 0">{{ $t('backup.failed_count') }}: {{ result.failed }}</p>
            <p>{{ $t('backup.duration') }}: {{ formatDuration(result.duration_secs) }}</p>
            <p>{{ $t('backup.avg_speed') }}: {{ result.avg_speed_mbps.toFixed(2) }} MB/s</p>
          </div>
        </template>
        <template #extra>
          <el-button @click="goToHistory">{{ $t('backup.view_details') }}</el-button>
          <el-button @click="goBack">{{ $t('common.back') }}</el-button>
        </template>
      </el-result>
    </div>

    <div v-else class="progress-container">
      <ProgressBar
        :label="$t('backup.overall_progress')"
        :percentage="overallPercentage"
        :show-text="true"
      />

      <div class="current-file-section" v-if="progress?.current_file">
        <p class="current-file-label">{{ $t('backup.current_file') }}</p>
        <p class="current-file-name">{{ progress.current_file }}</p>
        <el-progress :percentage="fileProgressPercent" :stroke-width="8" :show-text="false" />
      </div>

      <SpeedIndicator
        :speed="progress?.speed_mbps ?? 0"
        :remaining="progress?.remaining_secs ?? null"
        :transferred="transferredText"
        :total="totalText"
      />

      <div class="files-count">
        <el-tag type="info">{{ $t('backup.transferred') }}: {{ completedText }}</el-tag>
      </div>

      <div class="control-buttons">
        <el-button v-if="!paused" type="warning" :disabled="!running" @click="pause">
          {{ $t('backup.pause') }}
        </el-button>
        <el-button v-else type="success" :disabled="!running" @click="resume">
          {{ $t('backup.resume') }}
        </el-button>
        <el-button type="danger" :disabled="!running" @click="cancel">
          {{ $t('backup.cancel') }}
        </el-button>
      </div>

      <el-card shadow="never" class="log-card">
        <template #header>
          <span>{{ $t('backup.log') }}</span>
        </template>
        <div ref="logContainer" class="log-container">
          <p v-for="(msg, i) in logs" :key="i" class="log-line">{{ msg }}</p>
          <p v-if="logs.length === 0" class="log-empty">{{ $t('common.loading') }}...</p>
        </div>
      </el-card>
    </div>
  </div>
</template>
