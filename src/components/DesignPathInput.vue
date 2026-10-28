<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{
  label: string
  value: string
  placeholder: string
  hint?: string
  /** 设备类型等附加标签（feature 3.1.6），由调用方翻译后传入。 */
  badge?: string
  badgeIcon?: 'phone' | 'sd' | 'hdd'
}>()

const emit = defineEmits<{
  'update:value': [value: string]
}>()

async function browse() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: props.value
    })
    if (selected) {
      emit('update:value', selected)
    }
  } catch {
    // dialog unavailable (e.g. permission not granted)
  }
}
</script>

<template>
  <div class="design-path-input">
    <div class="label-row">
      <DesignLabel :text="label" />
      <span v-if="badge" class="device-badge">
        <icon-local-phone v-if="badgeIcon === 'phone'" />
        <icon-local-sd v-else-if="badgeIcon === 'sd'" />
        <icon-local-hdd v-else />
        {{ badge }}
      </span>
    </div>
    <div class="input-row">
      <input
        :value="value"
        @input="emit('update:value', ($event.target as HTMLInputElement).value)"
        :placeholder="placeholder"
        class="path-field"
      />
      <button class="browse-btn" @click="browse">
        <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
          <path d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
        </svg>
        {{ $t('common.browse') }}
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

.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.device-badge {
  display: inline-flex;
  padding: 1px 7px;
  font-size: 10px;
  font-weight: 500;
  color: var(--accent);
  background: var(--accent-dim);
  border-radius: 5px;
  align-items: center;
  gap: 4px;
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
