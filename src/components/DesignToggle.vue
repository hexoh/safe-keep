<script setup lang="ts">
defineProps<{
  label: string
  on: boolean
}>()

const emit = defineEmits<{
  change: [value: boolean]
}>()
</script>

<template>
  <button :class="['design-toggle', { active: on }]" @click="emit('change', !on)">
    {{ label }}
  </button>
</template>

<style scoped>
.design-toggle {
  padding: 4px 11px;
  font-family: var(--mono);
  font-size: 11px;
  font-weight: 500;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 5px;
}

.design-toggle.active {
  color: var(--accent);
  background: var(--accent-dim);
  border-color: rgb(79 127 255 / 40%);
}
</style>
