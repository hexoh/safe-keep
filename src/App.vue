<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { useUpdater } from '@/composables/useUpdater'

const { checking, updateAvailable, updateVersion, installUpdate } = useUpdater()

onMounted(() => {
  const dismissed = sessionStorage.getItem('update-dismissed')
  if (!dismissed) {
    useUpdater().checkForUpdates()
  }
})
</script>

<template>
  <div id="app-layout">
    <el-alert
      v-if="checking"
      type="info"
      show-icon
      :closable="false"
      :title="$t('common.loading')"
    />
    <el-alert v-if="updateAvailable" type="warning" show-icon :closable="false">
      <template #title>
        {{ $t('updater.available', { version: updateVersion }) }}
      </template>
      <template #default>
        <el-button size="small" type="primary" @click="installUpdate">
          {{ $t('updater.install') }}
        </el-button>
      </template>
    </el-alert>

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
