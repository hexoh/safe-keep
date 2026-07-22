<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const sourcePath = ref('')
const destPath = ref('')
const filter = reactive({
  includeImages: true,
  includeVideos: true,
  minSize: 0,
  maxSize: 0
})

function startBackup() {
  if (!sourcePath.value) return
  router.push({
    name: 'preview',
    query: {
      source: sourcePath.value,
      dest: destPath.value,
      images: filter.includeImages ? '1' : '0',
      videos: filter.includeVideos ? '1' : '0',
      minSize: String(filter.minSize),
      maxSize: String(filter.maxSize)
    }
  })
}
</script>

<template>
  <div class="home-view">
    <h1>{{ $t('home.title') }}</h1>
    <p class="subtitle">{{ $t('home.subtitle') }}</p>

    <el-card shadow="never" class="config-card">
      <div class="config-section">
        <PathSelector
          v-model="sourcePath"
          :label="$t('home.source_path')"
          :placeholder="$t('home.source_path')"
        />
      </div>

      <div class="config-section">
        <PathSelector
          v-model="destPath"
          :label="$t('home.dest_path')"
          :placeholder="$t('home.dest_path')"
        />
      </div>

      <div class="config-section">
        <FileFilter v-model:filter="filter" />
      </div>

      <div class="action-row">
        <el-button type="primary" size="large" :disabled="!sourcePath" @click="startBackup">
          {{ $t('home.start_backup') }}
        </el-button>
      </div>
    </el-card>
  </div>
</template>
