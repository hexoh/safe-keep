<template>
  <div class="layout-logo">
    <div class="logo-icon">归</div>
    <div class="logo-text">
      <span class="logo-title">Safe Keep</span>
      <span class="logo-version">v2.1.0</span>
    </div>
  </div>
</template>

<style scoped>
.layout-logo {
  display: flex;
  padding: 16px 16px 14px;
  border-bottom: 1px solid var(--border-subtle);
  align-items: center;
  gap: 9px;
}

.logo-icon {
  display: flex;
  width: 26px;
  height: 26px;
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  background: linear-gradient(135deg, #4f7fff 0%, #7b5bff 100%);
  border-radius: 7px;
  box-shadow: 0 2px 8px rgb(79 127 255 / 35%);
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.logo-text {
  display: flex;
  flex-direction: column;
}

.logo-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--text);
}

.logo-version {
  font-family: var(--mono);
  font-size: 9px;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}
</style>
