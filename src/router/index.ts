import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/preview',
      name: 'preview',
      component: () => import('../views/PreviewView.vue')
    },
    {
      path: '/backup',
      name: 'backup',
      component: () => import('../views/BackupView.vue')
    },
    {
      path: '/cleanup',
      name: 'cleanup',
      component: () => import('../views/CleanupView.vue')
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('../views/HistoryView.vue')
    },
    {
      path: '/restore',
      name: 'restore',
      component: () => import('../views/RestoreView.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue')
    }
  ]
})

export default router
