<script setup lang="ts">
defineProps<{
  on: boolean
}>()

const emit = defineEmits<{
  change: [value: boolean]
}>()
</script>

<template>
  <button :class="['design-switcher', { active: on }]" @click="emit('change', !on)">
    <div class="switcher-knob" />
  </button>
</template>

<style scoped>
.design-switcher {
  position: relative;
  width: 38px;
  height: 21px;
  cursor: pointer;
  background: var(--border);
  border: none;
  border-radius: 11px;
  transition: background 0.2s ease;
  flex-shrink: 0;
}

.design-switcher.active {
  background: var(--accent);
}

.switcher-knob {
  position: absolute;
  top: 2.5px;
  left: 2.5px;
  width: 16px;
  height: 16px;
  background: #fff;
  border-radius: 50%;
  box-shadow: 0 1px 4px rgb(0 0 0 / 40%);
  transition: left 0.2s ease;
}

.design-switcher.active .switcher-knob {
  left: 19px;
}
</style>
