const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WebpackPwaManifest = require("webpack-pwa-manifest");
const WorkboxPlugin = require("workbox-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const FaviconsWebpackPlugin = require("favicons-webpack-plugin");
const CopyPlugin = require("copy-webpack-plugin");
const ReactRefreshPlugin = require("@pmmmwh/react-refresh-webpack-plugin");
const ForkTsCheckerWebpackPlugin = require("fork-ts-checker-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = (env, argv) => {
    const { mode } = argv;

    const isDevelopment = mode === "development";
    let isProduction = mode === "production";
    if (!isDevelopment && !isProduction) {
        console.warn(`Unexpected mode: ${mode}`);
        isProduction = true;
    }

    const devtool = isProduction ? "source-map" : "eval-source-map";

    const reactProfiling = !!env.reactProfiling;
    const bundleAnalyzer = !!env.bundleAnalyzer;
    const hostAny = !!env.hostAny;

    const alias = reactProfiling
        ? {
              "react-dom$": "react-dom/profiling",
              "scheduler/tracing": "scheduler/tracing-profiling",
          }
        : {};

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
                    errors: true,
                    warnings: false,
                },
            },
            host: hostAny ? "0.0.0.0" : "127.0.0.1",
            hot: true,
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
        plugins: [
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
            }),
            // PWA
            isProduction &&
                !reactProfiling &&
                new WorkboxPlugin.GenerateSW({
                    clientsClaim: true,
                    skipWaiting: true,
                    maximumFileSizeToCacheInBytes: Math.pow(10, 8),
                }),
            new WebpackPwaManifest({
                name: "Sudoku",
                short_name: "Sudoku",
                description: "Touch optimized sudoku built with Rust/WASM/TypeScript/React",
                // MUI Theme: prefersDarkMode && palette.background.default
                background_color: "#121212",
                // CSS: (prefers-color-scheme: dark) var(--cell-bg-color-selected)
                theme_color: "#042143",
                icons: [
                    {
                        src: path.resolve("res/img/icon_dark.png"),
                        sizes: [96, 128, 192, 256, 384, 512],
                        destination: "assets",
                    },
                    {
                        src: path.resolve("res/img/icon_dark_maskable.png"),
                        sizes: [192, 512],
                        purpose: "maskable",
                        destination: "assets",
                    },
                ],
            }),
            new FaviconsWebpackPlugin({
                logo: "./res/img/icon_light.png",
                cache: true,
                favicons: {
                    icons: {
                        android: false,
                        appleIcon: ["apple-touch-icon-180x180.png"],
                        appleStartup: false,
                        coast: false,
                        favicons: true,
                        firefox: false,
                        windows: false,
                        yandex: false,
                    },
                },
            }),
            new CopyPlugin({ patterns: ["res/public"] }),
            bundleAnalyzer && new (require("webpack-bundle-analyzer").BundleAnalyzerPlugin)(),
        ].filter(Boolean),
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
