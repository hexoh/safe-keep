export default {
  root: true,
  plugins: ['stylelint-order'],
  // customSyntax: 'postcss-html',
  extends: [
    'stylelint-config-standard', // 基础的 Stylelint 规则集，适用于大多数项目
    'stylelint-config-standard-scss' // 针对 SCSS 的 Stylelint 规则集，提供了对 SCSS 语法的支持
  ],
  rules: {
    'selector-pseudo-class-no-unknown': [
      true,
      {
        ignorePseudoClasses: ['global', 'deep', 'export']
      }
    ],
    'property-no-unknown': [
      true,
      {
        ignoreProperties: [
          // 添加 CSS Modules 导出属性
          'composes',
          'namespace',
          'elNamespace',
          '/^[a-z][a-zA-Z0-9]*$/' // 正则匹配驼峰式属性名
        ]
      }
    ],
    'at-rule-no-unknown': [
      true,
      {
        ignoreAtRules: ['function', 'if', 'each', 'include', 'mixin']
      }
    ],
    'media-query-no-invalid': null,
    'function-no-unknown': null,
    'no-empty-source': null,
    'named-grid-areas-no-invalid': null,
    'no-descending-specificity': null,
    'font-family-no-missing-generic-family-keyword': null,
    'rule-empty-line-before': [
      'always',
      {
        ignore: ['after-comment', 'first-nested']
      }
    ],
    'unit-no-unknown': [
      true,
      {
        ignoreUnits: ['rpx']
      }
    ],
    'order/order': [
      [
        'dollar-variables',
        'custom-properties',
        'at-rules',
        'declarations',
        {
          type: 'at-rule',
          name: 'supports'
        },
        {
          type: 'at-rule',
          name: 'media'
        },
        'rules'
      ],
      {
        severity: 'warning'
      }
    ],
    // Specify the alphabetical order of the attributes in the declaration block
    'order/properties-order': [
      'position',
      'top',
      'right',
      'bottom',
      'left',
      'z-index',
      'display',
      'float',
      'width',
      'height',
      'max-width',
      'max-height',
      'min-width',
      'min-height',
      'padding',
      'padding-top',
      'padding-right',
      'padding-bottom',
      'padding-left',
      'margin',
      'margin-top',
      'margin-right',
      'margin-bottom',
      'margin-left',
      'margin-collapse',
      'margin-top-collapse',
      'margin-right-collapse',
      'margin-bottom-collapse',
      'margin-left-collapse',
      'overflow',
      'overflow-x',
      'overflow-y',
      'clip',
      'clear',
      'font',
      'font-family',
      'font-size',
      'font-smoothing',
      'osx-font-smoothing',
      'font-style',
      'font-weight',
      'hyphens',
      'src',
      'line-height',
      'letter-spacing',
      'word-spacing',
      'color',
      'text-align',
      'text-decoration',
      'text-indent',
      'text-overflow',
      'text-rendering',
      'text-size-adjust',
      'text-shadow',
      'text-transform',
      'word-break',
      'word-wrap',
      'white-space',
      'vertical-align',
      'list-style',
      'list-style-type',
      'list-style-position',
      'list-style-image',
      'pointer-events',
      'cursor',
      'background',
      'background-attachment',
      'background-color',
      'background-image',
      'background-position',
      'background-repeat',
      'background-size',
      'border',
      'border-collapse',
      'border-top',
      'border-right',
      'border-bottom',
      'border-left',
      'border-color',
      'border-image',
      'border-top-color',
      'border-right-color',
      'border-bottom-color',
      'border-left-color',
      'border-spacing',
      'border-style',
      'border-top-style',
      'border-right-style',
      'border-bottom-style',
      'border-left-style',
      'border-width',
      'border-top-width',
      'border-right-width',
      'border-bottom-width',
      'border-left-width',
      'border-radius',
      'border-top-right-radius',
      'border-bottom-right-radius',
      'border-bottom-left-radius',
      'border-top-left-radius',
      'border-radius-topright',
      'border-radius-bottomright',
      'border-radius-bottomleft',
      'border-radius-topleft',
      'content',
      'quotes',
      'outline',
      'outline-offset',
      'opacity',
      'filter',
      'visibility',
      'size',
      'zoom',
      'transform',
      'box-align',
      'box-flex',
      'box-orient',
      'box-pack',
      'box-shadow',
      'box-sizing',
      'table-layout',
      'animation',
      'animation-delay',
      'animation-duration',
      'animation-iteration-count',
      'animation-name',
      'animation-play-state',
      'animation-timing-function',
      'animation-fill-mode',
      'transition',
      'transition-delay',
      'transition-duration',
      'transition-property',
      'transition-timing-function',
      'background-clip',
      'backface-visibility',
      'resize',
      'appearance',
      'user-select',
      'interpolation-mode',
      'direction',
      'marks',
      'page',
      'set-link-source',
      'unicode-bidi',
      'speak'
    ]
  },
  ignoreFiles: ['**/*.js', '**/*.jsx', '**/*.tsx', '**/*.ts'],
  overrides: [
    {
      files: ['*.vue', '**/*.vue', '*.html', '**/*.html'],
      customSyntax: 'postcss-html',
      extends: [
        'stylelint-config-recommended', // 基础的 Stylelint 规则集，适用于大多数项目
        'stylelint-config-html', // 针对 HTML 文件的 Stylelint 规则集，提供了对 HTML 中内联样式的支持
        'stylelint-config-standard-scss' // 针对 SCSS 的 Stylelint 规则集，提供了对 SCSS 语法的支持
      ],
      rules: {
        'keyframes-name-pattern': null,
        'selector-class-pattern': null,
        'no-duplicate-selectors': null,
        'selector-pseudo-class-no-unknown': [
          true,
          {
            ignorePseudoClasses: ['deep', 'global', 'export']
          }
        ],
        'selector-pseudo-element-no-unknown': [
          true,
          {
            ignorePseudoElements: ['v-deep', 'v-global', 'v-slotted']
          }
        ],
        // SCSS 特有规则
        'scss/at-rule-no-unknown': [
          true,
          {
            ignoreAtRules: ['/^my-/']
          }
        ]
      }
    },
    // 新增 SCSS 文件的特定配置
    {
      files: ['**/*.scss', '**/*.sass'],
      // 为 SCSS/Sass 文件使用 postcss-scss
      customSyntax: 'postcss-scss',
      extends: ['stylelint-config-standard-scss'],
      rules: {
        'selector-pseudo-class-no-unknown': [
          true,
          {
            ignorePseudoClasses: ['export', 'global', 'deep', 'local', 'root'] // 确保包含 export
          }
        ],
        'property-no-unknown': [
          true,
          {
            ignoreProperties: [
              'composes',
              'namespace',
              'elNamespace',
              // 添加其他可能的导出属性
              /^[a-z][a-zA-Z0-9]*$/ // 匹配驼峰式属性
            ]
          }
        ],
        // SCSS 特有规则覆盖
        'scss/at-rule-no-unknown': null, // 禁用基础规则，因为已经扩展了标准配置
        'scss/dollar-variable-pattern': null, // 如果需要，可以自定义变量命名规则
        'scss/operator-no-unspaced': true, // 确保运算符两边有空格
        'scss/dollar-variable-empty-line-before': null, // 不强制变量前的空行
        'scss/double-slash-comment-whitespace-inside': 'always', // 强制 // 注释后的空格
        'scss/at-extend-no-missing-placeholder': true, // 确保 @extend 使用占位符
        'scss/declaration-nested-properties': 'never' // 不允许嵌套属性
      }
    }
  ]
}
