<script setup lang="ts">
import { useCleanup } from '@/composables/useCleanup'
import { getSourceRoots } from '@/api/cleanup'
import { formatBytes, formatDate } from '@/utils/format'
import type { CleanupFilter, CleanupFile } from '@/types/cleanup'

const { loading, dryRunResult, cleanupResult, progress, error, dryRun, execute, cancel } =
  useCleanup()

const sourceRoots = ref<string[]>([])
const selectedSource = ref('')
const beforeDays = ref<number>(30)
const fileType = ref<'all' | 'image' | 'video'>('all')
const minSize = ref<number>(0)
const maxSize = ref<number>(0)
const permanent = ref(false)
const step = ref<'filter' | 'preview' | 'executing' | 'result'>('filter')
const selectedFiles = ref<CleanupFile[]>([])
const selectAll = ref(true)

onMounted(async () => {
  try {
    sourceRoots.value = await getSourceRoots()
  } catch {}
})

async function runDryRun() {
  if (!selectedSource.value) return

  const filter: CleanupFilter = {
    source_root: selectedSource.value,
    before_days: beforeDays.value > 0 ? beforeDays.value : undefined,
    is_image: fileType.value === 'all' ? undefined : fileType.value === 'image',
    min_size: minSize.value > 0 ? minSize.value : undefined,
    max_size: maxSize.value > 0 ? maxSize.value : undefined
  }

  await dryRun(filter)
  if (dryRunResult.value) {
    selectedFiles.value = [...dryRunResult.value.files]
    step.value = 'preview'
  }
}

function toggleSelectAll() {
  if (!dryRunResult.value) return
  selectAll.value = !selectAll.value
  selectedFiles.value = selectAll.value ? [...dryRunResult.value.files] : []
}

async function confirmCleanup() {
  if (selectedFiles.value.length === 0) return
  step.value = 'executing'
  await execute(selectedFiles.value, permanent.value)
  step.value = 'result'
}

function resetForm() {
  step.value = 'filter'
  dryRunResult.value = null
  cleanupResult.value = null
  error.value = null
}

const totalFreed = computed(() => {
  if (!dryRunResult.value) return '0 B'
  return formatBytes(dryRunResult.value.total_size)
})

const selectedFreed = computed(() => {
  const bytes = selectedFiles.value.reduce((acc, f) => acc + f.file_size, 0)
  return formatBytes(bytes)
})
</script>

