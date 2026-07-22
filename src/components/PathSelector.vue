<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { detectDevice } from '@/api/device'
import type { DeviceType } from '@/api/device'

const props = withDefaults(
  defineProps<{
    modelValue: string
    label?: string
    placeholder?: string
  }>(),
  {
    placeholder: ''
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const deviceType = ref<DeviceType | null>(null)

const deviceIcon = computed(() => {
  switch (deviceType.value) {
    case 'MTP':
      return '📱'
    case 'RemovableDisk':
      return '💾'
    case 'LocalDisk':
      return '💻'
    default:
      return ''
  }
})

const deviceLabelKey = computed(() => {
  switch (deviceType.value) {
    case 'MTP':
      return 'device.mtp'
    case 'RemovableDisk':
      return 'device.removable'
    case 'LocalDisk':
      return 'device.local'
    case 'Unknown':
      return 'device.unknown'
    default:
      return ''
  }
})

watch(
  () => props.modelValue,
  async (path) => {
    if (path && path.length > 2) {
      try {
        deviceType.value = await detectDevice(path)
      } catch {
        deviceType.value = null
      }
    } else {
      deviceType.value = null
    }
  },
  { immediate: true }
)

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: props.label
  })
  if (selected) {
    emit('update:modelValue', selected as string)
  }
}
</script>

<template>
  <div class="path-selector">
    <label v-if="label" class="path-label">{{ label }}</label>
    <div class="path-input-group">
      <el-input
        :model-value="modelValue"
        :placeholder="placeholder || $t('home.select_folder')"
        clearable
        @update:model-value="(v: string) => $emit('update:modelValue', v)"
      >
        <template #prepend v-if="deviceType">
          <span class="device-badge" :title="$t(deviceLabelKey)">
            {{ deviceIcon }} {{ $t(deviceLabelKey) }}
          </span>
        </template>
        <template #append>
          <el-button @click="selectFolder">
            {{ $t('home.select_folder') }}
          </el-button>
        </template>
      </el-input>
    </div>
  </div>
</template>

<style scoped>
.path-input-group {
  display: flex;
  gap: 4px;
}
</style>
