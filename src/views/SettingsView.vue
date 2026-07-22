<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useTheme } from '@/composables/useTheme'
import { useSettings } from '@/composables/useSettings'
import type { ThemeMode } from '@/composables/useTheme'

const { locale } = useI18n()
const { theme } = useTheme()
const { settings, load, save } = useSettings()

onMounted(load)

function setTheme(mode: ThemeMode) {
  theme.value = mode
  settings.value.theme = mode
  save()
}

function setLanguage(lang: string) {
  locale.value = lang
  settings.value.language = lang
  save()
}

function onFieldChange() {
  save()
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

        <el-form-item :label="$t('settings.default_dest')">
          <el-input v-model="settings.default_dest" placeholder="/path/to/backup" @change="save" />
        </el-form-item>

        <el-form-item :label="$t('settings.concurrent_threads')">
          <el-input-number
            v-model="settings.concurrent_threads"
            :min="1"
            :max="16"
            @change="onFieldChange"
          />
        </el-form-item>

        <el-form-item :label="$t('settings.compare_strategy')">
          <el-select
            v-model="settings.compare_strategy"
            style="width: 200px"
            @change="onFieldChange"
          >
            <el-option value="size_time" label="Size + Time" />
            <el-option value="fast_hash" label="Fast Hash" />
            <el-option value="sha256" label="SHA256" />
          </el-select>
        </el-form-item>

        <el-form-item :label="$t('settings.conflict_strategy')">
          <el-select
            v-model="settings.conflict_strategy"
            style="width: 200px"
            @change="onFieldChange"
          >
            <el-option value="rename" label="Auto Rename" />
            <el-option value="skip" label="Skip" />
            <el-option value="overwrite" label="Overwrite" />
          </el-select>
        </el-form-item>

        <el-form-item :label="$t('settings.delete_strategy')">
          <el-select
            v-model="settings.delete_strategy"
            style="width: 200px"
            @change="onFieldChange"
          >
            <el-option value="recycle" :label="$t('cleanup.move_to_trash')" />
            <el-option value="permanent" :label="$t('cleanup.permanent_delete')" />
          </el-select>
        </el-form-item>

        <el-divider />

        <el-form-item :label="$t('settings.dry_run_default')">
          <el-switch v-model="settings.dry_run_default" @change="onFieldChange" />
        </el-form-item>

        <el-form-item :label="$t('settings.auto_cleanup_reminder')">
          <el-switch v-model="settings.auto_cleanup_reminder" @change="onFieldChange" />
        </el-form-item>

        <el-form-item :label="$t('settings.auto_update')">
          <el-switch v-model="settings.auto_update" @change="onFieldChange" />
        </el-form-item>

        <el-form-item :label="$t('settings.update_channel')">
          <el-select v-model="settings.update_channel" style="width: 200px" @change="onFieldChange">
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
