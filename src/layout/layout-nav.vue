<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const items = [
  { path: '/', label: 'nav.home', icon: 'home' },
  { path: '/preview', label: 'nav.preview', icon: 'search' },
  { path: '/backup', label: 'nav.backup', icon: 'upload' },
  { path: '/cleanup', label: 'nav.cleanup', icon: 'trash' },
  { path: '/history', label: 'nav.history', icon: 'clock' },
  { path: '/restore', label: 'nav.restore', icon: 'restore' }
]
</script>

<template>
  <nav class="layout-nav">
    <button
      v-for="item in items"
      :key="item.path"
      :class="['nav-item', { active: route.path === item.path }]"
      @click="router.push(item.path)"
    >
      <span class="nav-icon">
        <icon-local-home v-if="item.icon === 'home'" />
        <icon-local-search v-else-if="item.icon === 'search'" />
        <icon-local-upload v-else-if="item.icon === 'upload'" />
        <icon-local-trash v-else-if="item.icon === 'trash'" />
        <icon-local-clock v-else-if="item.icon === 'clock'" />
        <icon-local-restore v-else-if="item.icon === 'restore'" />
      </span>
      {{ $t(item.label) }}
    </button>
  </nav>
</template>

<style scoped>
.layout-nav {
  display: flex;
  padding: 9px 7px;
  flex: 1;
  flex-direction: column;
  gap: 1px;
}

.nav-item {
  display: flex;
  width: 100%;
  padding: 7px 9px;
  font-size: 14px;
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
