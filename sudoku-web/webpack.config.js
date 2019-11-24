/* eslint-disable */
const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");

const dist = path.resolve(__dirname, "dist");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require('webpack');
// const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;

// TODO: fix duplicate WasmPackPlugin (gets executed twice)

const appConfig = {
  name: "app",
  entry: "./src/index.tsx",
  output: {
    path: dist,
    filename: "app.js"
  },
  devServer: {
    contentBase: dist,
    host: '0.0.0.0',
    hot: true
  },
  resolve: {
    extensions: [".ts", ".tsx", ".js", ".wasm"]
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: path.resolve(__dirname, "res", "index.html")
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "../sudoku-wasm"),
      watchDirectories: [
        path.resolve(__dirname, "../sudoku-rs")
      ],
      outDir: path.resolve(__dirname, "../sudoku-wasm/pkg")
    }),
    // new BundleAnalyzerPlugin(),
    new webpack.HotModuleReplacementPlugin()
  ],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        loader: "ts-loader"
      },
      {
        test: /\.css$/,
        use: [{loader: 'style-loader'}, {loader: 'css-loader'}],
      },
    ]
  },
};

const workerConfig = {
  name: "worker",
  target: 'webworker',
  entry: "./src/worker.tsx",
  output: {
    path: dist,
    filename: "worker.js"
  },
  resolve: {
    extensions: [".ts", ".tsx", ".js", ".wasm"]
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "../sudoku-wasm"),
      watchDirectories: [
        path.resolve(__dirname, "../sudoku-rs")
      ],
      outDir: path.resolve(__dirname, "../sudoku-wasm/pkg")
    })
  ],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        loader: "ts-loader"
      }
    ]
  }
};


module.exports = [appConfig, workerConfig];