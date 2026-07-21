import { fileURLToPath, URL } from 'node:url'
import { type ConfigEnv, type UserConfigExport, loadEnv } from 'vite'
import { createViteBuildConfig } from './vite.build'
import { createVitePluginConfig } from './vite.plugin'

// https://vite.dev/config/
export default (configEnv: ConfigEnv): UserConfigExport => {
  const env = loadEnv(configEnv.mode, process.cwd())

  return {
    base: env.VITE_BASE_PATH || '/',

    // 配置开发服务器选项
    server: {
      port: 1420, // Tauri dev server 默认端口
      strictPort: true, // 端口被占用直接报错，避免端口切换导致 Tauri 无法连接
      host: '0.0.0.0',
      open: false
    },

    // https://v2.tauri.app/start/frontend/vite/#managing-the-vite-dev-server
    clearScreen: false,

    // 配置构建选项
    build: createViteBuildConfig(env),
    // 配置插件
    plugins: createVitePluginConfig(env),

    css: {
      preprocessorOptions: {
        scss: {
          additionalData: '@import "./src/styles/variables.module.scss";'
        }
      }
    },
    // 配置路径别名
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      }
    }
  }
}
