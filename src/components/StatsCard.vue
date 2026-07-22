<script setup lang="ts">
defineProps<{
  title: string
  value: string | number
  type?: 'primary' | 'success' | 'warning' | 'info'
}>()
</script>

<template>
  <el-card shadow="never" class="stats-card">
    <div class="stats-content">
      <div class="stats-value" :class="type ? `text-${type}` : ''">{{ value }}</div>
      <div class="stats-title">{{ title }}</div>
    </div>
  </el-card>
</template>
