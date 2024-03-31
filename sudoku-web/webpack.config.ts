import ReactRefreshPlugin from "@pmmmwh/react-refresh-webpack-plugin";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import CopyPlugin from "copy-webpack-plugin";
import ForkTsCheckerWebpackPlugin from "fork-ts-checker-webpack-plugin";
import HtmlWebpackPlugin from "html-webpack-plugin";
import path from "path";
import webpack from "webpack";
import "webpack-dev-server";
import WorkboxPlugin from "workbox-webpack-plugin";

import _ from "lodash";

const dist = path.resolve(__dirname, "dist");
export default async (
    env: Record<string, string | undefined>,
    { mode }: { mode: webpack.Configuration["mode"] },
): Promise<webpack.Configuration> => {
    const reactProfiling = !!env.reactProfiling;
    const bundleAnalyzer = !!env.bundleAnalyzer;
    const hostAny = !!env.hostAny;
    const debugSW = !!env.debugSW;

    const isDevelopment = mode === "development";
    const isProduction = mode === "production";
    if (!isDevelopment && !isProduction) {
        throw new Error(`Unexpected mode: ${mode}`);
    }
    const swEnabled = (isProduction && !reactProfiling) || debugSW;

    const devtool = isProduction ? "source-map" : "eval-source-map";

    const alias = {
        ...(reactProfiling
            ? {
                  "react-dom$": "react-dom/profiling",
                  "scheduler/tracing": "scheduler/tracing-profiling",
              }
            : undefined),
        "workbox-window": "workbox-window/Workbox.mjs",
    };

    const optimization = reactProfiling
        ? {
              minimize: false,
          }
        : {};

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
            static: {
                directory: dist,
            },
            client: {
                overlay: {
                    // ReactRefreshPlugin has its own overlay
                    errors: false,
                    warnings: false,
                    runtimeErrors: false,
                },
            },
            host: hostAny ? "0.0.0.0" : "127.0.0.1",
            hot: true,
            headers: {
                "Cross-Origin-Embedder-Policy": "require-corp",
                "Cross-Origin-Opener-Policy": "same-origin",
            },
        },
        devtool,
        resolve: {
            extensions: [".ts", ".tsx", ".js", ".wasm"],
            alias,
        },
        experiments: {
            asyncWebAssembly: true,
            topLevelAwait: true,
        },
        plugins: _.compact([
            isDevelopment && new ReactRefreshPlugin(),
            new ForkTsCheckerWebpackPlugin(),
            new HtmlWebpackPlugin({
                template: path.resolve(__dirname, "res", "index.html"),
                favicon: "",
            }),
            new WasmPackPlugin({
                crateDirectory: path.resolve(__dirname, "../sudoku-wasm"),
                watchDirectories: [path.resolve(__dirname, "../sudoku-rs")],
                outDir: path.resolve(__dirname, "../sudoku-wasm/pkg"),
                extraArgs: "--target web . -- -Z build-std=panic_abort,std",
                // webpack currently doesn't support wasm reference types: https://github.com/webpack/webpack/issues/15566
                // extraArgs: "--reference-types",
            }),
            // PWA
            swEnabled &&
                new WorkboxPlugin.GenerateSW({
                    maximumFileSizeToCacheInBytes: Math.pow(10, 8),
                }),
            new webpack.DefinePlugin({
                "process.env.SW_ENABLED": swEnabled,
            }),
            new CopyPlugin({ patterns: ["res/public"] }),
            bundleAnalyzer && new (await import("webpack-bundle-analyzer")).BundleAnalyzerPlugin(),
        ]),
        module: {
            rules: [
                {
                    test: /\.tsx?$/,
                    use: {
                        loader: "babel-loader",
                        options: {
                            presets: [
                                [
                                    "@babel/preset-env",
                                    {
                                        targets:
                                            "> .5% and last 2 versions and supports es6-module and supports wasm and supports async-functions",
                                    },
                                ],
                                "@babel/preset-typescript",
                                // Enable development transform of React with new automatic runtime
                                ["@babel/preset-react", { development: !isProduction, runtime: "automatic" }],
                            ],
                            // Applies the react-refresh Babel plugin on non-production modes only
                            ...(!isProduction && { plugins: ["react-refresh/babel"] }),
                        },
                    },
                },
                {
                    test: /\.css$/,
                    use: [{ loader: "style-loader" }, { loader: "css-loader" }],
                },
            ],
        },
        optimization,
    };
};
