<template>
  <div class="layout-storage">
    <div class="storage-header">
      <span class="storage-label">{{ $t('storage.backup_drive') }}</span>
      <span class="storage-free">{{ $t('storage.free_space', { size: '412 GB' }) }}</span>
    </div>
    <div class="storage-bar">
      <div class="storage-bar-fill" style="width: 41%" />
    </div>
    <div class="storage-footer">{{ $t('storage.used_space', { used: '339', total: '580' }) }}</div>
  </div>
</template>

<style scoped>
.layout-storage {
  padding: 9px 10px;
  margin: 8px 2px 0;
  background: var(--card);
  border-radius: 7px;
}

.storage-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
}

.storage-label {
  font-size: 10px;
  color: var(--text-muted);
}

.storage-free {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--text-sec);
}

.storage-bar {
  height: 3px;
  overflow: hidden;
  background: var(--border);
  border-radius: 3px;
}

.storage-bar-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.35s ease;
}

.storage-footer {
  margin-top: 4px;
  font-family: var(--mono);
  font-size: 10px;
  color: var(--text-muted);
}
</style>