<template>
  <div class="cleanup-view">
    <h1>{{ $t('cleanup.title') }}</h1>

    <!-- Step 1: Filter Conditions -->
    <template v-if="step === 'filter'">
      <el-card shadow="never" class="filter-card">
        <template #header>{{ $t('cleanup.filter_condition') }}</template>

        <el-form label-width="120px">
          <el-form-item :label="$t('cleanup.source')">
            <el-select v-model="selectedSource" style="width: 100%">
              <el-option v-for="root in sourceRoots" :key="root" :label="root" :value="root" />
            </el-select>
          </el-form-item>

          <el-form-item :label="$t('cleanup.time_range')">
            <el-radio-group v-model="beforeDays">
              <el-radio-button :value="7">7 {{ $t('common.days') }}</el-radio-button>
              <el-radio-button :value="30">30 {{ $t('common.days') }}</el-radio-button>
              <el-radio-button :value="90">90 {{ $t('common.days') }}</el-radio-button>
              <el-radio-button :value="180">180 {{ $t('common.days') }}</el-radio-button>
              <el-radio-button :value="365">365 {{ $t('common.days') }}</el-radio-button>
            </el-radio-group>
          </el-form-item>

          <el-form-item :label="$t('cleanup.file_type')">
            <el-radio-group v-model="fileType">
              <el-radio-button value="all">{{ $t('common.all') }}</el-radio-button>
              <el-radio-button value="image">{{ $t('common.images') }}</el-radio-button>
              <el-radio-button value="video">{{ $t('common.videos') }}</el-radio-button>
            </el-radio-group>
          </el-form-item>

          <el-form-item :label="$t('cleanup.file_size')">
            <div class="size-range">
              <el-input-number v-model="minSize" :min="0" :step="1024" size="small" />
              <span class="size-sep">~</span>
              <el-input-number v-model="maxSize" :min="0" :step="1024" size="small" />
              <span class="size-unit">bytes</span>
            </div>
          </el-form-item>

          <el-form-item :label="$t('cleanup.delete_strategy')">
            <el-switch
              v-model="permanent"
              :active-text="$t('cleanup.permanent_delete')"
              :inactive-text="$t('cleanup.move_to_trash')"
            />
          </el-form-item>
        </el-form>

        <div class="form-actions">
          <el-button
            type="primary"
            :loading="loading"
            :disabled="!selectedSource"
            @click="runDryRun"
          >
            {{ $t('cleanup.dry_run') }}
          </el-button>
        </div>
      </el-card>

      <div v-if="error" class="error-msg">
        <el-alert :title="error" type="error" show-icon :closable="false" />
      </div>
    </template>

    <!-- Step 2: Dry Run Preview -->
    <template v-else-if="step === 'preview' && dryRunResult">
      <div class="dry-run-summary">
        <el-alert type="info" show-icon :closable="false">
          <template #title>
            {{ $t('cleanup.total_files') }}: {{ dryRunResult.total_files }} |
            {{ $t('cleanup.total_release') }}: {{ totalFreed }} |
            {{ $t('cleanup.selected_count') }}: {{ selectedFiles.length }} ({{ selectedFreed }})
          </template>
        </el-alert>
      </div>

      <el-card shadow="never" class="table-card">
        <template #header>
          <div class="table-header">
            <el-checkbox :model-value="selectAll" @change="toggleSelectAll">
              {{ $t('common.select_all') }}
            </el-checkbox>
          </div>
        </template>

        <el-table
          :data="dryRunResult.files"
          style="width: 100%"
          size="small"
          @selection-change="
            (rows: CleanupFile[]) => {
              selectedFiles = rows
              selectAll = rows.length === dryRunResult!.files.length
            }
          "
        >
          <el-table-column type="selection" width="40" />
          <el-table-column
            prop="file_name"
            :label="$t('preview.file_name')"
            min-width="200"
            show-overflow-tooltip
          />
          <el-table-column :label="$t('preview.size')" width="100" sortable>
            <template #default="{ row }: { row: CleanupFile }">
              {{ formatBytes(row.file_size) }}
            </template>
          </el-table-column>
          <el-table-column :label="$t('preview.modified_at')" width="120" sortable>
            <template #default="{ row }: { row: CleanupFile }">
              {{ formatDate(row.modified_at) }}
            </template>
          </el-table-column>
          <el-table-column :label="$t('cleanup.backed_up_at')" width="180" sortable>
            <template #default="{ row }: { row: CleanupFile }">
              {{ row.backed_up_at ?? '--' }}
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <div class="form-actions">
        <el-button @click="step = 'filter'">{{ $t('common.back') }}</el-button>
        <el-button type="danger" :disabled="selectedFiles.length === 0" @click="confirmCleanup">
          {{ $t('cleanup.execute') }} ({{ selectedFiles.length }})
        </el-button>
      </div>
    </template>

    <!-- Step 3: Executing -->
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
            {{ $t('cleanup.result_success') }}: {{ progress?.succeeded ?? 0 }} |
            {{ $t('cleanup.result_failed') }}: {{ progress?.failed ?? 0 }}
          </p>
          <el-button type="danger" @click="cancel">
            {{ $t('backup.cancel') }}
          </el-button>
        </div>
      </el-card>
    </template>

    <!-- Step 4: Result -->
    <template v-else-if="step === 'result' && cleanupResult">
      <el-result
        :status="cleanupResult.failed > 0 ? 'warning' : 'success'"
        :title="$t('cleanup.completed')"
      >
        <template #sub-title>
          <div class="result-stats">
            <p>{{ $t('cleanup.result_success') }}: {{ cleanupResult.succeeded }}</p>
            <p v-if="cleanupResult.failed > 0">
              {{ $t('cleanup.result_failed') }}: {{ cleanupResult.failed }}
            </p>
            <p>{{ $t('cleanup.result_released') }}: {{ formatBytes(cleanupResult.freed_size) }}</p>
          </div>
        </template>
        <template #extra>
          <el-button @click="resetForm">{{ $t('common.back') }}</el-button>
        </template>
      </el-result>

      <el-card v-if="cleanupResult.errors.length > 0" shadow="never" class="error-card">
        <template #header>
          <span style="color: var(--el-color-danger)">{{ $t('common.error') }}</span>
        </template>
        <p v-for="(err, i) in cleanupResult.errors" :key="i" class="error-line">{{ err }}</p>
      </el-card>
    </template>
  </div>
</template>
