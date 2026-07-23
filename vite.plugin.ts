import { PluginOption } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import AutoImport from 'unplugin-auto-import/vite'
import Icons from 'unplugin-icons/vite'
import IconsResolver from 'unplugin-icons/resolver'
import { FileSystemIconLoader } from 'unplugin-icons/loaders'
import svgLoader from 'vite-svg-loader'
import ServerUrlCopy from 'vite-plugin-url-copy'
import progress from 'vite-plugin-progress'
import { formatISO } from 'date-fns'
import { ViteEjsPlugin } from 'vite-plugin-ejs'
import { createStyleImportPlugin, ElementPlusResolve } from 'vite-plugin-style-import'
import EslintPlugin from 'vite-plugin-eslint2'
import UnoCSS from 'unocss/vite'

/** 获取 Element Plus 样式配置，根据环境变量决定是否按需加载 */
const getElementPlusStyleConfig = (env: Record<string, string>) => {
  if (env.VITE_USE_ALL_ELEMENT_PLUS_STYLE === 'false') {
    return createStyleImportPlugin({
      resolves: [ElementPlusResolve()],
      libs: [
        {
          libraryName: 'element-plus',
          esModule: true,
          resolveStyle: (name) => {
            if (name === 'click-outside') {
              return ''
            }
            return `element-plus/es/components/${name.replace(/^el-/, '')}/style/css`
          }
        }
      ]
    })
  }
}

export function createVitePluginConfig(env: Record<string, string>): PluginOption[] {
  return [
    vue(), // Vue 插件，支持 Vue 单文件组件（.vue 文件）
    vueJsx(), // Vue JSX 插件，支持在 Vue 组件中使用 JSX 语法
    vueDevTools(), // Vue DevTools 插件，方便在开发过程中调试 Vue 组件
    ServerUrlCopy(), // 开发服务器 URL 复制插件，方便在开发过程中快速复制本地或网络地址
    progress(), // 构建进度插件，显示构建过程中的进度条

    getElementPlusStyleConfig(env), // 根据环境变量决定是否按需加载 Element Plus 样式

    EslintPlugin({
      cache: false,
      exclude: ['node_modules/**'],
      include: ['src/**/*.vue', 'src/**/*.ts', 'src/**/*.tsx'] // 检查的文件
    }),

    AutoImport({
      imports: ['vue', 'vue-router', 'pinia'], // 按需添加
      dts: 'types/auto-imports.d.ts',
      resolvers: [ElementPlusResolver()]
    }),

    // 配置按需自动加载组件
    Components({
      dirs: ['src/components'],
      resolvers: [
        // 自动注册图标组件
        IconsResolver({
          prefix: 'icon', // 自定义图标组件前缀，默认是 'i'，这里改为 'icon' 以防冲突
          // 可以选择开启 collection 限制，比如只允许 mdi 和 ep (Element Plus)
          enabledCollections: ['local']
        }),
        ElementPlusResolver() // 自动注册 Element Plus 组件
      ],
      dts: 'types/components.d.ts' // 自动生成 ts 声明文件，解决 TS 报错
    }),

    // 配置 Iconify 图标
    Icons({
      compiler: 'vue3', // 指定编译器为 vue3
      // 自定义图标集合，使用本地 SVG 文件,
      customCollections: {
        // 从 src/assets/icons 目录加载 SVG 图标，并且在加载时对 SVG 内容进行转换，添加 fill="currentColor" 属性，使图标颜色可以通过 CSS 控制
        local: FileSystemIconLoader('src/assets/icons', (svg) =>
          !svg.includes('fill=') ? svg.replace(/<svg\s*/, '<svg fill="currentColor" ') : svg
        )
      },
      autoInstall: true // 自动安装检测到的图标集（如果有未安装的，会自动用包管理器安装）
    }),

    /** 将 SVG 静态图转化为 Vue 组件 */
    svgLoader({ defaultImport: 'url' }),
    /** 修改标题 */
    ViteEjsPlugin({
      title: env.VITE_APP_TITLE
    }),
    /** 加入打包时间戳 */
    {
      name: 'html-transform',
      transformIndexHtml(html) {
        return html.replace(/<\/body>/, `<!-- build stamp ${formatISO(new Date())} -->\n</body>`)
      }
    },
    UnoCSS()
  ]
}
