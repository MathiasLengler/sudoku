/// <reference types="vitest/config" />

import { optimizeLodashImports } from "@optimize-lodash/rollup-plugin";
import { minimal2023Preset } from "@vite-pwa/assets-generator/config";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import jotaiDebugLabel from "jotai/babel/plugin-debug-label";
import jotaiReactRefresh from "jotai/babel/plugin-react-refresh";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";
import wasm from "vite-plugin-wasm";

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
    esbuild: {
        supported: {
            "top-level-await": true,
        },
        ...(mode === "profile" && {
            minifyIdentifiers: false, // makes Chrome DevTools easier to use
        }),
    },
    server: {
        fs: {
            // Allow serving files from one level up to the project root
            allow: [".."],
        },
        headers: {
            "Cross-Origin-Embedder-Policy": "require-corp",
            "Cross-Origin-Opener-Policy": "same-origin",
        },
        watch: {
            ignored: [
                // Defer watching of wasm-pack build output.
                // Background:
                // wasm-bindgen-rayon imports the wasm package itself from "snippets/".
                // "wasm-pack build" first updates the package.json, then emits those snippets, then finishes the containing wasm package.
                // If the vite watcher reloads between those points, the vite reload fails with:
                //   Failed to resolve import "../../.."
                // As a workaround, we ignore everything inside the wasm package,
                // except the top-level entry points (sudoku_wasm.js and similar).
                "**/sudoku-wasm/pkg/**",
                "!**/sudoku-wasm/pkg/sudoku_wasm.*",
            ],
        },
        allowedHosts: [".goat-snapper.ts.net"],
    },
    worker: {
        format: "es",
    },
    plugins: [
        react({ babel: { plugins: [jotaiDebugLabel, jotaiReactRefresh] } }),
        wasm(),
        optimizeLodashImports(),
        ...(mode !== "test"
            ? [
                  VitePWA({
                      strategies: "generateSW",
                      registerType: "autoUpdate",
                      devOptions: {
                          enabled: true,
                          navigateFallbackAllowlist: [/^\/$/],
                      },
                      filename: "service-worker.js",
                      manifestFilename: "manifest.json",
                      manifest: {
                          name: "Sudoku",
                          short_name: "Sudoku",
                          orientation: "natural",
                          description: "Sudoku: design your own difficulty",
                          theme_color: "#121212",
                          background_color: "#121212",
                      },
                      workbox: {
                          globPatterns: ["**/*.{js,wasm,css,html,png,svg,ico,woff2}"],
                          // We currently don't have a SPA router
                          navigateFallbackAllowlist: [/^\/$/],
                          maximumFileSizeToCacheInBytes: 20 * 1024 * 1024, // 20 MiB
                      },
                      pwaAssets: {
                          preset: {
                              ...minimal2023Preset,
                              maskable: {
                                  ...minimal2023Preset.maskable,
                                  resizeOptions: {
                                      background: "#121212",
                                  },
                              },
                              apple: {
                                  ...minimal2023Preset.apple,
                                  resizeOptions: {
                                      background: "#121212",
                                  },
                              },
                          },
                          image: "public/icon_dark.png",
                          injectThemeColor: false,
                      },
                  }),
              ]
            : []),
    ],
    resolve: {
        alias: {
            lodash: "lodash-es",
            ...(mode === "profile" && {
                "react-dom/client": "react-dom/profiling",
            }),
        },
    },
    test: {
        browser: {
            enabled: true,
            provider: playwright(),
            headless: false,
            instances: [{ browser: "chromium" }],
        },
    },
}));
