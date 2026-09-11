export default (ctx: {
  dev: boolean;
  mode: { name: string };
  modeName: string;
  appPaths: { resolve: { app: (p: string) => string } };
}) => {
  return {
    boot: ['auth', 'i18n', 'permission'],

    css: ['app.scss'],

    extras: [
      'roboto-font',
      'material-icons',
    ],

    build: {
      target: {
        browser: 'baseline-widely-available',
        node: 'node22',
      },

      typescript: {
        strict: true,
        vueShim: true,
      },

      vueRouterMode: 'hash',

      extendViteConf(viteConf: Record<string, unknown>) {
        if (!ctx.dev) {
          viteConf.build ??= {};
          const buildConf = viteConf.build as Record<string, unknown>;
          buildConf.chunkSizeWarningLimit ??= 3000;
          buildConf.cssMinify ??= 'esbuild';
          buildConf.minify ??= 'esbuild';
          buildConf.rollupOptions ??= {};
          const rollupOptions = buildConf.rollupOptions as Record<string, unknown>;
          rollupOptions.output ??= {};
          const output = rollupOptions.output as Record<string, unknown>;
          output.manualChunks ??= (id: string) => {
            if (id.includes('node_modules/quasar')) return 'quasar';
            if (id.includes('node_modules/vue')) return 'vue';
            if (id.includes('node_modules/pinia')) return 'pinia';
            if (id.includes('node_modules/echarts')) return 'echarts';
            if (id.includes('node_modules/exceljs')) return 'exceljs';
            if (id.includes('node_modules')) return 'vendor';
          };
        }

        // Add workspace package aliases
        viteConf.resolve ??= {};
        const resolveConf = viteConf.resolve as Record<string, unknown>;
        resolveConf.alias ??= {};
        const alias = resolveConf.alias as Record<string, string>;
        alias['@myai-workspace/api'] = ctx.appPaths.resolve.app('../../packages/api/src');
        alias['@myai-workspace/types'] = ctx.appPaths.resolve.app('../../packages/types/src');
        alias['@myai-workspace/utils'] = ctx.appPaths.resolve.app('../../packages/utils/src');
        alias['@myai-workspace/components'] = ctx.appPaths.resolve.app('../../packages/components/src');
        alias['@myai-workspace/composables'] = ctx.appPaths.resolve.app('../../packages/composables/src');
        alias['@myai-workspace/stores'] = ctx.appPaths.resolve.app('../../packages/stores/src');
        alias['@myai-workspace/boot'] = ctx.appPaths.resolve.app('../../packages/boot/src');
      },

      vitePlugins: [
        [
          '@intlify/unplugin-vue-i18n/vite',
          {
            ssr: ctx.modeName === 'ssr',
            include: [ctx.appPaths.resolve.app('src/i18n')],
          },
        ],
        [
          'vite-plugin-checker',
          {
            // 生产构建关闭 vue-tsc 类型检查
            vueTsc: false,
            eslint: {
              lintCommand: 'eslint -c ./eslint.config.js "./src*/**/*.{ts,js,mjs,cjs,vue}"',
              useFlatConfig: true,
            },
          },
          { server: false },
        ],
      ],
    },

    devServer: {
      open: true,
    },

    framework: {
      config: {},
      lang: 'zh-CN',
      plugins: ['Notify', 'Loading', 'Dialog', 'AppFullscreen'],
    },

    animations: [],

    ssr: {
      prodPort: 3000,
      middlewares: ['render'],
      pwa: false,
    },

    pwa: {
      workboxMode: 'GenerateSW',
    },

    cordova: {},

    capacitor: {
      hideSplashscreen: true,
    },

    electron: {
      extendElectronMainConf: {},
      extendElectronPreloadConf: {},
      inspectPort: 5858,
      bundler: 'packager',
      packager: {},
      builder: {
        appId: 'admin',
      },
    },
  };
};