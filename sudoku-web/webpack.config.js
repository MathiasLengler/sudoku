/* eslint-disable */
const path = require("path");
const webpack = require('webpack');
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WebpackPwaManifest = require('webpack-pwa-manifest')
const WorkboxPlugin = require('workbox-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const FaviconsWebpackPlugin = require('favicons-webpack-plugin')

const dist = path.resolve(__dirname, "dist");
// const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;

module.exports = (env, argv) => {
  const {mode} = argv;

  let devtool;
  let extraPlugins;
  if (mode === 'development') {
    devtool = 'eval-source-map';
    extraPlugins = [new webpack.HotModuleReplacementPlugin()]
  } else if (mode === 'production') {
    devtool = 'source-map';
    extraPlugins = [];
  } else {
    throw new Error(`Unexpected mode: ${mode}`);
  }

  const reactProfiling = !!(env && env.reactProfiling);

  const alias = reactProfiling ? {
      'react-dom$': 'react-dom/profiling',
      'scheduler/tracing': 'scheduler/tracing-profiling',
    }
    : {};

  const optimization = reactProfiling ? {
    minimize: false
  } : {};

  return {
    name: "app",
    entry: "./src/index.tsx",
    output: {
      path: dist,
      filename: "app.js",
      publicPath: "",
      clean: true,
    },
    devServer: {
      contentBase: dist,
      host: '127.0.0.1',
      hot: true
    },
    devtool,
    resolve: {
      extensions: [".ts", ".tsx", ".js", ".wasm"],
      alias
    },
    experiments: {
      syncWebAssembly: true
    },
    plugins: [
      new HtmlWebpackPlugin({
        template: path.resolve(__dirname, "res", "index.html"),
        favicon: ""
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "../sudoku-wasm"),
        watchDirectories: [
          path.resolve(__dirname, "../sudoku-rs")
        ],
        outDir: path.resolve(__dirname, "../sudoku-wasm/pkg")
      }),
      // PWA
      new WorkboxPlugin.GenerateSW({
        // these options encourage the ServiceWorkers to get in there fast
        // and not allow any straggling "old" SWs to hang around
        clientsClaim: true,
        skipWaiting: true,
      }),
      new WebpackPwaManifest({
        name: 'Sudoku',
        short_name: 'Sudoku',
        description: 'Touch optimized sudoku built with Rust/WASM/TypeScript/React',
        background_color: '#fafafa',
        icons: [
          {
            src: path.resolve('res/img/sudoku_icon_full_size.png'),
            sizes: [96, 128, 192, 256, 384, 512] // multiple sizes
          },
        ]
      }),
      new FaviconsWebpackPlugin({
        logo: './res/img/sudoku_icon_full_size.png',
        cache: true,
        favicons: {
          icons: {
            android: false,
            appleIcon: false,
            appleStartup: false,
            coast: false,
            favicons: true,
            firefox: false,
            windows: false,
            yandex: false
          }
        }
      }),
      // new BundleAnalyzerPlugin(),
      ...extraPlugins
    ],
    module: {
      rules: [
        {
          test: /\.tsx?$/,
          use: [{loader: "ts-loader", options: {compilerOptions: {noEmit: false}}}]
        },
        {
          test: /\.css$/,
          use: [{loader: 'style-loader'}, {loader: 'css-loader'}],
        },
      ]
    },
    optimization
  };
};