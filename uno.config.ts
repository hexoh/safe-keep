import {
  defineConfig,
  presetWind4,
  presetIcons,
  transformerVariantGroup,
  presetAttributify
} from 'unocss'

export default defineConfig({
  presets: [
    // presetWind4 预设
    presetWind4(),
    // 可选的图标预设
    presetIcons({
      autoInstall: true,
      prefix: 'i-',
      extraProperties: {
        display: 'inline-block',
        'vertical-align': 'middle'
      }
    }),
    // 启用属性化模式
    presetAttributify({
      prefix: 'un-', // 前缀
      prefixedOnly: true // 仅匹配带前缀的属性，即强制前缀
    })
  ],
  transformers: [transformerVariantGroup()],
  content: {
    pipeline: {
      include: [/\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html|ts)($|\?)/]
    }
  },
  shortcuts: {
    center: 'flex justify-center items-center'
  },
  // 可选：自定义规则
  rules: []
})
