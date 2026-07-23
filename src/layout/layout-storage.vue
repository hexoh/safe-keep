<template>
  <div class="layout-storage">
    <div class="storage-header">
      <span class="storage-label">Backup Drive</span>
      <span class="storage-free">412 GB free</span>
    </div>
    <div class="storage-bar">
      <div class="storage-bar-fill" style="width: 41%" />
    </div>
    <div class="storage-footer">339 / 580 GB used</div>
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
