<script setup lang="ts">
import { useRestore } from '@/composables/useRestore'
import { formatBytes, formatDuration } from '@/utils/format'
import type { RestoreFile } from '@/types/restore'

const { files, loading, progress, result, error, fetchFiles, start, cancel } = useRestore()

const step = ref<'select' | 'executing' | 'result'>('select')
const selectedFiles = ref<RestoreFile[]>([])
const restoreTarget = ref('')
const conflictStrategy = ref<'overwrite' | 'skip' | 'keep_both'>('overwrite')

const totalSelectedSize = computed(() => {
  const bytes = selectedFiles.value.reduce((acc, f) => acc + f.file_size, 0)
  return formatBytes(bytes)
})

const allSources = computed(() => {
  const set = new Set(files.value.map((f) => f.source_root))
  return Array.from(set)
})

const selectedSource = ref('')

const filteredFiles = computed(() => {
  if (!selectedSource.value) return []
  return files.value.filter((f) => f.source_root === selectedSource.value)
})

function toggleFile(file: RestoreFile) {
  const idx = selectedFiles.value.findIndex((f) => f.file_id === file.file_id)
  if (idx >= 0) {
    selectedFiles.value.splice(idx, 1)
  } else {
    selectedFiles.value.push(file)
  }
}

async function startRestore() {
  if (selectedFiles.value.length === 0 || !restoreTarget.value) return
  step.value = 'executing'
  await start({
    sourceRoot: selectedSource.value,
    restoreTarget: restoreTarget.value,
    files: selectedFiles.value,
    conflictStrategy: conflictStrategy.value
  })
  step.value = 'result'
}

function resetForm() {
  step.value = 'select'
  selectedFiles.value = []
  restoreTarget.value = ''
}

function handleSourceChange() {
  selectedFiles.value = []
}

onMounted(() => {
  fetchFiles()
})
</script>

<template>
  <div class="restore-view">
    <h1>{{ $t('restore.title') }}</h1>

    <!-- Step 1: Select files and target -->
    <template v-if="step === 'select'">
      <el-card shadow="never" class="select-card">
        <el-form label-width="140px">
          <el-form-item :label="$t('restore.select_source')">
            <el-select
              v-model="selectedSource"
              style="width: 100%"
              :placeholder="$t('restore.select_source')"
              @change="handleSourceChange"
            >
              <el-option v-for="src in allSources" :key="src" :label="src" :value="src" />
            </el-select>
          </el-form-item>

          <el-form-item :label="$t('restore.restore_target')">
            <el-input v-model="restoreTarget" :placeholder="$t('restore.restore_target')" />
          </el-form-item>

          <el-form-item :label="$t('settings.conflict_strategy')">
            <el-radio-group v-model="conflictStrategy">
              <el-radio value="overwrite">{{ $t('restore.overwrite') }}</el-radio>
              <el-radio value="skip">{{ $t('restore.skip') }}</el-radio>
              <el-radio value="keep_both">{{ $t('restore.keep_both') }}</el-radio>
            </el-radio-group>
          </el-form-item>
        </el-form>
      </el-card>

      <div v-if="loading" class="loading-state">
        <el-skeleton :rows="3" animated />
      </div>

      <template v-else-if="selectedSource">
        <el-card shadow="never" class="table-card">
          <template #header>
            <span>{{ $t('restore.file_list') }} ({{ filteredFiles.length }})</span>
          </template>

          <el-table
            :data="filteredFiles"
            style="width: 100%"
            size="small"
            @row-click="(row: RestoreFile) => toggleFile(row)"
          >
            <el-table-column type="selection" width="40" />
            <el-table-column
              prop="file_name"
              :label="$t('preview.file_name')"
              min-width="200"
              show-overflow-tooltip
            />
            <el-table-column :label="$t('preview.size')" width="100" sortable>
              <template #default="{ row }: { row: RestoreFile }">
                {{ formatBytes(row.file_size) }}
              </template>
            </el-table-column>
            <el-table-column prop="backed_up_at" :label="$t('cleanup.backed_up_at')" width="180" />
          </el-table>
        </el-card>

        <div class="bottom-actions">
          <el-text>
            {{ $t('preview.selected_count') }}: {{ selectedFiles.length }} ({{ totalSelectedSize }})
          </el-text>
          <el-button
            type="primary"
            :disabled="selectedFiles.length === 0 || !restoreTarget"
            @click="startRestore"
          >
            {{ $t('restore.start_restore') }}
          </el-button>
        </div>
      </template>

      <div v-else-if="!loading && !selectedSource" class="empty-state">
        <el-empty :description="$t('common.no_data')" />
      </div>

      <div v-if="error" class="error-msg">
        <el-alert :title="error" type="error" show-icon :closable="false" />
      </div>
    </template>

    <!-- Step 2: Executing -->
    <template v-else-if="step === 'executing'">
      <el-card shadow="never">
        <div class="executing-state">
          <el-progress
            :percentage="
              progress ? Math.round((progress.processed / progress.total_files) * 100) : 0
            "
            :stroke-width="20"
          />
          <p v-if="progress" class="executing-current">
            {{ $t('common.processing') }}: {{ progress.current_file }}
          </p>
          <p class="executing-stats">
            {{ $t('backup.success_count') }}: {{ progress?.succeeded ?? 0 }} |
            {{ $t('backup.failed_count') }}: {{ progress?.failed ?? 0 }}
          </p>
          <el-button type="danger" @click="cancel">
            {{ $t('backup.cancel') }}
          </el-button>
        </div>
      </el-card>
    </template>

    <!-- Step 3: Result -->
    <template v-else-if="step === 'result' && result">
      <el-result
        :status="result.failed > 0 ? 'warning' : 'success'"
        :title="$t('backup.completed')"
      >
        <template #sub-title>
          <div class="result-stats">
            <p>{{ $t('backup.success_count') }}: {{ result.succeeded }}</p>
            <p v-if="result.failed > 0">{{ $t('backup.failed_count') }}: {{ result.failed }}</p>
            <p>{{ $t('backup.duration') }}: {{ formatDuration(result.duration_secs) }}</p>
          </div>
        </template>
        <template #extra>
          <el-button @click="resetForm">{{ $t('common.back') }}</el-button>
        </template>
      </el-result>

      <el-card v-if="result.errors.length > 0" shadow="never" class="error-card">
        <template #header>
          <span style="color: var(--el-color-danger)">{{ $t('common.error') }}</span>
        </template>
        <p v-for="(err, i) in result.errors" :key="i" class="error-line">{{ err }}</p>
      </el-card>
    </template>
  </div>
</template>

<style scoped>
.select-card {
  margin-bottom: 16px;
}

.bottom-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
}

.executing-state {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
}

.executing-current {
  font-size: 14px;
}

.executing-stats {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}
</style>
