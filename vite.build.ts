import visualizer from 'rollup-plugin-visualizer'
import { BuildEnvironmentOptions } from 'vite'

export function createViteBuildConfig(env: Record<string, string>): BuildEnvironmentOptions {
  return {
    chunkSizeWarningLimit: 2000, // 消除打包大小超过 500kb 警告，默认为 500
    outDir: env.VITE_OUT_DIR || 'dist', // 输出目录，默认为 'dist'
    assetsDir: 'assets', // 静态资源目录，默认为 'assets'
    sourcemap: env.VITE_SOURCEMAP === 'true', // 是否生成 source map 文件，默认为 false
    minify: 'terser', // Vite 2.6.x 以上需要配置 minify: "terser", terserOptions 才能生效
    /** 在打包代码时移除 console.log、debugger 和 注释 */
    terserOptions: {
      compress: {
        drop_console: true, // 删除 console 语句，默认为 false
        drop_debugger: true, // 删除 debugger 语句，默认为 false
        pure_funcs: ['consola.log', 'consola.error', 'consola.info', 'consola.warn'] // 删除指定的函数调用，支持字符串或数组形式，默认为 []
      }
    },
    /** 配置 Rollup 选项，控制输出文件的命名和目录结构 */
    rollupOptions: {
      plugins: env.VITE_USE_BUNDLE_ANALYZER === 'true' ? [visualizer()] : undefined, // 如果启用包分析，则添加 visualizer 插件
      output: {
        manualChunks: (id) => {
          // 按依赖分组
          const chunks = {
            'vue-chunks': ['vue', 'vue-router', 'pinia', 'vue-i18n'],
            'element-plus': ['element-plus'],
            'wang-editor': ['@wangeditor/editor', '@wangeditor/editor-for-vue'],
            echarts: ['echarts', 'echarts-wordcloud'],
            lodash: ['lodash-es']
          }

          for (const [chunkName, modules] of Object.entries(chunks)) {
            if (modules.some((module) => id.includes(module))) {
              return chunkName
            }
          }
          if (id.includes('node_modules')) {
            return 'vendor'
          }
          return 'vendor'
        },
        chunkFileNames: 'js/[name]-[hash].js', // 代码分割后的文件名格式
        entryFileNames: 'js/[name]-[hash].js', // 入口文件名格式
        assetFileNames: ({ name }) => {
          if (/\.(css|s[ac]ss)$/.test(name ?? '')) {
            return 'css/[name]-[hash][extname]' // 如果是 CSS 文件，则放在 css 目录下
          }
          if (/\.(png|jpe?g|gif|svg|webp|ico)$/.test(name ?? '')) {
            return 'images/[name]-[hash][extname]' // 如果是图片文件，则放在 images 目录下
          }
          if (/\.(woff2?|eot|ttf|otf)$/.test(name ?? '')) {
            return 'fonts/[name]-[hash][extname]' // 如果是字体文件，则放在 fonts 目录下
          }
          return 'assets/[name]-[hash][extname]' // 其他静态资源放在 assets 目录下
        }
      }
    }
  }
}
