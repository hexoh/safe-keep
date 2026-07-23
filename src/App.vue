<script setup lang="ts">
import { useUpdater } from '@/composables/useUpdater'
import { useTheme } from '@/composables/useTheme'
import { useLocale } from '@/composables/useLocale'
import LayoutDefault from '@/layout/layout-default.vue'

const { checking, updateAvailable, updateVersion, installUpdate } = useUpdater()
useTheme()
useLocale()
</script>

<template>
  <el-alert v-if="checking" type="info" show-icon :closable="false" :title="$t('common.loading')" />
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

  <LayoutDefault />
</template>
