<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div id="app-layout">
    <header>
      <nav>
        <RouterLink to="/">{{ $t('nav.home') }}</RouterLink>
        <RouterLink to="/preview">{{ $t('nav.preview') }}</RouterLink>
        <RouterLink to="/backup">{{ $t('nav.backup') }}</RouterLink>
        <RouterLink to="/cleanup">{{ $t('nav.cleanup') }}</RouterLink>
        <RouterLink to="/history">{{ $t('nav.history') }}</RouterLink>
        <RouterLink to="/restore">{{ $t('nav.restore') }}</RouterLink>
        <RouterLink to="/settings">{{ $t('nav.settings') }}</RouterLink>
      </nav>
    </header>

    <main>
      <RouterView />
    </main>
  </div>
</template>
