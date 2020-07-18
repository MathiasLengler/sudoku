/* eslint-disable */
const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");

const dist = path.resolve(__dirname, "dist");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require('webpack');
// const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;
const WorkerPlugin = require('worker-plugin');

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
      filename: "app.js"
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
    plugins: [
      new HtmlWebpackPlugin({
        template: path.resolve(__dirname, "res", "index.html")
      }),
      new WorkerPlugin({
        globalObject: false
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "../sudoku-wasm"),
        watchDirectories: [
          path.resolve(__dirname, "../sudoku-rs")
        ],
        outDir: path.resolve(__dirname, "../sudoku-wasm/pkg"),
        extraArgs: "--profiling"
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