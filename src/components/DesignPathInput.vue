<script setup lang="ts">
defineProps<{
  label: string
  value: string
  placeholder: string
  hint?: string
}>()

const emit = defineEmits<{
  'update:value': [value: string]
}>()
</script>

<template>
  <div class="design-path-input">
    <DesignLabel :text="label" />
    <div class="input-row">
      <input
        :value="value"
        @input="emit('update:value', ($event.target as HTMLInputElement).value)"
        :placeholder="placeholder"
        class="path-field"
      />
      <button class="browse-btn">
        <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
          <path d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
        </svg>
        Browse
      </button>
    </div>
    <span v-if="hint" class="path-hint">{{ hint }}</span>
  </div>
</template>

<style scoped>
.design-path-input {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.input-row {
  display: flex;
  overflow: hidden;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 7px;
}

.path-field {
  padding: 8px 11px;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text);
  background: transparent;
  border: none;
  flex: 1;
}

.path-field::placeholder {
  color: var(--text-muted);
}

.browse-btn {
  display: flex;
  padding: 0 12px;
  font-size: 12px;
  color: var(--text-sec);
  cursor: pointer;
  background: var(--card);
  border: none;
  border-left: 1px solid var(--border);
  align-items: center;
  gap: 5px;
}

.path-hint {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
}
</style>
