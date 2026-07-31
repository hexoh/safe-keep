<script setup lang="ts">
import LayoutSidebar from './layout-sidebar.vue'
</script>

<template>
  <div class="app-layout">
    <LayoutSidebar />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--bg);
}

.main-content {
  display: flex;
  min-width: 0;
  overflow: hidden;
  flex: 1;
  background: var(--surface);
  flex-direction: column;
}
</style>
