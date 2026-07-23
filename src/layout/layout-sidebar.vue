<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import LayoutLogo from './layout-logo.vue'
import LayoutNav from './layout-nav.vue'
import LayoutStorage from './layout-storage.vue'

const router = useRouter()
const route = useRoute()
</script>

<template>
  <aside class="sidebar">
    <LayoutLogo />
    <LayoutNav />
    <div class="sidebar-bottom">
      <button
        :class="['nav-item', { active: route.path === '/settings' }]"
        @click="router.push('/settings')"
      >
        <span class="nav-icon"><icon-local-settings /></span>
        {{ $t('nav.settings') }}
      </button>
      <LayoutStorage />
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  width: 208px;
  height: 100%;
  background: var(--surface);
  border-right: 1px solid var(--border);
  flex-shrink: 0;
  flex-direction: column;
}

.sidebar-bottom {
  padding: 7px 7px 12px;
  border-top: 1px solid var(--border-subtle);
}

.nav-item {
  display: flex;
  width: 100%;
  padding: 7px 9px;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-sec);
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  align-items: center;
  gap: 8px;
}

.nav-item.active {
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-dim);
  border-color: rgb(79 127 255 / 25%);
}

.nav-item:hover {
  background: var(--accent-dim);
}

.nav-icon {
  display: flex;
  flex-shrink: 0;
}
</style>
