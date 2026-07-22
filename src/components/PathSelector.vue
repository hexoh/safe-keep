<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string
    label?: string
    placeholder?: string
  }>(),
  {
    placeholder: ''
  }
)

// const emit = defineEmits<{
//   'update:modelValue': [value: string]
// }>()
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
      />
    </div>
  </div>
</template>
