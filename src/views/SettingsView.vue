<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useTheme } from '@/composables/useTheme'
import type { ThemeMode } from '@/composables/useTheme'

const { locale } = useI18n()
const { theme } = useTheme()

const concurrentThreads = ref(4)
const conflictStrategy = ref<'rename' | 'skip' | 'overwrite'>('rename')
const deleteStrategy = ref<'recycle' | 'permanent'>('recycle')
const dryRunDefault = ref(false)
const autoCleanupReminder = ref(true)
const autoUpdate = ref(true)
const updateChannel = ref<'stable' | 'beta'>('stable')

function setTheme(mode: ThemeMode) {
  theme.value = mode
}

function setLanguage(lang: string) {
  locale.value = lang
}
</script>

<template>
  <div class="settings-view">
    <h1>{{ $t('settings.title') }}</h1>

    <el-card shadow="never" class="settings-card">
      <el-form label-width="180px">
        <el-form-item :label="$t('settings.theme_mode')">
          <el-radio-group :model-value="theme" @change="setTheme">
            <el-radio-button value="light">{{ $t('settings.theme_light') }}</el-radio-button>
            <el-radio-button value="dark">{{ $t('settings.theme_dark') }}</el-radio-button>
            <el-radio-button value="system">{{ $t('settings.theme_system') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>

        <el-form-item :label="$t('settings.language')">
          <el-radio-group :model-value="locale" @change="setLanguage">
            <el-radio-button value="zh-CN">中文</el-radio-button>
            <el-radio-button value="en">English</el-radio-button>
          </el-radio-group>
        </el-form-item>

        <el-divider />

        <el-form-item :label="$t('settings.concurrent_threads')">
          <el-input-number v-model="concurrentThreads" :min="1" :max="16" />
        </el-form-item>

        <el-form-item :label="$t('settings.conflict_strategy')">
          <el-select v-model="conflictStrategy" style="width: 200px">
            <el-option value="rename" label="Auto Rename" />
            <el-option value="skip" label="Skip" />
            <el-option value="overwrite" label="Overwrite" />
          </el-select>
        </el-form-item>

        <el-form-item :label="$t('settings.delete_strategy')">
          <el-select v-model="deleteStrategy" style="width: 200px">
            <el-option value="recycle" :label="$t('cleanup.move_to_trash')" />
            <el-option value="permanent" :label="$t('cleanup.permanent_delete')" />
          </el-select>
        </el-form-item>

        <el-divider />

        <el-form-item :label="$t('settings.dry_run_default')">
          <el-switch v-model="dryRunDefault" />
        </el-form-item>

        <el-form-item :label="$t('settings.auto_cleanup_reminder')">
          <el-switch v-model="autoCleanupReminder" />
        </el-form-item>

        <el-form-item :label="$t('settings.auto_update')">
          <el-switch v-model="autoUpdate" />
        </el-form-item>

        <el-form-item :label="$t('settings.update_channel')">
          <el-select v-model="updateChannel" style="width: 200px">
            <el-option value="stable" label="Stable" />
            <el-option value="beta" label="Beta" />
          </el-select>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style scoped>
.settings-card {
  max-width: 700px;
}
</style>
