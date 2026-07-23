<script setup lang="ts" generic="T extends string">
defineProps<{
  options: { value: T; label: string }[]
  modelValue: T
}>()

const emit = defineEmits<{
  'update:modelValue': [value: T]
}>()
</script>

<template>
  <div class="design-radio-group">
    <button
      v-for="opt in options"
      :key="opt.value"
      :class="['radio-option', { active: modelValue === opt.value }]"
      @click="emit('update:modelValue', opt.value)"
    >
      {{ opt.label }}
    </button>
  </div>
</template>

<style scoped>
.design-radio-group {
  display: flex;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 6px;
}

.radio-option {
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: none;
}

.radio-option.active {
  color: #fff;
  background: var(--accent);
}
</style>
