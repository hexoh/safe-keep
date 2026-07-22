<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

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
